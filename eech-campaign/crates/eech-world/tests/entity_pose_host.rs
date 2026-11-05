//! ENTITY_POSE through the reference host's real provider (EEDCS-49, D3, fifth slice): an OBSERVATION of a mobile candidate against the host's record of its identity.
//!
//! The provider reads the ONE `Movers` registry the movement callbacks step (shared by handle, never copied), compares the campaign's words with the record bit for bit and reports a class, the
//! kind of the record and the differences. It decides nothing and changes nothing; a malformed question is a named refusal, and a registry that is being stepped is never read through a second
//! borrow. These tests use the real provider and the real `Movers` (bound, stepped and retired through its own API), not a model of either.

use eech_world::movement::{Advance, Kind, Movers, Pose, Request, SharedMovers};
use mlua::{Lua, Table};
use std::cell::RefCell;
use std::rc::Rc;

const LEVEL: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

fn at(x: f32, z: f32, velocity: f32) -> Pose {
    Pose {
        position: [x, 2.0, z],
        velocity,
        attitude: LEVEL,
    }
}

/// a host with the reference model provider registered (no installation is needed for this operation) and, when `movers` is given, the registry shared the way the host shares it
fn host(movers: Option<Rc<RefCell<Movers>>>) -> Lua {
    let lua = Lua::new();
    let host = lua.create_table().unwrap();
    host.set("args", lua.create_table().unwrap()).unwrap();
    eech_world::model_host::register(&lua, &host).unwrap();
    lua.globals().set("host", host).unwrap();
    if let Some(movers) = movers {
        lua.set_app_data(SharedMovers(movers));
    }
    lua
}

fn pattern(float: f32) -> String {
    format!("0x{:08x}", float.to_bits())
}

/// the fifteen words as the campaign sends them, then (when there is one) a space and the identity
fn text(words: &[f32; 15], host_id: &str) -> String {
    let patterns = words.iter().map(|word| pattern(*word)).collect::<Vec<_>>().join(",");
    if host_id.is_empty() {
        patterns
    } else {
        format!("{patterns} {host_id}")
    }
}

fn words(position: [f32; 3], motion: [f32; 3], attitude: Option<[f32; 9]>) -> [f32; 15] {
    let mut all = [0.0f32; 15];
    all[..3].copy_from_slice(&position);
    all[3..6].copy_from_slice(&motion);
    if let Some(attitude) = attitude {
        all[6..].copy_from_slice(&attitude);
    }
    all
}

