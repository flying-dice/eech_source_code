//! The host's static tree data and tree segment collision (EEDCS-49, D3): the first physical responsibility moved under the host. The host
//! reads its OWN `tree_pos.sec` and `tree_pos.dat` into storage it owns, and answers whether a line segment hits a forest tree with EECH's own
//! `get_line_of_sight_collision_tree`, kept in C (`csrc/tree_step.c`, expression for expression) and driven here one answer at a time.
//!
//! This is preparation, not D3 completion. Nothing calls it from the campaign DLL: the campaign's tree queries still run the original
//! solver, and no production handoff is claimed. The campaign decides nothing here and the host decides nothing about campaign policy; the
//! campaign's side of a hosted query is only the two terrain elevations the original reads, which it supplies one at a time (the terrain
//! heights are not the host's yet; that is declared transitional coupling, as for line of sight).
//!
//! # Ownership and lifetimes
//!
//! * [`TreeScene`] owns everything the kernel reads: the decoded tree records, the per-sector `count`/`first` table, the optional decoded forest
//!   tree collision triangles and the map metadata. It is immutable once built. No engine heap, entity pointer or callback is involved, and no
//!   raw pointer is public.
//! * [`TreeCollision`] holds an `Arc<TreeScene>` and one opaque query block. Each call to the kernel receives a descriptor of the scene built
//!   from that `Arc` for the duration of the call; the C side keeps no pointer between calls, so the scene cannot be freed under a pending
//!   query, and dropping a `TreeCollision` drops its pending query with it. Raw pointers exist only inside this module's `unsafe` calls.
//!
//! # What the loader does with a map's files
//!
//! * The file names are taken as they are on disk: the directory is searched for the entries that equal `tree_pos.sec` and `tree_pos.dat`
//!   ignoring ASCII case (retail Georgia spells them in lower case, retail Lebanon in upper case), the entry's own spelling is opened, and
//!   nothing is renamed. Two entries that differ only in case are refused as ambiguous.
//! * Both files absent (or only one of them) is a known "no trees loaded", reported in the census, as in the original; a sector header that
//!   differs from the terrain's sector counts is also a known "no trees loaded" (the original ditches the trees there), and the census says so.
//! * Anything else that is wrong, a file shorter than its header says, a sector whose trees lie outside the data file, a negative count, is an
//!   error. Lengths are checked in `u64` arithmetic before anything is allocated or indexed.
//! * Trailing bytes beyond what the header and the whole records account for are accepted and counted in the census, not interpreted.
//!
//! # Dependencies that remain
//!
//! * The map metadata ([`TreeMap`]: extents, terrain sector counts, the terrain height range the tree heights use) is the TERRAIN's; the terrain
//!   host module (G1) is not part of this branch, so the metadata is supplied explicitly (the tests supply the facts the original recorded).
//! * The forest tree's collision triangles come from the model's collision object, whose decoding is not the host's yet. [`ForestGeometry`] must
//!   state either that the original has none ([`ForestGeometry::KnownAbsent`], the retail facts) or the decoded triangles; there is no default.
//!   The geometry branch is copied from the original but has not been run against a native witness.

use std::ffi::{c_int, c_void};
use std::path::Path;
use std::sync::Arc;

/// The terrain sector side the tree sector tables are indexed by (`TERRAIN_3D_SECTOR_SIDE_LENGTH`).
pub const SECTOR_SIDE: i32 = 2048;
/// The bytes of one tree record in the retail files (`sizeof (terrain_3d_tree_data)`, verified natively).
pub const RECORD_BYTES: usize = 8;

/// One tree, decoded explicitly from its 8 bytes: `x` and `z` signed 16 bits at byte offsets 0 and 2, `y` unsigned 16 bits at 4, the colour at
/// 6, and the last byte holding the height in its low 7 bits and the type in its high bit. The colour is carried and is not a physical input.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeRecord {
    pub x: i16,
    pub z: i16,
    pub y: u16,
    pub colour: u8,
    pub height: u8,
    pub kind: u8,
    reserved: u8,
}

impl TreeRecord {
    /// Decodes one record from its file bytes.
    #[must_use]
    pub fn from_bytes(bytes: [u8; RECORD_BYTES]) -> Self {
        Self {
            x: i16::from_le_bytes([bytes[0], bytes[1]]),
            z: i16::from_le_bytes([bytes[2], bytes[3]]),
            y: u16::from_le_bytes([bytes[4], bytes[5]]),
            colour: bytes[6],
            height: bytes[7] & 0x7f,
            kind: bytes[7] >> 7,
            reserved: 0,
        }
    }

