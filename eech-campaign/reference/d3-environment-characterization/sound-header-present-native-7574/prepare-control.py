"""Prepare the ONE fresh private retail Lebanon root for the header-present original-init boot control (no native is launched here).

1. copy the verified local retail bundle (the source every earlier original-init root was copied from; never modified here);
2. verify all original files against the UNCHANGED repo reference BEFORE anything is added (the stock tool, real exit);
3. add exactly TWO declared fixture files under cohokum/audio (a declared controlled input variation; no file of the root is touched);
4. build the CONTROLLED reference from the stock tool's own manifest of the prepared root, but only after an independent check that it equals the original reference plus exactly the two declared fixture entries;
   the accepted manifest (reference/lebanon-3h/inputs.json) is never overwritten and the deviation is never hidden.
New exclusive names only (stem original-sound-header-present-20261101-0400).
"""
import datetime
import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

B = Path('R:/d3-evidence')
STEM = 'original-sound-header-present-20261101-0400'
BUNDLE = Path('R:/eech-regression-20261025/roots/lebanon')
SNAPSHOT = B / 'environment-original-sound-init-source-7574caa2c'
REPO = B / 'environment-services-20261030'
TOOL = SNAPSHOT / 'eech-campaign/tools/input-manifest.py'
REFERENCE = SNAPSHOT / 'eech-campaign/reference/lebanon-3h/inputs.json'
FIXTURE = B / (STEM + '-fixture')
PARENT = B / ('environment-' + STEM + '-root')
ROOT = PARENT / 'lebanon-header-present'
RECEIPT = B / (STEM + '-prepare.json')
CONTROLLED = B / (STEM + '-controlled-inputs.json')
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
now = lambda: datetime.datetime.now(datetime.timezone.utc).isoformat()

assert not PARENT.exists() and not RECEIPT.exists() and not CONTROLLED.exists()
state = dict(pid=os.getpid(), state='preparing', started_utc=now(), stem=STEM, python=sys.executable,
             scope='Fresh private retail Lebanon root for ONE controlled original-init boot with a valid sound.hdr/sound.dat. No native launched. The 1179 original input files are preserved; exactly two declared fixture files are added under cohokum/audio.',
             steps=[])
def save():
    RECEIPT.write_text(json.dumps(state, indent=2) + '\n', encoding='utf8')
def step(name, **fields):
    state['steps'].append(dict(name=name, **fields)); save(); print(json.dumps(dict(step=name, **{k: v for k, v in fields.items() if k in ('actual_exit', 'ok')})), flush=True)
def tool(args, log):
    command = [sys.executable, '-B', '-X', 'utf8', str(TOOL), *map(str, args)]
    with log.open('xb') as f:
        p = subprocess.run(command, stdout=f, stderr=subprocess.STDOUT, creationflags=subprocess.CREATE_NO_WINDOW)
    return p.returncode, command

save()
try:
    # identities that must already hold
    assert sha(REFERENCE) == sha(REPO / 'eech-campaign/reference/lebanon-3h/inputs.json'), 'the snapshot reference differs from the repo reference'
    declaration = json.loads((FIXTURE / 'fixture-declaration.json').read_text(encoding='utf8'))
    for name, info in declaration['files'].items():
        assert (FIXTURE / name).stat().st_size == info['bytes'] and sha(FIXTURE / name) == info['sha256'], name
    reference = json.loads(REFERENCE.read_text(encoding='utf8'))
    state.update(reference=str(REFERENCE), reference_sha256=sha(REFERENCE), reference_files=reference['files'], reference_combined_sha256=reference['combined_sha256'], fixture=str(FIXTURE),
                 fixture_files=declaration['files'], fixture_declaration_sha256=sha(FIXTURE / 'fixture-declaration.json'), bundle=str(BUNDLE))
    step('identities', ok=True)

    PARENT.mkdir()
    shutil.copytree(BUNDLE, ROOT)
    step('copy', ok=True, root=str(ROOT))

    code, command = tool([ROOT, '--compare', REFERENCE], B / (STEM + '-root-before-addition.log'))
    step('original-files-before-addition', actual_exit=code, args=command, log=str(B / (STEM + '-root-before-addition.log')))
    assert code == 0, 'the copied retail root does not match the unchanged reference'

    # nothing of the fixture's names may exist anywhere in the root, and no audio directory
    existing = [str(p) for p in ROOT.rglob('*') if p.name.lower() in ('sound.hdr', 'sound.dat', 'audio')]
    assert not existing, existing
    audio = ROOT / 'cohokum' / 'audio'
    audio.mkdir()
    for name in ('sound.hdr', 'sound.dat'):
        with (audio / name).open('xb') as target:
            target.write((FIXTURE / name).read_bytes())
        assert sha(audio / name) == declaration['files'][name]['sha256']
    step('fixture-added', ok=True, directory=str(audio))

    code, command = tool([ROOT, '--compare', REFERENCE], B / (STEM + '-root-after-addition-vs-original.log'))
    step('after-addition-vs-original-reference', actual_exit=code, expected_exit=1, args=command, log=str(B / (STEM + '-root-after-addition-vs-original.log')))
    text = (B / (STEM + '-root-after-addition-vs-original.log')).read_text(encoding='utf8')
    extras = sorted(line.split(': ', 1)[1].strip() for line in text.splitlines() if line.strip().startswith('only in this root'))
    assert code == 1 and extras == ['cohokum/audio/sound.dat', 'cohokum/audio/sound.hdr'], (code, extras, text)
    assert 'only in the manifest' not in text and 'different content' not in text

    code, command = tool([ROOT, '--output', CONTROLLED], B / (STEM + '-controlled-manifest.log'))
    step('controlled-manifest', actual_exit=code, args=command, log=str(B / (STEM + '-controlled-manifest.log')))
    assert code == 0
    controlled = json.loads(CONTROLLED.read_text(encoding='utf8'))
    original = {e['path']: e for e in reference['inputs']}
    mine = {e['path']: e for e in controlled['inputs']}
    added = {p: mine[p] for p in set(mine) - set(original)}
    assert len(original) == 1179 and all(mine.get(p) == e for p, e in original.items()), 'an original input file changed'
    assert sorted(added) == ['cohokum/audio/sound.dat', 'cohokum/audio/sound.hdr']
    assert all(added['cohokum/audio/' + n]['sha256'] == i['sha256'] and added['cohokum/audio/' + n]['size'] == i['bytes'] for n, i in declaration['files'].items())
    assert controlled['files'] == 1181 and controlled['bytes'] == reference['bytes'] + sum(i['bytes'] for i in declaration['files'].values())
    step('controlled-reference-verified', ok=True, controlled=str(CONTROLLED), controlled_sha256=sha(CONTROLLED), controlled_files=controlled['files'], controlled_combined_sha256=controlled['combined_sha256'])

    code, command = tool([ROOT, '--compare', CONTROLLED], B / (STEM + '-root-vs-controlled.log'))
    step('root-vs-controlled-reference', actual_exit=code, args=command, log=str(B / (STEM + '-root-vs-controlled.log')))
    assert code == 0
    state.update(state='terminal', actual_exit=0, root=str(ROOT), controlled_reference=str(CONTROLLED), completed_utc=now())
    save()
    print('PREPARED', ROOT)
except BaseException as exc:
    state.update(state='terminal', actual_exit=1, error=repr(exc), completed_utc=now())
    save()
    raise
