# D3 entity/object collision: the host-owned narrow phase (EEDCS-49, second slice)

3 November 2026, status refreshed 5 November 2026. **The narrow phase is native-validated for the FINITE STATIONARY handoff only (corrected source `f153`/`7ead`; [`finite-authority/`](../reference/d3-entity-collision/finite-authority/README.md)); the moving public path, the physical scan and Task 49 are NOT accepted.** The body below is the dated 3 to 5 November account; its statements about runs are updated in place where they have since changed. This is the
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
| sector window, sector lists, tail-first order, exclusion, `TALLEST_STRUCTURE_HEIGHT` skip, city box, mobile/fixed/moving classification, the broad-phase radius test | campaign entities and the host-authored radius table | **campaign, declared transitional coupling** |
| moving sweep ends (`UPDATED` flag, motion vector, `get_delta_time`) | campaign entity state; the destructive `UPDATED` getter keeps its place because the scan is untouched | campaign |
| stationary placement (position, heading, terrain-aligned normal) | campaign entity state and the terrain question `get_3d_terrain_face_normal`, asked only when the broad phase passed, as before | campaign (derived in the original's order) |
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
controls ran, 14 full input checks, 87 payloads each, every exit 0. Healthy: 279 stationary armed questions (44 real hits, 235 real misses), six actual weapon lookups, one cohort member complete. Forced hit:
45 valid answers change the cohort terminal. Forced miss: 749 valid partial-write answers, no weapon entity hit (the healthy weapon made none either, so this is valid MISS coverage and **not** a measured
physical change). The three refusal modes stop at their exact anchored markers with nothing read afterwards. The Original profile reaches the entity caller with no host question: the expected capability RED.
The same independent reviewer found no findings (`FINITE_STATIONARY_ENTITY_HANDOFF_AUTHORITY_VERIFIED`). **Limits:** every queried pose was stationary, so the MOVING public path is not measured; the view input is
global while armed, with no per-weapon attribution; no new direct x87/MXCSR or RNG measurement of the public caller. The first native command was
`eech-world.exe <literal entity-authority.lua> root=<fresh root> mode=control out=<new .jsonl>`; runtime JSON Lines stay private (retail coordinates).

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
