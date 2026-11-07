//! SOURCE/BUILD-DERIVED census guard of the facts a sector's tallest-structure height rests on (EEDCS-49, D3, characterisation before the move to the host). The scalar is correct only while EVERY writer and
//! reader is the one this test names, in the order and under the conditions it names: a change that adds a writer, a reader, a recomputation or another condition fails here before any native run.
//! These are text patterns over the engine sources and the build's source list, not an exhaustive preprocessed-symbol or runtime census: where compiled inclusion is stated it is inferred from the build flags and the source list, not measured, and the guard cannot close the whole-D3 final compiled-consumer disposition. They are source evidence of what the supported single-player runtime does and of the routes that exist in source; the behaviour
//! is `sector_height` (feature `sector-height-check`, the original sector code run over declared hand-made data), and a native run is Root's later job (docs/d3-sector-height-contract.md).
//!
//!  1. WRITERS of `sector.tallest_structure_height`: the zero fill at creation (sc_seccreat.c), the raise at the sector-list link response (sc_msgs.c), the unpack (sc_pack.c). No other file writes it,
//!     and nothing lowers or recomputes it: the unlink response does not touch it, and a fixed entity that changes sector is a removal and an insertion (fx_vec3d.c);
//!  2. READERS: the float getter (sc_float.c, the only route to the value), the pack (sc_pack.c) and, through the getter, the hosted scan (mb_hostscan.h, once per served sector), the original scan
//!     (en_cllsn.c, only in the build that keeps the original solver) and the fixed-wing terrain-follow clearance (fw_move.c, before the host's movement step);
//!  3. the value is saved as 32 raw bits (FLOAT_TYPE_PROTOTYPE_1 is FLOAT_PACK_TYPE_UNPACKED), last in the sector's record, for PACK_MODE_SERVER_SESSION only, and the session unpacks the sectors
//!     before any entity (so the insertions of a load come after the restore and can only raise it);
//!  4. the host already consumes the value as a fact of the candidate question (mb_hostoffer.h, entity_candidate_step.c).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn source(relative: &str) -> String {
    let path = repo().join(relative);
    String::from_utf8_lossy(&std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).replace("\r\n", "\n")
}

fn walk(directory: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "c" || extension == "h") {
            found.push(path);
        }
    }
}

/// every engine source file (the aphavoc and modules trees, the source list's origin), read once: its path from the repository root and its text
fn engine_files() -> &'static Vec<(String, String)> {
    static FILES: OnceLock<Vec<(String, String)>> = OnceLock::new();
    FILES.get_or_init(|| {
        let mut files = Vec::new();
        walk(&repo().join("aphavoc/source"), &mut files);
        walk(&repo().join("modules"), &mut files);
        files.sort();
        files
            .into_iter()
            .map(|file| {
                let text = String::from_utf8_lossy(&std::fs::read(&file).unwrap()).replace("\r\n", "\n");
                (file.strip_prefix(repo()).unwrap().to_string_lossy().replace('\\', "/"), text)
            })
            .collect()
    })
}

/// every engine source file with the number of its lines that satisfy `wanted`
fn census(wanted: impl Fn(&str) -> bool) -> Vec<(String, usize)> {
    engine_files()
        .iter()
        .map(|(file, text)| (file.clone(), text.lines().filter(|line| wanted(line)).count()))
        .filter(|(_, count)| *count > 0)
        .collect()
}

fn expected(rows: &[(&str, usize)]) -> Vec<(String, usize)> {
    rows.iter().map(|(file, count)| ((*file).to_string(), *count)).collect()
}

/// the text with every run of whitespace collapsed to one space and the rows of slashes dropped, so a statement is found whatever its layout
fn squeeze(text: &str) -> String {
    text.split_whitespace()
        .filter(|word| !(word.len() >= 4 && word.chars().all(|c| c == '/')))
        .collect::<Vec<_>>()
        .join(" ")
}

