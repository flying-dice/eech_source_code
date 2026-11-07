//! TEST-ONLY (EEDCS-49, D3, feature `physical-flags-consumer-check`): the first REAL IMMEDIATE-CONSUMER control of the per-entity UPDATED word: the ORIGINAL fixed-wing `response_to_check_mobile_reached_guide` (fw_msgs.c 396..518) called through the real
//! message dispatch (`notify_local_entity`, `message_responses [FIXED_WING][CHECK_MOBILE_REACHED_GUIDE]`) over hand-made fixed-wing and guide records of the engine's own types, with the real getters, guide criterion accessors, `get_delta_time` and
//! `get_3d_perp_dist_of_point_from_line` (csrc/eech_physical_flags_consumer_check.c declares every hand-made collaborator and what is not run). It extends, and does not replace, the 199-expectation control of `physical_flags.rs`. No other consumer body
//! (collision pass-through, supply descriptor, Tacview) is run here.
use eech_engine_sys as _;
use std::ffi::{c_char, c_int};

extern "C" {
    fn eech_physical_flags_consumer_run(text: *mut c_char, capacity: c_int, checks: *mut c_int) -> c_int;
}

#[test]
fn the_original_fixed_wing_arrival_response_reads_the_raw_updated_mask_destructively_only_on_its_pass_through_branch() {
    let mut text = vec![0 as c_char; 16384];
    let mut checks: c_int = 0;
    let failures = unsafe { eech_physical_flags_consumer_run(text.as_mut_ptr(), text.len() as c_int, &mut checks) };
    let report: String = text.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect();
    assert_eq!(failures, 0, "the original consumer disagreed with its expectations:\n{report}");
    println!("physical flags consumer expectations made: {checks}\n{report}");
    // the count observed on the first green run (56: the family/response/getter facts, the positive and single-change cases, every raw mask, the radius and delta-time changes, the three early returns, both call orders, the neighbour destruction, the reset extent low and high, the slot with a fresh record)
    assert_eq!(
        checks, EXPECTED_CHECKS,
        "the control made {checks} expectations, not the {EXPECTED_CHECKS} of its groups: a group of cases did not run"
    );
}

const EXPECTED_CHECKS: c_int = 56;
