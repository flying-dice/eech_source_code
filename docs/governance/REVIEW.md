# Technical-lead review and merge process

Policy: [AGENT.md](../../AGENT.md). Charter: [PROJECT.md](../../PROJECT.md).
Bootstrap administration: [#51](https://github.com/flying-dice/eech_source_code/issues/51).
Protection prerequisite: [#52](https://github.com/flying-dice/eech_source_code/issues/52).

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
accepted. Implementing agents do not self-merge. Do not enable auto-merge while
the approval/protection contract is unresolved, bypass rules, dismiss the lead's
requests for changes or weaken checks to unblock a candidate.

Close a Task only after its approved delivery PR has merged and acceptance evidence
is linked. A closing keyword may not close an issue on a non-default integration
branch; verify the state explicitly. Child completion does not automatically
accept an Epic or milestone. Record the milestone acceptance and protected baseline.

## Repository enforcement: administrator action required

The proposed [CODEOWNERS](../../.github/CODEOWNERS) has one repository-wide owner,
fd-starscream-bot, including these governance files and the ownership file itself.
This avoids uncovered source, scenario or workflow paths. Adding another owner or
a later overriding pattern can weaken the named-approver rule and requires review.

**CODEOWNERS and this policy do not activate server-side merge protection.** An
administrator must apply and verify suitable rules for the current native
integration branch and for `master` when it receives the programme. Preserve
unrelated repository controls; do not issue a blanket settings replacement.

The effective controls must require PRs and the named lead's review (for example,
required code-owner review with that account as sole applicable owner), stale-review
dismissal for new reviewable changes, resolution of blocking review conversations
and the appropriate observed status checks. Restrict direct/force pushes and
routine bypasses, including administrator/automation bypass where supported.

Choose actual check contexts from observed runs, not guessed names. Existing
workflows have path filters: requiring a check that never runs for a governance PR
can deadlock it. A skipped job is not proof of executed tests. Any required-check
layout change is separate, reviewable work justified by the effective configuration.

GitHub uses ownership from the PR's base branch. A new CODEOWNERS on the candidate
does not protect its own bootstrap. The bootstrap therefore still needs explicit
lead approval and administrator-supervised protection activation; no silent
exception or automatic approval is authorised. If the connection authors the PR
as the lead, resolve the identity conflict before merge.

## Safe enforcement verification

Retain the inspected refs, rule identifiers/settings, eligible reviewer permission
and results. Use API/UI mergeability checks or disposable review probes without
merging unsafe changes. Establish that:

- no lead approval blocks merge, and another reviewer's approval alone does not pass;
- new reviewable commits require renewed lead approval;
- failing applicable required checks and unresolved blocking feedback block merge;
- the documented refs and changed paths actually receive these controls;
- routine implementation credentials cannot bypass the intended gate.

Do not claim enforcement from a configuration file alone. If settings access or
reviewer eligibility is unavailable, leave #52 and the governance acceptance gate
blocked. Record the limitation and maintain the no-merge policy. Never create a
workflow that grants approval on behalf of the technical lead.

## Fresh Claude instruction-discovery check

Start fresh sessions from the repository root and `eech-campaign/` after checking
out the candidate. Record Claude version/settings and working directory. Inspect
its loaded memory/context files and ask, without supplying the answers:

> Which campaign product and hosts are intended? Which GitHub Task is active?
> What must happen before moving existing behaviour? What does a pure refactor
> need to prove? Who must formally approve a PR, and what blocks this bootstrap?

Verify it derives the answers from the checked-in instructions and live Task, not
prior chat context. The root CLAUDE.md imports AGENT.md outside a code block;
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