fn before(text: &str, first: &str, second: &str, file: &str) {
    let a = text.find(first).unwrap_or_else(|| panic!("{file}: `{first}` is not there"));
    let b = text[a..].find(second).unwrap_or_else(|| panic!("{file}: `{second}` does not follow `{first}`")) + a;
    assert!(a < b);
}

fn once(text: &str, needle: &str, file: &str) -> usize {
    assert_eq!(text.matches(needle).count(), 1, "{file}: `{needle}` must occur exactly once");
    text.find(needle).unwrap()
}

/// the preprocessor conditions that enclose a line (an `else of` names the condition the line is in the else branch of)
fn enclosing(text: &str, line_index: usize) -> Vec<String> {
    let mut stack: Vec<(String, bool)> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if index == line_index {
            break;
        }
        let Some(rest) = line.trim_start().strip_prefix('#') else { continue };
        let rest = rest.trim_start();
        if let Some(name) = rest.strip_prefix("ifdef") {
            stack.push((format!("defined {}", name.trim()), false));
        } else if let Some(name) = rest.strip_prefix("ifndef") {
            stack.push((format!("!defined {}", name.trim()), false));
        } else if let Some(condition) = rest.strip_prefix("if") {
            stack.push((condition.trim().to_string(), false));
        } else if rest.starts_with("elif") || rest.starts_with("else") {
            if let Some(top) = stack.last_mut() {
                top.1 = true;
            }
        } else if rest.starts_with("endif") {
            stack.pop();
        }
    }
    stack
        .into_iter()
        .map(|(condition, in_else)| if in_else { format!("else of {condition}") } else { condition })
        .collect()
}

fn line_of(text: &str, needle: &str, file: &str) -> usize {
    let hits: Vec<usize> = text
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(needle))
        .map(|(index, _)| index)
        .collect();
    assert_eq!(hits.len(), 1, "{file}: `{needle}` must be on exactly one line");
    hits[0]
}

