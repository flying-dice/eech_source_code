# D2 sensing delivery review — EEDCS-43

**Current lead disposition:** source findings are resolved at runtime `a2a041fec`;
focused green evidence is checked and the frozen full pack is running. CI, full
regression/recording acceptance and normal merge remain pending. Earlier pending
statements below are historical; see the lead closure section for current gates.

Independent reviewer: the single Codex reviewer also used for EEDCS-42. One read-only
source, harness and comparator pass of `37989fa5f183abbffa122fb47535923a83f6bfa9`
against reviewed LOS predecessor `a42f1cfad39e5b29ce18bfd2d4ff6cf0f5e0cf12`.
No nested agents, builds or native scenarios were launched by the reviewer.

## Findings awaiting correction

1. **P1 — original EECH build inputs must invalidate reused targets.**
   `eech-campaign/crates/eech-engine-sys/build/main.rs:54` watches local build inputs,
   but stages original `modules` and `aphavoc/source` files at lines 83–91 and gives
   cc their staged copies at line 186. Original C/header edits alone need not rerun
   staging or compilation. Register dependencies on the original staged inputs,
   including headers and generated-texture input, beneath the resolved
   `EECH_SOURCE_ROOT`. Verify an incremental rebuild without cleaning; final
   acceptance also needs fresh shipping artifacts. Lead observed this failure:
   d588's reused fixture-1 retained the old C; fresh fixture-2 at 379 contains the
   correction and passes the strict trace.
2. **P2 — the host must be able to report generic physical nondetection.**
   `eech-campaign/crates/eech-dc/src/module.rs:409` requires each rejection to name
   one of four EECH predicates; `aphavoc/source/entity/mobile/mb_hostsense.c:76`
   further constrains reasons by scan. A host using its own physical model must
   currently fabricate an EECH-specific cause or fail the contract. Add a generic
   or unspecified nondetection result accepted across scan kinds, preserve the
   reference-host reasons as characterization diagnostics, and continue rejecting
   malformed reasons. Campaign target filtering, ranking and engagement stay core.

## Other source observations and limits

No additional correctness or numeric findings. The source preserves cumulative
floor mutation per eligible candidate, conditional draw/configuration reads,
range and sector quirks, normalized cosine ranking and campaign policy. Seeds
derived from the private entity heap and entity pointers do not cross the public
boundary. Composition passes the complete callback context; normal/fatal cleanup
clears sensing state and restores the FPU state.

The reviewer inspected root's complete strict comparison: 443 versus 443 events,
zero differences, including the tolerance recomputation counters. That was
evidence inspection, not an additional runtime run. Root's result is retained at
`../d2-evidence/independent-sensor-379-comparison.txt` relative to the worktree.

This review does not accept the task. Both findings need correction and a focused
follow-up by the same reviewer on those changes. Corrected-head builds, public
host-authority/refusal checks, shipping 900-second recording completion, final
full regression and recording gates, and accepted LOS predecessor delivery are
still required. No baseline or tolerance changes are authorized.

## Resolution status (implementation lane, after the review above)

The two findings above are kept as written. Corrections are in two focused commits on
`codex/d2-sensor-detections` (delta over the reviewed `37989fa5f`/`cee800898`);
the follow-up review of the same reviewer is for this delta only. Neither correction is accepted until that follow-up.

1. **P1 — corrected in `647558f8b`, verified, awaiting the reviewer's follow-up.**
   `build/main.rs` now registers `cargo:rerun-if-changed` for `modules/` and
   `aphavoc/source/` under the resolved `EECH_SOURCE_ROOT` (cargo scans a watched
   directory recursively, so headers, `textname.h` and added or removed files are
   covered). Evidence: `eech-campaign/reference/d2-sensor-detections/review-corrections/p1-incremental-check.log`.
   One never-cleaned cargo target and a scratch copy of the original tree. With the
   reviewed script, an edit of `mb_hostsense.c` does not rerun the build script and the
   staged copy stays stale (RED). With the corrected script, an edit of that `.c`, of its `.h`
   and of `modules/3d/textname.h` each rerun it and the staged file equals the edit; an unchanged
   build does not rerun it (GREEN). The source-watch change alone alters no runtime. The corrected
   source is built on fresh volumes (below), and the final acceptance still needs the root's
   fresh shipping artifacts.
2. **P2 — corrected in `5c00ab4fa`, verified at the Lua-to-DLL boundary, awaiting the reviewer's follow-up.**
   A generic reason `not_perceived` (`CAMPAIGN_SENSE_REASON_NOT_PERCEIVED`,
   `EECH_SENSE_REASON_NOT_PERCEIVED`, `SenseReason::NotPerceived`) is accepted for every
   scan kind. The reference host keeps `range`, `range_adjusted`, `altitude` and
   `tolerance` as characterisation diagnostics with their per-scan legality;
   unknown reasons are still refused (the `bad_reason` control). The sensor fixture trace
   records only the original's predicates, so it stops with a clear error on a generic
   verdict instead of relabelling it; the bit-exact reference path, its hashes and
   its expected counters are unchanged. No seed or entity index crosses the boundary.
   The callback, campaign-half and reference-kernel comments now describe the positions,
   radar altitude, configuration facts and unit figures as transitional mirrors that
   feed the reference model, not a physical solver the campaign owns. The public-path evidence
   (`mode=not_perceived`, `mode=own_model`) is in
   `eech-campaign/reference/d2-sensor-detections/review-corrections/`.

