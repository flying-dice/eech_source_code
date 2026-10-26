# EEDCS-44: evidence for the characterisation of the original physical weapon-aiming and readiness code (26 Oct 2026)

The account, results and limits are in [`docs/m5-aim-characterization.md`](../../docs/m5-aim-characterization.md). This directory holds the retained, safe-to-publish artifacts. No retail asset, installation root or binary is in Git;
the roots and binaries named here are private copies under `R:/d2-evidence/aim-characterization-20261026/` (R: is a subst of the parent workspace, to avoid the runner's unquoted-space argv). Nothing here claims EEDCS-44 or D2 complete:
the aiming code has not moved.

## Builds (`builds/`, `BUILD-INFO-*.txt`; the commit, the image, the toolchain and the SHA-256 of each file)

| commit | role | uncommitted build inputs |
|---|---|---|
| `b7f907894` | first observer. `-clean`: the build used for its 600 s pair and 900 s control | 0 |
| `b7f907894` `-dirty14-superseded` | the same commit built while 14 retained-evidence logs (CRLF working copies, LF index, byte-equal after CR removal) counted as changes; **not used** | 14 |
| `4927157a5` | per-kind detail and the person-branch counter; superseded by the correction before any evidence run | 0 |
| `f9f15f52f` | observer-safety correction (no read of an output the original did not write). 600 s pair, 900 s control, the Georgia run | 0 |
| `0ee1fa46b` | adds the readiness state machine. **The build behind the results**: 600 s pair, 900 s control. `-dirty2-superseded`: the first build of this commit, which counted the two untracked evidence paths; **not used** | 0 |
| `b4a00f29d` | first controlled person case; its first run failed in the driver (a host step without an attitude) and is not evidence | 0 |
| `74ae6a96d` | person case with the closest-pair selection and the release phase. **The build behind the person case**; fixture-only additions to `0ee1fa46b` | 0 |

Toolchain: repository `tools/Dockerfile` image in WSL2 distro `EECH-D2-Build`, rustc/cargo 1.99.1, MinGW gcc 12-posix `12.2.0-14+deb12u1+25.2+b1`, Lua 5.1.5; `tools/build-windows-aim-fixture.sh` in a fresh, uniquely named cargo volume per build.
Builds of identical source in different volumes produce different DLL bytes; compare behaviour, not DLL hashes.

## Traces (`traces/`, JSON Lines from the Lua driver; `*.summary.txt` is `tools/aim-trace-summary.py --all-counters`)

* `lebanon-r-a.jsonl`, `lebanon-r-b.jsonl`: `0ee1fa46b`, 600 s, two fresh roots, 1,962 events, byte-identical (SHA-256 `c4e9c2b1...ce62`). Physical `8d5afe5ea932338f`, decision `4e542d029ba7362a`.
* `lebanon-f-a.jsonl`, `-f-b`: `f9f15f52f` (1,740 events, `5612ccab...2841a`; physical `4702d5a5c44e532c`). `lebanon-b7f9-a`, `-b`: `b7f907894` (1,720 events; same two hashes). Retained as the older schemas.
* `georgia-f-g.jsonl`: `f9f15f52f`, `georgia_retail`, 600 s (physical `9bfcb4319945068f`, decision `eb2cf0a10c0696f6`).
* `person-target.jsonl`, `person-target-repeat.jsonl` (byte-identical), `person-control.jsonl`: `74ae6a96d`, the controlled person case (11 checks, 0 failed) and its control (3 checks, 0 failed).
* `*.stderr-final-line(s).txt`: the driver's last line (the engine prints about a thousand lines about missing artwork, dropped).
* **Line endings.** The Windows driver wrote the traces with CRLF; Git stores them with LF. The raw sizes and SHA-256 values quoted in the document (for example `c4e9c2b1...ce62`, 287,242 bytes for `lebanon-r-a`) are those of the original CRLF files, which are retained privately;
  `traces/SHA256SUMS.txt` gives both the original and the LF-normalised hash and size of every committed trace. The comparator and the summary tool read either form (JSON Lines).

Each run was one native process with exit 0 (the logs in the private output directories; the runner scripts in `controls/`). The intermediate development iterations of the person case (an `ipairs` over a step without an attitude; a far target whose link
the campaign removed after one frame; an expectation that the person would move after being linked) were run from the repository script against the `b4a00f29d` binaries and their traces were overwritten; they are described, not retained, except the last:
`traces/person-development-iteration-b4a00f29d-one-check-failed.jsonl` (the facing and the host placement are exercised and every check passes except the one that needed later non-`place` steps, which a linked person does not produce; the release phase was added to supply them).

## Checks (`checks/`)

Input manifests before and after each run (`tools/input-manifest.py --compare reference/lebanon-3h/inputs.json`, Georgia against `reference/m4-s3-mutation/inputs-georgia.json`): IDENTICAL every time. The person-case roots `pc-t2`, `pc-c2` and `pc-t3` were fresh copies without a before-run
manifest; their after-run manifests are identical to the reference, so the files were unchanged by the run. `static-checks-*.log`: `cargo fmt --all --check`, `cargo clippy --locked --all-targets -- -D warnings` (default features) and with `--features eech-dc/aim-fixture`, each cargo exit captured
(the image has no clippy or rustfmt, so the components are added in the disposable container). `strict-preprocess-0ee1fa46b.log`: feature OFF preprocesses identically to the baseline blobs for all eight instrumented sources (the root's own run of the same check is in the private evidence directory).
`crlf-worktree-files.txt`: the 14 retained-evidence files whose working copies were rewritten to their committed LF bytes.

## Controls (`controls/`)

* `comparator-controls-result-final.json` (and the earlier `comparator-controls-result.json`): the actual Python exit of 18 comparator cases, with the comparator's SHA-256.
* `ab-*-vs-supplied-baseline-900s.json`, `ab-*-observation-check.json`: the whole-campaign 900 s control of each build against the retained supplied-binary side (not re-run here); `ab-compare.py`.
* `schema-f9f15f52f-to-0ee1fa46b.json`, `schema-compare.py`: the schema-expanded equivalence.
* the runner and build scripts used (`build-aim-fixture.sh`, `static-checks.sh`, `run-*.ps1`).

## Not done here

No extraction, no host boundary, no shipping-build change, no full pack, no visual Tacview inspection, no player or cockpit path, no seed other than 1, no Georgia run past 600 s.
