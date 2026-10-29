Prebuild correction of the tree-transcript driver code (UNBUILT/UNRUN; STATUS.md has the full text): trace write, flush and close results are now checked and fail the run before
trace_written/done; any probe, read, check, repeat or I/O failure disarms the transcript, closes the file and re-raises the original error (a cleanup error is recorded, never substituted); and the
existence of trace_out is re-checked right after the output is created so a dir/./name alias of it is refused, not truncated (check-then-create, not claimed race-proof). Real-script controls cover
write, flush, close, disarm and alias. No physical, query or C change. File-format audit facts (8-byte records; Georgia 848187/2894 type-1, Lebanon 213955/371) are read-only inputs the trace must
still confirm natively. The host loader/kernel waits for Root's trace run and review.

Current-first: private tree-query transcript instrumentation (`trace_out`) is prepared for the first G3a extraction (the reference-host TREE collision solver and tree-file loader), UNBUILT and UNRUN.
It records each tree query in order with the original's own terrain lookups (passive wrapper at the two lookups in get_line_of_sight_collision_tree; the shipping expansion is unchanged), the
initial and final output bits, and a facts block (native 8-byte tree record layout and bit order, scales, forest triangles, raw live records). Bounded, private, fail-closed, off by default, report
identical either way. The correction phase and its closed review (native 5416adb3b, driver 54e5ece85) are inherited and not repeated; nothing is extracted; whole D3 active; production G2 not cut over;
D4/D5 not started. Root runs the seconds0 trace and compares all 205 report rows with 5416adb3b before the host kernel/loader work begins. See STATUS.md for the full design and bounds.

Current: source 5416adb3b moving 120s+30s witnesses all TERMINATED exit0 (digests equal, Lebanon reports identical, restoration/masks hold; Georgia 10 changed updated words / 62 cleared bits,
5 moving targets; Lebanon 6) as original observations with gaps, no valid oracle for the old d5 failures. Source 438940d39 Linux 9/9 + fmt + Clippy pass; its Windows run is in progress, not recorded as passed.
A review P2 in the driver feasibility check of `col-flags` is corrected in the real driver, UNBUILT/UNRUN: per array cleared_bits <= 32 x changed_words; sum over arrays of max(0, changed_words -
cleared_bits) <= added_bits; sum(cleared_bits) + added_bits <= 32 x sum(changed_words). added_bits is not forced to 0. Four new one-defect controls and one positive control were added (77 negative
controls). Review pending; no extraction; whole D3 active.

Windows guard result for d16c08bec (actual): 0/9 passed, every test failed with OS error 3 starting the host, because the test helper still used the compiled-in Linux CARGO_BIN_EXE path;
Linux d16 passed 9/9 with format and Clippy. A helper-only follow-up adds the runtime override EECH_WORLD_BIN (an existing file, else a clear failure; unset falls back to the cargo path) for the
collision driver tests only. It is UNBUILT/UNRUN until Root runs all 9 on Windows with the matching same-source eech-world.exe. Source 5416adb3b physical build and three boot probes passed
(205 rows, old 199 d5 row values matching, six new diagnostic rows separate); its moving probes are LIVE, not accepted. Review pending; whole D3 active.

Linux result for source 5416adb3b (actual, terminal, exit 101): format passed, 8 driver tests passed, 1 failed, Clippy not run. The failure was a stale test expectation
(common_cases_wrong expected 72; the validator and stand-in correctly give 44), not a physical regression. A follow-up corrects that and the second stale one (census_paths_wrong: 16, not 17)
after re-deriving all 73 negative expectations; it is test source only and UNRUN until Root executes it. The Windows physical build of 5416adb3b is LIVE, not passed. The old d5 failed
moving probes stay failed with no valid oracle; review remains changes required; whole D3 active.

Native checkpoint at clean d5ca5ee5b: Windows fixture build0/all43 exports verified;
three seconds0 probes on fresh verified roots native0/pre/post0,199 strict rows each,
repeat/restoration/RNG/candidate-state checks pass and separate Lebanon raw bytes
match. Moving/terrain-aligned/geometry-tree coverage gaps remain explicit. Five
120s+30s probe/control/repeat runs are LIVE; final results and combined review pending.
No Tacview/all-state/physical-extraction or shipping acceptance is inferred.
See reference/d3-collision-characterization/original-smokes-d5ca5ee5b.json.
The following is the earlier preparation/guard history.

