//! The headless EECH engine: the whole of Enemy Engaged: Comanche vs Hokum
//! (its dynamic campaign, AI, pathfinding, weapons, damage, comms and entity
//! system) compiled headless, behind a safe API.
//!
//! EECH is global state: there is one engine per process, booted once
//! ([`Engine::boot`]). A fatal error inside EECH (`debug_fatal`) returns
//! [`EngineError::Fatal`] and poisons the engine; every later call fails.

use eech_engine_sys as sys;
use serde::Serialize;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

static BOOTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub enum EngineError {
    /// EECH called debug_fatal; the engine is poisoned
    Fatal(String),
    /// an earlier fatal error poisoned the engine
    Poisoned,
    /// the call is not valid in the engine's state (booted twice, not booted)
    State,
    BadArgument(String),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::Fatal(m) => write!(f, "EECH fatal error: {m}"),
            EngineError::Poisoned => write!(f, "the engine is poisoned by an earlier fatal error"),
            EngineError::State => write!(f, "not valid in the engine's state"),
            EngineError::BadArgument(m) => write!(f, "bad argument: {m}"),
        }
    }
}

impl std::error::Error for EngineError {}

fn check(code: c_int) -> Result<(), EngineError> {
    match code {
        sys::EECH_ENGINE_OK => Ok(()),
        sys::EECH_ENGINE_FATAL => {
            // SAFETY: a static NUL-terminated buffer in the engine
            let message = unsafe { CStr::from_ptr(sys::eech_engine_fatal_message()) }.to_string_lossy().into_owned();
            Err(EngineError::Fatal(message))
        }
        sys::EECH_ENGINE_POISONED => Err(EngineError::Poisoned),
        sys::EECH_ENGINE_STATE => Err(EngineError::State),
        _ => Err(EngineError::BadArgument("rejected by the engine".into())),
    }
}

fn cstring(s: &str) -> Result<CString, EngineError> {
    CString::new(s).map_err(|_| EngineError::BadArgument(format!("{s:?} contains NUL")))
}

/// EECH's gunship types (GUNSHIP_TYPE_*): the player's (the server's) side
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gunship {
    Apache = 0,
    Havoc = 1,
    Comanche = 2,
    Hokum = 3,
}

#[derive(Clone, Debug)]
pub struct EngineConfig {
    /// an installation root: contains cohokum/ (the working directory) and common/
    pub install_root: PathBuf,
    /// EECH paths relative to cohokum/, e.g. `..\common\maps\map16`, `camp01`, `georgia.chc`
    pub map_path: String,
    pub campaign_directory: String,
    pub campaign_filename: String,
    pub gunship: Gunship,
    pub random_seed: u32,
    /// extra EECH command-line arguments (EECH.INI syntax, e.g. `dmrate=10`)
    pub arguments: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Helicopter,
    FixedWing,
    GroundVehicle,
    AirDefence,
    Ship,
    Infantry,
    Weapon,
    Keysite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Neutral,
    Blue,
    Red,
}

/// one observed object (see csrc/eech_observe.c)
#[derive(Clone, Debug, Serialize)]
pub struct Object {
    /// EECH entity index; reused after an entity is destroyed
    pub id: i32,
    pub kind: Kind,
    pub sub_type: i32,
    pub side: Side,
    pub alive: bool,
    /// the group's entity index, or for a weapon its launcher's
    pub group_id: Option<i32>,
    pub type_name: String,
    pub name: Option<String>,
    pub task: Option<String>,
    /// the group's verbose operational state ("En route", "Engaging", ...)
    pub state: Option<String>,
    /// EECH world metres: x east, y up, z north
    pub position: [f32; 3],
    /// radians
    pub heading: f32,
    pub pitch: f32,
    pub roll: f32,
    pub efficiency: f32,
    /// keysites: ammo and fuel supply levels (percent)
    pub ammo: f32,
    pub fuel: f32,
    /// keysites: EECH's usable state ("usable", "repairing", "out_of_action", ...)
    pub usable: Option<String>,
}

/// Copied facts supplied to the host when a physical member is first bound.
/// `entity_id` is a transient correlation value and may be reused by EECH; the
/// returned host identity is the durable identity.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct MemberDescriptor {
    pub member_key: u64,
    pub group_key: u64,
    pub entity_id: i32,
    pub entity_type: i32,
    pub sub_type: i32,
    pub side: Side,
    /// movable members only: the pose that seeds the host's record of the member
    pub pose: Option<VehiclePose>,
    /// what physical step the host performs for the member; present exactly when `pose` is
    pub movement_kind: Option<MovementKind>,
}

/// The physical step the host performs for a movable member.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[repr(i32)]
pub enum MovementKind {
    /// a ground convoy member: terrain-following heading and pitch
    Routed = 1,
    /// a ship: heading, speed and position along the guide
    Ship = 2,
    /// infantry: walk or run along the waypoint, in two phases around the campaign's terrain sample
    Person = 3,
}

/// A movable member's physical state: EECH world metres, speed in m/s, a row-major 3x3 attitude.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct VehiclePose {
    pub position: [f32; 3],
    pub velocity: f32,
    pub attitude: [f32; 9],
}

/// What the campaign asks of the host's record of one routed vehicle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MovementMode {
    /// step towards the waypoint; the campaign's pose is sent for comparison only
    Move,
    /// the campaign zeroes the speed (hold, no waypoint, no guide)
    Stop,
    /// the campaign placed the vehicle: the host adopts the pose
    Place,
    /// a ship's step towards the waypoint (single phase)
    ShipMove,
    /// first phase of an infantry step: the horizontal advance; the host holds the result until `PersonEnd`
    PersonBegin,
    /// second phase of an infantry step: the host places the member at the campaign's terrain elevation
    PersonEnd,
}

/// Terrain at the pre-move position, sampled by the campaign.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct TerrainSample {
    pub elevation: f32,
    pub forest: bool,
    pub face_normal: [f32; 3],
}

