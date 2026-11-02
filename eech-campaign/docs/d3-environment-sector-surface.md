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

* **Host** (`eech-world/csrc/terrain_ref.c host_terrain_types_in_sector`, `src/terrain.rs`): the sector choice (the original's `x / 2048`, `z / 2048` truncated toward zero. CORRECTED (independent review ENV-SECTOR-HOST-1): the earlier claim that an exact power-of-two scaling and a truncation cannot touch the floating-point
  environment is false -- the smallest subnormal over 2048 is inexact/underflows, an ordinary fractional quotient sets the precision flag on truncation, x87 loads and conversions change the status word,
  and the refusal paths are unguarded too -- so the entry owes the caller's COMPLETE x87 and SSE state back on every path; the guard is written in the correction batch at the end of this document, UNBUILT), its **domain** (a sector outside the map, a position that is not finite and a quotient that does not fit an int are refused with a message, where the original indexed its table unchecked), the
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

## Independent source review of 6711d5f11 and the red (test/driver) phase (UNBUILT, UNRUN, UNREVIEWED)

Review `ENV-SECTOR-HOST-6711-20261031` (SHA-256 `06419e7b...80b5`, CHANGES_REQUIRED, source review only): two grouped findings and one compile error from the Linux pipeline. The original-sector
evidence (three roots, `b2e55b18b`, independent review `1408817f...6200`) stays CLOSED and is reused, not re-run; no wind, field or original experiment is repeated.

* **ENV-SECTOR-HOST-1 (P1), the kernel's caller-FP boundary, is NOT FIXED in this phase.** `host_terrain_types_in_sector` does finite classification, a float division, comparisons and a float-to-int
  conversion with no save/restore of the caller's x87/SSE state, and its comment and the first version of this document said none was needed. That is wrong (see the corrected sentence above). THIS PHASE
  adds only the red test: `the_callers_complete_floating_point_state_survives_every_sector_query_in_every_mode` in `crates/eech-world/tests/terrain_sector.rs`. It calls the PUBLIC library entry
  `Terrain::types_in_sector` (so it survives a later correction of the private C ABI), accepted and refused, under a seeded caller state -- every pair of x87 and SSE rounding controls, two x87 precision
  controls, MXCSR sticky flags clear and set, FTZ/DAZ clear and set where `MXCSR_MASK` has them, and three x87 status words (clear; C3 and every exception flag; stack top 4 with C0, C1, C2) -- and
  requires the complete x87 control/status/tag words and the whole of MXCSR to be exactly the seeded values after each call (no mask, C1 included), and the answers to be the same whatever the state. The
  positions are ordinary fractional coordinates, the smallest, the largest and a negative subnormal, a subnormal beside a normal, the minimum normal, signed zero, a negative fraction, and the refusals
  (outside the map either side; NaN; infinity; a quotient that does not fit an integer; a surface id outside the types). Mismatches are collected and reported by kind of call, with the first seed they
  happened under. It reuses the field guard test's technique (`env.rs`, a private module, so the helpers are repeated). EXPECTED RESULT AGAINST 6711d5f11: red (at least the MXCSR sticky precision/underflow
  flags, with sticky clear, from the inexact truncation and the subnormal quotient; the x87 words on an x87-only target). That is an expectation, not evidence, until Root runs it. What it cannot show: the
  surrounding Rust runs in the seeded state too; the private C entry's argument loads before a guard are covered only as far as the public call exercises them; the FTZ/DAZ cases cannot detect a leak if the kernel
  sets no mode, and only protect a later guard from leaving a mode behind; nothing about the Core client or the shipping module.
