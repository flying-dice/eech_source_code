//! SOURCE CENSUS of the per-entity UPDATED / MOVED / ROTATED word (EEDCS-49, D3, ENV-PHYSICAL-FLAGS-RAWWORD-CONSUMER-CONTRACT). It runs in every profile and reads only text.
//!
//! It is NOT the characterisation: the behaviour of the real getter, setter and reset is run by `physical_flags.rs` (feature `physical-flags-check`, csrc/eech_physical_flags_check.c). This file pins the CENSUS the
//! behavioural control cannot reach (who writes and reads the three words, the source lines of the getter, the reset and the descriptor boundary), so a new writer, a new reader or a changed domain is a named failure
//! of THIS file and not a silent change of a contract the next extraction relies on. Text patterns, not a preprocessed-symbol census; comments are stripped first.

use std::collections::BTreeMap;
use std::path::PathBuf;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn aphavoc() -> PathBuf {
    crate_dir().join("../../../aphavoc/source")
}

fn read(path: &std::path::Path) -> String {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    bytes.iter().map(|&b| b as char).collect()
}

fn strip_comments(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after.find("*/").map_or(after.len(), |e| e + 2);
        out.push_str(&"\n".repeat(after[..end.min(after.len())].matches('\n').count()));
        rest = &after[end.min(after.len())..];
    }
    out.push_str(rest);
    out.lines().map(|l| l.split("//").next().unwrap_or("")).collect::<Vec<_>>().join("\n")
}

fn walk(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            walk(&path, out);
        } else if matches!(path.extension().and_then(|e| e.to_str()), Some("c" | "h")) {
            out.push(path);
        }
    }
}

/// every use of INT_TYPE_UPDATED / _MOVED / _ROTATED in the original sources (the enum and its name table excluded), as `file -> "WORD:kind" -> count`
fn census() -> BTreeMap<String, BTreeMap<String, usize>> {
    let root = aphavoc();
    let mut files = Vec::new();
    walk(&root, &mut files);
    let mut out: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    for path in files {
        let name = path.file_name().unwrap().to_str().unwrap();
        if name == "en_int.h" || name == "en_int.c" {
            continue;
        }
        let code = strip_comments(&read(&path));
        for line in code.lines() {
            for word in ["UPDATED", "MOVED", "ROTATED"] {
                let needle = format!("INT_TYPE_{word}");
                let mut from = 0;
                while let Some(at) = line[from..].find(&needle) {
                    let at = from + at;
                    let end = at + needle.len();
                    from = end;
                    if line[end..].starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_') {
                        continue; // INT_TYPE_MOVED_TYPE and the like
                    }
                    let before = &line[..at];
                    let kind = if before.contains("set_local_entity_int_value")
                        || before.contains("set_client_server_entity_int_value")
                        || before.contains("set_client_server_local_entity_int_value")
                    {
                        "set"
                    } else if before.contains("get_local_entity_int_value") || before.contains("CAMPAIGN_OFFER_UPDATED") {
                        "get"
                    } else {
                        "other"
                    };
                    let rel = path.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
                    *out.entry(rel).or_default().entry(format!("{word}:{kind}")).or_default() += 1;
                }
            }
        }
    }
    out
}

#[test]
fn every_writer_and_reader_of_the_three_words_is_the_known_census() {
    let wanted: &[(&str, &[(&str, usize)])] = &[
        ("entity/en_misc/en_maths/en_cllsn.c", &[("UPDATED:get", 2)]),
        ("entity/mobile/aircraft/fixwing/fw_move.c", &[("ROTATED:set", 1)]),
        ("entity/mobile/aircraft/fixwing/fw_msgs.c", &[("UPDATED:get", 1)]),
        ("entity/mobile/aircraft/fixwing/fw_updt.c", &[("ROTATED:get", 2)]),
        ("entity/mobile/aircraft/helicop/hc_move.c", &[("MOVED:set", 1), ("ROTATED:set", 1)]),
        ("entity/mobile/aircraft/helicop/hc_updt.c", &[("MOVED:get", 1), ("ROTATED:get", 2)]),
        (
            "entity/mobile/mb_int.c",
            &[
                ("MOVED:get", 1),
                // the two "other" mentions the hosted branches add to each of MOVED and UPDATED (Task49): the type-selecting conditional of the hosted setter and of the hosted getter (`type == INT_TYPE_MOVED`), whose meaning is pinned by
                // `the_hosted_build_holds_no_flag_array_and_every_access_goes_to_the_host` below; ROTATED is the else branch of the same conditional and is not named again
                ("MOVED:other", 7),
                ("MOVED:set", 1),
                ("ROTATED:get", 1),
                ("ROTATED:other", 3),
                ("ROTATED:set", 1),
                ("UPDATED:get", 1),
                ("UPDATED:other", 7),
                ("UPDATED:set", 1),
            ],
        ),
        ("entity/mobile/vehicle/anti_air/aa_updt.c", &[("ROTATED:get", 2)]),
        ("entity/mobile/vehicle/routed/rv_hostmv.c", &[("MOVED:set", 2), ("ROTATED:set", 4)]),
        ("entity/mobile/vehicle/routed/rv_move.c", &[("ROTATED:set", 1)]),
        ("entity/mobile/vehicle/routed/rv_updt.c", &[("ROTATED:get", 2)]),
        ("entity/special/guide/gd_supply_arrival.c", &[("UPDATED:get", 1)]),
        ("entity/special/update/up_update.c", &[("UPDATED:set", 1)]),
        ("gunships/dynamics/dynamics.c", &[("MOVED:set", 1), ("ROTATED:set", 1)]),
    ];
    let got = census();
    let want: BTreeMap<String, BTreeMap<String, usize>> = wanted
        .iter()
        .map(|(f, rows)| (f.to_string(), rows.iter().map(|(k, v)| (k.to_string(), *v)).collect()))
        .collect();
    assert_eq!(got, want, "a writer or reader of the UPDATED / MOVED / ROTATED word appeared, vanished or changed kind: the contract in docs/d3-physical-flags-contract.md must be re-read");
}