    /// The record's file bytes (the decoding is lossless: the height and type fill the last byte).
    #[must_use]
    pub fn to_bytes(self) -> [u8; RECORD_BYTES] {
        let x = self.x.to_le_bytes();
        let z = self.z.to_le_bytes();
        let y = self.y.to_le_bytes();
        [x[0], x[1], z[0], z[1], y[0], y[1], self.colour, (self.height & 0x7f) | ((self.kind & 1) << 7)]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SectorRecord {
    count: i32,
    first: u32,
}

/// The terrain's metadata the tree query reads. All of it is the terrain's, not the campaign's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeMap {
    pub min_x: f32,
    pub max_x: f32,
    pub min_z: f32,
    pub max_z: f32,
    /// the terrain's sector counts (`terrain_3d_map_width` and `terrain_3d_map_height`), which the tree sector header must equal
    pub sectors_x: i32,
    pub sectors_z: i32,
    /// `terrain_3d_map_minimum_height` and `terrain_3d_map_scaled_height_difference`
    pub minimum_height: f32,
    pub scaled_height_difference: f32,
}

impl TreeMap {
    /// Checks the metadata is usable: finite, a non-empty positive map inside the sector grid, so that every sector the kernel reaches exists.
    pub fn new(
        (min_x, max_x, min_z, max_z): (f32, f32, f32, f32),
        sectors_x: i32,
        sectors_z: i32,
        minimum_height: f32,
        scaled_height_difference: f32,
    ) -> Result<Self, String> {
        let finite = [min_x, max_x, min_z, max_z, minimum_height, scaled_height_difference];
        if finite.iter().any(|value| !value.is_finite()) {
            return Err("tree map metadata must be finite".into());
        }
        if !(1..=4096).contains(&sectors_x) || !(1..=4096).contains(&sectors_z) {
            return Err(format!("tree map sector counts {sectors_x} by {sectors_z} are outside 1 to 4096"));
        }
        if min_x < 0.0 || min_z < 0.0 || min_x > max_x || min_z > max_z {
            return Err("tree map extents must satisfy 0 <= min <= max on x and z".into());
        }
        if max_x >= (sectors_x * SECTOR_SIDE) as f32 || max_z >= (sectors_z * SECTOR_SIDE) as f32 {
            return Err("tree map extents reach beyond the terrain sector grid".into());
        }
        Ok(Self {
            min_x,
            max_x,
            min_z,
            max_z,
            sectors_x,
            sectors_z,
            minimum_height,
            scaled_height_difference,
        })
    }
}

/// What the original has for the forest tree's collision triangles. There is no default: a host says which.
#[derive(Clone, Debug, PartialEq)]
pub enum ForestGeometry {
    /// the original's `get_object_3d_collision_object_geometry_triangle` returns FALSE at depth 0 (the retail facts): every tree takes the quad path
    KnownAbsent,
    /// the triangles in the order the original returns them (depth 0, 1, 2 ...), each as three points in the tree's own scale
    Triangles(Vec<[[f32; 3]; 3]>),
}

/// Why a scene has no trees, when it has none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoTrees {
    /// no sector or data file was requested (a scene built without files)
    NotRequested,
    /// the original loads trees only when both files exist
    FilesAbsent { sector_file: bool, data_file: bool },
    /// the original ditches the trees when the header differs from the terrain's sector counts
    HeaderMismatch { tree_sectors_x: i32, tree_sectors_z: i32 },
}

/// What was loaded, for the record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeCensus {
    pub no_trees: Option<NoTrees>,
    /// the on-disk spellings used, when files were read
    pub sector_file: Option<String>,
    pub data_file: Option<String>,
    pub sector_file_bytes: usize,
    pub data_file_bytes: usize,
    pub sector_trailing_bytes: usize,
    pub data_trailing_bytes: usize,
    /// whole records in the data file
    pub records_in_file: usize,
    /// the sum of the sector counts (`tree_total` of the native facts)
    pub tree_total: u64,
    pub sectors_with_trees: usize,
    pub type1_trees: u64,
}

impl TreeCensus {
    fn none(reason: NoTrees) -> Self {
        Self {
            no_trees: Some(reason),
            sector_file: None,
            data_file: None,
            sector_file_bytes: 0,
            data_file_bytes: 0,
            sector_trailing_bytes: 0,
            data_trailing_bytes: 0,
            records_in_file: 0,
            tree_total: 0,
            sectors_with_trees: 0,
            type1_trees: 0,
        }
    }
}

/// The host's static tree data: immutable, owned, checked.
pub struct TreeScene {
    map: TreeMap,
    /// empty when no trees are loaded; else `sectors_x * sectors_z` entries, row-major by z then x
    sectors: Vec<SectorRecord>,
    trees: Vec<TreeRecord>,
    /// nine floats per triangle
    geometry: Vec<f32>,
    geometry_present: bool,
    census: TreeCensus,
}

fn geometry_from(geometry: ForestGeometry) -> Result<(Vec<f32>, bool), String> {
    match geometry {
        ForestGeometry::KnownAbsent => Ok((Vec::new(), false)),
        ForestGeometry::Triangles(triangles) => {
            let flat: Vec<f32> = triangles.iter().flat_map(|triangle| triangle.iter().flatten().copied()).collect();
            if flat.iter().any(|value| !value.is_finite()) {
                return Err("forest tree collision triangles must be finite".into());
            }
            Ok((flat, true))
        }
    }
}

