#!/usr/bin/env python3
# EEDCS-11: tools/arrival-check.py must reject what it claims to check. From a
# real arrival trace, each case injects one discrepancy into a copy and
# requires the named claim, and only a failing verdict, from the checker; the
# unchanged trace must pass every claim.
#
# With the run's observations, also: the trace and observations pass together,
# and removing the observations' supply events fails correlation.public_path.
#
# Usage: tools/arrival-check-selftest.py <arrivals-<pid>.jsonl> [<observations.jsonl[.gz]>]
# Exit 0 when every case is detected.
import copy, json, os, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
CHECK = os.path.join(HERE, 'arrival-check.py')


def first(events, pred):
    return next(e for e in events if pred(e))


def span_of(events, wp_type, need=None):
    """the first SUPPLY span at wp_type whose verdict, handler and effect satisfy need"""
    by = {}
    for e in events:
        if e.get('span'):
            by.setdefault(e['span'], []).append(e)
    for sid, evs in sorted(by.items()):
        v = next((e for e in evs if e['e'] == 'verdict'), None)
        if v and v['supply_task'] and v['wp_type'] == wp_type and (need is None or need(v, evs)):
            return v, evs
    raise LookupError(f'no {wp_type} span for this case')


def leader(v):
    return next(m for m in v['members'] if m['id'] == v['task_leader'])


def case_handler_outside(ev):
    first(ev, lambda e: e['e'] == 'handler')['span'] = 0


def case_last_to_reach(ev):
    first(ev, lambda e: e['e'] == 'verdict' and e['supply_task'] and e['four'])['last_to_reach'] = 1


def case_wingman_asked(ev):
    v = first(ev, lambda e: e['e'] == 'verdict' and e['supply_task'] and e['four'] and len(e['members']) > 1)
    other = next(m for m in v['members'] if m['id'] != v['task_leader'])
    v['queried'].append({'id': other['id'], 'is_task_leader': 0, 'reached': 1})


def case_leader_far(ev):
    v, _ = span_of(ev, 'WAYPOINT_DROP_OFF')
    leader(v)['distance'] = 5000.0
    if 'velocity' in leader(v):
        leader(v)['velocity'] = 0.0


def case_guide_not_advanced(ev):
    v, evs = span_of(ev, 'WAYPOINT_PREPARE_FOR_DROP_OFF', lambda v, evs: any('guide_wp_now' in e for e in evs if e['e'] == 'handler'))
    first(evs, lambda e: e['e'] == 'handler')['guide_wp_now'] = v['wp']


def case_pickup_no_drop(ev):
    v, evs = span_of(ev, 'WAYPOINT_PICK_UP', lambda v, evs: v.get('dependent') and v['dependent'].get('crates'))
    eff = first(evs, lambda e: e['e'] == 'effect')
    eff['dependent']['ammo'] = v['dependent']['ammo']
    eff['dependent']['fuel'] = v['dependent']['fuel']


def case_dropoff_not_full(ev):
    v, evs = span_of(ev, 'WAYPOINT_DROP_OFF', lambda v, evs: v.get('dependent') and v['dependent']['type'] == 'ENTITY_TYPE_KEYSITE'
                     and any(c.get('identify_cargo') for c in v.get('leader_aboard') or []))
    eff = first(evs, lambda e: e['e'] == 'effect')
    cargo = next(c for c in v['leader_aboard'] if c.get('identify_cargo'))['cargo']
    eff['dependent']['ammo' if cargo == 'CARGO_AMMO' else 'fuel'] = 50.0


def group_dropoffs(ev, need=None):
    """every SUPPLY DROP_OFF span to a group with identified cargo aboard the leader: (verdict, effect)"""
    eff = {e['span']: e for e in ev if e['e'] == 'effect'}
    out = [(v, eff[v['span']]) for v in ev if v['e'] == 'verdict' and v['supply_task'] and v['wp_type'] == 'WAYPOINT_DROP_OFF'
           and v.get('dependent') and v['dependent']['type'] == 'ENTITY_TYPE_GROUP' and v['span'] in eff and eff[v['span']].get('dependent')
           and any(c.get('identify_cargo') for c in v.get('leader_aboard') or []) and (need is None or need(v))]
    if not out:
        raise LookupError('no group drop-off with cargo aboard for this case')
    return out


def aboard_types(v):
    return {c.get('cargo') for c in v['leader_aboard'] if c.get('identify_cargo')}


def case_group_no_replenishment(ev):
    # the lead's reproduction (#77 review): every group drop-off leaves the group's levels as they were
    for v, e in group_dropoffs(ev):
        e['dependent']['ammo'], e['dependent']['fuel'] = v['dependent']['ammo'], v['dependent']['fuel']


