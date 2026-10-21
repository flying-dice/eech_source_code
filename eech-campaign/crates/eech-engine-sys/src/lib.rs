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
pub const CAMPAIGN_SUPPLY_MOBILE_FIXED_WING: c_int = 1;
pub const CAMPAIGN_SUPPLY_MOBILE_HELICOPTER: c_int = 2;

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
    pub waypoint_kind: c_int,
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
}

pub type eech_supply_reached_callback = unsafe extern "C" fn(
    descriptor: *const eech_supply_reach_descriptor,
    reached: *mut c_int,
    error: *mut c_char,
    error_size: usize,
    user: *mut c_void,
) -> c_int;

#[repr(C)]
pub struct eech_engine_host {
    pub bind_mobile: Option<eech_bind_mobile_callback>,
    pub observe_loss: Option<eech_observe_loss_callback>,
    pub supply_reached: Option<eech_supply_reached_callback>,
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
