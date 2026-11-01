//! The reference host's terrain (EEDCS-47, D3): the physical terrain the campaign asks about, loaded and owned by the host, independently of
//! the campaign DLL. The solver is EECH's own, kept in C (`csrc/terrain_ref.c`, the original `get_3d_terrain_point_data` and fast elevation
//! expression for expression, over bounds-checked copies of the map files); this module loads the files, owns their memory, and offers the
//! campaign-facing vocabulary as a Lua provider.
//!
//! The provider (`host.terrain_open`) answers three operations, all by value:
//!
//! - `{ op = "map" }`: the map facts the campaign reads once (extents, sector counts, heights, the fast grid's size).
//! - `{ op = "point", x, z, hint }`: elevation, stored elevation, face normal and surface type at a point, using the EECH `terrain_types` ids
//!   as the surface vocabulary. `hint` is the opaque string this host last returned for the same record, or nil for a cleared record;
//!   the answer carries the record's next `hint` (a string, or nil when the query left none) and `stored`, by value: whether the record holds a
//!   surface afterwards (the legacy "a face is stored" test; independent of the hint, so a host that keeps no per-record state returns no hint and
//!   `stored = true`, or leaves `stored` out, which means true). A host that keeps no such state returns no
//!   hint and ignores one it is given.
//! - `{ op = "fast", x, z }`: the fast grid's elevation.
//!
//! A malformed request, a hint this host did not issue and any missing data are errors with a message; nothing falls back to another solver.
//!
//! Threads: the C solver reads its map facts from process-global statics, so every query and every conformance sweep, on any map, runs inside one
//! process-wide lock (`C_SOLVER`); independent `Terrain` instances may therefore be used on different threads (each is `!Send`, so it is created
//! on the thread that uses it). Floating-point state is per thread and is saved and restored around each query by the kernel.

use mlua::prelude::*;
use std::ffi::{c_char, c_void};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

/// The one lock around every use of the C solver's process-global statics (see the module comment).
static C_SOLVER: Mutex<()> = Mutex::new(());

fn solver_lock() -> MutexGuard<'static, ()> {
    // a panic while the lock was held leaves no state behind (the statics are reloaded by every query), so a poisoned lock is still a lock
    C_SOLVER.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A fresh identity for a map instance, folded into every hint it issues so that a hint cannot be presented to another map or instance.
fn new_map_key() -> u64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    static SERIAL: AtomicU64 = AtomicU64::new(1);
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(SERIAL.fetch_add(1, Ordering::Relaxed));
    hasher.finish()
}

/// `host_terrain_facts` (csrc/terrain_ref.h).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Facts {
    pub sector_x_max: i32,
    pub sector_z_max: i32,
    pub map_number: i32,
    pub version: u32,
    pub min_x: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_z: f32,
    pub min_height: f32,
    pub max_height: f32,
    pub scaled_height_difference: f32,
    pub fast_width: i32,
    pub fast_height: i32,
    pub fast_x_grid: f32,
    pub fast_z_grid: f32,
}

/// `host_terrain_hint` (csrc/terrain_ref.h): what a query leaves in the legacy record, by value.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Hint {
    pub x_sector: i32,
    pub z_sector: i32,
    pub face_index: u32,
    pub reference_offset: u32,
    pub triangle_offset: i32,
    pub kind: i32,
    pub normal: [f32; 3],
    /// binds the hint to the map instance that issued it and to the fields above (csrc/terrain_ref.c `hint_tag`)
    pub tag: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RawAnswer {
    elevation: f32,
    last_elevation: f32,
    normal: [f32; 3],
    kind: i32,
    has_hint: i32,
    stored: i32,
    hint: Hint,
    fast_attempted: i32,
    fast_hit: i32,
    found: i32,
    clamped: i32,
    cuba: i32,
    nan_position: i32,
}

/// `host_terrain_sweep` (csrc/terrain_ref.h): the totals of the conformance sweep.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SweepReport {
    pub sample_hash: u64,
    pub primed_hash: u64,
    pub repeat: u64,
    pub private_primed: u64,
    pub shared_primed: u64,
    pub private_primed_fast: u64,
    pub shared_primed_fast: u64,
    pub elevation_macro: u64,
    pub elevation_and_normal: u64,
    pub face_normal: u64,
    pub result_ne_last: u64,
    pub snapped: u64,
    pub primer_skipped: u64,
    pub fast_hash: u64,
    pub fast_count: u64,
    pub fast_available: u64,
}

/// `host_terrain_sweep_sample`: one reference answer of the sweep.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SweepSample {
    pub x: f32,
    pub z: f32,
    pub result: f32,
    pub last: f32,
    pub nx: f32,
    pub ny: f32,
    pub nz: f32,
    pub kind: i32,
    pub fast: f32,
}

/// The campaign's world-map area, by value (`host_ground_world`): campaign geography the below-ground test compares against. It is supplied by the caller and not assumed
/// equal to the terrain's extents.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GroundWorld {
    pub min_x: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_z: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RawGroundBelow {
    below: i32,
    queried: i32,
    record: RawAnswer,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RawGroundAbove {
    above: i32,
    queried: i32,
    record: RawAnswer,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RawGroundImpact {
    above: [f32; 3],
    below: [f32; 3],
    impact: [f32; 3],
    kind: i32,
    own_assert_failures: i32,
    record: RawAnswer,
}

/// `host_ground_point_sample` (csrc/terrain_ref.h): one emitted point case of the ground conformance probe. Words are bit patterns.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GroundPointSample {
    pub n: i32,
    pub tag: i32,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub inside: i32,
    pub above: i32,
    pub above_queried: i32,
    pub below: i32,
    pub below_queried: i32,
    pub above_elevation: u32,
    pub below_elevation: u32,
    pub above_type: i32,
    pub below_type: i32,
}

/// `host_ground_impact_sample`: one emitted impact case of the ground conformance probe. Words are bit patterns.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GroundImpactSample {
    pub n: i32,
    pub kind: i32,
    pub a0: [u32; 3],
    pub b0: [u32; 3],
    pub class_alias: i32,
    pub pa: [u32; 3],
    pub qa: [u32; 3],
    pub class_separate: i32,
    pub pb: [u32; 3],
    pub qb: [u32; 3],
    pub rb: [u32; 3],
    pub alias_equal: i32,
    pub warm_same: i32,
    pub queried: i32,
    pub elevation: u32,
    pub type_word: i32,
}

/// The sample capacities of the ground probe (2 per point tag x 6 tags, 4 per impact type x 8 types).
pub const GROUND_POINT_SAMPLES: usize = 12;
pub const GROUND_IMPACT_SAMPLES: usize = 32;

/// `host_ground_sweep` (csrc/terrain_ref.h): the totals of the ground conformance probe. The counter names are the fixture's report keys, with its "record touched" facts
/// reported as `queried` (the call read the terrain).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GroundSweep {
    pub physical_combined: u64,
    pub physical_above: u64,
    pub physical_below: u64,
    pub physical_impact: u64,
    pub points: i32,
    pub in_map: i32,
    pub off_map: i32,
    pub input_mutated: i32,
    pub above_cases: i32,
    pub above_true: i32,
    pub above_queried: i32,
    pub above_true_unqueried: i32,
    pub above_warm_same: i32,
    pub below_cases: i32,
    pub below_true: i32,
    pub below_queried: i32,
    pub below_true_unqueried: i32,
    pub below_warm_same: i32,
    pub below_off_map_true: i32,
    pub below_off_map_queried: i32,
    pub in_map_both: i32,
    pub complement: i32,
    pub not_complement: i32,
    pub both_true: i32,
    pub both_false: i32,
    pub impacts: i32,
    pub a_corrected: i32,
    pub b_corrected: i32,
    pub xz_changed: i32,
    pub alias_equal: i32,
    pub alias_unequal: i32,
    pub warm_same: i32,
    pub warm_different: i32,
    pub impact_queried: i32,
    pub impact_unqueried: i32,
    pub impact_equals_corrected_below: i32,
    pub class_count: [i32; 4],
    pub class_other: i32,
    pub by_type: [i32; 8],
    pub original_below_endpoint_still_above: i32,
    pub extra_above_endpoint_not_above: i32,
    pub own_assert_failures: i32,
    pub point_sample_count: i32,
    pub impact_sample_count: i32,
    pub point_samples: [GroundPointSample; GROUND_POINT_SAMPLES],
    pub impact_samples: [GroundImpactSample; GROUND_IMPACT_SAMPLES],
}

