#!/usr/bin/env python3
# Shows that tools/supply-chain-check.py finds what it claims to. It must find
# a complete chain in a run's unmodified observations, and must no longer find
# that chain complete when one link is removed:
#   no-assignment  every observation of the group on TASK_SUPPLY before the pick-up
#   no-pickup      the supplier's one-crate drop at the pick-up
#   broken-flight  the aircraft's track in the middle of the flight
#   no-aircraft    the aircraft's track at the drop-off (the delivery is then unattributed)
#
# Usage: tools/supply-chain-check-selftest.py <observations.jsonl> <metrics.json>
# (a run with observe_supply=1 long enough for a delivery, e.g. campaign.lua hours=1)
import json, os, subprocess, sys, tempfile

here = os.path.dirname(os.path.abspath(__file__))
obs, metrics = sys.argv[1:3]
lines = open(obs, encoding='utf-8').read().splitlines()


def check(lines):
    with tempfile.TemporaryDirectory() as d:
        o, rep = os.path.join(d, 'o.jsonl'), os.path.join(d, 'r.json')
        open(o, 'w', encoding='utf-8', newline='\n').write('\n'.join(lines) + '\n')
        subprocess.run([sys.executable, os.path.join(here, 'supply-chain-check.py'), o, metrics, '--report', rep], capture_output=True)
        return json.load(open(rep))


base = check(lines)
chain = base['featured_chain']
if not chain:
    sys.exit('no complete chain in the unmodified observations')
g, t_del, t_pick, t_asg, aircraft = chain['group'], chain['delivery']['t'], chain['pickup']['t'], chain['assigned']['t'], chain['delivery']['aircraft']
supplier = chain['pickup']['supplier']['id']
print(f"unmodified     group {g}: assigned {t_asg}, pick-up {t_pick}, delivery {t_del}: complete")


def still_complete(report):
    return any(c['group'] == g and c['delivery']['t'] == t_del and c['complete'] for c in report['chains'])


def without(pred):
    return [l for l in lines if not pred(json.loads(l))]


mid = (t_pick + t_del) / 2
cases = {
    # the group never observed on TASK_SUPPLY before the pick-up
    'no-assignment': without(lambda r: r['ev'] == 'task' and r.get('group') == g and r.get('task') == 'TASK_SUPPLY' and r['t'] < t_pick),
    'no-pickup': without(lambda r: r['ev'] == 'supply' and r['id'] == supplier and r['t'] == t_pick),
    'broken-flight': without(lambda r: r['ev'] == 'track' and r['id'] == aircraft and mid - 30 <= r['t'] <= mid + 30),
    'no-aircraft': without(lambda r: r['ev'] == 'track' and r.get('group') == g and r['t'] == t_del),
}
status = 0
for name, mutated in cases.items():
    ok = not still_complete(check(mutated))
    status |= not ok
    print(f"{name:14} {'no longer complete: as expected' if ok else 'STILL COMPLETE: NOT AS EXPECTED'}")
sys.exit(status)
