# D2: characterisation of the original physical weapon-aiming and readiness code (EEDCS-44)

**Status of this document: characterisation of the ORIGINAL code, unchanged by the move that followed it; the extraction is in [`m5-aim-extraction.md`](m5-aim-extraction.md). When this document was written: characterisation only. No aiming responsibility had moved; no host boundary, no extraction, no shipping-build change, no full pack.** Task EEDCS-44 (under EEDCS-18), branch `codex/d2-weapon-aiming`,
from accepted integration `3ce17432e4026f85b19ebf06cc582314817ddac2`, composed with the reviewed sensing branch `e7e46dcc2` (runtime `a2a041fec`; development stacking, EEDCS-43 is not accepted and its delivery precedes this task). The
characterisation baseline is the clean `2608f8022e8a5ce7c19da635af128e34803a4def`. Scope authority is the live M5 goal (Confluence 11534337) and the migration ledger (14548994): physical aiming and weapon execution are the host's;
target and weapon choice, priorities, engagement, awareness and consequences stay in EECH_DC. Artifacts: [`reference/d2-aim-characterization/`](../reference/d2-aim-characterization/README.md).

This document records what the original code does, measured through the real host Lua -> public native module path during game stepping, what it was **not** shown to do, and the coupling a later physical cut must face. It does not choose the cut.

## Active call paths (verified in source, then measured)

