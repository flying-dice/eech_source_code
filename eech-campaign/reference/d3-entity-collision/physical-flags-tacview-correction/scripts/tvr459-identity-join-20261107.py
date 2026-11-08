"""Identity join of the Tacview disposition correction (family tvr459), run BEFORE the real PRE and sealed by it: what differs between the 3abf03e51 source (the source of the retained compiled evidence) and the current working tree
over the ranges the earlier prose claimed equal, which sources the disposition actually relies on are byte-identical to 3abf's, and that the retained extraction and reports are still the bytes it recorded. It repairs nothing and
rewrites nothing: the old extraction is read, never changed. Output created EXCLUSIVELY. Usage: join.py <repo-root> <out.json>"""
import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1])
OUT = pathlib.Path(sys.argv[2])
assert not OUT.exists(), OUT
PACKET = REPO / "eech-campaign/reference/d3-entity-collision/physical-flags-tacview-disposition"
PROFILE = pathlib.Path("D:/EECH-Migration-Evidence/supply-boolean-native-3abf-profile-20261107")
BASE = "3abf03e51"
GUARD = "eech-campaign/crates/eech-engine-sys/tests/physical_flags_guard.rs"
RELIED_ON = [
    "aphavoc/source/entity/tacview/tacview.c", "aphavoc/source/entity/tacview/tacview.h", "aphavoc/source/entity/mobile/aircraft/helicop/helicop.c", "aphavoc/source/flight.c", "aphavoc/source/cmndline.c",
    "aphavoc/source/gameflow/gameflow.c", "aphavoc/source/comms/comm_man.c", "aphavoc/source/entity/special/update/up_update.c", "aphavoc/source/entity/mobile/aircraft/fixwing/fw_updt.c",
    "aphavoc/source/entity/mobile/aircraft/helicop/hc_updt.c", "aphavoc/source/entity/mobile/vehicle/anti_air/aa_updt.c", "aphavoc/source/entity/mobile/vehicle/routed/rv_updt.c",
    "eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c", "eech-campaign/crates/eech-engine-sys/csrc/eech_headless_devices.c", "eech-campaign/crates/eech-engine-sys/sources.txt",
    "eech-campaign/crates/eech-engine-sys/build/main.rs", "eech-campaign/crates/eech-engine-sys/Cargo.toml",
]


def git(*args):
    return subprocess.run(["git", "-C", str(REPO), *args], capture_output=True, check=True).stdout.decode("utf-8", "replace")


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


ranges = ["aphavoc", "eech-campaign/crates"]
differing_working_tree = sorted(git("diff", "--name-only", BASE, "--", *ranges).split())
differing_head = sorted(git("diff", "--name-only", BASE, "HEAD", "--", *ranges).split())
relied = {}
for path in RELIED_ON:
    in_base = git("rev-parse", BASE + ":" + path).strip()
    on_disk = git("hash-object", path).strip()
    relied[path] = {"blob_at_" + BASE: in_base, "blob_of_working_bytes": on_disk, "equal": in_base == on_disk}
extraction_path = PACKET / "tvb1-dormancy-evidence-20261107.json"
extraction = json.loads(extraction_path.read_text(encoding="utf-8"))
reports = {}
for profile in ("ordinary", "original"):
    report = PROFILE / profile / "engine-archive-nm.txt"
    meta = json.loads((PROFILE / profile / "profile.json").read_text(encoding="utf-8"))
    recorded = extraction["profiles"][profile]
    reports[profile] = {"report_sha256_now": sha(report), "report_sha256_recorded": recorded["report_sha256"], "equal": sha(report) == recorded["report_sha256"],
                        "archive_sha256_now": meta["archive_sha256"], "archive_sha256_recorded": recorded["archive_sha256"], "archive_equal": meta["archive_sha256"] == recorded["archive_sha256"]}
problems = []
if differing_working_tree != [GUARD] or differing_head not in ([GUARD], []):
    problems.append("the files differing from %s in the stated ranges are not exactly the guard test: working tree %r, HEAD %r" % (BASE, differing_working_tree, differing_head))
problems += ["relied-on source differs from %s: %s" % (BASE, p) for p, row in relied.items() if not row["equal"]]
problems += ["retained report or archive differs from the extraction's record: " + p for p, row in reports.items() if not (row["equal"] and row["archive_equal"])]
result = {
    "kind": "identity join of the Tacview disposition correction (tvr459): what the earlier blanket 'git diff empty over aphavoc and eech-campaign/crates' claim hid, and the exact identities it can be replaced by",
    "base_of_the_compiled_evidence": BASE, "head_at_capture": git("rev-parse", "HEAD").strip(), "ranges": ranges,
    "files_differing_from_base_in_working_tree": differing_working_tree, "files_differing_from_base_at_head_commit": differing_head, "expected_only": [GUARD],
    "relied_on_sources": relied, "retained_extraction": {"path": str(extraction_path), "sha256": sha(extraction_path), "recorded_repo_head": extraction["repo_head"],
                                                       "recorded_product_and_crate_source_changed_since_3abf": extraction["product_and_crate_source_changed_since_3abf"]},
    "retained_reports": reports, "problems": problems,
    "scope": "read-only identity join: no build, native or replay; it says the production and build inputs read by the disposition are unchanged since 3abf03e51 and that the one changed file in the stated ranges is the source-only guard test",
}
with open(OUT, "xb") as handle:
    handle.write((json.dumps(result, indent=2) + "\n").encode("utf-8"))
print(json.dumps({"differing_working_tree": differing_working_tree, "differing_head": differing_head, "relied_equal": sum(r["equal"] for r in relied.values()), "relied": len(relied), "problems": problems}))
sys.exit(1 if problems else 0)
