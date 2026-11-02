//
// Source patches of the headless engine build: the ONLY places where the
// engine compiles something other than the original EECH text. Each patch
// replaces text that must occur exactly `count` times in the original file;
// the build fails if it does not. Every patch is listed, with its evidence,
// in eech-campaign/docs/engine.md, and reviewed (defect correction or
// intentional semantic change) in eech-campaign/docs/corrections.md.
//

pub struct Patch {
    pub file: &'static str,
    pub id: &'static str,
    #[allow(dead_code)]
    pub why: &'static str,
    pub original: &'static str,
    pub replacement: &'static str,
    pub count: usize,
}

pub const PATCHES: &[Patch] = &[
    Patch {
        file: "modules/system/debug.h",
        id: "E1-debug-log-macros",
        why: "T1 (docs/64-bit.md): the WIN32 release debug_log macros are `#define debug_log();`, called with arguments. Only MSVC's preprocessor accepts that. GCC-family compilers take EECH's own variadic definitions.",
        original: "#ifdef WIN32\n# ifdef __BORLANDC__\n",
        replacement: "/* EECH headless (E1): GCC-family compilers take the variadic definitions below */\n#if defined(WIN32) && !defined(__GNUC__)\n# ifdef __BORLANDC__\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/highlevl/highlevl.h",
        id: "E2-ai-log-macro",
        why: "T1 (docs/64-bit.md): the WIN32 release ai_log is `#define ai_log();`, called with arguments. GCC-family compilers take EECH's own variadic definition.",
        original: "#ifdef WIN32\n#define ai_log();\n",
        replacement: "/* EECH headless (E2): GCC-family compilers take the variadic definition below */\n#if defined(WIN32) && !defined(__GNUC__)\n#define ai_log();\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/special/effect/explosn/xp_dbase.h",
        id: "E3-explosion-union-member-names",
        why: "Two anonymous structs of one union both declare `frequency` and `smoke_lifetime`. MSVC's C++ front end accepts the repeated names (both at offsets 4 and 8); C rejects them. The second struct's copies are renamed; the layout and every access (which resolves to the first struct's members at the same offsets) are unchanged.",
        original: "\t\t\t\t\t\tgenerator_lifetime,\n\t\t\t\t\t\tfrequency,\n\t\t\t\t\t\tsmoke_lifetime;\n",
        replacement: "\t\t\t\t\t\tgenerator_lifetime,\n\t\t\t\t\t\t/* EECH headless (E3): same offsets as the first struct's frequency and smoke_lifetime */\n\t\t\t\t\t\tfrequency_shared_with_first_layout,\n\t\t\t\t\t\tsmoke_lifetime_shared_with_first_layout;\n",
        count: 1,
    },
    Patch {
        file: "modules/system/timer.c",
        id: "E4-timer-callback-declaration",
        why: "create_timer_event passes windows_timer_interrupt, declared with three DWORD parameters, as timeSetEvent's LPTIMECALLBACK, whose parameters are DWORD_PTR (the Win64 SDK's, and the compat shim's). On i686 both are 32-bit and the types are identical; on x86-64 they differ, and GCC 14+ rejects the call (-Wincompatible-pointer-types is an error). Declare the callback with the DWORD_PTR parameters timeSetEvent hands it.",
        original: "static void CALLBACK windows_timer_interrupt ( UINT\tIDEvent, UINT\tuReserved, DWORD\tdwUser, DWORD dwReserved1, DWORD dwReserved2 );\n",
        replacement: "static void CALLBACK windows_timer_interrupt ( UINT\tIDEvent, UINT\tuReserved, DWORD_PTR\tdwUser, DWORD_PTR dwReserved1, DWORD_PTR dwReserved2 ); /* EECH headless (E4) */\n",
        count: 1,
    },
    Patch {
        file: "modules/system/timer.c",
        id: "E4-timer-callback-user-data",
        why: "E4: the user data is the callback's function pointer, which `(DWORD) function` truncates to 32 bits on x86-64 before windows_timer_interrupt casts it back. Pass it as the DWORD_PTR timeSetEvent takes (identical on i686).",
        original: "timeSetEvent ( delay, resolution, windows_timer_interrupt, (DWORD) function, TIME_PERIODIC );\n",
        replacement: "timeSetEvent ( delay, resolution, windows_timer_interrupt, (DWORD_PTR) function, TIME_PERIODIC ); /* EECH headless (E4) */\n",
        count: 1,
    },
    Patch {
        file: "modules/system/timer.c",
        id: "E4-timer-callback-definition",
        why: "E4: the definition matching the declaration.",
        original: "static void CALLBACK windows_timer_interrupt ( UINT IDEvent, UINT uReserved, DWORD dwUser, DWORD dwReserved1, DWORD dwReserved2 )\n",
        replacement: "static void CALLBACK windows_timer_interrupt ( UINT IDEvent, UINT uReserved, DWORD_PTR dwUser, DWORD_PTR dwReserved1, DWORD_PTR dwReserved2 ) /* EECH headless (E4) */\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/system/en_funcs/en_creat.c",
        id: "P1a-stack-attributes-include",
        why: "64-bit blocker B1 (docs/64-bit.md, docs/patches.md P1): declares the attribute-list marshaller.",
        original: "#include \"project.h\"\n",
        replacement: "#include \"project.h\"\n\n/* EECH headless (P1) */\n#include \"eech_attrs.h\"\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/system/en_funcs/en_creat.c",
        id: "P1b-stack-attributes-storage",
        why: "64-bit blocker B1: storage for the marshalled attribute list, living as long as the original's argument stack did (this call frame).",
        original: "\tchar\n\t\t*pargs_buffer;\n",
        replacement: "\tchar\n\t\t*pargs_buffer;\n\n\t/* EECH headless (P1) */\n\tchar\n\t\teech_stack_attributes[EECH_STACK_ATTRIBUTES_SIZE];\n",
        count: 2,
    },
    Patch {
        file: "aphavoc/source/entity/system/en_funcs/en_creat.c",
        id: "P1c-stack-attributes-marshal",
        why: "64-bit blocker B1: `(char *) pargs` reinterprets a va_list as the i386 argument stack; on x86-64 it is a descriptor. The marshaller (csrc/eech_attrs.c) walks the attribute grammar with va_arg and writes the list get_list_item expects.",
        original: "\t\tpargs_buffer = (char *) pargs;\n",
        replacement: "\t\t/* EECH headless (P1) */\n\t\tpargs_buffer = eech_marshal_stack_attributes (eech_stack_attributes, sizeof (eech_stack_attributes), pargs);\n",
        count: 2,
    },
    Patch {
        file: "modules/userint2/ui_sys/ui_attrs/ui_attrs.c",
        id: "B2-ui-pointer-attributes",
        why: "64-bit blocker B2 (docs/engine.md): UI_ATTR_ASSOCIATION/CHILD/NEXT/PARENT/PREV pass a ui_object pointer, and set_ui_object_attributes reads it back as `(ui_object *) va_arg (pargs, int)`. On i386 both are 4 bytes; on x86-64 the pointer is truncated to 32 bits and the first UI screen built crashes (initialise_init_screen). Read the argument as the pointer it was passed as.",
        original: "= (ui_object *) va_arg (pargs, int);",
        replacement: "= (ui_object *) va_arg (pargs, void *); /* EECH headless (B2) */",
        count: 5,
    },
    Patch {
        file: "modules/userint2/ui_sys/ui_attrs/ui_attrs.c",
        id: "B2-ui-graphic-attributes",
        why: "64-bit blocker B2: UI_ATTR_TEXTURE_GRAPHIC / HIGHLIGHTED_ / SELECTED_TEXTURE_GRAPHIC / ZOOMABLE_PALETTE_GRAPHIC pass a graphic pointer, read back into an `int` and cast to the pointer type. On x86-64 the pointer is truncated (the first screen with a texture graphic crashes). Read it as a pointer.",
        original: "\t\t\t\tint\n\t\t\t\t\tgraphic;\n\n\t\t\t\tgraphic = va_arg (pargs, int);\n",
        replacement: "\t\t\t\tvoid\n\t\t\t\t\t*graphic; /* EECH headless (B2) */\n\n\t\t\t\tgraphic = va_arg (pargs, void *);\n",
        count: 4,
    },
    Patch {
        file: "modules/system/fpu.h",
        id: "X1-float-to-int",
        why: "64-bit blocker: the GNU convert_float_to_int / convert_double_to_int are `fistp (%1)`, which in AT&T syntax is the 16-bit store. Only the low 16 bits of the int are written, and values of 32,768 or more saturate, so every world coordinate past 32.7 km falls in the wrong sector (get_x_sector). The asm also pops the x87 stack without declaring it. Convert in C: on x86-64 that is cvttss2si/cvttsd2si, truncation, the rounding mode EECH sets for these conversions.",
        original: "#elif defined ( __GNUC__ )\n\ninline static void asm_convert_float_to_int ( float value, int *integer ) __attribute__((always_inline));\ninline static void asm_convert_float_to_int ( float value, int *integer )\n{\n  __asm__ __volatile__ (\"fistp (%1);\"\n\t\t\t\t\t\t: /* no outputs */ : \"t\" (value), \"d\" (integer) : \"memory\" );\n}\n\ninline static void asm_convert_double_to_int ( double value, int *integer ) __attribute__((always_inline));\ninline static void asm_convert_double_to_int ( double value, int *integer )\n{\n  __asm__ __volatile__ (\"fistp (%1);\"\n\t\t\t\t\t\t: /* no outputs */ : \"t\" (value), \"d\" (integer) : \"memory\" );\n}\n",
        replacement: "#elif defined ( __GNUC__ )\n\n/* EECH headless (X1): C conversions; cvttss2si/cvttsd2si truncate, the rounding EECH sets */\ninline static void asm_convert_float_to_int ( float value, int *integer ) __attribute__((always_inline));\ninline static void asm_convert_float_to_int ( float value, int *integer )\n{\n\t*integer = (int) value;\n}\n\ninline static void asm_convert_double_to_int ( double value, int *integer ) __attribute__((always_inline));\ninline static void asm_convert_double_to_int ( double value, int *integer )\n{\n\t*integer = (int) value;\n}\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/special/force/fc_msgs.c",
        id: "S1-supplier-not-requester",
        why: "A keysite low on ammo or fuel asks its force for a supplier: the closest factory or refinery, or the closest airbase if that is nearer. The airbase lookup excludes no keysite, so an airbase asking finds itself (0 km) and is its own supplier; it has no cargo of what it lacks, so no SUPPLY task is ever created and no airbase is ever resupplied. Exclude the requester, as get_closest_keysite allows.",
        original: "\t\t\tairbase = get_closest_keysite (ENTITY_SUB_TYPE_KEYSITE_AIRBASE, side, pos, 10 * KILOMETRE, &airbase_actual_range, TRUE, NULL);\n",
        replacement: "\t\t\tairbase = get_closest_keysite (ENTITY_SUB_TYPE_KEYSITE_AIRBASE, side, pos, 10 * KILOMETRE, &airbase_actual_range, TRUE, sender); /* EECH headless (S1) */\n",
        count: 2,
    },
    Patch {
        file: "modules/3d/terrain/terrelev.c",
        id: "N1-nan-terrain-query",
        why: "get_3d_terrain_point_data clamps off-map positions with x < min || x > max tests, which a NaN passes; its sector index is then garbage and the lookup reads outside terrain_sectors (an access violation 12 simulated hours into the Windows Lebanon campaign). Report the callers of a NaN query and use the map corner, so the run survives and the NaN's source is found.",
        original: "\tif ( ( x < terrain_3d_min_map_x ) || ( x > terrain_3d_max_map_x ) || ( z < terrain_3d_min_map_z ) || ( z > terrain_3d_max_map_z ) )\n\t{\n\t\tASSERT (!\"Terrain elevation off map\");",
        replacement: "\tif ( ( x != x ) || ( z != z ) )\n\t{\n\t\t/* EECH headless (N1): a NaN passes every range test below */\n\t\textern void eech_report_nan_position (const char *what, void *caller0, void *caller1, void *caller2);\n\t\teech_report_nan_position (\"terrain elevation\", __builtin_return_address (0), NULL, NULL);\n\t\tx = terrain_3d_min_map_x + 0.0001f;\n\t\tz = terrain_3d_min_map_z + 0.0001f;\n\t}\n\n\tif ( ( x < terrain_3d_min_map_x ) || ( x > terrain_3d_max_map_x ) || ( z < terrain_3d_min_map_z ) || ( z > terrain_3d_max_map_z ) )\n\t{\n\t\tASSERT (!\"Terrain elevation off map\");",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/mobile/mb_msgs.c",
        id: "S3-pick-up-the-tasks-cargo",
        why: "At a SUPPLY task's pick-up waypoint the transport takes the keysite's first cargo crate, whatever its type. Factories and airbases hold both ammo and fuel crates, so an ammo task often loads fuel (and the task still ends in success at the drop-off): in 3 simulated hours of Lebanon no ammo crate was ever picked up at a factory, and airbases never got ammo back. Take a crate of the task's cargo type (its user data, set by create_supply_task), and only failing that the first crate, as before.",
        original: "\tkeysite = get_local_entity_parent (sender, LIST_TYPE_TASK_DEPENDENT);\n\n\tcargo = get_local_entity_first_child (keysite, LIST_TYPE_CARGO);\n\n\twhile ((cargo) && (get_local_entity_type (cargo) != ENTITY_TYPE_CARGO))\n\t{\n\n\t\tcargo = get_local_entity_child_succ (cargo, LIST_TYPE_CARGO);\n\t}\n",
        replacement: "\tkeysite = get_local_entity_parent (sender, LIST_TYPE_TASK_DEPENDENT);\n\n\t/* EECH headless (S3): the crate of the task's cargo type */\n\t{\n\t\tentity *task = get_local_entity_parent (sender, LIST_TYPE_WAYPOINT);\n\t\tint wanted = task && (get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_TASK_SUPPLY) ? (int) get_local_entity_float_value (task, FLOAT_TYPE_TASK_USER_DATA) : -1;\n\n\t\tfor (cargo = get_local_entity_first_child (keysite, LIST_TYPE_CARGO); cargo; cargo = get_local_entity_child_succ (cargo, LIST_TYPE_CARGO))\n\t\t{\n\t\t\tif ((get_local_entity_type (cargo) == ENTITY_TYPE_CARGO) && (get_local_entity_int_value (cargo, INT_TYPE_ENTITY_SUB_TYPE) == wanted))\n\t\t\t{\n\t\t\t\tbreak;\n\t\t\t}\n\t\t}\n\t}\n\n\tif (!cargo)\n\t{\n\tcargo = get_local_entity_first_child (keysite, LIST_TYPE_CARGO);\n\n\twhile ((cargo) && (get_local_entity_type (cargo) != ENTITY_TYPE_CARGO))\n\t{\n\n\t\tcargo = get_local_entity_child_succ (cargo, LIST_TYPE_CARGO);\n\t}\n\t}\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ui_menu/ingame/campaign/ca_hist.c",
        id: "H1-empty-campaign-history",
        why: "maintain_campaign_history runs whenever a campaign item is destroyed (response_to_campaign_item_destroyed); headless, the UI history is always empty. With an empty history it reads campaign_history[-1] (AddressSanitizer: global-buffer-overflow), and can call remove_campaign_history (-1), which sets num_campaign_history_items to -1; the next addition then writes campaign_history[-1], corrupting the global before the array. Nothing to maintain in an empty history. (The Windows Lebanon crash 11-12 simulated hours in was N3, not this; docs/corrections.md.)",
        original: "\t//\n\t// Consider currently displayed page\n\t//\n\n\tlast = num_campaign_history_items - 1;\n",
        replacement: "\t//\n\t// Consider currently displayed page\n\t//\n\n\tif (num_campaign_history_items <= 0) /* EECH headless (H1) */\n\t{\n\t\treturn;\n\t}\n\n\tlast = num_campaign_history_items - 1;\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ui_menu/options/op_real.c",
        id: "U1-report-targets-text-count",
        why: "The realism options page sizes its co-pilot target-report button from option_cpg_report_targets_text, a two-entry array, but passes a count of 4, reading two pointers past it at boot (AddressSanitizer: global-buffer-overflow).",
        original: "\tpreprocess_translation_object_size (change_array [i], cpg_report_targets_area, option_cpg_report_targets_text, 4, RESIZE_OPTION_CYCLE_BUTTON);",
        replacement: "\tpreprocess_translation_object_size (change_array [i], cpg_report_targets_area, option_cpg_report_targets_text, 2, RESIZE_OPTION_CYCLE_BUTTON); /* EECH headless (U1) */",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/special/effect/smokelst/sl_move.c",
        id: "N2-nan-smoke-point-moving",
        why: "A smoke point with a NaN position reaches get_3d_terrain_elevation (wind drift), which indexes the terrain with it (N1): the Windows Lebanon crash 11-12 simulated hours in. Report the point (its smoke list, type and parent entity) and drop it.",
        original: "\t\tif ( smoke_info->wind_affected && !bound_position_to_map_volume(smoke_pos))\n",
        replacement: "\t\tif ( ( smoke_pos->x != smoke_pos->x ) || ( smoke_pos->y != smoke_pos->y ) || ( smoke_pos->z != smoke_pos->z ) ) /* EECH headless (N2) */\n\t\t{\n\t\t\textern void eech_report_nan_smoke (const char *where, entity *smoke, int smoke_type, const vec3d *pos, const vec3d *motion);\n\t\t\teech_report_nan_smoke (\"moving\", en, raw->smoke_type, smoke_pos, smoke_mv);\n\t\t\tsmoke_pos->x = smoke_pos->y = smoke_pos->z = -1.0;\n\t\t\traw->valid[ current ] = FALSE;\n\t\t\traw->alive_count --;\n\t\t\tcontinue;\n\t\t}\n\n\t\tif ( smoke_info->wind_affected && !bound_position_to_map_volume(smoke_pos))\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/special/effect/smokelst/sl_updt.c",
        id: "N2-nan-smoke-point-created",
        why: "Where a NaN smoke point comes from (N2): report a point created with a NaN position or motion, with its smoke list and parent entity.",
        original: "\thead_pos = &(raw->position[ raw->head ]);\n",
        replacement: "\tif ( ( new_pos->x != new_pos->x ) || ( new_pos->y != new_pos->y ) || ( new_pos->z != new_pos->z ) || ( initial_velocity->x != initial_velocity->x ) || ( initial_velocity->y != initial_velocity->y ) || ( initial_velocity->z != initial_velocity->z ) ) /* EECH headless (N2) */\n\t{\n\t\textern void eech_report_nan_smoke (const char *where, entity *smoke, int smoke_type, const vec3d *pos, const vec3d *motion);\n\t\teech_report_nan_smoke (\"created\", en, smoke_type, new_pos, initial_velocity);\n\t}\n\n\thead_pos = &(raw->position[ raw->head ]);\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/terrain/terrelev.c",
        id: "N3-degenerate-terrain-face",
        why: "get_3d_terrain_point_data takes a face's normal from a cross product of its edges; a degenerate retail terrain triangle (collinear or coincident points) gives a zero cross product, which normalise_any_3d_vector leaves as a zero vector, so the elevation (dy = ... / face_normal.y) is 0/0, NaN. Wind drift multiplies smoke motion by it, and the next lookup indexes the terrain with a NaN position: the Windows Lebanon crash 11-12 simulated hours in (N1, N2). Use the up vector, as the original's commented-out fallback below does; that fallback also set the elevation to the sector maximum, where this keeps the vertex height (docs/corrections.md).",
        original: "\t\t\t\tnormalise_3d_vector (&normal);\n\n\t\t\t\tpoint_data->face_normal = normal;\n",
        replacement: "\t\t\t\tnormalise_3d_vector (&normal);\n\n\t\t\t\t/* EECH headless (N3): a degenerate face has no normal; take the up vector, as the commented-out debug code below does */\n\t\t\t\tif ( !( normal.y > 0.00001f ) )\n\t\t\t\t{\n\t\t\t\t\tnormal.x = 0.0;\n\t\t\t\t\tnormal.y = 1.0;\n\t\t\t\t\tnormal.z = 0.0;\n\t\t\t\t}\n\n\t\t\t\tpoint_data->face_normal = normal;\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "C1-null-collision-object",
        why: "A scene's collision object index 0 means none: object 0 is the null object, with no points or surfaces. Scenes read from .EES files map 0 to -1, but scenes read from 3dobjdb.bin keep it, and the retail database stores 0 (RS_MANPAD, for one). The first weapon tested against such a scene reads the null object's NULL surface list (get_object_3d_collision_object_geometry_triangle). Map 0 to -1 on this path too.",
        original: "\t\tfread ( &objects_3d_scene_database[scene_index].collision_object_index, sizeof ( int ), 1, fp );\n",
        replacement: "\t{\n\t\tfread ( &objects_3d_scene_database[scene_index].collision_object_index, sizeof ( int ), 1, fp );\n\t\tif ( !objects_3d_scene_database[scene_index].collision_object_index ) objects_3d_scene_database[scene_index].collision_object_index = -1; /* EECH headless (C1) */\n\t}\n",
        count: 1,
    },
    Patch {
        file: "modules/graphics/textuser.c",
        id: "T1-texture-camo-mismatch",
        why: "Headless: the community objects (setup/cohokum/3ddata/objects) over a retail Apache vs Havoc database disagree with its texture set on which textures are camouflaged, and texture registration treats that as fatal. Nothing is drawn headless, so it is ignored (debug_log compiles to nothing: DEBUG is not defined).",
        original: "\t\t\tdebug_fatal ( \"Texture '%s': %s defined it as",
        replacement: "\t\t\tdebug_log ( /* EECH headless (T1) */ \"Texture '%s': %s defined it as",
        count: 2,
    },
    Patch {
        file: "modules/3d/3dobjid.c",
        id: "T2-missing-texture-animation",
        why: "Headless: a community object (setup/cohokum/3ddata/objects) can name a texture animation the retail texture set lacks (the community installer adds them as TGA files). Nothing is drawn headless: use animation 0 (the debug_log compiles to nothing: DEBUG is not defined), so the object's geometry, collision mesh and weapon mounts still load.",
        original: "\t\tdebug_fatal ( \"FAILED to find texture animation '%s'\", animation_name );\n",
        replacement: "\t\tdebug_log ( \"FAILED to find texture animation '%s'\", animation_name ); /* EECH headless (T2) */\n\t\ttexture_animation = 0;\n",
        count: 1,
    },
];