* **ENV-SECTOR-HOST-2 (P2), incomplete observer evidence, is fixed in the driver and judge** (test/driver files only). `lua/env-sector-authority.lua` now drains the whole event stream first and then reads the
  report, as the closed characterization driver does, and refuses (`witness_failure`, no `authority` row) a stream whose numbering has a gap or repeat or does not start at boot_begin, a kind outside the observer's
  30, a shared seed word that is not one word, a sound-fact first half not immediately followed by its second half, a second half alone, a first half alone at the end (truncated), a count above 62 or an impossible
  in_cockpit; and a report that is not the one real line, is not armed, dropped events or overflowed, has a capacity beyond the store or below what it stored, whose stored/read/next_seq are not exactly the events
  received, whose sound calls are not the session updates, or whose sound records are not the pairs found. The `authority` row publishes the observer's own counters (armed, capacity, stored, read, next_seq,
  dropped, overflow, events received, update_server; -1 for a run the engine stopped, which cannot be asked -- unchanged disclosure). The Rust judge (`tests/env_sector_authority.rs`) requires every counter
  (the sound records, once bound and never looked at, included), checks the same relations again, and refuses a changed run whose completed frames are not its configured frames. Guards: synthetic-row cases for every
  relation in the judge, and driver cases against the scripted engine for every stream/report defect above plus two-record positive controls. The scope is unchanged: only the FIRST record is published and judged
  (hash/count/position); this proves the stream was complete, not that every later scratch was compared, and it proves nothing about the DLL (the scripted engine invents no physics).
* **Compile error:** `crates/eech-dc/src/module.rs` (test only) passed a `char` where `replacen` takes a `&str`; it now passes `"2"`. No production semantics changed.

Commands for Root (nothing here was built or run; the Core C module need not be rebuilt for this test-only phase):

1. The red FP test against the unchanged 6711d5f11 kernel: `cargo test -p eech-world --test terrain_sector the_callers_complete_floating_point_state_survives_every_sector_query_in_every_mode -- --nocapture`
   (the other tests of that file are the green host-contract ones). A pass would mean the test cannot see the defect and must be examined before the production fix, not relied on.
2. The observer guards: `cargo test -p eech-world --test env_sector_authority` (the judge guards and the driver controls; the public witness prints SKIPPED without its environment) and
   `cargo test -p eech-dc` (the compile fix); then `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` (the new tests are written for the 160-column rustfmt, which Root runs).
3. Fresh roots are needed for any later public run after the production FP fix; a run of the current default 95-row candidate collection against the verified 6711 native is neither acceptance nor FP proof.

## Correction batch on e05bf775f: the sector kernel's complete caller-FP guard and the two actual observer-guard failures (UNBUILT, UNRUN, UNREVIEWED)

**Actual evidence this batch answers (Root's, not mine):** the focused public-call red `the_callers_complete_floating_point_state_survives_every_sector_query_in_every_mode` ran on Linux x86-64 against the
UNCHANGED 6711d5f11 kernel (e05bf775f is production-equivalent to 6711) and failed with `the sector query changed the caller's floating-point state in 76 kinds of call (6144 calls checked, no mask)`:
MXCSR leaks only (e.g. fractional (1, 1): `0x1f80 -> 0x1fa0`, the precision flag of an inexact truncation; a subnormal beside a normal: `0x1f80 -> 0x1fb2`, precision + underflow + denormal); the answers
did not vary. It is a Linux 64-bit observation: no 32-bit/x87 failure and no generic DLL defect is claimed. The retained log is `environment-sector-red-e05bf775f-linux-20261031.log`. The
`env_sector_authority` guard file ran 13 functions: 11 passed, 2 failed, both in the TESTS' bookkeeping (no product path; the public witness printed SKIPPED).
The first default 95-row candidate runs against 6711 (numeric only, receipt `environment-sector-first-6711d5f11-comparison-20261031.json`) are unchanged and say nothing about floating-point state.

**P1, the production fix (`eech-world/csrc/terrain_ref.c`, `terrain_ref.h`, `src/terrain.rs`).** `host_terrain_types_in_sector` now owns the caller's COMPLETE state around everything it does, the
reference host's established guard copied for this one entry (env_ref.c `fp_save/enter/restore`, itself from the closed model and ground components; not the MinGW fenv-only pattern, which leaves MXCSR
where the kernel put it):

* BEFORE the first floating-point action -- the argument validation and every refusal included -- the x87 environment (control, status and tag words, exception pointers: `fnstenv`, reloaded at once) and
  the whole of MXCSR (`stmxcsr`) are saved; round-toward-zero is set for both units and read back; all the arithmetic (the division by the sector side, the magnitude test, the truncation, the walk) runs in
  one non-inlined function between the save and the restore, writing only a LOCAL staged answer (return value, 62 flags, error text); the caller's state is restored (`fldenv`, `ldmxcsr`) on EVERY exit and
  read back COMPLETE, no mask (condition codes, stack top, tag word, every MXCSR bit); a restore that does not read back withholds the answer; only then are the caller's buffers written, by integer
  tests and bitwise copies.
