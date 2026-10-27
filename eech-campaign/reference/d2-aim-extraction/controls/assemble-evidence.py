#!/usr/bin/env python3
"""task-only (EEDCS-44 extraction): copies the retained, safe-to-publish artifacts of the private run directory into the repository evidence directory
eech-campaign/reference/d2-aim-extraction/ and writes SHA256SUMS.txt (raw and LF-normalised hash and size of every trace). Reads the private files only.
No retail asset, installation root or binary is copied."""
import hashlib
import json
import os
import shutil
import sys

SRC = r'R:\d2-evidence\aim-extraction-20261026'
DST = r'R:\eech_source_code\eech-campaign\reference\d2-aim-extraction'
TARGET = r'R:\eech_source_code\eech-campaign\target'

TRACES = {
    # name in the repository: private path under out/
    'lebanon-c2-a.jsonl': 'c2-a/trace.jsonl',
    'lebanon-c2-b.jsonl': 'c2-b/trace.jsonl',
    'lebanon-c-a.jsonl': 'c-a/trace.jsonl',
    'lebanon-c-b.jsonl': 'c-b/trace.jsonl',
    'lebanon-dev2-x2-a.jsonl': 'x2-a/trace.jsonl',
    'lebanon-dev1-x1-a-killed-at-167s-physical-hash-differed.jsonl': 'x1-a/trace.jsonl',
    'person-target.jsonl': 'p2-t/trace.jsonl',
    'person-control.jsonl': 'p2-c/trace.jsonl',
    'person-target-repeat.jsonl': 'p2-t2/trace.jsonl',
    'person-first-head-target.jsonl': 'p-t/trace.jsonl',
    'person-first-head-control.jsonl': 'p-c/trace.jsonl',
    'person-first-head-target-repeat.jsonl': 'p-t2/trace.jsonl',
    'provider-candidate-reference-300s.jsonl': 'pv-reference/trace.jsonl',
    'provider-candidate-ready_slow-300s.jsonl': 'pv-ready_slow/trace.jsonl',
    'provider-candidate-step_slow-300s.jsonl': 'pv-step_slow/trace.jsonl',
    'provider-candidate-invalid-300s.jsonl': 'pv-invalid/trace.jsonl',
    'provider-candidate-own_model-300s.jsonl': 'pv-own_model/trace.jsonl',
    'provider-original-build-ready_slow-300s.jsonl': 'orig-pv-ready_slow/trace.jsonl',
    'provider-original-build-invalid-300s.jsonl': 'orig-pv-invalid/trace.jsonl',
}
BUILDS = {
    'BUILD-INFO-x-dev1.txt': 'aim-fixture/x-dev1/BUILD-INFO.txt',
    'BUILD-INFO-x-dev2.txt': 'aim-fixture/x-dev2/BUILD-INFO.txt',
    'BUILD-INFO-f749878ce-fixture-dirty51-superseded.txt': 'aim-fixture/f749878ce/BUILD-INFO.txt',
    'BUILD-INFO-f749878ce-fixture.txt': 'aim-fixture/f749878ce-lf/BUILD-INFO.txt',
    'BUILD-INFO-f749878ce-ordinary.txt': 'windows-f749878ce/BUILD-INFO.txt',
    'BUILD-INFO-a0d3b6f9c-fixture.txt': 'aim-fixture/a0d3b6f9c/BUILD-INFO.txt',
    'BUILD-INFO-a0d3b6f9c-ordinary.txt': 'windows-a0d3b6f9c/BUILD-INFO.txt',
    'BUILD-INFO-d3c9a5c5e-fixture.txt': 'aim-fixture/d3c9a5c5e/BUILD-INFO.txt',
    'BUILD-INFO-d3c9a5c5e-ordinary.txt': 'windows-d3c9a5c5e/BUILD-INFO.txt',
}
CONTROL_FILES = [
    'run-one.ps1', 'run-person.ps1', 'run-provider.ps1', 'run-ordinary-900.ps1', 'queue-1.ps1', 'queue-2.ps1', 'queue-3.ps1', 'build-aim-fixture-lf.sh', 'build-ordinary.sh',
    'compare-complete.py', 'compare-checkpoint.py', 'compare-person.py', 'symcheck-aim.sh', 'assemble-evidence.py',
]