fn le_i32(bytes: &[u8], at: usize) -> i32 {
    i32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

impl TreeScene {
    /// A scene with no trees loaded, for a map that has none (or a test that wants none).
    pub fn without_trees(map: TreeMap, geometry: ForestGeometry) -> Result<Self, String> {
        let (geometry, geometry_present) = geometry_from(geometry)?;
        Ok(Self {
            map,
            sectors: Vec::new(),
            trees: Vec::new(),
            geometry,
            geometry_present,
            census: TreeCensus::none(NoTrees::NotRequested),
        })
    }

    /// Decodes a sector file and a data file already in memory, as the original does after it has opened them.
    pub fn from_bytes(map: TreeMap, sector_file: &[u8], data_file: &[u8], geometry: ForestGeometry) -> Result<Self, String> {
        let (geometry, geometry_present) = geometry_from(geometry)?;
        if sector_file.len() < 8 {
            return Err(format!("tree sector file is {} bytes, shorter than its 8-byte header", sector_file.len()));
        }
        let (tree_sectors_x, tree_sectors_z) = (le_i32(sector_file, 0), le_i32(sector_file, 4));
        if tree_sectors_x != map.sectors_x || tree_sectors_z != map.sectors_z {
            let mut census = TreeCensus::none(NoTrees::HeaderMismatch {
                tree_sectors_x,
                tree_sectors_z,
            });
            census.sector_file_bytes = sector_file.len();
            census.data_file_bytes = data_file.len();
            return Ok(Self {
                map,
                sectors: Vec::new(),
                trees: Vec::new(),
                geometry,
                geometry_present,
                census,
            });
        }
        // the header equals the validated map counts, so both are positive and at most 4096
        let cells = (tree_sectors_x as u64) * (tree_sectors_z as u64);
        let table_bytes = cells.checked_mul(8).and_then(|bytes| bytes.checked_add(8)).ok_or("tree sector table size overflows")?;
        if (sector_file.len() as u64) < table_bytes {
            return Err(format!(
                "tree sector file is {} bytes, shorter than the {table_bytes} its {tree_sectors_x} by {tree_sectors_z} header needs",
                sector_file.len()
            ));
        }
        let records_in_file = data_file.len() / RECORD_BYTES;
        if u32::try_from(records_in_file).is_err() {
            return Err(format!("tree data file holds {records_in_file} records, more than 32 bits can index"));
        }
        let mut sectors = Vec::with_capacity(cells as usize);
        let (mut tree_total, mut sectors_with_trees) = (0u64, 0usize);
        for cell in 0..cells as usize {
            let at = 8 + cell * 8;
            let (count, offset) = (le_i32(sector_file, at), le_i32(sector_file, at + 4));
            let (x, z) = (cell % tree_sectors_x as usize, cell / tree_sectors_x as usize);
            if count < 0 {
                return Err(format!("tree sector ({x}, {z}) has a negative tree count {count}"));
            }
            if count == 0 {
                // the original never reads through a zero-count sector's offset, so it is not judged
                sectors.push(SectorRecord { count: 0, first: 0 });
                continue;
            }
            let first = u32::try_from(offset).map_err(|_| format!("tree sector ({x}, {z}) has a negative record offset {offset}"))?;
            if u64::from(first) + u64::from(count as u32) > records_in_file as u64 {
                return Err(format!(
                    "tree sector ({x}, {z}) reaches records {first} to {} but the data file holds {records_in_file}",
                    u64::from(first) + u64::from(count as u32)
                ));
            }
            tree_total += u64::from(count as u32);
            sectors_with_trees += 1;
            sectors.push(SectorRecord { count, first });
        }
        let trees: Vec<TreeRecord> = data_file
            .chunks_exact(RECORD_BYTES)
            .map(|chunk| TreeRecord::from_bytes(chunk.try_into().expect("a whole record")))
            .collect();
        let type1_trees = sectors
            .iter()
            .filter(|sector| sector.count > 0)
            .map(|sector| trees[sector.first as usize..sector.first as usize + sector.count as usize].iter().filter(|tree| tree.kind == 1).count() as u64)
            .sum();
        let census = TreeCensus {
            no_trees: None,
            sector_file: None,
            data_file: None,
            sector_file_bytes: sector_file.len(),
            data_file_bytes: data_file.len(),
            sector_trailing_bytes: sector_file.len() - table_bytes as usize,
            data_trailing_bytes: data_file.len() % RECORD_BYTES,
            records_in_file,
            tree_total,
            sectors_with_trees,
            type1_trees,
        };
        Ok(Self {
            map,
            sectors,
            trees,
            geometry,
            geometry_present,
            census,
        })
    }

    /// Reads the map's own tree files from `directory`, spelled as they are on disk (see the module documentation).
    pub fn load(directory: &Path, map: TreeMap, geometry: ForestGeometry) -> Result<Self, String> {
        let find = |wanted: &str| -> Result<Option<String>, String> {
            let mut found: Vec<String> = Vec::new();
            for entry in std::fs::read_dir(directory).map_err(|e| format!("cannot list {}: {e}", directory.display()))? {
                let entry = entry.map_err(|e| format!("cannot list {}: {e}", directory.display()))?;
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.eq_ignore_ascii_case(wanted) && entry.path().is_file() {
                    found.push(name);
                }
            }
            match found.len() {
                0 => Ok(None),
                1 => Ok(found.pop()),
                _ => Err(format!("{} holds more than one spelling of {wanted}: {found:?}", directory.display())),
            }
        };
        let (sector_name, data_name) = (find("tree_pos.sec")?, find("tree_pos.dat")?);
        let (Some(sector_name), Some(data_name)) = (sector_name.clone(), data_name.clone()) else {
            let (geometry, geometry_present) = geometry_from(geometry)?;
            return Ok(Self {
                map,
                sectors: Vec::new(),
                trees: Vec::new(),
                geometry,
                geometry_present,
                census: TreeCensus::none(NoTrees::FilesAbsent {
                    sector_file: sector_name.is_some(),
                    data_file: data_name.is_some(),
                }),
            });
        };
        let sector_bytes = std::fs::read(directory.join(&sector_name)).map_err(|e| format!("cannot read {sector_name}: {e}"))?;
        let data_bytes = std::fs::read(directory.join(&data_name)).map_err(|e| format!("cannot read {data_name}: {e}"))?;
        let mut scene = Self::from_bytes(map, &sector_bytes, &data_bytes, geometry)?;
        scene.census.sector_file = Some(sector_name);
        scene.census.data_file = Some(data_name);
        Ok(scene)
    }

    #[must_use]
    pub fn map(&self) -> &TreeMap {
        &self.map
    }

    #[must_use]
    pub fn census(&self) -> &TreeCensus {
        &self.census
    }

    /// The trees of terrain sector (`x`, `z`) in file order: none for a sector outside the grid or when no trees are loaded.
    #[must_use]
    pub fn sector_trees(&self, x: i32, z: i32) -> &[TreeRecord] {
        if self.sectors.is_empty() || x < 0 || z < 0 || x >= self.map.sectors_x || z >= self.map.sectors_z {
            return &[];
        }
        let sector = self.sectors[z as usize * self.map.sectors_x as usize + x as usize];
        if sector.count <= 0 {
            return &[];
        }
        &self.trees[sector.first as usize..sector.first as usize + sector.count as usize]
    }

    fn raw(&self) -> RawScene {
        RawScene {
            map_min: [self.map.min_x, 0.0, self.map.min_z],
            map_max: [self.map.max_x, 0.0, self.map.max_z],
            sectors_x: self.map.sectors_x,
            sectors_z: self.map.sectors_z,
            minimum_height: self.map.minimum_height,
            scaled_height_difference: self.map.scaled_height_difference,
            sectors: if self.sectors.is_empty() { std::ptr::null() } else { self.sectors.as_ptr() },
            trees: self.trees.as_ptr(),
            tree_count: self.trees.len(),
            geometry_present: i32::from(self.geometry_present),
            reserved: 0,
            geometry: self.geometry.as_ptr(),
            geometry_triangles: self.geometry.len() / 9,
        }
    }
}

#[repr(C)]
struct RawScene {
    map_min: [f32; 3],
    map_max: [f32; 3],
    sectors_x: i32,
    sectors_z: i32,
    minimum_height: f32,
    scaled_height_difference: f32,
    sectors: *const SectorRecord,
    trees: *const TreeRecord,
    tree_count: usize,
    geometry_present: i32,
    reserved: i32,
    geometry: *const f32,
    geometry_triangles: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RawRequest {
    state: c_int,
    endpoint: c_int,
    x: f32,
    z: f32,
    hit: c_int,
    point: [f32; 3],
    normal: [f32; 3],
}

extern "C" {
    fn eech_tree_query_size() -> usize;
    fn eech_tree_record_size() -> usize;
    fn eech_tree_sector_size() -> usize;
    fn eech_tree_scene_size() -> usize;
    fn eech_tree_fp_state(x87_masks: *mut c_int, sse_masks: *mut c_int, sse_rounding: *mut c_int) -> c_int;
    fn eech_tree_begin(
        query: *mut c_void,
        scene: *const RawScene,
        old_position: *const f32,
        new_position: *const f32,
        intercept_point: *const f32,
        face_normal: *const f32,
        ignore_forest_edge_trees: c_int,
        skip_tree_top_test: c_int,
        out: *mut RawRequest,
    ) -> c_int;
    fn eech_tree_terrain(query: *mut c_void, scene: *const RawScene, elevation: f32, out: *mut RawRequest) -> c_int;
}

/// The floating-point state of the calling thread, read and never written: the exception masks (63 when every exception is masked, as the
/// original's callers have them) and the SSE rounding control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FpState {
    pub x87_masks: i32,
    pub sse_masks: i32,
    pub sse_rounding: i32,
}

/// Reads the calling thread's floating-point state; `Err` where it cannot be read.
pub fn fp_state() -> Result<FpState, String> {
    let (mut x87, mut sse, mut rounding) = (0, 0, 0);
    // SAFETY: three valid out slots; the call only reads registers.
    let code = unsafe { eech_tree_fp_state(&mut x87, &mut sse, &mut rounding) };
    if code != 0 {
        return Err("the floating-point exception masks cannot be read on this architecture".into());
    }
    Ok(FpState {
        x87_masks: x87,
        sse_masks: sse,
        sse_rounding: rounding,
    })
}

/// Which end point of the segment a terrain elevation is wanted for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endpoint {
    Old,
    New,
}

