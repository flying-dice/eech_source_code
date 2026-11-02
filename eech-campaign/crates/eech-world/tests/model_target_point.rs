//! The host-owned TARGET POINT geometry of the collision-model store (EEDCS-52, D3; src/models.rs `target_point_local` / `target_point_world`) against minimal format fixtures.
//!
//! These are contract guards for the lookup and the arithmetic the host answers with, not an oracle for any retail scene: the files are written here in the original's formats
//! from tiny hand-made objects. What is asserted is what the original's `initialise_target_point_offsets` and `get_3d_sub_object_world_viewpoint` make observable: which
//! sub-object a scene's FIRST TARGET_POINT record names (the scene's own records, then each top-level sub-object's own records before its children), the chain of placements
//! composed from the topmost sub-object down (position scaled by the scale so far, the root's attitude applied in the original's row-vector convention, the scale multiplied
//! on), the bounding-box centre fallback with its y clamp, the invalid scene's zero, and the difference between an answer, a true absence and a refusal. Every position
//! below is chosen so that the arithmetic is exact (whole numbers, scales of 1 and 2, permutation matrices); the sin / cos of a non-trivial angle is not asserted here, and
//! neither is anything about the retail data: what the retail scenes answer is the Root's comparison with the retained original census and queries (the host Lua
//! `compare_target_points`), never these tests. Only the state `keyframe0` is exercised; a live instance's moved sub-objects are not claimed.
use eech_world::models::{
    compiled_scene_name, compiled_subobject_name, fp_state, old_last, scene_last, subobject_last, FixedPath, FixedTarget, ModelStore, OffsetSource, QueryError,
    TargetPointLocal, TargetPointWorld,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

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
}

/// a line object (the shape the route tests load): points, the bounding box the scene bounds are taken from, (flags word, faces, colour word) per surface, the face point
/// list and the surface-point references
struct Mesh {
    points: Vec<[i16; 3]>,
    bb: [f32; 6],
    surfaces: Vec<(u32, u16, u32)>,
    list: Vec<u8>,
    sp: Vec<u16>,
}

fn mesh(bb: [f32; 6]) -> Mesh {
    Mesh {
        points: vec![[0, 0, 0], [32767, 0, 0], [0, -32767, 5], [1000, 2000, -3000]],
        bb,
        surfaces: vec![(3 << 24, 2, 0x00A0_B0C0), (2 << 24, 1, 0x0001_0203)],
        list: vec![0, 1, 1, 2, 0, 1],
        sp: vec![0, 1, 2, 2, 3],
    }
}

fn table(objects: &[Mesh]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let surfaces: usize = objects.iter().map(|m| m.surfaces.len()).sum();
    let points: usize = objects.iter().map(|m| m.points.len()).sum();
    let plain: usize = objects.iter().map(|m| m.list.len()).sum();
    let mut bin = W::default();
    bin.i32(objects.len() as i32)
        .i32(surfaces as i32)
        .i32(points as i32)
        .i32(0)
        .i32(plain as i32)
        .i32(0)
        .i32(0);
    let (mut pts, mut sp) = (W::default(), W::default());
    for m in objects {
        let faces: i32 = m.surfaces.iter().map(|s| s.1 as i32).sum();
        bin.i32(0)
            .i32(m.points.len() as i32)
            .i32(m.surfaces.len() as i32)
            .i32(faces)
            .i32(0)
            .i32(0)
            .i32(0);
        bin.f32(1.0).i32(m.sp.len() as i32).i32(0).i32(0);
        bin.floats(&m.bb).floats(&m.bb);
        for (flags, count, colour) in &m.surfaces {
            bin.u32(*flags).u16(0).u16(0).i32(*count as i32).u32(*colour).bytes(&[0, 0]);
        }
        bin.i32(m.list.len() as i32).i32(0);
        bin.bytes(&m.list);
        for p in &m.points {
            for c in p {
                pts.0.extend_from_slice(&c.to_le_bytes());
            }
        }
        for s in &m.sp {
            sp.u16(*s);
        }
    }
    (bin.0, pts.0, sp.0)
}

/// keyframe 0 of a sub-object: x y z, heading pitch roll, scale x y z (the three words after them are the interpolation parameters)
fn keyframe(w: &mut W, key: &[f32; 9]) {
    w.i32(0).i32(0).floats(key).floats(&[0.0, 0.0, 0.0]);
}

/// a keyframe that places nothing: no offset, no angle, scale 1
const REST: [f32; 9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0];

fn at(x: f32, y: f32, z: f32) -> [f32; 9] {
    [x, y, z, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]
}

/// one sub-object record: its object, its keyframe 0, its (kind, offset) records, its children
struct Sub {
    object: i32,
    key: [f32; 9],
    indices: Vec<(i32, i32)>,
    children: Vec<Sub>,
}

fn sub(key: [f32; 9], indices: Vec<(i32, i32)>, children: Vec<Sub>) -> Sub {
    Sub {
        object: 1,
        key,
        indices,
        children,
    }
}

fn sub_record(w: &mut W, s: &Sub) {
    w.i32(s.object).i32(0).i32(0).i32(0).i32(0); // object, contributes, in level, out level, no limits
    w.i32(1);
    keyframe(w, &s.key);
    w.f32(1.0).i32(0); // dissolve, value keyframes
    w.i32(s.indices.len() as i32);
    for (kind, offset) in &s.indices {
        w.i32(*kind).i32(*offset);
    }
    w.i32(s.children.len() as i32);
    for child in &s.children {
        sub_record(w, child);
    }
}

fn nodes_in(subs: &[Sub]) -> (i32, i32) {
    subs.iter().fold((0, 0), |(n, i), s| {
        let (cn, ci) = nodes_in(&s.children);
        (n + 1 + cn, i + s.indices.len() as i32 + ci)
    })
}

