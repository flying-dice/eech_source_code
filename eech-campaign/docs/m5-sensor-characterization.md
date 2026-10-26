# D2: host-owned physical sensing for the AI target scans (EEDCS-43)

**Status: source-complete and validated on the bounded evidence below; not accepted, not merged.** Task EEDCS-43 (under EEDCS-18), branch `codex/d2-sensor-detections`, draft PR102, from integration
`a7934aa7d1bcffc2ea5057bb3f1b0af332606c61`, **stacked on the reviewed frozen EEDCS-42 head `a42f1cfad` (development stacking: EEDCS-42 is not accepted or merged, and its delivery must precede this task's)**.
Scope authority is the live M5 goal (Confluence 11534337) and the migration ledger (14548994): physical sensing facts are the host's; candidate eligibility, enumeration, ranking, priorities, engagement
and awareness stay in EECH_DC. EEDCS-43 and D2 remain incomplete: no final full pack, independent review, merge or live DCS evidence exists. Artifacts:
[`reference/d2-sensor-detections/`](../reference/d2-sensor-detections/README.md) (the original characterisation, `composition/`, and `extraction/`).

The document has two parts. **Part 1** describes what moved and the evidence for it. **Part 2** is the characterisation of the original code that preceded the move and is the oracle the move was held to.

## Part 1: the extraction

### What moved to the host, and what stayed

The physical sensing of the four AI target scans (`ac_tgt.c` air-to-air and air-to-surface, `vh_tgt.c` surface-to-air and surface-to-surface) is now the host's, behind a sixth required host callback `sense`
(`mb_hostsense.[ch]` in the campaign, `eech-world/csrc/sense_step.c` and `src/sense.rs` in the reference host). The kernel is EECH's own arithmetic expression for expression, under the same float and double types and round-toward-zero.

| Moved (the host owns the calculation and the state between candidates) | Stays in EECH_DC |
|---|---|
| planar squared range (aircraft scans) and `max + (med + min)/4` approximate 3-D range (vehicle scans) | which targets are candidates: side, group, alive, airborne, target type, "group is already best", self |
| the aircraft scans' squared scan range and the stealth-scaled squared limit | candidate enumeration and order, the sector rectangles, including the squared-range quirk and the empty rectangle above about 46 km |
| the surface-to-air envelope: ceiling = group ceiling + the source's own altitude; the floor **lowered in place by every candidate asked about** (and by a human pilot's difficulty factor), never restored within a scan; the radar-altitude test | the `target_count` increments (before the range test for surface-to-surface, before line of sight for surface-to-air) |
| the random-tolerance limit `floor * (1.0 + 0.5 * draw)` and its test | the random **draw**, taken from the private source-derived generator at the original conditional point (after the altitude test); only the draw crosses |
| the emitter range adjustment (1.1 per switched-off emitter of a human-flown target) | the configuration facts it mirrors: player flag, emitter states, radar altitude, difficulty factor, positions |
| the radar-stealth factor of a helicopter: 1, less 0.4 weapon bay closed, less 0.35 stub wings off, less 0.1 undercarriage up | line of sight (host-owned since EEDCS-42), the ranking (`cos_angle`, range weighting, potential threat), best-group choice, engagement, speech |

No second authoritative physical predicate is left in the DLL: the shipping `ac_tgt.o` and `vh_tgt.o` no longer reference `get_sqr_2d_range` or `get_approx_3d_range`, the kernel symbols are in `eech-world.exe` and not in `eech_dc.dll`,
and the DLL holds neither the floor nor the limits (`extraction/checks/symbol-check-37989fa5f.log`). The campaign still squares the aircraft scan range for the sector bounds; that statement is candidate enumeration (the quirk, kept), not the predicate,
which the host squares again from the raw figure.

### The boundary

One callback, `sense`, in the same Lua -> `eech_dc` -> host path as the others (`dc.boot { sense = ... }`), with at most one scan open (`mb_hostsense.h`):

