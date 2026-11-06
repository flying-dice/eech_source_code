//! The host's physical test of whether a line segment meets ONE collision candidate (EEDCS-49, D3): the narrow phase of EECH's own `get_line_of_sight_collision_entity`
//! (`en_cllsn.c`), kept in C (`csrc/entity_step.c`, expression for expression) and called here for one question at a time.
//!
//! # What moved, what did not
//!
//! The campaign's `accurate_stationary_bounding_box_collision_test` and `accurate_moving_bounding_box_collision_test` no longer exist in the hosted module, and nothing about a candidate is
//! decided there. FIRST the campaign offers each flagged candidate with the facts it read from the entity and the host decides (`candidate`, below: the third slice, the model callback's
//! `ENTITY_CANDIDATE` operation); WHEN the host says the narrow phase is required it hands back the candidate's pose and the campaign asks `ENTITY_COLLISION` (`mb_hostmodel.h`). This module
//! answers the second question with the model data of the host's own final announced-order
//! [`ModelStore`], the same store the other model operations read: the scene's collision triangles one depth at a time (the original's cursor, so the first triangle that is hit
//! ends the test and later triangles are never read) or, for a scene with none, the corners of its bounding box without lines.
//!
//! The HOST owns which candidates are offered and in what order: the sector window and the traversal (`scan_step`) and which entities stand in each sector in what order (`crate::roster`, served
//! by `crate::model_host`: the campaign reports every insertion into and removal from a sector list and the host issues the keys it serves the scan by). The campaign's lists are only the
//! conformance mirror of that order. The campaign still owns the exclusion of the source and the target, a sector's tallest structure, a city's buildings, the reads of the entity state the
//! facts are made of, and it performs the two getters that have effects of their own when the host names them: that is declared coupling, required to move, and nothing here claims it has.
//!
//! # Exactness
//!
//! The intersection helpers write the intercept point and the face normal BEFORE they know whether the segment hits, so a miss leaves partial writes. The caller's words are an
//! input of the question and the final words are returned whether or not there was a hit. The arithmetic runs under the original's context, as the tree kernel's does: the entry
//! REQUIRES the x87 and SSE exception masks to be 63 and 63, saves the caller's complete floating-point state, sets round-toward-zero for both units, runs the kernel and restores
//! the state it found (read back, an error otherwise). The callback that reads the model does integer work and a table lookup between the kernel's steps.

use crate::models::{ModelStore, Triangle};
use crate::trees::{fp_state, Rounding, ScopedRounding};
use std::os::raw::{c_int, c_void};
use std::panic::{catch_unwind, AssertUnwindSafe};

/// the floats of a stationary candidate's question and of a moving candidate's: the segment's two ends, then the candidate's pose. The caller's intercept point and normal are
/// outputs of the original and are neither part of the question nor read.
pub const STATIONARY_POSE: usize = 18;
pub const MOVING_POSE: usize = 22;

/// the write mask of an answer: bits 0 to 2 the intercept point, bits 3 to 5 the face normal
pub const POINT_ASSIGNED: u32 = 0x07;
pub const NORMAL_ASSIGNED: u32 = 0x38;

/// The next collision triangle of the scene at `depth` (the original's cursor): `Some(nine words)`, or `None` for the end or the absence of one. An `Err` is a refusal of the model.
pub type Cursor<'a> = dyn FnMut(i32) -> Result<Option<[u32; 9]>, String> + 'a;

type TriangleFn = unsafe extern "C" fn(*mut c_void, c_int, *mut f32) -> c_int;

extern "C" {
    fn eech_entity_collision(
        pose: *const f32,
        is_moving: c_int,
        bounds: *const f32,
        triangles: TriangleFn,
        context: *mut c_void,
        result: *mut f32,
        assigned: *mut u32,
    ) -> c_int;
}

