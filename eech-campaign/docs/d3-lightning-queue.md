# D3: the lightning strike queue — ORIGINAL-behaviour probe (test-only; source prepared, UNBUILT, UNRUN, UNREVIEWED)

EEDCS-51 / D3, the physical-environment responsibility chosen to be characterised next. **Nothing here moves behaviour, hands anything to a host or claims any result.** It adds the smallest test-only
measurement of the ORIGINAL queue that `add_3d_lightning_strike` writes, so that a later extraction has a baseline. The compact plan that suggested this first choice
(`physical-effects-compact-plan-opus55-medium-20261031.md`) is source advice, not measured fact; its "unresolved facts" are exactly what this probe is for and every prediction in it stays unverified until Root
has built and run it.

## What the source says (read, not measured)

* `ss_updt.c`, the hosted `create_lightning_effects` (active) and the legacy `update_lightning_effect` (`#else`), both with the same original queue calls in the same order: when the timer is due it reads `get_3d_active_environment ()`; if that is `main_3d_env` it uses NULL for "nothing to
  restore", otherwise it calls `set_3d_active_environment (main_3d_env)` (which `ASSERT`s a non-NULL environment and realises fog, clip, light, viewpoint and shadow). It then calls `add_3d_lightning_strike`
  twice (CLOUD_BURST, FORKED_1; 0.3 s; the weather centre), restores the previous environment **only if the saved pointer is non-NULL**, and then creates the two sound entities.
