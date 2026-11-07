"""Explicit actual-input join of the load-order diagnostic correction to the frozen loader-fix source 90e9547ee: which map rows changed since the loader fix's final POST map, and the COMPILED-INPUT EQUIVALENCE to it, bound
from the raw per-file rows before the freeze: every compiled or linked input (the original campaign's C sources, the engine-sys C/build/source/manifest, engine, dc and world sources, the workspace manifest and lockfile) is raw-identical
between the two maps, and only the host Lua driver and its scripted test moved (neither is a compiled product input). Usage: join.py <repo> <e42bb-post-map> <new-post-map> <out.json>"""
import hashlib
import json
import subprocess
import sys

repo, old_path, new_path, out = sys.argv[1:5]
CODE = "90e9547ee5fbfb31a461a861fdaa2adac5fbafe0"
METADATA = "8ad1e4bca0a30ea19d38402cbd95c2cb3c8efa1d"
old = json.load(open(old_path, encoding="utf-8"))["files"]
new = json.load(open(new_path, encoding="utf-8"))["files"]
ALLOWED_MAP_CHANGES = {
    "eech-campaign/crates/eech-world/lua/sector-height-witness.lua": "the host Lua driver (run by eech-world as an EXTERNAL script, bound separately by Git blob and SHA-256; not compiled into any binary)",
    "eech-campaign/crates/eech-world/tests/sector_height_witness_driver.rs": "its scripted tests (a test source, not a product input)",
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
if compiled_changed or compiled_missing or agg_old != agg_new:
    problems.append("a compiled or linked input differs from the reference map: %s %s" % (compiled_changed, compiled_missing))


def git(*args):
    return subprocess.run(["git", "-C", repo, *args], capture_output=True, check=True).stdout.decode("utf-8", "replace")


# the Git view: nothing outside the allowed set, the contract, the records and the evidence packets differs from the code commit 90e9547ee (the committed metadata checkpoints sit on top of it)
since = sorted(set(git("diff", "--name-only", CODE).splitlines()) | set(git("ls-files", "--others", "--exclude-standard").splitlines()))
ALLOWED_GIT = set(ALLOWED_MAP_CHANGES) | {"eech-campaign/docs/d3-sector-height-production-contract.md", "STATUS.md", "docs/handover/ACTIVE-EXTRACTION.json"}
PACKETS = ("eech-campaign/reference/d3-entity-collision/",)
rows = []
for p in since:
    if p in ALLOWED_GIT:
        rows.append({"path": p, "class": ALLOWED_MAP_CHANGES.get(p, "contract prose or records")})
    elif p.startswith(PACKETS):
        rows.append({"path": p, "class": "retained evidence packet (not an input of any check)"})
    else:
        rows.append({"path": p, "class": "UNEXPECTED CHANGE SINCE 90e9547ee"})
        problems.append("changed since the code commit outside the allowed set: " + p)
for p in since:
    if p.startswith(COMPILED) or p.endswith((".c", ".h", "Cargo.toml", "Cargo.lock", "build.rs")) and not p.startswith("eech-campaign/reference/"):
        problems.append("a compiled-input path differs from 90e9547ee in Git: " + p)
result = {
    "kind": "explicit compiled-input equivalence and change join of the consumer-reference correction against the 90e9547ee code commit (loader-fix metadata) and its final POST map",
    "code_commit": CODE, "metadata_commit": METADATA,
    "map_rows_changed": changed, "map_rows_added": added, "map_rows_removed": removed, "map_rows_identical": sum(1 for k in new if k in old and new[k]["raw_sha256"] == old[k]["raw_sha256"]),
    "compiled_input_rows": len(compiled), "compiled_inputs_aggregate_sha256_e42bb": agg_old, "compiled_inputs_aggregate_sha256_now": agg_new, "compiled_inputs_identical": agg_old == agg_new and not compiled_changed and not compiled_missing,
    "changed_in_git_since_code_commit": rows,
    "allowed_map_changes": ALLOWED_MAP_CHANGES,
    "conclusion": "every compiled and linked input is raw-identical to the 90e9547ee/loader-fix map; only the host Lua driver, its scripted test and prose/records/evidence moved. A compiled binary of the exact loader-fix is reusable for this source ONLY through this equivalence and a separately Git/blob/SHA-bound corrected external Lua; nothing is relabelled and nothing is claimed natively" if not problems else "NOT EQUIVALENT: see problems",
    "problems": problems,
}
open(out, "w", encoding="utf-8", newline="\n").write(json.dumps(result, indent=2) + "\n")
print(json.dumps({"changed": changed, "added": added, "removed": removed, "compiled_rows": len(compiled), "identical": result["compiled_inputs_identical"], "paths": len(rows), "problems": problems}, indent=1))
sys.exit(1 if problems else 0)