/// The host-owned model data patches (EEDCS-50, D3; aphavoc/source/entity/mobile/mb_hostmodel.h): the four public accessors of the 3d module that read the campaign
/// DLL's own model arrays ask the host instead, and the host's scenes are installed (and cross-checked against the scene database the campaign just loaded) at the end of
/// initialise_3d_objects. They are part of the SHIPPING build and of the test variants that run the campaign; main.rs does NOT apply them for the `model-fixture`
/// feature, whose purpose is to ask the ORIGINAL accessors what they answer (that build has no host and says so). Same strict count check as PATCHES; the original
/// accessor body is kept in the staged text only inside `#if 0`, so the patch can be audited against the source and nothing of it is compiled.
pub const MODEL_HOST_PATCHES: &[Patch] = &[
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "D1a-model-triangle-from-host",
        why: "get_object_3d_collision_object_geometry_triangle decoded the collision triangles from the DLL's own object arrays. The host owns the models: the accessor asks the host for one step of the triangle cursor (FOUND writes the caller's buffer; END and ABSENT are the original's FALSE and leave it untouched; anything else stops the engine). The original body follows inside #if 0.",
        original: "int get_object_3d_collision_object_geometry_triangle ( object_3d_index_numbers index, vec3d *return_points, int depth )\n{\n",
        replacement: "extern int campaign_model_triangle ( int scene, vec3d *points, int depth ); /* EECH headless (D1) */\n\nint get_object_3d_collision_object_geometry_triangle ( object_3d_index_numbers index, vec3d *return_points, int depth )\n{\n\treturn ( campaign_model_triangle ( (int) index, return_points, depth ) );\n}\n\n#if 0 /* EECH headless (D1): the original, array-reading body is not compiled; see mb_hostmodel.h */\nint get_object_3d_collision_object_geometry_triangle ( object_3d_index_numbers index, vec3d *return_points, int depth )\n{\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "D1b-model-triangle-original-ends",
        why: "closes the #if 0 opened by D1a at the end of the original accessor (this text occurs once, at the end of that function)",
        original: "\t\t\treturn ( FALSE );\n\t\t}\n\t}\n\telse\n\t{\n\n\t\treturn ( FALSE );\n\t}\n}\n\n#ifndef OGRE_EE\n",
        replacement: "\t\t\treturn ( FALSE );\n\t\t}\n\t}\n\telse\n\t{\n\n\t\treturn ( FALSE );\n\t}\n}\n#endif /* EECH headless (D1) */\n\n#ifndef OGRE_EE\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "D2-model-scenes-installed",
        why: "the scene database is loaded and consolidated: ask the host for its catalogue and every scene, cross-check the shape identity, fill the immutable tables of the host's bounds and radius. A missing provider, a catalogue that disagrees or a malformed scene stops the boot.",
        original: "\tinitialised_3d_objects = TRUE;\n",
        replacement: "\tinitialised_3d_objects = TRUE;\n\n\t{\n\t\textern void campaign_model_install_scenes ( void ); /* EECH headless (D2) */\n\t\tcampaign_model_install_scenes ();\n\t}\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjvis.c",
        id: "D3a-model-bounding-box-from-host",
        why: "get_object_3d_bounding_box returned a pointer into the DLL's own scene entry. It returns a pointer into the host-authored table filled at boot (callers keep it across loops, so it is stable for the life of the process).",
        original: "\treturn ( &objects_3d_scene_database[object].bounding_box );\n",
        replacement: "\t{\n\t\textern struct OBJECT_3D_BOUNDS *campaign_model_bounding_box ( int scene ); /* EECH headless (D3) */\n\t\treturn ( campaign_model_bounding_box ( (int) object ) );\n\t}\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjvis.c",
        id: "D3b-model-bounding-box-without-lines-from-host",
        why: "as D3a, for the bounds the collision tests use",
        original: "\treturn ( &objects_3d_scene_database[object].bounding_box2 );\n",
        replacement: "\t{\n\t\textern struct OBJECT_3D_BOUNDS *campaign_model_bounding_box_without_lines ( int scene ); /* EECH headless (D3) */\n\t\treturn ( campaign_model_bounding_box_without_lines ( (int) object ) );\n\t}\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjvis.c",
        id: "D3c-model-radius-from-host",
        why: "as D3a, for the radius",
        original: "\treturn ( objects_3d_scene_database[object].radius );\n",
        replacement: "\t{\n\t\textern float campaign_model_radius ( int scene ); /* EECH headless (D3) */\n\t\treturn ( campaign_model_radius ( (int) object ) );\n\t}\n",
        count: 1,
    },
];

pub fn apply_model_host(file: &str, text: &str) -> String {
    let mut out = text.to_string();
    for p in MODEL_HOST_PATCHES.iter().filter(|p| p.file == file) {
        let found = out.matches(p.original).count();
        assert_eq!(
            found, p.count,
            "model-host patch {} expects {} occurrence(s) of its original text in {}, found {}",
            p.id, p.count, file, found
        );
        out = out.replace(p.original, p.replacement);
    }
    out
}

pub fn model_host_files() -> Vec<&'static str> {
    let mut files: Vec<&str> = MODEL_HOST_PATCHES.iter().map(|p| p.file).collect();
    files.sort_unstable();
    files.dedup();
    files
}

/// TEST-ONLY patches of the `model-fixture` feature (EEDCS-50): observation hooks in the real model loader, applied by main.rs ONLY when that feature is on,
/// to the staged copy that the shipping patches (C1) already produced. They record what the loader allocated, filled and consumed (csrc/eech_model_hooks.h);
/// each ADDS a call and changes no original statement, argument, order or value (a wrapped `fread` is evaluated exactly as before and its result handed back
/// unchanged). With the feature off none of this exists in the build. Same strict count check as PATCHES.
pub const MODEL_FIXTURE_PATCHES: &[Patch] = &[
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M1-hooks-include",
        why: "declares the observer hooks (csrc/eech_model_hooks.h)",
        original: "#include\t\"3d.h\"\n",
        replacement: "#include\t\"3d.h\"\n#include\t\"eech_model_hooks.h\" /* EECH model fixture (M1) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M2-reset-on-initialise",
        why: "a (re)load drops every record and moves the generation on, so nothing recorded for an earlier load can match",
        original: "\tinitialise_object_3d_lookup_tables ();\n",
        replacement: "\teech_mf_reset (); /* EECH model fixture (M2) */\n\tinitialise_object_3d_lookup_tables ();\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M3-common-faces-read",
        why: "the fread of one object's common faces: where the block starts, where this read began, items asked for and items returned",
        original: "\t\t\t\tfread ( current_faces, sizeof ( struct OBJECT_3D_FACE ), number_of_real_polygons, fp );\n",
        replacement: "\t\t\t\teech_mf_common_read ( 0, object_database_faces, current_faces, number_of_real_polygons, (long long) fread ( current_faces, sizeof ( struct OBJECT_3D_FACE ), number_of_real_polygons, fp ) ); /* EECH model fixture (M3) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M4-common-point-list-read",
        why: "the fread of one object's common face point list, as M3",
        original: "\t\t\t\tfread ( current_faces_point_list, sizeof ( struct POINT_3D_SHORT_REFERENCE ), point_reference_count, fp );\n",
        replacement: "\t\t\t\teech_mf_common_read ( 1, object_database_faces_point_list, current_faces_point_list, point_reference_count, (long long) fread ( current_faces_point_list, sizeof ( struct POINT_3D_SHORT_REFERENCE ), point_reference_count, fp ) ); /* EECH model fixture (M4) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M5-common-done",
        why: "the object loop is over and the override files (which mutate objects) are not yet read: the exact common allocations and every cursor the loader advanced",
        original: "\tfclose ( fp );\n\n\t/* 26JUL06 Casm Import of 3D objects BEGIN */\n",
        replacement: "\tfclose ( fp );\n\n\teech_mf_common_done ( object_database_faces, total_number_of_polygons, current_faces, object_database_faces_point_list, total_number_of_plain_points, current_faces_point_list, object_database_points, current_points, object_database_surface_point_list, current_object_surface_point_references, object_database_surfaces, total_number_of_surfaces, current_surfaces ); /* EECH model fixture (M5) */\n\n\t/* 26JUL06 Casm Import of 3D objects BEGIN */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M6-custom-points-filled",
        why: "read_object: how many points fread returned",
        original: "\t\tfread ( obj->points, 6, obj->number_of_points, file );\n",
        replacement: "\t\teech_mf_filled ( 0, fread ( obj->points, 6, obj->number_of_points, file ) ); /* EECH model fixture (M6) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M7-custom-faces-filled",
        why: "read_object: how many polygon faces fread returned",
        original: "\t\tfread ( obj->faces, 1, number_of_polygoned_faces, file );\n",
        replacement: "\t\teech_mf_filled ( 1, fread ( obj->faces, 1, number_of_polygoned_faces, file ) ); /* EECH model fixture (M7) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M8-custom-point-list-filled",
        why: "read_object: how many point references fread returned",
        original: "\t\tfread ( obj->object_faces_point_plain_list, 1, number_of_point_references, file );\n",
        replacement: "\t\teech_mf_filled ( 2, fread ( obj->object_faces_point_plain_list, 1, number_of_point_references, file ) ); /* EECH model fixture (M8) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M9-custom-surface-points-filled",
        why: "read_object: how many surface-point references fread returned",
        original: "\t\tfread ( obj->surface_points, 2, number_of_surface_point_references, file );\n",
        replacement: "\t\teech_mf_filled ( 3, fread ( obj->surface_points, 2, number_of_surface_point_references, file ) ); /* EECH model fixture (M9) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M12-custom-surfaces-filled",
        why: "read_object: how many 16-byte surface records fread returned (the allocation is 16 bytes per surface; a short read leaves later records uninitialised)",
        original: "\t\tfread ( obj->surfaces, 16, obj->number_of_surfaces, file );\n",
        replacement: "\t\teech_mf_filled ( 4, fread ( obj->surfaces, 16, obj->number_of_surfaces, file ) ); /* EECH model fixture (M12) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M13-common-surface-flags",
        why: "the object loop: the fread of a common surface record's flags word (bytes 0..3); a field counts only when fread returned 1",
        original: "\t\t\t\tfread ( &objects_3d_data[count].surfaces[surface_count].surface_flags, sizeof ( unsigned int ), 1, fp );\n",
        replacement: "\t\t\t\teech_mf_surface_field ( count, surface_count, 0, fread ( &objects_3d_data[count].surfaces[surface_count].surface_flags, sizeof ( unsigned int ), 1, fp ) ); /* EECH model fixture (M13) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M14-common-surface-texture",
        why: "as M13: texture_index (bytes 4..5)",
        original: "\t\t\t\tfread ( &objects_3d_data[count].surfaces[surface_count].texture_index, sizeof ( unsigned short int ), 1, fp );\n",
        replacement: "\t\t\t\teech_mf_surface_field ( count, surface_count, 1, fread ( &objects_3d_data[count].surfaces[surface_count].texture_index, sizeof ( unsigned short int ), 1, fp ) ); /* EECH model fixture (M14) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M15-common-surface-luminosity",
        why: "as M13: luminosity_texture_index (bytes 6..7)",
        original: "\t\t\t\tfread ( &objects_3d_data[count].surfaces[surface_count].luminosity_texture_index, sizeof ( unsigned short int ), 1, fp );\n",
        replacement: "\t\t\t\teech_mf_surface_field ( count, surface_count, 2, fread ( &objects_3d_data[count].surfaces[surface_count].luminosity_texture_index, sizeof ( unsigned short int ), 1, fp ) ); /* EECH model fixture (M15) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M16-common-surface-faces",
        why: "as M13: number_of_faces, read as an int into the 16-bit member at byte 8, so it writes bytes 8..11 (overlapping the reflectivity and specularity bytes)",
        original: "\t\t\t\tfread ( &objects_3d_data[count].surfaces[surface_count].number_of_faces, sizeof ( int ), 1, fp );\n",
        replacement: "\t\t\t\teech_mf_surface_field ( count, surface_count, 3, fread ( &objects_3d_data[count].surfaces[surface_count].number_of_faces, sizeof ( int ), 1, fp ) ); /* EECH model fixture (M16) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M17-common-surface-colour",
        why: "as M13: colour (bytes 12..15)",
        original: "\t\t\t\tfread ( &objects_3d_data[count].surfaces[surface_count].colour, sizeof ( unsigned int ), 1, fp );\n",
        replacement: "\t\t\t\teech_mf_surface_field ( count, surface_count, 4, fread ( &objects_3d_data[count].surfaces[surface_count].colour, sizeof ( unsigned int ), 1, fp ) ); /* EECH model fixture (M17) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M18-common-surface-reflectivity",
        why: "as M13: reflectivity (byte 10)",
        original: "\t\t\t\tfread ( &objects_3d_data[count].surfaces[surface_count].reflectivity, sizeof ( unsigned char ), 1, fp );\n",
        replacement: "\t\t\t\teech_mf_surface_field ( count, surface_count, 5, fread ( &objects_3d_data[count].surfaces[surface_count].reflectivity, sizeof ( unsigned char ), 1, fp ) ); /* EECH model fixture (M18) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M19-common-surface-specularity",
        why: "as M13: specularity (byte 11)",
        original: "\t\t\t\tfread ( &objects_3d_data[count].surfaces[surface_count].specularity, sizeof ( unsigned char ), 1, fp );\n",
        replacement: "\t\t\t\teech_mf_surface_field ( count, surface_count, 6, fread ( &objects_3d_data[count].surfaces[surface_count].specularity, sizeof ( unsigned char ), 1, fp ) ); /* EECH model fixture (M19) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M20-consolidate-begin",
        why: "the surface consolidation has allocated its block and not yet copied anything: the block and the record count it was allocated for",
        original: "\t\tcursurface = surfaces;\n",
        replacement: "\t\tcursurface = surfaces;\n\t\teech_mf_consolidate_begin ( surfaces, number_of_surfaces ); /* EECH model fixture (M20) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M21-consolidate-copy",
        why: "immediately before one object's surfaces are copied into the block: the object (its current pointer and count are the copy's source) and the destination",
        original: "\t\t\tmemcpy ( cursurface, obj->surfaces, sizeof ( face_surface_description ) * obj->number_of_surfaces );\n",
        replacement: "\t\t\teech_mf_consolidate_copy ( count, obj, cursurface ); /* EECH model fixture (M21) */\n\t\t\tmemcpy ( cursurface, obj->surfaces, sizeof ( face_surface_description ) * obj->number_of_surfaces );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M22-consolidate-end",
        why: "the new block is bound as object_database_surfaces: the block and the record count the loader stored",
        original: "\t\tobject_database_surfaces = surfaces;\n\t\ttotal_number_of_surfaces = number_of_surfaces;\n",
        replacement: "\t\tobject_database_surfaces = surfaces;\n\t\ttotal_number_of_surfaces = number_of_surfaces;\n\t\teech_mf_consolidate_end ( surfaces, number_of_surfaces ); /* EECH model fixture (M22) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M10-custom-recorded",
        why: "read_object has made the object custom, every allocation having succeeded (safe_malloc does not return on failure): record the extents it allocated by",
        original: "\tobj->custom = 1;\n",
        replacement: "\tobj->custom = 1;\n\teech_mf_custom ( obj, number_of_polygoned_faces, number_of_point_references, number_of_surface_point_references ); /* EECH model fixture (M10) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "M11-reset-on-deinitialise",
        why: "the objects are freed: every record is dropped",
        original: "void deinitialise_3d_objects ( void )\n{\n",
        replacement: "void deinitialise_3d_objects ( void )\n{\n\teech_mf_reset (); /* EECH model fixture (M11) */\n\n",
        count: 1,
    },
    // MA0..MA4: the scenes' texture animation lists (read_scene, add_animation). Additive records of what the original loader does; the wrapped fread passes its result on unchanged.
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "MA0-animation-list-declared",
        why: "read_scene has read the scene's declared texture animation count (and knows whether it was given a sceneid): a new record of the scene's list begins",
        original: "\tfread ( &number_of_texture_animations, sizeof ( int ), 1, fp );\n",
        replacement: "\tfread ( &number_of_texture_animations, sizeof ( int ), 1, fp );\n\teech_mf_anim_begin ( scene_index, sceneid, number_of_texture_animations ); /* EECH model fixture (MA0) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "MA1-animation-list-file-entry",
        why: "one fread of one entry of a declared list: what it returned is recorded and handed back unchanged to the statement (which ignores it, as the original did)",
        original: "\t\t\tfread ( &objects_3d_scene_database[scene_index].texture_animations[tmp], sizeof ( int ), 1, fp );\n",
        replacement: "\t\t\teech_mf_anim_file_read ( scene_index, tmp, number_of_texture_animations, objects_3d_scene_database[scene_index].texture_animations, fread ( &objects_3d_scene_database[scene_index].texture_animations[tmp], sizeof ( int ), 1, fp ) ); /* EECH model fixture (MA1) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "MA2-animation-list-allocated",
        why: "add_animation has allocated the grown array and bound it as the scene's list: its address and size are recorded",
        original: "\t\tscene->texture_animations = new_anim;\n",
        replacement: "\t\tscene->texture_animations = new_anim;\n\t\teech_mf_anim_alloc ( scene, new_anim, *allocated ); /* EECH model fixture (MA2) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "MA3-animation-list-appended",
        why: "add_animation has appended one entry: the scene's count is recorded",
        original: "\tscene->texture_animations[scene->number_of_texture_animations++] = animation;\n",
        replacement: "\tscene->texture_animations[scene->number_of_texture_animations++] = animation;\n\teech_mf_anim_append ( scene ); /* EECH model fixture (MA3) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "MA4-animation-list-final",
        why: "read_scene is finishing the scene: the list and count it leaves (and its failure flag) are recorded before the scene's success flag is stored",
        original: "\tobjects_3d_scene_database[scene_index].succeeded = !fail;\n",
        replacement: "\teech_mf_anim_done ( scene_index, fail ); /* EECH model fixture (MA4) */\n\tobjects_3d_scene_database[scene_index].succeeded = !fail;\n",
        count: 1,
    },
];

