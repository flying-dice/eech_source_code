#!/usr/bin/env python3
# Shows that tools/loss-chain-check.py requires the links it reports as
# observed. On a run's unmodified observations it must pass for the chosen
# loss, and fail when one link is removed:
#   no-loss        the aircraft's death
#   no-response    the surviving aircraft's task events after the loss
#   no-subsequent  the surviving aircraft's later TASK_SUPPLY assignments
#
# Usage: tools/loss-chain-check-selftest.py <observations.jsonl[.gz]> --unit <id>
import gzip, json, os, subprocess, sys, tempfile

here = os.path.dirname(os.path.abspath(__file__))
obs, unit = sys.argv[1], sys.argv[sys.argv.index('--unit') + 1]
opener = gzip.open if obs.endswith('.gz') else open
with opener(obs, 'rt', encoding='utf-8') as f:
    lines = f.read().splitlines()


def check(lines):
    with tempfile.TemporaryDirectory() as d:
        o, rep = os.path.join(d, 'o.jsonl'), os.path.join(d, 'r.json')
        open(o, 'w', encoding='utf-8', newline='\n').write('\n'.join(lines) + '\n')
        r = subprocess.run([sys.executable, os.path.join(here, 'loss-chain-check.py'), o, '--unit', unit, '--report', rep], capture_output=True)
        return r.returncode, (json.load(open(rep)) if os.path.exists(rep) else None)


code, base = check(lines)
if code != 0:
    sys.exit('the unmodified observations do not pass')
t = base['loss']['t']
survivors = [s['aircraft'] for s in base['response']['survivors']]
end = max(s['task_ended']['t'] for s in base['response']['survivors'])
print(f"unmodified     loss of {unit} at {t}: passes")


def without(pred):
    return [l for l in lines if not pred(json.loads(l))]


cases = {
    'no-loss': without(lambda r: r['ev'] == 'alive' and r['id'] == int(unit) and not r['alive']),
    'no-response': without(lambda r: r['ev'] == 'task' and r['id'] in survivors and r['t'] > t),
    'no-subsequent': without(lambda r: r['ev'] == 'task' and r['id'] in survivors and r['t'] > end and r.get('task') == 'TASK_SUPPLY'),
}
status = 0
for name, mutated in cases.items():
    code, _ = check(mutated)
    ok = code != 0
    status |= not ok
    print(f"{name:14} {'fails: as expected' if ok else 'PASSES: NOT AS EXPECTED'}")
sys.exit(status)
