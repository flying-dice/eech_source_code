#!/usr/bin/env python3
# Checks one eech-world run's two forms of world evidence against each other:
# the Tacview recording (acmi=) and the structured observations (observe=,
# lua/observations.lua), written from the same samples; and both against the
# run's metrics (metrics=). See docs/m2-reference.md.
#
#   identity and lifecycle  the observations, replayed through the recorder's
#       rules (crates/eech-world/src/main.rs, Recording::record), must give
#       exactly the recording's object declarations, removals and Destroyed
#       events, frame by frame, with the declared type, name and coalition
#   position  at every observation snapshot, each recorded object's last
#       Tacview position, projected back to EECH map metres by this script's
#       own implementation of the map projection, must be within the
#       recorder's change threshold (0.5 m per axis, 1 degree of heading) plus
#       the file's rounding
#   losses and captures  deaths and keysite side changes must agree between
#       the observations, the recording and the metrics
#   keysite state  the last snapshot's keysites, usable states and mean supply
#       must equal the metrics' last checkpoint
#   plausibility (findings, not failures)  speeds between consecutive Tacview
#       positions above a generous limit per kind; keysites that move
#
# Usage: tools/observation-check.py <recording.acmi[.zip]> <observations.jsonl[.gz]> <metrics.json> [--report <file.json>]
# Exit status 1 if a check fails.
import argparse, collections, gzip, io, json, math, re, sys, zipfile

KIND_OF_TYPE = {
    'Air+Rotorcraft': 'helicopter', 'Air+FixedWing': 'fixed_wing', 'Ground+Vehicle': 'ground_vehicle',
    'Ground+AntiAircraft': 'air_defence', 'Sea+Watercraft': 'ship', 'Ground+Light+Human+Infantry': 'infantry',
    'Ground+Static+Aerodrome': 'keysite',
}
COALITION = {'blue': 'United States', 'red': 'Russia'}
# m/s: well above what each kind can do, so a flag is a jump, not a fast mover
MAX_SPEED = {'infantry': 20, 'ground_vehicle': 50, 'air_defence': 50, 'ship': 30, 'helicopter': 150, 'fixed_wing': 900, 'weapon': 1800}
POSITION_TOLERANCE = 0.5 + 0.02     # the recorder's threshold, plus 7-decimal degrees (under 1 cm)
ALTITUDE_TOLERANCE = 0.5 + 0.06     # plus 0.1 m altitude rounding
HEADING_TOLERANCE = 1.0 + 0.06      # plus 0.1 degree rounding


def open_text(path):
    if zipfile.is_zipfile(path):
        z = zipfile.ZipFile(path)
        return io.TextIOWrapper(z.open(z.namelist()[0]), encoding='utf-8', errors='replace', newline='')
    if path.endswith('.gz'):
        return gzip.open(path, 'rt', encoding='utf-8', newline='')
    return open(path, encoding='utf-8', errors='replace', newline='')


def tacview_type(kind, type_name):
    """the recorder's Type for an observed kind (main.rs tacview_type)"""
    if kind == 'weapon':
        t = type_name.lower()
        if any(s in t for s in ('rocket', 's-8', 's-13', 'hydra')):
            return 'Weapon+Rocket'
        if any(s in t for s in ('bomb', 'mk-8', 'fab')):
            return 'Weapon+Bomb'
        return 'Weapon+Missile'
    return {v: k for k, v in KIND_OF_TYPE.items()}.get(kind, 'Ground+Static+Aerodrome')


def unescape(s):
    return re.sub(r'\\(.)', r'\1', s)


