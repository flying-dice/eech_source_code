"""Run one named public Lua/native diagnostic; preserve actual exits, never infer acceptance."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

p = argparse.ArgumentParser()
p.add_argument('--source', required=True)
p.add_argument('--snapshot', type=Path, required=True)
p.add_argument('--bin', type=Path, required=True)
p.add_argument('--root', type=Path, required=True)
p.add_argument('--out', type=Path, required=True)
p.add_argument('--script', choices=['campaign.lua', 'model-authority.lua', 'tree-authority.lua', 'entity-authority.lua'], required=True)
p.add_argument('--scenario', choices=['georgia_retail', 'lebanon_retail'], required=True)
p.add_argument('--nonretail-models', action='store_true')
p.add_argument('--wrapper', type=Path)
p.add_argument('--literal-driver', type=Path)
p.add_argument('--literal-source')
p.add_argument('--literal-git-blob')
p.add_argument('--literal-sha256')
p.add_argument('--arg', action='append', default=[])
a = p.parse_args()
assert re.fullmatch('[0-9a-f]{40}', a.source)
assert not a.out.exists(), 'output already exists'
assert a.root.is_dir()
repo = a.snapshot / 'eech-campaign'
assert subprocess.check_output(['git', '-C', str(a.snapshot), 'rev-parse', 'HEAD'], text=True).strip() == a.source
assert not subprocess.check_output(['git', '-c', 'core.longpaths=true', '-C', str(a.snapshot), 'status', '--porcelain'], text=True).strip()

def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

info = (a.bin / 'BUILD-INFO.txt').read_text(encoding='utf-8')
assert 'commit: ' + a.source in info and 'uncommitted changes in the build inputs: 0 files' in info
if a.script == 'campaign.lua':
    assert 'FIXTURE BUILD' not in info and not a.nonretail_models, 'Ordinary retail recording only'
entries = re.findall(r'^  ([0-9a-f]{64})  (.+)$', info, re.M)
wanted = {'eech_dc.dll', 'eech-world.exe', 'lua.dll'} | {'lua/' + x.name for x in (repo / 'crates/eech-world/lua').glob('*.lua')}
assert len(entries) == len(wanted) and {x[1] for x in entries} == wanted
assert all(sha(a.bin / name) == expected for expected, name in entries)
if a.literal_driver:
    assert a.script != 'campaign.lua' and not a.wrapper, 'No literal override of ordinary shipping scenario or mixed wrappers'
    assert a.literal_driver.is_file() and re.fullmatch('[0-9a-f]{40}', a.literal_source or '')
    assert re.fullmatch('[0-9a-f]{40}', a.literal_git_blob or '') and re.fullmatch('[0-9a-f]{64}', a.literal_sha256 or '')
    literal_bytes = a.literal_driver.read_bytes()
    assert hashlib.sha256(literal_bytes).hexdigest() == a.literal_sha256
    assert hashlib.sha1(b'blob ' + str(len(literal_bytes)).encode() + b'\0' + literal_bytes).hexdigest() == a.literal_git_blob

a.out.mkdir()
state = dict(source=a.source, snapshot=str(a.snapshot), binary=str(a.bin), root=str(a.root),
             script=a.script, scenario=a.scenario, controller_pid=os.getpid(), state='running',
             started_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
             verified_payloads=len(entries), build_info_sha256=sha(a.bin / 'BUILD-INFO.txt'),
             nonretail_models=a.nonretail_models, stages=[], scope='Focused public-path diagnostic; not full shipping acceptance')
if a.wrapper:
    assert a.wrapper.is_file()
    state['driver_wrapper'] = str(a.wrapper)
    state['driver_wrapper_sha256'] = sha(a.wrapper)

if a.literal_driver:
    state['external_diagnostic'] = {'path':str(a.literal_driver), 'source':a.literal_source,
                                    'git_blob':a.literal_git_blob, 'sha256':a.literal_sha256,
                                    'bytes':len(literal_bytes), 'no_native_binary_relabel':True}

def save():
    (a.out / 'controller.json').write_text(json.dumps(state, indent=2) + '\n', encoding='utf-8')

def run(name, command):
    with (a.out / (name + '.log')).open('xb') as log:
        child = subprocess.Popen([str(x) for x in command], cwd=a.out, stdout=subprocess.PIPE,
                                 stderr=subprocess.STDOUT, creationflags=subprocess.CREATE_NO_WINDOW)
        stage = dict(stage=name, args=[str(x) for x in command], pid=child.pid,
                     started_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(), state='running')
        state['stages'].append(stage)
        save()
        for chunk in iter(lambda: child.stdout.read1(65536), b''):
            log.write(chunk)
            log.flush()
        code = child.wait()
    stage.update(state='terminal', actual_exit=code, completed_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    save()
    print(json.dumps(dict(stage=name, pid=child.pid, actual_exit=code)), flush=True)
    return code

reference = repo / ('reference/m4-s3-mutation/inputs-georgia.json' if a.scenario == 'georgia_retail' else 'reference/lebanon-3h/inputs.json')
def manifest(name):
    return run(name, [sys.executable, '-B', '-X', 'utf8', repo / 'tools/input-manifest.py', a.root,
                      '--compare', reference, '--output', a.out / (name + '.json')])

save()
try:
    if not a.nonretail_models:
        assert manifest('pre-inputs') == 0
    command = [a.bin / 'eech-world.exe', a.literal_driver or a.wrapper or a.bin / ('lua/' + a.script),
               'root=' + a.root.as_posix(), 'scenario=' + a.scenario,
               'out=' + (a.out / 'model.jsonl').as_posix(),
               'metrics=' + (a.out / 'metrics.json').as_posix(), *a.arg]
    if a.wrapper:
        command += ['driver=' + (a.bin / ('lua/' + a.script)).as_posix(),
                    'audit=' + (a.out / 'frame-audit.json').as_posix()]
    native = run('native', command)
    if not a.nonretail_models:
        assert manifest('post-inputs') == 0
    assert all(sha(a.bin / name) == expected for expected, name in entries), 'payload changed'
    output = a.out / 'model.jsonl'
    if output.exists():
        rows = []
        def refuse_constant(value):
            raise ValueError('non-JSON constant: ' + value)
        for line in output.read_text(encoding='utf-8').splitlines():
            rows.append(json.loads(line, parse_constant=refuse_constant))
        state['json_rows'] = len(rows)
        state['json_events'] = [x.get('event') for x in rows]
        state['json_sha256'] = sha(output)
    state['actual_native_exit'] = native
    state['actual_exit'] = native
except BaseException as exc:
    state['actual_exit'] = 1
    state['controller_error'] = repr(exc)
    raise
finally:
    state.update(state='terminal', completed_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    save()
print(json.dumps({k: v for k, v in state.items() if k not in ('stages',)}), flush=True)
raise SystemExit(state['actual_exit'])