| Message | Contents | Host answers |
|---|---|---|
| `begin` | scan, source host identity (when the roster has one), source position, the unit's raw scan range; surface-to-air adds the group's raw ceiling and initial floor | `open` |
| `candidate` | target host identity (when it has one), position, player flag (aircraft-to-aircraft and surface-to-air), difficulty factor (a human pilot), radar altitude (surface-to-air) | `reject` with the failed predicate, `accept` with the measured range, or `need_draw` (surface-to-air) / `need_signature` |
| `draw` | the campaign's random draw in [0, 1] | `reject` (tolerance) or `need_signature` |
| `signature` | emitter states (human-flown targets) and, for a helicopter target, weapon-bay-closed, stub-wings-attached, undercarriage-up | `accept` or `reject` (range adjusted) |
| `end` | | `closed` |

The measured range also comes back on a rejection and is what the ranking uses. Validation in the campaign (`mb_hostsense.c`) refuses: a missing callback (boot: `eech: missing host sense callback`), an unknown state, a state the message cannot receive, a reject reason the scan cannot have,
an acceptance without a finite non-negative range, a non-finite value, a scan inside another, and more than 16 requests for one candidate; the Rust layer refuses non-boolean flags, a draw outside [0, 1], non-finite positions and an overlong or non-UTF-8 identity.
A host that owns the world can decide at `candidate` from its own state and never be asked for a draw or a signature. There is no campaign fallback. An optional `echo` table of the reference host's intermediate values is read only by the test-fixture build; the shipping module ignores it.
No pointer, entity index, list layout or seed crosses: identities are the ones the roster already holds.

**Declared transitional coupling** (facts the host is given, not physical authority): the unit-database figures (scan range, ceiling, floor), entity positions, the target's radar altitude, the helicopter's bay/stub-wing/undercarriage state, the emitter states, the pilot's difficulty factor, the sector entity lists that
supply candidates, and terrain and object collision for line of sight (D3, D5). The draw's generator is the campaign's own; how a DCS host would be given a stable per-source tolerance is open.

### Evidence

All runs are on private fresh copies of the Lebanon root (manifests IDENTICAL to `reference/lebanon-3h/inputs.json` before and after), one native process at a time while the lead's full packs ran. Details, hashes and commands: `reference/d2-sensor-detections/extraction/README.md`.

* **Red against the original (before the cut).** The retained composed original build `1fc0cdc62` booted with the new provider (`mode=blocked`, 300 s): it ran, consulted the provider **0** times (begin = candidate = 0) and the world was the unmodified one (185 weapon births, 6 losses). A concrete missing behaviour, not a setup failure.
* **The original oracle held across the move.** The same 600 s Lebanon witness (probes at 30, 300, 590 s; seed 1) on the extraction's fixture build `37989fa5f`: native exit 0; physical hash `d60c462829c9714e`, decision hash `2611b04d99f5c689`;
  the raw trace file is byte-identical to the composed original's (`bfb8fab3...`) and the corrected comparator reports **0 differences, Python exit 0** for run (75 events), probe (182), run-at-probe (186) and default (257). The host's numbers, folded from its echo, equal what the original computed in the DLL.
* **First causes of the two differences found on the way** (both kept in `extraction/intermediate/`): the first run's physical hash differed in one value, the tolerance limit, one ulp low (`...538` for `...54`). Cause: the existing `exact_f64s` reads a double through `string.format('%.17g')`, and the engine runs a callback under round-toward-zero, where the C library truncates the digits.
  Measured over 60,000 doubles, 4,010 come back one ulp low (3,199 of the 32,768 draws `k/32767`). The sensing dialogue reads its doubles in exact steps instead (`exact_f64_bits`, tested under both rounding modes); the existing reader and its D1 uses were left untouched and are reported to the lead as a finding.
  The second run (a rebuild after an `aphavoc/`-only edit) still carried the old fixture hook, because `eech-engine-sys` reruns its build script only when `csrc/`, `compat/`, `build/` or `sources.txt` change, not when the campaign sources do: a reused cargo volume linked stale C. Identity builds now use fresh volumes.
