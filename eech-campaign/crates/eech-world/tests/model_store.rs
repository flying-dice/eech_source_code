//! The host-owned collision-model store (EEDCS-50, src/models.rs) against minimal format fixtures.
//!
//! These are contract guards for the LOADER and the store, not an oracle for any retail model: the files are written here in the original's formats from tiny
//! hand-made objects, and what is asserted is that the loader reads what the formats say, refuses what they cannot hold, answers without depending on or disturbing
//! the caller's floating-point state, and keeps its data valid. The only numeric expectations are identities that do not need the original's formula: a scale of
//! exactly 1.0 returns the short coordinates unchanged, and a rounding-sensitive scale returns the same bits whatever rounding mode the caller is in. What the
//! retail data answers is the Root's comparison with the retained original report (the `model-host-compare` driver), never these tests.
use eech_world::models::{compiled_scene_name, fp_state, old_last, scene_last, ErrorKind, ModelStore, ObjectSource, QueryError, SceneSource, Triangle};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

// ---------------------------------------------------------------------------------------------------------------------------------
// the original's formats, written

#[derive(Default)]
struct W(Vec<u8>);

impl W {
    fn i32(&mut self, v: i32) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn u32(&mut self, v: u32) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn u16(&mut self, v: u16) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn f32(&mut self, v: f32) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn bytes(&mut self, v: &[u8]) -> &mut Self {
        self.0.extend_from_slice(v);
        self
    }
    fn floats(&mut self, v: &[f32]) -> &mut Self {
        for x in v {
            self.f32(*x);
        }
        self
    }
    fn string(&mut self, v: &str) -> &mut Self {
        self.0.extend_from_slice(v.as_bytes());
        self.0.push(0);
        self
    }
}

#[derive(Clone)]
struct Mesh {
    points: Vec<[i16; 3]>,
    bb: [f32; 6],
    bb2: [f32; 6],
    /// (flags word, faces)
    surfaces: Vec<(u32, u16)>,
    number_of_faces: i32,
    faces: Vec<u8>,
    list: Vec<u8>,
    sp: Vec<u16>,
}

fn polygon_flags(points: u32) -> u32 {
    1 | (points << 24)
}

/// a quad whose scale is exactly 1.0 (the bounding box's largest absolute value is 32767)
fn identity_quad() -> Mesh {
    Mesh {
        points: vec![[0, 0, 0], [32767, 0, 0], [32767, 32767, 0], [0, 32767, 0]],
        bb: [0.0, 32767.0, 0.0, 32767.0, 0.0, 32767.0],
        bb2: [0.0, 32767.0, 0.0, 32767.0, 0.0, 32767.0],
        surfaces: vec![(polygon_flags(4), 1)],
        number_of_faces: 1,
        faces: vec![4],
        list: vec![0, 1, 2, 3],
        sp: vec![0, 1, 2, 3],
    }
}

/// a quad whose scales are not exactly representable, so the bits depend on the rounding the arithmetic runs in
fn sensitive_quad() -> Mesh {
    Mesh {
        points: vec![[1, 12345, 30001], [-22222, 17, 9], [31000, -31000, 7], [123, 4567, -8901]],
        bb: [-10000.0, 9000.0, -12000.0, 11000.0, -9500.5, 8888.25],
        bb2: [-10000.0, 9000.0, -12000.0, 11000.0, -9500.5, 8888.25],
        surfaces: vec![(polygon_flags(4), 1)],
        number_of_faces: 1,
        faces: vec![4],
        list: vec![0, 1, 2, 3],
        sp: vec![0, 1, 2, 3],
    }
}

/// two faces by the header, one polygon in the face array: a polygon surface then a line surface (the mixed layout)
fn mixed_quad() -> Mesh {
    let mut mesh = identity_quad();
    mesh.surfaces = vec![(polygon_flags(4), 1), (4 << 24, 1)];
    mesh.number_of_faces = 2;
    mesh
}

enum Entry {
    Null,
    Mesh(Mesh),
}

struct Table {
    bin: Vec<u8>,
    pts: Vec<u8>,
    sp: Vec<u8>,
}

fn build_table(entries: &[Entry], declared_polygons: Option<i32>) -> Table {
    let (mut surfaces, mut points, mut plain, mut polygons) = (0, 0, 0, 0);
    for entry in entries {
        if let Entry::Mesh(m) = entry {
            surfaces += m.surfaces.len() as i32;
            points += m.points.len() as i32;
            plain += m.list.len() as i32;
            polygons += m.faces.len() as i32;
        }
    }
    let mut bin = W::default();
    bin.i32(entries.len() as i32)
        .i32(surfaces)
        .i32(points)
        .i32(0)
        .i32(plain)
        .i32(0)
        .i32(declared_polygons.unwrap_or(polygons));
    let (mut pts, mut sp) = (W::default(), W::default());
    for entry in entries {
        match entry {
            Entry::Null => {
                bin.i32(1);
            }
            Entry::Mesh(m) => {
                bin.i32(0)
                    .i32(m.points.len() as i32)
                    .i32(m.surfaces.len() as i32)
                    .i32(m.number_of_faces)
                    .i32(0)
                    .i32(0)
                    .i32(0);
                bin.f32(1.0).i32(m.sp.len() as i32).i32(0).i32(0);
                bin.floats(&m.bb).floats(&m.bb2);
                for (flags, faces) in &m.surfaces {
                    bin.u32(*flags).u16(0).u16(0).i32(*faces as i32).u32(0).bytes(&[0, 0]);
                }
                bin.i32(m.list.len() as i32).i32(0);
                bin.bytes(&m.faces).bytes(&m.list);
                for p in &m.points {
                    pts.0.extend_from_slice(&p[0].to_le_bytes());
                    pts.0.extend_from_slice(&p[1].to_le_bytes());
                    pts.0.extend_from_slice(&p[2].to_le_bytes());
                }
                for s in &m.sp {
                    sp.u16(*s);
                }
            }
        }
    }
    Table {
        bin: bin.0,
        pts: pts.0,
        sp: sp.0,
    }
}

