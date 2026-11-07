import sys

ROOT = sys.argv[1] + "/eech-campaign/crates/eech-engine-sys/tests/"


def patch(name, pairs):
    p = ROOT + name
    d = open(p, "rb").read().decode("utf-8")
    for old, new in pairs:
        assert d.count(old) == 1, (name, old[:90], d.count(old))
        d = d.replace(old, new)
    open(p, "wb").write(d.encode("utf-8"))


patch("entity_roster_guard.rs", [
    ('''        "#define CAMPAIGN_CITY_STATE_WORDS 10",
    ] {''', '''        "#define CAMPAIGN_CITY_STATE_WORDS 11",
        "#define CAMPAIGN_CITY_SHAPE 9",
    ] {'''),
    ('''    assert!(entity.contains("request.pose [CAMPAIGN_CITY_SUBTYPE] = state [9];"));
''', '''    assert!(entity.contains("request.pose [CAMPAIGN_CITY_SUBTYPE] = state [9];"));
    // the city's shape (Task49, the host-owned sector height): the one fact the host needs of a city to decide the raise at the city's arrival, stated in the fact the fixed report uses for its withdrawal word
    assert!(entity.contains("request.pose [CAMPAIGN_CITY_SHAPE] = state [10];"));
    assert!(function_code(&place, "static void place_city_read (").contains("state [10] = (float) get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);"));
'''),
])

patch("sector_height_guard.rs", [
    # (a) the reader census gains the pack's getter call
    ('''            ("aphavoc/source/entity/special/sector/sc_float.c", 2),
            ("aphavoc/source/entity/system/en_funcs/en_float.c", 1),
            ("aphavoc/source/entity/system/en_funcs/en_float.h", 1),
        ]),
        "a new reader of FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT needs its own characterisation"''', '''            ("aphavoc/source/entity/special/sector/sc_float.c", 2),
            ("aphavoc/source/entity/special/sector/sc_pack.c", 1),
            ("aphavoc/source/entity/system/en_funcs/en_float.c", 1),
            ("aphavoc/source/entity/system/en_funcs/en_float.h", 1),
        ]),
        "a new reader of FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT needs its own characterisation"'''),
    # (b) the getter: the host's word in a hosted build, the stored member only in the build that keeps the original body
    ('''    assert!(float.contains("case FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT: { value = raw->tallest_structure_height; break; }"));
''', '''    // Task49 (the host-owned sector height): in a hosted build the getter is the ONE read of the host's word, and the stored member is read only by the original body (the build that keeps the original entity solver)
    let getter = source("aphavoc/source/entity/special/sector/sc_float.c");
    let host_read = line_of(&getter, "campaign_model_sector_height ((int) raw->x_sector, (int) raw->z_sector, &value);", "sc_float.c");
    assert_eq!(enclosing(&getter, host_read), ["defined EECH_CAMPAIGN_HOST_ENTITY"]);
    let legacy_read = line_of(&getter, "value = raw->tallest_structure_height;", "sc_float.c");
    assert_eq!(enclosing(&getter, legacy_read), ["else of defined EECH_CAMPAIGN_HOST_ENTITY"]);
    assert!(float.contains("case FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT: { #ifdef EECH_CAMPAIGN_HOST_ENTITY"));
    assert!(float.contains("#endif break; }"));
'''),
])

# (c), (d), (e), (f) in the same file, in a second pass so the patch above stays readable
patch("sector_height_guard.rs", [
    ('''    // `height` is a float local: the sum is one float expression
    assert!(text.contains("float height; vec3d *position;"));
''', '''    // `height` is a float local: the sum is one float expression
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
'''),
    # (d) the pack and the unpack
    ('''    before(
        &text,
        "pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_COUNT, raw->keysite_count); pack_float_value (en, FLOAT_TYPE_PROTOTYPE_1, raw->tallest_structure_height); } } }",
        "void unpack_local_sector_data (pack_modes mode)",
        "sc_pack.c",
    );
    // unpack: the mirror, last in the record
    assert!(text.contains(
        "raw->keysite_count = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_COUNT); raw->tallest_structure_height = unpack_float_value (en, FLOAT_TYPE_PROTOTYPE_1); } } } }"
    ));''', '''    // Task49: the word packed is the HOST's (the getter's read, the same raw 32 bits), and the original line is compiled only where the original body is kept; neither is followed by anything in the record
    let code = squeeze(&pack.lines().map(|line| line.split("//").next().unwrap_or("")).collect::<Vec<_>>().join("\\n"));
    before(
        &code,
        "pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_COUNT, raw->keysite_count); #ifdef EECH_CAMPAIGN_HOST_ENTITY pack_float_value (en, FLOAT_TYPE_PROTOTYPE_1, get_local_entity_float_value (en, FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT)); #else pack_float_value (en, FLOAT_TYPE_PROTOTYPE_1, raw->tallest_structure_height); #endif } } }",
        "void unpack_local_sector_data (pack_modes mode)",
        "sc_pack.c",
    );
    // unpack: the mirror, last in the record: in a hosted build the saved word is stated to the host (it holds exactly these bits afterwards) and the legacy field is not written
    assert!(code.contains(
        "raw->keysite_count = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_COUNT); #ifdef EECH_CAMPAIGN_HOST_ENTITY { float saved_word; saved_word = unpack_float_value (en, FLOAT_TYPE_PROTOTYPE_1); campaign_model_sector_height_restore ((int) raw->x_sector, (int) raw->z_sector, &saved_word); } #else raw->tallest_structure_height = unpack_float_value (en, FLOAT_TYPE_PROTOTYPE_1); #endif } } } }"
    ));'''),
    # (e) the city's shape
    ('''    // the city's state names no shape, and its creation sets the invalid shape with the default resolution commented out: whether a real city is ever created with a valid shape is NOT known from source
    let city_read = place
        .find("static void place_city_read (struct ENTITY *en, float *state) {")
        .expect("the city state reader");
    let city_read_end = place[city_read..]
        .find("state [9] = (float) get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE); }")
        .expect("the end of the city state reader");
    assert!(
        !place[city_read..city_read + city_read_end].contains("SHAPE"),
        "the host's city state would then carry a shape"
    );''', '''    // Task49: the city's state states its SHAPE (fact 9: the one fact the host needs of a city to decide the raise at the city's arrival). Its creation sets the invalid shape with the default resolution commented out, so a city raises
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
    );'''),
])

# (f) a guard that no hosted-build code reads or writes the legacy field
p = ROOT + "sector_height_guard.rs"
d = open(p, "rb").read().decode("utf-8")
marker = "#[test]\nfn the_raise_is_a_fixed_entity_at_a_sector_list_link_by_position_y_plus_the_box_top_and_nothing_lowers_it() {"
assert d.count(marker) == 1
new = '''#[test]
fn the_legacy_field_is_neither_read_nor_written_in_a_hosted_build_and_the_host_is_asked_at_exactly_the_getter_and_the_unpack() {
    // Task49 (the host-owned sector height): every access of the member through a pointer is compiled only where the original entity solver is kept, so a hosted build neither reads nor writes it (no mirror, no fallback)
    for (file, text) in engine_files() {
        for (index, line) in text.lines().enumerate() {
            if line.contains("->tallest_structure_height") {
                let conditions = enclosing(text, index);
                assert!(
                    conditions.iter().any(|condition| condition == "!defined EECH_CAMPAIGN_HOST_ENTITY" || condition == "else of defined EECH_CAMPAIGN_HOST_ENTITY"),
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

'''
d = d.replace(marker, new + marker)
open(p, "wb").write(d.encode("utf-8"))
print("ok")
