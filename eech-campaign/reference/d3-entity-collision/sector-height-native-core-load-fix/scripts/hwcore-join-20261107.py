"""Explicit actual-input join of the Core-load loader fix to the source Root built and ran natively (the 17f build with the reviewed external Lua 9cb; its final POST map is the reference): which map rows changed, and WHICH COMPILED INPUTS changed. The loader fix changes ONE
compiled product input (aphavoc/source/ai/parser/parsgen.c); every other compiled or linked input is raw-identical, and the join states that the 17f Windows binaries are therefore NOT reusable for this source (new binaries need this new source).
Usage: join.py <repo> <reference-post-map> <new-post-map> <out.json>"""
import hashlib
import json
import subprocess
import sys

repo, old_path, new_path, out = sys.argv[1:5]
CODE = "9cb480fbb21e2bb5d492a056046e0bd854bc987b"
METADATA = "beb57c879af20e17aaa6b6c0c8cef20eb76fb690"
old = json.load(open(old_path, encoding="utf-8"))["files"]
new = json.load(open(new_path, encoding="utf-8"))["files"]
ALLOWED_MAP_CHANGES = {
    "aphavoc/source/ai/parser/parsgen.c": "THE ONE COMPILED PRODUCT INPUT of this fix: the saved-campaign loader (FILE_TAG_SAVED_CAMPAIGN): binary-handle size, full read, named fatals",
    "eech-campaign/crates/eech-engine-sys/tests/saved_campaign_load_guard.rs": "its source guard (a new test source, not a product input)",
}
# the compiled or linked product inputs of the native and hosted builds
COMPILED = (
    "aphavoc/source/", "modules/",
    "eech-campaign/crates/eech-engine-sys/csrc/", "eech-campaign/crates/eech-engine-sys/build/", "eech-campaign/crates/eech-engine-sys/src/", "eech-campaign/crates/eech-engine-sys/Cargo.toml", "eech-campaign/crates/eech-engine-sys/compat/",
    "eech-campaign/crates/eech-engine/src/", "eech-campaign/crates/eech-engine/Cargo.toml", "eech-campaign/crates/eech-dc/src/", "eech-campaign/crates/eech-dc/Cargo.toml",
    "eech-campaign/crates/eech-world/src/", "eech-campaign/crates/eech-world/Cargo.toml", "eech-campaign/Cargo.toml", "eech-campaign/Cargo.lock",
)
problems = []
changed = sorted(k for k in new if k in old and new[k]["raw_sha256"] != old[k]["raw_sha256"])
added = sorted(k for k in new if k not in old)
removed = sorted(k for k in old if k not in new)
for k in changed + added:
    if k not in ALLOWED_MAP_CHANGES:
        problems.append("a map row changed or was added outside the allowed set: " + k)
if removed:
    problems.append("map rows removed: %s" % removed)
compiled = sorted(k for k in new if k.startswith(COMPILED))
agg_old = hashlib.sha256("".join("%s %s\n" % (k, old[k]["raw_sha256"]) for k in compiled if k in old).encode()).hexdigest()
agg_new = hashlib.sha256("".join("%s %s\n" % (k, new[k]["raw_sha256"]) for k in compiled).encode()).hexdigest()
compiled_missing = [k for k in compiled if k not in old]
compiled_changed = [k for k in compiled if k in old and old[k]["raw_sha256"] != new[k]["raw_sha256"]]
if compiled_changed != ["aphavoc/source/ai/parser/parsgen.c"] or compiled_missing:
    problems.append("the compiled or linked inputs that differ from the reference map are not exactly parsgen.c: %s %s" % (compiled_changed, compiled_missing))


def git(*args):
    return subprocess.run(["git", "-C", repo, *args], capture_output=True, check=True).stdout.decode("utf-8", "replace")


# the Git view: nothing outside the allowed set, the contract, the records and the evidence packets differs from the code commit e42bb19a0 (the committed metadata checkpoints sit on top of it)
since = sorted(set(git("diff", "--name-only", CODE).splitlines()) | set(git("ls-files", "--others", "--exclude-standard").splitlines()))
ALLOWED_GIT = set(ALLOWED_MAP_CHANGES) | {"eech-campaign/docs/d3-sector-height-production-contract.md", "STATUS.md", "docs/handover/ACTIVE-EXTRACTION.json"}
ALLOWED_COMPILED_GIT = {"aphavoc/source/ai/parser/parsgen.c"}
PACKETS = ("eech-campaign/reference/d3-entity-collision/",)
rows = []
for p in since:
    if p in ALLOWED_GIT:
        rows.append({"path": p, "class": ALLOWED_MAP_CHANGES.get(p, "contract prose or records")})
    elif p.startswith(PACKETS):
        rows.append({"path": p, "class": "retained evidence packet (not an input of any check)"})
    else:
        rows.append({"path": p, "class": "UNEXPECTED CHANGE SINCE e42bb19a0"})
        problems.append("changed since the code commit outside the allowed set: " + p)
for p in since:
    if (p.startswith(COMPILED) or p.endswith((".c", ".h", "Cargo.toml", "Cargo.lock", "build.rs"))) and not p.startswith("eech-campaign/reference/") and p not in ALLOWED_COMPILED_GIT:
        problems.append("a compiled-input path other than parsgen.c differs from the code commit in Git: " + p)
result = {
    "kind": "explicit compiled-input change join of the Core-load loader fix against the 9cb480fbb code commit (beb metadata) and the reference final POST map",
    "code_commit": CODE, "metadata_commit": METADATA,
    "map_rows_changed": changed, "map_rows_added": added, "map_rows_removed": removed, "map_rows_identical": sum(1 for k in new if k in old and new[k]["raw_sha256"] == old[k]["raw_sha256"]),
    "compiled_input_rows": len(compiled), "compiled_inputs_changed": compiled_changed, "compiled_inputs_aggregate_sha256_reference": agg_old, "compiled_inputs_aggregate_sha256_now": agg_new, "compiled_inputs_identical": False, "windows_17f_binaries_reusable_for_this_source": False,
    "changed_in_git_since_code_commit": rows,
    "allowed_map_changes": ALLOWED_MAP_CHANGES,
    "conclusion": "exactly ONE compiled product input changed (aphavoc/source/ai/parser/parsgen.c, the saved-campaign loader); every other compiled and linked input is raw-identical to the reference map. The sealed 17f Windows binaries are NOT reused for this source: new Windows binaries need this new source, a new immutable plan and Root-owned native runs; nothing is relabelled and nothing is claimed natively" if not problems else "NOT AS EXPECTED: see problems",
    "problems": problems,
}
open(out, "w", encoding="utf-8", newline="\n").write(json.dumps(result, indent=2) + "\n")
print(json.dumps({"changed": changed, "added": added, "removed": removed, "compiled_rows": len(compiled), "identical": result["compiled_inputs_identical"], "paths": len(rows), "problems": problems}, indent=1))
sys.exit(1 if problems else 0)
