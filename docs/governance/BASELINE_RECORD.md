# Baseline and change-evidence record

Use this structure in a Task, PR or retained evidence document when useful. It is
not a requirement to create a separate file for every edit. Apply the
[operating policy](../../AGENT.md) and [review process](REVIEW.md).

## Identity and scope

- Task, Epic, milestone and acceptance question:
- Baseline branch/commit, candidate commit and intended merge base:
- Campaign/host versions, platform, build flags and relevant compatibility choices:
- Behaviour under examination and supported/unsupported scope:
- Scenario/configuration, data provenance and accessible input identities:
- Simulated time, time-driving rules and random inputs:

## Characterisation and expectations

Explain what the old system demonstrably does and where expectations come from:
defined C behaviour, a previous accepted baseline, an explicit compatibility
choice, an approved defect correction, or additional TSTL comparison evidence.
State uncertain historical semantics and known defects separately. Current output
is an observation, not automatic approval of that behaviour.

Identify real collaborating components and the genuine environmental boundaries
controlled by the tests. Explain which observable results/required interactions
are protected without locking down private implementation details.

## Execution evidence

| Phase | Exact command and revision | Actual result | Retained evidence |
|---|---|---|---|
| Baseline characterisation | | | |
| Meaningful red check, or justified not applicable | | | |
| Green candidate / sociable tests | | | |
| Post-change boundary assertion | | | |
| Applicable regression/conformance | | | |
| Integrated public Lua/module path, if affected | | | |

List checks not run, missing prerequisites and their impact. State reported versus
reproduced results. Private kernel replay, Lua integration and actual DCS execution
are separate claims; do not infer one from another.

## World observation, when applicable

Link Tacview and corresponding structured observations to the same run/inputs.
State whether recordings were generated, structurally checked or visually reviewed.
Record sampling/identity/coordinate limitations and the interpretation they permit.
Retain inputs and artifacts beyond an ephemeral session; respect private/retail data.
Do not produce synthetic physical activity just to fill this section.

## Differences and acceptance

Classify every meaningful delta: regression, defect correction, intentional semantic
change, or expected input/world variation. Explain the cause and retain the previous
expectation. Normalisation or tolerance changes need rationale; do not silently
regenerate a passing golden output. Distinguish repeatability within one setup
from cross-platform or cross-host equivalence.

Record the accepted result, residual risk, supported breadth, formal lead-review
link and approved merge when available. Before approval these are proposals, not
accepted baselines. Finish with the next bounded question and reproduction/rollback.
