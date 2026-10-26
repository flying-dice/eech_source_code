# Independent Codex delivery source review — EEDCS-42

Reviewed candidate `a7863a73fa8bfba56cfa6bac4bb1602e9b9b88eb` against accepted integration `a7934aa7d1bcffc2ea5057bb3f1b0af332606c61` on 26 October 2026. One independent Codex reviewer, one read-only source pass and focused evidence inspection; no nested agents, implementation, duplicate builds or formal GitHub approval.

## Finding

P3: `eech-campaign/crates/eech-dc/src/module.rs:567–574` declares `line_of_sight` and `sighter` twice inside the `dependent-fixture` feature. The first declarations are unused, so this supported test feature introduces unused-variable/unused-mut warnings and would fail Clippy with `-D warnings`. Remove one declaration of each. This does not affect default shipping execution. Determined from source; the reviewer did not run another build.

## Source disposition

No blocking shipping-source findings. The reviewer traced both campaign LOS checks through Lua and confirmed the legacy solver is excluded from hosted builds; checked host arithmetic, eye lift, sample/phase order, horizontal range versus 3D direction and strict target-end `>144` exclusion; checked semantic values and optional existing host IDs without pointers or heap indices crossing; checked error/entry cleanup and FPU restoration; and confirmed terrain/collision coupling remains declared with targeting policy in the campaign.

The reviewer independently compared all seven candidate40 trace lines with the retained original: identical, including 4,567 queries and 107,162 ordered lookups.

## Finding resolution and focused follow-up

The same reviewer checked new source changes at `61e85664540770ab10e3ac7e9f5dfa916291c52f`.
The source delta from a7863a73f removes exactly the two duplicate declarations; the other changes
are documentation/evidence. P3 is resolved, with no new change-specific correctness or boundary
finding. All-fixture and old-guard Clippy full logs and exit records were inspected: both
`CARGO_EXIT=0` and `DOCKER_EXIT=0`. This was a focused check of the new correction, not a second
full source-review pass. Retained symbol output had trailing spaces on two lines; the readable
copy was trimmed and its original output preserved outside Git.

The lead independently compared completed 900-second A/B artifacts by length and SHA-256:
Tacview 11,363,845 bytes, observations 6,629,273 bytes and metrics 11,777 bytes all match.
The exact independent report is retained under
`eech-campaign/reference/d2-los-observation/extraction/ab/independent-comparison.json`.
Both candidate600 cases and public-path causal/refusal checks completed as recorded in the
extraction evidence. The final ordinary-feature shipping build from clean 61e856645 passed;
its full pack completed all40 checks PASS. The supplied-binary destination baseline completed exit0/all40checks
PASS. All6 complete reference files independently match accepted D1 bytes/hashes, and all3 input
manifests still match after the run. The baseline completion/report/comparison records are in
`eech-campaign/reference/d2-los-observation/extraction/destination-baseline`.

## Limits and pending acceptance

Both final candidate reference runs have completed. The lead independently checked all six
complete Tacview/observation/metric files against accepted D1 sizes and SHA-256; all agree.
Record: `eech-campaign/reference/d2-los-observation/extraction/candidate-reference-comparison.json`.
Georgia and Lebanon regression and both reference runs are complete. All eight complete scenario
files match the supplied baseline, including both regression metrics. All three post-run input
manifests match (actual Python exits0).

The original runner exited1 on the report writer's final display line (`ntpath.relpath` between
R: output and C: cwd), after writing both PASS reports. A separate report-only invocation of
the unchanged writer exited0 using the original40 results and an explicit R: native cwd. It
called no scenario/check execution function and wrote to a separate directory. The same
independent reviewer inspected this new recovery, compared regenerated claims/checks/identity
and independently rehashed the original reports, console and eight scenario artifacts: no
blocking evidence issue. The original exit remains1; recovery is not relabelled as a successful
original process. See `eech-campaign/reference/d2-los-observation/extraction/final-pack/`.

No overall delivery acceptance yet. Pending: current final-evidence-head CI, normal merge
and Jira/Confluence delivery records. The earlier900-second A/B comparison and partial prefixes
are superseded by the completed full scenario comparisons; no coverage claim was expanded.

The user directs the lead to use the logged-in GitHub identity and manage normal merges autonomously. No authentication changes, copied credentials or fabricated GitHub self-approval.
