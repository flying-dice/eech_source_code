//! TEST-ONLY (EEDCS-49, D3, feature `sector-height-observer`): the recorder CORE -> the C report -> the REAL Rust converters, with a known coherent sequence and the join failures.
//!
//! The recorder core (csrc/eech_sho.c) is driven with plain data; the report is read back through `eech_engine::sector_height_summary_text` and `sector_height_row_text`, the very
//! functions the public `Engine::sector_height_*` methods call (only the C entry differs: the control uses the un-locked report functions because it holds no booted engine; the public
//! entries add the state check). It shows: INSERT, LINK and the host statement of ONE placement are joined by the record identity even when two same-family records in the same sector
//! interleave their statements; a city whose shape was not read is UNOBSERVED, never invalid; FIRST and fact-12 words are related per sector by their raw bits and distinct nonzero words
//! are not collapsed; every pack and unpack word is stored past the old 64 sample; a removal closes a record, and a statement, link or second insert that cannot join is counted and
//! makes the report incomplete; an index outside the table is refused by the converter. The report of the coherent part is also pinned against
//! `eech-world/tests/data/sector-height-observer-report.txt`, the real C text that the Lua driver's own guard consumes (so the chain C emitter -> converter -> driver runs on real text).
//!
//! It does NOT run the public funnel (no roster, no host, no entity heap, no retail data), and the Lua binding (`eech-dc/src/module.rs`) is pinned only by compilation and lint until the
//! native run. The observer's tables are private statics of one process, so this file holds ONE test.
use eech_engine::{sector_height_row_text, sector_height_summary_text, EngineError, SHO_CLEARANCES_MAX, SHO_EVENTS_MAX, SHO_SCAN_MAX};
use eech_engine_sys::sho_support as c;
use std::ffi::CString;

const FIXTURE: &str = include_str!("../../eech-world/tests/data/sector-height-observer-report.txt");

const OBJECT: i32 = 17;
const CITY: i32 = 5;
const FIXED: i32 = 0;
const CITY_FAMILY: i32 = 1;
const BUILDING_FAMILY: i32 = 2;
const UNOBSERVED: i32 = 0;
const VALID: i32 = 1;
const INVALID: i32 = 2;

fn word(f: f32) -> u32 {
    f.to_bits()
}

fn field(line: &str, key: &str) -> String {
    let prefix = format!("{key}=");
    match line.split_whitespace().find_map(|w| w.strip_prefix(prefix.as_str())) {
        Some(v) => v.to_string(),
        None => panic!("no {key} in {line}"),
    }
}

fn number(line: &str, key: &str) -> u64 {
    match field(line, key).parse() {
        Ok(v) => v,
        Err(_) => panic!("{key} is not a number in {line}"),
    }
}

fn summary() -> String {
    match sector_height_summary_text(c::eech_sho_summary_run) {
        Ok(s) => s,
        Err(e) => panic!("the summary converter failed: {e:?}"),
    }
}

fn line_of<'a>(summary: &'a str, prefix: &str) -> &'a str {
    match summary.lines().find(|l| l.starts_with(prefix)) {
        Some(l) => l,
        None => panic!("no {prefix} line in\n{summary}"),
    }
}

fn row(what: &str, index: i32, max: i32, entry: unsafe extern "C" fn(i32, *mut std::os::raw::c_char, i32) -> i32) -> String {
    match sector_height_row_text(what, index, max, entry) {
        Ok(s) => s,
        Err(e) => panic!("{what} {index}: {e:?}"),
    }
}

fn event(i: i32) -> String {
    row("event", i, SHO_EVENTS_MAX, c::eech_sho_event_run)
}

fn statement(ent: i32, etype: i32, family: i32, sector_placed: i32, key: &str, base: u32) {
    let key = match CString::new(key) {
        Ok(k) => k,
        Err(_) => panic!("key"),
    };
    let words = [base, base + 1, base + 2, base + 3, base + 4, base + 5];
    // SAFETY: the key is NUL-terminated and the words slice holds the count passed.
    unsafe { c::eech_sho_core_statement(ent, etype, family, sector_placed, key.as_ptr(), words.as_ptr(), 6) }
}

#[allow(clippy::too_many_arguments)]
fn link(ent: i32, etype: i32, family: i32, shape_state: i32, shape: i32, before: u32, after: u32, sx: i32, sz: i32) {
    let pos = [word(10.0), word(20.0), word(30.0)];
    let ymax = word(5.0);
    // SAFETY: pos holds three words; the pointer is only read.
    unsafe {
        c::eech_sho_core_link_begin(before);
        c::eech_sho_core_link_end(
            ent,
            etype,
            family,
            1,
            shape_state,
            shape,
            if shape_state == VALID { pos.as_ptr() } else { std::ptr::null() },
            ymax,
            after,
            sx,
            sz,
        );
    }
}

