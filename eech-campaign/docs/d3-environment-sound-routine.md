# D3 — the ambient sound routine of the session: original characterization (prep; UNBUILT, UNRUN, UNREVIEWED)

EEDCS-51 / EEDCS-21 (D3). Base: clean pushed `9c47b972a` (production kernel `14192c6e2`, sector enumeration host-owned and reviewed). **This document and its source only prepare the
ORIGINAL characterization of one more physical environment solver. Nothing here is built, run or reviewed; no production code changes; no physical move happens until Root has run and reviewed the
original results.** The solver's source is the current `14192`/`9c47` tree (`ss_updt.c`'s ambient body is unchanged since the original); that is the original of THIS solver, not the whole-original
DLL and not the sector original `b2e` (whose terrain scratch it reads). Reused, not repeated: the observer/driver 235 evidence of natural server updates, lightning, shared-RNG order and the sound
facts; group 10 (sector query and scratch); the closed wind/weather field evidence.

## Source semantics (`aphavoc/source/entity/special/session/ss_updt.c`, `update_session_sound_effects`)

Called from `update_server` after the time-of-day, resync and weather updates (and counted by the observer). Per call:

1. **Point and altitude.** `in_cockpit`: the gunship entity's position and `FLOAT_TYPE_RADAR_ALTITUDE`; otherwise the camera entity's (the camera's radar altitude is `max (y - cached terrain elevation, 0)`).
2. **Rain.** `rain_sound_levels[WEATHERMODE_LAST]` zeroed. If `camera.y < get_cloud_3d_base_height ()` and not (`!in_cockpit && alt >= 180`): `get_session_weather_at_point` gives the current mode,
   target mode and transitional state `trans`; equal modes: that level 1.0; otherwise target `trans`, current `1 - trans`; outdoors the target entry and then the current entry are multiplied SEPARATELY by `1 - alt / 180`: when the modes are equal both names alias one array entry, so the original applies the altitude factor TWICE (the `combined_heavy_rain_wind_sea_two_calls` case exercises it). This is the original's behaviour and is not changed, corrected or shadowed by any oracle.
3. **Wind.** `speed = get_session_wind_velocity_at_point`; `r = 0` if `speed <= minimum`, `1` if `speed >= maximum`, else `(speed - min) / (max - min)`; then `r == 0`: both levels 0; `r < 0.5`:
   light `2r`, heavy 0; else light `1 - 2(r - 0.5)`, heavy `2(r - 0.5)`.
4. **Terrain.** The camera/gunship position bounded to the map, `get_terrain_3d_types_in_sector` (host-owned since 14192), the shared scratch `terrain_types_in_sector`; `ENV_FXS` (observer) records.
5. **Children.** For each `LIST_TYPE_SPECIAL_EFFECT` child of the session in list order, by entity sub type: light/heavy rain and light/heavy wind are SET from the levels above; the six terrain
   children (sea, jungle = FOREST_TOP, amb1 = RIVER, amb2 = RESERVOIR, amb3 = ALTERED_LAND1, amb4 = ALTERED_LAND2) are SMOOTHED: `value = previous amplification +/- 0.5 * frame delta` (up if the
   type is flagged in the sector, else down), bounded to `[0, 0.3]`, times `bound (1 - alt / 280, 0, 1)`, times `0.75` in the cockpit; **`0` at `alt >= 280`**. The scaled value is what is stored, and
   is read back next call, so consecutive calls compound. Any other child is not touched.

Data sources, inspected: the ten children are attached at boot by `attach_session_sound_effects` (`create_campaign` / the parser), looping `ENTITY_TYPE_SOUND_EFFECT`s whose update
(`se_updt.c`) does not touch the amplification (`se_float.c` is the only other writer); the amplification is consumed by `se_snd.c` when the sound system plays them (none in the headless build).
`cloud_3d_base_height` is a plain global the full game sets to 6000 at render start-up (`3d_init.c`); the headless run has no renderer, so the value there is whatever the process holds (printed in
the plan). The only render reader of the scratch beyond this routine is `3drain.c` (ALTERED_LAND3; not exercised headless; the compiled reader/player/render disposition remains required D3 scope, and nothing is inferred from its absence here).

## The characterization (fixture group 11, `csrc/eech_env_fixture.c`, feature `env-fixture`; explicit only)

The REAL unchanged routine is called on the REAL booted state: the session and its ten real ambient children, the real camera entity, the real loaded terrain, and (host builds) the real host
providers. No amplitude is computed by the fixture or the driver. Controlled inputs per case (all printed as IEEE words): camera position and the cloud base (the cached elevation is forced to 0 so
alt = y; `cloud_3d_base_height` is set to 6000, or to `y` / just above / just below for the cloud-edge cases), the session weather (global model: modes and transitional state; or the local ring at a
fraction of its radius) and wind (limits, a zone of radius 8192 with the camera a fraction of the way in, gust phase), the frame delta, the children's initial amplification (terrain children a
controlled value, rain and wind children the sentinel 0.625 to show they are overwritten). The sector under the camera comes from a bounded probe of the real grid with the original sector query
(first sector with each of the six surfaces set, and one with none); a surface not found makes its case `refused` with the reason. Each case prints the original queries the routine consumes
(weather at the point, wind speed and range, radar altitude) as an INPUT echo, the child list in order before, and after each call: every child's amplification, the complete scratch, the shared seed
and whether it moved, and whether the session record changed.

