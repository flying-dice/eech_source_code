"""Metadata-only first checkpoint of the consumer-reference correction: the three source-closed ids are recorded CLOSED per the SAME review 507f49b7, the consumer join stays OPEN with both residual facts,
phase/next_action replaced, one ready_evidence item, ONE CURRENT. The card is round-tripped structurally (indent=2, ensure_ascii, CRLF + final CRLF: byte-identical when unchanged). Usage: patch.py <repo>"""
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
PK = "reference/d3-entity-collision/sector-height-native-witness-fix-reviewed"
REVIEW = "507f49b7aff355115713c7ec325b6fe250601991be7866c8ca6eaadad53dbf99"
BINDING = "f2ad25d984e1ed419f8c92a151e49445020c4ca6c030d05269da572de3096a21"
FIX = "e42bb19a0845106df6f03d08774be32cbaa87d52"
HEAD = "17f4b824b468c608ef488423b6c568382dd12798"
rows = {r["id"]: r for r in card["blockers"]}
SCOPE = "Closed at source and at the declared scope only (the SAME review " + REVIEW[:8] + " of " + FIX[:9] + "); no native execution, current-Lua acceptance, M5/M6, height, Task49, D3 or delivery acceptance."
CLOSURE = {
    "ENV-SECTOR-HEIGHT-M5-ROUNDING-ORACLE-1": "The toward-zero narrowing follows an exact-double-difference check; an unformable expected value is a FAILURE; the harness invokes the real reference SSE selection, reads back and restores MXCSR; finite non-exact controls accept the toward-zero result and reject the adjacent nearest one; the old-driver RED is retained. Narrow x86-64 control, not native fw_move, i686/x87 or universal FP coverage.",
    "ENV-SECTOR-HEIGHT-M6-PACK-RETRY-1": "Real-writer-order repeated sector reads reconcile into attempts with exact order, count and word comparison and a unique final saved-sector relationship; save and load-second-pack retry positives pass; changed word, omitted sector, unsupported ordering and excessive attempts fail; the old-driver RED is retained. Real store_session overflow, file and load behaviour is still unrun native validation.",
    "ENV-SECTOR-HEIGHT-M6-SAVE-PATH-BOUND-1": "The wrapper calls one shared C helper before any file access or store_session; terminated session arrays and both complete paths with suffix and NUL are bounded to the writer's 256 bytes; shared-helper controls cover 255 accepted and 256 refused for the data and the script path and no-file, empty-output refusal; the guard pins capacities and call order. The helper was first GREEN on new code, not an old-helper RED.",
}
for fid, text in CLOSURE.items():
    r = rows[fid]
    assert r["status"] == "open" and r["blocks_cutover"] is False, fid
    r["status"] = "closed"
    r["evidence"] += " CLOSED_SOURCE by the SAME review " + REVIEW + " (independent-sector-height-native-witness-fix-e42bb-source-20261107.json): " + text
    r["missing_fact"] = "None at the declared source scope; native execution of store_session, pack_session, unpack_session and fw_move.c stays unrun and the changed Lua is not accepted."
    r["next_fix"] = "None for this finding; do not reopen without new actual evidence."
    r["closure_review_sha256"] = REVIEW
    r["closure_scope"] = SCOPE
cj = rows["ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1"]
assert cj["status"] == "open"
cj["evidence"] += (" PARTIALLY_CLOSED_RESIDUAL_OPEN by the SAME review " + REVIEW + ": the same-host-id read/lookahead/move state, live hand-off and answer completeness, strict raw request/reply/seed ACKs, paired trajectory checks and whole-report schema/mode/profile/terminal-PASS validation of references are accepted at source. RESIDUAL (two facts, lua:237/271/285/805): "
                   "(1) control= requires hand-off, trajectory and pair records and window equality but never requires its min_height and answered_y to be present, valid, finite raw words: removing only answered_y from a successful paired control passes loading and a valid changed-word run can compare its real answer to nil and emit physical=changed while PASS; removing min_height treats absent evidence as a changed hand-off. "
                   "(2) setup validation compares scenario, seed, frame_ms and hours only: the existing campaign field is ignored and the effective map identity is absent, so another effective M5 campaign or map can pass the pre-intervention setup gate (aggregate trajectory counters cannot replace identity).")
cj["missing_fact"] = ("Every field a parity or effect claim consumes (hand-off min_height and answered_y, trajectory and counts) is present, valid and finite as a RAW WORD in the supplied reference before output, boot or intervention; a missing, malformed, nonfinite, duplicate, failed, truncated or mismatched reference is NONPASS and never a changed physical output (standalone controls stay explicitly unpaired); "
                      "the effective M5 campaign/map setup is bound for original, control and intervention by semantic setup, not by fresh root path (matching defaults and explicitly matching overrides stay valid); for M6 the legitimate original-save-to-generated-script relationship is preserved and a saved raw NaN history is untouched (the finiteness rule is ONLY for the physical control fields); "
                      "positive-first then single-removal and single-damage controls; the SAME reviewer's changed-material review. No production API, solver, framework, baseline, tolerance, window or actor hunt.")
