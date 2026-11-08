"""[tvr459 is the TACVIEW-DISPOSITION CORRECTION batch: derived from the tvb1 freeze script (itself from sdc5a/sb120); this docstring is inherited and describes those batches; the code below was edited for this one.] Freezes the SUPPLY-DESCRIPTOR batch packet (family tvr459): the third real immediate-consumer control of the per-entity UPDATED word, the shipping supply arrival descriptor against the engine's own Rust boundary, in the ORIGINAL and the
HOSTED profile; candidate, UNREVIEWED, native UNRUN) ONCE.
Usage: freeze.py readers-pre | freeze.py freeze <repo-root> <external-freeze-path> <post-utc> [dry]
This script and every reader it names are FINAL before the PRE identity capture: their hashes are recorded at PRE (`readers-pre`, tvr459-readers-PRE-20261107.json) and this script REFUSES to freeze if any of them differs now
(a deviation is an erratum, never hidden). Every run record is DERIVED from the files on disk and read AS A WHOLE: state terminal, the declared helper, a recorded exit, both logs present, and the actual `test result` lines parsed
(a Cargo exit alone is never trusted: a selected test binary must have run with a NONZERO count), and the control's own facts (descriptors delivered, host traffic) are read from its log. The changed files must be EXACTLY the expected set
(this batch changes a feature-gated C world, a feature-gated seam, the gate of two test-only helpers, two manifests and the lock, a guard and the contract). Root's supervision notes are read AT THE ACTUAL TIME: a file present that this
script was not told was read is a problem. Every file it creates is created exclusively."""
import datetime
import hashlib
import json
import pathlib
import re
import shutil
import subprocess
import sys

EXT = pathlib.Path("R:/d3-evidence")
READERS = ["run-physical-flags-tvr459-direct-20261107.py", "check-physical-flags-tvr459-linux-20261107.sh", "run-tvr459-final-set-20261107.sh", "tvr459-extra-identity-20261107.py", "freeze-tvr459-20261107.py", "publish-tvr459-20261107.py",
           "sector-roster-inputmap-batched-20261106.py", "sho-compare-maps-20261107.py", "preflight-receipts-20261107.py", "tvr459-name-check-20261107.py", "derive-tvr459-helpers-20261108.py", "tvr459-identity-join-20261107.py"]
READERS_PRE = EXT / "tvr459-readers-PRE-20261107.json"


def write_new(path, text):
    with open(path, "xb") as handle:
        handle.write(text.encode("utf-8"))


if sys.argv[1] == "readers-pre":
    write_new(READERS_PRE, json.dumps({r: hashlib.sha256((EXT / r).read_bytes()).hexdigest() for r in READERS}, indent=2) + "\n")
    print("readers recorded", len(READERS))
    sys.exit(0)
assert sys.argv[1] == "freeze"
REPO = pathlib.Path(sys.argv[2]).resolve()
FREEZE = pathlib.Path(sys.argv[3])
POST_UTC = sys.argv[4]
DRY = len(sys.argv) > 5 and sys.argv[5] == "dry"
PACK_REL = "eech-campaign/reference/d3-entity-collision/physical-flags-tacview-correction" + ("-DRY" if DRY else "")
PACK = REPO / PACK_REL
RUNS = PACK / "runs"
SCRIPTS = PACK / "scripts"
CHECKPOINT = "4596534a5"
PREFIX = "flags-tvr459-"
HELPER = "check-physical-flags-tvr459-linux-20261107.sh"
HELPER_TEXT = (EXT / HELPER).read_text(encoding="utf-8")
IMAGE = re.search(r"^image=(\S+)$", HELPER_TEXT, flags=re.M).group(1)
VOLUME = re.search(r"^volume=(\S+)$", HELPER_TEXT, flags=re.M).group(1)
ROOT_LEASED = {"image": "sha256:9d247b4ea15457f06579ef703ce287e96358128e7a2217f2833ee8b628fc85c3", "volume": "eech-linux-d3-physical-flags-9d247-20261107"}  # the names Root leased, asserted against the helper
PRE_UTC = (EXT / ("tvr459-dry-PRE-utc-20261107.txt" if DRY else "tvr459-PRE-utc-20261107.txt")).read_text().strip()
RETAINED = {label: (v["actual_exit"], v["why"]) for label, v in json.loads((EXT / "tvr459-retained-dev-runs-20261107.json").read_text(encoding="utf-8")).items()}
FINAL = ["s1-fmt-check", "s2-clippy-engine-sys", "s3-flags-guard"]
if DRY:
    FINAL = ["g1-green-brace-aware"]  # a dry freeze reads the real GREEN guard developer run as a stand-in for the final set (its output is the -DRY packet only)
