"""Identity of the scripts and tools the final affected checks read that the batched input map does not cover (tools/, the AST check, the direct runner, the Linux route)."""
import hashlib
import json
import pathlib
import subprocess
import sys

repo = pathlib.Path(sys.argv[1])
out = pathlib.Path(sys.argv[2])
assert not out.exists(), out
files = [
    "eech-campaign/tools/sector-height-baseline-windows.ps1",
    "eech-campaign/tools/build-windows-sector-height-observer.sh",
    "eech-campaign/tools/native-arguments.ps1",
    "eech-campaign/docs/d3-sector-height-public-baseline-plan.md",
]
external = [
    "R:/d3-evidence/check-sho-windows-wrapper-ast-20261107.ps1",
    "R:/d3-evidence/run-sector-height-baseline-direct-20261107.py",
    "R:/d3-evidence/check-pose-motion-diag-linux-20261105.sh",
    "R:/d3-evidence/sector-roster-inputmap-batched-20261106.py",
]
rows = []
for f in files:
    p = repo / f
    data = p.read_bytes()
    rows.append({"path": f, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(), "git_hash_object": subprocess.check_output(["git", "-C", str(repo), "hash-object", f], text=True).strip()})
for f in external:
    data = pathlib.Path(f).read_bytes()
    rows.append({"path": f, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(), "git_hash_object": None})
agg = hashlib.sha256("".join("%s %s\n" % (r["path"], r["sha256"]) for r in rows).encode()).hexdigest()
out.write_text(json.dumps({"kind": "tool and script identity read by the final affected checks (outside the batched input map)", "image": "sha256:4a5618cbc1e13e96caa4cb1ff0076c4c24dc883876d0bdec1398024f3a2d6c51",
                           "volume": "eech-linux-d3-collision-driver-5416adb3b-20261029", "files": rows, "aggregate_sha256": agg}, indent=2) + "\n", encoding="utf-8", newline="\n")
print(agg)
