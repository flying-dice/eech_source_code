# D3: the lightning sound effect entities and their lifetimes — ORIGINAL-behaviour probe (test-only; source prepared, UNBUILT, UNRUN, UNREVIEWED)

EEDCS-51 / D3, the physical-environment responsibility chosen next after the lightning queue. **Nothing here moves behaviour, hands anything to a host or claims any result.** It adds the smallest test-only
measurement of what the ORIGINAL code creates and how it ages it, so that a later extraction has a baseline. The sibling probe for the strike queue is [d3-lightning-queue.md](d3-lightning-queue.md); the
shape is the same (a separate finite probe store, a fixture group, a natural-run block of the retained driver, a control driver, guards). Every statement under "What the source says" is a reading of the code,
not a measurement; every "not known" stays unknown until Root has built and run this.

## What the source says (read, not measured)

* `ss_updt.c`, the hosted `create_lightning_effects` (active) and the legacy `update_lightning_effect` (`#else`) create two sound entities per lightning expiry through
  `create_client_server_sound_effect_entity` (the lightning sample, then the thunder sample), each with a pitch of `0.85 + 0.3 * frand1 ()` drawn **between** the creations: the draw order and the entity
  allocation order are part of what must be preserved. The observer already brackets each creation with `ENV_FX_SOUND_BEGIN` / `ENV_FX_SOUND_END` (the next free entity indices before and after).
* `soundeff.c`: every creator passes through `create_local_sound_effect_entity` (the real constructor). It creates the entity, links it into the parent's special-effect list and, when the effect is valid, the
  update list, and `set_local_sound_effect_sample_indices` gives it `effect_lifetime = sum(size / rate)` over its samples, clamped to 40 s (300 s for music).
* `se_updt.c`: the SERVER routine takes `get_delta_time ()` (the global `system_delta_time`) off a valid, non-looping effect each update; below zero it kills the entity when there is no playback handle or the
  handle is not playing, and leaves it alone while the handle is playing. The CLIENT routine takes the same delta, clamps the lifetime at zero and never kills; an invalid effect is unlinked from the update
  list and its handle destroyed after `valid_effect_lifetime` runs out.
* The playback handle (`sound_effect_data`) comes from `play_local_sound` (`se_snd.c`) through `create_single_system_sound_effect`, which returns NULL while `sound_system_initialised` is false. The headless
  frame never calls the play path. Whether the headless build initialises the sound system at all (it is enabled only when the command line does not say no-sound and the process is not a dedicated server) is
  **not known**; the probe prints the flags as facts.
* `application_sound_effects[]` supplies `rate`, `size` and `original_rate` per sample (`snd_data.c`; `size` from the retail `audio/sound.hdr` when present). A full retail boot may therefore load real data; do not
  assume it does not (the earlier texture-model assumption was wrong in exactly this way).
* The wind, rain and terrain ambient children that consume sound are the ambient routine already characterised (groups 11 and 12); this probe does not touch them.

## What is added (all feature-gated; with the feature off no token of any original body changes)

1. **A separate probe store, not the event stream** (`ENV_FXE`, `ss_envfx.h`; the observer's kinds, sequence numbers and transcript grammar are untouched). Eight points: `CREATED` (end of the real
   constructor), `SERVER_ENTER`, `SERVER_EXPIRED_NO_HANDLE` / `_IDLE` / `_PLAYING` (the three branches below zero), `SERVER_KILLED` (straight after `kill_client_server_entity`; only the entity slot is read, the sound
   data is gone), `CLIENT_ENTER` and `CLIENT_EXIT`. The hooks are in `soundeff.c` and `se_updt.c`, which are not under the host-env alternatives: they compile on the current hosted path. `ss_envfx.h` is now
   included unconditionally by both (the macros expand to nothing when the observer is off, as in `ss_updt.c`).
