"""Metadata-only: STATUS top summary and the active card for the proof-guards correction. Textual replacement (no JSON round trip). Usage: patch.py <repo> <code-commit>"""
import json
import pathlib
import sys

REPO = pathlib.Path(sys.argv[1])
CODE = sys.argv[2]
card_path = REPO / "docs/handover/ACTIVE-EXTRACTION.json"
status_path = REPO / "STATUS.md"
card = card_path.read_bytes().decode("utf-8")
status = status_path.read_bytes().decode("utf-8")
PK = "reference/d3-entity-collision/sector-height-proof-guards-fix"
FREEZE = "R:/d3-evidence/D3-sector-height-PROOF-GUARDS-FIX-CODE-FROZEN-20261107.json"
PROD = "0be1dd8465d9bf38da69cd3b303206cf5a64574d"
PREV = "5aae2256f02ca3d79e762d71c9b79c20dc34856b"
REVIEW = "409a2622b5314d82c02dc7710dcf3528f8390678e5b11162108847bcb83aa27b"
freeze = json.loads(pathlib.Path(FREEZE).read_text(encoding="utf-8"))
PRE = freeze["input_maps"]["pre_utc"]
POST = freeze["input_maps"]["post_utc"]
final = {r["label"]: r for r in freeze["runs"] if r["final"]}
driver_tests = final["v3-driver-tests"]["tests_passed"]
callback_tests = final["v4-host-callbacks"]["tests_passed"]
assert not freeze["hygiene_problems_at_freeze"], freeze["hygiene_problems_at_freeze"]


def j(t):
    return json.dumps(t, ensure_ascii=True)


def line_span(text, prefix):
    assert text.count(prefix) == 1, prefix
    s = text.index(prefix)
    return s, text.index("\n", s)


def replace_line(text, prefix, value):
    s, e = line_span(text, prefix)
    return text[:s] + prefix + j(value) + "," + text[e:]


def edit_row(text, ident, note):
    at = text.index('"id": "%s"' % ident)
    ev = '"evidence": "'
    e = text.index(ev, at)
    assert e - at < 400, ident
    return text[:e + len(ev)] + j(note)[1:-1] + " HISTORICAL TEXT OF THE ROW: " + text[e + len(ev):]


def edit_field(text, ident, field, value):
    at = text.index('"id": "%s"' % ident)
    key = '"%s": ' % field
    s = text.index(key, at)
    assert s - at < 3500, (ident, field)
    e = text.index("\n", s)
    trailing = "," if text[e - 1] == "," else ""
    return text[:s] + key + j(value) + trailing + text[e:]


card = edit_row(card, "ENV-SECTOR-HEIGHT-SCOPED-REPORT-COMPLETE-1",
                "CORRECTED in source " + CODE + " (UNREVIEWED; native UNRUN; the finding stays OPEN until the SAME reviewer): sector-height-baseline.lua validates BOTH sides as a whole scoped scan before comparing (the accepted original: one identity/sho-scan/sho-tables and the terminal sho-verdict whole=1 with the sector count, last; both: exact fields and domains, unique contiguous row indices, unique sector keys, declared count, totals and words consistent with the rows; the hosted side is not asked for sho-complete, events, raise or session); INVALID_ACCEPTED / INVALID_HOSTED are never EQUAL_AT_SCOPE, SCAN_ONLY or scan_whole=1; UNREACHED only for a clean whole scan with no read. Positive-first single-fault controls: 29 accepted and 17 live hosted faults; the real retained accepted report (2 sectors, 63 reads, 27615 offers) is VALID by an independent port of the same rules. Pack " + PK + ".")
card = edit_field(card, "ENV-SECTOR-HEIGHT-SCOPED-REPORT-COMPLETE-1", "next_fix",
                  "Corrected in " + CODE[:9] + " (UNREVIEWED, native UNRUN); closes only by the SAME reviewer's changed-material review. ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1 stays partially addressed and OPEN until the SAME reviewer closes all.")
card = edit_row(card, "ENV-SECTOR-HEIGHT-REFUSAL-INTERVENTION-1",
                "CORRECTED in source " + CODE + " (UNREVIEWED; native UNRUN; the finding stays OPEN until the SAME reviewer): entity-authority.lua credits a height refusal only with EXACTLY the one actual eligible height intervention (height_altered=1), independent of the secondary-input counters and of marker equality; any fatal before it is a FAILURE (the mode's own marker included); only a clean run with no eligible read is UNREACHED; wrong_word keeps its same-entry fact-12 consumption and exact candidate refusal. Per mode: the positive, then an unrelated fatal bearing the mode's marker injected after a valid configured hit and before any eligible read must FAIL with height_altered=0 (RED on the previous verdict, g7). Pack " + PK + ".")
