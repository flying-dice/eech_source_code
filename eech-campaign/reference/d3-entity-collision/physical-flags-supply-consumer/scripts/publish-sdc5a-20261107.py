"""Publishes the README of the supply-descriptor packet and checks, against the actual Git index blob bytes, every literal file of the packet and every changed source file, and that the manifest is the EXACT Git-tracked packet set minus
itself. Run AFTER the freeze (which wrote the packet files and the freeze receipt) and BEFORE the code commit. Usage: publish.py <repo-root> [dry] (dry: a throw-away -DRY packet, no source file is staged or compared)"""
import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1])
DRY = len(sys.argv) > 2 and sys.argv[2] == "dry"
PACK = REPO / ("eech-campaign/reference/d3-entity-collision/physical-flags-supply-consumer" + ("-DRY" if DRY else ""))
REL = str(PACK.relative_to(REPO)).replace("\\", "/")
FREEZE = "D3-PHYSICAL-FLAGS-SUPPLY-DESCRIPTOR-CODE-FROZEN-20261107.json"
freeze = json.loads((PACK / FREEZE).read_text(encoding="utf-8"))
problems = list(freeze["hygiene_problems_at_freeze"])
final = {k: {"exit": v["actual_exit"], "tests_passed": v["tests_passed"], "selected": v["selected_counts"], "descriptors_delivered": v.get("descriptors_delivered"), "hosted_traffic": v.get("hosted_traffic_groups_sets_reads_resets_fresh")} for k, v in freeze["final_runs"].items()}
readme = f"""# Physical flag store: the shipping supply arrival descriptor, the third real immediate consumer of the UPDATED word (characterisation; UNREVIEWED; native UNRUN)

Packet of the local checks of ONE bounded batch: a real control of `campaign_supply_arrival_check` (`special/guide/gd_supply_arrival.c`), the shipping arrival that reads `descriptor.updated = get_local_entity_int_value (member, INT_TYPE_UPDATED)` for a fixed-wing member and hands the descriptor to the engine's OWN Rust boundary (`supply_reached`, `checked_bool ("updated", ...)`), in the ORIGINAL and the HOSTED profile with the same scenarios and assertions. **It is a characterisation and evidence of one candidate, not a review, approval, native result or acceptance. No semantic repair, normalisation or baseline change is made.**

* source before: `{freeze["source_before"]}` (the launch checkpoint `{freeze["checkpoint"]}`). The frozen receipt is `{FREEZE}`; `manifest.json` lists every committed file of the packet with its SHA-256; `literal-bytes-check.json` compares the actual Git blobs. The changed files are exactly the nine of the receipt (a feature-gated C world, the engine's feature-gated seam and its test, the gate of two test-only helpers in the product C, two manifests and the lock, the build define and the guard) plus the contract's section 11.
* the control: `eech-engine/tests/physical_flags_supply.rs` (feature `physical-flags-supply-check`; the world is `csrc/eech_physical_flags_supply_check.c`, which declares every hand-made collaborator; the seam `eech_engine::supply_check` hands the real `supply_reached` to the test and refuses every other callback by name). Profiles: `--features physical-flags-supply-check,collision-fixture` (the engine's own arrays) and the default build (the host's words, the REAL store of eech-world behind the engine's model callback).
* **finding, CONFIRMED at this consumer scope (and left as it is):** the raw word of a set flag is its lane's mask, the real Rust boundary accepts only 0 and 1, so lane 0 (mask 1) passes as `true` and every other lane (masks 2, 4, 2^30, bit 31 as INT_MIN) is refused by name (`invalid updated flag <mask>`), which the campaign turns into its own fatal; a refused descriptor never reaches the host's answer. The source guard `physical_flags_guard.rs` pinned this as source-derived before; it is now run end to end. It is recorded as an open defect row of the card for a reviewer's decision, not corrected here.
* also observed in both profiles: the positive (a linked fixed-wing task leader, the flag unset, raw word 0, ONE descriptor with the gathered facts and the roster identity; in the hosted profile exactly one destructive read of the host per call), the destructive neighbour read and its order dependence, the reset extent (a set bit above 131072 slots survives and is read), and the matched negative of a member with no host identity (refused before the getter is read). Descriptors delivered per run: 9. Hosted traffic is nonzero in every kind and nothing was refused or off-grammar.
* final checks `s1` to `s11` (all exit 0, direct non-piped records in `runs/`, selected counts and the control's own facts read from the logs): {json.dumps(final)}.
* development runs retained with their reasons (`runs/`, the receipt): the first compile against the shared support file (a lint scope), a link error (the gate of the test-only key helpers), a segfault in the real waypoint-into-task insertion (the one link now set in the raw record and declared), the unlinked member that the real group response had linked after all (it is now kept out of the MEMBER list), the d4 trace run whose C string was broken by a heredoc, and the clippy runs that forgot `rustup component add`.
* PRE and POST full input maps and the extra identities are equal (`sdc5a-inputmaps-compare-20261107.json`); the three inherited uncompiled Lua files keep their CRLF/Git-LF qualification exactly (receipt). The accepted 020 binaries, the finite default and the finite witness family were not replayed.
* process: the family name `supplyc5a` was already used by Root's launcher, so this batch uses the verified-unused `sdc5a`; every helper was derived by an explicit script (`derive-sdc5a-helpers-20261107.py`, exclusive creation, one family-token replacement, output inspected) and then edited by hand; all names were checked unused by `sdc5a-name-check` (with its negative control) before the real PRE.
* **the exact product-source delta, qualified (Root's note of 22:01):** `aphavoc/source/entity/mobile/mb_hostmodel.c` IS a changed raw product-source file. The change is one preprocessor condition (and its comment): the gate of the two test-only key helpers `campaign_model_flag_key_for_test` and `campaign_model_flag_forget_keys_for_test` gains `|| defined(EECH_PHYSICAL_FLAGS_SUPPLY_CHECK)`. That define is set only by the build for the new feature (the guard pins the define, the feature, its absence from every manifest and the fixture's `#ifdef` confinement), so in a build without the feature the condition evaluates exactly as before: this is a source and preprocessor fact, shown by the guard and by the existing hosted and original flag controls running unchanged (s8, s9), not a claim that any raw product row equals the sealed 020's or that the new test code ran in the sealed 020 binaries; no binary is re-identified.
* **process lessons (Root's notes, adopted and carried into the final command block of the next batch):** (1) 22:05, editing: a shell/Python heredoc collapsed a C backslash-n and broke the trace build (d4, retained, a setup failure, not a behavioural result); new or changed source and diagnostic source is written with the Write and Edit tools with literal contents, shell tools only run commands, and any byte-copy or recovery scripting is kept in an explicit file whose output is inspected. (2) 22:15, toolchain: rustup components added in one `--rm` container are absent from the next (d11 and d12, retained as setup slips); every final fmt and clippy invocation carries its own `rustup component add` and its version command (`s1` to `s4`). (3) 22:22, validation efficiency: no developer run only to discover counts; the freeze parses the actual selected counts from the final receipts after they complete, requires the selected targets to have run with nonzero results and no failure, and pins no count in advance (d18 to d20 had already run and are kept, not relabelled as inside PRE). No test was added for a tooling or prose preference. (4) My own miss: I listed Root's notes with `| tail -1` and so read one of seven until the freeze script's own check of unread notes refused for the other six (all seven are read in full and recorded in `sdc5a-feedback-files-20261107.json`).
* Root's small safe supplemental receipts named in those notes are kept literally in the packet (the publication identity of the earlier packets, the live ledger/Jira/PR receipts, the live-services receipt and the exact CI readback of c5a3ed); none is evidence of this consumer.

## What the control ran (and what it did not)

Real: the producer and its roster identity, the mobile/aircraft/fixed-wing getters, the guide criteria, the real list functions (member, guide, group, current waypoint), the real update reset, the real Rust boundary, and in the hosted profile the real flag store. Hand-made and declared in the C file: the entity pool and raw records, the waypoint's parent pointer in its raw record, a private roster collaborator that issues `eech-world:<n>`, the delta time and comms model.

**Not run:** the helicopter and the other mobile kinds, the other consumers (Tacview, vehicle, ship, weapon-vector), the shipped `Engine::boot` bridge, any native or Windows run, whether and when a native campaign reaches a set flag at a non-zero lane here (the native 35-frame default is not evidence of this consumer), a lifetime or replacement of the host's store, whole-global or FPU restoration, the weapon-vector path.
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
    handle.write((json.dumps({"files": files, "scope": "Source and local-check packet of the supply arrival descriptor control only; not a native result, review, approval or acceptance."}, indent=2) + "\n").encode("utf-8"))
subprocess.run(["git", "-C", str(REPO), "add", "-A", REL], check=True)
# the manifest is the EXACT Git-tracked packet set minus itself (derived, never a guessed count)
tracked = sorted(p[len(REL) + 1:] for p in subprocess.run(["git", "-C", str(REPO), "ls-files", "--", REL], capture_output=True, check=True, text=True).stdout.split())
if tracked != sorted([f["path"] for f in files] + ["manifest.json"]):
    problems.append("the manifest plus itself is not the exact Git-tracked packet set")
print(json.dumps({"problems": problems, "manifest_entries": len(files), "git_tracked": len(tracked)}))
sys.exit(1 if problems else 0)
