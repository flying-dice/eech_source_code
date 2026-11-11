import sys

P = sys.argv[1] + "/eech-campaign/crates/eech-world/tests/entity_authority.rs"
d = open(P, "rb").read().decode("utf-8")


def rep(old, new):
    global d
    assert d.count(old) == 1, (old[:90], d.count(old))
    d = d.replace(old, new)


# the scripted engine's sector height read: the campaign's getter at a scan FIRST (boundary check, not native authority)
rep("""					words [13], words [14] = cursor [1], cursor [2]
					-- the entry the host served: the scan stands at it until the host serves the next one
					standing = answer.name""", """					words [13], words [14] = cursor [1], cursor [2]
					-- the entry the host served: the scan stands at it until the host serves the next one
					standing = answer.name
					-- the campaign's getter asks the host for the sector's height at a FIRST (scan phase 17: the sector in the cursor facts), checks the whole reply as the shared C does (the state, three patterns,
					-- the sector echoed) and carries the word into every candidate question of the scan as fact 12. A SCRIPTED boundary check of the driver: it is not a native campaign read
					if scenario.height_scan then
						local hw = {}
						for index = 1, 16 do hw [index] = ZERO end
						hw [12], hw [13], hw [14] = "0x41880000", cursor [1], cursor [2]
						local hok, hanswer = ask_model ("entity_scan", 0, nil, nil, nil, table.concat (hw, ","), not height_altering [MODE])
						if not hok then stop ("entity_scan.callback", tostring (hanswer)) end
						if type (hanswer) == "table" and hanswer.result == "refused" then stop ("entity_scan.refused", tostring (hanswer.reason)) end
						if type (hanswer) ~= "table" or hanswer.result ~= "height" then stop ("entity_scan.state", "a sector height read allows only height") end
						local hp = {}
						for token in string.gmatch (hanswer.p or "", "[^,]+") do hp [#hp + 1] = token end
						if #hp ~= 3 then stop ("entity_scan.answer", "p must be three bit patterns") end
						if hp [1] ~= cursor [1] or hp [2] ~= cursor [2] then stop ("entity_scan.sector_height", "another sector than the one asked about") end
						CAND_FACT12 = hp [3]
					end""")

rep("""			local function ask_model (op, scene, depth, kind, definition, key, expect_same)""", """			-- the modes that alter a sector height answer: their altered reply is not the host's own table
			local height_altering = { sector_height_wrong_word = true, sector_height_withheld = true, sector_height_malformed = true, sector_height_wrong_state = true, sector_height_wrong_sector = true }
			local function ask_model (op, scene, depth, kind, definition, key, expect_same)""")

rep("""				words [4], words [5], words [6] = NEW_W [1], NEW_W [2], NEW_W [3]
				if mobile then words [7] = "0x3f800000" end
				if mobile then pose_observe (0) end""", """				words [4], words [5], words [6] = NEW_W [1], NEW_W [2], NEW_W [3]
				if CAND_FACT12 then words [13] = CAND_FACT12 end
				if mobile then words [7] = "0x3f800000" end
				if mobile then pose_observe (0) end""")

# the scripted candidate states the provider's own refusal of a fact 12 that is not the host's word (read from the REAL roster)
rep("""		local bad = scenario.bad_real_candidate
		answer = { result = bad == "state" and "perhaps" or state,""", """		if scenario.height_scan and identity ~= "" and cw [11] == ZERO and ROSTER_HEIGHT (identity) ~= nil and ROSTER_HEIGHT (identity) ~= cw [13] then
			answer = { result = "refused", reason = "[candidate.sector_height] the campaign offers " .. cw [13] .. " as the tallest structure of its sector, but the host holds " .. ROSTER_HEIGHT (identity) .. ": the word is the host's and the campaign only passes it on" }
			ANSWERS [#ANSWERS + 1] = answer
			record.answer = answer
			return answer
		end
		local bad = scenario.bad_real_candidate
		answer = { result = bad == "state" and "perhaps" or state,""")

# the real roster probe
rep("""    // what the REAL roster holds of the fixed collision state of the sector placement a key names (a read, never a write)""", """    // the sector height the REAL roster holds for the sector a placement key stands in, as the pattern the wire carries (a read, never a write): the scripted candidate states the provider's refusal of any other fact 12 in its words
    let for_height = roster.clone();
    let height = lua
        .create_function(move |_, key: String| {
            Ok(eech_world::roster::parse_key(&key).and_then(|key| {
                let roster = for_height.borrow();
                roster.sector(key).map(|sector| format!("0x{:08x}", roster.sector_height(sector)))
            }))
        })
        .expect("the roster height probe");
    lua.globals().set("ROSTER_HEIGHT", height).expect("the probe");
    // what the REAL roster holds of the fixed collision state of the sector placement a key names (a read, never a write)""")

