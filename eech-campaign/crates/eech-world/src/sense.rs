//! The host's physical sensing kernel (EEDCS-43, D2): what a source's sensor perceives of a candidate target, kept in C (`csrc/sense_step.c`,
//! EECH's own arithmetic expression for expression) and driven here one message at a time. The campaign opens a scan with `begin`, asks about each
//! candidate it enumerated with `candidate` (and, when the host asks for them, `draw` and `signature`), and closes it with `end`. The host keeps
//! the physical state of the open scan between messages: the surface-to-air floor lowered by every candidate asked about.
//!
//! Which targets are candidates, in which order they are asked, the random generator behind the draw, the ranking and the engagement are not here.
//! The unit database figures, the positions, the radar altitude and the helicopter configuration facts are the campaign's mirror of the world,
//! supplied as facts; that is declared transitional coupling.

use std::ffi::{c_int, c_void};

pub const SCAN_AIR_TO_AIR: i32 = 1;
pub const SCAN_AIR_TO_SURFACE: i32 = 2;
pub const SCAN_SURFACE_TO_AIR: i32 = 3;
pub const SCAN_SURFACE_TO_SURFACE: i32 = 4;

const STATE_OPEN: c_int = 1;
const STATE_ACCEPT: c_int = 2;
const STATE_REJECT: c_int = 3;
const STATE_NEED_DRAW: c_int = 4;
const STATE_NEED_SIGNATURE: c_int = 5;
const STATE_CLOSED: c_int = 6;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RawAnswer {
    state: c_int,
    reason: c_int,
    has_range: c_int,
    target_range: f32,
    has_echo: c_int,
    echo_ceiling: f32,
    echo_floor: f32,
    echo_adjusted_range: f32,
    echo_stealth_factor: f32,
    echo_stealth_limit: f32,
    echo_tolerance_limit: f64,
}

extern "C" {
    fn eech_sense_scan_size() -> usize;
    fn eech_sense_begin(scan: *mut c_void, kind: c_int, source: *const f32, scan_range: f32, scan_ceiling: f32, scan_floor: f32, out: *mut RawAnswer) -> c_int;
    fn eech_sense_candidate(
        scan: *mut c_void,
        kind: c_int,
        target: *const f32,
        has_difficulty: c_int,
        difficulty: f32,
        radar_altitude: f32,
        out: *mut RawAnswer,
    ) -> c_int;
    fn eech_sense_draw(scan: *mut c_void, draw: f64, out: *mut RawAnswer) -> c_int;
    #[allow(clippy::too_many_arguments)]
    fn eech_sense_signature(
        scan: *mut c_void,
        has_emitters: c_int,
        radar_on: c_int,
        radar_jammer_on: c_int,
        infra_red_jammer_on: c_int,
        helicopter: c_int,
        weapon_bay_closed: c_int,
        stub_wings_attached: c_int,
        undercarriage_up: c_int,
        out: *mut RawAnswer,
    ) -> c_int;
    fn eech_sense_end(scan: *mut c_void, out: *mut RawAnswer) -> c_int;
}

/// Why a candidate was rejected: the predicate that failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    Range,
    RangeAdjusted,
    Altitude,
    Tolerance,
}

/// What the host says next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Open,
    Accept,
    Reject(Reason),
    NeedDraw,
    NeedSignature,
    Closed,
}

/// The host's intermediate physical values, reported for the test fixture's trace.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Echo {
    pub ceiling: f32,
    pub floor: f32,
    pub adjusted_range: f32,
    pub stealth_factor: f32,
    pub stealth_limit: f32,
    pub tolerance_limit: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Answer {
    pub state: State,
    pub target_range: Option<f32>,
    pub echo: Option<Echo>,
}

fn answer(raw: &RawAnswer) -> Result<Answer, String> {
    let state = match raw.state {
        STATE_OPEN => State::Open,
        STATE_ACCEPT => State::Accept,
        STATE_REJECT => State::Reject(match raw.reason {
            1 => Reason::Range,
            2 => Reason::RangeAdjusted,
            3 => Reason::Altitude,
            4 => Reason::Tolerance,
            other => return Err(format!("sensing kernel rejected with an unknown reason {other}")),
        }),
        STATE_NEED_DRAW => State::NeedDraw,
        STATE_NEED_SIGNATURE => State::NeedSignature,
        STATE_CLOSED => State::Closed,
        other => return Err(format!("sensing kernel returned an unknown state {other}")),
    };
    Ok(Answer {
        state,
        target_range: (raw.has_range != 0).then_some(raw.target_range),
        echo: (raw.has_echo != 0).then_some(Echo {
            ceiling: raw.echo_ceiling,
            floor: raw.echo_floor,
            adjusted_range: raw.echo_adjusted_range,
            stealth_factor: raw.echo_stealth_factor,
            stealth_limit: raw.echo_stealth_limit,
            tolerance_limit: raw.echo_tolerance_limit,
        }),
    })
}

