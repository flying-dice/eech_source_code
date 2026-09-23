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
	// slice 4: update_keysite_cargo (and get_closest_keysite, previously extracted)
	"aphavoc/source/entity/special/keysite/keysite.c",
	"aphavoc/source/entity/special/force/fc_int.c",
	"aphavoc/source/entity/special/force/fc_list.c",
	// slice 5a: the force's FORCE_LOW_ON_SUPPLIES response, and the task
	// database and accessors its duplicate-task decision reads
	"aphavoc/source/entity/special/force/fc_msgs.c",
	"aphavoc/source/entity/special/task/ts_dbase.c",
	"aphavoc/source/entity/special/task/ts_float.c",
	"aphavoc/source/entity/special/task/ts_int.c",
	"aphavoc/source/entity/special/task/ts_list.c",
	// slice 5a: route waypoints share a requester's LIST_TYPE_TASK_DEPENDENT list
	"aphavoc/source/entity/special/waypoint/wp_dbase.c",
	"aphavoc/source/entity/special/waypoint/wp_int.c",
	"aphavoc/source/entity/special/waypoint/wp_list.c",
	// slice 5b: supply task construction (create_supply_task -> create_task;
	// the taskgen.c functions are extracted: eech_extracted_taskgen.c)
	"aphavoc/source/entity/special/task/task.c",
	"aphavoc/source/entity/special/task/ts_creat.c",
	"aphavoc/source/entity/special/task/ts_ptr.c",
	"aphavoc/source/ai/highlevl/suitable.c",
	// slice 6a: assign_keysite_tasks reads a member's cruise velocity (ac_float.c,
	// aircraft_database) and the pilot lock of tasks and groups (pi_list.c)
	"aphavoc/source/entity/mobile/aircraft/ac_float.c",
	"aphavoc/source/entity/mobile/aircraft/ac_dbase.c",
	"aphavoc/source/entity/special/pilot/pi_list.c",
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

