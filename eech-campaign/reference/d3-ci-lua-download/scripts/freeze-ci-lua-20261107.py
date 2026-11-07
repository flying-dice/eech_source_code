"""Freezes the CI Lua pinned-download repair packet ONCE. Usage: freeze.py <repo-root> <external-freeze-path> <post-utc>
This script and every reader it names (final-set runner, mutation/real-curl/workflow checks, join, extra-identity script) are FINAL before the PRE identity capture."""
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
PACK_REL = "eech-campaign/reference/d3-ci-lua-download"
PACK = REPO / PACK_REL
RUNS = PACK / "runs"
CHECKPOINT = "04bd7e71b"
NATIVE_1B89 = "1b89c0b0ab613d2f6a08ea02dd6612457efa8982"
PIN = "2640fc56a795f29d28ef15e13c34a47e223960b0240e8cb0a82d9b0738695333"
SEALED = {
    "sector-height-publication-1b89-CI-initial-download-failure-20261107.json": "816bcd919cab4f57b3df11c941135a02937bbd2470e3e86ce0d78d84204abba7",
    "sector-height-publication-1b89-CI-retry-download-failure-20261107.json": "52b2049c32d7129861768cca75bfe4650b4b1ec90ee209ba7d63d20ea5bb197b",
    "lua-5.1.5-MacPorts-verification-20261107.json": "be4adb3d7d4c8b051a605a130f87e98f3ddc99de300d8be48c8754f4e537cedf",
}
COPIED = ["lua-5.1.5-MacPorts-verification-20261107.json"]  # the only sealed input copied literally into the packet
REFERENCED = {  # read in full, not recopied: the SAME review already sits literally in the sector-height-proof-guards-reviewed packet
    "independent-sector-height-proofguards-e326-source-20261107.json": "cbbd27d713adc2d7505ac4931e7fa91c5a22bb46fe43ebdf888d6ac2d319dd94",
}
ARCHIVE = EXT / "lua-5.1.5-MacPorts-verified-20261107.tar.gz"
CHILDREN = ["c0-archive-identity", "c1-syntax-bash-posix", "c2-selftest-generated-payload", "c3-selftest-actual-pinned-archive", "c4-selftest-actual-pinned-archive-bash-posix",
            "c5-mutation-sha-comparison-disabled-is-caught", "c6-workflow-static-vs-1b89", "c7-real-curl-options-closed-loopback", "c8-git-diff-check"]
PRE_UTC = (EXT / "ci-lua-PRE-utc-20261107.txt").read_text().strip()
SOURCES = [".github/workflows/eech-campaign.yml", "eech-campaign/tools/fetch-pinned-archive.sh", "eech-campaign/tools/fetch-pinned-archive-selftest.sh", "docs/governance/D3-FIVE-WHYS-20261102.md"]


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
for name, want in {**SEALED, **REFERENCED}.items():
    if sha(EXT / name) != want:
        problems.append("sealed input differs: " + name)
if sha(ARCHIVE) != PIN:
    problems.append("the pinned archive file does not have the pinned SHA-256")
feedback_files = sorted(p.name for p in EXT.glob("Root-*feedback*"))

# the children, from the real progress file and files
set_dir = EXT / "ci3-final-set-20261107"
progress = (set_dir / "progress").read_text()
exits = dict(re.findall(r"^(\S+) exit=(\d+)$", progress, flags=re.M))
if "DONE" not in progress.split() or sorted(exits) != sorted(CHILDREN) or any(v != "0" for v in exits.values()):
    problems.append("the final set is not nine children all exit 0 followed by DONE: %s" % exits)
RUNS.mkdir(parents=True, exist_ok=True)
shutil.copyfile(REPO / "eech-campaign/reference/d3-entity-collision/sector-height-proof-guards-reviewed/.gitattributes", PACK / ".gitattributes")  # `* -text` BEFORE the first git add
children = []
for label in CHILDREN:
    for suffix in ("stdout", "stderr"):
        shutil.copyfile(set_dir / ("%s.%s" % (label, suffix)), RUNS / ("%s.%s" % (label, suffix)))
    out = (RUNS / (label + ".stdout")).read_text(encoding="utf-8", errors="replace")
    err = (RUNS / (label + ".stderr")).read_text(encoding="utf-8", errors="replace")
    children.append({"label": label, "actual_exit": int(exits.get(label, -1)), "stdout_bytes": len((RUNS / (label + ".stdout")).read_bytes()), "stdout_sha256": sha(RUNS / (label + ".stdout")),
                     "stderr_bytes": len((RUNS / (label + ".stderr")).read_bytes()), "stderr_sha256": sha(RUNS / (label + ".stderr"))})
