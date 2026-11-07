"""Publishes the compact sibling packet of the SAME combined review (8edddd93) of the CI repair and the finite native 1b89 gates, with the literal Root files and a literal-Git-bytes check.
Usage: publish.py <repo-root>. Nothing is normalised; the packet copies only small safe records (no raw job log, authority id, coordinate or binary)."""
import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1])
EXT = pathlib.Path("R:/d3-evidence")
PACK_REL = "eech-campaign/reference/d3-entity-collision/sector-height-finite-native-ci-reviewed"
PACK = REPO / PACK_REL
FILES = {
    "independent-sector-height-finite-native-1b89-ci-lua-635-20261107.json": "8edddd934b091ea58567d43e3a71f4eaab378431de7b0b378990ae2703e77c32",
    "Root-feedback-height-native-CI-20261107.json": "452b10c8a4d129bc647cab737c01d637e8b7ff9b7e7dfefdff1aa5b7de12670a",
    "ci-lua-635-all9-CI-readback-20261107.json": "8c241395cb9e9dd99770cf6f6c551b52ec8dfe30eddc3982f45a47d14c0f13da",
    "sector-height-native-Root-binding-1b89-20261107.json": "fa02f29a90c38a1cbb08d78b7b5dcf8d753916f51f3ba392a41a3bfec6d15698",
}
problems = []
PACK.mkdir(parents=True, exist_ok=True)
(PACK / ".gitattributes").write_bytes((REPO / "eech-campaign/reference/d3-entity-collision/sector-height-proof-guards-reviewed/.gitattributes").read_bytes())
for name, want in FILES.items():
    data = (EXT / name).read_bytes()
    if hashlib.sha256(data).hexdigest() != want:
        problems.append("sealed input differs: " + name)
    (PACK / name).write_bytes(data)
review = json.loads((EXT / "independent-sector-height-finite-native-1b89-ci-lua-635-20261107.json").read_text(encoding="utf-8"))
decision = review["decision"]
for key, expected in (("findings", []), ("closed_ids", ["ENV-CI-LUA-ARCHIVE-AVAILABILITY-1"]), ("M5_guarded_fixed_wing_accepted", False), ("M6_native_Core_save_load_accepted", False),
                      ("whole_sector_height_extraction_accepted", False), ("whole_Task49_accepted", False), ("whole_D3_accepted", False), ("new_unreviewed_source_native_clearance", False)):
    if decision.get(key) != expected:
        problems.append("review decision field %s is %r, expected %r" % (key, decision.get(key), expected))
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
(PACK / "manifest.json").write_text(json.dumps({"files": rows, "scope": "Literal copies of the SAME combined review, Root's feedback, the exact 635 CI read-back and the finite native 1b89 binding; not a current native result, acceptance or clearance of any later work."}, indent=2) + "\n", encoding="utf-8", newline="\n")
subprocess.run(["git", "-C", str(REPO), "add", "-A", PACK_REL], check=True)
print(json.dumps({"problems": problems, "files": len(rows)}))
sys.exit(1 if problems else 0)
