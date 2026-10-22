//! The host's physical record of its alive ground convoy vehicles, ships,
//! infantry and fixed-wing aircraft, keyed by the host identity it issued at `bind_mobile`. The campaign
//! asks for a step; the host owns the position, speed and attitude it keeps, and
//! the answer is what the campaign applies.
//!
//! The campaign still holds a copy of each pose (combat, perception, arrival and
//! Tacview read it). On a step the copy is sent only to be compared with the
//! record: the host never adopts it, so a write the host did not make is an
//! error, not a silent overwrite. The campaign says so explicitly (`Place`) when
//! it puts a mobile somewhere.
//!
//! An infantryman who follows the ground moves in two phases, because the
//! terrain stays campaign-side: `PersonBegin` advances him horizontally and
//! holds the step as pending, the campaign samples the terrain at the advanced
//! position, and `PersonEnd` settles the height, attitude and speed. The
//! pending step is the host's; the record does not change until it ends, and no
//! other request is accepted for that man meanwhile.

use std::collections::HashMap;
use std::os::raw::c_int;

extern "C" {
    fn eech_convoy_step(
        position: *mut f32,
        velocity: *mut f32,
        attitude: *mut f32,
        delta_seconds: f32,
        desired_velocity: f32,
        max_acceleration: f32,
        waypoint: *const f32,
        tacview_logging: c_int,
        terrain_present: c_int,
        elevation: f32,
        forest: c_int,
        face_normal: *const f32,
        map_min: *const f32,
        map_max: *const f32,
        rotated: *mut c_int,
    );

    fn eech_ship_step(
        position: *mut f32,
        velocity: *mut f32,
        attitude: *mut f32,
        delta_seconds: f32,
        frame_delta: f32,
        guide_velocity: f32,
        max_acceleration: f32,
        g_max: f32,
        waypoint: *const f32,
        map_min: *const f32,
        map_max: *const f32,
        motion_vector: *mut f32,
    );

    fn eech_person_advance(
        position: *const f32,
        ps_vel: f32,
        delta_seconds: f32,
        waypoint: *const f32,
        new_position: *mut f32,
        increment: *mut f32,
        heading: *mut f32,
    );

    fn eech_person_settle(
        position: *mut f32,
        velocity: *mut f32,
        attitude: *mut f32,
        new_position: *const f32,
        increment: *const f32,
        heading: f32,
        ps_vel: f32,
        delta_seconds: f32,
        terrain_follow: c_int,
        elevation: f32,
        motion_vector: *mut f32,
    );

    fn eech_fw_step(
        phase: c_int,
        position: *mut f32,
        velocity: *mut f32,
        attitude: *mut f32,
        delta_seconds: f32,
        frame_delta: f32,
        wp_vel: f32,
        cruise_velocity: f32,
        g_max: f32,
        fuselage_angle: f32,
        cg_to_ground: f32,
        max_roll: f32,
        turn_speed_multi: f32,
        waypoint: *const f32,
        acceleration: *const f32,
        motion_vector: *mut f32,
        navigating: c_int,
        elevation: f32,
        map_min: *const f32,
        map_max: *const f32,
        tacview_logging: c_int,
        rotated: *mut c_int,
    );

    fn eech_hc_begin(motion_vector: *mut f32, acceleration_x: f32, acceleration_z: f32, wp_vec_y: f32, delta_seconds: f32, speed: *mut f32, moved: *mut c_int);

    fn eech_hc_end(
        position: *mut f32,
        motion_vector: *mut f32,
        attitude: *mut f32,
        desired_heading: f32,
        max_turn_rate: f32,
        desired_pitch: f32,
        desired_roll: f32,
        delta_seconds: f32,
        elevation: f32,
        cg_to_ground: f32,
        terrain_follow: c_int,
        map_min: *const f32,
        map_max: *const f32,
        rotated: *mut c_int,
    );

    fn eech_hc_rotor_alive(rotor: *mut f32, lift: f32, power_output: f32, delta_seconds: f32, gate: c_int, gated: *mut c_int);
    fn eech_hc_rotor_down(rotor: *mut f32, delta_seconds: f32);
    fn eech_hc_rotor_idle(rotor: *mut f32, delta_seconds: f32);
    fn eech_hc_fall_position(
        position: *const f32,
        motion_vector: *const f32,
        delta_seconds: f32,
        map_min: *const f32,
        map_max: *const f32,
        new_position: *mut f32,
    );
    fn eech_hc_fall_gravity(motion_vector: *mut f32, gravity: f32, delta_seconds: f32, speed: *mut f32);
    fn eech_hc_tumble(attitude: *mut f32, heading_draw: f64, roll_draw: f64, delta_seconds: f32);
    fn eech_hc_contact(point: *const f32, elevation: f32, cg_to_ground: f32, altitude: *mut f32, contact: *mut c_int);
    fn eech_hc_settle(
        candidate: *mut f32,
        held_position: *const f32,
        motion_vector: *mut f32,
        rotor: *mut f32,
        terrain_class: c_int,
        elevation: f32,
        cg_to_ground: f32,
        contact_altitude: f32,
        bury_limit: f32,
        buried: *mut c_int,
    );
    fn eech_hc_crash_begin(
        attitude: *mut f32,
        position: *const f32,
        motion_vector: *const f32,
        face_normal: *const f32,
        destroyed_pitch_offset: f32,
        destroyed_bank_offset: f32,
        delta_seconds: f32,
        map_min: *const f32,
        map_max: *const f32,
        candidate: *mut f32,
    );
    fn eech_hc_crash_end(
        position: *const f32,
        candidate: *mut f32,
        motion_vector: *mut f32,
        forest: c_int,
        elevation: f32,
        cg_to_ground: f32,
        face_normal_x: f32,
        face_normal_z: f32,
        delta_seconds: f32,
        speed_out: *mut f32,
        stopped: *mut c_int,
    );
    fn eech_hc_impact(motion_vector: *mut f32, attitude: *const f32, signed_speed: f32);
}

/// Position in EECH world metres, speed in m/s, a row-major 3x3 attitude.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub position: [f32; 3],
    pub velocity: f32,
    pub attitude: [f32; 9],
}

impl Pose {
    /// Bit-for-bit equality (the campaign's copy must be exactly the host's record):
    /// the first component that differs, with both values.
    fn first_difference(&self, other: &Pose) -> Option<String> {
        let parts: [(&str, &[f32], &[f32]); 3] = [
            ("position", &self.position, &other.position),
            ("velocity", std::slice::from_ref(&self.velocity), std::slice::from_ref(&other.velocity)),
            ("attitude", &self.attitude, &other.attitude),
        ];
        parts.iter().find_map(|(what, ours, theirs)| {
            let index = ours.iter().zip(*theirs).position(|(a, b)| a.to_bits() != b.to_bits())?;
            Some(format!("{what}[{index}] ({} against {})", ours[index], theirs[index]))
        })
    }
}

/// What a bound mobile is; each kind accepts only its own movement requests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Routed,
    Ship,
    Person,
    FixedWing,
    Helicopter,
}

/// Terrain at the pre-move position, sampled by the campaign.
#[derive(Clone, Copy, Debug)]
pub struct Terrain {
    pub elevation: f32,
    pub forest: bool,
    pub face_normal: [f32; 3],
}

/// A routed vehicle's step towards its waypoint.
#[derive(Clone, Copy, Debug)]
pub struct Advance {
    pub delta_seconds: f32,
    pub desired_velocity: f32,
    pub max_acceleration: f32,
    pub waypoint: [f32; 3],
    pub tacview_logging: bool,
    pub terrain: Option<Terrain>,
    pub map_min: [f32; 3],
    pub map_max: [f32; 3],
}

/// A ship's step towards its waypoint. `delta_seconds` is the engine's step delta and
/// `frame_delta` the frame reciprocal the motion vector uses: they are different values.
#[derive(Clone, Copy, Debug)]
pub struct ShipMove {
    pub delta_seconds: f32,
    pub frame_delta: f32,
    pub guide_velocity: f32,
    pub max_acceleration: f32,
    pub g_max: f32,
    pub waypoint: [f32; 3],
    pub map_min: [f32; 3],
    pub map_max: [f32; 3],
}

/// The horizontal phase of an infantryman's step. `ps_vel` is the speed of the animation the
/// campaign chose; `terrain_follow` says the man walks on the ground, so the step stays pending
/// until the campaign has sampled the terrain at the advanced position.
#[derive(Clone, Copy, Debug)]
pub struct PersonBegin {
    pub delta_seconds: f32,
    pub ps_vel: f32,
    pub waypoint: [f32; 3],
    pub terrain_follow: bool,
}

/// The final phase of a pending infantry step: the terrain elevation at the advanced position.
#[derive(Clone, Copy, Debug)]
pub struct PersonEnd {
    pub elevation: f32,
}

/// Which physical step of an alive fixed-wing aircraft the campaign's guidance chose.
/// The numbers are the C step's `PHASE_*`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FixedWingPhase {
    Taxi = 0,
    Flight = 1,
    Land = 2,
    Absolute = 3,
}

/// A fixed-wing aircraft's step. The campaign supplies the guidance (the phase, the waypoint,
/// the guide's speed, the navigator's acceleration, the previous motion vector and the class
/// parameters); the host integrates velocity, position and attitude and answers the new motion
/// vector. `delta_seconds` is the engine's step delta and `frame_delta` the frame reciprocal:
/// they are different values.
#[derive(Clone, Copy, Debug)]
pub struct FixedWingMove {
    pub phase: FixedWingPhase,
    pub delta_seconds: f32,
    pub frame_delta: f32,
    pub wp_vel: f32,
    pub cruise_velocity: f32,
    pub g_max: f32,
    pub fuselage_angle: f32,
    pub cg_to_ground: f32,
    pub max_roll: f32,
    pub turn_speed_multi: f32,
    pub waypoint: [f32; 3],
    pub acceleration: [f32; 3],
    /// The DLL's mirror of the aircraft's previous motion vector. The host retains the vector
    /// itself (physical state, seeded at binding) and flies with its own; the mirror is only
    /// validated bit for bit against it, like the pose, so a write the host did not make is refused.
    pub motion_vector: [f32; 3],
    pub navigating: bool,
    pub elevation: f32,
    pub tacview_logging: bool,
    pub map_min: [f32; 3],
    pub map_max: [f32; 3],
}

/// Phase one of an alive AI helicopter's step: the motion vector, its speed and whether the helicopter moved.
/// The campaign supplies its decisions (the world acceleration's x and z and the climb demand); the host
/// retains the vector and answers it, and the campaign applies the answer at once, because its guidance reads it
/// before phase two. `motion_vector` is the DLL's mirror of the previous vector, validated bit for bit like the pose.
#[derive(Clone, Copy, Debug)]
pub struct HelicopterBegin {
    pub delta_seconds: f32,
    pub acceleration_x: f32,
    pub acceleration_z: f32,
    pub wp_vec_y: f32,
    pub motion_vector: [f32; 3],
}

/// Phase two of an alive AI helicopter's step, after the campaign's guidance read phase one: the guidance's desired
/// heading, pitch and roll, the aircraft's turn-rate limit and the terrain under the helicopter. The host wraps the heading
/// to the shortest turn, bounds it to `max_turn_rate`, turns, integrates the position, bounds it to the map and the floor.
/// The desired attitude is guidance input and the turn limit a physical input; neither is a campaign decision made here.
/// `motion_vector` is the mirror phase one left, validated bit for bit.
#[derive(Clone, Copy, Debug)]
pub struct HelicopterEnd {
    pub delta_seconds: f32,
    pub desired_heading: f32,
    pub max_turn_rate: f32,
    pub desired_pitch: f32,
    pub desired_roll: f32,
    pub elevation: f32,
    pub cg_to_ground: f32,
    pub terrain_follow: bool,
    pub motion_vector: [f32; 3],
    pub map_min: [f32; 3],
    pub map_max: [f32; 3],
}

/// The helicopter lifecycle modes: the rotor and the dying, crash and impact physics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleMode {
    /// the explicit seed/reset: the host adopts `rotor`
    RotorSet,
    /// an alive rotor's integration; `moved` answers whether the spin-up gate held
    RotorAlive,
    /// a dying rotor's wind-down
    RotorDown,
    /// an idle rotor's linear wind-down
    RotorIdle,
    /// the candidate position of the fall (the host stores it; the pose is untouched)
    Fall,
    /// gravity on the vector and the speed, after the campaign's terrain sample at the candidate
    FallGravity,
    /// the altitude of the held position, or of the stored candidate when `flag`; the candidate's is retained
    Contact,
    /// settle the stored candidate on the terrain class, with the retained contact altitude
    Settle,
    /// the tumble attitude; the final position is the stored candidate
    Tumble,
    /// the crash slide's first phase; the step is open until `CrashEnd`
    CrashBegin,
    /// the crash slide's second phase
    CrashEnd,
    /// the shove along the attitude x axis
    Impact,
}

impl LifecycleMode {
    fn name(self) -> &'static str {
        match self {
            LifecycleMode::RotorSet => "rotor_set",
            LifecycleMode::RotorAlive => "rotor_alive",
            LifecycleMode::RotorDown => "rotor_down",
            LifecycleMode::RotorIdle => "rotor_idle",
            LifecycleMode::Fall => "fall",
            LifecycleMode::FallGravity => "fall_gravity",
            LifecycleMode::Contact => "contact",
            LifecycleMode::Settle => "settle",
            LifecycleMode::Tumble => "tumble",
            LifecycleMode::CrashBegin => "crash_begin",
            LifecycleMode::CrashEnd => "crash_end",
            LifecycleMode::Impact => "impact",
        }
    }
}

