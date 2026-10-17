# EECH Dynamic Campaign — One Engine, Interchangeable Worlds

[Project 8](https://github.com/orgs/flying-dice/projects/8) · [Repository milestones](https://github.com/flying-dice/eech_source_code/milestones) · [Governance Task #51](https://github.com/flying-dice/eech_source_code/issues/51)

This is the enduring multi-session charter, not a session plan or a live status
report. GitHub Issues/Projects hold work status and acceptance discussions.
[AGENT.md](AGENT.md) is the canonical operating policy;
[REVIEW.md](docs/governance/REVIEW.md) defines review and merge discipline.
[BOOTSTRAP.md](docs/governance/BOOTSTRAP.md) records the initial evidence and gaps.

## Product and intent

Deliver **one native EECH dynamic campaign Lua DLL, EECH_DC [1]**, using ports
and adapters to separate campaign behaviour from physical world simulation.
Original EECH C is the initial campaign implementation. Rust supplies the modern
public architecture, lifecycle and integration boundary; a rewrite is not required.

The same campaign product must operate under **EECH World [2]**, a Rust reference
host with its own Lua environment and physical world, and **DCS World [3]**, the
actual DCS host and physical simulator. Establish and protect the observable
reference-world baseline first; then introduce DCS and calibrate justified world
differences. Tests, structured observations and Tacview support every stage.

```text
EECH World.exe [2]                     DCS World.exe [3]
Rust reference runtime                DCS host runtime
Reference physical world              DCS physical world
Host-owned Lua environment            Host-owned Lua environment
EECH World adapter                    DCS adapter
              |                              |
              +---------------+--------------+
                              |
                 Shared Lua-facing host contract
                              |
                       EECH_DC.dll [1]
                 Native campaign / Rust boundary
                 Original campaign C initially
                 Campaign state and decisions
```

The hosts are alternatives, not layers: DCS does not run through EECH World.
Lua is the host-facing module surface, not the campaign implementation language.
The target is an in-process module, without a second Lua VM inside the DLL.
A sidecar or second campaign implementation requires an explicit project decision,
not an incidental workaround. Both hosts should use the same campaign artifact
within a compatible target; Linux and Windows builds remain one product lineage.

## Ownership and architectural boundaries

| Responsibility | Owner |
|---|---|
| Campaign entities, strategy, task generation/selection, assignment policy, logistics, objectives and progression | EECH_DC |
| Physical movement, combat resolution, terrain and execution of supported physical actions | Host world |
| Translating requests and observations between campaign concepts and a world runtime | Host adapter |
| Driving execution and supplying elapsed simulation time | Host |
| Applying campaign update semantics, scheduling rules and timers to that time | EECH_DC |
| Recording observations, exporting Tacview and evaluating assertions | Host/test tooling, without changing decisions |

Ownership is not decided by language or file location. A campaign route-selection
algorithm remains campaign behaviour even when it requires terrain data. Reusing
EECH C for reference physics is compatible with a Rust host; no fresh Rust physics
engine or complete DCS replica is mandated.

The campaign decides what it wants attempted; the host supplies the physical
attempt and its observations. The campaign applies the campaign consequences:

```text
Campaign observation -> campaign decision -> requested world action
    -> simulated attempt -> observed result -> campaign consequence
    -> subsequent campaign decision -> repeat
```

An adapter must not choose campaign priorities, replenish stocks or grant success
to compensate for missing campaign behaviour. Reference-world simplifications need
explicit rules and limitations; they may not fabricate successful test outcomes.

Product acceptance exercises **host-owned Lua -> public native module -> host
interactions**. Direct Rust/C tests are useful but not substitutes. A shared
implementation language must not give EECH World privileged access to campaign
internals. Legacy entity pointers, lists and dispatch layouts stay private.

A whole-engine Lua wrapper is useful transitional/reference scaffolding, not the
finished architecture if flight, weapons and damage remain authoritative inside
the DLL that should consume DCS observations. Record temporary coupling honestly.
Protect working behaviour before changing it; resolve host interchangeability
with evidence, not merely new interfaces or renamed crates.

No particular crate layout, ABI, callback/queue mechanism, ECS, test framework or
internal state redesign is prescribed by this charter. Implementation hypotheses
may evolve as source and runtime evidence reveal the actual boundaries.

## Historical direction: abandoned TSTL port

The [eech-core-ts port on master](https://github.com/flying-dice/eech_source_code/tree/master/eech-core-ts)
pursued the same broad campaign goal by translating C into TypeScript and
TSTL-generated Lua. **The project owner abandoned that direction because its
performance was unsuitable for the intended use and the DLL route avoids
translating campaign code that can execute natively.** This is the owner's
qualitative strategy rationale, not a newly measured benchmark or speedup claim.

Retain its source archaeology, architecture investigations, behaviour matrices,
C-reference harness, numerical findings, explicit compatibility choices and
scenarios. It is not an active roadmap, a second production implementation to
maintain, or an instruction to resume the old slice backlog. Historical coverage
supports only the behaviours actually exercised there.

Distinguish defined original-C behaviour, observed native behaviour, approved
compatibility/correction decisions and TSTL comparison results. Shared code limits
reference independence. Characterising a legacy defect does not endorse it forever;
corrections require explicit evidence and review. Unknown historical numerical
semantics remain unknown until evidence supports a decision.

## Roadmap and issue navigation

Milestones are cumulative architectural acceptance gates, not calendar estimates
or required code changes. Testing begins at M1; M4 strengthens confidence before
larger refactoring. Focused later-stage probes may run early to resolve material
risk without claiming later milestones complete. Each accepted baseline remains
protected by later work.

| Milestone | Epic | Outcome Tasks |
|---|---|---|
| [M1 — Prepare the campaign module](https://github.com/flying-dice/eech_source_code/milestone/1) | [#19](https://github.com/flying-dice/eech_source_code/issues/19) | [#51 governance](https://github.com/flying-dice/eech_source_code/issues/51), [#52 protection](https://github.com/flying-dice/eech_source_code/issues/52); then [#20](https://github.com/flying-dice/eech_source_code/issues/20), [#21](https://github.com/flying-dice/eech_source_code/issues/21), [#22](https://github.com/flying-dice/eech_source_code/issues/22) |
| [M2 — Establish EECH World](https://github.com/flying-dice/eech_source_code/milestone/2) | [#23](https://github.com/flying-dice/eech_source_code/issues/23) | [#24](https://github.com/flying-dice/eech_source_code/issues/24), [#25](https://github.com/flying-dice/eech_source_code/issues/25), [#26](https://github.com/flying-dice/eech_source_code/issues/26) |
| [M3 — Operate the campaign within EECH World](https://github.com/flying-dice/eech_source_code/milestone/3) | [#27](https://github.com/flying-dice/eech_source_code/issues/27) | [#28](https://github.com/flying-dice/eech_source_code/issues/28), [#29](https://github.com/flying-dice/eech_source_code/issues/29), [#30](https://github.com/flying-dice/eech_source_code/issues/30) |
| [M4 — Establish regression confidence](https://github.com/flying-dice/eech_source_code/milestone/4) | [#31](https://github.com/flying-dice/eech_source_code/issues/31) | [#32](https://github.com/flying-dice/eech_source_code/issues/32), [#33](https://github.com/flying-dice/eech_source_code/issues/33), [#34](https://github.com/flying-dice/eech_source_code/issues/34) |
| [M5 — Establish host interchangeability](https://github.com/flying-dice/eech_source_code/milestone/5) | [#35](https://github.com/flying-dice/eech_source_code/issues/35) | [#36](https://github.com/flying-dice/eech_source_code/issues/36), [#37](https://github.com/flying-dice/eech_source_code/issues/37), [#38](https://github.com/flying-dice/eech_source_code/issues/38) |
| [M6 — Establish the DCS integration](https://github.com/flying-dice/eech_source_code/milestone/6) | [#39](https://github.com/flying-dice/eech_source_code/issues/39) | [#40](https://github.com/flying-dice/eech_source_code/issues/40), [#41](https://github.com/flying-dice/eech_source_code/issues/41), [#42](https://github.com/flying-dice/eech_source_code/issues/42) |
| [M7 — Operate the campaign in DCS](https://github.com/flying-dice/eech_source_code/milestone/7) | [#43](https://github.com/flying-dice/eech_source_code/issues/43) | [#44](https://github.com/flying-dice/eech_source_code/issues/44), [#45](https://github.com/flying-dice/eech_source_code/issues/45), [#46](https://github.com/flying-dice/eech_source_code/issues/46) |
| [M8 — Calibrate comparable campaign outcomes](https://github.com/flying-dice/eech_source_code/milestone/8) | [#47](https://github.com/flying-dice/eech_source_code/issues/47) | [#48](https://github.com/flying-dice/eech_source_code/issues/48), [#49](https://github.com/flying-dice/eech_source_code/issues/49), [#50](https://github.com/flying-dice/eech_source_code/issues/50) |

These links are navigation, not a copied live status board. Native sub-issue and
Project membership must be verified in GitHub, not inferred from Markdown links.
The original eight Epics/24 outcome Tasks are supplemented by #51 and #52.

### M1 — Prepare the campaign module

**Intent:** Establish approximate native campaign preparation, not full extraction.
**Gate:** Reconcile current claims against a named revision; demonstrate real
campaign behaviour through a controlled public surface; expose supported lifecycle,
failures, compatibility decisions and remaining coupling. Preserve applicable
reference/conformance evidence and reproducible inputs/results.
**Protected baseline:** The candidate behaviour actually demonstrated and its limits.
**Not sufficient:** Compiling the engine, test counts, or calling a whole-engine
wrapper a completed campaign-only module.

### M2 — Establish EECH World

**Intent:** Establish an observable reference physical environment.
**Gate:** Select representative scenarios and identifiable, obtainable inputs;
demonstrate the physical capabilities they require without host-side campaign
policy; corroborate Tacview with structured observations and characterise
repeatability, approximations and observation limitations.
**Protected baseline:** Reference scenarios, world capabilities and observation fidelity.
**Not sufficient:** A Lua driver merely observing a physical world still hidden
inside the campaign DLL. Transitional scaffolding must be named as such.

### M3 — Operate the campaign within EECH World

**Intent:** Demonstrate the actual campaign/world feedback loop through Lua.
**Gate:** Real campaign decisions cause physical attempts; successful and relevant
adverse outcomes return to the campaign and affect subsequent decisions. Sustain
that loop through the public integration path, with correlated behavioural checks,
structured evidence and Tacview. Record remaining architectural coupling honestly.
**Protected baseline:** The accepted integrated scenario family and its limitations.
**Not sufficient:** Loading, spawning, moving units, a long recording, or scripts
that supply the campaign's supposed decisions and rewards.

### M4 — Establish regression confidence

**Intent:** Make the integrated baseline safe to change, not start testing late.
**Gate:** Map supported capabilities and risks to meaningful black-box checks,
including adverse paths and ongoing progression. Demonstrate regression sensitivity,
characterise expected variation, retain lower-level conformance at its true scope,
and establish reproducible evidence and explicit baseline-change review.
**Protected baseline:** A regression portfolio with expectation provenance and blind spots.
**Not sufficient:** A coverage percentage, final count, crash-free soak or regenerated
expected output alone. No arbitrary percentage, run duration or tool is mandated.

### M5 — Establish host interchangeability

**Intent:** Resolve whole-engine wrapping into replaceable physical-world authority.
**Gate:** Establish an evidenced semantic host contract; demonstrate that host
observations and action results actually drive the campaign without hidden shared
world authority. Preserve the M4 baseline across necessary refactoring, explain
all deltas and verify material DCS assumptions with targeted probes.
**Protected baseline:** A host-separated reference run and the proven shared contract.
**Not sufficient:** Renamed crates, interfaces, dependency graphs or unchanged
recordings over a still-coupled physical world.

### M6 — Establish the DCS integration

**Intent:** Build our DCS adapter, not DCS itself or a second campaign.
**Gate:** Evidence supported mappings and non-equivalent capabilities; provide
working request/observation/lifecycle translation and visible failures; verify
contract semantics and preserve the reference baseline. Separate controlled or
captured-data checks from actual DCS evidence; expose remaining prerequisites.
**Protected baseline:** The shared contract and accepted adapter scope.
**Not sufficient:** Plausible mapping documentation or mocks presented as live-host proof.

### M7 — Operate the campaign in DCS

**Intent:** Demonstrate real operation in the intended host.
**Gate:** Load the intended campaign artifact into DCS's Lua environment, sustain
real campaign-requested activity and observe actual outcomes affecting later
decisions, including adverse paths. Retain correlated Tacview and campaign evidence;
measure relevant runtime stability/performance with justified expectations.
Preserve the corresponding EECH World baseline.
**Protected baseline:** A functioning versioned DCS operating baseline and known gaps.
**Not sufficient:** A Windows build, successful require, initial spawn or activity
orchestrated by a test script. Missing DCS access is an unverified gate, not success.

### M8 — Calibrate comparable campaign outcomes

**Intent:** Obtain comparable operational dynamics, not identical physical trajectories.
**Gate:** Agree matched scenario families, measures and variation criteria before
tuning; classify differences; calibrate only justified causes; preserve shared
campaign policy and both host baselines. Repeated comparisons, tests, structured
measures and Tacview support acceptance of explained residual differences.
**Protected baseline:** A cross-host comparison suite and versioned tuning provenance.
**Not sufficient:** Matching one casualty total, forcing a familiar winner or
balancing away missing observations and incorrect commands.

## Evidence and baseline discipline

**A milestone adds evidence; it does not replace inconvenient evidence.**
Before moving behaviour, reproduce it on the named baseline and protect it with
sociable tests of real collaborating components. Investigate boundaries before
choosing implementation structure. Red/green/refactor protects behaviour, not a
preferred private type or layout. A pure refactor can start with passing
characterisation; after the move, assert both preserved behaviour and the newly
established ownership contract. See AGENT.md for the operating rules.

An accepted baseline identifies campaign/host builds and revisions, scenario and
configuration, data/asset provenance, time/random inputs, exact commands, results,
structured observations, recordings and known limits. Retain enough evidence to
survive an expired session. The [baseline record](docs/governance/BASELINE_RECORD.md)
is a template, not a storage-provider or format mandate.

Generated, synthetic and retail profiles are distinct evidence. Do not substitute
missing assets silently or publish private/retail data in issues. A changed map or
configuration is a changed experiment. Record available and unavailable inputs.

Tacview makes physical activity inspectable; assertions and structured observations
establish expectations. Neither may manufacture the other's result. Recording
must not alter decisions. Sampling losses, identity reuse, missing events and
coordinate errors limit interpretation. State whether an ACMI file was generated,
structurally validated, or actually visually inspected.

Record reported, reproduced, inferred and unavailable evidence separately.
Kernel conformance, Lua/module integration and actual DCS execution prove different
things. Shared code and limited scenarios restrict confidence. No test count,
coverage percentage or recording duration is itself a correctness oracle.

Classify meaningful changes as regression, approved defect correction, intentional
semantic change, or expected input/world variation. Preserve old evidence and
obtain review of changed expectations. Do not silently re-record outputs, weaken
assertions or widen tolerances to make a candidate pass. Known bugs may be fixed,
but the correction must be traceable and separate from a structural move.

## Host compatibility and balancing

EECH World should match the required host-integration semantics, not imitate all
DCS implementation details. Discover the shared meanings of lifecycle, capabilities,
identity, units/coordinates, time, ordering, observations and action outcomes.
Successful reference-host operation does not establish DCS compatibility: validate
material assumptions early where necessary and require actual DCS evidence at M7.

| Observed difference | Classification |
|---|---|
| Incorrect action, identity, coordinate, timing or observation mapping | Integration defect |
| Different campaign decisions from equivalent campaign inputs | Campaign/contract discrepancy |
| Different physical outcomes from different world simulators | Host-dependent behaviour to investigate |
| Deliberately changed scenario assumptions or tuning | Explicit calibration choice |

The first two are not balance issues. Equivalent-input campaign conformance is
separate from whole-war outcome similarity across different simulators. A shared
seed does not make their physics equivalent. Select operational measures from
actual scenarios, such as task completion, tempo, role-specific attrition, supply
sustainability, reinforcement or capture progression. Establish comparison criteria
before tuning; explain residual differences instead of forcing exact outcomes.

## Slicing, sessions and review

A slice is a bounded observable capability, protected behaviour, discovered
contract or resolved uncertainty, not a file, crate, language conversion or session.
Each Task states outcome, baseline, scope/non-goals, uncertainty, expected evidence
and an acceptance question. Implementation details follow investigation.

Each completed Task delivers a dedicated PR, including administration and research.
A PR can span many sessions. Agree smaller linked Tasks if the work is too large
for one coherent review; do not create an omnibus implementation PR. Replacement
PRs preserve links/history and leave one active delivery PR per Task.

The technical lead is **@fd-starscream-bot**. Every delivery PR requires that
identity's current formal **APPROVED** review, passing applicable checks and
resolved blocking feedback. Implementer and reviewer must be different PR identities.
New reviewable changes require fresh approval; material base changes require
renewed validation/review. No self-merge, self-approval, bypass or automatic
rubber-stamping is permitted. Files and review requests do not activate protection.

GitHub holds live statuses and dependency/acceptance discussions; this charter
holds intent and navigation. Use existing Project statuses without silently
reconfiguring workflow. Ready has a clear outcome and next question; Review has
evidence ready; Done means approved merge plus accepted Task evidence. Epic and
milestone acceptance require separate explicit review, not merely all children closed.

Begin with #51 and its #52 protection prerequisite. After governance is approved,
merged and accepted, #20 reassesses the native spike rather than assuming it is
new or complete. At each session boundary, record branch/commit, evidence gained,
checks actually run, discovered boundaries, baseline differences, retained artifacts,
limits/blockers, review status and next bounded action in the Task/PR.

Changing an implementation hypothesis is expected. Broadening semantic scope,
changing accepted contracts or milestone intent needs lead agreement. Later-stage
probes need a stated risk-based reason. Maps, assets, recordings, compatibility and
cleanup are work only insofar as they advance a milestone claim or resolve its risk.
A small scenario must not silently replace the long-term full campaign goal.

## Programme success and starting provenance

Success is one EECH campaign product controlling replaceable worlds through an
evidenced contract: sustained reference-world operation, actual DCS operation,
and increasingly strong tests and Tacview evidence supporting safe modernisation
and justified cross-host calibration. An executable and a DLL alone are not success.

Planning/source snapshot, checked 2026-09-24: native branch at
`b796a0fede3e97d8d02aa5880c1169f12d6fea57`. This is not milestone acceptance.
The existing whole-engine runs and narrower kernel conformance are separate evidence;
M1 must reassess them after governance rather than infer breadth from file counts.

- [Native overview](https://github.com/flying-dice/eech_source_code/blob/b796a0fede3e97d8d02aa5880c1169f12d6fea57/eech-campaign/README.md)
- [Whole-engine notes](https://github.com/flying-dice/eech_source_code/blob/b796a0fede3e97d8d02aa5880c1169f12d6fea57/eech-campaign/docs/engine.md)
- [Kernel conformance](https://github.com/flying-dice/eech_source_code/blob/b796a0fede3e97d8d02aa5880c1169f12d6fea57/eech-campaign/docs/conformance.md)
- [Recording profiles](https://github.com/flying-dice/eech_source_code/blob/b796a0fede3e97d8d02aa5880c1169f12d6fea57/eech-campaign/recordings/README.md)
- [DCS research, not live-operation proof](https://github.com/flying-dice/eech_source_code/blob/b796a0fede3e97d8d02aa5880c1169f12d6fea57/eech-campaign/docs/dcs.md)
