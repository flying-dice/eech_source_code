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
    /// aircraft members only: the aircraft's previous motion vector, physical state the host
    /// retains from binding (an explicit seed, never defaulted)
    pub motion_vector: Option<[f32; 3]>,
    /// helicopter members only: the main rotor rpm, tail rotor rpm and main rotor spin-up timer, physical state the
    /// host retains from binding (an explicit seed, never defaulted)
    pub rotor: Option<[f32; 3]>,
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
    /// a fixed-wing aircraft: taxi, flight, landing or the absolute turn-around, as the campaign's guidance selects
    FixedWing = 4,
    /// an alive AI helicopter: the motion vector, attitude and position in two phases around the campaign's guidance
    Helicopter = 5,
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
    /// a fixed-wing aircraft's step (single phase): the campaign's guidance selects taxi, flight, landing or absolute
    FixedWingMove,
    /// first phase of an alive AI helicopter's step: the motion vector, the speed and whether it moved, held for `HelicopterEnd`
    HelicopterBegin,
    /// second phase of an alive AI helicopter's step: the attitude, the position and the floor clamp
    HelicopterEnd,
    /// the DLL announces its rotor state (rpm, tail rpm, spin-up timer) as the host's: the explicit seed/reset it adopts
    RotorSet,
    /// an alive helicopter's rotor integration and spin-up timer
    RotorAlive,
    /// a dying helicopter's rotor wind-down by the movement delta
    RotorDown,
    /// dying phase one: the map-clamped candidate position; the host stores it
    Fall,
    /// the altitude of the held position or the stored candidate over the campaign's terrain sample
    Contact,
    /// settle on or in the terrain class at the stored candidate
    Settle,
    /// the tumble attitude and the final position and vector
    Tumble,
    /// first phase of the crash slide: the attitude and the candidate position, held for `CrashEnd`
    CrashBegin,
    /// second phase of the crash slide: the slope push, the position and whether the slide stopped
    CrashEnd,
    /// the shove along the attitude x axis
    Impact,
    /// dying phase two: gravity applied to the vector and speed after the terrain sample
    FallGravity,
    /// an idle helicopter's linear rotor wind-down by the frame delta
    RotorIdle,
    /// routed or anti-air wreck slide, phase one: the candidate x/z; the campaign refreshes the terrain there
    GvDeathCandidate,
    /// routed or anti-air wreck slide, phase two: the decayed speed and vector, the heading drift and the position
    GvDeathDecay,
    /// a falling routed or anti-air wreck: the candidate position, held until the settle
    GvFallCandidate,
    /// a falling wreck's gravity: the vector only (the scalar speed is held until the commit)
    GvFallGravity,
    /// a falling wreck's contact: stages the settled position, vector and speed and answers the contact `flags`
    GvFallSettle,
    /// a wreck's shove along the attitude x axis by the float the campaign drew
    GvImpact,
    /// a sinking ship's drag, sink rate and attitude drift; answers the new height
    ShipSink,
    /// a routed, anti-air or ship member claims its vector at a wreck, fall or sink entry
    VehicleVectorSeed,
    /// a dying fixed wing's attitude and final position, after the campaign's roll draw
    FwDeathAttitude,
    /// a fixed-wing crash slide, phase one: the candidate position
    FwCrashCandidate,
    /// a fixed-wing crash slide, phase two: friction and slope velocity
    FwCrashVelocity,
    /// the staged fall vector written after the effects
    GvFallVector,
    /// the staged fall scalar, position and pose written at the end of the falling step
    GvFallCommit,
    /// a dying fixed wing settles on or in the terrain class at the stored candidate
    FwSettle,
    /// a fixed-wing crash slide, phase three: the attitude, the staged position and the at-rest test
    FwCrashAttitude,
    /// the physical incarnation ended (heap release): the host retires the identity and answers no later step for it
    Retire,
    /// initial physical placement of a freshly created member: the host answers the pose and the Euler angles
    CreatePlace,
    /// a dead mobile's lifetime: the host accumulates the timer, gates it and decides whether the wreck has expired
    Expire,
    /// hosted helicopter control, phase one: the campaign's intent and observations; the host answers the lookups it needs
    HelicopterPlan,
    /// hosted helicopter control, phase two: the lookup answers; the host answers the guidance (lift, acceleration, climb demand)
    HelicopterGuide,
    /// hosted helicopter control, phase three: the host takes the retained guidance's acceleration and climb demand
    HelicopterGuidedBegin,
    /// hosted helicopter control, phase four: the host builds the heading, pitch and roll demands and answers the angles
    HelicopterGuidedEnd,
    /// an early exit of the hosted helicopter step (rotor or human-wait gate): the host discards the retained guidance
    HelicopterAbandon,
    /// the fixed-wing terrain look-ahead (non-mutating): the host answers the one point it will sample terrain at, from its
    /// own retained pose, speed and attitude
    FixedWingLookahead,
    /// a movement-dependent mobile follows the parent that just moved (EEDCS-40): the host checks the parent's move against its
    /// own record and answers the dependent's pose (and, for a helicopter, its attitude and speed)
    DependentCarry,
}

/// Terrain at the pre-move position, sampled by the campaign.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct TerrainSample {
    pub elevation: f32,
    pub forest: bool,
    pub face_normal: [f32; 3],
}

/// A group member as the step table carries it: a table holding its host id when the roster has one. A bare `None` in a
/// Lua array would leave a hole and end the sequence, so each member is a table (as the fixed-wing members are).
#[derive(Serialize)]
struct MemberId<'a> {
    host_id: Option<&'a str>,
}

fn serialize_members<S: serde::Serializer>(members: &[Option<String>], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(members.iter().map(|member| MemberId { host_id: member.as_deref() }))
}

/// One structure of the aircraft's own sector, as the DLL's structure scan saw it (sector list order).
#[derive(Clone, Copy, Debug, Serialize)]
pub struct HelicopterStructure {
    pub position: [f32; 3],
    pub avoidance_radius: f32,
    pub height: f32,
}

/// One terrain sample of the original `max (terrain_elevation, get_3d_terrain_point_data (...))`, as the getter returned
/// it: `second` is present only when the macro called the getter again, and the host folds the records.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct HelicopterTerrainSample {
    pub first: f32,
    pub second: Option<f32>,
}

/// What the host's plan asks the DLL to look up before the guidance (the fixed-wing look-ahead asks for one terrain sample).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HelicopterQuery {
    /// the structures of the aircraft's OWN sector are to be listed
    pub structure_scan: bool,
    /// the look-ahead point those structures are obstacle-tested against (not the sector to enumerate)
    pub structure_position: [f32; 3],
    /// the terrain samples, far to near
    pub terrain_sample_count: usize,
    pub terrain_sample: [[f32; 2]; sys::EECH_HC_TERRAIN_SAMPLES],
}

/// The host's guidance for the pre-begin physical state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HelicopterGuidanceAnswer {
    pub lift: f32,
    pub waypoint: [f32; 3],
    pub required_height: f32,
    pub world_acceleration: [f32; 3],
    pub wp_vec: [f32; 3],
}

/// The campaign's part of a terrain-following fixed-wing waypoint.  The host performs the climb arithmetic from it: the
/// look-ahead point is the host's own (`sample_point` is checked against it), the elevation is the terrain service's answer
/// there (transitional), `min_height` and `guide_position` are the campaign's intent, and `site_y` and `cg` are the
/// formation and centre-of-gravity heights added after the adjustment.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct FixedWingTerrain {
    pub min_height: f32,
    pub guide_position: [f32; 3],
    pub sample_point: [f32; 2],
    pub sample_elevation: f32,
    pub site_y: f32,
    pub cg: f32,
}

/// A group member a fixed-wing navigator avoids: the identity and the avoidance radius of its aircraft type.
#[derive(Clone, Debug, Serialize)]
pub struct NavigationMember {
    /// `None`: the campaign's roster has no identity for this member. The host refuses the step only if it reads it
    pub host_id: Option<String>,
    pub avoidance_radius: f32,
}

/// One movement request for the vehicle whose host identity is `host_id`.
/// `pose` is the campaign's copy: compared with the host's record on `Move` and `Stop`, adopted on `Place`.
/// The remaining fields depend on the mode: `Move` uses the routed set, `ShipMove` the ship set
/// (`frame_delta`, `desired_velocity` as the guide velocity, `g_max`), `PersonBegin` `ps_vel` and
/// `terrain_follow`, and `PersonEnd` `elevation`.  `FixedWingMove` carries `phase`, the guide's speed as
/// `desired_velocity`, `g_max`, the terrain `elevation`, and the fixed-wing guidance below.
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
    /// `FixedWingMove`: 0 taxi, 1 flight, 2 landing, 3 absolute
    pub phase: i32,
    /// `FixedWingMove` flight: the aircraft leads its group (its navigator makes no collision test)
    pub group_leader: bool,
    /// `FixedWingMove` flight: the aircraft leads its task and follows its waypoint, not the leader
    pub task_leader: bool,
    pub max_vertical_turn_allowance: f32,
    /// `FixedWingMove` flight: this aircraft's avoidance radius
    pub avoidance_radius: f32,
    /// `FixedWingMove` flight and landing: the task leader the aircraft pursues, when the campaign's roster has it. `None`
    /// is an unresolved observation (or an aircraft that is its own task leader); the host refuses the step only on a
    /// path that reads the leader
    pub leader_host_id: Option<String>,
    /// `FixedWingMove` flight: the group's other members in list order; empty when the aircraft leads its group.
    /// The host resolves each one's pose and motion from its own records
    pub members: Vec<NavigationMember>,
    /// `FixedWingMove`: the motion vector the previous step left (the old direction of travel); `Place`: the one
    /// aircraft placement vector (`fw_motion_vector`, fixed-wing and helicopter alike);
    /// `HelicopterBegin` and `HelicopterEnd`: the campaign's mirror of the vector the host retains
    pub motion_vector_in: [f32; 3],
    pub cruise_velocity: f32,
    pub fuselage_angle: f32,
    pub cg_to_ground: f32,
    pub max_roll: f32,
    /// `FixedWingMove` flight and landing: the DLL's cannon-aim policy found a target in range to engage (an intent fact;
    /// the host decides the turn it causes)
    pub aiming_cannon: bool,
    /// `FixedWingMove` flight and landing: the session's elapsed time, against which the host holds its turn state
    pub elapsed_time: f32,
    /// `FixedWingMove`: the operational state is navigating
    pub navigating: bool,
    /// `FixedWingMove`: the terrain observation of a terrain-following task leader's waypoint (the Lua key is
    /// `fixed_wing_terrain`; `terrain` is the routed sample); absent otherwise
    pub fixed_wing_terrain: Option<FixedWingTerrain>,
    /// `HelicopterBegin`: the campaign's acceleration decision along x and z, and the climb demand
    pub acceleration_x: f32,
    pub acceleration_z: f32,
    pub wp_vec_y: f32,
    /// `HelicopterEnd`: the guidance's desired heading, pitch and roll, and the aircraft's turn-rate limit (a physical
    /// input: the host wraps the heading to the shortest turn and bounds it to this rate)
    pub desired_heading: f32,
    pub max_turn_rate: f32,
    pub desired_pitch: f32,
    pub desired_roll: f32,
    /// helicopter lifecycle modes: the campaign's mirror of the rotor state the host retains (`RotorSet` adopts it)
    pub rotor: [f32; 3],
    /// `RotorAlive`: the lift and the power output the rotor integrates
    pub lift: f32,
    pub power_output: f32,
    /// `RotorAlive`: the spin-up gate held; `Contact`: the FALL candidate (rather than the held position) is measured
    pub flag: bool,
    /// `FallGravity`: the gravity the vector falls by
    pub gravity: f32,
    /// `Settle`: 0 solid, 1 sea, 2 river or lake, 3 forest
    pub terrain_class: i32,
    /// `Settle`: the depth beyond which the wreck is buried
    pub bury_limit: f32,
    /// `Tumble`/`FwDeathAttitude`: the campaign's random draws (heading, roll), the doubles `frand1 ()` returns: the
    /// original expression multiplies them in double before narrowing the result
    pub draws: [f64; 2],
    /// `CrashBegin`: the pitch and bank offsets
    pub pitch_offset: f32,
    pub bank_offset: f32,
    /// `Impact`: the signed speed of the shove
    pub signed_speed: f32,
    /// `CrashEnd`: the terrain class under the wreck is forest
    pub forest: bool,
    /// `CrashBegin` the OLD terrain normal, `CrashEnd` the NEW terrain normal
    pub face_normal: [f32; 3],
    /// `GvFallSettle`: the terrain elevation before the campaign's refresh at the candidate
    pub old_elevation: f32,
    /// `GvFallSettle`: the diagonal of the member's bounding box
    pub bbox_diagonal: f32,
    /// `GvDeathDecay`: the anti-air decay (the speed loss in double precision) rather than the routed one
    pub wide: bool,
    /// `GvFallSettle`: the terrain under the candidate is water
    pub water: bool,
    /// `Expire`: the campaign's mirror of the wreck's timer
    pub death_timer: f32,
    /// `Expire`: the campaign's squared 2D range to the nearest keysite; meaningful only with the range-supplied bit
    pub death_range: f32,
    /// `Expire`: the campaign's decision bits (1 never expires, 2 occupied by a player, 4 `death_range` is supplied)
    pub death_policy: i32,
    /// `HelicopterPlan`: the campaign's task and waypoint facts (the plan also uses `delta_seconds`, `terrain_follow`,
    /// `elevation` as the cached terrain elevation, `waypoint`, `map_min`, `map_max`, `motion_vector_in`, `power_output`);
    /// `HelicopterGuidedEnd` shares `hc_waypoint_kind`, `hc_task_leader_is_self`, `hc_guide_radius`, `hc_distance` and
    /// `hc_cruise_velocity`
    pub hc_group_leader: bool,
    pub hc_task_leader: bool,
    pub hc_task_leader_is_self: bool,
    pub hc_position_hold: bool,
    pub hc_terrain_follow_attack_altitude: bool,
    /// one of the `EECH_HC_WP_*` groups
    pub hc_waypoint_kind: i32,
    /// the guide's radius criterion; `None` when it has none
    pub hc_guide_radius: Option<f32>,
    pub hc_guide_altitude_valid: bool,
    pub hc_waypoint_velocity: [f32; 3],
    pub hc_distance: f32,
    pub hc_fuel_supply_level: f32,
    pub hc_cruise_velocity: f32,
    pub hc_cruise_altitude: f32,
    pub hc_attack_altitude: f32,
    /// the task leader's radar altitude: transitional terrain data, not host-owned
    pub hc_leader_radar_altitude: f32,
    /// `HelicopterPlan`: the group's other members' host ids in list order; empty when the aircraft leads its group.
    /// `None`: the campaign's roster has no identity for that member (the host refuses the step only if it reads it)
    #[serde(serialize_with = "serialize_members")]
    pub hc_plan_members: Vec<Option<String>>,
    /// `HelicopterGuide`: the structures of the aircraft's own sector (uncapped) and the terrain samples in call order
    pub hc_structures: Vec<HelicopterStructure>,
    pub hc_terrain_samples: Vec<HelicopterTerrainSample>,
    /// `HelicopterGuidedEnd`: the guide's required heading (campaign intent), when it has one
    pub hc_required_heading: Option<f32>,
    /// `HelicopterGuidedEnd`: the task leader's host id; the host reads its attitude only where the heading follows it
    pub hc_leader_host_id: Option<String>,
    pub hc_targeting_active: bool,
    pub hc_weapon_vector: [f32; 3],
    pub hc_weapon_to_intercept_vector: [f32; 3],
    pub hc_main_rotor_shaft_angle: f32,
    /// `DependentCarry`: the parent that notified the dependent (`None` in every other mode)
    pub dc_parent: Option<String>,
    /// `DependentCarry`: the dependent is a helicopter, which keeps its offset from the parent and turns it by the heading change
    pub dc_helicopter: bool,
    /// `DependentCarry`: the parent's position before and after the move the campaign is applying
    pub dc_parent_old: [f32; 3],
    pub dc_parent_new: [f32; 3],
    /// `DependentCarry`: the heading and speed the parent's setter sent its dependents
    pub dc_heading: f32,
    pub dc_velocity: f32,
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
    /// an infantry or helicopter step awaits its second phase
    pub pending: bool,
    /// `HelicopterBegin` only: the speed crossed the moved threshold
    pub moved: bool,
    /// helicopter lifecycle modes: the host's rotor state (every helicopter mode echoes it)
    pub rotor: [f32; 3],
    /// `Contact`: the altitude of the measured position over the terrain
    pub altitude: f32,
    /// per-mode result bits, checked against `result_flags_are_valid` before they reach the engine:
    /// `GvFallSettle` bit 0 contact, bit 1 splash, bit 2 sunk, bit 3 ground impact (splash and sunk need contact and
    /// water; ground impact needs contact and excludes both); `FwDeathAttitude` -1 the gate and draw disagree, 0
    /// nothing eased, 1 eased, 3 eased and the roll draw consumed; `FwCrashVelocity` bit 0 stopped;
    /// `FwCrashAttitude` bit 0 at rest; every other mode 0
    pub flags: i32,
    /// `CreatePlace` only: the heading, pitch and roll the host placed the member at (zero for every other mode); the
    /// host's physical position is `pose.position`
    pub angles: [f32; 3],
    /// `Expire` only: the host's timer after the step, which the campaign mirrors (zero for every other mode)
    pub death_timer: f32,
    /// `HelicopterPlan` (the lookups) and `FixedWingLookahead` (one terrain sample) only: what the DLL must look up (`None`
    /// in every other mode)
    pub query: Option<HelicopterQuery>,
    /// `HelicopterGuide` only: the host's guidance (`None` in every other mode)
    pub guidance: Option<HelicopterGuidanceAnswer>,
}