/// What the host says next.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TreeRequest {
    /// the terrain elevation at (`x`, `z`), asked for the segment's `endpoint`; answer with [`TreeCollision::terrain`]
    NeedTerrain { endpoint: Endpoint, x: f32, z: f32 },
    /// the query's answer: whether a tree was hit, and the intercept point and normal as the original leaves them (the caller's own, or what
    /// a miss partly overwrote, when there was no hit)
    Done { hit: bool, point: [f32; 3], normal: [f32; 3] },
}

/// One open tree segment query at a time, against a [`TreeScene`].
pub struct TreeCollision {
    scene: Arc<TreeScene>,
    query: Vec<u64>,
    open: bool,
}

fn finite3(values: &[f32; 3], what: &str) -> Result<(), String> {
    if values.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(format!("tree collision {what} must be finite"))
    }
}

impl TreeCollision {
    /// Opens a query holder on `scene`; refuses if the Rust and C sides disagree about the layout they share.
    pub fn new(scene: Arc<TreeScene>) -> Result<Self, String> {
        // SAFETY: pure size queries.
        let (query, record, sector, raw_scene) = unsafe { (eech_tree_query_size(), eech_tree_record_size(), eech_tree_sector_size(), eech_tree_scene_size()) };
        if record != std::mem::size_of::<TreeRecord>() || sector != std::mem::size_of::<SectorRecord>() || raw_scene != std::mem::size_of::<RawScene>() {
            return Err(format!(
                "tree kernel layout mismatch: record {record}/{}, sector {sector}/{}, scene {raw_scene}/{}",
                std::mem::size_of::<TreeRecord>(),
                std::mem::size_of::<SectorRecord>(),
                std::mem::size_of::<RawScene>()
            ));
        }
        Ok(Self {
            scene,
            query: vec![0u64; query.div_ceil(8)],
            open: false,
        })
    }

    #[must_use]
    pub fn scene(&self) -> &Arc<TreeScene> {
        &self.scene
    }

    /// The kernel's arithmetic is EECH's with every floating-point exception masked: refuse to run on a thread where that is not so, and never
    /// change the thread's state (there is nothing to restore).
    fn require_masks() -> Result<(), String> {
        let state = fp_state()?;
        if state.x87_masks != 63 || state.sse_masks != 63 {
            return Err(format!(
                "tree collision needs every floating-point exception masked (x87 {}, SSE {}, expected 63 and 63)",
                state.x87_masks, state.sse_masks
            ));
        }
        Ok(())
    }

    /// Starts a query for the segment from `old_position` to `new_position`. `intercept_point` and `face_normal` are the caller's own, inputs to the
    /// original's out parameters: a miss returns them as the original leaves them. Replaces any open query.
    pub fn begin(
        &mut self,
        old_position: [f32; 3],
        new_position: [f32; 3],
        intercept_point: [f32; 3],
        face_normal: [f32; 3],
        ignore_forest_edge_trees: bool,
        skip_tree_top_test: bool,
    ) -> Result<TreeRequest, String> {
        self.open = false;
        finite3(&old_position, "old position")?;
        finite3(&new_position, "new position")?;
        Self::require_masks()?;
        let scene = self.scene.raw();
        let mut raw = RawRequest::default();
        // SAFETY: `query` is the kernel's block, sized by it; `scene` describes storage `self.scene` keeps alive for the call; the vectors and
        // the result are valid for the call; nothing is retained by the kernel.
        let code = unsafe {
            eech_tree_begin(
                self.query.as_mut_ptr().cast(),
                &scene,
                old_position.as_ptr(),
                new_position.as_ptr(),
                intercept_point.as_ptr(),
                face_normal.as_ptr(),
                c_int::from(ignore_forest_edge_trees),
                c_int::from(skip_tree_top_test),
                &mut raw,
            )
        };
        self.settle(code, &raw)
    }

    /// The elevation for the lookup just asked about.
    pub fn terrain(&mut self, elevation: f32) -> Result<TreeRequest, String> {
        if !elevation.is_finite() {
            return Err("tree collision terrain elevation must be finite".into());
        }
        if !self.open {
            return Err("tree collision: a terrain answer arrived with no query open".into());
        }
        Self::require_masks()?;
        let scene = self.scene.raw();
        let mut raw = RawRequest::default();
        // SAFETY: as in `begin`.
        let code = unsafe { eech_tree_terrain(self.query.as_mut_ptr().cast(), &scene, elevation, &mut raw) };
        self.settle(code, &raw)
    }

