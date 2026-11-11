"""Freezes the KEY-FIX batch packet (the two source findings of the consolidated review V2 of the production candidate 0d0c2545b: ENV-PHYSICAL-FLAGS-KEY-CAPACITY-1 and ENV-PHYSICAL-FLAGS-ISSUED-GROUP-ALIAS-1, the accurate lifetime wording and the collision consumer
control; candidate, UNREVIEWED, native UNRUN) ONCE. Usage: freeze.py <repo-root> <external-freeze-path> <post-utc>
This script and every reader it names (the final-set runner, the join and its derivation, the extra-identity script, the runner, the check script, the batched map script, the comparer and the receipt pre-flight) are FINAL before the PRE identity capture: their SHA-256 are
recorded at PRE (hwkey-readers-PRE-20261107.json) and this script REFUSES to freeze if any of them differs now (a deviation is recorded as an erratum, never hidden). Every receipt path is DERIVED from the run records on disk (preflight-receipts-20261107.py ran on the
development runs before PRE and runs again here), never typed as a prefix. Root's key-fix feedback files are read AT THE ACTUAL TIME: the files present at the freeze are listed and a file this script has not been told was read is a problem."""
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
    "review_v2": "0269234c4559916934dba5b2bfbc4661696bf4f517c6abe6e7a607b3eb1c6587",
    "review_v1": "829f196e5d2ec89afc311d312af938914b06702a1a10f770ff945c53f0b619d5",
    "binding": "5e85f0baf0c64ec0f3aa5588e0363a4f890f4c6e70f8b557fc9239121757091a",
    "publication": "8f580428d3ebd355b2d4c05c5331b4f858fe04d7e2202fe64b6282720568c00a",
}
FEEDBACK_AT_START = []  # no Root-physical-flags-key-fix-feedback*.json existed when this batch began (checked after the metadata checkpoint 9c63a0967)
# the files that were read in full, with the time they were read; the freeze refuses a file present that is not listed here
FEEDBACK_READ = json.loads((EXT / "hwkey-feedback-files-20261107.json").read_text(encoding="utf-8"))["read"]
PY = "C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
PACK_REL = "eech-campaign/reference/d3-entity-collision/physical-flags-key-fix"
PACK = REPO / PACK_REL
RUNS = PACK / "runs"
CHECKPOINT = "9c63a0967"
PREFIX = "flags-keyfix-"
HELPER = "check-physical-flags-keyfix-linux-20261107.sh"
HELPER_TEXT = (EXT / HELPER).read_text(encoding="utf-8")
IMAGE = re.search(r"^image=(\S+)$", HELPER_TEXT, flags=re.M).group(1)
VOLUME = re.search(r"^volume=(\S+)$", HELPER_TEXT, flags=re.M).group(1)
ROOT_LEASED = {"image": "sha256:9d247b4ea15457f06579ef703ce287e96358128e7a2217f2833ee8b628fc85c3", "volume": "eech-linux-d3-physical-flags-9d247-20261107"}  # the names the task (Root) leased: asserted against the helper, never the source of the names
SEALED = {
    "independent-physical-flags-production-0d0-source-20261107-v2.json": SEALED_SHA["review_v2"],
    "independent-physical-flags-production-0d0-source-20261107.json": SEALED_SHA["review_v1"],
    "physical-flags-production-0d0-Root-binding-20261107.json": SEALED_SHA["binding"],
    "physical-flags-production-c126-publication-Root-20261107.json": SEALED_SHA["publication"],
}
PRE_UTC = (EXT / "hwkey-PRE-utc-20261107.txt").read_text().strip()
RETAINED = json.loads((EXT / "hwkey-retained-dev-runs-20261107.json").read_text(encoding="utf-8"))
RETAINED = {label: (v["actual_exit"], v["why"]) for label, v in RETAINED.items()}
FINAL = ["q1-fmt-check", "q2-clippy-engine-sys-default", "q3-clippy-engine-sys-flag-features", "q4-clippy-engine-sys-original", "q5-clippy-world-engine-dc", "q6-engine-sys-default", "q7-flags-hosted-controls", "q8-flags-original-profile",
         "q9-sector-height-hosted", "q10-sector-height-original", "q11-build-dc", "q12-world-all", "q13-engine-dc"]
