"""Build the literal publication packet sector-height-public-baseline-reviewed: copy the sealed files byte for byte, write .gitattributes and the manifest."""
import hashlib
import json
import pathlib
import shutil
import sys

repo = pathlib.Path(sys.argv[1])
ev = pathlib.Path("R:/d3-evidence")
pk = repo / "eech-campaign/reference/d3-entity-collision/sector-height-public-baseline-reviewed"
(pk / "receipts").mkdir(parents=True, exist_ok=True)
top = [
    "independent-sector-height-capacity-f01-native-20261107.json",
    "sector-height-capacity-f01-actuals-Root-binding-20261107.json",
    "sector-height-capacity-f01-Root-baseline-characterization-20261107.json",
    "sector-height-capacity-428-CI-readback-20261107.json",
    "Root-sector-height-production-start-disposition-20261107.json",
]
receipts = [
    "sector-height-capacity-f01-windows-20261107-completion.json",
    "sector-height-capacity-f01-profile-20261107-completion.json",
    "sector-height-capacity-f01-short-20261107-completion.json",
    "sector-height-capacity-f01-short-comparison-20261107.json",
    "sector-height-capacity-f01-baseline-20261107-completion.json",
    "sector-height-capacity-f01-plan-20261107.json",
]
for n in top:
    shutil.copyfile(ev / n, pk / n)
for n in receipts:
    shutil.copyfile(ev / n, pk / "receipts" / n)
(pk / ".gitattributes").write_bytes(b"* -text\n")
want = {"independent-sector-height-capacity-f01-native-20261107.json": "3665b264316b598b7d9892bb2bd66645a935c97bed68afdea0673141cbde498e",
        "sector-height-capacity-f01-actuals-Root-binding-20261107.json": "bac255fdc587c1f9a8c86dcbe98ba1ad22c98b54bdd175e7414dc10571766598",
        "sector-height-capacity-f01-Root-baseline-characterization-20261107.json": "1af90106db5bd68fc008315e32111aafc6386509eb6b3c48f54ff0dcf4b618f8"}
rows = []
problems = []
for p in sorted((q for q in pk.rglob("*") if q.is_file() and q.name != "manifest.json"), key=lambda q: q.relative_to(pk).as_posix().lower()):
    b = p.read_bytes()
    rel = p.relative_to(pk).as_posix()
    h = hashlib.sha256(b).hexdigest()
    if rel in want and want[rel] != h:
        problems.append(rel + " hash differs from the sealed value")
    rows.append({"path": rel, "bytes": len(b), "sha256": h})
(pk / "manifest.json").write_bytes((json.dumps({"files": rows}, indent=2) + "\n").encode("utf-8"))
print(json.dumps({"files": len(rows), "problems": problems}))
sys.exit(1 if problems else 0)
