//! TEST-ONLY (EEDCS-49, D3, feature `physical-flags-check`): the characterisation of the per-entity UPDATED / MOVED / ROTATED word against the engine's own mobile getter and setter, the three global flag arrays and
//! the REAL `update_client_server_entities` reset (csrc/eech_physical_flags_check.c). The entity pool is the engine's own `entity` struct array, the update list is empty and nothing else is scripted: no campaign, map,
//! host, random stream or floating-point arithmetic. It pins observed facts of the ORIGINAL code (the getter returns the entity's mask and clears every OTHER entity's bit of the shared word without clearing its own; the
//! reset is a quarter-memset that never clears entity indices from 131072 up; the key is the slot index alone), as compatibility facts and not as a design. The consumers' bodies are not run (docs/d3-physical-flags-contract.md).
//!
//! TWO PROFILES (Task49, the host-owned flag store), the SAME 199 scenarios in both. The ORIGINAL profile (`--features physical-flags-check,collision-fixture`: EECH_CAMPAIGN_HOST_ENTITY is not defined) runs them over the
//! engine's own global arrays, unchanged: this is the accepted characterisation. The HOSTED profile (the default build) runs the identical scenarios over the real getter, setter and reset, whose words are the HOST's: the
//! provider the engine asks is the REAL store of eech-world (`tests/support/flags_host.rs`), a scenario starts from a fresh host binding where the original cleared an array, and a word is observed by asking the host where
//! the original read an array. The entry point differs only by that installation; the expectations are the same text.
use eech_engine_sys as _;
use std::ffi::{c_char, c_int};

#[cfg(not(feature = "collision-fixture"))]
#[path = "support/flags_host.rs"]
mod flags_host;

extern "C" {
    fn eech_physical_flags_check_run(text: *mut c_char, capacity: c_int, checks: *mut c_int) -> c_int;
    /// the HOSTED profile only: installs the provider the engine asks, a fresh-binding entry and the host-word observation (csrc/eech_physical_flags_hosted.h)
    #[cfg(not(feature = "collision-fixture"))]
    fn eech_physical_flags_check_set_host(
        model: eech_engine_sys::eech_model_callback,
        fresh: extern "C" fn(),
        observe: unsafe extern "C" fn(*const c_char, c_int) -> u32,
    );
}

#[test]
fn the_per_entity_updated_moved_rotated_word_is_a_mask_the_getter_destroys_neighbours_of_and_the_reset_only_reaches_a_quarter_of() {
    #[cfg(not(feature = "collision-fixture"))]
    unsafe {
        eech_physical_flags_check_set_host(flags_host::model, flags_host::fresh, flags_host::observe)
    };
    let mut text = vec![0 as c_char; 16384];
    let mut checks: c_int = 0;
    let failures = unsafe { eech_physical_flags_check_run(text.as_mut_ptr(), text.len() as c_int, &mut checks) };
    let report: String = text.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect();
    assert_eq!(failures, 0, "the original flag code disagreed with its expectations:\n{report}");
    println!("physical flags expectations made: {checks}\n{report}");
    // the count observed on the first green run (199: six domain facts, three words of the word cases, the separate-array facts and the reset groups); a group that did not run changes it
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

const EXPECTED_CHECKS: c_int = 199;
