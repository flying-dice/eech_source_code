"""Freezes the sector-height PRODUCTION packet ONCE. Usage: freeze.py <repo-root> <external-freeze-path> <post-utc>"""
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
PACK = REPO / "eech-campaign/reference/d3-entity-collision/sector-height-production-source-and-checks"
RUNS = PACK / "runs"
SOURCE_BEFORE = "93495d1b6ddc519f8ec4fee9c6b1147d2c292f39"
REVIEW = EXT / "independent-sector-height-capacity-f01-native-20261107.json"
REVIEW_SHA = "3665b264316b598b7d9892bb2bd66645a935c97bed68afdea0673141cbde498e"
DISPOSITION = EXT / "Root-sector-height-production-start-disposition-20261107.json"
FEEDBACK = EXT / "Root-sector-height-production-feedback-20261107.json"
FEEDBACK_SHA = "db6f66ab8a21d0a653830a1ed75fc483aa1ed87bea5521b1ae5e56161fb58ba5"
PRE_UTC = (EXT / "sector-height-production-PRE-utc-20261107.txt").read_text().strip()

# dev and recovery runs whose exit is not 0, and why (retained, never relabelled)
EXPECTED_EXIT = {
    "c5-world-tests": (101, "the whole eech-world suite over the first Host source: 32 fixtures failed (every fact-12 value the tests carried was not the host's word; the driver tests' real provider had no model installation; the city state's new shape fact) - the RED of the fixture changes"),
    "c6-mtp": (101, "after the helper fixtures held 1000.0 and the driver tests had a root: 14 tests still failed (a root with no scene; negative sector words; the city shape fact)"),
    "c7-mtp": (101, "2 failures left (the city state grammar fixture's stray fact 9 was now the shape; a city skip that needed the negative word again)"),
    "c8-world": (101, "the raw candidate classifier test and a new test: both classifiers still demanded fact 12 be finite"),
    "c11-driver": (101, "the new driver test: the window counters had no sector_height field"),
    "c12-driver": (101, "the new driver test asserted the collision answer counter 'altered', which is not about the height"),
    "c14-sys": (101, "the engine-sys guards pinned the old census (city state words, the pack and getter statements, the readers of the float type): the intentional census change"),
    "c15-sys": (101, "one guard's enclosing-condition expectation missed the GET_LOCAL_REQUIRED wrapper"),
    "c17-obs-hosted": (101, "the observer recorder's compile-time pin on the city statement width (10 words) met the 11-word statement"),
    "c18-obs-legacy": (101, "the observer control pinned 179 expectations; the original-solver profile now runs 179 + 42 differential"),
    "f1-fmt-check": (1, "rustfmt differences in the new tests and converters (applied by f2)"),
    "f3-clippy": (101, "an unused field in the new host test (strict lints)"),
    "f4-clippy": (101, "expect and panic in two lib tests of the strictly linted crates"),
}


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def git(*args):
    return subprocess.check_output(["git", "-C", str(REPO)] + list(args), text=True).strip()


def run(*args):
    return subprocess.run(args, cwd=str(REPO), capture_output=True, text=True)


problems = []
head = git("rev-parse", "HEAD")
if head != SOURCE_BEFORE:
    problems.append("HEAD is not the source before: " + head)
if sha(REVIEW) != REVIEW_SHA:
    problems.append("review hash differs")
feedback_now = sha(FEEDBACK)
if feedback_now != FEEDBACK_SHA:
    problems.append("Root feedback changed since it was read: " + feedback_now)

RUNS.mkdir(parents=True, exist_ok=True)
runs = []
for record_path in sorted(EXT.glob("sector-height-production-*.json")):
    name = record_path.name[len("sector-height-production-"):-len(".json")]
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
    final = bool(re.match(r"^(i686-)?t\d+-", name))
    runs.append({"label": name, "final": final, "command": record["command"], "actual_exit": record["actual_exit"], "started_utc": record["started_utc"], "completed_utc": record.get("completed_utc"),
                 "tests_passed": passed, "tests_failed": failed, "stdout_sha256": sha(RUNS / (name + ".stdout.log")), "stderr_sha256": sha(RUNS / (name + ".stderr.log"))})
    expected = EXPECTED_EXIT.get(label, (0, ""))[0]
    if record["actual_exit"] != expected:
        problems.append("unexpected exit for %s: %s (expected %s)" % (name, record["actual_exit"], expected))
final_runs = [r for r in runs if r["final"]]
if len(final_runs) != 16 or any(r["actual_exit"] != 0 for r in final_runs):
    problems.append("the final set is not 16 runs all exit 0: %s" % [(r["label"], r["actual_exit"]) for r in final_runs])

# the maps: PRE and POST rows equal, tool identity equal
compare_out = EXT / "sector-height-production-inputmaps-compare-20261107.json"
if compare_out.exists():
    compare_out.unlink()