#[test]
fn the_field_is_written_by_the_zero_fill_the_link_response_and_the_unpack_and_read_by_the_getter_and_the_pack_only() {
    // every access of the member through a pointer or a struct: nothing but the named files touches it
    assert_eq!(
        census(|line| line.contains("->tallest_structure_height") || line.contains(".tallest_structure_height")),
        expected(&[
            ("aphavoc/source/entity/special/sector/sc_float.c", 1),
            ("aphavoc/source/entity/special/sector/sc_msgs.c", 2),
            ("aphavoc/source/entity/special/sector/sc_pack.c", 2),
        ]),
        "a new access of the sector's tallest structure height needs its own characterisation"
    );
    // the declaration is one member of the sector struct, and the creation fills the whole struct with zeros before any attribute
    assert_eq!(
        census(|line| line.trim() == "tallest_structure_height,"),
        expected(&[("aphavoc/source/entity/special/sector/sector.h", 1)])
    );
    let create = source("aphavoc/source/entity/special/sector/sc_seccreat.c");
    once(&create, "memset (raw, 0, sizeof (sector));", "sc_seccreat.c");
    before(
        &create,
        "memset (raw, 0, sizeof (sector));",
        "set_local_entity_attributes (en, pargs);",
        "sc_seccreat.c",
    );
    assert!(!create.contains("tallest"), "the creation names no height: the zero word is the memset");
    // the value is reached through its float type only: the type's users are the getter and the named consumers
    assert_eq!(
        census(|line| line.contains("FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT")),
        expected(&[
            ("aphavoc/source/entity/en_misc/en_maths/en_cllsn.c", 2),
            ("aphavoc/source/entity/mobile/aircraft/fixwing/fw_move.c", 1),
            ("aphavoc/source/entity/special/sector/sc_float.c", 2),
            ("aphavoc/source/entity/special/sector/sc_pack.c", 1),
            ("aphavoc/source/entity/system/en_funcs/en_float.c", 1),
            ("aphavoc/source/entity/system/en_funcs/en_float.h", 1),
        ]),
        "a new reader of FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT needs its own characterisation"
    );
    // the one name the hosted scan reads it through (the header names it once in its contract comment and once in its one read)
    assert_eq!(
        census(|line| line.contains("CAMPAIGN_SCAN_TALLEST")),
        expected(&[
            ("aphavoc/source/entity/en_misc/en_maths/en_cllsn.c", 1),
            ("aphavoc/source/entity/mobile/mb_hostscan.h", 2)
        ])
    );
    // the getter has one case and one overload, and serves the member as stored
    let float = squeeze(&source("aphavoc/source/entity/special/sector/sc_float.c"));
    // Task49 (the host-owned sector height): in a hosted build the getter is the ONE read of the host's word, and the stored member is read only by the original body (the build that keeps the original entity solver)
    let getter = source("aphavoc/source/entity/special/sector/sc_float.c");
    let host_read = line_of(
        &getter,
        "campaign_model_sector_height ((int) raw->x_sector, (int) raw->z_sector, &value);",
        "sc_float.c",
    );
    assert_eq!(
        enclosing(&getter, host_read).last().map(String::as_str),
        Some("defined EECH_CAMPAIGN_HOST_ENTITY")
    );
    let legacy_read = line_of(&getter, "value = raw->tallest_structure_height;", "sc_float.c");
    assert_eq!(
        enclosing(&getter, legacy_read).last().map(String::as_str),
        Some("else of defined EECH_CAMPAIGN_HOST_ENTITY")
    );
    assert!(float.contains("case FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT: { #ifdef EECH_CAMPAIGN_HOST_ENTITY"));
    assert!(float.contains("#endif break; }"));
    assert!(float.contains("fn_get_local_entity_float_value [ENTITY_TYPE_SECTOR][FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT] = get_local_float_value;"));
    assert!(
        !float.contains("set_local_float_value ["),
        "the sector has no float setter: nothing sets the height by the attribute path"
    );
}

#[test]
fn the_legacy_field_is_neither_read_nor_written_in_a_hosted_build_and_the_host_is_asked_at_exactly_the_getter_and_the_unpack() {
    // Task49 (the host-owned sector height): every access of the member through a pointer is compiled only where the original entity solver is kept, so a hosted build neither reads nor writes it (no mirror, no fallback)
    for (file, text) in engine_files() {
        for (index, line) in text.lines().enumerate() {
            if line.contains("->tallest_structure_height") {
                let conditions = enclosing(text, index);
                assert!(
                    conditions
                        .iter()
                        .any(|condition| condition == "!defined EECH_CAMPAIGN_HOST_ENTITY" || condition == "else of defined EECH_CAMPAIGN_HOST_ENTITY"),
                    "{file}:{}: a hosted build would touch the legacy field: {conditions:?}",
                    index + 1
                );
            }
        }
    }
    // the questions the campaign asks the host about a sector's height: the getter asks, and the session's unpack states (the pack and every other reader go through the getter)
    assert_eq!(
        census(|line| line.contains("campaign_model_sector_height (") || line.contains("campaign_model_sector_height_restore (")),
        expected(&[
            ("aphavoc/source/entity/mobile/mb_hostmodel.c", 2),
            ("aphavoc/source/entity/mobile/mb_hostmodel.h", 2),
            ("aphavoc/source/entity/special/sector/sc_float.c", 1),
            ("aphavoc/source/entity/special/sector/sc_pack.c", 1),
        ])
    );
}