READERS = ["run-physical-flags-keyfix-direct-20261107.py", HELPER, "run-hwkey-final-set-20261107.sh", "hwkey-join-20261107.py", "make-hwkey-join-20261107.py", "hwkey-extra-identity-20261107.py", "freeze-hwkey-20261107.py",
           "sector-roster-inputmap-batched-20261106.py", "sho-compare-maps-20261107.py", "preflight-receipts-20261107.py"]
SOURCES_EXCLUDED_PREFIXES = (PACK_REL + "/", "eech-campaign/reference/", "STATUS.md", "docs/handover/")
MIN_TARGETS = {
    "q6-engine-sys-default": [("tests/physical_flags_guard.rs", 6), ("tests/saved_campaign_load_guard.rs", 3), ("tests/sector_height_guard.rs", 7)],
    "q7-flags-hosted-controls": [("tests/physical_flags.rs", 1), ("tests/physical_flags_consumer.rs", 1), ("tests/physical_flags_hosted_faults.rs", 1), ("tests/physical_flags_identity.rs", 1), ("tests/physical_flags_collision.rs", 1), ("tests/physical_flags_guard.rs", 6), ("tests/entity_dialogue.rs", 1)],
    "q8-flags-original-profile": [("tests/physical_flags.rs", 1), ("tests/physical_flags_consumer.rs", 1), ("tests/physical_flags_collision.rs", 1), ("tests/physical_flags_guard.rs", 6)],
    "q9-sector-height-hosted": [("tests/sector_height.rs", 1), ("tests/sector_height_guard.rs", 7)],
    "q10-sector-height-original": [("tests/sector_height.rs", 1), ("tests/sector_height_guard.rs", 7)],
    "q12-world-all": [("tests/physical_flags_host.rs", 7), ("tests/entity_authority.rs", 120), ("tests/load_dc.rs", 1)],
    "q13-engine-dc": [("src/lib.rs", 195)],
}
MUST_PRINT = {
    "q7-flags-hosted-controls": ["physical flags expectations made: 199", "physical flags consumer expectations made: 56", "physical flags identity expectations made: 85", "physical flags collision expectations made: 16",
                                 "host traffic (groups, sets, reads, resets, fresh bindings)", "entity dialogue expectations made: "],
    "q8-flags-original-profile": ["physical flags expectations made: 199", "physical flags consumer expectations made: 56", "physical flags collision expectations made: 15"],
    "q9-sector-height-hosted": ["sector height expectations made: 150"],
    "q10-sector-height-original": ["sector height expectations made: 221"],
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
readers_pre = json.loads((EXT / "hwkey-readers-PRE-20261107.json").read_text(encoding="utf-8"))
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
key_feedback = sorted(p.name for p in EXT.glob("Root-physical-flags-key-fix-feedback*.json"))
unread_feedback = sorted(set(key_feedback) - set(FEEDBACK_READ))
if unread_feedback:
    problems.append("a key-fix feedback file is present that this batch has not read: %s" % unread_feedback)
for name, read_row in FEEDBACK_READ.items():
    if sha(EXT / name) != read_row["sha256"]:
        problems.append("a feedback file changed after it was read: " + name)

preflight_out = EXT / "hwkey-preflight-receipts-final-20261107.json"
if preflight_out.exists():
    preflight_out.unlink()
preflight = run(PY, "-I", "-B", "-X", "utf8", str(EXT / "preflight-receipts-20261107.py"), PREFIX, r"^[qd]\d+-", HELPER, str(preflight_out))
if preflight.returncode != 0:
    problems.append("the receipt pre-flight failed: " + preflight.stdout.strip())
RUNS.mkdir(parents=True, exist_ok=True)
(PACK / ".gitattributes").write_bytes((REPO / "eech-campaign/reference/d3-entity-collision/sector-height-proof-guards-reviewed/.gitattributes").read_bytes())  # `* -text` BEFORE the first git add
runs = []
batch_records = [p for p in EXT.glob(PREFIX + "*.json") if re.match(r"^[qd]\d+-", p.name[len(PREFIX):])]
for record_path in sorted(batch_records, key=lambda p: (p.name[len(PREFIX)], int(re.match(r"[qd](\d+)", p.name[len(PREFIX):]).group(1)), p.name)):
    name = record_path.name[len(PREFIX):-len(".json")]
    for suffix in ("json", "stdout.log", "stderr.log"):
        shutil.copyfile(EXT / ("%s%s.%s" % (PREFIX, name, suffix)), RUNS / ("%s.%s" % (name, suffix)))
    record = json.loads(record_path.read_text(encoding="utf-8"))
    if record.get("state") != "terminal":
        problems.append("run not terminal: " + name)
    text = (RUNS / (name + ".stdout.log")).read_text(encoding="utf-8", errors="replace")
    passed = sum(int(m) for m in re.findall(r"test result: \w+\. (\d+) passed", text))
    failed = sum(int(m) for m in re.findall(r"; (\d+) failed", text))
    final = bool(re.match(r"^q\d+-", name))
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
    problems.append("the final set is not q1..q13 all exit 0 with no failed test: %s" % [(r["label"], r["actual_exit"], r["tests_failed"]) for r in final_runs])
by_label = {r["label"]: r for r in final_runs}
for label in FINAL[5:]:
    if label == "q11-build-dc":
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
q7_text = (RUNS / "q7-flags-hosted-controls.stdout.log").read_text(encoding="utf-8", errors="replace")
dialogue_checks = [int(m) for m in re.findall(r"entity dialogue expectations made: (\d+)", q7_text)]
if not dialogue_checks or dialogue_checks[0] < 1149:
    problems.append("the dialogue control made fewer than 1149 expectations: %s" % dialogue_checks)
fault_lines = [l for l in q7_text.splitlines() if "expectations failed" in l]
if len(fault_lines) != 4:
    problems.append("the negative controls did not report four wrong hosts: %s" % fault_lines)
if IMAGE != ROOT_LEASED["image"] or VOLUME != ROOT_LEASED["volume"]:
    problems.append("the helper's volume/image are not the leased ones: %s %s" % (VOLUME, IMAGE))
helper_in_runs = [json.loads(p.read_text(encoding="utf-8")).get("args", []) for p in RUNS.glob("*.json") if re.match(r"^[qd]\d+-", p.name)]
if not helper_in_runs or any(not any(HELPER in a for a in args) for args in helper_in_runs):
    problems.append("a direct receipt does not use the key-fix helper")
progress = (EXT / "hwkey-final-set-20261107.progress").read_text()
exits = dict(re.findall(r"^(\S+) exit=(\d+)$", progress, flags=re.M))
if "DONE" not in progress.split() or sorted(exits) != sorted(FINAL) or any(v != "0" for v in exits.values()):
    problems.append("the progress file is not thirteen children all exit 0 then DONE: %s" % exits)
for name in ("hwkey-final-set-20261107.progress", "hwkey-final-set-lease-before-20261107.txt", "hwkey-final-set-lease-after-20261107.txt", "hwkey-preflight-receipts-20261107.json", "hwkey-preflight-receipts-final-20261107.json"):
    shutil.copyfile(EXT / name, RUNS / name)
lease_before = (EXT / "hwkey-final-set-lease-before-20261107.txt").read_text().split()
lease_after = (EXT / "hwkey-final-set-lease-after-20261107.txt").read_text().strip()
if lease_before[0] != IMAGE or "containers_before=0" not in lease_before or lease_after != "containers_after=0":
    problems.append("the lease facts are not the named image with no container before or after: %s / %s" % (lease_before, lease_after))
counts = {r["label"]: r["tests_passed"] for r in final_runs}

compare_out = EXT / "hwkey-inputmaps-compare-20261107.json"
join_out = EXT / "hwkey-join-vs-0d0-20261107.json"
for p in (compare_out, join_out):
    if p.exists():
        p.unlink()
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "sho-compare-maps-20261107.py"), str(EXT / "hwkey-inputmap-PRE-20261107.json"), str(EXT / "hwkey-inputmap-POST-20261107.json"),
    str(EXT / "hwkey-extra-identity-PRE-20261107.json"), str(EXT / "hwkey-extra-identity-POST-20261107.json"), str(compare_out))