/// What one question answers: whether the segment hit, which of the six output components the original's arithmetic assigned (`assigned`: `POINT_ASSIGNED` and `NORMAL_ASSIGNED`
/// bits; the original assigns the whole normal on its first intersection call and the whole point only once the segment has crossed a face's plane, so a miss can have assigned
/// the normal or both), and the computed words of the intercept point (3) then the face normal (3). The words of a component that was not assigned are zero and mean nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityTest {
    pub hit: bool,
    pub assigned: u32,
    pub words: [u32; 6],
}

struct Context<'a, 'b> {
    next: &'a mut Cursor<'b>,
    failure: Option<String>,
}

unsafe extern "C" fn triangle(context: *mut c_void, depth: c_int, points: *mut f32) -> c_int {
    // SAFETY: the kernel passes back the context pointer it was given for the duration of one call and a buffer of at least nine floats.
    let context = unsafe { &mut *context.cast::<Context>() };
    match catch_unwind(AssertUnwindSafe(|| (context.next)(depth))) {
        Ok(Ok(Some(words))) => {
            for (at, word) in words.iter().enumerate() {
                // SAFETY: `points` has room for nine floats (a triangle is three vec3d).
                unsafe { points.add(at).write(f32::from_bits(*word)) };
            }
            1
        }
        Ok(Ok(None)) => 0,
        Ok(Err(why)) => {
            context.failure = Some(why);
            -1
        }
        Err(_) => {
            context.failure = Some("the model cursor panicked".into());
            -1
        }
    }
}

/// Tests the segment against one candidate. `pose` is `STATIONARY_POSE` or `MOVING_POSE` words (see the module and `mb_hostmodel.h`), `bounds` the scene's bounding box without
/// lines (xmin, xmax, ymin, ymax, zmin, zmax) and `next` the scene's collision-triangle cursor.
pub fn collide(pose: &[u32], bounds: [u32; 6], next: &mut Cursor) -> Result<EntityTest, String> {
    let moving = match pose.len() {
        STATIONARY_POSE => false,
        MOVING_POSE => true,
        other => return Err(format!("an entity collision pose has {STATIONARY_POSE} or {MOVING_POSE} floats, not {other}")),
    };
    let state = fp_state()?;
    if state.x87_masks != 63 || state.sse_masks != 63 {
        return Err(format!(
            "the entity collision kernel requires every floating-point exception masked in both units (x87 {}, SSE {})",
            state.x87_masks, state.sse_masks
        ));
    }
    let pose: Vec<f32> = pose.iter().map(|word| f32::from_bits(*word)).collect();
    let bounds: [f32; 6] = bounds.map(f32::from_bits);
    let mut result = [0.0f32; 6];
    let mut assigned = 0u32;
    let mut context = Context { next, failure: None };
    let hit = {
        // the kernel's arithmetic is the original's: round toward zero, the caller's whole state handed back when the guard ends
        let _original_arithmetic = ScopedRounding::set(Rounding::Zero)?;
        // SAFETY: `pose` has the 18 or 22 floats the kernel reads for the kind it is told, `bounds` six, `result` six, `assigned` one; the context outlives the call and is used only by `triangle`.
        unsafe {
            eech_entity_collision(
                pose.as_ptr(),
                c_int::from(moving),
                bounds.as_ptr(),
                triangle,
                std::ptr::addr_of_mut!(context).cast(),
                result.as_mut_ptr(),
                &mut assigned,
            )
        }
    };
    let after = fp_state()?;
    if after != state {
        return Err("the entity collision kernel did not hand back the caller's floating-point state".into());
    }
    if hit < 0 {
        return Err(context.failure.unwrap_or_else(|| "the model cursor refused".into()));
    }
    Ok(EntityTest {
        hit: hit == 1,
        assigned,
        words: result.map(f32::to_bits),
    })
}

