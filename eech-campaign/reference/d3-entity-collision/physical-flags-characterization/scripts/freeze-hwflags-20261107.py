"""Freezes the physical-flags characterisation packet (ENV-PHYSICAL-FLAGS-RAWWORD-CONTRACT-1) ONCE. Usage: freeze.py <repo-root> <external-freeze-path> <post-utc>
This script and every reader it names (the final-set runner, the join, the extra-identity script, the runner and the check script) are FINAL before the PRE identity capture: their SHA-256 are recorded at PRE
(hwflags-readers-PRE-20261107.json) and this script REFUSES to freeze if any of them differs now (an earlier freeze's reader was edited after its POST; that cannot recur unnoticed; a deviation is recorded as an erratum, never hidden)."""
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
SEALED_SHA = {"binary": "c5bd47fae88dcc7a8a70f92bb0f53285d31fc53fa1f367558395f0b1c2359a17", "order": "af3d48a865cb4acb21a4f6ec269e2fa545fa39734c0744d29ee72e1e34a149c2", "reviewer": "4e3ed7124e91f2309b76457e49b61892b7e0fa1d5fc328925a04c8189c73e838"}
FEEDBACK_AT_START = []  # no Root-physical-flags-feedback*.json existed when this batch began
FEEDBACK_READ = {"Root-physical-flags-feedback-20261107-1550.json": "db5ce2376c1b8ad0e453ddaf47e8b4c1f955f1883e981d46011e7cefc7eb1316"}  # arrived 15:50, read before the freeze (and before the PRE that counts)
PY = "C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe"
PACK_REL = "eech-campaign/reference/d3-entity-collision/physical-flags-characterization"
PACK = REPO / PACK_REL
RUNS = PACK / "runs"
CHECKPOINT = "8c9e9b938"
REVIEWED_SOURCE = "3c2dee5b3"
NATIVE = "1b89c0b0ab613d2f6a08ea02dd6612457efa8982"
IMAGE = "sha256:9d247b4ea15457f06579ef703ce287e96358128e7a2217f2833ee8b628fc85c3"
VOLUME = "eech-linux-d3-height-reference-9d247-20261107"
SEALED = {
    "independent-sector-height-M6-native-764-3c2-20261107.json": SEALED_SHA["binary"],
    "height-core-load-native-764-positive-Root-audit-20261107.json": SEALED_SHA["order"],
    "height-core-load-native-764-Root-binding-20261107.json": SEALED_SHA["reviewer"],
}
PRE_UTC = (EXT / "hwflags-PRE-utc-20261107.txt").read_text().strip()
# the development runs that did not end 0, kept literally, and why (everything else, including all final children f1..f5, exits 0)
RETAINED = json.loads((EXT / "hwflags-retained-dev-runs-20261107.json").read_text(encoding="utf-8"))
RETAINED = {label: (v["actual_exit"], v["why"]) for label, v in RETAINED.items()}
FINAL = ["f1-fmt-check", "f2-clippy-engine-sys-default", "f3-clippy-engine-sys-flags-feature", "f4-engine-sys-default", "f5-flags-control-and-guard"]
READERS = ["run-physical-flags-direct-20261107.py", "check-physical-flags-linux-20261107.sh", "run-hwflags-final-set-20261107.sh", "hwflags-join-20261107.py", "hwflags-extra-identity-20261107.py", "freeze-hwflags-20261107.py", "sector-roster-inputmap-batched-20261106.py", "sho-compare-maps-20261107.py"]
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
readers_pre = json.loads((EXT / "hwflags-readers-PRE-20261107.json").read_text(encoding="utf-8"))
FREEZE_SELF = "freeze-hwflags-20261107.py"
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
batch_records = [p for p in EXT.glob("physical-flags-*.json") if re.match(r"^[pf]\d+[a-z]?-", p.name[len("physical-flags-"):])]
for record_path in sorted(batch_records, key=lambda p: (p.name[len("physical-flags-")], int(re.match(r"[pf](\d+)", p.name[len("physical-flags-"):]).group(1)), p.name)):
    name = record_path.name[len("physical-flags-"):-len(".json")]
    if not re.match(r"^[pf]\d+[a-z]?-", name):
        continue
    for suffix in ("json", "stdout.log", "stderr.log"):
        shutil.copyfile(EXT / ("physical-flags-%s.%s" % (name, suffix)), RUNS / ("%s.%s" % (name, suffix)))
    record = json.loads(record_path.read_text(encoding="utf-8"))
    if record.get("state") != "terminal":
        problems.append("run not terminal: " + name)
    text = (RUNS / (name + ".stdout.log")).read_text(encoding="utf-8", errors="replace")
    passed = sum(int(m) for m in re.findall(r"test result: \w+\. (\d+) passed", text))
    failed = sum(int(m) for m in re.findall(r"; (\d+) failed", text))
    final = bool(re.match(r"^f\d+-", name))
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
    problems.append("the final set is not f1..f5 all exit 0 with no failed test: %s" % [(r["label"], r["actual_exit"], r["tests_failed"]) for r in final_runs])
