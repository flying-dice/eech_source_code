#!/usr/bin/env python3
# Summarises an eech-world Tacview recording and its log: losses and weapons
# launched per side, keysite captures and the task mix.
#
# Usage: tools/acmi-summary.py <recording.acmi> [<run log>]
import collections, re, sys

objects = {}
destroyed = collections.Counter()
launched = collections.Counter()
first_frame = last_frame = None
for line in open(sys.argv[1], encoding='utf-8', errors='replace'):
    if line.startswith('#'):
        t = float(line[1:])
        first_frame = t if first_frame is None else first_frame
        last_frame = t
        continue
    ident, _, rest = line.rstrip('\n').partition(',')
    if ident == '0':
        if rest.startswith('Event=Destroyed|'):
            o = objects.get(rest.split('|')[1])
            if o:
                destroyed[(o['side'], o['kind'])] += 1
        continue
    if 'Type=' in rest and ident not in objects:
        props = dict(p.split('=', 1) for p in re.split(r',(?=[A-Za-z]+=)', rest) if '=' in p)
        kind = props['Type']
        kind = {'Air+Rotorcraft': 'helicopters', 'Air+FixedWing': 'fixed wing', 'Ground+Vehicle': 'vehicles',
                'Ground+Light+Human+Infantry': 'infantry', 'Ground+AntiAircraft': 'air defence',
                'Sea+Watercraft': 'ships', 'Ground+Static+Aerodrome': 'keysites'}.get(kind, kind)
        side = props.get('Color', '?')
        objects[ident] = {'side': side, 'kind': kind}
        if kind.startswith('Weapon'):
            launched[side] += 1

print(f'frames {first_frame:.0f}-{last_frame:.0f} s')
for side in ('Blue', 'Red'):
    losses = {k: n for (s, k), n in destroyed.items() if s == side}
    print(f'{side}: destroyed {sum(losses.values())}:', ', '.join(f'{n} {k}' for k, n in sorted(losses.items(), key=lambda x: -x[1])),
          f'| weapons launched {launched[side]}')

if len(sys.argv) > 2:
    tasks = collections.defaultdict(collections.Counter)
    for line in open(sys.argv[2], encoding='utf-8', errors='replace'):
        if 'captured by' in line:
            print(line.strip())
        m = re.search(r'tasks: (.*)', line)
        if m:
            # each diagnostic snapshot: the largest count of each task seen at once
            snap = collections.Counter()
            for side, task, n in re.findall(r'(blue|red) ([A-Z_]+)/[^=]+=(\d+)', m.group(1)):
                snap[(side, task)] += int(n)
            for k, n in snap.items():
                tasks[k[0]][k[1]] = max(tasks[k[0]][k[1]], n)
    for side, c in tasks.items():
        print(f'{side} tasks (peak concurrent):', ', '.join(f'{t} {n}' for t, n in c.most_common()))
