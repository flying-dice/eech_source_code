#!/usr/bin/env python3
# TEST-ONLY (EEDCS-43): compares two lua/sensor-characterization.lua traces (JSON Lines) field by field. Used to show that a
# run with the read-only probe, or a later extraction candidate, left every hash, counter, edge and sample unchanged.
#
# Usage: tools/sensor-trace-compare.py <a.jsonl> <b.jsonl> [--scope run|probe|run-at-probe ...] [--skip-samples]
#   --scope selects which report scopes are compared (default: run and probe); the "run-at-probe" snapshots are the run-wide
#   report read just before each probe.
# Exit 0 when every selected event is equal (same order), 1 otherwise; the differing events are printed.
import argparse
import json
import sys


def load(path, scopes, skip_samples):
    events = []
    with open(path, encoding='utf-8') as handle:
        for number, line in enumerate(handle, 1):
            event = json.loads(line)
            if event.get('scope') not in scopes:
                continue
            if skip_samples and event.get('event') == 'sensor-sample':
                continue
            events.append((number, event))
    return events


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('a')
    parser.add_argument('b')
    parser.add_argument('--scope', action='append')
    parser.add_argument('--skip-samples', action='store_true')
    args = parser.parse_args()
    scopes = set(args.scope or ['run', 'probe'])
    left = load(args.a, scopes, args.skip_samples)
    right = load(args.b, scopes, args.skip_samples)
    differences = 0
    for index in range(max(len(left), len(right))):
        l = left[index] if index < len(left) else None
        r = right[index] if index < len(right) else None
        if l is None or r is None or l[1] != r[1]:
            differences += 1
            if differences <= 20:
                print(f'difference at event {index}:')
                print('  a:', json.dumps(l[1], sort_keys=True) if l else None)
                print('  b:', json.dumps(r[1], sort_keys=True) if r else None)
    print(f'{len(left)} events in a, {len(right)} in b, scopes {sorted(scopes)}: {differences} differences')
    return 1 if differences else 0


if __name__ == '__main__':
    sys.exit(main())