by_label = {r["label"]: r for r in final_runs}
for label in FINAL[3:]:
    if by_label.get(label, {}).get("tests_passed", 0) <= 0:
        problems.append("a final test run selected no test: " + label)
for label, target_fragment, minimum in (("f4-engine-sys-default", "tests/physical_flags_guard.rs", 3), ("f4-engine-sys-default", "tests/saved_campaign_load_guard.rs", 3), ("f4-engine-sys-default", "tests/sector_height_guard.rs", 7), ("f5-flags-control-and-guard", "tests/physical_flags.rs", 1), ("f5-flags-control-and-guard", "tests/physical_flags_guard.rs", 3)):
    got = sum(v for k, v in by_label.get(label, {}).get("targets", {}).items() if target_fragment in k)
    if got < minimum:
        problems.append("%s: target %s ran %d tests (expected at least %d)" % (label, target_fragment, got, minimum))
f5_text = (RUNS / "f5-flags-control-and-guard.stdout.log").read_text(encoding="utf-8", errors="replace")
if "physical flags expectations made: 199" not in f5_text:
    problems.append("the real-C control did not report its 199 expectations")
# a retained first attempt of the final set (none unless a retry occurred)
attempt1 = EXT / "hwflags-attempt1"
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
a1_progress = (attempt1 / "hwflags-final-set-20261107.progress").read_text() if attempt1.is_dir() else ""
if attempt1.is_dir() and "exit=" not in a1_progress:
    problems.append("the retained first attempt has no progress")
progress = (EXT / "hwflags-final-set-20261107.progress").read_text()
exits = dict(re.findall(r"^(\S+) exit=(\d+)$", progress, flags=re.M))
if "DONE" not in progress.split() or sorted(exits) != sorted(FINAL) or any(v != "0" for v in exits.values()):
    problems.append("the progress file is not ten children all exit 0 then DONE: %s" % exits)
for name in ("hwflags-final-set-20261107.progress", "hwflags-final-set-lease-before-20261107.txt", "hwflags-final-set-lease-after-20261107.txt"):
    shutil.copyfile(EXT / name, RUNS / name)
lease_before = (EXT / "hwflags-final-set-lease-before-20261107.txt").read_text().split()
lease_after = (EXT / "hwflags-final-set-lease-after-20261107.txt").read_text().strip()
if lease_before[0] != IMAGE or "containers_before=0" not in lease_before or lease_after != "containers_after=0":
    problems.append("the lease facts are not the named image with no container before or after: %s / %s" % (lease_before, lease_after))
counts = {r["label"]: r["tests_passed"] for r in final_runs}