card = edit_field(card, "ENV-SECTOR-HEIGHT-REFUSAL-INTERVENTION-1", "next_fix",
                  "Corrected in " + CODE[:9] + " (UNREVIEWED, native UNRUN); closes only by the SAME reviewer's changed-material review. M5 (ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1), M6 (ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1), sector-height-host-move-open-facts and all ten D3 requirements stay required.")
card = edit_row(card, "ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1",
                "PARTIALLY ADDRESSED and STAYS OPEN until the SAME reviewer closes all (" + CODE[:9] + ", UNREVIEWED, native UNRUN): the two false-pass diagnostics of the SAME review " + REVIEW[:8] + " (ENV-SECTOR-HEIGHT-SCOPED-REPORT-COMPLETE-1, ENV-SECTOR-HEIGHT-REFUSAL-INTERVENTION-1) are corrected in the diagnostic Lua and driver tests only.")

phase = ("REQUIRED sector-height production extraction, substage 3 corrected AGAIN: the SAME review (" + REVIEW[:8] + ") of correction " + PREV[:9] + " found TWO P2 false-pass diagnostics (ENV-SECTOR-HEIGHT-SCOPED-REPORT-COMPLETE-1, ENV-SECTOR-HEIGHT-REFUSAL-INTERVENTION-1; no new production defect, native clearance false): both are corrected in " + CODE[:9] + " (UNREVIEWED, native UNRUN, findings OPEN until the SAME reviewer): "
         "both sides of the scoped comparison validated as a whole scan; a height refusal credited only with exactly the one eligible intervention. ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1 stays partially addressed and OPEN. M5/M6 witnesses, sector-height-host-move-open-facts and all ten D3 requirements stay required. P1 stays CLOSED at 4de. Prep count stays 2.")
card = replace_line(card, '  "phase": ', phase)
nxt = ("Root: ONE SAME changed-material review of " + CODE[:9] + " (sibling pack " + PK + ") of the two diagnostics (scoped whole-scan validation of both sides; refusal credit only with exactly one eligible intervention), with the affected checks, the PRE/POST maps and the explicit joins to " + PREV[:9] + " and the frozen production source. If cleared: fresh exact-source/profile gates, the observer-off ordinary short at the accepted identity, then section 11 of docs/d3-sector-height-production-contract.md: "
       "A (hosted observer build, tools/sector-height-baseline-windows.ps1 -Scope hosted-scan -Accepted <the f01 report>, window lebanon_retail hours 0.001 frame 100 seed 1) and B (entity-authority.lua sector_height_control, then the five refusal modes, one fresh root each); UNREACHED/NOT_COMPARABLE/INVALID_*/DIFFERENT/FAILED stay distinct, nothing searched or extended. "
       "ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1 and ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1 stay required OPEN (their fixture entries do not exist). No self-approval or native clearance by the author. All10, remaining hints, whole UPDATED/shared terrain, raw TREE/FOREST/CLOUD RNG, hierarchy/services, DCS API workarounds, natural moving, final current 40/all8+4 inputs, CI, review, normal merge and final actual census remain. No D4/D5/TypeScript.")
card = replace_line(card, '  "next_action": ', nxt)

item = ("SECTOR-HEIGHT PROOF-GUARDS CORRECTION (UNREVIEWED; native UNRUN; ENV-SECTOR-HEIGHT-SCOPED-REPORT-COMPLETE-1 and ENV-SECTOR-HEIGHT-REFUSAL-INTERVENTION-1 stay OPEN until the SAME reviewer; ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1 partially addressed, OPEN): code " + CODE + " over checkpoint fe1bf6723 (previous correction " + PREV + "; production source " + PROD + " untouched). "
        "Inputs read in full: the SAME review " + REVIEW + " (CHANGES_REQUIRED, native_launch_clearance=false, no new production defect; findings under decision.findings), the exact 3f CI read-back (all nine jobs success; it arrived after the reviewer's seal, so the review's 8/9 observation stays historical). "
        "Correction (diagnostic Lua + driver tests + prose; no production C/protocol/kernel): (1) sector-height-baseline.lua validates BOTH sides as a WHOLE scoped scan (terminal sho-verdict whole=1 of the accepted original, unique contiguous indices and sector keys, declared count, exact fields/domains, totals and words consistent) before comparing; the hosted scope stays narrower; statuses INVALID_ACCEPTED/INVALID_HOSTED added; 29 accepted + 17 live hosted single-fault controls from a coherent positive (RED g1 on the previous comparator); the real accepted report is VALID by an independent port. "
        "(2) entity-authority.lua credits a height refusal only with exactly one eligible intervention (height_altered=1) independent of the secondary counters and marker equality; any earlier fatal is FAILURE; clean absence UNREACHED; per-mode positives and injected unrelated fatals (RED g4/g7 on the previous verdict). "
        "Checks between the PRE (" + PRE + ") and POST (" + POST + ") maps (equal): cargo fmt --check, strict clippy of eech-world, entity_authority + sector_height_baseline_driver (" + str(driver_tests) + " passed), host_callbacks (" + str(callback_tests) + " passed), the real-report shape check; the 0be 16/49 matrix and 5aae 11/19 affected runs are reused only through the explicit input joins. Retained failures/REDs in the packet. Pack " + PK + "; external freeze " + FREEZE + ". "
        "Guards: production-cutover 0, fix-blocker for the three findings and height-open-facts 0, deliver unmet. Preparation count stays 2. No self-review, approval, native clearance, finding closure or whole Task49/D3/delivery claim.")
