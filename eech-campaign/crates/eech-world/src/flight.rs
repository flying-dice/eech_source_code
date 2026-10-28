//! The host's physical weapon flight (EEDCS-45, D2): the projectile simulation that produces the unguided ballistic tables and the guided missiles' range runs.
//! Kept in C (`csrc/wn_flight.c`, EECH's own arithmetic expression for expression) and driven here. The campaign sends each weapon's physical figures once at boot;
//! the table that results is this host's own and stays here (see `aim.rs`). The model here is the reference one: a host with another model (a DCS host) simulates or
//! looks up its own weapons and needs neither this arithmetic nor these figures.

use std::ffi::{c_char, c_int, c_uint};

/// Letters of a weapon's name the figures carry.
pub const NAME_MAX: usize = 63;

/// One weapon's physical figures: `csrc/wn_flight.c` `wf_figures`, field for field.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Figures {
    pub name: [c_char; NAME_MAX + 1],
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
    pub ignore_gravity: c_int,
    pub flight_profile_or_self_destr: c_int,
    pub parachute_type: c_int,
    pub weapon_class: c_uint,
    pub cruise_time_max_error: f32,
    pub detonation_radius: f32,
    pub max_seeker_limit: f32,
    pub max_launch_angle_error: f32,
    pub survive_ground_impact: c_int,
    pub in_flight_collision_check: c_int,
    pub guidance_type: c_int,
    pub aiming_type: c_int,
    pub decoy_type: c_int,
    pub warhead_type: c_int,
    pub hellfire_ii: c_int,
    pub smoke_grenade: c_int,
    pub submunition_count: c_int,
    pub submunition_velocity_factor: c_int,
    pub muzzle_velocity_max_error: f32,
    pub circular_error_probable: f32,
    pub rate_of_fire: f32,
    pub inverse_rate_of_fire: f32,
    pub acquire_forward: c_int,
}

impl Figures {
    /// The sixteen float figures, in the order of the struct.
    #[must_use]
    pub fn floats(&self) -> [f32; 16] {
        [
            self.start_weight,
            self.final_weight,
            self.max_range,
            self.muzzle_velocity,
            self.cruise_velocity,
            self.g_max,
            self.boost_time,
            self.cruise_time,
            self.inhibit_time,
            self.drag_coefficient,
            self.flightpath_deviation,
            self.boost_power,
            self.sustain_power,
            self.sustain_time,
            self.diameter,
            self.max_altitude,
        ]
    }

    /// The four floats appended after `weapon_class`.
    #[must_use]
    pub fn extra_floats(&self) -> [f32; 4] {
        [
            self.cruise_time_max_error,
            self.detonation_radius,
            self.max_seeker_limit,
            self.max_launch_angle_error,
        ]
    }

    /// The four floats appended after the integers of the weapon step: the birth's and the release's.
    #[must_use]
    pub fn launch_floats(&self) -> [f32; 4] {
        [
            self.muzzle_velocity_max_error,
            self.circular_error_probable,
            self.rate_of_fire,
            self.inverse_rate_of_fire,
        ]
    }

    /// Fills the name (at most `NAME_MAX` bytes, no NUL, not empty).
    pub fn set_name(&mut self, name: &str) -> Result<(), String> {
        let bytes = name.as_bytes();
        if bytes.is_empty() || bytes.len() > NAME_MAX || bytes.contains(&0) {
            return Err(format!("a weapon name must be 1 to {NAME_MAX} bytes without a NUL, got {} bytes", bytes.len()));
        }
        self.name = [0; NAME_MAX + 1];
        for (slot, byte) in self.name.iter_mut().zip(bytes) {
            *slot = *byte as c_char;
        }
        Ok(())
    }
}

/// What one simulated flight left behind (`wf_flight_record`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FlightRecord {
    pub velocity_test: c_int,
    pub pitch_index: c_int,
    pub range_mark: c_int,
    pub range_values: c_int,
    pub steps: c_int,
    pub end_time: f32,
}

/// What one range run left behind (`wf_range_record`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RangeRecord {
    pub ran: c_int,
    pub zero_pitch: c_int,
    pub end_z: f32,
    pub end_y: f32,
    pub end_velocity: f32,
    pub end_lifetime: f32,
}

