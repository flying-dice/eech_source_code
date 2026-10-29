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

/// TEST-ONLY (feature `weapon-fixture`, EEDCS-46): the report text of a controlled combat case when the engine rejected its arguments. The case driver writes its reason
/// (`cmb-case-error reason=... detail=...`) into the report buffer before it returns the argument error, and `check` turns every argument error into "rejected by the engine";
/// this keeps the reason instead of discarding it. Any other failure, and a rejection that wrote nothing, is unchanged.
#[cfg(feature = "weapon-fixture")]
fn keep_case_reason(status: Result<(), EngineError>, text: &[std::os::raw::c_char]) -> Result<String, EngineError> {
    // SAFETY: the buffer was zero-filled and the engine NUL-terminates anything it writes within it.
    let report = unsafe { CStr::from_ptr(text.as_ptr()) }.to_string_lossy().into_owned();
    match status {
        Ok(()) => Ok(report),
        Err(EngineError::BadArgument(_)) if !report.trim().is_empty() => Err(EngineError::BadArgument(report.trim().to_string())),
        Err(error) => Err(error),
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
    /// the session is in winter: the global season the original reads when it picks the terrain files (default-W.sec and default-W.rgb when both exist),
    /// set before the terrain loads and carried to the host terrain provider on every terrain request
    pub winter: bool,
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
    /// an infantry member with a target turns to face it (EEDCS-44): the host forms the heading from its own record of the person's position
    /// and the target position sent, turns its record and answers the attitude and the heading (`angles[0]`)
    PersonFace,
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
    /// `PersonFace`: the target's position
    pub pf_target: [f32; 3],
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

/// The kind of structure the host holds an identity for: the campaign's own classification (csrc/eech_engine.h EECH_STRUCTURE_*), not an entity-type number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StructureKind {
    CityBuilding,
    Pylon,
    Scenic,
    Segment,
    Site,
    SiteUpdatable,
}

/// One structure the host holds an identity for (EEDCS-46, R3): a copy of its state at the call.  The identity is opaque and was issued by the host; it is not an entity index.
#[derive(Clone, Debug, Serialize)]
pub struct Structure {
    pub host_id: String,
    pub kind: StructureKind,
    pub entity_type: i32,
    pub sub_type: i32,
    pub side: Side,
    pub alive: bool,
    pub damage_level: i32,
    pub initial_damage_level: i32,
    pub position: [f32; 3],
}

/// A physical fact the host reports about an object it holds the identity of: its damage level is now `damage_level` and/or it is `destroyed`.  See mb_hostobj.h.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectReport {
    pub host_id: String,
    pub damage_level: Option<i32>,
    pub destroyed: bool,
}

/// What one object report applied, copied from the performed campaign steps; an observation, never an input.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ObjectReceipt {
    pub entity_type: i32,
    pub rostered: bool,
    pub damage_level_before: i32,
    pub damage_level_after: i32,
    pub killed: bool,
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
    pub lookup_hash: u64,
    pub lookups: u64,
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
            lookup_hash: 0,
            lookups: 0,
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
            lookup_hash: raw.lookup_hash,
            lookups: raw.lookups,
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

/// TEST-ONLY (feature `sensor-fixture`): room for one sensor report; a report that does not fit is refused, never cut short.
#[cfg(feature = "sensor-fixture")]
const SENSOR_REPORT_CAPACITY: usize = 96 * 1024;

/// TEST-ONLY (feature `terrain-fixture`): room for one terrain report (the probe's samples and the per-call-site table); a report that does not fit is refused, never cut short.
#[cfg(feature = "terrain-fixture")]
const TERRAIN_REPORT_CAPACITY: usize = 192 * 1024;

/// TEST-ONLY (feature `aim-fixture`): room for one aiming report; a report that does not fit is refused, never cut short.
#[cfg(feature = "aim-fixture")]
const AIM_REPORT_CAPACITY: usize = 96 * 1024;

/// TEST-ONLY (feature `weapon-fixture`): room for one weapon-execution report or one chunk of the event log; a report that does not fit is refused, never cut short.
#[cfg(feature = "weapon-fixture")]
const WPN_REPORT_CAPACITY: usize = 160 * 1024;

/// the booted engine (one per process)
pub struct Engine {
    _not_send: std::marker::PhantomData<*const ()>,
}

/// One question the campaign asks of the host's physical terrain (aphavoc/.../mb_hostterrain.h). No EECH pointer or layout is in it: the hint is the
/// host's own opaque string for one find-fast record, kept by the campaign and returned with the next query on that record. Every question carries
/// `winter`: the season the terrain was loaded in (the global season the original reads when it picks its terrain files), captured before the first
/// question, so a host serves the map of the season the campaign's own sector data was loaded for.
#[derive(Clone, Debug, PartialEq)]
pub enum TerrainQuery {
    /// the map facts the core reads once at boot
    Map { winter: bool },
    /// the surface at a point, for a record whose last hint was `hint` (`None` for a fresh or cleared record)
    Point { x: f32, z: f32, hint: Option<String>, winter: bool },
    /// the fast gridpost elevation at a point
    Fast { x: f32, z: f32, winter: bool },
}

/// The facts about the map the host owns (`TerrainQuery::Map`). `fast_width`/`fast_height` are 0 when the host has no fast grid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainMapFacts {
    pub sector_x_max: i32,
    pub sector_z_max: i32,
    pub fast_width: i32,
    pub fast_height: i32,
    pub min_x: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_z: f32,
    pub min_height: f32,
    pub max_height: f32,
    pub scaled_height_difference: f32,
}

/// What the host answers. A point answer carries what the query returns (`elevation`) and what it stores in the record (`last_elevation`, the
/// normal, the surface type `kind` as an EECH terrain-type id), the record's next `hint` (`None` when the query left none; the hint is the host's
/// own state and says nothing about the surface) and `stored`: whether the record holds a surface afterwards, which is what the campaign's legacy
/// "a face is stored" test (`get_3d_terrain_point_data_has_face`) reads. A host with no per-record state answers `stored: true` with no hint.
#[derive(Clone, Debug, PartialEq)]
pub enum TerrainAnswer {
    Map(TerrainMapFacts),
    Point {
        elevation: f32,
        last_elevation: f32,
        normal: [f32; 3],
        kind: i32,
        hint: Option<String>,
        stored: bool,
    },
    Fast {
        elevation: f32,
    },
}

/// The host's terrain provider. It is installed once, at boot, and is the only source of the campaign's physical terrain: the campaign has no
/// terrain solver of its own to fall back to.
pub type TerrainProvider = Box<dyn FnMut(TerrainQuery) -> Result<TerrainAnswer, String>>;

thread_local! {
    // the engine and its host run on one thread; a provider is not `Send`
    static TERRAIN: std::cell::RefCell<Option<TerrainProvider>> = std::cell::RefCell::new(None);
}

fn terrain_hint_text(buffer: &[c_char]) -> Result<Option<String>, String> {
    let bytes: Vec<u8> = buffer.iter().map(|byte| *byte as u8).take_while(|byte| *byte != 0).collect();
    if bytes.is_empty() {
        return Ok(None);
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| "[terrain-provider:point.hint] the record's hint is not UTF-8".to_string())
}

fn terrain_hint_into(buffer: &mut [c_char], hint: Option<&str>) -> Result<(), String> {
    buffer.iter_mut().for_each(|byte| *byte = 0);
    if let Some(text) = hint {
        let bytes = text.as_bytes();
        if bytes.is_empty() || bytes.len() > sys::EECH_TERRAIN_HINT_MAX || bytes.contains(&0) {
            return Err(format!(
                "[terrain-provider:point.hint] a hint is 1 to {} bytes without a NUL, got {}",
                sys::EECH_TERRAIN_HINT_MAX,
                bytes.len()
            ));
        }
        for (slot, byte) in buffer.iter_mut().zip(bytes) {
            *slot = *byte as c_char;
        }
    }
    Ok(())
}

/// The C side's terrain callback (mb_hostterrain.h). It runs the installed provider; a missing provider, a refusal and a panic all become an error
/// the engine stops on, with the provider's marker in the message.
unsafe extern "C" fn terrain_callback(
    request: *const sys::eech_terrain_request,
    reply_out: *mut sys::eech_terrain_reply,
    error: *mut c_char,
    error_size: usize,
    _user: *mut c_void,
) -> c_int {
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        if request.is_null() || reply_out.is_null() {
            return Err("[terrain-provider:point.callback] invalid native terrain callback context".into());
        }
        // SAFETY: C supplies a readable request for the duration of this callback.
        let source = unsafe { &*request };
        let winter = match source.winter {
            0 => false,
            1 => true,
            other => return Err(format!("[terrain-provider:point.callback] terrain received a season flag of {other}")),
        };
        let (operation, query) = match source.op {
            sys::EECH_TERRAIN_MAP => ("map", TerrainQuery::Map { winter }),
            sys::EECH_TERRAIN_POINT => (
                "point",
                TerrainQuery::Point {
                    x: source.x,
                    z: source.z,
                    hint: terrain_hint_text(&source.hint)?,
                    winter,
                },
            ),
            sys::EECH_TERRAIN_FAST => (
                "fast",
                TerrainQuery::Fast {
                    x: source.x,
                    z: source.z,
                    winter,
                },
            ),
            other => return Err(format!("[terrain-provider:point.callback] terrain received unknown operation {other}")),
        };
        let answer = TERRAIN.with(|slot| -> Result<TerrainAnswer, String> {
            let mut slot = slot
                .try_borrow_mut()
                .map_err(|_| "[terrain-provider:point.callback] the terrain provider was asked again while it was answering".to_string())?;
            match slot.as_mut() {
                Some(provider) => provider(query),
                None => Err("[terrain-provider:missing] no terrain provider is installed".to_string()),
            }
        })?;
        // SAFETY: every field of the reply is plain data, and zero is a valid value for each
        let mut reply: sys::eech_terrain_reply = unsafe { std::mem::zeroed() };
        // the answer must be the kind of answer that was asked for: a provider that answers a point with the map facts, or a map with a point, has not
        // answered the question, and the reply it would fill is not the reply the campaign reads for that operation
        match (operation, answer) {
            ("map", TerrainAnswer::Map(facts)) => {
                reply.sector_x_max = facts.sector_x_max;
                reply.sector_z_max = facts.sector_z_max;
                reply.fast_width = facts.fast_width;
                reply.fast_height = facts.fast_height;
                reply.min_x = facts.min_x;
                reply.min_z = facts.min_z;
                reply.max_x = facts.max_x;
                reply.max_z = facts.max_z;
                reply.min_height = facts.min_height;
                reply.max_height = facts.max_height;
                reply.scaled_height_difference = facts.scaled_height_difference;
            }
            (
                "point",
                TerrainAnswer::Point {
                    elevation,
                    last_elevation,
                    normal,
                    kind,
                    hint,
                    stored,
                },
            ) => {
                reply.elevation = elevation;
                reply.last_elevation = last_elevation;
                reply.normal = normal;
                reply.kind = kind;
                reply.stored = c_int::from(stored);
                terrain_hint_into(&mut reply.hint, hint.as_deref())?;
            }
            ("fast", TerrainAnswer::Fast { elevation }) => reply.elevation = elevation,
            (operation, answer) => {
                let given = match answer {
                    TerrainAnswer::Map(_) => "map facts",
                    TerrainAnswer::Point { .. } => "a point answer",
                    TerrainAnswer::Fast { .. } => "a fast elevation",
                };
                return Err(format!(
                    "[terrain-provider:{operation}.answer] the provider answered a {operation} question with {given}"
                ));
            }
        }
        // SAFETY: C supplies a writable reply for the duration of this callback.
        unsafe { *reply_out = reply };
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
            // SAFETY: C owns this writable error buffer for the callback.
            unsafe { write_callback_text(error, error_size, "[terrain-provider:point.callback] the terrain provider panicked") };
            1
        }
    }
}

struct HostContext<'a, F, G, H, M, L, D, A> {
    bind_mobile: &'a mut F,
    observe_loss: &'a mut G,
    supply_reached: &'a mut H,
    move_vehicle: &'a mut M,
    line_of_sight: &'a mut L,
    sense: &'a mut D,
    aim: &'a mut A,
}

/// One message of the host-owned line-of-sight dialogue. The campaign sends `Begin` with the ray; the host answers `Done` or asks for
/// one environmental value, which the campaign supplies as `Terrain` or `Objects`. No EECH pointer or entity index is in a message.
#[derive(Clone, Debug, PartialEq)]
pub enum LosMessage {
    Begin(LosBegin),
    /// the terrain elevation at the point the host asked about
    Terrain {
        elevation: f32,
    },
    /// whether a collision entity blocks the ray the host asked about, and where, only when it does
    Objects {
        hit: bool,
        collision_point: Option<[f32; 3]>,
    },
}

/// The ray and the campaign facts that come with it. `criteria` are the original `MOBILE_LOS_CHECK_*` bits.
#[derive(Clone, Debug, PartialEq)]
pub struct LosBegin {
    pub entity_form: bool,
    pub criteria: i32,
    pub source_eye: bool,
    pub target_eye: bool,
    pub source_host_id: Option<String>,
    pub target_host_id: Option<String>,
    pub source_position: [f32; 3],
    pub target_position: [f32; 3],
}

/// What the host answers to a message: the visibility, or one value it needs from the campaign. `phase` is informational.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LosReply {
    NeedTerrain { phase: i32, point: [f32; 3] },
    NeedObjects { phase: i32, start: [f32; 3], end: [f32; 3] },
    Done { visible: bool },
}

/// Which of the four AI target scans a sensing message belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SenseScan {
    AirToAir,
    AirToSurface,
    SurfaceToAir,
    SurfaceToSurface,
}

/// One message of the host-owned sensing dialogue (aphavoc/.../mb_hostsense.h). The campaign opens a scan with `Begin`, asks about each
/// candidate it enumerated with `Candidate` (and, when the host asks, `Draw` and `Signature`), and closes the scan with `End`.
/// No EECH pointer, entity index or seed is in a message.
#[derive(Clone, Debug, PartialEq)]
pub enum SenseMessage {
    Begin(SenseBegin),
    Candidate(SenseCandidate),
    /// the random draw the campaign took, at the point the original scan took it
    Draw {
        draw: f64,
    },
    Signature(SenseSignature),
    End {
        scan: SenseScan,
    },
}

/// The source's position and sensor facts. `scan_ceiling` and `scan_floor` are the group's raw figures, meaningful for the surface-to-air scan.
#[derive(Clone, Debug, PartialEq)]
pub struct SenseBegin {
    pub scan: SenseScan,
    pub source_host_id: Option<String>,
    pub source_position: [f32; 3],
    pub scan_range: f32,
    pub scan_ceiling: f32,
    pub scan_floor: f32,
}

/// One candidate target. `radar_altitude` is present for the surface-to-air scan, `difficulty_factor` when a human flies the target.
#[derive(Clone, Debug, PartialEq)]
pub struct SenseCandidate {
    pub scan: SenseScan,
    pub target_host_id: Option<String>,
    pub target_position: [f32; 3],
    pub target_player: bool,
    pub difficulty_factor: Option<f32>,
    pub radar_altitude: Option<f32>,
}

/// The state of a human-flown target's emitters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SenseEmitters {
    pub radar_on: bool,
    pub radar_jammer_on: bool,
    pub infra_red_jammer_on: bool,
}

/// The configuration facts of a helicopter target's radar-stealth factor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SenseHelicopter {
    pub weapon_bay_closed: bool,
    pub stub_wings_attached: bool,
    pub undercarriage_up: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SenseSignature {
    pub scan: SenseScan,
    pub emitters: Option<SenseEmitters>,
    pub helicopter: Option<SenseHelicopter>,
}

/// What the host says: its answer state, and for a rejection why the candidate was not perceived.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SenseState {
    Open,
    Accept,
    Reject(SenseReason),
    NeedDraw,
    NeedSignature,
    Closed,
}

/// Why a candidate was not perceived. The first four name the predicates of the original EECH sensing model, which the reference host reproduces
/// (each is legal only for the scans whose original had it); `NotPerceived` is a host's own verdict from its own model and is legal for every scan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SenseReason {
    Range,
    RangeAdjusted,
    Altitude,
    Tolerance,
    NotPerceived,
}

/// The host's intermediate physical values, for the test fixture build's trace only; the shipping module ignores them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SenseEcho {
    pub ceiling: f32,
    pub floor: f32,
    pub adjusted_range: f32,
    pub stealth_factor: f32,
    pub stealth_limit: f32,
    pub tolerance_limit: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SenseReply {
    pub state: SenseState,
    /// the measured range, which the campaign's ranking uses: required with an acceptance
    pub target_range: Option<f32>,
    pub echo: Option<SenseEcho>,
}

/// One weapon's physical figures, mirrored from the campaign's weapon database (transitional: a host with a model of its own needs none of them).
#[derive(Clone, Debug, PartialEq)]
pub struct WeaponFigures {
    pub name: String,
    pub start_weight: f32,
    pub final_weight: f32,
    pub max_range: f32,
    pub muzzle_velocity: f32,
    pub cruise_velocity: f32,
    pub g_max: f32,
    pub boost_time: f32,
    pub cruise_time: f32,
    pub inhibit_time: f32,
    pub drag_coefficient: f32,
    pub flightpath_deviation: f32,
    pub boost_power: f32,
    pub sustain_power: f32,
    pub sustain_time: f32,
    pub diameter: f32,
    pub max_altitude: f32,
    pub ignore_gravity: i32,
    pub flight_profile_or_self_destr: i32,
    pub parachute_type: i32,
    pub weapon_class: u32,
    pub cruise_time_max_error: f32,
    pub detonation_radius: f32,
    pub max_seeker_limit: f32,
    pub max_launch_angle_error: f32,
    pub survive_ground_impact: i32,
    pub in_flight_collision_check: i32,
    pub guidance_type: i32,
    pub aiming_type: i32,
    pub decoy_type: i32,
    pub warhead_type: i32,
    pub hellfire_ii: i32,
    pub smoke_grenade: i32,
    pub submunition_count: i32,
    pub submunition_velocity_factor: i32,
    pub muzzle_velocity_max_error: f32,
    pub circular_error_probable: f32,
    pub rate_of_fire: f32,
    pub inverse_rate_of_fire: f32,
    pub acquire_forward: i32,
}

/// An unguided weapon's figures, handed to the host once at boot: the host simulates the weapon's flight and keeps the ballistic table that results (no cell crosses the
/// boundary). `first` and `last` flag the ends of the pass (the host writes a diagnostic file across it). `echo` is the test fixture build's request for the host's table
/// and flight records back (`echo_cells` is the number of floats the campaign has room for).
#[derive(Clone, Debug, PartialEq)]
pub struct AimTable {
    pub weapon: i32,
    pub figures: WeaponFigures,
    pub first: bool,
    pub last: bool,
    pub echo: bool,
    pub echo_cells: usize,
}

/// A guided weapon's figures, handed to the host once at boot: the host runs its two range simulations and writes its diagnostic file.
#[derive(Clone, Debug, PartialEq)]
pub struct AimRange {
    pub weapon: i32,
    pub figures: WeaponFigures,
    pub first: bool,
    pub last: bool,
    pub echo: bool,
}

/// One weapon's figures, handed to the host at boot to keep (every weapon, whether or not it has a table or a range run): the host flies the weapon by them.
#[derive(Clone, Debug, PartialEq)]
pub struct AimFigures {
    pub weapon: i32,
    pub figures: WeaponFigures,
}

/// The weapon as the campaign holds it for one frame of its flight. The entity-local random stream never leaves the campaign: `entity_samples` are its first four outputs
/// (raw integers in [0, 32767]), all any one call of the original uses.
#[derive(Clone, Debug, PartialEq)]
pub struct AimWeaponState {
    pub position: [f32; 3],
    pub attitude: [f32; 9],
    pub motion_vector: [f32; 3],
    pub velocity: f32,
    pub weapon_lifetime: f32,
    pub decoy_timer: f32,
    pub missile_phase: i32,
    pub loal_mode: bool,
    pub weapon_on_ground: bool,
    pub parachute_status: i32,
    pub kill_code: i32,
    pub entity_samples: [i32; 4],
    pub has_launcher: bool,
    pub has_target: bool,
    pub server: bool,
    /// A birth only: the weapon is a submunition, and `velocity` is its parent weapon's.
    pub submunition: bool,
    /// A seduction only: the decoy's weapon number, the difficulty coefficient and the chaff, flare and smoke effectiveness settings.
    pub other_weapon: i32,
    pub coefficient: f32,
    pub effectiveness: [f32; 3],
}

/// What a request of the weapon step protocol asks the host for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AimStepKind {
    /// A frame of a weapon's flight.
    Step,
    /// A weapon's birth.
    Birth,
    /// A decoy's update.
    Decoy,
    /// A seeker's answer to a decoy becoming active.
    Seduce,
    /// A seeker's answer to the decoy it was tracking expiring.
    Reacquire,
}

/// One frame of a falling weapon's parachute.
#[derive(Clone, Debug, PartialEq)]
pub struct AimAnimate {
    pub weapon: i32,
    pub parachute_status: i32,
    pub weapon_on_ground: bool,
    pub weapon_lifetime: f32,
    pub motion_vector_y: f32,
}

/// The rate-of-fire gate of a launcher releasing a weapon: the frame, the clock, the launcher's last-shot clock (when it has one) and whether it is not an AI.
#[derive(Clone, Debug, PartialEq)]
pub struct AimRelease {
    pub weapon: i32,
    pub delta_time: f32,
    pub current_time: u32,
    pub last_shot: Option<u32>,
    pub not_ai: bool,
}

/// The damage a weapon is capable of against a target, with no weapon in flight (EEDCS-46): the question by value.  `weapon_warhead` is the warhead type of the weapon's own
/// definition; `warhead`, `damage_capability`, `armor_penetration_capability` and `burst_duration` are the figures the forecast is made from (the weapon's own, or for a
/// conventional-munitions weapon those of the munition it scatters, as the campaign resolved them); `armour_level` is the target's table level; `target_not_ai` says the
/// player's damage setting `player_modifier` applies; `damage_modifier` and `velocity` are the arguments of the original function.  `echo` is the test fixture build's request
/// for the reference host's intermediate values.
#[derive(Clone, Debug, PartialEq)]
pub struct AimForecast {
    pub weapon: i32,
    pub weapon_warhead: i32,
    pub warhead: i32,
    pub damage_capability: i32,
    pub armor_penetration_capability: i32,
    pub burst_duration: f32,
    pub armour_level: i32,
    pub target_not_ai: bool,
    pub player_modifier: f32,
    pub damage_modifier: f32,
    pub velocity: f32,
    pub echo: bool,
}

/// One hit that reached a damage handler (EEDCS-46, real weapon), by value: see `eech_damage_request` (mb_hostaim.h campaign_damage_request).  `draw` is the sample of the
/// campaign's shared random stream (a raw integer in [0, 32767]) the host asked for, once the campaign has taken it.
#[derive(Clone, Debug, PartialEq)]
pub struct AimDamage {
    pub weapon: i32,
    pub weapon_warhead: i32,
    pub warhead: i32,
    pub damage_capability: i32,
    pub armor_penetration_capability: i32,
    pub burst_duration: f32,
    pub weapon_speed: f32,
    pub weapon_cruise_velocity: f32,
    pub damage_modifier: f32,
    pub armour_level: i32,
    pub armour_easy: bool,
    pub armour_model: i32,
    pub armour_front: f32,
    pub armour_rear: f32,
    pub armour_side: f32,
    pub weapon_position: [f32; 3],
    pub target_position: [f32; 3],
    pub target_attitude: [f32; 9],
    pub target_not_ai: bool,
    /// The player's damage setting, if the campaign could resolve one for the difficulty.
    pub player_modifier: Option<f32>,
    pub has_launcher: bool,
    pub multiplier: i32,
    pub invulnerable: bool,
    pub cheat_guard: bool,
    pub damage_level: i32,
    pub initial_damage_level: i32,
    pub critical_model: i32,
    pub draw: Option<i32>,
    pub echo: bool,
    /// The identity of the object that is hit (a rostered mobile's or a structure's), when it has one.
    pub target_host_id: Option<String>,
}

/// One health question that is not a hit (EEDCS-46), by value: see `eech_health_request` (mb_hostaim.h campaign_health_request).  `draw` is the sample of the campaign's shared random
/// stream (a raw integer in [0, 32767]) a timed roll asked for, once the campaign has taken it.
#[derive(Clone, Debug, PartialEq)]
pub struct AimHealth {
    pub op: i32,
    pub critical_model: i32,
    pub damage_level: i32,
    pub initial_damage_level: i32,
    pub old_damage_level: i32,
    pub explosive: bool,
    pub flammable: bool,
    pub delta_time: f32,
    pub aircraft_class: i32,
    pub self_hit: bool,
    pub target_not_ai: bool,
    pub draw: Option<i32>,
    /// The identity of the object whose health it is, when it has one.
    pub target_host_id: Option<String>,
}

/// What a weapon's death does to the world (EEDCS-46), by value: see `eech_impact_request` (mb_hostaim.h campaign_impact_request).  `terrain_draws_crater` is the terrain fact at the impact
/// position once the campaign has read it for the host (`None` until the host has asked).
#[derive(Clone, Debug, PartialEq)]
pub struct AimImpact {
    pub kill_code: i32,
    pub has_target: bool,
    pub target_alive: bool,
    pub terrain_draws_crater: Option<bool>,
    pub position: [f32; 3],
    /// The identity of the weapon's target, when it has a target that has one.
    pub target_host_id: Option<String>,
}

/// The facts of a structure the campaign asks the host to issue an identity for (EEDCS-46, R3), by value: see `eech_identify_request` (mb_hostaim.h campaign_identify_request).  `serial` counts the
/// identities the campaign has asked for; it is a diagnostic, not a heap index.
#[derive(Clone, Debug, PartialEq)]
pub struct AimIdentify {
    pub entity_type: i32,
    pub sub_type: i32,
    pub side: i32,
    pub alive: bool,
    pub damage_level: i32,
    pub initial_damage_level: i32,
    pub position: [f32; 3],
    pub serial: i32,
}

/// One object of a sector's list the campaign offered to the host (EEDCS-46), by value: see `eech_blast_entry` (mb_hostaim.h campaign_blast_entry).  `handle` names the object for
/// this blast only; `host_id` is the persistent identity the host issued for the object (a rostered mobile's or a structure's) when it has one.
#[derive(Clone, Debug, PartialEq)]
pub struct AimBlastEntry {
    pub handle: i32,
    pub entity_type: i32,
    pub readable: bool,
    pub alive: bool,
    pub has_position: bool,
    pub rocket: bool,
    pub excluded: bool,
    pub position: [f32; 3],
    pub host_id: Option<String>,
}

/// The window of a sector's list the host asked for: the sector, the handle it follows (0: the start of the list), whether the list ends with it, and the objects in list order.
#[derive(Clone, Debug, PartialEq)]
pub struct AimBlastWindow {
    pub sx: i32,
    pub sz: i32,
    pub after: i32,
    pub end: bool,
    pub entries: Vec<AimBlastEntry>,
}

/// One area blast (EEDCS-46), by value: see `eech_blast_request` (mb_hostaim.h campaign_blast_request).  The first request carries no window.
#[derive(Clone, Debug, PartialEq)]
pub struct AimBlast {
    pub position: [f32; 3],
    pub damage_capability: i32,
    pub warhead_type: i32,
    pub has_target: bool,
    pub sector_side_length: i32,
    pub min_x_sector: i32,
    pub max_x_sector: i32,
    pub min_z_sector: i32,
    pub max_z_sector: i32,
    pub window: Option<AimBlastWindow>,
    pub echo: bool,
}

/// One question about the physical collapse of a bridge segment (EEDCS-46, R5), by value: see `eech_bridge_request` (mb_hostaim.h campaign_bridge_request).  The terrain is present once the campaign has
/// read it for the host; the window is the vehicles walk's.
#[derive(Clone, Debug, PartialEq)]
pub struct AimBridge {
    pub op: i32,
    pub segment_kind: i32,
    pub alive: bool,
    pub server: bool,
    pub pred_present: bool,
    pub pred_alive: bool,
    pub succ_present: bool,
    pub succ_alive: bool,
    pub which: i32,
    pub neighbour_present: bool,
    pub neighbour_kind: i32,
    pub neighbour_alive: bool,
    pub position: [f32; 3],
    pub delta_time: f32,
    pub terrain: Option<(f32, bool)>,
    pub segment_length: f32,
    pub sector_side_length: i32,
    pub min_x_sector: i32,
    pub max_x_sector: i32,
    pub min_z_sector: i32,
    pub max_z_sector: i32,
    pub window: Option<AimBlastWindow>,
}

/// One frame of one weapon's flight, or (with `birth`) its birth: the weapon, the clock and the frame, and what the host has been told so far in answer to the lookups it asked for (`answers`, each
/// the lookup repeated with its results in `f[6..12]`) and the samples of the campaign's shared random stream it has been given (`draws`, raw integers in [0, 32767]).
#[derive(Clone, Debug, PartialEq)]
pub struct AimWeaponStep {
    pub kind: AimStepKind,
    pub echo: bool,
    pub weapon: i32,
    pub delta_time: f32,
    pub system_time: u32,
    pub map_min: [f32; 3],
    pub map_max: [f32; 3],
    pub state: AimWeaponState,
    pub answers: Vec<AimEvent>,
    pub draws: Vec<i32>,
}

