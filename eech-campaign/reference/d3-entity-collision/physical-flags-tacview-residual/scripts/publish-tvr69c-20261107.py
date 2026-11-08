"""Publishes the README of the Tacview-guard RESIDUAL packet and checks, against the actual Git index blob bytes, every literal file of the packet and every changed source file, and that the manifest is the EXACT
Git-tracked packet set minus itself. Run AFTER the freeze (which wrote the packet files and the freeze receipt) and BEFORE the commit. Derived from publish-tvr459-20261107.py (family token replaced by
derive-tvr69c-helpers-20261108.py) and then rewritten by hand for this batch.
Usage: publish.py <repo-root> [dry] (dry: a throw-away -DRY packet, no source file is staged or compared)"""
import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1])
DRY = len(sys.argv) > 2 and sys.argv[2] == "dry"
PACK = REPO / ("eech-campaign/reference/d3-entity-collision/physical-flags-tacview-residual" + ("-DRY" if DRY else ""))
REL = str(PACK.relative_to(REPO)).replace("\\", "/")
FREEZE = "D3-TACVIEW-GUARD-RESIDUAL-FROZEN-20261108.json"
freeze = json.loads((PACK / FREEZE).read_text(encoding="utf-8"))
problems = list(freeze["hygiene_problems_at_freeze"])
final = {k: {"exit": v["actual_exit"], "tests_passed": v["tests_passed"], "selected": v["selected_counts"]} for k, v in freeze["final_runs"].items()}
retained = {r["label"]: {"exit": r["actual_exit"], "why": r.get("why")} for r in freeze["retained_development_runs"]}
readme = f"""# Tacview guard: the residual of ENV-TACVIEW-DORMANCY-GUARD-ENCLOSURE-1 (exact supported positive guard shapes; UNREVIEWED; native UNRUN)

Packet of ONE tiny batch answering the SAME independent review `independent-tacview-correction-bccd-source-20261108.json` (SHA-256 873c09a44fe8bba0f5daff7c10c7db6c0d46a0ef0c4ffcc4bc8535430c2e1240, literal here). That review CLOSED_SOURCE `ENV-TACVIEW-DISPOSITION-IDENTITY-SCOPE-1` (not reopened here; the earlier packets `physical-flags-tacview-disposition/` and `physical-flags-tacview-correction/`, the raw extraction, the freezes and the erratum are preserved literally) and left ONE residual open: `ENV-TACVIEW-DORMANCY-GUARD-ENCLOSURE-1`. Current supported-headless dormancy is sound and this is a guard-only defect correction, not a product defect: **no production source, Rust algorithm, build or Cargo file changed, and nothing ran natively.** This packet is not a review or acceptance; the finding stays OPEN in the card until the SAME review closes it.

* source before: `{freeze["source_before"]}` (the launch checkpoint `{freeze["checkpoint"]}`). The frozen receipt is `{FREEZE}`; `manifest.json` lists every committed file with its SHA-256; `literal-bytes-check.json` compares the actual Git blobs.
* **the residual:** `inside_logging_block` selected a block by any `if (` line containing `tacview_is_logging()`, so `if (!tacview_is_logging() && ...)` and `if (other || tacview_is_logging())` (both of which can run while the log file is closed) counted as logging guards. It now selects a block ONLY when the condition line (whitespace runs collapsed, a trailing `{{` ignored) is exactly one of the three positive conjunctive shapes of the current sources, listed as `LOGGING_GUARDS`: `if (tacview_is_logging() && raw->ac.mob.alive)`, `if (tacview_is_logging() && raw->vh.mob.alive)`, `if (tacview_is_logging() && tacview_reset_frame())`; every other condition fails closed. No Boolean parser or framework. The source was read before the whitelist: all nine real conditions in `fw_updt.c`, `hc_updt.c`, `aa_updt.c`, `rv_updt.c` and `up_update.c` are exactly those shapes. The macro pin, the brace relation, the nine-read and reset census, the original and hosted reset membership and the earlier enclosure controls are unchanged.
* **same-predicate controls:** the synthetic positives now use the real shapes (the old synthetic `raw->alive` is not one of them; the vehicle shape was added) and two single-fault negatives call the SAME predicate the real checks call: a read inside a block whose condition is `if (!tacview_is_logging() && raw->ac.mob.alive)`, and inside `if (other || tacview_is_logging())`.
* **RED, then GREEN:** with the new negatives and real-shape positives added and the predicate still the previous one, the guard failed at the intended assertion (`r1-red-negated-or`, exit 101: `a read inside a block whose condition NEGATES the logging macro`) with the real shapes and the real census still passing; the whitelist then made the same test GREEN (`g1-green-whitelist`) and again inside the final set. A guard-only defect correction: no product RED.
* the contract's guard description (section 13) now says the exact supported positive guard shapes select the block and names the negated and OR negatives; the earlier errata are untouched.
* final checks (direct non-piped records in `runs/`, selected counts parsed from the logs after they completed, nothing pinned in advance): {json.dumps(final)}. The final set is only formatting, the strict lint of the crate whose test changed and that guard.
* retained developer runs with their reasons: {json.dumps(retained)}.
* PRE and POST full input maps and extra identities are equal (`tvr69c-inputmaps-compare-20261107.json`); the three inherited uncompiled Lua files keep their CRLF/Git-LF qualification exactly (receipt).
* process: the family `tvr69c` was verified unused before any helper or output; helpers were derived from the tvr459 ones by `derive-tvr69c-helpers-20261108.py` (exclusive creation, ONE family-token replacement; a first derivation whose outputs carried a date stem that disagreed with their own internal references was set aside unchanged as `*.attempt1` files and redone, before any use) and edited by hand; source with the Write and Edit tools; every final fmt and clippy invocation carries its own `rustup component add` and version command. The literal Root receipts here are the source binding (`tacview-correction-bccd-Root-binding-20261108.json`, 25f8) and the publication identity (`tacview-correction-69c0-Root-publication-20261108.json`, 28f6); no `Root-tacview-residual-feedback*.json` existed at any listing of this batch. Root's audits are batched Git joins and are not repeated here.

## Limits

Source-reachability disposition for the SUPPORTED headless configuration only; no run observed `tacview_log_file` staying NULL; custom hosts, configurations, network or device implementations and DCS-needed uses are not covered. `ENV-TACVIEW-DORMANCY-GUARD-ENCLOSURE-1` stays OPEN until the SAME review; `ENV-PHYSICAL-FLAGS-HOST-STORE-NATIVE-ACCEPTANCE-1`, the other consumers, the READ-dependent native steps, lifetime, DCS and all ten D3 requirements stay OPEN; no native, boot or lifetime proof and no build of product source.
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
    handle.write((json.dumps({"files": files, "scope": "Residual packet of the Tacview guard enclosure finding only (exact supported positive guard shapes); not a native or Windows result, review, approval or acceptance.", "limit": "no production source changed"}, indent=2) + "\n").encode("utf-8"))
subprocess.run(["git", "-C", str(REPO), "add", "-A", REL], check=True)
# the manifest is the EXACT Git-tracked packet set minus itself (derived, never a guessed count)
tracked = sorted(p[len(REL) + 1:] for p in subprocess.run(["git", "-C", str(REPO), "ls-files", "--", REL], capture_output=True, check=True, text=True).stdout.split())
if tracked != sorted([f["path"] for f in files] + ["manifest.json"]):
    problems.append("the manifest plus itself is not the exact Git-tracked packet set")
print(json.dumps({"problems": problems, "manifest_entries": len(files), "git_tracked": len(tracked)}))
sys.exit(1 if problems else 0)
