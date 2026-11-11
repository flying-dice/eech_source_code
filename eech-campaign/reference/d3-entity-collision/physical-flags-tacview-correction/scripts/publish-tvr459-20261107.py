"""Publishes the README of the Tacview-disposition CORRECTION packet and checks, against the actual Git index blob bytes, every literal file of the packet and every changed source file, and that the manifest is the EXACT
Git-tracked packet set minus itself. Run AFTER the freeze (which wrote the packet files and the freeze receipt) and BEFORE the commit. Derived from publish-tvb1-20261107.py (family token replaced by
derive-tvr459-helpers-20261108.py) and then rewritten by hand for this batch.
Usage: publish.py <repo-root> [dry] (dry: a throw-away -DRY packet, no source file is staged or compared)"""
import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1])
DRY = len(sys.argv) > 2 and sys.argv[2] == "dry"
PACK = REPO / ("eech-campaign/reference/d3-entity-collision/physical-flags-tacview-correction" + ("-DRY" if DRY else ""))
REL = str(PACK.relative_to(REPO)).replace("\\", "/")
FREEZE = "D3-TACVIEW-DISPOSITION-CORRECTION-FROZEN-20261107.json"
freeze = json.loads((PACK / FREEZE).read_text(encoding="utf-8"))
problems = list(freeze["hygiene_problems_at_freeze"])
final = {k: {"exit": v["actual_exit"], "tests_passed": v["tests_passed"], "selected": v["selected_counts"]} for k, v in freeze["final_runs"].items()}
retained = {r["label"]: {"exit": r["actual_exit"], "why": r.get("why")} for r in freeze["retained_development_runs"]}
readme = f"""# Tacview disposition: correction of both findings of the SAME review (guard enclosure and identity scope; UNREVIEWED; native UNRUN)

Packet of ONE bounded batch answering the SAME independent review `independent-tacview-disposition-8ffd-source-20261108.json` (SHA-256 9ab3482df6d7164e47255f77d21c22646b79d14c3a72680b944f4a86a5933413, literal here): it supports the CURRENT SUPPORTED-HEADLESS dormancy of the legacy Tacview writer but does NOT accept the pinned disposition until two bounded corrections pass the SAME review. **No production source, Rust algorithm, build or Cargo file changed; nothing new was run natively; the earlier packet `physical-flags-tacview-disposition/` (its raw extraction, receipt and README) is preserved unchanged.** This packet is not a review, approval or acceptance; both findings stay OPEN in the card until the SAME review closes them.

* source before: `{freeze["source_before"]}` (the launch checkpoint `{freeze["checkpoint"]}`). The frozen receipt is `{FREEZE}`; `manifest.json` lists every committed file with its SHA-256; `literal-bytes-check.json` compares the actual Git blobs.
* **ENV-TACVIEW-DORMANCY-GUARD-ENCLOSURE-1** (the guard accepted the NEAREST PRECEDING `if (` line, so a read or reset placed right after the logging block's closing brace passed, and the macro was never pinned): `physical_flags_guard.rs` now (1) pins `tacview.h`'s `tacview_is_logging ()` as exactly `(tacview_log_file != NULL)` through `macro_means_open_log_file`, and (2) requires every one of the nine reads of MOVED and ROTATED and both resets of the two words, the hosted `campaign_model_flag_reset` AND the original profile's `memset`, to be INSIDE THE BRACES of the block a `if (tacview_is_logging () ...)` opens, through `inside_logging_block` (brace matching over comment-stripped text; no parser, no framework). The synthetic negatives call the SAME two functions the real checks call: the same read after the closing brace, a reset after the block, a read in some other conditional, an unconditionally true macro and an inverted macro are rejected; a read inside and a read after a nested block still inside are accepted. The existing census (callers, assignment sites, shipping sources, reader files, read count) is unchanged.
* **RED, then GREEN, on a meaningful miss:** the new negatives and the real checks were first routed through the OLD predicate (`r1-red-old-predicate`, exit 101): it failed at the intended assertion, `NEGATIVE: the same read immediately AFTER the closing brace of the logging block`, with the macro pin and every real read already passing; the brace-aware predicate then made the same test GREEN (`g1-green-brace-aware`) and again inside the final set. The earlier packet's two premise failures (`g1`/`g2`, the open and reopen counts of `tacview.c`) stay labelled test-premise errors, not product findings; this batch read the source around `up_update.c`'s resets before pinning their counts.
* **ENV-TACVIEW-DISPOSITION-IDENTITY-SCOPE-1** (the contract said `git diff 3abf03e51 HEAD` over `aphavoc` and `eech-campaign/crates` is empty; false at `8ffd`/`459653` because the new guard test is in that range): the contract's section 13 now carries an ERRATUM: the retained extraction (SHA-256 4bf168c5..., written 2026-11-07T23:54:15Z, `repo_head` b1b9a5d9ea27cbbbeda33035972e416ee7925001) compared COMMITS between `3abf03e51` and the parent `b1b9a5d9e`; between `3abf03e51` and the delivered commits the ONE changed file in that range is the source-only guard test; every production and build input read by the disposition is unchanged since `3abf03e51`; the retained compiled reports are the 3abf build's own and say nothing about the new test. `tvr459-identity-join-20261107.json` (made by `tvr459-identity-join-20261107.py` before the PRE) shows it: the files differing from `3abf03e51` in both ranges are exactly the guard test, 17 sources the disposition relies on are byte-identical to 3abf's blobs, and the retained reports and archives still hash to the extraction's record. Root's mechanical binding (`tacview-disposition-8ffd-Root-binding-20261108.json`, literal here) agrees (3994 other mapped Git inputs unchanged). The old extraction and old packet are not rewritten.
* **the review's comms-route check, in the contract's prose:** `receive_comms_data` is compiled and the headless frame calls it, but it acts only on an INITIALISED connection; the headless host's `eech_headless_devices.c` supplies a zero-initialised private connection, a failing `direct_play_initialise_system` and a `direct_play_receive_data` that returns 0, so the second route to a user assignment (`comm_man.c`) is closed in the SUPPORTED headless configuration. A custom host, configuration, network or device implementation, a changed boot or a DCS-needed consumer is NOT covered and needs its own disposition. The disposition is not strengthened into a native null-log observation or any all-consumer closure.
* final checks (direct non-piped records in `runs/`, selected counts parsed from the logs after they completed, nothing pinned or assumed in advance): {json.dumps(final)}. The final set is only formatting, the lint of the crate whose test changed and that guard: nothing else compiled or ran changed.
* retained developer runs with their reasons: {json.dumps(retained)}.
* PRE and POST full input maps and extra identities are equal (`tvr459-inputmaps-compare-20261107.json`); the three inherited uncompiled Lua files keep their CRLF/Git-LF qualification exactly (receipt).
* process: the family name `tvr459` was verified unused before any helper or output; helpers were derived from the tvb1 ones by `derive-tvr459-helpers-20261108.py` (exclusive creation, ONE family-token replacement; a first draft of that script that also replaced the date token was replaced before it ran, because it would have rewritten references to shared helpers) and edited by hand; source with the Write and Edit tools; every final fmt and clippy invocation carries its own `rustup component add` and version command. Root's notes `Root-tacview-correction-feedback-20261108-0008.json` and `-0012.json` are read in full and literal here with the two receipts the first names (the second, which arrived during the batch, observed that the erratum's first wording claimed nothing above it was rewritten although this batch also corrected the section's guard, comms-route and identity prose: the erratum now says the earlier packet, raw capture and freeze remain literal and the current corrected prose supersedes its earlier version, with no new run); the first note's process learning (batched Git joins, not per-file processes) is applied: the identity join uses a handful of `git` calls on the files relied on.

## Limits

Source-reachability and compiled-reference disposition for the SUPPORTED headless configuration only. No run observed `tacview_log_file` staying NULL; arbitrary host, configuration, network or device implementations and any DCS-needed consumer are not covered; `ENV-PHYSICAL-FLAGS-HOST-STORE-NATIVE-ACCEPTANCE-1`, the other consumers, the READ-dependent native steps, lifetime, DCS and all ten D3 requirements stay OPEN; no native, boot or lifetime proof and no build of product source.
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
    handle.write((json.dumps({"files": files, "scope": "Correction packet of the Tacview consumer disposition only (guard enclosure and identity scope); not a native or Windows result, review, approval or acceptance.", "limit": "no production source changed"}, indent=2) + "\n").encode("utf-8"))
subprocess.run(["git", "-C", str(REPO), "add", "-A", REL], check=True)
# the manifest is the EXACT Git-tracked packet set minus itself (derived, never a guessed count)
tracked = sorted(p[len(REL) + 1:] for p in subprocess.run(["git", "-C", str(REPO), "ls-files", "--", REL], capture_output=True, check=True, text=True).stdout.split())
if tracked != sorted([f["path"] for f in files] + ["manifest.json"]):
    problems.append("the manifest plus itself is not the exact Git-tracked packet set")
print(json.dumps({"problems": problems, "manifest_entries": len(files), "git_tracked": len(tracked)}))
sys.exit(1 if problems else 0)