//
// Extra compile flags for one unit.
//
// Slice 5b, F1 (docs/slices/supply-task-construction.md): create_supply_task
// leaves prepare.y and finish.y uninitialised, and create_task reads them. EECH
// defines no value for them. The port resolves that undefined behaviour with a
// compatibility choice, 0.0, and the harness pins the otherwise unmodified
// original to the same choice by zero-initialising automatic variables in this
// unit only. This is not a claim that EECH produced 0.0: the F1 probe
// (npm run probe:f1, test/f1-probe) builds the same code zero-, pattern- and
// un-initialised to show the value is the build's, not the source's.
//
export const UNIT_FLAGS = {
	"eech_extracted_taskgen.c": ["-ftrivial-auto-var-init=zero"],
};

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
	// slice 4: update_keysite_cargo reads the game status (a host-set global)
	{ kind: "regex", pattern: "\\nextern game_status_types\\n\\tgame_status;", file: "aphavoc/source/global.h" },
	{ kind: "define", name: "get_game_status", file: "aphavoc/source/global.h" },
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
	// slice 4: keysite.c, compiled whole. Headers for what its other functions
	// (capture, destruction, speech, landing, tasks) name; the functions
	// themselves are fail-loud stubs in harness.c.
	{ kind: "regex", pattern: "\\ntypedef enum WEATHERMODES weathermodes;", file: "modules/3d/3denv.h" },
	{ kind: "regex", pattern: "\\nenum SESSION_TIME_OF_DAY_SETTINGS\\n\\{[^}]*\\};", file: "aphavoc/source/global.h" },
	{ kind: "regex", pattern: "\\ntypedef enum SESSION_TIME_OF_DAY_SETTINGS session_time_of_day_settings;", file: "aphavoc/source/global.h" },
	{ kind: "regex", pattern: "\\nenum SYS_COLOURS\\n\\{[^}]*\\};", file: "modules/graphics/colour.h" },
	{ kind: "regex", pattern: "\\ntypedef enum SYS_COLOURS sys_colours;", file: "modules/graphics/colour.h" },
	{ kind: "define", name: "SECONDS_IN_A_MINUTE", file: "modules/maths/constant.h" },
	{ kind: "define", name: "ONE_MINUTE", file: "modules/maths/constant.h" },
	{ kind: "regex", pattern: "\\nextern int\\n\\trandom_number_seed;", file: "modules/maths/random.h" },
	{ kind: "define", name: "get_random_number", file: "modules/maths/random.h" },
	{ kind: "define", name: "rand16", file: "modules/maths/random.h" },
	{ kind: "define", name: "sfrand1", file: "modules/maths/random.h" },
	{ kind: "regex", pattern: "\\nextern float get_3d_terrain_point_data \\( float x, float z, terrain_3d_point_data \\*point_data \\);", file: "modules/3d/terrain/terrelev.h" },
	{ kind: "regex", pattern: "\\n#define get_3d_terrain_elevation\\(X,Z\\) \\(get_3d_terrain_point_data \\(\\(X\\), \\(Z\\), NULL\\)\\)", file: "modules/3d/terrain/terrelev.h" },
	{ kind: "prototype", name: "file_exist", file: "modules/system/files.h" },
	{ kind: "include", name: "cmndline.h" },
	{ kind: "include", name: "misc/message.h" },
	{ kind: "include", name: "misc/msg_out.h" },
	{ kind: "include", name: "misc/tod.h" },
	{ kind: "regex", pattern: "\\ntypedef enum SOUND_LOCALITY_TYPES\\n\\{[^}]*\\} sound_locality_types;", file: "aphavoc/source/entity/special/effect/soundeff/soundeff.h" },
	{ kind: "define", name: "SOUND_LOCALITY_RADIO", file: "aphavoc/source/entity/special/effect/soundeff/soundeff.h" },
	{ kind: "include", name: "entity/special/effect/soundeff/speech.h" },
	{ kind: "include", name: "entity/special/task/task.h" },
	{ kind: "include", name: "entity/special/landing/landing.h" },
	{ kind: "include", name: "entity/special/regen/rg_updt.h" },
	{ kind: "include", name: "entity/special/session/session.h" },
	{ kind: "include", name: "entity/fixed/fixed.h" },
	// slice 5a: fc_msgs.c, compiled whole. What its other responses name.
	{ kind: "enum", name: "CAMPAIGN_COMPLETED_TYPES", file: "aphavoc/source/ui_menu/ingame/campaign/campaign.h" },
	{ kind: "regex", pattern: "\\ntypedef enum CAMPAIGN_COMPLETED_TYPES campaign_completed_types;", file: "aphavoc/source/ui_menu/ingame/campaign/campaign.h" },
	{ kind: "prototype", name: "campaign_completed", file: "aphavoc/source/ui_menu/ingame/campaign/campaign.h" },
	{ kind: "prototype", name: "play_client_server_radio_message_response", file: "aphavoc/source/misc/msg_in.h" },
	{ kind: "prototype", name: "get_sqr_2d_range", file: "modules/maths/range.h" },
	// slice 5b: create_supply_task
	{ kind: "prototype", name: "normalise_any_3d_vector", file: "modules/maths/vector.h" },
	// slice 6a: ac_float.c (compiled whole for FLOAT_TYPE_CRUISE_VELOCITY) names these in float types nothing ported reads
	{ kind: "prototype", name: "get_3d_terrain_point_data_elevation", file: "modules/3d/terrain/terrelev.h" },
	{ kind: "regex", pattern: "\\nstruct OBJECT_3D_INFORMATION\\n\\{[\\s\\S]*?\\n\\};\\n\\ntypedef struct OBJECT_3D_INFORMATION object_3d_information;", file: "modules/3d/3dobjid.h" },
	{ kind: "raw", text: "extern object_3d_information *object_3d_information_database;\n" },
	// slice 6a: ac_dbase.c (compiled whole: aircraft_database [].cruise_velocity) initialises with these
	{ kind: "include", name: "maths/constant.h" },
	{ kind: "include", name: "maths/convert.h" },
	{ kind: "regex", pattern: "\\nenum //SOUND_SAMPLE_INDICES\\n\\{[\\s\\S]*?\\n\\};", file: "aphavoc/source/appsound/snd_data.h" },
	{ kind: "regex", pattern: "\\n#define EXPLOSIVE_QUALITY_NONE[\\s\\S]*?#define EXPLOSIVE_QUALITY_FLAMMABLE\\t4", file: "aphavoc/source/entity/special/effect/explosn/explosn.h" },
	{ kind: "regex", pattern: "\\nenum\\n\\{\\n\\tEXPLOSIVE_POWER_NONE,[\\s\\S]*?\\n\\};", file: "aphavoc/source/entity/special/effect/explosn/explosn.h" },
	{ kind: "prototype", name: "initialise_group_task_array", file: "aphavoc/source/ai/highlevl/suitable.h" },
	// slice 5b: the campaign screen notification (a port in the TS core)
	{ kind: "enum", name: "CAMPAIGN_SCREEN_MESSAGES", file: "aphavoc/source/ui_menu/ingame/campaign/ca_msgs.h" },
	{ kind: "regex", pattern: "\\ntypedef enum CAMPAIGN_SCREEN_MESSAGES campaign_screen_messages;", file: "aphavoc/source/ui_menu/ingame/campaign/ca_msgs.h" },
	{ kind: "prototype", name: "notify_campaign_screen", file: "aphavoc/source/ui_menu/ingame/campaign/ca_msgs.h" },
	{ kind: "enum", name: "CAMPAIGN_SCREEN_MESSAGE_TARGETS", file: "aphavoc/source/ui_menu/ingame/campaign/ca_msgs.h" },
	{ kind: "regex", pattern: "\\nextern int \\(\\*campaign_screen_message_responses\\[NUM_CAMPAIGN_SCREEN_MESSAGE_TARGETS\\]\\[NUM_CAMPAIGN_SCREEN_MESSAGES\\]\\) \\(campaign_screen_messages message, entity \\*sender\\);", file: "aphavoc/source/ui_menu/ingame/campaign/ca_msgs.h" },
	{ kind: "define", name: "DEMO_VERSION", file: "aphavoc/source/project.h" },
	{ kind: "enum", name: "GAME_TYPES", file: "aphavoc/source/global.h" },
	{ kind: "regex", pattern: "\\ntypedef enum GAME_TYPES game_types;", file: "aphavoc/source/global.h" },
	{ kind: "regex", pattern: "\\nextern game_types\\n\\tgame_type;", file: "aphavoc/source/ui_menu/gametype/gametype.h" },
	{ kind: "define", name: "get_game_type", file: "aphavoc/source/ui_menu/gametype/gametype.h" },
	{ kind: "include", name: "entity/special/waypoint/waypoint.h" },
	// slice 6a: the pilot struct and its list functions (pi_list.c: LIST_TYPE_PILOT_LOCK)
	// (pilot.h's high score prototype names ui_object, an incomplete type here)
	{ kind: "regex", pattern: "\\ntypedef struct UI_OBJECT ui_object;", file: "modules/userint2/ui_sys/ui_types/ui_types.h" },
	{ kind: "include", name: "entity/special/pilot/pilot.h" },
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
	// slice 5a: what a waypoint answers for FLOAT_TYPE_TASK_USER_DATA (wp_float.c does not overload it)
	{ kind: "function", name: "default_get_entity_float_value", signature: "static float default_get_entity_float_value (entity *en, float_types type)", file: "aphavoc/source/entity/system/en_funcs/en_float.c" },

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

	// slice 5b: supply task construction reaches these (their files are otherwise not compiled)
	{ kind: "function", name: "normalise_any_3d_vector", signature: "float normalise_any_3d_vector ( vec3d *vector )", file: "modules/maths/vector.c" },
	{ kind: "function", name: "bound_position_to_adjusted_map_area", signature: "int bound_position_to_adjusted_map_area (vec3d *position)", file: "aphavoc/source/entity/system/en_main/en_world.c" },
	{ kind: "function", name: "get_local_sector_entity_enemy_defence_level", signature: "static float get_local_sector_entity_enemy_defence_level (float *array, entity_sides side)", file: "aphavoc/source/entity/special/sector/sector.c" },
	{ kind: "function", name: "get_local_sector_entity_enemy_surface_to_air_defence_level", signature: "float get_local_sector_entity_enemy_surface_to_air_defence_level (entity *sector_en, entity_sides side)", file: "aphavoc/source/entity/special/sector/sector.c" },
	{ kind: "function", name: "set_client_server_entity_parent", signature: "void set_client_server_entity_parent (entity *en, list_types type, entity *parent)", file: "aphavoc/source/entity/system/en_funcs/en_list.c" },

	// slice 5b: the campaign screen's guard (game status, game type); its
	// response table is the UI, supplied by the harness
	{ kind: "function", name: "notify_campaign_screen", signature: "int notify_campaign_screen (campaign_screen_messages message, entity *sender)", file: "aphavoc/source/ui_menu/ingame/campaign/ca_msgs.c" },

	// campaign functions (slice 1)
	{ kind: "function", name: "get_local_force_entity", signature: "entity *get_local_force_entity (entity_sides side)", file: "aphavoc/source/entity/special/force/force.c" },
	{ kind: "function", name: "assess_group_supplies", signature: "void assess_group_supplies (entity *en)", file: "aphavoc/source/entity/special/group/group.c" },
	// slice 6a: the locality test get_suitable_registered_group applies
	{ kind: "function", name: "assess_group_task_locality_factor", signature: "int assess_group_task_locality_factor (entity *group_en, entity *task_en, float *return_distance)", file: "aphavoc/source/entity/special/group/group.c" },

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
	// slice 5b: a task joins a group's LIST_TYPE_TASK_DEPENDENT list (the group is its objective)
	{ kind: "function", name: "response_to_link_child", signature: "static int response_to_link_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/special/group/gp_msgs.c" },
	{
		kind: "wrap",
		prologue: "void harness_overload_group_link_child_response (void)\n{",
		epilogue: "}",
		parts: [{ kind: "regex", pattern: "\\n\\tmessage_responses\\[ENTITY_TYPE_GROUP\\]\\[ENTITY_MESSAGE_LINK_CHILD\\][^;]*;", file: "aphavoc/source/entity/special/group/gp_msgs.c" }],
	},
	{
		kind: "raw",
		text: "/* C defaults for the update entity: its link responses are only overloaded under DEBUG_MODULE (up_msgs.c) */\nvoid harness_default_update_link_responses (void)\n{\n\tmessage_responses[ENTITY_TYPE_UPDATE][ENTITY_MESSAGE_LINK_CHILD] = default_message_response;\n\tmessage_responses[ENTITY_TYPE_UPDATE][ENTITY_MESSAGE_LINK_PARENT] = default_message_response;\n\tmessage_responses[ENTITY_TYPE_UPDATE][ENTITY_MESSAGE_UNLINK_PARENT] = default_message_response;\n}\n\nvoid (*harness_default_set_entity_int_value) (entity *en, int_types type, int value) = default_set_entity_int_value;\n\nvoid (*harness_default_set_entity_float_value) (entity *en, float_types type, float value) = default_set_entity_float_value;\n\nint (*harness_default_get_entity_int_value) (entity *en, int_types type) = default_get_entity_int_value;\n\nfloat (*harness_default_get_entity_float_value) (entity *en, float_types type) = default_get_entity_float_value;\n",
	},
];

