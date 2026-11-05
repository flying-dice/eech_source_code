# M4: baseline changes, classified, approved and non-destructive

This record is part of M4 ([#31](https://github.com/flying-dice/eech_source_code/issues/31)). It answers: **when accepted behaviour changes, can we tell why it changed, prevent an unapproved re-baseline, preserve what was previously accepted, and identify exactly which baseline is authoritative now?**

**Answer: yes, for the Windows campaign baselines.**

- **Why it changed.** A red pack becomes a change record. Every failed claim and check in it is classified, with evidence, as one of the programme's four classes. Until then its disposition is `UNCLASSIFIED`, and the pack says so.
- **No unapproved re-baseline.** The old path is closed: `regress-windows.ps1 -Update` is refused, and the runner no longer writes a missing baseline.
  - A regression never advances a baseline.
  - A defect correction or semantic change advances it only with a verified formal APPROVED review by fd-starscream-bot, of a commit that holds that exact record.
  - An input or world variation never replaces the equivalent-input baseline.
  - Overwriting the working baseline by hand fails the pack.
- **What was accepted is preserved.** Versions are never overwritten or removed. A change adds a version, and the superseded one stays with its provenance and the record that superseded it.
- **The authoritative baseline** is the lineage's current version. Today that is **`windows-retail` v1**: the accepted Georgia and Lebanon baselines, unchanged. Every pack report names it.

The procedure lives in the existing review process: [`docs/governance/REVIEW.md`](../../docs/governance/REVIEW.md), "Baseline changes".

## What was added

| | |
|---|---|
| [`regression/baselines/lineages.json`](../regression/baselines/lineages.json) | Lineage `windows-retail`: its configuration (Windows build, retail Georgia and Lebanon inputs by manifest, seed 1, 3 hours), its working path `regression/windows/`, its versions, and which is current. **v1** is the baseline in force at `e0801614`, stored in `regression/baselines/windows-retail/v1/`. It is byte-identical to `regression/windows/` and records its sources (#19, #63; Georgia from `12e8f692`, Lebanon from `0288ece2`). Earlier git history is not reconstructed. |
| [`regression/changes/`](../regression/changes/) | Change records, one JSON file each. A record carries: <ul><li>the change id;</li><li>the baseline version and revision under test;</li><li>the candidate's revision and build hashes;</li><li>the scenarios and inputs;</li><li>the pack report before the change, by hash;</li><li>every failed claim and check as a delta, each with its interpretation, evidence and classification;</li><li>the class-specific statements (`defect`, `semantic_change`, `variation`);</li><li>any claim changes, the proposed baseline, the previous baseline, and the residual risks;</li><li>after approval only, the approval reference and the resulting baseline versions.</li></ul> |
| [`tools/baseline-governance.py`](../tools/baseline-governance.py) | <ul><li>`propose`: a red report becomes a skeleton record, with every failure `UNCLASSIFIED`.</li><li>`status`: a report's disposition.</li><li>`adopt`: advance by an approved record.</li><li>`check`: lineages, versions, working copies and records are consistent.</li></ul> |
| **Approval** | A reference, `{pr, review_id, commit}`, never a flag. The tool fetches the review from GitHub. It must be by `fd-starscream-bot`, `APPROVED`, of that commit, and not followed by a request for changes. The record at that commit must have the same **proposal digest**: every field except the approval and the resulting baselines. Editing the classification after approval therefore voids it. |
| `tools/regress-windows.ps1` | `-Update` throws with the governed procedure, and a missing baseline is a FAIL, never written. |
| `tools/regression-pack.py` | <ul><li>The retained tier verifies `governance.lineage` (lineages, records, and `pack.json` naming the current version) and runs `governance.fixture`.</li><li>Every report names the accepted baseline.</li><li>Each failed claim and check gets its disposition beside the result.</li></ul> |
| [`regression/fixtures/governance/`](../regression/fixtures/governance/) and `tools/baseline-governance-selftest.py` | A test-only fixture (below). |

**Dispositions**, in the pack's words:

| Disposition | Meaning |
|---|---|
| `UNCLASSIFIED` | No record for this build, or a failure the record does not classify with evidence. |
| `CLASSIFIED AS <class> - APPROVAL REQUIRED` | Classified; no verified approval. |
| `CLASSIFIED AND APPROVED` | Classified, with a verified approval. Once adopted, a report judged against the older version reads, for example, `against v1: approved intentional-semantic-change; v2 is the accepted baseline`. |
| `CLASSIFIED AS REGRESSION - REJECTED` | Classified as a regression; nothing advances. |

A disposition sits beside the failure; it never replaces it. A classified red pack still exits 1.

## A. The rejected path: the S3 mutant (real programme evidence)

The candidate is #68's test-only mutant: `56b164d7` with S3 removed, reused, not rebuilt. Its red pack report is #69's retained `reference/m4-regression-pack/s3-mutant/report.json`. The outputs are in [`reference/m4-baseline-governance/`](../reference/m4-baseline-governance/) and were produced at `61b53bde`.

| Step | Result |
|---|---|
| `propose` from the red report (in a scratch copy) | 14 deltas covering the 14 failed claims and 9 failed checks, all `UNCLASSIFIED`. Disposition: `UNCLASSIFIED`. |
| Classified in [`regression/changes/2026-09-27-s3-mutant.json`](../regression/changes/2026-09-27-s3-mutant.json) | Every delta is `regression`, with evidence. **Disposition: `CLASSIFIED AS REGRESSION - REJECTED`, for all 23 failures.** |
| `adopt --change 2026-09-27-s3-mutant` | `refused: ... classified as a regression, which never advances a baseline; the candidate stays failing`, exit 1. `lineages.json` and `regression/windows/*.json` are byte-unchanged. |
| `regress-windows.ps1 -Update` | Refused with the governed procedure. The baselines are unchanged. |
| `baseline-governance.py check` | `windows-retail v1 (established e0801614; unchanged)`, and the record is consistent. |

**The evidence behind the classification**, as the record carries it:
- **Attribution from #68:**
  - `mutation.diff`: the S3 entry removed, nothing else;
  - the compiled patch markers: only S3's is missing;
  - the control and the mutant: the same toolchain and inputs, and the control passes.
- **The class, from #62:** S3 is an accepted defect correction.
- **What changed:** ammo deliveries vanish while fuel deliveries rise (`difference.txt`); in the reference run, all 30 deliveries are fuel (`chains.json`).
- **The corrections that did not change:** S1, S2's removal, the portability and the load-time corrections are recorded as failing only because the whole war diverged after S3's removal. Their difference is the S3 regression, not a change to them. The difference alone did not decide that; the one-patch diff did.

**The pack, end to end:** `tools\regression-pack.py` ran at `61b53bde` against the mutant (`s3-mutant/report.md`).
- **Behaviour unchanged from #69:** every claim's status and every check's result is identical to its report.
  - 2 claims fail as directly protected.
  - 12 fail as differing from the baseline.
  - 15 directly protected claims pass.
- **Each failure now also carries its disposition,** for example `FAIL - directly protected; CLASSIFIED AS REGRESSION - REJECTED`.
- **The verdict** is `FAIL - CLASSIFIED AS REGRESSION - REJECTED (change record ...)`, and the exit code is 1.
- **The report names the accepted baseline:** `windows-retail v1 (... unchanged)`.

**On the accepted build** (`accepted/report.md`), the same revision gives `PASS`:
- all 40 checks pass, including `governance.lineage` and `governance.fixture` (11 of 11);
- 17 claims directly protected and 12 baseline-sensitive;
- judged against `windows-retail v1`.

## B. The accepted-change path, on a test-only fixture

[`regression/fixtures/governance/`](../regression/fixtures/governance/) is **not an EECH campaign baseline** and is not evidence for any claim. Its approvals come from a local test approver, `governance-fixture-approver`, never from fd-starscream-bot. The programme's lineages refuse fixture lineages, and a fixture approval naming `fd-starscream-bot` is rejected.

`tools/baseline-governance-selftest.py` builds a temporary git repository and walks the transition ([`fixture-selftest.txt`](../reference/m4-baseline-governance/fixture-selftest.txt), **11 of 11 as expected**):

| Step | Result |
|---|---|
| v1 current; a red report; `propose` | `UNCLASSIFIED` |
| Classified as `intentional-semantic-change`, with the rule and the claim change | `CLASSIFIED - APPROVAL REQUIRED`; `adopt` refused |
| Committed; the test approver approves that commit; the record references the review | `CLASSIFIED AND APPROVED` |
| `adopt` | v2 current, v1 kept, working copy = v2. The old report now reads `against v1: approved intentional-semantic-change; v2 is the accepted baseline`, and the lineage (printed) shows v1 → v2 by `fixture-change`. |

**Broken variants**, each detected by `check` or refused by `adopt`:

| Variant | What it breaks |
|---|---|
| no classification | the classification removed |
| no approval | the approval reference removed |
| no previous version | the superseded v1 deleted |
| tampered | the record edited after its approval |
| wrong reviewer | the approval names another reviewer (here fd-starscream-bot) than the test approver |
| withdrawn | a later request for changes |
| overwritten | the working copy overwritten outside the governed path |
| invented class | a class outside the four (`benign-drift`) |
| regression | classified as a regression: refused, and v1 stays current |
| variation | an approved expected variation founds its own lineage, and v1 stays current |

## What this does not establish

- **The first real approved change.** No accepted baseline has changed yet. The approval path runs end to end only on the fixture, with its test approver.
  - Against GitHub, the approval check has been exercised as far as it goes without a real change: `check` and the pack run it for every record that references a review, and today no record does.
  - The first real baseline change will be the first live use.
- **Server-side enforcement.** A person with push rights can still edit files. What changes is that such an edit fails the pack loudly: an overwritten working copy, a missing version, an unapproved current version, or a record edited after approval. The merge gate remains the lead's review (`REVIEW.md`).
- **Classification by the tool.** It checks that every delta is classified, with evidence, in one of the four classes, and that the classes are used consistently. Whether the evidence supports a classification is the reviewer's judgement.
- **Other lineages.** The Linux baselines are stale, not accepted evidence, and outside the lineages. The retained M2/M3 artifacts are protected by their hashes in `pack.json`, not by lineages.

## Reproduce

```powershell
python tools\baseline-governance.py check
python tools\baseline-governance.py status --report reference\m4-regression-pack\s3-mutant\report.json
python tools\baseline-governance.py adopt --change 2026-09-27-s3-mutant      # refused
python tools\baseline-governance-selftest.py
python tools\regression-pack.py --retained-only
python tools\regression-pack.py --georgia <root> --lebanon <root> --lebanon-repeat <second root> --bin target\mutants\s3\mutant-bin
```
