//! TEST-ONLY (EEDCS-49, D3, feature `physical-flags-check`): the characterisation of the per-entity UPDATED / MOVED / ROTATED word against the engine's own mobile getter and setter, the three global flag arrays and
//! the REAL `update_client_server_entities` reset (csrc/eech_physical_flags_check.c). The entity pool is the engine's own `entity` struct array, the update list is empty and nothing else is scripted: no campaign, map,
//! host, random stream or floating-point arithmetic. It pins observed facts of the ORIGINAL code (the getter returns the entity's mask and clears every OTHER entity's bit of the shared word without clearing its own; the
//! reset is a quarter-memset that never clears entity indices from 131072 up; the key is the slot index alone), as compatibility facts and not as a design. The consumers' bodies are not run (docs/d3-physical-flags-contract.md).
use eech_engine_sys as _;
use std::ffi::{c_char, c_int};

extern "C" {
    fn eech_physical_flags_check_run(text: *mut c_char, capacity: c_int, checks: *mut c_int) -> c_int;
}

#[test]
fn the_per_entity_updated_moved_rotated_word_is_a_mask_the_getter_destroys_neighbours_of_and_the_reset_only_reaches_a_quarter_of() {
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
}

const EXPECTED_CHECKS: c_int = 199;
