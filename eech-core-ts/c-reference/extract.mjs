#!/usr/bin/env node
//
// Generates the C reference harness sources from the original EECH C:
//
//   build/c-reference/project.h        the reduced project.h that the original
//                                       translation units include: the harness
//                                       environment, verbatim fragments and
//                                       WHOLE original headers
//   build/c-reference/eech_extracted*.c verbatim copies of original functions and
//                                       definitions that live in files too large
//                                       to compile whole (one generated file per
//                                       group of extracts that share a C scope;
//                                       see EXTRACTED_UNITS)
//
// Original translation units that are compiled unchanged are listed in
// REAL_TRANSLATION_UNITS; build.mjs compiles them against the generated
// project.h.
//
// Each fragment is preceded by a #line directive to its origin. Extraction
// fails loudly if an original definition cannot be found.
//

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(here, "..");
const repoRoot = join(projectRoot, "..");

export const AP_SOURCE = "aphavoc/source";

// Original EECH translation units compiled unchanged into the harness.
export const REAL_TRANSLATION_UNITS = [
	"aphavoc/source/entity/special/group/gp_dbase.c",
	"aphavoc/source/entity/special/group/gp_float.c",
	"aphavoc/source/entity/special/group/gp_int.c",
	"aphavoc/source/entity/special/group/gp_list.c",
	"aphavoc/source/entity/special/group/gp_ptr.c",
	"aphavoc/source/entity/special/group/gp_updt.c",
	"aphavoc/source/entity/special/group/gp_vec3d.c",
	"aphavoc/source/entity/special/update/up_list.c",
	"aphavoc/source/entity/special/update/up_msgs.c",
	// slice 3: keysite and force accessors (replace the hand-written rows)
	"aphavoc/source/entity/special/keysite/ks_dbase.c",
	"aphavoc/source/entity/special/keysite/ks_float.c",
	"aphavoc/source/entity/special/keysite/ks_int.c",
	"aphavoc/source/entity/special/keysite/ks_list.c",
	"aphavoc/source/entity/special/keysite/ks_vec3d.c",
	"aphavoc/source/entity/special/force/fc_int.c",
	"aphavoc/source/entity/special/force/fc_list.c",
	// slice 3: entity heap, attributes, creation and destruction
	"aphavoc/source/entity/system/en_main/en_heap.c",
	"aphavoc/source/entity/system/en_attrs/en_attrs.c",
	"aphavoc/source/entity/system/en_funcs/en_creat.c",
	"aphavoc/source/entity/system/en_funcs/en_dstry.c",
	// slice 3: mobile, cargo and sector
	"aphavoc/source/entity/mobile/mb_int.c",
	"aphavoc/source/entity/mobile/mb_list.c",
	"aphavoc/source/entity/mobile/mb_vec3d.c",
	"aphavoc/source/entity/mobile/cargo/cg_creat.c",
	"aphavoc/source/entity/mobile/cargo/cg_dstry.c",
	"aphavoc/source/entity/mobile/cargo/cg_int.c",
	"aphavoc/source/entity/mobile/cargo/cg_list.c",
	"aphavoc/source/entity/special/sector/sc_int.c",
	"aphavoc/source/entity/special/sector/sc_list.c",
	"aphavoc/source/entity/special/sector/sc_msgs.c",
	"aphavoc/source/entity/special/sector/sc_seccreat.c",
];

