"""Freezes the sector-height PUBLIC-PROOF-PLAN-FIX packet ONCE. Usage: freeze.py <repo-root> <external-freeze-path> <post-utc>"""
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
PACK_REL = "eech-campaign/reference/d3-entity-collision/sector-height-production-public-proof-plan-fix"
PACK = REPO / PACK_REL
RUNS = PACK / "runs"
SOURCE_BEFORE_PREFIX = "4a228c305"
CODE_BEFORE = "0be1dd8465d9bf38da69cd3b303206cf5a64574d"
SEALED = {
    "independent-sector-height-production-0be-source-20261107.json": "49d7739edb0855e5593bce91602eebcdedd8fb88c820677ee6695259ee9ce58c",
    "sector-height-production-Root-binding-0be-v2-20261107.json": "ec7015b66db79a17f84753aad2a3324bb0c693827959704bd4ca05db113a227d",
    "sector-height-production-Root-binding-0be-20261107.json": "7b5cc4a7986967707d30b2dd9db8bd6155dd26d4c6bae0c3b049b18d866636a3",
    "sector-height-production-1cf5-CI-readback-20261107.json": "03abecc4ae50b7b6b7f519daa454f8772365ca1f6344c81a0d3cc48a97655c18",
}
FEEDBACK = EXT / "Root-sector-height-production-feedback-20261107.json"
FEEDBACK_SHA = "db6f66ab8a21d0a653830a1ed75fc483aa1ed87bea5521b1ae5e56161fb58ba5"
PRE_UTC = (EXT / "sector-height-proofplan-PRE-utc-20261107.txt").read_text().strip()
EXPECTED_EXIT = {
    "n2-red-old-driver": (101, "RED: the three new driver tests run against the PREVIOUS driver (stashed, restored): they fail (no sector height modes)"),
    "n3-baseline-driver": (101, "the first run of the hosted-scan driver tests: the accepted report's per-sector rows were never read (a one-character prefix length); fixed in the driver"),
    "n6-red-old-helper": (101, "RED: the hosted clearance boundary test against the PREVIOUS observer helper (stashed, restored): it reads the retired member"),
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
if not head.startswith(SOURCE_BEFORE_PREFIX):
    problems.append("HEAD is not the checkpoint: " + head)
for name, want in SEALED.items():
    if sha(EXT / name) != want:
        problems.append("sealed input differs: " + name)
feedback_now = sha(FEEDBACK)
if feedback_now != FEEDBACK_SHA:
    problems.append("Root production feedback changed since it was read: " + feedback_now)

RUNS.mkdir(parents=True, exist_ok=True)
runs = []
for record_path in sorted(EXT.glob("sector-height-production-*.json")):
    name = record_path.name[len("sector-height-production-"):-len(".json")]
    if not re.match(r"^(i686-)?(n\d+|u\d+)-", name):
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
    label = re.sub(r"^i686-", "", name)
    final = bool(re.match(r"^(i686-)?u\d+-", name))
    runs.append({"label": name, "final": final, "command": record["command"], "actual_exit": record["actual_exit"], "started_utc": record["started_utc"], "completed_utc": record.get("completed_utc"),
                 "tests_passed": passed, "tests_failed": failed, "stdout_sha256": sha(RUNS / (name + ".stdout.log")), "stderr_sha256": sha(RUNS / (name + ".stderr.log"))})
    expected = EXPECTED_EXIT.get(label, (0, ""))[0]
    if record["actual_exit"] != expected:
        problems.append("unexpected exit for %s: %s (expected %s)" % (name, record["actual_exit"], expected))
final_runs = [r for r in runs if r["final"]]
if len(final_runs) != 11 or any(r["actual_exit"] != 0 for r in final_runs):
    problems.append("the final set is not 11 runs all exit 0: %s" % [(r["label"], r["actual_exit"]) for r in final_runs])

# the wrapper's structural check, parse only (no process is started)
ast = subprocess.run(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(EXT / "check-sho-windows-wrapper-ast-20261107.ps1"), str(REPO / "eech-campaign/tools/sector-height-baseline-windows.ps1")],
                     capture_output=True, text=True)
if ast.returncode != 0:
    problems.append("the wrapper AST check failed: " + ast.stdout[-300:])

compare_out = EXT / "sector-height-proofplan-inputmaps-compare-20261107.json"
join_out = EXT / "sector-height-proofplan-join-vs-0be-20261107.json"
for p in (compare_out, join_out):
    if p.exists():
        p.unlink()
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "sho-compare-maps-20261107.py"), str(EXT / "sector-height-proofplan-inputmap-PRE-20261107.json"), str(EXT / "sector-height-proofplan-inputmap-POST-20261107.json"),
    str(EXT / "sector-height-proofplan-extra-identity-PRE-20261107.json"), str(EXT / "sector-height-proofplan-extra-identity-POST-20261107.json"), str(compare_out))
compare = json.loads(compare_out.read_text(encoding="utf-8"))
if not compare["all_rows_equal"] or not compare["extra_identity_equal"]:
    problems.append("PRE and POST maps differ")
