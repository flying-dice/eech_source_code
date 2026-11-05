# D3 entity/object collision: the host-owned narrow phase (EEDCS-49, second slice)

3 November 2026, status refreshed 5 November 2026 (the enumeration moved the same day, see "The enumeration"); the third slice (the per-candidate decisions, see "The scan") was added the same day as SOURCE and Linux checks only. **The narrow phase is native-validated for the FINITE STATIONARY handoff only (corrected source `f153`/`7ead`; [`finite-authority/`](../reference/d3-entity-collision/finite-authority/README.md)); the moving public path, the physical scan and Task 49 are NOT accepted.** The body below is the dated 3 to 5 November account; its statements about runs are updated in place where they have since changed. This is the
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
| sector window, sector lists, tail-first order, the source/target exclusion, the reads of each entity's state, in stages and only as the host asks | campaign entities (the sector lists are Core's transitional backing store that the host's SCAN decisions are served from, required to move; this slice changes none of those decisions) | **campaign, declared transitional coupling** |
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

## The enumeration: the host drives the cursor (fourth slice, 5 November 2026; source cleared for planned native validation; Linux controls only so far)

Before this slice the campaign still decided WHICH candidates were visited and in what order. `get_line_of_sight_collision_entity` now hands the enumeration to the host (`ENTITY_SCAN`, `mb_hostmodel.h`; the reference host is
`csrc/entity_scan_step.c`, the original's expressions): the segment's min and max, the 150 m widening and the clamp to the map's extents as sector indices, the traversal (x outer, z inner), that each sector's list is read from its
TAIL, that the source and the target are skipped, that an entry is offered for its decisions (`ENTITY_CANDIDATE`, unchanged) and that the FIRST hit ends the scan. The campaign's driver is `mb_hostscan.h`, bound to the real sector
lists in `en_cllsn.c`; the original window and loops stay byte for byte under `#else` for the `collision-fixture` profile.

**Why a cursor and not a roster.** The first hit ends the scan, so a farther entry in an earlier sector or list position beats a nearer one: the order IS the result, and the order is the heap's insertion and relink history.
A host-held roster would be a second physical store that must follow every position write, sector relink, flag change and release, with entity identities that outlive a scan; none of that exists, and it could not be verified
without running it. A cursor needs none of it: the campaign keeps the lists and SERVES the host's cursor, so the order is the heap's own by construction and nothing is copied. The host is stateless (the cursor travels in the
question and the answer, like the candidate stages), a list cannot change inside one scan (nothing in it links, unlinks or releases an entity; the two effectful getters change a flag word and the shared terrain record, never a
list), and an entry is named only by the host's cursor: no pointer, heap index or legacy layout crosses.

| host state | meaning | what the campaign does |
|---|---|---|
| `FIRST` + sector | serve the TAIL of that sector's list | reads the sector, its tallest structure height, its first child and the walk to the end (the original's reads, for a sector the host names only), reports whether an entry exists and whether it is the source or the target |
| `NEXT` | serve the predecessor of the current entry | reads the predecessor and reports the same |
| `OFFER` | offer the current entry | runs the candidate dialogue (`mb_hostoffer.h`) and reports whether it hit |
| `DONE_NONE` / `DONE_HIT` | the scan ends | returns no entity, or the entity offered last (a hit inside a city is the building) |

The exchange checks every answer before using it (`mb_hostentity.h`, `check_scan_reply`): the state must be the one the phase allows (an excluded entry is skipped and any other offered; a miss goes on and a hit ends the scan;
the end of a list goes to the next sector or ends the scan), a cursor is two whole numbers inside 0 to 16777216 classified from their raw words (a signalling NaN is refused before any float is loaded) and the campaign refuses a
sector outside its map before touching a list; a scan that does not end within 4 000 000 questions stops as `entity_scan.runaway`. Markers: `entity_scan.state`, `.cursor`, `.runaway`, and the provider's `.refused` / `.answer` /
`.callback`.

**The transitional Core backing dependency, REQUIRED to move and named so it cannot be mistaken for ownership.** The sector lists, their order, the entities' live state and the comparison with the source and the target remain the
campaign's entity heap; this slice moves the DECISIONS about the enumeration, not the store. It must not be called an independent host authority or a mirrored physical store. What remains for a roster: stable semantic host
identities for every collidable that outlive a scan, a synchronisation contract for position writes, sector relinks and releases, and the live pose without the per-candidate facts. City children are still enumerated by the campaign
inside the candidate dialogue (`BUILDINGS`), in the city's list order. The DCS side is unchanged and unresolved: a DCS host searches its own world and has no use for the campaign's sector lists, but then needs the hit named by a
host identity the campaign can resolve; no primary-evidenced API gives the first hit's point and normal.