Current execution checkpoint: eight actual Linux driver guard tests, formatting
and host all-target Clippy pass0; these use the stand-in and prove no physical
query. Tested hashes/exact log are in reference/d3-collision-characterization/.
The original C fixture/native retail runs remain unbuilt/unrun. Three fresh roots
are copied and pre-manifest verified. The following is the source-preparation
record from before those guards ran; live ledger is now v60.

# Observer defect (P1) and its source correction — UNBUILT, UNRUN

**What was measured.** The d5 long probes ended with `col-guard: world_unchanged is 0` after 1200 frames; the three zero-frame d5 smokes passed because nothing is flagged at boot. Those d5 results,
the failed moving probes and the coarse-only no-probe controls are retained as measured identities of the d5 source and are not re-labelled; the failed probes' raw measurements were not
retained (the driver discarded the report on validation failure) and there is **no valid moving oracle** to normalise.

**Cause (confirmed by source).** The d5 fingerprint (`world_fold`) read the updated flag with `get_local_entity_int_value (en, INT_TYPE_UPDATED)`. That getter is not a read:
`aphavoc/source/entity/mobile/mb_int.c` 398 does `value = updated_entities [index] &= mask` (moved 400 and rotated 402 are the same), which clears every OTHER bit of the entity's 32-entity
word in the bounded arrays defined in `up_update.c` 86 to 88 (`MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8)` words each, exposed unsized in `up_update.h`). The observer therefore changed the state the
original moving-target collision reads (`en_cllsn.c` 1330) before and after the measured queries. This is a defect of the extra observer, not a finding about a production mutation.

**What does NOT change.** The production getter and the original collision code are untouched. The original moving collision itself calls the same destructive getter, so the original
queries legitimately change these arrays; that effect stays inside the measured query sequence, in the same order, and is now measured instead of hidden.

**Source correction (this working tree).**
* The fingerprint reads the RAW updated bit, `(updated_entities [index >> 5] >> (index & 31)) & 1`, with the original's own index arithmetic and no getter. The fixture no longer calls
  the destructive getter anywhere.
* Before anything is observed the three arrays are copied in full (`flags_save`). After the queries the original's effect on the live arrays is measured against the copy
  (`flags_measure`: words changed, bits cleared, bits added; added bits are a finding, the original only clears) and the copy is written back (`flags_restore`) on every exit after the
  copy and at the end. The final fingerprint is taken only AFTER the restoration, so any other position, motion or list change is still caught; nothing else is restored to suppress a
  difference, the shared terrain record is restored exactly as before, and the game random stream is not reset.
* The result is reported as one LEGACY-diagnostic row, `col-flags` (`restored`, `words`, `before`, `after_queries`, `after_restore` as hashes; the per-array changed-word and cleared-bit counts;
  `added_bits`). No array, pointer or layout is in the report. The driver requires `restored=1`, `after_restore == before`, per-array changed words within `words`, changed words
  accounted for by cleared or added bits, and the hashes agreeing with the counts. The copy exists for the extra observer only; it is not a production semantic.
* **Failure retention.** When validation rejects a real report, `report_invalid` now carries the exact text the fixture returned as `raw_report` (JSON-escaped, in the private output), and a
  repeat that differs carries the differing report in `repeat_identical`. A malformed report is still never accepted and no physical rule is changed. Raw reports stay out of Git.
* Guarded by real-script tests without a second validator: seven `col-flags` negative controls and a strict-JSON round trip of a retained report with quote, backslash, control byte and UTF-8.

**Scope that remains precise.** The fingerprint still excludes the children of a CITY and any per-entity identity; the primary warm counters still do not cover the subsidiary near/warm queries.
Nothing here claims complete state purity. The original warm/cold answers of a moving target can legitimately differ because an earlier query cleared the bit that a later query reads;
that is an original behaviour the warm counters record, not something this correction normalises. Whether the zero-frame d5 physical folds, samples and counters still match after
the correction is for Root's run; it is not claimed here.

# Reviewer corrections to the d5 fixture — SOURCE ONLY, UNBUILT, UNRUN

One combined independent review of pinned `d5ca5ee5b` found four P2 defects and one P3 scope issue. They are corrected in source after the pinned d5 build and smokes
(which stay measured identities of the d5 source and are not re-labelled). Nothing below has been compiled or executed; the live d5 long runs are not a pass of the
corrected source. The original queries, the structured and random case generators, their order and the physical folds are unchanged.

1. **Exact hit-type conservation.** The fixture emits every positive `col-entity-type` bucket, so the driver now requires each group's bucket hits to add up to
   EXACTLY the group's hit count (it accepted `<=`, so a deleted bucket, or all of them, reached `done`). Negative controls: one struct bucket deleted, one random bucket
   deleted, all buckets omitted, and a bucket over-count. The valid stand-in now has buckets that sum to its 12 structured hits.
