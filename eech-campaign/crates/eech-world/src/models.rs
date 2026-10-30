//! The host-owned collision-model store (EEDCS-50, D3): the model data the campaign's physical queries read, loaded by the HOST from the retail disk assets, never
//! copied or exported from the campaign DLL's tables.
//!
//! What is read, from the original's own formats (modules/3d/3dobjdb.c `initialise_3d_objects`, `read_scene`, `read_object`, `read_custom_scene`, 3dobjid.c):
//!
//! * `bininfo.bin`: the scene names, checked against the compiled list (`data/scene-names.txt`, derived from modules/3d/3dmodels.h).
//! * `3dobjs.bin/.pts/.sp`: the object table. Per object the counts, bounds, radius, surfaces (the 18-byte record the loader reads into 16-byte structs), faces, the point list and the surface-point references, over the file-mapped point and surface-point arrays.
//! * `3dobjdb.bin`: the scene table. Every `read_scene` record, walked by its own variable counts (cameras, links, lights, keyframes, sub-object trees).
//! * `OBJECTS/????.EEO` and `OBJECTS/<dir>/*.EES`: binary-hex object overrides, and named scenes (versions -1 to -3) with their named `.EEO` objects resolved per directory. Named objects are numbered from the retail table's object COUNT as the original numbers them: the first replaces the last retail object.
//!
//! What is NOT read: textures, animations, cameras, lights, normals, the d3d buffers (their records are walked or skipped by the original's own counts, never by guessed
//! offsets); `3dobjs.spn/.spt`; the camera database after the scenes.
//!
//! The legacy layout stays private. The original's common blocks (faces, point list, surface points, points, the consolidated surfaces) are kept as bounds-checked
//! arrays with a `Span` per object; an object's own extent is NOT the limit of a read (the original's decoder walks past it into the next object's data when the
//! counts disagree: the 45 mixed-layout models), the BACKED length of the array is. A read the data does not back is a refusal (`Unbacked`), never a guess. The
//! decoder is the original's integer walk, translated exactly, with its floating-point part (the object's bounding box scaling the short points) in the original's own
//! C (`csrc/model_step.c`) under its round-toward-zero guard.
//!
//! The store is immutable after `ModelStore::load`; every query returns owned plain values, so nothing a caller holds can outlive or alias it.
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The compiled scene names (modules/3d/3dmodels.h, `OBJECT_3D_INDEX`): line n is scene n; scene 0 and `OBJECT_3D_LAST` have none.
const SCENE_NAMES: &str = include_str!("../data/scene-names.txt");

/// `custom_number_of_objects` in 3dobjdb.c: how many objects past the retail table the original has room for.
const CUSTOM_OBJECT_ROOM: i64 = 2000;
/// the largest file the store will read (the retail files are a few megabytes)
const MAX_FILE: u64 = 512 * 1024 * 1024;
/// the sub-object depth the numeric kernel accepts (`MODEL_MAX_DEPTH` in model_step.c)
const MAX_TREE_DEPTH: usize = 64;

extern "C" {
    fn eech_model_triangle(points: *const i16, bounds: *const f32, out: *mut f32) -> i32;
    fn eech_model_scene_bounds(
        main_object: i32,
        object_bounds: *const f32,
        object_count: i32,
        nodes: *const Node,
        node_count: i32,
        top_first: i32,
        top_count: i32,
        out_bounds: *mut f32,
        out_radius: *mut f32,
    ) -> i32;
    fn eech_model_node_size() -> usize;
    fn eech_model_fp_state(x87_masks: *mut i32, sse_masks: *mut i32, sse_rounding: *mut i32, x87_control: *mut u32, mxcsr_bits: *mut u32) -> i32;
}

/// One sub-object as the numeric kernel reads it (keyframe 0 only); the layout is `eech_model_node` in model_step.c.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct Node {
    object: i32,
    collision: i32,
    first_child: i32,
    child_count: i32,
    keyframe: [f32; 9],
}

/// The floating-point state of this thread as the numeric kernel's guard reads it: (x87 exception masks, SSE masks, SSE rounding, raw x87 control word, raw MXCSR).
/// Read-only; the tests use it to show a query hands the caller's state back bit for bit.
pub fn fp_state() -> Option<(i32, i32, i32, u32, u32)> {
    let (mut x87, mut sse, mut rounding, mut control, mut mxcsr) = (0, 0, 0, 0, 0);
    // SAFETY: five valid out-pointers to locals; the function only reads two registers.
    let status = unsafe { eech_model_fp_state(&mut x87, &mut sse, &mut rounding, &mut control, &mut mxcsr) };
    (status == 0).then_some((x87, sse, rounding, control, mxcsr))
}

// ---------------------------------------------------------------------------------------------------------------------------------
// errors

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// an asset the store needs is absent: the run cannot be made (NOT RUN), it is not a failure of the data
    Missing,
    Unreadable,
    Truncated,
    /// a value the original would not accept, or a format it does not know
    Incompatible,
    /// counts that contradict each other or the files they describe
    Inconsistent,
    /// a bound the store keeps (object room, file size, tree depth, floating-point environment)
    Limit,
}

#[derive(Debug, Clone)]
pub struct ModelError {
    pub kind: ErrorKind,
    pub file: String,
    pub detail: String,
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self.kind {
            ErrorKind::Missing => "missing",
            ErrorKind::Unreadable => "unreadable",
            ErrorKind::Truncated => "truncated",
            ErrorKind::Incompatible => "incompatible",
            ErrorKind::Inconsistent => "inconsistent",
            ErrorKind::Limit => "limit",
        };
        write!(f, "{kind} {}: {}", self.file, self.detail)
    }
}

impl std::error::Error for ModelError {}

fn fail<T>(kind: ErrorKind, file: &str, detail: impl Into<String>) -> Result<T, ModelError> {
    Err(ModelError {
        kind,
        file: file.to_string(),
        detail: detail.into(),
    })
}

// ---------------------------------------------------------------------------------------------------------------------------------
// the compiled scene names

fn compiled_names() -> &'static [&'static str] {
    static NAMES: OnceLock<Vec<&'static str>> = OnceLock::new();
    NAMES.get_or_init(|| {
        let mut names = vec![""];
        names.extend(SCENE_NAMES.lines());
        names
    })
}

/// One past the last valid scene: `OBJECT_3D_LAST`.
pub fn scene_last() -> usize {
    compiled_names().len()
}

/// The scene name the original compiles for `scene` (modules/3d/3dmodels.h), or None outside 1 .. `scene_last`.
pub fn compiled_scene_name(scene: usize) -> Option<&'static str> {
    if scene == 0 {
        return None;
    }
    compiled_names().get(scene).copied()
}

/// `OBJECT_3D_OLD_LAST`: how many scene names `bininfo.bin` lists (the scenes before it are the retail ones).
pub fn old_last() -> usize {
    compiled_names().iter().position(|n| *n == "OLD_LAST").unwrap_or(0)
}

/// `get_scene`: the first scene whose compiled name is `name` (already upper case).
fn scene_by_name(name: &[u8]) -> Option<usize> {
    compiled_names().iter().skip(1).position(|n| n.as_bytes() == name).map(|i| i + 1)
}

// ---------------------------------------------------------------------------------------------------------------------------------
// reading: every read is bounds-checked, every count is checked before it sizes anything

struct Reader<'a> {
    file: &'a str,
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(file: &'a str, data: &'a [u8]) -> Self {
        Reader { file, data, pos: 0 }
    }

    fn truncated<T>(&self, wanted: u64) -> Result<T, ModelError> {
        fail(
            ErrorKind::Truncated,
            self.file,
            format!("{wanted} bytes wanted at offset {}, file has {}", self.pos, self.data.len()),
        )
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], ModelError> {
        match self.pos.checked_add(n) {
            Some(end) if end <= self.data.len() => {
                let slice = &self.data[self.pos..end];
                self.pos = end;
                Ok(slice)
            }
            _ => self.truncated(n as u64),
        }
    }

    /// the bytes not yet read
    fn remaining(&self) -> u64 {
        (self.data.len() - self.pos) as u64
    }

    fn skip(&mut self, bytes: u64) -> Result<(), ModelError> {
        let remaining = (self.data.len() - self.pos) as u64;
        if bytes > remaining {
            return self.truncated(bytes);
        }
        self.pos += bytes as usize;
        Ok(())
    }

    fn i32(&mut self) -> Result<i32, ModelError> {
        let b = self.take(4)?;
        Ok(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn u32(&mut self) -> Result<u32, ModelError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn f32(&mut self) -> Result<f32, ModelError> {
        Ok(f32::from_bits(self.u32()?))
    }

    fn floats<const N: usize>(&mut self) -> Result<[f32; N], ModelError> {
        let mut out = [0.0f32; N];
        for value in out.iter_mut() {
            *value = self.f32()?;
        }
        Ok(out)
    }

    /// a count that must be non-negative
    fn count(&mut self, what: &str) -> Result<usize, ModelError> {
        let value = self.i32()?;
        if value < 0 {
            return fail(
                ErrorKind::Incompatible,
                self.file,
                format!("{what} is negative ({value}) at offset {}", self.pos - 4),
            );
        }
        Ok(value as usize)
    }

    /// `n` records of `size` bytes follow: refuses a count the rest of the file cannot hold, before anything is sized from it
    fn records(&mut self, what: &str, size: u64) -> Result<usize, ModelError> {
        let n = self.count(what)?;
        let remaining = (self.data.len() - self.pos) as u64;
        match (n as u64).checked_mul(size) {
            Some(bytes) if bytes <= remaining => Ok(n),
            _ => fail(
                ErrorKind::Truncated,
                self.file,
                format!("{what} {n} records of {size} bytes, {remaining} bytes remain"),
            ),
        }
    }

    /// `get_nul_string`: the bytes up to a NUL, upper-cased; EOF before the NUL, or a string that fills `limit`, is refused (the original leaves the rest of such a string
    /// unread and misparses everything after it)
    fn nul_string(&mut self, limit: usize) -> Result<Vec<u8>, ModelError> {
        let mut out = Vec::new();
        loop {
            let b = self.take(1)?[0];
            if b == 0 {
                return Ok(out);
            }
            out.push(b.to_ascii_uppercase());
            if out.len() >= limit {
                return fail(
                    ErrorKind::Incompatible,
                    self.file,
                    format!("a string fills its {limit}-byte buffer at offset {}", self.pos),
                );
            }
        }
    }
}

/// little-endian 16-bit words and 3 x 16-bit points out of a byte run (the callers have already checked the run holds whole elements)
fn words_of(raw: &[u8]) -> Vec<u16> {
    (0..raw.len() / 2).map(|i| u16::from_le_bytes([raw[2 * i], raw[2 * i + 1]])).collect()
}

fn points_of(raw: &[u8]) -> Vec<[i16; 3]> {
    let at = |i: usize, k: usize| i16::from_le_bytes([raw[6 * i + 2 * k], raw[6 * i + 2 * k + 1]]);
    (0..raw.len() / 6).map(|i| [at(i, 0), at(i, 1), at(i, 2)]).collect()
}

/// What a path is, following links.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Directory,
    File,
    Other,
}