// Contents of the generated project.h, in order.
// kind "include": a whole original header (path relative to aphavoc/source)
export const PROJECT_H = [
	{ kind: "local", name: "eech_harness_env.h" },
	{ kind: "struct", name: "VEC3D", file: "modules/maths/vector.h" },
	{ kind: "regex", pattern: "\\ntypedef struct VEC3D vec3d;", file: "modules/maths/vector.h" },
	{ kind: "define", name: "bound", file: "modules/maths/miscmath.h" },
	{ kind: "define", name: "METRE", file: "modules/maths/constant.h" },
	{ kind: "define", name: "KILOMETRE", file: "modules/maths/constant.h" },
	{ kind: "regex", pattern: "\\nextern float\\n\\tsystem_delta_time,\\n\\tsystem_one_over_delta_time;", file: "modules/system/time.h" },
	{ kind: "regex", pattern: "\\nextern int\\n\\tlocked_frame_rate;", file: "modules/system/time.h" },
	{ kind: "regex", pattern: "\\nextern void set_manual_delta_time \\(float delta_time\\);", file: "modules/system/time.h" },
	{ kind: "define", name: "get_delta_time", file: "modules/system/time.h" },
	{ kind: "enum", name: "COMMS_MODEL_TYPES", file: "aphavoc/source/comms/comms.h" },
	{ kind: "regex", pattern: "\\ntypedef enum COMMS_MODEL_TYPES comms_model_types;", file: "aphavoc/source/comms/comms.h" },
	{ kind: "enum", name: "COMMS_DATA_FLOW_TYPES", file: "aphavoc/source/comms/comms.h" },
	{ kind: "regex", pattern: "\\ntypedef enum COMMS_DATA_FLOW_TYPES comms_data_flow_types;", file: "aphavoc/source/comms/comms.h" },
	{ kind: "regex", pattern: "\\nextern comms_model_types\\n\\s*system_comms_model;", file: "aphavoc/source/comms/comms.h" },
	{ kind: "regex", pattern: "\\nextern comms_data_flow_types\\n\\s*system_comms_data_flow;", file: "aphavoc/source/comms/comms.h" },
	{ kind: "define", name: "get_comms_model", file: "aphavoc/source/comms/comms.h" },
	{ kind: "define", name: "get_comms_data_flow", file: "aphavoc/source/comms/comms.h" },
	{ kind: "enum", name: "MAP_LAYER_CONTROL_TYPES", file: "aphavoc/source/ui_menu/ingame/common/map.h" },
	{ kind: "define", name: "MAP_LAYER_CONTROL_NONE", file: "aphavoc/source/ui_menu/ingame/common/map.h" },
	{ kind: "enum", name: "MAP_ICON_TYPE", file: "aphavoc/source/ui_menu/ingame/common/map.h" },
	{ kind: "include", name: "entity/system/en_types/en_types.h" },
	{ kind: "include", name: "entity/system/en_main/en_heap.h" },
	{ kind: "include", name: "entity/system/en_msgs/en_msgs.h" },
	{ kind: "include", name: "entity/system/en_funcs/en_funcs.h" },
	{ kind: "include", name: "entity/system/en_comms/en_comms.h" },
	{ kind: "include", name: "entity/system/en_debug/en_dbmsg.h" },
	{ kind: "include", name: "ai_extrn.h" },
	{ kind: "include", name: "entity/special/update/up_update.h" },
	{ kind: "include", name: "entity/special/division/division.h" },
	{ kind: "include", name: "entity/special/group/group.h" },
	{ kind: "include", name: "entity/tacview/tacview.h" },
	// engine types the slice 3 headers name: verbatim where the headers use
	// them by value, typedef-only forward declarations where only pointers occur
	{ kind: "regex", pattern: "\\ntypedef struct OBJECT_3D_INSTANCE object_3d_instance;", file: "modules/3d/objects.h" },
	{ kind: "struct", name: "OBJECT_3D_BOUNDS", file: "modules/3d/objects.h" },
	{ kind: "regex", pattern: "\\ntypedef struct OBJECT_3D_BOUNDS object_3d_bounds;", file: "modules/3d/objects.h" },
	{ kind: "regex", pattern: "\\ntypedef int terrain_types;", file: "modules/3d/terrain/terrtype.h" },
	{ kind: "regex", pattern: "\\ntypedef struct TERRAIN_3D_POINT_WORD_REFERENCE terrain_3d_point_word_reference;", file: "modules/3d/terrain/terrdata.h" },
	{ kind: "regex", pattern: "\\ntypedef struct TERRAIN_3D_POINT_BYTE_REFERENCE terrain_3d_point_byte_reference;", file: "modules/3d/terrain/terrdata.h" },
	{ kind: "regex", pattern: "\\ntypedef struct TERRAIN_3D_FACE terrain_3d_face;", file: "modules/3d/terrain/terrdata.h" },
	{ kind: "regex", pattern: "\\ntypedef struct TERRAIN_3D_SECTOR terrain_3d_sector;", file: "modules/3d/terrain/terrdata.h" },
	{ kind: "struct", name: "TERRAIN_3D_POINT_DATA", file: "modules/3d/terrain/terrdata.h" },
	{ kind: "regex", pattern: "\\ntypedef struct TERRAIN_3D_POINT_DATA terrain_3d_point_data;", file: "modules/3d/terrain/terrdata.h" },
	{ kind: "struct", name: "DOUBLE_VEC3D", file: "modules/maths/vector.h" },
	{ kind: "regex", pattern: "\\ntypedef struct DOUBLE_VEC3D double_vec3d;", file: "modules/maths/vector.h" },
	{ kind: "regex", pattern: "\\ntypedef struct EVENT event;", file: "modules/system/event.h" },
	{ kind: "enum", name: "DEBUG_COLOURS", file: "modules/system/debug.h" },
	{ kind: "regex", pattern: "\\nextern void debug_colour_log \\( enum DEBUG_COLOURS colour, const char \\*string, \\.\\.\\. \\);", file: "modules/system/debug.h" },
	{ kind: "prototype", name: "get_identity_matrix3x3", file: "modules/maths/matrix.h" },
	{ kind: "regex", pattern: "\\ntypedef int sound_sample_indices;", file: "aphavoc/source/appsound/snd_data.h" },
	{ kind: "enum", name: "GAME_STATUS_TYPES", file: "aphavoc/source/global.h" },
	{ kind: "regex", pattern: "\\ntypedef enum GAME_STATUS_TYPES game_status_types;", file: "aphavoc/source/global.h" },
	{ kind: "enum", name: "GUNSHIP_TYPES", file: "aphavoc/source/global.h" },
	{ kind: "regex", pattern: "\\ntypedef enum GUNSHIP_TYPES gunship_types;", file: "aphavoc/source/global.h" },
	{ kind: "enum", name: "GAME_DIFFICULTY_SETTINGS", file: "aphavoc/source/global.h" },
	{ kind: "enum", name: "WEATHERMODES", file: "modules/3d/3denv.h" },
	{ kind: "enum", name: "SESSION_LIST_TYPES", file: "aphavoc/source/ui_menu/session/session.h" },
	{ kind: "regex", pattern: "\\ntypedef enum SESSION_LIST_TYPES session_list_types;", file: "aphavoc/source/ui_menu/session/session.h" },
	{ kind: "prototype", name: "int_bit_count", file: "aphavoc/source/misc/miscell.h" },
	{ kind: "include", name: "misc/listitem.h" },
	{ kind: "include", name: "3d/3dmodels.h" },
	{ kind: "include", name: "3d/textanim.h" },
	// slice 3: whole original headers
	{ kind: "include", name: "entity/special/keysite/keysite.h" },
	{ kind: "include", name: "entity/special/force/force.h" },
	{ kind: "include", name: "entity/system/en_main/en_world.h" },
	{ kind: "include", name: "entity/system/en_attrs/en_attrs.h" },
	{ kind: "include", name: "entity/mobile/mobile.h" },
	{ kind: "include", name: "entity/special/sector/sector.h" },
	{ kind: "include", name: "entity/system/en_debug/en_stats.h" },
	// prototypes of functions the slice 3 translation units call; the harness
	// supplies them as environment or fail-loud stubs (harness.c)
	{ kind: "prototype", name: "convert_float_to_int", file: "modules/system/fpu.h" },
	{ kind: "prototype", name: "set_comms_data_flow", file: "aphavoc/source/comms/comms.h" },
	{ kind: "prototype", name: "set_comms_model", file: "aphavoc/source/comms/comms.h" },
	{ kind: "prototype", name: "update_imap_surface_to_air_defence_level", file: "aphavoc/source/ai/highlevl/imaps.h" },
	{ kind: "prototype", name: "update_imap_surface_to_surface_defence_level", file: "aphavoc/source/ai/highlevl/imaps.h" },
	{ kind: "prototype", name: "get_local_group_member_landing_entity_from_keysite", file: "aphavoc/source/entity/special/landing/landing.h" },
	{ kind: "prototype", name: "destroy_local_sound_effects", file: "aphavoc/source/entity/special/effect/soundeff/soundeff.h" },
	{ kind: "prototype", name: "destroy_client_server_sound_effects", file: "aphavoc/source/entity/special/effect/soundeff/soundeff.h" },
	{ kind: "prototype", name: "create_local_camera_entity", file: "aphavoc/source/entity/special/camera/cm_creat.h" },
	{ kind: "prototype", name: "destroy_local_camera_entity", file: "aphavoc/source/entity/special/camera/cm_dstry.h" },
	{ kind: "prototype", name: "create_local_bridge_entities", file: "aphavoc/source/entity/special/bridge/bridge.h" },
	{ kind: "prototype", name: "create_local_pylon_entities", file: "aphavoc/source/entity/fixed/pylon/py_creat.h" },
	{ kind: "prototype", name: "destroy_local_pylon_entities", file: "aphavoc/source/entity/fixed/pylon/py_dstry.h" },
	{ kind: "prototype", name: "get_object_3d_bounding_box", file: "modules/3d/3dobjvis.h" },
	// en_debug/en_valid.h: the create index checks (debug-build ASSERTs)
	{ kind: "prototype", name: "assert_local_create_entity_index", file: "aphavoc/source/entity/system/en_debug/en_valid.h" },
	{ kind: "prototype", name: "assert_remote_create_entity_index", file: "aphavoc/source/entity/system/en_debug/en_valid.h" },
	{ kind: "define", name: "validate_local_create_entity_index", file: "aphavoc/source/entity/system/en_debug/en_valid.h" },
	{ kind: "define", name: "validate_remote_create_entity_index", file: "aphavoc/source/entity/system/en_debug/en_valid.h" },
	{ kind: "regex", pattern: "\\nextern entity\\n\\t\\*session_entity;", file: "aphavoc/source/entity/special/session/session.h" },
	{ kind: "define", name: "get_session_entity", file: "aphavoc/source/entity/special/session/session.h" },
	{ kind: "local", name: "eech_harness_decls.h" },
];

