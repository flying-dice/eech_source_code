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
round-toward-zero as the engine does. Nothing was re-formulated.

**The caller's complete floating-point state (corrected after the review of `05b8caf82`, finding ENV-PHYSICAL-1; its extent and its proof were corrected again for `2afd1b4c1`: see "Actual results at `2afd1b4c1`" below -- the restore is now the LAST floating-point act, the read-back compares the complete words, and the matrix seeds a complete state).** `fegetenv`/`fesetround`/`fesetenv` alone are not enough: on the shipping
toolchain (MinGW-w64 12 posix) `fesetenv` restores the x87 environment but leaves the SSE rounding control and the rest of MXCSR where the kernel put it (the retained actual control
`reference/d3-model-characterization/model-mingw-fenv-control-20261030.json`: MXCSR `0x1fb4`/`0x3fb4`/`0x5fb4` come back as `0x7fb4`). Every public entry of `env_ref.c` therefore saves,
before its first statement, the x87 environment (`fnstenv`: control, status and tag words and the exception pointers, reloaded straight away so the caller's masks stand) and the whole of
MXCSR (`stmxcsr`: rounding control, masks and sticky flags); sets round-toward-zero in both units and reads it back; and on **every** exit, refusals included, restores both (`fldenv`,
`ldmxcsr`) and reads the restored control and status back. A restore that does not read back as saved is reported as an environment failure and withholds the answer and the commit. The
guard is the established one of the reference host's closed components (`model_step.c` of the model component `80fd62ea4`/`e48808f53`: save, set, read back, restore with a read-back;
`terrain_ref.c` `host_fp_state` of the ground component `aafb21775`: the complete `fnstenv` image), copied for this component only. It is a guard of this component's public entries; it is
**not** a claim that the campaign DLL's own floating-point handling is fixed. The Rust test `the_callers_complete_floating_point_state_survives_every_operation_in_every_mode` reads and
writes the real x87 control/status and MXCSR registers (all four rounding modes, sticky flags clear and set, accepted and refused operations) and also requires the kernels' answers to be the
same bits in every caller mode; **it is source only: Root runs it on the actual Windows target, and the unchanged MinGW control is not rerun.**

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
creates the strikes and sound entities in the original order. A client's expiry needs no draw (the server sets the next timer).

**The protocol is enforced against the request the campaign built, before the draw (finding ENV-PHYSICAL-2).** `need_draw` is legal only as a **server's first question** (no draw yet): a
client's `need_draw`, a `need_draw` after the draw, a second one, or a server's expiry reported without the negotiation is a refusal **before** `frand1` is called and before the mirror is
written. A completed step must say `expired` explicitly as a boolean (a missing field is not `false`); after the draw the final answer must complete and still report the expiry. These
checks are made at three layers, each against the values of the request and never the provider's copy of the table: the Lua bridge (`module.rs`) captures `op`, `want_vector`, `server` and
the presence of `lightning_draw` **before** it calls the provider, so a provider that edits the request it is handed (clears `want_vector`, deletes `lightning_draw`, claims to be a server)
gains nothing; the engine's callback (`eech-engine` `check_env_reply`) checks every reply against the request it answers, so a **direct Rust `EnvProvider`** cannot bypass the Lua bridge; and
the C side (`ss_hostenv.c`) repeats the lightning protocol before drawing. A wind vector the campaign asked for is required (never synthesised as zeros).

### Refusals and markers (atomic; nothing falls back to another solver)

