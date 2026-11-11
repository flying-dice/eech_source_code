"""Freezes the PUBLIC WITNESS batch packet (the host-owned Lua driver of the physical flag store, its scripted-engine tests, the exact pattern helpers and the finite native plan; candidate, UNREVIEWED, native UNRUN) ONCE.
Usage: freeze.py readers-pre | freeze.py freeze <repo-root> <external-freeze-path> <post-utc> [dry]
This script and every reader it names are FINAL before the PRE identity capture: their hashes are recorded at PRE (`readers-pre`, hwcr-readers-PRE-20261107.json) and this script REFUSES to freeze if any of them differs now
(a deviation is an erratum, never hidden). Every run record is DERIVED from the files on disk and read AS A WHOLE: state terminal, the declared helper, a recorded exit, both logs present, and the actual `test result` lines parsed
(a Cargo exit alone is never trusted: a selected test binary must have run with a NONZERO count). Root's supervision notes are read AT THE ACTUAL TIME: a file present that this script was not told was read is a problem."""
import datetime
import hashlib
import json
import pathlib
import re
import shutil
import subprocess
import sys

EXT = pathlib.Path("R:/d3-evidence")
READERS = ["run-physical-flags-witfix-direct-20261107.py", "check-physical-flags-witfix-linux-20261107.sh", "run-hwcr-final-set-20261107.sh", "hwcr-extra-identity-20261107.py", "freeze-hwcr-20261107.py", "publish-hwcr-20261107.py",
           "sector-roster-inputmap-batched-20261106.py", "sho-compare-maps-20261107.py", "preflight-receipts-20261107.py"]
READERS_PRE = EXT / "hwcr-readers-PRE-20261107.json"
if sys.argv[1] == "readers-pre":
    assert not READERS_PRE.exists()
    READERS_PRE.write_text(json.dumps({r: hashlib.sha256((EXT / r).read_bytes()).hexdigest() for r in READERS}, indent=2) + "\n", encoding="utf-8", newline="\n")
    print("readers recorded", len(READERS))
    sys.exit(0)
assert sys.argv[1] == "freeze"
REPO = pathlib.Path(sys.argv[2]).resolve()
FREEZE = pathlib.Path(sys.argv[3])
POST_UTC = sys.argv[4]
DRY = len(sys.argv) > 5 and sys.argv[5] == "dry"
PACK_REL = "eech-campaign/reference/d3-entity-collision/physical-flags-public-witness-fix-020" + ("-DRY" if DRY else "")
PACK = REPO / PACK_REL
RUNS = PACK / "runs"
SCRIPTS = PACK / "scripts"
CHECKPOINT = "4d373c8f7"
PREFIX = "flags-witfix-"
HELPER = "check-physical-flags-witfix-linux-20261107.sh"
HELPER_TEXT = (EXT / HELPER).read_text(encoding="utf-8")
IMAGE = re.search(r"^image=(\S+)$", HELPER_TEXT, flags=re.M).group(1)
VOLUME = re.search(r"^volume=(\S+)$", HELPER_TEXT, flags=re.M).group(1)
ROOT_LEASED = {"image": "sha256:9d247b4ea15457f06579ef703ce287e96358128e7a2217f2833ee8b628fc85c3", "volume": "eech-linux-d3-physical-flags-9d247-20261107"}  # the names Root leased, asserted against the helper
CODE_COMMIT = "020fb3cfa5450acd91f136e678974160c12c8275"
DRIVER_REL = "eech-campaign/crates/eech-world/lua/physical-flags-witness.lua"
PRE_UTC = (EXT / ("hwcr-dry-PRE-utc-20261107.txt" if DRY else "hwcr-PRE-utc-20261107.txt")).read_text().strip()
RETAINED = {label: (v["actual_exit"], v["why"]) for label, v in json.loads((EXT / "hwcr-retained-dev-runs-20261107.json").read_text(encoding="utf-8")).items()}
FINAL = ["f1-fmt-check", "f2-clippy-world-engine-dc", "f3-witness-driver", "f4-world-all", "f5-engine-sys-lua-guards"]
if DRY:
    FINAL = ["d7-clippy-world-engine-dc", "d5-green-attempt4"]
