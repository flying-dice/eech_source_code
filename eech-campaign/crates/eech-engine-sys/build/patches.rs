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
