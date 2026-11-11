//! Compile guard of the lifecycle facts the host-record collision decision rests on (EEDCS-49, D3): a mobile candidate of a sector list is REFUSED by name when the host holds no canonical record for
//! its identity, which is correct only if no supported creation, link, placement, step, loss, wreck or release path leaves a collidable mobile in a sector list without one.
//!
//! These facts are read from the compiled source and are asserted here as text, so that a change that breaks one of them fails before any native run. They are the CURRENT compiled caller and lifecycle
//! evidence for the supported single-player runtime; they are not a proof of behaviour (the real `Movers`, provider and registry lifecycle control in eech-world, and the native runs, are) and they say
//! nothing about the multiplayer pack and unpack routes, which are an unreached, required domain:
//!
//!  1. every collidable mobile is an aircraft or a vehicle (the only two sources that answer `INT_TYPE_COLLISION_TEST_MOBILE`), and each of the six kinds is bound to a movement kind WITH a pose record;
//!  2. each of the six creators links the new mobile into its group's member list, which binds its identity (`campaign_roster_link_member`), BEFORE it inserts it into the sector list the collision scan
//!     reads, and the list insertion itself refuses a missing group (`ASSERT (parent)`), so a collidable mobile cannot be created without an identity;
//!  3. the group lifecycle (the loss's unlink) never clears the identity or retires the record: a wreck keeps both until its heap slot is released;
//!  4. each of the six destroyers removes the mobile from its sector list BEFORE it frees the heap slot (the only place the identity is retired), so a retired identity is never visible to a scan;
//!  5. the hosted entity decision is compiled only together with the hosted movement and identities.

use std::path::{Path, PathBuf};

fn source(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../aphavoc/source").join(relative);
    String::from_utf8_lossy(&std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).into_owned()
}

fn position(text: &str, needle: &str, file: &str) -> usize {
    text.find(needle).unwrap_or_else(|| panic!("{file}: `{needle}` is not there"))
}

const CREATORS: [&str; 6] = [
    "entity/mobile/vehicle/routed/rv_creat.c",
    "entity/mobile/vehicle/anti_air/aa_creat.c",
    "entity/mobile/vehicle/ship/sh_creat.c",
    "entity/mobile/vehicle/person/ps_creat.c",
    "entity/mobile/aircraft/fixwing/fw_creat.c",
    "entity/mobile/aircraft/helicop/hc_creat.c",
];

const DESTROYERS: [&str; 6] = [
    "entity/mobile/vehicle/routed/rv_dstry.c",
    "entity/mobile/vehicle/anti_air/aa_dstry.c",
    "entity/mobile/vehicle/ship/sh_dstry.c",
    "entity/mobile/vehicle/person/ps_dstry.c",
    "entity/mobile/aircraft/fixwing/fw_dstry.c",
    "entity/mobile/aircraft/helicop/hc_dstry.c",
];

fn walk(directory: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "c") {
            found.push(path);
        }
    }
}

#[test]
fn only_aircraft_and_vehicles_are_collidable_mobiles_and_every_kind_is_bound_with_a_pose_record() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../aphavoc/source/entity");
    let mut files = Vec::new();
    walk(&root, &mut files);
    let mut answering: Vec<String> = files
        .iter()
        .filter(|path| String::from_utf8_lossy(&std::fs::read(path).unwrap()).contains("case INT_TYPE_COLLISION_TEST_MOBILE:"))
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    answering.sort();
    assert_eq!(
        answering,
        ["ac_int.c", "vh_int.c"],
        "a new source of the collidable-mobile answer needs its own record contract"
    );
    // every kind they cover (the aircraft and the vehicle types) has a movement kind in the roster's bind, which carries the pose (and the canonical vector) of the record
    let roster = source("entity/special/group/gp_roster.c");
    let bind = &roster[position(&roster, "switch (descriptor.entity_type)", "gp_roster.c")..];
    let bind = &bind[..position(bind, "descriptor.has_pose = TRUE;", "gp_roster.c")];
    for (entity, kind) in [
        ("ENTITY_TYPE_ROUTED_VEHICLE", "CAMPAIGN_MOVEMENT_KIND_ROUTED"),
        ("ENTITY_TYPE_ANTI_AIRCRAFT", "CAMPAIGN_MOVEMENT_KIND_ROUTED"),
        ("ENTITY_TYPE_SHIP_VEHICLE", "CAMPAIGN_MOVEMENT_KIND_SHIP"),
        ("ENTITY_TYPE_PERSON", "CAMPAIGN_MOVEMENT_KIND_PERSON"),
        ("ENTITY_TYPE_FIXED_WING", "CAMPAIGN_MOVEMENT_KIND_FIXED_WING"),
        ("ENTITY_TYPE_HELICOPTER", "CAMPAIGN_MOVEMENT_KIND_HELICOPTER"),
    ] {
        let at = position(bind, &format!("case {entity}:"), "gp_roster.c");
        assert!(bind[at..].contains(kind), "{entity} is bound to {kind}");
    }
    assert!(
        roster.contains("descriptor.motion_vector[0] = mob->motion_vector.x;"),
        "the canonical vector is seeded for every hosted kind"
    );
    // the one place a member is bound is the group's member link: a mobile that is linked has an identity from then on
    let messages = source("entity/special/group/gp_msgs.c");
    assert!(messages.contains("campaign_roster_link_member (receiver, sender);"));
}

