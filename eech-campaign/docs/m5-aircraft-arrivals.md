# M5: host-observed radius arrival for non-SUPPLY aircraft (EEDCS-30)

**Scope.** One physical extraction toward criteria 2 and 3 of the
[M5 goal](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337),
version 2 (read live 2026-10-21 20:47 UTC; all six criteria remain unaccepted),
on integration `5469adf7`: a fixed-wing or helicopter member's arrival at a
non-SUPPLY waypoint whose only physical criterion is RADIUS (alone or with
LAST_TO_REACH) is now answered by the host through the seam the SUPPLY slice added
([`m5-supply-arrivals.md`](m5-supply-arrivals.md)). It claims no physical movement
and no complete D1: the other responsibilities stay open.

## What moved

`check_mobile_reached_waypoint` (`guide.c`) tries the SUPPLY target, then the new
radius-arrival target, then the legacy message. The radius-arrival target needs a
supported mobile (fixed-wing or helicopter), a task and a task leader, RADIUS
valid and ALTITUDE, HEADING and WEAPON_VECTOR not valid, and a member with a
roster host identity. The campaign still selects the waypoint, criteria, member
and answer combination. The shared helper `observe_reach` copies the same
primitive facts as for SUPPLY plus two: the waypoint sub-type and the waypoint
database's `player_skip_waypoint`. The Rust descriptor kind is `radius_arrival`;
the Lua callback key stays `supply_reached`.

The reference host (`eech-world`) applies the SUPPLY predicates unchanged:
fixed-wing landed, early range, direct radius and pass-through; helicopter AI
landed and rotor, leader radius, and nonleader cohesion. It adds the original
human-helicopter `player_skip_waypoint` waiver (`hc_msgs.c`). The four SUPPLY
waypoints are validated at boot to have that flag false, so SUPPLY is unaffected.

## Stays legacy (not claimed)

ALTITUDE, HEADING and WEAPON_VECTOR criteria (WEAPON_VECTOR consumes `frand1`; the
single random stream is untouched), ground vehicles, and members with no roster
host identity (the remaining coupling for unbound members).

## Known difference and debt

- The original helicopter nonleader path ASSERTs that LAST_TO_REACH is valid. The
  host path does not; see the classification under Author handoff.
- Naming debt: the generic seam is still called `supply_reached`,
  `campaign_supply_*` and `SupplyReachDescriptor`; see Author handoff.

## Evidence

Three revisions are involved and must not be conflated:

| Revision | Full SHA | Role |
|---|---|---|
| `044ea93a` | `044ea93a90b4fdb10db218626d0cd7b1ffb9ba7f` | the only native build under test |
| `ba892cbe` | `ba892cbe2ff749c5ad34e429f6e8308a482eaa05` | corrected witness script and the pack's `pack_revision`; binaries not rebuilt |
| `f17c3876` | `f17c38766d531c312cb7068c88e850f70792bcea` | first docs commit |