/// Tests the segment against the scene in the host's final store: the cursor and the bounds are the store's own, and a scene the store does not have loaded is a refusal.
pub fn collide_in_store(store: &ModelStore, scene: i64, pose: &[u32]) -> Result<EntityTest, String> {
    let view = store.scene(scene).map_err(|e| e.to_string())?;
    if !view.loaded {
        return Err(format!("scene {scene} is not loaded in the host's model store"));
    }
    collide(pose, view.bounding_box2, &mut |depth| match store.triangle(scene, i64::from(depth)) {
        Ok(Triangle::Found(words)) => Ok(Some(words)),
        Ok(Triangle::End | Triangle::Absent) => Ok(None),
        Ok(Triangle::Refused(reason)) => Err(format!(
            "the store cannot answer the collision triangle of scene {scene} depth {depth}: {reason}"
        )),
        Err(e) => Err(e.to_string()),
    })
}

/// The floats of an `ENTITY_CANDIDATE` question: the facts the campaign reads from the candidate it is visiting (`csrc/entity_candidate_step.c` lists them, staged in the order the original
/// needed them), and of the narrow-phase pose the host answers a `TEST` with.
pub const CANDIDATE_FACTS: usize = 44;
/// Where the facts of an ENTITY_CANDIDATE question that the host's physical record decides for a mobile candidate sit (mb_hostmodel.h, CAMPAIGN_CANDIDATE_*): the flags, the position (3), the motion vector (3),
/// the attitude (9) and the flag that says the placement facts were read.
pub const CANDIDATE_MOBILE: usize = 6;
pub const CANDIDATE_OF_CITY: usize = 10;
pub const CANDIDATE_POSITION: usize = 13;
pub const CANDIDATE_MOTION: usize = 16;
pub const CANDIDATE_ATTITUDE: usize = 27;
pub const CANDIDATE_PLACEMENT_GIVEN: usize = 43;
pub const CANDIDATE_STATIONARY_POSE: usize = 12;
pub const CANDIDATE_MOVING_POSE: usize = 16;

/// What the host decided about one candidate (`csrc/entity_candidate_step.c`). `NeedUpdated` and `NeedNormal` ask the campaign for a service that has effects of its own, `NeedFixed` and
/// `NeedPlacement` for facts the branch has only now reached (the fixed flag and what the fixed branch reads; the attitude, or the heading and align flag, once the broad phase has passed),
/// and the question is then asked again with them filled in; `Buildings` says a city's box is met, so its buildings are offered one by one; `Test` carries the candidate's pose for the
/// narrow phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateState {
    Skip,
    NeedUpdated,
    NeedNormal,
    Buildings,
    Test,
    NeedFixed,
    NeedPlacement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateStep {
    pub state: CandidateState,
    /// the narrow phase's candidate pose (words): 12 or 16 for `Test`, empty otherwise
    pub pose: Vec<u32>,
}

/// The radius of the candidate's scene, in words: asked for ONLY by a branch that has reached its broad phase, so that a candidate the original skipped needs no model. An `Err` is a refusal
/// of the model data (an unusable scene), never a default radius and never a miss.
pub type Radius<'a> = dyn FnMut() -> Result<u32, String> + 'a;

extern "C" {
    fn eech_entity_candidate(facts: *const f32, has_radius: c_int, radius: f32, candidate: *mut f32, candidate_count: *mut c_int) -> c_int;
}

const CANDIDATE_SKIP: c_int = 1;
const CANDIDATE_NEED_UPDATED: c_int = 2;
const CANDIDATE_NEED_NORMAL: c_int = 3;
const CANDIDATE_BUILDINGS: c_int = 4;
const CANDIDATE_TEST: c_int = 5;
const CANDIDATE_NEED_FIXED: c_int = 6;
const CANDIDATE_NEED_PLACEMENT: c_int = 7;
const CANDIDATE_NEED_RADIUS: c_int = 8;
const CANDIDATE_INVALID_TARGET: c_int = -1;

