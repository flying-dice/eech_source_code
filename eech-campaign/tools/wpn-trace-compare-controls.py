#!/usr/bin/env python3
# TEST-ONLY (EEDCS-45): runs wpn-trace-compare.py on retained real traces and on derived copies, and records each case's ACTUAL Python exit status (a subprocess
# return code, never a pipeline's), stdout and stderr, with the comparator's SHA-256 and the repository HEAD. The originals are only read; the derived inputs (empty,
# inactive, wrongly configured, wrong endpoint, changed-value, changed-hash, dropped-log-event, truncated, malformed) are written under --inputs.
#
# Usage: tools/wpn-trace-compare-controls.py --a <complete trace> --b <second complete trace of the same configuration> \
#            --inputs <dir for derived copies> --out <result.json>
# Exit 0 only when every case ended with its expected status.
import argparse
import hashlib
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TOOL = os.path.join(HERE, 'wpn-trace-compare.py')


def run(case, expected, arguments):
    command = [sys.executable, '-B', TOOL] + arguments
    done = subprocess.run(command, capture_output=True, text=True, encoding='utf-8')
    return {'case': case, 'command': ['python', 'tools/wpn-trace-compare.py'] + [os.path.basename(a) if os.path.exists(a) else a for a in arguments],
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

    def transform(name, change):
        out = []
        for event in events:
            event = json.loads(json.dumps(event))
            event = change(event)
            if event is not None:
                out.append(json.dumps(event, sort_keys=True))
        write(derived(name), out)
        return derived(name)

    def mutate(name, predicate, change):
        done = []

        def step(event):
            if not done and predicate(event):
                change(event)
                done.append(1)
            return event

        path = transform(name, step)
        assert done, 'nothing to change for ' + name
        return path

    def counter(scope, name):
        return lambda e: e.get('scope') == scope and e.get('event') == 'wpn-counter' and e.get('name') == name

    changed_counter = mutate('changed-run-counter.jsonl', counter('run', 'move_calls'), lambda e: e.update(value=e['value'] + 1))
    changed_hash = mutate('changed-run-hash.jsonl', lambda e: e.get('scope') == 'run' and e.get('event') == 'wpn-hash',
                          lambda e: e.update(flight=('0' if e['flight'][0] != '0' else '1') + e['flight'][1:]))
    changed_checkpoint = mutate('changed-checkpoint-counter.jsonl', counter('checkpoint', 'birth_calls'), lambda e: e.update(value=e['value'] + 1))
    changed_range = mutate('changed-run-range.jsonl', lambda e: e.get('scope') == 'run' and e.get('event') == 'wpn-range' and e.get('n', 0) > 0, lambda e: e.update(max=e['max'] + 1.0))
    changed_log = mutate('changed-log-event.jsonl', lambda e: e.get('scope') == 'log' and e.get('kind') == 'terminal', lambda e: e.update(distance=e['distance'] + 1.0))
    changed_table = mutate('changed-table-row.jsonl', lambda e: e.get('scope') == 'run' and e.get('event') == 'wpn-table',
                           lambda e: e.update(hash=('0' if e['hash'][0] != '0' else '1') + e['hash'][1:]))
    changed_boot_table = mutate('changed-boot-report.jsonl', lambda e: e.get('scope') == 'boot' and e.get('event') == 'wpn-counter' and e.get('name') == 'table_cells',
                                lambda e: e.update(value=e['value'] + 1))
    # wrongly configured but otherwise complete traces (both sides, so equality alone would not catch them)
    bad_seconds = mutate('wrong-seconds.jsonl', lambda e: e.get('event') == 'boot', lambda e: e.update(seconds=60))
    bad_seed = mutate('wrong-seed.jsonl', lambda e: e.get('event') == 'boot', lambda e: e.update(seed=2))
    bad_scenario = mutate('wrong-scenario.jsonl', lambda e: e.get('event') == 'boot', lambda e: e.update(scenario='georgia_retail'))
    bad_endpoint_short = mutate('endpoint-short.jsonl', lambda e: e.get('event') == 'run', lambda e: e.update(elapsed=599.9))
    bad_endpoint_long = mutate('endpoint-long.jsonl', lambda e: e.get('event') == 'run', lambda e: e.update(elapsed=650.0))
    bad_frames = mutate('wrong-frame-count.jsonl', lambda e: e.get('event') == 'run', lambda e: e.update(frames=e['frames'] + 5))
    # inactive: the activity counters zeroed in the run report, otherwise complete
    inactive = transform('inactive.jsonl', lambda e: (e.update(value=0) or e) if e.get('scope') == 'run' and e.get('event') == 'wpn-counter'
                         and e.get('name') in ('birth_calls', 'move_calls', 'terminal_calls') else e)
    no_table_activity = mutate('no-table-activity.jsonl', counter('run', 'table_supplies'), lambda e: e.update(value=0))
    no_hash = transform('no-run-hash.jsonl', lambda e: None if e.get('scope') == 'run' and e.get('event') == 'wpn-hash' else e)
    log_overflow = mutate('log-overflow.jsonl', lambda e: e.get('scope') == 'run' and e.get('event') == 'wpn-log', lambda e: e.update(overflow=1))
    bookkeeping = mutate('bookkeeping-unregistered.jsonl', counter('run', 'terminal_unregistered'), lambda e: e.update(value=1))
    # an ordered log with one event dropped (middle), then one reordered
    dropped = []
    skip = [len([x for x in events if x.get('scope') == 'log']) // 2]
    seen = [0]

    def drop(e):
        if e.get('scope') == 'log':
            seen[0] += 1
            if seen[0] == skip[0]:
                return None
        return e

    dropped_path = transform('dropped-log-event.jsonl', drop)
    write(derived('empty-a.jsonl'), [])
    write(derived('empty-b.jsonl'), [])
    write(derived('truncated-no-terminal.jsonl'), lines[:-1])
    write(derived('malformed.jsonl'), lines[:5] + ['{not json'] + lines[5:])
    snapshots = [line for line, e in zip(lines, events) if e.get('scope') == 'checkpoint' or e.get('event') in ('boot', 'checkpoint')]
    write(derived('checkpoints-only.jsonl'), snapshots)

    cases = [
        run('RED control 1: two empty files', 2, [derived('empty-a.jsonl'), derived('empty-b.jsonl')]),
        run('RED control 2: checkpoint snapshots without the final run report, scope checkpoint', 2,
            [derived('checkpoints-only.jsonl'), derived('checkpoints-only.jsonl'), '--scope', 'checkpoint']),
        run('checkpoint snapshots only, --skip-samples does not weaken completion', 2,
            [derived('checkpoints-only.jsonl'), derived('checkpoints-only.jsonl'), '--scope', 'checkpoint', '--skip-samples']),
        run('RED control 3: a complete but inactive trace against itself', 2, [inactive, inactive]),
        run('a complete trace without table activity against itself', 2, [no_table_activity, no_table_activity]),
        run('a complete trace without the run-wide hash', 2, [args.a, no_hash]),
        run('a wrongly configured trace (60 s) against itself', 2, [bad_seconds, bad_seconds]),
        run('a wrongly configured trace (seed 2) against itself', 2, [bad_seed, bad_seed]),
        run('a wrongly configured trace (other scenario) against itself', 2, [bad_scenario, bad_scenario]),
        run('a run that stopped one frame short of the endpoint (clock 599.9) against itself', 2, [bad_endpoint_short, bad_endpoint_short]),
        run('a run that ran past the first clock reaching the endpoint (650 s) against itself', 2, [bad_endpoint_long, bad_endpoint_long]),
        run('a run marker with a wrong frame count against itself', 2, [bad_frames, bad_frames]),
        run('an ordered log with an overflow flag', 2, [args.a, log_overflow]),
        run('fixture bookkeeping counter non-zero (terminal_unregistered 1)', 2, [args.a, bookkeeping]),
        run('an ordered log with one event dropped', 2, [args.a, dropped_path]),
        run('two complete traces of the same configuration, all scopes', 0, [args.a, args.b]),
        run('a complete trace against itself, all scopes', 0, [args.a, args.a]),
        run('a complete trace against itself, scope run', 0, [args.a, args.a, '--scope', 'run']),
        run('a complete trace against itself, scope log', 0, [args.a, args.a, '--scope', 'log']),
        run('a complete trace against itself, scope boot', 0, [args.a, args.a, '--scope', 'boot']),
        run('a complete trace against itself, scope checkpoint', 0, [args.a, args.a, '--scope', 'checkpoint']),
        run('a complete trace against itself, --skip-samples', 0, [args.a, args.a, '--skip-samples']),
        run('changed value in a copy (run counter move_calls + 1)', 1, [args.a, changed_counter]),
        run('changed run flight hash in a copy', 1, [args.a, changed_hash]),
        run('changed table row hash in a copy', 1, [args.a, changed_table]),
        run('changed boot-report counter in a copy (table_cells + 1)', 1, [args.a, changed_boot_table]),
        run('changed ordered-log terminal event in a copy (distance + 1)', 1, [args.a, changed_log]),
        run('changed ordered-log event in a copy, scope run only (the log change is outside the selection)', 0, [args.a, changed_log, '--scope', 'run']),
        run('changed checkpoint counter in a copy (birth_calls + 1), all scopes', 1, [args.a, changed_checkpoint]),
        run('changed checkpoint counter in a copy, scope run only (the checkpoint change is outside the selection)', 0, [args.a, changed_checkpoint, '--scope', 'run']),
        run('changed value range in a copy (max + 1)', 1, [args.a, changed_range]),
        run('truncated copy without the terminal event', 2, [args.a, derived('truncated-no-terminal.jsonl')]),
        run('malformed line in a copy', 2, [args.a, derived('malformed.jsonl')]),
        run('missing file', 2, [args.a, derived('does-not-exist.jsonl')]),
    ]
    head = subprocess.run(['git', 'rev-parse', 'HEAD'], capture_output=True, text=True, cwd=HERE).stdout.strip()
    result = {'comparator': 'tools/wpn-trace-compare.py', 'comparator_sha256': hashlib.sha256(open(TOOL, 'rb').read()).hexdigest(),
              'repository_head_when_run': head, 'python': sys.version.split()[0], 'cases': cases, 'all_ok': all(c['ok'] for c in cases)}
    with open(args.out, 'w', encoding='utf-8', newline='\n') as handle:
        json.dump(result, handle, indent=2)
        handle.write('\n')
    for c in cases:
        print(('ok   ' if c['ok'] else 'FAIL ') + f"exit {c['actual_exit']} (expected {c['expected_exit']}): {c['case']}")
    return 0 if result['all_ok'] else 1


if __name__ == '__main__':
    sys.exit(main())