def acmi_frames(path, header):
    """(time string, ops, position updates) per frame; fills header from the global lines"""
    frame, ops, updates = None, [], []
    for line in open_text(path):
        line = line.rstrip('\r\n')
        if not line:
            continue
        if line.startswith('#'):
            if frame is not None:
                yield frame, ops, updates
            frame, ops, updates = line[1:], [], []
            continue
        if frame is None:
            key, _, value = line.partition('=')
            header[key.split(',')[-1]] = value
            continue
        if line.startswith('-'):
            ops.append(('remove', int(line[1:], 16)))
            continue
        ident, _, rest = line.partition(',')
        if ident == '0':
            if rest.startswith('Event=Destroyed|'):
                ops.append(('destroyed', int(rest.split('|')[1], 16)))
            elif rest.startswith('Event=Message|'):
                ops.append(('message', unescape(rest[len('Event=Message|'):])))
            continue
        oid = int(ident, 16)
        props = {}
        for part in re.split(r'(?<!\\),', rest):
            k, _, v = part.partition('=')
            props[k] = v
        if 'Type' in props:
            ops.append(('declare', oid, props['Type'], unescape(props.get('Name', '')), props.get('Coalition', ''),
                        unescape(props['Group']) if 'Group' in props else None))
        if 'T' in props:
            updates.append((oid, props['T'].split('|')))
    if frame is not None:
        yield frame, ops, updates


def observation_groups(path, header):
    """(time string, records) per sample time; the header record into header"""
    group, records = None, []
    for line in open_text(path):
        r = json.loads(line)
        if r['ev'] == 'header':
            header.update(r)
            continue
        t = r.pop('t')
        if t != group:
            if group is not None:
                yield group, records
            group, records = t, []
        records.append(r)
    if group is not None:
        yield group, records


class Projection:
    """EECH map metres <-> degrees, as the scenario's affine map projection defines it"""

    def __init__(self, affine, ref_lat, ref_lon):
        a, b, c, d, tx, tz, lat0, lon0 = affine
        self.m, self.t, self.lat0, self.lon0 = (a, b, c, d), (tx, tz), lat0, lon0
        self.ref = (ref_lat, ref_lon)

    def map_of(self, dlon, dlat):
        lat, lon = self.ref[0] + dlat, self.ref[1] + dlon
        n = (lat - self.lat0) * 110574.0
        e = (lon - self.lon0) * 111320.0 * math.cos(math.radians(lat))
        a, b, c, d = self.m
        return a * e + b * n + self.t[0], c * e + d * n + self.t[1]


def wrap_degrees(a):
    return (a + 180.0) % 360.0 - 180.0


