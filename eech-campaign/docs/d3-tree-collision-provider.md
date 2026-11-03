# D3 tree segment collision: the host-owned query (EEDCS-49)

Status: **production source implemented and focused-checked; no native run yet** (Root owns the native actuals). This is the *tree* slice of Task 49. It is not
Task 49: `get_line_of_sight_collision_entity` (entity/object collision) is still the campaign's, and whole D3 / EEDCS-21 stay open.

## What changed

`get_line_of_sight_collision_tree` (`aphavoc/source/entity/en_misc/en_maths/en_cllsn.c`) no longer scans sectors or intersects anything in the ordinary module. With
`EECH_CAMPAIGN_HOST_TREE` (defined by `eech-engine-sys/build/main.rs` in every build except feature `collision-fixture`) it checks its arguments, keeps the debug line, and calls
`campaign_terrain_tree_collision` (`mb_hostterrain.c`). That carries the question to the host's **terrain provider**, as two more operations of it:

| op | request (every float a 32-bit pattern, `0x` + 8 hex digits, three comma separated for a position) | reply |
|---|---|---|
| `tree_begin` | `old`, `new`, `point`, `normal` (the caller's own buffers' words), `ignore_forest_edge_trees`, `skip_tree_top_test`, `winter` | `{state='need_terrain', end='old'\|'new', x, z}` or `{state='done', hit, point, normal}` |
| `tree_terrain` | `elevation` (pattern), `winter` | the same two |

The campaign answers each `need_terrain` with the same call, in the same order and with the same shared-record side effect as the original (`get_3d_terrain_elevation`, which is
itself host-backed since Task 47), at most twice (old end, then new end), and only writes the caller's `intercept_point` and `face_normal` after the whole reply is checked.
Patterns, not numbers, cross the boundary because the caller's buffers can hold anything (the weapon step passes an uninitialised `face_normal`) and a miss must hand them back
as the original left them: nothing is converted, normalised or zeroed.

Layers: C `mb_hostterrain.{h,c}` -> `eech-engine-sys` structs -> `eech-engine` `TerrainQuery::{TreeBegin,TreeTerrain}` / `TerrainAnswer::Tree` -> `eech-dc` Lua `TREE_BRIDGE` (shape
checks, markers `tree_begin.callback|answer`, `tree_terrain.callback|answer`, C-side `tree.lookups|answer|query|request`) -> the host's Lua provider. **No new boot key and no new
callback:** the terrain provider is already required and process-kept.

Reference host (`eech-world`): `src/tree_host.rs` drives the closed `trees.rs` loader/kernel (`csrc/tree_step.c`, unchanged) from `host.terrain_provider`:

* map facts: the terrain's own `Facts` (extents, sector counts, height range) = the facts the campaign took at boot;
* tree files: `tree_pos.sec/.dat` of the map's terrain directory (absent files or a header that differs from the terrain's sector counts are the original's "no trees loaded": an
  answer, the query still reads its two elevations and misses);
* forest tree collision triangles: walked from the host's **model store** (`OBJECT_3D_FOREST_TREE_OBJECT`, depth 0, 1, ...). None at depth 0 is `KnownAbsent` (retail); a model the
  store cannot show to be backed (`Refused`) refuses the query, never "absent";
* loaded by the first tree question, once. `host.terrain_open` providers get a tree service only when the directory is `<root>/common/maps/map<N>/terrain`, otherwise a tree question is refused by name.

## Contract coverage (source facts, not a new diagnostic programme)

