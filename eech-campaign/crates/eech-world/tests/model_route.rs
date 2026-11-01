//! The host-owned ROUTE service of the collision-model store (EEDCS-50, D3; src/models.rs `route_node` / `route_face`) against minimal format fixtures.
//!
//! These are contract guards for the route lookup and the ordered node / line-face cursors, not an oracle for any retail route: the files are written here in the original's
//! formats from tiny hand-made objects. What is asserted is the lookup the original's `initialise_scene_quick_sub_object_search` makes (scene-level records first, then each
//! top-level sub-object's own records before its children), the order and indirection of the line faces, the colour a face carries, and the difference between a true
//! absence, the end of a cursor, an argument error and a refusal. The node arithmetic is checked only where it needs no formula: scales of exactly 1.0 and 2.0 return the
//! short coordinates unchanged and doubled. What the retail data answers is the Root's comparison with the retained original route capture, never these tests.
use eech_world::models::{
    compiled_scene_name, compiled_subobject_name, old_last, subobject_last, AnimationRef, ModelStore, ObjectFacts, ObjectHeader, QueryError, RouteFace, RouteNode, ROLE_APPROXIMATION,
    ROLE_COLLISION, ROLE_MAIN, ROLE_SHADOW, ROLE_SUB_OBJECT,
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

/// a line object: points, its bounding box, (flags word, faces, colour word) per surface, the face point list and the surface-point references
struct Route {
    points: Vec<[i16; 3]>,
    bb: [f32; 6],
    surfaces: Vec<(u32, u16, u32)>,
    list: Vec<u8>,
    sp: Vec<u16>,
}

const COLOUR_A: u32 = 0x00A0_B0C0; // red 0xa0, green 0xb0, blue 0xc0
const COLOUR_B: u32 = 0x0001_0203; // red 1, green 2, blue 3

/// three line faces: surface A (3 points, 2 faces) joins nodes 0-1 and 1-2, surface B (2 points, 1 face) joins nodes 2-3 through its own run of the references
fn route(scale_max: f32) -> Route {
    Route {
        points: vec![[0, 0, 0], [32767, 0, 0], [0, -32767, 5], [1000, 2000, -3000]],
        bb: [0.0, scale_max, -scale_max, 0.0, -scale_max, scale_max],
        surfaces: vec![(3 << 24, 2, COLOUR_A), (2 << 24, 1, COLOUR_B)],
        list: vec![0, 1, 1, 2, 0, 1],
        sp: vec![0, 1, 2, 2, 3],
    }
}

fn table(objects: &[Route]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
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

fn keyframe(w: &mut W) {
    w.i32(0).i32(0).floats(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0]);
}

/// one sub-object record: its object, its (kind, offset) records, its children
struct Sub {
    object: i32,
    indices: Vec<(i32, i32)>,
    children: Vec<Sub>,
}

fn sub_record(w: &mut W, s: &Sub) {
    w.i32(s.object).i32(0).i32(0).i32(0).i32(0); // object, contributes, in level, out level, no limits
    w.i32(1);
    keyframe(w);
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
fn scene_record(scene: i32, main: i32, collision: i32, indices: &[(i32, i32)], subs: &[Sub]) -> Vec<u8> {
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
    w.i32(collision);
    w.i32(1);
    keyframe(&mut w);
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
    fn new(objects: &[Route], scenes: &[Vec<u8>]) -> Root {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "eech-world-model-route-{}-{}",
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

// three kinds the campaign selects, by their compiled names, and two more
fn kind(name: &str) -> i64 {
    (1..subobject_last())
        .find(|k| compiled_subobject_name(*k) == Some(name))
        .unwrap_or_else(|| panic!("no compiled sub-object kind {name}")) as i64
}

/// objects: 1 a route at scale 1.0, 2 the same route at scale 2.0, 3 the same with a face point list that is too short to back every face.
/// scene 1: scene-level records (landing route -> top-level 1, takeoff route -> top-level 0) and top-level 0 (object 3) holds a record of its own (the holding route ->
/// its child 0, object 2) AND a record for the landing route again, which must not win; top-level 1 is object 1. scene 2: a main object and no sub-objects.
fn standard() -> (Root, ModelStore) {
    let mut broken = route(32767.0);
    broken.list.truncate(3);
    let landing = kind("FIXED_WING_LANDING_ROUTE");
    let takeoff = kind("FIXED_WING_TAKEOFF_ROUTE");
    let holding = kind("FIXED_WING_LANDING_HOLDING_ROUTE");
    let subs = vec![
        Sub {
            object: 3,
            indices: vec![(holding as i32, 0), (landing as i32, 0)],
            children: vec![Sub {
                object: 2,
                indices: vec![],
                children: vec![],
            }],
        },
        Sub {
            object: 1,
            indices: vec![],
            children: vec![],
        },
    ];
    let root = Root::new(
        &[route(32767.0), route(65534.0), broken],
        &[
            scene_record(1, 1, 1, &[(landing as i32, 1), (takeoff as i32, 0)], &subs),
            scene_record(2, 1, 1, &[], &[]),
        ],
    );
    let store = ModelStore::load(&root.dir).expect("the fixture installation loads");
    (root, store)
}

fn bits(values: [f32; 3]) -> [u32; 3] {
    [values[0].to_bits(), values[1].to_bits(), values[2].to_bits()]
}

fn nodes(store: &ModelStore, scene: i64, kind: i64) -> Vec<RouteNode> {
    let mut out = Vec::new();
    for index in 0.. {
        let answer = store.route_node(scene, kind, index).unwrap();
        let end = !matches!(answer, RouteNode::Found(_));
        out.push(answer);
        if end {
            break;
        }
    }
    out
}

fn faces(store: &ModelStore, scene: i64, kind: i64) -> Vec<RouteFace> {
    let mut out = Vec::new();
    for index in 0.. {
        let answer = store.route_face(scene, kind, index).unwrap();
        let end = !matches!(answer, RouteFace::Found { .. });
        out.push(answer);
        if end {
            break;
        }
    }
    out
}

#[test]
fn the_scene_level_record_resolves_a_kind_and_its_nodes_come_in_order_then_the_end() {
    let (_root, store) = standard();
    let landing = kind("FIXED_WING_LANDING_ROUTE");
    // the scene-level record names top-level 1 (object 1, scale exactly 1.0): the shorts come back unchanged, as floats
    assert_eq!(
        nodes(&store, 1, landing),
        vec![
            RouteNode::Found(bits([0.0, 0.0, 0.0])),
            RouteNode::Found(bits([32767.0, 0.0, 0.0])),
            RouteNode::Found(bits([0.0, -32767.0, 5.0])),
            RouteNode::Found(bits([1000.0, 2000.0, -3000.0])),
            RouteNode::End,
        ]
    );
    // the record in top-level 0 for the same kind was filed after the scene-level one, so it does not win
    assert_ne!(nodes(&store, 1, landing)[2], RouteNode::Refused("unbacked_points"));
}

#[test]
fn a_record_inside_a_sub_object_resolves_its_child_and_the_scale_follows_that_objects_bounds() {
    let (_root, store) = standard();
    let holding = kind("FIXED_WING_LANDING_HOLDING_ROUTE");
    // top-level 0 holds the record; its child 0 is object 2, whose bounds give the scale exactly 2.0
    assert_eq!(
        nodes(&store, 1, holding),
        vec![
            RouteNode::Found(bits([0.0, 0.0, 0.0])),
            RouteNode::Found(bits([65534.0, 0.0, 0.0])),
            RouteNode::Found(bits([0.0, -65534.0, 10.0])),
            RouteNode::Found(bits([2000.0, 4000.0, -6000.0])),
            RouteNode::End,
        ]
    );
}

#[test]
fn the_line_faces_come_surface_by_surface_through_each_surfaces_own_references_with_its_colour() {
    let (_root, store) = standard();
    let landing = kind("FIXED_WING_LANDING_ROUTE");
    let face = |point: [u32; 2], colour: [u8; 3]| RouteFace::Found {
        point,
        colour,
        polygons: false,
    };
    assert_eq!(
        faces(&store, 1, landing),
        vec![
            face([0, 1], [0xa0, 0xb0, 0xc0]),
            face([1, 2], [0xa0, 0xb0, 0xc0]),
            // surface B's references 0 and 1 resolve through B's own run of the surface points (after A's three), not through A's
            face([2, 3], [1, 2, 3]),
            RouteFace::End,
        ]
    );
}

#[test]
fn an_absence_is_not_an_end_not_an_argument_error_and_not_a_refusal() {
    let (_root, store) = standard();
    let landing = kind("FIXED_WING_LANDING_ROUTE");
    let unused = kind("SHIP_LANDING_ROUTE");
    // a kind no record files, a scene with no sub-objects, and a scene the files never described
    for (scene, kind) in [(1, unused), (2, landing), (3, landing)] {
        assert_eq!(store.route_node(scene, kind, 0).unwrap(), RouteNode::Absent, "scene {scene} kind {kind}");
        assert_eq!(store.route_face(scene, kind, 0).unwrap(), RouteFace::Absent, "scene {scene} kind {kind}");
    }
    // the end is a cursor past the last node or face of a route that exists
    assert_eq!(store.route_node(1, landing, 4).unwrap(), RouteNode::End);
    assert_eq!(store.route_node(1, landing, 1_000_000).unwrap(), RouteNode::End);
    assert_eq!(store.route_face(1, landing, 3).unwrap(), RouteFace::End);
    // arguments the original could never produce are refused as arguments, before the data is touched
    assert!(matches!(store.route_node(0, landing, 0), Err(QueryError::BadScene(0))));
    assert!(matches!(store.route_node(1, 0, 0), Err(QueryError::BadKind(0))));
    assert!(matches!(store.route_node(1, subobject_last() as i64, 0), Err(QueryError::BadKind(_))));
    assert!(matches!(store.route_face(1, landing, -1), Err(QueryError::BadDepth(-1))));
    assert!(matches!(store.route_face(1, landing, i64::from(i32::MAX) + 1), Err(QueryError::BadDepth(_))));
}

fn facts(scale: f32) -> ObjectFacts {
    let bounds = [0.0, scale, -scale, 0.0, -scale, scale];
    ObjectFacts {
        counts: [4, 3, 2, 0, 0, 0],
        radius: 1.0f32.to_bits(),
        bounding_box: bounds.map(f32::to_bits),
        bounding_box2: bounds.map(f32::to_bits),
    }
}

#[test]
fn an_objects_header_facts_are_the_files_true_values_by_the_role_it_has_in_the_scene() {
    let (_root, store) = standard();
    // scene 1: main object 1 (scale 1.0), collision object 1, no approximation, no shadow object; sub-object array [object 3, object 1, object 2 (a child of the first)]
    assert_eq!(store.object_header(1, ROLE_MAIN, 0).unwrap(), ObjectHeader::Found(facts(32767.0)));
    assert_eq!(store.object_header(1, ROLE_COLLISION, 0).unwrap(), ObjectHeader::Found(facts(32767.0)));
    assert_eq!(store.object_header(1, ROLE_SUB_OBJECT, 0).unwrap(), ObjectHeader::Found(facts(32767.0)));
    assert_eq!(store.object_header(1, ROLE_SUB_OBJECT, 1).unwrap(), ObjectHeader::Found(facts(32767.0)));
    assert_eq!(store.object_header(1, ROLE_SUB_OBJECT, 2).unwrap(), ObjectHeader::Found(facts(65534.0)));
    // the file names no shadow object (0): no object in that role
    assert_eq!(store.object_header(1, ROLE_SHADOW, 0).unwrap(), ObjectHeader::Absent);
    // a cursor past the scene's own arrays is a refusal, not an absence and not a zero object
    assert_eq!(store.object_header(1, ROLE_APPROXIMATION, 0).unwrap(), ObjectHeader::Refused("approximation_outside"));
    assert_eq!(store.object_header(1, ROLE_SUB_OBJECT, 3).unwrap(), ObjectHeader::Refused("sub_object_outside"));
    // a scene the files never described has no object in any role
    assert_eq!(store.object_header(3, ROLE_MAIN, 0).unwrap(), ObjectHeader::Absent);
    assert_eq!(store.object_header(3, ROLE_COLLISION, 0).unwrap(), ObjectHeader::Absent);
    // arguments the campaign could never produce
    assert!(matches!(store.object_header(0, ROLE_MAIN, 0), Err(QueryError::BadScene(0))));
    assert!(matches!(store.object_header(1, 9, 0), Err(QueryError::BadRole(9))));
    assert!(matches!(store.object_header(1, ROLE_MAIN, -1), Err(QueryError::BadDepth(-1))));
}

#[test]
fn the_object_count_is_the_files_and_a_table_scene_has_no_animation_references() {
    let (_root, store) = standard();
    // a count of what the files hold; the room the campaign keeps for custom objects is its own policy and is not part of the answer
    assert_eq!(store.common_objects(), 3);
    // a scene of the binary database keeps its animation list in its own record: the host has none to give; a scene the files never described neither
    assert_eq!(store.animation_ref(1, 0).unwrap(), AnimationRef::Absent);
    assert_eq!(store.animation_ref(3, 0).unwrap(), AnimationRef::Absent);
    assert!(matches!(store.animation_ref(1, -1), Err(QueryError::BadDepth(-1))));
}

#[test]
fn data_that_cannot_be_shown_to_be_backed_is_refused_by_name_never_invented() {
    let (_root, store) = standard();
    let takeoff = kind("FIXED_WING_TAKEOFF_ROUTE");
    // the scene-level record names top-level 0 (object 3), whose face point list holds only three of the six references the three faces need: face 0 is backed, 1 and 2 are not
    assert!(matches!(store.route_face(1, takeoff, 0).unwrap(), RouteFace::Found { point: [0, 1], .. }));
    assert_eq!(store.route_face(1, takeoff, 1).unwrap(), RouteFace::Refused("unbacked_point_list"));
    assert_eq!(store.route_face(1, takeoff, 2).unwrap(), RouteFace::Refused("unbacked_point_list"));
    // its points are backed, so its nodes answer
    assert!(matches!(store.route_node(1, takeoff, 3).unwrap(), RouteNode::Found(_)));
}

#[test]
fn the_compiled_kind_names_are_the_originals_list() {
    assert!(subobject_last() > 500, "{}", subobject_last());
    assert_eq!(compiled_subobject_name(0), None);
    assert_eq!(compiled_subobject_name(subobject_last()), None);
    assert_eq!(compiled_subobject_name(1), Some("TAIL_ROTOR_HUB"));
    assert_eq!(
        compiled_subobject_name(kind("FIXED_WING_LANDING_ROUTE") as usize),
        Some("FIXED_WING_LANDING_ROUTE")
    );
}
