# D3 environment services (EEDCS-51): the host-owned physical weather and wind field

Status: **SOURCE PREPARED. UNBUILT. UNRUN. UNREVIEWED.** The author of this change has no C compiler, Rust toolchain or Lua; nothing here was compiled or executed. Root builds, runs and reviews.
This document says what the new source does and does not do. It claims no physical equivalence, no host authority proof, no cutover and no completion of Task 51 or D3.

Base: branch `codex/d3-environment-services`, clean pushed `b8874cd6c` (reference-evidence only, no production change). The measured original references, unchanged and unrun here: the original
characterisation `original99352e331` (164-case oracle, 120 s and variable-frame expectations) and the read-only observer evidence `2804ca4f`/`765`/`235`.

## What moved

The physical weather and wind **field** and every question about and change to it are now the **host's**. The campaign (the original session code in the DLL) keeps no weather or wind
arithmetic. It asks the host, synchronously, through the provider `dc.boot { environment = host.env_provider () }` (the same shape as the terrain provider of EEDCS-47), and keeps
only a labelled read-only **mirror** of the field in the session record.

| Moved to the host (original C as the reference host's kernels, `eech-world/csrc/env_ref.c`) | original source |
|---|---|
| weather at a point: the ring model (local weather model) or the global/fixed modes, the winter flag | `session.c get_session_weather_at_point` |
| the simple half-transition choice | `session.c get_simple_session_weather_at_point` |
| wind at a point: the drifting zone, the gust modulation, the direction | `session.c get_session_wind_velocity_at_point`, `get_wind_gusting_modulation_value` |
| random initialisation from the campaign's ten draws | `session.c set_session_random_weather_mode` |
| fixed initialisation | `session.c set_session_fixed_weather_mode` |
| the physical evolution: weather and wind drift, map-edge bounces, radius expand/contract, gust phase wrap, global transition, wind direction | `ss_updt.c update_transitional_weather` |
| the lightning timer: decrement, expiry, the new timer's arithmetic | `ss_updt.c update_lightning_effect` (the timer half) |

The kernels keep the original types and expression order (a float field times a double constant, a double expression stored in a float field, the single-precision `PI`), and run under
round-toward-zero as the engine does. The host saves the caller's complete floating-point environment (x87 control and status words, MXCSR) on entry to **every** operation, sets
round-toward-zero, and restores the caller's on **every** exit, early refusals included. Nothing was re-formulated.

## What stays in Core (the campaign), and why

* The campaign clock: elapsed time accumulation, skip-night, resync, scheduling, the order of the session update (**time of day, resync, physical weather/wind/lightning, sound**, with the
  **previous frame's delta**, as the original). The update order and the delta are unchanged.
* **Every draw of the shared random word and the libc `srand` interleaving.** `set_session_random_weather_mode` still makes its ten draws, in the original order and at the original site (rain:
  `frand1`, `rand16`, `frand1`, `frand1`, `sfrand1`; wind: the same five), and hands them to the host as values. No speculative draw is consumed. The lightning timer's one draw
  (`frand1`, at the original site straight after the expiry) is made by the campaign when the host reports the expiry (below). There is no continuation framework and no public re-entrant
  draw ABI.
* The creation of the lightning strikes and the two sound entities, in the original order with their pitch draws and the entity-index interleaving (`ss_updt.c create_lightning_effects`).
  **Open**: the sound/terrain-scratch bridge (`update_session_sound_effects` and the shared `terrain_types_in_sector`) is not moved and remains an explicitly open part of Task 51/D3.
* Route, geography, task policy, consequences, the time-of-day input to rendering and the player-facing consumers' use of the answers (they now read the host's answers through the original
  function names).

## The provider protocol

One Lua function; the request is a table with `op` and `winter` (the engine's season, read where the original read it, at each question) and the answer is a table. All numbers in a request
are Lua numbers handed over unchanged (a negative zero and a subnormal arrive bit for bit; the reference host reads them through the exact reader of `terrain.rs`, never through a path that
rounds a near-integer). Numbers the campaign sends are exact single-precision values (draws are doubles).

| op | request | answer |
|---|---|---|
| `import` | `state` | `{}` (the host takes the campaign's field as given) |
| `export` | | `{ state }` |
| `init_random` | `draws` (10), `config` (rain/wind radius min/max, speeds), `map` | `{ state }` |
| `init_fixed` | `mode` | `{ state }` |
| `weather` | `x`, `z` | `{ current_mode, target_mode, transitional_state, simple_mode }` |
| `wind` | `x`, `z`, `want_vector` | `{ speed, vector = {x, y, z} }` (vector only when asked) |
| `gust` | `phase` | `{ modulation }` |
| `step` | `delta`, `server`, `in_cockpit`, `camera`?, `lightning_draw`?, `map` | `{ need_draw = true, timer }` **or** `{ state, expired, strike?, new_timer? }` |

`state` is a table of the 23 floats and 5 integers of the field by name (`weather_radius`, ... `lightning_timer`; modes as integers 0 to 4, the three flags `local_weather_model`,
`weather_increasing`, `wind_increasing` as booleans); the names are in `eech-dc/src/module.rs` (`ENV_FLOAT_NAMES`, `ENV_INT_NAMES`) and `eech-world/src/env.rs` and must agree.

**STEP and the one lightning draw.** The campaign does not know in advance whether the timer expires. The host decrements and, on a server expiry with no `lightning_draw` given, answers
`need_draw` **without changing anything** (`timer` carries the decremented value). The campaign then draws `frand1` at the original site and asks again with `lightning_draw`; the host answers
with `expired`, `strike` (the weather centre **before** it moved, as the original struck), `new_timer` and the state after the update. The campaign sets the timer through its own setter and
creates the strikes and sound entities in the original order. A client's expiry needs no draw (the server sets the next timer). A second `need_draw` is a refusal.

### Refusals and markers (atomic; nothing falls back to another solver)

A missing provider is refused at boot (`[env-provider:missing]`, after the terrain's check) and at the engine's first question. The first line of the error the engine stops with carries
`[env-provider:<marker>]`:

* `<op>.callback` the provider raised an error (its text follows);
* `<op>.answer` the answer is not a table, or a number is missing, not finite, or a mode/flag is out of its domain; a `need_draw` re-ask; a server expiry without the draw;
* `<op>.state` the field in the answer is missing a number, not finite, a mode outside 0 to 4, or a flag that is not a boolean.

The campaign's mirror is written only after an answer has been checked in full. The reference host's kernels work on a copy of the field and commit it only when the operation has finished,
so a refused operation changes nothing on either side. Domain refusals in the kernel: a gust phase outside `[0, 1)` (the original asserts there), a non-finite field or camera, a mode or flag
out of range on import, a draw the original's draw macro could not give, a negative delta, a weather-zone update with no camera, and a gust phase too large for the original's wrap loop to end.

### Transport

Engine callbacks run under round-toward-zero. The Lua bridge in `module.rs` validates the reply in Lua, formats numbers exactly (`%.17g`) and the Rust side parses them with
`nearest_f32`, as every host number the engine takes. The host reads request numbers directly from Lua (exact, signed zero preserved) and replies with Lua numbers.

## The mirror

The 23 floats and 5 integers of the session record that were the physical field (`weather_radius`, `weather_mode_transitional_period/status`, the weather position and velocity, the wind
radius, gust, limits, direction, position and velocity, `lightning_timer`; `weather_mode`, `target_weather_mode`, `local_weather_model`, `weather_increasing`, `wind_increasing`) are now a
**read-only mirror** of the host's field:

* written only from checked host replies (initialisation, `step`, `export`);
* the setters (`ss_float.c`, `ss_int.c`, `ss_vec3d.c` for these types) push the record to the host with `import` after writing (a debug event, a test, a loaded value);
* session creation (`ss_creat.c`) imports the session's defaults; `unpack` imports the unpacked record; `pack` (client replication and saves) first refreshes the record with `export`;
* the player/render/cockpit consumers call the original function names (`get_session_weather_at_point`, `get_session_wind_velocity_at_point`, ...) which now ask the host: **one authoritative
  field**; the campaign has no second copy it computes from.

The compiled client/save paths are explicit source dispositions: they work through the mirror's export/import (the host must support both), or refuse clearly through a provider that does not.

## Weapon wind (D2)

`wn_move.c CAMPAIGN_STEP_LOOKUP_WIND` answers the weapon step's wind lookup with `get_session_wind_velocity_at_point`, which is now the host's question. A real launched projectile therefore
flies in the host's wind: a changed field changes its flight, a withheld or malformed wind answer stops it with the `[env-provider:wind.*]` marker. The proof (a launched weapon in a changed,
withheld and malformed host wind) is a Root gate, not run here; the source makes it possible without any other edit.

## The characterisation fixture and the observer

The test-only fixture (`eech_env_fixture.c`, feature `env-fixture`) and observer (feature `env-observer`) are ported, not weakened. Witnesses set synthetic physical state and **push** it to the
host through the campaign's own `import` (`env_push`), a restore pushes the saved record back, and a snapshot or fingerprint **re-reads** the host with the campaign's own `export` (`env_pull`), so
the fingerprint compares the host's field, not a stale copy. Against the original session code the push and re-read are no-ops, so the file is still the original's characterisation. The case
tables, groups and the 164-case oracle are unchanged; the printed grammar is unchanged. The observer's lightning hooks now live in `ss_hostenv.c` and `ss_updt.c` (`EXPIRED` for a server
expiry, `NEW_TIMER` for the server's new timer, none for a client's, as the original). **One recorded-word difference to classify (not hidden):** the `weather_draw` events keep their ten
original sites, ordinals, flags and shared-seed words (`rng`), but their `w0` word is now the draw value the campaign made (the double narrowed to a float), not the field value the original
computed from it, because the campaign no longer computes the field (the host does; that is the point of the move). The driver's grammar checks the ordinals, flags and `rng` and not `w0`, so it
still accepts the record; a byte-for-byte comparison of the ten `w0` words per weather episode against the retained `2804ca4f`/`765`/`235` records will differ, and is to be handled by the
baseline-governance classification (an intentional record-semantic change), not by editing the old evidence. The field values themselves are compared separately through `export` snapshots.

## Source changes

Core C (`aphavoc/source/entity/special/session/`): new `ss_hostenv.h`/`ss_hostenv.c`; `session.c` (queries, initialisers, gust: host requests under `EECH_CAMPAIGN_HOST_ENV`, the original
bodies preserved in the `#else` for the original game's build); `ss_updt.c` (`update_transitional_weather`: camera by value, `campaign_env_step`, the setter, `create_lightning_effects`);
`ss_creat.c`, `ss_float.c`, `ss_int.c`, `ss_vec3d.c`, `ss_pack.c` (mirror pushes/refreshes). Engine plumbing: `eech-engine-sys` (`sources.txt`, `build/main.rs` defining
`EECH_CAMPAIGN_HOST_ENV`, `eech_engine.h/.c` carrying the host callback and installing it at entry, `src/lib.rs` types), `eech-engine/src/lib.rs` (the provider and its thread-local),
`eech-dc/src/module.rs` (the Lua bridge, the boot requirement and unit tests). Reference host: `eech-world/csrc/env_ref.c/.h` (kernels), `eech-world/src/env.rs` (the provider and unit tests),
`lib.rs`/`main.rs`/`build.rs` registration, every boot script and harness now passes `environment = host.env_provider ()`.

## Remaining coupling and open gates (finite)

* The lightning strike and sound entity creation, the sound/terrain-scratch bridge and the time-of-day input stay in Core (explicitly open within Task 51/D3).
* The mirror is a campaign-side copy of host state used by pack/save/player paths; it is read-only by contract but nothing in C prevents a stray write; the setters and `pack`/`unpack` are the
  reviewed sync points.
* Not shown: that the build compiles, that the 164-case oracle and the 120 s/variable-frame comparison are byte-identical, that a changed/withheld/malformed host field changes or refuses real
  campaign and projectile behaviour (the final Task 51 gates are Root's), and any DCS API (none is claimed).
* One known grammar limit inherited from the observer: a fixed-weather request directly after a non-zero rain tag is refused by the observer's grammar.
