# M5: arrival at the four SUPPLY waypoint types, characterised

This record is part of M5 ([EEDCS-5](https://flying-dice.atlassian.net/browse/EEDCS-5)), under Task [EEDCS-11](https://flying-dice.atlassian.net/browse/EEDCS-11). The lead authorised it on [#35](https://github.com/flying-dice/eech_source_code/issues/35#issuecomment-6295796989) and in the [#74 approval](https://github.com/flying-dice/eech_source_code/pull/74) as **characterisation only**. It answers one question: **on the named baseline, how does the campaign decide and account arrival at PICK_UP, PREPARE_FOR_DROP_OFF, DROP_OFF and FINISH_DROP_OFF, and what does that decision consume from the world?** It starts from the approved ownership map, [`m5-supply-ownership.md`](m5-supply-ownership.md) (#74 at `44347ade`).

**Answer, from runtime evidence on retail Lebanon and Georgia:**
- **The verdict follows the task leader alone.** At these four types every arrival check asked the task leader for its per-member reach test, and asked no other member. That held in all 127,053 Lebanon checks and all 45,327 Georgia checks. It held in 127 multi-member arrivals: 34 Lebanon helicopter arrivals, 32 Lebanon fixed-wing arrivals and 61 Georgia helicopter arrivals. In 32 of the fixed-wing arrivals, the campaign's own all-members rule would have refused the verdict at that frame. On the same routes, the helicopter navigation legs are last-to-reach, and there the campaign did ask the other members.
- **Every consequence runs synchronously, inside the verdict's frame.** Each response reaches the leader once, through the single path `gd_updt.c` → `navigation_guide_waypoint_reached` → `response_to_waypoint_reached`. The guide has already moved on when the leader is told.
- **The accounting matches the source exactly, for both resources, at the task's own supplier and requester.**
  - A pick-up moves one crate onto the leader and takes exactly 10 off the supplier.
  - A keysite drop-off moves the first crate and sets that resource to 100.
  - A group drop-off raises each crate's resource by 1000, clamped to 100, and leaves every crate aboard.
  - FINISH_DROP_OFF sets the task to SUCCESS inside its own span.
- **Completion is not delivery, at runtime.** One Georgia task was set to SUCCESS at FINISH_DROP_OFF with nothing aboard. Three tasks were set to SUCCESS in the frame their requester group was freed, with no DROP_OFF arrival: one in Lebanon with no arrival at all, and two in Georgia after a PICK_UP.
- **Observing did not perturb.** The test-only observed build passed the whole M4 regression pack. `windows-retail v1` was IDENTICAL on Georgia and Lebanon, and the M2 recording and M3 observations were reproduced byte for byte.

This is one slice of EEDCS-11, not the Task. The live Task is broader: host-only world access, causal host observations and M4 preservation. It stays open.

Nothing here moves world authority, adds an ABI, trait or adapter, or changes a campaign rule. The product build is unchanged. The observation exists only in a test-only build, made the way the S3 mutant was made ([`m4-s3-mutation.md`](m4-s3-mutation.md)).

## Evidence labels

The labels are those of [`m5-supply-ownership.md`](m5-supply-ownership.md):
- **[R-run]** is reproduced in this record's runs. It is observed in the arrival trace, in the public path's observations, or in both.
- **[R-src]** is read in the source.
- **[I]** is inferred.
- **[U]** is unknown.

## Setup

| | |
|---|---|
| Baseline revision | Integration `74aaa168` (#73 merged), with #74's approved map merged in (`5ecfd216`). No product build input differs from `74aaa168`: no file under `crates/`, `aphavoc/`, `modules/`, `Cargo.lock`, `tools/Dockerfile` or `tools/build-windows.sh`. |
| Accepted baseline | `windows-retail v1` (Georgia and Lebanon), and the retained M2 and M3 artifacts, as the M4 pack judges them |
| Build under test | **OBSERVED, test only.** `tools/build-observed-windows.sh observed`, from commit `72451220`, plus 12 hooks. `eech_dc.dll` `ce17abcb…`, with the accepted `lua.dll` and Lua scripts ([`reference/m5-arrival-characterisation/observed-BUILD-INFO.txt`](../reference/m5-arrival-characterisation/observed-BUILD-INFO.txt)) |
| Scenarios and inputs | The M4 pack's own runs: retail Georgia (map3) and retail Lebanon (map5), seed 1, 3 simulated hours, 100 ms frames. Lebanon ran three times, on three roots: two reference runs with Tacview and observations (`observe_supply=1`), and one regression run. Every root was a private copy, checked against its manifest (Georgia 1,174 files `d373bc42…`; Lebanon 1,179 files `121a7890…`). |
| Command | `EECH_ARRIVAL_TRACE=<dir>`, then `python tools\regression-pack.py --georgia … --lebanon … --lebanon-repeat … --bin target\observed\observed-v2-bin` (18 minutes) |
| Analysis | `tools/arrival-check.py` (claims and facts) and `tools/arrival-check-selftest.py`, run on the retained traces |

### What the observer is

The hooks are in [`tools/arrival-observe/patches.json`](../tools/arrival-observe/patches.json), and the observer is [`tools/arrival-observe/eech_arrival_observe.c`](../tools/arrival-observe/eech_arrival_observe.c). The builder adds them to `build/patches.rs` and `csrc/` in a temporary worktree, after the product patches. It builds in a cargo volume of its own and stamps the output OBSERVED. `tools/arrival-observe/verify-patches.py` checks that every hook applies exactly once and keeps its original text. A1, A11 and A12 wrap a call instead; for those it checks that each wrapped call is kept exactly once.

| Hook | Where | Observes |
|---|---|---|
| A1 | `gd_updt.c`, the server guide update | Every verdict of `check_guide_reached_waypoint`, returned unchanged. A TRUE verdict on a SUPPLY task, or at one of the four types, is reported in full. The `WAYPOINT_REACHED` notification it causes is bracketed as one span. |
| A2 | `en_heap.c` `set_free_entity` | Each index's generation, so that `[index, generation]` names one entity for the whole run, and the freeing of any entity named earlier |
| A3 | `taskgen.c` `create_supply_task` | Each SUPPLY task: cargo type, supplier, requester |
| A4, A5 | `ts_msgs.c`, `ts_int.c` | A SUPPLY task receiving TASK_COMPLETED, and why; its completion state being set |
| A6–A9 | `mb_msgs.c` | Entry to each of the four responses of a mobile |
| A10–A12 | `guide.c` | Each per-member reach query that `check_guide_reached_waypoint` makes, and the answer, returned unchanged |

The observer reads state only through EECH's own getters. It sets no entity state. It calls no message handler, no random number and no function with effects. It writes JSON lines only when `EECH_ARRIVAL_TRACE` names a directory.

## Observation does not perturb

[`pack-report.md`](../reference/m5-arrival-characterisation/pack-report.md) gives **REGRESSION PACK: PASS** for the observed build, with tracing on in every run:
- 17 directly protected claims and 12 baseline-sensitive claims pass. The 8 evidence-only and 8 not-covered claims are unchanged.
- `regress.georgia.exact` and `regress.lebanon.exact` are **IDENTICAL to `windows-retail v1`**.
- `retained.recording` passes: the reference run's Tacview recording is byte-identical to M2's.
- `retained.observations` passes: its observations are byte-identical to M3's (`reference/m3-resupply-loop/`).
- `reference.observed_equals_unobserved` and `reference.repeatable` pass.

So with the hooks compiled and active, the campaign reproduced the accepted baseline exactly at each metric checkpoint, and every 1 s sample of the M2 and M3 evidence. [R-run]

An earlier observed build, without A10–A12, also passed the whole pack.

The three Lebanon traces have identical events (`summary.txt`). They came from two roots and the pack's copy of a third, so the characterisation itself is repeatable. [R-run]

**Not claimed:** this is not the current-candidate rebuild that #59 needs. The build under test carries test-only hooks. The control build from the same inputs (`eech_dc.dll` `3f234cdb…`, not run) and the accepted candidate (`da72f128…`) differ in hash. That difference belongs to #76 and #59, not to this Task.

## Claims, checked on every relevant event

The source makes these predictions ([`m5-supply-ownership.md`](m5-supply-ownership.md), stage 4–5). `tools/arrival-check.py` checks them against every relevant event. Each claim fails if a single event contradicts it. `tools/arrival-check-selftest.py` injects discrepancies into a copy of the retained trace, and the checker detects each one (`selftest-*.txt`). It injects at least one per claim, and seven more for the accounting claims (see "Accounting claims strengthened after review" below).
- **Lebanon:** all 17 injected cases, plus each of the 16 group replenishments removed on its own, plus the 2 correlation checks.
- **Georgia:** all 16 applicable cases, plus its 1 group replenishment removed on its own. The mixed-resource case does not apply: no Georgia group drop-off carried both resources.

| Claim | Lebanon | Georgia | Label |
|---|---|---|---|
| `path.single_route`: every response of the four types runs inside a guide verdict span, once per verdict, to that verdict's task leader, for its waypoint. No response ran outside a span. | 194 responses, 194 verdicts | 61, 61 | [R-run] |
| `criteria.leader_only`: at the four types, last-to-reach is off and there is no waypoint action. The radius is 100 m with a fixed-wing leader and 40 m with a helicopter leader. Every guide was `GUIDE_NAVIGATION_DIRECT`. | 194 | 61 | [R-run] |
| `members.leader_only_consumed`: every check at the four types asked the task leader, and only the task leader. Each verdict is the leader's own answer. | 194 verdicts; 127,053 checks, 0 queries of another member | 61; 45,327 checks, 0 | [R-run] |
| `leader.state_at_verdict`: at each verdict the leader is in the guide's group, not landed, and within the radius. A helicopter leader's rotors are spun up. | 194 (all within the radius: the fixed-wing pass-through never decided) | 61 | [R-run] |
| `order.guide_advances_first`: when the leader's response runs, its guide already has the next waypoint (`wp_msgs.c:413` before `:440`) | 194 | 61 | [R-run] |
| `pickup.accounting`: at the task's own supplier, with a crate there, exactly one crate leaves the supplier's stock for the task leader. The supplier's level of that resource drops by exactly 10, and its other resource is unchanged. The crate is of the task's type when the supplier has one (S3), else the first. With no crate, nothing changes. | 50 | 19 | [R-run] |
| `dropoff.accounting`: at the task's own requester, reading only the task leader's load, the requester's ammo **and** fuel equal the source's result exactly. For a keysite: the first identified crate is moved into its stock and that resource is set to 100; further crates stay aboard. For a group: every identified crate aboard raises its resource by 10 × 100, clamped to [0, 100], and every crate stays aboard. The other resource, and everything when nothing is aboard, is unchanged. | 48 | 14 | [R-run] |
| `finish.completes_success`: inside the FINISH_DROP_OFF span the task receives TASK_COMPLETED (objective message) and is set to SUCCESS | 48 | 14 | [R-run] |
| `identity.generations`: no event names an entity generation after that generation was freed | 1,640 events | 660 | [R-run] |
| `trace.complete`: the exit counters agree with the events | yes | yes | [R-run] |
| `correlation.public_path`: every keysite level change inside a PICK_UP or DROP_OFF span appears at the next 1 s observation sample. It is a `supply` event for the same keysite, in the same direction, with the task leader `track`ed there in the verdict's group. | 81 of 81 | not run (no Georgia observations) | [R-run] |

### The correlation rule, and two corrections made during development

The correlation ties each campaign effect to the public path. Two corrections were made to the rule while developing it. Both are recorded here, not hidden.
- **The sample.** The checker first looked at the sample printed with the verdict's own time. A 1-hour smoke run showed that this sample is taken before that frame's update: the change first appears one sample later. The rule now reads the next sample strictly after the verdict.
- **The size of the change.** The rule first required the sample's change to equal the span's change exactly. In Lebanon, 80 of 81 do. The other is Wujah Al Hajar at 7,154.25 s: the drop-off adds +90 ammo, but the sample shows +89.8, because a keysite usage tick (−0.2 ammo, `ks_updt.c`) fell in the same second. The claim now asserts presence, direction and attribution. The size of any difference is reported (`matched_with_other_change_in_sample`), not asserted.

## Leader-only versus all-members: what the multi-member cases show

Jester, the M3 featured chain, has one member, so its outcome cannot distinguish the rules ([`m5-supply-ownership.md`](m5-supply-ownership.md)). These runs contain 127 arrivals at the four types by groups with another valid member.

| | Lebanon | Georgia |
|---|---|---|
| Multi-member arrivals at the four types | 66 = 34 helicopter (Freelancer 12, Milkman 8, Striker 8, Vigilante 4, Phantom 1, Wolfpack 1) + 32 fixed wing (Goliath 8, Hellcat 8, Voodoo 8, Firebird 4, Tiger-Shark 4) | 61 helicopter (Phoenix 12, Lima 9, Sundown 9, Dragon 8, Cowboy 8, Cougar 5, Lancer 4, Tiger-Shark 4, Cobra 1, Eight-Ball 1) |
| Members asked by the deciding check | the task leader alone, at all 66 | the task leader alone, at all 61 |
| Task leader equals group leader | at all 66 | at all 61 |
| Arrivals where EECH's all-members rule would have refused the verdict at that frame | **32**: every fixed-wing case. The wingman was 305–5,054 m from its own guide position, beyond the 100 m radius plus one second's travel (`fw_msgs.c:409-433`). | 0 |
| Helicopter wingmen at the verdict | 124–376 m from the leader; airborne, rotors spun up | 202–367 m from the leader; airborne, rotors spun up |

All six columns of counts are [R-run]. The "would have refused" judgement is [I]: it applies the per-member test's early outs to the recorded state, and decides only "definitely fails".

**What this distinguishes.**
- **The observation consumed: distinguished for both kinds.** At the four types the campaign takes one reach observation, the task leader's, and never asks a wingman. Helicopter navigation legs on the same routes are last-to-reach, and there the other members were asked: 24 queries in Lebanon and 163 in Georgia, over 74 and 163 verdicts. [R-run]
- **The outcome, fixed wing: distinguished.** In 32 arrivals a wingman was far outside the radius, yet the verdict fired. All-members semantics would not have fired then. [R-run], with the rule's evaluation [I]
- **The outcome, helicopters: not distinguished.** EECH's all-members rule for a helicopter wingman is cohesion: within 750 m of the leader, airborne and spun up (`hc_msgs.c:766-788`). Every wingman in these runs met it, so either rule would have given the same verdict in the same frame. [I] A discriminating input needs a helicopter wingman landed, still spinning up, or more than 750 m from the leader when the leader arrives. None occurred in these 95 helicopter arrivals. Producing one needs a changed scenario or host control of a wingman; both are outside this Task. **Unknown, left for the lead** [U].

**Wolfpack's leader**, [U] in the map, is now observed. At its Beirut pick-up (verdict 1,044.09 s), the task leader was Mi-17 **87581**, the group leader. Only 87581 was asked. 87568 was 362 m away. The crate (fuel, entity 93565) went aboard 87581. [R-run]

**A correction to an attribution by proximity.**
- The M3 loss record lists two Beirut pick-ups during Wolfpack's period, at the 1,043.79 s and 1,044.79 s samples, with Wolfpack overhead (`reference/m3-supply-loss/loss-chain.json`; `m3-supply-loss.md`: "Two were picked up").
- The trace shows the first was **Phantom's** (group 87621, leader 87635, verdict 1,043.49 s, crate 93566). Only the second was Wolfpack's.
- So the lost Mi-17, 87568, carried no crate: it was not task leader, and only the leader loads. [R-run]
- This changes no accepted check. It is the risk the lead named: proximity is not task-specific proof. It is recorded for review, not edited into the M3 record.

**After the loss.** Wolfpack's task (92554) has no arrival after its PICK_UP until LAND at 2,115.85 s. That LAND ends the route, and the task is set to FAILURE (`TASK_TERMINATED_WAYPOINT_ROUTE_COMPLETE`). [R-run] Phantom's task (92540) did the same at 2,057.34 s, after a NAVIGATION arrival at 1,452.19 s; Phantom had one member at its LAND. [R-run] What moved either guide from its task route to LAND, without an arrival at PREPARE_FOR_DROP_OFF, is not observed [U]. These are the two tasks in Lebanon whose chains stop at PICK_UP.

## Ordering and lifetimes

- **One span, one frame.** A verdict, the guide's advance, the leader's response and the accounting all happen inside one `WAYPOINT_REACHED` notification, in that order. The trace's `effect` event is written after the notification returns. [R-run]
- **Lifetimes at completion.** After every FINISH_DROP_OFF (48 in Lebanon, 14 in Georgia), the guide, the task and the waypoint were still alive. Completion destroyed none of them inside the span, and nothing was freed inside any span of the four types. [R-run]
- **The guide dies at the end of the route.** In every LAND verdict that ended a route, the guide was freed inside the span while the task survived it: 48 in Lebanon, 14 in Georgia. This is the destruction `gd_nav.c:92` warns of, observed. [R-run] Tasks, waypoints and crates were freed later. Freed after being named in the Lebanon trace: 148 tasks, 48 guides, 426 waypoints and 239 crates. [R-run]
- **Identity reuse is real.** Indices were reused up to generation 181 for tasks, 179 for guides and 231 for waypoints in Lebanon. The Georgia maxima are 378, 366 and 567. A reference without a generation would name the wrong entity. [R-run]
- **Time bases differ.** At these verdicts, the host's simulated milliseconds run ahead of the session's elapsed time, by 0.07 s at 482 s and by 28.6 s at 10,501 s in Lebanon; Georgia is similar. [R-run] Why is not established [U]. The public observations use elapsed time. A future host contract must say which clock a timestamp is in.

## Routes, as flown

- **Waypoints at creation: none.** `create_supply_task` returns its task with no waypoint attached yet (all 192 Lebanon tasks and all 43 Georgia tasks). The route is attached later, at assignment. [R-run]
- **The route as flown.** Every task that reached all four types reached them in order: PICK_UP, PREPARE_FOR_DROP_OFF, DROP_OFF, FINISH_DROP_OFF. NAVIGATION legs came before and between them, and LAND came at the end. [R-run]
- **Who must arrive on each leg.**
  - Fixed-wing NAVIGATION legs had last-to-reach off, and the leader alone was asked (Lebanon: 150 verdicts, 27 of them in multi-member groups).
  - Helicopter NAVIGATION legs had last-to-reach on, and every valid member was asked (74 verdicts in Lebanon, 163 in Georgia).
  - [R-run] This confirms the map's reading of `wp_dbase.c:1032` at runtime.
- **Lebanon, 192 SUPPLY tasks.** 48 reached all four types. 2 stopped at PICK_UP (above). 142 never reached one: 107 expired unassigned (`TASK_TERMINATED_EXPIRE_TIME_REACHED`, task state unassigned), 2 lost their group, 1 was ended by its requester's destruction (below), and 32 were still open at the end. [R-run]
- **Georgia, 43 SUPPLY tasks.** 14 reached all four types, and 5 stopped at PICK_UP. 24 reached none: 23 expired unassigned, and 1 was still open at the end. [R-run]

## Completion and delivery

"Delivered" here means the requester's stock accounting moved at the DROP_OFF: a level was raised, or a crate entered a keysite's stock. Cargo merely being aboard does not count. `dropoff.accounting` checks the size of each change.

| SUPPLY task set to SUCCESS | Lebanon | Georgia |
|---|---|---|
| In its FINISH_DROP_OFF span; at its DROP_OFF a crate entered the requester keysite's stock and the level was raised | 32 | 12 |
| In its FINISH_DROP_OFF span; at its DROP_OFF the requester group's level was raised (the crates stayed aboard) | 16 | 1 |
| In its FINISH_DROP_OFF span; **nothing aboard at its DROP_OFF, requester stock unchanged** | 0 | **1** |
| **Outside any arrival span** (objective message), **no DROP_OFF arrival**, requester group freed in the same frame | **1**: task 100055, requester "Archer", 10,078.98 s, no arrival at all | **2**: tasks 65497 and 65469, at 1,467.39 s and 2,913.88 s, each after a PICK_UP |

All counts are [R-run]. For the last row, the sender is not recorded by the trace. That the requester's destruction sent the message (`group.c:1117`: a group being killed completes each task that depends on it, with the objective message) is [I], from the same-frame freeing and the source.

Other runtime outcomes:
- **Pick-up without a crate.** In Lebanon 1 pick-up found no crate at the supplier, and in Georgia 1. The task carried on empty. One Georgia pick-up found no crate of the task's type and took the first crate (the S3 fallback). [R-run]
- **Leftover crates.**
  - A keysite drop-off delivers only the first crate aboard. 12 Lebanon drop-offs had 1 or 2 more crates that stayed aboard, and 1 in Georgia had one more. [R-run]
  - Those extra crates were aboard because a group drop-off leaves its crate on the aircraft (16 group drop-offs in Lebanon, 1 in Georgia). [R-run]
- **Group drop-offs, exactly.** All 17 left the group's crate resource at 100 and its other resource unchanged.
  - 12 Lebanon drop-offs had 1 fuel crate aboard, 3 had 2 fuel crates, and 1 (span 242) had an ammo and a fuel crate, raising ammo 90 → 100 and fuel 0 → 100.
  - The Georgia drop-off had 1 fuel crate (60 → 100).
  - Each is the source's +1000 per crate, clamped. [R-run]

## Known campaign defects observed (recorded, not fixed)

None is changed here. Each is a candidate for `docs/corrections.md` review, as the map lists.

`dropoff.accounting` asserts defects 2 and 3 exactly as the source behaves today, because they are part of the characterised baseline. A correction would turn the claim red. That red would be an intentional semantic change, to classify under REVIEW.md. It is not an endorsement of the current behaviour.
1. **SUPPLY success does not require delivery.** It is now observed at runtime on two paths: FINISH_DROP_OFF with nothing aboard, and the requester group's destruction. [R-run]; the second path's sender is [I]
2. **A keysite drop-off delivers only the first crate aboard.** This is now observed at runtime. [R-run]
3. **A group drop-off leaves the crate aboard**, to be delivered again later. Seen at M3, and counted here. [R-run]

## Record defect (separate from the campaign defects)

**The M3 loss record attributes two Beirut pick-ups to Wolfpack by proximity.** They belong to two groups: the first was Phantom's (see above). [R-run] This is a defect of an accepted record, not of the campaign. It stays visible here; the retained M3 baseline is not rewritten.

## Accounting claims strengthened after review

The lead's review of `988342e0` found that `dropoff.accounting` accepted a group drop-off with no replenishment at all. The lead reverted the effect's levels for all 17 group drop-offs in the retained traces, and both still passed every claim. The group branch rejected only a lower level, looked only at the first crate's resource, and counted a task as delivered when cargo was merely aboard.
- **Red.** The new self-test cases were committed before the checker changed (`f327d40f`), and run against the unchanged checker. `red-selftest-lebanon.txt` and `red-selftest-georgia.txt` show 7 and 6 of them undetected:
  - no group replenishment (the lead's reproduction);
  - half a replenishment;
  - a mixed ammo and fuel drop-off losing one increment;
  - the unaffected resource moving;
  - the wrong destination group;
  - the other resource moving at a keysite drop-off;
  - the other resource moving at a pick-up.
- **Green.** Both accounting claims now compute the source's exact result for both resources, every identified crate and the clamp. They tie the effect to the task's own supplier or requester, and to the task leader's load and the crates' identities. Every case is now detected, and each of the 17 real group replenishments removed on its own is caught.
- **What did not change.** The retained traces and every accepted output are unchanged, and no tolerance moved. The regenerated reports pass all claims on both maps. No new defect was exposed.

## Tacview

The reference runs **generated** Tacview recordings.
- They were **structurally checked**: `observation-check.py` passed on both runs, and run 1's recording is byte-identical to M2's retained `reference/lebanon-3h/recording.zip.acmi` (pack: `M2.tacview_observations`, `retained.recording`).
- They were **not visually inspected** for this record.
- The recording carries no arrival, task or leader identity. The trace and the observations carry those.

## Limits and unknowns

- **Helicopter outcome discrimination** is open (above) [U].
- **Human players** are not exercised: `player_skip_waypoint`, and the AI-only landed and spin-up gates [U].
- **Coverage.** One seed, two maps, 3 hours each. The four types are covered only on AI fixed-wing and helicopter SUPPLY tasks, not every use of those enum values. No other task type used them in these runs (counter `other_four` = 0). [R-run]
- **Order inside the span.** The force's `FORCE_WAYPOINT_REACHED` (a no-op, `fc_msgs.c:978-1014`) is not observed. Whether entity update order within a frame matters to a verdict is [U].
- **Unobserved route change.** Why Wolfpack's and Phantom's guides left their task routes is [U].
- **Not observed by the public path.** The trace is test-only evidence. The public path still exposes none of these identities (`engine:objects ()` is unchanged). The causal public-path tests the lead requires for a structural slice are not attempted: reference provider, withheld PICK_UP, delayed DROP_OFF, missing provider.
- **Builds.** The evidence comes from the observed build only. Its non-perturbation is shown by the M4 pack, not by comparing machine code.

## The characterised seam, for the structural-slice decision

For the four SUPPLY types, on this baseline:
- **Consumed:** one per-member reach observation, for the task leader the campaign names, against the waypoint and radius the campaign names. It is consumed once per frame while the waypoint is current, and synchronously. [R-run]
- **Kept by the campaign:** waypoint selection, criteria, the choice of member, the combination rule, and every consequence. [R-run], [R-src]
- **Required of a future boundary** (all [R-run]):
  - Generation-safe identities for task, guide, waypoint and member.
  - A synchronous answer within the frame. The guide advances before the leader's response.
  - Lifetimes: nothing is freed inside the four types' spans, but the guide is freed inside the LAND verdict that ends its route.
  - A rule for which clock a timestamp is in.

A structural slice that moves this seam must keep every claim above green, on the same scenarios, and add the lead's causal public-path tests. That needs a separate authorisation.

## Retained evidence

In [`reference/m5-arrival-characterisation/`](../reference/m5-arrival-characterisation/):

| File | What it is |
|---|---|
| `arrivals-lebanon.jsonl.gz`, `arrivals-georgia.jsonl.gz` | The arrival traces: Lebanon reference run 1, and the Georgia regression run |
| `report-lebanon.json`, `report-georgia.json`, `check-*.txt` | `arrival-check.py` on them: claims, facts and correlation |
| `selftest-*.txt` | `arrival-check-selftest.py` on them |
| `red-selftest-*.txt` | The same self-test's accounting cases, run against the checker before the review fix (red) |
| `pack-report.md`, `pack-report.json` | The M4 pack's report for the observed build |
| `observed-BUILD-INFO.txt`, `control-BUILD-INFO.txt`, `observation.diff`, `hook-markers.txt` | The builds' identities, the exact hook diff, and the hooks found compiled |
| `summary.txt` | Commands, roots, trace hashes and repeatability |

## Reproduce

```sh
# Git Bash, Docker: the test-only build (from a clean commit)
tools/build-observed-windows.sh observed target/observed/observed-bin
```

```powershell
# PowerShell, Windows, retail roots as for the M4 pack
$env:EECH_ARRIVAL_TRACE = '<an existing absolute directory>'
python tools\regression-pack.py --georgia <root> --lebanon <root> --lebanon-repeat <second root> --bin target\observed\observed-bin --out target\observed\pack
python tools\arrival-check.py <dir>\arrivals-<pid>.jsonl --observations target\observed\pack\reference\run1\observations.jsonl --report report.json
```

The retained checks need no retail data (`summary.txt`).