/// The facts of a candidate that are flags or states, and the values each may have: the collision flags, the city facts and the role (6 to 11), the UPDATED state (20), the align flag and
/// the normal-given flag (22, 23), and the two facts-read flags (42, 43). Anything else is a malformed question, never a default.
fn check_candidate_facts(facts: &[f32]) -> Result<(), String> {
    if facts.len() != CANDIDATE_FACTS {
        return Err(format!("an entity candidate question has {CANDIDATE_FACTS} floats, not {}", facts.len()));
    }
    if let Some(at) = facts.iter().position(|value| !value.is_finite()) {
        return Err(format!("fact {at} of an entity candidate question is not a finite number"));
    }
    for at in [6, 7, 8, 9, 10, 11, 22, 23, 42, 43] {
        if facts[at] != 0.0 && facts[at] != 1.0 {
            return Err(format!("fact {at} of an entity candidate question is a flag and must be 0 or 1"));
        }
    }
    if ![0.0, 1.0, 2.0].contains(&facts[20]) {
        return Err("fact 20 of an entity candidate question is the UPDATED state and must be 0, 1 or 2".into());
    }
    if facts[8] == 1.0 && facts[9] == 1.0 {
        return Err("an entity candidate cannot be both a city and a city building".into());
    }
    if facts[10] == 1.0 && facts[8] == 1.0 {
        return Err("a building listed under a city cannot be a city".into());
    }
    Ok(())
}

/// The raw request classifier of an `ENTITY_CANDIDATE` question: the SAME finite and flag domains as `check_candidate_facts`, in the SAME order and with the SAME messages, but over the words as
/// they arrived and with INTEGER operations only (an exponent test for finite, an exact bit test for a flag or a state). It runs before anything is evaluated as a floating-point value and before any
/// registry lookup, so a signalling NaN or a malformed flag selects its request error without a floating-point comparison, without touching the caller's floating-point status and without being
/// routed through a record (or a busy registry) first. A flag is exactly zero (either sign) or one; the UPDATED state is zero, one or two. The facts that are not flags (the segment, the position,
/// the motion vector, the attitude, the box and the rest) are checked for being finite only, as before, and no otherwise-unused fact is validated further.
pub fn classify_candidate_words(words: &[u32]) -> Result<(), String> {
    const ONE: u32 = 0x3f80_0000;
    const TWO: u32 = 0x4000_0000;
    let finite = |word: u32| word & 0x7f80_0000 != 0x7f80_0000;
    let zero = |word: u32| word & 0x7fff_ffff == 0;
    if words.len() != CANDIDATE_FACTS {
        return Err(format!("an entity candidate question has {CANDIDATE_FACTS} floats, not {}", words.len()));
    }
    if let Some(at) = words.iter().position(|word| !finite(*word)) {
        return Err(format!("fact {at} of an entity candidate question is not a finite number"));
    }
    for at in [6, 7, 8, 9, 10, 11, 22, 23, 42, 43] {
        if !zero(words[at]) && words[at] != ONE {
            return Err(format!("fact {at} of an entity candidate question is a flag and must be 0 or 1"));
        }
    }
    if !(zero(words[20]) || words[20] == ONE || words[20] == TWO) {
        return Err("fact 20 of an entity candidate question is the UPDATED state and must be 0, 1 or 2".into());
    }
    if words[8] == ONE && words[9] == ONE {
        return Err("an entity candidate cannot be both a city and a city building".into());
    }
    if words[10] == ONE && words[8] == ONE {
        return Err("a building listed under a city cannot be a city".into());
    }
    Ok(())
}

/// Whether a fact word of an already classified question (`classify_candidate_words`) is a set flag: its value is exactly zero or one, so any bit that is not the sign decides.
pub fn candidate_flag_is_set(word: u32) -> bool {
    word & 0x7fff_ffff != 0
}

