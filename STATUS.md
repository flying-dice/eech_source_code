# D2 sensor detection — EEDCS-43 — 26 October 2026

Branch `codex/d2-sensor-detections`, from reverified integration
`a7934aa7d1bcffc2ea5057bb3f1b0af332606c61`. Draft delivery PR:
https://github.com/flying-dice/eech_source_code/pull/102.
Authority: live M5 goal 11534337 v3 and migration ledger 14548994 (v32 at start).
D1 remains accepted; D2 is active; D3 has not started.

## Stage: original-behaviour characterisation; evidence-tool correction; no host extraction

Nothing moved. The four AI scan functions (`ac_tgt.c` air-to-air and air-to-surface, `vh_tgt.c`
surface-to-air and surface-to-surface) are the original text; the only source additions are
test-only hooks that are empty macros unless the cargo feature `sensor-fixture` is on.

* Source: observer `60f6a4876e02d4ce4e6d32436930b69d82214260`; counters added in
  `e0d58d818661620ddfa7e386118551de8c191da0` (hooks unchanged, nothing that enters a hash).
  Fixture DLL at `e0d58d818`: `352f8f8563dc1f59266d06ec804de1420749e1dd5651c6d1091d686d99b693e8`;
  both fixture builds exit 0 from a clean tree (0 uncommitted inputs); fmt and clippy (default and
  `sensor-fixture`) exit 0 at both commits.
* Lebanon 600 s through the public Lua -> DLL path: physical hash `d60c462829c9714e`, decision hash
  `2611b04d99f5c689`, identical on two builds, two roots, with and without the read-only probe
  (75 run-wide events, 0 differences). Per-function tables, probes and samples are in
  `eech-campaign/docs/m5-sensor-characterization.md` and `eech-campaign/reference/d2-sensor-detections/`.
* Feature off preprocesses to the original text (`checks/preprocess-equivalence.log`).
* Hooks on, 900 s whole-campaign Lebanon run: Tacview recording, observations and metrics are
  byte-identical to the supplied-baseline-binary run the lead recorded (that side not re-run here);
  `observation-check.py` PASS. Not a visual Tacview inspection; not the 3 h pack.

## Evidence-tool correction (review feedback)

`tools/sensor-trace-compare.py` at `e0d58d818` was NOT acceptable: it exited 0 for empty files and for probe
snapshots with no final run report (two real red controls, retained in
`reference/d2-sensor-detections/controls/comparator-inputs/`). It now fails closed: exit 2 with a reason for a
missing, empty, malformed or incomplete file (the terminal `sensor-samples` event of scope `run` is required,
whatever scopes are selected and with `--skip-samples`) and for a selection that matches no event; exit 1 for a
difference; exit 0 only for complete files with a non-empty, equal selection. Actual Python exits are in
`controls/comparator-controls-result.json`: both red controls exit 2; the retained complete v2 pair `--scope run`
exits 0 (75 events, 0 differences); a complete probe trace against itself exits 0 (`--scope probe`, 182 events);
a changed-value copy exits 1; truncated, malformed, missing and empty-selection cases exit 2. No traces or
baselines were altered, no rebuild or campaign was rerun. The comparison of the original characterisation is not
claimed from zero or partial reports.

The 900 s control's configuration is now stated as verified versus inferred (docs section "Non-perturbation"):
verified from retained records that both runs used the same argument list (`hours=0.25 record_every=10
checkpoint_every=3600 observe_every=60 observe_supply=0`; seed 1 and 100 ms by default for both) and
byte-identical `campaign.lua`/`metrics.lua`/`observations.lua`; inferred that the original root was an
unmodified Lebanon copy. Identical bytes are the non-perturbation result for this configuration only.

## Boundary discovered (measured, not designed)

Physical facts per candidate: planar squared range (aircraft scans) or approximate 3-D range
(vehicle scans); target radar altitude against floor and ceiling (surface-to-air only); the
target's radar-stealth factor (a function of helicopter configuration); emitter and jammer state
(only for player-controlled targets); line of sight (already host-owned). Everything else in the
loops — side, airborne, type, group, best-group, alive, self filters, sector enumeration, ranking
and engagement — stays in EECH_DC. Two properties constrain extraction: the surface-to-air scan
floor is mutated cumulatively across candidates (288 of 2,444 evaluations differ from an
independent per-target floor; 606 calls evaluated it more than once), and the random tolerance
is a fixed per-source draw derived from a heap slot.

## Corrections to the task text, found in source and confirmed by measurement

* `cos_angle` is **not** a raw unnormalised dot product: `get_3d_vector_dot_product` normalises both
  vectors, so it is a true cosine (22 values in the run, all within [-1, 1]).
* The aircraft scans square the range before the sector bounds as stated: the rectangle is the whole map
  (1,676 of 1,676 air-to-surface calls), and for scan ranges above about 46.3 km (80 km observed, 179 of
  1,726 air-to-air calls) the squared range overflows `int` and the rectangle is empty, so the source
  finds nothing. Characterised, not fixed.

## Preconditions for the extraction

* Keep the required LOS entry wired: after EEDCS-42 the surface-to-air scan's LOS is the host-owned
  `campaign_los_query` dialogue (required `line_of_sight` callback, no fallback); this baseline predates it.
* The tolerance seed is a private heap index (`EN - entities`); never expose it or a seed derived from it publicly.

## Unexercised edges

Player-controlled targets (difficulty factor, emitter-off adjustments, all-criteria LOS form), altitude
above the ceiling, Georgia and other seeds, durations beyond 900 s. No player aircraft exists in the
headless world (0 of 265 aircraft entities at every probe).

## Handover

One Sonnet 5.5 High CLI lane, local context `67f40663-046f-435e-be4b-902073b9a69d`, worked here.
The lead manages publication, independent review, final full pack, normal merge and Jira/ledger.
The existing logged-in GitHub identity is used as directed; no authentication or credential
changes and no fabricated GitHub self-approval.

Predecessor EEDCS-42 / PR101 is source-complete and frozen in full validation; its source, binaries,
roots, logs and processes were not touched. This branch will incorporate accepted LOS integration
before moving sensor behaviour. Next bounded action: after that integration, choose the first
physical predicate to extract (range, altitude envelope, stealth, emitter/jammer), keep campaign
filters, sequence-dependent floor handling and ranking in EECH_DC, and compare against these
traces with `tools/sensor-trace-compare.py`. No accepted baseline or tolerance was changed.
