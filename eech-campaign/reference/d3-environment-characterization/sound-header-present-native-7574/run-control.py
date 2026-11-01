"""ONE controlled-input boot of the ORIGINAL sound initialization with a valid sound.hdr / sound.dat (Root launches this; it builds nothing).

Binds the EXISTING immutable 7574caa2c native (snapshot, bin, all 62 BUILD-INFO payloads including the unchanged env-sound-init.lua), the prepared private root, the fixture and the controlled reference, then runs
the native with seconds=0 frame_ms=100 (initialization and boot lookup only; NOT a replacement for the closed natural 120 s characterization, no Tacview, no full-pack equality claim). The input deviation is declared:
exactly two extra files under cohokum/audio, verified against the UNCHANGED original reference before and after the run. Real numeric exits are recorded per stage; nothing is retried, retained evidence is never overwritten.

usage: python -B -X utf8 run-original-sound-header-present-20261101-0400.py          (no arguments)
exit: 0 all stages and the independent expectation check passed; 3 only the expectation check found a difference (investigate, never tune); 1 any other failure.
"""
import datetime
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path

B = Path('R:/d3-evidence')
STEM = 'original-sound-header-present-20261101-0400'
SOURCE = '7574caa2ca87a2a44608a234fd1dd73136b8e4c0'
SNAPSHOT = B / 'environment-original-sound-init-source-7574caa2c'
BIN = SNAPSHOT / 'eech-campaign/target/env-observer/sound-init-original-7574caa2c'
REPO = SNAPSHOT / 'eech-campaign'
TOOL = REPO / 'tools/input-manifest.py'
REFERENCE = REPO / 'reference/lebanon-3h/inputs.json'
ROOT = B / ('environment-' + STEM + '-root') / 'lebanon-header-present'
FIXTURE = B / (STEM + '-fixture')
CONTROLLED = B / (STEM + '-controlled-inputs.json')
PREPARE = B / (STEM + '-prepare.json')
CHECKER = B / ('check-' + STEM + '.py')
OUT = B / ('environment-original-sound-header-present-7574caa2c-20261101-0400')
SCRIPT = 'env-sound-init.lua'
ARGS = ['seconds=0', 'frame_ms=100']
EXTRAS = ['cohokum/audio/sound.dat', 'cohokum/audio/sound.hdr']
now = lambda: datetime.datetime.now(datetime.timezone.utc).isoformat()


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


assert subprocess.check_output(['git', '-C', str(SNAPSHOT), 'rev-parse', 'HEAD'], text=True).strip() == SOURCE
assert not subprocess.check_output(['git', '-c', 'core.longpaths=true', '-C', str(SNAPSHOT), 'status', '--porcelain'], text=True).strip()
info = (BIN / 'BUILD-INFO.txt').read_text(encoding='utf-8')
assert 'commit: ' + SOURCE in info and 'uncommitted changes in the build inputs: 0 files' in info
assert 'FIXTURE BUILD (eech-dc/env-observer): test-only, not a shipping artifact' in info
entries = re.findall(r'^  ([0-9a-f]{64})  (.+)$', info, re.M)
wanted = {'eech_dc.dll', 'eech-world.exe', 'lua.dll'} | {'lua/' + x.name for x in (REPO / 'crates/eech-world/lua').glob('*.lua')}
assert len(entries) == 62 == len(wanted) and {x[1] for x in entries} == wanted
assert all(sha(BIN / name) == expected for expected, name in entries), 'a payload differs from BUILD-INFO'
script_sha = dict((name, expected) for expected, name in entries)['lua/' + SCRIPT]
assert sha(BIN / 'lua' / SCRIPT) == script_sha
assert ROOT.is_dir() and CONTROLLED.is_file() and CHECKER.is_file()
prepared = json.loads(PREPARE.read_text(encoding='utf-8'))
assert prepared['state'] == 'terminal' and prepared['actual_exit'] == 0
declaration = json.loads((FIXTURE / 'fixture-declaration.json').read_text(encoding='utf-8'))

OUT.mkdir()  # exclusive: fails when it exists
state = dict(source=SOURCE, snapshot=str(SNAPSHOT), binary=str(BIN), root=str(ROOT), script=SCRIPT, script_sha256=script_sha, scenario='lebanon_retail', controller_pid=os.getpid(), python=sys.executable,
             state='running', started_utc=now(), verified_payloads=len(entries), build_info_sha256=sha(BIN / 'BUILD-INFO.txt'), arguments=ARGS, cwd=str(OUT),
             reference=str(REFERENCE), reference_sha256=sha(REFERENCE), controlled_reference=str(CONTROLLED), controlled_reference_sha256=sha(CONTROLLED), preparation=str(PREPARE),
             fixture=str(FIXTURE), fixture_declaration_sha256=sha(FIXTURE / 'fixture-declaration.json'), fixture_files=declaration['files'],
             controlled_input_variation='cohokum/audio/sound.hdr and cohokum/audio/sound.dat added to a private copy of the retail Lebanon root; the 1179 original files are unchanged; synthetic bytes, not retail audio',
             scope='ORIGINAL sound initialization with a CONTROLLED valid header and data file: seconds=0, frame_ms=100, boot lookup only. Not the closed natural 120 s run, no Tacview, no full-pack equality, no host metadata move, not Task 51 / D3 acceptance.',
             stages=[])