/// One run of the kernel under the original's arithmetic, with the caller's complete floating-point state saved and restored: the raw state it answered and its pose floats.
fn run_candidate(facts: &[f32], radius: Option<f32>) -> Result<(c_int, [f32; CANDIDATE_MOVING_POSE], c_int), String> {
    let state = fp_state()?;
    if state.x87_masks != 63 || state.sse_masks != 63 {
        return Err(format!(
            "the entity candidate kernel requires every floating-point exception masked in both units (x87 {}, SSE {})",
            state.x87_masks, state.sse_masks
        ));
    }
    let mut pose = [0.0f32; CANDIDATE_MOVING_POSE];
    let mut count: c_int = 0;
    let decided = {
        let _original_arithmetic = ScopedRounding::set(Rounding::Zero)?;
        // SAFETY: `facts` has the 44 floats the kernel reads, `pose` has room for 16 and `count` is one int.
        unsafe {
            eech_entity_candidate(
                facts.as_ptr(),
                c_int::from(radius.is_some()),
                radius.unwrap_or(0.0),
                pose.as_mut_ptr(),
                &mut count,
            )
        }
    };
    let after = fp_state()?;
    if after != state {
        return Err("the entity candidate kernel did not hand back the caller's floating-point state".into());
    }
    Ok((decided, pose, count))
}

/// The host's decision about one candidate, from the campaign's facts. The decision is the original's, run under round-toward-zero with the caller's complete floating-point state saved and
/// restored, as the narrow phase's is. `radius` is asked for only if the branch the facts take reaches a broad phase.
pub fn candidate(facts: &[u32], radius: &mut Radius) -> Result<CandidateStep, String> {
    classify_candidate_words(facts)?;
    let facts: Vec<f32> = facts.iter().map(|word| f32::from_bits(*word)).collect();
    check_candidate_facts(&facts)?;
    let (mut decided, mut pose, mut count) = run_candidate(&facts, None)?;
    if decided == CANDIDATE_NEED_RADIUS {
        let radius = f32::from_bits(radius()?);
        if !radius.is_finite() {
            return Err("the radius of the candidate's scene is not a finite number".into());
        }
        (decided, pose, count) = run_candidate(&facts, Some(radius))?;
    }
    let step = |state, pose: Vec<u32>| Ok(CandidateStep { state, pose });
    match decided {
        CANDIDATE_SKIP => step(CandidateState::Skip, Vec::new()),
        CANDIDATE_NEED_UPDATED => step(CandidateState::NeedUpdated, Vec::new()),
        CANDIDATE_NEED_NORMAL => step(CandidateState::NeedNormal, Vec::new()),
        CANDIDATE_BUILDINGS => step(CandidateState::Buildings, Vec::new()),
        CANDIDATE_NEED_FIXED => step(CandidateState::NeedFixed, Vec::new()),
        CANDIDATE_NEED_PLACEMENT => step(CandidateState::NeedPlacement, Vec::new()),
        CANDIDATE_TEST => match count {
            12 | 16 => step(CandidateState::Test, pose[..count as usize].iter().map(|value| value.to_bits()).collect()),
            other => Err(format!("the entity candidate kernel gave a pose of {other} floats")),
        },
        CANDIDATE_INVALID_TARGET => Err("an entity candidate that is neither mobile nor fixed reached its pose (the original's `Invalid target`)".into()),
        CANDIDATE_NEED_RADIUS => Err("the entity candidate kernel asked for the radius twice".into()),
        other => Err(format!("the entity candidate kernel answered {other}")),
    }
}

/// The decision over the host's final store: the radius is the scene's own and is looked up only when the branch needs it (a candidate the original skipped, a direct city building and a
/// city need no model), and a scene the store does not have, or has not loaded, is then a refusal.
pub fn candidate_in_store(store: &ModelStore, scene: i64, facts: &[u32]) -> Result<CandidateStep, String> {
    candidate(facts, &mut || {
        if scene < 1 {
            return Err(format!("scene {scene} is not a scene a candidate can have"));
        }
        let view = store.scene(scene).map_err(|e| e.to_string())?;
        if !view.loaded {
            return Err(format!("scene {scene} is not loaded in the host's model store"));
        }
        Ok(view.radius)
    })
}

/// The floats of an `ENTITY_SCAN` question (`csrc/entity_scan_step.c` lists them): the segment, the campaign's world map, the phase of the scan, the cursor the host last gave and the two facts the
/// campaign reports about the entry it served.
pub const SCAN_FACTS: usize = 16;