shutil.copyfile(set_dir / "progress", RUNS / "progress")
selftest = {}
for label in CHILDREN[2:5]:
    m = re.search(r"all (\d+) checks passed \(pin ([0-9a-f]{64})\)", (RUNS / (label + ".stdout")).read_text(encoding="utf-8", errors="replace"))
    selftest[label] = {"checks_passed": int(m.group(1)), "pin": m.group(2)} if m else None
    if not m:
        problems.append("no 'all N checks passed' line in " + label)
for label in CHILDREN[3:5]:
    if selftest[label] and selftest[label]["pin"] != PIN:
        problems.append("the actual-archive selftest ran with another pin: " + label)
if selftest[CHILDREN[2]] and selftest[CHILDREN[2]]["pin"] == PIN:
    problems.append("the generated-payload run unexpectedly used the real pin")
if len({v["checks_passed"] for v in selftest.values() if v}) != 1:
    problems.append("the three selftest runs did not run the same matrix: %s" % selftest)
mutation = (RUNS / (CHILDREN[5] + ".stdout")).read_text(encoding="utf-8", errors="replace").splitlines()[0]
curl_run = (RUNS / (CHILDREN[7] + ".stdout")).read_text(encoding="utf-8", errors="replace")

ast_note = "no YAML parser or actionlint is installed locally: the workflow is checked structurally against the exact 1b89 text (c6); the real parse is the CI rerun, controlled by Root"

compare_out = EXT / "ci-lua-inputmaps-compare-20261107.json"
join_out = EXT / "ci-lua-join-vs-e326-and-1b89-20261107.json"
for p in (compare_out, join_out):
    if p.exists():
        p.unlink()
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "sho-compare-maps-20261107.py"), str(EXT / "ci-lua-inputmap-PRE-20261107.json"), str(EXT / "ci-lua-inputmap-POST-20261107.json"),
    str(EXT / "ci-lua-extra-identity-PRE-20261107.json"), str(EXT / "ci-lua-extra-identity-POST-20261107.json"), str(compare_out))
compare = json.loads(compare_out.read_text(encoding="utf-8"))
if not compare["all_rows_equal"] or not compare["extra_identity_equal"]:
    problems.append("PRE and POST maps differ")
run(PY, "-I", "-B", "-X", "utf8", str(EXT / "ci3-join-20261107.py"), str(REPO), str(EXT / "sector-height-proofguards-inputmap-POST-20261107.json"), str(EXT / "ci-lua-inputmap-POST-20261107.json"), str(join_out))
join = json.loads(join_out.read_text(encoding="utf-8"))
if join["problems"]:
    problems.append("the join has problems: %s" % join["problems"])
shutil.copyfile(compare_out, PACK / "inputmaps-compare.json")
shutil.copyfile(join_out, PACK / "join-vs-e326-and-1b89.json")
maps = {}
for name in ("ci-lua-inputmap-PRE-20261107.json", "ci-lua-inputmap-POST-20261107.json", "ci-lua-extra-identity-PRE-20261107.json", "ci-lua-extra-identity-POST-20261107.json"):
    maps[name] = {"bytes": (EXT / name).stat().st_size, "sha256": sha(EXT / name), "location": "external (full per-file rows)"}