def save():
    (OUT / 'controller.json').write_text(json.dumps(state, indent=2) + '\n', encoding='utf-8')


def run(name, command, cwd=OUT):
    with (OUT / (name + '.log')).open('xb') as log:
        child = subprocess.Popen([str(x) for x in command], cwd=cwd, stdout=log, stderr=subprocess.STDOUT, creationflags=subprocess.CREATE_NO_WINDOW)
        stage = dict(stage=name, args=[str(x) for x in command], cwd=str(cwd), pid=child.pid, started_utc=now(), state='running')
        state['stages'].append(stage)
        save()
        code = child.wait()
    stage.update(state='terminal', actual_exit=code, completed_utc=now())
    save()
    print(json.dumps(dict(stage=name, pid=child.pid, actual_exit=code)), flush=True)
    return code


def manifest(name, reference, expect):
    code = run(name, [sys.executable, '-B', '-X', 'utf8', TOOL, ROOT, '--compare', reference, '--output', OUT / (name + '.json')])
    return code == expect


def extras_only(name):
    text = (OUT / (name + '.log')).read_text(encoding='utf-8')
    found = sorted(line.split(': ', 1)[1].strip() for line in text.splitlines() if line.strip().startswith('only in this root'))
    return found == EXTRAS and 'only in the manifest' not in text and 'different content' not in text


def fixture_ok():
    return all(sha(FIXTURE / n) == i['sha256'] and sha(ROOT / 'cohokum/audio' / n) == i['sha256'] and (ROOT / 'cohokum/audio' / n).stat().st_size == i['bytes'] for n, i in declaration['files'].items())


verdict = dict()
try:
    save()
    verdict['fixture_before'] = fixture_ok()
    assert verdict['fixture_before'], 'fixture bytes differ before the run'
    assert manifest('pre-inputs-controlled', CONTROLLED, 0)
    assert manifest('pre-inputs-vs-original', REFERENCE, 1) and extras_only('pre-inputs-vs-original'), 'the root is not the original files plus exactly the two declared fixture files'
    command = [BIN / 'eech-world.exe', BIN / 'lua' / SCRIPT, 'root=' + ROOT.as_posix(), 'scenario=lebanon_retail', 'out=' + (OUT / 'environment.jsonl').as_posix(),
               'metrics=' + (OUT / 'metrics.json').as_posix(), *ARGS]
    native = run('native', command)
    state['actual_native_exit'] = native
    verdict['post_controlled'] = manifest('post-inputs-controlled', CONTROLLED, 0)
    verdict['post_vs_original_extras_only'] = manifest('post-inputs-vs-original', REFERENCE, 1) and extras_only('post-inputs-vs-original')
    verdict['fixture_after'] = fixture_ok()
    verdict['payloads_unchanged'] = all(sha(BIN / name) == expected for expected, name in entries)
    verdict['script_unchanged'] = sha(BIN / 'lua' / SCRIPT) == script_sha
    output = OUT / 'environment.jsonl'
    if output.exists():
        def refuse_constant(value):
            raise ValueError('non-JSON constant: ' + value)
        rows = [json.loads(line, parse_constant=refuse_constant) for line in output.read_text(encoding='utf-8').splitlines()]
        state['json_rows'] = len(rows)
        state['json_events'] = [x.get('event') for x in rows]
        state['json_sha256'] = sha(output)
    raw = OUT / 'environment.jsonl.case15.raw'
    if raw.exists():
        state['raw_sidecar_bytes'] = raw.stat().st_size
        state['raw_sidecar_sha256'] = sha(raw)
    infrastructure_ok = native == 0 and all(verdict.values()) and output.exists() and raw.exists()
    state['infrastructure_ok'] = infrastructure_ok
    if raw.exists():
        # the independent check runs even when something else failed, so a difference is never hidden behind another failure; it never changes a byte of evidence
        check = run('expectation-check', [sys.executable, '-B', '-X', 'utf8', '-I', CHECKER, OUT, '--out', OUT / 'expectation-check.json'], cwd=B)
        state['expectation_check_exit'] = check
    else:
        state['expectation_check_exit'] = None
    state['verdict'] = verdict
    state['actual_exit'] = 0 if infrastructure_ok and state['expectation_check_exit'] == 0 else (3 if infrastructure_ok and state['expectation_check_exit'] == 3 else 1)
except BaseException as exc:
    state['actual_exit'] = 1
    state['controller_error'] = repr(exc)
    state['verdict'] = verdict
    raise
finally:
    state.update(state='terminal', completed_utc=now())
    save()
print(json.dumps({k: v for k, v in state.items() if k not in ('stages', 'json_events')}), flush=True)
raise SystemExit(state['actual_exit'])