extern "C" {
    fn eech_wn_ballistics_open() -> c_int;
    fn eech_wn_ballistics_close();
    fn eech_wn_ballistics_weapon(
        figures: *const Figures,
        cells: *mut f32,
        cell_capacity: usize,
        records: *mut FlightRecord,
        record_capacity: c_int,
        flight_count: *mut c_int,
    ) -> c_int;
    fn eech_wn_range_open() -> c_int;
    fn eech_wn_range_close();
    fn eech_wn_range_weapon(figures: *const Figures, results: *mut RangeRecord) -> c_int;
}

/// The most flights one table can hold: five velocity tests of at most 85 pitch indices.
pub const FLIGHT_CAPACITY: usize = 5 * 85;

/// Velocity tests and pitch indices of every table, and the most range a table may span (the largest weapon range there is is 80 km).
const VELOCITY_TESTS: usize = 5;
const PITCH_INDICES: usize = 85;
pub const MAX_TABLE_RANGE: f32 = 200_000.0;

/// The range marks of a table for a weapon's range limit: `max_range / 20 + 2`. An error for a limit that is not a finite number in (0, `MAX_TABLE_RANGE`].
pub fn table_range_values(max_range: f32) -> Result<usize, String> {
    if !max_range.is_finite() || max_range <= 0.0 || max_range > MAX_TABLE_RANGE {
        return Err(format!(
            "a range limit of {max_range} m cannot give a table (a finite range in (0, {MAX_TABLE_RANGE}] is needed)"
        ));
    }
    Ok((f64::from(max_range) / 20.0) as usize + 2)
}

/// The floats of the table for a weapon's range limit: velocity tests by pitch indices by range marks, two floats a cell.
pub fn table_cell_count(max_range: f32) -> Result<usize, String> {
    VELOCITY_TESTS
        .checked_mul(PITCH_INDICES)
        .and_then(|cells| cells.checked_mul(table_range_values(max_range).ok()?))
        .and_then(|cells| cells.checked_mul(2))
        .ok_or_else(|| format!("the table for a range limit of {max_range} m is too large to size"))
}

/// Opens the ballistics diagnostic file in the working directory, as the original did before its first weapon. False when it cannot be opened (the tables do not depend on it).
pub fn ballistics_open() -> bool {
    // SAFETY: no pointers cross.
    unsafe { eech_wn_ballistics_open() != 0 }
}

pub fn ballistics_close() {
    // SAFETY: no pointers cross.
    unsafe { eech_wn_ballistics_close() }
}

/// Simulates one unguided weapon at every pitch and velocity test into `cells` (the table: `5 * 85 * range_values * 2` floats) and returns the flights it simulated.
pub fn ballistics_weapon(figures: &Figures, cells: &mut [f32]) -> Result<Vec<FlightRecord>, String> {
    let needed = table_cell_count(figures.max_range)?;
    if cells.len() != needed {
        return Err(format!(
            "the table for a range limit of {} m needs {needed} cells, not {}",
            figures.max_range,
            cells.len()
        ));
    }
    let mut records = vec![FlightRecord::default(); FLIGHT_CAPACITY];
    let mut count: c_int = 0;
    // SAFETY: the figures, the cells and the records are valid for the call; the cells hold exactly what the figures need (checked above, and again by the C function, which
    // writes nothing otherwise) and the record capacity passed is the records' length.
    let status = unsafe {
        eech_wn_ballistics_weapon(
            figures,
            cells.as_mut_ptr(),
            cells.len(),
            records.as_mut_ptr(),
            records.len() as c_int,
            &mut count,
        )
    };
    if status != 0 || count < 0 || count as usize > records.len() {
        return Err("the ballistic simulation did not complete (more flights than a table holds, or it asked for a draw or a wind sample)".into());
    }
    records.truncate(count as usize);
    Ok(records)
}

pub fn range_open() -> bool {
    // SAFETY: no pointers cross.
    unsafe { eech_wn_range_open() != 0 }
}

pub fn range_close() {
    // SAFETY: no pointers cross.
    unsafe { eech_wn_range_close() }
}

