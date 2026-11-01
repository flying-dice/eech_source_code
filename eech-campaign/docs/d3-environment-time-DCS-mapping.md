# D3 (EEDCS-51): time ownership and the DCS API / data mapping of the environment services

Status: **SOURCE DISPOSITION AND PLAN. Documentation only: no C, Rust, Lua, build, test, baseline, oracle, flag, target or configuration changed. Not a DCS runtime result, not an acceptance of Task 51 or D3.**
Source read: `78fe5efdb5cbaff904e3b645d680acbd7776597d` (this document edits nothing else). Evidence labels: **S** = read from that source by the author of this document; **R** = reported by Root, not reproduced
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
  every update in frame N uses the delta of frame N-1; `reset_delta_time` (`modules/system/time.c`, run at boot twice) makes the first delta 0.1 s with reciprocal 10.
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
| zero / locked / wrap | none | not claimed; the adapter never feeds a value outside what the closed cases covered without a new reviewed case. |

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

## 3. Weather: a concrete bounded mapping and an explicit data workaround

**What the Core needs (S).** `weather {x, z}` answers `{current_mode, target_mode, transitional_state, simple_mode}`, with modes `WEATHERMODE_DRY | LIGHT_RAIN | HEAVY_RAIN | SNOW` (`modules/3d/3denv.h`). Its consumers are the
ambient rain levels (`ss_updt.c`), the avionics/cockpit weather readouts and wipers, and heavy-rain sensor effects (`co_eo.c`). A global (non-local) model has **no zone, ring or lightning**: `init_fixed_impl`
(`env_ref.c`) sets `local_weather_model = FALSE`, `weather_mode = target_weather_mode = mode`, transitional status 0, and touches nothing else; `step_impl` then performs the global transition only when
`weather_mode != target_weather_mode`, and strikes (the `expired` / `strike` answers) occur only inside the local-model branch with a non-zero radius.

**What DCS declares (P).** `env.mission.weather` has `atmosphere_type: number`, `type_weather: number`, `name: string`, `clouds.{base, density, iprecptns, thickness: number; preset: string}`, `cyclones`, `fog`,
`enable_fog`, `dust_density`, `enable_dust`, `groundTurbulence`, `qnh`, `season.temperature`, `visibility.distance`, `halo.preset`, `modifiedTime` and `wind.{atGround, at2000, at8000}.{dir, speed}`. The pinned
files give **names and primitive types only**. They do not give: what any numeric value means (precipitation kind, `type_weather`, `atmosphere_type`), units for `clouds.base` or `thickness`, how `cyclones` is encoded,
whether the table changes while a mission runs, or whether the table is readable in the mission scripting state at all (U). No API in the three call files supplies precipitation, rings, transitions or lightning;
`atmosphere.ts` supplies wind, temperature and pressure only. **No value is interpreted below.**

**The workaround: a static, authored, paired weather contract (proposed; not implemented, not reviewed).** The campaign mission author supplies, beside the DCS mission, an explicit host-owned declaration of the
EECH weather the campaign will see, and the host refuses to run when the mission it is paired with does not match. DCS numbers are compared for **equality only**, never decoded.

```lua
-- ILLUSTRATIVE ONLY: not a schema, not read by any code, not a captured DCS mission. Placeholders must be copied
-- verbatim from the paired mission by its author; nothing here asserts what any DCS number means.
return {
  eech_weather = "DRY",                          -- exactly one of: DRY | LIGHT_RAIN | HEAVY_RAIN | SNOW (the EECH mode names)
  paired_with  = {                               -- copied from env.mission.* at authoring time; equality-checked at load
    ["env.mission.theatre"]                      = "<copied>",
    ["env.mission.weather.name"]                 = "<copied>",
    ["env.mission.weather.type_weather"]         = "<copied number>",
    ["env.mission.weather.atmosphere_type"]      = "<copied number>",
    ["env.mission.weather.clouds.preset"]        = "<copied string>",
    ["env.mission.weather.clouds.iprecptns"]     = "<copied number>",
  },
  wind = { minimum_speed = "<m/s, authored>", maximum_speed = "<m/s, authored>" },   -- the ambient-level thresholds only
}
```

Rules (all host-side, applied **before** any substituted physical answer):

1. **Load.** The file must name exactly one EECH mode and carry every `paired_with` key; an unknown mode name, a missing or extra key, or a non-finite wind threshold is refused by name. Each paired key is compared to the
   live `env.mission` value by equality; any difference is refused by name (a proposed tag `[env-weather:mission_pairing_mismatch]`). A mission that is not paired is therefore a refusal, not a guess.
2. **Provider.** The DCS provider's `weather` answer is the authored mode as a static global: `current = target = simple = mode`, `transitional_state = 0`. `step` evolves no weather and creates no strike, so the
   lightning queue stays empty. The Core's mirror is refreshed from these answers (host independently owns the field; the Core's session carries its read-only mirror).
3. **Refusals by name (never substituted):** `init_random` (a campaign file or session parameter asking for random local weather); `init_fixed` with a mode other than the authored one; an `import` whose weather
   fields are neither the session-creation defaults (`ss_creat.c`: local model, 40 km radius, DRY) nor the authored static state, which is how a loaded or client weather from another host is detected. The defaults
   carry a moving 40 km zone; the DCS host must not answer from them.
4. **Wind** is not part of the authored mode (section 4); only the two ambient thresholds are authored.
5. **Dynamic weather.** Nothing in the pinned files lets the host *detect* a mission whose precipitation changes at run time (U). The restriction is therefore declared, not checked: the contract asserts "static", the
   equality pairing is the only guard, and the M6 plan must say what a mission-authored change would do to the pairing. The campaign never receives a DCS precipitation observation, and no result may be reported as
   one.
