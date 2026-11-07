"""First compact checkpoint of the load-order batch: publishes packet sector-height-native-load-followup: the three SAME-reviewer constraint records literally and an ADDENDUM to the Core-load diagnosis (the earlier diagnosis packet stays at its own identity):
the mechanism is now established from source and retained bytes, the qualifications of the save/audit exits, and the corrected wording of what the failing call and the 1748 restores are. Usage: publish.py <repo-root>."""
import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1])
EXT = pathlib.Path("R:/d3-evidence")
BASE = "eech-campaign/reference/d3-entity-collision/"
REL = BASE + "sector-height-native-load-followup"
PACK = REPO / REL
RECORDS = ["Root-height-native-load-binary-size-cause-20261107.json", "Root-height-native-load-restore-order-constraint-20261107.json", "Root-height-native-load-reviewer-constraints-20261107.json"]
problems = []


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


PACK.mkdir(parents=True, exist_ok=True)
(PACK / ".gitattributes").write_bytes((REPO / (BASE + "sector-height-proof-guards-reviewed/.gitattributes")).read_bytes())
for name in RECORDS:
    (PACK / name).write_bytes((EXT / name).read_bytes())
size = json.loads((EXT / RECORDS[0]).read_text(encoding="utf-8"))
order = json.loads((EXT / RECORDS[1]).read_text(encoding="utf-8"))
if "167" not in " ".join(size["facts"]) or order["actuals"]["legitimate_pre_restore_boot_insertions"] != 282933:
    problems.append("the constraint records are not the expected ones")
