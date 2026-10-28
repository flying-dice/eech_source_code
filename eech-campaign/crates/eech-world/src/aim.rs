//! The host's physical aiming and weapon-system-readiness kernel (EEDCS-44, D2): where a launcher's weapon-system devices point, whether they are
//! ready, and the intercept, lead and ballistic predictions they are aimed with. Kept in C (`csrc/aim_step.c`, EECH's own arithmetic expression for
//! expression) and driven here one question at a time. Every question is pure: this host keeps nothing between questions except the ballistic tables
//! it generates itself at boot (EEDCS-45: the campaign hands it each unguided weapon's physical figures once and this host simulates the weapon's flight,
//! `flight.rs`; no table cell crosses the boundary).
//!
//! Which weapon is selected, which target is aimed at, which package's devices are used, whether the weapon system should be made ready, and the fire
//! decision are not here. The device viewpoint, the target point, the positions, speeds and motion vectors and the weapon and package figures are
//! transitional mirrors of what a host that owns its world already knows, supplied to this reference host as inputs of its model; they do not make
//! the campaign the owner of a physical solver. The model here is the reference one: a host with another model (a DCS host) answers each question
//! from its own state and needs neither this arithmetic nor the tables.

use std::collections::HashMap;
use std::ffi::c_int;

use crate::flight;

/// The shape of a ballistic table the kernel indexes: five velocity tests, 85 pitch indices, `max_range / 20 + 2` ranges.
const VELOCITY_TESTS: i32 = 5;
const PITCH_INDICES: i32 = 85;
const ECHO_MAX: usize = 48;

#[repr(C)]
#[derive(Clone, Copy)]
struct RawEvent {
    kind: c_int,
    a: c_int,
    b: c_int,
    c: c_int,
    d: c_int,
    f: [f32; 12],
}

#[repr(C)]
struct RawEcho {
    count: c_int,
    overflow: c_int,
    events: [RawEvent; ECHO_MAX],
}

#[repr(C)]
struct RawTable {
    present: c_int,
    velocity_tests: c_int,
    pitch_indices: c_int,
    range_values: c_int,
    max_range: f32,
    cells: *const f32,
}

extern "C" {
    fn eech_aim_ready_update(state: f32, animation_rate: f32, delta_time: f32, state_out: *mut f32) -> c_int;
    fn eech_aim_ready_command(animation_state: f32, open: c_int) -> f32;
    #[allow(clippy::too_many_arguments)]
    fn eech_aim_required(
        attitude: *const f32,
        intercept_vector: *const f32,
        min_heading: f32,
        max_heading: f32,
        min_pitch: f32,
        max_pitch_limit: f32,
        climb: c_int,
        required_heading: *mut f32,
        required_pitch: *mut f32,
    );
    fn eech_aim_required_unbounded(attitude: *const f32, intercept_vector: *const f32, required_heading: *mut f32, required_pitch: *mut f32);
    fn eech_aim_required_tracked(
        attitude: *const f32,
        device_position: *const f32,
        tracking_point: *const f32,
        pitch: f32,
        required_heading: *mut f32,
        required_pitch: *mut f32,
    );
    #[allow(clippy::too_many_arguments)]
    fn eech_aim_step(
        current_heading: f32,
        current_pitch: f32,
        required_heading: f32,
        required_pitch: f32,
        min_heading: f32,
        max_heading: f32,
        heading_rate: f32,
        pitch_rate: f32,
        delta_time: f32,
        out: *mut f32,
    );
    #[allow(clippy::too_many_arguments)]
    fn eech_aim_ballistic(
        table: *const RawTable,
        range: f32,
        height_diff_or_pitch: f32,
        simplified: c_int,
        fixed_pitch: c_int,
        parent_velocity: f32,
        aiming_pitch: *mut f32,
        time_of_flight: *mut f32,
        fault: *mut c_int,
    ) -> c_int;
    #[allow(clippy::too_many_arguments)]
    fn eech_aim_intercept(
        device_position: *const f32,
        has_weapon: c_int,
        cruise_velocity: f32,
        acquire_forward: c_int,
        source_velocity: f32,
        target_position: *const f32,
        target_point: *const f32,
        target_motion_vector: *const f32,
        intercept_point: *mut f32,
        echo: *mut RawEcho,
    );
    #[allow(clippy::too_many_arguments)]
    fn eech_aim_lead(
        table: *const RawTable,
        weapon: c_int,
        device_position: *const f32,
        parent_velocity: f32,
        target_point: *const f32,
        target_motion_vector: *const f32,
        map_min: *const f32,
        map_max: *const f32,
        comms_weapon_lag: f32,
        source_is_gunship: c_int,
        has_draw: c_int,
        draw_sample: c_int,
        out: *mut f32,
        time_written: *mut c_int,
        echo: *mut RawEcho,
        need_draw: *mut c_int,
        fault: *mut c_int,
    ) -> c_int;
    #[allow(clippy::too_many_arguments)]
    fn eech_aim_vectors(
        table: *const RawTable,
        weapon: c_int,
        aiming_type: c_int,
        cruise_velocity: f32,
        acquire_forward: c_int,
        source_velocity: f32,
        device_position: *const f32,
        device_attitude: *const f32,
        target_position: *const f32,
        target_point: *const f32,
        target_motion_vector: *const f32,
        target_mobile: c_int,
        has_tracking: c_int,
        tracking_point: *const f32,
        map_min: *const f32,
        map_max: *const f32,
        comms_weapon_lag: f32,
        source_is_gunship: c_int,
        has_draw: c_int,
        draw_sample: c_int,
        out: *mut f32,
        echo: *mut RawEcho,
        need_draw: *mut c_int,
        fault: *mut c_int,
    ) -> c_int;
    fn eech_aim_person_face(position: *const f32, target_position: *const f32, attitude: *mut f32) -> f32;
}

/// One intermediate value of the model's prediction, in its order (reported for the campaign's test fixture trace).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Event {
    pub kind: i32,
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
    pub f: [f32; 12],
}

/// A package's limits on the offsets its devices are asked to reach: heading min and max, pitch min and max, and whether the weapon is a ground-launched
/// guided missile.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub heading: [f32; 2],
    pub pitch: [f32; 2],
    pub climb: bool,
}

/// The player's cannon tracking: a gun at `device_position` aimed at `point`, lowered by `pitch`.
#[derive(Clone, Copy, Debug)]
pub struct Track {
    pub device_position: [f32; 3],
    pub point: [f32; 3],
    pub pitch: f32,
}

/// The target facts a prediction is made from.
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub position: [f32; 3],
    pub point: [f32; 3],
    pub motion_vector: [f32; 3],
    pub mobile: bool,
}

/// The world figures a prediction is bounded by.
#[derive(Clone, Copy, Debug)]
pub struct World {
    pub map_min: [f32; 3],
    pub map_max: [f32; 3],
    pub comms_weapon_lag: f32,
}

/// The weapon figures the host aims with. `present` is false when there is no weapon.
#[derive(Clone, Copy, Debug)]
pub struct Weapon {
    pub weapon: i32,
    pub present: bool,
    pub aiming_type: i32,
    pub cruise_velocity: f32,
    pub acquire_forward: bool,
    pub source_velocity: f32,
}

/// What a prediction needs from the campaign's random stream: nothing yet, or the raw sample (an integer in [0, 32767]).
pub type Draw = Option<i32>;

#[derive(Clone, Debug)]
pub struct Vectors {
    pub valid: bool,
    pub weapon_vector: [f32; 3],
    pub target_vector: [f32; 3],
    pub intercept_vector: [f32; 3],
    pub intercept_range: f32,
    pub echo: Option<Vec<Event>>,
}

#[derive(Clone, Debug)]
pub struct Lead {
    pub ok: bool,
    pub intercept_point: [f32; 3],
    pub angle: f32,
    /// The flight time, only when the prediction produced one (a successful ballistic lookup). The off-map and tiny-range paths produce none and the model
    /// invents no value; a host that has a flight time of its own may send it.
    pub time_of_flight: Option<f32>,
    pub echo: Option<Vec<Event>>,
}

#[derive(Clone, Copy, Debug)]
pub struct Ballistic {
    pub ok: bool,
    pub angle: f32,
    pub time_of_flight: f32,
}

#[derive(Clone, Debug)]
pub struct Intercept {
    pub intercept_point: [f32; 3],
    pub echo: Option<Vec<Event>>,
}

/// A prediction that needed the campaign's random sample and had none yet.
#[derive(Clone, Debug)]
pub enum Outcome<T> {
    Done(T),
    NeedDraw,
}

struct Table {
    velocity_tests: i32,
    pitch_indices: i32,
    range_values: i32,
    max_range: f32,
    cells: Vec<f32>,
}

/// The host: the ballistic tables it generated and nothing else (and whether its range diagnostic file is open across a boot pass).
#[derive(Default)]
pub struct Aim {
    tables: HashMap<i32, Table>,
    range_file_open: bool,
    /// Every weapon's physical figures, by the weapon's number, as the campaign handed them over at boot (the weapon step flies a weapon by them).
    figures: HashMap<i32, flight::Figures>,
}

fn finite(what: &str, values: &[f32]) -> Result<(), String> {
    if values.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(format!("aim {what} must be finite"))
    }
}

fn echo_events(raw: &RawEcho) -> Result<Vec<Event>, String> {
    if raw.overflow != 0 {
        return Err("aim model echoed more events than it can carry".into());
    }
    Ok(raw.events[..raw.count as usize]
        .iter()
        .map(|event| Event {
            kind: event.kind,
            a: event.a,
            b: event.b,
            c: event.c,
            d: event.d,
            f: event.f,
        })
        .collect())
}

fn new_echo() -> Box<RawEcho> {
    // SAFETY: plain data; all-zero is a valid empty echo.
    Box::new(unsafe { std::mem::zeroed() })
}

impl Aim {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Generates one unguided weapon's ballistic table from its physical figures: this host simulates the weapon's flight at every pitch and velocity test
    /// (`flight.rs`) and keeps the table, replacing any the weapon had. `first` and `last` flag the ends of the campaign's boot pass: the diagnostic file the
    /// simulation writes is opened before the first weapon and closed after the last. Returns what each simulated flight left behind, in order.
    pub fn generate_table(&mut self, weapon: i32, figures: &flight::Figures, first: bool, last: bool) -> Result<Vec<flight::FlightRecord>, String> {
        if weapon <= 0 {
            return Err(format!("aim table for weapon {weapon} is not a weapon"));
        }
        finite("weapon figure", &figures.floats())?;
        finite("weapon figure", &figures.extra_floats())?;
        if figures.max_range <= 0.0 {
            return Err(format!("aim figures for weapon {weapon} have no range limit"));
        }
        let range_values = flight::table_range_values(figures.max_range).map_err(|error| format!("aim figures for weapon {weapon}: {error}"))? as i32;
        let mut cells = vec![0.0f32; flight::table_cell_count(figures.max_range).map_err(|error| format!("aim figures for weapon {weapon}: {error}"))?];
        if first {
            flight::ballistics_open();
        }
        let flights = flight::ballistics_weapon(figures, &mut cells);
        if last {
            flight::ballistics_close();
        }
        let flights = flights.map_err(|error| format!("weapon {weapon}: {error}"))?;
        if cells.iter().any(|cell| !cell.is_finite()) {
            return Err(format!("aim table for weapon {weapon} holds a non-finite cell"));
        }
        self.tables.insert(
            weapon,
            Table {
                velocity_tests: VELOCITY_TESTS,
                pitch_indices: PITCH_INDICES,
                range_values,
                max_range: figures.max_range,
                cells,
            },
        );
        Ok(flights)
    }