/// One helicopter lifecycle step. Every mode carries the campaign's mirror of the host's rotor state and motion vector,
/// validated bit for bit before anything is mutated (`RotorSet` adopts the rotor, so only its vector is checked); the
/// remaining fields are the inputs of the one mode and ignored by the others.
#[derive(Clone, Copy, Debug)]
pub struct Lifecycle {
    pub mode: LifecycleMode,
    pub rotor: [f32; 3],
    pub motion_vector: [f32; 3],
    pub delta_seconds: f32,
    pub lift: f32,
    pub power_output: f32,
    /// `RotorAlive`: the spin-up gate; `Contact`: measure the stored candidate rather than the held position
    pub flag: bool,
    pub gravity: f32,
    /// `Settle`: 0 solid, 1 sea, 2 river or lake, 3 forest
    pub terrain_class: i32,
    pub bury_limit: f32,
    pub elevation: f32,
    pub cg_to_ground: f32,
    pub draws: [f64; 2],
    pub pitch_offset: f32,
    pub bank_offset: f32,
    pub signed_speed: f32,
    pub forest: bool,
    /// `CrashBegin` the old terrain normal, `CrashEnd` the new one
    pub face_normal: [f32; 3],
    pub map_min: [f32; 3],
    pub map_max: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
pub enum Request {
    /// a helicopter's rotor, dying, crash or impact step
    Lifecycle(Lifecycle),
    /// a routed vehicle steps towards the waypoint
    Advance(Advance),
    /// the campaign zeroes a routed vehicle's speed
    Stop,
    /// a ship steps towards the waypoint
    ShipMove(ShipMove),
    /// an infantryman's horizontal step
    PersonBegin(PersonBegin),
    /// an infantryman's final step, after the campaign sampled the terrain
    PersonEnd(PersonEnd),
    /// an alive fixed-wing aircraft's step
    FixedWingMove(FixedWingMove),
    /// an alive AI helicopter's step, phase one
    HelicopterBegin(HelicopterBegin),
    /// an alive AI helicopter's step, phase two
    HelicopterEnd(HelicopterEnd),
    /// the campaign placed the mobile: adopt its pose. `motion_vector` is the DLL's motion
    /// vector, adopted only by an aircraft record (declared transitional coupling: the DLL
    /// still writes an aircraft's vector outside a host step).
    Place { motion_vector: [f32; 3] },
}

impl Request {
    fn name(&self) -> &'static str {
        match self {
            Request::Advance(_) => "move",
            Request::Stop => "stop",
            Request::ShipMove(_) => "ship_move",
            Request::PersonBegin(_) => "person_begin",
            Request::PersonEnd(_) => "person_end",
            Request::FixedWingMove(_) => "fixed_wing_move",
            Request::HelicopterBegin(_) => "helicopter_begin",
            Request::HelicopterEnd(_) => "helicopter_end",
            Request::Lifecycle(step) => step.mode.name(),
            Request::Place { .. } => "place",
        }
    }

    /// The kind that may make this request; `None` for a placement, which any kind may receive.
    fn kind(&self) -> Option<Kind> {
        match self {
            Request::Advance(_) | Request::Stop => Some(Kind::Routed),
            Request::ShipMove(_) => Some(Kind::Ship),
            Request::PersonBegin(_) | Request::PersonEnd(_) => Some(Kind::Person),
            Request::FixedWingMove(_) => Some(Kind::FixedWing),
            Request::HelicopterBegin(_) | Request::HelicopterEnd(_) | Request::Lifecycle(_) => Some(Kind::Helicopter),
            Request::Place { .. } => None,
        }
    }
}

/// What the host answers. `motion_vector` is meaningful for ships and persons. While
/// `pending`, only `pose.position` is meaningful (the advanced horizontal position, to sample
/// the terrain at) and the campaign applies nothing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outcome {
    pub pose: Pose,
    pub rotated: bool,
    /// a helicopter's phase one: whether its speed crossed the movement threshold (false for every other request)
    pub moved: bool,
    pub motion_vector: [f32; 3],
    pub pending: bool,
    /// a helicopter's retained rotor (rpm, tail rpm, spin-up timer); zero for every other kind
    pub rotor: [f32; 3],
    /// a helicopter contact's altitude over the terrain
    pub altitude: f32,
}

/// An infantry step between its two phases.
#[derive(Clone, Copy, Debug)]
struct Pending {
    ps_vel: f32,
    delta_seconds: f32,
    new_position: [f32; 3],
    increment: [f32; 3],
    heading: f32,
}

struct Record {
    kind: Kind,
    pose: Pose,
    /// An aircraft's previous motion vector, retained by the host; `None` for other kinds.
    motion_vector: Option<[f32; 3]>,
    pending: Option<Pending>,
    /// A helicopter's phase one ran and phase two is owed. Nothing else may be requested until it is made.
    helicopter_pending: bool,
    /// A helicopter's rotor rpm, tail rotor rpm and spin-up timer, retained by the host; `None` for other kinds.
    rotor: Option<[f32; 3]>,
    /// A dying helicopter's candidate position (the fall, settled, or the crash slide's), kept until it is tumbled or the
    /// slide ends. Placements between the phases do not clear it.
    candidate: Option<[f32; 3]>,
    /// The altitude of the candidate at its contact, measured with the pre-kill centre of gravity; `Settle` consumes it.
    contact_altitude: Option<f32>,
    /// A crash slide's first phase ran and the second is owed. Nothing else may be requested until it is made.
    crash_pending: bool,
}

#[derive(Default)]
pub struct Movers {
    mobiles: HashMap<String, Record>,
}

impl Movers {
    /// Start the record of a mobile the host just issued `host_id` to. An aircraft (fixed-wing
    /// or helicopter) must be seeded with its motion vector, and no other kind may be: a missing
    /// seed is refused rather than defaulted. A helicopter must also be seeded with its rotor state, and no other kind may be.
    pub fn bind(&mut self, host_id: &str, kind: Kind, pose: Pose, motion_vector: Option<[f32; 3]>, rotor: Option<[f32; 3]>) -> Result<(), String> {
        match (kind, motion_vector) {
            (Kind::FixedWing | Kind::Helicopter, None) => {
                return Err(format!("host.bind_mobile: {kind:?} {host_id} was bound without a motion vector"));
            }
            (Kind::FixedWing | Kind::Helicopter, Some(_)) => {}
            (_, Some(_)) => return Err(format!("host.bind_mobile: {kind:?} {host_id} was bound with a motion vector")),
            (_, None) => {}
        }
        match (kind, rotor) {
            (Kind::Helicopter, None) => {
                return Err(format!("host.bind_mobile: helicopter {host_id} was bound without a rotor state"));
            }
            (Kind::Helicopter, Some(rotor)) if rotor.iter().any(|value| !value.is_finite()) => {
                return Err(format!("host.bind_mobile: helicopter {host_id} was bound with a non-finite rotor state"));
            }
            (Kind::Helicopter, Some(_)) | (_, None) => {}
            (_, Some(_)) => return Err(format!("host.bind_mobile: {kind:?} {host_id} was bound with a rotor state")),
        }
        self.mobiles.insert(
            host_id.to_owned(),
            Record {
                kind,
                pose,
                motion_vector,
                pending: None,
                helicopter_pending: false,
                rotor,
                candidate: None,
                contact_altitude: None,
                crash_pending: false,
            },
        );
        Ok(())
    }

    /// Run one request. `campaign` is the campaign's copy of the pose; the
    /// outcome is the host's record afterwards.
    pub fn step(&mut self, host_id: &str, campaign: Pose, request: Request) -> Result<Outcome, String> {
        let mut outcome = self.dispatch(host_id, campaign, request)?;
        if let Some(rotor) = self.mobiles.get(host_id).and_then(|record| record.rotor) {
            outcome.rotor = rotor;
        }
        Ok(outcome)
    }

    fn dispatch(&mut self, host_id: &str, campaign: Pose, request: Request) -> Result<Outcome, String> {
        let record = self
            .mobiles
            .get_mut(host_id)
            .ok_or_else(|| format!("host.move_vehicle: unknown host id {host_id}"))?;
        if request.kind().is_some_and(|kind| kind != record.kind) {
            return Err(format!(
                "host.move_vehicle: {} is not a request for {:?} {host_id}",
                request.name(),
                record.kind
            ));
        }
        match (&request, record.pending.is_some()) {
            (Request::PersonEnd(_), false) => {
                return Err(format!("host.move_vehicle: person_end for {host_id} without a pending person_begin"));
            }
            (Request::PersonEnd(_), true) => {}
            (_, true) => {
                return Err(format!("host.move_vehicle: {} for {host_id} while its person step is pending", request.name()));
            }
            (_, false) => {}
        }
        match (&request, record.helicopter_pending) {
            (Request::HelicopterEnd(_), false) => {
                return Err(format!("host.move_vehicle: helicopter_end for {host_id} without a pending helicopter_begin"));
            }
            (Request::HelicopterEnd(_), true) => {}
            (_, true) => {
                return Err(format!(
                    "host.move_vehicle: {} for {host_id} while its helicopter step is pending",
                    request.name()
                ));
            }
            (_, false) => {}
        }
        let crash_end = matches!(&request, Request::Lifecycle(step) if step.mode == LifecycleMode::CrashEnd);
        match (crash_end, record.crash_pending) {
            (true, false) => {
                return Err(format!("host.move_vehicle: crash_end for {host_id} without a pending crash_begin"));
            }
            (false, true) => {
                return Err(format!("host.move_vehicle: {} for {host_id} while its crash slide is pending", request.name()));
            }
            _ => {}
        }
        if let Request::Place { motion_vector } = request {
            record.pose = campaign;
            if matches!(record.kind, Kind::FixedWing | Kind::Helicopter) {
                record.motion_vector = Some(motion_vector);
            }
            return Ok(settled(record.pose, false, [0.0; 3]));
        }
        if let Some(what) = campaign.first_difference(&record.pose) {
            return Err(format!(
                "host.move_vehicle: the campaign's {what} for {host_id} differs from the host's record (a write the host did not make)"
            ));
        }
        match request {
            Request::Advance(advance) => {
                let rotated = advance_pose(&mut record.pose, &advance);
                Ok(settled(record.pose, rotated, [0.0; 3]))
            }
            Request::Stop => {
                record.pose.velocity = 0.0;
                Ok(settled(record.pose, false, [0.0; 3]))
            }
            Request::ShipMove(ship) => {
                let motion_vector = ship_pose(&mut record.pose, &ship);
                Ok(settled(record.pose, false, motion_vector))
            }
            Request::PersonBegin(begin) => Ok(begin_person(record, &begin)),
            Request::PersonEnd(end) => Ok(end_person(record, &end)),
            Request::FixedWingMove(fixed_wing) => {
                let held = record
                    .motion_vector
                    .ok_or_else(|| format!("host.move_vehicle: fixed-wing {host_id} has no retained motion vector"))?;
                if let Some(index) = held.iter().zip(&fixed_wing.motion_vector).position(|(a, b)| a.to_bits() != b.to_bits()) {
                    return Err(format!(
                        "host.move_vehicle: the campaign's motion_vector[{index}] ({} against {}) for {host_id} differs from the host's record (a write the host did not make)",
                        fixed_wing.motion_vector[index], held[index]
                    ));
                }
                let (motion_vector, rotated) = fixed_wing_pose(&mut record.pose, held, &fixed_wing);
                record.motion_vector = Some(motion_vector);
                Ok(settled(record.pose, rotated, motion_vector))
            }
            Request::HelicopterBegin(begin) => {
                let held = held_vector(record, host_id, &begin.motion_vector)?;
                let (motion_vector, speed, moved) = helicopter_begin(held, &begin);
                record.motion_vector = Some(motion_vector);
                record.pose.velocity = speed;
                record.helicopter_pending = true;
                // the step is open until its end: the DLL's begin requires this answer
                Ok(Outcome {
                    moved,
                    pending: true,
                    ..settled(record.pose, false, motion_vector)
                })
            }
            Request::HelicopterEnd(end) => {
                let held = held_vector(record, host_id, &end.motion_vector)?;
                let (motion_vector, rotated) = helicopter_end(&mut record.pose, held, &end);
                record.motion_vector = Some(motion_vector);
                record.helicopter_pending = false;
                Ok(settled(record.pose, rotated, motion_vector))
            }
            Request::Lifecycle(step) => lifecycle(record, host_id, &step),
            Request::Place { .. } => unreachable!("placed above"),
        }
    }
}

/// The helicopter's retained rotor, refused (before anything is mutated) unless the campaign's mirror equals it bit for bit.
fn held_rotor(record: &Record, host_id: &str, mirror: &[f32; 3]) -> Result<[f32; 3], String> {
    let held = record
        .rotor
        .ok_or_else(|| format!("host.move_vehicle: {:?} {host_id} has no retained rotor state", record.kind))?;
    if let Some(index) = held.iter().zip(mirror).position(|(a, b)| a.to_bits() != b.to_bits()) {
        return Err(format!(
            "host.move_vehicle: the campaign's rotor[{index}] ({} against {}) for {host_id} differs from the host's record (a write the host did not make)",
            mirror[index], held[index]
        ));
    }
    Ok(held)
}

