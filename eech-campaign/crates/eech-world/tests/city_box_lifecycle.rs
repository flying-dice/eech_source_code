//! A CITY's host-built bounding box across the city's lifetime (EEDCS-49, D3, REQUIRED_CITY_BOX_LIFECYCLE_RETENTION).
//!
//! The host builds a city's box when a building links (`set_city_box`) and the campaign only mirrors it. The box must outlive the city's sector PLACEMENT while the entity lives, because the shared
//! fixed-position setter relinks a city (a removal and a new insertion) and a diagnostic withdrawal of the city's state must not turn the campaign's mirror into the box's source. These tests drive the
//! REAL provider (`host.model`) with the REAL `SectorRoster`, speaking the questions the campaign's C driver sends (`mb_hostplacemap.h`, whose own half is checked in eech-engine-sys
//! `entity_dialogue`), and assert the externally meaningful facts: the box the host built is the box a later statement must carry, a record the host retains is named to the campaign and continued by the
//! relinked placement, a diagnostic withdrawal ends nothing, and only the campaign's release of the entity ends the record. The legacy path (a city the host never built a box for) is characterised too: it
//! ends with its last placement exactly as before.
//!
//! The box ARITHMETIC (the model store's scene bounds) is not repeated here: the host's own construction is the roster call the provider makes after it (`set_city_box`), and its arithmetic is pinned in
//! `model_target_point`.

use eech_world::movement::{Movers, SharedMovers};
use eech_world::roster::{CityState, FixedHold, SectorRoster, SharedRoster, ENTITY_CITY, ENTITY_CITY_BUILDING};
use eech_world::trees::{clear_exception_flags, exception_flags, fp_state, raise_exception_flags, rounding, FpState, Rounding, ScopedRounding};
use mlua::{Lua, Table};
use std::cell::RefCell;
use std::rc::Rc;

const SIDE: f32 = 1000.0;
const INSERT: f32 = 4.0;
const REMOVE: f32 = 5.0;
const CHILD_INSERT: f32 = 7.0;
const CHILD_REMOVE: f32 = 8.0;
const LINK: f32 = 11.0;
const CITY_STATE: f32 = 14.0;
/// the phase the campaign sends when the CITY ENTITY is released (the heap slot freed): the end of a record the host retained
const RELEASE: f32 = 16.0;
const SUB_TYPE_CITY: f32 = 4.0;

const CITY_AT: [f32; 3] = [1100.0, 3.0, 1100.0];
const INITIAL_MIN: [f32; 3] = [1.0e6, 1.0e6, 1.0e6];
const INITIAL_MAX: [f32; 3] = [-1.0e6, -1.0e6, -1.0e6];
const BUILT_MIN: [f32; 3] = [-40.0, 0.0, -30.0];
const BUILT_MAX: [f32; 3] = [55.0, 12.0, 70.0];
const STALE_MAX: [f32; 3] = [55.0, 12.0, 71.0];

fn host() -> (Lua, Rc<RefCell<SectorRoster>>) {
    let lua = Lua::new();
    let host = lua.create_table().unwrap();
    host.set("args", lua.create_table().unwrap()).unwrap();
    eech_world::model_host::register(&lua, &host).unwrap();
    lua.globals().set("host", host).unwrap();
    lua.set_app_data(SharedMovers(Rc::new(RefCell::new(Movers::default()))));
    let roster = Rc::new(RefCell::new(SectorRoster::default()));
    lua.set_app_data(SharedRoster(roster.clone()));
    (lua, roster)
}