/// What the host decided about the enumeration (`csrc/entity_scan_step.c`). `First` asks the campaign for the TAIL of the list of the sector it names; `Next` for the predecessor of the entry
/// it is at; `Offer` to offer the entry it is at for its decisions; `DoneNone` ends the scan with no hit; `DoneHit` ends it with the hit of the entry offered last.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanState {
    First,
    Next,
    Offer,
    DoneNone,
    DoneHit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanStep {
    pub state: ScanState,
    /// the sector the host names, as whole numbers in float words; meaningful for `First` only (zero otherwise)
    pub cursor: [u32; 2],
}

extern "C" {
    fn eech_entity_scan_step(facts: *const f32, cursor: *mut f32) -> c_int;
}

/// The facts of a scan question that are flags, phases or whole numbers, and the values each may have. Anything else is a malformed question, never a default.
fn check_scan_facts(facts: &[f32]) -> Result<(), String> {
    if facts.len() != SCAN_FACTS {
        return Err(format!("an entity scan question has {SCAN_FACTS} floats, not {}", facts.len()));
    }
    if let Some(at) = facts.iter().position(|value| !value.is_finite()) {
        return Err(format!("fact {at} of an entity scan question is not a finite number"));
    }
    let whole = |at: usize, low: f32, high: f32| -> Result<(), String> {
        if facts[at] != facts[at].floor() || facts[at] < low || facts[at] > high {
            return Err(format!("fact {at} of an entity scan question must be a whole number within {low}..{high}"));
        }
        Ok(())
    };
    whole(6, 1.0, 16_777_216.0)?;
    whole(11, 0.0, 3.0)?;
    whole(12, 0.0, 16_777_216.0)?;
    whole(13, 0.0, 16_777_216.0)?;
    whole(14, 0.0, 1.0)?;
    whole(15, 0.0, 1.0)?;
    for at in [0, 1, 2, 3, 4, 5, 7, 8, 9, 10] {
        if facts[at].abs() > 1.0e9 {
            return Err(format!("fact {at} of an entity scan question is out of range"));
        }
    }
    Ok(())
}

