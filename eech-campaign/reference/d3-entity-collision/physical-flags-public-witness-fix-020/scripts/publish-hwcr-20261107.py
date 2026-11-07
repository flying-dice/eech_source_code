"""Publishes the README of the public-witness packet and checks, against the actual Git index blob bytes, every literal file of the packet and every changed source file. Run AFTER the freeze (which wrote the packet files, the
freeze receipt and the plan) and BEFORE the code commit. Usage: publish.py <repo-root> [dry] (dry: a throw-away -DRY packet, no source file is staged or compared)"""
import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1])
DRY = len(sys.argv) > 2 and sys.argv[2] == "dry"
PACK = REPO / ("eech-campaign/reference/d3-entity-collision/physical-flags-public-witness-fix-020" + ("-DRY" if DRY else ""))
REL = str(PACK.relative_to(REPO)).replace("\\", "/")
FREEZE = "D3-PHYSICAL-FLAGS-PUBLIC-WITNESS-FIX-CODE-FROZEN-20261107.json"
freeze = json.loads((PACK / FREEZE).read_text(encoding="utf-8"))
problems = list(freeze["hygiene_problems_at_freeze"])
final = {k: {"exit": v["actual_exit"], "tests_passed": v["tests_passed"], "selected": v["selected_counts"]} for k, v in freeze["final_runs"].items()}
plan = json.loads((PACK / "physical-flags-public-witness-plan.json").read_text(encoding="utf-8"))
readme = f"""# Physical flag store, public witness corrections: the two P2s of the V2 review of f4d62c9b4 (candidate; UNREVIEWED; native UNRUN)

Packet of the local checks of ONE correction batch: `ENV-PHYSICAL-FLAGS-WITNESS-REFERENCE-COMPLETE-1` and `ENV-PHYSICAL-FLAGS-WITNESS-REFUSAL-CAUSAL-1`, corrected together in the TEST-only host-owned Lua driver (`eech-campaign/crates/eech-world/lua/physical-flags-witness.lua`), its scripted-engine tests over the REAL reference provider and flag store (`tests/physical_flags_witness_driver.rs`, 25 tests) and the contract's section 10. **No product C or Rust changed and no Windows build was made: the sealed 020 binaries are the plan's binaries and the corrected driver is a NEW, separately bound external Lua. It is evidence of one candidate, not a review, approval, native result or acceptance; both findings stay OPEN until the reviewer closes them.**

* source before: `{freeze["source_before"]}` (metadata checkpoint `{freeze["checkpoint"]}`, which published the V2 review and Root's bindings); the parent candidate is `f4d62c9b4`. The frozen receipt is `{FREEZE}`; `manifest.json` lists every file with its SHA-256; `literal-bytes-check.json` compares the actual Git blobs.
* the corrected driver literal: SHA-256 `{plan["driver_literal"]["sha256"]}`, Git blob `{plan["driver_literal"]["git_blob_of_raw_bytes"]}` (the parent's was `{plan["driver_literal"]["parent_driver_sha256"]}`); `physical-flags-public-witness-plan.json` is the same 23-step finite native plan and setup with the corrected driver and two new process rules.
* `ENV-PHYSICAL-FLAGS-WITNESS-REFERENCE-COMPLETE-1`: every reference (the Original report, the paired control) is validated COMPLETE before the output, the preparation, the boot, any provider call or any alteration: the exact schema 1 records in their order, canonical numbers and the healthy count, oracle, first-event and consume relationships; Original must show zero of ALL traffic; the group/set/reset-only control is kept and unlocks only the cases its positive reached. The tests generate a REAL Original and paired control first, then damage one field or required record at a time (17 Original and 29 control cases) and require a refusal naming that field before any effect.
* `ENV-PHYSICAL-FLAGS-WITNESS-REFUSAL-CAUSAL-1`: the refusal transaction is latched when the eligible fault is injected; any later host callback, a normal return, a fatal of another entry or another marker ends the credit, even when a later fatal carries the same marker; the engine's boot is captured as an entry (first-event frame 0). The tests run, for each of the 20 refusal modes, the genuine direct refusal first and then an engine that IGNORES the fault and (a) calls another callback and raises the same marker, (b) returns normally, (c) raises another marker (all FAIL), and the same at boot.
* the genuine parent RED is retained: `d1-red-parent-f4d-new-tests` (the new tests against the unchanged f4d62c9b4 driver: 5 of 25 failed, the damaged references were accepted and an ignored fault was credited, a boot refusal was not captured); the GREEN is `d5-green-attempt4` and the final set `f1` to `f5` (all exit 0, direct non-piped records in `runs/`, selected counts read from the logs): {json.dumps(final)}.
* retained development runs (`runs/`, with their reasons in the receipt): the RED, three green attempts that named test-expectation differences only (messages and an inserted duplicate row placed after the consume row), formatting, clippy and the whole suite.
* PRE and POST full input maps and the extra identities are equal (`hwcr-inputmaps-compare-20261107.json`); the product's compiled inputs are unchanged from the 020 sources (the freeze refuses a changed file outside Lua and tests).
* Precisions kept: the changed-answer proof is the carried UPDATED fact in the next candidate question (one of the 44 payload words changes), not a changed collision outcome; phase counters alone do not establish coverage of each UPDATED, MOVED or ROTATED array or of any consumer; the test bridge and fixture remain a dedicated process, not whole-global or FPU restoration.

## What the tests ran (and what none ran)

The REAL driver, unmodified, under a real Lua 5.1 state, the REAL exclusive-output host function and the REAL reference arithmetic profile, with ONE scripted collaborator standing in for the campaign DLL (boot and frames, optionally ignoring a refused answer) over the REAL provider and `FlagStore`.

**Not run:** the host-owned Lua -> native `eech_dc` module -> host campaign path (the plan is for Root's native run), any damaged-reference or bad-input native run, Windows, retail data, any full pack, the other consumers of the flag store, a replacement store, the cost of a provider call per entity per iteration, whole-FPU preservation, any live-DCS claim.
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
(PACK / "manifest.json").write_text(json.dumps({"files": files, "scope": "Source and local-check packet of the public flag-store witness corrections only; not a native result, review, approval or acceptance."}, indent=2) + "\n", encoding="utf-8", newline="\n")
subprocess.run(["git", "-C", str(REPO), "add", "-A", REL], check=True)
print(json.dumps({"problems": problems, "files": len(files)}))
sys.exit(1 if problems else 0)
