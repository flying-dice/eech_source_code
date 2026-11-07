"""Freezes the load-order diagnostic correction packet (ENV-SECTOR-HEIGHT-NATIVE-LOAD-ORDER-DIAGNOSTIC-1) ONCE. Usage: freeze.py <repo-root> <external-freeze-path> <post-utc>
This script and every reader it names (the final-set runner, the join, the extra-identity script, the runner and the check script) are FINAL before the PRE identity capture: their SHA-256 are recorded at PRE
(hwload-readers-PRE-20261107.json) and this script REFUSES to freeze if any of them differs now (an earlier freeze's reader was edited after its POST; that cannot recur unnoticed; a deviation is recorded as an erratum, never hidden)."""
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
SEALED_SHA = {"binary": hashlib.sha256((EXT / "Root-height-native-load-binary-size-cause-20261107.json").read_bytes()).hexdigest(), "order": hashlib.sha256((EXT / "Root-height-native-load-restore-order-constraint-20261107.json").read_bytes()).hexdigest(), "reviewer": hashlib.sha256((EXT / "Root-height-native-load-reviewer-constraints-20261107.json").read_bytes()).hexdigest()}
CONSTRAINT_FILES_AT_START = ["Root-height-native-load-binary-size-cause-20261107.json", "Root-height-native-load-restore-order-constraint-20261107.json", "Root-height-native-load-reviewer-constraints-20261107.json"]  # the three files present when this batch began (all read and published)
PY = "C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
PACK_REL = "eech-campaign/reference/d3-entity-collision/sector-height-native-load-order-fix"
PACK = REPO / PACK_REL
RUNS = PACK / "runs"
CHECKPOINT = "44afa7a6a"
REVIEWED_SOURCE = "90e9547ee"
NATIVE = "1b89c0b0ab613d2f6a08ea02dd6612457efa8982"
IMAGE = "sha256:9d247b4ea15457f06579ef703ce287e96358128e7a2217f2833ee8b628fc85c3"
VOLUME = "eech-linux-d3-height-reference-9d247-20261107"
SEALED = {  # the three same-reviewer constraint records, copied literally into the follow-up packet at the first commit of this batch
    "Root-height-native-load-binary-size-cause-20261107.json": SEALED_SHA["binary"],
    "Root-height-native-load-restore-order-constraint-20261107.json": SEALED_SHA["order"],
    "Root-height-native-load-reviewer-constraints-20261107.json": SEALED_SHA["reviewer"],
}
PRE_UTC = (EXT / "hwload-PRE-utc-20261107.txt").read_text().strip()
# the development runs that did not end 0, kept literally, and why (everything else, including all final children u1..u3, exits 0)
RETAINED = json.loads((EXT / "hwload-retained-dev-runs-20261107.json").read_text(encoding="utf-8"))
RETAINED = {label: (v["actual_exit"], v["why"]) for label, v in RETAINED.items()}
FINAL = ["u1-fmt-check", "u2-clippy-world", "u3-world-driver-tests"]
READERS = ["run-height-reference-direct-20261107.py", "check-height-reference-linux-20261107.sh", "run-hwload-final-set-20261107.sh", "hwload-join-20261107.py", "hwload-extra-identity-20261107.py", "freeze-hwload-20261107.py", "sector-roster-inputmap-batched-20261106.py", "sho-compare-maps-20261107.py"]
SOURCES_EXCLUDED_PREFIXES = (PACK_REL + "/", "eech-campaign/reference/", "STATUS.md", "docs/handover/")


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def git_bytes(*args):
    return subprocess.run(["git", "-C", str(REPO)] + list(args), capture_output=True, check=True).stdout


def git(*args):
    return git_bytes(*args).decode().strip()


def run(*args):
    return subprocess.run(args, cwd=str(REPO), capture_output=True, text=True)


problems = []
readers_pre = json.loads((EXT / "hwload-readers-PRE-20261107.json").read_text(encoding="utf-8"))
FREEZE_SELF = "freeze-hwload-20261107.py"
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

