"""First compact checkpoint of the Core-load blocker batch (metadata only): ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1 CLOSED at SOURCE per the SAME review f61dac32, the new ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1 registered OPEN with the real failure, cause
evidence, limits and the bounded intended fix, phase/next_action/ready_evidence, ONE CURRENT. The card is round-tripped structurally. Usage: patch.py <repo>"""
import json
import pathlib
import sys

REPO = pathlib.Path(sys.argv[1])
card_path = REPO / "docs/handover/ACTIVE-EXTRACTION.json"
status_path = REPO / "STATUS.md"
raw = card_path.read_bytes().decode("utf-8")
assert json.dumps(json.loads(raw), indent=2, ensure_ascii=True).replace("\n", "\r\n") + "\r\n" == raw, "the card does not round-trip"
card = json.loads(raw)
status = status_path.read_bytes().decode("utf-8")
REVIEW = "f61dac32f008b38b4795eaa0dd73357fb6bcd1fabf5f017e59874fb1cc77f81a"
PK_REVIEW = "reference/d3-entity-collision/sector-height-reference-fix-reviewed"
PK_DIAG = "reference/d3-entity-collision/sector-height-native-core-load"
rows = {r["id"]: r for r in card["blockers"]}
cj = rows["ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1"]
assert cj["status"] == "open"
cj["status"] = "closed"
cj["evidence"] += (" CLOSED_SOURCE by the SAME review " + REVIEW + " (independent-sector-height-reference-fix-9cb480-source-20261107.json, SOURCE_CLEAR_FOR_BOUNDED_POSITIVE_FIRST_NATIVE_VALIDATION, findings none) of the reference correction 9cb480fbb: complete finite raw physical fields, effective map/campaign setup binding, the legitimate original-save-to-generated-script relationship and raw NaN/Inf histories preserved. Literal records: " + PK_REVIEW + ".")
cj["missing_fact"] = "None at the declared source scope; the native M5/M6 runs, the current diagnostic Lua's acceptance and the Core load below stay open."
cj["next_fix"] = "None for this finding; do not reopen without new actual evidence."
cj["closure_review_sha256"] = REVIEW
cj["closure_scope"] = "Closed at source and at the declared scope only (the SAME review " + REVIEW[:8] + " of 9cb480fbb); no native execution, M5/M6, height, Task49, D3 or delivery acceptance."
new_id = "ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1"
assert new_id not in rows
card["blockers"].append({
    "id": new_id, "status": "open", "blocks_cutover": False,
    "evidence": ("REAL native failure of the required M6 Core load (Root's exact 17f save-feature profile with the reviewed external Lua ff120f0b): the first real Core save passed native 0 (1748 sectors, 1 attempt, 300873-byte data, version 272, generated script HW9CBSAVE.SCH); a FRESH Core boot of that actual generated saved script stopped native 1 after all 1748 sector restores with "
                 "'Invalid list type (entity type = ENTITY_TYPE_SECTOR, index = 0, list type = LIST_TYPE_FORCE)' at en_list.c:356 (the load report's terminal verdict is FAILURE; the wrapper regex knowing only FAIL is a wrapper issue, not the cause; Root's original-save controller exit 1 is the expected-suffix case mistake .Sch vs .SCH, retained). No negative save/load is unlocked. "
                 "DIAGNOSIS (" + PK_DIAG + ", aggregates only): the saved sector section decodes EXACTLY to the save report's 1748 words (1027 non-zero) but the load stated ZERO for every sector, so the unpack buffer was zero where the file holds data; an all-zero stream reads the session's force root as entity index 0 = the sector 0 entity and the force loop asks it for a force-list successor: the fatal is a symptom, the list machinery and entity-index lifecycle are correct for a correct stream. "
                 "The legacy loader (parsgen.c FILE_TAG_SAVED_CAMPAIGN) sizes the data with the text-mode file_size (), reads it with a one-element fread (size + 1) and ignores unpack_session's overflow return; the data holds 1593 Ctrl-Z bytes from offset 167 (48 bytes after the side RLE). The exact C-runtime mechanism is not yet observed."),
    "missing_fact": ("The cause is a correct, whole delivery of the saved data to the unpack: size from the binary handle, a full read, and a loud failure for any short read or unpack overflow; then a real native positive pair (the real Core save and the real fresh Core boot of its generated script) with all 1748 restored words equal to the saved ones in order, before any negative load is credited. "
                     "Unknown: whether the text-mode size or the one-element fread is the Windows C-runtime mechanism (the fix removes both and names any other cause). No native replay for context, no bypass of the Core unpack, no fake success, no baseline repair."),
    "next_fix": ("Bounded fix in parsgen.c FILE_TAG_SAVED_CAMPAIGN only (product C: new Windows binaries need NEW source, the 17f binaries are not reused for it): binary-handle size, fread (buffer, 1, size), debug_fatal on a short read or an unpack overflow; source guard and the diagnosis packet; then SAME review, then Root's native positive pair. M5, M6, height-open-facts and all ten requements stay required."),
})
card["phase"] = ("The SAME review f61dac32 closed ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1 at source (conditional native clearance). Root's first native M6 positive run: the real Core save passed native 0 but the fresh Core boot of its generated saved script failed native 1 (Invalid list type, en_list.c:356). Bounded diagnosis done: the saved data decodes exactly, the load stated zero for all 1748 sectors, the fatal is a symptom of an all-zero unpack input "
                 "and the legacy loader ignores short delivery. ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1 is OPEN with a bounded loader fix in progress (product C: new source). M5, M6, sector-height-host-move-open-facts and all ten D3 requirements remain OPEN; prep2.")
