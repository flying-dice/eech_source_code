"""Freezes the PRODUCTION batch packet (the complete physical UPDATED / MOVED / ROTATED store moves from EECH_DC into its host: Task49 candidate, UNREVIEWED, native UNRUN) ONCE. Usage: freeze.py <repo-root> <external-freeze-path> <post-utc>
This script and every reader it names (the final-set runner, the join, the extra-identity script, the runner, the check script, the batched map script, the comparer and the receipt pre-flight) are FINAL before the PRE identity capture: their SHA-256 are recorded at PRE
(hwprod-readers-PRE-20261107.json) and this script REFUSES to freeze if any of them differs now (an earlier freeze's reader was edited after its POST; that cannot recur unnoticed; a deviation is recorded as an erratum, never hidden). Every receipt path is DERIVED from the run records on disk
(preflight-receipts-20261107.py ran on the development runs before PRE and runs again here on the final set), never typed as a prefix."""
import datetime
import hashlib
import json
import pathlib
import re
import shutil
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1]).resolve()
FREEZE = pathlib.Path(sys.argv[2])
POST_UTC = sys.argv[3]
EXT = pathlib.Path("R:/d3-evidence")
SEALED_SHA = {
    "review": "ce3bdb213fb90513816df7bb044681c7adc0197ee7a1c5b1cdd853e555c8be0b",
    "binding": "cfdd1d732767a6776857ae9f73b5cdb285154247cb522ea8efbfba3da2ff21da",
    "publication": "1840f47d4ed621fdacb8e7eb6400bc289e83b4b80e4ee8811a5a124f0c0b6c41",
}
FEEDBACK_AT_START = []  # no Root-physical-flags-production-feedback*.json existed when this batch began
FEEDBACK_READ = {  # the two files that arrived during development, read in full before the final PRE
    "Root-physical-flags-production-feedback-20261107-1703.json": None,
    "Root-physical-flags-production-feedback-20261107-1708.json": None,
}
PY = "C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
PACK_REL = "eech-campaign/reference/d3-entity-collision/physical-flags-production-move"
PACK = REPO / PACK_REL
RUNS = PACK / "runs"
CHECKPOINT = "cf1026a4c"
PREFIX = "flags-production-"
HELPER = "check-physical-flags-production-linux-20261107.sh"
HELPER_TEXT = (EXT / HELPER).read_text(encoding="utf-8")
IMAGE = re.search(r"^image=(\S+)$", HELPER_TEXT, flags=re.M).group(1)
VOLUME = re.search(r"^volume=(\S+)$", HELPER_TEXT, flags=re.M).group(1)
ROOT_LEASED = {"image": "sha256:9d247b4ea15457f06579ef703ce287e96358128e7a2217f2833ee8b628fc85c3", "volume": "eech-linux-d3-physical-flags-9d247-20261107"}  # the names the task (Root) leased: asserted against the helper, never the source of the names
SEALED = {
    "independent-physical-flags-consumer-753ee-20261107.json": SEALED_SHA["review"],
    "physical-flags-consumer-753-Root-binding-20261107.json": SEALED_SHA["binding"],
    "physical-flags-consumer-ad029-publication-Root-20261107.json": SEALED_SHA["publication"],
}
PRE_UTC = (EXT / "hwprod-PRE-utc-20261107.txt").read_text().strip()
RETAINED = json.loads((EXT / "hwprod-retained-dev-runs-20261107.json").read_text(encoding="utf-8"))
RETAINED = {label: (v["actual_exit"], v["why"]) for label, v in RETAINED.items()}
FINAL = ["p1-fmt-check", "p2-clippy-engine-sys-default", "p3-clippy-engine-sys-flag-features", "p4-clippy-world-engine-dc", "p5-engine-sys-default", "p6-flags-hosted-controls", "p7-flags-original-profile", "p8-sector-height-hosted", "p9-sector-height-original", "p10-build-dc", "p11-world-all", "p12-engine-dc"]
READERS = ["run-physical-flags-production-direct-20261107.py", HELPER, "run-hwprod-final-set-20261107.sh", "hwprod-join-20261107.py", "hwprod-extra-identity-20261107.py", "freeze-hwprod-20261107.py", "sector-roster-inputmap-batched-20261106.py", "sho-compare-maps-20261107.py", "preflight-receipts-20261107.py"]
SOURCES_EXCLUDED_PREFIXES = (PACK_REL + "/", "eech-campaign/reference/", "STATUS.md", "docs/handover/")
# the tests each final run must have actually run (a target fragment and a minimum count of passing tests), and the control reports each must print
MIN_TARGETS = {
    "p5-engine-sys-default": [("tests/physical_flags_guard.rs", 5), ("tests/saved_campaign_load_guard.rs", 3), ("tests/sector_height_guard.rs", 7)],
    "p6-flags-hosted-controls": [("tests/physical_flags.rs", 1), ("tests/physical_flags_consumer.rs", 1), ("tests/physical_flags_hosted_faults.rs", 1), ("tests/physical_flags_guard.rs", 5), ("tests/entity_dialogue.rs", 1)],
    "p7-flags-original-profile": [("tests/physical_flags.rs", 1), ("tests/physical_flags_consumer.rs", 1), ("tests/physical_flags_guard.rs", 5)],
    "p8-sector-height-hosted": [("tests/sector_height.rs", 1), ("tests/sector_height_guard.rs", 7)],
    "p9-sector-height-original": [("tests/sector_height.rs", 1), ("tests/sector_height_guard.rs", 7)],
    "p11-world-all": [("tests/physical_flags_host.rs", 7), ("tests/entity_authority.rs", 119), ("tests/load_dc.rs", 1)],
    "p12-engine-dc": [("src/lib.rs", 190)],
}
MUST_PRINT = {
    "p6-flags-hosted-controls": ["physical flags expectations made: 199", "physical flags consumer expectations made: 56", "host traffic (groups, sets, reads, resets, fresh bindings)", "entity dialogue expectations made: "],
    "p7-flags-original-profile": ["physical flags expectations made: 199", "physical flags consumer expectations made: 56"],
    "p8-sector-height-hosted": ["sector height expectations made: 150"],
    "p9-sector-height-original": ["sector height expectations made: 221"],
}


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def git_bytes(*args):
    return subprocess.run(["git", "-C", str(REPO)] + list(args), capture_output=True, check=True).stdout