extern "C" {
    fn host_terrain_open(
        sector: *const u8,
        sector_length: usize,
        geometry: *const u8,
        geometry_length: usize,
        grid: *const u8,
        grid_length: usize,
        river_nodes: *const u8,
        river_node_length: usize,
        river_paths: *const u8,
        river_path_length: usize,
        map_number: i32,
        map_key: u64,
        error: *mut c_char,
        error_size: usize,
    ) -> *mut c_void;
    fn host_terrain_close(map: *mut c_void);
    fn host_terrain_grid_size(map: *const c_void) -> usize;
    fn host_terrain_copy_grid(map: *const c_void, out: *mut u8, capacity: usize, error: *mut c_char, error_size: usize) -> i32;
    fn host_terrain_get_facts(map: *const c_void, out: *mut Facts);
    fn host_terrain_query(map: *const c_void, x: f32, z: f32, hint: *const Hint, out: *mut RawAnswer, error: *mut c_char, error_size: usize) -> i32;
    fn host_terrain_fast_elevation(map: *const c_void, x: f32, z: f32, out: *mut f32, error: *mut c_char, error_size: usize) -> i32;
    fn host_terrain_conformance_sweep(
        map: *const c_void,
        seed: u32,
        count: i32,
        emit: i32,
        out: *mut SweepReport,
        samples: *mut SweepSample,
        error: *mut c_char,
        error_size: usize,
    ) -> i32;
    fn host_terrain_ground_above(
        map: *const c_void,
        world: *const GroundWorld,
        x: f32,
        y: f32,
        z: f32,
        hint: *const Hint,
        out: *mut RawGroundAbove,
        error: *mut c_char,
        error_size: usize,
    ) -> i32;
    fn host_terrain_ground_below(
        map: *const c_void,
        world: *const GroundWorld,
        x: f32,
        y: f32,
        z: f32,
        hint: *const Hint,
        out: *mut RawGroundBelow,
        error: *mut c_char,
        error_size: usize,
    ) -> i32;
    fn host_terrain_ground_impact(
        map: *const c_void,
        world: *const GroundWorld,
        above: *const f32,
        below: *const f32,
        hint: *const Hint,
        out: *mut RawGroundImpact,
        error: *mut c_char,
        error_size: usize,
    ) -> i32;
    fn host_terrain_ground_conformance(
        map: *const c_void,
        seed: u32,
        points: i32,
        impacts: i32,
        class_of_type: *const i32,
        class_count: usize,
        out: *mut GroundSweep,
        error: *mut c_char,
        error_size: usize,
    ) -> i32;
}

/// One point answer, with what the query did (for the conformance record; a provider for another world needs none of it).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Answer {
    pub elevation: f32,
    pub last_elevation: f32,
    pub normal: [f32; 3],
    pub kind: i32,
    pub hint: Option<Hint>,
    /// the record holds a face after this query (the legacy `terrain_face != NULL`, which stays true after a later no-face or Cuba answer): the
    /// campaign's `get_3d_terrain_point_data_has_face`, carried by value and independent of the hint
    pub stored: bool,
    pub fast_attempted: bool,
    pub fast_hit: bool,
    pub found: bool,
    pub clamped: bool,
    pub cuba: bool,
    pub nan_position: bool,
}

/// What `point_above_ground` answers (csrc/terrain_ref.c `host_terrain_ground_above`); `queried` and `record` are as for `GroundBelow`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundAbove {
    pub above: bool,
    pub queried: bool,
    pub record: Option<Answer>,
}

/// What `point_below_ground` answers (csrc/terrain_ref.c `host_terrain_ground_below`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundBelow {
    pub below: bool,
    /// the answer read the terrain. When false the caller's record must be left exactly as it is and `record` is `None`; when true `record` is the complete record the last read left
    /// (values, `stored` and the optional hint).
    pub queried: bool,
    pub record: Option<Answer>,
}

/// What `get_ground_impact_point` computes (csrc/terrain_ref.c `host_terrain_ground_impact`): the corrected endpoints, the impact point, the surface type the last query left
/// (the campaign maps it to a terrain class) and the record the sequence left. The caller composes the original's aliasing of the impact with the below endpoint from these.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundImpact {
    pub above: [f32; 3],
    pub below: [f32; 3],
    pub impact: [f32; 3],
    pub kind: i32,
    /// how many of the original function's ASSERT conditions were false (inert in the characterised build: counted, never fatal)
    pub own_assert_failures: i32,
    pub record: Answer,
}

fn answer_of(raw: &RawAnswer) -> Answer {
    Answer {
        elevation: raw.elevation,
        last_elevation: raw.last_elevation,
        normal: raw.normal,
        kind: raw.kind,
        hint: (raw.has_hint != 0).then_some(raw.hint),
        stored: raw.stored != 0,
        fast_attempted: raw.fast_attempted != 0,
        fast_hit: raw.fast_hit != 0,
        found: raw.found != 0,
        clamped: raw.clamped != 0,
        cuba: raw.cuba != 0,
        nan_position: raw.nan_position != 0,
    }
}

/// A loaded map. The kernel keeps its own copy of the sector geometry; the fast grid it reads in place, so that buffer is owned here.
pub struct Terrain {
    map: *mut c_void,
    facts: Facts,
    _grid: Option<Vec<u8>>,
}

impl Drop for Terrain {
    fn drop(&mut self) {
        let _guard = solver_lock();
        // SAFETY: `map` came from host_terrain_open and is closed once, under the solver lock; the grid buffer it points into is dropped after this.
        unsafe { host_terrain_close(self.map) }
    }
}

fn message(buffer: &[c_char]) -> String {
    let bytes: Vec<u8> = buffer.iter().take_while(|c| **c != 0).map(|c| *c as u8).collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// A file of `dir` by name, ignoring case (the retail installation mixes upper and lower case).
fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .find(|entry| entry.file_name().to_string_lossy().eq_ignore_ascii_case(name))
        .map(|entry| entry.path())
}

/// The map number the original parses from a path `...\map<N>\terrain`: the trailing digits of the directory above `terrain`.
pub fn map_number_of(terrain_dir: &Path) -> Result<i32, String> {
    let parent = terrain_dir
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| format!("terrain: {} has no parent directory to take the map number from", terrain_dir.display()))?;
    let digits: String = parent
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    digits
        .parse::<i32>()
        .map_err(|_| format!("terrain: the directory {parent:?} above {} does not end in a map number", terrain_dir.display()))
}

/// The sector file of a season (terrdata.c, load_3d_terrain_game_data): in winter the pair `default-W.sec` and `default-W.rgb` when BOTH exist,
/// otherwise the default `default.sec` (the colour file is not read here, but its presence decides the pair). The two seasons never mix: a lone
/// `default-W.sec` or `default-W.rgb` is ignored. `None` when the chosen sector file does not exist.
pub fn sector_file(terrain_dir: &Path, winter: bool) -> Option<PathBuf> {
    if winter {
        if let (Some(sector), Some(_)) = (find_file(terrain_dir, "default-W.sec"), find_file(terrain_dir, "default-W.rgb")) {
            return Some(sector);
        }
    }
    find_file(terrain_dir, "default.sec")
}