fn keyframe(w: &mut W) {
    // index, linear, x y z, heading pitch roll, scale x y z, tension continuity bias
    w.i32(0).i32(0).floats(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0]);
}

/// one 3dobjdb.bin scene record with no cameras, links, lights or sub-objects
fn binary_scene(scene: i32, main: i32, collision: i32) -> Vec<u8> {
    let mut w = W::default();
    for _ in 0..16 {
        w.i32(-1);
    }
    w.i32(scene).i32(0); // scene_index, self_shadows
    w.i32(0).i32(0).i32(0).i32(0); // lights, cameras, links, sprite lights
    w.i32(0).i32(0).i32(0); // sub-objects, named sub-objects, texture animations
    w.i32(0).i32(main); // approximations, main object
    w.i32(0).i32(0).floats(&[1.0, 1.0, 1.0]); // shadow_approximation_index, shadow polygon object, scale
    w.i32(collision);
    w.i32(1);
    keyframe(&mut w); // the main object's keyframes
    w.f32(1.0).i32(0); // dissolve, its keyframes
    w.i32(0).i32(0); // displacement sequence index, its keyframes
    w.i32(0).i32(0); // named sub-object indices, sub-objects
    w.0
}

fn database(scenes: &[Vec<u8>]) -> Vec<u8> {
    let mut w = W::default();
    w.i32(scenes.len() as i32);
    for _ in 0..9 {
        w.i32(0);
    }
    for s in scenes {
        w.bytes(s);
    }
    w.0
}

fn bininfo() -> Vec<u8> {
    let mut w = W::default();
    w.i32(old_last() as i32).i32(0);
    for scene in 1..old_last() {
        let name = compiled_scene_name(scene).unwrap();
        w.i32(name.len() as i32 + 1).bytes(name.as_bytes()).bytes(&[0]).f32(0.0);
    }
    w.0
}

/// an .EEO object, version 1 or 2
fn eeo(version: i32, m: &Mesh) -> Vec<u8> {
    let mut w = W::default();
    w.i32(version).i32(m.points.len() as i32).i32(m.number_of_faces).i32(m.faces.len() as i32);
    if version > 1 {
        w.i32(m.faces.len() as i32);
    }
    w.i32(m.surfaces.len() as i32)
        .i32(0)
        .i32(0)
        .i32(m.list.len() as i32)
        .i32(0)
        .i32(m.sp.len() as i32)
        .i32(0);
    w.floats(&m.bb).floats(&m.bb2).f32(1.0);
    for p in &m.points {
        w.bytes(&p[0].to_le_bytes()).bytes(&p[1].to_le_bytes()).bytes(&p[2].to_le_bytes());
    }
    w.bytes(&m.faces).bytes(&m.list);
    for (flags, faces) in &m.surfaces {
        w.u32(*flags).u16(0).u16(0).u16(*faces).u16(0).u32(0);
    }
    w.bytes(&vec![0; 2 * m.faces.len()]); // face normals
    for s in &m.sp {
        w.u16(*s);
    }
    w.0
}

/// a named scene (.EES, version -1) whose main and collision objects are `object`.EEO beside it
fn ees(object: &str) -> Vec<u8> {
    let mut w = W::default();
    w.i32(-1); // version
    w.i32(0).i32(0).i32(0); // cameras, links, sprite lights
    w.i32(0).i32(0).i32(0); // sub-objects, named sub-objects, texture animations
    w.i32(0).string(object); // approximations, main object
    w.i32(0).string("").floats(&[1.0, 1.0, 1.0]); // shadow_approximation_index, shadow polygon object (none), scale
    w.string(object); // collision object
    w.i32(1);
    keyframe(&mut w);
    w.f32(1.0).i32(0); // dissolve, its keyframes
    w.i32(0); // displacement amplitude keyframes
    w.i32(0).i32(0); // named sub-object indices, sub-objects
    w.0
}

// ---------------------------------------------------------------------------------------------------------------------------------
// an installation on disk

struct Root {
    dir: PathBuf,
}

impl Root {
    fn new(table: &Table, scenes: &[Vec<u8>]) -> Root {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "eech-world-model-store-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let data = dir.join("cohokum").join("3ddata");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::write(data.join("bininfo.bin"), bininfo()).unwrap();
        std::fs::write(data.join("3dobjs.bin"), &table.bin).unwrap();
        std::fs::write(data.join("3dobjs.pts"), &table.pts).unwrap();
        std::fs::write(data.join("3dobjs.sp"), &table.sp).unwrap();
        std::fs::write(data.join("3dobjdb.bin"), database(scenes)).unwrap();
        Root { dir }
    }

    fn data(&self) -> PathBuf {
        self.dir.join("cohokum").join("3ddata")
    }

