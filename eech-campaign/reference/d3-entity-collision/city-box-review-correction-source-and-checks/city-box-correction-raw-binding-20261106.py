"""Raw-bytes binding of the correction's input map to the committed Git blobs, and its delta against the f06 map (read-only).
usage: binding.py <repo> <commit> <map_before> <map_after> <f06_map_after> <out>"""
import hashlib, json, subprocess, sys
repo, commit, before, after, f06, out = sys.argv[1:]
mb, ma, mf = (json.load(open(p)) for p in (before, after, f06))
paths = sorted(ma["files"])
req = "".join(f"{commit}:{p}\n" for p in paths).encode()
proc = subprocess.run(["git", "-C", repo, "cat-file", "--batch"], input=req, capture_output=True, check=True).stdout
pos, equal, different, missing = 0, 0, [], []
for p in paths:
    nl = proc.index(b"\n", pos); header = proc[pos:nl].split(); pos = nl + 1
    if header[-1] == b"missing":
        missing.append(p); continue
    size = int(header[2]); blob = proc[pos:pos + size]; pos += size + 1
    e = ma["files"][p]
    if e["size"] == size and e["raw_sha256"] == hashlib.sha256(blob).hexdigest():
        equal += 1
    else:
        different.append({"path": p, "raw_size": e["size"], "raw_sha256": e["raw_sha256"], "git_size": size, "git_bytes_sha256": hashlib.sha256(blob).hexdigest(), "git_blob": header[0].decode()})
assert pos == len(proc)
delta = sorted(p for p in set(ma["files"]) | set(mf["files"]) if ma["files"].get(p, {}).get("raw_sha256") != mf["files"].get(p, {}).get("raw_sha256"))
rec = {
    "schema": "d3.city-box-review-correction.raw-source-binding.v1", "commit": commit,
    "map_before": before, "map_after": after, "maps_literal_equal": open(before, "rb").read() == open(after, "rb").read(),
    "map_file_count": len(paths), "files_raw_equal_to_commit_blob_bytes": equal, "files_missing_at_commit": missing,
    "files_raw_different_from_commit_blob_bytes": different,
    "f06_map": f06, "paths_whose_raw_sha256_differs_from_the_f06_map": delta,
    "method": "raw SHA-256 and size from the map against the SHA-256 and size of git cat-file --batch blob bytes of the commit; not the filtered hash-object identity",
}
json.dump(rec, open(out, "w"), indent=2); print(json.dumps({k: rec[k] for k in ("maps_literal_equal", "map_file_count", "files_raw_equal_to_commit_blob_bytes", "paths_whose_raw_sha256_differs_from_the_f06_map")}, indent=1), [d["path"] for d in different])