    /// Keeps one weapon's physical figures for its steps, replacing any it had.
    pub fn set_figures(&mut self, weapon: i32, figures: &flight::Figures) -> Result<(), String> {
        if weapon <= 0 {
            return Err(format!("aim figures for weapon {weapon} are not a weapon's"));
        }
        finite("weapon figure", &figures.floats())?;
        finite("weapon figure", &figures.extra_floats())?;
        finite("weapon figure", &figures.launch_floats())?;
        self.figures.insert(weapon, *figures);
        Ok(())
    }

    /// One pass of a weapon's step (`flight::weapon_step`) on the figures this host holds for it. An error when it holds none.
    pub fn weapon_step(&self, weapon: i32, input: &flight::StepInput, answers: &[flight::StepEvent], draws: &[i32]) -> Result<flight::StepOutcome, String> {
        let figures = self
            .figures
            .get(&weapon)
            .ok_or_else(|| format!("aim weapon step for weapon {weapon}: the host holds no figures for it"))?;
        flight::weapon_step(figures, input, answers, draws).map_err(|error| format!("weapon {weapon}: {error}"))
    }

    /// One pass of a weapon's birth (`flight::weapon_birth`) on the figures this host holds for it. An error when it holds none.
    pub fn weapon_birth(&self, weapon: i32, input: &flight::StepInput, answers: &[flight::StepEvent], draws: &[i32]) -> Result<flight::StepOutcome, String> {
        let figures = self
            .figures
            .get(&weapon)
            .ok_or_else(|| format!("aim weapon birth for weapon {weapon}: the host holds no figures for it"))?;
        flight::weapon_birth(figures, input, answers, draws).map_err(|error| format!("weapon {weapon}: {error}"))
    }

    /// One pass of a decoy's update (`flight::weapon_decoy`) on the figures this host holds for it. An error when it holds none.
    pub fn weapon_decoy(&self, weapon: i32, input: &flight::StepInput, answers: &[flight::StepEvent], draws: &[i32]) -> Result<flight::StepOutcome, String> {
        let figures = self
            .figures
            .get(&weapon)
            .ok_or_else(|| format!("aim decoy update for weapon {weapon}: the host holds no figures for it"))?;
        flight::weapon_decoy(figures, input, answers, draws).map_err(|error| format!("weapon {weapon}: {error}"))
    }

    /// One pass of a seeker's answer to the decoy `decoy` becoming active (`flight::weapon_seduce`), on the figures this host holds for both. An error when it holds none.
    pub fn weapon_seduce(
        &self,
        weapon: i32,
        decoy: i32,
        input: &flight::StepInput,
        answers: &[flight::StepEvent],
        draws: &[i32],
    ) -> Result<flight::StepOutcome, String> {
        let figures = self
            .figures
            .get(&weapon)
            .ok_or_else(|| format!("aim seduction of weapon {weapon}: the host holds no figures for it"))?;
        let decoy_figures = self
            .figures
            .get(&decoy)
            .ok_or_else(|| format!("aim seduction by decoy {decoy}: the host holds no figures for it"))?;
        flight::weapon_seduce(figures, decoy_figures, input, answers, draws).map_err(|error| format!("weapon {weapon}: {error}"))
    }

    /// One frame of a weapon's parachute (`flight::animate`) on the figures this host holds for it. An error when it holds none.
    pub fn animate(&self, weapon: i32, input: &flight::AnimateInput) -> Result<flight::AnimateOutcome, String> {
        let figures = self
            .figures
            .get(&weapon)
            .ok_or_else(|| format!("aim animation of weapon {weapon}: the host holds no figures for it"))?;
        flight::animate(figures, input).map_err(|error| format!("weapon {weapon}: {error}"))
    }

    /// The release of a weapon (`flight::release`) on the figures this host holds for it. An error when it holds none.
    pub fn release(&self, weapon: i32, input: &flight::ReleaseInput) -> Result<flight::ReleaseOutcome, String> {
        let figures = self
            .figures
            .get(&weapon)
            .ok_or_else(|| format!("aim release for weapon {weapon}: the host holds no figures for it"))?;
        flight::release(figures, input).map_err(|error| format!("weapon {weapon}: {error}"))
    }

    /// The cells of the table generated for a weapon (the test fixture build's echo asks for them).
    #[must_use]
    pub fn table_cells(&self, weapon: i32) -> Option<&[f32]> {
        self.tables.get(&weapon).map(|table| table.cells.as_slice())
    }

    /// Runs one guided weapon's two range simulations and writes the diagnostic file across the boot pass (opened before the first weapon, closed after the last).
    /// Returns the two runs' end states and whether the file was open.
    pub fn generate_range(&mut self, figures: &flight::Figures, first: bool, last: bool) -> Result<([flight::RangeRecord; 2], bool), String> {
        finite("weapon figure", &figures.floats())?;
        finite("weapon figure", &figures.extra_floats())?;
        if first {
            self.range_file_open = flight::range_open();
        }
        let open = self.range_file_open;
        let results = flight::range_weapon(figures);
        if last {
            flight::range_close();
            self.range_file_open = false;
        }
        Ok((results?, open))
    }

    /// Test helper: stores a table from raw cells (little-endian IEEE-754 single-precision (drop angle, flight time) pairs, velocity test by pitch index by
    /// range), as a custom host that brought its own tables would. The shape is the one the model indexes. The host's own tables come from `generate_table`.
    #[cfg(test)]
    pub fn set_table(&mut self, weapon: i32, max_range: f32, velocity_tests: i32, pitch_indices: i32, range_values: i32, cells: &[u8]) -> Result<(), String> {
        finite("table range limit", &[max_range])?;
        if weapon <= 0 {
            return Err(format!("aim table for weapon {weapon} is not a weapon"));
        }
        if velocity_tests != VELOCITY_TESTS || pitch_indices != PITCH_INDICES {
            return Err(format!(
                "aim table for weapon {weapon} has {velocity_tests} velocity tests and {pitch_indices} pitch indices, not the {VELOCITY_TESTS} and {PITCH_INDICES} the model indexes"
            ));
        }
        let expected_ranges = (f64::from(max_range) / 20.0) as i32 + 2;
        if range_values != expected_ranges {
            return Err(format!(
                "aim table for weapon {weapon} has {range_values} range values, not the {expected_ranges} its range limit {max_range} gives"
            ));
        }
        let count = (velocity_tests as usize) * (pitch_indices as usize) * (range_values as usize) * 2;
        if cells.len() != count * 4 {
            return Err(format!(
                "aim table for weapon {weapon} holds {} bytes, not the {} its shape needs",
                cells.len(),
                count * 4
            ));
        }
        let (words, _) = cells.as_chunks::<4>();
        let cells: Vec<f32> = words.iter().map(|word| f32::from_le_bytes(*word)).collect();
        if cells.iter().any(|cell| !cell.is_finite()) {
            return Err(format!("aim table for weapon {weapon} holds a non-finite cell"));
        }
        self.tables.insert(
            weapon,
            Table {
                velocity_tests,
                pitch_indices,
                range_values,
                max_range,
                cells,
            },
        );
        Ok(())
    }

    fn raw_table(&self, weapon: i32) -> RawTable {
        match self.tables.get(&weapon) {
            Some(table) => RawTable {
                present: 1,
                velocity_tests: table.velocity_tests,
                pitch_indices: table.pitch_indices,
                range_values: table.range_values,
                max_range: table.max_range,
                cells: table.cells.as_ptr(),
            },
            None => RawTable {
                present: 0,
                velocity_tests: 0,
                pitch_indices: 0,
                range_values: 0,
                max_range: 0.0,
                cells: std::ptr::null(),
            },
        }
    }

    /// One update of a weapon-system-ready keyframe: the advanced state and the phase it was in before (0 closed, 1 opening, 2 open, 3 closing).
    pub fn ready_update(&self, state: f32, rate: f32, delta_time: f32) -> Result<(f32, i32), String> {
        finite("ready figure", &[state, rate, delta_time])?;
        let mut advanced = 0.0f32;
        // SAFETY: a pure function of its arguments; the output is valid for the call.
        let phase = unsafe { eech_aim_ready_update(state, rate, delta_time, &mut advanced) };
        Ok((advanced, phase))
    }

    /// Opens or closes the weapon-system-ready devices: the new state.
    pub fn ready_command(&self, state: f32, open: bool) -> Result<f32, String> {
        finite("ready state", &[state])?;
        // SAFETY: a pure function of its arguments.
        Ok(unsafe { eech_aim_ready_command(state, c_int::from(open)) })
    }

    /// The offsets a device is asked to reach: heading, pitch. Toward `intercept_vector` within the package's `limits` (with the slight climb for a
    /// ground-launched guided missile), or with no limits as they are (the HUD's offsets); with `tracking`, those of a gun aimed at the tracking point.
    pub fn required(&self, attitude: [f32; 9], intercept_vector: [f32; 3], limits: Option<Limits>, tracking: Option<Track>) -> Result<(f32, f32), String> {
        finite("device attitude", &attitude)?;
        finite("intercept vector", &intercept_vector)?;
        let (mut heading, mut pitch) = (0.0f32, 0.0f32);
        match (limits, tracking) {
            (Some(limits), None) => {
                finite("device limits", &[limits.heading[0], limits.heading[1], limits.pitch[0], limits.pitch[1]])?;
                // SAFETY: a pure function of its arguments; the nine attitude floats and three vector floats are valid for the call.
                unsafe {
                    eech_aim_required(
                        attitude.as_ptr(),
                        intercept_vector.as_ptr(),
                        limits.heading[0],
                        limits.heading[1],
                        limits.pitch[0],
                        limits.pitch[1],
                        c_int::from(limits.climb),
                        &mut heading,
                        &mut pitch,
                    );
                }
            }
            (None, None) => {
                // SAFETY: as above.
                unsafe { eech_aim_required_unbounded(attitude.as_ptr(), intercept_vector.as_ptr(), &mut heading, &mut pitch) };
            }
            (None, Some(track)) => {
                finite("tracking", &[track.device_position, track.point].concat())?;
                finite("tracking pitch", &[track.pitch])?;
                // SAFETY: as above; the positions are three floats each.
                unsafe {
                    eech_aim_required_tracked(
                        attitude.as_ptr(),
                        track.device_position.as_ptr(),
                        track.point.as_ptr(),
                        track.pitch,
                        &mut heading,
                        &mut pitch,
                    );
                }
            }
            (Some(_), Some(_)) => return Err("aim required: a tracking request has no package limits".into()),
        }
        Ok((heading, pitch))
    }

    /// One rate-limited step toward the required offsets: delta heading, delta pitch, new heading, new pitch.
    #[allow(clippy::too_many_arguments)]
    pub fn step(&self, current: [f32; 2], required: [f32; 2], heading_limits: [f32; 2], rates: [f32; 2], delta_time: f32) -> Result<[f32; 4], String> {
        finite(
            "device figure",
            &[
                current[0],
                current[1],
                required[0],
                required[1],
                heading_limits[0],
                heading_limits[1],
                rates[0],
                rates[1],
                delta_time,
            ],
        )?;
        let mut out = [0.0f32; 4];
        // SAFETY: a pure function of its arguments; the output is valid for the call.
        unsafe {
            eech_aim_step(
                current[0],
                current[1],
                required[0],
                required[1],
                heading_limits[0],
                heading_limits[1],
                rates[0],
                rates[1],
                delta_time,
                out.as_mut_ptr(),
            );
        }
        Ok(out)
    }

