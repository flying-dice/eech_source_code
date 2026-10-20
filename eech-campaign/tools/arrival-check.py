#!/usr/bin/env python3
# EEDCS-11: characterises waypoint arrival on SUPPLY tasks from a test-only
# arrival trace (tools/arrival-observe/, EECH_ARRIVAL_TRACE), and optionally
# correlates it with the public path's structured observations
# (lua/observations.lua, observe_supply=1) of the same run.
#
# Claims are predictions from the source (docs/m5-supply-ownership.md),
# written before the observed run, and each is checked against every
# relevant event. A claim that fails is reported, never relaxed here.
# Facts that the source does not predict (which waypoints a SUPPLY route has,
# who was leader, how members stood) are counted, not asserted.
#
# Usage: tools/arrival-check.py <arrivals-<pid>.jsonl> [--observations <observations.jsonl[.gz]>] [--report <report.json>]
# Exit 0 when every claim passes, 1 when one fails, 2 on unusable input.
import argparse, gzip, json, math, sys
from collections import Counter, defaultdict

FOUR = ('WAYPOINT_PICK_UP', 'WAYPOINT_PREPARE_FOR_DROP_OFF', 'WAYPOINT_DROP_OFF', 'WAYPOINT_FINISH_DROP_OFF')
RADIUS = {'ENTITY_TYPE_HELICOPTER': 40.0, 'ENTITY_TYPE_FIXED_WING': 100.0}   # wp_dbase.c, for the four types
CRATE = 10.0                                          # CARGO_AMMO_SIZE, CARGO_FUEL_SIZE
HELI_COHESION = 750.0                                 # hc_msgs.c HELICOPTER_LAST_TO_REACH_DISTANCE
SUCCESS = 3                                           # TASK_COMPLETED_SUCCESS
EPS = 0.002                                           # the trace prints 3 decimals


def key(ref):
    return tuple(ref) if ref else None


def level_of(holder, cargo):
    return holder.get('ammo' if cargo == 'CARGO_AMMO' else 'fuel') if holder else None


def kind_of(member):
    return member.get('type')


def load(path):
    opener = gzip.open if path.endswith('.gz') else open
    with opener(path, 'rt', encoding='utf-8') as f:
        return [json.loads(l) for l in f if l.strip()]


class Claims:
    def __init__(self):
        self.result = {}

    def check(self, name, what, failures, checked):
        self.result[name] = {'claim': what, 'checked': checked, 'verdict': 'FAIL' if failures else 'PASS', 'failures': failures[:20],
                             'failure_count': len(failures)}


