# D3 (EEDCS-51): time ownership and the DCS API / data mapping of the environment services

Status: **SOURCE DISPOSITION AND PLAN. Documentation only: no C, Rust, Lua, build, test, baseline, oracle, flag, target or configuration changed. Not a DCS runtime result, not an acceptance of Task 51 or D3.**
Source read: `78fe5efdb5cbaff904e3b645d680acbd7776597d` (this document edits nothing else; revised after the independent documentation review 3f6aa907 of `4d84bb335`, which found two P2 items and left the weather
mapping row open: sections 1, 3 and 4 are corrected and made concrete here, and the row is **not** called closed until that review accepts them). Throughout, the **reference provider** is the existing EECH World
provider (`eech-world`, unchanged, default behaviour and baselines unchanged) and the **planned DCS provider** is the design below (nothing implemented). Evidence labels: **S** = read from that source by the author of this document; **R** = reported by Root, not reproduced
here; **P** = a pinned primary declaration (types and names, never native DCS behaviour); **U** = unknown, stated as such. Root owns review, gates and delivery.

Pinned primary reference (M5 names `flying-dice/tslua-dcs` as a REFERENCE, not an implementation; no TypeScript port is resumed): commit `1179554fcafcef92d71313336cc09500febf8245`, files under
`packages/tslua-dcs-mission-types/src/`, retained with their blob and SHA-256 in `D:/EECH-Migration-Evidence/D3-pinned-primary-reference-20261101/` (`receipt.json`, `env-receipt.json`):

| file | git blob | SHA-256 |
| --- | --- | --- |
| `timer.ts` | `abc1b98a108c48974cf2b84f3c55792fd30c5521` | `c66c6ca8dbd99c99160eb9d81f8bfe55096e6d593234c1e200493a47780388d9` |
| `atmosphere.ts` | `421e6f651978137ea35ed6f6b36bb6954edf3114` | `80a63a4718e163ae6f30a5baca4ab20bbe7e924cb26cf61cd8de52a46d28032b` |
| `land.ts` | `9e5fd4557fd94b7da38f613a36040439b6086fb3` | `f5578b02a4650185b5374d994f0f785336d0a1217b54b6b4d1bf72e6edd8bfa0` |
| `exports/env.export.ts` | `d54858e384e84c1dc63706d2cbaeae108c9aed65` | `c25a9d32468c89721a95d64c9ec2aca53013f2b89657e161ab41ceb09bd6c8db` |

The first three were verified by an independent reviewer (R). `env.export.ts` is a **generated data shape marked DCS 2.9.29.27468**: it names fields and their declared primitive types. It is **not** a numeric-enum or
meaning contract and **not** a representative mission fixture. The official singleton FAQ returned HTTP 403 on direct retrieval (R); its search index lists names only, so nothing below rests on it. The repository's
earlier read-only inventory (`reference/d3-environment-characterization/time-and-DCS-reference-audit/`) and the terrain mapping (`reference/d3-terrain-characterization/README.md`, "Mapping to DCS") are reused, not repeated.

## 1. Time: the actual ownership at 78fe5efdb

What the source does (S):

* **Public input.** `engine:frame(milliseconds)` takes an unsigned 32-bit integer (`eech-dc/src/module.rs`, method `frame`, `milliseconds: u32`); the engine passes it unchanged to native `eech_engine_frame`. The host Lua
  decides the number; the campaign never samples a simulation clock of its own.
* **The mirror.** `timeGetTime` (`eech_win32.c`) returns `eech_headless_time_ms ()`, a `uint32_t` mirror (`eech_log.c`) that only `eech_headless_advance_time_ms` (per frame) and `eech_headless_set_time_ms` (boot) change.
  Its **starting value is a function of the configured random seed** (`eech_engine_boot`: `((seed - 1) % 1000000) * 1000`), and `initialise_random_number_generator` reads it through `get_system_time`. So the mirror's
  epoch feeds the shared random stream. A DCS host must keep supplying only the per-frame increment; it must not set the mirror to a DCS clock.
