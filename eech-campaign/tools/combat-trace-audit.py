#!/usr/bin/env python3
"""Audit of a combat characterisation trace (EEDCS-46): lua/combat-characterization.lua's JSON Lines, written from the observer's report (csrc/eech_cmb_fixture.c).

usage: combat-trace-audit.py <trace.jsonl> [--run-json run.json] [--seconds 600] [--frame-ms 100] [--checkpoint-every 60] [--seed 1]
                             [--partial] [--max-bytes 268435456] [--summary out.json]

It reads the REAL schema the script writes (boot, checkpoint and run events; scope boot/checkpoint/run report lines; scope log ordered events) and decides, separately:

  PROCESS   the native process finished and exited 0, from run.json ONLY (status and actual_native_exit); the trace alone never says it
  ARTIFACT  the trace is complete and self-consistent: the boot configuration, every checkpoint, the final `run` event and its end point, six 16-hex-digit string hashes in
            every hash line, integrity (trusted, no overflow, no unbalanced delivery, no unknown identity, no release outside the entity array), the ordered event log (sequence
            contiguous from 1, every event of a known kind, blast begin/end paired, each explosion's blast count equal to the blasts it contained, the event counts equal to the
            report's counters), damage entry/exit balance, counters that never decrease
  COVERAGE  what the run MEASURED, per entity kind, warhead, kill code, timed roll, repair source and special path, and what it did not reach (reported, never a failure)

Exit status: 0 every evaluated check passed and the run is complete (PROCESS and ARTIFACT); 1 a check FAILED; 2 INCOMPLETE (no `run` event, or --partial, or run.json does not say
completed/0): the checks that could be evaluated passed, the rest are NOT_EVALUATED.  Nothing here changes the emitter, a golden or a tolerance; the file is read line by line
under --max-bytes and a line over 4 MiB is a failure, never a huge read.

Schema notes (facts of the emitter, not assumptions): the hash keys `damage collision blast repair timed order hash` are always strings, so in a `cmb-kind` row the COUNT `blast`
arrives as a decimal string; this audit reads it as an integer there and says so.  A run-wide counter that is a hash of nothing yet is the FNV offset basis cbf29ce484222325.
"""
import argparse
import json
import re
import sys

HASH_KEYS = ('damage', 'collision', 'blast', 'repair', 'timed', 'order')
HEX16 = re.compile(r'^[0-9a-f]{16}$')
EVENT_KINDS = {'collision', 'blast-begin', 'blast-end', 'explosion', 'repair', 'timed'}
KILL_CODES = {0: 'ok', 1: 'exhausted', 2: 'out_of_bounds', 3: 'hit_land', 4: 'hit_water', 5: 'hit_target', 6: 'overshot_target', 7: 'self_destruct'}
TIMED_NAMES = {1: 'helicopter_fire', 2: 'fixed_wing_fire', 3: 'person_bleeding', 4: 'anti_aircraft_fire', 5: 'routed_vehicle_fire', 6: 'ship_vehicle_fire'}
HEALTH_KINDS = ['ANTI_AIRCRAFT', 'CITY_BUILDING', 'FIXED_WING', 'HELICOPTER', 'OBJECT', 'PERSON', 'PYLON', 'ROUTED_VEHICLE', 'SCENIC', 'SEGMENT', 'SHIP_VEHICLE', 'SITE', 'SITE_UPDATABLE']
LINE_MAX = 4 * 1024 * 1024


class Audit:
    def __init__(self):
        self.results = []

    def add(self, section, name, status, detail=''):
        self.results.append({'section': section, 'name': name, 'status': status, 'detail': detail})

    def ok(self, section, name, condition, detail=''):
        self.add(section, name, 'PASS' if condition else 'FAIL', detail)

    def failed(self):
        return any(r['status'] == 'FAIL' for r in self.results)


