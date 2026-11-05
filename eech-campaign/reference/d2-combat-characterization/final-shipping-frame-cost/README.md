# Final ordinary-shipping native frame cost and 900-second control

Runtime: clean immutable64cf0f1c5551a2e59309a92917c32e6189d8ae60, ordinary
Windows shipping profile. Original launcher8557 and native17720 exited0;
native completion was2026-10-28T16:48:18.3018545Z. All9,000 raw samples and15
600-frame windows were recomputed by the retained strict auditor.

| Instrumented elapsed engine frame statistic | Milliseconds |
|---|---:|
| Mean | 260.2627778 |
| Maximum | 564 |
| Nearest-rank p50 | 254 |
| Nearest-rank p95 | 370 |
| Nearest-rank p99 | 429 |

`frame-cost.jsonl` is the exact556,377-byte timing-only raw file,
SHA256`447e0581deb484739e273108eebba7526d2d8676bf9dfac2e13fd2d97d971848`.
The public raw data permits independent recomputation. The original unrounded
values and summary formatting are preserved, not rewritten to the rounded table.

All three COMPLETE control files match retained ab-base exactly: Tacview11,363,845
bytes, observations6,629,273, metrics11,777. All1,179 post-run retail input files
and all42 exports/BUILD-INFO still match; frozen source remains clean. Original
ab-base root provenance remains INFERRED as previously declared. Private full
recordings/assets/binaries remain external; exact proof is retained here.

Independent runtime review a3a2ee76 is closed without findings. Generator/auditor
review803f6eb7 is reused. The probe is generated from the actual64cf shipping
campaign, whose remaining script and companion bytes are preserved; its Lua DLL
matches the retained f470 calibration by exact SHAaae318d5, so that calibration
is reused with its original provenance. BUILD-INFO stays64cf. Later63ea/aa19
changes affect only integration-test assertions and evidence, not product inputs.

This is elapsed `engine:frame` plus nested callbacks under recorded load. Four
full-pack campaigns ran concurrently; local targeted Rust checks and read-only
artifact checks also overlapped. Samples are serialized outside the timed span
but can affect later load. Objects/recording/metrics/outerloop and DCS scheduling
are excluded. The timer is quantized; neither a causal comparison with the
Task45-parent probe nor a DCS scheduling-budget pass is claimed. The exact DCS
follow-up is in `final-frame-cost-scope-and-followup.md`.

At publication, the full unchanged40-check pack38810/runner23828 remains ACTIVE.
This completed900s control does not prove40/full8/post3 or whole-D2 acceptance.
Allnine CI checks passed at aa19; subsequent evidence/publication commits still
need current-head CI and final review before normal merge/Task46/Epic18 closure.
No baseline/tolerance/scenario changes; noD3.