* **Phase.** In `frame` (`eech_engine.c`) the update work runs first, `time_acceleration` times: receive comms, `update_client_server_entities` (which includes the session's time-of-day, resync, weather/wind/lightning
  and sound updates), `update_update_functions`, then the speech buffers, the mobile-sprite light timers and the message log. **After** that, `eech_headless_advance_time_ms (milliseconds)` and `set_delta_time ()` run. So
  every update in frame N derives from the delta of frame N-1 (in the unlocked branch each entity-list walk sees that outer delta divided by the iteration count, see below); `reset_delta_time` (`modules/system/time.c`, run at boot twice) makes the first delta 0.1 s with reciprocal 10.
* **Derived substeps.** In the **unlocked** branch only, `up_update.c update_client_server_entities` reads the outer delta, `set_entity_update_frame_rate` picks `iterations = (int)(delta * rate + 1.0)` and calls
  `set_manual_delta_time (delta / iterations)`, the entity list is walked that many times, and `set_manual_delta_time (outer delta)` follows. Each setter call writes the delta, its reciprocal and advances the delta
  history. The locked branch bypasses all of it. This is scheduling and a derived cache, **not an independently advancing physical clock**. The outer delta is restored; the reciprocal and history are written (not
  transactionally restored), and this document claims no bit-level restoration and no new coverage of those branches. It does not say reset/`set_delta_time` are the only writers.
* **Wall clocks.** `GetSystemTime` (`eech_win32.c`) and `time()` still read the wall UTC. The closed classification (R: e582c414) is that their callers are profile dates, the join log, recording/theme and CPU
  instrumentation: metadata and diagnostics, not a missing physical-world clock input. They are not deterministic and are not removed merely for compiled absence.
* **Core keeps** campaign elapsed time, the time-of-day acceleration, skip-night and the day segment, timers, resync and scheduling policy (`ss_updt.c` order: time of day, resync, then physical weather/wind/lightning,
  then sound). These stay campaign policy even though they consume time.

Closed evidence reused (R, not reproduced): the positive variable-frame comparison 696648d7 (original 765 / driver 235 against candidate D8, Lebanon and Georgia, 900 frames at 100/250/50 ms): all 1800 frame-phase
lines, the 899 previous-delta links and the 8141 state / query / campaign / summary lines per case are exact; the 16 historical boot `weather_draw.w0` diagnostic differences stay declared. It covers **only** those
inputs: no zero-delta, locked-rate, 32-bit wrap, pause, time-acceleration, M6 or full floating-point coverage is added or implied.

### DCS time mapping (proposed adapter policy; nothing implemented)

| need | DCS call (P) | boundary and follow-up |
| --- | --- | --- |
| the frame increment | the host samples `timer.getTime()` inside a `timer.scheduleFunction` callback and forwards the elapsed model time to `engine:frame` | One adapter owns the conversion to **integer ms**. It must carry the fractional millisecond and decide missed callbacks; it must not round each interval independently. Unverified (U): pause, acceleration, long stalls, mission end, second mission, double advancement. The `timer.ts` comment that `getTime` keeps moving while paused is library prose, **not** adopted. Documented DCS probe facts (`m5-dcs-risks.md` F1/F2, R) only say `timer.getTime`/`scheduleFunction` exist in the mission state and MISSION START ran at model time 0. |
| teardown | `timer.removeFunction` | the callback is removed at mission end; M6 checks a second mission starts a fresh module and no callback survives. |
| absolute mission time | `timer.getAbsTime`, `timer.getTime0` | host metadata only; they do **not** replace the campaign calendar, elapsed time, skip-night or timers. Day crossing and reload are M6 checks. |
| the public input domain | none (existing API) | `engine:frame` keeps its existing u32-millisecond domain; this document does not narrow it. The 100/250/50 ms cases are **sampled evidence**, not an accepted-input set. Zero-delta, locked-rate, 32-bit wrap, pause and acceleration are simply **untested** by the closed comparison. Any future DCS adapter cadence, conversion range, overflow and stall behaviour and refusal policy is a separate design decision with its own validation; none is derived from the three tested values and none is imposed on the current API here. |