// Contents of the generated eech_extracted.c, in order.
export const EXTRACTED_C = [
	{ kind: "raw", text: '#include "project.h"\n\n#define DEBUG_MODULE 0\n\n#define LANDING_DEBUG 0\n' },

	// modules/system/time.c: the delta time the entity update loop reads and overrides
	{ kind: "define", name: "DELTA_TIME_HISTORY_SIZE", file: "modules/system/time.c" },
	{ kind: "regex", pattern: "\\nfloat\\n\\tsystem_delta_time_average = 0\\.1,\\n\\tsystem_delta_time = 0\\.1,\\n\\tsystem_one_over_delta_time = 10\\.0;", file: "modules/system/time.c" },
	{ kind: "regex", pattern: "\\nint\\n\\tlocked_frame_rate = FALSE;", file: "modules/system/time.c" },
	{ kind: "regex", pattern: "\\nstatic float\\n\\tsystem_delta_time_history\\[DELTA_TIME_HISTORY_SIZE\\];", file: "modules/system/time.c" },
	{ kind: "regex", pattern: "\\nstatic int\\n\\tsystem_delta_time_history_position;", file: "modules/system/time.c" },
	{ kind: "function", name: "set_manual_delta_time", signature: "void set_manual_delta_time (float delta_time)", file: "modules/system/time.c" },

	// modules/maths/range.c
	{ kind: "function", name: "get_2d_range", signature: "float get_2d_range (const vec3d *v1, const vec3d *v2)", file: "modules/maths/range.c" },
	{ kind: "function", name: "get_approx_2d_range", signature: "float get_approx_2d_range (const vec3d *v1, const vec3d *v2)", file: "modules/maths/range.c" },

	// entity system defaults and runtime
	{ kind: "function", name: "default_set_entity_int_value", signature: "static void default_set_entity_int_value (entity *en, int_types type, int value)", file: "aphavoc/source/entity/system/en_funcs/en_int.c" },
	{ kind: "function", name: "default_set_entity_float_value", signature: "static void default_set_entity_float_value (entity *en, float_types type, float value)", file: "aphavoc/source/entity/system/en_funcs/en_float.c" },
	{ kind: "function", name: "notify_local_entity", signature: "int notify_local_entity (entity_messages message, entity *receiver, entity *sender, ...)", file: "aphavoc/source/entity/system/en_msgs/en_msgs.c" },
	{ kind: "function", name: "default_message_response", signature: "static int default_message_response (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/system/en_msgs/en_msgs.c" },
	{ kind: "function", name: "insert_local_entity_into_parents_child_list", signature: "void insert_local_entity_into_parents_child_list (entity *en, list_types type, entity *parent, entity *pred)", file: "aphavoc/source/entity/system/en_funcs/en_list.c" },
	{ kind: "function", name: "delete_local_entity_from_parents_child_list", signature: "void delete_local_entity_from_parents_child_list (entity *en, list_types type)", file: "aphavoc/source/entity/system/en_funcs/en_list.c" },
	{ kind: "function", name: "unlink_local_entity_children", signature: "void unlink_local_entity_children (entity *en, list_types list)", file: "aphavoc/source/entity/system/en_funcs/en_list.c" },
	{ kind: "function", name: "default_get_entity_int_value", signature: "static int default_get_entity_int_value (entity *en, int_types type)", file: "aphavoc/source/entity/system/en_funcs/en_int.c" },

	// slice 3: entity system debug checks and statistics counters
	{ kind: "function", name: "assert_local_create_entity_index", signature: "int assert_local_create_entity_index (int index)", file: "aphavoc/source/entity/system/en_debug/en_valid.c" },
	{ kind: "function", name: "assert_remote_create_entity_index", signature: "int assert_remote_create_entity_index (int index)", file: "aphavoc/source/entity/system/en_debug/en_valid.c" },
	{ kind: "regex", pattern: "\\nstatic int\\n\\tentity_count,\\n\\tentity_peak_count;", file: "aphavoc/source/entity/system/en_debug/en_stats.c" },
	{ kind: "regex", pattern: "\\nstatic struct\\n\\{[^}]*\\}\\nentity_type_stats\\[NUM_ENTITY_TYPES\\];", file: "aphavoc/source/entity/system/en_debug/en_stats.c" },
	{ kind: "function", name: "update_create_entity_statistics", signature: "void update_create_entity_statistics (entity_types type)", file: "aphavoc/source/entity/system/en_debug/en_stats.c" },
	{ kind: "function", name: "update_destroy_entity_statistics", signature: "void update_destroy_entity_statistics (entity *en)", file: "aphavoc/source/entity/system/en_debug/en_stats.c" },

	// slice 3: world map, sector lookup and sector mobile values
	{ kind: "regex", pattern: "\\nworld_map_data\\n\\tworld_map;", file: "aphavoc/source/entity/system/en_main/en_world.c" },
	{ kind: "function", name: "set_entity_world_map_size", signature: "void set_entity_world_map_size (int num_map_x_sectors, int num_map_z_sectors, int sector_side_length)", file: "aphavoc/source/entity/system/en_main/en_world.c" },
	{ kind: "function", name: "int_bit_count", signature: "int int_bit_count (unsigned int value)", file: "aphavoc/source/misc/miscell.c" },
	{ kind: "function", name: "get_identity_matrix3x3", signature: "void get_identity_matrix3x3 (matrix3x3 m)", file: "modules/maths/matrix.c" },
	{ kind: "regex", pattern: "\\nentity\\n\\t\\*\\*entity_sector_map = NULL;", file: "aphavoc/source/entity/special/sector/sector.c" },
	{ kind: "function", name: "get_local_sector_entity", signature: "entity *get_local_sector_entity (vec3d *pos)", file: "aphavoc/source/entity/special/sector/sector.c" },
	{ kind: "function", name: "add_mobile_values_to_sector", signature: "void add_mobile_values_to_sector (entity *sector_en, entity *mobile_en)", file: "aphavoc/source/entity/special/sector/sector.c" },
	{ kind: "function", name: "remove_mobile_values_from_sector", signature: "void remove_mobile_values_from_sector (entity *sector_en, entity *mobile_en)", file: "aphavoc/source/entity/special/sector/sector.c" },

	// slice 3: sound effects of a destroyed family (soundeff.c is otherwise not compiled)
	{ kind: "function", name: "destroy_client_server_sound_effects", signature: "void destroy_client_server_sound_effects (entity *en)", file: "aphavoc/source/entity/special/effect/soundeff/soundeff.c" },

	// entity/special/update/up_update.c: the entity update loop
	{ kind: "regex", pattern: "\\nentity\\n\\t\\*update_entity = NULL,\\n\\t\\*update_succ = NULL;", file: "aphavoc/source/entity/special/update/up_update.c" },
	{ kind: "regex", pattern: "\\nunsigned int\\n\\tmoved_entities[^;]*;", file: "aphavoc/source/entity/special/update/up_update.c" },
	{ kind: "function", name: "update_client_server_entities", signature: "void update_client_server_entities (void)", file: "aphavoc/source/entity/special/update/up_update.c" },
	{ kind: "function", name: "set_entity_update_frame_rate", signature: "int set_entity_update_frame_rate (int frame_rate)", file: "aphavoc/source/entity/special/update/up_update.c" },

	// campaign functions (slice 1)
	{ kind: "function", name: "get_local_force_entity", signature: "entity *get_local_force_entity (entity_sides side)", file: "aphavoc/source/entity/special/force/force.c" },
	{
		kind: "function",
		name: "get_closest_keysite",
		signature: "entity *get_closest_keysite (entity_sub_types type, entity_sides side, vec3d *pos, float min_range, float *actual_range, int outside_of_range, entity *exclude_keysite)",
		file: "aphavoc/source/entity/special/keysite/keysite.c",
	},
	{ kind: "function", name: "assess_group_supplies", signature: "void assess_group_supplies (entity *en)", file: "aphavoc/source/entity/special/group/group.c" },

	// group link/unlink parent responses (gp_msgs.c is otherwise not compiled)
	{ kind: "function", name: "response_to_link_parent", signature: "static int response_to_link_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/special/group/gp_msgs.c" },
	{ kind: "function", name: "response_to_unlink_parent", signature: "static int response_to_unlink_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/special/group/gp_msgs.c" },
	{
		kind: "wrap",
		prologue: "void harness_overload_group_link_parent_responses (void)\n{",
		epilogue: "}",
		parts: [
			{ kind: "regex", pattern: "\\n\\tmessage_responses\\[ENTITY_TYPE_GROUP\\]\\[ENTITY_MESSAGE_LINK_PARENT\\][^;]*;", file: "aphavoc/source/entity/special/group/gp_msgs.c" },
			{ kind: "regex", pattern: "\\n\\tmessage_responses\\[ENTITY_TYPE_GROUP\\]\\[ENTITY_MESSAGE_UNLINK_PARENT\\][^;]*;", file: "aphavoc/source/entity/special/group/gp_msgs.c" },
		],
	},
	{
		kind: "raw",
		text: "/* C defaults for the update entity: its link responses are only overloaded under DEBUG_MODULE (up_msgs.c) */\nvoid harness_default_update_link_responses (void)\n{\n\tmessage_responses[ENTITY_TYPE_UPDATE][ENTITY_MESSAGE_LINK_CHILD] = default_message_response;\n\tmessage_responses[ENTITY_TYPE_UPDATE][ENTITY_MESSAGE_LINK_PARENT] = default_message_response;\n\tmessage_responses[ENTITY_TYPE_UPDATE][ENTITY_MESSAGE_UNLINK_PARENT] = default_message_response;\n}\n\nvoid (*harness_default_set_entity_int_value) (entity *en, int_types type, int value) = default_set_entity_int_value;\n\nvoid (*harness_default_set_entity_float_value) (entity *en, float_types type, float value) = default_set_entity_float_value;\n\nint (*harness_default_get_entity_int_value) (entity *en, int_types type) = default_get_entity_int_value;\n",
	},
];

