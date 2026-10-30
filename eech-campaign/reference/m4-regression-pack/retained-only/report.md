# Regression pack: INCOMPLETE (retained tier only)

- pack revision: `23016c4fde9cb591540e56fc672af3ca4d79bb01`; run 2026-09-27T22:43:57
- command: `tools/regression-pack.py --retained-only --out target/regression-pack/retained-self-check`
- roots: none (the runtime tier did not run)
- seed 1, 3 simulated hours, 100 ms frames; the scenarios, inputs, artifacts and checkers are listed in `regression/pack.json`

## Claims

| Status | Claim | Checks | Accepted | Sensitivity |
|---|---|---|---|---|
| **NOT RUN** | `M1.georgia.mechanics`: Retail Georgia plays out as an EECH campaign for 3 h: the expected air and ground task types, combat and helicopter and vehicle losses on both sides, red SUPPLY flown, airbases consuming ammo and fuel, keysites changing hands | `regress.georgia.expectations`: NOT RUN | #19 (docs/m1-baseline.md) | none demonstrated for Georgia's expectations: without S3 all 30 still hold (#68) |
| **NOT RUN** | `M1.georgia.war`: Georgia's 3 h war reproduces its accepted baseline exactly: every metric at every checkpoint | `regress.georgia.exact`: NOT RUN | #19 (docs/m1-baseline.md) | demonstrated: S3 removal breaks the exact comparison, and 11 metrics leave tolerance (#68; re-checked offline by sensitivity.s3_exact) |
| **NOT RUN** | `M1.lebanon.mechanics`: Retail Lebanon plays out as an EECH campaign for 3 h: task types, combat and losses, both sides flying SUPPLY, airbases consuming, jets and helicopters regenerated from reserves, factories and refineries producing | `regress.lebanon.expectations.mechanics`: NOT RUN | #19 (docs/m1-baseline.md) | reported: with regen disabled the four regen expectations fail (regression/README.md; no retained artifact) |
| **NOT RUN** | `M1.lebanon.resupply`: Lebanon airbases are resupplied with ammo and with fuel within 3 h | `regress.lebanon.expectations.resupply`: NOT RUN | #19 (docs/m1-baseline.md, docs/corrections.md) | demonstrated: without S3, 'airbases resupplied with ammo' fails (#68; re-checked offline by sensitivity.s3_lebanon_expectation) |
| **NOT RUN** | `M1.lebanon.war`: Lebanon's 3 h war reproduces its accepted (post-S2) baseline exactly | `regress.lebanon.exact`: NOT RUN | #19 (docs/m1-baseline.md; baseline from #63) | demonstrated: S3 removal (#68: 13 metrics outside tolerance) and S2 (#63: 23 outside tolerance, all 39 expectations held) |
| **NOT RUN** | `M1.lifecycle.one_boot`: One boot per process: a second boot raises 'not valid in the engine's state' and the first engine keeps running | `lifecycle.boot_twice`: NOT RUN | #19 (docs/m1-baseline.md) | reported: a root without the Lebanon map made 4 cases fail (docs/m1-baseline.md, at a8661eaa; no retained artifact) |
| **NOT RUN** | `M1.lifecycle.bad_arguments`: Argument errors (unknown gunship, missing fields, a non-table config, a negative or non-numeric frame step) are Lua errors raised before the engine is touched; a valid boot and frame (0) still work afterwards | `lifecycle.bad_arguments`: NOT RUN | #19 (docs/m1-baseline.md) | reported (as above) |
| **NOT RUN** | `M1.lifecycle.rejected_root`: An install root the engine rejects is a Lua error ('rejected by the engine'), and uses up the process's boot | `lifecycle.missing_root`: NOT RUN | #19 (docs/m1-baseline.md) | reported (as above) |
| **NOT RUN** | `M1.lifecycle.fatal_error`: An EECH fatal error at boot (a missing campaign file) is a Lua error, not a crash, and uses up the process's boot | `lifecycle.missing_campaign`: NOT RUN | #19 (docs/m1-baseline.md) | reported (as above) |
| **NOT RUN** | `M1.lifecycle.no_reuse`: No shutdown, reset or reuse: collecting the engine or requiring the module again does not allow another boot | `lifecycle.no_reuse`: NOT RUN | #19 (docs/m1-baseline.md) | reported (as above) |
| **NOT RUN** | `M1.lifecycle.missing_map_crash`: Known defect, characterised: a missing map directory crashes the host process during boot (0xC0000005); the check reports any change | `lifecycle.missing_map`: NOT RUN | #19 (docs/m1-baseline.md): an accepted limitation, not desired behaviour | reported (as above) |
| **NOT COVERED** | `M1.lifecycle.unverified`: A fatal error during frame poisons the engine; threading and reentrancy; host callbacks | - | #19: accepted as unverified or unsupported |  |
| **NOT RUN** | `M1.corrections.S3`: S3: a SUPPLY transport picks up a crate of its task's cargo type | `regress.lebanon.expectations.resupply`: NOT RUN | #62 (defect correction; docs/corrections.md) | demonstrated (#68): Lebanon's ammo-resupply expectation fails, and both exact baselines differ |
| **NOT RUN** | `M1.corrections.S2_removed`: S2 rejected and removed: the original EECH supplier rule (no task when the chosen supplier holds no crate of the type) | `regress.lebanon.exact`: NOT RUN | #62 (rejected), #63 (removed) | demonstrated (#62/#63): with S2, Lebanon differs in 23 metrics beyond tolerance while all 39 expectations hold: only the baseline would notice |
| **NOT RUN** | `M1.corrections.S1`: S1: an airbase low on supplies is not chosen as its own supplier | `regress.lebanon.exact`: NOT RUN<br>`regress.lebanon.expectations.resupply`: NOT RUN | #62 (defect correction) | not demonstrated: reported before S3 existed (1b47fa02: no airbase was ever resupplied) |
| **NOT RUN** | `M1.corrections.portability`: P1, B2, X1: entity attribute lists, UI attribute pointers and float-to-int conversion behave on x86-64 as the original does on 32-bit MSVC | `regress.georgia.exact`: NOT RUN<br>`regress.lebanon.exact`: NOT RUN<br>`lifecycle.boot_twice`: NOT RUN | #62 (defect corrections, portability) | not demonstrated at this candidate: reported that without them boots crash (B2), keysites land in the wrong sectors (X1) and every entity creation reads garbage (P1) |
| **NOT RUN** | `M1.corrections.load_time_data`: T1, T2 (accepted compatibility decisions) and C1: the mixed retail/community data set loads | `regress.georgia.exact`: NOT RUN<br>`regress.lebanon.exact`: NOT RUN<br>`lifecycle.boot_twice`: NOT RUN | #62 (T1/T2 accepted for this data profile only; C1 defect correction) | not demonstrated at this candidate: reported that without them loading stops (T1/T2 fatal) or faults (C1) |
| **EVIDENCE ONLY - not automatically checked** | `M1.corrections.W1`: W1: no ballistic solution when the target's height exceeds its range (no NaN table index) | - | #62 (defect correction) | fault reported (231bc98c); whether it triggers within 3 h is unknown |
| **NOT COVERED** | `M1.corrections.N3`: N3 and its accepted fallback value: degenerate terrain faces take the local vertex height instead of NaN | - | #62 (defect correction; fallback value accepted) | demonstrated NOT exercised in 3 h: the baselines predate N3 and still reproduce exactly; the fault it prevents was reported at 39,327 s |
| **NOT COVERED** | `M1.corrections.N1_N2`: N1, N2: NaN positions are reported and guarded in terrain queries and smoke | - | #62 (defect corrections) | nothing reported since N3; no 3 h run is known to reach them |
| **EVIDENCE ONLY - not automatically checked** | `M1.corrections.H1`: H1: an empty campaign history is not read at index -1 | - | #62 (defect correction) | exercised whenever a campaign item is destroyed headless, but a reversion is an out-of-bounds read whose effect on the metrics is unknown; AddressSanitizer reported (5f07697b) |
| **NOT COVERED** | `M1.corrections.U1`: U1: the co-pilot report button is sized from its 2-entry array | - | #62 (defect correction) | UI sizing only: nothing headless observes it |
| **EVIDENCE ONLY - not automatically checked** | `M1.corrections.F1`: F1 (accepted compatibility decision): uninitialised locals read as zero across the engine | - | #62 | one site traced (create_supply_task); what a reversion would change is unknown |
| **EVIDENCE ONLY - not automatically checked** | `M1.corrections.compile_only`: E1-E3 (compile-only) and -fwrapv, -fno-strict-aliasing, -fcommon (keep the original compiler's treatment) | - | #62 (no behavioural change intended) | traced; E1-E3 reversions fail to compile; the flags' equivalence is not independently verified |
| **EVIDENCE ONLY - not automatically checked** | `M1.rebuild`: A clean rebuild reproduces the candidate's code and data byte for byte (not its file hashes) | - | #60, #19 | checked once at a8661eaa; no retained artifact |
| **EVIDENCE ONLY - not automatically checked** | `M1.kernel`: Supporting kernel conformance: 1,401 corpus scenarios and the float fixtures match the C reference (Linux x86-64) | - | #19, as supporting evidence only (not the public path) | cargo test in the eech-build image; not part of this Windows pack |
| **NOT COVERED** | `M1.longer_runs`: Lebanon to a conclusion, 14 h and 195 h Lebanon runs, 24 h AddressSanitizer runs | - | #19: reported, not reproduced; no retained artifact |  |
| **NOT RUN** | `M2.inputs`: The reference scenario's inputs are identified: two separately assembled roots match the manifest (1,179 files) | `inputs.lebanon`: NOT RUN<br>`inputs.lebanon_repeat`: NOT RUN | #23 (docs/m2-reference.md) |  |
| **NOT RUN** | `M2.repeatable`: Two runs from the two roots give byte-identical recording, observations and metrics | `reference.repeatable`: NOT RUN | #23 | not demonstrated: a nondeterministic build was never run |
| **NOT RUN** | `M2.tacview_observations`: Tacview and the structured observations agree: identity and lifecycle frame by frame, positions within the recorder's threshold, losses three ways, keysite state equal to the metrics | `reference.observation_check`: NOT RUN | #23 | demonstrated: the checker fails each of five injected discrepancies (re-run offline by sensitivity.observation_check) |
| **NOT RUN** | `M2.unperturbed`: Observing does not change the campaign: the observed run's metrics equal the same build's unobserved run | `reference.observed_equals_unobserved`: NOT RUN | #23 (there, against the unobserved baseline) | not demonstrated: no perturbing observation was ever made |
| **NOT RUN** | `M2.reference_reproduced`: The reference run reproduces M2's retained Tacview recording byte for byte, and its metrics equal the Lebanon baseline with 39/39 expectations | `retained.recording`: NOT RUN<br>`reference.baseline`: NOT RUN | #23 (reference/lebanon-3h/) |  |
| **NOT RUN** | `M2.fidelity_characterised`: The characterised fidelity numbers are unchanged: session-clock drift (-29.63 s), deviations within threshold, attitude staleness, index-reuse replacements, losses and keysite changes (the observation checker's report) | `retained.check_report`: NOT RUN | #23 (accepted limits) |  |
| **EVIDENCE ONLY - not automatically checked** | `M2.plain_observations`: With observe_supply off, the observations are byte-identical to M2's retained file | - | #23; reproduced in #66 | the pack runs only the enriched settings |
| **NOT COVERED** | `M2.captures`: Keysite captures in Tacview and the observations | - | #23: not exercised (no capture in 3 h of Lebanon) |  |
| **NOT COVERED** | `M2.tacview_visual`: A visual review of the recording in Tacview | - | #23: generated and structurally checked, never visually reviewed |  |
| **NOT RUN** | `M3.resupply_loop`: A campaign SUPPLY assignment produces a continuous flight from supplier to receiver, with the pick-up and the drop-off in the same samples as the supply levels change; deliveries equal the metrics' resupplied counts | `reference.supply_chains`: NOT RUN | #27 (docs/m3-resupply-loop.md, #66) | demonstrated: removing any of four links breaks the chain (re-run offline by sensitivity.supply_chain_check) |
| **NOT RUN** | `M3.resupply_reproduced`: The retained loop is reproduced: 32 deliveries, all attributed, 31 complete chains, including Jester's (Il-76, Halat to Beirut) | `retained.chains_report`: NOT RUN<br>`retained.observations`: NOT RUN | #27 (reference/m3-resupply-loop/) |  |
| **NOT RUN** | `M3.feedback`: The campaign then uses the delivered stock: draws on it, other SUPPLY tasks pick up from it, it falls below the request threshold and is resupplied again | `retained.chains_report`: NOT RUN | #27 (docs/m3-resupply-loop.md) | the chain checker reports this ('afterwards') but does not assert it |
| **NOT RUN** | `M3.duplicate_crate`: Apparent original EECH defect, characterised: a drop-off to a group leaves the crate aboard, and a later task delivers it again (An-12 'Python') | `retained.chains_report`: NOT RUN | #27: recorded, not accepted as desired behaviour |  |
| **NOT RUN** | `M3.supply_loss`: When a SUPPLY aircraft is destroyed in flight, the loss, the survivors' response (or the group's end) and the group's next assignment are all observed | `reference.loss_chain`: NOT RUN | #27 (docs/m3-supply-loss.md, #67) | demonstrated: removing the death, the response or the next assignment fails the checker (re-run offline by sensitivity.loss_chain_check) |
| **NOT RUN** | `M3.supply_loss_reproduced`: The retained loss is reproduced: Mi-17 87568 of 'Wolfpack' destroyed at 1,584.66 s, the group reduced to one aircraft, re-tasked at 3,619.24 s, delivering at 4,013.67 s | `retained.loss_report`: NOT RUN | #27 (reference/m3-supply-loss/) |  |
| **NOT COVERED** | `M3.adaptive_compensation`: A campaign response aimed at the lost supply; a keysite-bound delivery loss, supplier or receiver loss, or a capture-driven failure | - | #27: explicitly not established |  |
| **EVIDENCE ONLY - not automatically checked** | `M3.chdir`: The engine's boot changes the process's working directory (a recorded host-integration hazard) | - | #27: recorded, not changed | campaign.lua works around it; nothing asserts it |
| **PASS - directly protected** | `M4.s3_detected`: The accepted checks reject the S3 mutant's retained output: Lebanon's ammo-resupply expectation and both exact baselines | `sensitivity.s3_lebanon_expectation`: PASS<br>`sensitivity.s3_exact`: PASS | #68 (docs/m4-s3-mutation.md) | rebuilding and rerunning the mutant is tools/mutation-windows.ps1, not part of the pack |

## Checks

| Check | Result | Detail |
|---|---|---|
| `evidence.artifacts` | PASS | 17 artifacts as recorded |
| `evidence.checkers` | PASS | 13 checkers and runners as recorded |
| `evidence.manifests` | PASS | lebanon 1179 files 121a789003b6, georgia 1174 files d373bc426246 |
| `evidence.reproduce.m2_check` | PASS | equal to reference/lebanon-3h/check.json |
| `evidence.reproduce.m3_check` | PASS | equal to reference/m3-resupply-loop/check.json |
| `evidence.reproduce.chains` | PASS | equal to reference/m3-resupply-loop/chains.json |
| `evidence.reproduce.loss` | PASS | equal to reference/m3-supply-loss/loss-chain.json |
| `sensitivity.observation_check` | PASS | 6 cases: unmodified   passes: as expected; moved        fails: as expected; undeclared   fails: as expected; no-death     fails: as expected; coalition    fails: as expected; not-gone     fails: as expected |
| `sensitivity.supply_chain_check` | PASS | 5 cases: unmodified     group 88235: assigned 1160.77, pick-up 1555.67, delivery 1904.58: complete; no-assignment  no longer complete: as expected; no-pickup      no longer complete: as expected; broken-flight  no longer complete: as expected; no-aircraft    no longer complete: as expected |
| `sensitivity.loss_chain_check` | PASS | 4 cases: unmodified     loss of 87568 at 1584.66: passes; no-loss        fails: as expected; no-response    fails: as expected; no-subsequent  fails: as expected |
| `sensitivity.s3_lebanon_expectation` | PASS | 'airbases resupplied with ammo' fails on the mutant's metrics, as in #68 |
| `sensitivity.s3_exact` | PASS | georgia_retail: OUTSIDE tolerance of the baseline (--exact: not identical) (11 metrics outside tolerance); lebanon_retail: OUTSIDE tolerance of the baseline (--exact: not identical) (13 metrics outside tolerance) |
| `blindspot.georgia_resupply` | PASS | confirmed: 30 of 30 hold on the S3 mutant |
| `inputs.georgia` | NOT RUN |  |
| `inputs.lebanon` | NOT RUN |  |
| `inputs.lebanon_repeat` | NOT RUN |  |
| `lifecycle.boot_twice` | NOT RUN |  |
| `lifecycle.bad_arguments` | NOT RUN |  |
| `lifecycle.missing_root` | NOT RUN |  |
| `lifecycle.missing_campaign` | NOT RUN |  |
| `lifecycle.no_reuse` | NOT RUN |  |
| `lifecycle.missing_map` | NOT RUN |  |
| `regress.georgia.expectations` | NOT RUN |  |
| `regress.georgia.exact` | NOT RUN |  |
| `regress.lebanon.expectations.mechanics` | NOT RUN |  |
| `regress.lebanon.expectations.resupply` | NOT RUN |  |
| `regress.lebanon.exact` | NOT RUN |  |
| `reference.observation_check` | NOT RUN |  |
| `reference.repeatable` | NOT RUN |  |
| `reference.baseline` | NOT RUN |  |
| `reference.observed_equals_unobserved` | NOT RUN |  |
| `reference.supply_chains` | NOT RUN |  |
| `reference.loss_chain` | NOT RUN |  |
| `retained.recording` | NOT RUN |  |
| `retained.observations` | NOT RUN |  |
| `retained.check_report` | NOT RUN |  |
| `retained.chains_report` | NOT RUN |  |
| `retained.loss_report` | NOT RUN |  |

## Blind spots

| Blind spot | What | Claims it limits | Check |
|---|---|---|---|
| `georgia-resupply` | Georgia's expectations do not assert resupply. Without S3 all 30 still hold; only Georgia's exact baseline caught it (#68). | `M1.georgia.mechanics`, `M1.georgia.war`, `M1.corrections.S3` | `blindspot.georgia_resupply`: PASS |
| `three-hours` | Every run is 3 simulated hours. Behaviour that emerges later (the N3 fault at 39,327 s, a campaign's conclusion, long-run supply balance) cannot be protected; the longer runs are reported only. | `M1.georgia.mechanics`, `M1.lebanon.mechanics`, `M1.corrections.W1`, `M1.corrections.N3`, `M1.corrections.N1_N2`, `M1.longer_runs` | documented |
| `unobservable-supply` | Crates, the task's record (its supplier, receiver and requested resource), request messages and group supply levels are not observable. Chains are attributed from position and supply changes; a drop-off to a group is invisible. The chain checker pairs pick-up and delivery by resource, so it cannot tell an ammo task that delivered fuel from a fuel task. | `M1.lebanon.resupply`, `M1.corrections.S3`, `M3.resupply_loop`, `M3.feedback`, `M3.duplicate_crate`, `M3.supply_loss` | documented |
| `adaptive-compensation` | M3's adverse evidence shows state damage and continued operation, not a response aimed at the lost supply. No keysite-bound loss, supplier or receiver loss, or capture-driven failure has been observed. | `M3.supply_loss`, `M3.supply_loss_reproduced`, `M3.adaptive_compensation` | documented |
| `index-reuse` | EECH reuses entity indices: same-type weapons can merge into one track (23 jumps and 12 launcher changes are lower bounds), and the metrics' launched counts undercount by the same mechanism. | `M2.tacview_observations`, `M2.fidelity_characterised` | documented |
| `tacview-limits` | Tacview attitude is written only with a position or heading change (pitch up to 5.1 degrees and roll up to 79.5 degrees stale); a keysite's coalition is fixed at declaration, and its state and supply are not in Tacview. | `M2.tacview_observations`, `M2.fidelity_characterised` | documented |
| `clock-projection-sampling` | Session time drifts -29.63 s over 3 h (about -4.6 s explained); Tacview's ReferenceTime is a fixed label; the map projection is an affine fit (1.1 km median residual, not re-verified); 1 s sampling misses shorter events and their order. | `M2.tacview_observations`, `M2.fidelity_characterised` | documented |
| `reported-only` | Some accepted behaviour rests on code review or on reported historical evidence without a retained artifact: most corrections, the longer runs, the M1 regen-disabled and lifecycle sensitivity checks, the rebuild check and the kernel's i686/Windows runs. | `M1.lebanon.mechanics`, `M1.corrections.S1`, `M1.corrections.portability`, `M1.corrections.load_time_data`, `M1.corrections.W1`, `M1.corrections.N1_N2`, `M1.corrections.H1`, `M1.corrections.U1`, `M1.corrections.F1`, `M1.corrections.compile_only`, `M1.kernel`, `M1.longer_runs` | documented |
| `retail-data` | The runtime tier needs local retail data (GOG Comanche vs Hokum, Steam Apache vs Havoc map3), which is not in the repository, so the complete pack is not self-contained and is not hosted-CI evidence. The retained tier runs without it. | `M2.inputs` | documented |
| `one-war` | One seed per scenario. An exact comparison detects any change, including a harmless one: anything that changes the random stream fights a different war (seed 2 moves 42 metrics beyond tolerance). A baseline difference alone does not say whether the change is a regression, a correction or expected variation. | `M1.georgia.war`, `M1.lebanon.war`, `M1.corrections.S2_removed`, `M2.repeatable`, `M2.reference_reproduced`, `M3.resupply_reproduced`, `M3.supply_loss_reproduced` | documented |
| `coupled-engine` | The campaign and the physical world both run inside eech_dc.dll. The pack cannot attribute a change to campaign policy or to the world, and says nothing about another host's world (M5). | `M2.tacview_observations`, `M3.resupply_loop` | documented |
| `captures` | No capture occurs in 3 h of Lebanon, so capture representation in Tacview and the observations is unchecked; Georgia's captures are seen only in its metrics. | `M2.captures` | documented |
| `plain-observations` | The pack runs the observations with observe_supply=1 only; the plain M2 settings' output is not re-run. | `M2.plain_observations` | documented |
| `tacview-visual` | No recording has been visually reviewed in Tacview; the pack compares bytes and structure. | `M2.reference_reproduced`, `M2.tacview_visual` | documented |
| `linux` | The Linux baselines are stale (before S3, N1-N3, H1, U1 and S2's removal) and not run; the repository CI is not evidence for the candidate. | - | documented |
| `lifecycle-unverified` | Frame-time poisoning, threading and reentrancy, and host callbacks are unverified or unsupported. | `M1.lifecycle.one_boot`, `M1.lifecycle.unverified` | documented |
| `build-identity` | A rebuild reproduces code and data but not file hashes (timestamps, symbol table). The pack identifies a build by its file hashes, so a rebuild of the accepted source is reported as a different build. | `M1.rebuild` | documented |

## Summary

- EVIDENCE ONLY - not automatically checked: 8
- NOT COVERED: 8
- NOT RUN: 28
- PASS - directly protected: 1

**REGRESSION PACK: INCOMPLETE (retained tier only)**
