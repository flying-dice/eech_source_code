"""Metadata-only: STATUS top summary and the active card for the M5/M6 witness source batch. Textual replacement (no JSON round trip); the card keeps its CRLF line endings. Usage: patch.py <repo> <code-commit>"""
import json
import pathlib
import sys

REPO = pathlib.Path(sys.argv[1])
CODE = sys.argv[2]
card_path = REPO / "docs/handover/ACTIVE-EXTRACTION.json"
status_path = REPO / "STATUS.md"
card = card_path.read_bytes().decode("utf-8")
status = status_path.read_bytes().decode("utf-8")
PK = "reference/d3-entity-collision/sector-height-native-witness-source"
FREEZE = "R:/d3-evidence/D3-SECTOR-HEIGHT-NATIVE-WITNESSES-CODE-FROZEN-20261107.json"
NATIVE = "1b89c0b0ab613d2f6a08ea02dd6612457efa8982"
freeze = json.loads(pathlib.Path(FREEZE).read_text(encoding="utf-8"))
assert not freeze["hygiene_problems_at_freeze"], freeze["hygiene_problems_at_freeze"]
PRE, POST = freeze["input_maps"]["pre_utc"], freeze["input_maps"]["post_utc"]
counts = freeze["test_counts_final"]
eol = "\r\n" if "\r\n" in card else "\n"
assert card.count("\n") == card.count(eol), "mixed line endings in the card"


def j(t):
    return json.dumps(t, ensure_ascii=True)


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
    assert s - at < 9000, (ident, field)
    e = text.index(eol, s)
    trailing = "," if text[e - 1] == "," else ""
    return text[:s] + key + j(value) + trailing + text[e:]


def replace_line(text, prefix, value):
    assert text.count(prefix) == 1, prefix
    s = text.index(prefix)
    e = text.index(eol, s)
    return text[:s] + prefix + j(value) + "," + text[e:]


M5 = "ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1"
M6 = "ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1"
card = edit_row(card, M5, "SOURCE FROZEN in " + CODE + " (UNREVIEWED; native UNRUN; the row stays OPEN until actual native proof): lua/sector-height-witness.lua modes clearance_original / clearance_control / clearance_changed_word / four refusal modes observe the REAL guarded fw_move.c branch at the host's own boundary (the phase 17 answers outside any scan and save; the hand-off min_height in host.move_vehicle's fixed_wing_move carrying fixed_wing_terrain after fixed_wing_lookahead) at the declared accepted regression-pack window (lebanon_retail, 3 simulated hours, 100 ms, seed 1); no C hook, no created leader, no tuned window; the original-solver profile is characterised first and the hosted control must equal it field for field; a changed host word must reach the hand-off exactly; refusals credited only after the one eligible intervention at their marker; an absent window is UNREACHED. Contract section 12.1; pack " + PK + ".")
card = edit_field(card, M5, "next_fix", "Source frozen in " + CODE[:9] + " (UNREVIEWED); Root: the SAME source review, then the native runs of contract section 12.1 on fresh roots (original-solver build, then the ordinary hosted build); the row closes only on actual native proof reviewed by the SAME reviewer.")
card = edit_row(card, M6, "SOURCE FROZEN in " + CODE + " (UNREVIEWED; native UNRUN; the row stays OPEN until actual native proof): feature eech-dc/sector-height-witness adds ONE fixture method, session:sector_height_witness_save, that makes the campaign's OWN store_session run (csrc/eech_hw_witness.c: refuses a bad name, a non-server or non-campaign session and any existing saved file; never overwrites, rotates or deletes); the load is the campaign's own later boot of the saved script (parsgen.c FILE_TAG_SAVED_CAMPAIGN, unpack_session), observed at the host's restore transaction by lua/sector-height-witness.lua modes save / save_withheld / load / load_withheld / load_wrong_sector / load_wrong_state / load_stale_word: raw negative, infinite and quiet-NaN words restored bit for bit for matched sectors with the ACK, before the first insertion, none lowered, each fault stopping the campaign's boot at its marker after the one intervention. Seeds are the host's declared inputs; the driver restores nothing for the campaign. Contract section 12.2; pack " + PK + ".")
card = edit_field(card, M6, "next_fix", "Source frozen in " + CODE[:9] + " (UNREVIEWED); Root: the SAME source review, then the witness build (tools/build-windows-sector-height-witness.sh) and the native save and load runs of contract section 12.2; the row closes only on actual native proof reviewed by the SAME reviewer.")

phase = ("Finite height gates VERIFIED at the true " + NATIVE[:9] + " identity (SAME review 8edddd93: findings empty; CI repair CLOSED at the exact 635 CI; M5, M6, the whole height extraction, Task49, D3 and delivery NOT accepted). "
         "The two REQUIRED witnesses are frozen as test-only source " + CODE[:9] + " (UNREVIEWED, native UNRUN): the guarded fixed-wing clearance witness (ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1) and the native server-session save and load witness (ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1), both OPEN until actual native proof; "
         "sector-height-host-move-open-facts and all ten D3 requirements remain OPEN; prep2.")
card = replace_line(card, '  "phase": ', phase)
nxt = ("Root: ONE SAME changed-material source review of " + CODE[:9] + " (pack " + PK + "): the M5 driver modes and the M6 fixture method with their controls and honest limits. If cleared: fresh exact-source Windows/profile gates (the original-solver build, the ordinary hosted build and the witness build), then the native runs of contract section 12 on fresh roots, "
       "one mode each in the stated order (UNREACHED is not a pass; the 1b89 binaries and actuals are never relabelled). No D4/D5/TypeScript, no hints/RNG/hierarchy. All10, final current 40/all8+4 inputs, CI, review, normal merge and census remain.")