MIN_TARGETS = {
    "f3-witness-driver": [("physical_flags_witness_driver", 25)],
    "f4-world-all": [("physical_flags_witness_driver", 25), ("host_callbacks", 82), ("entity_authority", 120), ("physical_flags_host", 7), ("sector_height_witness_driver", 27)],
    "f5-engine-sys-lua-guards": [("entity_roster_guard", 20), ("physical_flags_guard", 6)],
    "d5-green-attempt4": [("physical_flags_witness_driver", 25)],
    }
FEEDBACK_READ = json.loads((EXT / "hwcr-feedback-files-20261107.json").read_text(encoding="utf-8"))["read"]
ROOT_LITERALS = sorted(p.name for p in list(EXT.glob("Root-physical-flags-witness-fix-feedback*.json")))  # the review and Root records are literal in physical-flags-witness-f4d-reviewed
problems = []


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def git(*args, binary=False):
    result = subprocess.run(["git", "-C", str(REPO), *args], capture_output=True, check=True)
    return result.stdout if binary else result.stdout.decode("utf-8", "replace")


def jload(path):
    return json.loads(pathlib.Path(path).read_text(encoding="utf-8"))


# ---------------------------------------------------------------------------------------------------------------- readers, lease names, supervision notes
pre_readers = jload(READERS_PRE)
for reader in READERS:
    if pre_readers.get(reader) != sha(EXT / reader):
        problems.append("reader differs from its PRE identity (erratum required): " + reader)
if IMAGE != ROOT_LEASED["image"] or VOLUME != ROOT_LEASED["volume"]:
    problems.append("the helper names another image or volume than the one Root leased")
present_feedback = sorted(p.name for p in list(EXT.glob("Root-physical-flags-witness-feedback-*.json")) + list(EXT.glob("Root-physical-flags-witness-fix-feedback*.json")))
for name in present_feedback:
    if name not in FEEDBACK_READ:
        problems.append("a Root supervision note was present and not read: " + name)

# ---------------------------------------------------------------------------------------------------------------- the input maps, equal at PRE and POST
compare = jload(EXT / ("hwcr-dry-inputmaps-compare-20261107.json" if DRY else "hwcr-inputmaps-compare-20261107.json"))
if not compare["all_rows_equal"] or not compare["extra_identity_equal"] or compare["rows_that_differ"] or compare["rows_only_in_pre"] or compare["rows_only_in_post"]:
    problems.append("the PRE and POST input maps or extra identities are not equal")
post_map = jload(EXT / ("hwcr-dry-inputmap-POST-20261107.json" if DRY else "hwcr-inputmap-POST-20261107.json"))
changed_all = post_map["changed_from_head"]
# a file whose RAW bytes differ from the head blob but whose Git identity (after Git's own clean filters) equals it is an inherited line-ending qualification, never a change of this batch (Root's binding of the 684 publication names the same three uncompiled Lua files)
inherited = [p for p in changed_all if post_map["files"][p].get("head_blob") is not None and post_map["files"][p]["head_blob"] == post_map["files"][p]["git_blob_of_raw_bytes"]]
changed = [p for p in changed_all if p not in inherited]
for path in inherited:
    if not path.startswith("eech-campaign/crates/eech-world/tests/lua/tree-geometry-"):
        problems.append("an inherited raw-versus-Git difference other than the three known uncompiled Lua files: " + path)
inherited_rows = [{"path": p, "raw_sha256": post_map["files"][p]["raw_sha256"], "git_blob_of_raw_bytes": post_map["files"][p]["git_blob_of_raw_bytes"], "head_blob": post_map["files"][p]["head_blob"]} for p in inherited]
if post_map["deleted_from_head"]:
    problems.append("files deleted from head: %r" % post_map["deleted_from_head"])
