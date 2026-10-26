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
