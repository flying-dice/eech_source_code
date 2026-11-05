# D3 entity/object collision: the host-owned narrow phase (EEDCS-49, second slice)

3 November 2026, status refreshed 5 November 2026; the third slice (the per-candidate decisions, see "The scan") was added the same day as SOURCE and Linux checks only. **The narrow phase is native-validated for the FINITE STATIONARY handoff only (corrected source `f153`/`7ead`; [`finite-authority/`](../reference/d3-entity-collision/finite-authority/README.md)); the moving public path, the physical scan and Task 49 are NOT accepted.** The body below is the dated 3 to 5 November account; its statements about runs are updated in place where they have since changed. This is the
*entity/object* slice of Task 49 (the tree slice is [d3-tree-collision-provider.md](d3-tree-collision-provider.md)). It is not Task 49 and not D3.

## What `get_line_of_sight_collision_entity` is, from source

`en_cllsn.c` scans the sector window of the segment (its box widened by `LARGEST_3D_OBJECT_RADIUS`, x outer, z inner), walks each sector's entity list from the tail
and, for every entity that is not the excluded source or target, applies a broad phase (a box of radius + 10 m) and then a narrow phase against the entity's model:
the collision triangles when the scene has any, else the six faces of its bounding box without lines. The first candidate whose narrow phase hits ends the scan; the
intercept point and face normal are OUTPUTS written by the intersection helpers: the normal on every call, before any decision, and the point only once the segment has crossed the face's
plane, so a miss can leave a written normal, or both (`docs/m5-collision-characterization.md`; per-component account under "The output contract" below).

Inputs and where they come from (all measured or read, none assumed):

| Input | Source | Owner after this slice |
|---|---|---|
| segment ends, excluded source/target (the caller's point/normal buffers are outputs and are never read) | callers (`mb_hostlos.c`, `mb_tgt.c`, `wn_move.c` COLLISION lookup, gunship avionics/`co_force.c`) | campaign (unchanged) |
| sector window, sector lists, tail-first order, the source/target exclusion, the reads of each entity's state, in stages and only as the host asks | campaign entities (the sector lists ARE the campaign's membership) | **campaign, declared transitional coupling** |
| `TALLEST_STRUCTURE_HEIGHT` skip, mobile/fixed/city branch, moving classification, the broad-phase box of radius + 10 m (the host's radius table), the city's height-then-xz box, a building listed directly in a sector | the facts the campaign reads, and the host's model data | **host (third slice, moved)** |
| moving sweep ends (from the `UPDATED` answer, motion vector, `get_delta_time`) | the host decides from facts; the destructive `UPDATED` getter is performed by the campaign exactly when the host names it | **host decides, campaign performs the getter** |
| stationary placement (position, heading, terrain-aligned normal) | the host derives the placement; the terrain question `get_3d_terrain_face_normal` is asked by the campaign when the host names it, once the broad phase passed | **host decides, campaign performs the terrain question** |
| the model: collision triangles (one depth at a time), bounding box without lines | the host's final announced-order `ModelStore` (Task 50) | host |
| transform of the model into the world, which sweep positions a moving candidate tests (speed against body length), face order, intersection arithmetic, partial writes | `accurate_stationary_bounding_box_collision_test`, `accurate_moving_bounding_box_collision_test`, `vector.c` | **host (moved)** |

## What moved

The ordinary hosted module (`EECH_CAMPAIGN_HOST_ENTITY`) no longer compiles the two `accurate_*` tests, the face wrappers or the transform arithmetic. Each now derives the
candidate's pose from the entity, in the original's order, and asks the model callback's new `ENTITY_COLLISION` operation (`mb_hostmodel.h`; the campaign's half of the dialogue is
`mb_hostentity.h`): the segment's two ends, then a stationary candidate's position and attitude (18 floats) or a moving one's sweep ends, attitude and speed (22). **The caller's
intercept point and normal are not sent and not read.** The reply is a hit or a miss, a six-bit write mask naming the output components the original's arithmetic assigned, their
computed words, and the scene's compiled name (an echo). The campaign checks the whole reply and only then writes the assigned components, normal first. A refusal, a malformed answer or a
missing provider stops the engine before anything is written; nothing is invented. There is **no fallback** to the original arithmetic in the hosted module. The explicit
original-characterisation profile (`collision-fixture`) and the model fixtures keep the original branch (compile guard `entity_host_guard`).

The reference host answers with `csrc/entity_step.c` (the original expression for expression; the two intersections are the same text as the tree kernel's apart from one marked
line in each, which records the assignment of the point and changes no arithmetic; a test compares the two copies with those lines removed) driven by `src/entity_host.rs`, which, as
the tree kernel does, requires every floating-point exception masked, runs under round-toward-zero and hands the caller's complete state back. The model data is the *same* final store
the other model operations read: no second loader or cache.

