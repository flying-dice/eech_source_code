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

use std::collections::{HashMap, HashSet};
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

    fn eech_fw_navigate(input: *const FwNavInput, acceleration: *mut f32);

    fn eech_fw_facing_away(position: *const f32, motion_vector: *const f32, waypoint: *const f32) -> c_int;

    fn eech_fw_turn(input: *const FwTurnInput, turn_speed_multi: *mut f32, fixed_wing_toggle: *mut f32) -> c_int;

    fn eech_fw_lookahead(position: *const f32, velocity: f32, attitude: *const f32, point_xz: *mut f32);

    fn eech_fw_terrain_height(
        position: *const f32,
        velocity: f32,
        attitude: *const f32,
        guide_position: *const f32,
        terrain_elevation: f32,
        min_height: f32,
    ) -> f32;

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
    fn eech_place_attitude(face_normal: *const f32, heading: f32, fuselage_angle: f32, angles: *mut f32, attitude: *mut f32);
    fn eech_dependent_helicopter(
        parent_old: *const f32,
        parent_new: *const f32,
        heading: f32,
        position: *const f32,
        attitude: *const f32,
        out_position: *mut f32,
        out_attitude: *mut f32,
    );
    fn eech_expire_threshold(range: f32) -> f32;
    fn eech_expire_gate(timer: f32) -> c_int;
    fn eech_gv_death_candidate(position: *const f32, motion_vector: *const f32, delta_seconds: f32, out_xz: *mut f32);
    fn eech_gv_death_decay(
        candidate_xz: *const f32,
        elevation: f32,
        cg_to_ground: f32,
        motion_vector: *const f32,
        attitude: *const f32,
        delta_seconds: f32,
        wide: c_int,
        out_position: *mut f32,
        out_motion_vector: *mut f32,
        out_speed: *mut f32,
        out_attitude: *mut f32,
    );
    fn eech_gv_fall_candidate(position: *const f32, motion_vector: *const f32, delta_seconds: f32, out_position: *mut f32);
    fn eech_gv_fall_gravity(motion_vector: *mut f32, gravity: f32, delta_seconds: f32) -> f32;
    fn eech_gv_fall_settle(
        position: *const f32,
        candidate: *const f32,
        motion_vector: *mut f32,
        is_water: c_int,
        old_elevation: f32,
        elevation: f32,
        cg_to_ground: f32,
        bbox_diagonal: f32,
        out_position: *mut f32,
        out_speed: *mut f32,
    ) -> c_int;
    fn eech_gv_impact(motion_vector: *mut f32, speed: f32, xv: *const f32);
    fn eech_ship_sink(
        position: *const f32,
        motion_vector: *const f32,
        attitude: *const f32,
        delta_seconds: f32,
        out_position: *mut f32,
        out_motion_vector: *mut f32,
        out_speed: *mut f32,
        out_attitude: *mut f32,
    );
    fn eech_fw_crash_candidate(
        position: *const f32,
        motion_vector: *const f32,
        delta_seconds: f32,
        map_min: *const f32,
        map_max: *const f32,
        candidate: *mut f32,
    );
    fn eech_fw_crash_velocity(
        position: *const f32,
        candidate: *mut f32,
        motion_vector: *mut f32,
        face_normal: *const f32,
        forest: c_int,
        elevation: f32,
        cg_to_ground: f32,
        delta_seconds: f32,
        speed_out: *mut f32,
        stopped: *mut c_int,
    );
    fn eech_fw_crash_attitude(
        attitude: *mut f32,
        face_normal: *const f32,
        speed: f32,
        destroyed_pitch_offset: f32,
        destroyed_bank_offset: f32,
        delta_seconds: f32,
        at_rest: *mut c_int,
    );
    fn eech_fw_settle(
        candidate: *mut f32,
        held_position: *const f32,
        motion_vector: *mut f32,
        terrain_class: c_int,
        elevation: f32,
        cg_to_ground: f32,
        contact_altitude: f32,
        bury_limit: f32,
        buried: *mut c_int,
    );
    fn eech_fw_death_attitude(
        attitude: *mut f32,
        motion_vector: *const f32,
        speed: f32,
        altitude: f32,
        draw_made: c_int,
        roll_draw: f64,
        delta_seconds: f32,
    ) -> c_int;
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

/// Another member of the aircraft's group, as `fw_navigator.h` takes it (`eech_fw_nav_other`).
#[repr(C)]
struct FwNavOther {
    position: [f32; 3],
    motion_vector: [f32; 3],
    avoidance_radius: f32,
}

/// The host navigator's input, as `fw_navigator.h` declares it (`eech_fw_nav_input`).
#[repr(C)]
struct FwNavInput {
    group_leader: c_int,
    task_leader: c_int,
    delta_time: f32,
    position: [f32; 3],
    motion_vector: [f32; 3],
    attitude: [f32; 9],
    velocity: f32,
    navigation_velocity: [f32; 3],
    cruise_velocity: f32,
    max_vertical_turn_allowance: f32,
    g_max: f32,
    avoidance_radius: f32,
    waypoint: [f32; 3],
    guide_velocity: f32,
    task_leader_motion_vector: [f32; 3],
    other_count: c_int,
    others: *const FwNavOther,
}

/// The host's turn-around decision input, as `fw_navigator.h` declares it (`eech_fw_turn_input`).
#[repr(C)]
struct FwTurnInput {
    task_leader: c_int,
    aiming_cannon: c_int,
    elapsed_time: f32,
    position: [f32; 3],
    motion_vector: [f32; 3],
    waypoint: [f32; 3],
    leader_position: [f32; 3],
}

/// The turn multiplier and hold-until time the host retains between a fixed-wing aircraft's steps, as the original kept
/// them on the entity (`turn_speed_multi` and `fixed_wing_toggle`). Physical state: the campaign never sends them.
#[derive(Clone, Copy, Debug, PartialEq)]
struct FixedWingTurn {
    turn_speed_multi: f32,
    toggle: f32,
}

impl FixedWingTurn {
    /// the values the original creates an aircraft with
    const CREATED: Self = Self {
        turn_speed_multi: 2.0,
        toggle: 0.0,
    };
}

/// A group member a fixed-wing navigator avoids: the identity of another aircraft the host holds, and the avoidance
/// radius of its type (metadata). Its pose and motion come from the host's own record of it, never from the campaign.
#[derive(Clone, Debug)]
pub struct NavigationMember {
    /// `None`: the campaign's roster lacks the aircraft. The host refuses the step only if it reads the member.
    pub host_id: Option<String>,
    pub avoidance_radius: f32,
}

/// The navigation intent of a fixed-wing flight step. The campaign decides which aircraft matter and in what order; it
/// sends no steering and no pose or motion of any aircraft but its own mirror (which is validated, not used).
#[derive(Clone, Debug)]
pub struct FixedWingNavigation {
    /// the aircraft leads its group: its navigator makes no collision test, and `members` is not read
    pub group_leader: bool,
    /// the aircraft leads its task and follows its waypoint; otherwise it pursues `leader`
    pub task_leader: bool,
    pub max_vertical_turn_allowance: f32,
    pub avoidance_radius: f32,
    /// the task leader the aircraft pursues, if the campaign's roster has it. Required only on a path that reads it
    /// (facing away, or the navigator, when the aircraft is not `task_leader`); never refused for lacking it otherwise.
    pub leader: Option<String>,
    /// the group's other members in the group list's order (this aircraft excluded)
    pub members: Vec<NavigationMember>,
}

/// A flight step's navigator input once the host has resolved the other aircraft from its own records.
struct ResolvedNavigation {
    group_leader: bool,
    task_leader: bool,
    max_vertical_turn_allowance: f32,
    avoidance_radius: f32,
    /// the task leader's host-held position and motion; zero when the aircraft leads its task, or (landing) when the
    /// aircraft is not facing away from its waypoint and the leader is not read
    leader_position: [f32; 3],
    leader_motion_vector: [f32; 3],
    others: Vec<FwNavOther>,
}

/// What the campaign decided and observed about a terrain-following waypoint; the physical climb to it is the host's.
/// The campaign chose the guide position and the height to keep above the terrain (`min_height`, -1 for none), added
/// the formation's height offset and the centre of gravity, and answered the terrain elevation at the point the host
/// asked about ([`Request::FixedWingLookahead`]); the host performs the original climb arithmetic from its own pose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixedWingTerrain {
    pub min_height: f32,
    /// the guide's position, before the terrain adjustment and the formation and centre-of-gravity offsets
    pub guide_position: [f32; 3],
    /// the look-ahead point the campaign sampled the terrain at (an echo, validated bit for bit against the host's own)
    pub sample_point: [f32; 2],
    pub sample_elevation: f32,
    /// the formation site's height added after the adjustment (a task leader that does not lead its group), else 0
    pub site_y: f32,
    /// the centre-of-gravity height added last
    pub cg: f32,
}

/// A fixed-wing aircraft's step. The campaign supplies the guidance (the phase, the waypoint,
/// the guide's speed, the navigation intent of normal flight, the previous motion vector and the class
/// parameters); the host integrates velocity, position and attitude and answers the new motion
/// vector. `delta_seconds` is the engine's step delta and `frame_delta` the frame reciprocal:
/// they are different values.
#[derive(Clone, Debug)]
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
    pub waypoint: [f32; 3],
    /// Campaign policy: a cannon engagement is in range and the aircraft is aiming it (the original's gun-aim test,
    /// which needs the target and the weapon). The host turns this into its multiplier and hold; the campaign does
    /// not send them.
    pub aiming_cannon: bool,
    /// The session's elapsed time, for the hold-until time the host retains.
    pub elapsed_time: f32,
    /// Required in `Flight` and `Land`, where the host navigates and decides whether the aircraft is facing away from
    /// its waypoint (and so turns round in absolute flight); unused by the other phases. A `Land` step reads the
    /// leader only when the host finds the aircraft facing away, so it is not refused for lacking one otherwise.
    /// Likewise a held absolute flight reads neither the leader nor the members.
    pub navigation: Option<FixedWingNavigation>,
    /// The campaign's terrain-follow intent for this waypoint, when its guide follows terrain. The waypoint's height is
    /// then the HOST's: it is recomputed from the retained pose and this terrain observation, and the height sent in
    /// `waypoint` is not read. `None` leaves the waypoint as the campaign sent it.
    pub terrain: Option<FixedWingTerrain>,
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

/// The kind of the waypoint a helicopter's guidance reads, as `hc_guidance.h` classifies it (`EECH_HC_WP_*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelicopterWaypointKind {
    Other = 0,
    /// navigation, CAP loop or CAP start
    Route = 1,
    /// holding, holding loop or taxi
    Holding = 2,
    Landed = 3,
    /// prepare for insertion or troop insertion
    Insertion = 4,
}

/// A group member of a helicopter, named by the identity the campaign's roster holds (`None`: the roster lacks it).
/// Its position comes from the host's own record of it, never from the campaign.
pub type HelicopterMember = Option<String>;

/// Step one of an alive AI helicopter's hosted step: the campaign's guidance intent. The campaign decides the task, the
/// waypoint and the formation, and reports the environment it observes (the terrain under the helicopter, its leader's
/// radar altitude); it sends no steering, no avoidance and no pose or motion of any aircraft but its own mirror (which is
/// validated, not used). The host answers the lookups the guidance needs ([`HelicopterQueryPlan`]).
#[derive(Clone, Debug)]
pub struct HelicopterPlan {
    pub group_leader: bool,
    pub task_leader: bool,
    pub task_leader_is_self: bool,
    pub position_hold: bool,
    pub terrain_follow: bool,
    pub terrain_follow_attack_altitude: bool,
    pub waypoint_kind: HelicopterWaypointKind,
    /// the guide's radius, if it has one
    pub guide_radius: Option<f32>,
    pub guide_altitude_valid: bool,
    pub waypoint: [f32; 3],
    pub waypoint_velocity: [f32; 3],
    /// the distance the campaign's waypoint routine stored
    pub distance: f32,
    /// the terrain cache's elevation, read before any sample of this step
    pub terrain_elevation: f32,
    pub fuel_supply_level: f32,
    pub cruise_velocity: f32,
    pub power_output: f32,
    pub cruise_altitude: f32,
    pub attack_altitude: f32,
    /// The task leader's radar altitude: transitional terrain data (height above the terrain the campaign's terrain
    /// service measured), NOT host-owned physical state; consumed only when the helicopter follows another aircraft.
    pub leader_radar_altitude: f32,
    pub map_min: [f32; 3],
    pub map_max: [f32; 3],
    pub delta_seconds: f32,
    /// the group's other members in the group list's order (this aircraft excluded). Read only when the aircraft does
    /// not lead its group; no member is dropped or refused for the count.
    pub members: Vec<HelicopterMember>,
    /// the DLL's mirror of the aircraft's motion vector, validated bit for bit like the pose
    pub motion_vector: [f32; 3],
}

/// The lookups the campaign's transitional services perform for the host's guidance, as the host planned them from the
/// step's inputs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HelicopterQueryPlan {
    /// enumerate the structures of the AIRCRAFT'S OWN sector, in the sector list's order and without a cap, as the
    /// original scans them. `structure_position` is only the look-ahead point the host tests those structures against
    /// (the engine's y is 0); it is not the sector to enumerate.
    pub structure_scan: bool,
    pub structure_position: [f32; 3],
    /// how many terrain samples to take (at most two), in this order (far to near)
    pub terrain_sample_count: usize,
    pub terrain_sample: [[f32; 2]; 2],
}

/// A structure the campaign's sector enumeration found. The host decides which matter and how they steer the aircraft.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HelicopterStructure {
    pub position: [f32; 3],
    pub avoidance_radius: f32,
    pub height: f32,
}

/// One terrain sample as the original statement
/// `terrain_elevation = max (terrain_elevation, get_3d_terrain_point_data (x, z, &terrain_info))` took it. The `max` macro
/// evaluates the getter once, and a second time when the running maximum does not exceed that first return, selecting the
/// SECOND return. The DLL runs that statement literally and records each getter return in call order; the host folds them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HelicopterTerrainSample {
    /// the getter's first return
    pub first: f32,
    /// the getter's second return: present exactly when the running maximum did not exceed `first`
    pub second: Option<f32>,
}

/// Step two: the answers to the [`HelicopterQueryPlan`]. The terrain samples are taken in the plan's order, far to near,
/// each with the original `max` statement on a running maximum that starts at 0.0; every getter call writes `terrain_info`
/// and the last call is at the aircraft. Each sample carries every getter return the DLL obtained, and the host folds them
/// to the running maximum (a sample whose pattern does not match the host's fold is refused with the plan retained). The
/// structures are of the aircraft's own sector, in sector order. The host runs its guidance and retains the answer for
/// the phases that follow.
#[derive(Clone, Debug)]
pub struct HelicopterGuide {
    pub structures: Vec<HelicopterStructure>,
    pub terrain_samples: Vec<HelicopterTerrainSample>,
    /// the DLL's mirror of the aircraft's motion vector, validated bit for bit like the pose
    pub motion_vector: [f32; 3],
}

/// Phase two of a hosted helicopter step, after the campaign's rotor and early gates: the campaign's steering intent.
/// The host computes the desired heading, pitch and roll from the guidance it retained, the state phase one left and
/// these facts, then runs the end integration. Nothing here is a desired steering value.
#[derive(Clone, Debug)]
pub struct HelicopterSteer {
    pub delta_seconds: f32,
    pub max_turn_rate: f32,
    pub elevation: f32,
    pub cg_to_ground: f32,
    pub terrain_follow: bool,
    pub map_min: [f32; 3],
    pub map_max: [f32; 3],
    /// the waypoint kind the heading reads (the helicopter's current waypoint, which can differ from the guide's)
    pub waypoint_kind: HelicopterWaypointKind,
    pub task_leader_is_self: bool,
    pub guide_radius: Option<f32>,
    /// the heading the campaign requires, if it requires one
    pub required_heading: Option<f32>,
    /// The task leader, by host id. Its heading is the heading of the host's own record of its attitude, resolved only on
    /// the path that reads it (a follower close to its leader with no required heading and little speed); `None` or an
    /// id nothing on that path uses is never looked at. The campaign sends no heading.
    pub leader: Option<String>,
    pub targeting_active: bool,
    pub weapon_vector: [f32; 3],
    pub weapon_to_intercept_vector: [f32; 3],
    pub g_max: f32,
    pub main_rotor_shaft_angle: f32,
    /// the DLL's mirror of the motion vector phase one left, validated bit for bit
    pub motion_vector: [f32; 3],
}

/// `eech_hc_structure` in `hc_guidance.h`.
#[repr(C)]
#[derive(Clone, Copy)]
struct HcStructure {
    position: [f32; 3],
    avoidance_radius: f32,
    height: f32,
}

/// `eech_hc_other` in `hc_guidance.h`.
#[repr(C)]
#[derive(Clone, Copy)]
struct HcOther {
    position: [f32; 3],
}

/// `eech_hc_guidance_input` in `hc_guidance.h`.
#[repr(C)]
struct HcGuidanceInput {
    group_leader: c_int,
    task_leader: c_int,
    task_leader_is_self: c_int,
    position_hold: c_int,
    terrain_follow: c_int,
    terrain_follow_attack_altitude: c_int,
    waypoint_kind: c_int,
    guide_radius_valid: c_int,
    guide_radius: f32,
    guide_altitude_valid: c_int,
    waypoint: [f32; 3],
    waypoint_velocity: [f32; 3],
    distance: f32,
    position: [f32; 3],
    motion_vector: [f32; 3],
    attitude: [f32; 9],
    velocity: f32,
    terrain_elevation: f32,
    fuel_supply_level: f32,
    cruise_velocity: f32,
    power_output: f32,
    cruise_altitude: f32,
    attack_altitude: f32,
    leader_radar_altitude: f32,
    map_min: [f32; 3],
    map_max: [f32; 3],
    delta_time: f32,
    other_count: c_int,
    others: *const HcOther,
}

/// `eech_hc_query_plan` in `hc_guidance.h`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct HcQueryPlan {
    structure_scan: c_int,
    structure_position: [f32; 3],
    terrain_sample_count: c_int,
    terrain_sample: [[f32; 2]; 2],
}

/// `eech_hc_guidance_result` in `hc_guidance.h`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct HcGuidanceResult {
    world_acceleration: [f32; 3],
    wp_vec: [f32; 3],
    lift: f32,
    waypoint: [f32; 3],
    required_height: f32,
}

/// `eech_hc_demand_input` in `hc_guidance.h`.
#[repr(C)]
struct HcDemandInput {
    waypoint_kind: c_int,
    task_leader_is_self: c_int,
    guide_radius_valid: c_int,
    guide_radius: f32,
    distance: f32,
    required_heading_valid: c_int,
    required_heading: f32,
    leader_attitude_valid: c_int,
    leader_attitude: [f32; 9],
    targeting_active: c_int,
    weapon_vector: [f32; 3],
    weapon_to_intercept_vector: [f32; 3],
    attitude: [f32; 9],
    motion_vector: [f32; 3],
    velocity: f32,
    cruise_velocity: f32,
    g_max: f32,
    main_rotor_shaft_angle: f32,
    world_acceleration: [f32; 3],
    wp_vec: [f32; 3],
}

/// `eech_hc_demand_result` in `hc_guidance.h`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct HcDemandResult {
    desired_heading: f32,
    desired_pitch: f32,
    desired_roll: f32,
}

extern "C" {
    fn eech_hc_guidance_plan(input: *const HcGuidanceInput, plan: *mut HcQueryPlan);
    fn eech_hc_guidance(
        input: *const HcGuidanceInput,
        structures: *const HcStructure,
        structure_count: c_int,
        sampled_terrain_elevation: f32,
        result: *mut HcGuidanceResult,
    );
    /// 0: the demands are in `result`; `HC_DEMAND_NEEDS_LEADER`: the heading follows the leader and no leader attitude was given
    fn eech_hc_demand(input: *const HcDemandInput, result: *mut HcDemandResult) -> c_int;
}

/// `EECH_HC_DEMAND_NEEDS_LEADER` in `hc_guidance.h`.
const HC_DEMAND_NEEDS_LEADER: c_int = 1;

/// A hosted helicopter step's retained guidance, in the order the phases consume it. Physical state: the campaign never
/// sends it and never supplies an answer in its place. Any request outside the step's phases drops it, a new plan
/// replaces it, and a gated rotor ends it.
enum HelicopterFlow {
    /// the plan was answered; the lookups are owed
    Planned {
        plan: HelicopterPlan,
        others: Vec<HcOther>,
        query: HcQueryPlan,
    },
    /// the host's guidance ran. `rotor_checked`: the rotor step confirmed the lift and did not gate the helicopter.
    Guided {
        plan: HelicopterPlan,
        result: HcGuidanceResult,
        rotor_checked: bool,
    },
    /// phase one integrated with the retained acceleration and climb; the steering is owed
    Begun { plan: HelicopterPlan, result: HcGuidanceResult },
}

fn flag(value: bool) -> c_int {
    c_int::from(value)
}

/// The guidance input as the host builds it: the self state is the host's own record, the intent is the campaign's.
fn hc_guidance_input(pose: &Pose, motion_vector: [f32; 3], plan: &HelicopterPlan, others: &[HcOther]) -> HcGuidanceInput {
    HcGuidanceInput {
        group_leader: flag(plan.group_leader),
        task_leader: flag(plan.task_leader),
        task_leader_is_self: flag(plan.task_leader_is_self),
        position_hold: flag(plan.position_hold),
        terrain_follow: flag(plan.terrain_follow),
        terrain_follow_attack_altitude: flag(plan.terrain_follow_attack_altitude),
        waypoint_kind: plan.waypoint_kind as c_int,
        guide_radius_valid: flag(plan.guide_radius.is_some()),
        guide_radius: plan.guide_radius.unwrap_or(0.0),
        guide_altitude_valid: flag(plan.guide_altitude_valid),
        waypoint: plan.waypoint,
        waypoint_velocity: plan.waypoint_velocity,
        distance: plan.distance,
        position: pose.position,
        motion_vector,
        attitude: pose.attitude,
        velocity: pose.velocity,
        terrain_elevation: plan.terrain_elevation,
        fuel_supply_level: plan.fuel_supply_level,
        cruise_velocity: plan.cruise_velocity,
        power_output: plan.power_output,
        cruise_altitude: plan.cruise_altitude,
        attack_altitude: plan.attack_altitude,
        leader_radar_altitude: plan.leader_radar_altitude,
        map_min: plan.map_min,
        map_max: plan.map_max,
        delta_time: plan.delta_seconds,
        other_count: others.len() as c_int,
        others: others.as_ptr(),
    }
}

fn hc_plan_is_finite(plan: &HelicopterPlan) -> bool {
    plan.waypoint
        .iter()
        .chain(&plan.waypoint_velocity)
        .chain(&plan.map_min)
        .chain(&plan.map_max)
        .chain(plan.guide_radius.iter())
        .chain([
            &plan.distance,
            &plan.terrain_elevation,
            &plan.fuel_supply_level,
            &plan.cruise_velocity,
            &plan.power_output,
            &plan.cruise_altitude,
            &plan.attack_altitude,
            &plan.leader_radar_altitude,
            &plan.delta_seconds,
        ])
        .all(|value| value.is_finite())
}

fn hc_steer_is_finite(steer: &HelicopterSteer) -> bool {
    steer
        .map_min
        .iter()
        .chain(&steer.map_max)
        .chain(&steer.weapon_vector)
        .chain(&steer.weapon_to_intercept_vector)
        .chain(steer.guide_radius.iter())
        .chain(steer.required_heading.iter())
        .chain([
            &steer.delta_seconds,
            &steer.max_turn_rate,
            &steer.elevation,
            &steer.cg_to_ground,
            &steer.g_max,
            &steer.main_rotor_shaft_angle,
        ])
        .all(|value| value.is_finite())
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
    /// a dying routed or anti-air vehicle's slide candidate (x and z); the pose is untouched
    GvDeathCandidate,
    /// the slide's decay at the candidate, after the campaign's terrain refresh there
    GvDeathDecay,
    /// a falling routed or anti-air vehicle's candidate position; opens the fall
    GvFallCandidate,
    /// gravity on the vehicle's vector only, after the campaign's terrain refresh at the candidate
    GvFallGravity,
    /// stage the fall's settled position, vector and speed; nothing is applied
    GvFallSettle,
    /// the shove along the attitude x axis of a routed or anti-air wreck
    GvImpact,
    /// a dying ship's sink step
    ShipSink,
    /// the host adopts the vehicle's vector when it retains none; later it must match bit for bit
    VehicleVectorSeed,
    /// a dying fixed-wing aircraft's final attitude; the settled candidate becomes its position
    FwDeathAttitude,
    /// a fixed-wing crash slide's first phase: the candidate position
    FwCrashCandidate,
    /// the crash slide's second phase: the slope's effect on the vector and the speed
    FwCrashVelocity,
    /// the fall's staged vector, applied after the campaign's effects when it changed
    GvFallVector,
    /// the fall's end: the staged scalar and pose
    GvFallCommit,
    /// settle a fixed-wing aircraft's stored candidate on the terrain class
    FwSettle,
    /// the crash slide's last phase: the attitude and the final position
    FwCrashAttitude,
    /// a dead mobile's wreck ages one frame: the host owns the timer and says when the wreck's physical life is over
    /// (a ship's is its sinking out of sight instead)
    Expire,
}

