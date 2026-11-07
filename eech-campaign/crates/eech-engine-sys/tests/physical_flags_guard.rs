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
                ("MOVED:other", 5),
                ("MOVED:set", 1),
                ("ROTATED:get", 1),
                ("ROTATED:other", 3),
                ("ROTATED:set", 1),
                ("UPDATED:get", 1),
                ("UPDATED:other", 5),
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
fn the_supply_arrival_descriptor_carries_the_raw_mask_into_a_field_the_rust_side_accepts_only_as_zero_or_one() {
    // a SOURCE-DERIVED contract mismatch (not executed end to end: the Rust half is a private callback): the C side stores the getter's MASK, the Rust side validates a boolean
    let c = strip_comments(&read(&aphavoc().join("entity/special/guide/gd_supply_arrival.c")));
    assert!(
        c.contains("descriptor.updated = get_local_entity_int_value (member, INT_TYPE_UPDATED);"),
        "the descriptor still carries the raw getter value"
    );
    assert!(
        !c.contains("descriptor.updated = get_local_entity_int_value (member, INT_TYPE_UPDATED) ?")
            && !c.contains("descriptor.updated = (get_local_entity_int_value"),
        "no normalisation to a boolean at the descriptor"
    );
    let rust = std::fs::read_to_string(crate_dir().join("../eech-engine/src/lib.rs")).unwrap();
    assert!(rust.contains("updated: checked_bool(\"updated\", source.updated)?,"));
    let start = rust.find("fn checked_bool(name: &str, value: c_int)").expect("checked_bool");
    let body = &rust[start..start + 260];
    assert!(
        body.contains("0 => Ok(false)") && body.contains("1 => Ok(true)") && body.contains("_ => Err("),
        "checked_bool accepts exactly 0 and 1: {body}"
    );
    // the neighbouring flag of the same function IS normalised (`? TRUE : FALSE`): the asymmetry is the finding
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
