"""Freezes the four-P2 correction packet of the M5/M6 witness source ONCE. Usage: freeze.py <repo-root> <external-freeze-path> <post-utc>
This script and every reader it names (the final-set runner, the join, the extra-identity script, the runner and the check script) are FINAL before the PRE identity capture: their SHA-256 are recorded at PRE
(hwfix-readers-PRE-20261107.json) and this script REFUSES to freeze if any of them differs now (the first witness freeze's reader was edited after its POST; that cannot recur unnoticed)."""
import datetime
import hashlib
import json
import pathlib
import re
import shutil
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1])
FREEZE = pathlib.Path(sys.argv[2])
POST_UTC = sys.argv[3]
EXT = pathlib.Path("R:/d3-evidence")
PY = "C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
PACK_REL = "eech-campaign/reference/d3-entity-collision/sector-height-native-witness-fix"
PACK = REPO / PACK_REL
RUNS = PACK / "runs"
CHECKPOINT = "03eab486f"
REVIEWED_SOURCE = "6e4139043"
NATIVE = "1b89c0b0ab613d2f6a08ea02dd6612457efa8982"
IMAGE = "sha256:810e0121c7081fc7f4b51d8973beec085759e659be46e241e0bb8864245b13c2"
VOLUME = "eech-linux-d3-height-witnesses-20261107"
SEALED = {  # read in full, not recopied: they sit literally in the sector-height-native-witness-source-reviewed packet (commit 03eab486f)
    "independent-sector-height-native-witness-source-6e4139-20261107.json": "96b2f4140e47ef17b0e672691e39173b3d920ea75282ec0cee60d56cdab8730f",
    "height-witness-Root-binding-579-v3-20261107.json": "4ef8c75045809c6baa2e7cc16330011407848112cb2130bf8da5897047a50203",
    "height-witness-579-all9-CI-readback-20261107.json": "f8d741fd5e8c46685a4c8133d8a0f40362ecd3aee9d01bfdb072b7783727aef8",
}
PRE_UTC = (EXT / "hwfix-PRE-utc-20261107.txt").read_text().strip()
# the development runs that did not end 0, kept literally, and why (everything else, including all final children v1..v10, exits 0)
RETAINED = {
    "x1-witness-driver-tests": (101, "the first run of the rewritten driver tests: 22 passed and one failed, because the non-exact elevation search used the word 5000.0 whose subtraction differences are all exact ties (nearest and toward zero never differ); the test word was changed to 6000.0 (test setup, not a driver defect)"),
    "x3-red-old-driver": (1, "SETUP FAILURE, retained: two test filters were given to one cargo invocation and cargo rejected the second (no test ran); the two RED runs were then made one filter each (x4, x5)"),
    "x4-red-old-driver-rtz": (101, "RED on the PREVIOUS driver (only the driver file reverted from the working tree, tests unchanged): the expected-clearance control with a finite NON-EXACT subtraction under the real toward-zero profile; the old driver reported UNREACHED 'did not dominate (hand-off 47c061f2, expected 47c061f3)', i.e. it formed the expected word with round-to-nearest (ENV-SECTOR-HEIGHT-M5-ROUNDING-ORACLE-1); the new driver accepts the toward-zero hand-off (x2 and the final set)"),
    "x11-world-driver-tests": (101, "the first run of the world targets after the driver rewrite: the host_callbacks census of profile-selecting host scripts failed because the driver's header comment named the profile selection a second time (it pins exactly one occurrence per script); the comment was reworded and x12 passes (a prose-only change of the driver)"),
    "x5-red-old-driver-retry": (101, "RED on the PREVIOUS driver: a real store_session pack retry (every sector read again after a buffer growth) was rejected by the old driver as a duplicate read (ENV-SECTOR-HEIGHT-M6-PACK-RETRY-1); the new driver reconciles consistent repeats and still refuses inconsistent ones"),
}
FINAL = ["y1-fmt-check", "y2-clippy-workspace", "y3-clippy-witness-features", "y4-world-driver-tests", "y5-engine-sys-default", "y6-control-hosted", "y7-control-original-solver", "y8-witness-c-path-check-and-guard", "y9-engine-dc-default", "y10-engine-dc-witness"]
READERS = ["run-height-witness-direct-20261107.py", "check-height-witness-linux-20261107.sh", "run-hwfix-final-set-20261107.sh", "hwfix-join-20261107.py", "hwfix-extra-identity-20261107.py", "freeze-hwfix-20261107.py", "sector-roster-inputmap-batched-20261106.py", "sho-compare-maps-20261107.py"]
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
readers_pre = json.loads((EXT / "hwfix-readers-PRE-20261107.json").read_text(encoding="utf-8"))
FREEZE_SELF = "freeze-hwfix-20261107.py"
PRE_VERSION = EXT / "freeze-hwfix-20261107.PRE-version.py"
for name in READERS:
    if name == FREEZE_SELF:
        continue
    if readers_pre["readers"].get(name) != sha(EXT / name):
        problems.append("an audit reader differs from its PRE identity (edited after the PRE capture): " + name)
