# Governance bootstrap: evidence and remaining gates

This is a dated setup record, not live project status. Consult
[#51](https://github.com/flying-dice/eech_source_code/issues/51),
[#52](https://github.com/flying-dice/eech_source_code/issues/52) and their PRs.

## Starting point inspected on 2026-09-24

Target: `claude/eech-campaign-native-spike-tjkj6g`.
Base commit: `b796a0fede3e97d8d02aa5880c1169f12d6fea57`.
Base tree: `50d053cb300a1dc5a3cc86211d037f118a62427a`.

The root tree has no AGENT.md, AGENTS.md, CLAUDE.md or PROJECT.md. The .github
subtree contains the existing native and TSTL workflows, but no CODEOWNERS or PR
template. These governance files are additions; engine sources, existing tests,
fixtures, recordings and workflows are not changed by this bootstrap.

The supplied project charter and M1-M8 issue plan are the source for PROJECT.md.
Issue #51 supplies the operating/review requirements. These documents state the
intended architecture, not a claim that the present whole-engine Lua wrapper has
already separated campaign and physical-world authority.

## Checks and limits

The bootstrap PR records the actual validation commands/results and exact candidate
SHA. Static instruction-import, local-link, roadmap and scope checks can validate
these additions. They do not prove that Claude loaded the policy, that gameplay
still passes, or that GitHub requires the named review.

No campaign behaviour changes are intended. No gameplay test failure is invented
for this administrative change. Existing engine/conformance suites must not be
claimed as rerun unless they were actually executed. Any later runtime change
requires its own baseline characterisation and Task/PR evidence.

## Protection gap: #52

At inspection, GitHub's branches listing reports the native target and `master`
as `protected: false`. The repository rulesets listing with
`includes_parents=true` returned no rulesets. The connected identity is
`fd-starscream-bot`, with `write` permission, not repository administration.
No repository setting is changed by adding the proposed CODEOWNERS file.

Activation and effective verification remain **blocked on administrator action**
in #52. See [REVIEW.md](REVIEW.md) for required outcomes and safe verification.
Existing workflows are retained, not automatically promoted to guessed required
checks. Proposed ownership is repository-wide with the technical lead as sole owner.

## Bootstrap author/reviewer conflict

The connection preparing this change is `fd-starscream-bot`. A PR it opens has the
same author as the mandatory approver; GitHub cannot accept self-approval. Such a
PR is a prepared, unmerged handoff, not an accepted governance delivery.

An independently authenticated implementation contributor must open a replacement
PR from the prepared branch (linking the superseded PR and #51), so that
fd-starscream-bot can review the actual candidate independently. Creating that PR
requires coordinating the superseded one; do not change commit authors to fake
independence. Do not relax review policy, self-merge or use someone else's credentials.

## Claude discovery gap

A Claude executable is not available in the preparation environment. The import
syntax and paths can be checked, but a fresh Claude session from the root and
native workspace has **not been demonstrated**. Follow REVIEW.md's discovery
check on the candidate and retain version/settings, loaded files and responses.
Do not call #51 complete solely because CLAUDE.md contains a valid import.

## Acceptance remains explicit

The files require formal lead review, an approved integration merge, verified
protection and instruction-discovery evidence before #51 can be closed and #20
begins programme work. This document creates no bootstrap exemption and grants
no authority to merge. Update live evidence in the Task/PR and preserve this
starting snapshot rather than rewriting its history to imply prior success.
