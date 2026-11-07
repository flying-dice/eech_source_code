//! TEST-ONLY (EEDCS-49, D3, features `sector-height-check` + `sector-height-observer`): the control of the sector-height OBSERVER's hooks against the REAL sector code, not of the funnel.
//!
//! The existing real-C control (`eech_sector_height_check_run`) sends the real LINK_CHILD messages of a sector and packs and unpacks its words through the engine's own code. Here the same
//! run is made with the staged observer hooks compiled into those sources (build/patches.rs, SECTOR_HEIGHT_OBSERVER_PATCHES), and the observer's records are read back. It shows that the
//! hooks leave the control's results unchanged (the same expectations, none failing), that what the link hooks recorded obeys the source's own contract (a link never lowers the held word),
//! that the saved and restored raw words are the ones the control used and are ALL stored, and that the observer REFUSES to call this run whole: the control drives the message table
//! directly, with no en_list INSERT, so every fixed or city link has no open record and is counted as unjoined (the join rule, exercised by the real hooks). It does NOT observe the public
//! funnel (no roster, no host, no retail data): that observation is the native run of lua/sector-height-baseline.lua. The observer's tables are private statics of one process, so this
//! file holds ONE test.
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

fn count(line: &str, key: &str) -> u64 {
    field(line, key).parse().unwrap_or_else(|_| panic!("bad {key} in {line}"))
}

#[test]
fn the_observer_records_what_the_real_sector_code_did_and_refuses_an_unjoined_run() {
    let mut text = vec![0 as c_char; 16384];
    let mut checks: c_int = 0;
    let failures = unsafe { eech_sector_height_check_run(text.as_mut_ptr(), text.len() as c_int, &mut checks) };
    assert_eq!(failures, 0, "the hooks changed the control's results:\n{}", text_of(&text));
    assert_eq!(checks, 179, "the control made {checks} expectations, not the 179 of the unhooked run");

    let mut summary = vec![0 as c_char; 16384];
    assert_eq!(unsafe { eech_sho_summary_run(summary.as_mut_ptr(), summary.len() as c_int) }, 0);
    let summary = text_of(&summary);
    let line = |prefix: &str| {
        summary
            .lines()
            .find(|l| l.starts_with(prefix))
            .unwrap_or_else(|| panic!("no {prefix} line in\n{summary}"))
            .to_string()
    };

    // the control never inserts through en_list: every fixed or city link has no open record and the observer says so, instead of guessing one from a family or a sector
    let join = line("sho-join");
    let (fixed, city, mobile) = (count(&join, "link_fixed"), count(&join, "link_city"), count(&join, "link_mobile"));
    assert!(fixed + city > 0, "the control sent no fixed link the observer saw:\n{summary}");
    assert_eq!(count(&join, "links_joined"), 0, "{join}");
    assert_eq!(count(&join, "links_unjoined"), fixed + city, "{join}");
    assert_eq!(count(&join, "link_total"), fixed + city + mobile, "{join}");
    assert!(summary.contains("sho-complete value=0"), "an unjoined run must not be called whole:\n{summary}");
    let tables = line("sho-tables");
    assert_eq!(count(&tables, "events_overflow"), 0, "{tables}");

    // every shape state is explicit: a fixed sender's shape is read at the original's own gate, and nothing is credited invalid without a read
    let shape = line("sho-shape");
    let observed = count(&shape, "fixed_valid") + count(&shape, "fixed_invalid") + count(&shape, "city_valid") + count(&shape, "city_invalid");
    let unobserved = count(&shape, "fixed_unobserved") + count(&shape, "city_unobserved");
    assert_eq!(observed + unobserved, fixed + city, "{shape}");

    let session = line("sho-session");
    assert_eq!((count(&session, "pack_omitted"), count(&session, "unpack_omitted")), (0, 0), "{session}");

    let used = count(&tables, "events_used") as c_int;
    let mut seen_links = 0u64;
    let mut last_seq = 0u64;
    let mut packed = Vec::new();
    let mut unpacked = Vec::new();
    for i in 0..used {
        let mut row = vec![0 as c_char; 4096];
        assert_eq!(unsafe { eech_sho_event_run(i, row.as_mut_ptr(), row.len() as c_int) }, 0, "event {i}");
        let row = text_of(&row);
        let seq: u64 = field(&row, "seq").parse().unwrap_or_else(|_| panic!("bad seq in {row}"));
        assert!(seq > last_seq, "sequence numbers must increase: {row}");
        last_seq = seq;
        match field(&row, "kind").as_str() {
            "link" => {
                seen_links += 1;
                assert_eq!(field(&row, "rec"), "0", "no record was open, so none may be claimed: {row}");
                let (before, after) = (word(&row, "before"), word(&row, "after"));
                // the source never lowers the held word through a link: equal bits, or a strictly larger value
                assert!(
                    before == after || f32::from_bits(after) > f32::from_bits(before),
                    "a link lowered or reshaped the word: {row}"
                );
            }
            "pack" => packed.push(word(&row, "after")),
            "unpack" => unpacked.push(word(&row, "after")),
            _ => {}
        }
    }
    assert_eq!(seen_links, fixed + city, "every fixed or city link is a recorded event");
    assert!(
        !packed.is_empty() && packed.len() == unpacked.len(),
        "the control packed {} and unpacked {} words",
        packed.len(),
        unpacked.len()
    );
    // the control saves every pattern it restores: each restored raw word is one that was packed
    for w in &unpacked {
        assert!(packed.contains(w), "a restored word {w:#010x} was never packed");
    }
    println!(
        "sector height observer: {} links ({} fixed or city, all unjoined by design), {} packed, {} unpacked, {checks} control expectations",
        count(&join, "link_total"),
        fixed + city,
        packed.len(),
        unpacked.len()
    );
}
