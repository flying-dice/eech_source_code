#!/usr/bin/env python3
# TEST-ONLY (EEDCS-45): prints the run-wide (or one checkpoint's) report of a lua/weapon-characterization.lua trace as compact text: the five hashes, every non-zero
# counter, the measured value ranges, the branch/guidance/kill tallies, the weapons and tables, and the ordered log's event kinds.
#
# Usage: tools/wpn-trace-summary.py <trace.jsonl> [--scope run|checkpoint|boot] [--at <seconds>] [--all-counters]
import argparse
import collections
import json


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('trace')
    parser.add_argument('--scope', default='run')
    parser.add_argument('--at', type=float)
    parser.add_argument('--all-counters', action='store_true')
    args = parser.parse_args()
    events = []
    log = []
    with open(args.trace, encoding='utf-8') as handle:
        for line in handle:
            e = json.loads(line)
            if e.get('scope') == args.scope and (args.at is None or e.get('at') == args.at):
                events.append(e)
            if e.get('scope') == 'log':
                log.append(e)
    by = {}
    for e in events:
        by.setdefault(e['event'], []).append(e)
    h = by.get('wpn-hash', [{}])[0]
    print(f"scope {args.scope}  at {h.get('at')}  table {h.get('table')}  launch {h.get('launch')}  flight {h.get('flight')}  terminal {h.get('terminal')}  decision {h.get('decision')}")
    for e in by.get('wpn-counter', []):
        if args.all_counters or e['value']:
            print(f"  {e['name']:36} {e['value']}")
    for e in by.get('wpn-range', []):
        print(f"  range {e['name']:28} n={e['n']} min={e['min']} max={e['max']}")
    for name in ('wpn-path', 'wpn-kill', 'wpn-outcome'):
        for e in by.get(name, []):
            print(f"  {name[4:]:8} {e.get('name') or e.get('code'):24} {e['count']}")
    for e in by.get('wpn-guidance', []):
        print(f"  guidance {e['name']:20} valid {e['target_position_valid']} invalid {e['target_position_invalid']}")
    for e in by.get('wpn-launcher', []):
        print(f"  launcher {e['kind']:22} births {e['births']} terminals {e['terminals']}")
    for e in by.get('wpn-weapon', []):
        kills = {k[5:]: v for k, v in e.items() if k.startswith('kill_') and v}
        print(f"  weapon {e['sub_type']:4} {e['name']:34} births {e['births']} moves {e['moves']} guided {e['guided']} unguided {e['unguided']} loal {e['loal']} {kills}")
    for e in by.get('wpn-table', []):
        print(f"  table {e['sub_type']:4} {e['name']:34} cells {e['cells']} hash {e['hash']} flights {e['flights']} full {e['full']} short {e['short']}")
    for name in ('wpn-tables', 'wpn-samples', 'wpn-log'):
        for e in by.get(name, []):
            print(' ', name, {k: v for k, v in e.items() if k not in ('event', 'scope', 'at')})
    kinds = collections.Counter(e.get('kind') for e in log)
    print('  ordered log kinds:', dict(kinds))


if __name__ == '__main__':
    main()