def counterfactual_all_members(v):
    """Would the all-valid-members branch (guide.c:282-316) have failed at this
    verdict? Only "definitely fails" is decided, from the member's own state and
    the per-member tests' early outs (fw_msgs.c:409-433; hc_msgs.c:560-590,
    766-788); a member that might pass the fixed-wing pass-through is 'unknown'."""
    leader = key(v.get('task_leader'))
    lead = next((m for m in v['members'] if key(m['id']) == leader), None)
    out = []
    for m in v['members']:
        if key(m['id']) == leader or not (v['valid_members'] >> m['number']) & 1:
            continue
        why = None
        if m.get('player', 0) == 0 and m.get('landed'):
            why = 'landed'
        elif kind_of(m) == 'ENTITY_TYPE_HELICOPTER':
            if m.get('player', 0) == 0 and (m.get('rotor_spin_up') or 0) > 0:
                why = 'rotor not spun up'
            elif lead and lead.get('player', 0) == 0 and m.get('to_leader_2d', 0) > HELI_COHESION:
                why = f"{m['to_leader_2d']:.0f} m from the leader (> {HELI_COHESION:.0f} m)"
        elif kind_of(m) == 'ENTITY_TYPE_FIXED_WING':
            r = v['radius']
            if m.get('distance') is not None and m.get('velocity') is not None and m['distance'] - m['velocity'] > r:
                why = f"{m['distance']:.0f} m from its guide position (> radius {r:.0f} m + one second's travel)"
        out.append({'member': m['id'], 'definitely_fails': why is not None, 'why': why,
                    'distance': m.get('distance'), 'to_leader_2d': m.get('to_leader_2d')})
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('trace')
    ap.add_argument('--observations')
    ap.add_argument('--report')
    a = ap.parse_args()
    events = load(a.trace)
    if not events or events[0].get('e') != 'header':
        print('not an arrival trace (no header)', file=sys.stderr)
        return 2
    counters = next((e for e in events if e['e'] == 'counters'), None)

    tasks = {key(e['task']): e for e in events if e['e'] == 'task'}
    spans = {}
    for e in events:
        if e['e'] == 'verdict':
            spans[e['span']] = {'verdict': e, 'handlers': [], 'completion': [], 'completed': [], 'effect': None, 'freed': []}
        elif e['e'] in ('handler', 'completion', 'completed', 'freed', 'effect') and e.get('span'):
            s = spans.get(e['span'])
            if s is None:
                continue
            if e['e'] == 'effect':
                s['effect'] = e
            else:
                s['handlers' if e['e'] == 'handler' else e['e']].append(e)
    supply_four = [s for s in spans.values() if s['verdict']['supply_task'] and s['verdict']['four']]
    claims = Claims()

    # 1. the path: every response of the four types is inside an observed verdict span, to the span's task leader, for its waypoint, once
    handlers = [e for e in events if e['e'] == 'handler']
    bad = []
    for h in handlers:
        s = spans.get(h['span'])
        if not s:
            bad.append({'t': h['t'], 'handler': h['handler'], 'why': 'outside any arrival span'})
        elif not (h.get('receiver_is_span_leader') and h.get('wp_is_span_wp')):
            bad.append({'t': h['t'], 'span': h['span'], 'why': 'not the span leader or not the span waypoint'})
        elif 'WAYPOINT_' + h['handler'] != s['verdict']['wp_type']:
            bad.append({'t': h['t'], 'span': h['span'], 'why': f"{h['handler']} for a {s['verdict']['wp_type']} waypoint"})
    for s in supply_four:
        if len(s['handlers']) != 1:
            bad.append({'t': s['verdict']['t'], 'span': s['verdict']['span'], 'why': f"{len(s['handlers'])} responses to one verdict"})
    claims.check('path.single_route', 'Every PICK_UP/PREPARE_FOR_DROP_OFF/DROP_OFF/FINISH_DROP_OFF response of a mobile runs inside a guide arrival '
                 'verdict (gd_updt.c -> navigation_guide_waypoint_reached -> response_to_waypoint_reached), exactly once per verdict, '
                 'sent to that verdict\'s task leader for that waypoint', bad, len(handlers) + len(supply_four))

    # 2. the campaign's criteria at the four types: leader only, radius 100 m fixed wing / 40 m helicopter, no action check
    bad = []
    for s in supply_four:
        v = s['verdict']
        lead = next((m for m in v['members'] if key(m['id']) == key(v.get('task_leader'))), None)
        want = RADIUS.get(kind_of(lead) if lead else None)
        if v['last_to_reach'] or v['check_action'] or not v['radius_valid'] or want is None or abs(v['radius'] - want) > EPS:
            bad.append({'span': v['span'], 't': v['t'], 'wp': v['wp_type'], 'leader': kind_of(lead) if lead else None,
                        'last_to_reach': v['last_to_reach'], 'check_action': v['check_action'], 'radius': v['radius']})
    claims.check('criteria.leader_only', 'At the four SUPPLY types the guide\'s criteria are: last-to-reach off (task leader only), '
                 'no waypoint action, reached radius 100 m (fixed-wing leader) or 40 m (helicopter leader)', bad, len(supply_four))

    # 2b. whose physical observation the campaign consumed: at the four types, the task leader's alone, in every check
    bad = []
    for s in supply_four:
        v = s['verdict']
        q = v.get('queried')
        if q is None:
            bad.append({'span': v['span'], 'why': 'the trace does not record the members queried (observer without A11/A12)'})
        elif [key(x['id']) for x in q] != [key(v['task_leader'])] or not q[0]['reached']:
            bad.append({'span': v['span'], 't': v['t'], 'callsign': v.get('callsign'), 'members': len(v['members']), 'queried': q})
    if counters and 'member_queries' in counters and counters['member_queries']['supply_four_other_member']:
        bad.append({'why': f"{counters['member_queries']['supply_four_other_member']} queries of a member other than the task leader "
                           'in checks at the four types (exit counters, all checks)'})
    claims.check('members.leader_only_consumed', 'At the four SUPPLY types every arrival check asks the task leader, and only the task leader, '
                 'for its per-member reach test; each verdict follows the leader\'s own answer, whatever the other members\' state', bad, len(supply_four))

    # 3. the task leader's own state at the verdict is consistent with its per-member test
    bad = []
    for s in supply_four:
        v = s['verdict']
        lead = next((m for m in v['members'] if key(m['id']) == key(v.get('task_leader'))), None)
        why = None
        if not lead:
            why = 'the task leader is not a member of the guide\'s group'
        elif lead.get('landed'):
            why = 'leader landed'
        elif kind_of(lead) == 'ENTITY_TYPE_HELICOPTER' and ((lead.get('rotor_spin_up') or 0) > 0 or lead['distance'] > v['radius'] + EPS):
            why = f"helicopter leader at {lead['distance']} m, spin-up {lead.get('rotor_spin_up')}"
        elif kind_of(lead) == 'ENTITY_TYPE_FIXED_WING' and lead['distance'] - lead['velocity'] > v['radius'] + EPS:
            why = f"fixed-wing leader at {lead['distance']} m, {lead['velocity']} m/s"
        if why:
            bad.append({'span': v['span'], 't': v['t'], 'why': why})
    claims.check('leader.state_at_verdict', 'At each verdict the task leader is in the guide\'s group, not landed, and within the radius '
                 '(helicopter: rotors spun up and range <= radius; fixed wing: range - speed <= radius, the pass-through early out)', bad, len(supply_four))

    # 4. ordering: the guide has moved to its next waypoint before the leader is told (wp_msgs.c:413 before :440)
    bad, checked = [], 0
    for s in supply_four:
        for h in s['handlers']:
            if 'guide_wp_now' in h:
                checked += 1
                if key(h['guide_wp_now']) == key(s['verdict']['wp']):
                    bad.append({'span': h['span'], 't': h['t'], 'why': 'guide still on the reached waypoint when the leader is notified'})
    claims.check('order.guide_advances_first', 'When the leader\'s response runs, its guide already has the next waypoint', bad, checked)

    # 5. pick-up accounting
    bad, outcome = [], Counter()
    for s in [s for s in supply_four if s['verdict']['wp_type'] == 'WAYPOINT_PICK_UP']:
        v, e = s['verdict'], s['effect']
        task = tasks.get(key(v['task']))
        want = task['cargo_type'] if task else None
        before_dep, after_dep = v.get('dependent'), e.get('dependent') if e else None
        before_ids = {key(c['id']) for c in v.get('leader_aboard') or []}
        after = e.get('leader_aboard') if e else None
        if not before_dep or before_dep.get('type') != 'ENTITY_TYPE_KEYSITE' or after is None or not after_dep:
            outcome['not judged (no keysite supplier, or the leader or supplier gone)'] += 1
            continue
        crates = before_dep.get('crates', [])
        gained = [c for c in after if key(c['id']) not in before_ids]
        if not crates:
            outcome['no crate at the supplier: nothing taken'] += 1
            if gained or abs(after_dep['ammo'] - before_dep['ammo']) > EPS or abs(after_dep['fuel'] - before_dep['fuel']) > EPS:
                bad.append({'span': v['span'], 'why': 'no crate, yet the load or the level changed'})
            continue
        typed = [c for c in crates if c[2] == want]
        expect = typed[0] if typed else crates[0]
        outcome['crate of the task\'s type taken' if typed else 'no crate of the task\'s type: the first crate taken'] += 1
        moved = [c for c in gained if key(c['id']) == (expect[0], expect[1])]
        left = {(c[0], c[1]) for c in after_dep.get('crates', [])}
        drop = level_of(before_dep, expect[2]) - level_of(after_dep, expect[2])
        if len(gained) != 1 or not moved or (expect[0], expect[1]) in left or abs(drop - CRATE) > EPS:
            bad.append({'span': v['span'], 't': v['t'], 'why': f"expected crate {expect} onto the leader and -{CRATE} {expect[2]}; "
                        f"gained {[c['id'] for c in gained]}, level change {-drop:.3f}"})
    claims.check('pickup.accounting', 'PICK_UP: with a crate at the supplier, exactly one crate (of the task\'s cargo type when the supplier '
                 'has one, S3; else the first) moves from the supplier onto the task leader and the supplier\'s level of that type drops by 10; '
                 'with none, nothing changes', bad, sum(outcome.values()))
    pickup_outcomes = dict(outcome)

    # 6. drop-off accounting
    bad, outcome, delivered = [], Counter(), {}
    for s in [s for s in supply_four if s['verdict']['wp_type'] == 'WAYPOINT_DROP_OFF']:
        v, e = s['verdict'], s['effect']
        dep, after_dep = v.get('dependent'), e.get('dependent') if e else None
        load_before = v.get('leader_aboard') or []
        load_after = e.get('leader_aboard') if e else None
        ident = [c for c in load_before if c.get('identify_cargo')]
        tk = key(v['task'])
        if not dep:
            outcome['no requester'] += 1
            continue
        if load_after is None or not after_dep:
            outcome['not judged (the leader or requester gone)'] += 1
            continue
        after_ids = {key(c['id']) for c in load_after}
        if not ident:
            outcome[f"nothing aboard the leader ({dep['type']} requester)"] += 1
            delivered[tk] = False
            if after_ids != {key(c['id']) for c in load_before} or abs(after_dep['ammo'] - dep['ammo']) > EPS or abs(after_dep['fuel'] - dep['fuel']) > EPS:
                bad.append({'span': v['span'], 'why': 'nothing aboard, yet the load or the level changed'})
            continue
        delivered[tk] = True
        first = ident[0]
        if dep['type'] == 'ENTITY_TYPE_KEYSITE':
            outcome['keysite requester: first crate delivered' + ('' if len(ident) == 1 else f' ({len(ident) - 1} more stay aboard)')] += 1
            crates_after = {(c[0], c[1]) for c in after_dep.get('crates', [])}
            stayed = [c for c in ident[1:] if key(c['id']) in after_ids]
            if key(first['id']) in after_ids or key(first['id']) not in crates_after or abs(level_of(after_dep, first.get('cargo')) - 100.0) > EPS \
                    or len(stayed) != len(ident) - 1:
                bad.append({'span': v['span'], 't': v['t'], 'why': 'keysite drop-off did not move exactly the first crate and set its level to 100'})
        else:
            outcome['group requester: level raised, crate stays aboard'] += 1
            if after_ids != {key(c['id']) for c in load_before} or level_of(after_dep, first.get('cargo')) < level_of(dep, first.get('cargo')) - EPS:
                bad.append({'span': v['span'], 't': v['t'], 'why': 'group drop-off changed the load or lowered the level'})
    claims.check('dropoff.accounting', 'DROP_OFF: only the task leader\'s load is read; a keysite requester receives the first identified crate '
                 'aboard and its level of that type is set to 100 (any further crates stay aboard); a group requester\'s level rises and the '
                 'crate stays aboard; with nothing aboard nothing changes', bad, sum(outcome.values()))
    dropoff_outcomes = dict(outcome)

    # 7. completion: FINISH_DROP_OFF completes the task as SUCCESS inside its own span, whatever was delivered
    bad, completion_vs_delivery = [], Counter()
    for s in [s for s in supply_four if s['verdict']['wp_type'] == 'WAYPOINT_FINISH_DROP_OFF']:
        v = s['verdict']
        reasons = [c['reason'] for c in s['completion']]
        values = [c['value'] for c in s['completed']]
        if v.get('task_completed', 0) == 0 and (reasons != ['TASK_TERMINATED_OBJECTIVE_MESSAGE'] or values != [SUCCESS]):
            bad.append({'span': v['span'], 't': v['t'], 'reasons': reasons, 'values': values})
        tk = key(v['task'])
        completion_vs_delivery['SUCCESS, delivered at DROP_OFF' if delivered.get(tk) else
                               'SUCCESS, nothing delivered at DROP_OFF' if tk in delivered else 'SUCCESS, no DROP_OFF arrival observed'] += 1
    claims.check('finish.completes_success', 'FINISH_DROP_OFF: inside its span the task receives TASK_COMPLETED (objective message) and '
                 'is set to SUCCESS (when still incomplete)', bad, len([s for s in supply_four if s['verdict']['wp_type'] == 'WAYPOINT_FINISH_DROP_OFF']))

    # 8. identity: no event names an entity generation after it was freed
    bad, freed_at = [], {}
    refs = ('task', 'guide', 'wp', 'group', 'task_leader', 'receiver')
    for e in events:
        if e['e'] == 'freed':
            freed_at[key(e['id'])] = e['t']
            continue
        for r in refs:
            k = key(e.get(r)) if isinstance(e.get(r), list) else None
            if k and k in freed_at:
                bad.append({'t': e['t'], 'event': e['e'], 'ref': r, 'id': list(k), 'freed_at': freed_at[k]})
    claims.check('identity.generations', 'No event names an entity [index, generation] after that generation was freed', bad,
                 sum(1 for e in events if e['e'] != 'freed'))

    # 9. the trace is complete: its counters match its events
    bad = []
    if not counters:
        bad.append({'why': 'no counters (the process did not exit normally)'})
    else:
        n = Counter((e['supply_task'], e['four']) for e in events if e['e'] == 'verdict')
        got = {'supply_four': n[(1, 1)], 'supply_other': n[(1, 0)], 'other_four': n[(0, 1)]}
        for k2, v2 in got.items():
            if counters['verdicts'][k2] != v2:
                bad.append({'counter': k2, 'counted': counters['verdicts'][k2], 'events': v2})
        if counters['handlers']['outside_span']:
            bad.append({'counter': 'handlers outside span', 'value': counters['handlers']['outside_span']})
    claims.check('trace.complete', 'The trace\'s exit counters agree with its events, and no response ran outside a span', bad, 1)

    # facts, counted
    route_types = Counter(' '.join(w['type'] for w in t['waypoints']) or '(none yet)' for t in tasks.values())
    # every waypoint a SUPPLY task's guide was judged to reach, in order: the route as flown
    flown = defaultdict(list)
    for s in sorted((s for s in spans.values() if s['verdict']['supply_task']), key=lambda s: s['verdict']['span']):
        flown[key(s['verdict']['task'])].append(s['verdict']['wp_type'].replace('WAYPOINT_', ''))
    route_flown = Counter(' > '.join(seq) for seq in flown.values())
    verdict_types = Counter((s['verdict']['wp_type'], s['verdict']['last_to_reach'],
                             kind_of(next((m for m in s['verdict']['members'] if key(m['id']) == key(s['verdict'].get('task_leader'))), {})))
                            for s in spans.values() if s['verdict']['supply_task'])
    multi = []
    for s in supply_four:
        v = s['verdict']
        valid_others = [m for m in v['members'] if key(m['id']) != key(v.get('task_leader')) and (v['valid_members'] >> m['number']) & 1]
        if valid_others:
            cf = counterfactual_all_members(v)
            multi.append({'span': v['span'], 't': v['t'], 'callsign': v.get('callsign'), 'group': v['group'], 'wp': v['wp_type'],
                          'task_leader': v['task_leader'], 'group_leader': v.get('group_leader'),
                          'leader_kind': kind_of(next(m for m in v['members'] if key(m['id']) == key(v['task_leader']))),
                          'members': len(v['members']), 'queried': v.get('queried'), 'others': cf,
                          'all_members_would_fail': any(c['definitely_fails'] for c in cf)})
    # the other path on the same routes: last-to-reach waypoints with more than one valid member
    all_members_path = Counter()
    for s in spans.values():
        v = s['verdict']
        if not v['supply_task'] or not v['last_to_reach'] or v.get('queried') is None:
            continue
        valid = {key(m['id']) for m in v['members'] if (v['valid_members'] >> m['number']) & 1}
        asked = {key(x['id']) for x in v['queried']}
        all_members_path[f"{v['wp_type']} | {len(valid)} valid | asked {'all valid members' if asked == valid else str(len(asked))}"] += 1
    lifetimes = Counter()
    for s in [s for s in supply_four if s['verdict']['wp_type'] == 'WAYPOINT_FINISH_DROP_OFF' and s['effect']]:
        e = s['effect']
        lifetimes[f"after FINISH_DROP_OFF: guide {'alive' if e['guide_alive'] else 'freed'}, task {'alive' if e['task_alive'] else 'freed'}, "
                  f"waypoint {'alive' if e['wp_alive'] else 'freed'}"] += 1
    freed_types = Counter(e['type'] for e in events if e['e'] == 'freed')
    chains = Counter()
    by_task = defaultdict(list)
    for s in sorted(supply_four, key=lambda s: s['verdict']['span']):
        by_task[key(s['verdict']['task'])].append(s['verdict']['wp_type'])
    for tk, t in tasks.items():
        chains[' > '.join(by_task.get(tk, [])) or '(no arrival at the four types)'] += 1
    completions_outside = Counter((c['reason']) for c in events if c['e'] == 'completion' and not c.get('span'))
    # every SUPPLY task set to SUCCESS: where, and whether anything was delivered at its DROP_OFF
    freed_t = {key(e['id']): e['t'] for e in events if e['e'] == 'freed'}
    success, success_examples = Counter(), []
    for c in [e for e in events if e['e'] == 'completed' and e['value'] == SUCCESS]:
        tk = key(c['task'])
        where = spans[c['span']]['verdict']['wp_type'].replace('WAYPOINT_', 'in a ') + ' span' if c.get('span') in spans else 'outside any arrival span'
        reason = next((x['reason'] for x in events if x['e'] == 'completion' and key(x['task']) == tk and x['t'] == c['t']), '?')
        how = 'delivered at its DROP_OFF' if delivered.get(tk) else 'nothing delivered at its DROP_OFF' if tk in delivered else 'no DROP_OFF arrival'
        task = tasks.get(tk)
        req = task['requester'] if task else None
        req_freed = req is not None and freed_t.get(key(req['id'])) == c['t']
        label = f"{where}, {reason}, {how}" + (f", requester {req['type'].replace('ENTITY_TYPE_', '').lower()} freed in the same frame" if req_freed else '')
        success[label] += 1
        if how != 'delivered at its DROP_OFF' and len(success_examples) < 10:
            success_examples.append({'task': c['task'], 't': c['t'], 'label': label, 'requester': req and {k: req[k] for k in ('id', 'type', 'name')},
                                     'supplier': task and task['supplier'].get('name')})

    # correlation with the public path's observations
    correlation = None
    if a.observations:
        obs = load(a.observations)
        supply = defaultdict(list)
        track = {}
        site = {}
        for o in obs:
            ev = o.get('ev')
            if ev == 'supply':
                supply[o['id']].append(o)
            elif ev == 'track':
                track[(o['id'], round(o['t'], 2))] = o
            elif ev in ('appear',) and o.get('kind') == 'keysite':
                site[o['id']] = (o['x'], o['z'])
        times = sorted({round(o['t'], 2) for o in obs if 't' in o})
        import bisect
        matched, unmatched, examples, inexact = 0, [], [], []
        for s in supply_four:
            v, e = s['verdict'], s['effect']
            if v['wp_type'] not in ('WAYPOINT_PICK_UP', 'WAYPOINT_DROP_OFF') or not e or not v.get('dependent') or not e.get('dependent') \
                    or v['dependent']['type'] != 'ENTITY_TYPE_KEYSITE':
                continue
            before, after = v['dependent'], e['dependent']
            if abs(before['ammo'] - after['ammo']) <= EPS and abs(before['fuel'] - after['fuel']) <= EPS:
                continue
            kid = before['id'][0]
            # a sample printed with the verdict's own (rounded) time was taken before that frame's update:
            # the consequence is first visible at the next sample strictly after the verdict
            i = bisect.bisect_right(times, round(v['t'], 2) + 1e-6)
            sample = times[i] if i < len(times) else None
            ev = next((o for o in supply[kid] if sample is not None and abs(o['t'] - sample) < 0.006), None)
            # asserted: the change is there, in the same direction, at the next sample; measured: how far the sample's
            # change differs (another campaign change, e.g. a keysite usage tick, can fall in the same 1 s sample)
            residual = None
            ok = ev is not None
            if ev is not None:
                d_obs = {'ammo': ev['ammo'] - ev['from_ammo'], 'fuel': ev['fuel'] - ev['from_fuel']}
                d_trace = {'ammo': after['ammo'] - before['ammo'], 'fuel': after['fuel'] - before['fuel']}
                for r in ('ammo', 'fuel'):
                    if abs(d_trace[r]) > EPS and (d_obs[r] > 0) != (d_trace[r] > 0):
                        ok = False
                residual = round(max(abs(d_obs[r] - d_trace[r]) for r in ('ammo', 'fuel')), 3)
            leader = v['task_leader'][0]
            tr = track.get((leader, sample))
            row = {'span': v['span'], 'wp': v['wp_type'], 't': v['t'], 'sample': sample, 'keysite': before['name'],
                   'trace_change': {'ammo': round(after['ammo'] - before['ammo'], 3), 'fuel': round(after['fuel'] - before['fuel'], 3)},
                   'observed_supply_event': ev, 'residual': residual, 'leader': leader, 'leader_tracked_at_sample': tr is not None,
                   'leader_group_matches': tr is not None and tr.get('group') == v['group'][0],
                   'leader_to_keysite_2d_m': round(math.hypot(tr['x'] - site[kid][0], tr['z'] - site[kid][1]), 1) if tr and kid in site else None}
            if ok and tr is not None and row['leader_group_matches']:
                matched += 1
                if residual is not None and residual >= 0.01:
                    inexact.append(row)
                elif len(examples) < 6:
                    examples.append(row)
            else:
                unmatched.append(row)
        correlation = {'observations': a.observations, 'level_changing_spans': matched + len(unmatched), 'matched': matched,
                       'matched_exactly': matched - len(inexact), 'matched_with_other_change_in_sample': inexact,
                       'unmatched': unmatched[:30], 'unmatched_count': len(unmatched), 'examples': examples,
                       'rule': 'the first 1 s observation sample strictly after the verdict has a supply event for the same keysite, changing '
                               'the same resource in the same direction, and a track event for the task leader in the verdict\'s group; '
                               'how much the sample\'s change differs from the span\'s is reported, not asserted'}
        claims.check('correlation.public_path', 'Every keysite level change inside a PICK_UP or DROP_OFF span appears in the public path\'s '
                     'observations at the next 1 s sample, in the same direction, with the task leader tracked there in the same group', unmatched,
                     matched + len(unmatched))

    report = {
        'trace': a.trace, 'header': events[0], 'counters': counters,
        'claims': claims.result,
        'facts': {
            'supply_tasks_created': len(tasks),
            'waypoints_at_creation': dict(route_types),
            'arrivals_per_task_in_order': dict(route_flown.most_common()),
            'arrival_chains_per_task': dict(chains),
            'verdicts_on_supply_tasks_by_type_last_to_reach_leader_kind': {f'{t} | last_to_reach={l} | {k}': n for (t, l, k), n in sorted(verdict_types.items(), key=str)},
            'pickup_outcomes': pickup_outcomes,
            'dropoff_outcomes': dropoff_outcomes,
            'completion_versus_delivery': dict(completion_vs_delivery),
            'supply_completions_outside_arrival_spans_by_reason': dict(completions_outside),
            'supply_success_by_where_and_delivery': dict(success),
            'supply_success_without_delivery': success_examples,
            'lifetimes_at_finish': dict(lifetimes),
            'freed_named_entities_by_type': dict(freed_types),
            'last_to_reach_verdicts_on_supply_routes': dict(all_members_path),
            'multi_member_verdicts_at_the_four_types': {
                'count': len(multi),
                'by_leader_kind': dict(Counter(m['leader_kind'] for m in multi)),
                'where_all_members_would_have_failed': sum(m['all_members_would_fail'] for m in multi),
                'verdicts': multi,
            },
        },
        'correlation': correlation,
    }
    failed = [n for n, c in claims.result.items() if c['verdict'] == 'FAIL']
    if a.report:
        with open(a.report, 'w', encoding='utf-8', newline='\n') as f:
            json.dump(report, f, indent=1)
            f.write('\n')
    for n, c in claims.result.items():
        print(f"{c['verdict']:4} {n} ({c['checked']} checked{', ' + str(c['failure_count']) + ' failing' if c['failure_count'] else ''})")
    m = report['facts']['multi_member_verdicts_at_the_four_types']
    print(f"facts: {len(tasks)} SUPPLY tasks; {len(supply_four)} arrivals at the four types; {m['count']} with another valid member, "
          f"{m['where_all_members_would_have_failed']} where the all-members rule would have failed")
    print('FAIL: ' + ', '.join(failed) if failed else 'PASS')
    return 1 if failed else 0


if __name__ == '__main__':
    sys.exit(main())
