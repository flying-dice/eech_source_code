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

## What did NOT move (declared, required, not hidden)

This first slice is the **narrow phase** only. It is not whole entity/object collision, and nothing below is optional or dormant.

* **Core sector traversal and the physical broad phase stay in Core and are still required to move.** The sector window of the segment, the sector entity lists, the tail-first
  order, the exclusions of the source and target, the `TALLEST_STRUCTURE_HEIGHT` skip, the radius + 10 m broad-phase test, the city-level box and the classification of a candidate
  (collidable mobile or fixed, moving or stationary) are a physical scan over campaign-held entity state. So are the live pose derivation and its side effects: the destructive
  `UPDATED` getter (it ANDs the whole flag word, so it is a real query side effect whose order matters), the motion vector and delta time of a moving sweep, and a terrain-aligned fixed
  object's `get_3d_terrain_face_normal` (it writes the shared terrain point record, and is asked only after the broad phase passed). They keep their place because the scan was not
  touched. Moving them needs the host to hold, or be told, the live collidable roster (poses, scales, child state) with those side effects accounted for: that is the next Task 49 / D3 work
  and needs a characterised contract, not a guess. **The entity/object collision responsibility is not finished.**
* **Callers.** The hosted line-of-sight module `mb_hostlos.c`, the weapon-step collision lookup `wn_move.c` and the original-LOS caller `mb_tgt.c` call the unchanged function. The
  gunship avionics (`co_eo.c`, `co_hms.c`, `co_radar.c`) and `co_force.c` (which also reads `normal`) are compiled and call it too. **Whether the gunship callers are reachable from the
  supported headless runtime is unproved here, so no dormancy exclusion is made**: each stays a consumer of the same function and of the same host-asked narrow phase.
* Hierarchy (child/sub-object collision), live rotor/damage pose, smoke attachment, devices and muzzles are untouched required D3 work. City *children* are tested through
  the same narrow phase as before; a limit of the old fingerprint, not proof that collision ignores them.
* The raw tree loader, forest constructor and cloud block are as recorded in the card.

## DCS mapping (API shapes verified; the required mapping is unresolved)

A DCS host would answer `ENTITY_COLLISION` from its own world. Primary author-maintained declarations (`flying-dice/tslua-dcs` commit `1179554fcafcef92d71313336cc09500febf8245`;
`world.ts`, `Object.ts`, `Unit.ts`; Git blobs and SHA-256 in [PROVENANCE.json](../reference/d3-entity-collision/dcs-api/PROVENANCE.json), notes in
[MAPPING-NOTES.md](../reference/d3-entity-collision/dcs-api/MAPPING-NOTES.md)) declare these shapes. They are declarations, not DCS measurements and not an exhaustive proof that
nothing else exists.

| Needed fact | Declared call | What it does not establish |
|---|---|---|
| candidate discovery | `world.searchObjects(category, volume, handler, data)` (the handler can stop the search) | no nearest-hit order, no collision point or face normal, nothing like the legacy tail-first sector order: callback order must not be used as EECH's |
| live root pose and velocity | `Object.getPosition()` (`p`, `x`, `y`, `z`), `getPoint()`, `getVelocity()` | no named-child pose, mesh, or root-to-EECH mapping |
| existence, type, category | `Object.isExist()`, `getTypeName()`, `getCategory()` | a type name is not the campaign's host-registry identity |
| bounds | `Unit.getDesc()` / `getDescByName(typeName)`, `UnitDesc.box.min/max` | static descriptor bounds, not a collision mesh, child state or a hit/normal |
| animation input | `Unit.getDrawArgumentValue(argument)` | a draw argument is not a sub-object transform or geometry |

What the reference `ENTITY_COLLISION` needs is a candidate model at the supplied pose, the real hit decision and the final intercept and normal words. **None of the verified calls
supplies that contract, alone or together, so the required mapping is unresolved.** A rigid box from `UnitDesc.box` would be an explicitly different physical model, not an exact
model intersection, and is not approved as a silent substitute; no geometry catalogue is invented here, and an unsupported question is never answered as a miss. Which remaining active
Core callers need a DCS-facing primitive, and which are internal to the EECH reference physics backend (DCS resolves its own weapon physics without asking the EECH narrow phase, which does
not show that every Core collision consumer is unnecessary), is part of the next extraction. For any required primitive the work is to name a verified full API contract or a bounded,
versioned per-type host geometry and transform workaround with its provenance and supported types. Unknown objects, unsupported child state and missing geometry refuse by name.

## Checks (Linux, pinned image) and their status

**What each check covers.** `eech-world` `entity_collision_host` (9): triangle hit with crossing and normal, placement by position and attitude, a miss keeps the partial writes and a
non-crossing segment writes no point, the cursor is read in order and stops at the first hit, bounding-box fallback and its attitude, the three sweep tiers with strict boundaries, a refusal is
never a miss, rounding independence and state hand-back, and the intersection text equals the tree kernel's. `eech-engine-sys` `entity_host_guard` (3) and `tree_host_guard` (2): the hosted
branch has no solver, the original branch keeps it, the macro is defined only where the host model exists. `eech-engine` and `eech-dc`: the callback and the Lua bridge carry the pose and the final
words as exact bit patterns (a non-finite word included, for the entity words only) and refuse malformed requests and answers.

**Status, corrected.** The first-batch logs (`entity-collision-20261103-a`) piped Cargo into `tail`/`grep` under `sh` without `pipefail` and printed the outer shell's status, so their
`actual_exit` and `echo` codes are not Cargo's exits: `a3`/`a4` contain FAILED tests beside `actual_exit=0` (a real bridge defect, since fixed), `a9` a failed clippy invocation beside `c1=0`,
and `a8` a feature loop that built the default five times and a source hash from a failed Git command. They are retained as history and not counted. The valid evidence is the direct-exit gate
`entity-collision-20261105-b` (safe projection: [`entity-narrow-phase-direct-gate-20261105.json`](../reference/d3-entity-collision/entity-narrow-phase-direct-gate-20261105.json)): fourteen steps,
each a direct child whose exit was read on the next line, all 0 — `cargo fmt --all -- --check`; the four-crate tests with `--no-fail-fast` (62 result rows ok, no failed or panicked test);
`model_walk` (`model-fixture`) and `tree_dialogue` (`tree-dialogue-check`); `eech-engine-sys` builds for `collision-fixture`, `weapon-fixture`, `model-fixture`, `model-host-fixture` and
`weapon-fixture,collision-fixture` (so both the original and the hosted branch compile); strict clippy for the four crates, for `eech-dc` with `collision-fixture`, `weapon-fixture` and both, and for
`eech-engine-sys` with `tree-dialogue-check`. The code inputs' Git blob identity (2908 files, tracked and untracked, taken on the Windows checkout) is identical before and after the gate. The
progress guard's direct exit is 0 after the final card edit. A build whose artefacts were already fresh reports `Finished` from Cargo's own fingerprint check; that is a direct exit, not a recompile.

These are contract guards, not an oracle for retail models. What the retail models answer, and whether the campaign's behaviour is unchanged, is for the native comparison: the established
ordinary regression (byte-identical Tacview, observation and metric files) on a build of this source, plus, to show the host's answers actually reach the campaign, a changed-answer witness for
`ENTITY_COLLISION` (forced hit, forced miss, refusal) in the manner of `tree-authority.lua`. **Neither has been run.** The same independent reviewer's pass over this entity material is pending.