**Cost.** Each candidate now costs at least two scan questions besides its candidate dialogue, and each named sector one more (an empty sector costs `FIRST` and the end of its list). Timing on the ordinary short is an
observation to take, not a claim.

**The moving public path.** A moving candidate is a mobile one whose squared speed exceeds one, and the host asks for its `UPDATED` answer (`NEED_UPDATED`). The supported callers reach it: the weapon step's collision lookup
(`wn_move.c`) and the hosted line of sight (`mb_hostlos.c`) call the same function. It is NOT measured natively: in the finite Candidate evidence the armed windows had mobile candidates (198 to 462) and ZERO `NEED_UPDATED` and
zero moving questions, so every mobile candidate seen was stationary. The concrete native prerequisite, left open: a witness run whose evidence row shows `candidate_need_updated` greater than zero within the armed window
(the existing `warmup` argument is the only knob, to be chosen from observed counts of moving group members near the launcher, never tuned after an UNREACHED); stationary evidence does not cover it.

**Evidence (Linux, pinned image; no Windows or retail campaign ran, but the Linux C controls executed the compiled C).** CHARACTERISATION of the replaced behaviour on the clean source `4b3ec8470` (the candidate differential, 10 tests, and the C control, both green) before any change.
Then: the new differential compares the ORIGINAL scan over a 4 x 4 map (window arithmetic, x outer z inner, tail-first lists, exclusions, first hit, services, poses and refusals, transcribed from the source) with the host-driven
scan (400 worlds x 6 segments, segments of every size and at the map's edges, exclusions that name real entries): same entity and the SAME TRANSCRIPT. Kernel contracts (window, clamp, order, phases, refusals), the C control that
runs the REAL driver and the REAL exchange over scripted sector lists (positive paths first: tail-first order, first hit, an unnamed sector never read, exclusion facts, an empty list; then each illegal state, cursor and runaway by
marker), the bridge, the engine callback, and the real provider. Mutants of the window widening, the map clamp, the first-hit end, the exclusion, the traversal order, the list direction and the exclusion fact each fail their check. The
campaign's driver against the real heap getters is exercised natively only.

**Qualifications recorded by the independent source review (`independent-entity-enumeration-source-40ef-runtime8b852-20261105.json`, SHA-256 `1a856a53300c216d8c462c2da6fad26494f057d380cc7ab328dcaad70d4b470d`, SOURCE_CLEAR_FOR_PLANNED_NATIVE_VALIDATION, findings empty).** (1) The preamble of the hosted
branch in `en_cllsn.c` that still says the enumeration window, order and exclusions remain Core is HISTORICAL and stale for this slice: the decision kernel is the host's, and only the backing lists, the live state and the identity equality
remain Core transitional dependencies. The frozen compiled source `8b85230826ad1c1bea289f6d2592f31b705a03f6` is unchanged by this note; the comment can be corrected with the next production C change. (2) The differential is a Rust TRANSCRIPTION of the original scan compared with the
host decision functions and a model of the service: it is not an execution of the legacy `en_cllsn.c` loop, and its generator and thresholds differ from the old synthetic world set. (3) The 4 000 000-question limit bounds the host's questions, not
the campaign's own walk to the tail of a list.

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

* **The enumeration's DECISIONS moved (fourth slice, see "The enumeration"); its BACKING STORE did not, and is still required to move.** The sector lists, their insertion and relink history, the entities' live state and the
  source/target comparison are the campaign's heap, served to the host's cursor. A host-held roster (semantic identities that outlive a scan, a synchronisation contract for position writes, relinks and releases) is the
  REQUIRED remaining work; neither the cursor nor a future mirror may be called independent authority while the heap is the live state. City children are still enumerated by the campaign in the city's list order.
  **The entity/object collision responsibility is not finished.**
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

## The ENTITY_SCAN public authority witness (TEST-ONLY; finite native validation completed 5 November 2026)

The enumeration cutover (source `8b8523082`) is cleared for planned native validation but has no native actual. This witness extends the existing script (its checked output, cohort, WordCodec, exact engine-fatal recognition and VIEW_WEAPON input are
unchanged) and changes no shipping input: Root pins its literal blob and reuses the frozen `8b8523082` binaries.

**Grammar (16 facts, 5 states).** The request is scene `0` (a number), no depth, kind or definition, and the text place of exactly 16 `0x%08x` patterns (the segment, the sector side and world map, the PHASE 0 begin / 1 entry served / 2 list exhausted / 3
offer resolved, the cursor the host last gave, whether the served entry is the source or the target, whether the offered entry hit). A real reply is `{ result }` with `result` one of `first`, `next`, `offer`, `done_none`, `done_hit`; `first` alone carries `p`
(two patterns, the sector), the others carry no `p`, and the reference host's empty name is accepted while a non-empty one is a fault. The wrapper counts the ARMED questions by phase and classifies the real answers (global to the war, no weapon identity),
delegates the operation byte for byte in every mode that does not alter it, and treats zero scan questions as UNREACHED, never authority.

| mode | what the provider does after the launch | the verdict needs |
|---|---|---|
| `control` | nothing: the NATURAL healthy control, Root's first run | positive scan, candidate and narrow-phase chain (an offered entry, a candidate dialogue, a narrow-phase question), well-formed requests and real replies, a complete cohort; no configured secondary input |
| `scan_real` | PAIRED CONTROL: the real enumeration; `entity_collision` answered HIT (the existing forced-hit answer) whenever asked | the real scan offered entries, the candidate dialogue and the narrow phase followed, and the configured hit reached the weapon step's hit count; the cohort physically hits |
| `scan_none` | every armed BEGIN question answered a valid `done_none`; the SAME configured hit if `entity_collision` is ever asked | positive scan questions, every one a begin answered empty, NO candidate question, NO narrow-phase question, NO entity hit; the caller compares the cohort signature with `scan_real`, never with the natural control |
| `withheld_scan` | raises on the first armed scan question | exactly `[model-provider:entity_scan.callback]`, nothing asked afterwards, unread counters -1 |
| `bad_scan_state` | the first BEGIN question is answered `next` (illegal at the start) | exactly `[model-provider:entity_scan.state]` |
| `bad_scan_cursor` | the first real `first` is answered with a fractional x (0.5) and the real z | exactly `[model-provider:entity_scan.cursor]` (the finite fraction passes the bridge and is refused by the campaign's own cursor check, not by an earlier marker) |

The configured secondary input is declared, not natural, and identical in both paired modes: it gives a scan decision a physical consequence to change without any guessed geometry. The natural control is separate. Every alteration is formed from a well-formed
question and real reply (an eligible one: the begin, or the real first); a malformed real reply is counted and returned unaltered, so a witness failure cannot pose as an engine refusal. The Original profile (`weapon-fixture` + `collision-fixture`, `mode=control`) must
have the fixture API and the real caller lookups with ZERO scan, candidate and narrow-phase questions: the EXPECTED `CAPABILITY RED` (exit 1), its signature compared with the natural control. A missing API or a caller that never ran is UNREACHED.

**What the controls caught while being written.** The first version of the cursor alteration called a helper declared later in the file, so the witness itself raised inside the provider callback and the engine reported it as a callback failure at `entity_scan.callback`: the exact
masquerade the controls guard against. The mode's own marker check refused it (it stopped at the callback marker, not at the cursor marker); the alteration now uses no later-declared helper.

**Limits.** Every question is global to the war and not attributed to the case weapon; the view input is configured and global while armed. The controls are scripted collaborators that apply the exchange's legality rules: they prove the driver, not the DLL, the heap getters or the kernel.
The moving public path stays unmeasured (the finite counts have mobile candidates and zero `NEED_UPDATED`; no warmup is tuned after an UNREACHED). Native command (Root, ONE natural control first, fresh root, the literal blob):
`eech-world.exe <literal entity-authority.lua> root=<fresh root> mode=control out=<new .jsonl>`; the other modes only after it reached.

**SCAN native finite result.** Independently CLOSED, findings empty, report SHA256 `5cab070d2bd2b574aca46d8389e8f7327f1b3d1ac989c9b9243c10fb75e2880b`; [safe evidence](../reference/d3-entity-collision/scan-finite-authority/README.md). Frozen8b852/literalff608:7expectedcontrols/14fullinputs0/88payloads; natural70375SCAN/70470Candidate/279stationary/6caller/cohort1; pairsameconfiguredsecondaryhit changes actualcohort hit_target1step versus deliveredNone114/0Candidate/0narrowphase/hit_land6steps. Three exact SCAN refusals/no postfatalreads; Original expectedRED1/nohostquestions/equalNATURAL. Matrix0; all60commonoldnaturalfields equal1ccc/d39. SCANstate counters describe REALdelegated answers; deliveredNone has its separate counter. Ordinaryshort all3COMPLETEequal80fd;14.679s vs9.576s singleobservation/notbenchmark. Globalcounts/view, no movingpublic or wholeTask49/D3 claim. Allbacking/roster/pose/effectfulgetters/citychildren andother requiredscope remain open; nextactualphysicalpose authority usesONEexistingMovers onlyafter coverage/lifecycle characterization.

## ENTITY_POSE: the characterisation probe before a mobile candidate's pose is taken from the host (fifth slice; 5 November 2026; Linux-checked, NOT yet run natively; corrected after the independent observer review of `3ad06e953`)

> SUPERSEDED IN ROLE by the sixth slice below: the observer ran natively (`1f9a939bc`), and ENTITY_POSE is now the staged conformance assertion (a position, attitude or motion-vector difference stops the engine by `entity_pose.mirror`); the host's record decides the candidate's pose. The text below is the observer's own record and is kept at its identity.

**Why a probe and not the cutover.** The next required extraction is the host-sourced live pose of a rostered collidable mobile, from the ONE `Movers` registry the movement callbacks already step (no second pose cache in `ModelStore`). The cutover cannot be made honestly
until the collaborator contract is known, and four facts of it are not decidable from source: which mobile offers have a host identity at all, whether every identity that is offered has a record, whether the campaign's mirror is bit for bit the record at an offer, and whether a vehicle's
motion vector (which the narrow phase consumes) is the record's. This slice asks exactly those, observationally, on the real campaign. **It changes no decision, no physical read the original made and no order**: the observation is asked beside the original's reads, nothing waits for it, nothing is skipped
for it, and no entity pointer, index or heap layout crosses. **It does add three read-only metadata reads** the original did not make, per observation: the entity's TYPE (a direct field), its ALIVE flag (the mobile's own flag) and its roster HOST IDENTITY (a lookup in the roster's mapping, which stops the engine with the roster's own named
assertion only if that mapping is already inconsistent, as every movement setter's lookup would). The words (position, motion vector, attitude) are copied from the facts the original's scan already read and are no new read. A transcript of the original's PHYSICAL getters (the `reads` log of the isolated control) does not contain the three
metadata reads and is not a proof of all getters; the control counts them separately and asserts them (once each per observation, none for any candidate that is not observed). **Eligible:** a candidate of a sector list whose collision-test-mobile flag is set (an aircraft or a vehicle). **Not observed:** every non-mobile candidate, a city, and every building
listed under a city, including one whose collision flags read as mobile during its placement.

**What the source audit found** (read, not run; the probe measures what it cannot decide):
- *Identities.* A mobile has a host identity only once `campaign_roster_link_member` bound it (`gp_roster.c`, `bind_member`): group members. `current_member_record` is then the only source of `campaign_roster_host_id`. An entity that is created and never linked has none, and a member the host issued an identity to without a creation request is
  `unrecorded` (`Movers::issue`): both are possible among collidable mobiles (every aircraft and every vehicle says "collision-test mobile"). Which of them are offered, and as what, is the probe's first answer. The roster's identity domain is 1 to 128 bytes of UTF-8 text (`CAMPAIGN_HOST_ID_MAX`); the request carries the whole domain in its own field.
- *Records.* `Movers` keeps a record (pose, kind, an aircraft's retained motion vector, a wreck's `vehicle_vector`) from `bind` until `retire`; `Place` is the campaign's explicit write that the host ADOPTS (every setter of a hosted mobile's position, attitude or an aircraft's motion vector calls `campaign_movement_place`), and a step compares the campaign's copy
  bit for bit with the record and fails on a difference. The record therefore equals the campaign's mirror whenever that discipline held; a mobile whose pose the campaign wrote outside it (creation puts every new mobile at the map's middle before its placement) is what an offer would show as a position or attitude difference.
- *Vehicle motion vector.* A routed vehicle's is computed in `rv_hostmv.c` (`campaign_movement_move`) as the post-step attitude row 2 times the post-step speed, in the campaign, "for the view system", which collision also reads; a ship's is the host's result; the record holds no vector for a live vehicle (`motion_vector` is `None` for every kind but an aircraft, `vehicle_vector` is a wreck's). The probe says, per offer, whether the
  campaign's vector is the product of the record's attitude row 2 and its speed (see the formula below), whether the host holds a vector at all, and whether it matches the held one.
