//! The host's per-candidate decisions of the entity collision scan (EEDCS-49, D3, third slice; src/entity_host.rs over csrc/entity_candidate_step.c).
//!
//! Two kinds of guard, neither an oracle for retail geometry:
//!
//! * focused contracts for each branch of the original's `get_line_of_sight_collision_entity` (`en_cllsn.c`): the tallest-structure skip, the mobile-then-fixed order, the moving test
//!   (`|motion|^2 > 1`) and the two services the host must ask for rather than assume (the destructive `UPDATED` getter and the terrain face normal), the broad-phase box and its
//!   strict boundaries, a city's height-then-xz box, a building listed directly in a sector, and the refusal of a facts record that is malformed;
//! * a DIFFERENTIAL of the whole scan: the original's control flow and arithmetic, transcribed from the source (sector order, tail-first lists, exclusions, the skip, the branches), run
//!   over many generated worlds against the same scan driven by the host's decisions exactly as the campaign's half drives them (`campaign_offer_candidate`, mb_hostoffer.h). A few entities
//!   have no usable scene (a direct city building and a city never have one): where the original reached a broad phase for one, both scans must stop with the same refusal, and where it did not,
//!   neither may need a model. Both record
//!   a transcript: every `UPDATED` read, every terrain normal question, every pose the narrow phase is asked about (bit for bit) and the entity returned. The two must be equal, so a
//!   decision, a pose, a service or an order that differs is a failure. The narrow phase itself is a hash of the pose here (it is the subject of `entity_collision_host.rs`), which makes
//!   any pose difference visible without a second model of the intersection.

// the transcription below keeps the original's `!(a < b)` forms on purpose: they are not `a >= b` when a value is not a number, and the original's behaviour is what is transcribed
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use eech_world::entity_host::{candidate, CandidateState, CANDIDATE_FACTS};
use eech_world::trees::{Rounding, ScopedRounding};
use std::cell::Cell;

// ---- the layout of the facts (mb_hostmodel.h) ----
const MOBILE: usize = 6;
const FIXED: usize = 7;
const CITY: usize = 8;
const CITY_BUILDING: usize = 9;
const OF_CITY: usize = 10;
const HAS_BUILDINGS: usize = 11;
const TALLEST: usize = 12;
const POSITION: usize = 13;
const MOTION: usize = 16;
const DELTA_TIME: usize = 19;
const UPDATED: usize = 20;
const HEADING: usize = 21;
const ALIGN: usize = 22;
const NORMAL_GIVEN: usize = 23;
const NORMAL: usize = 24;
const ATTITUDE: usize = 27;
const BOX_MIN: usize = 36;
const BOX_MAX: usize = 39;
const FIXED_GIVEN: usize = 42;
const PLACEMENT_GIVEN: usize = 43;

const SPACE: [f32; 6] = [-100.0, -100.0, -100.0, 100.0, 100.0, 100.0];
const IDENTITY: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

fn bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

fn blank() -> [f32; CANDIDATE_FACTS] {
    let mut facts = [0.0f32; CANDIDATE_FACTS];
    facts[..6].copy_from_slice(&SPACE);
    facts[DELTA_TIME] = 0.1;
    facts
}

/// the host's decision, and how many times it asked for the scene's radius (a candidate the original skipped must not ask at all)
fn decide_counting(facts: &[f32], radius: f32) -> (CandidateState, Vec<f32>, usize) {
    let asked = Cell::new(0);
    let step = candidate(&bits(facts), &mut || {
        asked.set(asked.get() + 1);
        Ok(radius.to_bits())
    })
    .expect("a well-formed question is answered");
    (step.state, step.pose.iter().map(|word| f32::from_bits(*word)).collect(), asked.get())
}

fn decide(facts: &[f32], radius: f32) -> (CandidateState, Vec<f32>) {
    let (state, pose, _) = decide_counting(facts, radius);
    (state, pose)
}

/// the decision of a candidate whose scene has no usable model: asking for the radius is a refusal
fn decide_without_model(facts: &[f32]) -> Result<CandidateState, String> {
    candidate(&bits(facts), &mut || Err("scene 8 is not loaded in the host's model store".to_string())).map(|step| step.state)
}

/// a stationary mobile candidate at `position` (shape, position, motion and the placement read) whose box of `radius + 10` m the segment SPACE may or may not meet
fn mobile_at(position: [f32; 3]) -> [f32; CANDIDATE_FACTS] {
    let mut facts = blank();
    facts[MOBILE] = 1.0;
    facts[POSITION..POSITION + 3].copy_from_slice(&position);
    facts[ATTITUDE..ATTITUDE + 9].copy_from_slice(&IDENTITY);
    facts[PLACEMENT_GIVEN] = 1.0;
    facts
}

/// a fixed candidate whose fixed facts and placement were read
fn fixed_at(position: [f32; 3]) -> [f32; CANDIDATE_FACTS] {
    let mut facts = blank();
    facts[FIXED] = 1.0;
    facts[FIXED_GIVEN] = 1.0;
    facts[PLACEMENT_GIVEN] = 1.0;
    facts[TALLEST] = 1000.0;
    facts[POSITION..POSITION + 3].copy_from_slice(&position);
    facts
}

#[test]
fn a_stationary_mobile_candidate_is_skipped_outside_its_box_and_tested_at_its_pose_inside_it_with_strict_boundaries() {
    // radius 5: the box is 15 m. The segment's x range is -100..100, so a candidate at x = 115 has its minimum at exactly 100: the strict comparison keeps it
    let (state, pose) = decide(&mobile_at([115.0, 0.0, 0.0]), 5.0);
    assert_eq!(state, CandidateState::Test);
    assert_eq!(pose.len(), 12);
    assert_eq!(pose[..3], [115.0, 0.0, 0.0], "a stationary candidate is tested at its position");
    assert_eq!(pose[3..], IDENTITY, "with its own attitude");
    let (state, pose) = decide(&mobile_at([115.01, 0.0, 0.0]), 5.0);
    assert_eq!(
        (state, pose.len()),
        (CandidateState::Skip, 0),
        "a box that does not reach the segment is skipped, with no pose"
    );
    let (state, _) = decide(&mobile_at([-115.0, 0.0, 0.0]), 5.0);
    assert_eq!(state, CandidateState::Test, "the other end of the axis is as strict");
    let (state, _) = decide(&mobile_at([0.0, 0.0, 115.01]), 5.0);
    assert_eq!(state, CandidateState::Skip, "every axis counts");
}

