"""Metadata-only first bounded checkpoint of the M5/M6 witness lane: reconcile the CI finding CLOSED and the finite native gates VERIFIED on the SAME sealed review 8edddd93 and the exact 635 CI, and record that the
required witness implementation starts. Textual edits; the card keeps its CRLF line endings. Usage: patch.py <repo>"""
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
PK = "reference/d3-entity-collision/sector-height-finite-native-ci-reviewed"
REVIEW = "8edddd934b091ea58567d43e3a71f4eaab378431de7b0b378990ae2703e77c32"
CI_PROOF = "8c241395cb9e9dd99770cf6f6c551b52ec8dfe30eddc3982f45a47d14c0f13da"
BINDING = "fa02f29a90c38a1cbb08d78b7b5dcf8d753916f51f3ba392a41a3bfec6d15698"
FEEDBACK = "452b10c8a4d129bc647cab737c01d637e8b7ff9b7e7dfefdff1aa5b7de12670a"
WITNESS_BRANCH_HEAD = "635d4542c1a092febf3cd0269fefbff8aea637f4"


def j(t):
    return json.dumps(t, ensure_ascii=True)


def row_span(text, ident):
    at = text.index('"id": "%s"' % ident)
    return at


def edit_row(text, ident, note):
    at = row_span(text, ident)
    ev = '"evidence": "'
    e = text.index(ev, at)
    assert e - at < 400, ident
    return text[:e + len(ev)] + j(note)[1:-1] + " HISTORICAL TEXT OF THE ROW: " + text[e + len(ev):]


def edit_field(text, ident, field, value):
    at = row_span(text, ident)
    key = '"%s": ' % field
    s = text.index(key, at)
    assert s - at < 5500, (ident, field)
    e = text.index(eol, s)
    trailing = "," if text[e - 1] == "," else ""
    return text[:s] + key + j(value) + trailing + text[e:]


def replace_line(text, prefix, value):
    assert text.count(prefix) == 1, prefix
    s = text.index(prefix)
    e = text.index(eol, s)
    return text[:s] + prefix + j(value) + "," + text[e:]


# 1. the CI finding: CLOSED on the SAME sealed review and the exact 635 CI (Root's lead action)
CI = "ENV-CI-LUA-ARCHIVE-AVAILABILITY-1"
card = edit_row(card, CI, "CLOSED on the SAME independent review " + REVIEW[:8] + " (findings empty; decision FINITE_NATIVE_VERIFIED_AND_CI_REPAIR_CLOSED) and the exact 635d4542c CI (all nine jobs / three workflows success, no manual rerun: the actual lua.org timeout then the MacPorts source with the SAME pin; Lua build and Wine test success). Closed only at this identity; no compiled input differs from 1b89.")
card = edit_field(card, CI, "status", "closed")
at = row_span(card, CI)
nf = card.index('"next_fix": ', at)
ne = card.index(eol, nf)
line = card[nf:ne]
resolution = "CLOSED by the SAME review " + REVIEW + " and the exact 635d4542c CI read-back " + CI_PROOF + ", published literally in " + PK + ". No Native/M5/M6/Task49/D3 acceptance from it."
card = card[:ne] + "," + eol + '      "resolution": ' + j(resolution) + card[ne:]