Remaining Tasks 49 and 50 solvers receive their cached/substep delta from this same phase as an **input** when they move to the host. A host-supplied clock input is not a reason to leave a required physical solver in Core
or to defer it to D5.

## 2. Required environment services at 78: where they live

State of the extracted services (S from the provider docs and sources; checks run by Root are R). "Host" below is the reference EECH World host (`eech-world`, `csrc/env_ref.c`, `src/env.rs`); a DCS host supplies the same
vocabulary from the sources in sections 3 to 5 or refuses by name.

| service | reference host provides | Core keeps | DCS / supported restriction |
| --- | --- | --- | --- |
| weather mode at a point, wind, gust, physical evolution (`weather`, `wind`, `gust`, `step`) | the field and its arithmetic (`docs/d3-environment-provider.md`) | the session carries a **read-only mirror**; the shared-random draws at the original sites; creation of the strike and sound entities | section 3 and 4 |
| ambient rain and wind levels (`ambient`) | levels from the host field at a point | the observations `y`, `alt`, `cloud_base`, in-cockpit; sound entity writes | derived from the host weather/wind answers; no DCS audio |
| terrain sector surface set (sound routine scratch) | host-owned sector enumeration (`docs/d3-environment-sector-surface.md`) | the sound-routine decisions | depends on Tasks 47/48/49/50 terrain contracts (section 6) |
| lightning queue (`lightning_add`, `lightning_update`) | the 32-slot physical queue | what a strike means for the campaign, shared RNG at the original call sites | section 3: with a static global weather the step creates no strike |
| sound lifetime, pause/resume, rotor acoustics, sample metadata, backend availability | lifetime arithmetic, pause/resume decisions, rotor levels and pitch, the sample descriptors, "available = false" | sound entities, speech, network and pause/resume bookkeeping, **all policy gates** | section 5 |

## 3. Weather: the restricted static authored compatibility plan

**What this is.** The technical lead's choice for the initial DCS integration: a **restricted static authored weather compatibility plan**. It is legacy compatibility data. It is **not** a live precipitation
observation and **not** equivalent to a real DCS mission's weather. It is the M5 named data workaround, not the M6 adapter and not observed DCS validation. A dynamic, unpaired or unknown state is refused by name. It does
not change the reference provider, EECH World's default behaviour or any baseline. It needs no weather clone, protocol framework, new API, audio or UI product.

### 3.1 How the Core talks to the provider today (S: `ss_hostenv.c`, `module.rs` ENV_BRIDGE, `env.rs`)

| request | reply the Core reads | effect on the Core's mirror |
| --- | --- | --- |
| `import {state}` (28 fields) | an acknowledgement; the bridge returns without reading any state (`op == 'import'`) | **none**: import only acknowledges, it never rewrites the mirror |
| `init_random`, `init_fixed`, `export`, `step` | a full state: 23 finite floats, modes 0..4, three booleans; checked, then `raw_from_state` | **overwrites** the mirror |
| `weather {x,z}` | three modes 0..4 and a finite transitional state | none |
| `wind {x,z,want_vector}` | a finite speed, and three finite components when asked | none; **no speed/vector norm check** |
| `gust {phase}` | a finite modulation. **Production never asks** (only the test-only fixture `eech_env_fixture_gust_modulation` does) | none |
| `ambient {x,z,y,alt,cloud_base,in_cockpit}` | four finite levels, the two wind levels within 0..1 | none |
| `lightning_add`, `lightning_update` | the 32-slot queue, checked against the request and the mirror | queue mirror |

Imports happen at session creation (`ss_creat.c`, after the attributes), after any write through the weather float, int or vector setters (`ss_float.c`, `ss_int.c`, `ss_vec3d.c`) and after a session unpack
(`ss_pack.c`); an export precedes every pack. The original `init_fixed_impl` (`env_ref.c`) changes only `local_weather_model = FALSE`, `weather_mode = target_weather_mode = mode` and the transitional status `0`; it leaves
every wind and zone field alone. The **reference `step` still evolves the wind zone and the gust phase after `init_fixed`** (they sit outside the local/global weather branch); the lightning timer is decremented only
in the **local** model branch, so after `init_fixed` has made the model global the reference step no longer evolves it. The reference `init_fixed` and `step` are therefore **not** reused as "everything static":
the planned frozen F(M) step below is a separate provider behaviour.