/// The figures of a weapon the host aims with. `present` is false when there is no weapon (a zero speed).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AimWeapon {
    pub weapon: i32,
    pub present: bool,
    pub aiming_type: i32,
    pub cruise_velocity: f32,
    pub acquire_forward: bool,
    pub source_velocity: f32,
}

/// The target facts the campaign supplies: its position, the model's target point, its motion vector and whether it is a mobile.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AimTarget {
    pub position: [f32; 3],
    pub point: [f32; 3],
    pub motion_vector: [f32; 3],
    pub mobile: bool,
}

/// The world figures a prediction is bounded by.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AimWorld {
    pub map_min: [f32; 3],
    pub map_max: [f32; 3],
    pub comms_weapon_lag: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AimReadyUpdate {
    pub source: Option<String>,
    pub state: f32,
    pub rate: f32,
    pub delta_time: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AimReadyCommand {
    pub source: Option<String>,
    pub state: f32,
    pub open: bool,
}

/// A package's limits on the offsets its devices are asked to reach, and whether the weapon is a ground-launched guided missile (a slight climb is
/// asked for).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AimLimits {
    pub heading_min: f32,
    pub heading_max: f32,
    pub pitch_min: f32,
    pub pitch_max: f32,
    pub climb: bool,
}

/// The player's cannon tracking: the gun is aimed at `point` from `device_position`, lowered by `pitch`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AimTrack {
    pub device_position: [f32; 3],
    pub point: [f32; 3],
    pub pitch: f32,
}

/// The offsets a device is asked to reach: toward `intercept_vector` within the package's `limits`, or, without limits (the HUD's offsets, the player's
/// cannon tracking), as they are. With `tracking` there is no intercept vector (it is zero).
#[derive(Clone, Debug, PartialEq)]
pub struct AimRequired {
    pub source: Option<String>,
    pub device_attitude: [f32; 9],
    pub intercept_vector: [f32; 3],
    pub limits: Option<AimLimits>,
    pub tracking: Option<AimTrack>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AimStep {
    pub source: Option<String>,
    pub current_heading: f32,
    pub current_pitch: f32,
    pub required_heading: f32,
    pub required_pitch: f32,
    pub heading_min: f32,
    pub heading_max: f32,
    pub heading_rate: f32,
    pub pitch_rate: f32,
    pub delta_time: f32,
}

/// The pitch device's vectors for a launcher: with a weapon and a target (`weapon` and `target` set) or from the player's tracking point.
#[derive(Clone, Debug, PartialEq)]
pub struct AimVectors {
    pub echo: bool,
    pub source: Option<String>,
    pub target_id: Option<String>,
    pub world: AimWorld,
    pub device_position: [f32; 3],
    pub device_attitude: [f32; 9],
    pub tracking: Option<[f32; 3]>,
    pub weapon: Option<AimWeapon>,
    pub source_gunship: bool,
    pub target: Option<AimTarget>,
    /// the raw sample of the campaign's random stream the host asked for, an integer in [0, 32767]
    pub draw: Option<i32>,
}

/// The lead and ballistic prediction alone.
#[derive(Clone, Debug, PartialEq)]
pub struct AimLead {
    pub echo: bool,
    pub source: Option<String>,
    pub target_id: Option<String>,
    pub world: AimWorld,
    pub weapon: i32,
    pub source_gunship: bool,
    pub parent_velocity: f32,
    pub device_position: [f32; 3],
    pub target: AimTarget,
    pub draw: Option<i32>,
}

/// One table interpolation.
#[derive(Clone, Debug, PartialEq)]
pub struct AimBallistic {
    pub echo: bool,
    pub weapon: i32,
    pub range: f32,
    pub height_diff_or_pitch: f32,
    pub simplified: bool,
    pub fixed_pitch: bool,
    pub parent_velocity: f32,
}

/// The guided-weapon intercept prediction alone.
#[derive(Clone, Debug, PartialEq)]
pub struct AimIntercept {
    pub echo: bool,
    pub source: Option<String>,
    pub target_id: Option<String>,
    pub weapon: AimWeapon,
    pub device_position: [f32; 3],
    pub target: AimTarget,
}

/// One request of the host-owned physical aiming and weapon-system-readiness exchange (aphavoc/.../mb_hostaim.h). Every request is a pure
/// question: the host keeps nothing between requests except the ballistic tables it was given. No EECH pointer, index or seed is in a request.
#[derive(Clone, Debug, PartialEq)]
pub enum AimRequest {
    Table(AimTable),
    Range(AimRange),
    Figures(AimFigures),
    WeaponStep(AimWeaponStep),
    Release(AimRelease),
    Forecast(AimForecast),
    Damage(AimDamage),
    Health(AimHealth),
    Impact(AimImpact),
    Blast(AimBlast),
    Bridge(AimBridge),
    Identify(AimIdentify),
    Animate(AimAnimate),
    ReadyUpdate(AimReadyUpdate),
    ReadyCommand(AimReadyCommand),
    Required(AimRequired),
    Step(AimStep),
    Vectors(AimVectors),
    Lead(AimLead),
    Ballistic(AimBallistic),
    Intercept(AimIntercept),
}

/// One intermediate value of the reference host's prediction, in its order (for the test fixture build's trace only).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AimEvent {
    pub kind: i32,
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
    pub f: [f32; 12],
}

/// The host's answer. `need_draw` asks the campaign for one sample of its random stream (a vectors or lead request only); every other field
/// is read according to the request. `result` is the validity of the vectors or the success of the prediction.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AimReply {
    pub need_draw: bool,
    /// A weapon step only: the host needs the answers to `lookups` and asks again.
    pub need_lookup: bool,
    /// A weapon step: the lookups it still needs (with `need_lookup`), or the actions it left, in the order the campaign applies them (when done).
    pub lookups: Vec<AimEvent>,
    pub actions: Vec<AimEvent>,
    /// A release only: the launcher lets nothing go; the rounds it lets go; the last-shot clock the gate wrote (even when it then refused), if it wrote one.
    pub release_refuse: bool,
    pub release_burst_size: i32,
    pub release_last_shot: Option<u32>,
    /// A forecast only: the capability, never negative.
    pub forecast_capability: i32,
    /// A damage question only: see `eech_damage_result`.
    pub damage_exited: bool,
    pub damage_capability: i32,
    pub damage_written: bool,
    pub damage_new_level: i32,
    pub damage_critical_before: i32,
    pub damage_critical_after: i32,
    pub damage_bad_modifier: bool,
    /// A health question only: see `eech_health_result`.
    pub health_critical_class: i32,
    pub health_crossed: bool,
    pub health_recovered: bool,
    pub health_eject: bool,
    pub health_eligible: bool,
    pub health_rolled: bool,
    pub health_hit: bool,
    pub health_new_level: i32,
    pub health_fatal: bool,
    /// An impact question only: see `eech_impact_result`.  With `need_lookup` the host asks for the terrain fact (`impact_need_terrain`).
    pub impact_need_terrain: bool,
    pub impact_invalid: bool,
    pub impact_crater: bool,
    pub impact_direct_hit: bool,
    pub impact_blasts: i32,
    pub impact_exclude_target: bool,
    /// A blast question only: see `eech_blast_result`.  The radius is reported with every answer.  With `need_lookup` the host asks for the window at `blast_need` (sector x, sector
    /// z, the handle it follows); otherwise the answer is the action (0 the walk is over, 1 a collision, 2 a rocket destroyed) and, for an action, the object and the modifier.
    pub blast_radius: f32,
    pub blast_need: Option<(i32, i32, i32)>,
    pub blast_action: i32,
    pub blast_handle: i32,
    /// The identity of the object `blast_handle` names, repeated (see `eech_blast_result`).
    pub blast_target_host_id: Option<String>,
    pub blast_modifier: f32,
    /// A bridge question only: see `eech_bridge_result`.  With `need_lookup` the host asks for the terrain (`bridge_need_terrain`) or, for the vehicles walk, the window at `bridge_need`.
    pub bridge_kill: bool,
    pub bridge_variant: i32,
    pub bridge_notify: bool,
    pub bridge_need_terrain: bool,
    pub bridge_fell: bool,
    pub bridge_dropped_y: f32,
    pub bridge_floor_height: f32,
    pub bridge_landed: bool,
    pub bridge_landed_y: f32,
    pub bridge_remove: bool,
    pub bridge_need: Option<(i32, i32, i32)>,
    pub bridge_action: i32,
    pub bridge_handle: i32,
    /// The identity of the object `bridge_handle` names, repeated.
    pub bridge_target_host_id: Option<String>,
    /// An identify request only: the identity the host issued.
    pub identify_host_id: Option<String>,
    /// An animation only: the parachute's state, and the weapon's lifetime when the animation reset it.
    pub animate_status: i32,
    pub animate_lifetime: Option<f32>,
    pub result: bool,
    pub ready_state: f32,
    pub phase: i32,
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
    /// A partial output (a `lead` or, on success, a `ballistic` answer only): `None` when the host produced no flight time, in which case the campaign leaves its caller's
    /// variable alone. `Some` must be finite.
    pub time_of_flight: Option<f32>,
    pub echo: Option<Vec<AimEvent>>,
    /// A table request's echo (the test fixture build only): the cells of the table the host generated, (drop angle, flight time) float pairs, velocity test by pitch index
    /// by range; its length must be exactly the room the campaign gave.
    pub table_cells: Option<Vec<f32>>,
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