/// `Lifecycle::death_policy`: the mobile never expires (the anti-air gun accumulates its timer and is never removed)
pub const DEATH_NEVER_EXPIRES: i32 = 1;
/// `Lifecycle::death_policy`: a helicopter still occupied by a player is not removed; its timer restarts instead
pub const DEATH_OCCUPIED: i32 = 2;
/// `Lifecycle::death_policy`: `death_range` carries the campaign's squared range to the nearest keysite. Absent, the host
/// answers `EXPIRE_NEEDS_RANGE` on a gated frame and the campaign samples the range only then (the original scanned every
/// keysite only on the gated frame), asking again with this bit set.
pub const DEATH_RANGE_SUPPLIED: i32 = 4;
const DEATH_POLICY_BITS: i32 = DEATH_NEVER_EXPIRES | DEATH_OCCUPIED | DEATH_RANGE_SUPPLIED;
/// the `Expire` answer asking for the keysite range: the gate is open, nothing has been aged or recorded
pub const EXPIRE_NEEDS_RANGE: i32 = 2;

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
            LifecycleMode::GvDeathCandidate => "gv_death_candidate",
            LifecycleMode::GvDeathDecay => "gv_death_decay",
            LifecycleMode::GvFallCandidate => "gv_fall_candidate",
            LifecycleMode::GvFallGravity => "gv_fall_gravity",
            LifecycleMode::GvFallSettle => "gv_fall_settle",
            LifecycleMode::GvImpact => "gv_impact",
            LifecycleMode::ShipSink => "ship_sink",
            LifecycleMode::VehicleVectorSeed => "vehicle_vector_seed",
            LifecycleMode::FwDeathAttitude => "fw_death_attitude",
            LifecycleMode::FwCrashCandidate => "fw_crash_candidate",
            LifecycleMode::FwCrashVelocity => "fw_crash_velocity",
            LifecycleMode::GvFallVector => "gv_fall_vector",
            LifecycleMode::GvFallCommit => "gv_fall_commit",
            LifecycleMode::FwSettle => "fw_settle",
            LifecycleMode::FwCrashAttitude => "fw_crash_attitude",
            LifecycleMode::Expire => "expire",
        }
    }

    /// Whether a mobile of `kind` may make this step. The rotor, tumble, settle and crash modes are a helicopter's;
    /// fall, gravity, contact and impact are shared by both aircraft; the routed and anti-air vehicles' wreck and fall
    /// steps are `Routed`'s (the anti-air gun is bound as a routed vehicle), the sink step is a ship's and the
    /// vector seed is the one step both vehicle kinds make.
    pub fn allows(self, kind: Kind) -> bool {
        match self {
            LifecycleMode::RotorSet
            | LifecycleMode::RotorAlive
            | LifecycleMode::RotorDown
            | LifecycleMode::RotorIdle
            | LifecycleMode::Settle
            | LifecycleMode::Tumble
            | LifecycleMode::CrashBegin
            | LifecycleMode::CrashEnd => kind == Kind::Helicopter,
            LifecycleMode::Fall | LifecycleMode::FallGravity | LifecycleMode::Contact | LifecycleMode::Impact => {
                matches!(kind, Kind::Helicopter | Kind::FixedWing)
            }
            LifecycleMode::FwDeathAttitude
            | LifecycleMode::FwCrashCandidate
            | LifecycleMode::FwCrashVelocity
            | LifecycleMode::FwSettle
            | LifecycleMode::FwCrashAttitude => kind == Kind::FixedWing,
            LifecycleMode::GvDeathCandidate
            | LifecycleMode::GvDeathDecay
            | LifecycleMode::GvFallCandidate
            | LifecycleMode::GvFallGravity
            | LifecycleMode::GvFallSettle
            | LifecycleMode::GvImpact
            | LifecycleMode::GvFallVector
            | LifecycleMode::GvFallCommit => kind == Kind::Routed,
            LifecycleMode::ShipSink => kind == Kind::Ship,
            LifecycleMode::VehicleVectorSeed => matches!(kind, Kind::Routed | Kind::Ship),
            LifecycleMode::Expire => true,
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
    /// `GvFallSettle`: the terrain elevation before the fall's refresh, for the splash test
    pub old_elevation: f32,
    /// `GvFallSettle`: the vehicle's bounding box diagonal, for the sunk test
    pub bbox_diagonal: f32,
    /// `GvDeathDecay`: the anti-air gun's double-precision decay literal
    pub wide: bool,
    /// `GvFallSettle`: the terrain under the candidate is water
    pub water: bool,
    /// The DLL's mirror of a staged candidate (`GvDeathDecay`: x and z; `GvFallSettle`: x, y and z), validated
    /// bit for bit against the one the host retained
    pub waypoint: [f32; 3],
    /// `Expire`: the campaign's mirror of the wreck's timer, adopted while the host retains none, then validated bit for bit
    pub death_timer: f32,
    /// `Expire`: the squared range to the nearest keysite, which is campaign state; the host owns the curve over it
    pub death_range: f32,
    /// `Expire`: `DEATH_NEVER_EXPIRES` | `DEATH_OCCUPIED`
    pub death_policy: i32,
}

/// The campaign's placement intent for a mobile it has just built, and the physical facts the host needs to place it:
/// the formation's base point (x and z are the campaign's decision; y is the terrain height the campaign sampled from
/// the shared terrain service, or the formation's own height where no terrain is followed), the class's
/// centre-of-gravity offset above that base, which way it faces, the terrain face under it and the airframe's fuselage
/// angle. The host computes the physical height and attitude, owns the state this produces and answers it.
#[derive(Clone, Copy, Debug)]
pub struct Creation {
    pub position: [f32; 3],
    /// the physical offset the host adds to the base height (zero where the original added none)
    pub cg_to_ground: f32,
    pub heading: f32,
    pub fuselage_angle: f32,
    pub face_normal: [f32; 3],
    /// the DLL's motion vector, adopted only by an aircraft record (as `Request::Place`)
    pub motion_vector: [f32; 3],
}

#[derive(Clone, Debug)]
pub enum Request {
    /// the campaign decided a new mobile's placement: the host computes its initial attitude and adopts the pose, once
    Create(Creation),
    /// a rotor, dying, wreck, fall, sink, crash or impact step of the kinds `LifecycleMode::allows` names
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
    /// where the host wants the terrain sampled for a fixed-wing aircraft's terrain-following waypoint (its look-ahead
    /// point, from the retained pose); the host answers [`HelicopterAnswer::Query`] with one sample and changes nothing
    FixedWingLookahead,
    /// an alive AI helicopter's step, phase one
    HelicopterBegin(HelicopterBegin),
    /// an alive AI helicopter's step, phase two
    HelicopterEnd(HelicopterEnd),
    /// a hosted helicopter step, phase 0: the campaign's guidance intent; the host answers [`HelicopterAnswer::Query`]
    HelicopterPlan(Box<HelicopterPlan>),
    /// a hosted helicopter step, phase 1: the answers to the query; the host runs its guidance and answers [`HelicopterAnswer::Guidance`]
    HelicopterGuide(HelicopterGuide),
    /// a hosted helicopter step, phase 3: integrate with the retained acceleration and climb. `motion_vector` is the
    /// DLL's mirror, validated bit for bit.
    HelicopterGuidedBegin { motion_vector: [f32; 3] },
    /// a hosted helicopter step, phase 4: the host computes the steering demand and integrates
    HelicopterGuidedEnd(HelicopterSteer),
    /// the campaign's early gate (human wait, landed) ended the hosted step before its begin: drop the retained guidance
    HelicopterAbandon,
    /// the campaign placed the mobile: adopt its pose. `motion_vector` is the DLL's motion
    /// vector, adopted only by an aircraft record (declared transitional coupling: the DLL
    /// still writes an aircraft's vector outside a host step).
    Place { motion_vector: [f32; 3] },
    /// the campaign's parent moved and notified this attached child (membership is the campaign's): the host positions the
    /// child by the parent's displacement, as the original notification handler did, and answers the child's new pose
    DependentCarry(DependentCarry),
}

/// How an attached child follows its moving parent; fixed by the child's class in the original handlers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CarryShape {
    /// the routed vehicle, anti-aircraft gun and infantryman take the parent's new position and nothing else
    Position,
    /// the helicopter keeps its offset from the parent, rotated by the parent's heading change, and takes the parent's
    /// heading and speed
    Helicopter,
}

/// The parent's move as the campaign's notification carried it: the host identity of the parent, the position it had
/// (its stored one, as the notification went out before the write) and is given, and the heading and speed the original
/// message carried. The host validates both positions against its own record of the parent.
#[derive(Clone, Debug)]
pub struct DependentCarry {
    pub parent: String,
    pub shape: CarryShape,
    pub parent_old: [f32; 3],
    pub parent_new: [f32; 3],
    pub heading: f32,
    pub velocity: f32,
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
            Request::FixedWingLookahead => "fixed_wing_lookahead",
            Request::HelicopterBegin(_) => "helicopter_begin",
            Request::HelicopterEnd(_) => "helicopter_end",
            Request::HelicopterPlan(_) => "helicopter_plan",
            Request::HelicopterGuide(_) => "helicopter_guide",
            Request::HelicopterGuidedBegin { .. } => "helicopter_guided_begin",
            Request::HelicopterGuidedEnd(_) => "helicopter_guided_end",
            Request::HelicopterAbandon => "helicopter_abandon",
            Request::Lifecycle(step) => step.mode.name(),
            Request::Place { .. } => "place",
            Request::Create(_) => "create_place",
            Request::DependentCarry(_) => "dependent_carry",
        }
    }

    /// Whether a mobile of `kind` may make this request; any kind may receive a placement.
    fn allows(&self, kind: Kind) -> bool {
        match self {
            Request::Advance(_) | Request::Stop => kind == Kind::Routed,
            Request::ShipMove(_) => kind == Kind::Ship,
            Request::PersonBegin(_) | Request::PersonEnd(_) => kind == Kind::Person,
            Request::FixedWingMove(_) | Request::FixedWingLookahead => kind == Kind::FixedWing,
            Request::HelicopterBegin(_)
            | Request::HelicopterEnd(_)
            | Request::HelicopterPlan(_)
            | Request::HelicopterGuide(_)
            | Request::HelicopterGuidedBegin { .. }
            | Request::HelicopterGuidedEnd(_)
            | Request::HelicopterAbandon => kind == Kind::Helicopter,
            Request::Lifecycle(step) => step.mode.allows(kind),
            Request::Place { .. } | Request::Create(_) => true,
            Request::DependentCarry(carry) => match carry.shape {
                CarryShape::Position => matches!(kind, Kind::Routed | Kind::Person),
                CarryShape::Helicopter => kind == Kind::Helicopter,
            },
        }
    }
}

/// The host's guidance answer to a hosted helicopter step's plan or guide request, read with [`Movers::take_answer`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HelicopterAnswer {
    /// the lookups the campaign's transitional services owe the host
    Query(HelicopterQueryPlan),
    /// The host's guidance. `lift` is what the campaign's rotor step must present (it is validated against it); the
    /// acceleration and climb demand stay with the host for the begin phase and are reported for evidence only.
    Guidance(HelicopterGuidance),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HelicopterGuidance {
    pub lift: f32,
    pub waypoint: [f32; 3],
    pub required_height: f32,
    pub world_acceleration: [f32; 3],
    pub wp_vec: [f32; 3],
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
    /// The mode's result bits (see the engine's `result_flags_are_valid`): `GvFallSettle` 1 contact, 2 splash, 4 sunk,
    /// 8 ground impact; `FwDeathAttitude` -1 gate mismatch (nothing mutated), 1 eased, 3 eased with the roll draw
    /// consumed; `FwCrashVelocity` 1 stopped; `FwCrashAttitude` 1 at rest. Zero for every other request.
    pub flags: i32,
    /// `Create`: the heading, pitch and roll of the attitude the host built, for the campaign's angle setter; zero otherwise
    pub angles: [f32; 3],
    /// `Expire`: the wreck's timer after this frame, for the campaign's copy (its drawing reads it); zero otherwise.
    /// The expiry itself is `flags` bit 0.
    pub death_timer: f32,
}

impl Outcome {
    /// The empty answer to a retirement: the mobile no longer has a pose, so nothing here is applied.
    pub fn retired() -> Self {
        settled(
            Pose {
                position: [0.0; 3],
                velocity: 0.0,
                attitude: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            },
            false,
            [0.0; 3],
        )
    }
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
    /// The host built this mobile's initial pose (a `Create` request); a mobile is created once.
    placed: bool,
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
    /// A routed, anti-air or ship wreck's vector as the host holds it, which is also the vector the campaign's copy
    /// holds. Adopted by the seed only when absent, and then validated bit for bit; live movement (a step that
    /// legitimately changes the vector) releases it so the next wreck or fall establishes the current one. A placement
    /// never touches it.
    vehicle_vector: Option<[f32; 3]>,
    /// A dying vehicle's slide candidate (x and z), between its candidate and its decay.
    death_candidate: Option<[f32; 2]>,
    /// A falling routed or anti-air vehicle's open fall, from its candidate to its commit.
    fall: Option<GroundFall>,
    /// A dying fixed-wing aircraft's post-gravity, pre-bury speed, which its death attitude is verified against.
    fw_speed: Option<f32>,
    /// A fixed-wing crash slide between its candidate and its attitude. The candidate itself is `candidate`.
    fw_crash: Option<CrashStage>,
    /// An alive fixed-wing aircraft's turn multiplier and hold-until time; `None` for other kinds.
    fw_turn: Option<FixedWingTurn>,
    /// A dead wreck's age as the host holds it, adopted from the campaign's copy by the first `Expire` and validated bit
    /// for bit afterwards; `None` for a ship, which has no timer.
    death_timer: Option<f32>,
    /// A hosted helicopter step's retained guidance between its phases; `None` outside a hosted step.
    hc_flow: Option<HelicopterFlow>,
    /// The position this mobile had before the last request the host accepted for it. A parent the host itself moved is
    /// already at its new position when its children are notified, so this is the old one they are carried from.
    previous_position: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FallStage {
    /// the candidate position is retained
    Candidate,
    /// gravity changed the vehicle's vector
    Gravity,
    /// the settled position, vector, speed and flags are staged; nothing is applied
    Settled,
    /// the staged vector was applied after the campaign's effects
    Vectored,
}

/// A falling vehicle's retained state. The prior scalar speed is the pose's `velocity`, which only the commit changes, so
/// the campaign's effects see the original scalar while gravity has changed the vector.
#[derive(Clone, Copy, Debug)]
struct GroundFall {
    stage: FallStage,
    candidate: [f32; 3],
    staged_vector: [f32; 3],
    staged_position: [f32; 3],
    staged_speed: f32,
}

#[derive(Clone, Copy, Debug)]
enum CrashStage {
    Candidate,
    /// the slope's phase ran; the post-slope speed is kept for the at-rest test even when the stop cleared the scalar
    Velocity {
        speed: f32,
    },
}

/// The host's registry of the physical mobiles it issued identities to. An id is live (with a record, or issued by a link
/// that made no creation request) until the campaign reports the physical incarnation freed, then retired for good: the host
/// never answers a retired id and never issues it again.
#[derive(Default)]
pub struct Movers {
    mobiles: HashMap<String, Record>,
    /// ids issued to a member that has no movement record (a link without a creation request)
    unrecorded: HashSet<String>,
    retired: HashSet<String>,
    answer: Option<HelicopterAnswer>,
}

impl Movers {
    /// Whether `host_id` may still be issued or bound: refused when live or retired.
    fn ensure_fresh(&self, host_id: &str) -> Result<(), String> {
        if self.retired.contains(host_id) {
            return Err(format!("host.bind_mobile: host id {host_id} was retired and cannot be reused"));
        }
        if self.mobiles.contains_key(host_id) || self.unrecorded.contains(host_id) {
            return Err(format!("host.bind_mobile: host id {host_id} is already issued to a live mobile"));
        }
        Ok(())
    }

    /// Register an identity issued to a member that makes no movement record.
    pub fn issue(&mut self, host_id: &str) -> Result<(), String> {
        self.ensure_fresh(host_id)?;
        self.unrecorded.insert(host_id.to_owned());
        Ok(())
    }

    /// The physical incarnation ended (the entity was released): drop the record and any open step with it, and refuse the id
    /// from now on. Distinct from an expiry the host computes; this is the campaign's report that the heap slot is gone.
    pub fn retire(&mut self, host_id: &str) -> Result<(), String> {
        if self.retired.contains(host_id) {
            return Err(format!("host.retire: host id {host_id} was already retired"));
        }
        if self.mobiles.remove(host_id).is_none() && !self.unrecorded.remove(host_id) {
            return Err(format!("host.retire: unknown host id {host_id}"));
        }
        self.retired.insert(host_id.to_owned());
        Ok(())
    }

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
        self.ensure_fresh(host_id)?;
        self.mobiles.insert(
            host_id.to_owned(),
            Record {
                kind,
                pose,
                placed: false,
                motion_vector,
                pending: None,
                helicopter_pending: false,
                rotor,
                candidate: None,
                contact_altitude: None,
                crash_pending: false,
                vehicle_vector: None,
                death_candidate: None,
                fall: None,
                fw_speed: None,
                fw_crash: None,
                fw_turn: (kind == Kind::FixedWing).then_some(FixedWingTurn::CREATED),
                death_timer: None,
                hc_flow: None,
                previous_position: pose.position,
            },
        );
        Ok(())
    }

    /// The guidance answer the last [`Movers::step`] produced (a plan or guide request), if any; taken once.
    pub fn take_answer(&mut self) -> Option<HelicopterAnswer> {
        self.answer.take()
    }

    /// Run one request. `campaign` is the campaign's copy of the pose; the
    /// outcome is the host's record afterwards.
    pub fn step(&mut self, host_id: &str, campaign: Pose, request: Request) -> Result<Outcome, String> {
        self.answer = None;
        let before = self.mobiles.get(host_id).map(|record| record.pose.position);
        let mut outcome = self.dispatch(host_id, campaign, request)?;
        if let Some(record) = self.mobiles.get_mut(host_id) {
            if let Some(before) = before {
                record.previous_position = before;
            }
            if let Some(rotor) = record.rotor {
                outcome.rotor = rotor;
            }
        }
        Ok(outcome)
    }

    /// The parent of an attached child's carry, resolved from the host's own record and validated; `None` for any other
    /// request. Read-only and before the mutable borrow: a refusal changes nothing. The parent must be a live mobile the
    /// host holds, and not the child itself. Its old position is either its record's (the campaign is about to write the
    /// parent and place it afterwards, so the host still holds the position the notification came from) or the one before
    /// the host's own last step (the host already moved it, so its record is the new position, which must then match too).
    fn resolve_dependent_parent(&self, host_id: &str, request: &Request) -> Result<(), String> {
        let Request::DependentCarry(carry) = request else { return Ok(()) };
        let refuse = |reason: &str| format!("host.move_vehicle: dependent_carry for {host_id}: parent {} {reason}", carry.parent);
        let numbers = carry.parent_old.iter().chain(&carry.parent_new).chain([&carry.heading, &carry.velocity]);
        if numbers.into_iter().any(|value| !value.is_finite()) {
            return Err(format!("host.move_vehicle: dependent_carry for {host_id} has a non-finite input"));
        }
        if carry.parent == host_id {
            return Err(refuse("is the child itself"));
        }
        if self.retired.contains(&carry.parent) {
            return Err(refuse("is retired"));
        }
        let parent = self
            .mobiles
            .get(&carry.parent)
            .ok_or_else(|| refuse("is not a host id with a movement record"))?;
        let same = |a: &[f32; 3], b: &[f32; 3]| a.iter().zip(b).all(|(a, b)| a.to_bits() == b.to_bits());
        let before_the_write = same(&carry.parent_old, &parent.pose.position);
        let after_the_step = same(&carry.parent_old, &parent.previous_position) && same(&carry.parent_new, &parent.pose.position);
        if !before_the_write && !after_the_step {
            return Err(refuse(&format!(
                "was carried from {:?} to {:?}, but the host holds it at {:?} (from {:?}): a move the host did not make",
                carry.parent_old, carry.parent_new, parent.pose.position, parent.previous_position
            )));
        }
        Ok(())
    }

    /// The pose and retained motion of another fixed-wing aircraft, by the host's record of it.
    fn navigation_aircraft(&self, host_id: &str, other: &str, role: &str) -> Result<([f32; 3], [f32; 3]), String> {
        if other == host_id {
            return Err(format!("host.move_vehicle: fixed_wing_move for {host_id} names itself as its {role}"));
        }
        if self.retired.contains(other) {
            return Err(format!("host.move_vehicle: fixed_wing_move for {host_id}: its {role} {other} is retired"));
        }
        let record = self
            .mobiles
            .get(other)
            .ok_or_else(|| format!("host.move_vehicle: fixed_wing_move for {host_id}: its {role} {other} is not a host id"))?;
        if record.kind != Kind::FixedWing {
            return Err(format!(
                "host.move_vehicle: fixed_wing_move for {host_id}: its {role} {other} is {:?}, not a fixed-wing aircraft",
                record.kind
            ));
        }
        let motion_vector = record
            .motion_vector
            .ok_or_else(|| format!("host.move_vehicle: fixed_wing_move for {host_id}: its {role} {other} has no retained motion vector"))?;
        Ok((record.pose.position, motion_vector))
    }

    /// The navigator's other aircraft for a normal-flight step, resolved from host records and validated, or `None`
    /// for any other request. Nothing is mutated; every failure is a contract error naming the aircraft.
    fn resolve_navigation(&self, host_id: &str, request: &Request) -> Result<Option<ResolvedNavigation>, String> {
        let Request::FixedWingMove(step) = request else { return Ok(None) };
        if !matches!(step.phase, FixedWingPhase::Flight | FixedWingPhase::Land) {
            return Ok(None);
        }
        let phase = if step.phase == FixedWingPhase::Flight { "flight" } else { "landing" };
        // an unknown, retired or non-fixed-wing subject is reported by the ordinary checks
        if !self.mobiles.get(host_id).is_some_and(|record| record.kind == Kind::FixedWing) {
            return Ok(None);
        }
        let navigation = step
            .navigation
            .as_ref()
            .ok_or_else(|| format!("host.move_vehicle: fixed_wing_move {phase} for {host_id} carries no navigation"))?;
        let numbers = step.waypoint.iter().chain([
            &step.delta_seconds,
            &step.elapsed_time,
            &step.wp_vel,
            &step.cruise_velocity,
            &step.g_max,
            &navigation.max_vertical_turn_allowance,
            &navigation.avoidance_radius,
        ]);
        if numbers.into_iter().any(|value| !value.is_finite()) {
            return Err(format!(
                "host.move_vehicle: fixed_wing_move {phase} for {host_id} has a non-finite navigation input"
            ));
        }
        // The campaign names the leader and the members it knows; the host asks for one only on the path that reads it,
        // so an aircraft is never refused for an identity the original path did not look at. The turn decision is the
        // host's: facing away (the one place the original measured the leader's distance) reads the leader, and a
        // step the turn leaves to the navigator reads the leader it pursues and the members it avoids. A cannon or
        // turn-around hold in force, a landing facing forward, a task leader and a group leader read less.
        let record = self
            .mobiles
            .get(host_id)
            .ok_or_else(|| format!("host.move_vehicle: unknown host id {host_id}"))?;
        let read_leader = || {
            let leader = navigation
                .leader
                .as_deref()
                .ok_or_else(|| format!("host.move_vehicle: fixed_wing_move {phase} for {host_id} names no task leader"))?;
            self.navigation_aircraft(host_id, leader, "task leader")
        };
        let mut leader_state = ([0.0f32; 3], [0.0f32; 3]);
        let mut others = Vec::new();
        // a record without its retained vector or turn state is refused by the ordinary checks before anything is read
        if let (Some(held), Some(mut turn)) = (record.motion_vector, record.fw_turn) {
            if !navigation.task_leader && eech_facing_away(&record.pose.position, &held, &step.waypoint) {
                leader_state = read_leader()?;
            }
            // the same decision the step takes, on a copy: the retained multiplier and hold change only with the step
            let absolute = turn_is_absolute(step, record.pose.position, held, navigation.task_leader, leader_state.0, &mut turn);
            if step.phase == FixedWingPhase::Flight && !absolute {
                if !navigation.task_leader {
                    leader_state = read_leader()?;
                }
                // a group leader makes no collision test, so the members are not read. The group is as large as the
                // campaign sent: no member is dropped or refused for the count
                if !navigation.group_leader {
                    for (index, member) in navigation.members.iter().enumerate() {
                        let name = member
                            .host_id
                            .as_deref()
                            .ok_or_else(|| format!("host.move_vehicle: fixed_wing_move {phase} for {host_id}: group member #{index} names no host id"))?;
                        if !member.avoidance_radius.is_finite() {
                            return Err(format!(
                                "host.move_vehicle: fixed_wing_move {phase} for {host_id}: group member {name} has a non-finite avoidance radius"
                            ));
                        }
                        let (position, motion_vector) = self.navigation_aircraft(host_id, name, "group member")?;
                        others.push(FwNavOther {
                            position,
                            motion_vector,
                            avoidance_radius: member.avoidance_radius,
                        });
                    }
                }
            }
        }
        let (leader_position, leader_motion_vector) = leader_state;
        Ok(Some(ResolvedNavigation {
            group_leader: navigation.group_leader,
            task_leader: navigation.task_leader,
            max_vertical_turn_allowance: navigation.max_vertical_turn_allowance,
            avoidance_radius: navigation.avoidance_radius,
            leader_position,
            leader_motion_vector,
            others,
        }))
    }