# the product's compiled inputs are unchanged: nothing outside Lua, tests and docs may differ from the 020 sources
allowed_prefixes = ("eech-campaign/crates/eech-world/lua/", "eech-campaign/crates/eech-world/tests/")
for path in changed:
    if not path.startswith(allowed_prefixes):
        problems.append("a compiled or product input changed (this batch is Lua, tests and docs only): " + path)

# ---------------------------------------------------------------------------------------------------------------- the runs, read as a whole
def read_run(label, role):
    stem = PREFIX + label
    record_path = EXT / (stem + ".json")
    out_path, err_path = EXT / (stem + ".stdout.log"), EXT / (stem + ".stderr.log")
    if not (record_path.exists() and out_path.exists() and err_path.exists()):
        problems.append("%s: record or logs missing (%s)" % (role, label))
        return None
    record = jload(record_path)
    row = {"label": label, "state": record.get("state"), "actual_exit": record.get("actual_exit"), "started_utc": record.get("started_utc"), "completed_utc": record.get("completed_utc"), "command": record.get("command"),
           "record_sha256": sha(record_path), "stdout_sha256": sha(out_path), "stderr_sha256": sha(err_path), "uses_declared_helper": any(HELPER in a for a in record.get("args", []))}
    if record.get("state") != "terminal":
        problems.append("%s %s: not terminal" % (role, label))
    if not row["uses_declared_helper"]:
        problems.append("%s %s: not run with the declared helper" % (role, label))
    # cargo prints `Running <target>` (and `Doc-tests <crate>`) to STDERR and each target's `test result` line to STDOUT, one per target in the same order: the two lists are zipped, and a different number of each is a problem
    text = out_path.read_text(encoding="utf-8", errors="replace")
    names = []
    for line in err_path.read_text(encoding="utf-8", errors="replace").splitlines():
        m = re.match(r"\s*(?:Running|Doc-tests)\s+(.+?)(?: \(|$)", line)
        if m:
            name = m.group(1)
            name = name[len("tests/"):] if name.startswith("tests/") else name
            names.append(name[:-3] if name.endswith(".rs") else name)
    results = []
    for line in text.splitlines():
        m = re.match(r"test result: (\w+)\. (\d+) passed; (\d+) failed", line)
        if m:
            results.append({"status": m.group(1), "passed": int(m.group(2)), "failed": int(m.group(3))})
    counts = {}
    if names and len(names) != len(results):
        problems.append("%s %s: %d cargo targets on stderr and %d test result lines on stdout" % (role, label, len(names), len(results)))
    else:
        for name, result in zip(names, results):
            counts[name] = result
    row["test_binaries"] = len(counts)
    row["tests_passed"] = sum(c["passed"] for c in counts.values())
    row["tests_failed"] = sum(c["failed"] for c in counts.values())
    row["selected_counts"] = {}
    for name, minimum in MIN_TARGETS.get(label, []):
        got = counts.get(name)
        row["selected_counts"][name] = got["passed"] if got else None
        if got is None or got["passed"] < minimum or got["failed"]:
            problems.append("%s %s: %s ran %s tests, at least %d must pass" % (role, label, name, got, minimum))
    return row


final_rows = {}
for label in FINAL:
    row = read_run(label, "final")
    if row is not None:
        final_rows[label] = row
        if row["actual_exit"] != 0:
            problems.append("final %s exit %r" % (label, row["actual_exit"]))
        if label.startswith(("w2", "d9")) and "clippy" not in (row["command"] or ""):
            problems.append("final %s did not run clippy" % label)
progress = (EXT / "hwcr-final-set-20261107.progress").read_text(encoding="utf-8").split() if not DRY else []
if not DRY:
    expected = [x for label in FINAL for x in (label, "exit=0")] + ["DONE"]
    if progress != expected:
        problems.append("the final-set progress file is not exactly the labels with exit=0 and DONE: %r" % progress)
retained_rows = []
for label, (exit_code, why) in sorted(RETAINED.items()):
    row = read_run(label, "retained")
    if row is not None:
        if row["actual_exit"] != exit_code:
            problems.append("retained %s exit %r is not the recorded %r" % (label, row["actual_exit"], exit_code))
        row["why"] = why
        retained_rows.append(row)