#[test]
fn the_getter_the_setter_and_the_reset_are_the_source_the_characterisation_ran() {
    let mb = strip_comments(&read(&aphavoc().join("entity/mobile/mb_int.c")));
    // the setter ORs a bit (TRUE only), the getter ANDs the word with the mask and returns the result
    assert!(
        mb.contains("updated_entities [index] |= mask;") && mb.contains("moved_entities [index] |= mask;") && mb.contains("rotated_entities [index] |= mask;")
    );
    assert!(
        mb.contains("value = updated_entities [index] &= mask;")
            && mb.contains("value = moved_entities [index] &= mask;")
            && mb.contains("value = rotated_entities [index] &= mask;")
    );
    assert!(
        mb.contains("index = get_local_entity_index (en) >> 5;")
            && mb.contains("offset = get_local_entity_index (en) & 0x1f;")
            && mb.contains("mask = 1 << offset;")
    );
    assert!(mb.contains("ASSERT (value == TRUE);"));
    // the arrays: three words of MAX_NUM_ENTITIES / 32 unsigned ints; the reset counts BYTES, a quarter of them
    let up = strip_comments(&read(&aphavoc().join("entity/special/update/up_update.c")));
    for array in ["moved_entities", "rotated_entities", "updated_entities"] {
        assert!(
            up.contains(&format!("{array}[MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8)]"))
                || up.contains(&format!("{array} [MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8)]")),
            "{array}"
        );
        assert!(
            up.contains(&format!("memset ({array}, 0, MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8));")),
            "the reset of {array} is the quarter-memset"
        );
    }
    assert!(
        up.contains("if (tacview_is_logging() && tacview_reset_frame())"),
        "MOVED and ROTATED are reset only behind the Tacview frame reset"
    );
    assert!(
        up.contains("update_client_server_entity (en);") && up.contains("set_local_entity_int_value (en, INT_TYPE_UPDATED, TRUE);"),
        "UPDATED is set AFTER the entity's own update"
    );
    let heap = strip_comments(&read(&aphavoc().join("entity/system/en_main/en_heap.h")));
    assert!(
        heap.contains("#define MAX_NUM_ENTITIES\t\t\t\t(131072 * 4)") || heap.contains("MAX_NUM_ENTITIES"),
        "{heap}"
    );
    // the family: only the aircraft, vehicle, cargo and weapon initialisers install the mobile functions (and so the three words)
    let mut installers = 0;
    for rel in [
        "entity/mobile/aircraft/ac_funcs.c",
        "entity/mobile/vehicle/vh_funcs.c",
        "entity/mobile/cargo/cg_funcs.c",
        "entity/mobile/weapon/wn_funcs.c",
    ] {
        if strip_comments(&read(&aphavoc().join(rel))).contains("overload_mobile_functions (") {
            installers += 1;
        }
    }
    assert_eq!(installers, 4);
}

#[test]
fn the_supply_arrival_descriptor_normalises_only_its_own_value_to_the_boolean_the_rust_side_requires() {
    // HISTORY: the characterised ae367ca3a producer stored the getter's raw MASK in the descriptor (`descriptor.updated = get_local_entity_int_value (member, INT_TYPE_UPDATED);`), which the Rust side's
    // boolean check refused for every mask but lane 0's 1 (measured by the supply control in both profiles; that evidence and source are retained). The technical lead's decision corrected exactly that one
    // assignment, and ONLY the descriptor's own value: the single getter call is unchanged and the Rust boundary is NOT weakened. This test pins the corrected text the control now runs against
    let c = strip_comments(&read(&aphavoc().join("entity/special/guide/gd_supply_arrival.c")));
    assert!(
        c.contains("descriptor.updated = get_local_entity_int_value (member, INT_TYPE_UPDATED) ? TRUE : FALSE;"),
        "the descriptor carries the getter's value as the C boolean: zero is FALSE, every nonzero raw mask (bit 31 too) is TRUE"
    );
    assert!(
        !c.contains("descriptor.updated = get_local_entity_int_value (member, INT_TYPE_UPDATED);"),
        "the raw assignment of ae367ca3a is gone"
    );
    // the Rust boundary is exactly as characterised: it accepts only 0 and 1 (the correction satisfies it, it does not widen it)
    let rust = std::fs::read_to_string(crate_dir().join("../eech-engine/src/lib.rs")).unwrap();
    assert!(rust.contains("updated: checked_bool(\"updated\", source.updated)?,"));
    let start = rust.find("fn checked_bool(name: &str, value: c_int)").expect("checked_bool");
    let body = &rust[start..start + 260];
    assert!(
        body.contains("0 => Ok(false)") && body.contains("1 => Ok(true)") && body.contains("_ => Err("),
        "checked_bool accepts exactly 0 and 1: {body}"
    );
    // the neighbouring flag of the same function was already normalised the same way: the descriptor is now symmetric
    assert!(c.contains("get_local_entity_int_value (member, INT_TYPE_WEAPON_AND_TARGET_VECTORS_VALID) ? TRUE : FALSE"));
}

#[test]
fn the_consumer_source_is_the_one_the_real_consumer_control_ran() {
    // the response the control calls through the real message table (csrc/eech_physical_flags_consumer_check.c); a change of its reads, its order or its installation is a named failure here
    let msgs = strip_comments(&read(&aphavoc().join("entity/mobile/aircraft/fixwing/fw_msgs.c")));
    let start = msgs
        .find("static int response_to_check_mobile_reached_guide (entity_messages message, entity *receiver, entity *sender, va_list pargs)")
        .expect("the response");
    let end = start + msgs[start..].find("void overload_fixed_wing_message_responses (void)").expect("the installer");
    let body = &msgs[start..end];
    let order: Vec<usize> = [
        "get_local_entity_int_value (receiver, INT_TYPE_LANDED)",
        "get_local_entity_float_value (receiver, FLOAT_TYPE_DISTANCE)",
        "get_local_entity_float_value (receiver, FLOAT_TYPE_VELOCITY)",
        "get_local_entity_vec3d_ptr (receiver, VEC3D_TYPE_MOTION_VECTOR)",
        "get_local_entity_vec3d_ptr (receiver, VEC3D_TYPE_POSITION)",
        "get_local_entity_int_value (receiver, INT_TYPE_UPDATED)",
        "get_local_entity_vec3d (receiver, VEC3D_TYPE_GUIDE_POSITION, &guide_pos)",
        "get_3d_perp_dist_of_point_from_line (pos1, &pos2, &guide_pos, NULL)",
    ]
    .iter()
    .map(|needle| body.find(needle).unwrap_or_else(|| panic!("the response no longer contains `{needle}`")))
    .collect();
    assert!(order.windows(2).all(|w| w[0] < w[1]), "the reads of the response changed order: {order:?}");
    assert_eq!(body.matches("INT_TYPE_UPDATED").count(), 1, "the raw word is read exactly once");
    assert!(
        body.contains("pos1->x - (mv->x * get_delta_time ())") && body.contains("pos1->x + (mv->x * get_delta_time ())"),
        "the previous and the future position"
    );
    assert!(
        msgs.contains("message_responses[ENTITY_TYPE_FIXED_WING][ENTITY_MESSAGE_CHECK_MOBILE_REACHED_GUIDE]\t= response_to_check_mobile_reached_guide;"),
        "the installation"
    );
    // the guide position is read from the RECEIVER; only a guide registers that vec3d type (the fixed wing keeps the engine's default getter)
    let registrations: Vec<String> = [
        "entity/special/guide/gd_vec3d.c",
        "entity/mobile/mb_vec3d.c",
        "entity/mobile/aircraft/ac_vec3d.c",
        "entity/mobile/aircraft/fixwing/fw_vec3d.c",
    ]
    .iter()
    .filter_map(|rel| {
        let path = aphavoc().join(rel);
        path.exists().then(|| strip_comments(&read(&path)))
    })
    .filter(|text| text.contains("VEC3D_TYPE_GUIDE_POSITION"))
    .map(|text| {
        text.lines()
            .filter(|l| l.contains("fn_get_local_entity_vec3d") && l.contains("VEC3D_TYPE_GUIDE_POSITION"))
            .collect::<Vec<_>>()
            .join("\n")
    })
    .collect();
    assert_eq!(
        registrations.len(),
        1,
        "exactly one source registers the GUIDE_POSITION vec3d getter: {registrations:?}"
    );
    assert!(registrations[0].contains("ENTITY_TYPE_GUIDE"), "{registrations:?}");
    // the shipping profile routes the arrival to the campaign's own checks first; the original response is the one the profile retains for the other cases
    let guide = strip_comments(&read(&aphavoc().join("entity/special/guide/guide.c")));
    let router = guide
        .find("static int check_mobile_reached_waypoint (entity *member, entity *guide)")
        .expect("the router");
    let routed = &guide[router..router + 700];
    assert!(routed.find("campaign_supply_arrival_is_target").unwrap() < routed.find("campaign_radius_arrival_is_target").unwrap());
    assert!(
        routed.find("campaign_radius_arrival_is_target").unwrap()
            < routed
                .find("notify_local_entity (ENTITY_MESSAGE_CHECK_MOBILE_REACHED_GUIDE, member, guide)")
                .unwrap()
    );
}