A missing provider is refused at boot (`[env-provider:missing]`, after the terrain's check) and at the engine's first question. The first line of the error the engine stops with carries
`[env-provider:<marker>]`:

* `<op>.callback` the provider raised an error (its text follows);
* `<op>.answer` the answer is not a table, or a number is missing, not finite, or a mode/flag is out of its domain; a `need_draw` re-ask; a server expiry without the draw;
* `<op>.state` the field in the answer is missing a number, not finite, a mode outside 0 to 4, or a flag that is not a boolean.

The campaign's mirror is written only after an answer has been checked in full. The reference host's kernels work on a copy of the field and commit it only when the operation has finished,
so a refused operation changes nothing on either side. The kernels refuse only what is not a field or what the original itself refuses or cannot complete (finding ENV-PHYSICAL-3):
a gust phase outside `[0, 1)` **in a query** (the original asserts there), an imported field that is not finite or has a mode or flag out of range, a draw the original's draw macro
could not give, a weather-zone update with no camera, a **result** that is not a finite field (checked before anything is committed), and a gust phase that the update would make infinite
(the original's `while (gust >= 1.0) gust -= 1.0` can never leave infinity; every finite phase leaves it, under round-toward-zero each pass lowers it). They do **not** refuse up front a
negative or not-finite delta, an unused camera component (the original reads only x and z, and only in a local model with a zone), the map extents it does not use, or a gust phase
outside `[0, 1)` that the update wraps (a phase set to -0.25 with a delta of 5 reaches 0.25, as in the original). The old pre-update cutoff of `[0, 1e6)` is removed; the wrap is the
original's loop, with no clamp and no remainder, so a very large finite phase takes proportionally many passes, as in the original.

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

## Validation inherited at `05b8caf82` (reported by Root; not re-run here) and the review batch that followed

Reported at `05b8caf82`: the Windows environment-fixture build exited 0 and all 46 payloads verified; three real native 164-case probes (Georgia, Lebanon, a fresh Lebanon repeat) each
exited 0 at native and controller with all six pre/post inputs 0 and 705 strict rows, each preserving all 704 original raw lines **including line endings** exactly plus one explicit
validation-metadata row; both complete Lebanon raw files are byte-identical (SHA-256 `a25f409d468403911b31c7e5827b21264d2b77886016b4794b5655b13461d4a7`; comparison
`environment-physical-original-comparison-05b8caf82-20261031.json`). That is a diagnostic normal-domain conformance, **not** floating-point, authority or shipping acceptance. The original
oracles stay closed and are reused, never rebuilt. Linux default and `env-fixture` tests for the affected packages passed their functions (asset-dependent missing inputs are not retail
proof); strict Clippy first exited 101 at `eech-world/src/env.rs:359` (`needless_question_mark`), now fixed without a suppression; feature Clippy was not reached.

**Actual hosted failures at `05b8caf82`** (GitHub run 41192020532, log `environment-physical-05b8caf82-hosted-failures-20261031.log`; reported, not re-run here): (1) Windows-cross under
Wine, `the_caller_keeps_its_floating_point_environment` at `env.rs:651` expected the nearest quotient of 1/5 and got the round-toward-zero one (245 passed, 1 failed) -- the guard bug
(ENV-PHYSICAL-1) together with a test that assumed the ambient mode in a process that had initialised the engine; (2) `i686-unknown-linux-gnu`, with and without
`EECH_ORIGINAL_STACK_ATTRIBUTES=1`, `the_gust_modulation_is_the_originals_piecewise_table` at `env.rs:608` expected `gust(1.0 / 7.0) == 0.25` and got `0.24999999` (245 passed, 1 failed each);
(3) x86_64 Linux strict Clippy, the `needless_question_mark` above. Bounded test-context changes, none to production arithmetic, flags or evaluation order, no tolerance or baseline: the
superseded quotient test is replaced by the full x87/MXCSR matrix, which arranges each caller mode and status itself and restores the thread's registers when it ends (it assumes no
ambient mode); and the gust test no longer asserts an exact value at 1/7, which is not representable -- the float nearest 1/7 times 7 is 1.0000000447, which the original's x87 build keeps
in an extended register (so `ceil` is 2 and the result is 0.24999999) and an SSE build rounds to 1.0 (0.25); both are the original arithmetic, not a divergence. It now asserts exact bits at
the dyadic phases 1/8 (0.21875, `0x3e600000`) and 1/16 (0.109375, `0x3de00000`), where the product, segment, fraction and result are exact in any precision and either unit. The original
164-case fixture is untouched.

The ONE combined independent source review of `05b8caf82` (`independent-environment-physical-source-review-05b8caf82-20261031.json`, SHA-256 `9c9e85b8...a49e86`) returned
CHANGES_REQUIRED with three findings, closed in source by this batch (UNBUILT, UNRUN, UNREVIEWED): ENV-PHYSICAL-1 (complete x87/MXCSR guard, above), ENV-PHYSICAL-2 (request contract and
lightning protocol, above), ENV-PHYSICAL-3 (branch-specific reads and the gust wrap, above). The `weather_draw.w0` change (the draw value, not the computed field) is a documented diagnostic
change of a private feature; the old raw records are immutable and are not altered; it is not claimed byte-unchanged for that record.

## Actual results at `2afd1b4c1` and the second correction batch (UNBUILT, UNRUN, UNREVIEWED)

Reported by Root (not re-run here). **Successes:** the Windows environment-fixture build exited 0; three real retail 164-case probes (Georgia, Lebanon, fresh Lebanon repeat) each exited 0
at native and controller with six pre/post inputs 0 and 46 payloads; the complete 705-line outputs, line endings included, are exact against `05b8caf82`, so all 704 original raw lines
remain exact plus the one validation-metadata row (Georgia SHA-256 `5165602d...ca56f`; both Lebanon `a25f409d...5d4a7`). Cross-compiled Windows export of three packages 0, then on real
Windows: engine 76/76 exit 0, dc 21/21 exit 0. **Failures:** Linux default tests exit 101 (world 247 passed, 2 failed at `env.rs:891` and `:761`; the fixture and strict phases were not
reached), and the same two on real Windows (world 247 passed, 2 failed, exit 101). Author exit 0, a build exit 0 and numeric exactness imply no floating-point, authority or shipping
acceptance. The independent review of `2afd1b4c1` (`independent-environment-physical-correction-review-2afd1b4c1-20261031.json`, SHA-256 `70bdcdd0...ba20`) returned CHANGES_REQUIRED
with four findings, closed in source by this batch:

* **ENV-CORRECTION-1 (P1), the extent of the guard.** In 2afd the guard restored the caller's state and only then compared (`ts < 0.5`) and stored float results (weather, wind, gust,
  the float initialisers and the output clearing of the step); on a supported x87 target those can change condition/status bits even when the value is exact. Now **all** floating-point
  work -- the original's arithmetic, the simple-mode comparison, the staging of every result, the zero-initialisation of every float -- happens between `fp_save` and `fp_leave`; a result
  is staged in a local object that `ENV_FP_SETTLE` makes a stored object before the restore, and after the restore only integer tests and bitwise copies (`memcpy`) remain, the field
  commit included. No float is initialised before `fp_save`. The read-back of the restore compares the COMPLETE x87 control, status and tag words and the whole of MXCSR (the mask of
  2afd, which left the stack-fault and summary bits and the exception flags out of one comparison, is gone). The numeric expressions, types and evaluation order are unchanged. The Rust
  matrix now seeds a complete caller state itself -- every pair of the four x87 and four SSE rounding controls (so the mixed pairs), MXCSR sticky flags clear and set, and three
  deliberately arranged x87 status words (clear; C3 with every exception flag; stack top 4 with C0, C1 and C2) with an all-empty tag word -- compares the complete words, no mask, after
  each call, restores the test thread's complete environment even when an assertion fails, and covers `init_random` (accepted and refused), the first `NeedDraw` answer and the final `Done`
  after the draw, the simple-mode weather answer, the wind with and without a vector, and every kind of refusal. It assumes no ambient mode. Limit, stated: the Rust code around each
  call is the test's own; on an x87-only target it can itself touch the status word, and a difference reported there must be attributed to the guard or to the test's glue. A leak here is
  source-inferred, not a measured failure, until Root's actual execution; the unchanged MinGW fenv control is closed evidence and is not repeated.