    /// One table interpolation.
    pub fn ballistic(
        &self,
        weapon: i32,
        range: f32,
        height_diff_or_pitch: f32,
        simplified: bool,
        fixed_pitch: bool,
        parent_velocity: f32,
    ) -> Result<Ballistic, String> {
        finite("ballistic figure", &[range, height_diff_or_pitch, parent_velocity])?;
        self.require_table(weapon)?;
        let table = self.raw_table(weapon);
        let (mut angle, mut time_of_flight, mut fault) = (0.0f32, 0.0f32, 0);
        // SAFETY: the table and its cells are valid for the call; the outputs are local.
        let ok = unsafe {
            eech_aim_ballistic(
                &table,
                range,
                height_diff_or_pitch,
                c_int::from(simplified),
                c_int::from(fixed_pitch),
                parent_velocity,
                &mut angle,
                &mut time_of_flight,
                &mut fault,
            )
        } != 0;
        if fault != 0 {
            return Err(self.fault_text(weapon));
        }
        Ok(Ballistic { ok, angle, time_of_flight })
    }

    fn require_table(&self, weapon: i32) -> Result<(), String> {
        if self.tables.contains_key(&weapon) {
            Ok(())
        } else {
            Err(format!("aim has no ballistic table for weapon {weapon}"))
        }
    }

    fn fault_text(&self, weapon: i32) -> String {
        if self.tables.contains_key(&weapon) {
            format!("aim ballistic table of weapon {weapon} does not cover the request")
        } else {
            format!("aim has no ballistic table for weapon {weapon}")
        }
    }

    /// The guided-weapon intercept prediction alone.
    pub fn intercept(&self, device_position: [f32; 3], weapon: &Weapon, target: &Target, echo: bool) -> Result<Intercept, String> {
        finite("device position", &device_position)?;
        finite("weapon figure", &[weapon.cruise_velocity, weapon.source_velocity])?;
        finite("target", &[target.position, target.point, target.motion_vector].concat())?;
        let mut point = [0.0f32; 3];
        let mut raw = echo.then(new_echo);
        // SAFETY: a pure function of its arguments; the echo, when asked for, is valid for the call.
        unsafe {
            eech_aim_intercept(
                device_position.as_ptr(),
                c_int::from(weapon.present),
                weapon.cruise_velocity,
                c_int::from(weapon.acquire_forward),
                weapon.source_velocity,
                target.position.as_ptr(),
                target.point.as_ptr(),
                target.motion_vector.as_ptr(),
                point.as_mut_ptr(),
                raw.as_mut().map_or(std::ptr::null_mut(), |echo| &mut **echo),
            );
        }
        Ok(Intercept {
            intercept_point: point,
            echo: raw.as_deref().map(echo_events).transpose()?,
        })
    }

    /// The lead and ballistic prediction alone.
    #[allow(clippy::too_many_arguments)]
    pub fn lead(
        &self,
        weapon: i32,
        device_position: [f32; 3],
        parent_velocity: f32,
        target: &Target,
        world: &World,
        source_gunship: bool,
        draw: Draw,
        echo: bool,
    ) -> Result<Outcome<Lead>, String> {
        finite("device position", &device_position)?;
        finite("parent velocity", &[parent_velocity])?;
        finite("target", &[target.position, target.point, target.motion_vector].concat())?;
        finite("world", &[world.map_min, world.map_max].concat())?;
        finite("comms weapon lag", &[world.comms_weapon_lag])?;
        self.require_table(weapon)?;
        let table = self.raw_table(weapon);
        let mut out = [0.0f32; 5];
        let (mut need_draw, mut fault, mut time_written) = (0, 0, 0);
        let mut raw = echo.then(new_echo);
        // SAFETY: the table and its cells are valid for the call; the outputs and the echo are local.
        let ok = unsafe {
            eech_aim_lead(
                &table,
                weapon,
                device_position.as_ptr(),
                parent_velocity,
                target.point.as_ptr(),
                target.motion_vector.as_ptr(),
                world.map_min.as_ptr(),
                world.map_max.as_ptr(),
                world.comms_weapon_lag,
                c_int::from(source_gunship),
                c_int::from(draw.is_some()),
                draw.unwrap_or(0),
                out.as_mut_ptr(),
                &mut time_written,
                raw.as_mut().map_or(std::ptr::null_mut(), |echo| &mut **echo),
                &mut need_draw,
                &mut fault,
            )
        } != 0;
        if fault != 0 {
            return Err(self.fault_text(weapon));
        }
        if need_draw != 0 {
            return Ok(Outcome::NeedDraw);
        }
        Ok(Outcome::Done(Lead {
            ok,
            intercept_point: [out[0], out[1], out[2]],
            angle: out[3],
            time_of_flight: (time_written != 0).then_some(out[4]),
            echo: raw.as_deref().map(echo_events).transpose()?,
        }))
    }

    /// The pitch device's weapon, target and intercept vectors for a launcher with a weapon and a target (`weapon` and `target` set), or from the
    /// player's tracking point.
    #[allow(clippy::too_many_arguments)]
    pub fn vectors(
        &self,
        device_position: [f32; 3],
        device_attitude: [f32; 9],
        tracking: Option<[f32; 3]>,
        weapon: Option<&Weapon>,
        target: Option<&Target>,
        world: &World,
        source_gunship: bool,
        draw: Draw,
        echo: bool,
    ) -> Result<Outcome<Vectors>, String> {
        finite("device position", &device_position)?;
        finite("device attitude", &device_attitude)?;
        finite("world", &[world.map_min, world.map_max].concat())?;
        finite("comms weapon lag", &[world.comms_weapon_lag])?;
        let nothing_weapon = Weapon {
            weapon: 0,
            present: false,
            aiming_type: 0,
            cruise_velocity: 0.0,
            acquire_forward: false,
            source_velocity: 0.0,
        };
        let nothing_target = Target {
            position: [0.0; 3],
            point: [0.0; 3],
            motion_vector: [0.0; 3],
            mobile: false,
        };
        let weapon = weapon.copied_or(nothing_weapon);
        let target = target.copied_or(nothing_target);
        finite("weapon figure", &[weapon.cruise_velocity, weapon.source_velocity])?;
        finite("target", &[target.position, target.point, target.motion_vector].concat())?;
        let tracking_point = tracking.unwrap_or([0.0; 3]);
        finite("tracking point", &tracking_point)?;
        // the unguided weapons are aimed through their table: a host without it cannot answer, and says so
        if tracking.is_none() && matches!(weapon.aiming_type, 2 | 3) {
            self.require_table(weapon.weapon)?;
        }
        let table = self.raw_table(weapon.weapon);
        let mut out = [0.0f32; 16];
        let (mut need_draw, mut fault) = (0, 0);
        let mut raw = echo.then(new_echo);
        // SAFETY: the table and its cells are valid for the call; the outputs and the echo are local.
        let valid = unsafe {
            eech_aim_vectors(
                &table,
                weapon.weapon,
                weapon.aiming_type,
                weapon.cruise_velocity,
                c_int::from(weapon.acquire_forward),
                weapon.source_velocity,
                device_position.as_ptr(),
                device_attitude.as_ptr(),
                target.position.as_ptr(),
                target.point.as_ptr(),
                target.motion_vector.as_ptr(),
                c_int::from(target.mobile),
                c_int::from(tracking.is_some()),
                tracking_point.as_ptr(),
                world.map_min.as_ptr(),
                world.map_max.as_ptr(),
                world.comms_weapon_lag,
                c_int::from(source_gunship),
                c_int::from(draw.is_some()),
                draw.unwrap_or(0),
                out.as_mut_ptr(),
                raw.as_mut().map_or(std::ptr::null_mut(), |echo| &mut **echo),
                &mut need_draw,
                &mut fault,
            )
        } != 0;
        if fault != 0 {
            return Err(self.fault_text(weapon.weapon));
        }
        if need_draw != 0 {
            return Ok(Outcome::NeedDraw);
        }
        let mut echo = raw.as_deref().map(echo_events).transpose()?;
        if let Some(events) = echo.as_mut() {
            // the lead prediction's own outputs, which the answer to a vectors request does not carry (the trace of the prediction wants them)
            if tracking.is_none() && matches!(weapon.aiming_type, 2 | 3) {
                let mut f = [0.0f32; 12];
                f[..5].copy_from_slice(&[out[12], out[13], out[14], out[10], out[11]]);
                events.push(Event {
                    kind: 6,
                    a: i32::from(out[15] != 0.0), // whether the prediction produced a time of flight
                    b: 0,
                    c: 0,
                    d: i32::from(valid),
                    f,
                });
            }
        }
        Ok(Outcome::Done(Vectors {
            valid,
            weapon_vector: [out[0], out[1], out[2]],
            target_vector: [out[3], out[4], out[5]],
            intercept_vector: [out[6], out[7], out[8]],
            intercept_range: out[9],
            echo,
        }))
    }
}

/// An infantry member's attitude when it turns to face a target: the heading and the heading matrix. A pure function of the person's position and
/// the target's; the movement record of the person (`movement.rs`) owns the attitude it produces.
pub fn person_face(position: [f32; 3], target_position: [f32; 3]) -> Result<(f32, [f32; 9]), String> {
    finite("person position", &position)?;
    finite("target position", &target_position)?;
    let mut attitude = [0.0f32; 9];
    // SAFETY: a pure function of its arguments; the output is valid for the call.
    let heading = unsafe { eech_aim_person_face(position.as_ptr(), target_position.as_ptr(), attitude.as_mut_ptr()) };
    Ok((heading, attitude))
}

trait CopiedOr<T> {
    fn copied_or(self, fallback: T) -> T;
}