// Further generated translation units: verbatim extracts whose static names
// would collide with other extracts (each original .c file is its own scope).
export const EXTRACTED_UNITS = {
	"eech_extracted.c": EXTRACTED_C,
	// slice 5b: taskgen.c's supply task construction. The rest of taskgen.c
	// (the other task generators) reaches the 3D engine and is not part of the
	// port. Compiled with UNIT_FLAGS below.
	// slice 6a: assign.c's assignment decision. The rest of assign.c is the
	// assignment transaction (slice 6b / 6c) and the player's requests;
	// assign_primary_task_to_group is the harness's boundary trap.
	"eech_extracted_assign.c": [
		{ kind: "raw", text: '#include <limits.h>\n\n#include "project.h"\n\n#include "ai/taskgen/assign.h"\n#include "ai/taskgen/taskgen.h"\n#include "ai/highlevl/suitable.h"\n' },
		{ kind: "prototype", name: "quicksort_entity_list", file: "aphavoc/source/entity/en_misc/en_misc.h" },
		// the release (non-DEBUG, non-WIN32) ai_log of highlevl.h: compiled out
		{ kind: "regex", pattern: "\\n#define ai_log\\(a, x\\.\\.\\.\\) do \\{ \\} while\\(0\\);", file: "aphavoc/source/ai/highlevl/highlevl.h" },
		{ kind: "define", name: "DEBUG_MODULE", file: "aphavoc/source/ai/taskgen/assign.c" },
		{ kind: "function", name: "assign_keysite_tasks", signature: "void assign_keysite_tasks (entity *keysite, task_category_types category)", file: "aphavoc/source/ai/taskgen/assign.c" },
		{ kind: "function", name: "suitable_group_task_specific_checks", signature: "static int suitable_group_task_specific_checks (entity *task, entity *group)", file: "aphavoc/source/ai/taskgen/assign.c" },
		{ kind: "function", name: "get_suitable_registered_group", signature: "entity *get_suitable_registered_group (entity *task, int *idle_group_count)", file: "aphavoc/source/ai/taskgen/assign.c" },
		{ kind: "function", name: "check_group_members_awake", signature: "int check_group_members_awake (entity *group)", file: "aphavoc/source/ai/taskgen/assign.c" },
	],
	// slice 6a: en_misc.c's quicksort (qs is static)
	"eech_extracted_en_misc.c": [
		{ kind: "raw", text: '#include "project.h"\n' },
		{ kind: "function", name: "qs", signature: "static void qs (entity **en_list, float *sort_order, int left, int right)", file: "aphavoc/source/entity/en_misc/en_misc.c" },
		{ kind: "function", name: "quicksort_entity_list", signature: "void quicksort_entity_list (entity **en_list, int count, float *sort_order)", file: "aphavoc/source/entity/en_misc/en_misc.c" },
	],
	"eech_extracted_taskgen.c": [
		{ kind: "raw", text: '#include "project.h"\n\n#include "ai/taskgen/taskgen.h"\n' },
		{ kind: "define", name: "DEBUG_MODULE", file: "aphavoc/source/ai/taskgen/taskgen.c" },
		{ kind: "define", name: "TASK_SAFE_LIMIT", file: "aphavoc/source/ai/taskgen/taskgen.c" },
		{ kind: "regex", pattern: "\\nvec3d\\n   terminator_point = \\{-1\\.0, -1\\.0, -1\\.0\\};", file: "aphavoc/source/ai/taskgen/taskgen.c" },
		{ kind: "regex", pattern: "\\nstatic int get_task_start_keysite \\(entity_sub_types sub_type, entity_sides side, vec3d \\*start_pos, entity \\*\\*start_keysite\\);", file: "aphavoc/source/ai/taskgen/taskgen.c" },
		{ kind: "function", name: "create_task", signature: "entity *create_task", file: "aphavoc/source/ai/taskgen/taskgen.c" },
		{ kind: "function", name: "create_supply_task", signature: "entity *create_supply_task (entity *requester, entity *supplier, entity *cargo, movement_types movement_type, float priority, entity *start_keysite, entity *end_keysite)", file: "aphavoc/source/ai/taskgen/taskgen.c" },
		{ kind: "function", name: "validate_task_generation", signature: "int validate_task_generation (entity_sides side, entity_sub_types sub_type)", file: "aphavoc/source/ai/taskgen/taskgen.c" },
		{ kind: "function", name: "get_task_start_keysite", signature: "int get_task_start_keysite (entity_sub_types sub_type, entity_sides side, vec3d *start_pos, entity **start_keysite)", file: "aphavoc/source/ai/taskgen/taskgen.c" },
	],
	// slice 5b: the task's LINK_PARENT response (unassigned: state and the
	// campaign screen); ts_msgs.c's other responses are not part of the port
	"eech_extracted_ts_msgs.c": [
		{ kind: "raw", text: '#include "project.h"\n\n#define DEBUG_MODULE 0\n' },
		{ kind: "function", name: "response_to_link_parent", signature: "static int response_to_link_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/special/task/ts_msgs.c" },
		{
			kind: "wrap",
			prologue: "void harness_overload_task_link_parent_response (void)\n{",
			epilogue: "}",
			parts: [{ kind: "regex", pattern: "\\n\\tmessage_responses\\[ENTITY_TYPE_TASK\\]\\[ENTITY_MESSAGE_LINK_PARENT\\][^;]*;", file: "aphavoc/source/entity/special/task/ts_msgs.c" }],
		},
	],
	// slice 5b: ENTITY_COMMS_SET_TASK_POINTERS packs the route nodes with the
	// original pack_vec3d (its position check and in-place bound); the harness
	// supplies only the bit-stream sinks
	"eech_extracted_en_vec3d.c": [
		{ kind: "raw", text: '#include "project.h"\n' },
		{ kind: "define", name: "DEBUG_MODULE_PACK_ONE", file: "aphavoc/source/entity/system/en_funcs/en_vec3d.c" },
		{ kind: "define", name: "DEBUG_MODULE_PACK_ALL", file: "aphavoc/source/entity/system/en_funcs/en_vec3d.c" },
		{ kind: "regex", pattern: "\\nvec3d_type_data\\n\\tvec3d_type_database\\[NUM_VEC3D_TYPES\\] =\\n\\t\\{[\\s\\S]*?\\n\\t\\};", file: "aphavoc/source/entity/system/en_funcs/en_vec3d.c" },
		{ kind: "function", name: "pack_vec3d", signature: "void pack_vec3d (entity *en, vec3d_types type, vec3d *v)", file: "aphavoc/source/entity/system/en_funcs/en_vec3d.c" },
		{ kind: "function", name: "bound_position_to_map_volume", signature: "int bound_position_to_map_volume (vec3d *position)", file: "aphavoc/source/entity/system/en_main/en_world.c" },
	],
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