#[test]
fn a_mobile_candidate_is_moving_only_above_a_squared_speed_of_one_and_then_the_host_asks_for_the_updated_flag_before_anything_else() {
    let mut facts = mobile_at([0.0, 0.0, 0.0]);
    facts[MOTION] = 1.0;
    assert_eq!(decide(&facts, 5.0).0, CandidateState::Test, "a squared speed of exactly one is stationary");
    assert_eq!(decide(&facts, 5.0).1.len(), 12);
    facts[MOTION] = 1.0001;
    // moving: the destructive getter is needed first, and the model is not (the radius is only asked for after the sweep is known)
    let (state, _, radius_asked) = decide_counting(&facts, 5.0);
    assert_eq!((state, radius_asked), (CandidateState::NeedUpdated, 0));
    // even a candidate whose box is nowhere near the segment needs the answer: the original read the flag for every moving candidate it visited
    let mut far = mobile_at([5000.0, 0.0, 0.0]);
    far[MOTION] = 3.0;
    assert_eq!(decide(&far, 5.0).0, CandidateState::NeedUpdated);
    far[UPDATED] = 1.0;
    assert_eq!(decide(&far, 5.0).0, CandidateState::Skip, "once the flag is given the broad phase decides");
}

#[test]
fn a_moving_candidate_is_swept_from_the_position_by_the_flag_and_the_frame_time_and_the_pose_carries_the_speed() {
    let mut facts = mobile_at([10.0, 20.0, 30.0]);
    facts[MOTION..MOTION + 3].copy_from_slice(&[3.0, 0.0, 4.0]);
    facts[DELTA_TIME] = 0.5;
    facts[UPDATED] = 2.0;
    let (state, pose) = decide(&facts, 5.0);
    assert_eq!(state, CandidateState::Test);
    assert_eq!(pose.len(), 16);
    assert_eq!(
        pose[..3],
        [10.0 - 1.5, 20.0, 30.0 - 2.0],
        "an updated candidate is AT the new position and came from the old one"
    );
    assert_eq!(pose[3..6], [10.0, 20.0, 30.0]);
    assert_eq!(pose[15], 5.0, "the speed is the magnitude of the motion vector");
    facts[UPDATED] = 1.0;
    let (_, pose) = decide(&facts, 5.0);
    assert_eq!(
        pose[..3],
        [10.0, 20.0, 30.0],
        "a candidate that was not updated is at the old position and goes to the new one"
    );
    assert_eq!(pose[3..6], [11.5, 20.0, 32.0]);
}

#[test]
fn the_placement_is_asked_for_only_once_the_broad_phase_has_passed() {
    // a mobile candidate that meets the segment needs its attitude: the host asks, and does not ask for a candidate it skips
    let mut facts = mobile_at([0.0, 0.0, 0.0]);
    facts[PLACEMENT_GIVEN] = 0.0;
    assert_eq!(decide(&facts, 5.0).0, CandidateState::NeedPlacement);
    let mut far = mobile_at([5000.0, 0.0, 0.0]);
    far[PLACEMENT_GIVEN] = 0.0;
    assert_eq!(
        decide(&far, 5.0).0,
        CandidateState::Skip,
        "the placement of a candidate the broad phase removes is never read"
    );
    // the same for a fixed candidate and for a moving one
    let mut fixed = fixed_at([0.0, 0.0, 0.0]);
    fixed[PLACEMENT_GIVEN] = 0.0;
    assert_eq!(decide(&fixed, 5.0).0, CandidateState::NeedPlacement);
    fixed[POSITION] = 5000.0;
    assert_eq!(decide(&fixed, 5.0).0, CandidateState::Skip);
    let mut moving = mobile_at([0.0, 0.0, 0.0]);
    moving[MOTION] = 3.0;
    moving[UPDATED] = 2.0;
    moving[PLACEMENT_GIVEN] = 0.0;
    assert_eq!(decide(&moving, 5.0).0, CandidateState::NeedPlacement);
}

#[test]
fn a_candidate_that_is_not_mobile_is_skipped_in_a_sector_whose_tallest_structure_is_below_the_segment_before_its_fixed_flag_or_any_model_is_needed() {
    // the facts the original had read at that point: the segment, the mobile flag and the sector's tallest height. Nothing about a shape, a position or a model.
    let mut facts = blank();
    facts[TALLEST] = -100.5;
    let (state, pose, radius_asked) = decide_counting(&facts, 5.0);
    assert_eq!(
        (state, pose.len(), radius_asked),
        (CandidateState::Skip, 0, 0),
        "the segment's lowest point is above the tallest structure"
    );
    assert_eq!(decide_without_model(&facts), Ok(CandidateState::Skip), "and the skip needs no usable model");
    facts[TALLEST] = -100.0;
    let (state, _, radius_asked) = decide_counting(&facts, 5.0);
    assert_eq!(
        (state, radius_asked),
        (CandidateState::NeedFixed, 0),
        "equal is not above: the host asks for the fixed facts, and still needs no model"
    );
    let mut mobile = mobile_at([0.0, 0.0, 0.0]);
    mobile[TALLEST] = -500.0;
    assert_eq!(decide(&mobile, 5.0).0, CandidateState::Test, "the skip is not the mobile branch's");
}

#[test]
fn a_direct_city_building_and_an_unflagged_entity_are_skipped_without_a_scene_or_a_model_and_a_fixed_candidate_that_reaches_its_broad_phase_refuses_a_missing_model(
) {
    // a building listed directly in a sector: its fixed facts say so, and it has no shape; the host's skip needs no radius
    let mut listed = blank();
    listed[TALLEST] = 1000.0;
    listed[FIXED] = 1.0;
    listed[FIXED_GIVEN] = 1.0;
    listed[CITY_BUILDING] = 1.0;
    let (state, _, radius_asked) = decide_counting(&listed, 5.0);
    assert_eq!((state, radius_asked), (CandidateState::Skip, 0));
    assert_eq!(decide_without_model(&listed), Ok(CandidateState::Skip));
    // an entity that is neither mobile nor fixed: nothing
    let mut neither = blank();
    neither[TALLEST] = 1000.0;
    neither[FIXED_GIVEN] = 1.0;
    assert_eq!(decide_without_model(&neither), Ok(CandidateState::Skip));
    // a fixed candidate in a sector that does not skip it needs its model once it reaches the broad phase, and an unusable one is a refusal: never a skip, never a miss
    let reached = fixed_at([0.0, 0.0, 0.0]);
    let refused = decide_without_model(&reached).unwrap_err();
    assert!(refused.contains("not loaded"), "{refused}");
    // a city's own box needs no model either; its buildings are offered and each needs theirs
    let mut city = fixed_at([0.0, 0.0, 0.0]);
    city[CITY] = 1.0;
    city[HAS_BUILDINGS] = 1.0;
    city[BOX_MIN..BOX_MIN + 3].copy_from_slice(&[-10.0, -10.0, -10.0]);
    city[BOX_MAX..BOX_MAX + 3].copy_from_slice(&[10.0, 10.0, 10.0]);
    assert_eq!(decide_without_model(&city), Ok(CandidateState::Buildings));
    let mut building = fixed_at([0.0, 0.0, 0.0]);
    building[OF_CITY] = 1.0;
    assert!(
        decide_without_model(&building).is_err(),
        "a building that reaches its broad phase needs its model"
    );
}