# the selected targets that must have RUN with a nonzero count and no failure; the actual counts are parsed from the final receipts AFTER they complete and recorded, never pinned in advance (Root's note of 22:22)
MIN_TARGETS = {
    "s3-flags-guard": [("physical_flags_guard", 1)],
    "g1-green-brace-aware": [("physical_flags_guard", 1)],
}
# the files this batch changes in the batched input map, EXACTLY (the three inherited line-ending files are classified apart): the source guard only; NO production source changes
# (the contract is read through the extra identities and is committed beside them)
EXPECTED_CHANGED = sorted([
    "eech-campaign/crates/eech-engine-sys/tests/physical_flags_guard.rs",
])
FEEDBACK_READ = json.loads((EXT / "tvr459-feedback-files-20261107.json").read_text(encoding="utf-8"))["read"]
FEEDBACK_GLOBS = ("Root-physical-flags-witness-feedback-*.json", "Root-physical-flags-witness-fix-feedback*.json", "Root-physical-flags-witness-residual-feedback*.json", "Root-physical-flags-supply-consumer-feedback*.json", "Root-supply-boolean-fix-feedback*.json", "Root-tacview-consumer-feedback*.json", "Root-tacview-correction-feedback*.json")
# Root's supervision note of THIS batch, the SAME independent review it answers and the two small safe Root receipts it asks to be published literally (the mechanical binding of the reviewed source and the publication identity of 459653)
SUPPLEMENTAL = ["independent-tacview-disposition-8ffd-source-20261108.json", "tacview-disposition-8ffd-Root-binding-20261108.json", "tacview-disposition-459653-Root-publication-20261108.json"]
ROOT_LITERALS = sorted(p.name for p in EXT.glob("Root-tacview-correction-feedback*.json")) + SUPPLEMENTAL
problems = []


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def git(*args):
    result = subprocess.run(["git", "-C", str(REPO), *args], capture_output=True, check=True)
    return result.stdout.decode("utf-8", "replace")


def jload(path):
    return json.loads(pathlib.Path(path).read_text(encoding="utf-8"))


# ---------------------------------------------------------------------------------------------------------------- readers, lease names, supervision notes
pre_readers = jload(READERS_PRE)
for reader in READERS:
    if pre_readers.get(reader) != sha(EXT / reader):
        problems.append("reader differs from its PRE identity (erratum required): " + reader)
if IMAGE != ROOT_LEASED["image"] or VOLUME != ROOT_LEASED["volume"]:
    problems.append("the helper names another image or volume than the one Root leased")
present_feedback = sorted({p.name for pattern in FEEDBACK_GLOBS for p in EXT.glob(pattern)})
for name in present_feedback:
    if name not in FEEDBACK_READ:
        problems.append("a Root supervision note was present and not read: " + name)

# ---------------------------------------------------------------------------------------------------------------- the input maps, equal at PRE and POST; the changed files are exactly the expected set
compare = jload(EXT / ("tvr459-dry-inputmaps-compare-20261107.json" if DRY else "tvr459-inputmaps-compare-20261107.json"))
if not compare["all_rows_equal"] or not compare["extra_identity_equal"] or compare["rows_that_differ"] or compare["rows_only_in_pre"] or compare["rows_only_in_post"]:
    problems.append("the PRE and POST input maps or extra identities are not equal")