/// Runs one guided weapon's two range simulations (level, then aimed).
pub fn range_weapon(figures: &Figures) -> Result<[RangeRecord; 2], String> {
    let mut results = [RangeRecord::default(); 2];
    // SAFETY: the figures and the two result slots are valid for the call.
    let status = unsafe { eech_wn_range_weapon(figures, results.as_mut_ptr()) };
    if status != 0 {
        return Err("a range simulation asked for a draw or a wind sample it was not given".into());
    }
    Ok(results)
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------
// The weapon step (EEDCS-45): one weapon's movement for one frame, a pure question of the weapon's state and the answers to the lookups it asked for (`csrc/wn_flight.c`).

/// The most actions, lookups and echo events one step can produce, and the most draws it can be given (the C step's own limits).
pub const STEP_ACTIONS_MAX: usize = 32;
pub const STEP_LOOKUPS_MAX: usize = 16;
pub const STEP_ECHO_MAX: usize = 64;
pub const STEP_DRAWS_MAX: usize = 8;
/// The most lookup answers a request may carry (a step asks for a few dozen at most).
pub const STEP_ANSWERS_MAX: usize = 128;

/// A lookup, an answer to one, an action or an echo event: `wf_event`. A lookup is its kind, the integers a to d and the floats f[0..6]; its answer repeats them and fills
/// f[6..12].
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StepEvent {
    pub kind: c_int,
    pub a: c_int,
    pub b: c_int,
    pub c: c_int,
    pub d: c_int,
    pub f: [f32; 12],
}

/// `wf_step_in`.
#[repr(C)]
struct RawStepIn {
    figures: *const Figures,
    delta_time: f32,
    system_time: c_uint,
    position: [f32; 3],
    attitude: [f32; 9],
    motion_vector: [f32; 3],
    velocity: f32,
    weapon_lifetime: f32,
    decoy_timer: f32,
    missile_phase: c_int,
    loal_mode: c_int,
    weapon_on_ground: c_int,
    parachute_status: c_int,
    kill_code: c_int,
    entity_samples: [c_int; 4],
    has_launcher: c_int,
    has_target: c_int,
    server: c_int,
    submunition: c_int,
    map_min: [f32; 3],
    map_max: [f32; 3],
    echo: c_int,
    answers: *const StepEvent,
    answer_count: c_int,
    draws: *const c_int,
    draw_count: c_int,
    other: *const Figures,
    coefficient: f32,
    effectiveness: [f32; 3],
}

/// `wf_step_out`.
#[repr(C)]
struct RawStepOut {
    actions: [StepEvent; STEP_ACTIONS_MAX],
    action_count: c_int,
    lookups: [StepEvent; STEP_LOOKUPS_MAX],
    lookup_count: c_int,
    echo: [StepEvent; STEP_ECHO_MAX],
    echo_count: c_int,
    need_lookup: c_int,
    need_draw: c_int,
    failed: c_int,
    reason: [c_char; 96],
}

extern "C" {
    fn eech_wn_weapon_step(input: *const RawStepIn, output: *mut RawStepOut) -> c_int;
    fn eech_wn_weapon_birth(input: *const RawStepIn, output: *mut RawStepOut) -> c_int;
    fn eech_wn_weapon_decoy(input: *const RawStepIn, output: *mut RawStepOut) -> c_int;
    fn eech_wn_weapon_seduce(input: *const RawStepIn, output: *mut RawStepOut) -> c_int;
    fn eech_wn_weapon_reacquire(input: *const RawStepIn, output: *mut RawStepOut) -> c_int;
    fn eech_wn_weapon_animate(input: *const RawAnimateIn, output: *mut RawAnimateOut);
    fn eech_wn_release(input: *const RawReleaseIn, output: *mut RawReleaseOut);
}

/// `wf_animate_in`.
#[repr(C)]
struct RawAnimateIn {
    figures: *const Figures,
    parachute_status: c_int,
    weapon_on_ground: c_int,
    weapon_lifetime: f32,
    motion_vector_y: f32,
}

/// `wf_animate_out`.
#[repr(C)]
#[derive(Default)]
struct RawAnimateOut {
    failed: c_int,
    parachute_status: c_int,
    write_lifetime: c_int,
    weapon_lifetime: f32,
}

/// What the parachute animation of a weapon needs: its parachute, whether it rests on the ground, its lifetime and how it is moving vertically.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimateInput {
    pub parachute_status: i32,
    pub weapon_on_ground: bool,
    pub weapon_lifetime: f32,
    pub motion_vector_y: f32,
}