- *Not inspected by this audit, and not characterised by the probe unless the run reaches it:* the pack/unpack routes (`hc_pack.c`, `fw_pack.c`), the gunship dynamics writers of the player's aircraft and dependent carry. A healthy control characterises ONLY the offers and the runtime paths it actually reaches. A histogram with no position or attitude difference says nothing about a writer the run never
  reached, and a difference says THAT a mirror differed, never WHICH writer made it. The disposition of every unreached or unattributed writer remains required before any cutover; no universal coverage is inferred from the absence of a difference.

**The observation.** `ENTITY_POSE` (`mb_hostmodel.h`; shared exchange `mb_hostentity.h`; asked from `mb_hostoffer.h`, the code `en_cllsn.c` runs): for an eligible candidate the offer asks once with the position and motion vector the original read for every mobile candidate (STAGE 0, before the first candidate question) and, when the broad phase passed and the original read its
attitude, once more with the attitude (STAGE 1, beside `NEED_PLACEMENT`). The question carries `scene` 0, `depth` the stage, `kind` the entity's type, `definition` whether it is alive, the 15 words as the bytes the entity held (a signalling NaN stays a signalling NaN: the host reads them as bit patterns, never as floats), and the identity in its own
request field (up to 128 bytes; nothing is shortened or skipped), which the Lua provider receives after one space following the words. The reply is `{ result, kind, difference }`: `result` is `recorded`, `unrecorded`, `retired`, `unknown` (never issued, or no movers at all) or `no_id` (the question named none), `kind` the record's kind (1 routed vehicle, 2 ship, 3 person,
4 fixed-wing, 5 helicopter; 0 otherwise) and `difference` the bits 1 position, 2 attitude, 4 motion against a held vector, 8 no vector held, 16 motion formula (below). **The shared C checker validates the reply completely, including the stage's own masks:** a class outside the five, `no_id` exactly when no identity was named, a kind outside 0..5 or a mask outside 0..31, a class that is not a record
carrying a kind or a difference, a record of no kind, **an attitude difference (bit 2) at the offer stage (no attitude is read there), and bits 4 and 8 together (a difference from a held vector and none held)**; each stops by marker (`entity_pose.state`, `.answer`, `.callback`, `.request`). It never decides from the content.