/// Whether `flags` is a value `mode` can answer.  The bits are not generic truthiness: each mode defines its own.
pub fn result_flags_are_valid(mode: MovementMode, flags: i32) -> bool {
    match mode {
        // contact alone; contact with the water effects (splash, sunk, both); contact with the ground impact
        MovementMode::GvFallSettle => matches!(flags, 0 | 1 | 3 | 5 | 7 | 9),
        MovementMode::FwDeathAttitude => matches!(flags, -1 | 0 | 1 | 3),
        MovementMode::FwCrashVelocity | MovementMode::FwCrashAttitude => matches!(flags, 0 | 1),
        // not expired; expired; or the gate fell and the host needs the campaign's keysite range before it can decide
        MovementMode::Expire => matches!(flags, 0..=2),
        _ => flags == 0,
    }
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

/// TEST-ONLY (feature `los-fixture`, EEDCS-42): a fold of line-of-sight queries. `hash` is an order-dependent FNV-1a over each
/// query's criteria, six position floats (bit patterns) and answer; the counts are per criteria value (index = the criteria bits).
#[cfg(feature = "los-fixture")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LosTrace {
    pub hash: u64,
    pub calls: u64,
    pub visible: u64,
    pub skipped: u64,
    pub by_criteria_calls: Vec<u64>,
    pub by_criteria_visible: Vec<u64>,
}

#[cfg(feature = "los-fixture")]
impl LosTrace {
    fn blank() -> sys::eech_los_trace {
        sys::eech_los_trace {
            hash: 0,
            calls: 0,
            visible: 0,
            skipped: 0,
            by_criteria_calls: [0; sys::EECH_LOS_CRITERIA],
            by_criteria_visible: [0; sys::EECH_LOS_CRITERIA],
        }
    }
}

