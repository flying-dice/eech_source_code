//! TEST-ONLY (EEDCS-49, D3, Task49; feature `sector-height-witness`): the REAL C refusals of the session-save fixture method (`csrc/eech_hw_witness.c`), run in a process that has booted nothing.
//!
//! What this shows: every refusal the method makes BEFORE it would call `store_session` is made by the real C code, names its reason, and touches no file (the working directory is listed before and after, and a
//! refused call never creates one). What it cannot show, and does not claim: that `store_session` saves a live campaign, that `pack_session` packs the host's words or that a saved script loads. With no booted campaign
//! there is no current game session, so the method stops at that refusal; the save itself is the native run of `lua/sector-height-witness.lua` (docs/d3-sector-height-production-contract.md section 12).
use eech_engine_sys as _;
use std::ffi::{c_char, c_int, CString};

extern "C" {
    fn eech_hw_session_save_run(name: *const c_char, text: *mut c_char, capacity: c_int) -> c_int;
}

fn save(name: &str) -> (i32, String) {
    let name = CString::new(name).unwrap();
    let mut text = vec![0 as c_char; 512];
    let code = unsafe { eech_hw_session_save_run(name.as_ptr(), text.as_mut_ptr(), text.len() as c_int) };
    let line: String = text.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect();
    (code, line)
}

fn listing() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(".")
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
}

#[test]
fn the_real_c_refuses_a_bad_name_and_a_missing_session_by_reason_and_creates_no_file() {
    let before = listing();
    for bad in ["", "aaaaaaaaaaaaaaaaaaaaaaaaa", "../x", "a b", "a\\b", "a/b", "a.b", "a\u{e9}"] {
        let (code, line) = save(bad);
        assert_ne!(code, 0, "{bad:?} must be refused");
        assert_eq!(
            line, "sh-witness-refused reason=the name must be 1 to 24 characters of A-Z a-z 0-9 _ -\n",
            "{bad:?}"
        );
    }
    // a good name reaches the session check: nothing is booted, so there is no current game session, and the call stops there
    for good in ["a", "alpha", "Alpha_1-b", "aaaaaaaaaaaaaaaaaaaaaaaa"] {
        let (code, line) = save(good);
        assert_ne!(code, 0, "{good}: no session, no save");
        assert_eq!(line, "sh-witness-refused reason=there is no current game session\n", "{good}");
    }
    assert_eq!(listing(), before, "a refused call creates, renames and deletes nothing");
    // bad buffers fail safely: no write through a null or empty buffer
    let name = CString::new("alpha").unwrap();
    assert_ne!(unsafe { eech_hw_session_save_run(name.as_ptr(), std::ptr::null_mut(), 16) }, 0);
    let mut one = [0 as c_char; 1];
    assert_ne!(unsafe { eech_hw_session_save_run(name.as_ptr(), one.as_mut_ptr(), 0) }, 0);
    assert_ne!(unsafe { eech_hw_session_save_run(std::ptr::null(), one.as_mut_ptr(), 1) }, 0);
}
