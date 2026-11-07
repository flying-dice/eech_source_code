"""Freezes the M5/M6 witness packet ONCE. Usage: freeze.py <repo-root> <external-freeze-path> <post-utc>
This script and every reader it names (the final-set runner, the join, the extra-identity script, the runner and the check script) are FINAL before the PRE identity capture."""
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
PACK_REL = "eech-campaign/reference/d3-entity-collision/sector-height-native-witness-source"
PACK = REPO / PACK_REL
RUNS = PACK / "runs"
CHECKPOINT = "00823df40"
NATIVE = "1b89c0b0ab613d2f6a08ea02dd6612457efa8982"
IMAGE = "sha256:810e0121c7081fc7f4b51d8973beec085759e659be46e241e0bb8864245b13c2"
VOLUME = "eech-linux-d3-height-witnesses-20261107"
SEALED = {  # read in full, not recopied: they sit literally in the sector-height-finite-native-ci-reviewed packet
    "independent-sector-height-finite-native-1b89-ci-lua-635-20261107.json": "8edddd934b091ea58567d43e3a71f4eaab378431de7b0b378990ae2703e77c32",
    "Root-feedback-height-native-CI-20261107.json": "452b10c8a4d129bc647cab737c01d637e8b7ff9b7e7dfefdff1aa5b7de12670a",
    "sector-height-native-Root-binding-1b89-20261107.json": "fa02f29a90c38a1cbb08d78b7b5dcf8d753916f51f3ba392a41a3bfec6d15698",
    "independent-sector-height-proofguards-e326-source-20261107.json": "cbbd27d713adc2d7505ac4931e7fa91c5a22bb46fe43ebdf888d6ac2d319dd94",
}
PRE_UTC = (EXT / "hw-PRE-utc-20261107.txt").read_text().strip()
# the development runs that did not end 0, kept literally, and why (everything else, including all final children v1..v10, exits 0)
RETAINED = {
    "w0-baseline-hosted": (101, "the first baseline command named the wrong feature set: sector_height requires `sector-height-check` (setup, not a result)"),
    "w3-witness-driver-tests": (101, "the first run of the new driver tests: a scripted frame did not end its window with a callback, and a failing-stored-word expectation named the wrong failure; both were test setup, fixed"),
    "w5-witness-driver-tests": (101, "compile error in the new test (a character literal in a generated path replacement), fixed"),
    "w6-witness-driver-tests": (101, "the first original-profile control: the scripted frames still held a hosted scan read, so the driver rightly called the host-asked run not the original-solver profile; the control's frames were corrected"),
    "w7-witness-driver-tests": (101, "the scripted campaign's break_at_window compared nil with nil and stopped the original-profile control at once; the guard was corrected"),
    "w9-witness-guard": (101, "the first source guard measured the attribute distance too tightly (49 characters of indentation); replaced by a whitespace-only check"),
    "w15-fmt-apply": (1, "UNAVAILABLE PREREQUISITE: the leased image has no rustfmt component (cargo-fmt is not installed for the toolchain); retained"),
    "w16-clippy-probe": (1, "UNAVAILABLE PREREQUISITE: the leased image has no clippy component; retained"),
    "w17-rustup-fmt": (1, "rustfmt installed into the ephemeral container by `rustup component add`; `cargo fmt --check` then listed the formatting differences of the new files (the RED of formatting), applied by w18"),
    "w19-clippy-workspace": (101, "strict clippy found three lints in the new driver test (two needless borrows and a while-let loop); fixed, w20 is the clean run"),
}
FINAL = ["v1-fmt-check", "v2-clippy-workspace", "v3-clippy-witness-features", "v4-world-driver-tests", "v5-engine-sys-default", "v6-control-hosted", "v7-control-original-solver", "v8-witness-c-refusals-and-guard", "v9-engine-dc-default", "v10-engine-dc-witness"]
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
for record_path in sorted(EXT.glob("sector-height-witness-*.json"), key=lambda p: (p.name[len("sector-height-witness-")], int(re.match(r"[wv](\d+)-", p.name[len("sector-height-witness-"):]).group(1)))):
    name = record_path.name[len("sector-height-witness-"):-len(".json")]
    if not re.match(r"^[wv]\d+-", name):
        continue
    for suffix in ("json", "stdout.log", "stderr.log"):
        shutil.copyfile(EXT / ("sector-height-witness-%s.%s" % (name, suffix)), RUNS / ("%s.%s" % (name, suffix)))
    record = json.loads(record_path.read_text(encoding="utf-8"))
    if record.get("state") != "terminal":
        problems.append("run not terminal: " + name)
    text = (RUNS / (name + ".stdout.log")).read_text(encoding="utf-8", errors="replace")
    passed = sum(int(m) for m in re.findall(r"test result: \w+\. (\d+) passed", text))
    failed = sum(int(m) for m in re.findall(r"; (\d+) failed", text))
    final = bool(re.match(r"^v\d+-", name))
    runs.append({"label": name, "final": final, "command": record["command"], "actual_exit": record["actual_exit"], "started_utc": record["started_utc"], "completed_utc": record.get("completed_utc"), "tests_passed": passed, "tests_failed": failed,
                 "stdout_sha256": sha(RUNS / (name + ".stdout.log")), "stderr_sha256": sha(RUNS / (name + ".stderr.log"))})
    expected = RETAINED.get(name, (0, ""))[0]
    if record["actual_exit"] != expected:
        problems.append("unexpected exit for %s: %s (expected %s)" % (name, record["actual_exit"], expected))