// Further generated translation units: verbatim extracts whose static names
// would collide with other extracts (each original .c file is its own scope).
export const EXTRACTED_UNITS = {
	"eech_extracted.c": EXTRACTED_C,
	"eech_extracted_ac_msgs.c": [
	{ kind: "raw", text: '#include "project.h"\n\n#define DEBUG_MODULE 0\n' },
	{ kind: "define", name: "AIR_RADAR_CONTACT_TIMEOUT", file: "aphavoc/source/gunships/avionics/common/co_radar.h" },

	// slice 3: cargo link/unlink parent responses are the aircraft ones
	// (cg_msgs.c :: overload_cargo_message_responses calls overload_aircraft_message_responses)
	{ kind: "function", name: "response_to_link_parent", signature: "static int response_to_link_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/mobile/aircraft/ac_msgs.c" },
	{ kind: "function", name: "response_to_unlink_parent", signature: "static int response_to_unlink_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/mobile/aircraft/ac_msgs.c" },
	{
		kind: "wrap",
		prologue: "void harness_overload_aircraft_link_parent_responses (entity_types type)\n{",
		epilogue: "}",
		parts: [
			{ kind: "regex", pattern: "\\n\\tmessage_responses\\[type\\]\\[ENTITY_MESSAGE_LINK_PARENT\\][^;]*;", file: "aphavoc/source/entity/mobile/aircraft/ac_msgs.c" },
			{ kind: "regex", pattern: "\\n\\tmessage_responses\\[type\\]\\[ENTITY_MESSAGE_UNLINK_PARENT\\][^;]*;", file: "aphavoc/source/entity/mobile/aircraft/ac_msgs.c" },
		],
	},

	],
	"eech_extracted_ks_msgs.c": [
	{ kind: "raw", text: '#include "project.h"\n\n#define DEBUG_MODULE 0\n' },

	// slice 3: keysite link/unlink child responses (ks_msgs.c is otherwise not compiled)
	{ kind: "function", name: "response_to_link_child", signature: "static int response_to_link_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/special/keysite/ks_msgs.c" },
	{ kind: "function", name: "response_to_unlink_child", signature: "static int response_to_unlink_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/special/keysite/ks_msgs.c" },
	{
		kind: "wrap",
		prologue: "void harness_overload_keysite_link_child_responses (void)\n{",
		epilogue: "}",
		parts: [
			{ kind: "regex", pattern: "\\n\\tmessage_responses\\[ENTITY_TYPE_KEYSITE\\]\\[ENTITY_MESSAGE_LINK_CHILD\\][^;]*;", file: "aphavoc/source/entity/special/keysite/ks_msgs.c" },
			{ kind: "regex", pattern: "\\n\\tmessage_responses\\[ENTITY_TYPE_KEYSITE\\]\\[ENTITY_MESSAGE_UNLINK_CHILD\\][^;]*;", file: "aphavoc/source/entity/special/keysite/ks_msgs.c" },
		],
	},
	],
};

