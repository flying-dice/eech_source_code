//! TEST-ONLY (EEDCS-49, D3, feature `mq-report-check`): the isolated control of the controlled moving entity query case's report codec and bounds.
//!
//! It runs `eech_mq_report_check_run` (csrc/eech_mq_report_check.c), which formats with the SAME functions the `weapon-fixture` case uses (csrc/eech_mq_report.h): every roster identity of
//! 1 to 128 bytes (whitespace, newlines and UTF-8 included) is encoded reversibly and canonically; a HIT row with a maximum identity is longer than the previous 320-byte local buffer and
//! fits its report; and the worst-case report length, computed from the formatters, decides which buffers the case accepts before it asks its first query. It boots no engine.
use eech_engine_sys as _;
use std::ffi::{c_char, c_int};

extern "C" {
    fn eech_mq_report_check_run(text: *mut c_char, capacity: c_int, checks: *mut c_int) -> c_int;
}

#[test]
fn every_roster_identity_is_reported_exactly_and_the_worst_case_report_is_bounded_before_any_query() {
    let mut text = vec![0 as c_char; 16384];
    let mut checks: c_int = 0;
    let failures = unsafe { eech_mq_report_check_run(text.as_mut_ptr(), text.len() as c_int, &mut checks) };
    let report: String = text.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect();
    assert_eq!(
        failures, 0,
        "the report codec disagreed with its expectations:
{report}"
    );
    println!("moving query report expectations made: {checks}");
    assert!(checks >= 1500, "only {checks} expectations were made: the control did not run in full");
}