**The formula, and why no floating-point instruction runs.** Bit 16 says the campaign's motion vector is not bit for bit the product, rounded toward zero, of the record's attitude row 2 and the record's speed (the original's vehicle formula, from the HOST's record); it is also set when an operand is not finite, where no product is defined. The observer runs in a model callback that has no floating-point protection of its own, and an
ordinary float multiplication would raise sticky flags the Core can see, could trap under unmasked exceptions and would round according to the caller's mode. It therefore computes the product in INTEGERS (`product_toward_zero`, `movement.rs`): significands multiplied exactly in 48 bits and truncated (overflow gives the largest finite value, underflow the truncated subnormal or zero). The reference profile's round-toward-zero is the mode the
campaign's own product runs under. **What the product is, exactly:** the reference product, IEEE gradual-underflow round toward zero (denormals-are-zero and flush-to-zero both OFF, which is the reference profile's own setting and which the host's profile selection preserves in the caller's word); it is NOT claimed to equal a product computed under a caller's DAZ or FTZ mode, where the processor's result for a subnormal operand or result differs. The integer product is verified bit for bit against the processor's own product under an EXPLICIT profile the test sets and restores
(round toward zero, every exception masked, no sticky flag, DAZ=FTZ=0) over about 3 million operand pairs of every class (zeros, subnormals, overflow, underflow, every sign), and the whole observation is proved to leave the SSE control and status word exactly as found for every rounding mode, the eight seeded sticky-flag patterns (zero, each of the six individual flags, and all six set) and with all exceptions masked and unmasked.

