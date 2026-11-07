"""Metadata-only first bounded checkpoint of the witness-correction batch: register the SAME review's four P2s OPEN in the card, record the review/binding/CI publication, ONE CURRENT. Textual edits; the card keeps its CRLF
line endings. Usage: patch.py <repo>"""
import json
import pathlib
import sys

REPO = pathlib.Path(sys.argv[1])
card_path = REPO / "docs/handover/ACTIVE-EXTRACTION.json"
status_path = REPO / "STATUS.md"
card = card_path.read_bytes().decode("utf-8")
status = status_path.read_bytes().decode("utf-8")
eol = "\r\n" if "\r\n" in card else "\n"
assert card.count("\n") == card.count(eol), "mixed line endings in the card"
PK = "reference/d3-entity-collision/sector-height-native-witness-source-reviewed"
REVIEW = "96b2f4140e47ef17b0e672691e39173b3d920ea75282ec0cee60d56cdab8730f"
BINDING = "4ef8c75045809c6baa2e7cc16330011407848112cb2130bf8da5897047a50203"
CI = "f8d741fd5e8c46685a4c8133d8a0f40362ecd3aee9d01bfdb072b7783727aef8"
SRC = "6e41390439fc59f4fe091cf65ae13ab43aafba90"
WHO = "P2 of the SAME review " + REVIEW + " (independent-sector-height-native-witness-source-6e4139-20261107.json; CHANGES_REQUIRED, native_launch_clearance=false, Windows_build_clearance=false, no finding in the unchanged production, kernel, original-solver or finite 1b89 paths) of the M5/M6 witness source " + SRC + ": "
rows = [
    {"id": "ENV-SECTOR-HEIGHT-M5-ROUNDING-ORACLE-1", "evidence": WHO + "the driver's expected clearance subtraction (to_single) rounds to nearest, ties to even, but the native driver selects the accepted SSE round-toward-zero profile before boot and the real clearance float subtraction runs under it; the scripted test stubs use_reference_arithmetic and computes HANDOFF with ordinary f32, so its exact-valued controls do not establish the contract. A finite non-exact subtraction can produce the accepted RTZ word and differ from the expected one: changed_word reports UNREACHED for a reached, correctly consuming branch, and the control lower bound can be too high.",
     "missing_fact": "The expected subtraction uses the accepted round-toward-zero contract (no change to the original C expression, solver, tolerance or baseline); a finite non-exact subtraction where nearest and RTZ differ is accepted under the ACTUAL selected reference profile with the test FP state saved and restored; an exact positive and a genuinely wrong hand-off stay rejected; clean non-dominance is distinguished from an incorrectly formed expected word; the SAME reviewer's changed-material review.",
     "next_fix": "Being corrected in the sole lane (driver oracle + focused controls; no broad FP, i686 or O2 programme); closes only by the SAME reviewer."},
    {"id": "ENV-SECTOR-HEIGHT-M6-PACK-RETRY-1", "evidence": WHO + "the real store_session retries pack_session while growing its buffer, and each valid attempt repeats the sector reads; the driver treats ANY repeated sector read during save (also the second save of a load) as pack_duplicates and fails unconditionally, rejecting a legitimate original writer path before proving M6. The scripted pack_twice control models every repeat as a defect and not the real overflow and repack lifecycle. No claim is made that a native campaign has overflowed its initial buffer.",
     "missing_fact": "Consistent repeat reads of the real retry lifecycle are reconciled and reported (attempts, repeat telemetry qualified) with the unique final saved-sector relationship retained; an inconsistent repeated word, an omitted required saved or restored sector, or an unsupported repetition is non-pass; real store_session and the Core later boot stay (no synthetic writer, no provider restore as Core proof); positive legitimate retry controls in save and in the load's second pack, and single faults; the SAME reviewer's changed-material review.",
     "next_fix": "Being corrected in the sole lane (driver attempt reconciliation + focused controls); closes only by the SAME reviewer."},
    {"id": "ENV-SECTOR-HEIGHT-M6-SAVE-PATH-BOUND-1", "evidence": WHO + "the new wrapper checks the complete saved paths against 320-byte temporaries and then calls the original writer, whose data and script path buffers are 256 bytes populated with sprintf and strcat; the declared session domains (data_path[256], campaign_directory[64]) can build paths of 256 through 319 bytes that pass the new guard and overflow the writer. Restricting the basename to 24 bytes does not bound the whole path. A new fixture-entry precondition gap; no damaged native run is needed to establish it.",
     "missing_fact": "Both complete final writer paths (the data path with .sav and the script path with its extension, and the terminating NUL) are bounded to the actual smallest original writer buffer BEFORE store_session or any effect, by one shared helper the wrapper invokes and the focused real-C test calls directly (a fitting boundary path accepted, the first non-fitting path and both suffix variants refused, no file effect); safe-name, no-session and no-overwrite refusals retained; no rewrite of the original writer; the SAME reviewer's changed-material review.",
     "next_fix": "Being corrected in the sole lane (shared C path helper + focused real-C controls); closes only by the SAME reviewer."},
    {"id": "ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1", "evidence": WHO + "the paired and consumed proofs are incomplete: M5 uses global lookahead and pending booleans that do not bind lookahead and move to the same valid host id, pending state can survive unrelated callbacks, credit checks only finite elevation and min_height (identity, pre_y, answered_y, phase may be missing) and a missing physical answer can classify as a changed physical output; original= accepts a matching hand-off line without a whole successful original report or matching setup; M6 expect= accepts save-word lines of a truncated, failed or unmatched save report; the seed and positive reply parsers omit complete raw-list, name, sector-echo and ACK checks (empty comma segments are silently dropped). Diagnostic consumer and attribution gaps, not a claim that the production provider emitted malformed data.",
     "missing_fact": "The clearance window and the same-caller lookahead and movement relation bound by semantic identity and the complete current request and the real positive answer before consumption credit (a competing or missing identity or an intervening incompatible callback rejects; a bounded test-only real task-leader and guide fixture is authorized only if the relation cannot be established before intervention); whole successful original and save reference reports validated (matching mode, profile, setup, window, source and artifact relationships, unique ordered well-formed records, terminal PASS; partial, failed, duplicate, competing-actor, missing-field or wrong-setup references non-pass; an unpaired control stays explicitly unpaired); the complete raw-list, name, phase, sector, echo and seed ACK grammar checked before eligibility, intervention or seed credit; fatal, request and reply faults take precedence over a clean UNREACHED with no post-fatal engine inspection; positive-first single-fault controls per report; the SAME reviewer's changed-material review.",
     "next_fix": "Being corrected in the sole lane (driver consumer join and focused controls); closes only by the SAME reviewer. M5, M6, height-open-facts and all ten requirements stay required."},
]
marker = eol + "  ]," + eol + '  "next_action":'
assert card.count(marker) == 1


