# D3 tree segment collision: the host-owned query (EEDCS-49)

Status (5 November 2026): **the finite TREE query handoff and its native authority are independently CLOSED** (see "Final closure" below; the earlier `CHANGES_REQUIRED` and `UNREACHED` records are
kept as history). This is the *tree* slice of Task 49. It is not Task 49: the entity/object slice is only started (its narrow phase is host-asked, see
[d3-entity-collision-provider.md](d3-entity-collision-provider.md); its sector scan, broad phase and roster are still Core), and the raw Core tree loader, the forest-object constructor and the
cloud block (below) are still the campaign's. Whole Task 49, the full shipping regression, the dependencies and whole D3 / EEDCS-21 stay open.

## What changed

`get_line_of_sight_collision_tree` (`aphavoc/source/entity/en_misc/en_maths/en_cllsn.c`) no longer scans sectors or intersects anything in the ordinary module. With
`EECH_CAMPAIGN_HOST_TREE` (defined by `eech-engine-sys/build/main.rs` in every build except feature `collision-fixture`) it checks its arguments, keeps the debug line, and calls
`campaign_terrain_tree_collision` (`mb_hostterrain.c`, whose dialogue is `mb_hosttree.h`). That carries the question to the host's **terrain provider**, as two more operations of it:

| op | request (every float a 32-bit pattern, `0x` + 8 hex digits, three comma separated for a position) | reply |
|---|---|---|
| `tree_begin` | `old`, `new`, `point`, `normal` (the caller's own buffers' words), `ignore_forest_edge_trees`, `skip_tree_top_test`, `winter` | `{state='need_terrain', end='old'\|'new', x, z}` or `{state='done', hit, point, normal}` |
| `tree_terrain` | `elevation` (pattern), `winter` | the same two |

The campaign answers each `need_terrain` with the same call and the same shared-record side effect as the original (`get_3d_terrain_elevation`, which is itself host-backed since Task 47),
and only writes the caller's `intercept_point` and `face_normal` after the whole reply is checked. **The original reads the OLD end only when the old end is inside the map and the NEW
end only when the new end is**, so a query asks for none, OLD only, NEW only (a segment that enters the map), or OLD then NEW; an end is asked at most once and OLD never after NEW
(marker `tree.lookups`). The first batch's dialogue accepted only none / OLD / OLD-then-NEW, so a NEW-only segment was a fatal stop (RED below).
Patterns, not numbers, cross the boundary because the caller's buffers can hold anything (the weapon step passes an uninitialised `face_normal`) and a miss must hand them back as the
original left them: nothing is converted, normalised or zeroed.

Layers: C `mb_hostterrain.{h,c}` + `mb_hosttree.h` -> `eech-engine-sys` structs -> `eech-engine` `TerrainQuery::{TreeBegin,TreeTerrain}` / `TerrainAnswer::Tree` -> `eech-dc` Lua `TREE_BRIDGE`
(shape checks, markers `tree_begin.callback|answer`, `tree_terrain.callback|answer`, C-side `tree.lookups|answer|query|request`) -> the host's Lua provider. **No new boot key and no new
callback:** the terrain provider is already required and process-kept.

Reference host (`eech-world`): `src/tree_host.rs` drives the closed `trees.rs` loader/kernel (`csrc/tree_step.c`, unchanged) from `host.terrain_provider`:

* map facts: the terrain's own `Facts` (extents, sector counts, height range) = the facts the campaign took at boot;
* tree files: `tree_pos.sec/.dat` of the map's terrain directory (absent files or a header that differs from the terrain's sector counts are the original's "no trees loaded": an
  answer, the query still reads the elevations of the ends inside the map and misses);