#[test]
fn the_raise_is_a_fixed_entity_at_a_sector_list_link_by_position_y_plus_the_box_top_and_nothing_lowers_it() {
    let messages = source("aphavoc/source/entity/special/sector/sc_msgs.c");
    let text = squeeze(&messages);
    // the order of the link response: the list type, the fixed test, the valid shape, the box and the position, the sum, the strict comparison, the store, and only then the mobile bookkeeping
    let steps = [
        "if (list_type == LIST_TYPE_SECTOR)",
        "if (get_local_entity_int_value (sender, INT_TYPE_IDENTIFY_FIXED))",
        "object_3d_index = get_local_entity_int_value (sender, INT_TYPE_OBJECT_3D_SHAPE);",
        "if (object_3d_index != OBJECT_3D_INVALID_OBJECT_INDEX)",
        "bounding_box = get_object_3d_bounding_box (object_3d_index);",
        "position = get_local_entity_vec3d_ptr (sender, VEC3D_TYPE_POSITION);",
        "height = position->y + bounding_box->ymax;",
        "if (height > raw->tallest_structure_height)",
        "raw->tallest_structure_height = height;",
        "add_mobile_values_to_sector (receiver, sender);",
    ];
    for pair in steps.windows(2) {
        before(&text, pair[0], pair[1], "sc_msgs.c");
    }
    for step in steps {
        once(&text, step, "sc_msgs.c");
    }
    // `height` is a float local: the sum is one float expression
    assert!(text.contains("float height; vec3d *position;"));
    // Task49: in a hosted build the raise is the HOST's, at the arrival's state statement (model_host.rs, roster.rs): the whole block, its locals included, is compiled only where the original entity solver is kept
    for statement in [
        "if (get_local_entity_int_value (sender, INT_TYPE_IDENTIFY_FIXED))",
        "height = position->y + bounding_box->ymax;",
        "raw->tallest_structure_height = height;",
    ] {
        let at = line_of(&messages, statement, "sc_msgs.c");
        assert_eq!(enclosing(&messages, at), ["!defined EECH_CAMPAIGN_HOST_ENTITY"], "{statement}");
    }
    // the unlink response is the mobile bookkeeping and nothing else: no height is lowered, recomputed or reset by a removal
    let start = text.find("static int response_to_unlink_child").expect("the unlink response");
    let end = text[start..]
        .find("void overload_sector_message_responses")
        .map(|at| at + start)
        .expect("the end of the unlink response");
    let unlink = &text[start..end];
    assert!(unlink.contains("remove_mobile_values_from_sector (receiver, sender);") && !unlink.contains("tallest") && !unlink.contains("height"));
    // the mobile bookkeeping the link and unlink call never names the height either
    let sector = squeeze(&source("aphavoc/source/entity/special/sector/sector.c"));
    assert!(!sector.contains("tallest"));
    // a fixed entity that changes sector is a removal then an insertion; one that stays in its sector states only its placement
    let fixed = squeeze(&source("aphavoc/source/entity/fixed/fx_vec3d.c"));
    before(
        &fixed,
        "delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR);",
        "raw->position = *v;",
        "fx_vec3d.c",
    );
    before(
        &fixed,
        "raw->position = *v;",
        "insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, parent, NULL);",
        "fx_vec3d.c",
    );
    before(
        &fixed,
        "insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, parent, NULL); } else { raw->position = *v;",
        "campaign_place_fixed_state (en);",
        "fx_vec3d.c",
    );
    // the fixed int getter the test reads is the constant TRUE, and the shape is the stored index
    let int = squeeze(&source("aphavoc/source/entity/fixed/fx_int.c"));
    assert!(int.contains("case INT_TYPE_IDENTIFY_FIXED: { value = TRUE; break; }"));
    assert!(int.contains("case INT_TYPE_OBJECT_3D_SHAPE: { value = raw->object_3d_shape; break; }"));
    // every list insertion into a sector that raises it is one of the fixed creators and unpackers, all through the one funnel (entity_roster_guard asserts the funnel)
    let creators = census(|line| line.contains("insert_local_entity_into_parents_child_list") && line.contains("LIST_TYPE_SECTOR"));
    let fixed_creators: Vec<&str> = creators
        .iter()
        .map(|(file, _)| file.as_str())
        .filter(|file| file.contains("/entity/fixed/"))
        .collect();
    assert_eq!(
        fixed_creators.len(),
        12,
        "the fixed families that list themselves into a sector: {fixed_creators:?}"
    );
}