/// The river node and path files of the map above `terrain_dir` (terrmap.c, load_2d_terrain): `route/rivers.nde` with `rivers.wp`, else
/// `route/rivdata.nde` with `rivdata.wp`; `None` for a map with neither (a home-made map without rivers). A node file whose path file is missing is
/// an error: the original would read a file that is not there.
pub fn river_files(terrain_dir: &Path) -> Result<Option<(PathBuf, PathBuf)>, String> {
    let route = match terrain_dir.parent().and_then(|map| find_file(map, "route")) {
        Some(route) => route,
        None => return Ok(None),
    };
    for (nodes, paths) in [("rivers.nde", "rivers.wp"), ("rivdata.nde", "rivdata.wp")] {
        if let Some(node_file) = find_file(&route, nodes) {
            let path_file = find_file(&route, paths).ok_or_else(|| format!("terrain: {} has no {paths} beside it", node_file.display()))?;
            return Ok(Some((node_file, path_file)));
        }
    }
    Ok(None)
}

impl Terrain {
    /// Loads the map in `terrain_dir` (the `map<N>/terrain` directory): `terrain.ffp`, the sector file and, when present, `terrain.sim`.
    /// The sector file is the season's pair (`sector_file`): in winter `default-W.sec` when both it and `default-W.rgb` exist (terrdata.c), else
    /// `default.sec`; the season is an input, taken by the provider from the engine's map request. `map_number` overrides the number taken from the
    /// path. A map without `terrain.sim` gets the fast grid the original builds at load (256 m posts from the point query, the coast pass over the
    /// map's coastal rivers in `<map>/route/` and the sea normalisation; the host keeps it in memory instead of writing it into the installation).
    pub fn open(terrain_dir: &Path, winter: bool, map_number: Option<i32>) -> Result<Terrain, String> {
        let geometry_path = find_file(terrain_dir, "terrain.ffp").ok_or_else(|| format!("terrain: no terrain.ffp in {}", terrain_dir.display()))?;
        let sector_path = sector_file(terrain_dir, winter).ok_or_else(|| format!("terrain: no default.sec in {}", terrain_dir.display()))?;
        let number = match map_number {
            Some(number) => number,
            None => map_number_of(terrain_dir)?,
        };
        let read = |path: &Path| std::fs::read(path).map_err(|e| format!("terrain: cannot read {}: {e}", path.display()));
        let geometry = read(&geometry_path)?;
        let sector = read(&sector_path)?;
        let grid = match find_file(terrain_dir, "terrain.sim") {
            Some(path) => Some(read(&path)?),
            None => None,
        };
        // only a generated grid reads the rivers (terrmap.c load_2d_terrain: rivers.nde and rivers.wp, else rivdata.*, in the map's route directory)
        let rivers = match grid {
            Some(_) => None,
            None => river_files(terrain_dir)?
                .map(|(nodes, paths)| Ok::<_, String>((read(&nodes)?, read(&paths)?)))
                .transpose()?,
        };
        Terrain::from_bytes_with_rivers(&sector, &geometry, grid, rivers.as_ref().map(|(n, p)| (n.as_slice(), p.as_slice())), number)
    }

    /// Builds a map from the bytes of `default.sec`, `terrain.ffp` and (when there is one) `terrain.sim`; a map without a grid generates one with no
    /// coastal rivers (see `from_bytes_with_rivers`). The kernel checks every offset and count against the lengths and copies what it keeps; a
    /// malformed input is an error, never a read outside the buffers.
    pub fn from_bytes(sector: &[u8], geometry: &[u8], grid: Option<Vec<u8>>, map_number: i32) -> Result<Terrain, String> {
        Terrain::from_bytes_with_rivers(sector, geometry, grid, None, map_number)
    }

    /// `from_bytes`, with the bytes of the map's river nodes and river paths (`rivers.nde`/`rivers.wp` or `rivdata.nde`/`rivdata.wp`) for the coast
    /// pass of a generated grid. They are read while the grid is generated and not kept; a map that has a grid ignores them.
    pub fn from_bytes_with_rivers(
        sector: &[u8],
        geometry: &[u8],
        grid: Option<Vec<u8>>,
        rivers: Option<(&[u8], &[u8])>,
        map_number: i32,
    ) -> Result<Terrain, String> {
        let mut error = [0 as c_char; 512];
        let (nodes, paths) = match rivers {
            Some((nodes, paths)) => (nodes.as_ptr(), paths.as_ptr()),
            None => (std::ptr::null(), std::ptr::null()),
        };
        let (node_length, path_length) = rivers.map_or((0, 0), |(nodes, paths)| (nodes.len(), paths.len()));
        // generating the fast grid of a map without one uses the solver, so opening takes the solver lock as a query does
        let _guard = solver_lock();
        // SAFETY: the sector, geometry and river slices are valid for their lengths for the call only (the kernel copies what it keeps and reads the
        // rivers only while generating); the grid buffer is kept alive in the returned value for as long as the map exists; `error` is valid for
        // its length.
        let map = unsafe {
            host_terrain_open(
                sector.as_ptr(),
                sector.len(),
                geometry.as_ptr(),
                geometry.len(),
                grid.as_ref().map_or(std::ptr::null(), |g| g.as_ptr()),
                grid.as_ref().map_or(0, |g| g.len()),
                nodes,
                node_length,
                paths,
                path_length,
                map_number,
                new_map_key(),
                error.as_mut_ptr(),
                error.len(),
            )
        };
        if map.is_null() {
            return Err(message(&error));
        }
        let mut facts = Facts::default();
        // SAFETY: `map` is a live map and `facts` is a valid out pointer.
        unsafe { host_terrain_get_facts(map, &mut facts) };
        Ok(Terrain { map, facts, _grid: grid })
    }

    /// DIAGNOSTIC. The fast grid exactly as `terrain.sim` holds it (i32 width, i32 height, f32 x size, f32 z size, then the f32 posts, little endian),
    /// whether it was read from the file or generated; an error when the map has no grid. For comparing a generated grid with the original's file.
    pub fn grid_snapshot(&self) -> Result<Vec<u8>, String> {
        let _guard = solver_lock();
        // SAFETY: the map is live.
        let size = unsafe { host_terrain_grid_size(self.map) };
        if size == 0 {
            return Err("terrain: this map has no fast elevation grid".into());
        }
        let mut bytes = vec![0u8; size];
        let mut error = [0 as c_char; 256];
        // SAFETY: the map is live, `bytes` has the capacity it is told, and `error` is valid for its length.
        let status = unsafe { host_terrain_copy_grid(self.map, bytes.as_mut_ptr(), bytes.len(), error.as_mut_ptr(), error.len()) };
        if status != 0 {
            return Err(message(&error));
        }
        Ok(bytes)
    }

    pub fn facts(&self) -> &Facts {
        &self.facts
    }

    /// The point-surface query. `hint` is the one the last answer for this record carried (None for a cleared record).
    pub fn query(&self, x: f32, z: f32, hint: Option<&Hint>) -> Result<Answer, String> {
        let mut raw = RawAnswer::default();
        let mut error = [0 as c_char; 256];
        let _guard = solver_lock();
        // SAFETY: the map is live; the hint, when given, is a valid pointer for the call (the kernel validates its contents); the out pointers are valid;
        // the solver's process-global statics are used only under the solver lock.
        let status = unsafe {
            host_terrain_query(
                self.map,
                x,
                z,
                hint.map_or(std::ptr::null(), |h| h as *const Hint),
                &mut raw,
                error.as_mut_ptr(),
                error.len(),
            )
        };
        if status != 0 {
            return Err(message(&error));
        }
        Ok(Answer {
            elevation: raw.elevation,
            last_elevation: raw.last_elevation,
            normal: raw.normal,
            kind: raw.kind,
            hint: (raw.has_hint != 0).then_some(raw.hint),
            stored: raw.stored != 0,
            fast_attempted: raw.fast_attempted != 0,
            fast_hit: raw.fast_hit != 0,
            found: raw.found != 0,
            clamped: raw.clamped != 0,
            cuba: raw.cuba != 0,
            nan_position: raw.nan_position != 0,
        })
    }