* forest tree collision triangles: walked from the **same finalized model store the campaign's own model callbacks answer from** (`host.model`'s provider, `model_host::ReferenceModel`),
  scene `FOREST_TREE_OBJECT` (the compiled semantic name; `OBJECT_3D_FOREST_TREE_OBJECT` is the enumerator spelling and is no scene name), depth 0, 1, ... None at depth 0 is `KnownAbsent`
  (retail); a model the store cannot show to be backed (`Refused`) refuses the query, never "absent". The store is used only once the campaign's custom-model walk announcement is
  complete and the store was replayed (or verified) in that announced order (`walk_final`); a missing provider, a provider still being announced to, one that answered from a sorted image
  because no walk was announced, a `host.model_provider(store)` provider (not registered) and another installation are each a refusal by name. No second loader, sorted order or cache exists;
* loaded by the first tree question, once. `host.terrain_open` providers get a tree service only when the directory is `<root>/common/maps/map<N>/terrain`, otherwise a tree question is refused by name.

## Contract coverage (source facts, not a new diagnostic programme)

| contract item | where it is held |
|---|---|
| old/new positions, initial point+normal words, two flags | `TREE_BEGIN` words; `eech-engine`, `eech-dc` and the dialogue control assert signed zero / NaN patterns arrive unchanged both ways |
| map/tree/forest geometry identity | host: terrain facts + terrain directory tree files + the campaign's own final model store; the tree census is available from `trees.rs` but **not surfaced** by the provider path (limitation) |
| ordered endpoint lookups and their shared-record side effects | `mb_hosttree.h` (none / OLD / NEW / OLD then NEW; duplicate, reversal, unknown end and non-finite point stop at `tree.lookups`), exercised by `eech-engine-sys/tests/tree_dialogue.rs` through the REAL dialogue code bound to a scripted host; `lua/tree-authority.lua` asserts the same subsequences from what the provider sees, one `point` read per lookup at the asked coordinates, `tree_terrain.elevation` == that read's pattern, and the second read of a query carrying the first read's hint |
| partial point/normal writes on misses, early exits, map-outside, scan order, tie rules, zero-axis masked FP, complete FP state/order | host kernel, closed and unchanged: `tree_conformance` (15054 original queries / 29894 ordered terrain answers exact; default and controlled original raw unchanged) |
| consumers | the weapon step (`wn_move.c` `CAMPAIGN_STEP_LOOKUP_TREES`, flags FALSE,TRUE), avionics callers unchanged |
| RNG | no draw, constructor or padding in the adapter or the host service |
| cancellation / refusal | any refusal, malformed reply, duplicate or reversed end, missing provider or nested query is `debug_fatal` (the engine stops, named); the host drops a pending query on every refused request and a new `tree_begin` replaces any open one |
| equivalent absence vs missing/refused host | absence (no tree files, no forest triangles) is an *answer*; a missing provider or a refusal stops the engine by name |

## Genuine RED / GREEN

* NEW-only dialogue: with the first batch's logic (`lookups == 0` must be OLD) the isolated control `tree_dialogue` FAILED exactly on the two NEW-only cases (the dialogue stopped at
  `tree.lookups`); with the subsequence rule it passes. Child exit codes are in the STATUS entry. The native NEW-only witness is `mode=new_only`.
* Forest identity: with the first batch's constant `OBJECT_3D_FOREST_TREE_OBJECT` the compiled scene lookup returns nothing, so the production source could only refuse. The order-sensitive
  control (a scene defined by two custom directories; the final scene is the last definition walked) shows the campaign's order and the sorted image giving different geometry.

## Remaining coupling (honest)

* The elevations of the ends inside the map are still asked of the campaign's host-backed terrain (declared transitional, as for line of sight). A host that owns them answers `done` at `tree_begin` and is never asked.
* The debug *box* drawn from the solver's bounds (`draw_collision_boxes`, external view) is not carried; the debug line is. Headless modules never draw.
* The tree slice does not touch entity/object collision (`get_line_of_sight_collision_entity`), the sector entity lists, or any consequence of a hit.

## Original characterisation profile

