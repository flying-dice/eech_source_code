# EEDCS-47 terrain surface characterization (original solver)

Base `6bbaa8197a4843181d03cd355d562af998c75321`, branch `codex/d3-terrain-services`, 29 October 2026.
Census and group map: [`docs/handover/D3-SCOPE-AUDIT.md`](../../../docs/handover/D3-SCOPE-AUDIT.md).

**Status: fixture source written; not compiled, not built, not run. No retained baseline exists yet.**
This directory holds no results. Sections marked *source finding* come from reading code; *planned*
items are not evidence. Nothing in the shipping module's behaviour was moved or changed.

## What is prepared

A `terrain-fixture` cargo feature (test-only, same pattern as `los-fixture`/`sensor-fixture`; never
enabled in the shipping build) that observes the **original, unmoved** terrain solver inside the real
campaign DLL, run through the public path (host Lua `require ("eech_dc")` -> native module):

| Piece | File | Role |
|---|---|---|
| Observer hooks | `modules/3d/terrain/terrelev.c`, `terrelev.h` | Under `EECH_TERRAIN_FIXTURE` only: the query macros pass `__FILE__`/`__LINE__` (the existing `DEBUG_TERRAIN` form, now keyed on `TERRAIN_CALL_SITES`), and each return calls `eech_terrain_record`. With the feature off the solver and its control logic are unchanged by inspection only (no preprocess, compile or runtime check yet); the added lines shift diagnostic source locations such as the off-map `ASSERT` (`__LINE__` 1268 -> 1281). No numerical drift is alleged. |
| Trace + sweep | `crates/eech-engine-sys/csrc/eech_terrain_fixture.c` | Run-wide trace, per-call-site table, deterministic probe sweep, text report. |
| Engine entries | `eech_engine.c/.h`, `eech-engine-sys` (`lib.rs`, `build/main.rs`, `Cargo.toml`), `eech-engine`, `eech-dc` | `terrain_trace`, `terrain_trace_reset`, `terrain_probe (seed, count, emit)` Lua methods. |
| Driver | `crates/eech-world/lua/terrain-characterization.lua` | Boots Lebanon or Georgia, reports boot-phase queries, runs frames, takes probes, asserts the probe leaves the run-wide trace unchanged. |
| Build wrapper | `tools/build-windows-terrain-fixture.sh` | Single-argument adaptation of `build-windows.sh`, as for the other fixtures. |

### Why this exercises real collaborating components

The queries counted are the ones the campaign's own code makes (`hc_move`, `fw_move`, `rv_move`, `ps_move`,
LOS terrain phases, weapon steps, targeting, the boot-time geography in `popread`/`faction`/`taskgen`)
against the real map loader and the real solver, in the real update order. Nothing is mocked and no campaign
decision is controlled. The only controlled input is the sweep's point set (a private xorshift, never the game's
random stream), which asks the *same solver* about points it would otherwise not meet in a run. The sweep
saves and restores the shared find-fast record (`eech_terrain_system_point_data`) so the campaign's own hint is
left as found, and keeps its own trace.

### What the report contains (all `key=value` lines)

- **Run-wide** (`scope=boot`, then `scope=run`): order-dependent FNV-1a hash over every query's x and z bits, the
  hint/fast/found/clamped/Cuba flags, the returned elevation, `last_elevation`, the normal and the type; counts of
  shared-record (`hint_null`) and private-record queries, hints tried (`fast_attempted`) and hit, full sector scans (`slow_scans`, whether or not a face was found), the
  queries that found no face (`not_found`, a subset of those scans), Cuba-area returns, clamped queries, and `partition_ok` (1 when `fast_hit + slow_scans + cuba == calls`); a surface-type histogram; and one row per call site (file, line, calls,
  shared-record calls, fast hits, not found). The boot report precedes the trace reset, so boot-time geography
  is counted too.
- **Probe**: six fixed points (four corners inset 1 cm, the centre, one sector-corner point) then `count` random
  points, one in eight snapped to a multiple or half-multiple of the sector side +-1 mm. Per point the reference is a
  fresh zeroed record; the same point is then asked (a) again on that record, (b) on a private record primed 5 cm to
  500 m away, (c) on the shared record primed the same way, (d) through `get_3d_terrain_elevation`,
  `get_3d_terrain_elevation_and_face_normal` and `get_3d_terrain_face_normal`. Reported: `sample` and `primed`
  hashes, mismatch counts for each of (a) to (d) against the reference, `result_ne_last` (Cuba-style
  difference between return value and `last_elevation`), the type, class and ground-vehicle suitability of every
  type met, the first `emit` samples as raw float bit patterns, and `shared_record_restored`.

### Limits of the fixture

- It counts and compares; it does not say which answer is *right*. A mismatch between a hinted and a fresh answer is
  reported as original behaviour, never corrected.
- Per-site attribution uses the macro call site. Queries through the wrappers in `terrelev.c` attribute to `terrelev.c`.
- Off-map input is not probed (the original asserts). Cuba flaw behaviour is only observed if the Cuba map is run.
- A fixture build is a different binary from the shipping module; its run hash is not comparable to a shipping
  Tacview byte file. Step R5 below checks that the fixture is observational by running the accepted scenario.

## Planned run matrix (for the Root; none executed)