function lineOf(text, offset) {
	return text.slice(0, offset).split("\n").length;
}

function extractBraced(text, start, what) {
	const open = text.indexOf("{", start);
	let depth = 0;
	for (let i = open; i < text.length; i++) {
		if (text[i] === "{") {
			depth++;
		} else if (text[i] === "}") {
			depth--;
			if (depth === 0) {
				return i + 1;
			}
		}
	}
	throw new Error(`unbalanced braces extracting ${what}`);
}

function readSource(file) {
	return readFileSync(join(repoRoot, file), "latin1").replace(/\r\n/g, "\n");
}

export function extractOne(spec) {
	if (spec.kind === "local") {
		return `#include "${spec.name}"\n`;
	}
	if (spec.kind === "include") {
		return `#include "${spec.name}"\n`;
	}
	if (spec.kind === "raw") {
		return `${spec.text}\n`;
	}
	if (spec.kind === "wrap") {
		return `${spec.prologue}\n${spec.parts.map(extractOne).join("")}${spec.epilogue}\n`;
	}
	const text = readSource(spec.file);
	let start;
	let end;
	if (spec.kind === "function") {
		start = text.indexOf(`\n${spec.signature}\n`);
		if (start < 0) {
			throw new Error(`${spec.signature} not found in ${spec.file}`);
		}
		start += 1;
		end = extractBraced(text, start, spec.name);
	} else if (spec.kind === "enum" || spec.kind === "struct") {
		const match = new RegExp(`\\n${spec.kind} ${spec.name}\\s*\\n`).exec(text);
		if (!match) {
			throw new Error(`${spec.kind} ${spec.name} not found in ${spec.file}`);
		}
		start = match.index + 1;
		end = extractBraced(text, start, spec.name);
		if (text[end] !== ";") {
			throw new Error(`${spec.kind} ${spec.name} not terminated by ;`);
		}
		end += 1;
	} else if (spec.kind === "define") {
		const match = new RegExp(`\\n#define ${spec.name}\\b[^\\n]*`).exec(text);
		if (!match) {
			throw new Error(`#define ${spec.name} not found in ${spec.file}`);
		}
		start = match.index + 1;
		end = match.index + match[0].length;
	} else if (spec.kind === "prototype") {
		const match = new RegExp(`\\nextern [^\\n(]*\\b${spec.name} \\([^\\n]*\\);`).exec(text);
		if (!match) {
			throw new Error(`prototype ${spec.name} not found in ${spec.file}`);
		}
		start = match.index + 1;
		end = match.index + match[0].length;
	} else if (spec.kind === "regex") {
		const match = new RegExp(spec.pattern).exec(text);
		if (!match) {
			throw new Error(`/${spec.pattern}/ not found in ${spec.file}`);
		}
		start = match.index + (match[0].startsWith("\n") ? 1 : 0);
		end = match.index + match[0].length;
	} else {
		throw new Error(`unknown extract kind ${spec.kind}`);
	}
	return `#line ${lineOf(text, start)} "${spec.file}"\n${text.slice(start, end)}\n`;
}

const BANNER = "/* GENERATED by c-reference/extract.mjs - verbatim original EECH C. Do not edit. */\n";

export function generateProjectH() {
	return [BANNER, "#ifndef EECH_HARNESS_PROJECT_H", "#define EECH_HARNESS_PROJECT_H", "", ...PROJECT_H.map(extractOne), "#endif", ""].join("\n");
}

export function generateExtracted(specs = EXTRACTED_C) {
	return [BANNER, ...specs.map(extractOne)].join("\n");
}

export function writeGenerated(outDir) {
	mkdirSync(outDir, { recursive: true });
	writeFileSync(join(outDir, "project.h"), generateProjectH());
	for (const [name, specs] of Object.entries(EXTRACTED_UNITS)) {
		writeFileSync(join(outDir, name), generateExtracted(specs));
	}
}

const isMain = process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1];

if (isMain) {
	writeGenerated(join(projectRoot, "build", "c-reference"));
	console.log(`wrote build/c-reference/project.h and ${Object.keys(EXTRACTED_UNITS).join(", ")}`);
}