46 cases: rain (global modes and transitional states; indoor/outdoor altitude 0, 90, just below 180, exactly 180, 250; cloud base equal / just above / just below `y`; the four local-ring zones;
the cockpit); wind (calm, zone edge, 0.25, 0.5, 0.75, centre, a gust past the maximum, equal limits, zero minimum); terrain (rising, saturating at 0.3, falling, floor at 0, each of the six surfaces,
altitude just below / at / above 280, half factor, three consecutive calls compounding, zero and one-second delta, the cockpit, the camera east and west of the map clamped); one combined case.

**Restored before every return:** the session record and host field, season, shared seed, frame delta, camera position and cached elevation, cloud base, `in_cockpit`, the terrain scratch and the
amplification of every sound child; `env-sound-end` says `restored=1` for the whole and for each part (0 would be a defect of the fixture, and the driver stops on it). Refusals change nothing.

**Unsupported, declared in the plan (`env-sound-unsupported`):** the speaker output or any render of the children (no sound system headless); the `3drain.c` ALTERED_LAND3 reader; any
non-ambient special-effect child (none exists at boot; lightning strike sounds appear only after a strike); the cockpit branch when the dedicated server holds no gunship entity (those cases are
`refused`, not faked; with a gunship the routine reads its LIVE position, which is not moved). The routine called through a real `update_server`, its place in the update order and the shared-RNG draw
count are the observer's existing evidence, not repeated; a fixture call does not draw (each call row says whether the seed moved). An armed observer would count these direct calls: this driver does
not arm it.

## Driver, tests, commands (Root runs)

* `lua/env-sound-characterization.lua` (boot only, no frame): plan, sites, child order, unsupported items, one row per case and per call, a summary with the refusal reasons and `anomalies` (a moved seed
  or a changed session record are facts about the original, reported and never repaired). It stops (`env-sound: ...`) on a prerequisite that is not there (the session without its ten ambient
  children: `prerequisite_unavailable` row), a setup that is not what the fixture says it made, a child list in another order, an incomplete case, or any state not restored.
* `tests/env_sound_characterization.rs`: harness guards on a scripted engine (what the DRIVER keeps, refuses and reports; no amplitude is computed).
* Source checks: `cargo test -p eech-world --test env_sound_characterization`; `cargo fmt --check`; `cargo clippy --all-targets -- -D warnings` (with `--features eech-dc/env-fixture`).
* Build the fixture module (the existing env-fixture build, `cargo build -p eech-dc --features env-fixture`, or the existing Windows env builds); then, per retail root (fresh, verified):
  `eech-world lua/env-sound-characterization.lua root=<root> out=<private env-sound-<scenario>.jsonl> scenario=lebanon_retail|georgia_retail`.
  Run the same command AFTER any later physical move and compare row for row; any difference is classified per `REVIEW.md` (regression / approved correction / intentional change / input variation).
* Default behaviour is unchanged: group 11 is explicit only; the census (including its `env-unsupported item=update_session_sound_effects` line, the default oracle), the 164-case oracle and groups 0-10
  print exactly as before; the only default-path edit is the group range check (the last group is 11). No accessor, protocol, ABI, provider, default script or manifest changed.

## Remaining prerequisites and handoff

* Root builds and runs; reviews the original results. Possible outcomes to report, none assumed: the ten children exist (expected: `attach_session_sound_effects` runs in `create_campaign`); a
  gunship exists or not (cockpit cases); which of the six surfaces the probe finds on each map; the live `cloud_3d_base_height`.
* After that, the physical extraction (next instruction): the host computes rain/wind/terrain levels from its own weather, wind and terrain and the campaign only writes the amplifications of its
  children (the ordering, smoothing state and which children exist stay campaign state unless the review decides otherwise); no calendar, timer, scheduling or campaign-policy change. Same driver,
  same rows, before and after. D4/D5 are not started; Task 51/D3 remains open.

## Independent source review of c0b6c3386 and the red witness guard (test-only; UNBUILT, UNRUN)

