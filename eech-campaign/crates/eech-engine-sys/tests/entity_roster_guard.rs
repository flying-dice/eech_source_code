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
    // the anchor is the host record's name: kept when the last building leaves a placed city, dropped only when the record ends (no placement and no building)
    let child_remove = &map[map.find("static void place_child_remove").unwrap()..map.find("static void place_release_entity").unwrap()];
    before(
        child_remove,
        "if (!place_key_of_slot [city][0])",
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
        // a city is not of the fixed family the host holds a state for (its own position and box are a declared follow-on)
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
    // the family the host's record holds a state for is exactly the entity types that overload the fixed functions, a city excepted (its own position and box are a declared follow-on)
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
