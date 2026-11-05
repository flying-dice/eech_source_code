#!/usr/bin/env python3
# What a change did to a campaign: the metrics of two regression runs of the
# same scenarios (tools/regress-windows.ps1 -Out <dir>), compared at the last
# checkpoint, supply first. Used by tools/mutation-windows.ps1 to explain a
# mutant's failure against its control.
#
# Usage: tools/metrics-difference.py <control dir> <mutant dir>
import json, os, sys

SUPPLY_FIRST = ('resupplied', 'supply', 'sorties', 'captures', 'keysite_states', 'lost', 'spawned', 'launched', 'alive')


def flat(checkpoint):
    out = {}
    for group, values in checkpoint.get('counts', {}).items():
        for k, v in values.items():
            out[(group, k)] = v
    for group in ('keysites', 'keysite_states', 'supply', 'alive'):
        for k, v in checkpoint.get(group, {}).items():
            out[(group, k)] = v
    return out


def main():
    control, mutant = sys.argv[1:3]
    for scenario in ('georgia_retail', 'lebanon_retail'):
        a = json.load(open(os.path.join(control, scenario + '.json')))['checkpoints'][-1]
        b = json.load(open(os.path.join(mutant, scenario + '.json')))['checkpoints'][-1]
        fa, fb = flat(a), flat(b)
        keys = sorted(set(fa) | set(fb), key=lambda k: (SUPPLY_FIRST.index(k[0]) if k[0] in SUPPLY_FIRST else 99, k[1]))
        diffs = [(k, fa.get(k, 0), fb.get(k, 0)) for k in keys if fa.get(k, 0) != fb.get(k, 0)]
        print(f"{scenario} at {b['seconds']} s: {'IDENTICAL' if not diffs else f'{len(diffs)} of {len(keys)} final metrics differ (control -> mutant)'}")
        supply = [d for d in diffs if d[0][0] in ('resupplied', 'supply') or (d[0][0] in ('sorties', 'sortie_members') and d[0][1].endswith('SUPPLY'))]
        for (group, k), x, y in supply:
            print(f"  {group:12} {k:45} {x:>7} -> {y}")
        rest = [d for d in diffs if d not in supply]
        if rest:
            print(f"  and {len(rest)} other metrics, largest first:")
            rest.sort(key=lambda d: -abs((d[2] or 0) - (d[1] or 0)))
            for (group, k), x, y in rest[:12]:
                print(f"  {group:12} {k:45} {x:>7} -> {y}")
    return 0


if __name__ == '__main__':
    sys.exit(main())