fn finite(name: &str, values: &[f32]) -> Result<(), String> {
    if values.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(format!("sense {name} must be finite"))
    }
}

/// The one open scan, in the kernel's own state (an opaque, suitably aligned block).
pub struct Sense {
    scan: Vec<u64>,
    kind: i32,
}

impl Default for Sense {
    fn default() -> Self {
        Self::new()
    }
}

impl Sense {
    #[must_use]
    pub fn new() -> Self {
        // SAFETY: a pure size query.
        let bytes = unsafe { eech_sense_scan_size() };
        Self {
            scan: vec![0u64; bytes.div_ceil(8)],
            kind: 0,
        }
    }

    /// Opens a scan of `kind` for a source at `source` with the unit's raw scan range (and, for the surface-to-air scan, the group's raw ceiling and
    /// initial floor). Replaces any open scan.
    pub fn begin(&mut self, kind: i32, source: [f32; 3], scan_range: f32, scan_ceiling: f32, scan_floor: f32) -> Result<Answer, String> {
        finite("source position", &source)?;
        finite("sensor figure", &[scan_range, scan_ceiling, scan_floor])?;
        let mut raw = RawAnswer::default();
        // SAFETY: `scan` is the kernel's block, sized by it; the vector and the result are valid for the call.
        let code = unsafe {
            eech_sense_begin(
                self.scan.as_mut_ptr().cast(),
                kind,
                source.as_ptr(),
                scan_range,
                scan_ceiling,
                scan_floor,
                &mut raw,
            )
        };
        if code != 0 {
            return Err(format!("sense: unknown scan {kind}"));
        }
        self.kind = kind;
        answer(&raw)
    }

    /// One candidate. `difficulty` is present when a human flies the target (surface-to-air only); `radar_altitude` is the surface-to-air scan's.
    pub fn candidate(&mut self, kind: i32, target: [f32; 3], difficulty: Option<f32>, radar_altitude: f32) -> Result<Answer, String> {
        finite("target position", &target)?;
        finite("radar altitude", &[radar_altitude])?;
        if let Some(factor) = difficulty {
            finite("difficulty factor", &[factor])?;
        }
        let mut raw = RawAnswer::default();
        // SAFETY: as in `begin`.
        let code = unsafe {
            eech_sense_candidate(
                self.scan.as_mut_ptr().cast(),
                kind,
                target.as_ptr(),
                c_int::from(difficulty.is_some()),
                difficulty.unwrap_or(1.0),
                radar_altitude,
                &mut raw,
            )
        };
        match code {
            0 => answer(&raw),
            1 => Err("sense: a candidate arrived with no scan open or while one was unfinished".into()),
            _ => Err(format!("sense: a candidate of scan {kind} arrived in a scan of kind {}", self.kind)),
        }
    }

    /// The campaign's random draw, in [0, 1].
    pub fn draw(&mut self, draw: f64) -> Result<Answer, String> {
        if !draw.is_finite() || !(0.0..=1.0).contains(&draw) {
            return Err("sense draw must be a finite number in [0, 1]".into());
        }
        let mut raw = RawAnswer::default();
        // SAFETY: as in `begin`.
        let code = unsafe { eech_sense_draw(self.scan.as_mut_ptr().cast(), draw, &mut raw) };
        if code != 0 {
            return Err("sense: a draw arrived but none was asked for".into());
        }
        answer(&raw)
    }