/// One helicopter lifecycle step. Everything is validated (mirrors, phase) before the record changes.
fn lifecycle(record: &mut Record, host_id: &str, step: &Lifecycle) -> Result<Outcome, String> {
    let mode = step.mode;
    let mut vector = held_vector(record, host_id, &step.motion_vector)?;
    let mut rotor = if mode == LifecycleMode::RotorSet {
        step.rotor
    } else {
        held_rotor(record, host_id, &step.rotor)?
    };
    let candidate = match mode {
        LifecycleMode::FallGravity | LifecycleMode::Settle | LifecycleMode::Tumble | LifecycleMode::CrashEnd => Some(record.candidate.ok_or_else(|| {
            format!(
                "host.move_vehicle: {} for {host_id} without a stored candidate position (fall or crash_begin first)",
                mode.name()
            )
        })?),
        LifecycleMode::Contact if step.flag => Some(
            record
                .candidate
                .ok_or_else(|| format!("host.move_vehicle: contact of the candidate for {host_id} without a stored candidate position (fall first)"))?,
        ),
        _ => None,
    };
    let contact_altitude = if mode == LifecycleMode::Settle {
        Some(
            record
                .contact_altitude
                .ok_or_else(|| format!("host.move_vehicle: settle for {host_id} without the contact of its candidate (contact with the candidate first)"))?,
        )
    } else {
        None
    };
    let dt = step.delta_seconds;
    let mut outcome = settled(record.pose, false, vector);
    match mode {
        LifecycleMode::RotorSet => {}
        LifecycleMode::RotorAlive => {
            let mut gated: c_int = 0;
            // SAFETY: the rotor is 3 floats read then written; the others are scalars and one int.
            unsafe { eech_hc_rotor_alive(rotor.as_mut_ptr(), step.lift, step.power_output, dt, c_int::from(step.flag), &mut gated) };
            outcome.moved = gated != 0;
        }
        LifecycleMode::RotorDown => {
            // SAFETY: as above.
            unsafe { eech_hc_rotor_down(rotor.as_mut_ptr(), dt) };
        }
        LifecycleMode::RotorIdle => {
            // SAFETY: as above.
            unsafe { eech_hc_rotor_idle(rotor.as_mut_ptr(), dt) };
        }
        LifecycleMode::Fall => {
            let mut fallen = [0.0f32; 3];
            // SAFETY: every pointer is to a live array of 3 floats; the step keeps none of them.
            unsafe {
                eech_hc_fall_position(
                    record.pose.position.as_ptr(),
                    vector.as_ptr(),
                    dt,
                    step.map_min.as_ptr(),
                    step.map_max.as_ptr(),
                    fallen.as_mut_ptr(),
                );
            }
            record.candidate = Some(fallen);
            record.contact_altitude = None;
            // the candidate only: the pose waits for the tumble
            outcome.pose.position = fallen;
        }
        LifecycleMode::FallGravity => {
            let mut speed = 0.0f32;
            // SAFETY: as above.
            unsafe { eech_hc_fall_gravity(vector.as_mut_ptr(), step.gravity, dt, &mut speed) };
            record.pose.velocity = speed;
            outcome.pose = record.pose;
        }
        LifecycleMode::Contact => {
            let point = candidate.unwrap_or(record.pose.position);
            let mut altitude = 0.0f32;
            let mut contact: c_int = 0;
            // SAFETY: as above.
            unsafe { eech_hc_contact(point.as_ptr(), step.elevation, step.cg_to_ground, &mut altitude, &mut contact) };
            record.contact_altitude = step.flag.then_some(altitude);
            outcome.altitude = altitude;
            outcome.moved = contact != 0;
        }
        LifecycleMode::Settle => {
            let mut settled_candidate = candidate.expect("checked above");
            let mut buried: c_int = 0;
            // SAFETY: as above; the held position is the pose after the campaign's splash placement.
            unsafe {
                eech_hc_settle(
                    settled_candidate.as_mut_ptr(),
                    record.pose.position.as_ptr(),
                    vector.as_mut_ptr(),
                    rotor.as_mut_ptr(),
                    step.terrain_class,
                    step.elevation,
                    step.cg_to_ground,
                    contact_altitude.expect("checked above"),
                    step.bury_limit,
                    &mut buried,
                );
            }
            record.candidate = Some(settled_candidate);
            record.contact_altitude = None;
            outcome.pose.position = settled_candidate;
            outcome.moved = buried != 0;
        }
        LifecycleMode::Tumble => {
            let mut attitude = record.pose.attitude;
            // SAFETY: the attitude is 9 floats read then written.
            unsafe { eech_hc_tumble(attitude.as_mut_ptr(), step.draws[0], step.draws[1], dt) };
            record.pose.attitude = attitude;
            record.pose.position = candidate.expect("checked above");
            record.candidate = None;
            outcome.pose = record.pose;
        }
        LifecycleMode::CrashBegin => {
            let mut attitude = record.pose.attitude;
            let mut slid = [0.0f32; 3];
            // SAFETY: as above; the normal, bounds and position are 3 floats.
            unsafe {
                eech_hc_crash_begin(
                    attitude.as_mut_ptr(),
                    record.pose.position.as_ptr(),
                    vector.as_ptr(),
                    step.face_normal.as_ptr(),
                    step.pitch_offset,
                    step.bank_offset,
                    dt,
                    step.map_min.as_ptr(),
                    step.map_max.as_ptr(),
                    slid.as_mut_ptr(),
                );
            }
            record.pose.attitude = attitude;
            record.candidate = Some(slid);
            record.crash_pending = true;
            outcome.pose = record.pose;
            outcome.pose.position = slid;
            outcome.pending = true;
        }
        LifecycleMode::CrashEnd => {
            let mut slid = candidate.expect("checked above");
            let mut speed = 0.0f32;
            let mut stopped: c_int = 0;
            // SAFETY: as above.
            unsafe {
                eech_hc_crash_end(
                    record.pose.position.as_ptr(),
                    slid.as_mut_ptr(),
                    vector.as_mut_ptr(),
                    c_int::from(step.forest),
                    step.elevation,
                    step.cg_to_ground,
                    step.face_normal[0],
                    step.face_normal[2],
                    dt,
                    &mut speed,
                    &mut stopped,
                );
            }
            record.pose.position = slid;
            if stopped == 0 {
                record.pose.velocity = speed;
            }
            record.candidate = None;
            record.crash_pending = false;
            outcome.pose = record.pose;
            outcome.moved = stopped != 0;
        }
        LifecycleMode::Impact => {
            // SAFETY: as above; the attitude is 9 floats.
            unsafe { eech_hc_impact(vector.as_mut_ptr(), record.pose.attitude.as_ptr(), step.signed_speed) };
        }
    }
    record.rotor = Some(rotor);
    record.motion_vector = Some(vector);
    outcome.motion_vector = vector;
    Ok(outcome)
}

/// The aircraft's retained motion vector, refused (before anything is mutated) unless the campaign's mirror equals it bit for bit.
fn held_vector(record: &Record, host_id: &str, mirror: &[f32; 3]) -> Result<[f32; 3], String> {
    let held = record
        .motion_vector
        .ok_or_else(|| format!("host.move_vehicle: {:?} {host_id} has no retained motion vector", record.kind))?;
    if let Some(index) = held.iter().zip(mirror).position(|(a, b)| a.to_bits() != b.to_bits()) {
        return Err(format!(
            "host.move_vehicle: the campaign's motion_vector[{index}] ({} against {}) for {host_id} differs from the host's record (a write the host did not make)",
            mirror[index], held[index]
        ));
    }
    Ok(held)
}

fn settled(pose: Pose, rotated: bool, motion_vector: [f32; 3]) -> Outcome {
    Outcome {
        pose,
        rotated,
        moved: false,
        motion_vector,
        pending: false,
        rotor: [0.0; 3],
        altitude: 0.0,
    }
}

fn advance_pose(record: &mut Pose, advance: &Advance) -> bool {
    let terrain = advance.terrain.unwrap_or(Terrain {
        elevation: 0.0,
        forest: false,
        face_normal: [0.0; 3],
    });
    let mut rotated: c_int = 0;
    // SAFETY: every pointer is to a live array of the length the C step reads (3, 1, 9),
    // and the step keeps none of them.
    unsafe {
        eech_convoy_step(
            record.position.as_mut_ptr(),
            &mut record.velocity,
            record.attitude.as_mut_ptr(),
            advance.delta_seconds,
            advance.desired_velocity,
            advance.max_acceleration,
            advance.waypoint.as_ptr(),
            c_int::from(advance.tacview_logging),
            c_int::from(advance.terrain.is_some()),
            terrain.elevation,
            c_int::from(terrain.forest),
            terrain.face_normal.as_ptr(),
            advance.map_min.as_ptr(),
            advance.map_max.as_ptr(),
            &mut rotated,
        );
    }
    rotated != 0
}

/// The ship's motion vector.
fn ship_pose(record: &mut Pose, ship: &ShipMove) -> [f32; 3] {
    let mut motion_vector = [0.0f32; 3];
    // SAFETY: as for `advance_pose`; the motion vector is 3 floats.
    unsafe {
        eech_ship_step(
            record.position.as_mut_ptr(),
            &mut record.velocity,
            record.attitude.as_mut_ptr(),
            ship.delta_seconds,
            ship.frame_delta,
            ship.guide_velocity,
            ship.max_acceleration,
            ship.g_max,
            ship.waypoint.as_ptr(),
            ship.map_min.as_ptr(),
            ship.map_max.as_ptr(),
            motion_vector.as_mut_ptr(),
        );
    }
    motion_vector
}

/// The aircraft's new motion vector and whether the step turned it enough to report.
fn fixed_wing_pose(record: &mut Pose, previous: [f32; 3], step: &FixedWingMove) -> ([f32; 3], bool) {
    let mut motion_vector = previous;
    let mut rotated: c_int = 0;
    // SAFETY: as for `advance_pose`; the acceleration, waypoint and bounds are 3 floats, the
    // motion vector is 3 floats read then written.
    unsafe {
        eech_fw_step(
            step.phase as c_int,
            record.position.as_mut_ptr(),
            &mut record.velocity,
            record.attitude.as_mut_ptr(),
            step.delta_seconds,
            step.frame_delta,
            step.wp_vel,
            step.cruise_velocity,
            step.g_max,
            step.fuselage_angle,
            step.cg_to_ground,
            step.max_roll,
            step.turn_speed_multi,
            step.waypoint.as_ptr(),
            step.acceleration.as_ptr(),
            motion_vector.as_mut_ptr(),
            c_int::from(step.navigating),
            step.elevation,
            step.map_min.as_ptr(),
            step.map_max.as_ptr(),
            c_int::from(step.tacview_logging),
            &mut rotated,
        );
    }
    (motion_vector, rotated != 0)
}

/// A helicopter's phase one: the new motion vector, its speed and whether it moved.
fn helicopter_begin(previous: [f32; 3], begin: &HelicopterBegin) -> ([f32; 3], f32, bool) {
    let mut motion_vector = previous;
    let mut speed = 0.0f32;
    let mut moved: c_int = 0;
    // SAFETY: as for `advance_pose`; the motion vector is 3 floats read then written.
    unsafe {
        eech_hc_begin(
            motion_vector.as_mut_ptr(),
            begin.acceleration_x,
            begin.acceleration_z,
            begin.wp_vec_y,
            begin.delta_seconds,
            &mut speed,
            &mut moved,
        );
    }
    (motion_vector, speed, moved != 0)
}

/// A helicopter's phase two: the new motion vector (its y may be clamped after the speed was taken) and whether it turned enough to report.
fn helicopter_end(record: &mut Pose, previous: [f32; 3], end: &HelicopterEnd) -> ([f32; 3], bool) {
    let mut motion_vector = previous;
    let mut rotated: c_int = 0;
    // SAFETY: as for `advance_pose`; the bounds are 3 floats.
    unsafe {
        eech_hc_end(
            record.position.as_mut_ptr(),
            motion_vector.as_mut_ptr(),
            record.attitude.as_mut_ptr(),
            end.desired_heading,
            end.max_turn_rate,
            end.desired_pitch,
            end.desired_roll,
            end.delta_seconds,
            end.elevation,
            end.cg_to_ground,
            c_int::from(end.terrain_follow),
            end.map_min.as_ptr(),
            end.map_max.as_ptr(),
            &mut rotated,
        );
    }
    (motion_vector, rotated != 0)
}

fn begin_person(record: &mut Record, begin: &PersonBegin) -> Outcome {
    let mut pending = Pending {
        ps_vel: begin.ps_vel,
        delta_seconds: begin.delta_seconds,
        new_position: [0.0; 3],
        increment: [0.0; 3],
        heading: 0.0,
    };
    // SAFETY: every pointer is to a live array of the length the C step reads (3), and the step keeps none of them.
    unsafe {
        eech_person_advance(
            record.pose.position.as_ptr(),
            pending.ps_vel,
            pending.delta_seconds,
            begin.waypoint.as_ptr(),
            pending.new_position.as_mut_ptr(),
            pending.increment.as_mut_ptr(),
            &mut pending.heading,
        );
    }
    if begin.terrain_follow {
        let mut partial = record.pose;
        partial.position = pending.new_position;
        record.pending = Some(pending);
        return Outcome {
            pose: partial,
            rotated: false,
            moved: false,
            motion_vector: [0.0; 3],
            pending: true,
            rotor: [0.0; 3],
            altitude: 0.0,
        };
    }
    settle_person(&mut record.pose, &pending, false, 0.0)
}

