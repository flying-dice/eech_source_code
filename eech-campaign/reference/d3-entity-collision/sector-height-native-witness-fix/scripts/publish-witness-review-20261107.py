"""Publishes the compact sibling packet of the SAME review 96b2f414 of the M5/M6 witness source 6e4139043: the literal review, Root's qualified binding v3 and the exact 579 CI read-back, plus a truthful chronology erratum
for the one audit reader that changed after POST. Usage: publish.py <repo-root>. The original witness packet and its frozen report are not touched."""
import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1])
EXT = pathlib.Path("R:/d3-evidence")
PACK_REL = "eech-campaign/reference/d3-entity-collision/sector-height-native-witness-source-reviewed"
PACK = REPO / PACK_REL
FILES = {
    "independent-sector-height-native-witness-source-6e4139-20261107.json": "96b2f4140e47ef17b0e672691e39173b3d920ea75282ec0cee60d56cdab8730f",
    "height-witness-Root-binding-579-v3-20261107.json": "4ef8c75045809c6baa2e7cc16330011407848112cb2130bf8da5897047a50203",
    "height-witness-579-all9-CI-readback-20261107.json": "f8d741fd5e8c46685a4c8133d8a0f40362ecd3aee9d01bfdb072b7783727aef8",
}
problems = []
PACK.mkdir(parents=True, exist_ok=True)
(PACK / ".gitattributes").write_bytes((REPO / "eech-campaign/reference/d3-entity-collision/sector-height-proof-guards-reviewed/.gitattributes").read_bytes())
for name, want in FILES.items():
    data = (EXT / name).read_bytes()
    if hashlib.sha256(data).hexdigest() != want:
        problems.append("sealed input differs: " + name)
    (PACK / name).write_bytes(data)
review = json.loads((EXT / "independent-sector-height-native-witness-source-6e4139-20261107.json").read_text(encoding="utf-8"))
decision = review["decision"]
ids = [f["id"] for f in decision["findings"]]
expected_ids = ["ENV-SECTOR-HEIGHT-M5-ROUNDING-ORACLE-1", "ENV-SECTOR-HEIGHT-M6-PACK-RETRY-1", "ENV-SECTOR-HEIGHT-M6-SAVE-PATH-BOUND-1", "ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1"]
if ids != expected_ids or decision["status"] != "CHANGES_REQUIRED" or decision["native_launch_clearance"] or decision["Windows_build_clearance"]:
    problems.append("the review's decision is not the four P2s with no clearance: %s" % ids)
binding = json.loads((EXT / "height-witness-Root-binding-579-v3-20261107.json").read_text(encoding="utf-8"))
ci = json.loads((EXT / "height-witness-579-all9-CI-readback-20261107.json").read_text(encoding="utf-8"))
if not (ci["all_nine_jobs_success"] and ci["all_three_workflows_success"] and ci["no_manual_rerun"] and ci["commit"].startswith("579427c55")):
    problems.append("the CI read-back is not all nine jobs of the exact 579 head")
drift = binding["post_POST_reader_drifts"]
if len(drift) != 1 or drift[0]["current_sha256"] != hashlib.sha256((EXT / "freeze-hw-witnesses-20261107.py").read_bytes()).hexdigest():
    problems.append("the drifted reader is not the current freeze-hw-witnesses-20261107.py")
erratum = {
    "kind": "chronology erratum of the audit reader that changed after POST in the M5/M6 witness freeze (6e4139043); the frozen report and the original packet are preserved unchanged",
    "frozen_statement_that_is_false_as_written": "D3-SECTOR-HEIGHT-NATIVE-WITNESSES-CODE-FROZEN-20261107.json audit_reader_timing: 'this freeze script, the join, the extra-identity script, the final-set runner, the runner and the check script were FINAL before the PRE identity capture; no reader changed after the POST capture'",
    "what_is_true": {
        "pre_utc": "2026-11-07T11:35:17Z", "post_utc": "2026-11-07T11:36:44Z",
        "reader": "R:/d3-evidence/freeze-hw-witnesses-20261107.py",
        "bytes_and_sha256_at_PRE_POST": [21298, "0f02f842ec40615da4134adad34633cdccc8c86fb90475ce140979812af6bdd3"],
        "bytes_and_sha256_current": [21620, "47e382c7abd3d09fe6c0587ceb95049b62ab74152d7c826c3190b8fd2bd34bae"],
        "last_write": "2026-11-07T11:38:19 (after POST)",
        "cause": "the author's own edit, made after POST and before the final freeze run: the freeze copied the first attempt's full per-file PRE input map (over 200 KB) into the packet; the edit keeps files over 200000 bytes external and records their size and SHA-256 instead. It changed what the PACKET copies, no check, source, map or compiled input",
        "other_extras": "the other 16 extra identity entries are stable between PRE and POST and equal to the current files",
        "current_reader_is_literal_in": "the frozen packet's scripts/ and the Git packet (Root's binding v3 verified)",
    },
    "not_inferred": ["compiler, test or source drift", "that all 17 extras are current literals (16 are)", "a need to replay any check for a publisher-only edit"],
    "qualification_of_provenance": "the original packet stays literal; this erratum, the review and Root's binding v3 qualify it. Root's earlier strict extra-reader assertion and the v2 all-runs-final assumption errors are retained by Root; v3 filters the actual final=true receipts",
}
(PACK / "erratum-audit-reader-chronology.json").write_text(json.dumps(erratum, indent=2) + "\n", encoding="utf-8", newline="\n")
subprocess.run(["git", "-C", str(REPO), "add", "-A", PACK_REL], check=True)
literal = {}
for name in FILES:
    blob = subprocess.run(["git", "-C", str(REPO), "cat-file", "blob", ":" + PACK_REL + "/" + name], capture_output=True, check=True).stdout
    source = (EXT / name).read_bytes()
    literal[name] = {"git_index_blob_bytes_equal_external_file": blob == source, "bytes": len(source), "sha256": hashlib.sha256(source).hexdigest()}
    if blob != source:
        problems.append("not literal in Git: " + name)
(PACK / "literal-bytes-check.json").write_text(json.dumps({"kind": "actual Git index blob bytes (git cat-file) against the external sealed files", "files": literal, "problems": problems}, indent=2) + "\n", encoding="utf-8", newline="\n")
subprocess.run(["git", "-C", str(REPO), "add", "-A", PACK_REL], check=True)
rows = [{"path": str(p.relative_to(PACK)).replace("\\", "/"), "bytes": p.stat().st_size, "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(PACK.rglob("*")) if p.is_file() and p.name != "manifest.json"]
(PACK / "manifest.json").write_text(json.dumps({"files": rows, "scope": "Literal copies of the SAME review of the M5/M6 witness source, Root's qualified binding v3 and the exact 579 CI read-back (it arrived after the review's seal) and a chronology erratum; not a native result or clearance."}, indent=2) + "\n", encoding="utf-8", newline="\n")
subprocess.run(["git", "-C", str(REPO), "add", "-A", PACK_REL], check=True)
print(json.dumps({"problems": problems, "files": len(rows)}))
sys.exit(1 if problems else 0)