def block_of(row):
    full = {"id": row["id"], "status": "open", "blocks_cutover": False, "evidence": row["evidence"], "missing_fact": row["missing_fact"], "next_fix": row["next_fix"]}
    return "," + eol + "    " + json.dumps(full, indent=2, ensure_ascii=True).replace("\n", eol + "    ")


card = card.replace(marker, "".join(block_of(r) for r in rows) + marker)


def replace_line(text, prefix, value):
    assert text.count(prefix) == 1, prefix
    s = text.index(prefix)
    e = text.index(eol, s)
    return text[:s] + prefix + json.dumps(value, ensure_ascii=True) + "," + text[e:]


phase = ("Finite height gates VERIFIED at 1b89 (SAME review 8edddd93); the required M5/M6 witness SOURCE " + SRC[:9] + " (UNREVIEWED, native UNRUN) got CHANGES_REQUIRED from the SAME review " + REVIEW[:8] + ": FOUR P2s in the new diagnostics only (ENV-SECTOR-HEIGHT-M5-ROUNDING-ORACLE-1, -M6-PACK-RETRY-1, -M6-SAVE-PATH-BOUND-1, -WITNESS-CONSUMER-JOIN-1), "
         "no native or Windows build clearance. The batch correction is in progress (sole lane, test-only); the exact 579 CI passed all nine jobs. M5, M6, sector-height-host-move-open-facts and all ten D3 requirements remain OPEN; prep2.")