    /// The emitters of a human-flown target and the configuration of a helicopter target.
    pub fn signature(&mut self, emitters: Option<(bool, bool, bool)>, helicopter: Option<(bool, bool, bool)>) -> Result<Answer, String> {
        let (has_emitters, radar_on, jammer_on, ir_on) = match emitters {
            Some((radar, jammer, ir)) => (1, c_int::from(radar), c_int::from(jammer), c_int::from(ir)),
            None => (0, 1, 1, 1),
        };
        let (is_helicopter, bay_closed, stub_wings, gear_up) = match helicopter {
            Some((bay, stub, gear)) => (1, c_int::from(bay), c_int::from(stub), c_int::from(gear)),
            None => (0, 0, 1, 0),
        };
        let mut raw = RawAnswer::default();
        // SAFETY: as in `begin`.
        let code = unsafe {
            eech_sense_signature(
                self.scan.as_mut_ptr().cast(),
                has_emitters,
                radar_on,
                jammer_on,
                ir_on,
                is_helicopter,
                bay_closed,
                stub_wings,
                gear_up,
                &mut raw,
            )
        };
        match code {
            0 => answer(&raw),
            1 => Err("sense: a signature arrived but none was asked for".into()),
            _ => Err("sense: a helicopter's radar-stealth factor is not positive".into()),
        }
    }

    /// Closes the open scan.
    pub fn end(&mut self) -> Result<Answer, String> {
        let mut raw = RawAnswer::default();
        // SAFETY: as in `begin`.
        let code = unsafe { eech_sense_end(self.scan.as_mut_ptr().cast(), &mut raw) };
        if code != 0 {
            return Err("sense: a scan was ended with none open".into());
        }
        answer(&raw)
    }
}

#[cfg(test)]
mod tests {
    //! Focused contracts of the kernel against scripted candidates: the expectations are derived from EECH's scan functions (ac_tgt.c, vh_tgt.c)
    //! and the radar-stealth factor (hc_float.c) as written, including their quirks, not from this implementation. Bit-for-bit equality with the
    //! original scans is protected separately by the sensor trace of the whole campaign, which these tests do not replace.
    use super::*;

    const SRC: [f32; 3] = [1000.0, 100.0, 2000.0];

    fn accepted(answer: &Answer) -> bool {
        answer.state == State::Accept
    }

    #[test]
    fn air_to_surface_compares_the_planar_squared_range_with_the_squared_scan_range() {
        let mut sense = Sense::new();
        sense.begin(SCAN_AIR_TO_SURFACE, SRC, 4000.0, 0.0, 0.0).unwrap();
        // 4000 m due east is exactly the squared range: not greater, so detected, with the squared range reported
        let on = sense.candidate(SCAN_AIR_TO_SURFACE, [5000.0, 0.0, 2000.0], None, 0.0).unwrap();
        assert!(accepted(&on));
        assert_eq!(on.target_range, Some(16_000_000.0));
        // a hair farther is not; the height difference is ignored (planar)
        let off = sense.candidate(SCAN_AIR_TO_SURFACE, [5000.5, 9000.0, 2000.0], None, 0.0).unwrap();
        assert_eq!(off.state, State::Reject(Reason::Range));
        assert!(off.target_range.unwrap() > 16_000_000.0);
        assert_eq!(sense.end().unwrap().state, State::Closed);
    }

    #[test]
    fn surface_to_surface_uses_the_approximate_three_dimensional_range() {
        let mut sense = Sense::new();
        sense.begin(SCAN_SURFACE_TO_SURFACE, SRC, 5000.0, 0.0, 0.0).unwrap();
        // max + (med + min) / 4: dx 3000, dy 2000, dz 1000 gives 3000 + 750 = 3750, where the true range is 3741.66
        let near = sense.candidate(SCAN_SURFACE_TO_SURFACE, [4000.0, 2100.0, 3000.0], None, 0.0).unwrap();
        assert!(accepted(&near));
        assert_eq!(near.target_range, Some(3750.0));
        // 4800 + (4800 + 0) / 4 = 6000 > 5000
        let far = sense.candidate(SCAN_SURFACE_TO_SURFACE, [5800.0, 100.0, 6800.0], None, 0.0).unwrap();
        assert_eq!(far.state, State::Reject(Reason::Range));
    }