#[test]
fn the_fixed_flag_is_asked_for_after_the_skip_and_a_fixed_candidate_is_placed_by_its_heading_or_asks_for_the_terrain_normal_once_the_broad_phase_has_passed() {
    let mut facts = fixed_at([0.0, 0.0, 0.0]);
    facts[HEADING] = 0.0;
    let (state, pose) = decide(&facts, 5.0);
    assert_eq!(state, CandidateState::Test);
    assert_eq!(pose.len(), 12);
    assert_eq!(pose[3..], IDENTITY, "a zero heading is the identity");
    let mut unread = facts;
    unread[FIXED_GIVEN] = 0.0;
    unread[FIXED] = 0.0;
    assert_eq!(decide(&unread, 5.0).0, CandidateState::NeedFixed, "the fixed flag is asked for, not assumed");
    facts[ALIGN] = 1.0;
    assert_eq!(decide(&facts, 5.0).0, CandidateState::NeedNormal, "an aligned object needs the terrain");
    let mut outside = fixed_at([500.0, 0.0, 0.0]);
    outside[ALIGN] = 1.0;
    assert_eq!(
        decide(&outside, 5.0).0,
        CandidateState::Skip,
        "the terrain is asked only when the broad phase passed"
    );
    facts[NORMAL_GIVEN] = 1.0;
    facts[NORMAL..NORMAL + 3].copy_from_slice(&[0.0, 1.0, 0.0]);
    let (state, pose) = decide(&facts, 5.0);
    assert_eq!(state, CandidateState::Test);
    // flat ground and a zero heading give the identity placement, with the normal as the second row
    assert_eq!(pose[3..], IDENTITY);
    facts[NORMAL..NORMAL + 3].copy_from_slice(&[0.6, 0.8, 0.0]);
    let tilted = decide(&facts, 5.0).1;
    assert_eq!(tilted[6..9], [0.6, 0.8, 0.0], "a tilted ground's normal is the second row of the placement");
    // a normal that is almost horizontal gives the identity, as the original does
    facts[NORMAL..NORMAL + 3].copy_from_slice(&[1.0, 0.00005, 0.0]);
    assert_eq!(decide(&facts, 5.0).1[3..], IDENTITY);
}

#[test]
fn a_city_is_tested_by_height_then_by_the_xz_box_and_then_its_buildings_are_offered_and_a_building_listed_in_a_sector_is_skipped() {
    let mut city = fixed_at([0.0, 0.0, 0.0]);
    city[CITY] = 1.0;
    city[HAS_BUILDINGS] = 1.0;
    city[BOX_MIN..BOX_MIN + 3].copy_from_slice(&[-10.0, -10.0, -10.0]);
    city[BOX_MAX..BOX_MAX + 3].copy_from_slice(&[10.0, 10.0, 10.0]);
    assert_eq!(decide(&city, 0.0).0, CandidateState::Buildings);
    city[HAS_BUILDINGS] = 0.0;
    assert_eq!(decide(&city, 0.0).0, CandidateState::Skip, "a city with no building is nothing");
    city[HAS_BUILDINGS] = 1.0;
    city[POSITION + 1] = 300.0;
    assert_eq!(
        decide(&city, 0.0).0,
        CandidateState::Skip,
        "the height is tested first: a box above the segment's range"
    );
    city[POSITION + 1] = 0.0;
    city[POSITION] = 111.0;
    assert_eq!(decide(&city, 0.0).0, CandidateState::Skip, "then the box in x");
    city[POSITION] = 110.0;
    assert_eq!(decide(&city, 0.0).0, CandidateState::Buildings, "equal is a meeting");
    city[POSITION] = 0.0;
    city[TALLEST] = -500.0;
    assert_eq!(decide(&city, 0.0).0, CandidateState::Skip, "the sector skip applies to the city");
    // a building listed under a city that passed: its own broad phase, and no sector skip again
    let mut building = fixed_at([0.0, 0.0, 0.0]);
    building[OF_CITY] = 1.0;
    building[TALLEST] = -500.0;
    assert_eq!(decide(&building, 5.0).0, CandidateState::Test);
    building[POSITION] = 500.0;
    assert_eq!(decide(&building, 5.0).0, CandidateState::Skip);
    // listed directly in a sector, a building is skipped
    let mut listed = fixed_at([0.0, 0.0, 0.0]);
    listed[CITY_BUILDING] = 1.0;
    assert_eq!(decide(&listed, 5.0).0, CandidateState::Skip);
}