/// the staged text of `file` with the model-fixture patches applied (same strict count check as `apply`)
pub fn apply_model_fixture(file: &str, text: &str) -> String {
    let mut out = text.to_string();
    for p in MODEL_FIXTURE_PATCHES.iter().filter(|p| p.file == file) {
        let found = out.matches(p.original).count();
        assert_eq!(
            found, p.count,
            "model-fixture patch {} expects {} occurrence(s) of its original text in {}, found {}",
            p.id, p.count, file, found
        );
        out = out.replace(p.original, p.replacement);
    }
    out
}

pub fn model_fixture_files() -> Vec<&'static str> {
    let mut files: Vec<&str> = MODEL_FIXTURE_PATCHES.iter().map(|p| p.file).collect();
    files.dedup();
    files
}

pub fn apply(file: &str, text: &str) -> String {
    let mut out = text.to_string();
    for p in PATCHES.iter().filter(|p| p.file == file) {
        let found = out.matches(p.original).count();
        assert_eq!(
            found, p.count,
            "patch {} expects {} occurrence(s) of its original text in {}, found {}",
            p.id, p.count, file, found
        );
        out = out.replace(p.original, p.replacement);
    }
    out
}

pub fn patched_files() -> Vec<&'static str> {
    let mut files: Vec<&str> = PATCHES.iter().map(|p| p.file).collect();
    files.dedup();
    files
}

/// One ROUTE_HOST patch for a place where popread.c takes the legacy model number from the route entry it holds (`raw_object_index = <entry>->index;`): the number the parse
/// receives is now the entry's route key (campaign_model_route_handle in aphavoc/.../mb_hostmodel.c), so the parse names its route by the scene and kind the campaign's own
/// lookup selected and never by a legacy object number. The statement keeps its position, its evaluation count and its order; the extern declaration is block-local.
macro_rules! handle_patch {
    ($entry:literal, $count:expr) => {
        Patch {
            file: "aphavoc/source/ai/faction/popread.c",
            id: concat!("H3-route-handle-", $entry),
            why: "the parse receives the route entry's key (scene and kind) where it used to receive the legacy model number taken from the entry",
            original: concat!("raw_object_index = ", $entry, "->index;\n"),
            replacement: concat!(
                "{\n\t\t\t\textern int campaign_model_route_handle ( const void *entry ); /* EECH headless (D3) */\n\t\t\t\traw_object_index = campaign_model_route_handle ( ",
                $entry,
                " );\n\t\t\t}\n"
            ),
            count: $count,
        }
    };
}

/// Patches of the host-owned ROUTE data (EEDCS-50, D3), applied by main.rs unless the build keeps the original model accessors (feature model-fixture) or the original route
/// parser (feature route-original, the immutable original characterisation). Strict count-checked like the others. The route-planning statements of routegen.c (the tree, the
/// primary and start nodes, the end-slot matching) are NOT touched: only where the parser gets its nodes and line faces from changes, from the campaign DLL's own model
/// arrays to the host (campaign_model_route_object / campaign_model_route_position), and popread.c's lookup is told which entry belongs to which (scene, kind).
pub const ROUTE_HOST_PATCHES: &[Patch] = &[
    Patch {
        file: "aphavoc/source/ai/faction/popread.c",
        id: "H1-route-lookup-found",
        why: "a route entry was found for the airfield instance (scene index) and sub-object kind: the campaign records the entry as the key of that pair",
        original: "\t\treturn ( search.object_scene );\n",
        replacement: "\t\t{\n\t\t\textern void campaign_model_route_lookup ( int scene, int kind, const void *entry ); /* EECH headless (D3) */\n\t\t\tcampaign_model_route_lookup ( instance->object_number, ( int ) index, search.object_scene );\n\t\t}\n\t\treturn ( search.object_scene );\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/popread.c",
        id: "H2-route-lookup-missing",
        why: "no route entry exists for the airfield instance and sub-object kind: nothing to record (the fixture builds note the miss)",
        original: "\t\treturn ( NULL );\n#else\n\t\treturn 0;\n#endif\n",
        replacement: "\t\t{\n\t\t\textern void campaign_model_route_lookup ( int scene, int kind, const void *entry ); /* EECH headless (D3) */\n\t\t\tcampaign_model_route_lookup ( instance->object_number, ( int ) index, NULL );\n\t\t}\n\t\treturn ( NULL );\n#else\n\t\treturn 0;\n#endif\n",
        count: 1,
    },
    handle_patch!("fixedwing_landing_route", 1),
    handle_patch!("fixedwing_takeoff_route", 1),
    handle_patch!("fixedwing_landing_holding_route", 1),
    handle_patch!("fixedwing_takeoff_holding_route", 1),
    handle_patch!("helicopter_landing_route", 1),
    handle_patch!("helicopter_takeoff_route", 1),
    handle_patch!("helicopter_landing_holding_route", 1),
    handle_patch!("helicopter_takeoff_holding_route", 1),
    handle_patch!("vehicle_landing_route", 1),
    handle_patch!("vehicle_takeoff_route", 1),
    handle_patch!("vehicle_landing_holding_route", 1),
    handle_patch!("vehicle_takeoff_holding_route", 1),
    handle_patch!("ship_landing_route", 1),
    handle_patch!("ship_takeoff_route", 1),
    handle_patch!("ship_landing_holding_route", 1),
    handle_patch!("ship_takeoff_holding_route", 1),
    handle_patch!("landing_route", 3),
    handle_patch!("takeoff_route", 3),
    handle_patch!("landing_holding_route", 2),
    handle_patch!("takeoff_holding_route", 2),
    Patch {
        file: "aphavoc/source/ai/faction/popread.c",
        id: "H3-route-handle-troop-routes",
        why: "the troop landing and take-off routes are parsed straight from the entry the caller holds in `route`: `route` is read once, by the key lookup, whose result is the parser's argument",
        original: "parse_waypoint_routes_from_object ( route->index, 0, NULL );\n",
        replacement: "{\n\t\t\t\t\textern int campaign_model_route_handle ( const void *entry ); /* EECH headless (D3) */\n\t\t\t\t\tparse_waypoint_routes_from_object ( campaign_model_route_handle ( route ), 0, NULL );\n\t\t\t\t}\n",
        count: 2,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H4-route-object-from-host",
        why: "the parser's route object comes from the host (ordered nodes and line faces) instead of the campaign's own model arrays; the object is a packaging of the host's answers in the shape the unchanged loops walk",
        original: "\tobject = &objects_3d_data[object_index];\n",
        replacement: concat!(
            "#ifdef EECH_ROUTE_FIXTURE\n",
            "\t{\n",
            "\t\textern void eech_rf_begin_host ( int object_index, int number_of_slots, const void *slots ); /* EECH route fixture */\n",
            "\t\teech_rf_begin_host ( object_index, number_of_matching_slots, slots );\n",
            "\t}\n",
            "#endif\n",
            "\t{\n",
            "\t\textern struct OBJECT_3D *campaign_model_route_object ( int handle ); /* EECH headless (D3) */\n",
            "\t\tobject = campaign_model_route_object ( object_index );\n",
            "\t}\n"
        ),
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H5-route-node-from-host",
        why: "each node's position is the host's decode (the stored short scaled by the object's bounds, in float, under the original's rounding), not a decode here",
        original: "\t\t\twaypoint_nodes[count].position.x = object->points[count].x;\n\t\t\twaypoint_nodes[count].position.y = object->points[count].y;\n\t\t\twaypoint_nodes[count].position.z = object->points[count].z;\n\n\t\t\twaypoint_nodes[count].position.x *= xmax / 32767;\n\t\t\twaypoint_nodes[count].position.y *= ymax / 32767;\n\t\t\twaypoint_nodes[count].position.z *= zmax / 32767;\n",
        replacement: "\t\t\t{\n\t\t\t\textern void campaign_model_route_position ( int node, vec3d *position ); /* EECH headless (D3) */\n\t\t\t\tcampaign_model_route_position ( count, &waypoint_nodes[count].position );\n\t\t\t}\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H5b-route-bounds-setup-removed",
        why: "H5 takes each node's position from the host, so the bounds scale the parser used to decode a stored short with (xmax, ymax, zmax) has no reader left in the hosted parser: its declaration and computation are removed. The original parser (feature route-original) keeps them",
        original: "\t\tfloat\n\t\t\txmax,\n\t\t\tymax,\n\t\t\tzmax;\n\n\t\txmax = max ( fabs ( object->bounding_box.xmin ), fabs ( object->bounding_box.xmax ) );\n\t\tymax = max ( fabs ( object->bounding_box.ymin ), fabs ( object->bounding_box.ymax ) );\n\t\tzmax = max ( fabs ( object->bounding_box.zmin ), fabs ( object->bounding_box.zmax ) );\n\n",
        replacement: "",
        count: 1,
    },
    // H6..H15: the bounded-input guards of the host route boundary (mb_hostmodel.c, "The bounded-input contract of the unchanged route-planning code"). Each ADDS one call (or a
    // test in a loop condition) at a place where the unchanged parser would write past a table, read a start that is not there, or go round again with nothing changed; every
    // original statement, its order and its result for a route the parser can take stay as they are, and the original parser (feature route-original) has none of them.
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H6-route-guards-declared",
        why: "declares the guard functions of the host route boundary, once, at file scope",
        original: "int get_number_of_nodes_at_depth ( int tree_depth );\n",
        replacement: concat!(
            "/* the topology guards of the host route boundary (mb_hostmodel.c; EECH headless, D3) */\n",
            "void campaign_model_route_room ( int what, int used, int capacity );\n",
            "void campaign_model_route_progress_begin ( void );\n",
            "int campaign_model_route_progress ( int phase, int left );\n",
            "int campaign_model_route_match_pass ( void );\n",
            "void campaign_model_route_starts ( int starts );\n",
            "void campaign_model_route_primary_start ( int index, int starts );\n",
            "void campaign_model_route_swap_index ( int what, int index, int count, int capacity );\n",
            "\n",
            "int get_number_of_nodes_at_depth ( int tree_depth );\n"
        ),
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H7-route-start-capacity",
        why: "the 24-entry start table: the write of the next start is refused with a named marker when the table is full (the original has only an ASSERT); the check comes BEFORE the ASSERT so an assertion-enabled build reports the named marker, not a generic assertion",
        original: "ASSERT ( number_of_waypoint_starting_nodes < MAX_WAYPOINT_STARTS );\n",
        replacement: "campaign_model_route_room ( 0, number_of_waypoint_starting_nodes, MAX_WAYPOINT_STARTS );\n\n\t\t\tASSERT ( number_of_waypoint_starting_nodes < MAX_WAYPOINT_STARTS );\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H8-route-link-capacity",
        why: "the 24-entry child and parent tables of a node: the write of the next link (in the primary-node pass and in the all-node pass) is refused with a named marker when a table is full (the original has no check)",
        original: "number_of_parents = waypoint_nodes[child].number_of_parents;\n",
        replacement: concat!(
            "number_of_parents = waypoint_nodes[child].number_of_parents;\n",
            "\t\t\t\t\tcampaign_model_route_room ( 1, number_of_children, MAX_WAYPOINT_LINKS );\n",
            "\t\t\t\t\tcampaign_model_route_room ( 2, number_of_parents, MAX_WAYPOINT_LINKS );\n"
        ),
        count: 2,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H9-route-primary-pass-progress",
        why: "the primary-node pass walks the same line faces over the same nodes each time round: a pass that placed no primary node would be repeated for ever, so it is refused with a named marker",
        original: "while ( number_of_primary_nodes_left )\n",
        replacement: "while ( ( number_of_primary_nodes_left ) && ( campaign_model_route_progress ( 1, number_of_primary_nodes_left ) ) )\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H10-route-remaining-pass-progress",
        why: "the all-node pass: a pass that placed no node is refused with a named marker instead of repeating for ever (a node no start reaches)",
        original: "while ( number_of_nodes_left )\n\t{\n\n\t\tsurface_point_refs = object->surface_points;\n",
        replacement: "while ( ( number_of_nodes_left ) && ( campaign_model_route_progress ( 2, number_of_nodes_left ) ) )\n\t{\n\n\t\tsurface_point_refs = object->surface_points;\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H11-route-node-index-progress",
        why: "set_node_indices: a pass that gave no node its index is refused with a named marker instead of repeating for ever (defence in depth: the trees built before it make this unreachable). The state belongs to one invocation: H11b starts every invocation afresh",
        original: "while ( number_of_nodes_left )\n\t{\n\t\n\t\tfor ( count = 0; count < number_of_waypoint_nodes; count++ )\n",
        replacement: "while ( ( number_of_nodes_left ) && ( campaign_model_route_progress ( 3, number_of_nodes_left ) ) )\n\t{\n\t\n\t\tfor ( count = 0; count < number_of_waypoint_nodes; count++ )\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H11b-route-node-index-invocation",
        why: "set_node_indices runs once from the parse and again from every end-slot swap (after the swap invalidated the indices): each invocation begins its own progress count, so a later invocation that legitimately starts with as many nodes unnumbered as the last one ended with is never compared with it",
        original: "\t//\n\t// Count the number of nodes that haven't been given a valid node index\n\t//\n",
        replacement: "\tcampaign_model_route_progress_begin ();\n\n\t//\n\t// Count the number of nodes that haven't been given a valid node index\n\t//\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H12-route-position-pass-progress",
        why: "the per-depth position table: the search for the node holding the next index at that depth is refused with a named marker when a pass finds none (node indices that are not 0..n-1 at a depth), instead of repeating for ever; each depth is its own phase",
        original: "while ( number_of_nodes )\n",
        replacement: "while ( ( number_of_nodes ) && ( campaign_model_route_progress ( 8 + current_depth, number_of_nodes ) ) )\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H13-route-end-slot-match-passes",
        why: "the end-slot matching repeats while it swapped anything: swaps that undo each other would repeat for ever, so the passes are bounded and the bound is a named marker",
        original: "while ( !finished )\n",
        replacement: "while ( ( !finished ) && ( campaign_model_route_match_pass () ) )\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H14-route-start-exists",
        why: "the parser reads waypoint_starting_nodes[0] without asking whether the route has a start; a route with none is refused with a named marker",
        original: "if ( !waypoint_starting_nodes[0]->primary )\n",
        replacement: "campaign_model_route_starts ( number_of_waypoint_starting_nodes );\n\n\tif ( !waypoint_starting_nodes[0]->primary )\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H15-route-primary-start-index",
        why: "primary_start_index is only assigned when a later start is primary and is then used as an index; it starts at 0 (no later start found) and a route with none is refused with a named marker before the index is used",
        original: "\t\t\tprimary_start_index;\n",
        replacement: "\t\t\tprimary_start_index = 0;\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H15b-route-primary-start-checked",
        why: "the check of the primary start index, immediately before the swap that uses it",
        original: "tmp = waypoint_starting_nodes[0];\n",
        replacement: "campaign_model_route_primary_start ( primary_start_index, number_of_waypoint_starting_nodes );\n\n\t\ttmp = waypoint_starting_nodes[0];\n",
        count: 1,
    },
    // H16..H18: the end-slot matching's swap (swap_waypoint_nodes) resolves table indices by its own searches and then writes through them. The tree swap searches the start table from
    // entry 1, so a swap that involves the primary tree (entry 0) resolves to the logical count, and the original only has an ASSERT there; the children of one parent are searched with
    // two uninitialised indices. The checks are against the LOGICAL counts and the physical capacities, before the ASSERT and before either write, with the named marker route.swap.
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H16-route-swap-child-indices-initialised",
        why: "the two child indices of a same-parent swap are only assigned when the search finds the child; they start at -1 (not found) so that H17 can refuse an index the search did not resolve",
        original: "\t\tchild_index,\n\t\tother_child_index;\n",
        replacement: "\t\tchild_index = -1,\n\t\tother_child_index = -1;\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H17-route-swap-child-indices-checked",
        why: "a same-parent swap writes the parent's child table through two searched indices: each must be a resolved entry of the parent's logical child count (and of the 24-entry table) before the first write",
        original: "common_parent->children[child_index] = other_child;\n",
        replacement: concat!(
            "campaign_model_route_swap_index ( 1, child_index, common_parent->number_of_children, MAX_WAYPOINT_LINKS );\n",
            "\t\tcampaign_model_route_swap_index ( 1, other_child_index, common_parent->number_of_children, MAX_WAYPOINT_LINKS );\n",
            "\n",
            "\t\tcommon_parent->children[child_index] = other_child;\n"
        ),
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "H18-route-swap-root-indices-checked",
        why: "a tree swap writes the start table through two searched indices (the search begins at entry 1): each must be a resolved entry 1 .. logical count - 1 (and inside the 24-entry table) before the ASSERT that only the original has and before either write",
        original: "ASSERT ( child_index < number_of_waypoint_starting_nodes );\n",
        replacement: concat!(
            "campaign_model_route_swap_index ( 0, child_index, number_of_waypoint_starting_nodes, MAX_WAYPOINT_STARTS );\n",
            "\t\tcampaign_model_route_swap_index ( 0, other_child_index, number_of_waypoint_starting_nodes, MAX_WAYPOINT_STARTS );\n",
            "\n",
            "\t\tASSERT ( child_index < number_of_waypoint_starting_nodes );\n"
        ),
        count: 1,
    },
    // H19/H20: popread.c's CONSUMERS of the parser's per-depth position table index it with fixed steps from the deepest depth (current_route_waypoint_offset = depth count - 1, then
    // -1, -2 and so on to fixed floors), which a route a host answers need not support (a three-node black/green chain has three depths; fixed-wing approach reads -1). Every `route_waypoint_positions[
    // current_route_waypoint_offset]` of popread.c, and so the WAYPOINT_ALTITUDE macro that expands to one, indexes through a check that returns the same index. The six
    // `route_waypoint_positions[count]` copies (troop routes) are bounded by the table's own count and are not wrapped; the parser's own table writes are in routegen.c, not here.
    Patch {
        file: "aphavoc/source/ai/faction/popread.c",
        id: "H19-route-position-index-declared",
        why: "declares the check of an index into the parser's per-depth position table, at file scope before its first use (the WAYPOINT_ALTITUDE macro is the first place the pattern appears; a macro is expanded where it is used)",
        original: "#define WAYPOINT_ALTITUDE route_waypoint_positions[current_route_waypoint_offset].position.y\n",
        replacement: concat!(
            "/* the host route boundary's check of an index into the parser's per-depth position table (mb_hostmodel.c; EECH headless, D3) */\n",
            "int campaign_model_route_position_index ( int index, int count, const void *table );\n",
            "\n",
            "#define WAYPOINT_ALTITUDE route_waypoint_positions[current_route_waypoint_offset].position.y\n"
        ),
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/popread.c",
        id: "H20-route-position-index-checked",
        why: "each popread read of the per-depth position table (205 indexed expressions of popread.c, one of them inside the WAYPOINT_ALTITUDE macro) takes its index through a check that returns it unchanged, or stops with route.position, before the table is touched",
        original: "route_waypoint_positions[current_route_waypoint_offset]",
        replacement: "route_waypoint_positions[campaign_model_route_position_index ( current_route_waypoint_offset, number_of_route_waypoint_positions, route_waypoint_positions )]",
        count: 205,
    },
];