    #[test]
    fn the_surface_to_air_floor_is_lowered_by_every_candidate_asked_about_and_never_restored() {
        let mut sense = Sense::new();
        // range 6000, ceiling 6000 (+100 for the source's altitude), floor 18
        let begun = sense.begin(SCAN_SURFACE_TO_AIR, SRC, 6000.0, 6000.0, 18.0).unwrap();
        assert_eq!(begun.echo.unwrap().ceiling, 6100.0);
        // a candidate at the full range keeps the ratio at 1.0: the floor stays 18, and 15 m is below it
        let first = sense.candidate(SCAN_SURFACE_TO_AIR, [7000.0, 100.0, 2000.0], None, 15.0).unwrap();
        assert_eq!(first.state, State::Reject(Reason::Altitude));
        assert_eq!(first.echo.unwrap().floor, 18.0);
        // a candidate at half the range scales it by 0.5: 9, and 15 m now clears it
        let second = sense.candidate(SCAN_SURFACE_TO_AIR, [4000.0, 100.0, 2000.0], None, 15.0).unwrap();
        assert_eq!(second.state, State::NeedDraw);
        assert_eq!(second.echo.unwrap().floor, 9.0);
        sense.draw(0.0).unwrap();
        sense.signature(None, None).unwrap();
        // the same 15 m at the full range again: the floor was NOT restored (it is 9 and, scaled by 1.0, stays 9), so it now clears the altitude test
        let third = sense.candidate(SCAN_SURFACE_TO_AIR, [7000.0, 100.0, 2000.0], None, 15.0).unwrap();
        assert_eq!(third.echo.unwrap().floor, 9.0);
        assert_eq!(third.state, State::NeedDraw);
    }

    #[test]
    fn a_close_target_cannot_lower_the_floor_below_a_tenth_and_a_player_scales_it_by_the_difficulty() {
        let mut sense = Sense::new();
        sense.begin(SCAN_SURFACE_TO_AIR, SRC, 6000.0, 6000.0, 20.0).unwrap();
        let close = sense.candidate(SCAN_SURFACE_TO_AIR, [1010.0, 100.0, 2000.0], Some(2.0), 1000.0).unwrap();
        // 20 * 0.1 (the bound's lower limit) * 2.0 (easy)
        assert_eq!(close.echo.unwrap().floor, 4.0);
    }

    #[test]
    fn the_altitude_envelope_reaches_the_sources_altitude_above_the_group_ceiling() {
        let mut sense = Sense::new();
        sense.begin(SCAN_SURFACE_TO_AIR, SRC, 6000.0, 500.0, 0.0).unwrap();
        // the ceiling is 500 + 100: a target at 600 is inside, one at 600.5 is above it
        assert_eq!(
            sense.candidate(SCAN_SURFACE_TO_AIR, [2000.0, 100.0, 2000.0], None, 600.0).unwrap().state,
            State::NeedDraw
        );
        sense.draw(0.0).unwrap();
        sense.signature(None, None).unwrap();
        assert_eq!(
            sense.candidate(SCAN_SURFACE_TO_AIR, [2000.0, 100.0, 2000.0], None, 600.5).unwrap().state,
            State::Reject(Reason::Altitude)
        );
    }

    #[test]
    fn the_tolerance_limit_is_floor_times_one_plus_half_the_draw_and_comes_after_the_altitude_test() {
        let mut sense = Sense::new();
        sense.begin(SCAN_SURFACE_TO_AIR, SRC, 6000.0, 6000.0, 20.0).unwrap();
        let asked = sense.candidate(SCAN_SURFACE_TO_AIR, [7000.0, 100.0, 2000.0], None, 25.0).unwrap();
        assert_eq!(asked.state, State::NeedDraw);
        // draw 1.0: limit 20 * 1.5 = 30 > 25, so the target is rejected by the tolerance
        let rejected = sense.draw(1.0).unwrap();
        assert_eq!(rejected.state, State::Reject(Reason::Tolerance));
        assert_eq!(rejected.echo.unwrap().tolerance_limit, 30.0);
        // the same altitude with draw 0.0: limit 20 < 25, passes on to the signature
        sense.candidate(SCAN_SURFACE_TO_AIR, [7000.0, 100.0, 2000.0], None, 25.0).unwrap();
        // (the floor is still 20: the ratio was 1.0)
        assert_eq!(sense.draw(0.0).unwrap().state, State::NeedSignature);
    }