# ---------------------------------------------------------------------------------------------------------------- the changed files against actual Git bytes, and the plan
code_rows = []
for path in changed:
    data = (REPO / path).read_bytes()
    code_rows.append({"path": path, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(), "git_blob_of_raw_bytes": git("hash-object", path).strip(), "head_blob": post_map["files"][path].get("head_blob")})
driver_bytes = (REPO / DRIVER_REL).read_bytes()
driver_blob = git("hash-object", DRIVER_REL).strip()
driver_text = driver_bytes.decode("utf-8")
build = jload(EXT / "physical-flags-native-windows-020-20261107-completion.json")
if build["source"] != CODE_COMMIT or build["actual_exit"] != 0:
    problems.append("the 020 build receipt is not a successful build of the 020 source")
binaries = {}
for profile, folder in (("ordinary", "ordinary"), ("original", "original")):
    prof = build["profiles"][profile]
    named = {row["path"]: row for row in prof["payloads"] if row["path"] in ("eech_dc.dll", "eech-world.exe", "lua.dll")}
    binaries[folder] = {"directory": prof["binary"], "volume": prof["volume"], "build_exit": prof["actual_exit"], "files": {k: {"sha256": v["sha256"], "bytes": v["bytes"]} for k, v in named.items()}}
    if prof["actual_exit"] != 0 or len(named) != 3:
        problems.append("the %s profile receipt is incomplete" % profile)
MODES = [("flags_original", None), ("flags_control", None), ("read_changed", None)]
REFUSALS = [(f"{event}_{kind}", marker) for event, trio in (("group", ()), ("set", ()), ("read", ()), ("reset", ())) for kind, marker in (("withheld", "entity_scan.callback"), ("malformed", "entity_scan.answer"), ("wrong_state", "entity_scan.state"))]
REFUSALS += [("group_bad_key", "flag.key"), ("group_duplicate_key", "flag.key"), ("group_wrong_eligibility", "flag.answer"), ("set_wrong_lane", "flag.answer"), ("set_wrong_word", "flag.word"), ("read_wrong_array", "flag.answer"),
             ("read_wrong_word", "flag.word"), ("reset_wrong_array", "flag.answer")]
for mode, marker in MODES + REFUSALS:
    # the driver builds the generic modes from an event and a kind (`event .. "_withheld"`): those are checked by their suffix and the event, the others by their name
    head, _, kind = mode.partition("_")
    generic = head in ("group", "set", "read", "reset") and kind in ("withheld", "malformed", "wrong_state")
    needle = '"_' + kind + '"' if generic else mode
    if needle not in driver_text or (generic and '"' + head + '"' not in driver_text) or (marker is not None and marker not in driver_text):
        problems.append("the plan names a mode or marker the driver does not hold: %s %s" % (mode, marker))
if len(REFUSALS) != 20:
    problems.append("the plan must hold 20 refusal modes, holds %d" % len(REFUSALS))
SETUP = {"scenario": "lebanon_retail", "hours": "0.001", "frame_ms": "100", "seed": "1", "frames": 35, "established_by": "the unchanged-default 020 native runs of campaign.lua (ordinary_short and original_short, Root binding 40e88133...), actual 35 frames of 100 ms"}
COMMON = ["root=<fresh verified retail root>", "out=<new report path, created exclusively>", "scenario=lebanon_retail", "hours=0.001", "frame_ms=100", "seed=1"]
steps = [{"step": 1, "mode": "flags_original", "binaries": "original", "argv_after_script": COMMON + ["mode=flags_original"], "needs": [], "credited": "PASS: the host was never asked a flag question (FAILURE if it was)"},
         {"step": 2, "mode": "flags_control", "binaries": "ordinary", "argv_after_script": COMMON + ["mode=flags_control", "original=<step 1 report>"], "needs": [1], "credited": "PASS: an accepted group, set and reset, no fault, no backing difference; reads and consumption reported; UNREACHED if a phase was not accepted"},
         {"step": 3, "mode": "read_changed", "binaries": "ordinary", "argv_after_script": COMMON + ["mode=read_changed", "control=<step 2 report>"], "needs": [2], "credited": "PASS: the changed word carried into the next candidate question; refused before boot if the control reached no consumable read"}]
