#!/usr/bin/env python3
# EEDCS-11 helicopter discriminator: which arrival rule did the campaign apply
# when a real AI helicopter wingman was held beyond 750 m from its leader?
#
# Reads an arrival trace from the test-only CONTROLLED build run with
# EECH_ARRIVAL_CONTROL=hold:... (tools/arrival-observe/eech_arrival_control.c).
# Every guide check during a trial is a trial_check event: the leader's own
# state, each other member's distance to the leader, who the check asked, and
# the verdict. For each trial:
#
#   decisive frame   the task leader meets its own reach criteria for this
#                    waypoint (AI helicopter: not landed, rotors spun up,
#                    range to its guide position <= the radius; hc_msgs.c:529-610)
#                    while a valid wingman is NOT within 750 m of it, airborne
#                    and spun up (the all-members rule's per-member test,
#                    hc_msgs.c:766-788)
#   leader-only      the verdict fired at a decisive frame: the wingman did not hold it
#   all-members      the verdict was withheld at every decisive frame, and fired
#                    at the first later frame where every valid wingman passed
#   withheld         decisive frames, all withheld, and no verdict in the trial
#   inconclusive     no decisive frame (the leader never met its criteria while
#                    a wingman was held away), or a pattern neither rule gives
#
# The rule is judged from the outcome against recorded state; the members the
# check asked are reported beside it, not used to classify.
#
# Usage: tools/helicopter-discriminate.py <arrivals-<pid>.jsonl[.gz]> [--paired <ordinary trace>] [--report <report.json>]
# Exit 0 when the trace holds at least one trial, 2 when it holds none. The
# report states each trial's classification; it asserts no expected rule.
import argparse, gzip, json, sys
from collections import Counter, defaultdict

COHESION = 750.0   # hc_msgs.c HELICOPTER_LAST_TO_REACH_DISTANCE (its square root)
EPS = 0.002


def load(path):
    opener = gzip.open if path.endswith('.gz') else open
    with opener(path, 'rt', encoding='utf-8') as f:
        return [json.loads(l) for l in f if l.strip()]


def leader_meets(c):
    if c.get('leader_landed') or (c.get('leader_rotor_spin_up') or 0) > 0 or c.get('radius', -1) < 0:
        return False
    return c.get('leader_distance') is not None and c['leader_distance'] <= c['radius'] + EPS


def wingman_fails(o):
    return o['valid'] and (o['landed'] or (o.get('rotor_spin_up') or 0) > 0 or o['to_leader_2d'] > COHESION)


def classify(checks):
    """(rule, detail) for one trial's checks, in order"""
    decisive = [i for i, c in enumerate(checks) if leader_meets(c) and any(wingman_fails(o) for o in c['others'])]
    fired = [i for i, c in enumerate(checks) if c['verdict']]
    if not decisive:
        return 'inconclusive', 'the leader never met its criteria while a wingman was held away'
    if fired and fired[0] in decisive:
        return 'leader-only', f'the verdict fired at decisive frame {fired[0]} (of {len(decisive)} decisive)'
    if any(i in decisive for i in fired):
        return 'inconclusive', 'a verdict at a decisive frame after an earlier verdict'
    if not fired:
        return 'withheld', f'withheld at all {len(decisive)} decisive frames; no verdict in the trial'
    first = fired[0]
    if first < decisive[0]:
        return 'inconclusive', 'the verdict came before any decisive frame'
    ready = [i for i, c in enumerate(checks) if i > decisive[0] and leader_meets(c) and not any(wingman_fails(o) for o in c['others'])]
    if ready and first == ready[0]:
        return 'all-members', (f'withheld at all {len(decisive)} decisive frames; fired at the first frame ({first}) where the leader met its '
                               'criteria and every valid wingman passed')
    return 'inconclusive', f'withheld at decisive frames, then fired at frame {first}, not at the first frame every wingman passed ({ready[:1]})'