    fn dispatch(&mut self, host_id: &str, campaign: Pose, mut request: Request) -> Result<Outcome, String> {
        if self.retired.contains(host_id) {
            return Err(format!("host.move_vehicle: {} for retired host id {host_id}", request.name()));
        }
        // read-only, and before the mutable borrow: a terrain-following waypoint's height is the host's, so the turn,
        // navigation and step below all see it
        self.shape_fixed_wing_waypoint(host_id, &mut request)?;
        // read-only, and before the mutable borrow: the other aircraft are resolved from the host's own records
        let navigation = self.resolve_navigation(host_id, &request)?;
        let hc_others = self.resolve_helicopter_others(host_id, &request)?;
        self.resolve_dependent_parent(host_id, &request)?;
        // the leader's attitude, looked up now and refused only if the heading demand actually reads it
        let hc_leader = self.look_up_helicopter_leader(host_id, &request);
        let record = self
            .mobiles
            .get_mut(host_id)
            .ok_or_else(|| format!("host.move_vehicle: unknown host id {host_id}"))?;
        if !request.allows(record.kind) {
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
            (Request::HelicopterEnd(_) | Request::HelicopterGuidedEnd(_), false) => {
                return Err(format!(
                    "host.move_vehicle: {} for {host_id} without a pending helicopter_begin",
                    request.name()
                ));
            }
            (Request::HelicopterEnd(_) | Request::HelicopterGuidedEnd(_), true) => {}
            (_, true) => {
                return Err(format!(
                    "host.move_vehicle: {} for {host_id} while its helicopter step is pending",
                    request.name()
                ));
            }
            (_, false) => {}
        }
        if !matches!(request, Request::Place { .. } | Request::Lifecycle(_)) {
            let open = if record.fall.is_some() {
                Some("its fall is open")
            } else if record.fw_crash.is_some() {
                Some("its crash slide is open")
            } else if record.death_candidate.is_some() {
                Some("its death slide is pending (gv_death_decay first)")
            } else {
                None
            };
            if let Some(open) = open {
                return Err(format!("host.move_vehicle: {} for {host_id} while {open}", request.name()));
            }
        }
        // Retained guidance lives only through the hosted step's own requests (and the rotor step it feeds); any other
        // request that SUCCEEDS ends it, so a later step can never begin from a stale answer. A request refused for any
        // reason changes nothing: the flow is cleared only once the request has been applied.
        let in_flow = matches!(
            &request,
            Request::HelicopterPlan(_)
                | Request::HelicopterGuide(_)
                | Request::HelicopterGuidedBegin { .. }
                | Request::HelicopterGuidedEnd(_)
                | Request::HelicopterAbandon
        ) || matches!(&request, Request::Lifecycle(step) if step.mode == LifecycleMode::RotorAlive);
        // the legacy begin/end pair and the hosted pair are not mixed within one step
        if let Request::HelicopterEnd(_) = request {
            if matches!(record.hc_flow, Some(HelicopterFlow::Begun { .. })) {
                return Err(format!(
                    "host.move_vehicle: helicopter_end for {host_id} after a hosted begin (helicopter_guided_end)"
                ));
            }
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
        if let Request::Create(creation) = request {
            if record.placed {
                return Err(format!("host.move_vehicle: create_place for {host_id}, which the host already created"));
            }
            let numbers = creation.position.iter().chain(&creation.face_normal).chain(&creation.motion_vector).chain([
                &creation.heading,
                &creation.fuselage_angle,
                &creation.cg_to_ground,
            ]);
            if numbers.into_iter().any(|value| !value.is_finite()) {
                return Err(format!("host.move_vehicle: create_place for {host_id} has a non-finite input"));
            }
            // the physical height: the base plus the centre-of-gravity offset, in single precision as the original added it
            let placed = [creation.position[0], creation.position[1] + creation.cg_to_ground, creation.position[2]];
            if !placed[1].is_finite() {
                return Err(format!(
                    "host.move_vehicle: create_place for {host_id} places the member at a non-finite height"
                ));
            }
            let mut angles = [0.0f32; 3];
            let mut attitude = [0.0f32; 9];
            // SAFETY: both outputs are exactly the sizes the C function writes (three and nine floats); the inputs are read-only
            unsafe {
                eech_place_attitude(
                    creation.face_normal.as_ptr(),
                    creation.heading,
                    creation.fuselage_angle,
                    angles.as_mut_ptr(),
                    attitude.as_mut_ptr(),
                );
            }
            record.pose = Pose {
                position: placed,
                velocity: campaign.velocity,
                attitude,
            };
            if matches!(record.kind, Kind::FixedWing | Kind::Helicopter) {
                record.motion_vector = Some(creation.motion_vector);
            }
            record.placed = true;
            record.hc_flow = None;
            return Ok(Outcome {
                angles,
                ..settled(record.pose, false, [0.0; 3])
            });
        }
        if let Request::Place { motion_vector } = request {
            record.pose = campaign;
            if matches!(record.kind, Kind::FixedWing | Kind::Helicopter) {
                record.motion_vector = Some(motion_vector);
            }
            record.hc_flow = None;
            return Ok(settled(record.pose, false, [0.0; 3]));
        }
        if let Some(what) = campaign.first_difference(&record.pose) {
            return Err(format!(
                "host.move_vehicle: the campaign's {what} for {host_id} differs from the host's record (a write the host did not make)"
            ));
        }
        let result = match request {
            Request::Advance(advance) => {
                release_vehicle_vector(record);
                let rotated = advance_pose(&mut record.pose, &advance);
                Ok(settled(record.pose, rotated, [0.0; 3]))
            }
            Request::Stop => {
                release_vehicle_vector(record);
                record.pose.velocity = 0.0;
                Ok(settled(record.pose, false, [0.0; 3]))
            }
            Request::ShipMove(ship) => {
                release_vehicle_vector(record);
                let motion_vector = ship_pose(&mut record.pose, &ship);
                Ok(settled(record.pose, false, motion_vector))
            }
            Request::FixedWingLookahead => {
                let mut point = [0.0f32; 2];
                // SAFETY: the position and attitude are the 3 and 9 floats the C function reads; the point is the 2 it writes
                unsafe {
                    eech_fw_lookahead(
                        record.pose.position.as_ptr(),
                        record.pose.velocity,
                        record.pose.attitude.as_ptr(),
                        point.as_mut_ptr(),
                    );
                }
                self.answer = Some(HelicopterAnswer::Query(HelicopterQueryPlan {
                    structure_scan: false,
                    structure_position: [0.0; 3],
                    terrain_sample_count: 1,
                    terrain_sample: [point, [0.0; 2]],
                }));
                Ok(settled(record.pose, false, record.motion_vector.unwrap_or([0.0; 3])))
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
                let mut turn = record
                    .fw_turn
                    .ok_or_else(|| format!("host.move_vehicle: fixed-wing {host_id} has no retained turn state"))?;
                let (motion_vector, rotated) = fixed_wing_pose(&mut record.pose, held, &fixed_wing, navigation.as_ref(), &mut turn);
                record.motion_vector = Some(motion_vector);
                record.fw_turn = Some(turn);
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
            Request::HelicopterPlan(plan) => {
                if !hc_plan_is_finite(&plan) {
                    return Err(format!("host.move_vehicle: helicopter_plan for {host_id} has a non-finite input"));
                }
                let held = held_vector(record, host_id, &plan.motion_vector)?;
                let others = hc_others.unwrap_or_default();
                let input = hc_guidance_input(&record.pose, held, &plan, &others);
                let mut query = HcQueryPlan::default();
                // SAFETY: `input` points at `others`, which outlives the call; the plan is one struct the C writes
                unsafe { eech_hc_guidance_plan(&input, &mut query) };
                let answer = HelicopterQueryPlan {
                    structure_scan: query.structure_scan != 0,
                    structure_position: query.structure_position,
                    terrain_sample_count: query.terrain_sample_count.clamp(0, 2) as usize,
                    terrain_sample: query.terrain_sample,
                };
                record.hc_flow = Some(HelicopterFlow::Planned { plan: *plan, others, query });
                self.answer = Some(HelicopterAnswer::Query(answer));
                Ok(settled(record.pose, false, held))
            }
            Request::HelicopterGuide(guide) => {
                // everything is checked against the retained plan before the flow is touched: a refused answer leaves the
                // plan in place for a corrected one, and a repeated guide (no plan left) changes nothing
                let Some(HelicopterFlow::Planned { query, .. }) = &record.hc_flow else {
                    return Err(format!(
                        "host.move_vehicle: helicopter_guide for {host_id} without a planned step (helicopter_plan first)"
                    ));
                };
                let refuse = |reason: String| format!("host.move_vehicle: helicopter_guide for {host_id}: {reason}");
                let held = held_vector(record, host_id, &guide.motion_vector)?;
                let wanted = query.terrain_sample_count.max(0) as usize;
                if guide.terrain_samples.len() != wanted {
                    return Err(refuse(format!("{} terrain samples answered, {wanted} planned", guide.terrain_samples.len())));
                }
                if query.structure_scan == 0 && !guide.structures.is_empty() {
                    return Err(refuse("structures answered to a plan that scans none".to_owned()));
                }
                if guide
                    .structures
                    .iter()
                    .any(|s| s.position.iter().chain([&s.avoidance_radius, &s.height]).any(|v| !v.is_finite()))
                {
                    return Err(refuse("a non-finite structure".to_owned()));
                }
                // the original `terrain_elevation = max (terrain_elevation, get_3d_terrain_point_data (...))`, per sample in
                // the plan's order: the running maximum starts at 0.0; the getter's first return is kept when the running
                // value exceeds it (one call), otherwise the getter ran again and its SECOND return is selected
                let mut sampled = 0.0f32;
                for (index, sample) in guide.terrain_samples.iter().enumerate() {
                    if !sample.first.is_finite() || sample.second.is_some_and(|second| !second.is_finite()) {
                        return Err(refuse(format!("terrain sample #{index} is non-finite")));
                    }
                    if sampled > sample.first {
                        if sample.second.is_some() {
                            return Err(refuse(format!(
                                "terrain sample #{index} carries a second getter return, but the running maximum {sampled} exceeds its first {} (the original calls the getter once)",
                                sample.first
                            )));
                        }
                    } else {
                        sampled = sample.second.ok_or_else(|| {
                            refuse(format!(
                                "terrain sample #{index} carries no second getter return, but the running maximum {sampled} does not exceed its first {} (the original calls the getter again)",
                                sample.first
                            ))
                        })?;
                    }
                }
                let Some(HelicopterFlow::Planned { plan, others, .. }) = record.hc_flow.take() else {
                    unreachable!("checked above")
                };
                let structures: Vec<HcStructure> = guide
                    .structures
                    .iter()
                    .map(|s| HcStructure {
                        position: s.position,
                        avoidance_radius: s.avoidance_radius,
                        height: s.height,
                    })
                    .collect();
                let input = hc_guidance_input(&record.pose, held, &plan, &others);
                let mut result = HcGuidanceResult::default();
                // SAFETY: every pointer is to a live array of the length passed beside it; the C keeps none of them
                unsafe {
                    eech_hc_guidance(&input, structures.as_ptr(), structures.len() as c_int, sampled, &mut result);
                }
                self.answer = Some(HelicopterAnswer::Guidance(HelicopterGuidance {
                    lift: result.lift,
                    waypoint: result.waypoint,
                    required_height: result.required_height,
                    world_acceleration: result.world_acceleration,
                    wp_vec: result.wp_vec,
                }));
                record.hc_flow = Some(HelicopterFlow::Guided {
                    plan,
                    result,
                    rotor_checked: false,
                });
                Ok(settled(record.pose, false, held))
            }
            Request::HelicopterGuidedBegin { motion_vector } => {
                match &record.hc_flow {
                    Some(HelicopterFlow::Guided { rotor_checked: true, .. }) => {}
                    Some(HelicopterFlow::Guided { .. }) => {
                        return Err(format!(
                            "host.move_vehicle: helicopter_guided_begin for {host_id} before its rotor step (rotor_alive first)"
                        ));
                    }
                    _ => {
                        return Err(format!(
                            "host.move_vehicle: helicopter_guided_begin for {host_id} without host guidance (helicopter_plan, helicopter_guide first)"
                        ));
                    }
                }
                let held = held_vector(record, host_id, &motion_vector)?;
                let Some(HelicopterFlow::Guided { plan, result, .. }) = record.hc_flow.take() else {
                    unreachable!("checked above")
                };
                let begin = HelicopterBegin {
                    delta_seconds: plan.delta_seconds,
                    acceleration_x: result.world_acceleration[0],
                    acceleration_z: result.world_acceleration[2],
                    wp_vec_y: result.wp_vec[1],
                    motion_vector,
                };
                let (motion_vector, speed, moved) = helicopter_begin(held, &begin);
                record.motion_vector = Some(motion_vector);
                record.pose.velocity = speed;
                record.helicopter_pending = true;
                record.hc_flow = Some(HelicopterFlow::Begun { plan, result });
                Ok(Outcome {
                    moved,
                    pending: true,
                    ..settled(record.pose, false, motion_vector)
                })
            }
            Request::HelicopterGuidedEnd(steer) => {
                if !matches!(record.hc_flow, Some(HelicopterFlow::Begun { .. })) {
                    return Err(format!(
                        "host.move_vehicle: helicopter_guided_end for {host_id} without a hosted begin (helicopter_end is the legacy pair's)"
                    ));
                }
                if !hc_steer_is_finite(&steer) {
                    return Err(format!("host.move_vehicle: helicopter_guided_end for {host_id} has a non-finite input"));
                }
                let held = held_vector(record, host_id, &steer.motion_vector)?;
                let Some(HelicopterFlow::Begun { plan, result }) = record.hc_flow.take() else {
                    unreachable!("checked above")
                };
                // the demand reads the state the begin phase left: its vector and speed, and the unchanged attitude
                let mut input = HcDemandInput {
                    waypoint_kind: steer.waypoint_kind as c_int,
                    task_leader_is_self: flag(steer.task_leader_is_self),
                    guide_radius_valid: flag(steer.guide_radius.is_some()),
                    guide_radius: steer.guide_radius.unwrap_or(0.0),
                    distance: plan.distance,
                    required_heading_valid: flag(steer.required_heading.is_some()),
                    required_heading: steer.required_heading.unwrap_or(0.0),
                    leader_attitude_valid: 0,
                    leader_attitude: [0.0; 9],
                    targeting_active: flag(steer.targeting_active),
                    weapon_vector: steer.weapon_vector,
                    weapon_to_intercept_vector: steer.weapon_to_intercept_vector,
                    attitude: record.pose.attitude,
                    motion_vector: held,
                    velocity: record.pose.velocity,
                    cruise_velocity: plan.cruise_velocity,
                    g_max: steer.g_max,
                    main_rotor_shaft_angle: steer.main_rotor_shaft_angle,
                    world_acceleration: result.world_acceleration,
                    wp_vec: result.wp_vec,
                };
                let mut demand = HcDemandResult::default();
                // SAFETY: both are single structs; the C keeps no pointer
                let mut needs_leader = unsafe { eech_hc_demand(&input, &mut demand) };
                if needs_leader == HC_DEMAND_NEEDS_LEADER {
                    // the heading follows the leader: only now is its host record required, and its attitude is the host's
                    let leader = match hc_leader {
                        Some(Ok(attitude)) => attitude,
                        Some(Err(error)) => {
                            // nothing has changed: the step stays open for a corrected end
                            record.hc_flow = Some(HelicopterFlow::Begun { plan, result });
                            return Err(error);
                        }
                        None => unreachable!("a guided end always looks its leader up"),
                    };
                    input.leader_attitude_valid = 1;
                    input.leader_attitude = leader;
                    // SAFETY: as above
                    needs_leader = unsafe { eech_hc_demand(&input, &mut demand) };
                }
                debug_assert_eq!(needs_leader, 0, "the demand has its leader");
                let end = HelicopterEnd {
                    delta_seconds: steer.delta_seconds,
                    desired_heading: demand.desired_heading,
                    max_turn_rate: steer.max_turn_rate,
                    desired_pitch: demand.desired_pitch,
                    desired_roll: demand.desired_roll,
                    elevation: steer.elevation,
                    cg_to_ground: steer.cg_to_ground,
                    terrain_follow: steer.terrain_follow,
                    motion_vector: steer.motion_vector,
                    map_min: steer.map_min,
                    map_max: steer.map_max,
                };
                let (motion_vector, rotated) = helicopter_end(&mut record.pose, held, &end);
                record.motion_vector = Some(motion_vector);
                record.helicopter_pending = false;
                // the demand the host computed, for the record: heading, pitch, roll
                Ok(Outcome {
                    angles: [demand.desired_heading, demand.desired_pitch, demand.desired_roll],
                    ..settled(record.pose, rotated, motion_vector)
                })
            }
            Request::HelicopterAbandon => {
                record.hc_flow = None;
                Ok(settled(record.pose, false, record.motion_vector.unwrap_or([0.0; 3])))
            }
            Request::Lifecycle(step) => {
                let rotor_alive = step.mode == LifecycleMode::RotorAlive;
                if rotor_alive {
                    match &record.hc_flow {
                        Some(HelicopterFlow::Guided { plan, result, .. }) => {
                            if step.lift.to_bits() != result.lift.to_bits() {
                                return Err(format!(
                                    "host.move_vehicle: rotor_alive lift {} for {host_id} differs from the host's guidance ({})",
                                    step.lift, result.lift
                                ));
                            }
                            if step.power_output.to_bits() != plan.power_output.to_bits() {
                                return Err(format!(
                                    "host.move_vehicle: rotor_alive power {} for {host_id} differs from the planned step's ({})",
                                    step.power_output, plan.power_output
                                ));
                            }
                        }
                        Some(_) => {
                            return Err(format!(
                                "host.move_vehicle: rotor_alive for {host_id} between helicopter_plan and helicopter_guide"
                            ));
                        }
                        // no hosted step open: the legacy rotor step
                        None => {}
                    }
                }
                let outcome = lifecycle(record, host_id, &step)?;
                if rotor_alive {
                    // a gated rotor ends the step before it begins; otherwise the begin may now follow
                    match record.hc_flow.take() {
                        Some(HelicopterFlow::Guided { plan, result, .. }) if !outcome.moved => {
                            record.hc_flow = Some(HelicopterFlow::Guided {
                                plan,
                                result,
                                rotor_checked: true,
                            });
                        }
                        _ => {}
                    }
                }
                Ok(outcome)
            }
            Request::DependentCarry(carry) => {
                // a placement of the child, so (as `Place`) its wreck vector is left alone
                match carry.shape {
                    CarryShape::Position => record.pose.position = carry.parent_new,
                    CarryShape::Helicopter => {
                        let mut position = [0.0f32; 3];
                        let mut attitude = [0.0f32; 9];
                        // SAFETY: the positions and attitude are the 3, 3 and 9 floats the C function reads; it writes 3 and 9
                        unsafe {
                            eech_dependent_helicopter(
                                carry.parent_old.as_ptr(),
                                carry.parent_new.as_ptr(),
                                carry.heading,
                                record.pose.position.as_ptr(),
                                record.pose.attitude.as_ptr(),
                                position.as_mut_ptr(),
                                attitude.as_mut_ptr(),
                            );
                        }
                        record.pose.position = position;
                        record.pose.attitude = attitude;
                        record.pose.velocity = carry.velocity;
                    }
                }
                Ok(settled(record.pose, false, [0.0; 3]))
            }
            Request::Place { .. } | Request::Create(_) => unreachable!("placed above"),
        };
        if result.is_ok() && !in_flow {
            record.hc_flow = None;
        }
        result
    }

    /// A terrain-following fixed-wing waypoint's height, performed by the host: the original climb arithmetic from the
    /// retained pose and the terrain the campaign observed at the host's own look-ahead point, then the formation's
    /// height offset and the centre of gravity added in the original's order. The campaign's `waypoint` height is
    /// overwritten (its x and z, the formation's, stay). Read-only on the host's records and before the mutable
    /// borrow: a refusal changes nothing. Any other request, or one without terrain intent, is left alone.
    fn shape_fixed_wing_waypoint(&self, host_id: &str, request: &mut Request) -> Result<(), String> {
        let Request::FixedWingMove(step) = request else { return Ok(()) };
        let Some(terrain) = step.terrain else { return Ok(()) };
        // an unknown or non-fixed-wing subject is reported by the ordinary checks
        let Some(record) = self.mobiles.get(host_id).filter(|record| record.kind == Kind::FixedWing) else {
            return Ok(());
        };
        let refuse = |reason: &str| format!("host.move_vehicle: fixed_wing_move for {host_id}: terrain {reason}");
        let numbers =
            terrain
                .guide_position
                .iter()
                .chain(&terrain.sample_point)
                .chain([&terrain.min_height, &terrain.sample_elevation, &terrain.site_y, &terrain.cg]);
        if numbers.into_iter().any(|value| !value.is_finite()) {
            return Err(refuse("has a non-finite input"));
        }
        let pose = &record.pose;
        let mut point = [0.0f32; 2];
        // SAFETY: the position and attitude are the 3 and 9 floats the C function reads; the point is the 2 it writes
        unsafe { eech_fw_lookahead(pose.position.as_ptr(), pose.velocity, pose.attitude.as_ptr(), point.as_mut_ptr()) };
        if point
            .iter()
            .zip(&terrain.sample_point)
            .any(|(host, campaign)| host.to_bits() != campaign.to_bits())
        {
            return Err(refuse(&format!(
                "was sampled at ({}, {}), not the host's look-ahead point ({}, {})",
                terrain.sample_point[0], terrain.sample_point[1], point[0], point[1]
            )));
        }
        // SAFETY: the position, attitude and guide position are the 3, 9 and 3 floats the C function reads
        let mut height = unsafe {
            eech_fw_terrain_height(
                pose.position.as_ptr(),
                pose.velocity,
                pose.attitude.as_ptr(),
                terrain.guide_position.as_ptr(),
                terrain.sample_elevation,
                terrain.min_height,
            )
        };
        // as the original added them, in single precision and in this order: the formation site, then the centre of gravity
        height += terrain.site_y;
        height += terrain.cg;
        step.waypoint[1] = height;
        Ok(())
    }

    /// The attitude of the task leader a hosted helicopter's guided end names, from the host's own record of it. Read-only
    /// and before the mutable borrow. `None` when the request is not a guided end. The result is an error only if the id is
    /// missing or unusable, and the caller surfaces it only when the heading demand reads the leader: an id that path does
    /// not use is never refused, and no leader heading is invented.
    fn look_up_helicopter_leader(&self, host_id: &str, request: &Request) -> Option<Result<[f32; 9], String>> {
        let Request::HelicopterGuidedEnd(steer) = request else { return None };
        Some((|| {
            let name = steer
                .leader
                .as_deref()
                .ok_or_else(|| format!("host.move_vehicle: helicopter_guided_end for {host_id} follows its task leader but names no leader host id"))?;
            if name == host_id {
                return Err(format!(
                    "host.move_vehicle: helicopter_guided_end for {host_id} names itself as the task leader it follows"
                ));
            }
            if self.retired.contains(name) {
                return Err(format!("host.move_vehicle: helicopter_guided_end for {host_id}: task leader {name} is retired"));
            }
            let leader = self
                .mobiles
                .get(name)
                .ok_or_else(|| format!("host.move_vehicle: helicopter_guided_end for {host_id}: task leader {name} is not a host id"))?;
            if leader.kind != Kind::Helicopter {
                return Err(format!(
                    "host.move_vehicle: helicopter_guided_end for {host_id}: task leader {name} is not a helicopter"
                ));
            }
            Ok(leader.pose.attitude)
        })())
    }

    /// The positions of a hosted helicopter's group members, by the host's record of each, in the order the campaign's
    /// group list names them. Read-only and before the mutable borrow; `Ok(None)` when the request needs none (another
    /// request, a group leader, or a subject the ordinary checks refuse). The group is as large as the campaign sent.
    fn resolve_helicopter_others(&self, host_id: &str, request: &Request) -> Result<Option<Vec<HcOther>>, String> {
        let Request::HelicopterPlan(plan) = request else { return Ok(None) };
        if self.mobiles.get(host_id).map(|record| record.kind) != Some(Kind::Helicopter) || plan.group_leader {
            return Ok(None);
        }
        let mut others = Vec::with_capacity(plan.members.len());
        for (index, member) in plan.members.iter().enumerate() {
            let name = member
                .as_deref()
                .ok_or_else(|| format!("host.move_vehicle: helicopter_plan for {host_id}: group member #{index} names no host id"))?;
            if name == host_id {
                return Err(format!(
                    "host.move_vehicle: helicopter_plan for {host_id} names itself as group member #{index}"
                ));
            }
            if self.retired.contains(name) {
                return Err(format!("host.move_vehicle: helicopter_plan for {host_id}: group member {name} is retired"));
            }
            let other = self
                .mobiles
                .get(name)
                .ok_or_else(|| format!("host.move_vehicle: helicopter_plan for {host_id}: group member {name} is not a host id"))?;
            if other.kind != Kind::Helicopter {
                return Err(format!(
                    "host.move_vehicle: helicopter_plan for {host_id}: group member {name} is not a helicopter"
                ));
            }
            others.push(HcOther { position: other.pose.position });
        }
        Ok(Some(others))
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

/// Live movement changes the vehicle's vector legitimately, so the next wreck, fall or sink establishes it afresh.
fn release_vehicle_vector(record: &mut Record) {
    record.vehicle_vector = None;
    record.death_candidate = None;
}

/// One lifecycle step. Everything is validated (mirrors, phase) before the record changes. The vehicle and the
/// fixed-wing-only modes have their own steps; fall, gravity, contact and impact are shared by both aircraft, where only
/// a helicopter has a rotor.
fn lifecycle(record: &mut Record, host_id: &str, step: &Lifecycle) -> Result<Outcome, String> {
    let mode = step.mode;
    match mode {
        LifecycleMode::GvDeathCandidate
        | LifecycleMode::GvDeathDecay
        | LifecycleMode::GvFallCandidate
        | LifecycleMode::GvFallGravity
        | LifecycleMode::GvFallSettle
        | LifecycleMode::GvFallVector
        | LifecycleMode::GvFallCommit
        | LifecycleMode::GvImpact
        | LifecycleMode::ShipSink
        | LifecycleMode::VehicleVectorSeed => return vehicle_lifecycle(record, host_id, step),
        LifecycleMode::FwSettle
        | LifecycleMode::FwDeathAttitude
        | LifecycleMode::FwCrashCandidate
        | LifecycleMode::FwCrashVelocity
        | LifecycleMode::FwCrashAttitude => return fixed_wing_lifecycle(record, host_id, step),
        LifecycleMode::Expire => return expire_lifecycle(record, host_id, step),
        _ => {}
    }
    if record.fw_crash.is_some() {
        return Err(format!("host.move_vehicle: {} for {host_id} while its crash slide is open", mode.name()));
    }
    let helicopter = record.kind == Kind::Helicopter;
    let mut vector = held_vector(record, host_id, &step.motion_vector)?;
    let mut rotor = if !helicopter {
        [0.0; 3]
    } else if mode == LifecycleMode::RotorSet {
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
            record.fw_speed = None;
            // the candidate only: the pose waits for the tumble (a helicopter's) or the death attitude (a fixed-wing's)
            outcome.pose.position = fallen;
        }
        LifecycleMode::FallGravity => {
            let mut speed = 0.0f32;
            // SAFETY: as above.
            unsafe { eech_hc_fall_gravity(vector.as_mut_ptr(), step.gravity, dt, &mut speed) };
            record.pose.velocity = speed;
            if !helicopter {
                record.fw_speed = Some(speed);
            }
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
        _ => unreachable!("routed above"),
    }
    if helicopter {
        record.rotor = Some(rotor);
    }
    record.motion_vector = Some(vector);
    outcome.motion_vector = vector;
    Ok(outcome)
}

/// The end of a dead mobile's physical life (`mobile.c`, `rv_updt.c`, `ps_updt.c`, `fw_updt.c`, `hc_updt.c`, `sh_move.c`).
/// A ship has no timer: it is gone when its position has sunk below half its bounding-box diagonal under the waterline.
/// Every other wreck ages by the frame, and only on the frames where `(int)(timer + 1)` is a multiple of 300 is the timer
/// compared with the threshold that the squared range to the campaign's nearest keysite sets. The host owns the timer
/// (adopted from the campaign's copy while it holds none, then validated bit for bit) and the answer; the campaign
/// removes the entity family when told the life is over, which retires the host id. Nothing else is mutated by a refusal.
fn expire_lifecycle(record: &mut Record, host_id: &str, step: &Lifecycle) -> Result<Outcome, String> {
    let mode = step.mode;
    if record.fall.is_some() || record.death_candidate.is_some() || record.fw_crash.is_some() {
        return Err(format!(
            "host.move_vehicle: {} for {host_id} while its fall, death slide or crash slide is still open",
            mode.name()
        ));
    }
    if step.death_policy & !DEATH_POLICY_BITS != 0 {
        return Err(format!(
            "host.move_vehicle: expire for {host_id} carries unsupported death_policy bits {:#x}",
            step.death_policy & !DEATH_POLICY_BITS
        ));
    }
    let mut outcome = settled(record.pose, false, [0.0; 3]);
    if record.kind == Kind::Ship {
        // `obscured_altitude = -(0.5 * magnitude)` is a double product narrowed into a float, then compared with a float
        let obscured_altitude = (-(0.5f64 * f64::from(step.bbox_diagonal))) as f32;
        outcome.flags = i32::from(record.pose.position[1] < obscured_altitude);
        return Ok(outcome);
    }
    // the C gate converts `(int)(timer + 1)`: refuse a timer or delta it cannot convert, before anything is read or kept
    let valid_time = |value: f32| value.is_finite() && value >= 0.0;
    if !valid_time(step.death_timer) || !valid_time(step.delta_seconds) {
        return Err(format!("host.move_vehicle: expire for {host_id} needs a finite, nonnegative timer and delta"));
    }
    let held = match record.death_timer {
        Some(held) => {
            check_mirror("death_timer", host_id, &[held], &[step.death_timer])?;
            held
        }
        None => step.death_timer,
    };
    let mut timer = held + step.delta_seconds;
    // the gate's `(int)(timer + 1)` is undefined past the int range; `i32::MAX as f32` is 2^31, so strict
    if !timer.is_finite() || timer + 1.0 >= i32::MAX as f32 {
        return Err(format!("host.move_vehicle: expire for {host_id} would age its timer past the gate's range"));
    }
    let range_supplied = step.death_policy & DEATH_RANGE_SUPPLIED != 0;
    if range_supplied && !valid_time(step.death_range) {
        return Err(format!(
            "host.move_vehicle: expire for {host_id} supplies a non-finite or negative keysite range"
        ));
    }
    let mut expired = false;
    if step.death_policy & DEATH_NEVER_EXPIRES == 0 {
        // SAFETY: scalar-in, scalar-out arithmetic; the timer was range-checked above.
        let gated = unsafe { eech_expire_gate(timer) != 0 };
        if gated && !range_supplied {
            // the range is the campaign's costly full keysite scan, wanted only on a gated frame: ask for it, change nothing
            outcome.death_timer = held;
            outcome.flags = EXPIRE_NEEDS_RANGE;
            return Ok(outcome);
        }
        // SAFETY: scalar-in, scalar-out arithmetic.
        let threshold = if gated { unsafe { eech_expire_threshold(step.death_range) } } else { 0.0 };
        if gated && timer >= threshold {
            if step.death_policy & DEATH_OCCUPIED != 0 {
                timer = 0.0;
            } else {
                expired = true;
            }
        }
    }
    record.death_timer = Some(timer);
    outcome.death_timer = timer;
    outcome.flags = i32::from(expired);
    Ok(outcome)
}

/// Refused (before anything is mutated) unless the campaign's mirror equals what the host holds, bit for bit.
fn check_mirror(what: &str, host_id: &str, held: &[f32], mirror: &[f32]) -> Result<(), String> {
    if let Some(index) = held.iter().zip(mirror).position(|(a, b)| a.to_bits() != b.to_bits()) {
        return Err(format!(
            "host.move_vehicle: the campaign's {what}[{index}] ({} against {}) for {host_id} differs from the host's record (a write the host did not make)",
            mirror[index], held[index]
        ));
    }
    Ok(())
}

/// The open fall, refused unless it is at one of the `wanted` stages.
fn open_fall(record: &Record, host_id: &str, mode: LifecycleMode, wanted: &[FallStage]) -> Result<GroundFall, String> {
    match record.fall {
        Some(fall) if wanted.contains(&fall.stage) => Ok(fall),
        Some(fall) => Err(format!(
            "host.move_vehicle: {} for {host_id} out of order (the fall is at {:?})",
            mode.name(),
            fall.stage
        )),
        None => Err(format!(
            "host.move_vehicle: {} for {host_id} without an open fall (gv_fall_candidate first)",
            mode.name()
        )),
    }
}

/// One routed, anti-air or ship wreck, fall, sink or impact step. The campaign's copy of the vector is validated against the
/// host's claim on every mode, and everything is validated before the record changes.
fn vehicle_lifecycle(record: &mut Record, host_id: &str, step: &Lifecycle) -> Result<Outcome, String> {
    let mode = step.mode;
    if record.fall.is_some()
        && !matches!(
            mode,
            LifecycleMode::GvFallGravity | LifecycleMode::GvFallSettle | LifecycleMode::GvFallVector | LifecycleMode::GvFallCommit
        )
    {
        return Err(format!("host.move_vehicle: {} for {host_id} while its fall is open", mode.name()));
    }
    // a staged death slide ends only with its own decay (or a placement, which keeps it): nothing else may overwrite or abandon it
    if record.death_candidate.is_some() && mode != LifecycleMode::GvDeathDecay {
        return Err(format!(
            "host.move_vehicle: {} for {host_id} while its death slide is pending (gv_death_decay first)",
            mode.name()
        ));
    }
    if mode == LifecycleMode::VehicleVectorSeed {
        let held = match record.vehicle_vector {
            Some(held) => {
                check_mirror("motion_vector", host_id, &held, &step.motion_vector)?;
                held
            }
            None => step.motion_vector,
        };
        record.vehicle_vector = Some(held);
        return Ok(settled(record.pose, false, held));
    }
    let claim = record.vehicle_vector.ok_or_else(|| {
        format!(
            "host.move_vehicle: {} for {host_id} without a claimed vehicle vector (vehicle_vector_seed first)",
            mode.name()
        )
    })?;
    check_mirror("motion_vector", host_id, &claim, &step.motion_vector)?;
    let dt = step.delta_seconds;
    let mut outcome = settled(record.pose, false, claim);
    match mode {
        LifecycleMode::GvDeathCandidate => {
            let mut xz = [0.0f32; 2];
            // SAFETY: the position and vector are 3 floats and the output 2.
            unsafe { eech_gv_death_candidate(record.pose.position.as_ptr(), claim.as_ptr(), dt, xz.as_mut_ptr()) };
            record.death_candidate = Some(xz);
            outcome.pose.position[0] = xz[0];
            outcome.pose.position[2] = xz[1];
        }
        LifecycleMode::GvDeathDecay => {
            let candidate = record
                .death_candidate
                .ok_or_else(|| format!("host.move_vehicle: gv_death_decay for {host_id} without a death candidate (gv_death_candidate first)"))?;
            check_mirror("candidate", host_id, &candidate, &[step.waypoint[0], step.waypoint[2]])?;
            let mut position = [0.0f32; 3];
            let mut vector = [0.0f32; 3];
            let mut speed = 0.0f32;
            let mut attitude = [0.0f32; 9];
            // SAFETY: as above; the attitude is 9 floats written.
            unsafe {
                eech_gv_death_decay(
                    candidate.as_ptr(),
                    step.elevation,
                    step.cg_to_ground,
                    claim.as_ptr(),
                    record.pose.attitude.as_ptr(),
                    dt,
                    c_int::from(step.wide),
                    position.as_mut_ptr(),
                    vector.as_mut_ptr(),
                    &mut speed,
                    attitude.as_mut_ptr(),
                );
            }
            record.death_candidate = None;
            record.vehicle_vector = Some(vector);
            record.pose.position = position;
            record.pose.velocity = speed;
            record.pose.attitude = attitude;
            outcome.pose = record.pose;
            outcome.motion_vector = vector;
        }
        LifecycleMode::GvFallCandidate => {
            let mut candidate = [0.0f32; 3];
            // SAFETY: as above.
            unsafe { eech_gv_fall_candidate(record.pose.position.as_ptr(), claim.as_ptr(), dt, candidate.as_mut_ptr()) };
            record.fall = Some(GroundFall {
                stage: FallStage::Candidate,
                candidate,
                staged_vector: claim,
                staged_position: candidate,
                staged_speed: record.pose.velocity,
            });
            outcome.pose.position = candidate;
        }
        LifecycleMode::GvFallGravity => {
            let mut fall = open_fall(record, host_id, mode, &[FallStage::Candidate])?;
            let mut vector = claim;
            // SAFETY: as above. Gravity changes the vector only; the scalar stays the pose's until the commit.
            let _ = unsafe { eech_gv_fall_gravity(vector.as_mut_ptr(), step.gravity, dt) };
            fall.stage = FallStage::Gravity;
            record.fall = Some(fall);
            record.vehicle_vector = Some(vector);
            outcome.motion_vector = vector;
        }
        LifecycleMode::GvFallSettle => {
            let mut fall = open_fall(record, host_id, mode, &[FallStage::Gravity])?;
            check_mirror("candidate", host_id, &fall.candidate, &step.waypoint)?;
            let mut vector = claim;
            let mut position = [0.0f32; 3];
            let mut speed = 0.0f32;
            // SAFETY: as above. Nothing is applied: the position, vector and speed are staged for the commit.
            let flags = unsafe {
                eech_gv_fall_settle(
                    record.pose.position.as_ptr(),
                    fall.candidate.as_ptr(),
                    vector.as_mut_ptr(),
                    c_int::from(step.water),
                    step.old_elevation,
                    step.elevation,
                    step.cg_to_ground,
                    step.bbox_diagonal,
                    position.as_mut_ptr(),
                    &mut speed,
                )
            };
            fall.stage = FallStage::Settled;
            fall.staged_vector = vector;
            fall.staged_position = position;
            fall.staged_speed = speed;
            record.fall = Some(fall);
            outcome.pose.position = position;
            outcome.pose.velocity = speed;
            outcome.motion_vector = vector;
            outcome.flags = flags;
        }
        LifecycleMode::GvFallVector => {
            let mut fall = open_fall(record, host_id, mode, &[FallStage::Settled])?;
            fall.stage = FallStage::Vectored;
            record.fall = Some(fall);
            record.vehicle_vector = Some(fall.staged_vector);
            outcome.motion_vector = fall.staged_vector;
        }
        LifecycleMode::GvFallCommit => {
            let fall = open_fall(record, host_id, mode, &[FallStage::Settled, FallStage::Vectored])?;
            if let Err(error) = check_mirror("staged motion_vector", host_id, &claim, &fall.staged_vector) {
                return Err(format!("{error}; gv_fall_vector was not applied before the commit"));
            }
            record.fall = None;
            record.pose.velocity = fall.staged_speed;
            record.pose.position = fall.staged_position;
            outcome.pose = record.pose;
        }
        LifecycleMode::GvImpact => {
            let mut vector = claim;
            // SAFETY: as above; the attitude is 9 floats and its first row is the x axis.
            unsafe { eech_gv_impact(vector.as_mut_ptr(), step.signed_speed, record.pose.attitude.as_ptr()) };
            record.vehicle_vector = Some(vector);
            outcome.motion_vector = vector;
        }
        LifecycleMode::ShipSink => {
            let mut position = [0.0f32; 3];
            let mut vector = [0.0f32; 3];
            let mut speed = 0.0f32;
            let mut attitude = [0.0f32; 9];
            // SAFETY: as above.
            unsafe {
                eech_ship_sink(
                    record.pose.position.as_ptr(),
                    claim.as_ptr(),
                    record.pose.attitude.as_ptr(),
                    dt,
                    position.as_mut_ptr(),
                    vector.as_mut_ptr(),
                    &mut speed,
                    attitude.as_mut_ptr(),
                );
            }
            record.vehicle_vector = Some(vector);
            record.pose.position = position;
            record.pose.velocity = speed;
            record.pose.attitude = attitude;
            outcome.pose = record.pose;
            outcome.motion_vector = vector;
        }
        _ => unreachable!("not a vehicle mode"),
    }
    Ok(outcome)
}

/// One fixed-wing death settle, death attitude or crash-slide step (fall, gravity, contact and impact are `lifecycle`'s).
/// The staged candidate is the record's; a placement between phases keeps it.
fn fixed_wing_lifecycle(record: &mut Record, host_id: &str, step: &Lifecycle) -> Result<Outcome, String> {
    let mode = step.mode;
    let mut vector = held_vector(record, host_id, &step.motion_vector)?;
    let dt = step.delta_seconds;
    let mut outcome = settled(record.pose, false, vector);
    let crash = record.fw_crash;
    match (mode, crash) {
        (LifecycleMode::FwCrashVelocity, Some(CrashStage::Candidate)) | (LifecycleMode::FwCrashAttitude, Some(CrashStage::Velocity { .. })) => {}
        (LifecycleMode::FwCrashVelocity | LifecycleMode::FwCrashAttitude, _) => {
            return Err(format!(
                "host.move_vehicle: {} for {host_id} out of order (the crash slide is at {crash:?})",
                mode.name()
            ));
        }
        (_, Some(_)) => {
            return Err(format!("host.move_vehicle: {} for {host_id} while its crash slide is open", mode.name()));
        }
        _ => {}
    }
    match mode {
        LifecycleMode::FwSettle => {
            let mut candidate = record
                .candidate
                .ok_or_else(|| format!("host.move_vehicle: fw_settle for {host_id} without a stored candidate position (fall first)"))?;
            let altitude = record
                .contact_altitude
                .ok_or_else(|| format!("host.move_vehicle: fw_settle for {host_id} without the contact of its candidate (contact with the candidate first)"))?;
            let mut buried: c_int = 0;
            // SAFETY: as above; the held position is the pose after the campaign's splash placement.
            unsafe {
                eech_fw_settle(
                    candidate.as_mut_ptr(),
                    record.pose.position.as_ptr(),
                    vector.as_mut_ptr(),
                    step.terrain_class,
                    step.elevation,
                    step.cg_to_ground,
                    altitude,
                    step.bury_limit,
                    &mut buried,
                );
            }
            // the contact altitude stays: the death attitude is verified against it, and the position is applied there
            record.candidate = Some(candidate);
            outcome.pose.position = candidate;
            outcome.moved = buried != 0;
        }
        LifecycleMode::FwDeathAttitude => {
            let candidate = record
                .candidate
                .ok_or_else(|| format!("host.move_vehicle: fw_death_attitude for {host_id} without a stored candidate position (fall first)"))?;
            let altitude = record
                .contact_altitude
                .ok_or_else(|| format!("host.move_vehicle: fw_death_attitude for {host_id} without the contact of its candidate"))?;
            let speed = record
                .fw_speed
                .ok_or_else(|| format!("host.move_vehicle: fw_death_attitude for {host_id} without a stored post-gravity speed (fall_gravity first)"))?;
            check_mirror("altitude", host_id, &[altitude], &[step.elevation])?;
            check_mirror("speed", host_id, &[speed], &[step.signed_speed])?;
            let mut attitude = record.pose.attitude;
            // SAFETY: as above; the attitude is 9 floats read then written.
            let flags = unsafe {
                eech_fw_death_attitude(
                    attitude.as_mut_ptr(),
                    vector.as_ptr(),
                    speed,
                    altitude,
                    c_int::from(step.flag),
                    step.draws[1],
                    dt,
                )
            };
            outcome.flags = flags;
            if flags >= 0 {
                if flags & 1 != 0 {
                    record.pose.attitude = attitude;
                }
                // always: the staged candidate is the final position, even when the attitude gate is false
                record.pose.position = candidate;
                record.candidate = None;
                record.contact_altitude = None;
                record.fw_speed = None;
                outcome.pose = record.pose;
            }
            return Ok(outcome);
        }
        LifecycleMode::FwCrashCandidate => {
            if record.candidate.is_some() {
                return Err(format!(
                    "host.move_vehicle: fw_crash_candidate for {host_id} while a candidate position is stored"
                ));
            }
            let mut slid = [0.0f32; 3];
            // SAFETY: as above.
            unsafe {
                eech_fw_crash_candidate(
                    record.pose.position.as_ptr(),
                    vector.as_ptr(),
                    dt,
                    step.map_min.as_ptr(),
                    step.map_max.as_ptr(),
                    slid.as_mut_ptr(),
                );
            }
            record.candidate = Some(slid);
            record.fw_crash = Some(CrashStage::Candidate);
            outcome.pose.position = slid;
        }
        LifecycleMode::FwCrashVelocity => {
            let mut slid = record
                .candidate
                .ok_or_else(|| format!("host.move_vehicle: fw_crash_velocity for {host_id} without a stored candidate position"))?;
            let mut speed = 0.0f32;
            let mut stopped: c_int = 0;
            // SAFETY: as above.
            unsafe {
                eech_fw_crash_velocity(
                    record.pose.position.as_ptr(),
                    slid.as_mut_ptr(),
                    vector.as_mut_ptr(),
                    step.face_normal.as_ptr(),
                    c_int::from(step.forest),
                    step.elevation,
                    step.cg_to_ground,
                    dt,
                    &mut speed,
                    &mut stopped,
                );
            }
            record.candidate = Some(slid);
            record.fw_crash = Some(CrashStage::Velocity { speed });
            if stopped == 0 {
                record.pose.velocity = speed;
            }
            // when stopped the scalar is the campaign's to clear (a placement); the slope's speed stays retained
            outcome.pose.position = slid;
            outcome.pose.velocity = speed;
            outcome.flags = c_int::from(stopped != 0);
        }
        LifecycleMode::FwCrashAttitude => {
            let Some(CrashStage::Velocity { speed }) = crash else {
                unreachable!("checked above")
            };
            let slid = record.candidate.expect("stored with the crash candidate");
            let mut attitude = record.pose.attitude;
            let mut at_rest: c_int = 0;
            // SAFETY: as above; the attitude is 9 floats read then written.
            unsafe {
                eech_fw_crash_attitude(
                    attitude.as_mut_ptr(),
                    step.face_normal.as_ptr(),
                    speed,
                    step.pitch_offset,
                    step.bank_offset,
                    dt,
                    &mut at_rest,
                );
            }
            record.pose.attitude = attitude;
            record.pose.position = slid;
            record.candidate = None;
            record.fw_crash = None;
            outcome.pose = record.pose;
            outcome.flags = c_int::from(at_rest != 0);
        }
        _ => unreachable!("not a fixed-wing mode"),
    }
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
        flags: 0,
        angles: [0.0; 3],
        death_timer: 0.0,
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

/// Whether the waypoint lies behind the motion vector (the original's dot-product test, in the host's C).
fn eech_facing_away(position: &[f32; 3], motion_vector: &[f32; 3], waypoint: &[f32; 3]) -> bool {
    // SAFETY: each argument is 3 floats the C function only reads.
    unsafe { eech_fw_facing_away(position.as_ptr(), motion_vector.as_ptr(), waypoint.as_ptr()) != 0 }
}

/// The host's turn decision for an airborne step: whether the cannon hold, the turn-around or a hold still in force
/// sends the aircraft to the absolute branch instead of the navigator. `turn` (the retained multiplier and hold) is
/// updated in place; the leader's position is read only when the aircraft is facing away and does not lead its task.
/// The step and the host's look ahead at what it will read both come through here, so they cannot disagree.
fn turn_is_absolute(
    step: &FixedWingMove,
    position: [f32; 3],
    motion_vector: [f32; 3],
    task_leader: bool,
    leader_position: [f32; 3],
    turn: &mut FixedWingTurn,
) -> bool {
    let input = FwTurnInput {
        task_leader: c_int::from(task_leader),
        aiming_cannon: c_int::from(step.aiming_cannon),
        elapsed_time: step.elapsed_time,
        position,
        motion_vector,
        waypoint: step.waypoint,
        leader_position,
    };
    // SAFETY: `input` is read-only; the multiplier and toggle are single floats the C function reads and writes.
    unsafe { eech_fw_turn(&input, &mut turn.turn_speed_multi, &mut turn.toggle) != 0 }
}

/// The aircraft's new motion vector and whether the step turned it enough to report. An airborne step (flight or
/// landing) first lets the host decide, from its retained pose and motion, whether the aircraft is facing away from
/// its waypoint (or aiming a cannon) and so flies the absolute turn-around instead; `turn` is the retained multiplier
/// and hold, updated in place and committed by the caller only when the step is made. Taxi and an explicit absolute
/// step leave the decision alone and fly with the retained multiplier.
fn fixed_wing_pose(
    record: &mut Pose,
    previous: [f32; 3],
    step: &FixedWingMove,
    navigation: Option<&ResolvedNavigation>,
    turn: &mut FixedWingTurn,
) -> ([f32; 3], bool) {
    let mut motion_vector = previous;
    let mut rotated: c_int = 0;
    let mut phase = step.phase;
    if let (FixedWingPhase::Flight | FixedWingPhase::Land, Some(navigation)) = (step.phase, navigation) {
        if turn_is_absolute(step, record.position, previous, navigation.task_leader, navigation.leader_position, turn) {
            phase = FixedWingPhase::Absolute;
        }
    }
    // only normal flight reads the acceleration; the host computes it from its own pose and motion (before the step
    // moves them) and from the other aircraft its records resolved
    let mut acceleration = [0.0f32; 3];
    if let (FixedWingPhase::Flight, Some(navigation)) = (phase, navigation) {
        let input = FwNavInput {
            group_leader: c_int::from(navigation.group_leader),
            task_leader: c_int::from(navigation.task_leader),
            delta_time: step.delta_seconds,
            position: record.position,
            motion_vector: previous,
            attitude: record.attitude,
            velocity: record.velocity,
            navigation_velocity: previous,
            cruise_velocity: step.cruise_velocity,
            max_vertical_turn_allowance: navigation.max_vertical_turn_allowance,
            g_max: step.g_max,
            avoidance_radius: navigation.avoidance_radius,
            waypoint: step.waypoint,
            guide_velocity: step.wp_vel,
            task_leader_motion_vector: navigation.leader_motion_vector,
            other_count: navigation.others.len() as c_int,
            others: navigation.others.as_ptr(),
        };
        // SAFETY: `input` is read-only and its `others` points at `other_count` initialised entries that outlive the
        // call; the acceleration is the 3 floats the C function writes.
        unsafe { eech_fw_navigate(&input, acceleration.as_mut_ptr()) };
    }
    // SAFETY: as for `advance_pose`; the acceleration, waypoint and bounds are 3 floats, the
    // motion vector is 3 floats read then written.
    unsafe {
        eech_fw_step(
            phase as c_int,
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
            turn.turn_speed_multi,
            step.waypoint.as_ptr(),
            acceleration.as_ptr(),
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
            flags: 0,
            angles: [0.0; 3],
            death_timer: 0.0,
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

    /// a navigation that needs no other aircraft: the task leader of a group it leads
    fn task_leader_navigation() -> FixedWingNavigation {
        FixedWingNavigation {
            group_leader: true,
            task_leader: true,
            max_vertical_turn_allowance: 0.5,
            avoidance_radius: 20.0,
            leader: None,
            members: Vec::new(),
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
            waypoint: [0.0, 0.0, 5000.0],
            aiming_cannon: false,
            elapsed_time: 100.0,
            navigation: matches!(phase, FixedWingPhase::Flight | FixedWingPhase::Land).then(task_leader_navigation),
            terrain: None,
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

    fn creation(position: [f32; 3], heading: f32, face_normal: [f32; 3]) -> Creation {
        Creation {
            position,
            cg_to_ground: 0.0,
            heading,
            fuselage_angle: 0.0,
            face_normal,
            motion_vector: [0.0, 0.0, 100.0],
        }
    }

    #[test]
    fn the_host_answers_the_physical_height_from_the_base_and_the_cg_offset() {
        let mut low = movers(Kind::Routed, pose(0.0, 0.0));
        let mut high = movers(Kind::Routed, pose(0.0, 0.0));
        let mut request = creation([10.0, 100.0, 20.0], 0.0, [0.0, 1.0, 0.0]);
        request.cg_to_ground = 1.5;
        let first = low.step("v", pose(0.0, 0.0), Request::Create(request)).unwrap();
        // x and z are the campaign's formation intent; the height is the host's sum
        assert_eq!(first.pose.position, [10.0, 100.0 + 1.5, 20.0]);
        request.cg_to_ground = 4.0;
        let second = high.step("v", pose(0.0, 0.0), Request::Create(request)).unwrap();
        assert_eq!(second.pose.position, [10.0, 104.0, 20.0]);
        // the record keeps the host's height, not the request's base
        assert_eq!(high.step("v", second.pose, Request::Stop).unwrap().pose.position, [10.0, 104.0, 20.0]);
    }

    #[test]
    fn a_creation_whose_height_overflows_changes_nothing() {
        let mut movers = movers(Kind::Routed, pose(3.0, 4.0));
        let mut bad = creation([0.0, f32::MAX, 0.0], 0.0, [0.0, 1.0, 0.0]);
        bad.cg_to_ground = f32::MAX;
        assert!(movers.step("v", pose(3.0, 4.0), Request::Create(bad)).unwrap_err().contains("non-finite"));
        assert!(movers
            .step("v", pose(3.0, 4.0), Request::Create(creation([5.0; 3], 0.0, [0.0, 1.0, 0.0])))
            .is_ok());
    }

    #[test]
    fn the_host_builds_the_placement_attitude_on_flat_ground() {
        let mut movers = movers(Kind::Routed, pose(0.0, 0.0));
        let heading = 0.5f32;
        let made = movers
            .step("v", pose(0.0, 0.0), Request::Create(creation([10.0, 2.0, 20.0], heading, [0.0, 1.0, 0.0])))
            .unwrap();
        assert_eq!(made.pose.position, [10.0, 2.0, 20.0]);
        assert_eq!(made.angles[0], heading);
        assert!(
            made.angles[1].abs() < 1.0e-6 && made.angles[2].abs() < 1.0e-6,
            "level ground: {:?}",
            made.angles
        );
        // facing `heading`: the heading row is [sin, 0, cos]
        assert!((made.pose.attitude[6] - heading.sin()).abs() < 1.0e-6);
        assert!((made.pose.attitude[8] - heading.cos()).abs() < 1.0e-6);
        // the record is the host's: the next step compares against it
        let kept = movers.step("v", made.pose, Request::Stop).unwrap();
        assert_eq!(kept.pose.position, [10.0, 2.0, 20.0]);
    }

    #[test]
    fn the_terrain_face_and_the_fuselage_angle_shape_the_host_attitude() {
        let mut level = movers(Kind::Routed, pose(0.0, 0.0));
        let mut sloped = movers(Kind::Routed, pose(0.0, 0.0));
        let normal = {
            let (x, y, z) = (0.2f32, 0.9f32, -0.1f32);
            let length = (x * x + y * y + z * z).sqrt();
            [x / length, y / length, z / length]
        };
        let flat = level
            .step("v", pose(0.0, 0.0), Request::Create(creation([0.0; 3], 1.0, [0.0, 1.0, 0.0])))
            .unwrap();
        let tilted = sloped.step("v", pose(0.0, 0.0), Request::Create(creation([0.0; 3], 1.0, normal))).unwrap();
        assert_ne!(
            flat.pose.attitude, tilted.pose.attitude,
            "the host's attitude follows the terrain face it was given"
        );
        assert!(tilted.angles[1].abs() > 0.01 && tilted.angles[2].abs() > 0.01, "{:?}", tilted.angles);
        // the matrix is the one those angles build (its forward row is heading and pitch), so the campaign's angle setter
        // reproduces it
        let (sh, ch) = tilted.angles[0].sin_cos();
        let (sp, cp) = tilted.angles[1].sin_cos();
        let forward = [sh * cp, sp, ch * cp];
        for (offset, value) in forward.iter().enumerate() {
            assert!((tilted.pose.attitude[6 + offset] - value).abs() < 1.0e-5, "forward row [{offset}]");
        }
        // the fuselage angle lifts the pitch
        let mut lifted = movers(Kind::Helicopter, pose(0.0, 0.0));
        let mut request = creation([0.0; 3], 0.0, [0.0, 1.0, 0.0]);
        request.fuselage_angle = 0.1;
        let made = lifted.step("v", pose(0.0, 0.0), Request::Create(request)).unwrap();
        assert!((made.angles[1] - 0.1).abs() < 1.0e-6, "{:?}", made.angles);
    }

    #[test]
    fn a_mobile_is_created_once() {
        let mut movers = movers(Kind::Routed, pose(0.0, 0.0));
        movers
            .step("v", pose(0.0, 0.0), Request::Create(creation([1.0; 3], 0.0, [0.0, 1.0, 0.0])))
            .unwrap();
        let error = movers
            .step("v", pose(1.0, 1.0), Request::Create(creation([2.0; 3], 0.0, [0.0, 1.0, 0.0])))
            .unwrap_err();
        assert!(error.contains("already created"), "{error}");
    }

    #[test]
    fn a_creation_for_an_unknown_or_retired_id_is_refused() {
        let mut movers = movers(Kind::Routed, pose(0.0, 0.0));
        let request = Request::Create(creation([0.0; 3], 0.0, [0.0, 1.0, 0.0]));
        assert!(movers.step("other", pose(0.0, 0.0), request.clone()).unwrap_err().contains("unknown host id"));
        movers.retire("v").unwrap();
        assert!(movers.step("v", pose(0.0, 0.0), request).unwrap_err().contains("retired"));
    }

    #[test]
    fn a_creation_with_a_non_finite_input_changes_nothing() {
        let mut movers = movers(Kind::Routed, pose(3.0, 4.0));
        let mut bad = creation([0.0; 3], 0.0, [0.0, 1.0, 0.0]);
        bad.position[1] = f32::NAN;
        assert!(movers.step("v", pose(3.0, 4.0), Request::Create(bad)).unwrap_err().contains("non-finite"));
        // refused before the record changed: the creation may still be made
        let made = movers
            .step("v", pose(3.0, 4.0), Request::Create(creation([5.0; 3], 0.0, [0.0, 1.0, 0.0])))
            .unwrap();
        assert_eq!(made.pose.position, [5.0; 3]);
    }

    #[test]
    fn an_aircraft_creation_seeds_the_motion_vector_its_steps_are_checked_against() {
        let mut movers = movers(Kind::FixedWing, pose(0.0, 0.0));
        let mut request = creation([0.0, 10.0, 0.0], 0.0, [0.0, 1.0, 0.0]);
        request.motion_vector = [0.0, 0.0, 50.0];
        let made = movers.step("v", pose(0.0, 0.0), Request::Create(request)).unwrap();
        // a step that carries a different vector than the one created is a write the host did not make
        let mut step = fixed_wing_move(FixedWingPhase::Flight);
        step.motion_vector = [0.0, 0.0, 49.0];
        let error = movers.step("v", made.pose, Request::FixedWingMove(step)).unwrap_err();
        assert!(error.contains("motion_vector"), "{error}");
    }

    #[test]
    fn a_retired_mobile_answers_no_later_step() {
        let mut movers = movers(Kind::Routed, pose(0.0, 0.0));
        movers.step("v", pose(0.0, 0.0), Request::Stop).unwrap();
        movers.retire("v").unwrap();
        for request in [Request::Stop, PLACED, Request::Advance(advance([0.0, 0.0, 100.0]))] {
            let error = movers.step("v", pose(0.0, 0.0), request).unwrap_err();
            assert!(error.contains("retired"), "{error}");
        }
    }

    #[test]
    fn a_retired_id_is_never_reused() {
        let mut movers = movers(Kind::Routed, pose(0.0, 0.0));
        movers.retire("v").unwrap();
        let error = movers.bind("v", Kind::Routed, pose(0.0, 0.0), None, None).unwrap_err();
        assert!(error.contains("retired"), "{error}");
        assert!(movers.issue("v").unwrap_err().contains("retired"));
        assert!(movers.retire("v").unwrap_err().contains("retired"), "a mobile is retired once");
    }

    #[test]
    fn a_live_id_is_never_bound_twice() {
        let mut movers = movers(Kind::Routed, pose(1.0, 1.0));
        let error = movers.bind("v", Kind::Ship, pose(0.0, 0.0), None, None).unwrap_err();
        assert!(error.contains("already"), "{error}");
        assert!(movers.issue("v").unwrap_err().contains("already"));
        // the refused bind left the first record untouched
        let kept = movers.step("v", pose(1.0, 1.0), Request::Stop).unwrap();
        assert_eq!(kept.pose, pose(1.0, 1.0));
    }

    #[test]
    fn an_id_issued_without_a_creation_request_retires_cleanly() {
        let mut movers = Movers::default();
        movers.issue("link-only").unwrap();
        assert!(movers.issue("link-only").unwrap_err().contains("already"));
        let error = movers.step("link-only", pose(0.0, 0.0), Request::Stop).unwrap_err();
        assert!(error.contains("unknown host id"), "{error}");
        movers.retire("link-only").unwrap();
        assert!(movers.step("link-only", pose(0.0, 0.0), Request::Stop).unwrap_err().contains("retired"));
        assert!(movers.retire("never-issued").unwrap_err().contains("unknown host id"));
    }

    #[test]
    fn a_mobile_retires_from_any_open_step() {
        let mut movers = movers(Kind::Helicopter, pose(0.0, 0.0));
        movers.step("v", pose(0.0, 0.0), Request::HelicopterBegin(helicopter_begin_request())).unwrap();
        // the heap may free an entity mid-step: the host drops the open phase with the record
        movers.retire("v").unwrap();
        assert!(movers
            .step("v", pose(0.0, 0.0), Request::HelicopterEnd(helicopter_end_request()))
            .unwrap_err()
            .contains("retired"));
    }

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

    /// BASE CHARACTERISATION (EEDCS-40): the DLL moves a carried child by its own setters while the host steps the parent,
    /// and `applying_host_result` suppresses the child's `campaign_movement_place`. The host therefore never sees the
    /// child's new pose: its next request for the child is refused as a write the host did not make.
    #[test]
    fn base_a_child_pose_the_campaign_wrote_during_its_parents_step_is_a_write_the_host_never_saw() {
        let mut movers = Movers::default();
        movers.bind("ship", Kind::Ship, pose(0.0, 0.0), None, None).unwrap();
        movers.bind("child", Kind::Routed, pose(10.0, 0.0), None, None).unwrap();
        movers.bind("grandchild", Kind::Routed, pose(20.0, 0.0), None, None).unwrap();
        let moved = movers.step("ship", pose(0.0, 0.0), Request::ShipMove(ship_move([0.0, 0.0, 1000.0]))).unwrap();
        let delta = [moved.pose.position[0] - 0.0, moved.pose.position[1] - 0.0, moved.pose.position[2] - 0.0];
        assert_near(delta, [0.0, 0.0, 0.5]);
        // the campaign's setter carried the child and the grandchild with the parent; nothing told the host
        let carried = |start: f32| Pose {
            position: [start + delta[0], delta[1], delta[2]],
            ..pose(start, 0.0)
        };
        for (id, start) in [("child", 10.0), ("grandchild", 20.0)] {
            let error = movers.step(id, carried(start), Request::Stop).unwrap_err();
            assert!(
                error.contains("differs from the host's record (a write the host did not make)"),
                "{id}: {error}"
            );
        }
        // an unguarded setter would have placed it: that is what makes the record agree again
        for (id, start) in [("child", 10.0), ("grandchild", 20.0)] {
            movers.step(id, carried(start), PLACED).unwrap();
            movers.step(id, carried(start), Request::Stop).unwrap();
        }
    }

    fn carry(parent: &str, shape: CarryShape, old: [f32; 3], new: [f32; 3]) -> Request {
        Request::DependentCarry(DependentCarry {
            parent: parent.to_owned(),
            shape,
            parent_old: old,
            parent_new: new,
            heading: 0.0,
            velocity: 0.0,
        })
    }

    /// The fix for the witness above: the host positions the attached children itself, so its records follow the
    /// parent the host moved, nested one level further, and the next request for each is accepted unchanged.
    #[test]
    fn a_child_and_a_grandchild_follow_a_parent_the_host_moved_and_stay_consistent() {
        let mut movers = Movers::default();
        movers.bind("ship", Kind::Ship, pose(0.0, 0.0), None, None).unwrap();
        movers.bind("child", Kind::Routed, pose(10.0, 0.0), None, None).unwrap();
        movers.bind("grandchild", Kind::Person, pose(20.0, 0.0), None, None).unwrap();
        let moved = movers.step("ship", pose(0.0, 0.0), Request::ShipMove(ship_move([0.0, 0.0, 1000.0]))).unwrap();
        // the notification names the position the parent had; the host already holds the new one
        let child = movers
            .step("child", pose(10.0, 0.0), carry("ship", CarryShape::Position, [0.0; 3], moved.pose.position))
            .unwrap();
        // a copy of the parent's new position, the child's own attitude and speed untouched
        assert_eq!(child.pose.position, moved.pose.position);
        assert_eq!(child.pose.attitude, LEVEL);
        // the grandchild is notified by the child's setter, from the child's old position to its new one
        let grandchild = movers
            .step(
                "grandchild",
                pose(20.0, 0.0),
                carry("child", CarryShape::Position, [10.0, 0.0, 0.0], child.pose.position),
            )
            .unwrap();
        assert_eq!(grandchild.pose.position, moved.pose.position);
        // the DLL applied the same poses, so its next write is no unseen write
        movers.step("child", child.pose, Request::Stop).unwrap();
        movers.step("grandchild", grandchild.pose, PLACED).unwrap();
    }

    /// The other order: the campaign's own setter writes the parent after notifying, and places it afterwards. The host
    /// still holds the position the notification came from.
    #[test]
    fn a_child_follows_a_parent_the_campaign_is_about_to_write() {
        let mut movers = Movers::default();
        movers.bind("parent", Kind::Routed, pose(5.0, 5.0), None, None).unwrap();
        movers.bind("child", Kind::Routed, pose(10.0, 0.0), None, None).unwrap();
        let new = [6.0, 0.0, 7.0];
        let child = movers
            .step("child", pose(10.0, 0.0), carry("parent", CarryShape::Position, [5.0, 0.0, 5.0], new))
            .unwrap();
        assert_eq!(child.pose.position, new);
        // the campaign then writes and places the parent; the child's record is untouched by that
        movers.step("parent", pose(5.0, 5.0), PLACED).unwrap();
        movers.step("child", child.pose, PLACED).unwrap();
    }

    #[test]
    fn a_carry_the_host_cannot_vouch_for_is_refused_and_changes_nothing() {
        let mut movers = Movers::default();
        movers.bind("parent", Kind::Routed, pose(5.0, 5.0), None, None).unwrap();
        movers.bind("child", Kind::Routed, pose(10.0, 0.0), None, None).unwrap();
        movers.bind("ship", Kind::Ship, pose(0.0, 0.0), None, None).unwrap();
        movers.bind("gone", Kind::Routed, pose(0.0, 0.0), None, None).unwrap();
        movers.retire("gone").unwrap();
        let old = [5.0, 0.0, 5.0];
        let new = [6.0, 0.0, 7.0];
        let refused = |movers: &mut Movers, id: &str, request: Request, what: &str| {
            let error = movers.step(id, pose(10.0, 0.0), request).unwrap_err();
            assert!(error.contains(what), "{what}: {error}");
        };
        refused(&mut movers, "child", carry("child", CarryShape::Position, old, new), "is the child itself");
        refused(&mut movers, "child", carry("gone", CarryShape::Position, old, new), "is retired");
        refused(&mut movers, "child", carry("nobody", CarryShape::Position, old, new), "is not a host id");
        // the parent was not where the notification says it was: a move the host did not make
        refused(
            &mut movers,
            "child",
            carry("parent", CarryShape::Position, [9.0, 0.0, 9.0], new),
            "a move the host did not make",
        );
        // a non-finite input
        refused(
            &mut movers,
            "child",
            carry("parent", CarryShape::Position, old, [f32::NAN, 0.0, 0.0]),
            "non-finite",
        );
        // shapes the original handlers never had: a ship child, a helicopter shape for a routed child
        refused(
            &mut movers,
            "ship",
            carry("parent", CarryShape::Position, old, new),
            "is not a request for Ship",
        );
        refused(
            &mut movers,
            "child",
            carry("parent", CarryShape::Helicopter, old, new),
            "is not a request for Routed",
        );
        // an unknown child
        refused(&mut movers, "nobody", carry("parent", CarryShape::Position, old, new), "unknown host id");
        // nothing above moved the child, and it still agrees with the campaign
        movers.step("child", pose(10.0, 0.0), Request::Stop).unwrap();
    }

    /// The helicopter keeps its offset from the parent, turned by the parent's heading change: the original expressions,
    /// evaluated in the original's precision (doubles from the floats, narrowed on assignment).
    #[test]
    fn a_helicopter_child_is_carried_with_the_original_arithmetic() {
        let mut movers = Movers::default();
        movers.bind("parent", Kind::Routed, pose(0.0, 0.0), None, None).unwrap();
        movers.bind("child", Kind::Helicopter, pose(10.0, 3.0), Some([0.0; 3]), Some(ROTOR)).unwrap();
        let heading = 0.7f32;
        let (old, new) = ([0.0f32; 3], [1.5f32, 0.25, 2.5]);
        let request = Request::DependentCarry(DependentCarry {
            parent: "parent".to_owned(),
            shape: CarryShape::Helicopter,
            parent_old: old,
            parent_new: new,
            heading,
            velocity: 4.5,
        });
        let carried = movers.step("child", pose(10.0, 3.0), request).unwrap();
        // the child's own heading is level (zero), so the angle is the parent's heading alone
        let angle = f64::from(heading - 0.0f32);
        let (dx, dy, dz) = (f64::from(10.0f32 - old[0]), f64::from(0.0f32 - old[1]), f64::from(3.0f32 - old[2]));
        let expected = [
            (f64::from(new[0]) + (angle.cos() * dx + angle.sin() * dz)) as f32,
            (f64::from(new[1]) + dy) as f32,
            (f64::from(new[2]) + (angle.cos() * dz - angle.sin() * dx)) as f32,
        ];
        assert_eq!(carried.pose.position.map(f32::to_bits), expected.map(f32::to_bits));
        assert_eq!(carried.pose.velocity, 4.5);
        // the attitude is the parent's heading with the child's own (level) pitch and roll
        assert!((carried.pose.attitude[0] - heading.cos()).abs() < 1.0e-6, "{:?}", carried.pose.attitude);
        assert!(
            (carried.pose.attitude[6].abs() - heading.sin().abs()).abs() < 1.0e-6,
            "{:?}",
            carried.pose.attitude
        );
        // and the next request is checked against what the host answered
        movers.step("child", carried.pose, PLACED).unwrap();
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
        let own = |kind: Kind, request: &Request| request.allows(kind);
        for request in requests {
            for (kind, movers) in [
                (Kind::Routed, &mut routed),
                (Kind::Ship, &mut ship),
                (Kind::Person, &mut person),
                (Kind::FixedWing, &mut fixed_wing),
                (Kind::Helicopter, &mut helicopter),
            ] {
                let result = movers.step("v", pose(0.0, 0.0), request.clone());
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
        movers.step("v", pose(0.0, 0.0), placed.clone()).unwrap();
        // the host now flies at the placed 40 m/s, and the 100 m/s mirror is a write it did not make
        let mut flight = fixed_wing_move(FixedWingPhase::Flight);
        flight.elevation = 0.0;
        // the flight clamps the speed to 0.7 to 2 times the cruise velocity
        flight.cruise_velocity = 40.0;
        // the host's navigator now supplies the acceleration, and a guide speed of 40 asks for none
        flight.wp_vel = 40.0;
        let mut stale = flight.clone();
        stale.motion_vector = [0.0, 0.0, 100.0];
        let error = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(stale.clone())).unwrap_err();
        assert!(error.contains("motion_vector"), "{error}");
        flight.motion_vector = [0.0, 0.0, 40.0];
        let first = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(flight.clone())).unwrap();
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
        let first = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(taxi.clone())).unwrap();
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
        let first = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(flight.clone())).unwrap();
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
        let first = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(land.clone())).unwrap();
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
        let first = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(absolute.clone())).unwrap();
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
        let error = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(flight.clone())).unwrap_err();
        assert!(
            error.contains("motion_vector[2]") && error.contains("differs from the host's record"),
            "{error}"
        );
        // nothing moved and the host's vector is unchanged: the correct mirror still flies from it
        flight.motion_vector[2] = 100.0;
        let first = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(flight.clone())).unwrap();
        assert_near(first.pose.position, [0.0, 0.0, 50.0]);
    }

    #[test]
    fn the_next_step_flies_from_the_vector_the_host_answered() {
        let mut movers = movers(Kind::FixedWing, pose(0.0, 0.0));
        let mut taxi = fixed_wing_move(FixedWingPhase::Taxi);
        taxi.delta_seconds = 1.0;
        taxi.wp_vel = 10.0;
        let first = movers.step("v", pose(0.0, 0.0), Request::FixedWingMove(taxi.clone())).unwrap();
        // the stale mirror (the seed) is refused; the answered vector is accepted
        let error = movers.step("v", first.pose, Request::FixedWingMove(taxi.clone())).unwrap_err();
        assert!(error.contains("motion_vector"), "{error}");
        taxi.motion_vector = first.motion_vector;
        movers.step("v", first.pose, Request::FixedWingMove(taxi.clone())).unwrap();
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
            old_elevation: 0.0,
            bbox_diagonal: 0.0,
            wide: false,
            water: false,
            waypoint: [0.0; 3],
            death_timer: 0.0,
            death_range: 0.0,
            death_policy: 0,
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

    // ---- the routed, anti-air and ship lifecycle, and the fixed-wing death settle (EEDCS-37) ----

    /// the vector a test vehicle moves with: 5 m/s along a 3-4 diagonal
    const GROUND: [f32; 3] = [3.0, 0.0, 4.0];

    fn moving(y: f32) -> Pose {
        Pose {
            position: [0.0, y, 0.0],
            velocity: 7.0,
            attitude: LEVEL,
        }
    }

    /// a vehicle of `kind` that has claimed `vector`
    fn vehicle(kind: Kind, start: Pose, vector: [f32; 3]) -> Movers {
        let mut movers = movers(kind, start);
        let seeded = run(&mut movers, start, life(LifecycleMode::VehicleVectorSeed, [0.0; 3], vector)).unwrap();
        assert_eq!(seeded.motion_vector, vector);
        movers
    }

    fn vehicle_step(mode: LifecycleMode, vector: [f32; 3], delta_seconds: f32) -> Lifecycle {
        let mut step = life(mode, [0.0; 3], vector);
        step.delta_seconds = delta_seconds;
        step
    }

    fn heading_of(a: &[f32; 9]) -> f32 {
        a[6].atan2(a[8])
    }

    fn pitch_of(a: &[f32; 9]) -> f32 {
        a[7].asin()
    }

    fn roll_of(a: &[f32; 9]) -> f32 {
        (-a[1]).atan2(a[4])
    }

    /// a routed vehicle 10 m up, taken through its fall candidate and gravity (`gravity` for half a second) to its
    /// settle over a terrain at 10 m with the CG 1 m up: (movers, the start pose, the held vector, the settle's answer)
    fn settled_fall(water: bool, old_elevation: f32, bbox_diagonal: f32, gravity: f32) -> (Movers, Pose, [f32; 3], Outcome) {
        let start = moving(10.0);
        let mut movers = vehicle(Kind::Routed, start, GROUND);
        let staged = run(&mut movers, start, vehicle_step(LifecycleMode::GvFallCandidate, GROUND, 1.0)).unwrap();
        assert_within(&staged.pose.position, &[3.0, 10.0, 4.0], 1.0e-5);
        let mut pull = vehicle_step(LifecycleMode::GvFallGravity, GROUND, 0.5);
        pull.gravity = gravity;
        let pulled = run(&mut movers, start, pull).unwrap();
        let mut settle = vehicle_step(LifecycleMode::GvFallSettle, pulled.motion_vector, 0.5);
        settle.waypoint = [3.0, 10.0, 4.0];
        settle.water = water;
        settle.old_elevation = old_elevation;
        settle.elevation = 10.0;
        settle.cg_to_ground = 1.0;
        settle.bbox_diagonal = bbox_diagonal;
        let settled = run(&mut movers, start, settle).unwrap();
        (movers, start, pulled.motion_vector, settled)
    }

    #[test]
    fn a_vehicle_vector_is_claimed_once_and_only_live_movement_releases_it() {
        let start = moving(0.0);
        let mut movers = vehicle(Kind::Routed, start, GROUND);
        let again = run(&mut movers, start, life(LifecycleMode::VehicleVectorSeed, [0.0; 3], GROUND)).unwrap();
        assert_eq!(again.motion_vector, GROUND);
        let error = run(&mut movers, start, life(LifecycleMode::VehicleVectorSeed, [0.0; 3], [1.0, 2.0, 3.0])).unwrap_err();
        assert!(error.contains("motion_vector[0]"), "{error}");
        // a placement leaves the claim: the campaign's other vector is still a foreign one
        movers
            .step(
                "v",
                start,
                Request::Place {
                    motion_vector: [9.0, 9.0, 9.0],
                },
            )
            .unwrap();
        let error = run(&mut movers, start, life(LifecycleMode::VehicleVectorSeed, [0.0; 3], [9.0, 9.0, 9.0])).unwrap_err();
        assert!(error.contains("motion_vector[0]"), "{error}");
        let impact = run(&mut movers, start, life(LifecycleMode::GvImpact, [0.0; 3], GROUND)).unwrap();
        assert_eq!(impact.motion_vector, GROUND);
        // a vehicle that never claimed one cannot be stepped
        let mut unclaimed = self::movers(Kind::Routed, start);
        let error = run(&mut unclaimed, start, vehicle_step(LifecycleMode::GvDeathCandidate, GROUND, 1.0)).unwrap_err();
        assert!(error.contains("without a claimed vehicle vector"), "{error}");
        // live movement releases the claim: the next claim is a fresh one
        let stopped = movers.step("v", start, Request::Stop).unwrap();
        run(&mut movers, stopped.pose, life(LifecycleMode::VehicleVectorSeed, [0.0; 3], [1.0, 2.0, 3.0])).unwrap();
    }

    #[test]
    fn a_ground_fall_stages_its_results_and_only_the_commit_applies_them() {
        let (mut movers, start, held, settled) = settled_fall(false, 0.0, 5.0, 10.0);
        assert_within(&held, &[3.0, -5.0, 4.0], 1.0e-5);
        // solid ground: contact (1) and ground impact (8); the settled vector keeps no vertical part and the scalar is its length
        assert_eq!(settled.flags, 9);
        assert_within(&settled.pose.position, &[0.0, 11.0, 0.0], 1.0e-5);
        assert_within(&settled.motion_vector, &GROUND, 1.0e-5);
        assert_within(&[settled.pose.velocity], &[5.0], 1.0e-5);
        // nothing was applied: the commit before the vector is refused, and the refusal keeps the fall
        let error = run(&mut movers, start, vehicle_step(LifecycleMode::GvFallCommit, held, 0.5)).unwrap_err();
        assert!(error.contains("gv_fall_vector was not applied"), "{error}");
        let vectored = run(&mut movers, start, vehicle_step(LifecycleMode::GvFallVector, held, 0.5)).unwrap();
        assert_within(&vectored.motion_vector, &GROUND, 1.0e-5);
        let committed = run(&mut movers, start, vehicle_step(LifecycleMode::GvFallCommit, vectored.motion_vector, 0.5)).unwrap();
        assert_within(&committed.pose.position, &[0.0, 11.0, 0.0], 1.0e-5);
        assert_within(&[committed.pose.velocity], &[5.0], 1.0e-5);
        // the fall is over: a second commit is stale, and the vehicle can claim again the vector it ended with
        let error = run(
            &mut movers,
            committed.pose,
            vehicle_step(LifecycleMode::GvFallCommit, vectored.motion_vector, 0.5),
        )
        .unwrap_err();
        assert!(!error.is_empty());
        let seeded = run(
            &mut movers,
            committed.pose,
            life(LifecycleMode::VehicleVectorSeed, [0.0; 3], vectored.motion_vector),
        )
        .unwrap();
        assert_eq!(seeded.motion_vector, vectored.motion_vector);
    }

    #[test]
    fn an_open_fall_keeps_its_order_and_refuses_every_other_step() {
        let start = moving(10.0);
        let mut movers = vehicle(Kind::Routed, start, GROUND);
        run(&mut movers, start, vehicle_step(LifecycleMode::GvFallCandidate, GROUND, 1.0)).unwrap();
        for mode in [
            LifecycleMode::GvDeathCandidate,
            LifecycleMode::GvImpact,
            LifecycleMode::VehicleVectorSeed,
            LifecycleMode::GvFallCandidate,
        ] {
            assert!(run(&mut movers, start, vehicle_step(mode, GROUND, 1.0)).is_err(), "{mode:?}");
        }
        assert!(movers.step("v", start, Request::Stop).is_err());
        // a later phase before its turn is refused, and the fall is still at its first stage
        let mut settle = vehicle_step(LifecycleMode::GvFallSettle, GROUND, 1.0);
        settle.waypoint = [3.0, 10.0, 4.0];
        assert!(run(&mut movers, start, settle).is_err());
        assert!(run(&mut movers, start, vehicle_step(LifecycleMode::GvFallVector, GROUND, 1.0)).is_err());
        run(&mut movers, start, vehicle_step(LifecycleMode::GvFallGravity, GROUND, 1.0)).unwrap();
    }

    #[test]
    fn a_water_fall_splashes_and_a_deep_one_sinks_with_nothing_left_to_move() {
        // water at 10 m under a vehicle that was 10 m up over a terrain at 5 m: contact (1) and splash (2); it keeps its x and z
        let (_, _, _, splashed) = settled_fall(true, 5.0, 5.0, 0.0);
        assert_eq!(splashed.flags, 3);
        assert_within(&splashed.pose.position, &[0.0, 10.0, 0.0], 1.0e-5);
        assert_within(&splashed.motion_vector, &GROUND, 1.0e-5);
        // a bounding box shorter than the depth sinks it: the vector is zeroed (sunk, 4)
        let (_, _, _, sunk) = settled_fall(true, 5.0, 0.5, 0.0);
        assert_eq!(sunk.flags, 7);
        assert_eq!(sunk.motion_vector, [0.0; 3]);
        // a settle whose candidate mirror differs is refused before anything is staged, and the fall can still settle
        let start = moving(10.0);
        let mut movers = vehicle(Kind::Routed, start, GROUND);
        run(&mut movers, start, vehicle_step(LifecycleMode::GvFallCandidate, GROUND, 1.0)).unwrap();
        let pulled = run(&mut movers, start, vehicle_step(LifecycleMode::GvFallGravity, GROUND, 0.5)).unwrap();
        let mut settle = vehicle_step(LifecycleMode::GvFallSettle, pulled.motion_vector, 0.5);
        settle.waypoint = [3.0, 10.0, 5.0];
        assert!(run(&mut movers, start, settle).is_err());
        settle.waypoint = [3.0, 10.0, 4.0];
        run(&mut movers, start, settle).unwrap();
    }

    #[test]
    fn a_death_slide_decays_the_speed_and_lands_on_the_terrain_at_its_candidate() {
        for wide in [false, true] {
            let start = moving(10.0);
            let mut movers = vehicle(Kind::Routed, start, GROUND);
            let staged = run(&mut movers, start, vehicle_step(LifecycleMode::GvDeathCandidate, GROUND, 1.0)).unwrap();
            // the candidate is x and z only; the height waits for the terrain
            assert_within(&staged.pose.position, &[3.0, 10.0, 4.0], 1.0e-5);
            let mut decay = vehicle_step(LifecycleMode::GvDeathDecay, GROUND, 1.0);
            decay.waypoint = [3.0, 0.0, 4.0];
            decay.elevation = 20.0;
            decay.cg_to_ground = 2.0;
            decay.wide = wide;
            let decayed = run(&mut movers, start, decay).unwrap();
            // 5 m/s less 4 m/s each second, along the same direction
            assert_within(&decayed.pose.position, &[3.0, 22.0, 4.0], 1.0e-5);
            assert_within(&decayed.motion_vector, &[0.6, 0.0, 0.8], 1.0e-5);
            assert_within(&[decayed.pose.velocity], &[1.0], 1.0e-5);
            // a second second takes the rest and the speed stops at zero
            run(
                &mut movers,
                decayed.pose,
                vehicle_step(LifecycleMode::GvDeathCandidate, decayed.motion_vector, 1.0),
            )
            .unwrap();
            let mut last = vehicle_step(LifecycleMode::GvDeathDecay, decayed.motion_vector, 1.0);
            last.waypoint = [3.6, 0.0, 4.8];
            last.elevation = 20.0;
            last.cg_to_ground = 2.0;
            last.wide = wide;
            let stopped = run(&mut movers, decayed.pose, last).unwrap();
            assert_eq!(stopped.pose.velocity, 0.0);
            assert_within(&stopped.motion_vector, &[0.0; 3], 1.0e-6);
        }
    }

    #[test]
    fn the_decay_and_the_sink_turn_the_attitude_the_host_holds_and_the_dll_cannot_set_it() {
        let mut movers = vehicle(Kind::Routed, moving(10.0), GROUND);
        let mut pose = moving(10.0);
        let mut vector = GROUND;
        for turn in 1..=2 {
            run(&mut movers, pose, vehicle_step(LifecycleMode::GvDeathCandidate, vector, 0.5)).unwrap();
            let mut decay = vehicle_step(LifecycleMode::GvDeathDecay, vector, 0.5);
            decay.waypoint = [pose.position[0] + vector[0] * 0.5, 0.0, pose.position[2] + vector[2] * 0.5];
            let decayed = run(&mut movers, pose, decay).unwrap();
            // the heading is the held one less a tenth of a half-turn each time, accumulating on what the host kept
            assert_within(
                &[heading_of(&decayed.pose.attitude)],
                &[-0.1 * std::f32::consts::PI * 0.5 * turn as f32],
                1.0e-5,
            );
            assert_within(&[pitch_of(&decayed.pose.attitude), roll_of(&decayed.pose.attitude)], &[0.0, 0.0], 1.0e-5);
            pose = decayed.pose;
            vector = decayed.motion_vector;
        }
        // a sinking ship leaves its heading and drifts its pitch and roll from the held attitude
        let ship = moving(0.0);
        let mut movers = vehicle(Kind::Ship, ship, [0.0, 0.0, 1.0]);
        let mut pose = ship;
        let mut vector = [0.0, 0.0, 1.0];
        for sunk in 1..=2 {
            let sank = run(&mut movers, pose, vehicle_step(LifecycleMode::ShipSink, vector, 1.0)).unwrap();
            let attitude = sank.pose.attitude;
            assert_within(&[heading_of(&attitude)], &[0.0], 1.0e-5);
            assert_within(&[pitch_of(&attitude)], &[0.1f32.to_radians() * sunk as f32], 1.0e-5);
            assert_within(&[roll_of(&attitude)], &[0.5f32.to_radians() * sunk as f32], 1.0e-5);
            pose = sank.pose;
            vector = sank.motion_vector;
        }
    }

    #[test]
    fn a_pending_death_slide_refuses_everything_but_its_decay_and_a_placement() {
        let start = moving(10.0);
        let mut movers = vehicle(Kind::Routed, start, GROUND);
        run(&mut movers, start, vehicle_step(LifecycleMode::GvDeathCandidate, GROUND, 1.0)).unwrap();
        // none of these may overwrite or abandon the staged candidate (x 3, z 4)
        for mode in [
            LifecycleMode::GvDeathCandidate,
            LifecycleMode::GvImpact,
            LifecycleMode::GvFallCandidate,
            LifecycleMode::VehicleVectorSeed,
        ] {
            assert!(run(&mut movers, start, vehicle_step(mode, GROUND, 2.0)).is_err(), "{mode:?}");
        }
        assert!(movers.step("v", start, Request::Stop).is_err());
        assert!(movers.step("v", start, Request::Advance(advance([0.0, 0.0, 100.0]))).is_err());
        // a decay for another candidate is refused too
        let mut decay = vehicle_step(LifecycleMode::GvDeathDecay, GROUND, 1.0);
        decay.waypoint = [6.0, 0.0, 8.0];
        decay.elevation = 20.0;
        decay.cg_to_ground = 2.0;
        assert!(run(&mut movers, start, decay).is_err());
        // a placement keeps the candidate, and the decay then lands on the first one rather than any refused replacement
        movers.step("v", start, Request::Place { motion_vector: GROUND }).unwrap();
        decay.waypoint = [3.0, 0.0, 4.0];
        let decayed = run(&mut movers, start, decay).unwrap();
        assert_within(&decayed.pose.position, &[3.0, 22.0, 4.0], 1.0e-5);
        // and with the slide ended the vehicle accepts a new one
        run(
            &mut movers,
            decayed.pose,
            vehicle_step(LifecycleMode::GvDeathCandidate, decayed.motion_vector, 1.0),
        )
        .unwrap();
    }

    #[test]
    fn the_vehicle_steps_belong_to_their_own_kinds() {
        for (kind, mode) in [
            (Kind::Ship, LifecycleMode::GvDeathCandidate),
            (Kind::Helicopter, LifecycleMode::GvImpact),
            (Kind::FixedWing, LifecycleMode::VehicleVectorSeed),
            (Kind::Routed, LifecycleMode::ShipSink),
            (Kind::Routed, LifecycleMode::FwDeathAttitude),
            (Kind::Ship, LifecycleMode::FwSettle),
        ] {
            assert!(!mode.allows(kind));
            let mut movers = movers(kind, pose(0.0, 0.0));
            assert!(run(&mut movers, pose(0.0, 0.0), life(mode, ROTOR, SEED)).is_err(), "{kind:?} {mode:?}");
        }
        assert!(LifecycleMode::ShipSink.allows(Kind::Ship) && LifecycleMode::VehicleVectorSeed.allows(Kind::Ship));
    }

    /// a fixed-wing aircraft taken through its fall and gravity to its contact: (movers, the gravity's answer, the contact altitude)
    fn fallen_wing(start: Pose, delta_seconds: f32) -> (Movers, Outcome, f32) {
        let mut movers = movers(Kind::FixedWing, start);
        run(&mut movers, start, vehicle_step(LifecycleMode::Fall, SEED, delta_seconds)).unwrap();
        let pulled = run(&mut movers, start, vehicle_step(LifecycleMode::FallGravity, SEED, delta_seconds)).unwrap();
        let mut contact = vehicle_step(LifecycleMode::Contact, pulled.motion_vector, delta_seconds);
        contact.flag = true;
        let touched = run(&mut movers, pulled.pose, contact).unwrap();
        (movers, pulled, touched.altitude)
    }

    fn death_attitude(vector: [f32; 3], altitude: f32, speed: f32, flag: bool, draw: f64, delta_seconds: f32) -> Lifecycle {
        let mut step = vehicle_step(LifecycleMode::FwDeathAttitude, vector, delta_seconds);
        step.elevation = altitude;
        step.signed_speed = speed;
        step.flag = flag;
        step.draws = [0.0, draw];
        step
    }

    #[test]
    fn a_wing_death_attitude_checks_the_campaigns_gate_and_mirrors_and_then_commits_the_candidate() {
        let start = at(10.0, 100.0, 20.0);
        let (mut movers, pulled, altitude) = fallen_wing(start, 0.5);
        let speed = pulled.pose.velocity;
        // the campaign's gate disagrees with the aircraft's altitude: -1, and nothing is consumed
        let gate = run(
            &mut movers,
            pulled.pose,
            death_attitude(pulled.motion_vector, altitude, speed, altitude <= 12.0, 0.0, 0.5),
        )
        .unwrap();
        assert_eq!(gate.flags, -1);
        assert_eq!(gate.pose, pulled.pose);
        // an altitude or speed mirror that differs is refused before anything changes
        assert!(run(
            &mut movers,
            pulled.pose,
            death_attitude(pulled.motion_vector, altitude - 1.0, speed, true, 0.5, 0.5)
        )
        .is_err());
        assert!(run(
            &mut movers,
            pulled.pose,
            death_attitude(pulled.motion_vector, altitude, speed + 1.0, true, 0.5, 0.5)
        )
        .is_err());
        // the right step eases the attitude with its draw and ends at the staged candidate
        let eased = run(&mut movers, pulled.pose, death_attitude(pulled.motion_vector, altitude, speed, true, 0.5, 0.5)).unwrap();
        assert_eq!(eased.flags & 3, 3);
        assert_ne!(eased.pose.attitude, start.attitude);
        // the candidate was consumed: the same step cannot be made again
        assert!(run(&mut movers, eased.pose, death_attitude(pulled.motion_vector, altitude, speed, true, 0.5, 0.5)).is_err());
    }

    /// The fixed-wing roll draw is `frand1`'s double and the original forms `PI * 0.2 * draw * dt` in double before narrowing
    /// the sum: a draw narrowed to float first is a different roll for some draws, and the step must use the draw it received.
    #[test]
    fn the_wing_death_attitude_forms_its_roll_draw_in_double_end_to_end() {
        const DT: f32 = 1.0 / 30.0;
        let start = at(10.0, 100.0, 20.0);
        let direct = |draw: f64, speed: f32| {
            let mut attitude = start.attitude;
            // SAFETY: the attitude is 9 floats read then written and the vector 3.
            unsafe { eech_fw_death_attitude(attitude.as_mut_ptr(), SEED.as_ptr(), speed, 100.0, 1, draw, DT) };
            attitude
        };
        let mut checked = 0;
        for k in 1..=2000u32 {
            let draw = f64::from(k) * (1.0 / 32767.0);
            let (mut movers, pulled, altitude) = fallen_wing(start, DT);
            let speed = pulled.pose.velocity;
            if direct(draw, speed) == direct(f64::from(draw as f32), speed) {
                continue;
            }
            let died = run(&mut movers, pulled.pose, death_attitude(pulled.motion_vector, altitude, speed, true, draw, DT)).unwrap();
            assert_eq!(died.flags & 3, 3);
            assert_eq!(died.pose.attitude, direct(draw, speed), "draw {k}: the step did not use the draw it was given");
            checked += 1;
        }
        assert!(checked >= 50, "only {checked} draws separate the float transport from the double one");
    }

    #[test]
    fn a_wing_crash_slide_keeps_its_post_slope_speed_through_the_clear_and_commits_the_staged_candidate() {
        let start = Pose {
            position: [0.0, 50.0, 0.0],
            velocity: 0.6,
            attitude: LEVEL,
        };
        let slow = [0.6, 0.0, 0.0];
        let mut movers = movers(Kind::FixedWing, start);
        movers.step("v", start, Request::Place { motion_vector: slow }).unwrap();
        let step = |mode, vector| {
            let mut step = vehicle_step(mode, vector, 0.01);
            step.elevation = 30.0;
            step.cg_to_ground = 2.0;
            step
        };
        let candidate = run(&mut movers, start, step(LifecycleMode::FwCrashCandidate, slow)).unwrap();
        assert_within(&candidate.pose.position, &[0.006, 50.0, 0.0], 1.0e-6);
        // out of order, interleaved or repeated requests are refused, and the candidate survives them all
        for refused in [
            LifecycleMode::FwCrashAttitude,
            LifecycleMode::FwCrashCandidate,
            LifecycleMode::FwSettle,
            LifecycleMode::Fall,
        ] {
            assert!(run(&mut movers, start, step(refused, slow)).is_err(), "{refused:?}");
        }
        assert!(movers.step("v", start, Request::Stop).is_err());
        // flat ground: the friction (10 * 0.01) leaves 0.5, under 1, so the velocity phase asks the campaign to clear
        let velocity = run(&mut movers, start, step(LifecycleMode::FwCrashVelocity, slow)).unwrap();
        assert_eq!(velocity.flags, 1);
        assert_within(&[velocity.pose.velocity], &[0.5], 1.0e-6);
        assert_within(&velocity.motion_vector, &[0.5, 0.0, 0.0], 1.0e-6);
        assert_within(&velocity.pose.position, &[0.006, 32.0, 0.0], 1.0e-6);
        assert!(run(&mut movers, start, step(LifecycleMode::FwCrashVelocity, velocity.motion_vector)).is_err());
        // the campaign's clear is a placement of zero; the post-slope speed is still the one the rest test sees
        let cleared = Pose { velocity: 0.0, ..start };
        movers.step("v", cleared, Request::Place { motion_vector: [0.0; 3] }).unwrap();
        let landed = run(&mut movers, cleared, step(LifecycleMode::FwCrashAttitude, [0.0; 3])).unwrap();
        assert_eq!(landed.flags, 0, "a 0.5 m/s slide is not at rest, whatever the cleared scalar says");
        assert_within(&landed.pose.position, &[0.006, 32.0, 0.0], 1.0e-6);
        // the slide is closed: a crash phase now needs a new candidate, and a live step is accepted again
        assert!(run(&mut movers, landed.pose, step(LifecycleMode::FwCrashAttitude, [0.0; 3])).is_err());
        run(&mut movers, landed.pose, step(LifecycleMode::FwCrashCandidate, [0.0; 3])).unwrap();
    }

    fn expiry(timer: f32, delta: f32, range: f32, policy: i32) -> Lifecycle {
        let mut step = life(LifecycleMode::Expire, ROTOR, SEED);
        step.death_timer = timer;
        step.delta_seconds = delta;
        step.death_range = range;
        step.death_policy = policy;
        step
    }

    #[test]
    fn a_wreck_ages_by_the_frame_and_ends_only_on_a_gated_frame_past_the_ranges_threshold() {
        let start = pose(0.0, 0.0);
        let mut movers = movers(Kind::Routed, start);
        // the first copy is adopted (the campaign zeroes the timer at destruction) and then the host holds it
        let first = run(&mut movers, start, expiry(0.0, 0.5, 0.0, 0)).unwrap();
        assert_eq!((first.death_timer, first.flags), (0.5, 0));
        let second = run(&mut movers, start, expiry(first.death_timer, 0.5, 0.0, 0)).unwrap();
        assert_eq!((second.death_timer, second.flags), (1.0, 0));

        // within 1000 m the threshold is 300 s, but the gate (int(timer + 1) % 300 == 0) opens only for [299, 300)
        let mut near = Movers::default();
        near.bind("v", Kind::Routed, start, None, None).unwrap();
        run(&mut near, start, expiry(298.6, 0.0, 0.0, 0)).unwrap();
        let before = run(&mut near, start, expiry(298.6, 0.5, 0.0, DEATH_RANGE_SUPPLIED)).unwrap();
        assert_eq!((before.death_timer, before.flags), (299.1, 0), "gated but under 300 s");
        let past = run(&mut near, start, expiry(before.death_timer, 1.0, 0.0, 0)).unwrap();
        assert_eq!((past.death_timer, past.flags), (300.1, 0), "over the threshold but off the gate");

        // the next gate window is [599, 600): over the threshold there, the life is over
        let mut late = Movers::default();
        late.bind("v", Kind::Routed, start, None, None).unwrap();
        run(&mut late, start, expiry(598.9, 0.0, 0.0, 0)).unwrap();
        let ended = run(&mut late, start, expiry(598.9, 0.5, 0.0, DEATH_RANGE_SUPPLIED)).unwrap();
        assert_eq!((ended.death_timer, ended.flags), (599.4, 1));
    }

    #[test]
    fn the_threshold_follows_the_squared_range_to_the_nearest_keysite() {
        let curve = |range: f32| unsafe { eech_expire_threshold(range) };
        assert_eq!(curve(0.0), 300.0);
        assert_eq!(curve(1.0e6), 300.0);
        assert_eq!(curve(1.0e8), 10800.0);
        assert_eq!(curve(5.0e9), 10800.0);
        // the engine's unparenthesised divisor is 10000 * 10000 / 1000 * 1000 = 1e8, not 100
        assert_within(&[curve(5.0e7)], &[5445.0], 1.0e-2);

        // at 5e7 the gate window [5399, 5400) is under the 5445 s threshold, [5699, 5700) is over it
        let start = pose(0.0, 0.0);
        let mut movers = movers(Kind::Routed, start);
        run(&mut movers, start, expiry(5399.2, 0.0, 5.0e7, DEATH_RANGE_SUPPLIED)).unwrap();
        assert_eq!(run(&mut movers, start, expiry(5399.2, 0.5, 5.0e7, DEATH_RANGE_SUPPLIED)).unwrap().flags, 0);
        let mut over = Movers::default();
        over.bind("v", Kind::Routed, start, None, None).unwrap();
        run(&mut over, start, expiry(5699.0, 0.0, 5.0e7, DEATH_RANGE_SUPPLIED)).unwrap();
        assert_eq!(run(&mut over, start, expiry(5699.0, 0.5, 5.0e7, DEATH_RANGE_SUPPLIED)).unwrap().flags, 1);
    }

    #[test]
    fn an_occupied_helicopter_restarts_its_timer_and_anti_air_never_ends() {
        let start = pose(0.0, 0.0);
        let mut player = Movers::default();
        player.bind("v", Kind::Helicopter, start, Some(SEED), Some(ROTOR)).unwrap();
        run(&mut player, start, expiry(598.9, 0.0, 0.0, DEATH_OCCUPIED)).unwrap();
        let reset = run(&mut player, start, expiry(598.9, 0.5, 0.0, DEATH_OCCUPIED | DEATH_RANGE_SUPPLIED)).unwrap();
        assert_eq!((reset.death_timer, reset.flags), (0.0, 0));
        // the reset is the host's answer: the next frame ages from zero
        let aged = run(&mut player, start, expiry(reset.death_timer, 0.5, 0.0, DEATH_OCCUPIED)).unwrap();
        assert_eq!((aged.death_timer, aged.flags), (0.5, 0));

        let mut guns = Movers::default();
        guns.bind("v", Kind::Routed, start, None, None).unwrap();
        run(&mut guns, start, expiry(598.9, 0.0, 0.0, DEATH_NEVER_EXPIRES)).unwrap();
        let kept = run(&mut guns, start, expiry(598.9, 0.5, 0.0, DEATH_NEVER_EXPIRES)).unwrap();
        assert_eq!((kept.death_timer, kept.flags), (599.4, 0), "ages, but is never told its life is over");
    }

    #[test]
    fn a_ship_ends_when_it_has_sunk_half_its_diagonal_under_the_waterline() {
        let ship = |height: f32, diagonal: f32| {
            let start = at(0.0, height, 0.0);
            let mut movers = movers(Kind::Ship, start);
            let mut step = expiry(0.0, 0.5, 0.0, 0);
            step.bbox_diagonal = diagonal;
            run(&mut movers, start, step).unwrap()
        };
        assert_eq!(ship(-5.9, 12.0).flags, 0);
        assert_eq!(ship(-6.0, 12.0).flags, 0, "exactly half the diagonal is not below it");
        assert_eq!(ship(-6.1, 12.0).flags, 1);
        assert_eq!(ship(0.0, 12.0).flags, 0);
        assert_eq!(ship(0.0, 12.0).death_timer, 0.0, "a ship has no timer");
    }

    #[test]
    fn a_refused_expiry_changes_nothing_and_a_retired_wreck_is_never_answered() {
        let start = pose(0.0, 0.0);
        let mut movers = movers(Kind::Routed, start);
        run(&mut movers, start, expiry(10.0, 0.5, 0.0, 0)).unwrap();
        // a timer the host did not make is refused, and the record still holds what it did
        let error = run(&mut movers, start, expiry(11.0, 0.5, 0.0, 0)).unwrap_err();
        assert!(error.contains("death_timer"), "{error}");
        assert_eq!(run(&mut movers, start, expiry(10.5, 0.5, 0.0, 0)).unwrap().death_timer, 11.0);

        // an open fall means the wreck has not come to rest: the lifetime is not asked for yet
        let airborne = moving(10.0);
        let mut falling = vehicle(Kind::Routed, airborne, GROUND);
        run(&mut falling, airborne, vehicle_step(LifecycleMode::GvFallCandidate, GROUND, 1.0)).unwrap();
        let error = run(&mut falling, airborne, expiry(0.0, 0.5, 0.0, 0)).unwrap_err();
        assert!(error.contains("still open"), "{error}");

        movers.retire("v").unwrap();
        let error = run(&mut movers, start, expiry(11.0, 0.5, 0.0, 0)).unwrap_err();
        assert!(error.contains("retired"), "{error}");
    }

    #[test]
    fn the_keysite_range_is_asked_for_only_on_a_gated_frame_and_the_ask_changes_nothing() {
        let start = pose(0.0, 0.0);
        let mut movers = movers(Kind::Routed, start);
        // off the gate (int(timer + 1) is not a multiple of 300) the host never asks, whatever the range bit says
        for (timer, delta) in [(0.0, 0.5), (1.0, 0.5), (297.0, 0.5), (298.6, 0.0), (299.1, 1.0), (300.1, 0.5)] {
            let mut quiet = Movers::default();
            quiet.bind("v", Kind::Routed, start, None, None).unwrap();
            let answer = run(&mut quiet, start, expiry(timer, delta, 0.0, 0)).unwrap();
            assert_eq!(answer.flags, 0, "{timer}+{delta} is off the gate");
        }
        // on the gate it asks, and has neither aged nor recorded the timer, so the same step can be repeated
        run(&mut movers, start, expiry(598.9, 0.0, 0.0, 0)).unwrap();
        let asked = run(&mut movers, start, expiry(598.9, 0.5, 0.0, 0)).unwrap();
        assert_eq!((asked.flags, asked.death_timer), (EXPIRE_NEEDS_RANGE, 598.9));
        let again = run(&mut movers, start, expiry(598.9, 0.5, 0.0, 0)).unwrap();
        assert_eq!(again.flags, EXPIRE_NEEDS_RANGE);
        // supplied, the same step is answered and aged: 599.4 s is over the 300 s threshold at zero range
        let answered = run(&mut movers, start, expiry(598.9, 0.5, 0.0, DEATH_RANGE_SUPPLIED)).unwrap();
        assert_eq!((answered.flags, answered.death_timer), (1, 599.4));
        // a wreck that never expires does not ask for a range it would not use
        let mut guns = Movers::default();
        guns.bind("v", Kind::Routed, start, None, None).unwrap();
        run(&mut guns, start, expiry(598.9, 0.0, 0.0, DEATH_NEVER_EXPIRES)).unwrap();
        assert_eq!(run(&mut guns, start, expiry(598.9, 0.5, 0.0, DEATH_NEVER_EXPIRES)).unwrap().flags, 0);
    }

    #[test]
    fn an_expiry_the_gate_cannot_convert_is_refused_before_anything_is_kept() {
        let start = pose(0.0, 0.0);
        let mut movers = movers(Kind::Routed, start);
        run(&mut movers, start, expiry(10.0, 0.5, 0.0, 0)).unwrap();
        let ranged = DEATH_RANGE_SUPPLIED;
        let refused = [
            ("negative delta", expiry(10.5, -1.0, 0.0, 0), "nonnegative"),
            ("nan delta", expiry(10.5, f32::NAN, 0.0, 0), "nonnegative"),
            ("infinite delta", expiry(10.5, f32::INFINITY, 0.0, 0), "nonnegative"),
            ("negative timer", expiry(-1.0, 0.5, 0.0, 0), "nonnegative"),
            ("nan timer", expiry(f32::NAN, 0.5, 0.0, 0), "nonnegative"),
            ("overflowing delta", expiry(10.5, f32::MAX, 0.0, 0), "past the gate's range"),
            ("out of int range delta", expiry(10.5, 3.0e9, 0.0, 0), "past the gate's range"),
            ("unknown policy bit", expiry(10.5, 0.5, 0.0, 8), "death_policy"),
            ("negative policy", expiry(10.5, 0.5, 0.0, -1), "death_policy"),
            ("nan range", expiry(10.5, 0.5, f32::NAN, ranged), "keysite range"),
            ("negative range", expiry(10.5, 0.5, -1.0, ranged), "keysite range"),
        ];
        for (name, step, expected) in refused {
            let error = run(&mut movers, start, step).unwrap_err();
            assert!(error.contains(expected), "{name}: {error}");
        }
        // none of it reached the record: the host still holds 10.5 and ages from there
        assert_eq!(run(&mut movers, start, expiry(10.5, 0.5, 0.0, 0)).unwrap().death_timer, 11.0);
        // an unsupplied range may be anything: it is not read
        assert_eq!(run(&mut movers, start, expiry(11.0, 0.5, f32::NAN, 0)).unwrap().death_timer, 11.5);
    }

    /// the subject at the origin already flying at the cruise speed: the collision test's range is the host's own speed
    fn cruising() -> Pose {
        Pose {
            velocity: 100.0,
            ..pose(0.0, 0.0)
        }
    }

    /// the subject "v" beside a fixed-wing "leader" and "b", a helicopter "h" and a retired fixed-wing "r"
    fn fleet(leader_motion: [f32; 3], b: Pose, b_motion: [f32; 3]) -> Movers {
        let mut movers = movers(Kind::FixedWing, cruising());
        movers.bind("leader", Kind::FixedWing, pose(0.0, 500.0), Some(leader_motion), None).unwrap();
        movers.bind("b", Kind::FixedWing, b, Some(b_motion), None).unwrap();
        movers
            .bind("h", Kind::Helicopter, pose(0.0, 800.0), Some([0.0, 0.0, 10.0]), Some(ROTOR))
            .unwrap();
        movers.bind("r", Kind::FixedWing, pose(0.0, 900.0), Some([0.0, 0.0, 100.0]), None).unwrap();
        movers.retire("r").unwrap();
        movers
    }

    fn quiet_fleet() -> Movers {
        fleet([0.0, 0.0, 100.0], pose(0.0, 100_000.0), [0.0, 0.0, 100.0])
    }

    fn member(host_id: &str) -> NavigationMember {
        NavigationMember {
            host_id: Some(host_id.to_string()),
            avoidance_radius: 20.0,
        }
    }

    /// a non-leader of its group that pursues "leader" and avoids the named members
    fn pursuing(members: &[&str]) -> FixedWingNavigation {
        FixedWingNavigation {
            group_leader: false,
            task_leader: false,
            leader: Some("leader".to_string()),
            members: members.iter().map(|id| member(id)).collect(),
            ..task_leader_navigation()
        }
    }

    fn flight_with(navigation: FixedWingNavigation) -> Request {
        let mut flight = fixed_wing_move(FixedWingPhase::Flight);
        flight.navigation = Some(navigation);
        Request::FixedWingMove(flight)
    }

    /// the cruising subject at the origin looks ahead along +z: its speed plus ten metres
    const LOOKAHEAD: [f32; 2] = [0.0, 110.0];

    fn terrain_at(min_height: f32, guide_position: [f32; 3], sample_elevation: f32) -> FixedWingTerrain {
        FixedWingTerrain {
            min_height,
            guide_position,
            sample_point: LOOKAHEAD,
            sample_elevation,
            site_y: 0.0,
            cg: 0.0,
        }
    }

    /// the waypoint the host leaves a flight step with, after shaping, for a campaign waypoint of `waypoint`
    fn shaped_waypoint(movers: &Movers, terrain: Option<FixedWingTerrain>, waypoint: [f32; 3]) -> Result<[f32; 3], String> {
        let mut step = fixed_wing_move(FixedWingPhase::Flight);
        step.waypoint = waypoint;
        step.terrain = terrain;
        let mut request = Request::FixedWingMove(step);
        movers.shape_fixed_wing_waypoint("v", &mut request)?;
        match request {
            Request::FixedWingMove(step) => Ok(step.waypoint),
            _ => panic!("the request changed kind"),
        }
    }

    fn height_of(terrain: FixedWingTerrain) -> f32 {
        shaped_waypoint(&movers(Kind::FixedWing, cruising()), Some(terrain), [0.0, 12345.0, 5000.0]).unwrap()[1]
    }

    #[test]
    fn the_lookahead_query_answers_the_hosts_own_point_and_changes_nothing() {
        let mut movers = movers(Kind::FixedWing, cruising());
        let outcome = movers.step("v", cruising(), Request::FixedWingLookahead).unwrap();
        let Some(HelicopterAnswer::Query(query)) = movers.take_answer() else {
            panic!("no query was answered")
        };
        assert_eq!((query.terrain_sample_count, query.terrain_sample[0]), (1, LOOKAHEAD));
        assert!(!query.structure_scan);
        assert_eq!(outcome.pose.position, cruising().position);
        assert_eq!(outcome.pose.attitude, cruising().attitude);
        assert_eq!(outcome.motion_vector, [0.0, 0.0, 100.0]);
        // asking again gives the same answer: nothing was retained or advanced
        movers.step("v", cruising(), Request::FixedWingLookahead).unwrap();
        let Some(HelicopterAnswer::Query(again)) = movers.take_answer() else {
            panic!("no query was answered")
        };
        assert_eq!(again.terrain_sample[0], LOOKAHEAD);
    }

    #[test]
    fn the_lookahead_point_follows_the_hosts_pose_not_the_campaigns() {
        let mut flying = cruising();
        flying.position = [10.0, 0.0, 20.0];
        flying.velocity = 40.0;
        let mut movers = movers(Kind::FixedWing, flying);
        // the campaign's mirror is the same pose; the host's record fixes the point
        movers.step("v", flying, Request::FixedWingLookahead).unwrap();
        let Some(HelicopterAnswer::Query(query)) = movers.take_answer() else {
            panic!("no query was answered")
        };
        assert_eq!(query.terrain_sample[0], [10.0, 70.0]);
    }

    #[test]
    fn the_lookahead_query_is_for_fixed_wing_subjects_only() {
        let mut helicopter = movers(Kind::Helicopter, cruising());
        assert!(helicopter.step("v", cruising(), Request::FixedWingLookahead).is_err());
        let mut vehicle = movers(Kind::Routed, pose(0.0, 0.0));
        assert!(vehicle.step("v", pose(0.0, 0.0), Request::FixedWingLookahead).is_err());
    }

    #[test]
    fn the_host_climbs_to_the_terrain_when_the_look_ahead_height_is_inside_the_range() {
        // sampled terrain 20 and a minimum of 30: the look-ahead target is 50 high, 110 ahead, the waypoint 5000 away
        let height = height_of(terrain_at(30.0, [0.0, 0.0, 5000.0], 20.0));
        assert!((height - 5000.0 * 50.0 / 110.0).abs() < 1.0e-2, "{height}");
        // descending is the same arithmetic
        let down = height_of(terrain_at(0.0, [0.0, 0.0, 5000.0], -50.0));
        assert!((down + 5000.0 * 50.0 / 110.0).abs() < 1.0e-2, "{down}");
    }

    #[test]
    fn the_host_sets_the_target_height_when_the_look_ahead_is_steeper_than_it_is_far() {
        assert_eq!(height_of(terrain_at(30.0, [0.0, 0.0, 5000.0], 170.0)), 200.0);
        // exactly as steep as far is not "farther than it is high"
        let level = height_of(terrain_at(0.0, [0.0, 0.0, 5000.0], 110.0));
        assert!(level.is_finite());
    }

    #[test]
    fn the_host_sets_the_target_height_when_the_waypoint_is_within_a_metre() {
        assert_eq!(height_of(terrain_at(30.0, [0.0, 7.0, 0.5], 20.0)), 50.0);
    }

    #[test]
    fn a_minimum_height_of_minus_one_leaves_the_campaigns_guide_height() {
        assert_eq!(height_of(terrain_at(-1.0, [0.0, 123.0, 5000.0], 20.0)), 123.0);
    }

    #[test]
    fn a_look_ahead_that_is_the_aircraft_itself_leaves_the_campaigns_guide_height() {
        let mut still = cruising();
        still.attitude = [0.0; 9];
        let movers = movers(Kind::FixedWing, still);
        let terrain = FixedWingTerrain {
            sample_point: [0.0, 0.0],
            ..terrain_at(0.0, [0.0, 321.0, 5000.0], 0.0)
        };
        let waypoint = shaped_waypoint(&movers, Some(terrain), [0.0, 12345.0, 5000.0]).unwrap();
        assert_eq!(waypoint[1], 321.0);
    }

    #[test]
    fn the_formation_site_and_the_centre_of_gravity_are_added_after_the_adjustment_in_single_precision() {
        let base = height_of(terrain_at(30.0, [0.0, 0.0, 5000.0], 20.0));
        let terrain = FixedWingTerrain {
            site_y: 0.3,
            cg: 0.3,
            ..terrain_at(30.0, [0.0, 0.0, 5000.0], 20.0)
        };
        let mut expected = base;
        expected += 0.3;
        expected += 0.3;
        assert_eq!(height_of(terrain).to_bits(), expected.to_bits());
        // the addends move the height whatever the branch: a -1 minimum keeps the guide height and then adds them
        let kept = FixedWingTerrain {
            site_y: 2.0,
            cg: 3.0,
            ..terrain_at(-1.0, [0.0, 10.0, 5000.0], 20.0)
        };
        assert_eq!(height_of(kept), 15.0);
    }

    #[test]
    fn the_host_ignores_the_height_the_campaign_supplied_and_keeps_its_horizontal_waypoint() {
        let movers = movers(Kind::FixedWing, cruising());
        let terrain = Some(terrain_at(30.0, [0.0, 0.0, 5000.0], 20.0));
        let low = shaped_waypoint(&movers, terrain, [3.0, -999.0, 5000.0]).unwrap();
        let high = shaped_waypoint(&movers, terrain, [3.0, 99_999.0, 5000.0]).unwrap();
        assert_eq!(low, high);
        assert_eq!((low[0], low[2]), (3.0, 5000.0));
    }

    #[test]
    fn a_step_without_terrain_keeps_the_campaigns_waypoint_untouched() {
        let movers = movers(Kind::FixedWing, cruising());
        assert_eq!(shaped_waypoint(&movers, None, [1.0, 2.0, 3.0]).unwrap(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn a_terrain_sampled_somewhere_other_than_the_hosts_look_ahead_is_refused() {
        let movers = movers(Kind::FixedWing, cruising());
        let elsewhere = FixedWingTerrain {
            sample_point: [0.0, 111.0],
            ..terrain_at(30.0, [0.0, 0.0, 5000.0], 20.0)
        };
        let refused = shaped_waypoint(&movers, Some(elsewhere), [0.0, 0.0, 5000.0]).unwrap_err();
        assert!(refused.contains("look-ahead point"), "{refused}");
    }

    #[test]
    fn a_non_finite_terrain_input_is_refused() {
        let movers = movers(Kind::FixedWing, cruising());
        for broken in [
            FixedWingTerrain {
                sample_elevation: f32::NAN,
                ..terrain_at(30.0, [0.0, 0.0, 5000.0], 20.0)
            },
            FixedWingTerrain {
                min_height: f32::INFINITY,
                ..terrain_at(30.0, [0.0, 0.0, 5000.0], 20.0)
            },
            FixedWingTerrain {
                cg: f32::NAN,
                ..terrain_at(30.0, [0.0, 0.0, 5000.0], 20.0)
            },
            terrain_at(30.0, [f32::NAN, 0.0, 5000.0], 20.0),
        ] {
            let refused = shaped_waypoint(&movers, Some(broken), [0.0, 0.0, 5000.0]).unwrap_err();
            assert!(refused.contains("non-finite"), "{refused}");
        }
    }

    #[test]
    fn a_refused_terrain_changes_nothing_in_the_dispatched_step() {
        let mut movers = movers(Kind::FixedWing, cruising());
        let mut step = fixed_wing_move(FixedWingPhase::Flight);
        step.terrain = Some(FixedWingTerrain {
            sample_point: [1.0, 1.0],
            ..terrain_at(30.0, [0.0, 0.0, 5000.0], 20.0)
        });
        assert!(movers.step("v", cruising(), Request::FixedWingMove(step)).is_err());
        // the step that follows runs from the unchanged record
        let clean = fixed_wing_move(FixedWingPhase::Flight);
        assert!(movers.step("v", cruising(), Request::FixedWingMove(clean)).is_ok());
    }

    #[test]
    fn the_shaped_height_not_the_campaigns_reaches_the_flight() {
        let flight = |waypoint_y: f32, terrain: Option<FixedWingTerrain>| {
            let mut movers = movers(Kind::FixedWing, cruising());
            let mut step = fixed_wing_move(FixedWingPhase::Flight);
            step.waypoint = [0.0, waypoint_y, 5000.0];
            step.terrain = terrain;
            movers.step("v", cruising(), Request::FixedWingMove(step)).unwrap()
        };
        let terrain = Some(terrain_at(30.0, [0.0, 0.0, 5000.0], 20.0));
        let (low, high) = (flight(0.0, terrain), flight(9000.0, terrain));
        assert_eq!(
            (low.pose.position, low.pose.attitude, low.motion_vector),
            (high.pose.position, high.pose.attitude, high.motion_vector)
        );
        // without terrain the campaign's own height is what the flight steers to
        let (plain_low, plain_high) = (flight(0.0, None), flight(9000.0, None));
        assert_ne!(plain_low.motion_vector, plain_high.motion_vector);
    }

    #[test]
    fn a_flight_step_with_unresolvable_navigation_is_refused_before_any_change() {
        let with = |change: &dyn Fn(&mut FixedWingNavigation)| {
            let mut navigation = pursuing(&["b"]);
            change(&mut navigation);
            flight_with(navigation)
        };
        let mut no_navigation = fixed_wing_move(FixedWingPhase::Flight);
        no_navigation.navigation = None;
        let mut nan_waypoint = fixed_wing_move(FixedWingPhase::Flight);
        nan_waypoint.waypoint[1] = f32::NAN;
        nan_waypoint.navigation = Some(pursuing(&["b"]));
        let refused = [
            ("no navigation", Request::FixedWingMove(no_navigation), "carries no navigation"),
            ("no task leader", with(&|n| n.leader = None), "names no task leader"),
            ("itself as leader", with(&|n| n.leader = Some("v".into())), "names itself"),
            ("unknown leader", with(&|n| n.leader = Some("ghost".into())), "ghost is not a host id"),
            ("retired leader", with(&|n| n.leader = Some("r".into())), "r is retired"),
            ("helicopter leader", with(&|n| n.leader = Some("h".into())), "not a fixed-wing aircraft"),
            ("itself as member", with(&|n| n.members = vec![member("b"), member("v")]), "names itself"),
            ("retired member", with(&|n| n.members = vec![member("r")]), "r is retired"),
            ("helicopter member", with(&|n| n.members = vec![member("h")]), "not a fixed-wing aircraft"),
            (
                "the first unresolvable member is the one named",
                with(&|n| n.members = vec![member("b"), member("ghost1"), member("ghost2")]),
                "ghost1 is not a host id",
            ),
            (
                "non-finite member radius",
                with(&|n| n.members[0].avoidance_radius = f32::INFINITY),
                "non-finite avoidance radius",
            ),
            ("non-finite turn allowance", with(&|n| n.max_vertical_turn_allowance = f32::NAN), "non-finite"),
            ("non-finite avoidance radius", with(&|n| n.avoidance_radius = f32::NAN), "non-finite"),
            ("non-finite waypoint", Request::FixedWingMove(nan_waypoint), "non-finite"),
        ];
        let mut movers = quiet_fleet();
        for (name, request, expected) in refused {
            let error = movers.step("v", cruising(), request).unwrap_err();
            assert!(error.contains(expected), "{name}: {error}");
        }
        // none of it reached the record: the step the refusals shared a record with is the step a fresh host takes
        let valid = || flight_with(pursuing(&["b"]));
        let after = movers.step("v", cruising(), valid()).unwrap();
        let fresh = quiet_fleet().step("v", cruising(), valid()).unwrap();
        assert_eq!(after.pose, fresh.pose);
        assert_eq!(after.motion_vector, fresh.motion_vector);
    }

    #[test]
    fn the_members_of_a_group_leader_and_a_task_leaders_missing_leader_are_not_read() {
        // a group leader makes no collision test; a task leader follows its waypoint, so neither names what it never reads
        let group_leader = FixedWingNavigation {
            group_leader: true,
            members: vec![member("ghost")],
            ..pursuing(&[])
        };
        quiet_fleet().step("v", cruising(), flight_with(group_leader)).unwrap();
        let task_leader = FixedWingNavigation {
            task_leader: true,
            leader: None,
            ..pursuing(&["b"])
        };
        quiet_fleet().step("v", cruising(), flight_with(task_leader)).unwrap();
    }

    #[test]
    fn other_aircraft_steer_a_flight_step_from_the_host_records_not_from_the_campaign() {
        let fly = |movers: &mut Movers, navigation: FixedWingNavigation| movers.step("v", cruising(), flight_with(navigation)).unwrap();
        // the task leader's retained motion is what the pursuit reads: the campaign sends no pose or motion for it
        let towards = fly(&mut fleet([0.0, 0.0, 100.0], pose(0.0, 100_000.0), [0.0; 3]), pursuing(&[]));
        let across = fly(&mut fleet([100.0, 0.0, 0.0], pose(0.0, 100_000.0), [0.0; 3]), pursuing(&[]));
        assert_ne!(towards.motion_vector, across.motion_vector, "the leader's motion did not steer");
        // a group member in the way, from the host's record of it, bends the same request; far away it does not
        let clear = fly(&mut quiet_fleet(), pursuing(&["b"]));
        let head_on = fly(&mut fleet([0.0, 0.0, 100.0], pose(10.0, 120.0), [0.0, 0.0, -100.0]), pursuing(&["b"]));
        assert_ne!(clear.motion_vector, head_on.motion_vector, "the blocking member did not steer");
        // the same blocker is ignored when the aircraft leads its group, and when it is not named
        let leading = FixedWingNavigation {
            group_leader: true,
            ..pursuing(&["b"])
        };
        let blocked = || fleet([0.0, 0.0, 100.0], pose(10.0, 120.0), [0.0, 0.0, -100.0]);
        assert_eq!(fly(&mut blocked(), leading).motion_vector, clear.motion_vector);
        assert_eq!(fly(&mut blocked(), pursuing(&[])).motion_vector, clear.motion_vector);
    }

    /// the subject flying +z at the cruise speed, with its waypoint behind it
    fn facing_away(phase: FixedWingPhase) -> FixedWingMove {
        let mut step = fixed_wing_move(phase);
        step.waypoint = [0.0, 0.0, -5000.0];
        step.elevation = 0.0;
        step
    }

    /// the next step from what the host answered, with the answered motion vector as the campaign mirror
    fn then(movers: &mut Movers, after: &Outcome, mut step: FixedWingMove) -> Outcome {
        step.motion_vector = after.motion_vector;
        movers.step("v", after.pose, Request::FixedWingMove(step)).unwrap()
    }

    fn with_leader_at(z: f32) -> Movers {
        let mut movers = movers(Kind::FixedWing, cruising());
        movers.bind("leader", Kind::FixedWing, pose(0.0, z), Some([0.0, 0.0, 100.0]), None).unwrap();
        movers
    }

    fn only(navigation: FixedWingNavigation, mut step: FixedWingMove) -> Request {
        step.navigation = Some(navigation);
        Request::FixedWingMove(step)
    }

    #[test]
    fn the_host_finds_an_aircraft_facing_away_from_its_waypoint_and_flies_the_turn_around() {
        for phase in [FixedWingPhase::Flight, FixedWingPhase::Land] {
            // behind: the retained multiplier 2.0 halves the waypoint speed in the absolute branch,
            // 100 + min(50 - 100, 30) * 0.5
            let mut away = movers(Kind::FixedWing, cruising());
            let turned = away.step("v", cruising(), Request::FixedWingMove(facing_away(phase))).unwrap();
            assert_eq!(turned.pose.velocity, 75.0, "{phase:?}");
            assert!(!turned.rotated, "the absolute flight never reports a rotation: {phase:?}");
            // ahead: the same aircraft and request but for the waypoint is not turned around
            let mut ahead = movers(Kind::FixedWing, cruising());
            let mut step = facing_away(phase);
            step.waypoint = [0.0, 0.0, 5000.0];
            let straight = ahead.step("v", cruising(), Request::FixedWingMove(step)).unwrap();
            assert_ne!(straight.pose.velocity, 75.0, "{phase:?}");
        }
    }

    #[test]
    fn a_task_leader_is_always_turned_around_but_a_follower_only_beyond_fifteen_hundred_metres() {
        let follower = || only(pursuing(&[]), facing_away(FixedWingPhase::Flight));
        let near = with_leader_at(1500.0).step("v", cruising(), follower()).unwrap();
        let far = with_leader_at(1500.5).step("v", cruising(), follower()).unwrap();
        assert_ne!(near.pose.velocity, 75.0, "at exactly 1500 m the original does not correct");
        assert_eq!(far.pose.velocity, 75.0, "beyond 1500 m the host corrects");
        let led = only(task_leader_navigation(), facing_away(FixedWingPhase::Flight));
        let leading = movers(Kind::FixedWing, cruising()).step("v", cruising(), led).unwrap();
        assert_eq!(leading.pose.velocity, 75.0, "a task leader corrects without any leader");
    }

    #[test]
    fn the_cannon_intent_holds_the_turn_for_a_quarter_second_at_the_waypoint_speed() {
        let mut movers = movers(Kind::FixedWing, cruising());
        let mut ahead = fixed_wing_move(FixedWingPhase::Flight);
        ahead.wp_vel = 60.0;
        ahead.elevation = 0.0;
        // aiming: multiplier 1.0 and a hold to 100.25; 100 + min(60 - 100, 30) * 0.5
        let mut aiming = ahead.clone();
        aiming.aiming_cannon = true;
        let first = movers.step("v", cruising(), Request::FixedWingMove(aiming)).unwrap();
        assert_eq!(first.pose.velocity, 80.0);
        // no longer aiming, inside the hold: still absolute with the retained 1.0, 80 + (60 - 80) * 0.5
        let mut held = ahead.clone();
        held.elapsed_time = 100.2;
        let second = then(&mut movers, &first, held);
        assert_eq!(second.pose.velocity, 70.0);
        // the hold lapsed: the flight branch takes over, and not as the absolute 70 + (60 - 70) * 0.5
        let mut lapsed = ahead.clone();
        lapsed.elapsed_time = 100.3;
        let third = then(&mut movers, &second, lapsed);
        assert_ne!(third.pose.velocity, 65.0);
    }

    #[test]
    fn the_facing_away_hold_and_multiplier_persist_across_steps() {
        let mut movers = movers(Kind::FixedWing, cruising());
        let first = movers
            .step("v", cruising(), Request::FixedWingMove(facing_away(FixedWingPhase::Flight)))
            .unwrap();
        assert_eq!(first.pose.velocity, 75.0);
        // now flying +z, the waypoint is ahead, but the hold runs to 103.0 and the multiplier is still 2.0
        let mut ahead = fixed_wing_move(FixedWingPhase::Flight);
        ahead.elevation = 0.0;
        ahead.elapsed_time = 102.9;
        let second = then(&mut movers, &first, ahead.clone());
        assert_eq!(second.pose.velocity, 62.5, "75 + (50 - 75) * 0.5");
        ahead.elapsed_time = 103.5;
        let third = then(&mut movers, &second, ahead);
        assert_ne!(third.pose.velocity, 56.25, "the hold lapsed");
    }

    #[test]
    fn a_landing_needs_its_leader_only_when_the_host_finds_it_facing_away() {
        let follower = |step: FixedWingMove| only(FixedWingNavigation { leader: None, ..pursuing(&[]) }, step);
        // ahead of its waypoint it never reads the leader
        movers(Kind::FixedWing, cruising())
            .step("v", cruising(), follower(fixed_wing_move(FixedWingPhase::Land)))
            .unwrap();
        // facing away it must name one; the refusal leaves the retained turn state untouched
        let mut movers = with_leader_at(3000.0);
        let error = movers.step("v", cruising(), follower(facing_away(FixedWingPhase::Land))).unwrap_err();
        assert!(error.contains("names no task leader"), "{error}");
        let valid = || only(pursuing(&[]), facing_away(FixedWingPhase::Land));
        let after = movers.step("v", cruising(), valid()).unwrap();
        let fresh = with_leader_at(3000.0).step("v", cruising(), valid()).unwrap();
        assert_eq!(after.pose, fresh.pose);
        assert_eq!(after.pose.velocity, 75.0);
        // an unresolvable leader is a contract error when it is read
        let ghost = FixedWingNavigation {
            leader: Some("ghost".into()),
            ..pursuing(&[])
        };
        let error = with_leader_at(3000.0)
            .step("v", cruising(), only(ghost, facing_away(FixedWingPhase::Land)))
            .unwrap_err();
        assert!(error.contains("ghost is not a host id"), "{error}");
    }

    /// a member the campaign's roster lacks, in the place the group list gives it
    fn with_unresolved_member(mut navigation: FixedWingNavigation, at: usize) -> FixedWingNavigation {
        navigation.members.insert(
            at,
            NavigationMember {
                host_id: None,
                avoidance_radius: 20.0,
            },
        );
        navigation
    }

    #[test]
    fn a_member_the_roster_lacks_is_refused_only_where_the_navigator_reads_it() {
        // read: a navigating follower avoids every member in order, so the step is refused whole and changes nothing
        let mut movers = quiet_fleet();
        let error = movers
            .step("v", cruising(), flight_with(with_unresolved_member(pursuing(&["b"]), 1)))
            .unwrap_err();
        assert!(error.contains("group member #1 names no host id"), "{error}");
        let after = movers.step("v", cruising(), flight_with(pursuing(&["b"]))).unwrap();
        let fresh = quiet_fleet().step("v", cruising(), flight_with(pursuing(&["b"]))).unwrap();
        assert_eq!(after.pose, fresh.pose);
        // not read: a group leader makes no collision test, and a landing makes none either
        let led = FixedWingNavigation {
            group_leader: true,
            ..with_unresolved_member(pursuing(&["b"]), 0)
        };
        let reference = FixedWingNavigation {
            group_leader: true,
            ..pursuing(&["b"])
        };
        let led = quiet_fleet().step("v", cruising(), flight_with(led)).unwrap();
        let reference = quiet_fleet().step("v", cruising(), flight_with(reference)).unwrap();
        assert_eq!(led.pose, reference.pose);
        let mut landing = fixed_wing_move(FixedWingPhase::Land);
        landing.navigation = Some(with_unresolved_member(pursuing(&["b"]), 0));
        quiet_fleet().step("v", cruising(), Request::FixedWingMove(landing)).unwrap();
        // not read: a held absolute flight bypasses the navigator
        let mut aiming = fixed_wing_move(FixedWingPhase::Flight);
        aiming.aiming_cannon = true;
        aiming.navigation = Some(with_unresolved_member(pursuing(&["b"]), 0));
        quiet_fleet().step("v", cruising(), Request::FixedWingMove(aiming)).unwrap();
    }

    /// a follower whose roster has neither its task leader nor (here) a resolvable member
    fn leaderless(members: &[&str]) -> FixedWingNavigation {
        FixedWingNavigation {
            leader: None,
            ..pursuing(members)
        }
    }

    #[test]
    fn a_held_absolute_flight_reads_neither_its_leader_nor_its_members() {
        // the cannon verdict holds the absolute branch, so the navigator is bypassed: nothing it would read is asked for,
        // even a member the host cannot resolve
        let mut aiming = fixed_wing_move(FixedWingPhase::Flight);
        aiming.aiming_cannon = true;
        aiming.wp_vel = 60.0;
        aiming.elevation = 0.0;
        let held = movers(Kind::FixedWing, cruising())
            .step("v", cruising(), only(leaderless(&["ghost"]), aiming.clone()))
            .unwrap();
        let reference = movers(Kind::FixedWing, cruising())
            .step("v", cruising(), only(task_leader_navigation(), aiming))
            .unwrap();
        assert_eq!(held.pose, reference.pose);
        assert_eq!(held.motion_vector, reference.motion_vector);
        assert_eq!(held.pose.velocity, 80.0, "the absolute branch ran: 100 + min(60 - 100, 30) * 0.5");
    }

    #[test]
    fn a_hold_left_by_a_turn_around_needs_no_leader_until_it_lapses() {
        let turned_around = || {
            let mut movers = movers(Kind::FixedWing, cruising());
            let first = movers
                .step("v", cruising(), only(task_leader_navigation(), facing_away(FixedWingPhase::Flight)))
                .unwrap();
            assert_eq!(first.pose.velocity, 75.0);
            (movers, first)
        };
        let ahead = |elapsed: f32, navigation: FixedWingNavigation| {
            let mut step = fixed_wing_move(FixedWingPhase::Flight);
            step.elevation = 0.0;
            step.elapsed_time = elapsed;
            step.navigation = Some(navigation);
            step
        };
        // forward-facing inside the 3 s hold: the absolute branch again, with no leader and an unresolvable member
        let (mut movers, first) = turned_around();
        let held = then(&mut movers, &first, ahead(102.9, leaderless(&["ghost"])));
        assert_eq!(held.pose.velocity, 62.5, "75 + (50 - 75) * 0.5");
        // the hold lapsed: the navigator runs, reads the leader, and the step is refused whole
        let (mut movers, first) = turned_around();
        let mut lapsed = ahead(103.5, leaderless(&[]));
        lapsed.motion_vector = first.motion_vector;
        let error = movers.step("v", first.pose, Request::FixedWingMove(lapsed)).unwrap_err();
        assert!(error.contains("names no task leader"), "{error}");
        // the refusal changed nothing: the retained hold and multiplier still serve the next, valid, step
        let (mut twin, twin_first) = turned_around();
        let after = then(&mut movers, &first, ahead(102.9, task_leader_navigation()));
        let expected = then(&mut twin, &twin_first, ahead(102.9, task_leader_navigation()));
        assert_eq!(after.pose, expected.pose);
        assert_eq!(after.pose.velocity, 62.5);
    }

    #[test]
    fn a_navigating_flight_still_needs_its_leader_and_refuses_before_any_change() {
        let mut movers = quiet_fleet();
        let error = movers.step("v", cruising(), flight_with(leaderless(&["b"]))).unwrap_err();
        assert!(error.contains("names no task leader"), "{error}");
        let after = movers.step("v", cruising(), flight_with(pursuing(&["b"]))).unwrap();
        let fresh = quiet_fleet().step("v", cruising(), flight_with(pursuing(&["b"]))).unwrap();
        assert_eq!(after.pose, fresh.pose);
        // facing away, the turn-around reads it too: refused whole, the retained turn state untouched
        let mut movers = with_leader_at(3000.0);
        let error = movers
            .step("v", cruising(), only(leaderless(&[]), facing_away(FixedWingPhase::Flight)))
            .unwrap_err();
        assert!(error.contains("names no task leader"), "{error}");
        let valid = || only(pursuing(&[]), facing_away(FixedWingPhase::Flight));
        let after = movers.step("v", cruising(), valid()).unwrap();
        let fresh = with_leader_at(3000.0).step("v", cruising(), valid()).unwrap();
        assert_eq!(after.pose, fresh.pose);
        assert_eq!(after.pose.velocity, 75.0);
    }

    #[test]
    fn a_group_larger_than_thirty_two_is_flown_in_order_and_never_refused_for_its_count() {
        for count in [40usize, 63] {
            let fleet = || {
                let mut movers = quiet_fleet();
                for index in 0..count {
                    // the last member is the blocker; the rest are far away
                    let (at, motion) = if index + 1 == count {
                        (pose(10.0, 120.0), [0.0, 0.0, -100.0])
                    } else {
                        (pose(0.0, 100_000.0 + index as f32), [0.0, 0.0, 100.0])
                    };
                    movers.bind(&format!("m{index}"), Kind::FixedWing, at, Some(motion), None).unwrap();
                }
                movers
            };
            let names: Vec<String> = (0..count).map(|index| format!("m{index}")).collect();
            let all: Vec<&str> = names.iter().map(String::as_str).collect();
            let with = fleet().step("v", cruising(), flight_with(pursuing(&all))).unwrap();
            let without = fleet().step("v", cruising(), flight_with(pursuing(&all[..count - 1]))).unwrap();
            assert_ne!(with.motion_vector, without.motion_vector, "{count}: the last member did not steer");
            // the first unresolvable member, past the old limit, is the one named
            let mut broken: Vec<&str> = all.clone();
            broken.insert(33, "ghost1");
            broken.push("ghost2");
            let error = fleet().step("v", cruising(), flight_with(pursuing(&broken))).unwrap_err();
            assert!(error.contains("ghost1 is not a host id"), "{count}: {error}");
        }
    }

    // ---- the hosted helicopter step: plan, guide, rotor, begin, steer ----

    fn hc_plan(members: Vec<HelicopterMember>) -> HelicopterPlan {
        HelicopterPlan {
            group_leader: false,
            task_leader: true,
            task_leader_is_self: true,
            position_hold: false,
            terrain_follow: false,
            terrain_follow_attack_altitude: false,
            waypoint_kind: HelicopterWaypointKind::Route,
            guide_radius: None,
            guide_altitude_valid: false,
            waypoint: [0.0, 0.0, 1000.0],
            waypoint_velocity: [0.0; 3],
            distance: 1000.0,
            terrain_elevation: 0.0,
            fuel_supply_level: 100.0,
            cruise_velocity: 50.0,
            power_output: 1.0,
            cruise_altitude: 50.0,
            attack_altitude: 30.0,
            leader_radar_altitude: 0.0,
            map_min: [-1.0e6; 3],
            map_max: [1.0e6; 3],
            delta_seconds: 0.5,
            members,
            motion_vector: SEED,
        }
    }

    fn hc_steer(motion_vector: [f32; 3]) -> HelicopterSteer {
        HelicopterSteer {
            delta_seconds: 0.5,
            max_turn_rate: 10.0,
            elevation: -100.0,
            cg_to_ground: 0.0,
            terrain_follow: false,
            map_min: [-1.0e6; 3],
            map_max: [1.0e6; 3],
            waypoint_kind: HelicopterWaypointKind::Route,
            task_leader_is_self: true,
            guide_radius: None,
            required_heading: None,
            leader: None,
            targeting_active: false,
            weapon_vector: [0.0; 3],
            weapon_to_intercept_vector: [0.0; 3],
            g_max: 2.0,
            main_rotor_shaft_angle: 0.0,
            motion_vector,
        }
    }

    /// "v" and a wingman "w" at `other`, both helicopters
    fn wing(other: [f32; 3]) -> Movers {
        let mut movers = flying(pose(0.0, 0.0), SEED);
        movers
            .bind("w", Kind::Helicopter, at(other[0], other[1], other[2]), Some(SEED), Some(ROTOR))
            .unwrap();
        movers
    }

    /// the pose the host holds for "v": the campaign's next position must be this one
    fn held(movers: &Movers) -> Pose {
        movers.mobiles.get("v").unwrap().pose
    }

    fn planned(movers: &mut Movers, plan: HelicopterPlan) -> Result<HelicopterQueryPlan, String> {
        movers.step("v", held(movers), Request::HelicopterPlan(Box::new(plan)))?;
        match movers.take_answer() {
            Some(HelicopterAnswer::Query(query)) => Ok(query),
            other => panic!("a plan answers the lookups it needs, got {other:?}"),
        }
    }

    /// the recorded getter returns of the original `terrain_elevation = max (terrain_elevation, get (...))` over points
    /// whose terrain is `heights` (far to near), the running value starting at 0.0 and the getter repeatable
    fn recorded(heights: &[f32]) -> Vec<HelicopterTerrainSample> {
        let mut running = 0.0f32;
        heights
            .iter()
            .map(|&height| {
                if running > height {
                    HelicopterTerrainSample { first: height, second: None }
                } else {
                    running = height;
                    HelicopterTerrainSample {
                        first: height,
                        second: Some(height),
                    }
                }
            })
            .collect()
    }

    fn guided(movers: &mut Movers, samples: Vec<HelicopterTerrainSample>) -> Result<HelicopterGuidance, String> {
        let guide = HelicopterGuide {
            structures: Vec::new(),
            terrain_samples: samples,
            motion_vector: SEED,
        };
        movers.step("v", held(movers), Request::HelicopterGuide(guide))?;
        match movers.take_answer() {
            Some(HelicopterAnswer::Guidance(guidance)) => Ok(guidance),
            other => panic!("a guide answers the guidance, got {other:?}"),
        }
    }

    fn rotor_step(movers: &mut Movers, lift: f32, power_output: f32, gate: bool) -> Result<Outcome, String> {
        let mut alive = life(LifecycleMode::RotorAlive, ROTOR, SEED);
        alive.lift = lift;
        alive.power_output = power_output;
        alive.flag = gate;
        run(movers, held(movers), alive)
    }

    /// a whole hosted step up to and including the rotor check
    fn ready(movers: &mut Movers, plan: HelicopterPlan) -> HelicopterGuidance {
        let query = planned(movers, plan).unwrap();
        let guidance = guided(movers, recorded(&vec![0.0; query.terrain_sample_count])).unwrap();
        assert!(!rotor_step(movers, guidance.lift, 1.0, false).unwrap().moved);
        guidance
    }

    fn guided_begin(movers: &mut Movers) -> Result<Outcome, String> {
        movers.step("v", held(movers), Request::HelicopterGuidedBegin { motion_vector: SEED })
    }

    #[test]
    fn a_hosted_step_runs_plan_guide_rotor_begin_and_steer_and_the_host_owns_the_demand() {
        let mut movers = wing([5000.0, 0.0, 5000.0]);
        ready(&mut movers, hc_plan(vec![Some("w".into())]));
        let begun = guided_begin(&mut movers).unwrap();
        assert!(begun.pending);
        let ended = movers
            .step("v", begun.pose, Request::HelicopterGuidedEnd(hc_steer(begun.motion_vector)))
            .unwrap();
        assert!(!ended.pending);
        assert!(ended.angles.iter().all(|angle| angle.is_finite()), "{:?}", ended.angles);
        // the step is spent: the next one starts from a plan, and the retained guidance is not reused
        let error = guided_begin(&mut movers).unwrap_err();
        assert!(error.contains("without host guidance"), "{error}");
    }

    #[test]
    fn the_other_aircrafts_host_positions_decide_the_avoidance_not_anything_the_campaign_sends() {
        // the same plan and the same member name; only the host's record of that member differs
        let mut clear = wing([5000.0, 0.0, 5000.0]);
        let mut crossing = wing([3.0, 0.0, 100.0]);
        planned(&mut clear, hc_plan(vec![Some("w".into())])).unwrap();
        planned(&mut crossing, hc_plan(vec![Some("w".into())])).unwrap();
        let free = guided(&mut clear, Vec::new()).unwrap();
        let steered = guided(&mut crossing, Vec::new()).unwrap();
        assert_eq!(free.waypoint[0], 0.0);
        assert_ne!(free.waypoint, steered.waypoint, "a wingman on the line moved the waypoint");
        // a group leader does not avoid its wingmen, whatever the host records
        let mut leader = hc_plan(vec![Some("w".into())]);
        leader.group_leader = true;
        let mut leading = wing([3.0, 0.0, 100.0]);
        planned(&mut leading, leader).unwrap();
        assert_eq!(guided(&mut leading, Vec::new()).unwrap().waypoint, free.waypoint);
    }

    #[test]
    fn members_are_resolved_from_host_records_only_where_the_guidance_reads_them() {
        let mut movers = wing([50.0, 0.0, 50.0]);
        // a group leader reads no member, so none needs an id
        let mut leader = hc_plan(vec![None, Some("nobody".into())]);
        leader.group_leader = true;
        planned(&mut movers, leader).unwrap();
        // a follower names each member by a host id
        let error = planned(&mut movers, hc_plan(vec![Some("w".into()), None])).unwrap_err();
        assert!(error.contains("group member #1 names no host id"), "{error}");
        for (names, wanted) in [
            (vec![Some("v".to_owned())], "names itself"),
            (vec![Some("nobody".to_owned())], "is not a host id"),
        ] {
            let error = planned(&mut movers, hc_plan(names)).unwrap_err();
            assert!(error.contains(wanted), "{error}");
        }
        movers.bind("g", Kind::Routed, at(0.0, 0.0, 0.0), None, None).unwrap();
        let error = planned(&mut movers, hc_plan(vec![Some("g".into())])).unwrap_err();
        assert!(error.contains("is not a helicopter"), "{error}");
        // no member count is refused: a group of 64 resolves
        let mut big = wing([50.0, 0.0, 50.0]);
        for index in 0..63 {
            big.bind(
                &format!("m{index}"),
                Kind::Helicopter,
                at(100.0 + index as f32, 0.0, 0.0),
                Some(SEED),
                Some(ROTOR),
            )
            .unwrap();
        }
        let mut names: Vec<HelicopterMember> = (0..63).map(|index| Some(format!("m{index}"))).collect();
        names.push(Some("w".into()));
        planned(&mut big, hc_plan(names)).unwrap();
    }

    #[test]
    fn the_host_plans_the_lookups_far_to_near_and_none_without_terrain_following() {
        let mut movers = wing([5000.0, 0.0, 5000.0]);
        let flat = planned(&mut movers, hc_plan(Vec::new())).unwrap();
        assert!(!flat.structure_scan);
        assert_eq!(flat.terrain_sample_count, 0);
        let mut follow = hc_plan(Vec::new());
        follow.terrain_follow = true;
        let query = planned(&mut movers, follow).unwrap();
        assert!(query.structure_scan);
        assert_eq!(query.terrain_sample_count, 2);
        // the aircraft faces +z: the far sample first, the near one at the aircraft
        assert!(query.terrain_sample[0][1] > query.terrain_sample[1][1], "{:?}", query.terrain_sample);
        assert_eq!(query.terrain_sample[1], [0.0, 0.0]);
    }

    #[test]
    fn terrain_samples_set_the_required_height_and_a_wrong_answer_leaves_the_plan_to_be_corrected() {
        let mut movers = wing([5000.0, 0.0, 5000.0]);
        let mut follow = hc_plan(Vec::new());
        follow.terrain_follow = true;
        planned(&mut movers, follow).unwrap();
        let error = guided(&mut movers, recorded(&[10.0])).unwrap_err();
        assert!(error.contains("1 terrain samples answered, 2 planned"), "{error}");
        let nan = vec![
            HelicopterTerrainSample {
                first: 10.0,
                second: Some(10.0),
            },
            HelicopterTerrainSample { first: f32::NAN, second: None },
        ];
        let error = guided(&mut movers, nan).unwrap_err();
        assert!(error.contains("non-finite"), "{error}");
        let low = guided(&mut movers, recorded(&[10.0, 5.0])).unwrap();
        // the plan was answered once: a second answer has no plan to answer
        assert!(guided(&mut movers, recorded(&[10.0, 5.0])).unwrap_err().contains("without a planned step"));
        let mut again = hc_plan(Vec::new());
        again.terrain_follow = true;
        planned(&mut movers, again).unwrap();
        let high = guided(&mut movers, recorded(&[400.0, 5.0])).unwrap();
        assert!(
            high.required_height > low.required_height + 300.0,
            "{} vs {}",
            high.required_height,
            low.required_height
        );
    }

    /// the required height of a terrain-following plan answered with `samples`
    fn required_height_for(samples: Vec<HelicopterTerrainSample>) -> Result<f32, String> {
        let mut movers = wing([5000.0, 0.0, 5000.0]);
        let mut follow = hc_plan(Vec::new());
        follow.terrain_follow = true;
        planned(&mut movers, follow).unwrap();
        guided(&mut movers, samples).map(|guidance| guidance.required_height)
    }

    #[test]
    fn the_host_folds_the_recorded_getter_returns_as_the_original_max_macro_selects_them() {
        let sample = |first: f32, second: Option<f32>| HelicopterTerrainSample { first, second };
        // running 0.0 does not exceed a first return of 10: the getter is called again and the SECOND return is selected;
        // then running (10) exceeds the next first return (5): one call, the running value is kept
        let base = required_height_for(vec![sample(10.0, Some(10.0)), sample(5.0, None)]).unwrap();
        // the same first returns, but a lower second return: the second is what is selected, not the first
        let lower_second = required_height_for(vec![sample(10.0, Some(3.0)), sample(1.0, None)]).unwrap();
        assert!((base - lower_second - 7.0).abs() < 1.0e-3, "{base} vs {lower_second}");
        // a later second return that is lower than the running value replaces it: max is a selection, not a maximum
        let replaced = required_height_for(vec![sample(10.0, Some(10.0)), sample(20.0, Some(2.0))]).unwrap();
        assert!((base - replaced - 8.0).abs() < 1.0e-3, "{base} vs {replaced}");
        // the recorded pattern must be the one the macro produces: a second return where none can be, none where one is owed
        let error = required_height_for(vec![sample(10.0, Some(10.0)), sample(5.0, Some(5.0))]).unwrap_err();
        assert!(error.contains("carries a second getter return"), "{error}");
        let error = required_height_for(vec![sample(10.0, None), sample(5.0, None)]).unwrap_err();
        assert!(error.contains("carries no second getter return"), "{error}");
        // a refused answer leaves the plan in place: the corrected answer follows
        let mut movers = wing([5000.0, 0.0, 5000.0]);
        let mut follow = hc_plan(Vec::new());
        follow.terrain_follow = true;
        planned(&mut movers, follow).unwrap();
        assert!(guided(&mut movers, vec![sample(10.0, None), sample(5.0, None)]).is_err());
        let corrected = guided(&mut movers, vec![sample(10.0, Some(10.0)), sample(5.0, None)]).unwrap();
        assert_eq!(corrected.required_height, base);
    }

    #[test]
    fn the_phases_are_ordered_and_a_stale_or_unchecked_guidance_cannot_begin() {
        let mut movers = wing([5000.0, 0.0, 5000.0]);
        assert!(guided_begin(&mut movers).unwrap_err().contains("without host guidance"));
        let query = planned(&mut movers, hc_plan(Vec::new())).unwrap();
        assert!(guided_begin(&mut movers).unwrap_err().contains("without host guidance"));
        let guidance = guided(&mut movers, recorded(&vec![0.0; query.terrain_sample_count])).unwrap();
        // the rotor step has not run
        assert!(guided_begin(&mut movers).unwrap_err().contains("before its rotor step"));
        // the rotor step must carry the lift and the power the host's step used
        let error = rotor_step(&mut movers, guidance.lift + 0.25, 1.0, false).unwrap_err();
        assert!(error.contains("differs from the host's guidance"), "{error}");
        let error = rotor_step(&mut movers, guidance.lift, 0.5, false).unwrap_err();
        assert!(error.contains("differs from the planned step's"), "{error}");
        assert!(!rotor_step(&mut movers, guidance.lift, 1.0, false).unwrap().moved);
        guided_begin(&mut movers).unwrap();
        // the steering belongs to a hosted begin only; the legacy end is not its pair, and the refusal is that one
        let error = movers.step("v", held(&movers), Request::HelicopterEnd(helicopter_end_request())).unwrap_err();
        assert!(error.contains("after a hosted begin"), "{error}");
    }

    #[test]
    fn a_step_ended_early_by_a_gate_leaves_no_pending_phase_and_no_guidance_to_reuse() {
        // the campaign's own gate (the human wait, LANDED): abandon
        let mut abandoned = wing([5000.0, 0.0, 5000.0]);
        ready(&mut abandoned, hc_plan(Vec::new()));
        abandoned.step("v", held(&abandoned), Request::HelicopterAbandon).unwrap();
        assert!(guided_begin(&mut abandoned).unwrap_err().contains("without host guidance"));
        // the next step plans afresh and runs
        ready(&mut abandoned, hc_plan(Vec::new()));
        guided_begin(&mut abandoned).unwrap();

        // the rotor's spin-up gate holds the step: the begin is refused, and so is the guidance on a later step
        let mut gated = wing([5000.0, 0.0, 5000.0]);
        let spun = held(&gated);
        run(&mut gated, spun, life(LifecycleMode::RotorSet, [0.0, 0.0, 2.0], SEED)).unwrap();
        let query = planned(&mut gated, hc_plan(Vec::new())).unwrap();
        let guidance = guided(&mut gated, recorded(&vec![0.0; query.terrain_sample_count])).unwrap();
        let mut alive = life(LifecycleMode::RotorAlive, [0.0, 0.0, 2.0], SEED);
        alive.lift = guidance.lift;
        alive.flag = true;
        let held_at = held(&gated);
        assert!(run(&mut gated, held_at, alive).unwrap().moved);
        assert!(guided_begin(&mut gated).unwrap_err().contains("without host guidance"));

        // any other request between the phases drops the guidance
        let mut interrupted = wing([5000.0, 0.0, 5000.0]);
        ready(&mut interrupted, hc_plan(Vec::new()));
        let rotor = life(LifecycleMode::RotorSet, ROTOR, SEED);
        let held_at = held(&interrupted);
        run(&mut interrupted, held_at, rotor).unwrap();
        assert!(guided_begin(&mut interrupted).unwrap_err().contains("without host guidance"));

        // once begun, the step cannot be abandoned: the end is owed
        let mut owed = wing([5000.0, 0.0, 5000.0]);
        ready(&mut owed, hc_plan(Vec::new()));
        guided_begin(&mut owed).unwrap();
        assert!(owed.step("v", held(&owed), Request::HelicopterAbandon).is_err());
    }

    #[test]
    fn the_hosted_requests_belong_to_helicopters_and_the_answer_is_cleared_each_step() {
        let mut routed = movers(Kind::Routed, pose(0.0, 0.0));
        let plan = Request::HelicopterPlan(Box::new(hc_plan(Vec::new())));
        let error = routed.step("v", pose(0.0, 0.0), plan).unwrap_err();
        assert!(error.contains("is not a request for Routed"), "{error}");
        let mut movers = wing([5000.0, 0.0, 5000.0]);
        planned(&mut movers, hc_plan(Vec::new())).unwrap();
        // an unread answer is dropped by the next step rather than served later
        let rotor = life(LifecycleMode::RotorSet, ROTOR, SEED);
        let held_at = held(&movers);
        run(&mut movers, held_at, rotor).unwrap();
        assert!(movers.take_answer().is_none());
    }

    // ---- the flow is validated before it is changed ----

    #[test]
    fn a_legacy_end_after_a_hosted_begin_is_refused_by_the_pairing_guard_and_the_hosted_end_still_follows() {
        let mut movers = wing([5000.0, 0.0, 5000.0]);
        ready(&mut movers, hc_plan(Vec::new()));
        let begun = guided_begin(&mut movers).unwrap();
        let error = movers.step("v", begun.pose, Request::HelicopterEnd(helicopter_end_request())).unwrap_err();
        assert!(error.contains("after a hosted begin"), "{error}");
        // nothing changed: the step is still pending and its hosted end is still owed
        assert!(movers.mobiles.get("v").unwrap().helicopter_pending);
        let ended = movers
            .step("v", begun.pose, Request::HelicopterGuidedEnd(hc_steer(begun.motion_vector)))
            .unwrap();
        assert!(!ended.pending);
    }

    #[test]
    fn a_refused_guide_changes_nothing_and_a_repeated_guide_does_not_discard_the_open_step() {
        let mut movers = wing([5000.0, 0.0, 5000.0]);
        let mut follow = hc_plan(Vec::new());
        follow.terrain_follow = true;
        planned(&mut movers, follow).unwrap();
        // a refused answer leaves the plan: the corrected retry is accepted
        assert!(guided(&mut movers, recorded(&[10.0])).is_err());
        let guidance = guided(&mut movers, recorded(&[10.0, 5.0])).unwrap();
        // the step is guided; a repeated guide is refused and must not take the guidance with it
        let error = guided(&mut movers, recorded(&[10.0, 5.0])).unwrap_err();
        assert!(error.contains("without a planned step"), "{error}");
        assert!(matches!(movers.mobiles.get("v").unwrap().hc_flow, Some(HelicopterFlow::Guided { .. })));
        assert!(!rotor_step(&mut movers, guidance.lift, 1.0, false).unwrap().moved);
        assert!(guided_begin(&mut movers).unwrap().pending);
    }

    #[test]
    fn a_refused_unrelated_request_keeps_the_open_step_while_a_valid_one_or_a_gate_ends_it() {
        let mut movers = wing([5000.0, 0.0, 5000.0]);
        ready(&mut movers, hc_plan(Vec::new()));
        // refused (no crash is open): the step is untouched and begins
        let held_at = held(&movers);
        let error = run(&mut movers, held_at, life(LifecycleMode::CrashEnd, ROTOR, SEED)).unwrap_err();
        assert!(error.contains("without a pending crash_begin"), "{error}");
        assert!(matches!(
            movers.mobiles.get("v").unwrap().hc_flow,
            Some(HelicopterFlow::Guided { rotor_checked: true, .. })
        ));
        assert!(guided_begin(&mut movers).unwrap().pending);

        // a valid unrelated request still ends the stale guidance
        let mut interrupted = wing([5000.0, 0.0, 5000.0]);
        ready(&mut interrupted, hc_plan(Vec::new()));
        let held_at = held(&interrupted);
        run(&mut interrupted, held_at, life(LifecycleMode::RotorSet, ROTOR, SEED)).unwrap();
        assert!(interrupted.mobiles.get("v").unwrap().hc_flow.is_none());
        assert!(guided_begin(&mut interrupted).unwrap_err().contains("without host guidance"));

        // a valid early gate ends it too
        let mut gated = wing([5000.0, 0.0, 5000.0]);
        ready(&mut gated, hc_plan(Vec::new()));
        gated.step("v", held(&gated), Request::HelicopterAbandon).unwrap();
        assert!(gated.mobiles.get("v").unwrap().hc_flow.is_none());
    }

    #[test]
    fn a_successful_placement_ends_the_open_step_and_a_refused_one_keeps_it() {
        // refused (a non-finite creation): the guidance and the rotor check survive and the step begins
        let mut movers = wing([5000.0, 0.0, 5000.0]);
        ready(&mut movers, hc_plan(Vec::new()));
        let held_at = held(&movers);
        let bad = creation([f32::NAN, 0.0, 0.0], 0.0, [0.0, 1.0, 0.0]);
        assert!(movers.step("v", held_at, Request::Create(bad)).unwrap_err().contains("non-finite"));
        assert!(matches!(
            movers.mobiles.get("v").unwrap().hc_flow,
            Some(HelicopterFlow::Guided { rotor_checked: true, .. })
        ));
        assert!(guided_begin(&mut movers).unwrap().pending);

        // a Place between the guide and the begin moves the aircraft: the guidance is stale and cannot begin
        let mut placed = wing([5000.0, 0.0, 5000.0]);
        ready(&mut placed, hc_plan(Vec::new()));
        let moved = at(100.0, 0.0, 100.0);
        placed.step("v", moved, Request::Place { motion_vector: SEED }).unwrap();
        assert!(placed.mobiles.get("v").unwrap().hc_flow.is_none());
        assert!(guided_begin(&mut placed).unwrap_err().contains("without host guidance"));

        // so does a creation
        let mut created = wing([5000.0, 0.0, 5000.0]);
        ready(&mut created, hc_plan(Vec::new()));
        let held_at = held(&created);
        created
            .step("v", held_at, Request::Create(creation([100.0, 0.0, 100.0], 0.0, [0.0, 1.0, 0.0])))
            .unwrap();
        assert!(created.mobiles.get("v").unwrap().hc_flow.is_none());
        assert!(guided_begin(&mut created).unwrap_err().contains("without host guidance"));
    }

    // ---- the leader's heading is the host's record, read only where the demand reads it ----

    const STILL: [f32; 3] = [0.0; 3];

    /// "v" at rest following "l" at `leader_attitude`, planned, guided, rotor-checked and begun; the next call is the end
    fn follower_begun(leader_attitude: [f32; 9]) -> (Movers, Outcome) {
        let mut movers = Movers::default();
        movers.bind("v", Kind::Helicopter, at(0.0, 0.0, 0.0), Some(STILL), Some(ROTOR)).unwrap();
        let mut lead = at(500.0, 0.0, 0.0);
        lead.attitude = leader_attitude;
        movers.bind("l", Kind::Helicopter, lead, Some(STILL), Some(ROTOR)).unwrap();
        let mut plan = hc_plan(Vec::new());
        plan.group_leader = true;
        plan.task_leader_is_self = false;
        plan.guide_radius = Some(2000.0);
        plan.cruise_velocity = 1.0;
        plan.motion_vector = STILL;
        movers.step("v", held(&movers), Request::HelicopterPlan(Box::new(plan))).unwrap();
        let guide = HelicopterGuide {
            structures: Vec::new(),
            terrain_samples: Vec::new(),
            motion_vector: STILL,
        };
        movers.step("v", held(&movers), Request::HelicopterGuide(guide)).unwrap();
        let Some(HelicopterAnswer::Guidance(guidance)) = movers.take_answer() else {
            panic!("a guide answers the guidance")
        };
        let mut alive = life(LifecycleMode::RotorAlive, ROTOR, STILL);
        alive.lift = guidance.lift;
        let held_at = held(&movers);
        assert!(!run(&mut movers, held_at, alive).unwrap().moved);
        let begun = movers
            .step("v", held(&movers), Request::HelicopterGuidedBegin { motion_vector: STILL })
            .unwrap();
        (movers, begun)
    }

    fn follower_steer(begun: &Outcome, leader: Option<&str>) -> HelicopterSteer {
        let mut steer = hc_steer(begun.motion_vector);
        steer.task_leader_is_self = false;
        steer.guide_radius = Some(2000.0);
        steer.leader = leader.map(str::to_owned);
        steer
    }

    fn leader_turned(angle: f32) -> [f32; 9] {
        let (sin, cos) = angle.sin_cos();
        [cos, 0.0, sin, 0.0, 1.0, 0.0, -sin, 0.0, cos]
    }

    #[test]
    fn a_follower_close_to_its_leader_takes_the_heading_of_the_hosts_record_of_that_leader() {
        let mut headings = Vec::new();
        for angle in [0.4f32, -1.1] {
            let (mut movers, begun) = follower_begun(LEVEL);
            // the leader turns after the begin: the end reads the record as it is then, not as it was when the step began
            movers.mobiles.get_mut("l").unwrap().pose.attitude = leader_turned(angle);
            let ended = movers
                .step("v", begun.pose, Request::HelicopterGuidedEnd(follower_steer(&begun, Some("l"))))
                .unwrap();
            let expected = heading_of(&leader_turned(angle));
            assert!((ended.angles[0] - expected).abs() < 1.0e-5, "{} vs {expected}", ended.angles[0]);
            headings.push(ended.angles[0]);
        }
        assert_ne!(headings[0], headings[1], "the retained leader attitude decided the heading");
    }

    #[test]
    fn the_leader_is_resolved_only_where_the_heading_follows_it_and_a_refusal_keeps_the_step_open() {
        // a required heading is campaign intent: the leader id is not needed, even one that does not exist
        let (mut movers, begun) = follower_begun(leader_turned(0.4));
        let mut steer = follower_steer(&begun, Some("nobody"));
        steer.required_heading = Some(0.7);
        let ended = movers.step("v", begun.pose, Request::HelicopterGuidedEnd(steer)).unwrap();
        assert!((ended.angles[0] - 0.7).abs() < 1.0e-6, "{}", ended.angles[0]);

        // the task leader itself needs no leader either
        let (mut movers, begun) = follower_begun(leader_turned(0.4));
        let mut steer = follower_steer(&begun, None);
        steer.task_leader_is_self = true;
        movers.step("v", begun.pose, Request::HelicopterGuidedEnd(steer)).unwrap();

        // where the heading does follow the leader, a missing or unusable id is refused with the step still open
        let (mut movers, begun) = follower_begun(leader_turned(0.4));
        for (leader, wanted) in [
            (None, "names no leader host id"),
            (Some("v"), "names itself"),
            (Some("nobody"), "is not a host id"),
        ] {
            let error = movers
                .step("v", begun.pose, Request::HelicopterGuidedEnd(follower_steer(&begun, leader)))
                .unwrap_err();
            assert!(error.contains(wanted), "{error}");
            assert!(movers.mobiles.get("v").unwrap().helicopter_pending, "{wanted}: the step is still owed");
        }
        movers.bind("g", Kind::Routed, at(0.0, 0.0, 0.0), None, None).unwrap();
        let error = movers
            .step("v", begun.pose, Request::HelicopterGuidedEnd(follower_steer(&begun, Some("g"))))
            .unwrap_err();
        assert!(error.contains("is not a helicopter"), "{error}");
        // the corrected end then runs on the retained step
        let ended = movers
            .step("v", begun.pose, Request::HelicopterGuidedEnd(follower_steer(&begun, Some("l"))))
            .unwrap();
        assert!((ended.angles[0] - heading_of(&leader_turned(0.4))).abs() < 1.0e-5, "{}", ended.angles[0]);
    }
}
