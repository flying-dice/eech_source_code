//! Compile guard of the source facts the host's sector roster rests on (EEDCS-49, D3, sixth slice). The roster is correct only if EVERY write to a sector list is reported to it, in the order the list
//! was written, and if its answer is never completed from the campaign's own list. These facts are read from the compiled source and asserted as text so that a change that breaks one fails before any
//! native run. They are source evidence of the supported single-player runtime and of the unpack routes that exist in source; they are not a proof of behaviour (the isolated C dialogue
//! `entity_dialogue`, the host's `entity_roster_host` tests and the native runs are), and the raw legacy list-link unpack (`unpack_list_link`) is shown not to carry the sector list:
//!
//!  1. every write to a sector list is `insert_local_entity_into_parents_child_list` or `delete_local_entity_from_parents_child_list` (en_list.c), whoever calls them: no creator, relink, unpack, destroyer or
//!     message handler sets a sector list's first child, successor, predecessor or parent by any other route, and the raw list-link packers never name the sector list;
//!  2. every insertion into a sector list is at the head (a NULL predecessor), which is what makes the tail the oldest insertion and the scan's order the host's insertion order;
//!  3. the funnel reports the insertion after the list is written and before the link notifications, and the removal while the entity is still in the list and before the notifications;
//!  4. a heap slot's release removes an entity that is still placed before the roster releases it, and a boot ends the keys before the entity system is reinitialised;
//!  5. the hosted scan takes its entry from the host's key only: the campaign's list is the mirror and every disagreement is a named refusal;
//!  6. a hosted mobile's record is made current BEFORE the insertion of a sector change, in both setters that relink a hosted mobile, and the placement is still made exactly once.

use std::path::{Path, PathBuf};