# 2. M5 and M6: implementation of the required witnesses STARTS (UNREVIEWED source to follow; native UNRUN)
M5 = "ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1"
M6 = "ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1"
card = edit_row(card, M5, "IMPLEMENTATION STARTS at the first bounded checkpoint of the sole witness lane (test-only source to follow, native UNRUN, the row stays OPEN): the intended entry is a feature-gated fixture method of the public session (sector_height_witness_clearance) that runs ONE real basic_fixed_wing_movement of a declared fixed-wing task leader whose guide follows terrain, so the REAL guarded branch of fw_move.c (fixed_wing_movement_waypoint) executes with its real collaborators. Expected proof: both guards reached; the word consumed is the one the host answered for the leader's sector by the ONE getter read; the min_height handed to the host's fixed-wing move equals the original max expressions over (terrain elevation, word minus terrain elevation, waypoint altitude); a changed host word changes that hand-off and the host's physical result while Core policy is unchanged; withheld/malformed/wrong-sector answers fail at their named boundary only after the eligible branch; an absent prerequisite is UNREACHED, never a pass.")
card = edit_row(card, M6, "IMPLEMENTATION STARTS at the first bounded checkpoint of the sole witness lane (test-only source to follow, native UNRUN, the row stays OPEN): the intended entry is the REAL server-session lifecycle: the real store_session (uisession.c, pack_session PACK_MODE_SERVER_SESSION into the real .sav) over a live session whose host answers seeded raw histories, then a second real boot of that saved script (parsgen.c FILE_TAG_SAVED_CAMPAIGN, unpack_session) observed at the host's RESTORE transaction. Expected proof: raw negative/infinite/quiet-NaN words come back bit for bit for matched sectors with the ACK; restore-before-insert and no-lowering at the actual path; withheld, wrong-sector and stale stored-word ACK refused at the genuine boundary; invalid buffers/modes and fixture lifecycle fail safely. Not a host-only phase 18, the real-C scripted provider or a source text; signalling NaN and engine -O2 stay unmeasured.")

# 3. phase and next action
phase = ("Finite height gates VERIFIED at the true 1b89 identity (SAME review " + REVIEW[:8] + ": findings empty; CI repair CLOSED at the exact 635 CI; M5, M6, whole height extraction, Task49, D3 and delivery NOT accepted). "
         "REQUIRED witnesses in progress, test-only (sole lane): ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1 (real guarded fw_move.c branch) and ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1 (real store_session and boot of the saved script); both OPEN, native UNRUN. sector-height-host-move-open-facts and all ten D3 requirements remain OPEN; prep2.")
card = replace_line(card, '  "phase": ', phase)
nxt = ("Sole lane: implement the two required witnesses (test-only fixture entries and drivers reaching the REAL guarded fixed-wing branch and the REAL server-session save and saved-script load), check them sociably on the Linux lease, freeze and STOP for the SAME changed-material review. "
       "Root: after that review clears the source, fresh exact-source Windows/profile gates and native cases (UNREACHED is not a pass); never relabel the 1b89 binaries or actuals. No D4/D5/TypeScript, no hints/RNG/hierarchy. All10, final current 40/all8+4 inputs, CI, review, normal merge and census remain.")
card = replace_line(card, '  "next_action": ', nxt)

# 4. ready evidence
item = ("SECTOR-HEIGHT FINITE NATIVE GATES VERIFIED AND CI LUA REPAIR CLOSED (SAME review " + REVIEW + ", sealed 2026-11-07T10:29:19Z, findings empty): the nine finite native jobs of the immutable 1b89c0b0ab613d2f6a08ea02dd6612457efa8982 terminal 0 over three actual Windows profiles (ordinary/observer/weapon, not an original-solver build): "
        "ordinary and observer shorts complete with three files byte-identical to the accepted 80fd; the hosted M4 scan EQUAL_AT_SCOPE (2 sectors, 63 FIRST reads, 27615 fact-12 offers, differences 0, 0.001 h / 100 ms / seed 1); the healthy eligible height control (45 eligible, 45 joined, 90 consumed, 0 mismatch) and five single eligible changed/refused cases at their exact markers. "
        "The exact 635d4542c CI passed all nine jobs of all three workflows (actual lua.org timeout then the MacPorts source, same pin): ENV-CI-LUA-ARCHIVE-AVAILABILITY-1 CLOSED at that identity only. Literal review, Root feedback, CI read-back and native binding " + BINDING + " in " + PK + ". "
        "NOT accepted by the review: M5 guarded fixed wing, M6 native Core save/load, the whole height extraction, natural engagement, Task49, D3, shipping, delivery. Preparation count stays 2.")
marker = eol + "  ]," + eol + '  "blockers": ['
assert card.count(marker) == 1
card = card.replace(marker, "," + eol + "    " + j(item) + marker)

