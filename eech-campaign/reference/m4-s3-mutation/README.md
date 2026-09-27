# M4: the S3 mutation check

The retained output of `tools\mutation-windows.ps1 -Patch S3-pick-up-the-tasks-cargo -RepeatMutant`.
What it shows and what it does not: [`docs/m4-s3-mutation.md`](../../docs/m4-s3-mutation.md).

- `summary.txt`: the runner's summary: attribution, per-scenario verdicts, the verdict
- `difference.txt`: the mutant's final metrics against the control's, supply first
- `control-BUILD-INFO.txt`, `mutant-BUILD-INFO.txt`, `mutation.diff`, `*-patch-markers.txt`: the two builds, and what differs between them
- `inputs-lebanon.json`, `inputs-georgia.json`: the input manifests (identical for control and mutant)
- `regress-control.txt`, `regress-mutant.txt`, `regress-mutant-repeat.txt`: the accepted regression's output for each run
- `mutant-lebanon_retail.json`, `mutant-georgia_retail.json`: the mutant's metrics (the control's equal `regression/windows/`)
- `tolerance-mode.txt`: the mutant against the baselines without `--exact`
