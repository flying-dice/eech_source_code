#!/usr/bin/env python3
"""task-only (EEDCS-44 extraction): compares a controlled person-case trace of the extraction candidate with the original (74ae6a96d) trace of the same case.
Revision 2 (replaces the checker with SHA-256 959144d8...; that checker and its results are kept in superseded-person-checker-959144d8/).

usage: compare-person.py <candidate trace> <original trace> <result.json>
Exit 0 only when ALL of the following hold; any other outcome (including any unreadable, empty, truncated, prefix, failed or unexpectedly changed input) exits 1:

1. Both traces are HEALTHY, each on its own: a non-empty JSON Lines file whose every line is a JSON object and whose last line is complete; the first event is `boot` with
   case target|control, frames 20, frame_ms 100, scenario lebanon_retail, warmup 20; exactly one terminal `verdict` as the LAST event, with failed 0, frames 20 and the case's
   host_steps (target 60, control 40); the case's exact number of `check` events (target 11, control 3), every one ok true, none missing; exactly one `aim-hash` (two 16-hex
   hashes) and one terminal `aim-samples`; 20 `frame` events numbered 1..20 in order; the case's `state` events (target before/after_setup/after_release, control before).
   Both traces are of the SAME case.
2. The two traces have the same event types at every index (so nothing is missing, extra or reordered).
3. Every event is EXACTLY equal at every index except the following whitelist, which is applied and nothing else:
   a. the 20 `frame` events: the original's driver keys place_in/place_out become face_in/face_out. Target: face_out == place_out exactly; the original's place_in == place_out;
      face_in of frame 1 == the `before` state attitude (printed at 9 significant digits: agreement to 1e-8), and face_in of frame n == face_out of frame n-1 (exactly) (the exact input/output of each of the 20 facing steps); every other
      field equal. Control: the four keys are all the string "none" (no facing step); every other field equal.
   b. exactly three `check` events of the target case, at the original's positions 7, 8 and 11 of the 11 checks, and ONLY these strings (name, detail) -> (name, detail):
        ("the host was asked to place the person (D1 campaign_movement_place) in most frames", "20 places in 20 frames")
          -> ("the host was asked to turn the person (person_face) in most frames", "20 person_face steps in 20 frames")
        ("the host adopted exactly the pose the campaign sent (place: attitude in == out)", "")
          -> ("the host answered the facing of its own record toward the target (attitude and heading == the independent bearing)", "")
        ("every non-place step arrived with the attitude the host last answered (no write the host did not make); at least one occurred after the release", "40 non-place steps compared")
          -> ("every non-person_face step arrived with the attitude the host last answered (no write the host did not make); at least one occurred after the release", "40 non-person_face steps compared")
      The control case has NO whitelisted check: its 3 checks must be exactly equal. Any other rename, detail change or reorder of a check fails, even when both are ok.
   c. the `verdict`: step_modes ["person_begin=20","person_end=20","place=20"] -> ["person_begin=20","person_end=20","person_face=20"] (target); equal (control); every other field equal.
"""
import hashlib
import json
import re
import sys

CASES = {
    'target': {'checks': 11, 'host_steps': 60, 'states': ['before', 'after_setup', 'after_release'],
               'modes_original': ['person_begin=20', 'person_end=20', 'place=20'], 'modes_candidate': ['person_begin=20', 'person_end=20', 'person_face=20']},
    'control': {'checks': 3, 'host_steps': 40, 'states': ['before'],
                'modes_original': ['person_begin=20', 'person_end=20'], 'modes_candidate': ['person_begin=20', 'person_end=20']},
}
CHECK_MAP = {  # original position (1-based among the target case's checks) -> ((name, detail) original, (name, detail) candidate)
    7: (("the host was asked to place the person (D1 campaign_movement_place) in most frames", "20 places in 20 frames"),
        ("the host was asked to turn the person (person_face) in most frames", "20 person_face steps in 20 frames")),
    8: (("the host adopted exactly the pose the campaign sent (place: attitude in == out)", ""),
        ("the host answered the facing of its own record toward the target (attitude and heading == the independent bearing)", "")),
    11: (("every non-place step arrived with the attitude the host last answered (no write the host did not make); at least one occurred after the release",
          "40 non-place steps compared"),
         ("every non-person_face step arrived with the attitude the host last answered (no write the host did not make); at least one occurred after the release",
          "40 non-person_face steps compared")),
}
HEX16 = re.compile(r'^[0-9a-f]{16}$')


class Bad(Exception):
    pass


