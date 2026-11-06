//! The host's sector roster through the reference host's real provider (EEDCS-49, D3, sixth slice): which entities stand in which sector and in what order a scan visits them.
//!
//! The campaign reports each insertion into and removal from a sector list (the one funnel `en_list.c`) and the host answers with a key it issued; a scan then asks the host for the first entry of a
//! sector and for what follows an entry, and the host serves them from ITS order. These tests drive the REAL provider (`host.model`) with the REAL `SectorRoster` and the REAL `Movers` registry, speaking
//! the questions the campaign's C driver sends (`mb_hostscan.h`, `mb_hostentity.h`, whose own half is checked in eech-engine-sys `entity_dialogue`), and assert the externally meaningful facts: the order
//! (oldest insertion first, a relink goes to the end, mixed fixed and mobile families), the sector a hosted mobile is placed in (its own record's, never the campaign's word), and that an ended, unknown or
//! misplaced entry is refused by name and never skipped or served as the end of a list.

use eech_world::movement::{Kind, Movers, Pose, SharedMovers};
use eech_world::roster::{SectorRoster, SharedRoster};
use mlua::{Lua, Table};
use std::cell::RefCell;
use std::rc::Rc;

const SIDE: f32 = 1000.0;
const LEVEL: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
/// the phases of the question: the scan's own and the roster's events
const BEGIN: f32 = 0.0;
const ENTRY: f32 = 1.0;
const RESOLVED: f32 = 3.0;
const INSERT: f32 = 4.0;
const REMOVE: f32 = 5.0;
const RESET: f32 = 6.0;

fn pattern(float: f32) -> String {
    format!("0x{:08x}", float.to_bits())
}

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
    let patterns = facts.iter().map(|word| pattern(*word)).collect::<Vec<_>>().join(",");
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
