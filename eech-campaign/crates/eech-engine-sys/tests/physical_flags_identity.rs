//! TEST-ONLY (EEDCS-49, D3, Task49; feature `physical-flags-check`, HOSTED profile only): the IDENTITY control of the flag store's keys, ENV-PHYSICAL-FLAGS-KEY-CAPACITY-1 and ENV-PHYSICAL-FLAGS-ISSUED-GROUP-ALIAS-1.
//!
//! `csrc/eech_physical_flags_check.c` (`eech_physical_flags_identity_run`) drives the engine's REAL getter, setter and reset and the campaign's REAL binding (`mb_hostmodel.c`: the key table, the group open, the exchange) against the REAL host store
//! of eech-world, through the engine's own request and reply structs. The provider here (`tests/support/flags_host.rs`) gives one KEY fault at a time (a well-formed answer to a real question whose NAME is changed): the exact 20-byte key (positive, the
//! whole identity slot), a key one byte too long, a long name, `f0`, `f01`, an upper-case `F`, a roster-style key, and a key already bound to another LIVE group. The control catches the engine's own fatal where it happens and asserts that the refusal
//! comes before the cache commit, that every existing binding and host word is intact and that the corrected answer is accepted. It is NOT a lifetime or replacement test and not a Lua-to-module-to-host run.
#![cfg(not(feature = "collision-fixture"))]
use eech_engine_sys as _;
use std::ffi::{c_char, c_int};

#[path = "support/flags_host.rs"]
mod flags_host;

extern "C" {
    fn eech_physical_flags_identity_run(text: *mut c_char, capacity: c_int, checks: *mut c_int) -> c_int;
    fn eech_physical_flags_identity_set_host(
        model: eech_engine_sys::eech_model_callback,
        fresh: extern "C" fn(),
        observe: unsafe extern "C" fn(*const c_char, c_int) -> u32,
        fault: extern "C" fn(c_int),
    );
}

#[test]
fn the_campaigns_key_table_takes_the_whole_20_byte_key_and_refuses_every_other_key_and_every_live_duplicate_before_it_commits() {
    unsafe { eech_physical_flags_identity_set_host(flags_host::model, flags_host::fresh, flags_host::observe, flags_host::set_key_fault) };
    let mut text = vec![0 as c_char; 65536];
    let mut checks: c_int = 0;
    let failures = unsafe { eech_physical_flags_identity_run(text.as_mut_ptr(), text.len() as c_int, &mut checks) };
    let report: String = text.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect();
    println!("physical flags identity expectations made: {checks}\n{report}");
    assert_eq!(failures, 0, "the identity binding disagreed with its expectations:\n{report}");
    // the count observed on the first green run (85: the two independent groups, the exact 20-byte keys, each single key fault with its matched correction, each live duplicate with its matched correction and the reset extent)
    assert_eq!(
        checks, 85,
        "the identity control made {checks} expectations, not the 85 of its groups: a group of cases did not run"
    );
    // no question that is not a flag question was asked and the host refused none (a fault is an ANSWER the engine refuses, never a host refusal)
    use std::sync::atomic::Ordering::SeqCst;
    assert_eq!((flags_host::OTHER.load(SeqCst), flags_host::REFUSED.load(SeqCst)), (0, 0));
    assert!(
        flags_host::GROUPS.load(SeqCst) > 0 && flags_host::SETS.load(SeqCst) > 0 && flags_host::READS.load(SeqCst) > 0 && flags_host::RESETS.load(SeqCst) > 0
    );
}
