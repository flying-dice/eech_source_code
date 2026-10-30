#!/usr/bin/env python3
# TEST-ONLY (EEDCS-44): compares two lua/aim-characterization.lua traces (JSON Lines) field by field. Used to show that a candidate (or a
# rerun of the same build) left every hash, counter, value range, kind, weapon and sample unchanged.
#
# Usage: tools/aim-trace-compare.py <a.jsonl> <b.jsonl> [--scope run|checkpoint ...] [--skip-samples]
#   --scope selects which report scopes are compared (default: run and checkpoint). The scope-less marker events (boot, each checkpoint
#   marker, the run marker: configuration, frames, simulated time) are always compared as well. Comparison is strict: same events, same
#   order, every field equal; a differing counter is a difference, never ignored.
#
# Fails closed. Exit 0 only when both files are complete traces of an active run, at least one report event was selected, and every
# selected event is equal. Exit 1: a difference. Exit 2: invalid input, with the reason on stderr: a file that is missing, unreadable or not
# JSON Lines; not a COMPLETE trace (it must end with the terminal `aim-samples` event of scope `run`, which the driver writes last, so a
# truncated run is refused whatever scopes are selected and also with --skip-samples); a trace without the run-wide hash, counters or
# ranges; a trace with NO aiming activity (vec_calls and ws_calls both 0: two empty traces would be equal and prove nothing); or a selection
# that matches no report event.
import argparse
import json
import re
import sys

TERMINAL = ('aim-samples', 'run')
HASH = re.compile(r'^[0-9a-f]{16}$')


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
                      f'(event {TERMINAL[0]}, scope {TERMINAL[1]}); a truncated run does not qualify')
    run = [e for _, e in events if e.get('scope') == 'run']
    hashes = [e for e in run if e.get('event') == 'aim-hash']
    if len(hashes) != 1 or not HASH.match(str(hashes[0].get('physical', ''))) or not HASH.match(str(hashes[0].get('decision', ''))):
        raise Invalid(f'{path}: the run report has no well-formed aim-hash event')
    counters = {e.get('name'): e.get('value') for e in run if e.get('event') == 'aim-counter'}
    if not counters or not any(e.get('event') == 'aim-range' for e in run):
        raise Invalid(f'{path}: the run report has no aim-counter / aim-range events')
    for name in ('vec_calls', 'ws_calls'):
        if not isinstance(counters.get(name), int):
            raise Invalid(f'{path}: the run report lacks the counter {name}')
    if counters['vec_calls'] == 0 and counters['ws_calls'] == 0:
        raise Invalid(f'{path}: no aiming activity recorded (vec_calls and ws_calls are 0); an inactive trace is not a characterisation')
    return events


def select(events, scopes, skip_samples):
    chosen = []
    for _, e in events:
        scope = e.get('scope')
        if scope is None or scope in scopes:
            if skip_samples and e.get('event') == 'aim-sample':
                continue
            chosen.append(e)
    return chosen


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('a')
    parser.add_argument('b')
    parser.add_argument('--scope', action='append', choices=['run', 'checkpoint'])
    parser.add_argument('--skip-samples', action='store_true')
    args = parser.parse_args()
    scopes = set(args.scope or ['run', 'checkpoint'])
    try:
        left = select(load(args.a), scopes, args.skip_samples)
        right = select(load(args.b), scopes, args.skip_samples)
        reports = lambda events: [e for e in events if e.get('scope') in scopes]
        if not reports(left) or not reports(right):
            raise Invalid(f'scopes {sorted(scopes)} select no report events ({len(reports(left))} in a, {len(reports(right))} in b); nothing was compared')
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
