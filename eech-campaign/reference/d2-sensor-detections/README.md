# EEDCS-43: evidence for the characterisation of the original AI sensor-detection scans (26 Oct 2026)

The account, results and limits are in [`docs/m5-sensor-characterization.md`](../../docs/m5-sensor-characterization.md). This directory holds the
retained, safe-to-publish artifacts. No retail asset, installation root or binary is in Git; the roots and binaries named here are private copies under
`R:/d2-evidence/sensor-characterization-baseline-20261026/` (R: is a subst of the parent workspace, to avoid the runner's unquoted-space argv).

## Identities

| | |
|---|---|
| Base | `a7934aa7d1bcffc2ea5057bb3f1b0af332606c61` (D1 integration). The scans are unchanged; the only source additions are test-only hooks, empty macros without `EECH_SENSOR_FIXTURE` |
| Fixture A | commit `60f6a4876e02d4ce4e6d32436930b69d82214260`, `eech_dc.dll` `d15e48885171e7e0ce4e384252d469c933b2dfb6bf4ca7b7cec860ff282730c5`, `eech-world.exe` `3ef08b2f...7f27` (`builds/fixture-60f6a4876-BUILD-INFO.txt`; 0 uncommitted build inputs) |
| Fixture B (used for the results) | commit `e0d58d818661620ddfa7e386118551de8c191da0`, `eech_dc.dll` `352f8f8563dc1f59266d06ec804de1420749e1dd5651c6d1091d686d99b693e8`, `eech-world.exe` `3ef08b2f...7f27`, `lua.dll` `fdd3325e...3a05` (`builds/fixture-e0d58d818-BUILD-INFO.txt`; 0 uncommitted). B adds the empty-rectangle counter, scan-range extremes and player-aircraft count; nothing that enters a hash |
| Toolchain | repository `tools/Dockerfile` image in WSL2 distro `EECH-D2-Build`; rustc/cargo 1.99.1, MinGW gcc 12-posix `12.2.0-14+deb12u1+25.2+b1`; wrapper `tools/build-windows-sensor-fixture.sh` in cargo volume `eech-win-target-d2-sensor-fixture-1` (not a shipping or LOS volume) |
| Inputs | `lebanon-probe2`, `lebanon-noprobe2` (and `lebanon-probe`, used by fixture A) each a fresh copy of `eech-regression-20261025/roots/lebanon`, IDENTICAL to `reference/lebanon-3h/inputs.json` (1,179 files, `121a7890...`) before the runs (`checks/roots-manifest-before.log`) |
| Baseline binary | the supplied `eech-regression-20261025/bin` (`eech_dc.dll` `e0071743...92e2`, `checks/baseline-bin-sha.log`) for the A/B control |
| Run | `eech-world.exe lua/sensor-characterization.lua root=<copy> scenario=lebanon_retail seconds=600 frame_ms=100 probe_at=30,300,590 out=<trace.jsonl>` (campaign seed 1 inside the driver), through `tools/sensor-characterization-windows.ps1`; one native process at a time while the lead's eight pack processes ran |

## Files

* `traces/lebanon-probe.jsonl`, `lebanon-noprobe.jsonl`: fixture B, 600 s, with probes and without. `v1-60f6a4876-lebanon-probe.jsonl`: the first run on fixture A (same run hashes; its report lacks the fields B added).
  `*.stderr-filtered.txt` keep the driver's final line (the engine prints a thousand lines about missing artwork, dropped).
* `builds/`: BUILD-INFO of both fixtures. `checks/`: fmt and clippy of the committed tree (`cargo fmt --all --check` and `cargo clippy --locked --all-targets -- -D warnings`, default and
  `--features eech-dc/sensor-fixture`, each cargo exit captured; the image has no clippy so the component is added in the disposable container), the feature-off preprocessing equivalence, input manifests. (`preprocess-equivalence.log` ran at `60f6a4876`; the hook text in `ac_tgt.c` and `vh_tgt.c` is unchanged since — `git diff 60f6a4876 e0d58d818 -- aphavoc`
  is empty. Its "dirty files: 1" is the then-CRLF working copy of the fixture C file, content equal to the index, fixed before the builds.)
* `controls/`: `ab-fixture-vs-supplied-baseline-900s.json` (the whole-campaign comparison of the hooked build against the supplied baseline binary, with the verified-versus-inferred configuration comparison) and
  `ab-fixture-observation-check.json`; `comparator-controls-result.json` (actual Python exits of `tools/sensor-trace-compare-controls.py`, comparator SHA-256 `d5027336...1165`), its inputs in `comparator-inputs/` (the lead's two
  real red controls `empty-*.jsonl`, `incomplete-probe-*.jsonl` with their reproduction records, and `derived/` copies: changed value, truncated, malformed) and
  `lead-independent-original-characterization-check.json` (the lead's byte/field check of the complete reports; not a source review).

## Not done here

No extraction, no host boundary, no shipping-build change, no full pack, no Georgia run, no player-controlled target (see the document's unexercised edges).
