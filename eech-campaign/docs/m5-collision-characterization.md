Current execution checkpoint: eight actual Linux driver guard tests, formatting
and host all-target Clippy pass0; these use the stand-in and prove no physical
query. Tested hashes/exact log are in reference/d3-collision-characterization/.
The original C fixture/native retail runs remain unbuilt/unrun. Three fresh roots
are copied and pre-manifest verified. The following is the source-preparation
record from before those guards ran; live ledger is now v60.

# D3 object and tree collision queries — original characterisation (EEDCS-49)

29 October 2026. **SOURCE ONLY: nothing in this document has been built or run.** This is the original-behaviour characterisation fixture for
[EEDCS-49](https://flying-dice.atlassian.net/browse/EEDCS-49) (parent [D3/EEDCS-21](https://flying-dice.atlassian.net/browse/EEDCS-21)), prepared
on `codex/d3-object-tree-collision` from accepted integration `6bbaa8197a4843181d03cd355d562af998c75321`. No physical responsibility has moved:
production dispatch of `get_line_of_sight_collision_entity` and `get_line_of_sight_collision_tree` is unchanged and is still the original C.

Authority read live on 29 October 2026: [M5 goal](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337) v3,
[migration ledger](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/14548994) v59 (the D3 sections through "host ground-kernel preparation
verified"), EEDCS-49 and EEDCS-21 (both readable; nothing needed an unavailable tool). The historical roadmap and the TypeScript port are not scope.

## Scope of this task and what it does not do

G3a moves the physical segment/model/tree queries to the host and leaves campaign selection, exclusion policy, identity, decisions and consequences in
EECH_DC. **This step is only the original characterisation that precedes that move.** It does not touch model/subobject/device/target-point/bounds
services, weather/environment/RNG, host-time inputs, or physical loader/solver removal, and it does not decide the render/player reachability
disposition of the gunship consumers (below). Those remain whole-D3 work and are not deferred by this task.

## What the source says (SOURCE FINDINGS, not runtime evidence)

Read from `aphavoc/source/entity/en_misc/en_maths/en_cllsn.c` at the base revision. Each statement below is something the fixture is built to *measure*,
not something the fixture assumes.

### `get_line_of_sight_collision_entity (source, target, old, new, point, normal)`

* **It is first-found in a fixed scan order, not nearest-hit.** The sector window is the segment's bounding box widened by `LARGEST_3D_OBJECT_RADIUS`
  (150 m) and clamped to the map; sectors are scanned x outer, z inner; each sector list is scanned from the *tail* so static objects come before mobile
  ones; the first object whose accurate test hits is returned. A farther object in an earlier-scanned sector wins over a nearer one. The fixture's
  exclusion chain and shortened-ray re-query measure this.
* **Exclusion is on the sector-list entity only.** `target != los_source && target != los_target` is tested for the entity in the sector list. A city's
  buildings are children of the city and are never tested against the exclusion, so excluding a *building* does nothing; excluding its *city* excludes
  all of its buildings. A building listed directly in a sector is skipped.
* **Per-candidate gating.** Mobile candidates (`INT_TYPE_COLLISION_TEST_MOBILE`): moving if `|motion|² > 1`, then the swept box uses `INT_TYPE_UPDATED`
  and `get_delta_time()` (a moving target's old/new positions depend on both); stationary otherwise. Fixed candidates are skipped entirely when the
  segment's minimum y is above the sector's `FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT`. Broad phase is an AABB of radius + 10 m.
* **Narrow phase** uses the object's collision geometry triangles if the model has them, else its bounding box (six quads). A fixed object with
  `INT_TYPE_ALIGN_WITH_TERRAIN` also asks the terrain face normal at its position, which **writes the shared terrain point record**.
* **Outputs.** `point` and `normal` are documented "valid only on a hit", but the helpers (`modules/maths/vector.c`, `get_3d_line_triangle_intersection` and
  `get_3d_line_quad_intersection`) write the face normal **before** they know whether the segment hits, and write the plane-crossing point as soon as the
  segment crosses the plane, even when the crossing lies outside the face and the call then returns FALSE. So a *miss* can leave a written point and normal
  (the last face tested), and the same is true of the tree query when no tree is hit. That matters because the consumers report the point to the host even on a
  miss (below). The fixture measures it (`miss_point_written`, `miss_normal_written`) and folds the miss-time point into the physical fold.

### `get_line_of_sight_collision_tree (old, new, point, normal, ignore_forest_edge_trees, skip_tree_top_test)`

* Early `FALSE` when both ends are more than `TERRAIN_3D_MAX_TREE_HEIGHT` (35 m) above the terrain, or the segment is under 0.1 m in the xz plane.
  An end outside the map counts as 0 m above ground (new end: the old end's value). **Both elevation lookups write the shared terrain point record.**
* The scan window is the clamped segment box; every tree in the window is tested against y-range and a ±25 m xz window.
* **Two different tie rules.** A tree whose `type == 1` *and* for which `OBJECT_3D_FOREST_TREE_OBJECT` has collision geometry is tested by triangles and
  the **first triangle hit returns immediately** (it need not be the nearest, and it ignores a nearer quad hit already held). Every other tree (type 0,
  or type 1 with no geometry) is tested with two crossing quads (and a top quad if `skip_tree_top_test` is false), and the **nearest by 2D range wins**.
* The similar-triangle test divides by `dx` and `dz` of the segment. A segment with exactly zero x or z extent therefore divides by zero; the result is
  IEEE-defined only if FP exceptions are masked. **The fixture never probes such a segment** (declared below).
* `ignore_forest_edge_trees` skips `type == 0` trees; `skip_tree_top_test` omits the top quad. Every compiled consumer passes `(FALSE, TRUE)`.

### Compiled consumers (from `sources.txt` and the call sites)

| Consumer | Query | Call shape (what the fixture reproduces) |
|---|---|---|
| `mobile/mb_hostlos.c:275` (hosted LOS) | entity | source and target of the LOS pair excluded; `point` zeroed first, `normal` unwritten; hit flag and, on a hit, the point go to the host |
| `mobile/mb_tgt.c:455,494` (original LOS) | entity | same pair excluded; only hit/miss and, for the target end, the point's distance matter |
| `mobile/weapon/wn_move.c:466` (`CAMPAIGN_STEP_LOOKUP_COLLISION`) | entity | launcher as source, no target; `intercept` zeroed first; reports hit, `hit != the weapon's target`, `hit->type` and the intercept (also for a miss) |
| `mobile/weapon/wn_move.c:493` (`CAMPAIGN_STEP_LOOKUP_TREES`) | tree | `(FALSE, TRUE)`; `intercept` zeroed first; hit flag and the intercept (also for a miss) |
| `gunships/avionics/common/co_eo.c:681,1055`, `co_hms.c:733`, `co_radar.c:683-738`, `gunships/dynamics/common/co_force.c:1266,1334` | both | player gunship as source; `co_force.c` **uses `normal`** (dot product with the acceleration vector) |

The gunship files are in `sources.txt`, so they are compiled; whether any is reachable from the supported campaign-DLL runtime is **not decided
here**. D2's scope audit puts the dormant EECH cockpit/HUD path outside that runtime, but `co_force.c` and the avionics query callers are player
flight and sensor code, not that HUD path. Their disposition, and whether the `normal` output needs a host contract at all, is for D3's
render/player reachability item and the technical lead; the fixture records `normal` as a physical fact so either answer is supportable.

## The fixture

Feature `collision-fixture` (chain: `eech-dc/collision-fixture` → `eech-engine/collision-fixture` → `eech-engine-sys/collision-fixture`, which defines
`EECH_COLLISION_FIXTURE` for the C build). Never part of the shipping module.

* `csrc/eech_collision_fixture.c` — the probe. It calls the *original* functions; no production source was changed. `csrc/eech_engine.c`/`.h` gain
  the feature-gated entry `eech_engine_collision_probe`. The Rust wrapper is `Engine::collision_probe`; the Lua method is
  `engine:collision_probe (seed, entity_cases, tree_cases)`; it needs no host callbacks (neither query asks one).
* `lua/collision-characterization.lua` — the driver (public path: `require ("eech_dc")` → boot → frames → probe).
* `tools/build-windows-collision-fixture.sh` — the fixture build, the single-argument adaptation of `build-windows.sh` used by the other fixtures.
* `tests/lua/collision-caller.lua` and the `the_collision_driver_*` tests in `tests/host_callbacks.rs` — driver flow and report validation against a
  stand-in that measures nothing.

### Finite case set

*Structured* (a pure function of the loaded world; no random draw). Targets are real entities from the real sector lists in scan order, the first 3 of
each of 8 classes: moving / stationary mobile, fixed object, city building, each with / without collision-geometry triangles; and real trees from the
real tree sectors, the first 3 of each of 2 classes: quad-plane and geometry. Per entity target: a ray through its centre, the same ray reversed, a lateral
miss, and a ray over it; on the through ray, source/target/both exclusion of the target, a real decoy, the owning city of a building, and the chain of
hits found by excluding each hit in turn (up to three objects). Per tree: the through ray under all four combinations of the two flags, then reverse,
over, under, lateral, slant, high (above the 35 m gate), vertical and sub-0.1 m rays. Plus two edge rays per family across the map boundary (supported:
the entity scan clamps to the map, the tree query has an explicit outside-map branch).

*Random*: `entity_cases` and `tree_cases` seeded cases around entities/trees from a private reservoir sample (seed-derived; the game's random stream is
compared before and after and must not change).

### What each query records

Hit/miss; hit kind, hosted-id text and position; the intercept point (always, as the consumers report it) and the face normal (hits only), as float bit
patterns; whether a miss wrote the point or the normal; whether the cold query touched the shared terrain record; a **warm repeat** on exactly the
record the cold query left (a different warm answer is counted as cache state); and, for each hit, a **nearer-hit check** (the same query cut 0.1 % short
of the reported point; a hit there proves the first answer was not the nearest — the original asked a second question, not a model of it).

### Report schema (`key=value` text lines → one JSON Lines event each)

`col-probe`, `col-map`, `col-guard`, `col-census-entity`, `col-census-tree`, `col-targets`, `col-entity-kind` ×7 (struct: through, reverse, lateral,
above, exclusion, edge; random), `col-entity-exclusion`, `col-entity-common` ×2, `col-entity-type` ×0..n, `col-tree-kind` ×11, `col-tree-flags` ×8,
`col-tree-common` ×2, `col-physical`, `col-legacy`, `col-sample` ×≤(16+8)×2. The driver accepts a report only as a whole: exact row set, exact keys,
exact types (decimal count / flag / signed / 16- or 8-digit lowercase hex / three-word hex), counts that agree with the request and with each other
(see `cross_check`), sample counts equal to the quota or to the number of cases if fewer, and a header equal to the request. Anything else is
`report_invalid` and fails the run; nothing is read as zero.

Three kinds of fact are kept apart: **PHYSICAL** (`col-physical`: inputs, exclusion role, hit flag, hit kind/hosted id/position, point, normal),
**cache/short-circuit counters** (record touched, warm agreement, miss-time writes, nearest checks), and **LEGACY** (`col-legacy`: the hit's heap
slot and the terrain record's private bookkeeping — comparable only with a build that keeps both; never promoted to a physical claim). No pointer is
emitted.

### Preserved state, asserted

The shared terrain point record is snapshotted and restored byte for byte (`record_restored`, `record_words_before/after`); the game's random stream
is read before and after (`rng_unchanged`); every collision candidate's position, motion and updated flag, the sector membership and the frame delta
time are folded before and after (`world_unchanged`). The driver fails the run if any of the three does not hold. The probe is not claimed read-only
by name: it **does** write the shared terrain record during each query, which is why each cold query starts from a known sentinel record.

### Unsupported / never probed (declared)

NaN or infinite coordinates; a tree ray with exactly zero x or z extent (division by zero; depends on the FP exception mask, which is not verified);
`draw_collision_boxes` set; any world whose collision candidate has an invalid 3D shape or bounds (the probe is **refused** with
`col-refused reason=...` before anything is altered); a terrain whose extents differ from the world map; FPU not in round-toward-zero mode.

### Coverage is stated, not assumed

The `coverage` event and the final `done` event list every family the world offered no case or no hit for (no moving mobile at that time, no building,
no collision-geometry target, no terrain-aligned fixed object, no tree data, no structured hit, no two-object ray for scan order, no tree hit with the
consumer flags, …). `seconds=0` probes the freshly booted world where nothing moves; the default is 120 s so moving mobiles exist. A run with gaps exits 0
unless `require_coverage=yes`, and its final stdout line names the gaps: `COLLISION_CHARACTERIZATION_DONE probed=… repeats=… coverage_gaps=…`. **Absent
coverage is reported, never fabricated**: no ray is placed to manufacture a hit the world does not offer, and no scenario, campaign state or accepted
baseline was changed.

## How to run (expected command and environment — NOT YET RUN)

Build (Git Bash, Docker Desktop with Linux containers, clean checkout of the commit to characterise):

```sh
sh eech-campaign/tools/build-windows-collision-fixture.sh target/collision-fixture/<dir>
```

The wrapper adds `--features eech-dc/collision-fixture` to the one cargo line of `tools/build-windows.sh` after checking that is the only change.
`lua/*.lua` are copied by the stock build, so the driver is in `<dir>/lua/`; **this adds one file to the build's file manifest** (a test driver, not
a compile input; shipping DLL bytes are expected unchanged by this task but that is not verified).

Run, one native process per scenario, on a private copy of the verified retail root (the engine writes into its root; one root per run; no spaces in
paths):

```
<dir>\eech-world.exe <dir>/lua/collision-characterization.lua root=<root> out=<path>.jsonl scenario=lebanon_retail
    [seed=20261029] [entity_cases=2000] [tree_cases=2000] [repeats=2] [seconds=120] [after=0] [frame_ms=100] [probe=yes|no] [require_coverage=no|yes]
```

Knobs: arguments are validated before the output file is opened, `eech_dc` required or anything booted. `probe=no` with the same `seconds`/`after` is the
control; its `state_digest` is compared with the probed run's offline. The interesting runs are Lebanon and Georgia at `seconds=0` (nothing moving) and at
`seconds=120`, the separate Lebanon repeat root for a cross-boot reproduction, and `repeats>=2` (byte-identical reports in one process).

## Evidence status

| Item | Status |
|---|---|
| Source for fixture, driver, wrapper, build script | written, **unbuilt, unrun, not yet independently reviewed** |
| C fixture compiled (MinGW, `-Werror=implicit-function-declaration`) | **not done** |
| Rust feature code (`collision_probe` wrapper, `module.rs` method) built / clippy with `--features eech-dc/collision-fixture` | **not done** (default CI does not enable fixture features) |
| Lua driver and stand-in tests (`the_collision_driver_*`) | **not executed**; expected messages were derived by reading the driver, so the first run may expose a wrong expectation |
| Native probe on Georgia / Lebanon / Lebanon repeat; probe/no-probe control | **not done** |
| Tree and moving-target coverage in the retail worlds | **unknown** until run (headless boot loads trees only if the map's tree files exist and match the terrain) |
| Same-frame control for the terrain-record side effect | covered by design (cold sentinel + warm repeat + restore assertion); **unproven until run** |
| Production change / host handoff | **none** |

## Risks and couplings the characterisation is meant to expose

* Host contract must say what a **miss** returns in `point` (the weapon step reports it, and the source shows the original can leave a crossing point there rather
  than zero), and whether `normal` is part of the contract (player flight).
* A host that answers "nearest hit" would change which object is struck; the original is first-found in scan order.
* Building exclusion is by *city*; host identities for buildings versus cities must preserve that, or exclusion silently changes.
* A moving target's swept box depends on the engine's own frame delta time and updated flag — host time input (D3) coupling.
* Both queries write the shared terrain point record; hint/cache state already changed physical terrain answers in EEDCS-47, so this coupling must be
  carried or removed deliberately, not discovered by a Tacview difference.
* The tree geometry path and the quad path have different tie rules; a single host implementation must reproduce both or the difference must be an
  approved semantic change.
* The model geometry (collision triangles, bounds, radius) is a separate D3 service that the entity query consumes. This task does not move it.

## Next unmet gate

Compile the clean named fixture build, run the guard/stand-in tests and the native probes on fresh verified roots (Georgia, Lebanon, Lebanon repeat,
`seconds=0` and `seconds=120`, with and without probe), classify differences and gaps, and obtain independent review. Only then can the host handoff for
G3a be designed, and production G3 switching still waits for validated preceding delivery and freshly verified integration.
