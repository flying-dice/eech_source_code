# Original sound initialization and sample-data characterization

Native and Lua source: `7574caa2ca87a2a44608a234fd1dd73136b8e4c0`. This is original-behavior observation on
the already hosted ground/environment baseline. Metadata/backend behavior has
**not moved**. Task51 and D3 remain open; D4/D5 have not started.

The recovered probe and observation-off control both finish with native and
controller exit0. Four pre/post retail input checks pass; all62 native payloads
remain unchanged. Both canonical builds, Linux default/observer tests and strict
checks, all nine exact-source CI jobs and the actual Windows exclusive-output
probe pass. Independent SOURCE review has no findings; independent ACTUAL review
of this candidate remains required.

## Measured facts

- The catalogue has **2122 slots**, of which **2069 are named** and have nonzero
  rate/original-rate. No slot has nonzero size. These are distinct counts.
- Filled, initialized and report-time catalogue digest is `6605372942e1ea2c`.
- At initialization no-sound0/dedicated0; backend initialization was attempted.
  `DirectSoundCreate` ran once and returned `80004005`; the other three observed
  DirectSound entries were never called. Enabled/initialized remain0.
- The header had three failed open attempts; data had zero attempts. Effects
  initialization reports open_false1/open_true0. Header count/matches/misses0.
  These observations do not establish missing retail assets.
- The side-dependent loader entry ran once (side3, enabled0, dedicated1), while
  the initialization load flag is0. Entry calls and successful loading differ.
- Constructor calls5222, looping3548, sample reads1674, distinct keys64; all64
  retained FIRST-read metadata rows have rate22050, size0, original-rate22050.
  Later reads are covered by the per-read digest, not individual value rows. No out-of-domain key,
  dropped/orphaned/saturated record. Read digest `17e67b8187903754` matches every
  retained old constructor total/digest. The read-only report preserves the seed
  and next entity/local indices; raw values remain private.

## Complete comparisons and retained failure

The first read-only assertion report was generated in a combined shell command
that did not separately retain the checker's exit. One checker-only verification
on the unchanged retained files completed with explicitly recorded child exit0
(see `auditor-execution.json`); no campaign, input or native output was repeated.
The earlier assertion report remains retained with its original scope.

All **5,191,384 bytes / 17,283 rows** of the observation-off control match the
retained original822 trace, including line endings, SHA256
`48307ff0286cff2219a89d7d8b807a3624fa1cb673c440c4ed70d55a7b40d81d`.
The complete returned native report (7582 bytes) equals the join of every JSON
raw row. Its private sidecar is retained. New wrapper JSON includes an output
path; no whole-wrapper equality to the unavailable old report is claimed.

The first native458 / Lua50a probe remains FAILED1: its32-key collector omitted
1210 reads and the witness refused it. Its full returned in-memory report was
not saved and remains UNAVAILABLE; boot plus the real failure row are retained.
The separately successful458 observation-off control stays successful. Recovery
does not recreate the lost report. The old failed capture, meaningful reds,
observer lint failure, wrong-feature command failure and path-placement fix are
retained with their original scopes. Earlier firstsoundd1 raw gap also remains.

## Scope, reproduction and privacy

Run the repo's `lua/env-sound-init.lua` with this observer native, a fresh
manifest-verified Lebanon root, seed1 (the driver fixes it), seconds120 and
frame_ms100. Verify build/source/payloads and pre/post inputs, preserve
`<out>.case15.raw`, then run the unchanged `env-characterization.lua` old-stream
control on another fresh root. The retained host-specific read-only audit script
has the exact comparisons and receipt identities; it assumes the recorded R:
evidence layout. It neither changes a baseline nor runs a campaign.

This folder contains projections/receipts and code only: no retail files,
binaries, raw reports, recordings, allocation indices or world vectors. The
native-derived `derived` row is explicitly a driver's join, not a measurement.
No new visual Tacview inspection, shipping40/all8, positive device/handle,
reinitialization, loader data reads or other compiled-reader liveness is claimed.
The original metadata/assets, speech availability, winding-rotor duration,
per-family/range/device/destroy/network/pitch/amplitude and physical-time duties
remain D3 work. Campaign decisions, calendar, timers and scheduling remain Core.

## Published receipt text

Original receipts were copied byte for byte before staging. Git stores some
JSON receipt text with LF line endings. `publication-text-identities.json`
records both original-file and committed-text hashes and each exact transform.
This concerns receipt serialization only: all measured native/raw/recording
files and accepted baselines remain unchanged. The canonical source review is
LF already and its published bytes remain identical.
