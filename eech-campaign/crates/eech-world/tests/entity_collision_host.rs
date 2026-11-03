//! The host's narrow phase of the entity collision query (EEDCS-49, D3; src/entity_host.rs over csrc/entity_step.c) against hand-made models.
//!
//! These are contract guards for what the original's `accurate_stationary_bounding_box_collision_test` and `accurate_moving_bounding_box_collision_test` make observable, not an oracle
//! for any retail model: which of the model's faces are tested (its collision triangles one depth at a time, its bounding box when it has none), that the model is placed by the
//! candidate's attitude and position, which sweep positions of a moving candidate are tested for which speed, that the first triangle hit ends the test, that a miss still leaves the
//! writes the intersection helpers made, that a refusal of the model is never a miss, and that the answer does not depend on the rounding mode of the caller. What the retail models
//! answer is the native comparison with the retained original, never these tests.

use eech_world::entity_host::{collide, EntityTest, MOVING_POSE, STATIONARY_POSE};
use eech_world::trees::{fp_state, rounding, Rounding, ScopedRounding};

const IDENTITY: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
/// the point and normal the caller holds going in: distinct values no intersection produces
const POINT_IN: [f32; 3] = [7.0, 8.0, 9.0];
const NORMAL_IN: [f32; 3] = [-7.0, -8.0, -9.0];
const CUBE: [f32; 6] = [-1.0, 1.0, -1.0, 1.0, -1.0, 1.0];

