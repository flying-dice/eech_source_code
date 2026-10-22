# M5: host-observed remaining arrival and proximity decisions (EEDCS-31)

**Scope.** One physical extraction toward criteria 2 and 3 of the
[M5 goal](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337)
(all six criteria remain unaccepted; the goal page is live and versioned, so read
its current text), on integration `e31f2912` (the PR #89 merge). The physical
arrival and proximity decisions that
[`m5-aircraft-arrivals.md`](m5-aircraft-arrivals.md) left in the DLL are now
answered by the host through the same seam. It claims no physical movement and no
complete D1: the other responsibilities stay open.

## What moved

`supply_reached` (the callback key keeps its historical name) now takes a
descriptor with a `query` and returns a result structure, not a boolean. The
campaign still selects the waypoint, criteria, member and answer combination.

| Query | Original site | Host decides |
|---|---|---|
| `reached` | `check_mobile_reached_waypoint` via `campaign_radius_arrival_check` | the member's physical gates, by mobile kind |
| `action` | `check_guide_waypoint_action_reached` | the leader is inside the waypoint action radius |
| `recon` | `transmit_player_recon_data` | the helicopter is inside the recon radius (strict, squared 2D) |

`reached` covers fixed-wing, helicopter, routed vehicle, ship vehicle, anti-aircraft
and person, with the mixed criteria ALTITUDE, HEADING, WEAPON_VECTOR and
LAST_TO_REACH. Each kind keeps the checks and early-out order the original handler
used (`hc_msgs.c`, `fw_msgs.c` and the vehicle/person handlers); a kind that
ignored a criterion still ignores it. There is no generic AND of all bounds.

**Result.** `{gates_passed, launch_angle_error?, heading_target_missing?}`. A
deliberate physical denial is an explicit `gates_passed = false`. The reference
host's `host.supply_reached` returns that table, which is always truthy in Lua, so
callers read `.gates_passed`.

**Weapon vector.** The host reports the angle between the weapon and intercept
vectors; the DLL keeps `frand1`, the per-type exponent (fixed-wing 4, helicopter
6) and `weapon_database[...].max_launch_angle_error` at the original conditional
site, with the original conditional call count. There is no random number in the
host and no second stream. The angle is required only when the descriptor has
`weapon_gate`, `vectors_valid` and the earlier gates passed; an omitted angle
there is a boundary failure. The fixed-wing `NO_WEAPON` assertion is still fatal
in the DLL after the landed and radius gates and before vector handling; the
helicopter NO_WEAPON case still returns false.

**Heading.** The host derives the required heading. The descriptor carries
`heading_source` (`none`, `waypoint`, `zero` or `target`), the waypoint heading, a
`heading_target_present` flag and the target position. `zero` is the
prepare-for-insertion and troop-insert rule; `target` points from the member to
the task-dependent target (x east, z north), falling back to the waypoint heading
when the target has no readable position. The geometry is now host-side. When a
TARGET waypoint has no task or target, the host reports `heading_target_missing`
and the DLL re-runs the original `ASSERT`, which is fatal in this native build
(`eech_native_env.h` maps it to `EECH_STATUS_ASSERT`).

**Optional facts** (altitude, heading, vectors, rotor, human flags) are gathered
only where the legacy predicate read them.

**Invocation order** is unchanged: action, then recon, then reached, and the
SUPPLY action-bypass guard is kept.

## Unbound members are now a loud failure

Every campaign mobile kind above needs a roster host identity. A member without one
is `debug_fatal ("supply arrival: member has no host identity")` at the boundary.
Before this change such members fell through to the legacy handler silently. In the
retained Lebanon reference run every asked member had an identity (census
`identities=` column below).

## Host ask counts increased

The host now receives cases the DLL used to settle alone (a landed aircraft, an
out-of-range member, a non-leader). The reference census counts are therefore not
comparable with the PR #89 radius-only numbers: reached/helicopter 749574,
reached/person 1263015, reached/routed_vehicle 590272, reached/fixed_wing 797969,
reached/anti_aircraft 105313 (0 passed), reached/ship_vehicle 52873 (0 passed),
action/fixed_wing 13835 and action/helicopter 49761. The helicopter HEADING gate
sees `heading_source=target` 113175 times and `waypoint` 13212 times. The increase
is the intended shape of the change, not a behaviour difference.

## Follower LAST_TO_REACH diagnostic (relative to Task30)

The original helicopter non-leader branch asserts that LAST_TO_REACH is valid.
Task30 ([`m5-aircraft-arrivals.md`](m5-aircraft-arrivals.md), "Intentional
diagnostic difference") deliberately omitted that private-handler diagnostic. This
change restores an equivalent check at the host boundary: the reference host
requires `last_to_reach_valid` for a non-leader helicopter and fails the call
loudly when it is false (`follower_requires_the_last_to_reach_criterion`). The
campaign's own decisions are unchanged. The restored check lives in the host, not
the DLL; Task30's omission of the native `ASSERT` stands, and the DLL does not
re-run it on this path.

## Recon is unavailable end to end

`transmit_player_recon_data` is a player-helicopter path. The headless AI scenario
never reaches it, so no public-path witness exists. The recon query is covered by
focused tests only (`action_and_recon_queries`). This is reported as unavailable,
not as a pass. Recon never used the shared `frand1` stream.

## Evidence

**Tacview.** The complete reference `recording.acmi` of this pack (both repeatability runs and the retained copy) is byte-identical to the PR #89 recording (`3bd17b64...`, 117690861 bytes). This is a byte comparison of the whole file, not a visual inspection. It supports "no difference in the Lebanon reference run"; it does not cover the withheld-arrival witness runs, which are non-baseline by design.

Revisions involved; they must not be conflated:

| Revision | Full SHA | Role |
|---|---|---|
| `e31f2912` | `e31f291295269cf599947bd09ae8697350aa9432` | base (PR #89 merge) |
| `551f7aa9` | `551f7aa9ef92258812957b25ae50b03c29877059` | first implementation; its build and pack are **not evidence** |
| `cdcbd72e` | `cdcbd72e313502ccab7efc1d0cbe081a2e761e7c` | the only native build under test |
| `2158d0d8` | `2158d0d8cdb8c1e9227ef94997479ea8e3822cd8` | test-only witness fix; binaries not rebuilt |

The recorded build
and pack at that revision predate the heading repair and are discarded. The
native binaries come from `cdcbd72e`; the witness script (`2158d0d8`) is a Lua
file run from the repository path, so no binary was rebuilt for it.

Native build: `BUILD-INFO.txt` reports commit `cdcbd72e`, 0 uncommitted build
inputs, image
`sha256:bb10e44a20e9c6304b3a318902bcf0f8ec8175c162862240ce25c34bd7e562be`,
rustc 1.98.1, Lua 5.1.5, `x86_64-pc-windows-gnu` release.

| Artifact | SHA-256 |
|---|---|
| `eech_dc.dll` | `c67dc5a6766268277d316a4d760f1e1a98fb0712155a3841882f0e762dcd28b2` |
| `eech-world.exe` | `1a5f7d585c5c5fdf9b806cab2fc578167619c5541ce1e2f4b6e285abb35aaa52` |
| `lua.dll` | `b224cb2eacdea06fcb7fe273979e660fbef0c1a9c65be53c7be9fc0a4571daca` |
| `crates/eech-world/lua/remaining-arrivals.lua` (run, from `2158d0d8`) | `28f18272251a72d45f3b7bdbdfce14671f028234af73cc7a728d910ee39a07b7` |
| `lua/remaining-arrivals.lua` inside the build directory (cdcbd72e, pre-fix, not used for the final witness) | `cbd1f8dd8720bc4b7c7c522e1adb9bcde507219dec8c086159758be59862358e` |
| PR #89 reference `eech_dc.dll` (Tacview comparison baseline) | `14914a2b84828e58c58012a39c55a02df532176489d644396855f22a9cce8d8f` |
| PR #89 reference `eech-world.exe` | `81b82b28ec0db1ffbd0b2b04a01cdac26c5ba9836ce3d63e4eab7861a07f7b2e` |
| PR #89 `recording.acmi`, 117690861 bytes (local only) | `3bd17b647665b558c7378206a3717c94e9cf771d4781d78e0b2aaae9a7d2d19b` |

| Check | Result |
|---|---|
| Linux unit tests in the pinned image, at `cdcbd72e` | exit 0, no failure, including `helicopter_required_heading_follows_the_original_source_rules`, `helicopter_missing_target_is_reported_only_where_the_original_asserts`, `weapon_angle_is_reported_only_when_the_rule_applies`, `action_and_recon_queries` and `follower_requires_the_last_to_reach_criterion` |
| Reference census (`final3-witness/census.out`) | PASS; every mobile kind present, counts above |
| Withhold ground (person) | PASS: member held 10.1 s over 101 calls, advanced 0.1 s after release |
| Withhold altitude (helicopter leader) | PASS: advanced 83.5 s after release |
| Withhold heading (helicopter leader) | PASS **through the loss path**: see the classification below |
| Withhold weapon (helicopter leader) | PASS: advanced 15.8 s after release |
| Withhold action (fixed-wing) | PASS: action answer withheld 101 calls, advanced 6.7 s after release |
| Withhold recon | unavailable (see above); not allowed, not counted |
| One M4 regression pack over the `cdcbd72e` binaries (pack scripts at `2158d0d8`), exit 0 | PASS: 40 checks, 0 failed; `regress.georgia.exact` and `regress.lebanon.exact` IDENTICAL to windows-retail v1; `reference.repeatable` byte-identical across both Lebanon runs; the three 117690861-byte `recording.acmi` files all hash to `3bd17b64...` |

### Heading witness: failed expectation and classification

The parent's independent heading run at `cdcbd72e` exited 1 at the assertion
"the campaign did not advance the task after the arrival was released". It was not
unavailable, not a setup error and not a timeout. The diagnosis (deterministic;
the same selection and counts on two reruns; seed 1, 100 ms frames, same build):

- Selected identity `eech-world:2997`, an AI helicopter leader on waypoint
  `42949765773` (sub-type 27) at 272.0 s, `heading_source=target`; first
  descriptor: distance 18.86 against radius 20, heading 1.8945 against criterion
  0.0873.
- **Control, same candidate and seed, no denial, full hour:** the identity
  advances at 360.8 s (88.8 s after selection) after 887 asks, 237 passed. The
  helicopter is then lost at 371.4 s (a host `credit` and `physical` loss event for
  member 2997, group 678).
- **Withheld (denied 10.1 s, 101 calls, released at 282.1 s):** the identity stays
  on the waypoint. After release the reference answered `gates_passed=true` 151
  times, the last at 389.8 s. At 389.9 s the host reported the same member's
  `credit` and `physical` loss. The identity was never asked again and never
  advanced.

Interpretation: withholding delayed the arrival and the trajectory diverged; the
world destroyed the leader in both runs, at 371.4 s after advancing in the
control and at 389.9 s before advancing in the withheld run. Campaign state does
differ with the withheld observation (advance at 360.8 s becomes no advance). The
migration is not shown to differ from legacy behaviour; no baseline run of the
legacy path exists for this exact trajectory, so this is inferred from the
control, not reproduced against the original code.

Classification: the failed expectation, "the withheld identity always advances
after release", was an **invalid universal expectation** and the difference is
**expected input/world variation** (the physical trajectory diverged and the unit
was destroyed). It is not a regression, an approved defect correction or an
intentional semantic change. The delay, the scenario and the selection were not
changed.

The revised assertion (`2158d0d8`) still requires: the member was released, was
asked more than once while withheld, and did not advance while withheld. After
release it requires **either** an advance **or** a host-observed `physical` loss
of that identity at or after release, and it prints which (`outcome_after_release`
`advanced` or `lost`). A selection that is neither advanced nor lost still fails;
it is not turned into allowed-unavailable. The heading selection therefore passes
as `lost`: **this run does not demonstrate heading advancement after a withheld
observation**; the control shows advancement without a denial. Ground, altitude,
weapon and action did advance after release. A cleaner heading witness would need
a different selection, which was deliberately not hunted for.

### Retained evidence

Text files under [`../reference/m5-remaining-arrivals/`](../reference/m5-remaining-arrivals/),
byte-exact (a directory-scoped `.gitattributes` sets `-text`). No binaries, retail
roots or recordings are committed.

- [`BUILD-INFO.txt`](../reference/m5-remaining-arrivals/BUILD-INFO.txt): build provenance for `cdcbd72e`
- [`identities.txt`](../reference/m5-remaining-arrivals/identities.txt): hashes of the script and binaries used
- [`report.json`](../reference/m5-remaining-arrivals/report.json): the raw pack report
- `census.stdout.txt`, `withhold-{ground,altitude,heading,weapon,action}.stdout.txt`: the final witness outputs (script `2158d0d8`)
- `control-heading.stdout.txt`: the no-denial heading control (same candidate, seed and root; produced by an earlier diagnostic revision of the same script, whose diagnostic lines are identical)
- `linux-tests.txt`: the Linux unit test log and exit status

Full local artifacts are under `E:/eech-runs/m5-d1/remaining-arrivals-20261022/`
(not committed), including the parent's preserved failed run under
`independent-review-cdcbd72e/`. The `551f7aa9` build, pack and witness files there
are not evidence.

Limits: the host answers synchronously in the same frame, so the ordering of
consequences is unchanged. The DLL still owns the aircraft and their movement. The
physical result is the same reference predicate, not a DCS adapter. The recon query
and the unbound-member failure have no public-path witness.

## Remaining coupling

The raw facts the host judges (positions, headings, altitudes, velocities, weapon
and intercept vectors, rotor spin-up) are still produced by the DLL's physical
simulation. That is the D1/D2 coupling: the host is the decider, not yet the
source of the observations. Host force creation/removal, movement and route
execution, and complete lifecycle and loss handling are untouched.

## Author handoff

One inline author pass against SRP, DRY, naming, coupling, dead code, KISS,
boundary and panic safety. It is not an independent review.

- **SRP:** the campaign selects, iterates and progresses; the host answers only the
  physical predicate. Each query has its own entry point
  (`campaign_radius_arrival_check`, `campaign_waypoint_action_reached`,
  `campaign_recon_reached`). No finding.
- **DRY:** one descriptor-filling path and one result reader serve every query;
  the reference host's per-kind predicates are separate functions, not copies.
- **Naming:** `supply_reached`, `campaign_supply_*` and `SupplyReachDescriptor`
  are retained as accepted historical debt; no rename Task proposed.
- **Coupling:** see Remaining coupling. No pointer crosses the boundary; the
  descriptor is copied primitives and an opaque `host_id`.
- **Dead code:** the legacy bodies remain under `#else` for builds without
  `EECH_CAMPAIGN_HOST_ARRIVALS`; none is newly dead in the host build. The
  `551f7aa9` build directory scripts are not used.
- **KISS:** no new adapter, framework, RNG counter or generic criteria engine.
  The witness gained a control mode and a small loss watcher only.
- **Boundary and panic safety:** missing callbacks, callback errors, malformed
  results, a missing weapon angle and an unbound member each fail loudly through
  `debug_fatal`; Lua and callback errors cross the boundary as errors, not as
  silent passes.

## Commands

```
tools/build-windows.sh target/rem-final2-bin               # fresh EECH_WIN_TARGET volume
powershell -Command "& { tools\remaining-arrivals-windows.ps1 -Root <root> -Bin <bin> -Out <out> `
  -Selections @('ground','altitude','heading','weapon','action') }"
tools/regression-pack.py --georgia E:/eech-runs/roots/georgia --lebanon E:/eech-runs/m2/lebanon-root \
  --lebanon-repeat E:/eech-runs/m2/lebanon-m1-copy --bin target/rem-final2-bin --out <out>
```

`mode=control` on the same script (`select=heading`) runs the no-denial comparison.

## Next bounded action

Host force creation/removal, movement and route execution, and complete lifecycle
and loss handling (D1), with the observation source moving to the host (D2).