def case_group_partial(ev):
    v, e = group_dropoffs(ev, lambda v: v['dependent']['fuel'] <= 60 and 'CARGO_FUEL' in aboard_types(v))[0]
    e['dependent']['fuel'] = v['dependent']['fuel'] + (100.0 - v['dependent']['fuel']) / 2


def case_group_mixed_one_dropped(ev):
    # a drop-off carrying both resources: the ammo increment removed, the fuel one kept
    v, e = group_dropoffs(ev, lambda v: aboard_types(v) >= {'CARGO_AMMO', 'CARGO_FUEL'} and v['dependent']['ammo'] < 100)[0]
    e['dependent']['ammo'] = v['dependent']['ammo']


def case_group_unaffected_changed(ev):
    # a fuel-only drop-off that also moves the group's ammo
    v, e = group_dropoffs(ev, lambda v: aboard_types(v) == {'CARGO_FUEL'})[0]
    e['dependent']['ammo'] = v['dependent']['ammo'] - 10.0


def case_group_wrong_destination(ev):
    # the replenishment lands on a group other than the task's requester
    groups = [t['requester']['id'] for t in ev if t['e'] == 'task' and t['requester']['type'] == 'ENTITY_TYPE_GROUP']
    v, e = group_dropoffs(ev)[0]
    other = next(g for g in groups if g != v['dependent']['id'])
    v['dependent']['id'] = list(other)
    e['dependent']['id'] = list(other)


def case_keysite_unaffected_changed(ev):
    v, evs = span_of(ev, 'WAYPOINT_DROP_OFF', lambda v, evs: v.get('dependent') and v['dependent']['type'] == 'ENTITY_TYPE_KEYSITE'
                     and any(c.get('identify_cargo') for c in v.get('leader_aboard') or []))
    eff = first(evs, lambda e: e['e'] == 'effect')
    cargo = next(c for c in v['leader_aboard'] if c.get('identify_cargo'))['cargo']
    other = 'fuel' if cargo == 'CARGO_AMMO' else 'ammo'
    eff['dependent'][other] = v['dependent'][other] + 5.0


def case_pickup_unaffected_changed(ev):
    v, evs = span_of(ev, 'WAYPOINT_PICK_UP', lambda v, evs: v.get('dependent') and v['dependent'].get('crates'))
    eff = first(evs, lambda e: e['e'] == 'effect')
    taken = next(c for c in eff['leader_aboard'] if c['id'] not in [x['id'] for x in v.get('leader_aboard') or []])['cargo']
    other = 'fuel' if taken == 'CARGO_AMMO' else 'ammo'
    eff['dependent'][other] = v['dependent'][other] - 10.0


def case_finish_not_success(ev):
    v, evs = span_of(ev, 'WAYPOINT_FINISH_DROP_OFF', lambda v, evs: any(e['e'] == 'completed' for e in evs))
    ev.remove(first(evs, lambda e: e['e'] == 'completed'))


def case_reuse_after_free(ev):
    f = first(ev, lambda e: e['e'] == 'freed' and e['type'] == 'ENTITY_TYPE_TASK')
    i = ev.index(f)
    late = copy.deepcopy(first(ev, lambda e: e['e'] == 'completed'))
    late['task'] = f['id']
    late['span'] = 0
    ev.insert(i + 1, late)


def case_no_counters(ev):
    ev.remove(first(ev, lambda e: e['e'] == 'counters'))


CASES = [
    ('a response outside any span', case_handler_outside, 'path.single_route'),
    ('last-to-reach set at a SUPPLY waypoint', case_last_to_reach, 'criteria.leader_only'),
    ('a wingman also asked at a SUPPLY waypoint', case_wingman_asked, 'members.leader_only_consumed'),
    ('the leader far outside the radius at its verdict', case_leader_far, 'leader.state_at_verdict'),
    ('the guide not advanced when the leader is told', case_guide_not_advanced, 'order.guide_advances_first'),
    ('a pick-up without the supplier\'s drop', case_pickup_no_drop, 'pickup.accounting'),
    ('a keysite drop-off not set to 100', case_dropoff_not_full, 'dropoff.accounting'),
    ('no group drop-off replenishes its group (the #77 review\'s reproduction)', case_group_no_replenishment, 'dropoff.accounting'),
    ('a group drop-off replenishes only half way', case_group_partial, 'dropoff.accounting'),
    ('a mixed ammo and fuel group drop-off loses its ammo increment', case_group_mixed_one_dropped, 'dropoff.accounting'),
    ('a fuel-only group drop-off also moves the group\'s ammo', case_group_unaffected_changed, 'dropoff.accounting'),
    ('a group drop-off replenishes a group other than the task\'s requester', case_group_wrong_destination, 'dropoff.accounting'),
    ('a keysite drop-off also moves the other resource', case_keysite_unaffected_changed, 'dropoff.accounting'),
    ('a pick-up also moves the supplier\'s other resource', case_pickup_unaffected_changed, 'pickup.accounting'),
    ('FINISH_DROP_OFF without SUCCESS', case_finish_not_success, 'finish.completes_success'),
    ('an entity named after its generation was freed', case_reuse_after_free, 'identity.generations'),
    ('a truncated trace (no counters)', case_no_counters, 'trace.complete'),
]