* **Causal, through the public path** (ordinary-feature shipping build `37989fa5f`, Lebanon, 300 s, seed 1, one run each, so a demonstration and not statistics):

  | Provider | begin | candidates | draws | signatures | accepts | weapon births | losses |
  |---|---|---|---|---|---|---|---|
  | reference host (and the original with its provider ignored) | 16,840 | 277,652 | 174 | 5,341 | 356 | 185 | 6 |
  | host reports every candidate undetectable (`blocked`) | 17,047 | 324,946 | 0 | 0 | 0 | 176 | 1 |
  | reference host, every scan range tripled (`far`) | 16,334 | 81,876 | 551 | 2,726 | 4,303 | 208 | 10 |

  The reference host reproduces the original's 185/6 exactly; what the host perceives changes what the campaign does, while which targets are candidates, the order, the ranking and the engagement stay the campaign's. Blocking all detection does not stop all launches (176 births), as for line of sight: launches that do not depend on a detection are not separated here.
* **Missing and malformed providers** (same build): no provider fails at boot (`eech: missing host sense callback`); at the first scan or candidate that reaches it each of a begin answered with an accept, a nil, an unknown state, a reject reason no scan has, an accept with a non-finite range, a `need_draw` for a scan with no tolerance, and a Lua error stops the engine with `EECH fatal error: host sense: ...`
  and the specific boundary reason (`extraction/providers/`). The baseline module has no such concept and ignores the key by construction.
* **Whole campaign, Tacview unchanged.** The ordinary-feature shipping candidate ran Lebanon for 900 s (`campaign.lua hours=0.25 record_every=10 observe_every=60 observe_supply=0`, native exit 0): the Tacview recording (11,363,845 bytes), observations (6,629,273) and metrics (11,777) have the same SHA-256 as the retained supplied-baseline-binary run;
  the recording passed `observation-check.py` (generated, byte-compared and structurally checked; **not** visually inspected). The configuration is verified as in the earlier control (same argument list, defaults seed 1 and 100 ms, identical Lua scripts); one scenario, 900 s, not the full pack.
