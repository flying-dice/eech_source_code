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

`cargo fmt --check`; `cargo test -p eech-world --test env_lightning_queue`; `cargo test -p eech-world --test host_callbacks environment` and `cargo test -p eech-world --test host_callbacks lightning_queue` (the stand-in tests, 42 in all with the new one; each filter must report a NON-ZERO count, a filter that selects nothing is not a pass; the two new queue-argument refusals, the appended-rows test, the
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

## CORRECTION batch for the two groups of the review of 7791eea3c (source only; UNBUILT, UNRUN, UNREVIEWED)

**The red this answers (Root's actual, on `3691a921d`, `environment-lightning-queue-original-3691a921d-linux-20261031-completion.json`, exit 101 at 17:10:23):** both exact new tests executed independently and failed genuine assertions — natural 9 of 9 damaged reports ACCEPTED; primitive 20 of 21 damaged reports ACCEPTED, and the legitimate complete non-empty plan
followed by the fixture's explicit queue-not-empty refusal was REJECTED while the plan was parsed; the healthy prerequisites passed. The tests are kept as they were (the damaged forms and the 32-row healthy control), and no physical expectation changed. Also reported: the first clippy run failed `useless_format` at `env_lightning_queue.rs:56` (a literal `format!`).

**Corrected (witness and observer only; no original call, arithmetic, RNG, FPU use, heap or update order, no observer sequence or grammar, no host ownership move):**
* **Time word.** At the new site `eech_env_observer.c` now `memcpy`s the bits straight from `&active->time_of_day` into the record (zero when no environment is active); the value is no longer passed through a by-value float parameter. The old `eech_envobs_bits` helper and its old callsites are outside the review and untouched. This is a source-level correction, not an observed FPU defect.
* **Natural parser (`env-characterization.lua queue=on`).** Derived from what the C records satisfy, never from what an add does: the active category is 0, 1 or 2; the time word exists exactly when an environment is active; the main environment exists in every record or none, and when it is the active one; a point that follows no add has slot -1 and zero words; an add that
  occupied a slot has it in use (`w0` not 0); the masks of points with no add between them are equal (entry and first before; first after and second before; second after and restored); a later event can only have LOST slots since the previous restore; the final queue is the last restored queue exactly when the update never ran and a subset of it
  otherwise; the occupied slot rows are the final mask exactly, in range, in use, strictly ascending; the plan has four cases and a valid category and agrees with the records on the main environment. Queued / no active environment / queue full / unexplained remain observations.
* **Primitive driver (`env-lightning-queue.lua`).** The index-0 report is parsed as the C prints it — the plan, two unsupported rows, exactly one report of its schema, then one row per occupied slot — and a NON-EMPTY plan is accepted, so the fixture's explicit refusal is reached and reported as a REFUSAL (not as a physical success). The control's head must be the queue the plan described. A refusal must
  give a reason the fixture gives and be consistent with the head (a queue-not-empty refusal needs a non-empty mask; a no-main-environment refusal needs an empty mask, no main environment and a case that needs one); a control that ran needs an empty queue (and a main environment for cases 2 to 4). Case rows are in the order the C prints them: case 2 its typed input row; case 3 two typed step rows
  (add = capacity and capacity + 1, occupancy equal to the mask's, the last equal to the final mask; a slot gained, kept or lost between the two adds is kept as observed, see the residual correction below, of this case); case 4 its typed input row, and the three tagged snapshots of the slot the add occupied exactly when `first_slot` is not -1; then one `after` row for each occupied slot, unique, ascending, in use and equal to the final mask. Update calls rise by 0, or by 2 in case 4.
* **Tests.** The two red tests, the 32-row healthy full queue and every damaged form are unchanged. The older guards that named a message of the old parser were adapted to the corrected message (still refusals): `step rows` to `step row` and `unexpected row` for the two step-row forms, and the refused-control test now describes a queue with no main environment (the fixture refuses case 2 for that reason only when there is none). The clippy
  `useless_format` is fixed mechanically (the literal `format!` is a plain string); no lint is suppressed.
* **Selectors.** The documented command `cargo test -p eech-world --test host_callbacks env_` selected ZERO tests (41 filtered) and is not a pass. Use `--test host_callbacks environment`, `--test host_callbacks lightning_queue` (42 tests in the file with the new one) and `--test env_lightning_queue`, and check that each filter reports a non-zero count.

**Next (Root):** format; the two exact red tests now expected to PASS; both strict profiles; a NEW native build from a new snapshot (the corrected C needs it; the Windows 779 native is retained but its time probe is not safe for measurement); the non-empty-plan guard; then the one natural run and the four primitive controls on fresh roots (never reused). Nothing about the original queue has been measured; the sound
entities and lifetimes, time, terrain/spatial/model, full shipping 40 / all 8 byte identity, current checks and merges stay open D3 work; D4/D5 are unstarted; no acceptance claimed.

## RESIDUAL correction after the source pass on a27f6ead2 (primitive parser, test and docs only; UNBUILT, UNRUN, UNREVIEWED)

Sole source pass (SHA-256 `17803cfb...a4a2`), no other finding: the direct `memcpy` time word is closed; the natural cohort and record coherence are fixed in source; the selectors, the literal `format!` and the positive-refusal corrections are checked. Two residual items, both in the primitive driver:
* **P1 (actual, Root's):** on `a27f6ead2` all five primitive functions FAILED (exit 101 at 17:17:28.254781): the new index-0 report schema used the field kind `:u`, which the primitive driver's kind table does not have (it has `w`, `n`, `b`, `i`, `s`), so the validator was nil (Lua line 118) before any assertion. The report schema now uses the existing unsigned-count kind `:n` consistently (`capacity`, `stored`, `dropped`, `lightning_events`, `final_occupancy`). No new kind or framework. The actual failure logs are retained
  unchanged; this was a driver defect, not a green and not a protocol red. The host filters and both strict profiles were never reached, and the a27 gated measurement process exited at the Linux assert before any native run or stage (all five roots UNUSED; no queue measurement exists).
* **P2:** the case 3 check rejected ANY slot cleared between the two add snapshots ("lost a slot an add cannot free"). That predicts the original's behaviour instead of preserving a complete observation, so it is REMOVED. Kept: the typed step rows, their case and add ordinals (capacity and capacity + 1), each occupancy equal to its mask's, the last step's mask equal to the final mask, and the after-slot cohort. A new guard
  `a_complete_case_3_observation_in_which_a_slot_is_lost_is_preserved_and_not_rejected` feeds step 32 mask 1 / occupancy 1, step 33 mask 0 / occupancy 0, final mask 0 / occupancy 0 and no after-slot rows, and requires it to be accepted with every row on the file. It is a coherent alternative outcome, not a preferred one: no physical outcome or golden is asserted. The natural parser already keeps such a loss as `unexplained`.

Everything else is unchanged: all earlier checks, the 32-row healthy full queue, every damaged form, the legitimate non-empty-plan refusal guard. No C, Rust production, natural Lua or build input changed in this batch, so Root can reuse the unchanged a27 Windows native (build exit 0 at 17:18:32.150768, 53 payloads) with an explicit NEW primitive Lua override after proving the C/Rust/build and other Lua equivalence. Next (Root): format, pin, the
new filters and the non-empty guard on Linux, both strict profiles, the one natural fixed-Lebanon 120 s run and the four fresh primitive processes (each a fresh root), the complete comparison with the old non-queue raw rows, and the same review of the new delta and the actuals. Queue and sound behaviour, lifetimes, time, terrain/spatial/model, full shipping 40 / all 8 byte identity, current checks and merges stay open D3 work; D4/D5 unstarted.
`cargo test -p eech-world --test host_callbacks the_natural_queue_report_rejects_incomplete_slot_cohorts -- --exact --nocapture` must also run: its name contains neither `environment` nor `lightning_queue`; require one executed test. The two broader filters retain the older environment/queue guards.
