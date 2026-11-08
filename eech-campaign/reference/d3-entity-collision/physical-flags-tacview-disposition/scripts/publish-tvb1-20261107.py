"""Publishes the README of the Tacview-consumer disposition packet and checks, against the actual Git index blob bytes, every literal file of the packet and every changed source file, and that the manifest is the EXACT
Git-tracked packet set minus itself. Run AFTER the freeze (which wrote the packet files and the freeze receipt) and BEFORE the commit. Derived from publish-sb120-20261107.py (family token replaced by
derive-tvb1-helpers-20261107.py) and then rewritten by hand for this batch.
Usage: publish.py <repo-root> [dry] (dry: a throw-away -DRY packet, no source file is staged or compared)"""
import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1])
DRY = len(sys.argv) > 2 and sys.argv[2] == "dry"
PACK = REPO / ("eech-campaign/reference/d3-entity-collision/physical-flags-tacview-disposition" + ("-DRY" if DRY else ""))
REL = str(PACK.relative_to(REPO)).replace("\\", "/")
FREEZE = "D3-TACVIEW-CONSUMER-DISPOSITION-FROZEN-20261107.json"
freeze = json.loads((PACK / FREEZE).read_text(encoding="utf-8"))
problems = list(freeze["hygiene_problems_at_freeze"])
final = {k: {"exit": v["actual_exit"], "tests_passed": v["tests_passed"], "selected": v["selected_counts"]} for k, v in freeze["final_runs"].items()}
retained = {r["label"]: {"exit": r["actual_exit"], "why": r.get("why")} for r in freeze["retained_development_runs"]}
readme = f"""# Tacview consumers of MOVED and ROTATED: dormant in the headless shipping path (disposition from source, build and retained evidence; UNREVIEWED; native UNRUN)

Packet of ONE bounded batch: the next named required work of `ENV-PHYSICAL-FLAGS-HOST-STORE-NATIVE-ACCEPTANCE-1`, the Tacview/MOVED-ROTATED immediate consumer. The first step was to establish from real callers, the build's source selection and retained compiled evidence whether that consumer is ACTIVE in the shipping `EECH_DC` module, assuming neither answer. **It is a disposition, not a review, approval, native result or acceptance; it repairs and changes no production source.**

* source before: `{freeze["source_before"]}` (the launch checkpoint `{freeze["checkpoint"]}`); the product and crate sources equal `3abf03e51`'s (`git diff` empty, recorded in `tvb1-dormancy-evidence-20261107.json`), the source of the retained Windows reports. The frozen receipt is `{FREEZE}`; `manifest.json` lists every committed file of the packet with its SHA-256; `literal-bytes-check.json` compares the actual Git blobs.
* **finding:** the legacy C Tacview writer is COMPILED IN (it is in `sources.txt`; the retained ordinary and original profile `engine-archive-nm.txt` reports of the 3abf build define it and the mobile update objects reference it) but it is NOT REACHABLE from the headless boot. `tacview_is_logging ()` is `tacview_log_file != NULL`; the file is opened only by `start_tacview_logging`, whose only callers are the user's gunship assignment (`assign_entity_to_user`, `helicop.c`) and the flight loop's entry (`flight.c`, logging mode 3, called only by the game phases the headless boot replaces by its own sequence, without the logging start). No engine C file, no Rust of the shipping module or host names any of them (and the compiled `eech_engine.o` references no Tacview symbol). The logging mode defaults to 0 and the retail `EECH.INI` of both closed default roots reads `tacview_logging=0` (the engine reads that file at boot); neither root holds a `tacview-logs` directory or a `.txt.acmi` file.
* **consequence for the words:** every read of MOVED and ROTATED (nine, in `fw_updt.c`, `hc_updt.c`, `aa_updt.c`, `rv_updt.c`) is inside an `if (tacview_is_logging () ...)` block and the frame reset of both words (`up_update.c`) is behind `tacview_is_logging () && tacview_reset_frame ()`; the per-entity logging bit is set only by the writer. In the headless shipping path MOVED and ROTATED are SET (through the host store, the generic set contract) and never read or reset, so no consumer of them runs. The Tacview recordings of the shipping runs are the Rust recorder's (`eech_world::tacview`, `host.record`), made from host observations, not from these words. Hence no real-caller control in both profiles was written: there is no reached caller to characterise, and none was manufactured (no speculative renderer or writer fixture).
* **pinned, text only:** `physical_flags_guard.rs` gained `the_legacy_tacview_writer_is_compiled_in_but_every_read_of_moved_and_rotated_waits_on_a_log_file_the_headless_boot_never_opens` (compiled-in source list, the single assignment sites of the log file, the exactly two callers and where they sit, the shipping C and Rust sources naming none of them, every read inside the logging guard, the guarded frame reset, and a matched positive and negative of its own predicate on synthetic text). A new caller, an unguarded read or a mention in the shipping sources fails it and the consumer must then be characterised first. The contract's section 13 states the disposition and its limits.
* final checks (direct non-piped records in `runs/`, selected counts parsed from the logs after they completed, nothing pinned in advance): {json.dumps(final)}. The final set is only formatting, the lint of the crate whose test changed and that guard: nothing else compiled or ran changed (no production source changed), and no control, build, replay, native, Windows or full-pack run was made.
* retained developer runs with their reasons: {json.dumps(retained)}.
* PRE and POST full input maps and extra identities are equal (`tvb1-inputmaps-compare-20261107.json`); the three inherited uncompiled Lua files keep their CRLF/Git-LF qualification exactly (receipt). `tvb1-dormancy-evidence-20261107.json` holds the literal excerpts of the retained reports (each report's SHA-256 and path; the 8 MB reports are not copied) and of the closed default roots; no running job's root was touched.
* process: the family name `tvb1` was verified unused before any helper or output (its only content hit is a byte coincidence inside a retained recording, not a name); helpers were derived by `derive-tvb1-helpers-20261107.py` (exclusive creation, one family-token replacement) and edited by hand; new source with the Write and Edit tools; every final fmt and clippy invocation carries its own `rustup component add` and version command. Root's notes of this batch (2349, 2350) are read in full and published as literals with the two receipts they ask for (the publication identity of the source-reviewed and native-closure packets, and the exact-commit CI readback of `b1b9a5d9e`).

## Limits

This is a source-reachability and compiled-reference disposition. It does not observe `tacview_log_file` staying NULL in a run; a host that passed `tacview_logging=<n>` as an engine argument or an ini that set it is not excluded by the retained roots (what keeps `start_tacview_logging` unreachable is the boot's own sequence, which the guard pins). It does not decide whether DCS needs MOVED or ROTATED for anything else, does not close `ENV-PHYSICAL-FLAGS-HOST-STORE-NATIVE-ACCEPTANCE-1` or any other consumer (helicopter, vehicle, ship and weapon-vector remain as listed), and it is not a native, boot, lifetime or public-path proof. If the headless boot gains a gunship assignment or the flight entry, or a host wants the legacy writer, the consumer becomes active and needs the real-caller control in both profiles first.
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
    handle.write((json.dumps({"files": files, "scope": "Source, build and retained-evidence packet of the Tacview consumer disposition only; not a native or Windows result, review, approval or acceptance.", "limit": "no production source changed"}, indent=2) + "\n").encode("utf-8"))
subprocess.run(["git", "-C", str(REPO), "add", "-A", REL], check=True)
# the manifest is the EXACT Git-tracked packet set minus itself (derived, never a guessed count)
tracked = sorted(p[len(REL) + 1:] for p in subprocess.run(["git", "-C", str(REPO), "ls-files", "--", REL], capture_output=True, check=True, text=True).stdout.split())
if tracked != sorted([f["path"] for f in files] + ["manifest.json"]):
    problems.append("the manifest plus itself is not the exact Git-tracked packet set")
print(json.dumps({"problems": problems, "manifest_entries": len(files), "git_tracked": len(tracked)}))
sys.exit(1 if problems else 0)