def copy(src, dst):
    """a text copy with LF line endings (the stored form: Git stores LF, and a CRLF working copy counts as a change of the build inputs for a git without core.autocrlf)"""
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    data = open(src, 'rb').read().replace(b'\r\n', b'\n')
    open(dst, 'wb').write(data)


def digest(path):
    data = open(path, 'rb').read()
    return hashlib.sha256(data).hexdigest(), hashlib.sha256(data.replace(b'\r\n', b'\n')).hexdigest(), len(data), len(data.replace(b'\r\n', b'\n'))


def main():
    sums = []
    for name, rel in TRACES.items():
        src = os.path.join(SRC, 'out', rel)
        if not os.path.isfile(src):
            print('missing', src)
            continue
        dst = os.path.join(DST, 'traces', name)
        data = open(src, 'rb').read().replace(b'\r\n', b'\n')
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        open(dst, 'wb').write(data)
        raw, lf, nraw, nlf = digest(src)
        sums.append(f'{raw}  {nraw:>9}  raw   {name}\n{lf}  {nlf:>9}  lf    {name}')
        last = os.path.join(os.path.dirname(src), 'run.log')
        if os.path.isfile(last):
            lines = [l for l in open(last, encoding='utf-8', errors='replace').read().splitlines() if not l.startswith('eech: artwork not installed')]
            open(os.path.join(DST, 'traces', name.replace('.jsonl', '.stderr-final-lines.txt')), 'w', encoding='utf-8', newline='\n').write('\n'.join(lines[-3:]) + '\n')
    open(os.path.join(DST, 'traces', 'SHA256SUMS.txt'), 'w', encoding='utf-8', newline='\n').write('\n'.join(sums) + '\n')
    for name, rel in BUILDS.items():
        src = os.path.join(TARGET, rel)
        if os.path.isfile(src):
            copy(src, os.path.join(DST, 'builds', name))
        else:
            print('missing build info', src)
    for sub in ('comparisons',):
        d = os.path.join(SRC, sub)
        if os.path.isdir(d):
            for f in sorted(os.listdir(d)):
                copy(os.path.join(d, f), os.path.join(DST, sub, f))
    for f in CONTROL_FILES:
        s = os.path.join(SRC, f)
        if os.path.isfile(s):
            copy(s, os.path.join(DST, 'controls', f))
    # provider summaries, the shipping failure messages, the native exits
    exits = []
    out_dir = os.path.join(SRC, 'out')
    for name in sorted(os.listdir(out_dir)):
        d = os.path.join(out_dir, name)
        summary = os.path.join(d, 'summary.jsonl')
        if os.path.isfile(summary):
            copy(summary, os.path.join(DST, 'providers', name + '.summary.jsonl'))
            lines = [l for l in open(os.path.join(d, 'run.log'), encoding='utf-8', errors='replace').read().splitlines() if l.startswith('aim-provider:') or 'fatal' in l.lower()]
            open(os.path.join(DST, 'providers', name + '.stderr-lines.txt'), 'w', encoding='utf-8', newline='\n').write('\n'.join(lines[-5:]) + '\n')
        log = os.path.join(SRC, f'run-{name}.log')
        if os.path.isfile(log):
            text = open(log, encoding='utf-8', errors='replace').read()
            exits.append({'run': name, 'log': [l for l in text.splitlines() if ' end ' in l or ' DONE ' in l or 'native exit' in l or 'script exit' in l]})
    os.makedirs(os.path.join(DST, 'checks'), exist_ok=True)
    json.dump(exits, open(os.path.join(DST, 'checks', 'native-exits.json'), 'w', encoding='utf-8', newline='\n'), indent=1)
    for f in sorted(os.listdir(SRC)):
        if f.startswith(('roots-manifest-', 'verify-', 'symbol-check')) or f in ('queue-1.log', 'queue-2.log', 'queue-3.log'):
            copy(os.path.join(SRC, f), os.path.join(DST, 'checks', f))
        elif f.endswith('-v3.log') or f.endswith('-v4.log'):
            copy(os.path.join(SRC, f), os.path.join(DST, 'checks', f))
    print('assembled into', DST)


main()
