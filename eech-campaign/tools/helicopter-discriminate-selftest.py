#!/usr/bin/env python3
# EEDCS-11 helicopter discriminator: tools/helicopter-discriminate.py must tell
# the two rules apart. From a real CONTROLLED trace, each trial is re-written as
# the OTHER rule would have played it, using the trial's own recorded state,
# and the classifier must change its verdict:
#
#   a leader-only trial    -> the verdict withheld at every decisive frame (as
#                             all-members would, while the wingman is held away):
#                             must become 'withheld' or 'all-members'
#   an all-members trial   -> the verdict at the first decisive frame (as
#                             leader-only would): must become 'leader-only'
#   a withheld trial       -> likewise: must become 'leader-only'
#
# and the unchanged trace must classify as it did. Inconclusive trials are
# counted, not rewritten.
#
# Usage: tools/helicopter-discriminate-selftest.py <controlled arrivals-<pid>.jsonl[.gz]>
# Exit 0 when every rewritten trial is detected.
import copy, importlib.util, os, sys
from collections import Counter

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location('discriminate', os.path.join(HERE, 'helicopter-discriminate.py'))
d = importlib.util.module_from_spec(spec)
spec.loader.exec_module(d)


def trial_checks(events):
    by = {}
    for e in events:
        if e['e'] == 'trial_check':
            by.setdefault(e['trial'], []).append(e)
    return by


def main():
    events = d.load(sys.argv[1])
    trials = trial_checks(events)
    before = {tid: d.classify(c)[0] for tid, c in trials.items()}
    bad, done = 0, Counter()
    for tid, checks in sorted(trials.items()):
        rule = before[tid]
        if rule == 'inconclusive':
            done['inconclusive (not rewritten)'] += 1
            continue
        other = copy.deepcopy(checks)
        decisive = [i for i, c in enumerate(other) if d.leader_meets(c) and any(d.wingman_fails(o) for o in c['others'])]
        if rule == 'leader-only':
            for i in decisive:
                other[i]['verdict'] = 0
            want = {'withheld', 'all-members'}
        else:
            for c in other:
                c['verdict'] = 0
            other[decisive[0]]['verdict'] = 1
            want = {'leader-only'}
        got = d.classify(other)[0]
        ok = got in want
        bad += not ok
        done[f"{rule} rewritten as the other rule -> {got}{'' if ok else ' (NOT DETECTED)'}"] += 1
    rows, _ = d.analyse(events)
    unchanged = all(r['rule'] == before[r['trial']] for r in rows)
    bad += not unchanged
    print(f"{'ok  ' if unchanged else 'FAIL'} the unchanged trace classifies as before ({len(rows)} trials)")
    for k, n in sorted(done.items()):
        print(f"{'FAIL' if 'NOT DETECTED' in k else 'ok  '} {n} x {k}")
    print('PASS' if not bad and rows else 'FAIL')
    return 0 if not bad and rows else 1


if __name__ == '__main__':
    sys.exit(main())