freeze_self_erratum = {
    "script": FREEZE_SELF, "sha256_at_PRE": readers_pre["readers"][FREEZE_SELF], "sha256_now": sha(EXT / FREEZE_SELF), "pre_version_retained": PRE_VERSION.name, "pre_version_sha256": sha(PRE_VERSION),
    "edited_after_post": True,
    "cause": "the first freeze attempt (after POST 12:31:15Z, before any output file of the freeze was written) crashed in the sort key of the run-record listing: it applied an x/y label pattern to the first witness batch's w*/v* records (AttributeError, traceback retained). The edit filters the batch's x*/y* records first and then sorts, and records this erratum; nothing else in the script, and no other reader, changed.",
    "attempt1_stderr_retained": "hwfix-freeze-attempt1-20261107.stderr.txt",
    "replay": "none: a publisher-only edit does not change any input of any check; the PRE and POST maps and extras are equal, the final checks ran before the edit",
}
if freeze_self_erratum["sha256_at_PRE"] != freeze_self_erratum["pre_version_sha256"]:
    problems.append("the retained PRE version of the freeze script is not the PRE identity")
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
batch_records = [p for p in EXT.glob("sector-height-witness-*.json") if re.match(r"^[xy]\d+[a-z]?-", p.name[len("sector-height-witness-"):])]
for record_path in sorted(batch_records, key=lambda p: (p.name[len("sector-height-witness-")], int(re.match(r"[xy](\d+)", p.name[len("sector-height-witness-"):]).group(1)), p.name)):
    name = record_path.name[len("sector-height-witness-"):-len(".json")]
    if not re.match(r"^[xy]\d+[a-z]?-", name):
        continue
    for suffix in ("json", "stdout.log", "stderr.log"):
        shutil.copyfile(EXT / ("sector-height-witness-%s.%s" % (name, suffix)), RUNS / ("%s.%s" % (name, suffix)))
    record = json.loads(record_path.read_text(encoding="utf-8"))
    if record.get("state") != "terminal":
        problems.append("run not terminal: " + name)
    text = (RUNS / (name + ".stdout.log")).read_text(encoding="utf-8", errors="replace")
    passed = sum(int(m) for m in re.findall(r"test result: \w+\. (\d+) passed", text))
    failed = sum(int(m) for m in re.findall(r"; (\d+) failed", text))
    final = bool(re.match(r"^y\d+-", name))
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
    problems.append("the final set is not y1..y10 all exit 0 with no failed test: %s" % [(r["label"], r["actual_exit"], r["tests_failed"]) for r in final_runs])
by_label = {r["label"]: r for r in final_runs}
for label in FINAL[3:]:
    if by_label.get(label, {}).get("tests_passed", 0) <= 0:
        problems.append("a final test run selected no test: " + label)
for label, target_fragment, minimum in (("y4-world-driver-tests", "sector_height_witness_driver", 23), ("y8-witness-c-path-check-and-guard", "tests/sector_height_witness.rs", 2), ("y8-witness-c-path-check-and-guard", "sector_height_witness_guard", 4)):
    got = sum(v for k, v in by_label.get(label, {}).get("targets", {}).items() if target_fragment in k)
    if got < minimum:
        problems.append("%s: target %s ran %d tests (expected at least %d)" % (label, target_fragment, got, minimum))
# the FIRST attempt of the final set (none unless a retry occurred) ended v4 exit 101 on a real census test: the 64th profile-selecting host script is the new driver (63 -> 64); kept literally, the cycle repeated after the count was updated
attempt1 = EXT / "hwfix-attempt1"
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
a1_progress = (attempt1 / "hwfix-final-set-20261107.progress").read_text() if attempt1.is_dir() else ""
if attempt1.is_dir() and "exit=" not in a1_progress:
    problems.append("the retained first attempt has no progress")
