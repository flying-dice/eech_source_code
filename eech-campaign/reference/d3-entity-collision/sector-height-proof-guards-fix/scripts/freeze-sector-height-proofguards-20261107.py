"""Freezes the sector-height PROOF-GUARDS-FIX packet ONCE. Usage: freeze.py <repo-root> <external-freeze-path> <post-utc>
This script, shp3-join-20261107.py, shp3-extra-identity-20261107.py, run-shp3-final-set-20261107.sh and check-real-accepted-shape-20261107.py are FINAL before the PRE identity capture."""
import datetime
import glob
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
PACK_REL = "eech-campaign/reference/d3-entity-collision/sector-height-proof-guards-fix"
PACK = REPO / PACK_REL
RUNS = PACK / "runs"
CHECKPOINT_PREFIX = "fe1bf6723"
PREVIOUS_CODE = "5aae2256f02ca3d79e762d71c9b79c20dc34856b"
SEALED = {
    "independent-sector-height-proofplan-5aae-source-20261107.json": "409a2622b5314d82c02dc7710dcf3528f8390678e5b11162108847bcb83aa27b",
    "sector-height-proofplan-3f-CI-readback-20261107.json": "bb46f014e64e3fc3b0d5100e1879b015eed60ed797418f4c5719424711af7ce3",
    "sector-height-proofplan-Root-binding-5aae-20261107.json": "6ec510ae1c6c492e8cca6308aa14f98d84f4200de17692670af408b0533aaf2f",
}
PRE_UTC = (EXT / "sector-height-proofguards-PRE-utc-20261107.txt").read_text().strip()
EXPECTED_EXIT = {
    "g1-red-old-baseline": (101, "RED: the single-fault matrices of the scoped-report validation against the PREVIOUS comparator: the damaged accepted and live hosted inputs were not refused"),
    "g3-red-old-driver": (101, "the first run of the injection controls: the late scripted read of wrong_word is incoherent (a candidate question before the read refuses a stale fact 12); the control was restructured"),
    "g4-red-old-driver": (101, "RED: the injection controls against the verdict as it was at 5aae2256f (wrong_word stopped for another reason)"),
    "g5-driver": (101, "after the verdict fix: the clean-UNREACHED branch was hit by the exactly-once check placed before it, and the late scripted read refused a stale fact 12; both fixed"),
    "g7-red-previous-verdict": (101, "RED: with the previous driver verdict an unrelated fatal bearing sector_height_withheld's own marker, after a configured hit and before any eligible read, was CREDITED"),
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
head = git("rev-parse", "HEAD")
if not head.startswith(CHECKPOINT_PREFIX):
    problems.append("HEAD is not the checkpoint: " + head)
for name, want in SEALED.items():
    if sha(EXT / name) != want:
        problems.append("sealed input differs: " + name)
feedback_files = sorted(p.name for p in EXT.glob("Root-*feedback*"))

RUNS.mkdir(parents=True, exist_ok=True)
runs = []
for record_path in sorted(EXT.glob("sector-height-production-*.json")):
    name = record_path.name[len("sector-height-production-"):-len(".json")]
    if not re.match(r"^[gv]\d+-", name):
        continue
    if not (EXT / ("sector-height-production-%s.stdout.log" % name)).exists():
        continue
    record = json.loads(record_path.read_text(encoding="utf-8"))
    for suffix in ("json", "stdout.log", "stderr.log"):
        shutil.copyfile(EXT / ("sector-height-production-%s.%s" % (name, suffix)), RUNS / ("%s.%s" % (name, suffix)))
    if record.get("state") != "terminal":
        problems.append("run not terminal: " + name)
    text = (RUNS / (name + ".stdout.log")).read_text(encoding="utf-8", errors="replace")
    passed = sum(int(m) for m in re.findall(r"test result: \w+\. (\d+) passed", text))
    failed = sum(int(m) for m in re.findall(r"; (\d+) failed", text))
    final = bool(re.match(r"^v\d+-", name))
    runs.append({"label": name, "final": final, "command": record["command"], "actual_exit": record["actual_exit"], "started_utc": record["started_utc"], "completed_utc": record.get("completed_utc"),
                 "tests_passed": passed, "tests_failed": failed, "stdout_sha256": sha(RUNS / (name + ".stdout.log")), "stderr_sha256": sha(RUNS / (name + ".stderr.log"))})
    expected = EXPECTED_EXIT.get(name, (0, ""))[0]
    if record["actual_exit"] != expected:
        problems.append("unexpected exit for %s: %s (expected %s)" % (name, record["actual_exit"], expected))
final_runs = [r for r in runs if r["final"]]
if len(final_runs) != 4 or any(r["actual_exit"] != 0 for r in final_runs):
    problems.append("the final cargo set is not 4 runs all exit 0: %s" % [(r["label"], r["actual_exit"]) for r in final_runs])

progress = (EXT / "shp3-final-set-20261107.progress").read_text()
if "v5-real-accepted-shape exit=0" not in progress or "DONE" not in progress:
    problems.append("the real accepted shape check did not finish with exit 0: " + progress)
shape_text = (EXT / "shp3-real-accepted-shape-20261107.txt").read_text()
accepted_sha = (EXT / "shp3-real-accepted-sha-20261107.txt").read_text().split()[0]
for name in ("shp3-real-accepted-shape-20261107.txt", "shp3-real-accepted-sha-20261107.txt", "shp3-final-set-20261107.progress"):
    shutil.copyfile(EXT / name, RUNS / name)

ast = subprocess.run(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(EXT / "check-sho-windows-wrapper-ast-20261107.ps1"), str(REPO / "eech-campaign/tools/sector-height-baseline-windows.ps1")],
                     capture_output=True, text=True)