# 5. the review as a card object (the last key of the card)
tail = eol + "  }" + eol + "}"
stripped = card.rstrip()
assert stripped.endswith(tail), repr(stripped[-60:])
obj = {"review_sha256": REVIEW, "review_bytes": 20359, "sealed_utc": "2026-11-07T10:29:19.818246+00:00", "ci_source": "bad50e4db880761f4ddb4bbbb869302c2ec9d4bc", "ci_metadata": WITNESS_BRANCH_HEAD, "native_snapshot": "1b89c0b0ab613d2f6a08ea02dd6612457efa8982",
       "findings": [], "closed": [CI], "exact_ci_proof_sha256": CI_PROOF, "native_binding_sha256": BINDING, "root_feedback_sha256": FEEDBACK,
       "accepted_false": ["whole_sector_height_extraction", "M5_guarded_fixed_wing", "M6_native_Core_save_load", "whole_Task49", "whole_D3", "full_shipping", "delivery"],
       "scope": "Finite native 1b89 gates and the exact 635 CI only; no clearance of later work."}
block = "," + eol + '  "sector_height_finite_native_review": ' + json.dumps(obj, indent=2, ensure_ascii=True).replace("\n", eol + "  ")
card = stripped[:-len(eol + "}")] + block + eol + "}" + eol
json.loads(card)

# 6. STATUS: ONE CURRENT (the previous summary demoted)
first_end = status.index("\n")
head = status[:first_end]
assert head.startswith("# CURRENT — D3 Task49: CI reliability repair of the windows-cross Lua download"), head
new = (
    "# CURRENT — D3 Task49: finite native height gates VERIFIED at the true `1b89` identity and the CI Lua repair CLOSED (SAME review `" + REVIEW[:8] + "`); the required M5/M6 witnesses are being implemented (7 November 2026)\n\n"
    "The SAME independent reviewer sealed one combined pass (findings empty): the nine finite native jobs of `1b89c0b0a` (three actual Windows profiles, not an original-solver build) are terminal 0 — the ordinary and observer shorts complete and byte-identical to the accepted `80fd`, the hosted M4 scan `EQUAL_AT_SCOPE` "
    "(2 sectors, 63 FIRST reads, 27615 fact-12 offers), a healthy eligible height control and five matched single-fault cases at their exact markers — and the CI repair `bad50e4db`/`635d4542c` is correct: the exact `635` CI passed all nine jobs of all three workflows (the actual `lua.org` timeout then the MacPorts source with the same pin), so "
    "`ENV-CI-LUA-ARCHIVE-AVAILABILITY-1` is **CLOSED at that identity only**. Literal review, Root feedback, CI read-back and native binding: [" + PK + "](eech-campaign/" + PK + "/README.md). **Not accepted by that review:** M5, M6, the whole height extraction, natural engagement, Task49, D3, shipping, delivery.\n\n"
    "**In progress (sole lane, test-only, native UNRUN):** the two REQUIRED witnesses inside the height extraction. `ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1` needs the REAL guarded `fw_move.c` branch reached through its real collaborators and the host word it consumes observed at the host boundary; "
    "`ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1` needs the REAL server-session save (`store_session`/`pack_session`) and a real boot of the saved script (`unpack_session`) observed at the host's RESTORE transaction. Both stay OPEN, as do `sector-height-host-move-open-facts` and all ten D3 requirements; "
    "no hints, RNG, hierarchy, D4/D5 or TypeScript. The `1b89` binaries and actuals are never relabelled. Preparation count stays 2. `D3-CURRENT.json` is Root's and was not edited.\n\n"
)
demoted = "# HISTORICAL (superseded by the SAME finite-native/CI review " + REVIEW[:8] + ") — " + head[len("# CURRENT — "):]
status = new + demoted + status[first_end:]
card_path.write_bytes(card.encode("utf-8"))
status_path.write_bytes(status.encode("utf-8"))
print("ok")