/// What the animation left: the parachute's state, and the weapon's lifetime when the animation reset it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimateOutcome {
    pub parachute_status: i32,
    pub weapon_lifetime: Option<f32>,
}

/// The number of states a parachute has (closed, three phases of opening, collapsed).
const PARACHUTE_STATES: i32 = 5;

/// One frame of a falling crate's or ejector seat's parachute (`update_weapon_ejector_seat_animation`, `update_weapon_crate_animation`), by the host's figures for it. A
/// weapon with no parachute is left as it is. An error for a state that is not a parachute's or a number that is not finite.
pub fn animate(figures: &Figures, input: &AnimateInput) -> Result<AnimateOutcome, String> {
    if !input.weapon_lifetime.is_finite() || !input.motion_vector_y.is_finite() {
        return Err("the animation's lifetime and motion must be finite".into());
    }
    if !(0..PARACHUTE_STATES).contains(&input.parachute_status) {
        return Err(format!("{} is not a parachute state", input.parachute_status));
    }
    let raw = RawAnimateIn {
        figures,
        parachute_status: input.parachute_status,
        weapon_on_ground: c_int::from(input.weapon_on_ground),
        weapon_lifetime: input.weapon_lifetime,
        motion_vector_y: input.motion_vector_y,
    };
    let mut out = RawAnimateOut::default();
    // SAFETY: the figures and both structures are valid for the call.
    unsafe { eech_wn_weapon_animate(&raw, &mut out) };
    if out.failed != 0 || !(0..PARACHUTE_STATES).contains(&out.parachute_status) {
        return Err("the parachute animation reached a state it cannot continue from".into());
    }
    Ok(AnimateOutcome {
        parachute_status: out.parachute_status,
        weapon_lifetime: (out.write_lifetime != 0).then_some(out.weapon_lifetime),
    })
}

/// `wf_release_in`.
#[repr(C)]
struct RawReleaseIn {
    figures: *const Figures,
    delta_time: f32,
    current_time: c_uint,
    has_timer: c_int,
    last_shot: c_uint,
    not_ai: c_int,
}

/// `wf_release_out`.
#[repr(C)]
#[derive(Default)]
struct RawReleaseOut {
    refuse: c_int,
    burst_size: c_int,
    write_last_shot: c_int,
    last_shot: c_uint,
}

/// What the release of a weapon needs: the frame, the clock, the launcher's last-shot clock (when it has one) and whether the launcher is not an AI.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReleaseInput {
    pub delta_time: f32,
    pub current_time: u32,
    pub last_shot: Option<u32>,
    pub not_ai: bool,
}

/// How the release came out: whether the launcher lets nothing go, the rounds it lets go, and the last-shot clock the gate wrote (even when it then refused).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReleaseOutcome {
    pub refuse: bool,
    pub burst_size: i32,
    pub last_shot: Option<u32>,
}

/// The rate-of-fire gate of a launcher releasing a weapon, by the host's figures for it. An error for figures or a frame that are not finite.
pub fn release(figures: &Figures, input: &ReleaseInput) -> Result<ReleaseOutcome, String> {
    if !input.delta_time.is_finite() {
        return Err("the release's frame time must be finite".into());
    }
    let raw = RawReleaseIn {
        figures,
        delta_time: input.delta_time,
        current_time: input.current_time,
        has_timer: c_int::from(input.last_shot.is_some()),
        last_shot: input.last_shot.unwrap_or(0),
        not_ai: c_int::from(input.not_ai),
    };
    let mut out = RawReleaseOut::default();
    // SAFETY: the figures and both structures are valid for the call.
    unsafe { eech_wn_release(&raw, &mut out) };
    Ok(ReleaseOutcome {
        refuse: out.refuse != 0,
        burst_size: out.burst_size,
        last_shot: (out.write_last_shot != 0).then_some(out.last_shot),
    })
}

