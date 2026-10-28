# EEDCS-45: evidence for the characterisation of the original physical weapon-execution code (27 Oct 2026)

The account, results and limits are in [`docs/m5-weapon-characterization.md`](../../docs/m5-weapon-characterization.md). This directory holds the retained, safe-to-publish artifacts. No retail asset, installation root, binary or full raw recording is in Git; the roots, binaries and full outputs
named here are private copies under `R:/d2-evidence/weapon-characterization-20261027/` (R: is a subst of the parent workspace, to avoid the runner's unquoted-space argv). **Nothing here claims EEDCS-45 or D2 complete: the weapon-execution code has not moved.**

## Builds (`builds/`)

| commit | role | uncommitted build inputs |
|---|---|---|
| `0d6fa8a94` | first observer. SUPERSEDED: review found fixture defects; its two 600 s attempts were intentionally stopped (not crashes) | 0 |
| `22db40e5b` | review-1 corrections. SUPERSEDED: controlled witness exit 1 from the fixed observation window ending while CRATE remained in flight; the overbroad SITE-target guard was a separate source-review defect | 0 |
| `d9d6930b7` | role-specific guards and per-weapon live report. SUPERSEDED: controlled witness exit 1 (the in-flight 511 s crate) | 0 |
| `aaceb0e7d` | **the build behind every result in the document**: adds the birth-state initialisation of the live report; Lua scripts (lossless JSON, per-weapon horizon) are the committed ones of this build | 0 |

Toolchain: repository `tools/Dockerfile` image in WSL2 `EECH-D2-Build`, rustc/cargo 1.99.1, MinGW gcc 12-posix `12.2.0-14+deb12u1+25.2+b1`, Lua 5.1.5; `tools/build-windows-weapon-fixture.sh` in a fresh uniquely named cargo volume per build (the image id differs between builds; the DLL bytes of identical source differ across volumes, so behaviour is compared by trace). Build exits are the wrapper's `BUILD_EXIT=0` printed after cargo, recorded in the private build logs.

## Traces (`traces/`; full text in `SHA256SUMS.txt`)

* `lebanon-600s-a.jsonl.gz`: `aaceb0e7d`, Lebanon, seed 1, 100 ms, ends at the first clock reaching 600 s (6,001 frames, clock 600.0025). The second run `w5-600-b` is **raw-byte identical** (3,097,859 bytes, SHA-256 `faa94591...c0b`) and is not stored twice. Each: native/driver exit 0 (818 s, 820 s).
* `controlled-launch-w4.jsonl.gz`: the controlled launch witness, native/driver exit 0 after 884 s; 11 checks, verifier exit 0.
* `*.summary.txt`: `tools/wpn-trace-summary.py --all-counters`. `w5-queue.log`, `w4-case-a.queue.log`: the queue logs with the actual exit of every run.
* The traces were written by the Windows driver with CRLF; the sizes and hashes quoted are of the original files, retained privately; the gzip files are the same bytes compressed.

## Checks and controls (`checks/`, `controls/`)

* `checks/strict-preprocess-aaceb0e7d.log` (feature off equals baseline `e9cea88a2` for all four sources) and `...control-feature-on...` (differs for all four); earlier proofs for superseded builds are kept.
* `checks/fmt-check-*.log`, `static-checks-v2.log` (clippy default and `--features eech-dc/weapon-fixture`, exit 0, at `22db40e5b`), `post-run-manifests.log` (repository input manifest after every run: IDENTICAL).
* `controls/comparator-controls-result.json`: 34 comparator cases with the actual Python exit of each. `controls/ab-aaceb0e7d-*.json`: the 900 s control against the retained ab-base (byte-identical) and its structural check.
* `controls/*.sh`, `*.ps1`: the build wrapper, the compiler-level proofs, the root and queue scripts. `run-queue.ps1` fails the plan on any nonzero actual exit **and on a PowerShell setup exception**, and writes `PLAN COMPLETE` only when every run exited 0; `run-natives.ps1` is the earlier helper without the new setup-exception logging. Its initial superseded version continued after a nonzero native exit; the retained helper copy already includes the subsequent native-exit fail-fast correction. Use `run-queue.ps1` to reproduce the final sequence.

## Attempts (`attempts/`)

`superseded-0d6`, `superseded-22db`, `superseded-d9` READMEs, and `failed-setup-w2-case-a.console.txt` (the runner's single-backslash regex error: PowerShell stopped before Start-Process, **no native started, no native exit exists**). Their raw outputs, roots and exported binaries are retained privately.

## Limits

Equality evidence is hash/byte based; the Tacview recording was not visually inspected; the verified roots preserve retail 3D data (`textures.pal` is present, so the synthetic-database fallback is skipped) and load retail session terrain; one map, one seed; the controlled witness's launcher/weapon/target are test preconditions. The branches not reached are listed in the document.

## Physical extraction checks (`checks/`, source commits after `abc6d35bd`)

The design is in [`docs/m5-weapon-execution.md`](../../docs/m5-weapon-execution.md). These are source checks only; they are **not** a native comparison with the original behaviour, which is recorded separately when the integrated candidate has run.

* `checks/cargo-test-s2-5.log`, `clippy-s2-default.log`, `clippy-s2-fixture.log`: the flight cut (`ac9b5a6d8`): affected-package tests and Clippy, actual Cargo and Docker exit 0 each (the helper `controls/cargo-run-checked.sh` propagates the real status).
* `checks/cargo-test-s4-2.log`, `clippy-s4-default.log`, `clippy-s4-fixture.log`: the integrated source (flight, birth, release): eech-engine 44, eech-world 191 library, 19 binary, 8 host/Lua (incl. a parse of every shipped Lua script), actual exit 0; Clippy default and fixture, actual exit 0.
* `checks/syntax-check-s4.log`: `controls/syntax-check2.sh`, the touched C sources compiled (syntax and warnings) with and without the fixture macro against the staged headers with the working tree's changed headers laid over them.
* Earlier failures stay retained in the private evidence tree (`cargo-test-s2-1..3.log`, the first `World` alias collision and the NaN weight of a test gun with no motor times, both fixed in tests, not in the kernel, whose weight function is the original's).

* `checks/cargo-test-p2-1.log`, `clippy-p2-1-default.log`, `clippy-p2-1-fixture.log`, `syntax-check-p2-1.log`, `fmt-p2-1.log`: the integrated source with the decoy-expiry reacquisition and the strict 0/1 decisions (eech-engine 48, eech-world 197 library, 19 binary, 8 host/Lua, eech-dc 3, load_dc 2; actual Cargo and Docker exit 0 each; the syntax check is a subset of the engine: `init.c` and other callers are not in it).
* `c-checks/hostaim_validation.c`, `c-checks/run-hostaim-validation.sh`, `checks/hostaim-validation-1.log`: the real `mb_hostaim.c` compiled against a stand-in `project.h` and driven by a scripted host; every combination of -2, -1, 0, 1, 2, 3 and 256 for a decoy's, a seduction's and a reacquisition's decisions: only 0 and 1 accepted (`HOSTAIM_VALIDATION=ok: 0 failures`). This check needs the new protocol, so it has no meaningful result on the previous source; the previous source checked only an action's kind and finiteness (by reading, not by running).

* A defect found by the first native provider run, not by any earlier source check: `aim-provider.lua` and `weapon-case.lua` looked for `own-model.lua` beside themselves through `debug.getinfo (1, "S").source` expecting a leading `@`, but the binary names the main script's chunk by its path alone, so a main script that ran the lookup died at once with "cannot open own-model.lua" (the Lua host tests load by absolute path and never ran this lookup). `aim-provider.lua` runs it at load for every mode, so the failed queue's actual first failure was `g1-pref` (`mode=reference`), native exit 1 after 0 s; the `own_model` cases were reproduced by the new test on the real binary, not launched by that queue. Reproduced red by `tests/host_callbacks.rs the_provider_and_the_controlled_witness_find_the_model_files_beside_them_when_the_binary_runs_them_as_the_main_script` (`checks/cargo-test-p3-red.log`), fixed by accepting an optional `@` as `campaign.lua` already did (`checks/cargo-test-p3-1.log`, all packages green). Build `e6c13864b` (clean, BUILD_EXIT 0) therefore has the defect: its provider chain stopped at the first run (`controls/g1-queue-b-e6c13864b-provider-path-defect.log`, no physics involved) and it is superseded as the final candidate; its 600-second and other runs, if they completed, are interim evidence of that build only.

* Final candidate build `builds/BUILD-INFO-a137fc3ec-clean-fixture.txt`: commit `a137fc3ec`, 0 uncommitted inputs, BUILD_EXIT 0 and OUTER_EXIT 0. The first build of the same commit, `builds/BUILD-INFO-a137fc3ec-dirty1-attempt.txt`, recorded 1 uncommitted input: a CRLF log that the WSL Git saw as modified (autocrlf differs between the two Gits); it is kept, not used, and the file was normalised before the clean build.
* `checks/helper-status-controls/` (root's controls of `c-checks/run-hostaim-validation.sh`, which now captures and propagates the compile, run and Docker statuses and skips the run after a failed compile): the real C guard positive (66 assertions, compile 0, run 0, Docker 0, outer 0), an intentional compiler failure (compile 1, Docker 1, outer 1, no run) and a disposable fake compiler whose run exits 7 (run 7, Docker 7, outer 7). The negative controls are helper tests, not physics failures.

## Diagnostic native run of the flight, launch and release cut (`d71fec4fb`)

`builds/BUILD-INFO-d71fec4fb-fixture.txt`, `builds/build-fixture-d71fec4fb.log`: clean fixture build (BUILD_EXIT 0, zero uncommitted inputs). This build still had the reviewed post-flight seam in the campaign and the five reviewed defects, so its runs are **diagnostic**, not acceptance.

* `s4-600-a` (Lebanon, seed 1, 100 ms, 600 s): native exit 0 after 1,210 s; the trace is **byte-identical** to the retained original (3,097,859 bytes, SHA-256 `faa94591c299ddb3ae20fe8b8101a4317f431139871297e65703c042d6289c0b`); `tools/wpn-trace-compare.py` strict: 10,532 events, scopes boot, checkpoint, log and run, 0 differences. The live trace was also compared as a prefix against the original every 90 s while it grew (`tools/prefix-diff.py` in the private tree), equal at every sample.
* `s4-case-a`, the controlled launch witness on the same build (diagnostic, old source): native and driver exit 0 after 1,207 s (queue log `s4d71-case-queue.log`), `weapon-case: 11 checks, 0 failed, 5725 frames`; the trace is **byte-identical** to the retained original `w4-case-a` (2,009,115 bytes, SHA-256 `14dfda07b8cf327aff9b9daf08a22e2a2034b56f0020ad14c818a9cf262eee55`, compared with `sha256sum`). Like the 600-second run it is evidence about the arithmetic of that cut only.
