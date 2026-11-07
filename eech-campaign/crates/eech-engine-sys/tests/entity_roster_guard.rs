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

/// The city's sector placement is TRANSIENT while its buildings stay listed, and the wire domains of the city events are derived from the domains of their parts (seventh slice, corrections). Read from the compiled source:
///
///  1. a city has no position setter of its own: it registers the SHARED fixed-position setter, which removes the placement from its sector list and inserts it again whenever the position crosses a sector, never
///     touching the city's building list, so a list the host keeps must outlive the placement and be continued by the next one;
///  2. every question about a city's list names it by the stable name (`place_list_key`), never by the sector key alone; the unlisting of a building names the list it recorded at the insertion; a relinked city
///     placement is declared to continue the list; the object identity is named once per record (the `named` flag), not once per placement;
///  3. the request's identity field, and every buffer a city event is built or checked in, is sized from the key's domain and the object identity's own domain (never the host identity's, which the join would
///     otherwise squeeze), and the object identity's domain is the registry's (a compile-time assertion).
#[test]
fn a_citys_sector_placement_is_relinked_by_the_shared_fixed_setter_while_its_buildings_stay_listed_and_the_wire_domains_are_derived() {
    // 1. the shared setter
    assert!(source("entity/fixed/city/ct_funcs.c").contains("overload_fixed_functions (ENTITY_TYPE_CITY);"));
    assert!(source("entity/fixed/fx_funcs.c").contains("overload_fixed_vec3d_functions (type);"));
    let city_vec3d = source("entity/fixed/city/ct_vec3d.c");
    let overload = &city_vec3d[city_vec3d.find("void overload_city_vec3d_functions").unwrap()..];
    assert!(
        !overload.contains("fn_set_local_entity_vec3d"),
        "the city registers no setter of its own for any vec3d"
    );
    let fixed = source("entity/fixed/fx_vec3d.c");
    assert!(
        fixed.lines().any(|line| line.trim_start().starts_with("fn_set_local_entity_vec3d")
            && line.contains("[VEC3D_TYPE_POSITION]")
            && line.contains("= set_local_vec3d;")),
        "the fixed entities' position setter is the shared one"
    );
    let setter = &fixed[fixed.find("static void set_local_vec3d").unwrap()..fixed.find("static void set_remote_vec3d").unwrap()];
    before(
        setter,
        "delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR);",
        "insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, parent, NULL);",
        "fx_vec3d.c setter",
    );
    assert!(!setter.contains("LIST_TYPE_CITY_BUILDING"), "the setter does not touch a city's building list");

    // 2. the stable name of a city's list
    let map = source("entity/mobile/mb_hostplacemap.h");
    assert!(map.contains("static const char *place_list_key (int slot)"));
    assert!(map.contains("CAMPAIGN_SCAN_PHASE_CHILD_FIRST, CAMPAIGN_PLACE_TYPE (parent), place_list_key (slot), key)"));
    assert!(map.contains("place_pair (identity, sizeof (identity), place_list_key (slot), place_child_of_slot [child_slot]);"));
    assert!(map.contains("place_pair (identity, sizeof (identity), place_parent_of_slot [slot], place_child_of_slot [slot]);"));
    assert!(map.contains("list = place_list_key (parent_slot);"));
    let declare = &map[map.find("static void place_declare").unwrap()..map.find("// `en` was inserted at the head of a sector's list").unwrap()];
    assert!(declare.contains("place_pair (identity, sizeof (identity), place_key_of_slot [slot], place_list_of_slot [slot]);"));
    assert!(declare.contains("place_named_of_slot [slot] = 1;") && declare.contains("place_named_of_slot [slot] ||"));
    // the anchor is the host record's name: kept when the last building leaves a placed city, dropped only when the record ends (no placement, no building, and no box the host keeps the record for: the host
    // names such a record in the answer of the removal, and the campaign then keeps the name)
    let child_remove = &map[map.find("static void place_child_remove").unwrap()..map.find("static void place_release_entity").unwrap()];
    before(
        child_remove,
        "if (!place_key_of_slot [city][0] && !answered [0])",
        "place_list_of_slot [city][0] = 0;",
        "mb_hostplacemap.h place_child_remove",
    );
    let remove = &map[map.find("static void place_remove").unwrap()..map.find("static void place_child_insert").unwrap()];
    before(
        remove,
        "if (place_listed_of_slot [slot] <= 0)",
        "place_list_of_slot [slot][0] = 0;",
        "mb_hostplacemap.h place_remove",
    );
    let release = &map[map.find("static void place_release_entity").unwrap()..map.find("static void place_reset").unwrap()];
    assert!(release.contains("place_list_of_slot [index][0] = 0;") && release.contains("place_named_of_slot [index] = 0;"));

    // 3. the wire domains
    let model = source("entity/mobile/mb_hostmodel.h");
    assert!(model.contains("#define CAMPAIGN_MODEL_SCAN_IDENTITY_MAX (CAMPAIGN_ROSTER_KEY_MAX + 1 + CAMPAIGN_MODEL_OBJECT_ID_MAX)"));
    assert!(model.contains("char host_id[CAMPAIGN_MODEL_SCAN_IDENTITY_MAX + 1];"));
    assert!(source("entity/mobile/mb_hostobj.h").contains("#define CAMPAIGN_OBJECT_ID_MAX 128"));
    assert!(source("entity/mobile/mb_hostplace.c").contains("(CAMPAIGN_MODEL_OBJECT_ID_MAX == CAMPAIGN_OBJECT_ID_MAX)"));
    assert!(
        !map.contains("identity [CAMPAIGN_MODEL_HOST_ID_MAX + 1]"),
        "no city buffer is sized by the host identity's domain"
    );
    let entity = source("entity/mobile/mb_hostentity.h");
    let check = &entity[entity.find("static void city_identity_check").unwrap()..entity.find("static void check_city_reply").unwrap()];
    assert!(
        !check.contains("CAMPAIGN_MODEL_HOST_ID_MAX"),
        "the city event's checks are not bounded by the host identity's domain"
    );
    assert!(check.contains("CAMPAIGN_MODEL_SCAN_IDENTITY_MAX") && check.contains("CAMPAIGN_MODEL_OBJECT_ID_MAX"));
    let sys = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")).unwrap();
    assert!(sys.contains("pub host_id: [c_char; EECH_MODEL_SCAN_IDENTITY_MAX + 1],"));
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// FIXED/STATIC COLLISION BACKING (Task49): the host's record of a placement holds a fixed entity's collision state, so EVERY writer of a field of that state must state it. These are source facts of the
// supported single-player runtime and of the unpack routes that exist in source, asserted as text so that a new raw writer, a new fixed type or a creator that inserts before the state is complete fails here
// before any native run. They are not a proof of behaviour (the isolated dialogue, the host's provider tests and the native control are).

/// the text of the function that holds line `at` of `text`: from the line after the previous column-0 `}` to the next column-0 `}`
fn function_around(text: &str, at: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut start = at;
    while start > 0 && !lines[start - 1].starts_with('}') {
        start -= 1;
    }
    let mut end = at;
    while end + 1 < lines.len() && !lines[end].starts_with('}') {
        end += 1;
    }
    lines[start..=end].join("\n")
}

/// the fixed-family files (the entity types that carry the fixed record): every `.c` file under entity/fixed with its path relative to aphavoc/source
fn fixed_files() -> Vec<(String, String)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../aphavoc/source");
    let mut files = Vec::new();
    walk(&root.join("entity/fixed"), &mut files);
    let mut found: Vec<(String, String)> = files
        .into_iter()
        .filter(|path| path.extension().is_some_and(|extension| extension == "c"))
        .map(|path| {
            let relative = path.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
            let text = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).replace("\r\n", "\n");
            (relative, text)
        })
        .collect();
    found.sort();
    found
}