final_runs = [r for r in runs if r["final"]]
if sorted(r["label"] for r in final_runs) != sorted(FINAL) or any(r["actual_exit"] != 0 or r["tests_failed"] != 0 for r in final_runs):
    problems.append("the final set is not v1..v10 all exit 0 with no failed test: %s" % [(r["label"], r["actual_exit"], r["tests_failed"]) for r in final_runs])
# the FIRST attempt of the final set (PRE 11:28:34Z) ended v4 exit 101 on a real census test: the 64th profile-selecting host script is the new driver (63 -> 64); kept literally, the cycle repeated after the count was updated
attempt1 = EXT / "hw-attempt1"
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
a1_progress = (attempt1 / "hw-final-set-20261107.progress").read_text() if attempt1.is_dir() else ""
if "v4-world-driver-tests exit=101" not in a1_progress:
    problems.append("the first attempt's progress does not show the retained v4 failure")
progress = (EXT / "hw-final-set-20261107.progress").read_text()
exits = dict(re.findall(r"^(\S+) exit=(\d+)$", progress, flags=re.M))
if "DONE" not in progress.split() or sorted(exits) != sorted(FINAL) or any(v != "0" for v in exits.values()):
    problems.append("the progress file is not ten children all exit 0 then DONE: %s" % exits)
for name in ("hw-final-set-20261107.progress", "hw-final-set-lease-before-20261107.txt", "hw-final-set-lease-after-20261107.txt"):
    shutil.copyfile(EXT / name, RUNS / name)
lease_before = (EXT / "hw-final-set-lease-before-20261107.txt").read_text().split()
lease_after = (EXT / "hw-final-set-lease-after-20261107.txt").read_text().strip()
if lease_before[0] != IMAGE or "containers_before=0" not in lease_before or lease_after != "containers_after=0":
    problems.append("the lease facts are not the named image with no container before or after: %s / %s" % (lease_before, lease_after))
counts = {r["label"]: r["tests_passed"] for r in final_runs}

compare_out = EXT / "hw-inputmaps-compare-20261107.json"
join_out = EXT / "hw-join-vs-1b89-and-ci-20261107.json"
for p in (compare_out, join_out):
    if p.exists():
        p.unlink()
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "sho-compare-maps-20261107.py"), str(EXT / "hw-inputmap-PRE-20261107.json"), str(EXT / "hw-inputmap-POST-20261107.json"),
    str(EXT / "hw-extra-identity-PRE-20261107.json"), str(EXT / "hw-extra-identity-POST-20261107.json"), str(compare_out))
compare = json.loads(compare_out.read_text(encoding="utf-8"))
if not compare["all_rows_equal"] or not compare["extra_identity_equal"]:
    problems.append("PRE and POST maps differ")
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "hw-join-20261107.py"), str(REPO), str(EXT / "ci-lua-inputmap-POST-20261107.json"), str(EXT / "hw-inputmap-POST-20261107.json"), str(join_out))
join = json.loads(join_out.read_text(encoding="utf-8"))
if join["problems"]:
    problems.append("the join has problems: %s" % join["problems"])