Feature `collision-fixture` is the **explicitly original** tree solver (the genuine Original control builds `eech-dc` with BOTH `weapon-fixture` and `collision-fixture`):
`EECH_CAMPAIGN_HOST_TREE` is not defined and the fixture's observer hooks live inside the original body. Merged onto the host-terrain base it needed two honest changes, both labelled: (1) its shared
terrain record view is now the host-semantic record (`face_stored`, `hint` empty-or-not) instead of the legacy sector/triangle/pointer words, which do not exist on this base; the record rows and
hashes are **not comparable** with the original-terrain evidence at `47a4af0d1`/`b44d2c96f` (the physical words are); (2) `lua/collision-characterization.lua` now selects the reference arithmetic
profile and boots with the terrain/model/environment providers like every other retained driver. Old evidence is preserved, not re-recorded.

## DCS mapping: an UNRESOLVED REQUIRED mapping (not executed live)

No verified DCS query for forest or tree intersection is named. The pinned `tslua-dcs` declarations used for the target-point plan (`1179554fcafcef92d71313336cc09500febf8245`) name no tree,
forest or ray-against-scenery call, and no host-data workaround (a versioned tree/forest dataset for the DCS map) has been defined from primary evidence. So there is **no concrete DCS provider
plan** here, and none is invented. Until one is named and pinned, a host that cannot answer `tree_begin` must **refuse it** (the reference refusal is the named
`[terrain-provider:tree_begin.callback]` stop); it must NOT answer `done {hit=false}`: an unsupported physical query is unknown, not a measured or configured absence, and it would change the
weapon's physical behaviour. The only legitimate absences are the reference host's measured ones (no tree files in the map, no forest triangles in the model). This stays open as required
D3/Task49 mapping work and is not deferred to M6; live DCS behaviour checks remain M6. A genuine tree MISS returns the actual FINAL words of the original contract: the caller's own `point`/`normal`, or what a failed attempt partly overwrote (a miss does not universally leave the buffers unchanged, and the closed host kernel reproduces the partial writes); a host must not zero, normalise or restore them.

## Required remaining scope beside entity/object collision (open, not started here)

The ordinary boot still loads and constructs the raw Core tree data, because those loaders are not detached by the tree solver route (reviewer entry proof: `eech_engine.c` calls
`load_3d_terrain_game_data`; `zone_sc.c` unloads and reloads the terrain; both are in `sources.txt`; no staged patch replaces `load_3d_terrain`):

* the raw tree loader (`terrdata.c` maps `tree_pos.sec/.dat`, allocates the sector arrays, associates raw tree pointers) and the `FOREST_TREE_OBJECT` construction;
* the cloud block (`terrdata.c`, `#ifndef OGRE_EE`): `CLOUD` construction and, when `session_cloud_puffs > 1` or `== 1` and the season is not DESERT, per-sector allocation with CRT `rand()` draws.

Other compiled consumers of the raw tree data exist besides the (now hosted-out) solver: `gunships/dynamics/common/co_force.c` (`get_terrain_3d_tree_sector_data`, the player's flight-dynamics
tree force) and the renderer (`terrgeom.c`). Whether either is reachable in the supported headless boot has NOT been shown, so nothing is removed and nothing is called dormant.

RNG disposition (a fact, not a removal): the engine seeds the shared stream before the terrain loads (`eech_engine.c` `set_random_number_seed`, which also `srand`s the CRT), the cloud block draws
from that CRT stream, and the only other `set_random_number_seed` caller in the sources is the menu (`uisess_fn.c`); no reseed between the load and the first outside consumer was found in
source. So the cloud draws are **not shown to be erased**; removing the block would change the stream every later consumer sees, which is a baseline-governed semantic change, and the existing
`rng_unchanged` guard is not proof for the CRT stream (it may cover another stream). The tree loader's own text contains no `rand()`, but that is lexical, not transitive RNG proof: the forest constructor `construct_3d_object_by_name` (`3dobjid.c`) calls `construct_3d_object`, which in `3dobjdb.c` calls `skin_random`, and `skin_random` draws (`3dobjid.c`) when the object's `skin.animation >= 0`. The effective forest skin metadata, and so whether the constructor draws in the supported boot, is UNKNOWN until supported data, call and seed observations establish it. It belongs to the same loader/forest/cloud disposition below; no loader is deleted and no dummy draw is added on this account.