card = replace_line(card, '  "next_action": ', nxt)

item = ("SECTOR-HEIGHT M5/M6 WITNESS SOURCE (UNREVIEWED; native UNRUN; ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1 and ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1 stay OPEN): code " + CODE + " over checkpoint 00823df40 (the 1b89 native source " + NATIVE + " untouched and never relabelled). "
        "M5: no created leader and no C hook: the campaign's guarded branch is observed at the host's boundary at the declared accepted window; the driver derives eligibility from the campaign's own call structure and credits only the one eligible intervention (original-solver profile first, hosted control equal to it, changed word reaching the hand-off, four refusals at their markers). "
        "M6: one feature-gated fixture method makes the campaign's own store_session run; the load is its own boot of the saved script, observed at the host's restore transaction (raw words bit for bit with the ACK, before any insertion, none lowered, four single faults). "
        "Checks between the PRE (" + PRE + ") and POST (" + POST + ") maps (equal): fmt, strict clippy of the workspace and of the witness features, the 15 scripted driver tests with the existing driver controls, the real-C refusals of the save method and the source guards, the accepted real-engine controls in both profiles unchanged, engine and dc suites with and without the feature (final test counts "
        + ", ".join("%s=%s" % (k.split("-")[0], v) for k, v in counts.items()) + "); all exit 0. The explicit join shows from the Git diff that no C source of the campaign changed and the compiled additions are feature-gated. "
        "Limits: native UNRUN (store_session, pack_session, unpack_session and the guarded branch have not run here), the scripted collaborator checks the driver only, M5 is a natural reach at a declared window (UNREACHED if absent), the leased image needed rustfmt/clippy installed into ephemeral containers. Pack " + PK + "; external freeze " + FREEZE + ". Guards: production-cutover 0, fix-blocker for both witnesses and height-open-facts 0, deliver unmet. Preparation count stays 2. No self-review, approval, native clearance or whole Task49/D3/delivery claim.")
marker = eol + "  ]," + eol + '  "blockers": ['
assert card.count(marker) == 1
card = card.replace(marker, "," + eol + "    " + j(item) + marker)
json.loads(card)

first_end = status.index("\n")
head = status[:first_end]
assert head.startswith("# CURRENT \u2014 D3 Task49: finite native height gates VERIFIED at the true `1b89` identity and the CI Lua repair CLOSED"), head
new = (
    "# CURRENT \u2014 D3 Task49: the two REQUIRED witnesses (M5 guarded clearance, M6 native save and load) FROZEN as test-only source `" + CODE[:9] + "` (UNREVIEWED, native UNRUN); finite native height gates stay VERIFIED at `" + NATIVE[:9] + "` (7 November 2026)\n\n"
    "The finite native gates of `" + NATIVE[:9] + "` are verified and the CI Lua repair is closed at the exact `635` CI (SAME review `8edddd93`, findings empty; literal records in [reference/d3-entity-collision/sector-height-finite-native-ci-reviewed](eech-campaign/reference/d3-entity-collision/sector-height-finite-native-ci-reviewed/README.md)). "
    "That review accepted neither M5, M6, the height extraction nor Task49/D3; this batch completes the SOURCE of the two required witnesses and nothing else. **M5**: the guarded `fw_move.c` branch is observed at the host's own boundary (the sector-word answers outside any scan, the `min_height` hand-off in `fixed_wing_move`) at the declared accepted regression-pack window, "
    "with the original-solver profile characterised first, the hosted control required to equal it field for field, a changed host word reaching the hand-off exactly, and four refusals credited only after the one eligible intervention; no leader was created and no C hook added, because a created leader would override campaign decisions. "
    "**M6**: one feature-gated fixture method makes the campaign's OWN `store_session` run (never overwriting, rotating or deleting); the load is the campaign's own boot of the saved script, observed at the host's restore transaction (raw negative, infinite and quiet-NaN words bit for bit with the ACK, before any insertion, none lowered, four single faults). "
    "Contract section 12; packet [" + PK + "](eech-campaign/" + PK + "/README.md); external freeze `" + FREEZE + "`.\n\n"
    "Checks: fmt, strict clippy (workspace and the witness features), 15 scripted driver tests, the real-C refusals and source guards, the accepted real-engine controls in both profiles unchanged and the engine/dc suites with and without the feature, between timed equal PRE/POST maps; the explicit join shows from the Git diff that no C source of the campaign changed and the compiled additions are feature-gated. "
    "**Still OPEN:** `ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1` and `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1` until actual native proof (native UNRUN: `store_session`, `pack_session`, `unpack_session` and the guarded branch have not run here; M5 is UNREACHED if no eligible leader meets the branch in the declared window), "
    "`sector-height-host-move-open-facts` and all ten D3 requirements. No Windows or native launch, hints, RNG, hierarchy, D4/D5 or TypeScript; no self-approval or claim of Task49/D3/delivery. The `1b89` binaries and actuals are never relabelled. `D3-CURRENT.json` is Root's and was not edited. Preparation count stays 2.\n\n"
)
demoted = "# HISTORICAL (superseded by the witness source freeze " + CODE[:9] + ") \u2014 " + head[len("# CURRENT \u2014 "):]
status = new + demoted + status[first_end:]
card_path.write_bytes(card.encode("utf-8"))
status_path.write_bytes(status.encode("utf-8"))
print("ok")
