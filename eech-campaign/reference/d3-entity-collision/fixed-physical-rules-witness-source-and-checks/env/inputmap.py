"""Raw input map of the working tree of the witness lane: every tracked path with the sha256/size of its RAW bytes, the Git blob of those raw bytes, and the HEAD and 681 blobs.
usage: inputmap.py <repo> <out.json> <label>"""
import hashlib, json, subprocess, sys, datetime
from pathlib import Path

repo, out, label = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
CODE681 = '681ccfe5476a7c0a90036bc01cd054358c751162'


def git(*args):
    return subprocess.run(['git', '-C', str(repo), *args], check=True, capture_output=True).stdout


paths = [p for p in git('ls-files', '-z').decode().split('\0') if p]
head = git('rev-parse', 'HEAD').decode().strip()
tree = {}
for line in git('ls-tree', '-r', '-z', 'HEAD').decode().split('\0'):
    if line:
        meta, path = line.split('\t', 1)
        tree[path] = meta.split()[2]
tree681 = {}
for line in git('ls-tree', '-r', '-z', CODE681).decode().split('\0'):
    if line:
        meta, path = line.split('\t', 1)
        tree681[path] = meta.split()[2]
files, changed_head, changed_681, deleted = {}, [], [], []
for path in paths:
    full = repo / path
    if not full.is_file():
        deleted.append(path)
        continue
    raw = full.read_bytes()
    blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    entry = {'raw_sha256': hashlib.sha256(raw).hexdigest(), 'size': len(raw), 'git_blob_of_raw_bytes': blob,
             'head_blob': tree.get(path), 'blob_681': tree681.get(path)}
    files[path] = entry
    if blob != tree.get(path):
        changed_head.append(path)
    if blob != tree681.get(path):
        changed_681.append(path)
def classify(paths_flagged, blobs):
    """a path whose raw bytes differ from the commit's blob bytes ONLY by the working-tree line-ending filter is not a change (git status is the arbiter of the rest)"""
    real, eol_only = [], []
    for path in paths_flagged:
        want = blobs.get(path)
        if want is None:
            real.append(path)
            continue
        blob_bytes = git('cat-file', 'blob', want)
        raw = (repo / path).read_bytes()
        if raw == blob_bytes:
            continue
        crlf, lf = b'\r\n', b'\n'
        (eol_only if raw.replace(crlf, lf) == blob_bytes.replace(crlf, lf) else real).append(path)
    return real, eol_only


changed_head, eol_head = classify(changed_head, tree)
changed_681, eol_681 = classify(changed_681, tree681)
status = [line for line in git('status', '--porcelain=v1').decode().splitlines() if line]
record = {'label': label, 'created_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'head': head, 'code_681': CODE681,
          'file_count': len(files), 'changed_from_head': changed_head, 'changed_from_681': changed_681, 'eol_filter_only_vs_head': eol_head, 'eol_filter_only_vs_681': eol_681, 'git_status_porcelain': status, 'deleted_from_head': deleted,
          'untracked': [p for p in git('ls-files', '--others', '--exclude-standard', '-z').decode().split('\0') if p], 'files': files}
out.write_bytes((json.dumps(record, indent=1, sort_keys=True) + '\n').encode())
print(json.dumps({k: record[k] for k in ('label', 'head', 'file_count', 'changed_from_head', 'changed_from_681', 'git_status_porcelain', 'deleted_from_head', 'untracked')}, indent=1))