progress = (EXT / "hwfix-final-set-20261107.progress").read_text()
exits = dict(re.findall(r"^(\S+) exit=(\d+)$", progress, flags=re.M))
if "DONE" not in progress.split() or sorted(exits) != sorted(FINAL) or any(v != "0" for v in exits.values()):
    problems.append("the progress file is not ten children all exit 0 then DONE: %s" % exits)
for name in ("hwfix-final-set-20261107.progress", "hwfix-final-set-lease-before-20261107.txt", "hwfix-final-set-lease-after-20261107.txt"):
    shutil.copyfile(EXT / name, RUNS / name)
lease_before = (EXT / "hwfix-final-set-lease-before-20261107.txt").read_text().split()
lease_after = (EXT / "hwfix-final-set-lease-after-20261107.txt").read_text().strip()
if lease_before[0] != IMAGE or "containers_before=0" not in lease_before or lease_after != "containers_after=0":
    problems.append("the lease facts are not the named image with no container before or after: %s / %s" % (lease_before, lease_after))
counts = {r["label"]: r["tests_passed"] for r in final_runs}

compare_out = EXT / "hwfix-inputmaps-compare-20261107.json"
join_out = EXT / "hwfix-join-vs-1b89-ci-and-reviewed-20261107.json"
for p in (compare_out, join_out):
    if p.exists():
        p.unlink()
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "sho-compare-maps-20261107.py"), str(EXT / "hwfix-inputmap-PRE-20261107.json"), str(EXT / "hwfix-inputmap-POST-20261107.json"),
    str(EXT / "hwfix-extra-identity-PRE-20261107.json"), str(EXT / "hwfix-extra-identity-POST-20261107.json"), str(compare_out))
compare = json.loads(compare_out.read_text(encoding="utf-8"))
if not compare["all_rows_equal"] or not compare["extra_identity_equal"]:
    problems.append("PRE and POST maps differ")
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "hwfix-join-20261107.py"), str(REPO), str(EXT / "ci-lua-inputmap-POST-20261107.json"), str(EXT / "hwfix-inputmap-POST-20261107.json"), str(EXT / "hw-inputmap-POST-20261107.json"), str(join_out))
join = json.loads(join_out.read_text(encoding="utf-8"))
if join["problems"]:
    problems.append("the join has problems: %s" % join["problems"])
EXPECTED_SOURCES = {"eech-campaign/crates/eech-engine-sys/csrc/eech_hw_witness.c", "eech-campaign/crates/eech-engine-sys/tests/sector_height_witness.rs", "eech-campaign/crates/eech-engine-sys/tests/sector_height_witness_guard.rs", "eech-campaign/crates/eech-world/lua/sector-height-witness.lua", "eech-campaign/crates/eech-world/tests/sector_height_witness_driver.rs", "eech-campaign/docs/d3-sector-height-production-contract.md"}
shutil.copyfile(compare_out, PACK / "inputmaps-compare.json")
shutil.copyfile(join_out, PACK / "join-vs-1b89-ci-and-reviewed.json")
maps = {name: {"bytes": (EXT / name).stat().st_size, "sha256": sha(EXT / name), "location": "external (full per-file rows)"}
        for name in ("hwfix-inputmap-PRE-20261107.json", "hwfix-inputmap-POST-20261107.json", "hwfix-extra-identity-PRE-20261107.json", "hwfix-extra-identity-POST-20261107.json", "hw-inputmap-POST-20261107.json")}

# the stored bytes are what Git actually holds: stage the changed sources and compare every literal claim against the index blob bytes
porcelain = subprocess.check_output(["git", "-C", str(REPO), "status", "--porcelain", "--untracked-files=all"], text=True).splitlines()
source_paths = [l[3:] for l in porcelain if not l[3:].startswith(SOURCES_EXCLUDED_PREFIXES)]
if set(source_paths) != EXPECTED_SOURCES:
    problems.append("the changed source set is not the six expected files: %s" % sorted(set(source_paths) ^ EXPECTED_SOURCES))
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
for rel in ("eech-campaign/reference/d3-entity-collision/sector-height-proof-guards-reviewed", "eech-campaign/reference/d3-entity-collision/sector-height-finite-native-ci-reviewed", "eech-campaign/reference/d3-ci-lua-download", "eech-campaign/reference/d3-entity-collision/sector-height-native-witness-source", "eech-campaign/reference/d3-entity-collision/sector-height-native-witness-source-reviewed"):
    unchanged = subprocess.run(["git", "-C", str(REPO), "diff", "--quiet", "HEAD", "--", rel]).returncode == 0
    old_packets_unchanged[rel] = unchanged
    if not unchanged:
        problems.append("an older packet changed: " + rel)