**Layers and checks (Linux, direct child exits).** The isolated C control (`eech_entity_dialogue_check.c`, 356 expectations of which 41 are new) runs the REAL offer and exchange over scripted entities with the scripted pose host kept apart from the candidate queue, so every earlier expectation is unchanged: the words, stage, type, alive flag and identity of each observation, the three metadata reads counted, that a fixed candidate, a city and a city's building are never observed, that no
valid answer (unknown, retired, unrecorded, a record that differs) changes a decision, read, count or the hit, each shape violation including the stage-specific masks at both stages, and the identity domain (1, 127, 128 and 128 multi-byte bytes carried whole; 129 refused by marker). The bridge crossing (`eech-engine`, a 128-byte and a multi-byte identity included), the Lua marshalling (`eech-dc`), the real `Movers::observe` over a real registry bound, stepped,
placed and retired through its own API, the integer product and the floating-point preservation (`movement.rs`), and the real provider over that registry (`tests/entity_pose_host.rs`: 128-byte identities, a signalling NaN word, a registry being stepped refused by name and never read through a second borrow) are each tested with their own malformed cases. The corrections of the review are covered by focused mutants recorded in the gate record. The public witness
(`lua/entity-authority.lua`, no new script) validates the same stage masks and the 128-byte domain.

**The public witness and what one healthy run classifies together.**

| mode | what the provider does | the verdict needs |
|---|---|---|
| `control` | nothing: the NATURAL control | every earlier requirement, and something OBSERVED (a mobile offer, before or after the launch; zero is UNREACHED for the mobile contract), well-formed requests and replies; the facts are OBSERVED, never asserted |
| `withheld_pose` | raises on the first armed observation | exactly `entity_pose.callback`, nothing asked afterwards, unread counters -1 |
| `bad_pose_answer` | the first armed observation is answered kind 6 | exactly `entity_pose.answer` |
| `bad_pose_state` | the first armed observation that named an identity is answered `no_id` | exactly `entity_pose.state` |