/// the build as the HOSTED profile compiles it: the lines inside `#ifndef EECH_CAMPAIGN_HOST_ENTITY` (and the `#else` of an `#ifdef EECH_CAMPAIGN_HOST_ENTITY`) are dropped, every other conditional is kept as it is
fn hosted_view(text: &str) -> String {
    // each open conditional: (is it the hosted macro's, is its branch active in the hosted profile)
    let mut stack: Vec<(bool, bool)> = Vec::new();
    let mut out = String::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#ifdef EECH_CAMPAIGN_HOST_ENTITY") {
            stack.push((true, true));
        } else if trimmed.starts_with("#ifndef EECH_CAMPAIGN_HOST_ENTITY") {
            stack.push((true, false));
        } else if trimmed.starts_with("#if") {
            stack.push((false, true));
        } else if trimmed.starts_with("#else") {
            if let Some(top) = stack.last_mut() {
                if top.0 {
                    top.1 = !top.1;
                }
            }
        } else if trimmed.starts_with("#endif") {
            stack.pop();
        } else if stack.iter().all(|(ours, active)| !ours || *active) {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

#[test]
fn the_hosted_build_holds_no_flag_array_and_every_access_goes_to_the_host() {
    // Task49: in the hosted profile (EECH_CAMPAIGN_HOST_ENTITY, the default build) the three arrays are not defined or declared, so nothing can hold a bit of them, and the setter, the destructive getter and the three resets
    // are the host's. The ORIGINAL profile (feature collision-fixture) keeps every original line, which the other tests here and the 199 and 56 expectations of the original profile pin.
    let mb = hosted_view(&strip_comments(&read(&aphavoc().join("entity/mobile/mb_int.c"))));
    let up = hosted_view(&strip_comments(&read(&aphavoc().join("entity/special/update/up_update.c"))));
    let header = hosted_view(&strip_comments(&read(&aphavoc().join("entity/special/update/up_update.h"))));
    for (name, view) in [("mb_int.c", &mb), ("up_update.c", &up), ("up_update.h", &header)] {
        for array in ["moved_entities", "rotated_entities", "updated_entities"] {
            assert!(!view.contains(array), "the hosted build of {name} still names {array}");
        }
    }
    // no other source of the engine names an array (the hosted view of a file that has no hosted branch is the file)
    let root = aphavoc();
    let mut files = Vec::new();
    walk(&root, &mut files);
    for path in files {
        let rel = path.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
        if matches!(
            rel.as_str(),
            "entity/mobile/mb_int.c" | "entity/special/update/up_update.c" | "entity/special/update/up_update.h"
        ) {
            continue;
        }
        let view = hosted_view(&strip_comments(&read(&path)));
        for array in ["moved_entities", "rotated_entities", "updated_entities"] {
            assert!(!view.contains(array), "{rel} names {array}");
        }
    }
    // the setter is the host's set (TRUE-only: the assertion stays) and the getter returns the raw word the host answered, for the one array each type selects
    assert!(mb.contains("ASSERT (value == TRUE);"));
    assert!(mb.contains("campaign_model_flag_set ((type == INT_TYPE_UPDATED) ? CAMPAIGN_FLAG_ARRAY_UPDATED : ((type == INT_TYPE_MOVED) ? CAMPAIGN_FLAG_ARRAY_MOVED : CAMPAIGN_FLAG_ARRAY_ROTATED), get_local_entity_index (en));"));
    assert!(mb.contains("value = campaign_model_flag_read ((type == INT_TYPE_UPDATED) ? CAMPAIGN_FLAG_ARRAY_UPDATED : ((type == INT_TYPE_MOVED) ? CAMPAIGN_FLAG_ARRAY_MOVED : CAMPAIGN_FLAG_ARRAY_ROTATED), get_local_entity_index (en));"));
    assert_eq!(
        mb.matches("campaign_model_flag_").count(),
        2,
        "the hosted getter and setter each make exactly one host call"
    );
    // the resets, in the original's order and conditions: MOVED then ROTATED behind the Tacview frame reset, UPDATED once per update iteration before the update list is walked
    let tacview = up.find("if (tacview_is_logging() && tacview_reset_frame())").expect("the Tacview condition");
    let moved = up.find("campaign_model_flag_reset (CAMPAIGN_FLAG_ARRAY_MOVED);").expect("the MOVED reset");
    let rotated = up.find("campaign_model_flag_reset (CAMPAIGN_FLAG_ARRAY_ROTATED);").expect("the ROTATED reset");
    let loop_head = up.find("for (loop = 0; loop < iterations; loop ++)").expect("the iteration loop");
    let updated = up.find("campaign_model_flag_reset (CAMPAIGN_FLAG_ARRAY_UPDATED);").expect("the UPDATED reset");
    let walk_list = up
        .find("en = get_local_entity_first_child (get_update_entity (), LIST_TYPE_UPDATE);")
        .expect("the update list walk");
    let set_updated = up.find("set_local_entity_int_value (en, INT_TYPE_UPDATED, TRUE);").expect("the UPDATED set");
    assert!(
        tacview < moved && moved < rotated && rotated < loop_head && loop_head < updated && updated < walk_list && walk_list < set_updated,
        "the order of the resets and the set changed"
    );
    assert_eq!(up.matches("campaign_model_flag_reset (").count(), 3, "exactly the three resets");
    // the original profile still has the quarter-memsets (hosted_view of the ORIGINAL is the other branch): the reset EXTENT is not a Core constant any more but the one fact the campaign states per group
    let model = strip_comments(&read(&aphavoc().join("entity/mobile/mb_hostmodel.c")));
    assert!(
        model.contains("#define CAMPAIGN_FLAG_RESET_GROUPS ((MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8)) / sizeof (unsigned int))"),
        "the reset extent is the number of WORDS the original's byte-counted memset clears"
    );
    assert!(
        model.contains("(unsigned int) group < CAMPAIGN_FLAG_RESET_GROUPS"),
        "the campaign states a group's eligibility from its word index"
    );
    assert!(
        model.contains("campaign_entity_flag_exchange (CAMPAIGN_SCAN_PHASE_FLAG_SET, array, slot & 31, flag_group_key (slot), FALSE, NULL, NULL);"),
        "a set names the group by the host's key and the lane by the slot's five low bits, and never states the slot"
    );
    // the dialogue never sends a slot, a group number or an index: the only facts it states are the eligibility, the array and the lane
    let dialogue = strip_comments(&read(&aphavoc().join("entity/mobile/mb_hostentity.h")));
    let start = dialogue.find("static void campaign_entity_flag_exchange (").expect("the flag exchange");
    let body = &dialogue[start..];
    let body = &body[..body.find("\n}\n").expect("its end")];
    for stated in ["CAMPAIGN_FLAG_ELIGIBLE", "CAMPAIGN_FLAG_ARRAY", "CAMPAIGN_FLAG_LANE"] {
        assert!(body.contains(&format!("request.pose [{stated}]")), "{stated}");
    }
    assert_eq!(
        body.matches("request.pose [").count(),
        4,
        "the phase and the three flag facts are the only facts stated"
    );
    assert!(
        !body.contains("slot") && !body.contains(">> 5") && !body.contains("& 31"),
        "the dialogue derives and sends no slot or group number"
    );
}

#[test]
fn the_collision_pass_through_source_is_the_one_the_collision_control_ran_and_the_hosted_path_reads_the_word_once_when_the_host_asks() {
    // the ORIGINAL body the control runs (csrc/eech_physical_flags_collision_check.c) is the moving-mobile branch of get_line_of_sight_collision_entity: the speed test, then the raw UPDATED read, then the two position derivations with the
    // delta time; the hosted build compiles campaign_scan_entities instead and the campaign reads the word ONLY where the host asks for it (mb_hostoffer.h NEED_UPDATED) and states 2.0 or 1.0
    let cllsn = strip_comments(&read(&aphavoc().join("entity/en_misc/en_maths/en_cllsn.c")));
    let order: Vec<usize> = [
        "if (sqr_velocity > 1.0)",
        "if (get_local_entity_int_value (target, INT_TYPE_UPDATED))",
        "target_old_position.x = target_new_position.x - target_motion_vector->x * get_delta_time ();",
        "target_new_position.x = target_old_position.x + target_motion_vector->x * get_delta_time ();",
    ]
    .iter()
    .map(|needle| cllsn.find(needle).unwrap_or_else(|| panic!("en_cllsn.c no longer contains `{needle}`")))
    .collect();
    assert!(
        cllsn[order[3]..].contains("accurate_moving_bounding_box_collision_test"),
        "the narrow phase follows the derivations"
    );
    assert!(order.windows(2).all(|w| w[0] < w[1]), "the moving-mobile branch changed order: {order:?}");
    assert_eq!(
        cllsn.matches("INT_TYPE_UPDATED").count(),
        2,
        "the raw word is read in the original body and in the hosted binding of the offer, and nowhere else of this file"
    );
    assert!(
        cllsn.contains("return (campaign_scan_entities (los_source, los_target"),
        "the hosted build asks the host's scan instead of the body"
    );
    let offer = strip_comments(&read(&aphavoc().join("entity/mobile/mb_hostoffer.h")));
    assert!(
        offer.contains("facts [CAMPAIGN_CANDIDATE_UPDATED] = CAMPAIGN_OFFER_UPDATED (target) ? 2.0f : 1.0f;")
            && offer.matches("CAMPAIGN_OFFER_UPDATED (").count() == 1,
        "the hosted offer reads the raw word once, when the host asks for it, and states a state, not the mask"
    );
}

#[test]
fn the_supply_control_seam_exists_only_behind_its_feature_and_the_consumer_reads_the_getter_once_before_it_asks_the_host() {
    let manifests = [
        "eech-engine-sys",
        "eech-engine",
        "eech-dc",
        "eech-world",
        "eech-campaign",
        "eech-harness",
        "eech-sys",
        "eech-map",
    ];
    for name in manifests {
        let text = std::fs::read_to_string(crate_dir().join("..").join(name).join("Cargo.toml")).unwrap();
        let mentions = text.matches("physical-flags-supply-check").count();
        match name {
            "eech-engine-sys" => assert_eq!(mentions, 1, "the feature is declared once, empty: {text}"),
            "eech-engine" => assert_eq!(mentions, 3, "declared, forwarded and the one test that requires it: {name}"),
            _ => assert_eq!(mentions, 0, "{name} must not enable the supply control feature"),
        }
        let default = text.lines().find(|l| l.trim_start().starts_with("default")).unwrap_or("");
        assert!(!default.contains("supply"), "{name}: a default feature enables the control: {default}");
    }
    let sys = std::fs::read_to_string(crate_dir().join("Cargo.toml")).unwrap();
    assert!(sys.contains("\nphysical-flags-supply-check = []\n"));
    let build = std::fs::read_to_string(crate_dir().join("build/main.rs")).unwrap();
    assert_eq!(build.matches("EECH_PHYSICAL_FLAGS_SUPPLY_CHECK").count(), 1);
    assert!(build.contains(
        "if env::var_os(\"CARGO_FEATURE_PHYSICAL_FLAGS_SUPPLY_CHECK\").is_some() {\n        build.define(\"EECH_PHYSICAL_FLAGS_SUPPLY_CHECK\", None);"
    ));
    // the C world is entirely behind the define
    let fixture = read(&crate_dir().join("csrc/eech_physical_flags_supply_check.c"));
    let code = strip_comments(&fixture);
    assert!(
        code.trim_start().starts_with("#ifdef EECH_PHYSICAL_FLAGS_SUPPLY_CHECK") && code.trim_end().ends_with("#endif"),
        "the fixture is not wholly behind its define"
    );
    assert_eq!(code.matches("#ifdef EECH_PHYSICAL_FLAGS_SUPPLY_CHECK").count(), 1);
    // the engine's seam is behind the feature, is the one `pub` name added to the crate, and serves no other callback
    let lib = std::fs::read_to_string(crate_dir().join("../eech-engine/src/lib.rs")).unwrap();
    assert_eq!(lib.matches("pub mod supply_check").count(), 1);
    assert!(lib.contains("#[cfg(feature = \"physical-flags-supply-check\")]\npub mod supply_check {"));
    // the test-only key helpers of mb_hostmodel.c: the feature joins the gate they already had
    let model = read(&aphavoc().join("entity/mobile/mb_hostmodel.c"));
    assert!(
        model.contains("|| defined(EECH_PHYSICAL_FLAGS_SUPPLY_CHECK) || defined(EECH_PHYSICAL_FLAGS_MOTION_CHECK)\n"),
        "the gate of the test-only key helpers no longer names the supply and motion controls"
    );
    // the CONSUMER: the fixed-wing branch reads the getter into the descriptor (as the corrected boolean), ONCE, and the host is asked after it (before any gate)
    let consumer = strip_comments(&read(&aphavoc().join("entity/special/guide/gd_supply_arrival.c")));
    assert_eq!(
        consumer.matches("INT_TYPE_UPDATED").count(),
        1,
        "the supply arrival reads the word in one place"
    );
    let read_at = consumer
        .find("descriptor.updated = get_local_entity_int_value (member, INT_TYPE_UPDATED) ? TRUE : FALSE;")
        .expect("the descriptor read");
    let branch_at = consumer
        .find("if (mobile == CAMPAIGN_SUPPLY_MOBILE_FIXED_WING)\n\t{")
        .expect("the fixed-wing branch");
    let helicopter_at = consumer
        .find("else if (mobile == CAMPAIGN_SUPPLY_MOBILE_HELICOPTER)\n\t{")
        .expect("the helicopter branch");
    let ask_at = consumer.find("result = ask_host (&descriptor);").expect("the host question");
    assert!(
        branch_at < read_at && read_at < helicopter_at && helicopter_at < ask_at,
        "the read is in the fixed-wing branch and before the question to the host: {branch_at} {read_at} {helicopter_at} {ask_at}"
    );
}

/// every byte offset at which `needle` starts in `hay`
fn offsets(hay: &str, needle: &str) -> Vec<usize> {
    hay.match_indices(needle).map(|(at, _)| at).collect()
}

/// the nearest `if (` line before `at`: the conditional whose block holds the text at `at` (the Tacview readers put the read on the second line of the block their guard opens)
fn nearest_if_line_before(code: &str, at: usize) -> String {
    code[..at]
        .lines()
        .rev()
        .map(str::trim)
        .find(|l| l.starts_with("if (") || l.starts_with("if("))
        .unwrap_or("")
        .to_string()
}

/// the three positive conjunctive conditions of the current sources: the update functions of the aircraft (`raw->ac`) and the vehicles (`raw->vh`), and the frame reset; the logging macro is the FIRST operand of an AND in each
const LOGGING_GUARDS: [&str; 3] = [
    "if (tacview_is_logging() && raw->ac.mob.alive)",
    "if (tacview_is_logging() && raw->vh.mob.alive)",
    "if (tacview_is_logging() && tacview_reset_frame())",
];

/// THE predicate the Tacview dormancy guard uses for every read of MOVED / ROTATED and every reset of both words: is the text at `at` inside the braces of the block a `if (tacview_is_logging () ...)` opens.
/// Brace matching over comment-stripped text, no more: the block is the `{` that ends the condition's line or is the next thing after it, to its matching `}`; a text after that closing brace is OUTSIDE.
/// (The first version of this guard accepted the nearest preceding `if (` line instead, which also accepts a text placed after the block's closing brace: the retained red run r1.)
/// Only the three exact positive conditions of LOGGING_GUARDS select a block (any other condition containing the macro, a negated one or an OR arm, is skipped: fail closed).
fn inside_logging_block(code: &str, at: usize) -> bool {
    for start in offsets(code, "tacview_is_logging()") {
        let line_start = code[..start].rfind('\n').map_or(0, |i| i + 1);
        if !code[line_start..start].trim_start().starts_with("if (") {
            continue; // the macro's own definition line, a call that is not a condition
        }
        // fail closed on every condition that is not one of the three exact positive guards of the current sources (whitespace runs collapsed, a trailing `{` ignored): a negated macro or an OR arm
        // would also contain the macro's text and can run while the file is closed
        let line_end = code[start..].find('\n').map_or(code.len(), |i| start + i);
        let condition = code[line_start..line_end].trim_end().trim_end_matches('{');
        if !LOGGING_GUARDS.contains(&condition.split_whitespace().collect::<Vec<_>>().join(" ").as_str()) {
            continue;
        }
        let rest = &code[start..];
        let eol = rest.find('\n').unwrap_or(rest.len());
        let open = match rest[..eol].find('{') {
            Some(i) => start + i,
            None if rest[eol..].trim_start().starts_with('{') => start + eol + rest[eol..].find('{').unwrap(),
            None => continue, // a brace-less conditional opens no block
        };
        let mut depth = 0usize;
        for (i, c) in code[open..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        if at > open && at < open + i {
                            return true;
                        }
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    false
}

/// does the header's `tacview_is_logging ()` mean "the log file is open" (`tacview_log_file != NULL`), the relation every logging conditional rests on
fn macro_means_open_log_file(header: &str) -> bool {
    header.lines().any(|l| {
        l.contains("#define tacview_is_logging()")
            && l.split("tacview_is_logging()")
                .nth(1)
                .is_some_and(|rest| rest.trim() == "(tacview_log_file != NULL)")
    })
}

/// the files under the original sources whose comment-stripped text contains `needle`, as `relative path -> occurrences`
fn files_naming(needle: &str) -> BTreeMap<String, usize> {
    let root = aphavoc();
    let mut paths = Vec::new();
    walk(&root, &mut paths);
    let mut found = BTreeMap::new();
    for path in paths {
        let n = offsets(&strip_comments(&read(&path)), needle).len();
        if n > 0 {
            found.insert(path.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/"), n);
        }
    }
    found
}

#[test]
fn the_legacy_tacview_writer_is_compiled_in_but_every_read_of_moved_and_rotated_waits_on_a_log_file_the_headless_boot_never_opens() {
    // THE FINDING (ENV-PHYSICAL-FLAGS-HOST-STORE-NATIVE-ACCEPTANCE-1, the Tacview consumer, source text only): in the shipping EECH_DC engine the C Tacview writer (aphavoc/source/entity/tacview) is COMPILED IN, but
    // `tacview_is_logging ()` is `tacview_log_file != NULL`, the file is opened only by `start_tacview_logging`, and the headless boot reaches neither of its two callers. Every read of MOVED or ROTATED and the
    // frame reset of both words sit behind that condition, so in the headless shipping path they are never executed; only the SETTERS of the words run there. This pins the text the disposition relies on: a new
    // caller, a new unguarded reader or a boot that starts logging fails here and the consumer must then be characterised in both profiles. It is not a native, boot or lifetime proof.
    let sources = std::fs::read_to_string(crate_dir().join("sources.txt")).unwrap();
    assert!(
        sources.lines().any(|l| l.trim() == "aphavoc/source/entity/tacview/tacview.c"),
        "the writer is compiled, not stubbed: it is listed in the engine's sources"
    );
    // the condition every guard below rests on: the header's macro is exactly "the log file is open"
    assert!(
        macro_means_open_log_file(&strip_comments(&read(&aphavoc().join("entity/tacview/tacview.h")))),
        "tacview.h: tacview_is_logging () is no longer `tacview_log_file != NULL`"
    );
    let tacview_c = strip_comments(&read(&aphavoc().join("entity/tacview/tacview.c")));
    assert!(tacview_c.contains("FILE* tacview_log_file = NULL;"), "the log file starts unopened");

    // 1. the file is assigned only in tacview.c (the initialiser, the open and the closes), and opened only by the one static open, whose only caller is start_tacview_logging
    assert_eq!(
        files_naming("tacview_log_file = ").keys().cloned().collect::<Vec<_>>(),
        vec!["entity/tacview/tacview.c".to_string()],
        "another file assigns the log file"
    );
    assert_eq!(
        offsets(&tacview_c, "tacview_log_file = safe_fopen(").len(),
        2,
        "the log file is assigned from an open in exactly two places"
    );
    assert_eq!(
        offsets(&tacview_c, "tacview_log_file = safe_fopen(log_filename, \"wb\");").len(),
        1,
        "one FIRST open: open_tacview_log"
    );
    // the second is the periodic REOPEN of an already open file (it closes it first, which a never-opened file could not survive)
    assert_eq!(
        offsets(
            &tacview_c,
            "fclose(tacview_log_file);\n\t\ttacview_log_file = NULL;\n\t\ttacview_log_file = safe_fopen(log_filename, \"ab\");"
        )
        .len(),
        1
    );
    assert_eq!(offsets(&tacview_c, "open_tacview_log();").len(), 1, "the open has one caller");
    let start = tacview_c.find("void start_tacview_logging(entity* gunship)").expect("start_tacview_logging");
    assert!(
        start < tacview_c.find("open_tacview_log();").unwrap(),
        "the open is called from start_tacview_logging"
    );

    // 2. start_tacview_logging has exactly two callers: the user's gunship assignment (`assign_entity_to_user`, helicop.c) and the flight loop's entry (`flight`, flight.c, only for logging mode 3)
    let starters: BTreeMap<String, usize> = files_naming("start_tacview_logging");
    let want: BTreeMap<String, usize> = [
        ("entity/mobile/aircraft/helicop/helicop.c", 1),
        ("entity/tacview/tacview.c", 1),
        ("entity/tacview/tacview.h", 1),
        ("flight.c", 1),
    ]
    .iter()
    .map(|(f, n)| (f.to_string(), *n))
    .collect();
    assert_eq!(starters, want, "start_tacview_logging gained, lost or moved a caller");
    let helicop = strip_comments(&read(&aphavoc().join("entity/mobile/aircraft/helicop/helicop.c")));
    let call = helicop.find("start_tacview_logging(en);").unwrap();
    assert!(
        helicop[..call].rfind("void assign_entity_to_user (entity *en)").is_some() && helicop[call..].find("void set_raw_gunship_entity").is_some(),
        "the helicopter caller is inside assign_entity_to_user"
    );
    assert!(nearest_if_line_before(&helicop, call).contains("command_line_tacview_logging && command_line_tacview_logging < 3"));
    let flight = strip_comments(&read(&aphavoc().join("flight.c")));
    assert!(nearest_if_line_before(&flight, flight.find("start_tacview_logging(NULL);").unwrap()).contains("command_line_tacview_logging == 3"));
    // the logging mode defaults to off
    assert!(strip_comments(&read(&aphavoc().join("cmndline.c"))).contains("command_line_tacview_logging\t\t\t\t\t= 0,"));

    // 3. the headless boot starts neither: no engine C file, no Rust of the shipping module and host names the writer, the user assignment, the gunship setter or the flight loop's entry
    let names = [
        "start_tacview_logging",
        "tacview_log_file",
        "command_line_tacview_logging",
        "tacview_reset_frame",
        "write_tacview",
        "assign_entity_to_user",
        "set_gunship_entity",
    ];
    let crates = crate_dir().join("..");
    let mut shipping = Vec::new();
    for dir in [
        "eech-engine-sys/csrc",
        "eech-engine-sys/src",
        "eech-engine/src",
        "eech-dc/src",
        "eech-world/src",
        "eech-world/csrc",
    ] {
        let dir = crates.join(dir);
        for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
            let path = entry.unwrap().path();
            if matches!(path.extension().and_then(|e| e.to_str()), Some("c" | "h" | "rs")) {
                shipping.push(path);
            }
        }
    }
    assert!(shipping.len() > 20, "the scan reached the shipping sources: {}", shipping.len());
    for path in &shipping {
        let text = strip_comments(&read(path));
        for name in names {
            assert!(
                !text.contains(name),
                "{} names {name}: the headless boot may now start the legacy writer",
                path.display()
            );
        }
    }
    let engine = strip_comments(&read(&crate_dir().join("csrc/eech_engine.c")));
    assert!(
        !engine.contains("flight ()") && !engine.contains("flight();"),
        "the headless boot calls the flight loop's entry"
    );
    // `flight ()` is called from one place only (the game phases, which the headless boot replaces by its own sequence)
    assert_eq!(
        files_naming("\tflight ();").keys().cloned().collect::<Vec<_>>(),
        vec!["gameflow/gameflow.c".to_string()]
    );

    // 4. EVERY read of MOVED and ROTATED is inside a block opened by `if (tacview_is_logging () ...)`, the frame reset of both words likewise
    let readers = [
        "entity/mobile/aircraft/fixwing/fw_updt.c",
        "entity/mobile/aircraft/helicop/hc_updt.c",
        "entity/mobile/vehicle/anti_air/aa_updt.c",
        "entity/mobile/vehicle/routed/rv_updt.c",
    ];
    let mut reads = 0;
    for file in readers {
        let code = strip_comments(&read(&aphavoc().join(file)));
        for word in ["INT_TYPE_MOVED)", "INT_TYPE_ROTATED)"] {
            for at in offsets(&code, &format!("get_local_entity_int_value(en, {word}")) {
                reads += 1;
                assert!(inside_logging_block(&code, at), "{file}: a read of {word} outside the logging block");
            }
        }
    }
    assert_eq!(reads, 9, "the number of reads of MOVED and ROTATED the census pins (one MOVED, eight ROTATED)");
    let update = strip_comments(&read(&aphavoc().join("entity/special/update/up_update.c")));
    assert_eq!(
        offsets(&update, "if (tacview_is_logging() && tacview_reset_frame())").len(),
        1,
        "the guarded frame reset is the one conditional of its kind"
    );
    for (array, original) in [("MOVED", "memset (moved_entities, 0,"), ("ROTATED", "memset (rotated_entities, 0,")] {
        // the hosted reset and the original profile's memset of the same word: each named once and each a member of the guarded block (real block membership, not distance)
        for text in [format!("campaign_model_flag_reset (CAMPAIGN_FLAG_ARRAY_{array});"), original.to_string()] {
            let at = offsets(&update, &text);
            assert_eq!(at.len(), 1, "{text} is named once");
            assert!(inside_logging_block(&update, at[0]), "{text} is outside the logging block of the frame reset");
        }
        assert_eq!(
            offsets(&update, &format!("CAMPAIGN_FLAG_ARRAY_{array})")).len(),
            1,
            "the {array} reset is named once"
        );
    }
    // the per-entity logging bit is set only by the writer itself
    assert_eq!(
        files_naming("INT_TYPE_TACVIEW_LOGGING, TRUE").keys().cloned().collect::<Vec<_>>(),
        vec!["entity/tacview/tacview.c".to_string()]
    );

    // the guard's own predicates (the SAME functions used above), matched positives and negatives on synthetic text
    let read = "get_local_entity_int_value(en, INT_TYPE_ROTATED)";
    let at = |code: &str, what: &str| code.find(what).unwrap();
    let inside = "if (tacview_is_logging() && raw->ac.mob.alive)\n{\n int rotated = get_local_entity_int_value(en, INT_TYPE_ROTATED);\n}";
    assert!(inside_logging_block(inside, at(inside, read)), "POSITIVE: a read inside the logging block");
    let inside_vh = inside.replace("raw->ac.mob.alive", "raw->vh.mob.alive");
    assert!(
        inside_logging_block(&inside_vh, at(&inside_vh, read)),
        "POSITIVE: the vehicle shape of the same guard"
    );
    // single-fault negatives of the CONDITION (the read is inside the braces in both): the macro negated, and the macro as one arm of an OR; either can run while the file is closed
    let negated = "if (!tacview_is_logging() && raw->ac.mob.alive)\n{\n int rotated = get_local_entity_int_value(en, INT_TYPE_ROTATED);\n}";
    assert!(
        !inside_logging_block(negated, at(negated, read)),
        "NEGATIVE: a read inside a block whose condition NEGATES the logging macro"
    );
    let or_arm = "if (other || tacview_is_logging())\n{\n int rotated = get_local_entity_int_value(en, INT_TYPE_ROTATED);\n}";
    assert!(
        !inside_logging_block(or_arm, at(or_arm, read)),
        "NEGATIVE: a read inside a block whose condition is an OR with the logging macro as one arm"
    );
    let other_if = "if (raw->alive)\n{\n int rotated = get_local_entity_int_value(en, INT_TYPE_ROTATED);\n}";
    assert!(
        !inside_logging_block(other_if, at(other_if, read)),
        "NEGATIVE: a read inside some other conditional"
    );
    let after_brace =
        "if (tacview_is_logging() && raw->ac.mob.alive)\n{\n int moved = 0;\n}\nint rotated = get_local_entity_int_value(en, INT_TYPE_ROTATED);\n";
    assert!(
        !inside_logging_block(after_brace, at(after_brace, read)),
        "NEGATIVE: the same read immediately AFTER the closing brace of the logging block"
    );
    let reset_after =
        "if (tacview_is_logging() && tacview_reset_frame())\n{\n write_tacview_frame_header();\n}\ncampaign_model_flag_reset (CAMPAIGN_FLAG_ARRAY_MOVED);\n";
    assert!(
        !inside_logging_block(reset_after, at(reset_after, "campaign_model_flag_reset")),
        "NEGATIVE: a reset after the closing brace of the guarded block"
    );
    let nested =
        "if (tacview_is_logging() && raw->ac.mob.alive)\n{\n if (x)\n {\n y ();\n }\n int rotated = get_local_entity_int_value(en, INT_TYPE_ROTATED);\n}";
    assert!(
        inside_logging_block(nested, at(nested, read)),
        "POSITIVE: a read after a nested block, still inside the logging block"
    );
    let header = "#define tacview_is_logging()  (tacview_log_file != NULL)\n";
    assert!(macro_means_open_log_file(header), "POSITIVE: the macro is the open-log-file test");
    assert!(
        !macro_means_open_log_file("#define tacview_is_logging()  (1)\n"),
        "NEGATIVE: a macro that is unconditionally true"
    );
    assert!(
        !macro_means_open_log_file("#define tacview_is_logging()  (tacview_log_file == NULL)\n"),
        "NEGATIVE: the inverted macro"
    );
}

/// the name of the function whose column-0 definition line is the nearest one before `at`
fn enclosing_function(code: &str, at: usize) -> String {
    let line = code[..at]
        .lines()
        .rev()
        .find(|l| l.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) && l.contains(" ("))
        .unwrap_or("");
    line.split(" (")
        .next()
        .unwrap_or("")
        .rsplit(' ')
        .next()
        .unwrap_or("")
        .trim_start_matches('*')
        .to_string()
}

#[test]
fn the_motion_responses_set_moved_and_rotated_at_six_known_places_two_live_two_dead_and_two_behind_the_logging_bit() {
    // THE ROLES (the group of helicopter, vehicle, ship and weapon-vector motion responses of ENV-PHYSICAL-FLAGS-HOST-STORE-NATIVE-ACCEPTANCE-1), from source text: the only SETTERS of MOVED and ROTATED in the host-movement
    // responses are the six statements of rv_hostmv.c below, and nothing in them reads either word. Two sit behind the PER-ENTITY Tacview logging bit (the vehicle's and the fixed wing's `result.rotated`; the bit is set
    // only by the Tacview writer, pinned above), two are in public functions that nothing calls, and the two guided helicopter steps are the live ones (the control tests/physical_flags_motion.rs runs them).
    // The ship and the weapon-vector name none of the three words (the census above is exact), which the last assertion states for the two directories.
    let code = strip_comments(&read(&aphavoc().join("entity/mobile/vehicle/routed/rv_hostmv.c")));
    let mut found: Vec<(String, String, String)> = Vec::new();
    for (word, token) in [("MOVED", "INT_TYPE_MOVED, TRUE"), ("ROTATED", "INT_TYPE_ROTATED, TRUE")] {
        for at in offsets(&code, token) {
            let statement = code[..at].rfind("set_local_entity_int_value").expect("the setter call");
            let condition_start = code[..statement].rfind("if (").expect("a conditional before the set");
            let condition_line_end = code[condition_start..].find('\n').map(|i| condition_start + i).expect("the condition's line");
            let condition = code[condition_start..condition_line_end].trim().to_string();
            let between: String = code[condition_line_end..statement].split_whitespace().collect();
            assert_eq!(between, "{", "the set is the FIRST statement of the block its condition opens: {condition}");
            found.push((enclosing_function(&code, condition_start), condition, word.to_string()));
        }
    }
    found.sort();
    let expected: Vec<(String, String, String)> = [
        (
            "campaign_movement_fixed_wing_move",
            "if (raw->ac.mob.tacview_logging && result.rotated)",
            "ROTATED",
        ),
        ("campaign_movement_helicopter_begin", "if (result.moved)", "MOVED"),
        ("campaign_movement_helicopter_end", "if (result.rotated)", "ROTATED"),
        ("campaign_movement_helicopter_guided_begin", "if (result.moved)", "MOVED"),
        ("campaign_movement_helicopter_guided_end", "if (result.rotated)", "ROTATED"),
        ("campaign_movement_move", "if (raw->vh.mob.tacview_logging && result.rotated)", "ROTATED"),
    ]
    .iter()
    .map(|(f, c, w)| (f.to_string(), c.to_string(), w.to_string()))
    .collect();
    assert_eq!(
        found, expected,
        "a motion response gained, lost or changed the condition of a set of MOVED / ROTATED"
    );
    // the file reads neither word (the census pins the same for every file: no `get` of MOVED or ROTATED outside the Tacview readers)
    assert!(
        !code.contains("INT_TYPE_MOVED)") && !code.contains("INT_TYPE_ROTATED)"),
        "rv_hostmv.c reads a word"
    );

    // Read this guard's source census once for all seven role queries.
    let root = aphavoc();
    let mut paths = Vec::new();
    walk(&root, &mut paths);
    let sources: Vec<(String, String)> = paths
        .into_iter()
        .map(|path| {
            (
                path.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/"),
                strip_comments(&read(&path)),
            )
        })
        .collect();
    let naming = |needle: &str| -> BTreeMap<String, usize> {
        sources
            .iter()
            .filter_map(|(path, text)| {
                let n = offsets(text, needle).len();
                (n > 0).then(|| (path.clone(), n))
            })
            .collect()
    };
    // the two public functions of the non-guided helicopter step are called by nothing (their sets are dead), the guided pair by the helicopter's own movement only
    let begins = naming("campaign_movement_helicopter_begin");
    assert_eq!(
        begins.keys().cloned().collect::<Vec<_>>(),
        vec![
            "entity/mobile/vehicle/routed/rv_hostmv.c".to_string(),
            "entity/mobile/vehicle/routed/rv_hostmv.h".to_string()
        ],
        "campaign_movement_helicopter_begin gained a caller: its set is no longer dead"
    );
    let ends = naming("campaign_movement_helicopter_end");
    assert_eq!(
        ends.keys().cloned().collect::<Vec<_>>(),
        vec![
            "entity/mobile/vehicle/routed/rv_hostmv.c".to_string(),
            "entity/mobile/vehicle/routed/rv_hostmv.h".to_string()
        ],
        "campaign_movement_helicopter_end gained a caller: its set is no longer dead"
    );
    for name in ["campaign_movement_helicopter_guided_begin", "campaign_movement_helicopter_guided_end"] {
        assert_eq!(
            naming(name).keys().cloned().collect::<Vec<_>>(),
            vec![
                "entity/mobile/aircraft/helicop/hc_move.c".to_string(),
                "entity/mobile/vehicle/routed/rv_hostmv.c".to_string(),
                "entity/mobile/vehicle/routed/rv_hostmv.h".to_string()
            ],
            "{name} is called from the helicopter's own movement only"
        );
    }

    // the ship and the weapon-vector name none of the three words
    for word in ["INT_TYPE_UPDATED", "INT_TYPE_MOVED", "INT_TYPE_ROTATED"] {
        for file in naming(word).keys() {
            assert!(
                !file.starts_with("entity/mobile/vehicle/ship/") && !file.starts_with("entity/mobile/weapon/"),
                "{file} names {word}: the ship or the weapon-vector now touches a flag word"
            );
        }
    }
}

#[test]
fn the_motion_control_exists_only_behind_its_feature_and_its_world_is_wholly_inside_its_define() {
    for name in [
        "eech-engine-sys",
        "eech-engine",
        "eech-dc",
        "eech-world",
        "eech-campaign",
        "eech-harness",
        "eech-sys",
        "eech-map",
    ] {
        let text = std::fs::read_to_string(crate_dir().join("..").join(name).join("Cargo.toml")).unwrap();
        let mentions = text.matches("physical-flags-motion-check").count();
        match name {
            "eech-engine-sys" => assert_eq!(mentions, 2, "the feature is declared once, empty, and required by the one test: {text}"),
            _ => assert_eq!(mentions, 0, "{name} must not enable the motion control feature"),
        }
        let default = text.lines().find(|l| l.trim_start().starts_with("default")).unwrap_or("");
        assert!(!default.contains("motion"), "{name}: a default feature enables the control: {default}");
    }
    let sys = std::fs::read_to_string(crate_dir().join("Cargo.toml")).unwrap();
    assert!(sys.contains("\nphysical-flags-motion-check = []\n"));
    let build = std::fs::read_to_string(crate_dir().join("build/main.rs")).unwrap();
    assert_eq!(build.matches("EECH_PHYSICAL_FLAGS_MOTION_CHECK").count(), 1);
    assert!(build.contains(
        "if env::var_os(\"CARGO_FEATURE_PHYSICAL_FLAGS_MOTION_CHECK\").is_some() {\n        build.define(\"EECH_PHYSICAL_FLAGS_MOTION_CHECK\", None);"
    ));
    let fixture = strip_comments(&read(&crate_dir().join("csrc/eech_physical_flags_motion_check.c")));
    assert!(
        fixture.trim_start().starts_with("#ifdef EECH_PHYSICAL_FLAGS_MOTION_CHECK") && fixture.trim_end().ends_with("#endif"),
        "the fixture is not wholly behind its define"
    );
    assert_eq!(fixture.matches("#ifdef EECH_PHYSICAL_FLAGS_MOTION_CHECK").count(), 1);
    // the real bodies the control runs are the ones the roles above name
    assert!(fixture.contains("campaign_movement_helicopter_guided_begin (&pool [index])"));
    assert!(fixture.contains("campaign_movement_helicopter_guided_end (&pool [index], &steer, angles)"));
}
