//! TEST-ONLY (EEDCS-49, D3, feature `sector-height-observer`): the observer's completeness at its table bound.
//!
//! Every pack and unpack word is stored in the one event table (there is no sample cap); the table's bound is the only bound. The core is filled to EXACTLY the bound: the report is
//! still whole, the stored and total counts agree and nothing is omitted. One more word is then offered: it is visibly omitted (total above stored, omitted 1), the event overflow
//! latches, the report is incomplete, and no later record can hide it. The raw words themselves are never altered (a negative, infinite or quiet-NaN word is stored as its bits,
//! never refused or clamped). It uses the recorder core with plain data (no entity heap, no roster, no host); the observer's tables are private statics of one process, so this file holds
//! ONE test.
use eech_engine::{sector_height_row_text, sector_height_summary_text, EngineError, SHO_EVENTS_MAX};
use eech_engine_sys::sho_support as c;

fn summary() -> String {
    match sector_height_summary_text(c::eech_sho_summary_run) {
        Ok(s) => s,
        Err(e) => panic!("the summary converter failed: {e:?}"),
    }
}

fn field(line: &str, key: &str) -> u64 {
    let prefix = format!("{key}=");
    match line.split_whitespace().find_map(|w| w.strip_prefix(prefix.as_str())) {
        Some(v) => match v.parse() {
            Ok(n) => n,
            Err(_) => panic!("{key} is not a number in {line}"),
        },
        None => panic!("no {key} in {line}"),
    }
}

fn line_of<'a>(summary: &'a str, prefix: &str) -> &'a str {
    match summary.lines().find(|l| l.starts_with(prefix)) {
        Some(l) => l,
        None => panic!("no {prefix} line in\n{summary}"),
    }
}

// the actual finite baseline of eff3b4b33 attempted 304811 event records (262144 stored, 42667 omitted: the report was correctly refused); the bound must hold that demand
const MEASURED_ATTEMPTED_EVENTS: i32 = 304_811;
const _: () = assert!(
    SHO_EVENTS_MAX > MEASURED_ATTEMPTED_EVENTS,
    "the event table cannot hold the measured baseline demand"
);

#[test]
fn a_session_word_that_does_not_fit_is_visibly_omitted_and_the_report_is_incomplete() {
    let bound = SHO_EVENTS_MAX as u32;
    // the raw patterns the original may hold after a restore: a negative, both infinities and a quiet NaN travel as bits
    let patterns = [(-1.5f32).to_bits(), f32::INFINITY.to_bits(), f32::NEG_INFINITY.to_bits(), 0x7fc0_0001];
    // SAFETY (the whole loop): plain integers.
    unsafe {
        for i in 0..bound {
            c::eech_sho_core_session(
                if i % 2 == 0 { 1 } else { 0 },
                patterns[(i % 4) as usize],
                (i % 200) as i32,
                (i / 200 % 200) as i32,
            );
        }
    }
    let at_bound = summary();
    let session = line_of(&at_bound, "sho-session");
    assert_eq!(field(session, "pack_total") + field(session, "unpack_total"), u64::from(bound), "{session}");
    assert_eq!(
        (field(session, "pack_omitted"), field(session, "unpack_omitted")),
        (0, 0),
        "exactly at the bound nothing is omitted: {session}"
    );
    assert!(
        at_bound.contains("sho-complete value=1"),
        "a table filled exactly to its bound is still whole:\n{at_bound}"
    );

    // the Rust maximum and the C table bound are one number, the largest valid report index is the last stored event, and the first invalid index is refused by the converter and by the C report
    assert_eq!(
        field(line_of(&at_bound, "sho-tables"), "events_capacity"),
        u64::from(bound),
        "the C table bound and eech_engine::SHO_EVENTS_MAX disagree"
    );
    assert!(at_bound.contains("events_overflow=0"), "nothing overflowed at the bound");
    if let Err(e) = sector_height_row_text("event", SHO_EVENTS_MAX - 1, SHO_EVENTS_MAX, c::eech_sho_event_run) {
        panic!("the largest valid index must read: {e:?}");
    }
    assert!(matches!(
        sector_height_row_text("event", SHO_EVENTS_MAX, SHO_EVENTS_MAX, c::eech_sho_event_run),
        Err(EngineError::BadArgument(_))
    ));
    let mut buffer = vec![0 as std::os::raw::c_char; 4096];
    // SAFETY: the buffer outlives the call and the capacity passed is its length.
    assert_eq!(
        unsafe { c::eech_sho_event_run(SHO_EVENTS_MAX, buffer.as_mut_ptr(), buffer.len() as i32) },
        -1,
        "the C report must refuse the first index past its table"
    );
    // the raw words are stored as bits: the first four events carry the four patterns
    for (i, bits) in patterns.iter().enumerate() {
        let row = match sector_height_row_text("event", i as i32, SHO_EVENTS_MAX, c::eech_sho_event_run) {
            Ok(r) => r,
            Err(e) => panic!("event {i}: {e:?}"),
        };
        assert!(row.contains(&format!("after=0x{bits:08x}")), "the word must be stored as its bits: {row}");
    }

    // ONE more pack word: omitted, latched, incomplete
    // SAFETY: plain integers.
    unsafe { c::eech_sho_core_session(1, 0x3f80_0000, 1, 1) };
    let over = summary();
    let session = line_of(&over, "sho-session");
    assert_eq!(
        (
            field(session, "pack_total") - field(session, "pack_stored"),
            field(session, "unpack_total") - field(session, "unpack_stored")
        ),
        (1, 0),
        "{session}"
    );
    assert_eq!(field(session, "pack_omitted"), 1, "{session}");
    assert_eq!(field(line_of(&over, "sho-tables"), "events_overflow"), 1, "the overflow must latch");
    assert!(
        over.contains("sho-complete value=0"),
        "an omitted word must make the report incomplete:\n{over}"
    );

    // an unpack word after the overflow cannot restore completeness
    // SAFETY: plain integers.
    unsafe { c::eech_sho_core_session(0, 0x3f80_0000, 2, 2) };
    let later = summary();
    assert!(
        later.contains("sho-complete value=0") && field(line_of(&later, "sho-session"), "unpack_omitted") == 1,
        "{later}"
    );
}
