"""Metadata-only: STATUS top summary and the active card for the public-proof-plan correction. Textual replacement (no JSON round trip). Usage: patch.py <repo> <code-commit>"""
import json
import pathlib
import sys

REPO = pathlib.Path(sys.argv[1])
CODE = sys.argv[2]
card_path = REPO / "docs/handover/ACTIVE-EXTRACTION.json"
status_path = REPO / "STATUS.md"
card = card_path.read_bytes().decode("utf-8")
status = status_path.read_bytes().decode("utf-8")
PK = "reference/d3-entity-collision/sector-height-production-public-proof-plan-fix"
FREEZE = "R:/d3-evidence/D3-sector-height-PUBLIC-PROOF-PLAN-FIX-CODE-FROZEN-20261107.json"
PROD = "0be1dd8465d9bf38da69cd3b303206cf5a64574d"


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


card = edit_row(card, "ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1",
                "CORRECTED in source " + CODE + " (UNREVIEWED; native UNRUN; the finding stays OPEN until the SAME reviewer): executable public proof plan = driver modes sector_height_control / _wrong_word / _withheld / _malformed / _wrong_state / _wrong_sector with eligibility, consumption shown by fact 12 and named outcomes (entity-authority.lua), the scoped hosted-scan baseline comparison with the accepted original report at the unchanged window (sector-height-baseline.lua, -Scope/-Accepted in the wrapper), the observer clearance helper capturing no retired word (tallest=uncaptured), contract sections 7-10 corrected and section 11 added. No production solver, protocol or kernel change. Pack " + PK + ".")
card = edit_row(card, "ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1",
                "STAYS OPEN (corrected wording, " + CODE[:9] + "): NO control reaches the guarded fixed-wing execution (a static function of fw_move.c); the hosted-scan report counts the guards and shows the min_height after the original statement but never the consumed word. A controlled witness needs a named fixture entry that does not exist and is not invented.")
card = edit_row(card, "ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1",
                "STAYS OPEN (corrected wording, " + CODE[:9] + "): the real-C scripted-provider pack/unpack is NOT a native Core server save/load, and a Host-only raw restore (phase 18 asked by a test or a driver) is not one either; no mode credits a restore. The provider accepts a well-formed phase 18 with live placements by design (no restore-after-insertion refusal is claimed). Needs a real server-session save and load entry that does not exist.")

phase = ("REQUIRED sector-height production extraction, substage 3 corrected: the SAME combined review (49d7739e) of production " + PROD[:9] + " found ONE P2 (ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1, no new functional defect): the test-only witness and plan are corrected in " + CODE[:9] + " (UNREVIEWED, native UNRUN, finding OPEN until the SAME reviewer): "
         "driver authority/refusal modes, the scoped hosted-scan baseline comparison, the observer clearance helper, sections 7-10 and the executable section 11. M5/M6 witnesses, sector-height-host-move-open-facts and all ten D3 requirements stay required. P1 stays CLOSED at 4de. Prep count stays 2.")
card = replace_line(card, '  "phase": ', phase)
nxt = ("Root: ONE SAME changed-material review of " + CODE[:9] + " (sibling pack " + PK + "): the executable test-only witness/comparison and the truthful plan/prose, with the affected checks, the PRE/POST maps and the explicit join to the frozen production source. If cleared: fresh exact-source/profile gates, the observer-off ordinary short at the accepted identity, then section 11 of docs/d3-sector-height-production-contract.md: "
       "A (hosted observer build, tools/sector-height-baseline-windows.ps1 -Scope hosted-scan -Accepted <the f01 report>, window lebanon_retail hours 0.001 frame 100 seed 1) and B (entity-authority.lua sector_height_control, then the five refusal modes, one fresh root each); UNREACHED/NOT_COMPARABLE/DIFFERENT/FAILED stay distinct, nothing searched or extended. "
       "ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1 and ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1 stay required OPEN (their fixture entries do not exist). No self-approval or native clearance by the author. All10, remaining hints, whole UPDATED/shared terrain, raw TREE/FOREST/CLOUD RNG, hierarchy/services, DCS API workarounds, natural moving, final current 40/all8+4 inputs, CI, review, normal merge and final actual census remain. No D4/D5/TypeScript.")
card = replace_line(card, '  "next_action": ', nxt)

