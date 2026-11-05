#!/usr/bin/env python3
# TEST-ONLY (EEDCS-45): compares two lua/weapon-characterization.lua traces (JSON Lines) event by event. Used to show that a candidate (or a rerun of the same
# build) left every hash, counter, value range, per-weapon and per-table row, sample and ORDERED log event unchanged.
#
# Usage: tools/wpn-trace-compare.py <a.jsonl> <b.jsonl> [--scope boot|log|run|checkpoint ...] [--skip-samples]
#            [--expect-scenario lebanon_retail] [--expect-seconds 600] [--expect-frame-ms 100] [--expect-seed 1] [--expect-checkpoint 60]
#            [--min counter=N ...]
#   --scope selects which report scopes are compared (default: all four). The scope-less marker events (the boot configuration, each checkpoint marker and the
#   run marker) are always compared as well. Comparison is strict: same events, same order, every field equal; a differing counter is a difference, never ignored.
#
# Fails closed. Exit 0 only when both files are complete traces of an ACTIVE, CORRECTLY CONFIGURED run, at least one report event was selected, and every
# selected event is equal. Exit 1: a difference. Exit 2: invalid input, with the reason on stderr:
#   * a file that is missing, unreadable or not JSON Lines, or empty;
#   * not a COMPLETE trace (it must end with the terminal `wpn-log` event of scope `run`, which the driver writes last, so a truncated run is refused whatever
#     scopes are selected and also with --skip-samples);
#   * a trace whose boot configuration (scenario, seconds, frame length, seed, checkpoint interval) is not the expected one, or whose run marker does not show a
#     run that ended at the FIRST frame whose clock reached the requested endpoint, with the expected frame count and checkpoint count;
#   * a trace without the run-wide hashes, counters or ranges;
#   * a trace with NO activity: table_supplies, birth_calls, move_calls and terminal_calls must each exceed their minimum (default 1; --min overrides, and may
#     lower a threshold only explicitly, per case): two empty traces would be equal and prove nothing;
#   * an INCOMPLETE ordered log: the log must have no overflow, every stored event drained, the events in the trace must be exactly the stored ones with contiguous
#     sequence numbers, and the fixture bookkeeping counters (unregistered weapons, track overflow) must be zero;
#   * a selection that matches no report event.
import argparse
import json
import re
import sys

TERMINAL = ('wpn-log', 'run')
HASH = re.compile(r'^[0-9a-f]{16}$')
HASH_FIELDS = ('table', 'launch', 'flight', 'terminal', 'decision')
BOOKKEEPING_ZERO = ('birth_track_overflow', 'move_unregistered', 'terminal_unregistered', 'destroy_unregistered')
DEFAULT_MIN = {'table_supplies': 1, 'birth_calls': 1, 'move_calls': 1, 'terminal_calls': 1}


class Invalid(Exception):
    pass