#[test]
fn the_save_is_the_last_32_raw_bits_of_a_server_session_sector_record_and_the_sectors_are_unpacked_before_any_entity() {
    let pack = source("aphavoc/source/entity/special/sector/sc_pack.c");
    let text = squeeze(&pack);
    // pack: inside the per-sector block of the server session, after the criteria count, nothing after the height
    before(
        &text,
        "if (mode == PACK_MODE_SERVER_SESSION)",
        "pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_COUNT, raw->keysite_count);",
        "sc_pack.c",
    );
    // Task49: the word packed is the HOST's (the getter's read, the same raw 32 bits), and the original line is compiled only where the original body is kept; neither is followed by anything in the record
    let code = squeeze(&pack.lines().map(|line| line.split("//").next().unwrap_or("")).collect::<Vec<_>>().join("\n"));
    before(
        &code,
        "pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_COUNT, raw->keysite_count); #ifdef EECH_CAMPAIGN_HOST_ENTITY pack_float_value (en, FLOAT_TYPE_PROTOTYPE_1, get_local_entity_float_value (en, FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT)); #else pack_float_value (en, FLOAT_TYPE_PROTOTYPE_1, raw->tallest_structure_height); #endif } } }",
        "void unpack_local_sector_data (pack_modes mode)",
        "sc_pack.c",
    );
    // unpack: the mirror, last in the record: in a hosted build the saved word is stated to the host (it holds exactly these bits afterwards) and the legacy field is not written
    assert!(code.contains(
        "raw->keysite_count = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_COUNT); #ifdef EECH_CAMPAIGN_HOST_ENTITY { float saved_word; saved_word = unpack_float_value (en, FLOAT_TYPE_PROTOTYPE_1); campaign_model_sector_height_restore ((int) raw->x_sector, (int) raw->z_sector, &saved_word); } #else raw->tallest_structure_height = unpack_float_value (en, FLOAT_TYPE_PROTOTYPE_1); #endif } } } }"
    ));
    // a browse session packs and unpacks nothing of the sectors, and the per-sector record is the server session's only
    assert_eq!(text.matches("if (mode == PACK_MODE_BROWSE_SESSION) { return; }").count(), 2);
    assert_eq!(text.matches("if (mode == PACK_MODE_SERVER_SESSION)").count(), 2);
    // FLOAT_TYPE_PROTOTYPE_1 is stored unpacked, 32 raw bits, and no other float type shares the sector's record
    let float = squeeze(&source("aphavoc/source/entity/system/en_funcs/en_float.c"));
    assert!(float.contains("\"FLOAT_TYPE_PROTOTYPE_1\", // name FLOAT_PACK_TYPE_UNPACKED, // pack_type 0, // pack_num_whole_bits (excluding sign bit) 0, // pack_num_fractional_bits"));
    assert!(float.contains("case FLOAT_PACK_TYPE_UNPACKED: { pack_unsigned_data (*(int *)&value, 32);"));
    assert!(float.contains("case FLOAT_PACK_TYPE_UNPACKED: { int i; i = unpack_unsigned_data (32); value = *(float *)&i;"));
    assert_eq!(
        census(|line| line.contains("FLOAT_TYPE_PROTOTYPE_1"))
            .iter()
            .filter(|(file, _)| file.contains("/sector/"))
            .count(),
        1,
        "only sc_pack.c uses the prototype float"
    );
    // the session packs the sectors before every entity and unpacks them before every entity: a load restores the height first and the entities' insertions can only raise it
    let session = squeeze(&source("aphavoc/source/entity/system/en_comms/en_sessn.c"));
    before(&session, "pack_local_sector_data (mode);", "pack_local_pylon_data (mode);", "en_sessn.c");
    before(&session, "unpack_local_sector_data (mode);", "unpack_local_pylon_data (mode);", "en_sessn.c");
    before(
        &session,
        "unpack_local_sector_data (mode);",
        "unpack_local_entity_data (NULL, type, mode);",
        "en_sessn.c",
    );
}