fn ask_words(lua: &Lua, words: [u32; 16], identity: &str) -> Table {
    let patterns = words.iter().map(|word| format!("0x{word:08x}")).collect::<Vec<_>>().join(",");
    let text = if identity.is_empty() { patterns } else { format!("{patterns} {identity}") };
    lua.globals().set("KEY", text).unwrap();
    lua.load(r#"return host.model ("entity_scan", 0, nil, nil, nil, KEY)"#).eval().unwrap()
}

fn result(table: &Table) -> String {
    table.get("result").unwrap()
}

fn name(table: &Table) -> String {
    table.get::<Option<String>>("name").unwrap().unwrap_or_default()
}

fn reason(table: &Table) -> String {
    table.get::<Option<String>>("reason").unwrap().unwrap_or_default()
}

fn event(lua: &Lua, phase: f32, entity_type: f32, identity: &str) -> Table {
    let mut words = [0u32; 16];
    words[3] = entity_type.to_bits();
    words[11] = phase.to_bits();
    ask_words(lua, words, identity)
}

/// a roster insertion: the position, the entity type and the sector side length
fn insert(lua: &Lua, entity_type: f32, at: [f32; 3]) -> String {
    let mut words = [0u32; 16];
    for (index, word) in at.iter().enumerate() {
        words[index] = word.to_bits();
    }
    words[3] = entity_type.to_bits();
    words[6] = SIDE.to_bits();
    words[11] = INSERT.to_bits();
    let table = ask_words(lua, words, "");
    assert_eq!(result(&table), "placed", "{}", reason(&table));
    name(&table)
}

fn remove(lua: &Lua, key: &str) -> Table {
    event(lua, REMOVE, 0.0, key)
}

fn child_insert(lua: &Lua, list: &str) -> String {
    let table = event(lua, CHILD_INSERT, ENTITY_CITY_BUILDING as f32, list);
    assert_eq!(result(&table), "placed", "{}", reason(&table));
    name(&table)
}

fn child_remove(lua: &Lua, list: &str, child: &str) -> Table {
    event(lua, CHILD_REMOVE, 0.0, &format!("{list} {child}"))
}

fn link(lua: &Lua, sector: &str, list: &str) -> Table {
    event(lua, LINK, 0.0, &format!("{sector} {list}"))
}

fn release(lua: &Lua, identity: &str) -> Table {
    event(lua, RELEASE, 0.0, identity)
}

/// the city state report of the placement `key`: position, box minimum and maximum, the city's declared sub type
fn city_state(lua: &Lua, key: &str, position: [f32; 3], min: [f32; 3], max: [f32; 3]) -> Table {
    let mut words = [0u32; 16];
    for (index, word) in position.iter().chain(&min).chain(&max).enumerate() {
        words[index] = word.to_bits();
    }
    words[10] = SUB_TYPE_CITY.to_bits();
    words[11] = CITY_STATE.to_bits();
    ask_words(lua, words, key)
}

fn state_of(roster: &Rc<RefCell<SectorRoster>>, key: &str) -> FixedHold {
    roster.borrow().placement_fixed(key.parse().unwrap(), false).unwrap().2
}

fn bits(words: [f32; 3]) -> [u32; 3] {
    words.map(f32::to_bits)
}

fn built(position: [f32; 3]) -> CityState {
    CityState {
        position: bits(position),
        box_min: bits(BUILT_MIN),
        box_max: bits(BUILT_MAX),
        sub_type: 4,
    }
}

/// the campaign's creation order for a city, then one building whose link makes the HOST build the box (`set_city_box`, the call the provider makes after the model store's bounds), then the building's unlisting:
/// the city is empty again and holds the box the host built. Returns the placement's key (which is also the name of the city's record).
fn empty_city_with_a_host_built_box(lua: &Lua, roster: &Rc<RefCell<SectorRoster>>) -> String {
    let key = insert(lua, ENTITY_CITY as f32, CITY_AT);
    let stated = city_state(lua, &key, CITY_AT, INITIAL_MIN, INITIAL_MAX);
    assert_eq!(result(&stated), "placed", "{}", reason(&stated));
    let child = child_insert(lua, &key);
    roster
        .borrow_mut()
        .set_city_box(key.parse().unwrap(), bits(BUILT_MIN), bits(BUILT_MAX))
        .expect("the host builds the box of a city it holds the state of");
    assert_eq!(result(&child_remove(lua, &key, &child)), "removed");
    assert_eq!(state_of(roster, &key), FixedHold::City(built(CITY_AT)));
    key
}

#[test]
fn a_citys_host_built_box_outlives_its_sector_placement_and_the_relinked_placement_continues_the_record() {
    let (lua, roster) = host();
    let first = empty_city_with_a_host_built_box(&lua, &roster);
    // the shared fixed-position setter removes the placement: the host answers the name of the record it keeps, and the placement itself has ended
    let removed = remove(&lua, &first);
    assert_eq!(result(&removed), "removed", "{}", reason(&removed));
    assert_eq!(name(&removed), first, "the host names the record that holds the box it built");
    assert!(roster.borrow().placement_fixed(first.parse().unwrap(), false).is_err());
    // the new placement is a new key, declared to continue the record by that name; it holds the host's box at once, before the campaign states anything
    let second = insert(&lua, ENTITY_CITY as f32, [2100.0, 3.0, 1100.0]);
    assert_ne!(first, second, "a relink is a new key");
    let linked = link(&lua, &second, &first);
    assert_eq!((result(&linked), name(&linked)), ("placed".to_string(), first.clone()), "{}", reason(&linked));
    assert_eq!(state_of(&roster, &second), FixedHold::City(built(CITY_AT)));
    // the insertion then states the new position with the campaign's mirror of the box: the mirror carries the host's box
    let moved = city_state(&lua, &second, [2100.0, 3.0, 1100.0], BUILT_MIN, BUILT_MAX);
    assert_eq!(result(&moved), "placed", "{}", reason(&moved));
    assert_eq!(state_of(&roster, &second), FixedHold::City(built([2100.0, 3.0, 1100.0])));
    // a mirror that differs is NOT the source of the box: refused by name, nothing changes
    let stale = city_state(&lua, &second, [2100.0, 3.0, 1100.0], BUILT_MIN, STALE_MAX);
    assert_eq!(result(&stale), "refused");
    assert!(reason(&stale).contains("not the one the host holds"), "{}", reason(&stale));
    assert_eq!(state_of(&roster, &second), FixedHold::City(built([2100.0, 3.0, 1100.0])));
    // a second relink keeps the same record and the same name
    assert_eq!(name(&remove(&lua, &second)), first);
    let third = insert(&lua, ENTITY_CITY as f32, [1100.0, 3.0, 2100.0]);
    assert_eq!(result(&link(&lua, &third, &first)), "placed");
    assert_eq!(state_of(&roster, &third), FixedHold::City(built([2100.0, 3.0, 1100.0])));
}

#[test]
fn a_diagnostic_withdrawal_keeps_the_hosts_box_and_a_restoring_statement_must_carry_it() {
    let (lua, roster) = host();
    let key = empty_city_with_a_host_built_box(&lua, &roster);
    // the host's own withdrawal retires the STATE; the box it built is not the campaign's to restate differently
    roster.borrow_mut().withdraw_fixed(key.parse().unwrap()).unwrap();
    assert_eq!(state_of(&roster, &key), FixedHold::Withdrawn);
    let stale = city_state(&lua, &key, CITY_AT, BUILT_MIN, STALE_MAX);
    assert_eq!(result(&stale), "refused", "a restoration is not a second source for the box");
    assert!(reason(&stale).contains("not the one the host holds"), "{}", reason(&stale));
    assert_eq!(state_of(&roster, &key), FixedHold::Withdrawn, "a refused statement changes nothing");
    let initial = city_state(&lua, &key, CITY_AT, INITIAL_MIN, INITIAL_MAX);
    assert_eq!(result(&initial), "refused", "the campaign's initial box is not the box the host built");
    // the legitimate restoration: the mirror carries the box the host built
    let restored = city_state(&lua, &key, CITY_AT, BUILT_MIN, BUILT_MAX);
    assert_eq!(result(&restored), "placed", "{}", reason(&restored));
    assert_eq!(state_of(&roster, &key), FixedHold::City(built(CITY_AT)));
    // a withdrawal does not end the record or the retained box: the placement leaves, the record is kept, the relinked placement still holds the withdrawn state and the host's box
    roster.borrow_mut().withdraw_fixed(key.parse().unwrap()).unwrap();
    assert_eq!(name(&remove(&lua, &key)), key);
    let second = insert(&lua, ENTITY_CITY as f32, CITY_AT);
    assert_eq!(result(&link(&lua, &second, &key)), "placed");
    assert_eq!(state_of(&roster, &second), FixedHold::Withdrawn);
    assert_eq!(result(&city_state(&lua, &second, CITY_AT, BUILT_MIN, STALE_MAX)), "refused");
    assert_eq!(result(&city_state(&lua, &second, CITY_AT, BUILT_MIN, BUILT_MAX)), "placed");
}

#[test]
fn only_the_release_of_the_city_ends_a_retained_record_and_a_malformed_release_changes_nothing() {
    let (lua, roster) = host();
    let key = empty_city_with_a_host_built_box(&lua, &roster);
    assert_eq!(name(&remove(&lua, &key)), key);
    // a release names one key and only the record a city placement left: refused by name before anything changes
    let stranger = insert(&lua, 17.0, [1500.0, 3.0, 1500.0]);
    assert_eq!(result(&remove(&lua, &stranger)), "removed");
    for (what, identity, why) in [
        ("an empty identity", String::new(), "key"),
        ("a second key", format!("{key} {key}"), "key"),
        ("a key never issued", "9999".to_string(), "never issued"),
        ("a record that is no city's", stranger.clone(), "has ended"),
    ] {
        let refused = release(&lua, &identity);
        assert_eq!(result(&refused), "refused", "{what}");
        assert!(reason(&refused).contains(why), "{what}: {}", reason(&refused));
    }
    // a record that still holds a placement is not released: the campaign removes the placement first
    let second = insert(&lua, ENTITY_CITY as f32, CITY_AT);
    assert_eq!(result(&link(&lua, &second, &key)), "placed");
    let held = release(&lua, &key);
    assert_eq!(result(&held), "refused");
    assert!(reason(&held).contains("still holds a sector placement"), "{}", reason(&held));
    assert_eq!(
        state_of(&roster, &second),
        FixedHold::City(built(CITY_AT)),
        "the refused release changed nothing"
    );
    assert_eq!(name(&remove(&lua, &second)), key);
    // the release ends the record: nothing continues it and it is not released twice
    let ended = release(&lua, &key);
    assert_eq!((result(&ended), name(&ended)), ("removed".to_string(), String::new()), "{}", reason(&ended));
    let third = insert(&lua, ENTITY_CITY as f32, CITY_AT);
    let late = link(&lua, &third, &key);
    assert_eq!(result(&late), "refused");
    assert!(reason(&late).contains("has ended"), "{}", reason(&late));
    let twice = release(&lua, &key);
    assert_eq!(result(&twice), "refused");
    assert!(reason(&twice).contains("has ended"), "{}", reason(&twice));
}

#[test]
fn a_release_while_buildings_are_still_listed_ends_the_record_with_the_last_building() {
    let (lua, roster) = host();
    let key = insert(&lua, ENTITY_CITY as f32, CITY_AT);
    assert_eq!(result(&city_state(&lua, &key, CITY_AT, INITIAL_MIN, INITIAL_MAX)), "placed");
    let (a, b) = (child_insert(&lua, &key), child_insert(&lua, &key));
    roster
        .borrow_mut()
        .set_city_box(key.parse().unwrap(), bits(BUILT_MIN), bits(BUILT_MAX))
        .unwrap();
    // the heap release frees the city's placement while two buildings are listed: the list and the record stay for them, and the campaign then releases the entity
    assert_eq!(name(&remove(&lua, &key)), "", "buildings are still listed: the record lives by its list");
    assert_eq!(result(&release(&lua, &key)), "removed");
    assert_eq!(result(&child_remove(&lua, &key, &a)), "removed");
    let last = child_remove(&lua, &key, &b);
    assert_eq!(
        (result(&last), name(&last)),
        ("removed".to_string(), String::new()),
        "a released city retains nothing"
    );
    let ended = event(&lua, CHILD_INSERT, ENTITY_CITY_BUILDING as f32, &key);
    assert!(reason(&ended).contains("has ended"), "{}", reason(&ended));
}

#[test]
fn the_last_building_leaving_an_unplaced_city_retains_the_record_that_holds_the_hosts_box() {
    let (lua, roster) = host();
    let key = insert(&lua, ENTITY_CITY as f32, CITY_AT);
    assert_eq!(result(&city_state(&lua, &key, CITY_AT, INITIAL_MIN, INITIAL_MAX)), "placed");
    let child = child_insert(&lua, &key);
    roster
        .borrow_mut()
        .set_city_box(key.parse().unwrap(), bits(BUILT_MIN), bits(BUILT_MAX))
        .unwrap();
    // the placement leaves while the building stays listed (the record lives by its list), then the building is unlisted: only the box keeps the record, and the host names it
    assert_eq!(name(&remove(&lua, &key)), "");
    let last = child_remove(&lua, &key, &child);
    assert_eq!((result(&last), name(&last)), ("removed".to_string(), key.clone()), "{}", reason(&last));
    let second = insert(&lua, ENTITY_CITY as f32, CITY_AT);
    assert_eq!(result(&link(&lua, &second, &key)), "placed");
    assert_eq!(state_of(&roster, &second), FixedHold::City(built(CITY_AT)));
}

#[test]
fn a_city_the_host_never_built_a_box_for_ends_with_its_last_placement_as_it_always_did() {
    let (lua, roster) = host();
    // the legacy path: an empty city's relink is a removal and a new insertion, a new record that takes the campaign's statement (its own initial box) as it is
    let key = insert(&lua, ENTITY_CITY as f32, CITY_AT);
    assert_eq!(result(&city_state(&lua, &key, CITY_AT, INITIAL_MIN, INITIAL_MAX)), "placed");
    let removed = remove(&lua, &key);
    assert_eq!((result(&removed), name(&removed)), ("removed".to_string(), String::new()));
    let second = insert(&lua, ENTITY_CITY as f32, CITY_AT);
    let late = link(&lua, &second, &key);
    assert_eq!(result(&late), "refused");
    assert!(reason(&late).contains("has ended"), "{}", reason(&late));
    assert_eq!(state_of(&roster, &second), FixedHold::Never);
    // the campaign's statement of its initial box is accepted as before, and a city that did build a box then holds it from there
    assert_eq!(result(&city_state(&lua, &second, CITY_AT, INITIAL_MIN, INITIAL_MAX)), "placed");
    // nothing was retained, so a release names nothing
    let none = release(&lua, &key);
    assert_eq!(result(&none), "refused");
}

/// the caller's whole floating-point state as the roster call sees it: masks, rounding, raw control words and the raised status flags
fn caller_state() -> (FpState, Rounding, i32) {
    (fp_state().expect("readable"), rounding().expect("one of the four"), exception_flags())
}

#[test]
fn the_box_lifecycle_events_leave_the_callers_whole_floating_point_state_unchanged_and_a_malformed_release_changes_nothing() {
    const SIGNALING: u32 = 0x7fa0_0001;
    for mode in [Rounding::Nearest, Rounding::Up, Rounding::Down, Rounding::Zero] {
        for raised in [false, true] {
            let (lua, roster) = host();
            let _caller = ScopedRounding::set(mode).expect("a caller in this rounding mode");
            clear_exception_flags().expect("clear");
            if raised {
                raise_exception_flags().expect("raise");
            }
            let before = caller_state();
            let key = empty_city_with_a_host_built_box(&lua, &roster);
            assert_eq!(name(&remove(&lua, &key)), key);
            assert_eq!(
                caller_state(),
                before,
                "{mode:?} raised={raised}: the caller's whole state after the statement, the construction and the retaining removal"
            );
            // a release states nothing but its phase and its key: a signaling or infinite fact is refused by name before any floating-point use, and the record is untouched
            for (what, at, value) in [("a signaling type fact", 3usize, SIGNALING), ("an infinite unused fact", 14, 0x7f80_0000)] {
                let mut words = [0u32; 16];
                words[11] = RELEASE.to_bits();
                words[at] = value;
                let refused = ask_words(&lua, words, &key);
                assert_eq!(result(&refused), "refused", "{what}");
                assert!(reason(&refused).contains("is not zero"), "{what}: {}", reason(&refused));
                assert_eq!(caller_state(), before, "{mode:?} raised={raised}: the caller's whole state after {what}");
            }
            let second = insert(&lua, ENTITY_CITY as f32, CITY_AT);
            assert_eq!(
                result(&link(&lua, &second, &key)),
                "placed",
                "the record the refused releases left is still continued"
            );
            assert_eq!(name(&remove(&lua, &second)), key);
            assert_eq!(result(&release(&lua, &key)), "removed");
            assert_eq!(
                caller_state(),
                before,
                "{mode:?} raised={raised}: the caller's whole state after the continuation and the release"
            );
        }
    }
}

#[test]
fn a_live_city_with_buildings_still_relinks_and_a_released_one_is_never_continued_while_its_children_drain() {
    let (lua, roster) = host();
    let key = insert(&lua, ENTITY_CITY as f32, CITY_AT);
    assert_eq!(result(&city_state(&lua, &key, CITY_AT, INITIAL_MIN, INITIAL_MAX)), "placed");
    let child = child_insert(&lua, &key);
    roster
        .borrow_mut()
        .set_city_box(key.parse().unwrap(), bits(BUILT_MIN), bits(BUILT_MAX))
        .unwrap();
    // the live relink the legacy path always had: buildings are listed, so the record continues by its list (the supported positive)
    assert_eq!(name(&remove(&lua, &key)), "", "buildings are listed: the record lives by its list");
    let second = insert(&lua, ENTITY_CITY as f32, [2100.0, 3.0, 1100.0]);
    assert_eq!(result(&link(&lua, &second, &key)), "placed");
    assert_eq!(state_of(&roster, &second), FixedHold::City(built(CITY_AT)));
    assert_eq!(name(&remove(&lua, &second)), "");
    // the city entity is released while the building is still listed: the record is physically ended and only drains
    assert_eq!(result(&release(&lua, &key)), "removed");
    let third = insert(&lua, ENTITY_CITY as f32, CITY_AT);
    let record_before = roster.borrow().record(third.parse().unwrap());
    let late = link(&lua, &third, &key);
    assert_eq!(result(&late), "refused", "a released record is not continued, even while a child is listed");
    assert!(reason(&late).contains("released"), "{}", reason(&late));
    assert_eq!(state_of(&roster, &third), FixedHold::Never, "the refused continuation transferred no state");
    assert_eq!(
        roster.borrow().record(third.parse().unwrap()),
        record_before,
        "the placement keeps its own record"
    );
    // the listed child still drains by the list's name, which names nothing new
    let drained = child_remove(&lua, &key, &child);
    assert_eq!(
        (result(&drained), name(&drained)),
        ("removed".to_string(), String::new()),
        "{}",
        reason(&drained)
    );
    let ended = link(&lua, &third, &key);
    assert!(reason(&ended).contains("has ended"), "{}", reason(&ended));
}

#[test]
fn a_parked_city_record_is_continued_only_by_a_placement_that_is_a_city() {
    let (lua, roster) = host();
    let key = empty_city_with_a_host_built_box(&lua, &roster);
    assert_eq!(name(&remove(&lua, &key)), key);
    let stranger = insert(&lua, 17.0, [1500.0, 3.0, 1500.0]);
    let refused = link(&lua, &stranger, &key);
    assert_eq!(result(&refused), "refused", "a placement of another family does not inherit a city's state");
    assert!(reason(&refused).contains("not a city"), "{}", reason(&refused));
    assert_eq!(state_of(&roster, &stranger), FixedHold::Never);
    assert_eq!(roster.borrow().record(stranger.parse().unwrap()), Some(stranger.parse().unwrap()));
    assert!(roster.borrow().parked(key.parse().unwrap()), "the refusal left the record parked");
    // the city placement still continues it, with the host's box
    let second = insert(&lua, ENTITY_CITY as f32, CITY_AT);
    assert_eq!(result(&link(&lua, &second, &key)), "placed");
    assert_eq!(state_of(&roster, &second), FixedHold::City(built(CITY_AT)));
}
