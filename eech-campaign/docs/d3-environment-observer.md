# D3 environment services (EEDCS-51): the read-only observer of the ORIGINAL initialisation, shared-entropy order, lightning/sound effects and frame time phase

Status: **SOURCE PREPARED. UNBUILT. UNRUN. UNREVIEWED.** The author has no C compiler, Lua or cargo; Root builds, runs and reviews. This is a characterization step of the ORIGINAL code, not a production extraction:
no physical state, solver or service moves, and nothing here claims host authority, a cutover, Task 51 or D3 completion. The closed measurements of the first increment
(`d3-environment-characterization.md`, source `99352e33`) are unchanged evidence of what they measured; this step adds the evidence they explicitly excluded.

Base: branch `codex/d3-environment-services`, `b8e8b615` merged normally with the accepted integration `b6575594` (`9e7f7fd6`; one additive `STATUS.md` conflict, the NUL of `eech_engine.c` intact). The integration requires a
terrain provider at boot, so the driver now passes `terrain = host.terrain_provider (root, scenario.map)` (the reference provider, opened lazily); nothing else of the accepted terrain work is touched.

## What the remaining gap was, and what is now observable

The first increment read the session before and after frames and ran controlled queries and updates. It did not record: (1) the order of the shared-seed reseeds and the ten random-weather draws at session creation, (2) what the lightning
expiry draws and creates, (3) the frame time phase. The observer records those in the order the ORIGINAL code performs them, from integers and IEEE bit patterns it only reads.

## Source facts (read, not run; the observer is what turns them into measurements)

* **Seed and clock path.** `eech_engine.c` `eech_engine_boot` sets the simulated clock to `((seed - 1) % 1000000) * 1000` ms and calls `boot`, which runs `full_initialise_game` (through the init screen) and so
  `modules/maths/initmath.c:75 initialise_random_number_generator`, then `set_random_number_seed (config->random_seed)`. `modules/maths/random.c` `initialise_random_number_generator` seeds from `get_system_time ()`
  (`modules/system/time.c:202`, `timeGetTime`, the headless clock) and `set_random_number_seed` also calls libc `srand`. The campaign file loader (`ai/parser/parsgen.c`, the `create_local_entity (ENTITY_TYPE_SESSION)` block) creates the
  session: `ss_creat.c create_local` calls `get_session_random_start_time_of_day`, which calls `initialise_random_number_generator` AGAIN (`session.c get_session_random_start_time_of_day`), draws `rand16 () % 6` and `sfrand1 ()`;
  the loader then calls `set_session_random_weather_mode` (ten draws: radius, flag, x, z, heading for rain; the same five for wind); `FILE_TAG_WEATHER_RAIN` 0 calls `set_session_fixed_weather_mode (DRY)` (draws none).
  `ai/faction/parser.c`, with the same calls, is not in `sources.txt`. *Consequence to confirm by the record, not assumed:* the seed given to boot is replaced by the clock-derived seed at session creation.
* **Frame phase.** `eech_engine.c frame ()` runs the entity updates (`update_client_server_entities`, `update_update_functions`, ...) with `get_delta_time ()` = `system_delta_time`, THEN `eech_headless_advance_time_ms (ms)` and `set_delta_time ()`
  (`modules/system/time.c:142`, from `timeGetTime` differences; `reset_delta_time` starts it at 0.1 s). By source order a frame's updates use the previous frame's delta. A fixed 100 ms schedule cannot show that; a varying one can.
* **Per-frame session update.** `ss_updt.c update_server`: `update_time_of_day` (`elapsed_time += delta * acceleration`), `update_time_of_day_resync` (comms transmit each minute), `update_transitional_weather` (local model: weather/wind evolution and,
  through `update_lightning_effect`, the lightning timer), `update_session_sound_effects` (sound amplitudes; fills the shared `terrain_types_in_sector` through `get_terrain_3d_types_in_sector`). `OGRE_EE` is not defined, so all are live.
* **Lightning.** `update_lightning_effect` (server): `lightning_timer -= delta`; at zero it draws `frand1` for the next timer, sets it with `set_client_server_entity_float_value`, calls `add_3d_lightning_strike` twice and creates two
  client/server sound entities (each pitch argument draws `frand1`). The initial `lightning_timer` is `ONE_MINUTE` (`ss_creat.c`). The retained 120 s control shows resets at about frames 600 and 1076, which alone proves neither the draws nor an entity creation.
* **Other shared-entropy consumers.** libc `rand ()` is read by `camera.c:449–453`, `explosn.c:695` (explosion sample), `3dobjid.c:950` and others; it is reseeded with every `set_random_number_seed`. Its draws are NOT observed (no hook), and its state cannot be read or restored.
* **State consumers.** `ss_pack.c` (lines about 104–132 and 262–297) packs elapsed/lightning/start time, the weather and wind fields and flags for clients and saves; a dedicated server does not run that path in the retail scenarios. NOT audited by a run, NOT witnessed: a gap.