### 3.2 The authored contract K (illustrative; not a schema, not read by any code, not a captured mission)

```lua
return {
  eech_weather = "DRY",            -- M: exactly one of DRY | LIGHT_RAIN | HEAVY_RAIN | SNOW
  static = true,                   -- the author's assertion; the provider cannot verify it (section 3.6)
  paired_with = {                  -- copied from the paired mission by its author; equality only
    ["env.mission.theatre"] = "<copied>",  ["env.mission.weather.name"] = "<copied>",
    ["env.mission.weather.type_weather"] = "<copied>", ["env.mission.weather.atmosphere_type"] = "<copied>",
    ["env.mission.weather.clouds.preset"] = "<copied>", ["env.mission.weather.clouds.iprecptns"] = "<copied>",
  },
  ambient_wind = { minimum = "<finite m/s>", maximum = "<finite m/s>" },   -- section 4.1
}
```

Placeholders are not data. Load refuses by name (proposed tag family `[env-weather:...]`) a missing or extra key, an unknown mode name, a non-finite threshold, `static ~= true`, or a paired value that is not
**equal** to the live `env.mission` value. No DCS number is decoded or given a meaning.

### 3.3 The legacy carrier state: C0 and F(M)

The 23 floats and 5 integers are **compatibility carriers**: the Core's mirror, saves and packets use them. In the planned DCS provider they are **opaque**, not DCS observations, not a field the provider evolves.
`C0` is the session-creation default, exactly what `ss_creat.c` produces when no attribute is given (S: both campaign parsers create the session with `ENTITY_ATTR_END`):

| field | C0 (source expression, `ss_creat.c`; float32 as the engine evaluates it) |
| --- | --- |
| `weather_radius`, `wind_effect_radius` | `40.0 * KILOMETRE` |
| `weather_mode_transitional_period` / `_status` | `60.0` / `0.0` |
| `weather_position` x, y, z | `MID_MAP_X`, `0`, `MID_MAP_Z` |
| `weather_velocity` x, y, z | `knots_to_metres_per_second (300.0)`, `0`, `-knots_to_metres_per_second (100.0)` |
| `wind_effect_position` x, y, z | `0`, `0`, `0` |
| `wind_effect_velocity` x, y, z | `knots_to_metres_per_second (240.0)`, `0`, `knots_to_metres_per_second (350.0)` |
| `wind_direction` x, y, z | `normalise_any_3d_vector (0 - MID_MAP_X, 0, 0 - MID_MAP_Z)` |
| `wind_gusting_value` | `0` (the record is `memset` to zero) |
| `wind_minimum_speed` / `wind_maximum_speed` | `DEFAULT_MIN_WIND_SPEED` / `DEFAULT_MAX_WIND_SPEED` (`knots_to_metres_per_second (5.0)` / `(25.0)`) |
| `lightning_timer` | `ONE_MINUTE` |
| `local_weather_model`, `weather_mode`, `target_weather_mode` | `TRUE`, `DRY`, `DRY` |
| `weather_increasing`, `wind_increasing` | `FALSE`, `FALSE` (zero) |

The only map-derived inputs are `MID_MAP_X` and `MID_MAP_Z` (`en_world.c`: `min + (max - min) * 0.5`, float32), from the map facts the host already gave the terrain provider (`campaign_terrain_install_map_facts`).
The import request carries no map, so the planned provider must compute C0 from those same facts and refuse if it has none. The numbers are produced by evaluating these expressions in the engine's order, not retyped
here; a one-time comparison with the reference provider's export taken at creation is the proposed pin.

`F(M)` is `C0` with exactly three changes: `local_weather_model = FALSE`, `weather_mode = target_weather_mode = M`, `weather_mode_transitional_status = 0`. This is what the reference `init_fixed_impl` produces from `C0`; every
other carrier stays as in `C0` and never changes.

