"""Targeted raw input map of the positive-contract correction: the changed paths by raw bytes, git status, and the production roots' tree ids at the 681 runtime, the 96cc head and the working HEAD.
usage: targetmap.py <out.json> <label>"""
import datetime, hashlib, json, subprocess, sys
from pathlib import Path

REPO = Path('R:/d3-evidence/collision-characterization-20261029')
RUNTIME, PARENT = '681ccfe5476a7c0a90036bc01cd054358c751162', '96cc958d5fca9fdf847cf1853020583071dedde8'
LUA = 'eech-campaign/crates/eech-world/lua/entity-authority.lua'
TEST = 'eech-campaign/crates/eech-world/tests/entity_authority.rs'
ROOTS = ['eech-campaign/crates/eech-engine-sys', 'eech-campaign/crates/eech-dc', 'eech-campaign/crates/eech-engine', 'eech-campaign/crates/eech-world/src',
         'eech-campaign/Cargo.toml', 'eech-campaign/Cargo.lock', 'eech-campaign/.cargo', 'aphavoc', 'modules', '.github', 'eech-campaign/crates/eech-world/lua/campaign.lua']


def git(*args):
    return subprocess.run(['git', '-C', str(REPO), *args], check=True, capture_output=True).stdout


def blob(b):
    return hashlib.sha1(b'blob ' + str(len(b)).encode() + b'\0' + b).hexdigest()


head = git('rev-parse', 'HEAD').decode().strip()
files = {}
for path in (LUA, TEST):
    raw = (REPO / path).read_bytes()
    files[path] = {'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'git_blob_of_raw_bytes': blob(raw), 'blob_at_head': git('rev-parse', f'HEAD:{path}').decode().strip(),
                   'blob_at_96cc': git('rev-parse', f'{PARENT}:{path}').decode().strip(), 'blob_at_681': git('rev-parse', f'{RUNTIME}:{path}').decode().strip()}
roots = {r: {'at_681': git('rev-parse', f'{RUNTIME}:{r}').decode().strip(), 'at_working_head': git('rev-parse', f'HEAD:{r}').decode().strip()} for r in ROOTS}
for r, v in roots.items():
    v['equal'] = v['at_681'] == v['at_working_head']
record = {'label': sys.argv[2], 'created_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'head': head, 'runtime_681': RUNTIME, 'parent_96cc': PARENT,
          'git_status_porcelain': [l for l in git('status', '--porcelain=v1').decode().splitlines() if l], 'untracked': [p for p in git('ls-files', '--others', '--exclude-standard', '-z').decode().split('\0') if p],
          'changed_paths_raw': files, 'production_roots_vs_681': roots, 'lua_directory_paths_changed_681_to_head': git('diff', '--name-only', RUNTIME, 'HEAD', '--', 'eech-campaign/crates/eech-world/lua').decode().split()}
Path(sys.argv[1]).write_bytes((json.dumps(record, indent=1, sort_keys=True) + '\n').encode())
print(json.dumps({k: record[k] for k in ('label', 'head', 'git_status_porcelain', 'untracked')}, indent=1))
print({r: v['equal'] for r, v in roots.items()})
