//! The host-owned TARGET POINT geometry of the collision-model store (EEDCS-52, D3; src/models.rs `target_point_local` / `target_point_world`) against minimal format fixtures.
//!
//! These are contract guards for the lookup and the arithmetic the host answers with, not an oracle for any retail scene: the files are written here in the original's formats
//! from tiny hand-made objects. What is asserted is what the original's `initialise_target_point_offsets` and `get_3d_sub_object_world_viewpoint` make observable: which
//! sub-object a scene's FIRST TARGET_POINT record names (the scene's own records, then each top-level sub-object's own records before its children), the chain of placements
//! composed from the topmost sub-object down (position scaled by the scale so far, the root's attitude applied in the original's row-vector convention, the scale multiplied
//! on), the bounding-box centre fallback with its y clamp, the invalid scene's zero, and the difference between an answer, a true absence and a refusal. Every position
//! below is chosen so that the arithmetic is exact (whole numbers, scales of 1 and 2, permutation matrices); the sin / cos of a non-trivial angle is not asserted here, and
//! neither is anything about the retail data: what the retail scenes answer is the Root's comparison with the retained original census and queries (the host Lua
//! `compare_target_points`), never these tests. `target_point_world` answers the `keyframe0` state; `target_point_live` applies the placement a campaign supplies (the arithmetic of a moved sub-instance, from whole numbers), and what a retail instance holds at any moment is the Root's native actual, not these tests.
use eech_world::models::{
    compiled_scene_name, compiled_subobject_name, fp_state, old_last, scene_last, subobject_last, FixedPath, FixedTarget, ModelStore, OffsetSource, QueryError,
    TargetPointChain, TargetPointLocal, TargetPointWorld,
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

// ---------------------------------------------------------------------------------------------------------------------------------
// get_local_entity_target_point routed to the host (EEDCS-52, D3): the chain the host names, the world position for the campaign's CURRENT placement, and a fixed object's position

#[test]
fn the_chain_names_the_first_target_point_sub_object_by_its_sibling_position_at_each_level() {
    let (_root, store) = standard();
    // scene 1: the scene's own record names top-level 1 (the later sibling) and wins over top-level 0's record
    assert_eq!(store.target_point_chain(1).unwrap(), TargetPointChain::Found(vec![1]));
    // scene 2: depth first, a three-deep chain through the first children
    assert_eq!(store.target_point_chain(2).unwrap(), TargetPointChain::Found(vec![0, 0, 0]));
    // scene 3: a parent and its child
    assert_eq!(store.target_point_chain(3).unwrap(), TargetPointChain::Found(vec![0, 0]));
    assert_eq!(store.target_point_chain(4).unwrap(), TargetPointChain::Found(vec![0]));
    // no TARGET_POINT record (a main object only, sub-objects of another kind, an empty slot) and the invalid scene have no chain: the original's "not found" branch
    for scene in [0, 5, 6, 7, 8] {
        assert_eq!(store.target_point_chain(scene).unwrap(), TargetPointChain::NoTargetPoint, "scene {scene}");
    }
    // a scene outside the catalogue is a request error, never a chain
    assert!(store.target_point_chain(-1).is_err());
    assert!(store.target_point_chain(scene_last() as i64).is_err());
}

#[test]
fn the_world_position_follows_the_live_placement_the_campaign_gives_not_keyframe_zero() {
    let (_root, store) = standard();
    let (origin, scale) = (bits3([0.0, 0.0, 0.0]), bits3([1.0, 1.0, 1.0]));
    let identity = bits9(IDENTITY);
    // keyframe 0 of scene 3 as the campaign would hold it on a fresh instance: the parent at (1,0,0) scale 2, its child at (0,2,0)
    let parent = bits9([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 2.0, 2.0]);
    let child = bits9(at(0.0, 2.0, 0.0));
    let fresh = store.target_point_live(3, origin, identity, scale, &[parent, child]).unwrap();
    let keyframe_zero = store.target_point_world(3, origin, identity, scale).unwrap();
    assert_eq!(fresh, keyframe_zero, "a live placement equal to keyframe 0 gives the keyframe-0 answer");
    let TargetPointWorld::Found { position, .. } = fresh else {
        panic!("no target point")
    };
    assert_eq!(floats(position), [1.0, 4.0, 0.0]);
    let moved = |placements: &[[u32; 9]], position: [u32; 3], scale: [u32; 3]| match store.target_point_live(3, position, identity, scale, placements).unwrap()
    {
        TargetPointWorld::Found { position, .. } => floats(position),
        other => panic!("{other:?}"),
    };
    // the child has moved: (0,5,0) under the parent's scale 2 is (0,10,0), plus the parent's (1,0,0)
    assert_eq!(moved(&[parent, bits9(at(0.0, 5.0, 0.0))], origin, scale), [1.0, 10.0, 0.0]);
    // the parent's live scale is 3 now: the child's (0,2,0) is tripled
    assert_eq!(
        moved(&[bits9([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 3.0, 3.0, 3.0]), child], origin, scale),
        [1.0, 6.0, 0.0]
    );
    // the main object's own relative scale is 2 and its world position (10,20,30): both reach the answer
    assert_eq!(moved(&[parent, child], bits3([10.0, 20.0, 30.0]), bits3([2.0, 2.0, 2.0])), [12.0, 28.0, 30.0]);
    // a placement for another number of levels than the chain has is a refusal, never padded or cut
    for wrong in [vec![], vec![parent], vec![parent, child, child]] {
        assert_eq!(
            store.target_point_live(3, origin, identity, scale, &wrong).unwrap(),
            TargetPointWorld::Refused("live_chain_length_differs"),
            "{} levels",
            wrong.len()
        );
    }
    // a scene with no TARGET_POINT is not a position, whatever placement is given
    assert_eq!(
        store.target_point_live(5, origin, identity, scale, &[]).unwrap(),
        TargetPointWorld::NoTargetPoint
    );
}

#[test]
fn the_provider_answers_the_chain_the_world_position_and_a_fixed_position_and_refuses_what_it_cannot() {
    let (root, _store) = standard();
    let lua = lua_with_store(&root.dir);
    for (name, scene) in [("NAME0", 0), ("NAME3", 3), ("NAME5", 5), ("NAME9", 9), ("NAME10", 10)] {
        lua.globals().set(name, compiled_scene_name(scene).unwrap_or("")).unwrap();
    }
    lua.load(
        r#"
        local provider = host.model_provider (host.model_store (ROOT))
        -- the chain: sibling positions, topmost first; none for a scene without a target point and for the invalid scene
        local a = provider ("target_point_chain", 3)
        assert (a.result == "chain" and #a.offsets == 2 and a.offsets [1] == 0 and a.offsets [2] == 0 and a.name == NAME3, tostring (a.result))
        a = provider ("target_point_chain", 5)
        assert (a.result == "absent" and a.name == NAME5)
        a = provider ("target_point_chain", 0)
        assert (a.result == "absent" and a.name == NAME0)
        assert (not pcall (provider, "target_point_chain", 3, 1))
        assert (not pcall (provider, "target_point_chain", 3, nil, nil, nil, "0x00000000"))
        assert (not pcall (provider, "target_point_chain", nil))
        -- the world position for the placement given: scene 3 at keyframe 0 is (1,4,0); the child moved to (0,5,0) is (1,10,0)
        local root = "0x00000000,0x00000000,0x00000000,0x3f800000,0x00000000,0x00000000,0x00000000,0x3f800000,0x00000000,0x00000000,0x00000000,0x3f800000,0x3f800000,0x3f800000,0x3f800000"
        local parent = "0x3f800000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x40000000,0x40000000,0x40000000"
        a = provider ("target_point_world", 3, nil, nil, nil, root .. "," .. parent .. ",0x00000000,0x40000000,0x00000000,0x00000000,0x00000000,0x00000000,0x3f800000,0x3f800000,0x3f800000")
        assert (a.result == "world" and a.p == "0x3f800000,0x40800000,0x00000000" and a.name == NAME3, tostring (a.p))
        a = provider ("target_point_world", 3, nil, nil, nil, root .. "," .. parent .. ",0x00000000,0x40a00000,0x00000000,0x00000000,0x00000000,0x00000000,0x3f800000,0x3f800000,0x3f800000")
        assert (a.p == "0x3f800000,0x41200000,0x00000000", tostring (a.p))
        -- a level count that is not the chain's is the host's refusal as an answer, not a number; a scene with no target point likewise
        a = provider ("target_point_world", 3, nil, nil, nil, root .. "," .. parent .. ",0x00000000,0x40000000,0x00000000,0x00000000,0x00000000,0x00000000,0x3f800000,0x3f800000,0x3f800000,0x00000000,0x40000000,0x00000000,0x00000000,0x00000000,0x00000000,0x3f800000,0x3f800000,0x3f800000")
        assert (a.result == "refused" and a.reason == "live_chain_length_differs", tostring (a.reason))
        a = provider ("target_point_world", 5, nil, nil, nil, root .. "," .. parent)
        assert (a.result == "refused" and a.reason == "no_target_point", tostring (a.reason))
        -- a pose that is not whole levels of exact bit patterns is a malformed request
        assert (not pcall (provider, "target_point_world", 3, nil, nil, nil, root .. ",0x00000000"))
        assert (not pcall (provider, "target_point_world", 3, nil, nil, nil, "1,2,3"))
        assert (not pcall (provider, "target_point_world", 3, nil, nil, nil, nil))
        assert (not pcall (provider, "target_point_world", 3, 1, nil, nil, root .. "," .. parent))
        -- a fixed object: scene 9's offset is (0,2,0), no horizontal part, so three words are enough and y gains 2 (50 + 2 = 52, the position's x and z untouched)
        a = provider ("target_point_fixed", 9, nil, nil, nil, "0x42c80000,0x42480000,0xc1a00000")
        assert (a.result == "fixed" and a.p == "0x42c80000,0x42500000,0xc1a00000" and a.name == NAME9, tostring (a.p))
        -- scene 10's offset (0,1,3) has a horizontal part: a heading is needed, and a missing one is a refusal naming it (nothing is defaulted)
        a = provider ("target_point_fixed", 10, nil, nil, nil, "0x42c80000,0x42480000,0xc1a00000")
        assert (a.result == "refused" and a.reason == "heading_missing", tostring (a.reason))
        -- a zero heading is the legacy quirk: x and z are not added, y still is (50 + 1 = 51)
        a = provider ("target_point_fixed", 10, nil, nil, nil, "0x42c80000,0x42480000,0xc1a00000,0x00000000")
        assert (a.result == "fixed" and a.p == "0x42c80000,0x424c0000,0xc1a00000" and a.name == NAME10, tostring (a.p))
        -- the invalid shape's position copy is the campaign's own, and a pose of another size is malformed
        assert (not pcall (provider, "target_point_fixed", 0, nil, nil, nil, "0x42c80000,0x42480000,0xc1a00000"))
        assert (not pcall (provider, "target_point_fixed", 9, nil, nil, nil, "0x42c80000,0x42480000"))
        assert (not pcall (provider, "target_point_fixed", 9, nil, nil, nil, nil))
        "#,
    )
    .exec()
    .expect("the provider's target point answers");
}

#[test]
fn a_nonzero_heading_rotates_the_fixed_offset_with_the_originals_arithmetic_through_the_provider() {
    let (root, store) = standard();
    let lua = lua_with_store(&root.dir);
    // scene 10 (offset (0,1,3)) at heading 1 radian over the position (100,50,-20): the host's own store gives the words the provider must hand over
    let heading = 1.0f32.to_bits();
    let base = bits3([100.0, 50.0, -20.0]);
    let FixedTarget::Offset { result, path } = store.target_point_fixed(10, base, Some(heading)).unwrap() else {
        panic!("no fixed answer")
    };
    assert_eq!(path, FixedPath::Rotated);
    lua.globals()
        .set("POSE", format!("0x{:08x},0x{:08x},0x{:08x},0x{:08x}", base[0], base[1], base[2], heading))
        .unwrap();
    lua.globals()
        .set("WORDS", format!("0x{:08x},0x{:08x},0x{:08x}", result[0], result[1], result[2]))
        .unwrap();
    lua.load(r#"assert (host.model_provider (host.model_store (ROOT)) ("target_point_fixed", 10, nil, nil, nil, POSE).p == WORDS)"#)
        .exec()
        .expect("the provider hands over the store's rotated words");
}

// ---- ENTITY_CANDIDATE through the real provider and ModelStore boundary (EEDCS-49, D3): a candidate the original skipped before it read any shape needs no model ----

fn candidate_text(facts: &[f32]) -> String {
    facts.iter().map(|value| format!("0x{:08x}", value.to_bits())).collect::<Vec<_>>().join(",")
}

#[test]
fn a_direct_city_building_and_a_fixed_candidate_below_the_tallest_skip_are_skipped_without_a_model_and_a_candidate_that_needs_geometry_still_refuses() {
    let (root, _store) = standard();
    let lua = lua_with_store(&root.dir);
    // THE ENTRY OF THIS TEST CHANGED with the fixed/static collision backing (Task49): a candidate that is not a mobile names the placement the host served it by on every staged question, and the host holds the
    // fixed collision state of that placement's record. Every scenario and every assertion below is unchanged; each question now names a structure of the host's roster whose record holds the state the
    // campaign's words of that stage state (at the origin, heading 0, collision-fixed, not aligned; the shape is the scene the stage names)
    let roster = std::rc::Rc::new(std::cell::RefCell::new(eech_world::roster::SectorRoster::default()));
    lua.set_app_data(eech_world::roster::SharedRoster(roster.clone()));
    let placed = |shape: f32, entity_type: f32| -> String {
        let key = roster.borrow_mut().insert((0, 0), entity_type as i32, None).to_string();
        let report = fixed_words([0.0; 3], entity_type, 0.0, shape, 1.0, 0.0, 0.0);
        assert_eq!(table_result(&scan_words_answer(&lua, report, &key)).0, "placed");
        key
    };
    let (key_stage, key_building, key_shape8, key_shape0, key_shape1) = (
        placed(0.0, FIXED_TYPE),
        placed(0.0, BUILDING_TYPE),
        placed(8.0, FIXED_TYPE),
        placed(0.0, FIXED_TYPE),
        placed(1.0, FIXED_TYPE),
    );
    let keyed = |scene: i32, facts: &[f32; 44], key: &str| {
        let (result, reason, _) = candidate_with_identity(&lua, scene, facts, key);
        (result, reason)
    };
    // the facts of ENTITY_CANDIDATE (mb_hostmodel.h): 6 mobile, 7 fixed, 9 city building, 12 tallest, 13..15 position, 42 fixed facts read, 43 placement read
    let mut facts = [0.0f32; 44];
    facts[..6].copy_from_slice(&[-100.0, -100.0, -100.0, 100.0, 100.0, 100.0]);
    facts[19] = 0.1;
    // 1. a candidate that is not mobile in a sector whose tallest structure is below the segment: the original read nothing about it, the campaign sends the first stage (scene 0), and the host
    // skips it; even a scene the store has not loaded (8 is an empty slot) cannot make that a refusal
    let mut below = facts;
    below[12] = -500.0;
    assert_eq!(keyed(0, &below, &key_stage), ("skip".to_string(), String::new()));
    assert_eq!(keyed(8, &below, &key_stage).0, "skip", "the original skipped it before it read its shape");
    // 2. the same candidate in a sector it is not skipped in: the host asks for the fixed facts, and still needs no scene
    let mut reached = facts;
    reached[12] = 1000.0;
    assert_eq!(keyed(0, &reached, &key_stage), ("need_fixed".to_string(), String::new()));
    // 3. a building listed directly in a sector (fixed, a city building, no shape of its own): its legitimate skip needs no scene, no radius and no model
    let mut building = reached;
    building[7] = 1.0;
    building[9] = 1.0;
    building[42] = 1.0;
    assert_eq!(
        keyed(0, &building, &key_building),
        ("skip".to_string(), String::new()),
        "the original skipped it without a shape lookup"
    );
    // 4. a candidate that reaches its broad phase needs its model: an unloaded scene, and scene 0, are refusals named by the provider, never a skip or a miss
    let mut fixed = reached;
    fixed[7] = 1.0;
    fixed[42] = 1.0;
    let (result, reason) = keyed(8, &fixed, &key_shape8);
    assert_eq!((result.as_str(), reason.contains("not loaded")), ("refused", true), "{reason}");
    let (result, reason) = keyed(0, &fixed, &key_shape0);
    assert_eq!(
        (result.as_str(), reason.contains("not a scene a candidate can have")),
        ("refused", true),
        "{reason}"
    );
    // 5. with a loaded scene (1) the same facts reach the placement stage and then the narrow phase's pose: the model is the store's own
    assert_eq!(keyed(1, &fixed, &key_shape1), ("need_placement".to_string(), String::new()));
    fixed[43] = 1.0;
    assert_eq!(keyed(1, &fixed, &key_shape1).0, "test");
    // 6. a mobile candidate whose box is met asks for its UPDATED answer before any model, and then needs the model of its scene
    // (the mobile candidate's physical record is the host's: the registry holds the one this candidate's identity names, inside the segment's box)
    lua.set_app_data(eech_world::movement::SharedMovers(registry_with(
        "eech-world:7",
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 5.0],
    )));
    let mut mobile = facts;
    mobile[6] = 1.0;
    mobile[16] = 3.0;
    assert_eq!(
        candidate_with_identity(&lua, 8, &mobile, "eech-world:7").0,
        "need_updated",
        "the destructive getter first: the original read it before its radius"
    );
    mobile[20] = 1.0;
    assert_eq!(
        candidate_with_identity(&lua, 8, &mobile, "eech-world:7").0,
        "refused",
        "an unloaded scene is refused once the broad phase needs it"
    );
}

// ---- ENTITY_SCAN through the real provider (EEDCS-49, D3, fourth slice): the enumeration needs no model, and a malformed question is a named refusal ----

fn scan_answer(lua: &mlua::Lua, facts: &[f32]) -> (String, String, String) {
    scan_answer_standing(lua, facts, "")
}

/// the same question with the key of the entry the scan stands at in the identity place (ENTRY and RESOLVED carry one; the other phases carry none)
fn scan_answer_standing(lua: &mlua::Lua, facts: &[f32], standing: &str) -> (String, String, String) {
    let text = if standing.is_empty() {
        candidate_text(facts)
    } else {
        format!("{} {standing}", candidate_text(facts))
    };
    lua.globals().set("FACTS", text).unwrap();
    let table: mlua::Table = lua
        .load(r#"return host.model_provider (host.model_store (ROOT)) ("entity_scan", 0, nil, nil, nil, FACTS)"#)
        .eval()
        .expect("the provider answers");
    (
        table.get("result").unwrap(),
        table.get::<Option<String>>("p").unwrap().unwrap_or_default(),
        table.get::<Option<String>>("reason").unwrap().unwrap_or_default(),
    )
}

#[test]
fn the_enumeration_is_decided_by_the_real_provider_from_the_segment_and_the_map_alone_and_a_malformed_question_is_refused_by_name() {
    let (root, _store) = standard();
    let lua = lua_with_store(&root.dir);
    // the host's sector roster: the enumeration serves the entries it holds, so it keeps one, standing in sector (1, 1), for the scan to stand at (the roster's own order and refusals are entity_roster_host)
    let roster = std::rc::Rc::new(std::cell::RefCell::new(eech_world::roster::SectorRoster::default()));
    let standing = roster.borrow_mut().insert((1, 1), 20, None).to_string();
    lua.set_app_data(eech_world::roster::SharedRoster(roster));
    // the segment (250, 0, 250) to (260, 0, 250) over a map of sectors of 100 up to 399: the window is sectors 1..3 in each axis
    let mut facts = [0.0f32; 16];
    facts[..6].copy_from_slice(&[250.0, 0.0, 250.0, 260.0, 0.0, 250.0]);
    facts[6] = 100.0;
    facts[9] = 399.0;
    facts[10] = 399.0;
    // the scan starts at the window's minimum and the provider reads no model for it
    assert_eq!(
        scan_answer(&lua, &facts),
        ("first".to_string(), "0x3f800000,0x3f800000".to_string(), String::new())
    );
    // an entry that is served and not excluded is offered; an excluded one is skipped
    facts[11] = 1.0;
    facts[12] = 1.0;
    facts[13] = 1.0;
    assert_eq!(scan_answer_standing(&lua, &facts, &standing).0, "offer");
    facts[14] = 1.0;
    assert_eq!(scan_answer_standing(&lua, &facts, &standing).0, "next");
    // the end of the last list of the window ends the scan
    facts[14] = 0.0;
    facts[11] = 2.0;
    facts[12] = 3.0;
    facts[13] = 3.0;
    assert_eq!(scan_answer(&lua, &facts).0, "done_none");
    // a cursor outside the window is refused by name, never answered
    facts[12] = 0.0;
    let (result, _, reason) = scan_answer(&lua, &facts);
    assert_eq!(result, "refused", "{reason}");
    assert!(reason.contains("outside the sector window"), "{reason}");
}

// ---- ENTITY_CANDIDATE and the host's physical record (EEDCS-49, D3): a MOBILE candidate's pose is the ONE `Movers` registry's canonical record, never the campaign's words ----

fn candidate_with_identity(lua: &mlua::Lua, scene: i32, facts: &[f32], identity: &str) -> (String, String, Vec<u32>) {
    let text = if identity.is_empty() {
        candidate_text(facts)
    } else {
        format!("{} {identity}", candidate_text(facts))
    };
    lua.globals().set("FACTS", text).unwrap();
    lua.globals().set("SCENE", scene).unwrap();
    let table: mlua::Table = lua
        .load(r#"return host.model_provider (host.model_store (ROOT)) ("entity_candidate", SCENE, nil, nil, nil, FACTS)"#)
        .eval()
        .expect("the provider answers");
    let pose = table
        .get::<Option<String>>("p")
        .unwrap()
        .map(|text| {
            text.split(',')
                .map(|token| u32::from_str_radix(token.trim_start_matches("0x"), 16).unwrap())
                .collect()
        })
        .unwrap_or_default();
    (
        table.get("result").unwrap(),
        table.get::<Option<String>>("reason").unwrap().unwrap_or_default(),
        pose,
    )
}

/// the facts of a mobile candidate whose box the segment meets, as the campaign sends them once it has read the candidate's placement: its OWN words are deliberately NOT the host record's
fn mobile_facts(campaign_position: [f32; 3], campaign_motion: [f32; 3], campaign_attitude: [f32; 9]) -> [f32; 44] {
    let mut facts = [0.0f32; 44];
    facts[..6].copy_from_slice(&[-100.0, -100.0, -100.0, 100.0, 100.0, 100.0]);
    facts[6] = 1.0;
    facts[12] = 1000.0;
    facts[13..16].copy_from_slice(&campaign_position);
    facts[16..19].copy_from_slice(&campaign_motion);
    facts[19] = 0.1;
    // the UPDATED answer was given (1: false) and the placement facts were read
    facts[20] = 1.0;
    facts[27..36].copy_from_slice(&campaign_attitude);
    facts[43] = 1.0;
    facts
}

fn registry_with(identity: &str, position: [f32; 3], motion: [f32; 3]) -> std::rc::Rc<std::cell::RefCell<eech_world::movement::Movers>> {
    use eech_world::movement::{Kind, Movers, Pose};
    let mut movers = Movers::default();
    let pose = Pose {
        position,
        velocity: 5.0,
        attitude: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    };
    movers.bind(identity, Kind::Routed, pose, Some(motion), None).unwrap();
    std::rc::Rc::new(std::cell::RefCell::new(movers))
}

#[test]
fn a_mobile_candidates_pose_is_the_hosts_canonical_record_and_the_campaigns_words_decide_nothing() {
    let (root, store) = standard();
    let lua = lua_with_store(&root.dir);
    let movers = registry_with("eech-world:7", [0.0, 0.0, 0.0], [0.0, 0.0, 5.0]);
    lua.set_app_data(eech_world::movement::SharedMovers(movers.clone()));
    // the campaign's words are somewhere else entirely (far outside the segment's box, a turned attitude, another vector): were they the candidate's pose, the broad phase would skip it
    let garbage_attitude = [0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
    let words = mobile_facts([9000.0, 0.0, 9000.0], [1.0, 2.0, 3.0], garbage_attitude);
    let (result, reason, pose) = candidate_with_identity(&lua, 1, &words, "eech-world:7");
    assert_eq!(result, "test", "the host record is inside the box: {reason}");
    // the narrow phase's pose is the RECORD's: exactly what the real decision gives when the facts carry the record's own words
    let mut from_record = candidate_words(&words);
    from_record[13..16].copy_from_slice(&[0.0f32, 0.0, 0.0].map(f32::to_bits));
    from_record[16..19].copy_from_slice(&[0.0f32, 0.0, 5.0].map(f32::to_bits));
    from_record[27..36].copy_from_slice(&[1.0f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0].map(f32::to_bits));
    let expected = eech_world::entity_host::candidate_in_store(&store, 1, &from_record).expect("the decision from the record's own words");
    assert_eq!(expected.state, eech_world::entity_host::CandidateState::Test);
    assert_eq!(pose, expected.pose, "the pose the host returns is the record's, bit for bit");
    // and it is NOT what the campaign's words would have given (the RED against a provider that used them)
    let from_campaign = eech_world::entity_host::candidate_in_store(&store, 1, &candidate_words(&words)).expect("the decision from the campaign's words");
    assert_ne!(
        from_campaign.state,
        eech_world::entity_host::CandidateState::Test,
        "the campaign's words alone would have skipped it"
    );
    // a mobile whose words and record agree decides identically (the coherent default stays green)
    let agreeing = mobile_facts([0.0, 0.0, 0.0], [0.0, 0.0, 5.0], [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
    assert_eq!(candidate_with_identity(&lua, 1, &agreeing, "eech-world:7").2, expected.pose);
    // the placement facts not read yet: no attitude is required before the original read it
    let mut early = agreeing;
    early[43] = 0.0;
    early[27..36].copy_from_slice(&[0.0; 9]);
    assert_eq!(candidate_with_identity(&lua, 1, &early, "eech-world:7").0, "need_placement");
    // the record is read at EACH question: when the host moves the mobile (a placement it adopts), the same campaign words decide differently
    movers
        .borrow_mut()
        .step(
            "eech-world:7",
            eech_world::movement::Pose {
                position: [9000.0, 0.0, 9000.0],
                velocity: 5.0,
                attitude: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            },
            eech_world::movement::Request::Place {
                motion_vector: [0.0, 0.0, 5.0],
            },
        )
        .unwrap();
    assert_eq!(
        candidate_with_identity(&lua, 1, &agreeing, "eech-world:7").0,
        "skip",
        "the host moved the mobile away; the campaign's unchanged words did not keep it in the box"
    );
}

fn candidate_words(facts: &[f32; 44]) -> Vec<u32> {
    facts.iter().map(|value| value.to_bits()).collect()
}

#[test]
fn a_mobile_candidate_the_host_holds_no_record_of_is_refused_by_name_and_never_completed_from_the_campaigns_words() {
    let (root, _store) = standard();
    let lua = lua_with_store(&root.dir);
    let facts = mobile_facts([0.0, 0.0, 0.0], [0.0, 0.0, 5.0], [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
    let refused = |lua: &mlua::Lua, identity: &str, why: &str| {
        let (result, reason, pose) = candidate_with_identity(lua, 1, &facts, identity);
        assert_eq!(result, "refused", "{identity:?}: {reason}");
        assert!(reason.contains("candidate record") && reason.contains(why), "{identity:?}: {reason}");
        assert!(pose.is_empty(), "no pose is handed over");
    };
    // a host that keeps no movement records at all
    refused(&lua, "eech-world:7", "keeps no movement records");
    let movers = registry_with("eech-world:7", [0.0, 0.0, 0.0], [0.0, 0.0, 5.0]);
    lua.set_app_data(eech_world::movement::SharedMovers(movers.clone()));
    // the campaign has no identity for the candidate, never issued one, or the host issued it without a record, or retired it
    refused(&lua, "", "no host identity");
    refused(&lua, "eech-world:99", "never issued");
    movers.borrow_mut().issue("eech-world:8").unwrap();
    refused(&lua, "eech-world:8", "no movement record");
    movers.borrow_mut().retire("eech-world:7").unwrap();
    refused(&lua, "eech-world:7", "retired");
    // a registry being stepped is refused by name, never read through a second borrow
    let movers = registry_with("eech-world:7", [0.0, 0.0, 0.0], [0.0, 0.0, 5.0]);
    lua.set_app_data(eech_world::movement::SharedMovers(movers.clone()));
    let guard = movers.borrow_mut();
    let (result, reason, _) = candidate_with_identity(&lua, 1, &facts, "eech-world:7");
    assert_eq!((result.as_str(), reason.contains("candidate record busy")), ("refused", true), "{reason}");
    drop(guard);
    assert_eq!(candidate_with_identity(&lua, 1, &facts, "eech-world:7").0, "test");
    // a candidate that is not a mobile of a sector list needs no record: a fixed one, and a building listed under a city (even one whose flags read as mobile)
    let mut fixed = facts;
    fixed[6] = 0.0;
    fixed[7] = 1.0;
    fixed[42] = 1.0;
    let (_, reason, _) = candidate_with_identity(&lua, 1, &fixed, "");
    assert!(!reason.contains("candidate record"), "a fixed candidate needs no record: {reason}");
    let mut of_city = facts;
    of_city[10] = 1.0;
    let (_, reason, _) = candidate_with_identity(&lua, 1, &of_city, "");
    assert!(
        !reason.contains("candidate record"),
        "a building listed under a city is not a mobile of a sector list: {reason}"
    );
}

/// the same question as `candidate_with_identity`, with the facts as the bit patterns they arrive in (a signalling NaN is never formed as a float by the test either)
fn candidate_words_with_identity(lua: &mlua::Lua, scene: i32, words: &[u32], identity: &str) -> (String, String) {
    let patterns = words.iter().map(|word| format!("0x{word:08x}")).collect::<Vec<_>>().join(",");
    lua.globals()
        .set("FACTS", if identity.is_empty() { patterns } else { format!("{patterns} {identity}") })
        .unwrap();
    lua.globals().set("SCENE", scene).unwrap();
    let table: mlua::Table = lua
        .load(r#"return host.model_provider (host.model_store (ROOT)) ("entity_candidate", SCENE, nil, nil, nil, FACTS)"#)
        .eval()
        .expect("the provider answers");
    (table.get("result").unwrap(), table.get::<Option<String>>("reason").unwrap().unwrap_or_default())
}

#[test]
fn a_malformed_flag_word_is_a_request_error_before_any_record_lookup_whatever_the_registry_holds() {
    let (root, _store) = standard();
    let lua = lua_with_store(&root.dir);
    let movers = registry_with("eech-world:7", [0.0, 0.0, 0.0], [0.0, 0.0, 5.0]);
    lua.set_app_data(eech_world::movement::SharedMovers(movers.clone()));
    let good = candidate_words(&mobile_facts([0.0, 0.0, 0.0], [0.0, 0.0, 5.0], [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]));
    // positive first: the coherent question is decided from the record
    assert_eq!(candidate_words_with_identity(&lua, 1, &good, "eech-world:7").0, "test");
    // a malformed word selects the request error, whatever the registry holds: a record, nothing at all, an unknown or empty identity, or a registry being stepped
    let malformed = [
        (6usize, 0x7fa0_0001u32, "fact 6", "not a finite number"),
        (10, 0xffc0_0000, "fact 10", "not a finite number"),
        (43, 0x7f80_0000, "fact 43", "not a finite number"),
        (13, 0x7fa0_0002, "fact 13", "not a finite number"),
        (6, 0x4000_0000, "fact 6", "flag and must be 0 or 1"),
        (10, 0xbf80_0000, "fact 10", "flag and must be 0 or 1"),
        (43, 0x3f00_0000, "fact 43", "flag and must be 0 or 1"),
        (6, 0x0000_0001, "fact 6", "flag and must be 0 or 1"),
        (20, 0x4040_0000, "fact 20", "UPDATED state"),
    ];
    for (at, word, fact, why) in malformed {
        let mut words = good.clone();
        words[at] = word;
        let guard = movers.borrow_mut();
        let busy = candidate_words_with_identity(&lua, 1, &words, "eech-world:7");
        drop(guard);
        let cases = [
            candidate_words_with_identity(&lua, 1, &words, "eech-world:7"),
            candidate_words_with_identity(&lua, 1, &words, "eech-world:99"),
            candidate_words_with_identity(&lua, 1, &words, ""),
            busy,
        ];
        for (result, reason) in cases {
            assert_eq!(result, "refused", "{fact} {word:08x}: {reason}");
            assert!(reason.contains(fact) && reason.contains(why), "{fact} {word:08x}: {reason}");
            assert!(
                !reason.contains("candidate record"),
                "the request error wins over any record or busy result: {reason}"
            );
        }
    }
    // a host that keeps no registry at all gives the same
    let bare = lua_with_store(&root.dir);
    let mut words = good.clone();
    words[6] = 0x7fa0_0001;
    let (result, reason) = candidate_words_with_identity(&bare, 1, &words, "eech-world:7");
    assert_eq!(result, "refused");
    assert!(reason.contains("fact 6") && !reason.contains("candidate record"), "{reason}");
    // the old accepted semantics are kept: a negative zero is a zero (not a mobile, so no record is asked for), and a finite words elsewhere are not validated further
    let mut zero = good.clone();
    zero[6] = 0x8000_0000;
    zero[7] = 0x0000_0000;
    let (_, reason) = candidate_words_with_identity(&bare, 1, &zero, "");
    assert!(!reason.contains("candidate record") && !reason.contains("flag"), "{reason}");
}

#[test]
fn the_provider_decides_from_the_record_at_every_boundary_of_a_mobiles_lifetime_and_refuses_only_the_released_identity() {
    use eech_world::movement::{Advance, Canonical, Kind, Movers, Pose, Request};
    let (root, store) = standard();
    let lua = lua_with_store(&root.dir);
    let level = [1.0f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    let creation = Pose {
        position: [0.0, 0.0, 0.0],
        velocity: 0.0,
        attitude: level,
    };
    let movers = std::rc::Rc::new(std::cell::RefCell::new(Movers::default()));
    // the bind at creation: the record is there before the first collision question
    movers.borrow_mut().bind("eech-world:7", Kind::Routed, creation, Some([0.0; 3]), None).unwrap();
    lua.set_app_data(eech_world::movement::SharedMovers(movers.clone()));
    // the campaign's words never agree with anything here: only the record decides
    let questions = mobile_facts([5000.0, 0.0, 5000.0], [9.0, 9.0, 9.0], [0.0; 9]);
    let from_record = |movers: &std::rc::Rc<std::cell::RefCell<Movers>>| {
        let Canonical::Record(record) = movers.borrow().canonical("eech-world:7") else {
            panic!("no record")
        };
        let mut words = candidate_words(&questions);
        words[13..16].copy_from_slice(&record.position);
        words[16..19].copy_from_slice(&record.motion);
        words[27..36].copy_from_slice(&record.attitude);
        let step = eech_world::entity_host::candidate_in_store(&store, 1, &words).expect("the decision from the record's own words");
        (format!("{:?}", step.state), step.pose)
    };
    let provider = |lua: &mlua::Lua| {
        let (result, reason, pose) = candidate_with_identity(lua, 1, &questions, "eech-world:7");
        (result, reason, pose)
    };
    let agrees = |lua: &mlua::Lua, movers: &std::rc::Rc<std::cell::RefCell<Movers>>, boundary: &str| {
        let (oracle_state, oracle_pose) = from_record(movers);
        let (result, reason, pose) = provider(lua);
        let expected = match oracle_state.as_str() {
            "Test" => "test",
            "Skip" => "skip",
            other => panic!("{boundary}: unexpected {other}"),
        };
        assert_eq!((result.as_str(), reason.as_str()), (expected, ""), "{boundary}");
        assert_eq!(
            pose,
            if expected == "test" { oracle_pose } else { Vec::new() },
            "{boundary}: the pose is the record's"
        );
    };
    agrees(&lua, &movers, "the first question, before any step");
    // the creation placement, a live step (the real solver) and the skipped-step derivation each move the record, and the next question follows it
    movers
        .borrow_mut()
        .step("eech-world:7", creation, Request::Place { motion_vector: [0.0; 3] })
        .unwrap();
    agrees(&lua, &movers, "after the creation placement");
    let advance = Advance {
        delta_seconds: 1.0,
        desired_velocity: 10.0,
        max_acceleration: 4.0,
        waypoint: [30.0, 0.0, 0.0],
        tacview_logging: false,
        terrain: None,
        map_min: [-1.0e6; 3],
        map_max: [1.0e6; 3],
    };
    let moved = movers.borrow_mut().step("eech-world:7", creation, Request::Advance(advance)).unwrap();
    agrees(&lua, &movers, "after a live step");
    movers.borrow_mut().step("eech-world:7", moved.pose, Request::RoutedVector).unwrap();
    agrees(&lua, &movers, "after the skipped-step derivation");
    // the loss unlinks the member from its group: the host is told nothing, and the identity and the record answer exactly as before
    agrees(&lua, &movers, "after the loss");
    // the release is the only end: only then is the identity refused, by name
    movers.borrow_mut().retire("eech-world:7").unwrap();
    let (result, reason, pose) = provider(&lua);
    assert_eq!(result, "refused");
    assert!(reason.contains("candidate record") && reason.contains("retired") && pose.is_empty(), "{reason}");
}

// ---- FIXED/STATIC COLLISION BACKING through the real provider (EEDCS-49, D3, Task49): the host's record of a placement holds a fixed entity's collision state, and a non-mobile candidate is decided from it ----
//
// The campaign reports the state of every fixed-family placement (phase 13 of the scan question: position, type, heading, shape and the two flags the entity's type derives) and a non-mobile candidate names the
// placement the host served it by (the key in the identity place of every staged question). The provider joins that key to the record of the placement and decides from the record's words; the campaign's own
// words are only compared with it. A placement the host holds no state for, or whose state the host withdrew, is refused by name and never completed from the campaign's words.

const FIXED_STATE_PHASE: u32 = 0x4150_0000;
const FIXED_POSITION: [f32; 3] = [10.0, 0.0, 20.0];
/// ENTITY_TYPE_OBJECT (en_types.h): a member of the fixed family; a building listed in a city or a sector is ENTITY_TYPE_CITY_BUILDING and a city ENTITY_TYPE_CITY
const FIXED_TYPE: f32 = 17.0;
const BUILDING_TYPE: f32 = 6.0;
const CITY_TYPE: i32 = 5;

fn fixed_words(position: [f32; 3], entity_type: f32, heading: f32, shape: f32, collision: f32, align: f32, withdraw: f32) -> [u32; 16] {
    let mut words = [0u32; 16];
    for (at, word) in position.iter().enumerate() {
        words[at] = word.to_bits();
    }
    words[3] = entity_type.to_bits();
    words[4] = heading.to_bits();
    words[5] = shape.to_bits();
    words[7] = collision.to_bits();
    words[8] = align.to_bits();
    words[9] = withdraw.to_bits();
    words[11] = FIXED_STATE_PHASE;
    words
}

/// the standing report of the placement: the position, type 61, heading 0.5, shape 1 (the fixture's small scene), collision-fixed and not aligned
fn fixed_standing() -> [u32; 16] {
    fixed_words(FIXED_POSITION, FIXED_TYPE, 0.5, 1.0, 1.0, 0.0, 0.0)
}

fn scan_words_answer(lua: &mlua::Lua, words: [u32; 16], identity: &str) -> mlua::Table {
    let patterns = words.iter().map(|word| format!("0x{word:08x}")).collect::<Vec<_>>().join(",");
    lua.globals()
        .set("FACTS", if identity.is_empty() { patterns } else { format!("{patterns} {identity}") })
        .unwrap();
    lua.load(r#"return host.model_provider (host.model_store (ROOT)) ("entity_scan", 0, nil, nil, nil, FACTS)"#)
        .eval()
        .expect("the provider answers")
}

fn table_result(table: &mlua::Table) -> (String, String, String) {
    (
        table.get("result").unwrap(),
        table.get::<Option<String>>("name").unwrap().unwrap_or_default(),
        table.get::<Option<String>>("reason").unwrap().unwrap_or_default(),
    )
}

type RosterHandle = std::rc::Rc<std::cell::RefCell<eech_world::roster::SectorRoster>>;

/// a provider with the host's roster holding one placed structure (type 61, in sector (0, 0)) and its key
fn fixed_world(root: &std::path::Path) -> (mlua::Lua, RosterHandle, String) {
    let lua = lua_with_store(root);
    let roster = std::rc::Rc::new(std::cell::RefCell::new(eech_world::roster::SectorRoster::default()));
    let key = roster.borrow_mut().insert((0, 0), FIXED_TYPE as i32, None).to_string();
    lua.set_app_data(eech_world::roster::SharedRoster(roster.clone()));
    (lua, roster, key)
}

/// the facts of a fixed candidate at the stage the campaign has reached: nothing read (0), the fixed facts (1: the flag, the position), or the placement too (2: the heading and the align flag)
fn fixed_facts(stage: u32, position: [f32; 3], heading: f32, align: f32) -> [f32; 44] {
    let mut facts = [0.0f32; 44];
    facts[..6].copy_from_slice(&[-100.0, -100.0, -100.0, 100.0, 100.0, 100.0]);
    facts[12] = 1000.0;
    facts[19] = 0.1;
    if stage >= 1 {
        facts[7] = 1.0;
        facts[13..16].copy_from_slice(&position);
        facts[42] = 1.0;
    }
    if stage >= 2 {
        facts[21] = heading;
        facts[22] = align;
        facts[43] = 1.0;
    }
    facts
}

/// the scene a stage reads (0 until the shape was read)
fn fixed_scene(stage: u32) -> i32 {
    i32::from(stage >= 1)
}

#[test]
fn a_fixed_state_report_is_held_by_the_record_of_the_placement_and_the_staged_candidate_is_decided_from_it() {
    let (root, store) = standard();
    let (lua, _roster, key) = fixed_world(&root.dir);
    // the report is acknowledged by the key it names
    let (result, name, reason) = table_result(&scan_words_answer(&lua, fixed_standing(), &key));
    assert_eq!((result.as_str(), name.as_str()), ("placed", key.as_str()), "{reason}");
    // the same stages the campaign asks, each naming the placement: the first needs nothing of the state, the others are decided from it
    let first = candidate_with_identity(&lua, fixed_scene(0), &fixed_facts(0, FIXED_POSITION, 0.5, 0.0), &key);
    assert_eq!((first.0.as_str(), first.1.as_str()), ("need_fixed", ""));
    let second = candidate_with_identity(&lua, fixed_scene(1), &fixed_facts(1, FIXED_POSITION, 0.5, 0.0), &key);
    assert_eq!((second.0.as_str(), second.1.as_str()), ("need_placement", ""));
    let facts = fixed_facts(2, FIXED_POSITION, 0.5, 0.0);
    let third = candidate_with_identity(&lua, fixed_scene(2), &facts, &key);
    assert_eq!((third.0.as_str(), third.1.as_str()), ("test", ""));
    // the pose the narrow phase is given is the decision over exactly the record's words: the oracle is the host's own kernel over the same facts
    let oracle = eech_world::entity_host::candidate_in_store(&store, 1, &candidate_words(&facts)).expect("the decision");
    assert_eq!(third.2, oracle.pose, "the pose is the record's");
}

#[test]
fn the_campaigns_words_of_a_fixed_candidate_decide_nothing_a_difference_from_the_record_is_a_named_refusal() {
    let (root, _store) = standard();
    let (lua, _roster, key) = fixed_world(&root.dir);
    assert_eq!(table_result(&scan_words_answer(&lua, fixed_standing(), &key)).0, "placed");
    let refused = |facts: &[f32; 44], scene: i32, what: &str| {
        let (result, reason, pose) = candidate_with_identity(&lua, scene, facts, &key);
        assert_eq!(result, "refused", "{what}: {reason}");
        assert!(
            reason.contains("[candidate.mirror]") && reason.contains(what) && pose.is_empty(),
            "{what}: {reason}"
        );
    };
    refused(&fixed_facts(1, [11.0, 0.0, 20.0], 0.5, 0.0), 1, "position");
    refused(&fixed_facts(1, FIXED_POSITION, 0.5, 0.0), 2, "shape");
    refused(&fixed_facts(2, FIXED_POSITION, 0.75, 0.0), 1, "heading");
    refused(&fixed_facts(2, FIXED_POSITION, 0.5, 1.0), 1, "align");
    let mut not_fixed = fixed_facts(1, FIXED_POSITION, 0.5, 0.0);
    not_fixed[7] = 0.0;
    refused(&not_fixed, 1, "collision-fixed");
}

#[test]
fn a_fixed_candidate_the_host_holds_no_state_for_or_withdrew_it_from_is_refused_by_name_and_never_completed_from_the_campaigns_words() {
    let (root, _store) = standard();
    let (lua, _roster, key) = fixed_world(&root.dir);
    let facts = fixed_facts(1, FIXED_POSITION, 0.5, 0.0);
    // the host was never told this placement's state: the campaign says it is fixed, and the host does not take its word for the shape and the position
    let (result, reason, pose) = candidate_with_identity(&lua, 1, &facts, &key);
    assert_eq!(result, "refused", "{reason}");
    assert!(
        reason.contains("[candidate.fixed]") && reason.contains("holds no fixed physical state") && pose.is_empty(),
        "{reason}"
    );
    // told, it decides; the host's own withdrawal of the state (the placement stays standing) refuses the next question by the same name, and the same report restores it
    assert_eq!(table_result(&scan_words_answer(&lua, fixed_standing(), &key)).0, "placed");
    assert_eq!(candidate_with_identity(&lua, 1, &facts, &key).0, "need_placement");
    let withdrawal = fixed_words([0.0; 3], 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    let (result, _, reason) = table_result(&scan_words_answer(&lua, withdrawal, &key));
    assert_eq!(result, "removed", "{reason}");
    let (result, reason, _) = candidate_with_identity(&lua, 1, &facts, &key);
    assert_eq!(result, "refused");
    assert!(reason.contains("[candidate.fixed]") && reason.contains("withdrew"), "{reason}");
    assert_eq!(table_result(&scan_words_answer(&lua, fixed_standing(), &key)).0, "placed");
    assert_eq!(candidate_with_identity(&lua, 1, &facts, &key).0, "need_placement");
}

#[test]
fn every_staged_question_of_a_non_mobile_candidate_names_a_live_placement_of_the_kind_its_branch_has() {
    let (root, _store) = standard();
    let (lua, roster, key) = fixed_world(&root.dir);
    assert_eq!(table_result(&scan_words_answer(&lua, fixed_standing(), &key)).0, "placed");
    for stage in 0..=2 {
        let facts = fixed_facts(stage, FIXED_POSITION, 0.5, 0.0);
        let scene = fixed_scene(stage);
        // no key: the host cannot join the question to a record
        let (result, reason, _) = candidate_with_identity(&lua, scene, &facts, "");
        assert!(
            result == "refused" && reason.contains("[candidate.key]") && reason.contains("names no placement"),
            "stage {stage}: {reason}"
        );
        // a key the host never issued, and one that is not a key at all
        let (result, reason, _) = candidate_with_identity(&lua, scene, &facts, "999");
        assert!(
            result == "refused" && reason.contains("[candidate.key]") && reason.contains("never issued"),
            "stage {stage}: {reason}"
        );
        let (result, reason, _) = candidate_with_identity(&lua, scene, &facts, "eech-world:7");
        assert!(result == "refused" && reason.contains("[candidate.key]"), "stage {stage}: {reason}");
    }
    // a city child is not a sector placement: the question of a building listed in a sector must not name one, and a building under a city (the candidate says so) names its place in the city's list
    let city = roster.borrow_mut().insert((0, 0), CITY_TYPE, None);
    let child = roster.borrow_mut().child_insert(city, None, FIXED_TYPE as i32).unwrap().to_string();
    let (result, reason, _) = candidate_with_identity(&lua, 1, &fixed_facts(1, FIXED_POSITION, 0.5, 0.0), &child);
    assert!(
        result == "refused" && reason.contains("[candidate.key]") && reason.contains("city child"),
        "{reason}"
    );
    let mut of_city = fixed_facts(0, FIXED_POSITION, 0.5, 0.0);
    of_city[10] = 1.0;
    of_city[13..16].copy_from_slice(&FIXED_POSITION);
    let (result, reason, _) = candidate_with_identity(&lua, 1, &of_city, &key);
    assert!(
        result == "refused" && reason.contains("[candidate.key]") && reason.contains("sector entry"),
        "{reason}"
    );
    // an ended placement names nothing
    roster.borrow_mut().remove(key.parse().unwrap()).unwrap();
    let (result, reason, _) = candidate_with_identity(&lua, 1, &fixed_facts(1, FIXED_POSITION, 0.5, 0.0), &key);
    assert!(
        result == "refused" && reason.contains("[candidate.key]") && reason.contains("has ended"),
        "{reason}"
    );
}

#[test]
fn a_malformed_fixed_state_report_is_refused_by_name_before_the_record_changes() {
    let (root, _store) = standard();
    let (lua, roster, key) = fixed_world(&root.dir);
    let refused = |words: [u32; 16], identity: &str, what: &str| {
        let (result, _, reason) = table_result(&scan_words_answer(&lua, words, identity));
        assert_eq!(result, "refused", "{what}");
        assert!(reason.contains(what), "{what}: {reason}");
    };
    let standing = fixed_standing();
    refused(standing, "", "names no placement");
    refused(standing, "999", "never issued");
    refused(standing, "eech-world:1", "not a key");
    let mut wrong_type = standing;
    wrong_type[3] = 62.0f32.to_bits();
    refused(wrong_type, &key, "entity type");
    for (at, word, what) in [
        (0usize, 0x7fa0_0001u32, "position"),
        (1, f32::INFINITY.to_bits(), "position"),
        (2, 2.0e9f32.to_bits(), "position"),
        (4, 0x7fc0_0000, "heading"),
        (5, 1.5f32.to_bits(), "shape"),
        (5, (-1.0f32).to_bits(), "shape"),
        (7, 2.0f32.to_bits(), "collision"),
        (8, 0.5f32.to_bits(), "align"),
        (9, 2.0f32.to_bits(), "withdraw"),
        (6, 1.0f32.to_bits(), "states nothing"),
        (10, 1.0f32.to_bits(), "states nothing"),
        (12, 1.0f32.to_bits(), "states nothing"),
    ] {
        let mut bad = standing;
        bad[at] = word;
        refused(bad, &key, what);
    }
    // a withdrawal states no state, and one that does is refused
    let mixed = fixed_words(FIXED_POSITION, FIXED_TYPE, 0.5, 1.0, 1.0, 0.0, 1.0);
    refused(mixed, &key, "withdrawal");
    // nothing was recorded by any of them
    let (result, reason, _) = candidate_with_identity(&lua, 1, &fixed_facts(1, FIXED_POSITION, 0.5, 0.0), &key);
    assert!(result == "refused" && reason.contains("holds no fixed physical state"), "{reason}");
    assert_eq!(roster.borrow().len(), 1);
}

#[test]
fn the_state_follows_the_record_through_a_relink_the_two_placements_of_a_building_and_ends_with_the_last_placement() {
    let (root, _store) = standard();
    let (lua, roster, key) = fixed_world(&root.dir);
    let facts = fixed_facts(1, FIXED_POSITION, 0.5, 0.0);
    assert_eq!(table_result(&scan_words_answer(&lua, fixed_standing(), &key)).0, "placed");
    // a relink is a removal and a new insertion: a new key, a new record, and the state of the old one is gone with it until the campaign states it again
    roster.borrow_mut().remove(key.parse().unwrap()).unwrap();
    let relinked = roster.borrow_mut().insert((0, 0), FIXED_TYPE as i32, None).to_string();
    assert_ne!(relinked, key);
    let (result, reason, _) = candidate_with_identity(&lua, 1, &facts, &relinked);
    assert!(result == "refused" && reason.contains("holds no fixed physical state"), "{reason}");
    assert_eq!(table_result(&scan_words_answer(&lua, fixed_standing(), &relinked)).0, "placed");
    assert_eq!(candidate_with_identity(&lua, 1, &facts, &relinked).0, "need_placement");
    // a building has two placements, one record: the state stated under its place in a city's list is the state its sector placement decides from once the campaign declares them one entity
    let city = roster.borrow_mut().insert((0, 0), CITY_TYPE, None);
    let child = roster.borrow_mut().child_insert(city, None, FIXED_TYPE as i32).unwrap();
    assert_eq!(table_result(&scan_words_answer(&lua, fixed_standing(), &child.to_string())).0, "placed");
    let sector = roster.borrow_mut().insert((0, 0), FIXED_TYPE as i32, None);
    roster.borrow_mut().link(sector, child).unwrap();
    assert_eq!(
        candidate_with_identity(&lua, 1, &facts, &sector.to_string()).0,
        "need_placement",
        "the sector placement joined the child's record and its state"
    );
    // the child leaves the city's list while the sector placement stays: the record and its state stay with it; the last placement ends both
    roster.borrow_mut().child_remove(city, child).unwrap();
    assert_eq!(candidate_with_identity(&lua, 1, &facts, &sector.to_string()).0, "need_placement");
    roster.borrow_mut().remove(sector).unwrap();
    let (result, reason, _) = candidate_with_identity(&lua, 1, &facts, &sector.to_string());
    assert!(result == "refused" && reason.contains("has ended"), "{reason}");
}

#[test]
fn a_building_listed_under_a_city_is_decided_from_the_state_its_record_holds_and_is_named_by_its_place_in_the_citys_list() {
    let (root, store) = standard();
    let (lua, roster, _sector_key) = fixed_world(&root.dir);
    let city = roster.borrow_mut().insert((0, 0), CITY_TYPE, None);
    let child = roster.borrow_mut().child_insert(city, None, FIXED_TYPE as i32).unwrap().to_string();
    // the building was listed in its city before its sector (cb_creat.c): the state is stated under its place in the city's list
    assert_eq!(table_result(&scan_words_answer(&lua, fixed_standing(), &child)).0, "placed");
    // the first question of a building under a city reads its shape and position (the broad phase), and needs the record's state at once
    let mut facts = fixed_facts(0, FIXED_POSITION, 0.5, 0.0);
    facts[10] = 1.0;
    facts[13..16].copy_from_slice(&FIXED_POSITION);
    let first = candidate_with_identity(&lua, 1, &facts, &child);
    assert_eq!((first.0.as_str(), first.1.as_str()), ("need_placement", ""));
    // the placement stage reads the collision flag with the heading and the align flag
    facts[7] = 1.0;
    facts[21] = 0.5;
    facts[43] = 1.0;
    let placed = candidate_with_identity(&lua, 1, &facts, &child);
    assert_eq!((placed.0.as_str(), placed.1.as_str()), ("test", ""));
    let oracle = eech_world::entity_host::candidate_in_store(&store, 1, &candidate_words(&facts)).expect("the decision");
    assert_eq!(placed.2, oracle.pose, "the pose is the record's");
    // every word of the campaign is compared with the record's, at the stage it was read: the position and shape at the first question, the flag, heading and align flag at the placement
    let refused = |facts: &[f32; 44], scene: i32, what: &str| {
        let (result, reason, _) = candidate_with_identity(&lua, scene, facts, &child);
        assert!(
            result == "refused" && reason.contains("[candidate.mirror]") && reason.contains(what),
            "{what}: {result} {reason}"
        );
    };
    let mut moved = facts;
    moved[13] = 11.0;
    refused(&moved, 1, "position");
    refused(&facts, 2, "shape");
    let mut turned = facts;
    turned[21] = 0.75;
    refused(&turned, 1, "heading");
    let mut aligned = facts;
    aligned[22] = 1.0;
    refused(&aligned, 1, "align");
    let mut unflagged = facts;
    unflagged[7] = 0.0;
    refused(&unflagged, 1, "collision-fixed");
    // a building the host was never told a state for is refused at its first question, whatever the campaign's words are
    let unstated = roster
        .borrow_mut()
        .child_insert(city, Some(child.parse().unwrap()), FIXED_TYPE as i32)
        .unwrap()
        .to_string();
    let (result, reason, _) = candidate_with_identity(&lua, 1, &fixed_of_city(), &unstated);
    assert!(
        result == "refused" && reason.contains("[candidate.fixed]") && reason.contains("holds no fixed physical state"),
        "{reason}"
    );
}

/// the first question of a building under a city, as the campaign sends it
fn fixed_of_city() -> [f32; 44] {
    let mut facts = fixed_facts(0, FIXED_POSITION, 0.5, 0.0);
    facts[10] = 1.0;
    facts[13..16].copy_from_slice(&FIXED_POSITION);
    facts
}

#[test]
fn a_record_that_states_no_collision_decides_a_skip_and_the_campaign_cannot_call_it_fixed() {
    let (root, _store) = standard();
    let (lua, _roster, key) = fixed_world(&root.dir);
    // a fixed-family entity whose type derives no collision (the flag is the record's): the host decides the skip from the record, with no model and no scene read
    let report = fixed_words(FIXED_POSITION, FIXED_TYPE, 0.5, 1.0, 0.0, 0.0, 0.0);
    assert_eq!(table_result(&scan_words_answer(&lua, report, &key)).0, "placed");
    let mut not_fixed = fixed_facts(1, FIXED_POSITION, 0.5, 0.0);
    not_fixed[7] = 0.0;
    not_fixed[13..16].fill(0.0);
    assert_eq!(
        candidate_with_identity(&lua, 0, &not_fixed, &key).0,
        "skip",
        "the record says it is not collision-fixed, and the campaign agrees"
    );
    // a campaign that calls it fixed disagrees with the record: a named difference, never a candidate that is tested
    let (result, reason, pose) = candidate_with_identity(&lua, 1, &fixed_facts(1, FIXED_POSITION, 0.5, 0.0), &key);
    assert!(
        result == "refused" && reason.contains("[candidate.mirror]") && reason.contains("collision-fixed") && pose.is_empty(),
        "{reason}"
    );
}

// ---- the family of the placement decides what its record must hold (EEDCS-49, D3, Task49: ENV-FIXED-CANONICAL-FAMILY-AUTHORITY-1) ----
//
// Which stage of the campaign's dialogue needs the record's fixed state is decided by the placement the host actually holds (its entity type, which the host's own roster recorded at the insertion), never by the
// campaign's fixed, city and building mirrors: a placement of the fixed family needs its state at the stage its fixed facts were read whatever the campaign says of it, a city needs its own at the stage that reads its
// position and box, and a placement of no fixed family needs none. A mirror that contradicts the placement's family is a named refusal; a report of a type that is not of the fixed family is refused before the record changes.

/// the entity types of the fixed family (en_types.h): a building listed in a city, a crater, an object, a pylon, scenery, a bridge segment, a site and an updatable site
const FIXED_FAMILY_TYPES: [(i32, &str); 8] = [
    (6, "city building"),
    (7, "crater"),
    (17, "object"),
    (21, "pylon"),
    (24, "scenic"),
    (26, "segment"),
    (29, "site"),
    (30, "site updatable"),
];
/// entity types that are not of the fixed family: a city (it holds its own state: see the city's tests below), a fixed wing, a helicopter, a person, a sound effect and a weapon
const NOT_FIXED_FAMILY_TYPES: [(i32, &str); 6] = [
    (5, "city"),
    (10, "fixed wing"),
    (14, "helicopter"),
    (19, "person"),
    (32, "sound effect"),
    (37, "weapon"),
];

/// the facts of a candidate whose fixed facts the campaign read and found NOT fixed: the sector skip did not remove it, and nothing else of its branch is read
fn fixed_read_not_fixed() -> [f32; 44] {
    let mut facts = fixed_facts(0, FIXED_POSITION, 0.5, 0.0);
    facts[42] = 1.0;
    facts
}

/// a provider with the host's roster holding one placement of the type, stated nothing
fn placement_of(root: &std::path::Path, entity_type: i32) -> (mlua::Lua, RosterHandle, String) {
    let lua = lua_with_store(root);
    let roster = std::rc::Rc::new(std::cell::RefCell::new(eech_world::roster::SectorRoster::default()));
    let key = roster.borrow_mut().insert((0, 0), entity_type, None).to_string();
    lua.set_app_data(eech_world::roster::SharedRoster(roster.clone()));
    (lua, roster, key)
}

fn refused_family(lua: &mlua::Lua, scene: i32, facts: &[f32; 44], key: &str, what: &str, context: &str) {
    let (result, reason, pose) = candidate_with_identity(lua, scene, facts, key);
    assert_eq!(result, "refused", "{context}: {reason}");
    assert!(
        reason.contains("[candidate.family]") && reason.contains(what) && pose.is_empty(),
        "{context}: {reason}"
    );
}

#[test]
fn a_placement_of_the_fixed_family_the_host_holds_no_state_for_is_refused_when_the_campaign_says_it_is_not_fixed() {
    let (root, _store) = standard();
    for (entity_type, name) in FIXED_FAMILY_TYPES {
        let (lua, _roster, key) = placement_of(&root.dir, entity_type);
        // the campaign read the fixed flag (not collision-fixed): the family's record must hold its state anyway, and the campaign's flag is not the host's word for its absence
        let (result, reason, pose) = candidate_with_identity(&lua, 0, &fixed_read_not_fixed(), &key);
        assert_eq!(
            result, "refused",
            "{name}: the fixed family needs the record's state at its fixed facts: {reason}"
        );
        assert!(
            reason.contains("[candidate.fixed]") && reason.contains("holds no fixed physical state") && pose.is_empty(),
            "{name}: {reason}"
        );
        // the sector skip comes BEFORE the fixed facts were read: that stage needs no state and is not a refusal
        let mut below = fixed_facts(0, FIXED_POSITION, 0.5, 0.0);
        below[12] = -500.0;
        assert_eq!(
            candidate_with_identity(&lua, 0, &below, &key).0,
            "skip",
            "{name}: the early skip is the original's"
        );
        assert_eq!(
            candidate_with_identity(&lua, 0, &fixed_facts(0, FIXED_POSITION, 0.5, 0.0), &key).0,
            "need_fixed",
            "{name}"
        );
    }
}

#[test]
fn the_campaigns_city_and_building_mirrors_cannot_stand_in_for_the_family_of_the_placement_the_host_holds() {
    let (root, _store) = standard();
    // a placement of the fixed family that holds its state, with the campaign calling it a CITY: the state is not bypassed by the mirror, and the contradiction is named
    let (lua, _roster, key) = fixed_world(&root.dir);
    assert_eq!(table_result(&scan_words_answer(&lua, fixed_standing(), &key)).0, "placed");
    let mut as_city = fixed_facts(1, FIXED_POSITION, 0.5, 0.0);
    as_city[8] = 1.0;
    as_city[11] = 1.0;
    refused_family(&lua, 0, &as_city, &key, "city", "an object the campaign calls a city");
    // the same for a placement the host holds no state for, and one that is of no fixed family at all
    let (lua, _roster, never) = placement_of(&root.dir, FIXED_TYPE as i32);
    refused_family(&lua, 0, &as_city, &never, "city", "an object without state the campaign calls a city");
    let (lua, _roster, weapon) = placement_of(&root.dir, 37);
    refused_family(&lua, 0, &as_city, &weapon, "city", "a weapon the campaign calls a city");
    // the campaign calls an object a building listed in a sector (its shape lookup skipped without a state)
    let mut as_building = fixed_facts(1, FIXED_POSITION, 0.5, 0.0);
    as_building[9] = 1.0;
    let (lua, _roster, held) = fixed_world(&root.dir);
    assert_eq!(table_result(&scan_words_answer(&lua, fixed_standing(), &held)).0, "placed");
    refused_family(&lua, 0, &as_building, &held, "city building", "an object the campaign calls a building");
    // a city building the campaign calls an ordinary fixed candidate, and a city the campaign does not call one
    let (lua, _roster, building) = placement_of(&root.dir, BUILDING_TYPE as i32);
    refused_family(
        &lua,
        1,
        &fixed_facts(1, FIXED_POSITION, 0.5, 0.0),
        &building,
        "city building",
        "a building called ordinary",
    );
    let (lua, _roster, city) = placement_of(&root.dir, CITY_TYPE);
    refused_family(&lua, 1, &fixed_facts(1, FIXED_POSITION, 0.5, 0.0), &city, "city", "a city called ordinary");
    // a building listed under a city must be a placement of the fixed family, whose record can hold a state
    let (lua, roster, _key) = placement_of(&root.dir, CITY_TYPE);
    let parent = roster.borrow_mut().insert((0, 0), CITY_TYPE, None);
    let child = roster.borrow_mut().child_insert(parent, None, 37).unwrap().to_string();
    refused_family(&lua, 1, &fixed_of_city(), &child, "fixed family", "a weapon listed under a city");
}

#[test]
fn a_placement_of_no_fixed_family_is_decided_without_a_fixed_state_and_a_city_needs_none_of_the_fixed_familys() {
    let (root, store) = standard();
    // a real city needs no FIXED-FAMILY state: its position and box are its own state (the tests below), stated by its own report, and a city whose own state was stated is decided from that
    let (lua, _roster, city) = placement_of(&root.dir, CITY_TYPE);
    let mut facts = fixed_facts(1, FIXED_POSITION, 0.5, 0.0);
    facts[8] = 1.0;
    facts[11] = 1.0;
    facts[36..39].copy_from_slice(&[0.0, 0.0, 0.0]);
    facts[39..42].copy_from_slice(&[50.0, 50.0, 50.0]);
    let own = city_words(FIXED_POSITION, [0.0, 0.0, 0.0], [50.0, 50.0, 50.0]);
    assert_eq!(table_result(&scan_words_answer(&lua, own, &city)).0, "placed");
    let (result, reason, _) = candidate_with_identity(&lua, 0, &facts, &city);
    let oracle = eech_world::entity_host::candidate_in_store(&store, 0, &candidate_words(&facts)).expect("the decision");
    assert_eq!(
        oracle.state,
        eech_world::entity_host::CandidateState::Buildings,
        "the kernel's own decision over the same facts"
    );
    assert_eq!((result.as_str(), reason.as_str()), ("buildings", ""), "a city is decided from its own state");
    // a placement of no fixed family (a weapon, a sound effect, a mobile kind): no state was ever stated, and it is decided as the campaign says, never refused for the absence
    for (entity_type, name) in NOT_FIXED_FAMILY_TYPES {
        let (lua, _roster, key) = placement_of(&root.dir, entity_type);
        let (result, reason, _) = candidate_with_identity(&lua, 0, &fixed_read_not_fixed(), &key);
        assert_eq!((result.as_str(), reason.as_str()), ("skip", ""), "{name}: not fixed, the original skipped it");
    }
}

#[test]
fn a_fixed_state_report_of_a_type_that_is_not_of_the_fixed_family_is_refused_by_name_and_changes_nothing() {
    let (root, _store) = standard();
    for (entity_type, name) in NOT_FIXED_FAMILY_TYPES {
        let (lua, roster, key) = placement_of(&root.dir, entity_type);
        let report = fixed_words(FIXED_POSITION, entity_type as f32, 0.5, 1.0, 1.0, 0.0, 0.0);
        let (result, _, reason) = table_result(&scan_words_answer(&lua, report, &key));
        assert_eq!(result, "refused", "{name}: the host holds no fixed state for a type of no fixed family");
        assert!(reason.contains("[roster.fixed]") && reason.contains("fixed family"), "{name}: {reason}");
        let held = roster.borrow().placement_fixed(key.parse().unwrap(), false).expect("the placement stands").2;
        assert_eq!(held, eech_world::roster::FixedHold::Never, "{name}: the record was not changed");
    }
    // every member of the family is accepted by the same report
    for (entity_type, name) in FIXED_FAMILY_TYPES {
        let (lua, roster, key) = placement_of(&root.dir, entity_type);
        let report = fixed_words(FIXED_POSITION, entity_type as f32, 0.5, 1.0, 1.0, 0.0, 0.0);
        assert_eq!(table_result(&scan_words_answer(&lua, report, &key)).0, "placed", "{name}");
        let held = roster.borrow().placement_fixed(key.parse().unwrap(), false).expect("the placement stands").2;
        assert!(matches!(held, eech_world::roster::FixedHold::Held(_)), "{name}");
    }
}

// ---- THE CITY'S OWN COLLISION STATE through the real provider (EEDCS-49, D3, Task49: the follow-on of the fixed family) ----
//
// A city carries the fixed record too: its own position and bounding box are what a candidate question about the CITY is decided from once the campaign has read them (the city is fixed and its first building was
// served: a city with none, or not fixed, is skipped before either is read, and the sector skip comes before everything). The host's record of the city's placement holds them (phase 14 of the scan question:
// the position, the box minimum and the box maximum, 9 words), the campaign's own words for the same facts are only compared with it, bit for bit, and the record's words then decide. A city's record the host
// holds no state for, or whose state the host withdrew, is refused by name at that stage and never completed from the campaign's words. The box of a city that has no building yet is the original's INVERTED
// sentinel (the minimum 1.0e6 and the maximum -1.0e6): a valid statement, never a refusal for min > max.

const CITY_STATE_PHASE: u32 = 0x4160_0000;
const CITY_POSITION: [f32; 3] = [10.0, 3.0, 20.0];
const CITY_BOX_MIN: [f32; 3] = [-30.0, -5.0, -40.0];
const CITY_BOX_MAX: [f32; 3] = [50.0, 12.0, 60.0];

fn city_words(position: [f32; 3], min: [f32; 3], max: [f32; 3]) -> [u32; 16] {
    let mut words = [0u32; 16];
    for (at, word) in position.iter().chain(min.iter()).chain(max.iter()).enumerate() {
        words[at] = word.to_bits();
    }
    words[11] = CITY_STATE_PHASE;
    words
}

fn city_standing() -> [u32; 16] {
    city_words(CITY_POSITION, CITY_BOX_MIN, CITY_BOX_MAX)
}

/// the question of a city the campaign read as far as its position and box: fixed read and fixed, a city, its first building served (so its position and box were read)
fn city_facts(position: [f32; 3], min: [f32; 3], max: [f32; 3]) -> [f32; 44] {
    let mut facts = fixed_facts(1, position, 0.0, 0.0);
    facts[8] = 1.0;
    facts[11] = 1.0;
    facts[36..39].copy_from_slice(&min);
    facts[39..42].copy_from_slice(&max);
    facts
}

/// whether a record holds a city's own state (by the variant's name, so the same check reads the same against the source that has none)
fn holds_city_state(held: &eech_world::roster::FixedHold) -> bool {
    format!("{held:?}").starts_with("City(")
}

fn city_refused(lua: &mlua::Lua, facts: &[f32; 44], key: &str, marker: &str, what: &str) {
    let (result, reason, pose) = candidate_with_identity(lua, 0, facts, key);
    assert_eq!(result, "refused", "{what}: {reason}");
    assert!(reason.contains(marker) && reason.contains(what) && pose.is_empty(), "{what}: {reason}");
}

#[test]
fn a_city_the_host_holds_no_state_for_is_refused_where_the_campaign_reads_its_position_and_box() {
    let (root, _store) = standard();
    let (lua, _roster, key) = placement_of(&root.dir, CITY_TYPE);
    // the campaign read the city's position and box (its first building was served): they are not the campaign's word to give, and the host was never told them
    city_refused(
        &lua,
        &city_facts(CITY_POSITION, CITY_BOX_MIN, CITY_BOX_MAX),
        &key,
        "[candidate.city]",
        "holds no collision state",
    );
}

#[test]
fn a_city_state_report_is_held_by_the_record_of_the_placement_and_the_city_is_decided_from_it() {
    let (root, store) = standard();
    let (lua, roster, key) = placement_of(&root.dir, CITY_TYPE);
    let (result, name, reason) = table_result(&scan_words_answer(&lua, city_standing(), &key));
    assert_eq!((result.as_str(), name.as_str()), ("placed", key.as_str()), "{reason}");
    // the stage that needs none of it: the fixed flag's own request
    let first = candidate_with_identity(&lua, 0, &fixed_facts(0, FIXED_POSITION, 0.5, 0.0), &key);
    assert_eq!((first.0.as_str(), first.1.as_str()), ("need_fixed", ""));
    // the stage that reads the position and box: decided from the record's words, which the oracle (the host's own kernel over the same facts) decides identically
    let facts = city_facts(CITY_POSITION, CITY_BOX_MIN, CITY_BOX_MAX);
    let decided = candidate_with_identity(&lua, 0, &facts, &key);
    let oracle = eech_world::entity_host::candidate_in_store(&store, 0, &candidate_words(&facts)).expect("the decision");
    assert_eq!(oracle.state, eech_world::entity_host::CandidateState::Buildings);
    assert_eq!((decided.0.as_str(), decided.1.as_str()), ("buildings", ""));
    // a box the segment does not meet is a skip, decided from the same kind of record
    let (lua2, _roster2, far) = placement_of(&root.dir, CITY_TYPE);
    assert_eq!(
        table_result(&scan_words_answer(&lua2, city_words([1000.0, 3.0, 1000.0], CITY_BOX_MIN, CITY_BOX_MAX), &far)).0,
        "placed"
    );
    let far_facts = city_facts([1000.0, 3.0, 1000.0], CITY_BOX_MIN, CITY_BOX_MAX);
    assert_eq!(candidate_with_identity(&lua2, 0, &far_facts, &far).0, "skip");
    // the record holds the city's own state, and nothing of the fixed family's
    let held = roster
        .borrow()
        .placement_fixed(key.parse().expect("a key"), false)
        .expect("the placement stands")
        .2;
    assert!(holds_city_state(&held), "{held:?}");
}

#[test]
fn the_campaigns_words_of_a_city_decide_nothing_a_difference_from_the_record_is_a_named_refusal() {
    let (root, _store) = standard();
    let (lua, _roster, key) = placement_of(&root.dir, CITY_TYPE);
    assert_eq!(table_result(&scan_words_answer(&lua, city_standing(), &key)).0, "placed");
    city_refused(
        &lua,
        &city_facts([11.0, 3.0, 20.0], CITY_BOX_MIN, CITY_BOX_MAX),
        &key,
        "[candidate.mirror]",
        "position",
    );
    city_refused(
        &lua,
        &city_facts(CITY_POSITION, [-31.0, -5.0, -40.0], CITY_BOX_MAX),
        &key,
        "[candidate.mirror]",
        "bounding box minimum",
    );
    city_refused(
        &lua,
        &city_facts(CITY_POSITION, CITY_BOX_MIN, [50.0, 12.0, 61.0]),
        &key,
        "[candidate.mirror]",
        "bounding box maximum",
    );
    // a signed zero is a different word: the mirror is BITWISE
    let (lua, _roster, zero) = placement_of(&root.dir, CITY_TYPE);
    assert_eq!(
        table_result(&scan_words_answer(&lua, city_words([0.0, 3.0, 20.0], CITY_BOX_MIN, CITY_BOX_MAX), &zero)).0,
        "placed"
    );
    city_refused(
        &lua,
        &city_facts([-0.0, 3.0, 20.0], CITY_BOX_MIN, CITY_BOX_MAX),
        &zero,
        "[candidate.mirror]",
        "position",
    );
}

#[test]
fn a_cities_state_the_host_withdrew_is_refused_by_name_and_only_a_new_statement_restores_it() {
    let (root, _store) = standard();
    let (lua, roster, key) = placement_of(&root.dir, CITY_TYPE);
    let facts = city_facts(CITY_POSITION, CITY_BOX_MIN, CITY_BOX_MAX);
    assert_eq!(table_result(&scan_words_answer(&lua, city_standing(), &key)).0, "placed");
    assert_eq!(candidate_with_identity(&lua, 0, &facts, &key).0, "buildings");
    // the host's own retirement of the record's state (the withdrawal word of phase 13): the placement stays standing and nothing the campaign said changed
    let withdraw = fixed_words([0.0; 3], 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    assert_eq!(table_result(&scan_words_answer(&lua, withdraw, &key)).0, "removed");
    assert_eq!(
        roster.borrow().placement_fixed(key.parse().expect("a key"), false).expect("stands").2,
        eech_world::roster::FixedHold::Withdrawn
    );
    city_refused(&lua, &facts, &key, "[candidate.city]", "withdrew");
    // the stage that never needed it is not refused: the fixed flag's own request
    assert_eq!(
        candidate_with_identity(&lua, 0, &fixed_facts(0, FIXED_POSITION, 0.5, 0.0), &key).0,
        "need_fixed"
    );
    // a new statement of the campaign restores it
    assert_eq!(table_result(&scan_words_answer(&lua, city_standing(), &key)).0, "placed");
    assert_eq!(candidate_with_identity(&lua, 0, &facts, &key).0, "buildings");
}

#[test]
fn a_city_is_skipped_without_its_state_when_it_has_no_building_or_is_not_fixed_and_the_sector_skip_comes_first() {
    let (root, _store) = standard();
    // none of the three stages reads the city's position or box, so the record's absence (or withdrawal) is not a refusal there
    let (lua, _roster, key) = placement_of(&root.dir, CITY_TYPE);
    let mut empty = city_facts(CITY_POSITION, CITY_BOX_MIN, CITY_BOX_MAX);
    empty[11] = 0.0;
    empty[13..16].fill(0.0);
    empty[36..42].fill(0.0);
    assert_eq!(
        candidate_with_identity(&lua, 0, &empty, &key).0,
        "skip",
        "a city without a building is skipped before its position or box is read"
    );
    let mut not_fixed = fixed_facts(1, FIXED_POSITION, 0.5, 0.0);
    not_fixed[7] = 0.0;
    not_fixed[13..16].fill(0.0);
    assert_eq!(
        candidate_with_identity(&lua, 0, &not_fixed, &key).0,
        "skip",
        "a city the campaign reads as not fixed is skipped"
    );
    let mut below = fixed_facts(0, FIXED_POSITION, 0.5, 0.0);
    below[12] = -500.0;
    assert_eq!(candidate_with_identity(&lua, 0, &below, &key).0, "skip");
    // the same after a withdrawal: the host's retirement of the state is only felt where the state is read
    assert_eq!(table_result(&scan_words_answer(&lua, city_standing(), &key)).0, "placed");
    let withdraw = fixed_words([0.0; 3], 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    assert_eq!(table_result(&scan_words_answer(&lua, withdraw, &key)).0, "removed");
    assert_eq!(candidate_with_identity(&lua, 0, &empty, &key).0, "skip");
    assert_eq!(candidate_with_identity(&lua, 0, &below, &key).0, "skip");
}

#[test]
fn a_malformed_city_state_report_is_refused_by_name_before_the_record_changes_and_the_inverted_sentinel_box_is_a_valid_statement() {
    let (root, _store) = standard();
    let (lua, roster, key) = placement_of(&root.dir, CITY_TYPE);
    let held = |roster: &RosterHandle| roster.borrow().placement_fixed(key.parse().expect("a key"), false).expect("stands").2;
    let refuse = |words: [u32; 16], identity: &str, marker: &str, what: &str| {
        let (result, _, reason) = table_result(&scan_words_answer(&lua, words, identity));
        assert_eq!(result, "refused", "{what}");
        assert!(reason.contains(marker), "{what}: {reason}");
        assert_eq!(held(&roster), eech_world::roster::FixedHold::Never, "{what}: nothing was recorded");
    };
    // each word of the state: a signaling NaN, infinity and a position beyond the world are refused by integer classification, before the roster is looked at
    for (at, bad) in [
        (0usize, 0x7fa0_0001u32),
        (2, 0x7f80_0000),
        (1, 0x4e6e_6b29),
        (3, 0x7fc0_0000),
        (5, 0xff80_0000),
        (6, 0x7fa0_0001),
        (8, 0x7f80_0000),
    ] {
        let mut words = city_standing();
        words[at] = bad;
        let marker = if at < 3 { "[roster.position]" } else { "[roster.box]" };
        refuse(words, &key, marker, &format!("word {at}"));
    }
    // a word the report does not state, the withdrawal word (the host's own), a key that is no key, no key, and a key never issued
    for at in [9usize, 10, 12, 13, 14, 15] {
        let mut words = city_standing();
        words[at] = 0x3f80_0000;
        refuse(words, &key, "[roster.request]", &format!("fact {at}"));
    }
    refuse(city_standing(), "", "[roster.key]", "no key");
    refuse(city_standing(), "01", "[roster.key]", "a key with a leading zero");
    refuse(city_standing(), "99", "[roster.city]", "a key never issued");
    // the original's initial box: the minimum above the maximum (1.0e6 and -1.0e6), valid and held as stated
    let sentinel = city_words(CITY_POSITION, [1.0e6; 3], [-1.0e6; 3]);
    let (result, _, reason) = table_result(&scan_words_answer(&lua, sentinel, &key));
    assert_eq!(result, "placed", "{reason}");
    assert!(holds_city_state(&held(&roster)));
    // the stage that reads it compares the words and decides from the record all the same
    let facts = city_facts(CITY_POSITION, [1.0e6; 3], [-1.0e6; 3]);
    assert_eq!(candidate_with_identity(&lua, 0, &facts, &key).1, "");
}

#[test]
fn a_city_state_is_the_cities_alone_and_a_fixed_report_is_not_a_city_s() {
    let (root, _store) = standard();
    // a placement of the fixed family, a mobile kind and a building listed under a city are not cities: refused before their records change
    for (entity_type, name) in FIXED_FAMILY_TYPES
        .iter()
        .chain(NOT_FIXED_FAMILY_TYPES.iter().filter(|(kind, _)| *kind != CITY_TYPE))
    {
        let (lua, roster, key) = placement_of(&root.dir, *entity_type);
        let (result, _, reason) = table_result(&scan_words_answer(&lua, city_standing(), &key));
        assert_eq!(result, "refused", "{name}");
        assert!(reason.contains("[roster.city]") && reason.contains("not a city"), "{name}: {reason}");
        let held = roster
            .borrow()
            .placement_fixed(key.parse().expect("a key"), false)
            .expect("the placement stands")
            .2;
        assert_eq!(held, eech_world::roster::FixedHold::Never, "{name}: the record was not changed");
    }
    let (lua, roster, _sector) = placement_of(&root.dir, CITY_TYPE);
    let parent = roster.borrow_mut().insert((0, 0), CITY_TYPE, None);
    let child = roster
        .borrow_mut()
        .child_insert(parent, None, BUILDING_TYPE as i32)
        .expect("listed")
        .to_string();
    let (result, _, reason) = table_result(&scan_words_answer(&lua, city_standing(), &child));
    assert_eq!(result, "refused");
    assert!(reason.contains("[roster.city]"), "{reason}");
    // and a fixed state report naming a real city is still refused by its own phase: the family's words are not a city's
    let (lua, _roster, key) = placement_of(&root.dir, CITY_TYPE);
    let report = fixed_words(FIXED_POSITION, CITY_TYPE as f32, 0.5, 1.0, 1.0, 0.0, 0.0);
    let (result, _, reason) = table_result(&scan_words_answer(&lua, report, &key));
    assert!(
        result == "refused" && reason.contains("[roster.fixed]") && reason.contains("fixed family"),
        "{reason}"
    );
}

// ---- THE CITY'S BOUNDING-BOX EXPANSION through the real provider (EEDCS-49, D3, Task49: the physical construction the city cut left in the campaign) ----
//
// A building's link expands its city's bounding box (ct_msgs.c update_city_bounding_box): the building's scene bounds (without lines), turned by its heading as two corners, offset by the building's position
// from the city's, and united with the box the city holds (the box never shrinks). The host does that work from the records it already holds: the city's own state (position and current box) and the
// building's fixed state (position, heading, shape), both stated before the link (phases 13 and 14). The campaign asks with phase 15 (the identity place is the city's sector key, one space and the building's
// key; every other fact zero) and the host answers `expanded` with the city's key and the six words of the box it now holds, or refuses by name having changed nothing.

const CITY_EXPAND_PHASE: u32 = 0x4170_0000;

fn expand_words() -> [u32; 16] {
    let mut words = [0u32; 16];
    words[11] = CITY_EXPAND_PHASE;
    words
}

/// the original's inverted initial box (ct_creat.c): minimum 1.0e6 and maximum -1.0e6 on every axis
const SENTINEL_MIN: [f32; 3] = [1.0e6; 3];
const SENTINEL_MAX: [f32; 3] = [-1.0e6; 3];

/// a provider whose roster holds a city (state stated: its position and the inverted box) with one building listed under it (fixed state stated: position, heading, shape): (lua, roster, city, building)
fn city_with_building(root: &std::path::Path, shape: f32, heading: f32, building_position: [f32; 3]) -> (mlua::Lua, RosterHandle, String, String) {
    let (lua, roster, city) = placement_of(root, CITY_TYPE);
    let city_key: u64 = city.parse().expect("a key");
    let state = eech_world::roster::CityState {
        position: bits3(CITY_POSITION),
        box_min: bits3(SENTINEL_MIN),
        box_max: bits3(SENTINEL_MAX),
    };
    roster.borrow_mut().set_city(city_key, state).expect("the city's state");
    let child = roster.borrow_mut().child_insert(city_key, None, BUILDING_TYPE as i32).expect("listed");
    let fixed = eech_world::roster::FixedState {
        position: bits3(building_position),
        heading: heading.to_bits(),
        shape: shape as i32,
        collision_fixed: true,
        align: false,
    };
    roster.borrow_mut().set_fixed(child, BUILDING_TYPE as i32, fixed).expect("the building's state");
    (lua, roster, city, child.to_string())
}

fn expand_answer(lua: &mlua::Lua, city: &str, child: &str) -> mlua::Table {
    scan_words_answer(lua, expand_words(), &format!("{city} {child}"))
}

fn held_city(roster: &RosterHandle, city: &str) -> eech_world::roster::FixedHold {
    roster.borrow().placement_fixed(city.parse().expect("a key"), false).expect("the city stands").2
}

fn hex_words(text: &str) -> Vec<u32> {
    text.split(',')
        .map(|word| u32::from_str_radix(word.trim_start_matches("0x"), 16).expect("a bit pattern"))
        .collect()
}

#[test]
fn a_city_expansion_is_built_by_the_host_from_the_records_it_holds_and_answers_the_box_it_now_holds() {
    let (root, store) = standard();
    // scene 5 is a plain object: its bounds without lines are the object's own, (-4, 2) x (-10, 2) x (-2, 6); heading 0 and no offset turn and move nothing
    let view = store.scene(5).expect("the scene");
    assert_eq!(floats(view.bounding_box2), [-4.0, 2.0, -10.0, 2.0, -2.0, 6.0]);
    let (lua, roster, city, child) = city_with_building(&root.dir, 5.0, 0.0, CITY_POSITION);
    let answer = expand_answer(&lua, &city, &child);
    let (result, name, reason) = table_result(&answer);
    assert_eq!(
        (result.as_str(), name.as_str(), reason.as_str()),
        ("expanded", city.as_str(), ""),
        "the host built the box"
    );
    let words = hex_words(&answer.get::<String>("p").expect("the six words"));
    assert_eq!(floats::<6>(words.clone().try_into().expect("six words")), [-4.0, -10.0, -2.0, 2.0, 2.0, 6.0]);
    // the host's own record holds the same box (and the position it already held): the answer is the record's words, not a computation the campaign may adopt
    let held = held_city(&roster, &city);
    let eech_world::roster::FixedHold::City(state) = held else {
        panic!("{held:?}")
    };
    assert_eq!(state.position, bits3(CITY_POSITION));
    assert_eq!([state.box_min.as_slice(), state.box_max.as_slice()].concat(), words);
}

#[test]
fn a_city_expansion_the_host_cannot_build_is_refused_by_name_and_the_record_is_not_changed() {
    let (root, _store) = standard();
    for (what, shape) in [("the invalid scene", 0.0f32), ("a scene outside the catalogue", 16_777_000.0)] {
        let (lua, roster, city, child) = city_with_building(&root.dir, shape, 0.0, CITY_POSITION);
        let before = held_city(&roster, &city);
        let (result, _, reason) = table_result(&expand_answer(&lua, &city, &child));
        assert_eq!(result, "refused", "{what}");
        assert!(reason.contains("[roster.city_box.shape]"), "{what}: {reason}");
        assert_eq!(held_city(&roster, &city), before, "{what}: the record was not changed");
    }
    // a building whose state the host withdrew (the host's own retirement: the placement stands, only the state goes)
    let (lua, roster, city, child) = city_with_building(&root.dir, 5.0, 0.0, CITY_POSITION);
    let mut withdraw = [0u32; 16];
    withdraw[9] = 1.0f32.to_bits();
    withdraw[11] = FIXED_STATE_PHASE;
    assert_eq!(table_result(&scan_words_answer(&lua, withdraw, &child)).0, "removed");
    let before = held_city(&roster, &city);
    let (result, _, reason) = table_result(&expand_answer(&lua, &city, &child));
    assert_eq!(result, "refused");
    assert!(reason.contains("[roster.city_box.building_state]") && reason.contains("withdrew"), "{reason}");
    assert_eq!(held_city(&roster, &city), before);
    // a building the host was never told the state of
    let (lua, roster, city) = placement_of(&root.dir, CITY_TYPE);
    let child = roster
        .borrow_mut()
        .child_insert(city.parse().expect("a key"), None, BUILDING_TYPE as i32)
        .expect("listed")
        .to_string();
    let (result, _, reason) = table_result(&expand_answer(&lua, &city, &child));
    assert_eq!(result, "refused");
    assert!(
        reason.contains("[roster.city_box.city_state]") || reason.contains("[roster.city_box.building_state]"),
        "{reason}"
    );
}

#[test]
fn a_city_expansion_names_its_keys_and_a_pair_that_does_not_join_is_refused_before_anything_is_read() {
    let (root, _store) = standard();
    let (lua, roster, city, child) = city_with_building(&root.dir, 5.0, 0.0, CITY_POSITION);
    let before = held_city(&roster, &city);
    // another city's building, an unknown key, the child named as the city and the city named as the child, a lone key and a malformed pair
    let other_city = roster.borrow_mut().insert((0, 0), CITY_TYPE, None).to_string();
    for (what, identity) in [
        ("a building of another city", format!("{other_city} {child}")),
        ("an unknown building", format!("{city} 99999")),
        ("the building as the city", format!("{child} {child}")),
        ("the city as the building", format!("{city} {city}")),
        ("a lone key", city.clone()),
        ("a pair with a third key", format!("{city} {child} {child}")),
        ("a key that is not a key", format!("{city} 0{child}")),
    ] {
        let (result, _, reason) = table_result(&scan_words_answer(&lua, expand_words(), &identity));
        assert_eq!(result, "refused", "{what}");
        assert!(reason.contains("[roster.city_box."), "{what}: {reason}");
        assert_eq!(held_city(&roster, &city), before, "{what}: nothing changed");
    }
    // every fact but the phase is zero
    for at in [0usize, 3, 5, 9, 12, 15] {
        let mut words = expand_words();
        words[at] = 1.0f32.to_bits();
        let (result, _, reason) = table_result(&scan_words_answer(&lua, words, &format!("{city} {child}")));
        assert_eq!(result, "refused", "fact {at}");
        assert!(reason.contains("[roster.request]"), "fact {at}: {reason}");
    }
    assert_eq!(held_city(&roster, &city), before);
}

#[test]
fn a_city_state_restated_over_the_box_the_host_built_may_move_the_position_but_never_change_the_box() {
    let (root, _store) = standard();
    let (lua, roster, city, child) = city_with_building(&root.dir, 5.0, 0.0, CITY_POSITION);
    assert_eq!(table_result(&expand_answer(&lua, &city, &child)).0, "expanded");
    let box_min = [-4.0, -10.0, -2.0];
    let box_max = [2.0, 2.0, 6.0];
    // the campaign's own reports repeat the position and the mirror of the box it was given: a moved position is the campaign's geography and is held
    let moved = [11.0, 3.0, 21.0];
    assert_eq!(table_result(&scan_words_answer(&lua, city_words(moved, box_min, box_max), &city)).0, "placed");
    let held = held_city(&roster, &city);
    let eech_world::roster::FixedHold::City(state) = held else {
        panic!("{held:?}")
    };
    assert_eq!(state.position, bits3(moved));
    // a box that is not the host's is not the campaign's word to give
    let before = held_city(&roster, &city);
    let (result, _, reason) = table_result(&scan_words_answer(&lua, city_words(moved, [-5.0, -10.0, -2.0], box_max), &city));
    assert_eq!(result, "refused");
    assert!(reason.contains("[roster.city]") && reason.contains("not the one the host holds"), "{reason}");
    assert_eq!(held_city(&roster, &city), before, "the refused statement changed nothing");
}

// ---- THE CITY BOX KERNEL against the ORIGINAL's own statements (Task49, D3) ----
//
// `ModelStore::city_box_expand` is the host's arithmetic of ct_msgs.c update_city_bounding_box. The control (tests/csrc/city_box_control.c) holds the original's statements and the two maths routines they call, copied
// verbatim; the first test compares that text with the original sources, so the control IS the original under a test entry. The others compare the kernel with it bit for bit and say what the ORIGINAL's arithmetic does
// to the floating-point state (control words, rounding, masks: nothing; the sticky exception flags it raises stay in the engine's state until the engine entry's own restore, and nothing reads them) and what the
// host's kernel does (the caller's COMPLETE state handed back, exactly as every other kernel of this module does).

extern "C" {
    fn eech_city_box_original(bounds: *const f32, heading: f32, building: *const f32, city: *const f32, min: *mut f32, max: *mut f32, report: *mut u32) -> i32;
}

fn campaign_source(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").join(relative);
    String::from_utf8_lossy(&std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).replace("\r\n", "\n")
}

/// the statements of a block as comparable lines: trailing blanks cut, the empty and the comment-only lines dropped
fn statement_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with("//") && !line.trim_start().starts_with("/*"))
        .map(str::to_string)
        .collect()
}

/// the text from the first line that holds `from` to the line that holds `to`, both included
fn between(text: &str, from: &str, to: &str) -> String {
    let start = text.find(from).unwrap_or_else(|| panic!("`{from}` is not there"));
    let end = text[start..].find(to).unwrap_or_else(|| panic!("`{to}` is not there")) + start;
    let end = end + text[end..].find('\n').unwrap_or(text.len() - end);
    text[start..end].to_string()
}

/// a function of the original's `matrix.c`, from its signature to the closing brace at the start of a line, without its `static` (the control's only difference)
fn function_text(text: &str, signature: &str) -> String {
    let start = text.find(signature).unwrap_or_else(|| panic!("`{signature}` is not there"));
    let end = text[start..].find("\n}\n").unwrap() + start + 3;
    text[start..end].to_string()
}

#[test]
fn the_controls_statements_are_the_originals() {
    let control = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/csrc/city_box_control.c"))
        .expect("the control")
        .replace("\r\n", "\n");
    // the expansion's arithmetic, from the bounding box's first corner to the last comparison: the ORIGINAL-fixture branch of ct_msgs.c (after its `#else`), less the two reads of the entity the entry takes as arguments
    let messages = campaign_source("aphavoc/source/entity/fixed/city/ct_msgs.c");
    let original_branch = &messages[messages.find("\n#else").expect("the original-fixture branch")..];
    let original = between(
        original_branch,
        "p1.x = bounding_box->xmin;",
        "if (p2.z > raw->bounding_box_max.z) raw->bounding_box_max.z = p2.z;",
    );
    let original: Vec<String> = statement_lines(&original)
        .into_iter()
        .filter(|line| {
            !line.contains("get_local_entity_float_value (this_building, FLOAT_TYPE_HEADING)")
                && !line.contains("get_local_entity_vec3d_ptr (this_building, VEC3D_TYPE_POSITION)")
        })
        .collect();
    let mine = statement_lines(&between(
        &control,
        "p1.x = bounding_box->xmin;",
        "if (p2.z > raw->bounding_box_max.z) raw->bounding_box_max.z = p2.z;",
    ));
    assert_eq!(
        original.len(),
        30,
        "the original expansion has thirty statements: six corner words, the matrix, two products, three offsets, six additions, twelve comparisons"
    );
    assert_eq!(mine, original, "the control's statements are not the original's");
    // the two maths routines the statements call
    let matrix = campaign_source("modules/maths/matrix.c");
    for signature in [
        "void get_3d_transformation_matrix ( matrix3x3 m, float heading, float pitch, float roll )",
        "void multiply_matrix3x3_vec3d ( vec3d *result, const matrix3x3 m, const vec3d *v )",
    ] {
        let original = function_text(&matrix, signature);
        let copy = function_text(&control, &format!("static {signature}")).replacen("static ", "", 1);
        assert_eq!(copy, original, "{signature}");
    }
    // and the host's own copy of the same two routines is the same text as well (the kernel calls them): the control and the kernel meet in one arithmetic
    let arith = campaign_source("eech-campaign/crates/eech-world/csrc/eech_arith.h");
    let squeeze = |text: String| text.split_whitespace().collect::<Vec<_>>().join("");
    for (signature, mine) in [
        (
            "void get_3d_transformation_matrix ( matrix3x3 m, float heading, float pitch, float roll )",
            "static void get_3d_transformation_matrix(matrix3x3 m, float heading, float pitch, float roll)",
        ),
        (
            "void multiply_matrix3x3_vec3d ( vec3d *result, const matrix3x3 m, const vec3d *v )",
            "static void multiply_matrix3x3_vec3d(vec3d *result, const matrix3x3 m, const vec3d *v)",
        ),
    ] {
        assert_eq!(
            squeeze(function_text(&arith, mine)).replacen("static", "", 1),
            squeeze(function_text(&matrix, signature)),
            "{signature}"
        );
    }
}

/// the original's expansion through the control: the box words it leaves and its report of the floating-point state (x87 control, x87 status, MXCSR before; the same after)
fn original_box(bounds: [u32; 6], heading: u32, building: [u32; 3], city: [u32; 3], min: [u32; 3], max: [u32; 3]) -> ([u32; 3], [u32; 3], [u32; 6]) {
    let (bounds, building, city) = (bounds.map(f32::from_bits), building.map(f32::from_bits), city.map(f32::from_bits));
    let (mut min, mut max) = (min.map(f32::from_bits), max.map(f32::from_bits));
    let mut report = [0u32; 6];
    // SAFETY: the pointers are to locals of six, three and three floats (read) and three, three (read and written) and six words (written).
    let code = unsafe {
        eech_city_box_original(
            bounds.as_ptr(),
            f32::from_bits(heading),
            building.as_ptr(),
            city.as_ptr(),
            min.as_mut_ptr(),
            max.as_mut_ptr(),
            report.as_mut_ptr(),
        )
    };
    assert_eq!(code, 0);
    (min.map(f32::to_bits), max.map(f32::to_bits), report)
}

const HEADINGS: [f32; 9] = [0.0, -0.0, 0.5, -1.25, 3.141_592_7, 6.5, 1.0e-30, 100.0, -7.77];
const OFFSETS: [[f32; 3]; 5] = [[0.0, 0.0, 0.0], [12.5, -3.0, 40.25], [-100.0, 0.0, 7.0], [0.1, 0.2, 0.3], [-0.0, -0.0, -0.0]];
/// the boxes a city holds: the inverted initial box, one an earlier building made, one that contains everything a scene can add, and one of signed zeros (a comparison against an equal word leaves it)
const BOXES: [([f32; 3], [f32; 3]); 4] = [
    ([1.0e6; 3], [-1.0e6; 3]),
    ([-3.0, -9.0, -1.0], [4.0, 3.0, 8.0]),
    ([-1.0e5; 3], [1.0e5; 3]),
    ([0.0, -0.0, 0.0], [-0.0, 0.0, -0.0]),
];

#[test]
fn the_hosts_city_box_is_the_originals_bit_for_bit_over_every_heading_offset_and_box_and_every_caller_rounding_mode() {
    let (_root, store) = standard();
    let original_mxcsr = fpu::mxcsr();
    let original_x87 = fpu::x87_control();
    let mut compared = 0;
    // scenes 5 and 6 are plain objects with different bounds, 8 is a slot no file defines (the original's zero bounding box: a valid scene, not a refusal)
    for scene in [5i64, 6, 8, 9] {
        let view = store.scene(scene).expect("the scene");
        let bounds = view.bounding_box2;
        for heading in HEADINGS {
            for offset in OFFSETS {
                for (min, max) in BOXES {
                    let city = [10.0f32, 3.0, 20.0];
                    let building = [city[0] + offset[0], city[1] + offset[1], city[2] + offset[2]];
                    let (want_min, want_max, _) = original_box(bounds, heading.to_bits(), bits3(building), bits3(city), bits3(min), bits3(max));
                    for rounding in [0u32, 3] {
                        fpu::set_mxcsr(0x1f80 | (rounding << 13) | 0x21);
                        fpu::set_x87_control(0x037f | ((rounding as u16) << 10));
                        let got = store
                            .city_box_expand(scene, heading.to_bits(), bits3(building), bits3(city), bits3(min), bits3(max))
                            .expect("an argument");
                        fpu::set_mxcsr(original_mxcsr);
                        fpu::set_x87_control(original_x87);
                        assert_eq!(
                            got,
                            eech_world::models::CityBox::Expanded { min: want_min, max: want_max },
                            "scene {scene} heading {heading} offset {offset:?} box {min:?} {max:?} rounding {rounding}"
                        );
                        compared += 1;
                    }
                }
            }
        }
    }
    assert_eq!(compared, 4 * 9 * 5 * 4 * 2);
}

#[test]
fn the_original_arithmetic_changes_no_control_word_rounding_or_mask_and_only_raises_sticky_flags() {
    let (_root, store) = standard();
    let bounds = store.scene(5).expect("the scene").bounding_box2;
    let mut flags_seen = 0u32;
    for heading in HEADINGS {
        let (_, _, report) = original_box(
            bounds,
            heading.to_bits(),
            bits3([10.0, 3.0, 20.0]),
            bits3([10.0, 3.0, 20.0]),
            bits3([1.0e6; 3]),
            bits3([-1.0e6; 3]),
        );
        // before and after: the x87 control word and the MXCSR's control and mask bits (everything but the six sticky flags) are what the statements found
        assert_eq!(report[0], report[3], "heading {heading}: the x87 control word");
        assert_eq!(report[2] & !0x3f, report[5] & !0x3f, "heading {heading}: the MXCSR's control and mask bits");
        assert_eq!(report[2] & 0x6000, 0x6000, "the statements ran under round-toward-zero");
        assert_eq!(
            (report[0] & 0x3f, (report[2] >> 7) & 0x3f),
            (0x3f, 0x3f),
            "every exception masked, as the engine's callback context has them"
        );
        flags_seen |= report[5] & 0x3f;
    }
    // what the statements raise is recorded (the inexact flag is the one a sine or a product of unrepresentable numbers raises); nothing in the engine reads a flag, and the engine entry restores the caller's whole state
    eprintln!(
        "the original's statements raised the MXCSR sticky flags {flags_seen:#04x} over {} headings",
        HEADINGS.len()
    );
    assert_eq!(flags_seen & !0x3f, 0);
}

#[test]
fn the_hosts_city_box_hands_the_callers_complete_floating_point_state_back_and_refuses_an_unmasked_exception() {
    let (_root, store) = standard();
    let original_mxcsr = fpu::mxcsr();
    let original_x87 = fpu::x87_control();
    let ask = |store: &ModelStore| {
        store
            .city_box_expand(
                5,
                0.5f32.to_bits(),
                bits3([12.0, 3.0, 25.0]),
                bits3([10.0, 3.0, 20.0]),
                bits3(SENTINEL_MIN),
                bits3(SENTINEL_MAX),
            )
            .expect("an argument")
    };
    let reference = ask(&store);
    assert!(matches!(reference, eech_world::models::CityBox::Expanded { .. }), "{reference:?}");
    for rounding in 0u32..4 {
        // sticky flags set in the caller's state: the kernel hands them back as they were, never clears or adds one
        let mxcsr = 0x1f80 | (rounding << 13) | 0x21;
        fpu::set_mxcsr(mxcsr);
        fpu::set_x87_control(0x037f | ((rounding as u16) << 10));
        let before = fp_state().unwrap();
        let answer = ask(&store);
        let after = fp_state().unwrap();
        fpu::set_mxcsr(original_mxcsr);
        fpu::set_x87_control(original_x87);
        assert_eq!(after, before, "rounding {rounding}: the caller's floating-point state after the query");
        assert_eq!(answer, reference, "rounding {rounding}: the answer depended on the caller's rounding mode");
    }
    // an exception unmasked in the caller is refused, state untouched
    fpu::set_mxcsr(original_mxcsr & !0x80);
    let before = fp_state().unwrap();
    let refused = ask(&store);
    let after = fp_state().unwrap();
    fpu::set_mxcsr(original_mxcsr);
    assert_eq!(refused, eech_world::models::CityBox::Refused("fp_masks"));
    assert_eq!(before, after);
}

#[test]
fn the_hosts_city_box_classifies_every_word_by_its_bits_before_it_computes_and_names_what_it_cannot_build() {
    let (_root, store) = standard();
    let words = |heading: u32, building: [u32; 3], city: [u32; 3], min: [u32; 3], max: [u32; 3]| {
        store.city_box_expand(5, heading, building, city, min, max).expect("an argument")
    };
    let (zero, min, max) = ([0.0f32.to_bits(); 3], bits3(SENTINEL_MIN), bits3(SENTINEL_MAX));
    // an infinity, a quiet NaN and a signalling NaN in any input word: refused before the arithmetic, in every place
    for bad in [0x7f80_0000u32, 0xff80_0000, 0x7fc0_0000, 0x7f80_0001, 0xffa0_0000] {
        let refused = eech_world::models::CityBox::Refused("non_finite_input");
        assert_eq!(words(bad, zero, zero, min, max), refused, "{bad:#x} as the heading");
        assert_eq!(words(0, [bad, 0, 0], zero, min, max), refused, "{bad:#x} as a building word");
        assert_eq!(words(0, zero, [0, 0, bad], min, max), refused, "{bad:#x} as a city word");
        assert_eq!(words(0, zero, zero, [0, bad, 0], max), refused, "{bad:#x} as a box minimum word");
        assert_eq!(words(0, zero, zero, min, [0, 0, bad]), refused, "{bad:#x} as a box maximum word");
    }
    // the invalid scene and a scene outside the catalogue are argument errors, as for every other question
    assert!(matches!(store.city_box_expand(0, 0, zero, zero, min, max), Err(QueryError::BadScene(0))));
    assert!(matches!(
        store.city_box_expand(i64::from(scene_last() as i32), 0, zero, zero, min, max),
        Err(QueryError::BadScene(_))
    ));
    // a slot no file defines is the original's zero bounding box: the box is the building's own place, never a refusal
    let answer = store
        .city_box_expand(8, 0, bits3([4.0, 5.0, 6.0]), bits3([1.0, 1.0, 1.0]), min, max)
        .expect("a scene");
    assert_eq!(
        answer,
        eech_world::models::CityBox::Expanded {
            min: bits3([3.0, 4.0, 5.0]),
            max: bits3([3.0, 4.0, 5.0])
        }
    );
}