#[test]
fn a_building_that_is_neither_mobile_nor_fixed_is_the_originals_invalid_target_and_malformed_facts_are_refused_never_defaulted() {
    let mut unflagged = blank();
    unflagged[OF_CITY] = 1.0;
    // its placement is asked for first (the original read the flags in its narrow phase); once read, an unflagged building is the original's fatal
    assert_eq!(decide(&unflagged, 5.0).0, CandidateState::NeedPlacement);
    unflagged[PLACEMENT_GIVEN] = 1.0;
    let refused = candidate(&bits(&unflagged), &mut || Ok(5.0f32.to_bits())).unwrap_err();
    assert!(refused.contains("Invalid target"), "{refused}");
    let radius = || Ok(5.0f32.to_bits());
    for (label, at, value) in [
        ("a flag of 2", MOBILE, 2.0),
        ("a negative flag", FIXED, -1.0),
        ("the fixed-given flag 2", FIXED_GIVEN, 2.0),
        ("the placement-given flag 2", PLACEMENT_GIVEN, 2.0),
        ("the updated state 3", UPDATED, 3.0),
        ("a non-finite fact", POSITION, f32::NAN),
        ("an infinite fact", MOTION, f32::INFINITY),
    ] {
        let mut facts = mobile_at([0.0, 0.0, 0.0]);
        facts[at] = value;
        assert!(candidate(&bits(&facts), &mut || radius()).is_err(), "{label} must be refused");
    }
    let mut both = mobile_at([0.0, 0.0, 0.0]);
    both[CITY] = 1.0;
    both[CITY_BUILDING] = 1.0;
    assert!(candidate(&bits(&both), &mut || radius()).is_err(), "a city that is also a city building");
    assert!(candidate(&[0u32; 43], &mut || radius()).is_err(), "43 facts");
    assert!(candidate(&[0u32; 42], &mut || radius()).is_err(), "the 42 facts of the previous layout");
    assert!(
        candidate(&bits(&mobile_at([0.0; 3])), &mut || Ok(f32::NAN.to_bits())).is_err(),
        "a radius that is not a number"
    );
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the differential
// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Entity {
    mobile: bool,
    fixed: bool,
    city: bool,
    city_building: bool,
    scene: i64,
    position: [f32; 3],
    motion: [f32; 3],
    updated: bool,
    heading: f32,
    align: bool,
    attitude: [f32; 9],
    box_min: [f32; 3],
    box_max: [f32; 3],
    buildings: Vec<Entity>,
}

struct Sector {
    tallest: f32,
    /// head to tail, as the campaign's list holds them; the scan reads it from the tail
    list: Vec<Entity>,
}

#[derive(Clone, Debug, PartialEq)]
enum Event {
    Updated(usize, usize),
    Normal(usize, usize, [u32; 2]),
    Narrow(usize, usize, Vec<u32>),
    /// the scan stopped: a branch needed the radius of a scene that has no usable model (the original asserted; the host refuses by name)
    Refused(usize, usize),
}

/// what both scans return when the engine would stop
const STOPPED: (usize, usize, Option<usize>) = (usize::MAX, 0, None);

const DT: f32 = 0.0625;

/// the radius of the scene's model, or none when the scene has no usable model (scene 0, the invalid index)
fn radius_of(scene: i64) -> Option<f32> {
    (scene >= 1).then_some(2.0 + (scene as f32) * 3.25)
}

fn terrain_normal(x: f32, z: f32) -> [f32; 3] {
    let nx = (x * 0.0137).sin() * 0.4;
    let nz = (z * 0.0091).cos() * 0.4;
    let length = (nx * nx + 1.0 + nz * nz).sqrt();
    [nx / length, 1.0 / length, nz / length]
}

/// the narrow phase stand-in: a hash of the scene and every word of the pose, so any difference in a pose is a difference in the answer and in the transcript
fn narrow(scene: i64, pose: &[u32]) -> bool {
    let mut hash = 0x9e37_79b9u32 ^ scene as u32;
    for word in pose {
        hash = (hash ^ word).wrapping_mul(0x0100_0193).rotate_left(5);
    }
    hash.is_multiple_of(3)
}

// the original's arithmetic for the placement (modules/maths/matrix.c and vector.c), in single precision with the narrowing steps it has. These helpers are the Windows x64 original's evaluation, where every
// float operation rounds to float (this test binary does the same on i686, which has SSE2). The hosted kernel gets the same words on x87 targets by narrowing each float operation explicitly (NARROW in
// csrc/entity_candidate_step.c), because x87 evaluates whole expressions, and after inlining the variables between statements, in extended precision: on i686 the original text itself is not one answer.
fn heading_matrix(heading: f32) -> [f32; 9] {
    let (sinh, cosh) = ((heading as f64).sin() as f32, (heading as f64).cos() as f32);
    let (sinp, cosp, sinr, cosr) = (0.0f32, 1.0f32, 0.0f32, 1.0f32);
    let (sinrsinh, sinrcosh, cosrsinh, cosrcosh) = (sinr * sinh, sinr * cosh, cosr * sinh, cosr * cosh);
    [
        cosrcosh + (sinrsinh * sinp),
        -sinr * cosp,
        (sinrcosh * sinp) - cosrsinh,
        sinrcosh - (cosrsinh * sinp),
        cosr * cosp,
        -sinrsinh - (cosrcosh * sinp),
        cosp * sinh,
        sinp,
        cosp * cosh,
    ]
}

fn normalise(v: [f32; 3]) -> [f32; 3] {
    let (x, y, z) = (v[0] * v[0], v[1] * v[1], v[2] * v[2]);
    let length = ((x + y + z) as f64).sqrt() as f32;
    if length > 0.0 {
        let one_over_length = (1.0f64 / f64::from(length)) as f32;
        [v[0] * one_over_length, v[1] * one_over_length, v[2] * one_over_length]
    } else {
        [0.0; 3]
    }
}