    fn write(&self, relative: &str, bytes: &[u8]) {
        let path = self.data().join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    fn load(&self) -> Result<ModelStore, eech_world::models::ModelError> {
        ModelStore::load(&self.dir)
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// objects: 1 identity quad, 2 null, 3 rounding-sensitive quad, 4 mixed layout (last, so the face array ends with it).
/// scenes: 1 -> object 1; 2 -> object 1 with the binary collision index 0; 3 -> object 3; 4 -> object 4; 5 -> main 1, collision 2 (a null object).
fn standard() -> Root {
    let table = build_table(
        &[
            Entry::Mesh(identity_quad()),
            Entry::Null,
            Entry::Mesh(sensitive_quad()),
            Entry::Mesh(mixed_quad()),
        ],
        None,
    );
    Root::new(
        &table,
        &[
            binary_scene(1, 1, 1),
            binary_scene(2, 1, 0),
            binary_scene(3, 3, 3),
            binary_scene(4, 4, 4),
            binary_scene(5, 1, 2),
        ],
    )
}

fn coords(bits: &[u32; 9]) -> [f32; 9] {
    let mut out = [0.0f32; 9];
    for (o, b) in out.iter_mut().zip(bits) {
        *o = f32::from_bits(*b);
    }
    out
}

// ---------------------------------------------------------------------------------------------------------------------------------
// the loader reads what the formats say

#[test]
fn the_store_is_immutable_shared_data() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<ModelStore>();
}

#[test]
fn a_fan_of_a_quad_is_two_triangles_in_the_originals_order_and_then_the_end() {
    let root = standard();
    let store = root.load().unwrap();
    // scale exactly 1.0: the answer is the short coordinates themselves
    let Triangle::Found(t0) = store.triangle(1, 0).unwrap() else {
        panic!("depth 0")
    };
    assert_eq!(coords(&t0), [0.0, 0.0, 0.0, 32767.0, 0.0, 0.0, 32767.0, 32767.0, 0.0]);
    let Triangle::Found(t1) = store.triangle(1, 1).unwrap() else {
        panic!("depth 1")
    };
    assert_eq!(coords(&t1), [0.0, 0.0, 0.0, 32767.0, 32767.0, 0.0, 0.0, 32767.0, 0.0]);
    assert_eq!(store.triangle(1, 2).unwrap(), Triangle::End);
    assert_eq!(store.triangle(1, i32::MAX as i64).unwrap(), Triangle::End);
}

#[test]
fn the_scene_view_carries_the_originals_two_levels_and_plain_values() {
    let root = standard();
    let store = root.load().unwrap();
    let scene = store.scene(1).unwrap();
    assert!(scene.loaded);
    assert_eq!((scene.main, scene.collision, scene.source), (1, 1, SceneSource::Table));
    assert_eq!(scene.name, compiled_scene_name(1).unwrap());
    // a scene with no sub-objects at the identity rotation has its main object's bounds, and a radius at least its largest extent
    // compared as floats: a rotation by the identity may leave a signed zero where the box has an unsigned one
    let expected = [0.0f32, 32767.0, 0.0, 32767.0, 0.0, 32767.0];
    assert_eq!(scene.bounding_box.map(f32::from_bits), expected);
    assert_eq!(scene.bounding_box2.map(f32::from_bits), expected);
    assert!(f32::from_bits(scene.radius) >= 32767.0);
    let object = scene.collision_object.unwrap();
    assert_eq!(
        (object.points, object.faces, object.surfaces, object.source, object.layout),
        (4, 1, 1, ObjectSource::Table, "polygon")
    );
    assert_eq!(object.bounding_box, [0.0f32, 32767.0, 0.0, 32767.0, 0.0, 32767.0].map(f32::to_bits));
    // the mixed object says so
    assert_eq!(store.scene(4).unwrap().collision_object.unwrap().layout, "mixed");
    let summary = store.summary();
    assert_eq!((summary.table_objects, summary.scenes_in_table, summary.scenes_loaded), (4, 5, 5));
}

#[test]
fn a_binary_collision_index_of_zero_means_none_and_the_original_answers_the_absence() {
    let root = standard();
    let store = root.load().unwrap();
    let scene = store.scene(2).unwrap();
    assert_eq!(scene.collision, -1);
    assert!(scene.collision_object.is_none());
    assert_eq!(store.triangle(2, 0).unwrap(), Triangle::Absent);
    assert_eq!(store.triangle(2, 12345).unwrap(), Triangle::Absent);
}

#[test]
fn a_scene_that_was_never_read_and_a_null_collision_object_are_refused_not_invented() {
    let root = standard();
    let store = root.load().unwrap();
    // scene 6 is in range but is in no file: the original's zeroed entry, whose accessor would dereference NULL surfaces
    assert!(!store.scene(6).unwrap().loaded);
    assert_eq!(store.triangle(6, 0).unwrap(), Triangle::Refused("no_scene"));
    // scene 5 names object 2, a null object: no surfaces
    assert_eq!(store.triangle(5, 0).unwrap(), Triangle::Refused("null_object"));
}

#[test]
fn arguments_outside_the_scenes_and_depths_are_refused_before_the_data_is_touched() {
    let root = standard();
    let store = root.load().unwrap();
    let last = scene_last() as i64;
    for scene in [0, -1, last, last + 1, i64::MAX, i64::MIN] {
        assert_eq!(store.triangle(scene, 0), Err(QueryError::BadScene(scene)), "scene {scene}");
        assert_eq!(store.scene(scene), Err(QueryError::BadScene(scene)));
    }
    for depth in [-1, i32::MIN as i64, i32::MAX as i64 + 1, i64::MAX, i64::MIN] {
        assert_eq!(store.triangle(1, depth), Err(QueryError::BadDepth(depth)), "depth {depth}");
    }
    // the last scene is in range
    assert!(store.scene(last - 1).is_ok());
}

#[test]
fn the_mixed_layout_is_walked_as_the_original_walks_it_and_stops_where_the_data_is_not_backed() {
    let root = standard();
    let store = root.load().unwrap();
    // the two triangles of the polygon face are backed
    assert!(matches!(store.triangle(4, 0).unwrap(), Triangle::Found(_)));
    assert!(matches!(store.triangle(4, 1).unwrap(), Triangle::Found(_)));
    // past them the original reads the face array beyond the object's one polygon: here that is past the end of the whole array, which the file does not back
    assert_eq!(store.triangle(4, 2).unwrap(), Triangle::Refused("unbacked_faces"));
}

// ---------------------------------------------------------------------------------------------------------------------------------
// the caller's floating-point state

#[cfg(target_arch = "x86_64")]
mod fpu {
    use std::arch::asm;

    pub fn mxcsr() -> u32 {
        let mut value: u32 = 0;
        let pointer = &mut value as *mut u32;
        // SAFETY: stmxcsr writes four bytes to the valid local.
        unsafe { asm!("stmxcsr [{0}]", in(reg) pointer, options(nostack)) };
        value
    }

    pub fn set_mxcsr(value: u32) {
        let pointer = &value as *const u32;
        // SAFETY: ldmxcsr reads four bytes from the valid local; the values used set only rounding control and sticky flags, with every exception masked.
        unsafe { asm!("ldmxcsr [{0}]", in(reg) pointer, options(nostack)) };
    }

    pub fn x87_control() -> u16 {
        let mut value: u16 = 0;
        let pointer = &mut value as *mut u16;
        // SAFETY: fnstcw writes two bytes to the valid local.
        unsafe { asm!("fnstcw [{0}]", in(reg) pointer, options(nostack)) };
        value
    }

    pub fn set_x87_control(value: u16) {
        let pointer = &value as *const u16;
        // SAFETY: fldcw reads two bytes from the valid local; the values used set only rounding control, with every exception masked.
        unsafe { asm!("fldcw [{0}]", in(reg) pointer, options(nostack)) };
    }
}

#[cfg(target_arch = "x86_64")]
#[test]
fn a_query_hands_the_callers_floating_point_state_back_bit_for_bit_and_does_not_depend_on_it() {
    let root = standard();
    let store = root.load().unwrap();
    let original_mxcsr = fpu::mxcsr();
    let original_x87 = fpu::x87_control();

    let reference: Vec<Triangle> = (0..2).map(|d| store.triangle(3, d).unwrap()).collect();
    assert!(reference.iter().all(|t| matches!(t, Triangle::Found(_))));
    let reference_scene = store.scene(3).unwrap();

    // every rounding mode the caller might be in, with sticky flags raised: nearest, down, up, toward zero
    for rounding in 0u32..4 {
        let mxcsr = 0x1f80 | (rounding << 13) | 0x21; // all masked, invalid and precision flags set
        let x87 = 0x037f | ((rounding as u16) << 10);
        fpu::set_mxcsr(mxcsr);
        fpu::set_x87_control(x87);
        let before = fp_state().unwrap();
        let answers: Vec<Triangle> = (0..2).map(|d| store.triangle(3, d).unwrap()).collect();
        let after_triangle = fp_state().unwrap();
        let scene = store.scene(3).unwrap();
        let after_scene = fp_state().unwrap();
        // restore the thread's own state before asserting, so a failure cannot leak a strange mode into this thread
        fpu::set_mxcsr(original_mxcsr);
        fpu::set_x87_control(original_x87);
        assert_eq!(before.3, x87 as u32, "the x87 control word the test set");
        assert_eq!(before.4, mxcsr, "the MXCSR the test set");
        assert_eq!(after_triangle, before, "rounding {rounding}: the state after a triangle query");
        assert_eq!(after_scene, before, "rounding {rounding}: the state after a scene query");
        assert_eq!(answers, reference, "rounding {rounding}: the answer depended on the caller's rounding mode");
        assert_eq!(scene, reference_scene);
    }
    assert_eq!(fpu::mxcsr(), original_mxcsr);
    assert_eq!(fpu::x87_control(), original_x87);
}

#[cfg(target_arch = "x86_64")]
#[test]
fn a_caller_with_an_exception_unmasked_is_refused_and_its_state_is_untouched() {
    let root = standard();
    let store = root.load().unwrap();
    let original_mxcsr = fpu::mxcsr();
    // unmask the SSE invalid-operation exception (bit 7): the original's arithmetic never runs with an exception unmasked
    let unmasked = original_mxcsr & !0x80;
    fpu::set_mxcsr(unmasked);
    let before = fp_state().unwrap();
    let answer = store.triangle(1, 0);
    let after = fp_state().unwrap();
    fpu::set_mxcsr(original_mxcsr);
    assert_eq!(answer.unwrap(), Triangle::Refused("fp_masks"));
    assert_eq!(before, after);
}

// ---------------------------------------------------------------------------------------------------------------------------------
// refusals: the loader never falls back to a synthetic or empty database

#[test]
fn a_missing_asset_is_reported_as_missing_not_replaced() {
    let root = standard();
    for name in ["bininfo.bin", "3dobjs.bin", "3dobjs.pts", "3dobjs.sp", "3dobjdb.bin"] {
        let backup = std::fs::read(root.data().join(name)).unwrap();
        std::fs::remove_file(root.data().join(name)).unwrap();
        let error = root.load().err().unwrap_or_else(|| panic!("{name} absent was accepted"));
        assert_eq!(error.kind, ErrorKind::Missing, "{name}: {error}");
        assert!(error.to_string().contains(name), "{error}");
        std::fs::write(root.data().join(name), backup).unwrap();
    }
    assert!(root.load().is_ok());
    // no installation at all
    let nowhere = std::env::temp_dir().join("eech-world-model-store-nowhere");
    assert_eq!(ModelStore::load(&nowhere).err().unwrap().kind, ErrorKind::Missing);
}

#[test]
fn a_truncated_file_is_refused_at_every_cut_with_its_name_and_offset() {
    let root = standard();
    for name in ["bininfo.bin", "3dobjs.bin", "3dobjdb.bin"] {
        let whole = std::fs::read(root.data().join(name)).unwrap();
        // cut inside the first records, at the middle and one byte short: each must refuse, none may be accepted as a smaller database
        let mut cuts = vec![0usize, 1, 3, 7, whole.len() / 2, whole.len() - 1];
        cuts.dedup();
        for cut in cuts {
            std::fs::write(root.data().join(name), &whole[..cut]).unwrap();
            let error = root.load().err().unwrap_or_else(|| panic!("{name} cut at {cut} was accepted"));
            assert!(
                matches!(error.kind, ErrorKind::Truncated | ErrorKind::Incompatible | ErrorKind::Inconsistent),
                "{name} at {cut}: {error}"
            );
        }
        std::fs::write(root.data().join(name), &whole).unwrap();
    }
    assert!(root.load().is_ok());
}

#[test]
fn counts_and_magic_the_formats_cannot_hold_are_refused() {
    let table = build_table(&[Entry::Mesh(identity_quad())], None);
    let scenes = vec![binary_scene(1, 1, 1)];

    // the scene file's magic
    let mut broken = scenes.clone();
    broken[0][0..4].copy_from_slice(&0i32.to_le_bytes());
    let root = Root::new(&table, &broken);
    assert_eq!(root.load().err().unwrap().kind, ErrorKind::Incompatible);

    // a scene index outside the scenes
    for index in [0, -3, scene_last() as i32, i32::MAX] {
        let root = Root::new(&table, &[binary_scene(index, 1, 1)]);
        assert_eq!(root.load().err().unwrap().kind, ErrorKind::Incompatible, "scene {index}");
    }

    // a main object outside the object table
    let root = Root::new(&table, &[binary_scene(1, 5000, 1)]);
    assert!(root.load().is_err());
    let root = Root::new(&table, &[binary_scene(1, -1, 1)]);
    assert!(root.load().is_err());

    // a negative count in the object table
    let mut bad = build_table(&[Entry::Mesh(identity_quad())], None);
    bad.bin[0..4].copy_from_slice(&(-1i32).to_le_bytes());
    assert_eq!(Root::new(&bad, &scenes).load().err().unwrap().kind, ErrorKind::Incompatible);

    // the header declares fewer polygon faces than the objects hold (the original would overrun its block)
    let short = build_table(&[Entry::Mesh(identity_quad())], Some(0));
    assert_eq!(Root::new(&short, &scenes).load().err().unwrap().kind, ErrorKind::Inconsistent);

    // the mapped point file is shorter than the objects use
    let mut cut = build_table(&[Entry::Mesh(identity_quad())], None);
    cut.pts.truncate(6);
    assert_eq!(Root::new(&cut, &scenes).load().err().unwrap().kind, ErrorKind::Inconsistent);

    // a scene count the file cannot hold
    let mut huge = database(&scenes);
    huge[0..4].copy_from_slice(&i32::MAX.to_le_bytes());
    let root = Root::new(&table, &scenes);
    std::fs::write(root.data().join("3dobjdb.bin"), huge).unwrap();
    assert!(root.load().is_err());
}

#[test]
fn a_scene_name_the_compiled_list_disagrees_with_is_refused() {
    let root = standard();
    let mut names = std::fs::read(root.data().join("bininfo.bin")).unwrap();
    // the first name starts after the two header ints and its own length int
    names[12] ^= 0x20;
    std::fs::write(root.data().join("bininfo.bin"), names).unwrap();
    assert_eq!(root.load().err().unwrap().kind, ErrorKind::Incompatible);
    // and a list of the wrong length
    let root = standard();
    let mut names = std::fs::read(root.data().join("bininfo.bin")).unwrap();
    names[0..4].copy_from_slice(&((old_last() - 1) as i32).to_le_bytes());
    std::fs::write(root.data().join("bininfo.bin"), names).unwrap();
    assert_eq!(root.load().err().unwrap().kind, ErrorKind::Incompatible);
}

// ---------------------------------------------------------------------------------------------------------------------------------
// overrides

#[test]
fn a_binary_hex_override_replaces_the_object_and_is_reported_custom() {
    let root = standard();
    let mut replacement = identity_quad();
    replacement.points = vec![[0, 0, 0], [100, 0, 0], [100, 200, 0], [0, 200, 0]];
    replacement.bb = [0.0, 32767.0, 0.0, 32767.0, 0.0, 32767.0];
    root.write("OBJECTS/0001.EEO", &eeo(1, &replacement));
    let store = root.load().unwrap();
    assert_eq!(store.summary().eeo_hex_applied, 1);
    let object = store.scene(1).unwrap().collision_object.unwrap();
    assert_eq!(object.source, ObjectSource::HexOverride);
    let Triangle::Found(t) = store.triangle(1, 0).unwrap() else { panic!() };
    assert_eq!(coords(&t), [0.0, 0.0, 0.0, 100.0, 0.0, 0.0, 100.0, 200.0, 0.0]);
    // scene 2 shares the object: the override is the object's, not the scene's
    assert!(matches!(store.triangle(3, 0).unwrap(), Triangle::Found(_)));
    // a lower-case file name is the same file on a case-insensitive system, and is found here
    let other = standard();
    other.write("OBJECTS/0001.eeo", &eeo(2, &replacement));
    assert_eq!(other.load().unwrap().summary().eeo_hex_applied, 1);
}

#[test]
fn an_override_the_original_would_ignore_or_reject_is_recorded_not_applied_quietly() {
    // an unsupported version: the original returns before touching the object
    let root = standard();
    root.write("OBJECTS/0001.EEO", &eeo(7, &identity_quad()));
    let store = root.load().unwrap();
    assert_eq!((store.summary().eeo_hex_applied, store.summary().eeo_hex_unsupported_version), (0, 1));
    assert_eq!(store.scene(1).unwrap().collision_object.unwrap().source, ObjectSource::Table);
    // a face of two points: the original reads it, rejects it afterwards and keeps the data; the store counts it and refuses to decode it
    let mut degenerate = identity_quad();
    degenerate.faces = vec![2];
    let root = standard();
    root.write("OBJECTS/0001.EEO", &eeo(1, &degenerate));
    let store = root.load().unwrap();
    assert_eq!(store.summary().eeo_invalid, 1);
    assert_eq!(store.triangle(1, 0).unwrap(), Triangle::Refused("a polygon face of fewer than 3 points"));
    // a truncated override is a refusal of the load
    let root = standard();
    let whole = eeo(1, &identity_quad());
    root.write("OBJECTS/0001.EEO", &whole[..whole.len() - 3]);
    assert_eq!(root.load().err().unwrap().kind, ErrorKind::Truncated);
    // two files for one object (possible only on a case-sensitive system) are ambiguous
    let root = standard();
    root.write("OBJECTS/0001.EEO", &eeo(1, &identity_quad()));
    root.write("OBJECTS/0001.eeo", &eeo(1, &identity_quad()));
    if std::fs::read_dir(root.data().join("OBJECTS")).unwrap().count() == 2 {
        assert_eq!(root.load().err().unwrap().kind, ErrorKind::Inconsistent);
    }
}

#[test]
fn a_named_scene_replaces_the_scene_and_numbers_its_objects_after_the_table() {
    let root = standard();
    let mut mesh = identity_quad();
    mesh.points = vec![[0, 0, 0], [5, 0, 0], [5, 6, 0], [0, 6, 0]];
    let scene_name = compiled_scene_name(1).unwrap();
    root.write(&format!("OBJECTS/MODDIR/{scene_name}.ees"), &ees("Chassis"));
    root.write("OBJECTS/MODDIR/chassis.eeo", &eeo(1, &mesh));
    let store = root.load().unwrap();
    assert_eq!((store.summary().ees_applied, store.summary().named_objects), (1, 1));
    let scene = store.scene(1).unwrap();
    assert_eq!(scene.source, SceneSource::Ees);
    // the first object past the table's four
    assert_eq!((scene.main, scene.collision), (5, 5));
    assert_eq!(scene.collision_object.unwrap().source, ObjectSource::Named);
    let Triangle::Found(t) = store.triangle(1, 0).unwrap() else { panic!() };
    assert_eq!(coords(&t), [0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 5.0, 6.0, 0.0]);
    // the other scenes are the table's
    assert_eq!(store.scene(3).unwrap().source, SceneSource::Table);
}

#[test]
fn a_named_scene_that_cannot_be_read_or_resolved_is_a_refusal_or_a_recorded_gap() {
    // its object file is missing: the original's load of the scene fails fatally
    let root = standard();
    root.write(&format!("OBJECTS/MODDIR/{}.EES", compiled_scene_name(1).unwrap()), &ees("NOSUCH"));
    assert_eq!(root.load().err().unwrap().kind, ErrorKind::Inconsistent);
    // a version the original does not read is skipped, and counted
    let root = standard();
    let mut other = ees("X");
    other[0..4].copy_from_slice(&(-9i32).to_le_bytes());
    root.write(&format!("OBJECTS/MODDIR/{}.EES", compiled_scene_name(1).unwrap()), &other);
    let store = root.load().unwrap();
    assert_eq!((store.summary().ees_unsupported_version, store.summary().ees_applied), (1, 0));
    // a name no scene has is counted as unresolved, not dropped silently
    let root = standard();
    root.write("OBJECTS/MODDIR/NOT_A_SCENE.EES", &ees("X"));
    let store = root.load().unwrap();
    assert_eq!(store.summary().ees_unresolved_names.len(), 1);
    // a truncated named scene is a refusal
    let root = standard();
    let whole = ees("X");
    root.write(&format!("OBJECTS/MODDIR/{}.EES", compiled_scene_name(1).unwrap()), &whole[..whole.len() - 4]);
    root.write("OBJECTS/MODDIR/X.EEO", &eeo(1, &identity_quad()));
    assert!(matches!(root.load().err().unwrap().kind, ErrorKind::Truncated | ErrorKind::Incompatible));
}

// ---------------------------------------------------------------------------------------------------------------------------------
// provenance of the compiled names

#[test]
fn the_compiled_scene_names_are_the_originals_header_when_the_header_is_here() {
    let header = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../modules/3d/3dmodels.h");
    let Ok(text) = std::fs::read_to_string(&header) else {
        eprintln!("NOT CHECKED: {} is not available here", header.display());
        return;
    };
    let mut expected = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("OBJECT_3D_INDEX(") {
            if let Some(name) = rest.split(')').next() {
                if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    expected.push(name.to_string());
                }
            }
        }
    }
    assert_eq!(expected.len() + 1, scene_last(), "the header lists a different number of scenes");
    for (i, name) in expected.iter().enumerate() {
        assert_eq!(compiled_scene_name(i + 1), Some(name.as_str()), "scene {}", i + 1);
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// the comparison driver (lua/model-host-compare.lua), run by the real host on the fixture installation
//
// The report it compares with is built here, in the original model driver's row format, from the HOST'S OWN first run: this proves the comparator's plumbing (it matches what is
// identical, it names and counts what is altered, it refuses what it cannot read), not what any model contains. The retail comparison is the Root's, against the retained
// original report.

const COMPARE_DRIVER: &str = include_str!("../lua/model-host-compare.lua");

/// The host process. An explicit `EECH_WORLD_BIN` names the one to run (for a test binary built elsewhere, whose baked-in Cargo path does not exist here): it must be an
/// existing file, and a bad one is an error, never a silent fall back. Without it, Cargo's own binary for this package. The environment is only read.
fn host_binary() -> PathBuf {
    match std::env::var_os("EECH_WORLD_BIN") {
        Some(value) => {
            let path = PathBuf::from(&value);
            assert!(
                !value.is_empty() && path.is_file(),
                "EECH_WORLD_BIN is set to {value:?}, which is not an existing file; refusing to fall back to the Cargo binary"
            );
            path
        }
        None => PathBuf::from(env!("CARGO_BIN_EXE_eech-world")),
    }
}

struct Driven {
    status: std::process::ExitStatus,
    stderr: String,
    rows: Vec<serde_json::Value>,
    out_existed: bool,
}

fn slashes(path: &Path) -> String {
    path.display().to_string().replace('\u{5c}', "/")
}

fn temp_dir(label: &str) -> PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "eech-world-model-host-{label}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn drive(root: &Path, extra: &[String], existing_out: bool) -> Driven {
    let dir = temp_dir("driver");
    let script = dir.join("model-host-compare.lua");
    std::fs::write(&script, COMPARE_DRIVER).unwrap();
    let out = dir.join("host-model.jsonl");
    if existing_out {
        std::fs::write(&out, "do not overwrite\n").unwrap();
    }
    let mut command = std::process::Command::new(host_binary());
    command
        .arg(&script)
        .arg(format!("root={}", slashes(root)))
        .arg(format!("out={}", slashes(&out)));
    for arg in extra {
        command.arg(arg);
    }
    let run = command.output().unwrap();
    let out_existed = out.exists();
    let rows = if out_existed && !existing_out {
        std::fs::read_to_string(&out)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("malformed output row {l:?}: {e}")))
            .collect()
    } else {
        Vec::new()
    };
    let _ = std::fs::remove_dir_all(&dir);
    Driven {
        status: run.status,
        stderr: String::from_utf8_lossy(&run.stderr).to_string(),
        rows,
        out_existed,
    }
}