post_map = jload(EXT / ("tvr459-dry-inputmap-POST-20261107.json" if DRY else "tvr459-inputmap-POST-20261107.json"))
changed_all = post_map["changed_from_head"]
# a file whose RAW bytes differ from the head blob but whose Git identity (after Git's own clean filters) equals it is an inherited line-ending qualification, never a change of this batch
inherited = [p for p in changed_all if post_map["files"][p].get("head_blob") is not None and post_map["files"][p]["head_blob"] == post_map["files"][p]["git_blob_of_raw_bytes"]]
changed = sorted(p for p in changed_all if p not in inherited)
for path in inherited:
    if not path.startswith("eech-campaign/crates/eech-world/tests/lua/tree-geometry-"):
        problems.append("an inherited raw-versus-Git difference other than the three known uncompiled Lua files: " + path)
inherited_rows = [{"path": p, "raw_sha256": post_map["files"][p]["raw_sha256"], "git_blob_of_raw_bytes": post_map["files"][p]["git_blob_of_raw_bytes"], "head_blob": post_map["files"][p]["head_blob"]} for p in inherited]
if post_map["deleted_from_head"]:
    problems.append("files deleted from head: %r" % post_map["deleted_from_head"])
if not DRY and changed != EXPECTED_CHANGED:
    problems.append("the changed files are not exactly the expected set: unexpected %r, missing %r" % (sorted(set(changed) - set(EXPECTED_CHANGED)), sorted(set(EXPECTED_CHANGED) - set(changed))))


# ---------------------------------------------------------------------------------------------------------------- the runs, read as a whole
def read_run(label, role):
    stem = PREFIX + label
    record_path = EXT / (stem + ".json")
    out_path, err_path = EXT / (stem + ".stdout.log"), EXT / (stem + ".stderr.log")
    if not (record_path.exists() and out_path.exists() and err_path.exists()):
        problems.append("%s: record or logs missing (%s)" % (role, label))
        return None
    record = jload(record_path)
    row = {"label": label, "state": record.get("state"), "actual_exit": record.get("actual_exit"), "started_utc": record.get("started_utc"), "completed_utc": record.get("completed_utc"), "command": record.get("command"),
           "record_sha256": sha(record_path), "stdout_sha256": sha(out_path), "stderr_sha256": sha(err_path), "uses_declared_helper": any(HELPER in a for a in record.get("args", []))}
    if record.get("state") != "terminal":
        problems.append("%s %s: not terminal" % (role, label))
    if not row["uses_declared_helper"]:
        problems.append("%s %s: not run with the declared helper" % (role, label))
    # cargo prints `Running <target>` (and `Doc-tests <crate>`) to STDERR and each target's `test result` line to STDOUT, one per target in the same order: the two lists are zipped, and a different number of each is a problem
    text = out_path.read_text(encoding="utf-8", errors="replace")
    names = []
    for line in err_path.read_text(encoding="utf-8", errors="replace").splitlines():
        m = re.match(r"\s*(?:Running|Doc-tests)\s+(.+?)(?: \(|$)", line)
        if m:
            name = m.group(1)
            name = name[len("tests/"):] if name.startswith("tests/") else name
            names.append(name[:-3] if name.endswith(".rs") else name)
    results = []
    for line in text.splitlines():
        m = re.match(r"test result: (\w+)\. (\d+) passed; (\d+) failed", line)
        if m:
            results.append({"status": m.group(1), "passed": int(m.group(2)), "failed": int(m.group(3))})
    counts = {}
    if names and len(names) != len(results) and record.get("actual_exit") == 0:
        problems.append("%s %s: %d cargo targets on stderr and %d test result lines on stdout" % (role, label, len(names), len(results)))
    elif names:
        for name, result in zip(names, results):
            counts[name] = result
    row["test_binaries"] = len(counts)
    row["tests_passed"] = sum(c["passed"] for c in counts.values())
    row["tests_failed"] = sum(c["failed"] for c in counts.values())
    row["selected_counts"] = {}
    for name, minimum in MIN_TARGETS.get(label, []):
        got = counts.get(name)
        row["selected_counts"][name] = got["passed"] if got else None
        if got is None or got["passed"] < minimum or got["failed"]:
            problems.append("%s %s: %s ran %s tests, at least %d must pass" % (role, label, name, got, minimum))
    # the control's own facts from its log: the descriptors the real Rust boundary delivered, and in the hosted profile the host's traffic by kind
    delivered = re.findall(r"supply descriptors delivered to the host's answer: (\d+)", text)
    if delivered:
        row["descriptors_delivered"] = int(delivered[-1])
        if row["descriptors_delivered"] < 1:
            problems.append("%s %s: no descriptor reached the host's answer: the consumer was not reached" % (role, label))
    traffic = re.findall(r"host traffic \(groups, sets, reads, resets, fresh bindings\): \((\d+), (\d+), (\d+), (\d+), (\d+)\)", text)
    if traffic:
        row["hosted_traffic_groups_sets_reads_resets_fresh"] = [int(x) for x in traffic[-1]]
        if min(row["hosted_traffic_groups_sets_reads_resets_fresh"]) < 1:
            problems.append("%s %s: a kind of host traffic is zero: %r" % (role, label, traffic[-1]))
    made = re.findall(r"expectations made: (\d+)", text)
    if made:
        row["expectations_made"] = [int(x) for x in made]
    return row