/// The host's decision about the enumeration, from the campaign's facts, under the original's arithmetic with the caller's floating-point state saved and restored.
pub fn scan_step(facts: &[u32]) -> Result<ScanStep, String> {
    let facts: Vec<f32> = facts.iter().map(|word| f32::from_bits(*word)).collect();
    check_scan_facts(&facts)?;
    let state = fp_state()?;
    if state.x87_masks != 63 || state.sse_masks != 63 {
        return Err(format!(
            "the entity scan kernel requires every floating-point exception masked in both units (x87 {}, SSE {})",
            state.x87_masks, state.sse_masks
        ));
    }
    let mut cursor = [0.0f32; 2];
    let decided = {
        let _original_arithmetic = ScopedRounding::set(Rounding::Zero)?;
        // SAFETY: `facts` has the 16 floats the kernel reads and `cursor` has room for two.
        unsafe { eech_entity_scan_step(facts.as_ptr(), cursor.as_mut_ptr()) }
    };
    let after = fp_state()?;
    if after != state {
        return Err("the entity scan kernel did not hand back the caller's floating-point state".into());
    }
    let step = |state| {
        Ok(ScanStep {
            state,
            cursor: cursor.map(f32::to_bits),
        })
    };
    match decided {
        1 => step(ScanState::First),
        2 => step(ScanState::Next),
        3 => step(ScanState::Offer),
        4 => step(ScanState::DoneNone),
        5 => step(ScanState::DoneHit),
        -1 => Err("the cursor of an entity scan question lies outside the sector window of its segment".into()),
        other => Err(format!("the entity scan kernel answered {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// a deterministic stream of 32-bit words (xorshift)
    fn stream(seed: u32) -> impl FnMut() -> u32 {
        let mut state = seed;
        move || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state
        }
    }

    fn the_floating_classifier(words: &[u32]) -> Result<(), String> {
        let facts: Vec<f32> = words.iter().map(|word| f32::from_bits(*word)).collect();
        check_candidate_facts(&facts)
    }

    #[test]
    fn the_raw_classifier_is_the_established_floating_one_word_for_word_in_the_same_order_with_the_same_messages() {
        let mut next = stream(0x00f1_a9c5);
        let specials: [u32; 12] = [
            0,
            0x8000_0000,
            0x3f80_0000,
            0xbf80_0000,
            0x4000_0000,
            0x4040_0000,
            0x7f80_0000,
            0xff80_0000,
            0x7fa0_0001,
            0xffc0_0000,
            0x0000_0001,
            0x3f00_0000,
        ];
        let flags = [6usize, 7, 8, 9, 10, 11, 22, 23, 42, 43, 20];
        let mut checked = 0;
        for round in 0..200_000 {
            let mut words = vec![0u32; CANDIDATE_FACTS];
            // the ordinary case: the flags and states hold values near the valid domain, the rest are plain finite words; every round puts one or two odd words anywhere
            for (at, word) in words.iter_mut().enumerate() {
                *word = if flags.contains(&at) {
                    specials[(next() % 5) as usize]
                } else {
                    f32::to_bits((next() % 2000) as f32 - 1000.0)
                };
            }
            for _ in 0..(round % 3) {
                let at = (next() as usize) % CANDIDATE_FACTS;
                words[at] = if next().is_multiple_of(2) { specials[(next() % 12) as usize] } else { next() };
            }
            // `check_candidate_facts` compares only finite words, so the corpus may carry a NaN only where its own finite test comes first: which is exactly the order the two must share
            assert_eq!(classify_candidate_words(&words), the_floating_classifier(&words), "{words:x?}");
            checked += 1;
        }
        assert_eq!(checked, 200_000);
        for count in [0usize, 43, 45] {
            assert_eq!(classify_candidate_words(&vec![0; count]), the_floating_classifier(&vec![0; count]));
        }
        // the old accepted semantics: a negative zero is a zero, one is one, two is a state but not a flag
        let mut valid = vec![0u32; CANDIDATE_FACTS];
        valid[6] = 0x8000_0000;
        valid[43] = 0x3f80_0000;
        valid[20] = 0x4000_0000;
        assert_eq!(classify_candidate_words(&valid), Ok(()));
        valid[6] = 0x4000_0000;
        assert!(classify_candidate_words(&valid).unwrap_err().contains("fact 6") && classify_candidate_words(&valid).unwrap_err().contains("flag"));
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn classifying_a_malformed_question_changes_neither_the_status_nor_the_control_word_whatever_state_the_caller_is_in() {
        use crate::reference_arithmetic::{current_mxcsr, write_mxcsr};
        let signalling = {
            let mut words = vec![0u32; CANDIDATE_FACTS];
            words[6] = 0x7fa0_0001;
            words
        };
        let mut finite_bad = vec![0u32; CANDIDATE_FACTS];
        finite_bad[6] = 0x4000_0000;
        let saved = current_mxcsr();
        let mut failures = Vec::new();
        for rounding in 0u32..4 {
            for flags in [0x00u32, 0x01, 0x3f] {
                // every exception masked, then every exception UNMASKED: a floating-point comparison of the signalling NaN would trap here
                for masks in [0x1f80u32, 0x0000] {
                    let wanted = (rounding << 13) | masks | flags;
                    write_mxcsr(wanted);
                    let before = current_mxcsr();
                    let first = classify_candidate_words(&signalling);
                    let second = classify_candidate_words(&finite_bad);
                    let after = current_mxcsr();
                    if before != after || first.is_ok() || second.is_ok() {
                        failures.push((wanted, before, after));
                    }
                }
            }
        }
        write_mxcsr(saved);
        assert!(
            failures.is_empty(),
            "the raw classifier disturbed the floating-point state or accepted a malformed word: {failures:x?}"
        );
    }
}