RUNS.mkdir(parents=True, exist_ok=True)
(PACK / ".gitattributes").write_bytes((REPO / "eech-campaign/reference/d3-entity-collision/sector-height-proof-guards-reviewed/.gitattributes").read_bytes())  # `* -text` BEFORE the first git add
runs = []
batch_records = [p for p in EXT.glob("sector-height-witness-*.json") if re.match(r"^[tu]\d+[a-z]?-", p.name[len("sector-height-witness-"):])]
for record_path in sorted(batch_records, key=lambda p: (p.name[len("sector-height-witness-")], int(re.match(r"[tu](\d+)", p.name[len("sector-height-witness-"):]).group(1)), p.name)):
    name = record_path.name[len("sector-height-witness-"):-len(".json")]
    if not re.match(r"^[tu]\d+[a-z]?-", name):
        continue
    for suffix in ("json", "stdout.log", "stderr.log"):
        shutil.copyfile(EXT / ("sector-height-witness-%s.%s" % (name, suffix)), RUNS / ("%s.%s" % (name, suffix)))
    record = json.loads(record_path.read_text(encoding="utf-8"))
    if record.get("state") != "terminal":
        problems.append("run not terminal: " + name)
    text = (RUNS / (name + ".stdout.log")).read_text(encoding="utf-8", errors="replace")
    passed = sum(int(m) for m in re.findall(r"test result: \w+\. (\d+) passed", text))
    failed = sum(int(m) for m in re.findall(r"; (\d+) failed", text))
    final = bool(re.match(r"^u\d+-", name))
    # cargo prints `Running <target>` on stderr and `test result:` on stdout, one result per target in the same order: pair them, and refuse a run where they do not pair
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
    problems.append("the final set is not u1..u3 all exit 0 with no failed test: %s" % [(r["label"], r["actual_exit"], r["tests_failed"]) for r in final_runs])
by_label = {r["label"]: r for r in final_runs}
for label in FINAL[2:]:
    if by_label.get(label, {}).get("tests_passed", 0) <= 0:
        problems.append("a final test run selected no test: " + label)
for label, target_fragment, minimum in (("u3-world-driver-tests", "tests/sector_height_witness_driver.rs", 27), ("u3-world-driver-tests", "tests/host_callbacks.rs", 82), ("u3-world-driver-tests", "tests/entity_authority.rs", 116), ("u3-world-driver-tests", "tests/sector_height_baseline_driver.rs", 13)):
    got = sum(v for k, v in by_label.get(label, {}).get("targets", {}).items() if target_fragment in k)
    if got < minimum:
        problems.append("%s: target %s ran %d tests (expected at least %d)" % (label, target_fragment, got, minimum))
# a retained first attempt of the final set (none unless a retry occurred)
attempt1 = EXT / "hwload-attempt1"
attempt1_rows = []
if attempt1.is_dir():
    target = PACK / "runs-attempt1"
    target.mkdir(exist_ok=True)
    for source in sorted(attempt1.iterdir()):
        if source.stat().st_size > 200_000:
            # a full per-file input map: its identity (size and hash) is recorded, the file stays external
            attempt1_rows.append({"name": source.name, "bytes": source.stat().st_size, "sha256": sha(source), "location": "external " + str(source)})
            continue
        shutil.copyfile(source, target / source.name)
        attempt1_rows.append({"name": source.name, "sha256": sha(target / source.name)})
a1_progress = (attempt1 / "hwload-final-set-20261107.progress").read_text() if attempt1.is_dir() else ""
if attempt1.is_dir() and "exit=" not in a1_progress:
    problems.append("the retained first attempt has no progress")
progress = (EXT / "hwload-final-set-20261107.progress").read_text()
exits = dict(re.findall(r"^(\S+) exit=(\d+)$", progress, flags=re.M))
if "DONE" not in progress.split() or sorted(exits) != sorted(FINAL) or any(v != "0" for v in exits.values()):
    problems.append("the progress file is not ten children all exit 0 then DONE: %s" % exits)
for name in ("hwload-final-set-20261107.progress", "hwload-final-set-lease-before-20261107.txt", "hwload-final-set-lease-after-20261107.txt"):
    shutil.copyfile(EXT / name, RUNS / name)
lease_before = (EXT / "hwload-final-set-lease-before-20261107.txt").read_text().split()
lease_after = (EXT / "hwload-final-set-lease-after-20261107.txt").read_text().strip()
if lease_before[0] != IMAGE or "containers_before=0" not in lease_before or lease_after != "containers_after=0":
    problems.append("the lease facts are not the named image with no container before or after: %s / %s" % (lease_before, lease_after))
counts = {r["label"]: r["tests_passed"] for r in final_runs}

compare_out = EXT / "hwload-inputmaps-compare-20261107.json"
join_out = EXT / "hwload-join-vs-e42bb-20261107.json"
for p in (compare_out, join_out):
    if p.exists():
        p.unlink()
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "sho-compare-maps-20261107.py"), str(EXT / "hwload-inputmap-PRE-20261107.json"), str(EXT / "hwload-inputmap-POST-20261107.json"),
    str(EXT / "hwload-extra-identity-PRE-20261107.json"), str(EXT / "hwload-extra-identity-POST-20261107.json"), str(compare_out))