/// The weapon as the campaign holds it, and the world constants the step needs.
#[derive(Clone, Debug, PartialEq)]
pub struct StepInput {
    pub delta_time: f32,
    pub system_time: u32,
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
    pub submunition: bool,
    /// A seduction only: the difficulty coefficient and the chaff, flare and smoke effectiveness settings the campaign gives.
    pub coefficient: f32,
    pub effectiveness: [f32; 3],
    pub map_min: [f32; 3],
    pub map_max: [f32; 3],
    pub echo: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepState {
    Done,
    NeedLookup,
    NeedDraw,
}

/// What one pass of the step produced: its actions when it is done, the lookups it still needs, or a draw.
#[derive(Clone, Debug, PartialEq)]
pub struct StepOutcome {
    pub state: StepState,
    pub actions: Vec<StepEvent>,
    pub lookups: Vec<StepEvent>,
    pub echo: Vec<StepEvent>,
}

/// Runs one pass of a weapon's step. The figures are the host's own for the weapon; `answers` are the lookups answered so far and `draws` the samples of the campaign's random
/// stream given so far. An error for a state the step cannot run on or a failure of the step itself (the weapon asks for what it does not have).
pub fn weapon_step(figures: &Figures, input: &StepInput, answers: &[StepEvent], draws: &[i32]) -> Result<StepOutcome, String> {
    run_step(figures, input, answers, draws, StepKind::Step, None)
}

/// What a request of the step protocol asks the host for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepKind {
    /// A frame of a weapon's flight.
    Step,
    /// A weapon's birth.
    Birth,
    /// A decoy's update: its timer.
    Decoy,
    /// A seeker's answer to a decoy becoming active.
    Seduce,
    /// A seeker's answer to the decoy it was tracking expiring.
    Reacquire,
}

/// Runs one pass of a seeker's answer to the decoy `decoy` it was tracking expiring: whether it goes back to the decoy's launcher.
pub fn weapon_reacquire(figures: &Figures, decoy: &Figures, input: &StepInput, answers: &[StepEvent], draws: &[i32]) -> Result<StepOutcome, String> {
    run_step(figures, input, answers, draws, StepKind::Reacquire, Some(decoy))
}

/// Runs one pass of a decoy's update: the timer runs down and, when it runs out, restarts at a time drawn from the campaign's stream.
pub fn weapon_decoy(figures: &Figures, input: &StepInput, answers: &[StepEvent], draws: &[i32]) -> Result<StepOutcome, String> {
    run_step(figures, input, answers, draws, StepKind::Decoy, None)
}

/// Runs one pass of a seeker's answer to a decoy becoming active: `figures` are the seeker's, `decoy` the decoy's.
pub fn weapon_seduce(figures: &Figures, decoy: &Figures, input: &StepInput, answers: &[StepEvent], draws: &[i32]) -> Result<StepOutcome, String> {
    run_step(figures, input, answers, draws, StepKind::Seduce, Some(decoy))
}

/// Runs one pass of a weapon's birth: the launch kinematics of a weapon the launcher has just let go (`input.position` and `input.attitude` are the pose it left the
/// launcher in; `input.velocity` is the parent weapon's velocity for a submunition).
pub fn weapon_birth(figures: &Figures, input: &StepInput, answers: &[StepEvent], draws: &[i32]) -> Result<StepOutcome, String> {
    run_step(figures, input, answers, draws, StepKind::Birth, None)
}