if ast.returncode != 0:
    problems.append("the wrapper AST check failed: " + ast.stdout[-300:])

compare_out = EXT / "sector-height-proofguards-inputmaps-compare-20261107.json"
join_out = EXT / "sector-height-proofguards-join-vs-5aae-20261107.json"
for p in (compare_out, join_out):
    if p.exists():
        p.unlink()
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "sho-compare-maps-20261107.py"), str(EXT / "sector-height-proofguards-inputmap-PRE-20261107.json"), str(EXT / "sector-height-proofguards-inputmap-POST-20261107.json"),
    str(EXT / "sector-height-proofguards-extra-identity-PRE-20261107.json"), str(EXT / "sector-height-proofguards-extra-identity-POST-20261107.json"), str(compare_out))
compare = json.loads(compare_out.read_text(encoding="utf-8"))
if not compare["all_rows_equal"] or not compare["extra_identity_equal"]:
    problems.append("PRE and POST maps differ")
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "shp3-join-20261107.py"), str(REPO), str(EXT / "sector-height-proofplan-inputmap-POST-20261107.json"), str(EXT / "sector-height-proofguards-inputmap-POST-20261107.json"), str(join_out))
join = json.loads(join_out.read_text(encoding="utf-8"))
if join["problems"]:
    problems.append("the join has problems: %s" % join["problems"])
shutil.copyfile(compare_out, PACK / "inputmaps-compare.json")
shutil.copyfile(join_out, PACK / "join-vs-5aae-final-post.json")
maps = {}
for name in ("sector-height-proofguards-inputmap-PRE-20261107.json", "sector-height-proofguards-inputmap-POST-20261107.json", "sector-height-proofguards-extra-identity-PRE-20261107.json", "sector-height-proofguards-extra-identity-POST-20261107.json"):
    maps[name] = {"bytes": (EXT / name).stat().st_size, "sha256": sha(EXT / name), "location": "external (full per-file rows)"}

disagreements = [x if isinstance(x, str) else x.get("path") for x in compare["raw_vs_git_filtered_disagreements"]["post"]]
subprocess.run(["git", "-C", str(REPO), "add", "-u"], check=True)  # stage the modified tracked sources so the literal bytes are what Git actually stores
literal = {}
for path in sorted(disagreements):
    raw = (REPO / path).read_bytes()
    blob = git_bytes("cat-file", "blob", ":" + path)
    literal[path] = {"raw_bytes_equal_git_blob_bytes": raw == blob, "raw_sha256": hashlib.sha256(raw).hexdigest(), "git_blob_sha256": hashlib.sha256(blob).hexdigest(), "raw_crlf_lines": raw.count(b"\r\n"), "blob_crlf_lines": blob.count(b"\r\n")}
crlf_exceptions = sorted(p for p, v in literal.items() if not v["raw_bytes_equal_git_blob_bytes"])
filter_only = sorted(p for p, v in literal.items() if v["raw_bytes_equal_git_blob_bytes"])
if len(crlf_exceptions) != 3 or len(filter_only) != 4 or not all(p.endswith(".lua") for p in crlf_exceptions):
    problems.append("the raw-versus-Git qualification is not 3 Lua CRLF exceptions and 4 filter-only files: %s / %s" % (crlf_exceptions, filter_only))

