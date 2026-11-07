"""Publishes the README of the public-witness packet and checks, against the actual Git index blob bytes, every literal file of the packet and every changed source file. Run AFTER the freeze (which wrote the packet files, the
freeze receipt and the plan) and BEFORE the code commit. Usage: publish.py <repo-root> [dry] (dry: a throw-away -DRY packet, no source file is staged or compared)"""
import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1])
DRY = len(sys.argv) > 2 and sys.argv[2] == "dry"
PACK = REPO / ("eech-campaign/reference/d3-entity-collision/physical-flags-public-witness-020" + ("-DRY" if DRY else ""))
REL = str(PACK.relative_to(REPO)).replace("\\", "/")
FREEZE = "D3-PHYSICAL-FLAGS-PUBLIC-WITNESS-CODE-FROZEN-20261107.json"
freeze = json.loads((PACK / FREEZE).read_text(encoding="utf-8"))
problems = list(freeze["hygiene_problems_at_freeze"])
final = {k: {"exit": v["actual_exit"], "tests_passed": v["tests_passed"], "selected": v["selected_counts"]} for k, v in freeze["final_runs"].items()}
plan = json.loads((PACK / "physical-flags-public-witness-plan.json").read_text(encoding="utf-8"))
readme = f"""# Physical flag store: the public host-owned Lua witness driver, its scripted-engine tests and the finite native plan (candidate; UNREVIEWED; native UNRUN)

Packet of the local checks of ONE candidate: a TEST-only host-owned Lua driver (`eech-campaign/crates/eech-world/lua/physical-flags-witness.lua`), its 19 scripted-engine tests over the REAL reference provider and flag store
(`tests/physical_flags_witness_driver.rs`), one corrected helper block shared with `entity-authority.lua`, one pinned script count (`host_callbacks.rs`, 64 to 65), and the contract's section 10. **No product C or Rust changed: the 020
ordinary and Original binaries are the plan's binaries. It is evidence of one candidate, not a review, approval, native result or acceptance.**

* source before: `{freeze["source_before"]}` (metadata checkpoint `{freeze["checkpoint"]}`); binaries of the plan: the sealed 020 build of `{freeze["code_commit_of_the_binaries"]}` (Root's build and profile receipts are kept literally here). The frozen receipt is `{FREEZE}`; `manifest.json` lists every file with its SHA-256; `literal-bytes-check.json` compares the actual Git blobs.
* the driver literal: SHA-256 `{plan["driver_literal"]["sha256"]}`, Git blob `{plan["driver_literal"]["git_blob_of_raw_bytes"]}`; `physical-flags-public-witness-plan.json` is the finite executable native plan: {len(plan["steps"])} steps (the Original characterisation, the paired hosted control, the changed answer and the 20 single-fault refusals), the established finite default (lebanon_retail, hours 0.001, 35 frames of 100 ms, seed 1), the binary roles and hashes from Root's build receipt, and what is not claimed.
* a defect found and corrected on the way (test-only code): the lane-mask and whole-number helpers of `entity-authority.lua` used `2 ^ n`, one unit low under the reference round-toward-zero arithmetic the drivers select (the mask of lane 1 printed `0x00000001`). Reproduced in the retained red runs `d1`, `d3`, `d4`, `d5` (48 wrong values across the two drivers' copies) and green from `d6`; the staged copy in the 020 payload keeps the old text. Inferred, not observed natively: the old public driver's flag reply check would refuse genuine host masks under that profile.
* final checks `w1` to `w5` (all exit 0, direct non-piped records in `runs/`, selected counts read from the logs): {json.dumps(final)}.
* retained development runs (`runs/`, with their reasons in the receipt): the red runs, one scratch diagnostic that panics on purpose, the formatting run that named only the new test file, the whole-suite run that found the pinned script count. None is a capability RED except the helper runs.
* PRE and POST full input maps and the extra identities are equal (`hwwit-inputmaps-compare-20261107.json`); the product's compiled inputs are unchanged from the 020 sources (the freeze refuses a changed file outside Lua and tests).
* Root's supervision note of 20:12 and Root's literal records this batch binds (cache release, build, profile and snapshot receipts, the default-plan review and actuals, the publication checkpoint, the learnings) are kept literally; the freeze refused any supervision note not read.

## Exact path of what the tests ran (and what none ran)

1. The REAL driver, unmodified, under a real Lua 5.1 state, the REAL exclusive-output host function and the REAL reference arithmetic profile, with ONE scripted collaborator standing in for the campaign DLL (a boot and a frame loop that makes the calls the real campaign makes in the order its source makes them and checks every reply as the C bridge and the module's validator do) over the REAL provider and `FlagStore`: attempts, answered and accepted counted apart, the backing oracle against the real store, the paired-control rules, every single fault at its marker, one valid changed answer carried into the next candidate question, every refusal before any boot.
2. The pattern helpers of both drivers under the real round-toward-zero profile for every lane and every whole number they can name.

**Not run:** the host-owned Lua -> native `eech_dc` module -> host campaign path (the plan is for Root's native run), Windows, retail data, Tacview, any full pack, the other consumers of the flag store, a replacement store, the cost of a provider call per entity per iteration, whole-FPU preservation, any live-DCS claim.
"""
(PACK / "README.md").write_text(readme, encoding="utf-8", newline="\n")
subprocess.run(["git", "-C", str(REPO), "add", "-A", REL], check=True)
for row in ([] if DRY else freeze["changed_files"]):
    subprocess.run(["git", "-C", str(REPO), "add", row["path"]], check=True)