for n, (mode, marker) in enumerate(REFUSALS, start=4):
    steps.append({"step": n, "mode": mode, "binaries": "ordinary", "argv_after_script": COMMON + ["mode=" + mode, "control=<step 2 report>"], "needs": [2], "credited": "PASS only if the engine stops at %s after the one eligible intervention" % marker})
plan = {
    "kind": "finite executable native plan of the public host-owned Lua -> native EECH_DC module -> reference host witness of the physical flag store, CORRECTED for the two P2s of the V2 review of f4d62c9b4 (candidate; UNREVIEWED; native UNRUN; Root executes it, the author executes none; the same 23 steps and setup as the parent plan whose Git SHA-256 was 67b7faac8dfc93592f60513de2f8136dd6bf1e6533f751885068fe5956870633, the driver literal replaced)",
    "binds": {"code_commit_of_the_binaries": CODE_COMMIT, "immutable_snapshot": build["snapshot"], "snapshot_receipt_sha256": build["snapshot_receipt_sha256"], "build_receipt_sha256": sha(EXT / "physical-flags-native-windows-020-20261107-completion.json"),
              "cache_release_sha256": sha(EXT / "physical-flags-020-WINDOWS-CACHE-RELEASE-20261107.json"), "accepted_baseline": "windows-retail v1 (the M4 acceptance baseline; the established default plan of the unchanged 020 runs)",
              "binaries": binaries, "binaries_unchanged_note": "the 90 built payloads are unchanged; the driver is staged beside them from its frozen literal and the binaries are never re-identified as a later Lua"},
    "driver_literal": {"path": DRIVER_REL, "sha256": hashlib.sha256(driver_bytes).hexdigest(), "git_blob_of_raw_bytes": driver_blob, "bytes": len(driver_bytes),
                       "parent_candidate": "f4d62c9b4e7573a149973d5eb9357c77f08c609f", "parent_driver_sha256": "2cd536d77470049f4912dd9098132adbe56a0bbff37e1fa9f2fd7de1953902ab", "payload_entity_authority_sha256_in_020": "856f680ae2fc9a3b93ac1c56451bfdde2f209eb9f1e9e7eb42c2956320fb4e3e (the corrected entity-authority.lua of the parent candidate is not used by this plan)"},
    "setup": SETUP,
    "invocation": "<ordinary or Original eech-world.exe> <staged lua/physical-flags-witness.lua> <argv_after_script>",
    "process_rules": ["one bounded process per step, a fresh verified retail copy with pre/post input manifests each", "step 1 and step 2 FIRST (the characterisation, then the healthy paired control); no changed, withheld, malformed or wrong-identity step before the control PASSes",
                      "the same scenario, hours, frame_ms, seed and effective campaign and map in every report of the family", "a refusal of the driver's own argument validation is not a campaign result", "read-dependent cases may be UNREACHED in 35 frames; no actor, window or duration is chosen by what was reached",
                      "every reference (the Original report, the paired control) is validated COMPLETE before the output, preparation, boot, any provider call or any alteration: exact schema 1 records, canonical order and numbers, healthy count, oracle, first-event and consume relationships; a control unlocks only the cases its own positive reached",
                      "a refusal is credited only for the engine entry that received the altered answer stopping at the mode marker with no host callback in between; the boot is an engine entry (first-event frame 0); any later callback, normal return, other entry or other marker is a FAILURE",
                      "the changed-answer proof is the UPDATED fact carried into the next candidate question (one of the 44 payload words), not a changed collision outcome; phase counters alone do not establish coverage of each array or consumer"],
    "steps": steps,
    "not_claimed": ["any native result", "the supply, Tacview, helicopter, vehicle, ship, weapon-vector or message consumers", "a replacement store lifetime", "whole-FPU, i686, O2 or live-DCS behaviour", "the full 40 or all 8 regression packs", "approval or acceptance of anything"],
}

