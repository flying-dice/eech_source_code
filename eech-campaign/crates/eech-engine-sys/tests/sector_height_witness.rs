//! TEST-ONLY (EEDCS-49, D3, Task49; feature `sector-height-witness`): the REAL C refusals of the session-save fixture method (`csrc/eech_hw_witness.c`), run in a process that has booted nothing.
//!
//! What this shows: every refusal the method makes BEFORE it would call `store_session` is made by the real C code, names its reason, and touches no file (the working directory is listed before and after, and a
//! refused call never creates one). What it cannot show, and does not claim: that `store_session` saves a live campaign, that `pack_session` packs the host's words or that a saved script loads. With no booted campaign
//! there is no current game session, so the method stops at that refusal; the save itself is the native run of `lua/sector-height-witness.lua` (docs/d3-sector-height-production-contract.md section 12).
use eech_engine_sys as _;
use std::ffi::{c_char, c_int, CStr, CString};

extern "C" {
    fn eech_hw_session_save_run(name: *const c_char, text: *mut c_char, capacity: c_int) -> c_int;
    /// the ONE shared check of the complete saved paths against the original writer's 256-byte buffers; the wrapper calls it before store_session or any file access
    #[allow(clippy::too_many_arguments)]
    fn eech_hw_save_paths(
        data_path: *const c_char,
        data_path_capacity: usize,
        campaign_directory: *const c_char,
        directory_capacity: usize,
        name: *const c_char,
        extension: *const c_char,
        data_out: *mut c_char,
        script_out: *mut c_char,
        reason: *mut *const c_char,
    ) -> c_int;
}

const WRITER_CAPACITY: usize = 256;

/// the shared path check with the session's own array capacities (256 and 64); Ok((data path, script path)) or Err(reason)
fn paths(data_path: &str, campaign_directory: &str, name: &str, extension: &str) -> Result<(String, String), String> {
    let data = {
        let mut bytes = data_path.as_bytes().to_vec();
        bytes.resize(256, 0);
        bytes
    };
    let directory = {
        let mut bytes = campaign_directory.as_bytes().to_vec();
        bytes.resize(64, 0);
        bytes
    };
    paths_in(&data, &directory, name, extension)
}

fn paths_in(data: &[u8], directory: &[u8], name: &str, extension: &str) -> Result<(String, String), String> {
    let name = CString::new(name).unwrap();
    let extension = CString::new(extension).unwrap();
    let mut data_out = vec![0x55 as c_char; WRITER_CAPACITY];
    let mut script_out = vec![0x55 as c_char; WRITER_CAPACITY];
    let mut reason: *const c_char = std::ptr::null();
    let code = unsafe {
        eech_hw_save_paths(
            data.as_ptr() as *const c_char,
            data.len(),
            directory.as_ptr() as *const c_char,
            directory.len(),
            name.as_ptr(),
            extension.as_ptr(),
            data_out.as_mut_ptr(),
            script_out.as_mut_ptr(),
            &mut reason,
        )
    };
    let text = |buffer: &[c_char]| unsafe { CStr::from_ptr(buffer.as_ptr()) }.to_string_lossy().into_owned();
    if code == 0 {
        Ok((text(&data_out), text(&script_out)))
    } else {
        // a refusal leaves BOTH outputs empty, whichever path failed
        assert_eq!(
            (text(&data_out), text(&script_out)),
            (String::new(), String::new()),
            "a refused check writes nothing"
        );
        Err(unsafe { CStr::from_ptr(reason) }.to_string_lossy().into_owned())
    }
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

#[test]
fn the_shared_path_check_accepts_the_longest_fitting_writer_paths_and_refuses_the_first_that_does_not_fit_before_any_effect() {
    let before = listing();
    let name = "n".repeat(24);
    // 215 + separator + 10 + separator + 24 + the 4-byte suffix = 255 bytes plus the NUL: the longest pair the original writer's 256-byte buffers hold
    let data_path = "d".repeat(215);
    let directory = "c".repeat(10);
    let (data, script) = paths(&data_path, &directory, &name, ".Sch").expect("the boundary path fits");
    assert_eq!((data.len(), script.len()), (255, 255));
    assert!(
        data.ends_with(&format!("\\{name}.sav")) && script.ends_with(&format!("\\{name}.Sch")),
        "{data} {script}"
    );
    assert!(data.starts_with(&data_path) && data.contains(&format!("\\{directory}\\")), "{data}");
    // one byte more: the first path that does not fit, refused for the DATA path
    let reason = paths(&"d".repeat(216), &directory, &name, ".Sch").expect_err("256 bytes do not fit with the NUL");
    assert!(reason.contains("saved data path"), "{reason}");
    // the data path fits and the SCRIPT path (a longer suffix variant) does not: refused for the script, with the fitting data path NOT written either
    let reason = paths(&data_path, &directory, &name, ".Schx").expect_err("the script path is one byte too long");
    assert!(reason.contains("saved script path"), "{reason}");
    // the declared domains (data_path[256], campaign_directory[64]) reach 255 + 63 + the name: far beyond 256, all refused
    for (a, b) in [(255, 63), (255, 1), (230, 40), (240, 10)] {
        paths(&"d".repeat(a), &"c".repeat(b), &name, ".Sch").expect_err(&format!("{a}+{b} bytes of directories cannot fit"));
    }
    // short and minimal names and the ordinary Windows form fit
    let (data, script) = paths("..\\common\\maps\\map5", "camp01", "alpha", ".Sch").expect("an ordinary path fits");
    assert_eq!(
        (data.as_str(), script.as_str()),
        ("..\\common\\maps\\map5\\camp01\\alpha.sav", "..\\common\\maps\\map5\\camp01\\alpha.Sch")
    );
    paths(&data_path, &directory, "n", ".Sch").expect("a one character name fits");
    // an array that is not terminated within its own capacity cannot be copied and is refused (never read past it)
    let reason = paths_in(&[b'a'; 8], &[0u8; 8], "alpha", ".Sch").expect_err("an unterminated data path");
    assert!(reason.contains("not terminated"), "{reason}");
    let reason = paths_in(&[0u8; 8], &[b'c'; 8], "alpha", ".Sch").expect_err("an unterminated directory");
    assert!(reason.contains("not terminated"), "{reason}");
    assert_eq!(listing(), before, "the path check touches no file");
}