done = run(PY, "-I", "-B", "-X", "utf8", str(EXT / "sho-compare-maps-20261107.py"), str(EXT / "sector-height-production-inputmap-PRE-20261107.json"), str(EXT / "sector-height-production-inputmap-POST-20261107.json"),
           str(EXT / "sector-height-production-extra-identity-PRE-20261107.json"), str(EXT / "sector-height-production-extra-identity-POST-20261107.json"), str(compare_out))
compare = json.loads(compare_out.read_text(encoding="utf-8"))
if not compare["all_rows_equal"] or not compare["extra_identity_equal"]:
    problems.append("PRE and POST maps differ")
shutil.copyfile(compare_out, PACK / "inputmaps-compare.json")
maps = {}
for name in ("sector-height-production-inputmap-PRE-20261107.json", "sector-height-production-inputmap-POST-20261107.json", "sector-height-production-extra-identity-PRE-20261107.json", "sector-height-production-extra-identity-POST-20261107.json"):
    maps[name] = {"bytes": (EXT / name).stat().st_size, "sha256": sha(EXT / name), "location": "external (full per-file rows)"}

# the guards the lane must run before and at the freeze
guards = {}
for key, args in (("generic", []), ("production-cutover", ["--action", "production-cutover"]), ("fix-blocker sector-height-host-move-open-facts", ["--action", "fix-blocker", "--blocker", "sector-height-host-move-open-facts"]),
                  ("deliver (expected unmet)", ["--action", "deliver"])):
    guards[key] = run(PY, "-B", "-X", "utf8", "eech-campaign/tools/migration-progress.py", *args).returncode

scripts = PACK / "scripts"
scripts.mkdir(exist_ok=True)
script_rows = []
sources = [EXT / n for n in ("run-sector-height-production-direct-20261107.py", "run-sector-height-production-i686-20261107.py", "run-shp-final-set-20261107.sh", "shp-extra-identity-20261107.py",
                             "sho-compare-maps-20261107.py", "build-sector-height-reviewed-packet-20261107.py", "freeze-sector-height-production-20261107.py")]
sources += [pathlib.Path(p) for p in sorted(glob.glob("C:/Users/starscream/AppData/Local/Temp/sh_prod/*.py"))]
cutoff = datetime.datetime(2026, 11, 7, 6, 45, tzinfo=datetime.timezone.utc).timestamp()
sources += [pathlib.Path(p) for p in sorted(glob.glob("C:/Users/starscream/AppData/Local/Temp/ed_*.py")) if pathlib.Path(p).stat().st_mtime >= cutoff]
for src in sources:
    target = scripts / src.name
    shutil.copyfile(src, target)
    script_rows.append({"name": src.name, "sha256": sha(target)})
shutil.copyfile(REVIEW, PACK / REVIEW.name)
shutil.copyfile(FEEDBACK, PACK / FEEDBACK.name)

status = subprocess.check_output(["git", "-C", str(REPO), "status", "--porcelain", "--untracked-files=all"], text=True).splitlines()
changed = []
for line in status:
    path = line[3:]
    if path.startswith("eech-campaign/reference/d3-entity-collision/sector-height-production-source-and-checks/"):
        continue
    changed.append({"status": line[:2].strip(), "path": path, "sha256": sha(REPO / path), "git_blob_of_bytes": git("hash-object", path)})
numstat = subprocess.check_output(["git", "-C", str(REPO), "diff", "--numstat"], text=True).splitlines()
deleted = {l.split("\t")[2]: int(l.split("\t")[1]) for l in numstat if l.split("\t")[1] not in ("0", "-")}

