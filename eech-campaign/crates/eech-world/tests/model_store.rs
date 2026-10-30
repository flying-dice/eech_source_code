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
    // the wide positive values wrap to a valid scene (1, last - 1, 7) if they are narrowed to a 32-bit usize before the bound is checked: they must be refused as what they are
    let wide = 1i64 << 32;
    for scene in [
        0,
        -1,
        last,
        last + 1,
        i64::MAX,
        i64::MIN,
        wide + 1,
        wide + last - 1,
        wide + 7,
        wide * 3 + 1,
        i64::from(u32::MAX) + 2,
        i64::from(u32::MAX) + 1 + last,
    ] {
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
fn a_named_scene_replaces_the_scene_and_numbers_its_objects_from_the_tables_object_count() {
    // Provenance: modules/3d/3dobjdb.c get_next_object_id is `return total_number_of_objects++` and total_number_of_objects starts at the table's object COUNT (4 here), so the
    // FIRST named object is number 4 and REPLACES the last table object; the second is 5. (The retail audit found the first version of this loader one too high on all 61 named
    // objects and the consolidated surface block one too large.)
    let root = standard();
    let mut chassis = identity_quad();
    chassis.points = vec![[0, 0, 0], [5, 0, 0], [5, 6, 0], [0, 6, 0]];
    chassis.bb2 = [0.0, 5.0, 0.0, 6.0, 0.0, 0.0];
    let mut turret = identity_quad();
    turret.points = vec![[0, 0, 0], [7, 0, 0], [7, 8, 0], [0, 8, 0]];
    root.write(&format!("OBJECTS/MODDIR/{}.ees", compiled_scene_name(1).unwrap()), &ees("Chassis"));
    root.write(&format!("OBJECTS/MODDIR/{}.ees", compiled_scene_name(2).unwrap()), &ees("Turret"));
    root.write("OBJECTS/MODDIR/chassis.eeo", &eeo(1, &chassis));
    root.write("OBJECTS/MODDIR/turret.eeo", &eeo(1, &turret));
    let store = root.load().unwrap();
    assert_eq!((store.summary().ees_applied, store.summary().named_objects), (2, 2));
    let scene = store.scene(1).unwrap();
    assert_eq!(scene.source, SceneSource::Ees);
    assert_eq!(
        (scene.main, scene.collision),
        (4, 4),
        "the first named object takes the table's count as its number"
    );
    assert_eq!(scene.collision_object.unwrap().source, ObjectSource::Named);
    let Triangle::Found(t) = store.triangle(1, 0).unwrap() else { panic!() };
    assert_eq!(coords(&t), [0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 5.0, 6.0, 0.0]);
    assert_eq!(store.scene(1).unwrap().bounding_box2.map(f32::from_bits), [0.0, 5.0, 0.0, 6.0, 0.0, 0.0]);
    let second = store.scene(2).unwrap();
    assert_eq!((second.main, second.collision), (5, 5), "the second is the next number");
    let Triangle::Found(t) = store.triangle(2, 0).unwrap() else { panic!() };
    assert_eq!(coords(&t), [0.0, 0.0, 0.0, 7.0, 0.0, 0.0, 7.0, 8.0, 0.0]);

    // the first named object overwrote table object 4, which table scene 4 still names: it now reads the named object
    let table_scene = store.scene(4).unwrap();
    assert_eq!((table_scene.source, table_scene.main, table_scene.collision), (SceneSource::Table, 4, 4));
    assert_eq!(table_scene.collision_object.unwrap().source, ObjectSource::Named);
    // but the bounds of that scene were computed when it was read, from the object as it then was, and are not recomputed
    assert_eq!(table_scene.bounding_box2.map(f32::from_bits), [0.0, 32767.0, 0.0, 32767.0, 0.0, 32767.0]);

    // the consolidation runs to the counter's final value (4 + 2 = 6, a slot nobody holds): objects 1 and 3 (a surface each), the named 4 and 5 (a surface each)
    let summary = store.summary();
    assert_eq!((summary.table_objects, summary.objects_total, summary.surface_block), (4, 6, 4));
    // the other scenes are the table's
    assert_eq!(store.scene(3).unwrap().source, SceneSource::Table);
}

#[test]
fn with_no_named_object_the_last_table_object_is_kept_and_consolidated() {
    let root = standard();
    let store = root.load().unwrap();
    let summary = store.summary();
    // the loop runs to the counter, which stays at the table's count: objects 1 (1 surface), 3 (1) and 4 (2) are in the block
    assert_eq!(
        (summary.table_objects, summary.objects_total, summary.surface_block, summary.named_objects),
        (4, 4, 4, 0)
    );
    let last = store.scene(4).unwrap().collision_object.unwrap();
    assert_eq!((last.source, last.layout, last.surfaces), (ObjectSource::Table, "mixed", 2));
    assert!(matches!(store.triangle(4, 0).unwrap(), Triangle::Found(_)));
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

// A coherent original model-driver report, as the closed driver (lua/model-characterization.lua) writes one: its envelope (provenance, boot, the model state before, the
// summary, ..., the model state after, its two final checks and a done), then the catalogue pages with their checks and tallies when the catalogue is on, then a geometry
// group per scene asked for. It is built from the HOST'S OWN first run, so it proves the comparator's plumbing and contract, never what any model contains; the retail
// comparison is the Root's, against the retained original reports.

use serde_json::{json, Value};
use std::collections::BTreeMap;

/// one triangle row: depth, return, writes, the nine words when anything was written
type Tri = (i64, i64, i64, Option<String>);

/// a page header's safety facts: `fan_bound` and `end_read_backed` (1: the end read past the object is backed, 0: it is not)
type Gate = (i64, i64);

struct Spec {
    catalogue: bool,
    asked: i64,
    max_triangles: i64,
    page_triangles: i64,
}

fn state_row(phase: &str) -> Value {
    json!({"event": "model-state", "phase": phase, "tables": "0123456789abcdef", "pointers": "fedcba9876543210", "content": "00ff00ff00ff00ff", "content_objects": 3})
}

fn check_row(name: &str) -> Value {
    json!({"event": "check", "name": name, "pass": "true", "detail": ""})
}

/// a geometry page: its header, the triangle rows, and the terminal the original writes for it (`end`: reason and depth of a model-end and its closing row; None: a full page, closed)
fn page(scene: i64, status: &str, first: i64, count: i64, tris: &[Tri], end: Option<(&str, i64)>, gate: Gate) -> Vec<Value> {
    let collision: i64 = if status == "absent" { -1 } else { 1 };
    let (fan_bound, backed) = gate;
    let mut rows = vec![
        json!({"event": "model-geometry", "scene": scene, "name": "NAME", "collision": collision, "status": status, "detail": "-", "first": first, "count": count, "fan_bound": fan_bound, "end_read_backed": backed}),
    ];
    for (depth, returned, writes, p) in tris {
        let mut row = json!({"event": "model-tri", "depth": depth, "return": returned, "writes": writes});
        if let Some(p) = p {
            row["p"] = p.clone().into();
        }
        rows.push(row);
    }
    match status {
        "absent" => {}
        "safe" | "empty" => {
            if let Some((reason, depth)) = end {
                rows.push(json!({"event": "model-end", "scene": scene, "depth": depth, "reason": reason}));
            }
            rows.push(json!({"event": "model-geometry-end", "scene": scene, "calls": tris.len()}));
        }
        _ => rows.push(json!({"event": "model-refused", "scene": scene, "status": status, "detail": "-"})),
    }
    rows
}

/// a whole group of one callable scene on a single page of `page_triangles`: the true rows, the original's false, its end
fn group(scene: i64, tris: &[Tri], end: Option<(&str, i64)>, count: i64, gate: Gate) -> Vec<Value> {
    page(scene, "safe", 0, count, tris, end, gate)
}

/// the host's triangle rows for `scene` as the original's rows: a true per triangle, the false and its end where the host ends, a lone false for an absence
fn host_group(first: &Driven, scene: i64, count: i64) -> Vec<Value> {
    let results: Vec<&Value> = rows_of(first, "host-model-tri").into_iter().filter(|r| r["scene"] == scene).collect();
    if results[0]["result"] == "absent" {
        return page(scene, "absent", 0, count, &[(0, 0, 0, None)], None, (0, 0));
    }
    let mut tris: Vec<Tri> = Vec::new();
    let mut end = None;
    for r in results {
        let depth = r["depth"].as_i64().unwrap();
        match r["result"].as_str().unwrap() {
            "triangle" => tris.push((depth, 1, 9, Some(r["p"].as_str().unwrap().to_string()))),
            _ => {
                tris.push((depth, 0, 0, None));
                end = Some(("original_false", depth));
            }
        }
    }
    let fan = tris.iter().filter(|t| t.1 == 1).count() as i64;
    group(scene, &tris, end, count, (fan, 1))
}

/// the catalogue as the closed driver writes it: pages of 96 each followed by its check, then the tallies
fn catalogue_rows(first: &Driven) -> Vec<Value> {
    let scenes: Vec<Value> = rows_of(first, "host-model-scene")
        .into_iter()
        .map(|row| {
            let mut row = row.clone();
            let status = match (row["loaded"].as_bool(), row["collision"].as_i64()) {
                (Some(false), _) => "no_scene",
                (_, Some(-1)) => "absent",
                _ => "safe",
            };
            row["event"] = "model-scene".into();
            row["status"] = status.into();
            row
        })
        .collect();
    let mut out = Vec::new();
    let mut tally: BTreeMap<String, i64> = BTreeMap::new();
    let mut at = 0usize;
    while at < scenes.len() {
        let count = 96.min(scenes.len() - at);
        out.push(
            json!({"event": "model-page", "first": at + 1, "count": count, "scene_last": scenes.len() + 1, "entities_scanned": 0, "entities_with_shape": 0}),
        );
        for scene in &scenes[at..at + count] {
            *tally.entry(scene["status"].as_str().unwrap().to_string()).or_insert(0) += 1;
            out.push(scene.clone());
        }
        out.push(
            json!({"event": "check", "name": format!("catalogue page {} has {count} scenes", at + 1), "pass": "true", "detail": format!("found {count}")}),
        );
        at += count;
    }
    for (status, scenes) in tally {
        out.push(json!({"event": "catalogue-status", "status": status, "scenes": scenes}));
    }
    out
}

fn envelope(spec: &Spec, catalogue: Vec<Value>, geometry: Vec<Value>) -> Vec<Value> {
    let mut rows = vec![
        json!({"event": "provenance", "database": "retail"}),
        json!({"event": "boot", "catalogue": i32::from(spec.catalogue), "scenario": "georgia_retail", "scenes": spec.asked, "max_triangles": spec.max_triangles}),
        state_row("before"),
        json!({"event": "model-summary", "scene_last": scene_last(), "page_scenes": 96, "page_triangles": spec.page_triangles}),
    ];
    rows.extend(catalogue);
    rows.extend(geometry);
    rows.push(state_row("after"));
    rows.push(check_row("model tables unchanged"));
    rows.push(check_row("clock unchanged"));
    rows.push(json!({"event": "done", "failures": 0}));
    rows
}

const SPEC: Spec = Spec {
    catalogue: true,
    asked: 0,
    max_triangles: 4096,
    page_triangles: 256,
};

/// the host's first run (a catalogue and `scenes`' triangles) as a whole original report with the catalogue on
fn original_report(first: &Driven, scenes: &[i64]) -> Vec<Value> {
    let spec = Spec {
        asked: scenes.len() as i64,
        ..SPEC
    };
    let geometry = scenes.iter().flat_map(|s| host_group(first, *s, spec.page_triangles)).collect();
    envelope(&spec, catalogue_rows(first), geometry)
}

/// the same with the catalogue left off, as a selected-geometry run of the closed driver writes it
fn original_geometry_report(first: &Driven, scenes: &[i64]) -> Vec<Value> {
    let spec = Spec {
        catalogue: false,
        asked: scenes.len() as i64,
        ..SPEC
    };
    let geometry = scenes.iter().flat_map(|s| host_group(first, *s, spec.page_triangles)).collect();
    envelope(&spec, Vec::new(), geometry)
}

fn write_report(rows: &[Value]) -> (PathBuf, String) {
    let dir = temp_dir("report");
    let path = dir.join("original.jsonl");
    let text: String = rows.iter().map(|r| format!("{r}\n")).collect();
    std::fs::write(&path, text).unwrap();
    (dir, slashes(&path))
}

fn compare_args(path: &str) -> Vec<String> {
    vec!["catalogue=0".to_string(), format!("original={path}")]
}

type Alter = Box<dyn Fn(&mut Vec<Value>)>;

fn alter(f: impl Fn(&mut Vec<Value>) + 'static) -> Alter {
    Box::new(f)
}

fn scene_row(rows: &mut [Value], scene: i64) -> &mut Value {
    rows.iter_mut().find(|x| x["event"] == "model-scene" && x["scene"] == scene).unwrap()
}

/// the index range of the geometry rows of `scene`: its header up to, not including, the next header or the model state after
fn group_range(rows: &[Value], scene: i64) -> std::ops::Range<usize> {
    let start = rows.iter().position(|x| x["event"] == "model-geometry" && x["scene"] == scene).unwrap();
    let mut end = start + 1;
    while end < rows.len()
        && matches!(
            rows[end]["event"].as_str(),
            Some("model-tri" | "model-end" | "model-geometry-end" | "model-refused" | "model-capped")
        )
    {
        end += 1;
    }
    start..end
}

fn replace_group(rows: &mut Vec<Value>, scene: i64, with: Vec<Value>) {
    let range = group_range(rows, scene);
    rows.splice(range, with);
}

fn remove_event_of_group(rows: &mut Vec<Value>, scene: i64, event: &str) {
    let range = group_range(rows, scene);
    let at = rows[range].iter().position(|x| x["event"] == event).unwrap() + group_range(rows, scene).start;
    rows.remove(at);
}

fn first_index(rows: &[Value], event: &str) -> usize {
    rows.iter().position(|x| x["event"] == event).unwrap()
}

fn words(seed: u32) -> String {
    (0..9).map(|k| format!("0x{:08x}", 0x3f00_0000 + seed * 16 + k)).collect::<Vec<_>>().join(",")
}

/// nine words of which exactly two were written: what an original call that wrote only part of its buffer reports (the other seven are still the fixture's sentinel)
fn partial_words() -> String {
    [
        "0x3f000001",
        "0x3f000002",
        "0x7fc0a5a5",
        "0x7fc0a5a5",
        "0x7fc0a5a5",
        "0x7fc0a5a5",
        "0x7fc0a5a5",
        "0x7fc0a5a5",
        "0x7fc0a5a5",
    ]
    .join(",")
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

    // each alteration is a COHERENT report that says something different from the host: the contract passes it and the comparison names it
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
            alter(|r| replace_group(r, 1, group(1, &[(0, 0, 0, None)], Some(("original_false", 0)), 256, (2, 1)))),
        ),
        (
            "the original has a triangle where the host ends",
            "triangle_mismatches",
            alter(|r| {
                let range = group_range(r, 1);
                let ps: Vec<String> = r[range]
                    .iter()
                    .filter(|x| x["event"] == "model-tri" && x["p"].is_string())
                    .map(|x| x["p"].as_str().unwrap().to_string())
                    .collect();
                let tris = [
                    (0, 1, 9, Some(ps[0].clone())),
                    (1, 1, 9, Some(ps[1].clone())),
                    (2, 1, 9, Some(words(7))),
                    (3, 0, 0, None),
                ];
                replace_group(r, 1, group(1, &tris, Some(("original_false", 3)), 256, (2, 1)));
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
            "a different main object number",
            "scene_mismatches",
            alter(|r| scene_row(r, 1)["main"] = 2.into()),
        ),
        (
            "a different collision object number",
            "scene_mismatches",
            alter(|r| scene_row(r, 1)["collision"] = 2.into()),
        ),
        (
            "the original has no scene where the host loaded one",
            "scene_mismatches",
            alter(|r| {
                scene_row(r, 3)["status"] = "no_scene".into();
                retally(r, "safe", "no_scene");
            }),
        ),
        (
            "the original is absent where the host has a model",
            "scene_mismatches",
            alter(|r| scene_row(r, 3)["collision"] = (-1).into()),
        ),
    ];
    for (label, field, change) in alterations {
        let mut altered = report.clone();
        change(&mut altered);
        let (dir, path) = write_report(&altered);
        let run = drive(&root.dir, &compare_args(&path), false);
        let _ = std::fs::remove_dir_all(dir);
        assert!(!run.status.success(), "{label}: accepted\n{}", run.stderr);
        assert!(
            rows_of(&run, "compare-refused").is_empty(),
            "{label}: the contract refused a coherent report: {:?}",
            rows_of(&run, "compare-refused")
        );
        let compare = rows_of(&run, "compare")[0];
        assert!(compare[field].as_i64().unwrap() >= 1, "{label}: {compare}");
        assert!(!rows_of(&run, "compare-mismatch").is_empty(), "{label}: nothing was listed");
        assert!(rows_of(&run, "done")[0]["failures"].as_i64().unwrap() >= 1, "{label}");
    }
}

/// moves one scene from a status to another in the catalogue-status rows, as the original's tally would have it
fn retally(rows: &mut [Value], from: &str, to: &str) {
    for row in rows.iter_mut().filter(|x| x["event"] == "catalogue-status") {
        let delta = if row["status"] == from {
            -1
        } else if row["status"] == to {
            1
        } else {
            0
        };
        row["scenes"] = (row["scenes"].as_i64().unwrap() + delta).into();
    }
}

#[test]
fn an_incomplete_failed_or_incoherent_original_report_is_refused_before_anything_is_compared() {
    let root = standard();
    let first = drive(&root.dir, &["scenes=1,2,3".to_string()], false);
    assert!(first.status.success(), "{}", first.stderr);
    let report = original_report(&first, &[1, 2, 3]);

    // (label, how the coherent report is damaged, a fragment of the refusal)
    let damages: Vec<(&str, Alter, &str)> = vec![
        // the Root-named controls: cut off after the first scene, a repeated scene, a missing geometry terminal, a failed original done
        (
            "cut off after the first scene row",
            alter(|r| {
                let at = first_index(r, "model-scene");
                r.truncate(at + 1);
            }),
            "cut off",
        ),
        (
            "a scene row repeated (the count is right, the scenes are not)",
            alter(|r| {
                let at = first_index(r, "model-scene");
                let copy = r[at].clone();
                r.insert(at + 1, copy);
            }),
            "repeated, missing or out-of-order",
        ),
        (
            "a scene row missing from its page",
            alter(|r| {
                let at = first_index(r, "model-scene");
                r.remove(at + 1);
            }),
            "repeated, missing or out-of-order",
        ),
        (
            "two scene rows out of order",
            alter(|r| {
                let at = first_index(r, "model-scene");
                r.swap(at, at + 1);
            }),
            "repeated, missing or out-of-order",
        ),
        (
            "no geometry terminal: neither the end nor the closing row",
            alter(|r| {
                remove_event_of_group(r, 1, "model-end");
                remove_event_of_group(r, 1, "model-geometry-end");
            }),
            "before the previous page was closed",
        ),
        (
            "no closing row",
            alter(|r| remove_event_of_group(r, 1, "model-geometry-end")),
            "before the previous page was closed",
        ),
        (
            "no end row after a false return",
            alter(|r| remove_event_of_group(r, 1, "model-end")),
            "with no end row",
        ),
        (
            "the original's own done reports failures",
            alter(|r| {
                let last = r.len() - 1;
                r[last]["failures"] = 1.into();
            }),
            "no failures",
        ),
        (
            "an original check that did not pass",
            alter(|r| {
                let at = r.iter().rposition(|x| x["event"] == "check").unwrap();
                r[at]["pass"] = "false".into();
            }),
            "did not pass",
        ),
        (
            "no done row",
            alter(|r| {
                r.pop();
            }),
            "cut off",
        ),
        // the catalogue
        (
            "a catalogue page without its check",
            alter(|r| {
                let at = first_index(r, "check");
                r.remove(at);
            }),
            "is missing or did not pass",
        ),
        (
            "the catalogue declared but absent",
            alter(|r| {
                r.retain(|x| {
                    !matches!(x["event"].as_str(), Some("model-page" | "model-scene" | "catalogue-status"))
                        && !(x["event"] == "check" && x["name"].as_str().unwrap().starts_with("catalogue"))
                })
            }),
            "is missing",
        ),
        (
            "the catalogue declared off but present",
            alter(|r| {
                let at = first_index(r, "boot");
                r[at]["catalogue"] = 0.into();
            }),
            "unexpected",
        ),
        (
            "status tallies that are not the rows'",
            alter(|r| {
                let at = first_index(r, "catalogue-status");
                r[at]["scenes"] = (r[at]["scenes"].as_i64().unwrap() + 1).into();
            }),
            "status tallies",
        ),
        // the geometry
        (
            "a depth skipped",
            alter(|r| {
                let at = group_range(r, 1).start + 2;
                r[at]["depth"] = 5.into();
            }),
            "is due",
        ),
        (
            "a geometry group repeated",
            alter(|r| {
                let range = group_range(r, 1);
                let copy: Vec<Value> = r[range.clone()].to_vec();
                r.splice(range.end..range.end, copy);
            }),
            "continues after its terminal",
        ),
        (
            "fewer geometry groups than the report asked for",
            alter(|r| {
                let range = group_range(r, 3);
                r.drain(range);
            }),
            "asked for 3 geometry scenes",
        ),
        (
            "a closing row whose call count is not the page's",
            alter(|r| {
                let at = group_range(r, 1).end - 1;
                r[at]["calls"] = 99.into();
            }),
            "call count",
        ),
        (
            "an original_false with no false call before it",
            alter(|r| {
                let range = group_range(r, 1);
                let end = r[range.clone()].iter().position(|x| x["event"] == "model-end").unwrap() + range.start;
                r[end]["depth"] = 0.into();
            }),
            "original_false without an original call",
        ),
        // the fixture's own gate ends must agree with the header (the forgeries the Root reproduced against the 8a host: a backed scene whose calls are all replaced by an invented end)
        (
            "an invented end_probe_unbacked for a model whose end read is backed",
            alter(|r| replace_group(r, 1, page(1, "safe", 0, 256, &[], Some(("end_probe_unbacked", 0)), (2, 1)))),
            "end_probe_unbacked at depth 0 contradicts the header",
        ),
        (
            "an invented beyond_safe_bound inside the bound",
            alter(|r| replace_group(r, 1, page(1, "safe", 0, 256, &[], Some(("beyond_safe_bound", 0)), (2, 1)))),
            "beyond_safe_bound at depth 0 contradicts the header",
        ),
        (
            "an unbacked end that is not at the bound",
            alter(|r| {
                let tris = [(0, 1, 9, Some(words(1))), (1, 1, 9, Some(words(2)))];
                replace_group(r, 1, page(1, "safe", 0, 256, &tris, Some(("end_probe_unbacked", 2)), (5, 0)));
            }),
            "end_probe_unbacked at depth 2 contradicts the header",
        ),
        (
            "beyond_safe_bound at the unbacked end probe's own depth",
            alter(|r| {
                let tris = [(0, 1, 9, Some(words(1))), (1, 1, 9, Some(words(2)))];
                replace_group(r, 1, page(1, "safe", 0, 256, &tris, Some(("beyond_safe_bound", 2)), (2, 0)));
            }),
            "beyond_safe_bound at depth 2 contradicts the header",
        ),
        // the header's own facts
        (
            "an absent model with a collision object",
            alter(|r| {
                let at = group_range(r, 2).start;
                r[at]["collision"] = 5.into();
            }),
            "contradict each other",
        ),
        (
            "a callable model with no collision object",
            alter(|r| {
                let at = group_range(r, 1).start;
                r[at]["collision"] = (-1).into();
            }),
            "contradict each other",
        ),
        (
            "a negative fan_bound",
            alter(|r| {
                let at = group_range(r, 1).start;
                r[at]["fan_bound"] = (-1).into();
            }),
            "fan_bound",
        ),
        (
            "an end_read_backed that is neither 0 nor 1",
            alter(|r| {
                let at = group_range(r, 1).start;
                r[at]["end_read_backed"] = 2.into();
            }),
            "end_read_backed",
        ),
        (
            "a header without a name",
            alter(|r| {
                let at = group_range(r, 1).start;
                r[at]["name"] = "".into();
            }),
            "without a name",
        ),
        // the write count
        (
            "writes that is not the number of changed words",
            alter(|r| {
                let at = r.iter().position(|x| x["event"] == "model-tri" && x["p"].is_string()).unwrap();
                r[at]["writes"] = 8.into();
            }),
            "differ from the sentinel",
        ),
        (
            "a false that claims two writes with all nine words changed",
            alter(|r| {
                let tris = [(0, 1, 9, Some(words(1))), (1, 1, 9, Some(words(2))), (2, 0, 2, Some(words(3)))];
                replace_group(r, 1, group(1, &tris, Some(("original_false", 2)), 256, (2, 1)));
            }),
            "writes count 2 is not the 9 words",
        ),
        (
            "nine writes claimed with two words changed",
            alter(|r| {
                let tris = [(0, 1, 9, Some(partial_words())), (1, 1, 9, Some(words(2)))];
                replace_group(r, 1, group(1, &tris, None, 256, (2, 1)));
            }),
            "writes count 9 is not the 2 words",
        ),
        // the finite status set and the refusal's echo, as the closed driver requires (the three the Root reproduced against the 7a host: a ghost catalogue status with a matching tally,
        // a ghost geometry header with a ghost refusal that omitted a scene's calls, and a refusal whose detail differs from its header's)
        (
            "an unknown catalogue status with a tally that agrees with it",
            alter(|r| restatus(r, 1, "ghost")),
            "which the original driver does not write",
        ),
        (
            "an unknown geometry status with a refusal that echoes it",
            alter(|r| replace_group(r, 1, page(1, "ghost", 0, 256, &[], None, (0, 0)))),
            "which the original driver does not write",
        ),
        (
            "a refusal whose detail differs from its header's",
            alter(|r| {
                replace_group(r, 1, page(1, "custom_object", 0, 256, &[], None, (0, 0)));
                let at = r.iter().position(|x| x["event"] == "model-refused").unwrap();
                r[at]["detail"] = "faces".into();
            }),
            "(scene, status and detail)",
        ),
        (
            "a refusal without its detail",
            alter(|r| {
                replace_group(r, 1, page(1, "custom_object", 0, 256, &[], None, (0, 0)));
                let at = r.iter().position(|x| x["event"] == "model-refused").unwrap();
                r[at].as_object_mut().unwrap().remove("detail");
            }),
            "(scene, status and detail)",
        ),
        (
            "a refusal whose detail is not text",
            alter(|r| {
                replace_group(r, 1, page(1, "custom_object", 0, 256, &[], None, (0, 0)));
                let at = r.iter().position(|x| x["event"] == "model-refused").unwrap();
                r[at]["detail"] = 5.into();
            }),
            "(scene, status and detail)",
        ),
        (
            "a refusal that names another status than its header",
            alter(|r| {
                replace_group(r, 1, page(1, "custom_object", 0, 256, &[], None, (0, 0)));
                let at = r.iter().position(|x| x["event"] == "model-refused").unwrap();
                r[at]["status"] = "face_mismatch".into();
            }),
            "(scene, status and detail)",
        ),
        // what the report is about
        (
            "another scene count than this host's",
            alter(|r| {
                let at = first_index(r, "model-summary");
                r[at]["scene_last"] = 3000.into();
            }),
            "not this host's",
        ),
        ("a synthetic database", alter(|r| r[0]["database"] = "synthetic".into()), "retail database"),
        (
            "a model state after that is not the one before",
            alter(|r| {
                let at = r.iter().rposition(|x| x["event"] == "model-state").unwrap();
                r[at]["tables"] = "ffffffffffffffff".into();
            }),
            "after is not the one before",
        ),
    ];
    for (label, change, fragment) in damages {
        let mut damaged = report.clone();
        change(&mut damaged);
        let (dir, path) = write_report(&damaged);
        let run = drive(&root.dir, &compare_args(&path), false);
        let _ = std::fs::remove_dir_all(dir);
        assert!(!run.status.success(), "{label}: accepted\n{}", run.stderr);
        assert!(rows_of(&run, "done").is_empty(), "{label}: a done row after the refusal");
        let refused = rows_of(&run, "compare-refused");
        assert_eq!(refused.len(), 1, "{label}: expected the report to be refused\n{}", run.stderr);
        let message = refused[0]["message"].as_str().unwrap();
        assert!(message.contains(fragment), "{label}: refused as {message:?}, not for {fragment:?}");
        assert!(rows_of(&run, "compare").is_empty(), "{label}: a comparison was made of a refused report");
    }
}

#[test]
fn the_legitimate_original_modes_are_accepted_and_compared_as_they_are() {
    let root = standard();
    let first = drive(&root.dir, &["scenes=1,2,3".to_string()], false);
    assert!(first.status.success(), "{}", first.stderr);
    let p = |scene: i64, depth: i64| -> Option<String> {
        rows_of(&first, "host-model-tri")
            .into_iter()
            .find(|r| r["scene"] == scene && r["depth"] == depth)
            .and_then(|r| r["p"].as_str().map(str::to_string))
    };
    let (p0, p1) = (p(1, 0).unwrap(), p(1, 1).unwrap());
    // a true triangle row of the nine words in `text` (the lookups above give owned strings)
    let tri = |depth: i64, text: &str| (depth, 1, 9, Some(text.to_string()));
    let accepted = |label: &str, report: Vec<Value>, expect: &dyn Fn(&Value)| {
        let (dir, path) = write_report(&report);
        let run = drive(&root.dir, &compare_args(&path), false);
        let _ = std::fs::remove_dir_all(dir);
        assert!(run.status.success(), "{label}: refused\n{}", run.stderr);
        let compare = rows_of(&run, "compare")[0];
        assert_eq!(
            (compare["scene_mismatches"].as_i64(), compare["triangle_mismatches"].as_i64()),
            (Some(0), Some(0)),
            "{label}: {compare}"
        );
        expect(compare);
    };

    // the catalogue left off: a selected-geometry run, as the closed driver writes it
    accepted("catalogue off", original_geometry_report(&first, &[1, 2, 3]), &|c| {
        assert_eq!(
            (
                c["scene_rows"].as_i64(),
                c["original_geometry_scenes"].as_i64(),
                c["original_catalogue_scenes"].as_i64()
            ),
            (Some(0), Some(3), Some(0))
        );
        assert!(c["triangles_compared"].as_i64().unwrap() > 0);
    });

    // an explicit refusal by the original, beside a group that is compared: the refused model has no triangles to compare, and the host's answer for it is listed, not counted as a match
    let spec = Spec {
        catalogue: false,
        asked: 2,
        ..SPEC
    };
    let mut refused = host_group(&first, 1, spec.page_triangles);
    refused.extend(page(5, "custom_object", 0, 256, &[], None, (0, 0)));
    accepted("a refused model", envelope(&spec, Vec::new(), refused), &|c| {
        assert_eq!(c["original_refused_scenes"], 1);
        assert!(c["triangles_compared"].as_i64().unwrap() > 0);
    });

    // a walk that reached the cap exactly: a full page and the capped row
    let spec = Spec {
        catalogue: false,
        asked: 1,
        max_triangles: 2,
        ..SPEC
    };
    let mut capped = group(1, &[tri(0, &p0), tri(1, &p1)], None, 2, (2, 1));
    capped.push(json!({"event": "model-capped", "scene": 1, "calls": 2, "max_triangles": 2}));
    accepted("a capped walk", envelope(&spec, Vec::new(), capped), &|c| {
        assert_eq!(c["triangles_compared"], 2)
    });

    // the unbacked end probe: the end the fixture reports at the depth reached, with no false call
    let spec = Spec {
        catalogue: false,
        asked: 1,
        ..SPEC
    };
    let unbacked = group(1, &[tri(0, &p0), tri(1, &p1)], Some(("end_probe_unbacked", 2)), 256, (2, 0));
    accepted("an unbacked end", envelope(&spec, Vec::new(), unbacked), &|c| {
        assert_eq!(c["triangles_compared"], 2)
    });

    // a false return that nevertheless wrote part of its buffer is an observation of the original, not a malformed report
    let partial = group(
        1,
        &[tri(0, &p0), tri(1, &p1), (2, 0, 2, Some(partial_words()))],
        Some(("original_false", 2)),
        256,
        (2, 1),
    );
    accepted("a false that wrote", envelope(&spec, Vec::new(), partial), &|c| {
        assert_eq!(c["triangles_compared"], 3)
    });

    // a scene walked in several pages (the driver pages by page_triangles)
    let spec = Spec {
        catalogue: false,
        asked: 1,
        page_triangles: 1,
        ..SPEC
    };
    let mut paged = page(1, "safe", 0, 1, &[tri(0, &p0)], None, (2, 1));
    paged.extend(page(1, "safe", 1, 1, &[tri(1, &p1)], None, (2, 1)));
    paged.extend(page(1, "safe", 2, 1, &[(2, 0, 0, None)], Some(("original_false", 2)), (2, 1)));
    accepted("three pages", envelope(&spec, Vec::new(), paged), &|c| {
        assert_eq!((c["triangles_compared"].as_i64(), c["geometry_headers"].as_i64()), (Some(3), Some(3)))
    });
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

#[test]
fn the_comparison_reports_a_shifted_named_object_number_it_does_not_skip_named_objects() {
    // The first version of the loader numbered the first named object one higher than the original (N + 1 for N table objects, where the original gives N) and the comparison
    // skipped every named object number, so the retail audit found what the comparison could not. A same-input reference report is compared number for number.
    let root = standard();
    root.write(&format!("OBJECTS/MODDIR/{}.ees", compiled_scene_name(1).unwrap()), &ees("Chassis"));
    root.write("OBJECTS/MODDIR/chassis.eeo", &eeo(1, &identity_quad()));
    let first = drive(&root.dir, &["scenes=1".to_string()], false);
    assert!(first.status.success(), "{}", first.stderr);
    let host_scene = rows_of(&first, "host-model-scene").into_iter().find(|r| r["scene"] == 1).unwrap();
    assert_eq!((host_scene["main"].as_i64(), host_scene["collision"].as_i64()), (Some(4), Some(4)));

    let report = original_report(&first, &[1]);
    let (dir, path) = write_report(&report);
    let same = drive(&root.dir, &compare_args(&path), false);
    let _ = std::fs::remove_dir_all(dir);
    assert!(same.status.success(), "{}", same.stderr);

    let mut shifted = report.clone();
    scene_row(&mut shifted, 1)["main"] = 5.into();
    scene_row(&mut shifted, 1)["collision"] = 5.into();
    let (dir, path) = write_report(&shifted);
    let run = drive(&root.dir, &compare_args(&path), false);
    let _ = std::fs::remove_dir_all(dir);
    assert!(!run.status.success(), "a named object number one too high was accepted\n{}", run.stderr);
    let listed: Vec<String> = rows_of(&run, "compare-mismatch")
        .iter()
        .map(|r| r["text"].as_str().unwrap().to_string())
        .collect();
    assert!(listed.iter().any(|t| t.contains("main object original 5 host 4")), "{listed:?}");
    assert!(listed.iter().any(|t| t.contains("collision object original 5 host 4")), "{listed:?}");
}

// ---------------------------------------------------------------------------------------------------------------------------------
// sub-object counts the bytes cannot hold, and override directories that cannot be enumerated

/// a binary scene record whose header declares `declared` sub-objects and whose top-level sub-object count is `count`, followed by `tail`
fn binary_scene_with_nodes(scene: i32, main: i32, collision: i32, declared: i32, count: i32, tail: &[u8]) -> Vec<u8> {
    let mut w = W::default();
    for _ in 0..16 {
        w.i32(-1);
    }
    w.i32(scene).i32(0);
    w.i32(0).i32(0).i32(0).i32(0);
    w.i32(declared).i32(0).i32(0); // sub-objects, named sub-objects, texture animations
    w.i32(0).i32(main);
    w.i32(0).i32(0).floats(&[1.0, 1.0, 1.0]);
    w.i32(collision);
    w.i32(1);
    keyframe(&mut w);
    w.f32(1.0).i32(0);
    w.i32(0).i32(0);
    w.i32(0).i32(count); // named sub-object indices, sub-objects
    w.bytes(tail);
    w.0
}

/// one serialized sub-object of the smallest size the format allows (binary: 96 bytes; keyframe 0 at the origin, no children), with `children` as its child count
fn minimal_node(object: i32, children: i32) -> Vec<u8> {
    let mut w = W::default();
    w.i32(object).i32(1).i32(0).i32(0).i32(0); // object, contributes to collisions, the two approximation levels, no limits
    w.i32(1);
    keyframe(&mut w);
    w.f32(1.0).i32(0); // dissolve, its keyframes
    w.i32(0).i32(children); // named indices, sub-objects
    w.0
}

fn table_root(scenes: &[Vec<u8>]) -> Root {
    Root::new(&build_table(&[Entry::Mesh(identity_quad())], None), scenes)
}

#[test]
fn a_sub_object_count_the_remaining_bytes_cannot_hold_is_refused_before_any_node_is_allocated() {
    // Each of these files is a few hundred bytes. The first version resized its node vector to the file's count (52 bytes a node: about 111 GB for INT_MAX) before it looked at what
    // remained to read; now the count is checked against the smallest record the format allows first, and nothing is sized from it.
    let huge = binary_scene_with_nodes(1, 1, 1, i32::MAX, i32::MAX, &[]);
    assert!(huge.len() < 1024);
    let error = table_root(&[huge]).load().err().unwrap();
    assert_eq!(error.kind, ErrorKind::Truncated, "{error}");
    assert!(error.detail.contains("of at least 96 bytes"), "{error}");

    // a count that merely exceeds what the tail holds
    let two = binary_scene_with_nodes(1, 1, 1, 2, 2, &minimal_node(1, 0));
    let error = table_root(&[two]).load().err().unwrap();
    assert_eq!(error.kind, ErrorKind::Truncated, "{error}");

    // the header declares plenty and the count is huge but only one record follows
    let lie = binary_scene_with_nodes(1, 1, 1, i32::MAX, i32::MAX, &minimal_node(1, 0));
    assert_eq!(table_root(&[lie]).load().err().unwrap().kind, ErrorKind::Truncated);

    // a child count that cannot be held, inside a node that can: refused at the child level
    let nested = binary_scene_with_nodes(1, 1, 1, i32::MAX, 1, &minimal_node(1, i32::MAX));
    let error = table_root(&[nested]).load().err().unwrap();
    assert_eq!(error.kind, ErrorKind::Truncated, "{error}");

    // the same for a named scene: its smallest record is smaller (an empty name is one byte), so the bound is 93 for version -1
    let mut named = ees("Chassis");
    let at = named.len() - 8; // the named sub-object indices count and the sub-object count
    named[at + 4..at + 8].copy_from_slice(&i32::MAX.to_le_bytes());
    named[16..20].copy_from_slice(&i32::MAX.to_le_bytes()); // the header's sub-object total (after the version and three counts)
    let root = standard();
    root.write(&format!("OBJECTS/MODDIR/{}.ees", compiled_scene_name(1).unwrap()), &named);
    root.write("OBJECTS/MODDIR/chassis.eeo", &eeo(1, &identity_quad()));
    let error = root.load().err().unwrap();
    assert_eq!(error.kind, ErrorKind::Truncated, "{error}");
    assert!(error.detail.contains("of at least 93 bytes"), "{error}");
}

#[test]
fn a_sub_object_of_exactly_the_smallest_record_still_loads_and_keeps_its_place() {
    // the bound is derived from the format, not chosen: a real record of the minimum size is accepted, in order, with its children after it
    let one = binary_scene_with_nodes(1, 1, 1, 1, 1, &minimal_node(1, 0));
    let store = table_root(&[one]).load().unwrap();
    let scene = store.scene(1).unwrap();
    assert!(scene.loaded);
    // a node at the origin with no rotation and a scale of 1 adds the object's own box again: the scene's box is unchanged
    assert_eq!(scene.bounding_box.map(f32::from_bits), [0.0, 32767.0, 0.0, 32767.0, 0.0, 32767.0]);

    // a parent and one child, 96 bytes each, with the header's total exactly the two
    let mut tail = minimal_node(1, 1);
    tail.extend_from_slice(&minimal_node(1, 0));
    let two = binary_scene_with_nodes(1, 1, 1, 2, 1, &tail);
    assert!(table_root(&[two]).load().unwrap().scene(1).unwrap().loaded);

    // the header declaring fewer nodes than the records say is still the original's overrun: refused as inconsistent, after the bytes were shown to be there
    let mut tail = minimal_node(1, 1);
    tail.extend_from_slice(&minimal_node(1, 0));
    let short = binary_scene_with_nodes(1, 1, 1, 1, 1, &tail);
    assert_eq!(table_root(&[short]).load().err().unwrap().kind, ErrorKind::Inconsistent);
}

#[test]
fn an_override_directory_that_exists_but_is_not_a_directory_is_an_error_and_an_absent_one_is_not() {
    // genuinely absent: a valid installation without OBJECTS (the standard fixture has none)
    assert!(standard().load().is_ok());
    // present as a plain file: the install cannot be trusted to have no overrides
    let root = standard();
    root.write("OBJECTS", b"not a directory");
    let error = root.load().err().unwrap();
    assert_eq!(error.kind, ErrorKind::Inconsistent, "{error}");
    assert!(error.file.contains("OBJECTS") && error.detail.contains("not a directory"), "{error}");
    // present under another case and read through the resolver: still found and applied
    let root = standard();
    root.write("Objects/0001.EEO", &eeo(1, &identity_quad()));
    assert_eq!(root.load().unwrap().summary().eeo_hex_applied, 1);
}

#[test]
fn reserved_siblings_are_charged_against_the_same_bytes_as_every_later_list() {
    // The first correction checked each list's count against the bytes that remained, and charged nothing for the siblings already reserved and still to be read, so one stretch of
    // padding could justify a reservation at every level of a deep chain. Here the top list holds 4 nodes (384 bytes) in a tail of 768 bytes; the first node's 4 children also fit
    // (3 siblings pending + 4 children = 672 bytes of the 672 left: exactly full); the first child's 4 children do not (6 pending + 4 = 960 of 576), and are refused AT THE
    // RESERVATION, as unaccounted padding, not later as a malformed record. Nothing is allocated for them.
    let mut tail = minimal_node(1, 4);
    tail.extend_from_slice(&minimal_node(1, 4));
    tail.extend_from_slice(&[0u8; 576]);
    assert_eq!(tail.len(), 8 * 96);
    let scene = binary_scene_with_nodes(1, 1, 1, i32::MAX, 4, &tail);
    let error = table_root(&[scene]).load().err().unwrap();
    assert_eq!(error.kind, ErrorKind::Truncated, "{error}");
    assert!(error.detail.contains("(6 reserved siblings still pending)"), "{error}");

    // the same holds down a chain: nodes that each declare a long child list (100) over a tail that holds only 160 records are refused at the first child list the remaining bytes, with the
    // siblings already pending, cannot hold
    let mut chain = Vec::new();
    for _ in 0..40 {
        chain.extend_from_slice(&minimal_node(1, 100));
    }
    chain.extend_from_slice(&[0u8; 96 * 120]);
    let scene = binary_scene_with_nodes(1, 1, 1, i32::MAX, 100, &chain);
    let error = table_root(&[scene]).load().err().unwrap();
    assert_eq!(error.kind, ErrorKind::Truncated, "{error}");
    assert!(error.detail.contains("reserved siblings still pending"), "{error}");
}

#[test]
fn genuine_dense_and_nested_minimum_records_still_load_in_order() {
    // five minimum records in the parser's depth-first order: A (2 children), A1 (1 child), A11, A2, then the top list's second node B. Every reservation fits its bytes EXACTLY
    // (the third list is 3 nodes' worth over 3 nodes' worth of bytes), so a budget that charged even one byte too much would refuse this.
    let mut tail = minimal_node(1, 2);
    for children in [1, 0, 0, 0] {
        tail.extend_from_slice(&minimal_node(1, children));
    }
    let scene = binary_scene_with_nodes(1, 1, 1, 5, 2, &tail);
    assert!(table_root(&[scene]).load().unwrap().scene(1).unwrap().loaded);

    // a chain ten deep, one child each, no padding
    let mut chain = Vec::new();
    for _ in 0..9 {
        chain.extend_from_slice(&minimal_node(1, 1));
    }
    chain.extend_from_slice(&minimal_node(1, 0));
    let scene = binary_scene_with_nodes(1, 1, 1, 10, 1, &chain);
    assert!(table_root(&[scene]).load().unwrap().scene(1).unwrap().loaded);

    // one byte short of the dense tree is refused (the last record is cut), never accepted as a smaller tree
    let mut short = minimal_node(1, 2);
    for children in [1, 0, 0, 0] {
        short.extend_from_slice(&minimal_node(1, children));
    }
    short.pop();
    let scene = binary_scene_with_nodes(1, 1, 1, 5, 2, &short);
    assert_eq!(table_root(&[scene]).load().err().unwrap().kind, ErrorKind::Truncated);
}

#[test]
fn a_scenes_header_that_changes_between_pages_is_refused() {
    let root = standard();
    let first = drive(&root.dir, &["scenes=1".to_string()], false);
    assert!(first.status.success(), "{}", first.stderr);
    let word = |depth: i64| -> String {
        rows_of(&first, "host-model-tri")
            .into_iter()
            .find(|r| r["scene"] == 1 && r["depth"] == depth)
            .and_then(|r| r["p"].as_str().map(str::to_string))
            .unwrap()
    };
    let (p0, p1) = (word(0), word(1));
    let spec = Spec {
        catalogue: false,
        asked: 1,
        page_triangles: 1,
        ..SPEC
    };
    let paged = || {
        let mut rows = page(1, "safe", 0, 1, &[(0, 1, 9, Some(p0.clone()))], None, (2, 1));
        rows.extend(page(1, "safe", 1, 1, &[(1, 1, 9, Some(p1.clone()))], None, (2, 1)));
        rows.extend(page(1, "safe", 2, 1, &[(2, 0, 0, None)], Some(("original_false", 2)), (2, 1)));
        envelope(&spec, Vec::new(), rows)
    };
    // the unchanged three pages are accepted
    let (dir, path) = write_report(&paged());
    let run = drive(&root.dir, &compare_args(&path), false);
    let _ = std::fs::remove_dir_all(dir);
    assert!(run.status.success(), "{}", run.stderr);

    // the second page's header says something else about the same model
    let changes: Vec<(&str, &str, Value)> = vec![
        ("fan_bound", "fan_bound", 3.into()),
        ("end_read_backed", "end_read_backed", 0.into()),
        ("name", "name", "OTHER".into()),
        ("detail", "detail", "faces".into()),
        ("status", "status", "empty".into()),
        ("collision", "collision", 7.into()),
    ];
    for (label, field, value) in changes {
        let mut report = paged();
        let second = report
            .iter()
            .enumerate()
            .filter(|(_, x)| x["event"] == "model-geometry")
            .map(|(at, _)| at)
            .nth(1)
            .unwrap();
        report[second][field] = value;
        let (dir, path) = write_report(&report);
        let run = drive(&root.dir, &compare_args(&path), false);
        let _ = std::fs::remove_dir_all(dir);
        assert!(!run.status.success(), "{label}: accepted\n{}", run.stderr);
        let refused = rows_of(&run, "compare-refused");
        assert_eq!(refused.len(), 1, "{label}: {}", run.stderr);
        let message = refused[0]["message"].as_str().unwrap();
        assert!(message.contains("changed between pages"), "{label}: refused as {message:?}");
    }
}

/// moves one scene to another status in the catalogue and in the status tallies, so that the tallies agree with the scene rows (a tally of none is dropped, a first of a kind added)
fn restatus(rows: &mut Vec<Value>, scene: i64, to: &str) {
    let from = scene_row(rows, scene)["status"].as_str().unwrap().to_string();
    scene_row(rows, scene)["status"] = to.into();
    if from == to {
        return;
    }
    let at_from = rows
        .iter()
        .position(|x| x["event"] == "catalogue-status" && x["status"] == from.as_str())
        .unwrap();
    let n = rows[at_from]["scenes"].as_i64().unwrap();
    if n == 1 {
        rows.remove(at_from);
    } else {
        rows[at_from]["scenes"] = (n - 1).into();
    }
    match rows.iter().position(|x| x["event"] == "catalogue-status" && x["status"] == to) {
        Some(at) => rows[at]["scenes"] = (rows[at]["scenes"].as_i64().unwrap() + 1).into(),
        None => {
            let last = rows.iter().rposition(|x| x["event"] == "catalogue-status").unwrap();
            rows.insert(last + 1, json!({"event": "catalogue-status", "status": to, "scenes": 1}));
        }
    }
}

/// the fifteen verdicts the closed driver (lua/model-characterization.lua, `statuses`) accepts
const DRIVER_STATUSES: [&str; 15] = [
    "absent",
    "no_scene",
    "invalid_index",
    "null_surfaces",
    "surfaces_outside_block",
    "custom_object",
    "face_mismatch",
    "zero_face_surface",
    "null_arrays",
    "degenerate_face",
    "capacity_unobservable",
    "capacity_short",
    "index_violation",
    "empty",
    "safe",
];

#[test]
fn every_status_the_original_driver_writes_is_accepted_in_a_catalogue_and_as_a_refusal() {
    let root = standard();
    let first = drive(&root.dir, &["scenes=1".to_string()], false);
    assert!(first.status.success(), "{}", first.stderr);
    let healthy = original_report(&first, &[1]);

    for status in DRIVER_STATUSES {
        // as a catalogue status (with the tallies agreeing): the contract accepts it; whether the host agrees with the scene is the comparison's business, so a mismatch is allowed here
        let mut report = healthy.clone();
        restatus(&mut report, 3, status);
        let (dir, path) = write_report(&report);
        let run = drive(&root.dir, &compare_args(&path), false);
        let _ = std::fs::remove_dir_all(dir);
        assert!(
            rows_of(&run, "compare-refused").is_empty(),
            "{status} as a catalogue status was refused: {:?}\n{}",
            rows_of(&run, "compare-refused"),
            run.stderr
        );

        // as a geometry refusal beside a compared group: every status that is not the absence or a callable model is a refusal, with its detail echoed
        if status != "absent" && status != "safe" && status != "empty" {
            let spec = Spec {
                catalogue: false,
                asked: 2,
                ..SPEC
            };
            let mut geometry = host_group(&first, 1, spec.page_triangles);
            geometry.extend(page(5, status, 0, 256, &[], None, (0, 0)));
            let (dir, path) = write_report(&envelope(&spec, Vec::new(), geometry));
            let run = drive(&root.dir, &compare_args(&path), false);
            let _ = std::fs::remove_dir_all(dir);
            assert!(run.status.success(), "{status} as a refusal was not accepted\n{}", run.stderr);
            assert_eq!(rows_of(&run, "compare")[0]["original_refused_scenes"], 1, "{status}");
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// The reference host's model provider (EEDCS-50, D3): `host.model` and `host.model_provider`, what a script passes to `eech_dc.boot` as `model`. The campaign asks the host
// for the catalogue, every scene's radius and bounds, and every step of a collision-triangle cursor; the provider answers from the same immutable store the comparison
// tests above exercise, as exact bit patterns. These tests drive the real host binary on the synthetic installation; they need no DLL and say nothing about any retail model.

const PROVIDER_CHECK: &str = include_str!("../lua/model-provider-check.lua");
const JSON_LINES: &str = include_str!("../lua/json-lines.lua");
const JSON_LINES_CHECK: &str = include_str!("../lua/json-lines-check.lua");
const MODEL_AUTHORITY: &str = include_str!("../lua/model-authority.lua");

fn provider_rows(root: &Path) -> Vec<Value> {
    let dir = temp_dir("provider");
    let script = dir.join("model-provider-check.lua");
    std::fs::write(&script, PROVIDER_CHECK).unwrap();
    std::fs::write(dir.join("json-lines.lua"), JSON_LINES).unwrap();
    let out = dir.join("provider.jsonl");
    let run = std::process::Command::new(host_binary())
        .arg(&script)
        .arg(format!("root={}", slashes(root)))
        .arg(format!("out={}", slashes(&out)))
        .output()
        .unwrap();
    assert!(run.status.success(), "the provider check failed\n{}", String::from_utf8_lossy(&run.stderr));
    let rows = std::fs::read_to_string(&out)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("malformed output row {l:?}: {e}")))
        .collect();
    let _ = std::fs::remove_dir_all(&dir);
    rows
}

#[test]
fn the_reference_provider_answers_exactly_what_the_store_answers_by_both_routes() {
    let root = standard();
    let store = root.load().unwrap();
    let rows = provider_rows(&root.dir);
    for via in ["lazy", "store"] {
        let of = |event: &str| -> Vec<&Value> { rows.iter().filter(|r| r["event"] == event && r["via"] == via).collect() };

        let catalogue = of("catalogue");
        assert_eq!(catalogue.len(), 1, "{via}");
        assert_eq!(
            (catalogue[0]["ok"] == true, catalogue[0]["scenes"].as_u64()),
            (true, Some(scene_last() as u64)),
            "{via}: {}",
            catalogue[0]["error"]
        );

        for row in of("scene") {
            let scene = row["scene"].as_i64().unwrap();
            assert_eq!(row["ok"], true, "{via} scene {scene}: {}", row["error"]);
            if scene == 0 {
                // the null scene: the zeroed entry the original kept for the invalid object index, authored by the host
                assert_eq!(
                    (row["name"].as_str(), row["loaded"].as_bool(), row["collider"].as_bool(), row["radius"].as_str()),
                    (Some(""), Some(false), Some(false), Some("0x00000000")),
                    "{via}"
                );
                assert_eq!(
                    (row["bb"].as_str(), row["bb2"].as_str()),
                    (
                        Some("0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000"),
                        Some("0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000")
                    ),
                    "{via}"
                );
                continue;
            }
            let view = store.scene(scene).unwrap();
            // the public declaration: the compiled name, whether the host has a model, whether it has a collider; the store's own private numbering is only a reference
            // diagnostic beside them
            assert_eq!(row["name"].as_str(), Some(view.name), "{via} scene {scene}");
            assert_eq!(row["loaded"].as_bool(), Some(view.loaded), "{via} scene {scene}");
            assert_eq!(row["collider"].as_bool(), Some(view.loaded && view.collision != -1), "{via} scene {scene}");
            assert_eq!(row["reference_main"].as_i64(), Some(i64::from(view.main)), "{via} scene {scene}");
            assert_eq!(row["reference_collision"].as_i64(), Some(i64::from(view.collision)), "{via} scene {scene}");
            assert_eq!(row["radius"].as_str(), Some(format!("0x{:08x}", view.radius).as_str()), "{via} scene {scene}");
            assert_eq!(
                row["bb"].as_str(),
                Some(eech_world::models::hex_list(&view.bounding_box).as_str()),
                "{via} scene {scene}"
            );
            assert_eq!(
                row["bb2"].as_str(),
                Some(eech_world::models::hex_list(&view.bounding_box2).as_str()),
                "{via} scene {scene}"
            );
        }

        let mut kinds = BTreeMap::new();
        for row in of("triangle") {
            let (scene, depth) = (row["scene"].as_i64().unwrap(), row["depth"].as_i64().unwrap());
            if scene == 0 {
                // scene 0 has no collision triangles to ask for: the campaign never asks, and a host asked anyway refuses
                assert_eq!(row["ok"], false, "{via}");
                continue;
            }
            assert_eq!(row["ok"], true, "{via} scene {scene} depth {depth}: {}", row["error"]);
            let (kind, expected_points, expected_reason) = match store.triangle(scene, depth).unwrap() {
                Triangle::Found(bits) => ("triangle", Some(eech_world::models::hex_list(&bits)), None),
                Triangle::End => ("end", None, None),
                Triangle::Absent => ("absent", None, None),
                Triangle::Refused(reason) => ("refused", None, Some(reason)),
            };
            assert_eq!(row["result"].as_str(), Some(kind), "{via} scene {scene} depth {depth}");
            assert_eq!(
                row["p"].as_str().filter(|p| !p.is_empty()),
                expected_points.as_deref(),
                "{via} scene {scene} depth {depth}"
            );
            assert_eq!(
                row["reason"].as_str().filter(|p| !p.is_empty()),
                expected_reason,
                "{via} scene {scene} depth {depth}"
            );
            *kinds.entry(kind).or_insert(0usize) += 1;
        }
        // the synthetic installation has all four kinds of answer: a triangle, the end of a list, the absence of a collider and a refusal (a null collision object); a
        // legitimate absence is a different answer from a refusal
        for kind in ["triangle", "end", "absent", "refused"] {
            assert!(kinds.get(kind).copied().unwrap_or(0) > 0, "{via}: no {kind} answer in {kinds:?}");
        }
    }
}

#[test]
fn the_reference_provider_refuses_every_malformed_request_by_name_before_the_store_is_asked() {
    let root = standard();
    let rows = provider_rows(&root.dir);
    let bad: Vec<&Value> = rows.iter().filter(|r| r["event"] == "bad_request").collect();
    // twenty-two requests through each of the two routes
    assert_eq!(bad.len(), 44, "{bad:?}");
    for row in bad {
        assert_eq!(row["ok"], false, "{row}");
        let error = row["error"].as_str().unwrap();
        assert!(error.contains("[model-provider:"), "an unnamed refusal: {row}");
    }
}

#[test]
fn a_root_without_model_assets_refuses_the_provider_as_not_run_never_as_an_empty_database() {
    let dir = temp_dir("provider-missing");
    let run = {
        let script = dir.join("model-provider-check.lua");
        std::fs::write(&script, PROVIDER_CHECK).unwrap();
        std::fs::write(dir.join("json-lines.lua"), JSON_LINES).unwrap();
        std::process::Command::new(host_binary())
            .arg(&script)
            .arg(format!("root={}", slashes(&dir.join("no-such-installation"))))
            .arg(format!("out={}", slashes(&dir.join("out.jsonl"))))
            .output()
            .unwrap()
    };
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(!run.status.success(), "a root with no model assets was accepted\n{stderr}");
    assert!(stderr.contains("NOT RUN") || stderr.contains("model store refused"), "{stderr}");
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------------------------------------------------------------
// The witness scripts of the host-owned model data (EEDCS-50, D3): the strict JSON Lines writer, the finite and bounded arguments of model-authority.lua, and the propagation of a
// failed requested probe, a stuck clock and a lost output. They run the REAL scripts under the real host binary. The authority script needs the campaign DLL only to boot; here
// a stand-in `eech_dc` (package.preload) plays the engine, asks the real provider the questions the engine asks at boot, and answers the fixture methods, so what is tested
// is the script's own validation, bookkeeping, output and verdict. It says nothing about what the campaign does with a model; that is the native run's.

struct ScriptRun {
    status: std::process::ExitStatus,
    stderr: String,
    out: PathBuf,
    dir: PathBuf,
}

impl ScriptRun {
    fn rows(&self) -> Vec<Value> {
        std::fs::read_to_string(&self.out)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("malformed output row {l:?}: {e}")))
            .collect()
    }

    fn row(&self, event: &str) -> Value {
        self.rows()
            .into_iter()
            .find(|r| r["event"] == event)
            .unwrap_or_else(|| panic!("no {event} row in the output\n{}", self.stderr))
    }

    /// A number of the `event` row decoded from the row's own text with Rust's correctly rounded `f64` parse (the default serde_json decoder is not exact for every
    /// 17-digit double). The writer's keys are sorted and its numbers are plain, so the number ends at the next comma or the closing brace.
    fn raw_number(&self, event: &str, key: &str) -> f64 {
        let text = std::fs::read_to_string(&self.out).unwrap();
        let line = text
            .lines()
            .find(|l| l.contains(&format!("\"event\":\"{event}\"")))
            .unwrap_or_else(|| panic!("no {event} row in the output\n{}", self.stderr));
        let marker = format!("\"{key}\":");
        let start = line.find(&marker).unwrap_or_else(|| panic!("no {key} in {line}")) + marker.len();
        let end = line[start..].find([',', '}']).map_or(line.len(), |i| start + i);
        line[start..end]
            .parse::<f64>()
            .unwrap_or_else(|e| panic!("{key} in {line} is not a number: {e}"))
    }
}

impl Drop for ScriptRun {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Writes `files` into a fresh directory and runs the host binary on `main` with `args`. `{dir}` in an argument is that directory; the output is `{dir}/out.jsonl` unless an
/// `out=` argument names another.
fn run_lua(files: &[(&str, &str)], main: &str, args: &[String]) -> ScriptRun {
    let dir = temp_dir("witness");
    for (name, text) in files {
        std::fs::write(dir.join(name), text).unwrap();
    }
    let out = dir.join("out.jsonl");
    let mut command = std::process::Command::new(host_binary());
    command.arg(dir.join(main));
    if !args.iter().any(|a| a.starts_with("out=")) {
        command.arg(format!("out={}", slashes(&out)));
    }
    for arg in args {
        command.arg(arg.replace("{dir}", &slashes(&dir)));
    }
    let run = command.output().unwrap();
    ScriptRun {
        status: run.status,
        stderr: String::from_utf8_lossy(&run.stderr).to_string(),
        out,
        dir,
    }
}

#[test]
fn the_strict_json_writer_keeps_every_byte_and_raises_every_output_failure() {
    let run = run_lua(
        &[("json-lines.lua", JSON_LINES), ("json-lines-check.lua", JSON_LINES_CHECK)],
        "json-lines-check.lua",
        &[],
    );
    assert!(run.status.success(), "{}", run.stderr);
    let controls = run.row("controls");
    // every control byte, 0..31 and 127, survives a real JSON parser; so do the quote, the backslash, a multi-line native-style report and UTF-8
    let mut expected: String = (0u8..=31).map(char::from).collect();
    expected.push('\u{7f}');
    assert_eq!(controls["text"].as_str(), Some(expected.as_str()));
    assert_eq!(controls["quote_backslash"].as_str(), Some("a\"b\\c"));
    assert_eq!(
        controls["report"].as_str(),
        Some("model-host-scene scene=1 radius=0x3f800000\nmodel-host-tri scene=1 depth=0 return=1\n")
    );
    assert_eq!(controls["utf8"].as_str(), Some("caf\u{e9} \u{2713} \u{1f600}"));
    let numbers = run.row("numbers");
    assert_eq!(
        (
            numbers["zero"].as_i64(),
            numbers["negative"].as_i64(),
            numbers["fraction"].as_f64(),
            numbers["big"].as_i64()
        ),
        (Some(0), Some(-3), Some(1.5), Some(1_099_511_627_776))
    );
    // 1e-20 is written at 17 significant digits (9.9999999999999995e-21): the exact image of the double. serde_json's DEFAULT decoder (no `float_roundtrip` feature) reads that
    // text one unit in the last place high, so the number is decoded here from the row's own text by Rust's correctly rounded `str::parse`, which is exact; the expected value
    // is the literal, unrounded, and the comparison is equality.
    assert_eq!(run.raw_number("numbers", "tiny"), 1e-20);
    assert_eq!((numbers["yes"].as_bool(), numbers["no"].as_bool()), (Some(true), Some(false)));
    // a non-finite number, a value that is not text, number or boolean, a failed write, a failed flush, a failed close, an existing output and a closed writer all RAISE
    assert_eq!(run.row("nonfinite")["refused"].as_bool(), Some(true));
    assert_eq!(run.row("unwritable_value")["refused"].as_bool(), Some(true));
    let failures = run.row("output_failures");
    assert!(failures["write"].as_str().unwrap().contains("disk full"), "{failures}");
    assert!(failures["flush"].as_str().unwrap().contains("flush failed"), "{failures}");
    assert!(failures["close"].as_str().unwrap().contains("close failed"), "{failures}");
    assert!(run.row("existing")["refused"].as_str().unwrap().contains("already exists"));
    assert!(!run.row("closed")["refused"].as_str().unwrap().is_empty());
}

#[test]
fn the_authority_script_refuses_every_malformed_argument_before_it_writes_or_boots() {
    let root = standard();
    let cases: Vec<(Vec<String>, &str)> = {
        let one = |arg: &str, fragment: &'static str| (vec![arg.to_string()], fragment);
        let long_scenes = (1..=65).map(|n| n.to_string()).collect::<Vec<_>>().join(",");
        let wide_scenes = (1..=33).map(|n| n.to_string()).collect::<Vec<_>>().join(",");
        let wide_depths = (0..=32).map(|n| n.to_string()).collect::<Vec<_>>().join(",");
        vec![
            one("frame_ms=0", "frame_ms"),
            one("frame_ms=-5", "frame_ms"),
            one("frame_ms=abc", "frame_ms"),
            one("frame_ms=1.5", "frame_ms"),
            one("frame_ms=10001", "frame_ms"),
            one("frame_ms=", "frame_ms"),
            one("seconds=0", "seconds"),
            one("seconds=0.0", "seconds"),
            one("seconds=inf", "seconds"),
            one("seconds=nan", "seconds"),
            one("seconds=1e9", "seconds"),
            one("seconds=86401", "seconds"),
            one("seconds=-1", "seconds"),
            one("seconds=0x10", "seconds"),
            one("seconds=1.2.3", "seconds"),
            (vec!["seconds=86400".into(), "frame_ms=1".into()], "frames"),
            one("sweeps=0", "sweeps"),
            one("sweeps=10001", "sweeps"),
            one("sweeps=1.5", "sweeps"),
            one("seed=-1", "seed"),
            one("seed=2147483648", "seed"),
            one("poison=2", "poison"),
            one("los=yes", "los"),
            one("probe_scenes=1,,2", "probe_scenes"),
            one("probe_scenes=0", "probe_scenes"),
            one("probe_scenes=a", "probe_scenes"),
            one("probe_scenes=1,1", "twice"),
            one("probe_scenes=,1", "probe_scenes"),
            one("probe_scenes=1,", "probe_scenes"),
            one("probe_scenes=2147483647", "probe_scenes"),
            (vec![format!("probe_scenes={long_scenes}")], "more than 64"),
            (vec!["probe_scenes=1".into(), "probe_depths=-1".into()], "probe_depths"),
            (vec!["probe_scenes=1".into(), "probe_depths=1,x".into()], "probe_depths"),
            (
                vec![format!("probe_scenes={wide_scenes}"), format!("probe_depths={wide_depths}")],
                "more than 1024",
            ),
            one("probe_depths=1", "without probe_scenes"),
            one("mode=nonsense", "mode"),
            one("scenario=mars", "scenario"),
        ]
    };
    for (args, fragment) in cases {
        let mut all = vec![format!("root={}", slashes(&root.dir))];
        all.extend(args.iter().cloned());
        let run = run_lua(
            &[("json-lines.lua", JSON_LINES), ("model-authority.lua", MODEL_AUTHORITY)],
            "model-authority.lua",
            &all,
        );
        assert!(!run.status.success(), "{args:?} was accepted\n{}", run.stderr);
        assert!(
            run.stderr.contains(fragment),
            "{args:?}: the refusal does not name {fragment:?}\n{}",
            run.stderr
        );
        assert!(!run.out.exists(), "{args:?}: an output was created before the arguments were checked");
        assert!(!run.stderr.contains("stand-in booted"), "{args:?}");
    }
}

/// The engine as the authority script sees it, played by Lua: the real provider is asked what the engine asks at boot (the catalogue, scenes, a few triangle steps) and the
/// fixture methods answer as `standin=` says: healthy, refuse (every probe raises, with a two-line message), other (the report is for another scene), stuck (the clock never
/// advances) or noprobe (the DLL has no probe method).
const STAND_IN: &str = r#"
local standin = host.args.standin or "healthy"
-- when this engine asks the provider a triangle question the way the campaign does: never, at boot, in the second frame, or in the first requested probe
local ask = host.args.ask or "never"
local swallow = host.args.swallow == "1"
local unrelated = host.args.unrelated == "1"

-- the native stop on a provider refusal: the engine turns the provider's error (or a `refused` answer) into an error carrying the provider's marker. `swallow=1` plays an
-- engine that ACCEPTS the refused answer and goes on.
local function native_triangle (config)
	local ok, answer = pcall (config.model, "triangle", 1, 0)
	local message
	if not ok then message = "eech: [model-provider:triangle.callback] " .. tostring (answer)
	elseif type (answer) == "table" and answer.result == "refused" then message = "eech: [model-provider:triangle.refused] " .. tostring (answer.reason) end
	if message and not swallow then error (message, 0) end
end

package.preload["eech_dc"] = function ()
	local dc = { name = "stand-in" }
	function dc.prepare_installation () end
	function dc.boot (config)
		if config.model == nil then error ("eech: [model-provider:missing] boot was given no model provider", 0) end
		io.stderr:write ("stand-in booted\n")
		assert (config.model ("catalogue").scenes > 0)
		for scene = 0, 5 do config.model ("scene", scene) end
		-- the healthy runs ask three triangle steps at boot, as the campaign's own boot reaches the accessor; a negative mode asks only where `ask=` says
		if (host.args.mode or "reference") == "reference" then
			for depth = 0, 2 do config.model ("triangle", 1, depth) end
		end
		if ask == "boot" then native_triangle (config) end
		local now = 0
		local calls = 0
		local engine = {}
		function engine:frame (ms)
			calls = calls + 1
			if calls == 2 then
				if ask == "frame" then native_triangle (config) end
				if unrelated then error ("unrelated engine failure", 0) end
			end
			if standin ~= "stuck" then now = now + ms / 1000 end
		end
		function engine:clock () return { elapsed_seconds = now } end
		-- `ask=los` plays a DLL with the line-of-sight fixture and a live opposed pair, whose sweep reaches the guarded triangle question
		function engine:objects ()
			if ask ~= "los" then return {} end
			return { { id = 1, kind = "ground_vehicle", side = "blue", alive = true }, { id = 2, kind = "ground_vehicle", side = "red", alive = true } }
		end
		if ask == "los" then
			function engine:los_probe () native_triangle (config) return { hash = "0", calls = 1, visible = 0, skipped = 0, lookup_hash = "0", lookups = 0 } end
		end
		function engine:model_host_poison () end
		function engine:model_host_counters () return "model-host catalogue=1 scene=3026\n" end
		function engine:model_host_probe (scene, depth)
			if ask == "probe" then native_triangle (config) end
			if standin == "refuse" then error ("the probe was refused\nby the stand-in") end
			if standin == "other" then return string.format ("model-host-scene scene=%d radius=0x00000000\nmodel-host-tri scene=%d depth=%d return=0\n", scene + 1, scene, depth) end
			return string.format ("model-host-scene scene=%d radius=0x00000000 bb2=0 bb=0 stable=1\nmodel-host-tri scene=%d depth=%d return=0 writes=0 round_before=3072 round_after=3072\n", scene, scene, depth)
		end
		-- the complete outer floating-point state around a public entry, as the diagnostic build reports it. `fpu_diff` plays an entry that leaves MXCSR changed on its probe
		-- entry; `ask=fpu` plays a probe entry that takes the fatal path on the provider's refusal (every entry after it is POISONED, result 2)
		local stopped = false
		function engine:model_host_fpu (kind, scene, depth, rounding)
			local result, mxcsr, fatal = 0, 1, ""
			if stopped then
				result = 2
			elseif kind == 0 and standin == "fpu_fatal" then
				result, stopped, fatal = 1, true, "model-host-fpu-fatal the engine stopped\n"
			elseif kind == 0 and ask == "fpu" then
				local ok, err = pcall (native_triangle, config)
				if not ok then result, stopped, fatal = 1, true, "model-host-fpu-fatal " .. tostring (err) .. "\n" end
			end
			if standin == "fpu_diff" and kind == 0 then mxcsr = 0 end
			return string.format ("model-host-fpu kind=%d rounding=%d scene=%d depth=%d result=%d before_x87_control=0x037f before_x87_status=0x0034 before_mxcsr=0x00001fb4"
				.. " after_x87_control=0x037f after_x87_status=0x0034 after_mxcsr=0x%08x same_x87_control=1 same_x87_status=1 same_mxcsr=%d\n%s",
				kind, rounding, scene, depth, result, mxcsr == 1 and 0x1fb4 or 0x1f80, mxcsr, fatal)
		end
		if standin == "nofpu" then engine.model_host_fpu = nil end
		if standin == "noprobe" then engine.model_host_probe = nil end
		return engine
	end
	return dc
end
dofile (host.args.script)
"#;

fn stand_in_run(extra: &[&str], standin: &str) -> ScriptRun {
    let root = standard();
    let mut args = vec![
        format!("root={}", slashes(&root.dir)),
        "script={dir}/model-authority.lua".to_string(),
        format!("standin={standin}"),
        "seconds=1".to_string(),
        "frame_ms=100".to_string(),
    ];
    args.extend(extra.iter().map(|a| a.to_string()));
    run_lua(
        &[
            ("json-lines.lua", JSON_LINES),
            ("model-authority.lua", MODEL_AUTHORITY),
            ("stand-in.lua", STAND_IN),
        ],
        "stand-in.lua",
        &args,
    )
}

#[test]
fn a_healthy_authority_run_reports_every_requested_probe_in_strict_json() {
    let run = stand_in_run(&["probe_scenes=1,2", "probe_depths=0,1", "poison=1"], "healthy");
    assert!(run.status.success(), "{}", run.stderr);
    let rows = run.rows();
    let booted = run.row("booted");
    assert_eq!((booted["catalogue_asked"].as_i64(), booted["scenes_asked"].as_i64()), (Some(1), Some(6)));
    let probes: Vec<&Value> = rows.iter().filter(|r| r["event"] == "probe").collect();
    assert_eq!(probes.len(), 4);
    for probe in &probes {
        assert_eq!(probe["ok"], true);
        // the native-style report keeps its newlines through the JSON
        let report = probe["report"].as_str().unwrap();
        assert!(
            report.starts_with("model-host-scene scene=") && report.contains("\nmodel-host-tri scene=") && report.ends_with('\n'),
            "{report:?}"
        );
    }
    let summary = run.row("summary");
    assert_eq!(
        (summary["frames_run"].as_i64(), summary["probes_ok"].as_i64(), summary["probes_failed"].as_i64()),
        (Some(10), Some(4), Some(0))
    );
    assert_eq!(summary["failure"].as_str(), Some(""));
    // what the REFERENCE host answered and what the provider function handed back are counted apart (here the same, in the unmodified mode)
    assert_eq!(
        (
            summary["reference_scene"].as_i64(),
            summary["replied_scene"].as_i64(),
            summary["reference_triangle"].as_i64(),
            summary["replied_triangle"].as_i64()
        ),
        (Some(6), Some(6), Some(3), Some(3))
    );
    assert_eq!(summary["replied_found"].as_i64().unwrap() + summary["replied_ended"].as_i64().unwrap(), 3);
    assert!(run.row("native_counters")["report"].as_str().unwrap().contains("catalogue=1"));
}

#[test]
fn a_requested_probe_that_fails_or_reports_another_scene_fails_the_run_and_keeps_the_evidence() {
    for standin in ["refuse", "other"] {
        let run = stand_in_run(&["probe_scenes=1,2", "probe_depths=0,1"], standin);
        assert!(
            !run.status.success(),
            "{standin}: a run whose requested probes all failed was accepted\n{}",
            run.stderr
        );
        assert!(run.stderr.contains("requested probes failed"), "{standin}: {}", run.stderr);
        // the evidence is whole and parseable: four failed probe rows and a summary that says so
        let rows = run.rows();
        let probes: Vec<&Value> = rows.iter().filter(|r| r["event"] == "probe").collect();
        assert_eq!(probes.len(), 4, "{standin}");
        assert!(probes.iter().all(|p| p["ok"] == false), "{standin}");
        let summary = run.row("summary");
        assert_eq!(
            (summary["probes_failed"].as_i64(), summary["probes_ok"].as_i64()),
            (Some(4), Some(0)),
            "{standin}"
        );
        assert!(!summary["probe_problem"].as_str().unwrap().is_empty());
    }
    // a refusal's message is first-lined, so the row stays one line of strict JSON
    let run = stand_in_run(&["probe_scenes=1", "probe_depths=0"], "refuse");
    let probe = run.row("probe");
    assert!(probe["report"].as_str().unwrap().contains("the probe was refused") && !probe["report"].as_str().unwrap().contains('\n'));
    // probes asked of a DLL without the probe method are unavailable, not covered
    let run = stand_in_run(&["probe_scenes=1"], "noprobe");
    assert!(!run.status.success() && run.stderr.contains("model-host-fixture"), "{}", run.stderr);
}

#[test]
fn a_stuck_clock_a_missing_sweep_and_a_lost_output_are_failures_not_coverage() {
    let run = stand_in_run(&[], "stuck");
    assert!(!run.status.success() && run.stderr.contains("no progress"), "{}", run.stderr);
    assert_eq!(run.row("summary")["frames_run"].as_i64(), Some(0));
    // a requested line-of-sight sweep with no fixture to make it
    let run = stand_in_run(&["los=1"], "healthy");
    assert!(!run.status.success() && run.stderr.contains("los-fixture"), "{}", run.stderr);
    // an output that cannot be created stops the run before anything boots; an existing one is never overwritten
    let run = stand_in_run(&["out={dir}/no-such-directory/out.jsonl"], "healthy");
    assert!(!run.status.success() && !run.stderr.contains("stand-in booted"), "{}", run.stderr);
    let root = standard();
    let run = run_lua(
        &[
            ("json-lines.lua", JSON_LINES),
            ("model-authority.lua", MODEL_AUTHORITY),
            ("stand-in.lua", STAND_IN),
            ("existing.jsonl", "do not overwrite\n"),
        ],
        "stand-in.lua",
        &[
            format!("root={}", slashes(&root.dir)),
            "script={dir}/model-authority.lua".to_string(),
            "out={dir}/existing.jsonl".to_string(),
        ],
    );
    assert!(
        !run.status.success() && run.stderr.contains("already exists") && !run.stderr.contains("stand-in booted"),
        "{}",
        run.stderr
    );
    assert_eq!(std::fs::read_to_string(run.dir.join("existing.jsonl")).unwrap(), "do not overwrite\n");
}

// A negative mode is covered only by its OWN primary failure, at whatever stage the campaign first reaches the guarded answer: boot, a frame, or a requested probe. A later or
// unrelated failure, and a run that never gave the answer, are not coverage.
#[test]
fn a_negative_mode_is_covered_only_by_its_own_primary_failure_at_whatever_stage_it_comes() {
    for mode in ["decline", "refused"] {
        let marker = if mode == "decline" {
            "[model-provider:triangle.callback]"
        } else {
            "[model-provider:triangle.refused]"
        };
        for ask in ["boot", "frame", "probe"] {
            let run = stand_in_run(&[&format!("mode={mode}"), &format!("ask={ask}"), "probe_scenes=1", "probe_depths=0"], "healthy");
            assert!(
                run.status.success(),
                "{mode} asked at {ask}: the guard that was raised was not recognised\n{}",
                run.stderr
            );
            if ask == "boot" {
                let boot = run.row("boot_failed");
                assert_eq!(
                    (boot["stage"].as_str(), boot["matched"].as_bool(), boot["tripped"].as_bool()),
                    (Some("boot"), Some(true), Some(true)),
                    "{mode}"
                );
                assert!(boot["error"].as_str().unwrap().contains(marker), "{boot}");
            } else {
                let summary = run.row("summary");
                assert_eq!(summary["primary_stage"].as_str(), Some(ask), "{mode}");
                assert_eq!(summary["primary_matched"].as_bool(), Some(true), "{mode} {ask}");
                assert_eq!(summary["tripped"].as_bool(), Some(true), "{mode} {ask}");
                assert!(summary["primary_message"].as_str().unwrap().contains(marker), "{summary}");
                // a probe-stage refusal is the primary failure; the other probes (which fail because the engine has stopped) do not change the verdict
                if ask == "frame" {
                    assert_eq!(summary["frames_run"].as_i64(), Some(1));
                }
            }
        }
        // the answer was never asked for: UNREACHED, not coverage
        let run = stand_in_run(&[&format!("mode={mode}"), "ask=never"], "healthy");
        assert!(!run.status.success() && run.stderr.contains("UNREACHED"), "{mode}: {}", run.stderr);
        // the answer was given and accepted (an engine that swallowed it), and then something UNRELATED stopped the run: not coverage
        let run = stand_in_run(&[&format!("mode={mode}"), "ask=boot", "swallow=1", "unrelated=1"], "healthy");
        assert!(!run.status.success() && run.stderr.contains("for another reason"), "{mode}: {}", run.stderr);
        let summary = run.row("summary");
        assert_eq!(
            (
                summary["primary_stage"].as_str(),
                summary["primary_matched"].as_bool(),
                summary["tripped"].as_bool()
            ),
            (Some("frame"), Some(false), Some(true))
        );
        // the answer was given, swallowed, and nothing stopped the engine at all: the guard did not fire
        let run = stand_in_run(&[&format!("mode={mode}"), "ask=frame", "swallow=1"], "healthy");
        assert!(!run.status.success() && run.stderr.contains("accepted"), "{mode}: {}", run.stderr);
    }
    // a healthy mode still fails on a guard raised by a probe, a missing provider is the boot failure it expects, and a boot that fails in a healthy mode is a failure
    let run = stand_in_run(&["mode=missing"], "healthy");
    assert!(run.status.success(), "{}", run.stderr);
    assert_eq!(run.row("boot_failed")["matched"].as_bool(), Some(true));
    let run = stand_in_run(&["mode=decline", "ask=boot", "standin=healthy"], "healthy");
    assert!(run.status.success(), "{}", run.stderr);
}

// The complete outer floating-point state around the real public entries (fpu=1): the script measures nothing itself (the registers are the diagnostic DLL's harness's); what is
// tested here is that it asks for every requested initial rounding on both entries, judges each of the three words, classifies the refused entry as the PRIMARY failure at its own
// stage, does not judge the entries after the stop, and fails on a word that changed or on a DLL that cannot measure.
#[test]
fn the_script_judges_the_complete_outer_floating_point_words_across_healthy_and_refused_entries() {
    // healthy: both entries at all four initial roundings, every word identical
    let run = stand_in_run(&["fpu=1"], "healthy");
    assert!(run.status.success(), "{}", run.stderr);
    let rows: Vec<Value> = run.rows().into_iter().filter(|r| r["event"] == "fpu").collect();
    assert_eq!(rows.len(), 8, "{rows:?}");
    assert!(rows
        .iter()
        .all(|r| r["judged"] == true && r["same_x87_control"] == 1 && r["same_x87_status"] == 1 && r["same_mxcsr"] == 1 && r["result"] == 0));
    let kinds: Vec<(i64, i64)> = rows.iter().map(|r| (r["kind"].as_i64().unwrap(), r["rounding"].as_i64().unwrap())).collect();
    assert_eq!(kinds, [(1, 0), (1, 1), (1, 2), (1, 3), (0, 0), (0, 1), (0, 2), (0, 3)]);
    let summary = run.row("summary");
    assert_eq!(
        (
            summary["fpu_expected_entries"].as_i64(),
            summary["fpu_rows"].as_i64(),
            summary["fpu_measured"].as_i64()
        ),
        (Some(8), Some(8), Some(8))
    );

    // a word that changed across an entry is a measured failure, named, with the evidence kept
    let run = stand_in_run(&["fpu=1", "fpu_rounding=3"], "fpu_diff");
    assert!(
        !run.status.success() && run.stderr.contains("MXCSR changed across the public entry"),
        "{}",
        run.stderr
    );
    assert!(run.rows().iter().any(|r| r["event"] == "fpu" && r["same_mxcsr"] == 0));

    // the refused entry, at each initial rounding in its own process: the refusal is the PRIMARY failure at the fpu stage and covers the mode; the entries after it are not judged
    for rounding in 0..=3 {
        let run = stand_in_run(&["mode=decline", "ask=fpu", "fpu=1", &format!("fpu_rounding={rounding}")], "healthy");
        assert!(run.status.success(), "rounding {rounding}: {}", run.stderr);
        let summary = run.row("summary");
        assert_eq!(
            (
                summary["primary_stage"].as_str(),
                summary["primary_matched"].as_bool(),
                summary["tripped"].as_bool()
            ),
            (Some("fpu"), Some(true), Some(true))
        );
        // the counters entry and the refused probe entry were both MEASURED (rows are not the measure); nothing is unreached
        assert_eq!(
            (
                summary["fpu_expected_entries"].as_i64(),
                summary["fpu_rows"].as_i64(),
                summary["fpu_measured"].as_i64(),
                summary["fpu_unreached"].as_bool()
            ),
            (Some(2), Some(2), Some(2), Some(false)),
            "rounding {rounding}"
        );
    }
    // ... and exactly ONE initial rounding is measured in it: the refused entry stops the engine, so a second rounding could never be measured in the same process. A multi-rounding
    // request (or none named) is refused BEFORE anything is written or booted; it is never blessed with a poisoned, unjudged row standing in for the unmeasured rounding.
    for rounding in ["fpu_rounding=0,2", "fpu_rounding=0,1,2,3"] {
        let run = stand_in_run(&["mode=decline", "ask=fpu", "fpu=1", rounding], "healthy");
        assert!(
            !run.status.success() && run.stderr.contains("ONE initial rounding") && !run.stderr.contains("stand-in booted") && !run.out.exists(),
            "{rounding}: {}",
            run.stderr
        );
    }
    let run = stand_in_run(&["mode=decline", "ask=fpu", "fpu=1"], "healthy");
    assert!(
        !run.status.success() && run.stderr.contains("ONE initial rounding") && !run.out.exists(),
        "{}",
        run.stderr
    );
    let run = stand_in_run(&["mode=refused", "ask=fpu", "fpu=1", "fpu_rounding=0,1"], "healthy");
    assert!(!run.status.success() && run.stderr.contains("ONE initial rounding"), "{}", run.stderr);
    // the modes that are refused at boot have nothing to measure at all
    for mode in [
        "missing",
        "bad_nil",
        "bad_catalogue",
        "bad_name",
        "bad_scene_nan",
        "bad_collider_unloaded",
        "bad_scene0_loaded",
    ] {
        let run = stand_in_run(&[&format!("mode={mode}"), "fpu=1", "fpu_rounding=1"], "healthy");
        assert!(
            !run.status.success() && run.stderr.contains("cannot be measured") && !run.stderr.contains("stand-in booted") && !run.out.exists(),
            "{mode}: {}",
            run.stderr
        );
    }
    // a healthy mode may ask any number of roundings, each on both entries, each entry measured
    let run = stand_in_run(&["mode=absent_all", "fpu=1", "fpu_rounding=2,1"], "healthy");
    assert!(run.status.success(), "{}", run.stderr);
    let summary = run.row("summary");
    assert_eq!(
        (
            summary["fpu_expected_entries"].as_i64(),
            summary["fpu_rows"].as_i64(),
            summary["fpu_measured"].as_i64(),
            summary["fpu_unreached"].as_bool()
        ),
        (Some(4), Some(4), Some(4), Some(false))
    );
    // a refused entry that ALSO left a word changed still fails the run, though its refusal is the covered one
    let run = stand_in_run(&["mode=decline", "ask=fpu", "fpu=1", "fpu_rounding=1"], "fpu_diff");
    assert!(
        !run.status.success() && run.stderr.contains("MXCSR changed across the public entry"),
        "{}",
        run.stderr
    );
    // a refusal at the fpu stage in a HEALTHY mode is a failure of that stage
    let run = stand_in_run(&["mode=reference", "fpu=1", "fpu_rounding=2"], "fpu_fatal");
    assert!(!run.status.success() && run.stderr.contains("the fpu stage failed"), "{}", run.stderr);

    // a DLL that cannot measure, and arguments out of range, are failures, never coverage
    let run = stand_in_run(&["fpu=1"], "nofpu");
    assert!(!run.status.success() && run.stderr.contains("model-host-fixture"), "{}", run.stderr);
    for bad in [
        "fpu_rounding=4",
        "fpu_rounding=1,1",
        "fpu_rounding=0,1,2,3,0",
        "fpu_scene=0",
        "fpu_depth=-1",
        "fpu=2",
    ] {
        let root = standard();
        let run = run_lua(
            &[("json-lines.lua", JSON_LINES), ("model-authority.lua", MODEL_AUTHORITY)],
            "model-authority.lua",
            &[format!("root={}", slashes(&root.dir)), bad.to_string()],
        );
        assert!(!run.status.success() && !run.out.exists(), "{bad}: {}", run.stderr);
    }
}

// A requested floating-point measurement is covered only when every eligible requested entry was MEASURED. A genuine provider refusal that comes first (at boot, in a frame, in a
// requested probe or in the requested line-of-sight sweep) prevents it: the run reports UNREACHED, keeps the refusal as the primary failure with its stage, and fails, though the
// refusal itself is the covered one. A covered refusal never excuses an unmade measurement; a row is not a measured entry.
#[test]
fn a_requested_floating_point_measurement_is_never_called_covered_when_the_refusal_came_first_or_an_entry_was_not_measured() {
    for mode in ["decline", "refused"] {
        let marker = if mode == "decline" {
            "[model-provider:triangle.callback]"
        } else {
            "[model-provider:triangle.refused]"
        };
        let cases: [(&str, &[&str]); 4] = [
            ("boot", &[]),
            ("frame", &[]),
            ("probe", &["probe_scenes=1", "probe_depths=0"]),
            ("los", &["los=1"]),
        ];
        for (stage, extra) in cases {
            let mut args = vec![
                format!("mode={mode}"),
                format!("ask={stage}"),
                "fpu=1".to_string(),
                "fpu_rounding=1".to_string(),
            ];
            args.extend(extra.iter().map(|a| a.to_string()));
            let args: Vec<&str> = args.iter().map(String::as_str).collect();
            let run = stand_in_run(&args, "healthy");
            assert!(
                !run.status.success(),
                "{mode} refused at {stage}: a requested, unmade measurement was called covered\n{}",
                run.stderr
            );
            assert!(
                run.stderr.contains("UNREACHED") && run.stderr.contains(&format!("{stage} stage")) && run.stderr.contains(marker),
                "{mode} {stage}: the primary stage and reason are not kept\n{}",
                run.stderr
            );
            // the refusal is still the recorded, matching primary failure; no measurement row was made or invented
            let (primary_stage, matched, problem) = if stage == "boot" {
                let row = run.row("boot_failed");
                assert_eq!((row["fpu_requested"].as_bool(), row["fpu_unreached"].as_bool()), (Some(true), Some(true)));
                (row["stage"].as_str().unwrap().to_string(), row["matched"].as_bool().unwrap(), String::new())
            } else {
                let summary = run.row("summary");
                assert_eq!(
                    (
                        summary["fpu_expected_entries"].as_i64(),
                        summary["fpu_rows"].as_i64(),
                        summary["fpu_measured"].as_i64(),
                        summary["fpu_unreached"].as_bool()
                    ),
                    (Some(2), Some(0), Some(0), Some(true)),
                    "{mode} {stage}"
                );
                (
                    summary["primary_stage"].as_str().unwrap().to_string(),
                    summary["primary_matched"].as_bool().unwrap(),
                    summary["fpu_problem"].as_str().unwrap().to_string(),
                )
            };
            assert_eq!((primary_stage.as_str(), matched), (stage, true), "{mode} {stage}");
            if stage != "boot" {
                assert!(problem.contains("UNREACHED") && problem.contains(&format!("{stage} stage")), "{problem}");
            }
            assert!(
                !run.rows().iter().any(|r| r["event"] == "fpu"),
                "{mode} {stage}: a floating-point row was emitted though no entry was measured"
            );
        }
    }
    // an expected-negative mode WITHOUT a requested measurement is still covered by the same refusals (the completeness guard applies only to what was requested)
    for stage in ["boot", "frame", "probe", "los"] {
        let mut args = vec!["mode=decline".to_string(), format!("ask={stage}")];
        if stage == "probe" {
            args.extend(["probe_scenes=1".to_string(), "probe_depths=0".to_string()]);
        }
        if stage == "los" {
            args.push("los=1".to_string());
        }
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let run = stand_in_run(&args, "healthy");
        assert!(run.status.success(), "decline refused at {stage} with no measurement requested: {}", run.stderr);
    }

    // a healthy run whose probe entry takes the fatal path at the first rounding cannot measure the later roundings: the entries that were not measured are named, never
    // blessed by their poisoned rows
    let run = stand_in_run(&["mode=reference", "fpu=1", "fpu_rounding=0,1"], "fpu_fatal");
    assert!(!run.status.success(), "{}", run.stderr);
    assert!(
        run.stderr.contains("the fpu stage failed") && run.stderr.contains("only 3 of the 4 requested public entries were measured"),
        "{}",
        run.stderr
    );
    let summary = run.row("summary");
    assert_eq!(
        (
            summary["fpu_expected_entries"].as_i64(),
            summary["fpu_rows"].as_i64(),
            summary["fpu_measured"].as_i64(),
            summary["fpu_unreached"].as_bool()
        ),
        (Some(4), Some(4), Some(3), Some(true))
    );
    // a DLL that cannot measure is unreached too
    let run = stand_in_run(&["fpu=1"], "nofpu");
    assert!(!run.status.success(), "{}", run.stderr);
    assert_eq!(run.row("summary")["fpu_unreached"].as_bool(), Some(true));
}