fn rows_of<'a>(run: &'a Driven, event: &str) -> Vec<&'a serde_json::Value> {
    run.rows.iter().filter(|r| r["event"] == event).collect()
}

/// the host's rows as the original model driver would have written them (scene rows, then a geometry group per scene)
fn original_report(first: &Driven, scenes: &[i64]) -> Vec<serde_json::Value> {
    let mut report = Vec::new();
    for row in rows_of(first, "host-model-scene") {
        let mut row = row.clone();
        let status = match (row["loaded"].as_bool(), row["collision"].as_i64()) {
            (Some(false), _) => "no_scene",
            (_, Some(-1)) => "absent",
            _ => "safe",
        };
        row["event"] = "model-scene".into();
        row["status"] = status.into();
        report.push(row);
    }
    for scene in scenes {
        let tris: Vec<&serde_json::Value> = rows_of(first, "host-model-tri").into_iter().filter(|r| r["scene"] == *scene).collect();
        let absent = tris.first().is_some_and(|t| t["result"] == "absent");
        let status = if absent { "absent" } else { "safe" };
        report.push(serde_json::json!({"event": "model-geometry", "scene": scene, "status": status}));
        for t in tris {
            match t["result"].as_str().unwrap() {
                "triangle" => report.push(serde_json::json!({"event": "model-tri", "depth": t["depth"], "return": 1, "writes": 9, "p": t["p"]})),
                _ => report.push(serde_json::json!({"event": "model-tri", "depth": t["depth"], "return": 0, "writes": 0})),
            }
        }
    }
    report
}

