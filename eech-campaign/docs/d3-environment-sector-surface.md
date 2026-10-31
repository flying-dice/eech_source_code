# D3 environment services (EEDCS-51): characterisation of the ORIGINAL terrain-sector surface enumeration (the sound routine's terrain query)

Status: **TEST SOURCE PREPARED. UNBUILT. UNRUN. UNREVIEWED.** Nothing here moves a solver or changes a production path. It is the FIRST step (characterisation of the original, on a named baseline) of the next
extraction under Task 51: the surface query and scratch that the session sound routine still solves inside EECH_DC. Task 47 delivered only the point, fast-elevation and map-facts subset of the
host's terrain; no claim of full terrain is made. Campaign time/calendar/scheduling policy stays in Core and nothing here says anything about physical time.

## The original, read from the live source

* `aphavoc/source/entity/special/session/ss_updt.c update_session_sound_effects` runs every session update, with or without sound children. In its "terrain" block it clamps the camera position to the
  map (`bound (camera_pos->x, MIN_MAP_X, MAX_MAP_X)`, the same for z) and calls `get_terrain_3d_types_in_sector (xpos, zpos)`, then hands the clamped position and the wind levels to the observer hook
  (`ENV_FXS`). It reads no result of the call itself: the effect is the shared scratch.
* `modules/3d/terrain/terrdata.c get_terrain_3d_types_in_sector (x, z)`: selects the sector with `get_terrain_3d_sector` (it sets the FPU rounding mode to zero and TRUNCATES `x / 2048` and `z / 2048`
  toward zero with `convert_float_to_int`), indexes `terrain_sectors[z_sec][x_sec]` WITHOUT a range check, `memset`s the global `int terrain_types_in_sector[TERRAIN_TYPE_LAST]` (62 ints) to zero,
  sets the entry of the sector's FIRST surface (`sector->surface_changes->surface_id`) to `TRUE`, and then, for each of the sector's `number_of_polygons` polygons, when `polygon->surface_change` is set advances
  to the next surface entry, sets ITS entry to `TRUE` and counts it. It returns `1 + the number of surface_change flags walked`: **the number of transitions plus one, not the number of distinct flags** (a surface
  met twice is flagged once and counted twice). The only other caller is `3drain.c`, not part of the headless campaign.
* Undefined behaviour in the original (so never called by the characterisation): a sector index outside the loaded grid (the clamp to `MAX_MAP_X`/`MAX_MAP_Z` may itself reach the end index; the
  characterisation reports whether it does), a NULL surface or polygon table, a `surface_id` of `TERRAIN_TYPE_LAST` or more (an out-of-range index into the scratch).

## The entry point (test-only)

`csrc/eech_env_fixture.c`, group 10 (`env_sector_case`), reached through the existing `engine:env_case (10, index)` of an `eech-dc` built with the `env-fixture` feature. It is **explicit only**: it is not in the
default run, the census, the witness set or the 164-case oracle, and the accepted drivers (`lua/env-characterization.lua`), baselines and tolerances are untouched. Index 0 is the plan
(`env-sector-plan version=1 terrain_loaded sector_x_max sector_z_max side terrain_types cases scratch_ints` and the map extents as IEEE words); indices 1..cases are the samples. For each sample it calls the ORIGINAL
function (nothing is recomputed) and prints, as stable raw integers and IEEE words:

`label x_w z_w sector_x sector_z polygons returned distinct_flags nonbinary flags repeated repeat_equal sentinel_cleared interleave interleave_other_returned interleave_returned interleave_equal restored`

* `returned` is the original's return value; `distinct_flags` is the number of scratch entries left set (counted from the scratch); `polygons` is the sector's raw polygon count: the three are kept apart.
* `flags` is the COMPLETE scratch as 62 characters (`0`, `1`, or `?` for any other value; `nonbinary` counts the `?`).
* `sector_x`/`sector_z` are the sector the original itself selected (`get_terrain_3d_sector`).
* Reset behaviour: before each call the scratch is filled with a different sentinel (7, then 9); `sentinel_cleared` says none survived, `repeated`/`repeat_equal` that an identical second call returned and left the same,
  and `interleave*` that a call on the first sample's sector between a call and its repeat changed nothing (`na` when that sector cannot be asked).