## The scan: the per-candidate decisions (third slice, 5 November 2026; source and Linux checks only)

`get_line_of_sight_collision_entity` in the ordinary module (`EECH_CAMPAIGN_HOST_ENTITY`) no longer decides anything about a candidate. The campaign walks the sector window and each sector list from the tail,
skips the source and the target, and **offers** every entity in the list (`campaign_offer_candidate`, `mb_hostoffer.h`, bound to the real getters in `en_cllsn.c`). The host decides (`ENTITY_CANDIDATE`,
`mb_hostmodel.h`; the reference host is `csrc/entity_candidate_step.c`, the original's expressions): the sector's tallest-structure skip of everything that is not mobile, the branch (mobile first; then the
fixed flag, a city, a building listed directly in a sector, any other), whether a mobile candidate is moving (`|motion|^2 > 1`), a moving candidate's sweep ends, the broad-phase box of the scene's radius + 10 m with
its strict boundaries, a city's height-then-xz box and a stationary fixed candidate's placement. The question carries 44 facts (layout in `mb_hostmodel.h`); the answer is one of seven states.

**The facts are staged, in the order the original needed them** (correction of review finding ENV-ENTITY-CANDIDATE-PRECONDITION-1). The first question carries only what the original read for every candidate:
the segment, the mobile flag, the sector's tallest structure height and, for a mobile candidate, its shape, position and motion vector (a building under a passed city: its shape and position). A candidate that is not
mobile in a sector below the segment is skipped **there**: its fixed flag, shape, position, heading and align flag are never read and no model is looked up, as in the original, and a direct city building (which has
no shape) needs no scene. The host asks for the next stage only when the branch reaches it. The scene's radius is the host's own model data and is looked up only when a branch reaches its broad phase; a scene the
store cannot give then is a refusal by name, never a default radius, a miss or a skip.

| state | meaning | what the campaign does |
|---|---|---|
| `SKIP` | nothing more for this candidate | continues with the next one |
| `NEED_UPDATED` | a moving mobile candidate needs the `UPDATED` answer | performs the destructive getter, once, and asks again with fact 20 |
| `NEED_FIXED` | a non-mobile candidate the sector skip did not remove | reads the fixed flag and, if fixed, the city/city-building type and a city's first building, position and box, or any other candidate's shape and position |
| `NEED_PLACEMENT` | the broad phase passed | reads the attitude of a mobile candidate, or the heading and align flag of a fixed one (a building under a city: the flags too) |
| `NEED_NORMAL` | a terrain-aligned fixed candidate has its placement | asks the terrain for the face normal at its position (the terrain question and its shared-record effects, at the place the original asked it) and asks again |
| `BUILDINGS` | a city's box is met | offers the city's buildings in its list order, each as a building under the city (no second sector skip); the first hit ends the scan |
| `TEST` | the narrow phase is required, with the pose (12 or 16 floats) | sends the existing `ENTITY_COLLISION` question with that pose; a hit returns the candidate, a miss continues, and the partial writes are the narrow phase's |

The question is a pure function of its facts: nothing is kept between questions, so the decision is the one the original made in a single pass, and a candidate that needs something is simply asked again.
Every reply is checked completely before any of it is used (`mb_hostentity.h`, `check_candidate_reply`): a state the facts do not allow (each `NEED_*` is legal once, only for the facts that can need it: `NEED_PLACEMENT` only for a mobile candidate, a building under a city or an ordinary fixed candidate that has a scene, and refused before any getter for a city, a direct city building, an entity that is neither mobile nor fixed or a candidate without a scene; `BUILDINGS`
for a non-city; a `TEST` for a candidate that has no scene), a pose of another size or with a non-finite float (classified from its raw bytes through a pointer: a quiet or signalling NaN is never passed or loaded
as a float before it is refused), a scene name that is not the scene's (its own `entity_candidate.identity` label; the narrow phase keeps `entity_collision.identity`), a city that is given a scene and a refusal
stop the engine by marker; a refusal is never a skip. **No entity identity crosses**: a candidate is named only by the order the campaign offers it, and the host holds no roster or state.

**What the campaign reads, and when,** is exactly what the original read on the branch it took, so no entity is made to supply a shape, a fixed flag, a position, a heading or a model the original did not use.
An entity that is neither mobile nor fixed now costs one question in a sector that does not skip it, where it cost none (the original read its fixed flag there too). The two getters with effects (`UPDATED`, the
terrain normal) are performed on the host's request. The debug drawing of the collision boxes (`draw_collision_boxes`) is a declared unsupported diagnostic of the hosted module and is not carried: the host owns the boxes.

**Evidence (Linux, pinned image; nothing native).** (1) `tests/entity_candidate_host.rs` (10 tests): a contract for each branch and stage, including that a skipped candidate, a direct city building and a city ask
for no radius; and a DIFFERENTIAL of the whole scan, the original's control flow and arithmetic transcribed from the source against the same scan driven by the host's decisions as `campaign_offer_candidate` drives them,
over 400 generated worlds with six segments each (quarter-unit coordinates so boundary equalities occur, and a few entities with no usable scene): both must return the same entity and the SAME TRANSCRIPT of `UPDATED`
reads, terrain questions, narrow-phase poses and refusals, bit for bit, in the same order. (2) The isolated C control (`eech_entity_dialogue_check.c`, 216 expectations) runs the REAL exchange (`mb_hostentity.h`) and
the REAL offer (`mb_hostoffer.h`, the code `en_cllsn.c` runs) over scripted entities that log every getter in order: a direct city building skipped by height is read for its mobile flag only and asked with scene 0;
the same building not skipped reaches its host skip with scene 0; a fixed candidate skipped by height with no usable geometry is read for its mobile flag only; a mobile candidate skipped is not read for its attitude or
its UPDATED flag; the UPDATED getter and the terrain question are performed once, only when named, in the original's order; a city's buildings are read for shape and position in list order; and each illegal state, pose,
name and scene is refused by marker. (3) `model_target_point.rs` runs the same cases through the real provider and a real `ModelStore`: the skips need no model, a candidate that reaches its broad phase on an
unloaded scene or scene 0 is refused, and a loaded scene reaches the placement and the pose. (4) Controls RED before the correction, through these same collaborators: the C exchange stopped a direct city building at
`scene.index` and the provider refused it with "scene 0 is not a scene a candidate can have". (5) Mutants RED after it: the radius asked before the skip, the fixed facts never asked, an eager fixed-flag read, an eager
attitude read and the candidate's identity bound to the narrow phase's label each fail their check (exit 101) and each file was restored byte for byte. The classifier-by-pointer change is a source contract that no
x86-64 run can distinguish from the by-value one, so it is guarded by text and is not claimed as a measurement. The campaign's scan loop (`en_cllsn.c`) is bound to engine getters and is exercised natively only;
the Rust transcription of the scan in the differential is a model of the dialogue, not that code. The compile guard (`entity_host_guard`) shows the ordinary module holds none of the original's decisions, the offer
header holds none either, the original profile keeps all of them, and the identity labels are bound as stated.

## The output contract (review finding ENV-ENTITY-CALLER-BUFFER-1)

The first version of this slice sent the caller's prior intercept point and normal to the host and wrote all six words back. The independent source pass found that wrong: the original
reads neither. In `get_3d_line_triangle_intersection` and `get_3d_line_quad_intersection` (`modules/maths/vector.c`):

| output | assigned when | where |
|---|---|---|
| face normal (x, y, z) | on **every** call, at the top, before the degenerate-face test, the plane test or any decision; each component is then read only after it was assigned | `ni->x`, `ni->y`, `ni->z` |
| intercept point (x, y, z) | only once `mu` is in range, i.e. the segment crossed the face's plane (whether or not the crossing lies inside the face); all three together | `pi->x`, `pi->y`, `pi->z` |

So a miss can have assigned the normal only (a segment that never crossed any plane, or a degenerate face) or both (a crossing outside every face); a hit has assigned both. The real
weapon-step caller (`answer_weapon_step_lookup`, `wn_move.c`) declares `intercept` and `face_normal` as automatic locals, zeroes `intercept` only and passes both on: its normal is
indeterminate storage the original never read. The corrected exchange therefore never loads, copies or forwards either buffer; the host reports which of the six components were assigned and
their computed words; a component that was not assigned keeps exactly the words it held, unread and unwritten (an initialised signed zero, subnormal, infinity or NaN payload stays
bit for bit because nothing touches it); the unassigned words of a reply are zero and anything else is refused. A *computed* zero is an assigned word: a degenerate face assigns a zero
normal and the mask says so. Callers are not initialised globally and no word is normalised.

The write mask domain (checked in the campaign before any commit, `mb_hostentity.h`): 0 to 63; each vector assigned whole (bits 0 to 2, bits 3 to 5); a point implies a normal (the
original assigns the normal first); a hit is 0x3f; unassigned words are zero. Normal-before-point order, partial writes on a miss, first-hit termination, cursor order, refusal atomicity,
the original's arithmetic and the floating-point discipline are unchanged. The kernel records the assignment with one marked line after each helper's point assignment and with the
face wrappers, which set the normal bits on entry; the record is per thread and reset on every entry. This is a semantic fact about what the original wrote, carried as a mask, not a generic
protocol.

## What did NOT move (declared, required, not hidden)

The narrow phase (first slice) and the per-candidate decisions (third slice) have moved. It is still not whole entity/object collision, and nothing below is optional or dormant.

* **The enumeration of candidates stays in Core and is still required to move: the sector window of the segment, the sector lists and their order, the exclusions of the source and target, and the reads of
  each entity's own state.** The concrete missing fact is where the order comes from. The first hit ends the scan and a farther object in an earlier-scanned sector wins over a nearer one, so the result
  depends on (x outer, z inner) sector order and on each list's insertion history (mobiles tend to sit at the head and are visited last). That history is campaign state; the host has no copy of it, no
  entity identity crosses, and a host-held roster would be a second authoritative physical store that must follow every position write, sector relink and release (the campaign mutates positions, the
  `UPDATED` flags and the sector membership continuously). A bounded way to move the enumeration is a host-held roster mirror with a stated synchronisation contract, or a host-driven cursor over the campaign's
  lists; both need the roster's semantic identities and the live pose of every collidable, which are the REQUIRED live-pose/roster work, not guessed here. The window's 150 m widening and the clamp to the map
  edges are arithmetic over the campaign's map constants and have no effect except to prune the enumeration (an entity beyond them cannot pass a radius + 10 m box unless its radius exceeds 140 m: not shown
  here), so they move with the enumeration. **The entity/object collision responsibility is not finished.**