compare_out = EXT / "hwflags-inputmaps-compare-20261107.json"
join_out = EXT / "hwflags-join-vs-e42bb-20261107.json"
for p in (compare_out, join_out):
    if p.exists():
        p.unlink()
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "sho-compare-maps-20261107.py"), str(EXT / "hwflags-inputmap-PRE-20261107.json"), str(EXT / "hwflags-inputmap-POST-20261107.json"),
    str(EXT / "hwflags-extra-identity-PRE-20261107.json"), str(EXT / "hwflags-extra-identity-POST-20261107.json"), str(compare_out))
compare = json.loads(compare_out.read_text(encoding="utf-8"))
if not compare["all_rows_equal"] or not compare["extra_identity_equal"]:
    problems.append("PRE and POST maps differ")
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "hwflags-join-20261107.py"), str(REPO), str(EXT / "hwload-inputmap-POST-20261107.json"), str(EXT / "hwflags-inputmap-POST-20261107.json"), str(join_out))
join = json.loads(join_out.read_text(encoding="utf-8"))
if join["problems"]:
    problems.append("the join has problems: %s" % join["problems"])
EXPECTED_SOURCES = {"eech-campaign/crates/eech-engine-sys/Cargo.toml", "eech-campaign/crates/eech-engine-sys/build/main.rs", "eech-campaign/crates/eech-engine-sys/csrc/eech_physical_flags_check.c", "eech-campaign/crates/eech-engine-sys/tests/physical_flags.rs", "eech-campaign/crates/eech-engine-sys/tests/physical_flags_guard.rs", "eech-campaign/docs/d3-physical-flags-contract.md"}
shutil.copyfile(compare_out, PACK / "inputmaps-compare.json")
shutil.copyfile(join_out, PACK / "join-vs-reference.json")
maps = {name: {"bytes": (EXT / name).stat().st_size, "sha256": sha(EXT / name), "location": "external (full per-file rows)"}
        for name in ("hwflags-inputmap-PRE-20261107.json", "hwflags-inputmap-POST-20261107.json", "hwflags-extra-identity-PRE-20261107.json", "hwflags-extra-identity-POST-20261107.json", "hwload-inputmap-POST-20261107.json")}

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
for rel in ("eech-campaign/reference/d3-entity-collision/sector-height-proof-guards-reviewed", "eech-campaign/reference/d3-entity-collision/sector-height-finite-native-ci-reviewed", "eech-campaign/reference/d3-ci-lua-download", "eech-campaign/reference/d3-entity-collision/sector-height-native-witness-source", "eech-campaign/reference/d3-entity-collision/sector-height-native-witness-source-reviewed", "eech-campaign/reference/d3-entity-collision/sector-height-native-witness-fix", "eech-campaign/reference/d3-entity-collision/sector-height-native-witness-fix-reviewed", "eech-campaign/reference/d3-entity-collision/sector-height-native-witness-reference-fix", "eech-campaign/reference/d3-entity-collision/sector-height-reference-fix-reviewed", "eech-campaign/reference/d3-entity-collision/sector-height-native-core-load", "eech-campaign/reference/d3-entity-collision/sector-height-native-core-load-fix", "eech-campaign/reference/d3-entity-collision/sector-height-native-load-followup", "eech-campaign/reference/d3-entity-collision/sector-height-native-load-order-fix", "eech-campaign/reference/d3-entity-collision/sector-height-core-load-and-m5-reviewed", "eech-campaign/reference/d3-entity-collision/sector-height-m6-native-764-reviewed"):
    unchanged = subprocess.run(["git", "-C", str(REPO), "diff", "--quiet", "HEAD", "--", rel]).returncode == 0
    old_packets_unchanged[rel] = unchanged
    if not unchanged:
        problems.append("an older packet changed: " + rel)

