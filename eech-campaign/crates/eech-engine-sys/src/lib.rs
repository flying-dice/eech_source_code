//! Raw FFI to the headless EECH engine (csrc/eech_engine.h). Consumed only by
//! the safe `eech-engine` crate.

#![allow(non_camel_case_types)]

use std::ffi::{c_char, c_int, c_void};

pub const EECH_ENGINE_OK: c_int = 0;
pub const EECH_ENGINE_FATAL: c_int = 1;
pub const EECH_ENGINE_POISONED: c_int = 2;
pub const EECH_ENGINE_STATE: c_int = 3;
pub const EECH_ENGINE_BAD_ARGUMENT: c_int = 4;
pub const EECH_HOST_ID_MAX: usize = 128;
pub const EECH_LOSS_EVENT_CREDIT: c_int = 1;
pub const EECH_LOSS_EVENT_PHYSICAL: c_int = 2;
pub const EECH_LOSS_FORWARD: c_int = 1;
pub const EECH_LOSS_WITHHOLD: c_int = 2;
pub const EECH_LOSS_TASK_DEPENDENT_MAX: usize = 32;
pub const EECH_LOSS_LANDING_ACTION_MAX: usize = 3;
pub const EECH_LOSS_LANDING_UNRESERVE_SITE: c_int = 1;
pub const EECH_LOSS_LANDING_UNLOCK_ROUTE: c_int = 2;
pub const EECH_LOSS_LANDING_UNLOCK_TAKEOFF_ROUTE: c_int = 3;
pub const EECH_LOSS_LANDING_UNLOCK_SITE: c_int = 4;
pub const CAMPAIGN_SUPPLY_WAYPOINT_PICK_UP: c_int = 1;
pub const CAMPAIGN_SUPPLY_WAYPOINT_PREPARE_FOR_DROP_OFF: c_int = 2;
pub const CAMPAIGN_SUPPLY_WAYPOINT_DROP_OFF: c_int = 3;
pub const CAMPAIGN_SUPPLY_WAYPOINT_FINISH_DROP_OFF: c_int = 4;
pub const CAMPAIGN_RADIUS_ARRIVAL_WAYPOINT: c_int = 5;
pub const CAMPAIGN_SUPPLY_MOBILE_FIXED_WING: c_int = 1;
pub const CAMPAIGN_SUPPLY_MOBILE_HELICOPTER: c_int = 2;
pub const CAMPAIGN_SUPPLY_MOBILE_ROUTED_VEHICLE: c_int = 3;
pub const CAMPAIGN_SUPPLY_MOBILE_SHIP_VEHICLE: c_int = 4;
pub const CAMPAIGN_SUPPLY_MOBILE_ANTI_AIRCRAFT: c_int = 5;
pub const CAMPAIGN_SUPPLY_MOBILE_PERSON: c_int = 6;
pub const CAMPAIGN_ARRIVAL_QUERY_REACHED: c_int = 1;
pub const CAMPAIGN_ARRIVAL_QUERY_ACTION: c_int = 2;
pub const CAMPAIGN_ARRIVAL_QUERY_RECON: c_int = 3;
pub const CAMPAIGN_HEADING_SOURCE_NONE: c_int = 0;
pub const CAMPAIGN_HEADING_SOURCE_WAYPOINT: c_int = 1;
pub const CAMPAIGN_HEADING_SOURCE_ZERO: c_int = 2;
pub const CAMPAIGN_HEADING_SOURCE_TARGET: c_int = 3;

#[repr(C)]
pub struct eech_engine_config {
    pub install_root: *const c_char,
    pub map_path: *const c_char,
    pub campaign_directory: *const c_char,
    pub campaign_filename: *const c_char,
    pub gunship_type: c_int,
    pub random_seed: u32,
    pub arguments: *const *const c_char,
}

#[repr(C)]
pub struct eech_member_descriptor {
    pub member_key: u64,
    pub group_key: u64,
    pub entity_index: c_int,
    pub entity_type: c_int,
    pub sub_type: c_int,
    pub side: c_int,
    pub has_pose: c_int,
    pub position: [f32; 3],
    pub velocity: f32,
    pub attitude: [f32; 9],
    pub movement_kind: c_int,
    pub motion_vector: [f32; 3],
    pub rotor: [f32; 3],
}

