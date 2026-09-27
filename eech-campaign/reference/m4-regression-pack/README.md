# M4: the regression pack's runs

The retained output of `tools/regression-pack.py` at `90acbacd` (2026-09-27, Windows 11). What it shows and what
it does not: [`docs/m4-regression-pack.md`](../../docs/m4-regression-pack.md).

- `accepted/`: the accepted candidate's binaries (0288ece2) with the M3 scripts (0ea49a4a). **PASS.**
- `s3-mutant/`: #68's S3 mutant, not rebuilt (`reference/m4-s3-mutation/mutant-BUILD-INFO.txt`). **FAIL**, with
  Lebanon's airbase ammo resupply named.
- `retained-only/`: `--retained-only` at `23016c4f`, after `tools/regression-pack.py` was added to the recorded
  checkers (review of #69). All 13 retained checks pass, including `evidence.checkers` over 13 checkers and runners.
- `fail-loudly.txt`: four runs with altered, missing or edited evidence, the last an edit to `tools/regression-pack.py`.
  Each stops before any judgement.

In each run's directory:
- `report.md` and `report.json`: every claim's status, every check's result and detail, and the blind spots.
- `regress.txt`, `reference-summary.txt`, `lifecycle.txt`: the output of the existing runners.
- `chains.json` and `loss-chain.json`: the chain and loss checkers' reports on the run's first reference run.

The reports say "with uncommitted changes": the only uncommitted file then was the draft of
`docs/m4-regression-pack.md`. Two details read oddly:
- `reference.observed_equals_unobserved` prints `IDENTICAL to the baseline`, which is `regress-compare.py`'s wording.
  Its "baseline" there is the same build's unobserved run.
- The inputs checks name each root twice.

The runs' full outputs (recordings, observations, metrics, copied roots) stay local, under `target/regression-pack/`.
