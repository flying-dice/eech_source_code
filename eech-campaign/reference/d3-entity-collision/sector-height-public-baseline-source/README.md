# Sector-height public baseline: observer source and checks (first substage of the host-owned sector-height extraction)

Task EEDCS-49 (D3), 7 November 2026 (UTC). **UNREVIEWED, source prepared, UNRUN natively.** Source before: `d23882dc28225744a953f3e9335eca4bd7ae05fa`. This is the first substage of the REQUIRED
sector-height production extraction, not another preparation and not a production move: it prepares the test-only observation of the named missing facts of
`sector-height-host-move-open-facts` on the real public path, BEFORE any production behaviour moves. The plan (hooks, every input, state effect and immediate consumer, written before the
structural edits) is [`docs/d3-sector-height-public-baseline-plan.md`](../../../docs/d3-sector-height-public-baseline-plan.md). The external freeze is
`R:/d3-evidence/D3-sector-height-public-baseline-CODE-FROZEN-20261107.json` (a literal copy is here).

The progress guard was run first with `--action fix-blocker --blocker sector-height-host-move-open-facts` and `--action production-cutover`, both exit 0. The preparation count stays 2 (no
reset). `ENV-ENTITY-POSE-LONG-RUN-MOTION-1` stays CLOSED at f734 (SAME review `4de6d2e9`); `sector-height-host-move-open-facts` stays OPEN until the native observation has actually run and
been assessed.

## What was prepared

| item | where | what it is |
|---|---|---|
| feature `sector-height-observer` (default off) | `eech-engine-sys`, `eech-engine`, `eech-dc` `Cargo.toml` | test-only; with it off no hook, patch or definition exists |
| 11 staged patches `SH1`..`SH11` | `crates/eech-engine-sys/build/patches.rs`, `build/main.rs` | exact anchors with a strict count, applied to the STAGED copy only; **no path under `aphavoc/` or `modules/` is edited** |
| observer | `csrc/eech_sho.c`, `csrc/eech_sho_hooks.h` | read-only, bounded tables, no floating-point arithmetic (floats cross as pointers), overflow latched, `complete` verdict |
| four report entries | `csrc/eech_engine.{c,h}` (insertions only), `eech-engine-sys/src/lib.rs`, `eech-engine/src/lib.rs`, `eech-dc/src/module.rs` | `sector_height_summary`, `_event`, `_clearance`, `_scan` (feature builds only) |
| public driver | `crates/eech-world/lua/sector-height-baseline.lua` | the ordinary campaign driver's boot and frame loop, then the observer report in a strict line grammar; refuses an observation that is not whole |
| Windows build | `tools/build-windows-sector-height-observer.sh` | the unchanged `build-windows.sh` with one added cargo feature argument, own cargo volume |
| Windows run | `tools/sector-height-baseline-windows.ps1` | ONE driver process, never overwrites an observation, accepts only a whole verdict |
| controls | `eech-engine-sys/tests/sector_height_observer.rs`, `eech-world/tests/sector_height_baseline_driver.rs` | the recorder against the real sector code; the driver's integrity rules (NOT the funnel) |

The observer records, with one global sequence number: every sector-list INSERT of a fixed entity or city; every LINK_CHILD with type, fixed flag, shape and validity, position and box-top bits when
the original read them, and the word before and after; every statement the host receives (fixed six words, city ten words) under its key; per-sector aggregates of the FIRST scan reads;
counts of fact 12 offers; the two runtime guards, the words and the final `min_height` of the fixed-wing clearance; and the session pack/unpack words. Mobile links are counted only.

## Checks actually run (Linux, ONE cache owner: volume `eech-linux-d3-collision-driver-5416adb3b-20261029`, image `sha256:4a5618cb...`, no container active before or after)