guards = {}
for key, args in (("generic", []), ("production-cutover", ["--action", "production-cutover"]), ("fix-blocker sector-height-host-move-open-facts", ["--action", "fix-blocker", "--blocker", "sector-height-host-move-open-facts"]),
                  ("fix-blocker ENV-SECTOR-HEIGHT-SCOPED-REPORT-COMPLETE-1", ["--action", "fix-blocker", "--blocker", "ENV-SECTOR-HEIGHT-SCOPED-REPORT-COMPLETE-1"]),
                  ("fix-blocker ENV-SECTOR-HEIGHT-REFUSAL-INTERVENTION-1", ["--action", "fix-blocker", "--blocker", "ENV-SECTOR-HEIGHT-REFUSAL-INTERVENTION-1"]), ("deliver (expected unmet)", ["--action", "deliver"])):
    guards[key] = run(PY, "-B", "-X", "utf8", "eech-campaign/tools/migration-progress.py", *args).returncode

scripts = PACK / "scripts"
scripts.mkdir(exist_ok=True)
script_rows = []
sources = [EXT / n for n in ("run-shp3-final-set-20261107.sh", "shp3-extra-identity-20261107.py", "shp3-join-20261107.py", "check-real-accepted-shape-20261107.py", "freeze-sector-height-proofguards-20261107.py",
                             "patch-card-register-proof-guards-findings-20261107.py", "patch-card-status-proofguards-20261107.py", "sho-compare-maps-20261107.py", "check-sho-windows-wrapper-ast-20261107.ps1")]
sources += [pathlib.Path(p) for p in sorted(glob.glob("C:/Users/starscream/AppData/Local/Temp/sh_prod3/*.py"))]
for src in sources:
    target = scripts / src.name
    shutil.copyfile(src, target)
    script_rows.append({"name": src.name, "sha256": sha(target)})
for name in SEALED:
    shutil.copyfile(EXT / name, PACK / name)

subprocess.run(["git", "-C", str(REPO), "add", "-A", PACK_REL], check=True)
literal_check = {}
for name in SEALED:
    blob = git_bytes("cat-file", "blob", ":" + PACK_REL + "/" + name)
    source = (EXT / name).read_bytes()
    literal_check[name] = {"git_index_blob_bytes_equal_external_file": blob == source, "sha256": hashlib.sha256(source).hexdigest()}
    if blob != source:
        problems.append("the packet copy of %s is not the literal file in Git" % name)
(PACK / "literal-bytes-check.json").write_text(json.dumps({"kind": "actual Git blob bytes (git cat-file from the index) of each literal copy against the external sealed file", "files": literal_check,
                                                          "raw_versus_git_disagreements_of_the_input_map": literal, "lua_crlf_raw_vs_git_exceptions": crlf_exceptions, "core_filter_only_qualifications": filter_only}, indent=2) + "\n", encoding="utf-8", newline="\n")

status = subprocess.check_output(["git", "-C", str(REPO), "status", "--porcelain", "--untracked-files=all"], text=True).splitlines()
changed = []
for line in status:
    path = line[3:]
    if path.startswith(PACK_REL + "/"):
        continue
    changed.append({"status": line[:2].strip(), "path": path, "sha256": sha(REPO / path), "git_blob_of_bytes": git("hash-object", path)})
numstat = subprocess.check_output(["git", "-C", str(REPO), "diff", "--numstat"], text=True).splitlines()
deleted = {l.split("\t")[2]: int(l.split("\t")[1]) for l in numstat if l.split("\t")[1] not in ("0", "-")}

