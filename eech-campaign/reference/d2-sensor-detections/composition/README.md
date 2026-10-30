# EEDCS-43 composition: the original sensor observer across the host-owned line of sight (26 Oct 2026)

**Development stacking, not accepted integration.** PR101 / EEDCS-42 is not merged; its supplied-binary baseline pack and final-source candidate pack are still running (the lead's).
The reviewed frozen LOS head `a42f1cfad39e5b29ce18bfd2d4ff6cf0f5e0cf12` (runtime `61e856645`, later commits docs/evidence only) was merged into the sensor branch (`f22269490`) as
`1fc0cdc621fc0e643b1b78373c2fa230f8ef20c3`. EEDCS-42 delivery must precede EEDCS-43. No sensor physical behaviour moved: every sensor formula, filter, ranking, order and random-input use is the original text.

## What the composition changed

* Ten conflicts resolved keeping both sides: `los-fixture` and `sensor-fixture` features/defines/FFI/entries side by side; `eech_engine.c` edited byte-safely (one NUL byte kept; merged file = LOS parent plus the 61-line sensor block, nothing else differs from the LOS parent).
* **Entry wiring.** In the LOS code every host-using engine entry is `ENTER (..., host)`, which installs the `line_of_sight` context (`campaign_los_set_host`) and `LEAVE` clears it; an entry with no host leaves the surface-to-air scan with no provider (fatal). The sensor probe runs those scans, so
  `eech_engine_sensor_probe (text, capacity, host)` now enters with the host and refuses (bad argument) a missing host or a missing callback, exactly the five-callback check of a frame; the Rust `Engine::sensor_probe` builds the temporary host with `with_host` and
  the Lua method `sensor_probe` clones all five callbacks, as `los_probe` does. `sensor_trace`/`sensor_trace_reset` run no scan and keep the host-less entry. The driver `sensor-characterization.lua` boots with `line_of_sight = host.line_of_sight`
  (the boot refuses a missing one: `eech: missing host line_of_sight callback`). No legacy fallback was added.
* A rustfmt-only follow-up (`544debee3`, two line-break changes in the new probe wiring) after `cargo fmt --all --check` flagged them.

## Identities

| | |
|---|---|
| Witnessed source | `1fc0cdc621fc0e643b1b78373c2fa230f8ef20c3`, clean (0 uncommitted build inputs; a first build at 8 "dirty" files was CRLF working copies of retained evidence files, content equal to the index, and is superseded: its log was not kept in Git) |
| Binary | `eech_dc.dll` `917fe0f290ad30a9effeb93257725bed040a8560d8afda99d7fb6beff04e7e5c`, `eech-world.exe` `6c8ab7dce24c4bc1ad7494eefb85532fcd9f9afb13e40bbb2e7b0d1964f6973d`, `lua.dll` `fdd3325e...3a05` (`composed-1fc0cdc62-BUILD-INFO.txt`; FIXTURE BUILD `eech-dc/sensor-fixture`, unchanged `tools/build-windows.sh` recipe and Dockerfile through `tools/build-windows-sensor-fixture.sh`, rustc/cargo 1.99.1, MinGW gcc 12-posix, cargo volume `eech-win-target-d2-sensor-fixture-1`, exit 0) |
| rustfmt follow-up `544debee3` | rebuilt clean (`build-544debee3.log`, exit 0, 0 dirty) with a different `eech_dc.dll` hash `d8c36650...72ab`; it was **not** run (the hash differs although the change is line breaks only, presumably through debug line information, not separately demonstrated). The witness is of `1fc0cdc62` |
| Input | fresh private copy of `eech-regression-20261025/roots/lebanon`, manifest IDENTICAL to `reference/lebanon-3h/inputs.json` before and after the run (`composed-root-manifest-*.log`) |
| Run | `tools/sensor-characterization-windows.ps1 -Bin <composed> -Root <copy> -Out <dir> -Scenario lebanon_retail -Seconds 600 -FrameMs 100 -ProbeAt 30,300,590` (seed 1 inside the driver); one native process; native exit 0 after 742 s while the lead's packs ran |

## Results

* `composed-lebanon-probe.jsonl` has the **same SHA-256** as the retained original `traces/lebanon-probe.jsonl` (`dec1daf0...f753`, 451 lines): physical `d60c462829c9714e`, decision `2611b04d99f5c689`, every counter, sample and probe.
* Strict corrected comparator (`strict-comparison-original-vs-composed.txt`, actual Python exits, comparator `d5027336...1165`): `--scope run` 75 events, `--scope probe` 182, `--scope run-at-probe` 186, default 257, all three 443; 0 differences, exit 0 each.
  The run-wide report before and after each probe was equal (`run_trace_untouched_by_probe` true at 30, 300 and 590 s). No tolerance, scenario or oracle was changed or re-recorded.
* fmt and clippy (cargo's own exits): `fmt-clippy-1fc0cdc62.log` shows `FMT_EXIT=1` (the two line-break diffs it prints) and clippy 0 for default, all fixtures (`dependent-fixture,loss-fixture,los-fixture,sensor-fixture` on eech-engine, eech-dc, eech-world),
  `dependent-fixture-old-guard + sensor-fixture`, and `sensor-fixture` alone; `fmt-clippy-544debee3.log` (the rustfmt follow-up): fmt 0 and the same four clippy runs 0, `DOCKER_EXIT=0`. `check-composed.sh` is the script.

## Limits

One scenario and seed, 600 s; the witness validates the composed entry wiring across the LOS provider and the sensor oracle under it, not the LOS extraction (EEDCS-42's own evidence), the 900 s whole-campaign control (reused from the earlier stage, run on the pre-LOS source),
nor the full pack. `cargo test` was not run. No new 900 s or no-probe case was run. Nothing is accepted.
