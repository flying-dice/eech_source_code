#!/usr/bin/env python3
# TEST-ONLY (EEDCS-43): compares two lua/sensor-characterization.lua traces (JSON Lines) field by field. Used to show that a
# run with the read-only probe, or a later extraction candidate, left every hash, counter, edge and sample unchanged.
#
# Usage: tools/sensor-trace-compare.py <a.jsonl> <b.jsonl> [--scope run|probe|run-at-probe ...] [--skip-samples]
#   --scope selects which report scopes are compared (default: run and probe); the "run-at-probe" snapshots are the run-wide
#   report read just before each probe. Comparison is strict: same events, same order, every field equal.
#
# Fails closed. Exit 0 only when both files are complete traces, at least one event was selected, and every selected event is equal.
# Exit 1: a difference. Exit 2: invalid input, with the reason on stderr: a file that is missing, unreadable or not JSON Lines, a file
# that is not a COMPLETE trace, or a selection that matches no event. A complete trace ends with the terminal `sensor-samples` event of
# scope `run` (the driver writes the final run-wide report last), so a truncated run, or probe snapshots without the final report,
# are refused whatever scopes are selected. --skip-samples leaves out the individual `sensor-sample` rows of the selected scopes only;
# the completion check always looks at the whole file.
import argparse
import json
import sys

TERMINAL = ('sensor-samples', 'run')


class Invalid(Exception):
    pass


def load(path):
    events = []
    try:
        with open(path, encoding='utf-8') as handle:
            for number, line in enumerate(handle, 1):
                if not line.strip():
                    continue
                try:
                    event = json.loads(line)
                except ValueError as error:
                    raise Invalid(f'{path}: line {number} is not JSON ({error})')
                if not isinstance(event, dict):
                    raise Invalid(f'{path}: line {number} is not a JSON object')
                events.append((number, event))
    except OSError as error:
        raise Invalid(f'{path}: cannot read ({error.strerror or error})')
    if not events:
        raise Invalid(f'{path}: empty, not a trace')
    number, last = events[-1]
    if (last.get('event'), last.get('scope')) != TERMINAL:
        raise Invalid(f'{path}: incomplete, the last event (line {number}) is not the terminal run report '
                      f'(event {TERMINAL[0]}, scope {TERMINAL[1]}); a truncated run or probe snapshots alone do not qualify')
    return events


def select(events, scopes, skip_samples):
    return [e for _, e in events if e.get('scope') in scopes and not (skip_samples and e.get('event') == 'sensor-sample')]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('a')
    parser.add_argument('b')
    parser.add_argument('--scope', action='append', choices=['run', 'probe', 'run-at-probe'])
    parser.add_argument('--skip-samples', action='store_true')
    args = parser.parse_args()
    scopes = set(args.scope or ['run', 'probe'])
    try:
        left = select(load(args.a), scopes, args.skip_samples)
        right = select(load(args.b), scopes, args.skip_samples)
        if not left or not right:
            raise Invalid(f'scopes {sorted(scopes)} select no events ({len(left)} in a, {len(right)} in b); nothing was compared')
    except Invalid as error:
        print(f'invalid input: {error}', file=sys.stderr)
        return 2
    differences = 0
    for index in range(max(len(left), len(right))):
        l = left[index] if index < len(left) else None
        r = right[index] if index < len(right) else None
        if l != r:
            differences += 1
            if differences <= 20:
                print(f'difference at event {index}:')
                print('  a:', json.dumps(l, sort_keys=True) if l is not None else None)
                print('  b:', json.dumps(r, sort_keys=True) if r is not None else None)
    print(f'{len(left)} events in a, {len(right)} in b, scopes {sorted(scopes)}: {differences} differences')
    return 1 if differences else 0


if __name__ == '__main__':
    sys.exit(main())