* **The reads of an entity's state.** The facts are copied from the entity by the campaign (flags, shape, position, motion vector, attitude, heading, align flag, a city's box and first building), and the two
  getters that have effects of their own (the destructive `UPDATED` getter, which ANDs the whole flag word, and `get_3d_terrain_face_normal`, which writes the shared terrain point record) are performed by the
  campaign when the host names them. They keep their place and order exactly; they leave only with the roster.
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

## The public authority witness (`lua/entity-authority.lua`; TEST-ONLY; run natively 5 November 2026 on `7ead34d09`, finite stationary authority verified)

The unit and C controls prove the campaign's half and the host kernel. They do not show that the host's answer reaches the campaign through the **public** path. `lua/entity-authority.lua` is the sibling of
`tree-authority.lua`: host-owned Lua → the public native module (a DLL built with `weapon-fixture`) → the host's model provider (`host.model`) → a REAL weapon launched through the campaign's own launch call.
It reuses the tree witness's verified blocks verbatim (the exclusive checked output, the launch-report and terminal grammars, the configured VIEW_WEAPON aim delegate, the frozen cohort, the anchored engine-stop
recognition), a drift test compares its float-words block with the tree witness's, and nothing of the closed TREE witness or any default driver is changed.

**What it observes.** The provider wrapper delegates EVERY operation to `host.model` unchanged (the very table the host gave, the arguments untouched; a scripted check proves it in every mode) and counts them; only
an `entity_collision` question asked AFTER the launch ("armed") may be answered differently, by the chosen mode. For each armed question it records: the request (the key is 18 or 22 `0x%08x` patterns, nothing
else is sent: the corrected shape), the host's real reply (result, mask 0..63 with whole vectors, a point only with a normal, a hit is 63, zero words where the mask assigns nothing, a name) and, from the weapon
fixture, the run-wide counters `contact_any_tests` (the weapon step's entity lookups: the ACTUAL caller was reached), `contact_any_hits` and `precision_high`. Nothing is faked: no call, counter or answer is
made up, and the counters are the fixture's own.

**Branch preconditions, derived from source (no distance guess).** (1) `wn_flight.c` asks the entity lookup only when the weapon is a server weapon with in-flight collision checking and (the step is at high precision
or the weapon has no target); the launch links the case target, so the precision path is needed: the existing VIEW_WEAPON lookup answer is set to 1 by an aim delegate over the unmodified `host.aim` on every armed
`weapon_step`, hosted and Original alike, **configured and global while armed, not a camera and not a per-weapon identity**, with its use measured (`precision_high` must rise). (2) `get_line_of_sight_collision_entity`
excludes only the launcher for a weapon (`los_target` is NULL) and visits the sector window of the segment widened by 150 m; a candidate gets a narrow-phase question only if it is collidable (`INT_TYPE_COLLISION_TEST_MOBILE`
exists for vehicles and aircraft, `..._FIXED` for structures; a PERSON has neither) and the segment's box meets the candidate's box of radius + 10 m. So the case is the **nearest non-person enemy of any launcher
that has an unguided ballistic weapon in stock**, so that the target is the weapon's first broad-phase candidate. If no candidate ever passes the broad phase (still Core, still campaign-owned) the host is asked
nothing, and that is **UNREACHED**, never authority. The wrapper counts questions of the whole war (the provider carries no weapon identity); foreign activity is declared, not attributed.

**Modes (one per process; exit 0 is the verdict).**

| mode | what the provider does after the launch | the verdict needs |
|---|---|---|
| `control` | nothing altered | at least one `entity_collision` question, each request well formed, each real reply well formed, `contact_any_tests` rose, the host's hit count covers the weapon step's `contact_any_hits` |
| `forced_hit` | every question answered HIT, mask 63, point = the request's OLD end, normal up (the real answer is still asked for its name echo and discarded) | the forced hit reached the weapon step's own hit count; the cohort's PHYSICAL signature (type, code, steps, position, velocity, lifetime of each case weapon, never its ordinal) is recorded for comparison with the control |
| `forced_miss` | every question answered MISS, mask 56 (normal assigned, point not) | an accepted partial-write miss; no hit may reach the weapon step. If the control had no hits this is **valid MISS coverage, not a measured changed physical outcome**, and the row says so |
| `withheld_entity` | the provider raises on the first armed question | the engine stops at exactly `[model-provider:entity_collision.callback]` |
| `bad_write_mask` | a HIT answered with mask 56 | the engine stops at exactly `[model-provider:entity_collision.write_mask]` |
| `bad_answer` | a hit answered with mask 64 | the engine stops at exactly `[model-provider:entity_collision.answer]` |

An engine stop counts only as the anchored first line `runtime error: eech: EECH fatal error: host model: [model-provider:<marker>] `; a Lua error that merely mentions a marker, a witness failure, a different marker or a
stop with nothing altered is a failure of the witness, never proof. After a stop nothing is asked of the engine again, and counters that were not read stay unknown (-1). An altered answer is formed only from a
well-formed question and real reply; otherwise the host's own answer is returned and the counted fault fails the verdict (a witness failure raised inside the callback would come back carrying the callback marker the
refusal modes credit).