guards = {}
FIX_IDS = ["ENV-PHYSICAL-FLAGS-RAWWORD-CONTRACT-1", "ENV-SECTOR-HEIGHT-NATIVE-LOAD-ORDER-DIAGNOSTIC-1", "ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1", "ENV-SECTOR-HEIGHT-M5-ROUNDING-ORACLE-1", "ENV-SECTOR-HEIGHT-M6-PACK-RETRY-1", "ENV-SECTOR-HEIGHT-M6-SAVE-PATH-BOUND-1", "ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1", "ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1", "ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1", "sector-height-host-move-open-facts"]
checks = [("generic", []), ("production-cutover", ["--action", "production-cutover"])] + [("fix-blocker " + b, ["--action", "fix-blocker", "--blocker", b]) for b in FIX_IDS] + [("deliver (expected unmet)", ["--action", "deliver"])]
checks.append(("characterize ENV-PHYSICAL-FLAGS-RAWWORD-CONTRACT-1 (the intended guard action)", ["--action", "characterize", "--blocker", "ENV-PHYSICAL-FLAGS-RAWWORD-CONTRACT-1"]))
for key, args in checks:
    guards[key] = run(PY, "-B", "-X", "utf8", "eech-campaign/tools/migration-progress.py", *args).returncode

scripts = PACK / "scripts"
scripts.mkdir(exist_ok=True)
script_rows = []
for name in READERS + ["hwflags-retained-dev-runs-20261107.json", "hwflags-feedback-files-20261107.json"]:
    shutil.copyfile(EXT / name, scripts / name)
    script_rows.append({"name": name, "sha256": sha(scripts / name)})
subprocess.run(["git", "-C", str(REPO), "add", "-A", PACK_REL], check=True)