/// One movement request for the vehicle whose host identity is `host_id`.
/// `pose` is the campaign's copy: compared with the host's record on `Move` and `Stop`, adopted on `Place`.
/// The remaining fields depend on the mode: `Move` uses the routed set, `ShipMove` the ship set
/// (`frame_delta`, `desired_velocity` as the guide velocity, `g_max`), `PersonBegin` `ps_vel` and
/// `terrain_follow`, and `PersonEnd` `elevation`.
#[derive(Clone, Debug, Serialize)]
pub struct MovementStep {
    pub host_id: String,
    pub mode: MovementMode,
    pub pose: VehiclePose,
    pub delta_seconds: f32,
    pub desired_velocity: f32,
    pub max_acceleration: f32,
    pub waypoint: [f32; 3],
    /// the host reports `rotated` only when this is set
    pub tacview_logging: bool,
    /// present when the guide follows terrain
    pub terrain: Option<TerrainSample>,
    pub map_min: [f32; 3],
    pub map_max: [f32; 3],
    /// the frame delta, distinct from `delta_seconds` (the engine's movement delta)
    pub frame_delta: f32,
    pub g_max: f32,
    pub ps_vel: f32,
    /// `PersonBegin`: the campaign will sample terrain at the returned position and send `PersonEnd`
    pub terrain_follow: bool,
    /// `PersonEnd`: the terrain elevation the campaign sampled at the position `PersonBegin` returned
    pub elevation: f32,
}