card["next_action"] = ("Sole lane: land the bounded parsgen.c loader fix with its guard and evidence, freeze and STOP for the SAME changed-material review; the 17f binaries are not reused for this product C change (new Windows binaries need new source), then Root's native positive pair. No D4/D5/TypeScript. All10, final current 40/all8+4 inputs, CI, review, normal merge and census remain.")
card["ready_evidence"].append(
    "SECTOR-HEIGHT NATIVE CORE LOAD DIAGNOSIS (7 Nov 2026): Root's exact 17f save-feature native positive run (external Lua 9cb/ff120f0b): save native 0 (1748 sectors, 300873 B, version 272, SAVED_CAMPAIGN of LEBANON.CHC), fresh Core boot of the generated script native 1 at en_list.c:356 after 1748 restores; all 1179 original inputs and both artifacts unchanged; Root's original-save controller exit 1 is the .Sch/.SCH expected-suffix mistake (read-only audit dabd6e9c, retained). "
    "Source review f61dac32 (CLOSED_SOURCE consumer join), Root binding e1b5cad0 and the exact beb all-nine-jobs CI read-back are published literally in " + PK_REVIEW + "; the diagnosis (decode of the saved sector section equal to the save report, load stated all zero, loader weaknesses) is in " + PK_DIAG + ". Preparation count stays 2.")
card_path.write_bytes((json.dumps(card, indent=2, ensure_ascii=True).replace("\n", "\r\n") + "\r\n").encode("utf-8"))

first_end = status.index("\n")
head = status[:first_end]
assert head.startswith("# CURRENT — D3 Task49: the consumer-reference residual of review `507f49b7`"), head
new = (
    "# CURRENT — D3 Task49: the real Core load of the generated saved script fails natively; the fatal is a symptom of an all-zero unpack input and a bounded loader fix is in progress (`ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1`, 7 November 2026)\n\n"
    "The SAME review `f61dac32` closed `ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1` at source (conditional native clearance). Root's first native M6 run on the exact `17f` save-feature profile: the real Core save passed native 0, but a fresh Core boot of that generated saved script stopped native 1 after all 1748 sector restores with `Invalid list type (ENTITY_TYPE_SECTOR, index 0, LIST_TYPE_FORCE)` at `en_list.c:356`; no negative save/load is unlocked. "
    "**Diagnosis** ([" + PK_DIAG + "](eech-campaign/" + PK_DIAG + "/README.md)): the saved sector section decodes exactly to the save report's words, but the load stated zero for every sector, so the unpack input was zero where the file holds data; an all-zero stream reads the session's force root as entity index 0, the sector 0 entity, which is what the force loop then asks for a force-list successor. "
    "The legacy loader sizes the data with a text-mode helper, reads it with a one-element `fread` and ignores the unpack's overflow flag; the file holds Ctrl-Z bytes from offset 177 bytes in. The exact C-runtime mechanism is not yet observed; the bounded fix makes the loader size the data from its binary handle, require every byte and fail loudly, and a native run of the real positive pair is the validation. The literal review, Root's binding and the exact beb CI read-back are in [" + PK_REVIEW + "](eech-campaign/" + PK_REVIEW + "/README.md).\n\n"
    "**Still OPEN:** `ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1`; `ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1` and `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1` (no native acceptance); `sector-height-host-move-open-facts` and all ten D3 requirements; the corrected head needs its own CI; product C changes need new Windows binaries (the 17f binaries are not reused for them). No D4/D5/TypeScript; no self-approval or Task49/D3/delivery claim; `D3-CURRENT.json` is Root's. Preparation count stays 2.\n\n"
)
new = new.replace("from offset 177 bytes in", "from offset 167")
demoted = "# HISTORICAL (superseded by the Core-load diagnosis) — " + head[len("# CURRENT — "):]
status_path.write_bytes((new + demoted + status[first_end:]).encode("utf-8"))
print("ok")