def as_int(v):
    if isinstance(v, bool):
        raise ValueError('bool')
    if isinstance(v, int):
        return v
    if isinstance(v, float) and v == int(v):
        return int(v)
    if isinstance(v, str) and re.fullmatch(r'-?\d+', v):
        return int(v)
    raise ValueError(repr(v))


def read_trace(path, max_bytes):
    """-> (rows, problems). Streams the file by line; a file over max_bytes or a line over LINE_MAX is a problem, not a read."""
    rows, problems, size = [], [], 0
    with open(path, 'rb') as f:
        for number, raw in enumerate(f, 1):
            size += len(raw)
            if size > max_bytes:
                problems.append(f'file larger than --max-bytes {max_bytes}: stopped at line {number}')
                break
            if len(raw) > LINE_MAX:
                problems.append(f'line {number} longer than {LINE_MAX} bytes')
                break
            if not raw.endswith(b'\n'):
                problems.append(f'line {number} has no newline (a write in progress or a truncated file)')
                break
            try:
                row = json.loads(raw)
            except ValueError as error:
                problems.append(f'line {number} is not JSON: {error}')
                break
            if not isinstance(row, dict):
                problems.append(f'line {number} is not a JSON object')
                break
            rows.append(row)
    return rows, problems, size


def by(rows, scope, event):
    return [r for r in rows if r.get('scope') == scope and r.get('event') == event]


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('trace')
    ap.add_argument('--run-json')
    ap.add_argument('--seconds', type=float, default=600)
    ap.add_argument('--frame-ms', type=int, default=100)
    ap.add_argument('--checkpoint-every', type=float, default=60)
    ap.add_argument('--seed', type=int, default=1)
    ap.add_argument('--partial', action='store_true', help='the run is still going: evaluate what exists, never exit 0')
    ap.add_argument('--max-bytes', type=int, default=256 * 1024 * 1024)
    ap.add_argument('--summary')
    a = ap.parse_args()
    au = Audit()

    # ---- PROCESS (run.json only)
    process_done = False
    if a.run_json:
        try:
            run = json.load(open(a.run_json, 'rb'))
        except (OSError, ValueError) as error:
            run = None
            au.add('PROCESS', 'run.json readable', 'FAIL', str(error))
        if run is not None:
            status, code = run.get('status'), run.get('actual_native_exit')
            process_done = (status == 'completed') and code == 0
            if status in ('running', None) or code is None:
                au.add('PROCESS', 'native process finished', 'NOT_EVALUATED', f'run.json status={status!r} actual_native_exit={code!r}')
            else:
                au.ok('PROCESS', 'native process finished with exit 0', process_done, f'run.json status={status!r} actual_native_exit={code!r}')
    else:
        au.add('PROCESS', 'native process finished', 'NOT_EVALUATED', 'no --run-json: process completion is not inferred from the trace')

    rows, problems, size = read_trace(a.trace, a.max_bytes)
    for p in problems:
        # a partial trace is allowed to end in a line being written; anything else is a failure
        if a.partial and 'no newline' in p:
            au.add('ARTIFACT', 'trace readable', 'NOT_EVALUATED', p)
        else:
            au.add('ARTIFACT', 'trace readable', 'FAIL', p)
    if not problems:
        au.ok('ARTIFACT', 'trace readable', True, f'{len(rows)} JSON objects, {size} bytes')

    # ---- boot
    boots = [r for r in rows if r.get('event') == 'boot' and 'scope' not in r]
    if len(boots) == 1 and rows and rows[0] is boots[0]:
        b = boots[0]
        want = {'seconds': a.seconds, 'frame_ms': a.frame_ms, 'checkpoint_every': a.checkpoint_every, 'seed': a.seed}
        bad = {k: (b.get(k), v) for k, v in want.items() if b.get(k) != v}
        au.ok('ARTIFACT', 'boot configuration (seconds/frame_ms/checkpoint_every/seed)', not bad, f'{b}' if not bad else f'differs (got, wanted): {bad}')
    else:
        au.ok('ARTIFACT', 'boot configuration (seconds/frame_ms/checkpoint_every/seed)', False, f'{len(boots)} boot events; first row is boot: {bool(rows) and rows[0].get("event") == "boot"}')

    # ---- checkpoints and the end point
    checkpoints = [r for r in rows if r.get('event') == 'checkpoint' and 'scope' not in r]
    expected_cps = int(a.seconds // a.checkpoint_every) if a.checkpoint_every else 0
    ats = [r.get('at') for r in checkpoints]
    in_order = ats == [a.checkpoint_every * (i + 1) for i in range(len(ats))]
    frames = [r.get('frame') for r in checkpoints]
    au.ok('ARTIFACT', 'checkpoints at exact multiples, in order, frames increasing', in_order and all(x < y for x, y in zip(frames, frames[1:])), f'{len(checkpoints)} checkpoints, at={ats}')
    runs = [r for r in rows if r.get('event') == 'run' and 'scope' not in r]
    complete = len(runs) == 1
    if not complete:
        au.add('ARTIFACT', 'final run event (end point)', 'NOT_EVALUATED', f'{len(runs)} run events: the trace is not complete')
    else:
        r = runs[0]
        expected_frames = int(-(-a.seconds * 1000 // a.frame_ms))
        end_ok = (a.seconds <= r.get('elapsed', -1) <= a.seconds + 2 * a.frame_ms / 1000.0) and r.get('frames') in (expected_frames, expected_frames + 1) \
            and r.get('expected_frames') == expected_frames and r.get('checkpoints') == expected_cps == len(checkpoints) \
            and r.get('seconds') == a.seconds and r.get('frame_ms') == a.frame_ms
        au.ok('ARTIFACT', 'run event: end point reached, frames, checkpoints and configuration as booted', end_ok, f'{r}')
        au.ok('ARTIFACT', 'the run event is the last unscoped event and a final `run` report follows it', rows.index(r) > rows.index(checkpoints[-1]) if checkpoints else True)
        au.ok('ARTIFACT', 'final report (scope run) present', len(by(rows, 'run', 'cmb-hash')) == 1 and len(by(rows, 'run', 'cmb-integrity')) == 1)

    # ---- the report lines: hashes, integrity, damage balance, counters
    reports = {}
    for r in rows:
        s = r.get('scope')
        if s in ('boot', 'checkpoint', 'run') and str(r.get('event', '')).startswith('cmb-'):
            reports.setdefault((s, r.get('at') if s == 'checkpoint' else None), []).append(r)
    ordered_keys = sorted(reports, key=lambda k: (['boot', 'checkpoint', 'run'].index(k[0]), k[1] or 0))
    hash_lines = [(k, [x for x in reports[k] if x['event'] == 'cmb-hash']) for k in ordered_keys]
    bad_hash = []
    for k, lines in hash_lines:
        if len(lines) != 1:
            bad_hash.append((k, f'{len(lines)} cmb-hash lines'))
            continue
        for h in HASH_KEYS:
            v = lines[0].get(h)
            if not (isinstance(v, str) and HEX16.match(v)):
                bad_hash.append((k, h, v))
    au.ok('ARTIFACT', 'six hashes (damage collision blast repair timed order) in every report are 16 lowercase hex digits written as strings', not bad_hash and bool(hash_lines), f'{len(hash_lines)} reports; bad: {bad_hash[:3]}')

    bad_integrity = []
    for k in ordered_keys:
        for x in [y for y in reports[k] if y['event'] == 'cmb-integrity']:
            if not (x.get('trusted') == 1 and all(x.get(f) == 0 for f in ('depth_overflow', 'depth_unbalanced', 'identity_unknown', 'release_outside', 'log_overflow'))):
                bad_integrity.append((k, {f: x.get(f) for f in ('trusted', 'depth_overflow', 'depth_unbalanced', 'identity_unknown', 'release_outside', 'log_overflow')}))
    au.ok('ARTIFACT', 'integrity in every report: trusted, no nesting overflow, unbalanced delivery, unknown identity, outside release or log overflow', not bad_integrity and bool(ordered_keys), f'bad: {bad_integrity[:2]}')

    bad_damage = []
    for k in ordered_keys:
        for x in [y for y in reports[k] if y['event'] == 'cmb-damage']:
            if x.get('balanced') != 1 or x.get('entries') != x.get('exit_no_weapon', 0) + x.get('exit_invulnerable', 0) + x.get('exit_done', 0):
                bad_damage.append((k, x))
    au.ok('ARTIFACT', 'damage_client_server_entity entries equal exits in every report', not bad_damage and bool(ordered_keys), f'bad: {bad_damage[:1]}')

    def flat(key):
        out = {}
        for x in reports[key]:
            e = x['event']
            if e == 'cmb-kind':
                for f, v in x.items():
                    if f not in ('event', 'scope', 'at', 'type'):
                        try:
                            out[('cmb-kind', x['type'], f)] = as_int(v) if f != 'damage_sum' else float(v)
                        except ValueError:
                            out[('cmb-kind', x['type'], f)] = None
            elif e in ('cmb-warhead', 'cmb-blast-kill', 'cmb-timed', 'cmb-unobserved'):
                tag = x.get('type', x.get('code', x.get('kind')))
                for f, v in x.items():
                    if f not in ('event', 'scope', 'at', 'type', 'code', 'kind', 'note') and isinstance(v, (int, float)) and not isinstance(v, bool):
                        out[(e, tag, f)] = v
            elif e not in ('cmb-hash',):
                for f, v in x.items():
                    if f not in ('event', 'scope', 'at', 'balanced', 'trusted') and isinstance(v, (int, float)) and not isinstance(v, bool):
                        out[(e, None, f)] = v
        return out

    regress = []
    previous = None
    for k in ordered_keys:
        now = flat(k)
        if previous is not None:
            for key, v in now.items():
                p = previous.get(key)
                if v is not None and p is not None and key[2] not in ('max_candidates', 'max_blast_calls', 'depth_max', 'max_damage') and v < p:
                    regress.append((k, key, p, v))
        previous = now
    au.ok('ARTIFACT', 'run-wide counters never decrease from one report to the next', not regress, f'decreases: {regress[:2]}')

    # ---- the ordered event log
    ev = [r for r in rows if r.get('scope') == 'log' and r.get('event') == 'cmb-ev']
    seqs = [r.get('seq') for r in ev]
    au.ok('ARTIFACT', 'event log sequence is contiguous from 1', seqs == list(range(1, len(ev) + 1)), f'{len(ev)} events; first mismatch at {next((i for i, s in enumerate(seqs) if s != i + 1), None)}')
    unknown = [r for r in ev if r.get('kind') not in EVENT_KINDS]
    au.ok('ARTIFACT', 'every event is of a known kind', not unknown, f'{unknown[:1]}')
    ats_ev = [r.get('at') for r in ev]
    au.ok('ARTIFACT', 'event drain times never go backwards', all(x <= y for x, y in zip(ats_ev, ats_ev[1:])))
    # blast begin/end pairing and explosion containment
    pair_ok, open_blast, explosion_ok, blasts_since = True, False, True, 0
    for r in ev:
        k = r.get('kind')
        if k == 'blast-begin':
            pair_ok &= not open_blast
            open_blast = True
            if r.get('in_explosion') == 1:
                blasts_since += 1
        elif k == 'blast-end':
            pair_ok &= open_blast
            open_blast = False
        elif k == 'explosion':
            explosion_ok &= (r.get('blast_calls') == blasts_since)
            blasts_since = 0
    au.ok('ARTIFACT', 'blast begin/end events pair up in order', pair_ok and not (open_blast and complete), f'open at end: {open_blast}')
    au.ok('ARTIFACT', 'each explosion event reports exactly the blasts that ran inside it', explosion_ok and not (blasts_since and complete), f'blasts not yet closed by an explosion event: {blasts_since}')

    last_key = ordered_keys[-1] if ordered_keys else None
    if last_key:
        fl = flat(last_key)
        c = lambda tag, f: fl.get((tag, None, f))
        counts = {k: sum(1 for r in ev if r.get('kind') == k) for k in EVENT_KINDS}
        kinds_collisions = sum(v for (e, t, f), v in fl.items() if e == 'cmb-kind' and f == 'collisions' and v is not None)
        timed_hits = sum(v for (e, t, f), v in fl.items() if e == 'cmb-timed' and f == 'hits')
        repairs = sum(c('cmb-repair', f) or 0 for f in ('resupply', 'fixed_restore', 'other'))
        tallies = [
            ('collision events == observed deliveries == sum of per-kind collisions', counts['collision'] == c('cmb-install', 'observed_deliveries') == kinds_collisions),
            ('blast-begin events == blast calls', counts['blast-begin'] == c('cmb-blast', 'calls')),
            ('blast-end events == blast calls', counts['blast-end'] == c('cmb-blast', 'calls')),
            ('explosion events == explosion invocations', counts['explosion'] == c('cmb-explosion', 'invocations')),
            ('repair events == repair writes', counts['repair'] == repairs),
            ('timed events == timed hits', counts['timed'] == timed_hits),
            ('cmb-log events == stored == drained == events read, overflow 0', c('cmb-log', 'events') == c('cmb-log', 'stored') == c('cmb-log', 'drained') == len(ev) and c('cmb-log', 'overflow') == 0),
        ]
        for name, cond in tallies:
            if complete or not a.partial:
                au.ok('ARTIFACT', name + f' ({"final report" if last_key[0] == "run" else "last checkpoint"})', bool(cond), f'events {counts}')
            else:
                # a partial trace: the events after the last checkpoint are not drained yet, so equality is only required at the checkpoint's own drain
                au.ok('ARTIFACT', name + ' (last checkpoint of a partial trace)', bool(cond) or last_key[0] == 'checkpoint', f'events {counts}')
        # coverage
        cov = {'report': 'run' if last_key[0] == 'run' else f'checkpoint at {last_key[1]} s (PARTIAL)', 'kinds': {}, 'warheads': {}, 'kill_codes': {}, 'timed': {}, 'repair': {}, 'special': {}, 'unreached': []}
        for kind in HEALTH_KINDS:
            full = 'ENTITY_TYPE_' + kind
            row = {f: fl.get(('cmb-kind', full, f)) for f in ('collisions', 'from_weapon', 'from_other', 'direct', 'blast', 'applied', 'no_damage', 'invulnerable', 'fatal', 'destroyed', 'already_dead',
                                                                'candidates', 'in_range', 'notified', 'rocket_kills', 'max_damage')}
            # a kind with no row at all in the report had no collision and was never a blast candidate: all zero, and said so
            row = {f: (0 if v is None else v) for f, v in row.items()}
            row['row_present'] = ('cmb-kind', full, 'collisions') in fl
            cov['kinds'][kind] = row
            if not row['collisions']:
                cov['unreached'].append(f'collision on {kind}' + (' (blast candidates seen, none in range)' if row['candidates'] and not row['in_range'] else ('' if row['row_present'] else ' (never a blast candidate: no row in the report)')))
        for (e, t, f), v in fl.items():
            if e == 'cmb-warhead':
                cov['warheads'].setdefault(str(t), {})[f] = v
            elif e == 'cmb-blast-kill':
                cov['kill_codes'][KILL_CODES.get(t, str(t))] = v
            elif e == 'cmb-timed':
                cov['timed'].setdefault(TIMED_NAMES.get(t, str(t)), {})[f] = v
        for name, t in TIMED_NAMES.items():
            row = cov['timed'].get(t, {})
            if not row.get('hits'):
                cov['unreached'].append(f'timed health loss: {t} (attempts {row.get("attempts")}, draws {row.get("draws")}, hits {row.get("hits")})')
        cov['repair'] = {f: c('cmb-repair', f) for f in ('resupply', 'fixed_restore', 'other', 'zero_to_positive')}
        cov['special'] = {
            'real_capability_calls': c('cmb-capability', 'real'), 'forecast_capability_calls': c('cmb-capability', 'forecast'), 'penetrated': c('cmb-capability', 'penetrated'),
            'splash': c('cmb-capability', 'splash'), 'none': c('cmb-capability', 'none'), 'capability_draws': c('cmb-capability', 'draws'),
            'player_target': c('cmb-capability', 'player_target'), 'multiplier_moved': c('cmb-capability', 'multiplier_moved'),
            'damage_exit_no_weapon': c('cmb-damage', 'exit_no_weapon'), 'damage_exit_invulnerable': c('cmb-damage', 'exit_invulnerable'),
            'explosion_invocations': c('cmb-explosion', 'invocations'), 'explosions_with_repeated_blast': c('cmb-explosion', 'with_repeated_blast'),
            'blast_zero_radius': c('cmb-blast', 'zero_radius'), 'wrapped_collision_handlers_cumulative': c('cmb-install', 'wrapped_collision_handlers'),
            'bridge_cargo_weapon_deliveries_counted_only': sum(v for (e, t, f), v in fl.items() if e == 'cmb-unobserved' and f == 'deliveries'),
        }
        for label, key in (('a player-controlled target', 'player_target'), ('a weapon whose multiplayer multiplier moved the damage', 'multiplier_moved'),
                           ('a collision with no weapon (the no-weapon exit)', 'damage_exit_no_weapon'), ('an invulnerable target', 'damage_exit_invulnerable')):
            if not cov['special'].get(key):
                cov['unreached'].append(label)
        if not sum(v for (e, t, f), v in fl.items() if e == 'cmb-kind' and f == 'rocket_kills' and v):
            cov['unreached'].append('a rocket destroyed by a blast')
        if not cov['repair'].get('fixed_restore'):
            cov['unreached'].append('a fixed-structure restore (D4 coupling)')
    else:
        cov = None

    # ---- verdict
    evaluated_incomplete = (not complete) or a.partial or (a.run_json is None) or (not process_done)
    status = 'FAIL' if au.failed() else ('INCOMPLETE' if evaluated_incomplete else 'PASS')
    summary = {'trace': a.trace, 'bytes': size, 'status': status, 'process_done': process_done, 'artifact_complete': complete, 'checks': au.results, 'coverage': cov}
    for r in au.results:
        print(f'{r["status"]:<13} [{r["section"]}] {r["name"]}' + (f' :: {r["detail"]}' if r['status'] != 'PASS' and r['detail'] else ''))
    if cov:
        print(f'\nCOVERAGE (measured, {cov["report"]}):')
        for kind, row in cov['kinds'].items():
            print(f'  {kind:<15} collisions={row["collisions"]} direct={row["direct"]} blast={row["blast"]} applied={row["applied"]} fatal={row["fatal"]} candidates={row["candidates"]} in_range={row["in_range"]} notified={row["notified"]}')
        print('  warheads:', json.dumps(cov['warheads'], sort_keys=True))
        print('  kill codes in blasts:', json.dumps(cov['kill_codes'], sort_keys=True))
        print('  timed:', json.dumps(cov['timed'], sort_keys=True))
        print('  repair:', json.dumps(cov['repair'], sort_keys=True))
        print('  special:', json.dumps(cov['special'], sort_keys=True))
        print('  UNREACHED:', '; '.join(cov['unreached']) or 'none')
    print(f'\nSTATUS={status}')
    if a.summary:
        with open(a.summary, 'w', encoding='utf-8') as f:
            json.dump(summary, f, indent=1, sort_keys=True)
    sys.exit(1 if status == 'FAIL' else (2 if status == 'INCOMPLETE' else 0))


if __name__ == '__main__':
    main()
