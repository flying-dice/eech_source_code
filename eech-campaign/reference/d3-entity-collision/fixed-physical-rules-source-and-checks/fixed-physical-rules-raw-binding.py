"""Raw-bytes binding of the fixed-physical-rules input map to the committed Git blobs, plus the changed files outside the map's roots (read-only).

usage: binding.py <repo> <code-commit> <parent-commit> <map_before> <map_after> <out>

1. every file of the map: the size and raw SHA-256 of the working-tree bytes the map captured, against the size and SHA-256 of `git cat-file --batch` blob bytes of the code commit (not the filtered
   hash-object identity). A file whose raw bytes differ from the blob is listed apart with both identities, never normalised.
2. the changed files of the commit (parent..code) that are NOT in the map (the map's roots are the engine, world, dc crates and the C sources): their committed bytes are listed with size and SHA-256, and
   the working-tree bytes on disk now are compared with them literally (the working tree is the committed tree when this runs after the commit).
"""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

repo, commit, parent, before, after, out = sys.argv[1:]
mb, ma = (json.load(open(p, encoding="utf-8")) for p in (before, after))


def batch(rev_paths):
    req = "".join(f"{rev}:{path}\n" for rev, path in rev_paths).encode()
    proc = subprocess.run(["git", "-C", repo, "cat-file", "--batch"], input=req, capture_output=True, check=True).stdout
    pos, found = 0, []
    for rev, path in rev_paths:
        nl = proc.index(b"\n", pos)
        header = proc[pos:nl].split()
        pos = nl + 1
        if header[-1] == b"missing":
            found.append(None)
            continue
        size = int(header[2])
        found.append((header[0].decode(), proc[pos : pos + size]))
        pos += size + 1
    assert pos == len(proc)
    return found


paths = sorted(ma["files"])
blobs = batch([(commit, p) for p in paths])
equal, different, missing = 0, [], []
for path, blob in zip(paths, blobs):
    if blob is None:
        missing.append(path)
        continue
    entry = ma["files"][path]
    if entry["size"] == len(blob[1]) and entry["raw_sha256"] == hashlib.sha256(blob[1]).hexdigest():
        equal += 1
    else:
        different.append(
            {
                "path": path,
                "raw_size": entry["size"],
                "raw_sha256": entry["raw_sha256"],
                "git_size": len(blob[1]),
                "git_bytes_sha256": hashlib.sha256(blob[1]).hexdigest(),
                "git_blob": blob[0],
            }
        )

changed = subprocess.run(["git", "-C", repo, "diff", "--name-only", "-z", parent, commit], capture_output=True, check=True).stdout.decode().split("\0")
changed = sorted(p for p in changed if p)
outside = [p for p in changed if p not in ma["files"]]
outside_blobs = batch([(commit, p) for p in outside])
outside_rows = []
for path, blob in zip(outside, outside_blobs):
    on_disk = (Path(repo) / path).read_bytes() if (Path(repo) / path).exists() else None
    outside_rows.append(
        {
            "path": path,
            "git_blob": blob[0] if blob else None,
            "git_size": len(blob[1]) if blob else None,
            "git_bytes_sha256": hashlib.sha256(blob[1]).hexdigest() if blob else None,
            "working_tree_bytes_equal_to_blob": bool(blob) and on_disk == blob[1],
        }
    )

parent_paths = {p: None for p in changed}
parent_blobs = batch([(parent, p) for p in changed])
rec = {
    "schema": "d3.fixed-physical-rules.raw-source-binding.v1",
    "commit": commit,
    "parent": parent,
    "map_before": before,
    "map_after": after,
    "maps_literal_equal": open(before, "rb").read() == open(after, "rb").read(),
    "map_file_count": len(paths),
    "files_raw_equal_to_commit_blob_bytes": equal,
    "files_missing_at_commit": missing,
    "files_raw_different_from_commit_blob_bytes": different,
    "changed_files_vs_parent": changed,
    "changed_files_new_at_this_commit": [p for p, b in zip(changed, parent_blobs) if b is None],
    "changed_files_outside_the_map_roots": outside_rows,
    "method": "raw SHA-256 and size from the map against the SHA-256 and size of git cat-file --batch blob bytes of the commit; not the filtered hash-object identity",
}
json.dump(rec, open(out, "w", encoding="utf-8"), indent=2)
print(
    json.dumps(
        {
            k: rec[k]
            for k in ("maps_literal_equal", "map_file_count", "files_raw_equal_to_commit_blob_bytes", "files_missing_at_commit", "changed_files_vs_parent")
        },
        indent=1,
    )
)
print("raw different:", [d["path"] for d in different])
print("outside map, working tree != blob:", [r["path"] for r in outside_rows if not r["working_tree_bytes_equal_to_blob"]])