/// the staged text of `file` with the host-route patches applied (same strict count check as `apply`)
pub fn apply_route_host(file: &str, text: &str) -> String {
    let mut out = text.to_string();
    for p in ROUTE_HOST_PATCHES.iter().filter(|p| p.file == file) {
        let found = out.matches(p.original).count();
        assert_eq!(
            found, p.count,
            "route-host patch {} expects {} occurrence(s) of its original text in {}, found {}",
            p.id, p.count, file, found
        );
        out = out.replace(p.original, p.replacement);
    }
    out
}

pub fn route_host_files() -> Vec<&'static str> {
    let mut files: Vec<&str> = ROUTE_HOST_PATCHES.iter().map(|p| p.file).collect();
    files.sort_unstable();
    files.dedup();
    files
}

/// The loader detachment (EEDCS-50, D3), applied by main.rs in every build that has a host model AND the host route parser (i.e. not for feature model-fixture, which keeps the
/// original accessors, nor for feature route-original, whose parser reads the campaign's own arrays): the common (`3dobjs.*`) and custom (`.EEO`) loaders of modules/3d/3dobjdb.c
/// keep no point, polygon face, face-point-reference or surface-point-reference buffer. The streams are skipped by their exact byte counts (the custom loader's polygon faces are
/// streamed through a small local buffer for the original validation and kept nowhere), so every later read stays aligned and every count, bound, radius, surface descriptor,
/// normal and texture point is the file's true value. The campaign's readers of those buffers are the host's (the collision triangle, the route data); the horizon image reads its
/// pre-processed `.bin` (point colours and fog, never points) and stops by name when it would generate an uncached image; every legacy visual / display reader (renderer, shadow,
/// explosion, cloud, cockpit poly-lines, the Mi-24 HUD) stops by name at its head (campaign_model_geometry_reader). Same strict count check as the other sets.
pub const LOADER_HOST_PATCHES: &[Patch] = &[
    // ---- the common loader (initialise_3d_objects): 3dobjs.pts and 3dobjs.sp are not mapped, the face point list is neither allocated nor read into memory
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LC1-points-file-not-mapped",
        why: "3dobjs.pts (the stored points of every object) is not mapped: the host owns the points",
        original: "object_database_points = ( struct OBJECT_SHORT_3D_POINT * ) safe_mopen ( filename );\n",
        replacement: "object_database_points = NULL; /* EECH headless (D3 loader): the point file is not mapped */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LC2-surface-point-file-not-mapped",
        why: "3dobjs.sp (the surface-point references of every object) is not mapped: the host owns them",
        original: "object_database_surface_point_list = ( struct POINT_3D_PLAIN_REFERENCE * ) safe_mopen ( filename );\n",
        replacement: "object_database_surface_point_list = NULL; /* EECH headless (D3 loader): the surface-point file is not mapped */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LC3-face-point-list-not-allocated",
        why: "the database-wide face point list (the face-point-reference buffer of every object) is not allocated",
        original: "object_database_faces_point_list = ( struct POINT_3D_SHORT_REFERENCE * ) safe_malloc ( sizeof ( point_3d_short_reference ) * total_number_of_plain_points );\n",
        replacement: "object_database_faces_point_list = NULL; /* EECH headless (D3 loader): the face-point-reference buffer is not allocated */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LC4-object-points-not-set",
        why: "an object's points pointer into the (unmapped) point file is not set: the row keeps NULL and its true point count",
        original: "objects_3d_data[count].points = current_points;\n",
        replacement: "objects_3d_data[count].points = NULL; /* EECH headless (D3 loader) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LC5-point-cursor-not-advanced",
        why: "there is no point file to advance a cursor through",
        original: "current_points += objects_3d_data[count].number_of_points;\n",
        replacement: "/* EECH headless (D3 loader): the point file is not mapped; no cursor to advance */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LC6-object-surface-points-not-set",
        why: "an object's surface-point-reference pointer into the (unmapped) surface-point file is not set: the row keeps NULL",
        original: "objects_3d_data[count].surface_points = current_object_surface_point_references;\n",
        replacement: "objects_3d_data[count].surface_points = NULL; /* EECH headless (D3 loader) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LC7-surface-point-cursor-not-advanced",
        why: "there is no surface-point file to advance a cursor through (the count is still read from the stream, so the stream stays aligned)",
        original: "current_object_surface_point_references += number_of_point_references;\n",
        replacement: "/* EECH headless (D3 loader): the surface-point file is not mapped; no cursor to advance */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LC8-face-point-references-skipped",
        why: "an object's face point references are in the stream between its faces and its face normals: they are skipped by their exact byte count (point_reference_count entries of the reference record), not read into memory",
        original: "fread ( current_faces_point_list, sizeof ( struct POINT_3D_SHORT_REFERENCE ), point_reference_count, fp );\n",
        replacement: concat!(
            "extern void campaign_model_loader_skip ( FILE *file, long long count, unsigned long long element, const char *what ); /* EECH headless (D3 loader) */\n",
            "\t\t\t\tcampaign_model_loader_skip ( fp, point_reference_count, sizeof ( struct POINT_3D_SHORT_REFERENCE ), \"a common object's face point references\" );\n"
        ),
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LC9-object-face-point-references-not-set",
        why: "an object's face-point-reference pointer is not set: the row keeps NULL",
        original: "objects_3d_data[count].object_faces_point_plain_list = current_faces_point_list;\n",
        replacement: "objects_3d_data[count].object_faces_point_plain_list = NULL; /* EECH headless (D3 loader) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LC10-face-point-cursor-not-advanced",
        why: "there is no face point list to advance a cursor through",
        original: "current_faces_point_list += point_reference_count;\n",
        replacement: "/* EECH headless (D3 loader): the face point list is not allocated; no cursor to advance */\n",
        count: 1,
    },
    // ---- the custom loader (read_object, one .EEO): the same three streams are skipped by their exact byte counts; the polygon faces are still read and validated
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LE1-custom-points-skipped",
        why: "an .EEO's points (6 bytes each) are skipped, not allocated and read: the object keeps NULL and its true point count",
        original: "\tif ( obj->number_of_points )  // read all points\n\t{\n\t\tobj->points = ( struct OBJECT_SHORT_3D_POINT * ) safe_malloc ( 6 * obj->number_of_points );\n\t\tfread ( obj->points, 6, obj->number_of_points, file );\n\t}\n\telse\n\t\tobj->points = NULL;\n",
        replacement: concat!(
            "\tobj->points = NULL; /* EECH headless (D3 loader): the host owns the points; their bytes are read past, checked, and kept nowhere */\n",
            "\t{\n",
            "\t\textern void campaign_model_loader_skip ( FILE *file, long long count, unsigned long long element, const char *what ); /* EECH headless (D3 loader) */\n",
            "\n",
            "\t\tcampaign_model_loader_skip ( file, obj->number_of_points, 6, \"an EEO's points\" );\n",
            "\t}\n"
        ),
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LE2-custom-face-point-references-skipped",
        why: "an .EEO's face point references (1 byte each) are skipped, not allocated and read",
        original: "\tif ( number_of_point_references )  // the points of all faces in same order as faces\n\t{\n\t\tobj->object_faces_point_plain_list = ( struct POINT_3D_SHORT_REFERENCE * ) safe_malloc ( 1 * number_of_point_references );\n\t\tfread ( obj->object_faces_point_plain_list, 1, number_of_point_references, file );\n\t}\n\telse\n\t\tobj->object_faces_point_plain_list = NULL;\n",
        replacement: concat!(
            "\tobj->object_faces_point_plain_list = NULL; /* EECH headless (D3 loader): the host owns the face point references; their bytes are read past, checked, and kept nowhere */\n",
            "\t{\n",
            "\t\textern void campaign_model_loader_skip ( FILE *file, long long count, unsigned long long element, const char *what ); /* EECH headless (D3 loader) */\n",
            "\n",
            "\t\tcampaign_model_loader_skip ( file, number_of_point_references, 1, \"an EEO's face point references\" );\n",
            "\t}\n"
        ),
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LE3-custom-surface-points-skipped",
        why: "an .EEO's surface-point references (2 bytes each) are skipped, not allocated and read",
        original: "\tif ( number_of_surface_point_references )\n\t{\n\t\tobj->surface_points = ( struct POINT_3D_PLAIN_REFERENCE * ) safe_malloc ( 2 * number_of_surface_point_references );\n\t\tfread ( obj->surface_points, 2, number_of_surface_point_references, file );\n\t}\n\telse\n\t\tobj->surface_points = NULL;\n",
        replacement: concat!(
            "\tobj->surface_points = NULL; /* EECH headless (D3 loader): the host owns the surface-point references; their bytes are read past, checked, and kept nowhere */\n",
            "\t{\n",
            "\t\textern void campaign_model_loader_skip ( FILE *file, long long count, unsigned long long element, const char *what ); /* EECH headless (D3 loader) */\n",
            "\n",
            "\t\tcampaign_model_loader_skip ( file, number_of_surface_point_references, 2, \"an EEO's surface point references\" );\n",
            "\t}\n"
        ),
        count: 1,
    },
    // ---- the fourth buffer, the polygon faces (one polygon point count per polygon face): the common loader neither allocates nor reads it into memory, the custom loader streams it
    //      through a small local buffer for the original validation and keeps nothing
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LC11-face-buffer-not-allocated",
        why: "the database-wide polygon face buffer is not allocated",
        original: "object_database_faces = ( struct OBJECT_3D_FACE * ) safe_malloc ( sizeof ( object_3d_face ) * total_number_of_polygons );\n",
        replacement: "object_database_faces = NULL; /* EECH headless (D3 loader): the polygon face buffer is not allocated */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LC12-faces-skipped",
        why: "an object's polygon faces are in the stream between its point normals and its face point references: they are skipped by their exact byte count (number_of_real_polygons records), not read into memory",
        original: "fread ( current_faces, sizeof ( struct OBJECT_3D_FACE ), number_of_real_polygons, fp );\n",
        replacement: concat!(
            "extern void campaign_model_loader_skip ( FILE *file, long long count, unsigned long long element, const char *what ); /* EECH headless (D3 loader) */\n",
            "\t\t\t\tcampaign_model_loader_skip ( fp, number_of_real_polygons, sizeof ( struct OBJECT_3D_FACE ), \"a common object's polygon faces\" );\n"
        ),
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LC13-object-faces-not-set",
        why: "an object's faces pointer into the (unallocated) face buffer is not set: the row keeps NULL",
        original: "objects_3d_data[count].faces = current_faces;\n",
        replacement: "objects_3d_data[count].faces = NULL; /* EECH headless (D3 loader) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LC14-face-cursor-not-advanced",
        why: "there is no face buffer to advance a cursor through",
        original: "current_faces += number_of_real_polygons;\n",
        replacement: "/* EECH headless (D3 loader): the face buffer is not allocated; no cursor to advance */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LE4-custom-faces-streamed-for-validation",
        why: "an .EEO's polygon faces (1 byte each) are read through a local buffer, in chunks, only to find whether any has fewer than 3 points; the object keeps NULL. The result is acted on where the original acted on it, after the texture overrides",
        original: "\tif ( number_of_polygoned_faces )  // number of points in each poly\n\t{\n\t\tobj->faces = ( struct OBJECT_3D_FACE * ) safe_malloc ( 1 * number_of_polygoned_faces );\n\t\tfread ( obj->faces, 1, number_of_polygoned_faces, file );\n\t}\n\telse\n\t\tobj->faces = NULL;\n",
        replacement: concat!(
            "\tint eech_polygon_face_invalid = 0; /* EECH headless (D3 loader): a polygon face with fewer than 3 points was streamed past */\n",
            "\n",
            "\tobj->faces = NULL;\n",
            "\t{\n",
            "\t\textern void campaign_model_loader_count ( const char *what, int count ); /* EECH headless (D3 loader) */\n",
            "\n",
            "\t\tcampaign_model_loader_count ( \"an EEO's polygon face count\", number_of_polygoned_faces );\n",
            "\t}\n",
            "\tif ( number_of_polygoned_faces )  // number of points in each poly: streamed for the validation, not retained\n",
            "\t{\n",
            "\t\tstruct OBJECT_3D_FACE face_buffer[256];\n",
            "\t\tint remaining = number_of_polygoned_faces;\n",
            "\n",
            "\t\twhile ( remaining > 0 )\n",
            "\t\t{\n",
            "\t\t\tint chunk = ( remaining < 256 ) ? remaining : 256;\n",
            "\t\t\tint got = ( int ) fread ( face_buffer, 1, chunk, file );\n",
            "\t\t\tint k;\n",
            "\n",
            "\t\t\tif ( got != chunk )\n",
            "\t\t\t{\n",
            "\t\t\t\textern void campaign_model_loader_short ( const char *what ); /* EECH headless (D3 loader) */\n",
            "\t\t\t\tcampaign_model_loader_short ( \"an EEO's polygon faces\" );\n",
            "\t\t\t}\n",
            "\n",
            "\t\t\tfor ( k = 0; k < got; k++ )\n",
            "\t\t\t{\n",
            "\t\t\t\tif ( face_buffer[k].number_of_points < 3 )\n",
            "\t\t\t\t{\n",
            "\t\t\t\t\teech_polygon_face_invalid = 1;\n",
            "\t\t\t\t}\n",
            "\t\t\t}\n",
            "\n",
            "\t\t\tremaining -= chunk;\n",
            "\t\t}\n",
            "\t}\n"
        ),
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LE5-custom-face-validation-from-stream",
        why: "the original polygon-face validation (after the texture overrides, returning FALSE) reads the streamed result instead of the retained faces; its loop, its position and its DEBUG fatal are unchanged",
        original: "if ( obj->faces[i].number_of_points < 3 )\n",
        replacement: "if ( eech_polygon_face_invalid )\n",
        count: 1,
    },
    // ---- the horizon image: the cached branch is untouched; an UNCACHED image would decode the object's stored points and read a TGA, which the hosted engine does not support
    Patch {
        file: "modules/3d/3dhoriz.c",
        id: "LU1-uncached-horizon-image-refused",
        why: "add_3d_horizon_image takes the pre-processed `.bin` branch (point colours and fog only, no point read) when its point count matches the object's true count; only otherwise does it read a TGA and decode the object's stored points. That uncached generation is a legacy visual path the hosted engine does not support: it stops by name at the start of the branch, before any image I/O or geometry read. The fixture build counts the branch taken, before the stop",
        original: "\tif ( !binary_loaded )\n\t{\n\n\t\tfloat\n\t\t\txmax,\n",
        replacement: concat!(
            "#ifdef EECH_MODEL_HOST_FIXTURE\n",
            "\t{\n",
            "\t\textern void campaign_model_horizon_branch ( int binary_loaded ); /* EECH headless (D3 loader), fixture only */\n",
            "\t\tcampaign_model_horizon_branch ( binary_loaded );\n",
            "\t}\n",
            "#endif\n",
            "\n",
            "\tif ( !binary_loaded )\n",
            "\t{\n",
            "\t\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n",
            "\t\tcampaign_model_geometry_reader ( \"add_3d_horizon_image (an uncached horizon image: no pre-processed .bin of the object's point count for this image)\" );\n",
            "\n",
            "\t\tfloat\n",
            "\t\t\txmax,\n"
        ),
        count: 1,
    },
    // ---- every legacy visual / display entry that reads one of the four buffers or dominates a reader of them (scene, draw, sub-object, translucent, transform, illumination, shadow,
    //      explosion, cloud, horizon, cockpit poly-lines, Mi-24 HUD): a named stop at its head (campaign_model_geometry_reader), before any metadata, buffer, image or device work
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LR-reader-debug_object",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "static void debug_object(const char* filename, int objid)\n{\n",
        replacement: "static void debug_object(const char* filename, int objid)\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"debug_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LR-reader-get_object_3d_poly_line_data",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void get_object_3d_poly_line_data ( object_3d_index_numbers index, vec3d *return_points )\n{\n",
        replacement: "void get_object_3d_poly_line_data ( object_3d_index_numbers index, vec3d *return_points )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"get_object_3d_poly_line_data\" );\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/gunships/avionics/hind/hi_hud.c",
        id: "LR-reader-initialise_hind_hud",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void initialise_hind_hud (void)\n{\n",
        replacement: "void initialise_hind_hud (void)\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"initialise_hind_hud\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dhoriz.c",
        id: "LR-reader-draw_3d_horizon",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_horizon ( void )\n{\n",
        replacement: "void draw_3d_horizon ( void )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_horizon\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dhoriz.c",
        id: "LR-reader-transform_3d_horizon",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void transform_3d_horizon ( object_3d *object )\n{\n",
        replacement: "void transform_3d_horizon ( object_3d *object )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"transform_3d_horizon\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dhoriz.c",
        id: "LR-reader-render_3d_horizon_face",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "static void render_3d_horizon_face ( object_3d_face *this_face, int surface_index, int reference_offset, struct OBJECT_3D_INFO *object_base, int apply_filter, rgb_colour colour_filter)\n{\n",
        replacement: "static void render_3d_horizon_face ( object_3d_face *this_face, int surface_index, int reference_offset, struct OBJECT_3D_INFO *object_base, int apply_filter, rgb_colour colour_filter)\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"render_3d_horizon_face\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dhoriz.c",
        id: "LR-reader-render_3d_horizon_infrared_face",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "static void render_3d_horizon_infrared_face ( object_3d_face *this_face, int surface_index, int reference_offset, struct OBJECT_3D_INFO *object_base )\n{\n",
        replacement: "static void render_3d_horizon_infrared_face ( object_3d_face *this_face, int surface_index, int reference_offset, struct OBJECT_3D_INFO *object_base )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"render_3d_horizon_infrared_face\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dexplos.c",
        id: "LR-reader-transform_3d_explosion_object",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void transform_3d_explosion_object ( object_3d *object, vec3d *pos, displacement_map *displacement, explosion_object_points_lookup *points_lookup, float amplitude, vec3d *camera_direction )\n{\n",
        replacement: "void transform_3d_explosion_object ( object_3d *object, vec3d *pos, displacement_map *displacement, explosion_object_points_lookup *points_lookup, float amplitude, vec3d *camera_direction )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"transform_3d_explosion_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dexplos.c",
        id: "LR-reader-draw_3d_explosion_clipped_faces",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_explosion_clipped_faces ( int object_number, vec3d *pos, light_3d_source *lights, object_3d_info *this_object_3d_info )\n{\n",
        replacement: "void draw_3d_explosion_clipped_faces ( int object_number, vec3d *pos, light_3d_source *lights, object_3d_info *this_object_3d_info )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_explosion_clipped_faces\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dexplos.c",
        id: "LR-reader-draw_3d_explosion_unclipped_faces",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_explosion_unclipped_faces ( int object_number, vec3d *pos, light_3d_source *lights, object_3d_info *this_object_3d_info )\n{\n",
        replacement: "void draw_3d_explosion_unclipped_faces ( int object_number, vec3d *pos, light_3d_source *lights, object_3d_info *this_object_3d_info )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_explosion_unclipped_faces\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dexplos.c",
        id: "LR-reader-generate_explosion_object_texture_points",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "explosion_object_points_lookup * generate_explosion_object_texture_points ( int object_number, displacement_map *displacement )\n{\n",
        replacement: "explosion_object_points_lookup * generate_explosion_object_texture_points ( int object_number, displacement_map *displacement )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"generate_explosion_object_texture_points\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dfaces.c",
        id: "LR-reader-render_3d_object_face",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void render_3d_object_face ( object_3d_face_list *face_order )\n{\n",
        replacement: "void render_3d_object_face ( object_3d_face_list *face_order )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"render_3d_object_face\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobject.c",
        id: "LR-reader-calculate_line_colour",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "real_colour calculate_line_colour ( struct OBJECT_3D_INFO *object_base )\n{\n",
        replacement: "real_colour calculate_line_colour ( struct OBJECT_3D_INFO *object_base )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"calculate_line_colour\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobject.c",
        id: "LR-reader-calculate_line_normal",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void calculate_line_normal ( struct OBJECT_3D_INFO *object_base, vec3d *result_normal )\n{\n",
        replacement: "void calculate_line_normal ( struct OBJECT_3D_INFO *object_base, vec3d *result_normal )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"calculate_line_normal\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjvb.c",
        id: "LR-reader-initialise_3d_objects_in_d3d_old",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void initialise_3d_objects_in_d3d_old ( void )\n{\n",
        replacement: "void initialise_3d_objects_in_d3d_old ( void )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"initialise_3d_objects_in_d3d_old\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjvb.c",
        id: "LR-reader-object_3d_render_hardware_surface",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void object_3d_render_hardware_surface ( object_3d *object )\n{\n",
        replacement: "void object_3d_render_hardware_surface ( object_3d *object )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"object_3d_render_hardware_surface\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dscon.c",
        id: "LR-reader-construct_3d_surface_reflective_triangle_fan",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "vertex * construct_3d_surface_reflective_triangle_fan ( int triangle_index, int base_point, int *polygon_outcode, vec3d *normal )\n{\n",
        replacement: "vertex * construct_3d_surface_reflective_triangle_fan ( int triangle_index, int base_point, int *polygon_outcode, vec3d *normal )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"construct_3d_surface_reflective_triangle_fan\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dscon.c",
        id: "LR-reader-construct_3d_surface_gouraud_reflective_triangle_fan",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "vertex * construct_3d_surface_gouraud_reflective_triangle_fan ( int triangle_index, int base_point, int *polygon_outcode )\n{\n",
        replacement: "vertex * construct_3d_surface_gouraud_reflective_triangle_fan ( int triangle_index, int base_point, int *polygon_outcode )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"construct_3d_surface_gouraud_reflective_triangle_fan\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dshadow.c",
        id: "LR-reader-draw_3d_object_shadow",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_object_shadow ( object_3d_instance *obj )\n{\n",
        replacement: "void draw_3d_object_shadow ( object_3d_instance *obj )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_object_shadow\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dshadow.c",
        id: "LR-reader-transform_3d_object_shadow",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void transform_3d_object_shadow ( object_3d *object, vec3d *object_3d_scale, viewpoint *vp, vec3d *pos, vec3d *terrain_normal, vec3d *projection_direction, double projection_constant, double projection_factor, int points_base )\n{\n",
        replacement: "void transform_3d_object_shadow ( object_3d *object, vec3d *object_3d_scale, viewpoint *vp, vec3d *pos, vec3d *terrain_normal, vec3d *projection_direction, double projection_constant, double projection_factor, int points_base )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"transform_3d_object_shadow\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dwface1.c",
        id: "LR-reader-wrender_clipped_3d_object_polygon",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void wrender_clipped_3d_object_polygon ( object_3d_face *this_face )\n{\n",
        replacement: "void wrender_clipped_3d_object_polygon ( object_3d_face *this_face )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"wrender_clipped_3d_object_polygon\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dwface1.c",
        id: "LR-reader-wrender_3d_object_luminous_surface",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void wrender_3d_object_luminous_surface ( int number_of_points )\n{\n",
        replacement: "void wrender_3d_object_luminous_surface ( int number_of_points )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"wrender_3d_object_luminous_surface\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dwface2.c",
        id: "LR-reader-wrender_unclipped_3d_object_textured_polygon",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void wrender_unclipped_3d_object_textured_polygon ( object_3d_face *this_face )\n{\n",
        replacement: "void wrender_unclipped_3d_object_textured_polygon ( object_3d_face *this_face )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"wrender_unclipped_3d_object_textured_polygon\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dwface2.c",
        id: "LR-reader-wrender_unclipped_3d_object_gouraud_textured_polygon",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void wrender_unclipped_3d_object_gouraud_textured_polygon ( object_3d_face *this_face )\n{\n",
        replacement: "void wrender_unclipped_3d_object_gouraud_textured_polygon ( object_3d_face *this_face )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"wrender_unclipped_3d_object_gouraud_textured_polygon\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dwface2.c",
        id: "LR-reader-wrender_unclipped_3d_object_gouraud_textured_nospecular_polygon",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void wrender_unclipped_3d_object_gouraud_textured_nospecular_polygon ( object_3d_face *this_face )\n{\n",
        replacement: "void wrender_unclipped_3d_object_gouraud_textured_nospecular_polygon ( object_3d_face *this_face )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"wrender_unclipped_3d_object_gouraud_textured_nospecular_polygon\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dwface2.c",
        id: "LR-reader-wrender_unclipped_3d_object_polygon",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void wrender_unclipped_3d_object_polygon ( object_3d_face *this_face )\n{\n",
        replacement: "void wrender_unclipped_3d_object_polygon ( object_3d_face *this_face )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"wrender_unclipped_3d_object_polygon\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dwface2.c",
        id: "LR-reader-wrender_unclipped_3d_object_gouraud_polygon",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void wrender_unclipped_3d_object_gouraud_polygon ( object_3d_face *this_face )\n{\n",
        replacement: "void wrender_unclipped_3d_object_gouraud_polygon ( object_3d_face *this_face )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"wrender_unclipped_3d_object_gouraud_polygon\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dwface2.c",
        id: "LR-reader-wrender_translucent_unclipped_3d_object_reflective_polygon",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void wrender_translucent_unclipped_3d_object_reflective_polygon ( object_3d_face *this_face )\n{\n",
        replacement: "void wrender_translucent_unclipped_3d_object_reflective_polygon ( object_3d_face *this_face )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"wrender_translucent_unclipped_3d_object_reflective_polygon\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dwface2.c",
        id: "LR-reader-wrender_translucent_unclipped_3d_object_reflective_gouraud_polygon",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void wrender_translucent_unclipped_3d_object_reflective_gouraud_polygon ( object_3d_face *this_face )\n{\n",
        replacement: "void wrender_translucent_unclipped_3d_object_reflective_gouraud_polygon ( object_3d_face *this_face )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"wrender_translucent_unclipped_3d_object_reflective_gouraud_polygon\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/clouds/3dtrump.c",
        id: "LR-reader-transform_3d_cloud_trumpet",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void transform_3d_cloud_trumpet ( object_3d *object, viewpoint *vp, vec3d *relative_position )\n{\n",
        replacement: "void transform_3d_cloud_trumpet ( object_3d *object, viewpoint *vp, vec3d *relative_position )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"transform_3d_cloud_trumpet\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/clouds/3dtrump.c",
        id: "LR-reader-draw_3d_cloud_trumpet",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_cloud_trumpet ( void )\n{\n",
        replacement: "void draw_3d_cloud_trumpet ( void )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_cloud_trumpet\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/clouds/3dclouds.c",
        id: "LR-reader-transform_3d_cloud",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void transform_3d_cloud ( object_3d *object, vec3d *pos, light_3d_source *lights, int points_base )\n{\n",
        replacement: "void transform_3d_cloud ( object_3d *object, vec3d *pos, light_3d_source *lights, int points_base )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"transform_3d_cloud\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/clouds/3dclouds.c",
        id: "LR-reader-draw_3d_cloud",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_cloud ( object_3d_instance *obj )\n{\n",
        replacement: "void draw_3d_cloud ( object_3d_instance *obj )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_cloud\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrh.c",
        id: "LR-reader-draw_3d_object_hardware_faces",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "static void draw_3d_object_hardware_faces ( int object_number, object_3d_info *this_object_3d_info )\n{\n",
        replacement: "static void draw_3d_object_hardware_faces ( int object_number, object_3d_info *this_object_3d_info )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_object_hardware_faces\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrh.c",
        id: "LR-reader-draw_3d_object_untextured_hardware_faces",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "static void draw_3d_object_untextured_hardware_faces ( int object_number, object_3d_info *this_object_3d_info )\n{\n",
        replacement: "static void draw_3d_object_untextured_hardware_faces ( int object_number, object_3d_info *this_object_3d_info )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_object_untextured_hardware_faces\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrh.c",
        id: "LR-reader-draw_hardware_3d_object",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_hardware_3d_object ( object_3d_instance *obj, int infrared_override )\n{\n",
        replacement: "void draw_hardware_3d_object ( object_3d_instance *obj, int infrared_override )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_hardware_3d_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrh.c",
        id: "LR-reader-draw_hardware_zbuffered_3d_object",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_hardware_zbuffered_3d_object ( object_3d_instance *obj, int object_is_flat, int infrared_override )\n{\n",
        replacement: "void draw_hardware_zbuffered_3d_object ( object_3d_instance *obj, int object_is_flat, int infrared_override )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_hardware_zbuffered_3d_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrh.c",
        id: "LR-reader-draw_sub_object@3dobjdrh",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "static void draw_sub_object ( object_3d_sub_instance *obj, object_3d_database_entry *scene, viewpoint *parent_viewpoint, vec3d *position, int infrared_override )\n{\n",
        replacement: "static void draw_sub_object ( object_3d_sub_instance *obj, object_3d_database_entry *scene, viewpoint *parent_viewpoint, vec3d *position, int infrared_override )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_sub_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrh.c",
        id: "LR-reader-draw_zbuffered_sub_object",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "static void draw_zbuffered_sub_object ( object_3d_sub_instance *obj, object_3d_database_entry *scene, viewpoint *parent_viewpoint, vec3d *position, int infrared_override )\n{\n",
        replacement: "static void draw_zbuffered_sub_object ( object_3d_sub_instance *obj, object_3d_database_entry *scene, viewpoint *parent_viewpoint, vec3d *position, int infrared_override )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_zbuffered_sub_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrh.c",
        id: "LR-reader-draw_3d_hardware_translucent_object",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_hardware_translucent_object ( translucent_object_surface *translucent_surfaces )\n{\n",
        replacement: "void draw_3d_hardware_translucent_object ( translucent_object_surface *translucent_surfaces )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_hardware_translucent_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrh.c",
        id: "LR-reader-draw_3d_hardware_zbuffered_translucent_object",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_hardware_zbuffered_translucent_object ( translucent_object_surface *translucent_surfaces )\n{\n",
        replacement: "void draw_3d_hardware_zbuffered_translucent_object ( translucent_object_surface *translucent_surfaces )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_hardware_zbuffered_translucent_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrw.c",
        id: "LR-reader-draw_3d_object_clipped_faces",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_object_clipped_faces ( int object_number )\n{\n",
        replacement: "void draw_3d_object_clipped_faces ( int object_number )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_object_clipped_faces\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrw.c",
        id: "LR-reader-draw_3d_object_unclipped_faces",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_object_unclipped_faces ( int object_number )\n{\n",
        replacement: "void draw_3d_object_unclipped_faces ( int object_number )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_object_unclipped_faces\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrw.c",
        id: "LR-reader-draw_3d_object_untextured_clipped_faces",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_object_untextured_clipped_faces ( int object_number )\n{\n",
        replacement: "void draw_3d_object_untextured_clipped_faces ( int object_number )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_object_untextured_clipped_faces\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrw.c",
        id: "LR-reader-draw_3d_object_untextured_unclipped_faces",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_object_untextured_unclipped_faces ( int object_number )\n{\n",
        replacement: "void draw_3d_object_untextured_unclipped_faces ( int object_number )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_object_untextured_unclipped_faces\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrw.c",
        id: "LR-reader-draw_wbuffered_3d_object",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_wbuffered_3d_object ( object_3d_instance *obj, int object_is_flat, int infrared_override )\n{\n",
        replacement: "void draw_wbuffered_3d_object ( object_3d_instance *obj, int object_is_flat, int infrared_override )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_wbuffered_3d_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrw.c",
        id: "LR-reader-draw_sub_object@3dobjdrw",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_sub_object ( object_3d_sub_instance *obj, object_3d_database_entry *scene, viewpoint *parent_viewpoint, vec3d *position, int infrared_override )\n{\n",
        replacement: "void draw_sub_object ( object_3d_sub_instance *obj, object_3d_database_entry *scene, viewpoint *parent_viewpoint, vec3d *position, int infrared_override )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_sub_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrw.c",
        id: "LR-reader-draw_3d_translucent_object",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_translucent_object ( translucent_object_surface *translucent_surfaces )\n{\n",
        replacement: "void draw_3d_translucent_object ( translucent_object_surface *translucent_surfaces )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_translucent_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrw.c",
        id: "LR-reader-draw_3d_translucent_surface_clipped_faces",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_translucent_surface_clipped_faces ( translucent_object_surface *surface )\n{\n",
        replacement: "void draw_3d_translucent_surface_clipped_faces ( translucent_object_surface *surface )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_translucent_surface_clipped_faces\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrw.c",
        id: "LR-reader-draw_3d_translucent_surface_unclipped_faces",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_translucent_surface_unclipped_faces ( translucent_object_surface *surface )\n{\n",
        replacement: "void draw_3d_translucent_surface_unclipped_faces ( translucent_object_surface *surface )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_translucent_surface_unclipped_faces\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrw.c",
        id: "LR-reader-draw_3d_translucent_surface_clipped_reflection_faces",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_translucent_surface_clipped_reflection_faces ( translucent_object_surface *surface )\n{\n",
        replacement: "void draw_3d_translucent_surface_clipped_reflection_faces ( translucent_object_surface *surface )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_translucent_surface_clipped_reflection_faces\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrw.c",
        id: "LR-reader-draw_3d_translucent_surface_unclipped_reflection_faces",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_translucent_surface_unclipped_reflection_faces ( translucent_object_surface *surface )\n{\n",
        replacement: "void draw_3d_translucent_surface_unclipped_reflection_faces ( translucent_object_surface *surface )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_translucent_surface_unclipped_reflection_faces\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrz.c",
        id: "LR-reader-draw_zbuffered_3d_object",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_zbuffered_3d_object ( object_3d_instance *obj, int object_is_flat, int infrared_override )\n{\n",
        replacement: "void draw_zbuffered_3d_object ( object_3d_instance *obj, int object_is_flat, int infrared_override )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_zbuffered_3d_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdrz.c",
        id: "LR-reader-draw_sub_object@3dobjdrz",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "static void draw_sub_object ( object_3d_sub_instance *obj, object_3d_database_entry *scene, viewpoint *parent_viewpoint, vec3d *position, int infrared_override )\n{\n",
        replacement: "static void draw_sub_object ( object_3d_sub_instance *obj, object_3d_database_entry *scene, viewpoint *parent_viewpoint, vec3d *position, int infrared_override )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_sub_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjzdr.c",
        id: "LR-reader-draw_z_sorted_3d_object",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_z_sorted_3d_object ( object_3d_instance *obj )\n{\n",
        replacement: "void draw_z_sorted_3d_object ( object_3d_instance *obj )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_z_sorted_3d_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjzdr.c",
        id: "LR-reader-draw_z_sorted_sub_object",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_z_sorted_sub_object ( object_3d_sub_instance *obj, object_3d_database_entry *scene, viewpoint *parent_viewpoint, light_3d_source *parent_lights, vec3d *position )\n{\n",
        replacement: "void draw_z_sorted_sub_object ( object_3d_sub_instance *obj, object_3d_database_entry *scene, viewpoint *parent_viewpoint, light_3d_source *parent_lights, vec3d *position )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_z_sorted_sub_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjbuf.c",
        id: "LR-reader-draw_3d_scene",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_3d_scene ( void )\n{\n",
        replacement: "void draw_3d_scene ( void )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_3d_scene\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjbuf.c",
        id: "LR-reader-draw_normal_scene_objects",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_normal_scene_objects ( scene_slot_drawing_list *object_order )\n{\n",
        replacement: "void draw_normal_scene_objects ( scene_slot_drawing_list *object_order )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_normal_scene_objects\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjbuf.c",
        id: "LR-reader-draw_transparent_scene_objects",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_transparent_scene_objects ( scene_slot_drawing_list *object_order )\n{\n",
        replacement: "void draw_transparent_scene_objects ( scene_slot_drawing_list *object_order )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"draw_transparent_scene_objects\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dtrans.c",
        id: "LR-reader-transform_3d_object",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void transform_3d_object ( object_3d *object, vec3d *pos, light_3d_source *lights, vec3d *camera_direction, int points_base )\n{\n",
        replacement: "void transform_3d_object ( object_3d *object, vec3d *pos, light_3d_source *lights, vec3d *camera_direction, int points_base )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"transform_3d_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dtrans.c",
        id: "LR-reader-transform_flat_3d_object",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void transform_flat_3d_object ( object_3d *object, vec3d *pos, light_3d_source *lights, vec3d *camera_direction, int points_base )\n{\n",
        replacement: "void transform_flat_3d_object ( object_3d *object, vec3d *pos, light_3d_source *lights, vec3d *camera_direction, int points_base )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"transform_flat_3d_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dtrans.c",
        id: "LR-reader-transform_unclipped_3d_object",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void transform_unclipped_3d_object ( object_3d *object, vec3d *pos, light_3d_source *lights, vec3d *camera_direction, int points_base )\n{\n",
        replacement: "void transform_unclipped_3d_object ( object_3d *object, vec3d *pos, light_3d_source *lights, vec3d *camera_direction, int points_base )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"transform_unclipped_3d_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dtrans.c",
        id: "LR-reader-transform_unclipped_flat_3d_object",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void transform_unclipped_flat_3d_object ( object_3d *object, vec3d *pos, light_3d_source *lights, vec3d *camera_direction, int points_base )\n{\n",
        replacement: "void transform_unclipped_flat_3d_object ( object_3d *object, vec3d *pos, light_3d_source *lights, vec3d *camera_direction, int points_base )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"transform_unclipped_flat_3d_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dtrans.c",
        id: "LR-reader-illuminate_3d_object_surface",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void illuminate_3d_object_surface ( object_3d *object, vec3d *pos, light_3d_source *lights, vec3d *camera_direction, int points_base )\n{\n",
        replacement: "void illuminate_3d_object_surface ( object_3d *object, vec3d *pos, light_3d_source *lights, vec3d *camera_direction, int points_base )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"illuminate_3d_object_surface\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dtrans.c",
        id: "LR-reader-transform_3d_object_surface",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void transform_3d_object_surface ( object_3d *object, vec3d *pos, light_3d_source *lights, vec3d *camera_direction, int points_base )\n{\n",
        replacement: "void transform_3d_object_surface ( object_3d *object, vec3d *pos, light_3d_source *lights, vec3d *camera_direction, int points_base )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"transform_3d_object_surface\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dtrans.c",
        id: "LR-reader-transform_unclipped_3d_object_surface",
        why: "a legacy visual / display entry (a reader of a point, face, face-point-reference or surface-point buffer the loader no longer holds, or the draw, transform or illumination entry that dominates one): it stops by name at its head, before any buffer, image or device is touched",
        original: "void transform_unclipped_3d_object_surface ( object_3d *object, vec3d *pos, light_3d_source *lights, vec3d *camera_direction, int points_base )\n{\n",
        replacement: "void transform_unclipped_3d_object_surface ( object_3d *object, vec3d *pos, light_3d_source *lights, vec3d *camera_direction, int points_base )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader) */\n\tcampaign_model_geometry_reader ( \"transform_unclipped_3d_object_surface\" );\n",
        count: 1,
    },
];

/// the staged text of `file` with the loader-detachment patches applied (same strict count check as `apply`)
pub fn apply_loader_host(file: &str, text: &str) -> String {
    let mut out = text.to_string();
    for p in LOADER_HOST_PATCHES.iter().filter(|p| p.file == file) {
        let found = out.matches(p.original).count();
        assert_eq!(
            found, p.count,
            "loader-host patch {} expects {} occurrence(s) of its original text in {}, found {}",
            p.id, p.count, file, found
        );
        out = out.replace(p.original, p.replacement);
    }
    out
}

pub fn loader_host_files() -> Vec<&'static str> {
    let mut files: Vec<&str> = LOADER_HOST_PATCHES.iter().map(|p| p.file).collect();
    files.sort_unstable();
    files.dedup();
    files
}

/// EEDCS-50, D3 L2 (model-file METADATA): the common and custom model-file loaders no longer parse a model file. Applied by main.rs after LOADER_HOST_PATCHES, in the builds
/// that have the host-owned route data (not model-fixture or route-original, which keep the original loaders), to the staged text those sets produced; same strict count check.
/// The campaign's own object table, id numbering, name history and sharing, and the scene / hierarchy loader (3dobjdb.bin and .EES) are unchanged: the host is asked, by
/// (scene, role, cursor), for the metadata of the objects a scene names (campaign_model_scene_objects), for the object count (campaign_model_object_table) and for the ordered
/// animation references of a custom scene (campaign_model_animation_reference), all in aphavoc/source/entity/mobile/mb_hostmodel.c.
pub const LOADER_META_PATCHES: &[Patch] = &[
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM1-custom-loader-declaration-removed",
        why: "the custom model-file loader (a named .EEO) is not compiled: the host owns the files, so its forward declaration goes with it",
        original: "static int read_object ( object_3d *obj, const char* filename );\n",
        replacement: "/* EECH headless (D3 loader, L2): the custom model-file loader (a named .EEO) is not compiled; the host owns the files */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM2-named-object-not-read",
        why: "get_object keeps its id allocation, name history and sharing exactly; it no longer opens the object's .EEO, but a NEW number still replaces its row as the original's read did (the row is cleared and unfilled): the host describes the object when the scene that names it has been read (campaign_model_scene_objects, LM4)",
        original: "\tsprintf ( new_filename, \"%s\\\\%s.EEO\", prefix, ptr );\n\tif ( !read_object ( &objects_3d_data[objid], new_filename ) )\n\t{\n\t\tdebug_log ( \"failed to load object: %s\", new_filename );\n\t\tfail = 1;\n\t}\n\n\treturn objid;\n",
        replacement: "\t/* EECH headless (D3 loader, L2): the object's .EEO is the host's. The number is NEW, and the original opened the file into this row here, REPLACING whatever the row held (for the first named object, the last common object): the private row is cleared and unfilled, and the host describes it when the scene that names it has been read (campaign_model_scene_objects) */\n\t{\n\t\textern void campaign_model_object_reset ( int object_number );\n\n\t\tcampaign_model_object_reset ( objid );\n\t}\n\n\treturn objid;\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM3a-surface-animation-walk-not-compiled",
        why: "add_animation_object walked a loaded object's surfaces for animated textures; the surfaces are the host's now and the scene's animation references come from the host as logical names (LM4), so the walk is not compiled",
        original: "static void add_animation_object (struct OBJECT_3D_SCENE_DATABASE_ENTRY *scene, int *allocated, int object_id )\n{\n",
        replacement: "#if 0 /* EECH headless (D3 loader, L2): the surfaces are the host's; the scene's animation references are asked for by name (read_scene) */\nstatic void add_animation_object (struct OBJECT_3D_SCENE_DATABASE_ENTRY *scene, int *allocated, int object_id )\n{\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM3b-surface-animation-walk-end",
        why: "closes the block of LM3a",
        original: "\t\t\tadd_animation ( scene, allocated, object_3d->surfaces[surface].luminosity_texture_index );\n\t\t}\n\t}\n}\n",
        replacement: "\t\t\tadd_animation ( scene, allocated, object_3d->surfaces[surface].luminosity_texture_index );\n\t\t}\n\t}\n}\n#endif\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM4-scene-objects-and-animation-references-from-the-host",
        why: "after a scene's sub-objects are read (so before any bounding box is): the host describes every object the scene names by (scene, role, cursor) and the campaign's private rows are filled with its true counts, radius and bounds; and the animation list of a scene a custom scene file defines is built by the campaign's own add_animation from the host's ordered references (logical names), in the order, with the de-duplication, of the walk it replaces",
        original: "\tread_subobjects ( fp, &objects_3d_scene_database[scene_index].number_of_sub_objects, &objects_3d_scene_database[scene_index].sub_objects, NULL, version );\n\n\t// Check for animation textures references in the objects of the scene\n\tif ( sceneid )\n\t{\n\t\tint\n\t\t\tallocated,\n\t\t\tcount;\n\n\t\tallocated = 0;\n\n\t\tadd_animation_object ( &objects_3d_scene_database[scene_index], &allocated, objects_3d_scene_database[scene_index].index );\n\t\tfor ( count = 0; count < objects_3d_scene_database[scene_index].number_of_approximations; count++ )\n\t\t{\n\t\t\tadd_animation_object ( &objects_3d_scene_database[scene_index], &allocated, objects_3d_scene_database[scene_index].approximations[count].object_number );\n\t\t}\n\t\tfor ( count = 0; count < objects_3d_scene_database[scene_index].total_number_of_sub_objects; count++ )\n\t\t{\n\t\t\tadd_animation_object ( &objects_3d_scene_database[scene_index], &allocated, objects_3d_scene_database[scene_index].sub_objects[count].index );\n\t\t}\n\t}\n\n\tinitialise_scene_quick_sub_object_search ( scene_index );\n",
        replacement: "\tread_subobjects ( fp, &objects_3d_scene_database[scene_index].number_of_sub_objects, &objects_3d_scene_database[scene_index].sub_objects, NULL, version );\n\n\t/* EECH headless (D3 loader, L2): the host describes every object the scene names, by (scene, role, cursor), and the campaign's private rows are filled from its answers before anything reads a bounding box */\n\t{\n\t\textern void campaign_model_scene_objects ( int scene_index, int custom );\n\n\t\tcampaign_model_scene_objects ( scene_index, sceneid );\n\t}\n\n\t// Check for animation textures references in the objects of the scene: the host's ordered references (logical names), added by the campaign's own add_animation\n\tif ( sceneid )\n\t{\n\t\tint\n\t\t\tallocated,\n\t\t\tcount,\n\t\t\tanimation;\n\n\t\textern int campaign_model_animation_reference ( int scene_index, int index ); /* EECH headless (D3 loader, L2) */\n\n\t\tallocated = 0;\n\n\t\tfor ( count = 0; ( animation = campaign_model_animation_reference ( scene_index, count ) ) >= 0; count++ )\n\t\t{\n\t\t\tadd_animation ( &objects_3d_scene_database[scene_index], &allocated, animation );\n\t\t}\n\t}\n\n\tinitialise_scene_quick_sub_object_search ( scene_index );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM5a-custom-loader-not-compiled",
        why: "read_object parsed a named .EEO into the campaign's own arrays; the host owns the file now, and the loader is not compiled (the original text stays in the staged copy only, inside #if 0)",
        original: "int read_object ( object_3d *obj, const char* filename )\n{\n\tFILE\n",
        replacement: "#if 0 /* EECH headless (D3 loader, L2): the custom model-file loader is not compiled; the host owns the files */\nint read_object ( object_3d *obj, const char* filename )\n{\n\tFILE\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM5b-custom-loader-end",
        why: "closes the block of LM5a",
        original: "\t\t\t\treturn FALSE;\n\t\t\t}\n\t\t}\n\t}\n\n\treturn TRUE;\n}\n\n/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////\n/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////\n/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////\n\nvoid initialise_3d_objects ( const char *directory )\n",
        replacement: "\t\t\t\treturn FALSE;\n\t\t\t}\n\t\t}\n\t}\n\n\treturn TRUE;\n}\n#endif\n\n/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////\n/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////\n/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////\n\nvoid initialise_3d_objects ( const char *directory )\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM6a-common-loader-host-table",
        why: "the common loader no longer opens 3dobjs.bin or maps its point, surface-point, normal and texture-point files; the host says how many objects its database holds, and the campaign allocates its own rows (with its unchanged room for custom objects) from that count, every physical pointer NULL",
        original: "\tsprintf ( filename, \"%s\\\\3dobjs.bin\", directory );\n\tfp = safe_fopen ( filename, \"rb\" );\n",
        replacement: "\t/* EECH headless (D3 loader, L2): the host owns the model files. It says how many objects its common database holds; the campaign's rows, its custom room and its object numbering are its own, and every row starts zero, its physical pointers NULL, until the host describes the objects a scene names */\n\t{\n\t\textern int campaign_model_object_table ( int custom_room );\n\n\t\ttotal_number_of_objects = campaign_model_object_table ( custom_number_of_objects );\n\t}\n\n\ttotal_number_of_raw_3d_objects = total_number_of_objects + custom_number_of_objects;\n\n\tobjects_3d_data = ( struct OBJECT_3D * ) safe_malloc ( ( total_number_of_raw_3d_objects + 1 ) * sizeof ( struct OBJECT_3D ) );\n\tmemset ( objects_3d_data, 0, ( total_number_of_raw_3d_objects + 1 ) * sizeof ( struct OBJECT_3D ) );\n\n\tobject_database_points = NULL;\n\tobject_database_faces = NULL;\n\tobject_database_point_normals = NULL;\n\tobject_database_surfaces = NULL;\n\tobject_database_faces_point_list = NULL;\n\tobject_database_faces_face_normal_list = NULL;\n\tobject_database_surface_point_list = NULL;\n\tobject_database_surface_point_normal_list = NULL;\n\tobject_database_surface_point_texture_list = NULL;\n\n#if 0 /* EECH headless (D3 loader, L2): the common model-file loader and the binary-hex .EEO import are not compiled */\n\tsprintf ( filename, \"%s\\\\3dobjs.bin\", directory );\n\tfp = safe_fopen ( filename, \"rb\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM6b-common-loader-end",
        why: "closes the block of LM6a",
        original: "\t/* 26JUL06 Casm Import of 3D objects END */\n",
        replacement: "\t/* 26JUL06 Casm Import of 3D objects END */\n#endif\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM7a-surface-consolidation-not-compiled",
        why: "the consolidation of every object's surfaces into one block copied the loaded surfaces; there are none, the surfaces are the host's",
        original: "\t{\n\t\t// reallocates surfaces memory\n",
        replacement: "#if 0 /* EECH headless (D3 loader, L2): there are no surfaces to consolidate */\n\t{\n\t\t// reallocates surfaces memory\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM7b-surface-consolidation-end",
        why: "closes the block of LM7a",
        original: "\t\tobject_database_surfaces = surfaces;\n\t\ttotal_number_of_surfaces = number_of_surfaces;\n\t}\n",
        replacement: "\t\tobject_database_surfaces = surfaces;\n\t\ttotal_number_of_surfaces = number_of_surfaces;\n\t}\n#endif\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM-reader-get_object_3d_number_of_poly_lines",
        why: "a legacy visual / display entry (a reader of the surface or normal arrays, or of a point, face or reference buffer, that the loader no longer holds): it stops by name at its head, before any buffer, image or device is touched",
        original: "int get_object_3d_number_of_poly_lines ( object_3d_index_numbers index )\n{\n",
        replacement: "int get_object_3d_number_of_poly_lines ( object_3d_index_numbers index )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader, L2) */\n\tcampaign_model_geometry_reader ( \"get_object_3d_number_of_poly_lines\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dtrans.c",
        id: "LM-reader-illuminate_3d_object",
        why: "a legacy visual / display entry (a reader of the surface or normal arrays, or of a point, face or reference buffer, that the loader no longer holds): it stops by name at its head, before any buffer, image or device is touched",
        original: "void illuminate_3d_object ( object_3d *object, vec3d *pos, light_3d_source *lights, vec3d *camera_direction, int points_base )\n{\n",
        replacement: "void illuminate_3d_object ( object_3d *object, vec3d *pos, light_3d_source *lights, vec3d *camera_direction, int points_base )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader, L2) */\n\tcampaign_model_geometry_reader ( \"illuminate_3d_object\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dwface1.c",
        id: "LM-reader-wrender_translucent_clipped_3d_object_reflective_polygon",
        why: "a legacy visual / display entry (a reader of the surface or normal arrays, or of a point, face or reference buffer, that the loader no longer holds): it stops by name at its head, before any buffer, image or device is touched",
        original: "void wrender_translucent_clipped_3d_object_reflective_polygon ( object_3d_face *this_face )\n{\n",
        replacement: "void wrender_translucent_clipped_3d_object_reflective_polygon ( object_3d_face *this_face )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader, L2) */\n\tcampaign_model_geometry_reader ( \"wrender_translucent_clipped_3d_object_reflective_polygon\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/clouds/3dtrump.c",
        id: "LM-reader-render_3d_cloud_trumpet_face",
        why: "a legacy visual / display entry (a reader of the surface or normal arrays, or of a point, face or reference buffer, that the loader no longer holds): it stops by name at its head, before any buffer, image or device is touched",
        original: "void render_3d_cloud_trumpet_face ( object_3d_face *this_face, int surface_index, int reference_offset, object_3d_info *object_base )\n{\n",
        replacement: "void render_3d_cloud_trumpet_face ( object_3d_face *this_face, int surface_index, int reference_offset, object_3d_info *object_base )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader, L2) */\n\tcampaign_model_geometry_reader ( \"render_3d_cloud_trumpet_face\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobject.c",
        id: "LM-reader-pre_render_object_faces",
        why: "a legacy visual / display entry (a reader of the surface or normal arrays, or of a point, face or reference buffer, that the loader no longer holds): it stops by name at its head, before any buffer, image or device is touched",
        original: "void pre_render_object_faces ( int object_number )\n{\n",
        replacement: "void pre_render_object_faces ( int object_number )\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader, L2) */\n\tcampaign_model_geometry_reader ( \"pre_render_object_faces\" );\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/gunships/avionics/hind/hi_hud.c",
        id: "LM-reader-draw_hind_hud_on_texture",
        why: "a legacy visual / display entry (a reader of the surface or normal arrays, or of a point, face or reference buffer, that the loader no longer holds): it stops by name at its head, before any buffer, image or device is touched",
        original: "void draw_hind_hud_on_texture (void)\n{\n",
        replacement: "void draw_hind_hud_on_texture (void)\n{\n\textern void campaign_model_geometry_reader ( const char *reader ); /* EECH headless (D3 loader, L2) */\n\tcampaign_model_geometry_reader ( \"draw_hind_hud_on_texture\" );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dsubobj.c",
        id: "LM8a-scene-bounds-require-a-filled-row",
        why: "the scene's bounds are rotated from its main object's bounding boxes: that object's row must be one the host described (or the null object), else the engine stops by name before the row is read",
        original: "\t\trotate_sub_scene_boundaries ( &rotated_bounds, &objects_3d_data[index].bounding_box );\n\t\trotate_sub_scene_boundaries ( &rotated_bounds2, &objects_3d_data[index].bounding_box2 );\n",
        replacement: "\t\textern struct OBJECT_3D *campaign_model_object_row ( int object_number, const char *reader ); /* EECH headless (D3 loader, L2) */\n\t\trotate_sub_scene_boundaries ( &rotated_bounds, &campaign_model_object_row ( index, \"initialise_scene_bounding_sub_objects (scene main object)\" )->bounding_box );\n\t\trotate_sub_scene_boundaries ( &rotated_bounds2, &campaign_model_object_row ( index, \"initialise_scene_bounding_sub_objects (scene main object)\" )->bounding_box2 );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dsubobj.c",
        id: "LM8b-sub-object-bounds-require-a-filled-row",
        why: "a sub-object's bounds are rotated from its object's bounding boxes: that object's row must be one the host described (or the null object), else the engine stops by name before the row is read",
        original: "\trotate_sub_scene_boundaries ( &rotated_bounds, &objects_3d_data[ sub_object->index ].bounding_box );\n\trotate_sub_scene_boundaries ( &rotated_bounds2, &objects_3d_data[ sub_object->index ].bounding_box2 );\n",
        replacement: "\t{\n\t\textern struct OBJECT_3D *campaign_model_object_row ( int object_number, const char *reader ); /* EECH headless (D3 loader, L2) */\n\n\t\trotate_sub_scene_boundaries ( &rotated_bounds, &campaign_model_object_row ( sub_object->index, \"initialise_scene_bounding_sub_objects (sub-object)\" )->bounding_box );\n\t\trotate_sub_scene_boundaries ( &rotated_bounds2, &campaign_model_object_row ( sub_object->index, \"initialise_scene_bounding_sub_objects (sub-object)\" )->bounding_box2 );\n\t}\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dhoriz.c",
        id: "LM9a-horizon-environment-requires-a-filled-row",
        why: "the horizon environment sizes its point colours from the horizon object's true point count and takes its maximum height from its bounds: that object's row must be one the host described, else the engine stops by name",
        original: "\tobject = &objects_3d_data[env->horizon_3d_object_number];\n\n\tenv->horizon_point_colours =",
        replacement: "\t{\n\t\textern struct OBJECT_3D *campaign_model_object_row ( int object_number, const char *reader ); /* EECH headless (D3 loader, L2) */\n\n\t\tobject = campaign_model_object_row ( env->horizon_3d_object_number, \"the horizon environment\" );\n\t}\n\n\tenv->horizon_point_colours =",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dhoriz.c",
        id: "LM9b-horizon-image-requires-a-filled-row",
        why: "the horizon image compares its pre-processed .bin's point count with the horizon object's true count: that object's row must be one the host described, else the engine stops by name",
        original: "\tobject = &objects_3d_data[env->horizon_3d_object_number];\n\n\t//\n\t// Check to see if we have a pre-processed version of this file\n",
        replacement: "\t{\n\t\textern struct OBJECT_3D *campaign_model_object_row ( int object_number, const char *reader ); /* EECH headless (D3 loader, L2) */\n\n\t\tobject = campaign_model_object_row ( env->horizon_3d_object_number, \"add_3d_horizon_image\" );\n\t}\n\n\t//\n\t// Check to see if we have a pre-processed version of this file\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM10a-custom-walk-announcement-defined",
        why: "the campaign announces to the host the order in which it will walk the custom scene files (the file system's own, unsorted, order), before it reads any: what a custom definition sees depends on that order, so the host replays the campaign's and not a sorted one",
        original: "static void initialise_custom_scenes(const char* directory)\n{\n",
        replacement: "/* EECH headless (D3 loader, L2): the campaign ANNOUNCES the walk of the custom scene files to the host before it reads any. The walk is the same _findfirst / _findnext one\n   initialise_custom_scenes makes below (the file system's own order, not a sorted one), one announcement per file the walk will read, so that what a custom definition sees is\n   replayed by the host in the order the campaign sees it. */\nstatic void announce_custom_scenes ( const char *directory )\n{\n\tint\n\t\trc = 0,\n\t\tposition = 0;\n\tlong\n\t\thandle;\n\tstruct _finddata_t\n\t\tdi;\n\tchar\n\t\tpattern[512];\n\textern void campaign_model_custom_order ( int scene, int position, const char *directory_name );\n\textern void campaign_model_custom_order_end ( int count );\n\n\tsnprintf ( pattern, ARRAY_LENGTH(pattern), \"%s\\\\OBJECTS\\\\*.*\", directory );\n\tfor ( rc = handle = _findfirst ( pattern, &di ); rc != -1; rc = _findnext ( handle, &di ) )\n\t{\n\t\tint\n\t\t\tinner_rc;\n\t\tlong\n\t\t\tinner_handle;\n\t\tstruct _finddata_t\n\t\t\tfi;\n\n\t\tif ( !( di.attrib & _A_SUBDIR ) )\n\t\t\tcontinue;\n\n\t\tif ( !strcmp ( di.name, \".\" ) || !strcmp ( di.name, \"..\" ) )\n\t\t\tcontinue;\n\n\t\tsnprintf ( pattern, ARRAY_LENGTH(pattern), \"%s\\\\OBJECTS\\\\%s\\\\*.EES\", directory, di.name );\n\t\tfor ( inner_rc = inner_handle = _findfirst ( pattern, &fi ); inner_rc != -1; inner_rc = _findnext ( inner_handle, &fi ) )\n\t\t{\n\t\t\tint\n\t\t\t\tscene;\n\n\t\t\tif ( fi.attrib & _A_SUBDIR )\n\t\t\t\tcontinue;\n\n\t\t\tfi.name[strlen ( fi.name ) - 4] = '\\0';\n\t\t\tstrupr ( fi.name );\n\t\t\tscene = get_scene ( fi.name );\n\t\t\tif ( !scene )\n\t\t\t\tcontinue;\n\n\t\t\tcampaign_model_custom_order ( scene, position++, di.name );\n\t\t}\n\t\t_findclose ( inner_handle );\n\t}\n\n\t_findclose ( handle );\n\n\tcampaign_model_custom_order_end ( position );\n}\n\nstatic void initialise_custom_scenes(const char* directory)\n{\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM10b-custom-walk-announced",
        why: "the announcement is made before the first file of the walk is read",
        original: "\tsnprintf ( filename, ARRAY_LENGTH(filename), \"%s\\\\OBJECTS\\\\*.*\", directory );\n",
        replacement: "\tannounce_custom_scenes ( directory ); /* EECH headless (D3 loader, L2) */\n\n\tsnprintf ( filename, ARRAY_LENGTH(filename), \"%s\\\\OBJECTS\\\\*.*\", directory );\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM10c-custom-walk-verified",
        why: "as each custom scene file is opened, the file the walk reached is checked against the one announced at that position, so the host's definition for it is the file really read",
        original: "\t\t\tsnprintf ( prefix, ARRAY_LENGTH ( prefix ), \"%s\\\\OBJECTS\\\\%s\", directory, di.name );\n",
        replacement: "\t\t\tsnprintf ( prefix, ARRAY_LENGTH ( prefix ), \"%s\\\\OBJECTS\\\\%s\", directory, di.name );\n\n\t\t\t{\n\t\t\t\textern void campaign_model_custom_reading ( int scene, const char *directory_name ); /* EECH headless (D3 loader, L2) */\n\n\t\t\t\tcampaign_model_custom_reading ( sceneid, di.name );\n\t\t\t}\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM10d-custom-file-skipped-by-version",
        why: "read_custom_scene opens a file, finds a scene version the original does not read, logs it and returns without read_scene: the walk position the file held is consumed explicitly as SKIPPED (no object row is claimed read)",
        original: "\t\tdebug_log ( \"Unsupported scene version %i in file %s\", version, filename );\n",
        replacement: "\t\tdebug_log ( \"Unsupported scene version %i in file %s\", version, filename );\n\t\t{\n\t\t\textern void campaign_model_custom_skipped ( void ); /* EECH headless (D3 loader, L2) */\n\n\t\t\tcampaign_model_custom_skipped ();\n\t\t}\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "LM10e-custom-walk-complete",
        why: "the actual traversal of the custom scene files is over: every announced position must have been opened, each read or explicitly skipped, and no read left outstanding, before the boot goes on",
        original: "\t_findclose ( handle );\n\t/* 06FEB08 Casm Import of 3D scenes END */\n",
        replacement: "\t_findclose ( handle );\n\n\t{\n\t\textern void campaign_model_custom_walk_complete ( void ); /* EECH headless (D3 loader, L2) */\n\n\t\tcampaign_model_custom_walk_complete ();\n\t}\n\t/* 06FEB08 Casm Import of 3D scenes END */\n",
        count: 1,
    },
];

/// the staged text of `file` with the metadata-loader patches applied (same strict count check as `apply`)
pub fn apply_loader_meta(file: &str, text: &str) -> String {
    let mut out = text.to_string();
    for p in LOADER_META_PATCHES.iter().filter(|p| p.file == file) {
        let found = out.matches(p.original).count();
        assert_eq!(
            found, p.count,
            "loader-meta patch {} expects {} occurrence(s) of its original text in {}, found {}",
            p.id, p.count, file, found
        );
        out = out.replace(p.original, p.replacement);
    }
    out
}

pub fn loader_meta_files() -> Vec<&'static str> {
    let mut files: Vec<&str> = LOADER_META_PATCHES.iter().map(|p| p.file).collect();
    files.sort_unstable();
    files.dedup();
    files
}

/// TEST-ONLY patches of the `model-host-fixture` feature (EEDCS-50, D3 L2): observation hooks at the real stages of the scenes' texture animation lists (read_scene, add_animation) in the
/// staged text the shipping sets produced, the same stages the original-observer hooks MA0..MA4 take, so the host profile's rows report the allocation, reads and fills that
/// happened and infer none. Each ADDS a call and changes no statement, argument, order or value (the wrapped fread passes its result on unchanged). Same strict count check.
pub const HOST_ANIM_FIXTURE_PATCHES: &[Patch] = &[
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "HA1-hooks-declared",
        why: "declares the host-profile observer hooks (csrc/eech_model_host_fixture.c) before add_animation, which is the first place they are called",
        original: "static void add_animation (struct OBJECT_3D_SCENE_DATABASE_ENTRY *scene, int *allocated, int animation )\n{\n",
        replacement: "extern void eech_hf_anim_begin ( int scene, int sceneid, int declared ); /* EECH host fixture (HA1) */\nextern void eech_hf_anim_file_read ( int scene, int index, int total, const void *list, size_t returned );\nextern void eech_hf_anim_alloc ( const void *scene, const void *list, int capacity );\nextern void eech_hf_anim_append ( const void *scene );\nextern void eech_hf_anim_done ( int scene, int fail );\nextern void eech_hf_anim_reset ( void );\n\nstatic void add_animation (struct OBJECT_3D_SCENE_DATABASE_ENTRY *scene, int *allocated, int animation )\n{\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "HA2-animation-list-declared",
        why: "read_scene has read the scene's declared texture animation count (and knows whether it was given a sceneid): a new record of the scene's list begins",
        original: "\tfread ( &number_of_texture_animations, sizeof ( int ), 1, fp );\n",
        replacement: "\tfread ( &number_of_texture_animations, sizeof ( int ), 1, fp );\n\teech_hf_anim_begin ( scene_index, sceneid, number_of_texture_animations ); /* EECH host fixture (HA2) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "HA3-animation-list-file-entry",
        why: "one fread of one entry of a declared list: what it returned is recorded and handed back unchanged to the statement (which ignores it, as the original did)",
        original: "\t\t\tfread ( &objects_3d_scene_database[scene_index].texture_animations[tmp], sizeof ( int ), 1, fp );\n",
        replacement: "\t\t\teech_hf_anim_file_read ( scene_index, tmp, number_of_texture_animations, objects_3d_scene_database[scene_index].texture_animations, fread ( &objects_3d_scene_database[scene_index].texture_animations[tmp], sizeof ( int ), 1, fp ) ); /* EECH host fixture (HA3) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "HA4-animation-list-allocated",
        why: "add_animation has allocated the grown array and bound it as the scene's list: its address and size are recorded",
        original: "\t\tscene->texture_animations = new_anim;\n",
        replacement: "\t\tscene->texture_animations = new_anim;\n\t\teech_hf_anim_alloc ( scene, new_anim, *allocated ); /* EECH host fixture (HA4) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "HA5-animation-list-appended",
        why: "add_animation has appended one entry: the scene's count is recorded",
        original: "\tscene->texture_animations[scene->number_of_texture_animations++] = animation;\n",
        replacement: "\tscene->texture_animations[scene->number_of_texture_animations++] = animation;\n\teech_hf_anim_append ( scene ); /* EECH host fixture (HA5) */\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "HA6-animation-list-final",
        why: "read_scene is finishing the scene: the list and count it leaves (and its failure flag) are recorded before the scene's success flag is stored",
        original: "\tobjects_3d_scene_database[scene_index].succeeded = !fail;\n",
        replacement: "\teech_hf_anim_done ( scene_index, fail ); /* EECH host fixture (HA6) */\n\tobjects_3d_scene_database[scene_index].succeeded = !fail;\n",
        count: 1,
    },
    Patch {
        file: "modules/3d/3dobjdb.c",
        id: "HA7-reset-on-deinitialise",
        why: "the objects are freed: every record is dropped",
        original: "void deinitialise_3d_objects ( void )\n{\n",
        replacement: "void deinitialise_3d_objects ( void )\n{\n\teech_hf_anim_reset (); /* EECH host fixture (HA7) */\n\n",
        count: 1,
    },
];

pub fn apply_host_anim_fixture(file: &str, text: &str) -> String {
    let mut out = text.to_string();
    for p in HOST_ANIM_FIXTURE_PATCHES.iter().filter(|p| p.file == file) {
        let found = out.matches(p.original).count();
        assert_eq!(
            found, p.count,
            "host-anim-fixture patch {} expects {} occurrence(s) of its original text in {}, found {}",
            p.id, p.count, file, found
        );
        out = out.replace(p.original, p.replacement);
    }
    out
}

pub fn host_anim_fixture_files() -> Vec<&'static str> {
    let mut files: Vec<&str> = HOST_ANIM_FIXTURE_PATCHES.iter().map(|p| p.file).collect();
    files.sort_unstable();
    files.dedup();
    files
}

/// one R8 patch: after `raw_object_index = <entry>->index;` the fixture is handed the entry and the number (the extern declaration is a block-local one, so no include is needed)
macro_rules! caller_patch {
    ($entry:literal, $count:expr) => {
        Patch {
            file: "aphavoc/source/ai/faction/popread.c",
            id: concat!("R8-caller-", $entry),
            why: "the route entry the original caller holds, paired with the raw object number it is about to pass to the parse",
            original: concat!("raw_object_index = ", $entry, "->index;\n"),
            replacement: concat!(
                "raw_object_index = ",
                $entry,
                "->index;\n\t\t\t{\n\t\t\t\textern void eech_rf_caller ( const void *entry, int raw_object_index ); /* EECH route fixture (R8) */\n\t\t\t\teech_rf_caller ( ",
                $entry,
                ", raw_object_index );\n\t\t\t}\n"
            ),
            count: $count,
        }
    };
}

/// TEST-ONLY patches of the `route-fixture` feature (EEDCS-50, D3): observation hooks in the ORIGINAL airfield route parser and its route-object lookup, applied by
/// main.rs ONLY when that feature is on, to the staged copy the shipping patches already produced (neither file has a shipping patch). They record values the original
/// statements already read or computed (csrc/eech_route_hooks.h, csrc/eech_route_fixture.c); each ADDS a call and changes no original statement, argument, order or
/// value. R2 adds one static observer function that reads the parser's own tables after the parse and turns the node pointers into node-array indices. With the feature
/// off none of this exists in the build. R7a/R7b also hand the hook the route entry the lookup returned (a private token that is never reported), and R8 hands it, at every original
/// parse call site, the entry the caller holds and the raw number it passes (the caller context). Same strict count check as PATCHES.
pub const ROUTE_FIXTURE_PATCHES: &[Patch] = &[
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "R1-hooks-include",
        why: "declares the observer hooks (csrc/eech_route_hooks.h)",
        original: "#include \"routegen.h\"\n",
        replacement: "#include \"routegen.h\"\n#include \"eech_route_hooks.h\" /* EECH route fixture (R1) */\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "R2-final-observer",
        why: "a static function, run once at the end of a parse, that hands the parser's final node, link, start and per-depth tables to the hooks (node pointers become node-array indices)",
        original: "void parse_waypoint_routes_from_object ( int object_index, int number_of_matching_slots, vec3d *slots )\n{\n",
        replacement: concat!(
            "static void eech_rf_observe_final ( int maximum_depth ) /* EECH route fixture (R2) */\n",
            "{\n",
            "\tint n, k;\n",
            "\n",
            "\teech_rf_final_totals ( number_of_waypoint_nodes, number_of_waypoint_starting_nodes, number_of_route_waypoint_positions, maximum_depth );\n",
            "\tfor ( n = 0; n < number_of_waypoint_nodes && n < MAX_WAYPOINT_NODES; n++ )\n",
            "\t{\n",
            "\t\twaypoint_node *w = &waypoint_nodes[n];\n",
            "\n",
            "\t\teech_rf_final_node ( n, w->tree_depth, w->tree_number, w->node_index, w->primary, w->start, w->possible_start, w->references, w->in_route, w->processed, w->number_of_children, w->number_of_parents );\n",
            "\t\tfor ( k = 0; k < w->number_of_children && k < MAX_WAYPOINT_LINKS; k++ )\n",
            "\t\t{\n",
            "\t\t\teech_rf_final_link ( n, 0, k, w->children[k] ? ( int ) ( w->children[k] - waypoint_nodes ) : -1 );\n",
            "\t\t}\n",
            "\t\tfor ( k = 0; k < w->number_of_parents && k < MAX_WAYPOINT_LINKS; k++ )\n",
            "\t\t{\n",
            "\t\t\teech_rf_final_link ( n, 1, k, w->parents[k] ? ( int ) ( w->parents[k] - waypoint_nodes ) : -1 );\n",
            "\t\t}\n",
            "\t}\n",
            "\tfor ( k = 0; k < number_of_waypoint_starting_nodes && k < MAX_WAYPOINT_STARTS; k++ )\n",
            "\t{\n",
            "\t\teech_rf_final_start ( k, waypoint_starting_nodes[k] ? ( int ) ( waypoint_starting_nodes[k] - waypoint_nodes ) : -1 );\n",
            "\t}\n",
            "\tfor ( k = 0; k < number_of_route_waypoint_positions; k++ )\n",
            "\t{\n",
            "\t\teech_rf_final_position ( k, route_waypoint_positions[k].number_of_positions, &route_waypoint_positions[k].position, route_waypoint_positions[k].offsets );\n",
            "\t}\n",
            "\teech_rf_end ();\n",
            "}\n",
            "\n",
            "void parse_waypoint_routes_from_object ( int object_index, int number_of_matching_slots, vec3d *slots )\n{\n"
        ),
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "R3-begin",
        why: "the call's arguments and the raw object it is about to read, right after the parser selects the object",
        original: "\tobject = &objects_3d_data[object_index];\n",
        replacement: "\tobject = &objects_3d_data[object_index];\n\teech_rf_begin ( object_index, number_of_matching_slots, slots, object ); /* EECH route fixture (R3) */\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "R4-node",
        why: "each node's raw stored point and the position the parser has just decoded from it",
        original: "\t\t\twaypoint_nodes[count].position.z *= zmax / 32767;\n",
        replacement: "\t\t\twaypoint_nodes[count].position.z *= zmax / 32767;\n\t\t\teech_rf_node ( count, &object->points[count], &waypoint_nodes[count].position ); /* EECH route fixture (R4) */\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "R5-face",
        why: "each line face as the parser's first pass has just read it (before the pointer advance)",
        original: "\t\t\t\twaypoint_nodes[point1].primary = TRUE;\n\t\t\t}\n\n\t\t\tpoint_refs += 2;\n",
        replacement: "\t\t\t\twaypoint_nodes[point1].primary = TRUE;\n\t\t\t}\n\n\t\t\teech_rf_face ( surface, count, point_refs[0].point, point_refs[1].point, point0, point1, &object->surfaces[surface], number_of_surface_points ); /* EECH route fixture (R5) */\n\t\t\tpoint_refs += 2;\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/routegen.c",
        id: "R6-end",
        why: "the end of the parse: the parser's tables are final",
        original: "\t*/\n}\n",
        replacement: "\t*/\n\n\teech_rf_observe_final ( maximum_tree_depth ); /* EECH route fixture (R6) */\n}\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/popread.c",
        id: "R7a-lookup-found",
        why: "a route object was found for the airfield instance (scene index) and sub-object kind",
        original: "\t\treturn ( search.object_scene );\n",
        replacement: "\t\t{\n\t\t\textern void eech_rf_lookup ( int scene, int kind, int found, int legacy_object, const void *entry ); /* EECH route fixture (R7a) */\n\t\t\teech_rf_lookup ( instance->object_number, ( int ) index, 1, ( int ) search.object_scene->index, search.object_scene );\n\t\t}\n\t\treturn ( search.object_scene );\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/faction/popread.c",
        id: "R7b-lookup-missing",
        why: "no route object exists for the airfield instance and sub-object kind",
        original: "\t\treturn ( NULL );\n#else\n\t\treturn 0;\n#endif\n",
        replacement: "\t\t{\n\t\t\textern void eech_rf_lookup ( int scene, int kind, int found, int legacy_object, const void *entry ); /* EECH route fixture (R7b) */\n\t\t\teech_rf_lookup ( instance->object_number, ( int ) index, 0, -1, NULL );\n\t\t}\n\t\treturn ( NULL );\n#else\n\t\treturn 0;\n#endif\n",
        count: 1,
    },
    // R8: the caller context. At each place popread takes the raw object number from the route entry it holds (`raw_object_index = <entry>->index;`) the entry and that number are handed
    // to the fixture, which pairs them with the parse call that follows. The entry is the value the original caller itself selected (the result of its own lookup for a scene and kind);
    // the number is the argument the parse will receive. Neither is recomputed, and no statement, argument, order or value of the original changes.
    caller_patch!("fixedwing_landing_route", 1),
    caller_patch!("fixedwing_takeoff_route", 1),
    caller_patch!("fixedwing_landing_holding_route", 1),
    caller_patch!("fixedwing_takeoff_holding_route", 1),
    caller_patch!("helicopter_landing_route", 1),
    caller_patch!("helicopter_takeoff_route", 1),
    caller_patch!("helicopter_landing_holding_route", 1),
    caller_patch!("helicopter_takeoff_holding_route", 1),
    caller_patch!("vehicle_landing_route", 1),
    caller_patch!("vehicle_takeoff_route", 1),
    caller_patch!("vehicle_landing_holding_route", 1),
    caller_patch!("vehicle_takeoff_holding_route", 1),
    caller_patch!("ship_landing_route", 1),
    caller_patch!("ship_takeoff_route", 1),
    caller_patch!("ship_landing_holding_route", 1),
    caller_patch!("ship_takeoff_holding_route", 1),
    // the helper functions that receive the entry as a parameter
    caller_patch!("landing_route", 3),
    caller_patch!("takeoff_route", 3),
    caller_patch!("landing_holding_route", 2),
    caller_patch!("takeoff_holding_route", 2),
    // the two troop-route calls pass the entry inline: the hook is the statement just before the parse call
    Patch {
        file: "aphavoc/source/ai/faction/popread.c",
        id: "R8-caller-troop-routes",
        why: "the troop landing and take-off routes are parsed straight from the entry the caller holds in `route`: `route->index` is evaluated once, as the helper's argument, and the helper returns that same value as the parser's argument",
        original: "parse_waypoint_routes_from_object ( route->index, 0, NULL );\n",
        replacement: "{\n\t\t\t\t\textern int eech_rf_caller_index ( const void *entry, int raw_object_index ); /* EECH route fixture (R8) */\n\t\t\t\t\tparse_waypoint_routes_from_object ( eech_rf_caller_index ( route, route->index ), 0, NULL );\n\t\t\t\t}\n",
        count: 2,
    },
];

/// In the HOST route profile the parser no longer reads the campaign's model arrays, so only the patches that hook text the host profile leaves alone apply: the hooks include,
/// the final-table observer and the end of the parse. The caller context and the lookup come from the host profile itself (campaign_model_route_*), and the begin / node /
/// face observations are made where the host's answers are taken.
fn candidate_profile(id: &str) -> bool {
    matches!(id, "R1-hooks-include" | "R2-final-observer" | "R6-end")
}

/// the staged text of `file` with the route-fixture patches applied (same strict count check as `apply`); `original` selects the whole set for the original route parser, or the
/// subset of the host route profile
pub fn apply_route_fixture(file: &str, text: &str, original: bool) -> String {
    let mut out = text.to_string();
    for p in ROUTE_FIXTURE_PATCHES.iter().filter(|p| p.file == file && (original || candidate_profile(p.id))) {
        let found = out.matches(p.original).count();
        assert_eq!(
            found, p.count,
            "route-fixture patch {} expects {} occurrence(s) of its original text in {}, found {}",
            p.id, p.count, file, found
        );
        out = out.replace(p.original, p.replacement);
    }
    out
}

pub fn route_fixture_files(original: bool) -> Vec<&'static str> {
    let mut files: Vec<&str> = ROUTE_FIXTURE_PATCHES
        .iter()
        .filter(|p| original || candidate_profile(p.id))
        .map(|p| p.file)
        .collect();
    files.sort_unstable();
    files.dedup();
    files
}

/// TEST-ONLY patches of the `target-point-fixture` feature (EEDCS-52, D3): observation hooks in the ORIGINAL `get_local_entity_target_point` (en_maths.c), the original
/// `initialise_target_point_offsets` (wn_dbase.c) and the two campaign callers that reach the query (weapon.c `fill_aim_target`, wn_move.c the weapon step's target-vector
/// lookup), applied by main.rs ONLY when that feature is on, to the staged copies (none of the four files has a shipping patch). They record values the original statements
/// already read or computed (csrc/eech_tp_hooks.h, csrc/eech_tp_fixture.c); each ADDS a call and changes no original statement, argument, order or value. With the feature off none of
/// this exists in the build and the in-tree sources are never edited. Same strict count check as PATCHES.
pub const TARGET_POINT_FIXTURE_PATCHES: &[Patch] = &[
    Patch {
        file: "aphavoc/source/entity/en_misc/en_maths/en_maths.c",
        id: "TP1-hooks-include",
        why: "declares the observer hooks (csrc/eech_tp_hooks.h)",
        original: "#include \"project.h\"\n",
        replacement: "#include \"project.h\"\n#include \"eech_tp_hooks.h\" /* EECH target-point fixture (TP1) */\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/en_misc/en_maths/en_maths.c",
        id: "TP2-begin",
        why: "the query begins, right after the original read the entity's 3D instance",
        original: "\tinst3d = (object_3d_instance *) get_local_entity_ptr_value (en, PTR_TYPE_INSTANCE_3D_OBJECT);\n",
        replacement: "\tinst3d = (object_3d_instance *) get_local_entity_ptr_value (en, PTR_TYPE_INSTANCE_3D_OBJECT);\n\teech_tp_begin ( en, inst3d ); /* EECH target-point fixture (TP2) */\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/en_misc/en_maths/en_maths.c",
        id: "TP3-instance-target-point",
        why: "branch 1: the instance holds a TARGET_POINT sub-object; the hook runs before the original overwrites the instance viewpoint from the entity pose",
        original: "\t\t\tget_local_entity_vec3d (en, VEC3D_TYPE_POSITION, (vec3d *) &inst3d->vp.position);\n",
        replacement: "\t\t\teech_tp_instance_target_point (); /* EECH target-point fixture (TP3) */\n\t\t\tget_local_entity_vec3d (en, VEC3D_TYPE_POSITION, (vec3d *) &inst3d->vp.position);\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/en_misc/en_maths/en_maths.c",
        id: "TP4-instance-position",
        why: "branch 2: the instance holds no TARGET_POINT sub-object and the original answers the entity position",
        original: "\t\telse\n\t\t{\n\t\t\tget_local_entity_vec3d (en, VEC3D_TYPE_POSITION, target_point);\n",
        replacement: "\t\telse\n\t\t{\n\t\t\teech_tp_instance_position (); /* EECH target-point fixture (TP4) */\n\t\t\tget_local_entity_vec3d (en, VEC3D_TYPE_POSITION, target_point);\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/en_misc/en_maths/en_maths.c",
        id: "TP5-fixed",
        why: "no instance, and the entity is a fixed object",
        original: "\t\tif (get_local_entity_int_value (en, INT_TYPE_IDENTIFY_FIXED))\n\t\t{\n",
        replacement: "\t\tif (get_local_entity_int_value (en, INT_TYPE_IDENTIFY_FIXED))\n\t\t{\n\t\t\teech_tp_fixed (); /* EECH target-point fixture (TP5) */\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/en_misc/en_maths/en_maths.c",
        id: "TP6-shape",
        why: "the shape index the original has just read for a fixed object",
        original: "\t\t\tindex = get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);\n",
        replacement: "\t\t\tindex = get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);\n\t\t\teech_tp_shape ( index ); /* EECH target-point fixture (TP6) */\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/en_misc/en_maths/en_maths.c",
        id: "TP7-horizontal",
        why: "the shape's offset has a non-zero x or z (the original's own test, entered)",
        original: "\t\t\t\tif ((target_point_offsets[index].x != 0.0) || (target_point_offsets[index].z != 0.0))\n\t\t\t\t{\n",
        replacement: "\t\t\t\tif ((target_point_offsets[index].x != 0.0) || (target_point_offsets[index].z != 0.0))\n\t\t\t\t{\n\t\t\t\t\teech_tp_horizontal (); /* EECH target-point fixture (TP7) */\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/en_misc/en_maths/en_maths.c",
        id: "TP8-heading",
        why: "the heading the original has just read; the hook takes it by pointer and copies its bit pattern",
        original: "\t\t\t\t\theading = get_local_entity_float_value (en, FLOAT_TYPE_HEADING);\n",
        replacement: "\t\t\t\t\theading = get_local_entity_float_value (en, FLOAT_TYPE_HEADING);\n\t\t\t\t\teech_tp_heading ( &heading ); /* EECH target-point fixture (TP8) */\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/en_misc/en_maths/en_maths.c",
        id: "TP9-xz-applied",
        why: "the original's own `heading != 0.0` test was entered: it is about to add the rotated x/z offsets (a heading of zero never gets here, the legacy quirk)",
        original: "\t\t\t\t\t\tsin_heading = sin (heading);\n",
        replacement: "\t\t\t\t\t\teech_tp_xz_applied (); /* EECH target-point fixture (TP9) */\n\t\t\t\t\t\tsin_heading = sin (heading);\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/en_misc/en_maths/en_maths.c",
        id: "TP10-end",
        why: "the query ends: the target point it returned (the function has no earlier return)",
        original: "\t\t\t\ttarget_point->y += target_point_offsets[index].y;\n\t\t\t}\n\t\t}\n\t}\n}\n",
        replacement: "\t\t\t\ttarget_point->y += target_point_offsets[index].y;\n\t\t\t}\n\t\t}\n\t}\n\n\teech_tp_end ( target_point ); /* EECH target-point fixture (TP10) */\n}\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/mobile/weapon/wn_dbase.c",
        id: "TP11-hooks-include",
        why: "declares the observer hooks (csrc/eech_tp_hooks.h)",
        original: "#include \"project.h\"\n",
        replacement: "#include \"project.h\"\n#include \"eech_tp_hooks.h\" /* EECH target-point fixture (TP11) */\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/mobile/weapon/wn_dbase.c",
        id: "TP12-census-begin",
        why: "the census begins; the hook sits before calculate_seeker_limits_from_launch_angle, which is a separate campaign weapon-policy calculation that keeps its order and single execution",
        original: "\tdebug_assert (get_3d_objects_initialised ());\n\n\tcalculate_seeker_limits_from_launch_angle();\n",
        replacement: "\tdebug_assert (get_3d_objects_initialised ());\n\n\teech_tp_census_begin (); /* EECH target-point fixture (TP12) */\n\n\tcalculate_seeker_limits_from_launch_angle();\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/mobile/weapon/wn_dbase.c",
        id: "TP13-census-invalid",
        why: "the table's entry for the invalid object index, as the original has just zeroed it",
        original: "\ttarget_point_offsets[OBJECT_3D_INVALID_OBJECT_INDEX].z = 0.0;\n",
        replacement: "\ttarget_point_offsets[OBJECT_3D_INVALID_OBJECT_INDEX].z = 0.0;\n\teech_tp_census_invalid ( &target_point_offsets[OBJECT_3D_INVALID_OBJECT_INDEX] ); /* EECH target-point fixture (TP13) */\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/mobile/weapon/wn_dbase.c",
        id: "TP14-census-sub-object",
        why: "a model's entry taken from its TARGET_POINT sub-object's world position",
        original: "\t\t\ttarget_point_offsets[object_3d_index] = vp.position;\n",
        replacement: "\t\t\ttarget_point_offsets[object_3d_index] = vp.position;\n\t\t\teech_tp_census_sub_object ( object_3d_index, &target_point_offsets[object_3d_index] ); /* EECH target-point fixture (TP14) */\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/mobile/weapon/wn_dbase.c",
        id: "TP15-census-bbox",
        why: "a model's entry derived from its bounding box (y = max (centre y, 0)); the hook also reads the box the original used",
        original: "\t\t\ttarget_point_offsets[object_3d_index].z = (bounding_box->zmin + bounding_box->zmax) * 0.5;\n",
        replacement: "\t\t\ttarget_point_offsets[object_3d_index].z = (bounding_box->zmin + bounding_box->zmax) * 0.5;\n\t\t\teech_tp_census_bbox ( object_3d_index, &target_point_offsets[object_3d_index], bounding_box ); /* EECH target-point fixture (TP15) */\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/mobile/weapon/wn_dbase.c",
        id: "TP16-census-end",
        why: "the census ends after the last model",
        original: "\t\tdestruct_3d_object (inst3d);\n\t}\n}\n",
        replacement: "\t\tdestruct_3d_object (inst3d);\n\t}\n\n\teech_tp_census_end (); /* EECH target-point fixture (TP16) */\n}\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/mobile/weapon/weapon.c",
        id: "TP17-hooks-include",
        why: "declares the observer hooks (csrc/eech_tp_hooks.h)",
        original: "#include \"entity/mobile/mb_aimfx.h\"\n",
        replacement: "#include \"entity/mobile/mb_aimfx.h\"\n#include \"eech_tp_hooks.h\" /* EECH target-point fixture (TP17) */\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/mobile/weapon/weapon.c",
        id: "TP18-caller-aim",
        why: "the caller context of fill_aim_target: the aim step's mode (vectors, lead or intercept) says which host aim operation asked",
        original: "\tget_local_entity_target_point (target, &point);\n",
        replacement: "\teech_tp_caller ( 1, step->mode ); /* EECH target-point fixture (TP18) */\n\tget_local_entity_target_point (target, &point);\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/mobile/weapon/wn_move.c",
        id: "TP19-hooks-include",
        why: "declares the observer hooks (csrc/eech_tp_hooks.h)",
        original: "#include \"project.h\"\n",
        replacement: "#include \"project.h\"\n#include \"eech_tp_hooks.h\" /* EECH target-point fixture (TP19) */\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/mobile/weapon/wn_move.c",
        id: "TP20-caller-weapon-step",
        why: "the caller context of the weapon step's target-vector lookup (question 1, the target point)",
        original: "\t\t\t\t\tget_local_entity_target_point (target, &p);\n",
        replacement: "\t\t\t\t\teech_tp_caller ( 2, lookup->b ); /* EECH target-point fixture (TP20) */\n\t\t\t\t\tget_local_entity_target_point (target, &p);\n",
        count: 1,
    },
];

/// the staged text of `file` with the target-point-fixture patches applied (same strict count check as `apply`)
pub fn apply_target_point_fixture(file: &str, text: &str) -> String {
    let mut out = text.to_string();
    for p in TARGET_POINT_FIXTURE_PATCHES.iter().filter(|p| p.file == file) {
        let found = out.matches(p.original).count();
        assert_eq!(
            found, p.count,
            "target-point-fixture patch {} expects {} occurrence(s) of its original text in {}, found {}",
            p.id, p.count, file, found
        );
        out = out.replace(p.original, p.replacement);
    }
    out
}

pub fn target_point_fixture_files() -> Vec<&'static str> {
    let mut files: Vec<&str> = TARGET_POINT_FIXTURE_PATCHES.iter().map(|p| p.file).collect();
    files.sort_unstable();
    files.dedup();
    files
}