/// The host's record of the vehicle after the step; the campaign applies it.
/// While `pending`, only `pose.position` is meaningful (the horizontal advance) and the campaign
/// applies nothing until the `PersonEnd` result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovementResult {
    pub pose: VehiclePose,
    pub rotated: bool,
    /// the motion vector the step leaves on the member (ships and infantry)
    pub motion_vector: [f32; 3],
    /// an infantry step awaits its second phase
    pub pending: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LossEventKind {
    Credit,
    Physical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LossDecision {
    Forward,
    Withhold,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct LossTaskDependent {
    pub task_key: u64,
    pub entity_type: i32,
    pub sub_type: i32,
    pub task_state: Option<i32>,
    pub task_completed: Option<bool>,
    pub primary_task: bool,
    pub persistent_task: bool,
    pub objective_is_member: bool,
    pub task_dependent_children: i32,
    pub guide_children: i32,
    pub expire_timer: f32,
    pub stop_timer: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SupplyWaypoint {
    PickUp,
    PrepareForDropOff,
    DropOff,
    FinishDropOff,
    /// Not a SUPPLY waypoint: any other radius-only fixed-wing or helicopter waypoint.
    RadiusArrival,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LossLandingActionKind {
    UnreserveSite,
    UnlockRoute,
    UnlockTakeoffRoute,
    UnlockSite,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct LossLandingAction {
    pub landing_key: u64,
    pub action: LossLandingActionKind,
    pub count: i32,
    pub landing_sub_type: i32,
    pub reserved_landing_sites: i32,
    pub free_landing_sites: i32,
    pub landing_lock: bool,
    pub landed_lock: bool,
    pub takeoff_lock: bool,
}

/// Copied campaign and physical facts supplied synchronously at an existing
/// credit or physical-loss site. The opaque `host_id` identifies the physical
/// incarnation; entity indices are deliberately absent. `Withhold` defers only
/// a supported campaign consequence set; reference-world physical death always
/// continues.
#[derive(Clone, Debug, Serialize)]
pub struct LossDescriptor {
    pub host_id: String,
    pub member_key: u64,
    pub group_key: u64,
    pub event: LossEventKind,
    pub entity_type: i32,
    pub sub_type: i32,
    pub side: Side,
    pub force_info_category: i32,
    pub group_sub_type: i32,
    pub primary_task_sub_type: Option<i32>,
    pub primary_task_key: Option<u64>,
    pub group_member_number: i32,
    pub campaign_member_count: i32,
    pub player: i32,
    pub landed: bool,
    pub member_task_dependent_count: i32,
    pub member_task_dependents: Vec<LossTaskDependent>,
    pub group_task_dependents: i32,
    pub restricted_guides: i32,
    pub landing_action_count: i32,
    pub landing_actions: Vec<LossLandingAction>,
    pub movement_parent: bool,
    pub movement_dependents: i32,
    pub takeoff_queue: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SupplyMobile {
    FixedWing,
    Helicopter,
    RoutedVehicle,
    ShipVehicle,
    AntiAircraft,
    Person,
}

/// Which physical question the host is asked. The names `SupplyReachDescriptor`
/// and `supply_reached` are retained historical naming for the whole seam.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArrivalQuery {
    /// A member's reached-waypoint criteria.
    Reached,
    /// The task leader is inside the waypoint action radius.
    Action,
    /// A helicopter is strictly inside the recon radius.
    Recon,
}

/// How the guide derives a helicopter's required heading.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadingSource {
    /// The waypoint's own heading.
    Waypoint,
    /// Prepare-for-insertion and troop-insert waypoints hold a zero heading.
    Zero,
    /// Point from the member to the task-dependent target.
    Target,
}

/// Copied physical facts for one campaign-selected member at a waypoint.
/// Task and waypoint keys are session-local correlation keys; only `host_id`
/// is the host-issued mobile identity. Only the facts the queried mobile
/// kind's rule reads are populated; the rest are zero or false.
#[derive(Clone, Debug, Serialize)]
pub struct SupplyReachDescriptor {
    pub member_key: u64,
    pub task_key: u64,
    pub waypoint_key: u64,
    pub host_id: String,
    pub query: ArrivalQuery,
    pub waypoint: SupplyWaypoint,
    /// The campaign waypoint sub-type, so a host can tell non-SUPPLY waypoints apart.
    pub waypoint_sub_type: i32,
    /// Waypoint database fact: a human member's arrival is waived.
    pub player_skip_waypoint: bool,
    /// Waypoint database fact: a landing or troop-insert waypoint, where an unassisted human must be on the ground.
    pub landing_waypoint: bool,
    pub mobile: SupplyMobile,
    pub is_task_leader: bool,
    pub is_ai: bool,
    pub leader_is_ai: bool,
    pub landed: bool,
    pub updated: bool,
    pub rotor_spin_up_seconds: f32,
    pub distance: f32,
    pub velocity: f32,
    pub delta_seconds: f32,
    pub radius: f32,
    pub position: [f32; 3],
    pub motion: [f32; 3],
    pub guide_position: [f32; 3],
    pub leader_position: [f32; 3],
    pub radius_valid: bool,
    pub altitude_valid: bool,
    pub heading_valid: bool,
    pub last_to_reach_valid: bool,
    /// The weapon-vector rule applies to this member: the host reports an angle.
    pub weapon_gate: bool,
    /// Both weapon and target vectors are present and valid.
    pub vectors_valid: bool,
    pub airborne: bool,
    pub auto_pilot: bool,
    pub altitude_criterion: f32,
    pub heading_criterion: f32,
    pub altitude: f32,
    pub heading: f32,
    pub centre_of_gravity_to_ground: f32,
    pub guide_altitude: f32,
    /// Helicopter HEADING gate: how the guide derives the required heading. Absent when the gate
    /// does not apply to this member; the host derives the heading from the facts below.
    pub heading_source: Option<HeadingSource>,
    /// The waypoint's own heading; also the fallback when a TARGET waypoint has no target position.
    pub waypoint_heading: f32,
    /// TARGET waypoint: the guide's task has a task-dependent target.
    pub heading_target_present: bool,
    /// The task-dependent target's position, when it has one.
    pub heading_target_position: Option<[f32; 3]>,
    pub weapon_vector: [f32; 3],
    pub intercept_vector: [f32; 3],
    pub action_radius: f32,
    pub waypoint_position: [f32; 3],
}

/// The host's answer. `gates_passed` is every physical gate that precedes the
/// campaign's weapon-vector tolerance. `launch_angle_error` is the angle
/// between the weapon and intercept vectors, required when the descriptor has
/// `weapon_gate` and `vectors_valid` and the gates passed; the campaign keeps
/// the random tolerance and the weapon limit. `heading_target_missing` reports
/// that the host reached the HEADING gate of a TARGET waypoint whose task or
/// target is absent; the campaign then raises the legacy assertion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArrivalAnswer {
    pub gates_passed: bool,
    pub launch_angle_error: Option<f32>,
    pub heading_target_missing: bool,
}

/// One durable campaign roster entry. Membership can change while the same
/// physical incarnation and host identity continue.
#[derive(Clone, Debug, Serialize)]
pub struct Member {
    pub member_key: u64,
    pub group_key: u64,
    pub host_id: String,
    pub origin_entity_id: i32,
    pub current_entity_id: Option<i32>,
    pub entity_type: i32,
    pub sub_type: i32,
    pub side: Side,
    pub initial_alive: bool,
    pub physical_present: bool,
    pub in_group: bool,
}

/// Actual current campaign state for an extant SUPPLY task.
#[derive(Clone, Debug, Serialize)]
pub struct SupplyTask {
    pub task_key: u64,
    pub waypoint_key: Option<u64>,
    pub leader_host_id: Option<String>,
    pub task_entity_index: i32,
    pub waypoint_entity_index: Option<i32>,
    pub waypoint: Option<SupplyWaypoint>,
    pub task_completed: i32,
    pub supplier_entity_index: Option<i32>,
    pub receiver_entity_index: Option<i32>,
    pub cargo_type: i32,
    pub cargo_on_leader: i32,
    pub supplier_ammo: f32,
    pub supplier_fuel: f32,
    pub receiver_ammo: f32,
    pub receiver_fuel: f32,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct RosterStats {
    pub duplicate_comparisons: u64,
    pub duplicate_scan_seconds: f64,
}

/// What one `report_loss` applied, copied from the performed campaign actions; an observation, never an input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct LossReceipt {
    pub landing_reserved_before: i32,
    pub landing_reserved_after: i32,
    pub group_losses_before: i32,
    pub group_losses_after: i32,
    pub primary_task_losses_before: i32,
    pub primary_task_losses_after: i32,
    pub group_logical_members_before: i32,
    pub group_logical_members_after: i32,
    pub group_physical_members_before: i32,
    pub tasks_retired_expire: i32,
    pub tasks_completed_on_report: i32,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct Clock {
    pub elapsed_seconds: f32,
    pub time_of_day_seconds: f32,
    pub day: i32,
}

/// the booted engine (one per process)
pub struct Engine {
    _not_send: std::marker::PhantomData<*const ()>,
}

struct HostContext<'a, F, G, H, M> {
    bind_mobile: &'a mut F,
    observe_loss: &'a mut G,
    supply_reached: &'a mut H,
    move_vehicle: &'a mut M,
}

fn side(value: c_int) -> Side {
    match value {
        1 => Side::Blue,
        2 => Side::Red,
        _ => Side::Neutral,
    }
}

unsafe fn write_callback_text(target: *mut c_char, capacity: usize, value: &str) {
    if target.is_null() || capacity == 0 {
        return;
    }
    let bytes = value.as_bytes();
    let length = bytes.len().min(capacity - 1);
    // SAFETY: the caller supplies a writable callback buffer of `capacity` bytes.
    let out = unsafe { std::slice::from_raw_parts_mut(target.cast::<u8>(), capacity) };
    for (destination, source) in out.iter_mut().take(length).zip(bytes.iter()) {
        *destination = if *source == 0 { b'?' } else { *source };
    }
    out[length] = 0;
}

unsafe extern "C" fn bind_mobile<F, G, H, M>(
    descriptor: *const sys::eech_member_descriptor,
    host_id: *mut c_char,
    host_id_size: usize,
    error: *mut c_char,
    error_size: usize,
    user: *mut c_void,
) -> c_int
where
    F: FnMut(MemberDescriptor) -> Result<String, String>,
    G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
    H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
    M: FnMut(MovementStep) -> Result<MovementResult, String>,
{
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        if descriptor.is_null() || user.is_null() {
            return Err("invalid native bind_mobile callback context".into());
        }
        // SAFETY: C supplies both pointers for the duration of this callback.
        let (descriptor, context) = unsafe { (&*descriptor, &mut *(user as *mut HostContext<'_, F, G, H, M>)) };
        let descriptor = MemberDescriptor {
            member_key: descriptor.member_key,
            group_key: descriptor.group_key,
            entity_id: descriptor.entity_index,
            entity_type: descriptor.entity_type,
            sub_type: descriptor.sub_type,
            side: side(descriptor.side),
            pose: checked_bool("has_pose", descriptor.has_pose)?.then_some(VehiclePose {
                position: descriptor.position,
                velocity: descriptor.velocity,
                attitude: descriptor.attitude,
            }),
            movement_kind: if checked_bool("has_pose", descriptor.has_pose)? {
                Some(match descriptor.movement_kind {
                    sys::EECH_MOVEMENT_KIND_ROUTED => MovementKind::Routed,
                    sys::EECH_MOVEMENT_KIND_SHIP => MovementKind::Ship,
                    sys::EECH_MOVEMENT_KIND_PERSON => MovementKind::Person,
                    other => return Err(format!("bind_mobile received unknown movement kind {other}")),
                })
            } else {
                None
            },
        };
        let identity = (context.bind_mobile)(descriptor)?;
        let bytes = identity.as_bytes();
        if bytes.is_empty() {
            return Err("bind_mobile returned an empty identity".into());
        }
        if bytes.len() > sys::EECH_HOST_ID_MAX {
            return Err(format!("bind_mobile identity is {} bytes; maximum is {}", bytes.len(), sys::EECH_HOST_ID_MAX));
        }
        if bytes.contains(&0) {
            return Err("bind_mobile identity contains NUL".into());
        }
        if host_id.is_null() || host_id_size <= bytes.len() {
            return Err("native host identity buffer is too small".into());
        }
        // SAFETY: capacity was checked above and C owns the writable buffer.
        unsafe { write_callback_text(host_id, host_id_size, &identity) };
        Ok(())
    }));
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(message)) => {
            // SAFETY: C owns this writable error buffer for the callback.
            unsafe { write_callback_text(error, error_size, &message) };
            1
        }
        Err(_) => {
            // No Rust panic may unwind through C.
            unsafe { write_callback_text(error, error_size, "bind_mobile panicked") };
            1
        }
    }
}

unsafe extern "C" fn observe_loss<F, G, H, M>(
    descriptor: *const sys::eech_loss_descriptor,
    decision: *mut c_int,
    error: *mut c_char,
    error_size: usize,
    user: *mut c_void,
) -> c_int
where
    F: FnMut(MemberDescriptor) -> Result<String, String>,
    G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
    H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
    M: FnMut(MovementStep) -> Result<MovementResult, String>,
{
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        if descriptor.is_null() || decision.is_null() || user.is_null() {
            return Err("invalid native observe_loss callback context".into());
        }
        // SAFETY: C supplies these pointers for the duration of the callback.
        let (descriptor, decision, context) = unsafe { (&*descriptor, &mut *decision, &mut *(user as *mut HostContext<'_, F, G, H, M>)) };
        if descriptor.host_id.is_null() {
            return Err("observe_loss received an empty host identity".into());
        }
        // SAFETY: roster identities are NUL-terminated and live for the engine session.
        let host_id = unsafe { CStr::from_ptr(descriptor.host_id) }
            .to_str()
            .map_err(|_| "observe_loss received a non-UTF-8 host identity")?
            .to_owned();
        let event = match descriptor.event_kind {
            sys::EECH_LOSS_EVENT_CREDIT => LossEventKind::Credit,
            sys::EECH_LOSS_EVENT_PHYSICAL => LossEventKind::Physical,
            value => return Err(format!("observe_loss received unknown event kind {value}")),
        };
        let dependent_count =
            usize::try_from(descriptor.member_task_dependent_copied).map_err(|_| "observe_loss received a negative copied task-dependent count")?;
        if dependent_count > sys::EECH_LOSS_TASK_DEPENDENT_MAX {
            return Err(format!(
                "observe_loss received {} copied task dependents; maximum is {}",
                dependent_count,
                sys::EECH_LOSS_TASK_DEPENDENT_MAX
            ));
        }
        let member_task_dependents = descriptor.member_task_dependents[..dependent_count]
            .iter()
            .map(|dependent| LossTaskDependent {
                task_key: dependent.task_key,
                entity_type: dependent.entity_type,
                sub_type: dependent.sub_type,
                task_state: (dependent.task_state >= 0).then_some(dependent.task_state),
                task_completed: (dependent.task_completed >= 0).then_some(dependent.task_completed != 0),
                primary_task: dependent.primary_task != 0,
                persistent_task: dependent.persistent_task != 0,
                objective_is_member: dependent.objective_is_member != 0,
                task_dependent_children: dependent.task_dependent_children,
                guide_children: dependent.guide_children,
                expire_timer: dependent.expire_timer,
                stop_timer: dependent.stop_timer,
            })
            .collect();
        let landing_count = usize::try_from(descriptor.landing_action_copied).map_err(|_| "observe_loss received a negative copied landing-action count")?;
        if landing_count > sys::EECH_LOSS_LANDING_ACTION_MAX {
            return Err(format!(
                "observe_loss received {} copied landing actions; maximum is {}",
                landing_count,
                sys::EECH_LOSS_LANDING_ACTION_MAX
            ));
        }
        let landing_actions = descriptor.landing_actions[..landing_count]
            .iter()
            .map(|action| {
                let action_kind = match action.action_kind {
                    sys::EECH_LOSS_LANDING_UNRESERVE_SITE => LossLandingActionKind::UnreserveSite,
                    sys::EECH_LOSS_LANDING_UNLOCK_ROUTE => LossLandingActionKind::UnlockRoute,
                    sys::EECH_LOSS_LANDING_UNLOCK_TAKEOFF_ROUTE => LossLandingActionKind::UnlockTakeoffRoute,
                    sys::EECH_LOSS_LANDING_UNLOCK_SITE => LossLandingActionKind::UnlockSite,
                    value => return Err(format!("observe_loss received unknown landing action kind {value}")),
                };
                Ok(LossLandingAction {
                    landing_key: action.landing_key,
                    action: action_kind,
                    count: action.count,
                    landing_sub_type: action.landing_sub_type,
                    reserved_landing_sites: action.reserved_landing_sites,
                    free_landing_sites: action.free_landing_sites,
                    landing_lock: action.landing_lock != 0,
                    landed_lock: action.landed_lock != 0,
                    takeoff_lock: action.takeoff_lock != 0,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let descriptor = LossDescriptor {
            host_id,
            member_key: descriptor.member_key,
            group_key: descriptor.group_key,
            event,
            entity_type: descriptor.entity_type,
            sub_type: descriptor.sub_type,
            side: side(descriptor.side),
            force_info_category: descriptor.force_info_category,
            group_sub_type: descriptor.group_sub_type,
            primary_task_sub_type: (descriptor.primary_task_sub_type >= 0).then_some(descriptor.primary_task_sub_type),
            primary_task_key: (descriptor.primary_task_key != 0).then_some(descriptor.primary_task_key),
            group_member_number: descriptor.group_member_number,
            campaign_member_count: descriptor.campaign_member_count,
            player: descriptor.player,
            landed: descriptor.landed != 0,
            member_task_dependent_count: descriptor.member_task_dependent_count,
            member_task_dependents,
            group_task_dependents: descriptor.group_task_dependents,
            restricted_guides: descriptor.restricted_guides,
            landing_action_count: descriptor.landing_action_count,
            landing_actions,
            movement_parent: descriptor.movement_parent != 0,
            movement_dependents: descriptor.movement_dependents,
            takeoff_queue: descriptor.takeoff_queue != 0,
        };
        *decision = match (context.observe_loss)(descriptor)? {
            LossDecision::Forward => sys::EECH_LOSS_FORWARD,
            LossDecision::Withhold => sys::EECH_LOSS_WITHHOLD,
        };
        Ok(())
    }));
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(message)) => {
            // SAFETY: C owns this writable error buffer for the callback.
            unsafe { write_callback_text(error, error_size, &message) };
            1
        }
        Err(_) => {
            // No Rust panic may unwind through C.
            unsafe { write_callback_text(error, error_size, "observe_loss panicked") };
            1
        }
    }
}

fn supply_waypoint(value: c_int) -> Result<SupplyWaypoint, String> {
    match value {
        sys::CAMPAIGN_SUPPLY_WAYPOINT_PICK_UP => Ok(SupplyWaypoint::PickUp),
        sys::CAMPAIGN_SUPPLY_WAYPOINT_PREPARE_FOR_DROP_OFF => Ok(SupplyWaypoint::PrepareForDropOff),
        sys::CAMPAIGN_SUPPLY_WAYPOINT_DROP_OFF => Ok(SupplyWaypoint::DropOff),
        sys::CAMPAIGN_SUPPLY_WAYPOINT_FINISH_DROP_OFF => Ok(SupplyWaypoint::FinishDropOff),
        sys::CAMPAIGN_RADIUS_ARRIVAL_WAYPOINT => Ok(SupplyWaypoint::RadiusArrival),
        other => Err(format!("unknown SUPPLY waypoint kind {other}")),
    }
}

fn checked_bool(name: &str, value: c_int) -> Result<bool, String> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(format!("invalid {name} flag {value}")),
    }
}

fn checked_floats(values: &[(&str, f32)]) -> Result<(), String> {
    for (name, value) in values {
        if !value.is_finite() {
            return Err(format!("non-finite supply arrival {name}"));
        }
    }
    Ok(())
}

unsafe extern "C" fn supply_reached<F, G, H, M>(
    descriptor: *const sys::eech_supply_reach_descriptor,
    answer_out: *mut sys::eech_arrival_result,
    error: *mut c_char,
    error_size: usize,
    user: *mut c_void,
) -> c_int
where
    F: FnMut(MemberDescriptor) -> Result<String, String>,
    G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
    H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
    M: FnMut(MovementStep) -> Result<MovementResult, String>,
{
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        if descriptor.is_null() || answer_out.is_null() || user.is_null() {
            return Err("invalid native supply_reached callback context".into());
        }
        // SAFETY: C supplies both pointers for the duration of this callback.
        let (source, context) = unsafe { (&*descriptor, &mut *(user as *mut HostContext<'_, F, G, H, M>)) };
        let host_id_bytes = source.host_id.iter().map(|byte| *byte as u8).take_while(|byte| *byte != 0).collect::<Vec<_>>();
        if host_id_bytes.is_empty() || host_id_bytes.len() > sys::EECH_HOST_ID_MAX {
            return Err("supply_reached received an invalid host identity length".into());
        }
        let host_id = std::str::from_utf8(&host_id_bytes)
            .map_err(|_| "supply_reached received a non-UTF-8 host identity")?
            .to_owned();
        if source.member_key == 0 || source.task_key == 0 || source.waypoint_key == 0 {
            return Err("supply_reached received a zero correlation key".into());
        }
        let mobile = match source.mobile_kind {
            sys::CAMPAIGN_SUPPLY_MOBILE_FIXED_WING => SupplyMobile::FixedWing,
            sys::CAMPAIGN_SUPPLY_MOBILE_HELICOPTER => SupplyMobile::Helicopter,
            sys::CAMPAIGN_SUPPLY_MOBILE_ROUTED_VEHICLE => SupplyMobile::RoutedVehicle,
            sys::CAMPAIGN_SUPPLY_MOBILE_SHIP_VEHICLE => SupplyMobile::ShipVehicle,
            sys::CAMPAIGN_SUPPLY_MOBILE_ANTI_AIRCRAFT => SupplyMobile::AntiAircraft,
            sys::CAMPAIGN_SUPPLY_MOBILE_PERSON => SupplyMobile::Person,
            other => return Err(format!("unknown arrival mobile kind {other}")),
        };
        let query = match source.query {
            sys::CAMPAIGN_ARRIVAL_QUERY_REACHED => ArrivalQuery::Reached,
            sys::CAMPAIGN_ARRIVAL_QUERY_ACTION => ArrivalQuery::Action,
            sys::CAMPAIGN_ARRIVAL_QUERY_RECON => ArrivalQuery::Recon,
            other => return Err(format!("unknown arrival query {other}")),
        };
        let heading_source = match source.heading_source {
            sys::CAMPAIGN_HEADING_SOURCE_NONE => None,
            sys::CAMPAIGN_HEADING_SOURCE_WAYPOINT => Some(HeadingSource::Waypoint),
            sys::CAMPAIGN_HEADING_SOURCE_ZERO => Some(HeadingSource::Zero),
            sys::CAMPAIGN_HEADING_SOURCE_TARGET => Some(HeadingSource::Target),
            other => return Err(format!("unknown arrival heading source {other}")),
        };
        checked_floats(&[
            ("rotor_spin_up_seconds", source.rotor_spin_up_seconds),
            ("distance", source.distance),
            ("velocity", source.velocity),
            ("delta_seconds", source.delta_seconds),
            ("radius", source.radius),
            ("altitude_criterion", source.altitude_criterion),
            ("heading_criterion", source.heading_criterion),
            ("altitude", source.altitude),
            ("heading", source.heading),
            ("centre_of_gravity_to_ground", source.centre_of_gravity_to_ground),
            ("guide_altitude", source.guide_altitude),
            ("waypoint_heading", source.waypoint_heading),
            ("action_radius", source.action_radius),
        ])?;
        for (name, vector) in [
            ("position", source.position),
            ("motion", source.motion),
            ("guide_position", source.guide_position),
            ("leader_position", source.leader_position),
            ("waypoint_position", source.waypoint_position),
            ("heading_target_position", source.heading_target_position),
        ] {
            for value in vector {
                if !value.is_finite() {
                    return Err(format!("non-finite supply arrival {name}"));
                }
            }
        }
        if source.delta_seconds < 0.0 || source.radius < 0.0 {
            return Err("supply_reached received a negative delta or radius".into());
        }
        let descriptor = SupplyReachDescriptor {
            member_key: source.member_key,
            task_key: source.task_key,
            waypoint_key: source.waypoint_key,
            host_id,
            query,
            waypoint: supply_waypoint(source.waypoint_kind)?,
            waypoint_sub_type: source.waypoint_sub_type,
            player_skip_waypoint: checked_bool("player_skip_waypoint", source.player_skip_waypoint)?,
            landing_waypoint: checked_bool("landing_waypoint", source.landing_waypoint)?,
            mobile,
            is_task_leader: checked_bool("is_task_leader", source.is_task_leader)?,
            is_ai: checked_bool("is_ai", source.is_ai)?,
            leader_is_ai: checked_bool("leader_is_ai", source.leader_is_ai)?,
            landed: checked_bool("landed", source.landed)?,
            updated: checked_bool("updated", source.updated)?,
            rotor_spin_up_seconds: source.rotor_spin_up_seconds,
            distance: source.distance,
            velocity: source.velocity,
            delta_seconds: source.delta_seconds,
            radius: source.radius,
            position: source.position,
            motion: source.motion,
            guide_position: source.guide_position,
            leader_position: source.leader_position,
            radius_valid: checked_bool("radius_valid", source.radius_valid)?,
            altitude_valid: checked_bool("altitude_valid", source.altitude_valid)?,
            heading_valid: checked_bool("heading_valid", source.heading_valid)?,
            last_to_reach_valid: checked_bool("last_to_reach_valid", source.last_to_reach_valid)?,
            weapon_gate: checked_bool("weapon_gate", source.weapon_gate)?,
            vectors_valid: checked_bool("vectors_valid", source.vectors_valid)?,
            airborne: checked_bool("airborne", source.airborne)?,
            auto_pilot: checked_bool("auto_pilot", source.auto_pilot)?,
            altitude_criterion: source.altitude_criterion,
            heading_criterion: source.heading_criterion,
            altitude: source.altitude,
            heading: source.heading,
            centre_of_gravity_to_ground: source.centre_of_gravity_to_ground,
            guide_altitude: source.guide_altitude,
            heading_source,
            waypoint_heading: source.waypoint_heading,
            heading_target_present: checked_bool("heading_target_present", source.heading_target_present)?,
            heading_target_position: checked_bool("heading_target_position_present", source.heading_target_position_present)?
                .then_some(source.heading_target_position),
            weapon_vector: source.weapon_vector,
            intercept_vector: source.intercept_vector,
            action_radius: source.action_radius,
            waypoint_position: source.waypoint_position,
        };
        let answer = (context.supply_reached)(descriptor)?;
        // SAFETY: C supplies a writable result for the duration of this callback.
        unsafe {
            *answer_out = sys::eech_arrival_result {
                gates_passed: c_int::from(answer.gates_passed),
                angle_present: c_int::from(answer.launch_angle_error.is_some()),
                launch_angle_error: answer.launch_angle_error.unwrap_or(0.0),
                heading_target_missing: c_int::from(answer.heading_target_missing),
            };
        }
        Ok(())
    }));
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(message)) => {
            // SAFETY: C owns this writable error buffer for the callback.
            unsafe { write_callback_text(error, error_size, &message) };
            1
        }
        Err(_) => {
            unsafe { write_callback_text(error, error_size, "supply_reached panicked") };
            1
        }
    }
}

