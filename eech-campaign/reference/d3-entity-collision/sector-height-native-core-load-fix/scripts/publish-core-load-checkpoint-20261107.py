"""First compact checkpoint of the Core-load blocker batch: publishes (1) the literal SAME source review f61dac32 of the reference correction 9cb480fbb, Root's binding of it and the exact beb CI read-back (packet sector-height-reference-fix-reviewed)
and (2) the safe, aggregate-only DIAGNOSIS of the fresh Core boot of the real generated saved script (packet sector-height-native-core-load): facts, source locations, the causal chain, ranked candidate causes, the intended bounded fix.
Usage: publish.py <repo-root>. Earlier packets are not touched."""
import hashlib
import json
import pathlib
import subprocess
import sys

REPO = pathlib.Path(sys.argv[1])
EXT = pathlib.Path("R:/d3-evidence")
BASE = "eech-campaign/reference/d3-entity-collision/"
GIT_ATTR = (REPO / (BASE + "sector-height-proof-guards-reviewed/.gitattributes")).read_bytes()
problems = []


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def stage(rel):
    subprocess.run(["git", "-C", str(REPO), "add", "-A", rel], check=True)


def manifest(pack, scope):
    rows = [{"path": str(p.relative_to(pack)).replace("\\", "/"), "bytes": p.stat().st_size, "sha256": sha(p)} for p in sorted(pack.rglob("*")) if p.is_file() and p.name != "manifest.json"]
    (pack / "manifest.json").write_text(json.dumps({"files": rows, "scope": scope}, indent=2) + "\n", encoding="utf-8", newline="\n")


# ------------------------------------------------------------------ (1) literal review records
REVIEWED = {
    "independent-sector-height-reference-fix-9cb480-source-20261107.json": "f61dac32f008b38b4795eaa0dd73357fb6bcd1fabf5f017e59874fb1cc77f81a",
    "height-reference-Root-binding-beb-20261107.json": "e1b5cad058845f3c6a3d3d7b7fa6a1ac8846425d1ca8a061ae76b524af2cfb64",
    "height-reference-beb-all9-CI-readback-20261107.json": "9858cdb809c5ab5bdd88b32f22900f4261eacf54c8d35e98cdbbec64eba63d0a",
}
rel1 = BASE + "sector-height-reference-fix-reviewed"
pack1 = REPO / rel1
pack1.mkdir(parents=True, exist_ok=True)
(pack1 / ".gitattributes").write_bytes(GIT_ATTR)
for name, want in REVIEWED.items():
    if sha(EXT / name) != want:
        problems.append("sealed input differs: " + name)
    (pack1 / name).write_bytes((EXT / name).read_bytes())
review = json.loads((EXT / "independent-sector-height-reference-fix-9cb480-source-20261107.json").read_text(encoding="utf-8"))
decision = review["decision"]
if decision["findings"] != [] or decision["closed_source_ids"] != ["ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1"] or not decision["native_launch_clearance"] or decision["M6_native_Core_save_load_accepted"]:
    problems.append("the review's decision is not the consumer join closed at source with conditional native clearance and no acceptance")
ci = json.loads((EXT / "height-reference-beb-all9-CI-readback-20261107.json").read_text(encoding="utf-8"))
if not ci.get("all9green") or not ci["source"].startswith("beb57c879"):
    problems.append("the CI read-back is not all nine jobs of the exact beb head")
(pack1 / "README.md").write_text(
    "# SAME review f61dac32 of the consumer-reference correction 9cb480fbb (literal records)\n\n"
    "The review found nothing open in the residual and closes `ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1` at SOURCE (the three other source closures stay those of review 507f49b7); native launch clearance is conditional (bounded positive-first, reuse of the exact 17f binaries with a separately bound external Lua) and M5, M6, the whole extraction, Task49, D3 and delivery are not accepted. "
    "Root's binding of the exact clean beb head and the exact beb CI read-back (all nine jobs, three workflows) are kept beside it. Literal copies (`* -text`), not a native result, review of any later change, or acceptance.\n",
    encoding="utf-8", newline="\n")
stage(rel1)
literal = {}
for name in REVIEWED:
    blob = subprocess.run(["git", "-C", str(REPO), "cat-file", "blob", ":" + rel1 + "/" + name], capture_output=True, check=True).stdout
    literal[name] = {"git_index_blob_bytes_equal_external_file": blob == (EXT / name).read_bytes(), "bytes": len(blob), "sha256": hashlib.sha256(blob).hexdigest()}
    if blob != (EXT / name).read_bytes():
        problems.append("not literal in Git: " + name)
