#!/usr/bin/env python3
# Evidence, per campaign mechanic, from an eech-world recording and its log:
# weapons by type and side, losses by type, aircraft rebuilt from reserves,
# captures, ships, and keysite supply over time (from the diagnostics).
#
# Usage: tools/mechanics-check.py <recording.acmi> <run log>
import collections, re, sys

info, destroyed, launched, rebuilt = {}, collections.Counter(), collections.Counter(), collections.Counter()
declared_at = {}
t = 0.0
for line in open(sys.argv[1], encoding='utf-8', errors='replace'):
    if line.startswith('#'):
        t = float(line[1:])
        continue
    ident, _, rest = line.rstrip('\n').partition(',')
    if ident == '0':
        if rest.startswith('Event=Destroyed|'):
            o = info.get(rest.split('|')[1])
            if o:
                destroyed[(o.get('Color'), o.get('Type'), o.get('Name'))] += 1
        continue
    if ident.startswith('-'):
        info.pop(ident[1:], None)
        continue
    if 'Type=' in rest and ident not in info:
        o = dict(kv.split('=', 1) for kv in re.split(r',(?=[A-Za-z]+=)', rest) if '=' in kv)
        info[ident] = o
        if o['Type'].startswith('Weapon'):
            launched[(o.get('Color'), o['Type'], o.get('Name'))] += 1
        elif o['Type'].startswith('Air') and t > 60:
            rebuilt[(o.get('Color'), o.get('Name'))] += 1

def table(title, counter, top=12):
    print(f'\n{title}')
    for side in ('Blue', 'Red'):
        items = sorted(((n, k) for k, n in counter.items() if k[0] == side), reverse=True)[:top]
        print(f'  {side}: ' + ', '.join(f'{k[-1]} {n}' for n, k in items))

print(f'recording to {t:.0f} s')
table('Weapons launched (top)', launched)
table('Destroyed (top)', destroyed)
for kind in ('Air+FixedWing', 'Air+Rotorcraft', 'Sea+Watercraft', 'Ground+AntiAircraft'):
    by = collections.Counter({s: n for (s, k, _), n in destroyed.items() if k == kind for s in [s]})
    by = collections.Counter()
    for (s, k, _), n in destroyed.items():
        if k == kind: by[s] += n
    print(f'  {kind} destroyed: {dict(by)}')
aam = collections.Counter()
for (s, k, n), c in launched.items():
    if re.search(r'AIM-|R-27|R-73|R-77|Stinger|Igla|SA-|Chaparral|Sidewinder|Sparrow|AMRAAM|Standard|Strela|Hellfire|Vikhr|Ataka|Kh-|Maverick|Harpoon|Shturm', n or ''):
        aam[(s, n)] += c
print('\nAircraft appearing after the first minute (rebuilt from reserves or transferred):', dict(collections.Counter({s: 0 for s in ()}) + collections.Counter({k[0]: 0 for k in ()})) or '')
for side in ('Blue', 'Red'):
    items = [(n, k[1]) for k, n in rebuilt.items() if k[0] == side]
    print(f'  {side}: {sum(n for n, _ in items)}: ' + ', '.join(f'{name} {n}' for n, name in sorted(items, reverse=True)))

caps, sorties, supply = [], [], collections.defaultdict(list)
for line in open(sys.argv[2], encoding='utf-8', errors='replace'):
    if 'captured by' in line: caps.append(line.split('] ', 1)[-1].strip())
    if line.startswith('[campaign] sorties'): sorties.append(line.split('sorties ', 1)[1].strip())
    m = re.match(r'eech:   keysite (.+?) (KEYSITE_\w+) usable=(.+?) efficiency=([\d.]+) ammo=([\d.]+) fuel=([\d.]+)', line)
    if m and m.group(2) in ('KEYSITE_AIRBASE', 'KEYSITE_FACTORY', 'KEYSITE_OIL_REFINERY', 'KEYSITE_FARP'):
        supply[(m.group(1), m.group(2))].append((m.group(3), float(m.group(4)), float(m.group(5)), float(m.group(6))))
print(f'\nCaptures: {len(caps)}'); [print('  ' + c) for c in caps]
print('\nSupply over time (airbases; ammo/fuel per diagnostics snapshot):')
for (name, kind), series in sorted(supply.items()):
    if kind != 'KEYSITE_AIRBASE': continue
    rises = sum(1 for a, b in zip(series, series[1:]) if b[2] > a[2] + 1 or b[3] > a[3] + 1)
    print(f'  {name:24s} rises {rises:2d}: ' + ' '.join(f'{a:.0f}/{f:.0f}' for _, _, a, f in series[:16]))
states = collections.Counter(s[-1][0] for (n, k), s in supply.items() if k in ('KEYSITE_FACTORY', 'KEYSITE_OIL_REFINERY'))
print(f'  producers at the end: {dict(states)}')
if sorties:
    print('\nSorties:'); [print('  ' + s) for s in sorties]
