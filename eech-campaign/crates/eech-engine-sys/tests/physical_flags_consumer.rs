//! TEST-ONLY (EEDCS-49, D3, feature `physical-flags-consumer-check`): the first REAL IMMEDIATE-CONSUMER control of the per-entity UPDATED word: the ORIGINAL fixed-wing `response_to_check_mobile_reached_guide` (fw_msgs.c 396..518) called through the real
//! message dispatch (`notify_local_entity`, `message_responses [FIXED_WING][CHECK_MOBILE_REACHED_GUIDE]`) over hand-made fixed-wing and guide records of the engine's own types, with the real getters, guide criterion accessors, `get_delta_time` and
//! `get_3d_perp_dist_of_point_from_line` (csrc/eech_physical_flags_consumer_check.c declares every hand-made collaborator and what is not run). It extends, and does not replace, the 199-expectation control of `physical_flags.rs`. No other consumer body
//! (collision pass-through, supply descriptor, Tacview) is run here.
//!
//! TWO PROFILES (Task49, the host-owned flag store), the SAME 56 scenarios in both: the ORIGINAL profile (`--features physical-flags-consumer-check,collision-fixture`) over the engine's own global arrays, unchanged, and the HOSTED
//! profile (the default build) over the real getter, setter and reset whose words are the HOST's, served by the REAL store of eech-world (`tests/support/flags_host.rs`; a scenario starts from a fresh host binding where the original
//! cleared the array, and a word is observed by asking the host). The consumer is the same engine response in both.
use eech_engine_sys as _;
use std::ffi::{c_char, c_int};

#[cfg(not(feature = "collision-fixture"))]
#[path = "support/flags_host.rs"]
mod flags_host;

extern "C" {
    fn eech_physical_flags_consumer_run(text: *mut c_char, capacity: c_int, checks: *mut c_int) -> c_int;
    /// the HOSTED profile only: installs the provider the engine asks, a fresh-binding entry and the host-word observation (csrc/eech_physical_flags_hosted.h)
    #[cfg(not(feature = "collision-fixture"))]
    fn eech_physical_flags_consumer_set_host(
        model: eech_engine_sys::eech_model_callback,
        fresh: extern "C" fn(),
        observe: unsafe extern "C" fn(*const c_char, c_int) -> u32,
    );
}

#[test]
fn the_original_fixed_wing_arrival_response_reads_the_raw_updated_mask_destructively_only_on_its_pass_through_branch() {
    #[cfg(not(feature = "collision-fixture"))]
    unsafe {
        eech_physical_flags_consumer_set_host(flags_host::model, flags_host::fresh, flags_host::observe)
    };
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
    // the HOSTED profile: the host and no campaign copy backed every word. The host was actually ASKED each kind of question (the groups the campaign opened, the sets, the destructive reads, the resets) and was asked nothing else
    // and refused nothing. The counts are REPORTED, NOT pinned: a private call count is a diagnostic of this scenario text and not a contract, so what is asserted is the coverage (every operation reached the host) and the raw outcomes of the expectations
    #[cfg(not(feature = "collision-fixture"))]
    {
        use std::sync::atomic::Ordering::SeqCst;
        let traffic = (
            flags_host::GROUPS.load(SeqCst),
            flags_host::SETS.load(SeqCst),
            flags_host::READS.load(SeqCst),
            flags_host::RESETS.load(SeqCst),
            flags_host::STORES.load(SeqCst),
        );
        println!("host traffic (groups, sets, reads, resets, fresh bindings): {traffic:?}");
        assert_eq!(
            (flags_host::OTHER.load(SeqCst), flags_host::REFUSED.load(SeqCst)),
            (0, 0),
            "the host was asked a question that is not a flag question, or refused one"
        );
        assert!(
            traffic.0 > 0 && traffic.1 > 0 && traffic.2 > 0 && traffic.3 > 0 && traffic.4 > 0,
            "an operation never reached the host (groups, sets, reads, resets, fresh bindings): {traffic:?}"
        );
    }
}

const EXPECTED_CHECKS: c_int = 56;