Build, from a clean checkout of the commit that carries this directory, in the Git Bash used for the other fixtures
with Docker Desktop (Linux containers) running, from `eech-campaign/`:

    sh tools/build-windows-terrain-fixture.sh target/terrain-fixture/baseline-20261029

Then, with `root=` an installation root from `R:/d3-evidence/terrain-original-roots/{lebanon,georgia}` (private,
verified by the Root: 1179 and 1174 files) and outputs under `R:/d3-evidence/`:

| Run | Arguments | Purpose |
|---|---|---|
| R1 | `scenario=lebanon_retail seconds=600 frame_ms=100 probe_at=0,300 probe_seed=1 probe_count=20000 emit=256` | baseline run-wide trace, boot geography, sweep at the loaded and stepped world |
| R2 | same as R1 | determinism: every hash and count equal |
| R3 | `scenario=georgia_retail`, otherwise R1 | second map |
| R4 | R1 with `probe_at=` (empty) | no-probe control: the run-wide hash of R4 must equal R1's, showing the sweep does not disturb the campaign |
| R5 | the accepted default 900 s scenario with the fixture DLL, compared with the accepted baseline Tacview/observation/metric bytes | shows the hooks are observational |
| R6 | a second `probe_seed` and the `georgia` (map16) scenario | optional breadth |

Retain the JSON Lines outputs, the stderr, the exit codes, the BUILD-INFO and the post-run input manifests in
`R:/d3-evidence/`; commit only compact, non-retail summaries here. No retail asset, raw recording or credential
belongs in Git. Existing evidence to reuse without rerun: the frozen LOS terrain transcript
`reference/d2-los-observation/original-transcript/` and the D2 shipping pack.

## What this does not yet provide (needed before and after extraction)

- Controlled **changed and withheld host answers**: there is no host terrain callback yet, so nothing here can show a
  host answer changing a campaign state. Planned after the protocol is chosen: the reference host answers from the same
  solver (equal-input conformance, byte-identical regression), then an altered provider (as the LOS +10 km case),
  then a missing provider that must produce a clear error rather than a fallback.
- A **named DCS call or data workaround** per service is only listed as candidates in the audit (section 6).
- The **full byte-identical 40-check regression** with all eight artifacts after any behavioural move.

## Next

1. Independent review of the fixture source (never compiled; expect build fixes).
2. Root builds the fixture and runs R1 to R5; retains raw results.
3. Settle the open questions in the audit (section 5) with the technical lead.
4. Only then the smallest terrain surface extraction on EEDCS-47, with the red/green checks described in AGENT.md.

## Corrections after the first independent source review (pinned `257a18879`)

Source only; nothing built or run.

- **Driver argument validation (P2).** `terrain-characterization.lua` refuses a bad argument with a
  `terrain-characterization:` error *before* it opens the output file, requires the module or boots. Bounds (documented in the
  script header): `frame_ms` integer 10..1000; `seconds` decimal 0..36000 (positive: 1..1,000,000 frames and the clock must advance
  every frame; exactly `0`: an instantaneous boot-and-sweep that runs no frame and so has no clock-progress check, with every requested
  probe still required to be taken); `probe_seed` integer 0..4294967295 (a u32; the C sweep treats 0 as 1, so seeds 0 and 1 give the
  same sweep); `probe_count` 0..200000; `emit` 0..256; `scenario` one of the three; `root` and `out` non-empty. `probe_at` is empty
  (`probe_at=`, the no-probe control) or a strictly ascending list of plain non-negative decimals, none beyond the simulated total
  (so the default `0,300` is refused with `seconds=0`); malformed, empty, repeated, descending, negative and non-finite tokens are
  refused. Defaults are unchanged (600 s, 100 ms, `0,300`, seed 1, 20000 points, 64 samples). Unknown argument names are not
  rejected. Root's real-Lua-5.1 guard helper covers only argument setup under an I/O tripwire and never boots EECH or runs a frame:
  v1 was RED on `257a18879`; the corrected working tree was 11/11 at an earlier hash (`577ffab4`); the v2 cases (which add the two
  zero modes) were 4/14 on `257a18879` and 12/14 on the first correction, failing only `valid_probe_only` and
  `valid_zero_seed_alias`, which this preservation fix restores. The re-run is the Root's; none of it is native characterization
  acceptance.
- **Observer fidelity (P3).** `slow_scans` counts every non-fast, non-Cuba query, including a full scan that finds no face.
  Added `partition_ok`; the driver refuses a report where it is not 1. No claim is made that a no-face query occurs.
- **Evidence wording (P3).** See "Observer hooks" above: feature-off logic unchanged by inspection only.
- **Census (P2).** The audit now lists active model/device/target-point geometry as D3 (G3b, G3c) and no longer calls any active
  service dormant without evidence; see `docs/handover/D3-SCOPE-AUDIT.md`.

Root verification after the preservation fix: the real Lua 5.1 v2 setup guard
completed with actual exit 0, all 14 cases passing; syntax parsing also passed.
Tested driver SHA-256: 4400e7eded209f3cd721f3dfa290f7853011eddd9fcca99cb8c9a104d9e0749f.
The helper uses an output-opening tripwire and never boots EECH or runs a frame.
This supersedes the pending guard re-run statements above; C/Rust compilation,
original native characterization and observation controls remain unrun.