joined = run(PY, "-I", "-B", "-X", "utf8", str(EXT / "shp2-join-20261107.py"), str(REPO), str(EXT / "sector-height-production-inputmap-POST-20261107.json"), str(EXT / "sector-height-proofplan-inputmap-POST-20261107.json"), str(join_out))
join = json.loads(join_out.read_text(encoding="utf-8"))
if join["problems"]:
    problems.append("the join has problems: %s" % join["problems"])
shutil.copyfile(compare_out, PACK / "inputmaps-compare.json")
shutil.copyfile(join_out, PACK / "join-vs-0be-final-post.json")
maps = {}
for name in ("sector-height-proofplan-inputmap-PRE-20261107.json", "sector-height-proofplan-inputmap-POST-20261107.json", "sector-height-proofplan-extra-identity-PRE-20261107.json", "sector-height-proofplan-extra-identity-POST-20261107.json"):
    maps[name] = {"bytes": (EXT / name).stat().st_size, "sha256": sha(EXT / name), "location": "external (full per-file rows)"}

# the 7 raw-versus-filtered disagreements, by ACTUAL Git bytes (git cat-file of the committed blob), not by hash-object labels
post = json.loads((EXT / "sector-height-proofplan-inputmap-POST-20261107.json").read_text(encoding="utf-8"))["files"]
disagreements = [x if isinstance(x, str) else x.get("path") for x in compare["raw_vs_git_filtered_disagreements"]["post"]]
literal = {}
for path in sorted(disagreements):
    raw = (REPO / path).read_bytes()
    blob = git_bytes("cat-file", "blob", "HEAD:" + path)
    literal[path] = {"raw_bytes_equal_git_blob_bytes": raw == blob, "raw_sha256": hashlib.sha256(raw).hexdigest(), "git_blob_sha256": hashlib.sha256(blob).hexdigest(), "raw_crlf_lines": raw.count(b"\r\n"), "blob_crlf_lines": blob.count(b"\r\n")}
crlf_exceptions = sorted(p for p, v in literal.items() if not v["raw_bytes_equal_git_blob_bytes"])
filter_only = sorted(p for p, v in literal.items() if v["raw_bytes_equal_git_blob_bytes"])
if len(crlf_exceptions) != 3 or len(filter_only) != 4 or not all(p.endswith(".lua") for p in crlf_exceptions):
    problems.append("the raw-versus-Git qualification is not 3 Lua CRLF exceptions and 4 filter-only files: %s / %s" % (crlf_exceptions, filter_only))

guards = {}
for key, args in (("generic", []), ("production-cutover", ["--action", "production-cutover"]), ("fix-blocker sector-height-host-move-open-facts", ["--action", "fix-blocker", "--blocker", "sector-height-host-move-open-facts"]),
                  ("fix-blocker ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1", ["--action", "fix-blocker", "--blocker", "ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1"]), ("deliver (expected unmet)", ["--action", "deliver"])):
    guards[key] = run(PY, "-B", "-X", "utf8", "eech-campaign/tools/migration-progress.py", *args).returncode

scripts = PACK / "scripts"
scripts.mkdir(exist_ok=True)
script_rows = []
sources = [EXT / n for n in ("run-shp2-final-set-20261107.sh", "shp2-extra-identity-20261107.py", "shp2-join-20261107.py", "freeze-sector-height-proofplan-20261107.py", "patch-card-register-proof-plan-finding-20261107.py",
                             "patch-card-status-proofplan-20261107.py", "sho-compare-maps-20261107.py", "check-sho-windows-wrapper-ast-20261107.ps1")]
sources += [pathlib.Path(p) for p in sorted(glob.glob("C:/Users/starscream/AppData/Local/Temp/sh_prod2/*.py"))]
for src in sources:
    target = scripts / src.name
    shutil.copyfile(src, target)
    script_rows.append({"name": src.name, "sha256": sha(target)})
for name in SEALED:
    shutil.copyfile(EXT / name, PACK / name)
shutil.copyfile(FEEDBACK, PACK / FEEDBACK.name)

