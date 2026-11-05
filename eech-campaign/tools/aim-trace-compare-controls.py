#!/usr/bin/env python3
# TEST-ONLY (EEDCS-44): runs aim-trace-compare.py on retained real traces and on derived copies, and records each case's ACTUAL Python
# exit status (a subprocess return code, never a pipeline's), stdout and stderr, with the comparator's SHA-256 and the repository HEAD.
# The originals are only read; the derived inputs (empty, inactive, changed-value, changed-hash, truncated, malformed) are written under --inputs.
#
# Usage: tools/aim-trace-compare-controls.py --a <complete trace> --b <second complete trace of the same configuration> \
#            --inputs <dir for derived copies> --out <result.json>
# Exit 0 only when every case ended with its expected status.
import argparse
import hashlib
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TOOL = os.path.join(HERE, 'aim-trace-compare.py')


def run(case, expected, arguments):
    command = [sys.executable, TOOL] + arguments
    done = subprocess.run(command, capture_output=True, text=True, encoding='utf-8')
    return {'case': case, 'command': ['python', 'tools/aim-trace-compare.py'] + [os.path.basename(a) if os.path.exists(a) else a for a in arguments],
            'expected_exit': expected, 'actual_exit': done.returncode, 'stdout': done.stdout, 'stderr': done.stderr, 'ok': done.returncode == expected}


def write(path, lines):
    with open(path, 'w', encoding='utf-8', newline='\n') as handle:
        handle.write('\n'.join(lines) + ('\n' if lines else ''))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--a', required=True)
    parser.add_argument('--b', required=True)
    parser.add_argument('--inputs', required=True)
    parser.add_argument('--out', required=True)
    args = parser.parse_args()
    os.makedirs(args.inputs, exist_ok=True)
    with open(args.a, encoding='utf-8') as handle:
        lines = [line for line in handle.read().splitlines() if line.strip()]
    events = [json.loads(line) for line in lines]
    derived = lambda name: os.path.join(args.inputs, name)

    def mutate(name, predicate, change):
        done = False
        out = []
        for event in events:
            event = json.loads(json.dumps(event))
            if not done and predicate(event):
                change(event)
                done = True
            out.append(json.dumps(event, sort_keys=True))
        assert done, 'nothing to change for ' + name
        write(derived(name), out)
        return derived(name)

    # a changed counter in the terminal run report (one count off by one) and a changed physical hash
    changed_counter = mutate('changed-run-counter.jsonl', lambda e: e.get('scope') == 'run' and e.get('event') == 'aim-counter' and e.get('name') == 'lead_calls',
                             lambda e: e.update(value=e['value'] + 1))
    changed_hash = mutate('changed-run-hash.jsonl', lambda e: e.get('scope') == 'run' and e.get('event') == 'aim-hash',
                          lambda e: e.update(physical=('0' if e['physical'][0] != '0' else '1') + e['physical'][1:]))
    changed_checkpoint = mutate('changed-checkpoint-counter.jsonl', lambda e: e.get('scope') == 'checkpoint' and e.get('event') == 'aim-counter' and e.get('name') == 'vec_calls',
                                lambda e: e.update(value=e['value'] + 1))
    changed_range = mutate('changed-run-range.jsonl', lambda e: e.get('scope') == 'run' and e.get('event') == 'aim-range' and e.get('n', 0) > 0,
                           lambda e: e.update(max=e['max'] + 1.0))
    # an inactive trace: every vec/ws counter zeroed in the run report, otherwise complete
    inactive = []
    for event in events:
        event = json.loads(json.dumps(event))
        if event.get('scope') == 'run' and event.get('event') == 'aim-counter' and event.get('name') in ('vec_calls', 'ws_calls'):
            event['value'] = 0
        inactive.append(json.dumps(event, sort_keys=True))
    write(derived('inactive.jsonl'), inactive)
    # a complete trace without its run-wide hash
    no_hash = [json.dumps(e, sort_keys=True) for e in events if not (e.get('scope') == 'run' and e.get('event') == 'aim-hash')]
    write(derived('no-run-hash.jsonl'), no_hash)
    write(derived('empty-a.jsonl'), [])
    write(derived('empty-b.jsonl'), [])
    write(derived('truncated-no-terminal.jsonl'), lines[:-1])
    write(derived('malformed.jsonl'), lines[:5] + ['{not json'] + lines[5:])
    # checkpoint snapshots only, no final run report
    snapshots = [line for line, e in zip(lines, events) if e.get('scope') == 'checkpoint' or e.get('event') in ('boot', 'checkpoint')]
    write(derived('checkpoints-only.jsonl'), snapshots)

    cases = [
        run('RED control 1: two empty files', 2, [derived('empty-a.jsonl'), derived('empty-b.jsonl')]),
        run('RED control 2: checkpoint snapshots without the final run report, scope checkpoint', 2,
            [derived('checkpoints-only.jsonl'), derived('checkpoints-only.jsonl'), '--scope', 'checkpoint']),
        run('checkpoint snapshots only, --skip-samples does not weaken completion', 2,
            [derived('checkpoints-only.jsonl'), derived('checkpoints-only.jsonl'), '--scope', 'checkpoint', '--skip-samples']),
        run('RED control 3: a complete but inactive trace against itself', 2, [derived('inactive.jsonl'), derived('inactive.jsonl')]),
        run('a complete trace without the run-wide hash', 2, [args.a, derived('no-run-hash.jsonl')]),
        run('two complete traces of the same configuration, all scopes', 0, [args.a, args.b]),
        run('a complete trace against itself, all scopes', 0, [args.a, args.a]),
        run('a complete trace against itself, scope run', 0, [args.a, args.a, '--scope', 'run']),
        run('a complete trace against itself, scope checkpoint', 0, [args.a, args.a, '--scope', 'checkpoint']),
        run('a complete trace against itself, --skip-samples', 0, [args.a, args.a, '--skip-samples']),
        run('changed value in a copy (run counter lead_calls + 1)', 1, [args.a, changed_counter]),
        run('changed run physical hash in a copy', 1, [args.a, changed_hash]),
        run('changed checkpoint counter in a copy (vec_calls + 1), all scopes', 1, [args.a, changed_checkpoint]),
        run('changed checkpoint counter in a copy, scope run only (the checkpoint change is outside the selection)', 0, [args.a, changed_checkpoint, '--scope', 'run']),
        run('changed value range in a copy (max + 1)', 1, [args.a, changed_range]),
        run('truncated copy without the terminal event', 2, [args.a, derived('truncated-no-terminal.jsonl')]),
        run('malformed line in a copy', 2, [args.a, derived('malformed.jsonl')]),
        run('missing file', 2, [args.a, derived('does-not-exist.jsonl')]),
    ]
    head = subprocess.run(['git', 'rev-parse', 'HEAD'], capture_output=True, text=True, cwd=HERE).stdout.strip()
    result = {'comparator': 'tools/aim-trace-compare.py', 'comparator_sha256': hashlib.sha256(open(TOOL, 'rb').read()).hexdigest(),
              'repository_head_when_run': head, 'python': sys.version.split()[0], 'cases': cases, 'all_ok': all(c['ok'] for c in cases)}
    with open(args.out, 'w', encoding='utf-8', newline='\n') as handle:
        json.dump(result, handle, indent=2)
        handle.write('\n')
    for c in cases:
        print(('ok   ' if c['ok'] else 'FAIL ') + f"exit {c['actual_exit']} (expected {c['expected_exit']}): {c['case']}")
    return 0 if result['all_ok'] else 1


if __name__ == '__main__':
    sys.exit(main())
