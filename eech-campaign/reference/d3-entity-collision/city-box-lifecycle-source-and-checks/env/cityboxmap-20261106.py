"""Targeted raw map of the CITY box lifecycle cut: the changed paths by raw bytes, git status, and the production roots' tree ids at the sealed 681 runtime, the pre-change parent and the working HEAD.
usage: cityboxmap.py <out.json> <label> <parent-commit>"""
import datetime, hashlib, json, subprocess, sys
from pathlib import Path

REPO = Path('R:/d3-evidence/collision-characterization-20261029')
RUNTIME = '681ccfe5476a7c0a90036bc01cd054358c751162'
PARENT = sys.argv[3]
CHANGED = [
    'aphavoc/source/entity/mobile/mb_hostentity.h',
    'aphavoc/source/entity/mobile/mb_hostmodel.c',
    'aphavoc/source/entity/mobile/mb_hostmodel.h',
    'aphavoc/source/entity/mobile/mb_hostplacemap.h',
    'eech-campaign/crates/eech-engine-sys/csrc/eech_entity_dialogue_check.c',
    'eech-campaign/crates/eech-engine-sys/tests/entity_roster_guard.rs',
    'eech-campaign/crates/eech-world/lua/entity-authority.lua',
    'eech-campaign/crates/eech-world/src/model_host.rs',
    'eech-campaign/crates/eech-world/src/roster.rs',
    'eech-campaign/crates/eech-world/tests/city_box_lifecycle.rs',
    'eech-campaign/crates/eech-world/tests/entity_authority.rs',
]
ROOTS = ['eech-campaign/crates/eech-engine-sys', 'eech-campaign/crates/eech-dc', 'eech-campaign/crates/eech-engine', 'eech-campaign/crates/eech-world/src',
         'eech-campaign/crates/eech-world/lua', 'eech-campaign/crates/eech-world/lua/campaign.lua', 'eech-campaign/Cargo.toml', 'eech-campaign/Cargo.lock',
         'eech-campaign/.cargo', 'aphavoc', 'aphavoc/source/entity/mobile', 'modules', '.github']


def git(*args):
    return subprocess.run(['git', '-C', str(REPO), *args], check=True, capture_output=True).stdout


def blob(b):
    return hashlib.sha1(b'blob ' + str(len(b)).encode() + b'\0' + b).hexdigest()


def rev(spec):
    try:
        return git('rev-parse', spec).decode().strip()
    except subprocess.CalledProcessError:
        return None


head = git('rev-parse', 'HEAD').decode().strip()
files = {}
for path in CHANGED:
    raw = (REPO / path).read_bytes()
    files[path] = {'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'git_blob_of_raw_bytes': blob(raw), 'blob_at_head': rev(f'HEAD:{path}'),
                   'blob_at_parent': rev(f'{PARENT}:{path}'), 'blob_at_681': rev(f'{RUNTIME}:{path}')}
roots = {r: {'at_681': rev(f'{RUNTIME}:{r}'), 'at_parent': rev(f'{PARENT}:{r}'), 'at_working_head': rev(f'HEAD:{r}')} for r in ROOTS}
for v in roots.values():
    v['equal_681'] = v['at_681'] == v['at_working_head']
    v['equal_parent'] = v['at_parent'] == v['at_working_head']
record = {'label': sys.argv[2], 'created_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'head': head, 'runtime_681': RUNTIME, 'parent': PARENT,
          'git_status_porcelain': [l for l in git('status', '--porcelain=v1').decode().splitlines() if l], 'untracked': [p for p in git('ls-files', '--others', '--exclude-standard', '-z').decode().split('\0') if p],
          'changed_paths_raw': files, 'production_roots': roots,
          'paths_changed_parent_to_head': git('diff', '--name-only', PARENT, 'HEAD').decode().split()}
Path(sys.argv[1]).write_bytes((json.dumps(record, indent=1, sort_keys=True) + '\n').encode())
print(json.dumps({k: record[k] for k in ('label', 'head', 'git_status_porcelain', 'untracked')}, indent=1))
print({r: (v['equal_681'], v['equal_parent']) for r, v in roots.items()})
print(len(record['paths_changed_parent_to_head']), 'paths changed parent->head')
