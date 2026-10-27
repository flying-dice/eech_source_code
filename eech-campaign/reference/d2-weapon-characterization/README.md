# EEDCS-45: evidence for the characterisation of the original physical weapon-execution code (27 Oct 2026)

The account, results and limits are in [`docs/m5-weapon-characterization.md`](../../docs/m5-weapon-characterization.md). This directory holds the retained, safe-to-publish artifacts. No retail asset, installation root, binary or full raw recording is in Git; the roots, binaries and full outputs
named here are private copies under `R:/d2-evidence/weapon-characterization-20261027/` (R: is a subst of the parent workspace, to avoid the runner's unquoted-space argv). **Nothing here claims EEDCS-45 or D2 complete: the weapon-execution code has not moved.**

## Builds (`builds/`)

| commit | role | uncommitted build inputs |
|---|---|---|
| `0d6fa8a94` | first observer. SUPERSEDED: review found fixture defects; its two 600 s attempts were intentionally stopped (not crashes) | 0 |
| `22db40e5b` | review-1 corrections. SUPERSEDED: controlled witness exit 1 (shared target guard refused a valid SITE target; one unexplained case weapon) | 0 |
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
* `controls/*.sh`, `*.ps1`: the build wrapper, the compiler-level proofs, the root and queue scripts. `run-queue.ps1` fails the plan on any nonzero actual exit **and on a PowerShell setup exception**, and writes `PLAN COMPLETE` only when every run exited 0; `run-natives.ps1` is the earlier queue that continued past a failure (kept; used for the superseded attempts).

## Attempts (`attempts/`)

`superseded-0d6`, `superseded-22db`, `superseded-d9` READMEs, and `failed-setup-w2-case-a.console.txt` (the runner's single-backslash regex error: PowerShell stopped before Start-Process, **no native started, no native exit exists**). Their raw outputs, roots and exported binaries are retained privately.

## Limits

Equality evidence is hash/byte based; the Tacview recording was not visually inspected; the model, terrain and object lookups come from the reference build's synthetic scenes; one map, one seed; the controlled witness's launcher/weapon/target are test preconditions. The branches not reached are listed in the document.