/// a 3dobjdb.bin scene record with sub-objects and scene-level (kind, offset) records
fn scene_record(scene: i32, main: i32, indices: &[(i32, i32)], subs: &[Sub]) -> Vec<u8> {
    let (nodes, node_indices) = nodes_in(subs);
    let mut w = W::default();
    for _ in 0..16 {
        w.i32(-1);
    }
    w.i32(scene).i32(0); // scene_index, self_shadows
    w.i32(0).i32(0).i32(0).i32(0); // lights, cameras, links, sprite lights
    w.i32(nodes).i32(indices.len() as i32 + node_indices).i32(0); // sub-objects, named sub-objects, texture animations
    w.i32(0).i32(main); // approximations, main object
    w.i32(0).i32(0).floats(&[1.0, 1.0, 1.0]); // shadow_approximation_index, shadow polygon object, scale
    w.i32(1); // collision object
    w.i32(1);
    keyframe(&mut w, &REST);
    w.f32(1.0).i32(0);
    w.i32(0).i32(0); // displacement sequence index, its keyframes
    w.i32(indices.len() as i32);
    for (kind, offset) in indices {
        w.i32(*kind).i32(*offset);
    }
    w.i32(subs.len() as i32);
    for s in subs {
        sub_record(&mut w, s);
    }
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

struct Root {
    dir: PathBuf,
}

impl Root {
    fn new(objects: &[Mesh], scenes: &[Vec<u8>]) -> Root {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "eech-world-model-target-point-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let data = dir.join("cohokum").join("3ddata");
        std::fs::create_dir_all(&data).unwrap();
        let (bin, pts, sp) = table(objects);
        std::fs::write(data.join("bininfo.bin"), bininfo()).unwrap();
        std::fs::write(data.join("3dobjs.bin"), bin).unwrap();
        std::fs::write(data.join("3dobjs.pts"), pts).unwrap();
        std::fs::write(data.join("3dobjs.sp"), sp).unwrap();
        std::fs::write(data.join("3dobjdb.bin"), database(scenes)).unwrap();
        Root { dir }
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn kind(name: &str) -> i32 {
    (1..subobject_last())
        .find(|k| compiled_subobject_name(*k) == Some(name))
        .unwrap_or_else(|| panic!("no compiled sub-object kind {name}")) as i32
}

fn target_point() -> i32 {
    kind("TARGET_POINT")
}

/// permutation matrix, row-major, that sends the row vector (x, y, z) to (z, x, y) in the original's convention (`v.x * m[0][0] + v.y * m[1][0] + v.z * m[2][0]`, ...)
const CYCLE: [f32; 9] = [0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0];
const IDENTITY: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

/// objects: 1 a box with bounds that need no clamp, 2 a box whose y centre is negative, 3 a box whose y centre is positive. Both straddle the origin on every axis: the original's scene bounds start from a zeroed box, so they always include the origin and an object wholly to one side would be widened to it.
///
/// scene 1: the scene's own record names top-level 1 (at 1,0,0); top-level 0 (at 100,0,0) holds a TARGET_POINT record of its own for its child, which must not win.
/// scene 2: no scene-level record. Depth first: top-level 0 (3,0,0) holds none; its child (0,1,0) holds a record for ITS child (0,0,9), and top-level 1 (50,0,0) holds a
///          record too, but later. The answer is the three-deep chain (3,1,9).
/// scene 3: a placement with a scale: top-level 0 at (1,0,0) with scale 2 holds the record for its child at (0,2,0), so the child's offset is doubled: (1,4,0).
/// scene 4: the scene's own record names top-level 0 at (1,2,3), for composing with a root pose.
/// scene 5: a main object and no sub-objects (bounding box of object 2). scene 6: the same with object 3. scene 7: sub-objects but no TARGET_POINT record.
fn standard() -> (Root, ModelStore) {
    let tp = target_point();
    let other = kind("FIXED_WING_LANDING_ROUTE");
    let scaled_parent = {
        let mut key = at(1.0, 0.0, 0.0);
        key[6..].copy_from_slice(&[2.0, 2.0, 2.0]);
        key
    };
    let scenes = vec![
        scene_record(
            1,
            1,
            &[(tp, 1)],
            &[
                sub(at(100.0, 0.0, 0.0), vec![(tp, 0)], vec![sub(at(7.0, 0.0, 0.0), vec![], vec![])]),
                sub(at(1.0, 0.0, 0.0), vec![], vec![]),
            ],
        ),
        scene_record(
            2,
            1,
            &[],
            &[
                sub(
                    at(3.0, 0.0, 0.0),
                    vec![],
                    vec![sub(at(0.0, 1.0, 0.0), vec![(tp, 0)], vec![sub(at(0.0, 0.0, 9.0), vec![], vec![])])],
                ),
                sub(at(50.0, 0.0, 0.0), vec![(tp, 0)], vec![sub(at(0.0, 0.0, 0.0), vec![], vec![])]),
            ],
        ),
        scene_record(3, 1, &[], &[sub(scaled_parent, vec![(tp, 0)], vec![sub(at(0.0, 2.0, 0.0), vec![], vec![])])]),
        scene_record(4, 1, &[(tp, 0)], &[sub(at(1.0, 2.0, 3.0), vec![], vec![])]),
        scene_record(5, 2, &[], &[]),
        scene_record(6, 3, &[], &[]),
        scene_record(7, 1, &[(other, 0)], &[sub(at(5.0, 5.0, 5.0), vec![], vec![])]),
        // scenes 9 and 10 (8 stays an empty slot): a target point straight up (0,2,0), and one with only a z part (0,1,3), for a fixed object's offset
        scene_record(9, 1, &[(tp, 0)], &[sub(at(0.0, 2.0, 0.0), vec![], vec![])]),
        scene_record(10, 1, &[(tp, 0)], &[sub(at(0.0, 1.0, 3.0), vec![], vec![])]),
        scene_record(11, 1, &[(tp, 0)], &[sub(at(1.0e-30, 2.0, 0.0), vec![], vec![])]),
    ];
    let root = Root::new(
        &[
            mesh([-1.0, 1.0, -1.0, 1.0, -1.0, 1.0]),
            mesh([-4.0, 2.0, -10.0, 2.0, -2.0, 6.0]),
            mesh([-4.0, 2.0, -2.0, 10.0, 0.0, 6.0]),
        ],
        &scenes,
    );
    let store = ModelStore::load(&root.dir).expect("the fixture installation loads");
    (root, store)
}

fn bits3(v: [f32; 3]) -> [u32; 3] {
    v.map(f32::to_bits)
}

fn bits9(v: [f32; 9]) -> [u32; 9] {
    v.map(f32::to_bits)
}

fn floats<const N: usize>(bits: [u32; N]) -> [f32; N] {
    bits.map(f32::from_bits)
}

/// the local answer as floats (so that +0.0 and -0.0 are the same zero) and its source
fn local(store: &ModelStore, scene: i64) -> ([f32; 3], OffsetSource) {
    match store.target_point_local(scene).unwrap() {
        TargetPointLocal::Offset { offset, source } => (floats(offset), source),
        other => panic!("scene {scene}: {other:?}"),
    }
}

fn world(store: &ModelStore, scene: i64, position: [f32; 3], attitude: [f32; 9], scale: [f32; 3]) -> TargetPointWorld {
    store.target_point_world(scene, bits3(position), bits9(attitude), bits3(scale)).unwrap()
}

fn found(store: &ModelStore, scene: i64, position: [f32; 3], attitude: [f32; 9], scale: [f32; 3]) -> ([f32; 3], [f32; 9]) {
    match world(store, scene, position, attitude, scale) {
        TargetPointWorld::Found { position, attitude } => (floats(position), floats(attitude)),
        other => panic!("scene {scene}: {other:?}"),
    }
}

#[test]
fn the_scenes_own_record_names_the_target_point_before_any_sub_objects_records() {
    let (_root, store) = standard();
    // scene 1: the scene-level record names top-level 1 (at 1,0,0); top-level 0 (at 100,0,0) and its child hold later records for the same kind
    assert_eq!(local(&store, 1), ([1.0, 0.0, 0.0], OffsetSource::SubObject));
}

#[test]
fn without_a_scene_record_the_search_runs_depth_first_with_each_sub_objects_own_records_before_its_children() {
    let (_root, store) = standard();
    // scene 2: top-level 0 holds no record, its child does (naming the grandchild); top-level 1 also holds one, but it is reached only after the whole of top-level 0's subtree
    assert_eq!(local(&store, 2), ([3.0, 1.0, 9.0], OffsetSource::SubObject));
}

#[test]
fn the_placement_chain_is_composed_from_the_topmost_sub_object_with_the_scale_so_far() {
    let (_root, store) = standard();
    // scene 3: the parent places at (1,0,0) and doubles the scale; the child's own (0,2,0) is therefore doubled before it is added: (1,4,0), not (1,2,0)
    assert_eq!(local(&store, 3), ([1.0, 4.0, 0.0], OffsetSource::SubObject));
}

#[test]
fn an_instances_world_target_point_applies_the_root_pose_in_the_originals_convention() {
    let (_root, store) = standard();
    let one = [1.0, 1.0, 1.0];
    // an identity pose at the origin is the local offset
    let (position, attitude) = found(&store, 4, [0.0; 3], IDENTITY, one);
    assert_eq!(position, [1.0, 2.0, 3.0]);
    assert_eq!(attitude, IDENTITY);
    // the offset (1,2,3) is a row vector times the root attitude, then added to the root position: (3,1,2) + (10,20,30)
    let (position, attitude) = found(&store, 4, [10.0, 20.0, 30.0], CYCLE, one);
    assert_eq!(position, [13.0, 21.0, 32.0]);
    // the sub-object's own keyframe has no angle, so the world attitude is the root's
    assert_eq!(attitude, CYCLE);
    // the root's relative scale multiplies the placement before the attitude is applied: (2,4,6) -> (6,2,4)
    let (position, _) = found(&store, 4, [10.0, 20.0, 30.0], CYCLE, [2.0, 2.0, 2.0]);
    assert_eq!(position, [16.0, 22.0, 34.0]);
    // the whole chain of a deeper scene follows the same pose: scene 3's (1,4,0) turned by the cycle is (0,1,4)
    let (position, _) = found(&store, 3, [0.0; 3], CYCLE, one);
    assert_eq!(position, [0.0, 1.0, 4.0]);
}

#[test]
fn a_scene_without_a_target_point_falls_back_to_its_bounding_box_centre_with_y_clamped_at_zero() {
    let (_root, store) = standard();
    // the fixture's scene bounds are those of its main object (it straddles the origin, and no sub-objects widen it)
    assert_eq!(store.scene(5).unwrap().bounding_box, bits6([-4.0, 2.0, -10.0, 2.0, -2.0, 6.0]));
    // x and z are the centres; the y centre is -4 and is clamped to zero
    assert_eq!(local(&store, 5), ([-1.0, 0.0, 2.0], OffsetSource::BoundingBox));
    // a positive y centre is kept
    assert_eq!(store.scene(6).unwrap().bounding_box, bits6([-4.0, 2.0, -2.0, 10.0, 0.0, 6.0]));
    assert_eq!(local(&store, 6), ([-1.0, 4.0, 3.0], OffsetSource::BoundingBox));
    // sub-objects without a TARGET_POINT record do not make one: the bounding box still decides (the scene bounds, whatever they are, are the store's own)
    let bounds = floats(store.scene(7).unwrap().bounding_box);
    let expected = [
        (bounds[0] + bounds[1]) * 0.5,
        ((bounds[2] + bounds[3]) * 0.5).max(0.0),
        (bounds[4] + bounds[5]) * 0.5,
    ];
    assert_eq!(local(&store, 7), (expected, OffsetSource::BoundingBox));
}

#[test]
fn an_instance_of_a_scene_without_a_target_point_is_the_originals_second_branch_not_a_number() {
    let (_root, store) = standard();
    assert_eq!(world(&store, 5, [1.0, 2.0, 3.0], IDENTITY, [1.0; 3]), TargetPointWorld::NoTargetPoint);
    assert_eq!(world(&store, 7, [1.0, 2.0, 3.0], IDENTITY, [1.0; 3]), TargetPointWorld::NoTargetPoint);
}

fn bits6(v: [f32; 6]) -> [u32; 6] {
    v.map(f32::to_bits)
}

#[test]
fn the_invalid_scene_is_zero_a_scene_never_read_is_refused_and_a_scene_outside_the_catalogue_is_an_argument_error() {
    let (_root, store) = standard();
    // scene 0 is the original's invalid object index: its offset is the zero it sets, from no data
    assert_eq!(
        store.target_point_local(0).unwrap(),
        TargetPointLocal::Offset {
            offset: [0; 3],
            source: OffsetSource::InvalidZero
        }
    );
    // scene 8 is in the catalogue but no file defined it: the original still constructs it (a zeroed slot), so its offset is the zero bounding box and it has no target point.
    // That is an answer, and it is not the invalid scene's zero: the source says which.
    assert_eq!(
        store.target_point_local(8).unwrap(),
        TargetPointLocal::Offset {
            offset: [0; 3],
            source: OffsetSource::BoundingBox
        }
    );
    assert_eq!(world(&store, 8, [0.0; 3], IDENTITY, [1.0; 3]), TargetPointWorld::NoTargetPoint);
    // the last scene of the catalogue is still in the domain; one past it is not
    let last = scene_last() as i64 - 1;
    assert!(matches!(
        store.target_point_local(last).unwrap(),
        TargetPointLocal::Offset {
            source: OffsetSource::BoundingBox,
            ..
        }
    ));
    assert!(matches!(store.target_point_local(last + 1), Err(QueryError::BadScene(_))));
    assert!(matches!(
        store.target_point_world(last + 1, [0; 3], [0; 9], [0; 3]),
        Err(QueryError::BadScene(_))
    ));
    // arguments the original could never produce are refused as arguments
    assert!(matches!(store.target_point_local(-1), Err(QueryError::BadScene(-1))));
    assert!(matches!(store.target_point_local(i64::MAX), Err(QueryError::BadScene(_))));
    // the invalid scene has no instance to ask about
    assert!(matches!(store.target_point_world(0, [0; 3], [0; 9], [0; 3]), Err(QueryError::BadScene(0))));
}

#[test]
fn a_store_that_never_read_the_custom_files_cannot_say_what_a_slot_holds_and_a_missing_installation_does_not_load() {
    let (root, _store) = standard();
    let inventory = ModelStore::load_inventory(&root.dir).expect("the fixture inventory loads");
    for scene in [1, 5, 8] {
        assert_eq!(
            inventory.target_point_local(scene).unwrap(),
            TargetPointLocal::Refused("custom_definitions_not_loaded"),
            "scene {scene}"
        );
        assert_eq!(
            world(&inventory, scene, [0.0; 3], IDENTITY, [1.0; 3]),
            TargetPointWorld::Refused("custom_definitions_not_loaded"),
            "scene {scene}"
        );
    }
    // the invalid scene needs no data
    assert!(matches!(
        inventory.target_point_local(0).unwrap(),
        TargetPointLocal::Offset {
            source: OffsetSource::InvalidZero,
            ..
        }
    ));
    // an installation with a required file missing is a load failure, never a store whose slots answer zero
    std::fs::remove_file(root.dir.join("cohokum").join("3ddata").join("3dobjdb.bin")).unwrap();
    assert!(ModelStore::load(&root.dir).is_err());
}

#[test]
fn the_answers_do_not_move_between_asks() {
    let (_root, store) = standard();
    for scene in 1..=7 {
        let first = store.target_point_local(scene).unwrap();
        assert_eq!(store.target_point_local(scene).unwrap(), first, "scene {scene}");
    }
}

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
fn a_target_point_query_hands_the_callers_floating_point_state_back_and_does_not_depend_on_it() {
    let (_root, store) = standard();
    let original_mxcsr = fpu::mxcsr();
    let original_x87 = fpu::x87_control();
    // a pose whose arithmetic is rounding-sensitive (0.1 is not representable), through the bounding-box path, the chain path and the root-pose path
    let pose = |store: &ModelStore| {
        (
            store.target_point_local(3).unwrap(),
            store.target_point_local(5).unwrap(),
            store
                .target_point_world(4, bits3([0.1, 0.2, 0.3]), bits9(CYCLE), bits3([0.7, 0.7, 0.7]))
                .unwrap(),
        )
    };
    let reference = pose(&store);
    assert!(matches!(reference.2, TargetPointWorld::Found { .. }));

    for rounding in 0u32..4 {
        let mxcsr = 0x1f80 | (rounding << 13) | 0x21; // all masked, invalid and precision flags set
        let x87 = 0x037f | ((rounding as u16) << 10);
        fpu::set_mxcsr(mxcsr);
        fpu::set_x87_control(x87);
        let before = fp_state().unwrap();
        let answers = pose(&store);
        let after = fp_state().unwrap();
        // restore the thread's own state before asserting, so a failure cannot leak a strange mode into this thread
        fpu::set_mxcsr(original_mxcsr);
        fpu::set_x87_control(original_x87);
        assert_eq!(after, before, "rounding {rounding}: the caller's floating-point state after the queries");
        assert_eq!(answers, reference, "rounding {rounding}: the answers depended on the caller's rounding mode");
    }
    assert_eq!(fpu::mxcsr(), original_mxcsr);
    assert_eq!(fpu::x87_control(), original_x87);
}

#[cfg(target_arch = "x86_64")]
#[test]
fn a_caller_with_an_exception_unmasked_is_refused_for_every_target_point_question_and_its_state_is_untouched() {
    let (_root, store) = standard();
    let original_mxcsr = fpu::mxcsr();
    // unmask the SSE invalid-operation exception (bit 7): the original's arithmetic never runs with an exception unmasked
    fpu::set_mxcsr(original_mxcsr & !0x80);
    let before = fp_state().unwrap();
    let chain = store.target_point_local(3).unwrap();
    let bbox = store.target_point_local(5).unwrap();
    let in_world = store.target_point_world(4, bits3([0.0; 3]), bits9(IDENTITY), bits3([1.0; 3])).unwrap();
    let after = fp_state().unwrap();
    fpu::set_mxcsr(original_mxcsr);
    assert_eq!(chain, TargetPointLocal::Refused("fp_masks"));
    assert_eq!(bbox, TargetPointLocal::Refused("fp_masks"));
    assert_eq!(in_world, TargetPointWorld::Refused("fp_masks"));
    assert_eq!(before, after);
}

// ---------------------------------------------------------------------------------------------------------------------------------
// the host Lua face: the diagnostic path Root compares the retained original census and queries through

fn lua_with_store(root: &std::path::Path) -> mlua::Lua {
    use mlua::Lua;
    let lua = Lua::new();
    let host = lua.create_table().unwrap();
    let args = lua.create_table().unwrap();
    args.set("root", root.to_string_lossy().into_owned()).unwrap();
    host.set("args", args).unwrap();
    eech_world::model_host::register(&lua, &host).unwrap();
    lua.globals().set("host", host).unwrap();
    lua.globals().set("ROOT", root.to_string_lossy().into_owned()).unwrap();
    lua
}

/// the observer's text for a scene name (`clean_name`)
fn observer_name(scene: usize) -> String {
    compiled_scene_name(scene)
        .unwrap()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-') { c } else { '_' })
        .collect()
}

/// A retained report in the observer's shape, with literal hex words. Both passes of the census carry the same rows (scene 9 is not recorded by the original); the queries
/// are three: branch 1 at scene 4 with the root pose (10,20,30) and the cycle attitude (the original's answer 13,21,32), the same with a result that is one ulp off, and one
/// that took another branch. `damaged` changes one census word.
fn report_file(dir: &std::path::Path, damaged: bool) -> PathBuf {
    let pass = |generation: u32| {
        let z = if damaged && generation == 1 { "41100001" } else { "41100000" };
        format!(
            "tp-offset generation={generation} index=0 name=- state=invalid_zero offset=00000000,00000000,00000000 bounds=- live=1\n\
             tp-offset generation={generation} index=1 name=S1 state=sub_object offset=3f800000,00000000,00000000 bounds=- live=1\n\
             tp-offset generation={generation} index=2 name=S2 state=sub_object offset=40400000,3f800000,{z} bounds=- live=1\n\
             tp-offset generation={generation} index=3 name=S3 state=sub_object offset=3f800000,40800000,00000000 bounds=- live=1\n\
             tp-offset generation={generation} index=5 name=S5 state=bbox offset=bf800000,00000000,40000000 bounds=c0800000,40000000,c1200000,40000000,c0000000,40c00000 live=1\n\
             tp-offset generation={generation} index=6 name=S6 state=bbox offset=bf800000,40800000,40400000 bounds=c0800000,40000000,c0000000,41200000,00000000,40c00000 live=1\n\
             tp-offset generation={generation} index=8 name=S8 state=bbox offset=00000000,00000000,00000000 bounds=00000000,00000000,00000000,00000000,00000000,00000000 live=1
\n             tp-offset generation={generation} index=9 name=S9 state=none offset=- bounds=- live=-\n"
        )
    };
    let name = observer_name(4);
    let pose = "vp_after=41200000,41a00000,41f00000,00000000,3f800000,00000000,00000000,00000000,3f800000,3f800000,00000000,00000000";
    let query = |index: u32, branch: u32, result: &str| {
        format!("tp-event index={index} seq={index} gen=1 gen_open=0 branch={branch} model=4 model_name={name} {pose} result={result} host_id_len=0\n")
    };
    let mut lines = Vec::new();
    for (when, generation) in [("boot", 0u32), ("boot", 1), ("end", 0), ("end", 1)] {
        lines.push(serde_json::json!({ "event": "offsets", "when": when, "generation": generation, "text": pass(generation) }));
    }
    for (index, branch, result) in [
        (0, 1, "41500000,41a80000,42000000"),
        (1, 1, "41500000,41a80000,42000001"),
        (2, 2, "00000000,00000000,00000000"),
    ] {
        lines.push(serde_json::json!({ "event": "query", "index": index, "text": query(index, branch, result) }));
    }
    let path = dir.join(if damaged { "damaged.jsonl" } else { "report.jsonl" });
    std::fs::write(&path, lines.iter().map(|l| l.to_string() + "\n").collect::<String>()).unwrap();
    path
}

#[test]
fn the_host_lua_face_answers_local_offsets_and_instance_poses_as_plain_bit_patterns() {
    let (root, _store) = standard();
    let lua = lua_with_store(&root.dir);
    lua.globals().set("NAME4", compiled_scene_name(4).unwrap()).unwrap();
    lua.load(
        r#"
        local store = host.model_store (ROOT)
        local o = store:target_point_local (1)
        assert (o.result == "offset" and o.source == "sub_object" and o.offset == "0x3f800000,0x00000000,0x00000000", o.offset)
        o = store:target_point_local (5)
        assert (o.source == "bbox" and o.offset == "0xbf800000,0x00000000,0x40000000", o.offset)
        o = store:target_point_local (0)
        assert (o.source == "invalid_zero" and o.offset == "0x00000000,0x00000000,0x00000000")
        -- an in-domain slot no file defined is the original's zeroed slot: the zero bounding box, not a refusal and not the invalid scene's zero
        o = store:target_point_local (8)
        assert (o.result == "offset" and o.source == "bbox" and o.offset == "0x00000000,0x00000000,0x00000000", o.offset)
        assert (not pcall (store.target_point_local, store, -1))
        assert (not pcall (store.target_point_local, store, 1.5))

        local pose = {
            position = "0x41200000,0x41a00000,0x41f00000",
            attitude = "0x00000000,0x3f800000,0x00000000,0x00000000,0x00000000,0x3f800000,0x3f800000,0x00000000,0x00000000",
        }
        local w = store:target_point_world (4, pose)
        assert (w.result == "found" and w.state == "keyframe0" and w.position == "0x41500000,0x41a80000,0x42000000", w.position)
        pose.scale = "0x40000000,0x40000000,0x40000000"
        w = store:target_point_world (4, pose)
        assert (w.position == "0x41800000,0x41b00000,0x42080000", w.position)
        assert (store:target_point_world (5, pose).result == "no_target_point")
        assert (store:target_point_world (8, pose).result == "no_target_point")
        -- a pose that is not a pose is refused before the store is asked
        assert (not pcall (store.target_point_world, store, 4, { position = pose.position }))
        assert (not pcall (store.target_point_world, store, 4, { position = "0x1,0x2", attitude = pose.attitude }))
        assert (not pcall (store.target_point_world, store, 4, { position = "1,2,3", attitude = pose.attitude }))
        assert (store:scene_index ("NO_SUCH_SCENE") == nil)
        assert (store:scene_index (NAME4) == 4)
        "#,
    )
    .exec()
    .expect("the host Lua face answers as the store does");
}

#[test]
fn the_host_lua_comparison_checks_every_recorded_offset_row_of_both_passes_and_each_branch_one_query() {
    let (root, _store) = standard();
    let lua = lua_with_store(&root.dir);
    lua.globals().set("NAME4", compiled_scene_name(4).unwrap()).unwrap();
    let dir = root.dir.join("reports");
    std::fs::create_dir_all(&dir).unwrap();
    let whole = report_file(&dir, false);
    let damaged = report_file(&dir, true);
    lua.globals().set("WHOLE", whole.to_string_lossy().into_owned()).unwrap();
    lua.globals().set("DAMAGED", damaged.to_string_lossy().into_owned()).unwrap();
    lua.load(
        r#"
        local store = host.model_store (ROOT)
        local r = store:compare_target_points (WHOLE)
        -- two events of two passes each, eight rows a pass, one of them not recorded by the original: seven rows compared a pass (one is an empty slot with the zero box)
        assert (r.offset_events == 4 and r.offset_rows == 32 and r.offset_rows_compared == 28 and r.offset_rows_not_recorded == 4, "rows")
        assert (r.offset_sub_object == 12 and r.offset_bbox == 12 and r.offset_invalid_zero == 4, "states")
        assert (r.offset_mismatches == 0, table.concat (r.listed, "; "))
        assert (r.queries == 3 and r.other_branch_queries == 1 and r.branch1_queries == 2, "queries")
        assert (r.branch1_matched == 1 and r.branch1_mismatches == 1 and r.branch1_refused == 0, "branch 1")
        assert (r.state == "keyframe0")
        -- the query whose result is one ulp off is listed, with both answers
        assert (#r.listed == 1 and r.listed [1]:find ("41500000,41a80000,42000001", 1, true) and r.listed [1]:find ("41500000,41a80000,42000000", 1, true), r.listed [1])
        -- one damaged census word in the second pass is one mismatch in each of its two events, and is named
        local d = store:compare_target_points (DAMAGED)
        assert (d.offset_mismatches == 2, d.offset_mismatches)
        assert (d.listed [1]:find ("pass 1 scene 2", 1, true), d.listed [1])
        -- a report that is not the observer's is refused, never compared clean
        assert (not pcall (store.compare_target_points, store, ROOT .. "/no-such-report.jsonl"))
        "#,
    )
    .exec()
    .expect("the comparison reports what the retained rows hold");
}

// ---------------------------------------------------------------------------------------------------------------------------------
// a FIXED object's target point (branch 3 of get_local_entity_target_point): the host adds the shape's own offset to the entity position the original had copied

/// the entity position the tests give: 100, 50, -20
const BASE: [f32; 3] = [100.0, 50.0, -20.0];

fn fixed(store: &ModelStore, shape: i64, base: [u32; 3], heading: Option<u32>) -> FixedTarget {
    store.target_point_fixed(shape, base, heading).unwrap()
}

fn offset_answer(result: [f32; 3], path: FixedPath) -> FixedTarget {
    FixedTarget::Offset { result: bits3(result), path }
}

#[test]
fn an_offset_with_no_horizontal_part_never_reads_the_heading_and_only_adds_y() {
    let (_root, store) = standard();
    // scene 9's target point is (0, 2, 0): the original's first test is false, so the heading is not read at all
    let expected = offset_answer([100.0, 52.0, -20.0], FixedPath::NoHorizontal);
    assert_eq!(fixed(&store, 9, bits3(BASE), None), expected);
    // a heading the caller happens to have is not read either: not a NaN, not a rotation
    for heading in [0x7fc0_0000, 0x4000_0000, 0x0000_0000] {
        assert_eq!(fixed(&store, 9, bits3(BASE), Some(heading)), expected, "heading {heading:08x}");
    }
    // the empty slot (scene 8) has the zero offset: no horizontal part either
    assert!(matches!(
        fixed(&store, 8, bits3(BASE), None),
        FixedTarget::Offset {
            path: FixedPath::NoHorizontal,
            ..
        }
    ));
}

#[test]
fn a_zero_heading_keeps_the_legacy_quirk_x_and_z_are_not_added_but_y_still_is() {
    let (_root, store) = standard();
    // scene 4's offset is (1, 2, 3): it has a horizontal part, so the heading is read; zero (either sign) skips the rotation and its additions
    for heading in [0x0000_0000, 0x8000_0000] {
        assert_eq!(
            fixed(&store, 4, bits3(BASE), Some(heading)),
            offset_answer([100.0, 52.0, -20.0], FixedPath::HeadingZero),
            "heading {heading:08x}"
        );
    }
    assert_eq!(
        fixed(&store, 1, bits3(BASE), Some(0)),
        offset_answer([100.0, 50.0, -20.0], FixedPath::HeadingZero)
    );
}

#[test]
fn a_heading_the_original_would_have_read_but_the_caller_lacks_is_a_refusal_never_a_default() {
    let (_root, store) = standard();
    for shape in [1, 2, 4, 10] {
        assert_eq!(
            fixed(&store, shape, bits3(BASE), None),
            FixedTarget::Refused("heading_missing"),
            "shape {shape}"
        );
    }
}

#[test]
fn a_non_zero_heading_rotates_and_adds_x_and_z_exactly_where_the_arithmetic_is_exact() {
    let (_root, store) = standard();
    // scene 10's offset is (0, 1, 3). With a tiny heading h, sin h is h and cos h is exactly 1, so the rotated x is 3h, which a round-toward-zero add to 100 loses, and the rotated z
    // is exactly 3: the sum is (100, 51, -17). The path says the rotation was taken. What sin and cos give for a large heading is the measured original's to say (the retail replay), not this test's.
    let tiny = 1.0e-30f32.to_bits();
    assert_eq!(
        fixed(&store, 10, bits3(BASE), Some(tiny)),
        offset_answer([100.0, 51.0, -17.0], FixedPath::Rotated)
    );
}

#[test]
fn the_invalid_shape_copies_the_position_and_a_valid_empty_slot_adds_a_zero_that_changes_a_negative_zero() {
    let (_root, store) = standard();
    // the bits of the position the original copied: a negative zero in every coordinate and a NaN with a payload
    let negative_zero = [0x8000_0000, 0x8000_0000, 0x8000_0000];
    let payload = [0x7fc1_2345, 0x8000_0000, 0xff80_0001];
    // shape 0: the original copies the position and adds nothing; no add of zero, no float conversion
    assert_eq!(fixed(&store, 0, negative_zero, None), FixedTarget::PositionCopy { result: negative_zero });
    assert_eq!(fixed(&store, 0, payload, Some(0x3f80_0000)), FixedTarget::PositionCopy { result: payload });
    // shape 8 is a valid shape (an empty slot, offset zero): the original still adds its y, and -0.0 + 0.0 is +0.0; x and z are not touched
    assert_eq!(
        fixed(&store, 8, negative_zero, None),
        FixedTarget::Offset {
            result: [0x8000_0000, 0x0000_0000, 0x8000_0000],
            path: FixedPath::NoHorizontal
        }
    );
}

#[test]
fn a_shape_outside_the_catalogue_is_an_argument_error_and_an_inventory_store_cannot_say() {
    let (root, store) = standard();
    let last = scene_last() as i64 - 1;
    assert!(matches!(store.target_point_fixed(last + 1, bits3(BASE), None), Err(QueryError::BadScene(_))));
    assert!(matches!(store.target_point_fixed(-1, bits3(BASE), None), Err(QueryError::BadScene(-1))));
    assert!(matches!(store.target_point_fixed(last, bits3(BASE), None).unwrap(), FixedTarget::Offset { .. }));
    let inventory = ModelStore::load_inventory(&root.dir).expect("the fixture inventory loads");
    assert_eq!(
        fixed(&inventory, 1, bits3(BASE), Some(0)),
        FixedTarget::Refused("custom_definitions_not_loaded")
    );
    assert_eq!(fixed(&inventory, 0, bits3(BASE), None), FixedTarget::PositionCopy { result: bits3(BASE) });
}

#[cfg(target_arch = "x86_64")]
#[test]
fn a_fixed_target_query_hands_the_callers_floating_point_state_back_and_does_not_depend_on_it() {
    let (_root, store) = standard();
    let original_mxcsr = fpu::mxcsr();
    let original_x87 = fpu::x87_control();
    // a heading whose sin and cos are not exact, through the rotated, the zero-heading and the no-horizontal paths
    let ask = |store: &ModelStore| {
        (
            store.target_point_fixed(2, bits3(BASE), Some(0.5f32.to_bits())).unwrap(),
            store.target_point_fixed(2, bits3(BASE), Some(0)).unwrap(),
            store.target_point_fixed(9, bits3(BASE), None).unwrap(),
        )
    };
    let reference = ask(&store);
    assert!(matches!(reference.0, FixedTarget::Offset { path: FixedPath::Rotated, .. }));
    for rounding in 0u32..4 {
        let mxcsr = 0x1f80 | (rounding << 13) | 0x21;
        let x87 = 0x037f | ((rounding as u16) << 10);
        fpu::set_mxcsr(mxcsr);
        fpu::set_x87_control(x87);
        let before = fp_state().unwrap();
        let answers = ask(&store);
        let after = fp_state().unwrap();
        fpu::set_mxcsr(original_mxcsr);
        fpu::set_x87_control(original_x87);
        assert_eq!(after, before, "rounding {rounding}: the caller's floating-point state after the queries");
        assert_eq!(answers, reference, "rounding {rounding}: the answers depended on the caller's rounding mode");
    }
    // an exception unmasked in the caller is refused, state untouched
    fpu::set_mxcsr(original_mxcsr & !0x80);
    let before = fp_state().unwrap();
    let refused = store.target_point_fixed(2, bits3(BASE), Some(0.5f32.to_bits())).unwrap();
    let after = fp_state().unwrap();
    fpu::set_mxcsr(original_mxcsr);
    assert_eq!(refused, FixedTarget::Refused("fp_masks"));
    assert_eq!(before, after);
}

/// (shape, case, heading, the offset the original read, the result it returned, the base position when the observer recorded one)
type FixedRow<'a> = (usize, &'a str, String, &'a str, &'a str, Option<&'a str>);

/// A retained report in the observer's shape with branch-3 queries (the entity position, the heading the original read, the offset it read and the result it returned, as literal
/// words): eight fixed-object queries and one branch-2 query. The tiny heading is the float 1e-30.
fn fixed_report_file(dir: &std::path::Path) -> PathBuf {
    let tiny = format!("{:08x}", 1.0e-30f32.to_bits());
    let base = "42c80000,42480000,c1a00000";
    // (shape, case, heading, offset the original read, result, base)
    let rows: Vec<FixedRow> = vec![
        (
            9,
            "no_horizontal_offset",
            "-".into(),
            "00000000,40000000,00000000",
            "42c80000,42500000,c1a00000",
            Some(base),
        ),
        (
            1,
            "heading_zero_quirk",
            "00000000".into(),
            "3f800000,00000000,00000000",
            "42c80000,42480000,c1a00000",
            Some(base),
        ),
        (
            10,
            "rotated_horizontal_offset",
            tiny.clone(),
            "00000000,3f800000,40400000",
            "42c80000,424c0000,c1880000",
            Some(base),
        ),
        // the original's result differs from the host's by one ulp in y
        (
            9,
            "no_horizontal_offset",
            "-".into(),
            "00000000,40000000,00000000",
            "42c80000,42500001,c1a00000",
            Some(base),
        ),
        // an observer from before the base position was recorded
        (
            9,
            "no_horizontal_offset",
            "-".into(),
            "00000000,40000000,00000000",
            "42c80000,42500000,c1a00000",
            None,
        ),
        // the heading the original read is missing from the report
        (
            4,
            "rotated_horizontal_offset",
            "-".into(),
            "3f800000,40000000,40400000",
            "42c80000,42500000,c1a00000",
            Some(base),
        ),
        // the offset the original read is not the host's, the arithmetic from the host's agrees with the result
        (
            9,
            "no_horizontal_offset",
            "-".into(),
            "00000000,40400000,00000000",
            "42c80000,42500000,c1a00000",
            Some(base),
        ),
    ];
    let mut lines = vec![
        serde_json::json!({ "event": "offsets", "when": "boot", "generation": 0, "text": "tp-offset generation=0 index=0 name=- state=invalid_zero offset=00000000,00000000,00000000 bounds=- live=1\n" }),
    ];
    let mut index = 0;
    for (shape, case, heading, offset, result, base) in rows {
        let base_field = base.map_or(String::new(), |b| format!("base={b} "));
        let text = format!(
            "tp-event index={index} seq={index} branch=3 case={case} model={shape} model_name={} fixed=1 shape={shape} horizontal=1 heading={heading} xz_applied=0 offset={offset} result={result} {base_field}host_id_len=0\n",
            observer_name(shape)
        );
        lines.push(serde_json::json!({ "event": "query", "index": index, "text": text }));
        index += 1;
    }
    // a shape outside the catalogue and a branch-2 query
    let outside = format!("tp-event index={index} seq={index} branch=3 case=no_horizontal_offset model=-1 model_name=- fixed=1 shape=99999 horizontal=0 heading=- xz_applied=0 offset=- result={base} base={base} host_id_len=0\n");
    lines.push(serde_json::json!({ "event": "query", "index": index, "text": outside }));
    index += 1;
    let two = format!("tp-event index={index} seq={index} branch=2 case=instance_without_target_point model=7 model_name=- fixed=0 shape=-1 horizontal=0 heading=- xz_applied=0 offset=- result={base} base={base} host_id_len=0\n");
    lines.push(serde_json::json!({ "event": "query", "index": index, "text": two }));
    let path = dir.join("fixed-report.jsonl");
    std::fs::write(&path, lines.iter().map(|l| l.to_string() + "\n").collect::<String>()).unwrap();
    path
}

#[test]
fn the_host_lua_face_answers_a_fixed_target_from_words_and_the_comparer_replays_branch_three_from_the_recorded_inputs() {
    let (root, _store) = standard();
    let lua = lua_with_store(&root.dir);
    let dir = root.dir.join("reports");
    std::fs::create_dir_all(&dir).unwrap();
    lua.globals().set("FLOAT_TINY", 1.0e-30f32.to_bits()).unwrap();
    lua.globals().set("FIXED", fixed_report_file(&dir).to_string_lossy().into_owned()).unwrap();
    lua.load(
        r#"
        local store = host.model_store (ROOT)
        local base = "0x42c80000,0x42480000,0xc1a00000"
        local a = store:target_point_fixed (9, base)
        assert (a.result == "offset" and a.path == "no_horizontal" and a.position == "0x42c80000,0x42500000,0xc1a00000", a.position)
        a = store:target_point_fixed (1, base, "0x00000000")
        assert (a.path == "heading_zero" and a.position == "0x42c80000,0x42480000,0xc1a00000", a.position)
        a = store:target_point_fixed (10, base, string.format ("0x%08x", FLOAT_TINY))
        assert (a.path == "rotated" and a.position == "0x42c80000,0x424c0000,0xc1880000", a.position)
        a = store:target_point_fixed (4, base)
        assert (a.result == "refused" and a.reason == "heading_missing")
        a = store:target_point_fixed (0, "0x80000000,0x80000000,0x80000000")
        assert (a.result == "position_copy" and a.position == "0x80000000,0x80000000,0x80000000", a.position)
        assert (not pcall (store.target_point_fixed, store, -1, base))
        assert (not pcall (store.target_point_fixed, store, 9, "0x1,0x2"))
        assert (not pcall (store.target_point_fixed, store, 9, base, "1"))

        local r = store:compare_target_points (FIXED)
        assert (r.queries == 9 and r.branch3_queries == 8 and r.other_branch_queries == 1 and r.branch1_queries == 0, "queries")
        -- seven carry a base position (the old-format one does not); the host answers four exactly; one is a different result; two are refused (a missing heading, a shape outside)
        assert (r.branch3_complete == 7 and r.branch3_missing_input == 1, "inputs")
        assert (r.branch3_matched == 4 and r.branch3_mismatches == 1 and r.branch3_refused == 2, "outcomes")
        assert (r.branch3_offset_input_differs == 1, "input")
        -- nothing is passed by default: every branch-3 query is exactly one outcome
        assert (r.branch3_matched + r.branch3_mismatches + r.branch3_refused + r.branch3_missing_input == r.branch3_queries)
        local text = table.concat (r.listed, "\n")
        assert (text:find ("42c80000,42500001,c1a00000", 1, true) and text:find ("no base position", 1, true) and text:find ("heading_missing", 1, true) and text:find ("outside the catalogue", 1, true), text)
        "#,
    )
    .exec()
    .expect("the fixed-target face and the branch-3 replay count every outcome apart");
}

// ---------------------------------------------------------------------------------------------------------------------------------
// the provider's TARGET_POINT_OFFSET answer (EEDCS-52, D3): what initialise_target_point_offsets now asks the host for instead of constructing the scene

#[test]
fn the_provider_answers_a_scenes_local_offset_as_exact_bits_with_its_name_and_refuses_what_it_cannot() {
    let (root, store) = standard();
    let lua = lua_with_store(&root.dir);
    for (name, scene) in [("NAME8", 8), ("NAME9", 9), ("NAME10", 10), ("NAME11", 11)] {
        lua.globals().set(name, compiled_scene_name(scene).unwrap()).unwrap();
    }
    lua.globals().set("TINY", format!("0x{:08x}", 1.0e-30f32.to_bits())).unwrap();
    lua.globals().set("LAST", scene_last() as i64).unwrap();
    lua.load(
        r#"
        local provider = host.model_provider (host.model_store (ROOT))
        local a = provider ("target_point_offset", 9)
        assert (a.result == "offset" and a.p == "0x00000000,0x40000000,0x00000000" and a.name == NAME9, tostring (a.p))
        a = provider ("target_point_offset", 10)
        assert (a.p == "0x00000000,0x3f800000,0x40400000" and a.name == NAME10, tostring (a.p))
        -- a tiny coordinate keeps every bit through the exact boundary
        a = provider ("target_point_offset", 11)
        assert (a.p == TINY .. ",0x40000000,0x00000000" and a.name == NAME11, tostring (a.p))
        -- a catalogue slot no file defines is a valid answer: the zero bounding box, not a refusal
        a = provider ("target_point_offset", 8)
        assert (a.result == "offset" and a.p == "0x00000000,0x00000000,0x00000000" and a.name == NAME8, tostring (a.p))
        -- the invalid scene is the campaign's own zero and is not asked; a depth, a missing scene and a scene outside the catalogue are refused as requests
        assert (not pcall (provider, "target_point_offset", 0))
        assert (not pcall (provider, "target_point_offset", 9, 1))
        assert (not pcall (provider, "target_point_offset", nil))
        assert (not pcall (provider, "target_point_offset", LAST))
        assert (not pcall (provider, "target_point_offset", -1))
        "#,
    )
    .exec()
    .expect("the provider's offset answers");
    // the provider's answer is the store's own local offset, word for word, for every scene the fixture defines
    for scene in 1..=11i64 {
        let TargetPointLocal::Offset { offset, .. } = store.target_point_local(scene).unwrap() else {
            panic!("scene {scene} has no offset")
        };
        lua.globals().set("SCENE", scene).unwrap();
        lua.globals()
            .set("WORDS", format!("0x{:08x},0x{:08x},0x{:08x}", offset[0], offset[1], offset[2]))
            .unwrap();
        lua.load(r#"assert (host.model_provider (host.model_store (ROOT)) ("target_point_offset", SCENE).p == WORDS, SCENE)"#)
            .exec()
            .unwrap_or_else(|e| panic!("scene {scene}: {e}"));
    }
}

/// a report with an offsets event and these branch-3 rows, each `(case field, offset field)` as the row text holds them (an empty field is absent)
fn branch_three_report(dir: &std::path::Path, name: &str, rows: &[(&str, &str)]) -> PathBuf {
    let mut lines = vec![
        serde_json::json!({ "event": "offsets", "when": "boot", "generation": 0, "text": "tp-offset generation=0 index=0 name=- state=invalid_zero offset=00000000,00000000,00000000 bounds=- live=1\n" }),
    ];
    for (index, (case, offset)) in rows.iter().enumerate() {
        let text = format!(
            "tp-event index={index} seq={index} branch=3 {case}model=9 model_name={} fixed=1 shape=9 horizontal=0 heading=- xz_applied=0 {offset}result=42c80000,42500000,c1a00000 base=42c80000,42480000,c1a00000 host_id_len=0\n",
            observer_name(9)
        );
        lines.push(serde_json::json!({ "event": "query", "index": index, "text": text }));
    }
    let path = dir.join(name);
    std::fs::write(&path, lines.iter().map(|l| l.to_string() + "\n").collect::<String>()).unwrap();
    path
}

#[test]
fn a_branch_three_row_without_a_known_case_or_a_well_formed_offset_is_never_counted_matched() {
    let (root, _store) = standard();
    let lua = lua_with_store(&root.dir);
    let dir = root.dir.join("reports");
    std::fs::create_dir_all(&dir).unwrap();
    let good_case = "case=no_horizontal_offset ";
    let good_offset = "offset=00000000,40000000,00000000 ";
    // the positive first: a row that carries both is replayed and matches (scene 9's offset is (0, 2, 0): y 50 + 2 = 52)
    let control = branch_three_report(&dir, "control.jsonl", &[(good_case, good_offset)]);
    // a row with no offset (absent or "-") or no case label (absent or "-") cannot have those checks: counted as missing its input, listed once per reason
    let missing = branch_three_report(
        &dir,
        "missing.jsonl",
        &[
            (good_case, good_offset),
            ("", good_offset),
            ("case=- ", good_offset),
            (good_case, ""),
            (good_case, "offset=- "),
        ],
    );
    // a label that is none of the three and an offset that is not three words are malformed reports, not rows to skip
    let unknown = branch_three_report(&dir, "unknown.jsonl", &[("case=rotated_by_mistake ", good_offset)]);
    let short = branch_three_report(&dir, "short.jsonl", &[(good_case, "offset=00000000,40000000 ")]);
    let loose = branch_three_report(&dir, "loose.jsonl", &[(good_case, "offset=0,40000000,00000000 ")]);
    for (name, path) in [
        ("CONTROL", &control),
        ("MISSING", &missing),
        ("UNKNOWN", &unknown),
        ("SHORT", &short),
        ("LOOSE", &loose),
    ] {
        lua.globals().set(name, path.to_string_lossy().into_owned()).unwrap();
    }
    lua.load(
        r#"
        local store = host.model_store (ROOT)
        local r = store:compare_target_points (CONTROL)
        assert (r.branch3_queries == 1 and r.branch3_matched == 1 and r.branch3_missing_input == 0 and r.branch3_mismatches == 0 and r.branch3_refused == 0, "control")
        r = store:compare_target_points (MISSING)
        assert (r.branch3_queries == 5 and r.branch3_matched == 1 and r.branch3_missing_input == 4, "missing")
        assert (r.branch3_matched + r.branch3_mismatches + r.branch3_refused + r.branch3_missing_input == r.branch3_queries)
        local text = table.concat (r.listed, "\n")
        assert (text:find ("case label", 1, true) and text:find ("recorded offset", 1, true), text)
        local ok, why = pcall (store.compare_target_points, store, UNKNOWN)
        assert (not ok and tostring (why):find ("unknown case 'rotated_by_mistake'", 1, true), tostring (why))
        ok, why = pcall (store.compare_target_points, store, SHORT)
        assert (not ok and tostring (why):find ("bad offset", 1, true), tostring (why))
        ok, why = pcall (store.compare_target_points, store, LOOSE)
        assert (not ok and tostring (why):find ("bad offset", 1, true), tostring (why))
        "#,
    )
    .exec()
    .expect("a missing or malformed case or offset is counted or refused, never matched");
}