    /// CONFORMANCE ONLY: the characterisation probe of `eech_terrain_fixture.c` (seed, `count` random points after six fixed ones, the first
    /// `emit` reference answers returned) run against this map. See `host_terrain_conformance_sweep`.
    pub fn conformance_sweep(&self, seed: u32, count: i32, emit: i32) -> Result<(SweepReport, Vec<SweepSample>), String> {
        // the original probe's bounds, checked before any arithmetic on them: a count of 0..=200000 and a non-negative emit, capped at 256
        if !(0..=200_000).contains(&count) || emit < 0 {
            return Err(format!("terrain sweep: count {count} or emit {emit} is out of range"));
        }
        // only the samples the sweep really produces are returned: no more than `emit` (capped at 256) and no more than the points there are
        let produced = emit.min(256).min(count + 6) as usize;
        let mut report = SweepReport::default();
        let mut samples = vec![SweepSample::default(); produced];
        let mut error = [0 as c_char; 256];
        let _guard = solver_lock();
        // SAFETY: the map is live; `samples` holds exactly the number of entries the kernel will write (it applies the same caps); the out pointers are
        // valid; the solver's process-global statics are used only under the solver lock, held across the whole sweep.
        let status = unsafe {
            host_terrain_conformance_sweep(
                self.map,
                seed,
                count,
                produced as i32,
                &mut report,
                samples.as_mut_ptr(),
                error.as_mut_ptr(),
                error.len(),
            )
        };
        if status != 0 {
            return Err(message(&error));
        }
        Ok((report, samples))
    }

    /// `point_above_ground` over this map: the sector maximum first and, when it does not decide, the elevation, exactly as `ground_below` reads them (the same helper). `world` is
    /// accepted and validated like the other ground operations' (the campaign supplies it with every ground question) but this predicate does not compare against it. A decided
    /// answer reads no terrain (`queried` false, the hint not looked at). Refused: as `ground_below`.
    pub fn ground_above(&self, world: &GroundWorld, x: f32, y: f32, z: f32, hint: Option<&Hint>) -> Result<GroundAbove, String> {
        let mut raw = RawGroundAbove::default();
        let mut error = [0 as c_char; 256];
        let _guard = solver_lock();
        // SAFETY: as `ground_below`.
        let status = unsafe {
            host_terrain_ground_above(
                self.map,
                world,
                x,
                y,
                z,
                hint.map_or(std::ptr::null(), |h| h as *const Hint),
                &mut raw,
                error.as_mut_ptr(),
                error.len(),
            )
        };
        if status != 0 {
            return Err(message(&error));
        }
        Ok(GroundAbove {
            above: raw.above != 0,
            queried: raw.queried != 0,
            record: (raw.queried != 0).then(|| answer_of(&raw.record)),
        })
    }

    /// `point_below_ground` over this map (G2 host side; the campaign DLL asks for it through the provider's `ground_below` operation). `world` is the campaign's world-map area and `hint` the one the caller's record
    /// carries (None for a cleared record); both are inputs, never assumed. A point outside the world area is below ground without any terrain read (`queried` false, `hint` not
    /// looked at); otherwise the sector maximum and, when it does not decide, the elevation are read as the original does. Refused: a coordinate that is not finite, a read off the
    /// terrain (the original stops the engine there), and a hint the map did not issue when the terrain is first read.
    pub fn ground_below(&self, world: &GroundWorld, x: f32, y: f32, z: f32, hint: Option<&Hint>) -> Result<GroundBelow, String> {
        let mut raw = RawGroundBelow::default();
        let mut error = [0 as c_char; 256];
        let _guard = solver_lock();
        // SAFETY: the map is live; `world` and the hint, when given, are valid pointers for the call (the kernel validates the hint's contents when it is used); the out pointers are
        // valid; the solver's process-global statics are used only under the solver lock.
        let status = unsafe {
            host_terrain_ground_below(
                self.map,
                world,
                x,
                y,
                z,
                hint.map_or(std::ptr::null(), |h| h as *const Hint),
                &mut raw,
                error.as_mut_ptr(),
                error.len(),
            )
        };
        if status != 0 {
            return Err(message(&error));
        }
        Ok(GroundBelow {
            below: raw.below != 0,
            queried: raw.queried != 0,
            record: (raw.queried != 0).then(|| answer_of(&raw.record)),
        })
    }

    /// `get_ground_impact_point` over this map (G2 host side; the provider's `ground_impact` operation). The endpoints are the original's inputs (above and below the ground); the answer carries both after the
    /// original's correction, the impact point, the surface type the last query left and the record. See `ground_below` for the inputs and the refusals.
    pub fn ground_impact(&self, world: &GroundWorld, above: [f32; 3], below: [f32; 3], hint: Option<&Hint>) -> Result<GroundImpact, String> {
        let mut raw = RawGroundImpact::default();
        let mut error = [0 as c_char; 256];
        let _guard = solver_lock();
        // SAFETY: as `ground_below`; the endpoint arrays are valid for three floats each.
        let status = unsafe {
            host_terrain_ground_impact(
                self.map,
                world,
                above.as_ptr(),
                below.as_ptr(),
                hint.map_or(std::ptr::null(), |h| h as *const Hint),
                &mut raw,
                error.as_mut_ptr(),
                error.len(),
            )
        };
        if status != 0 {
            return Err(message(&error));
        }
        Ok(GroundImpact {
            above: raw.above,
            below: raw.below,
            impact: raw.impact,
            kind: raw.kind,
            own_assert_failures: raw.own_assert_failures,
            record: answer_of(&raw.record),
        })
    }

    /// CONFORMANCE ONLY: the ground probe of `eech_ground_fixture.c` (the one the original queries were characterised with) driven against `ground_below` and `ground_impact`.
    /// `points` and `impacts` are 0..=100000 and not both 0; `class_of_type` is the campaign's terrain-type to class table (`TERRAIN_TYPE_LAST` entries, each 0..=3), which the
    /// caller supplies because the vocabulary is the campaign's. See `host_terrain_ground_conformance`.
    pub fn ground_conformance(&self, seed: u32, points: i32, impacts: i32, class_of_type: &[i32]) -> Result<GroundSweep, String> {
        if !(0..=100_000).contains(&points) || !(0..=100_000).contains(&impacts) || points + impacts == 0 {
            return Err(format!("terrain ground sweep: points {points} or impacts {impacts} is out of range"));
        }
        let mut report = GroundSweep::default();
        let mut error = [0 as c_char; 256];
        let _guard = solver_lock();
        // SAFETY: the map is live; the class table is valid for its length, which the kernel checks; `report` is a valid out pointer of the layout the kernel writes; the solver's
        // process-global statics are used only under the solver lock, held across the whole sweep.
        let status = unsafe {
            host_terrain_ground_conformance(
                self.map,
                seed,
                points,
                impacts,
                class_of_type.as_ptr(),
                class_of_type.len(),
                &mut report,
                error.as_mut_ptr(),
                error.len(),
            )
        };
        if status != 0 {
            return Err(message(&error));
        }
        Ok(report)
    }

    /// The fast grid's elevation (terrsel.c). An error when the map has no grid or the position cannot index it.
    pub fn fast_elevation(&self, x: f32, z: f32) -> Result<f32, String> {
        let mut value = 0.0f32;
        let mut error = [0 as c_char; 256];
        // SAFETY: the map is live and the out pointers are valid.
        let status = unsafe { host_terrain_fast_elevation(self.map, x, z, &mut value, error.as_mut_ptr(), error.len()) };
        if status != 0 {
            return Err(message(&error));
        }
        Ok(value)
    }
}

const HINT_PREFIX: &str = "e2";
const HINT_BYTES: usize = 44;
const HEX: &[u8; 16] = b"0123456789abcdef";

