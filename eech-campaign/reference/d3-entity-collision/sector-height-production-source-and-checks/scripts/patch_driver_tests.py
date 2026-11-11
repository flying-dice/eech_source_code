import sys

P = sys.argv[1] + "/eech-campaign/crates/eech-world/tests/entity_authority.rs"
d = open(P, "rb").read().decode("utf-8")


def rep(old, new):
    global d
    assert d.count(old) == 1, (old[:80], d.count(old))
    d = d.replace(old, new)


# the scripted engine's sector height events: a READ and a RESTORE of a sector, as the campaign's getter and unpack ask them
rep("""			-- the city's own state at its insertion with the original's INVERTED initial box""", """			-- the sector's HIGH-WATER HEIGHT (phases 17 and 18, Task49): the campaign's getter asks the host for a sector's word by its coordinates (x in fact 12, z in fact 13), and a session unpack states each saved word RAW in fact 0.
			-- The scripted engine reads sector (3, 4) (the fresh zero word of a sector no statement raised), restores a quiet NaN with a payload and reads it back; each answer is checked as the shared C checks it (the state, no name, the sector
			-- echoed, the word exactly as stated). The host's answer comes back unaltered; a refusal is the bridge's own fatal with the host's reason
			local function height_event (kind, word)
				local words = {}
				for index = 1, 16 do words [index] = ZERO end
				words [12], words [13], words [14] = kind == "restore" and "0x41900000" or "0x41880000", "0x40400000", "0x40800000"
				if kind == "restore" then words [1] = word end
				local fault = scenario.bad_height_request
				local identity = ""
				if fault == "stray_fact" then words [6] = "0x3f800000"
				elseif fault == "identity" then identity = "1" end
				local ok, answer = ask_model ("entity_scan", 0, nil, nil, nil, table.concat (words, ",") .. (identity ~= "" and (" " .. identity) or ""), true)
				if not ok then stop ("entity_scan.callback", tostring (answer)) end
				if type (answer) == "table" and answer.result == "refused" then stop ("entity_scan.refused", tostring (answer.reason)) end
				if type (answer) ~= "table" or answer.result ~= (kind == "restore" and "restored" or "height") then stop ("entity_scan.state", "a sector height allows only height and restored") end
				local expected = kind == "restore" and word or answer.p and string.match (answer.p, "[^,]+$")
				if string.match (answer.p, "[^,]+$") ~= expected then stop ("entity_scan.sector_height", "the word the host answered is not the one it was to hold") end
				return answer
			end
			local function sector_height_events ()
				local first = height_event ("read")
				if string.match (first.p, "[^,]+$") ~= ZERO then stop ("entity_scan.sector_height", "a sector no arrival raised does not hold the zero word") end
				height_event ("restore", "0x7fc0f00d")
				local again = height_event ("read")
				if string.match (again.p, "[^,]+$") ~= "0x7fc0f00d" then stop ("entity_scan.sector_height", "the word restored is not the word read back") end
			end
			-- the city's own state at its insertion with the original's INVERTED initial box""")

rep("""				if scenario.fixed_state then fixed_report (roster_key) end
				if scenario.city_state then city_state_report (roster_key) end
				-- a SECOND live placement""", """				if scenario.fixed_state then fixed_report (roster_key) end
				if scenario.city_state then city_state_report (roster_key) end
				if scenario.sector_height then sector_height_events () end
				-- a SECOND live placement""")

# the tests, after the healthy fixed-state control
marker = "#[test]\nfn withdrawing_the_fixed_state_of_a_served_placement_makes_the_next_question_of_the_same_candidate_refused_by_name() {"
assert d.count(marker) == 1
new = '''#[test]
fn a_sector_height_read_and_restore_cross_the_public_driver_to_the_real_roster_and_are_counted_and_checked_with_nothing_altered() {
    let (driven, row) = scan_row(&format!("{FIXED_STATE}, sector_height = true"), "scan_real");
    assert!(driven.result.is_ok(), "{:?}\\n{}", driven.result, driven.rows);
    for (field, expected) in [
        ("request_faults", 0),
        ("reply_faults", 0),
        // two reads and one restore, each answered well-formed in the engine's own window (the boot walk), and none of them is a roster, city, fixed or scan question
        ("sector_height_total", 2),
        ("sector_height_answered", 2),
        ("sector_restore_total", 1),
        ("sector_restore_answered", 1),
        ("fixed_total", 1),
        ("roster_after", 2),
        ("altered", 0),
    ] {
        assert_eq!(row[field], expected, "{field}: {row}");
    }
}

#[test]
fn each_single_fault_of_the_sector_height_grammar_is_counted_at_its_question_and_is_never_a_pass() {
    for (fault, expected) in [
        ("stray_fact", "states nothing but the sector, its phase and, for a restore, the word"),
        ("identity", "a sector height names no placement"),
    ] {
        let (driven, row) = scan_row(&format!("{FIXED_STATE}, sector_height = true, bad_height_request = '{fault}'"), "scan_real");
        assert!(driven.result.is_err(), "{fault}: {:?}", driven.result);
        assert!(row["request_faults"].as_i64().unwrap() >= 1, "{fault}: {row}");
        let first = row["first_fault"].as_str().unwrap();
        assert!(first.starts_with("scan request:") && first.contains(expected), "{fault}: {first}");
    }
}

'''
d = d.replace(marker, new + marker)
open(P, "wb").write(d.encode("utf-8"))
print("ok")