fn source(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../aphavoc/source").join(relative);
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

fn before(text: &str, first: &str, second: &str, file: &str) {
    let a = text.find(first).unwrap_or_else(|| panic!("{file}: `{first}` is not there"));
    let b = text.find(second).unwrap_or_else(|| panic!("{file}: `{second}` is not there"));
    assert!(a < b, "{file}: `{first}` must come before `{second}`");
}

/// whether a line names the sector list `LIST_TYPE_SECTOR` itself and not a longer name that starts with it
fn names_the_sector_list(line: &str) -> bool {
    let name = "LIST_TYPE_SECTOR";
    let mut from = 0;
    while let Some(at) = line[from..].find(name) {
        let end = from + at + name.len();
        if !line[end..].chars().next().is_some_and(|c| c.is_ascii_alphanumeric() || c == '_') {
            return true;
        }
        from = end;
    }
    false
}

/// the code of a line: a `//` comment is not code
fn code(line: &str) -> &str {
    line.split("//").next().unwrap_or("")
}

#[test]
fn every_write_to_a_sector_list_goes_through_the_funnel_and_every_insertion_is_at_the_head() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../aphavoc/source");
    let mut files = Vec::new();
    walk(&root, &mut files);
    let mut inserts = 0;
    let mut deletes = 0;
    let mut tables = 0;
    let mut creation_attributes: Vec<String> = Vec::new();
    let mut unlinks: Vec<String> = Vec::new();
    let mut offenders = Vec::new();
    for path in files {
        let name = path.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
        if name == "entity/system/en_funcs/en_list.c" {
            continue;
        }
        let text = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).replace("\r\n", "\n");
        let mut in_block_comment = false;
        for (number, raw) in text.lines().enumerate() {
            // block comments are not code (the legacy unpack paths keep a disabled copy of an insertion)
            let mut line = code(raw).to_string();
            if in_block_comment {
                if let Some(end) = line.find("*/") {
                    line = line[end + 2..].to_string();
                    in_block_comment = false;
                } else {
                    continue;
                }
            }
            while let Some(start) = line.find("/*") {
                match line[start..].find("*/") {
                    Some(end) => line.replace_range(start..start + end + 2, ""),
                    None => {
                        line.truncate(start);
                        in_block_comment = true;
                    }
                }
            }
            // the sector list itself: LIST_TYPE_SECTOR_ROOT, _LINK, _TASK and _FRONTLINE are other lists and other names
            if !names_the_sector_list(&line) {
                continue;
            }
            let at = format!("{name}:{}: {}", number + 1, raw.trim());
            if line.contains("insert_local_entity_into_parents_child_list") {
                inserts += 1;
                // the head: the predecessor is NULL
                if !line
                    .trim_end()
                    .trim_end_matches(';')
                    .trim_end()
                    .trim_end_matches(')')
                    .trim_end()
                    .ends_with(", NULL")
                {
                    offenders.push(format!("an insertion that is not at the head: {at}"));
                }
            } else if line.contains("delete_local_entity_from_parents_child_list") {
                deletes += 1;
            } else if name == "entity/system/en_funcs/en_list.h" && line.trim() == "LIST_TYPE_SECTOR," {
                // the list type's own enumerator
            } else if name.starts_with("entity/system/en_funcs/en_list/") {
                // the dispatch tables of the list accessors: a definition of which function a family's sector list uses, not a write
                tables += 1;
            } else if line.contains("ENTITY_ATTR_PARENT") {
                // a creation attribute: the new entity's parent field is set before the creator's own insertion (checked below), which is the write the roster is told about
                creation_attributes.push(name.clone());
            } else if line.contains("unlink_local_entity_children") {
                // the destruction of a sector unlinks every child through the funnel's removal
                unlinks.push(name.clone());
            } else if line.contains("list_type == LIST_TYPE_SECTOR") {
                // the sector's own message handlers: a read of which list a link message is about
            } else if line.contains("get_local_entity_first_child")
                || line.contains("get_local_entity_child_succ")
                || line.contains("get_local_entity_child_pred")
                || line.contains("get_local_entity_parent")
                || line.contains("remove_mobile_values_from_sector")
            {
                // a read
            } else {
                offenders.push(format!("another use of the sector list: {at}"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "the sector list is written outside the funnel, or not at the head:\n{}",
        offenders.join("\n")
    );
    // the writers that exist in source (creators, unpack, relinks, destroyers): a change that drops them all must be seen, not passed
    // the one creation that names the sector list as an attribute is a city, whose creator then inserts it through the funnel
    assert_eq!(
        creation_attributes,
        vec!["ai/faction/popread.c".to_string()],
        "another creation names the sector list as an attribute"
    );
    assert!(source("entity/fixed/city/ct_creat.c").contains("insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR,"));
    assert_eq!(unlinks, vec!["entity/special/sector/sc_dstry.c".to_string()]);
    assert!(tables > 0);
    assert!(
        inserts >= 35 && deletes >= 25,
        "only {inserts} insertions and {deletes} deletions of a sector list were found"
    );
}

#[test]
fn the_funnel_reports_an_insertion_after_the_list_is_written_and_a_removal_while_the_entity_is_in_it() {
    let list = source("entity/system/en_funcs/en_list.c");
    let insert = &list[list.find("void insert_local_entity_into_parents_child_list").expect("the insertion")..];
    let insert = &insert[..insert.find("\n}\n").expect("its end")];
    before(
        insert,
        "set_local_entity_first_child (parent, type, en);",
        "campaign_place_insert (en);",
        "en_list.c insert",
    );
    before(
        insert,
        "campaign_place_insert (en);",
        "notify_local_entity (ENTITY_MESSAGE_LINK_CHILD",
        "en_list.c insert",
    );
    assert!(insert.contains("if (type == LIST_TYPE_SECTOR)"), "the report is for a sector list only");
    let delete = &list[list.find("void delete_local_entity_from_parents_child_list").expect("the removal")..];
    let delete = &delete[..delete.find("\n}\n").expect("its end")];
    before(
        delete,
        "parent = get_local_entity_parent (en, type);",
        "campaign_place_remove (en);",
        "en_list.c delete",
    );
    before(
        delete,
        "campaign_place_remove (en);",
        "notify_local_entity (ENTITY_MESSAGE_UNLINK_CHILD",
        "en_list.c delete",
    );
    before(
        delete,
        "campaign_place_remove (en);",
        "set_local_entity_parent (en, type, NULL);",
        "en_list.c delete",
    );
    assert!(delete.contains("if (type == LIST_TYPE_SECTOR)"), "the report is for a sector list only");
}

#[test]
fn a_released_slot_ends_its_placement_before_the_roster_and_a_boot_ends_the_keys_before_the_entities() {
    let heap = source("entity/system/en_main/en_heap.c");
    let release = &heap[heap.find("void set_free_entity").expect("set_free_entity")..];
    before(
        release,
        "campaign_place_release_entity (en);",
        "campaign_roster_release_entity (en);",
        "en_heap.c",
    );
    before(release, "campaign_place_release_entity (en);", "en->type = ENTITY_TYPE_UNKNOWN;", "en_heap.c");
    let engine_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("csrc/eech_engine.c");
    let engine = String::from_utf8_lossy(&std::fs::read(&engine_path).unwrap()).replace("\r\n", "\n");
    before(&engine, "campaign_place_reset ();", "reinitialise_entity_system ();", "eech_engine.c");
    let sources = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sources.txt")).unwrap();
    assert!(
        sources.contains("aphavoc/source/entity/mobile/mb_hostplace.c"),
        "the placement table is not in the headless build"
    );
}

#[test]
fn the_hosted_scan_takes_its_entry_from_the_hosts_key_and_the_campaigns_list_is_only_the_mirror() {
    let scan = source("entity/mobile/mb_hostscan.h");
    let function = &scan[scan.find("campaign_scan_entities\n").expect("the driver")..];
    // the entry is only ever assigned from the served key; the campaign's list getters feed the mirror
    for line in function.lines() {
        let line = code(line).trim();
        if line.starts_with("entry =") && line != "entry = NULL;" {
            assert!(line.contains("campaign_scan_served"), "the entry is not the host's: {line}");
        }
    }
    assert!(function.contains("mirror = CAMPAIGN_SCAN_FIRST (sector);") && function.contains("mirror = CAMPAIGN_SCAN_PRED (mirror);"));
    let served = &scan[scan.find("static CAMPAIGN_SCAN_ENTITY *campaign_scan_served").expect("the check")..];
    let served = &served[..served.find("\n}\n").expect("its end")];
    assert!(
        served.contains("entity_scan.roster_mirror") && served.contains("entity_scan.roster_key"),
        "a disagreement or an unknown key is a named refusal"
    );
    assert!(!served.contains("return (mirror)"), "the mirror is never the answer");
    // the real binding resolves a key through the placement table and nothing else
    let hosted = source("entity/en_misc/en_maths/en_cllsn.c");
    assert!(hosted.contains("#define CAMPAIGN_SCAN_ENTITY_OF(KEY) campaign_place_entity ((KEY))"));
}

#[test]
fn a_hosted_mobiles_record_is_made_current_before_the_insertion_of_a_sector_change_and_it_is_placed_exactly_once() {
    for file in ["entity/mobile/vehicle/vh_vec3d.c", "entity/mobile/aircraft/ac_vec3d.c"] {
        let text = source(file);
        let setter = &text[text.find("static void set_local_vec3d").expect("the setter")..];
        let setter = &setter[..setter.find("\n}\n").expect("its end")];
        let position = &setter[setter.find("case VEC3D_TYPE_POSITION:").expect("the position case")..];
        before(
            position,
            "campaign_movement_place (en);",
            "insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR",
            file,
        );
        before(
            position,
            "placed = TRUE;",
            "insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR",
            file,
        );
        // the placement after the notifications is made only when none was made before the insertion
        assert!(
            position.contains("if (!placed)"),
            "{file}: the placement after the sector change must be skipped when it was made before"
        );
        assert_eq!(
            position.matches("campaign_movement_place (en)").count(),
            2,
            "{file}: exactly one placement on each branch"
        );
    }
}

/// whether a line names the city building list `LIST_TYPE_CITY_BUILDING` itself and not a longer name that starts with it (`_LINK`, `_ROOT`)
fn names_the_city_list(line: &str) -> bool {
    let name = "LIST_TYPE_CITY_BUILDING";
    let mut from = 0;
    while let Some(at) = line[from..].find(name) {
        let end = from + at + name.len();
        if !line[end..].chars().next().is_some_and(|c| c.is_ascii_alphanumeric() || c == '_') {
            return true;
        }
        from = end;
    }
    false
}

/// the code lines of a source file that name the city building list: block and line comments are not code
fn city_list_lines(text: &str) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    let mut in_block_comment = false;
    for (number, raw) in text.lines().enumerate() {
        let mut line = code(raw).to_string();
        if in_block_comment {
            if let Some(end) = line.find("*/") {
                line = line[end + 2..].to_string();
                in_block_comment = false;
            } else {
                continue;
            }
        }
        while let Some(start) = line.find("/*") {
            match line[start..].find("*/") {
                Some(end) => line.replace_range(start..start + end + 2, ""),
                None => {
                    line.truncate(start);
                    in_block_comment = true;
                }
            }
        }
        if names_the_city_list(&line) {
            found.push((number + 1, line.trim().to_string()));
        }
    }
    found
}

/// The city building list (`LIST_TYPE_CITY_BUILDING`, seventh slice): the host holds a city's buildings and their order, so every write to the list must be reported, a building's place in it must be the host's one
/// record with its sector placement, and the scan's city traversal must be served from the host's list with the campaign's own list as the mirror. Read from the compiled source:
///
///  1. every write to the list is the funnel (en_list.c) from the one creator (`cb_creat.c`, which inserts after the predecessor the creation names: none, for every creation that exists) and the one destroyer
///     (`cb_dstry.c`, and the city's destruction of its children through it); no other file sets the list's links, and no creation names a predecessor for it;
///  2. the funnel reports the insertion after the list is written and before the link notifications, and the removal while the building is still in the list;
///  3. the list's other occurrences are creation attributes (the city is the building's parent, created and placed before it), reads, the message handler and the dispatch tables;
///  4. a building is listed before it is placed in its sector and unlisted after it leaves it, and a release removes a still-listed building before its sector placement;
///  5. the hosted scan's city traversal asks the host for the first and the next building, and the original body keeps the campaign's own list;
///  6. the object-report identity is still issued lazily (the same request, the same serial) and the host is told the identity afterwards.
#[test]
fn every_write_to_the_city_building_list_goes_through_the_funnel_and_a_scan_reads_the_hosts_list() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../aphavoc/source");
    let mut files = Vec::new();
    walk(&root, &mut files);
    let mut inserts: Vec<String> = Vec::new();
    let mut deletes: Vec<String> = Vec::new();
    let mut destroys: Vec<String> = Vec::new();
    let mut attributes: Vec<String> = Vec::new();
    let mut offenders = Vec::new();
    for path in files {
        let name = path.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
        if name == "entity/system/en_funcs/en_list.c" {
            continue;
        }
        let text = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).replace("\r\n", "\n");
        for (number, line) in city_list_lines(&text) {
            let at = format!("{name}:{number}: {line}");
            if line.contains("insert_local_entity_into_parents_child_list") {
                inserts.push(name.clone());
                if name != "entity/fixed/cbuild/cb_creat.c" || !line.contains("raw->city_building_link.parent, raw->city_building_link.child_pred") {
                    offenders.push(format!("an insertion outside the one creator: {at}"));
                }
            } else if line.contains("delete_local_entity_from_parents_child_list") {
                deletes.push(name.clone());
            } else if line.contains("destroy_local_entity_children") || line.contains("destroy_client_server_entity_children") {
                destroys.push(name.clone());
            } else if line.contains("ENTITY_ATTR_PARENT") {
                attributes.push(name.clone());
            } else if line.contains("ENTITY_ATTR_CHILD_PRED") {
                offenders.push(format!("a creation that names a predecessor in the city list: {at}"));
            } else if name == "entity/system/en_funcs/en_list.h" && line == "LIST_TYPE_CITY_BUILDING," {
                // the list type's own enumerator
            } else if name.starts_with("entity/system/en_funcs/en_list/") {
                // the dispatch tables of the list accessors
            } else if line.starts_with("case LIST_TYPE_CITY_BUILDING:") && name == "entity/fixed/city/ct_msgs.c" {
                // the city's message handler: which list a link message is about
            } else if line.contains("get_local_entity_first_child")
                || line.contains("get_local_entity_child_succ")
                || line.contains("get_local_entity_child_pred")
                || line.contains("get_local_entity_parent")
            {
                // a read (the offer's mirror, the drawing of a city's buildings)
            } else {
                offenders.push(format!("another use of the city building list: {at}"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "the city building list is written outside the funnel:\n{}",
        offenders.join("\n")
    );
    assert_eq!(inserts, vec!["entity/fixed/cbuild/cb_creat.c".to_string()]);
    assert_eq!(deletes, vec!["entity/fixed/cbuild/cb_dstry.c".to_string()]);
    assert_eq!(destroys.len(), 2, "the city destroys its buildings through their destroyer: {destroys:?}");
    assert!(destroys.iter().all(|file| file == "entity/fixed/city/ct_dstry.c"));
    // the only creations name the city as the building's parent and never a predecessor, so a building is always inserted at the head
    assert_eq!(attributes, vec!["ai/faction/popread.c".to_string(); 2], "{attributes:?}");

    // 2. the funnel reports after the write and before the notifications; the removal while the building is in the list
    let list = source("entity/system/en_funcs/en_list.c");
    let insert = &list[list.find("void insert_local_entity_into_parents_child_list").unwrap()..list.find("void clear_local_entity_list").unwrap()];
    before(
        insert,
        "set_local_entity_first_child (parent, type, en);",
        "campaign_place_child_insert (en, parent, pred);",
        "en_list.c insertion",
    );
    before(
        insert,
        "campaign_place_child_insert (en, parent, pred);",
        "notify_local_entity (ENTITY_MESSAGE_LINK_CHILD, parent, en, type);",
        "en_list.c insertion",
    );
    assert!(insert.contains("else if (type == LIST_TYPE_CITY_BUILDING)"));
    let delete = &list[list.find("void delete_local_entity_from_parents_child_list").unwrap()..];
    before(
        delete,
        "campaign_place_child_remove (en);",
        "notify_local_entity (ENTITY_MESSAGE_UNLINK_CHILD, parent, en, type);",
        "en_list.c removal",
    );
    before(
        delete,
        "campaign_place_child_remove (en);",
        "set_local_entity_parent (en, type, NULL);",
        "en_list.c removal",
    );

    // 4. listed before placed, unlisted after it leaves; the city is placed when it is created, before any building
    let created = source("entity/fixed/cbuild/cb_creat.c");
    before(
        &created,
        "insert_local_entity_into_parents_child_list (en, LIST_TYPE_CITY_BUILDING,",
        "insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR,",
        "cb_creat.c",
    );
    assert!(created.contains("ASSERT (raw->city_building_link.parent);"));
    assert!(source("entity/fixed/city/ct_creat.c").contains("insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR,"));
    let destroyed = source("entity/fixed/cbuild/cb_dstry.c");
    before(
        &destroyed,
        "delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR);",
        "delete_local_entity_from_parents_child_list (en, LIST_TYPE_CITY_BUILDING);",
        "cb_dstry.c",
    );
    let map = source("entity/mobile/mb_hostplacemap.h");
    let release = &map[map.find("static void place_release_entity").unwrap()..map.find("static void place_reset").unwrap()];
    before(release, "place_child_remove (en);", "place_remove (en);", "mb_hostplacemap.h release");

    // 5. the hosted scan's city traversal is the host's; the original body (kept for the characterisation profile) reads the campaign's list
    let collision = source("entity/en_misc/en_maths/en_cllsn.c");
    assert!(collision
        .contains("#define CAMPAIGN_OFFER_FIRST_BUILDING(E) campaign_place_child_first ((E), get_local_entity_first_child ((E), LIST_TYPE_CITY_BUILDING))"));
    assert!(collision.contains(
        "#define CAMPAIGN_OFFER_NEXT_BUILDING(C,B) campaign_place_child_next ((C), (B), get_local_entity_child_succ ((B), LIST_TYPE_CITY_BUILDING))"
    ));
    assert!(
        collision.contains("building = get_local_entity_first_child (target, LIST_TYPE_CITY_BUILDING);"),
        "the original body is kept"
    );
    let offer = source("entity/mobile/mb_hostoffer.h");
    assert!(offer.contains("building = CAMPAIGN_OFFER_NEXT_BUILDING (target, building);"));
    assert!(
        !offer.contains("get_local_entity_child_succ") && !offer.contains("get_local_entity_first_child"),
        "the offer reads no list of the campaign itself"
    );

    // 6. the identity is issued exactly as before (the same request, the same serial) and the host is told it afterwards
    let object = source("entity/mobile/mb_hostobj.c");
    assert!(object.contains("request.serial = issued_count + 1;"));
    before(
        &object,
        "campaign_aim_identify_object (&request, issued);",
        "campaign_place_name (en, records[record_count - 1].host_id);",
        "mb_hostobj.c",
    );
    before(
        &object,
        "issued_count++;",
        "campaign_place_name (en, records[record_count - 1].host_id);",
        "mb_hostobj.c",
    );
    assert_eq!(
        object.matches("campaign_place_name").count(),
        1,
        "the host is told the identity only where it is issued"
    );
}
