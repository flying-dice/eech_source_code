//! The HOST's physical per-entity UPDATED / MOVED / ROTATED store (EEDCS-49, D3, Task49) through the reference host's PUBLIC provider (`host.model`): the three words, their set, their destructive read and their reset,
//! held by the host (`eech_world::flags::FlagStore`, reached through the roster state the provider already keeps) and asked by the campaign's own wire grammar (`mb_hostmodel.h` phases 19 to 22).
//!
//! These tests run the REAL provider with the REAL store and speak the questions the campaign's C binding sends (`mb_hostentity.h` `campaign_entity_flag_exchange`, `mb_hostmodel.c` `campaign_model_flag_*`, whose own
//! half is checked in eech-engine-sys: `entity_dialogue` for the exchange and `physical_flags`, `physical_flags_consumer` for the real getter, setter, reset and the accepted guide consumer). The original behaviour the
//! store reproduces is characterised by those 199 and 56 real-C expectations of the ORIGINAL code (original-solver profile) and is repeated here as the SAME facts over the host's store:
//! the raw word of a read is the lane's own mask (0x80000000 for lane 31), every other lane of the shared word is destroyed and the lane's own is kept (a repeated read answers the mask again), the three words are
//! independent, the reset reaches only the groups inside the original's quarter-memset extent and a bit outside it survives until a word-mate's read, and a boot's roster clear never touches the store.
//!
//! WHAT THIS IS NOT: the campaign. No native module, no entity heap and no host-owned Lua driving the native path runs here (that is the native run Root owns); these checks establish that the provider the shipped
//! module asks IS the owner of the words, and that every question outside the contract is refused by name and changes nothing. Positive questions come first; each refusal is matched with the same question corrected.

use eech_world::movement::{Movers, SharedMovers};
use eech_world::roster::{SectorRoster, SharedRoster};
use mlua::{Lua, Table};
use std::cell::RefCell;
use std::rc::Rc;

const GROUP: f32 = 19.0;
const SET: f32 = 20.0;
const READ: f32 = 21.0;
const RESET: f32 = 22.0;
/// the roster's own boot event (phase 6): a new roster identity
const ROSTER_RESET: f32 = 6.0;
const ONE: u32 = 0x3f80_0000;
const UPDATED: f32 = 0.0;
const MOVED: f32 = 1.0;
const ROTATED: f32 = 2.0;

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

/// the facts of a flag question: the phase (fact 11), the array (fact 12), the lane (fact 13) and, for a group, the eligibility (fact 0)
fn facts(phase: f32, array: f32, lane: f32, eligible: u32) -> [u32; 16] {
    let mut words = [0u32; 16];
    words[0] = eligible;
    words[11] = phase.to_bits();
    words[12] = array.to_bits();
    words[13] = lane.to_bits();
    words
}

struct Answer {
    result: String,
    reason: String,
    name: String,
    words: Vec<u32>,
}

fn answer(table: &Table) -> Answer {
    let words = table
        .get::<Option<String>>("p")
        .unwrap()
        .map(|text| {
            text.split(',')
                .map(|token| u32::from_str_radix(token.trim_start_matches("0x"), 16).unwrap())
                .collect()
        })
        .unwrap_or_default();
    Answer {
        result: table.get("result").unwrap(),
        reason: table.get::<Option<String>>("reason").unwrap().unwrap_or_default(),
        name: table.get::<Option<String>>("name").unwrap().unwrap_or_default(),
        words,
    }
}

fn open(lua: &Lua, eligible: bool) -> String {
    let reply = answer(&ask_words(lua, facts(GROUP, 0.0, 0.0, if eligible { ONE } else { 0 }), ""));
    assert_eq!(reply.result, "flag_group", "{}", reply.reason);
    assert_eq!(
        reply.words,
        vec![if eligible { ONE } else { 0 }, 0, 0],
        "the eligibility is echoed and nothing else"
    );
    assert!(reply.name.starts_with('f'), "a flag key: {}", reply.name);
    reply.name
}

fn set(lua: &Lua, key: &str, array: f32, lane: u32) -> Answer {
    answer(&ask_words(lua, facts(SET, array, lane as f32, 0), key))
}

fn read(lua: &Lua, key: &str, array: f32, lane: u32) -> Answer {
    answer(&ask_words(lua, facts(READ, array, lane as f32, 0), key))
}

