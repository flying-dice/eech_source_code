#!/usr/bin/env python3
# TEST-ONLY (EEDCS-43): runs sensor-trace-compare.py on retained real traces and on derived copies, and records each case's ACTUAL Python
# exit status (a subprocess return code, never a pipeline's), stdout and stderr, with the comparator's SHA-256 and the repository HEAD.
# The originals are only read; the derived inputs (a changed-value copy, a truncated copy, a malformed copy) are written under --inputs.
#
# Usage: tools/sensor-trace-compare-controls.py --traces <dir with lebanon-probe.jsonl and lebanon-noprobe.jsonl> \
#            --negatives <dir with empty-a/b.jsonl and incomplete-probe-a/b.jsonl> --inputs <dir for derived copies> --out <result.json>
# Exit 0 only when every case ended with its expected status.
import argparse
import hashlib
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TOOL = os.path.join(HERE, 'sensor-trace-compare.py')


def run(case, expected, arguments):
    command = [sys.executable, TOOL] + arguments
    done = subprocess.run(command, capture_output=True, text=True, encoding='utf-8')
    return {'case': case, 'command': ['python', 'tools/sensor-trace-compare.py'] + [os.path.basename(a) if os.path.exists(a) else a for a in arguments],
            'expected_exit': expected, 'actual_exit': done.returncode, 'stdout': done.stdout, 'stderr': done.stderr, 'ok': done.returncode == expected}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--traces', required=True)
    parser.add_argument('--negatives', required=True)
    parser.add_argument('--inputs', required=True)
    parser.add_argument('--out', required=True)
    args = parser.parse_args()
    probe = os.path.join(args.traces, 'lebanon-probe.jsonl')
    noprobe = os.path.join(args.traces, 'lebanon-noprobe.jsonl')
    os.makedirs(args.inputs, exist_ok=True)
    with open(probe, encoding='utf-8') as handle:
        lines = [line for line in handle.read().splitlines() if line.strip()]
    # a changed value: one run-wide counter off by one, everything else untouched
    changed = os.path.join(args.inputs, 'changed-value-run.jsonl')
    done = False
    out = []
    for line in lines:
        event = json.loads(line)
        if not done and event.get('scope') == 'run' and event.get('event') == 'sensor-fn' and event.get('name') == 'surface_to_air':
            event['candidates'] += 1
            done = True
        out.append(json.dumps(event, sort_keys=True))
    assert done, 'no surface_to_air run report to change'
    with open(changed, 'w', encoding='utf-8', newline='\n') as handle:
        handle.write('\n'.join(out) + '\n')
    truncated = os.path.join(args.inputs, 'truncated-no-terminal.jsonl')
    with open(truncated, 'w', encoding='utf-8', newline='\n') as handle:
        handle.write('\n'.join(lines[:-1]) + '\n')
    malformed = os.path.join(args.inputs, 'malformed.jsonl')
    with open(malformed, 'w', encoding='utf-8', newline='\n') as handle:
        handle.write('\n'.join(lines[:5] + ['{not json'] + lines[5:]) + '\n')
    neg = lambda name: os.path.join(args.negatives, name)
    cases = [
        run('RED control 1: empty files, scope run', 2, [neg('empty-a.jsonl'), neg('empty-b.jsonl'), '--scope', 'run']),
        run('RED control 2: probe snapshots without the final run report, scope probe', 2,
            [neg('incomplete-probe-a.jsonl'), neg('incomplete-probe-b.jsonl'), '--scope', 'probe']),
        run('incomplete probe snapshots, --skip-samples does not weaken completion', 2,
            [neg('incomplete-probe-a.jsonl'), neg('incomplete-probe-b.jsonl'), '--scope', 'probe', '--skip-samples']),
        run('complete probe vs no-probe trace, scope run', 0, [probe, noprobe, '--scope', 'run']),
        run('complete probe trace against itself, scope probe', 0, [probe, probe, '--scope', 'probe']),
        run('complete probe trace against itself, default scopes', 0, [probe, probe]),
        run('complete probe trace against itself, scope probe --skip-samples', 0, [probe, probe, '--scope', 'probe', '--skip-samples']),
        run('changed value in a copy (one surface_to_air candidates count + 1), scope run', 1, [probe, changed, '--scope', 'run']),
        run('truncated copy without the terminal event, scope run', 2, [probe, truncated, '--scope', 'run']),
        run('malformed line in a copy', 2, [probe, malformed, '--scope', 'run']),
        run('missing file', 2, [probe, os.path.join(args.inputs, 'does-not-exist.jsonl'), '--scope', 'run']),
        run('scope with no selected event (no-probe trace has no probe report)', 2, [noprobe, noprobe, '--scope', 'probe']),
    ]
    head = subprocess.run(['git', 'rev-parse', 'HEAD'], capture_output=True, text=True, cwd=HERE).stdout.strip()
    result = {'comparator': 'tools/sensor-trace-compare.py', 'comparator_sha256': hashlib.sha256(open(TOOL, 'rb').read()).hexdigest(),
              'repository_head_when_run': head, 'python': sys.version.split()[0], 'cases': cases, 'all_ok': all(c['ok'] for c in cases)}
    with open(args.out, 'w', encoding='utf-8', newline='\n') as handle:
        json.dump(result, handle, indent=2)
        handle.write('\n')
    for c in cases:
        print(('ok   ' if c['ok'] else 'FAIL ') + f"exit {c['actual_exit']} (expected {c['expected_exit']}): {c['case']}")
    return 0 if result['all_ok'] else 1


if __name__ == '__main__':
    sys.exit(main())
