"""Publishes the README of the supply-descriptor BOOLEAN-CORRECTION packet and checks, against the actual Git index blob bytes, every literal file of the packet and every changed source file, and that the manifest is the EXACT
Git-tracked packet set minus itself. Run AFTER the freeze (which wrote the packet files and the freeze receipt) and BEFORE the code commit. Derived from publish-sdc5a-20261107.py (family token replaced by derive-sb120-helpers-20261107.py) and
then rewritten by hand for this batch.
Usage: publish.py <repo-root> [dry] (dry: a throw-away -DRY packet, no source file is staged or compared)"""
import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1])
DRY = len(sys.argv) > 2 and sys.argv[2] == "dry"
PACK = REPO / ("eech-campaign/reference/d3-entity-collision/physical-flags-supply-bool-fix" + ("-DRY" if DRY else ""))
REL = str(PACK.relative_to(REPO)).replace("\\", "/")
FREEZE = "D3-SUPPLY-DESCRIPTOR-BOOLEAN-CODE-FROZEN-20261107.json"
freeze = json.loads((PACK / FREEZE).read_text(encoding="utf-8"))
problems = list(freeze["hygiene_problems_at_freeze"])
final = {k: {"exit": v["actual_exit"], "tests_passed": v["tests_passed"], "selected": v["selected_counts"], "descriptors_delivered": v.get("descriptors_delivered"), "hosted_traffic": v.get("hosted_traffic_groups_sets_reads_resets_fresh")} for k, v in freeze["final_runs"].items()}
retained = {r["label"]: {"exit": r["actual_exit"], "why": r.get("why")} for r in freeze["retained_development_runs"]}
readme = f"""# Supply arrival descriptor: the boolean correction of `descriptor.updated` (an intentional defect correction; UNREVIEWED; native, Windows and default recordings UNRUN)

Packet of the local checks of ONE bounded batch: the single assignment of `descriptor.updated` in `special/guide/gd_supply_arrival.c` (the shipping `campaign_supply_arrival_check`, fixed-wing branch) now converts the one destructive getter value with the file's own `? TRUE : FALSE` convention, as the technical lead decided (`Root-supply-updated-boolean-disposition-20261107.json`, literal here; the SAME review of the characterisation, `independent-physical-flags-supply-ae367-source-20261107.json`, SHA-256 867a7a2add73..., findings empty, is literal here too). **It is evidence of one candidate, not a review, approval, native result or acceptance, and it is an INTENTIONAL CORRECTION OF AN EXISTING DEFECT, not a pure refactor.**

* source before: `{freeze["source_before"]}` (the launch checkpoint `{freeze["checkpoint"]}`); the characterised producer is `ae367ca3a` and its measured refusal evidence (packet `physical-flags-supply-consumer/`) is retained unchanged. The frozen receipt is `{FREEZE}`; `manifest.json` lists every committed file of the packet with its SHA-256; `literal-bytes-check.json` compares the actual Git blobs.
* **the exact product-source delta (declared):** `aphavoc/source/entity/special/guide/gd_supply_arrival.c`, one line: `descriptor.updated = get_local_entity_int_value (member, INT_TYPE_UPDATED);` became `descriptor.updated = get_local_entity_int_value (member, INT_TYPE_UPDATED) ? TRUE : FALSE;`. The getter is still called exactly once per call, before any gate and before the host is asked. Zero becomes FALSE and every nonzero raw mask (2, 4, ..., 2^30, and the negative bit 31 as INT_MIN) becomes TRUE. Nothing else of the product source changed: not the getter, the host store, the other consumers (the guide branch, the collision pass-through), the destructive neighbour read, the quarter reset extent, update order, RNG, FPU or campaign policy, and the Rust boundary's `checked_bool` is NOT weakened (the source guard pins it as accepting exactly 0 and 1). The changed files are exactly the three of the receipt (this product file, `physical_flags_supply.rs` and `physical_flags_guard.rs`) plus the contract's sections 3, 11 and the new section 12. **No claim is made that the compiled raw product sources equal the sealed 020's: this product C really changes and true new binaries are Root's later step.**
* **RED, then GREEN, with the existing real control:** the expectations of `eech-engine/tests/physical_flags_supply.rs` were changed FIRST (lanes 1, 2, 30 and 31 must reach the host's answer with `updated: true` and, hosted, one destructive host read; the getter's raw word must still be the lane's mask; both reads of the neighbour pair are now accepted; the descriptor count of the scenarios becomes 14) and run against the UNCHANGED producer: both profiles failed at lane 1 with the old refusal `invalid updated flag 2` (a real behavioural red, not a setup or compile failure; `runs/r1-red-supply-hosted.*`, `runs/r2-red-supply-original.*`). After the one assignment, the same test passed in both profiles (`runs/g1-green-supply-hosted.*`, `runs/g2-green-supply-original.*`, retained developer runs) and again inside the final set below.
* final checks `s1` to `s11` (direct non-piped records in `runs/`; selected counts and the control's own facts parsed from the logs AFTER they completed, nothing pinned in advance): {json.dumps(final)}.
* retained developer runs with their reasons: {json.dumps(retained)}.
* PRE and POST full input maps and extra identities are equal (`sb120-inputmaps-compare-20261107.json`); the three inherited uncompiled Lua files keep their CRLF/Git-LF qualification exactly (receipt). The accepted 020 binaries, the finite default and the finite witness family were not replayed; no Windows, native or full-pack run was made.
* process: the family name `sb120` was verified unused before any helper or output (`sb120-name-check` with its negative control); every helper was derived by an explicit script (`derive-sb120-helpers-20261107.py`, exclusive creation, one family-token replacement, output inspected) and then edited by hand with the Edit tool (new source with the Write and Edit tools, not shell string replacement); every final fmt and clippy invocation carries its own `rustup component add` and version command; Root's notes of this batch (2242, 2243: live readbacks and the CI clarification) are read in full and published as literals with the small safe supplemental receipts they name.

## What the control ran (and what it did not)

Real: the producer and its roster identity, the mobile/aircraft/fixed-wing getters, the guide criteria, the real list functions, the real update reset, the real Rust boundary, and in the hosted profile the real flag store. Hand-made and declared in the C file of the characterisation (unchanged): the entity pool and raw records, the waypoint's parent pointer in its raw record, a private roster collaborator that issues `eech-world:<n>`, the delta time and comms model.

**Not run / not shown:** whether a native campaign ever reached the old refusal or reaches the corrected path (the native 35-frame default is not evidence of this consumer), the helicopter (its descriptor does not read the word) and the other consumers (Tacview, vehicle, ship, weapon-vector), the shipped `Engine::boot` bridge, any Windows build, native run or default recording of the changed production C, a lifetime or replacement of the host's store, whole-global or FPU restoration. The defect row stays OPEN until the review of this correction and the later runtime proof.
"""
(PACK / "README.md").write_bytes(readme.encode("utf-8")) if not (PACK / "README.md").exists() else sys.exit("README.md already exists")
subprocess.run(["git", "-C", str(REPO), "add", "-A", REL], check=True)
for row in ([] if DRY else freeze["changed_files"]):
    subprocess.run(["git", "-C", str(REPO), "add", row["path"]], check=True)