* `3dstrike.c`: `add_3d_lightning_strike` does nothing unless `active_3d_environment` is non-NULL; otherwise it takes the FIRST free of 32 slots and fills it (position, y = 6000, type, frame 0, `time_started`
  from the environment's time of day, `time_length`); with no free slot it silently does nothing. `update_3d_lightning_strikes (env)` ages and frees strikes; its callers are in `3d_draw.c`
  (`update_application_3d_scene`, behind `application_3d_rain_active`), and the headless frame (`eech_engine.c frame`) calls none of the draw functions.
* Consequence for a "forced NULL" control: the real routine switches to `main_3d_env` before it adds, so forcing the active environment to NULL before the routine does **not** exercise a NULL add — and the
  existing `ENV_FX_LIGHTNING_ENV` flag (`current_3d_env == NULL`) is the same value both when the active environment already was the main one and when it was NULL and was switched to (and then not restored).
  The probe therefore reads the active environment BEFORE the routine looks at it.
* The strike position and expiry decision are already HOST-owned (`campaign_env_step`); nothing here moves them back.

These are readings of the code. Whether the main environment exists headless, which value the active one has when the first strike is due, whether the queue is ever written, filled or aged in the frames, and
what the primitives do with a NULL environment or a full queue are **not known** until the probe runs.

## What is added (all feature-gated; with the feature off no token of any original body changes)

1. **A separate probe store, not the event stream.** `ss_envfx.h` gains `ENV_FXQ (point)` and six points; the observer's event kinds, sequence numbers and the driver's transcript grammar are untouched and the
   probe adds no event to it (so the retained observer streams stay comparable). Points, per lightning event, in the original order: `ENTRY` (before the active environment is read), `BEFORE_1`, `AFTER_1`,
   `BEFORE_2`, `AFTER_2` (around each `add_3d_lightning_strike`), `RESTORED` (after the original's own restore, before the sound entities). The same six points are in BOTH compiled entry points, which compile exclusively so nothing is captured twice: the hosted `create_lightning_effects`, which is the ACTIVE path of the current baseline (`EECH_CAMPAIGN_HOST_ENV` is enabled; `campaign_env_step` consumes the host's expiry and position and then calls it, and it still executes the ORIGINAL queue calls and sound creations), and the legacy `update_lightning_effect` of the `#else` build, kept for that build's own use. The first version of this prep hooked only the legacy branch; Root identified that as the wrong (inactive) branch BEFORE any build, so it is not a measured runtime failure.
   What is characterised is the ORIGINAL QUEUE BEHAVIOUR ON THE CURRENT HOSTED FIELD/AMBIENT BASELINE, not a whole-original weather DLL; host-env is not turned off and no accepted field or ambient move is undone.
2. **`csrc/eech_env_observer.c`** stores at most 96 records (16 events) and counts what it cannot store. A record holds: the active environment (absent / the main one / another), whether the main one exists,
   the active one's time of day as a bit pattern, the queue's 32-bit occupancy mask and count and, after an add, the slot that became occupied with its eight words. It only READS: no draw, no allocation,
   no formatting, no write to a game variable, no floating-point operation on a value the original computed.
3. **`modules/3d/3dstrike.c`** gains, in the observer build only, a call counter of `update_3d_lightning_strikes` and a read-only accessor of one slot (`in_use`, `type`, `x`, `y`, `z`, `time_started`,
   `time_length` as stored / IEEE bit patterns, `current_frame_number`). The structure stays private to the file. **Root's feature-off token check must now include this file** (and `ss_updt.c`, `ss_envfx.h`).
4. **`csrc/eech_env_fixture.c` group 13** (explicit only; never in the default run, the census or the witness set). Index 0: the plan and the natural report (the probe's records and the queue as it stands). Indices 1
   to 4: four PRIMITIVE controls of `add_3d_lightning_strike` / `update_3d_lightning_strikes` themselves, each **process-isolated** (the queue cannot be put back, so a control refuses a queue that has been
   written and says `changed=1 queue_restored=0`): 1 an add with the active environment set to NULL by plain assignment; 2 an add with the main environment active; 3 one add more than the queue has slots for;
   4 a CLOUD_BURST aged by `update_3d_lightning_strikes` at two later times of day. They are NOT the lightning routine, NOT a missing-host proof, and they do not exercise the routine's switch (the natural
   run does). Unreachable or unsafe prerequisites stay explicit: no main environment or a non-empty queue is a printed refusal, nothing is forced; the forked types (they read `texture_animations`, which the
   headless build may not initialise) are not exercised and the plan says so; drawing and the application scene update are not exercised.
5. **Drivers.** `lua/env-characterization.lua` gains an optional `queue=on` (needs `observer=on`; default off). It **only appends** rows after the observer summary; with `queue=off` it is exactly as it was, and with
   it on every earlier row is unchanged (the new test compares the rows). `lua/env-lightning-queue.lua` (new, one control per fresh process, `case=1..4`) writes the fixture's rows verbatim with every value kept as
   text and judges **nothing physical** — it checks the report's shape only, so a strike that was queued and one that was not are equally accepted.

### Natural-run rows (`queue=on`)

Raw (scope `queue:report`): `env-lightning-plan`, `env-lightning-unsupported`, `env-lq-report` (armed, stored, dropped, `lightning_events`, `update_calls`, `queue_capacity`, final mask), `env-lq` (one per point), `env-lq-slot`
(every slot occupied now). Derived by the driver from those words, never assumed: `lightning_queue_outcome` per add — `queued` (one slot became occupied; its slot and words), `no_active_environment` (nothing changed and
the active environment was absent), `queue_full` (nothing changed with every slot in use), `unexplained` (nothing the records explain); `lightning_queue_environment` per event (entry / first-add / restored active
environment, `switched_to_main`, `restored_to_entry`); `lightning_queue_summary`. A report is refused, never truncated or repaired, if it is incomplete, dropped records, is out of order, has an occupancy that does not
agree with its mask, names a slot the masks do not show, or holds a different number of lightning events than the observer's own stream.

## Entry-point changes, stated

* `env-characterization.lua`: one new optional argument and an appended block; nothing else. The retained driver-235 outputs are compared with this driver's `queue=off`/`on` outputs minus the queue rows.
* A new engine build is needed (new C in `ss_updt.c`, `ss_envfx.h`, `3dstrike.c`, `eech_env_observer.c`, `eech_env_fixture.c`); the b7 "Lua-only reuse of the d559 native" no longer applies. The payload set must be derived
  from the new snapshot, not copied from the earlier 51.

## Commands for Root (nothing here was built or run)

`cargo fmt --check`; `cargo test -p eech-world --test env_lightning_queue`; `cargo test -p eech-world --test host_callbacks env_` (the stand-in tests; the two new queue-argument refusals, the appended-rows test, the
two coherent alternative outcomes and the fifteen damaged reports); clippy; build `eech-dc` with `env-observer`; the feature-off token check for the files above; then **one** natural fixed-Lebanon run with
exactly the arguments of the retained fixed-Lebanon observer control plus `queue=on` (the retained controls are not rerun), compared on every non-queue row with the retained summary; then, if wanted, the four
primitive controls, each in its own fresh process. Private coordinates and raw outputs stay outside Git.

## Honest limits

* The natural run can only report what the four retained-style configurations reach; it will not by itself show a full queue, a NULL add or aging if the frames never produce them — that is what the primitive controls
  are for, and they are controls of the primitives, not of the routine.
* The probe does not observe the sound entities, their lifetimes, any speaker, network delivery, the player, the renderer, Tacview or the shipping module. Those, the remaining time inputs, terrain, spatial/object and
  model services, full shipping 40 / all 8 byte identity and the merges stay open D3 work (D4/D5 unstarted). Choosing this queue first does not close Task 51 or narrow D3.
* Both compiled branches are hooked; this is the original queue on the current hosted field/ambient baseline, not a whole-original DLL. The probe, its drivers and its guards are still UNBUILT, UNRUN, UNREVIEWED. No host API, schema or ABI is proposed. Whatever interface the later extraction needs follows what the probe finds.

## Independent source review of 7791eea3c and the RED PREP for its queue-report completeness group (test-only; UNBUILT, UNRUN)

Review `ENV-LIGHTNING-QUEUE-ORIGINAL-SOURCE-779` (SHA-256 `6caf4597...fde3d`), no physical-outcome oracle, two P2 groups:
* **Time word (not fixed here):** `eech_envobs_bits (active->time_of_day)` passes a float by value before the helper's `memcpy`; the requested change is a direct `memcpy` from `&active->time_of_day` into the record's `uint32_t` (zero when no environment is active), without touching the old helper or callsites. A source reading, not a measured floating-point failure.
* **Queue-report completeness:** the natural parser accepts duplicate occupied-slot rows with a real slot missing, an added slot whose `w0` is 0, and point records that do not adjoin (entry/first add, first add/second add, second add/restore) or reconcile with the final queue; the primitive parser never checks its slot cohort, typed
  input rows, step rows or index-0 report, and a legitimate non-empty plan (`env-lq-slot` rows are printed whenever the queue is occupied) is rejected as malformed before the fixture's explicit queue-not-empty refusal. These are shape and state-view checks; none predicts whether an add queues, is refused for want of an environment, finds the queue full or ages.
* **Coverage correction owed:** the selector this document recommended for the stand-in tests (`cargo test -p eech-world --test host_callbacks env_`) selects ZERO tests (41 filtered) — not a pass. The fix batch replaces it with selectors that name tests that run; `env_lightning_queue` ran 4 functions.

**RED PREP (this step):** `the_natural_queue_report_rejects_incomplete_slot_cohorts` (`tests/host_callbacks.rs`, nine damaged natural reports; the healthy report first) and `the_primitive_queue_report_requires_complete_input_and_slot_cohorts` (`tests/env_lightning_queue.rs`, 21 damaged primitive reports, the repaired 32-row healthy full queue, and the
legitimate non-empty plan with the fixture's own refusal as a distinctly reported positive outcome). Every parser, driver, C body and physical expectation is unchanged, so on the current source both are expected to FAIL naming what they still accept (an expectation until Root runs the two exact names). Then one finite fix
batch. Nothing here is a runtime physics result; no retail queue characterization has been launched or accepted; the unsafe-time Windows native is not used for measurement.