def git(*args):
    return git_bytes(*args).decode().strip()


def run(*args):
    return subprocess.run(args, cwd=str(REPO), capture_output=True, text=True)


problems = []
readers_pre = json.loads((EXT / "hwprod-readers-PRE-20261107.json").read_text(encoding="utf-8"))
for name in READERS:
    if readers_pre["readers"].get(name) != sha(EXT / name):
        problems.append("an audit reader differs from its PRE identity (edited after the PRE capture): " + name)
head = git("rev-parse", "HEAD")
if not head.startswith(CHECKPOINT):
    problems.append("HEAD is not the checkpoint: " + head)
for name, want in SEALED.items():
    if sha(EXT / name) != want:
        problems.append("sealed input differs: " + name)
feedback_files = sorted(p.name for p in EXT.glob("Root-*feedback*"))
production_feedback = sorted(p.name for p in EXT.glob("Root-physical-flags-production-feedback*.json"))
unread_feedback = sorted(set(production_feedback) - set(FEEDBACK_READ))
if unread_feedback:
    problems.append("a production feedback file arrived that this batch has not read: %s" % unread_feedback)

# the run records, paths DERIVED from what is on disk (the pre-flight: every record has its stdout and stderr, is terminal and used the declared helper)
preflight_out = EXT / "hwprod-preflight-receipts-final-20261107.json"
if preflight_out.exists():
    preflight_out.unlink()