Each is ONE Cargo child through `R:/d3-evidence/run-sector-height-baseline-direct-20261107.py` (a copy of Root's direct runner with its own record stem): separate stdout and stderr, no pipeline,
the exit read from the runner's own record. All records, failures included, are in [`runs/`](runs/).

| label | command | exit | result |
|---|---|---|---|
| c1-build | `cargo build --locked -p eech-dc --features sector-height-observer` | 0 | all 11 patches matched once; DLL builds |
| c2-observer-test | the recorder control (before formatting) | 0 | 52 links, 51 link events, 17 packed and 17 unpacked, 179 expectations unchanged |
| c3-driver-test | `cargo test -p eech-world --test sector_height_baseline_driver` | **101** | compile error E0716 in my new test (retained) |
| c3b-driver-test | same | **101** | 5 of 6; my ordering assertion matched the summary's own `sho-clearance` line (a test defect, retained) |
| c3c-driver-test | same | 0 | 6 of 6 |
| d1-fmt | `cargo fmt -p ... -- --check` | **1** | formatting differences in the three new Rust sources (retained) |
| d1b-fmt-apply | `cargo fmt -p eech-engine-sys -p eech-engine -p eech-dc -p eech-world` | 0 | applied; `git diff` shows insertions only in tracked files |
| d2/e2 clippy default | `cargo clippy --locked -p eech-engine-sys -p eech-engine -p eech-dc -p eech-world --all-targets -- -D warnings` | 0 / 0 | feature OFF |
| e1-fmt-check | `cargo fmt ... -- --check` | 0 | |
| e3 clippy observer | `cargo clippy --locked -p eech-engine-sys --features sector-height-check,sector-height-observer --all-targets -- -D warnings` | 0 | |
| e4 clippy dc observer | `cargo clippy --locked -p eech-dc --features sector-height-observer --all-targets -- -D warnings` | 0 | |
| f1-guards | `cargo test -p eech-engine-sys --test sector_height_guard --test entity_roster_guard --test entity_host_guard --test tree_host_guard --test entity_pose_lifecycle_guard` | 0 | the five binaries passed 5, 4, 20, 6 and 2 tests (unchanged in-tree pins) |
| f2-control-unhooked | `... --features sector-height-check --test sector_height` | 0 | 179 expectations, unhooked |
| f3-observer-control | `... --features sector-height-check,sector-height-observer --test sector_height_observer` | 0 | same 179 with the hooks compiled in; recorder whole |
| f4-driver-guards | `cargo test -p eech-world --test sector_height_baseline_driver` | 0 | 6 passed |

The f-series and e-series ran on the final source bytes (after formatting). The engine C ran at its default level 1 (`EECH_C_OPT` does not reach `eech-engine-sys`); no -O2 claim.

## What the controls do and do not show

The recorder control drives the real `response_to_link_child`, float getter and pack/unpack with the hooks compiled in. It shows the hooks leave the 179 expectations unchanged, the recorder is
whole, a link never lowers the held word, and every restored raw word was packed. It does NOT run the public funnel: there is no roster, host, en_list insert or retail data, so the
`SH1`, `SH8`..`SH11` hooks and the fixed-wing hooks are exercised only by compilation here. The driver guards use a scripted engine. The funnel observation is the native run below.

## The matched native plan (for Root; nothing here has been run)

1. Clean checkout of the code commit named by Root's review. Build the observer DLL: `tools/build-windows-sector-height-observer.sh target/sector-height-observer/<id>` with a FRESH
   `EECH_WIN_TARGET` volume name (record image, commit, BUILD-INFO and the SHA-256 of every file). Build the ordinary DLL of the same commit as the identity reference.
2. Fresh verified copies of the retail roots (Lebanon map5 `LEBANON.CHC`; Georgia map3 `GEORGIA.CHC`) per run, never the consumed f734 roots; the three-state input manifest check before and after as in the
   earlier natives; no baseline, seed, frame, duration or tolerance changed.
3. Per scenario ONE run of `tools/sector-height-baseline-windows.ps1 -Root <fresh root> -Bin <observer bin> -Out <new dir> -Scenario lebanon_retail|georgia_retail` with the established defaults
   (`hours=1`, `frame_ms=100`, `seed=1`); keep stdout, stderr, the observation file, the process exit and the input manifests. A named second duration is Root's decision and is never chosen by outcome.
4. Control for read-only-ness: the ordinary short check run with the observer DLL against the three accepted short artifacts (the observer build takes the address of two locals in `fw_move.c`, so it is
   not assumed result-neutral). A difference is a finding about the observer.
5. Assess, before any production move: M1 order of INSERT, LINK_CHILD and the fixed-state statement per family; M2 whether the raise's shape/position/box-top equal what the statement carries; M3 whether any
   city arrives with a valid shape; M4 FIRST reads and fact 12; M5 whether the guarded clearance is reached and with which words; M6 whether a save or restore happens. A case with a zero counter is
   UNREACHED and stays named; a configured query is not a natural engagement.

## Limits

Source and Linux checks only. No Windows build, native run, retail data, campaign, war or Tacview result; no physical move; the Core high-water writer, clocks, the random stream and every policy are
unchanged; no baseline, input, seed, frame, duration or tolerance changed; no -O2, signalling-NaN, x87 control-word or exhaustive runtime-census claim. The observer's records will be evidence about one
scenario, not the supported domain. All ten D3 requirements and the remaining hints, whole UPDATED/shared terrain, raw TREE/FOREST/CLOUD RNG, needed hierarchy and services, DCS API workarounds, natural moving,
the final current 40/all-8/final-4-inputs, CI, review, normal merge and final actual census remain. No D4/D5/TypeScript. Author only: no self-review, approval or delivery claim.