pub type eech_bind_mobile_callback = unsafe extern "C" fn(
    descriptor: *const eech_member_descriptor,
    host_id: *mut c_char,
    host_id_size: usize,
    error: *mut c_char,
    error_size: usize,
    user: *mut c_void,
) -> c_int;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct eech_loss_task_dependent {
    pub task_key: u64,
    pub entity_type: c_int,
    pub sub_type: c_int,
    pub task_state: c_int,
    pub task_completed: c_int,
    pub primary_task: c_int,
    pub persistent_task: c_int,
    pub objective_is_member: c_int,
    pub task_dependent_children: c_int,
    pub guide_children: c_int,
    pub expire_timer: f32,
    pub stop_timer: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct eech_loss_landing_action {
    pub landing_key: u64,
    pub action_kind: c_int,
    pub count: c_int,
    pub landing_sub_type: c_int,
    pub reserved_landing_sites: c_int,
    pub free_landing_sites: c_int,
    pub landing_lock: c_int,
    pub landed_lock: c_int,
    pub takeoff_lock: c_int,
}

#[repr(C)]
pub struct eech_loss_descriptor {
    pub host_id: *const c_char,
    pub member_key: u64,
    pub group_key: u64,
    pub event_kind: c_int,
    pub entity_type: c_int,
    pub sub_type: c_int,
    pub side: c_int,
    pub force_info_category: c_int,
    pub group_sub_type: c_int,
    pub primary_task_sub_type: c_int,
    pub primary_task_key: u64,
    pub group_member_number: c_int,
    pub campaign_member_count: c_int,
    pub player: c_int,
    pub landed: c_int,
    pub member_task_dependent_count: c_int,
    pub member_task_dependent_copied: c_int,
    pub member_task_dependents: [eech_loss_task_dependent; EECH_LOSS_TASK_DEPENDENT_MAX],
    pub group_task_dependents: c_int,
    pub restricted_guides: c_int,
    pub landing_action_count: c_int,
    pub landing_action_copied: c_int,
    pub landing_actions: [eech_loss_landing_action; EECH_LOSS_LANDING_ACTION_MAX],
    pub movement_parent: c_int,
    pub movement_dependents: c_int,
    pub takeoff_queue: c_int,
}

pub type eech_observe_loss_callback =
    unsafe extern "C" fn(descriptor: *const eech_loss_descriptor, decision: *mut c_int, error: *mut c_char, error_size: usize, user: *mut c_void) -> c_int;

#[repr(C)]
pub struct eech_supply_reach_descriptor {
    pub member_key: u64,
    pub task_key: u64,
    pub waypoint_key: u64,
    pub host_id: [c_char; EECH_HOST_ID_MAX + 1],
    pub query: c_int,
    pub waypoint_kind: c_int,
    pub waypoint_sub_type: c_int,
    pub player_skip_waypoint: c_int,
    pub landing_waypoint: c_int,
    pub mobile_kind: c_int,
    pub is_task_leader: c_int,
    pub is_ai: c_int,
    pub leader_is_ai: c_int,
    pub landed: c_int,
    pub updated: c_int,
    pub rotor_spin_up_seconds: f32,
    pub distance: f32,
    pub velocity: f32,
    pub delta_seconds: f32,
    pub radius: f32,
    pub position: [f32; 3],
    pub motion: [f32; 3],
    pub guide_position: [f32; 3],
    pub leader_position: [f32; 3],
    pub radius_valid: c_int,
    pub altitude_valid: c_int,
    pub heading_valid: c_int,
    pub last_to_reach_valid: c_int,
    pub weapon_gate: c_int,
    pub vectors_valid: c_int,
    pub airborne: c_int,
    pub auto_pilot: c_int,
    pub altitude_criterion: f32,
    pub heading_criterion: f32,
    pub altitude: f32,
    pub heading: f32,
    pub centre_of_gravity_to_ground: f32,
    pub guide_altitude: f32,
    pub heading_source: c_int,
    pub waypoint_heading: f32,
    pub heading_target_present: c_int,
    pub heading_target_position_present: c_int,
    pub heading_target_position: [f32; 3],
    pub weapon_vector: [f32; 3],
    pub intercept_vector: [f32; 3],
    pub action_radius: f32,
    pub waypoint_position: [f32; 3],
}

#[repr(C)]
pub struct eech_arrival_result {
    pub gates_passed: c_int,
    pub angle_present: c_int,
    pub launch_angle_error: f32,
    pub heading_target_missing: c_int,
}

pub type eech_supply_reached_callback = unsafe extern "C" fn(
    descriptor: *const eech_supply_reach_descriptor,
    result: *mut eech_arrival_result,
    error: *mut c_char,
    error_size: usize,
    user: *mut c_void,
) -> c_int;

pub const EECH_MOVEMENT_HOST_ID_MAX: usize = 128;
pub const EECH_MOVEMENT_MOVE: c_int = 1;
pub const EECH_MOVEMENT_STOP: c_int = 2;
pub const EECH_MOVEMENT_PLACE: c_int = 3;
pub const EECH_MOVEMENT_SHIP_MOVE: c_int = 4;
pub const EECH_MOVEMENT_PERSON_BEGIN: c_int = 5;
pub const EECH_MOVEMENT_PERSON_END: c_int = 6;
pub const EECH_MOVEMENT_FIXED_WING_MOVE: c_int = 7;
pub const EECH_MOVEMENT_HELICOPTER_BEGIN: c_int = 8;
pub const EECH_MOVEMENT_HELICOPTER_END: c_int = 9;
pub const EECH_MOVEMENT_ROTOR_SET: c_int = 10;
pub const EECH_MOVEMENT_ROTOR_ALIVE: c_int = 11;
pub const EECH_MOVEMENT_ROTOR_DOWN: c_int = 12;
pub const EECH_MOVEMENT_FALL: c_int = 13;
pub const EECH_MOVEMENT_CONTACT: c_int = 14;
pub const EECH_MOVEMENT_SETTLE: c_int = 15;
pub const EECH_MOVEMENT_TUMBLE: c_int = 16;
pub const EECH_MOVEMENT_CRASH_BEGIN: c_int = 17;
pub const EECH_MOVEMENT_CRASH_END: c_int = 18;
pub const EECH_MOVEMENT_IMPACT: c_int = 19;
pub const EECH_MOVEMENT_FALL_GRAVITY: c_int = 20;
pub const EECH_MOVEMENT_ROTOR_IDLE: c_int = 21;
pub const EECH_MOVEMENT_GV_DEATH_CANDIDATE: c_int = 22;
pub const EECH_MOVEMENT_GV_DEATH_DECAY: c_int = 23;
pub const EECH_MOVEMENT_GV_FALL_CANDIDATE: c_int = 24;
pub const EECH_MOVEMENT_GV_FALL_GRAVITY: c_int = 25;
pub const EECH_MOVEMENT_GV_FALL_SETTLE: c_int = 26;
pub const EECH_MOVEMENT_GV_IMPACT: c_int = 27;
pub const EECH_MOVEMENT_SHIP_SINK: c_int = 28;
pub const EECH_MOVEMENT_VEHICLE_VECTOR_SEED: c_int = 29;
pub const EECH_MOVEMENT_FW_DEATH_ATTITUDE: c_int = 30;
pub const EECH_MOVEMENT_FW_CRASH_CANDIDATE: c_int = 31;
pub const EECH_MOVEMENT_FW_CRASH_VELOCITY: c_int = 32;
pub const EECH_MOVEMENT_GV_FALL_VECTOR: c_int = 33;
pub const EECH_MOVEMENT_GV_FALL_COMMIT: c_int = 34;
pub const EECH_MOVEMENT_FW_SETTLE: c_int = 35;
pub const EECH_MOVEMENT_FW_CRASH_ATTITUDE: c_int = 36;
pub const EECH_MOVEMENT_RETIRE: c_int = 37;
pub const EECH_MOVEMENT_CREATE_PLACE: c_int = 38;
pub const EECH_MOVEMENT_EXPIRE: c_int = 39;
/// EEDCS-39 hosted helicopter control: plan (answers the lookup `hc_query`), guide (answers the `hc_guidance`), the hosted
/// begin and end, and the abandon of an open step.
pub const EECH_MOVEMENT_HELICOPTER_PLAN: c_int = 40;
pub const EECH_MOVEMENT_HELICOPTER_GUIDE: c_int = 41;
pub const EECH_MOVEMENT_HELICOPTER_GUIDED_BEGIN: c_int = 42;
pub const EECH_MOVEMENT_HELICOPTER_GUIDED_END: c_int = 43;
pub const EECH_MOVEMENT_HELICOPTER_ABANDON: c_int = 44;
/// EEDCS-39 R1: the fixed-wing terrain look-ahead query. Non-mutating; answers `hc_query` with the host's own look-ahead point
/// as the single terrain sample.
pub const EECH_MOVEMENT_FIXED_WING_LOOKAHEAD: c_int = 45;
/// EEDCS-40: a movement-dependent mobile follows its parent; the host validates the parent's move and answers the dependent's pose.
pub const EECH_MOVEMENT_DEPENDENT_CARRY: c_int = 46;
/// EEDCS-44: an infantry member with a target turns to face it; the host forms the heading and answers the attitude and the heading (`angles[0]`).
pub const EECH_MOVEMENT_PERSON_FACE: c_int = 47;
/// The terrain samples a helicopter's guidance asks for at most (HELICOPTER_TERRAIN_AVOIDANCE_LOOKAHEAD_SAMPLE_NUMBER + 1).
pub const EECH_HC_TERRAIN_SAMPLES: usize = 2;

pub const EECH_MOVEMENT_KIND_ROUTED: c_int = 1;
pub const EECH_MOVEMENT_KIND_SHIP: c_int = 2;
pub const EECH_MOVEMENT_KIND_PERSON: c_int = 3;
pub const EECH_MOVEMENT_KIND_FIXED_WING: c_int = 4;
pub const EECH_MOVEMENT_KIND_HELICOPTER: c_int = 5;

/// A structure of the aircraft's own sector, as the DLL's structure scan saw it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct eech_hc_structure {
    pub position: [f32; 3],
    pub avoidance_radius: f32,
    pub height: f32,
}

