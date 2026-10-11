#!/usr/bin/env python3
# Shows that tools/observation-check.py detects what it claims to: it must
# pass a run's unmodified evidence, and fail each copy with one deliberate
# discrepancy:
#   moved       one object's recorded positions shifted about 11 m north
#   undeclared  one object's declaration dropped from the recording
#   no-death    one Destroyed event dropped from the recording
#   coalition   one declaration's coalition changed
#   not-gone    one disappearance dropped from the observations
#
# Usage: tools/observation-check-selftest.py <recording.acmi> <observations.jsonl> <metrics.json>
# (a short run, e.g. campaign.lua hours=0.05 acmi=... observe=... metrics=...)
import os, re, subprocess, sys, tempfile

here = os.path.dirname(os.path.abspath(__file__))
acmi, obs, metrics = sys.argv[1:4]
recording = open(acmi, encoding='utf-8', newline='').read().split('\n')
observations = open(obs, encoding='utf-8', newline='').read().split('\n')


def first(lines, pattern):
    return next(i for i, line in enumerate(lines) if re.search(pattern, line))


def moved(lines):
    # an aircraft: shift every position it has 0.0001 degree of latitude
    oid = lines[first(lines, r'^[0-9a-f]+,Type=Air\+')].split(',')[0]
    out = []
    for line in lines:
        if line.startswith(oid + ',T='):
            head, t = line.split(',T=')
            f = t.split('|')
            f[1] = f'{float(f[1]) + 0.0001:.7f}'
            line = head + ',T=' + '|'.join(f)
        out.append(line)
    return out


def undeclared(lines):
    i = first(lines, r'^[0-9a-f]+,Type=Air\+Rotorcraft')
    return lines[:i] + lines[i + 1:]


def no_death(lines):
    i = first(lines, r'^0,Event=Destroyed\|')
    return lines[:i] + lines[i + 1:]


def coalition(lines):
    i = first(lines, r'Coalition=Russia')
    return lines[:i] + [lines[i].replace('Coalition=Russia', 'Coalition=United States')] + lines[i + 1:]


def not_gone(lines):
    i = first(lines, r'"ev":"gone"')
    return lines[:i] + lines[i + 1:]


def check(recording_lines, observation_lines):
    with tempfile.TemporaryDirectory() as d:
        a, o = os.path.join(d, 'r.acmi'), os.path.join(d, 'o.jsonl')
        open(a, 'w', encoding='utf-8', newline='').write('\n'.join(recording_lines))
        open(o, 'w', encoding='utf-8', newline='').write('\n'.join(observation_lines))
        r = subprocess.run([sys.executable, os.path.join(here, 'observation-check.py'), a, o, metrics], capture_output=True, text=True)
        return r.returncode


cases = [
    ('unmodified', recording, observations, 0),
    ('moved', moved(recording), observations, 1),
    ('undeclared', undeclared(recording), observations, 1),
    ('no-death', no_death(recording), observations, 1),
    ('coalition', coalition(recording), observations, 1),
    ('not-gone', recording, not_gone(observations), 1),
]
status = 0
for name, r, o, want in cases:
    got = check(r, o)
    ok = got == want
    status |= not ok
    print(f"{name:12} {'passes' if got == 0 else 'fails'}: {'as expected' if ok else 'NOT AS EXPECTED'}")
sys.exit(status)