2. **Exact target selection.** The structured targets are the first 3 of each class in scan order, so each selected count must be EXACTLY `min(class census, 3)`. The
   report previously gave only totals, which cannot express that, so a new row `col-census-class` carries the census of each of the 8 entity classes and 2 tree classes (every
   candidate seen, not only those selected). The driver checks that it agrees with `col-census-entity` and `col-census-tree` (category totals, geometry/box path totals, tree
   split by `forest_geometry`) and that `col-targets` equals `min(class census, 3)` for all ten; a positive census with zero or under-selected targets is a defective report, not a coverage
   gap. The valid stand-in was impossible (4 stationary candidates, 1+1 selected; 40/60 trees, 2/1 selected); it now has a coherent census in which five classes exceed the quota.
3. **Verified floating-point mask contract; derived axis rays.** At probe entry the fixture READS (`fnstcw`, `stmxcsr`; never writes) the x87 control word masks (bits 0 to 5) and the SSE
   MXCSR masks (bits 7 to 12) and REQUIRES all six in both (63 and 63); otherwise the probe is refused (`x87_exceptions_not_all_masked`, `sse_exceptions_not_all_masked`, or
   `fp_masks_unreadable_on_this_architecture` off x86). Only mask bits are inspected, not sticky status flags, rounding or precision, and the existing round-toward-zero check stays.
   The facts are reported (`col-fp arch x87_masks sse_masks`) and re-checked by the driver. The old d5 reports have no such fact: **d5 did not check the masks, and nothing here infers that it
   ran with them masked.** The old claim "never queries a tree ray with zero x or z extent" was only true of the PRIMARY random tree rays (those are still skipped and counted as `skipped_axis`,
   which keeps the random case set unchanged). The nearer-check ray cut short of a returned f32 hit is a DERIVED ray: at large coordinates its x or z extent can be exactly zero while its range is
   above 0.1 m, so it reaches the original's division (`en_cllsn.c` 2021 and 2024 at 03413ccd3; 2035 and 2038 after the transcript hook added 14 lines). It is still queried, never screened or skipped, and is now counted: `derived_axis` (a new field of
   `col-entity-common`, always 0, and `col-tree-common`; the driver requires `derived_axis <= nearer_checked`). The deliberate vertical and sub-0.1 m structured tree rays remain
   range-gated cases (the gate returns before the division; the 0.05 m offset can round to zero extent at large coordinates). With the masks required, a zero-extent division is IEEE-defined
   (inf or NaN, no trap). That covers the rays counted; **a source check of axis extents alone is not a statement of the original's full physics.**
4. **Portable real-script guards.** The Rust tests embed the ACTUAL `collision-characterization.lua` with `include_str!`, copy it to a private file in the test directory and the
   stand-in loads that copy (`@DRIVER@`). The previous `@LUA_DIR@` built from `CARGO_MANIFEST_DIR` named the source machine's path inside a cross-built Windows test. There is no duplicate
   validator; Linux guards alone do not prove the Windows executable, which Root runs natively.