* The position crosses the C boundary as the BITS of the two floats (`uint32_t x_bits, z_bits`, passed by the Rust wrapper as `to_bits()`), not as `float` parameters: a 32-bit callee may home a float
  through the x87 stack in its prologue, before any guard could run. The finite test is made on the bits (all-ones exponent), the bits become floats inside the guard, and they go through an empty
  `asm` after the guard is entered so that no arithmetic that depends on them can be scheduled before it. No legacy pointer or layout is exposed; the ABI is private to this library and its wrapper.
* UNCHANGED: the sector selection (division by 2048, truncation toward zero), the domain (every finite SUBNORMAL is still admitted and truncates to sector 0, like any finite position whose truncated sector is in the map; finite coordinates outside the grid and huge ones whose quotient does not fit an integer are still refused; the refusals are the same
  ones with the same messages), the enumeration (first surface, then every polygon's surface change), the return value 1 + the changes, the zeroed flags and return value on a refusal. The
  false comment that the scaling and truncation cannot touch the floating-point environment is replaced in the kernel and in the header.
* NOT done: no change to the point, fast or env kernels, to the Core client, the Lua provider's own conversions (the Lua number to `f32` happens in the caller's state, before this entry), or any
  DLL-wide floating-point behaviour; no mask or skipped assertion in the red test.

**P2, the two actual guard failures (test bookkeeping only; the completeness contract is NOT relaxed).**

1. `a_refusal_counts_only_at_the_named_marker_after_a_sent_matching_alteration`: the "engine did not stop" case turned a refused synthetic run into a COMPLETED one but kept the refusal base's lack of a
   pair row, so the judge rightly rejected `observer counted 1 sound records for 30 calls but the stream holds 0 pairs`. The case now carries the one pair the completed base counts, so it is a coherent
   completed run; its assertion (`did not stop`) is unchanged. Unknown counters of a stopped run are still -1, never measured zero.
2. `the_driver_publishes_an_observer_record_only_when_the_whole_stream_and_report_are_complete_and_reconcile`: the helper required the detail directly after `env-sector-authority: `, but the driver's
   message is `env-sector-authority: OBSERVER SEQUENCE: the first event is kind 2, not boot_begin ...`. The helper now asserts BOTH the witness classification (`env-sector-authority: OBSERVER`, never an
   engine stop) and the specific detail anywhere in the message; the no-`authority`-row, `witness_failure` and strict-parse-failure assertions are unchanged, and every case still runs.

**Still not shown:** that the guard works (nothing here was compiled or run); that the 32-bit/x87 path is clean; the real Core client or the shipping module under the new ABI; any later published-record
comparison beyond the first record; the sound amplitudes, the renderer, a player. Fresh roots are needed for every later public run; the 6711 roots/binaries and the first comparison are preserved.

Commands for Root:

1. `cargo test -p eech-world --test terrain_sector` (the SAME 6144-call test must turn green on Linux x86-64 and on the applicable 32-bit/Windows jobs; the other tests of the file are the host-contract ones).
2. `cargo test -p eech-world --test env_sector_authority` (13 functions) and `cargo test -p eech-dc`; `cargo fmt --check`; `cargo clippy --all-targets -- -D warnings`; the complete `cargo test`.
3. Rebuild the native observer/fixture module from the corrected source into a NEW directory (the frozen 6711 native is not overwritten), then the three default 95-row comparisons on the fresh FIXED roots
   against the `b2e` originals and the five authority modes on the fresh authority roots (`EECH_SECTOR_ROOTS`), exactly as in the commands above; then a new independent review.

## Residual P2 of the correction review on 14192c6e2: the configured frame count (test judge only; UNBUILT, UNRUN)

