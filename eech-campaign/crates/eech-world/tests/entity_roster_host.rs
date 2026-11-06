//! The host's sector roster through the reference host's real provider (EEDCS-49, D3, sixth slice): which entities stand in which sector and in what order a scan visits them.
//!
//! The campaign reports each insertion into and removal from a sector list (the one funnel `en_list.c`) and the host answers with a key it issued; a scan then asks the host for the first entry of a
//! sector and for what follows an entry, and the host serves them from ITS order. These tests drive the REAL provider (`host.model`) with the REAL `SectorRoster` and the REAL `Movers` registry, speaking
//! the questions the campaign's C driver sends (`mb_hostscan.h`, `mb_hostentity.h`, whose own half is checked in eech-engine-sys `entity_dialogue`), and assert the externally meaningful facts: the order
//! (oldest insertion first, a relink goes to the end, mixed fixed and mobile families), the sector a hosted mobile is placed in (its own record's, never the campaign's word), and that an ended, unknown or
//! misplaced entry is refused by name and never skipped or served as the end of a list.

use eech_world::movement::{Kind, Movers, Pose, SharedMovers};
use eech_world::roster::{SectorRoster, SharedRoster};
use eech_world::trees::{clear_exception_flags, exception_flags, fp_state, raise_exception_flags, rounding, Rounding, ScopedRounding};
use mlua::{Lua, Table};
use std::cell::RefCell;
use std::rc::Rc;

const SIDE: f32 = 1000.0;
const LEVEL: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
/// the phases of the question: the scan's own and the roster's events
const BEGIN: f32 = 0.0;
const ENTRY: f32 = 1.0;
const END: f32 = 2.0;
const RESOLVED: f32 = 3.0;
const INSERT: f32 = 4.0;
const REMOVE: f32 = 5.0;
const RESET: f32 = 6.0;

fn host(movers: Rc<RefCell<Movers>>) -> (Lua, Rc<RefCell<SectorRoster>>) {
    let lua = Lua::new();
    let host = lua.create_table().unwrap();
    host.set("args", lua.create_table().unwrap()).unwrap();
    eech_world::model_host::register(&lua, &host).unwrap();
    lua.globals().set("host", host).unwrap();
    lua.set_app_data(SharedMovers(movers));
    let roster = Rc::new(RefCell::new(SectorRoster::default()));
    lua.set_app_data(SharedRoster(roster.clone()));
    (lua, roster)
}

fn registry() -> Rc<RefCell<Movers>> {
    let mut movers = Movers::default();
    let at = |x: f32, z: f32| Pose {
        position: [x, 2.0, z],
        velocity: 5.0,
        attitude: LEVEL,
    };
    movers
        .bind("eech-world:1", Kind::Routed, at(1500.0, 1500.0), Some([0.0, 0.0, 5.0]), None)
        .unwrap();
    movers
        .bind("eech-world:2", Kind::Routed, at(2500.0, 1500.0), Some([0.0, 0.0, 5.0]), None)
        .unwrap();
    movers.issue("eech-world:3").unwrap();
    movers.bind("eech-world:4", Kind::Person, at(1500.0, 1500.0), Some([0.0; 3]), None).unwrap();
    movers.retire("eech-world:4").unwrap();
    Rc::new(RefCell::new(movers))
}

/// one question of the scan or the roster: the sixteen facts (the segment, the map, the phase, the cursor, the two flags) and, when given, a space and the identity place
fn ask(lua: &Lua, facts: [f32; 16], identity: &str) -> Table {
    ask_words(lua, facts.map(f32::to_bits), identity)
}