/// The opaque form of a hint for the Lua boundary: a version tag and the hint's fields, little endian, in hex. Only this host reads it.
pub fn encode_hint(hint: &Hint) -> String {
    let mut bytes = Vec::with_capacity(HINT_BYTES);
    bytes.extend_from_slice(&hint.x_sector.to_le_bytes());
    bytes.extend_from_slice(&hint.z_sector.to_le_bytes());
    bytes.extend_from_slice(&hint.face_index.to_le_bytes());
    bytes.extend_from_slice(&hint.reference_offset.to_le_bytes());
    bytes.extend_from_slice(&hint.triangle_offset.to_le_bytes());
    bytes.extend_from_slice(&hint.kind.to_le_bytes());
    for component in hint.normal {
        bytes.extend_from_slice(&component.to_le_bytes());
    }
    bytes.extend_from_slice(&hint.tag.to_le_bytes());
    let mut text = String::with_capacity(HINT_PREFIX.len() + 2 * HINT_BYTES);
    text.push_str(HINT_PREFIX);
    for byte in bytes {
        text.push(HEX[usize::from(byte >> 4)] as char);
        text.push(HEX[usize::from(byte & 15)] as char);
    }
    text
}

/// The inverse; anything that is not exactly what `encode_hint` writes is refused.
pub fn decode_hint(text: &str) -> Result<Hint, String> {
    let digits = text
        .strip_prefix(HINT_PREFIX)
        .ok_or_else(|| "terrain hint: not a hint this host issued".to_string())?;
    if digits.len() != 2 * HINT_BYTES || !digits.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return Err("terrain hint: not a hint this host issued".into());
    }
    let mut bytes = [0u8; HINT_BYTES];
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&digits[2 * i..2 * i + 2], 16).map_err(|_| "terrain hint: not a hint this host issued".to_string())?;
    }
    let word = |i: usize| [bytes[4 * i], bytes[4 * i + 1], bytes[4 * i + 2], bytes[4 * i + 3]];
    Ok(Hint {
        x_sector: i32::from_le_bytes(word(0)),
        z_sector: i32::from_le_bytes(word(1)),
        face_index: u32::from_le_bytes(word(2)),
        reference_offset: u32::from_le_bytes(word(3)),
        triangle_offset: i32::from_le_bytes(word(4)),
        kind: i32::from_le_bytes(word(5)),
        normal: [f32::from_le_bytes(word(6)), f32::from_le_bytes(word(7)), f32::from_le_bytes(word(8))],
        tag: u64::from_le_bytes([bytes[36], bytes[37], bytes[38], bytes[39], bytes[40], bytes[41], bytes[42], bytes[43]]),
    })
}

/// Reads a request number with the exact bits Lua holds. mlua 0.10 on Lua 5.1 turns any number within `f64::EPSILON` of an integer into `LuaValue::Integer`
/// when it is read as a `LuaValue` (main.rs `exact_numbers`), so a `-0.0` or a subnormal such as 1.4e-45 arrived as 0: the sign of a zero and every tiny magnitude were
/// erased before the terrain was asked. (The kernel's `coordinate - 1024` is computed under the engine's round-toward-zero mode, which does not absorb a tiny offset the way
/// round-to-nearest would, so the erased bits can reach an answer.)
/// A typed `f64` read of a real Lua number (`lua_tonumberx`) keeps every bit, but it also accepts a numeric string; so the field is looked up ONCE, in a small Lua
/// function (the same lookup a `request.get` makes, metamethods included), which records its type and, only for a real `number`, the unchanged value in a scratch table the
/// Rust side then reads. A string, nil, boolean or table is refused as before.
const COORDINATE_READER_KEY: &str = "eech_world_terrain_coordinate_reader";

const COORDINATE_READER: &str = r#"
local slot = {}
return {
	slot = slot,
	read = function (request, field)
		local value = request [field]
		local kind = type (value)
		slot.kind = kind
		slot.is_number = kind == "number"
		if kind == "number" then slot.value = value else slot.value = nil end
	end,
}
"#;

/// The coordinate reader of a Lua state: made once, kept in the registry.
struct CoordinateReader {
    read: LuaFunction,
    slot: LuaTable,
}

fn coordinate_reader(lua: &Lua) -> LuaResult<CoordinateReader> {
    let reader = match lua.named_registry_value::<LuaValue>(COORDINATE_READER_KEY)? {
        LuaValue::Table(reader) => reader,
        _ => {
            let reader: LuaTable = lua
                .load(COORDINATE_READER)
                .set_name("eech terrain coordinate reader")
                .into_function()?
                .call(())?;
            lua.set_named_registry_value(COORDINATE_READER_KEY, reader.clone())?;
            reader
        }
    };
    Ok(CoordinateReader {
        read: reader.get("read")?,
        slot: reader.get("slot")?,
    })
}

/// A request number that is exactly a single-precision value (the campaign's own), or not a finite number at all (the original answers a NaN
/// position with the map corner); any other double would be silently rounded and is refused.
fn coordinate(reader: &CoordinateReader, request: &LuaTable, field: &str) -> LuaResult<f32> {
    reader.read.call::<()>((request, field))?;
    if !reader.slot.get::<bool>("is_number")? {
        let kind: String = reader.slot.get("kind")?;
        return Err(LuaError::runtime(format!("host.terrain: {field} must be a number, got {kind}")));
    }
    let value: f64 = reader.slot.get("value")?;
    let single = value as f32;
    if value.is_finite() && f64::from(single) != value {
        return Err(LuaError::runtime(format!(
            "host.terrain: {field} {value} is not an exact single-precision value"
        )));
    }
    Ok(single)
}

fn facts_table(lua: &Lua, facts: &Facts) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    table.set("sector_x_max", facts.sector_x_max)?;
    table.set("sector_z_max", facts.sector_z_max)?;
    table.set("map_number", facts.map_number)?;
    table.set("version", f64::from(facts.version))?;
    table.set("min_x", f64::from(facts.min_x))?;
    table.set("min_z", f64::from(facts.min_z))?;
    table.set("max_x", f64::from(facts.max_x))?;
    table.set("max_z", f64::from(facts.max_z))?;
    table.set("min_height", f64::from(facts.min_height))?;
    table.set("max_height", f64::from(facts.max_height))?;
    table.set("scaled_height_difference", f64::from(facts.scaled_height_difference))?;
    table.set("fast_width", facts.fast_width)?;
    table.set("fast_height", facts.fast_height)?;
    table.set("fast_x_grid", f64::from(facts.fast_x_grid))?;
    table.set("fast_z_grid", f64::from(facts.fast_z_grid))?;
    Ok(table)
}

/// The provider function for a loaded map: see the module comment for the vocabulary.
pub fn provider(lua: &Lua, terrain: Rc<Terrain>) -> LuaResult<LuaFunction> {
    lua.create_function(move |lua, request: LuaTable| respond(lua, &terrain, &request))
}