compare = json.loads(compare_out.read_text(encoding="utf-8"))
if not compare["all_rows_equal"] or not compare["extra_identity_equal"]:
    problems.append("PRE and POST maps differ")
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "hwload-join-20261107.py"), str(REPO), str(EXT / "hwcore-inputmap-POST-20261107.json"), str(EXT / "hwload-inputmap-POST-20261107.json"), str(join_out))
join = json.loads(join_out.read_text(encoding="utf-8"))
if join["problems"]:
    problems.append("the join has problems: %s" % join["problems"])
EXPECTED_SOURCES = {"eech-campaign/crates/eech-world/lua/sector-height-witness.lua", "eech-campaign/crates/eech-world/tests/sector_height_witness_driver.rs", "eech-campaign/docs/d3-sector-height-production-contract.md"}
shutil.copyfile(compare_out, PACK / "inputmaps-compare.json")
shutil.copyfile(join_out, PACK / "join-vs-loader-fix.json")
maps = {name: {"bytes": (EXT / name).stat().st_size, "sha256": sha(EXT / name), "location": "external (full per-file rows)"}
        for name in ("hwload-inputmap-PRE-20261107.json", "hwload-inputmap-POST-20261107.json", "hwload-extra-identity-PRE-20261107.json", "hwload-extra-identity-POST-20261107.json", "hwcore-inputmap-POST-20261107.json")}

# the stored bytes are what Git actually holds: stage the changed sources and compare every literal claim against the index blob bytes
porcelain = subprocess.check_output(["git", "-C", str(REPO), "status", "--porcelain", "--untracked-files=all"], text=True).splitlines()
source_paths = [l[3:] for l in porcelain if not l[3:].startswith(SOURCES_EXCLUDED_PREFIXES)]
if set(source_paths) != EXPECTED_SOURCES:
    problems.append("the changed source set is not the three expected files: %s" % sorted(set(source_paths) ^ EXPECTED_SOURCES))
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
disagreements = [x if isinstance(x, str) else x.get("path") for x in compare["raw_vs_git_filtered_disagreements"]["post"]]
literal = {}
for path in sorted(disagreements):
    raw = (REPO / path).read_bytes()
    blob = git_bytes("cat-file", "blob", ":" + path)
    literal[path] = {"raw_bytes_equal_git_blob_bytes": raw == blob, "raw_sha256": hashlib.sha256(raw).hexdigest(), "git_blob_sha256": hashlib.sha256(blob).hexdigest(), "raw_crlf_lines": raw.count(b"\r\n"), "blob_crlf_lines": blob.count(b"\r\n")}
crlf_exceptions = sorted(p for p, v in literal.items() if not v["raw_bytes_equal_git_blob_bytes"])
filter_only = sorted(p for p, v in literal.items() if v["raw_bytes_equal_git_blob_bytes"])
if len(crlf_exceptions) != 3 or len(filter_only) != 4 or not all(p.endswith(".lua") for p in crlf_exceptions):
    problems.append("the raw-versus-Git qualification is not 3 Lua CRLF exceptions and 4 filter-only files: %s / %s" % (crlf_exceptions, filter_only))
old_packets_unchanged = {}
for rel in ("eech-campaign/reference/d3-entity-collision/sector-height-proof-guards-reviewed", "eech-campaign/reference/d3-entity-collision/sector-height-finite-native-ci-reviewed", "eech-campaign/reference/d3-ci-lua-download", "eech-campaign/reference/d3-entity-collision/sector-height-native-witness-source", "eech-campaign/reference/d3-entity-collision/sector-height-native-witness-source-reviewed", "eech-campaign/reference/d3-entity-collision/sector-height-native-witness-fix", "eech-campaign/reference/d3-entity-collision/sector-height-native-witness-fix-reviewed", "eech-campaign/reference/d3-entity-collision/sector-height-native-witness-reference-fix", "eech-campaign/reference/d3-entity-collision/sector-height-reference-fix-reviewed", "eech-campaign/reference/d3-entity-collision/sector-height-native-core-load", "eech-campaign/reference/d3-entity-collision/sector-height-native-core-load-fix"):
    unchanged = subprocess.run(["git", "-C", str(REPO), "diff", "--quiet", "HEAD", "--", rel]).returncode == 0
    old_packets_unchanged[rel] = unchanged
    if not unchanged:
        problems.append("an older packet changed: " + rel)