    /// Drops any open query (a host that is told the campaign stopped mid-query).
    pub fn abandon(&mut self) {
        self.open = false;
    }

    fn settle(&mut self, code: c_int, raw: &RawRequest) -> Result<TreeRequest, String> {
        match code {
            0 => {}
            -1 => {
                self.open = false;
                return Err("tree collision: a terrain answer arrived but none was asked for".into());
            }
            _ => {
                self.open = false;
                return Err("tree collision: the scene's tree sectors do not cover a sector the query reached".into());
            }
        }
        match raw.state {
            1 => {
                self.open = false;
                Ok(TreeRequest::Done {
                    hit: raw.hit != 0,
                    point: raw.point,
                    normal: raw.normal,
                })
            }
            2 => {
                self.open = true;
                Ok(TreeRequest::NeedTerrain {
                    endpoint: if raw.endpoint == 0 { Endpoint::Old } else { Endpoint::New },
                    x: raw.x,
                    z: raw.z,
                })
            }
            other => {
                self.open = false;
                Err(format!("tree collision kernel returned an unknown state {other}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Focused contracts of the loader and the kernel against scripted environments. The expectations are derived by hand from EECH's
    //! `get_line_of_sight_collision_tree` (en_cllsn.c) and the line-quad and line-triangle tests (vector.c) as written, not from this
    //! implementation; the worked examples say how. Bit-for-bit equality with the original solver over real recorded queries is protected by
    //! `tests/tree_conformance.rs`, which these tests do not replace.
    use super::*;

    const GRID: i32 = 2;

    fn map() -> TreeMap {
        // two by two terrain sectors (4096 m square), a terrain from -0 m to ... with unit height scale: a record's y is then its height in metres
        TreeMap::new((0.0, 4095.0, 0.0, 4095.0), GRID, GRID, 0.0, 1.0).unwrap()
    }

    fn sector_file(width: i32, height: i32, cells: &[(i32, i32)]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&width.to_le_bytes());
        bytes.extend_from_slice(&height.to_le_bytes());
        for (count, offset) in cells {
            bytes.extend_from_slice(&count.to_le_bytes());
            bytes.extend_from_slice(&offset.to_le_bytes());
        }
        bytes
    }

    fn data_file(records: &[TreeRecord]) -> Vec<u8> {
        records.iter().flat_map(|record| record.to_bytes()).collect()
    }

    fn tree(x: i16, z: i16, y: u16, height: u8, kind: u8) -> TreeRecord {
        TreeRecord {
            x,
            z,
            y,
            colour: 0,
            height,
            kind,
            reserved: 0,
        }
    }

    /// Sector (0, 0) of the two by two grid holds `records`; the other three are empty.
    fn scene_with(records: &[TreeRecord], geometry: ForestGeometry) -> Arc<TreeScene> {
        let sectors = sector_file(GRID, GRID, &[(records.len() as i32, 0), (0, 0), (0, 0), (0, 0)]);
        Arc::new(TreeScene::from_bytes(map(), &sectors, &data_file(records), geometry).unwrap())
    }

    /// Runs one query to its answer, giving `terrain (x, z)` for each lookup and recording the lookups in the order asked.
    fn run(scene: &Arc<TreeScene>, old: [f32; 3], new: [f32; 3], flags: (bool, bool), terrain: impl Fn(f32, f32) -> f32) -> (TreeRequest, Vec<(Endpoint, f32, f32)>) {
        let mut query = TreeCollision::new(scene.clone()).unwrap();
        let mut asked = Vec::new();
        let mut next = query.begin(old, new, [1.0, 2.0, 3.0], [4.0, 5.0, 6.0], flags.0, flags.1).unwrap();
        loop {
            match next {
                TreeRequest::Done { .. } => return (next, asked),
                TreeRequest::NeedTerrain { endpoint, x, z } => {
                    asked.push((endpoint, x, z));
                    next = query.terrain(terrain(x, z)).unwrap();
                }
            }
            assert!(asked.len() < 3, "a query that never ends");
        }
    }

    fn done(request: TreeRequest) -> (bool, [f32; 3], [f32; 3]) {
        match request {
            TreeRequest::Done { hit, point, normal } => (hit, point, normal),
            other => panic!("not an answer: {other:?}"),
        }
    }

    #[test]
    fn a_record_decodes_from_the_bytes_the_native_probe_recorded() {
        // the native probe (tf-layout): x = -2, z = 0x1234, y = 0xabcd, colour = 0xef, height = 0x55, type = 1 stored as feff3412cdabefd5;
        // a height of 127 alone is a last byte of 0x7f and a type of 1 alone is 0x80
        let record = TreeRecord::from_bytes([0xfe, 0xff, 0x34, 0x12, 0xcd, 0xab, 0xef, 0xd5]);
        assert_eq!((record.x, record.z, record.y, record.colour, record.height, record.kind), (-2, 0x1234, 0xabcd, 0xef, 0x55, 1));
        assert_eq!(record.to_bytes(), [0xfe, 0xff, 0x34, 0x12, 0xcd, 0xab, 0xef, 0xd5]);
        let height_only = TreeRecord::from_bytes([0, 0, 0, 0, 0, 0, 0, 0x7f]);
        assert_eq!((height_only.height, height_only.kind), (127, 0));
        let type_only = TreeRecord::from_bytes([0, 0, 0, 0, 0, 0, 0, 0x80]);
        assert_eq!((type_only.height, type_only.kind), (0, 1));
    }

    #[test]
    fn the_loader_reads_sectors_in_file_order_and_counts_them() {
        let records = [tree(1, 2, 3, 4, 0), tree(5, 6, 7, 8, 1), tree(-9, -10, 11, 12, 1)];
        // sector (1, 1) holds the last two records, sector (0, 0) the first; the cell order is z rows of x
        let sectors = sector_file(GRID, GRID, &[(1, 0), (0, 77), (0, 0), (2, 1)]);
        let mut data = data_file(&records);
        data.extend_from_slice(&[9, 9, 9]); // a partial record at the end is counted, not read
        let mut file = sectors;
        file.extend_from_slice(&[1, 2, 3, 4, 5]); // and so are bytes after the table
        let scene = TreeScene::from_bytes(map(), &file, &data, ForestGeometry::KnownAbsent).unwrap();
        assert_eq!(scene.sector_trees(0, 0), &records[0..1]);
        assert_eq!(scene.sector_trees(1, 1), &records[1..3]);
        assert!(scene.sector_trees(1, 0).is_empty() && scene.sector_trees(0, 1).is_empty() && scene.sector_trees(9, 9).is_empty());
        let census = scene.census();
        assert_eq!((census.tree_total, census.sectors_with_trees, census.type1_trees, census.records_in_file), (3, 2, 2, 3));
        assert_eq!((census.data_trailing_bytes, census.sector_trailing_bytes), (3, 5));
        assert!(census.no_trees.is_none());
    }

    #[test]
    fn a_header_that_differs_from_the_terrain_is_a_known_no_trees_and_malformed_data_is_an_error() {
        let mismatch = TreeScene::from_bytes(map(), &sector_file(3, 2, &[]), &[], ForestGeometry::KnownAbsent).unwrap();
        assert_eq!(mismatch.census().no_trees, Some(NoTrees::HeaderMismatch { tree_sectors_x: 3, tree_sectors_z: 2 }));
        assert!(mismatch.sector_trees(0, 0).is_empty());

        let one = [tree(0, 0, 0, 0, 0)];
        let cases: [(&str, Vec<u8>, Vec<u8>, &str); 7] = [
            ("empty sector file", vec![1, 2, 3], vec![], "shorter than its 8-byte header"),
            ("truncated table", sector_file(GRID, GRID, &[(0, 0), (0, 0)]), vec![], "shorter than the 40"),
            ("negative count", sector_file(GRID, GRID, &[(-1, 0), (0, 0), (0, 0), (0, 0)]), data_file(&one), "negative tree count -1"),
            ("negative offset", sector_file(GRID, GRID, &[(1, -1), (0, 0), (0, 0), (0, 0)]), data_file(&one), "negative record offset -1"),
            ("past the end", sector_file(GRID, GRID, &[(2, 0), (0, 0), (0, 0), (0, 0)]), data_file(&one), "reaches records 0 to 2 but the data file holds 1"),
            ("offset past the end", sector_file(GRID, GRID, &[(1, 1), (0, 0), (0, 0), (0, 0)]), data_file(&one), "reaches records 1 to 2"),
            (
                "i32 overflow",
                sector_file(GRID, GRID, &[(i32::MAX, i32::MAX), (0, 0), (0, 0), (0, 0)]),
                data_file(&one),
                "reaches records 2147483647",
            ),
        ];
        for (name, sectors, data, expected) in cases {
            let error = TreeScene::from_bytes(map(), &sectors, &data, ForestGeometry::KnownAbsent).err().unwrap_or_else(|| panic!("{name} was accepted"));
            assert!(error.contains(expected), "{name}: {error}");
        }
    }

    #[test]
    fn files_are_found_by_their_on_disk_spelling_and_absent_files_are_a_known_no_trees() {
        let base = std::env::temp_dir().join(format!("eech-tree-loader-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let record = [tree(0, 0, 0, 0, 0)];
        let sectors = sector_file(GRID, GRID, &[(1, 0), (0, 0), (0, 0), (0, 0)]);
        for (name, sector_name, data_name) in [("lower", "tree_pos.sec", "tree_pos.dat"), ("upper", "TREE_POS.SEC", "TREE_POS.DAT")] {
            let dir = base.join(name);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(sector_name), &sectors).unwrap();
            std::fs::write(dir.join(data_name), data_file(&record)).unwrap();
            let scene = TreeScene::load(&dir, map(), ForestGeometry::KnownAbsent).unwrap();
            assert_eq!(scene.census().sector_file.as_deref(), Some(sector_name));
            assert_eq!(scene.census().data_file.as_deref(), Some(data_name));
            assert_eq!(scene.sector_trees(0, 0), &record[..]);
        }
        let one_file = base.join("one");
        std::fs::create_dir_all(&one_file).unwrap();
        std::fs::write(one_file.join("tree_pos.sec"), &sectors).unwrap();
        let scene = TreeScene::load(&one_file, map(), ForestGeometry::KnownAbsent).unwrap();
        assert_eq!(scene.census().no_trees, Some(NoTrees::FilesAbsent { sector_file: true, data_file: false }));
        let neither = base.join("neither");
        std::fs::create_dir_all(&neither).unwrap();
        assert_eq!(
            TreeScene::load(&neither, map(), ForestGeometry::KnownAbsent).unwrap().census().no_trees,
            Some(NoTrees::FilesAbsent { sector_file: false, data_file: false })
        );
        // a case-sensitive file system can hold both spellings: that is ambiguous and refused (Windows cannot make the second file)
        let both = base.join("both");
        std::fs::create_dir_all(&both).unwrap();
        std::fs::write(both.join("tree_pos.sec"), &sectors).unwrap();
        if std::fs::write(both.join("TREE_POS.SEC"), &sectors).is_ok() && std::fs::read_dir(&both).unwrap().count() == 2 {
            std::fs::write(both.join("tree_pos.dat"), data_file(&record)).unwrap();
            let error = TreeScene::load(&both, map(), ForestGeometry::KnownAbsent).err().expect("two spellings are ambiguous");
            assert!(error.contains("more than one spelling"), "{error}");
        }
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn the_map_metadata_is_checked() {
        let bad: [(Result<TreeMap, String>, &str); 5] = [
            (TreeMap::new((0.0, f32::NAN, 0.0, 1.0), 1, 1, 0.0, 1.0), "finite"),
            (TreeMap::new((0.0, 1.0, 0.0, 1.0), 0, 1, 0.0, 1.0), "outside 1 to 4096"),
            (TreeMap::new((-1.0, 1.0, 0.0, 1.0), 1, 1, 0.0, 1.0), "0 <= min <= max"),
            (TreeMap::new((0.0, 2048.0, 0.0, 1.0), 1, 1, 0.0, 1.0), "beyond the terrain sector grid"),
            (TreeMap::new((2.0, 1.0, 0.0, 1.0), 1, 1, 0.0, 1.0), "0 <= min <= max"),
        ];
        for (result, expected) in bad {
            let error = result.err().expect("accepted");
            assert!(error.contains(expected), "{error}");
        }
    }

    // A tree record x = z = 0 stands at its sector's middle, 1024 m in from the corner of sector (0, 0); with y = 0, height 127 and a unit height
    // scale it stands on the ground at y = 0 and is (127 * (19 / 127) + 6) * 1.1 = 27.5 m tall (to float rounding).
    fn lone_tree() -> Arc<TreeScene> {
        scene_with(&[tree(0, 0, 0, 127, 0)], ForestGeometry::KnownAbsent)
    }

    #[test]
    fn a_segment_through_the_middle_of_a_tree_hits_its_plane_at_the_hand_derived_point() {
        // The segment runs along x at z = 1024 and y = 10 from x = 1000 to 1048 over flat ground at 0 m (radial altitude 10 m, under 35 m). dz is 0, so the
        // original's dx2 is 0 * 48 / 0, a NaN, and ddz = 0 - 0 = 0 takes the "within 5 m" test. The z-facing quad lies in the ray (zero denominator,
        // a miss); the x-facing quad's plane is x = 1024, crossed at mu = 0.5, point (1024, 10, 1024), with the normal from (p2 - p1) x (p3 - p1) =
        // (0, 27.5, 0) x (0, 27.5, 5), which is +x.
        let (answer, asked) = run(&lone_tree(), [1000.0, 10.0, 1024.0], [1048.0, 10.0, 1024.0], (false, true), |_, _| 0.0);
        let (hit, point, normal) = done(answer);
        assert!(hit);
        assert_eq!(point, [1024.0, 10.0, 1024.0]);
        assert_eq!(normal, [1.0, 0.0, 0.0]);
        assert_eq!(asked, vec![(Endpoint::Old, 1000.0, 1024.0), (Endpoint::New, 1048.0, 1024.0)]);
    }

    #[test]
    fn a_miss_returns_the_callers_point_and_normal_and_nothing_nearer_is_chosen_by_sorting() {
        // above the tree top (27.5 m): the intercept height 40 m is over the top, so no quad is tried
        let (answer, _) = run(&lone_tree(), [1000.0, 40.0, 1024.0], [1048.0, 40.0, 1024.0], (false, true), |_, _| 10.0);
        let (hit, point, normal) = done(answer);
        assert!(!hit);
        assert_eq!((point, normal), ([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]));
        // two trees on the line: the nearer to the old end wins whatever their order in the file (strictly nearer replaces)
        let near_first = scene_with(&[tree(-100, 0, 0, 127, 0), tree(100, 0, 0, 127, 0)], ForestGeometry::KnownAbsent);
        let far_first = scene_with(&[tree(100, 0, 0, 127, 0), tree(-100, 0, 0, 127, 0)], ForestGeometry::KnownAbsent);
        // x = -100 * 0.0625 * 0.5 = -3.125 m from the middle, +3.125 m for 100: trees at 1020.875 and 1027.125; both are crossed, the nearer to x = 1000 is first
        for scene in [near_first, far_first] {
            let (answer, _) = run(&scene, [1000.0, 10.0, 1024.0], [1048.0, 10.0, 1024.0], (false, true), |_, _| 0.0);
            let (hit, point, _) = done(answer);
            assert!(hit);
            assert_eq!(point, [1020.875, 10.0, 1024.0]);
        }
    }

    #[test]
    fn the_early_exits_leave_the_callers_outputs_and_ask_only_what_the_original_asks() {
        // radial altitude over 35 m at both ends: an immediate miss after the two lookups
        let (answer, asked) = run(&lone_tree(), [1000.0, 50.0, 1024.0], [1048.0, 50.0, 1024.0], (false, true), |_, _| 0.0);
        assert!(!done(answer).0);
        assert_eq!(asked.len(), 2);
        // a range under 0.1 m is a miss; the lookups are still asked first
        let (answer, asked) = run(&lone_tree(), [1024.0, 10.0, 1024.0], [1024.05, 10.0, 1024.0], (false, true), |_, _| 0.0);
        let (hit, point, normal) = done(answer);
        assert!(!hit && point == [1.0, 2.0, 3.0] && normal == [4.0, 5.0, 6.0]);
        assert_eq!(asked.len(), 2);
        // one end above 35 m, the other low: the LOWER radial altitude decides, so the scan goes on and finds the tree
        let (answer, _) = run(&lone_tree(), [1000.0, 10.0, 1024.0], [1048.0, 60.0, 1024.0], (false, true), |_, _| 0.0);
        let _ = done(answer);
    }

    #[test]
    fn end_points_outside_the_map_are_not_looked_up() {
        // the map is 0 to 4095 on x and z, inclusive, and y is ignored
        let (_, asked) = run(&lone_tree(), [-1.0, 10.0, 1024.0], [1048.0, 10.0, 1024.0], (false, true), |_, _| 0.0);
        assert_eq!(asked, vec![(Endpoint::New, 1048.0, 1024.0)]);
        let (_, asked) = run(&lone_tree(), [1000.0, 10.0, 1024.0], [4095.5, 10.0, 1024.0], (false, true), |_, _| 0.0);
        assert_eq!(asked, vec![(Endpoint::Old, 1000.0, 1024.0)]);
        let (_, asked) = run(&lone_tree(), [4095.0, -9000.0, 4095.0], [0.0, 1e9, 0.0], (false, true), |_, _| 0.0);
        assert_eq!(asked.len(), 2, "the bounds are inclusive and y is ignored");
        let (answer, asked) = run(&lone_tree(), [-5.0, 10.0, 1024.0], [-4000.0, 10.0, 1024.0], (false, true), |_, _| 0.0);
        assert!(asked.is_empty());
        assert!(!done(answer).0);
    }

    #[test]
    fn the_forest_edge_filter_and_the_tree_top_flag_do_what_the_original_text_says() {
        // a type 0 tree is skipped when ignore_forest_edge_trees is set; a type 1 tree is not
        let edge = scene_with(&[tree(0, 0, 0, 127, 0)], ForestGeometry::KnownAbsent);
        let interior = scene_with(&[tree(0, 0, 0, 127, 1)], ForestGeometry::KnownAbsent);
        for (scene, ignore, expected) in [(&edge, false, true), (&edge, true, false), (&interior, true, true)] {
            let (answer, _) = run(scene, [1000.0, 10.0, 1024.0], [1048.0, 10.0, 1024.0], (ignore, true), |_, _| 0.0);
            assert_eq!(done(answer).0, expected);
        }
        // a segment dropping onto the tree from above hits only the top quad: with skip_tree_top_test set it misses, without it it hits at the tree top
        // (the tree top is at y = 27.5 m to float rounding; the segment runs from y = 40 to y = 20 over 4 m of x at z = 1024, crossing the top at x = ...)
        let (answer, _) = run(&lone_tree(), [1022.0, 40.0, 1024.0], [1026.0, 20.0, 1024.0], (false, false), |_, _| 0.0);
        let with_top = done(answer);
        let (answer, _) = run(&lone_tree(), [1022.0, 40.0, 1024.0], [1026.0, 20.0, 1024.0], (false, true), |_, _| 0.0);
        let without_top = done(answer);
        assert!(with_top.0 || !without_top.0, "skipping the top test can only lose hits");
        if with_top.0 && !without_top.0 {
            assert!((with_top.1[1] - 27.5).abs() < 0.01, "the top quad is at the tree top: {:?}", with_top.1);
        }
    }

    #[test]
    fn a_type_one_tree_takes_the_geometry_branch_when_the_scene_has_triangles_and_returns_at_the_first_hit() {
        // one triangle in the plane x = 0 with points (0,0,0), (0,1,0), (0,0,1) in the tree's own units, scaled by the tree height (27.5) and moved to
        // the tree: (1024, 0, 1024), (1024, 27.5, 1024), (1024, 0, 1051.5). The segment from (1000, 5, 1030) to (1048, 5, 1030) crosses x = 1024 at mu =
        // 0.5, point (1024, 5, 1030), inside the triangle (y / 27.5 + (z - 1024) / 27.5 = 0.4 < 1), with the normal (+x) from the same cross product
        let triangle = vec![[[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]];
        let scene = scene_with(&[tree(0, 0, 0, 127, 1)], ForestGeometry::Triangles(triangle.clone()));
        let (answer, _) = run(&scene, [1000.0, 5.0, 1030.0], [1048.0, 5.0, 1030.0], (false, true), |_, _| 0.0);
        let (hit, point, normal) = done(answer);
        assert!(hit);
        for (got, want) in point.iter().zip([1024.0, 5.0, 1030.0]) {
            assert!((got - want).abs() < 0.001, "{point:?}");
        }
        for (got, want) in normal.iter().zip([1.0, 0.0, 0.0]) {
            assert!((got - want).abs() < 0.001, "{normal:?}");
        }
        // the same segment past the triangle's far corner (z = 1060, outside it) misses, and a type 0 tree never uses the triangles
        let (answer, _) = run(&scene, [1000.0, 5.0, 1060.0], [1048.0, 5.0, 1060.0], (false, true), |_, _| 0.0);
        assert!(!done(answer).0);
        let edge = scene_with(&[tree(0, 0, 0, 127, 0)], ForestGeometry::Triangles(triangle));
        let (answer, _) = run(&edge, [1000.0, 5.0, 1030.0], [1048.0, 5.0, 1030.0], (false, true), |_, _| 0.0);
        assert!(!done(answer).0, "off the tree's own quads (z = 1024 plus or minus 2.5 m) the quad path misses at z = 1030");
        // non-finite triangles are refused
        assert!(TreeScene::without_trees(map(), ForestGeometry::Triangles(vec![[[f32::NAN; 3]; 3]])).is_err());
    }

    #[test]
    fn answers_out_of_sequence_and_non_finite_inputs_are_refused() {
        let mut query = TreeCollision::new(lone_tree()).unwrap();
        assert!(query.terrain(0.0).unwrap_err().contains("no query open"));
        let first = query.begin([1000.0, 10.0, 1024.0], [1048.0, 10.0, 1024.0], [0.0; 3], [0.0; 3], false, true).unwrap();
        assert!(matches!(first, TreeRequest::NeedTerrain { endpoint: Endpoint::Old, .. }));
        assert!(query.terrain(f32::NAN).unwrap_err().contains("finite"));
        // a new begin replaces the open query: the next lookup asked is the first again
        let again = query.begin([1000.0, 10.0, 1024.0], [1048.0, 10.0, 1024.0], [0.0; 3], [0.0; 3], false, true).unwrap();
        assert_eq!(first, again);
        assert!(matches!(query.terrain(0.0).unwrap(), TreeRequest::NeedTerrain { endpoint: Endpoint::New, .. }));
        assert!(matches!(query.terrain(0.0).unwrap(), TreeRequest::Done { .. }));
        assert!(query.terrain(0.0).unwrap_err().contains("no query open"), "an answer after the query ended");
        query.begin([1000.0, 10.0, 1024.0], [1048.0, 10.0, 1024.0], [0.0; 3], [0.0; 3], false, true).unwrap();
        query.abandon();
        assert!(query.terrain(0.0).is_err());
        assert!(query.begin([f32::INFINITY, 0.0, 0.0], [0.0; 3], [0.0; 3], [0.0; 3], false, true).is_err());
        assert!(query.begin([0.0; 3], [0.0, f32::NAN, 0.0], [0.0; 3], [0.0; 3], false, true).is_err());
    }

    #[test]
    fn a_scene_whose_tables_do_not_cover_the_scan_is_a_fault_not_an_out_of_bounds_read() {
        // map metadata that says 2 by 2 sectors while the tree table has none is impossible to build through `from_bytes`; a consistent scene has every
        // sector the clamped scan can reach, so the whole-map scan is fine
        let scene = scene_with(&[tree(0, 0, 0, 127, 0)], ForestGeometry::KnownAbsent);
        let (answer, asked) = run(&scene, [0.0, 10.0, 0.0], [4095.0, 10.0, 4095.0], (false, true), |_, _| 0.0);
        assert_eq!(asked.len(), 2);
        let _ = done(answer);
    }

    #[test]
    fn the_floating_point_state_is_read_without_being_changed() {
        let first = fp_state().unwrap();
        let second = fp_state().unwrap();
        assert_eq!(first, second);
        assert_eq!((first.x87_masks, first.sse_masks), (63, 63), "a test thread has every exception masked, as the kernel requires");
    }
}