#[test]
fn the_recorder_joins_one_placement_and_reports_what_it_cannot_join() {
    // ---- a coherent sequence: two same-family records in ONE sector, interleaved statements, a city with an unread shape, an observed-invalid city, a mobile
    // SAFETY (the whole block): plain integers and pointers to live locals.
    unsafe {
        c::eech_sho_core_insert(100, OBJECT, FIXED, 3, 4);
        c::eech_sho_core_insert(101, OBJECT, FIXED, 3, 4);
        c::eech_sho_core_insert_mobile();
    }
    link(100, OBJECT, FIXED, VALID, 12, 0, word(100.0), 3, 4);
    link(101, OBJECT, FIXED, VALID, 13, word(100.0), word(128.0), 3, 4);
    // the host's statements arrive in the OPPOSITE order of the inserts
    statement(101, OBJECT, FIXED, 1, "2002", 0x4000_0000);
    statement(100, OBJECT, FIXED, 1, "2001", 0x4100_0000);
    // SAFETY: plain integers.
    unsafe {
        c::eech_sho_core_insert(200, CITY, CITY_FAMILY, 3, 4);
        c::eech_sho_core_insert(201, CITY, CITY_FAMILY, 7, 8);
        c::eech_sho_core_link_begin(word(128.0));
        c::eech_sho_core_link_end(200, CITY, CITY_FAMILY, 1, UNOBSERVED, 0, std::ptr::null(), 0, word(128.0), 3, 4);
    }
    link(201, CITY, CITY_FAMILY, INVALID, 0, 0, 0, 7, 8);
    // a city building's FIRST statement is made under its city-list key (its sector placement does not exist yet): no sector record is expected for it and it is counted apart.
    // (This control drives the recorder core in a preferred sequence; the real nested callback order and the building's two stages are the funnel control's, sector_height_funnel.rs.)
    statement(300, 6, BUILDING_FAMILY, 0, "3003", 0x4200_0000);
    // SAFETY: plain integers.
    unsafe {
        // FIRST reads and fact 12: two sectors with distinct nonzero words; one offer carries another word than its sector's FIRST
        c::eech_sho_core_scan_first(3, 4, word(128.0));
        c::eech_sho_core_candidate_fact(word(128.0));
        c::eech_sho_core_candidate_fact(word(100.0));
        c::eech_sho_core_scan_first(5, 6, word(256.0));
        c::eech_sho_core_candidate_fact(word(256.0));
        c::eech_sho_core_candidate_fact(word(256.0));
        // the guarded clearance
        c::eech_sho_core_clearance_leader();
        c::eech_sho_core_clearance_follow();
        c::eech_sho_core_clearance_tallest(3, 4, word(128.0), word(40.0), word(88.0));
        c::eech_sho_core_clearance_final(word(88.0));
        // the session: past the old 64-word sample
        for i in 0..70 {
            c::eech_sho_core_session(1, word(i as f32), i, 0);
            c::eech_sho_core_session(0, word(i as f32), i, 0);
        }
    }

    let report = summary();
    assert!(report.contains("sho-complete value=1"), "a coherent sequence must be whole:\n{report}");
    let join = line_of(&report, "sho-join");
    assert_eq!(number(join, "links_joined"), 4, "{join}");
    assert_eq!(number(join, "links_unjoined"), 0, "{join}");
    assert_eq!(number(join, "stmt_joined"), 2, "{join}");
    assert_eq!(number(join, "stmt_child"), 1, "{join}");
    assert_eq!(number(join, "link_mobile"), 0, "{join}");
    let shape = line_of(&report, "sho-shape");
    assert_eq!(
        (number(shape, "fixed_valid"), number(shape, "city_unobserved"), number(shape, "city_invalid")),
        (2, 1, 1),
        "an unread city shape is UNOBSERVED, not invalid: {shape}"
    );
    let scan = line_of(&report, "sho-scan");
    assert_eq!(
        (
            number(scan, "fact_offers"),
            number(scan, "fact_equal_first"),
            number(scan, "fact_differ_first"),
            number(scan, "fact_without_first")
        ),
        (4, 3, 1, 0),
        "{scan}"
    );
    let session = line_of(&report, "sho-session");
    assert_eq!(
        (number(session, "pack_total"), number(session, "pack_stored"), number(session, "pack_omitted")),
        (70, 70, 0),
        "the pack record must not stop at 64: {session}"
    );
    assert_eq!(
        (
            number(session, "unpack_total"),
            number(session, "unpack_stored"),
            number(session, "unpack_omitted")
        ),
        (70, 70, 0),
        "{session}"
    );

    // ---- the rows: every record of a placement carries ITS record identity, not one guessed from the family or the sector
    let events = number(line_of(&report, "sho-tables"), "events_used") as i32;
    let mut by_ent: std::collections::BTreeMap<u64, Vec<(String, u64)>> = std::collections::BTreeMap::new();
    let mut text = String::new();
    for i in 0..events {
        let e = event(i);
        by_ent.entry(number(&e, "ent")).or_default().push((field(&e, "kind"), number(&e, "rec")));
        text.push_str(&e);
    }
    let kinds = |ent: u64| {
        by_ent
            .get(&ent)
            .map(|v| v.iter().map(|(k, r)| format!("{k}:{r}")).collect::<Vec<_>>())
            .unwrap_or_default()
    };
    assert_eq!(kinds(100), ["insert:1", "link:1", "stmt_fixed:1"], "{:?}", by_ent);
    assert_eq!(
        kinds(101),
        ["insert:2", "link:2", "stmt_fixed:2"],
        "interleaved statements must stay with their own record: {:?}",
        by_ent
    );
    assert_eq!(kinds(200), ["insert:3", "link:3"]);
    let link_200 = (0..events).map(event).find(|e| field(e, "kind") == "link" && number(e, "ent") == 200);
    assert!(link_200.is_some_and(|e| field(&e, "shape_state") == "unobserved"));

    // ---- the whole C report of this coherent state, as text the Lua driver's guard replays
    let mut pinned = report.clone();
    pinned.push_str(&text);
    for i in 0..number(line_of(&report, "sho-clearance"), "stored") as i32 {
        pinned.push_str(&row("clearance", i, SHO_CLEARANCES_MAX, c::eech_sho_clearance_run));
    }
    for i in 0..number(line_of(&report, "sho-scan"), "sectors") as i32 {
        pinned.push_str(&row("scan", i, SHO_SCAN_MAX, c::eech_sho_scan_run));
    }
    println!("-----BEGIN SHO REPORT-----\n{pinned}-----END SHO REPORT-----");
    assert_eq!(
        pinned, FIXTURE,
        "the real C report changed: regenerate eech-world/tests/data/sector-height-observer-report.txt from the stdout block above"
    );

    // ---- the converter refuses an index the table cannot hold or the C side did not record
    assert!(matches!(
        sector_height_row_text("event", -1, SHO_EVENTS_MAX, c::eech_sho_event_run),
        Err(EngineError::BadArgument(_))
    ));
    assert!(matches!(
        sector_height_row_text("event", SHO_EVENTS_MAX, SHO_EVENTS_MAX, c::eech_sho_event_run),
        Err(EngineError::BadArgument(_))
    ));
    assert!(
        matches!(
            sector_height_row_text("event", events, SHO_EVENTS_MAX, c::eech_sho_event_run),
            Err(EngineError::BadArgument(_))
        ),
        "an unrecorded index is refused by the C report"
    );

    // ---- join failures are counted and break completeness: a removal closes a record; the same entity's later statement, a link with no record and a second insert cannot join
    // SAFETY: plain integers.
    unsafe { c::eech_sho_core_remove(100, OBJECT, FIXED, 1) };
    assert!(summary().contains("sho-complete value=1"), "a removal alone keeps the report whole");
    statement(100, OBJECT, FIXED, 1, "2001", 0x4300_0000);
    let after = summary();
    assert!(
        after.contains("sho-complete value=0") && number(line_of(&after, "sho-join"), "stmt_unjoined") == 1,
        "{after}"
    );
    link(900, OBJECT, FIXED, VALID, 3, 0, 0, 1, 1);
    // SAFETY: plain integers.
    unsafe {
        c::eech_sho_core_insert(101, OBJECT, FIXED, 3, 4);
        c::eech_sho_core_release(101);
        c::eech_sho_core_insert(101, OBJECT, FIXED, 3, 4);
    }
    let join = summary();
    let join = line_of(&join, "sho-join");
    assert_eq!(number(join, "links_unjoined"), 1, "{join}");
    assert_eq!(
        number(join, "double_insert"),
        1,
        "the first insert of 101 was never removed; the release then lets the reused index join a new record: {join}"
    );

    // a statement under a city-list key from a family that has no such stage (not a city building) is outside the placement map's domain and is a failure of its own
    statement(101, OBJECT, FIXED, 0, "2002", 0x4400_0000);
    let last = summary();
    assert_eq!(number(line_of(&last, "sho-join"), "stmt_child_unexpected"), 1, "{last}");
}
