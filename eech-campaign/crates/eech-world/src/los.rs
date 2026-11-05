//! The host's line-of-sight kernel (EEDCS-42, D2): the physical calculation of whether one point sees another, kept in C
//! (`csrc/los_step.c`, EECH's own arithmetic expression for expression) and driven here one answer at a time. The campaign asks a question
//! with `begin`; the host answers `Done`, or asks the campaign for one terrain elevation or one object-ray result, which the campaign supplies
//! with `terrain` or `objects`. There is at most one open query, a new `begin` replaces it, and an answer that was not asked for is refused.
//!
//! The terrain heights (D3) and the object collision scan (D5) are not the host's yet; that is declared transitional coupling. Nothing here
//! decides what to ask about or what the answer means.

use std::ffi::{c_int, c_void};

/// The campaign's `campaign_los_phase` values, which the original solver's transcript names.
pub const PHASE_COARSE: i32 = 1;
pub const PHASE_SOURCE_FINE: i32 = 2;
pub const PHASE_TARGET_FINE: i32 = 3;
pub const PHASE_SOURCE_OBJECTS: i32 = 4;
pub const PHASE_TARGET_OBJECTS: i32 = 5;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RawRequest {
    state: c_int,
    phase: c_int,
    point: [f32; 3],
    start: [f32; 3],
    end: [f32; 3],
    visible: c_int,
}

extern "C" {
    fn eech_los_query_size() -> usize;
    fn eech_los_begin(query: *mut c_void, source: *const f32, target: *const f32, criteria: c_int, source_eye: c_int, target_eye: c_int, out: *mut RawRequest);
    fn eech_los_terrain(query: *mut c_void, elevation: f32, out: *mut RawRequest) -> c_int;
    fn eech_los_objects(query: *mut c_void, hit: c_int, collision_point: *const f32, out: *mut RawRequest) -> c_int;
}

/// What the host says next.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LosRequest {
    /// the terrain elevation at `point` (x, the ray's height there, z)
    NeedTerrain {
        phase: i32,
        point: [f32; 3],
    },
    /// whether an entity blocks the ray from `start` to `end`, and where
    NeedObjects {
        phase: i32,
        start: [f32; 3],
        end: [f32; 3],
    },
    Done {
        visible: bool,
    },
}

fn request(raw: &RawRequest) -> Result<LosRequest, String> {
    match raw.state {
        1 => Ok(LosRequest::Done { visible: raw.visible != 0 }),
        2 => Ok(LosRequest::NeedTerrain {
            phase: raw.phase,
            point: raw.point,
        }),
        3 => Ok(LosRequest::NeedObjects {
            phase: raw.phase,
            start: raw.start,
            end: raw.end,
        }),
        other => Err(format!("line of sight kernel returned an unknown state {other}")),
    }
}

/// The one open query, in the kernel's own state (an opaque, suitably aligned block).
pub struct Los {
    query: Vec<u64>,
    open: bool,
}

impl Default for Los {
    fn default() -> Self {
        Self::new()
    }
}

impl Los {
    #[must_use]
    pub fn new() -> Self {
        // SAFETY: a pure size query.
        let bytes = unsafe { eech_los_query_size() };
        Self {
            query: vec![0u64; bytes.div_ceil(8)],
            open: false,
        }
    }

    /// Starts a query for the ray from `source` to `target` with the campaign's `criteria` bits (the five `MOBILE_LOS_CHECK_*` bits), lifting
    /// an end 1.5 m when the campaign says it is a vehicle. Replaces any open query.
    pub fn begin(&mut self, source: [f32; 3], target: [f32; 3], criteria: i32, source_eye: bool, target_eye: bool) -> Result<LosRequest, String> {
        if !(0..32).contains(&criteria) {
            return Err(format!("line of sight criteria {criteria} are outside the five check bits"));
        }
        if source.iter().chain(target.iter()).any(|value| !value.is_finite()) {
            return Err("line of sight end points must be finite".into());
        }
        let mut raw = RawRequest::default();
        // SAFETY: `query` is the kernel's block, sized by it; the vectors and the result are valid for the call.
        unsafe {
            eech_los_begin(
                self.query.as_mut_ptr().cast(),
                source.as_ptr(),
                target.as_ptr(),
                criteria,
                c_int::from(source_eye),
                c_int::from(target_eye),
                &mut raw,
            );
        }
        self.settle(&raw)
    }

