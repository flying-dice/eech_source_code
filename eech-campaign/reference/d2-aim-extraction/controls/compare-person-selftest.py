#!/usr/bin/env python3
"""HELPER self-test of compare-person.py (revision 2). This runs NO native campaign and no game: it derives malformed, truncated, renamed and failed copies of the
retained person-case traces (written under --work, the originals are only read and hashed before and after) and records the ACTUAL exit status of the checker as a
subprocess for each. The genuine pairs must exit 0; every derived case must exit 1.

usage: compare-person-selftest.py --candidate-target T --candidate-control C --original-target OT --original-control OC --work DIR --out result.json
"""
import argparse
import copy
import hashlib
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
CHECKER = os.path.join(HERE, 'compare-person.py')


def sha(path):
    return hashlib.sha256(open(path, 'rb').read()).hexdigest()


def read(path):
    return [json.loads(l) for l in open(path, encoding='utf-8') if l.strip()]


def write(path, events, final_newline=True):
    text = '\n'.join(json.dumps(e, separators=(',', ':')) for e in events)
    open(path, 'w', encoding='utf-8', newline='\n').write(text + ('\n' if final_newline else ''))


def run(case, expected, cand, orig, work):
    result = os.path.join(work, f'{case}.result.json')
    done = subprocess.run([sys.executable, CHECKER, cand, orig, result], capture_output=True, text=True, encoding='utf-8')
    problems = []
    try:
        problems = json.load(open(result, encoding='utf-8')).get('problems', [])
    except Exception:
        pass
    return {'case': case, 'expected_exit': expected, 'actual_exit': done.returncode, 'ok': done.returncode == expected, 'first_problem': (problems or [''])[0][:200]}


