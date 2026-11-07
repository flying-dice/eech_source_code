"""Batched per-file input map of the sector-roster change (EEDCS-49 / Task49): raw SHA-256 of every file the selected checks can read, and its Git blob identity.

usage: inputmap.py <repo> <out.json> [<head-rev>]

The set is every tracked or untracked (not ignored) file under the C sources the engine build compiles (aphavoc/source, modules), the engine/world/dc crates (src, csrc, compat, build, tests, Cargo.toml),
the workspace Cargo.toml/Cargo.lock and the sources.txt/build inputs. For each file: the size, the raw SHA-256 of the working-tree bytes, the Git blob id Git would give those bytes (`git hash-object
--path`, i.e. after Git's own clean filters), and, when the file exists at `head-rev`, that revision's blob id and whether the raw bytes equal the blob's bytes. Raw and Git-filtered identities that differ
are listed apart, never normalised.
"""
import hashlib
import json
import subprocess
import sys

repo, out = sys.argv[1], sys.argv[2]
head = sys.argv[3] if len(sys.argv) > 3 else "HEAD"


def git(*args, binary=False):
    result = subprocess.run(["git", "-C", repo, *args], capture_output=True, check=True)
    return result.stdout if binary else result.stdout.decode("utf-8", "replace")


roots = [
    "aphavoc/source",
    "modules",
    "eech-campaign/Cargo.toml",
    "eech-campaign/Cargo.lock",
    "eech-campaign/crates/eech-engine-sys",
    "eech-campaign/crates/eech-engine",
    "eech-campaign/crates/eech-dc",
    "eech-campaign/crates/eech-world",
    "eech-campaign/rust-toolchain.toml",
]
listed = git("ls-files", "--cached", "--others", "--exclude-standard", "-z", "--", *roots).split("\0")
paths = sorted({p for p in listed if p})
at_head = {}
for line in git("ls-tree", "-r", "-z", head, "--", *roots).split("\0"):
    if line:
        meta, path = line.split("\t", 1)
        at_head[path] = meta.split()[2]

# Same evidence schema as inputmap.py; batch Git plumbing rather than two subprocesses per file.
# Raw source must remain stable for this capture. The post-hash reread below enforces that.
from pathlib import Path
captured = {}
for path in paths:
    try:
        captured[path] = (Path(repo) / path).read_bytes()
    except FileNotFoundError:
        pass
paths = sorted(captured)
if any("\n" in p or "\r" in p for p in paths):
    raise RuntimeError("A newline-bearing path needs explicit Git C-quoting; do not silently hash it")
filtered = subprocess.run(["git", "-C", repo, "hash-object", "--stdin-paths"], input=("\n".join(paths) + "\n").encode(), capture_output=True, check=True).stdout.decode().splitlines()
assert len(filtered) == len(paths)
unique_blobs = sorted({at_head[p] for p in paths if p in at_head})
raw_batch = subprocess.run(["git", "-C", repo, "cat-file", "--batch"], input=("\n".join(unique_blobs) + "\n").encode(), capture_output=True, check=True).stdout
blobs = {}
offset = 0
for expected_blob in unique_blobs:
    split = raw_batch.index(b"\n", offset)
    blob, kind, size = raw_batch[offset:split].decode().split()
    assert blob == expected_blob and kind == "blob"
    size = int(size)
    offset = split + 1
    blobs[blob] = raw_batch[offset:offset + size]
    assert len(blobs[blob]) == size and raw_batch[offset + size:offset + size + 1] == b"\n"
    offset += size + 1
assert offset == len(raw_batch)
files = {}
differs_raw_vs_git = []
for path, git_blob in zip(paths, filtered):
    data = captured[path]
    assert (Path(repo) / path).read_bytes() == data, "Source changed during map capture: " + path
    entry = {"size": len(data), "raw_sha256": hashlib.sha256(data).hexdigest(), "git_blob_of_raw_bytes": git_blob}
    if path in at_head:
        entry["head_blob"] = at_head[path]
        committed = blobs[at_head[path]]
        entry["head_blob_bytes_sha256"] = hashlib.sha256(committed).hexdigest()
        entry["raw_equals_head_blob_bytes"] = committed == data
        entry["changed_from_head"] = committed != data
        entry["git_filter_would_rewrite_identity"] = at_head[path] != git_blob
        if (committed == data and at_head[path] != git_blob) or (committed != data and at_head[path] == git_blob):
            differs_raw_vs_git.append(path)
    else:
        entry["head_blob"] = None
        entry["changed_from_head"] = True
    files[path] = entry

changed = sorted(p for p, e in files.items() if e["changed_from_head"])
deleted = sorted(p for p in at_head if p not in files)
digest = hashlib.sha256(json.dumps({p: files[p]["raw_sha256"] for p in sorted(files)}, sort_keys=True).encode()).hexdigest()
report = {
    "repo": repo,
    "head_rev": head,
    "head_sha": git("rev-parse", head).strip(),
    "file_count": len(files),
    "changed_from_head_count": len(changed),
    "changed_from_head": changed,
    "deleted_from_head": deleted,
    "files_whose_raw_bytes_and_git_filtered_identity_disagree_with_the_committed_blob": differs_raw_vs_git,
    "fingerprint_of_raw_sha256_map": digest,
    "files": files,
}
json.dump(report, open(out, "w"), indent=1, sort_keys=True)
print(json.dumps({k: v for k, v in report.items() if k != "files" and k != "changed_from_head"}, indent=1))