/// The provider for the map of a boot, opened when the first question arrives and for the season that question carries (`winter`, from the engine:
/// the global season the original reads when it picks its terrain files). `asserted` is the season the script expects, if it says (the optional third
/// argument of `host.terrain_provider`): the engine's season is the truth, so a script that is wrong is refused rather than served another season's
/// map. A later question for another season is refused too: one boot, one map, never a mix of the two seasons' files.
fn seasonal_provider(lua: &Lua, dir: PathBuf, asserted: Option<bool>) -> LuaResult<LuaFunction> {
    let opened: std::cell::RefCell<Option<(bool, Rc<Terrain>)>> = std::cell::RefCell::new(None);
    lua.create_function(move |lua, request: LuaTable| {
        let winter = match request.get::<LuaValue>("winter")? {
            LuaValue::Boolean(winter) => winter,
            other => {
                return Err(LuaError::runtime(format!(
                    "host.terrain_provider: the request must carry the season as a boolean `winter`, got {}",
                    other.type_name()
                )))
            }
        };
        if asserted.is_some_and(|expected| expected != winter) {
            return Err(LuaError::runtime(format!(
                "host.terrain_provider: the script asserted winter = {} but the engine's season is winter = {winter}",
                !winter
            )));
        }
        let terrain = {
            let mut slot = opened.borrow_mut();
            match &*slot {
                Some((season, terrain)) if *season == winter => terrain.clone(),
                Some((season, _)) => {
                    return Err(LuaError::runtime(format!(
                        "host.terrain_provider: the map was opened for winter = {season} and is now asked for winter = {winter}"
                    )))
                }
                None => {
                    let terrain = open_cached(&dir, winter, None).map_err(|e| LuaError::runtime(format!("host.terrain_provider: {e}")))?;
                    *slot = Some((winter, terrain.clone()));
                    terrain
                }
            }
        };
        respond(lua, &terrain, &request)
    })
}

// ---- the ground operations of the provider (EEDCS-48, D3 G2; the campaign DLL sends them: mb_hostterrain.c, bridged by eech-dc GROUND_BRIDGE) ----
//
//   { op = "ground_above",  world = { min_x, min_z, max_x, max_z }, x, y, z, hint? }
//   { op = "ground_below",  world = { min_x, min_z, max_x, max_z }, x, y, z, hint? }
//   { op = "ground_impact", world = { min_x, min_z, max_x, max_z }, above = { x, y, z }, below = { x, y, z }, hint? }
//
// `world` is the campaign's world-map area: geography DATA the campaign supplies; the host performs the physical off-map predicate with it and never assumes it equals
// the terrain's extents. Every number is read with `coordinate` (a Lua number only, an exact single, the exact bits Lua holds, one field lookup). The calculations are the
// reviewed `Terrain::ground_below` and `Terrain::ground_impact`; nothing here computes a ground predicate. The hint is parsed here (host-private shape) and its tag is
// validated by the kernel when the terrain is first READ, so a decided-without-a-read answer never looks at it.
//
// Replies, by value:
//   ground_above:  { above = bool, queried = bool, record = nil | { last_elevation, normal = { nx, ny, nz }, type, stored, hint? } }
//   ground_below:  { below = bool, queried = bool, record = nil | { last_elevation, normal = { nx, ny, nz }, type, stored, hint? } }
//   ground_impact: { above = { x, y, z }, below = { x, y, z }, impact = { x, y, z }, type, record = { last_elevation, normal = { nx, ny, nz }, type, stored, hint? } }
// `record` is absent exactly when `queried` is false (the caller then keeps its whole record); when present it is the complete last record: `stored` is independent of `hint`,
// and a host that keeps no hint simply omits it. `type` is an EECH terrain type id; the type -> class mapping is the campaign's. The diagnostic assertion counters of the
// kernel are not part of the reply.

/// A request field that must be a table (the nested shapes of the ground operations).
fn request_table(request: &LuaTable, field: &str) -> LuaResult<LuaTable> {
    match request.get::<LuaValue>(field)? {
        LuaValue::Table(table) => Ok(table),
        other => Err(LuaError::runtime(format!("host.terrain: {field} must be a table, got {}", other.type_name()))),
    }
}

/// `coordinate` of `path.key`: the same strict read, its refusal naming where it was.
fn nested_coordinate(reader: &CoordinateReader, table: &LuaTable, path: &str, key: &str) -> LuaResult<f32> {
    coordinate(reader, table, key).map_err(|error| match error {
        LuaError::RuntimeError(message) => LuaError::runtime(format!("host.terrain: {path}.{key}: {}", message.trim_start_matches("host.terrain: "))),
        other => other,
    })
}

fn request_world(reader: &CoordinateReader, request: &LuaTable) -> LuaResult<GroundWorld> {
    let world = request_table(request, "world")?;
    Ok(GroundWorld {
        min_x: nested_coordinate(reader, &world, "world", "min_x")?,
        min_z: nested_coordinate(reader, &world, "world", "min_z")?,
        max_x: nested_coordinate(reader, &world, "world", "max_x")?,
        max_z: nested_coordinate(reader, &world, "world", "max_z")?,
    })
}

fn request_position(reader: &CoordinateReader, request: &LuaTable, field: &str) -> LuaResult<[f32; 3]> {
    let position = request_table(request, field)?;
    Ok([
        nested_coordinate(reader, &position, field, "x")?,
        nested_coordinate(reader, &position, field, "y")?,
        nested_coordinate(reader, &position, field, "z")?,
    ])
}

/// The record's hint, nil or the host-private string form: its shape is parsed here, its tag is not (the kernel checks that when it reads the terrain).
fn request_hint(request: &LuaTable) -> LuaResult<Option<Hint>> {
    match request.get::<LuaValue>("hint")? {
        LuaValue::Nil => Ok(None),
        LuaValue::String(text) => Ok(Some(decode_hint(&text.to_str()?).map_err(LuaError::runtime)?)),
        other => Err(LuaError::runtime(format!(
            "host.terrain: hint must be a string or nil, got {}",
            other.type_name()
        ))),
    }
}

fn position_table(lua: &Lua, position: &[f32; 3]) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    table.set("x", f64::from(position[0]))?;
    table.set("y", f64::from(position[1]))?;
    table.set("z", f64::from(position[2]))?;
    Ok(table)
}

/// The complete record a query left, by value: values, `stored` (independent of the hint) and the optional hint.
fn record_table(lua: &Lua, record: &Answer) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    table.set("last_elevation", f64::from(record.last_elevation))?;
    table.set("normal", record.normal.iter().map(|c| f64::from(*c)).collect::<Vec<f64>>())?;
    table.set("type", record.kind)?;
    table.set("stored", record.stored)?;
    if let Some(hint) = record.hint {
        table.set("hint", encode_hint(&hint))?;
    }
    Ok(table)
}