shutil.copyfile(compare_out, PACK / "inputmaps-compare.json")
shutil.copyfile(join_out, PACK / "join-vs-1b89-and-ci.json")
maps = {name: {"bytes": (EXT / name).stat().st_size, "sha256": sha(EXT / name), "location": "external (full per-file rows)"}
        for name in ("hw-inputmap-PRE-20261107.json", "hw-inputmap-POST-20261107.json", "hw-extra-identity-PRE-20261107.json", "hw-extra-identity-POST-20261107.json")}

# the stored bytes are what Git actually holds: stage the changed sources and compare every literal claim against the index blob bytes
porcelain = subprocess.check_output(["git", "-C", str(REPO), "status", "--porcelain", "--untracked-files=all"], text=True).splitlines()
source_paths = [l[3:] for l in porcelain if not l[3:].startswith(SOURCES_EXCLUDED_PREFIXES)]
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
for rel in ("eech-campaign/reference/d3-entity-collision/sector-height-proof-guards-reviewed", "eech-campaign/reference/d3-entity-collision/sector-height-finite-native-ci-reviewed", "eech-campaign/reference/d3-ci-lua-download"):
    unchanged = subprocess.run(["git", "-C", str(REPO), "diff", "--quiet", "HEAD", "--", rel]).returncode == 0
    old_packets_unchanged[rel] = unchanged
    if not unchanged:
        problems.append("an older packet changed: " + rel)

guards = {}
for key, args in (("generic", []), ("production-cutover", ["--action", "production-cutover"]), ("fix-blocker ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1", ["--action", "fix-blocker", "--blocker", "ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1"]),
                  ("fix-blocker ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1", ["--action", "fix-blocker", "--blocker", "ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1"]),
                  ("fix-blocker sector-height-host-move-open-facts", ["--action", "fix-blocker", "--blocker", "sector-height-host-move-open-facts"]), ("deliver (expected unmet)", ["--action", "deliver"])):
    guards[key] = run(PY, "-B", "-X", "utf8", "eech-campaign/tools/migration-progress.py", *args).returncode

scripts = PACK / "scripts"
scripts.mkdir(exist_ok=True)
script_rows = []
for name in ("run-height-witness-direct-20261107.py", "check-height-witness-linux-20261107.sh", "run-hw-final-set-20261107.sh", "hw-join-20261107.py", "hw-extra-identity-20261107.py", "freeze-hw-witnesses-20261107.py",
             "patch-card-status-hw-20261107.py", "patch-card-status-height-checkpoint1-20261107.py", "publish-height-native-ci-reviewed-20261107.py", "sector-roster-inputmap-batched-20261106.py", "sho-compare-maps-20261107.py"):
    shutil.copyfile(EXT / name, scripts / name)
    script_rows.append({"name": name, "sha256": sha(scripts / name)})
subprocess.run(["git", "-C", str(REPO), "add", "-A", PACK_REL], check=True)