preflight = run(PY, "-I", "-B", "-X", "utf8", str(EXT / "preflight-receipts-20261107.py"), PREFIX, r"^(p|d)\d+-", HELPER, str(preflight_out))
if preflight.returncode != 0:
    problems.append("the receipt pre-flight failed: " + preflight.stdout.strip())
RUNS.mkdir(parents=True, exist_ok=True)
(PACK / ".gitattributes").write_bytes((REPO / "eech-campaign/reference/d3-entity-collision/sector-height-proof-guards-reviewed/.gitattributes").read_bytes())  # `* -text` BEFORE the first git add
runs = []
batch_records = [p for p in EXT.glob(PREFIX + "*.json") if re.match(r"^[pd]\d+-", p.name[len(PREFIX):])]
for record_path in sorted(batch_records, key=lambda p: (p.name[len(PREFIX)], int(re.match(r"[pd](\d+)", p.name[len(PREFIX):]).group(1)), p.name)):
    name = record_path.name[len(PREFIX):-len(".json")]
    for suffix in ("json", "stdout.log", "stderr.log"):
        shutil.copyfile(EXT / ("%s%s.%s" % (PREFIX, name, suffix)), RUNS / ("%s.%s" % (name, suffix)))
    record = json.loads(record_path.read_text(encoding="utf-8"))
    if record.get("state") != "terminal":
        problems.append("run not terminal: " + name)
    text = (RUNS / (name + ".stdout.log")).read_text(encoding="utf-8", errors="replace")
    passed = sum(int(m) for m in re.findall(r"test result: \w+\. (\d+) passed", text))
    failed = sum(int(m) for m in re.findall(r"; (\d+) failed", text))
    final = bool(re.match(r"^p\d+-", name))
    # cargo prints `Running <target>` on stderr and `test result:` on stdout, one result per target in the same order: pair them, and refuse a passing run where they do not pair
    err_text = (RUNS / (name + ".stderr.log")).read_text(encoding="utf-8", errors="replace")
    running = re.findall(r"^\s+(?:Running|Doc-tests) (?:unittests )?(\S+)", err_text, flags=re.M)
    results = [int(m) for m in re.findall(r"^test result: \w+\. (\d+) passed", text, flags=re.M)]
    targets = {}
    if running and len(running) == len(results):
        for target, count in zip(running, results):
            targets[target] = targets.get(target, 0) + count
    elif running or results:
        targets = {"UNPAIRED": len(results)}
        if record["actual_exit"] == 0:
            problems.append("the test targets and results of a passing run do not pair: " + name)
    runs.append({"label": name, "final": final, "command": record["command"], "actual_exit": record["actual_exit"], "started_utc": record["started_utc"], "completed_utc": record.get("completed_utc"), "tests_passed": passed, "tests_failed": failed,
                 "targets": targets, "stdout_sha256": sha(RUNS / (name + ".stdout.log")), "stderr_sha256": sha(RUNS / (name + ".stderr.log"))})
    expected = RETAINED.get(name, (0, ""))[0]
    if record["actual_exit"] != expected:
        problems.append("unexpected exit for %s: %s (expected %s)" % (name, record["actual_exit"], expected))
final_runs = [r for r in runs if r["final"]]
if sorted(r["label"] for r in final_runs) != sorted(FINAL) or any(r["actual_exit"] != 0 or r["tests_failed"] != 0 for r in final_runs):
    problems.append("the final set is not p1..p12 all exit 0 with no failed test: %s" % [(r["label"], r["actual_exit"], r["tests_failed"]) for r in final_runs])
by_label = {r["label"]: r for r in final_runs}
for label in FINAL[4:11] + FINAL[11:]:
    if label == "p10-build-dc":
        continue
    if by_label.get(label, {}).get("tests_passed", 0) <= 0:
        problems.append("a final test run selected no test: " + label)
