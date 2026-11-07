//! TEST-ONLY (EEDCS-49, D3, feature `sector-height-funnel-check`): the control of the observer's JOIN against the REAL placement map in the callback order of the staged funnel.
//!
//! The real funnel is `en_list.c`: `campaign_place_insert (en)` is ONE call that registers the placement, declares it and states the entity's fixed or city state to the host
//! (`mb_hostplacemap.h` `place_insert` -> `place_fixed_state`) BEFORE it returns, and LINK_CHILD follows. A city building is placed twice (its city's list, then its sector), and the placement
//! map prefers the sector key as soon as it exists. Here the real STAGED `mb_hostplacemap.h` (with its staged statement hooks) runs bound to scripted entities and a scripted host
//! (`csrc/eech_sho_funnel_check.c`), and the recorder core, the C report and the real Rust converters are the ones the public methods use.
//!
//! THE ORDER IS NOT HARD-CODED. Whether the observer's insert hook stands before or after `campaign_place_insert`, and the order of the other staged hooks, are read from the ACTUAL staged patch
//! text: this test applies the real patch table (`build/patches.rs`, included as a module) to the real in-tree `en_list.c`, `sc_msgs.c` and `en_heap.c` and asserts the relations the control
//! relies on, then hands the insert order to the C control. A staged patch that stood after the call (as in `d74ca2e1b`) therefore makes THIS test fail, with the nested statement meeting no record.
//!
//! It does NOT run the public funnel (the real en_list, the entity heap, the roster provider or the host), the staged `en_list.c` itself, or the Lua binding. The observer's tables are private
//! statics of one process, so this file holds ONE test.
use eech_engine::{sector_height_row_text, sector_height_summary_text, SHO_EVENTS_MAX};
use eech_engine_sys::sho_support as c;
use std::collections::BTreeMap;

#[allow(dead_code)]
#[path = "../../eech-engine-sys/build/patches.rs"]
mod patches;

const EN_LIST: &[u8] = include_bytes!("../../../../aphavoc/source/entity/system/en_funcs/en_list.c");
const SC_MSGS: &[u8] = include_bytes!("../../../../aphavoc/source/entity/special/sector/sc_msgs.c");
const EN_HEAP: &[u8] = include_bytes!("../../../../aphavoc/source/entity/system/en_main/en_heap.c");
const PLACEMAP: &[u8] = include_bytes!("../../../../aphavoc/source/entity/mobile/mb_hostplacemap.h");

fn staged(file: &str, original: &[u8]) -> String {
    // the sources are Latin-1: bytes are chars 0..=255, as build/main.rs stages them
    let text: String = original.iter().map(|&b| b as char).collect();
    patches::apply_sector_height_observer(file, &text)
}

fn at(text: &str, needle: &str) -> usize {
    match text.find(needle) {
        Some(i) => i,
        None => panic!("the staged text has no {needle:?}"),
    }
}

/// the insert order of the staged en_list.c, and every other relation of the staged hooks that the C control reproduces
fn staged_order() -> bool {
    let list = staged("aphavoc/source/entity/system/en_funcs/en_list.c", EN_LIST);
    let (hook, place, link) = (
        at(&list, "eech_sho_sector_insert (en, parent)"),
        at(&list, "campaign_place_insert (en);"),
        at(&list, "ENTITY_MESSAGE_LINK_CHILD, parent, en, type"),
    );
    assert!(place < link && hook < link, "the insert hook must stand before LINK_CHILD");
    let (place_remove, remove_hook) = (at(&list, "campaign_place_remove (en);"), at(&list, "eech_sho_sector_remove (en)"));
    assert!(
        place_remove < remove_hook,
        "the removal hook follows the roster's removal, while the entity is still listed (before UNLINK_CHILD)"
    );
    assert!(remove_hook < at(&list, "ENTITY_MESSAGE_UNLINK_CHILD, parent, en, type"));
    let sector = staged("aphavoc/source/entity/special/sector/sc_msgs.c", SC_MSGS);
    let (begin, raise, end) = (
        at(&sector, "eech_sho_link_begin (receiver"),
        at(&sector, "if (height > raw->tallest_structure_height)"),
        at(&sector, "eech_sho_link_end (receiver"),
    );
    assert!(begin < raise && raise < end, "the link hooks surround the original raise");
    let heap = staged("aphavoc/source/entity/system/en_main/en_heap.c", EN_HEAP);
    assert!(
        at(&heap, "campaign_place_release_entity (en);") < at(&heap, "eech_sho_entity_release (en)"),
        "the release hook follows the placement map's release"
    );
    // the statement hooks stand before the host's statement and pass the SECTOR-placement relationship of the key the map itself chooses
    let map = staged("aphavoc/source/entity/mobile/mb_hostplacemap.h", PLACEMAP);
    let key = at(
        &map,
        "key = place_key_of_slot [slot][0] ? place_key_of_slot [slot] : place_child_of_slot [slot];",
    );
    for (hook, call) in [
        (
            "eech_sho_statement_city (en, place_key_of_slot [slot][0] != 0, key, city)",
            "campaign_model_entity_city_state (key, city);",
        ),
        (
            "eech_sho_statement_fixed (en, CAMPAIGN_PLACE_TYPE (en), place_key_of_slot [slot][0] != 0, key, state)",
            "campaign_model_entity_fixed (CAMPAIGN_PLACE_TYPE (en), key, state);",
        ),
    ] {
        assert!(
            key < at(&map, hook) && at(&map, hook) < at(&map, call),
            "{hook} must stand between the key choice and the host statement"
        );
    }
    hook < place
}