The final HEAD adds only rustfmt line-wrapping of one `matches!` in
`crates/eech-world/src/main.rs` (checked to be that hunk alone), documentation and
retained text evidence. No binary was rebuilt at `ba892cbe`, `f17c3876` or the
final HEAD, and none is presented as built there. The base is `5469adf7`
(`5469adf7d97ae8ae9fb63cdee89d16294c5db415`, the PR #88 merge).

Native build: `BUILD-INFO.txt` reports commit `044ea93a`, 0 uncommitted build
inputs, image
`sha256:bb10e44a20e9c6304b3a318902bcf0f8ec8175c162862240ce25c34bd7e562be`,
rustc 1.98.1, Lua 5.1.5, `x86_64-pc-windows-gnu` release.

| Artifact | SHA-256 |
|---|---|
| `eech_dc.dll` | `14914a2b84828e58c58012a39c55a02df532176489d644396855f22a9cce8d8f` |
| `eech-world.exe` | `81b82b28ec0db1ffbd0b2b04a01cdac26c5ba9836ce3d63e4eab7861a07f7b2e` |
| `lua.dll` | `b224cb2eacdea06fcb7fe273979e660fbef0c1a9c65be53c7be9fc0a4571daca` |
| `crates/eech-world/lua/aircraft-arrivals.lua` (run, from `ba892cbe`) | `ed15c47c4366904b81cb2cd86bae2b54091bc245203cb637057256463816cd91` |
| `lua/aircraft-arrivals.lua` inside the build directory (pre-repair, not used) | `b4e5e3c9bca6359f59d4a060985b94fc9ad6715c4c29a8520cc7e1ee0c1f8eff` |
| pack `report.json` | `6d2cdb2e3ebd66b6f264ea0f2e6b8c1499dbf0dd67cf856a0476bbbcf35d6b9e` |
| pack `report.md` (local only) | `16fadbb0a7fea91f127a106aa2b49d0f0f5b89aea3c5dc1997360bfb1a603908` |
| `recording.acmi`, 117690861 bytes (reference run1, run2 and retained; local only) | `3bd17b647665b558c7378206a3717c94e9cf771d4781d78e0b2aaae9a7d2d19b` |
| `linux-test.txt` (local only) | `ac2984bd9403456b13665d8378e044891723c6001c9ca0603893496fcb7bcd70` |

| Check | Result |
|---|---|
| Linux unit tests in the pinned image, at `044ea93a` | all suites ok, no failure, including `radius_arrival_keeps_the_same_physical_boundaries` |
| Baseline `5469adf7` aircraft reference | red as expected: "reference host received no non-SUPPLY fixed-wing arrival" |
| Candidate reference (original console) | fixed_wing 794579, helicopter 593649, pass_through 0, landed_reject 78484, helicopter_non_leader 322, ok. The same console records an initial failed WITHHOLD from the earlier invalid script (see Retained evidence) |
| Candidate withhold (`withhold-identity-3`, script `ba892cbe`) | ok, below |
| SUPPLY causal runs, candidate vs baseline | 3 of 3 `.out` files equal in content |
| One M4 pack, `ba892cbe` over the `044ea93a` binaries | PASS, exit 0; all 40 checks pass (parent re-audited); `regress.georgia.exact` and `regress.lebanon.exact` IDENTICAL to windows-retail v1 |
| Tacview | identical in both runs and to the retained recording; compared by bytes and structure, not visually reviewed |

There is no baseline difference to classify and nothing was re-recorded or
widened. Earlier attempts and their logs are preserved locally, not overwritten.
The earlier loss pack (`pack-loss-logical`, 40/40) applies to other binaries and is
not evidence for this build.

### Retained evidence

Text files under [`../reference/m5-aircraft-arrivals/`](../reference/m5-aircraft-arrivals/),
byte-exact (a directory-scoped `.gitattributes` sets `-text` so Git does not
normalise line endings). No binaries, retail roots or recordings are committed.

- [`report.json`](../reference/m5-aircraft-arrivals/report.json): the raw pack report (40 checks)
- [`BUILD-INFO.txt`](../reference/m5-aircraft-arrivals/BUILD-INFO.txt): build provenance for `044ea93a`
- [`identities.txt`](../reference/m5-aircraft-arrivals/identities.txt): hashes of the script and binaries used for the witness
- [`withhold-identity-3.stdout.txt`](../reference/m5-aircraft-arrivals/withhold-identity-3.stdout.txt): the successful witness output
- [`original-candidate-public-console.txt`](../reference/m5-aircraft-arrivals/original-candidate-public-console.txt):
  the first candidate run's console, recovered by the parent. The reference part
  passed; the same run's first WITHHOLD failed (an invalid pre-repair script,
  superseded by `withhold-identity-3`), so this file is not a clean pass.

**Causal witness.** The withhold script selects one reached non-SUPPLY helicopter
task leader that is already flying (velocity above 20 m/s), `eech-world:2999`, by
opaque host identity. It denies only that identity and gives every other member
the reference answer. While denied for 10.1 s over 101 calls, that member's own
position moved 67.407 m (`136028.781, 604.727, 68524.531` to
`135972.031, 604.727, 68488.156`). Its waypoint stayed `4295046965`; 0.1 s after
release the campaign advanced it to waypoint `4295046974`, and the task changed
`4295046955` to `4295046953`. It asserts that the member did not advance while
withheld. A task can have several guides, so identity, not task, is the unit;
after release the identity may advance to the next task, so advancement is the
changed (task, waypoint) pair. Earlier attempts (a hover with 0.221 m of motion,
and two task-key assertions that were invalid) are kept locally in
`withhold-identity-1` and `-2`.

Limits: the host answers synchronously in the same frame, so the ordering of
consequences is unchanged. The DLL still owns the aircraft and their movement.
The physical result is the same reference predicate, not a DCS adapter.