fn respond(lua: &Lua, terrain: &Terrain, request: &LuaTable) -> LuaResult<LuaTable> {
    {
        let op: String = match request.get::<LuaValue>("op")? {
            LuaValue::String(op) => op.to_str()?.to_string(),
            other => return Err(LuaError::runtime(format!("host.terrain: op must be a string, got {}", other.type_name()))),
        };
        match op.as_str() {
            "map" => facts_table(lua, terrain.facts()),
            "point" => {
                let reader = coordinate_reader(lua)?;
                let (x, z) = (coordinate(&reader, request, "x")?, coordinate(&reader, request, "z")?);
                let hint = match request.get::<LuaValue>("hint")? {
                    LuaValue::Nil => None,
                    LuaValue::String(text) => Some(decode_hint(&text.to_str()?).map_err(LuaError::runtime)?),
                    other => {
                        return Err(LuaError::runtime(format!(
                            "host.terrain: hint must be a string or nil, got {}",
                            other.type_name()
                        )))
                    }
                };
                let answer = terrain
                    .query(x, z, hint.as_ref())
                    .map_err(|e| LuaError::runtime(format!("host.terrain: {e}")))?;
                let table = lua.create_table()?;
                table.set("elevation", f64::from(answer.elevation))?;
                table.set("last_elevation", f64::from(answer.last_elevation))?;
                table.set("normal", answer.normal.iter().map(|c| f64::from(*c)).collect::<Vec<f64>>())?;
                table.set("type", answer.kind)?;
                table.set("stored", answer.stored)?;
                if let Some(hint) = answer.hint {
                    table.set("hint", encode_hint(&hint))?;
                }
                // what the query did: a diagnostic for conformance records, given only when the request asks (`diagnostic = true`); the campaign never does
                if matches!(request.get::<LuaValue>("diagnostic")?, LuaValue::Boolean(true)) {
                    let notes = lua.create_table()?;
                    notes.set("fast_attempted", answer.fast_attempted)?;
                    notes.set("fast_hit", answer.fast_hit)?;
                    notes.set("found", answer.found)?;
                    notes.set("clamped", answer.clamped)?;
                    notes.set("cuba", answer.cuba)?;
                    notes.set("nan_position", answer.nan_position)?;
                    table.set("notes", notes)?;
                }
                Ok(table)
            }
            "fast" => {
                let reader = coordinate_reader(lua)?;
                let (x, z) = (coordinate(&reader, request, "x")?, coordinate(&reader, request, "z")?);
                let elevation = terrain.fast_elevation(x, z).map_err(|e| LuaError::runtime(format!("host.terrain: {e}")))?;
                let table = lua.create_table()?;
                table.set("elevation", f64::from(elevation))?;
                Ok(table)
            }
            "ground_above" => {
                let reader = coordinate_reader(lua)?;
                let world = request_world(&reader, request)?;
                let (x, y, z) = (
                    coordinate(&reader, request, "x")?,
                    coordinate(&reader, request, "y")?,
                    coordinate(&reader, request, "z")?,
                );
                let hint = request_hint(request)?;
                let answer = terrain
                    .ground_above(&world, x, y, z, hint.as_ref())
                    .map_err(|e| LuaError::runtime(format!("host.terrain: {e}")))?;
                let table = lua.create_table()?;
                table.set("above", answer.above)?;
                table.set("queried", answer.queried)?;
                if let Some(record) = &answer.record {
                    table.set("record", record_table(lua, record)?)?;
                }
                Ok(table)
            }
            "ground_below" => {
                let reader = coordinate_reader(lua)?;
                let world = request_world(&reader, request)?;
                let (x, y, z) = (
                    coordinate(&reader, request, "x")?,
                    coordinate(&reader, request, "y")?,
                    coordinate(&reader, request, "z")?,
                );
                let hint = request_hint(request)?;
                let answer = terrain
                    .ground_below(&world, x, y, z, hint.as_ref())
                    .map_err(|e| LuaError::runtime(format!("host.terrain: {e}")))?;
                let table = lua.create_table()?;
                table.set("below", answer.below)?;
                table.set("queried", answer.queried)?;
                if let Some(record) = &answer.record {
                    table.set("record", record_table(lua, record)?)?;
                }
                Ok(table)
            }
            "ground_impact" => {
                let reader = coordinate_reader(lua)?;
                let world = request_world(&reader, request)?;
                let above = request_position(&reader, request, "above")?;
                let below = request_position(&reader, request, "below")?;
                let hint = request_hint(request)?;
                let answer = terrain
                    .ground_impact(&world, above, below, hint.as_ref())
                    .map_err(|e| LuaError::runtime(format!("host.terrain: {e}")))?;
                let table = lua.create_table()?;
                table.set("above", position_table(lua, &answer.above)?)?;
                table.set("below", position_table(lua, &answer.below)?)?;
                table.set("impact", position_table(lua, &answer.impact)?)?;
                table.set("type", answer.kind)?;
                table.set("record", record_table(lua, &answer.record)?)?;
                Ok(table)
            }
            other => Err(LuaError::runtime(format!("host.terrain: unknown op {other:?}"))),
        }
    }
}

/// The identity of an opened map: its terrain directory, the season it was opened for, and the map number override (if any).
type OpenMapKey = (PathBuf, bool, Option<i32>);

/// The maps this host has opened, by `OpenMapKey`.
type OpenMaps = std::collections::HashMap<OpenMapKey, Rc<Terrain>>;

thread_local! {
    // the maps this host has opened, by terrain directory, season and map number: two providers for one map share it (and so the identity its hints
    // carry), and a script that asks for a provider more than once does not reload the files
    static OPEN_MAPS: std::cell::RefCell<OpenMaps> = std::cell::RefCell::new(OpenMaps::new());
}

fn open_cached(dir: &Path, winter: bool, map_number: Option<i32>) -> Result<Rc<Terrain>, String> {
    OPEN_MAPS.with(|maps| {
        let key = (dir.to_path_buf(), winter, map_number);
        if let Some(terrain) = maps.borrow().get(&key) {
            return Ok(terrain.clone());
        }
        let terrain = Rc::new(Terrain::open(dir, winter, map_number)?);
        maps.borrow_mut().insert(key, terrain.clone());
        Ok(terrain)
    })
}

/// The terrain directory of the map a campaign is booted with: EECH map paths are relative to the installation's `cohokum` directory (the engine's
/// working directory), as in `..\common\maps\map5`.
pub fn terrain_dir_for(install_root: &str, map: &str) -> PathBuf {
    let map = map.replace('\\', "/");
    let root = Path::new(install_root);
    match map.strip_prefix("../") {
        Some(rest) => root.join(rest).join("terrain"),
        None => root.join("cohokum").join(&map).join("terrain"),
    }
}

/// Installs `host.terrain_provider (install_root, map [, winter])`: the reference host's terrain provider for the map a campaign boots with, to be given to
/// `dc.boot` as `terrain = ...`. It is a small shared helper for the reference host's scripts, not a default: the module never takes a provider from the
/// host's globals, and a boot without `terrain` is refused.
///
/// The map is opened lazily, when the engine first asks, and for the season that question carries (`request.winter`, the global season the original reads
/// when it picks `default-W.sec`/`default-W.rgb` or `default.sec`/`default.rgb`; `dc.boot { winter = true }` can explicitly override that native selection). The optional
/// `winter` argument is not a season to serve but an assertion of the one the script expects: a mismatch with the engine's season is refused. (The
/// engine's season is the only source, so the host's files and the sector data the campaign's own module loads can never be of different seasons.)
fn install_provider(lua: &Lua, host: &LuaTable) -> LuaResult<()> {
    host.set(
        "terrain_provider",
        lua.create_function(|lua, (install_root, map, winter): (String, String, Option<bool>)| {
            seasonal_provider(lua, terrain_dir_for(&install_root, &map), winter)
        })?,
    )
}