#[test]
fn the_consumers_read_it_once_per_served_sector_or_before_the_hosted_movement_and_the_host_already_takes_it_as_a_fact() {
    // the hosted scan reads it once, with the sector the host named, and nothing after
    let scan = source("aphavoc/source/entity/mobile/mb_hostscan.h");
    let first = scan.find("case CAMPAIGN_MODEL_SCAN_FIRST:").expect("the FIRST case");
    let next = scan.find("case CAMPAIGN_MODEL_SCAN_NEXT:").expect("the NEXT case");
    let read = once(&scan, "sector_tallest_structure_height = CAMPAIGN_SCAN_TALLEST (sector);", "mb_hostscan.h");
    assert!(
        first < read && read < next,
        "the height is read where the host names a sector, not on a step through its list"
    );
    assert!(scan.contains("hit = campaign_offer_candidate (entry, FALSE, sector_tallest_structure_height, source_old, source_new, point, normal);"));
    // the offer sends it as fact 12 of the candidate question, unchanged, and the building of a city is offered with the same word
    let offer = source("aphavoc/source/entity/mobile/mb_hostoffer.h");
    once(
        &offer,
        "facts [CAMPAIGN_CANDIDATE_TALLEST] = sector_tallest_structure_height;",
        "mb_hostoffer.h",
    );
    assert!(offer.contains("hit = campaign_offer_candidate (building, TRUE, sector_tallest_structure_height, source_old, source_new, point, normal);"));
    assert!(source("aphavoc/source/entity/mobile/mb_hostmodel.h").contains("#define CAMPAIGN_CANDIDATE_TALLEST 12"));
    // the host's kernel compares it with the segment's lowest point, strictly, for every candidate that is not mobile
    let kernel = source("eech-campaign/crates/eech-world/csrc/entity_candidate_step.c");
    assert!(squeeze(&kernel).contains("if (source_min_position.y > facts[12]) { return CANDIDATE_SKIP; }"));

    // the hosted scan's reader is in the hosted branch and the original scan's reader is only in the build that keeps the original solver (the else of the hosted entity macro)
    let collision = source("aphavoc/source/entity/en_misc/en_maths/en_cllsn.c");
    let define = line_of(
        &collision,
        "#define CAMPAIGN_SCAN_TALLEST(S) get_local_entity_float_value ((S), FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT)",
        "en_cllsn.c",
    );
    assert!(enclosing(&collision, define)
        .iter()
        .any(|condition| condition == "defined EECH_CAMPAIGN_HOST_ENTITY"));
    let original = line_of(
        &collision,
        "if (source_min_position.y > get_local_entity_float_value (sector, FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT))",
        "en_cllsn.c",
    );
    assert!(enclosing(&collision, original)
        .iter()
        .any(|condition| condition == "else of defined EECH_CAMPAIGN_HOST_ENTITY"));
    // and it is a strict comparison of the segment's lowest point against the height, once per sector of the scanned window
    assert!(squeeze(&collision).contains("if (source_min_position.y > get_local_entity_float_value (sector, FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT)) { ignore_all_fixed_entities_in_sector = TRUE; } else { ignore_all_fixed_entities_in_sector = FALSE; }"));

    // the fixed-wing terrain follow reads it for every waypoint step, outside every conditional, and BEFORE the hosted movement block takes the climb
    let movement = source("aphavoc/source/entity/mobile/aircraft/fixwing/fw_move.c");
    let clearance = line_of(
        &movement,
        "min_height = max (min_height, get_local_entity_float_value (sector, FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT) - terrain_elevation);",
        "fw_move.c",
    );
    assert!(
        enclosing(&movement, clearance).is_empty(),
        "the clearance is not conditional: {:?}",
        enclosing(&movement, clearance)
    );
    let hosted = movement
        .lines()
        .enumerate()
        .skip(clearance)
        .find(|(_, line)| line.trim() == "#ifdef EECH_CAMPAIGN_HOST_MOVEMENT")
        .map(|(index, _)| index)
        .expect("the hosted movement block");
    assert!(clearance < hosted);
    let squeezed = squeeze(&movement);
    before(
        &squeezed,
        "min_height = terrain_elevation;",
        "min_height = max (min_height, get_local_entity_float_value (sector, FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT) - terrain_elevation);",
        "fw_move.c",
    );
    before(
        &squeezed,
        "get_local_entity_float_value (sector, FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT) - terrain_elevation);",
        "min_height = max (get_local_entity_float_value (wp, FLOAT_TYPE_ALTITUDE), min_height);",
        "fw_move.c",
    );
    before(
        &squeezed,
        "min_height = max (get_local_entity_float_value (wp, FLOAT_TYPE_ALTITUDE), min_height);",
        "terrain->min_height = min_height;",
        "fw_move.c",
    );
}