subprocess.run(["git", "-C", str(REPO), "add", "-A", PACK_REL], check=True)
literal_check = {}
for name in list(SEALED) + [FEEDBACK.name]:
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
    "kind": "D3 sector-height PUBLIC-PROOF-PLAN-FIX CODE_FROZEN (test-only witness and plan correction of the SAME review's P2; UNREVIEWED; native UNRUN)",
    "frozen_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "task": "EEDCS-49",
    "stage": "required bounded correction inside the height extraction (not a preparation batch; the count stays 2); no production solver, protocol or kernel change",
    "source_before": head,
    "production_code": CODE_BEFORE,
    "finding": {"id": "ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1", "severity": "P2", "review": {"path": "R:/d3-evidence/independent-sector-height-production-0be-source-20261107.json", "sha256": SEALED["independent-sector-height-production-0be-source-20261107.json"]},
                "state": "OPEN until the SAME reviewer closes it; registered OPEN in the card at the first checkpoint (4a228c305)"},
    "sealed_inputs": {name: {"sha256": sha(EXT / name)} for name in SEALED},
    "root_feedback": {"path": str(FEEDBACK), "sha256": feedback_now, "note": "unchanged since it was read; present before the PRE map"},
    "initial_root_binding_qualification": "sector-height-production-Root-binding-0be-20261107.json (7b5cc4a7...) confused filtered identities with raw bytes; it is kept literally beside the corrected v2 binding (ec7015b6...); direct Git-byte comparison shows exactly 3 unchanged Lua CRLF raw-versus-Git exceptions and 4 Core filter-only qualifications (raw bytes equal the Git blob): literal-bytes-check.json",
    "correction": {
        "driver_modes": "entity-authority.lua: sector_height_control, _wrong_word, _withheld, _malformed, _wrong_state, _wrong_sector: attribution of a phase-17 read to the real scan FIRST, eligibility (armed, joined by sector, a non-mobile served entry, complete reply), consumption shown by the fact 12 of a candidate question of that entry, named outcomes, no restore credit",
        "observer_scope": "sector-height-baseline.lua scope=hosted-scan (+ accepted=): only the scan rows and clearance guard records, compared by sector at the same window with the accepted original report; the wrapper gains -Scope and -Accepted (parse-only AST check passes)",
        "m5_helper": "eech_sho.c clearance helper: in a hosted build it reads no retired member and records `tallest=uncaptured`; boundary test sector_height_hosted_clearance.rs (RED against the old helper)",
        "prose": "contract sections 7 to 10 corrected and section 11 added (commands, features, modes, row schema, eligibility, expected outcomes); restore-after-insertion is NOT a refusal; no control reaches the guarded fw execution; scripted tests are boundary checks",
    },
    "changed_paths": changed,
    "tracked_lines_deleted_by_file": deleted,
    "guards": guards,
    "runs": runs,
    "retained_failures_and_recoveries": {label: {"actual_exit": code, "why": why} for label, (code, why) in EXPECTED_EXIT.items()},
    "input_maps": {"files": maps, "compare_pre_post": compare, "pre_utc": PRE_UTC, "post_utc": POST_UTC, "post_utc_note": "recorded by `date -u` immediately before the POST capture; not reconstructed from a file time",
                   "no_source_write_between": "the final set u1-u11 ran between the maps; the packet files under reference/ and STATUS and the card are not inputs of any check and were written after the POST capture",
                   "join_vs_0be1dd846_final_post": join},
    "reused_unchanged_matrices": "the 0be1dd846 matrices (16 final checks, 190 packet files, 49 runs) are reused only through the explicit join: no aphavoc/ or modules/ path and no non-test crate source changed; every changed row has its own affected check here",
    "scripts": script_rows,
    "cache": {"volume": "eech-linux-d3-collision-driver-5416adb3b-20261029", "owners": "one (this lane); no container before or after", "cleanup": "none performed; the Root-prepared 1cf5 clone was not touched"},
    "not_run": ["Windows build", "native, campaign or war launch", "retail data", "the full 16-check and 49-run matrices", "agents/MCP/authentication/model change", "cache or process deletion", "merge"],
    "limits": [
        "source and Linux checks only; nothing here is a native result or a native clearance; every native run is Root's after the SAME changed-material review",
        "the scripted tests that make the campaign read or restore a sector height are boundary checks of the drivers, not native campaign authority; the comparison reuses the accepted original evidence only at its true scope (63 FIRST reads over 2 sectors, 27615 fact-12 offers)",
        "ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1 and ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1 stay required OPEN: the fixture entries they need (a terrain-following task-leader fixed wing; a real server-session save and load) do not exist and are not invented",
        "no signalling-NaN or engine -O2 claim; no baseline, input, seed, frame, duration or tolerance changed",
    ],
    "state": {"finding": "ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1 OPEN until the SAME reviewer", "sector-height-host-move-open-facts": "OPEN", "claims": "no self-review, approval, native clearance, whole Task49 or D3, delivery", "D3-CURRENT.json": "not edited (Root owns it)"},
    "hygiene_problems_at_freeze": problems,
}
FREEZE.write_text(json.dumps(freeze, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
shutil.copyfile(FREEZE, PACK / FREEZE.name)
if not (PACK / ".gitattributes").exists():
    shutil.copyfile(REPO / "eech-campaign/reference/d3-entity-collision/sector-height-public-baseline-reviewed/.gitattributes", PACK / ".gitattributes")
files = [{"path": str(p.relative_to(PACK)).replace("\\", "/"), "bytes": p.stat().st_size, "sha256": sha(p)} for p in sorted(PACK.rglob("*")) if p.is_file() and p.name != "manifest.json"]
(PACK / "manifest.json").write_text(json.dumps({"files": files, "scope": "Source and Linux-check packet of the public-proof-plan correction only; not a native result, review, approval or acceptance."}, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps({"problems": problems, "runs": len(runs), "final_runs": len(final_runs), "changed": len(changed), "files": len(files), "guards": guards, "crlf_exceptions": crlf_exceptions, "filter_only": filter_only}, indent=1))
sys.exit(1 if problems else 0)