5. **P3 — honest scope of the repeat and state guards.** The sector-list fingerprint (`world_before`/`world_after`, flag `world_unchanged`; names kept) covers, in sector order, a marker per
   sector and per sector-list entity in list order its type and collision-candidate flags, for a candidate its position, for a mobile its motion vector and updated flag, and the frame delta time.
   It does **not** traverse the buildings of a city and hashes no per-entity identity. The warm counters (`warm_same`/`warm_different`) compare only the PRIMARY cold query with its warm repeat;
   the warm repeat of the shortened nearer-check ray is made but not compared. Byte-identical repeated reports plus a coarse state digest are repeatability and bounded observations, not complete
   candidate-state purity, all-state equivalence or Tacview equivalence. The driver's failure texts say so (`the sector-list fingerprint changed during the probe`; `the report is not
   repeatable for the seed and the loaded world`). The old d5 raw reports are preserved; this is a correction of what they may be claimed to show, not a finding that anything mutated at runtime.

New report rows and fields (metadata, not physical): `col-census-class`, `col-fp`, `derived_axis`. Root compares the physical folds, samples and counters of the corrected build with the
retained d5 and separates those additions.

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

*Structured* (derived from the loaded world only; no random draw). Targets are real entities from the real sector lists in scan order, the first 3 of
each of 8 classes: moving / stationary mobile, fixed object, city building, each with / without collision-geometry triangles; and real trees from the
real tree sectors, the first 3 of each of 2 classes: quad-plane and geometry. Per entity target: a ray through its centre, the same ray reversed, a lateral
miss, and a ray over it; on the through ray, source/target/both exclusion of the target, a real decoy, the owning city of a building, and the chain of
hits found by excluding each hit in turn (up to three objects). Per tree: the through ray under all four combinations of the two flags, then reverse,
over, under, lateral, slant, high (above the 35 m gate), vertical and sub-0.1 m rays. Plus two edge rays per family across the map boundary (supported:
the entity scan clamps to the map, the tree query has an explicit outside-map branch).

*Random*: `entity_cases` and `tree_cases` seeded cases around entities/trees from a private reservoir sample (seed-derived; the game's random stream word is
compared before and after and must not change).

### What each query records

Hit/miss; hit kind, hosted-id text and position; the intercept point (always, as the consumers report it) and the face normal (hits only), as float bit
patterns; whether a miss wrote the point or the normal; whether the cold query touched the shared terrain record; a **warm repeat** on exactly the
record the cold query left (a different warm answer is counted as cache state); and, for each hit, a **nearer-hit check** (the same query cut 0.1 % short
of the reported point; a hit there proves the first answer was not the nearest — the original asked a second question, not a model of it).

### Report schema (`key=value` text lines → one JSON Lines event each)

`col-probe`, `col-map`, `col-guard`, `col-fp`, `col-census-entity`, `col-census-tree`, `col-census-class`, `col-targets`, `col-entity-kind` ×7 (struct: through, reverse, lateral,
above, exclusion, edge; random), `col-entity-exclusion`, `col-entity-common` ×2, `col-entity-type` ×0..n, `col-tree-kind` ×11, `col-tree-flags` ×8,
`col-tree-common` ×2, `col-physical`, `col-legacy`, `col-sample` ×≤(16+8)×2. The driver accepts a report only as a whole: exact row set, exact keys,
exact types (decimal count / flag / signed / 16- or 8-digit lowercase hex / three-word hex), counts that agree with the request and with each other
(see `cross_check`; selected targets exactly `min(class census, 3)`, type-bucket hits exactly the hit count, both FP mask sets 63), sample counts equal to the quota or to the number of cases if fewer, and a header equal to the request. Anything else is
`report_invalid` and fails the run; nothing is read as zero.

Three kinds of fact are kept apart: **PHYSICAL** (`col-physical`: inputs, exclusion role, hit flag, hit kind/hosted id/position, point, normal),
**cache/short-circuit counters** (record touched, warm agreement, miss-time writes, nearest checks), and **LEGACY** (`col-legacy`: the hit's heap
slot and the terrain record's private bookkeeping — comparable only with a build that keeps both; never promoted to a physical claim). No pointer is
emitted.

### State checked, and its true scope

The shared terrain point record is snapshotted and restored byte for byte (`record_restored`, `record_words_before/after`); the game's random stream
is read before and after (`rng_unchanged`); a sector-list fingerprint (entity type, collision flags, candidates' position, mobiles' motion and updated flag, the frame delta time; not
city buildings, not identities) is compared before and after (`world_unchanged`). The driver fails the run if any of the three does not hold. These are bounded observations, not a proof that
nothing else changed. The probe is not claimed read-only
by name: it **does** write the shared terrain record during each query, which is why each cold query starts from a known sentinel record.

### Unsupported / never probed (declared)

NaN or infinite coordinates; a PRIMARY random tree ray with exactly zero x or z extent (skipped and counted as `skipped_axis`); any run in which the six floating-point exceptions are not all masked in both the x87 control word and the SSE MXCSR (refused; the masks are read, never set). Zero-extent rays that are queried and only counted: the deliberate vertical and sub-0.1 m tree rays (range-gated) and the derived shortened nearer-check rays (`derived_axis`);
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
| Reviewer corrections 1 to 5 above (exact type and selection checks, FP mask precondition, derived-axis counter, embedded real driver, wording) | source only: **UNBUILT, UNRUN**; the d5 build and smokes are not a pass of them |

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
# Current native result (29 October 2026)

Clean fixture source `5416adb3b` built successfully and passed all three boot
probes with both FP masks 63 and terrain/update-flag restoration. All 199 original
boot-row values, folds and samples match the retained d5 measurements; six new
diagnostic rows are separate. The 205-row Lebanon reports repeat byte for byte,
and all three pre/post manifests pass. Moving probes and controls are running;
there is still no valid old moving oracle. Two stale test diagnostic strings are
corrected below, with their new Linux/Windows validation and independent review
pending. The earlier status paragraphs are historical checkpoints.