Review `ENV-SOUND-ORIGINAL-C0B` (SHA-256 `b618371b...af63`, CHANGES_REQUIRED, source review only): the fixture's ordinary and error-return cleanup looks sound in the inspected finite path, the routine is
called unchanged on the real session/camera/children with no amplitude formula in the fixture or judge, and the lack of a headless speaker/renderer/player is not dead-code evidence (the compiled player/
render consumers stay required D3 scope). ONE grouped P2 (witness-record integrity): an answered case accepted (1) a missing `fp_before`/`fp_after` (both missing compares `nil == nil` and passes as
restored), (2) duplicate or foreign `env-sound-session` rows (last row wins, the case index is never checked), (3) child amplitude words of any hex width (`%x+`, not the serializer's eight digits).

**This step adds only the red guard**, no driver or fixture fix: `the_driver_refuses_an_answered_case_whose_restoration_session_or_amplitude_records_are_missing_foreign_duplicated_or_malformed`
(`tests/env_sound_characterization.rs`). It feeds the REAL driver the healthy scripted case with ONE actual-grammar record damaged, 23 times, and requires each to be refused as a failure of the witness (an
`env-sound:` error, a `witness_failure` row, no summary): either or both fingerprints missing, fingerprints of 15 / 17 digits or not hex, a short equal pair; the session row duplicated, of another case, without
its fact, or with a fact that is not 0 or 1; `rng_unchanged` missing or 2; a post-call amplitude word of 6 or 9 digits, not hex, or `na` for a sound child; a plan word of 7 digits; and the end row of another case.
Controls kept: the healthy case is accepted, and a moved seed or a changed session record (`0`) is still a genuine fact of the original (accepted, one anomaly). The mutations that the driver already refuses
(a fingerprint that differs, a non-hex word, an end row of another case) are in the list as controls that must keep holding. The test collects every damaged row the driver still ACCEPTS and fails listing them, so the
failure itself is the evidence. No amplitude is computed and none is judged: a record is refused for what it is (absent, malformed, duplicated, of another case), never for its value.

It is prepared, NOT claimed red: Root runs it on the unchanged c0b6c3386 driver (`cargo test -p eech-world --test env_sound_characterization the_driver_refuses_an_answered_case_whose_restoration_session_or_amplitude_records_are_missing_foreign_duplicated_or_malformed`),
and the single grouped driver correction follows. Preserved for the later physical move, and NOT to be corrected or shadowed by any oracle: outdoors the original multiplies `rain_sound_levels[target]` and then
`rain_sound_levels[current]` separately, so for EQUAL modes the two assignments alias one entry and the altitude factor is applied TWICE; the `combined_heavy_rain_wind_sea_two_calls` case exercises it and
the measured words stay authoritative.

## ENV-SOUND-ORIGINAL-1: actual red and the one grouped driver correction (Lua only; UNBUILT, UNRUN)

**Actual red (Root's, on frozen `93bc520d1`, Lua/C/native exactly c0b):** the focused guard above compiled and ran one test and failed (exit 101, 14:16:09 UTC; receipt
`environment-sound-driver-93bc520d1-red-linux-20261031-completion.json`, same-stem log, preserved): the driver ACCEPTED 13 of the 23 damaged records -- both fingerprints missing and the equal short pair, the
duplicate and the foreign session rows, the missing / `2` / `x` session facts, the missing / `2` `rng_unchanged` facts, the short, long and `na` sound words, and the short plan word. The healthy case and the
genuine-anomaly controls held. The original retail ambient measurement has NOT been run (the three fresh roots are unused).

**Correction (`lua/env-sound-characterization.lua` only; the 23 guard cases and the original four tests are unchanged):**

* both state fingerprints must be exactly 16 lowercase hex digits BEFORE they are compared (`nil == nil` can no longer pass); the existing "changed fingerprint" refusal is unchanged;
* each answered case must have exactly ONE `env-sound-session` row, whose `i` is the current case, with a `session_unchanged_by_the_routine` fact of `0` or `1`; the check runs before any row of the case is
  written, so a damaged case publishes no partial rows;
* each call row's `rng_unchanged` is `0` or `1`;
* every amplitude word `children_of` parses (the plan order, the case's `children_before`, every post-call list) is exactly 8 lowercase hex digits; `na` is accepted only for a child the fixture declares NOT a
  sound child (class name `other`), never for one of the ten ambient children.
* a genuine `0` stays what it was: counted as an anomaly of the original, reported, never repaired or refused. Valid native rows keep identical JSON and raw values and their output order.

The state fingerprint is bookkeeping of the fixture's rendered environment state (session fields, season, shared seed, delta, camera): it is NOT evidence of the CPU floating-point state. Equal outdoor rain
modes alias one array entry and are attenuated TWICE in the original; that behaviour is unchanged and must stay measured, not corrected. The renderer reader of the scratch is not exercised headless; its compiled
reader/player/render disposition remains required D3 scope. Pending: Root proves c0b native + the new driver provenance, runs ONLY the new guards and the affected strict checks, then the first retail ambient
measurement and its review; the physical host amplitude solver is the NEXT step after that and is not started. Task 51/D3 (effects, time, remaining terrain, spatial/object, model, full shipping, identity,
review, current CI, normal merges) stays open; D4/D5 are not started.

## The physical move: the ambient amplitude is the host's (production source; UNBUILT, UNRUN, UNREVIEWED)

**Base:** clean pushed `4779d6434`. The original behaviour of `update_session_sound_effects` is MEASURED and independently reviewed (not repeated here): the 21ba driver on the unchanged c0b native, three roots, 46 distinct
cases (44 answered, 2 explicit missing-gunship refusals), 47 actual routine calls, 0 anomalies, every restoration/session/RNG check; the private originals and the review `independent-environment-ambient-original-actual-review-...`
(no findings) are reused as they stand. That baseline is the unchanged ambient body on the hosted field/sector baseline, not the whole-original DLL, and it makes no audio, render, player or CPU-FP claim.

**What moved.** The rain, wind and terrain-smoothed AMPLITUDE arithmetic is no longer computed in EECH_DC (the hosted build, `EECH_CAMPAIGN_HOST_ENV` + `EECH_CAMPAIGN_HOST_TERRAIN`; the original body stays in the
`#else` for the original game's build). The existing host services answer, by value:

* **Environment provider, op `ambient`** (`ss_hostenv.h` AMBIENT; `eech-world` `env.rs`/`env_ref.c env_ref_ambient`): request `{ op, winter, x, z, y, alt, cloud_base, in_cockpit }` -- the point the sound is heard at (the camera, or
  the gunship in the cockpit) and the campaign's observations of it; reply `{ rain_light, rain_heavy, wind_light, wind_heavy }`. The host computes them from ITS weather and wind with the original's expressions in the
  original's order: the weather is asked only inside the rain branch (below the cloud base, and outdoors below 180 m of radar altitude); outdoors the target mode's entry and then the current mode's entry are attenuated
  SEPARATELY by `1 - alt / 180`, so equal modes -- one aliased array entry -- are attenuated TWICE (the original's behaviour, kept); the wind levels are the piecewise function of `r` between the session's limits.
* **Terrain provider, op `ambient`** (`mb_hostterrain.h` AMBIENT; `terrain.rs`/`terrain_ref.c host_terrain_ambient`): request `{ op, winter, x, z, alt, delta, in_cockpit, children = { { class, previous }, ... } }` -- the
  point (clamped to the map, the same the sector question is asked at), the radar altitude, the PREVIOUS frame's delta, and for each of the session's terrain ambient sound children (sea, jungle, amb1-amb4 as classes 0-5), in the
  session's child order, its previous amplification; reply `{ amplitudes = { ... } }`, one per child. The host decides it from ITS sector surfaces (the enumeration of EEDCS-51, so the same sector) with the original's smoothing
  (previous +/- 0.5 * delta, bound to [0, 0.3], times `bound (1 - alt / 280, 0, 1)`, times 0.75 in the cockpit, and 0 at or above 280 m; the previous amplitude is the already SCALED value, so it compounds; it is used only
  where the original read it).
* **Campaign keeps** (`ss_updt.c`): the call and its observer counters, the point and its radar altitude, the clamp to the map, the sector query that still fills the legacy scratch mirror `terrain_types_in_sector` for its
  other readers (the renderer, the observer hash) -- unchanged and still host-owned -- the child list, identities and order, each child's previous amplification (campaign state) and the writes of the new amplification. It
  computes no level and no smoothing and has no Core fallback in the hosted build. No shared-RNG draw is made or moved; the delta is read where it was.

**Observations the campaign supplies (explicit):** the point's height, its radar altitude (from the camera or gunship entity), the cloud base height (`get_cloud_3d_base_height`, a renderer-owned global: the headless run
holds whatever the process holds, as the measured baseline printed), the cockpit flag, the frame delta and each child's previous amplification. They are observations of campaign/render state, not physical answers.

**Boundary correctness.** Finite semantic requests and replies by value, no EECH pointer or layout. Requests are validated before they are sent (at most 16 terrain children, each class one of the six, the cockpit flag 0 or 1; the
count of children is captured before the Lua provider runs). Replies are validated whole before anything is written: four finite levels with the wind levels in [0, 1]; exactly one finite amplitude per child. Named markers:
`[env-provider:ambient.callback|answer]`, `[terrain-provider:ambient.request|callback|answer|amplitudes]`. A missing or malformed host stops the engine with an error naming the provider; there is no fallback. The two kernels
reuse the field/sector FP lessons: no floating-point argument homing (the position as bits, every other float in an array read INSIDE the guard), no validation arithmetic before the guard, the complete x87/MXCSR state
saved before the first floating-point action and restored and read back on every path, the answer staged and written only afterwards.

**Compiled consumers.** The campaign routine is the one compiled consumer of these amplitudes; the sound-effect update that plays them (`se_snd.c`) and the renderer's reader of the scratch (`3drain.c`) are not moved here,
are not exercised headless, and are NOT dead code: their compiled player/render disposition remains required D3 scope. No amplitude path was deferred to D5.

**Tests added (targeted):** `env.rs` (the rain branches and the TWICE attenuation of equal outdoor modes, the wind piecewise levels with exact dyadic numbers, and the complete caller x87/SSE state over the seeded states);
`terrain_sector.rs` (the six classes against four synthetic worlds, the bounds, the altitude and cockpit factors, the unused previous amplitude at or above 280 m, per-child previous amplitudes, the refusals, the Lua provider op, and
the complete caller state over the seeded states, 14 questions); `eech-engine` (the by-value crossing, the request and answer refusals before anything is written, the environment answer); `eech-dc` (both Lua bridges).
The generic witness provider `terrain-provider.lua` answers `ambient` (a land world with none of the six surfaces: silence). The measured original driver, fixture, rows and baselines are NOT changed.

**Commands for Root** (nothing here was built or run; Root builds, then compares the SAME 44-case original driver on fresh roots with this candidate):
1. `cargo fmt --check`; `cargo test -p eech-engine`; `cargo test -p eech-dc`; `cargo test -p eech-world --test terrain_sector --test env_sound_characterization --test env_sector_authority` and the `eech-world` library tests;
   `cargo clippy --all-targets -- -D warnings` (default, `--features eech-dc/env-fixture`, `--features eech-dc/env-observer`).
2. Rebuild the env-fixture native module and the shipping module from this source into NEW directories (the frozen c0b and 14192 natives are not overwritten).
3. `eech-world lua/env-sound-characterization.lua root=<fresh candidate root> out=<private jsonl> scenario=lebanon_retail|georgia_retail` on each fresh candidate root, and compare row for row with the measured originals (every
   amplification word); classify any difference per `REVIEW.md`. The first retail comparison is NOT run.
4. Then the next finite public witness: changed / withheld / malformed amplitude answers from the host reaching the real consumer (not written here).

**Remaining couplings (open):** lightning strike/sound entity creation and execution and the lightning timer's campaign side; the physical time inputs and the campaign calendar/timers/scheduling (Core by decision); the
rest of terrain (trees, loaders, the per-type triangle functions); spatial/object and model services; the compiled player/render/sound-system consumers; full shipping, identity, review, CI and merges. D4/D5 are not started.

## Independent source review of a39a7a2ec and the red controls (test/fixture/driver only; UNBUILT, UNRUN)

Review `ENV-AMBIENT-HOST-A39` (SHA-256 `8cced1aa...a9304c`, CHANGES_REQUIRED, source review only): the delegation, the guards' ingress and the C/Rust ABI check out, and the reviewer's NaN-reader suspicion was withdrawn (not a finding).
Two grouped findings; production is NOT changed in this step, so Root can reproduce both.

**Actuals Root has reported on a39a7a2ec (not mine):** Linux pipeline exit 101 at 14:48:21 before the broader packages; Windows observer native + three libs + `terrain_sector` exports 0, Windows engine 80 / module 24 / world 254 all 0;
the new terrain ambient tests: 2 PASS, 2 FAIL -- the terrain ambient FP matrix (5376 questions, 192 changed answers for the subnormal altitude: `0.14999998` in the normal mode, `0.14999999` under caller FTZ/DAZ) and the basic
saturation expectation; the env ambient tests (3) PASS but their matrix had no FTZ/DAZ or precision variants. The first retail candidate comparison (Lebanon, same 21ba driver, 108 rows) is byte-exact to the originals and is
numeric only (not FP, authority or acceptance evidence).

**P1 control (strengthened, `eech-world/src/env.rs`):** `the_callers_complete_floating_point_state_survives_every_ambient_sound_question_in_every_mode` now seeds every x87 and SSE rounding pair, TWO x87 precision controls, MXCSR sticky
flags clear/set, FTZ and DAZ clear/set (as far as `MXCSR_MASK` has them) and three x87 status words (new helpers `fp::mxcsr_mask`, `fp::seed_modes`; the old `fp::seed` and the old field test are untouched), over ten questions
that now include SUBNORMAL inputs whose arithmetic a denormal mode changes: a subnormal transitional state attenuated outdoors (3 * 2^-149 times the altitude factor) and in the cockpit, and subnormal wind limits (a speed that is the subnormal
maximum). The answers are compared as BIT PATTERNS (never float equality), the full CW/SW/TW/MXCSR words are checked as before, and mismatches are collected. The kernels (`env_ref_ambient`, `host_terrain_ambient`)
and their guards are UNCHANGED, so the new env cases are expected to fail on a39 on an SSE target (a kernel that inherits the caller's FTZ/DAZ answers differently); that is an expectation until Root runs it. The terrain
FP test (5376 questions) is retained unchanged: its red is already valid.

**Saturation classification (from the retained originals and the compiled arithmetic; resolved, no baseline change).** The original sea-saturating case (retained original case 28, altitude 10) has the word `3e941d42`. The original
statement is `value = bound (value, 0.0, 0.3)` with the ternary macro of `miscmath.h:67`, then `value *= bound (1.0 - (alt / 280.0), 0.0, 1.0)`. The two candidate stored bounds give: `0.3f32` (nearest, `0x3e99999a`) times the
altitude factor `0.9642857...` is `3e941d42` (as the nearest or the round-toward-zero product); the run-time-directed `0x3e999999` gives `3e941d41`. Only the first reproduces the retained word, and the candidate's
numeric comparison is exact to it: the constant `0.3` of the ternary is converted to a float by the COMPILER, to the nearest, not at run time under round-toward-zero. The first synthetic expectation in `terrain_sector.rs` (`0x3e999999`)
was the run-time one and was WRONG; it is an unaccepted new test expectation, so it is corrected to `0.3f32` with the evidence in its comment. No retail baseline, original raw file or tolerance changes; no implementation changes.

**P2 control (the wide child list):** the same real path, a tiny new explicit fixture group 12 (`eech_env_fixture.c`, process-isolated like group 9; groups 0-11, the census and the 46-case oracle print exactly as before;
the only default-path edit is the group range check, last group 12) and a short driver `lua/env-sound-wide-children.lua`. Group 12 creates ADDITIONAL terrain ambient sound children with the REAL constructor
`create_local_sound_effect_entity`, exactly as `attach_session_sound_effects` calls it (cycling the six classes, so duplicates of every class exist), until the session holds 16, then 17, then 20 terrain ambient children,
gives each its own DISTINCT previous amplitude (j + 1) / 128 in list order, and calls the REAL unchanged routine with the REAL camera at radar altitude 140 outdoors (height 140 over its cached elevation 0; the echo is read back and must be `430c0000`) with a frame delta of 0. Nothing is mocked and no kernel is called privately.
The driver prints the created count and entity ids, the list (position, entity, subtype, class, word) before and after, and checks only the harness plus one property of the original that this setup makes unambiguous: with
delta 0, every previous amplitude j/128 inside [0, 0.3] and the altitude factor 1 - 140 / 280 EXACTLY 0.5 (the factor of the original's half-altitude cases) a child that was PROCESSED holds exactly HALF its own previous word, j/256, while a
child the routine skipped keeps j/128 and fails the witness: ALL children, in list order, each with its own previous value, none dropped, reordered or re-valued. (An earlier version of this control used delta 0 at altitude 0, an
IDENTITY setup in which a skipped child keeps its previous word exactly as a processed one does; it could catch the 17-child stop but could not substantiate that all children were processed, and is replaced. The harness guard that leaves
one child's after word unchanged and requires the witness to fail is `the_driver_refuses_a_wide_list_that_dropped_reordered_or_revalued_a_child_and_a_control_that_did_not_run`.) No shadow oracle: the half is a setup-derived dyadic fact, not a
recomputation of the routine. Expected healthy: all three cases. Expected on a39: the 16-child case runs; the 17-child case stops in the engine with `[terrain-provider:ambient.request] ... more than 16 ...` (the Core client's cap), reported as
`engine_failure` naming the case and ending the run with `env-sound-wide: CASE 2 (17 terrain ambient children) FAILED in the engine: ...` -- an expectation, not a result, until Root runs it. The control is
process-isolated: creating entities changes the entity heap and the session child list and the report says `changed=1 process_isolated=required heap_restored=0 amplitudes_restored=0`; it restores the session record and host field,
season, shared seed, delta, camera, cloud base, cockpit flag and scratch only. Speaker, network delivery, the player and the renderer are not exercised and not claimed. The boundaries: 0 children at the transport (the existing
engine test), exactly the transport bound (new `an_ambient_terrain_question_of_exactly_the_transport_limit_is_carried_whole_and_in_order`) and one past it (refused by the existing engine test) are the TRANSPORT's;
16, 17 and 20 children of one SESSION are the client's, here. The transport bound stays an implementation detail of one crossing; a host backend may still reject an oversized individual request.

**Commands for Root (nothing here was built or run):**
1. P1 control: `cargo test -p eech-world --lib the_callers_complete_floating_point_state_survives_every_ambient_sound_question_in_every_mode` (and, retained, `--test terrain_sector the_callers_complete_floating_point_state_survives_every_terrain_ambient_question_in_every_mode`);
   with the corrected saturation expectation `cargo test -p eech-world --test terrain_sector the_host_decides_the_terrain_ambient_amplitude_from_its_own_sector_surfaces_with_the_originals_smoothing` should pass on a39.
2. Harness guards: `cargo test -p eech-world --test env_sound_wide_children` and `cargo test -p eech-engine an_ambient_terrain_question`; `cargo fmt --check`; `cargo clippy --all-targets -- -D warnings`.
3. The real wide-list control: build the env-fixture module from this source (the C fixture changed) into a NEW directory, then ONE fresh verified root and ONE fresh process:
   `eech-world lua/env-sound-wide-children.lua root=<fresh root> out=<private env-sound-wide-<scenario>.jsonl> scenario=lebanon_retail|georgia_retail`.
   Prerequisites: a fresh root (the remaining candidate Georgia / repeat roots are unused); the process ends with the control (it cannot be reused for any other measurement).

Then, in the same context, ONE bounded batch fixes both groups: the new ambient arithmetic isolated from the caller's denormal modes and x87 precision inside the guard (restoring every caller bit), and the Core
16-item transport bound replaced by bounded batches that keep every child, the list/application order and each child's previous value. The 46-case original driver, group 11 and all accepted raw originals stay unchanged.

## The two review groups corrected on bf64653de (production source; UNBUILT, UNRUN, UNREVIEWED)

**Actual reds on bf64653de (Root's, not mine; production was a39a7a2ec's, unchanged, verified):**
* **P1, the stronger env FP control** -- Linux exit 101 at 15:16:41 (`environment-ambient-mode-red-bf64653de-linux-20261031-completion.json`): the BIT PATTERNS of the answers differed in 768 of 3840 calls (the subnormal transitional state
  outdoors and in the cockpit, the subnormal wind limits at the zone centre) while the exact CW/SW/TW/MXCSR words, seeds and masks held; Windows, the same filter: 101, 2 PASS / 1 FAIL. The a39 terrain FP red is 192 of 5376 on Linux and Windows
  and 576 of 5376 on the hosted i686 build, so the x87 PRECISION as well as the SSE denormal modes reach the answer. Restoring the caller's state afterwards does not make the answer independent of it.
* **P2, the real wide child-list control** -- `environment-sound-wide-bf64653de-real-red-20261031/controller.json` (native/controller 1, pre/post inputs 0, 51 payloads unchanged; five JSON rows retained, SHA-256
  `eccb519f...a80eb`): case 1 (16 terrain ambient children) is healthy and its NON-identity property holds (every child half its own previous word); case 2 (17) stops in the engine at
  `[terrain-provider:ambient.request] the session has more than 16 terrain ambient sound children`; case 3 (20) was not reached. This is the real constructor -> session list -> routine -> host boundary, not a setup failure.
* **Harness assertion (ancillary):** the bf Linux harness run failed one unaccepted new test, `an_engine_that_stops_in_the_routine_is_the_result_and_names_its_case`, which wrongly required mlua's OUTER error string to hold no
  `stack traceback`; the driver correctly writes only the engine's first line into the JSON `engine_failure.reason` and `error()` then adds the driver's own traceback to the outer error. The test now asserts the EMITTED record (reason's
  first line and marker, case 2, target 17, only case 1 kept, no summary, no witness failure) and tolerates the ordinary outer trace. No real error is suppressed and no production code changed for a test-format assumption.

**Correction 1 -- the new ambient kernels run in a CANONICAL arithmetic environment** (`env_ref.c env_ref_ambient`, `terrain_ref.c host_terrain_ambient`; the closed field and sector kernels keep their guards exactly). `fp_enter` / `sector_fp_enter` change only the
rounding control, so the arithmetic inherited the caller's FTZ/DAZ and x87 precision. The two new entries now, after the caller's COMPLETE state is saved and BEFORE any floating-point work, set and read back whole the environment the
original ran in: x87 control `0x0f7f` (extended precision -- the MinGW runtime's default -- an ASSUMPTION about the engine process, which the numeric comparison with the measured originals tests --, round toward zero, every exception masked) and MXCSR `0x7f80` (round toward zero, every exception masked,
flags clear, flush-to-zero and denormals-are-zero OFF). The restore and the complete read-back of every caller bit (CW, SW, TW, MXCSR) on every exit are unchanged; the position still arrives as bits, the other floats in arrays read inside the guard,
the outputs staged; no validation arithmetic precedes the guard. A caller in the default state (MXCSR `0x1f80`, x87 `0x037f`) sees the identical arithmetic as before (round toward zero was already set), so the default numeric results -- the 44 answered
original cases, the first Lebanon comparison -- are preserved; only a caller in another mode now gets that same answer. The strong env FP control (10 questions, FTZ/DAZ and two precisions, bit patterns, full words) and the 5376-question terrain
control are UNCHANGED and are the proof Root runs.

**Correction 2 -- the session's child list is not bounded by the campaign** (`ss_updt.c`, hosted routine). The fatal "more than 16" abort is gone. The walk now carries the terrain children to the host in as many bounded questions as the list needs
(`CAMPAIGN_TERRAIN_AMBIENT_MAX` is the bound of ONE crossing, a transport property): a question is sent when it holds 16 children, when the walk reaches a rain or wind child (so its write follows the writes of the terrain children before it, as in the
original), and at the end of the list. Every child is asked about with ITS OWN previous amplitude (read when the walk reaches it; no write to another child can change it), none is dropped, clamped or deduplicated, duplicates and interleaved subtypes keep
their places, and each question's whole reply is validated by `campaign_terrain_ambient` before any of its children is written. No question is sent when there is no terrain child. The private by-value ABI, the 0/16/17 transport checks, the shared scratch
mirror, the previous-frame delta and the absence of any shared-RNG draw are unchanged. The real constructor control (fixture group 12, 16/17/20 children, altitude 140, delta 0, each child half its previous word, a skipped child fails) is retained.

**Expectations, not results:** the stronger env FP test, the terrain FP matrix on Linux/Windows/hosted i686, and the wide control (cases 2 and 3 now processed) should pass; the first retail comparison and the 46-case original driver must stay byte-exact.
The accepted originals, group 11, the 46-case driver and every baseline are unchanged.

**Commands for Root:** `cargo fmt --check`; `cargo test -p eech-world --lib the_callers_complete_floating_point_state_survives_every_ambient_sound_question_in_every_mode`;
`cargo test -p eech-world --test terrain_sector` (the 5376 matrix and the saturation expectation); `cargo test -p eech-world --test env_sound_wide_children`; `cargo test -p eech-engine -p eech-dc`; clippy as before; the hosted i686 build of the same tests;
then rebuild the env-fixture and shipping modules into NEW directories, the three fresh numeric 108-row comparisons on the fresh candidate roots, and the real wide-list control on ONE fresh wide root in ONE fresh process
(`eech-world lua/env-sound-wide-children.lua root=<fresh wide root> out=<private jsonl> scenario=lebanon_retail|georgia_retail`). The consumed a39/bf roots are not reused.

## Group 12 fixture cleanup corrected: ENV-AMBIENT-WIDE-1 (fixture/driver/harness only; UNBUILT, UNRUN, UNREVIEWED)

Review of f6d812c7c (SHA-256 `6da71305...c6`, one new P1): the production corrections (the canonical arithmetic environment of the two new ambient kernels, the bounded batches that carry a session's whole child list) are
accepted in source; the new finding is in the FIXTURE. `env_wide_case` saved the whole session record before creating children and `env_restore_state` memcpy-restored it afterwards. The record holds `special_effect_root` (and
`force_root`, `update_link`): the real constructor inserts each new sound child at the head of the session's special-effect list, so restoring the old root detached the new children and left the entity links inconsistent. After a
successful 16-child case the next case saw 6 terrain children, not 16. **Actual (Root's) f6 real fixture diagnostic** (`environment-sound-wide-f6-fixture-diagnostic-20261031`, native/controller 1, pre/post inputs 0, 51 payloads
unchanged): case 1 healthy; case 2's head row says `terrain_before=6 created=11` and the driver stops with a witness failure (counts do not fit the plan). That red is preserved; the original output is not rewritten. It is a fixture defect,
not grounds for changing production, and the production routines, kernels, guards, group 11 (46 cases), `env-sound-characterization.lua` and every baseline are UNCHANGED.

**Correction (fixture `eech_env_fixture.c`, driver `env-sound-wide-children.lua`, harness `env_sound_wide_children.rs`):**
* The cleanup restores the scalar, field and global session state but keeps the LIST roots and links as the creation left them (`force_root`, `special_effect_root`, `update_link` are saved after the body and written back after
  `env_restore_state`). The children, the lists, the heap and the amplitudes are NOT restored, and the report still says so (`heap_restored=0 amplitudes_restored=0`).
* New observations, each by the real accessors: `env_wide_topology` walks the special-effect list link by link (every child's parent is the session, every predecessor is the child walked before it, the list ends); it is reported after creation
  (a bad list refuses the control), in the after row (`topology=`) and in the end row after the cleanup (`list_topology_after_cleanup=`), where `list_preserved=` also says whether the list after the cleanup is the SAME children in the SAME places as
  the one the routine ran on.
* The driver requires all of that, and ties the cases together: the fixture's own live count of terrain children at the start of a case (`terrain_before`) must equal what the previous case left; this case's list WITHOUT the entities it reports as newly
  created must equal the previous case's after list (entity, subtype, class, in order); the created entities must not have been in that list. Amplitudes are deliberately reset by each case and are not compared.
* Kept: the real constructor with the 16/17/20 targets, altitude 140 / delta 0 and the requirement that every terrain child holds exactly half its own previous word (a skipped child fails), and every healthy / missing / duplicate / malformed assertion.
  New guards refuse: `list_preserved=0`, a bad topology after the cleanup or after the routine, a case whose `terrain_before` is not what the last left (the signature of the first fixture), an entity reported as new that the last case had, and a retained child that is
  another entity. The Rust header and the healthy test's name now describe the altitude-140 half-amplitude setup, not the stale altitude-zero prose.

**Commands for Root:** `cargo fmt --check`; `cargo test -p eech-world --test env_sound_wide_children`; clippy; rebuild the env-fixture module (the C fixture changed) into a NEW directory; ONE fresh wide root and ONE fresh process:
`eech-world lua/env-sound-wide-children.lua root=<fresh wide root> out=<private jsonl> scenario=lebanon_retail|georgia_retail` (cases 16, 17 and 20 should all run); then the same sole independent review. The authority witness for the amplitude answers
(changed / withheld / malformed host answers reaching the real consumer) is NOT started. Still coupled and open: the sound-effect update and speaker, the renderer and player consumers, lightning effect execution, the physical time inputs, the rest of terrain,
spatial/object and model services, full shipping 40 / all 8 complete byte identity, review, CI, merges. The canonical extended x87 precision of the two new kernels is an explicit, unmeasured REFERENCE ASSUMPTION, not a measurement of the engine's startup control word.

## The public host-authority witness for the ambient amplitude (Lua/Rust test and docs only; UNBUILT, UNRUN, UNREVIEWED)

**Context (Root's actuals, not mine):** d559c3859 -- the corrected group 12 passes the real 16/17/20 control on a fresh root (native/controller 0, pre/post inputs 0, 51 payloads; each terrain child half its own previous word, list identity continuity, topology), the same sole independent
review closed it with no findings (`independent-environment-G12-correction-review-d559c3859-20261031.json`, SHA-256 `4cbd6867...36c6`), and all production and group 11 are proved exactly f6. Prior numeric equality (three roots, byte-identical) is NOT ownership proof; this witness is.

**What it is.** `lua/env-ambient-authority.lua` + `tests/env_ambient_authority.rs`, new files only; no production, kernel, ABI, group 11, original-characterization, oracle, baseline or tolerance changed, and no native rebuild is needed (the reusable d559 native with its 51 payloads can run the NEW embedded Lua and
judge: an explicitly MIXED provenance Root records). The driver boots through the public path (host-owned Lua -> `require ("eech_dc")` -> the real campaign/session routine -> the real sound children) with BOTH reference providers wrapped, arms the wrappers just before the selected group 11 case and runs it
UNCHANGED (`env_case (11, 27)`, `terrain_sea_rising_from_zero`, one call of the routine, all ten ambient children; or 6). Only `op = "ambient"` is altered; every other operation of both providers (the fixture's weather and wind echoes, the sector question, the map facts) is delegated to the reference unchanged, and the
reference is not asked an altered ambient question. Seven modes, one fresh root and one process each:

| mode | what the host sends | expected |
|---|---|---|
| `control` | both providers delegated; every ambient question and the reference's reply recorded | the ten children hold exactly the words the reference answered |
| `changed_env_ambient` | `{ rain_light = 0.375, rain_heavy = 0.1875, wind_light = 0.625, wind_heavy = 0.3125 }` (valid, finite, distinct dyadics) | light/heavy rain and wind children hold those words; terrain delegated |
| `changed_terrain_ambient` | amplitude `(4 i + 1) / 256` for the i-th child asked about | each real terrain child, in the order asked, holds ITS amplitude (the mapping by class and order, not a count) |
| `withheld_env_ambient` | the provider raises | engine stops at `[env-provider:ambient.callback]` |
| `malformed_env_ambient` | wind level 1.5 | engine stops at `[env-provider:ambient.answer]` |
| `withheld_terrain_ambient` | the provider raises | engine stops at `[terrain-provider:ambient.callback]` |
| `malformed_terrain_ambient` | one amplitude fewer than children asked | engine stops at `[terrain-provider:ambient.amplitudes]` |

The markers are the existing bridge markers (the Lua bridges' `ambient.callback`, `ambient.answer`, `ambient.amplitudes`), read from the bridges' code, not guessed. A refusal run is honest about what it is: the ENGINE stopped, the fixture never returned, no case row exists, the request counts say how far it got (the
environment question comes first; for a terrain refusal the environment question was answered by the reference and the terrain question was altered); it is not turned into a native success and there is no fallback to a campaign solver.

**The judge** (`tests/env_ambient_authority.rs`, strict about its evidence): a completed run needs the case row (label, calls), exactly one environment and one terrain ambient question per call of the routine, the ten children in order, the terrain question's classes equal to the terrain children the session lists (the mapping), and
every child's word equal to the answer the provider gave -- the reference's in the control (so the children are shown to hold the HOST's answers even unaltered), the sent value in a changed mode, which must also differ from the control's reference answer (non-vacuous) and be distinct per child. Counts, children, rows and counters must
reconcile, so missing, truncated or dropped evidence cannot pass; a witness failure row, a stopped run with a case row, an alteration in the control and the wrong marker are refused. Guards on the rows the REAL driver wrote against a SCRIPTED engine (it asks the installed providers as the campaign does and validates their answers
as the real bridges do; it invents no physics) prove only the bookkeeping, and they are NOT the actual run. The public test prints SKIPPED without its environment. The child-process wall control is the sector witness's repaired one (the test executable re-run as ONE owned child, kill and wait shown; no shell, no `ping`).

**Commands for Root** (nothing here was built or run): `cargo fmt --check`; `cargo test -p eech-world --test env_ambient_authority`; clippy; then, with the reusable d559 native beside `eech-world`, and the seven fresh Lebanon roots prepared for this witness (not run or consumed by me):
`EECH_WORLD_BIN=<eech-world> EECH_AMBIENT_ROOTS=<seven fresh verified roots, ';' on Windows> [EECH_AMBIENT_MODES=control,changed_env_ambient,...] [EECH_AMBIENT_SCENARIO=lebanon_retail] [EECH_AMBIENT_DIR=<private dir>] cargo test -p eech-world --test env_ambient_authority -- --nocapture`
(one root per mode, the control first). Private coordinates and raw outputs stay outside Git. Root records the mixed provenance (new embedded Lua and judge, unchanged frozen d559 native).

**Scope and limits.** This is a focused fixture control of the ambient amplitude of ONE group 11 case. It is not a real frame or `update_server` run, and it observes nothing of the sound-effect update, any speaker, network delivery, the player, the renderer, Tacview or the shipping module; no audible, network or general
DCS capability is inferred. The 16-child transport bound, the wide-list control and the numeric comparisons are separate evidence. Still coupled and open in D3: the physical sound execution and the speaker, lightning effect execution, the physical time inputs, the rest of terrain, spatial/object and model services, the compiled
player/render consumers, full shipping 40 / all 8 complete byte identity, review, current CI and merges. D4/D5 are unstarted.
