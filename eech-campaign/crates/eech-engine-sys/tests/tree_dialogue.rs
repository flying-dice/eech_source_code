//! TEST-ONLY (EEDCS-49, feature `tree-dialogue-check`): the isolated control of the campaign's half of the host-owned tree segment query.
//!
//! It runs `eech_tree_dialogue_check_run` (csrc/eech_tree_dialogue_check.c), which binds the REAL dialogue the shipping module includes (aphavoc/.../mb_hosttree.h) to a scripted
//! host and a recording terrain call. It checks the lookups the campaign answers (none, OLD only, NEW only, OLD then NEW), their exact coordinates and patterns, what is sent to
//! the host, what the caller's buffers hold afterwards (a signalling NaN and a signed zero survive both ways) and every stop: an end twice, NEW then OLD, a third question, a
//! non-finite point, an unknown state, a hit that is not a boolean, a nested query. It loads no terrain and no model and says nothing about the host's tree kernel.
use eech_engine_sys as _;
use std::ffi::{c_char, c_int};

extern "C" {
    fn eech_tree_dialogue_check_run(text: *mut c_char, capacity: c_int, checks: *mut c_int) -> c_int;
}

#[test]
fn the_campaign_answers_every_valid_endpoint_subsequence_and_stops_on_every_other_one() {
    let mut text = vec![0 as c_char; 8192];
    let mut checks: c_int = 0;
    let failures = unsafe { eech_tree_dialogue_check_run(text.as_mut_ptr(), text.len() as c_int, &mut checks) };
    let report: String = text.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect();
    assert_eq!(failures, 0, "the dialogue disagreed with its expectations:\n{report}");
    assert!(checks >= 60, "only {checks} expectations were made: the control did not run in full");
}
