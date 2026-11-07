import sys

P = sys.argv[1] + "/eech-campaign/crates/eech-world/tests/entity_authority.rs"
d = open(P, "rb").read().decode("utf-8")


def rep(old, new):
    global d
    assert d.count(old) == 1, (old[:90], d.count(old))
    d = d.replace(old, new)


# ---- the scripted engine: the height read as a function, early (at FIRST, as the campaign's getter) or LATE (after the offer, so that a configured hit precedes it), and the unrelated faults of the controls
rep("""					if scenario.height_scan then
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
					end""", """					HEIGHT_CURSOR = cursor
					if scenario.height_scan and not scenario.height_scan_late then height_read () end""")

rep("""				local answer, standing
				if scenario.scan_from_entry then""", """				-- the campaign's getter asks the host for the sector's height (scan phase 17: the sector in the cursor facts), checks the whole reply as the shared C does (the state, three patterns, the sector echoed) and carries
				-- the word into every candidate question of the scan as fact 12. A SCRIPTED boundary check of the driver, not a native campaign read. `height_stray_request` makes the question itself malformed (a stray fact)
				local function height_read ()
					local cursor = HEIGHT_CURSOR
					local hw = {}
					for index = 1, 16 do hw [index] = ZERO end
					hw [12], hw [13], hw [14] = "0x41880000", cursor [1], cursor [2]
					if scenario.height_stray_request then hw [6] = "0x3f800000" end
					local hok, hanswer = ask_model ("entity_scan", 0, nil, nil, nil, table.concat (hw, ","), not height_altering [MODE])
					if not hok then stop ("entity_scan.callback", tostring (hanswer)) end
					if type (hanswer) == "table" and hanswer.result == "refused" then stop ("entity_scan.refused", tostring (hanswer.reason)) end
					if type (hanswer) ~= "table" or hanswer.result ~= "height" then stop (scenario.height_wrong_marker and "entity_scan.cursor" or "entity_scan.state", "a sector height read allows only height") end
					local hp = {}
					for token in string.gmatch (hanswer.p or "", "[^,]+") do hp [#hp + 1] = token end
					if #hp ~= 3 then stop ("entity_scan.answer", "p must be three bit patterns") end
					if hp [1] ~= cursor [1] or hp [2] ~= cursor [2] then stop ("entity_scan.sector_height", "another sector than the one asked about") end
					CAND_FACT12 = hp [3]
				end
				local answer, standing
				if scenario.scan_from_entry then""")

rep("""				last_hit = false
				offer ()
				answer = ask ("resolved", false, last_hit, standing)""", """				last_hit = false
				offer ()
				-- LATE: the read comes after the offer, so that the configured narrow-phase hit has been answered before it (the order of the control that injects an unrelated fatal between the two)
				if scenario.height_scan and scenario.height_scan_late then
					if scenario.inject_fatal_before_height then stop (scenario.inject_fatal_before_height, "an unrelated fatal of the scripted engine") end
					height_read ()
				end
				answer = ask ("resolved", false, last_hit, standing)""")

marker = "#[test]\nfn withdrawing_the_fixed_state_of_a_served_placement_makes_the_next_question_of_the_same_candidate_refused_by_name() {"
assert d.count(marker) == 1
new = '''// ---- ENV-SECTOR-HEIGHT-REFUSAL-INTERVENTION-1: a height refusal is credited only with EXACTLY the one actual eligible height intervention, whatever the configured secondary input altered and whatever marker stopped the
// engine; any fatal before it is a FAILURE, only a clean run with no eligible read is UNREACHED

const HEIGHT_MODES: [(&str, &str); 5] = [
    ("sector_height_wrong_word", "entity_candidate.refused"),
    ("sector_height_withheld", "entity_scan.callback"),
    ("sector_height_malformed", "entity_scan.answer"),
    ("sector_height_wrong_state", "entity_scan.state"),
    ("sector_height_wrong_sector", "entity_scan.sector_height"),
];

#[test]
fn an_unrelated_fatal_bearing_a_height_modes_own_marker_before_any_eligible_intervention_is_a_failure_never_a_credit_or_unreached() {
    for (mode, marker) in HEIGHT_MODES {
        // the coherent positive first: the read after the offer (the configured hit already answered) IS the eligible intervention and credits the mode
        let positive = drive(&healthy(&format!("{FIXED_STATE}{HEIGHT_SCAN}, height_scan_late = true")), &[("mode", mode)]);
        assert!(positive.result.is_ok(), "{mode}: the late positive is credited: {:?}\\n{}", positive.result, positive.rows);
        let row = authority(&positive);
        assert_eq!((row["height_altered"].clone(), row["failure_marker"].clone()), (Value::from(1), Value::from(marker)), "{mode}: {row}");
        // the single fault: AFTER the valid configured secondary hit and BEFORE any eligible read, an unrelated fatal that bears this mode's expected marker
        let injected = drive(&healthy(&format!("{FIXED_STATE}{HEIGHT_SCAN}, height_scan_late = true, inject_fatal_before_height = '{marker}'")), &[("mode", mode)]);
        let row = authority(&injected);
        assert_eq!(row["height_altered"], 0, "{mode}: {row}");
        assert!(row["altered"].as_i64().unwrap() >= 1, "{mode}: the configured secondary hit was answered before the fatal: {row}");
        assert_eq!(row["failure_marker"], marker, "{mode}: {row}");
        let error = error_of(injected);
        assert!(error.contains("before an eligible sector height read was altered"), "{mode}: {error}");
        assert!(!error.contains("UNREACHED"), "{mode}: {error}");
    }
}

#[test]
fn a_height_refusal_run_that_fails_for_another_reason_or_stops_at_another_marker_after_the_intervention_is_not_credited() {
    // a clean run with no eligible read is UNREACHED (the scripted engine never reads)
    for (mode, _) in HEIGHT_MODES {
        let error = error_of(drive(&healthy(FIXED_STATE), &[("mode", mode)]));
        assert!(error.contains(&format!("UNREACHED: {mode}: no ELIGIBLE sector height read")), "{mode}: {error}");
    }
    // a request fault at the read (a stray fact) means no read was attributed: the run failed first, never UNREACHED and never credited
    let error = error_of(drive(&healthy(&format!("{FIXED_STATE}{HEIGHT_SCAN}, height_stray_request = true")), &[("mode", "sector_height_withheld")]));
    assert!(error.contains("no sector height answer was altered because the run failed first"), "{error}");
    // the ACTUAL intervention, then the engine stops at another marker than the mode's: a failure naming both
    let error = error_of(drive(&healthy(&format!("{FIXED_STATE}{HEIGHT_SCAN}, height_wrong_marker = true")), &[("mode", "sector_height_wrong_state")]));
    assert!(error.contains("stopped at [model-provider:entity_scan.cursor], not at [model-provider:entity_scan.state]"), "{error}");
}

'''
d = d.replace(marker, new + marker)
open(P, "wb").write(d.encode("utf-8"))
print("ok")