def load(path, expect, minimums):
    events = []
    try:
        with open(path, encoding='utf-8') as handle:
            for number, line in enumerate(handle, 1):
                if not line.strip():
                    continue
                try:
                    event = json.loads(line)
                except ValueError as error:
                    raise Invalid(f'{path}: line {number} is not JSON ({error})')
                if not isinstance(event, dict):
                    raise Invalid(f'{path}: line {number} is not a JSON object')
                events.append((number, event))
    except OSError as error:
        raise Invalid(f'{path}: cannot read ({error.strerror or error})')
    if not events:
        raise Invalid(f'{path}: empty, not a trace')
    number, last = events[-1]
    if (last.get('event'), last.get('scope')) != TERMINAL:
        raise Invalid(f'{path}: incomplete, the last event (line {number}) is not the terminal run report '
                      f'(event {TERMINAL[0]}, scope {TERMINAL[1]}); a truncated run does not qualify')
    boots = [e for _, e in events if e.get('event') == 'boot']
    runs = [e for _, e in events if e.get('event') == 'run']
    checkpoints = [e for _, e in events if e.get('event') == 'checkpoint']
    if len(boots) != 1 or len(runs) != 1:
        raise Invalid(f'{path}: expected exactly one boot and one run marker, found {len(boots)} and {len(runs)}')
    boot, run_marker = boots[0], runs[0]
    for key, want in expect.items():
        if boot.get(key) != want:
            raise Invalid(f'{path}: wrongly configured: boot {key} is {boot.get(key)!r}, expected {want!r}')
    seconds, frame_ms = expect['seconds'], expect['frame_ms']
    elapsed = run_marker.get('elapsed')
    if not isinstance(elapsed, (int, float)) or elapsed < seconds or elapsed >= seconds + frame_ms / 1000.0 * 1.0001:
        raise Invalid(f'{path}: the run marker clock {elapsed!r} is not the first clock reaching {seconds} s with {frame_ms} ms frames')
    wanted_frames = int(-(-seconds * 1000 // frame_ms))
    frames = run_marker.get('frames')
    if not isinstance(frames, int) or abs(frames - wanted_frames) > 1 or run_marker.get('expected_frames') != wanted_frames:
        raise Invalid(f'{path}: the run marker frame count {frames!r} (expected_frames {run_marker.get("expected_frames")!r}) is not about {wanted_frames}')
    interval = expect['checkpoint_every']
    want_checkpoints = int(elapsed // interval) if interval else 0
    if len(checkpoints) != want_checkpoints or run_marker.get('checkpoints') != want_checkpoints:
        raise Invalid(f'{path}: {len(checkpoints)} checkpoint markers and run marker checkpoints={run_marker.get("checkpoints")!r}, expected {want_checkpoints}')
    run = [e for _, e in events if e.get('scope') == 'run']
    hashes = [e for e in run if e.get('event') == 'wpn-hash']
    if len(hashes) != 1 or not all(HASH.match(str(hashes[0].get(k, ''))) for k in HASH_FIELDS):
        raise Invalid(f'{path}: the run report has no well-formed wpn-hash event')
    counters = {e.get('name'): e.get('value') for e in run if e.get('event') == 'wpn-counter'}
    if not counters or not any(e.get('event') == 'wpn-range' for e in run):
        raise Invalid(f'{path}: the run report has no wpn-counter / wpn-range events')
    for name, floor in minimums.items():
        value = counters.get(name)
        if not isinstance(value, int):
            raise Invalid(f'{path}: the run report lacks the counter {name}')
        if value < floor:
            raise Invalid(f'{path}: no {name} activity recorded ({value} < {floor}); an inactive trace is not a characterisation')
    for name in BOOKKEEPING_ZERO:
        if counters.get(name) != 0:
            raise Invalid(f'{path}: fixture bookkeeping counter {name} is {counters.get(name)!r}, not 0')
    logs = [e for e in run if e.get('event') == 'wpn-log']
    tables = [e for e in run if e.get('event') == 'wpn-tables']
    if len(logs) != 1 or len(tables) != 1:
        raise Invalid(f'{path}: the run report has {len(logs)} wpn-log and {len(tables)} wpn-tables events')
    log, tbl = logs[0], tables[0]
    if log.get('overflow') != 0 or tbl.get('overflow') != 0:
        raise Invalid(f'{path}: overflow (log {log.get("overflow")!r}, tables {tbl.get("overflow")!r}); the trace is incomplete')
    stored = [e for _, e in events if e.get('event') == 'wpn-ev' and e.get('scope') == 'log']
    if not (log.get('events') == log.get('stored') == log.get('drained') == len(stored)):
        raise Invalid(f'{path}: ordered log incomplete: report events={log.get("events")!r} stored={log.get("stored")!r} drained={log.get("drained")!r}, '
                      f'trace carries {len(stored)} wpn-ev lines')
    for index, e in enumerate(stored, 1):
        if e.get('seq') != index:
            raise Invalid(f'{path}: ordered log sequence broken at event {index} (seq {e.get("seq")!r})')
    return events


def select(events, scopes, skip_samples):
    chosen = []
    for _, e in events:
        scope = e.get('scope')
        if scope is None or scope in scopes:
            if skip_samples and e.get('event') == 'wpn-sample':
                continue
            chosen.append(e)
    return chosen


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('a')
    parser.add_argument('b')
    parser.add_argument('--scope', action='append', choices=['boot', 'log', 'run', 'checkpoint'])
    parser.add_argument('--skip-samples', action='store_true')
    parser.add_argument('--expect-scenario', default='lebanon_retail')
    parser.add_argument('--expect-seconds', type=float, default=600)
    parser.add_argument('--expect-frame-ms', type=float, default=100)
    parser.add_argument('--expect-seed', type=int, default=1)
    parser.add_argument('--expect-checkpoint', type=float, default=60)
    parser.add_argument('--min', action='append', default=[], metavar='COUNTER=N')
    args = parser.parse_args()
    scopes = set(args.scope or ['boot', 'log', 'run', 'checkpoint'])
    expect = {'scenario': args.expect_scenario, 'seconds': args.expect_seconds, 'frame_ms': args.expect_frame_ms, 'seed': args.expect_seed,
              'checkpoint_every': args.expect_checkpoint}
    minimums = dict(DEFAULT_MIN)
    for item in args.min:
        name, _, value = item.partition('=')
        minimums[name] = int(value)
    try:
        left = select(load(args.a, expect, minimums), scopes, args.skip_samples)
        right = select(load(args.b, expect, minimums), scopes, args.skip_samples)
        reports = lambda events: [e for e in events if e.get('scope') in scopes]
        if not reports(left) or not reports(right):
            raise Invalid(f'scopes {sorted(scopes)} select no report events ({len(reports(left))} in a, {len(reports(right))} in b); nothing was compared')
    except Invalid as error:
        print(f'invalid input: {error}', file=sys.stderr)
        return 2
    differences = 0
    for index in range(max(len(left), len(right))):
        l = left[index] if index < len(left) else None
        r = right[index] if index < len(right) else None
        if l != r:
            differences += 1
            if differences <= 20:
                print(f'difference at event {index}:')
                print('  a:', json.dumps(l, sort_keys=True) if l is not None else None)
                print('  b:', json.dumps(r, sort_keys=True) if r is not None else None)
    print(f'{len(left)} events in a, {len(right)} in b, scopes {sorted(scopes)}: {differences} differences')
    return 1 if differences else 0


if __name__ == '__main__':
    sys.exit(main())
