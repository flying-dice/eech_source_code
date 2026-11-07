//! A minimal model installation for the tests that drive the REAL provider's roster with a scripted engine: the files `ModelStore::load` needs, written here in the original's formats from tiny
//! hand-made objects (the same writers as tests/model_target_point.rs, reduced to what a store with one scene needs). Scene 1 is a box whose top is 1.0; every other scene slot no file defines is the original's zero
//! bounding box, so the height raise the host makes at an arrival is `position.y + 1.0` for a structure of shape 1 and `position.y + 0` for any other shape, which these tests can state exactly.
use eech_world::models::old_last;
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

/// one line object: points, the bounding box, (flags word, faces, colour word) per surface, the face point list and the surface-point references
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

/// a 3dobjdb.bin scene record with a main object and nothing else: no sub-object, no named sub-object, no animation
fn scene_record(scene: i32, main: i32) -> Vec<u8> {
    let mut w = W::default();
    for _ in 0..16 {
        w.i32(-1);
    }
    w.i32(scene).i32(0); // scene_index, self_shadows
    w.i32(0).i32(0).i32(0).i32(0); // lights, cameras, links, sprite lights
    w.i32(0).i32(0).i32(0); // sub-objects, named sub-objects, texture animations
    w.i32(0).i32(main); // approximations, main object
    w.i32(0).i32(0).floats(&[1.0, 1.0, 1.0]); // shadow_approximation_index, shadow polygon object, scale
    w.i32(1); // collision object
    w.i32(1);
    w.i32(0).i32(0).floats(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]).floats(&[0.0, 0.0, 0.0]); // keyframe 0: rest
    w.f32(1.0).i32(0);
    w.i32(0).i32(0); // displacement sequence index, its keyframes
    w.i32(0); // scene-level indices
    w.i32(0); // top-level sub-objects
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
        let name = eech_world::models::compiled_scene_name(scene).expect("a compiled scene name");
        w.i32(name.len() as i32 + 1).bytes(name.as_bytes()).bytes(&[0]).f32(0.0);
    }
    w.0
}

/// A temporary installation root that is removed when it is dropped.
pub struct Root {
    pub dir: PathBuf,
}

impl Root {
    /// An installation with one object (a box of half-width 1) and one scene, scene 1, whose main object it is: the top of scene 1 is 1.0 and every other scene slot is the original's zero bounding box.
    pub fn minimal() -> Root {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "eech-world-minimal-root-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let data = dir.join("cohokum").join("3ddata");
        std::fs::create_dir_all(&data).expect("the fixture directory");
        let (bin, pts, sp) = table(&[mesh([-1.0, 1.0, -1.0, 1.0, -1.0, 1.0])]);
        std::fs::write(data.join("bininfo.bin"), bininfo()).expect("bininfo");
        std::fs::write(data.join("3dobjs.bin"), bin).expect("3dobjs.bin");
        std::fs::write(data.join("3dobjs.pts"), pts).expect("3dobjs.pts");
        std::fs::write(data.join("3dobjs.sp"), sp).expect("3dobjs.sp");
        std::fs::write(data.join("3dobjdb.bin"), database(&[scene_record(1, 1)])).expect("3dobjdb.bin");
        Root { dir }
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