/// One terrain sample's getter returns, in call order: `second` is meaningful only when `second_present`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct eech_hc_terrain_sample {
    pub first: f32,
    pub second: f32,
    pub second_present: c_int,
}

/// What the host's plan asks the DLL to look up.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct eech_hc_query {
    pub structure_scan: c_int,
    pub structure_position: [f32; 3],
    pub terrain_sample_count: c_int,
    pub terrain_sample: [[f32; 2]; EECH_HC_TERRAIN_SAMPLES],
}

/// The host's guidance for the pre-begin physical state.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct eech_hc_guidance_answer {
    pub lift: f32,
    pub waypoint: [f32; 3],
    pub required_height: f32,
    pub world_acceleration: [f32; 3],
    pub wp_vec: [f32; 3],
}

#[repr(C)]
pub struct eech_movement_step {
    pub host_id: [c_char; EECH_MOVEMENT_HOST_ID_MAX + 1],
    pub mode: c_int,
    pub position: [f32; 3],
    pub velocity: f32,
    pub attitude: [f32; 9],
    pub delta_seconds: f32,
    pub desired_velocity: f32,
    pub max_acceleration: f32,
    pub waypoint: [f32; 3],
    pub tacview_logging: c_int,
    pub terrain_follow: c_int,
    pub terrain_present: c_int,
    pub elevation: f32,
    pub forest: c_int,
    pub face_normal: [f32; 3],
    pub map_min: [f32; 3],
    pub map_max: [f32; 3],
    pub frame_delta: f32,
    pub g_max: f32,
    pub ps_vel: f32,
    pub fw_phase: c_int,
    pub fw_motion_vector: [f32; 3],
    pub fw_cruise_velocity: f32,
    pub fw_fuselage_angle: f32,
    pub fw_cg_to_ground: f32,
    pub fw_max_roll: f32,
    pub fw_aiming_cannon: c_int,
    pub fw_elapsed_time: f32,
    pub fw_navigating: c_int,
    pub fw_group_leader: c_int,
    pub fw_task_leader: c_int,
    pub fw_max_vertical_turn_allowance: f32,
    pub fw_avoidance_radius: f32,
    /// the task leader's host id; empty when the aircraft leads its task or the campaign roster lacks the leader
    /// (unresolved: the host refuses the step only if it reads the leader)
    pub fw_leader_host_id: [c_char; EECH_MOVEMENT_HOST_ID_MAX + 1],
    pub fw_member_count: c_int,
    /// `fw_member_count` NUL-terminated host ids and radii, in the group's order; the caller owns both arrays and they
    /// are read only during the call. Null only when the count is zero. A null or empty entry is a member the roster
    /// lacks (unresolved, its place and radius kept); the host refuses only if it reads that member.
    pub fw_member_host_id: *const *const c_char,
    pub fw_member_radius: *const f32,
    pub hc_acceleration_x: f32,
    pub hc_acceleration_z: f32,
    pub hc_wp_vec_y: f32,
    pub hc_desired_heading: f32,
    pub hc_max_turn_rate: f32,
    pub hc_desired_pitch: f32,
    pub hc_desired_roll: f32,
    pub hc_cg_to_ground: f32,
    pub hc_motion_vector: [f32; 3],
    pub hc_rotor: [f32; 3],
    pub hc_lift: f32,
    pub hc_power_output: f32,
    pub hc_flag: c_int,
    pub hc_gravity: f32,
    pub hc_terrain_class: c_int,
    pub hc_bury_limit: f32,
    pub hc_pitch_offset: f32,
    pub hc_bank_offset: f32,
    pub hc_signed_speed: f32,
    pub gv_old_elevation: f32,
    pub gv_bbox_diagonal: f32,
    pub gv_wide: c_int,
    pub gv_water: c_int,
    pub death_timer: f32,
    pub death_range: f32,
    pub death_policy: c_int,
    /// EEDCS-39 helicopter plan (and the steering facts GUIDED_END shares): the campaign's task and waypoint facts. The plan
    /// also uses `delta_seconds`, `terrain_follow`, `elevation` (the cached terrain elevation), `waypoint`, `map_min`,
    /// `map_max`, `hc_motion_vector`, `hc_power_output`; GUIDED_END uses `delta_seconds`, `hc_max_turn_rate`, `elevation`,
    /// `hc_cg_to_ground`, `terrain_follow`, `map_min`, `map_max`, `g_max`, `hc_motion_vector`.
    pub hc_group_leader: c_int,
    pub hc_task_leader: c_int,
    pub hc_task_leader_is_self: c_int,
    pub hc_position_hold: c_int,
    pub hc_terrain_follow_attack_altitude: c_int,
    pub hc_waypoint_kind: c_int,
    pub hc_guide_radius_valid: c_int,
    pub hc_guide_radius: f32,
    pub hc_guide_altitude_valid: c_int,
    pub hc_waypoint_velocity: [f32; 3],
    pub hc_distance: f32,
    pub hc_fuel_supply_level: f32,
    pub hc_cruise_velocity: f32,
    pub hc_cruise_altitude: f32,
    pub hc_attack_altitude: f32,
    pub hc_leader_radar_altitude: f32,
    /// PLAN: the group's other members' NUL-terminated host ids in list order (null or empty entry: the roster lacks it).
    /// Caller-owned, read only during the call; null only when the count is zero.
    pub hc_member_count: c_int,
    pub hc_member_host_id: *const *const c_char,
    /// GUIDE: the structures of the aircraft's own sector (sector order, uncapped) and the terrain samples' getter returns
    /// in call order. Caller-owned, read only during the call; null only when the count is zero.
    pub hc_structure_count: c_int,
    pub hc_structures: *const eech_hc_structure,
    pub hc_terrain_sample_count: c_int,
    pub hc_terrain_samples: *const eech_hc_terrain_sample,
    /// GUIDED_END: the guide's required heading (campaign intent), the task leader's host id (empty: none), the targeting
    /// decision and the weapon vectors, and the rotor shaft angle
    pub hc_required_heading_valid: c_int,
    pub hc_required_heading: f32,
    pub hc_leader_host_id: [c_char; EECH_MOVEMENT_HOST_ID_MAX + 1],
    pub hc_targeting_active: c_int,
    pub hc_weapon_vector: [f32; 3],
    pub hc_weapon_to_intercept_vector: [f32; 3],
    pub hc_main_rotor_shaft_angle: f32,
    /// FIXED_WING_MOVE: the terrain observation for a terrain-following task leader's waypoint (`fw_terrain` false otherwise).
    pub fw_terrain: c_int,
    pub fw_terrain_min_height: f32,
    pub fw_terrain_guide_position: [f32; 3],
    pub fw_terrain_sample_point: [f32; 2],
    pub fw_terrain_sample_elevation: f32,
    pub fw_terrain_site_y: f32,
    pub fw_terrain_cg: f32,
    /// DEPENDENT_CARRY: the parent that notified the dependent, the parent's position before and after the move, the heading and
    /// speed its setter sent, and whether the dependent is a helicopter (it keeps its offset, turned by the heading change)
    pub dc_parent_host_id: [c_char; EECH_MOVEMENT_HOST_ID_MAX + 1],
    pub dc_helicopter: c_int,
    pub dc_parent_old: [f32; 3],
    pub dc_parent_new: [f32; 3],
    pub dc_heading: f32,
    pub dc_velocity: f32,
    /// PERSON_FACE: the target's position.
    pub pf_target: [f32; 3],
    /// The two tumble draws are `frand1 ()` doubles and stay doubles; last, so the double's alignment moves no other field.
    pub hc_draws: [f64; 2],
}

