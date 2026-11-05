#!/usr/bin/env python3
# Follows campaign resupply chains through a run's structured observations
# (campaign.lua observe=... observe_supply=1; lua/observations.lua), from the
# public objects the engine reports, and checks one complete chain:
#
#   1 campaign   a group's primary task becomes TASK_SUPPLY (the assignment)
#   2 physical   the group's aircraft flies, continuously and at plausible
#                speed, to the supplier and on to the receiver
#   3 physical   at the pick-up, an aircraft of the group is over the supplier
#      campaign  in the same sample the supplier's ammo or fuel drops by one
#                crate (CARGO_*_SIZE, 10: ks_msgs.c pick-up)
#   4 physical   at the drop-off, an aircraft of the group is over the receiver
#      campaign  in the same sample the receiver's level of that resource goes
#                to 100 (ks_msgs.c drop-off)
#   5 campaign   afterwards the delivered stock is used by the campaign: other
#                SUPPLY tasks pick up from it (one crate, with a SUPPLY
#                aircraft over it), its other decreases are, by the code,
#                groups rearming/refuelling from it (group.c
#                assess_group_supplies; not attributable from the
#                observations), and once it falls to the request threshold
#                (75, keysite.c) the keysite is resupplied again
#
# The supply task's own record (its supplier and receiver) is not among the
# public observations: the chain is attributed from where the group is when
# the supply levels change, at the same 1-second sample, within --radius
# (500 m; the waypoints are overflown within about 100 m). A group can fly one
# SUPPLY task after another without its task changing, so a chain's pick-up is
# searched after the group's previous delivery; it is the first one-crate drop
# at another keysite with the group over it. A pick-up at a supplier with no
# crate takes nothing and leaves no step (mb_msgs.c), and a crate dropped off
# to a group stays aboard and can be delivered on a later task, so a delivery
# of such a carried-over crate is reported incomplete. Every delivery in the run is listed;
# the check passes when at least one chain is complete and the deliveries
# agree with the metrics' resupplied counts.
#
# Usage: tools/supply-chain-check.py <observations.jsonl[.gz]> <metrics.json> [--report <file.json>] [--radius <m>]
import argparse, collections, gzip, json, math, sys

CRATE = 10.0            # CARGO_AMMO_SIZE, CARGO_FUEL_SIZE (cargo.h)
REQUEST_THRESHOLD = 75.0   # KEYSITE_SUPPLY_REQUEST_THRESHOLD (en_suply.h)
MAX_SPEED = {'helicopter': 200, 'fixed_wing': 900}


def records(path):
    f = gzip.open(path, 'rt', encoding='utf-8') if path.endswith('.gz') else open(path, encoding='utf-8')
    for line in f:
        yield json.loads(line)


