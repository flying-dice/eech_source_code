#!/usr/bin/env python3
"""Compares campaign metrics (campaign.lua metrics=..., see lua/metrics.lua)
with a baseline, for the regression test (tools/regress.sh).

The runs are deterministic, so an unchanged engine reproduces the baseline
exactly: that is reported as IDENTICAL. When it does not, the comparison is
on aggregates, at every checkpoint, with tolerances, so that a change that
moves individual events but not the war's effects still passes:

  sorties        per side, kind and task      the tasking
  aircraft       aircraft sorties per side    the air effort
  launched       weapons per side             combat
  lost, spawned  per side and kind            attrition, regen and reinforcement
  captures       per side and keysite type    the front
  keysites       held, per side and type      the front
  states         per side, type and state     damage and repair
  supply         mean ammo/fuel, side, type   production and supply
  alive          per side and kind            the forces

Usage: regress-compare.py <baseline.json> <current.json> [--exact]
Exit status: 0 identical or within tolerance, 1 outside it (or not identical with --exact).
"""
import collections, json, sys

# relative tolerance on counts, with an absolute floor; absolute on supply (percentage points)
REL, ABS = 0.20, 3
SUPPLY_ABS = 15.0


def aggregates(cp):
    """the compared metrics of one checkpoint: {group: {key: value}}"""
    c = cp['counts']
    out = collections.defaultdict(dict)
    out['sorties'] = dict(c['sorties'])
    air = collections.Counter()
    for k, n in c['airframes'].items():
        air[k.split(' ', 1)[0]] += n
    out['aircraft'] = dict(air)
    launched = collections.Counter()
    for k, n in c['launched'].items():
        launched[k.split(' ', 1)[0]] += n
    out['launched'] = dict(launched)
    out['lost'] = dict(c['lost'])
    out['spawned'] = dict(c['spawned'])
    out['captures'] = dict(c['captures'])
    out['keysites'] = dict(cp['keysites'])
    out['states'] = dict(cp['keysite_states'])
    out['supply'] = dict(cp['supply'])
    out['alive'] = dict(cp['alive'])
    return out


def within(group, base, cur):
    if group == 'supply':
        return abs(cur - base) <= SUPPLY_ABS
    return abs(cur - base) <= max(ABS, REL * abs(base))


def compare(baseline, current, exact):
    if baseline == current:
        return True, ['IDENTICAL to the baseline']
    lines, ok = [], True
    if baseline['run'] != current['run']:
        lines.append(f"run settings differ: baseline {baseline['run']}, current {current['run']}")
        ok = False
    bcp = {cp['seconds']: cp for cp in baseline['checkpoints']}
    ccp = {cp['seconds']: cp for cp in current['checkpoints']}
    if sorted(bcp) != sorted(ccp):
        lines.append(f'checkpoints differ: baseline {sorted(bcp)}, current {sorted(ccp)}')
        ok = False
    for t in sorted(set(bcp) & set(ccp)):
        ba, ca = aggregates(bcp[t]), aggregates(ccp[t])
        for group in ba:
            for key in sorted(set(ba[group]) | set(ca[group])):
                b, c = ba[group].get(key, 0), ca[group].get(key, 0)
                if b == c:
                    continue
                good = within(group, b, c)
                ok = ok and good
                lines.append(f"{'  ok ' if good else 'FAIL '} {t / 3600:4.1f} h  {group:9s} {key:45s} {b:>8} -> {c}")
    if exact:
        ok = False
    lines.insert(0, ('within tolerance of the baseline' if ok else 'OUTSIDE tolerance of the baseline') +
                 ('' if not exact else ' (--exact: not identical)'))
    return ok, lines


def summary(m):
    """a short human summary of a metrics file: start and end"""
    cps = m['checkpoints']
    first, last = cps[0], cps[-1]
    a0, a1 = aggregates(first), aggregates(last)
    out = [f"{m['run']['scenario']}: {last['seconds'] / 3600:.1f} h, conclusion: {m['run']['conclusion']}"]
    for side in ('blue', 'red'):
        tasks = collections.Counter()
        for k, n in a1['sorties'].items():
            s, kind, task = k.split(' ', 2)
            if s == side:
                tasks[task] += n
        out.append(f"  {side} sorties {sum(tasks.values())}: " + ', '.join(f'{t} {n}' for t, n in tasks.most_common()))
        out.append(f"  {side} aircraft sorties {a1['aircraft'].get(side, 0)}, weapons launched {a1['launched'].get(side, 0)}")
        out.append(f"  {side} lost: " + ', '.join(f"{k.split(' ', 1)[1]} {n}" for k, n in sorted(a1['lost'].items()) if k.startswith(side + ' ')))
        out.append(f"  {side} spawned: " + ', '.join(f"{k.split(' ', 1)[1]} {n}" for k, n in sorted(a1['spawned'].items()) if k.startswith(side + ' ')))
        caps = {k.split(' ', 1)[1]: n for k, n in a1['captures'].items() if k.startswith(side + ' ')}
        out.append(f"  {side} captured: {caps or 'nothing'}")
        for kind in ('KEYSITE_AIRBASE', 'KEYSITE_FARP', 'KEYSITE_FACTORY', 'KEYSITE_OIL_REFINERY'):
            key = f'{side} {kind}'
            if key + ' ammo' in a1['supply'] or key + ' ammo' in a0['supply']:
                out.append(f"  {side} {kind[8:].lower():12s} held {a0['keysites'].get(key, 0)} -> {a1['keysites'].get(key, 0)}, "
                           f"ammo {a0['supply'].get(key + ' ammo', '-')} -> {a1['supply'].get(key + ' ammo', '-')}, "
                           f"fuel {a0['supply'].get(key + ' fuel', '-')} -> {a1['supply'].get(key + ' fuel', '-')}")
    return out


def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    exact = '--exact' in sys.argv
    if len(args) == 1:
        print('\n'.join(summary(json.load(open(args[0])))))
        return 0
    if len(args) != 2:
        print(__doc__)
        return 2
    baseline, current = (json.load(open(p)) for p in args)
    ok, lines = compare(baseline, current, exact)
    print('\n'.join(summary(current)))
    print('\n'.join(lines))
    return 0 if ok else 1


if __name__ == '__main__':
    sys.exit(main())