#[cfg(feature = "los-fixture")]
impl From<sys::eech_los_trace> for LosTrace {
    fn from(raw: sys::eech_los_trace) -> Self {
        Self {
            hash: raw.hash,
            calls: raw.calls,
            visible: raw.visible,
            skipped: raw.skipped,
            by_criteria_calls: raw.by_criteria_calls.to_vec(),
            by_criteria_visible: raw.by_criteria_visible.to_vec(),
        }
    }
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
        let movement_kind = if checked_bool("has_pose", descriptor.has_pose)? {
            Some(match descriptor.movement_kind {
                sys::EECH_MOVEMENT_KIND_ROUTED => MovementKind::Routed,
                sys::EECH_MOVEMENT_KIND_SHIP => MovementKind::Ship,
                sys::EECH_MOVEMENT_KIND_PERSON => MovementKind::Person,
                sys::EECH_MOVEMENT_KIND_FIXED_WING => MovementKind::FixedWing,
                sys::EECH_MOVEMENT_KIND_HELICOPTER => MovementKind::Helicopter,
                other => return Err(format!("bind_mobile received unknown movement kind {other}")),
            })
        } else {
            None
        };
        let motion_vector = matches!(movement_kind, Some(MovementKind::FixedWing | MovementKind::Helicopter)).then_some(descriptor.motion_vector);
        if motion_vector.is_some_and(|vector| vector.iter().any(|value| !value.is_finite())) {
            return Err("bind_mobile received a non-finite motion vector".into());
        }
        let rotor = matches!(movement_kind, Some(MovementKind::Helicopter)).then_some(descriptor.rotor);
        if rotor.is_some_and(|rotor| rotor.iter().any(|value| !value.is_finite())) {
            return Err("bind_mobile received a non-finite rotor state".into());
        }
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
            movement_kind,
            motion_vector,
            rotor,
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

/// `fw_phase` of normal flight
const FIXED_WING_FLIGHT_PHASE: i32 = 1;
/// `fw_phase` of a landing approach; with flight, the phases whose navigation and turn intent the campaign sends
const FIXED_WING_LAND_PHASE: i32 = 2;

/// Reads one NUL-terminated host identity through a pointer the caller supplied, never past the longest legal identity.
/// A null pointer or an empty string is an identity the campaign does not have: `None`, not an error. One that is
/// supplied but too long or not UTF-8 is refused.
///
/// # Safety
/// `id` is null or points to at least `EECH_MOVEMENT_HOST_ID_MAX + 1` readable bytes, or to a NUL-terminated string
/// shorter than that.
unsafe fn navigation_pointer_id(id: *const c_char, role: &str) -> Result<Option<String>, String> {
    if id.is_null() {
        return Ok(None);
    }
    let mut bytes = Vec::new();
    for index in 0..=sys::EECH_MOVEMENT_HOST_ID_MAX {
        // SAFETY: the caller guarantees the bytes up to the terminator, or `EECH_MOVEMENT_HOST_ID_MAX + 1` of them
        let byte = unsafe { *id.add(index) } as u8;
        if byte == 0 {
            break;
        }
        bytes.push(byte);
    }
    checked_navigation_id(bytes, role)
}

fn navigation_id(bytes: &[c_char], role: &str) -> Result<Option<String>, String> {
    let bytes = bytes.iter().map(|byte| *byte as u8).take_while(|byte| *byte != 0).collect::<Vec<_>>();
    checked_navigation_id(bytes, role)
}

fn checked_navigation_id(bytes: Vec<u8>, role: &str) -> Result<Option<String>, String> {
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes.len() > sys::EECH_MOVEMENT_HOST_ID_MAX {
        return Err(format!("move_vehicle received an invalid {role} host identity length"));
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| format!("move_vehicle received a non-UTF-8 {role} host identity"))
}

type Navigation = (bool, bool, Option<String>, Vec<NavigationMember>);

/// The variable-length inputs of a hosted helicopter step, converted for the mode that carries them.
#[derive(Default)]
struct HelicopterLists {
    members: Vec<Option<String>>,
    structures: Vec<HelicopterStructure>,
    terrain_samples: Vec<HelicopterTerrainSample>,
    leader_host_id: Option<String>,
}

/// The lists a hosted helicopter mode carries: `HelicopterPlan` the group's other members (read only when the aircraft
/// does not lead its group), `HelicopterGuide` the own-sector structures and the terrain getter returns,
/// `HelicopterGuidedEnd` the task leader's host id. No count is refused but a negative one, and a null list for a
/// positive one; every number must be finite.
///
/// # Safety
/// `hc_member_host_id` points to `hc_member_count` NUL-terminated ids, `hc_structures` to `hc_structure_count` entries
/// and `hc_terrain_samples` to `hc_terrain_sample_count` entries, each readable and only for the mode that uses it,
/// whenever the count is positive.
unsafe fn helicopter_lists(mode: MovementMode, source: &sys::eech_movement_step) -> Result<HelicopterLists, String> {
    let mut lists = HelicopterLists::default();
    match mode {
        MovementMode::HelicopterPlan => {
            if !checked_bool("hc_group_leader", source.hc_group_leader)? {
                let count = usize::try_from(source.hc_member_count)
                    .map_err(|_| format!("move_vehicle received an invalid group member count {}", source.hc_member_count))?;
                if count > 0 && source.hc_member_host_id.is_null() {
                    return Err(format!("move_vehicle received {count} group members without their identity list"));
                }
                for index in 0..count {
                    // SAFETY: the list holds `count` entries, per this function's contract
                    let id = unsafe { *source.hc_member_host_id.add(index) };
                    // SAFETY: each listed id is a NUL-terminated string, per this function's contract
                    lists.members.push(unsafe { navigation_pointer_id(id, "group member") }?);
                }
            }
        }
        MovementMode::HelicopterGuide => {
            let structures = usize::try_from(source.hc_structure_count)
                .map_err(|_| format!("move_vehicle received an invalid structure count {}", source.hc_structure_count))?;
            if structures > 0 && source.hc_structures.is_null() {
                return Err(format!("move_vehicle received {structures} structures without their list"));
            }
            for index in 0..structures {
                // SAFETY: the list holds `structures` entries, per this function's contract
                let entry = unsafe { *source.hc_structures.add(index) };
                if entry
                    .position
                    .iter()
                    .chain([&entry.avoidance_radius, &entry.height])
                    .any(|value| !value.is_finite())
                {
                    return Err("non-finite movement hc_structures".into());
                }
                lists.structures.push(HelicopterStructure {
                    position: entry.position,
                    avoidance_radius: entry.avoidance_radius,
                    height: entry.height,
                });
            }
            let samples = usize::try_from(source.hc_terrain_sample_count)
                .map_err(|_| format!("move_vehicle received an invalid terrain sample count {}", source.hc_terrain_sample_count))?;
            if samples > 0 && source.hc_terrain_samples.is_null() {
                return Err(format!("move_vehicle received {samples} terrain samples without their list"));
            }
            for index in 0..samples {
                // SAFETY: the list holds `samples` entries, per this function's contract
                let entry = unsafe { *source.hc_terrain_samples.add(index) };
                let second = checked_bool("hc_terrain_sample second_present", entry.second_present)?;
                if !entry.first.is_finite() || (second && !entry.second.is_finite()) {
                    return Err("non-finite movement hc_terrain_samples".into());
                }
                lists.terrain_samples.push(HelicopterTerrainSample {
                    first: entry.first,
                    second: second.then_some(entry.second),
                });
            }
        }
        MovementMode::HelicopterGuidedEnd => {
            lists.leader_host_id = navigation_id(&source.hc_leader_host_id, "task leader")?;
        }
        _ => {}
    }
    Ok(lists)
}

/// The flight navigation intent: the leader flags, the task leader's identity (carried only when the aircraft does not
/// lead its task) and the group's other members in order (carried only when it does not lead its group). An identity
/// the campaign's roster lacks arrives as `None`: whether the host reads it depends on what it finds, so only the host
/// refuses it. The group is as large as the campaign listed: no count is refused but a negative one, and a null list
/// for a positive one.
///
/// # Safety
/// `fw_member_host_id` and `fw_member_radius` each point to `fw_member_count` readable entries (each id a
/// NUL-terminated string) whenever the aircraft does not lead its group and the count is positive.
unsafe fn navigation_from(source: &sys::eech_movement_step) -> Result<Navigation, String> {
    let group_leader = checked_bool("fw_group_leader", source.fw_group_leader)?;
    let task_leader = checked_bool("fw_task_leader", source.fw_task_leader)?;
    let leader_host_id = if task_leader {
        None
    } else {
        navigation_id(&source.fw_leader_host_id, "task leader")?
    };
    let mut members = Vec::new();
    if !group_leader {
        let count =
            usize::try_from(source.fw_member_count).map_err(|_| format!("move_vehicle received an invalid group member count {}", source.fw_member_count))?;
        if count > 0 && (source.fw_member_host_id.is_null() || source.fw_member_radius.is_null()) {
            return Err(format!("move_vehicle received {count} group members without their identity or radius list"));
        }
        for index in 0..count {
            // SAFETY: both lists hold `count` entries, per this function's contract
            let (id, avoidance_radius) = unsafe { (*source.fw_member_host_id.add(index), *source.fw_member_radius.add(index)) };
            if !avoidance_radius.is_finite() {
                return Err("non-finite movement fw_member_radius".into());
            }
            members.push(NavigationMember {
                // SAFETY: each listed id is a NUL-terminated string, per this function's contract
                host_id: unsafe { navigation_pointer_id(id, "group member") }?,
                avoidance_radius,
            });
        }
    }
    Ok((group_leader, task_leader, leader_host_id, members))
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
            sys::EECH_MOVEMENT_FIXED_WING_MOVE => MovementMode::FixedWingMove,
            sys::EECH_MOVEMENT_HELICOPTER_BEGIN => MovementMode::HelicopterBegin,
            sys::EECH_MOVEMENT_HELICOPTER_END => MovementMode::HelicopterEnd,
            sys::EECH_MOVEMENT_ROTOR_SET => MovementMode::RotorSet,
            sys::EECH_MOVEMENT_ROTOR_ALIVE => MovementMode::RotorAlive,
            sys::EECH_MOVEMENT_ROTOR_DOWN => MovementMode::RotorDown,
            sys::EECH_MOVEMENT_FALL => MovementMode::Fall,
            sys::EECH_MOVEMENT_CONTACT => MovementMode::Contact,
            sys::EECH_MOVEMENT_SETTLE => MovementMode::Settle,
            sys::EECH_MOVEMENT_TUMBLE => MovementMode::Tumble,
            sys::EECH_MOVEMENT_CRASH_BEGIN => MovementMode::CrashBegin,
            sys::EECH_MOVEMENT_CRASH_END => MovementMode::CrashEnd,
            sys::EECH_MOVEMENT_IMPACT => MovementMode::Impact,
            sys::EECH_MOVEMENT_FALL_GRAVITY => MovementMode::FallGravity,
            sys::EECH_MOVEMENT_ROTOR_IDLE => MovementMode::RotorIdle,
            sys::EECH_MOVEMENT_GV_DEATH_CANDIDATE => MovementMode::GvDeathCandidate,
            sys::EECH_MOVEMENT_GV_DEATH_DECAY => MovementMode::GvDeathDecay,
            sys::EECH_MOVEMENT_GV_FALL_CANDIDATE => MovementMode::GvFallCandidate,
            sys::EECH_MOVEMENT_GV_FALL_GRAVITY => MovementMode::GvFallGravity,
            sys::EECH_MOVEMENT_GV_FALL_SETTLE => MovementMode::GvFallSettle,
            sys::EECH_MOVEMENT_GV_IMPACT => MovementMode::GvImpact,
            sys::EECH_MOVEMENT_SHIP_SINK => MovementMode::ShipSink,
            sys::EECH_MOVEMENT_VEHICLE_VECTOR_SEED => MovementMode::VehicleVectorSeed,
            sys::EECH_MOVEMENT_FW_DEATH_ATTITUDE => MovementMode::FwDeathAttitude,
            sys::EECH_MOVEMENT_FW_CRASH_CANDIDATE => MovementMode::FwCrashCandidate,
            sys::EECH_MOVEMENT_FW_CRASH_VELOCITY => MovementMode::FwCrashVelocity,
            sys::EECH_MOVEMENT_GV_FALL_VECTOR => MovementMode::GvFallVector,
            sys::EECH_MOVEMENT_GV_FALL_COMMIT => MovementMode::GvFallCommit,
            sys::EECH_MOVEMENT_FW_SETTLE => MovementMode::FwSettle,
            sys::EECH_MOVEMENT_FW_CRASH_ATTITUDE => MovementMode::FwCrashAttitude,
            sys::EECH_MOVEMENT_RETIRE => MovementMode::Retire,
            sys::EECH_MOVEMENT_CREATE_PLACE => MovementMode::CreatePlace,
            sys::EECH_MOVEMENT_EXPIRE => MovementMode::Expire,
            sys::EECH_MOVEMENT_HELICOPTER_PLAN => MovementMode::HelicopterPlan,
            sys::EECH_MOVEMENT_HELICOPTER_GUIDE => MovementMode::HelicopterGuide,
            sys::EECH_MOVEMENT_HELICOPTER_GUIDED_BEGIN => MovementMode::HelicopterGuidedBegin,
            sys::EECH_MOVEMENT_HELICOPTER_GUIDED_END => MovementMode::HelicopterGuidedEnd,
            sys::EECH_MOVEMENT_HELICOPTER_ABANDON => MovementMode::HelicopterAbandon,
            sys::EECH_MOVEMENT_FIXED_WING_LOOKAHEAD => MovementMode::FixedWingLookahead,
            sys::EECH_MOVEMENT_DEPENDENT_CARRY => MovementMode::DependentCarry,
            other => return Err(format!("move_vehicle received unknown mode {other}")),
        };
        let terrain_follow = checked_bool("terrain_follow", source.terrain_follow)?;
        let terrain_present = checked_bool("terrain_present", source.terrain_present)?;
        // a routed step follows terrain exactly when it carries a sample; an infantry begin announces a sample to come
        // an infantry begin and a helicopter end announce their terrain sample (the elevation) separately
        let announces_terrain = matches!(
            mode,
            MovementMode::PersonBegin | MovementMode::HelicopterEnd | MovementMode::HelicopterPlan | MovementMode::HelicopterGuidedEnd
        );
        if !announces_terrain && terrain_follow != terrain_present {
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
            ("fw_cruise_velocity", source.fw_cruise_velocity),
            ("fw_fuselage_angle", source.fw_fuselage_angle),
            ("fw_cg_to_ground", source.fw_cg_to_ground),
            ("fw_max_roll", source.fw_max_roll),
            ("fw_elapsed_time", source.fw_elapsed_time),
            ("fw_max_vertical_turn_allowance", source.fw_max_vertical_turn_allowance),
            ("fw_avoidance_radius", source.fw_avoidance_radius),
            ("hc_acceleration_x", source.hc_acceleration_x),
            ("hc_acceleration_z", source.hc_acceleration_z),
            ("hc_wp_vec_y", source.hc_wp_vec_y),
            ("hc_desired_heading", source.hc_desired_heading),
            ("hc_max_turn_rate", source.hc_max_turn_rate),
            ("hc_desired_pitch", source.hc_desired_pitch),
            ("hc_desired_roll", source.hc_desired_roll),
            ("hc_cg_to_ground", source.hc_cg_to_ground),
            ("hc_lift", source.hc_lift),
            ("hc_power_output", source.hc_power_output),
            ("hc_gravity", source.hc_gravity),
            ("hc_bury_limit", source.hc_bury_limit),
            ("hc_pitch_offset", source.hc_pitch_offset),
            ("hc_bank_offset", source.hc_bank_offset),
            ("hc_signed_speed", source.hc_signed_speed),
            ("gv_old_elevation", source.gv_old_elevation),
            ("gv_bbox_diagonal", source.gv_bbox_diagonal),
            ("death_timer", source.death_timer),
            ("death_range", source.death_range),
            ("hc_guide_radius", source.hc_guide_radius),
            ("hc_distance", source.hc_distance),
            ("hc_fuel_supply_level", source.hc_fuel_supply_level),
            ("hc_cruise_velocity", source.hc_cruise_velocity),
            ("hc_cruise_altitude", source.hc_cruise_altitude),
            ("hc_attack_altitude", source.hc_attack_altitude),
            ("hc_leader_radar_altitude", source.hc_leader_radar_altitude),
            ("hc_required_heading", source.hc_required_heading),
            ("hc_main_rotor_shaft_angle", source.hc_main_rotor_shaft_angle),
            ("dc_heading", source.dc_heading),
            ("dc_velocity", source.dc_velocity),
        ])?;
        for (name, values) in [
            ("position", &source.position[..]),
            ("attitude", &source.attitude[..]),
            ("waypoint", &source.waypoint[..]),
            ("face_normal", &source.face_normal[..]),
            ("map_min", &source.map_min[..]),
            ("map_max", &source.map_max[..]),
            ("fw_motion_vector", &source.fw_motion_vector[..]),
            ("hc_motion_vector", &source.hc_motion_vector[..]),
            ("hc_rotor", &source.hc_rotor[..]),
            ("hc_waypoint_velocity", &source.hc_waypoint_velocity[..]),
            ("hc_weapon_vector", &source.hc_weapon_vector[..]),
            ("hc_weapon_to_intercept_vector", &source.hc_weapon_to_intercept_vector[..]),
            ("dc_parent_old", &source.dc_parent_old[..]),
            ("dc_parent_new", &source.dc_parent_new[..]),
        ] {
            if values.iter().any(|value| !value.is_finite()) {
                return Err(format!("non-finite movement {name}"));
            }
        }
        if source.hc_draws.iter().any(|value| !value.is_finite()) {
            return Err("non-finite movement hc_draws".into());
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
        // every helicopter mode carries its vector and CG height in the helicopter fields
        let helicopter_step = !matches!(
            mode,
            MovementMode::Move
                | MovementMode::Stop
                | MovementMode::Place
                | MovementMode::ShipMove
                | MovementMode::PersonBegin
                | MovementMode::PersonEnd
                | MovementMode::FixedWingMove
                | MovementMode::FixedWingLookahead
                | MovementMode::DependentCarry
                | MovementMode::Retire
                | MovementMode::CreatePlace
                | MovementMode::Expire
        );
        // the terrain observation belongs to a fixed-wing move; its floats are read only when it is present
        let fixed_wing_terrain = if checked_bool("fw_terrain", source.fw_terrain)? {
            if mode != MovementMode::FixedWingMove {
                return Err(format!("move_vehicle received a fixed-wing terrain observation for {mode:?}"));
            }
            checked_floats(&[
                ("fw_terrain_min_height", source.fw_terrain_min_height),
                ("fw_terrain_sample_elevation", source.fw_terrain_sample_elevation),
                ("fw_terrain_site_y", source.fw_terrain_site_y),
                ("fw_terrain_cg", source.fw_terrain_cg),
            ])?;
            for (name, values) in [
                ("fw_terrain_guide_position", &source.fw_terrain_guide_position[..]),
                ("fw_terrain_sample_point", &source.fw_terrain_sample_point[..]),
            ] {
                if values.iter().any(|value| !value.is_finite()) {
                    return Err(format!("non-finite movement {name}"));
                }
            }
            Some(FixedWingTerrain {
                min_height: source.fw_terrain_min_height,
                guide_position: source.fw_terrain_guide_position,
                sample_point: source.fw_terrain_sample_point,
                sample_elevation: source.fw_terrain_sample_elevation,
                site_y: source.fw_terrain_site_y,
                cg: source.fw_terrain_cg,
            })
        } else {
            None
        };
        let navigation = if mode == MovementMode::FixedWingMove && matches!(source.fw_phase, FIXED_WING_FLIGHT_PHASE | FIXED_WING_LAND_PHASE) {
            // SAFETY: the C side supplies the member lists for the duration of this callback
            Some(unsafe { navigation_from(source) }?)
        } else {
            None
        };
        let (group_leader, task_leader, leader_host_id, members) = navigation.unwrap_or_default();
        // SAFETY: the C side supplies the hosted helicopter lists for the duration of this callback
        let lists = unsafe { helicopter_lists(mode, source) }?;
        let dc_parent = navigation_id(&source.dc_parent_host_id, "dependent parent")?;
        if dc_parent.is_some() != (mode == MovementMode::DependentCarry) {
            return Err(format!(
                "move_vehicle received {mode:?} {} a dependent parent",
                if dc_parent.is_some() { "with" } else { "without" }
            ));
        }
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
            phase: source.fw_phase,
            group_leader,
            task_leader,
            max_vertical_turn_allowance: source.fw_max_vertical_turn_allowance,
            avoidance_radius: source.fw_avoidance_radius,
            leader_host_id,
            members,
            motion_vector_in: if helicopter_step { source.hc_motion_vector } else { source.fw_motion_vector },
            cruise_velocity: source.fw_cruise_velocity,
            fuselage_angle: source.fw_fuselage_angle,
            cg_to_ground: if helicopter_step { source.hc_cg_to_ground } else { source.fw_cg_to_ground },
            max_roll: source.fw_max_roll,
            aiming_cannon: checked_bool("fw_aiming_cannon", source.fw_aiming_cannon)?,
            elapsed_time: source.fw_elapsed_time,
            navigating: checked_bool("fw_navigating", source.fw_navigating)?,
            fixed_wing_terrain,
            acceleration_x: source.hc_acceleration_x,
            acceleration_z: source.hc_acceleration_z,
            wp_vec_y: source.hc_wp_vec_y,
            desired_heading: source.hc_desired_heading,
            max_turn_rate: source.hc_max_turn_rate,
            desired_pitch: source.hc_desired_pitch,
            desired_roll: source.hc_desired_roll,
            rotor: source.hc_rotor,
            lift: source.hc_lift,
            power_output: source.hc_power_output,
            flag: checked_bool("hc_flag", source.hc_flag)?,
            gravity: source.hc_gravity,
            terrain_class: source.hc_terrain_class,
            bury_limit: source.hc_bury_limit,
            draws: source.hc_draws,
            pitch_offset: source.hc_pitch_offset,
            bank_offset: source.hc_bank_offset,
            signed_speed: source.hc_signed_speed,
            forest: checked_bool("forest", source.forest)?,
            face_normal: source.face_normal,
            old_elevation: source.gv_old_elevation,
            bbox_diagonal: source.gv_bbox_diagonal,
            wide: checked_bool("gv_wide", source.gv_wide)?,
            water: checked_bool("gv_water", source.gv_water)?,
            death_timer: source.death_timer,
            death_range: source.death_range,
            death_policy: source.death_policy,
            hc_group_leader: checked_bool("hc_group_leader", source.hc_group_leader)?,
            hc_task_leader: checked_bool("hc_task_leader", source.hc_task_leader)?,
            hc_task_leader_is_self: checked_bool("hc_task_leader_is_self", source.hc_task_leader_is_self)?,
            hc_position_hold: checked_bool("hc_position_hold", source.hc_position_hold)?,
            hc_terrain_follow_attack_altitude: checked_bool("hc_terrain_follow_attack_altitude", source.hc_terrain_follow_attack_altitude)?,
            hc_waypoint_kind: source.hc_waypoint_kind,
            hc_guide_radius: checked_bool("hc_guide_radius_valid", source.hc_guide_radius_valid)?.then_some(source.hc_guide_radius),
            hc_guide_altitude_valid: checked_bool("hc_guide_altitude_valid", source.hc_guide_altitude_valid)?,
            hc_waypoint_velocity: source.hc_waypoint_velocity,
            hc_distance: source.hc_distance,
            hc_fuel_supply_level: source.hc_fuel_supply_level,
            hc_cruise_velocity: source.hc_cruise_velocity,
            hc_cruise_altitude: source.hc_cruise_altitude,
            hc_attack_altitude: source.hc_attack_altitude,
            hc_leader_radar_altitude: source.hc_leader_radar_altitude,
            hc_plan_members: lists.members,
            hc_structures: lists.structures,
            hc_terrain_samples: lists.terrain_samples,
            hc_required_heading: checked_bool("hc_required_heading_valid", source.hc_required_heading_valid)?.then_some(source.hc_required_heading),
            hc_leader_host_id: lists.leader_host_id,
            hc_targeting_active: checked_bool("hc_targeting_active", source.hc_targeting_active)?,
            hc_weapon_vector: source.hc_weapon_vector,
            hc_weapon_to_intercept_vector: source.hc_weapon_to_intercept_vector,
            hc_main_rotor_shaft_angle: source.hc_main_rotor_shaft_angle,
            dc_parent,
            dc_helicopter: checked_bool("dc_helicopter", source.dc_helicopter)?,
            dc_parent_old: source.dc_parent_old,
            dc_parent_new: source.dc_parent_new,
            dc_heading: source.dc_heading,
            dc_velocity: source.dc_velocity,
        };
        let answer = (context.move_vehicle)(request)?;
        if !result_flags_are_valid(mode, answer.flags) {
            return Err(format!("move_vehicle host returned flags {} that {mode:?} cannot answer", answer.flags));
        }
        // the lookup plan and the guidance each belong to their mode: a missing one would reach C as zeros
        if answer.query.is_some() != matches!(mode, MovementMode::HelicopterPlan | MovementMode::FixedWingLookahead) {
            return Err(format!(
                "move_vehicle host {} a lookup plan for {mode:?}",
                if answer.query.is_some() { "returned" } else { "did not return" }
            ));
        }
        if answer.guidance.is_some() != (mode == MovementMode::HelicopterGuide) {
            return Err(format!(
                "move_vehicle host {} guidance for {mode:?}",
                if answer.guidance.is_some() { "returned" } else { "did not return" }
            ));
        }
        let query = answer.query.unwrap_or(HelicopterQuery {
            structure_scan: false,
            structure_position: [0.0; 3],
            terrain_sample_count: 0,
            terrain_sample: [[0.0; 2]; sys::EECH_HC_TERRAIN_SAMPLES],
        });
        if query.terrain_sample_count > sys::EECH_HC_TERRAIN_SAMPLES {
            return Err(format!("move_vehicle host asked for {} terrain samples", query.terrain_sample_count));
        }
        if mode == MovementMode::FixedWingLookahead && query.terrain_sample_count != 1 {
            return Err(format!(
                "move_vehicle host answered the fixed-wing look-ahead with {} terrain samples",
                query.terrain_sample_count
            ));
        }
        let guidance = answer.guidance.unwrap_or(HelicopterGuidanceAnswer {
            lift: 0.0,
            waypoint: [0.0; 3],
            required_height: 0.0,
            world_acceleration: [0.0; 3],
            wp_vec: [0.0; 3],
        });
        let pose = answer.pose;
        if !pose.velocity.is_finite()
            || pose
                .position
                .iter()
                .chain(pose.attitude.iter())
                .chain(answer.motion_vector.iter())
                .chain(answer.rotor.iter())
                .chain(answer.angles.iter())
                .chain(std::iter::once(&answer.altitude))
                .chain(std::iter::once(&answer.death_timer))
                .chain(query.structure_position.iter())
                .chain(query.terrain_sample.iter().flatten())
                .chain(guidance.waypoint.iter())
                .chain(guidance.world_acceleration.iter())
                .chain(guidance.wp_vec.iter())
                .chain([&guidance.lift, &guidance.required_height])
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
                moved: c_int::from(answer.moved),
                rotor: answer.rotor,
                altitude: answer.altitude,
                flags: answer.flags,
                angles: answer.angles,
                death_timer: answer.death_timer,
                hc_query: sys::eech_hc_query {
                    structure_scan: c_int::from(query.structure_scan),
                    structure_position: query.structure_position,
                    // bounded by `EECH_HC_TERRAIN_SAMPLES` above
                    terrain_sample_count: query.terrain_sample_count as c_int,
                    terrain_sample: query.terrain_sample,
                },
                hc_guidance: sys::eech_hc_guidance_answer {
                    lift: guidance.lift,
                    waypoint: guidance.waypoint,
                    required_height: guidance.required_height,
                    world_acceleration: guidance.world_acceleration,
                    wp_vec: guidance.wp_vec,
                },
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

    /// TEST-ONLY (feature `dependent-fixture`, EEDCS-40): re-links `child` under `parent` as a movement dependent
    /// (`op` 1), moves `parent` through the campaign's own position setter (`op` 2, `position`), or runs a ship
    /// parent's own sinking step (`op` 3). The campaign's genuine notification handlers do the rest; the host answers
    /// the movement steps they send. Not part of the shipping module.
    #[cfg(feature = "dependent-fixture")]
    #[allow(clippy::too_many_arguments)]
    pub fn dependent_fixture<F, G, H, M>(
        &self,
        op: i32,
        parent: i32,
        child: i32,
        position: [f64; 3],
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
        // SAFETY: the position array and the temporary host callback are valid for the call.
        with_host(bind_mobile_fn, observe_loss_fn, supply_reached_fn, move_vehicle_fn, |host| unsafe {
            sys::eech_engine_dependent_fixture(op, parent, child, position.as_ptr(), host)
        })
    }

    /// TEST-ONLY (feature `loss-fixture`): ends one live hosted member through the campaign's own kill path.
    #[cfg(feature = "loss-fixture")]
    pub fn loss_fixture<F, G, H, M>(
        &self,
        entity_id: i32,
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
            sys::eech_engine_loss_fixture(entity_id, host)
        })
    }

    /// TEST-ONLY (feature `loss-fixture`): releases the tasks that target one hosted member's group, through the campaign's own task
    /// completion handlers, and returns how many it notified.  Setup for the fixed-wing witness; the group itself is not ended.
    #[cfg(feature = "loss-fixture")]
    pub fn loss_release<F, G, H, M>(
        &self,
        entity_id: i32,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
    ) -> Result<i32, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
    {
        let mut released = 0i32;
        // SAFETY: the out pointer and the temporary host callback are valid for the call.
        with_host(bind_mobile_fn, observe_loss_fn, supply_reached_fn, move_vehicle_fn, |host| unsafe {
            sys::eech_engine_loss_release(entity_id, &mut released, host)
        })?;
        Ok(released)
    }

    /// TEST-ONLY (feature `loss-fixture`): one side's total losses, the category and regen-queue counts, then force-info current
    /// hardware per category, then per regen queue its count, front and size and the type, sub type and group of the slot written
    /// last, read-only.  These are the values the force-info and regen loss phases change.
    #[cfg(feature = "loss-fixture")]
    pub fn loss_state(&self, side: i32) -> Result<Vec<i32>, EngineError> {
        let mut values = [0i32; 256];
        let mut count = 0i32;
        // SAFETY: the buffer and count outlive the call and the capacity passed is the buffer length.
        check(unsafe { sys::eech_engine_loss_state(side, values.as_mut_ptr(), values.len() as i32, &mut count) })?;
        Ok(values[..count as usize].to_vec())
    }

    /// TEST-ONLY (feature `loss-fixture`): why a bounded family would refuse to withhold this hosted member's loss, or `""` when it
    /// would accept; read-only, runs the guard the withhold itself runs.
    #[cfg(feature = "loss-fixture")]
    pub fn loss_shape(&self, entity_id: i32) -> Result<String, EngineError> {
        let mut reason = [0 as std::os::raw::c_char; 256];
        // SAFETY: the buffer outlives the call and the capacity passed is its length; the engine NUL-terminates it.
        check(unsafe { sys::eech_engine_loss_shape(entity_id, reason.as_mut_ptr(), reason.len() as i32) })?;
        // SAFETY: the engine wrote a NUL-terminated string within the buffer.
        Ok(unsafe { std::ffi::CStr::from_ptr(reason.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// TEST-ONLY (feature `loss-fixture`): the task dependents and guide stack of this hosted member's group as text; read-only.
    #[cfg(feature = "loss-fixture")]
    pub fn loss_group(&self, entity_id: i32) -> Result<String, EngineError> {
        let mut text = [0 as std::os::raw::c_char; 512];
        // SAFETY: the buffer outlives the call and the capacity passed is its length; the engine NUL-terminates it.
        check(unsafe { sys::eech_engine_loss_group(entity_id, text.as_mut_ptr(), text.len() as i32) })?;
        // SAFETY: the engine wrote a NUL-terminated string within the buffer.
        Ok(unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// TEST-ONLY (feature `los-fixture`, EEDCS-42): the run-wide fold of every line-of-sight query the campaign has made since boot
    /// or the last reset. Read-only.
    #[cfg(feature = "los-fixture")]
    pub fn los_trace(&self) -> Result<LosTrace, EngineError> {
        let mut raw = LosTrace::blank();
        // SAFETY: the out pointer is valid for the call.
        check(unsafe { sys::eech_engine_los_trace(&mut raw) })?;
        Ok(LosTrace::from(raw))
    }

    /// TEST-ONLY (feature `los-fixture`, EEDCS-42): clears the run-wide line-of-sight fold.
    #[cfg(feature = "los-fixture")]
    pub fn los_trace_reset(&self) -> Result<(), EngineError> {
        // SAFETY: no pointers cross.
        check(unsafe { sys::eech_engine_los_trace_reset() })
    }

    /// TEST-ONLY (feature `los-fixture`, EEDCS-42): runs the deterministic line-of-sight sweep of `eech_los_fixture.c` between two live
    /// entities through the campaign's own solver, and returns the fold of just that sweep; the run-wide fold is untouched.
    #[cfg(feature = "los-fixture")]
    #[allow(clippy::too_many_arguments)]
    pub fn los_probe<F, G, H, M>(
        &self,
        source_id: i32,
        target_id: i32,
        seed: u32,
        count: i32,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
    ) -> Result<LosTrace, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
    {
        let mut raw = LosTrace::blank();
        // SAFETY: the out pointer and the temporary host callback are valid for the call.
        with_host(bind_mobile_fn, observe_loss_fn, supply_reached_fn, move_vehicle_fn, |host| unsafe {
            sys::eech_engine_los_probe(source_id, target_id, seed, count, &mut raw, host)
        })?;
        Ok(LosTrace::from(raw))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The refusal of a step that must be refused (`expect_err` is denied in this crate).
    fn refusal<T, E>(result: Result<T, E>) -> E {
        match result {
            Err(error) => error,
            Ok(_) => panic!("the step must be refused"),
        }
    }

    /// The value of a step that must be accepted (`unwrap` is denied in this crate).
    fn accepted<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("the step must be accepted: {error:?}"),
        }
    }

    /// A `PLACE` step as the C side builds it for an aircraft (fixed-wing or helicopter): the campaign's motion vector
    /// is in the one placement field, `fw_motion_vector`.
    fn place_step(vector: [f32; 3]) -> sys::eech_movement_step {
        // SAFETY: the step is plain data; zero is a valid value for every field.
        let mut step: sys::eech_movement_step = unsafe { std::mem::zeroed() };
        step.mode = sys::EECH_MOVEMENT_PLACE;
        step.host_id[0] = b'h' as c_char;
        step.attitude[0] = 1.0;
        step.attitude[4] = 1.0;
        step.attitude[8] = 1.0;
        step.fw_motion_vector = vector;
        step
    }

    fn plan_query() -> HelicopterQuery {
        HelicopterQuery {
            structure_scan: true,
            structure_position: [1.0, 0.0, 2.0],
            terrain_sample_count: 2,
            terrain_sample: [[10.0, 20.0], [30.0, 40.0]],
        }
    }

    fn lookahead_query() -> HelicopterQuery {
        HelicopterQuery {
            structure_scan: false,
            structure_position: [0.0; 3],
            terrain_sample_count: 1,
            terrain_sample: [[7.0, 8.0], [0.0; 2]],
        }
    }

    /// the lookup plan the host answers `mode` with, when it answers one
    fn query_for(mode: MovementMode) -> Option<HelicopterQuery> {
        match mode {
            MovementMode::HelicopterPlan => Some(plan_query()),
            MovementMode::FixedWingLookahead => Some(lookahead_query()),
            _ => None,
        }
    }

    fn plan_guidance() -> HelicopterGuidanceAnswer {
        HelicopterGuidanceAnswer {
            lift: 9.5,
            waypoint: [5.0, 6.0, 7.0],
            required_height: 120.0,
            world_acceleration: [0.5, -0.25, 0.75],
            wp_vec: [1.0, 2.0, 3.0],
        }
    }

    /// The request the host receives for `step`, as it crosses the engine's `move_vehicle` callback.
    fn received(step: &sys::eech_movement_step) -> MovementStep {
        let mode = received_mode(step);
        let mut seen = None;
        let mut bind_mobile = |_: MemberDescriptor| -> Result<String, String> { Err("unused".into()) };
        let mut observe_loss = |_: LossDescriptor| -> Result<LossDecision, String> { Ok(LossDecision::Forward) };
        let mut supply_reached = |_: SupplyReachDescriptor| -> Result<ArrivalAnswer, String> { Err("unused".into()) };
        let mut host_move = |request: MovementStep| -> Result<MovementResult, String> {
            let pose = request.pose;
            seen = Some(request);
            Ok(MovementResult {
                pose,
                rotated: false,
                motion_vector: [0.0; 3],
                pending: false,
                moved: false,
                rotor: [0.0; 3],
                altitude: 0.0,
                flags: 0,
                angles: [0.0; 3],
                death_timer: 0.0,
                query: query_for(mode),
                guidance: (mode == MovementMode::HelicopterGuide).then(plan_guidance),
            })
        };
        let mut result: sys::eech_movement_result = unsafe { std::mem::zeroed() };
        let mut error = [0 as c_char; 256];
        // the production wiring: with_host builds the callback table the engine is given
        let outcome = with_host(&mut bind_mobile, &mut observe_loss, &mut supply_reached, &mut host_move, |host| {
            // SAFETY: the table and its user context are valid for the call; every other pointer is local.
            unsafe {
                let host = &*host;
                let Some(call) = host.move_vehicle else { return -1 };
                call(step, &mut result, error.as_mut_ptr(), error.len(), host.user)
            }
        });
        assert!(outcome.is_ok(), "the callback refused the step");
        let Some(request) = seen else { panic!("the host was not called") };
        request
    }

    fn received_mode(step: &sys::eech_movement_step) -> MovementMode {
        match step.mode {
            sys::EECH_MOVEMENT_HELICOPTER_PLAN => MovementMode::HelicopterPlan,
            sys::EECH_MOVEMENT_HELICOPTER_GUIDE => MovementMode::HelicopterGuide,
            sys::EECH_MOVEMENT_FIXED_WING_LOOKAHEAD => MovementMode::FixedWingLookahead,
            sys::EECH_MOVEMENT_DEPENDENT_CARRY => MovementMode::DependentCarry,
            _ => MovementMode::Place,
        }
    }

    /// What the callback does with `step` when the host answers with `answer`: the result C is given, or the refusal.
    fn crossing(step: &sys::eech_movement_step, answer: impl FnOnce(MovementStep) -> MovementResult) -> (Result<(), String>, sys::eech_movement_result) {
        let mut answer = Some(answer);
        let mut bind_mobile = |_: MemberDescriptor| -> Result<String, String> { Err("unused".into()) };
        let mut observe_loss = |_: LossDescriptor| -> Result<LossDecision, String> { Ok(LossDecision::Forward) };
        let mut supply_reached = |_: SupplyReachDescriptor| -> Result<ArrivalAnswer, String> { Err("unused".into()) };
        let mut host_move = |request: MovementStep| -> Result<MovementResult, String> {
            match answer.take() {
                Some(answer) => Ok(answer(request)),
                None => Err("the host was called twice".into()),
            }
        };
        // SAFETY: the result is plain data; zero is a valid value for every field.
        let mut result: sys::eech_movement_result = unsafe { std::mem::zeroed() };
        let mut error = [0 as c_char; 256];
        let outcome = with_host(&mut bind_mobile, &mut observe_loss, &mut supply_reached, &mut host_move, |host| {
            // SAFETY: the table and its user context are valid for the call; every other pointer is local.
            unsafe {
                let host = &*host;
                let Some(call) = host.move_vehicle else { return -1 };
                call(step, &mut result, error.as_mut_ptr(), error.len(), host.user)
            }
        });
        let text = error.iter().take_while(|byte| **byte != 0).map(|byte| *byte as u8 as char).collect::<String>();
        (outcome.map(|_| ()).map_err(|_| text), result)
    }

    fn answer_for(request: &MovementStep) -> MovementResult {
        MovementResult {
            pose: request.pose,
            rotated: false,
            motion_vector: [0.0; 3],
            pending: false,
            moved: false,
            rotor: [0.0; 3],
            altitude: 0.0,
            flags: 0,
            angles: [0.0; 3],
            death_timer: 0.0,
            query: query_for(request.mode),
            guidance: (request.mode == MovementMode::HelicopterGuide).then(plan_guidance),
        }
    }

    fn set_id(slot: &mut [c_char], id: &str) {
        for (index, byte) in id.bytes().enumerate() {
            slot[index] = byte as c_char;
        }
        slot[id.len()] = 0;
    }

    /// The storage behind a step's member lists, as the C side keeps it for the duration of the call.
    struct Group {
        ids: Vec<std::ffi::CString>,
        pointers: Vec<*const c_char>,
        radii: Vec<f32>,
    }

    impl Group {
        fn new(members: &[(&str, f32)]) -> Self {
            let ids: Vec<std::ffi::CString> = members.iter().map(|(id, _)| accepted(std::ffi::CString::new(*id))).collect();
            let pointers = ids.iter().map(|id| id.as_ptr()).collect();
            Self {
                ids,
                pointers,
                radii: members.iter().map(|(_, radius)| *radius).collect(),
            }
        }

        fn attach(&self, step: &mut sys::eech_movement_step) {
            step.fw_member_count = self.ids.len() as c_int;
            step.fw_member_host_id = self.pointers.as_ptr();
            step.fw_member_radius = self.radii.as_ptr();
        }
    }

    /// A normal-flight step as the C side builds it: a follower of "lead" that avoids the group's members.
    fn flight_step(group: &Group) -> sys::eech_movement_step {
        let mut step = place_step([0.0; 3]);
        step.mode = sys::EECH_MOVEMENT_FIXED_WING_MOVE;
        step.fw_phase = FIXED_WING_FLIGHT_PHASE;
        set_id(&mut step.fw_leader_host_id, "lead");
        group.attach(&mut step);
        step.fw_max_vertical_turn_allowance = 0.5;
        step.fw_avoidance_radius = 20.0;
        step
    }

    #[test]
    fn flight_navigation_intent_crosses_the_callback_boundary_in_group_order() {
        let group = Group::new(&[("a", 12.0), ("b", 30.0)]);
        let seen = received(&flight_step(&group));
        assert!(!seen.group_leader && !seen.task_leader);
        assert_eq!(seen.leader_host_id.as_deref(), Some("lead"));
        let members: Vec<(&str, f32)> = seen
            .members
            .iter()
            .map(|member| (member.host_id.as_deref().unwrap_or("?"), member.avoidance_radius))
            .collect();
        assert_eq!(members, [("a", 12.0), ("b", 30.0)]);
        assert_eq!((seen.max_vertical_turn_allowance, seen.avoidance_radius), (0.5, 20.0));
    }

    #[test]
    fn groups_beyond_thirty_two_cross_the_callback_boundary_whole_and_in_order() {
        for count in [32usize, 33, 40, 63, 1000] {
            let listed: Vec<(String, f32)> = (0..count).map(|index| (format!("member-{index}"), index as f32)).collect();
            let group = Group::new(&listed.iter().map(|(id, radius)| (id.as_str(), *radius)).collect::<Vec<_>>());
            let seen = received(&flight_step(&group));
            assert_eq!(seen.members.len(), count);
            for (index, member) in seen.members.iter().enumerate() {
                assert_eq!(
                    (member.host_id.as_deref(), member.avoidance_radius),
                    (Some(listed[index].0.as_str()), index as f32)
                );
            }
        }
    }

    #[test]
    fn the_landing_phase_carries_its_navigation_and_the_turn_intent_not_a_multiplier() {
        let group = Group::new(&[("a", 12.0)]);
        let mut step = flight_step(&group);
        step.fw_phase = FIXED_WING_LAND_PHASE;
        step.fw_aiming_cannon = 1;
        step.fw_elapsed_time = 123.5;
        let seen = received(&step);
        assert_eq!(seen.leader_host_id.as_deref(), Some("lead"));
        assert_eq!(seen.members.len(), 1);
        assert!(seen.aiming_cannon);
        assert_eq!(seen.elapsed_time, 123.5);
        // the other airborne phases and taxi send no navigation
        for phase in [0, 3] {
            let mut step = flight_step(&group);
            step.fw_phase = phase;
            let seen = received(&step);
            assert!(seen.leader_host_id.is_none() && seen.members.is_empty(), "phase {phase}");
        }
    }

    #[test]
    fn a_leader_is_not_asked_for_what_it_never_reads() {
        let mut step = flight_step(&Group::new(&[]));
        step.fw_group_leader = 1;
        step.fw_task_leader = 1;
        step.fw_leader_host_id[0] = 0;
        step.fw_member_count = -1;
        step.fw_member_host_id = std::ptr::null();
        step.fw_member_radius = std::ptr::null();
        let seen = received(&step);
        assert!(seen.group_leader && seen.task_leader);
        assert!(seen.leader_host_id.is_none() && seen.members.is_empty());
    }

    #[test]
    fn a_leader_the_roster_lacks_crosses_the_boundary_as_unresolved_in_flight_and_landing() {
        // whether the host reads the leader depends on what it finds (facing away, a hold in force), so the boundary
        // must not refuse a step for want of one
        let group = Group::new(&[("a", 12.0)]);
        for phase in [FIXED_WING_FLIGHT_PHASE, FIXED_WING_LAND_PHASE] {
            let mut step = flight_step(&group);
            step.fw_phase = phase;
            step.fw_leader_host_id[0] = 0;
            let seen = received(&step);
            assert!(!seen.task_leader && seen.leader_host_id.is_none(), "phase {phase}");
        }
    }

    #[test]
    fn a_member_the_roster_lacks_crosses_the_boundary_as_unresolved_and_keeps_its_place() {
        // an empty entry and a null entry are the same observation: no identity, the radius and the order still carried
        let group = Group::new(&[("a", 12.0), ("", 30.0), ("c", 7.0)]);
        let mut step = flight_step(&group);
        let mut pointers = group.pointers.clone();
        pointers[1] = std::ptr::null();
        step.fw_member_host_id = pointers.as_ptr();
        for step in [flight_step(&group), step] {
            let seen = received(&step);
            let members: Vec<(Option<&str>, f32)> = seen.members.iter().map(|member| (member.host_id.as_deref(), member.avoidance_radius)).collect();
            assert_eq!(members, [(Some("a"), 12.0), (None, 30.0), (Some("c"), 7.0)]);
        }
    }

    #[test]
    fn invalid_flight_navigation_is_refused_at_the_boundary() {
        let group = Group::new(&[("a", 12.0), ("b", 30.0)]);
        let refused = |change: &dyn Fn(&mut sys::eech_movement_step)| {
            let mut step = flight_step(&group);
            change(&mut step);
            // SAFETY: `group` outlives the call and holds the two members the step lists
            refusal(unsafe { navigation_from(&step) })
        };
        // a leader that is supplied but malformed is refused; one that is absent is the host's business, not the boundary's
        assert!(refused(&|s| s.fw_leader_host_id.fill(b'x' as c_char)).contains("task leader"));
        assert!(refused(&|s| s.fw_leader_host_id[0] = 0xffu8 as c_char).contains("task leader"));
        assert!(refused(&|s| s.fw_group_leader = 2).contains("fw_group_leader"));
        assert!(refused(&|s| s.fw_member_count = -1).contains("group member count"));
        assert!(refused(&|s| s.fw_member_host_id = std::ptr::null()).contains("without their identity"));
        assert!(refused(&|s| s.fw_member_radius = std::ptr::null()).contains("without their identity"));
        let mut step;
        let too_long = Group::new(&[("a", 12.0), (&"x".repeat(sys::EECH_MOVEMENT_HOST_ID_MAX + 1), 30.0)]);
        step = flight_step(&too_long);
        assert!(refusal(unsafe { navigation_from(&step) }).contains("group member"));
        let longest = Group::new(&[("a", 12.0), (&"x".repeat(sys::EECH_MOVEMENT_HOST_ID_MAX), 30.0)]);
        step = flight_step(&longest);
        assert_eq!(
            accepted(unsafe { navigation_from(&step) }).3[1].host_id.as_ref().map(String::len),
            Some(sys::EECH_MOVEMENT_HOST_ID_MAX)
        );
        let mut pointers = group.pointers.clone();
        pointers[1] = std::ptr::null();
        step = flight_step(&group);
        step.fw_member_host_id = pointers.as_ptr();
        assert!(accepted(unsafe { navigation_from(&step) }).3[1].host_id.is_none(), "a null entry is unresolved");
        let bad_utf8 = [0xffu8 as c_char, 0];
        pointers[1] = bad_utf8.as_ptr();
        assert!(refusal(unsafe { navigation_from(&step) }).contains("non-UTF-8"));
        let nan = Group::new(&[("a", 12.0), ("b", f32::NAN)]);
        step = flight_step(&nan);
        assert!(refusal(unsafe { navigation_from(&step) }).contains("fw_member_radius"));
    }

    #[test]
    fn an_aircraft_placement_carries_its_nonzero_vector_across_the_callback_boundary() {
        // helicopter and fixed-wing placements are the same step on the wire: neither may be reset to zero
        let vector = [3.0, -2.0, 90.0];
        let request = received(&place_step(vector));
        assert_eq!(request.mode, MovementMode::Place);
        assert_eq!(request.motion_vector_in, vector);
    }

    #[test]
    fn a_placement_reads_the_placement_field_not_the_helicopter_step_field() {
        let mut step = place_step([3.0, -2.0, 90.0]);
        step.hc_motion_vector = [7.0, 7.0, 7.0];
        assert_eq!(received(&step).motion_vector_in, [3.0, -2.0, 90.0]);
    }

    #[test]
    fn the_helicopter_step_modes_read_the_helicopter_vector_not_the_placement_field() {
        for mode in [sys::EECH_MOVEMENT_HELICOPTER_BEGIN, sys::EECH_MOVEMENT_HELICOPTER_END] {
            let mut step = place_step([9.0, 9.0, 9.0]);
            step.mode = mode;
            step.hc_motion_vector = [1.0, 2.0, 3.0];
            step.hc_cg_to_ground = 1.5;
            step.hc_desired_heading = 0.5;
            step.hc_max_turn_rate = 0.25;
            let request = received(&step);
            assert_eq!(request.motion_vector_in, [1.0, 2.0, 3.0]);
            assert_eq!(request.cg_to_ground, 1.5);
            assert_eq!((request.desired_heading, request.max_turn_rate), (0.5, 0.25));
        }
    }

    #[test]
    fn every_lifecycle_mode_reads_the_helicopter_fields_and_carries_its_inputs() {
        let modes = [
            (sys::EECH_MOVEMENT_ROTOR_SET, MovementMode::RotorSet),
            (sys::EECH_MOVEMENT_ROTOR_ALIVE, MovementMode::RotorAlive),
            (sys::EECH_MOVEMENT_ROTOR_DOWN, MovementMode::RotorDown),
            (sys::EECH_MOVEMENT_FALL, MovementMode::Fall),
            (sys::EECH_MOVEMENT_CONTACT, MovementMode::Contact),
            (sys::EECH_MOVEMENT_SETTLE, MovementMode::Settle),
            (sys::EECH_MOVEMENT_TUMBLE, MovementMode::Tumble),
            (sys::EECH_MOVEMENT_CRASH_BEGIN, MovementMode::CrashBegin),
            (sys::EECH_MOVEMENT_CRASH_END, MovementMode::CrashEnd),
            (sys::EECH_MOVEMENT_IMPACT, MovementMode::Impact),
            (sys::EECH_MOVEMENT_FALL_GRAVITY, MovementMode::FallGravity),
            (sys::EECH_MOVEMENT_ROTOR_IDLE, MovementMode::RotorIdle),
            (sys::EECH_MOVEMENT_GV_DEATH_CANDIDATE, MovementMode::GvDeathCandidate),
            (sys::EECH_MOVEMENT_GV_DEATH_DECAY, MovementMode::GvDeathDecay),
            (sys::EECH_MOVEMENT_GV_FALL_CANDIDATE, MovementMode::GvFallCandidate),
            (sys::EECH_MOVEMENT_GV_FALL_GRAVITY, MovementMode::GvFallGravity),
            (sys::EECH_MOVEMENT_GV_FALL_SETTLE, MovementMode::GvFallSettle),
            (sys::EECH_MOVEMENT_GV_IMPACT, MovementMode::GvImpact),
            (sys::EECH_MOVEMENT_SHIP_SINK, MovementMode::ShipSink),
            (sys::EECH_MOVEMENT_VEHICLE_VECTOR_SEED, MovementMode::VehicleVectorSeed),
            (sys::EECH_MOVEMENT_FW_DEATH_ATTITUDE, MovementMode::FwDeathAttitude),
            (sys::EECH_MOVEMENT_FW_CRASH_CANDIDATE, MovementMode::FwCrashCandidate),
            (sys::EECH_MOVEMENT_FW_CRASH_VELOCITY, MovementMode::FwCrashVelocity),
            (sys::EECH_MOVEMENT_GV_FALL_VECTOR, MovementMode::GvFallVector),
            (sys::EECH_MOVEMENT_GV_FALL_COMMIT, MovementMode::GvFallCommit),
            (sys::EECH_MOVEMENT_FW_SETTLE, MovementMode::FwSettle),
            (sys::EECH_MOVEMENT_FW_CRASH_ATTITUDE, MovementMode::FwCrashAttitude),
        ];
        for (mode, expected) in modes {
            let mut step = place_step([9.0, 9.0, 9.0]);
            step.mode = mode;
            step.hc_motion_vector = [1.0, 2.0, 3.0];
            step.hc_rotor = [90.0, 80.0, 4.0];
            step.hc_lift = 0.25;
            step.hc_power_output = 0.75;
            step.hc_flag = 1;
            step.hc_gravity = 9.5;
            step.hc_terrain_class = 2;
            step.hc_bury_limit = 3.5;
            step.hc_draws = [1.0 / 32767.0, 0.5];
            step.hc_pitch_offset = 0.1;
            step.hc_bank_offset = -0.2;
            step.hc_signed_speed = -6.0;
            step.hc_cg_to_ground = 1.5;
            step.forest = 1;
            step.face_normal = [0.0, 0.5, 0.5];
            let request = received(&step);
            assert_eq!(request.mode, expected);
            assert_eq!(request.motion_vector_in, [1.0, 2.0, 3.0]);
            assert_eq!(request.rotor, [90.0, 80.0, 4.0]);
            assert_eq!((request.lift, request.power_output, request.flag), (0.25, 0.75, true));
            assert_eq!((request.gravity, request.terrain_class, request.bury_limit), (9.5, 2, 3.5));
            // double draws, not representable as float: they arrive bit for bit
            assert_eq!(request.draws, [1.0 / 32767.0, 0.5]);
            assert_eq!((request.pitch_offset, request.bank_offset, request.signed_speed), (0.1, -0.2, -6.0));
            assert_eq!(request.cg_to_ground, 1.5);
            assert!(request.forest);
            assert_eq!(request.face_normal, [0.0, 0.5, 0.5]);
            assert!(request.terrain.is_none(), "a lifecycle step announces no routed terrain sample");
        }
    }

    #[test]
    fn a_retire_step_crosses_the_callback_with_its_identity_and_no_terrain() {
        let mut step = place_step([0.0; 3]);
        step.mode = sys::EECH_MOVEMENT_RETIRE;
        let request = received(&step);
        assert_eq!(request.mode, MovementMode::Retire);
        assert_eq!(request.host_id, "h");
        assert!(request.terrain.is_none());
        assert!(result_flags_are_valid(MovementMode::Retire, 0) && !result_flags_are_valid(MovementMode::Retire, 1));
    }

    #[test]
    fn a_create_place_step_carries_the_constructor_inputs_across_the_callback() {
        let mut step = place_step([4.0, -1.0, 80.0]);
        step.mode = sys::EECH_MOVEMENT_CREATE_PLACE;
        step.position = [10.0, 20.0, 30.0];
        step.fw_cg_to_ground = 1.75;
        step.hc_desired_heading = 1.25;
        step.fw_fuselage_angle = 0.1;
        step.face_normal = [0.0, 0.8, 0.6];
        let request = received(&step);
        assert_eq!(request.mode, MovementMode::CreatePlace);
        assert_eq!(request.pose.position, [10.0, 20.0, 30.0]);
        // the physical offset is an input fact the host adds; it is not folded into the position
        assert_eq!(request.cg_to_ground, 1.75);
        assert_eq!((request.desired_heading, request.fuselage_angle), (1.25, 0.1));
        assert_eq!(request.face_normal, [0.0, 0.8, 0.6]);
        // the aircraft vector is the placement field, as for Place
        assert_eq!(request.motion_vector_in, [4.0, -1.0, 80.0]);
        assert!(request.terrain.is_none());
        assert!(result_flags_are_valid(MovementMode::CreatePlace, 0) && !result_flags_are_valid(MovementMode::CreatePlace, 1));
    }

    #[test]
    fn a_dependent_carry_step_carries_the_parent_and_both_of_its_positions_across_the_callback() {
        let mut step = place_step([0.0; 3]);
        step.mode = sys::EECH_MOVEMENT_DEPENDENT_CARRY;
        for (slot, byte) in step.dc_parent_host_id.iter_mut().zip(b"parent") {
            *slot = *byte as c_char;
        }
        step.dc_helicopter = 1;
        step.dc_parent_old = [1.0, 2.0, 3.0];
        step.dc_parent_new = [4.0, 5.0, 6.0];
        step.dc_heading = 0.75;
        step.dc_velocity = 12.5;
        let request = received(&step);
        assert_eq!(request.mode, MovementMode::DependentCarry);
        assert_eq!(request.dc_parent.as_deref(), Some("parent"));
        assert!(request.dc_helicopter);
        assert_eq!((request.dc_parent_old, request.dc_parent_new), ([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]));
        assert_eq!((request.dc_heading, request.dc_velocity), (0.75, 12.5));
        assert!(request.terrain.is_none());
        assert!(result_flags_are_valid(MovementMode::DependentCarry, 0) && !result_flags_are_valid(MovementMode::DependentCarry, 1));
    }

    #[test]
    fn a_dependent_carry_step_without_a_parent_or_with_a_non_finite_position_is_refused() {
        let mut step = place_step([0.0; 3]);
        step.mode = sys::EECH_MOVEMENT_DEPENDENT_CARRY;
        let (outcome, _) = crossing(&step, |_| panic!("the host must not be asked"));
        assert!(outcome.is_err(), "a carry names its parent");
        step.dc_parent_host_id[0] = b'p' as c_char;
        step.dc_parent_new = [f32::NAN, 0.0, 0.0];
        let (outcome, _) = crossing(&step, |_| panic!("the host must not be asked"));
        assert!(outcome.is_err(), "a non-finite parent position never reaches the host");
    }

    #[test]
    fn an_expire_step_carries_the_wreck_timer_range_and_policy_and_answers_only_a_yes_or_no() {
        let mut step = place_step([0.0; 3]);
        step.mode = sys::EECH_MOVEMENT_EXPIRE;
        step.delta_seconds = 0.25;
        step.death_timer = 598.5;
        step.death_range = 5.0e7;
        step.death_policy = 2;
        step.gv_bbox_diagonal = 12.0;
        let request = received(&step);
        assert_eq!(request.mode, MovementMode::Expire);
        assert_eq!(
            (request.delta_seconds, request.death_timer, request.death_range, request.death_policy),
            (0.25, 598.5, 5.0e7, 2)
        );
        assert_eq!(request.bbox_diagonal, 12.0);
        assert!(request.terrain.is_none());
        assert!(result_flags_are_valid(MovementMode::Expire, 0) && result_flags_are_valid(MovementMode::Expire, 1));
        // 2: the gate fell and the host asks for the keysite range
        assert!(result_flags_are_valid(MovementMode::Expire, 2));
        assert!(!result_flags_are_valid(MovementMode::Expire, 3));
    }

    #[test]
    fn each_mode_answers_only_the_flags_it_defines() {
        let accepted = |mode, flags: &[i32]| {
            for flags in flags {
                assert!(result_flags_are_valid(mode, *flags), "{mode:?} {flags}");
            }
            for flags in -2..=10 {
                assert_eq!(result_flags_are_valid(mode, flags), flags_in(flags, flags_of(mode)), "{mode:?} {flags}");
            }
        };
        fn flags_of(mode: MovementMode) -> &'static [i32] {
            match mode {
                MovementMode::GvFallSettle => &[0, 1, 3, 5, 7, 9],
                MovementMode::FwDeathAttitude => &[-1, 0, 1, 3],
                MovementMode::FwCrashVelocity | MovementMode::FwCrashAttitude => &[0, 1],
                MovementMode::Expire => &[0, 1, 2],
                _ => &[0],
            }
        }
        fn flags_in(flags: i32, set: &[i32]) -> bool {
            set.contains(&flags)
        }
        accepted(MovementMode::GvFallSettle, &[0, 1, 3, 5, 7, 9]);
        accepted(MovementMode::FwDeathAttitude, &[-1, 0, 1, 3]);
        accepted(MovementMode::FwCrashVelocity, &[0, 1]);
        accepted(MovementMode::FwCrashAttitude, &[0, 1]);
        accepted(MovementMode::Expire, &[0, 1, 2]);
        accepted(MovementMode::GvDeathDecay, &[0]);
        accepted(MovementMode::FwSettle, &[0]);
        // the ground impact and a splash that is not also a contact are not answers
        assert!(!result_flags_are_valid(MovementMode::GvFallSettle, 8) && !result_flags_are_valid(MovementMode::GvFallSettle, 2));
    }

    #[test]
    fn a_non_finite_lifecycle_field_or_a_bad_flag_is_refused_before_the_host_runs() {
        let refuses = |edit: &dyn Fn(&mut sys::eech_movement_step)| {
            let mut step = place_step([0.0; 3]);
            step.mode = sys::EECH_MOVEMENT_ROTOR_ALIVE;
            edit(&mut step);
            let mut called = false;
            let mut bind_mobile = |_: MemberDescriptor| -> Result<String, String> { Err("unused".into()) };
            let mut observe_loss = |_: LossDescriptor| -> Result<LossDecision, String> { Ok(LossDecision::Forward) };
            let mut supply_reached = |_: SupplyReachDescriptor| -> Result<ArrivalAnswer, String> { Err("unused".into()) };
            let mut host_move = |_: MovementStep| -> Result<MovementResult, String> {
                called = true;
                Err("must not run".into())
            };
            let mut result: sys::eech_movement_result = unsafe { std::mem::zeroed() };
            let mut error = [0 as c_char; 256];
            let outcome = with_host(&mut bind_mobile, &mut observe_loss, &mut supply_reached, &mut host_move, |host| unsafe {
                let host = &*host;
                let Some(call) = host.move_vehicle else { return -1 };
                call(&step, &mut result, error.as_mut_ptr(), error.len(), host.user)
            });
            assert!(outcome.is_err());
            assert!(!called);
        };
        refuses(&|step| step.hc_rotor[1] = f32::NAN);
        refuses(&|step| step.hc_lift = f32::INFINITY);
        refuses(&|step| step.hc_draws[0] = f64::NAN);
        refuses(&|step| step.hc_flag = 2);
        // a wreck timer or range that is not a number never reaches the host
        refuses(&|step| {
            step.mode = sys::EECH_MOVEMENT_EXPIRE;
            step.death_timer = f32::NAN;
        });
        refuses(&|step| {
            step.mode = sys::EECH_MOVEMENT_EXPIRE;
            step.death_range = f32::INFINITY;
        });
    }

    /// A hosted helicopter step of `mode` with the aircraft's identity and a unit attitude.
    fn hosted_step(mode: c_int) -> sys::eech_movement_step {
        let mut step = place_step([0.0; 3]);
        step.mode = mode;
        step
    }

    #[test]
    fn a_plan_step_carries_the_campaign_facts_and_the_other_members_host_ids_in_list_order_uncapped() {
        let mut step = hosted_step(sys::EECH_MOVEMENT_HELICOPTER_PLAN);
        step.hc_task_leader = 1;
        step.hc_waypoint_kind = 2;
        step.hc_guide_radius_valid = 1;
        step.hc_guide_radius = 750.0;
        step.hc_guide_altitude_valid = 1;
        step.hc_waypoint_velocity = [1.0, 2.0, 3.0];
        step.hc_distance = 4000.0;
        step.hc_fuel_supply_level = 0.5;
        step.hc_cruise_velocity = 60.0;
        step.hc_cruise_altitude = 90.0;
        step.hc_attack_altitude = 30.0;
        step.hc_leader_radar_altitude = 45.0;
        // many more members than any fixed buffer would hold, one without an identity
        let names: Vec<std::ffi::CString> = (0..300).map(|index| accepted(std::ffi::CString::new(format!("m{index}")))).collect();
        let mut ids: Vec<*const c_char> = names.iter().map(|name| name.as_ptr()).collect();
        ids[7] = std::ptr::null();
        step.hc_member_count = ids.len() as c_int;
        step.hc_member_host_id = ids.as_ptr();
        // the lists of the other modes are ignored here
        step.hc_structure_count = 5;
        let (outcome, result) = crossing(&step, |request| {
            assert_eq!(request.mode, MovementMode::HelicopterPlan);
            assert!(request.hc_task_leader && !request.hc_group_leader && !request.hc_position_hold);
            assert_eq!(request.hc_waypoint_kind, 2);
            assert_eq!(request.hc_guide_radius, Some(750.0));
            assert!(request.hc_guide_altitude_valid);
            assert_eq!(request.hc_waypoint_velocity, [1.0, 2.0, 3.0]);
            assert_eq!(
                (request.hc_distance, request.hc_fuel_supply_level, request.hc_cruise_velocity),
                (4000.0, 0.5, 60.0)
            );
            assert_eq!(
                (request.hc_cruise_altitude, request.hc_attack_altitude, request.hc_leader_radar_altitude),
                (90.0, 30.0, 45.0)
            );
            assert_eq!(request.hc_plan_members.len(), 300);
            assert_eq!(request.hc_plan_members[0].as_deref(), Some("m0"));
            assert_eq!(request.hc_plan_members[7], None);
            assert_eq!(request.hc_plan_members[299].as_deref(), Some("m299"));
            assert!(request.hc_structures.is_empty() && request.hc_terrain_samples.is_empty());
            answer_for(&request)
        });
        assert_eq!(outcome, Ok(()));
        // the host's lookup plan reaches C unchanged
        assert_eq!(result.hc_query.structure_scan, 1);
        assert_eq!(result.hc_query.structure_position, [1.0, 0.0, 2.0]);
        assert_eq!(result.hc_query.terrain_sample_count, 2);
        assert_eq!(result.hc_query.terrain_sample, [[10.0, 20.0], [30.0, 40.0]]);
        assert_eq!(result.hc_guidance.lift, 0.0);
    }

    #[test]
    fn a_group_leaders_plan_does_not_read_the_member_list_and_an_invalid_list_is_refused_before_the_host_runs() {
        let mut leader = hosted_step(sys::EECH_MOVEMENT_HELICOPTER_PLAN);
        leader.hc_group_leader = 1;
        leader.hc_member_count = -5;
        let (outcome, _) = crossing(&leader, |request| {
            assert!(request.hc_group_leader && request.hc_plan_members.is_empty());
            answer_for(&request)
        });
        assert_eq!(outcome, Ok(()));

        let refuses = |edit: &dyn Fn(&mut sys::eech_movement_step), expected: &str| {
            let mut step = hosted_step(sys::EECH_MOVEMENT_HELICOPTER_PLAN);
            edit(&mut step);
            let mut called = false;
            let (outcome, _) = crossing(&step, |request| {
                called = true;
                answer_for(&request)
            });
            let message = refusal(outcome);
            assert!(message.contains(expected), "{message}");
            assert!(!called, "the host ran");
        };
        refuses(&|step| step.hc_member_count = -1, "invalid group member count");
        refuses(&|step| step.hc_member_count = 3, "without their identity list");
        refuses(&|step| step.hc_group_leader = 2, "invalid hc_group_leader flag");
        refuses(&|step| step.hc_guide_radius_valid = 7, "invalid hc_guide_radius_valid flag");
        refuses(&|step| step.hc_distance = f32::NAN, "hc_distance");
        refuses(&|step| step.hc_leader_radar_altitude = f32::INFINITY, "hc_leader_radar_altitude");
        refuses(&|step| step.hc_waypoint_velocity[2] = f32::NAN, "hc_waypoint_velocity");
    }

    #[test]
    fn a_guide_step_carries_every_structure_of_the_sector_and_the_getter_returns_in_call_order() {
        let structures: Vec<sys::eech_hc_structure> = (0..1000)
            .map(|index| sys::eech_hc_structure {
                position: [index as f32, 0.0, -(index as f32)],
                avoidance_radius: 5.0 + index as f32,
                height: 10.0,
            })
            .collect();
        let samples = [
            sys::eech_hc_terrain_sample {
                first: 12.0,
                second: 0.0,
                second_present: 0,
            },
            sys::eech_hc_terrain_sample {
                first: 3.0,
                second: 14.5,
                second_present: 1,
            },
        ];
        let mut step = hosted_step(sys::EECH_MOVEMENT_HELICOPTER_GUIDE);
        step.hc_structure_count = structures.len() as c_int;
        step.hc_structures = structures.as_ptr();
        step.hc_terrain_sample_count = samples.len() as c_int;
        step.hc_terrain_samples = samples.as_ptr();
        // the member list of the plan is not read in this mode
        step.hc_member_count = -1;
        let (outcome, result) = crossing(&step, |request| {
            assert_eq!(request.mode, MovementMode::HelicopterGuide);
            assert_eq!(request.hc_structures.len(), 1000);
            assert_eq!(request.hc_structures[999].position, [999.0, 0.0, -999.0]);
            assert_eq!(request.hc_structures[999].avoidance_radius, 1004.0);
            assert_eq!(request.hc_structures[3].height, 10.0);
            // the records are not folded here: the second return is kept only where the macro made the second call
            assert_eq!(request.hc_terrain_samples.len(), 2);
            assert_eq!((request.hc_terrain_samples[0].first, request.hc_terrain_samples[0].second), (12.0, None));
            assert_eq!((request.hc_terrain_samples[1].first, request.hc_terrain_samples[1].second), (3.0, Some(14.5)));
            answer_for(&request)
        });
        assert_eq!(outcome, Ok(()));
        let guidance = plan_guidance();
        assert_eq!(result.hc_guidance.lift, guidance.lift);
        assert_eq!(result.hc_guidance.waypoint, guidance.waypoint);
        assert_eq!(result.hc_guidance.required_height, guidance.required_height);
        assert_eq!(result.hc_guidance.world_acceleration, guidance.world_acceleration);
        assert_eq!(result.hc_guidance.wp_vec, guidance.wp_vec);
        // no lookup plan outside the plan step
        assert_eq!(result.hc_query.structure_scan, 0);
        assert_eq!(result.hc_query.terrain_sample_count, 0);
    }

    #[test]
    fn a_guide_step_with_bad_lists_is_refused_before_the_host_runs() {
        let structure = sys::eech_hc_structure {
            position: [0.0; 3],
            avoidance_radius: 1.0,
            height: 1.0,
        };
        let sample = sys::eech_hc_terrain_sample {
            first: 1.0,
            second: 2.0,
            second_present: 1,
        };
        let refuses = |edit: &dyn Fn(&mut sys::eech_movement_step), expected: &str| {
            let mut step = hosted_step(sys::EECH_MOVEMENT_HELICOPTER_GUIDE);
            step.hc_structures = &structure;
            step.hc_terrain_samples = &sample;
            edit(&mut step);
            let mut called = false;
            let (outcome, _) = crossing(&step, |request| {
                called = true;
                answer_for(&request)
            });
            let message = refusal(outcome);
            assert!(message.contains(expected), "{message}");
            assert!(!called, "the host ran");
        };
        refuses(&|step| step.hc_structure_count = -2, "invalid structure count");
        refuses(
            &|step| {
                step.hc_structure_count = 1;
                step.hc_structures = std::ptr::null();
            },
            "without their list",
        );
        refuses(&|step| step.hc_terrain_sample_count = -1, "invalid terrain sample count");
        refuses(
            &|step| {
                step.hc_terrain_sample_count = 1;
                step.hc_terrain_samples = std::ptr::null();
            },
            "without their list",
        );
        let nan_structure = sys::eech_hc_structure {
            position: [0.0, f32::NAN, 0.0],
            ..structure
        };
        refuses(
            &|step| {
                step.hc_structure_count = 1;
                step.hc_structures = &nan_structure;
            },
            "non-finite movement hc_structures",
        );
        let nan_height = sys::eech_hc_structure {
            height: f32::INFINITY,
            ..structure
        };
        refuses(
            &|step| {
                step.hc_structure_count = 1;
                step.hc_structures = &nan_height;
            },
            "non-finite movement hc_structures",
        );
        let nan_sample = sys::eech_hc_terrain_sample { first: f32::NAN, ..sample };
        refuses(
            &|step| {
                step.hc_terrain_sample_count = 1;
                step.hc_terrain_samples = &nan_sample;
            },
            "non-finite movement hc_terrain_samples",
        );
        // a second return that is not a number matters only where it was recorded
        let nan_second = sys::eech_hc_terrain_sample { second: f32::NAN, ..sample };
        refuses(
            &|step| {
                step.hc_terrain_sample_count = 1;
                step.hc_terrain_samples = &nan_second;
            },
            "non-finite movement hc_terrain_samples",
        );
        let bad_flag = sys::eech_hc_terrain_sample { second_present: 2, ..sample };
        refuses(
            &|step| {
                step.hc_terrain_sample_count = 1;
                step.hc_terrain_samples = &bad_flag;
            },
            "second_present",
        );
        // an unrecorded second return is not read, whatever the field holds
        let ignored = sys::eech_hc_terrain_sample {
            first: 1.0,
            second: f32::NAN,
            second_present: 0,
        };
        let mut step = hosted_step(sys::EECH_MOVEMENT_HELICOPTER_GUIDE);
        step.hc_terrain_sample_count = 1;
        step.hc_terrain_samples = &ignored;
        let (outcome, _) = crossing(&step, |request| {
            assert_eq!(request.hc_terrain_samples[0].second, None);
            answer_for(&request)
        });
        assert_eq!(outcome, Ok(()));
    }

    #[test]
    fn a_guided_end_step_names_the_leader_by_host_id_and_carries_the_steering_intent() {
        let mut step = hosted_step(sys::EECH_MOVEMENT_HELICOPTER_GUIDED_END);
        set_id(&mut step.hc_leader_host_id, "leader-1");
        step.hc_waypoint_kind = 1;
        step.hc_task_leader_is_self = 0;
        step.hc_guide_radius_valid = 1;
        step.hc_guide_radius = 300.0;
        step.hc_distance = 120.0;
        step.hc_cruise_velocity = 55.0;
        step.hc_required_heading_valid = 1;
        step.hc_required_heading = 1.25;
        step.hc_targeting_active = 1;
        step.hc_weapon_vector = [0.0, 0.0, 1.0];
        step.hc_weapon_to_intercept_vector = [0.1, 0.2, 0.9];
        step.hc_main_rotor_shaft_angle = 0.05;
        step.elevation = 33.0;
        let (outcome, _) = crossing(&step, |request| {
            assert_eq!(request.mode, MovementMode::HelicopterGuidedEnd);
            assert_eq!(request.hc_leader_host_id.as_deref(), Some("leader-1"));
            assert_eq!(request.hc_required_heading, Some(1.25));
            assert_eq!(request.hc_guide_radius, Some(300.0));
            assert!(request.hc_targeting_active && !request.hc_task_leader_is_self);
            assert_eq!(request.hc_weapon_to_intercept_vector, [0.1, 0.2, 0.9]);
            assert_eq!(request.hc_main_rotor_shaft_angle, 0.05);
            // the cached terrain elevation announced by the DLL, not a new sample
            assert_eq!(request.elevation, 33.0);
            let mut answer = answer_for(&request);
            answer.angles = [0.5, -0.1, 0.25];
            answer
        });
        assert_eq!(outcome, Ok(()));

        // no leader and no required heading are legitimate: the host refuses only where it reads them
        let mut alone = hosted_step(sys::EECH_MOVEMENT_HELICOPTER_GUIDED_END);
        alone.hc_required_heading = 7.0;
        let (outcome, _) = crossing(&alone, |request| {
            assert_eq!((request.hc_leader_host_id.as_deref(), request.hc_required_heading), (None, None));
            answer_for(&request)
        });
        assert_eq!(outcome, Ok(()));
        let mut not_finite = hosted_step(sys::EECH_MOVEMENT_HELICOPTER_GUIDED_END);
        not_finite.hc_required_heading_valid = 1;
        not_finite.hc_required_heading = f32::NAN;
        assert!(crossing(&not_finite, |request| answer_for(&request)).0.is_err());
    }

    #[test]
    fn the_host_must_answer_the_plan_with_a_lookup_plan_and_the_guide_with_guidance_and_nothing_else_does() {
        let plan = hosted_step(sys::EECH_MOVEMENT_HELICOPTER_PLAN);
        let missing = crossing(&plan, |request| MovementResult {
            query: None,
            ..answer_for(&request)
        })
        .0;
        assert!(refusal(missing).contains("did not return a lookup plan"));
        let too_many = crossing(&plan, |request| {
            let mut answer = answer_for(&request);
            if let Some(query) = answer.query.as_mut() {
                query.terrain_sample_count = sys::EECH_HC_TERRAIN_SAMPLES + 1;
            }
            answer
        })
        .0;
        assert!(refusal(too_many).contains("terrain samples"));
        let not_a_number = crossing(&plan, |request| {
            let mut answer = answer_for(&request);
            if let Some(query) = answer.query.as_mut() {
                query.terrain_sample[1][0] = f32::NAN;
            }
            answer
        })
        .0;
        assert!(not_a_number.is_err());

        let guide = hosted_step(sys::EECH_MOVEMENT_HELICOPTER_GUIDE);
        let missing = crossing(&guide, |request| MovementResult {
            guidance: None,
            ..answer_for(&request)
        })
        .0;
        assert!(refusal(missing).contains("did not return guidance"));
        let not_a_number = crossing(&guide, |request| {
            let mut answer = answer_for(&request);
            if let Some(guidance) = answer.guidance.as_mut() {
                guidance.wp_vec[1] = f32::INFINITY;
            }
            answer
        })
        .0;
        assert!(not_a_number.is_err());

        // a lookup plan or guidance on any other mode is a host that mixed its answers up
        for mode in [
            sys::EECH_MOVEMENT_HELICOPTER_GUIDE,
            sys::EECH_MOVEMENT_HELICOPTER_GUIDED_BEGIN,
            sys::EECH_MOVEMENT_HELICOPTER_GUIDED_END,
            sys::EECH_MOVEMENT_HELICOPTER_ABANDON,
        ] {
            let step = hosted_step(mode);
            let stray = crossing(&step, |request| MovementResult {
                query: Some(plan_query()),
                ..answer_for(&request)
            })
            .0;
            assert!(refusal(stray).contains("a lookup plan"), "mode {mode}");
        }
        for mode in [
            sys::EECH_MOVEMENT_HELICOPTER_PLAN,
            sys::EECH_MOVEMENT_HELICOPTER_GUIDED_BEGIN,
            sys::EECH_MOVEMENT_HELICOPTER_GUIDED_END,
            sys::EECH_MOVEMENT_HELICOPTER_ABANDON,
        ] {
            let step = hosted_step(mode);
            let stray = crossing(&step, |request| MovementResult {
                guidance: Some(plan_guidance()),
                ..answer_for(&request)
            })
            .0;
            assert!(refusal(stray).contains("guidance"), "mode {mode}");
        }
        // the hosted modes answer no flags
        for mode in [
            MovementMode::HelicopterPlan,
            MovementMode::HelicopterGuide,
            MovementMode::HelicopterGuidedBegin,
            MovementMode::HelicopterGuidedEnd,
            MovementMode::HelicopterAbandon,
        ] {
            assert!(result_flags_are_valid(mode, 0) && !result_flags_are_valid(mode, 1) && !result_flags_are_valid(mode, -1));
        }
    }

    #[test]
    fn the_hosted_helicopter_modes_are_helicopter_steps_and_a_plan_and_the_end_announce_the_terrain() {
        for mode in [
            sys::EECH_MOVEMENT_HELICOPTER_PLAN,
            sys::EECH_MOVEMENT_HELICOPTER_GUIDE,
            sys::EECH_MOVEMENT_HELICOPTER_GUIDED_BEGIN,
            sys::EECH_MOVEMENT_HELICOPTER_GUIDED_END,
            sys::EECH_MOVEMENT_HELICOPTER_ABANDON,
        ] {
            let mut step = hosted_step(mode);
            step.hc_motion_vector = [1.0, 2.0, 3.0];
            step.hc_power_output = 0.75;
            // the plan and the end announce their terrain sample; for the other modes a sample without terrain-follow is refused
            let announces = mode == sys::EECH_MOVEMENT_HELICOPTER_PLAN || mode == sys::EECH_MOVEMENT_HELICOPTER_GUIDED_END;
            step.terrain_present = announces as c_int;
            let (outcome, _) = crossing(&step, |request| {
                assert_eq!(request.terrain.is_some(), announces, "{:?}", request.mode);
                assert_eq!(request.motion_vector_in, [1.0, 2.0, 3.0], "{:?}", request.mode);
                answer_for(&request)
            });
            assert_eq!(outcome, Ok(()), "mode {mode}");
        }
    }

    /// a terrain-following flight step: the campaign's observation of the waypoint, as the C side fills it
    fn terrained_flight_step(group: &Group) -> sys::eech_movement_step {
        let mut step = flight_step(group);
        step.fw_terrain = 1;
        step.fw_terrain_min_height = 30.0;
        step.fw_terrain_guide_position = [1.0, 2.0, 3.0];
        step.fw_terrain_sample_point = [4.0, 5.0];
        step.fw_terrain_sample_elevation = 6.0;
        step.fw_terrain_site_y = 0.25;
        step.fw_terrain_cg = 1.5;
        step
    }

    #[test]
    fn a_terrain_following_flight_step_carries_the_campaigns_observation_and_no_shaped_height() {
        let group = Group::new(&[("a", 12.0)]);
        let step = terrained_flight_step(&group);
        let (outcome, _) = crossing(&step, |request| {
            assert_eq!(request.mode, MovementMode::FixedWingMove);
            assert_eq!(
                request.fixed_wing_terrain,
                Some(FixedWingTerrain {
                    min_height: 30.0,
                    guide_position: [1.0, 2.0, 3.0],
                    sample_point: [4.0, 5.0],
                    sample_elevation: 6.0,
                    site_y: 0.25,
                    cg: 1.5,
                })
            );
            answer_for(&request)
        });
        assert_eq!(outcome, Ok(()));
    }

    #[test]
    fn a_step_without_the_terrain_flag_carries_none_and_its_fields_are_not_read() {
        let group = Group::new(&[("a", 12.0)]);
        let mut step = terrained_flight_step(&group);
        step.fw_terrain = 0;
        step.fw_terrain_cg = f32::NAN;
        step.fw_terrain_sample_point = [f32::NAN, 0.0];
        let (outcome, _) = crossing(&step, |request| {
            assert_eq!(request.fixed_wing_terrain, None);
            answer_for(&request)
        });
        assert_eq!(outcome, Ok(()));
    }

    #[test]
    fn a_terrain_observation_is_refused_on_any_other_mode_and_when_it_is_not_finite() {
        let group = Group::new(&[("a", 12.0)]);
        let refused = |step: &sys::eech_movement_step, expected: &str| {
            let mut called = false;
            let (outcome, _) = crossing(step, |request| {
                called = true;
                answer_for(&request)
            });
            let message = refusal(outcome);
            assert!(message.contains(expected), "{message}");
            assert!(!called, "the host ran");
        };
        let mut misplaced = terrained_flight_step(&group);
        misplaced.mode = sys::EECH_MOVEMENT_PLACE;
        refused(&misplaced, "fixed-wing terrain observation");
        let mut hosted = hosted_step(sys::EECH_MOVEMENT_HELICOPTER_GUIDED_END);
        hosted.fw_terrain = 1;
        refused(&hosted, "fixed-wing terrain observation");
        let mut flag = terrained_flight_step(&group);
        flag.fw_terrain = 2;
        assert!(crossing(&flag, |request| answer_for(&request)).0.is_err());
        for edit in [
            (|step: &mut sys::eech_movement_step| step.fw_terrain_min_height = f32::INFINITY) as fn(&mut sys::eech_movement_step),
            |step| step.fw_terrain_sample_elevation = f32::NAN,
            |step| step.fw_terrain_site_y = f32::NAN,
            |step| step.fw_terrain_cg = f32::NAN,
            |step| step.fw_terrain_guide_position[1] = f32::NAN,
            |step| step.fw_terrain_sample_point[0] = f32::INFINITY,
        ] {
            let mut step = terrained_flight_step(&group);
            edit(&mut step);
            refused(&step, "non-finite");
        }
    }

    #[test]
    fn the_fixed_wing_lookahead_is_a_non_mutating_fixed_wing_step_the_host_answers_with_one_sample() {
        let mut step = hosted_step(sys::EECH_MOVEMENT_FIXED_WING_LOOKAHEAD);
        step.fw_motion_vector = [1.0, 2.0, 3.0];
        step.hc_motion_vector = [9.0, 9.0, 9.0];
        step.fw_terrain = 0;
        let (outcome, result) = crossing(&step, |request| {
            assert_eq!(request.mode, MovementMode::FixedWingLookahead);
            assert_eq!(request.fixed_wing_terrain, None);
            // the vector is the fixed-wing one, not the helicopter field
            assert_eq!(request.motion_vector_in, [1.0, 2.0, 3.0]);
            answer_for(&request)
        });
        assert_eq!(outcome, Ok(()));
        assert_eq!(result.hc_query.terrain_sample_count, 1);
        assert_eq!(result.hc_query.terrain_sample[0], [7.0, 8.0]);
        assert_eq!(result.hc_query.structure_scan, 0);
    }

    #[test]
    fn the_host_must_answer_the_fixed_wing_lookahead_with_exactly_one_sample_and_no_flags() {
        let step = hosted_step(sys::EECH_MOVEMENT_FIXED_WING_LOOKAHEAD);
        let missing = crossing(&step, |request| MovementResult {
            query: None,
            ..answer_for(&request)
        })
        .0;
        assert!(refusal(missing).contains("did not return a lookup plan"));
        for count in [0, 2] {
            let wrong = crossing(&step, |request| {
                let mut answer = answer_for(&request);
                if let Some(query) = answer.query.as_mut() {
                    query.terrain_sample_count = count;
                }
                answer
            })
            .0;
            assert!(refusal(wrong).contains("look-ahead"), "count {count}");
        }
        let not_a_number = crossing(&step, |request| {
            let mut answer = answer_for(&request);
            if let Some(query) = answer.query.as_mut() {
                query.terrain_sample[0][1] = f32::NAN;
            }
            answer
        })
        .0;
        assert!(not_a_number.is_err());
        let flagged = crossing(&step, |request| MovementResult {
            flags: 1,
            ..answer_for(&request)
        })
        .0;
        assert!(flagged.is_err());
        assert!(result_flags_are_valid(MovementMode::FixedWingLookahead, 0));
        assert!(!result_flags_are_valid(MovementMode::FixedWingLookahead, 1));
        assert!(!result_flags_are_valid(MovementMode::FixedWingLookahead, -1));
        // a move that is not the look-ahead is not answered with a lookup plan
        let group = Group::new(&[("a", 12.0)]);
        let flight = flight_step(&group);
        let stray = crossing(&flight, |request| MovementResult {
            query: Some(lookahead_query()),
            ..answer_for(&request)
        })
        .0;
        assert!(refusal(stray).contains("a lookup plan"));
    }
}
