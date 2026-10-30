//! TEST-ONLY (EEDCS-50, feature `model-fixture`): the isolated control of the original-model read-safety walk (csrc/eech_model_walk.h).
//!
//! It runs `eech_model_walkcheck_run` (csrc/eech_model_walkcheck.c), which builds small synthetic arrays in its own static memory and checks the walk's verdict against the
//! reads the original accessor's integer path would make: backed, missing, short, uninitialised, stale (below the region), misaligned and wrapping backing; the end call's read
//! of the next surface; the 256-point surface; the line face; faces of 0 and 1 points stepped exactly as the original steps them (the depth they add, the depths they leave to the next face, the unbacked reads they lead to); the step budget. It calls NO original code and loads no model, so it proves only the verifier's
//! own bounds logic, never a property of any retail model. The fixture's use of it against the real loader's reports is exercised only by the native fixture run.
use eech_engine_sys as _;
use std::ffi::{c_char, c_int};

extern "C" {
    fn eech_model_walkcheck_run(text: *mut c_char, capacity: c_int, checks: *mut c_int) -> c_int;
}

#[test]
fn the_walk_verdicts_match_the_reads_of_the_original_accessor() {
    let mut text = vec![0 as c_char; 8192];
    let mut checks: c_int = 0;
    let failures = unsafe { eech_model_walkcheck_run(text.as_mut_ptr(), text.len() as c_int, &mut checks) };
    let report: String = text.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect();
    assert_eq!(failures, 0, "the walk disagreed with its expectations:\n{report}");
    assert!(checks >= 44, "only {checks} expectations were made: the control did not run in full");
}