# ---------------------------------------------------------------------------------------------------------------- write the packet
if not DRY:
    assert not FREEZE.exists(), FREEZE
for d in (PACK, RUNS, SCRIPTS):
    d.mkdir(parents=True, exist_ok=True)
for row in list(final_rows.values()) + retained_rows:
    for suffix in ("json", "stdout.log", "stderr.log"):
        shutil.copyfile(EXT / ("%s%s.%s" % (PREFIX, row["label"], suffix)), RUNS / ("%s%s.%s" % (PREFIX, row["label"], suffix)))
for reader in READERS:
    shutil.copyfile(EXT / reader, SCRIPTS / reader)
for name in ("hwcr-readers-PRE-20261107.json", "hwcr-PRE-utc-20261107.txt", "hwcr-final-set-20261107.progress", "hwcr-final-set-lease-before-20261107.txt", "hwcr-final-set-lease-after-20261107.txt",
             "hwcr-inputmaps-compare-20261107.json", "hwcr-extra-identity-PRE-20261107.json", "hwcr-extra-identity-POST-20261107.json", "hwcr-retained-dev-runs-20261107.json", "hwcr-feedback-files-20261107.json", "hwcr-name-collision-erratum-20261107.json", "hwcr-preflight-receipts-20261107.json", "hwcr-preflight-negative-control-20261107.json"):
    source = EXT / name
    if source.exists():
        shutil.copyfile(source, PACK / name)
    elif not DRY:
        problems.append("expected evidence file missing: " + name)
for name in ROOT_LITERALS:
    shutil.copyfile(EXT / name, PACK / name)
(PACK / "physical-flags-public-witness-plan.json").write_text(json.dumps(plan, indent=2) + "\n", encoding="utf-8", newline="\n")
gitattributes = (REPO / "eech-campaign/reference/d3-entity-collision/physical-flags-keyfix-020-reviewed/.gitattributes").read_bytes()
(PACK / ".gitattributes").write_bytes(gitattributes)
receipt = {
    "kind": "CODE_FROZEN receipt of the public flag-store witness batch (Lua driver, scripted-engine tests, exact pattern helpers, finite native plan; candidate; UNREVIEWED; native UNRUN)",
    "frozen_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "dry": DRY, "source_before": git("rev-parse", "HEAD").strip(), "checkpoint": CHECKPOINT, "code_commit_of_the_binaries": CODE_COMMIT,
    "pre_utc": PRE_UTC, "post_utc": POST_UTC, "lease": {"volume": VOLUME, "image": IMAGE, "root_leased_names_asserted": ROOT_LEASED},
    "changed_files": code_rows, "inherited_raw_versus_git_qualifications": inherited_rows, "final_runs": final_rows, "retained_development_runs": retained_rows,
    "inputmaps_compare": compare, "feedback_files_present_at_freeze": present_feedback, "readers_PRE_identity_equal": not any("reader differs" in p for p in problems),
    "driver_literal": plan["driver_literal"], "hygiene_problems_at_freeze": problems,
    "not_run": ["any native, Windows, i686 or O2 run", "any C or Rust control (nothing they compile changed)", "the matrix, characterisation replay or full pack", "the other consumers of the flag store", "any Root-owned gate"],
}
FREEZE_TEXT = json.dumps(receipt, indent=2) + "\n"
if not DRY:
    FREEZE.write_text(FREEZE_TEXT, encoding="utf-8", newline="\n")
(PACK / "D3-PHYSICAL-FLAGS-PUBLIC-WITNESS-FIX-CODE-FROZEN-20261107.json").write_text(FREEZE_TEXT, encoding="utf-8", newline="\n")
print(json.dumps({"problems": problems, "final": {k: [v["actual_exit"], v["selected_counts"]] for k, v in final_rows.items()}, "retained": len(retained_rows), "changed": len(code_rows)}, indent=1))
sys.exit(1 if problems else 0)