* **ENV-CORRECTION-2 (P2), the unrequested wind vector in C.** `campaign_env_wind_at` required a finite vector even when none was requested, while the Rust boundary (which does not read
  it) accepted such a reply and the Lua bridge hid the difference by synthesising zeros. Now C requires the speed always and the vector only when the campaign passed a vector; a requested
  vector remains mandatory. A real-C-consumer test (`eech-engine` tests, `the_c_wind_consumer_reads_a_requested_vector_and_does_not_read_one_nobody_asked_for`) calls the real
  `campaign_env_wind_at` with a C host installed and shows: a requested valid vector arrives, an unrequested vector is not read even when it is not finite, and the request carries the
  right `want_vector`. No boot, map or asset is needed. The requested-and-malformed half ends in `debug_fatal`, which needs an engine entry to unwind to, so it is not a unit test; it is
  the engine-level mode `malformed_wind_vector` of the wind-authority witness (a later, separately authorised test), and until that runs it is source-read only. Through the Lua path the C
  consumer can never see an unrequested bad vector (the bridge normalises it), so this unit is the only real witness of that half.
* **ENV-CORRECTION-3 (P2), the early `NeedDraw` success.** With the blanket delta check gone, an infinite delta made a server's timer -infinity and the kernel returned `need_draw` with
  it before the final check of the field. The timer is now validated at that return and refused without a commit (`an_infinite_delta_that_would_ask_for_a_draw_is_...` keeps the
  refusal assertion, and the matrix keeps its infinite-delta refusal). No blanket input check was restored. The Lua bridge and the engine boundary already refused a non-finite timer
  before the shared random word was drawn from, so no shipping consumption of a draw for this value is claimed.
* **ENV-CORRECTION-4 (P2), the radius-zero test sequence.** The first step of a calm zero-radius zone expands the radius to the original's minimum (`0 - 100` is below it), after which a
  camera IS needed, so the second, camera-less step legitimately refused. The test now asserts that evolved refusal (and that it changes nothing), then re-imports the calm state before the
  independent absent-camera step. The radius arithmetic and the required-camera error are unchanged.

## Actual results at `d4e5c34d2` and the confirmed C-ABI prologue cause (UNBUILT, UNRUN, UNREVIEWED correction)