(pack1 / "literal-bytes-check.json").write_text(json.dumps({"kind": "actual Git index blob bytes (git cat-file) against the external sealed files", "files": literal, "problems": problems}, indent=2) + "\n", encoding="utf-8", newline="\n")
manifest(pack1, "Literal copies of the SAME source review of the consumer-reference correction, Root's binding of the exact beb head and the exact beb CI read-back; not a native result or acceptance.")
stage(rel1)

# ------------------------------------------------------------------ (2) the diagnosis of the fresh Core boot of the generated saved script
rel2 = BASE + "sector-height-native-core-load"
pack2 = REPO / rel2
pack2.mkdir(parents=True, exist_ok=True)
(pack2 / ".gitattributes").write_bytes(GIT_ATTR)
for name in ("decode-saved-sector-section-20261107.py", "hw-core-load-decode-result-20261107.json"):
    (pack2 / name.replace("hw-core-load-", "")).write_bytes((EXT / name).read_bytes())
decode = json.loads((EXT / "hw-core-load-decode-result-20261107.json").read_text(encoding="utf-8"))
if not (decode["joins"]["decoded_words_equal_save_report_in_order"] and decode["load_report"]["restored_words_all_zero"] and decode["decoded"]["words_nonzero"] == 1027 and decode["decoded"]["rle_sector_total"] == 1748):
    problems.append("the decode result is not the expected diagnosis input")