compare = json.loads(compare_out.read_text(encoding="utf-8"))
if not compare["all_rows_equal"] or not compare["extra_identity_equal"]:
    problems.append("PRE and POST maps differ")
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "hwkey-join-20261107.py"), str(REPO), str(EXT / "hwprod-inputmap-POST-20261107.json"), str(EXT / "hwkey-inputmap-POST-20261107.json"), str(join_out))
join = json.loads(join_out.read_text(encoding="utf-8"))
if join["problems"]:
    problems.append("the join has problems: %s" % join["problems"])
previous_compare = json.loads((EXT / "hwprod-inputmaps-compare-20261107.json").read_text(encoding="utf-8"))
if compare["raw_vs_git_filtered_disagreements"]["post"] != previous_compare["raw_vs_git_filtered_disagreements"]["post"]:
    problems.append("the raw-versus-Git qualification changed against the reference batch")
EXPECTED_SOURCES = set(join["map_rows_changed"]) | set(join["map_rows_added"]) | {"eech-campaign/docs/d3-physical-flags-contract.md"}
shutil.copyfile(compare_out, PACK / "inputmaps-compare.json")
shutil.copyfile(join_out, PACK / "join-vs-reference.json")
maps = {name: {"bytes": (EXT / name).stat().st_size, "sha256": sha(EXT / name), "location": "external (full per-file rows)"}
        for name in ("hwkey-inputmap-PRE-20261107.json", "hwkey-inputmap-POST-20261107.json", "hwkey-extra-identity-PRE-20261107.json", "hwkey-extra-identity-POST-20261107.json", "hwprod-inputmap-POST-20261107.json")}

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
for rel in ("eech-campaign/reference/d3-entity-collision/physical-flags-production-move", "eech-campaign/reference/d3-entity-collision/physical-flags-production-0d0-reviewed", "eech-campaign/reference/d3-entity-collision/physical-flags-consumer-control",
            "eech-campaign/reference/d3-entity-collision/physical-flags-consumer-753-reviewed"):
    unchanged = subprocess.run(["git", "-C", str(REPO), "diff", "--quiet", "HEAD", "--", rel]).returncode == 0
    old_packets_unchanged[rel] = unchanged
    if not unchanged:
        problems.append("an older packet changed: " + rel)

