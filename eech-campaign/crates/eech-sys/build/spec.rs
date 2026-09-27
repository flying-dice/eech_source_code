//
// The native kernel's source closure.
//
// Forked from eech-core-ts/c-reference/extract.mjs at 81ed32e (slice 6a):
// PROJECT_H, EXTRACTED_UNITS and REAL_TRANSLATION_UNITS are transcribed
// entry for entry, so the native kernel executes exactly the original code the
// C reference executes. Native additions are marked `NATIVE:` and listed in
// eech-campaign/docs/closure.md.
//

use crate::extract::Spec::{self, *};

/// Original translation units compiled unchanged (paths relative to the repository root).
pub const REAL_TRANSLATION_UNITS: &[&str] = &[
    "aphavoc/source/entity/special/group/gp_dbase.c",
    "aphavoc/source/entity/special/group/gp_float.c",
    "aphavoc/source/entity/special/group/gp_int.c",
    "aphavoc/source/entity/special/group/gp_list.c",
    "aphavoc/source/entity/special/group/gp_ptr.c",
    "aphavoc/source/entity/special/group/gp_updt.c",
    "aphavoc/source/entity/special/group/gp_vec3d.c",
    "aphavoc/source/entity/special/update/up_list.c",
    "aphavoc/source/entity/special/update/up_msgs.c",
    "aphavoc/source/entity/special/keysite/ks_dbase.c",
    "aphavoc/source/entity/special/keysite/ks_float.c",
    "aphavoc/source/entity/special/keysite/ks_int.c",
    "aphavoc/source/entity/special/keysite/ks_list.c",
    "aphavoc/source/entity/special/keysite/ks_vec3d.c",
    "aphavoc/source/entity/special/keysite/keysite.c",
    "aphavoc/source/entity/special/force/fc_int.c",
    "aphavoc/source/entity/special/force/fc_list.c",
    // fc_msgs.c: see UNIT_DEFINES (the create_supply_task observation point)
    "aphavoc/source/entity/special/force/fc_msgs.c",
    "aphavoc/source/entity/special/task/ts_dbase.c",
    "aphavoc/source/entity/special/task/ts_float.c",
    "aphavoc/source/entity/special/task/ts_int.c",
    "aphavoc/source/entity/special/task/ts_list.c",
    "aphavoc/source/entity/special/waypoint/wp_dbase.c",
    "aphavoc/source/entity/special/waypoint/wp_int.c",
    "aphavoc/source/entity/special/waypoint/wp_list.c",
    "aphavoc/source/entity/special/task/task.c",
    "aphavoc/source/entity/special/task/ts_creat.c",
    "aphavoc/source/entity/special/task/ts_ptr.c",
    "aphavoc/source/ai/highlevl/suitable.c",
    "aphavoc/source/entity/mobile/aircraft/ac_float.c",
    "aphavoc/source/entity/mobile/aircraft/ac_dbase.c",
    "aphavoc/source/entity/special/pilot/pi_list.c",
    // en_heap.c: see UNIT_DEFINES (entity identity: allocation observation)
    "aphavoc/source/entity/system/en_main/en_heap.c",
    "aphavoc/source/entity/system/en_attrs/en_attrs.c",
    // en_creat.c is PATCHED (64-bit): see patches.rs
    "aphavoc/source/entity/system/en_funcs/en_dstry.c",
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
    // NATIVE: the task update function (expiry and stop timers), reached by step ()
    "aphavoc/source/entity/special/task/ts_updt.c",
];

/// Controlled compile definitions for single original units. Each renames one
/// function at its definition or call site so that the native layer can
/// observe it without editing the source (portable replacement for the C
/// reference's `ld --wrap`).
pub const UNIT_DEFINES: &[(&str, &[&str])] = &[
    // fc_msgs.c calls create_supply_task through eech_observe_create_supply_task
    // (csrc/eech_env.c), which reports the call and runs the original
    (
        "aphavoc/source/entity/special/force/fc_msgs.c",
        &["create_supply_task=eech_observe_create_supply_task"],
    ),
    // en_heap.c defines get_free_entity under another name; csrc/eech_kernel.c's
    // get_free_entity runs it and records the new incarnation (EntityId generation)
    (
        "aphavoc/source/entity/system/en_main/en_heap.c",
        &["get_free_entity=eech_original_get_free_entity"],
    ),
];

/// Original units that are patched (see patches.rs), generated into OUT_DIR.
pub const PATCHED_UNITS: &[&str] = &[
    "aphavoc/source/entity/system/en_funcs/en_creat.c",
    "aphavoc/source/entity/special/keysite/ks_updt.c",
];