#[test]
fn every_writer_of_a_field_of_a_fixed_entitys_collision_state_states_it_to_the_hosts_record() {
    let mut writers = 0;
    for (file, text) in fixed_files() {
        // a creator writes the state of an entity that is not placed yet (the insertion states it, which the next test pins); every other writer is a writer of a PLACED entity
        // or an unpack that is followed by the insertion
        let creator = file.ends_with("_creat.c");
        // a city is not of the fixed family the host holds a fixed-shape state for: its own position and box have their own census (the two tests below the family's)
        if file.contains("/city/") {
            continue;
        }
        for (at, line) in text.lines().enumerate() {
            let body = code(line);
            let shape = body.contains("object_3d_shape = ") && !body.contains("approximation_object_3d_shape") && !body.contains("object_3d_shape == ");
            let sub_type = (body.contains("->sub_type = ") || body.contains("fix.sub_type = ")) && !body.contains("== ");
            let heading = (body.contains("->heading = ") || body.contains("fix.heading = ")) && !body.contains("== ");
            let height = body.contains("fix.position.y = ") || body.contains("pos->y = ");
            let position = body.contains("raw->position = ") || body.contains("&raw->position");
            if !(shape || sub_type || heading || height || position) || creator {
                continue;
            }
            writers += 1;
            let function = function_around(&text, at);
            // two writers are not the placed entity's: the unpack of a session's own record (the entity is inserted into its sector list right after, ob_pack.c and cr_pack.c: the next test) and the raw
            // vec3d setter that only the creators' attribute application uses (the last test)
            let exempt = (file == "entity/fixed/fx_pack.c" && function.contains("unpack_fixed_data"))
                || (file == "entity/fixed/fx_pack.c" && function.contains("pack_fixed_data"))
                || (file == "entity/fixed/fx_vec3d.c" && function.contains("set_local_raw_vec3d"))
                || (file == "entity/fixed/fx_vec3d.c" && function.contains("get_local_vec3d_ptr"));
            if exempt {
                continue;
            }
            assert!(
                function.contains("campaign_place_fixed_state ("),
                "{file}:{}: `{}` writes a field of a fixed entity's collision state in a function that does not state it to the host's record (campaign_place_fixed_state)",
                at + 1,
                body.trim()
            );
        }
    }
    assert!(writers >= 20, "only {writers} writers were seen: the census did not reach the fixed family");
    // the setters every other writer goes through state the new value BEFORE they return, in the same case
    let int_setter = source("entity/fixed/fx_int.c");
    for assignment in ["raw->sub_type = value;", "raw->object_3d_shape = value;"] {
        let at = int_setter.find(assignment).unwrap_or_else(|| panic!("fx_int.c: `{assignment}` is not there"));
        let case = &int_setter[at..];
        let stated = case.find("campaign_place_fixed_state (en);").expect("the case states the value");
        let end = case.find("break;").expect("the case ends");
        assert!(stated < end, "fx_int.c: `{assignment}` is not stated before its case ends");
    }
    assert_eq!(
        int_setter.matches("campaign_place_fixed_state (en);").count(),
        2,
        "fx_int.c: ALIVE is not collision state and states nothing"
    );
    let float_setter = source("entity/fixed/fx_float.c");
    before(&float_setter, "raw->heading = value;", "campaign_place_fixed_state (en);", "fx_float.c");
    let position_setter = source("entity/fixed/fx_vec3d.c");
    before(
        &position_setter,
        "insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, parent, NULL);",
        "campaign_place_fixed_state (en);",
        "fx_vec3d.c",
    );
}

#[test]
fn a_session_unpack_writes_the_record_before_the_insertion_and_every_creator_completes_the_state_before_it_inserts() {
    // the two session unpacks of a fixed entity write the record (position, sub type, shape, heading) of an entity that is not placed, and insert it afterwards: the insertion states it
    for file in ["entity/fixed/object/ob_pack.c", "entity/fixed/crater/cr_pack.c"] {
        let text = source(file);
        before(
            &text,
            "unpack_fixed_data (en, &raw->fix, mode);",
            "insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR",
            file,
        );
    }
    // no other pack file unpacks the fixed record's own fields (the others only set the alive and shape of entities that exist already, which the previous test covers)
    for (file, text) in fixed_files() {
        if file.ends_with("_pack.c") && !file.ends_with("ob_pack.c") && !file.ends_with("cr_pack.c") && file != "entity/fixed/fx_pack.c" {
            assert!(
                !code_of(&text).contains("unpack_fixed_data"),
                "{file} unpacks the fixed record: its insertion must state it"
            );
        }
    }
    // every creator resolves the entity's shape from its type's default BEFORE it inserts the entity into its sector list (a building lists itself in its city first and its sector second, both after)
    for (file, text) in fixed_files() {
        if !file.ends_with("_creat.c") || file.contains("/city/") {
            continue;
        }
        let Some(insert) = text.find("insert_local_entity_into_parents_child_list (en, LIST_TYPE_") else {
            continue;
        };
        let resolved = text.find("default_3d_shape;").or_else(|| text.find("default_object_3d_shape"));
        let resolved = resolved.unwrap_or_else(|| panic!("{file}: no default shape is resolved"));
        assert!(resolved < insert, "{file}: the shape is resolved after the first insertion");
        if file.ends_with("cb_creat.c") {
            let sector = text.find("LIST_TYPE_SECTOR").unwrap();
            let city = text.find("LIST_TYPE_CITY_BUILDING").unwrap();
            assert!(city < sector, "{file}: a building is listed in its city before its sector");
        }
    }
}