#[repr(C)]
pub struct eech_movement_result {
    pub position: [f32; 3],
    pub velocity: f32,
    pub attitude: [f32; 9],
    pub rotated: c_int,
    pub motion_vector: [f32; 3],
    pub pending: c_int,
    pub moved: c_int,
    pub rotor: [f32; 3],
    pub altitude: f32,
    pub flags: c_int,
    /// EEDCS-38 CREATE_PLACE: the heading, pitch and roll the host built the attitude from; zero in every other mode.
    pub angles: [f32; 3],
    pub death_timer: f32,
    /// HELICOPTER_PLAN: what the DLL must look up; zero in every other mode
    pub hc_query: eech_hc_query,
    /// HELICOPTER_GUIDE: the host's guidance; zero in every other mode
    pub hc_guidance: eech_hc_guidance_answer,
}

pub type eech_move_vehicle_callback =
    unsafe extern "C" fn(step: *const eech_movement_step, result: *mut eech_movement_result, error: *mut c_char, error_size: usize, user: *mut c_void) -> c_int;

pub const EECH_LOS_HOST_ID_MAX: usize = 128;
pub const EECH_LOS_BEGIN: c_int = 1;
pub const EECH_LOS_TERRAIN: c_int = 2;
pub const EECH_LOS_OBJECTS: c_int = 3;
pub const EECH_LOS_DONE: c_int = 1;
pub const EECH_LOS_NEED_TERRAIN: c_int = 2;
pub const EECH_LOS_NEED_OBJECTS: c_int = 3;
pub const EECH_LOS_PHASE_COARSE: c_int = 1;
pub const EECH_LOS_PHASE_SOURCE_FINE: c_int = 2;
pub const EECH_LOS_PHASE_TARGET_FINE: c_int = 3;
pub const EECH_LOS_PHASE_SOURCE_OBJECTS: c_int = 4;
pub const EECH_LOS_PHASE_TARGET_OBJECTS: c_int = 5;

