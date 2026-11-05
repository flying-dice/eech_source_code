#!/usr/bin/env python3
# TEST-ONLY (EEDCS-43): prints the run-wide (or one probe's) report of a lua/sensor-characterization.lua trace as a compact text table.
#
# Usage: tools/sensor-trace-summary.py <trace.jsonl> [--scope run|probe] [--at <seconds>]
import argparse
import json
import sys

FUNCTIONS = ['air_to_air', 'air_to_surface', 'surface_to_air', 'surface_to_surface']
VERDICTS = ['accept', 'range', 'range_adjusted', 'altitude', 'tolerance', 'line_of_sight']
SKIPS = ['self', 'side', 'identify', 'airborne', 'target_type', 'no_group', 'best_group', 'dead']


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('trace')
    parser.add_argument('--scope', default='run')
    parser.add_argument('--at', type=float)
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

    def per_function(event):
        return {e['name']: e for e in by.get(event, [])}

    h = by.get('sensor-hash', [{}])[0]
    print(f"scope {args.scope}  physical_hash {h.get('physical')}  decision_hash {h.get('decision')}")
    fn, ver, skp, edg, geo = (per_function(n) for n in ('sensor-fn', 'sensor-verdicts', 'sensor-skips', 'sensor-edges', 'sensor-geometry'))
    print(f"{'':20}" + ''.join(f'{f:>20}' for f in FUNCTIONS))
    rows = [('calls', fn, 'calls'), ('scanned', fn, 'scanned'), ('non-null group', fn, 'nonnull'), ('target_count sum', fn, 'target_count_sum'),
            ('count>0, null group', fn, 'count_positive_null_group'), ('candidates', fn, 'candidates'), ('displaced best', fn, 'replaced')]
    rows += [(f'verdict {v}', ver, v) for v in VERDICTS] + [(f'skip {s}', skp, s) for s in SKIPS]
    rows += [('player candidates', edg, 'player_candidates'), ('emitter off', edg, 'emitter_off'), ('stealth != 1', edg, 'stealth_not_one'),
             ('adjusted != raw', edg, 'adjusted_differs'), ('sector whole map', geo, 'sector_whole_map'), ('sector empty', geo, 'sector_empty'),
             ('scan range min', geo, 'scan_range_min'), ('scan range max', geo, 'scan_range_max'), ('cos values', geo, 'cos_values'),
             ('cos min', geo, 'cos_min'), ('cos max', geo, 'cos_max'), ('cos outside [-1,1]', geo, 'cos_outside_unit')]
    for label, table, key in rows:
        print(f'{label:20}' + ''.join(f"{str(table.get(f, {}).get(key, '')):>20}" for f in FUNCTIONS))
    for name in ('sensor-surface-to-air', 'sensor-tolerance', 'sensor-probe'):
        for e in by.get(name, []):
            print(name, {k: v for k, v in e.items() if k not in ('event', 'scope')})
    for e in by.get('sensor-kind', []):
        print('kind', e['source'], e['function'], 'calls', e['calls'], 'scanned', e['scanned'])


if __name__ == '__main__':
    sys.exit(main())