freeze = {
    "kind": "D3 sector-height PRODUCTION CODE_FROZEN: the Host-owned per-sector high-water height (UNREVIEWED; native UNRUN; the move is built in source, not accepted)",
    "frozen_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "task": "EEDCS-49",
    "stage": "the REQUIRED production move of the sector high-water scalar (not a preparation batch; the count stays 2)",
    "source_before": head,
    "published_before_any_production_edit": {"evidence_commit": "7e3d6afa0", "metadata_commit": "93495d1b6", "packet": "eech-campaign/reference/d3-entity-collision/sector-height-public-baseline-reviewed"},
    "sealed_inputs": {"review": {"path": str(REVIEW), "sha256": sha(REVIEW)}, "root_disposition": {"path": str(DISPOSITION), "sha256": sha(DISPOSITION)},
                      "root_feedback": {"path": str(FEEDBACK), "sha256": feedback_now,
                                        "timing": "it did not exist at the preflight listing and at the checkpoint guard run; it was written at 06:40Z during the work and I read it at 07:21Z, before the final PRE map and every final check; it is a publication readback with no scope or test request"}},
    "design": {
        "authority": "one word per sector in the host's roster (roster.rs), raised at the ARRIVAL = the first FIXED_STATE or CITY_STATE statement under a sector placement's key after its insertion; the host's own scene table gives the box top; the arithmetic is one C kernel (model_step.c, expression csrc/sector_height_raise.h) under the model kernels' environment guard",
        "reads": "the sector float getter (sc_float.c) asks the host by the sector's coordinates (scan phase 17): the one read for the scan FIRST (unchanged text), the original-solver body, the fixed-wing clearance (unchanged text) and the pack; the candidate question's fact 12 is the host's word passed on and the host compares its bits",
        "restore": "scan phase 18 states the unpacked raw word to the host, which stores exactly those bits; the campaign checks the acknowledgement before the load goes on",
        "retired": "the legacy field is never written or read in a hosted build (census-guarded); no mirror, no fallback; the original body is compiled only where the original entity solver is kept (collision-fixture, model-fixture)",
        "city": "CITY_STATE gains the shape word (fact 9): a city raises only with a valid shape; the accepted baseline's 14395 cities were all invalid",
        "classifiers": "fact 12 is exempt from the finite classifiers (integer and floating mirror): any raw word is valid history",
    },
    "changed_paths": changed,
    "tracked_lines_deleted_by_file": deleted,
    "guards": guards,
    "runs": runs,
    "retained_failures_and_recoveries": {label: {"actual_exit": code, "why": why} for label, (code, why) in EXPECTED_EXIT.items()},
    "input_maps": {"files": maps, "compare_pre_post": compare, "pre_utc": PRE_UTC, "post_utc": POST_UTC, "post_utc_note": "recorded by `date -u` immediately before the POST capture; not reconstructed from a file time",
                   "no_source_write_between": "the final set t1-t16 ran between the maps; the packet files under reference/ are not inputs of any check and were written after the POST capture"},
    "scripts": script_rows,
    "cache": {"volume": "eech-linux-d3-collision-driver-5416adb3b-20261029", "image_x64": "sha256:4a5618cbc1e13e96caa4cb1ff0076c4c24dc883876d0bdec1398024f3a2d6c51", "image_i686": "eech-d3-tree-i686-20261106:2", "owners": "one (this lane); no container before or after", "cleanup": "none performed"},
    "not_run": ["Windows build", "native, campaign or war launch", "retail data", "the sector-height observer baseline (it observes the retired original word)", "signalling-NaN or engine -O2 measurement", "agents/MCP/authentication/model change", "cache or process deletion", "merge"],
    "limits": [
        "source and Linux checks only; every native gate (fresh source/profile, observer-off ordinary short against the accepted 80fd files, the declared window compared word for word, the controlled public witnesses) is Root's after the SAME combined source review",
        "the guarded fixed-wing clearance and the native server-session save and restore are UNREACHED: named blockers ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1 and ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1 stay OPEN",
        "the host does not know the world map: a sector it never saw answers the zero word and the campaign only asks for sectors it holds; the word is never validated",
        "no signalling-NaN, engine -O2 or exhaustive census claim; the host's raise expression is the original's text compared with the original response on x86-64 and i686 over 21 domain cases under both rounding modes",
        "no baseline, input, seed, frame, duration or tolerance changed; no clock, random draw or order changed",
    ],
    "state": {"sector-height-host-move-open-facts": "OPEN until the move is reviewed and its native authority, refusal and comparison evidence exist", "P1": "ENV-ENTITY-POSE-LONG-RUN-MOTION-1 CLOSED at f734 (4de6d2e9)",
              "claims": "no self-review, approval, native clearance, whole Task49 or D3, delivery", "D3-CURRENT.json": "not edited (Root owns it)"},
    "hygiene_problems_at_freeze": problems,
}
FREEZE.write_text(json.dumps(freeze, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
shutil.copyfile(FREEZE, PACK / FREEZE.name)
if not (PACK / ".gitattributes").exists():
    shutil.copyfile(REPO / "eech-campaign/reference/d3-entity-collision/sector-height-public-baseline-reviewed/.gitattributes", PACK / ".gitattributes")
files = [{"path": str(p.relative_to(PACK)).replace("\\", "/"), "bytes": p.stat().st_size, "sha256": sha(p)} for p in sorted(PACK.rglob("*")) if p.is_file() and p.name != "manifest.json"]
(PACK / "manifest.json").write_text(json.dumps({"files": files, "scope": "Source and Linux-check packet of the Host-owned sector height production move only; not a native result, review, approval or acceptance."}, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps({"problems": problems, "runs": len(runs), "final_runs": len(final_runs), "changed": len(changed), "files": len(files), "guards": guards}, indent=1))
sys.exit(1 if problems else 0)