### 3.4 Boot, import and the replies (planned DCS provider)

The source order at a campaign boot (S): the parser creates the session and, **unconditionally when no session exists, immediately calls `set_session_random_weather_mode`** (`parser.c`, `parsgen.c`: ten shared-random
draws, then `init_random`); a campaign file's rain tag with value 0 calls `set_session_fixed_weather_mode (DRY)`; later `process_host_session_setup_options` (called from `boot`, `eech_engine.c`) applies the
configured campaign weather: GOOD = DRY, FAIR = LIGHT_RAIN, POOR = SNOW in winter and HEAVY_RAIN otherwise (`sparm_sc.c`), each as `init_fixed`; RANDOM calls `init_random` again **only if** the mirror still says
`local_weather_model`. That setting is `session_campaign_weather` (free flight: `session_free_flight_weather`) in the binary `game.cfg` that `boot` loads from the `cohokum` directory (`load_global_options_data`);
its default is RANDOM (`global.c`), and no engine config field, argument or INI key in this source sets it.

**The request sequences that reach a fixed weather equal to M** (the provider sees only `init_fixed (mode)`; it cannot tell which source caused it, and the plan adds no separate configuration pre-validation, boot
control or restriction): (a) a campaign file whose rain tag is 0 calls `init_fixed (DRY)` (`parsgen.c`), which is compatible when M = DRY; (b) the configured campaign weather GOOD / FAIR / POOR calls
`init_fixed` with DRY / LIGHT_RAIN / HEAVY_RAIN (winter off) or SNOW (winter on, the engine config's `winter`), compatible when it equals M. Either source is enough. Once any compatible fixed request has been
answered the mirror says `local_weather_model = FALSE`, so a RANDOM or missing `game.cfg` (the default is RANDOM) issues **no** later `init_random`: with M = DRY and rain tag 0 such a boot reaches STATIC and is
accepted. A RANDOM or missing `game.cfg` fails **only when neither source has established a compatible fixed sequence**. No code change is needed; where the configured source is wanted, producing the `game.cfg` is a
packaging step (the game's own options screen writes it) and there is no first-class boot control for it.

**The exact obstacle to "init_random refuses".** The first `init_random` is made by the Core at every parser boot, before any configured fixed weather exists, so refusing every `init_random` would refuse every campaign
boot. The plan therefore does **not** turn that request into static data silently:

| step | planned provider | reply and what it does to the Core mirror |
| --- | --- | --- |
| first `import` | accept only if bitwise equal to C0 (28 values by bit pattern); P := C0, phase BOOT. Anything else, including a session created with attributes, is refused `[env-weather:import_not_creation_default]` | acknowledgement; mirror unchanged |
| first `init_random` (phase BOOT, P = C0) | a **no-effect** request: the Core's draws are made and ignored (shared-random order unchanged); answers **C0 unchanged** and arms the requirement below | state reply C0: the first mirror refresh, identical to what it already holds |
| any other `init_random` (a second one, or P != C0) | refused `[env-weather:random_weather]`. This is what a RANDOM setting produces **only when no earlier compatible fixed request made the model global**: the mirror then still says local model, so the Core asks again; a wrong-mode fixed request and an actual later `init_random` always refuse | none |
| `init_fixed (m)` | `m` must equal M, else refused `[env-weather:fixed_mode_mismatch]`; each fixed request (the file's rain tag and the configured setting) is checked on its own, so a file with rain tag 0 needs M = DRY; answers F(M); P := F(M), phase STATIC | state reply F(M): the mirror is now F(M) |
| `weather`, `wind`, `gust`, `ambient`, `step`, `export`, `lightning_*` before phase STATIC | refused `[env-weather:fixed_weather_not_applied]` (covers a boot where neither the campaign file nor the configured setting sent a compatible fixed request, for example a RANDOM or missing `game.cfg` without rain tag 0) | none |
| later `import` (setter, unpack, load) | accepted only if bitwise equal to P, else refused `[env-weather:import_mismatch]` | acknowledgement only; it never rewrites the mirror |
| `export` | F(M) | state reply: refreshes the mirror |
| `weather {x,z}` | `current = target = simple = M`, `transitional_state = 0`, global (non-local) | none |
| `step` | **not** the reference step: F(M), `expired = false`, no `need_draw`, no strike, no `new_timer`; no zone, wind-zone, gust-phase or lightning-timer evolution (the mirror therefore shows constant carriers) | state reply F(M) every update |
| `gust` | not asked by production; if asked, modulation `0.0` (the unity factor `1 + 0`) | none |
| `lightning_add` | refused `[env-weather:unsupported_strike]`: nothing in the static plan creates a strike (S: strikes come only from a step expiry), so a request means the world is not static | none |
| `lightning_update` | answers the empty queue (no slot in use) | queue mirror stays empty |

No operation of the vocabulary can load a non-empty queue (`import` carries only the 28 fields, S); one that did would be refused. This says nothing about whether DCS has a lightning API. A boot with no compatible fixed request
(a RANDOM or missing `game.cfg` and no rain tag 0, or a different mode) ends in a refusal, never in a substituted answer. The existing reference queue and lifetime implementations stay unchanged.

### 3.5 What is and is not an observation

The only DCS observations in the weather plan are the **paired equality checks at load** and the wind sample of section 4. M, F(M), the carriers and the ambient thresholds are legacy compatibility data. A mission whose
real weather differs from M will look different from what the campaign believes: that divergence is a declared restriction, never hidden. A synthetic K is not a captured DCS mission.

### 3.6 Native questions, kept separate from the choices above (none answered here)

(U) Is `env.mission` readable in the mission scripting state and unchanged after load; do the paired fields have the value types the generated shape declares; is `static = true` true of a real mission, and what a
mission-authored weather change would do to it; the numeric meaning of `type_weather`, `atmosphere_type`, `clouds.iprecptns`; units of `clouds.base`/`thickness`. The M6 check of this section is the restricted outcomes of the table above against a real boot, as each source of the
fixed sequence (rain tag, configured setting, neither) and as each accepted or refused import; it does not imply general reload, second-mission, save or client-state support, which succeed only where the table accepts them. No enum is interpreted and no comment is treated as evidence.

## 4. Wind and ambient: one observation plane

**Consumer inventory, by route (S).** The wind request is `wind {x, z, want_vector}`; the hosted `get_session_wind_velocity_at_point` passes **x and z only** and returns the vector with `y = 0` from the reference
provider. Active host route: the weapon-step lookup `CAMPAIGN_STEP_LOOKUP_WIND` (`wn_move.c`) calls it, builds a 3-D point whose altitude is dropped, and uses the **vector**. Also source readers: `sl_move.c` (smoke),
`3d_draw.c`, `speech.c`, the `ss_updt.c` ambient routine (asks `ambient`, not `wind`), `ha_instr.c`, and the weather readers (`get_simple_session_weather_at_point` in MFDs, wipers, `co_eo.c`). The listing is an **inventory**;
their reachability in the supported public frame is not established here, and this document adds no liveness or dormancy result.

**The chosen plane (planned DCS provider; one declared compatibility observation plane).** For each `wind` request:

1. map the requested EECH `{x, z}` to the DCS horizontal point with the agreed terrain coordinate transform T (`reference/d3-terrain-characterization/README.md`: EECH `x` east, `z` north; DCS `Vec2 {x = north, y = east}`
   plus the map origin); a missing transform or a non-finite result is refused;
2. `h = land.getHeight` at that point (P: `land.ts`); a non-finite `h` is refused;
3. **one** `atmosphere.getWind` (the steady variant, P: `atmosphere.ts`) at that horizontal point and height `h + 10` metres. The 10 m is **our chosen legacy plane**, not a DCS standard and not the requesting
   aircraft's altitude (which the request does not carry); no altitude field or API is added;
4. from that **one** sample derive the whole reply: the horizontal components go through the linear part of T back into EECH axes (east, north), the vertical component of the reply vector is `0` (the legacy
   shape), and `speed` is the scalar horizontal magnitude of that projected sample. The DCS vertical component is discarded. The tuple is coherent by construction **before** narrowing;
5. encoding: the provider computes in double and refuses, by name, any non-finite component and any speed or component above `float32` max; the Core bridge then formats `%.17g` and narrows with `nearest_f32`
   (S: `module.rs`), so each of the three numbers is within the nearest float32. **The Core still checks only that the speed is finite and, when a vector is requested, that its three components are finite**; the scalar/vector
   coherence is the planned provider's obligation, not a Core validation;
6. no legacy gust factor and no turbulence is applied on top (`gust` is unity, section 3.4); the carrier `wind_gusting_value` is not advanced. Every `wind` request takes its **own** sample; nothing is cached across
   requests; the observation time is the native DCS state at the moment of the call.

Steady `getWind` is chosen as an **observation policy**: the declaration says "steady, excluding turbulence", which does not prove time invariance, absence of internal randomness, determinism or campaign replay
reproducibility, and none is claimed. Turbulence (`getWindWithTurbulence`) is not used.

### 4.1 Ambient: an observation-based acoustic helper

The reference `env_ref_ambient` resamples its **own** legacy field, so it is not called. The planned DCS provider answers `ambient` with the same acoustic arithmetic the extracted routine already uses
(`env_ref.c ambient_at`), fed by the **same plane, transform and steady policy**, sampled **once for that request**: the wind speed is the scalar of step 4 above; rain comes from M as a global mode
(`LIGHT_RAIN` fills the first rain level, `HEAVY_RAIN` the second, `DRY`/`SNOW` neither), only when `y < cloud_base` and (in cockpit or `alt < 180`), attenuated outdoors by `(1 - alt/180)` applied twice for equal modes, as the
extracted arithmetic does; the wind levels come from the speed against the authored `ambient_wind` minimum/maximum. `y`, `alt`, `cloud_base` and `in_cockpit` are the Core's observations: `cloud_base` is the
campaign's own 3-D environment height and is **not** replaced by `env.mission.weather.clouds.base`, whose units are not inferred from the generated shape. The reference gust and the DCS sample are never both applied.

Validation of the thresholds against that formula: both finite, representable as float32, `0 <= minimum < maximum` strictly, and the float32 difference `maximum - minimum > 0`; the division happens only in the branch
`minimum < speed < maximum`, so no denominator is zero. The wind levels then lie in 0..1 as the Core requires. No DCS audio or backend is needed.

### 4.2 Native questions (kept separate; M6)

(U) `Vec3` axes and units of `getWind` at `h + 10` m; its steady character; ground and point-below-terrain behaviour; the precision at the float32 narrowing; the horizontal mapping of `Vec3` to the terrain
transform; the meaning of the generated `wind.atGround/at2000/at8000` fields (not used). The 10 m plane and the unity gust are policy choices, not claims about DCS.

## 5. Sound, speech and the physical backend (high level)

* **Moved to the host (R/S):** ambient levels, the sound-effect lifetime, pause/resume decisions, rotor acoustics and sample **metadata**, and the backend **availability** (always `false`; the Core refuses `true` by
  name). **Stays in Core:** sound/speech/entity/network/pause/resume policy, the option and locality gates, and the creation of the sound entities.
* **Removed from the campaign build (S at 78, plus R actual):** `modules/sound/{dirsound,dsound,dsound3d,soundhlp,soundman}.c` are not in `sources.txt`; the small `aphavoc/source/appsound/snd_nobackend.c` answers what the
  legacy callers handled without a backend (initialise and device list `FALSE`, creators `NULL`, empty handle set) and stops by name for a live native handle (`[env-sound:unsupported_native_handle]`) or for the
  sample store, listener and handle table (`[env-sound:unsupported_native_backend]`). The hosted `se_snd.c play_local_sound` keeps the Core gates and decides at entry, before the physical mixer: a held handle stops
  by name, no handle is an empty request; the mixer is compiled only for the original game. Reported by Root at 78 (R, not reproduced): the independent actual review
  `independent-environment-sound-backend-detach-actual-78fe5efdb-20261101.json` (SHA-256 `03369ba4...e418`, findings `[]`, a finite backend closure) covers the default Core archive and `se_snd.o` (all five backend objects
  and the entry references absent, the adapter present, no observer), the first three actual native runs with controllers 0, 6 complete inputs and the 74 payloads unchanged, the whole unarmed run
  (5191384 bytes, 17283 rows) exactly equal to the reused `48307ff...` including its endings, and the two fatal raws (467 and 471 bytes, distinct causes, no post-poison query); Linux and Windows guards, format and
  strict lint 0, and all 9 current CI checks pass at 78. The ordinary shipping 78 build (33391, actual 0) exists and the full first Task 51 pack was launched and is running: its 40 checks, all 8 and the Tacview
  completion are **pending**, as are the final review and the normal delivery. This is a finite backend closure, not a whole-D3 or whole-Task-51 acceptance.
* **Not removed, to avoid an overclaim:** `snd_data.c` still compiles its native sample loader bodies (`load_application_sound_samples`, `load_side_dependant_application_sound_samples`, the file-system open/close and
  header reads) behind `application_sound_system_enabled`. That flag is FALSE because the host answers "not available" and the Core refuses an "available" answer; **if it were true** the side-dependent loader
  would perform file and header work (`safe_fopen`/`fread`), and `allocate_source_sound_samples`/`create_source_sound_sample` would stop by name at the adapter. The document does not say the loaders are absent, and
  headless silence is not used to waive any retained loader, device, render or network branch.
* **DCS:** no DCS audio backend is needed for campaign decisions. The DCS host declares its native audio availability (the same `sound_backend` answer) and supplies the logical sample descriptors
  (`sound_catalogue`); a DCS host that wanted real playback would be a new service, not an extension of this mapping. No audio or UI product is created solely for the campaign.
* **Remaining coupling (not claimed removed):** the original game's `modules/sound` and its `#else` callers; the unlisted family `*_snd.c` creators; the COM stubs and compat headers (now unreferenced by the Core);
  the observer's reads of the two flags in the adapter.

## 6. Terrain, models, spatial: dependencies, not copies

Terrain elevation, surface class and the point/fast/map-facts subset are Task 47/48 contracts; the terrain README's "Mapping to DCS" (named surface table, central-difference normal workaround, `land.getIP` for
impact, terrain-only `land.isVisible`) is reused unchanged. Object and tree collision, model hierarchy, muzzle/viewpoint/target points and physical route data are **Tasks 49 and 50's final contracts**; this document
does not copy them into Task 51, does not assert that DCS lacks a corresponding call (these four files do not cover objects), and is not authority to retain their native solvers in Core or to defer them to D5.

## 7. Follow-ups: M6 plan items, none run

M5 asks for reviewed calls and a workaround plan, not a live campaign. The planned policy choices are in sections 3 and 4; what remains is native verification, each with its exact unanswered check.

1. **Time:** pause, accelerated and stalled callbacks; fractional-ms carry and integer rounding; mission end teardown; a second mission; no double advancement; track replay. The adapter's own cadence and refusal policy is a
   separate design to be validated.
2. **Weather contract:** `env.mission` availability and stability; the pairing against a real mission; whether `static = true` holds; the restricted boot outcomes (rain tag, configured setting, neither; accepted and refused imports, no general reload or client-state support implied) of
   section 3.4 against a real boot.
3. **Wind and ambient:** `Vec3` axes and units at `h + 10` m, steady character, ground behaviour, float32 precision, and the horizontal mapping.
4. **Backend:** the host's availability answer and sample descriptors in a DCS host.

## 8. What this document does not claim

No DCS execution, no adapter, no schema or framework, no new clock, weather, audio or UI product, no change to any baseline or tolerance, no whole Task 51, Tasks 49/50, time-service, mapping or D3 acceptance, no
shipping 40/all 8, Tacview, current CI, final review or delivery result. D4 and D5 are unstarted. The historical TypeScript port, the unfinished call-map branch and `ee-dcs` are neither scope nor evidence here.
M5 v3 and the Root ledger (115 at this revision) are Root's live authority; this revision does not touch the running shipping pack. The mapping review has not accepted this document.
