//! TEST-ONLY (EEDCS-49, D3, features `sector-height-check` + `sector-height-observer`): the control of the sector-height OBSERVER's recorder, not of the funnel.
//!
//! The existing real-C control (`eech_sector_height_check_run`) sends the real LINK_CHILD messages of a sector and packs and unpacks its words through the engine's own code. Here the same
//! run is made with the staged observer hooks compiled into those sources (build/patches.rs, SECTOR_HEIGHT_OBSERVER_PATCHES), and the observer's records are read back. It shows that the
//! hooks leave the control's results unchanged (the same expectations, none failing), that the recorder is whole, and that what it recorded obeys the source's own contract (a link never
//! lowers the held word; the saved and restored raw words are the ones the control used). It does NOT observe the public funnel (no roster, no host, no retail data): that observation is
//! the native run of lua/sector-height-baseline.lua. The observer's tables are private statics of one process, so this file holds ONE test.
use eech_engine_sys as _;
use std::ffi::{c_char, c_int};

extern "C" {
    fn eech_sector_height_check_run(text: *mut c_char, capacity: c_int, checks: *mut c_int) -> c_int;
    fn eech_sho_summary_run(text: *mut c_char, capacity: c_int) -> c_int;
    fn eech_sho_event_run(index: c_int, text: *mut c_char, capacity: c_int) -> c_int;
}

fn text_of(buffer: &[c_char]) -> String {
    buffer.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect()
}

fn field(line: &str, key: &str) -> String {
    let prefix = format!("{key}=");
    line.split_whitespace()
        .find_map(|w| w.strip_prefix(prefix.as_str()))
        .unwrap_or_else(|| panic!("no {key} in {line}"))
        .to_string()
}

fn word(line: &str, key: &str) -> u32 {
    u32::from_str_radix(field(line, key).trim_start_matches("0x"), 16).unwrap_or_else(|_| panic!("bad {key} in {line}"))
}

#[test]
fn the_observer_records_what_the_real_sector_code_did_and_leaves_its_results_unchanged() {
    let mut text = vec![0 as c_char; 16384];
    let mut checks: c_int = 0;
    let failures = unsafe { eech_sector_height_check_run(text.as_mut_ptr(), text.len() as c_int, &mut checks) };
    assert_eq!(failures, 0, "the hooks changed the control's results:\n{}", text_of(&text));
    assert_eq!(checks, 179, "the control made {checks} expectations, not the 179 of the unhooked run");

    let mut summary = vec![0 as c_char; 16384];
    assert_eq!(unsafe { eech_sho_summary_run(summary.as_mut_ptr(), summary.len() as c_int) }, 0);
    let summary = text_of(&summary);
    assert!(summary.contains("sho-complete value=1"), "the recorder is not whole:\n{summary}");

    let counts = summary.lines().find(|l| l.starts_with("sho-counts")).expect("counts line");
    let links: usize = field(counts, "link_total").parse().expect("link_total");
    assert!(links > 0, "the control sent no LINK_CHILD that the observer saw:\n{summary}");

    let tables = summary.lines().find(|l| l.starts_with("sho-tables")).expect("tables line");
    let used: i32 = field(tables, "events_used").parse().expect("events_used");

    let mut seen_links = 0usize;
    let mut last_seq = 0u64;
    let mut packed = Vec::new();
    let mut unpacked = Vec::new();
    for i in 0..used {
        let mut row = vec![0 as c_char; 4096];
        assert_eq!(unsafe { eech_sho_event_run(i, row.as_mut_ptr(), row.len() as c_int) }, 0, "event {i}");
        let row = text_of(&row);
        let seq: u64 = field(&row, "seq").parse().expect("seq");
        assert!(seq > last_seq, "sequence numbers must increase: {row}");
        last_seq = seq;
        match field(&row, "kind").as_str() {
            "link" => {
                seen_links += 1;
                let (before, after) = (word(&row, "before"), word(&row, "after"));
                let (b, a) = (f32::from_bits(before), f32::from_bits(after));
                // the source never lowers the held word through a link: equal bits, or a strictly larger value
                assert!(before == after || a > b, "a link lowered or reshaped the word: {row}");
            }
            "pack" => packed.push(word(&row, "after")),
            "unpack" => unpacked.push(word(&row, "after")),
            _ => {}
        }
    }
    assert!(seen_links > 0 && seen_links <= links, "{seen_links} link events of {links} links");
    assert!(
        !packed.is_empty() && !unpacked.is_empty(),
        "the control packed {} and unpacked {} words",
        packed.len(),
        unpacked.len()
    );
    // the control saves every pattern it restores: each restored raw word is one that was packed
    for w in &unpacked {
        assert!(packed.contains(w), "a restored word {w:#010x} was never packed");
    }
    println!(
        "sector height observer: {links} links, {seen_links} link events, {} packed, {} unpacked, {checks} control expectations",
        packed.len(),
        unpacked.len()
    );
}