guards = {}
FIX_IDS = ["ENV-SECTOR-HEIGHT-M5-ROUNDING-ORACLE-1", "ENV-SECTOR-HEIGHT-M6-PACK-RETRY-1", "ENV-SECTOR-HEIGHT-M6-SAVE-PATH-BOUND-1", "ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1", "ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1", "ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1", "sector-height-host-move-open-facts"]
checks = [("generic", []), ("production-cutover", ["--action", "production-cutover"])] + [("fix-blocker " + b, ["--action", "fix-blocker", "--blocker", b]) for b in FIX_IDS] + [("deliver (expected unmet)", ["--action", "deliver"])]
for key, args in checks:
    guards[key] = run(PY, "-B", "-X", "utf8", "eech-campaign/tools/migration-progress.py", *args).returncode

scripts = PACK / "scripts"
scripts.mkdir(exist_ok=True)
script_rows = []
shutil.copyfile(EXT / "freeze-hwfix-20261107.PRE-version.py", scripts / "freeze-hwfix-20261107.PRE-version.py")
shutil.copyfile(EXT / "hwfix-freeze-attempt1-20261107.stderr.txt", scripts / "hwfix-freeze-attempt1-20261107.stderr.txt")
script_rows.append({"name": "freeze-hwfix-20261107.PRE-version.py", "sha256": sha(scripts / "freeze-hwfix-20261107.PRE-version.py")})
script_rows.append({"name": "hwfix-freeze-attempt1-20261107.stderr.txt", "sha256": sha(scripts / "hwfix-freeze-attempt1-20261107.stderr.txt")})
for name in READERS + ["publish-witness-review-20261107.py", "patch-card-status-witness-checkpoint1-20261107.py", "height-witness-reviewed-README-20261107.md"]:
    shutil.copyfile(EXT / name, scripts / name)
    script_rows.append({"name": name, "sha256": sha(scripts / name)})
subprocess.run(["git", "-C", str(REPO), "add", "-A", PACK_REL], check=True)

