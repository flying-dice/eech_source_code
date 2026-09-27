#!/usr/bin/env python3
# Follows what happens when an aircraft on a SUPPLY task is destroyed, from a
# run's structured observations (campaign.lua observe=... observe_supply=1;
# lua/observations.lua), and sorts every fact into observed, inferred and
# not observable:
#
#   1 loss        the aircraft, its group, its task and state, where and when
#                 it died; the weapons that disappeared at it in the same
#                 sample, with their launchers (the hit itself is inferred)
#   2 cargo       the group's pick-ups before the loss (one-crate drops with
#                 the group over the supplier) and which aircraft were over
#                 it. Crates are not reported, so whether the lost aircraft
#                 carried one is not observable; by the code a crate aboard is
#                 destroyed with the wreck (hc_dstry.c destroy_local_family),
#                 whose removal is observed
#   3 response    the surviving aircraft: their task and state after the
#                 loss, their farthest point from the supplier, the end of the
#                 task, and any delivery; the group's membership afterwards
#   4 receiver    keysites request a resource only at or below 75
#                 (keysite.c); if no keysite was requesting it when the task
#                 was assigned (apart from the supplier), the requester was a
#                 group (group.c assess_group_supplies), whose supply is not
#                 reported
#   5 subsequent  the group's next SUPPLY assignment and its outcome, and
#                 other SUPPLY flights through the loss area afterwards
#
# The check passes when the loss, the group's response and its subsequent
# assignment (or the group's end) are observed.
#
# Usage: tools/loss-chain-check.py <observations.jsonl[.gz]> [--unit <id>] [--report <file.json>]
#   --unit: the lost aircraft (default: the first SUPPLY aircraft destroyed in flight)
import argparse, bisect, collections, gzip, json, math, sys

CRATE = 10.0
REQUEST_THRESHOLD = 75.0
OVER = 500.0            # m: an aircraft over a keysite (waypoints are overflown within about 350 m)
HIT = 1000.0            # m: a weapon's last position from the aircraft when both vanish in one sample
GROUND_STATES = {'Waiting', 'Taxiing', 'Repairing', 'Rearming', 'Refueling', 'Preparing for takeoff'}


def records(path):
    f = gzip.open(path, 'rt', encoding='utf-8') if path.endswith('.gz') else open(path, encoding='utf-8')
    for line in f:
        yield json.loads(line)