fn reset(lua: &Lua, array: f32) -> Answer {
    answer(&ask_words(lua, facts(RESET, array, 0.0, 0), ""))
}

/// the raw word a read answered
fn raw(reply: &Answer) -> u32 {
    assert_eq!(reply.result, "flag_read", "{}", reply.reason);
    reply.words[2]
}

#[test]
fn a_set_and_a_destructive_read_through_the_public_provider_answer_the_originals_raw_masks_and_destroy_the_neighbours() {
    let (lua, roster) = host();
    let key = open(&lua, true);
    assert_eq!(key, "f1");
    // the SET answers the lane's mask and holds the bit in the HOST's store
    for (lane, mask) in [(0u32, 1u32), (1, 2), (30, 0x4000_0000), (31, 0x8000_0000)] {
        let reply = set(&lua, &key, UPDATED, lane);
        assert_eq!(
            (reply.result.as_str(), reply.words.clone()),
            ("flag_set", vec![UPDATED.to_bits(), (lane as f32).to_bits(), mask]),
            "{}",
            reply.reason
        );
    }
    assert_eq!(roster.borrow().flags.word(&key, 0).unwrap(), 0xc000_0003, "the bits are the host's");
    // a read of lane 1 answers its own mask, destroys lanes 0, 30 and 31 and keeps lane 1; the repeated read answers the mask again
    assert_eq!(raw(&read(&lua, &key, UPDATED, 1)), 2);
    assert_eq!(roster.borrow().flags.word(&key, 0).unwrap(), 2);
    assert_eq!(raw(&read(&lua, &key, UPDATED, 1)), 2);
    // a lane whose bit is clear reads 0 and destroys the set neighbour
    assert_eq!(raw(&read(&lua, &key, UPDATED, 5)), 0);
    assert_eq!(roster.borrow().flags.word(&key, 0).unwrap(), 0);
    // lane 31: the raw word is 0x80000000 (a negative int in C), never normalised to 1, and survives its own repeated read
    set(&lua, &key, UPDATED, 31);
    assert_eq!(raw(&read(&lua, &key, UPDATED, 31)), 0x8000_0000);
    assert_eq!(raw(&read(&lua, &key, UPDATED, 31)), 0x8000_0000);
}

#[test]
fn the_three_words_and_the_groups_are_independent_and_a_read_of_one_destroys_nothing_of_another() {
    let (lua, _roster) = host();
    let first = open(&lua, true);
    let second = open(&lua, true);
    assert_eq!((first.as_str(), second.as_str()), ("f1", "f2"));
    set(&lua, &first, UPDATED, 3);
    set(&lua, &first, MOVED, 3);
    set(&lua, &second, ROTATED, 3);
    assert_eq!(raw(&read(&lua, &second, UPDATED, 3)), 0, "another group's UPDATED lane is clear");
    assert_eq!(raw(&read(&lua, &first, ROTATED, 3)), 0, "another word of the same group is clear");
    assert_eq!(raw(&read(&lua, &first, UPDATED, 3)), 8);
    assert_eq!(raw(&read(&lua, &first, MOVED, 3)), 8, "MOVED survived the UPDATED read");
    assert_eq!(raw(&read(&lua, &second, ROTATED, 3)), 8);
}

#[test]
fn a_reset_reaches_only_the_groups_inside_the_originals_extent_and_the_array_it_names() {
    let (lua, roster) = host();
    let inside = open(&lua, true);
    let outside = open(&lua, false);
    for key in [&inside, &outside] {
        for array in [UPDATED, MOVED, ROTATED] {
            set(&lua, key, array, 7);
        }
    }
    let cleared = reset(&lua, UPDATED);
    assert_eq!(
        (cleared.result.as_str(), cleared.words[0], cleared.words[2]),
        ("flag_reset", UPDATED.to_bits(), 0),
        "{}",
        cleared.reason
    );
    assert_eq!(f32::from_bits(cleared.words[1]), 1.0, "one group lies inside the extent");
    {
        let roster = roster.borrow();
        assert_eq!(roster.flags.word(&inside, 0).unwrap(), 0);
        assert_eq!(
            roster.flags.word(&outside, 0).unwrap(),
            0x80,
            "a bit outside the extent survives the reset (the original's quarter-memset)"
        );
        assert_eq!(
            (roster.flags.word(&inside, 1).unwrap(), roster.flags.word(&inside, 2).unwrap()),
            (0x80, 0x80),
            "MOVED and ROTATED were not reset"
        );
    }
    // reset the other two words the way the Tacview frame does: MOVED then ROTATED
    reset(&lua, MOVED);
    reset(&lua, ROTATED);
    assert_eq!(roster.borrow().flags.word(&inside, 1).unwrap(), 0);
    assert_eq!(roster.borrow().flags.word(&outside, 2).unwrap(), 0x80);
    // survival is not permanence: a word-mate's read clears the bit the reset could not reach
    assert_eq!(raw(&read(&lua, &outside, UPDATED, 8)), 0);
    assert_eq!(
        roster.borrow().flags.word(&outside, 0).unwrap(),
        0,
        "the UPDATED bit the reset could not reach is gone: survival is not permanence"
    );
    assert_eq!(roster.borrow().flags.word(&outside, 1).unwrap(), 0x80, "only the UPDATED word was read");
    assert_eq!(raw(&read(&lua, &outside, MOVED, 9)), 0);
    assert_eq!(roster.borrow().flags.word(&outside, 1).unwrap(), 0);
}