/// Installs `host.terrain_open { terrain_dir = "<...>/map<N>/terrain", winter = false, map_number = nil }`, which loads the map and returns its provider.
pub fn install(lua: &Lua, host: &LuaTable) -> LuaResult<()> {
    host.set(
        "terrain_open",
        lua.create_function(|lua, spec: LuaTable| {
            let dir: String = match spec.get::<LuaValue>("terrain_dir")? {
                LuaValue::String(dir) => dir.to_str()?.to_string(),
                other => {
                    return Err(LuaError::runtime(format!(
                        "host.terrain_open: terrain_dir must be a string, got {}",
                        other.type_name()
                    )))
                }
            };
            let winter = match spec.get::<LuaValue>("winter")? {
                LuaValue::Nil => false,
                LuaValue::Boolean(winter) => winter,
                other => {
                    return Err(LuaError::runtime(format!(
                        "host.terrain_open: winter must be a boolean, got {}",
                        other.type_name()
                    )))
                }
            };
            let map_number = match spec.get::<LuaValue>("map_number")? {
                LuaValue::Nil => None,
                LuaValue::Integer(number) => Some(i32::try_from(number).map_err(|_| LuaError::runtime("host.terrain_open: map_number is out of range"))?),
                other => {
                    return Err(LuaError::runtime(format!(
                        "host.terrain_open: map_number must be an integer, got {}",
                        other.type_name()
                    )))
                }
            };
            let terrain = open_cached(Path::new(&dir), winter, map_number).map_err(|e| LuaError::runtime(format!("host.terrain_open: {e}")))?;
            provider(lua, terrain)
        })?,
    )?;
    install_provider(lua, host)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hint_round_trips_and_nothing_else_decodes() {
        let hint = Hint {
            x_sector: 12,
            z_sector: -1,
            face_index: 4_000_000_000,
            reference_offset: 77,
            triangle_offset: 3,
            kind: 9,
            normal: [0.0, 0.97, -0.25],
            tag: 0x0123_4567_89ab_cdef,
        };
        let text = encode_hint(&hint);
        assert_eq!(text.len(), 2 + 88);
        assert_eq!(decode_hint(&text).unwrap(), hint);
        let bad: Vec<String> = vec![
            String::new(),
            "e2".into(),
            "e2".into(),
            text[..text.len() - 2].to_string(),
            format!("{text}00"),
            text.to_uppercase(),
            text.replacen("e2", "x2", 1),
            "e2zz".into(),
        ];
        for bad in &bad {
            assert!(decode_hint(bad).is_err(), "accepted {bad:?}");
        }
    }

    #[test]
    fn a_boot_map_path_names_its_terrain_directory() {
        assert_eq!(
            terrain_dir_for("R:/root", "..\\common\\maps\\map5"),
            Path::new("R:/root/common/maps/map5/terrain")
        );
        assert_eq!(terrain_dir_for("R:/root", "../common/maps/map3"), Path::new("R:/root/common/maps/map3/terrain"));
        assert_eq!(terrain_dir_for("R:/root", "maps\\map16"), Path::new("R:/root/cohokum/maps/map16/terrain"));
    }

    #[test]
    fn the_map_number_is_the_trailing_digits_above_terrain() {
        assert_eq!(map_number_of(Path::new("root/common/maps/map5/terrain")).unwrap(), 5);
        assert_eq!(map_number_of(Path::new("root/common/maps/map16/terrain")).unwrap(), 16);
        assert!(map_number_of(Path::new("root/common/maps/lebanon/terrain")).is_err());
        assert!(map_number_of(Path::new("terrain")).is_err());
    }

    // EEDCS-48 / EEDCS-47 regression: the original reader failed the actual library test for both fields of negative zero,
    // the smallest positive/negative subnormal and the smallest normal (one executed test, exit 101).
    //
    // The campaign's bridge hands the provider a Lua table `{ op, x, z, hint, winter }` whose x and z are Lua numbers made from the query's single-precision floats, and
    // `coordinate` reads them back with `request.get::<LuaValue>`. main.rs documents (exact_numbers) that mlua 0.10 on Lua 5.1 reports ANY number within f64::EPSILON of an integer
    // as `LuaValue::Integer`, so -0.0 and values such as 1.4e-45 arrive as Integer(0) and `coordinate` returns +0.0: the sign of a zero and every sub-epsilon magnitude are erased
    // before the terrain is asked. The original ground fixture's point generator deliberately asks for -0.0 and the next float above 0.0 (map-edge coordinates).
    //
    // This builds the request exactly as the bridge does (a Lua function making the table from Rust f64 arguments) and asserts the exact f32 bits `coordinate` returns, for the
    // edge values and for ordinary ones. The engine's directed rounding can preserve the effect of tiny coordinate differences;
    // case-level traces and original/candidate native probes establish the physical consequence separately from this input contract.
    #[test]
    fn a_request_coordinate_keeps_its_exact_single_precision_bits_including_signed_zero_and_tiny_values() {
        let lua = Lua::new();
        let reader = coordinate_reader(&lua).expect("the coordinate reader");
        let bridge: LuaFunction = lua
            .load("return function (op, x, z) return { op = op, x = x, z = z } end")
            .eval()
            .expect("the request builder");
        let cases: [(&str, f32); 12] = [
            ("negative zero", -0.0),
            ("positive zero", 0.0),
            ("smallest positive", f32::from_bits(1)),
            ("smallest negative", -f32::from_bits(1)),
            ("smallest normal", f32::MIN_POSITIVE),
            ("one", 1.0),
            ("a sector boundary", 2048.0),
            ("one float below a sector boundary", 2048.0_f32.next_down()),
            ("an ordinary fraction", 1234.5678),
            ("a large integer-valued coordinate", 155_647.0),
            ("one float below it", 155_647.0_f32.next_down()),
            ("an ordinary negative", -17.25),
        ];
        // every case is checked before the test fails, so one run classifies all of them
        let mut wrong = Vec::new();
        for (label, value) in cases {
            let request: LuaTable = bridge.call(("point", f64::from(value), f64::from(value))).expect("a request");
            for field in ["x", "z"] {
                match coordinate(&reader, &request, field) {
                    Ok(read) if read.to_bits() == value.to_bits() => {}
                    Ok(read) => wrong.push(format!(
                        "{label} ({field}): the provider read {:#010x}, the campaign sent {:#010x}",
                        read.to_bits(),
                        value.to_bits()
                    )),
                    Err(e) => wrong.push(format!("{label} ({field}): refused: {e}")),
                }
            }
        }
        assert!(wrong.is_empty(), "coordinates that did not arrive with their exact bits:\n{}", wrong.join("\n"));
    }

    // The numeric boundary of `coordinate` (EEDCS-48 / EEDCS-47): a typed f64 read keeps every bit of a real Lua number but would also take a numeric string, so the contract is
    // checked at the same boundary, on real Lua values: only a Lua number is a coordinate; a number that is not exactly a single is refused, not rounded; NaN and the infinities are
    // passed on (the solver answers them), not refused.
    #[test]
    fn a_request_coordinate_is_numeric_only_exact_single_precision_and_passes_nan_and_infinities_on() {
        let lua = Lua::new();
        let reader = coordinate_reader(&lua).expect("the coordinate reader");
        let request: LuaTable = lua
            .load(
                "return { s = '1.5', b = true, t = {}, f = function () end, n = 0.1, big = 1e39, tiny = 1e-50, nan = 0/0, pinf = 1/0, ninf = -1/0, one = 1.5 }",
            )
            .eval()
            .expect("a request");
        // not a number: refused with the type named; a numeric string is not a number
        for (field, kind) in [("s", "string"), ("b", "boolean"), ("t", "table"), ("f", "function"), ("absent", "nil")] {
            let error = coordinate(&reader, &request, field).expect_err(field).to_string();
            assert!(
                error.contains(&format!("host.terrain: {field} must be a number, got {kind}")),
                "{field}: {error}"
            );
        }
        // a finite double that is not exactly a single: refused, never rounded (0.1, beyond the single range, below the smallest subnormal single)
        for field in ["n", "big", "tiny"] {
            let error = coordinate(&reader, &request, field).expect_err(field).to_string();
            assert!(
                error.contains(&format!("host.terrain: {field} ")) && error.contains("is not an exact single-precision value"),
                "{field}: {error}"
            );
        }
        // NaN and the infinities are forwarded as they are
        assert!(coordinate(&reader, &request, "nan").expect("nan").is_nan());
        assert_eq!(coordinate(&reader, &request, "pinf").expect("+inf").to_bits(), f32::INFINITY.to_bits());
        assert_eq!(coordinate(&reader, &request, "ninf").expect("-inf").to_bits(), f32::NEG_INFINITY.to_bits());
        assert_eq!(coordinate(&reader, &request, "one").expect("1.5").to_bits(), 1.5_f32.to_bits());
    }

    // The field is read ONCE (a getter may be stateful, and a type check followed by a second read would see another value): this request answers 1.5 the first time it is asked
    // and a string afterwards, and counts its reads.
    #[test]
    fn a_request_coordinate_reads_its_field_exactly_once() {
        let lua = Lua::new();
        let reader = coordinate_reader(&lua).expect("the coordinate reader");
        let (request, reads): (LuaTable, LuaFunction) = lua
            .load(
                "local reads = 0
                 local request = setmetatable ({}, { __index = function (_, key) reads = reads + 1 if reads == 1 then return 1.5 end return 'later' end })
                 return request, function () return reads end",
            )
            .into_function()
            .expect("the witness")
            .call(())
            .expect("the witness request");
        assert_eq!(coordinate(&reader, &request, "x").expect("the first read").to_bits(), 1.5_f32.to_bits());
        assert_eq!(reads.call::<i32>(()).expect("the count"), 1, "the field was read more than once");
    }
}