def main():
    p = argparse.ArgumentParser()
    p.add_argument('observations')
    p.add_argument('--unit', type=int)
    p.add_argument('--report')
    a = p.parse_args()

    header, keysites = {}, {}
    history = collections.defaultdict(list)      # id -> [(t, appear/retype record)]
    tasks = collections.defaultdict(list)         # unit -> task events
    track = collections.defaultdict(list)         # unit -> track points
    supply, deaths, gone = [], [], collections.defaultdict(list)
    members = collections.defaultdict(set)        # snapshot t -> {(group, unit)}
    keysite_levels = collections.defaultdict(dict)  # snapshot t -> keysite id -> (ammo, fuel)
    for r in records(a.observations):
        ev = r['ev']
        if ev == 'header':
            header = r
        elif ev in ('appear', 'retype'):
            history[r['id']].append((r['t'], r))
            if r['kind'] == 'keysite':
                keysites[r['id']] = r
        elif ev == 'task':
            tasks[r['id']].append(r)
        elif ev == 'track':
            track[r['id']].append(r)
        elif ev == 'supply':
            supply.append(r)
        elif ev == 'alive' and not r['alive']:
            deaths.append(r)
        elif ev == 'gone':
            gone[r['t']].append(r)
        elif ev == 'snap':
            if r['kind'] == 'keysite':
                keysite_levels[r['t']][r['id']] = (r['ammo'], r['fuel'])
            elif r.get('group') is not None:
                members[r['t']].add((r['group'], r['id']))
    if 'supply_detail' not in header:
        sys.exit('no supply detail in these observations: run campaign.lua with observe_supply=1')
    snap_times = sorted(members)
    ks_times = sorted(keysite_levels)

    def record_at(i, t):
        rs = [r for tt, r in history[i] if tt <= t]
        return rs[-1] if rs else None

    def task_at(i, t):
        """the unit's task event before t (at its death sample it leaves its group: that change is not its state before)"""
        es = [e for e in tasks[i] if e['t'] < t]
        return es[-1] if es else None

    def group_members(g, t, after=False):
        ts = [s for s in snap_times if (s > t if after else s <= t)]
        s = (ts[0] if after else ts[-1]) if ts else None
        return s, sorted(u for gg, u in members.get(s, ()) if gg == g)

    def dist2(p, k):
        return math.dist((p['x'], p['z']), (k['x'], k['z']))

    # the SUPPLY aircraft destroyed, and the one to follow
    losses = []
    for d in deaths:
        e = task_at(d['id'], d['t'])
        rec = record_at(d['id'], d['t'])
        if e and e.get('task') == 'TASK_SUPPLY' and rec and rec['kind'] in ('helicopter', 'fixed_wing'):
            losses.append((d, e, rec))
    if a.unit:
        chosen = next((x for x in losses if x[0]['id'] == a.unit), None)
    else:
        chosen = next((x for x in losses if x[1].get('state') not in GROUND_STATES), None)
    if not chosen:
        sys.exit('no such SUPPLY aircraft loss in these observations')
    d, e, rec = chosen
    t, unit, g = d['t'], d['id'], e.get('group')
    pos = (d['x'], d['y'], d['z'])

    # 1 the loss and the weapons that vanished at it in the same sample
    weapons = []
    for w in gone.get(t, []):
        if w['kind'] == 'weapon' and math.dist((w['x'], w['y'], w['z']), pos) <= HIT:
            wr = record_at(w['id'], t)
            lau = record_at(wr['group'], wr['t']) if wr and wr.get('group') is not None else None
            weapons.append({'weapon': w['id'], 'type': wr and wr['type'], 'side': wr and wr['side'], 'launched_t': wr and wr['t'],
                            'launcher': lau and {'id': wr['group'], 'kind': lau['kind'], 'type': lau['type'], 'callsign': lau.get('name')},
                            'last_seen_m_from_aircraft': round(math.dist((w['x'], w['y'], w['z']), pos))})

    # the group's SUPPLY period: from its last change to TASK_SUPPLY before the loss
    period_start = None
    for ev in sorted((x for u in tasks for x in tasks[u] if x.get('group') == g and x['t'] <= t), key=lambda x: x['t']):
        if ev.get('task') == 'TASK_SUPPLY' and period_start is None:
            period_start = ev['t']
        elif ev.get('task') != 'TASK_SUPPLY':
            period_start = None
    before_t, before = group_members(g, t)
    after_t, after = group_members(g, t, after=True)
    survivors = [u for u in before if u != unit]

    # 2 pick-ups by the group in this period, with who was over the supplier
    pickups = []
    for s in supply:
        if period_start is not None and period_start <= s['t'] <= t:
            for res in ('ammo', 'fuel'):
                if abs((s['from_' + res] - s[res]) - CRATE) < 0.05:
                    k = keysites[s['id']]
                    over = [{'aircraft': u, 'horizontal_m': round(dist2(q, k))} for u in before for q in track[u] if q['t'] == s['t'] and dist2(q, k) <= OVER]
                    if over:
                        pickups.append({'t': s['t'], 'supplier': {'id': s['id'], 'name': k.get('name'), 'type': k['type']}, 'resource': res,
                                        'level': [round(s['from_' + res], 1), round(s[res], 1)], 'aircraft_over_supplier': over})
    wreck_removed = next((x['t'] for tt in sorted(gone) if tt > t for x in gone[tt] if x['id'] == unit), None)

    # 3 the survivors' response
    supplier = keysites[pickups[0]['supplier']['id']] if pickups else None
    response = []
    task_end = None
    for u in survivors:
        evs = [x for x in tasks[u] if x['t'] > t]
        end = next((x for x in evs if x.get('task') != 'TASK_SUPPLY'), None)
        pts = [q for q in track[u] if q['t'] >= t and (end is None or q['t'] <= end['t'])]
        far = max(pts, key=lambda q: dist2(q, supplier)) if pts and supplier else None
        near_end = min(((dist2(end, k), k.get('name'), k['type']) for k in keysites.values()), default=None) if end else None
        response.append({'aircraft': u, 'states_after_loss': sorted({x.get('state') for x in evs if end is None or x['t'] < end['t']} | ({pts[0]['state']} if pts else set())),
                         'farthest_from_supplier': far and {'t': far['t'], 'm': round(dist2(far, supplier)), 'seconds_after_loss': round(far['t'] - t, 2)},
                         'task_ended': end and {'t': end['t'], 'task': end.get('task'), 'state': end.get('state'),
                                                'nearest_keysite': near_end and {'name': near_end[1], 'type': near_end[2], 'm': round(near_end[0])}}})
        if end and (task_end is None or end['t'] > task_end):
            task_end = end['t']
    deliveries_by_group = []
    for s in supply:
        if t < s['t'] <= (task_end or t):
            for res in ('ammo', 'fuel'):
                if s[res] - s['from_' + res] >= 5 and s[res] >= 99.0:
                    k = keysites[s['id']]
                    if any(q['t'] == s['t'] and dist2(q, k) <= OVER for u in survivors for q in track[u]):
                        deliveries_by_group.append({'t': s['t'], 'receiver': k.get('name'), 'resource': res})
    later_members = sorted({u for s in snap_times if s > t for gg, u in members[s] if gg == g})
    replacements = [u for u in later_members if u not in before]

    # 4 the receiver: which keysites were requesting the resource when the task was assigned
    resource = pickups[0]['resource'] if pickups else None
    receiver = {'status': 'not observable', 'why': 'no pick-up observed, so the resource is unknown'}
    if resource and period_start is not None:
        i = bisect.bisect_right(ks_times, period_start) - 1
        ts = ks_times[max(i, 0)]
        side = rec['side']
        requesting = [{'name': keysites[k].get('name'), 'type': keysites[k]['type'], 'level': round(v[0 if resource == 'ammo' else 1], 1)}
                      for k, v in keysite_levels[ts].items() if keysites[k]['side'] == side and v[0 if resource == 'ammo' else 1] <= REQUEST_THRESHOLD]
        others = [x for x in requesting if not supplier or x['name'] != supplier.get('name')]
        receiver = {'resource': resource, 'keysites_requesting_at_assignment': requesting, 'snapshot_t': ts}
        if not others:
            receiver.update(status='inferred: a group',
                            why=f'no {side} keysite other than the supplier had {resource} at or below {REQUEST_THRESHOLD:g} when the task was '
                                'assigned (keysites request only then, keysite.c); the other requesters are groups (group.c '
                                'assess_group_supplies), whose supply levels are not reported')
        elif len(others) == 1:
            receiver.update(status='inferred: the only keysite requesting it', keysite=others[0])
        else:
            receiver.update(status='not identifiable', why='several keysites were requesting it')

    # 5 subsequent decisions: the group's next SUPPLY assignment and its outcome; other SUPPLY flights through the area
    nxt = None
    if task_end is not None:
        for u in survivors:
            ev = next((x for x in tasks[u] if x['t'] > task_end and x.get('task') == 'TASK_SUPPLY'), None)
            if ev and (nxt is None or ev['t'] < nxt['t']):
                nxt = ev
    outcome = None
    if nxt:
        for s in supply:
            if s['t'] > nxt['t']:
                for res in ('ammo', 'fuel'):
                    k = keysites[s['id']]
                    if s[res] - s['from_' + res] >= 5 and s[res] >= 99.0 and any(q['t'] == s['t'] and dist2(q, k) <= OVER for u in survivors for q in track[u]):
                        outcome = outcome or {'delivery_t': s['t'], 'receiver': k.get('name'), 'receiver_type': k['type'], 'resource': res,
                                              'level': [round(s['from_' + res], 1), round(s[res], 1)]}
    area = collections.defaultdict(lambda: [math.inf, None])
    for u, pts in track.items():
        for q in pts:
            if t < q['t'] <= t + 1800 and q['group'] != g:
                dd = math.dist((q['x'], q['z']), (pos[0], pos[2]))
                if dd < area[q['group']][0]:
                    area[q['group']] = [dd, q['t']]
    through = sorted(({'group': gg, 'closest_m': round(v[0]), 't': v[1]} for gg, v in area.items() if v[0] <= 5000), key=lambda x: x['t'])

    at_death = next((x for x in tasks[unit] if x['t'] == t), None)
    report = {
        'observations': a.observations,
        'loss': {'t': t, 'aircraft': unit, 'type': rec['type'], 'side': rec['side'], 'group': g, 'callsign': rec.get('name'),
                 'task': e.get('task'), 'state': e.get('state'), 'task_state_since_t': e['t'], 'position': [round(x, 1) for x in pos],
                 'at_death_sample': at_death and {'group': at_death.get('group'), 'task': at_death.get('task'), 'state': at_death.get('state')},
                 'weapons_vanished_at_it': weapons},
        'group': {'members_before': before, 'members_before_snapshot_t': before_t, 'members_after': after, 'members_after_snapshot_t': after_t,
                  'replacements_later_in_run': replacements, 'supply_period_started': period_start},
        'cargo': {'pickups_this_period': pickups, 'lost_aircraft_wreck_removed_t': wreck_removed},
        'response': {'survivors': response, 'deliveries_by_group_before_task_end': deliveries_by_group},
        'receiver': receiver,
        'subsequent': {'group_next_supply_assignment': nxt and {'t': nxt['t'], 'aircraft': nxt['id'], 'state': nxt.get('state')},
                       'its_first_delivery': outcome, 'other_supply_groups_within_5km_in_next_30min': through},
        'other_supply_losses_in_run': [{'t': x[0]['t'], 'aircraft': x[0]['id'], 'type': x[2]['type'], 'side': x[2]['side'], 'group': x[1].get('group'),
                                        'state': x[1].get('state')} for x in losses],
    }
    report['classification'] = {
        'observed': [
            'the aircraft, its group, TASK_SUPPLY and its state, its position and death, and the sample of its death',
            'the weapons that vanished at that sample within 1 km of it, their types, launch times and launchers',
            'the one-crate drops at the supplier with the group over it, and the distance of each aircraft',
            "the group's membership before and after, and the absence of replacements",
            "the survivors' task and state, their farthest point from the supplier, the end of their task and any delivery",
            'the keysite levels of the resource at the snapshot before the assignment',
            "the group's next SUPPLY assignment and its first delivery",
        ],
        'inferred': [
            'that the weapons hit the aircraft (they vanished at it in the sample it died)',
            'the receiver, from the keysite levels and the code (see receiver.why)',
        ],
        'not_observable': [
            'crates, and so whether the lost aircraft carried one (crates are not reported)',
            "the task's own supplier and receiver",
            "a group requester's identity and supply level, and whether a crate reached it (a drop-off to a group changes nothing reported)",
            'the request message itself, and why the survivors turned back',
        ],
        'code': [
            'a crate goes aboard the aircraft that reaches the pick-up (mb_msgs.c response_to_waypoint_pick_up_reached)',
            'a crate aboard a destroyed helicopter is destroyed with its wreck (hc_dstry.c destroy_local_family)',
        ],
    }
    ok = bool(weapons is not None and before and (survivors == [] or response) and (not survivors or task_end is not None) and (not survivors or nxt))
    report['pass'] = ok
    print(json.dumps(report, indent=1))
    if a.report:
        with open(a.report, 'w', newline='\n') as f:
            json.dump(report, f, indent=1)
            f.write('\n')
    print(f"loss chain of aircraft {unit} (group {g}) at {t}: " + ('PASS' if ok else 'FAIL'), file=sys.stderr)
    return 0 if ok else 1


if __name__ == '__main__':
    sys.exit(main())