pub fn project_h() -> Vec<Spec> {
    vec![
        Local("eech_native_env.h"),
        Struct {
            name: "VEC3D",
            file: "modules/maths/vector.h",
        },
        Regex {
            pattern: r"\ntypedef struct VEC3D vec3d;",
            file: "modules/maths/vector.h",
        },
        Define {
            name: "bound",
            file: "modules/maths/miscmath.h",
        },
        Define {
            name: "METRE",
            file: "modules/maths/constant.h",
        },
        Define {
            name: "KILOMETRE",
            file: "modules/maths/constant.h",
        },
        Regex {
            pattern: r"\nextern float\n\tsystem_delta_time,\n\tsystem_one_over_delta_time;",
            file: "modules/system/time.h",
        },
        Regex {
            pattern: r"\nextern int\n\tlocked_frame_rate;",
            file: "modules/system/time.h",
        },
        Regex {
            pattern: r"\nextern void set_manual_delta_time \(float delta_time\);",
            file: "modules/system/time.h",
        },
        Define {
            name: "get_delta_time",
            file: "modules/system/time.h",
        },
        Enum {
            name: "COMMS_MODEL_TYPES",
            file: "aphavoc/source/comms/comms.h",
        },
        Regex {
            pattern: r"\ntypedef enum COMMS_MODEL_TYPES comms_model_types;",
            file: "aphavoc/source/comms/comms.h",
        },
        Enum {
            name: "COMMS_DATA_FLOW_TYPES",
            file: "aphavoc/source/comms/comms.h",
        },
        Regex {
            pattern: r"\ntypedef enum COMMS_DATA_FLOW_TYPES comms_data_flow_types;",
            file: "aphavoc/source/comms/comms.h",
        },
        Regex {
            pattern: r"\nextern comms_model_types\n\s*system_comms_model;",
            file: "aphavoc/source/comms/comms.h",
        },
        Regex {
            pattern: r"\nextern comms_data_flow_types\n\s*system_comms_data_flow;",
            file: "aphavoc/source/comms/comms.h",
        },
        Define {
            name: "get_comms_model",
            file: "aphavoc/source/comms/comms.h",
        },
        Define {
            name: "get_comms_data_flow",
            file: "aphavoc/source/comms/comms.h",
        },
        Enum {
            name: "MAP_LAYER_CONTROL_TYPES",
            file: "aphavoc/source/ui_menu/ingame/common/map.h",
        },
        Define {
            name: "MAP_LAYER_CONTROL_NONE",
            file: "aphavoc/source/ui_menu/ingame/common/map.h",
        },
        Enum {
            name: "MAP_ICON_TYPE",
            file: "aphavoc/source/ui_menu/ingame/common/map.h",
        },
        Include("entity/system/en_types/en_types.h"),
        Include("entity/system/en_main/en_heap.h"),
        Include("entity/system/en_msgs/en_msgs.h"),
        Include("entity/system/en_funcs/en_funcs.h"),
        Include("entity/system/en_comms/en_comms.h"),
        Include("entity/system/en_debug/en_dbmsg.h"),
        Include("ai_extrn.h"),
        Include("entity/special/update/up_update.h"),
        Include("entity/special/division/division.h"),
        Include("entity/special/group/group.h"),
        Include("entity/tacview/tacview.h"),
        Regex {
            pattern: r"\ntypedef struct OBJECT_3D_INSTANCE object_3d_instance;",
            file: "modules/3d/objects.h",
        },
        Struct {
            name: "OBJECT_3D_BOUNDS",
            file: "modules/3d/objects.h",
        },
        Regex {
            pattern: r"\ntypedef struct OBJECT_3D_BOUNDS object_3d_bounds;",
            file: "modules/3d/objects.h",
        },
        Regex {
            pattern: r"\ntypedef int terrain_types;",
            file: "modules/3d/terrain/terrtype.h",
        },
        Regex {
            pattern: r"\ntypedef struct TERRAIN_3D_POINT_WORD_REFERENCE terrain_3d_point_word_reference;",
            file: "modules/3d/terrain/terrdata.h",
        },
        Regex {
            pattern: r"\ntypedef struct TERRAIN_3D_POINT_BYTE_REFERENCE terrain_3d_point_byte_reference;",
            file: "modules/3d/terrain/terrdata.h",
        },
        Regex {
            pattern: r"\ntypedef struct TERRAIN_3D_FACE terrain_3d_face;",
            file: "modules/3d/terrain/terrdata.h",
        },
        Regex {
            pattern: r"\ntypedef struct TERRAIN_3D_SECTOR terrain_3d_sector;",
            file: "modules/3d/terrain/terrdata.h",
        },
        Struct {
            name: "TERRAIN_3D_POINT_DATA",
            file: "modules/3d/terrain/terrdata.h",
        },
        Regex {
            pattern: r"\ntypedef struct TERRAIN_3D_POINT_DATA terrain_3d_point_data;",
            file: "modules/3d/terrain/terrdata.h",
        },
        Struct {
            name: "DOUBLE_VEC3D",
            file: "modules/maths/vector.h",
        },
        Regex {
            pattern: r"\ntypedef struct DOUBLE_VEC3D double_vec3d;",
            file: "modules/maths/vector.h",
        },
        Regex {
            pattern: r"\ntypedef struct EVENT event;",
            file: "modules/system/event.h",
        },
        Enum {
            name: "DEBUG_COLOURS",
            file: "modules/system/debug.h",
        },
        Regex {
            pattern: r"\nextern void debug_colour_log \( enum DEBUG_COLOURS colour, const char \*string, \.\.\. \);",
            file: "modules/system/debug.h",
        },
        Prototype {
            name: "get_identity_matrix3x3",
            file: "modules/maths/matrix.h",
        },
        Regex {
            pattern: r"\ntypedef int sound_sample_indices;",
            file: "aphavoc/source/appsound/snd_data.h",
        },
        Enum {
            name: "GAME_STATUS_TYPES",
            file: "aphavoc/source/global.h",
        },
        Regex {
            pattern: r"\ntypedef enum GAME_STATUS_TYPES game_status_types;",
            file: "aphavoc/source/global.h",
        },
        Regex {
            pattern: r"\nextern game_status_types\n\tgame_status;",
            file: "aphavoc/source/global.h",
        },
        Define {
            name: "get_game_status",
            file: "aphavoc/source/global.h",
        },
        Enum {
            name: "GUNSHIP_TYPES",
            file: "aphavoc/source/global.h",
        },
        Regex {
            pattern: r"\ntypedef enum GUNSHIP_TYPES gunship_types;",
            file: "aphavoc/source/global.h",
        },
        Enum {
            name: "GAME_DIFFICULTY_SETTINGS",
            file: "aphavoc/source/global.h",
        },
        Enum {
            name: "WEATHERMODES",
            file: "modules/3d/3denv.h",
        },
        Enum {
            name: "SESSION_LIST_TYPES",
            file: "aphavoc/source/ui_menu/session/session.h",
        },
        Regex {
            pattern: r"\ntypedef enum SESSION_LIST_TYPES session_list_types;",
            file: "aphavoc/source/ui_menu/session/session.h",
        },
        Prototype {
            name: "int_bit_count",
            file: "aphavoc/source/misc/miscell.h",
        },
        Include("misc/listitem.h"),
        Include("3d/3dmodels.h"),
        Include("3d/textanim.h"),
        Include("entity/special/keysite/keysite.h"),
        Include("entity/special/force/force.h"),
        Include("entity/system/en_main/en_world.h"),
        Include("entity/system/en_attrs/en_attrs.h"),
        Include("entity/mobile/mobile.h"),
        Include("entity/special/sector/sector.h"),
        Include("entity/system/en_debug/en_stats.h"),
        Regex {
            pattern: r"\ntypedef enum WEATHERMODES weathermodes;",
            file: "modules/3d/3denv.h",
        },
        Regex {
            pattern: r"\nenum SESSION_TIME_OF_DAY_SETTINGS\n\{[^}]*\};",
            file: "aphavoc/source/global.h",
        },
        Regex {
            pattern: r"\ntypedef enum SESSION_TIME_OF_DAY_SETTINGS session_time_of_day_settings;",
            file: "aphavoc/source/global.h",
        },
        Regex {
            pattern: r"\nenum SYS_COLOURS\n\{[^}]*\};",
            file: "modules/graphics/colour.h",
        },
        Regex {
            pattern: r"\ntypedef enum SYS_COLOURS sys_colours;",
            file: "modules/graphics/colour.h",
        },
        Define {
            name: "SECONDS_IN_A_MINUTE",
            file: "modules/maths/constant.h",
        },
        Define {
            name: "ONE_MINUTE",
            file: "modules/maths/constant.h",
        },
        Regex {
            pattern: r"\nextern int\n\trandom_number_seed;",
            file: "modules/maths/random.h",
        },
        Define {
            name: "get_random_number",
            file: "modules/maths/random.h",
        },
        Define {
            name: "rand16",
            file: "modules/maths/random.h",
        },
        Define {
            name: "sfrand1",
            file: "modules/maths/random.h",
        },
        Regex {
            pattern: r"\nextern float get_3d_terrain_point_data \( float x, float z, terrain_3d_point_data \*point_data \);",
            file: "modules/3d/terrain/terrelev.h",
        },
        Regex {
            pattern: r"\n#define get_3d_terrain_elevation\(X,Z\) \(get_3d_terrain_point_data \(\(X\), \(Z\), NULL\)\)",
            file: "modules/3d/terrain/terrelev.h",
        },
        Prototype {
            name: "file_exist",
            file: "modules/system/files.h",
        },
        Include("cmndline.h"),
        Include("misc/message.h"),
        Include("misc/msg_out.h"),
        Include("misc/tod.h"),
        Regex {
            pattern: r"\ntypedef enum SOUND_LOCALITY_TYPES\n\{[^}]*\} sound_locality_types;",
            file: "aphavoc/source/entity/special/effect/soundeff/soundeff.h",
        },
        Define {
            name: "SOUND_LOCALITY_RADIO",
            file: "aphavoc/source/entity/special/effect/soundeff/soundeff.h",
        },
        Include("entity/special/effect/soundeff/speech.h"),
        Include("entity/special/task/task.h"),
        Include("entity/special/landing/landing.h"),
        Include("entity/special/regen/rg_updt.h"),
        Include("entity/special/session/session.h"),
        Include("entity/fixed/fixed.h"),
        Enum {
            name: "CAMPAIGN_COMPLETED_TYPES",
            file: "aphavoc/source/ui_menu/ingame/campaign/campaign.h",
        },
        Regex {
            pattern: r"\ntypedef enum CAMPAIGN_COMPLETED_TYPES campaign_completed_types;",
            file: "aphavoc/source/ui_menu/ingame/campaign/campaign.h",
        },
        Prototype {
            name: "campaign_completed",
            file: "aphavoc/source/ui_menu/ingame/campaign/campaign.h",
        },
        Prototype {
            name: "play_client_server_radio_message_response",
            file: "aphavoc/source/misc/msg_in.h",
        },
        Prototype {
            name: "get_sqr_2d_range",
            file: "modules/maths/range.h",
        },
        Prototype {
            name: "normalise_any_3d_vector",
            file: "modules/maths/vector.h",
        },
        Prototype {
            name: "get_3d_terrain_point_data_elevation",
            file: "modules/3d/terrain/terrelev.h",
        },
        Regex {
            pattern: r"\nstruct OBJECT_3D_INFORMATION\n\{[\s\S]*?\n\};\n\ntypedef struct OBJECT_3D_INFORMATION object_3d_information;",
            file: "modules/3d/3dobjid.h",
        },
        Raw("extern object_3d_information *object_3d_information_database;\n".into()),
        Include("maths/constant.h"),
        Include("maths/convert.h"),
        Regex {
            pattern: r"\nenum //SOUND_SAMPLE_INDICES\n\{[\s\S]*?\n\};",
            file: "aphavoc/source/appsound/snd_data.h",
        },
        Regex {
            pattern: r"\n#define EXPLOSIVE_QUALITY_NONE[\s\S]*?#define EXPLOSIVE_QUALITY_FLAMMABLE\t4",
            file: "aphavoc/source/entity/special/effect/explosn/explosn.h",
        },
        Regex {
            pattern: r"\nenum\n\{\n\tEXPLOSIVE_POWER_NONE,[\s\S]*?\n\};",
            file: "aphavoc/source/entity/special/effect/explosn/explosn.h",
        },
        Prototype {
            name: "initialise_group_task_array",
            file: "aphavoc/source/ai/highlevl/suitable.h",
        },
        Enum {
            name: "CAMPAIGN_SCREEN_MESSAGES",
            file: "aphavoc/source/ui_menu/ingame/campaign/ca_msgs.h",
        },
        Regex {
            pattern: r"\ntypedef enum CAMPAIGN_SCREEN_MESSAGES campaign_screen_messages;",
            file: "aphavoc/source/ui_menu/ingame/campaign/ca_msgs.h",
        },
        Prototype {
            name: "notify_campaign_screen",
            file: "aphavoc/source/ui_menu/ingame/campaign/ca_msgs.h",
        },
        Enum {
            name: "CAMPAIGN_SCREEN_MESSAGE_TARGETS",
            file: "aphavoc/source/ui_menu/ingame/campaign/ca_msgs.h",
        },
        Regex {
            pattern: r"\nextern int \(\*campaign_screen_message_responses\[NUM_CAMPAIGN_SCREEN_MESSAGE_TARGETS\]\[NUM_CAMPAIGN_SCREEN_MESSAGES\]\) \(campaign_screen_messages message, entity \*sender\);",
            file: "aphavoc/source/ui_menu/ingame/campaign/ca_msgs.h",
        },
        Define {
            name: "DEMO_VERSION",
            file: "aphavoc/source/project.h",
        },
        Enum {
            name: "GAME_TYPES",
            file: "aphavoc/source/global.h",
        },
        Regex {
            pattern: r"\ntypedef enum GAME_TYPES game_types;",
            file: "aphavoc/source/global.h",
        },
        Regex {
            pattern: r"\nextern game_types\n\tgame_type;",
            file: "aphavoc/source/ui_menu/gametype/gametype.h",
        },
        Define {
            name: "get_game_type",
            file: "aphavoc/source/ui_menu/gametype/gametype.h",
        },
        Include("entity/special/waypoint/waypoint.h"),
        Regex {
            pattern: r"\ntypedef struct UI_OBJECT ui_object;",
            file: "modules/userint2/ui_sys/ui_types/ui_types.h",
        },
        Include("entity/special/pilot/pilot.h"),
        Prototype {
            name: "convert_float_to_int",
            file: "modules/system/fpu.h",
        },
        Prototype {
            name: "set_comms_data_flow",
            file: "aphavoc/source/comms/comms.h",
        },
        Prototype {
            name: "set_comms_model",
            file: "aphavoc/source/comms/comms.h",
        },
        Prototype {
            name: "update_imap_surface_to_air_defence_level",
            file: "aphavoc/source/ai/highlevl/imaps.h",
        },
        Prototype {
            name: "update_imap_surface_to_surface_defence_level",
            file: "aphavoc/source/ai/highlevl/imaps.h",
        },
        Prototype {
            name: "get_local_group_member_landing_entity_from_keysite",
            file: "aphavoc/source/entity/special/landing/landing.h",
        },
        Prototype {
            name: "destroy_local_sound_effects",
            file: "aphavoc/source/entity/special/effect/soundeff/soundeff.h",
        },
        Prototype {
            name: "destroy_client_server_sound_effects",
            file: "aphavoc/source/entity/special/effect/soundeff/soundeff.h",
        },
        Prototype {
            name: "create_local_camera_entity",
            file: "aphavoc/source/entity/special/camera/cm_creat.h",
        },
        Prototype {
            name: "destroy_local_camera_entity",
            file: "aphavoc/source/entity/special/camera/cm_dstry.h",
        },
        Prototype {
            name: "create_local_bridge_entities",
            file: "aphavoc/source/entity/special/bridge/bridge.h",
        },
        Prototype {
            name: "create_local_pylon_entities",
            file: "aphavoc/source/entity/fixed/pylon/py_creat.h",
        },
        Prototype {
            name: "destroy_local_pylon_entities",
            file: "aphavoc/source/entity/fixed/pylon/py_dstry.h",
        },
        Prototype {
            name: "get_object_3d_bounding_box",
            file: "modules/3d/3dobjvis.h",
        },
        Prototype {
            name: "assert_local_create_entity_index",
            file: "aphavoc/source/entity/system/en_debug/en_valid.h",
        },
        Prototype {
            name: "assert_remote_create_entity_index",
            file: "aphavoc/source/entity/system/en_debug/en_valid.h",
        },
        Define {
            name: "validate_local_create_entity_index",
            file: "aphavoc/source/entity/system/en_debug/en_valid.h",
        },
        Define {
            name: "validate_remote_create_entity_index",
            file: "aphavoc/source/entity/system/en_debug/en_valid.h",
        },
        Regex {
            pattern: r"\nextern entity\n\t\*session_entity;",
            file: "aphavoc/source/entity/special/session/session.h",
        },
        Define {
            name: "get_session_entity",
            file: "aphavoc/source/entity/special/session/session.h",
        },
        Local("eech_native_decls.h"),
    ]
}

