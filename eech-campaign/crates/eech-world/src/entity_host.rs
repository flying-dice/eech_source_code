//! The host's physical test of whether a line segment meets ONE collision candidate (EEDCS-49, D3): the narrow phase of EECH's own `get_line_of_sight_collision_entity`
//! (`en_cllsn.c`), kept in C (`csrc/entity_step.c`, expression for expression) and called here for one question at a time.
//!
//! # What moved, what did not
//!
//! The campaign's `accurate_stationary_bounding_box_collision_test` and `accurate_moving_bounding_box_collision_test` no longer transform the model or intersect anything in
//! the hosted module: they derive the candidate's live pose from the entity (in the original's order, including a terrain-aligned fixed object's placement) and ask the host
//! through the model callback's `ENTITY_COLLISION` operation (`mb_hostmodel.h`). This module answers it with the model data of the host's own final announced-order
//! [`ModelStore`], the same store the other model operations read: the scene's collision triangles one depth at a time (the original's cursor, so the first triangle that is hit
//! ends the test and later triangles are never read) or, for a scene with none, the corners of its bounding box without lines.
//!
//! The campaign still owns WHICH candidates are visited and in what order (the sector scan, the exclusions, the broad-phase radius test) and the candidate state the pose is
//! derived from. That is declared transitional coupling: the sector lists and the entities' live state are the campaign's until the physical roster and broad phase move too.
//! Nothing here claims they have.
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