/// One message of the host-owned line-of-sight dialogue (aphavoc/.../mb_hostlos.h): BEGIN carries the ray, TERRAIN and OBJECTS the
/// campaign's answers to what the host asked.
#[repr(C)]
pub struct eech_los_step {
    pub mode: c_int,
    pub entity_form: c_int,
    pub criteria: c_int,
    pub source_eye: c_int,
    pub target_eye: c_int,
    pub source_host_id: [c_char; EECH_LOS_HOST_ID_MAX + 1],
    pub target_host_id: [c_char; EECH_LOS_HOST_ID_MAX + 1],
    pub source_position: [f32; 3],
    pub target_position: [f32; 3],
    pub elevation: f32,
    pub hit: c_int,
    pub collision_point: [f32; 3],
}

#[repr(C)]
pub struct eech_los_result {
    pub state: c_int,
    pub phase: c_int,
    pub point: [f32; 3],
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub visible: c_int,
}

pub type eech_line_of_sight_callback =
    unsafe extern "C" fn(step: *const eech_los_step, result: *mut eech_los_result, error: *mut c_char, error_size: usize, user: *mut c_void) -> c_int;

pub const EECH_SENSE_HOST_ID_MAX: usize = 128;
pub const EECH_SENSE_BEGIN: c_int = 1;
pub const EECH_SENSE_CANDIDATE: c_int = 2;
pub const EECH_SENSE_DRAW: c_int = 3;
pub const EECH_SENSE_SIGNATURE: c_int = 4;
pub const EECH_SENSE_END: c_int = 5;
pub const EECH_SENSE_AIR_TO_AIR: c_int = 1;
pub const EECH_SENSE_AIR_TO_SURFACE: c_int = 2;
pub const EECH_SENSE_SURFACE_TO_AIR: c_int = 3;
pub const EECH_SENSE_SURFACE_TO_SURFACE: c_int = 4;
pub const EECH_SENSE_OPEN: c_int = 1;
pub const EECH_SENSE_ACCEPT: c_int = 2;
pub const EECH_SENSE_REJECT: c_int = 3;
pub const EECH_SENSE_NEED_DRAW: c_int = 4;
pub const EECH_SENSE_NEED_SIGNATURE: c_int = 5;
pub const EECH_SENSE_CLOSED: c_int = 6;
pub const EECH_SENSE_REASON_RANGE: c_int = 1;
pub const EECH_SENSE_REASON_RANGE_ADJUSTED: c_int = 2;
pub const EECH_SENSE_REASON_ALTITUDE: c_int = 3;
pub const EECH_SENSE_REASON_TOLERANCE: c_int = 4;
pub const EECH_SENSE_REASON_NOT_PERCEIVED: c_int = 5;

