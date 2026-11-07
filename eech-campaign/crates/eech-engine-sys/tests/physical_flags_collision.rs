//! TEST-ONLY (EEDCS-49, D3, Task49; feature `physical-flags-collision-check`): the second REAL immediate-consumer control of the per-entity UPDATED word, the moving-mobile PASS-THROUGH of the collision query
//! (`get_line_of_sight_collision_entity`, en_cllsn.c), through the real public function over hand-made entities of the engine's own types (csrc/eech_physical_flags_collision_check.c declares every hand-made collaborator).
//! It is a different consumer from the guide control (`physical_flags_consumer.rs`).
//!
//! TWO PROFILES, the SAME world and the SAME segments. The ORIGINAL profile (`--features physical-flags-collision-check,collision-fixture`) compiles the pass-through body (`if (get_local_entity_int_value (target, INT_TYPE_UPDATED))`) and
//! reads the engine's own array. The HOSTED profile (the default build) compiles `campaign_scan_entities` instead: the decision is the host's (the roster, the scan kernel, the candidate kernel that asks for the UPDATED word once, and the
//! narrow phase), the campaign reads the word through the host's store (`campaign_offer_candidate`, mb_hostoffer.h) and states 2.0 or 1.0 to the host.
//!
//! In the HOSTED profile each mobile is first linked to the campaign's own roster group and the host issues its identity and starts its mover record (the private test collaborator `bind_member`); the real roster placement, scan, candidate, narrow-phase and flag
//! questions are the real provider's, the same 15 scenarios hold and a 16th is the matched negative (a mobile with no host identity is refused by name).
//!
//! BOTH profiles are served by the REAL reference provider (`host.model`, its roster, flag store and model store over the minimal installation) through a TEST BRIDGE (`tests/support/lua_provider.rs`) that is NOT the shipped `Engine::boot`
//! bridge: this is NOT a host-owned Lua -> native EECH_DC module -> host run and says nothing about that path.
#![cfg(feature = "physical-flags-collision-check")]
use eech_engine_sys as _;
use std::ffi::{c_char, c_int};

#[path = "support/lua_provider.rs"]
mod lua_provider;

extern "C" {
    fn eech_physical_flags_collision_run(text: *mut c_char, capacity: c_int, checks: *mut c_int) -> c_int;
    fn eech_physical_flags_collision_set_host(
        model: eech_engine_sys::eech_model_callback,
        fresh: extern "C" fn(),
        observe: unsafe extern "C" fn(*const c_char, c_int) -> u32,
        bind: unsafe extern "C" fn(*const lua_provider::MemberDescriptor, *mut c_char, usize, *mut c_char, usize, *mut std::ffi::c_void) -> c_int,
    );
}

#[test]
fn the_moving_mobile_pass_through_of_the_collision_query_reads_the_raw_updated_word_destructively_and_swaps_the_path_it_tests() {
    lua_provider::install();
    unsafe { eech_physical_flags_collision_set_host(lua_provider::model, lua_provider::fresh, lua_provider::observe, lua_provider::bind_member) };
    let mut text = vec![0 as c_char; 16384];
    let mut checks: c_int = 0;
    let failures = unsafe { eech_physical_flags_collision_run(text.as_mut_ptr(), text.len() as c_int, &mut checks) };
    let report: String = text.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect();
    println!("physical flags collision expectations made: {checks}\n{report}");
    println!(
        "questions asked of the provider (scan, candidate, pose, collision, flag, other): {:?}; candidate decisions: {:?}",
        lua_provider::asked(),
        lua_provider::log()
    );
    assert_eq!(failures, 0, "the collision consumer disagreed with its expectations:\n{report}");
    if cfg!(feature = "collision-fixture") {
        // the ORIGINAL profile: the pass-through body ran against the real scene data of the real provider (catalogue, scenes, no triangle)
        // the count observed on the first green run (15: the unset flag and its two segments, the set flag and its two, the second read, both list orders and the destruction of the neighbour's bit)
        assert_eq!(checks, 15, "the original-profile control made {checks} expectations, not the 15 of its groups");
        assert!(
            report.contains("physical-flags-collision: consumer=get_line_of_sight_collision_entity"),
            "{report}"
        );
    } else {
        // the HOSTED profile: the SAME scenarios, decided by the host. The mobiles are bound through the campaign's own roster (the host issues each an identity and starts its mover record), the real scan, candidate and narrow-phase
        // questions are answered by the real provider, the candidate kernel asks for the UPDATED word and the flag store answers it; a mobile with no host identity is refused by name (the matched negative).
        // the count observed on the first green run (16: the original profile's 15 scenarios and the matched negative of a mobile with no host identity)
        assert_eq!(checks, 16, "the hosted control made {checks} expectations, not the 16 of its groups");
        let asked = lua_provider::asked();
        assert!(
            asked[0] > 0 && asked[1] > 0 && asked[2] > 0 && asked[3] > 0 && asked[4] > 0,
            "an operation never reached the host (scan, candidate, pose, collision, flag): {asked:?}"
        );
        let log = lua_provider::log();
        assert!(
            log.iter().any(|line| line == "need_updated") && log.iter().any(|line| line == "test"),
            "the host never asked for the UPDATED word or never decided a narrow-phase test: {log:?}"
        );
        assert!(
            report.contains("physical-flags-collision: consumer=get_line_of_sight_collision_entity"),
            "{report}"
        );
    }
}