subprocess.run(["git", "-C", str(REPO), "add", "eech-campaign/docs/d3-physical-flags-contract.md"], check=True) if not DRY else None
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
with open(PACK / "literal-bytes-check.json", "xb") as handle:
    handle.write((json.dumps({"kind": "actual Git index blob bytes (git cat-file) against the working bytes of every packet file except this record and the manifest, and of every changed source file", "files": literal, "problems": problems}, indent=2) + "\n").encode("utf-8"))
files = [{"path": str(p.relative_to(PACK)).replace("\\", "/"), "bytes": p.stat().st_size, "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(PACK.rglob("*")) if p.is_file() and p.name != "manifest.json"]
with open(PACK / "manifest.json", "xb") as handle:
    handle.write((json.dumps({"files": files, "scope": "Source and local-check packet of the supply descriptor boolean correction only; not a native or Windows result, review, approval or acceptance."}, indent=2) + "\n").encode("utf-8"))
subprocess.run(["git", "-C", str(REPO), "add", "-A", REL], check=True)
# the manifest is the EXACT Git-tracked packet set minus itself (derived, never a guessed count)
tracked = sorted(p[len(REL) + 1:] for p in subprocess.run(["git", "-C", str(REPO), "ls-files", "--", REL], capture_output=True, check=True, text=True).stdout.split())
if tracked != sorted([f["path"] for f in files] + ["manifest.json"]):
    problems.append("the manifest plus itself is not the exact Git-tracked packet set")
print(json.dumps({"problems": problems, "manifest_entries": len(files), "git_tracked": len(tracked)}))
sys.exit(1 if problems else 0)
