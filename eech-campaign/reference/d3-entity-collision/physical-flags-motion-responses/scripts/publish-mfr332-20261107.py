"""Publishes the README of the motion-response flag group packet and checks, against the actual Git index blob bytes, every literal file of the packet and every changed source file, and that the manifest is the EXACT
Git-tracked packet set minus itself. Run AFTER the freeze (which wrote the packet files and the freeze receipt) and BEFORE the commit. Derived from publish-tvr69c-20261107.py (family token replaced by
derive-mfr332-helpers-20261108.py) and then rewritten by hand for this batch. The README is an f-string: every literal brace in its text is doubled.
Usage: publish.py <repo-root> [dry] (dry: a throw-away -DRY packet, no source file is staged or compared)"""
import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1])
DRY = len(sys.argv) > 2 and sys.argv[2] == "dry"
PACK = REPO / ("eech-campaign/reference/d3-entity-collision/physical-flags-motion-responses" + ("-DRY" if DRY else ""))
REL = str(PACK.relative_to(REPO)).replace("\\", "/")
FREEZE = "D3-MOTION-RESPONSE-FLAGS-FROZEN-20261108.json"
freeze = json.loads((PACK / FREEZE).read_text(encoding="utf-8"))
problems = list(freeze["hygiene_problems_at_freeze"])
final = {k: {"exit": v["actual_exit"], "tests_passed": v["tests_passed"], "selected": v["selected_counts"], "hosted_traffic": v.get("hosted_traffic_groups_sets_reads_resets_fresh")} for k, v in freeze["final_runs"].items()}
retained = {r["label"]: {"exit": r["actual_exit"], "why": r.get("why")} for r in freeze["retained_development_runs"]}
readme = f"""# The motion-response flag group: roles from source and the one live join characterised (UNREVIEWED; native UNRUN)

Packet of ONE bounded batch: the next named required finite group of `ENV-PHYSICAL-FLAGS-HOST-STORE-NATIVE-ACCEPTANCE-1`, the helicopter, vehicle, ship and weapon-vector motion responses and the host-owned MOVED / ROTATED (and UPDATED) words. The first step was to establish their actual role (read or write, live or dormant) from the real callers and bodies, not from a label; only the one genuinely missing join was then characterised. **It is not a review, approval, native result or acceptance; no policy, normalisation or algorithm changed, and nothing ran natively.**

* source before: `{freeze["source_before"]}` (the launch checkpoint `{freeze["checkpoint"]}`). The frozen receipt is `{FREEZE}`; `manifest.json` lists every committed file with its SHA-256; `literal-bytes-check.json` compares the actual Git blobs.
* **roles (contract section 14, pinned by `physical_flags_guard.rs`):** no motion response READS any of the three words (the readers are the already characterised or disposed ones). The ship and the weapon-vector name none of them. The host-movement responses only SET: `rv_hostmv.c` has exactly six set statements, each the first statement of the block its condition opens: two LIVE (the AI helicopter's `campaign_movement_helicopter_guided_begin`, `if (result.moved)` sets MOVED, and `_guided_end`, `if (result.rotated)` sets ROTATED, called from `hc_move.c`), two DEAD (the same statements in `campaign_movement_helicopter_begin` and `_end`, which nothing calls) and two behind the per-entity Tacview logging bit (`campaign_movement_move` and `campaign_movement_fixed_wing_move`, `if (raw->..mob.tacview_logging && result.rotated)`, a bit only the Tacview writer sets). Their other `result.moved` / `result.rotated` uses are ordinary response booleans validated 0 or 1 by the answer check, not array accesses.
* **the join, characterised in BOTH profiles with the same scenarios** (`tests/physical_flags_motion.rs`, feature `physical-flags-motion-check`, world `csrc/eech_physical_flags_motion_check.c`, which declares every hand-made part): the REAL guided begin and end bodies, the real mobile/aircraft/helicopter functions (the real setter and getter, the real position setter), the campaign's roster link and, hosted, the real store of eech-world; the one scripted input is the host's physical answer. A moved/rotated answer sets exactly the helicopter's lane of its own word and no other; not-moved/not-rotated applies the pose and sets nothing; one cycle makes two set operations and no read reaches the host; lane 31 is bit 31 and the real getter returns the negative raw mask; two helicopters of one word leave both bits and the first's real read destroys the other's; a response ORs its lane and leaves a bit another lane's setter put there; a malformed answer is refused by name before anything is applied or set; an unlinked helicopter is refused before the host is asked. The first developer compile failed (the scripted attitude indexed a flat array as a matrix); after that fixture correction it passed in both profiles: a characterisation of unchanged behaviour (no red is manufactured for it); development runs are retained below.
* **qualified source delta (honest):** the gate of the two test-only key helpers in the product C `aphavoc/source/entity/mobile/mb_hostmodel.c` now also names the new define (a changed raw product-source file; without the feature the condition evaluates as before: a preprocessor fact the guard pins, not a claim that any compiled raw row equals the sealed 3abf's); a feature-gated C fixture; the feature's manifest entry and test entry; the build define; the guard; the contract. No Windows or native rebuild was made: the retained 3abf binaries are not this source and are not relabelled.
* final checks (direct non-piped records in `runs/`, selected counts and host traffic parsed from the logs after they completed, nothing pinned in advance): {json.dumps(final)}.
* retained developer runs with their reasons: {json.dumps(retained)}.
* PRE and POST full input maps and extra identities are equal (`mfr332-inputmaps-compare-20261107.json`); the three inherited uncompiled Lua files keep their CRLF/Git-LF qualification exactly (receipt).
* process: the family `mfr332` was verified unused before any helper or output; helpers were derived from the tvr69c ones by `derive-mfr332-helpers-20261108.py` (exclusive creation, ONE family-token replacement, file stems kept consistent with the scripts' own references) and edited by hand; the preserved source was resumed by the explicitly announced Codex fallback after the Claude OAuth failure (actual exit 1, not quota; both literal receipts included), with no authentication change; every final fmt and clippy invocation carries its own `rustup component add` and version command. Root's notes 0038, 0044 and 0052 are read in full and literal here with the four receipts they name (the live-services readback, the ledger v190 preservation, the publication identity of 332955 and its exact-commit CI readback).

## Limits

The vehicle and fixed-wing sets are pinned, not run; the order of the flag set against the position write inside the end step is not contractual and not observable here; the Rust boundary of the movement host and the host's answers are covered by `eech-world`'s own tests and the accepted D1/D2 movement evidence, reused at their own identities and not re-run; whether and how often native runs set these words is not observed (nothing reads them, so a set shows only as store traffic); the four hosted reads are explicit fixture inspections, not reads by a motion consumer; resets are zero because no reset case runs here (no broader reset proof); a lifetime of the host's store, whole-global or FPU restoration and any native, boot or Windows run are not shown. `ENV-PHYSICAL-FLAGS-HOST-STORE-NATIVE-ACCEPTANCE-1`, the READ-dependent native steps, lifetime, DCS and all ten D3 requirements stay OPEN.
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
    handle.write((json.dumps({"files": files, "scope": "Packet of the motion-response flag group only (roles from source, the helicopter guided-step join in both profiles); not a native or Windows result, review, approval or acceptance.", "limit": "no policy, normalisation or algorithm change; a feature-gated fixture and a test-only gate"}, indent=2) + "\n").encode("utf-8"))
subprocess.run(["git", "-C", str(REPO), "add", "-A", REL], check=True)
# the manifest is the EXACT Git-tracked packet set minus itself (derived, never a guessed count)
tracked = sorted(p[len(REL) + 1:] for p in subprocess.run(["git", "-C", str(REPO), "ls-files", "--", REL], capture_output=True, check=True, text=True).stdout.split())
if tracked != sorted([f["path"] for f in files] + ["manifest.json"]):
    problems.append("the manifest plus itself is not the exact Git-tracked packet set")
print(json.dumps({"problems": problems, "manifest_entries": len(files), "git_tracked": len(tracked)}))
sys.exit(1 if problems else 0)