cj["next_fix"] = ("Being corrected in the sole lane (Lua witness, focused Rust driver tests, contract, evidence only; no C, Rust product, build, Cargo or compiled input change); closes only by the SAME reviewer. Root owns the caches and builds the immutable exact 17f; no cargo/rustup/docker/wsl until Root's released marker.")
phase = ("The SAME review " + REVIEW[:8] + " of the correction " + FIX[:9] + " closed three findings at source (ENV-SECTOR-HEIGHT-M5-ROUNDING-ORACLE-1, -M6-PACK-RETRY-1, -M6-SAVE-PATH-BOUND-1), kept ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1 OPEN_RESIDUAL (control physical words and effective campaign/map setup of a reference) and gave a CONDITIONAL build-only Windows clearance for the exact 17f; native_launch_clearance=false, the current diagnostic Lua is not accepted. "
         "The residual reference correction is in progress (sole lane; Lua, tests, contract, evidence only). M5, M6, sector-height-host-move-open-facts and all ten D3 requirements remain OPEN; prep2.")
card["phase"] = phase
card["next_action"] = ("Sole lane: correct only the residual reference validation (complete raw finite control physical fields, effective campaign/map setup binding), freeze and STOP for the SAME changed-material review; no C, Rust product, build, Cargo or compiled-input change. Root builds the immutable exact 17f and reuses compiled binaries only via exact compiled-input equivalence and a separately bound corrected external Lua; "
                       "native stays blocked until that review clears the corrected source and a source-bound positive-first plan exists. M5 and M6 stay required OPEN. No D4/D5/TypeScript. All10, final current 40/all8+4 inputs, CI, review, normal merge and census remain.")
card["ready_evidence"].append(
    "SECTOR-HEIGHT M5/M6 WITNESS CORRECTION REVIEW (SAME review " + REVIEW + ", sealed 2026-11-07T12:41:33Z) of " + FIX + " (metadata " + HEAD[:9] + "): CHANGES_REQUIRED_DIAGNOSTIC_REFERENCE_ONLY; ENV-SECTOR-HEIGHT-M5-ROUNDING-ORACLE-1, -M6-PACK-RETRY-1 and -M6-SAVE-PATH-BOUND-1 CLOSED_SOURCE; ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1 OPEN_RESIDUAL (see its row); Windows_build_clearance=true CONDITIONAL on the exact 17f (read-only profile/object/export/payload sealing, no execution of the diagnostic), native_launch_clearance=false, current Lua not accepted. "
    "Root's binding of the exact clean 17f " + BINDING + ": 3976 PRE/POST rows current and Git-qualified, 90 packet entries literal, 23 extras of which 22 current plus the explicit publisher-only freeze-script crash fix after POST (PRE body retained, f54d678f to cfc428df), ten final children 0 with 673 instances. Literal records: " + PK + ". Preparation count stays 2.")
card_path.write_bytes((json.dumps(card, indent=2, ensure_ascii=True).replace("\n", "\r\n") + "\r\n").encode("utf-8"))

first_end = status.index("\n")
head = status[:first_end]
assert head.startswith("# CURRENT — D3 Task49: the four P2s of the M5/M6 witness source review"), head
new = (
    "# CURRENT — D3 Task49: the SAME review `" + REVIEW[:8] + "` closed three of the four witness findings at source and left ONE residual (`ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1`); the reference-validation correction is in progress, test-only (7 November 2026)\n\n"
    "Closed at source and declared scope: `ENV-SECTOR-HEIGHT-M5-ROUNDING-ORACLE-1`, `ENV-SECTOR-HEIGHT-M6-PACK-RETRY-1`, `ENV-SECTOR-HEIGHT-M6-SAVE-PATH-BOUND-1`. Residual, two facts: a supplied `control=` report's `min_height` and `answered_y` are not required to be present, valid, finite raw words (a missing answer can print `physical=changed`), and the effective campaign / map setup of a reference is not bound. "
    "Root may compile the exact frozen `17f` read-only (conditional Windows build clearance); no native launch is cleared and the current diagnostic Lua is not accepted. Literal records: [" + PK + "](eech-campaign/" + PK + "/README.md).\n\n"
    "**Still OPEN:** the consumer join until the SAME reviewer; `ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1` and `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1` (native UNRUN); `sector-height-host-move-open-facts` and all ten D3 requirements; the corrected head needs its own CI. No D4/D5/TypeScript; no self-approval or Task49/D3/delivery claim; `D3-CURRENT.json` is Root's. Preparation count stays 2.\n\n"
)
demoted = "# HISTORICAL (superseded by the review " + REVIEW[:8] + " of the correction) — " + head[len("# CURRENT — "):]
status_path.write_bytes((new + demoted + status[first_end:]).encode("utf-8"))
print("ok")
