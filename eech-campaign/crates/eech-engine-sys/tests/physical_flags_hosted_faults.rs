//! TEST-ONLY (EEDCS-49, D3, Task49; feature `physical-flags-check`, HOSTED profile only): the NEGATIVE controls of the hosted flag control. The engine's own reply check (mb_hostentity.h) refuses a MALFORMED answer by stopping the
//! engine (checked in `entity_dialogue`); this checks the other half: a host whose answers are well-formed but WRONG, which no reply check can see, makes the SAME 199 scenarios of the real getter, setter and reset FAIL. Each fault
//! is one change of the real store's behaviour (`tests/support/flags_host.rs`): a read that always answers the lane's mask, a read that always answers 0, a set that is acknowledged but never stored, and a read that
//! destroys no neighbour. The matched positive runs first and last with the real store (0 failures, 199 expectations), so each failure is the one changed behaviour and not the setup.
#![cfg(not(feature = "collision-fixture"))]
use eech_engine_sys as _;
use std::ffi::{c_char, c_int};
use std::sync::atomic::Ordering;

#[path = "support/flags_host.rs"]
mod flags_host;

extern "C" {
    fn eech_physical_flags_check_run(text: *mut c_char, capacity: c_int, checks: *mut c_int) -> c_int;
    fn eech_physical_flags_check_set_host(
        model: eech_engine_sys::eech_model_callback,
        fresh: extern "C" fn(),
        observe: unsafe extern "C" fn(*const c_char, c_int) -> u32,
    );
}

fn run() -> (c_int, c_int, String) {
    let mut text = vec![0 as c_char; 65536];
    let mut checks: c_int = 0;
    let failures = unsafe { eech_physical_flags_check_run(text.as_mut_ptr(), text.len() as c_int, &mut checks) };
    (failures, checks, text.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect())
}

#[test]
fn the_hosted_control_fails_for_a_host_that_does_not_do_what_the_original_arrays_did_and_passes_for_the_real_store() {
    unsafe { eech_physical_flags_check_set_host(flags_host::model, flags_host::fresh, flags_host::observe) };
    flags_host::FAULT.store(flags_host::NO_FAULT, Ordering::SeqCst);
    let (failures, checks, report) = run();
    assert_eq!((failures, checks), (0, 199), "the real store failed the control:\n{report}");
    for (fault, name) in [
        (flags_host::READ_ALWAYS_MASK, "a read that always answers the lane's mask"),
        (flags_host::READ_ALWAYS_ZERO, "a read that always answers 0"),
        (flags_host::SET_IGNORED, "a set that is acknowledged and never stored"),
        (
            flags_host::READ_NON_DESTRUCTIVE,
            "a read that answers the right raw word but destroys no neighbour",
        ),
    ] {
        flags_host::FAULT.store(fault, Ordering::SeqCst);
        let (failures, checks, report) = run();
        println!("{name}: {failures} of {checks} expectations failed");
        assert_eq!(checks, 199, "{name}: the control did not make all its expectations");
        assert!(failures > 0, "{name}: the control did not notice the wrong host");
        assert!(report.contains("FAIL"), "{name}: no failure was reported:\n{report}");
    }
    // the control is repeatable and the real store passes it again
    flags_host::FAULT.store(flags_host::NO_FAULT, Ordering::SeqCst);
    let (failures, checks, report) = run();
    assert_eq!((failures, checks), (0, 199), "the real store failed the control after the faults:\n{report}");
}
