# EEDCS-9: instruction-discovery and static checks

Evidence for [EEDCS-9](https://flying-dice.atlassian.net/browse/EEDCS-9)
([#71](https://github.com/flying-dice/eech_source_code/issues/71)). Retained as the
fresh-session check in [REVIEW.md](../REVIEW.md) requires. Recorded 2026-10-11.

## Setup

- Claude Code 2.1.296, cloud container, default settings, no project memory.
- Each session was a new `claude -p` process, limited to Read, Grep and Glob,
  with no prior conversation. None of them had a Jira connection, so none could
  read the live Task; each says so.
- Base: `a02d489c`. Candidate: this PR's governance commit.

## Stale wording on the base

`AGENT.md` at `a02d489c`:

> Governance Task [#51] must be accepted before campaign/world restructuring.
> Read its live blockers; the presence of these files alone does not mean the
> gate is accepted.

> Do not merge this bootstrap or mark #51 complete while its approval, protection
> or instruction-discovery requirements remain unmet.

#51 and #52 are closed `not_planned`. The records that replace them: the M4
acceptance review ([#31](https://github.com/flying-dice/eech_source_code/issues/31#issuecomment-5860569409),
[EEDCS-4](https://flying-dice.atlassian.net/browse/EEDCS-4)), the M5 kickoff
([#35](https://github.com/flying-dice/eech_source_code/issues/35#issuecomment-5860740386))
and Epic [EEDCS-5](https://flying-dice.atlassian.net/browse/EEDCS-5).

## Static checks

A script checked the eight governance files for broken relative links, the
`@AGENT.md` import outside a code block in `CLAUDE.md`, the stale gate phrases,
and the rules that must survive (`fd-starscream-bot`, `APPROVED`, file presence
proves no gate, `windows-retail v1`, the regression pack, no silent re-record).

| Tree | Result |
|---|---|
| Base `a02d489c` | 7 failures: 4 stale phrases, 3 entry conditions missing |
| Candidate | 0 failures; no broken links; import active |

## Fresh session on the base, from the repository root

Asked: what authorises M5 structural work, and may it start now?

```text
**It may not start now.** Nothing in the repo authorises M5 structural work yet.

**What gates it.** AGENT.md says governance Task #51 must be *accepted* before any campaign/world restructuring. Having the governance files in the repo doesn't count as acceptance. BOOTSTRAP.md says #51 closes only after all of these:
- A formal **APPROVED** review from the technical lead, @fd-starscream-bot.
- An approved integration merge.
- Verified branch protection (prerequisite #52).
- Evidence from a fresh Claude session that it discovers these instructions.

PROJECT.md then has #20 reassess the native spike once governance is accepted.

**Conditions once it's authorised:**
- Work in a dedicated Task and PR, from a different author identity than the lead.
- Characterise behaviour first, using sociable tests on a named baseline.
- Preserve the M4 baseline across the refactor, and explain every delta.
- Show that host observations and actions actually drive the campaign, with no hidden shared world authority.
- Get the lead's agreement before broadening scope.

Renamed crates, interfaces or unchanged recordings over a still-coupled world don't meet the M5 gate.
```

It cannot start: the base's instructions block M5 on a Task that can no longer
be accepted.

## Fresh session on the candidate, from the repository root

Asked the REVIEW.md questions, then whether M5 structural work may start now.

```text
**Loaded at startup:** `CLAUDE.md`, which imports `AGENT.md`. The userEmail context was also attached. I read `PROJECT.md` just now to answer. I did not read `REVIEW.md` or the live Jira Task, so anything that depends on them is unverified.

**Product and hosts:** The product is one native EECH dynamic campaign Lua DLL, EECH_DC. It is original C behind a Rust public boundary, not a rewrite. It must run under two alternative hosts. EECH World is a Rust reference host with its own Lua environment and physical world. DCS World is the real DCS host. DCS does not run through EECH World.

**Active Task and Epic:** The active Epic is EEDCS-5 (M5, host interchangeability). The instructions don't name a specific active Task. They say to read its live state in Jira EEDCS. I couldn't confirm that here. Tasks #51 and #52 are retired and aren't gates.

**What authorises M5 structural work:** The M4 acceptance review (issue #31 comment, EEDCS-4) and the M5 kickoff (issue #35 comment). They open M5 under EEDCS-5 from integration state `a02d489c`, with `windows-retail v1` as the authoritative baseline. The presence of the instruction files doesn't authorise anything.

**Before moving existing behaviour:**
- Reproduce it on the named baseline.
- Protect it with sociable tests of the real collaborating components.
- Record passing behaviour, known defects, unsupported cases and unavailable prerequisites.

**What a pure refactor must prove:**
- A passing characterisation before the move is fine. A red test isn't required.
- After the move, the sociable tests stay green.
- It asserts both the preserved behaviour and the newly established ownership contract.
- Host observations actually influence the campaign, and host actions are actually used. Interface declarations don't prove this.
- It reuses the same scenarios across the old and new paths.
- Structural moves stay separate from bug fixes and rule changes.

**Approval:** A current formal GitHub APPROVED review from **@fd-starscream-bot** is mandatory. The PR author must be a different identity. Passing checks and resolved blocking feedback are also required. Comments, CI success and self-review don't substitute.

**If the M4 regression pack differs:** Classify the difference as regression, approved defect correction, intentional semantic change, or expected input/world variation. Follow REVIEW.md's baseline governance. Preserve the old evidence and get explicit review of any changed expectations. Never silently re-record golden outputs, widen tolerances, or delete failing tests.

**May M5 structural work start now?** Yes, in principle. It's authorised under EEDCS-5 on a dedicated Task branch and PR, and it must meet these conditions:
- Characterise first.
- Run the M4 regression pack against each candidate.
- Obtain the technical lead's formal approval of each PR.
- Check the live Task, Epic and base state first. Neither `master` nor this branch is assumed to contain the native work.
```

## Fresh session on the candidate, from `eech-campaign/`

```text
**Loaded at startup:** `CLAUDE.md` (project instructions) and its `@AGENT.md` import. I then read `AGENT.md` and `PROJECT.md` explicitly. I didn't read `REVIEW.md`, and I loaded no memory files.

**Product and hosts:** The product is one native EECH dynamic campaign Lua DLL, EECH_DC [1]. The hosts are alternatives: EECH World [2], a Rust reference host, and DCS World [3]. DCS does not run through EECH World.

**Active Task and Epic:** The active Epic is EEDCS-5, which is M5, "Establish host interchangeability". The repo doesn't name an active Task. It says to read the live Jira state, and I haven't done that. EEDCS-4 is the M4 acceptance review. Tasks #51 and #52 are retired and are not gates.

**What authorises M5 structural work:** Two accepted records do:
- the M4 acceptance review (#31 / EEDCS-4), and
- the M5 kickoff (#35).

They open the work under EEDCS-5 from integration state `a02d489c`, with `windows-retail v1` as the authoritative baseline. The files themselves authorise nothing.

**Before moving existing behaviour:** Reproduce it on the named baseline revision. Protect it with sociable tests of the real collaborating components. Record passing behaviour, known defects, unsupported cases and unavailable prerequisites.

**What a pure refactor must prove:**
- It needs no manufactured red test; characterisation tests may pass before the move.
- The sociable tests stay green across the move.
- Preserved behaviour is asserted, and so is the new ownership contract.
- Host observations demonstrably influence the campaign, and host actions are demonstrably used.
- The same behavioural scenarios run on the old and new paths.
- Structural moves are kept separate from bug fixes and rule changes.

**PR approval:** A current formal GitHub **APPROVED** review from **@fd-starscream-bot** is mandatory, alongside passing checks and resolved blocking feedback. Comments, CI success, other approvals and agent self-reviews don't substitute. The PR author must be a different identity.

**If the M4 regression pack differs:**
- Handle the difference under REVIEW.md's baseline governance.
- Classify it as regression, approved defect correction, intentional semantic change, or expected variation.
- Preserve the old evidence and get explicit review.
- Never silently re-record outputs, widen tolerances or delete failing tests.

**May M5 structural work start now?** Yes, in principle, because M1–M4 are accepted and M5 is open. These conditions apply:
- Verify the live Jira Task and Epic first.
- Use a dedicated Task branch and PR under a non-lead identity.
- Check the branch and base SHA against the agreed target (`claude/eech-campaign-native-spike-tjkj6g`).
- Characterise behaviour before moving it.
- Run the M4 pack against each candidate.
- Get the lead's formal approval for each PR.

Server-side merge protection is unverified, so the review gate is enforced by process only.
```

(Superseded: that last line reflects candidate wording the lead rejected. See the
refresh below.)

Both candidate sessions loaded `CLAUDE.md` and its `AGENT.md` import, reported the
same entry conditions, and kept the lead approval, characterise-first and baseline
rules. Neither read the live Jira Task; that is a limit of the test environment,
not of the instructions, which tell a session to read it.

## Refresh after the lead's review (2026-10-20)

The lead's review on #73 asked that unverified protection stay an explicit merge
blocker, that the fresh sessions read REVIEW.md and the active Task/Epic, and that
the commands, revision, settings and static check be retained. The sessions above
are kept as history; this section supersedes them.

### Setup

- Candidate: `86356fb` (this PR's governance wording, merged with integration
  head `d2d4792a`). Each session ran in a detached worktree of that commit.
- Claude Code 2.1.304, cloud container, default settings, no project memory,
  no MCP servers, no prior conversation.
- Command, run once from the repository root and once from `eech-campaign/`:

  ```sh
  claude -p --allowedTools "Read,Grep,Glob" \
    --disallowedTools "Bash,Edit,Write,WebFetch,WebSearch" \
    --output-format text "$(cat eedcs-9-fresh-question.txt)"
  ```

- The question, verbatim: [eedcs-9-fresh-question.txt](eedcs-9-fresh-question.txt).
  It adds to REVIEW.md's questions a request to say what the session would do if
  asked to restructure campaign code or merge an approved PR, and which records it
  could not reach. It supplies none of the answers.
- Full transcripts: [root](eedcs-9-fresh-root.txt),
  [eech-campaign](eedcs-9-fresh-eech-campaign.txt).

### Results

| Check | Root | `eech-campaign/` |
|---|---|---|
| Loaded `CLAUDE.md` and the `AGENT.md` import | yes | yes |
| Read REVIEW.md | yes | yes |
| Named the active Epic (EEDCS-5) | yes | yes |
| Read the live Task/Epic | no: no Jira or GitHub access; said so and called the Task unknown | same |
| Would not start structural work before reading the live Task, checking the branch and base, and characterising behaviour | yes | yes |
| Refused to merge, citing unverified protection as a blocker whatever the reviews or checks | yes | yes |
| Kept lead approval, characterise-first and baseline rules | yes | yes |

Limitation: neither session could reach Jira or GitHub, so neither read the live
Task. Both said so instead of inventing one, and both treated reading it as a
precondition for structural work. This is a limit of the test environment.

### Static check

[govcheck.py](govcheck.py) is the retained script. It checks the governance files
for broken relative links, the active `@AGENT.md` import, the retired #51/#52 gate
phrases, the rules that must survive, and that unverified protection is still an
explicit merge blocker.

```sh
python3 docs/governance/evidence/govcheck.py <checkout>
```

| Tree | Result |
|---|---|
| Integration head `d2d4792a` | exit 1, 8 failures: 4 stale phrases, 4 entry conditions missing |
| Candidate `86356fb` | exit 0, 0 failures; no broken links; import active |