def load(path):
    try:
        data = open(path, 'rb').read()
    except OSError as error:
        raise Bad(f'{path}: unreadable ({error})')
    if not data:
        raise Bad(f'{path}: empty file')
    if not data.endswith(b'\n'):
        raise Bad(f'{path}: the last line is incomplete (no line end)')
    events = []
    for number, line in enumerate(data.decode('utf-8', errors='strict').splitlines(), 1):
        if not line.strip():
            continue
        try:
            event = json.loads(line)
        except ValueError as error:
            raise Bad(f'{path}: line {number} is not JSON ({error})')
        if not isinstance(event, dict) or not isinstance(event.get('event'), str):
            raise Bad(f'{path}: line {number} is not an event object')
        events.append(event)
    if not events:
        raise Bad(f'{path}: no events')
    return data, events


def healthy(label, events):
    """one trace on its own; returns its case"""
    first, last = events[0], events[-1]
    if first['event'] != 'boot':
        raise Bad(f'{label}: the first event is {first["event"]}, not boot')
    case = first.get('case')
    if case not in CASES:
        raise Bad(f'{label}: boot case {case!r} is not target or control')
    spec = CASES[case]
    for key, value in (('frames', 20), ('frame_ms', 100), ('scenario', 'lebanon_retail'), ('warmup', 20)):
        if first.get(key) != value:
            raise Bad(f'{label}: boot {key} is {first.get(key)!r}, not {value!r}')
    if last['event'] != 'verdict' or sum(e['event'] == 'verdict' for e in events) != 1:
        raise Bad(f'{label}: the trace does not end in exactly one verdict (truncated or duplicated)')
    if last.get('case') != case or last.get('failed') != 0 or last.get('frames') != 20 or last.get('host_steps') != spec['host_steps']:
        raise Bad(f'{label}: the verdict is not a successful {case} verdict: {json.dumps(last)}')
    checks = [e for e in events if e['event'] == 'check']
    if len(checks) != spec['checks']:
        raise Bad(f'{label}: {len(checks)} checks, expected {spec["checks"]}')
    for check in checks:
        if check.get('ok') is not True:
            raise Bad(f'{label}: check not ok: {check.get("name")!r}')
        if not isinstance(check.get('name'), str) or not isinstance(check.get('detail'), str):
            raise Bad(f'{label}: a check without a name or detail')
    hashes = [e for e in events if e['event'] == 'aim-hash']
    if len(hashes) != 1 or not HEX16.match(str(hashes[0].get('physical'))) or not HEX16.match(str(hashes[0].get('decision'))):
        raise Bad(f'{label}: not exactly one well-formed aim-hash')
    if sum(e['event'] == 'aim-samples' for e in events) != 1:
        raise Bad(f'{label}: not exactly one terminal aim-samples report')
    frames = [e for e in events if e['event'] == 'frame']
    if [f.get('frame') for f in frames] != list(range(1, 21)):
        raise Bad(f'{label}: the frame events are not 1..20 in order')
    states = [e.get('label') for e in events if e['event'] == 'state']
    if states != spec['states']:
        raise Bad(f'{label}: state labels {states}, expected {spec["states"]}')
    released = [e for e in events if e['event'] == 'released']
    if case == 'target' and (len(released) != 1 or released[0].get('host_steps_after_release') != 40):
        raise Bad(f'{label}: the target case lacks its single release with 40 later host steps')
    if case == 'control' and released:
        raise Bad(f'{label}: the control case released a link')
    for name in ('warmed_up', 'candidates', 'chosen'):
        if sum(e['event'] == name for e in events) != 1:
            raise Bad(f'{label}: not exactly one {name} event')
    return case


def attitude_equal(value, expected, exact):
    """the input of a facing step against the attitude before it. Exact (the previous step's output, full precision) except for frame 1, whose reference is the `before`
    state, which the driver prints at 9 significant digits: there the nine numbers must agree to 1e-8."""
    if not (isinstance(value, list) and isinstance(expected, list) and len(value) == 9 and len(expected) == 9):
        return False
    if exact:
        return value == expected
    return all(isinstance(a, (int, float)) and isinstance(b, (int, float)) and abs(a - b) <= 1e-8 for a, b in zip(value, expected))