fn observe(lua: &Lua, stage: i64, entity_type: i64, alive: i64, key: &str) -> mlua::Result<(String, i64, i64)> {
    lua.globals().set("KEY", key.to_string()).unwrap();
    let table: Table = lua
        .load(format!(r#"return host.model ("entity_pose", 0, {stage}, {entity_type}, {alive}, KEY)"#))
        .eval()?;
    Ok((table.get("result")?, table.get("kind")?, table.get("difference")?))
}

fn registry() -> Rc<RefCell<Movers>> {
    let mut movers = Movers::default();
    movers.bind("eech-world:1", Kind::Routed, at(1.0, 3.0, 5.0), None, None).unwrap();
    movers
        .bind("eech-world:2", Kind::FixedWing, at(10.0, 30.0, 0.0), Some([0.0, 0.0, 100.0]), None)
        .unwrap();
    movers.issue("eech-world:3").unwrap();
    movers.bind("eech-world:4", Kind::Person, at(7.0, 7.0, 1.0), None, None).unwrap();
    movers.retire("eech-world:4").unwrap();
    Rc::new(RefCell::new(movers))
}

#[test]
fn a_recorded_identity_is_compared_with_the_hosts_own_record_and_every_class_is_reported() {
    let lua = host(Some(registry()));
    // a routed vehicle (kind 1): the host holds no motion vector for it (8) and the campaign's equals the record's attitude row 2 times its speed, so only that is reported
    let routed = words([1.0, 2.0, 3.0], [0.0, 0.0, 5.0], None);
    assert_eq!(observe(&lua, 0, 4, 1, &text(&routed, "eech-world:1")).unwrap(), ("recorded".into(), 1, 8));
    // the placement stage adds the attitude, compared bit for bit
    let placed = words([1.0, 2.0, 3.0], [0.0, 0.0, 5.0], Some(LEVEL));
    assert_eq!(observe(&lua, 1, 4, 1, &text(&placed, "eech-world:1")).unwrap(), ("recorded".into(), 1, 8));
    let mut turned = LEVEL;
    turned[5] = 1.0e-7;
    let turned = words([1.0, 2.0, 3.0], [0.0, 0.0, 5.0], Some(turned));
    assert_eq!(observe(&lua, 1, 4, 1, &text(&turned, "eech-world:1")).unwrap(), ("recorded".into(), 1, 8 | 2));
    // a position that is not the record's, and a motion vector that is not the formula's
    let off = words([1.0, 2.5, 3.0], [0.0, 0.0, 4.0], None);
    assert_eq!(observe(&lua, 0, 4, 1, &text(&off, "eech-world:1")).unwrap(), ("recorded".into(), 1, 8 | 1 | 16));
    // a fixed-wing aircraft (kind 4) holds its vector: equal to it, the formula (a speed of zero) is the only difference; another vector is a difference of the held one too
    let held = words([10.0, 2.0, 30.0], [0.0, 0.0, 100.0], None);
    assert_eq!(observe(&lua, 0, 3, 1, &text(&held, "eech-world:2")).unwrap(), ("recorded".into(), 4, 16));
    let other = words([10.0, 2.0, 30.0], [0.0, 0.0, 99.0], None);
    assert_eq!(observe(&lua, 0, 3, 1, &text(&other, "eech-world:2")).unwrap(), ("recorded".into(), 4, 4 | 16));
    // the other classes, with the fields that go with them
    assert_eq!(observe(&lua, 0, 4, 1, &text(&routed, "eech-world:3")).unwrap(), ("unrecorded".into(), 0, 0));
    assert_eq!(observe(&lua, 0, 4, 0, &text(&routed, "eech-world:4")).unwrap(), ("retired".into(), 0, 0));
    assert_eq!(observe(&lua, 0, 4, 1, &text(&routed, "eech-world:99")).unwrap(), ("unknown".into(), 0, 0));
    assert_eq!(observe(&lua, 0, 4, 1, &text(&routed, "")).unwrap(), ("no_id".into(), 0, 0));
    // an identity may hold any text after the single space, a space included
    assert_eq!(observe(&lua, 0, 4, 1, &text(&routed, "a b c")).unwrap(), ("unknown".into(), 0, 0));
}

#[test]
fn the_host_record_decides_what_differs_and_an_observation_changes_nothing() {
    let movers = registry();
    let lua = host(Some(movers.clone()));
    let before = words([1.0, 2.0, 3.0], [0.0, 0.0, 5.0], None);
    assert_eq!(observe(&lua, 0, 4, 1, &text(&before, "eech-world:1")).unwrap().2 & 1, 0);
    // a deliberately different set of words is reported as different and adopted by nobody: the next step still finds the record exactly as it was
    let different = words([99.0, 99.0, 99.0], [9.0, 9.0, 9.0], None);
    assert_eq!(observe(&lua, 0, 4, 1, &text(&different, "eech-world:1")).unwrap().2 & 1, 1);
    let stopped = movers.borrow_mut().step("eech-world:1", at(1.0, 3.0, 5.0), Request::Stop).unwrap();
    assert_eq!(stopped.pose.position, [1.0, 2.0, 3.0]);
    // the host moves the mobile (its own step): the campaign's earlier words are now the ones that differ, with no word of the campaign involved
    let advance = Advance {
        delta_seconds: 1.0,
        desired_velocity: 10.0,
        max_acceleration: 4.0,
        waypoint: [100.0, 0.0, 3.0],
        tacview_logging: false,
        terrain: None,
        map_min: [-1.0e6; 3],
        map_max: [1.0e6; 3],
    };
    let moved = movers.borrow_mut().step("eech-world:1", stopped.pose, Request::Advance(advance)).unwrap();
    assert_ne!(moved.pose.position, [1.0, 2.0, 3.0]);
    assert_eq!(
        observe(&lua, 0, 4, 1, &text(&before, "eech-world:1")).unwrap().2 & 1,
        1,
        "the record moved, the campaign's words did not"
    );
    // retiring the identity (the campaign's report that the incarnation ended) changes the class
    movers.borrow_mut().retire("eech-world:1").unwrap();
    assert_eq!(observe(&lua, 0, 4, 1, &text(&before, "eech-world:1")).unwrap().0, "retired");
}

#[test]
fn a_host_that_keeps_no_movers_classifies_a_named_identity_as_unknown_and_an_unnamed_one_as_none() {
    let lua = host(None);
    let routed = words([1.0, 2.0, 3.0], [0.0, 0.0, 5.0], None);
    assert_eq!(observe(&lua, 0, 4, 1, &text(&routed, "eech-world:1")).unwrap(), ("unknown".into(), 0, 0));
    assert_eq!(observe(&lua, 0, 4, 1, &text(&routed, "")).unwrap(), ("no_id".into(), 0, 0));
}

#[test]
fn a_malformed_observation_is_a_named_refusal_and_a_registry_being_stepped_is_never_read_through_a_second_borrow() {
    let movers = registry();
    let lua = host(Some(movers.clone()));
    let good = text(&words([1.0, 2.0, 3.0], [0.0, 0.0, 5.0], None), "eech-world:1");
    let refused = |result: mlua::Result<(String, i64, i64)>, marker: &str| {
        let error = result.expect_err("accepted").to_string();
        assert!(error.contains(marker), "{marker}: {error}");
    };
    // the question's numbers: the scene is 0, the stage is 0 or 1, the type is given and the alive flag is 0 or 1
    lua.globals().set("KEY", good.clone()).unwrap();
    for call in [
        r#"host.model ("entity_pose", 5, 0, 4, 1, KEY)"#,
        r#"host.model ("entity_pose", nil, 0, 4, 1, KEY)"#,
        r#"host.model ("entity_pose", 0, 2, 4, 1, KEY)"#,
        r#"host.model ("entity_pose", 0, nil, 4, 1, KEY)"#,
        r#"host.model ("entity_pose", 0, 0, nil, 1, KEY)"#,
        r#"host.model ("entity_pose", 0, 0, 4, 2, KEY)"#,
        r#"host.model ("entity_pose", 0, 0, 4, nil, KEY)"#,
        r#"host.model ("entity_pose", 0, 0, 4, 1, nil)"#,
    ] {
        let error = lua.load(format!("return {call}")).eval::<Table>().expect_err(call).to_string();
        assert!(error.contains("[model-provider:entity_pose.request]"), "{call}: {error}");
    }
    // the text: fifteen fixed-width patterns, and the identity after exactly one space
    let patterns = good.split(' ').next().unwrap().to_string();
    let fourteen = patterns.rsplit_once(',').unwrap().0.to_string();
    for bad in [
        fourteen,
        format!("{patterns},0x00000000"),
        patterns.replacen("0x", "0y", 1),
        patterns.replacen("0x3f800000", "0x3f80000", 1),
        format!("{patterns}x"),
        format!("{patterns},"),
        String::new(),
    ] {
        refused(observe(&lua, 0, 4, 1, &bad), "[model-provider:entity_pose.request]");
    }
    // a registry that is being stepped is refused by name, not read, and answers again once it is free
    let guard = movers.borrow_mut();
    refused(observe(&lua, 0, 4, 1, &good), "[model-provider:entity_pose.busy]");
    drop(guard);
    assert_eq!(observe(&lua, 0, 4, 1, &good).unwrap().0, "recorded");
}

#[test]
fn the_rosters_whole_identity_domain_is_observed_whole_and_a_signalling_nan_word_is_only_a_difference() {
    // the longest identity the roster issues is 128 bytes of text (multi-byte text included): the real registry binds it and the real provider finds it, whole
    let mut movers = Movers::default();
    let longest = "w".repeat(128);
    let multibyte = "\u{e9}".repeat(64);
    assert_eq!(multibyte.len(), 128);
    movers.bind(&longest, Kind::Routed, at(1.0, 3.0, 5.0), None, None).unwrap();
    movers.bind(&multibyte, Kind::Ship, at(4.0, 6.0, 2.0), None, None).unwrap();
    let movers = Rc::new(RefCell::new(movers));
    let lua = host(Some(movers));
    let routed = words([1.0, 2.0, 3.0], [0.0, 0.0, 5.0], None);
    assert_eq!(observe(&lua, 0, 4, 1, &text(&routed, &longest)).unwrap(), ("recorded".into(), 1, 8));
    let ship = words([4.0, 2.0, 6.0], [0.0, 0.0, 2.0], None);
    assert_eq!(observe(&lua, 0, 4, 1, &text(&ship, &multibyte)).unwrap(), ("recorded".into(), 2, 8));
    // a prefix of an identity is not the identity, and nothing is shortened on the way
    assert_eq!(observe(&lua, 0, 4, 1, &text(&routed, &longest[..127])).unwrap().0, "unknown");
    // a signalling NaN word crosses as its bits (no float is formed from it): it is a position difference, and the registry is untouched
    let mut signalling = routed;
    signalling[2] = f32::from_bits(0x7fa0_0001);
    assert_eq!(observe(&lua, 0, 4, 1, &text(&signalling, &longest)).unwrap(), ("recorded".into(), 1, 8 | 1));
    assert!(text(&signalling, "").contains("0x7fa00001"), "the pattern is the bits");
    assert_eq!(observe(&lua, 0, 4, 1, &text(&routed, &longest)).unwrap(), ("recorded".into(), 1, 8));
}