unsafe extern "C" fn move_vehicle<F, G, H, M>(
    step: *const sys::eech_movement_step,
    result_out: *mut sys::eech_movement_result,
    error: *mut c_char,
    error_size: usize,
    user: *mut c_void,
) -> c_int
where
    F: FnMut(MemberDescriptor) -> Result<String, String>,
    G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
    H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
    M: FnMut(MovementStep) -> Result<MovementResult, String>,
{
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        if step.is_null() || result_out.is_null() || user.is_null() {
            return Err("invalid native move_vehicle callback context".into());
        }
        // SAFETY: C supplies all three pointers for the duration of this callback.
        let (source, context) = unsafe { (&*step, &mut *(user as *mut HostContext<'_, F, G, H, M>)) };
        let host_id_bytes = source.host_id.iter().map(|byte| *byte as u8).take_while(|byte| *byte != 0).collect::<Vec<_>>();
        if host_id_bytes.is_empty() || host_id_bytes.len() > sys::EECH_MOVEMENT_HOST_ID_MAX {
            return Err("move_vehicle received an invalid host identity length".into());
        }
        let host_id = std::str::from_utf8(&host_id_bytes)
            .map_err(|_| "move_vehicle received a non-UTF-8 host identity")?
            .to_owned();
        let mode = match source.mode {
            sys::EECH_MOVEMENT_MOVE => MovementMode::Move,
            sys::EECH_MOVEMENT_STOP => MovementMode::Stop,
            sys::EECH_MOVEMENT_PLACE => MovementMode::Place,
            sys::EECH_MOVEMENT_SHIP_MOVE => MovementMode::ShipMove,
            sys::EECH_MOVEMENT_PERSON_BEGIN => MovementMode::PersonBegin,
            sys::EECH_MOVEMENT_PERSON_END => MovementMode::PersonEnd,
            other => return Err(format!("move_vehicle received unknown mode {other}")),
        };
        let terrain_follow = checked_bool("terrain_follow", source.terrain_follow)?;
        let terrain_present = checked_bool("terrain_present", source.terrain_present)?;
        // a routed step follows terrain exactly when it carries a sample; an infantry begin announces a sample to come
        if mode != MovementMode::PersonBegin && terrain_follow != terrain_present {
            return Err("move_vehicle received terrain-follow without a terrain sample".into());
        }
        checked_floats(&[
            ("velocity", source.velocity),
            ("delta_seconds", source.delta_seconds),
            ("desired_velocity", source.desired_velocity),
            ("max_acceleration", source.max_acceleration),
            ("elevation", source.elevation),
            ("frame_delta", source.frame_delta),
            ("g_max", source.g_max),
            ("ps_vel", source.ps_vel),
        ])?;
        for (name, values) in [
            ("position", &source.position[..]),
            ("attitude", &source.attitude[..]),
            ("waypoint", &source.waypoint[..]),
            ("face_normal", &source.face_normal[..]),
            ("map_min", &source.map_min[..]),
            ("map_max", &source.map_max[..]),
        ] {
            if values.iter().any(|value| !value.is_finite()) {
                return Err(format!("non-finite movement {name}"));
            }
        }
        if source.delta_seconds < 0.0 || source.frame_delta < 0.0 {
            return Err("move_vehicle received a negative delta".into());
        }
        let terrain = if terrain_present {
            Some(TerrainSample {
                elevation: source.elevation,
                forest: checked_bool("forest", source.forest)?,
                face_normal: source.face_normal,
            })
        } else {
            None
        };
        let request = MovementStep {
            host_id,
            mode,
            pose: VehiclePose {
                position: source.position,
                velocity: source.velocity,
                attitude: source.attitude,
            },
            delta_seconds: source.delta_seconds,
            desired_velocity: source.desired_velocity,
            max_acceleration: source.max_acceleration,
            waypoint: source.waypoint,
            tacview_logging: checked_bool("tacview_logging", source.tacview_logging)?,
            terrain,
            map_min: source.map_min,
            map_max: source.map_max,
            frame_delta: source.frame_delta,
            g_max: source.g_max,
            ps_vel: source.ps_vel,
            terrain_follow,
            elevation: source.elevation,
        };
        let answer = (context.move_vehicle)(request)?;
        let pose = answer.pose;
        if !pose.velocity.is_finite()
            || pose
                .position
                .iter()
                .chain(pose.attitude.iter())
                .chain(answer.motion_vector.iter())
                .any(|value| !value.is_finite())
        {
            return Err("move_vehicle host returned a non-finite pose".into());
        }
        // SAFETY: C supplies a writable result for the duration of this callback.
        unsafe {
            *result_out = sys::eech_movement_result {
                position: pose.position,
                velocity: pose.velocity,
                attitude: pose.attitude,
                rotated: c_int::from(answer.rotated),
                motion_vector: answer.motion_vector,
                pending: c_int::from(answer.pending),
            };
        }
        Ok(())
    }));
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(message)) => {
            // SAFETY: C owns this writable error buffer for the callback.
            unsafe { write_callback_text(error, error_size, &message) };
            1
        }
        Err(_) => {
            // No Rust panic may unwind through C.
            unsafe { write_callback_text(error, error_size, "move_vehicle panicked") };
            1
        }
    }
}

