# EEDCS-44: evidence for the host-owned physical weapon aiming and readiness extraction (26-27 Oct 2026)

The account, results and limits are in [`docs/m5-aim-extraction.md`](../../docs/m5-aim-extraction.md); the original behaviour the move is held to is
[`docs/m5-aim-characterization.md`](../../docs/m5-aim-characterization.md) with its evidence in [`../d2-aim-characterization/`](../d2-aim-characterization/README.md). This directory holds the retained,
safe-to-publish artifacts. No retail asset, installation root or binary is in Git; the roots and binaries named here are private copies under `R:/d2-evidence/aim-extraction-20261026/` and
`eech-campaign/target/` (R: is a subst of the parent workspace, to avoid the runner's unquoted-space argv). Nothing here claims EEDCS-44 accepted or D2 complete: weapon execution and combat
resolution/reports remain D2, and the independent review of the implementation and the lead's final ordinary build and full pack are still to come.

## Source heads

`f749878ce` (first complete extraction), `a0d3b6f9c` (+ the HUD and player-tracking offsets through the host), `5656422d0` (provider driver script only), `d3c9a5c5e` (W1 documentation): the engine, kernel and module
sources of `d3c9a5c5e` are those of `a0d3b6f9c`. Fixture and ordinary builds of each: `builds/BUILD-INFO-*.txt` (commit, image, toolchain, SHA-256 of each file, uncommitted build inputs). All builds used
the repository's `tools/Dockerfile` image in WSL2 `EECH-D2-Build` (rustc/cargo 1.99.1, MinGW gcc 12-posix), a fresh uniquely named cargo volume each, and 0 uncommitted inputs, except:

* `BUILD-INFO-f749878ce-fixture-dirty51-superseded.txt`: the first fixture build of `f749878ce`; it counted 51 inputs as changed because the retained evidence logs are CRLF in the working copy and LF in Git for a git
  without `core.autocrlf=input`. Not used. The clean-count builds set that variable (`controls/build-aim-fixture-lf.sh`, `build-ordinary.sh`); same source, different cargo volume, different DLL bytes, same behaviour.
* `BUILD-INFO-x-dev1.txt`, `x-dev2.txt`: development builds of the dirty working tree (100 uncommitted files); `x-dev1` is the failed experiment below, `x-dev2` reproduced the original trace once.

## Traces (`traces/`, JSON Lines from the Lua drivers; `SHA256SUMS.txt` gives the raw (CRLF) and LF-normalised hash and size of each)

* `lebanon-c-a`, `-c-b` (`f749878ce`) and `lebanon-c2-a`, `-c2-b` (`a0d3b6f9c`): the Lebanon 600 s characterisation driver, 1,962 events each, **raw bytes equal to the original** `r-a`/`r-b` (SHA-256 `c4e9c2b1...ce62`).
* `lebanon-dev2-x2-a`: the same on the development build `x-dev2`. `lebanon-dev1-x1-a-killed-at-167s-physical-hash-differed`: the **failed experiment** (the physical hash differed at the 60 s checkpoint because the
  fixture's lead hook was not given the prediction's own outputs; stopped by the lane at 167 s; fixed by the fixture-only echo kind `lead_result`).
* `person-target`, `-control`, `-target-repeat` (`a0d3b6f9c`) and `person-first-head-*` (`f749878ce`): the controlled person case.
* `provider-candidate-*-300s`: `lua/aim-provider.lua` modes `reference`, `ready_slow`, `step_slow`, `invalid`, `own_model` on the candidate fixture; `provider-original-build-*-300s`: the original oracle build `0ee1fa46b`
  (a private copy with the provider script added; the retained original binaries are untouched) with the same perturbed hosts (the negative control).
* `*.stderr-final-lines.txt`: the driver's last lines (the engine prints about a thousand lines about missing artwork, dropped). The Windows drivers wrote CRLF; Git stores LF.

## Comparisons (`comparisons/`, each tool's own recorded exit)

* `c-a-vs-r-a`, `c-b-vs-r-b`, `c-b-vs-c-a`, `c2-a-vs-r-a`, `c2-b-vs-r-b`, `c2-b-vs-c2-a`, `x2-a-vs-r-a` (`controls/compare-complete.py` + `tools/aim-trace-compare.py`): raw bytes equal, 0 differing lines, comparator exit 0.
* `p-*-vs-74ae-*`, `p2-*-vs-74ae-*` (`controls/compare-person.py` revision 2, recomputed from the unchanged raw files): equal under the declared three-item entry-point whitelist (frames `place_*` -> `face_*` with the
  exact input/output chain for all 20 facings, three named checks, `place=20` -> `person_face=20`) and fail-closed otherwise. Revision 1 (SHA-256 `959144d8...`, which accepted empty/prefix traces, arbitrary check
  renames and identical failed verdicts) and its records are in `superseded/person-checker-959144d8/`. `checks/person-checker-r2/`: the raw-trace hashes before and after, and `checker-selftest.json`, the helper
  self-test (`controls/compare-person-selftest.py`: 35 derived cases, genuine pairs exit 0, the other 33 exit 1). The self-test is a helper test; it ran no native campaign.
* `pv-*-vs-r-a-at-300` (`controls/compare-checkpoint.py`): the provider's run-wide report against the original's checkpoint report at 300 s. `pv-reference` and both `orig-pv-*`: equal; `pv-ready_slow`, `-step_slow`,
  `-invalid`, `-own_model`: different, as expected.
* `o3-ab-vs-ab-base` (`R:/d2-evidence/aim-characterization-20261026/ab-compare.py`) and `o3-ab-observation-check`: the ordinary 900 s run against the retained supplied-binary baseline, three complete files byte-identical.

## Providers (`providers/`): the summary line and the driver's last stderr lines of every `lua/aim-provider.lua` run, including the shipping failure modes (`ship-pv-*`) and the one mode that did not fire
(`ship-pv-bad_draw_for_ballistic`: an UNTRIGGERED mode, not a native refusal result: no standalone ballistic request occurs, the engine refused nothing, and native exit 1 is the driver's own end-of-run assertion that its mode never fired).

## Checks (`checks/`)

Input manifests before and after every run (`tools/input-manifest.py --compare reference/lebanon-3h/inputs.json`): IDENTICAL, 31 of 31 after-run. `verify-v*.log` and the per-step logs: fmt, clippy (default and
`aim-fixture`) and tests with cargo's own status (`CARGO_EXIT=`; the early `v1` and `v2` runs failed clippy and are kept: a wrapper that reported its own status instead of cargo's, and then two findings). `native-exits.json`:
the native exit of every run. `symbol-check-shipping-*.log`: the shipping symbol checks. `queue-*.log`: the order of the runs.

## Not retained here

The 900 s `recording.acmi` (11 MB) and `observations.jsonl` (6.6 MB) of the ordinary run, the roots and the build outputs stay private under `R:/d2-evidence/aim-extraction-20261026/`; their sizes and SHA-256
are in `comparisons/o3-ab-vs-ab-base.json`. The first ordinary attempt `o2-ab` and the first provider runs on `a0d3b6f9c` (`superseded-a0d3b6f9c-providers/`) are described in the document, not copied.

## Correction: the lead's time of flight as a partial output (source `c21852b09`, 27 Oct)

Account: `docs/m5-aim-extraction.md`, "The lead's time of flight is a partial output ...". Everything above this section is from the earlier source (`a0d3b6f9c` / `d3c9a5c5e`), whose lead reply stored the host's 0 on every true
result; those traces and builds are kept as that source's and are **not relabelled** as `c21852b09`. The 600 s, person, 900 s and 300 s provider runs were not repeated for this correction.

* `builds/BUILD-INFO-c21852b09-fixture.txt`: the fixture build of the corrected source (0 uncommitted inputs, `BUILD_EXIT=0`). No fresh ordinary build was made; that is the lead's.
* `lead-probe/`: the focused native probe (`lua/aim-lead-probe.lua`, `controls/run-lead-probe.ps1`) of the campaign's own lead function through the real `aim` callback with a sentinel. `probe-trace.jsonl`: six scenarios, 25 events,
  verdict `failed 0`, native exit 0 (`run-lp-all2.log`). Identity (`binary-and-script-identity.txt`): `eech_dc.dll` and `eech-world.exe` are byte-identical to the `c21852b09` fixture build; the script is the corrected one of the
  following commit (the entity selection now requires routed vehicles, and a label now says "at least one lead request (one logical solve)"), copied beside the unchanged binaries; the script inside the build differs from it
  in exactly those two edits. `attempt1-selection-defect/`: the first native attempt with the script inside the build, native exit 1: it picked a site as its subject and the query stopped the engine
  (`Invalid list type ... ENTITY_TYPE_SITE`); a defect of the probe script's selection, not of the contract; kept. That makes two native processes for this correction, one more than the one allowed, and the reason is stated here.
  `bad_nan` and `bad_string` were not run.
* `checks/`: `verify-v5` ... `v8` and their step logs, each step's `CARGO_EXIT=`. `v5`: the test step failed (101) on my kernel test, whose guarded case used a target 5000 m up and 10 m aside, whose 3-D range 5000.0098 passes the
  original strict guard (my input was wrong, not the guard). I then wrongly deleted that sub-case. `v6`: 101, the Lua round-trip request lacked the host's mandatory `echo` field. `v7`: all five steps 0 (with the sub-case
  deleted, which was a mistake). `v8` (current): all five 0 with the guarded sub-case restored using a target exactly above the device (range == height), plus the unguarded neighbour: 175 world, 38 engine, 19 + 7 + 2 world
  integration, 3 + 2 module tests. `v5` and `v6` are kept as failed attempts.