literal = {}
names = sorted(str(p.relative_to(PACK)).replace("\\", "/") for p in PACK.rglob("*") if p.is_file() and p.name not in ("manifest.json", "literal-bytes-check.json"))
for name in names:
    blob = subprocess.run(["git", "-C", str(REPO), "cat-file", "blob", ":" + REL + "/" + name], capture_output=True, check=True).stdout
    work = (PACK / name).read_bytes()
    literal[name] = {"git_index_blob_bytes_equal_working_bytes": blob == work, "bytes": len(work), "sha256": hashlib.sha256(work).hexdigest()}
    if blob != work:
        problems.append("not literal in Git: " + name)
for row in ([] if DRY else freeze["changed_files"]):
    blob = subprocess.run(["git", "-C", str(REPO), "cat-file", "blob", ":" + row["path"]], capture_output=True, check=True).stdout
    work = (REPO / row["path"]).read_bytes()
    literal[row["path"]] = {"git_index_blob_bytes_equal_working_bytes": blob == work, "bytes": len(work), "sha256": hashlib.sha256(work).hexdigest(), "frozen_sha256_equal": hashlib.sha256(work).hexdigest() == row["sha256"]}
    if blob != work or hashlib.sha256(work).hexdigest() != row["sha256"]:
        problems.append("changed source not literal or not the frozen bytes: " + row["path"])
(PACK / "literal-bytes-check.json").write_text(json.dumps({"kind": "actual Git index blob bytes (git cat-file) against the working bytes of every packet file except this record and the manifest, and of every changed source file", "files": literal, "problems": problems}, indent=2) + "\n", encoding="utf-8", newline="\n")
files = [{"path": str(p.relative_to(PACK)).replace("\\", "/"), "bytes": p.stat().st_size, "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(PACK.rglob("*")) if p.is_file() and p.name != "manifest.json"]
(PACK / "manifest.json").write_text(json.dumps({"files": files, "scope": "Source and local-check packet of the public flag-store witness candidate only; not a native result, review, approval or acceptance."}, indent=2) + "\n", encoding="utf-8", newline="\n")
subprocess.run(["git", "-C", str(REPO), "add", "-A", REL], check=True)
print(json.dumps({"problems": problems, "files": len(files)}))
sys.exit(1 if problems else 0)
