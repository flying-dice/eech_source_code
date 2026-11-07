"""Publishes the compact sibling packet of the SAME review 507f49b7 of the e42bb19a0 witness correction: the literal review and Root's binding 17f. Usage: publish.py <repo-root>.
The original witness packet, the first correction packet and the earlier reviewed packet are not touched."""
import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1])
EXT = pathlib.Path("R:/d3-evidence")
PACK_REL = "eech-campaign/reference/d3-entity-collision/sector-height-native-witness-fix-reviewed"
PACK = REPO / PACK_REL
FILES = {
    "independent-sector-height-native-witness-fix-e42bb-source-20261107.json": "507f49b7aff355115713c7ec325b6fe250601991be7866c8ca6eaadad53dbf99",
    "height-witness-fix-Root-binding-17f-20261107.json": "f2ad25d984e1ed419f8c92a151e49445020c4ca6c030d05269da572de3096a21",
}
problems = []
PACK.mkdir(parents=True, exist_ok=True)
(PACK / ".gitattributes").write_bytes((REPO / "eech-campaign/reference/d3-entity-collision/sector-height-proof-guards-reviewed/.gitattributes").read_bytes())
for name, want in FILES.items():
    data = (EXT / name).read_bytes()
    if hashlib.sha256(data).hexdigest() != want:
        problems.append("sealed input differs: " + name)
    (PACK / name).write_bytes(data)
review = json.loads((EXT / "independent-sector-height-native-witness-fix-e42bb-source-20261107.json").read_text(encoding="utf-8"))
decision = review["decision"]
if (
    [f["id"] for f in decision["findings"]] != ["ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1"]
    or decision["closed_source_ids"] != ["ENV-SECTOR-HEIGHT-M5-ROUNDING-ORACLE-1", "ENV-SECTOR-HEIGHT-M6-PACK-RETRY-1", "ENV-SECTOR-HEIGHT-M6-SAVE-PATH-BOUND-1"]
    or decision["native_launch_clearance"] or not decision["Windows_build_clearance_is_conditional"] or decision["current_diagnostic_Lua_accepted"]
):
    problems.append("the review's decision is not three CLOSED_SOURCE ids, one OPEN_RESIDUAL id, conditional build-only clearance and no native launch")
binding = json.loads((EXT / "height-witness-fix-Root-binding-17f-20261107.json").read_text(encoding="utf-8"))
if not (binding["head"].startswith("17f4b824b") and binding["clean"] and binding["all_3976_pre_post_current_raw_equal"] and binding["extra_files_current_literal_count"] == 22):
    problems.append("Root's binding is not the exact clean 17f head with 3976 equal rows and 22 current extras")
(PACK / "README.md").write_text(
    "# SAME review 507f49b7 of the four-P2 correction e42bb19a0 (literal records)\n\n"
    "The review closes `ENV-SECTOR-HEIGHT-M5-ROUNDING-ORACLE-1`, `ENV-SECTOR-HEIGHT-M6-PACK-RETRY-1` and `ENV-SECTOR-HEIGHT-M6-SAVE-PATH-BOUND-1` at source (declared scope) and keeps `ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1` OPEN_RESIDUAL: a supplied `control=` report's `min_height` / `answered_y` words are not validated before comparison, and the effective campaign / map setup of a reference is not bound.\n"
    "Windows_build_clearance is true and CONDITIONAL (exact 17f, read-only profile/object/export/payload sealing, no execution of the diagnostic); native_launch_clearance=false; the current diagnostic Lua is not accepted. Root's binding of the exact 17f head is kept beside it.\n\n"
    "The files are literal copies (`* -text`, bytes checked by `git cat-file` in `literal-bytes-check.json`). This is a record, not a native result, review of the follow-up, clearance or acceptance; M5, M6, height-open-facts and all ten requirements stay OPEN.\n",
    encoding="utf-8", newline="\n")
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
(PACK / "manifest.json").write_text(json.dumps({"files": rows, "scope": "Literal copies of the SAME review of the e42bb19a0 correction and Root's binding of the exact 17f head; not a native result or clearance."}, indent=2) + "\n", encoding="utf-8", newline="\n")
subprocess.run(["git", "-C", str(REPO), "add", "-A", PACK_REL], check=True)
print(json.dumps({"problems": problems, "files": len(rows)}))
sys.exit(1 if problems else 0)
