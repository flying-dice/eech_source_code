#!/usr/bin/env python3
"""task-only (EEDCS-44 extraction): compares a controlled person-case trace of the extraction candidate with the original (74ae6a96d) trace of the same case.

usage: compare-person.py <candidate trace> <original trace> <result.json>
The extraction changed the test entry point and nothing else of the case: the original asked the host to PLACE the pose the campaign had turned
(driver keys place_in/place_out, step mode `place`); the candidate asks the host to turn the person (`person_face`, keys face_in/face_out). So the comparison is exact
on every event and field EXCEPT these named ones, and for them it maps: face_out == place_out on every frame, step_modes `place=N` <-> `person_face=N`, and the
check lines that name the entry point (their ok flags must still be true). Everything else (boot, selection, states, headings, hook counters, hashes, samples)
must be equal. Exit 0 only when the whole mapping holds.
"""
import json
import sys


def load(path):
    return [json.loads(line) for line in open(path, encoding='utf-8') if line.strip()]


def main():
    cand, orig, result = load(sys.argv[1]), load(sys.argv[2]), sys.argv[3]
    problems, mapped = [], {'frames_face_out_equals_place_out': 0, 'frames_face_in_present': 0, 'check_lines_renamed': 0}
    if len(cand) != len(orig):
        problems.append(f'event count {len(cand)} vs {len(orig)}')
    for i, (x, y) in enumerate(zip(cand, orig), 1):
        if x == y:
            continue
        if x.get('event') == 'frame' and y.get('event') == 'frame':
            ok = x.get('face_out') == y.get('place_out')
            mapped['frames_face_out_equals_place_out'] += 1 if ok else 0
            mapped['frames_face_in_present'] += 1 if 'face_in' in x else 0
            rest_x = {k: v for k, v in x.items() if k not in ('face_in', 'face_out')}
            rest_y = {k: v for k, v in y.items() if k not in ('place_in', 'place_out')}
            if not ok or rest_x != rest_y:
                problems.append(f'line {i}: frame differs beyond the entry-point keys')
        elif x.get('event') == 'check' and y.get('event') == 'check':
            mapped['check_lines_renamed'] += 1
            if not (x.get('ok') is True and y.get('ok') is True):
                problems.append(f'line {i}: a check is not ok: {x.get("name")}')
        elif x.get('event') == 'verdict' and y.get('event') == 'verdict':
            mx = sorted(m.replace('person_face', 'place') for m in x.get('step_modes', []))
            my = sorted(y.get('step_modes', []))
            if mx != my or {k: v for k, v in x.items() if k != 'step_modes'} != {k: v for k, v in y.items() if k != 'step_modes'}:
                problems.append(f'line {i}: verdict differs beyond the step mode name')
        else:
            problems.append(f'line {i}: {x.get("event")} differs')
    record = {'candidate': sys.argv[1], 'original': sys.argv[2], 'events': [len(cand), len(orig)], 'mapped': mapped, 'problems': problems, 'equal_under_the_mapping': not problems}
    json.dump(record, open(result, 'w', encoding='utf-8'), indent=1)
    print(json.dumps(record))
    sys.exit(0 if not problems else 1)


main()