Independent correction review `ENV-SECTOR-HOST-14192-CORRECTION` (SHA-256 `000b5f69...95`): P1 (the kernel's caller-FP design) closes in source; the observer stream/gap/counter fixes hold, and the two
fixture fixes weakened nothing. ONE residual P2 remains, in the Rust judge only: `Facts::parse` never read the boot row's `frames`, so a completed run could be judged against a rewritten count (the terminal row
saying `frames = frames_wanted = 20` with 30 sound calls, against `boot.frames = 30`; or the boot count edited or removed). Fixed in `tests/env_sector_authority.rs`: `Facts::parse` now requires `boot.frames` as a
whole positive number equal to `authority.frames_wanted`, for healthy and stopped runs, before any `Facts` exist; the existing completed-frames equality (`frames == frames_wanted`, control and changed) then protects
every configured frame. New guard: `the_configured_boot_frame_count_must_be_present_valid_and_the_same_as_the_authority_rows_count` (the 20-of-20 against boot 30 forgery; the boot count rewritten to 40 or 20; missing,
string, fractional, null and negative boot counts; both rows agreeing on 0; the same for a stopped run; a positive control that a run configured for 20 and completing 20 is still a healthy control). The embedded
`env-sector-authority.lua` already writes the right `boot.frames` and is unchanged, and so is all production source (C, Rust, ABI, provider, default Lua): this is a test-judge change, so the pinned 14192 native and the
embedded Lua can be used with explicit provenance and the native need not be rebuilt for it.

Wording corrected (no numerical change): the correction batch said the domain admits "every finite value and every subnormal ... truncates to sector 0". The intent, and the behaviour, is: every finite SUBNORMAL is
admitted and truncates to sector 0; finite coordinates outside the grid, and huge ones whose quotient does not fit an integer, are still refused.

**Actual results Root has reported on 14192 (not mine; status of each stated as given):** the Linux fast stage passes all 5 `terrain_sector` tests, including the UNCHANGED 6144-call full CW/SW/TW/MXCSR regression, and all 13
`env_sector_authority` functions pass (the public path printed SKIPPED without its environment: not authority evidence). The earlier actual red (6144 calls, 76 kinds of MXCSR leak on Linux x86-64) is retained. The
Linux full default+observer package/strict pipeline and the Windows observer native + test exports were still running; there is no final exit and no native/public claim yet. The first 6711 numeric 95-row files are
exactly the originals, and say nothing about the 6711 kernel's FP state, which is unaccepted. The fresh fixed (3) and authority (5) roots are verified and UNUSED.

Commands for Root: `cargo test -p eech-world --test env_sector_authority` (14 functions now), then `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`; the fast and full stages already running cover
the rest. Nothing else changes: no original, field, model or kernel experiment and no FP regression rerun is needed because of this change.

## Two strict-Clippy test lints on 05b474e88 (test-only; UNBUILT, UNRUN)

Actual 14192 Linux full pipeline (Root, overall 101): the default and env-observer affected-package test phases pass; the first strict Clippy stage failed only on (1) `terrain_sector.rs` `type_complexity` for the reference
vector of the 6144-call regression and (2) `env_sector_authority.rs` `manual_range_contains` for the observer-capacity test of the judge; the observer strict stage was not reached. Both are corrected without suppression:

1. `type SectorAnswer = Result<(i32, Vec<bool>), String>;` declared in the test function; `reference` is `Vec<Option<SectorAnswer>>`. Same type, so the matrix (6144 calls: rounding pairs, precision controls, stickies,
   FTZ/DAZ, status words, every position and refusal), the seeds, the collected-mismatch reporting and every assertion are untouched.
2. `if !(1..=OBSERVER_CAPACITY).contains(&capacity) || capacity < stored`: exactly the values the old `capacity < 1 || capacity > OBSERVER_CAPACITY || capacity < stored` refused (below 1, above the store's 16384, or below what
   was stored), same message. The embedded Lua's own check is a different file and is unchanged.

No C, header, Rust library source, ABI, provider, Lua, FP logic or domain changed, so the actual 05b14 guards and any public run under the pinned 14192 native and embedded Lua remain valid under this semantic equivalence; Root runs the
new strict/guard checks. The subsequent Windows 05b attempt failed 13/14 at the timeout control before any real public mode or PRE check; five POST checks passed. All three corrected default 95-row cases are now exact to the originals, as recorded below.