#[test]
fn every_file_that_writes_or_reads_the_height_is_in_the_engine_source_list() {
    let list = source("eech-campaign/crates/eech-engine-sys/sources.txt").replace('\\', "/");
    for file in [
        "aphavoc/source/entity/special/sector/sc_msgs.c",
        "aphavoc/source/entity/special/sector/sc_float.c",
        "aphavoc/source/entity/special/sector/sc_pack.c",
        "aphavoc/source/entity/special/sector/sc_seccreat.c",
        "aphavoc/source/entity/en_misc/en_maths/en_cllsn.c",
        "aphavoc/source/entity/mobile/aircraft/fixwing/fw_move.c",
        "aphavoc/source/entity/fixed/fx_vec3d.c",
        "aphavoc/source/entity/system/en_comms/en_sessn.c",
    ] {
        assert!(list.lines().any(|line| line.trim() == file), "{file} is not in the engine source list");
    }
    // the header the hosted scan is bound from is included by en_cllsn.c (a source-list file)
    assert!(source("aphavoc/source/entity/en_misc/en_maths/en_cllsn.c").contains("#include \"entity/mobile/mb_hostscan.h\""));
}

#[test]
fn the_box_top_is_the_hosts_in_every_profile_but_the_original_model_fixture_and_nine_fixed_families_can_raise_the_height() {
    // the accessor the link response calls: the tree's own text reads the campaign's scene entry, and the shipping profiles stage a replacement that serves the host-authored table
    let accessor = source("modules/3d/3dobjvis.c");
    once(&accessor, "return ( &objects_3d_scene_database[object].bounding_box );", "3dobjvis.c");
    let patches = squeeze(&source("eech-campaign/crates/eech-engine-sys/build/patches.rs"));
    assert!(patches.contains(r#"id: "D3a-model-bounding-box-from-host""#));
    assert!(patches.contains(r#"original: "\treturn ( &objects_3d_scene_database[object].bounding_box );\n","#));
    assert!(patches.contains("return ( campaign_model_bounding_box ( (int) object ) );"));
    let build = squeeze(&source("eech-campaign/crates/eech-engine-sys/build/main.rs"));
    // the selection itself, not a comment: the model patches (D3a among them) are applied to the staged tree in every build except the one with the model-fixture feature
    assert!(patches.contains("pub const MODEL_HOST_PATCHES: &[Patch] = &["));
    assert!(
        patches.contains("pub fn model_host_files() -> Vec<&'static str> { let mut files: Vec<&str> = MODEL_HOST_PATCHES.iter().map(|p| p.file).collect();")
    );
    assert!(build.contains(r#"let model_fixture = env::var_os("CARGO_FEATURE_MODEL_FIXTURE").is_some();"#));
    assert!(build.contains("if !model_fixture { for file in patches::model_host_files() {"));
    assert!(build.contains("patches::apply_model_host(file, &text)"));
    // the host's table is written once at boot from a provider that is refused a bound that is not finite, and a read before that install stops the engine
    let model = squeeze(&source("aphavoc/source/entity/mobile/mb_hostmodel.c"));
    assert!(model.contains("!all_finite (reply.bounding_box, 6)"));
    before(
        &model,
        "bounds_from (&host_bounding_box[scene], reply.bounding_box);",
        "tables_installed = TRUE;",
        "mb_hostmodel.c",
    );
    assert!(model.contains("return &host_bounding_box[scene];"));
    assert!(model.contains("host model: [model-provider:not.installed] %s was asked before the host's scenes were installed"));

    // the families that carry IDENTIFY_FIXED: nine, each by its own overload call; the host's fixed collision state holds the eight that are not the city, with the position and the shape
    assert_eq!(
        census(|line| line.contains("overload_fixed_functions (ENTITY_TYPE_")),
        expected(&[
            ("aphavoc/source/entity/fixed/cbuild/cb_funcs.c", 1),
            ("aphavoc/source/entity/fixed/city/ct_funcs.c", 1),
            ("aphavoc/source/entity/fixed/crater/cr_funcs.c", 1),
            ("aphavoc/source/entity/fixed/object/ob_funcs.c", 1),
            ("aphavoc/source/entity/fixed/pylon/py_funcs.c", 1),
            ("aphavoc/source/entity/fixed/scenic/sn_funcs.c", 1),
            ("aphavoc/source/entity/fixed/segment/sg_funcs.c", 1),
            ("aphavoc/source/entity/fixed/site/st_funcs.c", 1),
            ("aphavoc/source/entity/fixed/siteupdt/su_funcs.c", 1),
        ])
    );
    let place = squeeze(&source("aphavoc/source/entity/mobile/mb_hostplace.c"));
    assert!(place.contains(
        "case ENTITY_TYPE_CITY_BUILDING: case ENTITY_TYPE_CRATER: case ENTITY_TYPE_OBJECT: case ENTITY_TYPE_PYLON: case ENTITY_TYPE_SCENIC: case ENTITY_TYPE_SEGMENT: case ENTITY_TYPE_SITE: case ENTITY_TYPE_SITE_UPDATABLE: { return TRUE; }"
    ));
    assert!(place.contains("#define CAMPAIGN_PLACE_CITY(TYPE) (((entity_types) (TYPE)) == ENTITY_TYPE_CITY)"));
    assert!(place.contains("state [4] = (float) get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);"));
    // Task49: the city's state states its SHAPE (fact 9: the one fact the host needs of a city to decide the raise at the city's arrival). Its creation sets the invalid shape with the default resolution commented out, so a city raises
    // the height only if some creator or restore gives it a valid one: whether a real city ever is, is NOT known from source (the accepted baseline reached 14395 city links, all invalid), and the host decides it from the stated word, never from a blanket absence
    let city_read = place
        .find("static void place_city_read (struct ENTITY *en, float *state) {")
        .expect("the city state reader");
    let city_read_end = place[city_read..]
        .find("state [10] = (float) get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE); }")
        .expect("the end of the city state reader");
    assert!(
        place[city_read..city_read + city_read_end].contains("state [9] = (float) get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);"),
        "the shape word follows the sub type word"
    );
    let city = squeeze(&source("aphavoc/source/entity/fixed/city/ct_creat.c"));
    assert!(city.contains("raw->fix.object_3d_shape = OBJECT_3D_INVALID_OBJECT_INDEX;"));
    assert!(city.contains("/* if (raw->fix.object_3d_shape == OBJECT_3D_INVALID_OBJECT_INDEX) { raw->fix.object_3d_shape = fixed_database[raw->fix.sub_type].default_3d_shape; } */"));
}