fn end_person(record: &mut Record, end: &PersonEnd) -> Outcome {
    let pending = record.pending.take().expect("checked by the caller");
    settle_person(&mut record.pose, &pending, true, end.elevation)
}

fn settle_person(pose: &mut Pose, pending: &Pending, terrain_follow: bool, elevation: f32) -> Outcome {
    let mut motion_vector = [0.0f32; 3];
    // SAFETY: as for `advance_pose`; the motion vector is 3 floats.
    unsafe {
        eech_person_settle(
            pose.position.as_mut_ptr(),
            &mut pose.velocity,
            pose.attitude.as_mut_ptr(),
            pending.new_position.as_ptr(),
            pending.increment.as_ptr(),
            pending.heading,
            pending.ps_vel,
            pending.delta_seconds,
            c_int::from(terrain_follow),
            elevation,
            motion_vector.as_mut_ptr(),
        );
    }
    settled(*pose, false, motion_vector)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEVEL: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

    fn pose(x: f32, z: f32) -> Pose {
        Pose {
            position: [x, 0.0, z],
            velocity: 0.0,
            attitude: LEVEL,
        }
    }

    fn advance(waypoint: [f32; 3]) -> Advance {
        Advance {
            delta_seconds: 1.0,
            desired_velocity: 10.0,
            max_acceleration: 4.0,
            waypoint,
            tacview_logging: true,
            terrain: None,
            map_min: [-1.0e6; 3],
            map_max: [1.0e6; 3],
        }
    }

    fn ship_move(waypoint: [f32; 3]) -> ShipMove {
        ShipMove {
            delta_seconds: 0.5,
            frame_delta: 30.0,
            guide_velocity: 10.0,
            max_acceleration: 2.0,
            g_max: 2.0,
            waypoint,
            map_min: [-1.0e6; 3],
            map_max: [1.0e6; 3],
        }
    }

    fn person_begin(waypoint: [f32; 3], ps_vel: f32, terrain_follow: bool) -> PersonBegin {
        PersonBegin {
            delta_seconds: 2.0,
            ps_vel,
            waypoint,
            terrain_follow,
        }
    }

    fn fixed_wing_move(phase: FixedWingPhase) -> FixedWingMove {
        FixedWingMove {
            phase,
            delta_seconds: 0.5,
            frame_delta: 2.0,
            wp_vel: 100.0,
            cruise_velocity: 100.0,
            g_max: 50.0,
            fuselage_angle: 0.0,
            cg_to_ground: 2.0,
            max_roll: 1.0,
            turn_speed_multi: 1.0,
            waypoint: [0.0, 0.0, 5000.0],
            acceleration: [0.0; 3],
            motion_vector: [0.0, 0.0, 100.0],
            navigating: false,
            elevation: 10.0,
            tacview_logging: true,
            map_min: [-1.0e6; 3],
            map_max: [1.0e6; 3],
        }
    }

    fn helicopter_begin_request() -> HelicopterBegin {
        HelicopterBegin {
            delta_seconds: 1.0,
            acceleration_x: 0.0,
            acceleration_z: 0.0,
            wp_vec_y: 0.0,
            motion_vector: [0.0, 0.0, 100.0],
        }
    }

    fn helicopter_end_request() -> HelicopterEnd {
        HelicopterEnd {
            delta_seconds: 1.0,
            desired_heading: 0.0,
            // far above any step in these tests, so only the tests of the bound see it
            max_turn_rate: 10.0,
            desired_pitch: 0.0,
            desired_roll: 0.0,
            elevation: -100.0,
            cg_to_ground: 0.0,
            terrain_follow: false,
            motion_vector: [0.0, 0.0, 100.0],
            map_min: [-1.0e6; 3],
            map_max: [1.0e6; 3],
        }
    }

    /// a helicopter at the origin, stepped through phase one; the mirror for phase two is what the host answered
    fn begun(begin: HelicopterBegin) -> (Movers, Outcome) {
        let mut movers = movers(Kind::Helicopter, pose(0.0, 0.0));
        let first = movers.step("v", pose(0.0, 0.0), Request::HelicopterBegin(begin)).unwrap();
        (movers, first)
    }

    fn assert_near(actual: [f32; 3], expected: [f32; 3]) {
        for (actual, expected) in actual.iter().zip(expected) {
            assert!((actual - expected).abs() < 1.0e-5, "{actual:?} vs {expected:?}");
        }
    }

    fn movers(kind: Kind, start: Pose) -> Movers {
        let mut movers = Movers::default();
        // an aircraft is seeded with the vector `fixed_wing_move` mirrors
        let seed = matches!(kind, Kind::FixedWing | Kind::Helicopter).then_some([0.0, 0.0, 100.0]);
        movers.bind("v", kind, start, seed, (kind == Kind::Helicopter).then_some(ROTOR)).unwrap();
        movers
    }

    /// the rotor a test helicopter is seeded with: at speed, the spin-up timer idle
    const ROTOR: [f32; 3] = [100.0, 100.0, 0.0];

    const PLACED: Request = Request::Place {
        motion_vector: [0.0, 0.0, 100.0],
    };

    #[test]
    fn accelerates_towards_the_waypoint_and_keeps_its_own_record() {
        let mut movers = movers(Kind::Routed, pose(0.0, 0.0));
        let first = movers.step("v", pose(0.0, 0.0), Request::Advance(advance([0.0, 0.0, 100.0]))).unwrap();
        // 4 m/s^2 for 1 s, then one second at that speed, due +z
        assert_eq!(first.pose.velocity, 4.0);
        assert_near(first.pose.position, [0.0, 0.0, 4.0]);
        assert!(!first.rotated, "already facing +z");
        // the next step compares the campaign's copy with the record, so it must be what was returned
        let second = movers.step("v", first.pose, Request::Advance(advance([0.0, 0.0, 100.0]))).unwrap();
        assert_eq!(second.pose.velocity, 8.0);
        assert_near(second.pose.position, [0.0, 0.0, 12.0]);
    }

    #[test]
    fn turning_towards_a_waypoint_reports_the_rotation() {
        let mut movers = movers(Kind::Routed, pose(0.0, 0.0));
        let turned = movers.step("v", pose(0.0, 0.0), Request::Advance(advance([100.0, 0.0, 0.0]))).unwrap();
        assert!(turned.rotated);
        // facing +x: heading row [sin, 0, cos]
        assert!((turned.pose.attitude[6] - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn a_vehicle_on_its_waypoint_is_not_moved() {
        let mut movers = movers(Kind::Routed, pose(5.0, 5.0));
        let same = movers.step("v", pose(5.0, 5.0), Request::Advance(advance([5.0, 0.0, 5.5]))).unwrap();
        assert_eq!(same.pose, pose(5.0, 5.0));
        assert!(!same.rotated);
    }

    #[test]
    fn the_map_volume_bounds_the_position() {
        let mut movers = movers(Kind::Routed, pose(0.0, 0.0));
        let mut towards_the_edge = advance([0.0, 0.0, 100.0]);
        towards_the_edge.map_max = [1.0e6, 1.0e6, 2.0];
        let bounded = movers.step("v", pose(0.0, 0.0), Request::Advance(towards_the_edge)).unwrap();
        assert_eq!(bounded.pose.position[2], 2.0);
    }

    #[test]
    fn a_campaign_copy_the_host_did_not_make_is_refused() {
        let mut movers = movers(Kind::Routed, pose(0.0, 0.0));
        let error = movers.step("v", pose(1.0, 0.0), Request::Advance(advance([0.0, 0.0, 100.0]))).unwrap_err();
        assert!(error.contains("position[0]") && error.contains("differs from the host's record"), "{error}");
        let error = movers.step("v", pose(1.0, 0.0), Request::Stop).unwrap_err();
        assert!(error.contains("position"), "{error}");
    }

    #[test]
    fn placing_adopts_the_campaign_pose_and_stopping_zeroes_the_speed() {
        let mut movers = movers(Kind::Routed, pose(0.0, 0.0));
        let mut moving = pose(50.0, 60.0);
        moving.velocity = 7.0;
        let adopted = movers.step("v", moving, PLACED).unwrap();
        assert_eq!(adopted.pose, moving);
        let stopped = movers.step("v", moving, Request::Stop).unwrap();
        assert_eq!(stopped.pose.velocity, 0.0);
        assert_eq!(stopped.pose.position, moving.position);
    }

    #[test]
    fn an_unknown_host_id_is_an_error() {
        let mut movers = Movers::default();
        let requests = [
            Request::Stop,
            PLACED,
            Request::Advance(advance([0.0; 3])),
            Request::ShipMove(ship_move([0.0; 3])),
            Request::PersonBegin(person_begin([0.0; 3], 1.0, false)),
            Request::PersonEnd(PersonEnd { elevation: 0.0 }),
            Request::FixedWingMove(fixed_wing_move(FixedWingPhase::Flight)),
        ];
        for request in requests {
            let error = movers.step("eech-world:9", pose(0.0, 0.0), request).unwrap_err();
            assert!(error.contains("unknown host id eech-world:9"), "{error}");
        }
    }

    #[test]
    fn a_ship_accelerates_ahead_and_uses_the_frame_delta_only_for_the_motion_vector() {
        let mut movers = movers(Kind::Ship, pose(0.0, 0.0));
        let first = movers.step("v", pose(0.0, 0.0), Request::ShipMove(ship_move([0.0, 0.0, 1000.0]))).unwrap();
        // min(10 - 0, 2 m/s^2) over the 0.5 s step delta; the position follows at that speed
        assert_eq!(first.pose.velocity, 1.0);
        assert_near(first.pose.position, [0.0, 0.0, 0.5]);
        // (new - old) * the 30 per second frame reciprocal
        assert_near(first.motion_vector, [0.0, 0.0, 15.0]);
        assert!(!first.pending);
        let second = movers.step("v", first.pose, Request::ShipMove(ship_move([0.0, 0.0, 1000.0]))).unwrap();
        assert_eq!(second.pose.velocity, 2.0);
        assert_near(second.pose.position, [0.0, 0.0, 1.5]);
    }

    #[test]
    fn a_ship_turns_at_its_g_limited_rate_towards_a_waypoint_abeam() {
        let mut start = pose(0.0, 0.0);
        start.velocity = 10.0;
        let mut movers = movers(Kind::Ship, start);
        let turned = movers.step("v", start, Request::ShipMove(ship_move([100.0, 0.0, 0.0]))).unwrap();
        // turn rate g_max / velocity = 0.2 rad/s over 0.5 s; the wanted turn (90 degrees) is far more
        let heading = 0.2f32 * 0.5;
        assert!((turned.pose.attitude[6] - heading.sin()).abs() < 1.0e-6);
        assert!((turned.pose.attitude[8] - heading.cos()).abs() < 1.0e-6);
        // sqrt(0.5 * 100 * 2) = 10 equals the speed, so no change of speed; the position follows the new heading
        assert_eq!(turned.pose.velocity, 10.0);
        assert_near(turned.pose.position, [10.0 * heading.sin() * 0.5, 0.0, 10.0 * heading.cos() * 0.5]);
        assert_eq!(turned.pose.position[1], 0.0);
    }

    #[test]
    fn a_ship_nearly_on_its_waypoint_still_turns_by_the_x_offset() {
        // the legacy near-waypoint branch sets z twice and leaves x as it was
        let mut start = pose(0.0, 0.0);
        start.velocity = 10.0;
        let mut movers = movers(Kind::Ship, start);
        let nearly = movers.step("v", start, Request::ShipMove(ship_move([0.05, 0.0, 0.0]))).unwrap();
        let required = (0.05f32).atan2(1.0);
        let heading = required.min(0.2) * 0.5;
        assert!((nearly.pose.attitude[6] - heading.sin()).abs() < 1.0e-6, "{:?}", nearly.pose.attitude);
    }

    #[test]
    fn a_ship_is_bounded_by_the_map_volume() {
        let mut movers = movers(Kind::Ship, pose(0.0, 0.0));
        let mut towards_the_edge = ship_move([0.0, 0.0, 1000.0]);
        towards_the_edge.map_max = [1.0e6, 1.0e6, 0.25];
        let bounded = movers.step("v", pose(0.0, 0.0), Request::ShipMove(towards_the_edge)).unwrap();
        assert_eq!(bounded.pose.position[2], 0.25);
        assert_near(bounded.motion_vector, [0.0, 0.0, 0.25 * 30.0]);
    }

    #[test]
    fn a_person_without_terrain_follow_moves_in_one_step() {
        let mut movers = movers(Kind::Person, pose(0.0, 0.0));
        let walked = movers
            .step("v", pose(0.0, 0.0), Request::PersonBegin(person_begin([0.0, 0.0, 100.0], 1.5, false)))
            .unwrap();
        assert!(!walked.pending);
        // 1.5 m/s for 2 s, due +z
        assert_near(walked.pose.position, [0.0, 0.0, 3.0]);
        assert_eq!(walked.pose.velocity, 1.5);
        assert_near(walked.motion_vector, [0.0, 0.0, 1.5]);
        // the next request compares against the settled record
        movers
            .step("v", walked.pose, Request::PersonBegin(person_begin([0.0, 0.0, 100.0], 1.5, false)))
            .unwrap();
    }

    #[test]
    fn a_person_never_overshoots_the_waypoint_and_a_stopped_man_only_turns() {
        let mut movers = movers(Kind::Person, pose(0.0, 0.0));
        let arrived = movers
            .step("v", pose(0.0, 0.0), Request::PersonBegin(person_begin([0.0, 0.0, 1.0], 1.5, false)))
            .unwrap();
        assert_near(arrived.pose.position, [0.0, 0.0, 1.0]);
        let stopped = movers
            .step("v", arrived.pose, Request::PersonBegin(person_begin([100.0, 0.0, 1.0], 0.0, false)))
            .unwrap();
        assert_eq!(stopped.pose.position, arrived.pose.position);
        assert_eq!(stopped.pose.velocity, 0.0);
        // facing +x
        assert!((stopped.pose.attitude[6] - 1.0).abs() < 1.0e-5);
    }

    #[test]
    fn a_ground_following_person_stays_pending_until_the_terrain_is_given() {
        let mut movers = movers(Kind::Person, pose(0.0, 0.0));
        let begun = movers
            .step("v", pose(0.0, 0.0), Request::PersonBegin(person_begin([0.0, 0.0, 100.0], 1.5, true)))
            .unwrap();
        assert!(begun.pending);
        // the horizontal position to sample the terrain at; nothing else is applied
        assert_near(begun.pose.position, [0.0, 0.0, 3.0]);
        assert_eq!(begun.pose.velocity, 0.0);
        assert_eq!(begun.pose.attitude, LEVEL);
        let ended = movers.step("v", pose(0.0, 0.0), Request::PersonEnd(PersonEnd { elevation: 12.0 })).unwrap();
        assert!(!ended.pending);
        assert_near(ended.pose.position, [0.0, 12.01, 3.0]);
        assert_eq!(ended.pose.velocity, 1.5);
        assert_near(ended.motion_vector, [0.0, 0.0, 1.5]);
        // settled: the next step compares against the settled pose
        movers
            .step("v", ended.pose, Request::PersonBegin(person_begin([0.0, 0.0, 100.0], 1.5, false)))
            .unwrap();
    }

    #[test]
    fn a_pending_person_step_refuses_every_other_request_and_survives_the_refusals() {
        let mut movers = movers(Kind::Person, pose(0.0, 0.0));
        movers
            .step("v", pose(0.0, 0.0), Request::PersonBegin(person_begin([0.0, 0.0, 100.0], 1.5, true)))
            .unwrap();
        for request in [PLACED, Request::PersonBegin(person_begin([0.0, 0.0, 100.0], 1.5, true))] {
            let error = movers.step("v", pose(0.0, 0.0), request).unwrap_err();
            assert!(error.contains("while its person step is pending"), "{error}");
        }
        let ended = movers.step("v", pose(0.0, 0.0), Request::PersonEnd(PersonEnd { elevation: 1.0 })).unwrap();
        assert_near(ended.pose.position, [0.0, 1.01, 3.0]);
    }

    #[test]
    fn ending_a_person_step_that_was_not_begun_is_an_error() {
        let mut movers = movers(Kind::Person, pose(0.0, 0.0));
        let error = movers.step("v", pose(0.0, 0.0), Request::PersonEnd(PersonEnd { elevation: 1.0 })).unwrap_err();
        assert!(error.contains("without a pending person_begin"), "{error}");
        // a finished step is not pending either
        movers
            .step("v", pose(0.0, 0.0), Request::PersonBegin(person_begin([0.0, 0.0, 100.0], 1.5, true)))
            .unwrap();
        movers.step("v", pose(0.0, 0.0), Request::PersonEnd(PersonEnd { elevation: 1.0 })).unwrap();
        let error = movers.step("v", pose(0.0, 0.0), Request::PersonEnd(PersonEnd { elevation: 1.0 })).unwrap_err();
        assert!(error.contains("without a pending person_begin"), "{error}");
    }

    #[test]
    fn a_person_copy_the_host_did_not_make_is_refused_in_both_phases() {
        let mut movers = movers(Kind::Person, pose(0.0, 0.0));
        let error = movers
            .step("v", pose(1.0, 0.0), Request::PersonBegin(person_begin([0.0, 0.0, 100.0], 1.5, true)))
            .unwrap_err();
        assert!(error.contains("position[0]"), "{error}");
        movers
            .step("v", pose(0.0, 0.0), Request::PersonBegin(person_begin([0.0, 0.0, 100.0], 1.5, true)))
            .unwrap();
        let error = movers.step("v", pose(0.0, 2.0), Request::PersonEnd(PersonEnd { elevation: 1.0 })).unwrap_err();
        assert!(error.contains("position[2]"), "{error}");
    }

    #[test]
    fn a_request_for_another_kind_or_an_unsupported_one_is_refused() {
        let mut routed = movers(Kind::Routed, pose(0.0, 0.0));
        let mut ship = movers(Kind::Ship, pose(0.0, 0.0));
        let mut person = movers(Kind::Person, pose(0.0, 0.0));
        let mut fixed_wing = movers(Kind::FixedWing, pose(0.0, 0.0));
        let mut helicopter = movers(Kind::Helicopter, pose(0.0, 0.0));
        let requests = [
            Request::Advance(advance([0.0, 0.0, 10.0])),
            Request::Stop,
            Request::ShipMove(ship_move([0.0, 0.0, 10.0])),
            Request::PersonBegin(person_begin([0.0, 0.0, 10.0], 1.0, false)),
            Request::PersonEnd(PersonEnd { elevation: 0.0 }),
            Request::FixedWingMove(fixed_wing_move(FixedWingPhase::Taxi)),
            Request::HelicopterBegin(helicopter_begin_request()),
            Request::HelicopterEnd(helicopter_end_request()),
        ];
        let own = |kind: Kind, request: &Request| request.kind() == Some(kind);
        for request in requests {
            for (kind, movers) in [
                (Kind::Routed, &mut routed),
                (Kind::Ship, &mut ship),
                (Kind::Person, &mut person),
                (Kind::FixedWing, &mut fixed_wing),
                (Kind::Helicopter, &mut helicopter),
            ] {
                let result = movers.step("v", pose(0.0, 0.0), request);
                if !own(kind, &request) {
                    let error = result.unwrap_err();
                    assert!(error.contains(&format!("is not a request for {kind:?}")), "{error}");
                }
            }
        }
    }

    #[test]
    fn placing_a_ship_a_person_or_an_aircraft_adopts_the_campaign_pose() {
        for kind in [Kind::Ship, Kind::Person, Kind::FixedWing, Kind::Helicopter] {
            let mut movers = movers(kind, pose(0.0, 0.0));
            let mut moving = pose(50.0, 60.0);
            moving.velocity = 7.0;
            let adopted = movers.step("v", moving, PLACED).unwrap();
            assert_eq!(adopted.pose, moving);
        }
    }

    #[test]
    fn placing_an_aircraft_adopts_the_campaign_motion_vector_and_other_kinds_ignore_it() {
        let mut movers = movers(Kind::FixedWing, pose(0.0, 0.0));
        let placed = Request::Place {
            motion_vector: [0.0, 0.0, 40.0],
        };
        movers.step("v", pose(0.0, 0.0), placed).unwrap();
        // the host now flies at the placed 40 m/s, and the 100 m/s mirror is a write it did not make
        let mut flight = fixed_wing_move(FixedWingPhase::Flight);
        flight.elevation = 0.0;
        // the flight clamps the speed to 0.7 to 2 times the cruise velocity
        flight.cruise_velocity = 40.0;
        let mut stale = flight;
        stale.motion_vector = [0.0, 0.0, 100.0];
        let error = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(stale)).unwrap_err();
        assert!(error.contains("motion_vector"), "{error}");
        flight.motion_vector = [0.0, 0.0, 40.0];
        let first = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(flight)).unwrap();
        assert_eq!(first.pose.velocity, 40.0);
        let mut ship = self::movers(Kind::Ship, pose(0.0, 0.0));
        ship.step("v", pose(0.0, 0.0), placed).unwrap();
    }

    #[test]
    fn an_aircraft_needs_a_motion_vector_seed_and_no_other_kind_takes_one() {
        let mut movers = Movers::default();
        let error = movers.bind("v", Kind::FixedWing, pose(0.0, 0.0), None, None).unwrap_err();
        assert!(error.contains("without a motion vector"), "{error}");
        for kind in [Kind::Routed, Kind::Ship, Kind::Person] {
            let error = movers.bind("v", kind, pose(0.0, 0.0), Some([0.0; 3]), None).unwrap_err();
            assert!(error.contains("with a motion vector"), "{error}");
        }
        let error = movers
            .step("v", pose(0.0, 0.0), Request::FixedWingMove(fixed_wing_move(FixedWingPhase::Flight)))
            .unwrap_err();
        assert!(error.contains("unknown host id"), "nothing was bound: {error}");
    }

    #[test]
    fn a_taxiing_aircraft_accelerates_on_the_ground_and_keeps_to_the_terrain() {
        let mut movers = movers(Kind::FixedWing, pose(0.0, 0.0));
        let mut taxi = fixed_wing_move(FixedWingPhase::Taxi);
        taxi.delta_seconds = 1.0;
        taxi.wp_vel = 10.0;
        let first = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(taxi)).unwrap();
        // min(10 - 0, 3 m/s^2) over 1 s, due +z; the height is the terrain plus the centre of gravity
        assert_eq!(first.pose.velocity, 3.0);
        assert_near(first.pose.position, [0.0, 12.0, 3.0]);
        assert_near(first.motion_vector, [0.0, 0.0, 3.0]);
        assert!(!first.rotated);
    }

    #[test]
    fn a_flying_aircraft_follows_its_motion_vector_and_scales_the_new_one_by_the_frame_delta() {
        let mut movers = movers(Kind::FixedWing, pose(0.0, 0.0));
        // the terrain is level with the aircraft, so the clamp does not lift it
        let mut flight = fixed_wing_move(FixedWingPhase::Flight);
        flight.elevation = 0.0;
        let first = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(flight)).unwrap();
        // 100 m/s along +z for the 0.5 s engine delta; (new - old) * the frame reciprocal 2
        assert_eq!(first.pose.velocity, 100.0);
        assert_near(first.pose.position, [0.0, 0.0, 50.0]);
        assert_near(first.motion_vector, [0.0, 0.0, 100.0]);
    }

    #[test]
    fn a_landing_aircraft_takes_the_waypoint_speed_at_once() {
        let mut movers = movers(Kind::FixedWing, pose(0.0, 0.0));
        let mut land = fixed_wing_move(FixedWingPhase::Land);
        land.wp_vel = 50.0;
        land.delta_seconds = 1.0;
        land.frame_delta = 1.0;
        land.elevation = 0.0;
        let first = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(land)).unwrap();
        assert_eq!(first.pose.velocity, 50.0);
        assert_near(first.pose.position, [0.0, 0.0, 50.0]);
        assert_near(first.motion_vector, [0.0, 0.0, 50.0]);
    }

    #[test]
    fn the_absolute_flight_integrates_from_the_attitude_before_its_turn_and_never_reports_rotation() {
        let mut movers = movers(Kind::FixedWing, pose(0.0, 0.0));
        let mut absolute = fixed_wing_move(FixedWingPhase::Absolute);
        absolute.waypoint = [5000.0, 0.0, 0.0];
        absolute.delta_seconds = 1.0;
        absolute.elevation = 0.0;
        let first = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(absolute)).unwrap();
        // min(100 - 0, 30 m/s^2) over 1 s, along the old +z axis although the waypoint is abeam
        assert_eq!(first.pose.velocity, 30.0);
        assert_near(first.pose.position, [0.0, 0.0, 30.0]);
        assert!(!first.rotated, "the absolute flight never reports a rotation");
    }

    #[test]
    fn an_aircraft_whose_campaign_copy_differs_from_the_record_is_refused() {
        let mut movers = movers(Kind::FixedWing, pose(0.0, 0.0));
        let error = movers
            .step("v", pose(0.0, 1.0), Request::FixedWingMove(fixed_wing_move(FixedWingPhase::Flight)))
            .unwrap_err();
        assert!(error.contains("position[2]") && error.contains("differs from the host's record"), "{error}");
    }

    #[test]
    fn an_aircraft_whose_only_difference_is_the_campaign_motion_vector_is_refused_before_the_host_moves() {
        let mut movers = movers(Kind::FixedWing, pose(0.0, 0.0));
        let mut flight = fixed_wing_move(FixedWingPhase::Flight);
        flight.elevation = 0.0;
        // the pose is exactly the host's; the mirror differs in the last bit of one component
        flight.motion_vector[2] = f32::from_bits(100.0_f32.to_bits() + 1);
        let error = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(flight)).unwrap_err();
        assert!(
            error.contains("motion_vector[2]") && error.contains("differs from the host's record"),
            "{error}"
        );
        // nothing moved and the host's vector is unchanged: the correct mirror still flies from it
        flight.motion_vector[2] = 100.0;
        let first = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(flight)).unwrap();
        assert_near(first.pose.position, [0.0, 0.0, 50.0]);
    }

    #[test]
    fn the_next_step_flies_from_the_vector_the_host_answered() {
        let mut movers = movers(Kind::FixedWing, pose(0.0, 0.0));
        let mut taxi = fixed_wing_move(FixedWingPhase::Taxi);
        taxi.delta_seconds = 1.0;
        taxi.wp_vel = 10.0;
        let first = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(taxi)).unwrap();
        // the stale mirror (the seed) is refused; the answered vector is accepted
        let error = movers.step("v", first.pose, Request::FixedWingMove(taxi)).unwrap_err();
        assert!(error.contains("motion_vector"), "{error}");
        taxi.motion_vector = first.motion_vector;
        movers.step("v", first.pose, Request::FixedWingMove(taxi)).unwrap();
    }
    #[test]
    fn a_helicopter_step_is_two_phases_and_the_second_flies_from_the_vector_the_first_answered() {
        let mut begin = helicopter_begin_request();
        begin.acceleration_x = 4.0;
        begin.wp_vec_y = 3.0;
        let (mut movers, first) = begun(begin);
        // the vector is advanced by the acceleration over the step, the climb demand replaces y, and the speed is its magnitude
        assert_near(first.motion_vector, [4.0, 3.0, 100.0]);
        assert!(first.moved && first.pending && !first.rotated, "phase one answers an open step");
        assert_near([first.pose.velocity, 0.0, 0.0], [(16.0f32 + 9.0 + 10000.0).sqrt(), 0.0, 0.0]);
        assert_eq!(first.pose.position, [0.0; 3], "phase one moves nothing");
        let mut end = helicopter_end_request();
        end.motion_vector = first.motion_vector;
        let second = movers.step("v", first.pose, Request::HelicopterEnd(end)).unwrap();
        assert_near(second.pose.position, [4.0, 3.0, 100.0]);
        assert_eq!(second.pose.velocity, first.pose.velocity, "phase two does not recompute the speed");
        assert!(!second.rotated && !second.pending, "phase two closes the step");
        assert_eq!(second.motion_vector, first.motion_vector);
    }

    /// the attitude a helicopter at the level attitude ends a one second step with, when guidance asks for `desired_heading` under `max_turn_rate`
    fn turned(desired_heading: f32, max_turn_rate: f32) -> Outcome {
        let (mut movers, first) = begun(helicopter_begin_request());
        let mut end = helicopter_end_request();
        end.motion_vector = first.motion_vector;
        end.desired_heading = desired_heading;
        end.max_turn_rate = max_turn_rate;
        movers.step("v", first.pose, Request::HelicopterEnd(end)).unwrap()
    }

    fn assert_attitude(actual: [f32; 9], expected: [f32; 9]) {
        for (actual, expected) in actual.iter().zip(expected) {
            assert!((actual - expected).abs() < 1.0e-5, "{actual:?} vs {expected:?}");
        }
    }

    #[test]
    fn a_helicopter_turns_the_shortest_way_to_the_desired_heading() {
        use std::f32::consts::{FRAC_PI_2, PI};
        let wide = 10.0;
        // a demand just inside half a turn is followed directly; the same angle the other way round the circle wraps to it
        assert_attitude(turned(0.75 * PI, wide).pose.attitude, turned(-1.25 * PI, wide).pose.attitude);
        assert_attitude(turned(-0.75 * PI, wide).pose.attitude, turned(1.25 * PI, wide).pose.attitude);
        // beyond half a turn the shortest turn is the other way: 3/2 pi is a quarter turn back, not a three-quarter turn on
        assert_attitude(turned(3.0 * FRAC_PI_2, wide).pose.attitude, turned(-FRAC_PI_2, wide).pose.attitude);
        let direct = turned(0.75 * PI, wide).pose.attitude;
        assert!(direct.iter().zip(turned(-0.75 * PI, wide).pose.attitude).any(|(a, b)| (a - b).abs() > 1.0e-3));
        assert!(turned(3.0 * FRAC_PI_2, wide).rotated);
    }

    #[test]
    fn a_helicopter_turns_no_faster_than_its_turn_rate() {
        let limit = 0.25;
        // a demand beyond the limit turns by exactly the limit over the step, in either direction
        assert_attitude(turned(1.0, limit).pose.attitude, turned(limit, 10.0).pose.attitude);
        assert_attitude(turned(-1.0, limit).pose.attitude, turned(-limit, 10.0).pose.attitude);
        // a demand inside the limit is reached, not the limit
        assert_attitude(turned(0.1, limit).pose.attitude, turned(0.1, 10.0).pose.attitude);
        assert!(turned(1.0, limit)
            .pose
            .attitude
            .iter()
            .zip(turned(1.0, 10.0).pose.attitude)
            .any(|(a, b)| (a - b).abs() > 1.0e-3));
        // a zero limit does not turn at all
        assert!(!turned(1.0, 0.0).rotated);
    }

    #[test]
    fn a_hovering_helicopter_has_not_moved() {
        let mut begin = helicopter_begin_request();
        begin.motion_vector = [0.0, 0.0, 0.0];
        let mut movers = Movers::default();
        movers.bind("v", Kind::Helicopter, pose(0.0, 0.0), Some([0.0; 3]), Some(ROTOR)).unwrap();
        let first = movers.step("v", pose(0.0, 0.0), Request::HelicopterBegin(begin)).unwrap();
        assert!(!first.moved);
        assert_eq!(first.pose.velocity, 0.0);
    }

    #[test]
    fn the_floor_clamp_stops_the_climb_after_the_speed_was_taken() {
        let mut begin = helicopter_begin_request();
        begin.wp_vec_y = -5.0;
        let (mut movers, first) = begun(begin);
        let speed = first.pose.velocity;
        assert_near([speed, 0.0, 0.0], [(25.0f32 + 10000.0).sqrt(), 0.0, 0.0]);
        let mut end = helicopter_end_request();
        end.motion_vector = first.motion_vector;
        end.elevation = 0.0;
        end.cg_to_ground = 2.0;
        let second = movers.step("v", first.pose, Request::HelicopterEnd(end)).unwrap();
        // the position falls to -5, below the floor (2 m, level attitude): put back on it, the downward vector stopped, the speed untouched
        assert_near(second.pose.position, [0.0, 2.0, 100.0]);
        assert_eq!(second.motion_vector[1], 0.0);
        assert_eq!(second.pose.velocity, speed);
        // terrain-follow raises the floor by 2 m
        let (mut movers, first) = begun(begin);
        end.terrain_follow = true;
        let second = movers.step("v", first.pose, Request::HelicopterEnd(end)).unwrap();
        assert_near(second.pose.position, [0.0, 4.0, 100.0]);
    }

    #[test]
    fn a_helicopter_is_bounded_to_the_map() {
        let (mut movers, first) = begun(helicopter_begin_request());
        let mut end = helicopter_end_request();
        end.motion_vector = first.motion_vector;
        end.map_max = [1.0e6, 1.0e6, 60.0];
        let second = movers.step("v", first.pose, Request::HelicopterEnd(end)).unwrap();
        assert_near(second.pose.position, [0.0, 0.0, 60.0]);
    }

    #[test]
    fn a_helicopter_reports_a_rotation_only_beyond_a_tenth_of_a_degree() {
        // pitch and roll move half the demand over a one second step; the heading moves the demand (inside the turn limit)
        let tenth_of_a_degree = 0.1f32.to_radians();
        for (step, rotated) in [(tenth_of_a_degree * 0.99, false), (tenth_of_a_degree * 1.01, true)] {
            for axis in 0..3 {
                let (mut movers, first) = begun(helicopter_begin_request());
                let mut end = helicopter_end_request();
                end.motion_vector = first.motion_vector;
                match axis {
                    0 => end.desired_pitch = step * 2.0,
                    1 => end.desired_roll = step * 2.0,
                    _ => end.desired_heading = step,
                }
                let second = movers.step("v", first.pose, Request::HelicopterEnd(end)).unwrap();
                assert_eq!(second.rotated, rotated, "axis {axis} step {step}");
            }
        }
    }

    #[test]
    fn a_helicopter_end_without_a_begin_or_a_second_begin_is_refused_before_any_change() {
        let mut movers = movers(Kind::Helicopter, pose(0.0, 0.0));
        let error = movers.step("v", pose(0.0, 0.0), Request::HelicopterEnd(helicopter_end_request())).unwrap_err();
        assert!(error.contains("without a pending helicopter_begin"), "{error}");
        let first = movers.step("v", pose(0.0, 0.0), Request::HelicopterBegin(helicopter_begin_request())).unwrap();
        // while the step is pending nothing else is accepted: not another begin, not a placement, not another kind's request
        for request in [Request::HelicopterBegin(helicopter_begin_request()), PLACED, Request::Stop] {
            let error = movers.step("v", first.pose, request).unwrap_err();
            assert!(error.contains("pending") || error.contains("is not a request"), "{error}");
        }
        let mut end = helicopter_end_request();
        end.motion_vector = first.motion_vector;
        let second = movers.step("v", first.pose, Request::HelicopterEnd(end)).unwrap();
        assert_near(second.pose.position, [0.0, 0.0, 100.0]);
        // the step is finished: a second end is refused again
        let error = movers.step("v", second.pose, Request::HelicopterEnd(end)).unwrap_err();
        assert!(error.contains("without a pending helicopter_begin"), "{error}");
    }

    #[test]
    fn a_helicopter_mirror_that_differs_is_refused_and_nothing_moves() {
        let mut movers = movers(Kind::Helicopter, pose(0.0, 0.0));
        let mut begin = helicopter_begin_request();
        begin.motion_vector[2] = f32::from_bits(100.0f32.to_bits() + 1);
        let error = movers.step("v", pose(0.0, 0.0), Request::HelicopterBegin(begin)).unwrap_err();
        assert!(
            error.contains("motion_vector[2]") && error.contains("differs from the host's record"),
            "{error}"
        );
        // not pending, not moved: the correct begin still works from the seed
        begin.motion_vector[2] = 100.0;
        let first = movers.step("v", pose(0.0, 0.0), Request::HelicopterBegin(begin)).unwrap();
        // a stale pose or phase-two mirror is refused too, and the pending step survives for the correct one
        let mut end = helicopter_end_request();
        end.motion_vector = first.motion_vector;
        let mut stale = first.pose;
        stale.velocity += 1.0;
        let error = movers.step("v", stale, Request::HelicopterEnd(end)).unwrap_err();
        assert!(error.contains("velocity"), "{error}");
        let mut wrong = end;
        wrong.motion_vector[0] = 1.0;
        let error = movers.step("v", first.pose, Request::HelicopterEnd(wrong)).unwrap_err();
        assert!(error.contains("motion_vector[0]"), "{error}");
        movers.step("v", first.pose, Request::HelicopterEnd(end)).unwrap();
    }

    #[test]
    fn a_helicopter_is_never_bound_without_its_motion_vector() {
        let mut movers = Movers::default();
        let error = movers.bind("v", Kind::Helicopter, pose(0.0, 0.0), None, Some(ROTOR)).unwrap_err();
        assert!(error.contains("without a motion vector"), "{error}");
        let error = movers
            .step("v", pose(0.0, 0.0), Request::HelicopterBegin(helicopter_begin_request()))
            .unwrap_err();
        assert!(error.contains("unknown host id"), "{error}");
    }

    #[test]
    fn placing_a_helicopter_adopts_the_campaign_motion_vector() {
        let mut movers = movers(Kind::Helicopter, pose(0.0, 0.0));
        let placed = Request::Place {
            motion_vector: [1.0, 2.0, 3.0],
        };
        movers.step("v", pose(5.0, 6.0), placed).unwrap();
        let mut begin = helicopter_begin_request();
        begin.motion_vector = [1.0, 2.0, 3.0];
        let first = movers.step("v", pose(5.0, 6.0), Request::HelicopterBegin(begin)).unwrap();
        assert_near(first.motion_vector, [1.0, 0.0, 3.0]);
    }

    // ---- the helicopter lifecycle: rotor, dying, crash and impact ----

    /// the vector a test helicopter flies with, and the one it falls with
    const SEED: [f32; 3] = [0.0, 0.0, 100.0];
    const DROP: [f32; 3] = [0.0, -20.0, 0.0];

    fn at(x: f32, y: f32, z: f32) -> Pose {
        Pose {
            position: [x, y, z],
            velocity: 0.0,
            attitude: LEVEL,
        }
    }

    /// a lifecycle request with every input neutral: a test sets the ones its mode owns
    fn life(mode: LifecycleMode, rotor: [f32; 3], motion_vector: [f32; 3]) -> Lifecycle {
        Lifecycle {
            mode,
            rotor,
            motion_vector,
            delta_seconds: 0.5,
            lift: 0.0,
            power_output: 1.0,
            flag: false,
            gravity: 0.0,
            terrain_class: 0,
            bury_limit: 0.0,
            elevation: 0.0,
            cg_to_ground: 0.0,
            draws: [0.0; 2],
            pitch_offset: 0.0,
            bank_offset: 0.0,
            signed_speed: 0.0,
            forest: false,
            face_normal: [0.0, 1.0, 0.0],
            map_min: [-1.0e6; 3],
            map_max: [1.0e6; 3],
        }
    }

    fn run(movers: &mut Movers, campaign: Pose, step: Lifecycle) -> Result<Outcome, String> {
        movers.step("v", campaign, Request::Lifecycle(step))
    }

    fn flying(start: Pose, vector: [f32; 3]) -> Movers {
        let mut movers = Movers::default();
        movers.bind("v", Kind::Helicopter, start, Some(vector), Some(ROTOR)).unwrap();
        movers
    }

    fn assert_within(actual: &[f32], expected: &[f32], tolerance: f32) {
        for (actual, expected) in actual.iter().zip(expected) {
            assert!((actual - expected).abs() <= tolerance, "{actual:?} vs {expected:?} (within {tolerance})");
        }
    }

    /// a helicopter dropped from 10 m at 20 m/s: after the fall its candidate is 10 m under the origin and in contact at
    /// altitude -12 (the pre-kill CG height is 2), and the campaign has made its splash placement at (5, 0, 6)
    fn in_contact() -> (Movers, Pose) {
        let start = at(0.0, 10.0, 0.0);
        let mut movers = flying(start, DROP);
        let mut fall = life(LifecycleMode::Fall, ROTOR, DROP);
        fall.delta_seconds = 1.0;
        run(&mut movers, start, fall).unwrap();
        let mut contact = life(LifecycleMode::Contact, ROTOR, DROP);
        contact.flag = true;
        contact.cg_to_ground = 2.0;
        let touched = run(&mut movers, start, contact).unwrap();
        assert!(touched.moved);
        assert_within(&[touched.altitude], &[-12.0], 1.0e-5);
        let splash = at(5.0, 0.0, 6.0);
        movers.step("v", splash, Request::Place { motion_vector: DROP }).unwrap();
        (movers, splash)
    }

    fn settle_step(terrain_class: i32, elevation: f32, cg_to_ground: f32, bury_limit: f32) -> Lifecycle {
        let mut settle = life(LifecycleMode::Settle, ROTOR, DROP);
        settle.terrain_class = terrain_class;
        settle.elevation = elevation;
        settle.cg_to_ground = cg_to_ground;
        settle.bury_limit = bury_limit;
        settle
    }

    #[test]
    fn a_dying_rotor_winds_down_by_its_law_and_an_idle_one_linearly_by_the_frame_delta() {
        let seed = life(LifecycleMode::RotorSet, [50.0, 50.0, 3.0], SEED);
        let mut dying = movers(Kind::Helicopter, pose(0.0, 0.0));
        run(&mut dying, pose(0.0, 0.0), seed).unwrap();
        let down = run(&mut dying, pose(0.0, 0.0), life(LifecycleMode::RotorDown, [50.0, 50.0, 3.0], SEED)).unwrap();
        // 50 rpm: the wind-down factor is 50^2 / 3600, so 50 - 50 * 0.6944 * 0.5; the tail follows and the timer is untouched
        assert_within(&down.rotor, &[32.6389, 32.6389, 3.0], 1.0e-3);

        let mut idle = movers(Kind::Helicopter, pose(0.0, 0.0));
        run(&mut idle, pose(0.0, 0.0), seed).unwrap();
        let ticked = run(&mut idle, pose(0.0, 0.0), life(LifecycleMode::RotorIdle, [50.0, 50.0, 3.0], SEED)).unwrap();
        assert_eq!(ticked.rotor, [47.5, 47.5, 3.0]);

        // idle never goes below zero; the answer is what the next step must mirror
        let mut late = life(LifecycleMode::RotorIdle, ticked.rotor, SEED);
        late.delta_seconds = 100.0;
        assert_eq!(run(&mut idle, pose(0.0, 0.0), late).unwrap().rotor, [0.0, 0.0, 3.0]);
    }

    #[test]
    fn an_alive_rotor_spins_up_and_only_the_campaigns_gate_holds_the_step_and_counts_the_timer_down() {
        let spin_up = |flag: bool, rotor: [f32; 3], lift: f32| {
            let mut movers = movers(Kind::Helicopter, pose(0.0, 0.0));
            run(&mut movers, pose(0.0, 0.0), life(LifecycleMode::RotorSet, rotor, SEED)).unwrap();
            let mut alive = life(LifecycleMode::RotorAlive, rotor, SEED);
            alive.lift = lift;
            alive.flag = flag;
            run(&mut movers, pose(0.0, 0.0), alive).unwrap()
        };
        // stopped, winding up: a tenth of a percent of the way (the 1% floor of the spin-up factor), the timer 2 -> 1.5
        let gated = spin_up(true, [0.0, 0.0, 2.0], 1.0);
        assert!(gated.moved);
        assert_within(&gated.rotor, &[0.5, 0.5, 1.5], 1.0e-5);
        // the same step for a helicopter the campaign does not gate leaves the timer
        let free = spin_up(false, [0.0, 0.0, 2.0], 1.0);
        assert!(!free.moved);
        assert_within(&free.rotor, &[0.5, 0.5, 2.0], 1.0e-5);
        // at speed with the timer spent the gate does not hold, and 75% lift settles the rotor towards 75
        let flying = spin_up(true, [100.0, 100.0, 0.0], 0.5);
        assert!(!flying.moved);
        assert_within(&flying.rotor, &[87.5, 87.5, 0.0], 1.0e-4);
    }

    #[test]
    fn rotor_set_adopts_the_campaigns_rotor_and_the_old_mirror_is_then_refused() {
        let mut movers = movers(Kind::Helicopter, pose(0.0, 0.0));
        let set = run(&mut movers, pose(0.0, 0.0), life(LifecycleMode::RotorSet, [40.0, 40.0, 5.0], SEED)).unwrap();
        assert_eq!(set.rotor, [40.0, 40.0, 5.0]);
        let error = run(&mut movers, pose(0.0, 0.0), life(LifecycleMode::RotorIdle, ROTOR, SEED)).unwrap_err();
        assert!(error.contains("rotor[0]"), "{error}");
        let idle = run(&mut movers, pose(0.0, 0.0), life(LifecycleMode::RotorIdle, [40.0, 40.0, 5.0], SEED)).unwrap();
        assert_eq!(idle.rotor, [37.5, 37.5, 5.0]);
    }

    #[test]
    fn a_rotor_or_vector_mirror_that_differs_is_refused_and_nothing_is_stored() {
        let mut movers = movers(Kind::Helicopter, pose(0.0, 0.0));
        let error = run(&mut movers, pose(0.0, 0.0), life(LifecycleMode::RotorIdle, [99.0, 100.0, 0.0], SEED)).unwrap_err();
        assert!(error.contains("rotor[0]"), "{error}");
        let error = run(&mut movers, pose(0.0, 0.0), life(LifecycleMode::Impact, ROTOR, [0.0, 0.0, 99.0])).unwrap_err();
        assert!(error.contains("motion_vector[2]"), "{error}");
        // the adopting mode checks the vector too, and keeps the rotor it had
        let error = run(&mut movers, pose(0.0, 0.0), life(LifecycleMode::RotorSet, [1.0, 1.0, 1.0], [0.0, 0.0, 1.0])).unwrap_err();
        assert!(error.contains("motion_vector[2]"), "{error}");
        // a refused fall stored no candidate for the gravity step to find
        let error = run(&mut movers, pose(0.0, 0.0), life(LifecycleMode::Fall, ROTOR, [0.0, 0.0, 99.0])).unwrap_err();
        assert!(error.contains("motion_vector[2]"), "{error}");
        let error = run(&mut movers, pose(0.0, 0.0), life(LifecycleMode::FallGravity, ROTOR, SEED)).unwrap_err();
        assert!(error.contains("without a stored candidate"), "{error}");
        // and the true mirror still steps from the untouched record
        let idle = run(&mut movers, pose(0.0, 0.0), life(LifecycleMode::RotorIdle, ROTOR, SEED)).unwrap();
        assert_eq!(idle.rotor, [97.5, 97.5, 0.0]);
    }

    #[test]
    fn a_fall_returns_the_candidate_only_and_gravity_applies_the_vector_and_speed_afterwards() {
        let start = at(10.0, 100.0, 20.0);
        let mut movers = flying(start, SEED);
        let mut fall = life(LifecycleMode::Fall, ROTOR, SEED);
        fall.map_max = [1.0e6, 1.0e6, 1.0e6];
        let fell = run(&mut movers, start, fall).unwrap();
        assert_within(&fell.pose.position, &[10.0, 100.0, 70.0], 1.0e-4);
        assert_eq!(fell.motion_vector, SEED);
        assert!(!fell.pending && !fell.moved);

        // the record did not move: the campaign's unchanged pose still mirrors it, and the speed is not yet touched
        let mut gravity = life(LifecycleMode::FallGravity, ROTOR, SEED);
        gravity.gravity = 10.0;
        let after = run(&mut movers, start, gravity).unwrap();
        assert_within(&after.motion_vector, &[0.0, -5.0, 100.0], 1.0e-5);
        assert_within(&after.pose.position, &start.position, 0.0);
        assert_within(&[after.pose.velocity], &[(25.0f32 + 10000.0).sqrt()], 1.0e-3);

        // the tumble places the candidate, once
        let mut tumble = life(LifecycleMode::Tumble, ROTOR, after.motion_vector);
        tumble.draws = [0.25, -0.5];
        let tumbled = run(&mut movers, after.pose, tumble).unwrap();
        assert_within(&tumbled.pose.position, &[10.0, 100.0, 70.0], 1.0e-4);
        let error = run(&mut movers, tumbled.pose, tumble).unwrap_err();
        assert!(error.contains("without a stored candidate"), "{error}");
    }

    /// The original dying tumble is `heading += PI * 0.2 * frand1 () * dt` with `PI` a float constant and `frand1 ()` a
    /// double: the product is formed in double and only the sum is narrowed. A draw narrowed to float first lands the
    /// heading or roll one float step away for some draws, which is what moved a recorded wreck's height from 0.75041 to
    /// 0.75040 in the native pack.
    #[test]
    fn a_tumble_forms_its_draws_in_double_as_the_original_expression_does() {
        // the original's `PI` (3.14159265359f) is this float
        const PI: f32 = std::f32::consts::PI;
        const DT: f32 = 1.0 / 30.0;
        let heading_for = |draw: f64| (0.0 + f64::from(PI) * 0.2 * draw * f64::from(DT)) as f32;
        let roll_for = |draw: f64| (0.0 - f64::from(PI) * 0.2 * draw * f64::from(DT)) as f32;
        let tumble = |heading: f64, roll: f64| {
            let mut attitude = LEVEL;
            // SAFETY: the attitude is 9 floats read then written.
            unsafe { eech_hc_tumble(attitude.as_mut_ptr(), heading, roll, DT) };
            attitude
        };
        // a draw whose exact product gives back `want` as the float sum, searched among the neighbouring doubles
        let reproducing = |want: f32, form: &dyn Fn(f64) -> f32, sign: f64, spacing: f64| {
            let centre = sign * f64::from(want) / (f64::from(PI) * 0.2 * f64::from(DT));
            (0..64)
                .flat_map(|step| [centre + f64::from(step) * spacing, centre - f64::from(step) * spacing])
                .find(|candidate| form(*candidate).to_bits() == want.to_bits())
        };
        let mut separated = 0;
        for k in 1..=2000u32 {
            let heading_draw = f64::from(k) * (1.0 / 32767.0);
            let roll_draw = f64::from((k * 7) % 32767 + 1) * (1.0 / 32767.0);
            let narrowed = (f64::from(heading_draw as f32), f64::from(roll_draw as f32));
            let heading = heading_for(heading_draw);
            let roll = roll_for(roll_draw);
            if heading.to_bits() == heading_for(narrowed.0).to_bits() && roll.to_bits() == roll_for(narrowed.1).to_bits() {
                continue;
            }
            // the draws as transported (double) against the draws as the old float transport delivered them
            let exact = tumble(heading_draw, roll_draw);
            assert_ne!(
                exact,
                tumble(narrowed.0, narrowed.1),
                "draw {k}: the float-narrowed draws give the same attitude"
            );
            // the host's attitude is the one for the float heading and roll the original expression assigns: the same
            // attitude arises from any pair of draws that forms those floats in double
            let (Some(h), Some(r)) = (reproducing(heading, &heading_for, 1.0, 1.0e-12), reproducing(roll, &roll_for, -1.0, 1.0e-12)) else {
                continue;
            };
            assert_eq!(
                exact,
                tumble(h, r),
                "draw {k}: the attitude is not the one for heading {heading} and roll {roll}"
            );
            separated += 1;
        }
        assert!(separated >= 50, "only {separated} draws separate the float transport from the double one");
    }

    /// The helper alone is not the transport: the step must hand the helper the draws it received, undiminished.
    #[test]
    fn the_tumble_step_hands_the_helper_the_draws_it_received_in_double() {
        const DT: f32 = 1.0 / 30.0;
        let start = at(10.0, 100.0, 20.0);
        let mut checked = 0;
        for k in 1..=2000u32 {
            let draws = [f64::from(k) * (1.0 / 32767.0), f64::from((k * 7) % 32767 + 1) * (1.0 / 32767.0)];
            let direct = |heading: f64, roll: f64| {
                let mut attitude = start.attitude;
                // SAFETY: the attitude is 9 floats read then written.
                unsafe { eech_hc_tumble(attitude.as_mut_ptr(), heading, roll, DT) };
                attitude
            };
            let exact = direct(draws[0], draws[1]);
            if exact == direct(f64::from(draws[0] as f32), f64::from(draws[1] as f32)) {
                continue;
            }
            let mut movers = flying(start, SEED);
            let mut fall = life(LifecycleMode::Fall, ROTOR, SEED);
            fall.map_max = [1.0e6, 1.0e6, 1.0e6];
            run(&mut movers, start, fall).unwrap();
            let mut tumble = life(LifecycleMode::Tumble, ROTOR, SEED);
            tumble.delta_seconds = DT;
            tumble.draws = draws;
            let tumbled = run(&mut movers, start, tumble).unwrap();
            assert_eq!(tumbled.pose.attitude, exact, "draw {k}: the step did not use the draws it was given");
            checked += 1;
        }
        assert!(checked >= 50, "only {checked} draws separate the float transport from the double one");
    }

    #[test]
    fn a_fall_is_bound_to_the_map_volume() {
        let start = at(0.0, 100.0, 0.0);
        let mut movers = flying(start, SEED);
        let mut fall = life(LifecycleMode::Fall, ROTOR, SEED);
        fall.map_max = [1.0e6, 1.0e6, 50.0];
        let fell = run(&mut movers, start, fall).unwrap();
        assert_within(&fell.pose.position, &[0.0, 100.0, 50.0], 1.0e-4);
    }

    #[test]
    fn the_retained_contact_altitude_buries_and_the_fresh_cg_only_sets_the_final_height_over_a_splash_place() {
        // altitude -12 retained. Sea: the held (splash) x and z, the candidate's height. A bury limit of -13 does not bury
        // by the retained -12, though the fresh CG of 5 would have measured -15 and buried.
        let (mut movers, splash) = in_contact();
        let kept = run(&mut movers, splash, settle_step(1, 0.0, 5.0, -13.0)).unwrap();
        assert!(!kept.moved);
        assert_within(&kept.pose.position, &[5.0, -10.0, 6.0], 1.0e-5);
        assert_eq!(kept.motion_vector, DROP);
        assert_eq!(kept.rotor, ROTOR);

        // a limit of -11 buries: the rotor stops, the vector clears; the position stays the candidate's
        let (mut movers, splash) = in_contact();
        let buried = run(&mut movers, splash, settle_step(1, 0.0, 5.0, -11.0)).unwrap();
        assert!(buried.moved);
        assert_eq!(buried.motion_vector, [0.0; 3]);
        assert_eq!(buried.rotor, [0.0, 0.0, 0.0]);
        assert_within(&buried.pose.position, &[5.0, -10.0, 6.0], 1.0e-5);

        // the campaign's clearing placement keeps the candidate, which the tumble then places (not the held position)
        movers.step("v", splash, Request::Place { motion_vector: [0.0; 3] }).unwrap();
        let tumbled = run(&mut movers, splash, life(LifecycleMode::Tumble, [0.0; 3], [0.0; 3])).unwrap();
        assert_within(&tumbled.pose.position, &[5.0, -10.0, 6.0], 1.0e-5);
        assert_within(&[tumbled.pose.velocity], &[0.0], 0.0);
    }

    #[test]
    fn solid_ground_and_a_river_settle_to_the_fresh_cg_height() {
        // solid ground: x and z stay the candidate's (0, 0), the height is the ground plus the destroyed CG
        let (mut movers, splash) = in_contact();
        let ground = run(&mut movers, splash, settle_step(0, 3.0, 4.0, -13.0)).unwrap();
        assert!(!ground.moved);
        assert_within(&ground.pose.position, &[0.0, 7.0, 0.0], 1.0e-5);

        // a river deeper than 3 m under the contact altitude: the bed height from the fresh CG, and buried
        let (mut movers, splash) = in_contact();
        let river = run(&mut movers, splash, settle_step(2, 0.0, 5.0, -100.0)).unwrap();
        assert!(river.moved);
        assert_within(&river.pose.position, &[5.0, 2.0, 6.0], 1.0e-5);
        assert_eq!(river.motion_vector, [0.0; 3]);
    }

    #[test]
    fn a_settle_needs_the_contact_of_its_own_candidate_and_consumes_it() {
        let start = at(0.0, 10.0, 0.0);
        let mut movers = flying(start, DROP);
        for mode in [
            LifecycleMode::FallGravity,
            LifecycleMode::Tumble,
            LifecycleMode::Settle,
            LifecycleMode::CrashEnd,
        ] {
            let error = run(&mut movers, start, life(mode, ROTOR, DROP)).unwrap_err();
            assert!(
                error.contains("without a stored candidate") || error.contains("without a pending crash_begin"),
                "{error}"
            );
        }
        let mut measure_candidate = life(LifecycleMode::Contact, ROTOR, DROP);
        measure_candidate.flag = true;
        let error = run(&mut movers, start, measure_candidate).unwrap_err();
        assert!(error.contains("without a stored candidate"), "{error}");

        run(&mut movers, start, life(LifecycleMode::Fall, ROTOR, DROP)).unwrap();
        // a fall alone, then a contact of the held position, give no contact of the candidate
        let error = run(&mut movers, start, settle_step(1, 0.0, 0.0, 0.0)).unwrap_err();
        assert!(error.contains("without the contact"), "{error}");
        run(&mut movers, start, life(LifecycleMode::Contact, ROTOR, DROP)).unwrap();
        let error = run(&mut movers, start, settle_step(1, 0.0, 0.0, 0.0)).unwrap_err();
        assert!(error.contains("without the contact"), "{error}");

        // the contact of the candidate lets one settle through, and a new fall forgets it
        run(&mut movers, start, measure_candidate).unwrap();
        run(&mut movers, start, life(LifecycleMode::Fall, ROTOR, DROP)).unwrap();
        let error = run(&mut movers, start, settle_step(1, 0.0, 0.0, 0.0)).unwrap_err();
        assert!(error.contains("without the contact"), "{error}");
        run(&mut movers, start, measure_candidate).unwrap();
        run(&mut movers, start, settle_step(1, 0.0, 0.0, 0.0)).unwrap();
        let error = run(&mut movers, start, settle_step(1, 0.0, 0.0, 0.0)).unwrap_err();
        assert!(error.contains("without the contact"), "{error}");
    }

    #[test]
    fn the_crash_slide_is_two_open_phases_and_returns_the_vector_before_it_is_cleared() {
        let start = at(0.0, 50.0, 0.0);
        let vector = [3.0, 0.0, 4.0];
        let mut movers = flying(start, vector);
        let mut begin = life(LifecycleMode::CrashBegin, ROTOR, vector);
        begin.delta_seconds = 1.0;
        let slid = run(&mut movers, start, begin).unwrap();
        assert!(slid.pending);
        assert_within(&slid.pose.position, &[3.0, 50.0, 4.0], 1.0e-5);

        // nothing else is accepted until the second phase, a placement included, and the refusals change nothing
        let held = Pose {
            position: start.position,
            ..slid.pose
        };
        let error = movers.step("v", held, Request::Place { motion_vector: vector }).unwrap_err();
        assert!(error.contains("crash slide is pending"), "{error}");
        let error = run(&mut movers, held, life(LifecycleMode::Impact, ROTOR, vector)).unwrap_err();
        assert!(error.contains("crash slide is pending"), "{error}");

        // a forest keeps the held height; the friction takes 40 * 0.05 = 2 off a speed of 5
        let mut end = life(LifecycleMode::CrashEnd, ROTOR, vector);
        end.delta_seconds = 0.05;
        end.forest = true;
        end.elevation = 30.0;
        end.cg_to_ground = 2.0;
        let ended = run(&mut movers, held, end).unwrap();
        assert!(!ended.moved && !ended.pending);
        assert_within(&ended.motion_vector, &[1.8, 0.0, 2.4], 1.0e-5);
        assert_within(&ended.pose.position, &[3.0, 50.0, 4.0], 1.0e-5);
        assert_within(&[ended.pose.velocity], &[3.0], 1.0e-5);
        // the second phase closed the slide: it cannot be ended twice, and a placement (the DLL's clear) is accepted again
        let error = run(&mut movers, ended.pose, end).unwrap_err();
        assert!(error.contains("without a pending crash_begin"), "{error}");
        movers.step("v", ended.pose, Request::Place { motion_vector: [0.0; 3] }).unwrap();
    }

    #[test]
    fn crash_end_settles_to_the_ground_stops_on_the_post_friction_speed_and_adds_the_slope_push() {
        let start = at(0.0, 50.0, 0.0);
        let vector = [3.0, 0.0, 4.0];
        let slide = |end: Lifecycle| {
            let mut movers = flying(start, vector);
            let mut begin = life(LifecycleMode::CrashBegin, ROTOR, vector);
            begin.delta_seconds = 1.0;
            let slid = run(&mut movers, start, begin).unwrap();
            let held = Pose {
                position: start.position,
                ..slid.pose
            };
            run(&mut movers, held, end).unwrap()
        };
        let mut end = life(LifecycleMode::CrashEnd, ROTOR, vector);
        end.delta_seconds = 0.05;
        end.elevation = 30.0;
        end.cg_to_ground = 2.0;
        end.face_normal = [0.1, 1.0, -0.2];
        // not a forest: the ground plus the CG (32), a fall of 18 m in 0.05 s, and g * normal * dt in x and z
        let ended = slide(end);
        assert!(!ended.moved);
        assert_within(&ended.pose.position, &[3.0, 32.0, 4.0], 1.0e-5);
        assert_within(&ended.motion_vector, &[1.8 + 9.81 * 0.1 * 0.05, -360.0, 2.4 - 9.81 * 0.2 * 0.05], 1.0e-3);

        // 5 m/s is above the stopping speed, but the friction (40 * 0.2 = 8) leaves none: the post-friction speed decides
        end.delta_seconds = 0.2;
        end.forest = true;
        end.face_normal = [0.0, 1.0, 0.0];
        let stopped = slide(end);
        assert!(stopped.moved);
        assert_within(&stopped.motion_vector, &[0.0; 3], 0.0);
    }

    #[test]
    fn crash_end_needs_a_crash_begin_whatever_candidate_a_fall_left() {
        let start = at(0.0, 10.0, 0.0);
        let mut movers = flying(start, DROP);
        let error = run(&mut movers, start, life(LifecycleMode::CrashEnd, ROTOR, DROP)).unwrap_err();
        assert!(error.contains("without a pending crash_begin"), "{error}");
        run(&mut movers, start, life(LifecycleMode::Fall, ROTOR, DROP)).unwrap();
        let error = run(&mut movers, start, life(LifecycleMode::CrashEnd, ROTOR, DROP)).unwrap_err();
        assert!(error.contains("without a pending crash_begin"), "{error}");
    }

    #[test]
    fn an_impact_shoves_along_the_attitudes_x_axis_and_the_next_step_mirrors_the_answer() {
        let mut movers = movers(Kind::Helicopter, pose(0.0, 0.0));
        let mut impact = life(LifecycleMode::Impact, ROTOR, SEED);
        impact.signed_speed = 3.0;
        let shoved = run(&mut movers, pose(0.0, 0.0), impact).unwrap();
        assert_within(&shoved.motion_vector, &[3.0, 0.0, 100.0], 1.0e-5);
        let error = run(&mut movers, pose(0.0, 0.0), impact).unwrap_err();
        assert!(error.contains("motion_vector[0]"), "{error}");
        impact.motion_vector = shoved.motion_vector;
        impact.signed_speed = -2.0;
        let shoved = run(&mut movers, pose(0.0, 0.0), impact).unwrap();
        assert_within(&shoved.motion_vector, &[1.0, 0.0, 100.0], 1.0e-5);
    }

    #[test]
    fn a_lifecycle_request_needs_its_helicopter_and_no_open_helicopter_step() {
        let mut routed = movers(Kind::Routed, pose(0.0, 0.0));
        let error = run(&mut routed, pose(0.0, 0.0), life(LifecycleMode::RotorDown, ROTOR, [0.0; 3])).unwrap_err();
        assert!(error.contains("rotor_down is not a request"), "{error}");
        let error = run(&mut Movers::default(), pose(0.0, 0.0), life(LifecycleMode::RotorDown, ROTOR, SEED)).unwrap_err();
        assert!(error.contains("unknown host id"), "{error}");
        let (mut open, first) = begun(helicopter_begin_request());
        assert!(first.pending);
        let error = run(&mut open, first.pose, life(LifecycleMode::RotorDown, ROTOR, first.motion_vector)).unwrap_err();
        assert!(error.contains("while its helicopter step is pending"), "{error}");
    }

    #[test]
    fn only_a_helicopter_is_bound_with_a_rotor_state_and_it_must_be_finite() {
        let mut movers = Movers::default();
        let error = movers.bind("v", Kind::Helicopter, pose(0.0, 0.0), Some(SEED), None).unwrap_err();
        assert!(error.contains("without a rotor state"), "{error}");
        let error = movers.bind("v", Kind::Routed, pose(0.0, 0.0), None, Some(ROTOR)).unwrap_err();
        assert!(error.contains("with a rotor state"), "{error}");
        let error = movers
            .bind("v", Kind::Helicopter, pose(0.0, 0.0), Some(SEED), Some([f32::NAN, 0.0, 0.0]))
            .unwrap_err();
        assert!(error.contains("non-finite rotor"), "{error}");
    }
}