/// One message of the host-owned sensing dialogue (aphavoc/.../mb_hostsense.h), field for field.
#[repr(C)]
pub struct eech_sense_step {
    pub mode: c_int,
    pub scan: c_int,
    pub source_host_id: [c_char; EECH_SENSE_HOST_ID_MAX + 1],
    pub source_position: [f32; 3],
    pub scan_range: f32,
    pub scan_ceiling: f32,
    pub scan_floor: f32,
    pub target_host_id: [c_char; EECH_SENSE_HOST_ID_MAX + 1],
    pub target_position: [f32; 3],
    pub target_player: c_int,
    pub has_difficulty: c_int,
    pub difficulty_factor: f32,
    pub radar_altitude: f32,
    pub draw: f64,
    pub has_emitters: c_int,
    pub radar_on: c_int,
    pub radar_jammer_on: c_int,
    pub infra_red_jammer_on: c_int,
    pub target_helicopter: c_int,
    pub weapon_bay_closed: c_int,
    pub stub_wings_attached: c_int,
    pub undercarriage_up: c_int,
}

#[repr(C)]
pub struct eech_sense_result {
    pub state: c_int,
    pub reason: c_int,
    pub has_range: c_int,
    pub target_range: f32,
    pub has_echo: c_int,
    pub echo_ceiling: f32,
    pub echo_floor: f32,
    pub echo_adjusted_range: f32,
    pub echo_stealth_factor: f32,
    pub echo_stealth_limit: f32,
    pub echo_tolerance_limit: f64,
}

pub type eech_sense_callback =
    unsafe extern "C" fn(step: *const eech_sense_step, result: *mut eech_sense_result, error: *mut c_char, error_size: usize, user: *mut c_void) -> c_int;

pub const EECH_AIM_HOST_ID_MAX: usize = 128;
pub const EECH_AIM_ECHO_MAX: usize = 48;
pub const EECH_AIM_TABLE: c_int = 1;
pub const EECH_AIM_READY_UPDATE: c_int = 2;
pub const EECH_AIM_READY_COMMAND: c_int = 3;
pub const EECH_AIM_REQUIRED: c_int = 4;
pub const EECH_AIM_STEP: c_int = 5;
pub const EECH_AIM_VECTORS: c_int = 6;
pub const EECH_AIM_LEAD: c_int = 7;
pub const EECH_AIM_BALLISTIC: c_int = 8;
pub const EECH_AIM_INTERCEPT: c_int = 9;
pub const EECH_AIM_DONE: c_int = 1;
pub const EECH_AIM_NEED_DRAW: c_int = 2;
pub const EECH_AIM_EVENT_INTERCEPT: c_int = 1;
pub const EECH_AIM_EVENT_LEAD_PATH: c_int = 2;
pub const EECH_AIM_EVENT_LEAD_RANGE: c_int = 3;
pub const EECH_AIM_EVENT_LEAD_ITERATION: c_int = 4;
pub const EECH_AIM_EVENT_BALLISTIC: c_int = 5;
pub const EECH_AIM_EVENT_LEAD_RESULT: c_int = 6;

/// One intermediate value of the reference host's prediction, in its order (read only by the test fixture build).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct eech_aim_event {
    pub kind: c_int,
    pub a: c_int,
    pub b: c_int,
    pub c: c_int,
    pub d: c_int,
    pub f: [f32; 12],
}

/// One request of the host-owned aiming and readiness exchange (aphavoc/.../mb_hostaim.h), field for field.
#[repr(C)]
pub struct eech_aim_step {
    pub mode: c_int,
    pub echo: c_int,
    pub echo_out: *mut eech_aim_event,
    pub echo_capacity: c_int,
    pub source_host_id: [c_char; EECH_AIM_HOST_ID_MAX + 1],
    pub target_host_id: [c_char; EECH_AIM_HOST_ID_MAX + 1],
    pub weapon: c_int,
    pub table_max_range: f32,
    pub table_velocity_tests: c_int,
    pub table_pitch_indices: c_int,
    pub table_range_values: c_int,
    pub table_cells: *const f32,
    pub table_cell_count: usize,
    pub ready_state: f32,
    pub ready_rate: f32,
    pub delta_time: f32,
    pub ready_open: c_int,
    pub device_attitude: [f32; 9],
    pub intercept_vector: [f32; 3],
    pub heading_min: f32,
    pub heading_max: f32,
    pub pitch_min: f32,
    pub pitch_max: f32,
    pub heading_rate: f32,
    pub pitch_rate: f32,
    pub climb: c_int,
    pub current_heading: f32,
    pub current_pitch: f32,
    pub required_heading: f32,
    pub required_pitch: f32,
    pub unbounded: c_int,
    pub track_pitch: f32,
    pub aiming_type: c_int,
    pub cruise_velocity: f32,
    pub acquire_forward: c_int,
    pub source_velocity: f32,
    pub parent_velocity: f32,
    pub device_position: [f32; 3],
    pub target_position: [f32; 3],
    pub target_point: [f32; 3],
    pub target_motion_vector: [f32; 3],
    pub target_mobile: c_int,
    pub has_tracking: c_int,
    pub tracking_point: [f32; 3],
    pub map_min: [f32; 3],
    pub map_max: [f32; 3],
    pub comms_weapon_lag: f32,
    pub source_gunship: c_int,
    pub has_weapon: c_int,
    pub has_draw: c_int,
    pub draw_sample: c_int,
    pub range: f32,
    pub height_diff_or_pitch: f32,
    pub simplified: c_int,
    pub fixed_pitch: c_int,
}