fn extracted_c() -> Vec<Spec> {
    vec![
        Raw("#include \"project.h\"\n\n#define DEBUG_MODULE 0\n\n#define LANDING_DEBUG 0\n".into()),
        Define {
            name: "DELTA_TIME_HISTORY_SIZE",
            file: "modules/system/time.c",
        },
        Regex {
            pattern: r"\nfloat\n\tsystem_delta_time_average = 0\.1,\n\tsystem_delta_time = 0\.1,\n\tsystem_one_over_delta_time = 10\.0;",
            file: "modules/system/time.c",
        },
        Regex {
            pattern: r"\nint\n\tlocked_frame_rate = FALSE;",
            file: "modules/system/time.c",
        },
        Regex {
            pattern: r"\nstatic float\n\tsystem_delta_time_history\[DELTA_TIME_HISTORY_SIZE\];",
            file: "modules/system/time.c",
        },
        Regex {
            pattern: r"\nstatic int\n\tsystem_delta_time_history_position;",
            file: "modules/system/time.c",
        },
        Function {
            name: "set_manual_delta_time",
            signature: "void set_manual_delta_time (float delta_time)",
            file: "modules/system/time.c",
        },
        Function {
            name: "get_2d_range",
            signature: "float get_2d_range (const vec3d *v1, const vec3d *v2)",
            file: "modules/maths/range.c",
        },
        Function {
            name: "get_approx_2d_range",
            signature: "float get_approx_2d_range (const vec3d *v1, const vec3d *v2)",
            file: "modules/maths/range.c",
        },
        Function {
            name: "default_set_entity_int_value",
            signature: "static void default_set_entity_int_value (entity *en, int_types type, int value)",
            file: "aphavoc/source/entity/system/en_funcs/en_int.c",
        },
        Function {
            name: "default_set_entity_float_value",
            signature: "static void default_set_entity_float_value (entity *en, float_types type, float value)",
            file: "aphavoc/source/entity/system/en_funcs/en_float.c",
        },
        Function {
            name: "notify_local_entity",
            signature: "int notify_local_entity (entity_messages message, entity *receiver, entity *sender, ...)",
            file: "aphavoc/source/entity/system/en_msgs/en_msgs.c",
        },
        Function {
            name: "default_message_response",
            signature: "static int default_message_response (entity_messages message, entity *receiver, entity *sender, va_list pargs)",
            file: "aphavoc/source/entity/system/en_msgs/en_msgs.c",
        },
        Function {
            name: "insert_local_entity_into_parents_child_list",
            signature: "void insert_local_entity_into_parents_child_list (entity *en, list_types type, entity *parent, entity *pred)",
            file: "aphavoc/source/entity/system/en_funcs/en_list.c",
        },
        Function {
            name: "delete_local_entity_from_parents_child_list",
            signature: "void delete_local_entity_from_parents_child_list (entity *en, list_types type)",
            file: "aphavoc/source/entity/system/en_funcs/en_list.c",
        },
        Function {
            name: "unlink_local_entity_children",
            signature: "void unlink_local_entity_children (entity *en, list_types list)",
            file: "aphavoc/source/entity/system/en_funcs/en_list.c",
        },
        Function {
            name: "default_get_entity_int_value",
            signature: "static int default_get_entity_int_value (entity *en, int_types type)",
            file: "aphavoc/source/entity/system/en_funcs/en_int.c",
        },
        Function {
            name: "default_get_entity_float_value",
            signature: "static float default_get_entity_float_value (entity *en, float_types type)",
            file: "aphavoc/source/entity/system/en_funcs/en_float.c",
        },
        Function {
            name: "assert_local_create_entity_index",
            signature: "int assert_local_create_entity_index (int index)",
            file: "aphavoc/source/entity/system/en_debug/en_valid.c",
        },
        Function {
            name: "assert_remote_create_entity_index",
            signature: "int assert_remote_create_entity_index (int index)",
            file: "aphavoc/source/entity/system/en_debug/en_valid.c",
        },
        Regex {
            pattern: r"\nstatic int\n\tentity_count,\n\tentity_peak_count;",
            file: "aphavoc/source/entity/system/en_debug/en_stats.c",
        },
        Regex {
            pattern: r"\nstatic struct\n\{[^}]*\}\nentity_type_stats\[NUM_ENTITY_TYPES\];",
            file: "aphavoc/source/entity/system/en_debug/en_stats.c",
        },
        Function {
            name: "update_create_entity_statistics",
            signature: "void update_create_entity_statistics (entity_types type)",
            file: "aphavoc/source/entity/system/en_debug/en_stats.c",
        },
        Function {
            name: "update_destroy_entity_statistics",
            signature: "void update_destroy_entity_statistics (entity *en)",
            file: "aphavoc/source/entity/system/en_debug/en_stats.c",
        },
        Regex {
            pattern: r"\nworld_map_data\n\tworld_map;",
            file: "aphavoc/source/entity/system/en_main/en_world.c",
        },
        Function {
            name: "set_entity_world_map_size",
            signature: "void set_entity_world_map_size (int num_map_x_sectors, int num_map_z_sectors, int sector_side_length)",
            file: "aphavoc/source/entity/system/en_main/en_world.c",
        },
        Function {
            name: "int_bit_count",
            signature: "int int_bit_count (unsigned int value)",
            file: "aphavoc/source/misc/miscell.c",
        },
        Function {
            name: "get_identity_matrix3x3",
            signature: "void get_identity_matrix3x3 (matrix3x3 m)",
            file: "modules/maths/matrix.c",
        },
        Regex {
            pattern: r"\nentity\n\t\*\*entity_sector_map = NULL;",
            file: "aphavoc/source/entity/special/sector/sector.c",
        },
        Function {
            name: "get_local_sector_entity",
            signature: "entity *get_local_sector_entity (vec3d *pos)",
            file: "aphavoc/source/entity/special/sector/sector.c",
        },
        Function {
            name: "add_mobile_values_to_sector",
            signature: "void add_mobile_values_to_sector (entity *sector_en, entity *mobile_en)",
            file: "aphavoc/source/entity/special/sector/sector.c",
        },
        Function {
            name: "remove_mobile_values_from_sector",
            signature: "void remove_mobile_values_from_sector (entity *sector_en, entity *mobile_en)",
            file: "aphavoc/source/entity/special/sector/sector.c",
        },
        Function {
            name: "destroy_client_server_sound_effects",
            signature: "void destroy_client_server_sound_effects (entity *en)",
            file: "aphavoc/source/entity/special/effect/soundeff/soundeff.c",
        },
        Regex {
            pattern: r"\nentity\n\t\*update_entity = NULL,\n\t\*update_succ = NULL;",
            file: "aphavoc/source/entity/special/update/up_update.c",
        },
        Regex {
            pattern: r"\nunsigned int\n\tmoved_entities[^;]*;",
            file: "aphavoc/source/entity/special/update/up_update.c",
        },
        Function {
            name: "update_client_server_entities",
            signature: "void update_client_server_entities (void)",
            file: "aphavoc/source/entity/special/update/up_update.c",
        },
        Function {
            name: "set_entity_update_frame_rate",
            signature: "int set_entity_update_frame_rate (int frame_rate)",
            file: "aphavoc/source/entity/special/update/up_update.c",
        },
        Function {
            name: "normalise_any_3d_vector",
            signature: "float normalise_any_3d_vector ( vec3d *vector )",
            file: "modules/maths/vector.c",
        },
        Function {
            name: "bound_position_to_adjusted_map_area",
            signature: "int bound_position_to_adjusted_map_area (vec3d *position)",
            file: "aphavoc/source/entity/system/en_main/en_world.c",
        },
        Function {
            name: "get_local_sector_entity_enemy_defence_level",
            signature: "static float get_local_sector_entity_enemy_defence_level (float *array, entity_sides side)",
            file: "aphavoc/source/entity/special/sector/sector.c",
        },
        Function {
            name: "get_local_sector_entity_enemy_surface_to_air_defence_level",
            signature: "float get_local_sector_entity_enemy_surface_to_air_defence_level (entity *sector_en, entity_sides side)",
            file: "aphavoc/source/entity/special/sector/sector.c",
        },
        Function {
            name: "set_client_server_entity_parent",
            signature: "void set_client_server_entity_parent (entity *en, list_types type, entity *parent)",
            file: "aphavoc/source/entity/system/en_funcs/en_list.c",
        },
        Function {
            name: "notify_campaign_screen",
            signature: "int notify_campaign_screen (campaign_screen_messages message, entity *sender)",
            file: "aphavoc/source/ui_menu/ingame/campaign/ca_msgs.c",
        },
        Function {
            name: "get_local_force_entity",
            signature: "entity *get_local_force_entity (entity_sides side)",
            file: "aphavoc/source/entity/special/force/force.c",
        },
        Function {
            name: "assess_group_supplies",
            signature: "void assess_group_supplies (entity *en)",
            file: "aphavoc/source/entity/special/group/group.c",
        },
        Function {
            name: "assess_group_task_locality_factor",
            signature: "int assess_group_task_locality_factor (entity *group_en, entity *task_en, float *return_distance)",
            file: "aphavoc/source/entity/special/group/group.c",
        },
        Function {
            name: "response_to_link_parent",
            signature: "static int response_to_link_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)",
            file: "aphavoc/source/entity/special/group/gp_msgs.c",
        },
        Function {
            name: "response_to_unlink_parent",
            signature: "static int response_to_unlink_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)",
            file: "aphavoc/source/entity/special/group/gp_msgs.c",
        },
        Wrap {
            prologue: "void harness_overload_group_link_parent_responses (void)\n{",
            epilogue: "}",
            parts: vec![
                Regex {
                    pattern: r"\n\tmessage_responses\[ENTITY_TYPE_GROUP\]\[ENTITY_MESSAGE_LINK_PARENT\][^;]*;",
                    file: "aphavoc/source/entity/special/group/gp_msgs.c",
                },
                Regex {
                    pattern: r"\n\tmessage_responses\[ENTITY_TYPE_GROUP\]\[ENTITY_MESSAGE_UNLINK_PARENT\][^;]*;",
                    file: "aphavoc/source/entity/special/group/gp_msgs.c",
                },
            ],
        },
        Function {
            name: "response_to_link_child",
            signature: "static int response_to_link_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)",
            file: "aphavoc/source/entity/special/group/gp_msgs.c",
        },
        Wrap {
            prologue: "void harness_overload_group_link_child_response (void)\n{",
            epilogue: "}",
            parts: vec![Regex {
                pattern: r"\n\tmessage_responses\[ENTITY_TYPE_GROUP\]\[ENTITY_MESSAGE_LINK_CHILD\][^;]*;",
                file: "aphavoc/source/entity/special/group/gp_msgs.c",
            }],
        },
        Raw(concat!(
            "/* C defaults for the update entity: its link responses are only overloaded under DEBUG_MODULE (up_msgs.c) */\n",
            "void harness_default_update_link_responses (void)\n{\n",
            "\tmessage_responses[ENTITY_TYPE_UPDATE][ENTITY_MESSAGE_LINK_CHILD] = default_message_response;\n",
            "\tmessage_responses[ENTITY_TYPE_UPDATE][ENTITY_MESSAGE_LINK_PARENT] = default_message_response;\n",
            "\tmessage_responses[ENTITY_TYPE_UPDATE][ENTITY_MESSAGE_UNLINK_PARENT] = default_message_response;\n}\n\n",
            "void (*harness_default_set_entity_int_value) (entity *en, int_types type, int value) = default_set_entity_int_value;\n\n",
            "void (*harness_default_set_entity_float_value) (entity *en, float_types type, float value) = default_set_entity_float_value;\n\n",
            "int (*harness_default_get_entity_int_value) (entity *en, int_types type) = default_get_entity_int_value;\n\n",
            "float (*harness_default_get_entity_float_value) (entity *en, float_types type) = default_get_entity_float_value;\n\n",
            // NATIVE: the statics of this unit are campaign (or scratch) state; a
            // new campaign instance starts from their initial values
            "/* NATIVE: reset of this unit's file-scope statics (eech-campaign/docs/global-state.md) */\n",
            "void eech_reset_extracted_statics (void)\n{\n",
            "\tmemset (system_delta_time_history, 0, sizeof (system_delta_time_history));\n",
            "\tsystem_delta_time_history_position = 0;\n",
            "\tentity_count = 0;\n",
            "\tentity_peak_count = 0;\n",
            "\tmemset (entity_type_stats, 0, sizeof (entity_type_stats));\n}\n",
        )
        .into()),
    ]
}