freeze = {
    "kind": "D3 sector-height M5/M6 witness source CODE_FROZEN (UNREVIEWED; test-only; native UNRUN; ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1 and ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1 stay OPEN)",
    "frozen_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "task": "EEDCS-49",
    "stage": "required witnesses inside the current sector-height extraction (not a preparation batch; the count stays 2); no production policy, kernel, protocol or C source of the original campaign changed",
    "source_before": head,
    "native_identity_not_relabelled": NATIVE,
    "findings": {"ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1": "OPEN until actual native proof", "ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1": "OPEN until actual native proof", "sector-height-host-move-open-facts": "OPEN"},
    "sealed_inputs": {name: {"sha256": sha(EXT / name)} for name in SEALED},
    "root_feedback_files_present": feedback_files,
    "lease": {"volume": VOLUME, "image": IMAGE, "image_named_never_substituted": True, "containers_before": lease_before[1:], "containers_after": lease_after, "cache_cleanup": "none",
              "tooling_note": "the leased image has no rustfmt or clippy; they were installed into each EPHEMERAL container by `rustup component add` (network read, no image, volume or global change); the unavailable-component failures are retained (w15, w16)"},
    "change": {
        "m5": "lua/sector-height-witness.lua modes clearance_original / clearance_control / clearance_changed_word / clearance_withheld / _malformed / _wrong_state / _wrong_sector: the guarded fixed-wing clearance observed at the host's own boundary (the phase 17 answers outside any scan and save; the hand-off in host.move_vehicle's fixed_wing_move carrying fixed_wing_terrain after fixed_wing_lookahead) at the declared accepted regression-pack window; no C hook and no created leader",
        "m6": "feature `sector-height-witness`: eech_hw_witness.c (the campaign's own store_session, refusing a bad name, a non-server/non-campaign session and any existing saved file), eech_engine_sh_witness_save, sector_height_witness_save in sys/engine/dc; driver modes save / save_withheld / load / load_withheld / load_wrong_sector / load_wrong_state / load_stale_word (the load is the campaign's own boot of the saved script)",
        "tools": "tools/build-windows-sector-height-witness.sh (a copy of the observer wrapper with one cargo argument)",
        "docs": "contract sections 8, 10, 11C and the new section 12 (what the source taught, the executable plan, the limits)",
        "process": "one concise lesson in docs/governance/D3-FIVE-WHYS-20261102.md",
    },
    "first_attempt_retained": {"pre_utc": "2026-11-07T11:28:34Z", "result": "ten children, nine exit 0 and v4 exit 101: host_callbacks every_reference_script_that_prepares_the_installation... expected 63 profile-selecting scripts and found 64 (the new witness driver is the 64th); an INTENTIONAL pinned-count change, classified and updated (63 -> 64, its comment) and the whole cycle repeated from a new PRE; the first attempt's maps, children and lease files are kept literally", "files": attempt1_rows},
    "test_counts_final": counts,
    "retained_development_runs": {label: {"actual_exit": code, "why": why} for label, (code, why) in RETAINED.items()},
    "runs": runs,
    "guards": guards,
    "input_maps": {"files": maps, "compare_pre_post": compare, "pre_utc": PRE_UTC, "post_utc": POST_UTC, "post_utc_note": "recorded by `date -u` immediately before the POST capture; not reconstructed from a file time", "join_vs_1b89_and_ci": join},
    "literal_bytes": {"changed_sources": source_literal, "lua_crlf_raw_vs_git_exceptions": crlf_exceptions, "core_filter_only_qualifications": filter_only, "older_packets_unchanged_against_HEAD": old_packets_unchanged},
    "audit_reader_timing": "this freeze script, the join, the extra-identity script, the final-set runner, the runner and the check script were FINAL before the PRE identity capture; no reader changed after the POST capture",
    "reused_unchanged": "the 1b89 native evidence (ordinary and observer shorts, hosted M4 scan, healthy height control, five matched modes), the accepted 179 characterisation and the finite 3665 and 0be/e326 matrices keep their true identity: they are reused only through the join, which shows from the Git diff that no C source of the original campaign changed and that the compiled additions are feature-gated (nothing is claimed natively for this changed, unbuilt-natively source)",
    "scripts": script_rows,
    "not_run": ["native, Windows, retail or war launch (Root's, after the SAME review)", "the witness runs themselves (natively UNRUN: `store_session`, `pack_session`, `unpack_session` and the guarded fw_move.c branch have not run here)", "i686 or EECH_C_OPT=2 builds (the feature-off sources are unchanged in effect; the gated fixture compiles in the default x86-64 profile only)",
                "the whole workspace or whole eech-world suites (the changed crates' own targets and the existing controls were run)", "the unchanged 16/49 and e326 matrices", "agents/MCP/authentication/model change", "cache, image or process deletion", "merge"],
    "limits": [
        "source and Linux checks only; nothing here is a native result, clearance or acceptance; both witness findings stay OPEN",
        "the scripted collaborator of tests/sector_height_witness_driver.rs stands in for the campaign: it checks the DRIVER (eligibility, single intervention, markers, fault controls), never fw_move.c, store_session, pack_session or unpack_session",
        "M5 is a natural reach at a declared window (the accepted regression-pack identity, 3 simulated hours): no leader was created and no actor, ray, seed or window was tuned; if no eligible leader meets the branch in the window the witness is UNREACHED",
        "the headless engine has never run store_session: its calls (the music stop, the directory listing, the script writers) are unproven in the dedicated-server build until the native run",
        "signalling-NaN payloads and engine -O2 stay unmeasured; seeds are the host's declared inputs, not natural save histories",
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