freeze = {
    "kind": "D3 sector-height PROOF-GUARDS-FIX CODE_FROZEN (two false-pass diagnostics of the SAME review 409a2622; test-only Lua diagnostics and driver tests; UNREVIEWED; native UNRUN)",
    "frozen_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "task": "EEDCS-49",
    "stage": "required bounded correction inside the height extraction (not a preparation batch; the count stays 2); no production C, protocol or kernel change",
    "source_before": head,
    "previous_correction_code": PREVIOUS_CODE,
    "findings": {
        "ENV-SECTOR-HEIGHT-SCOPED-REPORT-COMPLETE-1": "OPEN until the SAME reviewer; registered OPEN at fe1bf6723",
        "ENV-SECTOR-HEIGHT-REFUSAL-INTERVENTION-1": "OPEN until the SAME reviewer; registered OPEN at fe1bf6723",
        "ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1": "partially addressed, stays OPEN until the SAME reviewer closes all",
    },
    "sealed_inputs": {name: {"sha256": sha(EXT / name)} for name in SEALED},
    "root_feedback_files_present": feedback_files,
    "root_feedback_note": "no new Root feedback file exists since the production feedback (06:40Z); all review and CI material was available before the PRE map",
    "ci_note": "the exact 3f CI read-back (all nine jobs success) arrived after the reviewer's exclusive seal, so the review's own 8/9 observation stays historical; green CI is no physical or native acceptance",
    "correction": {
        "scoped_report_complete": "sector-height-baseline.lua validates BOTH sides as a whole scoped scan before comparing (the accepted original: exactly one identity/sho-scan/sho-tables/sho-verdict, the verdict last with whole=1 and the sector count; both: exact fields and domains, unique contiguous indices, unique sectors, declared count, totals and words consistent with the rows; hosted not asked for original-link completeness); statuses INVALID_ACCEPTED and INVALID_HOSTED; UNREACHED only for a clean whole scan with no read; positive-first matrices of 29 accepted and 17 live hosted single faults",
        "refusal_intervention": "entity-authority.lua: a height refusal is credited only with height_altered=1 (exactly the one eligible intervention), independent of the secondary-input counters and marker equality; any fatal before it is FAILURE (the mode's own marker included); clean absence is UNREACHED; per-mode positive then an injected unrelated fatal bearing the mode's marker after a configured hit; wrong-marker and request-fault controls",
        "real_accepted_report": {"check": "check-real-accepted-shape-20261107.py (an independent port of the same rules) over the retained accepted report", "output": shape_text.strip(), "report_sha256": accepted_sha},
    },
    "changed_paths": changed,
    "tracked_lines_deleted_by_file": deleted,
    "guards": guards,
    "runs": runs,
    "retained_failures_and_recoveries": {label: {"actual_exit": code, "why": why} for label, (code, why) in EXPECTED_EXIT.items()},
    "input_maps": {"files": maps, "compare_pre_post": compare, "pre_utc": PRE_UTC, "post_utc": POST_UTC, "post_utc_note": "recorded by `date -u` immediately before the POST capture; not reconstructed from a file time",
                   "no_source_write_between": "the final set v1-v5 ran between the maps; the packet files under reference/, STATUS and the card are not inputs of any check and were written after the POST capture",
                   "join_vs_5aae_final_post": join},
    "audit_reader_timing": "this freeze script, the join, the extra-identity script, the final-set script and the real-report check were FINAL before the PRE identity capture (contrast the 5aae batch, where the join reader was corrected 08:54:55Z after its POST: a post-processing audit reader, not a test or source input, no replay was demanded); the old 4836-byte PRE/POST extra-identity files of that batch (SHA 2dd277db...) and the final 4852-byte reader (SHA ca27dda8...) are not all literal-current, and are kept as stated",
    "reused_unchanged_matrices": "the 0be1dd846 matrices (16 final checks, 190 files, 49 runs) and the 5aae2256f u1-u11 (19 runs) are reused only through the explicit joins: no aphavoc/ or modules/ path, no C, no non-test crate source and no Cargo change; every changed row has its own affected check here",
    "scripts": script_rows,
    "cache": {"volume": "eech-linux-d3-collision-driver-5416adb3b-20261029", "owners": "one (this lane); no container before or after", "cleanup": "none performed; the Root-prepared 3f snapshot was not touched"},
    "not_run": ["Windows build", "native, campaign or war launch", "retail data (only the retained accepted report was READ by the shape check)", "C, x87, legacy workspace or whole-World reruns", "the full 16-check, 49-run or 11-check matrices", "agents/MCP/authentication/model change", "cache or process deletion", "merge"],
    "limits": [
        "source and Linux checks only; nothing here is a native result or clearance; the scripted tests are boundary checks of the diagnostics, not native campaign authority",
        "the real accepted report was checked by an independent Python port of the Lua rules, not by running the Lua loader over it (the host Lua tests use scripted reports); the first native comparison is Root's",
        "ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1 and ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1 stay required OPEN; no baseline, input, seed, frame, duration or tolerance changed",
    ],
    "state": {"findings": "OPEN until the SAME reviewer", "sector-height-host-move-open-facts": "OPEN", "claims": "no self-review, approval, native clearance, finding closure, whole Task49 or D3, delivery", "D3-CURRENT.json": "not edited (Root owns it)"},
    "hygiene_problems_at_freeze": problems,
}
FREEZE.write_text(json.dumps(freeze, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
shutil.copyfile(FREEZE, PACK / FREEZE.name)
if not (PACK / ".gitattributes").exists():
    shutil.copyfile(REPO / "eech-campaign/reference/d3-entity-collision/sector-height-public-baseline-reviewed/.gitattributes", PACK / ".gitattributes")
files = [{"path": str(p.relative_to(PACK)).replace("\\", "/"), "bytes": p.stat().st_size, "sha256": sha(p)} for p in sorted(PACK.rglob("*")) if p.is_file() and p.name != "manifest.json"]
(PACK / "manifest.json").write_text(json.dumps({"files": files, "scope": "Source and Linux-check packet of the proof-guards correction only; not a native result, review, approval or acceptance."}, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps({"problems": problems, "runs": len(runs), "final_runs": len(final_runs), "changed": len(changed), "files": len(files), "guards": guards, "crlf_exceptions": crlf_exceptions, "filter_only": filter_only}, indent=1))
sys.exit(1 if problems else 0)
