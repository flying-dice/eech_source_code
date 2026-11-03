# D3 entity/object collision: the host-owned narrow phase (EEDCS-49, second slice)

3 November 2026. **SOURCE and focused Linux checks only. No native, Windows or retail run has exercised this change; nothing here is accepted.** This is the
*entity/object* slice of Task 49 (the tree slice is [d3-tree-collision-provider.md](d3-tree-collision-provider.md)). It is not Task 49 and not D3.

## What `get_line_of_sight_collision_entity` is, from source

`en_cllsn.c` scans the sector window of the segment (its box widened by `LARGEST_3D_OBJECT_RADIUS`, x outer, z inner), walks each sector's entity list from the tail
and, for every entity that is not the excluded source or target, applies a broad phase (a box of radius + 10 m) and then a narrow phase against the entity's model:
the collision triangles when the scene has any, else the six faces of its bounding box without lines. The first candidate whose narrow phase hits ends the scan; the
intercept point and face normal are written by the intersection helpers *before* the hit decision, so a miss can leave partial writes (`docs/m5-collision-characterization.md`).

Inputs and where they come from (all measured or read, none assumed):

| Input | Source | Owner after this slice |
|---|---|---|
| segment ends, caller's point/normal buffers, excluded source/target | callers (`mb_hostlos.c`, `mb_tgt.c`, `wn_move.c` COLLISION lookup, gunship avionics/`co_force.c`) | campaign (unchanged) |
| sector window, sector lists, tail-first order, exclusion, `TALLEST_STRUCTURE_HEIGHT` skip, city box, mobile/fixed/moving classification, the broad-phase radius test | campaign entities and the host-authored radius table | **campaign, declared transitional coupling** |
| moving sweep ends (`UPDATED` flag, motion vector, `get_delta_time`) | campaign entity state; the destructive `UPDATED` getter keeps its place because the scan is untouched | campaign |
| stationary placement (position, heading, terrain-aligned normal) | campaign entity state and the terrain question `get_3d_terrain_face_normal`, asked only when the broad phase passed, as before | campaign (derived in the original's order) |
| the model: collision triangles (one depth at a time), bounding box without lines | the host's final announced-order `ModelStore` (Task 50) | host |
| transform of the model into the world, which sweep positions a moving candidate tests (speed against body length), face order, intersection arithmetic, partial writes | `accurate_stationary_bounding_box_collision_test`, `accurate_moving_bounding_box_collision_test`, `vector.c` | **host (moved)** |

## What moved

The ordinary hosted module (`EECH_CAMPAIGN_HOST_ENTITY`) no longer compiles the two `accurate_*` tests, the face wrappers or the transform arithmetic. Each now derives the
candidate's pose from the entity, in the original's order, and asks the model callback's new `ENTITY_COLLISION` operation (`mb_hostmodel.h`): the segment's two ends, the
caller's intercept point and normal (inputs *and* outputs), then a stationary candidate's position and attitude (24 floats) or a moving one's sweep ends, attitude and speed
(28). The reply is a hit or a miss, the final point and normal words (copied back whether or not it hit, none required finite) and the scene's compiled name (an echo). A
refusal, a malformed answer or a missing provider stops the engine; nothing is invented. There is **no fallback** to the original arithmetic in the hosted module. The explicit
original-characterisation profile (`collision-fixture`) and the model fixtures keep the original branch (compile guard `entity_host_guard`).

The reference host answers with `csrc/entity_step.c` (the original expression for expression; the two intersections are the same text as the tree kernel's, a test compares them)
driven by `src/entity_host.rs`, which, as the tree kernel does, requires every floating-point exception masked, runs under round-toward-zero and hands the caller's complete
state back. The model data is the *same* final store the other model operations read: no second loader or cache.

## What did NOT move (declared, not hidden)

* The sector scan, the entity lists, the exclusions, the broad-phase radius test, the city-level box and the classification of candidates remain in the campaign. This is a
  physical *broad phase* over campaign-held state, and it is still Core: **the entity/object collision responsibility is not finished.** Moving it needs the host to hold
  or be told the live collidable roster (poses, scales, child state); that is the next Task 49 / D3 slice and needs a characterised contract, not a guess.
* Callers that are not reachable on the supported headless path (the gunship avionics and `co_force.c`, which also reads `normal`) and the original-LOS caller `mb_tgt.c` are
  unchanged and keep calling the same function; whether each is reachable is still undecided here.
* Hierarchy (child/sub-object collision), live rotor/damage pose, smoke attachment, devices and muzzles are untouched required D3 work. City *children* are tested through
  the same narrow phase as before.
* The raw tree loader, forest constructor and cloud block, and the DCS tree mapping, are as recorded in the card.

## DCS mapping (unresolved, not claimed)

A DCS host would answer `ENTITY_COLLISION` from its own world. The pinned `tslua-dcs` declarations used for the target-point plan were not re-read for this slice, so **no DCS
call is claimed**. What the host must supply per candidate is: a hit/miss for the candidate's model at the pose the campaign sends, and the intercept point and normal if it
hits. Candidate primitives to verify against the primary declarations (not yet verified): a segment/volume object search that yields candidate objects, plus a way to obtain
the first intersection point and face normal on a hit. If no primary-evidenced call gives the point and normal, a bounded workaround (model data for the DCS object types)
must be named. Until then a DCS host must *refuse* the question by name.

## Checks run (Linux, pinned image; see the card for the log identities)

* `eech-world` `entity_collision_host` (9): triangle hit with crossing and normal, placement by position and attitude, miss keeps partial writes, the cursor is read in order
  and stops at the first hit, bounding-box fallback and its attitude, the three sweep tiers with strict boundaries, refusal is never a miss, rounding independence and
  state hand-back, the intersection text equals the tree kernel's.
* `eech-engine-sys` `entity_host_guard` (3), `tree_host_guard` (2): the hosted branch has no solver, the original branch keeps it, the build defines the macro only where the host
  model exists.
* `eech-engine` and `eech-dc`: the callback and the Lua bridge carry the pose and the final words as exact bit patterns and refuse malformed requests and answers.

These are contract guards, not an oracle for retail models. What the retail models answer, and whether the campaign's behaviour is unchanged, is for the native comparison:
the established ordinary regression (byte-identical Tacview, observation and metric files) on a build of this source, plus, to show the host's answers actually reach the
campaign, a changed-answer witness for `ENTITY_COLLISION` (forced hit, forced miss, refusal) in the manner of `tree-authority.lua`. **Neither has been run.**