def main():
    ap = argparse.ArgumentParser()
    for name in ('candidate_target', 'candidate_control', 'original_target', 'original_control', 'work', 'out'):
        ap.add_argument('--' + name.replace('_', '-'), required=True)
    a = ap.parse_args()
    os.makedirs(a.work, exist_ok=True)
    inputs = [a.candidate_target, a.candidate_control, a.original_target, a.original_control]
    before = {p: sha(p) for p in inputs}
    ct, cc, ot, oc = (read(p) for p in inputs)
    rows = []
    w = a.work

    def case(name, expected, cand_events, orig_path, cand_newline=True, orig_events=None):
        cand = os.path.join(w, name + '.candidate.jsonl')
        write(cand, cand_events, cand_newline)
        orig = orig_path
        if orig_events is not None:
            orig = os.path.join(w, name + '.original.jsonl')
            write(orig, orig_events)
        rows.append(run(name, expected, cand, orig, w))

    # genuine pairs (positive controls of the checker itself)
    rows.append(run('genuine-target', 0, a.candidate_target, a.original_target, w))
    rows.append(run('genuine-control', 0, a.candidate_control, a.original_control, w))
    # empty and prefix traces, on either side, and equal-but-empty
    case('empty-candidate', 1, [], a.original_target)
    open(os.path.join(w, 'empty-both.jsonl'), 'w').close()
    rows.append(run('empty-both-equal', 1, os.path.join(w, 'empty-both.jsonl'), os.path.join(w, 'empty-both.jsonl'), w))
    case('prefix-candidate-half', 1, ct[:100], a.original_target)
    case('prefix-both-same-prefix', 1, ot[:100], None, orig_events=ot[:100])
    case('prefix-original-half', 1, ct, None, orig_events=ot[:100])
    case('missing-final-verdict', 1, ct[:-1], a.original_target)
    case('truncated-last-line', 1, ct, a.original_target, cand_newline=False)
    # a truncated line inside the file and a non-JSON line
    cand = os.path.join(w, 'malformed-line.candidate.jsonl')
    lines = open(a.candidate_target, encoding='utf-8').read().splitlines()
    lines[50] = lines[50][: len(lines[50]) // 2]
    open(cand, 'w', encoding='utf-8', newline='\n').write('\n'.join(lines) + '\n')
    rows.append(run('malformed-json-line', 1, cand, a.original_target, w))
    # failed verdicts: candidate alone, and identical on both sides (equal but failed)
    bad = copy.deepcopy(ct)
    bad[-1]['failed'] = 1
    case('candidate-failed-verdict', 1, bad, a.original_target)
    badboth_c, badboth_o = copy.deepcopy(ot), copy.deepcopy(ot)
    for events in (badboth_c, badboth_o):
        events[-1]['failed'] = 1
        events[next(i for i, e in enumerate(events) if e['event'] == 'check')]['ok'] = False
    case('identical-failed-original-both-sides', 1, badboth_c, None, orig_events=badboth_o)
    bad = copy.deepcopy(ct)
    next(e for e in bad if e['event'] == 'check')['ok'] = False
    case('candidate-check-not-ok', 1, bad, a.original_target)
    # check renames: an unexpected check renamed (both ok true), a whitelisted one changed, a detail changed, a swap, a deleted check
    checks = [i for i, e in enumerate(ct) if e['event'] == 'check']
    bad = copy.deepcopy(ct)
    bad[checks[3]]['name'] = bad[checks[3]]['name'] + ' (weakened)'
    case('unexpected-check-renamed', 1, bad, a.original_target)
    bad = copy.deepcopy(ct)
    bad[checks[2]]['detail'] = 'person_facing 19'
    case('unexpected-check-detail-changed', 1, bad, a.original_target)
    bad = copy.deepcopy(ct)
    bad[checks[6]]['name'] = 'the host was asked to turn the person in some frames'
    case('whitelisted-check-name-altered', 1, bad, a.original_target)
    bad = copy.deepcopy(ct)
    bad[checks[6]]['detail'] = '19 person_face steps in 20 frames'
    case('whitelisted-check-detail-altered', 1, bad, a.original_target)
    bad = copy.deepcopy(ct)
    bad[checks[6]], bad[checks[7]] = bad[checks[7]], bad[checks[6]]
    case('whitelisted-checks-swapped', 1, bad, a.original_target)
    bad = copy.deepcopy(ct)
    bad[checks[7]]['name'] = bad[checks[6]]['name']
    case('whitelisted-name-used-twice', 1, bad, a.original_target)
    bad = copy.deepcopy(ct)
    del bad[checks[4]]
    case('check-deleted', 1, bad, a.original_target)
    bad = copy.deepcopy(ct)
    bad[checks[0]]['name'] = 'a different check'
    case('control-style-rename-on-first-check', 1, bad, a.original_target)
    bad = copy.deepcopy(cc)
    next(e for e in bad if e['event'] == 'check')['name'] = 'control: renamed'
    case('control-check-renamed', 1, bad, a.original_control)
    # frame mapping: output differs, input chain broken, frame missing, frame renumbered, extra facing in the control
    frames = [i for i, e in enumerate(ct) if e['event'] == 'frame']
    bad = copy.deepcopy(ct)
    bad[frames[9]]['face_out'][0] += 1e-9
    case('frame-output-differs-in-last-digits', 1, bad, a.original_target)
    bad = copy.deepcopy(ct)
    bad[frames[5]]['face_in'] = bad[frames[0]]['face_in']
    case('frame-input-chain-broken', 1, bad, a.original_target)
    bad = copy.deepcopy(ct)
    bad[frames[0]]['face_in'] = bad[frames[0]]['face_out']
    case('first-frame-input-not-the-before-attitude', 1, bad, a.original_target)
    bad = copy.deepcopy(ct)
    del bad[frames[19]]
    case('frame-missing', 1, bad, a.original_target)
    bad = copy.deepcopy(ct)
    bad[frames[7]]['frame'] = 99
    case('frame-renumbered', 1, bad, a.original_target)
    bad = copy.deepcopy(ct)
    bad[frames[3]]['heading'] += 1e-9
    case('frame-heading-differs', 1, bad, a.original_target)
    bad = copy.deepcopy(cc)
    bad[[i for i, e in enumerate(cc) if e['event'] == 'frame'][4]]['face_out'] = [1, 0, 0, 0, 1, 0, 0, 0, 1]
    case('control-has-a-facing', 1, bad, a.original_control)
    # verdict and hook-report changes, a wrong pairing, an unmapped event change
    bad = copy.deepcopy(ct)
    bad[-1]['step_modes'] = ['person_begin=20', 'person_end=20', 'place=20']
    case('verdict-step-mode-not-renamed', 1, bad, a.original_target)
    bad = copy.deepcopy(ct)
    bad[-1]['host_steps'] = 59
    case('verdict-host-steps-changed', 1, bad, a.original_target)
    bad = copy.deepcopy(ct)
    next(e for e in bad if e['event'] == 'aim-hash')['physical'] = '0' * 16
    case('hook-hash-changed', 1, bad, a.original_target)
    bad = copy.deepcopy(ct)
    bad.insert(60, {'event': 'extra'})
    case('extra-event', 1, bad, a.original_target)
    case('target-candidate-against-control-original', 1, ct, a.original_control)
    case('control-candidate-against-target-original', 1, cc, a.original_target)

    after = {p: sha(p) for p in inputs}
    record = {
        'note': 'checker self-test (HELPER): derived copies only, no native campaign was run; originals hashed before and after',
        'checker_sha256': sha(CHECKER), 'input_hashes_before': before, 'input_hashes_after': after, 'inputs_unchanged': before == after,
        'cases': rows, 'all_ok': before == after and all(r['ok'] for r in rows),
    }
    json.dump(record, open(a.out, 'w', encoding='utf-8', newline='\n'), indent=1)
    for r in rows:
        print(('ok  ' if r['ok'] else 'FAIL'), r['case'], 'exit', r['actual_exit'], '|', r['first_problem'][:100])
    print('ALL OK' if record['all_ok'] else 'FAILURES')
    sys.exit(0 if record['all_ok'] else 1)


main()