/// the code of a whole file: every line's code without its `//` comment
fn code_of(text: &str) -> String {
    text.lines().map(code).collect::<Vec<_>>().join("\n")
}

#[test]
fn the_fixed_family_is_the_types_that_carry_the_fixed_record_the_funnel_is_the_last_step_of_both_insertions_and_raw_setters_are_the_creators_alone() {
    // the family the host's record holds a state for is exactly the entity types that overload the fixed functions, a city excepted (it holds its own position and box: see its census)
    let mut overloaded: Vec<String> = Vec::new();
    for (file, text) in fixed_files() {
        if file.ends_with("_funcs.c") && file != "entity/fixed/fx_funcs.c" {
            for line in text.lines() {
                if let Some(rest) = code(line).trim().strip_prefix("overload_fixed_functions (") {
                    overloaded.push(rest.trim_end_matches([')', ';', ' ']).to_string());
                }
            }
        }
    }
    overloaded.retain(|name| name != "ENTITY_TYPE_CITY");
    overloaded.sort();
    let family_text = source("entity/mobile/mb_hostplace.c");
    let start = family_text.find("static int place_fixed_family").expect("the family");
    let end = family_text[start..].find("\n}\n").unwrap() + start;
    let mut family: Vec<String> = family_text[start..end]
        .lines()
        .filter_map(|line| {
            code(line)
                .trim()
                .strip_prefix("case ")
                .map(|rest| rest.trim_end_matches(':').trim().to_string())
        })
        .collect();
    family.sort();
    assert_eq!(
        family, overloaded,
        "the host holds a state for the fixed entity types the campaign has, no more and no fewer"
    );
    assert!(overloaded.len() >= 8, "{overloaded:?}");
    // both insertions state the entity's state as their LAST step, after the host has issued the key and declared the placements one entity
    let map = source("entity/mobile/mb_hostplacemap.h");
    for (function, last) in [
        ("static void place_insert (", "place_register (slot, 0, key);"),
        ("static void place_child_insert (", "place_register (slot, 1, key);"),
    ] {
        let at = map.find(function).unwrap_or_else(|| panic!("{function}"));
        let end = map[at..].find("\n}\n").unwrap() + at;
        let body = &map[at..end];
        before(body, last, "place_declare (en, slot);", function);
        before(body, "place_declare (en, slot);", "place_fixed_state (en);", function);
        assert!(
            body.trim_end().ends_with("place_fixed_state (en);"),
            "{function}: the state is the last thing a placement states"
        );
    }
    // the state is read from the entity at the moment of the report and never kept by the campaign: the funnel reads, sends and keeps nothing
    let at = map.find("static void place_fixed_state (").unwrap();
    let end = map[at..].find("\n}\n").unwrap() + at;
    let body = &map[at..end];
    assert!(body.contains("CAMPAIGN_PLACE_FIXED_READ (en, state);") && body.contains("campaign_model_entity_fixed (CAMPAIGN_PLACE_TYPE (en), key, state);"));
    assert!(
        !body.lines().skip(1).any(|line| code(line).contains("static ")),
        "the funnel keeps no state of its own"
    );
    // a raw setter (which does not relink or state anything) is used only by the creators' attribute application, which runs before an entity is inserted
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../aphavoc/source");
    let mut files = Vec::new();
    walk(&root, &mut files);
    for path in files {
        let relative = path.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
        if relative.starts_with("entity/system/en_attrs/") || relative.starts_with("entity/system/en_funcs/") || relative.starts_with("entity/mobile/") {
            continue;
        }
        let text = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).replace("\r\n", "\n");
        for (at, line) in text.lines().enumerate() {
            let body = code(line);
            let own_table = body.contains("fn_set_local_entity_raw_") || body.trim_start().starts_with("fn_set_local_entity_raw");
            assert!(
                own_table || !body.contains("set_local_entity_raw_"),
                "{relative}:{}: `{}` is a raw setter used outside the creators' attribute application",
                at + 1,
                body.trim()
            );
        }
    }
}

