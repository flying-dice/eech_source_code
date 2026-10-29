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

/// A request number that is exactly a single-precision value (the campaign's own), or not a finite number at all (the original answers a NaN
/// position with the map corner); any other double would be silently rounded and is refused.
fn coordinate(request: &LuaTable, field: &str) -> LuaResult<f32> {
    let value = match request.get::<LuaValue>(field)? {
        LuaValue::Number(value) => value,
        LuaValue::Integer(value) => value as f64,
        other => return Err(LuaError::runtime(format!("host.terrain: {field} must be a number, got {}", other.type_name()))),
    };
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

fn respond(lua: &Lua, terrain: &Terrain, request: &LuaTable) -> LuaResult<LuaTable> {
    {
        let op: String = match request.get::<LuaValue>("op")? {
            LuaValue::String(op) => op.to_str()?.to_string(),
            other => return Err(LuaError::runtime(format!("host.terrain: op must be a string, got {}", other.type_name()))),
        };
        match op.as_str() {
            "map" => facts_table(lua, terrain.facts()),
            "point" => {
                let (x, z) = (coordinate(request, "x")?, coordinate(request, "z")?);
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
                let (x, z) = (coordinate(request, "x")?, coordinate(request, "z")?);
                let elevation = terrain.fast_elevation(x, z).map_err(|e| LuaError::runtime(format!("host.terrain: {e}")))?;
                let table = lua.create_table()?;
                table.set("elevation", f64::from(elevation))?;
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

    // EEDCS-48 / EEDCS-47 classification test: the actual library run fails for both fields of negative zero,
    // the smallest positive/negative subnormal and the smallest normal. No production fix is included.
    //
    // The campaign's bridge hands the provider a Lua table `{ op, x, z, hint, winter }` whose x and z are Lua numbers made from the query's single-precision floats, and
    // `coordinate` reads them back with `request.get::<LuaValue>`. main.rs documents (exact_numbers) that mlua 0.10 on Lua 5.1 reports ANY number within f64::EPSILON of an integer
    // as `LuaValue::Integer`, so -0.0 and values such as 1.4e-45 arrive as Integer(0) and `coordinate` returns +0.0: the sign of a zero and every sub-epsilon magnitude are erased
    // before the terrain is asked. The original ground fixture's point generator deliberately asks for -0.0 and the next float above 0.0 (map-edge coordinates).
    //
    // This builds the request exactly as the bridge does (a Lua function making the table from Rust f64 arguments) and asserts the exact f32 bits `coordinate` returns, for the
    // edge values and for ordinary ones. This proves input-bit loss, not the cause of the separate ground-physical hash mismatch:
    // the terrain kernel's coordinate offset can absorb these tiny differences. That mismatch still requires case-level tracing.
    #[test]
    fn a_request_coordinate_keeps_its_exact_single_precision_bits_including_signed_zero_and_tiny_values() {
        let lua = Lua::new();
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
                match coordinate(&request, field) {
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
}
