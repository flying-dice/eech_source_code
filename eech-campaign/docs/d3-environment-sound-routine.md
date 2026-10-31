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