fn write_report(rows: &[serde_json::Value]) -> (PathBuf, String) {
    let dir = temp_dir("report");
    let path = dir.join("original.jsonl");
    let text: String = rows.iter().map(|r| format!("{r}\n")).collect();
    std::fs::write(&path, text).unwrap();
    (dir, slashes(&path))
}

fn compare_args(path: &str) -> Vec<String> {
    vec!["catalogue=0".to_string(), format!("original={path}")]
}

type Alter = Box<dyn Fn(&mut [serde_json::Value])>;

fn alter(f: impl Fn(&mut [serde_json::Value]) + 'static) -> Alter {
    Box::new(f)
}

fn scene_row(rows: &mut [serde_json::Value], scene: i64) -> &mut serde_json::Value {
    rows.iter_mut().find(|x| x["event"] == "model-scene" && x["scene"] == scene).unwrap()
}

#[test]
fn the_driver_lists_the_store_and_the_comparison_matches_what_is_identical() {
    let root = standard();
    let first = drive(&root.dir, &["scenes=1,2,3".to_string()], false);
    assert!(first.status.success(), "{}", first.stderr);
    assert_eq!(rows_of(&first, "host-model-scene").len(), scene_last() - 1);
    assert_eq!(rows_of(&first, "done")[0]["failures"], 0);
    assert_eq!(rows_of(&first, "provenance")[0]["campaign_dll"], "not loaded");

    let report = original_report(&first, &[1, 2, 3]);
    let (dir, path) = write_report(&report);
    let same = drive(&root.dir, &compare_args(&path), false);
    let _ = std::fs::remove_dir_all(dir);
    assert!(same.status.success(), "{}", same.stderr);
    let compare = rows_of(&same, "compare")[0];
    assert!(
        compare["scene_rows_compared"].as_i64().unwrap() > 0 && compare["triangles_compared"].as_i64().unwrap() > 0,
        "{compare}"
    );
    assert_eq!(
        (compare["scene_mismatches"].as_i64(), compare["triangle_mismatches"].as_i64()),
        (Some(0), Some(0)),
        "{compare}"
    );
    assert_eq!(compare["absences_confirmed"], 1);
}

