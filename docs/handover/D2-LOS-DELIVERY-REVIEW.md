# Independent Codex delivery source review — EEDCS-42

Reviewed candidate `a7863a73fa8bfba56cfa6bac4bb1602e9b9b88eb` against accepted integration `a7934aa7d1bcffc2ea5057bb3f1b0af332606c61` on 26 October 2026. One independent Codex reviewer, one read-only source pass and focused evidence inspection; no nested agents, implementation, duplicate builds or formal GitHub approval.

## Finding

P3: `eech-campaign/crates/eech-dc/src/module.rs:567–574` declares `line_of_sight` and `sighter` twice inside the `dependent-fixture` feature. The first declarations are unused, so this supported test feature introduces unused-variable/unused-mut warnings and would fail Clippy with `-D warnings`. Remove one declaration of each. This does not affect default shipping execution. Determined from source; the reviewer did not run another build.

## Source disposition

No blocking shipping-source findings. The reviewer traced both campaign LOS checks through Lua and confirmed the legacy solver is excluded from hosted builds; checked host arithmetic, eye lift, sample/phase order, horizontal range versus 3D direction and strict target-end `>144` exclusion; checked semantic values and optional existing host IDs without pointers or heap indices crossing; checked error/entry cleanup and FPU restoration; and confirmed terrain/collision coupling remains declared with targeting policy in the campaign.

The reviewer independently compared all seven candidate40 trace lines with the retained original: identical, including 4,567 queries and 107,162 ordered lookups.

## Limits and pending acceptance

No overall delivery acceptance yet. Pending: correction and affected-feature check; final corrected-head provenance; complete candidate600 comparisons and public-path causal/refusal evidence; complete supplied-binary baseline and shipping candidate packs; full-file Tacview/observation/metric comparisons; current CI; normal merge and Jira/Confluence delivery records. Partial prefixes and active processes do not pass those gates.

The user directs the lead to use the logged-in GitHub identity and manage normal merges autonomously. No authentication changes, copied credentials or fabricated GitHub self-approval.
