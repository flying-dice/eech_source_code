"""Explicit actual-input join of the CI Lua download repair to the accepted sources: the input map of this batch against the final POST map of the e326 proof-guards batch, and the Git paths against the frozen 1b89 native source.
No compiled, tested or linked input may change: the repair is the workflow, one helper, its selftest, prose, the card and this packet. Usage: join.py <repo> <e326-post-map> <new-post-map> <out.json>"""
import json
import subprocess
import sys

repo, old_path, new_path, out = sys.argv[1:5]
NATIVE_1B89 = "1b89c0b0ab613d2f6a08ea02dd6612457efa8982"
E326 = "e326a35724feb0566491c51d84d587995e2f9de8"
old = json.load(open(old_path, encoding="utf-8"))
new = json.load(open(new_path, encoding="utf-8"))


def git(*args):
    return subprocess.run(["git", "-C", repo, *args], capture_output=True, check=True).stdout.decode("utf-8", "replace")


problems = []
old_rows, new_rows = old["files"], new["files"]
changed = sorted(p for p in old_rows if p in new_rows and old_rows[p].get("raw_sha256") != new_rows[p].get("raw_sha256"))
added = sorted(set(new_rows) - set(old_rows))
removed = sorted(set(old_rows) - set(new_rows))
if changed or added or removed:
    problems.append("input map rows differ: %d changed, %d added, %d removed" % (len(changed), len(added), len(removed)))
if old["fingerprint_of_raw_sha256_map"] != new["fingerprint_of_raw_sha256_map"]:
    problems.append("the map fingerprints differ")

ROOTS = ("aphavoc/", "modules/", "eech-campaign/crates/")
CARGO = ("eech-campaign/Cargo.toml", "eech-campaign/Cargo.lock", "eech-campaign/tools/Dockerfile")
ALLOWED_EXACT = {
    ".github/workflows/eech-campaign.yml": "the workflow: the Lua step's download and the one added selftest step (ci3-workflow-static)",
    "eech-campaign/tools/fetch-pinned-archive.sh": "the new helper",
    "eech-campaign/tools/fetch-pinned-archive-selftest.sh": "the new selftest of the helper",
    "docs/governance/D3-FIVE-WHYS-20261102.md": "prose (one lesson)",
    "docs/handover/ACTIVE-EXTRACTION.json": "the card (finding registered; the CURRENT checkpoint)",
    "STATUS.md": "the CURRENT summary",
}
ALLOWED_PREFIX = ("eech-campaign/reference/d3-ci-lua-download/",)
paths = set(git("diff", "--name-only", NATIVE_1B89).splitlines()) | set(git("ls-files", "--others", "--exclude-standard").splitlines())
rows = []
for p in sorted(paths):
    if p.startswith(ROOTS) or p in CARGO:
        problems.append("a compiled, tested or linked input path changed against 1b89: " + p)
        rows.append({"path": p, "class": "COMPILED_OR_BUILD_INPUT (PROBLEM)"})
    elif p in ALLOWED_EXACT:
        rows.append({"path": p, "class": ALLOWED_EXACT[p]})
    elif p.startswith(ALLOWED_PREFIX):
        rows.append({"path": p, "class": "the new compact packet"})
    else:
        problems.append("unclassified path: " + p)
        rows.append({"path": p, "class": "UNCLASSIFIED (PROBLEM)"})
packet_rows = [r for r in rows if r["class"] == "the new compact packet"]
rows = [r for r in rows if r["class"] != "the new compact packet"]
between = git("diff", "--name-only", E326, NATIVE_1B89).splitlines()
between_native = [p for p in between if p.startswith(ROOTS) or p in CARGO]
if between_native:
    problems.append("the accepted e326 source differs from 1b89 in compiled inputs: %s" % between_native)
result = {
    "kind": "explicit input join: this batch against the e326 final POST map and the frozen 1b89 native source",
    "old_map": {"path": old_path, "fingerprint": old["fingerprint_of_raw_sha256_map"], "rows": len(old_rows)},
    "new_map": {"path": new_path, "fingerprint": new["fingerprint_of_raw_sha256_map"], "rows": len(new_rows)},
    "map_rows_changed": len(changed), "map_rows_added": len(added), "map_rows_removed": len(removed),
    "changed_paths_against_1b89": rows, "packet_files_against_1b89": len(packet_rows),
    "e326_to_1b89_paths": between, "e326_to_1b89_compiled_input_paths": between_native,
    "dockerfile_cargo_unchanged_against_1b89": subprocess.run(["git", "-C", repo, "diff", "--quiet", NATIVE_1B89, "--", *CARGO]).returncode == 0,
    "conclusion": "no compiled, tested or linked input changed: the accepted 0be 16/49 and the e326 affected runs, and Root's native 1b89 evidence (ordinary and observer shorts, hosted scan), keep their true identity and are reused unchanged; only the CI download step, a helper with its selftest, prose and records changed" if not problems else "NOT EQUIVALENT: see problems",
    "problems": problems,
}
open(out, "w", encoding="utf-8", newline="\n").write(json.dumps(result, indent=2) + "\n")
print(json.dumps({"changed": len(changed), "added": len(added), "removed": len(removed), "paths": len(rows), "problems": problems}, indent=1))