    #[test]
    fn a_helicopters_stealth_factor_follows_its_configuration_and_scales_the_surface_to_air_limit() {
        let run = |bay, stub, gear| {
            let mut sense = Sense::new();
            sense.begin(SCAN_SURFACE_TO_AIR, SRC, 6000.0, 6000.0, 0.0).unwrap();
            assert_eq!(
                sense.candidate(SCAN_SURFACE_TO_AIR, [4000.0, 0.0, 2000.0], None, 50.0).unwrap().state,
                State::NeedDraw
            );
            assert_eq!(sense.draw(0.0).unwrap().state, State::NeedSignature);
            sense.signature(None, Some((bay, stub, gear))).unwrap()
        };
        // a plain helicopter with stub wings fitted and its gear down is 1.0
        assert_eq!(run(false, true, false).echo.unwrap().stealth_factor, 1.0);
        // wings off: 1 - 0.35 narrowed from a double; gear up: 1 - 0.1; bay closed: 1 - 0.4
        assert_eq!(run(false, false, false).echo.unwrap().stealth_factor, (1.0f64 - 0.35) as f32);
        assert_eq!(run(false, true, true).echo.unwrap().stealth_factor, (1.0f64 - 0.1) as f32);
        // all three subtract in turn, each narrowed to float: the fixed-point 0.15 the campaign sees for a stripped Comanche
        let all = run(true, false, true).echo.unwrap();
        let mut expected = 1.0f32;
        expected = (f64::from(expected) - 0.4) as f32;
        expected = (f64::from(expected) - 0.35) as f32;
        expected = (f64::from(expected) - 0.1) as f32;
        assert_eq!(all.stealth_factor, expected);
        assert!((all.stealth_factor - 0.15).abs() < 1e-6);
        // the limit is the raw scan range times the factor
        assert_eq!(all.stealth_limit, 6000.0f32 * all.stealth_factor);
    }

    #[test]
    fn an_aircraft_scan_scales_the_squared_range_and_a_switched_off_emitter_closes_it_by_ten_percent() {
        let mut sense = Sense::new();
        sense.begin(SCAN_AIR_TO_AIR, SRC, 5000.0, 0.0, 0.0).unwrap();
        // exactly the squared range: detected by an AI-flown target (no emitters), with the stealth limit the squared range times 1
        let target = [6000.0, 0.0, 2000.0];
        assert_eq!(sense.candidate(SCAN_AIR_TO_AIR, target, None, 0.0).unwrap().state, State::NeedSignature);
        let ai = sense.signature(None, Some((false, true, false))).unwrap();
        assert!(accepted(&ai));
        assert_eq!(ai.echo.unwrap().stealth_limit, 25_000_000.0);
        // the same target flown by a human with the radar jammer off: 25e6 * 1.1 (a double constant) exceeds the limit and the target is not seen
        sense.candidate(SCAN_AIR_TO_AIR, target, None, 0.0).unwrap();
        let human = sense.signature(Some((true, false, true)), Some((false, true, false))).unwrap();
        assert_eq!(human.state, State::Reject(Reason::RangeAdjusted));
        assert_eq!(human.echo.unwrap().adjusted_range, (25_000_000.0f64 * 1.1) as f32);
    }

    #[test]
    fn messages_that_do_not_fit_the_state_are_refused() {
        let mut sense = Sense::new();
        assert!(sense.candidate(SCAN_AIR_TO_SURFACE, [0.0; 3], None, 0.0).is_err(), "no scan is open");
        assert!(sense.draw(0.5).is_err());
        assert!(sense.signature(None, None).is_err());
        assert!(sense.end().is_err());
        assert!(sense.begin(9, SRC, 1.0, 0.0, 0.0).is_err(), "unknown scan");
        sense.begin(SCAN_SURFACE_TO_AIR, SRC, 6000.0, 6000.0, 18.0).unwrap();
        assert!(sense.draw(0.5).is_err(), "a draw nobody asked for");
        assert!(sense.candidate(SCAN_AIR_TO_AIR, [0.0; 3], None, 0.0).is_err(), "a candidate of another scan");
        assert!(sense.candidate(SCAN_SURFACE_TO_AIR, [f32::NAN, 0.0, 0.0], None, 0.0).is_err());
        assert_eq!(
            sense.candidate(SCAN_SURFACE_TO_AIR, [7000.0, 100.0, 2000.0], None, 25.0).unwrap().state,
            State::NeedDraw
        );
        assert!(
            sense.candidate(SCAN_SURFACE_TO_AIR, [7000.0, 100.0, 2000.0], None, 25.0).is_err(),
            "a second candidate while one is unfinished"
        );
        assert!(sense.draw(1.5).is_err(), "a draw outside [0, 1]");
        assert!(sense.signature(None, None).is_err(), "a signature before the draw");
    }
}