addendum = {
    "kind": "addendum to the Core-load diagnosis (sector-height-native-core-load, committed at b23c06c16 and left at its own identity): what the same-reviewer constraint records and a source check add; no native replay, no corrected load",
    "supersedes": {
        "diagnosis.json cause / ranked_candidates / 'mechanism not observed'": "the mechanism is established from source plus retained bytes: the compiled shim's text file_size () is 167, not 300873",
    },
    "mechanism": {
        "chain": ["modules/system/files.c 538/555/571: file_size () opens its input with plain \"r\"", "compat/eech_api.h 35: fopen is eech_fopen", "csrc/eech_win32.c 1399..1401: a plain-r (no b, no +) read goes to open_text_for_reading", "csrc/eech_win32.c 1260..1263: that reader stops at the first 0x1a (Ctrl-Z)"],
        "retained_bytes": "the saved data (300873 bytes, SHA-256 ff0fa7b04ea46190de638a72807e12a526744a60f2978ac696824014afb37862, the same bytes in the save root and the load root) has its first 0x1a at byte 167",
        "consequence": "file_size () = 167; parsgen.c passes 163 bytes after the 4-byte version number into unpack_session; unpack_unsigned_data / unpack_signed_data answer 0 for every value once their buffer is exhausted: the zero restored heights and the later numeric force index 0",
        "verified_by_this_lane": "the shim chain lines above were read in the repository at the loader-fix source (compat/eech_api.h and csrc/eech_win32.c are inputs of the 17f build); the first-0x1a offset is the decode result's first_ctrl_z_offset (167)",
        "not_needed_to_explain": "the one-element fread (size + 1, 1) of the old loader; the loader fix removes it as well",
        "loader_fix_consistency": "binary-handle sizing, a counted full read and an unpack-overflow fatal (parsgen.c, commit 90e9547ee) address exactly this; the global text semantics (parser CRLF/Ctrl-Z), the pack schema and the entity indices are unchanged",
        "claim": "NO corrected load has run: the real load remains failure (native 1); the loader fix is unreviewed and native-unrun",
    },
    "corrected_wording": {
        "failing_call": "en_list.c:356 is default_get_entity_child_succ (not a get-first-child call): unpack_session reads the session's FORCE root through unpack_list_root as a numeric heap index (fc_pack.c deliberately does not serialize the force link), unpacks each force and then asks for the force successor. A zero stream makes the root index 0, so the entity asked is entities[0], a sector; this does not by itself show anything about get_session_entity (); the force access is not suppressed",
        "restores": "1748 restore callbacks all stating 0x00000000 (the first three also stored 0x00000000, unlike the seeded bf800000/ff800000/7fc12345 of the save) are NOT raw-history parity and NOT a successful load",
        "insertion_counts": "287934 inserts = 282933 before the first restore (the supported boot's baseline: CAMPAIGN_DATA is built before SAVED_CAMPAIGN) + 5001 after the last restore before the fatal; no second pack; the 5001 alone is not a successful load",
    },
    "qualified_exits": {
        "save": "the real save run's native exit was 0 (actual_native_exit 0); the controller's own exit was 1 (actual_exit 1: an expected-added-inputs assertion, the .Sch versus real .SCH case spelling, retained unchanged); the read-only audit reader exited 0 and records original_controller_exit 1 beside it (dabd6e9c); the audit's INITIAL attempt failed on the assumption that the saved script embeds the saved name (it does not: parsgen.c derives the .sav name from the current campaign name before overwriting it) and its reader is preserved separately",
        "load": "native exit 1 (actual_native_exit 1), controller exit 1, terminal verdict status=FAILURE (the wrapper regex that knew only FAIL is a wrapper issue, not the cause)",
        "earlier_statement_corrected": "the diagnosis packet's actual_inputs 'save_receipt_exit: 0' means the NATIVE save exit 0, not the controller's exit (1) and not the audit reader's exit (0)",
    },
    "new_blocker": "ENV-SECTOR-HEIGHT-NATIVE-LOAD-ORDER-DIAGNOSTIC-1: the driver's load verdict refused any insertion before the first restore (lua 1021 of the 9cb literal), a gate no supported generated script can pass; corrected in the next commit (positions only, telemetry kept, strict saved-state checks kept)",
}
(PACK / "diagnosis-addendum.json").write_text(json.dumps(addendum, indent=2) + "\n", encoding="utf-8", newline="\n")
(PACK / "README.md").write_text(
    "# Native load follow-up: constraint records and the diagnosis addendum\n\n"
    "Three SAME-reviewer records about Root's first native M6 load are kept literally (`* -text`): the binary-size cause (the compiled shim's text `file_size ()` is 167, not 300873), the restore-order constraint (a never-passable diagnostic gate: the supported boot's baseline insertions precede the restores) and the reviewer's facts about the failing call and the zero restores. "
    "`diagnosis-addendum.json` supersedes the 'mechanism not observed' statements of the earlier diagnosis packet (left at its own identity), corrects the wording of the failing call and the restores, qualifies the save/audit exits and names the new diagnostic-order blocker. **No corrected load has run**: the real load remains a native failure; the loader fix and the diagnostic correction are unreviewed. Not a native result, a fix claim or an acceptance.\n",
    encoding="utf-8", newline="\n")
subprocess.run(["git", "-C", str(REPO), "add", "-A", REL], check=True)
literal = {}
for name in RECORDS:
    blob = subprocess.run(["git", "-C", str(REPO), "cat-file", "blob", ":" + REL + "/" + name], capture_output=True, check=True).stdout
    literal[name] = {"git_index_blob_bytes_equal_external_file": blob == (EXT / name).read_bytes(), "bytes": len(blob), "sha256": hashlib.sha256(blob).hexdigest()}
    if blob != (EXT / name).read_bytes():
        problems.append("not literal in Git: " + name)
(PACK / "literal-bytes-check.json").write_text(json.dumps({"kind": "actual Git index blob bytes (git cat-file) against the external records", "files": literal, "problems": problems}, indent=2) + "\n", encoding="utf-8", newline="\n")
rows = [{"path": str(p.relative_to(PACK)).replace("\\", "/"), "bytes": p.stat().st_size, "sha256": sha(p)} for p in sorted(PACK.rglob("*")) if p.is_file() and p.name != "manifest.json"]
(PACK / "manifest.json").write_text(json.dumps({"files": rows, "scope": "Literal same-reviewer constraint records and a diagnosis addendum; not a native result, fix claim or acceptance."}, indent=2) + "\n", encoding="utf-8", newline="\n")
subprocess.run(["git", "-C", str(REPO), "add", "-A", REL], check=True)
print(json.dumps({"problems": problems, "files": len(rows)}))
sys.exit(1 if problems else 0)