def analyse(events):
    trials = defaultdict(lambda: {'checks': [], 'start': None, 'end': None, 'release': None, 'holds': 0})
    for e in events:
        if e['e'] == 'trial_check':
            trials[e['trial']]['checks'].append(e)
        elif e['e'] == 'trial_start':
            trials[e['trial']]['start'] = e
        elif e['e'] == 'trial_end':
            trials[e['trial']]['end'] = e
        elif e['e'] == 'trial_release':
            trials[e['trial']]['release'] = e
        elif e['e'] == 'hold':
            trials[e['trial']]['holds'] += 1
    rows = []
    for tid, t in sorted(trials.items()):
        checks = t['checks']
        if not checks:
            continue
        rule, detail = classify(checks)
        verdict_check = next((c for c in checks if c['verdict']), None)
        far = [max((o['to_leader_2d'] for o in c['others'] if o['valid']), default=0) for c in checks if c['verdict']]
        rows.append({'trial': tid, 'wp_type': checks[0]['wp_type'], 'last_to_reach': checks[0]['last_to_reach'], 'radius': checks[0]['radius'],
                     'guide': checks[0]['guide'], 'task_leader': checks[0]['task_leader'], 't_start': checks[0]['t'],
                     'checks': len(checks), 'released': t['release'] is not None, 'rule': rule, 'detail': detail,
                     'verdict_t': verdict_check['t'] if verdict_check else None,
                     'wingman_to_leader_at_verdict_m': round(far[0], 1) if far else None,
                     'asked_at_verdict': [q['id'] for q in verdict_check['queried']] if verdict_check else None,
                     'asked_wingman_in_trial': any(not q['is_task_leader'] for c in checks for q in c['queried'])})
    by = defaultdict(Counter)
    for r in rows:
        by[f"{r['wp_type']} | last_to_reach={r['last_to_reach']}"][r['rule']] += 1
    return rows, {k: dict(v) for k, v in by.items()}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('trace')
    ap.add_argument('--report')
    ap.add_argument('--paired', help='the ordinary (control off) trace of the same scenario: each trial\'s waypoint is matched by '
                                     '[index, generation] and the verdict times compared')
    a = ap.parse_args()
    events = load(a.trace)
    rows, summary = analyse(events)
    paired = None
    if a.paired:
        ordinary = {tuple(e['wp']): e for e in load(a.paired) if e['e'] == 'verdict'}
        deltas = defaultdict(list)
        unmatched = 0
        for r in rows:
            first = next(c for c in events if c['e'] == 'trial_check' and c['trial'] == r['trial'])
            o = ordinary.get(tuple(first['wp']))
            r['ordinary_verdict_t'] = o['t'] if o else None
            r['ordinary_wingman_to_leader_m'] = None
            if o:
                others = [m['to_leader_2d'] for m in o['members'] if m['id'] != o['task_leader']]
                r['ordinary_wingman_to_leader_m'] = round(max(others), 1) if others else None
            if o and r['verdict_t'] is not None:
                r['delay_s'] = round(r['verdict_t'] - o['t'], 3)
                deltas[r['wp_type']].append(r['delay_s'])
            else:
                unmatched += 1
        paired = {'ordinary': a.paired, 'unmatched_trials': unmatched,
                  'delay_s_by_waypoint': {k: {'n': len(v), 'zero': sum(1 for x in v if abs(x) < 1e-6), 'min': min(v), 'max': max(v)}
                                          for k, v in deltas.items()},
                  'note': 'the same waypoint ([index, generation]) in the ordinary run; the controlled scenario diverges after its first hold, '
                          'so later waypoints may not exist in the ordinary run or may be reached by a changed flight'}
    report = {'trace': a.trace, 'header': events[0] if events else None, 'trials': len(rows), 'by_waypoint_and_rule': summary,
              'paired_with_ordinary': paired, 'per_trial': rows}
    if a.report:
        with open(a.report, 'w', encoding='utf-8', newline='\n') as f:
            json.dump(report, f, indent=1)
            f.write('\n')
    print(f'{len(rows)} trials')
    for k, v in sorted(summary.items()):
        print(f'  {k}: ' + ', '.join(f'{n} {r}' for r, n in sorted(v.items())))
    if paired:
        for k, v in sorted(paired['delay_s_by_waypoint'].items()):
            print(f"  paired {k}: {v['n']} matched, verdict delay {v['min']}..{v['max']} s ({v['zero']} with no delay)")
        print(f"  paired: {paired['unmatched_trials']} trials without a matching ordinary verdict")
    return 0 if rows else 2


if __name__ == '__main__':
    sys.exit(main())