#[test]
fn the_comparison_finds_counts_and_fails_on_every_kind_of_alteration() {
    let root = standard();
    let first = drive(&root.dir, &["scenes=1,2,3".to_string()], false);
    assert!(first.status.success(), "{}", first.stderr);
    let report = original_report(&first, &[1, 2, 3]);

    let alterations: Vec<(&str, &str, Alter)> = vec![
        (
            "a flipped bit in one triangle word",
            "triangle_mismatches",
            alter(|r| {
                let tri = r.iter_mut().find(|x| x["event"] == "model-tri" && x["p"].is_string()).unwrap();
                let p = tri["p"].as_str().unwrap().to_string();
                let flipped = format!("0x{:08x}{}", u32::from_str_radix(&p[2..10], 16).unwrap() ^ 1, &p[10..]);
                tri["p"] = flipped.into();
            }),
        ),
        (
            "the original ends where the host has a triangle",
            "triangle_mismatches",
            alter(|r| {
                let tri = r.iter_mut().find(|x| x["event"] == "model-tri" && x["return"] == 1).unwrap();
                tri["return"] = 0.into();
            }),
        ),
        (
            "the original has a triangle where the host ends",
            "triangle_mismatches",
            alter(|r| {
                let tri = r.iter_mut().find(|x| x["event"] == "model-tri" && x["return"] == 0 && x["depth"] == 2).unwrap();
                tri["return"] = 1.into();
                tri["p"] = "0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000".into();
            }),
        ),
        (
            "a different radius",
            "scene_mismatches",
            alter(|r| scene_row(r, 1)["radius"] = "0x3f800000".into()),
        ),
        (
            "a different bounding box",
            "scene_mismatches",
            alter(|r| scene_row(r, 3)["bb2"] = "0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000".into()),
        ),
        (
            "a different collision object count",
            "scene_mismatches",
            alter(|r| scene_row(r, 1)["c_faces"] = 9.into()),
        ),
        (
            "a different name",
            "scene_mismatches",
            alter(|r| scene_row(r, 1)["name"] = "SOMETHING_ELSE".into()),
        ),
        (
            "the original has no scene where the host loaded one",
            "scene_mismatches",
            alter(|r| scene_row(r, 3)["status"] = "no_scene".into()),
        ),
        (
            "the original is absent where the host has a model",
            "scene_mismatches",
            alter(|r| scene_row(r, 3)["collision"] = (-1).into()),
        ),
    ];
    for (label, field, change) in alterations {
        let mut altered = report.clone();
        change(&mut altered[..]);
        let (dir, path) = write_report(&altered);
        let run = drive(&root.dir, &compare_args(&path), false);
        let _ = std::fs::remove_dir_all(dir);
        assert!(!run.status.success(), "{label}: accepted\n{}", run.stderr);
        let compare = rows_of(&run, "compare")[0];
        assert!(compare[field].as_i64().unwrap() >= 1, "{label}: {compare}");
        assert!(!rows_of(&run, "compare-mismatch").is_empty(), "{label}: nothing was listed");
        assert!(rows_of(&run, "done")[0]["failures"].as_i64().unwrap() >= 1, "{label}");
    }
}