for label, wants in MIN_TARGETS.items():
    for fragment, minimum in wants:
        got = sum(v for k, v in by_label.get(label, {}).get("targets", {}).items() if fragment in k)
        if got < minimum:
            problems.append("%s: target %s ran %d tests (expected at least %d)" % (label, fragment, got, minimum))
report_lines = {}
for label, needles in MUST_PRINT.items():
    out_text = (RUNS / (label + ".stdout.log")).read_text(encoding="utf-8", errors="replace")
    for needle in needles:
        if needle not in out_text:
            problems.append("%s did not print: %s" % (label, needle))
        else:
            line = next(l for l in out_text.splitlines() if needle in l)
            report_lines.setdefault(label, []).append(line)
dialogue_checks = [int(m) for m in re.findall(r"entity dialogue expectations made: (\d+)", (RUNS / "p6-flags-hosted-controls.stdout.log").read_text(encoding="utf-8", errors="replace"))]
if not dialogue_checks or dialogue_checks[0] < 1130:
    problems.append("the dialogue control made fewer than 1130 expectations: %s" % dialogue_checks)
fault_lines = [l for l in (RUNS / "p6-flags-hosted-controls.stdout.log").read_text(encoding="utf-8", errors="replace").splitlines() if "expectations failed" in l]
if len(fault_lines) != 4:
    problems.append("the negative controls did not report four wrong hosts: %s" % fault_lines)
if IMAGE != ROOT_LEASED["image"] or VOLUME != ROOT_LEASED["volume"]:
    problems.append("the helper's volume/image are not the leased ones: %s %s" % (VOLUME, IMAGE))
helper_in_runs = [json.loads(p.read_text(encoding="utf-8")).get("args", []) for p in RUNS.glob("*.json") if re.match(r"^[pd]\d+-", p.name)]
if not helper_in_runs or any(not any(HELPER in a for a in args) for args in helper_in_runs):
    problems.append("a direct receipt does not use the production helper")
progress = (EXT / "hwprod-final-set-20261107.progress").read_text()
exits = dict(re.findall(r"^(\S+) exit=(\d+)$", progress, flags=re.M))
if "DONE" not in progress.split() or sorted(exits) != sorted(FINAL) or any(v != "0" for v in exits.values()):
    problems.append("the progress file is not twelve children all exit 0 then DONE: %s" % exits)
for name in ("hwprod-final-set-20261107.progress", "hwprod-final-set-lease-before-20261107.txt", "hwprod-final-set-lease-after-20261107.txt", "hwprod-preflight-receipts-20261107.json", "hwprod-preflight-receipts-final-20261107.json"):
    shutil.copyfile(EXT / name, RUNS / name)
lease_before = (EXT / "hwprod-final-set-lease-before-20261107.txt").read_text().split()
lease_after = (EXT / "hwprod-final-set-lease-after-20261107.txt").read_text().strip()
if lease_before[0] != IMAGE or "containers_before=0" not in lease_before or lease_after != "containers_after=0":
    problems.append("the lease facts are not the named image with no container before or after: %s / %s" % (lease_before, lease_after))
counts = {r["label"]: r["tests_passed"] for r in final_runs}

compare_out = EXT / "hwprod-inputmaps-compare-20261107.json"
join_out = EXT / "hwprod-join-vs-753ee-20261107.json"
for p in (compare_out, join_out):
    if p.exists():
        p.unlink()
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "sho-compare-maps-20261107.py"), str(EXT / "hwprod-inputmap-PRE-20261107.json"), str(EXT / "hwprod-inputmap-POST-20261107.json"),
    str(EXT / "hwprod-extra-identity-PRE-20261107.json"), str(EXT / "hwprod-extra-identity-POST-20261107.json"), str(compare_out))
