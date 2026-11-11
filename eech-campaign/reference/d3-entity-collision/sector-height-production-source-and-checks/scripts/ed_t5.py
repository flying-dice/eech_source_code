p = "support/model_root.rs"
d = open(p, "rb").read().decode("utf-8")

def rep(old, new):
    global d
    assert d.count(old) == 1, old[:70]
    d = d.replace(old, new)

rep("""fn database() -> Vec<u8> {
    let mut w = W::default();
    w.i32(0);
    for _ in 0..9 {
        w.i32(0);
    }
    w.0
}""", """/// a 3dobjdb.bin scene record with a main object and nothing else: no sub-object, no named sub-object, no animation
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
}""")
rep("""std::fs::write(data.join("3dobjdb.bin"), database()).expect("3dobjdb.bin");""", """std::fs::write(data.join("3dobjdb.bin"), database(&[scene_record(1, 1)])).expect("3dobjdb.bin");""")
rep("""    /// An installation with one object and no scene of its own: every scene slot is the zero bounding box.""", """    /// An installation with one object (a box of half-width 1) and one scene, scene 1, whose main object it is: the top of scene 1 is 1.0 and every other scene slot is the original's zero bounding box.""")
rep("""//! hand-made objects (the same writers as tests/model_target_point.rs, reduced to what a store with no custom scene needs). Every scene slot no file defines is the original's zero bounding box, so a
//! structure of any shape arrives at a sector with a top of zero: the height raise the host makes at an arrival is then `position.y + 0`, which these tests can state exactly.""", """//! hand-made objects (the same writers as tests/model_target_point.rs, reduced to what a store with one scene needs). Scene 1 is a box whose top is 1.0; every other scene slot no file defines is the original's zero
//! bounding box, so the height raise the host makes at an arrival is `position.y + 1.0` for a structure of shape 1 and `position.y + 0` for any other shape, which these tests can state exactly.""")
open(p, "wb").write(d.encode("utf-8"))
print("ok")