fn with_host<F, G, H, M>(
    bind_mobile_fn: &mut F,
    observe_loss_fn: &mut G,
    supply_reached_fn: &mut H,
    move_vehicle_fn: &mut M,
    call: impl FnOnce(*const sys::eech_engine_host) -> c_int,
) -> Result<(), EngineError>
where
    F: FnMut(MemberDescriptor) -> Result<String, String>,
    G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
    H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
    M: FnMut(MovementStep) -> Result<MovementResult, String>,
{
    let mut context = HostContext {
        bind_mobile: bind_mobile_fn,
        observe_loss: observe_loss_fn,
        supply_reached: supply_reached_fn,
        move_vehicle: move_vehicle_fn,
    };
    let host = sys::eech_engine_host {
        bind_mobile: Some(bind_mobile::<F, G, H, M>),
        observe_loss: Some(observe_loss::<F, G, H, M>),
        supply_reached: Some(supply_reached::<F, G, H, M>),
        move_vehicle: Some(move_vehicle::<F, G, H, M>),
        user: (&mut context as *mut HostContext<'_, F, G, H, M>).cast::<c_void>(),
    };
    check(call(&host))
}

/// EECH's keysite usable states (en_state.h KEYSITE_USABLE_STATES)
fn usable_name(state: c_int) -> String {
    match state {
        0 => "usable",
        1 => "out_of_action",
        2 => "repairing",
        _ => "unknown",
    }
    .to_string()
}

/// writes the generated part of an installation under `root`: the 3D object
/// database and texture names (cohokum/3ddata) and the briefing texts
/// (common/data/brief_en.dat); the retail versions are not in the repository
pub fn prepare_installation(root: &Path) -> Result<(), EngineError> {
    std::fs::create_dir_all(root).map_err(|e| EngineError::BadArgument(format!("{}: {e}", root.display())))?;
    let dir = cstring(&root.to_string_lossy())?;
    // SAFETY: a valid C string for the call
    check(unsafe { sys::eech_engine_prepare_installation(dir.as_ptr()) })
}

impl Engine {
    /// boots EECH's dedicated-server path into the campaign (once per process)
    pub fn boot<F, G, H, M>(
        config: &EngineConfig,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
    ) -> Result<Engine, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
    {
        if BOOTED.swap(true, Ordering::SeqCst) {
            return Err(EngineError::State);
        }
        let install = cstring(&config.install_root.to_string_lossy())?;
        let map = cstring(&config.map_path)?;
        let dir = cstring(&config.campaign_directory)?;
        let file = cstring(&config.campaign_filename)?;
        let args: Vec<CString> = config.arguments.iter().map(|a| cstring(a)).collect::<Result<_, _>>()?;
        let mut argv: Vec<*const c_char> = args.iter().map(|a| a.as_ptr()).collect();
        argv.push(std::ptr::null());
        let c = sys::eech_engine_config {
            install_root: install.as_ptr(),
            map_path: map.as_ptr(),
            campaign_directory: dir.as_ptr(),
            campaign_filename: file.as_ptr(),
            gunship_type: config.gunship as c_int,
            random_seed: config.random_seed,
            arguments: argv.as_ptr(),
        };
        // SAFETY: every config pointer and the temporary host callback are valid for the call.
        with_host(bind_mobile_fn, observe_loss_fn, supply_reached_fn, move_vehicle_fn, |host| unsafe {
            sys::eech_engine_boot(&c, host)
        })?;
        Ok(Engine {
            _not_send: std::marker::PhantomData,
        })
    }

    /// advances simulated time and runs one iteration of EECH's flight loop
    pub fn frame<F, G, H, M>(
        &self,
        milliseconds: u32,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
    ) -> Result<(), EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
    {
        // SAFETY: the temporary host callback is valid for the call.
        with_host(bind_mobile_fn, observe_loss_fn, supply_reached_fn, move_vehicle_fn, |host| unsafe {
            sys::eech_engine_frame(milliseconds, host)
        })
    }

    /// Applies one previously withheld loss by opaque physical-incarnation ID.
    /// Duplicate, unknown, stale and unsupported reports fail visibly and return no receipt.
    pub fn report_loss(&self, host_id: &str) -> Result<LossReceipt, EngineError> {
        let host_id = cstring(host_id)?;
        let mut r = sys::eech_loss_report_receipt::default();
        // SAFETY: a valid NUL-terminated identity and out pointer for the call; the receipt is read only on success.
        check(unsafe { sys::eech_engine_report_loss(host_id.as_ptr(), &mut r) })?;
        Ok(LossReceipt {
            landing_reserved_before: r.landing_reserved_before,
            landing_reserved_after: r.landing_reserved_after,
            group_losses_before: r.group_losses_before,
            group_losses_after: r.group_losses_after,
            primary_task_losses_before: r.primary_task_losses_before,
            primary_task_losses_after: r.primary_task_losses_after,
            group_logical_members_before: r.group_logical_members_before,
            group_logical_members_after: r.group_logical_members_after,
            group_physical_members_before: r.group_physical_members_before,
            tasks_retired_expire: r.tasks_retired_expire,
            tasks_completed_on_report: r.tasks_completed_on_report,
        })
    }

    pub fn objects(&self) -> Result<Vec<Object>, EngineError> {
        unsafe extern "C" fn collect(o: *const sys::eech_object, user: *mut c_void) {
            // SAFETY: the engine passes a valid object; user is the Vec below
            let (o, out) = unsafe { (&*o, &mut *(user as *mut Vec<Object>)) };
            let text = |p: *const c_char| (!p.is_null()).then(|| unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned());
            let kind = match o.kind {
                1 => Kind::Helicopter,
                2 => Kind::FixedWing,
                3 => Kind::GroundVehicle,
                4 => Kind::AirDefence,
                5 => Kind::Ship,
                6 => Kind::Infantry,
                7 => Kind::Weapon,
                _ => Kind::Keysite,
            };
            out.push(Object {
                id: o.id,
                kind,
                sub_type: o.sub_type,
                side: side(o.side),
                alive: o.alive != 0,
                group_id: (o.group_id >= 0).then_some(o.group_id),
                type_name: text(o.type_name).unwrap_or_default(),
                name: text(o.name),
                task: text(o.task),
                state: text(o.state),
                position: [o.x, o.y, o.z],
                heading: o.heading,
                pitch: o.pitch,
                roll: o.roll,
                efficiency: o.efficiency,
                ammo: o.ammo,
                fuel: o.fuel,
                usable: (kind == Kind::Keysite).then(|| usable_name(o.usable)),
            });
        }
        let mut out: Vec<Object> = Vec::new();
        let mut count: c_int = 0;
        // SAFETY: the callback only touches `out` during the call
        check(unsafe { sys::eech_engine_objects(collect, &mut out as *mut Vec<Object> as *mut c_void, &mut count) })?;
        Ok(out)
    }

    pub fn members(&self) -> Result<Vec<Member>, EngineError> {
        unsafe extern "C" fn collect(member: *const sys::eech_member, user: *mut c_void) {
            // SAFETY: the engine passes a valid member; user is the Vec below.
            let (member, out) = unsafe { (&*member, &mut *(user as *mut Vec<Member>)) };
            let host_id = if member.host_id.is_null() {
                String::new()
            } else {
                // SAFETY: roster identities are NUL-terminated for the engine session.
                unsafe { CStr::from_ptr(member.host_id) }.to_string_lossy().into_owned()
            };
            out.push(Member {
                member_key: member.member_key,
                group_key: member.group_key,
                host_id,
                origin_entity_id: member.origin_entity_id,
                current_entity_id: (member.current_entity_id >= 0).then_some(member.current_entity_id),
                entity_type: member.entity_type,
                sub_type: member.sub_type,
                side: side(member.side),
                initial_alive: member.initial_alive != 0,
                physical_present: member.physical_present != 0,
                in_group: member.in_group != 0,
            });
        }
        let mut out: Vec<Member> = Vec::new();
        let mut count: c_int = 0;
        // SAFETY: the callback only touches `out` during the call.
        check(unsafe { sys::eech_engine_members(collect, (&mut out as *mut Vec<Member>).cast::<c_void>(), &mut count) })?;
        Ok(out)
    }

    pub fn supply_tasks(&self) -> Result<Vec<SupplyTask>, EngineError> {
        unsafe extern "C" fn collect(task: *const sys::eech_supply_task, user: *mut c_void) {
            // SAFETY: the engine passes a valid task; user is the Vec below.
            let (task, out) = unsafe { (&*task, &mut *(user as *mut Vec<SupplyTask>)) };
            let leader_host_id = task
                .leader_host_id
                .iter()
                .map(|byte| *byte as u8)
                .take_while(|byte| *byte != 0)
                .collect::<Vec<_>>();
            let leader_host_id = (!leader_host_id.is_empty()).then(|| String::from_utf8_lossy(&leader_host_id).into_owned());
            out.push(SupplyTask {
                task_key: task.task_key,
                waypoint_key: (task.waypoint_key != 0).then_some(task.waypoint_key),
                leader_host_id,
                task_entity_index: task.task_entity_index,
                waypoint_entity_index: (task.waypoint_entity_index >= 0).then_some(task.waypoint_entity_index),
                waypoint: supply_waypoint(task.waypoint_kind).ok(),
                task_completed: task.task_completed,
                supplier_entity_index: (task.supplier_entity_index >= 0).then_some(task.supplier_entity_index),
                receiver_entity_index: (task.receiver_entity_index >= 0).then_some(task.receiver_entity_index),
                cargo_type: task.cargo_type,
                cargo_on_leader: task.cargo_on_leader,
                supplier_ammo: task.supplier_ammo,
                supplier_fuel: task.supplier_fuel,
                receiver_ammo: task.receiver_ammo,
                receiver_fuel: task.receiver_fuel,
            });
        }
        let mut out: Vec<SupplyTask> = Vec::new();
        let mut count: c_int = 0;
        // SAFETY: the callback only touches `out` during the call.
        check(unsafe { sys::eech_engine_supply_tasks(collect, (&mut out as *mut Vec<SupplyTask>).cast::<c_void>(), &mut count) })?;
        Ok(out)
    }

    pub fn roster_stats(&self) -> Result<RosterStats, EngineError> {
        let mut stats = sys::eech_roster_stats::default();
        // SAFETY: a valid out pointer; the native operation guard rejects same-thread re-entry.
        check(unsafe { sys::eech_engine_roster_stats(&mut stats) })?;
        Ok(RosterStats {
            duplicate_comparisons: stats.duplicate_comparisons,
            duplicate_scan_seconds: stats.duplicate_scan_seconds,
        })
    }

    pub fn clock(&self) -> Result<Clock, EngineError> {
        let mut c = sys::eech_clock::default();
        // SAFETY: a valid out pointer
        check(unsafe { sys::eech_engine_clock(&mut c) })?;
        Ok(Clock {
            elapsed_seconds: c.elapsed_seconds,
            time_of_day_seconds: c.time_of_day_seconds,
            day: c.day,
        })
    }

    /// Writes the campaign's force state (reserves, regen queues, keysites,
    /// air groups) to the log.
    pub fn diagnostics(&self) -> Result<(), EngineError> {
        // SAFETY: no arguments; the engine checks its own state
        check(unsafe { sys::eech_engine_diagnostics() })
    }
}