guards = {}
FIX_IDS = ["ENV-SECTOR-HEIGHT-NATIVE-LOAD-ORDER-DIAGNOSTIC-1", "ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1", "ENV-SECTOR-HEIGHT-M5-ROUNDING-ORACLE-1", "ENV-SECTOR-HEIGHT-M6-PACK-RETRY-1", "ENV-SECTOR-HEIGHT-M6-SAVE-PATH-BOUND-1", "ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1", "ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1", "ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1", "sector-height-host-move-open-facts"]
checks = [("generic", []), ("production-cutover", ["--action", "production-cutover"])] + [("fix-blocker " + b, ["--action", "fix-blocker", "--blocker", b]) for b in FIX_IDS] + [("deliver (expected unmet)", ["--action", "deliver"])]
for key, args in checks:
    guards[key] = run(PY, "-B", "-X", "utf8", "eech-campaign/tools/migration-progress.py", *args).returncode

scripts = PACK / "scripts"
scripts.mkdir(exist_ok=True)
script_rows = []
for name in READERS + ["publish-load-followup-20261107.py", "hwload-retained-dev-runs-20261107.json"]:
    shutil.copyfile(EXT / name, scripts / name)
    script_rows.append({"name": name, "sha256": sha(scripts / name)})
subprocess.run(["git", "-C", str(REPO), "add", "-A", PACK_REL], check=True)

