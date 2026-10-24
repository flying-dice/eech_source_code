# Technical-lead review and merge process

Policy: [AGENT.md](../../AGENT.md). Charter: [PROJECT.md](../../PROJECT.md).
Work tracking: Jira project [EEDCS](https://flying-dice.atlassian.net/browse/EEDCS).
Retired bootstrap Tasks [#51](https://github.com/flying-dice/eech_source_code/issues/51)
and [#52](https://github.com/flying-dice/eech_source_code/issues/52) are historical
records, not gates; AGENT.md states the current entry conditions.

## One Task, one active delivery PR

Use a dedicated branch based on the agreed integration revision; never push
Task changes directly onto the shared integration branch. Record the actual base.
The initial native integration target is
`claude/eech-campaign-native-spike-tjkj6g`, not `master`. Future retargeting is an
explicit integration decision. Administrative and investigation Tasks also deliver
PRs; a single Task/PR can span many sessions.

If the Task is too large, agree linked smaller Tasks. If a PR must be replaced,
link its successor and preserve discussion/evidence rather than pretending it
never existed. Keep one active delivery PR per Task. The title is descriptive;
its body links Task, Epic and milestone and follows the
[PR template](../../.github/PULL_REQUEST_TEMPLATE.md).

## Author evidence before review

Characterise the affected baseline before structural edits; prefer sociable tests
of real collaborating components. Provide the relevant red reproduction for a
bug/new capability, or explain why a pure refactor/admin change has no artificial
red step. Show green candidate results and the discovered boundary checks.

Document why the tests protect observable behaviour rather than a speculative
implementation. For a split, demonstrate actual host authority/interaction use,
not just interface declarations. Preserve integrated Lua/module evidence when
that path is affected. Identify the validation scope and all checks not run.

Supply base/candidate SHAs, scenario and input identities, exact commands, results,
retained artifacts, meaningful differences, limitations and reproduction/rollback.
Tacview supports physical interpretation; assertions and structured observations
support behavioural claims. Use [BASELINE_RECORD.md](BASELINE_RECORD.md) as an aid,
not a requirement to create a large document for a small change.

## Baseline changes

Accepted campaign baselines are versioned lineages in `eech-campaign/regression/baselines/lineages.json`. Each lineage is one configuration: platform, inputs, seed and duration. Its current version is the one the runners read.
- **Never re-recorded in place.** No runner writes a baseline (`regress-windows.ps1 -Update` is refused).
- **Never overwritten or removed.** A version, once accepted, stays.
- **Changed only by an approved change record.** A baseline changes only through a change record in `eech-campaign/regression/changes/`, handled by `eech-campaign/tools/baseline-governance.py`.

A red regression pack (`tools/regression-pack.py`) produces a proposal, not a new baseline. The procedure:

1. **Propose.** `baseline-governance.py propose --report <pack report.json> --id <id> [--candidate <dir>]` turns the red report into a change record that lists every failed claim and check as `UNCLASSIFIED`.
   - The record keeps the pack report from before the change.
   - It carries the identities of the baseline version under test, the candidate build, the scenarios and inputs, and the previous baseline for rollback.
   - With `--candidate`, it also carries the proposed baseline.
2. **Classify every meaningful delta, with evidence,** as exactly one of the classes in [AGENT.md](../../AGENT.md). A difference never classifies itself: the evidence must show its cause.
   - **`regression`:** rejected. It never advances a baseline, and the candidate stays failing until the behaviour is corrected or a different decision is explicitly made.
   - **`approved-defect-correction`:** name the corrected defect and the behaviour it retires (`defect`).
   - **`intentional-semantic-change`:** state the changed rule or contract (`semantic_change`) and the accepted claims it changes (`claim_changes`), and update them deliberately.
   - **`expected-input-world-variation`:** name what input, world or configuration changed (`variation`). It never replaces the equivalent-input baseline. When approved, it founds a separate lineage (or records comparison evidence).
3. **Review.** The PR carries:
   - the pack report before the change;
   - the change record, with every delta classified and its evidence;
   - the old and proposed baseline identities;
   - any claim or protection-class changes in `regression/pack.json`.

   Only a current formal **APPROVED** review by fd-starscream-bot of a commit holding that exact record approves it.
4. **Adopt.** After the approval, a follow-up commit does two things:
   - it adds the approval reference, `approval: {pr, review_id, commit}`, to the record;
   - it runs `baseline-governance.py adopt --change <id>`.

   The tool checks the review on GitHub: reviewer, state, reviewed commit, and that no later request for changes withdrew it. It then checks that the record's content at the reviewed commit is unchanged. Only then does it add the new version, keep the superseded one, and make the new version current. The PR then shows the pack result against the new baseline and links the superseded version. These new commits need fresh approval, as any change does.

Approval is never a field anyone can set: the record references a GitHub review, and the tool verifies it. `tools/regression-pack.py` then:
- verifies the lineages, versions, working copies and records, and fails loudly on broken provenance;
- reports each failure with its disposition: `UNCLASSIFIED`, `CLASSIFIED - APPROVAL REQUIRED`, `CLASSIFIED AND APPROVED`, or `REGRESSION - REJECTED`.

A disposition explains a difference; it never makes a failing check pass.

The Linux baselines (`regression/*.json`) are stale and are not accepted evidence, so they are outside these lineages.

## Independent technical-lead review

The required reviewer is **fd-starscream-bot**. Verify the actual GitHub login,
not a display name or commit author string. Implementing agents must author PRs
under a different authorised identity. Do not borrow credentials or impersonate
another contributor. Never request or manufacture self-approval.

The lead reviews scope against the milestone, baseline provenance, test relevance,
source/compatibility semantics, actual campaign/world ownership, regression risk,
reproduction evidence and the proposed final diff. Review the current head and
its intended merge result; a materially changed base also warrants renewed
validation and review.

A current formal GitHub **APPROVED** review from fd-starscream-bot is mandatory.
A comment, requested review, self-review, another approval or green CI is not
sufficient. An author response is not resolution of blocking lead feedback.
New reviewable changes after approval require fresh lead review; do not treat an
old approval on different behaviour as permission to merge.

## Merge and completion

An authorised maintainer may merge only after the lead approval, applicable
required checks and blocking discussions are satisfied, and the Task outcome is
accepted. Implementing agents do not self-merge. Do not enable auto-merge: with
no server-side protection it would not wait for the lead's approval. Do not bypass
rules, dismiss the lead's requests for changes or weaken checks to unblock a
candidate.

Close a Task only after its approved delivery PR has merged and acceptance evidence
is linked. A closing keyword may not close an issue on a non-default integration
branch; verify the state explicitly. Child completion does not automatically
accept an Epic or milestone. Record the milestone acceptance and protected baseline.

## Repository enforcement: by process, not server-side

By the repository owner's decision of 2026-10-20 ("remove the rules then"), the
review and merge rules do not depend on server-side branch protection or rulesets.
None was configured then: GitHub reported the native integration branch and
`master` unprotected, with no rulesets. The rules in this file and in AGENT.md
are enforced by process and recorded evidence only. Absent or unverified
server-side protection is not a merge blocker, and no Task or merge requires
activating or verifying it.

The decision removes no other rule. The merge conditions above still apply in
full: a current formal APPROVED review from fd-starscream-bot on the head commit,
passing applicable checks, resolved blocking feedback, an accepted Task outcome,
and different author and reviewer identities, with no self-approval, self-merge,
bypass or automatic approval. If the connection authors the PR as the lead,
resolve the identity conflict before merge. GitHub will not stop a merge that
breaks these conditions, so whoever merges checks them against the exact head
first and records that in the PR. A skipped job is not proof of executed tests.
Do not claim server enforcement from a configuration file. Never create a
workflow that grants approval on behalf of the technical lead.

[CODEOWNERS](../../.github/CODEOWNERS) names fd-starscream-bot as the single
repository-wide owner. Without protection it requests the lead's review but
enforces nothing. Adding another owner or an overriding pattern still requires
review.

## Fresh Claude instruction-discovery check

Start fresh sessions from the repository root and `eech-campaign/` after checking
out the candidate. Record Claude version/settings and working directory. Inspect
its loaded memory/context files and ask, without supplying the answers:

> Which campaign product and hosts are intended? Which Task and Epic are active?
> What authorises M5 structural work, and what must happen before moving existing
> behaviour? What may you do read-only before that authorisation? What does a pure
> refactor need to prove? Who must formally approve a PR, what blocks a merge, and
> what happens when the M4 regression pack differs?

Verify it reads REVIEW.md and the live Task and Epic, and derives the answers from
them and the checked-in instructions, not prior chat context or retained evidence
of earlier checks. Record any live record it could not reach as a gap. The root CLAUDE.md imports AGENT.md outside a code block;
AGENTS.md points other agents to the same canonical file. Check any subagents'
instruction scope separately if used. Static path checking is useful but is not a
fresh-session test. Record exclusions or ignored-import settings as gaps.

## Platform references

These references explain platform mechanics, not a completed repository setup.
Verify against the current runtime/settings before changing controls.

- [Claude memory and imports](https://code.claude.com/docs/en/memory)
- [GitHub code owners](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners)
- [Protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)
- [Required-review approval](https://docs.github.com/en/pull-requests/how-tos/review-pull-requests/approving-a-pull-request-with-required-reviews)