2. **`csrc/eech_env_observer.c`** records a creation only while a lightning sound creation is open (`ENV_FX_SOUND_BEGIN` to `ENV_FX_SOUND_END` of the observer's own stream) or while a fixture control has switched
   tracking on, and follows at most eight entities. It READS: the stored fields are copied **as bits** with `memcpy` (no float is loaded by value, so no x87 narrowing), the handle is a present/absent flag, the list
   links are present/absent flags, and the sample table's `rate`, `size`, `original_rate` for the first sample are read. It draws nothing, allocates nothing, formats nothing at the point and writes no game
   variable. At most 80 records are stored; each entity's first three updates are stored whole and the rest are only COUNTED (`upd` on every later record says how many ran). Records beyond the capacity are counted
   as dropped and the report says so; the driver refuses a report that dropped anything.
3. **`csrc/eech_env_fixture.c` group 14** (explicit only, observer build only). Index 0: the plan, the facts of the sound database and the sound system, two UNSUPPORTED rows and the probe's report and records. Indices 1 to 3
   are PROCESS-ISOLATED controls (an entity cannot be put back): the real constructor builds one entity with the lightning routine's arguments, tracking is switched on, and the real compiled update routine is
   called through the entity function table with `system_delta_time` set to a value the control names (the delta is restored): **1** the server routine at 0.1 s then at a delta far beyond the lifetime;
   **2** the client routine at the large delta, then at 0.1 s (clamp, no kill); **3** nothing is created: the facts and the explicit statement that the playback-handle branches are not exercised. A process that is not the
   server, or has no session, REFUSES cases 1 and 2 and says why.
4. **Drivers.** `lua/env-characterization.lua` gains an optional `sound=on` (needs `observer=on`; default off). It only APPENDS rows after the queue rows. `lua/env-sound-lifetime.lua` (new, one control per fresh process,
   `case=1..3`) arms the observer before boot, writes the fixture's rows verbatim with every value kept as text and judges **nothing physical**: it checks the report's shape and that each tracked entity's chain of points
   can occur, so any lifetime, any expiry branch and any kill outcome is equally accepted.

### Natural-run rows (`sound=on`)

Raw (scope `sound:report`): `env-snd-plan`, `env-snd-facts`, `env-snd-unsupported`, `env-snd-report`, `env-snd` (one per point; words as text). Derived by the driver from those rows, never assumed:
`sound_entity` per tracked entity (`n` 1 lightning / 2 thunder, `created_frame`, `updates_at_last_record`, `expiry` = `none` / `no_handle` / `idle_handle` / `playing_handle`, `killed`, `handle_ever`, first and last
lifetime words) and `sound_summary`. A report is refused, never truncated or repaired, if it is out of order, dropped records, has a record sequence with a gap, a chain whose points cannot follow one another, a
record of an entity other than the one tracked, update counts that go down or exceed the probe's own count, kills that the records do not show, or a number of creations that differs from the observer's own
`sound_begin` / `sound_end` events.

## Entry-point changes, stated

Source entry points changed (Root's feature-off token check must include them): `aphavoc/source/entity/special/session/ss_envfx.h`, `.../effect/soundeff/soundeff.c`, `.../effect/soundeff/se_updt.c`,
`eech-campaign/crates/eech-engine-sys/csrc/eech_env_observer.c`, `.../csrc/eech_env_fixture.c`, `eech-campaign/crates/eech-world/lua/env-characterization.lua` (one optional argument and an appended block), plus the new
`lua/env-sound-lifetime.lua` and the guards in `tests/env_sound_lifetime.rs` and `tests/host_callbacks.rs`. `eech_engine.c` (one NUL byte) is **not** touched. The raw natural reports are unchanged when `sound=off`; the
retained observer and queue reports stay comparable (the new records are in a separate store, and no event, kind or sequence number was added to the stream). A new engine build is needed.

## Commands for Root (nothing here was built or run)

`cargo fmt --check`; `cargo test -p eech-world --test env_sound_lifetime`; `cargo test -p eech-world --test host_callbacks sound` and `... environment` (each filter must report a NON-ZERO count: a filter that
selects nothing proves nothing); `cargo test -p eech-world --test env_lightning_queue`; clippy; build `eech-dc` with `env-observer`; the feature-off token check for the files above (the default-feature
`cargo check` of `eech-engine`/`eech-dc` compiled the hooks away in the WIP check, which is not that check); then **one** natural fixed-Lebanon 120 s run with exactly the arguments of the retained fixed-Lebanon
observer control plus `queue=on sound=on` (the retained controls are not rerun), compared on every non-sound row with the retained summary (a difference is classified, never normalised); then the three controls, each in its
own fresh process and root. Private coordinates and raw outputs stay outside Git.

## Honest limits

* The natural run can only report what the retained-style configuration reaches. The previously recorded fixed run holds 2 lightning events and 4 sound creations, the variable Lebanon/Georgia runs 1 event and 2
  creations; those counts do not say what the creations produced or how long the entities lived, which is what this probe is for.
* The CLIENT routine is not reachable in the headless server process; it is observed only by control 2, on one entity built by the real constructor. The playback-handle branches (a handle that is playing, or is
  not) are NOT exercised and the plan says so: they need a sound-system device, no handle is faked and no backend callback is invented. The `valid = false` unlink path is not used by the lightning routine and is
  not covered. Audible output, network delivery and the player are not exercised or claimed.
* The probe does not decide what a host should do with a sound. What stays coupled for now: the generic entity heap, ids, list links and mirrors (the constructor is the engine's own), the sound-system and device
  state, and the campaign's time inputs. Campaign policy, scheduling and consequences stay in the Core.
* Remaining whole-D3 work: this characterisation run and comparison; moving the sound responsibility to the host only after it; the remaining time inputs; the required terrain, spatial and model services; the full
  shipping 40 and all 8 complete byte identities; current checks, review and the normal merges. D4 and D5 are unstarted. Choosing sound next does not close Task 51 or narrow D3. The probe, its drivers and its guards
  are UNBUILT (apart from a WIP compile check), UNRUN and UNREVIEWED. No host API, schema or ABI is proposed.