def main():
    p = argparse.ArgumentParser()
    p.add_argument('acmi')
    p.add_argument('observations')
    p.add_argument('metrics')
    p.add_argument('--report')
    a = p.parse_args()

    acmi_header, obs_header = {}, {}
    frames = acmi_frames(a.acmi, acmi_header)
    observations = observation_groups(a.observations, obs_header)
    failures = []
    def fail(msg):
        if len(failures) < 50:
            failures.append(msg)
        counts['failures'] += 1

    counts = collections.Counter()
    tracked = {}            # EECH index -> {'acmi', 'kind', 'side', 'alive', 'eech'}
    by_acmi = {}            # Tacview id -> {'kind', 'side', 'eech', 'declared'}
    info = {}               # EECH index -> the last observed kind, sub-type, side, type, name
    next_acmi = 0x100
    acmi_pos = {}           # Tacview id -> [lon, lat, alt, roll, pitch, heading] (strings; '' = unchanged)
    last_map = {}           # Tacview id -> (time, x, y, z) of its last position update
    lost = {'observations': collections.Counter(), 'recording': collections.Counter()}
    captures = {'observations': collections.Counter(), 'recording': collections.Counter()}
    usable_changes = collections.Counter()
    deviation = collections.defaultdict(lambda: [0.0, 0.0, 0.0, 0.0])   # kind -> max |dx|, |dy|, |dz|, |dh|
    pitch_roll = [0.0, 0.0]
    speed_flags = collections.Counter()
    speed_examples = collections.defaultdict(list)
    moved_keysites = 0
    keysite_home = {}
    last_snapshot = None
    projection = None

    pending = next(observations, None)
    first_t = previous_t = None
    intervals = [math.inf, 0.0]
    for t, ops, updates in frames:
        if projection is None:
            affine = [float(x) for x in obs_header['affine'].split()]
            projection = Projection(affine, float(acmi_header['ReferenceLatitude']), float(acmi_header['ReferenceLongitude']))
        counts['frames'] += 1
        if previous_t is not None:
            step = float(t) - previous_t
            if step <= 0:
                fail(f'{t}: frame time does not increase')
            intervals[0], intervals[1] = min(intervals[0], step), max(intervals[1], step)
        first_t = float(t) if first_t is None else first_t
        previous_t = float(t)
        records = []
        # both files print the sample time with two decimals, so the parsed numbers are equal
        while pending is not None and float(pending[0]) < float(t):
            fail(f'{pending[0]}: observations with no Tacview frame')
            pending = next(observations, None)
        if pending is not None and float(pending[0]) == float(t):
            records = pending[1]
            pending = next(observations, None)

        # the recorder's rules, applied to the raw observations: the operations the recording must hold
        expected, messages, snaps = [], collections.Counter(), []

        def declare(eech):
            nonlocal next_acmi
            o = info[eech]
            name = o['name'] if o['kind'] == 'keysite' and o.get('name') else o['type']
            group = o.get('name') if o['kind'] != 'keysite' else None
            expected.append(('declare', next_acmi, tacview_type(o['kind'], o['type']), name, COALITION.get(o['side'], 'Neutral'), group))
            tracked[eech] = {'acmi': next_acmi, 'kind': o['kind'], 'side': o['side'], 'alive': True}
            by_acmi[next_acmi] = {'kind': o['kind'], 'side': o['side'], 'eech': eech}
            counts['declared ' + o['kind']] += 1
            next_acmi += 1

        for r in records:
            ev, eech = r['ev'], r.get('id')
            if ev in ('appear', 'retype'):
                info[eech] = {k: r.get(k) for k in ('kind', 'sub', 'side', 'type', 'name')}
                info[eech]['alive'] = r['alive']
                if eech in tracked:
                    expected.append(('remove', tracked.pop(eech)['acmi']))
                if r['alive']:
                    declare(eech)
                else:
                    counts['appeared dead (never recorded)'] += 1
            elif ev == 'alive':
                info[eech]['alive'] = r['alive']
                if r['alive']:
                    counts['index reused by a live entity of the same type'] += 1
                    if eech in tracked:
                        expected.append(('remove', tracked.pop(eech)['acmi']))
                    declare(eech)
                elif eech in tracked and tracked[eech]['alive']:
                    tracked[eech]['alive'] = False
                    expected.append(('destroyed', tracked[eech]['acmi']))
                    lost['observations'][f"{info[eech]['side']} {info[eech]['kind']}"] += 1
            elif ev == 'side':
                info[eech]['side'] = r['side']
                if info[eech]['kind'] == 'keysite':
                    messages[f"{info[eech]['name']} captured by {r['side']}"] += 1
                    captures['observations'][f"{r['side']} {info[eech]['type']}"] += 1
            elif ev == 'usable':
                usable_changes[f"{r['from']} -> {r['usable']}"] += 1
            elif ev == 'gone':
                if eech in tracked:
                    expected.append(('remove', tracked.pop(eech)['acmi']))
                info.pop(eech, None)
                counts['gone ' + r['kind']] += 1
            elif ev == 'snap':
                snaps.append(r)

        recorded = [op for op in ops if op[0] != 'message']
        if recorded != expected:
            counts['frames with different operations'] += 1
            fail(f'{t}: the recording holds {len(recorded)} operations, the observations give {len(expected)}; first difference: '
                 + str(next(((x, y) for x, y in zip(recorded, expected) if x != y), (recorded[len(expected):len(expected) + 1], expected[len(recorded):len(recorded) + 1]))))
        for op in recorded:
            if op[0] == 'destroyed' and op[1] in by_acmi:
                o = by_acmi[op[1]]
                lost['recording'][f"{o['side']} {o['kind']}"] += 1
        recorded_messages = collections.Counter(op[1] for op in ops if op[0] == 'message' and ' captured by ' in op[1])
        for msg, n in recorded_messages.items():
            name, _, side = msg.rpartition(' captured by ')
            kind = next((info[e]['type'] for e in info if info[e].get('name') == name and info[e]['kind'] == 'keysite'), '?')
            captures['recording'][f'{side} {kind}'] += n
        if recorded_messages != messages:
            fail(f'{t}: capture messages {dict(recorded_messages)}, observed side changes {dict(messages)}')

        # positions written this frame, and the plausibility of each move
        for oid, fields in updates:
            cur = acmi_pos.setdefault(oid, [''] * 6)
            for i, v in enumerate(fields):
                if v != '':
                    cur[i] = v
            if oid not in by_acmi:
                continue
            x, z = projection.map_of(float(cur[0]), float(cur[1]))
            y, now = float(cur[2]), float(t)
            kind = by_acmi[oid]['kind']
            if oid in last_map:
                t0, x0, y0, z0 = last_map[oid]
                if now > t0 and kind in MAX_SPEED:
                    speed = math.dist((x, y, z), (x0, y0, z0)) / (now - t0)
                    if speed > MAX_SPEED[kind]:
                        speed_flags[kind] += 1
                        ex = speed_examples[kind]
                        ex.append((round(speed), t, by_acmi[oid]['eech'], round(math.dist((x, z), (x0, z0)))))
                        ex.sort(reverse=True)
                        del ex[5:]
            last_map[oid] = (now, x, y, z)

        # every snapshot object the recorder tracks: its Tacview position against the observed one
        for r in snaps:
            last_snapshot = (t, snaps)
            eech = r['id']
            if r['kind'] == 'keysite':
                home = keysite_home.setdefault(eech, (r['x'], r['z']))
                if math.dist(home, (r['x'], r['z'])) > 1.0:
                    moved_keysites += 1
            tr = tracked.get(eech)
            if not tr:
                counts['snapshot objects not recorded (appeared dead)'] += 1
                continue
            cur = acmi_pos.get(tr['acmi'])
            if not cur or cur[0] == '':
                fail(f'{t}: object {eech} (Tacview {tr["acmi"]:x}) has no recorded position')
                continue
            counts['position comparisons'] += 1
            x, z = projection.map_of(float(cur[0]), float(cur[1]))
            d = deviation[r['kind']]
            dx, dy, dz = abs(x - r['x']), abs(float(cur[2]) - r['y']), abs(z - r['z'])
            d[0], d[1], d[2] = max(d[0], dx), max(d[1], dy), max(d[2], dz)
            bad = dx > POSITION_TOLERANCE or dz > POSITION_TOLERANCE or dy > ALTITUDE_TOLERANCE
            if r['kind'] != 'keysite':
                dh = abs(wrap_degrees(float(cur[5]) - math.degrees(r['h']) % 360.0))
                d[3] = max(d[3], dh)
                bad = bad or dh > HEADING_TOLERANCE
                pitch_roll[0] = max(pitch_roll[0], abs(float(cur[4]) - math.degrees(r['p'])))
                pitch_roll[1] = max(pitch_roll[1], abs(wrap_degrees(float(cur[3]) + math.degrees(r['r']))))
            if bad:
                counts['positions outside tolerance'] += 1
                fail(f'{t}: {r["kind"]} {eech} observed ({r["x"]:.2f}, {r["y"]:.2f}, {r["z"]:.2f}), recorded ({x:.2f}, {float(cur[2]):.2f}, {z:.2f})')

    if pending is not None:
        fail(f'{pending[0]}: observations after the last Tacview frame')

    # the metrics: losses, captures, and the keysite state at the last checkpoint
    metrics = json.load(open(a.metrics))
    final = metrics['checkpoints'][-1]
    lost['metrics'] = collections.Counter({k: v for k, v in final['counts'].get('lost', {}).items() if v})
    captures['metrics'] = collections.Counter({k.replace('KEYSITE_', 'KEYSITE_', 1): v for k, v in final['counts'].get('captures', {}).items() if v})
    for label, forms in (('losses', lost), ('captures', captures)):
        if not (forms['observations'] == forms['recording'] == forms['metrics']):
            fail(f"{label}: observations {dict(forms['observations'])}, recording {dict(forms['recording'])}, metrics {dict(forms['metrics'])}")
    keysite_check = 'not compared (the last snapshot is not at the last checkpoint)'
    if last_snapshot and abs(float(last_snapshot[0]) - final['seconds']) < 1:
        held, states, supply = collections.Counter(), collections.Counter(), collections.defaultdict(list)
        for r in last_snapshot[1]:
            if r['kind'] == 'keysite':
                key = f"{r['side']} {r['type']}"
                held[key] += 1
                states[f"{key} {r['usable']}"] += 1
                supply[key].append((r['ammo'], r['fuel']))
        mean = {}
        for key, v in supply.items():
            mean[key + ' ammo'] = sum(x[0] for x in v) / len(v)
            mean[key + ' fuel'] = sum(x[1] for x in v) / len(v)
        off = [k for k in final['supply'] if abs(mean.get(k, -99) - final['supply'][k]) > 0.051]
        if held != collections.Counter(final['keysites']) or states != collections.Counter(final['keysite_states']) or off or set(mean) != set(final['supply']):
            fail(f'keysite state at {final["seconds"]} s differs from the metrics: held {dict(held)} vs {final["keysites"]}; supply {off}')
            keysite_check = 'DIFFERENT'
        else:
            keysite_check = f'identical at {final["seconds"]} s: {sum(held.values())} keysites, their usable states and mean supply'

    report = {
        'recording': a.acmi, 'observations': a.observations, 'metrics': a.metrics,
        'frames': counts['frames'], 'failures': counts['failures'], 'first_failures': failures,
        'time': {
            'first_s': first_t, 'last_s': previous_t, 'sample_interval_s': {'min': round(intervals[0], 3), 'max': round(intervals[1], 3)},
            # sample k (from 1) is nominally k * sample_every_s of host frame time
            'nominal_last_s': counts['frames'] * float(obs_header.get('sample_every_s', 1)),
            'session_clock_drift_s': round(previous_t - counts['frames'] * float(obs_header.get('sample_every_s', 1)), 3) if previous_t is not None else None,
        },
        'counts': {k: v for k, v in sorted(counts.items()) if k not in ('frames', 'failures')},
        'max_deviation': {k: {'x': round(v[0], 3), 'y': round(v[1], 3), 'z': round(v[2], 3), 'heading_deg': round(v[3], 3)} for k, v in sorted(deviation.items())},
        'max_pitch_roll_difference_deg': {'pitch': round(pitch_roll[0], 2), 'roll': round(pitch_roll[1], 2)},
        'losses': {k: dict(sorted(v.items())) for k, v in lost.items()},
        'captures': {k: dict(sorted(v.items())) for k, v in captures.items()},
        'keysite_usable_changes': dict(sorted(usable_changes.items())),
        'keysite_state_vs_metrics': keysite_check,
        'keysites_moved': moved_keysites,
        'speed_flags': dict(sorted(speed_flags.items())),
        'speed_flag_examples': {k: [{'speed_m_s': s, 't': tt, 'eech_id': e, 'jump_m': j} for s, tt, e, j in v] for k, v in sorted(speed_examples.items())},
        'speed_limits_m_s': MAX_SPEED,
    }
    print(json.dumps(report, indent=1))
    if a.report:
        with open(a.report, 'w', newline='\n') as f:
            json.dump(report, f, indent=1)
            f.write('\n')
    print('observation check: ' + ('PASS' if counts['failures'] == 0 else f"FAIL ({counts['failures']} failures)"), file=sys.stderr)
    return 1 if counts['failures'] else 0


if __name__ == '__main__':
    sys.exit(main())