card = replace_line(card, '  "phase": ', phase)
nxt = ("Sole lane: correct the four bounded groups together (RTZ oracle, real store_session retry reconciliation, the shared writer path bound, the consumer join and whole-reference validation), freeze and STOP for the SAME changed-material review. Root: no Windows build or native launch until that review clears the corrected immutable source; the corrected head needs its own CI. "
       "M5 and M6 stay required OPEN; the 1b89 binaries and actuals are never relabelled. No D4/D5/TypeScript, no hints/RNG/hierarchy. All10, final current 40/all8+4 inputs, CI, review, normal merge and census remain.")
card = replace_line(card, '  "next_action": ', nxt)
item = ("SECTOR-HEIGHT M5/M6 WITNESS SOURCE REVIEW (SAME review " + REVIEW + ", sealed 2026-11-07T11:50:20Z): CHANGES_REQUIRED, four P2s registered OPEN at this checkpoint (see their rows), native_launch_clearance=false, Windows_build_clearance=false; scoped positive observations (the M6 call reaches the real store_session and the later saved-script boot; a natural fixed-window M5 plan is not intrinsically invalid) and no finding in the unchanged paths. "
        "Root's qualified binding v3 " + BINDING + ": 3976 PRE/POST rows equal, 18 changed sources and 150 packet entries literal in Git, ten final children exit 0; 17 extra entries PRE/POST equal but only 16 current literals (freeze-hw-witnesses-20261107.py changed after POST; the frozen no-reader-changed statement is false as written; a chronology erratum is published, the original packet preserved). "
        "The exact 579427c55 CI passed all nine jobs of all three workflows, no manual rerun (" + CI + ", arrived after the seal, retained separately, no finding waiver). Literal records: " + PK + ". Preparation count stays 2.")
marker2 = eol + "  ]," + eol + '  "blockers": ['
assert card.count(marker2) == 1
card = card.replace(marker2, "," + eol + "    " + json.dumps(item, ensure_ascii=True) + marker2)
json.loads(card)

first_end = status.index("\n")
head = status[:first_end]
assert head.startswith("# CURRENT — D3 Task49: the two REQUIRED witnesses"), head
new = (
    "# CURRENT — D3 Task49: the M5/M6 witness source `" + SRC[:9] + "` got CHANGES_REQUIRED (four P2s, SAME review `" + REVIEW[:8] + "`); the correction batch is in progress, test-only (7 November 2026)\n\n"
    "The SAME reviewer found no defect in the unchanged production, kernel, original-solver or finite `1b89` paths and four P2s in the NEW diagnostics, registered OPEN: `ENV-SECTOR-HEIGHT-M5-ROUNDING-ORACLE-1` (the clearance oracle must use the accepted round-toward-zero profile, not nearest), `ENV-SECTOR-HEIGHT-M6-PACK-RETRY-1` (the real `store_session` retries `pack_session`: consistent repeat reads are reconciled, inconsistent or unsupported ones fail), "
    "`ENV-SECTOR-HEIGHT-M6-SAVE-PATH-BOUND-1` (complete saved paths must fit the original writer's 256-byte buffers before any effect) and `ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1` (same-caller binding, whole reference reports, the whole reply grammar and fault precedence). No native launch or Windows build is cleared. The exact `579427c55` CI passed all nine jobs (it arrived after the seal; kept separately, no waiver). "
    "Root's binding v3 qualifies the provenance: 16 of 17 extra readers are current literals; the freeze reader changed after POST (a chronology erratum is published; the original packet is preserved). Literal records: [" + PK + "](eech-campaign/" + PK + "/README.md).\n\n"
    "**Still OPEN:** the four findings until the SAME reviewer; `ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1` and `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1` (native UNRUN); `sector-height-host-move-open-facts` and all ten D3 requirements; the corrected head needs its own CI. No D4/D5/TypeScript; no self-approval or Task49/D3/delivery claim; `D3-CURRENT.json` is Root's. Preparation count stays 2.\n\n"
)
demoted = "# HISTORICAL (superseded by the witness-source review " + REVIEW[:8] + ") — " + head[len("# CURRENT — "):]
status = new + demoted + status[first_end:]
card_path.write_bytes(card.encode("utf-8"))
status_path.write_bytes(status.encode("utf-8"))
print("ok")
