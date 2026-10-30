# D3 environment services (EEDCS-51): characterisation of the ORIGINAL weather, wind and clock — first increment

Status: **SOURCE PREPARED. UNBUILT. UNRUN. UNREVIEWED.** Nothing in this increment has been compiled, executed or independently reviewed (no C compiler, Lua or cargo was available to the author; the root builds
and runs). It is a behaviour oracle for the ORIGINAL code only. **No physical cutover is made or implied**; D3 (EEDCS-21) stays active and incomplete, D4 and D5 have not started, and this document claims no host authority,
no regression result, no Tacview equality and no completion of EEDCS-51. Task 47's original shipping pack is untouched. The preparations of Tasks 48 to 50 are not used and are not integrated acceptance.

Scope (live goal, as read by the root): the required physical terrain and spatial queries, the weather and environment, and the host time inputs leave Core within D3; campaign geography, route choices, timers,
scheduling, policy and consequences stay Core. This increment covers **only the weather, wind and clock part, and only as a behaviour oracle**. Terrain and spatial queries are a separate part of D3 and are not touched.

## What was added (every path is `eech-campaign/` unless stated)

| path | change |
|---|---|
| `crates/eech-engine-sys/csrc/eech_env_fixture.c` | new, whole file inside `#ifdef EECH_ENV_FIXTURE`: the census, the retail snapshot and the controlled witnesses |
| `crates/eech-engine-sys/csrc/eech_engine.c` / `.h` | appended `eech_engine_env_case` under `#ifdef EECH_ENV_FIXTURE` (the file's one embedded NUL byte is preserved at offset 38199; appended with `cat >>`, never rewritten) |
| `../aphavoc/source/entity/special/session/session.c`, `ss_updt.c` (repo root `aphavoc/`) | one appended `#ifdef EECH_ENV_FIXTURE` wrapper each, exposing the `static` `get_wind_gusting_modulation_value` and `update_time_of_day` unchanged. Nothing else in the originals is touched; the feature-off preprocessed text is expected to equal the baseline except the trailing blank lines — **to be proven by the root's strict-preprocess check (not run)** |
| `crates/eech-engine-sys/{Cargo.toml,build/main.rs,src/lib.rs}`, `crates/eech-engine/{Cargo.toml,src/lib.rs}`, `crates/eech-dc/{Cargo.toml,src/module.rs}` | feature `env-fixture`; `Engine::env_case(group, index)`; Lua method `engine:env_case(group, index)` |
| `crates/eech-world/lua/env-characterization.lua` | the driver |
| `crates/eech-world/tests/host_callbacks.rs` | four stand-in checks of the driver's contract (below) |
| `tools/build-windows-env-fixture.sh` | a `sed` copy of `build-windows-weapon-fixture.sh` with the feature `eech-dc/env-fixture` (same single-line-change guard) |

Shipping dispatch and every numerical expression of the originals are unchanged; the feature is off in every shipping build. The fixture asks no host anything and calls only original routines; it never recomputes an answer.

## Source findings (read, not run — each is a fact of the source, not a runtime result)

Routines (`aphavoc/source/entity/special/session/`): `session.c` `calculate_session_time_of_day` 136, `get_day_segment_type` 177, `get_segment_time_of_day` 210, `get_session_random_start_time_of_day` 263,
`get_session_preset_time_of_day` 313, `get_session_weather_at_point` 339, `get_simple_session_weather_at_point` 490, `set_session_fixed_weather_mode` 515, `set_session_random_weather_mode` 538,
`get_session_wind_velocity_at_point` 595, `get_wind_gusting_modulation_value` 888 (static; seven segments over [0,1), six sample amounts; it **asserts** a gust outside [0,1)). `ss_updt.c` `update_time_of_day` 79 (static),
`update_time_of_day_resync` 126 (static), `update_lightning_effect` 159, `update_transitional_weather` 303, `update_session_sound_effects` 536, `update_server` 921 (the server dispatch: time of day, resync, weather and wind, sound).

1. **What executes in the shipped headless build.** `sources.txt` compiles `session.c` and all of `ss_*.c`; `build/patches.rs` patches none of them; `OGRE_EE` is not defined anywhere in the build, so `update_lightning_effect` and the weather
   part of `update_session_sound_effects` are live. `eech_engine.c` `frame()` (~line 303) runs `update_client_server_entities()`, which reaches `update_server`, so **the session's clock, weather, wind, gust phase, lightning timer, resync counter and sound
   amplitudes evolve in every headless frame**. The session's update is camera-dependent: `ss_creat.c` asserts a camera entity (`cm_creat.c:89` creates it) and `update_transitional_weather` reads the **camera's** position (or the gunship's when `in_cockpit`) to
   set `weather_mode`, `target_weather_mode` and `weather_mode_transitional_status` in the local model. So those three fields in a dedicated-server run are a function of the headless camera entity's position, a view artifact; the weather centre, wind centre,
   radii, gust phase and wind direction do not depend on it. Whether a retail run's weather is local, fixed dry, or global, and its season, are **unknown until the census/snapshot runs**.
2. **How the session is initialised.** `ss_creat.c` 139/143: `start_time = get_session_random_start_time_of_day()` (which calls `initialise_random_number_generator`, reseeding the shared generator **and the libc generator** from `get_system_time`, i.e. the
   headless simulated clock whose epoch `eech_engine.c` ~296 derives from the boot seed) and `local_weather_model = TRUE`. The campaign loader `ai/parser/parsgen.c` 1205 then creates the session and calls `set_session_random_weather_mode` (ten draws of the shared
   generator: radius, direction flag, position and heading for rain and again for wind); `FILE_TAG_WEATHER_RAIN` with value 0 (1295–1310) calls `set_session_fixed_weather_mode(DRY)`; `FILE_TAG_WEATHER_WIND` (1318) is parsed and only logged. `ai/faction/parser.c`, which has the same calls, is **not** in `sources.txt`.
   The session record is packed/unpacked by `ss_pack.c` for clients and saves; a dedicated server does not exercise that path.
3. **Time enters from the host in one place.** `frame()` advances the headless clock (`eech_headless_advance_time_ms`) and then `set_delta_time()` (`modules/system/time.c` 142: `system_delta_time` from `timeGetTime` differences); the updates of the NEXT frame read it with
   `get_delta_time()`. By source order the updates of a frame therefore use the previous frame's delta (the very first uses 0.1 s from `reset_delta_time`, time.c 117). `update_time_of_day` adds `delta * time_of_day_acceleration` to `elapsed_time`. **To be confirmed by the
   per-frame retail snapshots**, not asserted here.
4. **Consumers** (compiled consumers of the queries; execution in a headless run is established only by a run, not by this list):
   * campaign-visible, physical-world input: `weapon/wn_move.c` 399–410 serves the host's weapon-flight lookup 7 (`CAMPAIGN_STEP_LOOKUP_WIND`, `mb_hostaim.h:223`; `eech-world/src/aim.rs:1245`) **from the campaign's own session wind** — the D2 weapon step asks the campaign
     for wind, so D3 reverses that authority; `mb_msgs.c` 1701 switches vehicle headlights on `INT_TYPE_DAY_SEGMENT_TYPE == NIGHT`;
   * campaign timing (stays Core on host-supplied time): `FLOAT_TYPE_ELAPSED_TIME` in `ai/highlevl/highlevl.c` 361/3160, `ai/taskgen/assign.c` 633, `ai/parser/parsgen.c` 2653, `special/task/ts_float.c` 227, `task.c` 1374/1415 (kill/loss log time and day), the fixed-wing toggle in `fw_move.c`; Tacview's start time and clock use
     `FLOAT_TYPE_TIME_OF_DAY` and `INT_TYPE_DAY` (`entity/tacview/tacview.c` 155/166/235);
   * visual/audio/UI only: `smokelst/sl_move.c` 114 (smoke drift), `soundeff/speech.c` 1762, `app3d/3d_draw.c` 101/201–207/494–500/594–600, the sound amplitudes of `update_session_sound_effects`, `ui_menu/ingame/campaign/ca_brief.c` 651/691, the player gunship avionics,
     dynamics and wipers (`gunships/**`, player flight, not the dedicated server's).
   The split above is **proposed for review, not decided**: it follows the live goal's statement and the consumers' use, and the visual/audio rows are not "not required" — they still execute (item 1) and must be served or consciously ignored by a later host contract.

## The fixture

`engine:env_case(group, index)` returns `key=value` text lines; one call is one case (32 KiB buffer; a report that does not fit is refused, never cut). Groups: **0 census** (case counts, enum values, constants, the configured `EECH.INI` radii and speeds, the map extents, and the
`env-unsupported` items) and **1 retail snapshot** (read-only; `tag=retail`) are the only calls that look at the unmodified session; **2–8 are controlled witnesses** (`kind=witness`): the case sets only the session fields it names, calls the original routine, prints the words,
restores, and ends with `env-witness-end restored=1|0 fp_before fp_after`. A witness that did not restore is a fixture defect and fails the run. What is restored: the whole session record, the map season, the shared seed word `random_number_seed` (the libc generator is never touched)
and `system_delta_time`. A path through the engine's fatal handler cannot restore anything; every case is domain-guarded so no original `ASSERT` fires (a gust outside [0,1) is **refused and printed, never called**).
Floats are IEEE-754 words (`_w`, `%08x`), so a signed zero or a neighbouring float is visible; fingerprints are FNV-1a 64 of the rendered state text.

| group | witness | original routines called | cases |
|---|---|---|---|
| 2 weather | local model, radius 30 000 m, summer and winter: centre, each ring edge (0.333, 0.666, 1.0 of the radius) with the float below / at / above, one point per zone, far outside; desert (neither summer nor winter); radius 0 (dry, state 1.0); a diagonal point (the approximate range); global/fixed model: three mode pairs × transitional state {0, .25, below .5, .5, .75, 1.0}, and equal modes at .5 — each with the simple choice | `get_session_weather_at_point`, `get_simple_session_weather_at_point`, `get_approx_2d_range` | 55 |
| 3 wind | inside / at / outside the zone radius (float below and above the edge), at gust 0 and 0.3; zero radius; zero minimum with a reversed direction (signed zero); equal limits; reversed and zero directions; a diagonal centre; vector and NULL output compared | `get_session_wind_velocity_at_point`, the gust modulation | 21 |
| 4 gust | every sample boundary k/7 with the float below and above, mid-segment points, +0, −0, the smallest denormal, 0.25, 0.5, the float below 1; and 2 REFUSED (1.0, −0.5) | the gust modulation; the calm-field speed | 33 (31 called, 2 refused) |
| 5 clock | day roll-over and the clamp of the time of day (start/elapsed pairs incl. −0 and negative elapsed), the accessors `FLOAT_TYPE_TIME_OF_DAY`/`INT_TYPE_DAY`, the day-segment edges (6, 8, 18, 20 h, below and at), and the constants (segment times, five presets) | `calculate_session_time_of_day`, `get_day_segment_type`, `get_segment_time_of_day`, `get_session_preset_time_of_day` | 22 |
| 6 init | the four fixed modes, and the random mode from three controlled seed words (1, 1 234 567, 20 261 030) with the resulting fields, the generator word after, and queries at two points | `set_session_fixed_weather_mode`, `set_session_random_weather_mode` | 7 |
| 7 clock update | before/after `update_time_of_day`: real time, ×60, each segment edge, the day wrap, skip-night after dusk and before dawn, zero delta | `update_time_of_day` (via the wrapper) | 10 |
| 8 weather/wind update | before/after `update_transitional_weather` with lightning kept from firing (`lightning_timer = 1e6`): radius clamp at max/min and the direction flip, weather bounce at each map edge, radius 0, global transition progress / completion / steady, gust phase wrap and the exact-1.0 float case, wind radius clamps, wind-centre bounce and the direction refresh. REFUSED, not run, if `in_cockpit`, no camera entity, or not a server | `update_transitional_weather` | 16 |

Total 164 cases. The `coverage` rows of a run report available / ran / refused / truncated per group; the census prints the counts from the C side, which are authoritative over this table.

**Not witnessed, and said so in the census:** `get_session_random_start_time_of_day` (it reseeds the libc generator; unrestorable here); `update_lightning_effect` (creates strike and sound entities and draws the shared generator — seen only through the real frames' `lightning_timer` and `rng`);
`update_session_sound_effects` (every headless frame; sound amplitudes and the shared `terrain_types_in_sector`); `update_time_of_day_resync` (comms transmit each minute); the session pack/unpack. The exact separate real-source fixture that would close the lightning/sound gap is a read-only
observer hook (the pattern of `mb_cmbfx.h`) around `update_lightning_effect` and `update_session_sound_effects` recording the order of the generator draws and entity creations inside a natural frame; nothing in this increment starts it.

## Driver, commands and protocol (for the root; none of this has been run)

Build (Git Bash, Docker, Linux containers; from a clean checkout of the commit to characterise): `tools/build-windows-env-fixture.sh target/env-fixture/<dir>`. For a feature-enabled check of the Rust: `cargo test -p eech-world`, and clippy/fmt as for the other fixtures, with and without `--features eech-dc/env-fixture`.

```
eech-world lua/env-characterization.lua root=<verified retail root> out=<env-lebanon-probe.jsonl> scenario=lebanon_retail seed=1 seconds=120 frame_ms=100 probes=512 probe_at=0 snapshot_frames=20 snapshot_every=60
eech-world lua/env-characterization.lua root=<same root>                out=<env-lebanon-control.jsonl> scenario=lebanon_retail seed=1 seconds=120 frame_ms=100 probes=0   probe_at=0 snapshot_frames=20 snapshot_every=60
```
and the same pair with `scenario=georgia_retail`. A DLL built **without** the feature is refused: a `refusal` row, the message `the engine has no env_case method`, a non-zero exit, no frame run. Every argument is validated before the output file is created or anything is prepared or booted.

**Control protocol (probe noninterference).** The pair above differs only in `probes`. Expected, to be checked by the root: every `snapshot` row of the two runs (initial, frames 1–20, periodic, final), the `campaign_end` row and the clock are byte-identical apart from the `boot` row's `probes`/`mode`;
inside the probe run, `probe_noninterference` is true (retail and campaign fingerprints equal before and after the whole witness set) and every witness ends `restored=1`. A single difference is a finding about the fixture, not a result about the original. Repeat each run once to
show whether two runs of the same command are identical (the seed, the simulated clock and the schedule are fixed; the engine's other nondeterminism is not assumed away).

**Output schema (JSON Lines, sorted keys, one object per line, written with every write/flush/close checked).** `boot` (arguments, `mode` control|probe); one object per report line with `event` = the line's first word, `scope` (`census`, `snapshot:initial`, `snapshot:frame:N`,
`snapshot:periodic:T`, `snapshot:before_probes`, `snapshot:after_probes`, `snapshot:final`, `probe:g<group>:<index>`), `at` (simulated seconds) and the line's fields; keys ending `_w` and `fp*` are **strings** (all-digit words stay strings), other plain decimal values are numbers;
`campaign` rows (object count, alive, a polynomial fingerprint of `engine:objects()` at the start, around the probes and at the end); `probe_noninterference`; `coverage` (per group); `run` (frames, probes requested/run, cases available, `complete`, unsupported items). `refusal`/`failure` precede a non-zero exit.

## Host-side contract checks (stand-in engine; they say nothing about the original's weather, wind or clock)

`host_callbacks.rs`: 20 malformed arguments refused before anything is prepared, booted, run or written; a module without the fixture refused with a `refusal` row and no frame; a control run that asks for no witness, a capped probe run whose all-digit and signed-zero words stay strings and whose
coverage and noninterference rows are reported; and a witness that did not restore failing the run. `every_shipped_lua_script_parses_under_the_hosts_lua` also parses the driver. **Not run.**

## Expected native limitations and unknowns (not results)

* No part of this compiles or runs yet: header names and macros (`knots_to_metres_per_second`, `MIN_MAP_X`, `in_cockpit`, `system_delta_time`, `nextafterf`) are used as `session.c`/`ss_updt.c` use them and need the root's real-header compile (the other fixtures' `syntax-*.log` method).
* The camera entity's headless position is unknown; the local-model `weather_mode` of an unmodified frame, and of the group 8 cases, depend on it (printed as `camera_*_w`). The retail weather model (local, fixed dry), season and map extents are read, not assumed.
* The ring-edge inputs are derived in the engine's round-toward-zero mode and printed as words; a different rounding mode on a later path gives different input words, which the comparison would show rather than hide.
* The first frame's delta (source: 0.1 s) and the per-frame increments are unconfirmed until the snapshots are read.
* Coverage of one scenario/seed is not coverage of other seasons or of a fixed-weather campaign file; the desert/summer/winter cases are synthetic season values, not retail maps.

## Missing gates for full D3 (explicit; none started here)

1. Root's build, native runs (control + probe, both scenarios, repeated), strict feature-off preprocess equality, the stand-in tests, clippy/fmt, and independent review of this source; then acceptance or rejection of the recorded ORIGINAL behaviour by the technical lead.
2. The host contract for weather, wind and the time-of-day input (what a host supplies, what Core derives, how DCS maps), designed from the accepted behaviour — not designed here.
3. The real-source fixture for lightning/sound/terrain-type draws and entity creation, and for the session pack/unpack, if the accepted contract needs them.
4. The rewiring and per-consumer evidence: the weapon-step wind lookup, smoke, speech, UI/briefing, headlights, Tacview time reference, campaign timers on host-supplied time; shared-generator position and update order preserved.
5. The terrain/spatial-query part of D3, separately.
6. The M4 regression pack and the existing Tacview comparisons against any candidate that moves authority, with the baseline governance of `docs/m4-baseline-governance.md`.

Next validation action: the root builds the fixture DLL, runs the control/probe pair on Lebanon and checks the census, the `restored` and noninterference invariants and the unmodified snapshots before any case table is trusted.