#[test]
fn a_comparison_that_cannot_read_or_compare_anything_is_a_refusal_never_a_pass() {
    let root = standard();
    // an empty report, a report that is not JSON, and one with no model rows
    for (label, text) in [
        ("empty", ""),
        ("not JSON", "this is not a report\n"),
        ("no model rows", "{\"event\":\"boot\"}\n{\"event\":\"done\",\"failures\":0}\n"),
    ] {
        let dir = temp_dir("report");
        let path = dir.join("original.jsonl");
        std::fs::write(&path, text).unwrap();
        let run = drive(&root.dir, &["catalogue=0".to_string(), format!("original={}", slashes(&path))], false);
        let _ = std::fs::remove_dir_all(dir);
        assert!(!run.status.success(), "{label}: accepted\n{}", run.stderr);
        assert!(
            !rows_of(&run, "compare-refused").is_empty() || !rows_of(&run, "compare-empty").is_empty(),
            "{label}: {}",
            run.stderr
        );
    }
    // an original report that is not there
    let run = drive(&root.dir, &["catalogue=0".to_string(), "original=/no/such/report.jsonl".to_string()], false);
    assert!(!run.status.success() && !rows_of(&run, "compare-refused").is_empty(), "{}", run.stderr);
}

#[test]
fn the_driver_says_not_run_for_an_absent_asset_refuses_a_bad_store_and_never_overwrites() {
    // no installation: NOT RUN, the provenance row only, no done
    let nowhere = std::env::temp_dir().join("eech-world-model-host-nowhere");
    let run = drive(&nowhere, &[], false);
    assert!(!run.status.success() && run.stderr.contains("NOT RUN"), "{}", run.stderr);
    assert!(rows_of(&run, "done").is_empty() && rows_of(&run, "host-model-summary").is_empty());
    assert!(!rows_of(&run, "store-refused").is_empty());
    // a truncated asset is a refusal that names the file, not a smaller database
    let root = standard();
    let whole = std::fs::read(root.data().join("3dobjs.bin")).unwrap();
    std::fs::write(root.data().join("3dobjs.bin"), &whole[..whole.len() / 2]).unwrap();
    let run = drive(&root.dir, &[], false);
    assert!(
        !run.status.success() && run.stderr.contains("model store refused") && run.stderr.contains("3dobjs.bin"),
        "{}",
        run.stderr
    );
    assert!(rows_of(&run, "done").is_empty());
    // an output that exists is left as it is, and arguments are checked before anything is opened
    let root = standard();
    let run = drive(&root.dir, &[], true);
    assert!(!run.status.success() && run.stderr.contains("already exists"), "{}", run.stderr);
    for bad in [
        "catalogue=2",
        "max_triangles=0",
        "max_triangles=65537",
        "scenes=1,1",
        "scenes=0",
        "scenes=1.5",
        "scenes=abc",
    ] {
        let run = drive(&root.dir, &[bad.to_string()], false);
        assert!(!run.status.success(), "{bad} was accepted");
        assert!(!run.out_existed, "{bad}: the output was created");
    }
    let run = drive(&root.dir, &["catalogue=0".to_string()], false);
    assert!(!run.status.success() && !run.out_existed, "a run that asks nothing was accepted");
}

#[test]
fn a_scene_the_host_cannot_list_is_a_failure_and_the_cap_is_reported_not_hidden() {
    let root = standard();
    let run = drive(&root.dir, &["catalogue=0".to_string(), format!("scenes={}", scene_last())], false);
    assert!(!run.status.success());
    assert!(!rows_of(&run, "host-model-skipped").is_empty());
    let run = drive(
        &root.dir,
        &["catalogue=0".to_string(), "scenes=1".to_string(), "max_triangles=1".to_string()],
        false,
    );
    assert!(run.status.success(), "{}", run.stderr);
    assert_eq!(rows_of(&run, "host-model-capped").len(), 1);
    assert_eq!(rows_of(&run, "host-model-tri").len(), 1, "one triangle, then the cap");
}