impl<T: Copy> CopiedOr<T> for Option<&T> {
    fn copied_or(self, fallback: T) -> T {
        self.copied().unwrap_or(fallback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEVEL: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1.0e-6
    }

    fn zero_table(weapon: i32, max_range: f32) -> Aim {
        let ranges = (f64::from(max_range) / 20.0) as i32 + 2;
        let mut aim = Aim::new();
        let cells = vec![0u8; (VELOCITY_TESTS * PITCH_INDICES * ranges * 2) as usize * 4];
        aim.set_table(weapon, max_range, VELOCITY_TESTS, PITCH_INDICES, ranges, &cells).unwrap();
        aim
    }

    #[test]
    fn the_ready_keyframe_opens_holds_closes_and_wraps_as_the_original_did() {
        let aim = Aim::new();
        assert_eq!(aim.ready_command(0.0, true).unwrap(), 1.0);
        assert_eq!(aim.ready_command(1.5, true).unwrap(), 1.5, "an opening keyframe keeps its place");
        assert_eq!(
            aim.ready_command(3.25, true).unwrap(),
            2.0 - 0.25,
            "a closing keyframe reverses from where it is"
        );
        assert_eq!(aim.ready_command(2.0, false).unwrap(), 3.0);
        assert_eq!(aim.ready_command(1.25, false).unwrap(), 3.75, "an opening keyframe closes from where it is");
        assert_eq!(aim.ready_update(1.0, 0.5, 0.5).unwrap(), (1.25, 1));
        assert_eq!(aim.ready_update(1.75, 0.5, 1.0).unwrap(), (2.0, 1), "opening stops at open");
        assert_eq!(aim.ready_update(2.0, 0.5, 1.0).unwrap(), (2.0, 2), "open holds");
        assert_eq!(aim.ready_update(3.5, 0.5, 2.0).unwrap(), (0.0, 3), "closing wraps to closed");
        assert_eq!(aim.ready_update(0.0, 0.5, 1.0).unwrap(), (0.0, 0), "closed holds");
        assert!(aim.ready_update(f32::NAN, 0.5, 1.0).is_err());
    }

    #[test]
    fn the_required_offsets_are_the_bearing_of_the_intercept_vector_in_the_device_frame_within_its_limits() {
        let aim = Aim::new();
        let half = std::f32::consts::FRAC_1_SQRT_2;
        let limits = |heading: [f32; 2], pitch: [f32; 2], climb: bool| Some(Limits { heading, pitch, climb });
        let (heading, pitch) = aim.required(LEVEL, [0.0, 0.0, 1.0], limits([-1.0, 1.0], [-1.0, 1.0], false), None).unwrap();
        assert!(near(heading, 0.0) && near(pitch, 0.0));
        let (heading, _) = aim.required(LEVEL, [half, 0.0, half], limits([-1.0, 1.0], [-1.0, 1.0], false), None).unwrap();
        assert!(near(heading, std::f32::consts::FRAC_PI_4), "{heading}");
        let (heading, _) = aim.required(LEVEL, [half, 0.0, half], limits([-0.5, 0.5], [-1.0, 1.0], false), None).unwrap();
        assert!(near(heading, 0.5), "the heading is bounded by the package's limit: {heading}");
        // a ground-launched guided missile is aimed at a slight climb: the pitch floor is at least ten degrees
        let (_, pitch) = aim.required(LEVEL, [0.0, 0.0, 1.0], limits([-1.0, 1.0], [-0.5, 0.5], true), None).unwrap();
        assert!(near(pitch, 10.0f32.to_radians()), "{pitch}");
        assert!(aim
            .required(LEVEL, [0.0, f32::INFINITY, 1.0], limits([-1.0, 1.0], [-1.0, 1.0], false), None)
            .is_err());
        // with no limits the offsets come back as they are (the HUD's), and a gun aimed at a tracking point is lowered by the ballistic pitch (the player's)
        let (heading, pitch) = aim.required(LEVEL, [half, half, 0.0], None, None).unwrap();
        assert!(
            near(heading, std::f32::consts::FRAC_PI_2) && near(pitch, std::f32::consts::FRAC_PI_4),
            "{heading} {pitch}"
        );
        let track = |point: [f32; 3], pitch: f32| {
            Some(Track {
                device_position: [0.0; 3],
                point,
                pitch,
            })
        };
        let (heading, pitch) = aim.required(LEVEL, [0.0; 3], None, track([100.0, 0.0, 0.0], 0.0)).unwrap();
        assert!(near(heading, std::f32::consts::FRAC_PI_2) && near(pitch, 0.0), "{heading} {pitch}");
        let (heading, pitch) = aim.required(LEVEL, [0.0; 3], None, track([0.0, 5.0, 100.0], 0.5)).unwrap();
        assert!(
            near(heading, 0.0) && near(pitch.abs(), 0.5),
            "the tracking point's height is ignored; the ballistic pitch is the gun's elevation: {heading} {pitch}"
        );
        assert!(aim
            .required(LEVEL, [0.0; 3], limits([-1.0, 1.0], [-1.0, 1.0], false), track([1.0, 0.0, 1.0], 0.0))
            .is_err());
    }

    #[test]
    fn a_device_steps_at_most_its_rate_times_the_time_and_a_full_circle_device_takes_the_short_way_round() {
        let aim = Aim::new();
        let pi = std::f32::consts::PI;
        let step = aim.step([0.0, 0.0], [1.0, -1.0], [-1.0, 1.0], [0.5, 0.25], 0.5).unwrap();
        assert!(
            near(step[0], 0.25) && near(step[1], -0.125) && near(step[2], 0.25) && near(step[3], -0.125),
            "{step:?}"
        );
        let step = aim.step([0.0, 0.0], [0.1, 0.1], [-1.0, 1.0], [10.0, 10.0], 1.0).unwrap();
        assert!(near(step[0], 0.1) && near(step[2], 0.1), "a device within reach arrives: {step:?}");
        // a device that can turn the whole circle goes from just under +pi to just over -pi by the short way and wraps its heading
        let step = aim.step([3.0, 0.0], [-3.0, 0.0], [-pi, pi], [10.0, 10.0], 1.0).unwrap();
        assert!(near(step[0], 2.0 * pi - 6.0) && near(step[2], -3.0), "{step:?}");
        // a device with a limited arc does not: it turns back through the middle
        let step = aim.step([1.0, 0.0], [-1.0, 0.0], [-2.0, 2.0], [10.0, 10.0], 1.0).unwrap();
        assert!(near(step[0], -2.0) && near(step[2], -1.0), "{step:?}");
    }

    #[test]
    fn a_ballistic_request_without_the_weapons_table_is_refused_and_a_point_blank_one_has_no_solution() {
        let aim = Aim::new();
        let error = aim.ballistic(7, 50.0, 0.0, false, false, 0.0).unwrap_err();
        assert!(error.contains("no ballistic table for weapon 7"), "{error}");
        let aim = zero_table(7, 100.0);
        // the height above or below exceeds the range: the original took an arcsine of more than one (patch W1); the model has no solution
        let answer = aim.ballistic(7, 10.0, 11.0, false, false, 0.0).unwrap();
        assert!(!answer.ok);
        let answer = aim.ballistic(7, 10.0, -10.0, false, false, 0.0).unwrap();
        assert!(!answer.ok, "the boundary itself has no solution either");
        assert!(aim.ballistic(7, f32::NAN, 0.0, false, false, 0.0).is_err());
    }

    #[test]
    fn a_table_of_another_shape_than_the_model_indexes_is_refused_not_misread() {
        let mut aim = Aim::new();
        assert!(aim
            .set_table(7, 100.0, 4, PITCH_INDICES, 7, &[0; 4 * 85 * 7 * 2 * 4])
            .unwrap_err()
            .contains("velocity tests"));
        assert!(aim
            .set_table(7, 100.0, VELOCITY_TESTS, PITCH_INDICES, 6, &[0; 5 * 85 * 6 * 2 * 4])
            .unwrap_err()
            .contains("range values"));
        assert!(aim
            .set_table(7, 100.0, VELOCITY_TESTS, PITCH_INDICES, 7, &[0; 12])
            .unwrap_err()
            .contains("bytes"));
        assert!(aim.set_table(0, 100.0, VELOCITY_TESTS, PITCH_INDICES, 7, &[0; 5 * 85 * 7 * 2 * 4]).is_err());
        let mut cells = vec![0u8; 5 * 85 * 7 * 2 * 4];
        cells[8..12].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(aim
            .set_table(7, 100.0, VELOCITY_TESTS, PITCH_INDICES, 7, &cells)
            .unwrap_err()
            .contains("non-finite"));
    }

    #[test]
    fn a_lead_request_for_a_target_at_the_device_has_no_solution_and_the_prediction_is_the_target_point() {
        let aim = zero_table(7, 100.0);
        let target = Target {
            position: [10.0, 0.0, 10.0],
            point: [10.0, 0.0, 10.0],
            motion_vector: [0.0; 3],
            mobile: true,
        };
        let world = World {
            map_min: [0.0; 3],
            map_max: [1.0e5; 3],
            comms_weapon_lag: 0.0,
        };
        let Outcome::Done(lead) = aim.lead(7, [10.0, 0.0, 10.0], 0.0, &target, &world, false, None, false).unwrap() else {
            panic!("a request that reaches no random jitter asks for no draw")
        };
        assert!(!lead.ok, "a range under a millimetre has no solution");
        assert_eq!(lead.intercept_point, [10.0, 0.0, 10.0]);
        assert_eq!(lead.angle, 0.0);
        assert!(lead.echo.is_none(), "an echo is produced only when it is asked for");
    }

    fn lead_request(aim: &Aim, device: [f32; 3], point: [f32; 3], parent_velocity: f32) -> Lead {
        let target = Target {
            position: point,
            point,
            motion_vector: [0.0; 3],
            mobile: true,
        };
        let world = World {
            map_min: [0.0; 3],
            map_max: [1.0e5; 3],
            comms_weapon_lag: 0.0,
        };
        match aim.lead(7, device, parent_velocity, &target, &world, false, None, false).unwrap() {
            Outcome::Done(lead) => lead,
            Outcome::NeedDraw => panic!("a stationary target reaches no random jitter"),
        }
    }

    #[test]
    fn the_flight_time_of_a_lead_is_a_partial_output_that_the_model_never_invents() {
        let aim = zero_table(7, 100.0);
        // a stationary target inside the map: a ballistic lookup succeeds, so the model produced a flight time
        let normal = lead_request(&aim, [0.0, 0.0, 0.0], [500.0, 0.0, 0.0], 0.0);
        assert!(normal.ok && normal.time_of_flight.is_some(), "{normal:?}");
        // off the map the solver takes the projectile-angle fall-back, which succeeds and writes no flight time: none is reported, and it is not zero
        let off_map = lead_request(&aim, [198_500.0, 0.0, 0.0], [200_000.0, 0.0, 0.0], 500.0);
        assert!(off_map.ok, "the fall-back found an angle for a target within reach: {off_map:?}");
        assert!(off_map.time_of_flight.is_none(), "the off-map path produces no flight time: {off_map:?}");
        // a range under a millimetre fails before any lookup: none either
        let tiny = lead_request(&aim, [10.0, 0.0, 10.0], [10.0, 0.0, 10.0], 0.0);
        assert!(!tiny.ok && tiny.time_of_flight.is_none(), "{tiny:?}");
        // a target exactly above the device: the 3-D range equals the height, the original's strict guard (patch W1: the height must be less than the range) finds no
        // solution, and the failed lookup produced no flight time. (A target 5000 m up and 10 m aside has a range of 5000.0098, passes the guard, and is not this case.)
        let guarded = lead_request(&aim, [0.0, 0.0, 0.0], [0.0, 5000.0, 0.0], 0.0);
        assert!(!guarded.ok && guarded.time_of_flight.is_none(), "{guarded:?}");
        let not_guarded = lead_request(&aim, [0.0, 0.0, 0.0], [0.0, 5000.0, 10.0], 0.0);
        assert!(not_guarded.ok && not_guarded.time_of_flight.is_some(), "{not_guarded:?}");
    }

    #[test]
    fn a_person_faces_the_bearing_to_the_target_in_the_horizontal_plane() {
        let (heading, attitude) = person_face([0.0, 5.0, 0.0], [0.0, 900.0, 10.0]).unwrap();
        assert!(near(heading, 0.0), "height is ignored");
        assert!(attitude.iter().zip(LEVEL).all(|(a, b)| near(*a, b)), "{attitude:?}");
        let (heading, _) = person_face([0.0; 3], [-10.0, 0.0, 0.0]).unwrap();
        assert!(near(heading, -std::f32::consts::FRAC_PI_2), "{heading}");
        assert!(person_face([0.0; 3], [f32::NAN, 0.0, 0.0]).is_err());
    }

    /// A small gun: 900 m/s at the muzzle, a 1 km limit, drag and a 20 mm calibre; no rocket motor, so the lifetime is the cruise time.
    fn gun(name: &str) -> flight::Figures {
        // SAFETY: plain data; all-zero is a valid value of every field.
        let mut figures: flight::Figures = unsafe { std::mem::zeroed() };
        figures.set_name(name).unwrap();
        figures.start_weight = 0.1;
        figures.final_weight = 0.1;
        figures.max_range = 1000.0;
        figures.muzzle_velocity = 900.0;
        figures.cruise_velocity = 900.0;
        figures.cruise_time = 4.0;
        figures.drag_coefficient = 0.3;
        figures.diameter = 0.02;
        figures.weapon_class = 1 << 3;
        figures
    }

    /// The (drop angle, flight time) pair of one cell: velocity test, pitch index, range index.
    fn cell(aim: &Aim, weapon: i32, test: usize, pitch: usize, range: usize) -> (f32, f32) {
        let table = aim.tables.get(&weapon).unwrap();
        let at = ((test * PITCH_INDICES as usize + pitch) * table.range_values as usize + range) * 2;
        (table.cells[at], table.cells[at + 1])
    }

    #[test]
    fn the_host_generates_the_table_from_the_figures_in_the_shape_the_model_indexes_and_repeats_it_bit_for_bit() {
        let mut aim = Aim::new();
        let flights = aim.generate_table(7, &gun("test gun"), false, false).unwrap();
        let table = aim.tables.get(&7).unwrap();
        assert_eq!(
            (table.velocity_tests, table.pitch_indices, table.range_values),
            (VELOCITY_TESTS, PITCH_INDICES, 52)
        );
        assert_eq!(table.cells.len(), 5 * 85 * 52 * 2);
        assert!(table.cells.iter().all(|cell| cell.is_finite()));
        // a surface-to-surface weapon (class bit 3) simulates only the first velocity test of each pitch and copies it for the other four
        assert_eq!(flights.len(), 83, "{} flights", flights.len());
        assert!(flights
            .iter()
            .all(|flight| flight.velocity_test == 0 && flight.steps > 0 && flight.range_mark <= flight.range_values));
        for test in 1..5 {
            for pitch in 0..85 {
                for range in 0..52 {
                    assert_eq!(
                        cell(&aim, 7, test, pitch, range),
                        cell(&aim, 7, 0, pitch, range),
                        "test {test} pitch {pitch} range {range}"
                    );
                }
            }
        }
        // the rows at plus and minus ninety degrees are all zero (the original never simulates them)
        assert!((0..52).all(|range| cell(&aim, 7, 0, 0, range) == (0.0, 0.0) && cell(&aim, 7, 0, 84, range) == (0.0, 0.0)));
        // a level shot falls and takes longer to reach a farther mark
        let level = 42;
        let (drop_near, time_near) = cell(&aim, 7, 0, level, 10);
        let (drop_far, time_far) = cell(&aim, 7, 0, level, 40);
        assert!(
            time_far > time_near && time_near > 0.0 && drop_far > drop_near,
            "{drop_near} {time_near} {drop_far} {time_far}"
        );
        // the same figures give the same table, bit for bit, and the host keeps only the latest
        let mut again = Aim::new();
        again.generate_table(7, &gun("test gun"), false, false).unwrap();
        assert_eq!(
            aim.table_cells(7).unwrap().iter().map(|cell| cell.to_bits()).collect::<Vec<_>>(),
            again.table_cells(7).unwrap().iter().map(|cell| cell.to_bits()).collect::<Vec<_>>()
        );
        assert!(aim.table_cells(8).is_none());
    }

    #[test]
    fn the_generated_table_follows_the_figures_a_host_is_given() {
        let mut reference = Aim::new();
        reference.generate_table(7, &gun("test gun"), false, false).unwrap();
        let mut slow = gun("slow gun");
        slow.muzzle_velocity = 450.0;
        let mut draggy = gun("draggy gun");
        draggy.drag_coefficient = 0.9;
        let mut other = Aim::new();
        other.generate_table(7, &slow, false, false).unwrap();
        other.generate_table(8, &draggy, false, false).unwrap();
        let level = 42;
        let (_, reference_time) = cell(&reference, 7, 0, level, 40);
        let (_, slow_time) = cell(&other, 7, 0, level, 40);
        assert!(
            slow_time > reference_time * 1.5,
            "half the muzzle velocity takes much longer: {slow_time} against {reference_time}"
        );
        let (_, draggy_time) = cell(&other, 8, 0, level, 40);
        assert!(draggy_time > reference_time, "more drag takes longer: {draggy_time} against {reference_time}");
        // and the lookup the aiming uses answers differently from each table
        let reference_ballistic = reference.ballistic(7, 800.0, 0.0, false, false, 0.0).unwrap();
        let slow_ballistic = other.ballistic(7, 800.0, 0.0, false, false, 0.0).unwrap();
        assert!(reference_ballistic.ok && slow_ballistic.ok);
        assert!(
            slow_ballistic.angle > reference_ballistic.angle,
            "{} {}",
            slow_ballistic.angle,
            reference_ballistic.angle
        );
    }

    #[test]
    fn figures_that_cannot_be_a_weapon_are_refused_without_a_table() {
        let mut aim = Aim::new();
        assert!(aim.generate_table(0, &gun("test gun"), false, false).unwrap_err().contains("not a weapon"));
        let mut infinite = gun("test gun");
        infinite.drag_coefficient = f32::INFINITY;
        assert!(aim.generate_table(7, &infinite, false, false).unwrap_err().contains("finite"));
        let mut rangeless = gun("test gun");
        rangeless.max_range = 0.0;
        assert!(aim.generate_table(7, &rangeless, false, false).unwrap_err().contains("range limit"));
        assert!(aim.table_cells(7).is_none());
        let mut figures = gun("test gun");
        assert!(figures.set_name("").is_err());
        assert!(figures.set_name(&"x".repeat(64)).is_err());
        assert!(figures.set_name("a\u{0}b").is_err());
    }

    // ---------------------------------------------------------------------------------------------------------------------------------------------------------
    // the weapon step (EEDCS-45): a frame of a weapon's flight, a pure question of its state and the answers to the lookups it asked for

    const LOOKUP_VIEW_WEAPON: i32 = 1;
    const LOOKUP_LAUNCHER_EO: i32 = 2;
    const LOOKUP_LAUNCHER_INT: i32 = 3;
    const LOOKUP_TARGET_INT: i32 = 5;
    const LOOKUP_TARGET_VEC: i32 = 6;
    const LOOKUP_WIND: i32 = 7;
    const LOOKUP_GROUND_PROBE: i32 = 8;
    const LOOKUP_GROUND_IMPACT: i32 = 9;
    const LOOKUP_FACE_NORMAL: i32 = 10;
    const LOOKUP_COLLISION: i32 = 12;
    const ACTION_STATE: i32 = 1;
    const ACTION_ATTITUDE: i32 = 2;
    const ACTION_TARGET_LINK: i32 = 6;
    const ACTION_POSITION: i32 = 5;
    const IDENTITY: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

    /// A heavy, thick round with a short motor: a weapon whose weight is defined for a positive lifetime (the original takes a motor's times as non-zero for any weapon
    /// that lives), so the drag it flies by is measurable in one frame.
    fn flying_gun(name: &str) -> flight::Figures {
        let mut figures = gun(name);
        figures.start_weight = 5.0;
        figures.final_weight = 5.0;
        figures.diameter = 0.1;
        figures.boost_time = 1.0;
        figures.sustain_time = 2.0;
        figures.boost_power = 1.0;
        figures.sustain_power = 1.0;
        figures
    }

    /// A round in flight over open ground, in a map a hundred kilometres across, level at 500 m and pointing along +z.
    fn round() -> flight::StepInput {
        flight::StepInput {
            delta_time: 0.1,
            system_time: 1000,
            position: [0.0, 500.0, 0.0],
            attitude: IDENTITY,
            motion_vector: [0.0; 3],
            velocity: 900.0,
            weapon_lifetime: 3.0,
            decoy_timer: 0.0,
            missile_phase: 0,
            loal_mode: false,
            weapon_on_ground: false,
            parachute_status: 0,
            kill_code: 0,
            entity_samples: [100, 200, 300, 400],
            has_launcher: false,
            has_target: false,
            server: true,
            submunition: false,
            coefficient: 0.0,
            effectiveness: [0.0; 3],
            map_min: [-100_000.0, -1000.0, -100_000.0],
            map_max: [100_000.0, 10_000.0, 100_000.0],
            echo: false,
        }
    }

    /// What a world answers: the results r0.. of a lookup.
    type Answerer<'a> = &'a mut dyn FnMut(&flight::StepEvent) -> [f32; 6];

    fn open_world(lookup: &flight::StepEvent) -> [f32; 6] {
        let _ = lookup;
        [0.0; 6]
    }

    /// Asks the host for one step until it is done, answering what it asks from `world` and giving it `draws` in order; returns what it left and the kinds it asked, in order.
    fn fly(aim: &Aim, weapon: i32, input: &flight::StepInput, world: Answerer, draws: &[i32]) -> (flight::StepOutcome, Vec<i32>) {
        let mut answers: Vec<flight::StepEvent> = Vec::new();
        let mut given: Vec<i32> = Vec::new();
        let mut asked = Vec::new();
        for _ in 0..64 {
            let outcome = aim.weapon_step(weapon, input, &answers, &given).unwrap();
            match outcome.state {
                flight::StepState::Done => return (outcome, asked),
                flight::StepState::NeedDraw => given.push(draws[given.len()]),
                flight::StepState::NeedLookup => {
                    assert!(outcome.actions.is_empty(), "a step that is not done leaves no action");
                    for lookup in outcome.lookups {
                        asked.push(lookup.kind);
                        let mut answer = lookup;
                        answer.f[6..].copy_from_slice(&world(&lookup));
                        answers.push(answer);
                    }
                }
            }
        }
        panic!("the step did not finish");
    }

    fn last_action(outcome: &flight::StepOutcome, kind: i32) -> Option<flight::StepEvent> {
        outcome.actions.iter().rev().find(|action| action.kind == kind).copied()
    }

    fn step_aim(figures: &flight::Figures) -> Aim {
        let mut aim = Aim::new();
        aim.set_figures(5, figures).unwrap();
        aim
    }

    #[test]
    fn a_free_round_flies_a_frame_by_the_figures_and_asks_only_for_the_world_it_meets() {
        let aim = step_aim(&flying_gun("test gun"));
        let input = round();
        // with nothing answered it stops for lookups and has changed nothing
        let first = aim.weapon_step(5, &input, &[], &[]).unwrap();
        assert_eq!(first.state, flight::StepState::NeedLookup);
        assert!(first.actions.is_empty() && !first.lookups.is_empty());
        let (outcome, asked) = fly(&aim, 5, &input, &mut open_world, &[]);
        assert!(
            asked.contains(&LOOKUP_WIND) && asked.contains(&LOOKUP_VIEW_WEAPON) && asked.contains(&LOOKUP_GROUND_PROBE),
            "{asked:?}"
        );
        assert!(
            !asked.contains(&LOOKUP_LAUNCHER_EO) && !asked.contains(&LOOKUP_TARGET_INT) && !asked.contains(&LOOKUP_COLLISION),
            "{asked:?}"
        );
        let state = last_action(&outcome, ACTION_STATE).unwrap();
        assert_eq!((state.a, state.b, state.c), (0, 0, 0), "no phase change, not on the ground, no kill code");
        assert_eq!(state.f[4].to_bits(), (3.0f32 - 0.1f32).to_bits(), "the lifetime runs down by exactly the frame");
        assert!(state.f[0] < 900.0 && state.f[0] > 800.0, "drag slows the round: {}", state.f[0]);
        let position = last_action(&outcome, ACTION_POSITION).unwrap();
        assert!(position.f[2] > 80.0 && position.f[2] < 90.0, "{:?}", position.f);
        assert_eq!(position.f[1], 500.0, "gravity acts after the move");
        let attitude = last_action(&outcome, ACTION_ATTITUDE).unwrap();
        assert!(attitude.f[7] < 0.0, "gravity pitches the round down: {:?}", attitude.f);
        // the same question gives the same answer, bit for bit
        let (again, _) = fly(&aim, 5, &input, &mut open_world, &[]);
        assert_eq!(
            outcome.actions.iter().map(|action| action.f.map(f32::to_bits)).collect::<Vec<_>>(),
            again.actions.iter().map(|action| action.f.map(f32::to_bits)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn the_host_flies_the_weapon_by_the_figures_it_was_given() {
        let reference = step_aim(&flying_gun("test gun"));
        let mut draggy = flying_gun("draggy gun");
        draggy.drag_coefficient = 1.2;
        let other = step_aim(&draggy);
        let input = round();
        let speed = |aim: &Aim| last_action(&fly(aim, 5, &input, &mut open_world, &[]).0, ACTION_STATE).unwrap().f[0];
        assert!(speed(&other) < speed(&reference) - 1.0, "{} {}", speed(&other), speed(&reference));
    }

    #[test]
    fn what_the_campaign_answers_about_the_ground_decides_how_the_weapon_ends() {
        let mut figures = flying_gun("test gun");
        let impact = |class: f32| {
            move |lookup: &flight::StepEvent| match lookup.kind {
                LOOKUP_GROUND_PROBE => [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                LOOKUP_GROUND_IMPACT => [class, 1.0, 2.0, 3.0, 0.0, 0.0],
                _ => [0.0; 6],
            }
        };
        let aim = step_aim(&figures);
        let (land, _) = fly(&aim, 5, &round(), &mut impact(3.0), &[]);
        assert_eq!(last_action(&land, ACTION_STATE).unwrap().c, 3, "land: hit-land");
        assert_eq!(&last_action(&land, ACTION_POSITION).unwrap().f[..3], &[1.0, 2.0, 3.0], "at the impact point");
        let (water, _) = fly(&aim, 5, &round(), &mut impact(1.0), &[]);
        assert_eq!(last_action(&water, ACTION_STATE).unwrap().c, 4, "water: hit-water");
        // a weapon that survives ground impact rests on it, aligned with the terrain
        figures.survive_ground_impact = 1;
        let aim = step_aim(&figures);
        let mut sloped = |lookup: &flight::StepEvent| match lookup.kind {
            LOOKUP_GROUND_PROBE => [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            LOOKUP_GROUND_IMPACT => [3.0, 1.0, 2.0, 3.0, 0.0, 0.0],
            LOOKUP_FACE_NORMAL => [0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
            _ => [0.0; 6],
        };
        let (rests, asked) = fly(&aim, 5, &round(), &mut sloped, &[]);
        assert!(asked.contains(&LOOKUP_FACE_NORMAL));
        let state = last_action(&rests, ACTION_STATE).unwrap();
        assert_eq!((state.b, state.c), (1, 0), "on the ground, still alive");
        let attitude = last_action(&rests, ACTION_ATTITUDE).unwrap();
        assert_eq!(&attitude.f[3..6], &[0.0, 1.0, 0.0], "the weapon's up is the terrain's normal");
    }

    #[test]
    fn a_contact_with_something_switches_the_weapon_to_it_and_ends_it() {
        let mut figures = flying_gun("test gun");
        figures.in_flight_collision_check = 1;
        let aim = step_aim(&figures);
        let segment = std::cell::Cell::new([0.0f32; 6]);
        let mut world = |lookup: &flight::StepEvent| match lookup.kind {
            LOOKUP_COLLISION => {
                segment.set(lookup.f[..6].try_into().unwrap());
                [1.0, 1.0, 5.0, 5.0, 6.0, 7.0]
            }
            _ => [0.0; 6],
        };
        let (outcome, asked) = fly(&aim, 5, &round(), &mut world, &[]);
        assert!(asked.contains(&LOOKUP_COLLISION));
        assert_eq!(last_action(&outcome, ACTION_STATE).unwrap().c, 5, "hit-target");
        let link = last_action(&outcome, ACTION_TARGET_LINK).unwrap();
        assert_eq!(link.a, 2, "the weapon's target is now what it hit");
        assert_eq!(
            (link.b, &link.f[..6]),
            (-1, &segment.get()[..]),
            "and the action names the collision question that found it, so a host that asks several may choose among them"
        );
        assert_eq!(
            &last_action(&outcome, ACTION_POSITION).unwrap().f[..3],
            &[5.0, 6.0, 7.0],
            "where the segment met it"
        );
        // a collision lookup that finds nothing leaves the round flying
        let (clear, _) = fly(&aim, 5, &round(), &mut open_world, &[]);
        assert_eq!(last_action(&clear, ACTION_STATE).unwrap().c, 0);
        assert!(last_action(&clear, ACTION_TARGET_LINK).is_none());
    }

    #[test]
    fn a_weapon_that_leaves_the_map_is_put_on_its_edge_and_ends_without_asking_about_the_ground() {
        let aim = step_aim(&flying_gun("test gun"));
        let mut input = round();
        input.map_max[2] = 50.0;
        let (outcome, asked) = fly(&aim, 5, &input, &mut open_world, &[]);
        assert!(!asked.contains(&LOOKUP_GROUND_PROBE), "{asked:?}");
        assert_eq!(last_action(&outcome, ACTION_STATE).unwrap().c, 2, "out of bounds");
        assert_eq!(last_action(&outcome, ACTION_POSITION).unwrap().f[2], 50.0, "on the map's edge");
    }

    #[test]
    fn a_weapon_whose_lifetime_has_run_out_is_over_before_it_moves() {
        let aim = step_aim(&flying_gun("test gun"));
        let mut input = round();
        input.weapon_lifetime = -4.0;
        let outcome = aim.weapon_step(5, &input, &[], &[]).unwrap();
        assert_eq!(outcome.state, flight::StepState::Done, "it asked the campaign nothing");
        assert_eq!(last_action(&outcome, ACTION_STATE).unwrap().c, 1, "exhausted");
        assert!(last_action(&outcome, ACTION_POSITION).is_none());
    }

    #[test]
    fn a_guided_weapon_asks_for_the_target_and_its_spiral_takes_the_campaigns_random_samples_in_order() {
        let mut figures = flying_gun("test missile");
        figures.guidance_type = 1;
        figures.flight_profile_or_self_destr = 2;
        figures.cruise_velocity = 400.0;
        figures.muzzle_velocity = 300.0;
        figures.g_max = 30.0;
        figures.boost_time = 3.0;
        figures.sustain_time = 6.0;
        figures.inhibit_time = 2.0;
        figures.cruise_time = 20.0;
        figures.max_seeker_limit = -1.0;
        figures.boost_power = 5000.0;
        figures.sustain_power = 2000.0;
        figures.start_weight = 20.0;
        figures.final_weight = 15.0;
        figures.diameter = 0.1;
        figures.flightpath_deviation = 0.0;
        let aim = step_aim(&figures);
        let mut input = round();
        input.velocity = 300.0;
        input.weapon_lifetime = 5.0;
        input.has_target = true;
        let mut world = |lookup: &flight::StepEvent| match (lookup.kind, lookup.b) {
            (LOOKUP_TARGET_INT, 0) => [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            (LOOKUP_TARGET_VEC, 1) => [0.0, 500.0, 4000.0, 0.0, 0.0, 0.0],
            (LOOKUP_TARGET_VEC, 2) => [0.0, 0.0, 50.0, 0.0, 0.0, 0.0],
            (LOOKUP_TARGET_VEC, 0) => [0.0, 500.0, 4000.0, 0.0, 0.0, 0.0],
            _ => [0.0; 6],
        };
        // without samples it stops for one, then another
        assert_eq!(aim.weapon_step(5, &input, &[], &[]).unwrap().state, flight::StepState::NeedLookup);
        let (a, _) = fly(&aim, 5, &input, &mut world, &[16000, 8000]);
        let (b, _) = fly(&aim, 5, &input, &mut world, &[16000, 8000]);
        let (c, _) = fly(&aim, 5, &input, &mut world, &[1000, 30000]);
        let position = |outcome: &flight::StepOutcome| last_action(outcome, ACTION_POSITION).unwrap().f.map(f32::to_bits);
        assert_eq!(position(&a), position(&b), "the same samples give the same flight");
        assert_ne!(
            position(&a),
            position(&c),
            "other samples give another: the spiral is the campaign's random stream, taken in the host's order"
        );
        // given too few samples the step asks for more
        let mut answers = Vec::new();
        loop {
            let outcome = aim.weapon_step(5, &input, &answers, &[16000]).unwrap();
            match outcome.state {
                flight::StepState::NeedLookup => {
                    for lookup in outcome.lookups {
                        let mut answer = lookup;
                        answer.f[6..].copy_from_slice(&world(&lookup));
                        answers.push(answer);
                    }
                }
                flight::StepState::NeedDraw => break,
                flight::StepState::Done => panic!("the spiral needs two samples"),
            }
        }
    }

    #[test]
    fn a_semi_active_weapon_with_no_launcher_loses_its_target_and_flies_unguided() {
        let mut figures = flying_gun("test sarh");
        figures.guidance_type = 2;
        figures.cruise_velocity = 400.0;
        figures.cruise_time = 20.0;
        figures.max_seeker_limit = -1.0;
        let aim = step_aim(&figures);
        let mut input = round();
        input.has_target = true;
        let (outcome, _) = fly(&aim, 5, &input, &mut open_world, &[]);
        assert_eq!(
            last_action(&outcome, ACTION_TARGET_LINK).unwrap().a,
            0,
            "no guidance source: the target is dropped"
        );
    }

    #[test]
    fn a_step_the_host_cannot_run_is_refused_with_the_reason() {
        let aim = step_aim(&flying_gun("test gun"));
        assert!(Aim::new().weapon_step(5, &round(), &[], &[]).unwrap_err().contains("no figures"));
        assert!(aim.weapon_step(6, &round(), &[], &[]).unwrap_err().contains("no figures"));
        let mut input = round();
        input.position[1] = f32::NAN;
        assert!(aim.weapon_step(5, &input, &[], &[]).unwrap_err().contains("finite"));
        input = round();
        input.entity_samples[2] = 40000;
        assert!(aim.weapon_step(5, &input, &[], &[]).unwrap_err().contains("[0, 32767]"));
        assert!(aim.weapon_step(5, &round(), &[], &[0; 9]).unwrap_err().contains("draws"));
        assert!(aim
            .weapon_step(5, &round(), &[flight::StepEvent::default(); 129], &[])
            .unwrap_err()
            .contains("lookup answers"));
        // a weapon that asks for its launcher when it has none is a failure, not a default
        let mut laser = flying_gun("test laser");
        laser.guidance_type = 4;
        laser.cruise_velocity = 400.0;
        laser.cruise_time = 20.0;
        let mut aim = step_aim(&laser);
        let mut input = round();
        input.has_target = true;
        let mut answers = Vec::new();
        let error = loop {
            match aim.weapon_step(5, &input, &answers, &[]) {
                Err(error) => break error,
                Ok(outcome) => {
                    assert_eq!(outcome.state, flight::StepState::NeedLookup);
                    for lookup in outcome.lookups {
                        let mut answer = lookup;
                        answer.f[6..].copy_from_slice(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
                        answers.push(answer);
                    }
                }
            }
        };
        assert!(error.contains("launcher"), "{error}");
        assert!(aim.set_figures(0, &gun("x")).unwrap_err().contains("not a weapon"));
    }

    // ---------------------------------------------------------------------------------------------------------------------------------------------------------
    // the weapon's birth and release (EEDCS-45)

    const LOOKUP_PARENT_VELOCITY: i32 = 14;
    const LOOKUP_FUZE_SOLVE: i32 = 15;
    const ACTION_BIRTH: i32 = 8;

    /// Asks the host for one birth until it is done, answering from `world` and giving it `draws` in order; returns what it left and the kinds it asked, in order.
    fn be_born(aim: &Aim, weapon: i32, input: &flight::StepInput, world: Answerer, draws: &[i32]) -> (flight::StepOutcome, Vec<i32>) {
        let mut answers: Vec<flight::StepEvent> = Vec::new();
        let mut given: Vec<i32> = Vec::new();
        let mut asked = Vec::new();
        for _ in 0..64 {
            let outcome = aim.weapon_birth(weapon, input, &answers, &given).unwrap();
            match outcome.state {
                flight::StepState::Done => return (outcome, asked),
                flight::StepState::NeedDraw => given.push(draws[given.len()]),
                flight::StepState::NeedLookup => {
                    assert!(outcome.actions.is_empty(), "a birth that is not done leaves no action");
                    for lookup in outcome.lookups {
                        asked.push(lookup.kind);
                        let mut answer = lookup;
                        answer.f[6..].copy_from_slice(&world(&lookup));
                        answers.push(answer);
                    }
                }
            }
        }
        panic!("the birth did not finish");
    }

    /// A gun that takes its launcher's velocity, with a muzzle-velocity error and no dispersion, as a launcher lets it go.
    fn muzzle_gun(name: &str) -> flight::Figures {
        let mut figures = flying_gun(name);
        figures.muzzle_velocity = 900.0;
        figures.muzzle_velocity_max_error = 10.0;
        figures.acquire_forward = 1;
        figures
    }

    #[test]
    fn a_weapon_leaves_its_launcher_with_the_launchers_velocity_the_muzzle_velocity_and_the_error_of_its_own_stream() {
        let aim = step_aim(&muzzle_gun("test gun"));
        let mut input = round();
        input.entity_samples = [16384, 0, 0, 0];
        let mut world = |lookup: &flight::StepEvent| match lookup.kind {
            LOOKUP_PARENT_VELOCITY => [100.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            _ => [0.0; 6],
        };
        let (born, asked) = be_born(&aim, 5, &input, &mut world, &[]);
        assert_eq!(asked, vec![LOOKUP_PARENT_VELOCITY], "only the launcher's velocity is asked");
        let birth = last_action(&born, ACTION_BIRTH).unwrap();
        assert_eq!(birth.f[0], 1000.0, "the launcher's 100 plus the muzzle's 900; a sample of 16384 is no error");
        assert_eq!(birth.f[1], 3.0, "the lifetime is the motor's boost and sustain");
        assert_eq!(birth.f[2], 0.0, "no decoy timer");
        assert_eq!(
            &last_action(&born, ACTION_ATTITUDE).unwrap().f[..9],
            &IDENTITY,
            "without dispersion it leaves as the launcher pointed it, bit for bit"
        );
        // the muzzle-velocity error is the weapon's own sample, scaled
        input.entity_samples[0] = 32767;
        let (born, _) = be_born(&aim, 5, &input, &mut world, &[]);
        let high = last_action(&born, ACTION_BIRTH).unwrap().f[0];
        assert!(high > 1000.0 && high < 1005.0, "{high}");
        // a weapon that does not take its launcher's velocity asks nothing
        let mut standing = muzzle_gun("test gun");
        standing.acquire_forward = 0;
        let aim = step_aim(&standing);
        let (born, asked) = be_born(&aim, 5, &input, &mut world, &[]);
        assert!(asked.is_empty());
        assert!(last_action(&born, ACTION_BIRTH).unwrap().f[0] > 900.0);
        // a submunition leaves with its parent weapon's velocity
        input.submunition = true;
        input.velocity = 250.0;
        let (born, asked) = be_born(&step_aim(&muzzle_gun("test gun")), 5, &input, &mut world, &[]);
        assert!(asked.is_empty(), "{asked:?}");
        assert!(last_action(&born, ACTION_BIRTH).unwrap().f[0] > 1150.0);
    }

    #[test]
    fn dispersion_rotates_the_launchers_attitude_by_two_samples_of_the_campaigns_stream_in_order() {
        let mut figures = muzzle_gun("test gun");
        figures.circular_error_probable = 0.002;
        let aim = step_aim(&figures);
        let input = round();
        let mut world = |_: &flight::StepEvent| [0.0; 6];
        // without samples it stops for them
        assert_eq!(aim.weapon_birth(5, &input, &[], &[]).unwrap().state, flight::StepState::NeedLookup);
        let (a, _) = be_born(&aim, 5, &input, &mut world, &[24576, 8192]);
        let (b, _) = be_born(&aim, 5, &input, &mut world, &[24576, 8192]);
        let (swapped, _) = be_born(&aim, 5, &input, &mut world, &[8192, 24576]);
        let attitude = |born: &flight::StepOutcome| last_action(born, ACTION_ATTITUDE).unwrap().f.map(f32::to_bits);
        assert_eq!(attitude(&a), attitude(&b), "the same samples give the same attitude");
        assert_ne!(attitude(&a), attitude(&swapped), "the first sample is the heading's and the second the pitch's");
        assert_ne!(
            &last_action(&a, ACTION_ATTITUDE).unwrap().f[..9],
            &IDENTITY,
            "and the weapon no longer leaves as it was pointed"
        );
        // a sample in the middle of the stream is no dispersion
        let (centred, _) = be_born(&aim, 5, &input, &mut world, &[16384, 16384]);
        assert_eq!(&last_action(&centred, ACTION_ATTITUDE).unwrap().f[..9], &IDENTITY);
    }

    #[test]
    fn a_shell_with_a_remote_fuze_is_set_from_the_lead_prediction_the_campaign_answers() {
        let mut shell = muzzle_gun("test shell");
        shell.warhead_type = 7;
        shell.cruise_time_max_error = 0.0;
        let aim = step_aim(&shell);
        let input = round();
        // the launcher has no target, or the prediction does not solve: no fuze timer
        let mut unsolved = |_: &flight::StepEvent| [0.0; 6];
        let (born, asked) = be_born(&aim, 5, &input, &mut unsolved, &[]);
        assert!(asked.contains(&LOOKUP_FUZE_SOLVE));
        assert_eq!(last_action(&born, ACTION_BIRTH).unwrap().f[2], 0.0);
        // solved: the timer is how long the shell flies less the time of flight
        let mut solved = |lookup: &flight::StepEvent| match lookup.kind {
            LOOKUP_FUZE_SOLVE => [1.0, 5.0, 0.0, 0.0, 0.0, 0.0],
            _ => [0.0; 6],
        };
        let (born, _) = be_born(&aim, 5, &input, &mut solved, &[16384]);
        assert_eq!(
            last_action(&born, ACTION_BIRTH).unwrap().f[2],
            2.0,
            "(5 - 3) with no cruise-time error; one sample of the stream is taken"
        );
        // conventional munitions burst a second early
        shell.warhead_type = 8;
        let (born, _) = be_born(&step_aim(&shell), 5, &input, &mut solved, &[16384]);
        assert_eq!(last_action(&born, ACTION_BIRTH).unwrap().f[2], 1.0);
        // a guided weapon has no remote fuze to set
        shell.guidance_type = 1;
        let (_, asked) = be_born(&step_aim(&shell), 5, &input, &mut solved, &[]);
        assert!(!asked.contains(&LOOKUP_FUZE_SOLVE));
        assert!(Aim::new().weapon_birth(5, &input, &[], &[]).unwrap_err().contains("no figures"));
    }

    fn rate_limited() -> flight::Figures {
        let mut figures = flying_gun("test cannon");
        figures.rate_of_fire = 600.0;
        figures.inverse_rate_of_fire = 1.0 / 600.0;
        figures
    }

    fn gate(aim: &Aim, input: &flight::ReleaseInput) -> flight::ReleaseOutcome {
        aim.release(5, input).unwrap()
    }

    #[test]
    fn the_rate_of_fire_gate_lets_rounds_go_by_the_time_since_the_last_shot_and_hands_back_what_it_wrote() {
        let aim = step_aim(&rate_limited());
        let base = flight::ReleaseInput {
            delta_time: 0.1,
            current_time: 1000,
            last_shot: Some(0),
            not_ai: false,
        };
        // the first shot always goes, and starts the clock
        assert_eq!(
            gate(&aim, &base),
            flight::ReleaseOutcome {
                refuse: false,
                burst_size: 1,
                last_shot: Some(1000)
            }
        );
        // not enough time has passed for a round: nothing goes, nothing is written
        let early = gate(
            &aim,
            &flight::ReleaseInput {
                current_time: 1050,
                last_shot: Some(1000),
                ..base
            },
        );
        assert_eq!((early.refuse, early.last_shot), (true, None));
        // the clock is ahead of the time (the launcher shot in the future): nothing goes
        assert!(
            gate(
                &aim,
                &flight::ReleaseInput {
                    current_time: 900,
                    last_shot: Some(1000),
                    ..base
                }
            )
            .refuse
        );
        // three rounds' time has passed: three go and the clock advances by their time
        assert_eq!(
            gate(
                &aim,
                &flight::ReleaseInput {
                    current_time: 1300,
                    last_shot: Some(1000),
                    ..base
                }
            ),
            flight::ReleaseOutcome {
                refuse: false,
                burst_size: 3,
                last_shot: Some(1300)
            }
        );
        // a clock long behind: a launcher that is not an AI is caught up to the frame, an AI's rounds are owed in full
        let player = gate(
            &aim,
            &flight::ReleaseInput {
                current_time: 2000,
                last_shot: Some(1000),
                not_ai: true,
                ..base
            },
        );
        assert_eq!((player.burst_size, player.last_shot), (1, Some(2100)), "{player:?}");
        let ai = gate(
            &aim,
            &flight::ReleaseInput {
                current_time: 2000,
                last_shot: Some(1000),
                not_ai: false,
                ..base
            },
        );
        assert_eq!((ai.burst_size, ai.last_shot), (10, Some(2000)), "{ai:?}");
        // the catch-up is written even when the gate then refuses
        let refused = gate(
            &aim,
            &flight::ReleaseInput {
                delta_time: 0.001,
                current_time: 2000,
                last_shot: Some(1000),
                not_ai: true,
            },
        );
        assert_eq!((refused.refuse, refused.last_shot), (true, Some(2000)), "{refused:?}");
        // a launcher with no clock lets one round go
        let clockless = gate(&aim, &flight::ReleaseInput { last_shot: None, ..base });
        assert_eq!((clockless.refuse, clockless.burst_size, clockless.last_shot), (false, 1, None));
        // a single-shot weapon is not gated
        let single = step_aim(&flying_gun("test gun"));
        assert_eq!(gate(&single, &base).burst_size, 1);
        assert_eq!(gate(&single, &base).last_shot, None);
        assert!(Aim::new().release(5, &base).unwrap_err().contains("no figures"));
        assert!(aim
            .release(5, &flight::ReleaseInput { delta_time: f32::NAN, ..base })
            .unwrap_err()
            .contains("finite"));
    }

    #[test]
    fn a_weapon_that_did_not_move_asks_for_the_ground_under_its_point_once() {
        // a precise weapon (its launcher is not an AI) tests the ground at the new position and at nine points between the old and the new; when it did not move they
        // are one point, asked once, and the step finishes
        let aim = step_aim(&flying_gun("test gun"));
        let mut input = round();
        input.velocity = 0.0;
        // coasting (no motor, no drag at rest): it does not move at all
        input.weapon_lifetime = -1.0;
        input.has_launcher = true;
        let world = |lookup: &flight::StepEvent| match lookup.kind {
            LOOKUP_LAUNCHER_INT => [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            _ => [0.0; 6],
        };
        let mut answers: Vec<flight::StepEvent> = Vec::new();
        let mut ground_batches = Vec::new();
        for _ in 0..16 {
            let outcome = aim.weapon_step(5, &input, &answers, &[]).unwrap();
            match outcome.state {
                flight::StepState::Done => {
                    assert_eq!(ground_batches, vec![1], "the ground is asked about once");
                    assert_eq!(last_action(&outcome, ACTION_STATE).unwrap().c, 0);
                    return;
                }
                flight::StepState::NeedDraw => panic!("this flight takes no sample"),
                flight::StepState::NeedLookup => {
                    let mut seen: Vec<(i32, i32, i32, [u32; 12])> = Vec::new();
                    for lookup in &outcome.lookups {
                        let key = (lookup.kind, lookup.a, lookup.b, lookup.f.map(f32::to_bits));
                        assert!(!seen.contains(&key), "the same question twice in one batch: {lookup:?}");
                        seen.push(key);
                    }
                    let probes = outcome.lookups.iter().filter(|lookup| lookup.kind == LOOKUP_GROUND_PROBE).count();
                    if probes > 0 {
                        ground_batches.push(probes);
                    }
                    for lookup in outcome.lookups {
                        let mut answer = lookup;
                        answer.f[6..].copy_from_slice(&world(&lookup));
                        answers.push(answer);
                    }
                }
            }
        }
        panic!("the step did not finish");
    }

    // ---------------------------------------------------------------------------------------------------------------------------------------------------------
    // after the flight (EEDCS-45): the parachute, the decoy and the seeker's seduction

    const ACTION_DECOY: i32 = 9;
    const ACTION_SEDUCE: i32 = 10;

    fn parachute(kind: i32) -> flight::Figures {
        let mut figures = flying_gun("test crate");
        figures.parachute_type = kind;
        figures
    }

    fn fall(status: i32, lifetime: f32, motion_y: f32, on_ground: bool) -> flight::AnimateInput {
        flight::AnimateInput {
            parachute_status: status,
            weapon_on_ground: on_ground,
            weapon_lifetime: lifetime,
            motion_vector_y: motion_y,
        }
    }

    #[test]
    fn a_falling_crate_opens_its_parachute_in_phases_and_each_phase_resets_its_lifetime() {
        let mut aim = Aim::new();
        aim.set_figures(5, &parachute(2)).unwrap();
        let step = |aim: &Aim, input| aim.animate(5, &input).unwrap();
        // closed: it opens when its lifetime is past half a second negative and it is falling
        assert_eq!(
            step(&aim, fall(0, -0.4, -3.0, false)),
            flight::AnimateOutcome {
                parachute_status: 0,
                weapon_lifetime: None
            }
        );
        assert_eq!(
            step(&aim, fall(0, -0.6, 3.0, false)),
            flight::AnimateOutcome {
                parachute_status: 0,
                weapon_lifetime: None
            },
            "rising, it stays shut"
        );
        assert_eq!(
            step(&aim, fall(0, -0.6, -3.0, false)),
            flight::AnimateOutcome {
                parachute_status: 1,
                weapon_lifetime: Some(0.0)
            }
        );
        assert_eq!(
            step(&aim, fall(1, -0.7, -3.0, false)),
            flight::AnimateOutcome {
                parachute_status: 2,
                weapon_lifetime: Some(0.0)
            }
        );
        assert_eq!(step(&aim, fall(1, -0.5, -3.0, false)).parachute_status, 1, "not yet");
        assert_eq!(
            step(&aim, fall(2, -0.8, -3.0, false)),
            flight::AnimateOutcome {
                parachute_status: 3,
                weapon_lifetime: Some(0.0)
            }
        );
        assert_eq!(
            step(&aim, fall(3, -9.0, -3.0, false)),
            flight::AnimateOutcome {
                parachute_status: 3,
                weapon_lifetime: None
            },
            "open stays open"
        );
        // on the ground it collapses, whatever phase it was in, and nothing else is written
        assert_eq!(
            step(&aim, fall(3, -9.0, 0.0, true)),
            flight::AnimateOutcome {
                parachute_status: 4,
                weapon_lifetime: None
            }
        );
        assert_eq!(
            step(&aim, fall(0, -0.6, -3.0, true)),
            flight::AnimateOutcome {
                parachute_status: 4,
                weapon_lifetime: None
            }
        );
    }

    #[test]
    fn an_ejector_seat_opens_by_its_own_lifetimes_and_never_resets_them_and_a_weapon_with_no_parachute_has_nothing_to_animate() {
        let mut aim = Aim::new();
        aim.set_figures(5, &parachute(1)).unwrap();
        aim.set_figures(6, &parachute(0)).unwrap();
        let step = |aim: &Aim, weapon, input| aim.animate(weapon, &input).unwrap();
        assert_eq!(
            step(&aim, 5, fall(0, -1.5, -3.0, false)),
            flight::AnimateOutcome {
                parachute_status: 1,
                weapon_lifetime: None
            }
        );
        assert_eq!(step(&aim, 5, fall(0, -0.9, -3.0, false)).parachute_status, 0);
        assert_eq!(step(&aim, 5, fall(1, -1.2, -3.0, false)).parachute_status, 2);
        assert_eq!(
            step(&aim, 5, fall(2, 1.0, -3.0, false)).parachute_status,
            3,
            "the original's third phase tests a positive lifetime"
        );
        assert_eq!(step(&aim, 5, fall(2, 2.0, -3.0, false)).parachute_status, 2);
        assert_eq!(
            step(&aim, 5, fall(0, 0.0, 0.0, true)).parachute_status,
            0,
            "a seat does not collapse on the ground"
        );
        assert_eq!(
            step(&aim, 6, fall(0, -9.0, -3.0, true)),
            flight::AnimateOutcome {
                parachute_status: 0,
                weapon_lifetime: None
            }
        );
        // a state that is not a parachute's, a number that is not finite, and a weapon the host has no figures for are refused
        assert!(aim.animate(5, &fall(7, 0.0, 0.0, false)).unwrap_err().contains("parachute state"));
        assert!(aim.animate(5, &fall(0, f32::NAN, 0.0, false)).unwrap_err().contains("finite"));
        assert!(aim.animate(9, &fall(0, 0.0, 0.0, false)).unwrap_err().contains("no figures"));
        // a seat in the collapsed state, which only a crate has, is a state it cannot continue from
        assert!(aim.animate(5, &fall(4, 0.0, 0.0, false)).unwrap_err().contains("cannot continue"));
    }

    fn decoy_figures(decoy_type: i32, guidance_type: i32) -> flight::Figures {
        let mut figures = flying_gun("test decoy");
        figures.decoy_type = decoy_type;
        figures.guidance_type = guidance_type;
        figures
    }

    fn decoy_input(timer: f32) -> flight::StepInput {
        let mut input = round();
        input.decoy_timer = timer;
        input.delta_time = 0.1;
        input
    }

    #[test]
    fn a_decoys_timer_runs_down_and_restarts_at_a_time_drawn_from_the_campaigns_stream_when_it_runs_out() {
        let mut aim = Aim::new();
        aim.set_figures(5, &decoy_figures(1, 0)).unwrap();
        aim.set_figures(6, &decoy_figures(3, 0)).unwrap();
        aim.set_figures(7, &flying_gun("test gun")).unwrap();
        // while the timer has time left it runs down and takes no sample
        let running = aim.weapon_decoy(5, &decoy_input(0.5), &[], &[]).unwrap();
        assert_eq!(running.state, flight::StepState::Done);
        let action = running.actions[0];
        assert_eq!((action.kind, action.a, action.f[0].to_bits()), (ACTION_DECOY, 0, (0.5f32 - 0.1f32).to_bits()));
        // when it runs out the host asks for a sample, and the restart is the original's: 0.4 + 0.2 of the sample (chaff and flare), 1.0 + 0.5 of it (smoke)
        let asks = aim.weapon_decoy(5, &decoy_input(0.05), &[], &[]).unwrap();
        assert_eq!((asks.state, asks.actions.len()), (flight::StepState::NeedDraw, 0));
        let chaff = aim.weapon_decoy(5, &decoy_input(0.05), &[], &[32767]).unwrap();
        let frand1 = 32767.0f64 * (1.0f64 / 32767.0f64);
        assert_eq!(
            (chaff.actions[0].a, chaff.actions[0].f[0].to_bits()),
            (1, ((0.4f64 + (frand1 * 0.2f64)) as f32).to_bits())
        );
        let smoke = aim.weapon_decoy(6, &decoy_input(0.0), &[], &[0]).unwrap();
        assert_eq!(
            (smoke.actions[0].a, smoke.actions[0].f[0]),
            (1, 1.0),
            "a timer of exactly zero is not yet out; ran down by the frame it is"
        );
        // a weapon that is no decoy has no timer to run, and one the host has no figures for cannot be asked
        assert!(aim.weapon_decoy(7, &decoy_input(0.5), &[], &[]).unwrap_err().contains("no decoy"));
        assert!(aim.weapon_decoy(9, &decoy_input(0.5), &[], &[]).unwrap_err().contains("no figures"));
    }

    fn seduction(coefficient: f32) -> flight::StepInput {
        let mut input = round();
        input.coefficient = coefficient;
        input.effectiveness = [0.5, 0.5, 1.0];
        input
    }

    #[test]
    fn a_seeker_goes_for_a_decoy_only_when_the_sample_is_under_the_decoys_chance_and_the_decoy_is_of_the_seekers_kind() {
        let mut aim = Aim::new();
        aim.set_figures(10, &decoy_figures(0, 2)).unwrap(); // a semi-active radar seeker
        aim.set_figures(11, &decoy_figures(0, 1)).unwrap(); // an infra-red seeker
        aim.set_figures(12, &decoy_figures(0, 4)).unwrap(); // a laser seeker
        aim.set_figures(20, &decoy_figures(1, 0)).unwrap(); // chaff
        aim.set_figures(21, &decoy_figures(2, 0)).unwrap(); // a flare
        aim.set_figures(22, &decoy_figures(3, 0)).unwrap(); // a smoke grenade
        aim.set_figures(23, &flying_gun("test gun")).unwrap(); // no decoy
        let go = |seeker, decoy, coefficient, draws: &[i32]| {
            let outcome = aim.weapon_seduce(seeker, decoy, &seduction(coefficient), &[], draws).unwrap();
            assert_eq!(outcome.state, flight::StepState::Done, "{seeker} {decoy} {draws:?}");
            let action = outcome.actions[0];
            assert_eq!(action.kind, ACTION_SEDUCE);
            (action.a, action.b)
        };
        // the chance of chaff is 0.2 ** coefficient * 0.5: 0.1 at a coefficient of 1; a sample of 1000 (0.03) is under it, 20000 (0.61) over
        assert_eq!(go(10, 20, 1.0, &[1000]), (1, 1), "drawn to chaff, a radar seeker goes for it");
        assert_eq!(go(10, 20, 1.0, &[20000]), (0, 0), "not drawn");
        assert_eq!(go(10, 21, 1.0, &[1000]), (1, 0), "drawn to a flare, a radar seeker does not go for it");
        assert_eq!(go(11, 21, 1.0, &[1000]), (1, 1), "an infra-red seeker goes for a flare");
        assert_eq!(go(11, 20, 1.0, &[1000]), (1, 0));
        assert_eq!(go(12, 22, 1.0, &[1000]), (1, 1), "a laser seeker goes for smoke");
        // a smaller coefficient (an easy game) gives a decoy a larger chance: 0.2 ** 0.1 is 0.85, so chaff has 0.85 * 0.5 = 0.43 (a sample of 20000, 0.61, is over it) and smoke has 0.85 (under it)
        assert_eq!(go(10, 20, 0.1, &[20000]), (0, 0));
        assert_eq!(go(12, 22, 0.1, &[20000]), (1, 1));
        // a weapon that is no decoy never draws anyone, and takes no sample
        let none = aim.weapon_seduce(10, 23, &seduction(1.0), &[], &[]).unwrap();
        assert_eq!((none.state, none.actions[0].a, none.actions[0].b), (flight::StepState::Done, 0, 0));
        // the sample is taken whether or not the seeker could be seduced
        assert_eq!(aim.weapon_seduce(10, 21, &seduction(1.0), &[], &[]).unwrap().state, flight::StepState::NeedDraw);
        assert!(aim.weapon_seduce(10, 99, &seduction(1.0), &[], &[]).unwrap_err().contains("no figures"));
        assert!(aim.weapon_seduce(99, 20, &seduction(1.0), &[], &[1]).unwrap_err().contains("no figures"));
        let mut infinite = seduction(1.0);
        infinite.effectiveness[1] = f32::INFINITY;
        assert!(aim.weapon_seduce(10, 20, &infinite, &[], &[1]).unwrap_err().contains("finite"));
    }
}