#[test]
fn every_mobile_creator_binds_the_identity_by_the_group_link_before_the_sector_list_the_scan_reads_and_a_missing_group_is_refused() {
    for file in CREATORS {
        let text = source(file);
        let link = position(&text, "insert_local_entity_into_parents_child_list (en, LIST_TYPE_MEMBER", file);
        let sector = position(&text, "insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR", file);
        assert!(
            link < sector,
            "{file}: the group link (which binds the identity) must precede the sector insertion"
        );
        // the defaults (the attitude from the terrain face included) are final before the bind seeds the record from them
        let defaults = position(&text, "memset (raw, 0, sizeof", file);
        assert!(defaults < link, "{file}: the working defaults precede the link");
        assert!(
            text.contains("ENTITY_ATTR_PARENT (LIST_TYPE_MEMBER, group)") || text.contains("member_link.parent"),
            "{file}: the creator names its group"
        );
    }
    let list = source("entity/system/en_funcs/en_list.c");
    let insert = &list[position(&list, "void insert_local_entity_into_parents_child_list (", "en_list.c")..];
    assert!(insert[..400].contains("ASSERT (parent);"), "the list insertion refuses a missing group");
}

#[test]
fn a_loss_and_a_group_unlink_keep_the_identity_and_the_record_until_the_heap_slot_is_released_and_a_scan_never_sees_a_released_one() {
    let roster = source("entity/special/group/gp_roster.c");
    let unlink = &roster[position(&roster, "void campaign_roster_unlink_member (", "gp_roster.c")..];
    let unlink = &unlink[..position(unlink, "void campaign_roster_release_entity (", "gp_roster.c")];
    assert!(!unlink.contains("campaign_movement_retire"), "an unlink does not retire the record");
    // the mapping is READ to find the record, never assigned: a wreck is still named
    for line in unlink.lines().filter(|line| line.contains("current_member_by_entity[")) {
        let after = line.split("current_member_by_entity[").nth(1).unwrap();
        let rest = after.split_once(']').map_or("", |(_, rest)| rest.trim_start());
        assert!(
            !(rest.starts_with('=') && !rest.starts_with("==")),
            "an unlink does not clear the identity mapping: {line}"
        );
    }
    let release = &roster[position(&roster, "void campaign_roster_release_entity (", "gp_roster.c")..];
    let release = &release[..position(release, "int campaign_roster_observe_credit (", "gp_roster.c")];
    assert!(
        release.contains("campaign_movement_retire (record->host_id);"),
        "the release retires the record"
    );
    assert!(
        roster.matches("campaign_movement_retire (").count() == 1,
        "the release is the ONLY place a record is retired"
    );
    let heap = source("entity/system/en_main/en_heap.c");
    assert!(
        heap.contains("campaign_roster_release_entity (en);"),
        "the heap slot's release is where the roster releases"
    );
    for file in DESTROYERS {
        let text = source(file);
        let sector = position(&text, "delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR)", file);
        let free = position(&text, "set_free_entity (en)", file);
        assert!(
            sector < free,
            "{file}: the mobile leaves the sector list before its heap slot (and its identity) is released"
        );
    }
}

#[test]
fn the_hosted_entity_decision_is_compiled_only_with_the_hosted_movement_and_identities_that_give_every_mobile_a_record() {
    let collision = source("entity/en_misc/en_maths/en_cllsn.c");
    let guard = position(
        &collision,
        "#if !defined(EECH_CAMPAIGN_HOST_MOVEMENT) || !defined(EECH_CAMPAIGN_HOST_IDENTITIES)",
        "en_cllsn.c",
    );
    let hosted = position(&collision, "#ifdef EECH_CAMPAIGN_HOST_ENTITY", "en_cllsn.c");
    assert!(
        hosted < guard && collision[guard..].starts_with("#if !defined(EECH_CAMPAIGN_HOST_MOVEMENT)"),
        "the compile-time dependency sits in the hosted branch"
    );
    assert!(collision[guard..guard + 600].contains("#error"));
    let build = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("build/main.rs")).unwrap();
    assert!(
        build.contains(".define(\"EECH_CAMPAIGN_HOST_IDENTITIES\", None)") && build.contains(".define(\"EECH_CAMPAIGN_HOST_MOVEMENT\", None)"),
        "every build of the module defines both"
    );
}