The smallest bounded capture that would settle it, if the removal is wanted: for the supported scenarios (Lebanon map5, Georgia), the effective `session_cloud_puffs` and season at the boot
`load_3d_terrain` and at the zone reload, the count of CRT `rand()` calls made inside `load_3d_terrain`, and whether a `srand` precedes the first consumer afterwards (the existing counting header
`eech_tp_rand.h` renames every `rand()` of a build and is the existing way to count). The same capture must also show whether `construct_3d_object` for `FOREST_TREE_OBJECT` (and `CLOUD`) reaches `skin_random` (the object's effective `skin.animation`). If the cloud branch is not taken and the constructors draw nothing there are no draws and the block can go with no RNG effect; if draws
happen and are not erased, removal is a semantic change for lead review, and dummy draws are not the answer. Required Task49/D3 work, OPEN.

## The native controls so far and the controlled view input

The source `f0f928147` was cleared for native validation by the same reviewer (`ad1c1503…`) and Root built Windows hosted, explicit Original, ordinary and Linux hosted fixtures (all actual exit 0, 87 payloads verified) and ran the ordinary exact short comparison: Tacview, observations and metrics COMPLETE and literally equal to the retained `80fd` baseline. The `tree-authority` hosted control was then run twice, and **both runs correctly failed UNREACHED** (zero tree tests); both are preserved, never turned into a pass:

1. The first selector (the first launcher with an unguided ballistic weapon and its nearest enemy): an anti-aircraft M61 cannon at a distant enemy, one weapon, `hit_land` in 7 steps, `tests_delta=0`.
2. The second selector (the nearest launcher-target pair within 50 m, commit `3619cea4d`, Lua blob `ed56799d`): a person with a PKM weapon and a target 0.039 m away, `hit_land` in 72 steps, `tests_delta=0`. Safe projections: `reference/d3-tree-collision-production/f0-native-identities-and-first-control-20261103.json` and `f0-second-control-selector3619-unreached-20261103.json`.

Source fact (not a physics regression): the original flight (`csrc/wn_flight.c`) asks for the tree lookup only after an entity-collision MISS and only when `high_precision_collision` holds: a non-AI target or launcher, a viewed weapon, or the weapon's NEW position within 50 m of its target. The nearest pair at launch says nothing about the new position of a projectile that moves tens of metres a step, so a distance rule over launch positions is the wrong precondition.

**The controlled view input (existing protocol, Lua only).** The host protocol already carries the physical fact the rule reads: the weapon-step lookup `CAMPAIGN_STEP_LOOKUP_VIEW_WEAPON` (`mb_hostaim.h`), which the campaign answers as `r0` in `wn_move.c` and the host's step consumes before its unmodified precision rule and its real entity and tree queries. The weapon-step request reaches the host's aim function with the lookup answers as events `{kind, a, b, c, d, f}` (twelve floats, `f[6..11]` = `r0..r5`; `aim_event_table` in `eech-dc`, read back by `step_events` in `eech-world`). The witness's aim delegate therefore, when armed and for `mode == "weapon_step"` only, copies the request and the single answer of kind 1 and sets its sixth result word (`f[7]` in Lua) to 1, then calls the unmodified `host.aim`. Every other field, event, lookup and mode is passed as it came (a signed zero in another word stays a signed zero; the caller's table is not modified). It is a CONFIGURED diagnostic view input, not a claim that a camera viewed the round and not a campaign or player decision; no tree call, entity-miss answer or counter is faked: the entity miss, the tree question and `contact_trees_tests` stay measured. It is applied identically in the hosted and the Original profile and to every weapon step of the war while armed (the protocol carries no weapon identity, so natural weapons are affected too, and the control is "the TREE provider unmodified under this controlled view input", not a wholly unaltered host). Ordinary and full runs never load this witness and are entirely unmodified.