/// The two filesystem questions the resolver and the enumerator ask, as a small boundary so that the failures a test cannot portably cause (an unreadable directory, an entry
/// that cannot be read, a path whose type cannot be determined) can be put in front of the actual resolver code. Production uses `RealFs`.
trait Fs {
    /// the names in `dir`: Err if it cannot be listed (a directory that does not exist is `NotFound`); each entry is its name or the error reading it
    fn list(&self, dir: &Path) -> std::io::Result<Vec<std::io::Result<String>>>;
    /// what `path` is, following links; Ok(None) when it does not exist
    fn kind(&self, path: &Path) -> std::io::Result<Option<Kind>>;
}

struct RealFs;

impl Fs for RealFs {
    fn list(&self, dir: &Path) -> std::io::Result<Vec<std::io::Result<String>>> {
        Ok(std::fs::read_dir(dir)?
            .map(|entry| entry.map(|e| e.file_name().to_string_lossy().to_string()))
            .collect())
    }

    fn kind(&self, path: &Path) -> std::io::Result<Option<Kind>> {
        match std::fs::metadata(path) {
            Ok(meta) if meta.is_dir() => Ok(Some(Kind::Directory)),
            Ok(meta) if meta.is_file() => Ok(Some(Kind::File)),
            Ok(_) => Ok(Some(Kind::Other)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }
}

fn io_failure(path: &Path, what: &str, error: &std::io::Error) -> ModelError {
    ModelError {
        kind: ErrorKind::Unreadable,
        file: path.display().to_string(),
        detail: format!("{what}: {error}"),
    }
}

/// The entries of `dir` that are `want`, with their paths. A directory that cannot be listed, an entry that cannot be read, a path whose type cannot be determined, or an
/// entry that vanished while it was being listed is an error naming the path: nothing is silently left out. (An entry of another kind is not an error: the original's search
/// skips directories when it wants files and files when it wants directories.)
fn typed_entries(fs: &dyn Fs, dir: &Path, want: Kind) -> Result<Vec<(String, PathBuf)>, ModelError> {
    let listing = fs.list(dir).map_err(|e| io_failure(dir, "cannot list the directory", &e))?;
    let mut out = Vec::new();
    for item in listing {
        let name = item.map_err(|e| io_failure(dir, "cannot read a directory entry", &e))?;
        let path = dir.join(&name);
        match fs.kind(&path).map_err(|e| io_failure(&path, "cannot determine what the entry is", &e))? {
            Some(kind) if kind == want => out.push((name, path)),
            Some(_) => {}
            None => {
                return fail(
                    ErrorKind::Inconsistent,
                    &path.display().to_string(),
                    "the entry vanished while the directory was being listed",
                )
            }
        }
    }
    Ok(out)
}

/// `name` in `dir` as a path of kind `want`. An EXACT spelling wins, as it does on a case-sensitive file system and as the original's exact lookup does: if `dir` holds `name`
/// itself it is returned, whatever else differs from it only in case. Only when the exact spelling is absent is the directory searched by upper-case comparison (a Windows file
/// system is case-insensitive and the original's names are not consistently cased), and then two or more matches are ambiguous and an error. Ok(None) only when nothing of that
/// name exists, or `dir` itself does not; a name that exists as another kind, an ambiguous fallback, and every I/O failure are errors naming the path.
fn find_named(fs: &dyn Fs, dir: &Path, name: &str, want: Kind) -> Result<Option<PathBuf>, ModelError> {
    let noun = if want == Kind::Directory { "directory" } else { "file" };
    let exact = dir.join(name);
    match fs.kind(&exact).map_err(|e| io_failure(&exact, "cannot determine what the path is", &e))? {
        Some(kind) if kind == want => return Ok(Some(exact)),
        Some(_) => return fail(ErrorKind::Inconsistent, &exact.display().to_string(), format!("exists but is not a {noun}")),
        None => {}
    }
    let listing = match fs.list(dir) {
        Ok(listing) => listing,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(io_failure(dir, "cannot list the directory", &e)),
    };
    let mut found: Option<PathBuf> = None;
    for item in listing {
        let entry_name = item.map_err(|e| io_failure(dir, "cannot read a directory entry", &e))?;
        if !entry_name.eq_ignore_ascii_case(name) {
            continue;
        }
        let path = dir.join(&entry_name);
        match fs.kind(&path).map_err(|e| io_failure(&path, "cannot determine what the entry is", &e))? {
            Some(kind) if kind == want => {}
            Some(_) => return fail(ErrorKind::Inconsistent, &path.display().to_string(), format!("exists but is not a {noun}")),
            None => {
                return fail(
                    ErrorKind::Inconsistent,
                    &path.display().to_string(),
                    "the entry vanished while the directory was being listed",
                )
            }
        }
        if found.is_some() {
            return fail(ErrorKind::Inconsistent, name, format!("two {noun}s in {} differ only in case", dir.display()));
        }
        found = Some(path);
    }
    Ok(found)
}

fn find_file(fs: &dyn Fs, dir: &Path, name: &str) -> Result<Option<PathBuf>, ModelError> {
    find_named(fs, dir, name, Kind::File)
}

fn read_file(path: &Path, label: &str) -> Result<Vec<u8>, ModelError> {
    let meta = std::fs::metadata(path).map_err(|e| ModelError {
        kind: ErrorKind::Unreadable,
        file: label.to_string(),
        detail: e.to_string(),
    })?;
    if meta.len() > MAX_FILE {
        return fail(ErrorKind::Limit, label, format!("{} bytes exceeds the {MAX_FILE}-byte limit", meta.len()));
    }
    std::fs::read(path).map_err(|e| ModelError {
        kind: ErrorKind::Unreadable,
        file: label.to_string(),
        detail: e.to_string(),
    })
}

fn required(fs: &dyn Fs, dir: &Path, name: &str) -> Result<Vec<u8>, ModelError> {
    match find_file(fs, dir, name)? {
        Some(path) => read_file(&path, name),
        None => fail(ErrorKind::Missing, name, format!("not found under {}", dir.display())),
    }
}

/// the sub-directories (or files) of `dir`, sorted by upper-case name: the order the Windows directory search returns them in, which the original's numbering of custom
/// objects and its last-definition-wins rule depend on. Any failure to enumerate is an error (see `typed_entries`).
fn sorted_entries(fs: &dyn Fs, dir: &Path, directories: bool) -> Result<Vec<(String, PathBuf)>, ModelError> {
    let mut out = typed_entries(fs, dir, if directories { Kind::Directory } else { Kind::File })?;
    out.sort_by(|a, b| a.0.to_ascii_uppercase().cmp(&b.0.to_ascii_uppercase()).then(a.0.cmp(&b.0)));
    Ok(out)
}

// ---------------------------------------------------------------------------------------------------------------------------------
// the model data

/// A run of an array: `block` (0 is the common block read from the files, every custom object adds its own) and the start in it.
#[derive(Clone, Copy, Debug)]
struct Span {
    block: u32,
    start: usize,
}

/// Arrays of one kind. A read past the BACKED length of the block is `None`, whatever the object's own counts say.
#[derive(Default)]
struct Arena<T> {
    blocks: Vec<Vec<T>>,
}

impl<T: Copy> Arena<T> {
    fn add(&mut self, block: Vec<T>) -> u32 {
        self.blocks.push(block);
        (self.blocks.len() - 1) as u32
    }

    fn get(&self, span: Span, index: usize) -> Option<T> {
        let block = self.blocks.get(span.block as usize)?;
        block.get(span.start.checked_add(index)?).copied()
    }
}

/// One surface description as the decoder reads it: the flags word (polygon bit 0, point count in byte 3) and the face count.
#[derive(Clone, Copy, Debug, Default)]
struct Surface {
    flags: u32,
    faces: u16,
}

impl Surface {
    fn polygons(&self) -> bool {
        self.flags & 1 != 0
    }
    fn smoothed(&self) -> bool {
        self.flags & 4 != 0
    }
    /// `number_of_points`, where 0 means 256
    fn points(&self) -> i64 {
        let n = ((self.flags >> 24) & 0xff) as i64;
        if n == 0 {
            256
        } else {
            n
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectSource {
    /// `3dobjs.bin`
    Table,
    /// `OBJECTS/XXXX.EEO` replacing a table object
    HexOverride,
    /// a named `.EEO` an `.EES` scene loaded
    Named,
}

#[derive(Clone, Debug)]
struct Object {
    present: bool,
    source: ObjectSource,
    number_of_points: u16,
    number_of_faces: u16,
    number_of_surfaces: u16,
    radius: f32,
    bounding_box: [f32; 6],
    bounding_box2: [f32; 6],
    points: Option<Span>,
    faces: Option<Span>,
    point_list: Option<Span>,
    surface_points: Option<Span>,
    /// where this object's surfaces start in the consolidated surface block
    surfaces_start: usize,
    /// an `.EEO` the original would have rejected after reading it (a face of fewer than 3 points, a smoothed non-polygon surface): decoding refuses it
    invalid: Option<&'static str>,
}

impl Object {
    fn null() -> Object {
        Object {
            present: false,
            source: ObjectSource::Table,
            number_of_points: 0,
            number_of_faces: 0,
            number_of_surfaces: 0,
            radius: 0.0,
            bounding_box: [0.0; 6],
            bounding_box2: [0.0; 6],
            points: None,
            faces: None,
            point_list: None,
            surface_points: None,
            surfaces_start: 0,
            invalid: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneSource {
    /// `3dobjdb.bin`
    Table,
    /// an `.EES` file replaced it
    Ees,
}

#[derive(Clone, Debug)]
struct Scene {
    loaded: bool,
    source: SceneSource,
    /// the scene's main object
    index: i32,
    /// the effective collision object index: -1 for none (the binary 0 maps to -1, as the accepted C1 patch does and as `.EES` scenes always did)
    collision: i32,
    bounds: [f32; 12],
    radius: f32,
}

impl Scene {
    fn empty() -> Scene {
        Scene {
            loaded: false,
            source: SceneSource::Table,
            index: 0,
            collision: 0,
            bounds: [0.0; 12],
            radius: 0.0,
        }
    }
}

/// What the load found, for the report; none of it is a campaign decision.
#[derive(Clone, Debug, Default)]
pub struct LoadSummary {
    pub scene_last: usize,
    pub table_objects: usize,
    pub objects_total: usize,
    pub object_room: usize,
    pub scenes_in_table: usize,
    pub scenes_loaded: usize,
    pub ees_applied: usize,
    pub ees_unsupported_version: usize,
    pub ees_unresolved_names: Vec<String>,
    pub ees_duplicate_scenes: Vec<String>,
    pub eeo_hex_applied: usize,
    pub eeo_hex_unsupported_version: usize,
    pub eeo_invalid: usize,
    pub named_objects: usize,
    pub oversize_counts: usize,
    pub surface_block: usize,
    pub points_block: usize,
    pub surface_points_block: usize,
    pub points_trailing_bytes: usize,
    pub surface_points_trailing_bytes: usize,
}

pub struct ModelStore {
    scenes: Vec<Scene>,
    objects: Vec<Object>,
    surfaces: Vec<Surface>,
    points: Arena<[i16; 3]>,
    faces: Arena<u8>,
    point_list: Arena<u8>,
    surface_points: Arena<u16>,
    summary: LoadSummary,
}

// ---------------------------------------------------------------------------------------------------------------------------------
// the loader

struct Loader {
    objects: Vec<Object>,
    object_bounds: Vec<f32>,
    own_surfaces: HashMap<usize, Vec<Surface>>,
    scenes: Vec<Scene>,
    points: Arena<[i16; 3]>,
    faces: Arena<u8>,
    point_list: Arena<u8>,
    surface_points: Arena<u16>,
    table_objects: usize,
    raw_objects: usize,
    next_object: usize,
    room: i64,
    history: Vec<(Vec<u8>, usize)>,
    summary: LoadSummary,
}

/// The fewest bytes one serialized sub-object occupies, from `initialise_3d_sub_object` in the original's order: the object (a binary int, or for a named scene
/// `skip_schem`, 12 bytes from version -3, and a NUL-terminated name, at least the NUL), four ints (contributes to collisions, the two approximation levels, the flag with
/// its limits absent), the keyframes (a count, and at least one 56-byte keyframe: the original asserts one), the dissolve float, and three counts (value keyframes, named
/// indices, sub-objects). Derived from the reads, not chosen: a real record is never smaller, so a count the remaining bytes cannot hold at this size is not a scene.
fn min_sub_object_bytes(named: Option<&Named>) -> u64 {
    let object = match named {
        None => 4,
        Some(named) if named.version <= -3 => 12 + 1,
        Some(_) => 1,
    };
    object + 4 * 4 + (4 + 56) + 4 + 3 * 4
}

/// The sub-object nodes of one scene and the room its header declares for them (the original allocates exactly that and overruns it if the records say more).
struct Tree {
    nodes: Vec<Node>,
    node_budget: usize,
    index_budget: usize,
    /// nodes reserved (their room taken) whose records have not been started: each still has to be read, from the bytes that remain, after the one being read now
    pending: u64,
}

/// What a named (`.EES`) scene needs to resolve names the way the original does.
struct Named<'a> {
    version: i32,
    prefix: &'a Path,
}

impl Loader {
    fn set_object(&mut self, index: usize, object: Object) {
        self.object_bounds[index * 12..index * 12 + 6].copy_from_slice(&object.bounding_box);
        self.object_bounds[index * 12 + 6..index * 12 + 12].copy_from_slice(&object.bounding_box2);
        self.objects[index] = object;
    }

    // ---- 3dobjs.bin ----

    fn read_table(&mut self, bin: &[u8], pts: &[u8], sp: &[u8]) -> Result<(), ModelError> {
        const FILE: &str = "3dobjs.bin";
        let mut r = Reader::new(FILE, bin);
        let total_objects = r.count("total_number_of_objects")?;
        let _total_surfaces = r.count("total_number_of_surfaces")?;
        let _total_points = r.count("total_number_of_points")?;
        let _total_point_normals = r.count("total_number_of_point_normals")?;
        let total_plain_points = r.count("total_number_of_plain_points")?;
        let _total_face_normal_points = r.count("total_number_of_face_normal_points")?;
        let total_polygons = r.count("total_number_of_polygons")?;
        if total_objects == 0 || total_objects > 60000 {
            return fail(ErrorKind::Incompatible, FILE, format!("{total_objects} objects"));
        }
        self.table_objects = total_objects;
        self.raw_objects = total_objects + CUSTOM_OBJECT_ROOM as usize;
        // get_next_object_id returns total_number_of_objects++ from its initial value N, the table's object COUNT: the first custom object is numbered N and REPLACES the
        // last table object, and the counter then stands one past the last number given out
        self.next_object = total_objects;
        self.room = CUSTOM_OBJECT_ROOM;
        self.objects = vec![Object::null(); self.raw_objects + 1];
        self.object_bounds = vec![0.0; (self.raw_objects + 1) * 12];

        let points_len = pts.len() / 6;
        let sp_len = sp.len() / 2;
        self.summary.points_trailing_bytes = pts.len() % 6;
        self.summary.surface_points_trailing_bytes = sp.len() % 2;
        self.summary.points_block = points_len;
        self.summary.surface_points_block = sp_len;
        let block_points = points_of(pts);
        let block_sp = words_of(sp);
        self.points.add(block_points);
        self.surface_points.add(block_sp);

        let mut faces_block: Vec<u8> = Vec::new();
        let mut list_block: Vec<u8> = Vec::new();
        let mut points_cursor = 0usize;
        let mut sp_cursor = 0usize;

        for index in 1..=total_objects {
            let pretend_null = r.i32()?;
            if pretend_null != 0 {
                continue;
            }
            let number_of_points = r.i32()?;
            let number_of_surfaces = r.i32()?;
            let number_of_faces = r.i32()?;
            let number_of_point_normals = r.i32()?;
            let _number_of_lighting_normals = r.i32()?;
            let _culling_normals_offset = r.i32()?;
            // the object's counts are unsigned shorts in the original: a larger file value is truncated there, and the cursors below advance by the truncated value
            for value in [number_of_points, number_of_surfaces, number_of_faces, number_of_point_normals] {
                if !(0..=0xffff).contains(&value) {
                    self.summary.oversize_counts += 1;
                }
            }
            let number_of_points = (number_of_points as u32 & 0xffff) as u16;
            let number_of_surfaces = (number_of_surfaces as u32 & 0xffff) as u16;
            let number_of_faces = (number_of_faces as u32 & 0xffff) as u16;
            let number_of_point_normals = (number_of_point_normals as u32 & 0xffff) as usize;
            let radius = r.f32()?;
            let point_references = r.count("number_of_point_references")?;
            let _normals_local = r.count("number_of_point_normals (local)")?;
            let _texture_points = r.count("number_of_point_textures")?;
            let bounding_box = r.floats::<6>()?;
            let bounding_box2 = r.floats::<6>()?;

            let mut real_polygons = 0usize;
            let mut surfaces = Vec::new();
            for _ in 0..number_of_surfaces {
                let flags = r.u32()?;
                let _texture_index = r.take(2)?;
                let _luminosity_texture_index = r.take(2)?;
                // fread (&number_of_faces, sizeof (int)) into the 16-bit field: the low half stays
                let faces_word = r.u32()?;
                let _colour = r.u32()?;
                let _reflectivity_specularity = r.take(2)?;
                let surface = Surface {
                    flags,
                    faces: (faces_word & 0xffff) as u16,
                };
                if surface.polygons() {
                    real_polygons += surface.faces as usize;
                }
                surfaces.push(surface);
            }

            let points_span = if number_of_points != 0 {
                let span = Span {
                    block: 0,
                    start: points_cursor,
                };
                points_cursor += number_of_points as usize;
                Some(span)
            } else {
                None
            };
            r.skip(2 * number_of_point_normals as u64)?;
            let point_reference_count = r.count("point_reference_count")?;
            let face_normal_reference_count = r.count("face_normal_reference_count")?;

            let faces_span = if real_polygons != 0 {
                let bytes = r.take(real_polygons)?;
                let span = Span {
                    block: 0,
                    start: faces_block.len(),
                };
                faces_block.extend_from_slice(bytes);
                Some(span)
            } else {
                None
            };
            let list_span = if point_reference_count != 0 {
                let bytes = r.take(point_reference_count)?;
                let span = Span {
                    block: 0,
                    start: list_block.len(),
                };
                list_block.extend_from_slice(bytes);
                Some(span)
            } else {
                None
            };
            r.skip(2 * face_normal_reference_count as u64)?;

            // surface_points: the pointer is always set, then advanced by the object's declared references
            let sp_span = Span { block: 0, start: sp_cursor };
            sp_cursor = match sp_cursor.checked_add(point_references) {
                Some(v) => v,
                None => return fail(ErrorKind::Inconsistent, FILE, "surface point references overflow"),
            };

            self.set_object(
                index,
                Object {
                    present: true,
                    source: ObjectSource::Table,
                    number_of_points,
                    number_of_faces,
                    number_of_surfaces,
                    radius,
                    bounding_box,
                    bounding_box2,
                    points: points_span,
                    faces: faces_span,
                    point_list: list_span,
                    surface_points: Some(sp_span),
                    surfaces_start: 0,
                    invalid: None,
                },
            );
            self.own_surfaces.insert(index, surfaces);
        }

        // the declared totals must cover what the objects used, and the mapped files must cover the declared cursors (the original would write or read past them)
        if faces_block.len() > total_polygons {
            return fail(
                ErrorKind::Inconsistent,
                FILE,
                format!("the objects hold {} polygon faces, the header declares {total_polygons}", faces_block.len()),
            );
        }
        if list_block.len() > total_plain_points {
            return fail(
                ErrorKind::Inconsistent,
                FILE,
                format!("the objects hold {} plain points, the header declares {total_plain_points}", list_block.len()),
            );
        }
        if points_cursor > points_len {
            return fail(
                ErrorKind::Inconsistent,
                "3dobjs.pts",
                format!("the objects use {points_cursor} points, the file holds {points_len}"),
            );
        }
        if sp_cursor > sp_len {
            return fail(
                ErrorKind::Inconsistent,
                "3dobjs.sp",
                format!("the objects use {sp_cursor} surface point references, the file holds {sp_len}"),
            );
        }
        self.faces.add(faces_block);
        self.point_list.add(list_block);
        Ok(())
    }

    // ---- 3dobjs EEO objects ----

    /// `read_object`. Ok(None): the original returns FALSE before touching the object (unsupported version). The object comes back with its own arrays.
    fn read_eeo(&mut self, bytes: &[u8], file: &str, source: ObjectSource) -> Result<Option<(Object, Vec<Surface>)>, ModelError> {
        let mut r = Reader::new(file, bytes);
        let version = r.i32()?;
        if !(1..=2).contains(&version) {
            return Ok(None);
        }
        let n_points = r.count("points")?;
        let n_faces = r.count("faces")?;
        let n_polygoned = r.count("polygoned faces")?;
        let n_face_normals = if version > 1 { r.count("face normals")? } else { n_polygoned };
        let n_surfaces = r.count("surfaces")?;
        let n_point_normals = r.count("point normals")?;
        let n_lighting_normals = r.count("lighting normals")?;
        let n_point_refs = r.count("point references")?;
        let n_texture_points = r.count("texture points")?;
        let n_surface_point_refs = r.count("surface point references")?;
        let n_surface_point_normals = r.count("surface point normals")?;
        let _ = n_lighting_normals;
        for (name, value) in [
            ("points", n_points),
            ("faces", n_faces),
            ("surfaces", n_surfaces),
            ("point normals", n_point_normals),
        ] {
            if value > 0xffff {
                return fail(
                    ErrorKind::Incompatible,
                    file,
                    format!("{name} count {value} does not fit the original's 16-bit field"),
                );
            }
        }
        let bounding_box = r.floats::<6>()?;
        let bounding_box2 = r.floats::<6>()?;
        let radius = r.f32()?;

        let mut points = Vec::new();
        if n_points != 0 {
            let raw = r.take(n_points * 6)?;
            points = points_of(raw);
        }
        let faces_raw: Vec<u8> = if n_polygoned != 0 { r.take(n_polygoned)?.to_vec() } else { Vec::new() };
        r.skip(2 * n_point_normals as u64)?;
        let list_raw: Vec<u8> = if n_point_refs != 0 { r.take(n_point_refs)?.to_vec() } else { Vec::new() };
        let mut surfaces = Vec::new();
        if n_surfaces != 0 {
            for _ in 0..n_surfaces {
                let record = r.take(16)?;
                let flags = u32::from_le_bytes([record[0], record[1], record[2], record[3]]);
                let faces = u16::from_le_bytes([record[8], record[9]]);
                surfaces.push(Surface { flags, faces });
            }
        }
        r.skip(2 * n_face_normals as u64)?;
        r.skip(8 * n_texture_points as u64)?;
        let sp_raw: Vec<u16> = if n_surface_point_refs != 0 {
            words_of(r.take(n_surface_point_refs * 2)?)
        } else {
            Vec::new()
        };
        r.skip(2 * n_surface_point_normals as u64)?;
        // (the texture-name overrides that follow are not physical and are not read)

        let mut invalid = None;
        if faces_raw.iter().any(|&n| n < 3) {
            invalid = Some("a polygon face of fewer than 3 points");
        } else if surfaces.iter().any(|s| !s.polygons() && s.smoothed()) {
            invalid = Some("a smoothed non-polygon surface");
        }
        let object = Object {
            present: true,
            source,
            number_of_points: n_points as u16,
            number_of_faces: n_faces as u16,
            number_of_surfaces: n_surfaces as u16,
            radius,
            bounding_box,
            bounding_box2,
            points: if n_points != 0 {
                Some(Span {
                    block: self.points.add(points),
                    start: 0,
                })
            } else {
                None
            },
            faces: if n_polygoned != 0 {
                Some(Span {
                    block: self.faces.add(faces_raw),
                    start: 0,
                })
            } else {
                None
            },
            point_list: if n_point_refs != 0 {
                Some(Span {
                    block: self.point_list.add(list_raw),
                    start: 0,
                })
            } else {
                None
            },
            // always set in the original (a pointer to the exact allocation, or none when nothing was read)
            surface_points: if n_surface_point_refs != 0 {
                Some(Span {
                    block: self.surface_points.add(sp_raw),
                    start: 0,
                })
            } else {
                None
            },
            surfaces_start: 0,
            invalid,
        };
        Ok(Some((object, surfaces)))
    }

    // ---- 3dobjdb.bin / .EES scenes ----

    /// `get_object` for a named scene: the object a name resolves to, loading its `.EEO` from `prefix` the first time.
    fn named_object(&mut self, r: &mut Reader, named: &Named) -> Result<i32, ModelError> {
        let file = r.file.to_string();
        let full = r.nul_string(260)?;
        if full.is_empty() {
            return Ok(0);
        }
        let start = full
            .iter()
            .rposition(|&b| b == b'/')
            .or_else(|| full.iter().rposition(|&b| b == 0x5c))
            .or_else(|| full.iter().rposition(|&b| b == b':'))
            .map(|p| p + 1)
            .unwrap_or(0);
        let base = &full[start..];
        // XXXX.LWO with four hex digits: a retail table object by number
        if base.len() == 8 && base[..4].iter().all(|b| b.is_ascii_hexdigit()) && &base[4..] == b".LWO" {
            let text = std::str::from_utf8(&base[..4]).unwrap_or("0");
            let id = i64::from_str_radix(text, 16).unwrap_or(0);
            if id > 0 && id < self.raw_objects as i64 {
                return Ok(id as i32);
            }
        }
        let stem: Vec<u8> = match base.iter().rposition(|&b| b == b'.') {
            Some(dot) => base[..dot].to_vec(),
            None => base.to_vec(),
        };
        if let Some((_, id)) = self.history.iter().find(|(name, _)| *name == stem) {
            return Ok(*id as i32);
        }
        if self.room <= 0 {
            return fail(ErrorKind::Limit, &file, "too many custom objects for the room the original has");
        }
        self.room -= 1;
        let id = self.next_object;
        self.next_object += 1;
        if self.history.len() < 1000 {
            self.history.push((stem.clone(), id));
        }
        let stem_text = String::from_utf8_lossy(&stem).to_string();
        let label = format!("{stem_text}.EEO");
        let path = match find_file(&RealFs, named.prefix, &label)? {
            Some(path) => path,
            None => {
                return fail(
                    ErrorKind::Inconsistent,
                    &label,
                    format!("named by {file}, not found under {}", named.prefix.display()),
                )
            }
        };
        let bytes = read_file(&path, &label)?;
        match self.read_eeo(&bytes, &label, ObjectSource::Named)? {
            Some((object, surfaces)) => {
                if let Some(why) = object.invalid {
                    return fail(ErrorKind::Incompatible, &label, format!("the original rejects this object: {why}"));
                }
                self.own_surfaces.insert(id, surfaces);
                self.set_object(id, object);
                self.summary.named_objects += 1;
            }
            None => return fail(ErrorKind::Incompatible, &label, "an object version the original does not read"),
        }
        Ok(id as i32)
    }

    fn skip_schem(r: &mut Reader, named: Option<&Named>) -> Result<(), ModelError> {
        if let Some(named) = named {
            if named.version <= -3 {
                r.skip(4 + 4 * 2)?;
            }
        }
        Ok(())
    }

    /// `read_keyframes` with its count read: the count, and keyframe 0's nine placement values (x y z heading pitch roll scale_x scale_y scale_z) out of the
    /// 14 words of each keyframe. The whole run is size-checked before anything is skipped.
    fn keyframes(r: &mut Reader) -> Result<(usize, [f32; 9]), ModelError> {
        let n = r.records("keyframes", 56)?;
        if n == 0 {
            return Ok((0, [0.0; 9]));
        }
        r.skip(8)?; // index, linear
        let first = r.floats::<9>()?;
        r.skip(12)?; // tension, continuity, bias
        r.skip((n as u64 - 1) * 56)?;
        Ok((n, first))
    }

    fn value_keyframes(r: &mut Reader) -> Result<(), ModelError> {
        let n = r.records("value keyframes", 24)?;
        r.skip(n as u64 * 24)
    }

    fn indices(r: &mut Reader, named: Option<&Named>, tree: &mut Tree) -> Result<(), ModelError> {
        let n = r.count("sub-object indices")?;
        if n == 0 {
            return Ok(());
        }
        if n > tree.index_budget {
            return fail(
                ErrorKind::Inconsistent,
                r.file,
                format!("{n} sub-object indices exceed the {} the scene declares", tree.index_budget),
            );
        }
        tree.index_budget -= n;
        for _ in 0..n {
            if named.is_some() {
                // get_subobject: a name (the index it resolves to is not physical)
                r.nul_string(1024)?;
            } else {
                r.i32()?;
            }
            r.i32()?;
        }
        Ok(())
    }

    /// `read_subobjects`: the count, then room for that many consecutive nodes (the children of one node are consecutive in the original's array and in the kernel's),
    /// then each is read in turn, its own children's room taken when its turn comes.
    fn sub_objects(&mut self, r: &mut Reader, named: Option<&Named>, tree: &mut Tree, depth: usize) -> Result<(i32, i32), ModelError> {
        let n = r.count("sub-objects")?;
        if n == 0 {
            return Ok((0, 0));
        }
        if depth >= MAX_TREE_DEPTH {
            return fail(ErrorKind::Limit, r.file, format!("a sub-object tree deeper than {MAX_TREE_DEPTH}"));
        }
        // Before anything is sized from the file's count: the bytes that remain must be able to hold that many records AND every record already reserved and not yet read
        // (the siblings of this list's parents, each still to come after the one being read), whatever the scene header declared. One shared byte budget: the same padding
        // cannot justify a reservation at every level, so the nodes held at any moment never exceed the minimum records the unread bytes could contain (a tiny file that
        // says INT_MAX nodes is refused, and so is a deep chain that reuses one stretch of padding for each level's siblings).
        let least = min_sub_object_bytes(named);
        let outstanding = tree.pending + n as u64;
        match outstanding.checked_mul(least) {
            Some(bytes) if bytes <= r.remaining() => {}
            _ => {
                return fail(
                    ErrorKind::Truncated,
                    r.file,
                    format!(
                        "{n} sub-objects of at least {least} bytes each ({} reserved siblings still pending), {} bytes remain",
                        tree.pending,
                        r.remaining()
                    ),
                )
            }
        }
        if n > tree.node_budget {
            return fail(
                ErrorKind::Inconsistent,
                r.file,
                format!("{n} sub-objects exceed the {} the scene declares", tree.node_budget),
            );
        }
        let first = tree.nodes.len();
        // the kernel takes 32-bit counts and indices
        let total = match first.checked_add(n) {
            Some(total) if total <= i32::MAX as usize => total,
            _ => {
                return fail(
                    ErrorKind::Limit,
                    r.file,
                    format!("{first} + {n} sub-object nodes are more than a scene can hold"),
                )
            }
        };
        if tree.nodes.try_reserve(n).is_err() {
            return fail(ErrorKind::Limit, r.file, format!("cannot reserve room for {n} sub-object nodes"));
        }
        tree.node_budget -= n;
        tree.pending += n as u64;
        tree.nodes.resize(total, Node::default());
        for k in 0..n {
            self.sub_object(r, named, tree, first + k, depth)?;
        }
        Ok((first as i32, n as i32))
    }

    /// `initialise_3d_sub_object`
    fn sub_object(&mut self, r: &mut Reader, named: Option<&Named>, tree: &mut Tree, at: usize, depth: usize) -> Result<(), ModelError> {
        // this node's record starts now: it is no longer waiting, and its bytes are being consumed
        tree.pending = tree.pending.saturating_sub(1);
        let index = match named {
            Some(named) => {
                Self::skip_schem(r, Some(named))?;
                self.named_object(r, named)?
            }
            None => r.i32()?,
        };
        let index = if index < 0 { 0 } else { index };
        let contributes = r.i32()?;
        let _in_level = r.i32()?;
        let _out_level = r.i32()?;
        let flag = r.i32()?;
        if flag != 0 {
            r.skip(6 * 4)?;
        }
        let (count, keyframe) = Self::keyframes(r)?;
        if count == 0 {
            return fail(ErrorKind::Incompatible, r.file, "a sub-object without a keyframe (the original asserts one)");
        }
        let _dissolve = r.f32()?;
        Self::value_keyframes(r)?;
        Self::indices(r, named, tree)?;
        let (first_child, child_count) = self.sub_objects(r, named, tree, depth + 1)?;
        tree.nodes[at] = Node {
            object: index,
            collision: contributes,
            first_child,
            child_count,
            keyframe,
        };
        Ok(())
    }

    /// `read_scene`. `named` is Some for an `.EES` file (`sceneid` set in the original), None for a record of `3dobjdb.bin`.
    fn read_scene(&mut self, r: &mut Reader, named: Option<(usize, Named)>) -> Result<usize, ModelError> {
        let file = r.file.to_string();
        let named_ref = named.as_ref().map(|(_, n)| n);
        let scene_index: i64 = match &named {
            Some((id, _)) => *id as i64,
            None => {
                for _ in 0..16 {
                    if r.i32()? != -1 {
                        return fail(ErrorKind::Incompatible, &file, "3d objects scene definitions file is corrupted");
                    }
                }
                r.i32()? as i64
            }
        };
        if scene_index < 1 || scene_index >= scene_last() as i64 {
            return fail(
                ErrorKind::Incompatible,
                &file,
                format!("scene index {scene_index} is out of range (1 to {})", scene_last() - 1),
            );
        }
        let scene_index = scene_index as usize;
        if named.is_none() {
            r.i32()?; // self_shadows
        }
        // lights
        let lights = if named.is_none() { r.records("scene lights", 36)? } else { 0 };
        r.skip(lights as u64 * 36)?;
        // cameras
        let cameras = r.count("scene cameras")?;
        for _ in 0..cameras {
            if let Some(named) = named_ref {
                Self::skip_schem(r, Some(named))?;
                r.nul_string(1024)?; // the camera name
                r.nul_string(1024)?; // the camera's target sub-object name
                r.skip(3 * 4)?;
                Self::keyframes(r)?;
            } else {
                r.skip(8)?;
            }
        }
        // scene links
        let links = r.count("scene links")?;
        for _ in 0..links {
            if let Some(named) = named_ref {
                Self::skip_schem(r, Some(named))?;
                let linked = r.nul_string(1024)?;
                // the original logs a link it cannot resolve and goes on: the link is not physical
                let _ = scene_by_name(&linked);
            } else {
                r.i32()?;
            }
            r.skip(6 * 4)?;
        }
        // sprite lights
        let sprites = r.count("sprite lights")?;
        for _ in 0..sprites {
            Self::skip_schem(r, named_ref)?;
            r.skip(9 * 4)?;
        }
        let version = named_ref.map(|n| n.version).unwrap_or(0);
        // ambient and distant lights (versions -2 and below)
        for per_light in [3u64 * 4, 6 * 4] {
            let n = if version <= -2 { r.count("lights")? } else { 0 };
            for _ in 0..n {
                Self::skip_schem(r, named_ref)?;
                r.skip(per_light)?;
                r.nul_string(1024)?; // get_light
            }
        }
        let scene_sub_objects = r.count("scene sub-objects")?;
        let scene_named_indices = r.count("named sub-objects")?;
        let animations = r.count("texture animations")?;
        if named.is_some() && animations != 0 {
            return fail(
                ErrorKind::Incompatible,
                &file,
                "a named scene with texture animations (the original asserts none)",
            );
        }
        r.skip(animations as u64 * 4)?;
        Self::skip_schem(r, named_ref)?;
        let approximations = r.count("approximations")?;
        let main_object: i32 = match named_ref {
            Some(named) => self.named_object(r, named)?,
            None => r.i32()?,
        };
        if main_object < 0 || main_object as i64 > self.raw_objects as i64 {
            return fail(
                ErrorKind::Incompatible,
                &file,
                format!("scene {scene_index}: main object {main_object} is outside the object table"),
            );
        }
        for _ in 0..approximations {
            match named_ref {
                Some(named) => {
                    self.named_object(r, named)?;
                }
                None => {
                    r.i32()?;
                }
            }
            r.f32()?;
        }
        Self::skip_schem(r, named_ref)?;
        r.i32()?; // shadow_approximation_index
        match named_ref {
            Some(named) => {
                self.named_object(r, named)?;
            }
            None => {
                r.i32()?;
            }
        }
        r.skip(3 * 4)?; // shadow_polygon_object_scale
        let collision: i32 = match named_ref {
            Some(named) => {
                Self::skip_schem(r, Some(named))?;
                let object = self.named_object(r, named)?;
                if object == 0 {
                    -1
                } else {
                    object
                }
            }
            None => {
                // the accepted C1 patch: a collision object index of 0 in the binary database means none
                let object = r.i32()?;
                if object == 0 {
                    -1
                } else {
                    object
                }
            }
        };
        // the main object's keyframes (asserted non-empty), dissolve, displacement
        let (count, _) = Self::keyframes(r)?;
        if count == 0 {
            return fail(
                ErrorKind::Incompatible,
                &file,
                format!("scene {scene_index}: the main object has no keyframe (the original asserts one)"),
            );
        }
        r.f32()?;
        Self::value_keyframes(r)?;
        if named.is_none() {
            r.i32()?; // displacement_sequence_index
        }
        Self::value_keyframes(r)?;
        let mut tree = Tree {
            nodes: Vec::new(),
            node_budget: scene_sub_objects,
            index_budget: scene_named_indices,
            pending: 0,
        };
        Self::indices(r, named_ref, &mut tree)?;
        let (top_first, top_count) = self.sub_objects(r, named_ref, &mut tree, 0)?;
        let nodes = tree.nodes;

        // the scene's bounds and radius (3dsubobj.c), from the objects as they stand now
        let mut out_bounds = [0.0f32; 12];
        let mut out_radius = 0.0f32;
        // SAFETY: the pointers are to live slices/locals of exactly the lengths passed; the kernel reads them and writes the two out values.
        let code = unsafe {
            eech_model_scene_bounds(
                main_object,
                self.object_bounds.as_ptr(),
                (self.raw_objects + 1) as i32,
                nodes.as_ptr(),
                nodes.len() as i32,
                top_first,
                top_count,
                out_bounds.as_mut_ptr(),
                &mut out_radius,
            )
        };
        match code {
            0 => {}
            1 => {
                return fail(
                    ErrorKind::Limit,
                    &file,
                    "the floating-point exception masks are not the original's (all masked); the model store will not load",
                )
            }
            2 => return fail(ErrorKind::Limit, &file, "the floating-point environment could not be set and restored"),
            _ => {
                return fail(
                    ErrorKind::Inconsistent,
                    &file,
                    format!("scene {scene_index}: a sub-object names an object outside the table, or the tree is malformed"),
                )
            }
        }
        self.scenes[scene_index] = Scene {
            loaded: true,
            source: if named.is_some() { SceneSource::Ees } else { SceneSource::Table },
            index: main_object,
            collision,
            bounds: out_bounds,
            radius: out_radius,
        };
        Ok(scene_index)
    }

    // ---- everything ----

    fn load(root: &Path) -> Result<ModelStore, ModelError> {
        let data = root.join("cohokum").join("3ddata");
        match RealFs.kind(&data).map_err(|e| io_failure(&data, "cannot determine what the path is", &e))? {
            Some(Kind::Directory) => {}
            _ => return fail(ErrorKind::Missing, "3ddata", format!("{} is not a directory", data.display())),
        }
        let bininfo = required(&RealFs, &data, "bininfo.bin")?;
        let bin = required(&RealFs, &data, "3dobjs.bin")?;
        let pts = required(&RealFs, &data, "3dobjs.pts")?;
        let sp = required(&RealFs, &data, "3dobjs.sp")?;
        let db = required(&RealFs, &data, "3dobjdb.bin")?;

        let mut loader = Loader {
            objects: Vec::new(),
            object_bounds: Vec::new(),
            own_surfaces: HashMap::new(),
            scenes: vec![Scene::empty(); scene_last()],
            points: Arena::default(),
            faces: Arena::default(),
            point_list: Arena::default(),
            surface_points: Arena::default(),
            table_objects: 0,
            raw_objects: 0,
            next_object: 0,
            room: 0,
            history: Vec::new(),
            summary: LoadSummary {
                scene_last: scene_last(),
                ..LoadSummary::default()
            },
        };

        // names: the file must agree with the compiled list, as the original asserts
        {
            let mut r = Reader::new("bininfo.bin", &bininfo);
            let count = r.count("scene name count")?;
            if count != old_last() {
                return fail(
                    ErrorKind::Incompatible,
                    "bininfo.bin",
                    format!("{count} scene names, the compiled list has {} before OLD_LAST", old_last()),
                );
            }
            r.i32()?;
            for scene in 1..count {
                let length = r.count("name length")?;
                if length == 0 || length > 4096 {
                    return fail(ErrorKind::Incompatible, "bininfo.bin", format!("name {scene} has length {length}"));
                }
                let raw = r.take(length)?;
                let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
                if raw[..end] != *compiled_names()[scene].as_bytes() {
                    return fail(
                        ErrorKind::Incompatible,
                        "bininfo.bin",
                        format!(
                            "scene {scene} is named {:?}, the compiled list has {:?}",
                            String::from_utf8_lossy(&raw[..end]),
                            compiled_names()[scene]
                        ),
                    );
                }
                r.f32()?;
            }
        }

        loader.read_table(&bin, &pts, &sp)?;

        // binary-hex object overrides
        let objects_dir = find_named(&RealFs, &data, "OBJECTS", Kind::Directory)?;
        if let Some(objects_dir) = &objects_dir {
            let mut seen: BTreeMap<usize, String> = BTreeMap::new();
            for (name, path) in sorted_entries(&RealFs, objects_dir, false)? {
                let upper = name.to_ascii_uppercase();
                let b = upper.as_bytes();
                if b.len() != 8 || !b[..4].iter().all(|c| c.is_ascii_hexdigit()) || &b[4..] != b".EEO" {
                    continue;
                }
                let id = usize::from_str_radix(&upper[..4], 16).unwrap_or(0);
                if id == 0 || id > loader.table_objects {
                    continue;
                }
                if let Some(previous) = seen.insert(id, name.clone()) {
                    return fail(ErrorKind::Inconsistent, &name, format!("object {id} is overridden by {previous} and {name}"));
                }
                let bytes = read_file(&path, &name)?;
                match loader.read_eeo(&bytes, &name, ObjectSource::HexOverride)? {
                    Some((mut object, surfaces)) => {
                        if object.invalid.is_some() {
                            loader.summary.eeo_invalid += 1;
                        }
                        object.source = ObjectSource::HexOverride;
                        loader.own_surfaces.insert(id, surfaces);
                        loader.set_object(id, object);
                        loader.summary.eeo_hex_applied += 1;
                    }
                    None => loader.summary.eeo_hex_unsupported_version += 1,
                }
            }
        }

        // the scene table
        {
            let mut r = Reader::new("3dobjdb.bin", &db);
            let total_scenes = r.count("total_number_of_scenes")?;
            if total_scenes == 0 {
                return fail(ErrorKind::Incompatible, "3dobjdb.bin", "no scenes");
            }
            r.skip(9 * 4)?;
            for _ in 0..total_scenes {
                loader.read_scene(&mut r, None)?;
                loader.summary.scenes_in_table += 1;
            }
            // (the camera database that follows the scenes is not read)
        }

        // named scenes
        if let Some(objects_dir) = &objects_dir {
            let mut defined: BTreeMap<usize, String> = BTreeMap::new();
            for (dir_name, dir_path) in sorted_entries(&RealFs, objects_dir, true)? {
                if dir_name == "." || dir_name == ".." {
                    continue;
                }
                loader.history.clear();
                for (file_name, file_path) in sorted_entries(&RealFs, &dir_path, false)? {
                    let upper = file_name.to_ascii_uppercase();
                    let Some(stem) = upper.strip_suffix(".EES") else { continue };
                    let Some(scene) = scene_by_name(stem.as_bytes()) else {
                        loader.summary.ees_unresolved_names.push(format!("{dir_name}/{file_name}"));
                        continue;
                    };
                    let bytes = read_file(&file_path, &format!("{dir_name}/{file_name}"))?;
                    let label = format!("{dir_name}/{file_name}");
                    let mut r = Reader::new(&label, &bytes);
                    let version = r.i32()?;
                    if !(-3..=-1).contains(&version) {
                        loader.summary.ees_unsupported_version += 1;
                        continue;
                    }
                    if let Some(previous) = defined.insert(scene, label.clone()) {
                        loader.summary.ees_duplicate_scenes.push(format!("{} replaced by {label}", previous));
                    }
                    loader.read_scene(&mut r, Some((scene, Named { version, prefix: &dir_path })))?;
                    loader.summary.ees_applied += 1;
                }
            }
        }

        loader.finish()
    }

    /// The consolidated surface block (the original's "reallocates surfaces memory"), the summary, the store.
    fn finish(mut self) -> Result<ModelStore, ModelError> {
        // the original consolidates objects 1 ..= total_number_of_objects, INCLUSIVE of the counter's final value: with no custom object that is the last table object, and with
        // M custom ones the counter N + M names a slot nobody was given (zero surfaces), which is in the loop and adds nothing
        let final_objects = self.next_object;
        let mut surfaces: Vec<Surface> = Vec::new();
        for index in 1..=final_objects {
            let Some(own) = self.own_surfaces.remove(&index) else { continue };
            if own.len() != self.objects[index].number_of_surfaces as usize {
                return fail(
                    ErrorKind::Inconsistent,
                    "3dobjs",
                    format!(
                        "object {index}: {} surface records, {} declared",
                        own.len(),
                        self.objects[index].number_of_surfaces
                    ),
                );
            }
            self.objects[index].surfaces_start = surfaces.len();
            surfaces.extend_from_slice(&own);
        }
        self.summary.objects_total = final_objects;
        self.summary.table_objects = self.table_objects;
        self.summary.object_room = self.room.max(0) as usize;
        self.summary.surface_block = surfaces.len();
        self.summary.scenes_loaded = self.scenes.iter().filter(|s| s.loaded).count();
        Ok(ModelStore {
            scenes: self.scenes,
            objects: self.objects,
            surfaces,
            points: self.points,
            faces: self.faces,
            point_list: self.point_list,
            surface_points: self.surface_points,
            summary: self.summary,
        })
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// the queries

/// Why a query was refused. The scene or depth was not an acceptable argument, or the model's data cannot be shown to be backed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryError {
    BadScene(i64),
    BadDepth(i64),
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryError::BadScene(scene) => write!(f, "scene {scene} is outside 1 to {}", scene_last() - 1),
            QueryError::BadDepth(depth) => write!(f, "depth {depth} is not a non-negative 32-bit whole number"),
        }
    }
}

impl std::error::Error for QueryError {}

/// The answer to one triangle query, as the original's accessor gives it (plus the explicit refusals the original has no word for).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Triangle {
    /// the accessor returned TRUE: the nine floats' bit patterns, point 0 x y z, point 1, point 2
    Found([u32; 9]),
    /// the accessor returned FALSE with a collision object present: no triangle at this depth or beyond
    End,
    /// the scene has no collision object (-1): the accessor returns FALSE without reading any
    Absent,
    /// the data cannot be shown to be backed, or is one the original would have crashed on: no answer is given
    Refused(&'static str),
}

/// A scene as the service shows it: plain values, no reference into the store.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneView {
    pub scene: usize,
    pub name: &'static str,
    pub loaded: bool,
    pub source: SceneSource,
    pub main: i32,
    pub collision: i32,
    pub radius: u32,
    pub bounding_box2: [u32; 6],
    pub bounding_box: [u32; 6],
    pub collision_object: Option<ObjectView>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectView {
    pub source: ObjectSource,
    pub points: u16,
    pub faces: u16,
    pub surfaces: u16,
    pub radius: u32,
    pub bounding_box: [u32; 6],
    pub surface_faces: usize,
    pub polygon_faces: usize,
    pub nonpolygon_surfaces: usize,
    pub zero_face_surfaces: usize,
    pub layout: &'static str,
    pub invalid: Option<&'static str>,
}

fn bits6(values: &[f32]) -> [u32; 6] {
    let mut out = [0u32; 6];
    for (slot, value) in out.iter_mut().zip(values) {
        *slot = value.to_bits();
    }
    out
}

impl ModelStore {
    /// Loads the model data of the retail installation at `root` from its disk assets. Every refusal names the file and why; there is no fallback to a synthetic or
    /// empty database.
    pub fn load(root: &Path) -> Result<ModelStore, ModelError> {
        // the kernel's size must be the one this side lays out
        // SAFETY: a plain function returning a size.
        let kernel_node = unsafe { eech_model_node_size() };
        if kernel_node != std::mem::size_of::<Node>() {
            return fail(
                ErrorKind::Limit,
                "model_step.c",
                format!("the numeric kernel's node is {kernel_node} bytes, the host's {}", std::mem::size_of::<Node>()),
            );
        }
        Loader::load(root)
    }

    pub fn summary(&self) -> &LoadSummary {
        &self.summary
    }

    fn check_scene(&self, scene: i64) -> Result<usize, QueryError> {
        // the signed 64-bit value is compared with the scene count BEFORE it is narrowed: `4294967297 as usize` is 1 on a 32-bit target
        if scene < 1 || scene >= self.scenes.len() as i64 {
            return Err(QueryError::BadScene(scene));
        }
        usize::try_from(scene).map_err(|_| QueryError::BadScene(scene))
    }

    pub fn scene(&self, scene: i64) -> Result<SceneView, QueryError> {
        let id = self.check_scene(scene)?;
        let s = &self.scenes[id];
        let collision_object = if s.loaded && s.collision > 0 && (s.collision as usize) < self.objects.len() && self.objects[s.collision as usize].present {
            Some(self.object_view(&self.objects[s.collision as usize]))
        } else {
            None
        };
        Ok(SceneView {
            scene: id,
            name: compiled_scene_name(id).unwrap_or(""),
            loaded: s.loaded,
            source: s.source,
            main: s.index,
            collision: s.collision,
            radius: s.radius.to_bits(),
            bounding_box2: bits6(&s.bounds[6..]),
            bounding_box: bits6(&s.bounds[..6]),
            collision_object,
        })
    }

    fn object_view(&self, object: &Object) -> ObjectView {
        let (mut surface_faces, mut polygon_faces, mut nonpolygon, mut zero) = (0usize, 0usize, 0usize, 0usize);
        for k in 0..object.number_of_surfaces as usize {
            if let Some(surface) = self.surfaces.get(object.surfaces_start + k) {
                surface_faces += surface.faces as usize;
                if surface.polygons() {
                    polygon_faces += surface.faces as usize;
                } else {
                    nonpolygon += 1;
                }
                if surface.faces == 0 {
                    zero += 1;
                }
            }
        }
        let faces = object.number_of_faces as usize;
        let layout = if zero > 0 {
            "zero_face_surface"
        } else if faces == surface_faces && faces == polygon_faces {
            "polygon"
        } else if faces == surface_faces {
            "mixed"
        } else {
            "inconsistent"
        };
        ObjectView {
            source: object.source,
            points: object.number_of_points,
            faces: object.number_of_faces,
            surfaces: object.number_of_surfaces,
            radius: object.radius.to_bits(),
            bounding_box: bits6(&object.bounding_box),
            surface_faces,
            polygon_faces,
            nonpolygon_surfaces: nonpolygon,
            zero_face_surfaces: zero,
            layout,
            invalid: object.invalid,
        }
    }

    /// `get_object_3d_collision_object_geometry_triangle (scene, points, depth)`, once, for a depth that is a non-negative 32-bit whole number.
    pub fn triangle(&self, scene: i64, depth: i64) -> Result<Triangle, QueryError> {
        let id = self.check_scene(scene)?;
        if !(0..=i32::MAX as i64).contains(&depth) {
            return Err(QueryError::BadDepth(depth));
        }
        let s = &self.scenes[id];
        if !s.loaded {
            // the original's accessor on a scene that was never read: the zeroed entry says index 0, not -1, and the first surface read is a NULL dereference
            return Ok(Triangle::Refused("no_scene"));
        }
        if s.collision == -1 {
            return Ok(Triangle::Absent);
        }
        if s.collision < 1 || s.collision as usize >= self.objects.len() {
            return Ok(Triangle::Refused("collision_object_outside_table"));
        }
        if !self.objects[s.collision as usize].present {
            return Ok(Triangle::Refused("null_object"));
        }
        Ok(self.decode(&self.objects[s.collision as usize], depth as i32))
    }

    /// The original's walk, translated exactly (3dobjdb.c); each read is checked against the BACKED length of its array.
    fn decode(&self, obj: &Object, depth_in: i32) -> Triangle {
        if let Some(why) = obj.invalid {
            return Triangle::Refused(why);
        }
        // obj->surfaces[0] is read unconditionally
        let surface_at = |k: usize| self.surfaces.get(obj.surfaces_start + k).copied();
        if obj.number_of_surfaces == 0 {
            return Triangle::Refused("null_surfaces");
        }
        let Some(first) = surface_at(0) else {
            return Triangle::Refused("unbacked_surfaces");
        };
        let mut depth = depth_in as i64;
        let mut count = 0usize;
        let mut surface = 0usize;
        let mut refs = 0usize;
        let mut surface_point_base = 0usize;
        // None: the surface read went past the backed block (the original reads memory it does not own); it only matters if the loop uses it
        let mut current: Option<Surface> = Some(first);
        let mut number_of_surface_points = first.points();
        let mut number_of_surface_faces = first.faces as i64;
        let n_faces = obj.number_of_faces as usize;

        loop {
            if count >= n_faces {
                break;
            }
            let Some(face_span) = obj.faces else { return Triangle::Refused("null_faces") };
            let Some(face_points) = self.faces.get(face_span, count) else {
                return Triangle::Refused("unbacked_faces");
            };
            let face_points = face_points as i64;
            if !((face_points - 3) < depth) {
                break;
            }
            if current.is_none() {
                return Triangle::Refused("unbacked_surfaces");
            }
            depth -= face_points - 2;
            refs += face_points as usize;
            count += 1;
            number_of_surface_faces -= 1;
            if number_of_surface_faces == 0 {
                surface_point_base += number_of_surface_points as usize;
                surface += 1;
                current = surface_at(surface);
                match current {
                    Some(next) => {
                        number_of_surface_points = next.points();
                        number_of_surface_faces = next.faces as i64;
                    }
                    None => {
                        number_of_surface_points = 0;
                        number_of_surface_faces = 0;
                    }
                }
            }
        }
        if count >= n_faces {
            return Triangle::End;
        }

        // the triangle: fan point 0 and the two points of this depth
        let Some(list) = obj.point_list else {
            return Triangle::Refused("null_point_list");
        };
        let Some(surface_points) = obj.surface_points else {
            return Triangle::Refused("null_surface_points");
        };
        let Some(points) = obj.points else { return Triangle::Refused("null_points") };
        let mut coordinates = [0i16; 9];
        for (slot, offset) in [0i64, depth + 1, depth + 2].into_iter().enumerate() {
            let Some(index) = (refs as i64).checked_add(offset).filter(|v| *v >= 0).map(|v| v as usize) else {
                return Triangle::Refused("unbacked_point_list");
            };
            let Some(reference) = self.point_list.get(list, index) else {
                return Triangle::Refused("unbacked_point_list");
            };
            let Some(point_index) = self.surface_points.get(surface_points, surface_point_base + reference as usize) else {
                return Triangle::Refused("unbacked_surface_points");
            };
            let Some(point) = self.points.get(points, point_index as usize) else {
                return Triangle::Refused("unbacked_points");
            };
            coordinates[slot * 3] = point[0];
            coordinates[slot * 3 + 1] = point[1];
            coordinates[slot * 3 + 2] = point[2];
        }
        let mut out = [0.0f32; 9];
        // SAFETY: the pointers are to locals of exactly 9 shorts, 6 floats and 9 floats; the kernel reads the first two and writes the third.
        let code = unsafe { eech_model_triangle(coordinates.as_ptr(), obj.bounding_box.as_ptr(), out.as_mut_ptr()) };
        match code {
            0 => {}
            1 => return Triangle::Refused("fp_masks"),
            _ => return Triangle::Refused("fp_environment"),
        }
        let mut bits = [0u32; 9];
        for (slot, value) in bits.iter_mut().zip(out.iter()) {
            *slot = value.to_bits();
        }
        Triangle::Found(bits)
    }
}

/// `0x%08x,...` as the original's reports write float bit patterns.
pub fn hex_list(bits: &[u32]) -> String {
    bits.iter().map(|b| format!("0x{b:08x}")).collect::<Vec<_>>().join(",")
}

// ---------------------------------------------------------------------------------------------------------------------------------
// the resolver and the enumerator against failures a test cannot portably cause (an unreadable directory, an entry that cannot be read, a path whose type cannot be
// determined). The actual `find_named`, `typed_entries` and `sorted_entries` run; only the two filesystem questions are answered by the fake.

#[cfg(test)]
mod fs_tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::{Error, ErrorKind as IoKind};

    enum Node {
        /// a directory whose listing is these entries, each its name or the error reading it
        Dir(Vec<Result<&'static str, IoKind>>),
        /// a directory that cannot be listed
        Unlistable(IoKind),
        File,
        /// a path whose type cannot be determined
        Unknowable(IoKind),
    }

    struct FakeFs(HashMap<PathBuf, Node>);

    impl Fs for FakeFs {
        fn list(&self, dir: &Path) -> std::io::Result<Vec<std::io::Result<String>>> {
            match self.0.get(dir) {
                Some(Node::Dir(items)) => Ok(items
                    .iter()
                    .map(|item| match item {
                        Ok(name) => Ok(name.to_string()),
                        Err(kind) => Err(Error::new(*kind, "entry unreadable")),
                    })
                    .collect()),
                Some(Node::Unlistable(kind)) => Err(Error::new(*kind, "listing refused")),
                Some(_) => Err(Error::other("not a directory")),
                None => Err(Error::new(IoKind::NotFound, "no such directory")),
            }
        }

        fn kind(&self, path: &Path) -> std::io::Result<Option<Kind>> {
            match self.0.get(path) {
                Some(Node::Dir(_)) | Some(Node::Unlistable(_)) => Ok(Some(Kind::Directory)),
                Some(Node::File) => Ok(Some(Kind::File)),
                Some(Node::Unknowable(kind)) => Err(Error::new(*kind, "type unknown")),
                None => Ok(None),
            }
        }
    }

    fn data() -> PathBuf {
        PathBuf::from("/install/cohokum/3ddata")
    }

    fn fake(nodes: Vec<(&str, Node)>) -> FakeFs {
        FakeFs(
            nodes
                .into_iter()
                .map(|(name, node)| (if name.is_empty() { data() } else { data().join(name) }, node))
                .collect(),
        )
    }

    fn denied() -> IoKind {
        IoKind::PermissionDenied
    }

    #[test]
    fn a_genuinely_absent_optional_directory_is_none_and_a_present_one_is_found_exactly_or_by_case() {
        let fs = fake(vec![("", Node::Dir(vec![Ok("bininfo.bin")])), ("bininfo.bin", Node::File)]);
        assert_eq!(find_named(&fs, &data(), "OBJECTS", Kind::Directory).unwrap(), None);
        // the parent itself being absent is absent too
        assert_eq!(find_named(&FakeFs(HashMap::new()), &data(), "OBJECTS", Kind::Directory).unwrap(), None);
        let fs = fake(vec![("OBJECTS", Node::Dir(vec![]))]);
        assert_eq!(find_named(&fs, &data(), "OBJECTS", Kind::Directory).unwrap(), Some(data().join("OBJECTS")));
        let fs = fake(vec![("", Node::Dir(vec![Ok("Objects")])), ("Objects", Node::Dir(vec![]))]);
        assert_eq!(find_named(&fs, &data(), "OBJECTS", Kind::Directory).unwrap(), Some(data().join("Objects")));
    }

    #[test]
    fn an_existing_path_that_cannot_be_examined_listed_or_is_the_wrong_kind_is_an_error_naming_it() {
        // present but a file
        let fs = fake(vec![("OBJECTS", Node::File)]);
        let error = find_named(&fs, &data(), "OBJECTS", Kind::Directory).unwrap_err();
        assert!(error.detail.contains("not a directory") && error.file.contains("OBJECTS"), "{error}");
        // present with another case, but a file
        let fs = fake(vec![("", Node::Dir(vec![Ok("objects")])), ("objects", Node::File)]);
        assert!(find_named(&fs, &data(), "OBJECTS", Kind::Directory).unwrap_err().file.contains("objects"));
        // its type cannot be determined
        let fs = fake(vec![("OBJECTS", Node::Unknowable(denied()))]);
        let error = find_named(&fs, &data(), "OBJECTS", Kind::Directory).unwrap_err();
        assert_eq!((error.kind, error.file.contains("OBJECTS")), (ErrorKind::Unreadable, true), "{error}");
        // the parent cannot be listed to look for another case
        let fs = fake(vec![("", Node::Unlistable(denied()))]);
        let error = find_named(&fs, &data(), "OBJECTS", Kind::Directory).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Unreadable);
        assert!(error.file.contains("3ddata"), "{error}");
        // one entry of the parent cannot be read
        let fs = fake(vec![("", Node::Dir(vec![Ok("bininfo.bin"), Err(denied())])), ("bininfo.bin", Node::File)]);
        assert_eq!(find_named(&fs, &data(), "OBJECTS", Kind::Directory).unwrap_err().kind, ErrorKind::Unreadable);
        // two directories that differ only in case
        let fs = fake(vec![
            ("", Node::Dir(vec![Ok("Objects"), Ok("OBJECTs")])),
            ("Objects", Node::Dir(vec![])),
            ("OBJECTs", Node::Dir(vec![])),
        ]);
        assert_eq!(find_named(&fs, &data(), "OBJECTS", Kind::Directory).unwrap_err().kind, ErrorKind::Inconsistent);
        // an entry of that name that vanished between the listing and the question
        let fs = fake(vec![("", Node::Dir(vec![Ok("objects")]))]);
        assert_eq!(find_named(&fs, &data(), "OBJECTS", Kind::Directory).unwrap_err().kind, ErrorKind::Inconsistent);
    }

    #[test]
    fn files_resolve_exactly_or_by_case_and_every_failure_is_an_error_not_an_absent_file() {
        let fs = fake(vec![("", Node::Dir(vec![Ok("3DOBJS.BIN")])), ("3DOBJS.BIN", Node::File)]);
        assert_eq!(find_file(&fs, &data(), "3dobjs.bin").unwrap(), Some(data().join("3DOBJS.BIN")));
        assert_eq!(find_file(&fs, &data(), "3dobjdb.bin").unwrap(), None);
        // a directory where a file is wanted
        let fs = fake(vec![("3dobjs.bin", Node::Dir(vec![]))]);
        assert!(find_file(&fs, &data(), "3dobjs.bin").unwrap_err().detail.contains("not a file"));
        // cannot list to look for another case
        let fs = fake(vec![("", Node::Unlistable(denied()))]);
        assert_eq!(find_file(&fs, &data(), "3dobjs.bin").unwrap_err().kind, ErrorKind::Unreadable);
        // cannot determine the type
        let fs = fake(vec![("3dobjs.bin", Node::Unknowable(denied()))]);
        assert_eq!(find_file(&fs, &data(), "3dobjs.bin").unwrap_err().kind, ErrorKind::Unreadable);
        // two files that differ only in case: neither is the spelling asked for, so the fallback is ambiguous
        let fs = fake(vec![
            ("", Node::Dir(vec![Ok("A.EEO"), Ok("a.eeo")])),
            ("A.EEO", Node::File),
            ("a.eeo", Node::File),
        ]);
        assert_eq!(find_file(&fs, &data(), "a.EEO").unwrap_err().kind, ErrorKind::Inconsistent);
        // ... but an exact spelling wins over the case-only duplicate, whichever spelling is asked for (the contract find_named documents)
        assert_eq!(find_file(&fs, &data(), "A.EEO").unwrap(), Some(data().join("A.EEO")));
        assert_eq!(find_file(&fs, &data(), "a.eeo").unwrap(), Some(data().join("a.eeo")));
        // the directory resolver has the same precedence
        let fs = fake(vec![
            ("", Node::Dir(vec![Ok("Objects"), Ok("OBJECTS")])),
            ("Objects", Node::Dir(vec![])),
            ("OBJECTS", Node::Dir(vec![])),
        ]);
        assert_eq!(find_named(&fs, &data(), "OBJECTS", Kind::Directory).unwrap(), Some(data().join("OBJECTS")));
        assert_eq!(find_named(&fs, &data(), "objects", Kind::Directory).unwrap_err().kind, ErrorKind::Inconsistent);
    }

    #[test]
    fn enumeration_keeps_the_originals_order_and_kind_filter_when_it_succeeds() {
        let fs = fake(vec![
            ("", Node::Dir(vec![Ok("b.EES"), Ok("A.ees"), Ok("sub"), Ok("c.txt"), Ok("a.ees")])),
            ("b.EES", Node::File),
            ("A.ees", Node::File),
            ("a.ees", Node::File),
            ("c.txt", Node::File),
            ("sub", Node::Dir(vec![])),
        ]);
        let files: Vec<String> = sorted_entries(&fs, &data(), false).unwrap().into_iter().map(|(name, _)| name).collect();
        // by upper-case name, ties by the exact name; the directory is not a file
        assert_eq!(files, vec!["A.ees", "a.ees", "b.EES", "c.txt"]);
        let directories: Vec<String> = sorted_entries(&fs, &data(), true).unwrap().into_iter().map(|(name, _)| name).collect();
        assert_eq!(directories, vec!["sub"]);
        // an empty directory is an empty list, not an error
        let empty = fake(vec![("", Node::Dir(vec![]))]);
        assert!(sorted_entries(&empty, &data(), true).unwrap().is_empty());
    }

    #[test]
    fn a_failed_enumeration_is_an_error_naming_the_path_never_an_empty_or_shorter_list() {
        // the directory cannot be listed
        let fs = fake(vec![("", Node::Unlistable(denied()))]);
        let error = sorted_entries(&fs, &data(), true).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Unreadable);
        assert!(error.file.contains("3ddata") && error.detail.contains("cannot list"), "{error}");
        // a missing directory is also an error for the enumerator (the caller found it first)
        assert_eq!(sorted_entries(&FakeFs(HashMap::new()), &data(), true).unwrap_err().kind, ErrorKind::Unreadable);
        // one entry cannot be read, after others that can
        let fs = fake(vec![
            ("", Node::Dir(vec![Ok("a.ees"), Err(denied()), Ok("b.ees")])),
            ("a.ees", Node::File),
            ("b.ees", Node::File),
        ]);
        let error = sorted_entries(&fs, &data(), false).unwrap_err();
        assert!(error.detail.contains("cannot read a directory entry"), "{error}");
        // one entry's type cannot be determined
        let fs = fake(vec![
            ("", Node::Dir(vec![Ok("a.ees"), Ok("b.ees")])),
            ("a.ees", Node::File),
            ("b.ees", Node::Unknowable(denied())),
        ]);
        let error = sorted_entries(&fs, &data(), false).unwrap_err();
        assert!(error.file.contains("b.ees") && error.detail.contains("cannot determine"), "{error}");
        // an entry that vanished during the listing
        let fs = fake(vec![("", Node::Dir(vec![Ok("a.ees")]))]);
        assert_eq!(sorted_entries(&fs, &data(), false).unwrap_err().kind, ErrorKind::Inconsistent);
    }
}