Not re-run, by decision: the unchanged 600 s witness, the retained reference/blocked/far/malformed
records, and the 900 s shipping control of `379` (reference-host behaviour is unchanged; the
new branch is covered by the focused control). The final full regression pack and the fresh
shipping build remain the root's.

## Lead disposition after focused independent follow-up at5c00ab4fa

The same independent reviewer checked only the new379-to5c00 delta. P1 is resolved;
P2 is resolved for shipping. The reference arithmetic remains unchanged, and the
new unknown-reason handling still fails closed. New focused finding, **P2**:
`mb_hostsense.c:301-302` deliberately calls `debug_fatal` for a valid generic
`NOT_PERCEIVED` verdict, reached for every fixture rejection at417. This makes
enabling observational `sensor-fixture` poison an engine that shipping accepts.

Lead agrees this needs correction. The fixture must observe a valid generic
nondetection distinctly, without relabelling it as range/altitude/tolerance or
aborting a valid provider. Record it separately (or skip reference-only folding
for that verdict while counting it separately), preserving all existing reference
hashes and report events when no generic verdict occurs. Keep the old443-event
oracle unchanged; do not add always-zero fields/rows to it or change expected
data. A focused short public Lua->DLL fixture case should establish that the
valid generic result is observational rather than fatal. Reuse completed shipping
generic controls and unchanged379 reference runs; no extra whole war is needed.
The same reviewer will inspect only this new correction and evidence. Generic
shipping runs currently in progress should finish and be preserved.

## Lead closure of source and focused evidence gates

The same reviewer checked5c00 against379 (P1 resolved, P2 shipping resolved), then
only the newa2a041fec fixture/driver delta against5c00. The fixture finding is
resolved in source, with no new change-specific finding. It records generic
nondetection separately, clears pending candidate/tolerance state, isolates/reset
counts correctly and leaves reference hashes/events unchanged when no generic
verdict occurs. Disabled-feature macros and explicit non-oracle driver modes are
present. No golden output changed; no additional full source pass was requested.

Root inspected actual native/wrapper exits0 for the new300s shipping generic and
own-model controls, all three short fixture cases and the300s generic fixture.
Final generic counts6245/312160/520/6021 match the shipping control across all4
scan kinds; the fixture no longer aborts. Root independently compared all83 raw
events at the original30s reference probe/run-at-probe checkpoint: exact bytes,
15544bytes, SHA-2564466c77606ae6bba49fe8307e94b637a174ef8700741af316933730a2c9fc71f.
Both source traces are complete, and the originals are unmodified. This short
control checks the new fixture path; it is not a new600s run or full pack.
Recorded actual fmt/default/all-fixtures/old-guard+sensor/sensor-only Clippy exits
are0; default workspace tests at5c00 also exited0. The reference-model path is
unchanged, so its earlier600s/900s evidence is reused explicitly.

Root's fresh ordinarya2a041fec build exited0, clean, with all26 manifest files
verified. The immutable final full pack is running; CI, its40-check result and
complete raw comparisons, normal merge and delivery records remain required.
Accepted LOS predecessor3ce17432e is delivered (Jira42Done, ledgerv34/five reports).
This source/evidence closure is not overall EEDCS-43 or D2 acceptance.

## Resolution status of the follow-up finding (implementation lane)

The follow-up **P2** above (the fixture aborting on a valid generic verdict) is corrected in
`a2a041fec`, awaiting the same reviewer's check of only that delta. Not accepted until then.

* **Red.** The `5c00ab4fa` fixture build run with the new driver (`sensor-characterization.lua sense=not_perceived`, 60 s) stops
  after 6 s, native exit 1: `debug_fatal: host sense: the sensor fixture trace has no verdict for host reason 5`.
* **Correction.** A host's `NOT_PERCEIVED` is observed apart (`eech_sensor_not_perceived`): a per-scan counter and a
  `<scan>/host_not_perceived` sample, reported in `sensor-host-verdicts` rows that appear **only** when a host answered it. It is
  not relabelled as range, altitude or tolerance, folds nothing into the physical or decision hash, and the reference-only hooks
  of that exchange (candidate, target range, floor, tolerance, emitter and stealth) are skipped. A reference run therefore hashes,
  counts and reports as before; no always-zero row or field was added and the 443-event oracle and its expected data are unchanged.
* **Green** (fixture build `a2a041fec`, public Lua to DLL path, fresh roots, one native process at a time): `sense=not_perceived`
  60 s and 300 s and `sense=own_model` 60 s exit 0. At 300 s the generic verdicts are air-to-air 6,245, air-to-surface 312,160,
  surface-to-air 520, surface-to-surface 6,021 (equal to the shipping run's consultations) with every table verdict 0. A reference
  run (45 s, probe at 30 s) has no generic row or sample and its 83 probe and run-at-probe events at 30 s equal the retained original's byte
  for byte (an ad-hoc text comparison of those events, not a run of the strict comparator). The completed shipping generic controls and the
  `379` reference runs were reused; no whole war was run.
* **Shipping unchanged by it:** `ac_tgt.o` and `vh_tgt.o` byte-identical to `5c00ab4fa`'s; `mb_hostsense.o` differs only in four
  `debug_fatal` line-number immediates. fmt 0 and clippy 0 for default, all fixtures, old-guard plus sensor, and sensor alone.

Evidence: `eech-campaign/reference/d2-sensor-detections/review-corrections/` (README). The final shipping build, the full regression pack
and the reviewer's follow-up remain the lead's.