#[repr(C)]
pub struct eech_aim_result {
    pub state: c_int,
    pub result: c_int,
    pub ready_state: f32,
    pub phase: c_int,
    pub required_heading: f32,
    pub required_pitch: f32,
    pub delta_heading: f32,
    pub delta_pitch: f32,
    pub new_heading: f32,
    pub new_pitch: f32,
    pub weapon_vector: [f32; 3],
    pub target_vector: [f32; 3],
    pub intercept_vector: [f32; 3],
    pub intercept_range: f32,
    pub intercept_point: [f32; 3],
    pub angle: f32,
    pub time_of_flight: f32,
    pub has_time_of_flight: c_int,
    pub has_echo: c_int,
    pub echo_count: c_int,
    pub echo: *const eech_aim_event,
}

pub type eech_aim_callback =
    unsafe extern "C" fn(step: *const eech_aim_step, result: *mut eech_aim_result, error: *mut c_char, error_size: usize, user: *mut c_void) -> c_int;

#[repr(C)]
pub struct eech_engine_host {
    pub bind_mobile: Option<eech_bind_mobile_callback>,
    pub observe_loss: Option<eech_observe_loss_callback>,
    pub supply_reached: Option<eech_supply_reached_callback>,
    pub move_vehicle: Option<eech_move_vehicle_callback>,
    pub line_of_sight: Option<eech_line_of_sight_callback>,
    pub sense: Option<eech_sense_callback>,
    pub aim: Option<eech_aim_callback>,
    pub user: *mut c_void,
}

#[repr(C)]
pub struct eech_object {
    pub id: c_int,
    pub kind: c_int,
    pub sub_type: c_int,
    pub side: c_int,
    pub alive: c_int,
    pub group_id: c_int,
    pub type_name: *const c_char,
    pub name: *const c_char,
    pub task: *const c_char,
    pub state: *const c_char,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub heading: f32,
    pub pitch: f32,
    pub roll: f32,
    pub efficiency: f32,
    pub ammo: f32,
    pub fuel: f32,
    pub usable: c_int,
}

#[repr(C)]
pub struct eech_member {
    pub member_key: u64,
    pub group_key: u64,
    pub host_id: *const c_char,
    pub origin_entity_id: c_int,
    pub current_entity_id: c_int,
    pub entity_type: c_int,
    pub sub_type: c_int,
    pub side: c_int,
    pub initial_alive: c_int,
    pub physical_present: c_int,
    pub in_group: c_int,
}

#[repr(C)]
pub struct eech_supply_task {
    pub task_key: u64,
    pub waypoint_key: u64,
    pub leader_host_id: [c_char; EECH_HOST_ID_MAX + 1],
    pub task_entity_index: c_int,
    pub waypoint_entity_index: c_int,
    pub waypoint_kind: c_int,
    pub task_completed: c_int,
    pub supplier_entity_index: c_int,
    pub receiver_entity_index: c_int,
    pub cargo_type: c_int,
    pub cargo_on_leader: c_int,
    pub supplier_ammo: f32,
    pub supplier_fuel: f32,
    pub receiver_ammo: f32,
    pub receiver_fuel: f32,
}

#[repr(C)]
#[derive(Default)]
pub struct eech_roster_stats {
    pub duplicate_comparisons: u64,
    pub duplicate_scan_seconds: f64,
}

#[repr(C)]
#[derive(Default)]
pub struct eech_loss_report_receipt {
    pub landing_reserved_before: c_int,
    pub landing_reserved_after: c_int,
    pub group_losses_before: c_int,
    pub group_losses_after: c_int,
    pub primary_task_losses_before: c_int,
    pub primary_task_losses_after: c_int,
    pub group_logical_members_before: c_int,
    pub group_logical_members_after: c_int,
    pub group_physical_members_before: c_int,
    pub tasks_retired_expire: c_int,
    pub tasks_completed_on_report: c_int,
}

#[repr(C)]
#[derive(Default)]
pub struct eech_clock {
    pub elapsed_seconds: f32,
    pub time_of_day_seconds: f32,
    pub day: c_int,
}

pub type eech_object_callback = unsafe extern "C" fn(object: *const eech_object, user: *mut c_void);
pub type eech_member_callback = unsafe extern "C" fn(member: *const eech_member, user: *mut c_void);
pub type eech_supply_task_callback = unsafe extern "C" fn(task: *const eech_supply_task, user: *mut c_void);
pub type eech_log_sink = unsafe extern "C" fn(level: c_int, message: *const c_char, user: *mut c_void);

extern "C" {
    pub fn eech_engine_boot(config: *const eech_engine_config, host: *const eech_engine_host) -> c_int;
    pub fn eech_engine_frame(milliseconds: u32, host: *const eech_engine_host) -> c_int;
    pub fn eech_engine_report_loss(host_id: *const c_char, receipt: *mut eech_loss_report_receipt) -> c_int;
    pub fn eech_engine_fatal_message() -> *const c_char;
    pub fn eech_engine_prepare_installation(root: *const c_char) -> c_int;
    pub fn eech_engine_objects(callback: eech_object_callback, user: *mut c_void, count: *mut c_int) -> c_int;
    pub fn eech_engine_members(callback: eech_member_callback, user: *mut c_void, count: *mut c_int) -> c_int;
    pub fn eech_engine_roster_stats(stats: *mut eech_roster_stats) -> c_int;
    pub fn eech_engine_supply_tasks(callback: eech_supply_task_callback, user: *mut c_void, count: *mut c_int) -> c_int;
    pub fn eech_engine_clock(clock: *mut eech_clock) -> c_int;
    pub fn eech_engine_diagnostics() -> c_int;
    pub fn eech_set_log_sink(sink: Option<eech_log_sink>, user: *mut c_void);
}