final_rows = {}
for label in FINAL:
    row = read_run(label, "final")
    if row is not None:
        final_rows[label] = row
        if row["actual_exit"] != 0:
            problems.append("final %s exit %r" % (label, row["actual_exit"]))
        if "clippy" in label and "clippy" not in (row["command"] or ""):
            problems.append("final %s did not run clippy" % label)
progress = (EXT / "tvr459-final-set-20261107.progress").read_text(encoding="utf-8").split() if not DRY else []
if not DRY:
    expected = [x for label in FINAL for x in (label, "exit=0")] + ["DONE"]
    if progress != expected:
        problems.append("the final-set progress file is not exactly the labels with exit=0 and DONE: %r" % progress)
retained_rows = []
for label, (exit_code, why) in sorted(RETAINED.items()):
    row = read_run(label, "retained")
    if row is not None:
        if row["actual_exit"] != exit_code:
            problems.append("retained %s exit %r is not the recorded %r" % (label, row["actual_exit"], exit_code))
        row["why"] = why
        retained_rows.append(row)

# ---------------------------------------------------------------------------------------------------------------- the changed files against actual Git bytes
code_rows = []
for path in changed:
    data = (REPO / path).read_bytes()
    code_rows.append({"path": path, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(), "git_blob_of_raw_bytes": git("hash-object", path).strip(), "head_blob": post_map["files"][path].get("head_blob")})

# ---------------------------------------------------------------------------------------------------------------- write the packet (every file created exclusively; short basenames: the Windows path length is real)
for d in (PACK, RUNS, SCRIPTS):
    d.mkdir(parents=True, exist_ok=True)
if not DRY:
    assert not FREEZE.exists(), FREEZE
short_names = {}
unique_rows = {row["label"]: row for row in list(final_rows.values()) + retained_rows}
for row in unique_rows.values():
    for suffix in ("json", "stdout.log", "stderr.log"):
        source = EXT / ("%s%s.%s" % (PREFIX, row["label"], suffix))
        target = RUNS / ("%s.%s" % (row["label"], suffix))
        short_names[source.name] = "runs/" + target.name
        with open(target, "xb") as handle:
            handle.write(source.read_bytes())
