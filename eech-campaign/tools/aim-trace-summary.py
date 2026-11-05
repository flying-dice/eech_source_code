#!/usr/bin/env python3
# TEST-ONLY (EEDCS-44): prints the run-wide (or one checkpoint's) report of a lua/aim-characterization.lua trace as compact text:
# the two hashes, every non-zero counter, the measured value ranges, the entity kinds and weapons that aimed.
#
# Usage: tools/aim-trace-summary.py <trace.jsonl> [--scope run|checkpoint] [--at <seconds>] [--all-counters]
import argparse
import json


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('trace')
    parser.add_argument('--scope', default='run')
    parser.add_argument('--at', type=float)
    parser.add_argument('--all-counters', action='store_true')
    args = parser.parse_args()
    events = []
    with open(args.trace, encoding='utf-8') as handle:
        for line in handle:
            e = json.loads(line)
            if e.get('scope') == args.scope and (args.at is None or e.get('at') == args.at):
                events.append(e)
    by = {}
    for e in events:
        by.setdefault(e['event'], []).append(e)
    h = by.get('aim-hash', [{}])[0]
    print(f"scope {args.scope}  at {h.get('at')}  physical_hash {h.get('physical')}  decision_hash {h.get('decision')}")
    for e in by.get('aim-counter', []):
        if args.all_counters or e['value']:
            print(f"  {e['name']:42} {e['value']}")
    for e in by.get('aim-range', []):
        print(f"  range {e['name']:30} n={e['n']} min={e['min']} max={e['max']}")
    for e in by.get('aim-kind', []):
        print(f"  kind {e['source']:24} {e['function']:22} calls {e['calls']} reached_devices {e.get('reached_devices')} valid {e.get('valid')} steps {e.get('steps')}")
    for e in by.get('aim-weapon', []):
        print(f"  weapon {e['sub_type']:4} {e['name']:34} {e['aiming']:20} solves {e['solves']} true {e['solves_true']} leads {e['leads']} moving {e['leads_moving']} ballistic {e['ballistic']}")
    for name in ('aim-kinds', 'aim-weapons', 'aim-samples'):
        for e in by.get(name, []):
            print(' ', name, {k: v for k, v in e.items() if k not in ('event', 'scope', 'at')})


if __name__ == '__main__':
    main()
