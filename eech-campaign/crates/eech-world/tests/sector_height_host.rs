//! The HOST's sector high-water height (EEDCS-49, D3, Task49): the one physical word of each sector, held by the host's roster, raised at the ARRIVAL of a fixed structure, restored from a session and read by the campaign.
//!
//! These tests run the REAL provider (`eech_world::model_host`) over the REAL roster, the real model store of a minimal installation (scene 1 is a box whose top is 1.0, every other scene slot is the original's zero
//! bounding box) and the real raise kernel (`csrc/model_step.c`, whose expression is `csrc/sector_height_raise.h`, the text the real-engine control runs beside the original response). The campaign's half is
//! `csrc/eech_entity_dialogue_check.c` (the request and the checks of the reply) and `csrc/eech_sector_height_check.c` (the real getter, pack and unpack over a scripted provider); the public driver, scripted engine and
//! real roster together are `tests/entity_authority.rs`. The original's behaviour the raise reproduces is characterised by the 179-expectation control (`eech-engine-sys`, `sector_height`, original-solver profile).
//!
//! What is pinned: the arrival is the FIRST state statement under a sector placement's key after its insertion (a later statement, a setter or a restore of the state never raises); a structure with the invalid shape
//! (0) never raises; the comparison is the original's strict greater-than against the word the host ACTUALLY holds (a zero history never stores a negative or zero sum, a restored negative word can be raised, a restored
//! NaN or infinity is never exceeded); the restore stores ANY raw word and the read answers exactly it; a boot's clear ends every word; every refusal is named and changes nothing.
#[path = "support/model_root.rs"]
mod model_root;

use mlua::Lua;

const FIXED_STATE: u32 = 0x4150_0000;
const CITY_STATE: u32 = 0x4160_0000;
const RESET: u32 = 0x40c0_0000;
const SECTOR_HEIGHT: u32 = 0x4188_0000;
const SECTOR_RESTORE: u32 = 0x4190_0000;
const OBJECT_TYPE: f32 = 17.0;
const CITY_BUILDING_TYPE: f32 = 6.0;
const CITY_TYPE: f32 = 5.0;
const SUB_MILITARY: f32 = 12.0;
const SUB_CITY: f32 = 4.0;

type Roster = std::rc::Rc<std::cell::RefCell<eech_world::roster::SectorRoster>>;

struct World {
    lua: Lua,
    roster: Roster,
    _root: model_root::Root,
}

fn world() -> World {
    let root = model_root::Root::minimal();
    let lua = Lua::new();
    let host = lua.create_table().expect("a table");
    let args = lua.create_table().expect("a table");
    args.set("root", root.dir.to_string_lossy().into_owned()).expect("root");
    host.set("args", args).expect("args");
    eech_world::model_host::register(&lua, &host).expect("the provider registers");
    lua.globals().set("host", host).expect("host");
    lua.globals().set("ROOT", root.dir.to_string_lossy().into_owned()).expect("root text");
    let roster = std::rc::Rc::new(std::cell::RefCell::new(eech_world::roster::SectorRoster::default()));
    lua.set_app_data(eech_world::roster::SharedRoster(roster.clone()));
    World { lua, roster, _root: root }
}

struct Answer {
    result: String,
    reason: String,
    words: Vec<u32>,
}