// TEST-ONLY (feature dependent-fixture, EEDCS-40): the dependent-positioning fixture driver.
#[cfg(feature = "dependent-fixture")]
pub const EECH_FIXTURE_LINK: c_int = 1;
#[cfg(feature = "dependent-fixture")]
pub const EECH_FIXTURE_MOVE: c_int = 2;
#[cfg(feature = "dependent-fixture")]
pub const EECH_FIXTURE_SINK: c_int = 3;
#[cfg(feature = "dependent-fixture")]
extern "C" {
    pub fn eech_engine_dependent_fixture(op: c_int, parent_id: c_int, child_id: c_int, position: *const f64, host: *const eech_engine_host) -> c_int;
}

#[cfg(feature = "loss-fixture")]
extern "C" {
    pub fn eech_engine_loss_fixture(entity_id: c_int, host: *const eech_engine_host) -> c_int;
    pub fn eech_engine_loss_release(entity_id: c_int, released: *mut c_int, host: *const eech_engine_host) -> c_int;
    pub fn eech_engine_loss_state(side: c_int, values: *mut c_int, capacity: c_int, count: *mut c_int) -> c_int;
    pub fn eech_engine_loss_shape(entity_id: c_int, reason: *mut c_char, capacity: c_int) -> c_int;
    pub fn eech_engine_loss_group(entity_id: c_int, text: *mut c_char, capacity: c_int) -> c_int;
}

/// TEST-ONLY (feature los-fixture, EEDCS-42): a fold of line-of-sight queries; see csrc/eech_los_fixture.c.
#[cfg(feature = "los-fixture")]
pub const EECH_LOS_CRITERIA: usize = 32;

#[cfg(feature = "los-fixture")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct eech_los_trace {
    pub hash: u64,
    pub calls: u64,
    pub visible: u64,
    pub skipped: u64,
    pub lookup_hash: u64,
    pub lookups: u64,
    pub by_criteria_calls: [u64; EECH_LOS_CRITERIA],
    pub by_criteria_visible: [u64; EECH_LOS_CRITERIA],
}

#[cfg(feature = "los-fixture")]
extern "C" {
    pub fn eech_engine_los_trace(out: *mut eech_los_trace) -> c_int;
    pub fn eech_engine_los_trace_reset() -> c_int;
    pub fn eech_engine_los_probe(source_id: c_int, target_id: c_int, seed: u32, count: c_int, out: *mut eech_los_trace, host: *const eech_engine_host)
        -> c_int;
}

#[cfg(feature = "sensor-fixture")]
extern "C" {
    pub fn eech_engine_sensor_trace(text: *mut c_char, capacity: c_int) -> c_int;
    pub fn eech_engine_sensor_trace_reset() -> c_int;
    pub fn eech_engine_sensor_probe(text: *mut c_char, capacity: c_int, host: *const eech_engine_host) -> c_int;
}

#[cfg(feature = "weapon-fixture")]
extern "C" {
    pub fn eech_engine_wpn_trace(text: *mut c_char, capacity: c_int) -> c_int;
    pub fn eech_engine_wpn_events(text: *mut c_char, capacity: c_int, more: *mut c_int) -> c_int;
    pub fn eech_engine_wpn_case_query(host_id: *const c_char, weapon: c_int, text: *mut c_char, capacity: c_int) -> c_int;
    pub fn eech_engine_wpn_case_kind(host_id: *const c_char, text: *mut c_char, capacity: c_int) -> c_int;
    pub fn eech_engine_wpn_case_launch(
        launcher_id: *const c_char,
        target_id: *const c_char,
        weapon: c_int,
        salvo_count: c_int,
        text: *mut c_char,
        capacity: c_int,
        host: *const eech_engine_host,
    ) -> c_int;
}

#[cfg(feature = "aim-fixture")]
extern "C" {
    pub fn eech_engine_aim_trace(text: *mut c_char, capacity: c_int) -> c_int;
    pub fn eech_engine_aim_trace_reset() -> c_int;
    pub fn eech_engine_aim_person_query(host_id: *const c_char, text: *mut c_char, capacity: c_int) -> c_int;
    pub fn eech_engine_aim_person_case(
        person_id: *const c_char,
        target_id: *const c_char,
        text: *mut c_char,
        capacity: c_int,
        host: *const eech_engine_host,
    ) -> c_int;
    pub fn eech_engine_aim_lead_probe(
        source_id: *const c_char,
        target_id: *const c_char,
        weapon: c_int,
        sentinel: f32,
        text: *mut c_char,
        capacity: c_int,
        host: *const eech_engine_host,
    ) -> c_int;
}

// The tumble draws are the only doubles in the step. They stay its last field, so the double's alignment (4 or 8 by
// target) cannot move any float or int before it; the C struct in rv_hostmv.h declares them last as well.
const _: () = assert!(core::mem::offset_of!(eech_movement_step, hc_draws) > core::mem::offset_of!(eech_movement_step, gv_water));
