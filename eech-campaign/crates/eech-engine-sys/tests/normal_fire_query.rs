//! Dedicated process for one model-provider/capture lifetime. The legacy
//! collision target stays separate: neither test ordering nor global resets
//! may align a fresh Lua counter with the persistent C callback ordinal.
#![cfg(not(any(feature = "collision-fixture", feature = "model-fixture")))]
use eech_engine_sys as sys;
use std::ffi::{c_char, c_int};

#[allow(dead_code)]
#[path = "support/lua_provider.rs"]
mod lua_provider;

extern "C" {
    fn eech_nf_query_join_check(text: *mut c_char, capacity: c_int, checks: *mut c_int) -> c_int;
    fn eech_physical_flags_collision_set_host(
        model: sys::eech_model_callback,
        fresh: extern "C" fn(),
        observe: unsafe extern "C" fn(*const c_char, c_int) -> u32,
        bind: unsafe extern "C" fn(*const lua_provider::MemberDescriptor, *mut c_char, usize, *mut c_char, usize, *mut std::ffi::c_void) -> c_int,
    );
}

#[test]
fn normal_query_joins_the_actual_host_callback_entries_without_claiming_a_normal_birth() {
    lua_provider::install();
    lua_provider::normal_capture_start();
    unsafe { eech_physical_flags_collision_set_host(lua_provider::model, lua_provider::fresh, lua_provider::observe, lua_provider::bind_member) };
    let mut text = vec![0 as c_char; 24000];
    let mut checks = 0;
    let failures = unsafe { eech_nf_query_join_check(text.as_mut_ptr(), text.len() as c_int, &mut checks) };
    let report: String = text.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect();
    println!("new hosted identity expectations={checks}\n{report}");
    assert_eq!(failures, 0, "{report}");
    assert_eq!(checks, 3);
    let events = report
        .lines()
        .filter(|line| line.starts_with("normal-ev "))
        .map(|line| format!("{line}\n"))
        .collect::<String>();
    lua_provider::normal_capture_check(&events);
}