#[test]
fn a_boot_and_a_new_roster_identity_never_clear_the_store_because_the_originals_static_arrays_are_not_cleared() {
    let (lua, roster) = host();
    let key = open(&lua, true);
    set(&lua, &key, UPDATED, 4);
    set(&lua, &key, MOVED, 4);
    // the roster's own boot event (phase 6) clears the roster; the set bits and the key stay
    let boot = answer(&ask_words(&lua, facts(ROSTER_RESET, 0.0, 0.0, 0), ""));
    assert_eq!(boot.result, "cleared", "{}", boot.reason);
    roster.borrow_mut().clear();
    assert_eq!(roster.borrow().flags.word(&key, 0).unwrap(), 0x10);
    assert_eq!(
        raw(&read(&lua, &key, UPDATED, 4)),
        0x10,
        "the history of the slot survives a fresh roster identity"
    );
    assert_eq!(raw(&read(&lua, &key, MOVED, 4)), 0x10);
    // a group opened after the boot is a new group with clear words, and the key is never reused
    let later = open(&lua, true);
    assert_eq!(later, "f2");
    assert_eq!(raw(&read(&lua, &later, UPDATED, 4)), 0);
}

fn refused(reply: &Answer, tag: &str) {
    assert_eq!(reply.result, "refused", "accepted: wanted {tag}");
    assert!(reply.reason.starts_with(&format!("[{tag}")), "{}: wanted {tag}", reply.reason);
    assert!(reply.words.is_empty(), "a refusal carries no words");
}