item = ("SECTOR-HEIGHT PUBLIC-PROOF-PLAN CORRECTION (UNREVIEWED; native UNRUN; ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1 stays OPEN until the SAME reviewer): code " + CODE + " over checkpoint 4a228c305 (production source " + PROD + " untouched; the finding was registered OPEN at 4a228c305). "
        "Inputs read in full: the SAME review 49d7739e (CHANGES_REQUIRED, native_launch_clearance=false, no new functional/math/protocol defect), Root's corrected binding v2 ec7015b6 (the initial 7b5cc4a7 retained with its reader error), the exact 1cf5 CI read-back (all nine success). "
        "Correction (test-only, no production solver/protocol/kernel change): (1) entity-authority.lua sector height modes: a phase-17 read is ELIGIBLE only when armed, joined by sector to the real scan FIRST that just named it, the served entry a non-mobile placement of the host's roster and the reply complete; consumption is shown by the fact 12 of a candidate question of that entry; the control is credited only with eligible+consumed+no mismatch; wrong_word/withheld/malformed/wrong_state/wrong_sector credit their named marker; no mode credits a restore; (2) sector-height-baseline.lua scope=hosted-scan (+accepted): ONLY the scan rows (SH10/SH11 still capture the Host-returned FIRST/fact12) and the clearance guard records, compared by sector at the same window with the accepted original report (63 FIRST reads, 27615 offers at true scope), statuses EQUAL_AT_SCOPE/DIFFERENT/NOT_COMPARABLE/UNREACHED, wrapper -Scope/-Accepted (parse-only AST check passes); "
        "(3) the observer clearance helper reads no retired member in a hosted build (tallest=uncaptured; never re-queried), boundary test RED against the old helper; (4) contract sections 7-10 corrected (restore after insertion is NOT a refusal; no control reaches the guarded fw execution; scripted tests are boundary checks; the observer is not globally unusable) and section 11 added with exact commands/features/modes/row schema/eligibility/expected outcomes. "
        "Checks: u1-u11 affected final set between the PRE (08:43:19Z) and POST maps, the explicit join to the frozen production POST map, RED runs retained (n2, n3, n6). Pack " + PK + "; external freeze " + FREEZE + ". M5/M6 witnesses stay OPEN. Guards: production-cutover 0, fix-blocker height-open-facts 0, deliver unmet. Preparation count stays 2. No self-review, approval, native clearance or whole Task49/D3/delivery claim.")
marker = '\n  ],\n  "blockers": ['
assert card.count(marker) == 1
card = card.replace(marker, ',\n    ' + j(item) + marker)
json.loads(card)

head = "# CURRENT \u2014 D3 Task49: the Host-owned sector high-water word BUILT IN SOURCE (`0be1dd846`): UNREVIEWED, native UNRUN, not accepted (7 November 2026)"
assert status.startswith(head), status[:140]
new = (
    "# CURRENT \u2014 D3 Task49: SAME review CHANGES_REQUIRED (one P2) of the Host-owned sector height `0be1dd846`; the public proof plan is CORRECTED in `" + CODE[:9] + "` (UNREVIEWED, native UNRUN) (7 November 2026)\n\n"
    "The SAME combined review (`49d7739e`) of the production source found **no new functional, math or protocol defect** and ONE P2, `ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1` (registered OPEN at `4a228c305`): the promised public comparison and authority/refusal witness was not executable and some control and observer descriptions overstated their scope. "
    "Exact `1cf5` CI: all nine jobs success; Root's corrected binding `v2` is the raw-byte account (the initial `7b5` proof is kept with its reader error). Corrected here, **test-only** (no production solver, protocol or kernel change): "
    "the public driver gains the sector height modes (`sector_height_control` and the five named refusal/changed-word modes) that credit only an ELIGIBLE real consuming request (armed, joined by sector to the scan FIRST that named it, a non-mobile served entry, a complete reply) and show consumption through fact 12; "
    "the baseline driver gains the `hosted-scan` scope, which reuses what the observer still sees of the Host's word (SH10/SH11: the FIRST word and every fact 12) and compares it by sector at the unchanged window with the accepted original report, never the original-link records; "
    "the observer's clearance helper no longer reads the retired member (`tallest=uncaptured`); and contract sections 7 to 10 are corrected (a restore with live placements is accepted by design; no control reaches the guarded fixed-wing execution; scripted tests are boundary checks) with the executable section 11. "
    "Packet [" + PK + "](eech-campaign/" + PK + "/README.md); external freeze `" + FREEZE + "`.\n\n"
    "Checks: the affected final set u1\u2013u11 between timed PRE and POST maps and an explicit join to the frozen production source (the unchanged `0be1dd846` matrices are reused only through it); RED runs retained. **Still OPEN:** the finding until the SAME reviewer; `sector-height-host-move-open-facts`; "
    "`ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1` and `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1` (their fixture entries do not exist and are not invented). No Windows or native launch here; no self-approval, native clearance or whole Task49/D3 claim. "
    "P1 `ENV-ENTITY-POSE-LONG-RUN-MOTION-1` stays CLOSED. Preparation count stays 2. All ten D3 requirements and the remaining hints, whole UPDATED/shared terrain, raw TREE/FOREST/CLOUD RNG, hierarchy/services, DCS API workarounds, natural moving, the final current 40/all-8/final-4-inputs, CI, review, normal merge and final actual census remain. No D4/D5/TypeScript. Jira49/PR109 draft stay open.\n\n"
)
demoted = "# HISTORICAL (superseded by the public-proof-plan correction " + CODE[:9] + ") \u2014 " + status[len("# CURRENT \u2014 "):len(head)]
status = new + demoted + status[len(head):]
card_path.write_bytes(card.encode("utf-8"))
status_path.write_bytes(status.encode("utf-8"))
print("ok")