Every observation is classified BEFORE the launch (boot, warmup, the natural weapons until then: `pose_pre_*`, never altered) and after it (`pose_*`) into the histograms of (stage, entity type, alive, class, kind, difference); the per-stage buckets are the truthful counts of observation QUESTIONS. The `pose_fact_*` fields sum both phases into the facts the cutover needs, in ONE healthy retail run, with these
**denominators**: an OFFER is one stage-0 observation (the scan offered a mobile candidate to the host once; the same entity offered again by a later scan counts again, so offers are neither unique mobiles nor unique records); every offer-level fact counts STAGE-0 observations only; the attitude facts count STAGE-1 observations only (a mobile whose broad phase passed is observed a second time, with its attitude, and that second observation repeats the position and motion, which are therefore not counted again);
`pose_fact_observations` counts every observation question of both stages. The facts: `offers`, `no_id`, `unrecorded`, `retired`, `unknown`, `recorded` (identity and record coverage);
`dead` and `dead_recorded` (wrecks and destroyed entities are alive 0); `position_differs`, `attitude_compared`, `attitude_differs` (the mirror against the record); `motion_held`, `motion_held_differs`, `motion_not_held` (what the host holds); `formula_equal`, `formula_differs`, `vehicle_formula_equal`, `vehicle_formula_differs`, `aircraft_motion_differs` (the vehicle vector); `recorded_by_kind`. `scan_none` additionally requires no observation; the Original
profile requires none. Counts are global to the war and are not attributed to the case weapon; a count of mobile offers is not a measure of moving candidates (`candidate_need_updated` is that counter). Which writer produced a difference, and writers the run does not reach, are NOT answered.

**What Root runs.** The compiled C and Rust changed, so the Windows retail module and `eech-world.exe` are rebuilt from this source: the ORDINARY shipping profile and the HOSTED profile only (no Original native build or refusal matrix before the one healthy contract probe, per the independent reviewer). (1) The ordinary short on the ordinary shipping profile, comparing its three complete output files (Tacview, observations, campaign metrics) with the preserved `80fd` files, to show the
observation changed nothing; the extra decision-neutral question per mobile offer costs wall time: record it, it is not a benchmark. (2) ONE healthy natural `control` with the literal Lua on a fresh root: `eech-world.exe <literal entity-authority.lua> root=<fresh root> mode=control out=<new .jsonl>`. The refusal modes only after it reached. Reading it: `no_id`, `unrecorded`, `retired` or `unknown` rows are candidates the cutover must give a characterised explicit contract
(not a fallback to Core inside the query); a `recorded` row whose difference has bit 1 or 2 is a mirror that was not the record (the writer is not named); for a routed vehicle, difference 8 alone means the campaign's vector is the formula and the host holds none (the cutover must put the solver's vector in the record), while bit 16 means the formula is not what the campaign holds. A zero count is UNREACHED and is not tuned away. The physical-authority cutover stays open until the
returned characterisation, and moving public parity stays unmeasured.

## The host's physical record decides a mobile candidate's pose (sixth slice; 5 November 2026; Linux-checked, UNREVIEWED, NOT yet run natively)