| contract item | where it is held |
|---|---|
| old/new positions, initial point+normal words, two flags | `TREE_BEGIN` words; `eech-engine` and `eech-dc` tests assert signed zero / NaN patterns arrive unchanged both ways |
| map/tree/forest geometry identity | host: terrain facts + terrain directory tree files + model store; census is available from `trees.rs` but **not surfaced** by the provider path (limitation) |
| ordered endpoint lookups and their shared-record side effects | C adapter (old before new, <= 2, finite, `tree.lookups` marker otherwise); `lua/tree-authority.lua` control asserts order, one `point` read per lookup, `tree_terrain.elevation` == that read's pattern, and the second read carrying the first read's hint |
| partial point/normal writes on misses, early exits, map-outside, scan order, tie rules, zero-axis masked FP, complete FP state/order | host kernel, closed and unchanged: `tree_conformance` (15054 original queries / 29894 ordered terrain answers exact; default and controlled original raw unchanged) |
| consumers | the weapon step (`wn_move.c` `CAMPAIGN_STEP_LOOKUP_TREES`, flags FALSE,TRUE), avionics callers unchanged |
| RNG | no draw, constructor or padding in the adapter or the host service |
| cancellation / refusal | any refusal, malformed reply, third lookup, wrong end, missing provider or nested query is `debug_fatal` (the engine stops, named); the host drops a pending query on every refused request and a new `tree_begin` replaces any open one |
| equivalent absence vs missing/refused host | absence (no tree files, no forest triangles) is an *answer*; a missing provider or a refusal stops the engine by name |

## Remaining coupling (honest)

* The two terrain elevations are still asked of the campaign's host-backed terrain (declared transitional, as for line of sight). A host that owns them answers `done` at `tree_begin` and is never asked.
* The debug *box* drawn from the solver's bounds (`draw_collision_boxes`, external view) is not carried; the debug line is. Headless modules never draw.
* The tree slice does not touch entity/object collision (`get_line_of_sight_collision_entity`), the sector entity lists, or any consequence of a hit.
* The forest geometry is a second load of the model store inside the host process at the first tree question (bounded, documented; not shared with `host.model`'s store).

## Original characterisation profile

Feature `collision-fixture` (+ `weapon-fixture` etc. as before) is the **explicitly original** tree solver: `EECH_CAMPAIGN_HOST_TREE` is not defined and the fixture's
observer hooks live inside the original body. Merged onto the host-terrain base it needed two honest changes, both labelled: (1) its shared terrain record view is now the
host-semantic record (`face_stored`, `hint` empty-or-not) instead of the legacy sector/triangle/pointer words, which do not exist on this base; the record rows and hashes are **not comparable**
with the original-terrain evidence at `47a4af0d1`/`b44d2c96f` (the physical words are); (2) `lua/collision-characterization.lua` now selects the reference arithmetic profile and boots with
the terrain/model/environment providers like every other retained driver. Old evidence is preserved, not re-recorded.

## DCS mapping (not executed live; M6)

A DCS host implements `tree_begin`/`tree_terrain` from its own world. It has no `tree_pos` files: it may answer `done` at `tree_begin` from its own ray test against forest objects, or
`done {hit=false}` *as an explicit host decision* when it cannot test trees. That is a degradation to be recorded as such (trees invisible to weapon steps), never a fabricated hit or a
hidden fallback to the original solver. It must return the caller's own `point`/`normal` words on a miss. Nothing here claims that DCS can supply forest data or that live M6
execution happened.

## Checks and commands

Focused (pinned Linux image, cache `eech-linux-d3-collision-driver-5416adb3b-20261029`): `cargo test --locked -p eech-engine-sys -p eech-engine -p eech-dc -p eech-world`, `cargo fmt --all -- --check`,
`cargo clippy --locked -p eech-engine-sys -p eech-engine -p eech-dc -p eech-world --all-targets -- -D warnings` and the same for `-p eech-dc --features collision-fixture`. The compile guard is
`eech-engine-sys/tests/tree_host_guard.rs`. Native authority (Root): build `eech-dc` with `--features weapon-fixture` (never `collision-fixture`) and run
`eech-world lua/tree-authority.lua root=<install> mode=<control|forced_hit|withheld_tree|bad_tree_state|bad_tree_word|wrong_end_first|third_lookup|done_not_boolean> out=<jsonl>`;
exit 0 is the verdict (it self-asserts; `forced_hit` is additionally compared with the control's terminal rows).