## The observer (TEST-ONLY, feature `env-observer`, which implies `env-fixture`; off in every shipping build)

* **Hooks** (`aphavoc/source/entity/special/session/ss_envfx.h`, macros that expand to nothing without the feature, like `mb_cmbfx.h`): `session.c` (start-time reseed/index/time/result, the ten weather draws with the value each produced, the fixed mode), `ss_creat.c`
  (session defaults), `parsgen.c` (before/after the session creation, the rain and wind tags), `ss_updt.c` (time-of-day begin/end, lightning expiry/new timer/timer set/active environment/each strike/each sound-entity creation with the entity heap's next free indices
  before and after, the sound-effect facts on change of the terrain scratch or the camera, counters of every per-frame routine), `modules/maths/random.c` (every `set_random_number_seed` with its argument and the word before, and the clock value
  `initialise_random_number_generator` seeded from; a plain `#ifdef`), `eech_engine.c` (boot begin/end, frame begin/end with the host's milliseconds, the clock and `system_delta_time`, under `#ifdef`; the NUL byte kept).
* **Read-only.** Each event is the shared seed word at that point plus integers/IEEE words already computed. No hook draws, reseeds, allocates, calls the engine or writes a variable. A change of the seed word between two events with no draw hook between them is a draw
  by code that has no hook, and is shown as such. Storage is 16,384 fixed events; overflow drops and counts, and the driver refuses a record that dropped, was not armed before boot, or has a gap.
* **Glue.** `dc.env_observer_arm (true)` (module function, valid only before boot), `engine:env_observer_events ()` and `engine:env_observer_report ()`; Rust `eech_engine::env_observer_arm`, `Engine::env_observer_events/report`; C
  `eech_engine_env_observer_arm/report/events`. No new crate or public ABI.
* **Not provided (said, not hidden):** no save/restore of anything. Lightning/sound/entity effects, the libc stream and created entities cannot be rolled back, so the only isolation is a fresh process.

## The driver (`lua/env-characterization.lua`)

New arguments, all validated before anything runs: `observer=on|off` (needs `probes=0`; arms before boot; refused with a `refusal` row for a module without the feature), `frame_schedule=100,250,50` (1 to 16 cycled frame lengths of 1 to 1000 ms; not combined with `frame_ms`),
`controlled=lightning_due|local_weather_on` (needs `probes=0`; fixture group 9: ONE labelled case of the original setters — the 0.5 s lightning timer, and for `local_weather_on` the local model with a 40 km radius — right after the initial snapshots; it restores nothing and prints `process_isolated=required`;
a fresh process, never a default run). The record is drained after boot and every 200 frames and checked: row shape, an unbroken sequence starting with `boot_begin`, the signed seed word equal to its hex word, frames in pairs, `frame_begin` = frames run, `tod_begin` = the
original's own count of session updates. `observer_summary` counts the kinds. A run without the new arguments writes the rows it always wrote.

## Finite cases and Root's commands (none run)

1. Build: `tools/build-windows-env-observer.sh target/env-observer/<dir>` (a copy of the env-fixture wrapper). Also `cargo test -p eech-world --test host_callbacks the_environment_`, Clippy with and without `--features eech-dc/env-observer`, and the strict feature-off preprocess equality of `session.c`, `ss_creat.c`, `ss_updt.c`, `parsgen.c`, `random.c`, `eech_engine.c`.
2. OFF/ON pair, same root, seed and schedule (Lebanon, then Georgia): `eech-world lua/env-characterization.lua root=<root> out=<f> scenario=lebanon_retail seed=1 seconds=120 frame_schedule=100,250,50 probes=0 snapshot_frames=20 snapshot_every=60 observer=off|on`.
   Expected: every non-observer row (snapshots, `campaign_end`, the clock, the final state) identical between the pair apart from the boot row's `observer`; if a recording is made, compare it too.
3. Natural frames first: read `observer:boot` for the reseed/draw order and `observer:frame:*`/`observer:final` for the time phase (`frame_begin` `a` against the next `tod_begin` delta word) and for whether the lightning expiry created strikes and sound entities (and which heap indices).
4. Only if natural frames do not reach a branch: a fresh process per `controlled=` case, `observer=on`, a short `seconds=` (the 0.5 s timer expires in the first frames).

## Remaining limitations and coupling

* Unrun and unreviewed; compile details (`TERRAIN_TYPE_LAST`, `terrain_types_in_sector`, entity-heap macros) follow the headers the originals use and need Root's real-header compile.
* The observer sees the server branch of lightning only; the client branch, session pack/unpack and the libc stream are unobserved. The sound-effect amplitudes are not recorded (only their inputs and the terrain scratch).
* The retail weather model and whether lightning fires in 120 s are unknown until a run; the camera's headless position still decides the local-model `weather_mode` fields.
* The physical environment state, evolution, the lightning/sound/terrain-scratch effects, the time-of-day input and the loaders still have to move to the host within Task 51 / D3, with campaign timers, scheduling and consequences staying in Core. D4/D5 are not touched.