fn face_normal_matrix(normal: [f32; 3], heading: f32) -> [f32; 9] {
    if f64::from(normal[1]).abs() > 0.0001 {
        let (r20, r22) = ((heading as f64).sin() as f32, (heading as f64).cos() as f32);
        let r21 = -((normal[0] * r20) + (normal[2] * r22)) / normal[1];
        let row2 = normalise([r20, r21, r22]);
        let row0 = [
            (normal[1] * row2[2]) - (row2[1] * normal[2]),
            (normal[2] * row2[0]) - (row2[2] * normal[0]),
            (normal[0] * row2[1]) - (row2[0] * normal[1]),
        ];
        [row0[0], row0[1], row0[2], normal[0], normal[1], normal[2], row2[0], row2[1], row2[2]]
    } else {
        [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
    }
}

fn magnitude(v: [f32; 3]) -> f32 {
    let (x, y, z) = (v[0] * v[0], v[1] * v[1], v[2] * v[2]);
    ((x + y + z) as f64).sqrt() as f32
}

fn meets(tmin: [f32; 3], tmax: [f32; 3], smin: [f32; 3], smax: [f32; 3]) -> bool {
    !(tmax[0] < smin[0]) && !(tmin[0] > smax[0]) && !(tmax[1] < smin[1]) && !(tmin[1] > smax[1]) && !(tmax[2] < smin[2]) && !(tmin[2] > smax[2])
}

fn min_max(old: [f32; 3], new: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let mut min = [0.0; 3];
    let mut max = [0.0; 3];
    for axis in 0..3 {
        if old[axis] <= new[axis] {
            min[axis] = old[axis];
            max[axis] = new[axis];
        } else {
            min[axis] = new[axis];
            max[axis] = old[axis];
        }
    }
    (min, max)
}

fn around(position: [f32; 3], radius: f32) -> ([f32; 3], [f32; 3]) {
    (
        [position[0] - radius - 10.0, position[1] - radius - 10.0, position[2] - radius - 10.0],
        [position[0] + radius + 10.0, position[1] + radius + 10.0, position[2] + radius + 10.0],
    )
}

/// the sectors in the order the scan visits them (each with its index in the map's grid), the segment, and the entry that is the source or the target as (sector index, list index)
type Scan<'a> = (&'a [(usize, &'a Sector)], [f32; 3], [f32; 3], (usize, usize));

/// the ORIGINAL scan (en_cllsn.c, get_line_of_sight_collision_entity and the two accurate_* placements), transcribed: returns the entity that was hit as (sector, list index[, building])
fn original_scan(scan: Scan, log: &mut Vec<Event>) -> Option<(usize, usize, Option<usize>)> {
    let (sectors, old, new, excluded) = scan;
    let (smin, smax) = min_max(old, new);
    for &(s, sector) in sectors.iter() {
        let ignore_all_fixed_entities_in_sector = smin[1] > sector.tallest;
        for (l, ent) in sector.list.iter().enumerate().rev() {
            if (s, l) == excluded {
                continue;
            }
            if ent.mobile {
                let m = ent.motion;
                let sqr_velocity = m[0] * m[0] + m[1] * m[1] + m[2] * m[2];
                if sqr_velocity > 1.0 {
                    log.push(Event::Updated(s, l));
                    let (told, tnew) = if ent.updated {
                        let tnew = ent.position;
                        ([tnew[0] - m[0] * DT, tnew[1] - m[1] * DT, tnew[2] - m[2] * DT], tnew)
                    } else {
                        let told = ent.position;
                        (told, [told[0] + m[0] * DT, told[1] + m[1] * DT, told[2] + m[2] * DT])
                    };
                    let (mut tmin, mut tmax) = min_max(told, tnew);
                    let Some(radius) = radius_of(ent.scene) else {
                        log.push(Event::Refused(s, l * 1000));
                        return Some(STOPPED);
                    };
                    for axis in 0..3 {
                        tmin[axis] -= radius + 10.0;
                        tmax[axis] += radius + 10.0;
                    }
                    if meets(tmin, tmax, smin, smax) {
                        let pose = [told.as_slice(), tnew.as_slice(), ent.attitude.as_slice(), &[magnitude(m)]].concat();
                        let words = bits(&[old.as_slice(), new.as_slice(), pose.as_slice()].concat());
                        log.push(Event::Narrow(s, l * 1000, words.clone()));
                        if narrow(ent.scene, &words) {
                            return Some((s, l, None));
                        }
                    }
                } else {
                    let Some(radius) = radius_of(ent.scene) else {
                        log.push(Event::Refused(s, l * 1000));
                        return Some(STOPPED);
                    };
                    let (tmin, tmax) = around(ent.position, radius);
                    if meets(tmin, tmax, smin, smax) {
                        let pose = [ent.position.as_slice(), ent.attitude.as_slice()].concat();
                        let words = bits(&[old.as_slice(), new.as_slice(), pose.as_slice()].concat());
                        log.push(Event::Narrow(s, l * 1000, words.clone()));
                        if narrow(ent.scene, &words) {
                            return Some((s, l, None));
                        }
                    }
                }
            } else if !ignore_all_fixed_entities_in_sector && ent.fixed {
                if ent.city {
                    if !ent.buildings.is_empty() {
                        let (mut tmin, mut tmax) = (ent.box_min, ent.box_max);
                        tmin[1] += ent.position[1];
                        tmax[1] += ent.position[1];
                        if !(tmax[1] < smin[1]) && !(tmin[1] > smax[1]) {
                            tmin[0] += ent.position[0];
                            tmin[2] += ent.position[2];
                            tmax[0] += ent.position[0];
                            tmax[2] += ent.position[2];
                            if !(tmax[0] < smin[0]) && !(tmin[0] > smax[0]) && !(tmax[2] < smin[2]) && !(tmin[2] > smax[2]) {
                                for (b, building) in ent.buildings.iter().enumerate() {
                                    let Some(radius) = radius_of(building.scene) else {
                                        log.push(Event::Refused(s, l * 1000 + b + 1));
                                        return Some(STOPPED);
                                    };
                                    let (bmin, bmax) = around(building.position, radius);
                                    if meets(bmin, bmax, smin, smax) {
                                        let words = original_stationary(building, old, new, (s, l, Some(b)), log);
                                        if narrow(building.scene, &words) {
                                            return Some((s, l, Some(b)));
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if !ent.city_building {
                    let Some(radius) = radius_of(ent.scene) else {
                        log.push(Event::Refused(s, l * 1000));
                        return Some(STOPPED);
                    };
                    let (tmin, tmax) = around(ent.position, radius);
                    if meets(tmin, tmax, smin, smax) {
                        let words = original_stationary(ent, old, new, (s, l, None), log);
                        if narrow(ent.scene, &words) {
                            return Some((s, l, None));
                        }
                    }
                }
            }
        }
    }
    None
}

/// accurate_stationary_bounding_box_collision_test up to the narrow phase: the placement, with the terrain question where the original asked it
fn original_stationary(ent: &Entity, old: [f32; 3], new: [f32; 3], at: (usize, usize, Option<usize>), log: &mut Vec<Event>) -> Vec<u32> {
    let key = at.1 * 1000 + at.2.map_or(0, |b| b + 1);
    let attitude = if ent.mobile {
        ent.attitude
    } else if ent.fixed {
        if ent.align {
            let normal = terrain_normal(ent.position[0], ent.position[2]);
            log.push(Event::Normal(at.0, key, [ent.position[0].to_bits(), ent.position[2].to_bits()]));
            face_normal_matrix(normal, ent.heading)
        } else {
            heading_matrix(ent.heading)
        }
    } else {
        panic!("the original stops: Invalid target");
    };
    let pose = [ent.position.as_slice(), attitude.as_slice()].concat();
    let words = bits(&[old.as_slice(), new.as_slice(), pose.as_slice()].concat());
    log.push(Event::Narrow(at.0, key, words.clone()));
    words
}

/// the campaign's half exactly as `campaign_offer_candidate` (mb_hostoffer.h) drives the host: what it reads and when (the mobile flag first; a mobile candidate's shape, position and motion; the fixed
/// facts only when the host asks; the placement only after the broad phase), the two services performed when the host names them, the buildings in order. A refusal stops the scan.
fn offer(
    ent: &Entity,
    of_city: bool,
    tallest: f32,
    old: [f32; 3],
    new: [f32; 3],
    at: (usize, usize, Option<usize>),
    log: &mut Vec<Event>,
) -> Option<(usize, usize, Option<usize>)> {
    let key = at.1 * 1000 + at.2.map_or(0, |b| b + 1);
    let mut facts = [0.0f32; CANDIDATE_FACTS];
    facts[..3].copy_from_slice(&old);
    facts[3..6].copy_from_slice(&new);
    facts[TALLEST] = tallest;
    facts[DELTA_TIME] = DT;
    let mut scene = 0;
    if of_city {
        facts[OF_CITY] = 1.0;
        scene = ent.scene;
        facts[POSITION..POSITION + 3].copy_from_slice(&ent.position);
    } else {
        facts[MOBILE] = f32::from(u8::from(ent.mobile));
        if ent.mobile {
            scene = ent.scene;
            facts[POSITION..POSITION + 3].copy_from_slice(&ent.position);
            facts[MOTION..MOTION + 3].copy_from_slice(&ent.motion);
        }
    }
    loop {
        let decided = candidate(&bits(&facts), &mut || {
            radius_of(scene).map(f32::to_bits).ok_or_else(|| format!("scene {scene} has no usable model"))
        });
        let step = match decided {
            Ok(step) => step,
            Err(why) => {
                assert!(why.contains("has no usable model"), "only the missing model is a refusal here: {why}");
                log.push(Event::Refused(at.0, key));
                return Some(STOPPED);
            }
        };
        match step.state {
            CandidateState::Skip => return None,
            CandidateState::NeedUpdated => {
                log.push(Event::Updated(at.0, at.1));
                facts[UPDATED] = if ent.updated { 2.0 } else { 1.0 };
            }
            CandidateState::NeedFixed => {
                let fixed = ent.fixed;
                facts[FIXED] = f32::from(u8::from(fixed));
                facts[FIXED_GIVEN] = 1.0;
                if fixed {
                    if ent.city {
                        facts[CITY] = 1.0;
                        facts[HAS_BUILDINGS] = f32::from(u8::from(!ent.buildings.is_empty()));
                        if !ent.buildings.is_empty() {
                            facts[POSITION..POSITION + 3].copy_from_slice(&ent.position);
                            facts[BOX_MIN..BOX_MIN + 3].copy_from_slice(&ent.box_min);
                            facts[BOX_MAX..BOX_MAX + 3].copy_from_slice(&ent.box_max);
                        }
                    } else if ent.city_building {
                        facts[CITY_BUILDING] = 1.0;
                    } else {
                        scene = ent.scene;
                        facts[POSITION..POSITION + 3].copy_from_slice(&ent.position);
                    }
                }
            }
            CandidateState::NeedPlacement => {
                if of_city {
                    facts[MOBILE] = f32::from(u8::from(ent.mobile));
                    facts[FIXED] = f32::from(u8::from(!ent.mobile && ent.fixed));
                }
                if facts[MOBILE] != 0.0 {
                    facts[ATTITUDE..ATTITUDE + 9].copy_from_slice(&ent.attitude);
                } else if facts[FIXED] != 0.0 {
                    facts[HEADING] = ent.heading;
                    facts[ALIGN] = f32::from(u8::from(ent.align));
                }
                facts[PLACEMENT_GIVEN] = 1.0;
            }
            CandidateState::NeedNormal => {
                let normal = terrain_normal(facts[POSITION], facts[POSITION + 2]);
                log.push(Event::Normal(at.0, key, [facts[POSITION].to_bits(), facts[POSITION + 2].to_bits()]));
                facts[NORMAL_GIVEN] = 1.0;
                facts[NORMAL..NORMAL + 3].copy_from_slice(&normal);
            }
            CandidateState::Buildings => {
                for (b, building) in ent.buildings.iter().enumerate() {
                    if let Some(hit) = offer(building, true, tallest, old, new, (at.0, at.1, Some(b)), log) {
                        return Some(hit);
                    }
                }
                return None;
            }
            CandidateState::Test => {
                let words = [bits(&old).as_slice(), bits(&new).as_slice(), step.pose.as_slice()].concat();
                log.push(Event::Narrow(at.0, key, words.clone()));
                return narrow(scene, &words).then_some(at);
            }
        }
    }
}

const SIDE: i32 = 100;
const GRID: usize = 4;
const MAP_MAX: f32 = 399.0;

/// the ORIGINAL's window (en_cllsn.c: the segment's min and max, widened by LARGEST_3D_OBJECT_RADIUS, clamped to the map, as sector indices), transcribed
fn original_window(old: [f32; 3], new: [f32; 3]) -> (i32, i32, i32, i32) {
    let (smin, smax) = min_max(old, new);
    let high = |a: f32, b: f32| if a > b { a } else { b };
    let low = |a: f32, b: f32| if a < b { a } else { b };
    let sector = |value: f32| (value as i32) / SIDE;
    (
        sector(high(smin[0] - 150.0, 0.0)),
        sector(high(smin[2] - 150.0, 0.0)),
        sector(low(smax[0] + 150.0, MAP_MAX)),
        sector(low(smax[2] + 150.0, MAP_MAX)),
    )
}

/// the entity that was hit: (sector index, list index, the building under a city)
type Hit = (usize, usize, Option<usize>);

/// the ORIGINAL scan over the grid: x outer, z inner, each sector's list from its tail
fn original_grid_scan(grid: &[Sector], old: [f32; 3], new: [f32; 3], excluded: (usize, usize), log: &mut Vec<Event>) -> Option<(usize, usize, Option<usize>)> {
    let (x_min, z_min, x_max, z_max) = original_window(old, new);
    let mut visited = Vec::new();
    for x in x_min..=x_max {
        for z in z_min..=z_max {
            let index = x as usize * GRID + z as usize;
            visited.push((index, &grid[index]));
        }
    }
    original_scan((&visited[..], old, new, excluded), log)
}

/// the campaign's driver exactly as `campaign_scan_entities` (mb_hostscan.h) drives the host's cursor over the campaign's lists: it serves the tail of the sector the host names, the predecessor of
/// its entry, the fact that the entry is the source or the target, the offer (`campaign_offer_candidate`) and its result; the host decides everything else. A refusal stops the scan.
fn hosted_grid_scan(grid: &[Sector], old: [f32; 3], new: [f32; 3], excluded: (usize, usize), log: &mut Vec<Event>) -> Option<(usize, usize, Option<usize>)> {
    use eech_world::entity_host::{scan_step, ScanState};
    let mut facts = [0.0f32; 16];
    facts[..3].copy_from_slice(&old);
    facts[3..6].copy_from_slice(&new);
    facts[6] = SIDE as f32;
    facts[9] = MAP_MAX;
    facts[10] = MAP_MAX;
    let (mut sector, mut entry, mut hit): (usize, Option<usize>, Option<Hit>) = (0, None, None);
    for _ in 0..100_000 {
        let step = scan_step(&bits(&facts)).expect("the host decides the enumeration");
        match step.state {
            ScanState::First => {
                let (x, z) = (f32::from_bits(step.cursor[0]) as usize, f32::from_bits(step.cursor[1]) as usize);
                facts[12] = x as f32;
                facts[13] = z as f32;
                sector = x * GRID + z;
                entry = grid[sector].list.len().checked_sub(1);
            }
            ScanState::Next => entry = entry.and_then(|l| l.checked_sub(1)),
            ScanState::Offer => {
                let l = entry.expect("an entry is offered only when one is served");
                hit = offer(&grid[sector].list[l], false, grid[sector].tallest, old, new, (sector, l, None), log);
                facts[15] = f32::from(u8::from(hit.is_some()));
                facts[11] = 3.0;
                continue;
            }
            ScanState::DoneHit => return hit,
            ScanState::DoneNone => return None,
        }
        facts[15] = 0.0;
        facts[11] = if entry.is_some() { 1.0 } else { 2.0 };
        facts[14] = f32::from(u8::from(entry.is_some_and(|l| (sector, l) == excluded)));
    }
    panic!("the host did not end the scan");
}

struct Random(u64);

impl Random {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }

    /// a multiple of a quarter, so that equalities at the boundaries of the boxes happen
    fn quarter(&mut self, range: i32) -> f32 {
        ((self.next() % (4 * range as u32 * 2 + 1)) as i32 - 4 * range) as f32 * 0.25
    }

    fn pick(&mut self, n: u32) -> u32 {
        self.next() % n
    }
}

fn entity(r: &mut Random, kind: u32, extent: i32) -> Entity {
    let mut e = Entity {
        mobile: false,
        fixed: false,
        city: false,
        city_building: false,
        // a few entities have no usable scene: the original needed one only where it reached a broad phase
        scene: if r.pick(25) == 0 { 0 } else { 1 + i64::from(r.pick(9)) },
        position: [200.0 + r.quarter(extent), r.quarter(extent / 4), 200.0 + r.quarter(extent)],
        motion: [0.0; 3],
        updated: r.pick(2) == 0,
        heading: r.quarter(12) * 0.5,
        align: r.pick(2) == 0,
        attitude: heading_matrix(r.quarter(12) * 0.25),
        box_min: [-r.quarter(30).abs() - 1.0, -r.quarter(10).abs() - 1.0, -r.quarter(30).abs() - 1.0],
        box_max: [r.quarter(30).abs() + 1.0, r.quarter(10).abs() + 1.0, r.quarter(30).abs() + 1.0],
        buildings: Vec::new(),
    };
    match kind {
        0..=3 => {
            e.mobile = true;
            // squared speeds around one: 0, exactly one, just above, and moving
            e.motion = match r.pick(5) {
                0 => [0.0; 3],
                1 => [1.0, 0.0, 0.0],
                2 => [0.0, 1.0000001, 0.0],
                _ => [r.quarter(8), r.quarter(2), r.quarter(8)],
            };
        }
        4..=7 => e.fixed = true,
        8 => {
            e.fixed = true;
            e.city = true;
            // a city has no shape of its own
            e.scene = 0;
            for _ in 0..r.pick(5) {
                let mut building = entity(r, 4, extent);
                building.position = [e.position[0] + r.quarter(30), e.position[1], e.position[2] + r.quarter(30)];
                e.buildings.push(building);
            }
        }
        9 => {
            e.fixed = true;
            e.city_building = true;
            // a building listed directly in a sector has no shape: the original skipped it without a shape lookup
            e.scene = 0;
        }
        _ => {}
    }
    e
}

#[test]
fn the_host_driven_enumeration_is_the_original_scan_in_window_order_exclusions_first_hit_services_poses_and_refusals_over_many_worlds() {
    let _original_arithmetic = ScopedRounding::set(Rounding::Zero).expect("round toward zero");
    let mut random = Random(0x5eed_1234_abcd_ef01);
    let (mut hits, mut narrows, mut updates, mut normals, mut cities, mut moving, mut refusals, mut excluded_seen, mut windows) =
        (0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    for world in 0..400 {
        // a 4 x 4 map of sectors of 100: every sector has a list (head to tail, as the campaign's list holds them) and its tallest structure
        let grid: Vec<Sector> = (0..GRID * GRID)
            .map(|_| Sector {
                tallest: random.quarter(30),
                list: (0..random.pick(6))
                    .map(|_| {
                        let kind = random.pick(11);
                        entity(&mut random, kind, 190)
                    })
                    .collect(),
            })
            .collect();
        for _ in 0..6 {
            // segments of every size inside the map: the window is the box widened by 150 m and clamped, so small and large windows, and the map's edges, are all reached
            let at = |r: &mut Random| [200.0 + r.quarter(200), r.quarter(25), 200.0 + r.quarter(200)];
            let old = at(&mut random);
            let new = if random.pick(3) == 0 {
                [old[0] + random.quarter(30), random.quarter(25), old[2] + random.quarter(30)]
            } else {
                at(&mut random)
            };
            let new = [new[0].clamp(0.0, 399.0), new[1], new[2].clamp(0.0, 399.0)];
            let source_sector = random.pick((GRID * GRID) as u32) as usize;
            let excluded = (source_sector, random.pick(6) as usize);
            let (mut a, mut b) = (Vec::new(), Vec::new());
            let original = original_grid_scan(&grid, old, new, excluded, &mut a);
            let hosted = hosted_grid_scan(&grid, old, new, excluded, &mut b);
            assert_eq!(hosted, original, "world {world}: the entity that is hit");
            assert_eq!(b, a, "world {world}: the services, the poses and their order");
            let (x_min, z_min, x_max, z_max) = original_window(old, new);
            windows += ((x_max - x_min + 1) * (z_max - z_min + 1)) as usize;
            excluded_seen += usize::from(excluded.1 < grid[excluded.0].list.len());
            hits += usize::from(original.is_some());
            for event in &a {
                match event {
                    Event::Updated(..) => updates += 1,
                    Event::Refused(..) => refusals += 1,
                    Event::Normal(..) => normals += 1,
                    Event::Narrow(_, key, words) => {
                        narrows += 1;
                        moving += usize::from(words.len() == 22);
                        cities += usize::from(*key % 1000 != 0);
                    }
                }
            }
        }
    }
    // the generated worlds reach every branch, or the equality above would say little
    assert!(hits > 100, "hits {hits}");
    assert!(narrows > 1000, "narrow phases {narrows}");
    assert!(updates > 100, "UPDATED reads {updates}");
    assert!(normals > 50, "terrain questions {normals}");
    assert!(moving > 50, "moving candidates {moving}");
    assert!(cities > 10, "city buildings {cities}");
    assert!(refusals > 5, "scans that stopped on a scene with no model {refusals}");
    assert!(excluded_seen > 300, "exclusions that named a real entry {excluded_seen}");
    assert!(windows > 2000, "sectors visited {windows}");
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// the enumeration kernel's own contract (csrc/entity_scan_step.c): the window, the traversal order, the phases and the refusals

fn scan_facts(old: [f32; 3], new: [f32; 3], phase: f32, cursor: [f32; 2]) -> [f32; 16] {
    let mut facts = [0.0f32; 16];
    facts[..3].copy_from_slice(&old);
    facts[3..6].copy_from_slice(&new);
    facts[6] = 100.0;
    facts[9] = 399.0;
    facts[10] = 399.0;
    facts[11] = phase;
    facts[12] = cursor[0];
    facts[13] = cursor[1];
    facts
}

fn scan(facts: &[f32; 16]) -> (eech_world::entity_host::ScanState, [f32; 2]) {
    let step = eech_world::entity_host::scan_step(&bits(facts)).expect("a well-formed question is answered");
    (step.state, step.cursor.map(f32::from_bits))
}

#[test]
fn the_window_is_the_segments_box_widened_by_150_and_clamped_to_the_map_and_the_traversal_is_x_outer_z_inner() {
    use eech_world::entity_host::ScanState::{DoneNone, First};
    // a segment from (250, 0, 250) to (260, 0, 250): the box x 250..260, z 250..250 widened to x 100..410, z 100..400, clamped to the map's 399: sectors x 1..3, z 1..3
    let (old, new) = ([250.0, 0.0, 250.0], [260.0, 0.0, 250.0]);
    assert_eq!(
        scan(&scan_facts(old, new, 0.0, [0.0, 0.0])),
        (First, [1.0, 1.0]),
        "the scan starts at the window's minimum"
    );
    // the end of a list goes to the next z, then to the next x at the window's minimum z
    assert_eq!(scan(&scan_facts(old, new, 2.0, [1.0, 1.0])), (First, [1.0, 2.0]));
    assert_eq!(scan(&scan_facts(old, new, 2.0, [1.0, 3.0])), (First, [2.0, 1.0]), "z inner, x outer");
    assert_eq!(
        scan(&scan_facts(old, new, 2.0, [3.0, 3.0])).0,
        DoneNone,
        "the last sector of the window ends the scan"
    );
    // a window at the map's origin is clamped at 0, not at a negative sector
    assert_eq!(scan(&scan_facts([0.0; 3], [10.0, 0.0, 10.0], 0.0, [0.0, 0.0])), (First, [0.0, 0.0]));
    assert_eq!(
        scan(&scan_facts([0.0; 3], [10.0, 0.0, 10.0], 2.0, [0.0, 1.0])),
        (First, [1.0, 0.0]),
        "a window of sectors 0..1 in each axis goes on from the end of z to the next x"
    );
    // the window does not depend on the order of the segment's ends
    assert_eq!(scan(&scan_facts(new, old, 0.0, [0.0, 0.0])), scan(&scan_facts(old, new, 0.0, [0.0, 0.0])));
}

#[test]
fn an_entry_that_is_the_source_or_the_target_is_skipped_every_other_is_offered_and_the_first_hit_ends_the_scan() {
    use eech_world::entity_host::ScanState::{DoneHit, Next, Offer};
    let (old, new) = ([250.0, 0.0, 250.0], [260.0, 0.0, 250.0]);
    let mut entry = scan_facts(old, new, 1.0, [1.0, 1.0]);
    assert_eq!(scan(&entry).0, Offer);
    entry[14] = 1.0;
    assert_eq!(scan(&entry).0, Next, "the source and the target are never candidates");
    let mut resolved = scan_facts(old, new, 3.0, [1.0, 1.0]);
    assert_eq!(scan(&resolved).0, Next, "a miss goes on");
    resolved[15] = 1.0;
    assert_eq!(
        scan(&resolved).0,
        DoneHit,
        "the first hit ends the scan: a farther entry in an earlier position wins"
    );
}

#[test]
fn a_malformed_scan_question_or_a_cursor_outside_its_window_is_refused_never_defaulted() {
    let (old, new) = ([250.0, 0.0, 250.0], [260.0, 0.0, 250.0]);
    let refused = |facts: &[f32]| eech_world::entity_host::scan_step(&bits(facts)).is_err();
    assert!(refused(&scan_facts(old, new, 1.0, [0.0, 1.0])), "a cursor sector x below the window");
    assert!(refused(&scan_facts(old, new, 2.0, [1.0, 4.0])), "a cursor sector z above the window");
    assert!(refused(&scan_facts(old, new, 3.0, [4.0, 1.0])), "a cursor sector x above the window");
    for (label, at, value) in [
        ("phase 4", 11, 4.0),
        ("a fractional phase", 11, 0.5),
        ("side 0", 6, 0.0),
        ("a fractional cursor", 12, 1.5),
        ("a negative cursor", 13, -1.0),
        ("an excluded flag of 2", 14, 2.0),
        ("a hit flag of 2", 15, 2.0),
        ("a NaN segment", 0, f32::NAN),
    ] {
        let mut facts = scan_facts(old, new, 1.0, [1.0, 1.0]);
        facts[at] = value;
        assert!(refused(&facts), "{label}");
    }
    assert!(refused(&[0.0; 15]), "15 facts");
    assert!(refused(&[0.0; 17]), "17 facts");
}