fn ask(world: &World, words: [u32; 16], identity: &str) -> Answer {
    let patterns = words.iter().map(|word| format!("0x{word:08x}")).collect::<Vec<_>>().join(",");
    world
        .lua
        .globals()
        .set("FACTS", if identity.is_empty() { patterns } else { format!("{patterns} {identity}") })
        .expect("facts");
    let table: mlua::Table = world
        .lua
        .load(r#"return host.model_provider (host.model_store (ROOT)) ("entity_scan", 0, nil, nil, nil, FACTS)"#)
        .eval()
        .expect("the provider answers");
    let words = table
        .get::<Option<String>>("p")
        .expect("p")
        .map(|text| {
            text.split(',')
                .map(|token| u32::from_str_radix(token.trim_start_matches("0x"), 16).expect("a pattern"))
                .collect()
        })
        .unwrap_or_default();
    Answer {
        result: table.get("result").expect("result"),
        reason: table.get::<Option<String>>("reason").expect("reason").unwrap_or_default(),
        words,
    }
}

fn fixed(position: [f32; 3], entity_type: f32, shape: f32) -> [u32; 16] {
    let mut words = [0u32; 16];
    for (at, word) in position.iter().enumerate() {
        words[at] = word.to_bits();
    }
    words[3] = entity_type.to_bits();
    words[4] = 0.5f32.to_bits();
    words[5] = shape.to_bits();
    words[10] = SUB_MILITARY.to_bits();
    words[11] = FIXED_STATE;
    words
}

fn city(position: [f32; 3], shape: f32) -> [u32; 16] {
    let mut words = [0u32; 16];
    for (at, word) in position.iter().enumerate() {
        words[at] = word.to_bits();
    }
    for (at, word) in [-30.0f32, -5.0, -40.0, 50.0, 12.0, 60.0].iter().enumerate() {
        words[3 + at] = word.to_bits();
    }
    words[9] = shape.to_bits();
    words[10] = SUB_CITY.to_bits();
    words[11] = CITY_STATE;
    words
}

fn question(phase: u32, x: f32, z: f32, restored: Option<u32>) -> [u32; 16] {
    let mut words = [0u32; 16];
    words[11] = phase;
    words[12] = x.to_bits();
    words[13] = z.to_bits();
    if let Some(word) = restored {
        words[0] = word;
    }
    words
}

/// the host's word of sector (x, z), asked through the provider as the campaign asks it
fn read(world: &World, x: f32, z: f32) -> u32 {
    let answer = ask(world, question(SECTOR_HEIGHT, x, z, None), "");
    assert_eq!(answer.result, "height", "{}", answer.reason);
    assert_eq!(answer.words.len(), 3);
    assert_eq!(
        (answer.words[0], answer.words[1]),
        (x.to_bits(), z.to_bits()),
        "the answer names the sector asked about"
    );
    answer.words[2]
}

fn restore(world: &World, x: f32, z: f32, word: u32) {
    let answer = ask(world, question(SECTOR_RESTORE, x, z, Some(word)), "");
    assert_eq!(answer.result, "restored", "{}", answer.reason);
    assert_eq!(answer.words, vec![x.to_bits(), z.to_bits(), word], "the host holds exactly the bits stated");
}

/// a sector placement of `entity_type` in sector (0, 0), stated at once: the insertion then the nested state statement, the order of `place_insert`; the key
fn arrive(world: &World, entity_type: f32, position: [f32; 3], shape: f32) -> String {
    let key = world.roster.borrow_mut().insert((0, 0), entity_type as i32, None).to_string();
    let answer = ask(world, fixed(position, entity_type, shape), &key);
    assert_eq!(answer.result, "placed", "{}", answer.reason);
    key
}

const ZERO: u32 = 0;

#[test]
fn a_fresh_sector_holds_the_zero_word_and_an_arrival_raises_it_by_the_originals_strict_greater_than() {
    let w = world();
    assert_eq!(read(&w, 0.0, 0.0), ZERO, "a sector the host holds no word for is the zero fill of a new sector");
    assert_eq!(read(&w, 3.0, 7.0), ZERO);
    // shape 1 is a box whose top is 1.0: position.y + 1.0
    arrive(&w, OBJECT_TYPE, [10.0, 100.0, 20.0], 1.0);
    assert_eq!(read(&w, 0.0, 0.0), 101.0f32.to_bits());
    assert_eq!(read(&w, 1.0, 0.0), ZERO, "no other sector changed");
    // a lower arrival leaves it, an equal one leaves it, a higher one raises it (shape 8 is a slot no file defines: the zero box)
    arrive(&w, OBJECT_TYPE, [10.0, 50.0, 20.0], 1.0);
    assert_eq!(read(&w, 0.0, 0.0), 101.0f32.to_bits());
    arrive(&w, OBJECT_TYPE, [10.0, 101.0, 20.0], 8.0);
    assert_eq!(read(&w, 0.0, 0.0), 101.0f32.to_bits());
    arrive(&w, OBJECT_TYPE, [10.0, 500.0, 20.0], 8.0);
    assert_eq!(read(&w, 0.0, 0.0), 500.0f32.to_bits());
    // a removal never lowers it
    let key = arrive(&w, OBJECT_TYPE, [10.0, 900.0, 20.0], 1.0);
    assert_eq!(read(&w, 0.0, 0.0), 901.0f32.to_bits());
    w.roster.borrow_mut().remove(key.parse().expect("a key")).expect("removed");
    assert_eq!(
        read(&w, 0.0, 0.0),
        901.0f32.to_bits(),
        "a destroyed or moved tall structure keeps its clearance"
    );
}

#[test]
fn only_the_arrival_raises_a_later_statement_under_the_same_key_never_does() {
    let w = world();
    let key = arrive(&w, OBJECT_TYPE, [10.0, 100.0, 20.0], 1.0);
    assert_eq!(read(&w, 0.0, 0.0), 101.0f32.to_bits());
    // a setter, a kill's destroyed shape or a restore of the state restates it: the original never re-evaluates the height
    let again = ask(&w, fixed([10.0, 900.0, 20.0], OBJECT_TYPE, 1.0), &key);
    assert_eq!(again.result, "placed", "{}", again.reason);
    assert_eq!(read(&w, 0.0, 0.0), 101.0f32.to_bits());
    // a relink is a removal and a NEW insertion, which is a new arrival: the new key raises with the state it holds now
    w.roster.borrow_mut().remove(key.parse().expect("a key")).expect("removed");
    arrive(&w, OBJECT_TYPE, [10.0, 900.0, 20.0], 1.0);
    assert_eq!(read(&w, 0.0, 0.0), 901.0f32.to_bits());
}

#[test]
fn a_structure_without_a_shape_never_raises_and_a_sum_that_is_not_greater_than_the_zero_word_stores_nothing() {
    let w = world();
    // the invalid shape (0) never reads a bounding box
    arrive(&w, OBJECT_TYPE, [10.0, 700.0, 20.0], 0.0);
    assert_eq!(read(&w, 0.0, 0.0), ZERO);
    // over a zero history a negative sum, a zero sum and a negative zero sum store nothing: the floor is the zero word, not a clamp
    arrive(&w, OBJECT_TYPE, [10.0, -50.0, 20.0], 1.0);
    assert_eq!(read(&w, 0.0, 0.0), ZERO);
    arrive(&w, OBJECT_TYPE, [10.0, -1.0, 20.0], 1.0);
    assert_eq!(read(&w, 0.0, 0.0), ZERO, "-1 + 1 is zero: not greater than zero");
    arrive(&w, OBJECT_TYPE, [10.0, -0.0, 20.0], 8.0);
    assert_eq!(read(&w, 0.0, 0.0), ZERO, "negative zero is never stored");
    // the smallest subnormal is greater than zero and is stored exactly
    let mut subnormal = fixed([10.0, 0.0, 20.0], OBJECT_TYPE, 8.0);
    subnormal[1] = 1;
    let key = w.roster.borrow_mut().insert((0, 0), OBJECT_TYPE as i32, None).to_string();
    let answer = ask(&w, subnormal, &key);
    assert_eq!(answer.result, "placed", "{}", answer.reason);
    assert_eq!(read(&w, 0.0, 0.0), 1);
}

#[test]
fn a_city_raises_the_height_only_with_a_valid_shape_and_a_building_through_its_sector_placement_alone() {
    let w = world();
    // a city is created with the invalid shape: it arrives and raises nothing
    let key = w.roster.borrow_mut().insert((0, 0), CITY_TYPE as i32, None).to_string();
    let stated = ask(&w, city([10.0, 300.0, 20.0], 0.0), &key);
    assert_eq!(stated.result, "placed", "{}", stated.reason);
    assert_eq!(read(&w, 0.0, 0.0), ZERO);
    // a city that has a valid shape (a creator or a restore gave it one) raises like any fixed sender: position.y + the box top
    let valid = w.roster.borrow_mut().insert((0, 0), CITY_TYPE as i32, None).to_string();
    let stated = ask(&w, city([10.0, 300.0, 20.0], 1.0), &valid);
    assert_eq!(stated.result, "placed", "{}", stated.reason);
    assert_eq!(read(&w, 0.0, 0.0), 301.0f32.to_bits());
    // a building listed under a city: the statement under its place in the city's list is NOT an arrival in a sector (the city's list notifies the city, not the sector) ...
    let city_key: u64 = key.parse().expect("a key");
    let child = w.roster.borrow_mut().child_insert(city_key, None, CITY_BUILDING_TYPE as i32).expect("listed");
    let staged = ask(&w, fixed([10.0, 800.0, 20.0], CITY_BUILDING_TYPE, 1.0), &child.to_string());
    assert_eq!(staged.result, "placed", "{}", staged.reason);
    assert_eq!(read(&w, 0.0, 0.0), 301.0f32.to_bits(), "a building listed in a city's list raises nothing");
    // ... its SECTOR placement is: the host's records of the two placements are joined (LINK) BEFORE the sector placement's own statement, so the record already holds the staged state, and the arrival is still taken
    let sector_key = w.roster.borrow_mut().insert((0, 0), CITY_BUILDING_TYPE as i32, None);
    w.roster.borrow_mut().link(sector_key, child).expect("linked");
    let arrived = ask(&w, fixed([10.0, 800.0, 20.0], CITY_BUILDING_TYPE, 1.0), &sector_key.to_string());
    assert_eq!(arrived.result, "placed", "{}", arrived.reason);
    assert_eq!(
        read(&w, 0.0, 0.0),
        801.0f32.to_bits(),
        "the building's sector placement is an arrival even though its record already held a state"
    );
}

#[test]
fn a_restore_stores_any_raw_word_and_a_later_arrival_compares_against_exactly_that_word() {
    let w = world();
    let words: [u32; 17] = [
        0x0000_0000,
        0x8000_0000,
        0x0000_0001,
        0x007f_ffff,
        0x0080_0000,
        0x3f80_0000,
        0x42dc_0000,
        0x461c_4000,
        0x7f7f_ffff,
        0xff7f_ffff,
        0x7f80_0000,
        0xff80_0000,
        0x7fc0_0000,
        0xbf80_0000,
        0xc2dc_0000,
        0x7fc0_f00d,
        0xffc0_0001,
    ];
    for word in words {
        restore(&w, 2.0, 5.0, word);
        assert_eq!(read(&w, 2.0, 5.0), word, "the read answers exactly the word restored: {word:#010x}");
        assert_eq!(read(&w, 5.0, 2.0), ZERO, "no other sector changed");
    }
    // a restored NEGATIVE word is a valid history: a structure whose sum is larger raises it, one that is not larger leaves it
    let negative = (-200.0f32).to_bits();
    restore(&w, 0.0, 0.0, negative);
    arrive(&w, OBJECT_TYPE, [10.0, -300.0, 20.0], 1.0);
    assert_eq!(read(&w, 0.0, 0.0), negative, "-299 is not greater than -200");
    arrive(&w, OBJECT_TYPE, [10.0, -151.0, 20.0], 1.0);
    assert_eq!(
        read(&w, 0.0, 0.0),
        (-150.0f32).to_bits(),
        "a larger negative sum raises a restored negative word"
    );
    // a restored NaN is never exceeded, and neither is a restored infinity
    for word in [0x7fc0_0000u32, 0x7fc0_f00d, 0xffc0_0001, 0x7f80_0000] {
        restore(&w, 0.0, 0.0, word);
        arrive(&w, OBJECT_TYPE, [10.0, 900.0, 20.0], 1.0);
        assert_eq!(read(&w, 0.0, 0.0), word, "{word:#010x} is never exceeded");
    }
    // a restored negative infinity is exceeded by any finite sum
    restore(&w, 0.0, 0.0, 0xff80_0000);
    arrive(&w, OBJECT_TYPE, [10.0, 7.0, 20.0], 8.0);
    assert_eq!(read(&w, 0.0, 0.0), 7.0f32.to_bits());
}

#[test]
fn a_boots_clear_ends_every_height_and_the_arrival_of_the_new_boot_starts_from_zero() {
    let w = world();
    arrive(&w, OBJECT_TYPE, [10.0, 100.0, 20.0], 1.0);
    restore(&w, 4.0, 4.0, 0x3f80_0000);
    let mut reset = [0u32; 16];
    reset[11] = RESET;
    assert_eq!(ask(&w, reset, "").result, "cleared");
    assert_eq!(read(&w, 0.0, 0.0), ZERO, "a word never survives a boot");
    assert_eq!(read(&w, 4.0, 4.0), ZERO);
    arrive(&w, OBJECT_TYPE, [10.0, 5.0, 20.0], 8.0);
    assert_eq!(read(&w, 0.0, 0.0), 5.0f32.to_bits());
}

#[test]
fn a_statement_the_roster_refuses_takes_no_arrival_and_raises_nothing() {
    let w = world();
    let key = w.roster.borrow_mut().insert((0, 0), OBJECT_TYPE as i32, None).to_string();
    // the wrong entity type for the placement: refused before the record changes, and the arrival is still to come
    let wrong = ask(&w, fixed([10.0, 100.0, 20.0], 21.0, 1.0), &key);
    assert_eq!(wrong.result, "refused");
    assert_eq!(read(&w, 0.0, 0.0), ZERO);
    assert!(w.roster.borrow().awaiting_arrival(key.parse().expect("a key")).is_some());
    // an undeclared sub type: the same
    let mut sub = fixed([10.0, 100.0, 20.0], OBJECT_TYPE, 1.0);
    sub[10] = 99.0f32.to_bits();
    assert_eq!(ask(&w, sub, &key).result, "refused");
    assert_eq!(read(&w, 0.0, 0.0), ZERO);
    // a shape no scene has: named, nothing changes
    let mut beyond = fixed([10.0, 100.0, 20.0], OBJECT_TYPE, 1.0);
    beyond[5] = 5_000_000.0f32.to_bits();
    let refused = ask(&w, beyond, &key);
    assert_eq!(refused.result, "refused");
    assert!(refused.reason.contains("[roster.sector_height.shape]"), "{}", refused.reason);
    assert_eq!(read(&w, 0.0, 0.0), ZERO);
    assert!(w.roster.borrow().awaiting_arrival(key.parse().expect("a key")).is_some());
    // the right statement is then the arrival
    let placed = ask(&w, fixed([10.0, 100.0, 20.0], OBJECT_TYPE, 1.0), &key);
    assert_eq!(placed.result, "placed", "{}", placed.reason);
    assert_eq!(read(&w, 0.0, 0.0), 101.0f32.to_bits());
    assert!(w.roster.borrow().awaiting_arrival(key.parse().expect("a key")).is_none());
}

#[test]
fn a_malformed_height_question_is_refused_by_name_before_anything_is_read_or_changed() {
    let w = world();
    restore(&w, 1.0, 1.0, 0x3f80_0000);
    let refused = |words: [u32; 16], identity: &str, marker: &str, what: &str| {
        let answer = ask(&w, words, identity);
        assert_eq!(answer.result, "refused", "{what}");
        assert!(answer.reason.contains(marker), "{what}: {}", answer.reason);
        assert_eq!(read(&w, 1.0, 1.0), 0x3f80_0000, "{what}: nothing changed");
    };
    // a fact the question does not state, in every place that is not the phase, the sector or (for a restore) fact 0
    for at in (0..16).filter(|at| !matches!(at, 11..=13)) {
        let mut words = question(SECTOR_HEIGHT, 1.0, 1.0, None);
        words[at] = 0x3f80_0000;
        refused(words, "", "[roster.sector_height.request]", &format!("a read stating fact {at}"));
    }
    for at in (1..16).filter(|at| !matches!(at, 11..=13)) {
        let mut words = question(SECTOR_RESTORE, 1.0, 1.0, Some(0x3f80_0000));
        words[at] = 0x3f80_0000;
        refused(words, "", "[roster.sector_height.request]", &format!("a restore stating fact {at}"));
    }
    // a placement named, a fractional, negative, signaling, infinite or out-of-domain sector
    refused(
        question(SECTOR_HEIGHT, 1.0, 1.0, None),
        "1",
        "[roster.sector_height.key]",
        "a read that names a placement",
    );
    refused(
        question(SECTOR_RESTORE, 1.0, 1.0, Some(0)),
        "1",
        "[roster.sector_height.key]",
        "a restore that names a placement",
    );
    for bad in [
        1.5f32.to_bits(),
        (-1.0f32).to_bits(),
        0x7fa0_0001,
        0x7fc0_0000,
        f32::INFINITY.to_bits(),
        33_554_432.0f32.to_bits(),
    ] {
        let mut words = question(SECTOR_HEIGHT, 1.0, 1.0, None);
        words[12] = bad;
        refused(words, "", "[roster.sector_height.sector]", &format!("x word {bad:#x}"));
        let mut words = question(SECTOR_RESTORE, 1.0, 1.0, Some(0x3f80_0000));
        words[13] = bad;
        refused(words, "", "[roster.sector_height.sector]", &format!("z word {bad:#x}"));
    }
}

fn candidate(world: &World, scene: i32, facts: &[f32; 44], key: &str) -> (String, String) {
    let patterns = facts.iter().map(|word| format!("0x{:08x}", word.to_bits())).collect::<Vec<_>>().join(",");
    world.lua.globals().set("FACTS", format!("{patterns} {key}")).expect("facts");
    world.lua.globals().set("SCENE", scene).expect("scene");
    let table: mlua::Table = world
        .lua
        .load(r#"return host.model_provider (host.model_store (ROOT)) ("entity_candidate", SCENE, nil, nil, nil, FACTS)"#)
        .eval()
        .expect("the provider answers");
    (
        table.get("result").expect("result"),
        table.get::<Option<String>>("reason").expect("reason").unwrap_or_default(),
    )
}

#[test]
fn a_candidate_question_carries_the_hosts_word_and_a_word_that_is_not_the_one_the_host_holds_is_refused_by_name() {
    let w = world();
    let key = arrive(&w, OBJECT_TYPE, [10.0, 100.0, 20.0], 1.0);
    let mut facts = [0.0f32; 44];
    facts[..6].copy_from_slice(&[-100.0, -100.0, -100.0, 100.0, 100.0, 100.0]);
    facts[19] = 0.1;
    // the campaign passes on the host's word: the sector skip is decided over it (the segment's lowest point 200 is above 101: skipped)
    facts[12] = 101.0;
    facts[1] = 200.0;
    facts[4] = 300.0;
    assert_eq!(candidate(&w, 0, &facts, &key), ("skip".to_string(), String::new()));
    // a word the host does not hold for the candidate's own sector is refused, whatever the campaign calls it: the campaign has no word of its own
    for wrong in [0.0f32, 100.0, 101.5, -101.0] {
        facts[12] = wrong;
        let (result, reason) = candidate(&w, 0, &facts, &key);
        assert_eq!(result, "refused", "{wrong}");
        assert!(
            reason.contains("[candidate.sector_height]") && reason.contains("0x42ca0000"),
            "{wrong}: {reason}"
        );
    }
    // the comparison is of bits: the other zero is not the zero word the host holds, and a NaN word the host holds matches only its own bits
    restore(&w, 0.0, 0.0, 0x0000_0000);
    facts[12] = -0.0;
    let (result, reason) = candidate(&w, 0, &facts, &key);
    assert_eq!(result, "refused", "negative zero is not the zero word");
    assert!(reason.contains("[candidate.sector_height]"), "{reason}");
    restore(&w, 0.0, 0.0, 0x7fc0_f00d);
    facts[12] = f32::from_bits(0x7fc0_f00d);
    facts[1] = -1000.0;
    facts[4] = 0.0;
    // every strict comparison with a NaN is false, so nothing is skipped on the host's NaN word
    assert_eq!(candidate(&w, 0, &facts, &key), ("need_fixed".to_string(), String::new()));
    facts[12] = f32::from_bits(0x7fc0_0000);
    assert_eq!(candidate(&w, 0, &facts, &key).0, "refused", "another NaN payload is not the host's word");
}