| Entry | Called by | Notes |
|---|---|---|
| `update_entity_weapon_systems` (`weapon.c`) | all six family updaters, each twice (`hc_updt`, `fw_updt`, `rv_updt`, `sh_updt`, `aa_updt`, `ps_updt`) | readiness, device slew, shared-device arbitration; "ORDER IS CRITICAL" after the target scan and the ready update |
| `update_entity_weapon_system_weapon_and_target_vectors` | the same updaters, directly after it; also `co_wpn.c` (player) | writes the launcher's weapon vector, weapon-to-target and weapon-to-intercept vectors, their range and `WEAPON_AND_TARGET_VECTORS_VALID` |
| `get_pitch_device_to_target_vector` (static) | the vector update; `get_local_entity_selected_weapon_to_target_offsets` (HUD, player only) | dispatches on the weapon's `aiming_type` |
| `get_intercept_point` (static) | the guided branch of the above; `get_target_intercept_point` (HUD, player only) | up to three lead iterations |
| `get_lead_and_ballistic_intercept_point_and_angle_of_projection` | the unguided branches of the above; **`wn_creat.c` at weapon launch** (decoy/fuze timer) | 3 or 5 iterations against a moving target; draws from the shared random stream |
| `get_ballistic_pitch_deflection` (`wn_move.c`) | the lead solver; `gd_attck.c` (AI salvo aim, fixed pitch); `co_wpn.c` (player); the player cannon point-lock in the weapon-system update | table interpolation; the tables are generated at initialisation by simulating each weapon |
| `aircraft_fire_weapon`, `update_aircraft_weapon_fire` (`ac_wpn.c`), `update_vehicle_weapon_fire` (`vh_wpn.c`) | the guide attack code and the updaters | the fire decision: ready, vectors valid, measured launch-angle error against `max_launch_angle_error` |
| `update_person_animation` (`ps_anim.c`) | `ps_updt.c` | turns a person to face its target and calls `campaign_movement_place` (D1's host-owned placement) |

`aa_wpn.c`, `rv_wpn.c` and `sh_wpn.c` contain further launch-angle code but are `#ifdef 0` and are not in `sources.txt`: dead, not characterised.

## Physical inputs and results, separated from campaign decisions (from source and runtime behaviour, not a proposed interface)

| | Physical (candidate for the host's aiming responsibility) | Campaign (stays in EECH_DC) |
|---|---|---|
| Inputs | the pitch device's world viewpoint (entity position and attitude through the model's heading/pitch device tree, with the device's current angles); the target point (the model's `TARGET_POINT` sub-object, or the position, plus the shape's offset table for fixed objects); the target's motion vector and speed; the launcher's speed; the weapon's database figures (aiming type, cruise velocity, `acquire_parent_forward_velocity`, the ballistic tables); the map-area check; the elapsed frame time | the target link (`LIST_TYPE_TARGET` parent), the selected weapon, ammunition counts and damage (which package's devices are used), `make_weapon_system_ready` configuration, the player flag |
| Results | `weapon_vector`, `weapon_to_target_vector`, `weapon_to_intercept_point_vector`, the intercept range, the `VECTORS_VALID` flag; the package devices' `weapon_system_heading/pitch` (and their model mirror, the sub-objects' `relative_heading/pitch`); `SELECTED_WEAPON_SYSTEM_READY` (and the aircraft ready animation state); the person's facing attitude | whether and when to fire (`ready`, `valid`, the launch-angle threshold and its accuracy multiplier, burst/salvo timers, range and altitude checks, LOS), reload, target and weapon selection |
| Measured, then consumed | the launch-angle error `acos(dot(weapon_vector, intercept_vector))` | the comparison with `max_launch_angle_error` (times 0.25, 0.5 or 1.0 by guidance and accuracy mode) |

`get_viewpoint_from_weapon` (used by the slew) **writes** the 3D instance's sub-objects (resets them to zero, reads the viewpoint, restores them), so the device angles live in two places: the package status (physical state) and the model tree (a mirror the solver also reads).

## Method

A gated observer (`mb_aimfx.h` hooks in `weapon.c`, `wn_move.c`, `ac_wpn.c`, `vh_wpn.c` and `ps_anim.c`; `csrc/eech_aim_fixture.c`; entries in `eech_engine.c`; cargo feature `aim-fixture`; wrapper `tools/build-windows-aim-fixture.sh`; driver
`lua/aim-characterization.lua`) records, in two hashes, what the campaign's own update loop makes the aiming code compute:

* **physical hash**: every pitch-device solve (the target and intercept vectors and range it produced), every guided intercept prediction (target point, speeds, predicted point), every lead-and-ballistic solve (its branch, iterations, the range before and after the original's
  random jitter, the intercept point, projection angle, time of flight, **the number of draws it made from the shared random stream**), every ballistic-table lookup, every device slew step (required and current offsets, the rate-limited step, shared-device flags), the vectors the update leaves
  on the launcher, the final device angles, the readiness state machine (each `update_aircraft/vehicle_weapon_system_ready` step with the state before and after, and each open/close command), the measured launch-angle error at each fire decision, and a person's facing heading;
* **decision hash**: how far the vector update got (the gate reached), whether the vectors were valid, the ready verdict, the early exits and every fire-decision rejection (not ready, vectors invalid, launch angle).

Three solver entries are **wrapped** (the original body is renamed behind a feature-only macro; a wrapper of the original name calls it and reports its arguments and outputs) and the rest are single statements between original statements. No hash or report carries an entity index, address or seed; the shared
stream is observed only by the number of draws between its state before and after a solver call (a local copy of the generator is stepped to the later state). A hook reads arguments, results and entity fields the original has already produced; it evaluates no campaign getter the original did
not and draws nothing. Outputs the original leaves unwritten on some exits (the lead solver's time of flight on its range-check and off-map exits; the ballistic lookup's outputs on failure) are passed as pointers and dereferenced only where the original wrote them (a call-boundary defect found by the lead in `b7f907894`
and corrected in `f9f15f52f`; see "Observer correction").

**There is no probe.** The pitch-device solver draws from the shared random stream and the weapon-system update turns devices, so a sweep outside the game's own order would need the stream saved and restored; the natural calls during stepping are numerous (below), so the fixture observes those and nothing else.
The Lua driver boots the campaign through the public path (`require ("eech_dc")`, the reference host's six callbacks), runs frames and reads the run-wide report at 60 s checkpoints and at the end.

## Results

The natural Lebanon witnesses use the original code through `eech-world.exe lua/aim-characterization.lua` (campaign seed 1, 100 ms frames, reference host) on fresh private Lebanon copies, with input manifests identical to `reference/lebanon-3h/inputs.json` before and after each witness (`checks/`). Georgia uses its own root and manifest; the ordinary 900-second controls use `campaign.lua`; the controlled person cases use `aim-person-case.lua`. The person roots have matching after-run manifests only; their before-run identity was not independently verified. Native and driver exits are the process's own (`exit 0` in the retained final-run stderr and logs). Repeat identity applies only to the named pairs below; Georgia and each ordinary control ran once. Intermediate failures and preservation limits are disclosed separately.

### Identities

| | |
|---|---|
| Baseline (original aiming code) | `2608f8022e8a5ce7c19da635af128e34803a4def` (clean; all five original aiming sources and three readiness sources byte-equal to it when the feature is off) |
| Oracle build used for the results | `0ee1fa46b016c9173f05653f6ca22aafa73c834c`, `eech_dc.dll` `df4cc450...e7749`, `eech-world.exe` `f6447884...3000`, `lua.dll` `fdd3325e...3a05`, 0 uncommitted build inputs (`builds/BUILD-INFO-0ee1fa46b.txt`) |
| Person-case build | `74ae6a96d64c5ec994bf4bdd321ce96ff80cf6bc`, `eech_dc.dll` `b52bcc5b...9138`, `eech-world.exe` `19021dab...8477`, 0 uncommitted. It differs from `0ee1fa46b` only by fixture-only additions (`git diff --stat 0ee1fa46b 74ae6a96d`: the two engine entries, their Rust plumbing, `lua/aim-person-case.lua`, its runner; no hooked source, no hook, no ordinary driver) |
| Earlier oracle builds (kept, superseded) | `b7f907894` (first observer; its 600 s pair and 900 s control are retained and valid for that schema), `4927157a5` (per-kind detail; superseded before its runs), `f9f15f52f` (observer-safety correction; 600 s pair, 900 s control and the Georgia run are retained) |
| Toolchain | repository `tools/Dockerfile` image in WSL2 `EECH-D2-Build`, rustc/cargo 1.99.1, MinGW gcc 12-posix `12.2.0-14+deb12u1+25.2+b1`, Lua 5.1.5; wrapper `tools/build-windows-aim-fixture.sh`, a fresh uniquely named cargo volume per build |
| Hashes (Lebanon 600 s, `0ee1fa46b`) | physical `8d5afe5ea932338f`, decision `4e542d029ba7362a`; run A and run B: 1,962 events, raw trace bytes 287,242, SHA-256 `c4e9c2b1...ce62` (`traces/lebanon-r-a.jsonl`, `-r-b.jsonl`) |

Two builds of identical source in different cargo volumes produced different DLL bytes (`b7f907894`: `1ac46498...` dirty-count build, `5044869a...` clean build) and identical traces, so the DLL hash identifies a build, not its behaviour; behaviour was compared by trace.

### What ran (Lebanon, 600 s = 6,000 frames)

The weapon-system update and the vector update each ran 9,429,480 times, about 1,571 per frame, for every entity of all six kinds. Almost all of them end at the first gate:

| vector-update gate reached | calls |
|---|---|
| 1 (a package status exists; no weapon selected) | 9,297,475 |
| 2 (a weapon is selected; no target and not a player) | 18,153 |
| 3 (target or player present; no package with ammunition) | 1,231 |
| 6 (devices found: a pitch-device solve ran) | 112,621 (all valid; 113,852 had a target link) |
| 4, 5 (ended after finding the package / the heading device) | 0 |

Per kind (`aim-kind` rows): vector updates that **reached the devices and ended valid**, and weapon-system updates in which a device **turned**:

| kind | updates | valid vectors | device steps | ready-state steps |
|---|---|---|---|---|
| helicopter | 1,026,828 | 29,644 | 11,664 | 216 |
| routed vehicle | 2,680,192 | 47,691 | 24,742 | 387 |
| ship | 215,797 | 13,595 | 4,153 | 0 |
| anti-aircraft | 4,127,717 | 12,258 | 42,086 | 189 |
| fixed wing | 545,946 | 9,433 | **0** (fixed mounts: valid vectors, no rotation) | 21 |
| **person** | 833,000 | **0** | **0** | 0 |

**Persons never aim in the natural 600 s witness, and the person target-facing branch (`ps_anim.c`) is never entered** (`person_branch_entered` 0, `person_branch_with_target_link` 0, `person_facing` 0), on Lebanon and on Georgia (below). Function connectivity from all six updaters (verified in source) is therefore not runtime coverage; person facing is characterised only by the controlled case below.

Solvers (all of them returned success; failures, tiny-range and off-map exits, triangulated range and the point-blank patch were not exercised):

| | calls | notes |
|---|---|---|
| pitch-device solve | 112,621 | guided-mobile 65,020 (15 weapon types); angle-of-projection 30,681 (5 types); lead-and-ballistic 16,920 (8 types). The guided branch for a non-mobile target (static point) 0 |
| guided intercept prediction | 65,020 | iterated 25,489 (target slower than the weapon); target not moving 39,531; weapon not moving 0; target not slower 0 |
| lead-and-ballistic solve | 47,744 | stationary-target path 40,068; **moving-target path 7,676** (3 iterations 6,210; 5 iterations 1,466); 143 calls outside the vector update (the launch-time call in `wn_creat.c` for the fuze/decoy timer) |
| ballistic-table lookup | 73,704 | all from the lead solver; simplified (non-final iterations) 25,960; fixed-pitch mode (`gd_attck.c`, `co_wpn.c`) **0**; zero time of flight (range <= 0.1 m) 22 |

Value ranges measured: intercept range 5.35 m to 42,432 m; time of flight 0.007 s to 66.1 s; lead angle -0.366 to 1.307 rad; table parent velocity -0.996 (a reversing vehicle: the lookup clamps it) to 65.3 m/s; launch-angle error 0 to 3.129 rad against limits 0.0175 to 1.571 rad; one device step at most 0.1745 rad (10 degrees) per frame in heading.

Readiness (the state machine the fire decision reads): the keyframed ready state of every aircraft and vehicle is advanced by `update_*_weapon_system_ready` on every update whether or not it is drawn (9,429,480 updates: 1,572,774 aircraft, 7,856,706 vehicle). 9,403,903 found it closed, 591 opening, 24,764 open, 222 closing; `open_entity_weapon_system_ready` was commanded 25,322 times and `close_entity_weapon_system_ready` **3,387,517 times** (an idempotent command issued every update by a unit that has a ready device which is not the selected weapon's). The weapon-system update reported the selected weapon system ready 8,804,470 times and not ready 591 times, holding the device in place 474 times (`ws_edge_hold_not_ready`). Opening takes 2 s (rate 0.5 per second).

Devices: 17,842,815 rotating-package visits; 101,131 had located devices; 82,645 required a different offset from the current one; 47,547 heading steps and 1,482 pitch steps stopped short of the required offset (rate limit or wrap); shared-heading arbitration 56,874 steps, shared-pitch 31,904; the ground-launched-guided climb bump (minimum pitch 10 degrees) 33,430.

The fire decision's reads (campaign policy consuming the physical result): launch-angle error evaluated 1,242 (aircraft fire) / 25,271 (aircraft update) / 64,367 (vehicle update) times; rejected for angle 1,177 / 14,869 / 14,548; the vectors were invalid 916 (aircraft update) and 2 (vehicle); the weapon system was not ready 450 (vehicle).

Checkpoint hashes every 60 s (`aim-hash` events, scope `checkpoint`) date any later divergence; the nine physical/decision pairs are in the trace.

### Georgia (`georgia_retail`, 600 s, build `f9f15f52f`)

Complete, native 0. Physical `9bfcb4319945068f`, decision `eb2cf0a10c0696f6`; 8,463,778 vector updates, **6,249 valid vector solutions**, lead path stationary only (3,696), 105 not-ready results, still **person branch 0, fixed-pitch 0**. Georgia is much quieter than Lebanon and adds no coverage the Lebanon run lacks except a second map; it does not close the person gap. (`traces/georgia-f-g.jsonl`; it predates the readiness hooks.)

### Controlled person target-facing case (`74ae6a96d`, `lua/aim-person-case.lua`)

Because the natural witnesses never enter the branch, one bounded fresh case does, through the same host path: boot (reference host, six callbacks, wrapped to record), step 20 s, rank eligible person/enemy-ground-target pairs by their binding-time descriptor positions, then query current state and require a nontrivial turn. The selected pair is `eech-world:3525` and the anti-aircraft vehicle `eech-world:186`: binding separation 5,853.8 m; current queried separation 5,856.5 m, with a turn of 1.65 rad needed. This does not establish the currently closest pair. Give the person the target through the campaign's own `set_client_server_entity_parent` (a case **precondition**; it says nothing about campaign target selection), step 20 frames normally, release the link through the same call and step 20 more. Only host ids cross the fixture entries (`aim_person_case`, `aim_person_query`). `case=control` repeats the selection without the link.

Results (`out/person-target`, native 0 after 21 s; all 11 checks recorded as `check` events, 0 failed): the branch was entered 20 times with a target link; the facing hook fired 20 times; the hook's heading 0.327984422 equals the bearing computed independently in the driver from the hook's own positions (0.327984446; difference 2.4e-8); the turn from the heading before (-1.32664262) is 1.65 rad; the engine's mirror attitude after the first frame equals it (0.327984416); the host was asked to place the person (D1 `campaign_movement_place`) in 20 of 20 frames and **adopted exactly the pose the campaign sent** (attitude in equals attitude out, all nine components); the mirror attitude after each frame equals the attitude the host answered, i.e. the **mirror and the retained D1 host pose agree**; after the release the person's own movement resumed (`person_begin` 20, `person_end` 20) and **all 40 of those steps arrived with exactly the attitude the host had last answered**, so the host (which refuses a campaign write it did not make) accepted the facing as its own record. Control (`out/person-control`, native 0 after 19 s): no link, branch entered 0, facing 0, the person turned 0.0001 rad, 40 host steps (`person_begin`/`person_end` only). The target case was repeated in a fresh root: raw trace byte-identical (SHA-256 `7c5878b4...cf3f94`, 32,887 bytes).

Findings of the case, not corrected: (1) in a development iteration (its trace was overwritten and is not retained) a target 11.7 km away kept the link for one frame and then lost it (the campaign removed it; the removing code was not identified), while the 5.85 km target kept it for all 20 targeted frames; the cause is not characterised. (2) With a link the person's movement steps stop and only `place` steps arrive, which is why the host-record comparison needed the release phase. (3) The original `ps_anim.c` person branch also draws from the shared stream (`rand16()` when it enters an aim state from a non-aim state, and when an idle animation wraps); that draw is not observed by this witness, and the arrangement is a coupling a cut must keep in order.

### Controls

* **Determinism.** Lebanon A and B on separate fresh roots: 1,962 events, 0 differences, equal raw bytes, for each of `b7f907894` (1,720 events), `f9f15f52f` (1,740) and `0ee1fa46b` (1,962). `tools/aim-trace-compare.py` exit 0 (`controls/`).
* **Schema-expanded equivalence.** From `b7f907894` to `f9f15f52f` every old field is equal (the root's `lead-b7-f9-schema-comparison.json`); from `f9f15f52f` to `0ee1fa46b` every old event is present with every old field equal except the declared `physical` hash (it now folds the readiness updates), the kind-table row total and the sample count (`controls/schema-f9f15f52f-to-0ee1fa46b.json`, `schema-compare.py`). The decision hash is `4e542d029ba7362a` in all three; the physical hash is `4702d5a5c44e532c` in the first two and `8d5afe5ea932338f` in the third.
* **Hooks on are observational.** Whole-campaign Lebanon 900 s (`campaign.lua hours=0.25 record_every=10 checkpoint_every=3600 observe_every=60 observe_supply=0`, seed 1, 100 ms, the retained supplied-binary side `los-characterization-baseline-20261026/out/ab-base`, which was **not** re-run): with the hooks on, `b7f907894`, `f9f15f52f` and `0ee1fa46b` each produced a Tacview recording of 11,363,845 bytes, observations of 6,629,273 and metrics of 11,777 whose complete files equal the baseline's (SHA-256 `45b26516...48c348f` / `430db8f2...eef9b` / `ffc743d3...3474`; `controls/ab-*-vs-supplied-baseline-900s.json`, `ab-compare.py`, native exit 0 each; 1,191 to 1,292 s). The recording was generated, byte-compared and structurally checked (`observation-check.py` PASS, 900 frames, `controls/ab-*-observation-check.json`); it was **not** visually inspected in Tacview. 447 weapons were launched in the window, so the aiming outcomes are in those bytes. The configuration is the one recorded for the baseline side (the EEDCS-42 transcript's command; same scripts); that the baseline root was an unmodified Lebanon copy is inferred, not verified.
* **Feature off is the original text.** The root's strict check and mine (`checks/strict-preprocess-0ee1fa46b.log`) preprocess all eight instrumented sources (`weapon.c`, `wn_move.c`, `ac_wpn.c`, `vh_wpn.c`, `ps_anim.c`, `en_anim.c`, `ac_anim.c`, `vh_anim.c`) with the build's flags: feature OFF is identical to the baseline blob, with every real numeric literal kept and only source-line arguments after quoted `.c` paths normalised, and compiler failures propagated (my first, broader normaliser is superseded and not relied on).
* **The comparator refuses what it should** (`controls/comparator-controls-result-final.json`, 18 cases with the actual Python exit of each): two empty files, checkpoint snapshots without the final report (with and without `--skip-samples`), a complete but **inactive** trace, a trace without the run-wide hash, a truncated copy, a malformed line and a missing file all exit 2 with a reason; a changed run counter, a changed physical hash, a changed checkpoint counter and a changed value range exit 1; complete pairs exit 0.
* **Static.** `cargo fmt --all --check` and `cargo clippy --locked --all-targets -- -D warnings` (default and `--features eech-dc/aim-fixture`) exit 0 at `4927157a5`, `f9f15f52f`, `0ee1fa46b` (`checks/static-checks-*.log`) and at `74ae6a96d` (`checks/static-checks-74ae6a96d.log`). All builds exit 0 with 0 uncommitted build inputs (two builds that showed uncommitted files are kept as superseded: the first counted 14 retained-evidence logs whose working copies were CRLF while the index is LF, byte-equal after CR removal, which I rewrote to the committed LF bytes; the second counted the two untracked evidence paths).

### Observer correction

The lead found that the `b7f907894` wrappers passed `*time_of_flight` (and `*aiming_pitch`) by value at the call site, evaluating outputs the original deliberately leaves unwritten on its range-check, off-map and failure exits. They now pass pointers that the hooks dereference only where the original wrote the value (the angle always; the time of flight on success paths other than off-map; the pitch and time on a table success), and the end hook of the weapon-system update no longer evaluates `config_type` when there is no package status (`f9f15f52f`). Those exits were absent from the recorded runs (`lead_false`, `lead_path_tiny_range`, `lead_path_off_map`, `ballistic_false` all 0), so the `b7f907894` values stand, and the corrected build reproduces them.

### Source hazards: exercised, characterised, not fixed

1. **The lead solver consumes the shared random stream.** On the moving-target path, in its last iteration, `range += 5 * sfrand1()` draws once: 7,676 draws in 600 s, observed as exactly one per moving solve and none on the stationary path (`lead_rng_draws_1` 7,676, `_0` 40,068, `_more` 0). Launch-time calls (`wn_creat.c`) add more (and that function draws again for the fuze timer and the dispersion). A cut must keep these draws in the game's order.
2. **Order inside the update.** The weapon-system update slews devices using the **previous** update's weapon-to-intercept vector and valid flag, then the vector update computes new vectors from the **rotated** devices, and the fire decision reads both in the same update ("ORDER IS CRITICAL" in the family updaters; ready-state advancement first).
3. **Unwritten outputs.** The lead solver leaves `time_of_flight` unwritten on tiny-range, off-map and failure exits, and its caller supplies an uninitialised local; the table lookup writes both outputs only on success. Unexercised here.
4. **Device state lives twice.** The package status holds the angles; the model sub-objects hold a mirror that `get_viewpoint_from_weapon` zeroes and restores each time it is used.
5. **Early `return`s** in the weapon-system update (device not located) skip the remaining packages and the muzzle-flash timers; 0 observed.
6. **Per-update idempotent commands:** 3.39 million `close` commands to an already closed ready device.
7. **Frequency.** 9.4 million aiming-related updates per 600 s against 112,621 solves: any exchange per update would multiply that.

### Not exercised (stated, not covered)

Player and cockpit paths (`ws_edge_non_ai_player`, `_player_local`, point lock, EO tracking point, cannon tracking: all 0; the HUD callers `get_target_intercept_point` and `get_local_entity_selected_weapon_to_target_offsets`: the solver counters outside the vector update are 0, so they never ran); the lead solver's tiny-range and off-map exits, triangulated range and failure; the ballistic fixed-pitch mode and the W1 point-blank patch; the guided static-point branch; the weapon-system update's early exits; the ready animation test branch (`in_flight_articulation_test`); person aiming (natural runs); Georgia beyond 600 s and any other seed or duration; `aa_wpn.c`, `rv_wpn.c`, `sh_wpn.c` (dead code). The hashes are equality evidence (float bit patterns folded into FNV-1a), not an independent oracle of correctness. The device viewpoints come from the reference build's synthetic 3-D scenes (`eech_synth3d_weapons.c`), not retail models.

### Remaining physical coupling a cut must face (observed, not a design)

* **Raw facts that may stay explicit transitional lookups:** the pitch device's world viewpoint (model device tree: synthetic in the reference build), the target-point sub-object and the shape offset table, the weapon database figures and the weapon configuration table (limits, rates, share masks, aiming types), the map-area check, the entity mirrors (positions, motion vectors, speeds, package status).
* **Solver and dynamics that are the physical responsibility** (not a lookup under another name): the guided intercept prediction, the lead-and-ballistic iteration and its random jitter, the ballistic-table interpolation **and the simulation that generates the tables at initialisation** (`generate_ballistics_tables`, a weapon-flight calculation: D2 weapon-execution coupling, not D3), the device slew with its rate limits and shared-device arbitration, the readiness keyframe dynamics, and the person facing.
* **Campaign inputs the physical result depends on:** target link, selected weapon, ammunition and damage (which package's devices are used), the player flag, `make_weapon_system_ready` configuration; and campaign consumers: the launch-angle threshold and its accuracy multiplier, burst/salvo timers, range, altitude and LOS checks.
* **Shared state:** the global random stream (above), `get_delta_time`, the 3-D sub-object mirror, and `campaign_movement_place` for persons.

## Reproduce

From `eech-campaign/` at a clean checkout of `0ee1fa46b`: `sh tools/build-windows-aim-fixture.sh target/aim-fixture/<dir>` (fresh `EECH_WIN_TARGET` volume), then `tools/aim-characterization-windows.ps1 -Bin <dir> -Root <fresh private root> -Out <dir> -Scenario lebanon_retail -Seconds 600 -Checkpoint 60` (one native process; the engine writes into its root). Compare two traces with `tools/aim-trace-compare.py`, read one with `tools/aim-trace-summary.py`, exercise the comparator with `tools/aim-trace-compare-controls.py`. The person case needs `74ae6a96d` and `tools/aim-person-case-windows.ps1 -Case target|control`. The 900 s control is `campaign.lua` with the flags above against the retained baseline files (`controls/ab-compare.py`).

