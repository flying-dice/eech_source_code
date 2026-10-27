#!/usr/bin/env python3
# TEST-ONLY (EEDCS-45): verifies a lua/weapon-case.lua trace (the controlled physical-launch witness). Fails closed: exit 0 only when the trace is complete, correctly
# configured, its ordered log is complete, and EXACTLY the expected checks were recorded, each once and each passing. A check that is renamed, missing, duplicated or failing,
# an extra check, an incomplete or inactive trace, a log with an overflow or a gap, or a summary that disagrees with the check events exits 2 with the reason on stderr.
#
# Usage: tools/wpn-case-verify.py <trace.jsonl>
import json
import re
import sys

EXPECTED = [
    'rejects_unsupported_launcher', 'rejects_unsupported_target', 'queries_reject_unsupported_subject', 'rejection_leaves_state_unchanged',
    'empty_target_establishes_no_link', 'empty_target_restores_previous_link', 'empty_target_birth_has_no_target',
    'no_ammo_outcome', 'no_ammo_changes_nothing', 'targeted_launches_created', 'case_weapons_reach_terminal',
]
HASH = re.compile(r'^[0-9a-f]{16}$')


class Invalid(Exception):
    pass


def main(path):
    events = []
    try:
        with open(path, encoding='utf-8') as handle:
            for number, line in enumerate(handle, 1):
                if line.strip():
                    try:
                        events.append(json.loads(line))
                    except ValueError as error:
                        raise Invalid(f'line {number} is not JSON ({error})')
    except OSError as error:
        raise Invalid(f'cannot read {path} ({error.strerror or error})')
    if not events:
        raise Invalid('empty trace')
    last = events[-1]
    if last.get('event') != 'summary':
        raise Invalid('incomplete: the last event is not the summary')
    boots = [e for e in events if e.get('event') == 'boot']
    runs = [e for e in events if e.get('event') == 'run']
    if len(boots) != 1 or len(runs) != 1:
        raise Invalid(f'expected one boot and one run marker, found {len(boots)} and {len(runs)}')
    boot = boots[0]
    for key, want in (('scenario', 'lebanon_retail'), ('case', 'controlled_launch'), ('seed', 1), ('frame_ms', 100)):
        if boot.get(key) != want:
            raise Invalid(f'wrongly configured: boot {key} is {boot.get(key)!r}, expected {want!r}')
    names = [e.get('name') for e in events if e.get('event') == 'check']
    if names != EXPECTED:
        raise Invalid(f'the recorded checks {names} are not exactly the expected {EXPECTED} in order')
    failing = [e.get('name') for e in events if e.get('event') == 'check' and e.get('ok') is not True]
    if failing or last.get('failed') != 0 or last.get('checks') != len(EXPECTED):
        raise Invalid(f'failing checks {failing}; summary {last}')
    run = [e for e in events if e.get('scope') == 'run']
    hashes = [e for e in run if e.get('event') == 'wpn-hash']
    if len(hashes) != 1 or not all(HASH.match(str(hashes[0].get(k, ''))) for k in ('table', 'launch', 'flight', 'terminal', 'decision')):
        raise Invalid('the run report has no well-formed wpn-hash event')
    counters = {e.get('name'): e.get('value') for e in run if e.get('event') == 'wpn-counter'}
    for name in ('table_supplies', 'birth_calls', 'move_calls', 'terminal_calls', 'launch_no_ammo', 'launch_created'):
        if not isinstance(counters.get(name), int) or counters[name] < 1:
            raise Invalid(f'no {name} activity ({counters.get(name)!r})')
    for name in ('birth_track_overflow', 'move_unregistered', 'terminal_unregistered', 'destroy_unregistered'):
        if counters.get(name) != 0:
            raise Invalid(f'fixture bookkeeping counter {name} is {counters.get(name)!r}')
    logs = [e for e in run if e.get('event') == 'wpn-log']
    if len(logs) != 1 or logs[0].get('overflow') != 0:
        raise Invalid(f'log report {logs}')
    stored = [e for e in events if e.get('event') == 'wpn-ev' and e.get('scope') == 'log']
    if not (logs[0].get('events') == logs[0].get('stored') == logs[0].get('drained') == len(stored)):
        raise Invalid(f'ordered log incomplete: {logs[0]} against {len(stored)} wpn-ev lines')
    for index, e in enumerate(stored, 1):
        if e.get('seq') != index:
            raise Invalid(f'ordered log sequence broken at {index}')
    made = [e for e in events if e.get('event') == 'targeted_launches']
    if len(made) != 1 or made[0].get('created', 0) < 3:
        raise Invalid(f'targeted launches {made}')
    return 0


if __name__ == '__main__':
    try:
        sys.exit(main(sys.argv[1]))
    except Invalid as error:
        print(f'invalid input: {error}', file=sys.stderr)
        sys.exit(2)