freeze = {
    "kind": "D3 sector-height LOAD-ORDER DIAGNOSTIC CORRECTION CODE_FROZEN (UNREVIEWED; test-only Lua, scripted tests and contract; native UNRUN): ENV-SECTOR-HEIGHT-NATIVE-LOAD-ORDER-DIAGNOSTIC-1 and ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1 stay OPEN until the SAME combined review and a real native positive pair",
    "frozen_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "task": "EEDCS-49",
    "stage": "one focused correction batch inside the current sector-height extraction (not a preparation batch; the count stays 2): the never-passed load-order diagnostic gate, plus the mechanism of the Core-load blocker established from source and retained bytes; NO C, Rust product, build or Cargo input changed: the loader fix 90e9547ee keeps its identity",
    "source_before": head,
    "loader_fix_commit": "90e9547ee5fbfb31a461a861fdaa2adac5fbafe0",
    "native_identity_not_relabelled": NATIVE,
    "findings": {"ENV-SECTOR-HEIGHT-NATIVE-LOAD-ORDER-DIAGNOSTIC-1": "OPEN until the SAME reviewer accepts the correction", "ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1": "OPEN until the SAME reviewer and a real native positive pair loads", "ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1": "CLOSED_SOURCE (unchanged)", "ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1": "OPEN until actual native proof", "ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1": "OPEN until actual native proof", "sector-height-host-move-open-facts": "OPEN"},
    "constraint_files_read": {"at_start": CONSTRAINT_FILES_AT_START, "at_freeze": sorted(p.name for p in EXT.glob("Root-height-native-load-*.json")), "sha256": SEALED_SHA, "unread_new_files_at_freeze": sorted(set(p.name for p in EXT.glob("Root-height-native-load-*.json")) - set(CONSTRAINT_FILES_AT_START) - set(SEALED))},
    "sealed_inputs": {name: {"sha256": sha(EXT / name)} for name in SEALED},
    "root_feedback_files_present": feedback_files,
    "lease": {"volume": VOLUME, "image": IMAGE, "image_named_never_substituted": True, "containers_before": lease_before[1:], "containers_after": lease_after, "cache_cleanup": "none", "root_lease": {"file": "Root-height-reference-reader-9d247-lease-20261107.json", "sha256": sha(EXT / "Root-height-reference-reader-9d247-lease-20261107.json")},
              "tooling_note": "rustfmt and clippy were installed into each EPHEMERAL container by `rustup component add`; the image is Root's explicit 9d247 lease image, measured in the runs"},
    "change": {
        "load_order": "lua/sector-height-witness.lua: the load verdict no longer refuses insertions before the first restore (the supported boot builds a baseline, CAMPAIGN_DATA before SAVED_CAMPAIGN: 282933 in the actual native load); they are reported telemetry, never zeroed or hidden; insertions BETWEEN restores stay a failure, saved-entity insertions AFTER the last restore are still required (else UNREACHED); every restored raw word, ACK, coverage, order check, the real second pack and its per-sector no-lowering/NaN relation stay strict; positions only, no caller attribution",
        "diagnosis_followup": "the mechanism of the Core-load blocker is established from source and retained bytes: the compiled shim reads file_size () input as text and cuts at the first Ctrl-Z (byte 167 of 300873), so 163 bytes reached unpack_session; the 1748 restore callbacks all stated zero and are not raw-history parity; the failing call is default_get_entity_child_succ (the force root read as numeric index 0); original controller exit 1 (case mistake) and native exit 0 / reader exit 0 qualified",
        "docs": "contract section 12.7 and the load row of 12.2",
    },
    "red_green": {"red_on_previous_lua": [label for label in RETAINED if "red-previous" in label], "green": ["the final set u3"], "refactor": "none (test-only source)"},
    "test_counts_final": counts,
    "retained_development_runs": {label: {"actual_exit": code, "why": why} for label, (code, why) in RETAINED.items()},
    "runs": runs,
    "guards": guards,
    "input_maps": {"files": maps, "compare_pre_post": compare, "pre_utc": PRE_UTC, "post_utc": POST_UTC, "post_utc_note": "recorded by `date -u` immediately before the POST capture; not reconstructed from a file time", "join_vs_loader_fix": join},
    "literal_bytes": {"changed_sources": source_literal, "lua_crlf_raw_vs_git_exceptions": crlf_exceptions, "core_filter_only_qualifications": filter_only, "older_packets_unchanged_against_HEAD": old_packets_unchanged},
    "compiled_input_equivalence": {"to": "the loader fix 90e9547ee (all compiled and linked inputs, parsgen.c included)", "identical": join["compiled_inputs_identical"], "rows": join["compiled_input_rows"], "aggregate_sha256": join["compiled_inputs_aggregate_sha256_now"], "statement": "bound from the raw per-file maps BEFORE this freeze: no C, Rust product, build or Cargo input changed since the loader fix, so its retained C checks (s1..s9 of its freeze) keep their true identity and nothing is replayed; the 17f binaries stay not reusable for the loader fix"},
    "audit_reader_timing": "this freeze script, the join, the extra-identity script, the final-set runner, the runner, the check script, the batched map script and the comparer were FINAL before the PRE identity capture (their SHA-256 recorded at PRE and verified unchanged by this script); no reader changed after the POST capture",
    "reused_unchanged": "the loader fix, its guard and its C checks at their own identity; the existing M5 Root runs keep their actual 9cb/17f identity and stay valid only at that finite scope",
    "scripts": script_rows,
    "not_run": ["native, Windows, retail or war launch (Root's, with NEW binaries from the loader-fix source)", "any replay of the failed load", "unchanged C, engine-sys, engine or dc checks (compiled inputs identical to the loader fix)", "the 179/150/221 characterisations, the 16 matrices, campaign replays", "agents/MCP/authentication/model change", "cache, image or process deletion", "merge"],
    "limits": [
        "source and Linux checks only; nothing here is a native result or an accepted corrected load: the real load stays failure 1 until a new native run; the scripted collaborator checks the DRIVER, never parsgen.c, unpack_session or the Windows C runtime",
        "the corrected Lua needs its own literal identity; the existing M5 Root runs used the 9cb literal and 17f binaries and remain valid only at that scope",
        "the insertion classes are positions relative to the restores, not caller attribution: a baseline insertion is not shown to be 'static population' by the driver itself (that is the source fact of parsgen.c CAMPAIGN_DATA)",
        "fmt and clippy ran only because the components were installed into each ephemeral container of the leased image",
    ],
    "state": {"claims": "no self-review, approval, native clearance, finding closure, whole Task49 or D3, delivery", "D3-CURRENT.json": "not edited (Root owns it)"},
    "hygiene_problems_at_freeze": problems,
}
FREEZE.write_text(json.dumps(freeze, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
shutil.copyfile(FREEZE, PACK / FREEZE.name)
files = [{"path": str(p.relative_to(PACK)).replace("\\", "/"), "bytes": p.stat().st_size, "sha256": sha(p)} for p in sorted(PACK.rglob("*")) if p.is_file() and p.name != "manifest.json"]
(PACK / "manifest.json").write_text(json.dumps({"files": files, "scope": "Source and local-check packet of the M5/M6 witness batch only; not a native result, review, approval or acceptance."}, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps({"problems": problems, "runs": len(runs), "final": counts, "files": len(files), "guards": guards, "crlf_exceptions": crlf_exceptions, "filter_only": filter_only, "sources": len(source_paths)}, indent=1))
sys.exit(1 if problems else 0)