# cases that need an event only some runs contain (Georgia has no mixed ammo and fuel group drop-off)
OPTIONAL = {case_group_mixed_one_dropped}


def run(events, tmp, name, observations=None):
    trace = os.path.join(tmp, name + '.jsonl')
    report = os.path.join(tmp, name + '.json')
    with open(trace, 'w', encoding='utf-8', newline='\n') as f:
        f.writelines(json.dumps(e) + '\n' for e in events)
    extra = ['--observations', observations] if observations else []
    code = subprocess.run([sys.executable, CHECK, trace, '--report', report] + extra, stdout=subprocess.DEVNULL).returncode
    return code, json.load(open(report))['claims']


def correlation_case(events, observations, tmp):
    """the public path's supply events removed: the correlation claim must fail"""
    code, claims = run(events, tmp, 'with-observations', observations)
    ok_unchanged = code == 0 and all(c['verdict'] == 'PASS' for c in claims.values())
    print(f"{'ok  ' if ok_unchanged else 'FAIL'} the unchanged trace with its observations passes every claim")
    stripped = os.path.join(tmp, 'no-supply-events.jsonl')
    opener = __import__('gzip').open if observations.endswith('.gz') else open
    with opener(observations, 'rt', encoding='utf-8') as f, open(stripped, 'w', encoding='utf-8', newline='\n') as g:
        g.writelines(l for l in f if '"ev":"supply"' not in l)
    code, claims = run(events, tmp, 'no-supply', stripped)
    ok = code == 1 and claims['correlation.public_path']['verdict'] == 'FAIL'
    print(f"{'ok  ' if ok else 'FAIL'} the observations without their supply events: correlation.public_path {claims['correlation.public_path']['verdict']}")
    return (not ok_unchanged) + (not ok)


def main():
    opener = __import__('gzip').open if sys.argv[1].endswith('.gz') else open
    with opener(sys.argv[1], 'rt', encoding='utf-8') as f:
        events = [json.loads(l) for l in f if l.strip()]
    bad = 0
    with tempfile.TemporaryDirectory() as tmp:
        code, claims = run(events, tmp, 'unchanged')
        failing = [n for n, c in claims.items() if c['verdict'] != 'PASS']
        print(f"{'ok  ' if code == 0 and not failing else 'FAIL'} the unchanged trace passes every claim{': ' + ', '.join(failing) if failing else ''}")
        bad += bool(code or failing)
        for what, inject, claim in CASES:
            mutated = copy.deepcopy(events)
            try:
                inject(mutated)
            except (LookupError, StopIteration) as e:
                if inject in OPTIONAL:
                    print(f"n/a  {what}: this trace has no such event ({e})")
                    continue
                print(f"FAIL {what}: the trace has no event to inject into ({e})")
                bad += 1
                continue
            code, claims = run(mutated, tmp, claim)
            others = [n for n, c in claims.items() if c['verdict'] == 'FAIL' and n != claim]
            ok = code == 1 and claims[claim]['verdict'] == 'FAIL'
            print(f"{'ok  ' if ok else 'FAIL'} {what}: {claim} {claims[claim]['verdict']}" + (f" (also failing: {', '.join(others)})" if others else ''))
            bad += not ok
        # every real group replenishment, removed on its own: each must fail dropoff.accounting
        spans = [v['span'] for v, _ in group_dropoffs(copy.deepcopy(events))]
        missed = []
        for sid in spans:
            mutated = copy.deepcopy(events)
            v, e = next((v, e) for v, e in group_dropoffs(mutated) if v['span'] == sid)
            e['dependent']['ammo'], e['dependent']['fuel'] = v['dependent']['ammo'], v['dependent']['fuel']
            code, claims = run(mutated, tmp, f'group-{sid}')
            if not (code == 1 and claims['dropoff.accounting']['verdict'] == 'FAIL'):
                missed.append(sid)
        print(f"{'ok  ' if not missed else 'FAIL'} each of the {len(spans)} group drop-offs with its replenishment removed alone: "
              f"dropoff.accounting FAIL for {len(spans) - len(missed)}" + (f"; missed spans {missed}" if missed else ''))
        bad += bool(missed)
        if len(sys.argv) > 2:
            bad += correlation_case(events, sys.argv[2], tmp)
    print('PASS' if not bad else f'FAIL ({bad})')
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())