Commands for Root: `cargo clippy --all-targets -- -D warnings` (default, then `--features eech-dc/env-observer`), `cargo fmt --check`, `cargo test -p eech-world --test terrain_sector --test env_sector_authority`.

## Windows judge-guard fixture failure on 05b474e88: the hung-child control's process shape (test-only; UNBUILT, UNRUN)

**Actual (Root's, factual):** the new 05b judge compiled and exported 0 on Windows with the unchanged 14192 native and embedded Lua; `env_sector_authority.exe` then ran 14 tests: 13 passed, 1 FAILED --
`the_wall_limit_is_a_finite_number_of_seconds_from_one_to_14400_and_a_hung_child_is_killed`, `the hung child was waited for: 29.337829s` (log `environment-sector-five-modes-controller-05b474e88-20261031/actual-Windows-judge-guards.log`).
All 13 other bookkeeping functions passed, including the boot-frame coherence guard. No campaign mode and no preflight had started; the real public path was NOT RUN (the guard run finished 101, then only the five
post-input checks, 0). The candidate roots for it are UNUSED. This is a defect of the test fixture's process shape, not a native or campaign result, and not a pass of anything public. (Root's earlier controller receipt
that counted "ten" pre/post checks was vacuously true with no PRE stages; Root preserves it with an audit and requires exactly 5 pre + 5 post in a corrected private helper. This document claims neither ten checks nor
any actual five-mode pass.)

**Cause.** On Windows the control spawned `cmd /C "ping -n 30 127.0.0.1 > NUL"`. The runner killed `cmd`, but the `ping` descendant inherited and kept the stdout/stderr pipes open, so joining the pipe readers waited
for it (~29 s, against the 25 s safety assertion). Linux used `sleep 30` directly (one process) and passed.

**Fix (`tests/env_sector_authority.rs`, test-only).** The control now uses the pattern of the closed `env_wind_authority.rs`: the test executable itself is the child (`std::env::current_exe()` re-run with
`--exact the_child_of_the_bounded_wait_controls`, `ENV_SECTOR_CHILD=hang|fail`), so there is ONE process on every OS and no shell, `ping`, `sleep` or grandchild. Kept exactly: the finite 1-14400 s wall parser cases, the
2-second wall, the 25-second safety assertion. Added assertions: the error classification (`outlived its wall limit of 2 s`), the child's own diagnostic in the report, the child shown to be stopped (`kill: Ok(())`
and `wait: Ok(`), at least the 2-second wall actually waited, and a second control that a failing child (exit 3) is read to its end with its diagnostic and is not a timeout. The helper test is a no-op without its
environment variable. No skip, allow or tolerance changes; the `run_bounded` code is unchanged except for a doc comment that states its scope honestly: it kills the ONE child it started (the public witness's target is one
`eech-world` process), it does not manage a process tree, and no job-object or process-manager framework is added. A descendant holding inherited pipes would still delay the join; nothing here claims otherwise.

The two strict-Clippy fixes (the `SectorAnswer` alias in `terrain_sector.rs`, the equivalent `contains` range in the capacity predicate) are preserved. `env_sector_authority` now has 15 test functions (the 14 that ran on Windows, one of them rewritten,
plus the child helper, a no-op without its environment variable). Commands for Root: `cargo test -p eech-world --test env_sector_authority` (the wall control takes about 2 s), `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings` (default, then `--features eech-dc/env-observer`); then the Windows guards and the real five modes under the pinned 14192 native with explicit semantic equivalence.

**Other actuals as reported by Root (not mine):** the corrected 14192 default 95-row cases, all three roots: native/controller 0, all six inputs 0, 49 payloads unchanged, 95 raw lines and endings EXACT against the `b2e`
originals (receipt `environment-sector-fixed-14192c6e2-comparison-20261031.json`); Windows libs 78/22/251 all 0, sector 5 (incl. the unchanged 6144-call FP regression) and characterization 4 pass; Linux default and
observer package tests pass; the e05 red (6144 calls, 76 kinds of MXCSR leak) and the 6711 numeric files stay retained. There is no actual sector host-authority result yet; Task 51/D3 stays open.