    /// The campaign's elevation for the sample just asked about.
    pub fn terrain(&mut self, elevation: f32) -> Result<LosRequest, String> {
        if !elevation.is_finite() {
            return Err("line of sight terrain elevation must be finite".into());
        }
        if !self.open {
            return Err("line of sight: a terrain answer arrived with no query open".into());
        }
        let mut raw = RawRequest::default();
        // SAFETY: as in `begin`.
        let code = unsafe { eech_los_terrain(self.query.as_mut_ptr().cast(), elevation, &mut raw) };
        if code != 0 {
            return Err("line of sight: a terrain answer arrived but none was asked for".into());
        }
        self.settle(&raw)
    }

    /// The campaign's answer to the object ray just asked about; the collision point is meaningful, and required, only when it hit.
    pub fn objects(&mut self, hit: bool, collision_point: Option<[f32; 3]>) -> Result<LosRequest, String> {
        if !self.open {
            return Err("line of sight: an objects answer arrived with no query open".into());
        }
        let point = match (hit, collision_point) {
            (true, Some(point)) if point.iter().all(|value| value.is_finite()) => point,
            (true, Some(_)) => return Err("line of sight collision point must be finite".into()),
            (true, None) => return Err("line of sight: an objects hit needs its collision point".into()),
            (false, None) => [0.0; 3],
            (false, Some(_)) => return Err("line of sight: an objects miss must not carry a collision point".into()),
        };
        let mut raw = RawRequest::default();
        // SAFETY: as in `begin`.
        let code = unsafe { eech_los_objects(self.query.as_mut_ptr().cast(), c_int::from(hit), point.as_ptr(), &mut raw) };
        if code != 0 {
            return Err("line of sight: an objects answer arrived but none was asked for".into());
        }
        self.settle(&raw)
    }

    /// Drops any open query (a host that is told the campaign stopped mid-query).
    pub fn abandon(&mut self) {
        self.open = false;
    }

    fn settle(&mut self, raw: &RawRequest) -> Result<LosRequest, String> {
        let next = request(raw)?;
        self.open = !matches!(next, LosRequest::Done { .. });
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    //! Focused contracts of the kernel against a scripted environment: the terrain heights and the object answers are the genuine
    //! environmental inputs the campaign supplies, and the expectations are derived from EECH's `check_position_line_of_sight`
    //! (mb_tgt.c) as written, including its quirks, not from this implementation. Bit-for-bit equality with the original solver is
    //! protected separately by the lookup transcript, which these tests do not replace.
    use super::*;

    const COARSE: i32 = 1;
    const SOURCE_FINE: i32 = 2;
    const TARGET_FINE: i32 = 4;
    const SOURCE_OBJECTS: i32 = 8;
    const TARGET_OBJECTS: i32 = 16;
    const ALL: i32 = 31;

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.01
    }

    /// Runs one query to its answer, giving `terrain (point)` for each terrain request and `objects (start, end)` for each object ray.
    fn run(
        source: [f32; 3],
        target: [f32; 3],
        criteria: i32,
        eyes: (bool, bool),
        terrain: impl Fn([f32; 3]) -> f32,
        objects: impl Fn([f32; 3], [f32; 3]) -> Option<[f32; 3]>,
    ) -> (bool, Vec<LosRequest>) {
        let mut los = Los::new();
        let mut seen = Vec::new();
        let mut next = los.begin(source, target, criteria, eyes.0, eyes.1).unwrap();
        loop {
            match next {
                LosRequest::Done { visible } => return (visible, seen),
                LosRequest::NeedTerrain { point, .. } => {
                    seen.push(next);
                    next = los.terrain(terrain(point)).unwrap();
                }
                LosRequest::NeedObjects { start, end, .. } => {
                    seen.push(next);
                    let hit = objects(start, end);
                    next = los.objects(hit.is_some(), hit).unwrap();
                }
            }
            assert!(seen.len() < 10_000, "a query that never ends");
        }
    }

    fn flat(_: [f32; 3]) -> f32 {
        0.0
    }

    fn clear(_: [f32; 3], _: [f32; 3]) -> Option<[f32; 3]> {
        None
    }