# the stored bytes are what Git actually holds: stage the changed sources and compare every literal claim against the index blob bytes
subprocess.run(["git", "-C", str(REPO), "add", "--"] + SOURCES, check=True)
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
source_literal = {}
for path in SOURCES:
    raw = (REPO / path).read_bytes()
    blob = git_bytes("cat-file", "blob", ":" + path)
    source_literal[path] = {"working_bytes_equal_git_index_blob_bytes": raw == blob, "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw), "crlf_lines": raw.count(b"\r\n")}
    if raw != blob or raw.count(b"\r\n"):
        problems.append("the changed source is not literally stored as LF bytes: " + path)

dual_dir = "eech-campaign/reference/d3-entity-collision/sector-height-proof-guards-reviewed"
dual_unchanged = subprocess.run(["git", "-C", str(REPO), "diff", "--quiet", "HEAD", "--", dual_dir]).returncode == 0
if not dual_unchanged:
    problems.append("the old dual-identity packet changed")
dual_receipt = {"path": dual_dir + "/audit-output-dual-identity.json", "sha256": sha(REPO / dual_dir / "audit-output-dual-identity.json"), "unchanged_against_HEAD": dual_unchanged,
                "note": "the old normalized-stdout dual receipt is preserved literally, with its own scope"}

guards = {}
for key, args in (("generic", []), ("production-cutover", ["--action", "production-cutover"]), ("fix-blocker ENV-CI-LUA-ARCHIVE-AVAILABILITY-1", ["--action", "fix-blocker", "--blocker", "ENV-CI-LUA-ARCHIVE-AVAILABILITY-1"]),
                  ("fix-blocker sector-height-host-move-open-facts", ["--action", "fix-blocker", "--blocker", "sector-height-host-move-open-facts"]), ("deliver (expected unmet)", ["--action", "deliver"])):
    guards[key] = run(PY, "-B", "-X", "utf8", "eech-campaign/tools/migration-progress.py", *args).returncode

# only the small Root receipt is copied literally; the two failure records embed whole raw job logs (runner details, git fetch output), so the packet carries identities and a safe summary of them instead
for name in COPIED:
    shutil.copyfile(EXT / name, PACK / name)
summary_run = run(PY, "-I", "-B", "-X", "utf8", str(EXT / "ci3-safe-summary-20261107.py"), str(PACK / "ci-failures-safe-summary.json"))
if summary_run.returncode != 0:
    problems.append("the safe summary of the failure records failed: " + summary_run.stdout[-300:])
scripts = PACK / "scripts"
scripts.mkdir(exist_ok=True)
script_rows = []
for name in ("run-ci3-final-set-20261107.sh", "ci3-mutation-20261107.sh", "ci3-real-curl-options-20261107.sh", "ci3-workflow-static-20261107.py", "ci3-join-20261107.py", "ci3-extra-identity-20261107.py",
             "freeze-ci-lua-20261107.py", "patch-card-register-ci-lua-finding-20261107.py", "patch-card-status-ci-lua-20261107.py", "sector-roster-inputmap-batched-20261106.py", "sho-compare-maps-20261107.py",
             "verify-lua-5.1.5-fallback-archive-20261107.py", "ci3-safe-summary-20261107.py"):
    shutil.copyfile(EXT / name, scripts / name)
    script_rows.append({"name": name, "sha256": sha(scripts / name)})
subprocess.run(["git", "-C", str(REPO), "add", "-A", PACK_REL], check=True)
literal_check = {}
for name in COPIED:
    blob = git_bytes("cat-file", "blob", ":" + PACK_REL + "/" + name)
    source = (EXT / name).read_bytes()
    literal_check[name] = {"git_index_blob_bytes_equal_external_file": blob == source, "sha256": hashlib.sha256(source).hexdigest()}
    if blob != source:
        problems.append("the packet copy of %s is not the literal file in Git" % name)
(PACK / "literal-bytes-check.json").write_text(json.dumps({"kind": "actual Git index blob bytes (git cat-file) against the external sealed files and against the changed sources", "sealed_copies": literal_check, "changed_sources": source_literal,
                                                          "raw_versus_git_disagreements_of_the_input_map": literal, "lua_crlf_raw_vs_git_exceptions": crlf_exceptions, "core_filter_only_qualifications": filter_only,
                                                          "old_dual_identity_receipt": dual_receipt}, indent=2) + "\n", encoding="utf-8", newline="\n")

freeze = {
    "kind": "D3 CI Lua pinned-download repair CODE_FROZEN (workflow + one shell helper + its selftest; UNREVIEWED; no compiled input changed; native UNRUN here)",
    "frozen_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "task": "EEDCS-49",
    "stage": "necessary CI reliability repair only (not a preparation batch; the count stays 2); finding ENV-CI-LUA-ARCHIVE-AVAILABILITY-1 registered OPEN at the checkpoint, blocks_cutover=false",
    "source_before": head,
    "native_source_unchanged": NATIVE_1B89,
    "finding": "ENV-CI-LUA-ARCHIVE-AVAILABILITY-1 stays OPEN until the SAME independent review AND an actual green CI of the corrected workflow (Root controls the rerun)",
    "defect": {"run": 42359965325, "initial_job": 128366083321, "retry_job": 128369471075, "download_exit": 28, "seconds_to_failure": ["135.185", "136.790"], "origin": "www.lua.org:443", "stage": "before compile, test or any extract",
               "pinned_sha256": PIN, "fallback": "https://distfiles.macports.org/lua/lua-5.1.5.tar.gz (same 221213 bytes, exact hash verified by Root over HTTPS)"},
    "sealed_inputs": {name: {"sha256": sha(EXT / name)} for name in {**SEALED, **REFERENCED}},
    "archive": {"path": str(ARCHIVE), "sha256": sha(ARCHIVE), "bytes": ARCHIVE.stat().st_size, "note": "an external verified file used only for the focused checks; not copied, vendored or uploaded"},
    "root_feedback_files_present": feedback_files,
    "change": {
        "helper": "eech-campaign/tools/fetch-pinned-archive.sh <dest> <sha256> <https-url>...: tries the URLs in order (--connect-timeout 15, --max-time 120, --retry 0, https only including redirects, no certificate bypass), moves only bytes with the pinned SHA-256 to <dest> (mktemp file in the destination directory, removed on every exit), reuses a valid existing dest, never uses an invalid one; exit 0 / 1 (no source) / 2 (usage)",
        "workflow": "windows-cross 'Lua 5.1.5 as lua.dll': the curl and check lines replaced by the pinned variable, the helper with the lua.org URL first then the MacPorts URL, and the unchanged sha256sum check; one added no-network selftest step in evidence-selftest; version, SHA-256, compiler flags, link flags, the matrix and every other job unchanged",
        "not_changed": ["tools/Dockerfile (its native builders already built all three profiles with exit 0)", "Cargo files", "any C/Rust/Lua/model/physics source", "no cache, vendoring, upload, checksum, version or dependency substitution"],
    },
    "children": children,
    "selftest_matrix": selftest,
    "mutation_control": mutation,
    "real_curl_loopback_control": curl_run.strip().splitlines(),
    "workflow_structure_note": ast_note,
    "guards": guards,
    "input_maps": {"files": maps, "compare_pre_post": compare, "pre_utc": PRE_UTC, "post_utc": POST_UTC, "post_utc_note": "recorded by `date -u` immediately before the POST capture; not reconstructed from a file time",
                   "join_vs_e326_and_1b89": join},
    "literal_bytes": {"changed_sources": source_literal, "lua_crlf_raw_vs_git_exceptions": crlf_exceptions, "core_filter_only_qualifications": filter_only, "old_dual_identity_receipt": dual_receipt},
    "audit_reader_timing": "this freeze script, the join, the extra-identity script, the final-set runner and every check it names were FINAL before the PRE identity capture; no reader changed after the POST capture",
    "reused_unchanged": "the 0be 16/49 matrix, the e326 affected runs and Root's native 1b89 evidence (ordinary and observer shorts, the hosted M4 scan) are reused only through the explicit join: no compiled, tested or linked path changed against 1b89 and the map rows are identical to the e326 final POST map",
    "packet_policy": "the two failure records (816bcd91..., 52b2049c...) embed the whole raw job logs; they stay external and the packet carries ci-failures-safe-summary.json (identities, jobs, the failed job's steps and the three log lines of the failing step) and Root's small MacPorts receipt literally; no archive, raw log, credential or retail data",
    "superseded_first_capture": "an earlier PRE (2026-11-07T10:13:39Z) / checks / POST (10:14:27Z) over the SAME source and map (aggregate 3307008445a7...) was discarded as a packet, not as evidence: its freeze reader copied the two raw-log-bearing failure records into the packet. The reader (this script, plus the added ci3-safe-summary script and its identity entry) was corrected and the whole PRE/checks/POST sequence repeated so that every reader is final before the PRE capture; the first outputs are retained externally with the 'attempt1' name, the source did not change in between",
    "scripts": script_rows,
    "not_run": ["any C, Cargo, workspace, x87, Docker or Windows build", "native, campaign or war launch", "the ordinary or observer short, or the hosted scan", "the actual CI (Root controls the rerun)", "old experiments for CI or prose", "agents/MCP/authentication/model change", "cache or process deletion", "merge"],
    "limits": [
        "the controls use a fake transport and the actual verified archive; they show the helper's decisions, not that lua.org or MacPorts is reachable from the runner, and not that the runner's curl and sha256sum behave as the local ones (the real curl option set was exercised only against a refused loopback port, c7)",
        "no YAML parser or actionlint exists locally; the workflow is checked structurally against 1b89 (c6) and its real parse and the windows-cross job result are the CI rerun, controlled by Root",
        "tools/Dockerfile still downloads from lua.org alone (the native builders are Root's and finished 0; not changed here)",
        "the finding stays OPEN until the SAME independent review and an actual green CI; all ten D3 requirements, the M5 and M6 witnesses and sector-height-host-move-open-facts stay required; no self-review, approval, merge or Task49/D3/native authority claim; D3-CURRENT.json was not edited",
    ],
    "hygiene_problems_at_freeze": problems,
}
FREEZE.write_text(json.dumps(freeze, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
shutil.copyfile(FREEZE, PACK / FREEZE.name)
files = [{"path": str(p.relative_to(PACK)).replace("\\", "/"), "bytes": p.stat().st_size, "sha256": sha(p)} for p in sorted(PACK.rglob("*")) if p.is_file() and p.name != "manifest.json"]
(PACK / "manifest.json").write_text(json.dumps({"files": files, "scope": "Source and local-check packet of the CI Lua pinned-download repair only; not a native result, CI result, review, approval or acceptance."}, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps({"problems": problems, "children": len(children), "selftest": selftest, "mutation": mutation, "files": len(files), "guards": guards, "crlf_exceptions": crlf_exceptions, "filter_only": filter_only}, indent=1))
sys.exit(1 if problems else 0)