* **Checks** (cargo's own exits, fresh clippy volume): `cargo fmt --all --check` 0; `cargo clippy --locked --all-targets -- -D warnings` for the default features, for all fixtures together (dependent, loss, LOS, sensor), for old-guard plus sensor, and for sensor alone: all 0. `cargo test --locked -p eech-engine -p eech-dc -p eech-world` (default features): 3+2, 30, 165, 19, 6, 2 tests pass.
  New focused contracts: nine kernel contracts derived from the original scans (range forms and their quirks, the cumulative floor and its 0.1 bound, the ceiling at the source's altitude, tolerance after altitude, the stealth subtractions, the emitter adjustment, refusals), the Lua boundary of `host.sense`, and the exact-double transport under both rounding modes.
* **Cost.** 316,847 host crossings in 300 s (about 1,056 per second). Wall time for 300 simulated seconds was 273 s with the reference host and 276 s for the original with no crossings, measured under different background loads: no penalty was visible, and this is not a benchmark. No batching was introduced.

### Coverage that is NOT claimed

* Player-controlled targets, the difficulty factor and the emitter-off adjustment are not exercised by any campaign run (the headless world has no player aircraft; 0 of 265 aircraft entities); they are covered only by the kernel contracts, which are not the whole campaign. Altitude above the ceiling is likewise unexercised in the campaign (0 in the original run).
* One scenario (Lebanon), one seed, 600 s and 900 s. Georgia was not run. The full 3 h pack, the independent review and merge are the lead's. `cargo test` was run with default features only.
* The equality evidence is a float-bit fold and a byte comparison, not an independent oracle of correctness.

# Part 2: the original behaviour that preceded the move (the characterisation the extraction was held to)

The sections below describe the **original** code as it was before EEDCS-43 moved the physical predicates; line references, the "hooks" and the "recomputed" values describe the characterisation build. In the extraction the same hooks are fed from the host's echo.

## Active call paths (verified in source, not inferred from names)

| Updater | Calls | Scan functions (file) |
|---|---|---|
| `hc_updt.c` (helicopter), `fw_updt.c` (fixed wing) | `update_aircraft_target_scan` | `get_aircraft_air_to_air_scan_group_targets`, `get_aircraft_air_to_surface_scan_group_targets` (`ac_tgt.c`, file-local) |
| `rv_updt.c` (routed), `sh_updt.c` (ship), `aa_updt.c` (anti-air), `ps_updt.c` (person) | `update_vehicle_target_scan` | `get_vehicle_surface_to_air_scan_group_targets`, `get_vehicle_surface_to_surface_scan_group_targets` (`vh_tgt.c`) |

The four group scan functions have no other caller. Gating before them is campaign policy and unchanged: offensive capability, scan delay,
group `ENGAGE_ENEMY`, aircraft airborne, task leader / primary task / AI player and the task's `TARGET_SOURCE` (aircraft); troop-put-down
waypoint, task `TARGET_SOURCE`, first group member only (vehicles). The scan timers draw from the **global** stream (`frand1`, `sfrand1`) at each
reset; the scans themselves draw nothing from it. Air-to-air also runs air-to-surface after it for `SCAN_ALL`; surface-to-air also runs
surface-to-surface for everything except `SCAN_AIR` (`TASK_OBJECTIVE` returns before either). The routing is observed (run table below): all six families scan.

## What each scan does (source), and which parts are physical facts

| | Air-to-air (aircraft) | Air-to-surface (aircraft) | Surface-to-air (vehicle group) | Surface-to-surface (vehicle group) |
|---|---|---|---|---|
| Range test | squared 2-D range (x, z) of the pair, scaled for emitters, against `scan_range²` times the target's stealth factor | squared 2-D range against `scan_range²` | approximate 3-D range (`max + (med + min)/4`) scaled for emitters, against `scan_range` times stealth | approximate 3-D range against `scan_range` |
| Altitude envelope | none | none | target **radar altitude** against a **cumulatively mutated** floor, and against ceiling = group ceiling + the source's own y; then a **per-source random tolerance** | none |
| Emitters | non-AI-player target only: radar, radar jammer, IR jammer each switched off multiplies the compared range by 1.1 | none | same as air-to-air | none |
| Stealth | `FLOAT_TYPE_RADAR_STEALTH_FACTOR` of the target | none | same | none |
| Line of sight | none | none | `check_entity_line_of_sight` (host-owned since EEDCS-42) | none |
| Campaign-owned in the same loop | side, airborne, target type, group, "group is already best", alive, self; sector enumeration; ranking by `range × (1 − cos)`; best group | the same, plus the identify and "aircraft on the ground" filters | the same, plus weighted range and potential-threat ranking | the same, no altitude or LOS |
| `target_count` is incremented | after the range test | after the range test | after altitude, tolerance, range and stealth, **before** line of sight | **before** the range test |

## Method

A gated observer (`mb_sensorfx.h` hooks in `ac_tgt.c` and `vh_tgt.c`, `csrc/eech_sensor_fixture.c`, engine entries in `eech_engine.c`, feature
`sensor-fixture`, wrapper `tools/build-windows-sensor-fixture.sh`, driver `lua/sensor-characterization.lua`) records, in two separate hashes:

* **physical hash**: per call the source position and scan envelope, per candidate that reaches a physical predicate its measured range, the floor
  after the original's mutation and the floor an independent per-target threshold would have given, radar altitude, the tolerance draw and limit,
  the emitter flags the original actually read, the stealth factor and stealth-scaled limit, and the verdict (accepted, or the predicate that
  rejected it), in the order the scan computed them;
* **decision hash**: each eligibility skip by reason, the ranking inputs (`cos_angle`, result or select criteria, whether it displaced the best)
  and the call outcome (`target_count`, the best group's position, its score).

No hash or report carries an entity index, address or seed. The tolerance draw is recomputed on a local copy of the original's seed, so neither
the game's random stream nor the original's variable is touched; the hook compares its recomputed outcome with the original's own branch
(`recompute_mismatch`). Hooks run in the engine's own frames and entries, under its round-toward-zero mode, and the recomputed expressions keep the original's float and double
types (`float * double` for the tolerance limit, `float *= double` for the floor). Hooks read only; reads of emitter fields happen only where the original reads them. The probe walks the engine's own used-entity list (never
an index), accepts only allocated entities in the six families and runs both scans of each eligible source (alive, offensive, scan delay, group;
airborne for aircraft; first group member for vehicles) with no engagement, speech or timer change; it has a trace of its own.
The Lua driver boots the campaign through the public path (`require ("eech_dc")`, the reference host's callbacks), runs frames, takes probes at 30, 300 and 590
simulated seconds and reads the run-wide report at the end. The campaign decides what to scan.

## Results: Lebanon (`lebanon_retail`, seed 1), 600 simulated seconds, 6,000 frames of 100 ms

Run-wide, fixture `e0d58d818` (hashes equal to the first build `60f6a4876`):
physical `d60c462829c9714e`, decision `2611b04d99f5c689`. Re-read with `tools/sensor-trace-summary.py`.

| | air-to-air | air-to-surface | surface-to-air | surface-to-surface |
|---|---|---|---|---|
| calls / scanned (a usable scan range) | 1,726 / 1,726 | 1,676 / 1,676 | 19,363 / 19,363 | 19,363 / 12,351 |
| candidates that reached a physical predicate | 36,354 | 1,059,762 | 2,444 | 5,771 |
| accepted | 10 | 12 | 100 | 595 |
| rejected: range | 0 | 1,059,750 | 0 | 5,176 |
| rejected: stealth/emitter-scaled range | 36,344 | 0 | 859 | 0 |
| rejected: altitude (all below the floor; none above the ceiling) | 0 | 0 | 1,046 | 0 |
| rejected: random tolerance | 0 | 0 | 319 | 0 |
| rejected: line of sight | 0 | 0 | 120 | 0 |
| `target_count` summed / calls returning a group | 10 / 7 | 12 / 10 | 220 / 93 | 5,771 / 223 |
| calls with `target_count > 0` and **no** group | 0 | 0 | 44 | 345 |
| candidates with a stealth factor other than 1 | 28,032 | 0 | 787 | 0 |
| player-controlled candidates / emitter switched off | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| `cos_angle` values (min, max) | 10 (−0.997, 0.906) | 12 (−0.981, 0.99994) | none | none |
| sector rectangle: whole map / empty | 1,547 / 179 | 1,676 / 0 | 0 / 0 | 0 / 0 |
| scan range, min to max (m) | 3,000 to 80,000 | 4,000 to 8,000 | 2,000 to 14,000 | 300 to 15,000 |

Sources: helicopters 1,471 and fixed wing 255 (205 air-to-surface) for the aircraft scans; routed vehicles 1,600, ships 552, anti-air 16,195, persons 1,016 for
the vehicle scans. The three probes (a sweep of every eligible source against the live world, 555, 591 and 610 sources) have their own hashes
(`597fd62d647eadc7`/`912d5270e53abcbc` at 30 s, `61eb294050ce0883`/`ffaa7cc371527fd9` at 300 s, `60159ea2579ffa5f`/`9cdc6c8bbdd8a8c4` at 590 s)
and, being different world states, show the same classes of outcome. No aircraft is airborne at 30 s, so the aircraft scans first appear in the second probe. There is
no player aircraft in the world at any probe (0 of 265 aircraft entities).

Source hazards that are exercised, characterised and **not** fixed:

1. **Cumulative surface-to-air scan floor.** `source_air_scan_floor` is multiplied in place for each candidate that gets past the eligibility
   filters, including the player difficulty factor, and is never reset inside the call. 606 calls evaluated the floor for two or more candidates;
   in 288 of the 2,444 candidate evaluations the mutated floor differed from an independent per-target threshold, and the floor fell to 0.048 of its
   initial value within one call. Candidate order and the eligibility skips therefore change the result. An extraction that gives each target an independent
   threshold would change behaviour.
2. **Random tolerance from a source-derived seed.** The seed is the source entity's heap slot (`get_client_server_entity_random_number_seed`), drawn once
   per candidate with `frand1x`, so the draw is a fixed number per source: 1,398 draws between 0.0137 and 0.996, `varies_within_call` 0,
   `recompute_mismatch` 0, 319 rejections (limit = floor × (1 + 0.5 × draw)). It is applied after the altitude test only, with the mutated floor. A host that has no heap slot needs a
   stable per-source value; that is an extraction decision, not made here.
3. **Aircraft scans square the range before the sector bounds.** The rectangle is therefore the whole map in every air-to-surface call (1,676 of 1,676, 632 candidates
   evaluated per call on average) and in 1,547 of 1,726 air-to-air calls, and 1,059,762 air-to-surface candidates were evaluated for 12 accepted.
   In the other 179 air-to-air calls the squared range leaves `int` range inside `convert_float_to_int` (any scan range above about 46.3 km, the square root of 2³¹; the retained samples are
   80,000 m, squared 6.4e9), the conversion gives `INT_MIN`, the sector bounds become `x_max = z_max = −524288` and **no sector is visited**: such a source finds nothing in these runs.
4. **`cos_angle` is a true cosine.** The task text called it a raw unnormalised dot product. In source, `get_3d_vector_dot_product` normalises both vectors on
   local copies (a zero vector gives 0) and returns a value in [−1, 1]. The measured values (22 in this run, 152 in one probe) are all inside it (`cos_outside_unit` 0). The
   ranking `range × (2 − (cos + 1))` therefore uses a real cosine and the 2-D squared range.
5. **`target_count` and the count/group mismatch.** Surface-to-surface counts before the range test and surface-to-air counts before line of sight, so a call can report
   `target_count > 0` with no group (345 and 44 calls). The caller then does nothing; the quirk is preserved and counted.
6. **The ceiling mixes frames.** The group ceiling is added to the source's absolute `y` but compared with the target's radar altitude (height above terrain).
   `altitude_above_ceiling` was 0 in this run, so that branch is **unexercised**.
7. **Stealth scales differently.** It multiplies the *squared* range in air-to-air (effective radius scales with its square root) and the plain range in surface-to-air.
   The factor is derived from the target helicopter's configuration (`hc_float.c`: Comanche weapon bay, stub wings, undercarriage; 1.0 for other types); values 0.15 to 1 were observed.

## Non-perturbation and inertness (bounded controls)

* **The probe is inert.** The run-wide report with probes at 30/300/590 s and with none (`probe_at=`) is identical event for event, including every counter and
  sample (`tools/sensor-trace-compare.py --scope run`: 75 events, 0 differences), and the run-wide report before and after each probe was equal
  (`run_trace_untouched_by_probe` true at all three). Two separate roots and two builds (`60f6a4876`, `e0d58d818`) give the same run hashes, so a candidate's hash is a meaningful comparison.
* **Feature off is the original text.** `checks/preprocess-equivalence.log`: with the shipping definitions, `ac_tgt.c` and `vh_tgt.c` preprocess to text identical to the base
  (blank lines dropped, assertion `__LINE__` and path normalised; the added hook lines shift the line numbers inside assertion messages, nothing else). With the feature on, 143 lines are added to each.
* **Hooks are observational at campaign level.** The fixture build (hooks on, `e0d58d818`) ran the whole Lebanon campaign for 900 s (`campaign.lua scenario=lebanon_retail hours=0.25 record_every=10
  observe_every=60`, exit 0): the Tacview recording (11,363,845 bytes), observations (6,629,273) and metrics (11,777) have the same SHA-256 as the supplied-baseline-binary run the lead recorded
  (`independent-ab-comparison-20261026.json`; that side was **not** re-run here). Configuration, verified from retained records: the original run's command is in the EEDCS-42 lane transcript and is the same
  argument list as this one (`hours=0.25 record_every=10 checkpoint_every=3600 observe_every=60 observe_supply=0`; neither passes `seed` or `frame_ms`, so the defaults, seed 1 and 100 ms, apply to both), and
  `campaign.lua`, `metrics.lua` and `observations.lua` are byte-identical in the supplied bin and the fixture bin. Inferred, not verified: that the original run's root was an unmodified Lebanon copy (its manifest was not re-compared
  here; the fixture root was, before and after). Identical bytes are the non-perturbation result for **this** configuration only; no other scenario, seed, duration or observation setting is claimed. The recording was generated,
  byte-compared and structurally checked (`observation-check.py`: PASS, 900 frames, 0 failures); it was **not** visually inspected in Tacview. This is 900 s of one scenario, not the 3 h pack.
  `reference/d2-sensor-detections/controls/`.

* **Static checks** (Linux image, disposable containers, cargo's own exit captured, `checks/`): `cargo fmt --all --check`, `cargo clippy --locked --all-targets -- -D warnings` (default
  features) and `cargo clippy --locked -p eech-engine -p eech-dc --features eech-dc/sensor-fixture --all-targets -- -D warnings` all exit 0 at `60f6a4876` and again at `e0d58d818`. Both Windows fixture builds
  exit 0 from a clean checkout (0 uncommitted build inputs). **Not run:** `cargo test`, the shipping-feature Windows build and its pack, any Georgia run, any visual Tacview inspection.
* **The comparator fails closed** (correction after review feedback). `tools/sensor-trace-compare.py` first passed empty files and probe-only snapshots with exit 0 (two real red controls, retained in
  `controls/comparator-inputs/`). It now exits 2 with a reason for a missing, empty, malformed or incomplete file, and for a selection that matches no event; a complete trace must end with the terminal
  `sensor-samples` event of scope `run`, required whatever scopes are selected and also with `--skip-samples`; exit 1 is a difference; exit 0 only for complete files with at least one selected event, all equal in
  value and order. `tools/sensor-trace-compare-controls.py` records the actual Python exit of each case in `controls/comparator-controls-result.json` (comparator SHA-256 `d5027336...1165`): the two red controls and
  a `--skip-samples` variant exit 2; the retained complete pair `--scope run` exits 0 (75 events, 0 differences); a complete probe trace against itself exits 0 (`--scope probe`: 182 events; default scopes: 257);
  a copy with one run-wide counter changed by one exits 1; a truncated copy, a malformed copy, a missing file and a selection with no events exit 2. The earlier "75 events, 0 differences" in this document was
  produced by the pre-correction tool on complete files; the corrected tool gives the same result.

## Unexercised edges (stated, not covered)

* Player-controlled targets: the difficulty factor, the emitter-off range adjustments and the all-criteria line-of-sight form (`player_candidates` 0 everywhere; no player aircraft exists in
  the headless world). The bounded probe cannot set a player flag without writing campaign state, so these remain uncharacterised.
* Altitude above the ceiling; the `target_type` invalid skip; the `dead` skip outside surface-to-air; a zero-length target vector.
* Georgia (`georgia_retail`, `georgia`) and any seed other than 1; durations past 600 s. One scenario at one seed was run because it exercised every physical predicate except those above; Georgia was not needed to resolve a coverage hole.
* The numbers are equality evidence (float bit patterns folded into FNV-1a), not an independent oracle of correctness.

## The bounded physical-fact boundary that followed

Measured before the move, per candidate and per source, the facts a host can answer: the pair's distance (planar squared or approximate 3-D), the target's height above terrain, the target's radar signature factor, and (for players) its radar and jammer state, plus visibility (already host-owned).
The campaign keeps the source's scan envelope (range, floor, ceiling from its unit database), the candidate set, every eligibility filter, the ranking and the engagement. Two properties constrained the extraction and were preserved: the surface-to-air floor makes that scan **sequence-dependent**
(hazard 1), and the tolerance draw is a fixed per-source value from a private seed (hazard 2). The interface that followed is Part 1's.

**Preconditions that were kept explicit and are now met:** the surface-to-air scan reaches the required `line_of_sight` provider through `campaign_los_query` (the extraction kept that entry wired, and the sensor probe is handed the same complete host as a frame); the tolerance seed is the source's heap index
(`get_client_server_entity_random_number_seed (EN) = EN - entities`), a private legacy value that appears in no public payload, trace or host message: only the draw crosses.

## Composition with the host-owned line of sight (development stacking)

The reviewed frozen EEDCS-42 head `a42f1cfad` is merged under the observer (`1fc0cdc62`, then `544debee3`) so the observer can run across the required LOS entry wiring; EEDCS-42 is not accepted and delivery 42 precedes 43. The sensor probe now enters with the complete
host, and the driver supplies the `line_of_sight` provider. The same 600 s Lebanon witness produced a trace byte-identical to the original one above (same SHA-256, corrected comparator 0 differences on all scopes). Details and limits:
[`reference/d2-sensor-detections/composition/README.md`](../reference/d2-sensor-detections/composition/README.md). Nothing in this document's formulas or findings changes.

## Reproduce

From `eech-campaign/` at a clean checkout: `sh tools/build-windows-sensor-fixture.sh target/sensor-fixture/<dir>` (Docker image `eech-build`; the shipping recipe and Dockerfile are untouched), then
`tools/sensor-characterization-windows.ps1 -Bin <dir> -Root <private root> -Out <dir> -Scenario lebanon_retail -Seconds 600 -ProbeAt 30,300,590` (one native process; the
engine writes into its root, so use one fresh copy per run). Compare two traces with `tools/sensor-trace-compare.py`; read one with `tools/sensor-trace-summary.py`.