freeze = {
    "kind": "D3 sector-height M5/M6 witness source CORRECTION CODE_FROZEN (UNREVIEWED; test-only; native UNRUN): the four P2s of review 96b2f414 (ENV-SECTOR-HEIGHT-M5-ROUNDING-ORACLE-1, -M6-PACK-RETRY-1, -M6-SAVE-PATH-BOUND-1, -WITNESS-CONSUMER-JOIN-1) corrected in source; all four, the two witness findings and height-open-facts stay OPEN",
    "frozen_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "task": "EEDCS-49",
    "stage": "correction of the required witnesses inside the current sector-height extraction (not a preparation batch; the count stays 2); no production policy, kernel, protocol, tolerance, baseline or C source of the original campaign changed",
    "source_before": head,
    "reviewed_source": REVIEWED_SOURCE,
    "native_identity_not_relabelled": NATIVE,
    "findings": {fid: "OPEN until the SAME review accepts the correction (and, for the witnesses, actual native proof)" for fid in FIX_IDS[:4]} | {"ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1": "OPEN until actual native proof", "ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1": "OPEN until actual native proof", "sector-height-host-move-open-facts": "OPEN"},
    "sealed_inputs": {name: {"sha256": sha(EXT / name)} for name in SEALED},
    "root_feedback_files_present": feedback_files,
    "lease": {"volume": VOLUME, "image": IMAGE, "image_named_never_substituted": True, "containers_before": lease_before[1:], "containers_after": lease_after, "cache_cleanup": "none",
              "tooling_note": "the leased image has no rustfmt or clippy; they were installed into each EPHEMERAL container by `rustup component add` (network read, no image, volume or global change); the earlier unavailable-component facts are in the first witness packet (w15, w16)"},
    "change": {
        "m5_rounding_oracle": "lua/sector-height-witness.lua forms the expected clearance word in the accepted SSE round-toward-zero profile (toward_zero_single, subtract_single with a two-sum exactness test; division-free); a clean non-dominance (hand-off above the expected value and equal to the control's) is UNREACHED, an incorrectly formed hand-off FAILURE; the scripted harness runs its hand-off under the REAL eech_world::reference_arithmetic::select_sse_toward_zero with MXCSR saved and restored; no C policy, solver, tolerance or baseline change",
        "m6_pack_retry": "the driver reconciles store_session's pack_session retries (every attempt re-reads every sector): repeats start a new attempt, every attempt must repeat the first in order with the same words, at most 64; the unique final saved-sector relation is what save and load report; inconsistent, omitted or unsupported repetition is NONPASS",
        "m6_path_bound": "csrc/eech_hw_witness.c: ONE shared helper eech_hw_save_paths bounds BOTH complete writer paths (data and script, suffix and NUL) to the original writer's data_filename [256] and script_filename [256] before any file access and before store_session; called directly by real-C tests (255 accepted, 256 refused for the data and the longer-suffix script path, no output and no file on refusal) and pinned by the source guard against the original writer's buffers",
        "consumer_join": "same-caller lookahead/move identity with a real answer before eligibility, intervention or consumption; trajectory binding of an intervention window to the paired control report (control=), whole-report validation of original=, control= and expect= (schema 2), strict raw comma-list grammar, request/reply/abort fault precedence over clean UNREACHED, no post-fatal inspection; no created task leader was needed",
        "docs": "contract section 12 (12.1 table and commands corrected, new 12.4)",
    },
    "red_green": {"red_on_previous_driver": ["x4-red-old-driver-rtz", "x5-red-old-driver-retry"], "green": ["x2 (23 driver tests, before formatting)", "the final set y4"], "c_path": "the new helper has no previous version to fail: its real-C boundary test was first run against the new C (x6) and the guard x7; the old 320-byte temporaries are pinned absent", "refactor": "none (test-only source)"},
    "test_counts_final": counts,
    "retained_development_runs": {label: {"actual_exit": code, "why": why} for label, (code, why) in RETAINED.items()},
    "runs": runs,
    "guards": guards,
    "input_maps": {"files": maps, "compare_pre_post": compare, "pre_utc": PRE_UTC, "post_utc": POST_UTC, "post_utc_note": "recorded by `date -u` immediately before the POST capture; not reconstructed from a file time", "join_vs_1b89_ci_and_reviewed": join},
    "literal_bytes": {"changed_sources": source_literal, "lua_crlf_raw_vs_git_exceptions": crlf_exceptions, "core_filter_only_qualifications": filter_only, "older_packets_unchanged_against_HEAD": old_packets_unchanged},
    "freeze_script_chronology_erratum": freeze_self_erratum,
    "audit_reader_timing": "EXCEPT the freeze script's one erratum above (a publisher-only crash fix after POST), this freeze script, the join, the extra-identity script, the final-set runner, the runner, the check script, the batched map script and the comparer were FINAL before the PRE identity capture (their SHA-256 recorded at PRE and verified unchanged by this script); no reader changed after the POST capture",
    "reused_unchanged": "the 1b89 native evidence and the accepted 179/150/221 characterisations keep their true identity and are reused only through the join, which shows from the Git diff that no C source of the original campaign changed and that the compiled additions are feature-gated; the hosted and original-solver controls (y6, y7) were run again because they are cheap and show the shipping build is unchanged; nothing is claimed natively for this changed source",
    "scripts": script_rows,
    "not_run": ["native, Windows, retail or war launch (Root's, after the SAME review)", "the witness runs themselves (natively UNRUN: `store_session`, `pack_session`, `unpack_session` and the guarded fw_move.c branch have not run here)", "i686 or EECH_C_OPT=2 builds (the feature-off sources are unchanged in effect)",
                "the whole workspace or whole eech-world suites, the 16 matrices and any campaign replay (the changed crates' own targets and the existing controls were run)", "agents/MCP/authentication/model change", "cache, image or process deletion", "merge"],
    "limits": [
        "source and Linux checks only; nothing here is a native result, clearance or acceptance; the four findings and both witness findings stay OPEN",
        "the scripted collaborator of tests/sector_height_witness_driver.rs stands in for the campaign: it checks the DRIVER (eligibility, same-caller join, single intervention, markers, fault controls, reference validation), never fw_move.c, store_session, pack_session or unpack_session",
        "the toward-zero oracle was exercised under the real selected profile on x86-64 SSE (the accepted reference profile); no i686/x87 or EECH_C_OPT=2 claim is made",
        "the trajectory binding proves the paired control and the intervention reached the same window state of a deterministic campaign; it does not identify the caller of an individual read (reads carry no caller identity)",
        "M5 is still a natural reach at a declared window: if the paired control never meets an eligible hand-off the witness is UNREACHED; a bounded test-only task leader was not needed and not built",
        "fmt and clippy ran only because the components were installed into each ephemeral container of the leased image (disclosed above)",
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
