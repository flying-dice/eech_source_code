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