* `restored` says the scratch was put back byte for byte; `restored=0` is a defect of the fixture (the driver stops with `FIXTURE DEFECT`), never a result.
* A sample the original cannot be asked is **rejected at the boundary** with its reason (`sector_outside_the_loaded_grid`, `no_surface_table`, `no_polygon_table`,
  `surface_id_outside_the_terrain_types`, `not_finite`): the original is not called and nothing is repaired. The surface-id check reads the same pointers the original walks, only for the one value that would
  be undefined behaviour there.
* The samples: the centres of the first, x-last, z-last, last and middle sectors; a finite sweep of 48 sector centres across the loaded grid; the float below, at and above each lower edge and below and at each
  upper edge, in x and in z, around three sectors (first, middle, last); zero, negative zero, `-0.5` (which truncates to sector 0, not -1), `-2048` (sector -1, rejected), the end of the terrain minus 1 mm and at the
  end (rejected), and the sound routine's own clamp limits (`MIN_MAP`, middle, `MAX_MAP`).

The existing observer evidence is REUSED, not repeated: it already records the hash of this scratch inside real frames together with the shared random word, the entity indices and the time phase
(`docs/d3-environment-observer.md`). This group adds only the missing surface-enumeration semantics. The synthetic-looking cases (sentinels, boundary floats) are inputs to the real original, not expected values.

## The driver and its guards

`lua/env-sector-characterization.lua` boots the campaign (no frame is run: the loaded terrain is all the routine reads) and writes one strict JSON Lines row per sample (`boot`, `plan`, `sample`..., `summary`).
It refuses malformed lines and lines whose counts disagree with their own flags, treats a scratch that was not restored as a fixture defect, reports a repeat/sentinel/interleave difference as an `anomaly` of the
original (never repaired), and fails distinctly (`env-sector:` prefix) on a write, flush or close failure. `tests/env_sector_characterization.rs` runs the real driver against a scripted engine: it proves only what the
DRIVER does with the lines it is given (it computes no surface and mirrors no part of the original), and prints nothing about the DLL.

## Commands and the next gate (Root)

* Build the fixture DLL from the commit to characterise (clean checkout): `tools/build-windows-env-fixture.sh target/env-fixture/<dir>` (this is `--features eech-dc/env-fixture`; the production flags and volume are untouched).
  On Linux/for the unit file: `cargo test -p eech-world --test env_sector_characterization`, and clippy/fmt as for the other fixtures, with and without `--features eech-dc/env-fixture`.
* Collect the ORIGINAL facts on each of the three retail roots (Lebanon, Georgia, and the fresh Lebanon repeat; one fresh verified root per run, the engine writes into its root):
  `eech-world lua/env-sector-characterization.lua root=<verified retail root> out=<env-sector-<scenario>-<root>.jsonl> scenario=lebanon_retail` and the same with `scenario=georgia_retail`.
  Retain the outputs privately (they contain map data; none belongs in Git) with their hashes. The expectation to check first: every row's `restored` is `1`; the `plan` reports a loaded terrain; whether
  `sound_clamp_maximum` is answered or rejected (a rejection there is a finding about the original's own clamp, to be recorded, not repaired).
* After any host cutover of this query, run the same command on the same root with the same scenario and compare the `sample` rows (every field except `raw` is an integer or an IEEE word; `raw` repeats the fixture line):
  they must be identical, sample for sample, including the rejected ones. A difference is classified as a regression, an approved defect correction, an intentional semantic change or an input variation, and the old
  evidence is kept.
* No production move is made until Root has recorded the actual original results. Remaining Task 51/D3 work (lightning effects, physical time, the rest of terrain, spatial, model) stays in D3; no claim of completion.