diagnosis = {
    "kind": "bounded diagnosis of ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1 (fresh Core boot of the real generated saved script fatal at en_list.c:356); aggregates only, no retail data",
    "task": "EEDCS-49", "findings_open": ["ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1"],
    "actual_inputs": {
        "plan": "R:/d3-evidence/height-witness-native-9cb-plan-20261107.json", "save_receipt_exit": 0, "load_receipt_native_exit": 1, "runs": "D:/EECH-Migration-Evidence/height-witness-native-9cb-runs-20261107/{save,load} (outside Git)",
        "saved_data_sha256": decode["saved_data"]["sha256"], "saved_script_sha256": "ca41e2cb207a2aef2005c929c2f6013c17db6d7cc96374c6492bde2c5885c833", "session_version": decode["saved_data"]["version_int"],
    },
    "facts": [
        {"id": "F1", "fact": "the saved data is self-consistent: decoding its sector section with the generic bit routines (reproduced from en_pack.c) yields 1748 sector words, 1027 non-zero, EQUAL IN ORDER to the save report's 1748 packed words; pack and unpack of the sector section are symmetric (sc_pack.c) and the session/force sections read in the order they are packed (en_sessn.c)", "source": "decode-result.json joins.decoded_words_equal_save_report_in_order=true"},
        {"id": "F2", "fact": "the real load stated ZERO for all 1748 sectors although 1027 saved words are non-zero (restored_words_all_zero=true, 1027 mismatches, including the three seeded NaN/infinite/negative histories): the unpack input was zero where the file holds data, i.e. zero from before the first sector word (file byte 7116) or earlier; mb_hostentity.h memcmp-checks that the host stored exactly the stated bits, so the driver recorded what the engine really stated", "source": "decode-result.json joins + load witness restore rows"},
        {"id": "F3", "fact": "the fatal is a SYMPTOM: unpack_local_session_data reads the session's force-list root as a 20-bit signed entity index (ENTITY_INDEX_DONT_CARE = -1, index 0 = entities[0] = the sector 0 entity); an all-zero stream reads index 0, so unpack_session's force loop (en_sessn.c) asks get_local_entity_child_succ (sector 0, LIST_TYPE_FORCE) and default_get_entity_child_succ stops at en_list.c:356 'Invalid list type (entity type = ENTITY_TYPE_SECTOR, index = 0, list type = LIST_TYPE_FORCE)'. The list machinery and the entity-index lifecycle are correct for a correct stream", "source": "en_sessn.c unpack_session, ss_pack.c unpack_local_session_data, en_heap.h get_local_entity_safe_ptr, en_list.c:356"},
        {"id": "F4", "fact": "the loader ignores every signal that the data did not arrive: parsgen.c FILE_TAG_SAVED_CAMPAIGN sizes the buffer with file_size (), which opens the BINARY file a second time in TEXT mode, reads it with fread (buffer, size + 1, 1, f) (a short read returns 0 elements and is not checked) and discards unpack_session's overflow return; unpack_unsigned_data/unpack_signed_data return 0 forever once the buffer size is exceeded. The saved data holds 1593 Ctrl-Z (0x1A) bytes, the first at offset 167, 48 bytes after the side RLE ends (bit 954) and long before the first sector word (byte 7116), and 14 CRLF pairs: a text-mode size or EOF decision of the C runtime would cut the unpack off right there. en_pack.c itself records that saved campaigns do not load with builds that are not Watcom's (JB 030311 'Gotcha', pack_bit/unpack_bit generic C versions)", "source": "parsgen.c ~2253-2310, modules/system/files.c file_size, en_pack.c unpack_*_data"},
    ],
    "cause": "the Core unpack was fed an input that is zero beyond an early point of the file (F2) and the legacy loader neither detects nor reports that (F4); the list fatal is the first place the zeros become observable (F3). The exact C-runtime mechanism (the text-mode file_size () size, the one-element fread, or another) is NOT yet observed: no native replay was made for context",
    "ranked_candidates": [
        "file_size () text-mode size shorter than the file (Ctrl-Z at byte 167), the unpack overflowing into zeros while the version int and the first RLE bytes were valid",
        "fread (buffer, size + 1, 1, f) delivering only part of the data on the Windows C runtime (a partial element is indeterminate)",
        "anything else that makes the unpack buffer differ from the file: now observable because the loader will state the sizes",
    ],
    "intended_fix": "aphavoc/source/ai/parser/parsgen.c, FILE_TAG_SAVED_CAMPAIGN only: size the data from the binary handle that was opened for it (fseek/ftell on it), read it with fread (buffer, 1, size, f) and require every byte, and make an unpack_session overflow a debug_fatal naming the file and sizes. pack_session sizes the data to the last partly used byte (close_pack_buffer), so a correct unpack never overflows. No campaign decision, pack format, host contract or sector-height rule changes; the host still receives only the words the Core unpack reads",
    "what_it_does_not_establish": [
        "that the text-mode size or the one-element fread IS the mechanism: the fix removes both candidates and turns any other cause into a named fatal; only a native run of the real positive pair can show the load works",
        "any native result, M6 acceptance or negative-load credit; a Linux check cannot reproduce the Windows C runtime's text-mode and partial-element behaviour",
        "that the legacy faction/parser.c saved-data loader (not on this boot path) needs the same change",
    ],
    "decode_result": decode,
}
(pack2 / "diagnosis.json").write_text(json.dumps(diagnosis, indent=2) + "\n", encoding="utf-8", newline="\n")
(pack2 / "README.md").write_text(
    "# Core load of the real generated saved script: bounded diagnosis (ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1)\n\n"
    "First real Core save (feature profile of the exact 17f) passed native 0. A fresh Core boot of that actual generated saved script stopped native 1 after 1748 sector restores with `Invalid list type (ENTITY_TYPE_SECTOR, index 0, LIST_TYPE_FORCE)` at `en_list.c:356`. "
    "**The fatal is a symptom**: the saved sector section decodes exactly to the save report's words (`decode-result.json`), but the load stated zero for every sector, so the unpack buffer was zero where the file holds data; an all-zero stream reads the session's force root as entity index 0, the sector 0 entity, and the force loop asks it for a force-list successor. "
    "The legacy loader (`parsgen.c`, `FILE_TAG_SAVED_CAMPAIGN`) sizes the data with a text-mode helper, reads it with a one-element `fread` and ignores the unpack's overflow flag; the data holds Ctrl-Z bytes from offset 167. See `diagnosis.json` for facts, locations, ranked candidates, the intended bounded fix and what it does not establish.\n\n"
    "`decode-saved-sector-section-20261107.py` re-implements only the sector section over the retained private artifacts and writes aggregates; the artifacts, words and coordinates stay outside Git. Not a native result, a fix claim, or an acceptance of M6.\n",
    encoding="utf-8", newline="\n")
stage(rel2)
manifest(pack2, "Safe aggregate-only diagnosis of the fresh Core boot failure of the real generated saved script; no retail data; not a native result, fix claim or acceptance.")
stage(rel2)
print(json.dumps({"problems": problems, "pack1": len(list(pack1.rglob("*"))), "pack2": len(list(pack2.rglob("*")))}))
sys.exit(1 if problems else 0)