def main():
    p = argparse.ArgumentParser()
    p.add_argument('observations')
    p.add_argument('metrics')
    p.add_argument('--report')
    p.add_argument('--radius', type=float, default=500.0, help='horizontal metres: an aircraft "over" a keysite')
    a = p.parse_args()

    header, keysites, units = {}, {}, {}
    supply = []                                   # keysite supply steps
    tasks = collections.defaultdict(list)         # group -> task events of its members
    track = collections.defaultdict(list)         # group -> track points of its members
    at = collections.defaultdict(list)            # time -> track points (every SUPPLY-task aircraft)
    for r in records(a.observations):
        ev = r['ev']
        if ev == 'header':
            header = r
        elif ev in ('appear', 'retype'):
            if r['kind'] == 'keysite':
                keysites[r['id']] = r
            else:
                units[r['id']] = r
        elif ev == 'supply':
            supply.append(r)
        elif ev == 'task' and r.get('group') is not None:
            tasks[r['group']].append(r)
        elif ev == 'track':
            track[r['group']].append(r)
            at[r['t']].append(r)
    if 'supply_detail' not in header:
        sys.exit('no supply detail in these observations: run campaign.lua with observe_supply=1')

    def over(t, keysite):
        """SUPPLY-task aircraft over a keysite at a sample: [(distance, point)]"""
        k = keysites[keysite]
        return sorted((math.dist((q['x'], q['z']), (k['x'], k['z'])), q) for q in at.get(t, []) if math.dist((q['x'], q['z']), (k['x'], k['z'])) <= a.radius)

    def ks(i):
        k = keysites[i]
        return {'id': i, 'name': k.get('name'), 'type': k['type'], 'side': k['side']}

    # deliveries: a level going up to 100 (less what the keysite consumed in the rest of that sample)
    def delivered(q, res):
        return q[res] - q['from_' + res] >= 5 and q[res] >= 99.0
    deliveries = []
    for s in supply:
        for res in ('ammo', 'fuel'):
            if delivered(s, res):
                deliveries.append((s, res))

    chains, unattributed = [], []
    for s, res in deliveries:
        t, receiver = s['t'], s['id']
        cand = over(t, receiver)
        groups = sorted({q['group'] for _, q in cand})
        if len(groups) != 1:
            unattributed.append({'t': t, 'receiver': ks(receiver), 'resource': res, 'supply_groups_over_it': groups})
            continue
        g = groups[0]
        dist, member = cand[0]
        # the assignment: the latest change of the group's task to TASK_SUPPLY before the delivery
        assigned = None
        for e in sorted(tasks[g], key=lambda e: e['t']):
            if e['t'] > t:
                break
            if e.get('task') == 'TASK_SUPPLY' and (assigned is None or assigned['task_before'] != 'TASK_SUPPLY'):
                assigned = {'t': e['t'], 'state': e.get('state'), 'task_before': 'TASK_SUPPLY'}
            elif e.get('task') != 'TASK_SUPPLY':
                assigned = {'task_before': e.get('task')}
        assigned = assigned if assigned and 't' in assigned else None
        # the task's own record is not observed: a group can fly one SUPPLY task after another without its
        # task changing, so this chain starts after the assignment and after the group's previous delivery
        previous = max((c['delivery']['t'] for c in chains if c['group'] == g and c['delivery']['t'] < t), default=None)
        start = max(x for x in (assigned and assigned['t'], previous) if x is not None) if (assigned or previous) else None
        # the pick-up: the first one-crate drop of this resource, at another keysite, with the group over it
        pickup = None
        for q in supply:
            if start is not None and start <= q['t'] < t and q['id'] != receiver and abs((q['from_' + res] - q[res]) - CRATE) < 0.05:
                over_it = [x for x in over(q['t'], q['id']) if x[1]['group'] == g]
                if over_it:
                    pickup = (q, over_it[0])
                    break
        # the end of the task
        ended = next((e for e in sorted(tasks[g], key=lambda e: e['t']) if e['t'] > t and e.get('task') != 'TASK_SUPPLY'), None)
        # the flight between pick-up and drop-off, from the aircraft's 1-second track
        flight = None
        if pickup:
            pts = [x for x in track[g] if x['id'] == member['id'] and pickup[0]['t'] <= x['t'] <= t]
            pts.sort(key=lambda x: x['t'])
            kind = units.get(member['id'], {}).get('kind')
            segs = list(zip(pts, pts[1:]))
            flown = sum(math.dist((u['x'], u['y'], u['z']), (v['x'], v['y'], v['z'])) for u, v in segs)
            vmax = max((math.dist((u['x'], u['y'], u['z']), (v['x'], v['y'], v['z'])) / (v['t'] - u['t']) for u, v in segs), default=0)
            gap = max((v['t'] - u['t'] for u, v in segs), default=0)
            direct = math.dist((keysites[pickup[0]['id']]['x'], keysites[pickup[0]['id']]['z']), (keysites[receiver]['x'], keysites[receiver]['z']))
            flight = {'aircraft': member['id'], 'kind': kind, 'samples': len(pts), 'largest_gap_s': round(gap, 2),
                      'flown_m': round(flown), 'direct_m': round(direct), 'max_speed_m_s': round(vmax, 1),
                      'altitude_m': [round(min(x['y'] for x in pts)), round(max(x['y'] for x in pts))] if pts else None,
                      'continuous': bool(pts) and gap <= 1.01 and flown >= 0.9 * direct and vmax <= MAX_SPEED.get(kind, 900)}
        # what the campaign does with the delivered stock, up to the next delivery of it
        later = sorted((q for q in supply if q['id'] == receiver and q['t'] > t), key=lambda q: q['t'])
        nxt = next((q for q in later if delivered(q, res)), None)
        draws, pickups_from, below = [], [], None
        for q in later:
            if nxt and q['t'] >= nxt['t']:
                break
            d = q['from_' + res] - q[res]
            if d >= 0.5:
                if abs(d - CRATE) < 0.05 and over(q['t'], receiver):
                    pickups_from.append({'t': q['t'], 'drawn': round(d, 1), 'groups': sorted({x['group'] for _, x in over(q['t'], receiver)})})
                else:
                    # no SUPPLY aircraft over it: by the code, groups rearming/refuelling from the keysite (assess_group_supplies)
                    draws.append({'t': q['t'], 'drawn': round(d, 1)})
            if below is None and q[res] <= REQUEST_THRESHOLD:
                below = q['t']
        member_unit = units.get(member['id'], {})
        chain = {
            'group': g, 'callsign': member_unit.get('name'), 'side': member_unit.get('side'), 'aircraft_type': member_unit.get('type'),
            'resource': res,
            'assigned': assigned and {'t': assigned['t'], 'state': assigned['state']},
            'previous_delivery_by_group': previous,
            'pickup': pickup and {'t': pickup[0]['t'], 'supplier': ks(pickup[0]['id']), 'level': [round(pickup[0]['from_' + res], 1), round(pickup[0][res], 1)],
                                  'aircraft': pickup[1][1]['id'], 'horizontal_m': round(pickup[1][0]), 'altitude_m': round(pickup[1][1]['y'])},
            'delivery': {'t': t, 'receiver': ks(receiver), 'level': [round(s['from_' + res], 1), round(s[res], 1)],
                         'aircraft': member['id'], 'horizontal_m': round(dist), 'altitude_m': round(member['y'])},
            'flight': flight,
            'task_ended': ended and {'t': ended['t'], 'task': ended.get('task'), 'state': ended.get('state')},
            'afterwards': {'other_decreases': draws, 'pickups_by_other_supply_tasks': pickups_from,
                           'at_or_below_request_threshold_s': below, 'next_delivery_s': nxt and nxt['t']},
        }
        chain['complete'] = bool(assigned and pickup and flight and flight['continuous'] and start < pickup[0]['t'] < t)
        chains.append(chain)

    # the metrics count a level rising by 20 or more in a sample (metrics.lua): they must equal the
    # deliveries of 20 or more observed here
    metrics = json.load(open(a.metrics))
    resupplied = {k: v for k, v in metrics['checkpoints'][-1]['counts'].get('resupplied', {}).items() if v}
    observed = collections.Counter(f"{keysites[s['id']]['side']} {keysites[s['id']]['type']} {res}" for s, res in deliveries)
    observed_20 = collections.Counter(f"{keysites[s['id']]['side']} {keysites[s['id']]['type']} {res}" for s, res in deliveries
                                      if s[res] - s['from_' + res] >= 20)
    metrics_ok = observed_20 == collections.Counter(resupplied)

    complete = [c for c in chains if c['complete']]
    report = {
        'observations': a.observations, 'metrics': a.metrics, 'radius_m': a.radius,
        'deliveries': len(deliveries), 'attributed': len(chains), 'complete_chains': len(complete),
        'unattributed_deliveries': unattributed,
        'deliveries_by_keysite_type': dict(sorted(observed.items())),
        'deliveries_of_20_or_more': dict(sorted(observed_20.items())), 'metrics_resupplied': dict(sorted(resupplied.items())),
        'metrics_agree': metrics_ok,
        'featured_chain': complete[0] if complete else None,
        'chains': chains,
    }
    print(json.dumps(report, indent=1))
    if a.report:
        with open(a.report, 'w', newline='\n') as f:
            json.dump(report, f, indent=1)
            f.write('\n')
    ok = bool(complete) and metrics_ok
    print(f"supply chains: {len(complete)} complete of {len(deliveries)} deliveries ({len(unattributed)} unattributed); metrics {'agree' if metrics_ok else 'DISAGREE'}: "
          + ('PASS' if ok else 'FAIL'), file=sys.stderr)
    return 0 if ok else 1


if __name__ == '__main__':
    sys.exit(main())