**The explicit Original profile (`profile=original`, control only).** The module is built with `weapon-fixture` AND `collision-fixture` (the original narrow phase; `EECH_CAMPAIGN_HOST_ENTITY` is not defined). The fixture
API must be PRESENT, the actual entity caller must have been reached (`contact_any_tests` rose, with the identical configured view input) and the host must have been asked NO `entity_collision` question: that is reported as
`CAPABILITY RED` and the process fails on purpose. A module without the fixture API, a run with no entity lookup, or one that asked the host, is `UNREACHED` or "not the original", never a red. The counter cannot show
that the original narrow phase itself ran (it counts the caller's lookup), so the row reports the lookups that hit.

**Controls** (`tests/entity_authority.rs`, 14 tests, the REAL driver in a real Lua state against scripted collaborators; they invent no physics and say nothing about the DLL or the host kernel): a healthy control with
stationary and moving questions and several per step joined with the counters; a real hit checked against the weapon step's hit count; the request shape (18 and 22 patterns, a count, token or extra-argument fault) and a
malformed real reply (the driver's own check with a lenient campaign, and the campaign's own stop); every UNREACHED distinction (no candidate question, no entity lookup, view ignored, no view answer, a short view answer);
the view input (only an armed `weapon_step` with the answer differs, only in `answers[2].f[7]`, a signed zero in an unrelated word survives, the caller's table is untouched); exact delegation of every other operation and
of every question before the launch in every mode; the Original profile's red and each of its non-reds; a forced hit that must change the cohort's signature and must reach the hit count (and fails when it does not); a
forced miss that must not reach it; the three refusal modes at their exact markers, the wrong marker, the text-only mention, three unwrapped stops and a stop with nothing altered; the case selection (a person is not a
candidate, a friend is not an enemy, the smallest distance across launchers, the next pair when the nearest launcher has no weapon, no candidate at all); the cohort and the launch report; and the setup, output and
argument failures with the first error kept.

**Identities.** `lua/entity-authority.lua` Git blob `dcf9c1eae1c7adacc5d33d7d9784a105405665c1` (LF), `tests/entity_authority.rs` blob `ee8df2e3fd8becb85c59e336a6bfec77abcebfe2`. They change no compiled input: the runtime is the corrected source `f153`/`7ead`.

**Native result (Root, immutable corrected build `7ead34d09`, witness blob above, safe record under [`finite-authority/`](../reference/d3-entity-collision/finite-authority/README.md)).** The healthy control and six further
controls ran, 14 full input checks0, 87 payloads each; six native/controller exits0, Original native/controller1 as expected, matrix queue0. Healthy: 279 stationary armed questions (44 real hits, 235 real misses), six actual weapon lookups, one cohort member complete. Forced hit:
45 valid answers change the cohort terminal. Forced miss: 749 valid partial-write answers, no weapon entity hit (the healthy weapon made none either, so this is valid MISS coverage and **not** a measured
physical change). The three refusal modes stop at their exact anchored markers with nothing read afterwards. The Original profile reaches the entity caller with no host question: the expected capability RED.
The same independent reviewer found no findings (`FINITE_STATIONARY_ENTITY_HANDOFF_AUTHORITY_VERIFIED`). **Limits:** every queried pose was stationary, so the MOVING public path is not measured; the view input is
global while armed, with no per-weapon attribution; no new direct x87/MXCSR or RNG measurement of the public caller. The first native command was
`eech-world.exe <literal entity-authority.lua> root=<fresh root> mode=control out=<new .jsonl>`; runtime JSON Lines stay private (retail coordinates).

## The ENTITY_CANDIDATE public authority witness (TEST-ONLY; finite native validation completed 5 November 2026)

The finite stationary witness above shows that the host's `entity_collision` answer reaches the campaign. The candidate decisions (the third slice) need the same proof of their own: that a host's `entity_candidate`
answer reaches the real scan and changes what a real weapon cohort does, and that a withheld or malformed one stops the engine at the candidate's own marker. Nothing of the shipping code changes for it: it is pure Lua
and tests, extending the existing script and reusing its checked output, cohort, WordCodec, exact engine-fatal recognition and controlled VIEW_WEAPON input unchanged.

**Boundary.** Host Lua to the real public module (`weapon-fixture`), the REAL `ENTITY_CANDIDATE` operation, the native scan with its services and narrow phase, and an actual weapon cohort. **Grammar (44 facts, 7 states):**
the request is a scene number >= 0 (0 until a shape was read) and the facts text of exactly 44 comma-separated `0x%08x` patterns, nothing else; a real reply is `{ result, name }` with `result` one of `skip`,
`need_updated`, `need_fixed`, `need_placement`, `need_normal`, `buildings`, `test`, `name` the scene's compiled name (EMPTY for scene 0) and, for `test` only, `p` of 12 or 16 patterns. The wrapper delegates every other
operation, and every candidate question in a mode that does not alter it, byte for byte; it counts the ARMED questions (the war's, global, with no weapon identity), classifies the real answers and the fixture's own counters,
and a count of zero is UNREACHED, never authority. The source-derived case and view preconditions of the stationary witness are kept (global while armed; no per-weapon identity).

| mode | what the provider does after the launch | the verdict needs |
|---|---|---|
| `control` | nothing: the NATURAL healthy control, the first thing Root runs | candidate questions >= 1, well-formed requests and real replies, the old entity checks; no configured secondary input |
| `candidate_real` | PAIRED CONTROL: the real candidate answers; `entity_collision` answered HIT (mask 63 at the request's OLD end, real name) whenever asked | a real `test` decision reached the narrow phase and the configured hit reached the weapon step's hit count; the cohort physically hits |
| `candidate_skip` | every candidate question answered a valid SKIP (the real answer is asked first for its name); the SAME configured hit if `entity_collision` is ever asked | positive candidate questions, every one skipped, no `entity_collision` question, no weapon entity hit; the cohort signature is recorded for comparison with `candidate_real` by the caller |
| `withheld_candidate` | raises on the first armed candidate question | the engine stops at exactly `[model-provider:entity_candidate.callback]`, nothing asked afterwards, unread counters -1 |
| `bad_candidate_state` | the first question that is not a city is answered `buildings` | exactly `[model-provider:entity_candidate.state]` |
| `bad_candidate_identity` | the real state under another scene's name | exactly `[model-provider:entity_candidate.identity]` |

**The configured secondary input is declared, not natural.** Both paired modes hold the existing, already tested forced-hit `entity_collision` answer, so a candidate decision has a physical consequence to change without guessing
any geometry: the real scan must reach the narrow phase (`candidate_real`) and the skip must prevent it and change the cohort (`candidate_skip`). The natural healthy comparison stays separate (`control`): the paired modes are
compared with each other, never with it. This is a diagnostic world variation, not a source or physics change, and no question is fabricated. The Original profile (`weapon-fixture` + `collision-fixture`, `profile=original`,
`mode=control`) must have the fixture API and the real caller lookups (`contact_any_tests` rising) with ZERO candidate and zero narrow-phase questions: the EXPECTED `CAPABILITY RED`, exit 1; its physical signature is compared with
the natural healthy control, not with the paired one. A module without the fixture API, or no lookup, is UNREACHED, never a red.

**Native command (Root; ONE natural healthy control first, on the frozen `1ccc9e2ed` hosted `weapon-fixture` binaries, one fresh root).** `eech-world.exe <literal entity-authority.lua> root=<fresh root> mode=control out=<new, non-existing .jsonl>`
(defaults: warmup 30 s, settle 120 s, frame 100 ms). Exit 0 is a reached, well-formed, complete control. `UNREACHED` (a zero candidate count, a view that never reached a weapon step, no entity lookup) is a result to preserve, not to
retry or loosen, and the case, view or distance is not to be tuned afterwards without actual branch facts. Only after it reached the candidate questions: `candidate_real`, `candidate_skip`, `withheld_candidate`,
`bad_candidate_state` and `bad_candidate_identity` on their own fresh roots, then the Original control on the `weapon-fixture,collision-fixture` build. The literal blob of the script is the identity: it is a new Lua blob, never a relabelled
`b24ca9038` one. Runtime JSON Lines stay private (retail coordinates); safe counts and hashes only.

**Controls** (`tests/entity_authority.rs`, 21 tests now, the REAL driver in a real Lua state against scripted collaborators that apply the shared dialogue's rules: the staged states, the name echo and the legality of each state against
the facts): the natural control with staged and repeated dialogues; zero candidate questions UNREACHED even when the narrow phase was asked; a malformed request and real replies counted; the paired control reaching the narrow phase
and the cohort hitting; the skip variant preventing every narrow-phase question and hit and changing the signature, and failing when a narrow phase is still asked; the three refusals and the withheld answer at their exact candidate
markers (at a candidate with and without a scene), the old marker not credited, a witness failure unable to pose as the host withholding, nothing asked after the fatal; the Original profile's RED naming both counts; and a checked close.
Five mutants of the witness (a skip that answers the real state, an unchecked hit use, an unchecked downstream question, an unaltered identity and the old marker credited) each fail them (exit 101). They invent no physics and say
nothing about the DLL.

**Native finite result.** Seven expected outcomes on frozen1ccc plus literal d39 are independently closed, findings empty, report SHA256 `0e27b680f812de14d9aa837b70fbfbfb634576b96e7382fbcc8904a2374d1456`; [safe evidence](../reference/d3-entity-collision/candidate-finite-authority/README.md). Healthy70470 Candidate/279 stationary narrowphase/6 caller lookups/cohort1complete. Paired real55131/45 with configured hit reaches hit_target after1 step; paired55876 valid skips prevent all narrowphase questions and end hit_land after6. The SAME secondary input is held across the pair; natural healthy remains separate. Three exact Candidate refusals, Original expected native/controller1 with API/caller reached and host questions0, physical cohort equal natural; matrix0,14 full input checks0,88 unchanged payloads each. Ordinary exact short: all3 COMPLETE files literally equal80fd. These are finite authority and short conformance, not wholeTask49/fullpack/D3. Moving-native coverage and every required coupling listed above remain open.

**Literal comment qualification.** The unchanged d39 opening comments that put radius+10/broadphase in Core describe the earlier slice. Runtime1ccc has Host-owned per-candidate decisions. Core still owns the physical enumeration and backing state/getter services, all required to move; no campaign-policy exclusion is inferred. Preserve literal d39 bytes and actuals.

## Checks (Linux, pinned image) and their status

**Review status.** The independent source pass over `c510c55c7` returned CHANGES_REQUIRED with one P1, ENV-ENTITY-CALLER-BUFFER-1 (the question read caller output storage the original never reads). The literal
report is kept in [`independent-entity-narrowphase-source-c510c55c7-20261105.json`](../reference/d3-entity-collision/independent-entity-narrowphase-source-c510c55c7-20261105.json) (SHA-256
`18dba44b15530097e3a04e37ac7e7d062dcd39569f86e659511ff50679111251`); it is not an approval. The correction is in "The output contract" above. The SAME reviewer cleared the corrected source `f153`/`7ead` with no findings (report
[`independent-entity-output-contract-source-7ead34d09-20261105.json`](../reference/d3-entity-collision/independent-entity-output-contract-source-7ead34d09-20261105.json), SHA-256 `1ab8b6a2d1295ed47a189b18af9b35a461876e4e65959f2a30db9112157ce039`), and the finite stationary native authority is closed (previous section).

**What each check covers.** `eech-engine-sys` `entity_dialogue` (`entity-dialogue-check`, the REAL `mb_hostentity.h` against a scripted host): the request holds only the segment and the pose, bit for bit,
and is identical whatever the caller's buffers hold; with the intercept point and normal on pages that can neither be read nor written, a miss that assigned nothing completes (a read or a write would fault);
a hit, a miss that crossed a plane outside the face (both vectors) and a miss that never crossed (normal only, the point's page unreadable, and on a second run its initialised signed-zero, subnormal and
signalling-NaN words stay bit for bit) write exactly the assigned components with their computed words, including a computed signed zero and NaN payloads; a degenerate face's computed zero normal is an
assigned word; and twenty-one refusals (the host's refusal, scene 0 and out of range, a candidate of 13 or 0 floats, another scene's name, results 0 and absent, masks 64 and -1, a point or a normal in part, a
point without a normal, a hit that assigned one vector or none, a word for an unassigned component including a negative zero) each stop before a commit and leave BOTH initialised buffers byte for byte
unchanged. `eech-world` `entity_collision_host` (9, plus the output contract asserted on every answer): hit assigns both; a crossing outside the face assigns both and is a miss; a segment that never
crosses assigns only the normal (the point words are zero); a degenerate face assigns a zero normal; ascending cursor and first hit; bounding-box fallback and attitude; the three sweep tiers with strict
boundaries; a refusal is never a miss; rounding independence and state hand-back; the intersection text equals the tree kernel's apart from the two marked assignment lines. `eech-engine-sys`
`entity_host_guard` (3) and `tree_host_guard` (2): the hosted branch has no solver and the macro is defined only where the host model exists. `eech-engine` and `eech-dc`: pose sizes 18 and 22 only, the mask
and computed words cross as given, a mask outside 0 to 63, non-integral, negative or not a number, a missing mask, a short list, an unnamed or another kind of answer and a refusal stop with their marker.

**Red controls (not counted as green).** The isolated C control was run against three deliberately mutated adapters, each of which must fail and did (exit 101): one that reads the caller's point buffer
(SIGSEGV on the unreadable page), one that writes the normal before the reply is checked (SIGSEGV on the unwritable page), and one that commits the assigned normal before checking the reply (FAIL lines: each refused
answer had changed the caller's normal). The mutation was applied to `mb_hostentity.h` on the working tree and reverted; its blob id was verified equal afterwards. The old native path was not run to manufacture a red.

**Direct-exit gate** `D:/EECH-Migration-Evidence/entity-collision-20261105-c` (safe projection [`entity-output-contract-direct-gate-20261105.json`](../reference/d3-entity-collision/entity-output-contract-direct-gate-20261105.json)):
seventeen steps, each a direct child whose exit was read on the next line, one combined log, excerpts taken after the status, all 0: `cargo fmt --all -- --check`; the four-crate tests with `--no-fail-fast` (63 result
rows ok, no failed or panicked test); `model_walk`; `tree_dialogue`; `entity_dialogue`; `eech-engine-sys` builds for `collision-fixture`, `weapon-fixture`, `model-fixture`, `model-host-fixture` and
`weapon-fixture,collision-fixture` (so both the original and the hosted branch compile); strict clippy for the four crates, for `eech-dc` with `collision-fixture`, `weapon-fixture` and both, and for
`eech-engine-sys` with `tree-dialogue-check` and with `entity-dialogue-check`. A build whose artefacts were already fresh reports `Finished` from Cargo's own fingerprint check, a direct exit and not a recompile.

**Source identity.** Taken outside the container on the Windows checkout where Git resolves, new inputs included (2911 rows), before and after the gate, equal. Two columns, never conflated: the *filtered* column is the
Git blob id of each working file as Git's clean filter would store it; the *raw* column is the SHA-256 of the working file's raw bytes. Three pre-existing CRLF inputs (`aphavoc/laa.c`,
`aphavoc/source/entity/special/effect/explosn/xp_types.h`, `aphavoc/source/entity/special/effect/smokelst/sl_types.h`) have a filtered id (core.autocrlf=input) that is the LF-normalised form and is **not** their
committed blob, while their raw working bytes equal their committed bytes (CR counts 64, 66 and 119 in both; [`filter-qualification-20261105.txt`](../reference/d3-entity-collision/filter-qualification-20261105.txt)).
So "every listed id is a committed-byte hash" is not claimed; the raw column is the byte identity, and no accepted C file was normalised or rewritten. After the commit the Git blobs of the changed and new inputs bind the source.

These are contract guards, not an oracle for retail models. The native comparison has since run on `7ead34d09`: the ordinary exact short is COMPLETE and literal-equal to the retained `80fd` baseline and the
changed-answer witness (forced hit, forced miss, refusals, Original control) is verified for stationary candidates (see the previous sections). The **whole Task 49 regression pack** (accepted 40, all 8, current CI, review, normal merge) has NOT run for this candidate. The closed TREE adapter (`mb_hosttree.h`) copies the caller's prior tree buffers as bytes
into its question; the reviewer closed TREE at its own identity and raised nothing there, so it is not touched, but it is the same shape of read and is named here so it is not rediscovered.