fn words(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

fn floats(words: &[u32]) -> Vec<f32> {
    words.iter().map(|word| f32::from_bits(*word)).collect()
}

fn stationary(old: [f32; 3], new: [f32; 3], position: [f32; 3], attitude: [f32; 9]) -> Vec<u32> {
    let pose = [
        old.as_slice(),
        new.as_slice(),
        POINT_IN.as_slice(),
        NORMAL_IN.as_slice(),
        position.as_slice(),
        attitude.as_slice(),
    ]
    .concat();
    assert_eq!(pose.len(), STATIONARY_POSE);
    words(&pose)
}

fn moving(old: [f32; 3], new: [f32; 3], from: [f32; 3], to: [f32; 3], attitude: [f32; 9], speed: f32) -> Vec<u32> {
    let pose = [
        old.as_slice(),
        new.as_slice(),
        POINT_IN.as_slice(),
        NORMAL_IN.as_slice(),
        from.as_slice(),
        to.as_slice(),
        attitude.as_slice(),
        &[speed],
    ]
    .concat();
    assert_eq!(pose.len(), MOVING_POSE);
    words(&pose)
}

/// A model whose collision triangles are `triangles` (none: the bounding box is tested); returns the answer and every depth the kernel asked for, in order.
fn run(pose: &[u32], bounds: [f32; 6], triangles: &[[f32; 9]]) -> (Result<EntityTest, String>, Vec<i32>) {
    let mut asked = Vec::new();
    let answer = collide(pose, words(&bounds).try_into().unwrap(), &mut |depth| {
        asked.push(depth);
        Ok(usize::try_from(depth)
            .ok()
            .and_then(|at| triangles.get(at))
            .map(|triangle| words(triangle).try_into().unwrap()))
    });
    (answer, asked)
}

/// the corners of a right triangle in the plane y = 0
const FLOOR: [f32; 9] = [0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0, 4.0];

fn point(test: &EntityTest) -> Vec<f32> {
    floats(&test.words[..3])
}

fn normal(test: &EntityTest) -> Vec<f32> {
    floats(&test.words[3..])
}

#[test]
fn a_segment_through_a_collision_triangle_hits_it_at_the_crossing_with_the_faces_normal() {
    let pose = stationary([1.0, 1.0, 1.0], [1.0, -1.0, 1.0], [0.0; 3], IDENTITY);
    let (answer, asked) = run(&pose, CUBE, &[FLOOR]);
    let test = answer.unwrap();
    assert!(test.hit);
    let at = point(&test);
    assert!((at[0] - 1.0).abs() < 1e-4 && at[1].abs() < 1e-4 && (at[2] - 1.0).abs() < 1e-4, "{at:?}");
    let n = normal(&test);
    assert!(n[0].abs() < 1e-4 && (n[1].abs() - 1.0).abs() < 1e-4 && n[2].abs() < 1e-4, "{n:?}");
    assert_eq!(asked, [0], "the first triangle that is hit ends the test: no later depth is read");
}

#[test]
fn the_triangle_is_placed_by_the_candidates_position_and_attitude() {
    // the same floor triangle, moved to x = 20, then turned so that its plane faces x
    let ray = |x: f32| ([x, 1.0, 1.0], [x, -1.0, 1.0]);
    let (old, new) = ray(21.0);
    let (answer, _) = run(&stationary(old, new, [20.0, 0.0, 0.0], IDENTITY), CUBE, &[FLOOR]);
    assert!(answer.unwrap().hit, "translated");
    let (old, new) = ray(1.0);
    let (answer, _) = run(&stationary(old, new, [20.0, 0.0, 0.0], IDENTITY), CUBE, &[FLOOR]);
    assert!(!answer.unwrap().hit, "the unmoved ray no longer meets the moved model");
    // a quarter turn (row-vector convention): the model's y axis becomes x, so the floor now stands in the plane x = 0 and a ray along x crosses it
    let quarter = [0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
    let (answer, _) = run(&stationary([-1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [0.0; 3], quarter), CUBE, &[FLOOR]);
    assert!(answer.unwrap().hit, "turned");
    let (answer, _) = run(&stationary([-1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [0.0; 3], IDENTITY), CUBE, &[FLOOR]);
    assert!(!answer.unwrap().hit, "unturned, the same ray runs along the floor and meets nothing");
}

#[test]
fn a_miss_keeps_the_words_the_intersection_helpers_wrote_and_a_segment_that_never_crosses_writes_no_point() {
    // crosses the plane of the triangle OUTSIDE it: a miss, but the helper had already written the crossing
    let pose = stationary([3.0, 1.0, 3.0], [3.0, -1.0, 3.0], [0.0; 3], IDENTITY);
    let (answer, asked) = run(&pose, CUBE, &[FLOOR]);
    let test = answer.unwrap();
    assert!(!test.hit);
    let at = point(&test);
    assert!(
        (at[0] - 3.0).abs() < 1e-4 && at[1].abs() < 1e-4 && (at[2] - 3.0).abs() < 1e-4,
        "the crossing is still written on a miss: {at:?}"
    );
    assert_eq!(asked, [0, 1], "a miss walks the cursor to its end");
    // entirely above the plane: no crossing, so the caller's point is returned untouched
    let pose = stationary([1.0, 2.0, 1.0], [1.0, 1.0, 1.0], [0.0; 3], IDENTITY);
    let test = run(&pose, CUBE, &[FLOOR]).0.unwrap();
    assert!(!test.hit);
    assert_eq!(test.words[..3], words(&POINT_IN)[..], "no crossing, no point written");
}

#[test]
fn the_triangles_are_read_in_order_and_the_first_one_that_is_hit_wins() {
    let far: [f32; 9] = [10.0, 0.0, 10.0, 14.0, 0.0, 10.0, 10.0, 0.0, 14.0];
    let pose = stationary([1.0, 1.0, 1.0], [1.0, -1.0, 1.0], [0.0; 3], IDENTITY);
    let (answer, asked) = run(&pose, CUBE, &[far, far, FLOOR, far]);
    assert!(answer.unwrap().hit);
    assert_eq!(asked, [0, 1, 2], "depths ascend one at a time and stop at the hit");
    let (answer, asked) = run(&pose, CUBE, &[far, far]);
    assert!(!answer.unwrap().hit);
    assert_eq!(asked, [0, 1, 2], "the cursor is read until it ends");
}

#[test]
fn a_model_without_collision_triangles_is_its_bounding_box_placed_by_the_pose() {
    let down = |x: f32| ([x, 5.0, 0.0], [x, -5.0, 0.0]);
    let (old, new) = down(10.0);
    let (answer, asked) = run(&stationary(old, new, [10.0, 0.0, 0.0], IDENTITY), CUBE, &[]);
    assert!(answer.unwrap().hit);
    assert_eq!(asked, [0], "an empty cursor is asked once and the box is used");
    let (old, new) = down(20.0);
    assert!(!run(&stationary(old, new, [10.0, 0.0, 0.0], IDENTITY), CUBE, &[]).0.unwrap().hit);
    // a long box along x is turned a quarter so that it lies along z: the ray at x = 3 crosses the first and not the second
    let long = [-5.0, 5.0, -0.5, 0.5, -0.5, 0.5];
    let (old, new) = down(3.0);
    assert!(run(&stationary(old, new, [0.0; 3], IDENTITY), long, &[]).0.unwrap().hit);
    let quarter = [0.0, 0.0, 1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0];
    assert!(!run(&stationary(old, new, [0.0; 3], quarter), long, &[]).0.unwrap().hit);
}

#[test]
fn a_moving_candidate_is_tested_at_the_sweep_positions_its_speed_asks_for() {
    // a body of length 2 sweeping from x = 0 to x = 10. Slow (<= a quarter of the length): the middle only. Medium: the two ends. Fast (> the length): both ends and the middle.
    let sweep = |x: f32, speed: f32| moving([x, 5.0, 0.0], [x, -5.0, 0.0], [0.0; 3], [10.0, 0.0, 0.0], IDENTITY, speed);
    let hit = |x: f32, speed: f32| run(&sweep(x, speed), CUBE, &[]).0.unwrap().hit;
    assert!(hit(5.0, 0.4), "slow: the middle position");
    assert!(!hit(5.0, 1.0), "medium: only the ends, so the middle is empty");
    assert!(hit(5.0, 3.0), "fast: the middle too");
    assert!(!hit(10.0, 0.4), "slow: the new end is not tested");
    assert!(hit(10.0, 1.0), "medium: the new end");
    assert!(hit(0.0, 1.0), "medium: the old end");
    assert!(hit(0.0, 3.0), "fast: the old end");
    assert!(!hit(0.0, 0.4), "slow: the old end is not tested");
    // the boundaries are strict: a speed equal to a quarter of the length is still slow, a speed equal to the length is still medium
    assert!(hit(5.0, 0.5), "equal to a quarter of the length: the middle only");
    assert!(!hit(5.0, 2.0), "equal to the length: the ends only");
    // triangles are swept the same way, and a sweep that hits ends the cursor
    let (answer, asked) = run(
        &moving([3.0, 1.0, 1.0], [3.0, -1.0, 1.0], [0.0; 3], [10.0, 0.0, 0.0], IDENTITY, 0.4),
        CUBE,
        &[FLOOR, FLOOR],
    );
    assert!(
        !answer.unwrap().hit,
        "slow: the floor covers x = 0 to 4 at the old position only, and x = 5 to 9 at the middle one"
    );
    assert_eq!(asked, [0, 1, 2]);
    let fast = moving([3.0, 1.0, 1.0], [3.0, -1.0, 1.0], [0.0; 3], [10.0, 0.0, 0.0], IDENTITY, 3.0);
    assert!(run(&fast, CUBE, &[FLOOR]).0.unwrap().hit, "fast: the old position is tested");
    let (answer, asked) = run(
        &moving([6.0, 1.0, 1.0], [6.0, -1.0, 1.0], [0.0; 3], [10.0, 0.0, 0.0], IDENTITY, 0.4),
        CUBE,
        &[FLOOR],
    );
    assert!(answer.unwrap().hit, "slow: the floor at the middle position covers x = 5 to 9");
    assert_eq!(asked, [0]);
}

#[test]
fn a_refusal_of_the_model_is_an_error_and_never_a_miss() {
    let pose = stationary([1.0, 1.0, 1.0], [1.0, -1.0, 1.0], [0.0; 3], IDENTITY);
    let refused = collide(&pose, words(&CUBE).try_into().unwrap(), &mut |depth| {
        if depth == 1 {
            Err("no such collision object".to_string())
        } else {
            Ok(Some([0; 9]))
        }
    });
    assert_eq!(refused.unwrap_err(), "no such collision object");
    let short = collide(&pose[..20], words(&CUBE).try_into().unwrap(), &mut |_| Ok(None));
    assert!(
        short.unwrap_err().contains("24 or 28"),
        "a pose that is neither a stationary nor a moving question is refused"
    );
}

#[test]
fn the_answer_does_not_depend_on_the_callers_rounding_and_the_callers_state_is_handed_back() {
    // an attitude whose entries and products are not exact, a long ray and a nearby crossing, so that the rounding direction reaches the last bits
    let (sin, cos) = 0.3_f32.sin_cos();
    let attitude = [cos, 0.0, sin, 0.0, 1.0, 0.0, -sin, 0.0, cos];
    let pose = stationary([0.3, 17.3, 0.7], [0.4, -9.1, 0.9], [0.1, 0.2, 0.3], attitude);
    let tilted: [f32; 9] = [-0.37, 0.11, -0.52, 4.7, 0.13, -0.29, 0.33, 0.07, 6.1];
    let before = fp_state().unwrap();
    let answers: Vec<_> = [Rounding::Zero, Rounding::Nearest, Rounding::Up, Rounding::Down]
        .into_iter()
        .map(|mode| {
            let _caller = ScopedRounding::set(mode).unwrap();
            let state = fp_state().unwrap();
            let answer = run(&pose, [-1.3, 1.1, -0.9, 1.7, -1.1, 0.8], &[tilted]).0.unwrap();
            assert_eq!(fp_state().unwrap(), state, "{mode:?}: the caller's complete state is handed back");
            assert_eq!(rounding().unwrap(), mode, "{mode:?}: and so is its rounding mode");
            answer
        })
        .collect();
    assert!(
        answers.iter().all(|answer| *answer == answers[0]),
        "the kernel runs under the original's rounding whatever the caller's is: {answers:?}"
    );
    assert_eq!(fp_state().unwrap(), before);
}

#[test]
fn the_intersection_helpers_are_the_same_text_as_the_tree_kernels() {
    // the entity kernel's copies of the original's two intersections must not drift from the copies the tree kernel's conformance is established on
    let read = |name: &str| {
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("csrc").join(name))
            .unwrap()
            .replace("\r\n", "\n")
    };
    let section = |text: &str| -> String {
        let start = text
            .find("static int get_3d_line_triangle_intersection")
            .expect("the line-triangle intersection");
        let end = text.find("/* en_cllsn.c: the two face wrappers").expect("the end of the intersections");
        text[start..end].to_string()
    };
    let (tree, entity) = (section(&read("tree_step.c")), section(&read("entity_step.c")));
    assert!(
        tree.contains("static int get_3d_line_quad_intersection"),
        "the section holds both intersections"
    );
    assert_eq!(tree.trim_end(), entity.trim_end());
}