6. **Not a DCS observation.** The authored mode is a **legacy-compatibility declaration** (it exists because the Core's consumers and saved sessions use the EECH vocabulary). The DCS observations are only wind,
   temperature and pressure (section 4) and the paired equality checks. The legacy mirror fields (`weather_radius`, positions, velocities, `wind_effect_*`, `lightning_timer`, `wind_direction_*`) are mirror
   carriers the DCS host fills with the static or default values it declares; they carry no DCS meaning.

**Unknowns that remain and are not fabricated:** the meaning of the numeric weather flags and the units of the cloud fields; whether `env.mission` is live-updated or readable in the target state; whether a mission can
change weather at run time and how that would be observed; whether temperature/pressure from `getTemperatureAndPressure` should ever replace any EECH constant (no provider operation asks for either today (S)). A DCS mission whose real weather differs from the authored mode will look different from what the campaign believes; that divergence is a declared restriction, not a defect hidden by this
mapping. A synthetic authored file is **not** a captured DCS mission and must not be reported as one.

## 4. Wind: the one weather quantity with a pinned DCS call

`wind {x, z, want_vector}` answers `{speed, vector}`. In the reference host the speed already includes the original's gust modulation (`env_ref.c wind_at`); the Core's consumers are weapon drift (`wn_move.c`), smoke
(`sl_move.c`), the 3D draw (`3d_draw.c`), speech wind callouts (`speech.c`), the ambient wind levels (`ss_updt.c`) and a cockpit instrument.

* DCS (P): `atmosphere.getWind(point)` = steady wind, `getWindWithTurbulence(point)` = instantaneous wind including turbulence, both `Vec3` in "local DCS coordinates and metres" returning world-axis metres per second.
  The declaration comment says points at or below terrain return zero for the turbulent variant; that is **not** adopted as measured behaviour.
* Proposal: use the steady `getWind` first (it is not random, so a campaign run stays reproducible) and make the turbulence choice an explicit, reviewed per-consumer decision; the reference host's gust expression
  is a legacy compatibility calculation and is not a DCS observation. Both choices stay open (U) until M6 measures them.
* The host builds the `Vec3` sample point from the Core's `{x, z}` plus an altitude it must choose and state (ground height from `land.getHeight` plus a stated offset, or the consumer's own altitude where the request
  carries one); today's request has no altitude, so this is a **selected-point decision**, not a given. The EECH-to-DCS axis transform is the terrain mapping's (EECH `x` east, `z` north; the DCS `Vec2` there is
  `{x = north, y = east}` plus the map origin). The `Vec3` axis order and units are P-declared only as "world axes, metres"; M6 verifies all three axes, the ground and above-ground points and the units.
* The scalar `speed` and the vector must be mutually consistent and finite (the Core refuses a malformed answer by name, `[env-provider:wind.answer]`, S).

## 5. Sound, speech and the physical backend (high level)

* **Moved to the host (R/S):** ambient levels, the sound-effect lifetime, pause/resume decisions, rotor acoustics and sample **metadata**, and the backend **availability** (always `false`; the Core refuses `true` by
  name). **Stays in Core:** sound/speech/entity/network/pause/resume policy, the option and locality gates, and the creation of the sound entities.
* **Removed from the campaign build (S at 78, plus R actual):** `modules/sound/{dirsound,dsound,dsound3d,soundhlp,soundman}.c` are not in `sources.txt`; the small `aphavoc/source/appsound/snd_nobackend.c` answers what the
  legacy callers handled without a backend (initialise and device list `FALSE`, creators `NULL`, empty handle set) and stops by name for a live native handle (`[env-sound:unsupported_native_handle]`) or for the
  sample store, listener and handle table (`[env-sound:unsupported_native_backend]`). The hosted `se_snd.c play_local_sound` keeps the Core gates and decides at entry, before the physical mixer: a held handle stops
  by name, no handle is an empty request; the mixer is compiled only for the original game. R at 78: the default Core archive and `se_snd.o` actually audited (all five backend objects absent, adapter present, no
  legacy volume/position/pitch/create/get-playing/`frand1` references in `se_snd.o`); candidates 23 and 24 each stopped with the strict named fatal and no query after the poison; the whole controls, the default
  actual independent review, the unarmed 120 comparison, shipping 40/all 8, current CI and delivery are **pending**.
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

M5 asks for reviewed calls and a workaround plan, not a live campaign. Each item below keeps its exact unanswered check.

1. **Time:** pause, accelerated and stalled callbacks; fractional-ms carry and integer rounding; mission end teardown; a second mission; no double advancement; track replay.
2. **Weather file:** is `env.mission` readable and unchanged after load in the mission scripting state; the real value domains of `type_weather`, `atmosphere_type`, `clouds.iprecptns`, `clouds.preset`; whether the
   weather can change at run time and how to detect it; the pairing refusal tag and its behaviour on a mismatch.
3. **Wind:** `Vec3` axes and units at three altitudes, the sample-point choice, steady versus turbulent per consumer, zero-wind and below-terrain behaviour.
4. **Backend:** the host's availability answer and sample descriptors in a DCS host; the 78 candidate controls and default review by Root.

## 8. What this document does not claim

No DCS execution, no adapter, no schema or framework, no new clock, weather, audio or UI product, no change to any baseline or tolerance, no whole Task 51, Tasks 49/50, time-service, mapping or D3 acceptance, no
shipping 40/all 8, Tacview, current CI, final review or delivery result. D4 and D5 are unstarted. The historical TypeScript port, the unfinished call-map branch and `ee-dcs` are neither scope nor evidence here.
M5 v3 and ledger 114 are Root's live authority.
