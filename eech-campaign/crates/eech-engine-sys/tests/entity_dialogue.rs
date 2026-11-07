//! TEST-ONLY (EEDCS-49, feature `entity-dialogue-check`): the isolated control of the campaign's half of the host-owned entity collision narrow phase.
//!
//! It runs `eech_entity_dialogue_check_run` (csrc/eech_entity_dialogue_check.c), which binds the REAL dialogue the shipping module includes (aphavoc/.../mb_hostentity.h) to a
//! scripted host. The intercept point and normal are OUTPUTS of the original (a real caller passes a normal it never initialised), so the control places them on unreadable pages
//! where the platform allows it, and checks: what is sent (the segment and the pose, never the caller's words), what is written (only the components the host says the original
//! assigned, normal first, with their computed words, including a computed signed zero and NaN payloads), that initialised untouched words keep their exact bits, and that every
//! malformed answer (a mask or a word outside the domain, a hit that has not assigned both vectors, another kind of result, another scene's name, a refusal) stops the engine before
//! a single word of either buffer is written. The same control covers the campaign's half of the host's per-candidate decisions (ENTITY_CANDIDATE): the 44 facts it sends, bit for bit, the
//! seven states and the narrow-phase pose it accepts, the state, pose, identity and scene it refuses by marker, and the campaign's offer over scripted entities that log every getter. It loads no model and says nothing about the host's kernel. It also covers the fixed/static collision backing (Task49): the FIXED_STATE report every insertion and every writer of a fixed entity's collision state makes, and the placement key every staged question of a non-mobile candidate names.
use eech_engine_sys as _;
use std::ffi::{c_char, c_int};

extern "C" {
    fn eech_entity_dialogue_check_run(text: *mut c_char, capacity: c_int, checks: *mut c_int) -> c_int;
}

#[test]
fn the_campaign_reads_no_caller_output_writes_only_what_the_original_assigned_and_commits_nothing_on_a_refusal() {
    let mut text = vec![0 as c_char; 16384];
    let mut checks: c_int = 0;
    let failures = unsafe { eech_entity_dialogue_check_run(text.as_mut_ptr(), text.len() as c_int, &mut checks) };
    let report: String = text.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect();
    assert_eq!(failures, 0, "the dialogue disagreed with its expectations:\n{report}");
    println!("entity dialogue expectations made: {checks}");
    assert!(checks >= 1130, "only {checks} expectations were made: the control did not run in full");
}