/// the same question from the raw words: a signaling word is sent as the bits the campaign held, never moved through a float
fn ask_words(lua: &Lua, words: [u32; 16], identity: &str) -> Table {
    let patterns = words.iter().map(|word| format!("0x{word:08x}")).collect::<Vec<_>>().join(",");
    let text = if identity.is_empty() { patterns } else { format!("{patterns} {identity}") };
    lua.globals().set("KEY", text).unwrap();
    lua.load(r#"return host.model ("entity_scan", 0, nil, nil, nil, KEY)"#).eval().unwrap()
}

/// the facts of a scan question over a segment that lies wholly inside sector (1, 1), whatever the phase
fn scan_facts(phase: f32, cursor: [f32; 2], excluded: f32, hit: f32) -> [f32; 16] {
    [
        1500.0, 50.0, 1500.0, 1600.0, 50.0, 1600.0, SIDE, 0.0, 0.0, 9999.0, 9999.0, phase, cursor[0], cursor[1], excluded, hit,
    ]
}

/// the facts of a roster event: the position (x, y, z), the entity's type, the side length and the phase
fn event_facts(phase: f32, position: [f32; 3], entity_type: f32) -> [f32; 16] {
    let mut facts = [0.0f32; 16];
    facts[..3].copy_from_slice(&position);
    facts[3] = entity_type;
    facts[6] = SIDE;
    facts[11] = phase;
    facts
}

fn result(table: &Table) -> String {
    table.get("result").unwrap()
}

fn name(table: &Table) -> String {
    table.get::<Option<String>>("name").unwrap().unwrap_or_default()
}

fn reason(table: &Table) -> String {
    table.get("reason").unwrap()
}

/// an insertion; panics on a refusal (the refusal cases ask directly)
fn insert(lua: &Lua, position: [f32; 3], entity_type: f32, identity: &str) -> String {
    let table = ask(lua, event_facts(INSERT, position, entity_type), identity);
    assert_eq!(result(&table), "placed", "{:?}", table.get::<Option<String>>("reason"));
    name(&table)
}

fn remove(lua: &Lua, key: &str) -> Table {
    ask(lua, event_facts(REMOVE, [0.0; 3], 0.0), key)
}

/// the keys a scan of sector (1, 1) is served, from the first entry to the end, exactly as the campaign's driver asks: FIRST, then the question that follows an entry names it
fn served(lua: &Lua) -> Vec<String> {
    let begin = ask(lua, scan_facts(BEGIN, [0.0; 2], 0.0, 0.0), "");
    assert_eq!(result(&begin), "first");
    let mut keys = Vec::new();
    let mut at = name(&begin);
    while !at.is_empty() {
        keys.push(at.clone());
        // not excluded: the entry is offered, and a miss goes on to the next one
        let offered = ask(lua, scan_facts(ENTRY, [1.0, 1.0], 0.0, 0.0), &at);
        assert_eq!(result(&offered), "offer");
        let next = ask(lua, scan_facts(RESOLVED, [1.0, 1.0], 0.0, 0.0), &at);
        assert_eq!(result(&next), "next", "{:?}", next.get::<Option<String>>("reason"));
        at = name(&next);
    }
    keys
}

#[test]
fn a_scan_is_served_the_oldest_insertion_first_across_fixed_and_mobile_entities_and_a_relink_goes_to_the_end() {
    let (lua, roster) = host(registry());
    // a structure, a hosted vehicle that stands in the same sector by the host's own record, and a second structure: three different families in one list
    let building = insert(&lua, [1100.0, 3.0, 1200.0], 20.0, "");
    let vehicle = insert(&lua, [1500.0, 2.0, 1500.0], 3.0, "eech-world:1");
    let scenery = insert(&lua, [1900.0, 1.0, 1999.0], 21.0, "");
    assert_eq!(
        served(&lua),
        vec![building.clone(), vehicle.clone(), scenery.clone()],
        "oldest first: the order of the insertions"
    );
    assert_eq!(
        roster.borrow().host_id(vehicle.parse().unwrap()),
        Some("eech-world:1"),
        "the mobile's key maps to its semantic identity"
    );
    assert_eq!(roster.borrow().host_id(building.parse().unwrap()), None);
    // the vehicle leaves the sector list and is inserted again (a relink): it is now the newest, and it has a new key
    assert_eq!(result(&remove(&lua, &vehicle)), "removed");
    let again = insert(&lua, [1500.0, 2.0, 1500.0], 3.0, "eech-world:1");
    assert_ne!(again, vehicle, "a removed key is never issued again");
    assert_eq!(served(&lua), vec![building.clone(), scenery.clone(), again.clone()]);
    // removing an entry leaves the others in order
    assert_eq!(result(&remove(&lua, &building)), "removed");
    assert_eq!(served(&lua), vec![scenery, again]);
}

#[test]
fn a_sector_with_no_entries_is_served_as_an_empty_list_and_an_other_sector_is_not_mixed_in() {
    let (lua, _) = host(registry());
    let _elsewhere = insert(&lua, [2500.0, 3.0, 1500.0], 20.0, "");
    assert!(served(&lua).is_empty(), "sector (1, 1) holds nothing: the entry in sector (2, 1) is not served");
    let first = ask(&lua, scan_facts(BEGIN, [0.0; 2], 0.0, 0.0), "");
    assert_eq!(result(&first), "first");
    assert_eq!(name(&first), "");
}

#[test]
fn a_hosted_mobile_is_placed_by_the_hosts_own_record_and_the_campaigns_word_is_only_compared_with_it() {
    let (lua, roster) = host(registry());
    // the campaign states a position in sector (1, 1) for the vehicle whose record stands in sector (2, 1): a named refusal, and nothing is placed
    let table = ask(&lua, event_facts(INSERT, [1500.0, 2.0, 1500.0], 3.0), "eech-world:2");
    assert_eq!(result(&table), "refused");
    assert!(
        reason(&table).contains("[roster.pose]") && reason(&table).contains("eech-world:2"),
        "{}",
        reason(&table)
    );
    assert_eq!(roster.borrow().len(), 0);
    // stated where the record is: placed, in sector (2, 1) (not served by a scan of sector (1, 1))
    let key = insert(&lua, [2500.0, 2.0, 1500.0], 3.0, "eech-world:2");
    assert_eq!(roster.borrow().sector(key.parse().unwrap()), Some((2, 1)));
    assert!(served(&lua).is_empty());
    // a retired identity cannot be inserted; an identity the host issued without a record states its own position; an identity it never issued does the same (it keeps no record to compare)
    let retired = ask(&lua, event_facts(INSERT, [1500.0, 2.0, 1500.0], 3.0), "eech-world:4");
    assert_eq!(result(&retired), "refused");
    assert!(reason(&retired).contains("retired"), "{}", reason(&retired));
    let unrecorded = insert(&lua, [1500.0, 2.0, 1500.0], 3.0, "eech-world:3");
    assert_eq!(roster.borrow().sector(unrecorded.parse().unwrap()), Some((1, 1)));
}

#[test]
fn an_ended_unknown_or_misplaced_entry_is_refused_by_name_and_is_never_skipped_or_an_end() {
    let (lua, _) = host(registry());
    let a = insert(&lua, [1100.0, 3.0, 1100.0], 20.0, "");
    let b = insert(&lua, [1200.0, 3.0, 1200.0], 21.0, "");
    assert_eq!(result(&remove(&lua, &a)), "removed");
    // the scan stands at an entry that has ended: refused by name (it is not the end of the list and not the entry after it)
    let ended = ask(&lua, scan_facts(RESOLVED, [1.0, 1.0], 0.0, 0.0), &a);
    assert_eq!(result(&ended), "refused");
    assert!(
        reason(&ended).contains("[roster.key]") && reason(&ended).contains("has ended"),
        "{}",
        reason(&ended)
    );
    let unknown = ask(&lua, scan_facts(RESOLVED, [1.0, 1.0], 0.0, 0.0), "999");
    assert!(reason(&unknown).contains("never issued"), "{}", reason(&unknown));
    // an entry in another sector than the one the scan is in
    // (the segment is widened so that sector (2, 1) is inside the window the cursor must lie in)
    let mut wide = scan_facts(RESOLVED, [2.0, 1.0], 0.0, 0.0);
    wide[3] = 2600.0;
    let misplaced = ask(&lua, wide, &b);
    assert_eq!(result(&misplaced), "refused");
    assert!(reason(&misplaced).contains("not in the sector"), "{}", reason(&misplaced));
    // a question that is not a key at all
    let text = ask(&lua, scan_facts(RESOLVED, [1.0, 1.0], 0.0, 0.0), "eech-world:1");
    assert_eq!(result(&text), "refused");
    // removing an ended or unknown key is refused too, and a removal that is not a key at all
    assert_eq!(result(&remove(&lua, &a)), "refused");
    assert_eq!(result(&remove(&lua, "12345")), "refused");
    assert!(reason(&remove(&lua, "x")).contains("[roster.key]"));
}

#[test]
fn a_reset_ends_every_entry_and_a_key_of_the_earlier_boot_names_nothing() {
    let (lua, roster) = host(registry());
    let before = insert(&lua, [1100.0, 3.0, 1100.0], 20.0, "");
    let table = ask(&lua, event_facts(RESET, [0.0; 3], 0.0), "");
    assert_eq!(result(&table), "cleared");
    assert!(roster.borrow().is_empty());
    assert!(served(&lua).is_empty());
    let ended = ask(&lua, scan_facts(RESOLVED, [1.0, 1.0], 0.0, 0.0), &before);
    assert_eq!(result(&ended), "refused");
    assert!(reason(&ended).contains("has ended"), "{}", reason(&ended));
    let after = insert(&lua, [1100.0, 3.0, 1100.0], 20.0, "");
    assert_ne!(after, before, "a key of an earlier boot is never reissued");
}

#[test]
fn a_malformed_insertion_is_refused_before_anything_is_placed() {
    let (lua, roster) = host(registry());
    let mut facts = event_facts(INSERT, [1100.0, 3.0, 1100.0], 20.0);
    facts[3] = 2.5;
    assert!(reason(&ask(&lua, facts, "")).contains("[roster.request]"));
    facts = event_facts(INSERT, [1100.0, 3.0, 1100.0], 20.0);
    facts[6] = 0.0;
    assert!(reason(&ask(&lua, facts, "")).contains("[roster.request]"));
    facts = event_facts(INSERT, [f32::NAN, 3.0, 1100.0], 20.0);
    assert!(reason(&ask(&lua, facts, "")).contains("[roster.position]"));
    facts = event_facts(INSERT, [f32::INFINITY, 3.0, 1100.0], 20.0);
    assert!(reason(&ask(&lua, facts, "")).contains("[roster.position]"));
    assert_eq!(roster.borrow().len(), 0);
}

#[test]
fn the_roster_events_need_the_hosts_roster_and_a_host_without_one_refuses_by_name() {
    let lua = Lua::new();
    let host = lua.create_table().unwrap();
    host.set("args", lua.create_table().unwrap()).unwrap();
    eech_world::model_host::register(&lua, &host).unwrap();
    lua.globals().set("host", host).unwrap();
    let insertion = ask(&lua, event_facts(INSERT, [1100.0, 3.0, 1100.0], 20.0), "");
    assert_eq!(result(&insertion), "refused");
    assert!(reason(&insertion).contains("keeps no sector roster"), "{}", reason(&insertion));
    // a scan that needs an entry from a roster the host does not keep is refused too, and is never the end of a list
    let first = ask(&lua, scan_facts(BEGIN, [0.0; 2], 0.0, 0.0), "");
    assert_eq!(result(&first), "refused");
    assert!(reason(&first).contains("[roster.missing]"), "{}", reason(&first));
}

/// a scan question over a segment wide enough that sector (2, 1) is inside the window the cursor must lie in
fn wide_facts(phase: f32, cursor: [f32; 2], excluded: f32, hit: f32) -> [f32; 16] {
    let mut facts = scan_facts(phase, cursor, excluded, hit);
    facts[3] = 2600.0;
    facts
}

#[test]
fn the_standing_key_of_an_entry_or_a_resolved_offer_is_validated_before_any_decision_and_only_those_phases_carry_one() {
    let (lua, roster) = host(registry());
    let a = insert(&lua, [1100.0, 3.0, 1100.0], 20.0, "");
    let b = insert(&lua, [1200.0, 3.0, 1200.0], 21.0, "");
    let elsewhere = insert(&lua, [2500.0, 3.0, 1500.0], 22.0, "");
    let ended = insert(&lua, [1300.0, 3.0, 1300.0], 23.0, "");
    assert_eq!(result(&remove(&lua, &ended)), "removed");
    let held = roster.borrow().len();

    // positive first: a live entry of the sector the scan is in is offered, skipped, hit or missed exactly as the kernel decides, in every phase that carries a key; BEGIN and END carry none
    let offered = ask(&lua, wide_facts(ENTRY, [1.0, 1.0], 0.0, 0.0), &a);
    assert_eq!((result(&offered), name(&offered)), ("offer".to_string(), String::new()));
    let skipped = ask(&lua, wide_facts(ENTRY, [1.0, 1.0], 1.0, 0.0), &a);
    assert_eq!((result(&skipped), name(&skipped)), ("next".to_string(), b.clone()));
    let hit = ask(&lua, wide_facts(RESOLVED, [1.0, 1.0], 0.0, 1.0), &a);
    assert_eq!((result(&hit), name(&hit)), ("done_hit".to_string(), String::new()));
    let missed = ask(&lua, wide_facts(RESOLVED, [1.0, 1.0], 0.0, 0.0), &a);
    assert_eq!((result(&missed), name(&missed)), ("next".to_string(), b.clone()));
    let begin = ask(&lua, wide_facts(BEGIN, [0.0; 2], 0.0, 0.0), "");
    assert_eq!(
        (result(&begin), name(&begin)),
        ("first".to_string(), a.clone()),
        "the sector's oldest insertion"
    );
    let end = ask(&lua, wide_facts(END, [2.0, 1.0], 0.0, 0.0), "");
    assert_eq!(result(&end), "done_none");

    // the same questions with a key that is not the live entry of the sector the scan stands in: refused by name, and the decision (an offer, a skip, a hit, a next) is never served
    let cases: [(&str, &str, &str); 7] = [
        ("unknown", "999", "never issued"),
        ("ended", &ended, "has ended"),
        ("another sector", &elsewhere, "not in the sector"),
        ("text", "eech-world:1", "not a key this host issued"),
        ("leading zero", "0012", "not a key this host issued"),
        ("signed", "+1", "not a key this host issued"),
        ("none", "", "names none"),
    ];
    for (phase, excluded, hit, decision) in [
        (ENTRY, 0.0, 0.0, "offer"),
        (ENTRY, 1.0, 0.0, "next"),
        (RESOLVED, 0.0, 1.0, "done_hit"),
        (RESOLVED, 0.0, 0.0, "next"),
    ] {
        for (what, key, why) in cases {
            let refused = ask(&lua, wide_facts(phase, [1.0, 1.0], excluded, hit), key);
            assert_eq!(result(&refused), "refused", "phase {phase} ({decision}) with the {what} key");
            assert!(
                reason(&refused).contains("[roster.key]") && reason(&refused).contains(why),
                "phase {phase} ({decision}) with the {what} key: {}",
                reason(&refused)
            );
            assert_eq!(name(&refused), "", "a refusal serves no entry");
        }
    }

    // BEGIN, END and the boot carry no standing key: one is refused by name where the real driver never sends one
    for (phase, cursor) in [(BEGIN, [0.0; 2]), (END, [1.0, 1.0])] {
        let stray = ask(&lua, wide_facts(phase, cursor, 0.0, 0.0), &a);
        assert_eq!(result(&stray), "refused", "phase {phase}");
        assert!(
            reason(&stray).contains("[roster.key]") && reason(&stray).contains("carries no"),
            "{}",
            reason(&stray)
        );
    }
    let stray = ask(&lua, event_facts(RESET, [0.0; 3], 0.0), &a);
    assert_eq!(result(&stray), "refused");
    assert!(
        reason(&stray).contains("[roster.key]") && reason(&stray).contains("carries no"),
        "{}",
        reason(&stray)
    );
    assert_eq!(roster.borrow().len(), held, "nothing a refused question named was changed");
    assert_eq!(served(&lua), vec![a, b], "and the roster still serves what it held");
}

/// the caller's whole floating-point state as the roster call sees it: masks, rounding, raw control words and the raised status flags
fn caller_state() -> (eech_world::trees::FpState, Rounding, i32) {
    (fp_state().expect("readable"), rounding().expect("one of the four"), exception_flags())
}

#[test]
fn the_roster_arithmetic_hands_back_the_callers_whole_floating_point_state_for_fractional_and_malformed_words() {
    const SIGNALING: u32 = 0x7fa0_0001;
    for mode in [Rounding::Nearest, Rounding::Up, Rounding::Down, Rounding::Zero] {
        for raised in [false, true] {
            let (lua, roster) = host(registry());
            let _caller = ScopedRounding::set(mode).expect("a caller in this rounding mode");
            clear_exception_flags().expect("clear");
            if raised {
                raise_exception_flags().expect("raise");
            }
            let before = caller_state();
            assert_eq!(before.2 != 0, raised, "the test's own setup");
            // valid fractional coordinates are truncated toward zero whatever the caller's rounding is, and the call is a placement
            let fractional = event_facts(INSERT, [1100.7, 3.3, 1100.9], 20.0).map(f32::to_bits);
            let placed = ask_words(&lua, fractional, "");
            assert_eq!(result(&placed), "placed", "{mode:?}: {:?}", placed.get::<Option<String>>("reason"));
            let key = name(&placed);
            assert_eq!(roster.borrow().sector(key.parse().unwrap()), Some((1, 1)), "{mode:?}: truncated, never rounded");
            assert_eq!(
                caller_state(),
                before,
                "{mode:?} raised={raised}: the caller's whole state after a placement with fractional coordinates"
            );
            // a live entry served and finished with the cursor the kernel names: the sector arithmetic of a valid scan
            let begin = ask_words(&lua, scan_facts(BEGIN, [0.0; 2], 0.0, 0.0).map(f32::to_bits), "");
            assert_eq!((result(&begin), name(&begin)), ("first".to_string(), key.clone()), "{mode:?}");
            assert_eq!(
                caller_state(),
                before,
                "{mode:?} raised={raised}: the caller's whole state after a served first entry"
            );
            // every malformed word is refused by name before any floating-point use of it
            let scan = scan_facts(BEGIN, [0.0; 2], 0.0, 0.0).map(f32::to_bits);
            let entry = scan_facts(ENTRY, [1.0, 1.0], 0.0, 0.0).map(f32::to_bits);
            let insertion = event_facts(INSERT, [1100.0, 3.0, 1100.0], 20.0).map(f32::to_bits);
            let with = |mut words: [u32; 16], at: usize, word: u32| {
                words[at] = word;
                words
            };
            let cases: [(&str, [u32; 16], &str, &str); 11] = [
                ("a signaling phase", with(scan, 11, SIGNALING), "", "fact 11"),
                ("an infinite phase", with(scan, 11, 0x7f80_0000), "", "fact 11"),
                ("a signaling position x", with(insertion, 0, SIGNALING), "", "[roster.position]"),
                ("a signaling position z", with(insertion, 2, SIGNALING), "", "[roster.position]"),
                ("an infinite position", with(insertion, 0, 0xff80_0000), "", "[roster.position]"),
                ("a signaling type", with(insertion, 3, SIGNALING), "", "[roster.request]"),
                ("a signaling side", with(insertion, 6, SIGNALING), "", "[roster.request]"),
                ("a fractional type", with(insertion, 3, 2.5f32.to_bits()), "", "[roster.request]"),
                ("a signaling cursor at an entry", with(entry, 12, SIGNALING), &key, "fact 12"),
                ("a signaling excluded flag at an entry", with(entry, 14, SIGNALING), &key, "fact 14"),
                ("a signaling hit flag at an entry", with(entry, 15, SIGNALING), &key, "fact 15"),
            ];
            for (what, words, identity, expected) in cases {
                let refused = ask_words(&lua, words, identity);
                assert_eq!(result(&refused), "refused", "{mode:?} raised={raised}: {what}");
                assert!(reason(&refused).contains(expected), "{mode:?} raised={raised}: {what}: {}", reason(&refused));
                assert_eq!(caller_state(), before, "{mode:?} raised={raised}: the caller's whole state after {what}");
            }
            assert_eq!(roster.borrow().len(), 1, "only the valid fractional placement is in the roster");
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the city lists (phases 7 to 12): a building's place in its city's list, served head first from the HOST's order, and one record for a building's two placements and its object-report identity

const CHILD_INSERT: f32 = 7.0;
const CHILD_REMOVE: f32 = 8.0;
const CHILD_FIRST: f32 = 9.0;
const CHILD_NEXT: f32 = 10.0;
const LINK: f32 = 11.0;
const NAME: f32 = 12.0;

/// a city event: every fact is zero but the phase and, for an insertion, the entity type
fn city(lua: &Lua, phase: f32, entity_type: f32, identity: &str) -> Table {
    let mut words = [0u32; 16];
    words[3] = entity_type.to_bits();
    words[11] = phase.to_bits();
    ask_words(lua, words, identity)
}

fn child_insert(lua: &Lua, parent: &str, pred: Option<&str>) -> String {
    let identity = pred.map_or(parent.to_string(), |pred| format!("{parent} {pred}"));
    let table = city(lua, CHILD_INSERT, 21.0, &identity);
    assert_eq!(result(&table), "placed", "{:?}", table.get::<Option<String>>("reason"));
    name(&table)
}

/// the keys the host serves for a city, from its first building to the end of the list, exactly as the campaign's driver asks: FIRST names the parent, NEXT names the parent and the building it stands at
fn listed(lua: &Lua, parent: &str) -> Vec<String> {
    let first = city(lua, CHILD_FIRST, 0.0, parent);
    assert_eq!(result(&first), "next", "{:?}", first.get::<Option<String>>("reason"));
    let mut keys = Vec::new();
    let mut at = name(&first);
    while !at.is_empty() {
        keys.push(at.clone());
        let next = city(lua, CHILD_NEXT, 0.0, &format!("{parent} {at}"));
        assert_eq!(result(&next), "next", "{:?}", next.get::<Option<String>>("reason"));
        at = name(&next);
    }
    keys
}

#[test]
fn a_city_list_is_served_head_first_from_the_hosts_own_order_and_a_removal_keeps_the_others_in_order() {
    let (lua, roster) = host(registry());
    let place = insert(&lua, [1100.0, 3.0, 1100.0], 22.0, "");
    assert!(listed(&lua, &place).is_empty(), "a city with no building lists none");
    // the campaign creates a building with no predecessor: it goes to the HEAD, so the newest is served first; a predecessor puts it right after that building
    let (a, b) = (child_insert(&lua, &place, None), child_insert(&lua, &place, None));
    assert_eq!(listed(&lua, &place), vec![b.clone(), a.clone()], "head first: the newest insertion");
    let c = child_insert(&lua, &place, Some(&a));
    assert_eq!(listed(&lua, &place), vec![b.clone(), a.clone(), c.clone()], "after its predecessor");
    let d = child_insert(&lua, &place, Some(&b));
    assert_eq!(listed(&lua, &place), vec![b.clone(), d.clone(), a.clone(), c.clone()]);
    assert_eq!(roster.borrow().child_type(a.parse().unwrap()), Some(21));
    // another city keeps its own list, and the sector's order of the cities is untouched by the buildings
    let other = insert(&lua, [1500.0, 3.0, 1500.0], 22.0, "");
    let lone = child_insert(&lua, &other, None);
    assert_eq!(listed(&lua, &other), vec![lone.clone()]);
    assert_eq!(served(&lua), vec![place.clone(), other.clone()]);
    // a removal leaves the others in their order, and the removed key never names anything again
    assert_eq!(result(&city(&lua, CHILD_REMOVE, 0.0, &format!("{place} {d}"))), "removed");
    assert_eq!(listed(&lua, &place), vec![b.clone(), a.clone(), c.clone()]);
    let ended = city(&lua, CHILD_NEXT, 0.0, &format!("{place} {d}"));
    assert_eq!(result(&ended), "refused");
    assert!(
        reason(&ended).contains("[roster.child]") && reason(&ended).contains("has ended"),
        "{}",
        reason(&ended)
    );
    // the buildings leave before their city does; the city's own placement is refused by name while any is listed, and then leaves
    let early = remove(&lua, &place);
    assert_eq!(result(&early), "refused");
    assert!(reason(&early).contains("city children that are still listed"), "{}", reason(&early));
    for child in [&b, &a, &c] {
        assert_eq!(result(&city(&lua, CHILD_REMOVE, 0.0, &format!("{place} {child}"))), "removed");
    }
    assert_eq!(result(&remove(&lua, &place)), "removed");
}

#[test]
fn a_city_event_is_validated_against_the_live_roster_and_its_parent_before_anything_is_served_or_changed() {
    let (lua, roster) = host(registry());
    let place = insert(&lua, [1100.0, 3.0, 1100.0], 22.0, "");
    let other = insert(&lua, [1500.0, 3.0, 1500.0], 22.0, "");
    let a = child_insert(&lua, &place, None);
    let b = child_insert(&lua, &other, None);
    let ended_parent = insert(&lua, [1200.0, 3.0, 1200.0], 22.0, "");
    assert_eq!(result(&remove(&lua, &ended_parent)), "removed");
    // positive first: the listed building of its own parent
    assert_eq!(name(&city(&lua, CHILD_FIRST, 0.0, &place)), a);
    assert_eq!(result(&city(&lua, CHILD_NEXT, 0.0, &format!("{place} {a}"))), "next");
    let held = roster.borrow().len();
    // every other relation is a named refusal that serves nothing and changes nothing
    let cases: [(&str, f32, String, &str); 14] = [
        ("an unknown parent", CHILD_FIRST, "999".into(), "never issued"),
        ("an ended parent", CHILD_FIRST, ended_parent.clone(), "has ended"),
        ("a child as the parent", CHILD_FIRST, a.clone(), "city child"),
        ("text as the parent", CHILD_FIRST, "eech-world:1".into(), "not a key this host issued"),
        ("no parent", CHILD_FIRST, String::new(), "not a key this host issued"),
        ("two keys for a first", CHILD_FIRST, format!("{place} {a}"), "identity place"),
        ("one key for a next", CHILD_NEXT, place.clone(), "identity place"),
        ("a child of another parent", CHILD_NEXT, format!("{place} {b}"), "listed under"),
        (
            "a sector entry as the child",
            CHILD_NEXT,
            format!("{place} {other}"),
            "sector entry, not a city child",
        ),
        ("an unknown child", CHILD_NEXT, format!("{place} 999"), "never issued"),
        ("a removal under another parent", CHILD_REMOVE, format!("{other} {a}"), "listed under"),
        ("a predecessor of another parent", CHILD_INSERT, format!("{place} {b}"), "listed under"),
        (
            "a predecessor that is a sector entry",
            CHILD_INSERT,
            format!("{place} {other}"),
            "sector entry, not a city child",
        ),
        ("an insertion under an ended parent", CHILD_INSERT, ended_parent.clone(), "has ended"),
    ];
    for (what, phase, identity, why) in cases {
        let refused = city(&lua, phase, if phase == CHILD_INSERT { 21.0 } else { 0.0 }, &identity);
        assert_eq!(result(&refused), "refused", "{what}");
        assert!(reason(&refused).contains(why), "{what}: {}", reason(&refused));
        assert_eq!(name(&refused), "", "{what}: a refusal serves nothing");
    }
    assert_eq!(roster.borrow().len(), held);
    assert_eq!(listed(&lua, &place), vec![a.clone()], "the lists are as they were");
    assert_eq!(listed(&lua, &other), vec![b.clone()]);
    // a boot ends every list and every key
    assert_eq!(result(&ask(&lua, event_facts(RESET, [0.0; 3], 0.0), "")), "cleared");
    let gone = city(&lua, CHILD_FIRST, 0.0, &place);
    assert!(reason(&gone).contains("has ended"), "{}", reason(&gone));
}

#[test]
fn a_buildings_two_placements_and_its_object_identity_are_one_record_that_follows_a_relink() {
    let (lua, roster) = host(registry());
    let place = insert(&lua, [1100.0, 3.0, 1100.0], 22.0, "");
    // the campaign's creation order: the building joins its city's list, then its sector list; the second placement is declared the same entity
    let child = child_insert(&lua, &place, None);
    let sector = insert(&lua, [1150.0, 3.0, 1150.0], 21.0, "");
    let linked = city(&lua, LINK, 0.0, &format!("{sector} {child}"));
    assert_eq!((result(&linked), name(&linked)), ("placed".to_string(), child.clone()));
    let record = roster.borrow().record(sector.parse().unwrap()).unwrap();
    assert_eq!(roster.borrow().record(child.parse().unwrap()), Some(record));
    assert_eq!(roster.borrow().record_members(record), 2);
    // the legacy object-report identity is issued later, lazily, once: it names the ONE record, whichever placement the campaign names it by
    let named = city(&lua, NAME, 0.0, &format!("{sector} eech-world:object:7"));
    assert_eq!((result(&named), name(&named)), ("placed".to_string(), sector.clone()));
    assert_eq!(roster.borrow().object_of(child.parse().unwrap()), Some("eech-world:object:7"));
    assert_eq!(roster.borrow().object_record("eech-world:object:7"), Some(record));
    // an identity names one record and a record has one identity
    for (what, identity, why) in [
        ("the same identity again", format!("{place} eech-world:object:7"), "names the record"),
        ("another identity for the record", format!("{child} eech-world:object:8"), "is named already"),
        ("an empty identity", format!("{child} "), "empty"),
        ("a control character", format!("{child} eech-world:object:9\n"), "control character"),
        ("an ended placement", "999 eech-world:object:9".to_string(), "never issued"),
    ] {
        let refused = city(&lua, NAME, 0.0, &identity);
        assert_eq!(result(&refused), "refused", "{what}");
        assert!(
            reason(&refused).contains("[roster.name]") && reason(&refused).contains(why),
            "{what}: {}",
            reason(&refused)
        );
    }
    // a relink of the building's sector placement: the new placement joins the child's record, which still holds the identity (the child never left it)
    assert_eq!(result(&remove(&lua, &sector)), "removed");
    assert_eq!(roster.borrow().record_members(record), 1);
    let again = insert(&lua, [1250.0, 3.0, 1250.0], 21.0, "");
    assert_eq!(result(&city(&lua, LINK, 0.0, &format!("{again} {child}"))), "placed");
    assert_eq!(roster.borrow().object_of(again.parse().unwrap()), Some("eech-world:object:7"));
    let joined = roster.borrow().record(again.parse().unwrap()).unwrap();
    assert_eq!(roster.borrow().record(child.parse().unwrap()), Some(joined));
    assert_eq!(roster.borrow().record_members(joined), 2);
    assert_eq!(roster.borrow().object_record("eech-world:object:7"), Some(joined));
    // a structure with no city child: its record ends with its placement, and the relinked placement is named again by the identity the campaign still holds
    let lone = insert(&lua, [1300.0, 3.0, 1300.0], 23.0, "");
    assert_eq!(result(&city(&lua, NAME, 0.0, &format!("{lone} eech-world:object:11"))), "placed");
    assert_eq!(result(&remove(&lua, &lone)), "removed");
    assert_eq!(
        roster.borrow().object_record("eech-world:object:11"),
        None,
        "the record and its identity end with the last placement"
    );
    let relinked = insert(&lua, [1350.0, 3.0, 1350.0], 23.0, "");
    assert_eq!(result(&city(&lua, NAME, 0.0, &format!("{relinked} eech-world:object:11"))), "placed");
    // a record holds one placement of each kind: the same pair again is the same record, and neither a second sector placement nor a second child can join it
    let again_pair = city(&lua, LINK, 0.0, &format!("{again} {child}"));
    assert_eq!(result(&again_pair), "placed", "{:?}", again_pair.get::<Option<String>>("reason"));
    let stranger = insert(&lua, [1420.0, 3.0, 1420.0], 21.0, "");
    let refused = city(&lua, LINK, 0.0, &format!("{stranger} {child}"));
    assert_eq!(result(&refused), "refused");
    assert!(
        reason(&refused).contains("[roster.link]") && reason(&refused).contains("already hold a sector placement"),
        "{}",
        reason(&refused)
    );
    let second_child = child_insert(&lua, &place, None);
    let refused = city(&lua, LINK, 0.0, &format!("{again} {second_child}"));
    assert_eq!(result(&refused), "refused");
    assert!(reason(&refused).contains("already hold a city child"), "{}", reason(&refused));
    // a placement the roster does not hold cannot be linked
    let refused = city(&lua, LINK, 0.0, &format!("999 {child}"));
    assert!(reason(&refused).contains("never issued"), "{}", reason(&refused));
}

#[test]
fn the_city_words_are_classified_without_floating_point_and_leave_the_callers_whole_state_unchanged() {
    const SIGNALING: u32 = 0x7fa0_0001;
    for mode in [Rounding::Nearest, Rounding::Up, Rounding::Down, Rounding::Zero] {
        for raised in [false, true] {
            let (lua, _) = host(registry());
            let place = insert(&lua, [1100.0, 3.0, 1100.0], 22.0, "");
            let _caller = ScopedRounding::set(mode).expect("a caller in this rounding mode");
            clear_exception_flags().expect("clear");
            if raised {
                raise_exception_flags().expect("raise");
            }
            let before = caller_state();
            let child = child_insert(&lua, &place, None);
            assert_eq!(listed(&lua, &place), vec![child.clone()]);
            assert_eq!(
                caller_state(),
                before,
                "{mode:?} raised={raised}: the caller's whole state after valid city events"
            );
            let word = |phase: f32, at: usize, value: u32| {
                let mut words = [0u32; 16];
                words[3] = 21.0f32.to_bits();
                words[11] = phase.to_bits();
                words[at] = value;
                words
            };
            let cases: [(&str, [u32; 16], &str); 5] = [
                ("a signaling type", word(CHILD_INSERT, 3, SIGNALING), "[roster.request]"),
                ("a fractional type", word(CHILD_INSERT, 3, 2.5f32.to_bits()), "[roster.request]"),
                ("a signaling unused fact", word(CHILD_INSERT, 5, SIGNALING), "is not zero"),
                ("a signaling fact of a first", word(CHILD_FIRST, 0, SIGNALING), "is not zero"),
                ("an infinite fact of a next", word(CHILD_NEXT, 14, 0x7f80_0000), "is not zero"),
            ];
            for (what, words, why) in cases {
                let refused = ask_words(&lua, words, &place);
                assert_eq!(result(&refused), "refused", "{what}");
                assert!(reason(&refused).contains(why), "{what}: {}", reason(&refused));
                assert_eq!(caller_state(), before, "{mode:?} raised={raised}: the caller's whole state after {what}");
            }
            // a city phase is classified from its bits too: a signaling phase word is the scan kernel's named refusal, not a city event
            let mut words = [0u32; 16];
            words[11] = SIGNALING;
            let refused = ask_words(&lua, words, &place);
            assert_eq!(result(&refused), "refused");
            assert_eq!(
                caller_state(),
                before,
                "{mode:?} raised={raised}: the caller's whole state after a signaling phase"
            );
        }
    }
}