compare = json.loads(compare_out.read_text(encoding="utf-8"))
if not compare["all_rows_equal"] or not compare["extra_identity_equal"]:
    problems.append("PRE and POST maps differ")
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "hwprod-join-20261107.py"), str(REPO), str(EXT / "hwcons-inputmap-POST-20261107.json"), str(EXT / "hwprod-inputmap-POST-20261107.json"), str(join_out))
join = json.loads(join_out.read_text(encoding="utf-8"))
if join["problems"]:
    problems.append("the join has problems: %s" % join["problems"])
previous_compare = json.loads((EXT / "hwcons-inputmaps-compare-20261107.json").read_text(encoding="utf-8"))
if compare["raw_vs_git_filtered_disagreements"]["post"] != previous_compare["raw_vs_git_filtered_disagreements"]["post"]:
    problems.append("the raw-versus-Git qualification changed against the reference batch")
EXPECTED_SOURCES = set(join["map_rows_changed"]) | set(join["map_rows_added"]) | {"eech-campaign/docs/d3-physical-flags-contract.md"}
shutil.copyfile(compare_out, PACK / "inputmaps-compare.json")
shutil.copyfile(join_out, PACK / "join-vs-reference.json")
maps = {name: {"bytes": (EXT / name).stat().st_size, "sha256": sha(EXT / name), "location": "external (full per-file rows)"}
        for name in ("hwprod-inputmap-PRE-20261107.json", "hwprod-inputmap-POST-20261107.json", "hwprod-extra-identity-PRE-20261107.json", "hwprod-extra-identity-POST-20261107.json", "hwcons-inputmap-POST-20261107.json")}

# the stored bytes are what Git actually holds: stage the changed sources and compare every literal claim against the index blob bytes
porcelain = subprocess.check_output(["git", "-C", str(REPO), "status", "--porcelain", "--untracked-files=all"], text=True).splitlines()
source_paths = [l[3:] for l in porcelain if not l[3:].startswith(SOURCES_EXCLUDED_PREFIXES)]
if set(source_paths) != EXPECTED_SOURCES:
    problems.append("the changed source set is not the declared one: %s" % sorted(set(source_paths) ^ EXPECTED_SOURCES))