for reader in READERS:
    with open(SCRIPTS / reader, "xb") as handle:
        handle.write((EXT / reader).read_bytes())
for name in ("tvr459-readers-PRE-20261107.json", "tvr459-PRE-utc-20261107.txt", "tvr459-final-set-20261107.progress", "tvr459-final-set-lease-before-20261107.txt", "tvr459-final-set-lease-after-20261107.txt",
             "tvr459-inputmaps-compare-20261107.json", "tvr459-extra-identity-PRE-20261107.json", "tvr459-extra-identity-POST-20261107.json", "tvr459-retained-dev-runs-20261107.json", "tvr459-feedback-files-20261107.json",
             "tvr459-name-check-result-20261107.json", "tvr459-name-check-negative-control-20261107.json", "tvr459-preflight-receipts-20261107.json", "tvr459-preflight-negative-control-20261107.json", "tvr459-identity-join-20261107.json"):
    source = EXT / name
    if source.exists():
        with open(PACK / name, "xb") as handle:
            handle.write(source.read_bytes())
    elif not DRY:
        problems.append("expected evidence file missing: " + name)
for name in ROOT_LITERALS:
    with open(PACK / name, "xb") as handle:
        handle.write((EXT / name).read_bytes())
with open(PACK / ".gitattributes", "xb") as handle:
    handle.write((REPO / "eech-campaign/reference/d3-entity-collision/physical-flags-keyfix-020-reviewed/.gitattributes").read_bytes())
receipt = {
    "kind": "FROZEN receipt of the CORRECTION of the Tacview/MOVED-ROTATED disposition (both findings of the SAME review 9ab3482d: ENV-TACVIEW-DORMANCY-GUARD-ENCLOSURE-1 and ENV-TACVIEW-DISPOSITION-IDENTITY-SCOPE-1): the guard's real block-membership predicate, RED on the old predicate then GREEN, the contract's identity erratum and comms-route scope, and the identity join; one source-guard test and the contract; NO production source change; UNREVIEWED; native, Windows and default recordings UNRUN)",
    "frozen_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "dry": DRY, "source_before": git("rev-parse", "HEAD").strip(), "checkpoint": CHECKPOINT,
    "pre_utc": PRE_UTC, "post_utc": POST_UTC, "lease": {"volume": VOLUME, "image": IMAGE, "root_leased_names_asserted": ROOT_LEASED},
    "changed_files": code_rows, "expected_changed_files": EXPECTED_CHANGED, "inherited_raw_versus_git_qualifications": inherited_rows, "final_runs": final_rows, "retained_development_runs": retained_rows,
    "run_file_short_names": short_names, "inputmaps_compare": compare, "feedback_files_present_at_freeze": present_feedback, "readers_PRE_identity_equal": not any("reader differs" in p for p in problems),
    "hygiene_problems_at_freeze": problems,
    "not_run": ["any native, Windows, i686 or O2 run", "any build of product source (none changed)", "any replay of the accepted default, the finite witness, the supply correction or the first Tacview packet's checks", "the other consumers of the flag store", "the shipped Engine::boot bridge", "any Root-owned gate"],
}
FREEZE_TEXT = json.dumps(receipt, indent=2) + "\n"
if not DRY:
    write_new(FREEZE, FREEZE_TEXT)
write_new(PACK / "D3-TACVIEW-DISPOSITION-CORRECTION-FROZEN-20261107.json", FREEZE_TEXT)
print(json.dumps({"problems": problems, "final": {k: [v["actual_exit"], v["selected_counts"], v.get("descriptors_delivered"), v.get("hosted_traffic_groups_sets_reads_resets_fresh")] for k, v in final_rows.items()}, "retained": len(retained_rows), "changed": len(code_rows)}, indent=1))
sys.exit(1 if problems else 0)
