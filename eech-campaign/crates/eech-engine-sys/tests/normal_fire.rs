//! Real Core model/detach control; private environment, no normal-fire/native claim.
#![cfg(feature = "normal-fire-check")]
#![allow(clippy::unwrap_used)]
use eech_engine_sys as _;
use std::ffi::{c_char, c_int, CStr, CString};
extern "C" {
    fn eech_nf_check(directory: *const c_char, output: *mut c_char, capacity: c_int) -> c_int;
}
#[test]
fn real_model_detachment_preserves_device_absence_fallbacks_and_observes_effects() {
    let dir = std::env::temp_dir().join(format!("eech-normal-fire-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let path = CString::new(dir.to_str().unwrap()).unwrap();
    let mut output = vec![0_i8; 32000];
    let result = unsafe { eech_nf_check(path.as_ptr(), output.as_mut_ptr(), output.len() as c_int) };
    let output = unsafe { CStr::from_ptr(output.as_ptr()) }.to_str().unwrap();
    println!("{output}");
    assert_eq!(result, 0, "real collaborator control failed: {output}");
}
