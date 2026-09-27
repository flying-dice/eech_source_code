# Agent operating policy

Canonical shared instructions for the EECH native campaign programme. Read this
file before work. `CLAUDE.md` imports it; `AGENTS.md` directs other agents here.
The enduring charter and linked roadmap are in [PROJECT.md](PROJECT.md).
The review procedure is in [REVIEW.md](docs/governance/REVIEW.md).

## Start each session

Read PROJECT.md, the active Task and parent Epic, the latest accepted baseline,
and outstanding PR review feedback. Inspect the actual branch, base SHA and
working tree; do not overwrite another session's work. The native integration
branch selected for this bootstrap is `claude/eech-campaign-native-spike-tjkj6g`;
verify the current agreed target rather than assuming `master` contains it.

Work on a dedicated Task branch and PR. Continue that PR across sessions. Resolve
the earliest unmet acceptance gate; justify any later-stage probe by the risk it
resolves. Governance Task [#51](https://github.com/flying-dice/eech_source_code/issues/51)
must be accepted before campaign/world restructuring. Read its live blockers;
the presence of these files alone does not mean the gate is accepted.

## Architecture to protect

- One native campaign Lua module, EECH_DC [1], initially original C behind a Rust
  public boundary. This is not a campaign rewrite into Rust or TSTL.
- EECH World [2] and DCS World [3] are alternative hosts, each owning its Lua
  environment and physical world. The reference host may reuse EECH C physics.
- The campaign owns state, decisions, task policy, logistics and progression.
  Hosts execute physical attempts and supply observations. Campaign algorithms
  remain core even when they consume environmental data.
- Exercise host-owned Lua -> public native module -> world interactions for
  product acceptance. Private Rust/C tests cannot substitute for that path.
- A whole-engine DLL is useful transitional evidence, not proof of replaceable
  world authority. Identify coupling honestly; do not split it before evidence.
- Do not add a second authoritative physical world inside the campaign, move
  campaign policy into adapters, fabricate success, or substitute a sidecar or
  second campaign implementation without an explicit architectural decision.
- Keep legacy pointers and layouts private. Discover semantic boundaries from
  source and runtime behaviour; do not impose a speculative trait/ABI/crate plan.

## Behaviour-led red / green / refactor

**Characterise first.** Before changing, splitting or moving behaviour, identify
and exercise it on a named baseline revision. Establish sociable tests around
real collaborating campaign components before restructuring them. Control genuine
environmental inputs, not the campaign decisions being asserted. Record passing
behaviour, known defects, unsupported cases and unavailable prerequisites.

**Red where meaningful.** For a defect, reproduce the failing behaviour before
fixing it. For a new capability or discovered boundary contract, show the relevant
check fails for the missing behaviour, not a setup accident. Do not manufacture a
red test for a pure refactor or an administrative edit: characterisation tests
may correctly pass before the move. No mutation-testing framework is mandated.

**Green with a bounded change.** Preserve existing behaviour while satisfying the
new contract where applicable. Separate structural moves from intentional rule
changes, bug fixes and balancing. Avoid mock-per-module tests that reproduce a
chosen implementation rather than exercise the real system.

**Refactor and prove the boundary.** Keep sociable tests green across the move.
Assert externally meaningful state, invariants, transitions and interactions;
assert ordering where it has contractual meaning, not a private call graph.
After separation, demonstrate that host observations actually influence the
campaign and host actions are actually used. Interface declarations and similar
recordings do not prove this. Reuse the same behavioural scenarios across old
and new paths; explain necessary changes to the test entry point.

TDD is a means of protecting behaviour, not crystallising an untested design.
Do not write tests requiring a preferred private type, directory, crate layout or
algorithm solely to make an implementation choice permanent. Focused isolated
tests are appropriate for genuine local contracts and numerical edge cases.

## Baselines and evidence

Run the affected sociable/contract tests and applicable retained regression and
conformance checks. Include integrated scenarios and Tacview evidence when world
activity is affected. Select validation by risk: do not demand a full war for a
prose edit or call a narrow unit run proof of the entire engine.

Record exact revisions, commands, results, scenario/data/configuration, time and
random inputs, retained artifacts and limitations. Use the optional
[baseline record](docs/governance/BASELINE_RECORD.md). Distinguish reported,
reproduced, inferred and unavailable evidence. A generated, structurally checked,
and visually inspected Tacview recording are three different claims. Recording
must remain observational; missing samples are not authoritative events.

Never silently re-record golden outputs, widen tolerances, delete failing tests,
suppress unsupported paths or tune scenarios just to make new output pass.
Classify differences as regression, approved defect correction, intentional
semantic change, or expected input/world variation. Preserve old evidence and
obtain explicit review of changed expectations. Characterisation does not approve
a legacy defect forever. Exact equivalent-input campaign conformance is separate
from comparable outcomes across different physical simulators.

The abandoned `eech-core-ts` port is reference material, not an active roadmap.
Reuse its source archaeology, C harness, scenarios and numerical findings without
resuming TSTL work or overstating its coverage or independence. Preserve declared
compatibility decisions; unknown historical behaviour must remain unknown.

## Task, review and merge rules

Each completed Task produces its own reviewable PR/MR, including investigations
and administration. If too large, agree linked smaller Tasks before bundling work.
Milestones describe outcomes and evidence, not mandatory implementation edits.
Seek technical-lead agreement before broadening semantic scope, changing accepted
contracts or revising milestone intent. Resolving an uncertainty is valid progress.

The designated technical lead is **@fd-starscream-bot**. A current formal GitHub
**APPROVED** review from that identity is mandatory before merge, alongside passing
applicable checks and resolved blocking feedback. Comments, review requests,
other approvals, CI success and agent self-reviews do not replace it. New
reviewable changes require fresh approval; material base changes require renewed
validation and review. Use the [PR template](.github/PULL_REQUEST_TEMPLATE.md).

Implementing agents must use a different PR-author identity from the technical
lead. Never self-approve, self-merge, bypass rules, dismiss blocking lead feedback,
impersonate reviewers or use another account's credentials. An identity conflict
is a blocker, not an exemption. Rules and templates are not server enforcement;
verify effective protection as described in REVIEW.md. Missing permissions or
unverified protections remain blockers. Governance itself is reviewable work.

Do not close a Task when code is merely pushed or approved. Link the approved
merge and acceptance evidence first; non-default-branch merges may require an
explicit issue-state update. Epic and milestone acceptance requires its own review.
Do not merge this bootstrap or mark #51 complete while its approval, protection
or instruction-discovery requirements remain unmet.

## Handover at every session boundary

Update the Task/PR with: branch and commit; scope and milestone gate advanced;
behaviour characterised; red/green/refactor evidence or justified non-applicability;
boundaries discovered/asserted; actual test results; retained artifacts; baseline
differences; limitations/blockers; review status; and the next bounded action.
Keep continuity in GitHub and reviewed repository records, not only chat history
or ephemeral paths. A failed experiment is evidence, not a completed capability.
