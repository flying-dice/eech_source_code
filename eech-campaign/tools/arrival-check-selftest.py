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
    ('the leader far outside the radius at its verdict', case_leader_far, 'leader.state_at_verdict'),
    ('the guide not advanced when the leader is told', case_guide_not_advanced, 'order.guide_advances_first'),
    ('a pick-up without the supplier\'s drop', case_pickup_no_drop, 'pickup.accounting'),
    ('a keysite drop-off not set to 100', case_dropoff_not_full, 'dropoff.accounting'),
    ('FINISH_DROP_OFF without SUCCESS', case_finish_not_success, 'finish.completes_success'),
    ('an entity named after its generation was freed', case_reuse_after_free, 'identity.generations'),
    ('a truncated trace (no counters)', case_no_counters, 'trace.complete'),
]


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
    events = [json.loads(l) for l in open(sys.argv[1], encoding='utf-8') if l.strip()]
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
                print(f"FAIL {what}: the trace has no event to inject into ({e})")
                bad += 1
                continue
            code, claims = run(mutated, tmp, claim)
            others = [n for n, c in claims.items() if c['verdict'] == 'FAIL' and n != claim]
            ok = code == 1 and claims[claim]['verdict'] == 'FAIL'
            print(f"{'ok  ' if ok else 'FAIL'} {what}: {claim} {claims[claim]['verdict']}" + (f" (also failing: {', '.join(others)})" if others else ''))
            bad += not ok
        if len(sys.argv) > 2:
            bad += correlation_case(events, sys.argv[2], tmp)
    print('PASS' if not bad else f'FAIL ({bad})')
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())