Reported by Root (not re-run here). **Successes:** Windows fixture build 0; real Windows engine 77, dc 21, world 250, all exit 0 (the full matrix and the C consumer test included); Linux default and
fixture tests and both strict Clippy phases exit 0 (completion 11:04:48), formatting 0; three retail 164-case probes each native/controller 0 with six pre/post inputs 0 and 46 payloads, the
complete 705-line outputs exact against `05b8caf82` (all 704 original raw lines preserved); two 120 s configuration cases (control 10850 lines, probe 11514) exit 0 with every original
mode-specific raw line and ending exact except the one added validation-metadata line, the common retail snapshot and initial/final campaign raw lines (10829 per mode) exact, the immediate
probe retail/campaign fingerprints equal, and 1200 frames / elapsed 119.99835968017578 exact (`Run.complete` there is probe COVERAGE, control false / probe true, not campaign completion; an
earlier selector that counted the probe-only rows and the run metadata failed and is retained as v1, the corrected v2 declares the mode flags; comparison
`environment-physical-120-original-comparison-d4e5c34d2-v2-20261031.json`). **Failure (hosted only):** `i686-unknown-linux-gnu`, normal and `EECH_ORIGINAL_STACK_ATTRIBUTES=1`, GitHub run
41194708801: the matrix assertion at `env.rs:776` after the first `NeedDraw` step, seed x87 rounding 0 / SSE rounding 0 / sticky 0 / status `0x2700`: the caller's x87 status reads `0x2500`
(C1 cleared; control, tag and MXCSR equal; world 249 passed, 1 failed). Hosted x86_64, cross-built Windows and actual native Windows pass. The independent source review of `d4e5c34d2` closed
the four earlier findings with no new source finding (SHA-256 `4ffc19d7...6d506`); this runtime bug is new and needs its own new-delta review.

**Cause, confirmed by measurement (Root's private isolation probe, `environment-i686-c-abi-fp-isolation-d4-20261031/`).** The probe links the unmodified frozen kernel and seeds the complete x87
control/status/tag words and MXCSR; on 32-bit it exits 1 (24 cases; every run A, a do-nothing call with the same prototype, exact; 16 real-kernel runs differ in the x87 status; 0 setup failures),
on 64-bit it exits 0. The 32-bit disassembly shows the cause precisely: `env_ref_step` took the lightning draw as a **by-value `double`**, and the 32-bit callee homes that parameter in its
prologue -- `+0x34f5 fld qword [esp+0x1ac]`, `+0x34fc fstp qword [esp+0x30]` -- and calls `fp_save` only at `+0x357e`. The x87 load clears C1 **before** the guard captures the caller's state, so
the guard saves and faithfully restores an already-changed status. The output and field copies after the restore are integer `MOV`/`REP MOVS` and are not the cause; the Rust glue is not the cause.

**Correction (finite).** `env_ref_step` now takes `const double *draw` (`env_ref.h`, `env_ref.c`) and the wrapper assigns an uninitialised local `double draw_value` **only after
`fp_save`/`fp_enter`** (a NULL address with `have_draw` set is a refusal); `step_impl` and its arithmetic are unchanged. Rust (`env.rs`) declares the pointer and `Field::step` builds the draw's
bits before the call with an integer move (`f64::to_bits`) and passes the address of that `u64` as `*const f64`: the same bits, signed zero and every value preserved. This is the private
reference-host backend's ABI only; the campaign request/reply structs, the Lua numeric wire and the Core draw sites are untouched. The other entries (weather, wind, gust, which take float
parameters by value) are not changed: their matrix operations passed this same seed. The matrix (96 conditions, every C1 assertion, i686 included) is unchanged and unmasked. A private probe v2
(`environment-i686-c-abi-fp-probe-v2-20261031.c`, SHA-256 `8b0a5a56...38092`, with build notes) differs from the first only in this ABI, so Root can re-run the same matrix on the corrected
kernel and retain the assembly: no x87 instruction may precede `fp_save` in `env_ref_step`. No generic DLL floating-point claim; source only until Root's execution.

## Remaining coupling and open gates (finite)

* The lightning strike and sound entity creation, the sound/terrain-scratch bridge and the time-of-day input stay in Core (explicitly open within Task 51/D3).
* The mirror is a campaign-side copy of host state used by pack/save/player paths; it is read-only by contract but nothing in C prevents a stray write; the setters and `pack`/`unpack` are the
  reviewed sync points.
* Not shown: that the build compiles, that the 164-case oracle and the 120 s/variable-frame comparison are byte-identical, that a changed/withheld/malformed host field changes or refuses real
  campaign and projectile behaviour (the final Task 51 gates are Root's), and any DCS API (none is claimed).
* One known grammar limit inherited from the observer: a fixed-weather request directly after a non-zero rain tag is refused by the observer's grammar.
