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
## Review `a78b6ab3c` of the original-sector characterization: one syntax correction (UNBUILT, UNRUN, UNREVIEWED)

The independent review (SHA-256 `f23a1f53...27797`) found one P1: the group-10 error string in `eech_env_fixture.c` (lines 1308-1309) spanned a literal newline, so the file did not compile. Actual results at the frozen
`a78b6ab3c` (Root's, preserved): the four new Linux driver guards passed, the Linux env-fixture compile exited 101 and the Windows env-fixture build exited 1, both at exactly that missing terminating quote; strict Clippy was
not reached; no retail sector query has run and the three fresh roots are unused. The string now ends `index=%d\n"` on one line (same wording, group, index and refusal behaviour); the whole file was scanned for another
unbalanced string and none was found. The semantics, cases, driver and protocol of this document are unchanged; the original facts are still to be collected by Root with the rebuilt fixture DLL.

**Strict-Clippy lint at `b2e55b18b` (UNBUILT, UNRUN, UNREVIEWED).** The narrow syntax-correction review (SHA-256 `9e702f33...e824`) closed the string finding. Actual (Root's): Windows env-fixture build/export 0 (48 payloads), Linux env-fixture native
compile 0, then the first affected strict Clippy 101 on `useless_vec` only at `env_sector_characterization.rs:147`; fixed by a fixed-size array, no suppression and no other change. The three actual original sector runs on fresh verified retail roots are
LIVE with the frozen `b2e55b18b` native; no result is claimed.


## The cutover of the sector surface enumeration to the host's terrain (production/client/provider batch; UNBUILT, UNRUN, UNREVIEWED)

**Baseline (Root's, closed, not repeated):** the unchanged original `get_terrain_3d_types_in_sector` on the hosted-field native `b2e55b18b` (Windows fixture build 0, 48 payloads; Linux native compile 0): three
retail runs (Georgia, Lebanon, Lebanon repeat) each native/controller 0 with six pre/post input manifests 0; each 95-row file has 92 samples (88 answered, 4 rejected as outside the grid), full 62-entry flags, every
restore/repeat/sentinel/interleave equal, 0 anomalies, `sound_clamp_maximum` answered; the Lebanon and repeat raw files byte-identical; no sampled retail sector returns a count that differs from its distinct
flags (independent source semantics stay: 1 + the surface changes, not the number of distinct flags; the 5-versus-3 row is bookkeeping only, not retail duplicate geometry). Scope: the original helper on the
hosted-field baseline, not the unhooked whole original DLL. The independent actual-evidence/new-guard review closed with no findings (SHA-256 `1408817f...6200`).

### Callers and writers, inspected (all of them)

| code | role | now |
|---|---|---|
| `get_terrain_3d_types_in_sector` (`modules/3d/terrain/terrdata.c`) | clears the scratch, flags the first surface and every polygon's surface, returns 1 + the changes | **host** (below) |
| `ss_updt.c update_session_sound_effects` | the only caller of the headless campaign: clamps the camera to the map, calls it once per update (even with no sound children), reads `terrain_types_in_sector[SEA, FOREST_TOP, RIVER, RESERVOIR, ALTERED_LAND1, ALTERED_LAND2]` to set the sound children's amplitudes | **campaign** (unchanged order, clamp and readers), now fed by the host's answer |
| `3drain.c` | reads `terrain_types_in_sector[ALTERED_LAND3]` | dormant in the headless campaign; reads the same mirror; **not inferred from any headless run** |
| `terrain_types_in_sector` (global `int[62]`) | the scratch | a by-value **mirror** of the last host reply, written entry for entry only after the reply was checked; no pointer or layout is exposed |
| `get_terrain_3d_number_of_type_triangles_in_sector`, `get_terrain_3d_type_triangles_in_sector` (the guide cover code, `gd_cover.c`) | per-type triangle count and list; the count function also writes TRUE into the same scratch (without clearing) | **unmoved campaign code** (named remaining terrain coupling); their scratch writes are the original's and keep their order |
| `eech_env_observer.c`, `eech_env_fixture.c` group 10 | read/hash the scratch; the fixture calls the function and restores the scratch | unchanged; reused as the like-for-like comparison (the fixture's rejection domain still reads the campaign's own loaded sectors only to reject what the original could not be asked) |

### Original versus host ownership

* **Host** (`eech-world/csrc/terrain_ref.c host_terrain_types_in_sector`, `src/terrain.rs`): the sector choice (the original's `x / 2048`, `z / 2048` truncated toward zero; an exact scaling and a truncation, so no floating-point
  environment is touched), its **domain** (a sector outside the map, a position that is not finite and a quotient that does not fit an int are refused with a message, where the original indexed its table unchecked), the
  walk of the sector's polygons over the host's own loaded, bounds-checked copy of the geometry (first surface, then the surface of EVERY polygon with `surface_change`), the reset (all 62 flags are rewritten), and the
  return value 1 + the changes met. A surface id outside the 62 terrain types, which the original would use to index outside its scratch, is refused.
* **Campaign** (`aphavoc/.../mb_hostterrain.c campaign_terrain_types_in_sector`, `terrdata.c` under `EECH_CAMPAIGN_HOST_TERRAIN`): builds the request (`op = SECTOR`, position, the season captured before the first question), asks the
  host, checks the answer in full (flags exactly 62 characters `0`/`1` and the terminator; at least one set; the return value not below the number set) BEFORE the mirror is touched, then writes the mirror and returns the return
  value. It computes and solves nothing, has no fallback to its own sectors, and any refusal, withheld or malformed answer is a fatal `host terrain: [terrain-provider:sector.*]` error. The original body stays in the `#else`
  for the original game's build.
* **Operation** (`mb_hostterrain.h`, `eech-engine-sys` mirror, `TerrainQuery::Sector` / `TerrainAnswer::Sector`, the Lua bridge): request `{ op = "sector", x, z, winter }`; reply `{ returned, flags }` with `flags` a string of one `0`
  or `1` per terrain type. The same checks run at three layers (the Lua bridge with markers `sector.flags` and `sector.returned`, the engine callback for a direct Rust provider, the C client), so a provider cannot hand the
  campaign an enumeration that is not one. The other operations (map facts, point with its opaque hints, fast), the season selection and the existing floating-point and boundary contracts are unchanged; the reply struct only
  gains a return value and a 64-byte flags field at its end (both sides updated together).
* **Untouched, named:** the campaign still loads its own sectors for the other sector readers above and for the rest of terrain (trees, object placement, loaders: remaining D3), and the per-type triangle functions are unmoved.

### What the new tests and controls can show, and what they cannot

* `terrain_sector.rs` (host kernel and provider, one-sector synthetic maps built from bytes, bookkeeping of a LOCAL contract): the return value is 1 + the changes and not the number of flags (a surface met twice is flagged once
  and counted twice), the first entry is flagged even when no face is typed by it, an empty sector gives 1, the sector is chosen by truncation toward zero (a small negative position is sector 0), outside positions and non-finite
  numbers are refused, a surface id outside the types is refused, and the Lua provider answers `{ returned, flags }`. It is not conformance with a real map.
* the engine and bridge unit tests (`eech-engine`, `eech-dc`): the answer travels by value and every malformed shape is refused under its marker before anything is written.
* `lua/env-sector-authority.lua` with `tests/env_sector_authority.rs` (needs an `eech-dc` built with `env-observer`): through the public path, **control** records the host's own answer to every sound update and requires exactly one
  request per update (the observer's sound-update count) and the real routine's scratch (hash of all 62 entries, number set, the clamped position) to be that answer; **changed_sector** sends a valid alternative enumeration for
  every request and requires the scratch the real routine read to be exactly what was sent, and different from the host's own (so the host's answer reaches the real consumer); **withheld_sector**,
  **malformed_sector_flags** and **malformed_sector_returned** must stop the engine at `[terrain-provider:sector.callback]`, `...sector.flags`, `...sector.returned` with the alteration recorded as sent. The altered evidence is kept
  apart from the default three-root 95-row original/host comparison. The judge's guards and the driver controls (a scripted engine) prove only bookkeeping and the driver's behaviour, not the DLL.
* **Not shown:** the sound amplitudes the routine then sets on the session's sound children are not observed (an extra accessor would be needed and is not added); the dormant renderer reader is not exercised and not inferred;
  the equivalence of the unaltered host enumeration with the original is the separate three-root comparison below; the other sector readers are unmoved and unobserved; nothing is claimed for a campaign with a player.
  After the arm the provider is the campaign's whole terrain world for the sector query: this is a bounded alternate-host-world input, not an isolation of every consumer.

### Commands for Root (nothing here was built or run)

1. Source checks: `cargo fmt --check`; `cargo test -p eech-engine`; `cargo test -p eech-dc`; `cargo test -p eech-world --test terrain_sector --test env_sector_authority --test env_sector_characterization --test terrain_backend`
   (and the whole `cargo test`); `cargo clippy --all-targets -- -D warnings`, with and without `--features eech-dc/env-fixture` and `--features eech-dc/env-observer`.
2. Builds: the shipping Windows module `tools/build-windows.sh target/<dir>`; the observer/fixture module (group 10 and the authority witness) `tools/build-windows-env-observer.sh target/env-observer/<dir>`.
3. The like-for-like comparison with the closed original, on the three fresh candidate roots (Georgia, Lebanon, Lebanon repeat), one run each with the observer/fixture build:
   `eech-world lua/env-sector-characterization.lua root=<fresh candidate root> out=<private jsonl> scenario=georgia_retail|lebanon_retail`; compare the `sample` rows with the `b2e55b18b` raw originals (every field except `raw` is
   an integer or an IEEE word; the `raw` line repeats the fixture line): they must be identical sample for sample, the four rejected ones included. Classify any difference (regression, approved correction, intentional change,
   input variation) and keep the old evidence.
4. The authority witness: `EECH_WORLD_BIN=<eech-world> EECH_SECTOR_ROOTS=<one fresh verified root per mode, `;` on Windows> [EECH_SECTOR_MODES=control,changed_sector,...] [EECH_SECTOR_SCENARIO=georgia_retail]
   [EECH_SECTOR_DIR=<private dir>] cargo test -p eech-world --test env_sector_authority -- --nocapture` (five modes need five fresh roots, the control first; the three candidate roots are for step 3).
5. The EEDCS-47 witness `lua/terrain-provider.lua` in its `generic` mode now answers the `sector` operation (a one-surface land world); re-run it where it is part of the gate.
Remaining Task 51/D3 after this batch: lightning effects and the strike/sound entities, physical time inputs, the rest of terrain (trees, loaders, the triangle functions), spatial/object and model services. Campaign
geography, route planning, decisions, scheduling and time/calendar policy stay in the campaign.
