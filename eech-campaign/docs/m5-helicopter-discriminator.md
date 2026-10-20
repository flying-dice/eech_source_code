# M5: the helicopter arrival rule, discriminated

This record belongs to EEDCS-11 ([EEDCS-11](https://flying-dice.atlassian.net/browse/EEDCS-11), Epic [EEDCS-5](https://flying-dice.atlassian.net/browse/EEDCS-5)). It is a **separate, test-only discriminator**: the lead authorised it in review 5590065336 on #77, in the #35 summary, and in EEDCS-11 comment 10422. It builds on the characterisation in [`m5-arrival-characterisation.md`](m5-arrival-characterisation.md) (#77, approved at `ab279d44`). It is not a structural slice.

**The question.** In #77, every helicopter wingman stayed within 750 m of its leader. So at the four SUPPLY waypoint types, the campaign's leader-only rule and its all-members rule (helicopter cohesion: within 750 m of the leader, airborne, rotors spun up) gave the same outcome. Which rule decides when they would differ?

**The answer, from controlled runs on retail Lebanon and Georgia:**
- **At PICK_UP, PREPARE_FOR_DROP_OFF, DROP_OFF and FINISH_DROP_OFF the verdict is leader-only.** [R-run]
  - In all 99 trials, the verdict fired while a real AI wingman was held 1,500 m from the leader.
  - Only the task leader was asked.
  - Where the same waypoint exists in the ordinary run (36 of the 99), the verdict came in the very same frame, with no delay.
  - Accounting and completion followed as in the ordinary runs.
- **On helicopter NAVIGATION legs (last-to-reach on) the verdict is all-members.** [R-run]
  - In 137 of 140 trials, the verdict was withheld at every frame where the leader alone would have arrived.
  - The check asked the wingman, which answered no.
  - The verdict fired in the first frame the wingman was back within 750 m: at 742.8 to 750.0 m, after the hold ended.
  - The other 3 trials were cut off by the end of the run.
- **So the same controlled input separates the two rules, and the campaign applies each rule where its source says it does.** The input under control is one genuine physical one: the wingman's position. That position decides the outcome only where the campaign's own criteria ask for it.

## The input, and what is not controlled

**Builds.** The test-only CONTROLLED build (`tools/build-observed-windows.sh controlled`, commit `a03b242f`) is the #77 OBSERVED build plus one hook:
- **A13** (`tools/arrival-observe/patches-control.json`) calls `tools/arrival-observe/eech_arrival_control.c` just before each server guide arrival check (`gd_updt.c`).
- `eech_dc.dll` is `3498393d…`; the Lua and `lua.dll` are the accepted ones (`reference/m5-helicopter-discriminator/controlled-BUILD-INFO.txt`, `observation.diff`, `hook-markers.txt`: A1–A13 compiled).
- The control is **dormant** unless `EECH_ARRIVAL_CONTROL` is set.

**The input.** `EECH_ARRIVAL_CONTROL=hold:<four|navigation>:1500:1000:120`. A trial starts on a SUPPLY task's guide when all of these hold:
- its task leader is an AI helicopter;
- it has another valid AI helicopter member that is airborne;
- its current waypoint is of the chosen kind;
- the leader is within 1,000 m of its guide position.

During the trial, before each check, every such wingman is held 1,500 m from the leader, horizontally, on the side away from the guide position, at the leader's height above the terrain. The hold lasts until the waypoint changes or for 120 simulated seconds.

The wingman is moved through EECH's own aircraft position setter, `set_local_entity_vec3d (…, VEC3D_TYPE_POSITION, …)`. That is the path the flight model takes, and it keeps sectors and movement dependents consistent.

**Not controlled.** The control never touches any of these; the campaign's own code ran them unchanged:
- the verdict;
- last-to-reach or any other criterion;
- the guide, the task or the group;
- who is task leader;
- which members are asked;
- any handler or accounting.

The observer recorded each check of a trial guide (`trial_check`): the leader's state, each other member's distance to the leader, who was asked and their answers, and the verdict. It also recorded the #77 events (verdict, handler, effect, completion).

No product build input changes: `crates/`, `aphavoc/`, `modules/`, `Cargo.lock`, the Dockerfile and `build-windows.sh` are identical to integration `74aaa168`.

## The paired ordinary case, and baseline preservation

The same CONTROLLED build, with the control **off** and tracing on, ran the M4 regression pack (`paired-ordinary-pack-report.md`):
- **PASS.** 17 directly protected and 12 baseline-sensitive claims pass.
- **`windows-retail v1` IDENTICAL** on Georgia and Lebanon.
- The M2 Tacview recording and the M3 observations are reproduced byte for byte.
- [R-run]

Its traces have **events identical** to #77's retained ordinary traces: Lebanon from all three runs, and Georgia. The event-content SHA-256 values are `21b19d60…` and `4563072d…`. [R-run] So with the control dormant, the controlled build is the ordinary case, exactly.

**The changed scenarios** are identified on their own, never as `windows-retail v1`: `run-*.txt`, `artifact-hashes.txt`.

| Scenario | Root (verified against its manifest) | Control |
|---|---|---|
| `lebanon_retail-four` | Lebanon, 1,179 files `121a7890…` | `hold:four:1500:1000:120` |
| `lebanon_retail-navigation` | Lebanon, a separate copy | `hold:navigation:1500:1000:120` |
| `georgia_retail-four` | Georgia, 1,174 files `d373bc42…` | `hold:four:1500:1000:120` |
| `georgia_retail-navigation` | Georgia, a separate copy | `hold:navigation:1500:1000:120` |

All four used seed 1, 3 simulated hours and 100 ms frames, with Tacview (`record_every=10`), observations (`observe_supply=1`) and metrics. Their metrics are retained, and are not compared with the baseline: they are different experiments.

## Results

`tools/helicopter-discriminate.py` classifies each trial from its outcome against the recorded state. A **decisive frame** is a check where:
- the leader meets its own reach criteria: not landed, rotors spun up, range to its guide position within the radius;
- while a valid wingman fails the all-members per-member test: landed, spinning up, or more than 750 m from the leader.

If the verdict fires at a decisive frame, the trial is *leader-only*. If the verdict is withheld at every decisive frame and fires at the first later frame where every wingman passes, the trial is *all-members*. The members the check asked are reported beside the classification, not used for it.

| Run | Trials | Rule (by outcome) | Asked at the verdict | Wingman at the verdict | Paired with the ordinary run |
|---|---|---|---|---|---|
| Lebanon, four types | 39 (12 PICK_UP, 9 PREPARE, 9 DROP_OFF, 9 FINISH) | **39 leader-only** | the leader alone, at all 39 | 1,500 m | 6 matched, 6 with no delay |
| Georgia, four types | 60 (18, 14, 14, 14) | **60 leader-only** | the leader alone, at all 60 | 1,500 m | 30 matched, 30 with no delay |
| Lebanon, NAVIGATION (last-to-reach) | 34 | **32 all-members**, 2 withheld (the run ended while held) | leader and wingman, at all 32 | 743.3–749.7 m | 5 matched, delayed 109–354 s |
| Georgia, NAVIGATION (last-to-reach) | 106 | **105 all-members**, 1 inconclusive (the run ended before the leader arrived) | leader and wingman, at all 105 | 742.8–750.0 m | 2 matched, delayed 111–230 s |

All counts are [R-run]. "Paired" matches the trial's waypoint, by `[index, generation]`, to the ordinary run's verdict for that waypoint. The controlled scenario diverges after its first hold, so later waypoints may not exist in the ordinary run; the unmatched trials are listed in `discriminate-*.json`.

**A worked trial.** Lebanon, NAVIGATION, trial 1, leader 87608, wingman 87595, radius 150 m:
- The hold starts at 990.8 s, with the leader 933 m from its guide position.
- From 1,014.8 s the leader is inside the radius, and every check asks the leader (yes) and then the wingman (no, 1,500 m away).
- The hold ends at 1,110.8 s. The wingman flies back: 1,228 m, then 944 m, then 755 m.
- At 1,121.9 s it is at 748.8 m. The wingman answers yes, and the verdict fires in that frame.
- In the ordinary run the same waypoint was reached at 1,012.7 s, 109.2 s earlier.

**Accounting and completion in the trials** [R-run], from the retained traces:

| Waypoint | Lebanon | Georgia |
|---|---|---|
| PICK_UP | 11 crates onto the leader; 1 supplier had no crate | 17 crates onto the leader; 1 supplier had no crate |
| DROP_OFF | 6 keysite and 1 group stock raised; 1 keysite and 1 group unchanged (nothing aboard, or already full) | 12 keysite and 1 group stock raised; 1 keysite unchanged |
| FINISH_DROP_OFF | SUCCESS in all 9 | SUCCESS in all 14 |

`tools/arrival-check.py` passes **every #77 claim** on all four controlled runs, including the exact pick-up and drop-off accounting and `members.leader_only_consumed`, and its public-path correlation with each run's own observations (`arrival-check-*.json`). So holding a wingman away changed neither the four types' accounting rule nor the members consulted.

## The discriminator detects each rule (red/green)

`tools/helicopter-discriminate-selftest.py` re-writes each classified trial as the other rule would have played it, using the trial's own recorded state:
- a leader-only trial gets its verdict withheld at every decisive frame;
- an all-members or withheld trial gets its verdict at the first decisive frame.

The classifier must then change its answer, and the unchanged trace must classify as before (`selftest-*.txt`):
- Lebanon, four types: 39 leader-only → withheld, 39 of 39.
- Georgia, four types: 60 → withheld, 60 of 60.
- Lebanon, NAVIGATION: 32 all-members and 2 withheld → leader-only, 34 of 34.
- Georgia, NAVIGATION: 105 → leader-only, 105 of 105; the 1 inconclusive is counted, not re-written.

All four PASS. A 1-hour smoke of each Lebanon mode, run first, gave the same split (6 leader-only, 5 all-members).

## Tacview

All four controlled runs **generated** Tacview recordings. Each was **structurally checked**: `tools/observation-check.py` PASS against that run's own observations and metrics (`observation-check-*.json`), so the 1,500 m holds did not break the check's continuity or identity rules. They were **not visually inspected**.

The recordings and observations (115–185 MB each) are **not retained**; their SHA-256 and sizes are in `artifact-hashes.txt`. They come from changed scenarios, so they are evidence for this experiment only.

## Labels, limits and unknowns

- **Reproduced:**
  - leader-only at the four types and all-members at helicopter NAVIGATION legs, by outcome, under the controlled input;
  - the members asked;
  - zero delay at the four types against the paired ordinary run;
  - accounting unchanged.
- **Inferred** [I]:
  - That a helicopter *landed* or *spinning up* wingman would act like a distant one at these waypoints. Only distance was controlled. The source puts landed and spin-up in the same per-member test, and that test is never reached at the four types.
  - That fixed-wing groups follow the same leader-only rule at the four types. #77's 32 fixed-wing cases show it by recorded state, but this control was not applied to fixed wing.
- **Unknown** [U]:
  - Human players. The control holds AI wingmen only, and `player_skip_waypoint` and the AI-only gates are not exercised.
  - Other seeds.
  - Hold distances other than 1,500 m. The rule's threshold is the source's 750 m; it is observed at 742.8–750.0 m, not swept.
  - Other waypoint types.
- **Scope.**
  - This is a test-only experiment. The control is not a host input and not a product ABI, and nothing here authorises moving world authority.
  - The control is a new test-only hook, the same kind as #77's observation hooks. It touches only a member's physical position: not a verdict, a criterion or a member choice.
- **Classification of future defect fixes.** Comment 10422 says that a future fix to the known supply defects is classified under the existing review process. Such a fix is not automatically an intentional semantic change. The sentence in [`m5-arrival-characterisation.md`](m5-arrival-characterisation.md) that calls such a fix one is superseded by that ruling. That file belongs to #77 and is not edited here.

## Retained evidence

In [`reference/m5-helicopter-discriminator/`](../reference/m5-helicopter-discriminator/):

| File | What it is |
|---|---|
| `trace-<scenario>-<four or navigation>.jsonl.gz` | The four controlled runs' arrival traces, including every trial check |
| `discriminate-*.json`, `discriminate-*.txt` | The per-trial classification, paired with #77's retained ordinary traces |
| `selftest-*.txt` | The discriminator self-test on each trace |
| `arrival-check-*.json` | #77's claims on each controlled run, with its own observations |
| `observation-check-*.json` | The Tacview, observation and metrics structural check of each controlled run |
| `metrics-*.json`, `run-*.txt`, `artifact-hashes.txt` | Each changed scenario's identity, metrics and the hashes of its unretained recording and observations |
| `paired-ordinary-pack-report.md`, `.json` | The M4 pack on the controlled build with the control off |
| `controlled-BUILD-INFO.txt`, `observation.diff`, `hook-markers.txt` | The build's identity, the exact hook diff, and the hooks found compiled |

## Reproduce

```sh
tools/build-observed-windows.sh controlled target/observed/controlled-bin   # Git Bash, Docker, clean commit
```

```powershell
# the paired ordinary case: the pack with the control off
$env:EECH_ARRIVAL_TRACE = '<dir>'; python tools\regression-pack.py --georgia <root> --lebanon <root> --lebanon-repeat <root> --bin target\observed\controlled-bin
# the controlled runs: one verified root copy each
tools\discriminator-windows.ps1 -Bin target\observed\controlled-bin -Out <dir> -Runs @('lebanon_retail|<root>|hold:four:1500:1000:120', 'lebanon_retail|<root 2>|hold:navigation:1500:1000:120', 'georgia_retail|<root 3>|hold:four:1500:1000:120', 'georgia_retail|<root 4>|hold:navigation:1500:1000:120')
```

```sh
# from the retained files, no retail data
cd reference/m5-helicopter-discriminator
python ../../tools/helicopter-discriminate.py trace-lebanon_retail-four.jsonl.gz --paired ../m5-arrival-characterisation/arrivals-lebanon.jsonl.gz
python ../../tools/helicopter-discriminate-selftest.py trace-lebanon_retail-navigation.jsonl.gz
```