marker = '\n  ],\n  "blockers": ['
assert card.count(marker) == 1
card = card.replace(marker, ',\n    ' + j(item) + marker)
json.loads(card)

first_end = status.index("\n")
head = status[:first_end]
assert head.startswith("# CURRENT \u2014 D3 Task49: SAME review CHANGES_REQUIRED (one P2) of the Host-owned sector height"), head
new = (
    "# CURRENT \u2014 D3 Task49: SAME review CHANGES_REQUIRED (two P2 false-pass diagnostics) of the public proof plan `" + PREV[:9] + "`; both CORRECTED in `" + CODE[:9] + "` (UNREVIEWED, native UNRUN) (7 November 2026)\n\n"
    "The SAME reviewer (`" + REVIEW[:8] + "`) found **no new production defect** and TWO P2 test-diagnostic false passes, registered OPEN at `fe1bf6723`: `ENV-SECTOR-HEIGHT-SCOPED-REPORT-COMPLETE-1` (the scoped comparison could call a truncated, duplicated or incomplete scan EQUAL_AT_SCOPE) and `ENV-SECTOR-HEIGHT-REFUSAL-INTERVENTION-1` (an unrelated fatal bearing a height mode's expected marker, after a configured hit and before any eligible read, could be credited). "
    "Corrected here, **test-only** (diagnostic Lua, driver tests, prose; no production C, protocol or kernel change): the baseline comparison validates BOTH sides as a whole scoped scan (terminal verdict, unique contiguous rows and sector keys, declared count, consistent totals/words, exact fields) and reports INVALID_ACCEPTED / INVALID_HOSTED instead of a pass; "
    "a height refusal is credited only with exactly the one eligible intervention, any earlier fatal is a failure, and a clean absence is UNREACHED. Each has positive-first single-fault controls with a retained RED on the previous behaviour; the real retained accepted report (2 sectors, 63 reads, 27615 offers) is valid under the same rules by an independent port. "
    "Exact `3f` CI: all nine jobs success (arrived after the reviewer's seal; the review's 8/9 observation stays historical). Packet [" + PK + "](eech-campaign/" + PK + "/README.md); external freeze `" + FREEZE + "`.\n\n"
    "Checks: fmt, strict World clippy, both driver targets and host_callbacks between timed, equal PRE/POST maps; the unchanged `0be1dd846` and `5aae2256f` matrices are reused only through explicit input joins. **Still OPEN:** both findings and `ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1` until the SAME reviewer; `sector-height-host-move-open-facts`; "
    "`ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1` and `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1`. No Windows or native launch here; no self-approval, native clearance, finding closure or whole Task49/D3 claim. P1 `ENV-ENTITY-POSE-LONG-RUN-MOTION-1` stays CLOSED. Preparation count stays 2. "
    "All ten D3 requirements and the remaining hints, whole UPDATED/shared terrain, raw TREE/FOREST/CLOUD RNG, hierarchy/services, DCS API workarounds, natural moving, the final current 40/all-8/final-4-inputs, CI, review, normal merge and final actual census remain. No D4/D5/TypeScript. Jira49/PR109 draft stay open.\n\n"
)
demoted = "# HISTORICAL (superseded by the proof-guards correction " + CODE[:9] + ") \u2014 " + head[len("# CURRENT \u2014 "):]
status = new + demoted + status[first_end:]
card_path.write_bytes(card.encode("utf-8"))
status_path.write_bytes(status.encode("utf-8"))
print("ok")