def compare(cand, orig, case):
    problems, mapped = [], {'frames_mapped': 0, 'checks_mapped': 0, 'verdict_mapped': 0, 'events_exactly_equal': 0}
    spec = CASES[case]
    if [e['event'] for e in cand] != [e['event'] for e in orig]:
        raise Bad('the two traces do not have the same event types at every index')
    before_attitude = next(e for e in cand if e['event'] == 'state' and e.get('label') == 'before').get('attitude')
    previous_out = before_attitude
    check_position = 0
    first_frame_index = next(i for i, e in enumerate(cand, 1) if e['event'] == 'frame')
    for index, (x, y) in enumerate(zip(cand, orig), 1):
        kind = x['event']
        if kind == 'frame':
            if case == 'target':
                ok = (set(x) == {'alive', 'event', 'face_in', 'face_out', 'frame', 'has_target', 'heading', 'host_steps'}
                      and set(y) == {'alive', 'event', 'frame', 'has_target', 'heading', 'host_steps', 'place_in', 'place_out'}
                      and y['place_in'] == y['place_out'] and isinstance(y['place_out'], list) and len(y['place_out']) == 9
                      and x['face_out'] == y['place_out'] and attitude_equal(x['face_in'], previous_out, exact=(index != first_frame_index)))
            else:
                ok = (set(x) == {'alive', 'event', 'face_in', 'face_out', 'frame', 'has_target', 'heading', 'host_steps'}
                      and set(y) == {'alive', 'event', 'frame', 'has_target', 'heading', 'host_steps', 'place_in', 'place_out'}
                      and x['face_in'] == 'none' and x['face_out'] == 'none' and y['place_in'] == 'none' and y['place_out'] == 'none')
            rest_x = {k: v for k, v in x.items() if k not in ('face_in', 'face_out')}
            rest_y = {k: v for k, v in y.items() if k not in ('place_in', 'place_out')}
            if not ok or rest_x != rest_y:
                problems.append(f'index {index}: frame {x.get("frame")} breaks the face/place input-output mapping')
            else:
                mapped['frames_mapped'] += 1
            if case == 'target':
                previous_out = x.get('face_out')
        elif kind == 'check':
            check_position += 1
            pair = ((y.get('name'), y.get('detail')), (x.get('name'), x.get('detail')))
            rest_x = {k: v for k, v in x.items() if k not in ('name', 'detail')}
            rest_y = {k: v for k, v in y.items() if k not in ('name', 'detail')}
            if case == 'target' and check_position in CHECK_MAP:
                if pair != CHECK_MAP[check_position] or rest_x != rest_y:
                    problems.append(f'index {index}: check {check_position} is not the whitelisted mapping: {pair}')
                else:
                    mapped['checks_mapped'] += 1
            elif x != y:
                problems.append(f'index {index}: check {check_position} differs outside the whitelist: {pair}')
            else:
                mapped['events_exactly_equal'] += 1
        elif kind == 'verdict':
            if (x.get('step_modes') != spec['modes_candidate'] or y.get('step_modes') != spec['modes_original']
                    or {k: v for k, v in x.items() if k != 'step_modes'} != {k: v for k, v in y.items() if k != 'step_modes'}):
                problems.append(f'index {index}: the verdict differs beyond the step-mode name')
            else:
                mapped['verdict_mapped'] += 1
        elif x != y:
            problems.append(f'index {index}: {kind} differs (no mapping applies)')
        else:
            mapped['events_exactly_equal'] += 1
    expected_checks = len(CHECK_MAP) if case == 'target' else 0
    if mapped['checks_mapped'] != expected_checks:
        problems.append(f'{mapped["checks_mapped"]} checks mapped, expected {expected_checks}')
    if mapped['frames_mapped'] != 20 or mapped['verdict_mapped'] != 1:
        problems.append(f'{mapped["frames_mapped"]} frames and {mapped["verdict_mapped"]} verdict mapped, expected 20 and 1')
    return problems, mapped


def main():
    cand_path, orig_path, result = sys.argv[1:4]
    record = {'checker': 'compare-person.py revision 2', 'candidate': cand_path, 'original': orig_path, 'equal_under_the_mapping': False, 'problems': []}
    try:
        cand_raw, cand = load(cand_path)
        orig_raw, orig = load(orig_path)
        record['candidate_sha256'] = hashlib.sha256(cand_raw).hexdigest()
        record['original_sha256'] = hashlib.sha256(orig_raw).hexdigest()
        record['events'] = [len(cand), len(orig)]
        case_c, case_o = healthy('candidate', cand), healthy('original', orig)
        if case_c != case_o:
            raise Bad(f'candidate is the {case_c} case, original the {case_o} case')
        record['case'] = case_c
        problems, mapped = compare(cand, orig, case_c)
        record['mapped'] = mapped
        record['problems'] = problems
        record['equal_under_the_mapping'] = not problems
    except Bad as error:
        record['problems'].append(str(error))
    except Exception as error:  # fail closed on anything unexpected
        record['problems'].append(f'unexpected {type(error).__name__}: {error}')
    json.dump(record, open(result, 'w', encoding='utf-8', newline='\n'), indent=1)
    print(json.dumps({k: record.get(k) for k in ('case', 'events', 'mapped', 'problems', 'equal_under_the_mapping')}))
    sys.exit(0 if record['equal_under_the_mapping'] else 1)


main()