**What the native characterisation of the observer (`1f9a939bc`) showed, and what it did not.** On the ordinary supported fresh runtime every observed mobile offer (9252 observation questions: 6091 offers and 3161 attitude questions) was a recorded identity of kind 1 (the host's routed kind), alive, entity type 1 (`ENTITY_TYPE_ANTI_AIRCRAFT`) or 23 (`ENTITY_TYPE_ROUTED_VEHICLE`); no
offer lacked an identity or a record, no position or attitude differed, and the record held NO motion vector on any offer: the campaign's vector equalled attitude row 2 times the speed on 3727 offers and differed on 2364 (all 2334 anti-aircraft offers, 30 routed). That is why the vector cannot be recomputed from the speed and the attitude: a stationary
anti-aircraft gun's vector is the +0 it was created with, where the legacy expression gives -0 for a negative attitude element. It is not licence to default or zero a vector, and it says nothing about aircraft, ships, persons, wrecks or any unreached state. No offer was moving (`moving` and `candidate_need_updated` were 0).

**Where the legacy vector is written** (read from source; the supported hosted runtime): (1) the routed move's end (`rv_hostmv.c`, `campaign_movement_move`) and the two early returns of `routed_vehicle_movement` (`rv_move.c`: no waypoint, not in a state to move), each evaluating `attitude[2][k] * velocity`; (2) the ship's hosted move, whose vector is the host's own result;
(3) the vehicle lifecycle (wreck, fall, impact, sink), whose vector the host already holds as a claim (`vehicle_vector`); (4) nowhere for an anti-aircraft gun that is alive or a person (in the hosted build `ps_move.c`'s vector write is in the non-hosted branch), so their vector stays what it was created with. A mobile's vector is seeded by `campaign_roster_link_member` for aircraft only before this slice.

**The cutover.** The host's `Movers` record holds EVERY kind's canonical motion vector: (a) it is SEEDED at the bind, explicitly, from the campaign's vector at that moment (`descriptor.motion_vector` now for every hosted kind; a missing or non-finite seed is refused, never defaulted); (b) a PLACEMENT adopts the campaign's vector for every kind
(`campaign_movement_place` sends it), as it already did for aircraft; (c) the routed SOLVER evaluates the legacy expression with the same single-precision products and retains the result (`Advance`), and the new `ROUTED_VECTOR` request does the same at the two early returns, where the legacy code evaluated it without a step (a hosted vehicle only; an unhosted one is derived locally exactly as before);
the engine WRITES the host's answer instead of its own product; (d) the ship's solver result is retained; (e) the lifecycle's claim IS the canonical vector (holding one holds the other). A stop leaves the vector as the legacy code left it (stale) until it is derived again. **The lifecycle claim still ADOPTS the campaign's vector the first time it is made (the existing explicit claim contract); that moment is
Core-origin and REQUIRED to retire.** Nothing is derived at query time and no copy is made in `ModelStore`.

**The decision.** An `ENTITY_CANDIDATE` question of a MOBILE candidate of a sector list (never a building listed under a city) carries the candidate's semantic host identity after the 44 facts (one space, then 1 to 128 bytes). The provider reads the SAME `Movers` registry the movement callbacks step (`try_borrow`: a registry being stepped is refused by name, never read through a second borrow, never a partial record)
and takes the candidate's position, motion vector and attitude from the canonical record, at the first question (the position and vector the original read for every mobile candidate) and, for the attitude, only once the placement facts were read (no attitude is required early). The campaign's words for them are still read and sent (the original read them) and are compared with the record by ENTITY_POSE, but they decide nothing:
a RED/GREEN test at the real provider boundary gives words far outside the segment's box and a different attitude and vector, and the decision and the narrow phase's pose are exactly those the real decision gives from the record's own words, bit for bit, while the campaign's words alone would have skipped the candidate; the record moved by the host moves the next decision with the campaign's words unchanged. Which candidate is offered, in what order and
which first hit ends the scan are untouched (the host's SCAN decisions are unchanged by this slice; the sector lists they are served from are Core backing, required to move).

**The conformance assertion.** ENTITY_POSE is retained, no longer as an observer but as the declared staged conformance assertion: the shared C checker stops the engine by `entity_pose.mirror` when the campaign's position, attitude or motion vector differs from the host's record (a write that bypassed the host). Its other difference bits only describe the record. It is asked beside the original's reads at the offer and at the placement, as before; its three metadata reads are unchanged.

**Request validation precedes every lookup.** The provider classifies the 44 raw request words FIRST, with integers only (`entity_host::classify_candidate_words`: the same finite and flag domains, order and messages as the decision's own validation, and no floating-point state touched), and only then reads the MOBILE, OF_CITY and PLACEMENT_GIVEN flags (a flag is "set" by its bit pattern, as the old `!= 0.0` test read it) and looks the identity up. A malformed word (a signalling NaN, a flag outside +-0 / 1.0, a contradictory city flag) is therefore a request error whatever the registry holds: a missing, retired or busy record never replaces it, and an arbitrary finite nonzero flag never reaches the registry. A differential test compares the classifier with the decision's own validator over a large word set, and an x86-64 test shows the rounding mode and exception flags are preserved.

**Dispositions of the record domain** (the record is the one `Movers` registry; no query-time seeding, cache or fallback exists). A refusal names a state the provider cannot decide from; it is the correct answer to a forged, stale or withdrawn identity, and it is NOT the way a supported lifecycle is made to work. For each category the compiled source proves where the supported path stands:

| category at the question | compiled proof (`eech-engine-sys/tests/entity_pose_lifecycle_guard.rs` asserts the text) | control | disposition |
| --- | --- | --- | --- |
| mobile in a sector list with no host identity | all six creators link the member (which binds the identity and seeds the record) BEFORE the sector insertion, and the list insertion refuses a missing group | guard test 2; provider refusal control | refusal kept for a forged or absent identity only; not a supported state |
| identity issued, no record | the bind seeds the record from the finished defaults, and every collidable kind has a movement kind | guard 1; `movement.rs` lifecycle test | not reachable by creation; refusal kept for an unissued identity |
| record without a motion vector | the canonical vector is seeded at the bind for every kind and held by the lifecycle claim | guard 1; Movers lifecycle test | required to stay seeded; refusal kept as the tripwire |
| loss, wreck, group unlink | an unlink never clears the identity or retires the record; only the heap release retires | guard 3; Movers test; provider boundary test decides a wreck from its record | preserved (a wreck is decided from its record) |
| released / retired identity | every destroyer removes the mobile from its sector list BEFORE the heap slot (the only retire) is released | guard 4; provider refusal of the released identity | a retired identity is never visible to a scan; refusal is a stale-ID control |
| host keeps no records / unhosted ordinary build | the hosted entity decision is compiled only with the hosted movement and identities (`#error` otherwise); an ordinary build without it keeps the original behaviour | guard 5 | not a runtime state of the hosted build |
| busy registry | the provider reads with `try_borrow` and refuses by name | provider control | the correct answer to re-entrancy; no supported path asks while stepping |

**Open dispositions, not waived.** (a) The normal hosted AI aircraft path returns through the host before the named legacy bodies: `fw_move.c` calls `hosted_fixed_wing_step` before the sole taxi call (1959/1980), calls `hosted_fixed_wing_flight` before the non-hosted absolute caller (1984/2059), and `basic_fixed_wing_movement_absolute` itself returns through `hosted_fixed_wing_step` (2193). The helicopter uses hosted guided begin/end (`hc_move.c` 802); its legacy attitude/vector/position block (867) is excluded by the hosted-movement guard. Aircraft POSITION/MOTION setters announce `campaign_movement_place` (`ac_vec3d.c`). Those legacy body names therefore do not establish an active hosted bypass. This is limited source proof for the normal server-created path: player flight dynamics, pack/unpack and other physical writers remain REQUIRED; their ownership and coherence are not established by the kind-1 observer or by this lifecycle proof. (b) Alive anti-aircraft and persons have no hosted vector writer: their vector is seeded and held, not stepped. (c) The multiplayer pack and unpack routes are unreached and required. (d) The lifecycle claim still ADOPTS the campaign's vector at its first use (Core-origin, required to retire). (e) The native observer saw only kind-1 offers, so it dispositions no other kind's vector or any missing-record category; the table's proofs are source reads and sociable controls, not a native observation. (f) The moving public path (below). Every other scope item listed under "What this does NOT close" remains required.

**The public witness** (`lua/entity-authority.lua`, no new script; the inventory stays 62): the natural `control` also requires that no mobile candidate question was unnamed (`candidate_mobile_named` / `candidate_mobile_unnamed`); `withdrawn_record` makes the HOST retire the identity of the first named mobile candidate question through its own movement callback, and the real provider must then refuse it by name (`entity_candidate.refused`, "retired") with no narrow phase and nothing asked afterwards;
`displaced_record` makes the host place that candidate's record elsewhere (a host-only record change) and the observation must stop the engine at `entity_pose.mirror`. A completed control proves nothing about which writer made a difference and the withheld, malformed and mismatch controls are not a physical-cohort comparison. **`withdrawn_record` and `displaced_record` are named stops, not a positive physical-effect proof.**

**The host-side movement modes** (completion modes, not refusals; the engine's own movement callback is the host's `hosted_move`, a pass-through that logs each `move` answer per identity). `moving_control` selects, instead of the nearest enemy, the nearest enemy of a launcher with an unguided ballistic weapon that is a routed vehicle whose LAST host step (an actual `move` answered in the final warmup frame) has a motion vector above 1 m/s; none is the exact blocker `UNREACHED: no moving routed enemy at launch` (nothing is tuned). It requires the target itself to be asked `need_updated` (the shared C asks only of a moving candidate: the per-identity proof), to reach a `test` and every mobile question to carry an identity. `terrain_lift` adds, after the launch (the launch aim is unchanged), one real physical input change in the host's own callback: the target's routed `move` step carries a terrain elevation raised by a constant (500 m, only within the step's own map ceiling; the waypoint, speed demand and every campaign decision untouched), the host's solver answers at that elevation, the engine writes it to Core, and ENTITY_POSE must still agree (a difference stops at `entity_pose.mirror`). It needs a later question about the target (weapon updates run before vehicles in a frame, so the first question that can see a lift is in the next frame; a cohort that ends first is `UNREACHED`). Both may run under the original profile (they change movement, not an entity question), where the outcome and the trace hashes, never a per-candidate question, are comparable. A coherent Core-and-record change alone does not show which of the two the old solver consumed: that is the provider's canonical-versus-discarded-mirror test, and the limit is stated. The scripted driver controls (`tests/entity_authority.rs`) prove the selection, the lift's bound and its exact scope (only the target's armed `move` step; the engine's own table is never mutated), and every UNREACHED distinction; the physical effect is Root's native run.

**What this does NOT close.** The moving public path: no Linux fixture of a moving collision candidate exists and the native control had `moving` 0 and `candidate_need_updated` 0, so mobile counts are not moving parity and no warmup is tuned. The pack and gunship routes and every unreached writer. The destructive UPDATED getter's whole-flag AND and the shared terrain record keep their order and effects (untouched). The sector lists, their insertion and relink history,
static, fixed and city objects and city children, the lifecycle claim's Core-origin adoption, the unrecorded and pre-bind states, the observation's own round trips (they cost wall time), the raw tree/forest/cloud/RNG, hierarchy/devices/muzzles/rotor/smoke/host identities and the DCS needed-only mapping all remain required work.

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
