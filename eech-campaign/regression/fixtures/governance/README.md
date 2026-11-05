# Governance fixture: NOT an EECH campaign baseline

Test data for `tools/baseline-governance-selftest.py`, which exercises the baseline-change machinery
(`tools/baseline-governance.py`) in a temporary git repository. It is not campaign behaviour, not
evidence for any accepted claim, and its approvals come from a local test approver
(`governance-fixture-approver`), never from fd-starscream-bot. The programme's lineages refuse fixture
lineages and fixture approvals.

- `v1/fixture_metrics.json`: the fixture lineage's first version (value 1)
- `candidate/fixture_metrics.json`: a deliberately changed candidate (value 2)
- `report-before.json`: a minimal regression-pack-shaped report in which the candidate fails against v1
- `classification.json`: the classification the self-test gives it (an intentional semantic change: the
  fixture "rule" now yields 2)