    fn terrain_points(seen: &[LosRequest], wanted_phase: i32) -> Vec<[f32; 3]> {
        seen.iter()
            .filter_map(|request| match request {
                LosRequest::NeedTerrain { phase, point } if *phase == wanted_phase => Some(*point),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn no_criteria_is_visible_without_asking_the_campaign_anything() {
        let (visible, seen) = run([0.0, 10.0, 0.0], [1000.0, 10.0, 0.0], 0, (false, false), flat, clear);
        assert!(visible);
        assert!(seen.is_empty());
    }

    #[test]
    fn coarse_samples_start_at_the_source_every_200_m_and_end_before_the_target() {
        let (visible, seen) = run([0.0, 100.0, 0.0], [1000.0, 100.0, 0.0], COARSE, (false, false), flat, clear);
        assert!(visible);
        let points = terrain_points(&seen, PHASE_COARSE);
        assert_eq!(points.len(), 4, "{points:?}");
        for (k, point) in points.iter().enumerate() {
            assert!(near(point[0], 200.0 * (k as f32 + 1.0)), "{points:?}");
            assert!(near(point[1], 100.0) && near(point[2], 0.0), "{points:?}");
        }
    }

    #[test]
    fn the_coarse_loop_runs_while_the_counter_is_above_one() {
        // range / 200 is 1.0 at 200 m: no sample; 400 m gives 2.0 and one sample; 399 m gives 1.995 and one sample
        for (range, samples) in [(200.0f32, 0usize), (400.0, 1), (399.0, 1), (199.0, 0), (600.0, 2)] {
            let (visible, seen) = run([0.0, 50.0, 0.0], [range, 50.0, 0.0], COARSE, (false, false), flat, clear);
            assert!(visible);
            assert_eq!(seen.len(), samples, "range {range}");
        }
    }

    #[test]
    fn the_first_blocker_ends_the_query_and_later_samples_are_never_asked_for() {
        let ridge = |p: [f32; 3]| if near(p[0], 600.0) { 150.0 } else { 0.0 };
        let (visible, seen) = run([0.0, 100.0, 0.0], [1000.0, 100.0, 0.0], ALL, (false, false), ridge, clear);
        assert!(!visible);
        assert_eq!(seen.len(), 3, "samples at 200, 400 and 600, then nothing: {seen:?}");
    }

    #[test]
    fn a_sample_exactly_at_the_ray_height_does_not_block() {
        let level = |p: [f32; 3]| p[1];
        let (visible, _) = run([0.0, 100.0, 0.0], [1000.0, 100.0, 0.0], COARSE, (false, false), level, clear);
        assert!(visible, "terrain equal to the ray height is not above it");
        let above = |p: [f32; 3]| p[1] + 0.5;
        let (visible, seen) = run([0.0, 100.0, 0.0], [1000.0, 100.0, 0.0], COARSE, (false, false), above, clear);
        assert!(!visible);
        assert_eq!(seen.len(), 1);
    }

    #[test]
    fn the_range_is_horizontal_but_the_steps_follow_the_three_dimensional_direction() {
        // 1000 m east and 1000 m up: the original counts 1000 m of range (x and z only), so 4 samples, but steps 200 m along
        // the 45 degree direction, so the first sample is 141 m east and 141 m up
        let (visible, seen) = run([0.0, 0.0, 0.0], [1000.0, 1000.0, 0.0], COARSE, (false, false), |_| -10_000.0, clear);
        assert!(visible);
        let points = terrain_points(&seen, PHASE_COARSE);
        assert_eq!(points.len(), 4);
        assert!(near(points[0][0], 141.42) && near(points[0][1], 141.42), "{points:?}");
        assert!(near(points[3][0], 565.69) && near(points[3][1], 565.69), "{points:?}");
    }

    #[test]
    fn source_end_fine_terrain_is_at_most_seven_samples_of_40_m_from_the_source() {
        let (_, seen) = run([0.0, 100.0, 0.0], [1000.0, 100.0, 0.0], SOURCE_FINE, (false, false), flat, clear);
        let points = terrain_points(&seen, PHASE_SOURCE_FINE);
        assert_eq!(points.len(), 7, "a counter capped at 8 and run while above 1 makes 7 queries: {points:?}");
        for (k, point) in points.iter().enumerate() {
            assert!(near(point[0], 40.0 * (k as f32 + 1.0)), "{points:?}");
        }
        // a short ray: 100 m / 40 m = 2.5, so two samples
        let (_, seen) = run([0.0, 100.0, 0.0], [100.0, 100.0, 0.0], SOURCE_FINE, (false, false), flat, clear);
        assert_eq!(seen.len(), 2);
    }

    #[test]
    fn target_end_fine_terrain_walks_back_from_the_target_in_24_m_steps() {
        let (_, seen) = run([0.0, 100.0, 0.0], [1000.0, 100.0, 0.0], TARGET_FINE, (false, false), flat, clear);
        let points = terrain_points(&seen, PHASE_TARGET_FINE);
        assert_eq!(points.len(), 7, "{points:?}");
        for (k, point) in points.iter().enumerate() {
            assert!(near(point[0], 1000.0 - 24.0 * (k as f32 + 1.0)), "{points:?}");
        }
    }

    #[test]
    fn the_phases_come_in_the_original_order() {
        let (visible, seen) = run([0.0, 100.0, 0.0], [1000.0, 100.0, 0.0], ALL, (false, false), flat, clear);
        assert!(visible);
        let phases: Vec<i32> = seen
            .iter()
            .map(|request| match request {
                LosRequest::NeedTerrain { phase, .. } | LosRequest::NeedObjects { phase, .. } => *phase,
                LosRequest::Done { .. } => 0,
            })
            .collect();
        let mut expected = vec![PHASE_COARSE; 4];
        expected.extend([PHASE_SOURCE_FINE; 7]);
        expected.extend([PHASE_TARGET_FINE; 7]);
        expected.extend([PHASE_SOURCE_OBJECTS, PHASE_TARGET_OBJECTS]);
        assert_eq!(phases, expected);
    }

    #[test]
    fn each_object_ray_is_at_most_200_m_long_from_its_own_end() {
        let (_, seen) = run(
            [0.0, 10.0, 0.0],
            [1000.0, 10.0, 0.0],
            SOURCE_OBJECTS | TARGET_OBJECTS,
            (false, false),
            flat,
            clear,
        );
        let LosRequest::NeedObjects { start, end, phase } = seen[0] else {
            panic!("{seen:?}")
        };
        assert_eq!(phase, PHASE_SOURCE_OBJECTS);
        assert!(near(start[0], 0.0) && near(end[0], 200.0));
        let LosRequest::NeedObjects { start, end, phase } = seen[1] else {
            panic!("{seen:?}")
        };
        assert_eq!(phase, PHASE_TARGET_OBJECTS);
        assert!(near(start[0], 1000.0) && near(end[0], 800.0));
        // a ray shorter than 200 m is as long as the range
        let (_, seen) = run([0.0, 10.0, 0.0], [50.0, 10.0, 0.0], SOURCE_OBJECTS, (false, false), flat, clear);
        let LosRequest::NeedObjects { end, .. } = seen[0] else { panic!("{seen:?}") };
        assert!(near(end[0], 50.0));
    }

    #[test]
    fn a_source_end_object_hit_blocks_at_any_distance() {
        let (visible, seen) = run(
            [0.0, 10.0, 0.0],
            [1000.0, 10.0, 0.0],
            SOURCE_OBJECTS | TARGET_OBJECTS,
            (false, false),
            flat,
            |start, _| near(start[0], 0.0).then_some([0.5, 10.0, 0.0]),
        );
        assert!(!visible);
        assert_eq!(seen.len(), 1, "the target-end ray is never asked after a source-end block");
    }

    #[test]
    fn a_target_end_hit_blocks_only_beyond_12_m_of_the_target() {
        let hit_at = |x: f32| {
            run([0.0, 0.0, 0.0], [1000.0, 0.0, 0.0], TARGET_OBJECTS, (false, false), flat, move |_, _| {
                Some([x, 0.0, 0.0])
            })
            .0
        };
        assert!(hit_at(1000.0), "at the target");
        assert!(hit_at(988.0), "exactly 12 m: the squared range 144 is not greater than 144");
        assert!(!hit_at(987.0), "13 m is beyond the exclusion distance");
        assert!(!hit_at(1013.0), "13 m on the far side counts as well");
        assert!(hit_at(1012.0), "12 m on the far side is excluded too");
    }

    #[test]
    fn the_exclusion_distance_is_three_dimensional() {
        // 12 m along x and 5 m of height: 13 m
        let (visible, _) = run([0.0, 0.0, 0.0], [1000.0, 0.0, 0.0], TARGET_OBJECTS, (false, false), flat, |_, _| {
            Some([988.0, 5.0, 0.0])
        });
        assert!(!visible);
    }

    #[test]
    fn the_entity_form_lifts_a_vehicle_end_by_one_and_a_half_metres() {
        let seen_at = |eyes: (bool, bool)| {
            let (_, seen) = run([0.0, 10.0, 0.0], [1000.0, 10.0, 0.0], SOURCE_OBJECTS | TARGET_OBJECTS, eyes, flat, clear);
            let LosRequest::NeedObjects { start: s, .. } = seen[0] else { panic!() };
            let LosRequest::NeedObjects { start: t, .. } = seen[1] else { panic!() };
            (s[1], t[1])
        };
        assert_eq!(seen_at((false, false)), (10.0, 10.0));
        assert_eq!(seen_at((true, false)), (11.5, 10.0));
        assert_eq!(seen_at((false, true)), (10.0, 11.5));
        assert_eq!(seen_at((true, true)), (11.5, 11.5));
    }

    #[test]
    fn a_vertical_ray_has_no_range_and_so_no_terrain_samples_and_zero_length_object_rays() {
        // characterised, not corrected: the direction is straight up, the horizontal range is 0, every terrain counter is 0, and the
        // object rays are zero long at each end
        let (visible, seen) = run([0.0, 0.0, 0.0], [0.0, 100.0, 0.0], ALL, (false, false), |_| 1.0e9, clear);
        assert!(visible, "no terrain sample is taken, however high the terrain");
        assert_eq!(seen.len(), 2);
        let LosRequest::NeedObjects { start, end, .. } = seen[0] else {
            panic!("{seen:?}")
        };
        assert_eq!((start, end), ([0.0; 3], [0.0; 3]));
        let LosRequest::NeedObjects { start, end, .. } = seen[1] else {
            panic!("{seen:?}")
        };
        assert_eq!((start, end), ([0.0, 100.0, 0.0], [0.0, 100.0, 0.0]));
    }

    #[test]
    fn two_ends_at_the_same_point_behave_as_a_zero_range_ray() {
        let (visible, seen) = run([5.0, 5.0, 5.0], [5.0, 5.0, 5.0], ALL, (false, false), |_| 1.0e9, clear);
        assert!(visible);
        assert_eq!(seen.len(), 2, "only the two zero-length object rays: {seen:?}");
    }

    #[test]
    fn an_answer_that_was_not_asked_for_is_refused_and_changes_nothing() {
        let mut los = Los::new();
        assert!(los.terrain(0.0).is_err(), "nothing is open");
        assert!(los.objects(false, None).is_err(), "nothing is open");
        let first = los.begin([0.0, 100.0, 0.0], [1000.0, 100.0, 0.0], COARSE, false, false).unwrap();
        assert!(matches!(first, LosRequest::NeedTerrain { .. }));
        assert!(los.objects(false, None).is_err(), "terrain was asked for");
        // the refused answer left the query exactly where it was
        let second = los.terrain(0.0).unwrap();
        let LosRequest::NeedTerrain { point, .. } = second else {
            panic!("{second:?}")
        };
        assert!(near(point[0], 400.0), "{point:?}");
    }

    #[test]
    fn an_answer_after_the_query_is_done_is_refused() {
        let mut los = Los::new();
        let done = los.begin([0.0, 0.0, 0.0], [10.0, 0.0, 0.0], 0, false, false).unwrap();
        assert_eq!(done, LosRequest::Done { visible: true });
        assert!(los.terrain(0.0).is_err());
        assert!(los.objects(false, None).is_err());
    }

    #[test]
    fn a_new_query_replaces_an_open_one() {
        let mut los = Los::new();
        los.begin([0.0, 100.0, 0.0], [1000.0, 100.0, 0.0], COARSE, false, false).unwrap();
        los.terrain(0.0).unwrap();
        let fresh = los.begin([0.0, 100.0, 0.0], [1000.0, 100.0, 0.0], COARSE, false, false).unwrap();
        let LosRequest::NeedTerrain { point, .. } = fresh else { panic!("{fresh:?}") };
        assert!(near(point[0], 200.0), "the replacement starts from its own first sample: {point:?}");
    }

    #[test]
    fn malformed_inputs_are_refused() {
        let mut los = Los::new();
        assert!(los.begin([0.0; 3], [1.0; 3], 32, false, false).is_err());
        assert!(los.begin([0.0; 3], [1.0; 3], -1, false, false).is_err());
        assert!(los.begin([f32::NAN, 0.0, 0.0], [1.0; 3], 1, false, false).is_err());
        assert!(los.begin([0.0; 3], [f32::INFINITY, 0.0, 0.0], 1, false, false).is_err());
        los.begin([0.0, 100.0, 0.0], [1000.0, 100.0, 0.0], COARSE, false, false).unwrap();
        assert!(los.terrain(f32::NAN).is_err());
        let mut los = Los::new();
        los.begin([0.0, 0.0, 0.0], [1000.0, 0.0, 0.0], SOURCE_OBJECTS, false, false).unwrap();
        assert!(los.objects(true, None).is_err(), "a hit needs its point");
        assert!(los.objects(false, Some([0.0; 3])).is_err(), "a miss carries no point");
        assert!(los.objects(true, Some([f32::NAN, 0.0, 0.0])).is_err());
    }
}