fn text_of(f: impl Fn(&mut [std::os::raw::c_char]) -> i32) -> String {
    let mut buffer = vec![0 as std::os::raw::c_char; 512];
    f(&mut buffer);
    buffer.iter().take_while(|&&ch| ch != 0).map(|&ch| ch as u8 as char).collect()
}

fn run(scenario: i32, hook_first: bool) {
    // SAFETY: plain integers; the control catches the placement map's own stop.
    let stopped = unsafe { c::eech_sho_funnel_run(scenario, i32::from(hook_first)) };
    // SAFETY: the buffer outlives the call and the capacity passed is its length.
    let why = text_of(|b| unsafe { c::eech_sho_funnel_text(b.as_mut_ptr(), b.len() as i32) });
    assert_eq!(stopped, 0, "scenario {scenario} stopped the placement map: {why}");
}

fn summary() -> String {
    match sector_height_summary_text(c::eech_sho_summary_run) {
        Ok(s) => s,
        Err(e) => panic!("the summary converter failed: {e:?}"),
    }
}

fn line_of<'a>(report: &'a str, prefix: &str) -> &'a str {
    match report.lines().find(|l| l.starts_with(prefix)) {
        Some(l) => l,
        None => panic!("no {prefix} line in\n{report}"),
    }
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

#[test]
fn the_real_placement_map_in_the_staged_order_joins_every_placement_and_still_refuses_what_has_no_record() {
    let hook_first = staged_order();
    run(0, hook_first);
    for scenario in [10, 11, 12, 13, 14, 16, 17] {
        run(scenario, hook_first);
    }

    let report = summary();
    let join = line_of(&report, "sho-join");
    assert_eq!(
        number(join, "stmt_unjoined"),
        0,
        "the first state statement of a placement met no observer record (the insert hook stands after the nested call): {join}\n{report}"
    );
    assert!(report.contains("sho-complete value=1"), "the real funnel sequence must be whole:\n{report}");
    assert_eq!((number(join, "links_joined"), number(join, "links_unjoined")), (6, 0), "{join}");
    assert_eq!(
        (number(join, "stmt_joined"), number(join, "stmt_child"), number(join, "stmt_child_unexpected")),
        (9, 1, 0),
        "{join}"
    );
    assert_eq!(
        (
            number(join, "double_insert"),
            number(join, "remove_unjoined"),
            number(join, "stmt_type_mismatch")
        ),
        (0, 0, 0),
        "{join}"
    );
    let counts = line_of(&report, "sho-counts");
    assert_eq!(
        (
            number(counts, "inserts_total"),
            number(counts, "removes_recorded"),
            number(counts, "released_open")
        ),
        (6, 1, 1),
        "{counts}"
    );

    // the rows: the callback order of ONE placement is INSERT, the nested statement, LINK; two same-family records of one sector stay apart; the building's child-stage statement carries no record
    // and its sector-stage statement carries the building's
    let events = number(line_of(&report, "sho-tables"), "events_used") as i32;
    let mut by_ent: BTreeMap<u64, Vec<String>> = BTreeMap::new();
    for i in 0..events {
        let row = match sector_height_row_text("event", i, SHO_EVENTS_MAX, c::eech_sho_event_run) {
            Ok(r) => r,
            Err(e) => panic!("event {i}: {e:?}"),
        };
        by_ent
            .entry(number(&row, "ent"))
            .or_default()
            .push(format!("{}:{}", field(&row, "kind"), field(&row, "rec")));
    }
    let kinds = |ent: u64| by_ent.get(&ent).cloned().unwrap_or_default();
    assert_eq!(
        kinds(1),
        [
            "insert:1",
            "stmt_fixed:1",
            "link:1",
            "stmt_fixed:1",
            "remove:1",
            "insert:5",
            "stmt_fixed:5",
            "link:5"
        ],
        "{by_ent:?}"
    );
    assert_eq!(
        kinds(2),
        [
            "insert:2",
            "stmt_fixed:2",
            "link:2",
            "stmt_fixed:2",
            "release:2",
            "insert:6",
            "stmt_fixed:6",
            "link:6"
        ],
        "interleaved same-family records must stay apart: {by_ent:?}"
    );
    assert_eq!(kinds(3), ["insert:3", "stmt_city:3", "link:3"], "{by_ent:?}");
    assert_eq!(
        kinds(4),
        ["stmt_fixed:0", "insert:4", "stmt_fixed:4", "link:4", "stmt_fixed:4"],
        "only the sector-stage statement of a city building joins its sector record: {by_ent:?}"
    );

    // ---- genuinely unjoined: a placement the observer never saw, and a later restatement of it, are still refused and break completeness
    run(18, hook_first);
    run(19, hook_first);
    let after = summary();
    let join = line_of(&after, "sho-join");
    assert!(
        after.contains("sho-complete value=0"),
        "an unjoined placement must make the report incomplete:\n{after}"
    );
    assert_eq!(number(join, "stmt_unjoined"), 2, "{join}");
    assert_eq!(number(join, "links_unjoined"), 1, "{join}");
}