#[test]
fn a_non_mobile_candidate_names_the_placement_the_host_served_it_by_and_a_mobile_still_names_its_identity() {
    let offer = source("entity/mobile/mb_hostoffer.h");
    // a building listed under a city names its place in the city's list, every other non-mobile candidate its sector placement; a mobile keeps its semantic identity
    assert!(offer.contains("CAMPAIGN_OFFER_FIXED_KEY (target, TRUE)") && offer.contains("CAMPAIGN_OFFER_FIXED_KEY (target, FALSE)"));
    assert!(offer.contains("host_id = CAMPAIGN_OFFER_HOST_ID (target);"));
    before(
        &offer,
        "CAMPAIGN_OFFER_HOST_ID (target);",
        "CAMPAIGN_OFFER_FIXED_KEY (target, FALSE)",
        "mb_hostoffer.h",
    );
    // the key is taken once per offer into a local copy and the same value is sent with every staged question of that offer; nothing is kept between offers
    assert!(offer.contains("char\n\t\tfixed_key [CAMPAIGN_ROSTER_KEY_MAX + 1];"));
    assert!(!offer.contains("static char"), "the offer keeps no state between candidates");
    let collision = source("entity/en_misc/en_maths/en_cllsn.c");
    assert!(collision.contains("#define CAMPAIGN_OFFER_FIXED_KEY(E,OF_CITY) campaign_place_key ((E), (OF_CITY))"));
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// THE CITY'S OWN COLLISION STATE (Task49, the follow-on of the fixed family): its position and bounding box are held by the host's record of the city's placement, so EVERY writer of either, in the supported
// single-player runtime and the unpack routes that exist in source, must state them. A city is a placement of the sector list like the fixed family and carries the same fixed record, but it has no setter,
// no pack data and no shape of its own: these are the writers that exist, asserted as text so that a new writer fails here before any native run (they are not a proof of behaviour: the isolated dialogue,
// the provider tests and the native control are).

#[test]
fn the_cities_position_and_box_have_exactly_these_writers() {
    // the box: the creator's sentinel (the original's inverted initial box) before the attributes and the insertion, and the expansion a building's link makes; no other file assigns it, and every reader of
    // the box through its getters only reads (the avionics draw a hatched circle, the collision offer reads it)
    let created = source("entity/fixed/city/ct_creat.c");
    for sentinel in ["raw->bounding_box_min.x = 1000000.0;", "raw->bounding_box_max.x = -1000000.0;"] {
        before(&created, sentinel, "set_local_entity_attributes (en, pargs);", "ct_creat.c");
    }
    before(
        &created,
        "set_local_entity_attributes (en, pargs);",
        "insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR,",
        "ct_creat.c",
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../aphavoc/source");
    let mut files = Vec::new();
    walk(&root, &mut files);
    let mut assigners = Vec::new();
    let mut readers = Vec::new();
    for path in files {
        let name = path.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
        let text = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).replace("\r\n", "\n");
        for line in text.lines().map(code) {
            let assigns = ["bounding_box_min.", "bounding_box_max.", "bounding_box_min->", "bounding_box_max->"]
                .iter()
                .any(|member| {
                    line.match_indices(member).any(|(at, _)| {
                        let rest = &line[at + member.len()..];
                        // `member.x = ...`, `member.x < ...` is a read: only a single `=` that is not `==` assigns
                        let after = rest.trim_start_matches(|c: char| c.is_ascii_alphabetic()).trim_start();
                        after.starts_with('=') && !after.starts_with("==")
                    })
                });
            if assigns && !assigners.contains(&name) {
                assigners.push(name.clone());
            }
            if line.contains("VEC3D_TYPE_BOUNDING_BOX_") && !readers.contains(&name) && !name.starts_with("entity/system/en_funcs/") {
                readers.push(name.clone());
            }
        }
    }
    assigners.sort();
    assert_eq!(
        assigners,
        ["entity/fixed/city/ct_creat.c".to_string(), "entity/fixed/city/ct_msgs.c".to_string()],
        "the box has a creator's sentinel and one expansion: a new writer must state it to the host's record"
    );
    readers.sort();
    // the avionics (a hatched circle), the collision offer (its conformance words), the city's own getters and, since the city's state moved behind its host record, the one funnel reader (mb_hostplace.c)
    assert_eq!(readers.len(), 11, "{readers:?}");
    assert!(readers.iter().all(|file| file.starts_with("gunships/avionics/")
        || file == "entity/en_misc/en_maths/en_cllsn.c"
        || file == "entity/fixed/city/ct_vec3d.c"
        || file == "entity/mobile/mb_hostplace.c"));
    // a city has no setter of the box (the vec3d overload table has only getters), and its session pack and unpack are empty: nothing restores a city's position or box
    let vec3d = source("entity/fixed/city/ct_vec3d.c");
    assert!(
        !vec3d.contains("fn_set_local_entity_vec3d") && !vec3d.contains("fn_set_local_entity_raw_vec3d"),
        "a city has no setter of its own"
    );
    assert!(vec3d.contains("#define SET_LOCAL_REQUIRED\t\t\t\t0") && vec3d.contains("#define SET_CLIENT_SERVER_REQUIRED\t0"));
    let pack = source("entity/fixed/city/ct_pack.c");
    for function in [
        "static void pack_local_data (entity *en, pack_modes mode)\n{\n}",
        "static void unpack_local_data (entity *en, entity_types type, pack_modes mode)\n{\n}",
    ] {
        assert!(pack.contains(function), "a city's pack hooks are empty: {function}");
    }
    // the position: the creator's default and the attribute the population file gives (the raw setter, before the insertion), and the shared setter every fixed entity registers; the population reader never
    // moves a city after it created it
    assert!(created.contains("raw->fix.position.x = MID_MAP_X;"));
    let popread = source("ai/faction/popread.c");
    let at = popread.find("ENTITY_TYPE_CITY,").expect("popread creates the city");
    let end = popread[at..].find("ENTITY_ATTR_END").unwrap() + at;
    assert!(popread[at..end].contains("ENTITY_ATTR_VEC3D ( VEC3D_TYPE_POSITION, x, y, z ),"));
    assert!(!popread.contains("set_local_entity_vec3d ( city") && !popread.contains("set_local_entity_raw_vec3d ( city"));
}

#[test]
fn every_writer_of_a_placed_citys_position_or_box_states_them_through_the_one_funnel_after_the_write() {
    // the expansion a building's link makes is the only writer of a PLACED city's box. In the ORIGINAL-fixture build (the original expansion, kept as the characterisation oracle) it states the city's whole state as its
    // last step, after the six comparisons and the three assertions; the shipping build has the host build the box and states nothing back (the next test)
    let messages = source("entity/fixed/city/ct_msgs.c");
    let (_, original) = city_messages_branches(&messages);
    let at = original.find("static void update_city_bounding_box").expect("the expansion");
    let end = original[at..].find("\n}\n").unwrap() + at;
    let body = &original[at..end];
    before(body, "raw->bounding_box_max.z = p2.z;", "campaign_place_fixed_state (this_city);", "ct_msgs.c");
    before(
        body,
        "ASSERT (raw->bounding_box_min.z <= raw->bounding_box_max.z);",
        "campaign_place_fixed_state (this_city);",
        "ct_msgs.c",
    );
    assert_eq!(body.matches("campaign_place_fixed_state (").count(), 1, "one statement, after the last write");
    assert!(
        body.trim_end().ends_with("campaign_place_fixed_state (this_city);"),
        "the statement is the last step of the expansion"
    );
    assert!(messages.contains("#include \"entity/mobile/mb_hostplace.h\""));
    // the position's writers of a placed city are the shared setter's two branches, both of which state it (the relink through its insertion), and the funnel handles a city by its type
    let setter = source("entity/fixed/fx_vec3d.c");
    before(
        &setter,
        "insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, parent, NULL);",
        "campaign_place_fixed_state (en);",
        "fx_vec3d.c",
    );
    let map = source("entity/mobile/mb_hostplacemap.h");
    let at = map.find("static void place_fixed_state (").expect("the funnel");
    let end = map[at..].find("\n}\n").unwrap() + at;
    let funnel = &map[at..end];
    assert!(
        funnel.contains("CAMPAIGN_PLACE_CITY (CAMPAIGN_PLACE_TYPE (en))"),
        "the funnel recognises the city by its type"
    );
    assert!(funnel.contains("CAMPAIGN_PLACE_CITY_READ (en, city);") && funnel.contains("campaign_model_entity_city_state (key, city);"));
    // the state is read from the entity at the moment of the report and kept by nobody
    assert!(
        !funnel.lines().skip(1).any(|line| code(line).contains("static ")),
        "the funnel keeps no state of its own"
    );
    // the city is a type of the entity system, bound in the real placement map by its own type, and its reads are the entity's own getters (position, then the two box pointers)
    let place = source("entity/mobile/mb_hostplace.c");
    assert!(place.contains("#define CAMPAIGN_PLACE_CITY(TYPE) (((entity_types) (TYPE)) == ENTITY_TYPE_CITY)"));
    assert!(place.contains("#define CAMPAIGN_PLACE_CITY_READ(E,OUT) place_city_read ((E), (OUT))"));
    let at = place.find("static void place_city_read").expect("the reader");
    let end = place[at..].find("\n}\n").unwrap() + at;
    let reader = &place[at..end];
    for getter in ["VEC3D_TYPE_POSITION", "VEC3D_TYPE_BOUNDING_BOX_MIN", "VEC3D_TYPE_BOUNDING_BOX_MAX"] {
        assert!(reader.contains(getter), "{getter}");
    }
    assert!(
        !reader.contains("float_value") && !reader.contains("get_local_entity_vec3d ("),
        "the words are copied as the entity holds them, never loaded as floats"
    );
    // the city is NOT of the fixed family the host holds a fixed-shape state for (it holds its own position and box instead)
    assert!(!place.contains("case ENTITY_TYPE_CITY:"));
    // the first statement is the insertion's: place_insert states a city as it states the family, as its last step
    let at = map.find("static void place_insert (").unwrap();
    let end = map[at..].find("\n}\n").unwrap() + at;
    assert!(map[at..end].trim_end().ends_with("place_fixed_state (en);"));
}

/// The text of `ct_msgs.c` the SHIPPING (hosted) build compiles and the text only the ORIGINAL-fixture build compiles: the part before `#else` and the part after it, both inside the one `#ifdef EECH_CAMPAIGN_HOST_ENTITY`
/// that holds the expansion. A file with no such conditional compiles all of it into the shipping build, so it is all hosted text and nothing is original-only.
fn city_messages_branches(text: &str) -> (String, String) {
    let Some(start) = text.find("#ifdef EECH_CAMPAIGN_HOST_ENTITY") else {
        return (text.to_string(), String::new());
    };
    let after = &text[start..];
    let Some(other) = after.find("\n#else") else {
        return (text.to_string(), String::new());
    };
    let end = after.find("\n#endif").expect("the conditional ends");
    (
        format!("{}{}", &text[..start], &after[..other]),
        format!("{}{}", &text[..start], &after[other..end]),
    )
}

#[test]
fn the_shipping_cities_box_expansion_is_the_hosts_and_the_original_physical_body_stays_only_in_the_original_fixture_build() {
    let messages = source("entity/fixed/city/ct_msgs.c");
    let (hosted, original) = city_messages_branches(&messages);
    // the physical work (the scene's bounds, the heading's rotation as two corners, the offset and the union of the two corners with the box) is not compiled into the shipping campaign
    for physical in [
        "get_object_3d_bounding_box_without_lines",
        "get_3d_transformation_matrix",
        "multiply_matrix3x3_vec3d",
        "object_3d_bounds",
        "FLOAT_TYPE_HEADING",
        "raw->bounding_box_min.x = p1.x;",
    ] {
        assert!(!hosted.contains(physical), "the shipping build still computes the city's box: `{physical}`");
    }
    // the shipping build asks the host once per link, with the link's own pair, and applies the checked answer to the entity's mirror
    assert!(
        hosted.contains("campaign_place_city_expand (this_city, this_building, box);"),
        "the link asks the host"
    );
    assert_eq!(hosted.matches("campaign_place_city_expand (").count(), 1, "one question per link");
    before(
        &hosted,
        "campaign_place_city_expand (this_city, this_building, box);",
        "raw->bounding_box_min.x = box [0];",
        "ct_msgs.c",
    );
    assert!(
        !hosted.contains("campaign_place_fixed_state (this_city);"),
        "the host holds the box it built: the campaign does not state it back"
    );
    // the original fixture build keeps the original expansion, statement for statement, as the characterisation oracle
    for original_statement in [
        "bounding_box = get_object_3d_bounding_box_without_lines (object_3d_index);",
        "get_3d_transformation_matrix (attitude, heading, 0.0, 0.0);",
        "multiply_matrix3x3_vec3d (&p1, attitude, &p1);",
        "multiply_matrix3x3_vec3d (&p2, attitude, &p2);",
        "if (p2.z > raw->bounding_box_max.z) raw->bounding_box_max.z = p2.z;",
    ] {
        assert!(original.contains(original_statement), "the oracle lost `{original_statement}`");
    }
}

#[test]
fn an_expanded_boxs_axis_order_is_checked_from_the_raw_words_before_any_word_of_the_callers_box_is_written() {
    let text = source("entity/mobile/mb_hostentity.h");
    let start = text.find("static void check_city_expand_reply (").expect("the reply check");
    let end = start
        + text[start..]
            .find("\nstatic void campaign_entity_city_expand_exchange")
            .expect("the exchange follows the check");
    let check = &text[start..end];
    let order = "finite_word_order (&reply->candidate [index]) > finite_word_order (&reply->candidate [index + 3])";
    assert!(check.contains(order), "the expanded axes are ordered by the reply check");
    // every raw word (finite, no word beyond the six) is classified before the order, and no floating value of the untrusted words is loaded or compared
    before(check, "candidate_word_is_finite (&reply->candidate [index])", order, "mb_hostentity.h");
    before(check, "the host answered more than %d words for a city expansion", order, "mb_hostentity.h");
    assert!(
        !check.contains("reply->candidate [index] >") && !check.contains("reply->candidate [index] <"),
        "no float comparison of an untrusted word"
    );
    // the check is the exchange's, before the words are copied out: the campaign's ASSERTs in ct_msgs.c only log in a shipping build and run after the writes
    let exchange = &text[end..];
    before(
        exchange,
        "check_city_expand_reply (&reply, city);",
        "memcpy (box, reply.candidate",
        "mb_hostentity.h",
    );
}

/// the text of a C function from its signature to the closing brace at the start of a line, with the comments of its lines removed
fn function_code(text: &str, signature: &str) -> String {
    let at = text.find(signature).unwrap_or_else(|| panic!("`{signature}` is not there"));
    let end = text[at..].find("\n}\n").unwrap() + at;
    code_of(&text[at..end])
}

#[test]
fn the_reporter_states_the_sub_type_the_host_derives_its_rules_from_and_never_the_flags_the_campaign_derived() {
    // the original derives the collision-fixed and align flags from the fixed record's sub type (fx_int.c, fixed_database[sub_type]); the host derives them too (roster.rs, FIXED_RULES) from the sub type it is
    // stated, so neither reporter reads a flag getter: a flag the campaign derived is never the host's physical input
    let place = source("entity/mobile/mb_hostplace.c");
    for signature in ["static void place_fixed_read (", "static void place_city_read ("] {
        let body = function_code(&place, signature);
        assert!(body.contains("INT_TYPE_ENTITY_SUB_TYPE"), "{signature}: the sub type is stated");
        for flag in ["INT_TYPE_COLLISION_TEST_FIXED", "INT_TYPE_ALIGN_WITH_TERRAIN", "fixed_database"] {
            assert!(!body.contains(flag), "{signature} reads {flag}, a value the campaign derived");
        }
    }
    assert!(function_code(&place, "static void place_fixed_read (").contains("state [5] = (float) get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);"));
    assert!(function_code(&place, "static void place_city_read (").contains("state [9] = (float) get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);"));
    // the domain of the fact is the declared enumeration, tied at compile time to the number the host's table is sized by
    assert!(code_of(&place).contains(
        "typedef char campaign_fixed_sub_type_domain_is_the_declared_enumeration [(CAMPAIGN_FIXED_SUB_TYPES == NUM_ENTITY_SUB_TYPE_FIXED) ? 1 : -1];"
    ));
    // the wire layout: the sub type is fact 10 of both reports, the two flag facts no longer exist, and the state arrays are one word each longer or shorter accordingly
    let model = code_of(&source("entity/mobile/mb_hostmodel.h"));
    for definition in [
        "#define CAMPAIGN_FIXED_SUBTYPE 10",
        "#define CAMPAIGN_CITY_SUBTYPE 10",
        "#define CAMPAIGN_FIXED_SUB_TYPES 15",
        "#define CAMPAIGN_FIXED_STATE_WORDS 6",
        "#define CAMPAIGN_CITY_STATE_WORDS 11",
        "#define CAMPAIGN_CITY_SHAPE 9",
    ] {
        assert!(model.contains(definition), "{definition}");
    }
    assert!(!model.contains("CAMPAIGN_FIXED_COLLISION") && !model.contains("CAMPAIGN_FIXED_ALIGN"));
    let entity = code_of(&source("entity/mobile/mb_hostentity.h"));
    assert!(entity.contains("request.pose [CAMPAIGN_FIXED_SUBTYPE] = state [5];"));
    assert!(entity.contains("request.pose [CAMPAIGN_CITY_SUBTYPE] = state [9];"));
    // the city's shape (Task49, the host-owned sector height): the one fact the host needs of a city to decide the raise at the city's arrival, stated in the fact the fixed report uses for its withdrawal word
    assert!(entity.contains("request.pose [CAMPAIGN_CITY_SHAPE] = state [10];"));
    assert!(function_code(&place, "static void place_city_read (").contains("state [10] = (float) get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);"));
    assert!(!entity.contains("CAMPAIGN_FIXED_COLLISION") && !entity.contains("CAMPAIGN_FIXED_ALIGN"));
}

#[test]
fn a_fixed_entitys_sub_type_has_these_writers_each_before_the_insertion_or_stated_by_the_setter() {
    // every assignment to a fixed record's sub type in the fixed family: a creator's default (before the entity is inserted), the session unpack (the previous test: before the insertion) and the setter
    // that states the change to the host. A creator's attribute list (parser.c, popread.c, bridge.c, py_creat.c) reaches the same setter through the raw funnel before the insertion.
    let mut writers: Vec<(String, usize)> = Vec::new();
    for (file, text) in fixed_files() {
        let count = code_of(&text)
            .lines()
            .filter(|line| {
                let line = line.replace(' ', "");
                // a write to the record's member (`raw->sub_type =`, `raw->fix.sub_type =`): a local variable of the same name (a pylon creator's attribute, a segment's read) is not a write to a record
                (line.contains("->sub_type=") || line.contains(".sub_type=")) && !line.contains("sub_type==")
            })
            .count();
        if count > 0 {
            writers.push((file, count));
        }
    }
    let expected: Vec<(String, usize)> = [
        ("entity/fixed/cbuild/cb_creat.c", 1),
        ("entity/fixed/city/ct_creat.c", 1),
        ("entity/fixed/crater/cr_creat.c", 1),
        ("entity/fixed/fx_int.c", 1),
        ("entity/fixed/fx_pack.c", 1),
        ("entity/fixed/object/ob_creat.c", 1),
        ("entity/fixed/scenic/sn_creat.c", 1),
        ("entity/fixed/segment/sg_creat.c", 1),
        ("entity/fixed/site/st_creat.c", 1),
        ("entity/fixed/siteupdt/su_creat.c", 1),
    ]
    .map(|(file, count)| (file.to_string(), count))
    .to_vec();
    assert_eq!(writers, expected, "a new writer of a fixed sub type must state the change to the host");
    // a creator's default is written before its first insertion into any list
    for (file, text) in fixed_files() {
        if !file.ends_with("_creat.c") {
            continue;
        }
        let text = code_of(&text);
        let Some(write) = text.find("fix.sub_type =") else { continue };
        let Some(insert) = text.find("insert_local_entity_into_parents_child_list (en, LIST_TYPE_") else {
            continue;
        };
        assert!(write < insert, "{file}: the sub type is written after the first insertion");
    }
    // the setter's write is followed, in the same case, by the statement to the host
    let setter = source("entity/fixed/fx_int.c");
    let at = setter.find("raw->sub_type = value;").expect("the setter's write");
    let case = &setter[at..at + setter[at..].find("break;").unwrap()];
    assert!(
        code_of(case).contains("campaign_place_fixed_state (en);"),
        "the setter states a change of the sub type"
    );
    // the getters stay as the conformance mirror: they still derive from the original's table
    let getters = code_of(&setter);
    assert!(getters.contains("value = fixed_database[raw->sub_type].collision_test_fixed;"));
    assert!(getters.contains("value = fixed_database[raw->sub_type].align_with_terrain;"));
}

#[test]
fn the_no_host_extractions_of_the_list_funnel_carry_the_hook_header_that_compiles_every_report_away() {
    // en_list.c reports every sector and city-list write through `campaign_place_*` (mb_hostplace.h, which it includes beside project.h). The no-host builds that compile its insert and delete bodies VERBATIM into a generated
    // unit (the original-semantics oracle crate eech-sys and the historical C reference harness) drop that include, so the generated unit must name the header itself: with no EECH_CAMPAIGN_HOST_ENTITY defined
    // the header turns every report into `((void) 0)`, which is the original, no-host list behaviour. Without it the unit does not compile (implicit declarations at en_list.c:498, 503, 571 and 575).
    let list = source("entity/system/en_funcs/en_list.c");
    assert!(code_of(&list).contains("#include \"entity/mobile/mb_hostplace.h\""));
    for call in [
        "campaign_place_insert (en);",
        "campaign_place_child_insert (en, parent, pred);",
        "campaign_place_remove (en);",
        "campaign_place_child_remove (en);",
    ] {
        assert!(code_of(&list).contains(call), "en_list.c no longer calls {call}");
    }
    let header = code_of(&source("entity/mobile/mb_hostplace.h"));
    let (host, no_host) = header.split_once("#else").expect("the header's no-host branch");
    for hook in [
        "campaign_place_insert",
        "campaign_place_child_insert",
        "campaign_place_remove",
        "campaign_place_child_remove",
    ] {
        assert!(host.contains(hook), "{hook} is a declared hook");
        assert!(
            no_host.contains(&format!("#define {hook}(")),
            "{hook} must compile away in a build with no host entity"
        );
    }
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let read =
        |relative: &str| String::from_utf8_lossy(&std::fs::read(repo.join(relative)).unwrap_or_else(|e| panic!("{relative}: {e}"))).replace("\r\n", "\n");
    let spec = read("eech-campaign/crates/eech-sys/build/spec.rs");
    assert!(
        spec.contains("Raw(\"#include \\\"project.h\\\"\\n\\n#include \\\"entity/mobile/mb_hostplace.h\\\"\\n\\n#define DEBUG_MODULE 0\\n\\n#define LANDING_DEBUG 0\\n\".into()),"),
        "eech-sys: the generated eech_extracted.c names the hook header"
    );
    let harness = read("eech-core-ts/c-reference/extract.mjs");
    assert!(
        harness.contains(
            "text: '#include \"project.h\"\\n\\n#include \"entity/mobile/mb_hostplace.h\"\\n\\n#define DEBUG_MODULE 0\\n\\n#define LANDING_DEBUG 0\\n'"
        ),
        "the historical harness: the generated eech_extracted.c names the hook header"
    );
}

/// The city's release (REQUIRED_CITY_BOX_LIFECYCLE_RETENTION): the host keeps a city's record for the bounding box it built, so the campaign must state when the city ENTITY ends. Read from the compiled source:
///
///  1. the release is the sixteenth phase of the scan question, the value the host's provider (`ROSTER_RELEASE`, 16.0 as bits) and the driver script classify;
///  2. a removal keeps the record's name only when the host named it in the answer (`place_retain_record`, a city only), and the release is stated while the city's name is still held, before it is dropped;
///  3. a placement that joins a record the host kept is declared to continue it by the same LINK a city with listed buildings uses (the name is kept in `place_list_of_slot`), and the answer of a building's
///     removal can only name the list it was asked about.
#[test]
fn the_city_release_is_the_phase_the_provider_and_the_driver_classify_and_the_campaign_states_it_while_it_holds_the_name() {
    let model = source("entity/mobile/mb_hostmodel.h");
    let after_enum = &model[model.find("enum campaign_model_scan_phase").unwrap()..];
    let body = &after_enum[after_enum.find('{').unwrap() + 1..after_enum.find("};").unwrap()];
    let mut text = String::new();
    let mut rest = body;
    while let Some(start) = rest.find("/*") {
        text.push_str(&rest[..start]);
        rest = &rest[start + rest[start..].find("*/").unwrap() + 2..];
    }
    text.push_str(rest);
    let members: Vec<&str> = text.split(',').filter_map(|member| member.split_whitespace().next()).collect();
    assert_eq!(members.iter().position(|member| *member == "CAMPAIGN_SCAN_PHASE_CITY_EXPAND"), Some(15));
    assert_eq!(members.iter().position(|member| *member == "CAMPAIGN_SCAN_PHASE_CITY_RELEASE"), Some(16));
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read =
        |relative: &str| String::from_utf8_lossy(&std::fs::read(repo.join(relative)).unwrap_or_else(|e| panic!("{relative}: {e}"))).replace("\r\n", "\n");
    assert_eq!(f32::from_bits(0x4180_0000), 16.0, "the phase word of 16.0");
    assert!(
        read("crates/eech-world/src/model_host.rs").contains("const ROSTER_RELEASE: u32 = 0x4180_0000;"),
        "the provider classifies the release as phase 16.0"
    );
    assert!(
        read("crates/eech-world/lua/entity-authority.lua").contains("[\"0x41800000\"] = 16"),
        "the driver script classifies the release as phase 16"
    );

    let map = source("entity/mobile/mb_hostplacemap.h");
    let release = &map[map.find("static void place_release_entity").unwrap()..map.find("static void place_reset").unwrap()];
    before(
        release,
        "campaign_model_entity_city (CAMPAIGN_SCAN_PHASE_CITY_RELEASE, CAMPAIGN_PLACE_TYPE (en), place_list_of_slot [index], answered);",
        "place_list_of_slot [index][0] = 0;",
        "mb_hostplacemap.h place_release_entity",
    );
    assert!(
        release.contains("CAMPAIGN_PLACE_CITY (CAMPAIGN_PLACE_TYPE (en))"),
        "only a city entity states a release"
    );
    let remove = &map[map.find("static void place_remove").unwrap()..map.find("static void place_child_insert").unwrap()];
    assert!(
        remove.contains("place_retain_record (slot, answered);"),
        "a removal keeps only the name the host answered"
    );
    // the acknowledgement is validated BEFORE any key is forgotten or any list name written (a pure check, then the commit)
    before(
        remove,
        "place_check_retained (en, slot, answered);",
        "place_forget_key (slot, 0);",
        "mb_hostplacemap.h place_remove",
    );
    before(
        remove,
        "place_forget_key (slot, 0);",
        "place_retain_record (slot, answered);",
        "mb_hostplacemap.h place_remove",
    );
    let child_remove = &map[map.find("static void place_child_remove").unwrap()..map.find("static void place_release_entity").unwrap()];
    before(
        child_remove,
        "the campaign's city does not hold\", answered, slot);",
        "place_forget_key (slot, 1);",
        "mb_hostplacemap.h place_child_remove",
    );
    let check = &map[map.find("static void place_check_retained").unwrap()..map.find("static void place_retain_record").unwrap()];
    assert!(
        check.contains("CAMPAIGN_PLACE_CITY (CAMPAIGN_PLACE_TYPE (en))")
            && check.contains("debug_fatal")
            && check.contains("strcmp (place_key_of_slot [slot], name)"),
        "a name for an entity that is no city, or for another city's record, stops the engine"
    );
    let retain = &map[map.find("static void place_retain_record").unwrap()..map.find("// `en` was inserted at the head of a sector's list").unwrap()];
    assert!(
        !retain.contains("debug_fatal"),
        "the commit validates nothing: it only writes what the check accepted"
    );
    let declare = &map[map.find("static void place_declare").unwrap()..map.find("static void place_fixed_state").unwrap()];
    assert!(
        declare.contains("else if (place_key_of_slot [slot][0] && place_list_of_slot [slot][0])"),
        "a placement that holds the record's name is declared to continue it"
    );
}

/// The CITY box lifecycle witness (`eech_wpn_fixture.c`, feature `weapon-fixture`) reaches the changed Core placement map ONLY through the campaign's own shared services, and never asks or answers the host itself. Read from the source:
///
///  1. it drains and relists the city's buildings with the real list funnel and relinks the city with the shared fixed-position setter, and asks the unchanged production entity query;
///  2. it makes no host question of its own: no `campaign_model_` call and no `campaign_place_` call but the read-only key lookup, and it declares itself terminal;
///  3. the engine entry, the raw binding, the Rust wrapper and the Lua method name the same function, all under the weapon-fixture gate, so no default or shipping build holds it.
#[test]
fn the_city_lifecycle_witness_uses_only_the_real_shared_services_and_asks_the_host_nothing_itself() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read =
        |relative: &str| String::from_utf8_lossy(&std::fs::read(repo.join(relative)).unwrap_or_else(|e| panic!("{relative}: {e}"))).replace("\r\n", "\n");
    let fixture = read("crates/eech-engine-sys/csrc/eech_wpn_fixture.c");
    let start = fixture.find("#define CL_MAX_BUILDINGS").expect("the witness");
    let witness = &fixture[start..];
    for needed in [
        "delete_local_entity_from_parents_child_list (buildings[i], LIST_TYPE_CITY_BUILDING)",
        "insert_local_entity_into_parents_child_list (buildings[i], LIST_TYPE_CITY_BUILDING, city, NULL)",
        "set_local_entity_vec3d (city, VEC3D_TYPE_POSITION, &moved)",
        "set_local_entity_vec3d (city, VEC3D_TYPE_POSITION, &position)",
        "get_line_of_sight_collision_entity (NULL, NULL, &a, &b, &point, &normal)",
        "terminal=1",
        "placement_key=%s",
        "box_min_after=%s box_max_after=%s",
        "memset (answer->point, 0, sizeof (answer->point));",
    ] {
        assert!(witness.contains(needed), "the witness lacks `{needed}`");
    }
    assert!(!witness.contains("campaign_model_"), "the witness asks the host nothing itself");
    for line in witness.lines().filter(|line| line.contains("campaign_place_")) {
        assert!(
            line.contains("campaign_place_key ("),
            "the witness calls no placement hook but the key lookup: {line}"
        );
    }
    assert!(
        fixture[..start].contains("#ifdef EECH_WEAPON_FIXTURE"),
        "the witness is inside the weapon-fixture gate"
    );
    assert!(
        witness.contains("#ifdef EECH_CAMPAIGN_HOST_ENTITY") && witness.contains("not_a_host_asking_build"),
        "the Original-like build reports the witness unreached"
    );
    let engine = read("crates/eech-engine-sys/csrc/eech_engine.c");
    assert!(
        engine.contains("eech_wpn_city_lifecycle_run (text, capacity)") && engine.contains("int eech_engine_wpn_city_lifecycle (char *text, int capacity)")
    );
    assert!(read("crates/eech-engine-sys/src/lib.rs").contains("pub fn eech_engine_wpn_city_lifecycle(text: *mut c_char, capacity: c_int) -> c_int;"));
    let wrapper = read("crates/eech-engine/src/lib.rs");
    let at = wrapper.find("pub fn wpn_city_lifecycle(&self)").expect("the wrapper");
    assert!(
        wrapper[..at].rfind("#[cfg(feature = \"weapon-fixture\")]").is_some_and(|gate| at - gate < 80),
        "the wrapper is gated"
    );
    let module = std::fs::read(repo.join("crates/eech-dc/src/module.rs")).expect("module.rs");
    let module = String::from_utf8_lossy(&module);
    let at = module.find("methods.add_method(\"wpn_city_lifecycle\"").expect("the Lua method");
    assert!(
        module[..at].rfind("#[cfg(feature = \"weapon-fixture\")]").is_some_and(|gate| at - gate < 80),
        "the Lua method is gated"
    );
}