fn run_step(
    figures: &Figures,
    input: &StepInput,
    answers: &[StepEvent],
    draws: &[i32],
    kind: StepKind,
    other: Option<&Figures>,
) -> Result<StepOutcome, String> {
    let finite = |what: &str, values: &[f32]| -> Result<(), String> {
        if values.iter().all(|value| value.is_finite()) {
            Ok(())
        } else {
            Err(format!("the weapon step's {what} must be finite"))
        }
    };
    finite("delta time", &[input.delta_time])?;
    finite("position", &input.position)?;
    finite("attitude", &input.attitude)?;
    finite("motion vector", &input.motion_vector)?;
    finite("state", &[input.velocity, input.weapon_lifetime, input.decoy_timer])?;
    finite("map extent", &input.map_min)?;
    finite("map extent", &input.map_max)?;
    finite("seduction setting", &[input.coefficient])?;
    finite("seduction setting", &input.effectiveness)?;
    if answers.len() > STEP_ANSWERS_MAX {
        return Err(format!(
            "the weapon step was given {} lookup answers, more than the {STEP_ANSWERS_MAX} it takes",
            answers.len()
        ));
    }
    if draws.len() > STEP_DRAWS_MAX {
        return Err(format!(
            "the weapon step was given {} draws, more than the {STEP_DRAWS_MAX} it takes",
            draws.len()
        ));
    }
    if draws.iter().any(|draw| !(0..=32767).contains(draw)) || input.entity_samples.iter().any(|draw| !(0..=32767).contains(draw)) {
        return Err("a random sample given to the weapon step is outside [0, 32767]".into());
    }
    let raw = RawStepIn {
        figures,
        delta_time: input.delta_time,
        system_time: input.system_time,
        position: input.position,
        attitude: input.attitude,
        motion_vector: input.motion_vector,
        velocity: input.velocity,
        weapon_lifetime: input.weapon_lifetime,
        decoy_timer: input.decoy_timer,
        missile_phase: input.missile_phase,
        loal_mode: c_int::from(input.loal_mode),
        weapon_on_ground: c_int::from(input.weapon_on_ground),
        parachute_status: input.parachute_status,
        kill_code: input.kill_code,
        entity_samples: input.entity_samples,
        has_launcher: c_int::from(input.has_launcher),
        has_target: c_int::from(input.has_target),
        server: c_int::from(input.server),
        submunition: c_int::from(input.submunition),
        map_min: input.map_min,
        map_max: input.map_max,
        echo: c_int::from(input.echo),
        answers: answers.as_ptr(),
        answer_count: answers.len() as c_int,
        draws: draws.as_ptr(),
        draw_count: draws.len() as c_int,
        other: other.map_or(std::ptr::null(), std::ptr::from_ref),
        coefficient: input.coefficient,
        effectiveness: input.effectiveness,
    };
    // SAFETY: plain data; all-zero is a valid value of every field. Boxed because the output is several kilobytes.
    let mut out: Box<RawStepOut> = Box::new(unsafe { std::mem::zeroed() });
    // SAFETY: the figures, the answers and the draws are valid for the call and the counts passed are their lengths; the output is a writable `RawStepOut`.
    let status = unsafe {
        match kind {
            StepKind::Step => eech_wn_weapon_step(&raw, &mut *out),
            StepKind::Birth => eech_wn_weapon_birth(&raw, &mut *out),
            StepKind::Decoy => eech_wn_weapon_decoy(&raw, &mut *out),
            StepKind::Seduce => eech_wn_weapon_seduce(&raw, &mut *out),
            StepKind::Reacquire => eech_wn_weapon_reacquire(&raw, &mut *out),
        }
    };
    if status != 0 || out.failed != 0 {
        let bytes = out.reason.iter().map(|byte| *byte as u8).take_while(|byte| *byte != 0).collect::<Vec<_>>();
        return Err(format!("the weapon step failed: {}", String::from_utf8_lossy(&bytes)));
    }
    let counted = |what: &str, count: c_int, max: usize| -> Result<usize, String> {
        usize::try_from(count)
            .ok()
            .filter(|count| *count <= max)
            .ok_or_else(|| format!("the weapon step reported {count} {what}"))
    };
    let state = if out.need_lookup != 0 {
        StepState::NeedLookup
    } else if out.need_draw != 0 {
        StepState::NeedDraw
    } else {
        StepState::Done
    };
    let actions = out.actions[..counted("actions", out.action_count, STEP_ACTIONS_MAX)?].to_vec();
    let lookups = out.lookups[..counted("lookups", out.lookup_count, STEP_LOOKUPS_MAX)?].to_vec();
    let echo = out.echo[..counted("echo events", out.echo_count, STEP_ECHO_MAX)?].to_vec();
    Ok(StepOutcome { state, actions, lookups, echo })
}
