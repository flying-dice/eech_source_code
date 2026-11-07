//! TEST-ONLY (EEDCS-49, D3, feature `sector-height-check`): the characterisation of a sector's tallest-structure height against the engine's own sector code.
//!
//! It runs `eech_sector_height_check_run` (csrc/eech_sector_height_check.c), which sends the real LINK_CHILD and UNLINK_CHILD messages of a sector to the response table the engine
//! installs (sc_msgs.c), reads the value through the engine's own float getter (sc_float.c), and packs and unpacks the sector map with the engine's own session functions (sc_pack.c)
//! over its real pack stream. The senders and the sector map are hand-made structs of the engine's own types, and the model provider the hosted bounding-box accessor serves from is one scripted callback (declared in the control's header); no engine function is replaced. It pins the high-water behaviour (a lower,
//! equal or reinserted structure and a removal never lower it), the float words it can hold, the rounding mode it depends on, which senders can raise it, and the save bytes. It loads
//! no retail data, boots no campaign and says nothing about a host: the source/build-derived census of every writer and reader is tests/sector_height_guard.rs (text patterns, not a preprocessed-symbol or runtime census).
use eech_engine_sys as _;
use std::ffi::{c_char, c_int};

extern "C" {
    fn eech_sector_height_check_run(text: *mut c_char, capacity: c_int, checks: *mut c_int) -> c_int;
}

#[test]
fn the_sector_height_is_a_high_water_word_the_original_code_stores_serves_and_saves_bit_for_bit() {
    let mut text = vec![0 as c_char; 16384];
    let mut checks: c_int = 0;
    let failures = unsafe { eech_sector_height_check_run(text.as_mut_ptr(), text.len() as c_int, &mut checks) };
    let report: String = text.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect();
    assert_eq!(failures, 0, "the original sector code disagreed with its expectations:\n{report}");
    println!("sector height expectations made: {checks}");
    assert!(checks >= 150, "only {checks} expectations were made: the control did not run in full");
}
