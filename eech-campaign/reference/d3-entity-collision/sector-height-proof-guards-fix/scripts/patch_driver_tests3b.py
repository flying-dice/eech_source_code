import sys

P = sys.argv[1] + "/eech-campaign/crates/eech-world/tests/entity_authority.rs"
d = open(P, "rb").read().decode("utf-8")


def rep(old, new):
    global d
    assert d.count(old) == 1, (old[:90], d.count(old))
    d = d.replace(old, new)


rep("""				if scenario.height_scan and scenario.height_scan_late then
					if scenario.inject_fatal_before_height then stop (scenario.inject_fatal_before_height, "an unrelated fatal of the scripted engine") end
					height_read ()
				end""", """				-- an unrelated fatal of the scripted engine AFTER the configured narrow-phase hit was answered, whether or not a height read follows
				if scenario.inject_fatal_after_offer then stop (scenario.inject_fatal_after_offer, "an unrelated fatal of the scripted engine") end
				if scenario.height_scan and scenario.height_scan_late then
					if scenario.inject_fatal_before_height then stop (scenario.inject_fatal_before_height, "an unrelated fatal of the scripted engine") end
					height_read ()
				end""")

a = d.index("#[test]\nfn an_unrelated_fatal_bearing_a_height_modes_own_marker_before_any_eligible_intervention_is_a_failure_never_a_credit_or_unreached() {")
b = d.index("#[test]\nfn a_height_refusal_run_that_fails_for_another_reason_or_stops_at_another_marker_after_the_intervention_is_not_credited() {")
d = d[:a] + '''#[test]
fn an_unrelated_fatal_bearing_a_height_modes_own_marker_before_any_eligible_intervention_is_a_failure_never_a_credit_or_unreached() {
    for (mode, marker) in HEIGHT_MODES {
        // the coherent positive first (the read at the scan FIRST is the eligible intervention and credits the mode)
        let positive = drive(&healthy(&format!("{FIXED_STATE}{HEIGHT_SCAN}")), &[("mode", mode)]);
        assert!(positive.result.is_ok(), "{mode}: the positive is credited: {:?}\\n{}", positive.result, positive.rows);
        let row = authority(&positive);
        assert_eq!((row["height_altered"].clone(), row["failure_marker"].clone()), (Value::from(1), Value::from(marker)), "{mode}: {row}");
        // the single fault: AFTER the valid configured secondary hit (the narrow phase was answered a hit) and with NO eligible read, an unrelated fatal that bears this mode's expected marker
        let injected = drive(&healthy(&format!("{FIXED_STATE}, inject_fatal_after_offer = '{marker}'")), &[("mode", mode)]);
        let row = authority(&injected);
        assert_eq!(row["height_altered"], 0, "{mode}: {row}");
        assert!(row["altered"].as_i64().unwrap() >= 1, "{mode}: the configured secondary hit was answered before the fatal: {row}");
        assert_eq!(row["failure_marker"], marker, "{mode}: {row}");
        let error = error_of(injected);
        assert!(error.contains("before an eligible sector height read was altered"), "{mode}: {error}");
        assert!(!error.contains("UNREACHED"), "{mode}: {error}");
    }
    // the same for the four modes that stop at the read itself, with the eligible read STILL TO COME: the late read is the intervention (positive), and a fatal between the hit and the read is a failure
    for (mode, marker) in HEIGHT_MODES.iter().filter(|(mode, _)| *mode != "sector_height_wrong_word") {
        let positive = drive(&healthy(&format!("{FIXED_STATE}{HEIGHT_SCAN}, height_scan_late = true")), &[("mode", mode)]);
        assert!(positive.result.is_ok(), "{mode}: the late positive is credited: {:?}\\n{}", positive.result, positive.rows);
        assert_eq!(authority(&positive)["height_altered"], 1, "{mode}");
        let injected = drive(&healthy(&format!("{FIXED_STATE}{HEIGHT_SCAN}, height_scan_late = true, inject_fatal_before_height = '{marker}'")), &[("mode", mode)]);
        let row = authority(&injected);
        assert_eq!((row["height_altered"].clone(), row["failure_marker"].clone()), (Value::from(0), Value::from(*marker)), "{mode}: {row}");
        let error = error_of(injected);
        assert!(error.contains("before an eligible sector height read was altered") && !error.contains("UNREACHED"), "{mode}: {error}");
    }
}

''' + d[b:]
open(P, "wb").write(d.encode("utf-8"))
print("ok")