The launch is back to the simple known one (the first launcher with an unguided ballistic weapon in stock and its nearest enemy, the f0 baseline pair). The run records `view_steps_seen`, `view_answers_seen`, `view_applied`, `view_already_set` and the weapon fixture's `precision_high` delta, and requires that the input reached a weapon step and that the step took the high-precision path before the tree counters mean anything; otherwise it is UNREACHED. The tree counter / provider guard is unchanged: a run with no tree test is still UNREACHED. If the existing transport could not represent the view fact the work would have stopped here; it can (the answer events carry all twelve words).

## Final closure of the finite TREE handoff (independent, 3 November 2026)

The same independent reviewer's final report [`independent-tree-production-authority-eb2-nativef0-20261103.json`](../reference/d3-tree-collision-production/finite-authority/independent-tree-production-authority-eb2-nativef0-20261103.json)
(SHA-256 `7ad1a698f7996012eda90584661aca717c6b34bf4817f48e2837f6c595fc1dca`) has no findings. Native source `f0f928147f6ca81906212038d86ecdf98ce5b400`; literal witness
`eb2b2eb6781fefce7e6b76abdbbd4d947010a7f7` (`tree-authority.lua` blob `9123f1448ec8ec412b146a2ff290360e5d7769ab`). Verified actuals: the healthy control plus the ten-case matrix (11 cases,
22 full input lists, 87 payloads each); the forced hit changes the real cohort's physical terminal; seven named refusals; the explicit Original profile reaches real TREE with no host, the
expected capability RED with a matching physical signature; and the ordinary f0 short is COMPLETE and literal-equal to the retained `80fd` baseline. The two earlier selectors that never
reached TREE stay recorded as UNREACHED; nothing was retried, searched or re-graded. The safe projections are in
[`reference/d3-tree-collision-production/finite-authority/`](../reference/d3-tree-collision-production/finite-authority/README.md).

**Limits, stated by the review and kept:** the view input is an explicit configured diagnostic input (not a claim that a camera viewed the round), the armed window is global (the protocol
carries no weapon identity), and a direct measurement of the new TREE caller's floating-point state is NOT claimed. Open and unchanged: the raw Core tree loader, forest constructor (RNG unproved)
and cloud block, the retail forest-store fact, the DCS tree mapping, the established M4 regression pack on the complete Task 49 candidate, the dependencies, and whole D3.

## Checks and commands

Focused (pinned Linux image, cache `eech-linux-d3-collision-driver-5416adb3b-20261029`): `cargo test --locked -p eech-engine-sys -p eech-engine -p eech-dc -p eech-world`,
`cargo test --locked -p eech-engine-sys --features tree-dialogue-check --test tree_dialogue` (the dialogue control), `cargo fmt --all -- --check`, strict clippy for the four crates and for
`-p eech-dc` with `collision-fixture`, `weapon-fixture` and both. The compile guard is `eech-engine-sys/tests/tree_host_guard.rs`; the driver controls are `eech-world/tests/tree_authority.rs`.

Native authority (Root), one fresh installation root and process per run, exit 0 is the verdict, evidence created exclusively through `host.create_exclusive` (its handle observes the platform close; `out=` must not exist). The witness checks the launch report against the real emitter's schema (`restored` is -1 link kept or 1 restored, `creates` a positive whole number, whole-number counts) and every cohort terminal for all fields the emitter writes (type, code, steps, pos, velocity, lifetime) in their `%d`/`%.9g` grammar:

* hosted module = `eech-dc --features weapon-fixture`: `eech-world lua/tree-authority.lua root=<install> mode=control out=<new.jsonl>`, then `mode=new_only`, `mode=forced_hit` (compare its terminal rows
  with the control) and the refusals `withheld_tree`, `bad_tree_state`, `bad_tree_word`, `done_not_boolean`, `new_then_old`, `old_twice`, `third_lookup`;
* Original control = `eech-dc --features weapon-fixture,collision-fixture`: `... profile=original mode=control`. It FAILS BY DESIGN with `CAPABILITY RED` when the weapon step made real tree tests
  (`contact_trees_tests` rose), the fixture API is present and the host was asked nothing; anything else is `UNREACHED` (setup), not a red.