freeze = {
    "kind": "D3 PHYSICAL FLAGS CHARACTERISATION CODE_FROZEN (UNREVIEWED; test-only feature-gated C control, Rust runner, census guard and contract; NO production flag moves; native UNRUN): ENV-PHYSICAL-FLAGS-RAWWORD-CONTRACT-1 stays OPEN",
    "frozen_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "task": "EEDCS-49",
    "stage": "characterisation of ONE next REQUIRED physical contract (the per-entity UPDATED/MOVED/ROTATED word) under the named required-blocker exception; preparation count stays 2; no production flag, no ABI/trait/layout chosen",
    "source_before": head,
    "native_identity_not_relabelled": NATIVE,
    "findings": {"ENV-PHYSICAL-FLAGS-RAWWORD-CONTRACT-1": "OPEN (blocks_cutover for the flag move) until the SAME review decides the first extraction step", "ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1": "CLOSED finite (unchanged)", "ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1": "CLOSED finite (unchanged)", "sector-height-host-move-open-facts": "OPEN"},
    "feedback_files": {"at_start": FEEDBACK_AT_START, "at_freeze": sorted(p.name for p in EXT.glob("Root-physical-flags-feedback*.json")), "read_before_freeze": {n: {"sha256_expected": h, "sha256_now": sha(EXT / n)} for n, h in FEEDBACK_READ.items()}, "unread_new_files_at_freeze": sorted(set(p.name for p in EXT.glob("Root-physical-flags-feedback*.json")) - set(FEEDBACK_AT_START) - set(FEEDBACK_READ))},
    "sealed_inputs": {name: {"sha256": sha(EXT / name)} for name in SEALED},
    "root_feedback_files_present": feedback_files,
    "lease": {"volume": VOLUME, "image": IMAGE, "image_named_never_substituted": True, "volume_created_by_this_lane_first_docker_run": True, "containers_before": lease_before[1:], "containers_after": lease_after, "cache_cleanup": "none",
              "tooling_note": "rustfmt and clippy were installed into each EPHEMERAL container by `rustup component add`; the image is the explicitly retained 9d247 image (retention tag eech-height-reference-reader-9d247-20261107), measured in the runs (cargo 1.99.1, GCC 12.2 inside), NOT the Root 3a3 Windows producer identity"},
    "physical_contract_learned": [
        "three GLOBAL bit arrays (MAX_NUM_ENTITIES/32 words each), keyed by entity heap INDEX alone; the 'per-entity 32-bit word' is the array word holding the entity's bit",
        "the getter returns the entity's MASK (INT_MIN for bit 31, not 1), clears every OTHER entity's bit of the word and keeps its own",
        "UPDATED reset is a quarter-memset (bytes vs words): entity indices >= 131072 are never reset; MOVED/ROTATED reset only behind the Tacview frame reset",
        "UPDATED is physical (collision pass-through, fw pass-through, supply descriptor); MOVED/ROTATED are Tacview-observational; no campaign decision reads the words",
        "the supply-arrival descriptor carries the raw mask into a Rust checked_bool(0/1): a source-derived mismatch candidate, reachability unknown",
    ],
    "change": {"control": "csrc/eech_physical_flags_check.c (feature physical-flags-check, 199 expectations over the real getter/setter/flag arrays/update reset) + tests/physical_flags.rs", "census": "tests/physical_flags_guard.rs (text census: exact writer/reader table, the getter/reset source, the descriptor boundary)", "docs": "docs/d3-physical-flags-contract.md",
               "wiring": "Cargo.toml feature + [[test]], build/main.rs define block (added lines only)"},
    "red_green": {"red": "none: a characterisation of the ORIGINAL passes it (the legacy defects are preserved as compatibility facts, not repaired)", "green": ["f5 (199 expectations + 3 census tests)"]},
    "test_counts_final": counts,
    "retained_development_runs": {label: {"actual_exit": code, "why": why} for label, (code, why) in RETAINED.items()},
    "runs": runs,
    "guards": guards,
    "input_maps": {"files": maps, "compare_pre_post": compare, "pre_utc": PRE_UTC, "post_utc": POST_UTC, "post_utc_note": "recorded by `date -u` immediately before the POST capture; not reconstructed from a file time", "join_vs_reference": join},
    "literal_bytes": {"changed_sources": source_literal, "lua_crlf_raw_vs_git_exceptions": crlf_exceptions, "core_filter_only_qualifications": filter_only, "older_packets_unchanged_against_HEAD": old_packets_unchanged},
    "compiled_input_change": {"statement": "every compiled-input change is a test-only feature-gated ADDITION (verified in the join from the Git diff and the file text): the shipping builds are unchanged in effect", "compiled_rows_changed": join["compiled_rows_changed"], "compiled_rows_added": join["compiled_rows_added"], "unchanged_compiled_rows": join["unchanged_compiled_rows"]},
    "audit_reader_timing": "this freeze script, the join, the extra-identity script, the final-set runner, the runner, the check script, the batched map script and the comparer were FINAL before the PRE identity capture (their SHA-256 recorded at PRE and verified unchanged by this script); no reader changed after the POST capture",
    "scripts": script_rows,
    "not_run": ["native, Windows, retail or war launch (Root's; its 764 full-40 job is untouched)", "any consumer body (pass-through arithmetic, supply policy, Tacview writers), the setter with FALSE, the Tacview reset path", "the 179/150/221 characterisations, the 16 matrices, campaign replays", "i686, -O2 or other platforms", "agents/MCP/authentication/model change", "cache, image or process deletion outside the sole fresh volume", "merge"],
    "limits": [
        "the control is representative (indices and bit offsets), not exhaustive; it uses the engine's own entity struct and tables with an EMPTY update list, so the order effect of the update loop with real members and the consumer arithmetic are census facts, not behaviour",
        "the descriptor/checked_bool mismatch is source-derived and was not executed end to end (the Rust callback is private); its reachability and the entity-index range of real campaigns are UNKNOWN",
        "no production flag moved and no ABI, trait or layout was chosen; the first proposed step is for the SAME review to confirm or reject",
        "fmt and clippy ran only because the components were installed into each ephemeral container of the retained image",
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