/// Generated translation units of verbatim extracts (one per C scope).
pub fn extracted_units() -> Vec<(&'static str, Vec<Spec>)> {
    vec![
        ("eech_extracted.c", extracted_c()),
        (
            "eech_extracted_assign.c",
            vec![
                Raw("#include <limits.h>\n\n#include \"project.h\"\n\n#include \"ai/taskgen/assign.h\"\n#include \"ai/taskgen/taskgen.h\"\n#include \"ai/highlevl/suitable.h\"\n".into()),
                Prototype { name: "quicksort_entity_list", file: "aphavoc/source/entity/en_misc/en_misc.h" },
                Regex { pattern: r"\n#define ai_log\(a, x\.\.\.\) do \{ \} while\(0\);", file: "aphavoc/source/ai/highlevl/highlevl.h" },
                Define { name: "DEBUG_MODULE", file: "aphavoc/source/ai/taskgen/assign.c" },
                Function { name: "assign_keysite_tasks", signature: "void assign_keysite_tasks (entity *keysite, task_category_types category)", file: "aphavoc/source/ai/taskgen/assign.c" },
                Function { name: "suitable_group_task_specific_checks", signature: "static int suitable_group_task_specific_checks (entity *task, entity *group)", file: "aphavoc/source/ai/taskgen/assign.c" },
                Function { name: "get_suitable_registered_group", signature: "entity *get_suitable_registered_group (entity *task, int *idle_group_count)", file: "aphavoc/source/ai/taskgen/assign.c" },
                Function { name: "check_group_members_awake", signature: "int check_group_members_awake (entity *group)", file: "aphavoc/source/ai/taskgen/assign.c" },
            ],
        ),
        (
            "eech_extracted_en_misc.c",
            vec![
                Raw("#include \"project.h\"\n".into()),
                Function { name: "qs", signature: "static void qs (entity **en_list, float *sort_order, int left, int right)", file: "aphavoc/source/entity/en_misc/en_misc.c" },
                Function { name: "quicksort_entity_list", signature: "void quicksort_entity_list (entity **en_list, int count, float *sort_order)", file: "aphavoc/source/entity/en_misc/en_misc.c" },
            ],
        ),
        (
            "eech_extracted_taskgen.c",
            vec![
                Raw("#include \"project.h\"\n\n#include \"ai/taskgen/taskgen.h\"\n".into()),
                Define { name: "DEBUG_MODULE", file: "aphavoc/source/ai/taskgen/taskgen.c" },
                Define { name: "TASK_SAFE_LIMIT", file: "aphavoc/source/ai/taskgen/taskgen.c" },
                Regex { pattern: r"\nvec3d\n   terminator_point = \{-1\.0, -1\.0, -1\.0\};", file: "aphavoc/source/ai/taskgen/taskgen.c" },
                Regex { pattern: r"\nstatic int get_task_start_keysite \(entity_sub_types sub_type, entity_sides side, vec3d \*start_pos, entity \*\*start_keysite\);", file: "aphavoc/source/ai/taskgen/taskgen.c" },
                Function { name: "create_task", signature: "entity *create_task", file: "aphavoc/source/ai/taskgen/taskgen.c" },
                Function { name: "create_supply_task", signature: "entity *create_supply_task (entity *requester, entity *supplier, entity *cargo, movement_types movement_type, float priority, entity *start_keysite, entity *end_keysite)", file: "aphavoc/source/ai/taskgen/taskgen.c" },
                Function { name: "validate_task_generation", signature: "int validate_task_generation (entity_sides side, entity_sub_types sub_type)", file: "aphavoc/source/ai/taskgen/taskgen.c" },
                Function { name: "get_task_start_keysite", signature: "int get_task_start_keysite (entity_sub_types sub_type, entity_sides side, vec3d *start_pos, entity **start_keysite)", file: "aphavoc/source/ai/taskgen/taskgen.c" },
            ],
        ),
        (
            "eech_extracted_ts_msgs.c",
            vec![
                Raw("#include \"project.h\"\n\n#define DEBUG_MODULE 0\n".into()),
                Function { name: "response_to_link_parent", signature: "static int response_to_link_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/special/task/ts_msgs.c" },
                Wrap {
                    prologue: "void harness_overload_task_link_parent_response (void)\n{",
                    epilogue: "}",
                    parts: vec![Regex { pattern: r"\n\tmessage_responses\[ENTITY_TYPE_TASK\]\[ENTITY_MESSAGE_LINK_PARENT\][^;]*;", file: "aphavoc/source/entity/special/task/ts_msgs.c" }],
                },
            ],
        ),
        (
            "eech_extracted_en_vec3d.c",
            vec![
                Raw("#include \"project.h\"\n".into()),
                Define { name: "DEBUG_MODULE_PACK_ONE", file: "aphavoc/source/entity/system/en_funcs/en_vec3d.c" },
                Define { name: "DEBUG_MODULE_PACK_ALL", file: "aphavoc/source/entity/system/en_funcs/en_vec3d.c" },
                Regex { pattern: r"\nvec3d_type_data\n\tvec3d_type_database\[NUM_VEC3D_TYPES\] =\n\t\{[\s\S]*?\n\t\};", file: "aphavoc/source/entity/system/en_funcs/en_vec3d.c" },
                Function { name: "pack_vec3d", signature: "void pack_vec3d (entity *en, vec3d_types type, vec3d *v)", file: "aphavoc/source/entity/system/en_funcs/en_vec3d.c" },
                Function { name: "bound_position_to_map_volume", signature: "int bound_position_to_map_volume (vec3d *position)", file: "aphavoc/source/entity/system/en_main/en_world.c" },
            ],
        ),
        (
            "eech_extracted_ac_msgs.c",
            vec![
                Raw("#include \"project.h\"\n\n#define DEBUG_MODULE 0\n".into()),
                Define { name: "AIR_RADAR_CONTACT_TIMEOUT", file: "aphavoc/source/gunships/avionics/common/co_radar.h" },
                Function { name: "response_to_link_parent", signature: "static int response_to_link_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/mobile/aircraft/ac_msgs.c" },
                Function { name: "response_to_unlink_parent", signature: "static int response_to_unlink_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/mobile/aircraft/ac_msgs.c" },
                Wrap {
                    prologue: "void harness_overload_aircraft_link_parent_responses (entity_types type)\n{",
                    epilogue: "}",
                    parts: vec![
                        Regex { pattern: r"\n\tmessage_responses\[type\]\[ENTITY_MESSAGE_LINK_PARENT\][^;]*;", file: "aphavoc/source/entity/mobile/aircraft/ac_msgs.c" },
                        Regex { pattern: r"\n\tmessage_responses\[type\]\[ENTITY_MESSAGE_UNLINK_PARENT\][^;]*;", file: "aphavoc/source/entity/mobile/aircraft/ac_msgs.c" },
                    ],
                },
            ],
        ),
        (
            "eech_extracted_ks_msgs.c",
            vec![
                Raw("#include \"project.h\"\n\n#define DEBUG_MODULE 0\n".into()),
                Function { name: "response_to_link_child", signature: "static int response_to_link_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/special/keysite/ks_msgs.c" },
                Function { name: "response_to_unlink_child", signature: "static int response_to_unlink_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)", file: "aphavoc/source/entity/special/keysite/ks_msgs.c" },
                Wrap {
                    prologue: "void harness_overload_keysite_link_child_responses (void)\n{",
                    epilogue: "}",
                    parts: vec![
                        Regex { pattern: r"\n\tmessage_responses\[ENTITY_TYPE_KEYSITE\]\[ENTITY_MESSAGE_LINK_CHILD\][^;]*;", file: "aphavoc/source/entity/special/keysite/ks_msgs.c" },
                        Regex { pattern: r"\n\tmessage_responses\[ENTITY_TYPE_KEYSITE\]\[ENTITY_MESSAGE_UNLINK_CHILD\][^;]*;", file: "aphavoc/source/entity/special/keysite/ks_msgs.c" },
                    ],
                },
            ],
        ),
    ]
}

/// Compile definitions for one generated unit (as extract.mjs UNIT_FLAGS).
///
/// F1 (eech-core-ts docs/slices/supply-task-construction.md): create_supply_task
/// leaves prepare.y and finish.y uninitialised and create_task reads them. The
/// C reference pins the unit to zero-initialised automatic variables; the
/// native kernel inherits the same compatibility decision (not a claim that
/// EECH produced 0.0).
pub const ZERO_INIT_UNITS: &[&str] = &["eech_extracted_taskgen.c"];
