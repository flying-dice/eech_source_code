# M4: baseline-change governance, demonstrated

What these show, and what they do not: [`docs/m4-baseline-governance.md`](../../docs/m4-baseline-governance.md).

- `check.txt`: `tools/baseline-governance.py check`. Lineage `windows-retail` v1 is current and unchanged, and the
  one change record is consistent.
- `s3-regression.txt`: the real rejected case. #69's red S3 report, proposed with every failure UNCLASSIFIED (in a
  scratch copy); then classified by `regression/changes/2026-09-27-s3-mutant.json` as REGRESSION - REJECTED, every
  failure; then `adopt` refused. `lineages.json` and the working baselines are unchanged.
- `update-refused.txt`: `tools/regress-windows.ps1 -Update` is refused, and the baselines are unchanged.
- `fixture-selftest.txt`: the test-only accepted-change path (`regression/fixtures/governance/`, **not an EECH
  baseline**, approved by a local test approver, never fd-starscream-bot): v1 to v2, and ten broken variants, 11 of
  11 as expected.
- `s3-mutant/` and `accepted/`: the regression pack at the same revision. On the S3 mutant, each failure is reported
  with its disposition. On the accepted build, the pack passes against windows-retail v1.