## Author handoff

One inline author pass over this bounded outgoing change against SRP, DRY, naming,
coupling, dead code, KISS, boundary and panic safety. It is not an independent
review.

- **SRP:** the campaign selects the waypoint, criteria and member; the host answers
  only the physical predicate. `mobile_type_supported` and `observe_reach` each do
  one thing. No finding.
- **DRY:** `observe_reach` is shared by the SUPPLY and radius-arrival targets
  instead of copied, and the reference host reuses the SUPPLY predicates.
- **Naming:** the generic seam keeps `supply_reached`, `campaign_supply_*` and
  `SupplyReachDescriptor`. This is a deliberate compatibility tradeoff for a
  bounded extension of the existing seam; the new kind is `radius_arrival`. No
  separate rename Task is proposed.
- **Coupling:** unchanged for ALTITUDE, HEADING, WEAPON_VECTOR, ground vehicles and
  members with no roster host identity (they fall through to the legacy handler).
  Physical data is still produced in the DLL (interim coupling).
- **Dead code:** none added; the legacy handlers stay reachable for the cases
  above. The pre-repair `lua/aircraft-arrivals.lua` in the build directory is not
  used.
- **KISS:** no new adapter, framework or generic criteria engine. The human
  helicopter `player_skip_waypoint` waiver is one flag copied from the original
  handler.
- **Boundary:** the host sees primitive facts and an opaque `host_id`; legacy
  pointers stay private. Missing callables, callback errors and non-boolean results
  fail loudly (SUPPLY slice).
- **Panic safety:** the host answer is read as a boolean through the existing
  checked seam; callback and Lua errors are returned across the boundary rather
  than ignored. No native guards were added in this change.

**Intentional diagnostic difference.** The original helicopter nonleader branch
(`hc_msgs.c`, `ASSERT (get_guide_criteria_valid (sender, GUIDE_CRITERIA_LAST_TO_REACH))`)
is not reproduced on the host path. In this native build `ASSERT` is not compiled
out: `eech_native_env.h` maps it to `eech_native_assert`, which aborts the current
entry with `EECH_STATUS_ASSERT`. So the legacy path would fail the entry on a
nonleader whose waypoint lacks LAST_TO_REACH, while the host path answers with the
reference predicate. The technical lead accepts this as an intentional change to a
private-handler invariant diagnostic. No valid-state physical-output difference was
observed in the pack or the reference run. Classification: intentional semantic
change confined to an invalid-state diagnostic, not a regression.

## Commands

```
tools/build-windows.sh target/aircraft-arrivals-bin        # fresh EECH_WIN_TARGET volume
tools/aircraft-arrivals-windows.ps1 -Root <root> -Bin <bin> -Out <out>
tools/supply-arrivals-windows.ps1  -Root <root> -Bin <bin> -Out <out>
tools/regression-pack.py --georgia E:/eech-runs/roots/georgia --lebanon E:/eech-runs/m2/lebanon-root \
  --lebanon-repeat E:/eech-runs/m2/lebanon-m1-copy --bin target/aircraft-arrivals-bin --out <out>
```

Full local artifacts are under `E:/eech-runs/m5-d1/aircraft-arrivals-20261021/`
(not committed).

## Next bounded action

One coherent remaining-arrival cut, from the advisory plan with the parent's
corrections binding (`E:/eech-runs/m5-d1/remaining-arrival-plan-20261021/LEAD-DECISIONS.md`).
It is not started here, and a new Task is raised at implementation start.

- **Scope:** mixed aircraft criteria, ground/sea/mobile predicates, direct
  waypoint-action proximity and recon proximity.
- **Stays in the DLL:** campaign selection, member iteration, task progression and
  the attack random tolerance (`frand1` and its per-type tolerance stay at the
  original conditional sites; no second stream).
- **Preserved:** each mobile kind's actual checks, including criteria it ignores;
  fixed-wing's release-build behaviour around its NO_WEAPON assertion; optional
  facts gathered only where the legacy predicate used them.
- **Host result:** physical gates plus an optional angle, validated when the
  valid-vector state requires it.
- **Unbound members:** a loud boundary failure, after the creation and link
  ordering is confirmed.
- **Still open after it:** host force creation/removal, movement/route execution
  and complete lifecycle and loss handling (D1).