guards = {}
checks = [("generic", []), ("production-cutover (expected refused)", ["--action", "production-cutover"]), ("deliver (expected unmet)", ["--action", "deliver"])]
for key, args in checks:
    guards[key] = run(PY, "-B", "-X", "utf8", "eech-campaign/tools/migration-progress.py", *args).returncode

scripts = PACK / "scripts"
scripts.mkdir(exist_ok=True)
script_rows = []
for name in READERS + ["hwkey-retained-dev-runs-20261107.json", "hwkey-feedback-files-20261107.json", "hwkey-readers-PRE-20261107.json", "hwkey-inputmap-DRY-20261107.json", "hwkey-join-DRY-20261107.json"]:
    shutil.copyfile(EXT / name, scripts / name)
    script_rows.append({"name": name, "sha256": sha(scripts / name)})
for name in sorted(FEEDBACK_READ):
    shutil.copyfile(EXT / name, PACK / name)
copies = PACK / "fixed-sources-at-the-red-run"
copies.mkdir(exist_ok=True)
for name in ("mb_hostentity.h", "mb_hostmodel.c"):
    shutil.copyfile(EXT / "keyfix-fixed-sources-at-the-red-run" / name, copies / name)
subprocess.run(["git", "-C", str(REPO), "add", "-A", PACK_REL], check=True)