#[test]
fn every_malformed_unknown_or_wrong_identity_question_is_refused_by_name_and_changes_nothing_and_the_corrected_question_is_accepted() {
    let (lua, roster) = host();
    let key = open(&lua, true);
    set(&lua, &key, UPDATED, 2);
    let held = || roster.borrow().flags.word("f1", 0).unwrap();
    assert_eq!(held(), 4);
    // the WRONG IDENTITY: a group the host never issued, a roster key, a key of another namespace, a malformed key, no key at all
    refused(&set(&lua, "f2", UPDATED, 2), "flag.key.unknown");
    refused(&read(&lua, "f2", UPDATED, 2), "flag.key.unknown");
    refused(&read(&lua, "1", UPDATED, 2), "flag.key.malformed");
    refused(&read(&lua, "f01", UPDATED, 2), "flag.key.malformed");
    refused(&read(&lua, "f", UPDATED, 2), "flag.key.malformed");
    refused(&read(&lua, "F1", UPDATED, 2), "flag.key.malformed");
    // the byte bound of the key domain (ENV-PHYSICAL-FLAGS-KEY-CAPACITY-1): exactly 20 bytes is inside it (and was never issued), 21 bytes and a long name are outside it even though canonical
    refused(&read(&lua, "f9999999999999999999", UPDATED, 2), "flag.key.unknown");
    refused(&read(&lua, "f99999999999999999999", UPDATED, 2), "flag.key.malformed");
    refused(&read(&lua, &format!("f{}", "1".repeat(100)), UPDATED, 2), "flag.key.malformed");
    refused(&read(&lua, "", UPDATED, 2), "flag.key.missing");
    refused(&answer(&ask_words(&lua, facts(READ, 0.0, 2.0, 0), "f1 f1")), "flag.key.malformed");
    // a key where a group or a reset names none
    refused(&answer(&ask_words(&lua, facts(GROUP, 0.0, 0.0, ONE), "f1")), "flag.key");
    refused(&answer(&ask_words(&lua, facts(RESET, 0.0, 0.0, 0), "f1")), "flag.key");
    // the facts: an eligibility that is not 0.0 or 1.0, an array or a lane outside the domain or not whole, a fact the question does not state
    refused(&answer(&ask_words(&lua, facts(GROUP, 0.0, 0.0, 0x3f00_0000), "")), "flag.eligible");
    refused(&answer(&ask_words(&lua, facts(GROUP, 0.0, 0.0, 0x3f80_0001), "")), "flag.eligible");
    refused(&set(&lua, &key, 3.0, 2), "flag.array");
    refused(&set(&lua, &key, -1.0, 2), "flag.array");
    refused(&set(&lua, &key, 0.5, 2), "flag.array");
    refused(&answer(&ask_words(&lua, facts(SET, 0.0, 32.0, 0), &key)), "flag.lane");
    refused(&answer(&ask_words(&lua, facts(READ, 0.0, -1.0, 0), &key)), "flag.lane");
    refused(&answer(&ask_words(&lua, facts(READ, 0.0, 2.5, 0), &key)), "flag.lane");
    refused(&answer(&ask_words(&lua, facts(READ, 0.0, f32::NAN, 0), &key)), "flag.lane");
    refused(&reset(&lua, 3.0), "flag.array");
    for fact in [1, 2, 5, 10, 14, 15] {
        let mut words = facts(READ, 0.0, 2.0, 0);
        words[fact] = 1;
        refused(&answer(&ask_words(&lua, words, &key)), "flag.request");
    }
    let mut words = facts(SET, 0.0, 2.0, ONE);
    words[0] = ONE;
    refused(&answer(&ask_words(&lua, words, &key)), "flag.request");
    // nothing changed: the one bit is as it was, and the CORRECTED questions are accepted
    assert_eq!(held(), 4);
    assert_eq!(raw(&read(&lua, &key, UPDATED, 2)), 4);
    assert_eq!(set(&lua, &key, MOVED, 2).result, "flag_set");
    assert_eq!(reset(&lua, MOVED).result, "flag_reset");
    assert_eq!(roster.borrow().flags.len(), 1, "no refused question opened a group");
}

#[test]
fn a_provider_with_no_roster_state_refuses_a_flag_question_by_name_instead_of_answering_zero() {
    let lua = Lua::new();
    let host = lua.create_table().unwrap();
    host.set("args", lua.create_table().unwrap()).unwrap();
    eech_world::model_host::register(&lua, &host).unwrap();
    lua.globals().set("host", host).unwrap();
    lua.set_app_data(SharedMovers(Rc::new(RefCell::new(Movers::default()))));
    for words in [
        facts(GROUP, 0.0, 0.0, ONE),
        facts(SET, 0.0, 2.0, 0),
        facts(READ, 0.0, 2.0, 0),
        facts(RESET, 0.0, 0.0, 0),
    ] {
        let reply = answer(&ask_words(
            &lua,
            words,
            if words[11] == SET.to_bits() || words[11] == READ.to_bits() {
                "f1"
            } else {
                ""
            },
        ));
        refused(&reply, "flag.missing");
    }
}

#[test]
fn the_answer_to_every_phase_has_exactly_the_three_words_of_its_grammar() {
    let (lua, _roster) = host();
    let key = open(&lua, true);
    for reply in [
        set(&lua, &key, ROTATED, 9),
        read(&lua, &key, ROTATED, 9),
        reset(&lua, ROTATED),
        answer(&ask_words(&lua, facts(GROUP, 0.0, 0.0, 0), "")),
    ] {
        assert_eq!(reply.words.len(), 3, "{}", reply.result);
    }
    // a set's third word is the mask, a read's the raw word, a reset's the count and its third place is zero
    assert_eq!(set(&lua, &key, UPDATED, 31).words, vec![UPDATED.to_bits(), 31f32.to_bits(), 0x8000_0000]);
    assert_eq!(read(&lua, &key, UPDATED, 31).words, vec![UPDATED.to_bits(), 31f32.to_bits(), 0x8000_0000]);
    assert_eq!(reset(&lua, UPDATED).words[2], 0);
}