# tests
marker = "#[test]\nfn withdrawing_the_fixed_state_of_a_served_placement_makes_the_next_question_of_the_same_candidate_refused_by_name() {"
assert d.count(marker) == 1
new = '''// ---- the SECTOR HEIGHT authority and refusal modes (Task49): the scripted engine is a BOUNDARY CHECK of the driver (it makes the campaign's getter read at a scan FIRST and carries the word as fact 12 itself); the native
// campaign witnesses are the same modes run by Root on a real build. Nothing here is native campaign authority, and no real Core save/load is made.

const HEIGHT_SCAN: &str = ", height_scan = true";

#[test]
fn the_sector_height_control_credits_only_an_eligible_read_whose_word_a_candidate_question_carried_back() {
    let (driven, row) = scan_row(&format!("{FIXED_STATE}{HEIGHT_SCAN}"), "sector_height_control");
    assert!(driven.result.is_ok(), "{:?}\\n{}", driven.result, driven.rows);
    for (field, expected) in [
        ("request_faults", 0),
        ("reply_faults", 0),
        ("sector_height_total", 1),
        ("sector_height_answered", 1),
        ("height_reads", 1),
        ("height_scan_joined", 1),
        ("height_eligible", 1),
        ("height_other_callers", 0),
        ("height_altered", 0),
        ("height_mismatch", 0),
        // no real Core save/load is reached in this window: the restore is reported and never credited
        ("sector_restore_total", 0),
    ] {
        assert_eq!(row[field], expected, "{field}: {row}");
    }
    assert!(row["height_consumed"].as_i64().unwrap() >= 1, "{row}");
}

#[test]
fn a_sector_height_read_is_unreached_and_never_credited_without_an_eligible_consumer() {
    // no read at all: the scripted engine never asks for the sector's word
    let error = error_of(drive(&healthy(FIXED_STATE), &[("mode", "sector_height_wrong_word")]));
    assert!(error.contains("UNREACHED: sector_height_wrong_word: no ELIGIBLE sector height read (0 reads"), "{error}");
    // a read whose served entry is answered by a MOBILE candidate dialogue (no key): the word is not carried back by a question that names the entry
    let error = error_of(drive(&healthy(", real_scan = true, height_scan = true"), &[("mode", "sector_height_control")]));
    assert!(error.contains("UNREACHED: sector_height_control: 1 eligible reads, but no candidate question of a served entry carried the word"), "{error}");
}

#[test]
fn every_altered_sector_height_answer_stops_the_engine_at_its_own_named_marker_and_the_changed_word_is_shown_consumed() {
    for (mode, marker, text) in [
        ("sector_height_wrong_word", "entity_candidate.refused", "[candidate.sector_height]"),
        ("sector_height_withheld", "entity_scan.callback", "withholds its sector height answer"),
        ("sector_height_malformed", "entity_scan.answer", "p must be three bit patterns"),
        ("sector_height_wrong_state", "entity_scan.state", "allows only height"),
        ("sector_height_wrong_sector", "entity_scan.sector_height", "another sector"),
    ] {
        let driven = drive(&healthy(&format!("{FIXED_STATE}{HEIGHT_SCAN}")), &[("mode", mode)]);
        assert!(driven.result.is_ok(), "{mode}: {:?}\\n{}", driven.result, driven.rows);
        let row = authority(&driven);
        assert_eq!(row["completed"], false, "{mode}");
        assert_eq!(row["failure_marker"], marker, "{mode}: {row}");
        assert!(row["failure_line"].as_str().unwrap().contains(text), "{mode}: {row}");
        assert_eq!((row["height_eligible"].clone(), row["height_altered"].clone()), (Value::from(1), Value::from(1)), "{mode}: {row}");
        // the changed word is shown consumed only where it reached a candidate question; the others stop at the read itself
        let consumed = if mode == "sector_height_wrong_word" { 1 } else { 0 };
        assert_eq!(row["height_consumed_altered"], consumed, "{mode}: {row}");
    }
}

'''
d = d.replace(marker, new + marker)
open(P, "wb").write(d.encode("utf-8"))
print("ok")