subprocess.run(["git", "-C", str(REPO), "add", "--"] + source_paths, check=True)
source_literal = {}
for path in source_paths:
    raw = (REPO / path).read_bytes()
    blob = git_bytes("cat-file", "blob", ":" + path)
    source_literal[path] = {"working_bytes_equal_git_index_blob_bytes": raw == blob, "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest(), "crlf_lines": raw.count(b"\r\n")}
    if raw != blob:
        problems.append("the changed source is not literally what Git holds: " + path)
    if raw.count(b"\r\n"):
        problems.append("the changed source holds CRLF lines: " + path)
old_packets_unchanged = {}
for rel in ("eech-campaign/reference/d3-entity-collision/physical-flags-consumer-control", "eech-campaign/reference/d3-entity-collision/physical-flags-consumer-753-reviewed", "eech-campaign/reference/d3-entity-collision/physical-flags-characterization", "eech-campaign/reference/d3-entity-collision/sector-height-proof-guards-reviewed"):
    unchanged = subprocess.run(["git", "-C", str(REPO), "diff", "--quiet", "HEAD", "--", rel]).returncode == 0
    old_packets_unchanged[rel] = unchanged
    if not unchanged:
        problems.append("an older packet changed: " + rel)

guards = {}
checks = [("generic", []), ("production-cutover", ["--action", "production-cutover"]), ("deliver (expected unmet)", ["--action", "deliver"])]
for key, args in checks:
    guards[key] = run(PY, "-B", "-X", "utf8", "eech-campaign/tools/migration-progress.py", *args).returncode

scripts = PACK / "scripts"
scripts.mkdir(exist_ok=True)
script_rows = []
for name in READERS + ["hwprod-retained-dev-runs-20261107.json", "hwprod-feedback-files-20261107.json", "hwprod-readers-PRE-20261107.json", "hwprod-inputmap-DRY-20261107.json", "hwprod-join-DRY-20261107.json"]:
    shutil.copyfile(EXT / name, scripts / name)
    script_rows.append({"name": name, "sha256": sha(scripts / name)})
for name in sorted(FEEDBACK_READ):
    shutil.copyfile(EXT / name, PACK / name)
    FEEDBACK_READ[name] = sha(PACK / name)
subprocess.run(["git", "-C", str(REPO), "add", "-A", PACK_REL], check=True)

freeze = {
    "kind": "D3 PHYSICAL FLAGS PRODUCTION MOVE CODE_FROZEN (UNREVIEWED CANDIDATE; native UNRUN): the complete physical UPDATED / MOVED / ROTATED store (three arrays, TRUE-only set, destructive raw read, quarter-memset resets) moves from EECH_DC into its host as ONE responsibility in the hosted profile; the original profile is unchanged",
    "frozen_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "task": "EEDCS-49",
    "stage": "Root's authorised next bounded production implementation after the accepted characterisation (review ce3bdb21): a candidate and its validation; NOT generic cutover, native, host-authority or delivery acceptance; preparation count stays 2; no supply-descriptor repair, no boolean normalisation, no second Core store",
    "source_before": head,
    "native": "this batch launched NO native job, no Windows build, no Tacview recording and no retail run; the protected current shipping runtime (Root's 764 full 40/all 8 job) is contextual only and was not touched",
    "feedback_files": {"at_start": FEEDBACK_AT_START, "at_freeze": production_feedback, "read_before_final_PRE_and_freeze": {n: {"sha256_now": h} for n, h in FEEDBACK_READ.items()}, "unread_new_files_at_freeze": unread_feedback},
    "sealed_inputs": {name: {"sha256": sha(EXT / name)} for name in SEALED},
    "root_feedback_files_present": feedback_files,
    "lease": {"asserted_from_the_helper_text": {"helper": HELPER, "helper_sha256": sha(EXT / HELPER), "volume": VOLUME, "image": IMAGE}, "root_leased_names": ROOT_LEASED, "names_equal": VOLUME == ROOT_LEASED["volume"] and IMAGE == ROOT_LEASED["image"], "warm_volume_reused": True,
              "containers_before": lease_before, "containers_after": lease_after, "cache_cleanup": "none",
              "tooling_note": "rustfmt and clippy were installed into each EPHEMERAL container by `rustup component add`; the image is the explicitly retained 9d247 image, measured in the runs, not the Root 3a3 Windows producer; old physical-flags-* helpers, receipts, freezes and maps were not overwritten"},
    "change": {
        "moved": "mb_int.c setter and getter and up_update.c resets and arrays: in the hosted profile the three arrays are not defined or declared and every set, destructive read and reset is a question to the host's store (entity_scan phases 19 to 22)",
        "kept_in_core": "update scheduling and iteration, the set after each member's own update, every consumer call and decision, the heap-slot to group-and-lane derivation and one key table of identities (no bit, mask or word)",
        "host": "eech-world flags.rs (the store), model_host.rs (flag_answer, the one function the provider and the controls call; flag_event), roster.rs (the field; clear never touches it); wire: sys constants, eech-engine ScanDecision::Flag*, eech-dc validator and parse; the public entity-authority.lua driver passes the four phases through with its own grammar and reply checks",
        "profiles": "hosted (default) and the TRUE original (collision-fixture), the SAME 199 and 56 scenarios in both; the original profile compiles the reference's code (join)",
    },
    "red_green": {"red": "none manufactured: the move is a structural change with the accepted characterisation as its sociable tests; the matched NEGATIVE controls (four well-formed but wrong hosts fail the same 199 scenarios; every malformed answer stops the engine by marker) are the sensitivity evidence; one ineffective proposed fault (a fresh binding keeping the old store) is retained as a failed attempt and not claimed", "green": ["p5..p12"]},
    "test_counts_final": counts,
    "control_reports": report_lines,
    "negative_controls": fault_lines,
    "retained_development_runs": {label: {"actual_exit": code, "why": why} for label, (code, why) in RETAINED.items()},
    "runs": runs,
    "guards": guards,
    "input_maps": {"files": maps, "compare_pre_post": compare, "pre_utc": PRE_UTC, "post_utc": POST_UTC, "post_utc_note": "recorded by `date -u` immediately before the POST capture; not reconstructed from a file time", "join_vs_reference": join},
    "literal_bytes": {"changed_sources": source_literal, "older_packets_unchanged_against_HEAD": old_packets_unchanged},
    "compiled_input_change": {"statement": "the production move changes the campaign C in six files; the ORIGINAL profile's view of the three files that carry it has the reference's code (verified in the join), the other three only add lines, every control source is inside its feature gate and the Rust and driver files are the declared set", "compiled_rows_changed": join["compiled_rows_changed"], "compiled_rows_added": join["compiled_rows_added"], "unchanged_compiled_rows": join["unchanged_compiled_rows"], "unchanged_compiled_aggregate_sha256": join["unchanged_compiled_aggregate_sha256"]},
    "audit_reader_timing": "this freeze script, the join, the extra-identity script, the final-set runner, the runner, the check script, the batched map script, the comparer and the receipt pre-flight were FINAL before the PRE identity capture (their SHA-256 recorded at PRE and verified unchanged at freeze)",
    "scripts": script_rows,
    "unrun_gates": [
        "the public host-owned Lua -> native EECH_DC module -> host path for the flags (the chain is shown in parts: the REAL C against the REAL store through the engine's own request and reply structs; the REAL provider behind the REAL eech-dc bridge; the public entity-authority.lua driver over the real provider with a scripted engine)",
        "native, Windows, retail, Tacview, full-pack, 764-class or current 40/all 8 runs and the four inputs; the windows-cross and i686 builds of the changed C",
        "the other consumers (en_cllsn.c moving-mobile pass-through, the supply descriptor, the Tacview writers, the weapon-vector path, the helicopter, vehicle and ship responses) through the host",
        "the real session lifetime and a host replaced inside one process (a key the new host never issued is refused by name)",
        "the cost of a provider call per entity per update iteration (not measured); whole-FPU-state preservation (not measured)",
        "DCS: no live-DCS claim; the needed-API disposition is only the host's own implementation of the three words and their operations, nothing geometric",
    ],
    "limits": [
        "the hosted controls' fresh-binding start and host-word observation replace the original's array clear and array read (csrc/eech_physical_flags_hosted.h); the expectations are the original's text",
        "private diagnostic traffic counts are reported, not pinned",
        "the supply-descriptor mismatch stays a source-derived UNCONFIRMED, UNFIXED candidate; the original &= and quarter-memset are reproduced, not fixed",
        "fmt and clippy ran only because the components were installed into each ephemeral container of the retained image",
    ],
    "state": {"claims": "no self-review, approval, native clearance, finding closure, whole Task49 or D3, delivery", "D3-CURRENT.json": "not edited (Root owns it)"},
    "hygiene_problems_at_freeze": problems,
}
FREEZE.write_text(json.dumps(freeze, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
shutil.copyfile(FREEZE, PACK / FREEZE.name)
files = [{"path": str(p.relative_to(PACK)).replace("\\", "/"), "bytes": p.stat().st_size, "sha256": sha(p)} for p in sorted(PACK.rglob("*")) if p.is_file() and p.name != "manifest.json"]
(PACK / "manifest.json").write_text(json.dumps({"files": files, "scope": "Source and local-check packet of the physical flags production move candidate only; not a native result, review, approval or acceptance."}, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps({"problems": problems, "runs": len(runs), "final": counts, "files": len(files), "guards": guards, "sources": len(source_paths)}, indent=1))
sys.exit(1 if problems else 0)