freeze = {
    "kind": "D3 PHYSICAL FLAGS KEY-FIX CODE_FROZEN (UNREVIEWED CANDIDATE; native UNRUN): ENV-PHYSICAL-FLAGS-KEY-CAPACITY-1 and ENV-PHYSICAL-FLAGS-ISSUED-GROUP-ALIAS-1 corrected together, the lifetime wording bounded, the collision consumer control built in the true original profile and its hosted block observed",
    "frozen_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "task": "EEDCS-49",
    "stage": "ONE consolidated required-correctness batch for the SAME consolidated review V2; both findings stay OPEN until the reviewer closes them; preparation count stays 2; no supply-descriptor repair, no boolean normalisation, no second Core store, no epoch or transfer design",
    "source_before": head,
    "native": "this batch launched NO native job, no Windows build, no Tacview recording and no retail run; the protected current shipping runtime (Root's 764 full 40/all 8 job) is contextual only and was not touched",
    "feedback_files": {"at_start": FEEDBACK_AT_START, "read_in_full": FEEDBACK_READ, "at_freeze": key_feedback, "unread_new_files_at_freeze": unread_feedback},
    "sealed_inputs": {name: {"sha256": sha(EXT / name)} for name in SEALED},
    "root_feedback_files_present": feedback_files,
    "lease": {"asserted_from_the_helper_text": {"helper": HELPER, "helper_sha256": sha(EXT / HELPER), "volume": VOLUME, "image": IMAGE}, "root_leased_names": ROOT_LEASED, "names_equal": VOLUME == ROOT_LEASED["volume"] and IMAGE == ROOT_LEASED["image"], "warm_volume_reused": True,
              "containers_before": lease_before, "containers_after": lease_after, "cache_cleanup": "none",
              "tooling_note": "rustfmt and clippy were installed into each EPHEMERAL container by `rustup component add`; the image is the explicitly retained 9d247 image; no old helper, receipt, freeze or map was overwritten; the one hung development run (d14) was my own container, killed after inspecting its CPU time (exit 137)"},
    "change": {
        "key_capacity": "mb_hostentity.h flag_key_is_valid: the whole canonical domain (f, a first digit 1 to 9, further digits, at most 20 bytes before the NUL, the identity slot's size) before any copy; the host's FlagStore::index has the same bound; the generic 127-byte name domain is unchanged",
        "issued_group_alias": "mb_hostmodel.c flag_group_key: the host's answer is staged and compared with every key bound to another group before the cache entry is written; a bound key stops the engine by name; identity only",
        "lifetime": "one persistent SharedRoster/FlagStore binding per process is the supported lifetime; a replacement store is unsupported and unproved (a new store restarts fN numbering)",
        "collision_consumer": "csrc/eech_physical_flags_collision_check.c: the moving-mobile pass-through of get_line_of_sight_collision_entity, the SAME scenarios in both profiles: the TRUE original profile (15 expectations, real public function, real sector lists, real getters, real provider scene data) and the HOSTED profile (16: the 15 plus the matched negative of a mobile with no host identity), where each mobile is linked to the campaign's roster group and the host issues its identity and mover record through the private test collaborator bind_member; the real scan, candidate (asking for the UPDATED word), pose, narrow-phase and flag questions are the real provider's",
        "public_driver": "entity-authority.lua: flag keys are checked against the 20-byte canonical grammar and a key already issued for another group is a named reply fault",
    },
    "red_green": {"red": "the new dialogue and identity controls against the 0d0 sources: the dialogue control fails and the identity control aborts the process (a 21-byte key accepted and written past its 21-byte slot): development run d2-red-0d0-source, with the corrected sources of that moment kept in fixed-sources-at-the-red-run/ (mb_hostmodel.c later gained one test-hook gate)", "green": ["q1..q13"]},
    "test_counts_final": counts,
    "control_reports": report_lines,
    "negative_controls": fault_lines,
    "retained_development_runs": {label: {"actual_exit": code, "why": why} for label, (code, why) in RETAINED.items()},
    "runs": runs,
    "guards": guards,
    "input_maps": {"files": maps, "compare_pre_post": compare, "pre_utc": PRE_UTC, "post_utc": POST_UTC, "post_utc_note": "recorded by `date -u` immediately before the POST capture; not reconstructed from a file time", "join_vs_reference": join},
    "literal_bytes": {"changed_sources": source_literal, "older_packets_unchanged_against_HEAD": old_packets_unchanged},
    "compiled_input_change": {"statement": "the campaign C changes in two files only (the key validator and the staged group binding; the production move's own files are unchanged rows), every control source is inside its feature gate, and the Rust and driver files are the declared set (verified in the join)", "compiled_rows_changed": join["compiled_rows_changed"], "compiled_rows_added": join["compiled_rows_added"], "unchanged_compiled_rows": join["unchanged_compiled_rows"], "unchanged_compiled_aggregate_sha256": join["unchanged_compiled_aggregate_sha256"]},
    "audit_reader_timing": "this freeze script, the join and its derivation, the extra-identity script, the final-set runner, the runner, the check script, the batched map script, the comparer and the receipt pre-flight were FINAL before the PRE identity capture (their SHA-256 recorded at PRE and verified unchanged at freeze)",
    "scripts": script_rows,
    "unrun_gates": [
        "the public host-owned Lua -> native EECH_DC module -> host path for the flags (the chain is shown in parts only; none of the checks is that path)",
        "the hosted collision consumer is run through a small TEST bridge over the real reference provider (tests/support/lua_provider.rs), not the shipped Engine::boot bridge: it is not a host-owned Lua -> module -> host run",
        "native, Windows, retail, Tacview, full-pack, 764-class or current 40/all 8 runs and the four inputs; the windows-cross and i686 builds of the changed C",
        "the other consumers (supply descriptor, Tacview writers, weapon-vector path, helicopter, vehicle and ship responses) through the host",
        "the real session lifetime; a replacement store (unsupported and unproved); whole-FPU, x87-subnormal, i686, O2 and native guarantees (not measured); transactional rollback of Host mutations (not provided)",
        "DCS: no live-DCS claim; no fake DCS geometry",
    ],
    "limits": [
        "the controls' fresh-binding start and host-word observation replace the original's array clear and read; the expectations are the original's text",
        "the collision control's model data is the real provider's minimal installation with one record field (the scene's collision object) set to the original's -1, a real kind of scene whose body the original tests by bounding box",
        "the collision control's provider bridge is a small TEST bridge over the real reference provider, not the shipped Engine::boot bridge; the private test collaborator bind_member is the host's registration of a member as the reference host's bind_mobile does it",
        "the supply-descriptor mismatch stays a source-derived UNCONFIRMED, UNFIXED candidate; the original &= and quarter-memset are reproduced, not fixed",
    ],
    "state": {"claims": "no self-review, approval, native clearance, finding closure, whole Task49 or D3, delivery", "D3-CURRENT.json": "not edited (Root owns it)"},
    "hygiene_problems_at_freeze": problems,
}
FREEZE.write_text(json.dumps(freeze, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
shutil.copyfile(FREEZE, PACK / FREEZE.name)
files = [{"path": str(p.relative_to(PACK)).replace("\\", "/"), "bytes": p.stat().st_size, "sha256": sha(p)} for p in sorted(PACK.rglob("*")) if p.is_file() and p.name != "manifest.json"]
(PACK / "manifest.json").write_text(json.dumps({"files": files, "scope": "Source and local-check packet of the physical flags key-fix candidate only; not a native result, review, approval or acceptance."}, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps({"problems": problems, "runs": len(runs), "final": counts, "files": len(files), "guards": guards, "sources": len(source_paths)}, indent=1))
sys.exit(1 if problems else 0)