unsafe extern "C" fn bind_mobile<F, G, H, M, L, D, A>(
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
    L: FnMut(LosMessage) -> Result<LosReply, String>,
    D: FnMut(SenseMessage) -> Result<SenseReply, String>,
    A: FnMut(AimRequest) -> Result<AimReply, String>,
{
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        if descriptor.is_null() || user.is_null() {
            return Err("invalid native bind_mobile callback context".into());
        }
        // SAFETY: C supplies both pointers for the duration of this callback.
        let (descriptor, context) = unsafe { (&*descriptor, &mut *(user as *mut HostContext<'_, F, G, H, M, L, D, A>)) };
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

unsafe extern "C" fn observe_loss<F, G, H, M, L, D, A>(
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
    L: FnMut(LosMessage) -> Result<LosReply, String>,
    D: FnMut(SenseMessage) -> Result<SenseReply, String>,
    A: FnMut(AimRequest) -> Result<AimReply, String>,
{
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        if descriptor.is_null() || decision.is_null() || user.is_null() {
            return Err("invalid native observe_loss callback context".into());
        }
        // SAFETY: C supplies these pointers for the duration of the callback.
        let (descriptor, decision, context) = unsafe { (&*descriptor, &mut *decision, &mut *(user as *mut HostContext<'_, F, G, H, M, L, D, A>)) };
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

unsafe extern "C" fn supply_reached<F, G, H, M, L, D, A>(
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
    L: FnMut(LosMessage) -> Result<LosReply, String>,
    D: FnMut(SenseMessage) -> Result<SenseReply, String>,
    A: FnMut(AimRequest) -> Result<AimReply, String>,
{
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        if descriptor.is_null() || answer_out.is_null() || user.is_null() {
            return Err("invalid native supply_reached callback context".into());
        }
        // SAFETY: C supplies both pointers for the duration of this callback.
        let (source, context) = unsafe { (&*descriptor, &mut *(user as *mut HostContext<'_, F, G, H, M, L, D, A>)) };
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

/// Reads a NUL-terminated host identity out of a fixed buffer: an empty one is `None`, one that is not UTF-8 is refused.
fn los_host_id(buffer: &[c_char], role: &str) -> Result<Option<String>, String> {
    let bytes: Vec<u8> = buffer.iter().map(|byte| *byte as u8).take_while(|byte| *byte != 0).collect();
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes.len() > sys::EECH_LOS_HOST_ID_MAX {
        return Err(format!("line_of_sight received an overlong {role} host identity"));
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| format!("line_of_sight received a non-UTF-8 {role} host identity"))
}

unsafe extern "C" fn line_of_sight<F, G, H, M, L, D, A>(
    step: *const sys::eech_los_step,
    result_out: *mut sys::eech_los_result,
    error: *mut c_char,
    error_size: usize,
    user: *mut c_void,
) -> c_int
where
    F: FnMut(MemberDescriptor) -> Result<String, String>,
    G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
    H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
    M: FnMut(MovementStep) -> Result<MovementResult, String>,
    L: FnMut(LosMessage) -> Result<LosReply, String>,
    D: FnMut(SenseMessage) -> Result<SenseReply, String>,
    A: FnMut(AimRequest) -> Result<AimReply, String>,
{
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        if step.is_null() || result_out.is_null() || user.is_null() {
            return Err("invalid native line_of_sight callback context".into());
        }
        // SAFETY: C supplies both pointers for the duration of this callback.
        let (source, context) = unsafe { (&*step, &mut *(user as *mut HostContext<'_, F, G, H, M, L, D, A>)) };
        let message = match source.mode {
            sys::EECH_LOS_BEGIN => {
                for (name, values) in [
                    ("source_position", &source.source_position[..]),
                    ("target_position", &source.target_position[..]),
                ] {
                    if values.iter().any(|value| !value.is_finite()) {
                        return Err(format!("non-finite line_of_sight {name}"));
                    }
                }
                if !(0..32).contains(&source.criteria) {
                    return Err(format!("line_of_sight received criteria {} outside the five check bits", source.criteria));
                }
                LosMessage::Begin(LosBegin {
                    entity_form: checked_bool("entity_form", source.entity_form)?,
                    criteria: source.criteria,
                    source_eye: checked_bool("source_eye", source.source_eye)?,
                    target_eye: checked_bool("target_eye", source.target_eye)?,
                    source_host_id: los_host_id(&source.source_host_id, "source")?,
                    target_host_id: los_host_id(&source.target_host_id, "target")?,
                    source_position: source.source_position,
                    target_position: source.target_position,
                })
            }
            sys::EECH_LOS_TERRAIN => {
                if !source.elevation.is_finite() {
                    return Err("non-finite line_of_sight terrain elevation".into());
                }
                LosMessage::Terrain { elevation: source.elevation }
            }
            sys::EECH_LOS_OBJECTS => {
                let hit = checked_bool("hit", source.hit)?;
                if hit && source.collision_point.iter().any(|value| !value.is_finite()) {
                    return Err("non-finite line_of_sight collision point".into());
                }
                LosMessage::Objects {
                    hit,
                    collision_point: hit.then_some(source.collision_point),
                }
            }
            other => return Err(format!("line_of_sight received unknown mode {other}")),
        };
        let reply = (context.line_of_sight)(message)?;
        let mut answer = sys::eech_los_result {
            state: 0,
            phase: 0,
            point: [0.0; 3],
            start: [0.0; 3],
            end: [0.0; 3],
            visible: 0,
        };
        match reply {
            LosReply::NeedTerrain { phase, point } => {
                answer.state = sys::EECH_LOS_NEED_TERRAIN;
                answer.phase = phase;
                answer.point = point;
            }
            LosReply::NeedObjects { phase, start, end } => {
                answer.state = sys::EECH_LOS_NEED_OBJECTS;
                answer.phase = phase;
                answer.start = start;
                answer.end = end;
            }
            LosReply::Done { visible } => {
                answer.state = sys::EECH_LOS_DONE;
                answer.visible = c_int::from(visible);
            }
        }
        // SAFETY: C supplies a writable result for the duration of this callback.
        unsafe { *result_out = answer };
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
            unsafe { write_callback_text(error, error_size, "line_of_sight panicked") };
            1
        }
    }
}

fn sense_host_id(buffer: &[c_char], role: &str) -> Result<Option<String>, String> {
    let bytes: Vec<u8> = buffer.iter().map(|byte| *byte as u8).take_while(|byte| *byte != 0).collect();
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes.len() > sys::EECH_SENSE_HOST_ID_MAX {
        return Err(format!("sense received an overlong {role} host identity"));
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| format!("sense received a non-UTF-8 {role} host identity"))
}

fn sense_scan(value: c_int) -> Result<SenseScan, String> {
    match value {
        sys::EECH_SENSE_AIR_TO_AIR => Ok(SenseScan::AirToAir),
        sys::EECH_SENSE_AIR_TO_SURFACE => Ok(SenseScan::AirToSurface),
        sys::EECH_SENSE_SURFACE_TO_AIR => Ok(SenseScan::SurfaceToAir),
        sys::EECH_SENSE_SURFACE_TO_SURFACE => Ok(SenseScan::SurfaceToSurface),
        other => Err(format!("sense received unknown scan {other}")),
    }
}

unsafe extern "C" fn sense<F, G, H, M, L, D, A>(
    step: *const sys::eech_sense_step,
    result_out: *mut sys::eech_sense_result,
    error: *mut c_char,
    error_size: usize,
    user: *mut c_void,
) -> c_int
where
    F: FnMut(MemberDescriptor) -> Result<String, String>,
    G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
    H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
    M: FnMut(MovementStep) -> Result<MovementResult, String>,
    L: FnMut(LosMessage) -> Result<LosReply, String>,
    D: FnMut(SenseMessage) -> Result<SenseReply, String>,
    A: FnMut(AimRequest) -> Result<AimReply, String>,
{
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        if step.is_null() || result_out.is_null() || user.is_null() {
            return Err("invalid native sense callback context".into());
        }
        // SAFETY: C supplies both pointers for the duration of this callback.
        let (source, context) = unsafe { (&*step, &mut *(user as *mut HostContext<'_, F, G, H, M, L, D, A>)) };
        let finite = |name: &str, values: &[f32]| -> Result<(), String> {
            if values.iter().any(|value| !value.is_finite()) {
                Err(format!("non-finite sense {name}"))
            } else {
                Ok(())
            }
        };
        let scan = sense_scan(source.scan)?;
        let message = match source.mode {
            sys::EECH_SENSE_BEGIN => {
                finite("source_position", &source.source_position)?;
                finite("sensor figure", &[source.scan_range, source.scan_ceiling, source.scan_floor])?;
                SenseMessage::Begin(SenseBegin {
                    scan,
                    source_host_id: sense_host_id(&source.source_host_id, "source")?,
                    source_position: source.source_position,
                    scan_range: source.scan_range,
                    scan_ceiling: source.scan_ceiling,
                    scan_floor: source.scan_floor,
                })
            }
            sys::EECH_SENSE_CANDIDATE => {
                finite("target_position", &source.target_position)?;
                let player = checked_bool("target_player", source.target_player)?;
                let surface_to_air = scan == SenseScan::SurfaceToAir;
                let difficulty_factor = if checked_bool("has_difficulty", source.has_difficulty)? {
                    finite("difficulty_factor", &[source.difficulty_factor])?;
                    Some(source.difficulty_factor)
                } else {
                    None
                };
                if surface_to_air {
                    finite("radar_altitude", &[source.radar_altitude])?;
                }
                SenseMessage::Candidate(SenseCandidate {
                    scan,
                    target_host_id: sense_host_id(&source.target_host_id, "target")?,
                    target_position: source.target_position,
                    target_player: player,
                    difficulty_factor,
                    radar_altitude: surface_to_air.then_some(source.radar_altitude),
                })
            }
            sys::EECH_SENSE_DRAW => {
                if !source.draw.is_finite() || !(0.0..=1.0).contains(&source.draw) {
                    return Err("sense received a draw outside [0, 1]".into());
                }
                SenseMessage::Draw { draw: source.draw }
            }
            sys::EECH_SENSE_SIGNATURE => {
                let emitters = if checked_bool("has_emitters", source.has_emitters)? {
                    Some(SenseEmitters {
                        radar_on: checked_bool("radar_on", source.radar_on)?,
                        radar_jammer_on: checked_bool("radar_jammer_on", source.radar_jammer_on)?,
                        infra_red_jammer_on: checked_bool("infra_red_jammer_on", source.infra_red_jammer_on)?,
                    })
                } else {
                    None
                };
                let helicopter = if checked_bool("target_helicopter", source.target_helicopter)? {
                    Some(SenseHelicopter {
                        weapon_bay_closed: checked_bool("weapon_bay_closed", source.weapon_bay_closed)?,
                        stub_wings_attached: checked_bool("stub_wings_attached", source.stub_wings_attached)?,
                        undercarriage_up: checked_bool("undercarriage_up", source.undercarriage_up)?,
                    })
                } else {
                    None
                };
                SenseMessage::Signature(SenseSignature { scan, emitters, helicopter })
            }
            sys::EECH_SENSE_END => SenseMessage::End { scan },
            other => return Err(format!("sense received unknown mode {other}")),
        };
        let reply = (context.sense)(message)?;
        let (state, reason) = match reply.state {
            SenseState::Open => (sys::EECH_SENSE_OPEN, 0),
            SenseState::Accept => (sys::EECH_SENSE_ACCEPT, 0),
            SenseState::Reject(reason) => (
                sys::EECH_SENSE_REJECT,
                match reason {
                    SenseReason::Range => sys::EECH_SENSE_REASON_RANGE,
                    SenseReason::RangeAdjusted => sys::EECH_SENSE_REASON_RANGE_ADJUSTED,
                    SenseReason::Altitude => sys::EECH_SENSE_REASON_ALTITUDE,
                    SenseReason::Tolerance => sys::EECH_SENSE_REASON_TOLERANCE,
                    SenseReason::NotPerceived => sys::EECH_SENSE_REASON_NOT_PERCEIVED,
                },
            ),
            SenseState::NeedDraw => (sys::EECH_SENSE_NEED_DRAW, 0),
            SenseState::NeedSignature => (sys::EECH_SENSE_NEED_SIGNATURE, 0),
            SenseState::Closed => (sys::EECH_SENSE_CLOSED, 0),
        };
        let mut answer = sys::eech_sense_result {
            state,
            reason,
            has_range: c_int::from(reply.target_range.is_some()),
            target_range: reply.target_range.unwrap_or(0.0),
            has_echo: c_int::from(reply.echo.is_some()),
            echo_ceiling: 0.0,
            echo_floor: 0.0,
            echo_adjusted_range: 0.0,
            echo_stealth_factor: 0.0,
            echo_stealth_limit: 0.0,
            echo_tolerance_limit: 0.0,
        };
        if let Some(echo) = reply.echo {
            answer.echo_ceiling = echo.ceiling;
            answer.echo_floor = echo.floor;
            answer.echo_adjusted_range = echo.adjusted_range;
            answer.echo_stealth_factor = echo.stealth_factor;
            answer.echo_stealth_limit = echo.stealth_limit;
            answer.echo_tolerance_limit = echo.tolerance_limit;
        }
        // SAFETY: C supplies a writable result for the duration of this callback.
        unsafe { *result_out = answer };
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
            unsafe { write_callback_text(error, error_size, "sense panicked") };
            1
        }
    }
}

fn aim_host_id(buffer: &[c_char], role: &str) -> Result<Option<String>, String> {
    let bytes: Vec<u8> = buffer.iter().map(|byte| *byte as u8).take_while(|byte| *byte != 0).collect();
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes.len() > sys::EECH_AIM_HOST_ID_MAX {
        return Err(format!("aim received an overlong {role} host identity"));
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| format!("aim received a non-UTF-8 {role} host identity"))
}

/// An identity as the NUL-terminated field the campaign reads: empty for none, otherwise 1 to 128 bytes of UTF-8 with no NUL (the string already is UTF-8).
fn host_id_buffer(id: &Option<String>, role: &str) -> Result<[c_char; sys::EECH_AIM_HOST_ID_MAX + 1], String> {
    let mut buffer = [0 as c_char; sys::EECH_AIM_HOST_ID_MAX + 1];
    if let Some(id) = id {
        let bytes = id.as_bytes();
        if bytes.is_empty() || bytes.len() > sys::EECH_AIM_HOST_ID_MAX || bytes.contains(&0) {
            return Err(format!(
                "aim answered a {role} host identity that is empty, longer than {} bytes or holds a NUL",
                sys::EECH_AIM_HOST_ID_MAX
            ));
        }
        for (slot, byte) in buffer.iter_mut().zip(bytes) {
            *slot = *byte as c_char;
        }
    }
    Ok(buffer)
}

fn weapon_figures(source: &sys::eech_weapon_figures) -> Result<WeaponFigures, String> {
    let name_bytes = source.name.iter().map(|byte| *byte as u8).take_while(|byte| *byte != 0).collect::<Vec<_>>();
    if name_bytes.is_empty() || name_bytes.len() > sys::EECH_WEAPON_NAME_MAX {
        return Err("aim received figures with an invalid weapon name".into());
    }
    let name = String::from_utf8_lossy(&name_bytes).into_owned();
    let floats = [
        source.start_weight,
        source.final_weight,
        source.max_range,
        source.muzzle_velocity,
        source.cruise_velocity,
        source.g_max,
        source.boost_time,
        source.cruise_time,
        source.inhibit_time,
        source.drag_coefficient,
        source.flightpath_deviation,
        source.boost_power,
        source.sustain_power,
        source.sustain_time,
        source.diameter,
        source.max_altitude,
        source.cruise_time_max_error,
        source.detonation_radius,
        source.max_seeker_limit,
        source.max_launch_angle_error,
        source.muzzle_velocity_max_error,
        source.circular_error_probable,
        source.rate_of_fire,
        source.inverse_rate_of_fire,
    ];
    if floats.iter().any(|value| !value.is_finite()) {
        return Err(format!("aim received non-finite figures for weapon {name}"));
    }
    Ok(WeaponFigures {
        name,
        start_weight: source.start_weight,
        final_weight: source.final_weight,
        max_range: source.max_range,
        muzzle_velocity: source.muzzle_velocity,
        cruise_velocity: source.cruise_velocity,
        g_max: source.g_max,
        boost_time: source.boost_time,
        cruise_time: source.cruise_time,
        inhibit_time: source.inhibit_time,
        drag_coefficient: source.drag_coefficient,
        flightpath_deviation: source.flightpath_deviation,
        boost_power: source.boost_power,
        sustain_power: source.sustain_power,
        sustain_time: source.sustain_time,
        diameter: source.diameter,
        max_altitude: source.max_altitude,
        ignore_gravity: source.ignore_gravity,
        flight_profile_or_self_destr: source.flight_profile_or_self_destr,
        parachute_type: source.parachute_type,
        weapon_class: source.weapon_class,
        cruise_time_max_error: source.cruise_time_max_error,
        detonation_radius: source.detonation_radius,
        max_seeker_limit: source.max_seeker_limit,
        max_launch_angle_error: source.max_launch_angle_error,
        survive_ground_impact: source.survive_ground_impact,
        in_flight_collision_check: source.in_flight_collision_check,
        guidance_type: source.guidance_type,
        aiming_type: source.aiming_type,
        decoy_type: source.decoy_type,
        warhead_type: source.warhead_type,
        hellfire_ii: source.hellfire_ii,
        smoke_grenade: source.smoke_grenade,
        submunition_count: source.submunition_count,
        submunition_velocity_factor: source.submunition_velocity_factor,
        muzzle_velocity_max_error: source.muzzle_velocity_max_error,
        circular_error_probable: source.circular_error_probable,
        rate_of_fire: source.rate_of_fire,
        inverse_rate_of_fire: source.inverse_rate_of_fire,
        acquire_forward: source.acquire_forward,
    })
}

fn aim_request(source: &sys::eech_aim_step) -> Result<AimRequest, String> {
    let finite = |name: &str, values: &[f32]| -> Result<(), String> {
        if values.iter().any(|value| !value.is_finite()) {
            Err(format!("non-finite aim {name}"))
        } else {
            Ok(())
        }
    };
    let echo = checked_bool("echo", source.echo)?;
    let world = || -> Result<AimWorld, String> {
        finite("map bound", &source.map_min)?;
        finite("map bound", &source.map_max)?;
        finite("comms weapon lag", &[source.comms_weapon_lag])?;
        Ok(AimWorld {
            map_min: source.map_min,
            map_max: source.map_max,
            comms_weapon_lag: source.comms_weapon_lag,
        })
    };
    let weapon = || -> Result<AimWeapon, String> {
        finite("weapon figure", &[source.cruise_velocity, source.source_velocity])?;
        Ok(AimWeapon {
            weapon: source.weapon,
            present: checked_bool("has_weapon", source.has_weapon)?,
            aiming_type: source.aiming_type,
            cruise_velocity: source.cruise_velocity,
            acquire_forward: checked_bool("acquire_forward", source.acquire_forward)?,
            source_velocity: source.source_velocity,
        })
    };
    let target = || -> Result<AimTarget, String> {
        finite("target position", &source.target_position)?;
        finite("target point", &source.target_point)?;
        finite("target motion vector", &source.target_motion_vector)?;
        Ok(AimTarget {
            position: source.target_position,
            point: source.target_point,
            motion_vector: source.target_motion_vector,
            mobile: checked_bool("target_mobile", source.target_mobile)?,
        })
    };
    let draw = || -> Result<Option<i32>, String> {
        if checked_bool("has_draw", source.has_draw)? {
            if !(0..=32767).contains(&source.draw_sample) {
                return Err(format!("aim draw sample {} is outside [0, 32767]", source.draw_sample));
            }
            Ok(Some(source.draw_sample))
        } else {
            Ok(None)
        }
    };
    let source_id = || aim_host_id(&source.source_host_id, "source");
    let target_id = || aim_host_id(&source.target_host_id, "target");
    Ok(match source.mode {
        sys::EECH_AIM_TABLE | sys::EECH_AIM_RANGE => {
            let figures = weapon_figures(&source.figures)?;
            if source.weapon <= 0 || figures.max_range <= 0.0 || figures.max_range > 200_000.0 {
                return Err(format!("aim received figures of an inconsistent weapon {}", source.weapon));
            }
            let (first, last) = (checked_bool("pass_first", source.pass_first)?, checked_bool("pass_last", source.pass_last)?);
            if source.mode == sys::EECH_AIM_TABLE {
                if echo && (source.echo_cells.is_null() || source.echo_cell_capacity == 0) {
                    return Err("aim was asked to echo a table with no room for it".into());
                }
                AimRequest::Table(AimTable {
                    weapon: source.weapon,
                    figures,
                    first,
                    last,
                    echo,
                    echo_cells: if echo { source.echo_cell_capacity } else { 0 },
                })
            } else {
                AimRequest::Range(AimRange {
                    weapon: source.weapon,
                    figures,
                    first,
                    last,
                    echo,
                })
            }
        }
        sys::EECH_AIM_FIGURES => {
            let figures = weapon_figures(&source.figures)?;
            if source.weapon <= 0 {
                return Err(format!("aim received figures for weapon {}, which is not a weapon", source.weapon));
            }
            AimRequest::Figures(AimFigures {
                weapon: source.weapon,
                figures,
            })
        }
        sys::EECH_AIM_RELEASE => {
            finite("release frame", &[source.delta_time])?;
            if source.weapon <= 0 {
                return Err(format!("aim received a release of weapon {}, which is not a weapon", source.weapon));
            }
            let has_timer = checked_bool("release_has_timer", source.release_has_timer)?;
            AimRequest::Release(AimRelease {
                weapon: source.weapon,
                delta_time: source.delta_time,
                current_time: source.system_time,
                last_shot: has_timer.then_some(source.release_last_shot),
                not_ai: checked_bool("release_not_ai", source.release_not_ai)?,
            })
        }
        sys::EECH_AIM_FORECAST => {
            finite(
                "forecast figure",
                &[
                    source.forecast_figures.burst_duration,
                    source.forecast_player_modifier,
                    source.forecast_damage_modifier,
                    source.forecast_velocity,
                ],
            )?;
            if source.weapon <= 0 {
                return Err(format!("aim received a forecast of weapon {}, which is not a weapon", source.weapon));
            }
            if source.forecast_armour_level < 0 {
                return Err(format!(
                    "aim received a forecast against armour level {}, below zero",
                    source.forecast_armour_level
                ));
            }
            AimRequest::Forecast(AimForecast {
                weapon: source.weapon,
                weapon_warhead: source.forecast_figures.weapon_warhead,
                warhead: source.forecast_figures.warhead,
                damage_capability: source.forecast_figures.damage_capability,
                armor_penetration_capability: source.forecast_figures.armor_penetration_capability,
                burst_duration: source.forecast_figures.burst_duration,
                armour_level: source.forecast_armour_level,
                target_not_ai: checked_bool("forecast_target_not_ai", source.forecast_target_not_ai)?,
                player_modifier: source.forecast_player_modifier,
                damage_modifier: source.forecast_damage_modifier,
                velocity: source.forecast_velocity,
                echo,
            })
        }
        sys::EECH_AIM_DAMAGE => {
            let request = &source.damage;
            finite(
                "damage figure",
                &[
                    request.figures.burst_duration,
                    request.weapon_speed,
                    request.weapon_cruise_velocity,
                    request.damage_modifier,
                    request.armour_front,
                    request.armour_rear,
                    request.armour_side,
                    request.player_modifier,
                ],
            )?;
            finite("damage position", &request.weapon_position)?;
            finite("damage position", &request.target_position)?;
            finite("damage attitude", &request.target_attitude)?;
            if source.weapon <= 0 {
                return Err(format!("aim received a damage question about weapon {}, which is not a weapon", source.weapon));
            }
            if !(0..=2).contains(&request.armour_model) || !(0..=3).contains(&request.critical_model) {
                return Err(format!(
                    "aim received a damage question with armour model {} and critical model {}",
                    request.armour_model, request.critical_model
                ));
            }
            if matches!(request.critical_model, 1 | 3) && request.initial_damage_level <= 0 {
                return Err(format!(
                    "aim received a damage question about a health classified with initial level {}",
                    request.initial_damage_level
                ));
            }
            AimRequest::Damage(AimDamage {
                weapon: source.weapon,
                weapon_warhead: request.figures.weapon_warhead,
                warhead: request.figures.warhead,
                damage_capability: request.figures.damage_capability,
                armor_penetration_capability: request.figures.armor_penetration_capability,
                burst_duration: request.figures.burst_duration,
                weapon_speed: request.weapon_speed,
                weapon_cruise_velocity: request.weapon_cruise_velocity,
                damage_modifier: request.damage_modifier,
                armour_level: request.armour_level,
                armour_easy: checked_bool("damage armour_easy", request.armour_easy)?,
                armour_model: request.armour_model,
                armour_front: request.armour_front,
                armour_rear: request.armour_rear,
                armour_side: request.armour_side,
                weapon_position: request.weapon_position,
                target_position: request.target_position,
                target_attitude: request.target_attitude,
                target_not_ai: checked_bool("damage target_not_ai", request.target_not_ai)?,
                player_modifier: checked_bool("damage player_modifier_valid", request.player_modifier_valid)?.then_some(request.player_modifier),
                has_launcher: checked_bool("damage has_launcher", request.has_launcher)?,
                multiplier: request.multiplier,
                invulnerable: checked_bool("damage invulnerable", request.invulnerable)?,
                cheat_guard: checked_bool("damage cheat_guard", request.cheat_guard)?,
                damage_level: request.damage_level,
                initial_damage_level: request.initial_damage_level,
                critical_model: request.critical_model,
                draw: draw()?,
                echo,
                target_host_id: target_id()?,
            })
        }
        sys::EECH_AIM_HEALTH => {
            let request = &source.health;
            finite("health frame time", &[request.delta_time])?;
            if !(1..=4).contains(&request.op) || !(0..=3).contains(&request.critical_model) || !(0..=2).contains(&request.aircraft_class) {
                return Err(format!(
                    "aim received a health question with op {}, critical model {} and aircraft class {}",
                    request.op, request.critical_model, request.aircraft_class
                ));
            }
            if (matches!(request.critical_model, 1 | 3) || request.op == 2) && request.initial_damage_level <= 0 && request.op != 4 {
                return Err(format!(
                    "aim received a health question about a health with initial level {}",
                    request.initial_damage_level
                ));
            }
            AimRequest::Health(AimHealth {
                op: request.op,
                critical_model: request.critical_model,
                damage_level: request.damage_level,
                initial_damage_level: request.initial_damage_level,
                old_damage_level: request.old_damage_level,
                explosive: checked_bool("health explosive", request.explosive)?,
                flammable: checked_bool("health flammable", request.flammable)?,
                delta_time: request.delta_time,
                aircraft_class: request.aircraft_class,
                self_hit: checked_bool("health self_hit", request.self_hit)?,
                target_not_ai: checked_bool("health target_not_ai", request.target_not_ai)?,
                draw: draw()?,
                target_host_id: target_id()?,
            })
        }
        sys::EECH_AIM_IMPACT => {
            let request = &source.impact;
            finite("impact position", &request.position)?;
            let terrain_known = checked_bool("impact terrain_known", request.terrain_known)?;
            let draws_crater = checked_bool("impact terrain_draws_crater", request.terrain_draws_crater)?;
            let has_target = checked_bool("impact has_target", request.has_target)?;
            let target_alive = checked_bool("impact target_alive", request.target_alive)?;
            if (target_alive && !has_target) || (draws_crater && !terrain_known) {
                return Err("aim received an impact question that is inconsistent about its target or its terrain".into());
            }
            AimRequest::Impact(AimImpact {
                kill_code: request.kill_code,
                has_target,
                target_alive,
                terrain_draws_crater: terrain_known.then_some(draws_crater),
                position: request.position,
                target_host_id: target_id()?,
            })
        }
        sys::EECH_AIM_BLAST => {
            let request = &source.blast;
            finite("blast position", &request.position)?;
            let window_valid = checked_bool("blast window_valid", request.window_valid)?;
            let window_end = checked_bool("blast window_end", request.window_end)?;
            if request.sector_side_length <= 0
                || request.min_x_sector > request.max_x_sector
                || request.min_z_sector > request.max_z_sector
                || !(0..=sys::EECH_BLAST_WINDOW_MAX as c_int).contains(&request.window_count)
                || (!window_valid && (request.window_count != 0 || window_end || request.window_after != 0))
                || request.window_after < 0
            {
                return Err("aim received a blast question with inconsistent sector figures or window".into());
            }
            let window = if window_valid {
                let mut entries = Vec::with_capacity(request.window_count as usize);
                for entry in request.window.iter().take(request.window_count as usize) {
                    finite("blast object position", &entry.position)?;
                    if entry.handle <= 0 {
                        return Err(format!("aim received a blast object with handle {}", entry.handle));
                    }
                    entries.push(AimBlastEntry {
                        handle: entry.handle,
                        entity_type: entry.kind,
                        readable: checked_bool("blast readable", entry.readable)?,
                        alive: checked_bool("blast alive", entry.alive)?,
                        has_position: checked_bool("blast has_position", entry.has_position)?,
                        rocket: checked_bool("blast rocket", entry.rocket)?,
                        excluded: checked_bool("blast excluded", entry.excluded)?,
                        position: entry.position,
                        host_id: aim_host_id(&entry.host_id, "blast object")?,
                    });
                }
                Some(AimBlastWindow {
                    sx: request.window_sx,
                    sz: request.window_sz,
                    after: request.window_after,
                    end: window_end,
                    entries,
                })
            } else {
                None
            };
            AimRequest::Blast(AimBlast {
                position: request.position,
                damage_capability: request.damage_capability,
                warhead_type: request.warhead_type,
                has_target: checked_bool("blast has_target", request.has_target)?,
                sector_side_length: request.sector_side_length,
                min_x_sector: request.min_x_sector,
                max_x_sector: request.max_x_sector,
                min_z_sector: request.min_z_sector,
                max_z_sector: request.max_z_sector,
                window,
                echo,
            })
        }
        sys::EECH_AIM_IDENTIFY => {
            let request = &source.identify;
            finite("identify position", &request.position)?;
            if request.entity_type <= 0 || request.sub_type < 0 || request.damage_level < 0 || request.initial_damage_level < 0 || request.serial <= 0 {
                return Err("aim received an identify question with inconsistent figures".into());
            }
            AimRequest::Identify(AimIdentify {
                entity_type: request.entity_type,
                sub_type: request.sub_type,
                side: request.side,
                alive: checked_bool("identify alive", request.alive)?,
                damage_level: request.damage_level,
                initial_damage_level: request.initial_damage_level,
                position: request.position,
                serial: request.serial,
            })
        }
        sys::EECH_AIM_BRIDGE => {
            let request = &source.bridge;
            finite("bridge position", &request.position)?;
            finite("bridge figure", &[request.delta_time, request.terrain_height, request.segment_length])?;
            if !(1..=6).contains(&request.op) || request.segment_length < 0.0 {
                return Err(format!(
                    "aim received a bridge question with op {} and segment length {}",
                    request.op, request.segment_length
                ));
            }
            let terrain_known = checked_bool("bridge terrain_known", request.terrain_known)?;
            let window_valid = checked_bool("bridge window_valid", request.window_valid)?;
            let window_end = checked_bool("bridge window_end", request.window_end)?;
            if !(0..=sys::EECH_BLAST_WINDOW_MAX as c_int).contains(&request.window_count)
                || request.window_after < 0
                || (request.op != 5 && (window_valid || request.window_count != 0 || window_end || request.window_after != 0))
                || (request.op == 5
                    && (request.sector_side_length <= 0 || request.min_x_sector > request.max_x_sector || request.min_z_sector > request.max_z_sector))
                || (!window_valid && (request.window_count != 0 || window_end || request.window_after != 0))
            {
                return Err("aim received a bridge question with inconsistent sector figures or window".into());
            }
            let window = if window_valid {
                let mut entries = Vec::with_capacity(request.window_count as usize);
                for entry in request.window.iter().take(request.window_count as usize) {
                    finite("bridge object position", &entry.position)?;
                    if entry.handle <= 0 {
                        return Err(format!("aim received a bridge object with handle {}", entry.handle));
                    }
                    entries.push(AimBlastEntry {
                        handle: entry.handle,
                        entity_type: entry.kind,
                        readable: checked_bool("bridge readable", entry.readable)?,
                        alive: checked_bool("bridge alive", entry.alive)?,
                        has_position: checked_bool("bridge has_position", entry.has_position)?,
                        rocket: checked_bool("bridge rocket", entry.rocket)?,
                        excluded: checked_bool("bridge excluded", entry.excluded)?,
                        position: entry.position,
                        host_id: aim_host_id(&entry.host_id, "bridge object")?,
                    });
                }
                Some(AimBlastWindow {
                    sx: request.window_sx,
                    sz: request.window_sz,
                    after: request.window_after,
                    end: window_end,
                    entries,
                })
            } else {
                None
            };
            AimRequest::Bridge(AimBridge {
                op: request.op,
                segment_kind: request.segment_kind,
                alive: checked_bool("bridge alive", request.alive)?,
                server: checked_bool("bridge server", request.server)?,
                pred_present: checked_bool("bridge pred_present", request.pred_present)?,
                pred_alive: checked_bool("bridge pred_alive", request.pred_alive)?,
                succ_present: checked_bool("bridge succ_present", request.succ_present)?,
                succ_alive: checked_bool("bridge succ_alive", request.succ_alive)?,
                which: i32::from(checked_bool("bridge which", request.which)?),
                neighbour_present: checked_bool("bridge neighbour_present", request.neighbour_present)?,
                neighbour_kind: request.neighbour_kind,
                neighbour_alive: checked_bool("bridge neighbour_alive", request.neighbour_alive)?,
                position: request.position,
                delta_time: request.delta_time,
                terrain: terrain_known.then_some((request.terrain_height, checked_bool("bridge terrain_water", request.terrain_water)?)),
                segment_length: request.segment_length,
                sector_side_length: request.sector_side_length,
                min_x_sector: request.min_x_sector,
                max_x_sector: request.max_x_sector,
                min_z_sector: request.min_z_sector,
                max_z_sector: request.max_z_sector,
                window,
            })
        }
        sys::EECH_AIM_ANIMATE => {
            finite("animation figure", &[source.animate_lifetime, source.animate_motion_y])?;
            if source.weapon <= 0 {
                return Err(format!("aim received an animation of weapon {}, which is not a weapon", source.weapon));
            }
            if !(0..5).contains(&source.animate_status) {
                return Err(format!("aim received a parachute state {}", source.animate_status));
            }
            AimRequest::Animate(AimAnimate {
                weapon: source.weapon,
                parachute_status: source.animate_status,
                weapon_on_ground: checked_bool("animate_on_ground", source.animate_on_ground)?,
                weapon_lifetime: source.animate_lifetime,
                motion_vector_y: source.animate_motion_y,
            })
        }
        sys::EECH_AIM_WEAPON_STEP | sys::EECH_AIM_WEAPON_BIRTH | sys::EECH_AIM_WEAPON_DECOY | sys::EECH_AIM_WEAPON_SEDUCE | sys::EECH_AIM_WEAPON_REACQUIRE => {
            let state = &source.weapon_state;
            finite("seduction setting", &[state.coefficient])?;
            finite("seduction setting", &state.effectiveness)?;
            finite("weapon position", &state.position)?;
            finite("weapon attitude", &state.attitude)?;
            finite("weapon motion vector", &state.motion_vector)?;
            finite("weapon state", &[state.velocity, state.weapon_lifetime, state.decoy_timer, source.delta_time])?;
            finite("map bound", &source.map_min)?;
            finite("map bound", &source.map_max)?;
            if source.weapon <= 0 {
                return Err(format!("aim received a step of weapon {}, which is not a weapon", source.weapon));
            }
            if !(0..=sys::EECH_STEP_ANSWERS_MAX as c_int).contains(&source.answer_count)
                || !(0..=sys::EECH_STEP_DRAWS_MAX as c_int).contains(&source.draw_count)
            {
                return Err("aim received a weapon step with an answer or draw count out of range".into());
            }
            let draws = source.draws[..source.draw_count as usize].to_vec();
            if draws.iter().chain(state.entity_samples.iter()).any(|sample| !(0..=32767).contains(sample)) {
                return Err("aim received a weapon step with a random sample outside [0, 32767]".into());
            }
            let mut answers = Vec::with_capacity(source.answer_count as usize);
            for answer in &source.answers[..source.answer_count as usize] {
                finite("lookup answer", &answer.f)?;
                answers.push(AimEvent {
                    kind: answer.kind,
                    a: answer.a,
                    b: answer.b,
                    c: answer.c,
                    d: answer.d,
                    f: answer.f,
                });
            }
            AimRequest::WeaponStep(AimWeaponStep {
                kind: match source.mode {
                    sys::EECH_AIM_WEAPON_BIRTH => AimStepKind::Birth,
                    sys::EECH_AIM_WEAPON_DECOY => AimStepKind::Decoy,
                    sys::EECH_AIM_WEAPON_SEDUCE => AimStepKind::Seduce,
                    sys::EECH_AIM_WEAPON_REACQUIRE => AimStepKind::Reacquire,
                    _ => AimStepKind::Step,
                },
                echo,
                weapon: source.weapon,
                delta_time: source.delta_time,
                system_time: source.system_time,
                map_min: source.map_min,
                map_max: source.map_max,
                state: AimWeaponState {
                    position: state.position,
                    attitude: state.attitude,
                    motion_vector: state.motion_vector,
                    velocity: state.velocity,
                    weapon_lifetime: state.weapon_lifetime,
                    decoy_timer: state.decoy_timer,
                    missile_phase: state.missile_phase,
                    loal_mode: checked_bool("loal_mode", state.loal_mode)?,
                    weapon_on_ground: checked_bool("weapon_on_ground", state.weapon_on_ground)?,
                    parachute_status: state.parachute_status,
                    kill_code: state.kill_code,
                    entity_samples: state.entity_samples,
                    has_launcher: checked_bool("has_launcher", state.has_launcher)?,
                    has_target: checked_bool("has_target", state.has_target)?,
                    server: checked_bool("server", state.server)?,
                    submunition: checked_bool("submunition", state.submunition)?,
                    other_weapon: state.other_weapon,
                    coefficient: state.coefficient,
                    effectiveness: state.effectiveness,
                },
                answers,
                draws,
            })
        }
        sys::EECH_AIM_READY_UPDATE => {
            finite("ready figure", &[source.ready_state, source.ready_rate, source.delta_time])?;
            AimRequest::ReadyUpdate(AimReadyUpdate {
                source: source_id()?,
                state: source.ready_state,
                rate: source.ready_rate,
                delta_time: source.delta_time,
            })
        }
        sys::EECH_AIM_READY_COMMAND => {
            finite("ready figure", &[source.ready_state])?;
            AimRequest::ReadyCommand(AimReadyCommand {
                source: source_id()?,
                state: source.ready_state,
                open: checked_bool("ready_open", source.ready_open)?,
            })
        }
        sys::EECH_AIM_REQUIRED => {
            finite("device attitude", &source.device_attitude)?;
            finite("intercept vector", &source.intercept_vector)?;
            let limits = if checked_bool("unbounded", source.unbounded)? {
                None
            } else {
                finite("device limit", &[source.heading_min, source.heading_max, source.pitch_min, source.pitch_max])?;
                Some(AimLimits {
                    heading_min: source.heading_min,
                    heading_max: source.heading_max,
                    pitch_min: source.pitch_min,
                    pitch_max: source.pitch_max,
                    climb: checked_bool("climb", source.climb)?,
                })
            };
            let tracking = if checked_bool("has_tracking", source.has_tracking)? {
                finite("device position", &source.device_position)?;
                finite("tracking point", &source.tracking_point)?;
                finite("tracking pitch", &[source.track_pitch])?;
                Some(AimTrack {
                    device_position: source.device_position,
                    point: source.tracking_point,
                    pitch: source.track_pitch,
                })
            } else {
                None
            };
            AimRequest::Required(AimRequired {
                source: source_id()?,
                device_attitude: source.device_attitude,
                intercept_vector: source.intercept_vector,
                limits,
                tracking,
            })
        }
        sys::EECH_AIM_STEP => {
            finite(
                "device figure",
                &[
                    source.current_heading,
                    source.current_pitch,
                    source.required_heading,
                    source.required_pitch,
                    source.heading_min,
                    source.heading_max,
                    source.heading_rate,
                    source.pitch_rate,
                    source.delta_time,
                ],
            )?;
            AimRequest::Step(AimStep {
                source: source_id()?,
                current_heading: source.current_heading,
                current_pitch: source.current_pitch,
                required_heading: source.required_heading,
                required_pitch: source.required_pitch,
                heading_min: source.heading_min,
                heading_max: source.heading_max,
                heading_rate: source.heading_rate,
                pitch_rate: source.pitch_rate,
                delta_time: source.delta_time,
            })
        }
        sys::EECH_AIM_VECTORS => {
            finite("device position", &source.device_position)?;
            finite("device attitude", &source.device_attitude)?;
            let tracking = if checked_bool("has_tracking", source.has_tracking)? {
                finite("tracking point", &source.tracking_point)?;
                Some(source.tracking_point)
            } else {
                None
            };
            AimRequest::Vectors(AimVectors {
                echo,
                source: source_id()?,
                target_id: target_id()?,
                world: world()?,
                device_position: source.device_position,
                device_attitude: source.device_attitude,
                weapon: if tracking.is_none() { Some(weapon()?) } else { None },
                target: if tracking.is_none() { Some(target()?) } else { None },
                tracking,
                source_gunship: checked_bool("source_gunship", source.source_gunship)?,
                draw: draw()?,
            })
        }
        sys::EECH_AIM_LEAD => {
            finite("device position", &source.device_position)?;
            finite("parent velocity", &[source.parent_velocity])?;
            AimRequest::Lead(AimLead {
                echo,
                source: source_id()?,
                target_id: target_id()?,
                world: world()?,
                weapon: source.weapon,
                source_gunship: checked_bool("source_gunship", source.source_gunship)?,
                parent_velocity: source.parent_velocity,
                device_position: source.device_position,
                target: target()?,
                draw: draw()?,
            })
        }
        sys::EECH_AIM_BALLISTIC => {
            finite("ballistic figure", &[source.range, source.height_diff_or_pitch, source.parent_velocity])?;
            AimRequest::Ballistic(AimBallistic {
                echo,
                weapon: source.weapon,
                range: source.range,
                height_diff_or_pitch: source.height_diff_or_pitch,
                simplified: checked_bool("simplified", source.simplified)?,
                fixed_pitch: checked_bool("fixed_pitch", source.fixed_pitch)?,
                parent_velocity: source.parent_velocity,
            })
        }
        sys::EECH_AIM_INTERCEPT => {
            finite("device position", &source.device_position)?;
            AimRequest::Intercept(AimIntercept {
                echo,
                source: source_id()?,
                target_id: target_id()?,
                weapon: weapon()?,
                device_position: source.device_position,
                target: target()?,
            })
        }
        other => return Err(format!("aim received unknown mode {other}")),
    })
}

/// Writes the reference host's echo into the room the campaign gave for it.
fn copy_echo(source: &sys::eech_aim_step, events: &[AimEvent], answer: &mut sys::eech_aim_result) -> Result<(), String> {
    if source.echo_out.is_null() || source.echo_capacity <= 0 {
        return Err("aim sent an echo nobody asked for".into());
    }
    if events.len() > source.echo_capacity as usize {
        return Err(format!(
            "aim echoed {} events, more than the {} it can carry",
            events.len(),
            source.echo_capacity
        ));
    }
    for (index, event) in events.iter().enumerate() {
        // SAFETY: the campaign gave room for `echo_capacity` events at `echo_out` for this call, and `index` is below it.
        unsafe {
            source.echo_out.add(index).write(sys::eech_aim_event {
                kind: event.kind,
                a: event.a,
                b: event.b,
                c: event.c,
                d: event.d,
                f: event.f,
            });
        }
    }
    answer.has_echo = 1;
    answer.echo_count = events.len() as c_int;
    Ok(())
}

fn aim_answer(source: &sys::eech_aim_step, reply: &AimReply) -> Result<sys::eech_aim_result, String> {
    // SAFETY: the result is plain data; all-zero is a valid value of every field (the echo pointer is the campaign's, set after the call).
    let mut answer: sys::eech_aim_result = unsafe { std::mem::zeroed() };
    let event = |event: &AimEvent| sys::eech_aim_event {
        kind: event.kind,
        a: event.a,
        b: event.b,
        c: event.c,
        d: event.d,
        f: event.f,
    };
    if reply.need_draw {
        if !matches!(
            source.mode,
            sys::EECH_AIM_VECTORS
                | sys::EECH_AIM_LEAD
                | sys::EECH_AIM_DAMAGE
                | sys::EECH_AIM_HEALTH
                | sys::EECH_AIM_WEAPON_STEP
                | sys::EECH_AIM_WEAPON_BIRTH
                | sys::EECH_AIM_WEAPON_DECOY
                | sys::EECH_AIM_WEAPON_SEDUCE
                | sys::EECH_AIM_WEAPON_REACQUIRE
        ) {
            return Err("aim asked for a draw in answer to a request that cannot take one".into());
        }
        answer.state = sys::EECH_AIM_NEED_DRAW;
        return Ok(answer);
    }
    if reply.need_lookup && source.mode == sys::EECH_AIM_IMPACT {
        if !reply.impact_need_terrain {
            return Err("aim asked an impact question for lookups other than the terrain".into());
        }
        answer.state = sys::EECH_AIM_NEED_LOOKUP;
        answer.impact.need_terrain = 1;
        return Ok(answer);
    }
    if source.mode == sys::EECH_AIM_IDENTIFY {
        if reply.need_draw || reply.need_lookup || reply.echo.is_some() {
            return Err("aim answered an identify question with a draw, a lookup or an echo, which it does not take".into());
        }
        if reply.identify_host_id.is_none() {
            return Err("aim issued no identity in answer to an identify question".into());
        }
        answer.state = sys::EECH_AIM_DONE;
        answer.identify.host_id = host_id_buffer(&reply.identify_host_id, "issued")?;
        return Ok(answer);
    }
    if source.mode == sys::EECH_AIM_BRIDGE {
        if reply.need_lookup {
            answer.state = sys::EECH_AIM_NEED_LOOKUP;
            if reply.bridge_need_terrain {
                answer.bridge.need_terrain = 1;
            } else if let Some((sx, sz, after)) = reply.bridge_need {
                answer.bridge.need_sx = sx;
                answer.bridge.need_sz = sz;
                answer.bridge.need_after = after;
            } else {
                return Err("aim asked a bridge question for a lookup that is neither the terrain nor a window".into());
            }
            return Ok(answer);
        }
        answer.state = sys::EECH_AIM_DONE;
        answer.bridge = sys::eech_bridge_result {
            kill: c_int::from(reply.bridge_kill),
            variant: reply.bridge_variant,
            notify: c_int::from(reply.bridge_notify),
            need_terrain: 0,
            fell: c_int::from(reply.bridge_fell),
            dropped_y: reply.bridge_dropped_y,
            floor_height: reply.bridge_floor_height,
            landed: c_int::from(reply.bridge_landed),
            landed_y: reply.bridge_landed_y,
            remove: c_int::from(reply.bridge_remove),
            need_sx: 0,
            need_sz: 0,
            need_after: 0,
            action: reply.bridge_action,
            handle: reply.bridge_handle,
            target_host_id: host_id_buffer(&reply.bridge_target_host_id, "bridge target")?,
        };
        if reply.echo.is_some() {
            return Err("aim sent an echo to a bridge question, which has none".into());
        }
        return Ok(answer);
    }
    if source.mode == sys::EECH_AIM_BLAST {
        answer.blast.radius = reply.blast_radius;
        if reply.need_lookup {
            let Some((sx, sz, after)) = reply.blast_need else {
                return Err("aim asked a blast question for a lookup that is not a window".into());
            };
            answer.state = sys::EECH_AIM_NEED_LOOKUP;
            answer.blast.need_sx = sx;
            answer.blast.need_sz = sz;
            answer.blast.need_after = after;
            if let Some(events) = &reply.echo {
                copy_echo(source, events, &mut answer)?;
            }
            return Ok(answer);
        }
        answer.blast.action = reply.blast_action;
        answer.blast.handle = reply.blast_handle;
        answer.blast.target_host_id = host_id_buffer(&reply.blast_target_host_id, "blast target")?;
        answer.blast.modifier = reply.blast_modifier;
    }
    if reply.need_lookup {
        if !matches!(
            source.mode,
            sys::EECH_AIM_WEAPON_STEP | sys::EECH_AIM_WEAPON_BIRTH | sys::EECH_AIM_WEAPON_DECOY | sys::EECH_AIM_WEAPON_SEDUCE | sys::EECH_AIM_WEAPON_REACQUIRE
        ) {
            return Err("aim asked for lookups in answer to a request that cannot take them".into());
        }
        if reply.lookups.is_empty() || reply.lookups.len() > sys::EECH_STEP_LOOKUPS_MAX {
            return Err(format!(
                "aim asked for {} lookups, not 1 to {}",
                reply.lookups.len(),
                sys::EECH_STEP_LOOKUPS_MAX
            ));
        }
        answer.state = sys::EECH_AIM_NEED_LOOKUP;
        answer.lookup_count = reply.lookups.len() as c_int;
        for (slot, lookup) in answer.lookups.iter_mut().zip(&reply.lookups) {
            *slot = event(lookup);
        }
        return Ok(answer);
    }
    if source.mode == sys::EECH_AIM_RELEASE {
        if reply.release_last_shot.is_some() && source.release_has_timer == 0 {
            return Err("aim wrote a last-shot clock for a launcher that has none".into());
        }
        answer.release_refuse = c_int::from(reply.release_refuse);
        answer.release_burst_size = reply.release_burst_size;
        if let Some(last_shot) = reply.release_last_shot {
            answer.release_write_last_shot = 1;
            answer.release_last_shot = last_shot;
        }
    }
    if source.mode == sys::EECH_AIM_HEALTH {
        answer.health = sys::eech_health_result {
            critical_class: reply.health_critical_class,
            crossed: c_int::from(reply.health_crossed),
            recovered: c_int::from(reply.health_recovered),
            eject: c_int::from(reply.health_eject),
            eligible: c_int::from(reply.health_eligible),
            rolled: c_int::from(reply.health_rolled),
            hit: c_int::from(reply.health_hit),
            new_level: reply.health_new_level,
            fatal: c_int::from(reply.health_fatal),
        };
    }
    if source.mode == sys::EECH_AIM_IMPACT {
        answer.impact = sys::eech_impact_result {
            need_terrain: 0,
            invalid: c_int::from(reply.impact_invalid),
            crater: c_int::from(reply.impact_crater),
            direct_hit: c_int::from(reply.impact_direct_hit),
            blasts: reply.impact_blasts,
            exclude_target: c_int::from(reply.impact_exclude_target),
        };
    }
    if source.mode == sys::EECH_AIM_DAMAGE {
        answer.damage = sys::eech_damage_result {
            exited: c_int::from(reply.damage_exited),
            capability: reply.damage_capability,
            written: c_int::from(reply.damage_written),
            new_level: reply.damage_new_level,
            critical_before: reply.damage_critical_before,
            critical_after: reply.damage_critical_after,
            bad_modifier: c_int::from(reply.damage_bad_modifier),
        };
    }
    if source.mode == sys::EECH_AIM_FORECAST {
        if reply.forecast_capability < 0 {
            return Err(format!("aim answered a forecast with a negative capability {}", reply.forecast_capability));
        }
        answer.forecast_capability = reply.forecast_capability;
    }
    if source.mode == sys::EECH_AIM_ANIMATE {
        answer.animate_status = reply.animate_status;
        if let Some(lifetime) = reply.animate_lifetime {
            if !lifetime.is_finite() {
                return Err("aim answered a non-finite lifetime".into());
            }
            answer.animate_write_lifetime = 1;
            answer.animate_lifetime = lifetime;
        }
    }
    if !reply.actions.is_empty() {
        if !matches!(
            source.mode,
            sys::EECH_AIM_WEAPON_STEP | sys::EECH_AIM_WEAPON_BIRTH | sys::EECH_AIM_WEAPON_DECOY | sys::EECH_AIM_WEAPON_SEDUCE | sys::EECH_AIM_WEAPON_REACQUIRE
        ) {
            return Err("aim answered a request that cannot carry actions with some".into());
        }
        if reply.actions.len() > sys::EECH_STEP_ACTIONS_MAX {
            return Err(format!(
                "aim answered with {} actions, more than the {} a step can carry",
                reply.actions.len(),
                sys::EECH_STEP_ACTIONS_MAX
            ));
        }
        answer.action_count = reply.actions.len() as c_int;
        for (slot, action) in answer.actions.iter_mut().zip(&reply.actions) {
            *slot = event(action);
        }
    }
    answer.state = sys::EECH_AIM_DONE;
    answer.result = c_int::from(reply.result);
    answer.ready_state = reply.ready_state;
    answer.phase = reply.phase;
    answer.required_heading = reply.required_heading;
    answer.required_pitch = reply.required_pitch;
    answer.delta_heading = reply.delta_heading;
    answer.delta_pitch = reply.delta_pitch;
    answer.new_heading = reply.new_heading;
    answer.new_pitch = reply.new_pitch;
    answer.weapon_vector = reply.weapon_vector;
    answer.target_vector = reply.target_vector;
    answer.intercept_vector = reply.intercept_vector;
    answer.intercept_range = reply.intercept_range;
    answer.intercept_point = reply.intercept_point;
    answer.angle = reply.angle;
    if let Some(time_of_flight) = reply.time_of_flight {
        if source.mode != sys::EECH_AIM_LEAD && source.mode != sys::EECH_AIM_BALLISTIC {
            return Err("aim answered a request that cannot carry a flight time with one".into());
        }
        if !time_of_flight.is_finite() {
            return Err("aim answered a non-finite flight time".into());
        }
        answer.time_of_flight = time_of_flight;
        answer.has_time_of_flight = 1;
    }
    if let Some(events) = &reply.echo {
        copy_echo(source, events, &mut answer)?;
    }
    if let Some(cells) = &reply.table_cells {
        if source.mode != sys::EECH_AIM_TABLE || source.echo_cells.is_null() {
            return Err("aim echoed table cells nobody asked for".into());
        }
        if cells.len() > source.echo_cell_capacity {
            return Err(format!(
                "aim echoed {} table cells, more than the {} it can carry",
                cells.len(),
                source.echo_cell_capacity
            ));
        }
        // SAFETY: the campaign gave room for `echo_cell_capacity` floats at `echo_cells` for this call, and the count was checked against it.
        unsafe { std::ptr::copy_nonoverlapping(cells.as_ptr(), source.echo_cells, cells.len()) };
        answer.echo_cell_count = cells.len();
    }
    Ok(answer)
}

unsafe extern "C" fn aim<F, G, H, M, L, D, A>(
    step: *const sys::eech_aim_step,
    result_out: *mut sys::eech_aim_result,
    error: *mut c_char,
    error_size: usize,
    user: *mut c_void,
) -> c_int
where
    F: FnMut(MemberDescriptor) -> Result<String, String>,
    G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
    H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
    M: FnMut(MovementStep) -> Result<MovementResult, String>,
    L: FnMut(LosMessage) -> Result<LosReply, String>,
    D: FnMut(SenseMessage) -> Result<SenseReply, String>,
    A: FnMut(AimRequest) -> Result<AimReply, String>,
{
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        if step.is_null() || result_out.is_null() || user.is_null() {
            return Err("invalid native aim callback context".into());
        }
        // SAFETY: C supplies all three pointers for the duration of this callback.
        let (source, context) = unsafe { (&*step, &mut *(user as *mut HostContext<'_, F, G, H, M, L, D, A>)) };
        let request = aim_request(source)?;
        let reply = (context.aim)(request)?;
        let answer = aim_answer(source, &reply)?;
        // SAFETY: C supplies a writable result for the duration of this callback.
        unsafe { std::ptr::write(result_out, answer) };
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
            unsafe { write_callback_text(error, error_size, "aim panicked") };
            1
        }
    }
}

unsafe extern "C" fn move_vehicle<F, G, H, M, L, D, A>(
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
    L: FnMut(LosMessage) -> Result<LosReply, String>,
    D: FnMut(SenseMessage) -> Result<SenseReply, String>,
    A: FnMut(AimRequest) -> Result<AimReply, String>,
{
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        if step.is_null() || result_out.is_null() || user.is_null() {
            return Err("invalid native move_vehicle callback context".into());
        }
        // SAFETY: C supplies all three pointers for the duration of this callback.
        let (source, context) = unsafe { (&*step, &mut *(user as *mut HostContext<'_, F, G, H, M, L, D, A>)) };
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
            sys::EECH_MOVEMENT_PERSON_FACE => MovementMode::PersonFace,
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
            ("pf_target", &source.pf_target[..]),
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
                | MovementMode::PersonFace
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
            pf_target: source.pf_target,
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

#[allow(clippy::too_many_arguments)]
fn with_host<F, G, H, M, L, D, A>(
    bind_mobile_fn: &mut F,
    observe_loss_fn: &mut G,
    supply_reached_fn: &mut H,
    move_vehicle_fn: &mut M,
    line_of_sight_fn: &mut L,
    sense_fn: &mut D,
    aim_fn: &mut A,
    call: impl FnOnce(*const sys::eech_engine_host) -> c_int,
) -> Result<(), EngineError>
where
    F: FnMut(MemberDescriptor) -> Result<String, String>,
    G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
    H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
    M: FnMut(MovementStep) -> Result<MovementResult, String>,
    L: FnMut(LosMessage) -> Result<LosReply, String>,
    D: FnMut(SenseMessage) -> Result<SenseReply, String>,
    A: FnMut(AimRequest) -> Result<AimReply, String>,
{
    let mut context = HostContext {
        bind_mobile: bind_mobile_fn,
        observe_loss: observe_loss_fn,
        supply_reached: supply_reached_fn,
        move_vehicle: move_vehicle_fn,
        line_of_sight: line_of_sight_fn,
        sense: sense_fn,
        aim: aim_fn,
    };
    let host = sys::eech_engine_host {
        bind_mobile: Some(bind_mobile::<F, G, H, M, L, D, A>),
        observe_loss: Some(observe_loss::<F, G, H, M, L, D, A>),
        supply_reached: Some(supply_reached::<F, G, H, M, L, D, A>),
        move_vehicle: Some(move_vehicle::<F, G, H, M, L, D, A>),
        line_of_sight: Some(line_of_sight::<F, G, H, M, L, D, A>),
        sense: Some(sense::<F, G, H, M, L, D, A>),
        aim: Some(aim::<F, G, H, M, L, D, A>),
        terrain: Some(terrain_callback),
        user: (&mut context as *mut HostContext<'_, F, G, H, M, L, D, A>).cast::<c_void>(),
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
    #[allow(clippy::too_many_arguments)]
    pub fn boot<F, G, H, M, L, D, A>(
        config: &EngineConfig,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
        line_of_sight_fn: &mut L,
        sense_fn: &mut D,
        aim_fn: &mut A,
        terrain: TerrainProvider,
    ) -> Result<Engine, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
        L: FnMut(LosMessage) -> Result<LosReply, String>,
        D: FnMut(SenseMessage) -> Result<SenseReply, String>,
        A: FnMut(AimRequest) -> Result<AimReply, String>,
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
            winter: c_int::from(config.winter),
            arguments: argv.as_ptr(),
        };
        // The host's terrain provider is installed before the engine boots: the campaign asks about terrain while it loads and places its forces, and
        // from every entry afterwards. It is kept for the life of the process (the engine is one per process).
        TERRAIN.with(|slot| *slot.borrow_mut() = Some(terrain));
        // SAFETY: every config pointer and the temporary host callback are valid for the call.
        with_host(
            bind_mobile_fn,
            observe_loss_fn,
            supply_reached_fn,
            move_vehicle_fn,
            line_of_sight_fn,
            sense_fn,
            aim_fn,
            |host| unsafe { sys::eech_engine_boot(&c, host) },
        )?;
        Ok(Engine {
            _not_send: std::marker::PhantomData,
        })
    }

    /// advances simulated time and runs one iteration of EECH's flight loop
    #[allow(clippy::too_many_arguments)]
    pub fn frame<F, G, H, M, L, D, A>(
        &self,
        milliseconds: u32,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
        line_of_sight_fn: &mut L,
        sense_fn: &mut D,
        aim_fn: &mut A,
    ) -> Result<(), EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
        L: FnMut(LosMessage) -> Result<LosReply, String>,
        D: FnMut(SenseMessage) -> Result<SenseReply, String>,
        A: FnMut(AimRequest) -> Result<AimReply, String>,
    {
        // SAFETY: the temporary host callback is valid for the call.
        with_host(
            bind_mobile_fn,
            observe_loss_fn,
            supply_reached_fn,
            move_vehicle_fn,
            line_of_sight_fn,
            sense_fn,
            aim_fn,
            |host| unsafe { sys::eech_engine_frame(milliseconds, host) },
        )
    }

    /// TEST-ONLY (feature `dependent-fixture`, EEDCS-40): re-links `child` under `parent` as a movement dependent
    /// (`op` 1), moves `parent` through the campaign's own position setter (`op` 2, `position`), or runs a ship
    /// parent's own sinking step (`op` 3). The campaign's genuine notification handlers do the rest; the host answers
    /// the movement steps they send. Not part of the shipping module.
    #[cfg(feature = "dependent-fixture")]
    #[allow(clippy::too_many_arguments)]
    pub fn dependent_fixture<F, G, H, M, L, D, A>(
        &self,
        op: i32,
        parent: i32,
        child: i32,
        position: [f64; 3],
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
        line_of_sight_fn: &mut L,
        sense_fn: &mut D,
        aim_fn: &mut A,
    ) -> Result<(), EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
        L: FnMut(LosMessage) -> Result<LosReply, String>,
        D: FnMut(SenseMessage) -> Result<SenseReply, String>,
        A: FnMut(AimRequest) -> Result<AimReply, String>,
    {
        // SAFETY: the position array and the temporary host callback are valid for the call.
        with_host(
            bind_mobile_fn,
            observe_loss_fn,
            supply_reached_fn,
            move_vehicle_fn,
            line_of_sight_fn,
            sense_fn,
            aim_fn,
            |host| unsafe { sys::eech_engine_dependent_fixture(op, parent, child, position.as_ptr(), host) },
        )
    }

    /// TEST-ONLY (feature `loss-fixture`): ends one live hosted member through the campaign's own kill path.
    #[cfg(feature = "loss-fixture")]
    #[allow(clippy::too_many_arguments)]
    pub fn loss_fixture<F, G, H, M, L, D, A>(
        &self,
        entity_id: i32,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
        line_of_sight_fn: &mut L,
        sense_fn: &mut D,
        aim_fn: &mut A,
    ) -> Result<(), EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
        L: FnMut(LosMessage) -> Result<LosReply, String>,
        D: FnMut(SenseMessage) -> Result<SenseReply, String>,
        A: FnMut(AimRequest) -> Result<AimReply, String>,
    {
        // SAFETY: the temporary host callback is valid for the call.
        with_host(
            bind_mobile_fn,
            observe_loss_fn,
            supply_reached_fn,
            move_vehicle_fn,
            line_of_sight_fn,
            sense_fn,
            aim_fn,
            |host| unsafe { sys::eech_engine_loss_fixture(entity_id, host) },
        )
    }

    /// TEST-ONLY (feature `loss-fixture`): releases the tasks that target one hosted member's group, through the campaign's own task
    /// completion handlers, and returns how many it notified.  Setup for the fixed-wing witness; the group itself is not ended.
    #[cfg(feature = "loss-fixture")]
    #[allow(clippy::too_many_arguments)]
    pub fn loss_release<F, G, H, M, L, D, A>(
        &self,
        entity_id: i32,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
        line_of_sight_fn: &mut L,
        sense_fn: &mut D,
        aim_fn: &mut A,
    ) -> Result<i32, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
        L: FnMut(LosMessage) -> Result<LosReply, String>,
        D: FnMut(SenseMessage) -> Result<SenseReply, String>,
        A: FnMut(AimRequest) -> Result<AimReply, String>,
    {
        let mut released = 0i32;
        // SAFETY: the out pointer and the temporary host callback are valid for the call.
        with_host(
            bind_mobile_fn,
            observe_loss_fn,
            supply_reached_fn,
            move_vehicle_fn,
            line_of_sight_fn,
            sense_fn,
            aim_fn,
            |host| unsafe { sys::eech_engine_loss_release(entity_id, &mut released, host) },
        )?;
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
    pub fn los_probe<F, G, H, M, L, D, A>(
        &self,
        source_id: i32,
        target_id: i32,
        seed: u32,
        count: i32,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
        line_of_sight_fn: &mut L,
        sense_fn: &mut D,
        aim_fn: &mut A,
    ) -> Result<LosTrace, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
        L: FnMut(LosMessage) -> Result<LosReply, String>,
        D: FnMut(SenseMessage) -> Result<SenseReply, String>,
        A: FnMut(AimRequest) -> Result<AimReply, String>,
    {
        let mut raw = LosTrace::blank();
        // SAFETY: the out pointer and the temporary host callback are valid for the call.
        with_host(
            bind_mobile_fn,
            observe_loss_fn,
            supply_reached_fn,
            move_vehicle_fn,
            line_of_sight_fn,
            sense_fn,
            aim_fn,
            |host| unsafe { sys::eech_engine_los_probe(source_id, target_id, seed, count, &mut raw, host) },
        )?;
        Ok(LosTrace::from(raw))
    }

    /// TEST-ONLY (feature `sensor-fixture`, EEDCS-43): the run-wide report of every AI target scan the campaign has made since boot or
    /// the last reset, as `key=value` text lines. Read-only.
    #[cfg(feature = "sensor-fixture")]
    pub fn sensor_trace(&self) -> Result<String, EngineError> {
        let mut text = vec![0 as std::os::raw::c_char; SENSOR_REPORT_CAPACITY];
        // SAFETY: the buffer outlives the call and the capacity passed is its length; the engine NUL-terminates it.
        check(unsafe { sys::eech_engine_sensor_trace(text.as_mut_ptr(), text.len() as i32) })?;
        // SAFETY: the engine wrote a NUL-terminated string within the buffer.
        Ok(unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// TEST-ONLY (feature `aim-fixture`, EEDCS-44): the run-wide report of the physical weapon-aiming code the campaign's own update loop has run
    /// since boot or the last reset, as `key=value` text lines. Read-only; there is no probe.
    #[cfg(feature = "aim-fixture")]
    pub fn aim_trace(&self) -> Result<String, EngineError> {
        let mut text = vec![0 as std::os::raw::c_char; AIM_REPORT_CAPACITY];
        // SAFETY: the buffer outlives the call and the capacity passed is its length; the engine NUL-terminates it.
        check(unsafe { sys::eech_engine_aim_trace(text.as_mut_ptr(), text.len() as i32) })?;
        // SAFETY: the engine wrote a NUL-terminated string within the buffer.
        Ok(unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// TEST-ONLY (feature `aim-fixture`, EEDCS-44): the pose of one host-identified entity (type, alive, position, attitude, target link), as `key=value` text.
    #[cfg(feature = "aim-fixture")]
    pub fn aim_person_query(&self, host_id: &str) -> Result<String, EngineError> {
        let host_id = std::ffi::CString::new(host_id).map_err(|_| EngineError::BadArgument("a host id contains a NUL byte".into()))?;
        let mut text = vec![0 as std::os::raw::c_char; AIM_REPORT_CAPACITY];
        // SAFETY: the string and the buffer outlive the call and the capacity passed is the buffer length; the engine NUL-terminates it.
        check(unsafe { sys::eech_engine_aim_person_query(host_id.as_ptr(), text.as_mut_ptr(), text.len() as i32) })?;
        // SAFETY: the engine wrote a NUL-terminated string within the buffer.
        Ok(unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// TEST-ONLY (feature `aim-fixture`, EEDCS-44): gives one host-identified person one host-identified target through the campaign's own target link,
    /// and reports the person before and after. The call is given the same complete host as a frame.
    #[cfg(feature = "aim-fixture")]
    #[allow(clippy::too_many_arguments)]
    pub fn aim_person_case<F, G, H, M, L, D, A>(
        &self,
        person_id: &str,
        target_id: &str,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
        line_of_sight_fn: &mut L,
        sense_fn: &mut D,
        aim_fn: &mut A,
    ) -> Result<String, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
        L: FnMut(LosMessage) -> Result<LosReply, String>,
        D: FnMut(SenseMessage) -> Result<SenseReply, String>,
        A: FnMut(AimRequest) -> Result<AimReply, String>,
    {
        let person_id = std::ffi::CString::new(person_id).map_err(|_| EngineError::BadArgument("a host id contains a NUL byte".into()))?;
        let target_id = std::ffi::CString::new(target_id).map_err(|_| EngineError::BadArgument("a host id contains a NUL byte".into()))?;
        let mut text = vec![0 as std::os::raw::c_char; AIM_REPORT_CAPACITY];
        // SAFETY: the strings, the buffer and the temporary host callback outlive the call and the capacity passed is the buffer length; the engine NUL-terminates it.
        with_host(
            bind_mobile_fn,
            observe_loss_fn,
            supply_reached_fn,
            move_vehicle_fn,
            line_of_sight_fn,
            sense_fn,
            aim_fn,
            |host| unsafe { sys::eech_engine_aim_person_case(person_id.as_ptr(), target_id.as_ptr(), text.as_mut_ptr(), text.len() as i32, host) },
        )?;
        // SAFETY: the engine wrote a NUL-terminated string within the buffer.
        Ok(unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// TEST-ONLY (feature `aim-fixture`, EEDCS-44): the campaign's own lead-and-ballistic function asked once for two host-identified live entities with the caller's
    /// time-of-flight variable set to `sentinel`; the report says whether the variable still holds it. Given the same complete host as a frame.
    #[cfg(feature = "aim-fixture")]
    #[allow(clippy::too_many_arguments)]
    pub fn aim_lead_probe<F, G, H, M, L, D, A>(
        &self,
        source_id: &str,
        target_id: &str,
        weapon: i32,
        sentinel: f32,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
        line_of_sight_fn: &mut L,
        sense_fn: &mut D,
        aim_fn: &mut A,
    ) -> Result<String, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
        L: FnMut(LosMessage) -> Result<LosReply, String>,
        D: FnMut(SenseMessage) -> Result<SenseReply, String>,
        A: FnMut(AimRequest) -> Result<AimReply, String>,
    {
        let source_id = std::ffi::CString::new(source_id).map_err(|_| EngineError::BadArgument("a host id contains a NUL byte".into()))?;
        let target_id = std::ffi::CString::new(target_id).map_err(|_| EngineError::BadArgument("a host id contains a NUL byte".into()))?;
        let mut text = vec![0 as std::os::raw::c_char; AIM_REPORT_CAPACITY];
        // SAFETY: the strings, the buffer and the temporary host callback outlive the call and the capacity passed is the buffer length; the engine NUL-terminates it.
        with_host(
            bind_mobile_fn,
            observe_loss_fn,
            supply_reached_fn,
            move_vehicle_fn,
            line_of_sight_fn,
            sense_fn,
            aim_fn,
            |host| unsafe {
                sys::eech_engine_aim_lead_probe(
                    source_id.as_ptr(),
                    target_id.as_ptr(),
                    weapon,
                    sentinel,
                    text.as_mut_ptr(),
                    text.len() as i32,
                    host,
                )
            },
        )?;
        // SAFETY: the engine wrote a NUL-terminated string within the buffer.
        Ok(unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// TEST-ONLY (feature `aim-fixture`, EEDCS-44): clears the run-wide aiming trace.
    #[cfg(feature = "aim-fixture")]
    pub fn aim_trace_reset(&self) -> Result<(), EngineError> {
        // SAFETY: no pointers cross.
        check(unsafe { sys::eech_engine_aim_trace_reset() })
    }

    /// TEST-ONLY (feature `weapon-fixture`, EEDCS-46): the run-wide report of the physical combat-resolution code (damage capability, collisions, blasts, repair writes) since
    /// process start, as `key=value` text lines. Read-only.
    #[cfg(feature = "weapon-fixture")]
    pub fn cmb_trace(&self) -> Result<String, EngineError> {
        let mut text = vec![0 as std::os::raw::c_char; WPN_REPORT_CAPACITY];
        // SAFETY: the buffer outlives the call and the capacity passed is its length; the engine NUL-terminates it.
        check(unsafe { sys::eech_engine_cmb_trace(text.as_mut_ptr(), text.len() as i32) })?;
        // SAFETY: the engine wrote a NUL-terminated string within the buffer.
        Ok(unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// TEST-ONLY (feature `weapon-fixture`, EEDCS-46): a controlled combat case that launches nothing (op 0 the census of living entities per health-bearing kind; op 1 the
    /// `ordinal`-th living entity of `kind` set to `percent` of its initial health), as `key=value` text. Given the same complete host as a frame: the health case sets a health
    /// through the campaign's own setter and asks the critical predicate, and the host answers both (the physical health facts are the host's since EEDCS-46).
    #[cfg(feature = "weapon-fixture")]
    #[allow(clippy::too_many_arguments)]
    pub fn cmb_case_state<F, G, H, M, L, D, A>(
        &self,
        op: i32,
        kind: &str,
        ordinal: i32,
        percent: i32,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
        line_of_sight_fn: &mut L,
        sense_fn: &mut D,
        aim_fn: &mut A,
    ) -> Result<String, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
        L: FnMut(LosMessage) -> Result<LosReply, String>,
        D: FnMut(SenseMessage) -> Result<SenseReply, String>,
        A: FnMut(AimRequest) -> Result<AimReply, String>,
    {
        let kind = std::ffi::CString::new(kind).map_err(|_| EngineError::BadArgument("a kind contains a NUL byte".into()))?;
        let mut text = vec![0 as std::os::raw::c_char; WPN_REPORT_CAPACITY];
        // SAFETY: the string, the buffer and the temporary host callback outlive the call and the capacity passed is the buffer length; the engine NUL-terminates it.
        let status = with_host(
            bind_mobile_fn,
            observe_loss_fn,
            supply_reached_fn,
            move_vehicle_fn,
            line_of_sight_fn,
            sense_fn,
            aim_fn,
            |host| unsafe { sys::eech_engine_cmb_case_state(op, kind.as_ptr(), ordinal, percent, text.as_mut_ptr(), text.len() as i32, host) },
        );
        keep_case_reason(status, &text)
    }

    /// TEST-ONLY (feature `weapon-fixture`, EEDCS-46): a controlled combat case that launches a real weapon through the campaign's own launch call (op 1: the `ordinal`-th living
    /// entity of `kind` is delivered a collision with modifier `amount`, by that weapon or by none; op 2: the weapon is put through the campaign's own killed-weapon explosion
    /// `amount` metres from that entity with `kill_code`). Given the same complete host as a frame.
    #[cfg(feature = "weapon-fixture")]
    #[allow(clippy::too_many_arguments)]
    pub fn cmb_case_act<F, G, H, M, L, D, A>(
        &self,
        op: i32,
        kind: &str,
        ordinal: i32,
        launcher_id: &str,
        weapon: i32,
        amount: f64,
        kill_code: i32,
        weaponless: bool,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
        line_of_sight_fn: &mut L,
        sense_fn: &mut D,
        aim_fn: &mut A,
    ) -> Result<String, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
        L: FnMut(LosMessage) -> Result<LosReply, String>,
        D: FnMut(SenseMessage) -> Result<SenseReply, String>,
        A: FnMut(AimRequest) -> Result<AimReply, String>,
    {
        let kind = std::ffi::CString::new(kind).map_err(|_| EngineError::BadArgument("a kind contains a NUL byte".into()))?;
        let launcher_id = std::ffi::CString::new(launcher_id).map_err(|_| EngineError::BadArgument("a host id contains a NUL byte".into()))?;
        let mut text = vec![0 as std::os::raw::c_char; WPN_REPORT_CAPACITY];
        // SAFETY: the strings, the buffer and the temporary host callback outlive the call and the capacity passed is the buffer length; the engine NUL-terminates it.
        let status = with_host(
            bind_mobile_fn,
            observe_loss_fn,
            supply_reached_fn,
            move_vehicle_fn,
            line_of_sight_fn,
            sense_fn,
            aim_fn,
            |host| unsafe {
                sys::eech_engine_cmb_case_act(
                    op,
                    kind.as_ptr(),
                    ordinal,
                    launcher_id.as_ptr(),
                    weapon,
                    amount,
                    kill_code,
                    i32::from(weaponless),
                    text.as_mut_ptr(),
                    text.len() as i32,
                    host,
                )
            },
        );
        keep_case_reason(status, &text)
    }

    /// TEST-ONLY (feature `weapon-fixture`, EEDCS-46, R5): the controlled bridge case of the ORIGINAL collapse (op 0 survey of every bridge, op 1 state of the bridge at `ordinal`, op 2 the declared direct
    /// structural failure of segment `index` of it, or of the first by `criterion` when `index` is -1, op 3 disarm the observer), as `key=value` text. Given the same complete host as a frame, because the
    /// failure runs the campaign's own kill, explosion effects and consequences and the campaign asks the host for the physical facts they need.
    #[cfg(feature = "weapon-fixture")]
    #[allow(clippy::too_many_arguments)]
    pub fn cmb_case_bridge<F, G, H, M, L, D, A>(
        &self,
        op: i32,
        ordinal: i32,
        index: i32,
        criterion: i32,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
        line_of_sight_fn: &mut L,
        sense_fn: &mut D,
        aim_fn: &mut A,
    ) -> Result<String, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
        L: FnMut(LosMessage) -> Result<LosReply, String>,
        D: FnMut(SenseMessage) -> Result<SenseReply, String>,
        A: FnMut(AimRequest) -> Result<AimReply, String>,
    {
        let mut text = vec![0 as std::os::raw::c_char; WPN_REPORT_CAPACITY];
        // SAFETY: the buffer and the temporary host callback outlive the call and the capacity passed is the buffer length; the engine NUL-terminates it.
        let status = with_host(
            bind_mobile_fn,
            observe_loss_fn,
            supply_reached_fn,
            move_vehicle_fn,
            line_of_sight_fn,
            sense_fn,
            aim_fn,
            |host| unsafe { sys::eech_engine_cmb_case_bridge(op, ordinal, index, criterion, text.as_mut_ptr(), text.len() as i32, host) },
        );
        keep_case_reason(status, &text)
    }
    /// TEST-ONLY (feature `weapon-fixture`, EEDCS-46): drains the ordered combat event log (collisions, blasts, repairs) from the cursor and returns every event not yet returned,
    /// one `cmb-ev` line each. Read-only apart from advancing the cursor.
    #[cfg(feature = "weapon-fixture")]
    pub fn cmb_events(&self) -> Result<String, EngineError> {
        let mut all = String::new();
        loop {
            let mut text = vec![0 as std::os::raw::c_char; WPN_REPORT_CAPACITY];
            let mut more: std::os::raw::c_int = 0;
            // SAFETY: the buffer and `more` outlive the call and the capacity passed is the buffer length; the engine NUL-terminates the text.
            check(unsafe { sys::eech_engine_cmb_events(text.as_mut_ptr(), text.len() as i32, &mut more) })?;
            // SAFETY: the engine wrote a NUL-terminated string within the buffer.
            all.push_str(&unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy());
            if more == 0 {
                return Ok(all);
            }
        }
    }

    /// TEST-ONLY (feature `weapon-fixture`, EEDCS-45): the run-wide report of the physical weapon-execution code (table generation at boot, launch, flight, update order,
    /// contact and terminal facts) since process start, as `key=value` text lines. Read-only.
    #[cfg(feature = "weapon-fixture")]
    pub fn wpn_trace(&self) -> Result<String, EngineError> {
        let mut text = vec![0 as std::os::raw::c_char; WPN_REPORT_CAPACITY];
        // SAFETY: the buffer outlives the call and the capacity passed is its length; the engine NUL-terminates it.
        check(unsafe { sys::eech_engine_wpn_trace(text.as_mut_ptr(), text.len() as i32) })?;
        // SAFETY: the engine wrote a NUL-terminated string within the buffer.
        Ok(unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// TEST-ONLY (feature `weapon-fixture`, EEDCS-45): drains the ordered event log (births, launch outcomes, terminals, destroys, table summaries) from the cursor and returns
    /// every event not yet returned, one `wpn-ev` line each. Read-only apart from advancing the cursor.
    #[cfg(feature = "weapon-fixture")]
    pub fn wpn_events(&self) -> Result<String, EngineError> {
        let mut all = String::new();
        loop {
            let mut text = vec![0 as std::os::raw::c_char; WPN_REPORT_CAPACITY];
            let mut more: std::os::raw::c_int = 0;
            // SAFETY: the buffer and `more` outlive the call and the capacity passed is the buffer length; the engine NUL-terminates the text.
            check(unsafe { sys::eech_engine_wpn_events(text.as_mut_ptr(), text.len() as i32, &mut more) })?;
            // SAFETY: the engine wrote a NUL-terminated string within the buffer.
            all.push_str(&unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy());
            if more == 0 {
                return Ok(all);
            }
        }
    }

    /// TEST-ONLY (feature `weapon-fixture`, EEDCS-45): the state of one host-identified entity and its count of one weapon type, as `key=value` text.
    #[cfg(feature = "weapon-fixture")]
    pub fn wpn_case_query(&self, host_id: &str, weapon: i32) -> Result<String, EngineError> {
        let host_id = std::ffi::CString::new(host_id).map_err(|_| EngineError::BadArgument("a host id contains a NUL byte".into()))?;
        let mut text = vec![0 as std::os::raw::c_char; 4096];
        // SAFETY: the string and the buffer outlive the call and the capacity passed is the buffer length; the engine NUL-terminates it.
        check(unsafe { sys::eech_engine_wpn_case_query(host_id.as_ptr(), weapon, text.as_mut_ptr(), text.len() as i32) })?;
        // SAFETY: the engine wrote a NUL-terminated string within the buffer.
        Ok(unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// TEST-ONLY (feature `weapon-fixture`, EEDCS-45): the entity kind of a roster host id from its type field alone, as `key=value` text (`found=0` for an unknown id).
    #[cfg(feature = "weapon-fixture")]
    pub fn wpn_case_kind(&self, host_id: &str) -> Result<String, EngineError> {
        let host_id = std::ffi::CString::new(host_id).map_err(|_| EngineError::BadArgument("a host id contains a NUL byte".into()))?;
        let mut text = vec![0 as std::os::raw::c_char; 4096];
        // SAFETY: the string and the buffer outlive the call and the capacity passed is the buffer length; the engine NUL-terminates it.
        check(unsafe { sys::eech_engine_wpn_case_kind(host_id.as_ptr(), text.as_mut_ptr(), text.len() as i32) })?;
        // SAFETY: the engine wrote a NUL-terminated string within the buffer.
        Ok(unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// TEST-ONLY (feature `weapon-fixture`, EEDCS-45): one physical launch attempt of one weapon type by one host-identified launcher through the campaign's own launch call,
    /// after an optional host-identified target link set through the campaign's own link call (an empty target id sets none). Given the same complete host as a frame.
    #[cfg(feature = "weapon-fixture")]
    #[allow(clippy::too_many_arguments)]
    pub fn wpn_case_launch<F, G, H, M, L, D, A>(
        &self,
        launcher_id: &str,
        target_id: &str,
        weapon: i32,
        salvo_count: i32,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
        line_of_sight_fn: &mut L,
        sense_fn: &mut D,
        aim_fn: &mut A,
    ) -> Result<String, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
        L: FnMut(LosMessage) -> Result<LosReply, String>,
        D: FnMut(SenseMessage) -> Result<SenseReply, String>,
        A: FnMut(AimRequest) -> Result<AimReply, String>,
    {
        let launcher_id = std::ffi::CString::new(launcher_id).map_err(|_| EngineError::BadArgument("a host id contains a NUL byte".into()))?;
        let target_id = std::ffi::CString::new(target_id).map_err(|_| EngineError::BadArgument("a host id contains a NUL byte".into()))?;
        let mut text = vec![0 as std::os::raw::c_char; WPN_REPORT_CAPACITY];
        // SAFETY: the strings, the buffer and the temporary host callback outlive the call and the capacity passed is the buffer length; the engine NUL-terminates it.
        with_host(
            bind_mobile_fn,
            observe_loss_fn,
            supply_reached_fn,
            move_vehicle_fn,
            line_of_sight_fn,
            sense_fn,
            aim_fn,
            |host| unsafe {
                sys::eech_engine_wpn_case_launch(
                    launcher_id.as_ptr(),
                    target_id.as_ptr(),
                    weapon,
                    salvo_count,
                    text.as_mut_ptr(),
                    text.len() as i32,
                    host,
                )
            },
        )?;
        // SAFETY: the engine wrote a NUL-terminated string within the buffer.
        Ok(unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// TEST-ONLY (feature `sensor-fixture`, EEDCS-43): clears the run-wide sensor trace.
    #[cfg(feature = "sensor-fixture")]
    pub fn sensor_trace_reset(&self) -> Result<(), EngineError> {
        // SAFETY: no pointers cross.
        check(unsafe { sys::eech_engine_sensor_trace_reset() })
    }

    /// TEST-ONLY (feature `terrain-fixture`, EEDCS-47): the run-wide report of every terrain query the campaign has made since boot or the
    /// last reset, as `key=value` text lines. Read-only.
    #[cfg(feature = "terrain-fixture")]
    pub fn terrain_trace(&self) -> Result<String, EngineError> {
        let mut text = vec![0 as std::os::raw::c_char; TERRAIN_REPORT_CAPACITY];
        // SAFETY: the buffer outlives the call and the capacity passed is its length; the engine NUL-terminates it.
        check(unsafe { sys::eech_engine_terrain_trace(text.as_mut_ptr(), text.len() as i32) })?;
        // SAFETY: the engine wrote a NUL-terminated string within the buffer.
        Ok(unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// TEST-ONLY (feature `terrain-fixture`, EEDCS-47): clears the run-wide terrain trace.
    #[cfg(feature = "terrain-fixture")]
    pub fn terrain_trace_reset(&self) -> Result<(), EngineError> {
        // SAFETY: no pointers cross.
        check(unsafe { sys::eech_engine_terrain_trace_reset() })
    }

    /// TEST-ONLY (feature `terrain-fixture`, EEDCS-47): runs the deterministic terrain probe sweep over the loaded map (six fixed points,
    /// then `count` from a private generator seeded with `seed`), writing the first `emit` samples (at most 256) in full. Read-only: the
    /// run-wide trace and the shared find-fast record are left as found. In the unmoved module it asks nothing of the host; in the host-terrain module every
    /// point and fast-elevation query of the sweep is a question to the installed terrain provider.
    #[cfg(feature = "terrain-fixture")]
    pub fn terrain_probe(&self, seed: u32, count: i32, emit: i32) -> Result<String, EngineError> {
        let mut text = vec![0 as std::os::raw::c_char; TERRAIN_REPORT_CAPACITY];
        // SAFETY: the buffer outlives the call and the capacity passed is its length; the engine NUL-terminates it.
        check(unsafe { sys::eech_engine_terrain_probe(seed, count, emit, text.as_mut_ptr(), text.len() as i32) })?;
        // SAFETY: the engine wrote a NUL-terminated string within the buffer.
        Ok(unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// TEST-ONLY (feature `terrain-fixture`, EEDCS-47): the campaign's record predicate (`get_3d_terrain_point_data_has_face`, which the fixed-wing
    /// movement asks) through the real client queries at (x, z), on a private and on the shared record: fresh, after a query, after a second one and
    /// after a clear (`terrain-face-probe` line). Only the host-terrain module has the predicate; the unmoved one refuses. Every query it makes is a
    /// question to the installed terrain provider.
    #[cfg(feature = "terrain-fixture")]
    pub fn terrain_face_probe(&self, x: f32, z: f32) -> Result<String, EngineError> {
        let mut text = vec![0 as std::os::raw::c_char; TERRAIN_REPORT_CAPACITY];
        // SAFETY: the buffer outlives the call and the capacity passed is its length; the engine NUL-terminates it.
        check(unsafe { sys::eech_engine_terrain_face_probe(x, z, text.as_mut_ptr(), text.len() as i32) })?;
        // SAFETY: the engine wrote a NUL-terminated string within the buffer.
        Ok(unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy().into_owned())
    }

    /// TEST-ONLY (feature `sensor-fixture`, EEDCS-43): runs the AI target-scan group functions for every eligible live source,
    /// read-only, and reports a trace of just that sweep; the run-wide trace is untouched. The surface-to-air scan asks the host for line
    /// of sight, so the sweep is given the same complete host as a frame.
    #[cfg(feature = "sensor-fixture")]
    pub fn sensor_probe<F, G, H, M, L, D, A>(
        &self,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
        line_of_sight_fn: &mut L,
        sense_fn: &mut D,
        aim_fn: &mut A,
    ) -> Result<String, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
        L: FnMut(LosMessage) -> Result<LosReply, String>,
        D: FnMut(SenseMessage) -> Result<SenseReply, String>,
        A: FnMut(AimRequest) -> Result<AimReply, String>,
    {
        let mut text = vec![0 as std::os::raw::c_char; SENSOR_REPORT_CAPACITY];
        // SAFETY: the buffer and the temporary host callback outlive the call and the capacity passed is the buffer length; the engine NUL-terminates it.
        with_host(
            bind_mobile_fn,
            observe_loss_fn,
            supply_reached_fn,
            move_vehicle_fn,
            line_of_sight_fn,
            sense_fn,
            aim_fn,
            |host| unsafe { sys::eech_engine_sensor_probe(text.as_mut_ptr(), text.len() as i32, host) },
        )?;
        // SAFETY: the engine wrote a NUL-terminated string within the buffer.
        Ok(unsafe { std::ffi::CStr::from_ptr(text.as_ptr()) }.to_string_lossy().into_owned())
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

    /// Applies one physical fact the host reports about an object it holds the identity of (EEDCS-46, R3): a structure's, issued by `AimRequest::Identify`, or a rostered mobile's.  The campaign applies it
    /// once through the object's own collision steps, so the consequences come from the real handlers, which ask the host for the physical facts they need (the complete host is given as for a frame).
    /// An identity that is unknown, ended or not of such an object, an object already destroyed and a report that would raise a damage level fail visibly and change nothing.
    #[allow(clippy::too_many_arguments)]
    pub fn report_object<F, G, H, M, L, D, A>(
        &self,
        report: &ObjectReport,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
        line_of_sight_fn: &mut L,
        sense_fn: &mut D,
        aim_fn: &mut A,
    ) -> Result<ObjectReceipt, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
        L: FnMut(LosMessage) -> Result<LosReply, String>,
        D: FnMut(SenseMessage) -> Result<SenseReply, String>,
        A: FnMut(AimRequest) -> Result<AimReply, String>,
    {
        let bytes = report.host_id.as_bytes();
        if bytes.is_empty() || bytes.len() > sys::EECH_AIM_HOST_ID_MAX || bytes.contains(&0) {
            return Err(EngineError::BadArgument(
                "an object report needs an identity of 1 to 128 bytes without a NUL".into(),
            ));
        }
        let mut raw = sys::eech_object_report {
            host_id: [0 as c_char; sys::EECH_AIM_HOST_ID_MAX + 1],
            has_damage_level: c_int::from(report.damage_level.is_some()),
            damage_level: report.damage_level.unwrap_or(0),
            destroyed: c_int::from(report.destroyed),
        };
        for (slot, byte) in raw.host_id.iter_mut().zip(bytes) {
            *slot = *byte as c_char;
        }
        let mut receipt = sys::eech_object_report_receipt::default();
        // SAFETY: the report and the receipt outlive the call; the temporary host callback is valid for the call; the receipt is read only on success.
        with_host(
            bind_mobile_fn,
            observe_loss_fn,
            supply_reached_fn,
            move_vehicle_fn,
            line_of_sight_fn,
            sense_fn,
            aim_fn,
            |host| unsafe { sys::eech_engine_report_object(&raw, &mut receipt, host) },
        )?;
        Ok(ObjectReceipt {
            entity_type: receipt.entity_type,
            rostered: receipt.rostered != 0,
            damage_level_before: receipt.damage_level_before,
            damage_level_after: receipt.damage_level_after,
            killed: receipt.killed != 0,
        })
    }

    /// Every structure with its identity, identified now (the host is asked, once per incarnation) if it has none; copies of values (EEDCS-46, R3).
    #[allow(clippy::too_many_arguments)]
    pub fn structures<F, G, H, M, L, D, A>(
        &self,
        bind_mobile_fn: &mut F,
        observe_loss_fn: &mut G,
        supply_reached_fn: &mut H,
        move_vehicle_fn: &mut M,
        line_of_sight_fn: &mut L,
        sense_fn: &mut D,
        aim_fn: &mut A,
    ) -> Result<Vec<Structure>, EngineError>
    where
        F: FnMut(MemberDescriptor) -> Result<String, String>,
        G: FnMut(LossDescriptor) -> Result<LossDecision, String>,
        H: FnMut(SupplyReachDescriptor) -> Result<ArrivalAnswer, String>,
        M: FnMut(MovementStep) -> Result<MovementResult, String>,
        L: FnMut(LosMessage) -> Result<LosReply, String>,
        D: FnMut(SenseMessage) -> Result<SenseReply, String>,
        A: FnMut(AimRequest) -> Result<AimReply, String>,
    {
        unsafe extern "C" fn collect(structure: *const sys::eech_structure, user: *mut c_void) {
            // SAFETY: the engine passes a valid structure; user is the Vec below.
            let (structure, out) = unsafe { (&*structure, &mut *(user as *mut Vec<Structure>)) };
            let host_id = if structure.host_id.is_null() {
                String::new()
            } else {
                // SAFETY: the identity is NUL-terminated and borrowed for this callback only (the registry may move it when it grows); it is copied into an owned String here, before the call returns.
                unsafe { CStr::from_ptr(structure.host_id) }.to_string_lossy().into_owned()
            };
            out.push(Structure {
                host_id,
                kind: match structure.kind {
                    sys::EECH_STRUCTURE_CITY_BUILDING => StructureKind::CityBuilding,
                    sys::EECH_STRUCTURE_PYLON => StructureKind::Pylon,
                    sys::EECH_STRUCTURE_SCENIC => StructureKind::Scenic,
                    sys::EECH_STRUCTURE_SEGMENT => StructureKind::Segment,
                    sys::EECH_STRUCTURE_SITE => StructureKind::Site,
                    _ => StructureKind::SiteUpdatable,
                },
                entity_type: structure.entity_type,
                sub_type: structure.sub_type,
                side: side(structure.side),
                alive: structure.alive != 0,
                damage_level: structure.damage_level,
                initial_damage_level: structure.initial_damage_level,
                position: structure.position,
            });
        }
        let mut out: Vec<Structure> = Vec::new();
        let mut count: c_int = 0;
        let out_ptr = (&mut out as *mut Vec<Structure>).cast::<c_void>();
        // SAFETY: the callback only touches `out` during the call; the temporary host callback is valid for the call.
        with_host(
            bind_mobile_fn,
            observe_loss_fn,
            supply_reached_fn,
            move_vehicle_fn,
            line_of_sight_fn,
            sense_fn,
            aim_fn,
            |host| unsafe { sys::eech_engine_structures(collect, out_ptr, &mut count, host) },
        )?;
        Ok(out)
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
        let mut line_of_sight = |_: LosMessage| -> Result<LosReply, String> { Err("unused".into()) };
        let mut sense = |_: SenseMessage| -> Result<SenseReply, String> { Err("unused".into()) };
        let mut aim = |_: AimRequest| -> Result<AimReply, String> { Err("unused".into()) };
        let outcome = with_host(
            &mut bind_mobile,
            &mut observe_loss,
            &mut supply_reached,
            &mut host_move,
            &mut line_of_sight,
            &mut sense,
            &mut aim,
            |host| {
                // SAFETY: the table and its user context are valid for the call; every other pointer is local.
                unsafe {
                    let host = &*host;
                    let Some(call) = host.move_vehicle else { return -1 };
                    call(step, &mut result, error.as_mut_ptr(), error.len(), host.user)
                }
            },
        );
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
            sys::EECH_MOVEMENT_PERSON_FACE => MovementMode::PersonFace,
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
        let mut line_of_sight = |_: LosMessage| -> Result<LosReply, String> { Err("unused".into()) };
        let mut sense = |_: SenseMessage| -> Result<SenseReply, String> { Err("unused".into()) };
        let mut aim = |_: AimRequest| -> Result<AimReply, String> { Err("unused".into()) };
        let outcome = with_host(
            &mut bind_mobile,
            &mut observe_loss,
            &mut supply_reached,
            &mut host_move,
            &mut line_of_sight,
            &mut sense,
            &mut aim,
            |host| {
                // SAFETY: the table and its user context are valid for the call; every other pointer is local.
                unsafe {
                    let host = &*host;
                    let Some(call) = host.move_vehicle else { return -1 };
                    call(step, &mut result, error.as_mut_ptr(), error.len(), host.user)
                }
            },
        );
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
            let mut line_of_sight = |_: LosMessage| -> Result<LosReply, String> { Err("unused".into()) };
            let mut sense = |_: SenseMessage| -> Result<SenseReply, String> { Err("unused".into()) };
            let mut aim = |_: AimRequest| -> Result<AimReply, String> { Err("unused".into()) };
            let outcome = with_host(
                &mut bind_mobile,
                &mut observe_loss,
                &mut supply_reached,
                &mut host_move,
                &mut line_of_sight,
                &mut sense,
                &mut aim,
                |host| unsafe {
                    let host = &*host;
                    let Some(call) = host.move_vehicle else { return -1 };
                    call(&step, &mut result, error.as_mut_ptr(), error.len(), host.user)
                },
            );
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

    // ---- EEDCS-44: the seventh callback, `aim` ----

    fn blank_aim_step(mode: c_int) -> sys::eech_aim_step {
        // SAFETY: plain data; all-zero is a valid (empty) request of every field.
        let mut step: sys::eech_aim_step = unsafe { std::mem::zeroed() };
        step.mode = mode;
        step
    }

    /// What the production wiring does with `step` when the host answers with `answer`: the request the host saw (none when the callback refused it
    /// first), the result C is given, and C's code.
    fn aim_crossing(
        step: &sys::eech_aim_step,
        answer: impl FnOnce(AimRequest) -> Result<AimReply, String>,
    ) -> (Option<AimRequest>, sys::eech_aim_result, c_int, String) {
        let mut answer = Some(answer);
        let mut seen = None;
        let mut bind_mobile = |_: MemberDescriptor| -> Result<String, String> { Err("unused".into()) };
        let mut observe_loss = |_: LossDescriptor| -> Result<LossDecision, String> { Ok(LossDecision::Forward) };
        let mut supply_reached = |_: SupplyReachDescriptor| -> Result<ArrivalAnswer, String> { Err("unused".into()) };
        let mut host_move = |_: MovementStep| -> Result<MovementResult, String> { Err("unused".into()) };
        let mut line_of_sight = |_: LosMessage| -> Result<LosReply, String> { Err("unused".into()) };
        let mut sense = |_: SenseMessage| -> Result<SenseReply, String> { Err("unused".into()) };
        let mut aim = |request: AimRequest| -> Result<AimReply, String> {
            seen = Some(request.clone());
            match answer.take() {
                Some(answer) => answer(request),
                None => Err("the host was asked twice".into()),
            }
        };
        let mut result: sys::eech_aim_result = unsafe { std::mem::zeroed() };
        let mut error = [0 as c_char; 256];
        let mut code = -1;
        let outcome = with_host(
            &mut bind_mobile,
            &mut observe_loss,
            &mut supply_reached,
            &mut host_move,
            &mut line_of_sight,
            &mut sense,
            &mut aim,
            |host| {
                // SAFETY: the table and its user context are valid for the call; every other pointer is local.
                unsafe {
                    let host = &*host;
                    let Some(call) = host.aim else { return -1 };
                    code = call(step, &mut result, error.as_mut_ptr(), error.len(), host.user);
                }
                0
            },
        );
        assert!(outcome.is_ok());
        let text: Vec<u8> = error.iter().map(|byte| *byte as u8).take_while(|byte| *byte != 0).collect();
        (seen, result, code, String::from_utf8_lossy(&text).into_owned())
    }

    fn figures_step(mode: c_int) -> sys::eech_aim_step {
        let mut step = blank_aim_step(mode);
        step.weapon = 12;
        for (slot, byte) in step.figures.name.iter_mut().zip(b"M2HB_test_gun\0") {
            *slot = *byte as c_char;
        }
        step.figures.max_range = 100.0;
        step.figures.muzzle_velocity = 890.0;
        step.figures.drag_coefficient = 0.3;
        step.figures.diameter = 0.0127;
        step.figures.weapon_class = 8;
        step.pass_first = 1;
        step.pass_last = 0;
        step
    }

    #[test]
    fn an_aim_table_request_hands_the_host_the_weapons_figures_and_no_cell_and_figures_that_cannot_be_a_weapon_never_reach_it() {
        let step = figures_step(sys::EECH_AIM_TABLE);
        let (seen, _, code, _) = aim_crossing(&step, |_| Ok(AimReply::default()));
        assert_eq!(code, 0);
        let Some(AimRequest::Table(table)) = seen else {
            panic!("the host was not asked to generate a table")
        };
        assert_eq!(
            (
                table.weapon,
                table.figures.name.as_str(),
                table.figures.max_range,
                table.figures.muzzle_velocity
            ),
            (12, "M2HB_test_gun", 100.0, 890.0)
        );
        assert_eq!(
            (table.first, table.last, table.echo, table.echo_cells, table.figures.weapon_class),
            (true, false, false, 0, 8)
        );
        // figures that are not a weapon's never reach the host: no name, a non-finite figure, no range, a flag that is not a boolean
        let mut unnamed = figures_step(sys::EECH_AIM_TABLE);
        unnamed.figures.name[0] = 0;
        let mut infinite = figures_step(sys::EECH_AIM_TABLE);
        infinite.figures.drag_coefficient = f32::INFINITY;
        let mut rangeless = figures_step(sys::EECH_AIM_TABLE);
        rangeless.figures.max_range = 0.0;
        let mut flagged = figures_step(sys::EECH_AIM_TABLE);
        flagged.pass_first = 2;
        for (what, bad) in [("name", unnamed), ("figure", infinite), ("range", rangeless), ("flag", flagged)] {
            let (seen, _, code, error) = aim_crossing(&bad, |_| panic!("the host must not be asked"));
            assert!(seen.is_none() && code != 0, "{what}: {error}");
        }
    }

    #[test]
    fn an_aim_table_echo_is_written_into_the_room_the_campaign_gave_and_never_past_it() {
        let mut room = vec![0.0f32; 6];
        let mut step = figures_step(sys::EECH_AIM_TABLE);
        step.echo = 1;
        step.echo_cells = room.as_mut_ptr();
        step.echo_cell_capacity = room.len();
        let (seen, result, code, _) = aim_crossing(&step, |_| {
            Ok(AimReply {
                table_cells: Some(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]),
                ..AimReply::default()
            })
        });
        assert_eq!(code, 0);
        assert!(matches!(seen, Some(AimRequest::Table(ref table)) if table.echo && table.echo_cells == 6));
        assert_eq!(result.echo_cell_count, 6);
        assert_eq!(room, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let (_, _, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                table_cells: Some(vec![0.0; 7]),
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("more than the 6"), "{error}");
        // a host that echoes cells nobody asked for is refused
        let quiet = figures_step(sys::EECH_AIM_TABLE);
        let (_, _, code, error) = aim_crossing(&quiet, |_| {
            Ok(AimReply {
                table_cells: Some(vec![0.0; 6]),
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("nobody asked"), "{error}");
    }

    #[test]
    fn an_aim_range_request_hands_the_host_a_guided_weapons_figures() {
        let step = figures_step(sys::EECH_AIM_RANGE);
        let (seen, _, code, _) = aim_crossing(&step, |_| Ok(AimReply::default()));
        assert_eq!(code, 0);
        let Some(AimRequest::Range(range)) = seen else {
            panic!("the host was not asked for a range run")
        };
        assert_eq!(
            (range.weapon, range.figures.name.as_str(), range.first, range.last, range.echo),
            (12, "M2HB_test_gun", true, false, false)
        );
    }

    #[test]
    fn an_aim_figures_request_hands_the_host_one_weapons_figures_to_keep_and_a_weapon_that_is_not_one_never_reaches_it() {
        let mut step = figures_step(sys::EECH_AIM_FIGURES);
        let (seen, _, code, _) = aim_crossing(&step, |_| Ok(AimReply::default()));
        assert_eq!(code, 0);
        let Some(AimRequest::Figures(figures)) = seen else {
            panic!("the host was not handed the figures")
        };
        assert_eq!(
            (figures.weapon, figures.figures.name.as_str(), figures.figures.muzzle_velocity),
            (12, "M2HB_test_gun", 890.0)
        );
        step.weapon = 0;
        let (seen, _, code, error) = aim_crossing(&step, |_| panic!("the host must not be asked"));
        assert!(seen.is_none() && code != 0, "{error}");
        let mut infinite = figures_step(sys::EECH_AIM_FIGURES);
        infinite.figures.rate_of_fire = f32::INFINITY;
        let (seen, _, code, error) = aim_crossing(&infinite, |_| panic!("the host must not be asked"));
        assert!(seen.is_none() && code != 0, "{error}");
    }

    fn step_request(mode: c_int) -> sys::eech_aim_step {
        let mut step = blank_aim_step(mode);
        step.weapon = 5;
        step.delta_time = 0.1;
        step.system_time = 4242;
        step.map_max = [1.0e5; 3];
        step.weapon_state.attitude = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        step.weapon_state.velocity = 900.0;
        step.weapon_state.weapon_lifetime = 3.0;
        step.weapon_state.entity_samples = [1, 2, 3, 4];
        step.weapon_state.server = 1;
        step
    }

    fn event(kind: c_int, c: c_int, value: f32) -> AimEvent {
        AimEvent {
            kind,
            a: 0,
            b: 0,
            c,
            d: 0,
            f: [value; 12],
        }
    }

    #[test]
    fn a_weapon_step_carries_the_weapon_the_answers_so_far_and_the_samples_given_and_the_hosts_lookups_and_actions_come_back() {
        let mut step = step_request(sys::EECH_AIM_WEAPON_STEP);
        step.answer_count = 1;
        step.answers[0].kind = 8;
        step.answers[0].f[6] = 1.0;
        step.draw_count = 2;
        step.draws[0] = 100;
        step.draws[1] = 32767;
        let (seen, result, code, _) = aim_crossing(&step, |_| {
            Ok(AimReply {
                need_lookup: true,
                lookups: vec![event(7, 0, 0.5)],
                ..AimReply::default()
            })
        });
        assert_eq!(code, 0);
        let Some(AimRequest::WeaponStep(request)) = seen else {
            panic!("the host was not asked for a step")
        };
        assert_eq!(request.kind, AimStepKind::Step);
        assert_eq!(
            (
                request.weapon,
                request.delta_time,
                request.system_time,
                request.state.velocity,
                request.state.entity_samples
            ),
            (5, 0.1, 4242, 900.0, [1, 2, 3, 4])
        );
        assert_eq!(
            (request.draws.clone(), request.answers.len(), request.answers[0].kind, request.answers[0].f[6]),
            (vec![100, 32767], 1, 8, 1.0)
        );
        assert_eq!(
            (result.state, result.lookup_count, result.lookups[0].kind, result.action_count),
            (sys::EECH_AIM_NEED_LOOKUP, 1, 7, 0)
        );
        // done, with the actions in order
        let (_, result, code, _) = aim_crossing(&step, |_| {
            Ok(AimReply {
                actions: vec![event(1, 3, 2.0), event(5, 0, 7.0)],
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state, result.action_count), (0, sys::EECH_AIM_DONE, 2));
        assert_eq!(
            (result.actions[0].kind, result.actions[0].c, result.actions[1].kind, result.actions[1].f[0]),
            (1, 3, 5, 7.0)
        );
        // a draw is asked for by the host, not given
        let (_, result, code, _) = aim_crossing(&step, |_| {
            Ok(AimReply {
                need_draw: true,
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state), (0, sys::EECH_AIM_NEED_DRAW));
        // a birth is the same request with its flag
        let mut birth = step_request(sys::EECH_AIM_WEAPON_BIRTH);
        birth.weapon_state.submunition = 1;
        let (seen, _, code, _) = aim_crossing(&birth, |_| Ok(AimReply::default()));
        assert_eq!(code, 0);
        assert!(matches!(seen, Some(AimRequest::WeaponStep(ref request)) if request.kind == AimStepKind::Birth && request.state.submunition));
    }

    #[test]
    fn a_weapon_step_the_campaign_cannot_hold_never_reaches_the_host_and_an_answer_it_cannot_take_is_refused() {
        let mut nan = step_request(sys::EECH_AIM_WEAPON_STEP);
        nan.weapon_state.position[1] = f32::NAN;
        let mut draw = step_request(sys::EECH_AIM_WEAPON_STEP);
        draw.draw_count = 1;
        draw.draws[0] = 40000;
        let mut answers = step_request(sys::EECH_AIM_WEAPON_STEP);
        answers.answer_count = sys::EECH_STEP_ANSWERS_MAX as c_int + 1;
        let mut sample = step_request(sys::EECH_AIM_WEAPON_STEP);
        sample.weapon_state.entity_samples[3] = -1;
        let mut flag = step_request(sys::EECH_AIM_WEAPON_STEP);
        flag.weapon_state.has_target = 2;
        let mut nobody = step_request(sys::EECH_AIM_WEAPON_STEP);
        nobody.weapon = 0;
        for (what, bad) in [
            ("position", nan),
            ("draw", draw),
            ("answers", answers),
            ("sample", sample),
            ("flag", flag),
            ("weapon", nobody),
        ] {
            let (seen, _, code, error) = aim_crossing(&bad, |_| panic!("the host must not be asked"));
            assert!(seen.is_none() && code != 0, "{what}: {error}");
        }
        let step = step_request(sys::EECH_AIM_WEAPON_STEP);
        let (_, _, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                need_lookup: true,
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("lookups"), "{error}");
        let (_, _, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                actions: vec![event(1, 0, 0.0); sys::EECH_STEP_ACTIONS_MAX + 1],
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("more than the"), "{error}");
        // lookups, a draw or actions answer only a step
        let (_, _, code, error) = aim_crossing(&blank_aim_step(sys::EECH_AIM_READY_COMMAND), |_| {
            Ok(AimReply {
                need_lookup: true,
                lookups: vec![event(7, 0, 0.0)],
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("cannot take them"), "{error}");
        let (_, _, code, error) = aim_crossing(&figures_step(sys::EECH_AIM_TABLE), |_| {
            Ok(AimReply {
                need_draw: true,
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("cannot take one"), "{error}");
        let (_, _, code, error) = aim_crossing(&figures_step(sys::EECH_AIM_RANGE), |_| {
            Ok(AimReply {
                actions: vec![event(1, 0, 0.0)],
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("cannot carry actions"), "{error}");
    }

    #[test]
    fn a_weapon_steps_collision_answers_are_kept_by_the_question_they_answer_so_the_host_may_choose_among_them() {
        // SAFETY: plain data; all-zero is a valid empty table.
        let mut hits: sys::campaign_step_hits = unsafe { std::mem::zeroed() };
        let (a, b, c) = (
            [0.0f32, 500.0, 0.0, 0.0, 500.0, 90.0],
            [0.0f32, 500.0, 0.0, 0.0, 500.0, 91.0],
            [0.0f32, 500.0, 0.0, 5.0, 500.0, 90.0],
        );
        let (first, second) = (0x10usize as *mut std::ffi::c_void, 0x20usize as *mut std::ffi::c_void);
        let find = |hits: &sys::campaign_step_hits, which: c_int, segment: &[f32; 6]| -> Option<*mut std::ffi::c_void> {
            let mut entity = 0x99usize as *mut std::ffi::c_void;
            // SAFETY: the table, the segment and the out pointer are valid for the call.
            let found = unsafe { sys::campaign_step_hits_find(hits, which, segment.as_ptr(), &mut entity) };
            (found != 0).then_some(entity)
        };
        // SAFETY: the table and the segments are valid for the calls; the entities are never dereferenced.
        unsafe {
            sys::campaign_step_hits_clear(&mut hits);
            // a hit, then a miss, then another hit: each question keeps its own answer, whatever order the host chooses them in
            sys::campaign_step_hits_add(&mut hits, 0, a.as_ptr(), first);
            sys::campaign_step_hits_add(&mut hits, 0, b.as_ptr(), std::ptr::null_mut());
            sys::campaign_step_hits_add(&mut hits, 0, c.as_ptr(), second);
        }
        assert_eq!(find(&hits, 0, &a), Some(first));
        assert_eq!(find(&hits, 0, &b), Some(std::ptr::null_mut()), "a miss is an answer, and not the last hit");
        assert_eq!(find(&hits, 0, &c), Some(second));
        // the question includes whose target it was about, and one never asked has no answer
        assert_eq!(find(&hits, 1, &a), None);
        assert_eq!(find(&hits, 0, &[1.0; 6]), None);
        // clearing forgets
        // SAFETY: as above.
        unsafe { sys::campaign_step_hits_clear(&mut hits) };
        assert_eq!(find(&hits, 0, &a), None);
    }

    #[test]
    fn an_aim_animate_request_carries_the_weapons_parachute_and_the_hosts_state_comes_back_and_a_state_it_cannot_hold_never_reaches_the_host() {
        let mut step = blank_aim_step(sys::EECH_AIM_ANIMATE);
        step.weapon = 5;
        step.animate_status = 1;
        step.animate_on_ground = 0;
        step.animate_lifetime = -0.7;
        step.animate_motion_y = -3.0;
        let (seen, result, code, _) = aim_crossing(&step, |_| {
            Ok(AimReply {
                animate_status: 2,
                animate_lifetime: Some(0.0),
                ..AimReply::default()
            })
        });
        assert_eq!(code, 0);
        let Some(AimRequest::Animate(request)) = seen else {
            panic!("the host was not asked to animate")
        };
        assert_eq!(
            (request.weapon, request.parachute_status, request.weapon_lifetime, request.motion_vector_y),
            (5, 1, -0.7, -3.0)
        );
        assert_eq!((result.animate_status, result.animate_write_lifetime, result.animate_lifetime), (2, 1, 0.0));
        let (_, result, code, _) = aim_crossing(&step, |_| {
            Ok(AimReply {
                animate_status: 3,
                ..AimReply::default()
            })
        });
        assert_eq!(
            (code, result.animate_status, result.animate_write_lifetime),
            (0, 3, 0),
            "a lifetime the animation did not reset is not written"
        );
        let mut bad_state = step_copy_animate(&step);
        bad_state.animate_status = 9;
        let mut bad_weapon = step_copy_animate(&step);
        bad_weapon.weapon = 0;
        let mut bad_number = step_copy_animate(&step);
        bad_number.animate_lifetime = f32::NAN;
        for (what, bad) in [("state", bad_state), ("weapon", bad_weapon), ("number", bad_number)] {
            let (seen, _, code, error) = aim_crossing(&bad, |_| panic!("the host must not be asked"));
            assert!(seen.is_none() && code != 0, "{what}: {error}");
        }
    }

    fn step_copy_animate(step: &sys::eech_aim_step) -> sys::eech_aim_step {
        let mut copy = blank_aim_step(sys::EECH_AIM_ANIMATE);
        copy.weapon = step.weapon;
        copy.animate_status = step.animate_status;
        copy.animate_on_ground = step.animate_on_ground;
        copy.animate_lifetime = step.animate_lifetime;
        copy.animate_motion_y = step.animate_motion_y;
        copy
    }

    #[test]
    fn a_decoy_update_and_a_seduction_are_weapon_steps_with_their_own_kind_and_the_seductions_settings_cross_exactly() {
        let decoy = step_request(sys::EECH_AIM_WEAPON_DECOY);
        let (seen, _, code, _) = aim_crossing(&decoy, |_| Ok(AimReply::default()));
        assert_eq!(code, 0);
        assert!(matches!(seen, Some(AimRequest::WeaponStep(ref request)) if request.kind == AimStepKind::Decoy));
        let mut seduce = step_request(sys::EECH_AIM_WEAPON_SEDUCE);
        seduce.weapon_state.other_weapon = 21;
        seduce.weapon_state.coefficient = 0.5;
        seduce.weapon_state.effectiveness = [0.5, 0.25, 1.0];
        let (seen, result, code, _) = aim_crossing(&seduce, |_| {
            Ok(AimReply {
                actions: vec![event(10, 0, 1.0)],
                ..AimReply::default()
            })
        });
        assert_eq!(code, 0);
        let Some(AimRequest::WeaponStep(request)) = seen else { panic!("not a step") };
        assert_eq!(request.kind, AimStepKind::Seduce);
        assert_eq!(
            (request.state.other_weapon, request.state.coefficient, request.state.effectiveness),
            (21, 0.5, [0.5, 0.25, 1.0])
        );
        assert_eq!((result.state, result.action_count), (sys::EECH_AIM_DONE, 1));
        // either asks for a sample, and neither is given a setting that is not a number
        let (_, result, code, _) = aim_crossing(&seduce, |_| {
            Ok(AimReply {
                need_draw: true,
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state), (0, sys::EECH_AIM_NEED_DRAW));
        seduce.weapon_state.effectiveness[2] = f32::NAN;
        let (seen, _, code, error) = aim_crossing(&seduce, |_| panic!("the host must not be asked"));
        assert!(seen.is_none() && code != 0, "{error}");
    }

    #[test]
    fn a_reacquisition_is_a_weapon_step_of_its_own_kind_that_names_the_decoy_and_can_take_neither_a_sample_nor_a_lookup_answer_it_did_not_ask_for() {
        let mut reacquire = step_request(sys::EECH_AIM_WEAPON_REACQUIRE);
        reacquire.weapon_state.other_weapon = 21;
        let (seen, result, code, error) = aim_crossing(&reacquire, |_| {
            Ok(AimReply {
                actions: vec![event(11, 0, 0.0)],
                ..AimReply::default()
            })
        });
        assert_eq!(code, 0, "{error}");
        let Some(AimRequest::WeaponStep(request)) = seen else { panic!("not a step") };
        assert_eq!((request.kind, request.state.other_weapon), (AimStepKind::Reacquire, 21));
        assert_eq!((result.state, result.action_count, result.actions[0].kind), (sys::EECH_AIM_DONE, 1, 11));
        // it may ask for a lookup or a sample like any step, which the campaign refuses it in the handler; the engine boundary only carries them
        let (_, result, code, _) = aim_crossing(&reacquire, |_| {
            Ok(AimReply {
                need_draw: true,
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state), (0, sys::EECH_AIM_NEED_DRAW));
        let (_, result, code, _) = aim_crossing(&reacquire, |_| {
            Ok(AimReply {
                need_lookup: true,
                lookups: vec![event(8, 0, 0.0)],
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state, result.lookup_count), (0, sys::EECH_AIM_NEED_LOOKUP, 1));
    }

    fn carried_target_identity(seen: Option<AimRequest>, what: &str) -> Option<String> {
        match seen {
            Some(AimRequest::Damage(d)) => d.target_host_id,
            Some(AimRequest::Health(h)) => h.target_host_id,
            Some(AimRequest::Impact(i)) => i.target_host_id,
            other => panic!("{what}: {other:?}"),
        }
    }

    #[test]
    fn the_identity_of_the_object_a_damage_a_health_and_an_impact_question_is_about_crosses_and_a_bad_one_is_refused() {
        // a hit, a health fact and a weapon's death about an object that has an identity carry it; one about an object that has none carries none
        for (what, mut step) in [("damage", damage_step()), ("health", health_step(3)), ("impact", impact_step())] {
            step.target_host_id = test_id("veh:7");
            let (seen, _, _, _) = aim_crossing(&step, |_| Ok(AimReply::default()));
            assert_eq!(carried_target_identity(seen, what).as_deref(), Some("veh:7"), "{what}");
            step.target_host_id = [0 as c_char; sys::EECH_AIM_HOST_ID_MAX + 1];
            let (seen, _, _, _) = aim_crossing(&step, |_| Ok(AimReply::default()));
            assert_eq!(carried_target_identity(seen, what), None, "{what}: an object with no identity carries none");
            // an identity that does not end inside its field, and one that is not UTF-8, are refused before the host is asked
            step.target_host_id = [b'x' as c_char; sys::EECH_AIM_HOST_ID_MAX + 1];
            let (seen, _, code, error) = aim_crossing(&step, |_| Ok(AimReply::default()));
            assert!(
                code != 0 && seen.is_none() && error.contains("host identity"),
                "{what}: overlong: {code} {error}"
            );
            step.target_host_id = test_id("veh:7");
            step.target_host_id[0] = 0xff_u8 as c_char;
            let (seen, _, code, error) = aim_crossing(&step, |_| Ok(AimReply::default()));
            assert!(
                code != 0 && seen.is_none() && error.contains("host identity"),
                "{what}: not UTF-8: {code} {error}"
            );
        }
    }

    fn impact_step() -> sys::eech_aim_step {
        let mut step = blank_aim_step(sys::EECH_AIM_IMPACT);
        step.impact.kill_code = 5;
        step.impact.has_target = 1;
        step.impact.target_alive = 1;
        step.impact.position = [250.0, 12.0, 750.0];
        step.target_host_id = test_id("site:9");
        step
    }

    #[test]
    fn an_impact_question_carries_the_kill_code_and_the_targets_facts_asks_for_the_terrain_once_and_the_plan_comes_back() {
        let mut step = impact_step();
        let (seen, result, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                need_lookup: true,
                impact_need_terrain: true,
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state, result.impact.need_terrain), (0, sys::EECH_AIM_NEED_LOOKUP, 1), "{error}");
        assert_eq!(
            seen,
            Some(AimRequest::Impact(AimImpact {
                kill_code: 5,
                has_target: true,
                target_alive: true,
                terrain_draws_crater: None,
                position: [250.0, 12.0, 750.0],
                target_host_id: Some("site:9".into()),
            }))
        );
        // asked again with the terrain the campaign read, the host plans: a crater, a direct hit and two blasts that leave the target out
        step.impact.terrain_known = 1;
        step.impact.terrain_draws_crater = 1;
        let (seen, result, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                impact_crater: true,
                impact_direct_hit: true,
                impact_blasts: 2,
                impact_exclude_target: true,
                ..AimReply::default()
            })
        });
        assert_eq!(code, 0, "{error}");
        assert!(matches!(seen, Some(AimRequest::Impact(ref r)) if r.terrain_draws_crater == Some(true)));
        let p = result.impact;
        assert_eq!(
            (result.state, p.need_terrain, p.invalid, p.crater, p.direct_hit, p.blasts, p.exclude_target),
            (sys::EECH_AIM_DONE, 0, 0, 1, 1, 2, 1)
        );
        // a code the host does not know is carried as such
        let (_, result, code, _) = aim_crossing(&step, |_| {
            Ok(AimReply {
                impact_invalid: true,
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.impact.invalid), (0, 1));
    }

    #[test]
    fn an_impact_question_refuses_what_has_no_defined_answer_and_never_takes_a_sample() {
        let step = impact_step();
        // a sample is not part of the exchange
        let (_, _, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                need_draw: true,
                ..AimReply::default()
            })
        });
        assert_ne!(code, 0, "an impact question may not ask for a sample");
        assert!(error.contains("cannot take one"), "{error}");
        // a lookup other than the terrain
        let (_, _, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                need_lookup: true,
                ..AimReply::default()
            })
        });
        assert_ne!(code, 0);
        assert!(error.contains("terrain"), "{error}");
        // a request that is not one: a position that is not finite, a target that is alive and not there, a crater on a terrain not read, a flag that is not a boolean
        for (name, change) in [
            (
                "position",
                Box::new(|s: &mut sys::eech_aim_step| s.impact.position[1] = f32::NAN) as Box<dyn Fn(&mut sys::eech_aim_step)>,
            ),
            ("alive without target", Box::new(|s: &mut sys::eech_aim_step| s.impact.has_target = 0)),
            ("crater unread", Box::new(|s: &mut sys::eech_aim_step| s.impact.terrain_draws_crater = 1)),
            ("flag", Box::new(|s: &mut sys::eech_aim_step| s.impact.terrain_known = 2)),
        ] {
            let mut bad = impact_step();
            change(&mut bad);
            let (seen, _, code, _) = aim_crossing(&bad, |_| Ok(AimReply::default()));
            assert_ne!(code, 0, "{name}");
            assert!(seen.is_none(), "{name}: the host was asked");
        }
    }

    fn blast_step(window: bool) -> sys::eech_aim_step {
        let mut step = blank_aim_step(sys::EECH_AIM_BLAST);
        step.blast.position = [150.0, 5.0, 150.0];
        step.blast.damage_capability = 400;
        step.blast.warhead_type = 7;
        step.blast.has_target = 1;
        step.blast.sector_side_length = 100;
        step.blast.max_x_sector = 2;
        step.blast.max_z_sector = 2;
        if window {
            step.blast.window_valid = 1;
            step.blast.window_sx = 1;
            step.blast.window_sz = 1;
            step.blast.window_after = 7;
            step.blast.window_count = 2;
            step.blast.window_end = 1;
            step.blast.window[0] = sys::eech_blast_entry {
                handle: 8,
                kind: 6,
                readable: 1,
                alive: 1,
                has_position: 1,
                rocket: 0,
                excluded: 0,
                position: [160.0, 5.0, 150.0],
                host_id: [0 as c_char; sys::EECH_AIM_HOST_ID_MAX + 1],
            };
            step.blast.window[1] = sys::eech_blast_entry {
                handle: 9,
                kind: 37,
                readable: 1,
                alive: 1,
                has_position: 1,
                rocket: 1,
                excluded: 1,
                position: [155.0, 5.0, 150.0],
                host_id: test_id("mob-9"),
            };
        }
        step
    }

    #[test]
    fn a_blast_question_carries_the_explosion_and_the_window_by_value_and_the_hosts_walk_comes_back() {
        // the first question has no window: the host asks for one (the sector, and the handle it follows)
        let (seen, result, code, error) = aim_crossing(&blast_step(false), |_| {
            Ok(AimReply {
                need_lookup: true,
                blast_radius: 20.0,
                blast_need: Some((1, 1, 0)),
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state), (0, sys::EECH_AIM_NEED_LOOKUP), "{error}");
        assert_eq!(
            (result.blast.radius, result.blast.need_sx, result.blast.need_sz, result.blast.need_after),
            (20.0, 1, 1, 0)
        );
        let Some(AimRequest::Blast(first)) = seen else {
            panic!("the host was not asked about the blast")
        };
        assert_eq!(
            (first.position, first.damage_capability, first.warhead_type, first.has_target),
            ([150.0, 5.0, 150.0], 400, 7, true)
        );
        assert_eq!(
            (
                first.sector_side_length,
                first.min_x_sector,
                first.max_x_sector,
                first.min_z_sector,
                first.max_z_sector
            ),
            (100, 0, 2, 0, 2)
        );
        assert!(first.window.is_none());
        // supplied the window, the host reaches an object and the campaign is told what to do with it
        let (seen, result, code, error) = aim_crossing(&blast_step(true), |_| {
            Ok(AimReply {
                blast_radius: 20.0,
                blast_action: 1,
                blast_handle: 8,
                blast_modifier: 0.5,
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state), (0, sys::EECH_AIM_DONE), "{error}");
        assert_eq!(
            (result.blast.radius, result.blast.action, result.blast.handle, result.blast.modifier),
            (20.0, 1, 8, 0.5)
        );
        let Some(AimRequest::Blast(second)) = seen else {
            panic!("the host was not asked about the blast")
        };
        let Some(window) = second.window else { panic!("the window did not cross") };
        assert_eq!((window.sx, window.sz, window.after, window.end, window.entries.len()), (1, 1, 7, true, 2));
        assert_eq!(
            window.entries[1],
            AimBlastEntry {
                handle: 9,
                entity_type: 37,
                readable: true,
                alive: true,
                has_position: true,
                rocket: true,
                excluded: true,
                position: [155.0, 5.0, 150.0],
                host_id: Some("mob-9".into()),
            }
        );
        // the walk's end names nothing
        let (_, result, code, _) = aim_crossing(&blast_step(true), |_| {
            Ok(AimReply {
                blast_radius: 20.0,
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.blast.action, result.blast.handle), (0, 0, 0));
    }

    #[test]
    fn a_blast_question_refuses_what_has_no_defined_answer_and_never_takes_a_sample() {
        let (_, _, code, error) = aim_crossing(&blast_step(true), |_| {
            Ok(AimReply {
                need_draw: true,
                ..AimReply::default()
            })
        });
        assert_ne!(code, 0, "a blast may not ask for a sample");
        assert!(error.contains("cannot take one"), "{error}");
        // a lookup that is not a window
        let (_, _, code, error) = aim_crossing(&blast_step(true), |_| {
            Ok(AimReply {
                need_lookup: true,
                blast_radius: 20.0,
                ..AimReply::default()
            })
        });
        assert_ne!(code, 0);
        assert!(error.contains("not a window"), "{error}");
        // a request the campaign would not send
        for (name, change) in [
            (
                "position",
                Box::new(|s: &mut sys::eech_aim_step| s.blast.position[0] = f32::INFINITY) as Box<dyn Fn(&mut sys::eech_aim_step)>,
            ),
            ("sector", Box::new(|s: &mut sys::eech_aim_step| s.blast.sector_side_length = 0)),
            ("rectangle", Box::new(|s: &mut sys::eech_aim_step| s.blast.min_x_sector = 5)),
            ("window count", Box::new(|s: &mut sys::eech_aim_step| s.blast.window_count = 17)),
            ("handle", Box::new(|s: &mut sys::eech_aim_step| s.blast.window[0].handle = 0)),
            (
                "object position",
                Box::new(|s: &mut sys::eech_aim_step| s.blast.window[1].position[2] = f32::NAN),
            ),
            ("flag", Box::new(|s: &mut sys::eech_aim_step| s.blast.window[0].alive = 2)),
            ("window not valid but counted", Box::new(|s: &mut sys::eech_aim_step| s.blast.window_valid = 0)),
        ] {
            let mut bad = blast_step(true);
            change(&mut bad);
            let (seen, _, code, _) = aim_crossing(&bad, |_| Ok(AimReply::default()));
            assert_ne!(code, 0, "{name}");
            assert!(seen.is_none(), "{name}: the host was asked");
        }
    }
    /// An identity as the NUL-terminated field a window entry carries.
    fn test_id(text: &str) -> [c_char; sys::EECH_AIM_HOST_ID_MAX + 1] {
        let mut field = [0 as c_char; sys::EECH_AIM_HOST_ID_MAX + 1];
        for (slot, byte) in field.iter_mut().zip(text.as_bytes()) {
            *slot = *byte as c_char;
        }
        field
    }
    fn bridge_step(op: c_int) -> sys::eech_aim_step {
        let mut step = blank_aim_step(sys::EECH_AIM_BRIDGE);
        step.bridge.op = op;
        step.bridge.segment_kind = 3;
        step.bridge.alive = 1;
        step.bridge.server = 1;
        step.bridge.position = [250.0, 120.0, 250.0];
        step.bridge.delta_time = 0.1;
        step
    }

    #[test]
    fn a_bridge_question_carries_the_facts_by_value_and_each_answer_comes_back_whole() {
        // KILL
        let (seen, result, code, error) = aim_crossing(&bridge_step(1), |_| {
            Ok(AimReply {
                bridge_kill: true,
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state, result.bridge.kill), (0, sys::EECH_AIM_DONE, 1), "{error}");
        assert!(
            matches!(seen, Some(AimRequest::Bridge(ref b)) if b.op == 1 && b.segment_kind == 3 && b.alive && b.server && b.terrain.is_none() && b.window.is_none())
        );
        // SHAPE
        let mut shape = bridge_step(2);
        shape.bridge.pred_present = 1;
        shape.bridge.succ_present = 1;
        shape.bridge.succ_alive = 1;
        let (seen, result, code, _) = aim_crossing(&shape, |_| {
            Ok(AimReply {
                bridge_variant: 2,
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.bridge.variant), (0, 2));
        assert!(matches!(seen, Some(AimRequest::Bridge(ref b)) if b.pred_present && !b.pred_alive && b.succ_present && b.succ_alive));
        // NEIGHBOUR
        let mut neighbour = bridge_step(3);
        neighbour.bridge.which = 1;
        neighbour.bridge.neighbour_present = 1;
        let (seen, result, code, _) = aim_crossing(&neighbour, |_| {
            Ok(AimReply {
                bridge_notify: true,
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.bridge.notify), (0, 1));
        assert!(matches!(seen, Some(AimRequest::Bridge(ref b)) if b.which == 1 && b.neighbour_present));
        // FALL: the host asks for the terrain, then, given it, answers the fall
        let (_, result, code, error) = aim_crossing(&bridge_step(4), |_| {
            Ok(AimReply {
                need_lookup: true,
                bridge_need_terrain: true,
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state, result.bridge.need_terrain), (0, sys::EECH_AIM_NEED_LOOKUP, 1), "{error}");
        let mut fall = bridge_step(4);
        fall.bridge.terrain_known = 1;
        fall.bridge.terrain_height = 100.0;
        fall.bridge.terrain_water = 1;
        let (seen, result, code, _) = aim_crossing(&fall, |_| {
            Ok(AimReply {
                bridge_fell: true,
                bridge_dropped_y: 119.0,
                bridge_floor_height: 99.0,
                bridge_remove: false,
                ..AimReply::default()
            })
        });
        assert_eq!(
            (
                code,
                result.bridge.fell,
                result.bridge.dropped_y,
                result.bridge.floor_height,
                result.bridge.landed,
                result.bridge.remove
            ),
            (0, 1, 119.0, 99.0, 0, 0)
        );
        assert!(matches!(seen, Some(AimRequest::Bridge(ref b)) if b.terrain == Some((100.0, true))));
        // REST: the terrain is asked for once, then the height the host rests the dead segment at comes back and nothing else
        let mut rest = bridge_step(6);
        rest.bridge.alive = 0;
        let (_, result, code, error) = aim_crossing(&rest, |_| {
            Ok(AimReply {
                need_lookup: true,
                bridge_need_terrain: true,
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state, result.bridge.need_terrain), (0, sys::EECH_AIM_NEED_LOOKUP, 1), "{error}");
        rest.bridge.terrain_known = 1;
        rest.bridge.terrain_height = 100.0;
        let (seen, result, code, _) = aim_crossing(&rest, |_| {
            Ok(AimReply {
                bridge_landed: true,
                bridge_landed_y: 100.0,
                ..AimReply::default()
            })
        });
        assert_eq!(
            (code, result.bridge.landed, result.bridge.landed_y, result.bridge.fell, result.bridge.remove),
            (0, 1, 100.0, 0, 0)
        );
        assert!(matches!(seen, Some(AimRequest::Bridge(ref b)) if b.op == 6 && !b.alive && b.terrain == Some((100.0, false))));
    }

    #[test]
    fn a_bridge_vehicles_walk_carries_its_window_and_the_hosts_walk_comes_back() {
        let mut step = bridge_step(5);
        step.bridge.segment_length = 10.0;
        step.bridge.sector_side_length = 100;
        step.bridge.max_x_sector = 9;
        step.bridge.max_z_sector = 9;
        let (_, result, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                need_lookup: true,
                bridge_need: Some((2, 2, 0)),
                ..AimReply::default()
            })
        });
        assert_eq!(
            (code, result.state, result.bridge.need_sx, result.bridge.need_sz, result.bridge.need_after),
            (0, sys::EECH_AIM_NEED_LOOKUP, 2, 2, 0),
            "{error}"
        );
        step.bridge.window_valid = 1;
        step.bridge.window_sx = 2;
        step.bridge.window_sz = 2;
        step.bridge.window_count = 1;
        step.bridge.window_end = 1;
        step.bridge.window[0] = sys::eech_blast_entry {
            handle: 4,
            kind: 23,
            readable: 1,
            alive: 1,
            has_position: 1,
            rocket: 0,
            excluded: 0,
            position: [255.0, 5.0, 250.0],
            host_id: test_id("veh-4"),
        };
        let (seen, result, code, _) = aim_crossing(&step, |_| {
            Ok(AimReply {
                bridge_action: 1,
                bridge_handle: 4,
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.bridge.action, result.bridge.handle), (0, 1, 4));
        assert!(matches!(seen, Some(AimRequest::Bridge(ref b)) if b.window.as_ref().is_some_and(|w| w.entries.len() == 1 && w.entries[0].handle == 4)));
    }

    #[test]
    fn a_bridge_question_refuses_what_has_no_defined_answer_and_never_takes_a_sample() {
        let (_, _, code, error) = aim_crossing(&bridge_step(1), |_| {
            Ok(AimReply {
                need_draw: true,
                ..AimReply::default()
            })
        });
        assert_ne!(code, 0, "a bridge question may not ask for a sample");
        assert!(error.contains("cannot take one"), "{error}");
        // a lookup that is neither the terrain nor a window
        let (_, _, code, error) = aim_crossing(&bridge_step(4), |_| {
            Ok(AimReply {
                need_lookup: true,
                ..AimReply::default()
            })
        });
        assert_ne!(code, 0);
        assert!(error.contains("neither the terrain nor a window"), "{error}");
        // a request the campaign would not send
        for (name, change) in [
            (
                "op",
                Box::new(|s: &mut sys::eech_aim_step| s.bridge.op = 9) as Box<dyn Fn(&mut sys::eech_aim_step)>,
            ),
            ("position", Box::new(|s: &mut sys::eech_aim_step| s.bridge.position[1] = f32::NAN)),
            ("delta time", Box::new(|s: &mut sys::eech_aim_step| s.bridge.delta_time = f32::INFINITY)),
            ("flag", Box::new(|s: &mut sys::eech_aim_step| s.bridge.alive = 2)),
            ("length", Box::new(|s: &mut sys::eech_aim_step| s.bridge.segment_length = -1.0)),
            ("window on a fall", Box::new(|s: &mut sys::eech_aim_step| s.bridge.window_valid = 1)),
        ] {
            let mut bad = bridge_step(4);
            change(&mut bad);
            let (seen, _, code, _) = aim_crossing(&bad, |_| Ok(AimReply::default()));
            assert_ne!(code, 0, "{name}");
            assert!(seen.is_none(), "{name}: the host was asked");
        }
    }
    fn identify_step() -> sys::eech_aim_step {
        let mut step = blank_aim_step(sys::EECH_AIM_IDENTIFY);
        step.identify.entity_type = 5;
        step.identify.sub_type = 41;
        step.identify.side = 2;
        step.identify.alive = 1;
        step.identify.damage_level = 80;
        step.identify.initial_damage_level = 100;
        step.identify.position = [100.0, 5.0, 250.0];
        step.identify.serial = 3;
        step
    }
    fn text_of(field: &[c_char]) -> String {
        field.iter().map(|byte| *byte as u8).take_while(|byte| *byte != 0).map(char::from).collect()
    }

    #[test]
    fn a_structure_identity_is_asked_of_the_host_by_value_and_the_one_it_issues_comes_back_whole() {
        let (seen, result, code, error) = aim_crossing(&identify_step(), |_| {
            Ok(AimReply {
                identify_host_id: Some("eech-world:object:7".into()),
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state), (0, sys::EECH_AIM_DONE), "{error}");
        assert_eq!(text_of(&result.identify.host_id), "eech-world:object:7");
        assert_eq!(
            seen,
            Some(AimRequest::Identify(AimIdentify {
                entity_type: 5,
                sub_type: 41,
                side: 2,
                alive: true,
                damage_level: 80,
                initial_damage_level: 100,
                position: [100.0, 5.0, 250.0],
                serial: 3,
            }))
        );
        // what has no defined answer: no identity, an overlong or empty one, a NUL inside it, a sample, a lookup
        for (what, reply) in [
            ("no identity", AimReply::default()),
            (
                "an overlong identity",
                AimReply {
                    identify_host_id: Some("x".repeat(129)),
                    ..AimReply::default()
                },
            ),
            (
                "an empty identity",
                AimReply {
                    identify_host_id: Some(String::new()),
                    ..AimReply::default()
                },
            ),
            (
                "a NUL inside the identity",
                AimReply {
                    identify_host_id: Some("a\0b".into()),
                    ..AimReply::default()
                },
            ),
            (
                "a sample",
                AimReply {
                    need_draw: true,
                    identify_host_id: Some("a".into()),
                    ..AimReply::default()
                },
            ),
            (
                "a lookup",
                AimReply {
                    need_lookup: true,
                    identify_host_id: Some("a".into()),
                    ..AimReply::default()
                },
            ),
        ] {
            let (_, _, code, error) = aim_crossing(&identify_step(), |_| Ok(reply.clone()));
            assert_ne!(code, 0, "{what} was accepted");
            assert!(
                error.contains("identity") || error.contains("identify") || error.contains("draw") || error.contains("lookup"),
                "{what}: {error}"
            );
        }
        // a request the campaign would not send
        type Change = fn(&mut sys::eech_aim_step);
        let changes: [(&str, Change); 4] = [
            ("a non-finite position", |s| s.identify.position[1] = f32::NAN),
            ("a serial that does not count from one", |s| s.identify.serial = 0),
            ("a flag that is not a boolean", |s| s.identify.alive = 2),
            ("a negative damage level", |s| s.identify.damage_level = -1),
        ];
        for (name, change) in changes {
            let mut bad = identify_step();
            change(&mut bad);
            let (seen, _, code, _) = aim_crossing(&bad, |_| Ok(AimReply::default()));
            assert_ne!(code, 0, "{name}");
            assert!(seen.is_none(), "{name}: the host was asked");
        }
    }

    #[test]
    fn the_identity_of_an_offered_object_crosses_and_the_hosts_repeat_of_it_comes_back() {
        // a blast: the entry carries the identity the host gave the object (none for a weapon), and the host repeats it for the object it reaches
        let (seen, result, code, error) = aim_crossing(&blast_step(true), |_| {
            Ok(AimReply {
                blast_radius: 20.0,
                blast_action: 1,
                blast_handle: 9,
                blast_modifier: 0.5,
                blast_target_host_id: Some("mob-9".into()),
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state), (0, sys::EECH_AIM_DONE), "{error}");
        assert_eq!(text_of(&result.blast.target_host_id), "mob-9");
        let Some(AimRequest::Blast(blast)) = seen else { panic!("not a blast") };
        let Some(window) = blast.window else { panic!("the window did not cross") };
        let ids: Vec<Option<String>> = window.entries.into_iter().map(|e| e.host_id).collect();
        assert_eq!(ids, vec![None, Some("mob-9".to_string())]);
        // none repeated is none on the wire; an overlong one is refused
        let (_, result, code, _) = aim_crossing(&blast_step(true), |_| {
            Ok(AimReply {
                blast_radius: 20.0,
                blast_action: 1,
                blast_handle: 8,
                blast_modifier: 0.5,
                ..AimReply::default()
            })
        });
        assert_eq!((code, text_of(&result.blast.target_host_id)), (0, String::new()));
        let (_, _, code, error) = aim_crossing(&blast_step(true), |_| {
            Ok(AimReply {
                blast_radius: 20.0,
                blast_action: 1,
                blast_handle: 9,
                blast_modifier: 0.5,
                blast_target_host_id: Some("x".repeat(129)),
                ..AimReply::default()
            })
        });
        assert_ne!(code, 0);
        assert!(error.contains("identity"), "{error}");
        // a bridge vehicles walk: the same
        let mut step = bridge_step(5);
        step.bridge.segment_length = 10.0;
        step.bridge.sector_side_length = 100;
        step.bridge.max_x_sector = 9;
        step.bridge.max_z_sector = 9;
        step.bridge.window_valid = 1;
        step.bridge.window_sx = 2;
        step.bridge.window_sz = 2;
        step.bridge.window_count = 1;
        step.bridge.window_end = 1;
        step.bridge.window[0] = sys::eech_blast_entry {
            handle: 4,
            kind: 23,
            readable: 1,
            alive: 1,
            has_position: 1,
            rocket: 0,
            excluded: 0,
            position: [255.0, 5.0, 250.0],
            host_id: test_id("veh-4"),
        };
        let (seen, result, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                bridge_action: 1,
                bridge_handle: 4,
                bridge_target_host_id: Some("veh-4".into()),
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state, result.bridge.handle), (0, sys::EECH_AIM_DONE, 4), "{error}");
        assert_eq!(text_of(&result.bridge.target_host_id), "veh-4");
        let Some(AimRequest::Bridge(bridge)) = seen else {
            panic!("not a bridge question")
        };
        let Some(window) = bridge.window else { panic!("the window did not cross") };
        assert_eq!(window.entries[0].host_id.as_deref(), Some("veh-4"));
    }

    // The identity DTOs are mirrored by hand, field for field; as for the weapon state, an outer size does not establish the inner order, so every field's offset and the alignment are read from the campaign as
    // compiled (csrc/eech_aim_layout.c cases 130 to 164).
    #[test]
    fn the_object_identity_dtos_have_the_layout_the_campaign_compiled() {
        use std::mem::{align_of, offset_of, size_of};
        let layout = |which: c_int| unsafe { sys::eech_engine_aim_layout(which) };
        assert_eq!(layout(130), size_of::<sys::eech_identify_request>(), "campaign_identify_request");
        assert_eq!(layout(131), offset_of!(sys::eech_identify_request, sub_type));
        assert_eq!(layout(132), offset_of!(sys::eech_identify_request, side));
        assert_eq!(layout(133), offset_of!(sys::eech_identify_request, alive));
        assert_eq!(layout(134), offset_of!(sys::eech_identify_request, damage_level));
        assert_eq!(layout(135), offset_of!(sys::eech_identify_request, initial_damage_level));
        assert_eq!(layout(136), offset_of!(sys::eech_identify_request, position));
        assert_eq!(layout(137), offset_of!(sys::eech_identify_request, serial));
        assert_eq!(layout(138), size_of::<sys::eech_identify_result>(), "campaign_identify_result");
        assert_eq!(layout(139), offset_of!(sys::eech_aim_step, identify));
        assert_eq!(layout(140), offset_of!(sys::eech_aim_result, identify));
        assert_eq!(layout(141), offset_of!(sys::eech_blast_entry, host_id));
        assert_eq!(layout(142), offset_of!(sys::eech_blast_result, target_host_id));
        assert_eq!(layout(143), offset_of!(sys::eech_bridge_result, target_host_id));
        assert_eq!(layout(144), sys::EECH_AIM_IDENTIFY as usize);
        assert_eq!(layout(145), size_of::<sys::eech_object_report>(), "eech_object_report");
        assert_eq!(layout(146), offset_of!(sys::eech_object_report, has_damage_level));
        assert_eq!(layout(147), offset_of!(sys::eech_object_report, damage_level));
        assert_eq!(layout(148), offset_of!(sys::eech_object_report, destroyed));
        assert_eq!(layout(149), size_of::<sys::eech_object_report_receipt>(), "eech_object_report_receipt");
        assert_eq!(layout(150), offset_of!(sys::eech_object_report_receipt, rostered));
        assert_eq!(layout(151), offset_of!(sys::eech_object_report_receipt, damage_level_before));
        assert_eq!(layout(152), offset_of!(sys::eech_object_report_receipt, damage_level_after));
        assert_eq!(layout(153), offset_of!(sys::eech_object_report_receipt, killed));
        assert_eq!(layout(154), size_of::<sys::eech_structure>(), "eech_structure");
        assert_eq!(layout(165), offset_of!(sys::eech_structure, kind));
        assert_eq!(layout(155), offset_of!(sys::eech_structure, entity_type));
        assert_eq!(layout(156), offset_of!(sys::eech_structure, sub_type));
        assert_eq!(layout(157), offset_of!(sys::eech_structure, side));
        assert_eq!(layout(158), offset_of!(sys::eech_structure, alive));
        assert_eq!(layout(159), offset_of!(sys::eech_structure, damage_level));
        assert_eq!(layout(160), offset_of!(sys::eech_structure, initial_damage_level));
        assert_eq!(layout(161), offset_of!(sys::eech_structure, position));
        assert_eq!(layout(162), align_of::<sys::eech_identify_request>());
        assert_eq!(layout(163), align_of::<sys::eech_object_report>());
        assert_eq!(layout(164), align_of::<sys::eech_structure>());
        // and the first field of each, which sits at zero in both
        assert_eq!(offset_of!(sys::eech_identify_request, entity_type), 0);
        assert_eq!(offset_of!(sys::eech_object_report, host_id), 0);
        assert_eq!(offset_of!(sys::eech_structure, host_id), 0);
    }

    // The weapon state is embedded in the C-layout aim step and read by the campaign as campaign_weapon_state, so its INNER field order is part of the ABI: a same-size outer offset does not
    // establish it.  Every field's offset, the size and the alignment are read from the campaign as compiled (csrc/eech_aim_layout.c cases 100 to 120).
    fn weapon_state_layout_mismatches(rust_size: usize, rust_align: usize, rust_offsets: [usize; 19]) -> Vec<usize> {
        let layout = |which: c_int| unsafe { sys::eech_engine_aim_layout(which) };
        let mut wrong = Vec::new();
        if layout(100) != rust_size {
            wrong.push(100);
        }
        if layout(101) != rust_align {
            wrong.push(101);
        }
        for (index, offset) in rust_offsets.iter().enumerate() {
            if layout(102 + index as c_int) != *offset {
                wrong.push(102 + index);
            }
        }
        wrong
    }
    #[test]
    fn the_inner_layout_of_the_weapon_state_is_the_campaigns() {
        use std::mem::{align_of, offset_of, size_of};
        type W = sys::eech_weapon_state;
        let offsets = [
            offset_of!(W, position),
            offset_of!(W, attitude),
            offset_of!(W, motion_vector),
            offset_of!(W, velocity),
            offset_of!(W, weapon_lifetime),
            offset_of!(W, decoy_timer),
            offset_of!(W, missile_phase),
            offset_of!(W, loal_mode),
            offset_of!(W, weapon_on_ground),
            offset_of!(W, parachute_status),
            offset_of!(W, kill_code),
            offset_of!(W, entity_samples),
            offset_of!(W, has_launcher),
            offset_of!(W, has_target),
            offset_of!(W, server),
            offset_of!(W, submunition),
            offset_of!(W, other_weapon),
            offset_of!(W, coefficient),
            offset_of!(W, effectiveness),
        ];
        assert_eq!(weapon_state_layout_mismatches(size_of::<W>(), align_of::<W>(), offsets), Vec::<usize>::new());
        // A control: the comparison does fail for a mirror whose fields are in another order (the first two integers swapped), so a pass above is not an empty check.
        #[repr(C)]
        struct Reordered {
            position: [f32; 3],
            attitude: [f32; 9],
            motion_vector: [f32; 3],
            velocity: f32,
            weapon_lifetime: f32,
            decoy_timer: f32,
            loal_mode: c_int,
            missile_phase: c_int,
            weapon_on_ground: c_int,
            parachute_status: c_int,
            kill_code: c_int,
            entity_samples: [c_int; 4],
            has_launcher: c_int,
            has_target: c_int,
            server: c_int,
            submunition: c_int,
            other_weapon: c_int,
            coefficient: f32,
            effectiveness: [f32; 3],
        }
        let control = [
            offset_of!(Reordered, position),
            offset_of!(Reordered, attitude),
            offset_of!(Reordered, motion_vector),
            offset_of!(Reordered, velocity),
            offset_of!(Reordered, weapon_lifetime),
            offset_of!(Reordered, decoy_timer),
            offset_of!(Reordered, missile_phase),
            offset_of!(Reordered, loal_mode),
            offset_of!(Reordered, weapon_on_ground),
            offset_of!(Reordered, parachute_status),
            offset_of!(Reordered, kill_code),
            offset_of!(Reordered, entity_samples),
            offset_of!(Reordered, has_launcher),
            offset_of!(Reordered, has_target),
            offset_of!(Reordered, server),
            offset_of!(Reordered, submunition),
            offset_of!(Reordered, other_weapon),
            offset_of!(Reordered, coefficient),
            offset_of!(Reordered, effectiveness),
        ];
        // by NAME, the reordered mirror puts missile_phase and loal_mode in each other's slots: the comparison names exactly those two (size and alignment still agree)
        assert_eq!(
            weapon_state_layout_mismatches(size_of::<Reordered>(), align_of::<Reordered>(), control),
            vec![108, 109]
        );
    }
    // The Rust mirrors of the aim exchange are written by hand, field for field; one that drifts from the C structure is memory corruption at the first request.  The campaign
    // reports the sizes and the offsets of the fields appended last (csrc/eech_aim_layout.c) and the mirrors must agree.
    #[test]
    fn the_rust_mirrors_of_the_aim_exchange_have_the_layout_the_campaign_compiled() {
        use std::mem::{offset_of, size_of};
        let layout = |which: c_int| unsafe { sys::eech_engine_aim_layout(which) };
        assert_eq!(layout(0), size_of::<sys::eech_aim_step>(), "campaign_aim_step");
        assert_eq!(layout(1), size_of::<sys::eech_aim_result>(), "campaign_aim_result");
        assert_eq!(layout(2), size_of::<sys::eech_weapon_figures>(), "campaign_weapon_figures");
        assert_eq!(layout(3), size_of::<sys::eech_damage_figures>(), "campaign_damage_figures");
        assert_eq!(layout(4), offset_of!(sys::eech_aim_step, release_not_ai));
        assert_eq!(layout(5), offset_of!(sys::eech_aim_step, animate_motion_y));
        assert_eq!(layout(6), offset_of!(sys::eech_aim_step, forecast_figures));
        assert_eq!(layout(7), offset_of!(sys::eech_aim_step, forecast_armour_level));
        assert_eq!(layout(8), offset_of!(sys::eech_aim_step, forecast_velocity));
        assert_eq!(layout(9), offset_of!(sys::eech_aim_result, release_last_shot));
        assert_eq!(layout(10), offset_of!(sys::eech_aim_result, animate_lifetime));
        assert_eq!(layout(11), offset_of!(sys::eech_aim_result, forecast_capability));
        assert_eq!(layout(12), offset_of!(sys::eech_damage_figures, burst_duration));
        assert_eq!(layout(13), offset_of!(sys::eech_aim_step, weapon_state));
        assert_eq!(layout(14), offset_of!(sys::eech_aim_step, answers));
        assert_eq!(layout(15), size_of::<sys::eech_damage_request>(), "campaign_damage_request");
        assert_eq!(layout(16), size_of::<sys::eech_damage_result>(), "campaign_damage_result");
        assert_eq!(layout(17), offset_of!(sys::eech_aim_step, damage));
        assert_eq!(layout(18), offset_of!(sys::eech_aim_result, damage));
        assert_eq!(layout(19), offset_of!(sys::eech_damage_request, weapon_speed));
        assert_eq!(layout(20), offset_of!(sys::eech_damage_request, armour_level));
        assert_eq!(layout(21), offset_of!(sys::eech_damage_request, armour_front));
        assert_eq!(layout(22), offset_of!(sys::eech_damage_request, target_attitude));
        assert_eq!(layout(23), offset_of!(sys::eech_damage_request, target_not_ai));
        assert_eq!(layout(24), offset_of!(sys::eech_damage_request, player_modifier));
        assert_eq!(layout(25), offset_of!(sys::eech_damage_request, critical_model));
        assert_eq!(layout(26), offset_of!(sys::eech_damage_result, critical_before));
        assert_eq!(layout(27), size_of::<sys::eech_health_request>(), "campaign_health_request");
        assert_eq!(layout(28), size_of::<sys::eech_health_result>(), "campaign_health_result");
        assert_eq!(layout(29), offset_of!(sys::eech_aim_step, health));
        assert_eq!(layout(30), offset_of!(sys::eech_aim_result, health));
        assert_eq!(layout(31), offset_of!(sys::eech_health_request, delta_time));
        assert_eq!(layout(32), offset_of!(sys::eech_health_request, aircraft_class));
        assert_eq!(layout(33), offset_of!(sys::eech_health_request, target_not_ai));
        assert_eq!(layout(34), offset_of!(sys::eech_health_result, eligible));
        assert_eq!(layout(35), offset_of!(sys::eech_health_result, fatal));
        // the impact and blast exchange (EEDCS-46)
        assert_eq!(layout(36), size_of::<sys::eech_impact_request>(), "campaign_impact_request");
        assert_eq!(layout(37), size_of::<sys::eech_impact_result>(), "campaign_impact_result");
        assert_eq!(layout(38), size_of::<sys::eech_blast_entry>(), "campaign_blast_entry");
        assert_eq!(layout(39), size_of::<sys::eech_blast_request>(), "campaign_blast_request");
        assert_eq!(layout(40), size_of::<sys::eech_blast_result>(), "campaign_blast_result");
        assert_eq!(layout(41), offset_of!(sys::eech_aim_step, impact));
        assert_eq!(layout(42), offset_of!(sys::eech_aim_step, blast));
        assert_eq!(layout(43), offset_of!(sys::eech_aim_result, impact));
        assert_eq!(layout(44), offset_of!(sys::eech_aim_result, blast));
        assert_eq!(layout(45), offset_of!(sys::eech_blast_request, window));
        assert_eq!(layout(46), offset_of!(sys::eech_blast_entry, position));
        assert_eq!(layout(47), offset_of!(sys::eech_blast_result, modifier));
        assert_eq!(layout(48), offset_of!(sys::eech_impact_request, position));
        assert_eq!(layout(49), offset_of!(sys::eech_blast_request, window_valid));
        assert_eq!(sys::EECH_BLAST_WINDOW_MAX, 16);
        // the bridge exchange (EEDCS-46, R5)
        assert_eq!(layout(50), size_of::<sys::eech_bridge_request>(), "campaign_bridge_request");
        assert_eq!(layout(51), size_of::<sys::eech_bridge_result>(), "campaign_bridge_result");
        assert_eq!(layout(52), offset_of!(sys::eech_aim_step, bridge));
        assert_eq!(layout(53), offset_of!(sys::eech_aim_result, bridge));
        assert_eq!(layout(54), offset_of!(sys::eech_bridge_request, position));
        assert_eq!(layout(55), offset_of!(sys::eech_bridge_request, terrain_known));
        assert_eq!(layout(56), offset_of!(sys::eech_bridge_request, segment_length));
        assert_eq!(layout(57), offset_of!(sys::eech_bridge_request, window));
        assert_eq!(layout(58), offset_of!(sys::eech_bridge_result, dropped_y));
        assert_eq!(layout(59), offset_of!(sys::eech_bridge_result, handle));
        // the campaign's entity sub types of a bridge segment the reference kernel hard-codes (end, supported mid, unsupported mid)
        assert_eq!((90..=92).map(layout).collect::<Vec<usize>>(), vec![1, 2, 3], "ENTITY_SUB_TYPE_FIXED_BRIDGE_*");
        assert_eq!(
            (93..=98).map(layout).collect::<Vec<usize>>(),
            vec![1, 2, 3, 4, 5, 6],
            "campaign_bridge_op: the numbers the reference host and the DC module name"
        );
        // the campaign's enum values the reference host's kernel hard-codes (eech-world csrc/cmb_step.c; its own test compares the same two lists): the thirteen kinds a blast
        // examines, in the order of the probe's cases 60 to 72, and the weapon kill codes 0 to 7
        let entity_types: Vec<usize> = (60..=72).map(layout).collect();
        assert_eq!(
            entity_types,
            vec![1, 6, 10, 14, 19, 21, 23, 24, 26, 28, 29, 30, 37],
            "ENTITY_TYPES the blast examines"
        );
        let kill_codes: Vec<usize> = (80..=87).map(layout).collect();
        assert_eq!(kill_codes, vec![0, 1, 2, 3, 4, 5, 6, 7], "WEAPON_KILL_CODES");
    }

    fn damage_step() -> sys::eech_aim_step {
        let mut step = blank_aim_step(sys::EECH_AIM_DAMAGE);
        step.weapon = 54;
        step.damage.figures = sys::eech_damage_figures {
            weapon_warhead: 5,
            warhead: 5,
            damage_capability: 100,
            armor_penetration_capability: 1,
            burst_duration: 0.0,
        };
        step.damage.weapon_speed = 300.0;
        step.damage.weapon_cruise_velocity = 250.0;
        step.damage.damage_modifier = 0.5;
        step.damage.armour_level = 4;
        step.damage.armour_model = 1;
        step.damage.armour_front = 1.0;
        step.damage.armour_rear = 0.5;
        step.damage.armour_side = 0.75;
        step.damage.weapon_position = [1.0, 2.0, 3.0];
        step.damage.target_position = [0.0, 0.0, 0.0];
        step.damage.target_attitude = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        step.damage.target_not_ai = 1;
        step.damage.player_modifier_valid = 1;
        step.damage.player_modifier = 0.66;
        step.damage.has_launcher = 1;
        step.damage.multiplier = 2;
        step.damage.damage_level = 80;
        step.damage.initial_damage_level = 100;
        step.damage.critical_model = 1;
        step
    }

    #[test]
    fn a_damage_request_carries_the_hit_by_value_takes_the_one_sample_and_the_hosts_answer_comes_back() {
        let mut step = damage_step();
        // the first question has no sample: the host asks for one (a damage question may; a forecast may not)
        let (seen, result, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                need_draw: true,
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state), (0, sys::EECH_AIM_NEED_DRAW), "{error}");
        let Some(AimRequest::Damage(first)) = seen else {
            panic!("the host was not asked about the hit")
        };
        assert_eq!(first.draw, None);
        // asked again with the sample the campaign took, the host answers
        step.has_draw = 1;
        step.draw_sample = 12345;
        let (seen, result, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                damage_capability: 61,
                damage_written: true,
                damage_new_level: 19,
                damage_critical_before: 0,
                damage_critical_after: 1,
                ..AimReply::default()
            })
        });
        assert_eq!(code, 0, "{error}");
        let Some(AimRequest::Damage(request)) = seen else {
            panic!("the host was not asked about the hit")
        };
        assert_eq!(request.draw, Some(12345));
        assert_eq!(
            (
                request.weapon,
                request.weapon_warhead,
                request.warhead,
                request.damage_capability,
                request.armor_penetration_capability
            ),
            (54, 5, 5, 100, 1)
        );
        assert_eq!(
            (request.weapon_speed, request.weapon_cruise_velocity, request.damage_modifier),
            (300.0, 250.0, 0.5)
        );
        assert_eq!((request.armour_level, request.armour_easy, request.armour_model), (4, false, 1));
        assert_eq!((request.armour_front, request.armour_rear, request.armour_side), (1.0, 0.5, 0.75));
        assert_eq!((request.weapon_position, request.target_position), ([1.0, 2.0, 3.0], [0.0; 3]));
        assert_eq!(request.target_attitude, [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
        assert_eq!(
            (request.target_not_ai, request.player_modifier, request.has_launcher, request.multiplier),
            (true, Some(0.66), true, 2)
        );
        assert_eq!((request.invulnerable, request.cheat_guard), (false, false));
        assert_eq!((request.damage_level, request.initial_damage_level, request.critical_model), (80, 100, 1));
        // the host's own numbers are what the campaign gets back, none computed in this layer
        let d = result.damage;
        assert_eq!(
            (
                result.state,
                d.exited,
                d.capability,
                d.written,
                d.new_level,
                d.critical_before,
                d.critical_after,
                d.bad_modifier
            ),
            (sys::EECH_AIM_DONE, 0, 61, 1, 19, 0, 1, 0)
        );
        // a player whose damage setting the campaign could not resolve is carried as a missing setting
        step.damage.player_modifier_valid = 0;
        let (seen, _, code, _) = aim_crossing(&step, |_| Ok(AimReply::default()));
        assert_eq!(code, 0);
        assert!(matches!(seen, Some(AimRequest::Damage(ref r)) if r.player_modifier.is_none()));
    }

    // The fixture build's echo of a hit travels as raw-bit halves (a hit with no direction has a not-a-number aspect armour); the engine writes the events into the room the
    // campaign gave, exactly.
    #[test]
    fn a_damage_echo_of_raw_bit_halves_reaches_the_campaigns_room_unchanged() {
        let mut room = [sys::eech_aim_event {
            kind: 0,
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            f: [0.0; 12],
        }; 4];
        let mut step = damage_step();
        step.has_draw = 1;
        step.draw_sample = 0;
        step.echo = 1;
        step.echo_out = room.as_mut_ptr();
        step.echo_capacity = 4;
        let mut f = [0.0f32; 12];
        f[1] = 15.0;
        f[3] = 0.75;
        f[6..].copy_from_slice(&[16752.0, 0.0, 65472.0, 0.0, 16192.0, 0.0]);
        let (_, result, code, error) = aim_crossing(&step, |request| {
            let Some(()) = matches!(request, AimRequest::Damage(ref r) if r.echo && r.draw == Some(0)).then_some(()) else {
                return Err("not the damage question with its echo".into());
            };
            Ok(AimReply {
                damage_new_level: 80,
                echo: Some(vec![AimEvent {
                    kind: sys::EECH_AIM_EVENT_DAMAGE,
                    a: 0,
                    b: 0,
                    c: 3,
                    d: 0,
                    f,
                }]),
                ..AimReply::default()
            })
        });
        assert_eq!(code, 0, "{error}");
        assert_eq!((result.has_echo, result.echo_count), (1, 1));
        assert_eq!(
            (room[0].kind, room[0].a, room[0].b, room[0].c, room[0].d),
            (sys::EECH_AIM_EVENT_DAMAGE, 0, 0, 3, 0)
        );
        assert_eq!(room[0].f, f);
        let armour_bits = ((room[0].f[8] as u32) << 16) | (room[0].f[9] as u32);
        assert_eq!(armour_bits, 0xFFC0_0000, "the not-a-number armour, sign and payload");
    }

    fn health_step(op: c_int) -> sys::eech_aim_step {
        let mut step = blank_aim_step(sys::EECH_AIM_HEALTH);
        step.health.op = op;
        step.health.critical_model = 1;
        step.health.damage_level = 14;
        step.health.initial_damage_level = 100;
        step.health.old_damage_level = 14;
        step.health.explosive = 1;
        step.health.delta_time = 0.1;
        step
    }

    #[test]
    fn a_health_question_crosses_by_value_a_timed_roll_takes_its_one_sample_and_the_answer_comes_back() {
        let mut step = health_step(3);
        // the host asks for the sample of an eligible target; the campaign gives it and the host rolls
        let (seen, result, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                need_draw: true,
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state), (0, sys::EECH_AIM_NEED_DRAW), "{error}");
        let Some(AimRequest::Health(first)) = seen else {
            panic!("the host was not asked")
        };
        assert_eq!(
            (
                first.op,
                first.critical_model,
                first.damage_level,
                first.initial_damage_level,
                first.explosive,
                first.flammable,
                first.draw
            ),
            (3, 1, 14, 100, true, false, None)
        );
        step.has_draw = 1;
        step.draw_sample = 6553;
        let (seen, result, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                health_critical_class: 1,
                health_eligible: true,
                health_rolled: true,
                health_hit: true,
                health_new_level: 13,
                ..AimReply::default()
            })
        });
        assert_eq!(code, 0, "{error}");
        let Some(AimRequest::Health(request)) = seen else {
            panic!("the host was not asked")
        };
        assert_eq!((request.draw, request.delta_time), (Some(6553), 0.1));
        let h = result.health;
        assert_eq!(
            (result.state, h.critical_class, h.eligible, h.rolled, h.hit, h.new_level, h.fatal, h.crossed),
            (sys::EECH_AIM_DONE, 1, 1, 1, 1, 13, 0, 0)
        );
        // a self-hit question carries its two facts
        let mut hit = health_step(4);
        hit.health.self_hit = 1;
        let (seen, _, code, _) = aim_crossing(&hit, |_| Ok(AimReply::default()));
        assert_eq!(code, 0);
        assert!(matches!(seen, Some(AimRequest::Health(ref r)) if r.op == 4 && r.self_hit && !r.target_not_ai));
    }

    // Which targets roll is the HOST's physical eligibility: a host with a model of its own may ask for the campaign's sample for a person with health or for an aircraft in no class, and
    // the engine boundary carries the request and the answer as it does for any other target.
    #[test]
    fn a_generic_hosts_timed_roll_for_a_person_or_an_aircraft_in_no_class_crosses_the_engine_boundary() {
        for (model, class) in [(2, 0), (3, 0)] {
            let mut step = health_step(3);
            step.health.critical_model = model;
            step.health.explosive = 0;
            step.health.aircraft_class = class;
            step.health.damage_level = 40;
            let (seen, result, code, error) = aim_crossing(&step, |_| {
                Ok(AimReply {
                    need_draw: true,
                    ..AimReply::default()
                })
            });
            assert_eq!((code, result.state), (0, sys::EECH_AIM_NEED_DRAW), "model {model}: {error}");
            assert!(matches!(seen, Some(AimRequest::Health(ref r)) if r.op == 3 && r.critical_model == model && r.draw.is_none()));
            step.has_draw = 1;
            step.draw_sample = 100;
            let (_, result, code, error) = aim_crossing(&step, |_| {
                Ok(AimReply {
                    health_eligible: true,
                    health_rolled: true,
                    health_hit: true,
                    health_new_level: 39,
                    ..AimReply::default()
                })
            });
            assert_eq!(code, 0, "model {model}: {error}");
            let h = result.health;
            assert_eq!((h.eligible, h.rolled, h.hit, h.new_level, h.fatal), (1, 1, 1, 39, 0));
        }
    }

    #[test]
    fn a_health_question_refuses_what_has_no_defined_answer_and_never_takes_a_lookup() {
        let ok = |_: AimRequest| Ok(AimReply::default());
        for (what, mutate) in [
            (
                "op",
                Box::new(|s: &mut sys::eech_aim_step| s.health.op = 0) as Box<dyn Fn(&mut sys::eech_aim_step)>,
            ),
            ("op high", Box::new(|s: &mut sys::eech_aim_step| s.health.op = 5)),
            ("model", Box::new(|s: &mut sys::eech_aim_step| s.health.critical_model = 4)),
            ("class", Box::new(|s: &mut sys::eech_aim_step| s.health.aircraft_class = 3)),
            ("flag", Box::new(|s: &mut sys::eech_aim_step| s.health.explosive = 2)),
            ("frame", Box::new(|s: &mut sys::eech_aim_step| s.health.delta_time = f32::NAN)),
            ("initial", Box::new(|s: &mut sys::eech_aim_step| s.health.initial_damage_level = 0)),
            (
                "sample",
                Box::new(|s: &mut sys::eech_aim_step| {
                    s.has_draw = 1;
                    s.draw_sample = 40000;
                }),
            ),
        ] {
            let mut bad = health_step(3);
            mutate(&mut bad);
            let (seen, _, code, error) = aim_crossing(&bad, ok);
            assert!(seen.is_none() && code != 0 && !error.is_empty(), "{what}: {error}");
        }
        // a person has no initial level to divide by and is not classified: a legal question
        let mut person = health_step(1);
        person.health.critical_model = 2;
        person.health.initial_damage_level = 0;
        let (seen, _, code, error) = aim_crossing(&person, ok);
        assert!(seen.is_some() && code == 0, "{error}");
        let step = health_step(1);
        let (_, _, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                need_lookup: true,
                lookups: vec![AimEvent {
                    kind: 1,
                    a: 0,
                    b: 0,
                    c: 0,
                    d: 0,
                    f: [0.0; 12],
                }],
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("cannot take them"), "{error}");
        let (_, _, code, error) = aim_crossing(&step, |_| Err("the host has no model for this health".into()));
        assert!(code != 0 && error.contains("no model"), "{error}");
    }

    #[test]
    fn a_damage_request_refuses_what_has_no_defined_answer_and_never_takes_a_lookup() {
        let ok = |_: AimRequest| Ok(AimReply::default());
        for (what, mutate) in [
            (
                "weapon",
                Box::new(|s: &mut sys::eech_aim_step| s.weapon = 0) as Box<dyn Fn(&mut sys::eech_aim_step)>,
            ),
            ("armour model", Box::new(|s: &mut sys::eech_aim_step| s.damage.armour_model = 3)),
            ("critical model", Box::new(|s: &mut sys::eech_aim_step| s.damage.critical_model = 4)),
            ("initial level", Box::new(|s: &mut sys::eech_aim_step| s.damage.initial_damage_level = 0)),
            ("flag", Box::new(|s: &mut sys::eech_aim_step| s.damage.invulnerable = 2)),
            ("position", Box::new(|s: &mut sys::eech_aim_step| s.damage.target_position[1] = f32::NAN)),
            ("attitude", Box::new(|s: &mut sys::eech_aim_step| s.damage.target_attitude[4] = f32::INFINITY)),
            ("speed", Box::new(|s: &mut sys::eech_aim_step| s.damage.weapon_speed = f32::NAN)),
            (
                "sample",
                Box::new(|s: &mut sys::eech_aim_step| {
                    s.has_draw = 1;
                    s.draw_sample = 40000;
                }),
            ),
        ] {
            let mut bad = damage_step();
            mutate(&mut bad);
            let (seen, _, code, error) = aim_crossing(&bad, ok);
            assert!(seen.is_none() && code != 0 && !error.is_empty(), "{what}: {error}");
        }
        // a person has no initial level to divide by and is not classified: that is a legal question
        let mut person = damage_step();
        person.damage.critical_model = 2;
        person.damage.initial_damage_level = 0;
        let (seen, _, code, error) = aim_crossing(&person, ok);
        assert!(seen.is_some() && code == 0, "{error}");
        // a damage question asks for no lookup
        let step = damage_step();
        let (_, _, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                need_lookup: true,
                lookups: vec![AimEvent {
                    kind: 1,
                    a: 0,
                    b: 0,
                    c: 0,
                    d: 0,
                    f: [0.0; 12],
                }],
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("cannot take them"), "{error}");
        let (_, _, code, error) = aim_crossing(&step, |_| Err("the host has no model for this hit".into()));
        assert!(code != 0 && error.contains("no model"), "{error}");
    }

    #[test]
    fn a_forecast_request_carries_its_figures_by_value_and_the_hosts_capability_comes_back() {
        let mut step = blank_aim_step(sys::EECH_AIM_FORECAST);
        step.weapon = 54;
        step.echo = 1;
        step.forecast_figures = sys::eech_damage_figures {
            weapon_warhead: 8,
            warhead: 5,
            damage_capability: 30,
            armor_penetration_capability: 7,
            burst_duration: 2.5,
        };
        step.forecast_armour_level = 9;
        step.forecast_target_not_ai = 1;
        step.forecast_player_modifier = 0.66;
        step.forecast_damage_modifier = 0.0;
        step.forecast_velocity = 1.0;
        let (seen, result, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                forecast_capability: 41,
                ..AimReply::default()
            })
        });
        assert_eq!(code, 0, "{error}");
        let Some(AimRequest::Forecast(request)) = seen else {
            panic!("the host was not asked to forecast")
        };
        assert_eq!(
            request,
            AimForecast {
                weapon: 54,
                weapon_warhead: 8,
                warhead: 5,
                damage_capability: 30,
                armor_penetration_capability: 7,
                burst_duration: 2.5,
                armour_level: 9,
                target_not_ai: true,
                player_modifier: 0.66,
                damage_modifier: 0.0,
                velocity: 1.0,
                echo: true,
            }
        );
        // the host's own number is what the campaign gets back, not one this layer computes
        assert_eq!((result.state, result.forecast_capability), (sys::EECH_AIM_DONE, 41));
    }

    #[test]
    fn a_forecast_refuses_what_has_no_defined_answer_and_never_takes_a_draw_or_a_lookup() {
        let mut step = blank_aim_step(sys::EECH_AIM_FORECAST);
        step.weapon = 54;
        step.forecast_player_modifier = 1.0;
        step.forecast_velocity = 1.0;
        let ok = |_: AimRequest| Ok(AimReply::default());
        // malformed requests never reach the host
        for (what, mutate) in [
            (
                "weapon",
                Box::new(|s: &mut sys::eech_aim_step| s.weapon = 0) as Box<dyn Fn(&mut sys::eech_aim_step)>,
            ),
            ("armour", Box::new(|s: &mut sys::eech_aim_step| s.forecast_armour_level = -1)),
            ("flag", Box::new(|s: &mut sys::eech_aim_step| s.forecast_target_not_ai = 2)),
            ("burst", Box::new(|s: &mut sys::eech_aim_step| s.forecast_figures.burst_duration = f32::NAN)),
            ("modifier", Box::new(|s: &mut sys::eech_aim_step| s.forecast_damage_modifier = f32::INFINITY)),
            ("player", Box::new(|s: &mut sys::eech_aim_step| s.forecast_player_modifier = f32::NAN)),
            ("velocity", Box::new(|s: &mut sys::eech_aim_step| s.forecast_velocity = f32::NAN)),
        ] {
            let mut bad = blank_aim_step(sys::EECH_AIM_FORECAST);
            bad.weapon = 54;
            bad.forecast_player_modifier = 1.0;
            bad.forecast_velocity = 1.0;
            mutate(&mut bad);
            let (seen, _, code, error) = aim_crossing(&bad, ok);
            assert!(seen.is_none() && code != 0 && !error.is_empty(), "{what}: {error}");
        }
        // a negative capability from the host is refused
        let (_, _, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                forecast_capability: -1,
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("negative capability"), "{error}");
        // a forecast consumes no sample of the shared stream and asks for no lookup
        let (_, _, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                need_draw: true,
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("cannot take one"), "{error}");
        let (_, _, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                need_lookup: true,
                lookups: vec![AimEvent {
                    kind: 1,
                    a: 0,
                    b: 0,
                    c: 0,
                    d: 0,
                    f: [0.0; 12],
                }],
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("cannot take them"), "{error}");
        // a host that fails says so
        let (_, _, code, error) = aim_crossing(&step, |_| Err("the host has no model for this weapon".into()));
        assert!(code != 0 && error.contains("no model"), "{error}");
    }

    #[test]
    fn a_release_request_carries_the_launchers_clock_and_the_hosts_gate_comes_back_even_when_it_refuses() {
        let mut step = blank_aim_step(sys::EECH_AIM_RELEASE);
        step.weapon = 5;
        step.delta_time = 0.1;
        step.system_time = 2000;
        step.release_has_timer = 1;
        step.release_last_shot = 1000;
        step.release_not_ai = 1;
        let (seen, result, code, _) = aim_crossing(&step, |_| {
            Ok(AimReply {
                release_refuse: true,
                release_burst_size: 0,
                release_last_shot: Some(2000),
                ..AimReply::default()
            })
        });
        assert_eq!(code, 0);
        let Some(AimRequest::Release(request)) = seen else {
            panic!("the host was not asked to release")
        };
        assert_eq!(
            (request.weapon, request.delta_time, request.current_time, request.last_shot, request.not_ai),
            (5, 0.1, 2000, Some(1000), true)
        );
        assert_eq!(
            (
                result.release_refuse,
                result.release_burst_size,
                result.release_write_last_shot,
                result.release_last_shot
            ),
            (1, 0, 1, 2000),
            "what the gate wrote comes back though it then refused"
        );
        // a launcher with no clock sends none, and a gate that wrote nothing returns nothing
        step.release_has_timer = 0;
        step.release_last_shot = 77;
        let (seen, result, code, _) = aim_crossing(&step, |_| {
            Ok(AimReply {
                release_burst_size: 3,
                ..AimReply::default()
            })
        });
        assert_eq!(code, 0);
        assert!(matches!(seen, Some(AimRequest::Release(ref request)) if request.last_shot.is_none()));
        assert_eq!((result.release_refuse, result.release_burst_size, result.release_write_last_shot), (0, 3, 0));
        // a clock written for a launcher that sent none is refused, whatever else the answer says
        let (_, _, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                release_burst_size: 1,
                release_last_shot: Some(5),
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("has none"), "{error}");
        step.weapon = 0;
        let (seen, _, code, error) = aim_crossing(&step, |_| panic!("the host must not be asked"));
        assert!(seen.is_none() && code != 0, "{error}");
    }

    #[test]
    fn an_aim_ready_update_carries_the_source_the_keyframe_and_the_time_and_returns_the_hosts_state_and_phase() {
        let mut step = blank_aim_step(sys::EECH_AIM_READY_UPDATE);
        for (slot, byte) in step.source_host_id.iter_mut().zip(b"launcher") {
            *slot = *byte as c_char;
        }
        step.ready_state = 1.25;
        step.ready_rate = 0.5;
        step.delta_time = 0.1;
        let (seen, result, code, _) = aim_crossing(&step, |_| {
            Ok(AimReply {
                ready_state: 1.3,
                phase: 1,
                ..AimReply::default()
            })
        });
        assert_eq!(code, 0);
        let Some(AimRequest::ReadyUpdate(request)) = seen else {
            panic!("not a ready update")
        };
        assert_eq!(request.source.as_deref(), Some("launcher"));
        assert_eq!((request.state, request.rate, request.delta_time), (1.25, 0.5, 0.1));
        assert_eq!((result.state, result.ready_state, result.phase), (sys::EECH_AIM_DONE, 1.3, 1));
    }

    #[test]
    fn an_aim_vectors_request_carries_a_weapon_and_a_target_or_a_tracking_point_never_both() {
        let mut step = blank_aim_step(sys::EECH_AIM_VECTORS);
        step.has_weapon = 1;
        step.weapon = 3;
        step.aiming_type = 3;
        step.cruise_velocity = 800.0;
        step.target_position = [1.0, 2.0, 3.0];
        step.target_point = [4.0, 5.0, 6.0];
        step.target_mobile = 1;
        step.map_max = [1.0e5; 3];
        let (seen, _, code, _) = aim_crossing(&step, |_| Ok(AimReply::default()));
        assert_eq!(code, 0);
        let Some(AimRequest::Vectors(vectors)) = seen else {
            panic!("not a vectors request")
        };
        assert!(vectors.tracking.is_none());
        assert_eq!(
            vectors.weapon.map(|weapon| (weapon.weapon, weapon.aiming_type, weapon.cruise_velocity)),
            Some((3, 3, 800.0))
        );
        assert_eq!(
            vectors.target.map(|target| (target.position, target.point, target.mobile)),
            Some(([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], true))
        );
        step.has_tracking = 1;
        step.tracking_point = [7.0, 8.0, 9.0];
        let (seen, _, _, _) = aim_crossing(&step, |_| Ok(AimReply::default()));
        let Some(AimRequest::Vectors(vectors)) = seen else {
            panic!("not a vectors request")
        };
        assert_eq!(vectors.tracking, Some([7.0, 8.0, 9.0]));
        assert!(vectors.weapon.is_none() && vectors.target.is_none());
    }

    #[test]
    fn an_aim_draw_is_asked_for_only_by_a_vectors_or_lead_request_and_carries_a_raw_sample_in_range() {
        // a lead request that asks, then is asked again with the sample
        let mut step = blank_aim_step(sys::EECH_AIM_LEAD);
        let (_, result, code, _) = aim_crossing(&step, |_| {
            Ok(AimReply {
                need_draw: true,
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.state), (0, sys::EECH_AIM_NEED_DRAW));
        step.has_draw = 1;
        step.draw_sample = 20000;
        let (seen, _, _, _) = aim_crossing(&step, |_| Ok(AimReply::default()));
        let Some(AimRequest::Lead(lead)) = seen else { panic!("not a lead request") };
        assert_eq!(lead.draw, Some(20000));
        // a sample outside [0, 32767] never reaches the host
        step.draw_sample = 32768;
        let (seen, _, code, error) = aim_crossing(&step, |_| panic!("the host must not be asked"));
        assert!(seen.is_none() && code != 0 && error.contains("draw sample"), "{error}");
        // a ballistic request cannot be answered with a request for a draw
        let step = blank_aim_step(sys::EECH_AIM_BALLISTIC);
        let (_, _, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                need_draw: true,
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("cannot take one"), "{error}");
    }

    #[test]
    fn an_aim_request_with_a_non_finite_figure_or_an_unknown_mode_never_reaches_the_host_and_a_failing_host_stops_it() {
        let mut step = blank_aim_step(sys::EECH_AIM_STEP);
        step.current_heading = f32::NAN;
        let (seen, _, code, error) = aim_crossing(&step, |_| panic!("the host must not be asked"));
        assert!(seen.is_none() && code != 0 && error.contains("non-finite"), "{error}");
        let step = blank_aim_step(99);
        let (seen, _, code, error) = aim_crossing(&step, |_| panic!("the host must not be asked"));
        assert!(seen.is_none() && code != 0 && error.contains("unknown mode"), "{error}");
        let step = blank_aim_step(sys::EECH_AIM_READY_COMMAND);
        let (seen, _, code, error) = aim_crossing(&step, |_| Err("the model refused".into()));
        assert!(seen.is_some() && code != 0 && error.contains("the model refused"), "{error}");
    }

    #[test]
    fn an_aim_echo_travels_in_order_and_one_that_does_not_fit_is_refused_not_cut_short() {
        let mut room = vec![
            sys::eech_aim_event {
                kind: 0,
                a: 0,
                b: 0,
                c: 0,
                d: 0,
                f: [0.0; 12],
            };
            sys::EECH_AIM_ECHO_MAX
        ];
        let mut step = blank_aim_step(sys::EECH_AIM_INTERCEPT);
        step.echo_out = room.as_mut_ptr();
        step.echo_capacity = room.len() as c_int;
        let events: Vec<AimEvent> = (0..3)
            .map(|index| AimEvent {
                kind: 3,
                a: index,
                b: 0,
                c: 0,
                d: 0,
                f: [index as f32; 12],
            })
            .collect();
        let sent = events.clone();
        let (_, result, code, _) = aim_crossing(&step, move |_| {
            Ok(AimReply {
                echo: Some(sent),
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.has_echo, result.echo_count), (0, 1, 3));
        assert!(room[..3]
            .iter()
            .enumerate()
            .all(|(index, event)| event.a == index as c_int && event.f[0] == index as f32));
        let too_many = vec![events[0]; sys::EECH_AIM_ECHO_MAX + 1];
        let (_, _, code, error) = aim_crossing(&step, move |_| {
            Ok(AimReply {
                echo: Some(too_many),
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("more than"), "{error}");
        // an echo nobody gave room for is refused too
        step.echo_out = std::ptr::null_mut();
        step.echo_capacity = 0;
        let (_, _, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                echo: Some(vec![events[0]]),
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("nobody asked for"), "{error}");
    }

    #[test]
    fn a_lead_flight_time_travels_only_when_the_host_produced_one_and_only_a_lead_or_ballistic_answer_may_carry_it() {
        let step = blank_aim_step(sys::EECH_AIM_LEAD);
        // none: the presence flag is clear and the value is zero
        let (_, result, code, _) = aim_crossing(&step, |_| Ok(AimReply::default()));
        assert_eq!((code, result.has_time_of_flight, result.time_of_flight), (0, 0, 0.0));
        // one: present and exact, whatever the outcome (a failed solve may have produced one before it failed)
        for ok in [true, false] {
            let (_, result, code, _) = aim_crossing(&step, move |_| {
                Ok(AimReply {
                    result: ok,
                    time_of_flight: Some(12.5),
                    ..AimReply::default()
                })
            });
            assert_eq!(
                (code, result.has_time_of_flight, result.time_of_flight, result.result),
                (0, 1, 12.5, c_int::from(ok))
            );
        }
        // a ballistic success carries it too
        let (_, result, code, _) = aim_crossing(&blank_aim_step(sys::EECH_AIM_BALLISTIC), |_| {
            Ok(AimReply {
                result: true,
                time_of_flight: Some(2.0),
                ..AimReply::default()
            })
        });
        assert_eq!((code, result.has_time_of_flight, result.time_of_flight), (0, 1, 2.0));
        // a non-finite one, and one on an answer that cannot carry it, are refused with the reason
        let (_, _, code, error) = aim_crossing(&step, |_| {
            Ok(AimReply {
                time_of_flight: Some(f32::NAN),
                ..AimReply::default()
            })
        });
        assert!(code != 0 && error.contains("non-finite flight time"), "{error}");
        for mode in [sys::EECH_AIM_READY_UPDATE, sys::EECH_AIM_STEP, sys::EECH_AIM_VECTORS, sys::EECH_AIM_INTERCEPT] {
            let (_, _, code, error) = aim_crossing(&blank_aim_step(mode), |_| {
                Ok(AimReply {
                    time_of_flight: Some(1.0),
                    ..AimReply::default()
                })
            });
            assert!(code != 0 && error.contains("cannot carry"), "mode {mode}: {error}");
        }
    }

    #[test]
    fn a_person_face_step_carries_the_target_position_and_a_non_finite_target_is_refused() {
        let mut step = place_step([0.0; 3]);
        step.mode = sys::EECH_MOVEMENT_PERSON_FACE;
        step.pf_target = [10.0, 20.0, 30.0];
        let request = received(&step);
        assert_eq!(request.pf_target, [10.0, 20.0, 30.0]);
        step.pf_target = [f32::INFINITY, 0.0, 0.0];
        let (outcome, _) = crossing(&step, |_| panic!("the host must not be asked"));
        assert!(outcome.is_err(), "a non-finite target position never reaches the host");
    }

    /// One crossing of the terrain callback, as the C side makes it: the request is built the way mb_hostterrain.c builds it, this thread's provider is
    /// `provider`, and the reply (which starts as a recognisable sentinel, so that a refusal that wrote something shows) and the error text come back.
    fn terrain_crossing(
        operation: c_int,
        winter: c_int,
        hint: &str,
        provider: impl FnMut(TerrainQuery) -> Result<TerrainAnswer, String> + 'static,
    ) -> (c_int, sys::eech_terrain_reply, String) {
        TERRAIN.with(|slot| *slot.borrow_mut() = Some(Box::new(provider)));
        // SAFETY: the request and reply are plain data; zero is a valid value for every field.
        let mut request: sys::eech_terrain_request = unsafe { std::mem::zeroed() };
        request.op = operation;
        request.winter = winter;
        request.x = 10.0;
        request.z = 20.0;
        for (slot, byte) in request.hint.iter_mut().zip(hint.as_bytes()) {
            *slot = *byte as c_char;
        }
        // SAFETY: as above.
        let mut reply: sys::eech_terrain_reply = unsafe { std::mem::zeroed() };
        reply.elevation = 123.0;
        reply.last_elevation = 456.0;
        reply.sector_x_max = 7;
        reply.stored = 9;
        reply.hint[0] = b'#' as c_char;
        let mut error = [0 as c_char; 256];
        // SAFETY: the request and reply are valid for the call, and the error buffer is writable for its length.
        let code = unsafe { terrain_callback(&request, &mut reply, error.as_mut_ptr(), error.len(), std::ptr::null_mut()) };
        TERRAIN.with(|slot| *slot.borrow_mut() = None);
        let text: Vec<u8> = error.iter().map(|byte| *byte as u8).take_while(|byte| *byte != 0).collect();
        (code, reply, String::from_utf8_lossy(&text).into_owned())
    }

    fn terrain_facts() -> TerrainMapFacts {
        TerrainMapFacts {
            sector_x_max: 4,
            sector_z_max: 5,
            fast_width: 0,
            fast_height: 0,
            min_x: 0.0,
            min_z: 0.0,
            max_x: 4000.0,
            max_z: 5000.0,
            min_height: -1.0,
            max_height: 100.0,
            scaled_height_difference: 0.5,
        }
    }

    fn terrain_point(hint: Option<&str>, stored: bool) -> TerrainAnswer {
        TerrainAnswer::Point {
            elevation: 1.5,
            last_elevation: 1.25,
            normal: [0.0, 1.0, 0.0],
            kind: 3,
            hint: hint.map(str::to_string),
            stored,
        }
    }

    #[test]
    fn a_terrain_answer_of_the_wrong_kind_is_refused_before_anything_is_written() {
        let answers: [(&str, fn() -> TerrainAnswer); 3] = [
            ("map", || TerrainAnswer::Map(terrain_facts())),
            ("point", || terrain_point(None, true)),
            ("fast", || TerrainAnswer::Fast { elevation: 2.0 }),
        ];
        let operations = [
            ("map", sys::EECH_TERRAIN_MAP),
            ("point", sys::EECH_TERRAIN_POINT),
            ("fast", sys::EECH_TERRAIN_FAST),
        ];
        let mut refused = 0;
        for (asked, operation) in operations {
            for (given, answer) in &answers {
                let answer = *answer;
                let (code, reply, error) = terrain_crossing(operation, 0, "", move |_| Ok(answer()));
                if asked == *given {
                    assert_eq!(code, 0, "{asked} answered with {given}: {error}");
                    continue;
                }
                refused += 1;
                assert_ne!(code, 0, "a {asked} question answered with a {given} answer must be refused");
                assert!(error.starts_with(&format!("[terrain-provider:{asked}.answer]")), "{asked}/{given}: {error}");
                // nothing of the answer reached the reply: it is still the sentinel the crossing started with
                assert_eq!(
                    (reply.elevation, reply.last_elevation, reply.sector_x_max, reply.stored, reply.hint[0]),
                    (123.0, 456.0, 7, 9, b'#' as c_char),
                    "{asked}/{given} wrote into the reply"
                );
            }
        }
        assert_eq!(refused, 6, "all six wrong pairings were exercised");
    }

    #[test]
    fn a_terrain_question_reaches_the_provider_with_its_season_and_a_matching_answer_reaches_the_reply() {
        for winter in [0, 1] {
            let (code, reply, error) = terrain_crossing(sys::EECH_TERRAIN_MAP, winter, "", move |query| {
                assert_eq!(query, TerrainQuery::Map { winter: winter == 1 });
                Ok(TerrainAnswer::Map(terrain_facts()))
            });
            assert_eq!(code, 0, "{error}");
            assert_eq!((reply.sector_x_max, reply.sector_z_max, reply.max_x), (4, 5, 4000.0));
            let (code, _, error) = terrain_crossing(sys::EECH_TERRAIN_FAST, winter, "", move |query| {
                assert_eq!(
                    query,
                    TerrainQuery::Fast {
                        x: 10.0,
                        z: 20.0,
                        winter: winter == 1
                    }
                );
                Ok(TerrainAnswer::Fast { elevation: 2.0 })
            });
            assert_eq!(code, 0, "{error}");
        }
        let (code, reply, error) = terrain_crossing(sys::EECH_TERRAIN_POINT, 1, "h", |query| {
            assert_eq!(
                query,
                TerrainQuery::Point {
                    x: 10.0,
                    z: 20.0,
                    hint: Some("h".into()),
                    winter: true
                }
            );
            Ok(terrain_point(None, true))
        });
        assert_eq!(code, 0, "{error}");
        // a surface was stored although the host returned no hint: the flag is the host's, by value
        assert_eq!(
            (reply.elevation, reply.last_elevation, reply.kind, reply.stored, reply.hint[0]),
            (1.5, 1.25, 3, 1, 0)
        );
        let (code, reply, error) = terrain_crossing(sys::EECH_TERRAIN_POINT, 0, "", |_| Ok(terrain_point(Some("opaque"), false)));
        assert_eq!(code, 0, "{error}");
        assert_eq!((reply.stored, reply.hint[0]), (0, b'o' as c_char));
    }

    #[test]
    fn a_terrain_season_flag_that_is_not_a_flag_and_a_provider_refusal_stop_the_crossing() {
        let (code, _, error) = terrain_crossing(sys::EECH_TERRAIN_MAP, 2, "", |_| panic!("the provider must not be asked"));
        assert_ne!(code, 0);
        assert!(error.contains("[terrain-provider:point.callback]") && error.contains("season flag"), "{error}");
        let (code, _, error) = terrain_crossing(sys::EECH_TERRAIN_FAST, 0, "", |_| Err("[terrain-provider:fast.elevation] nope".into()));
        assert_ne!(code, 0);
        assert_eq!(error, "[terrain-provider:fast.elevation] nope");
    }
}
