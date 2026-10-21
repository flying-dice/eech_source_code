# M5: host-observed radius arrival for non-SUPPLY aircraft (EEDCS-30)

**Scope.** One physical extraction toward criteria 2 and 3 of the
[M5 goal](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337),
version 1, on integration `5469adf7`: a fixed-wing or helicopter member's arrival
at a non-SUPPLY waypoint whose only physical criterion is RADIUS (alone or with
LAST_TO_REACH) is now answered by the host through the seam the SUPPLY slice
added ([`m5-supply-arrivals.md`](m5-supply-arrivals.md)). It claims no physical
movement and no complete D1: the other responsibilities stay open.

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
  host path does not assert it; the ASSERT guarded a data invariant, not a
  physical result, and is not reproduced.
- Naming debt: the generic seam is still called `supply_reached`,
  `campaign_supply_*` and `SupplyReachDescriptor`. Renaming is a separate
  structural change and was not mixed in.

## Evidence

Build under test: native commit `044ea93a`, `BUILD-INFO.txt` 0 uncommitted inputs,
image `sha256:bb10e44a…e562be`, rustc 1.98.1. `eech_dc.dll` `14914a2b…8d8f`,
`eech-world.exe` `81b82b28…7b2e`, `lua.dll` `b224cb2e…daca`. The later commit
`ba892cbe` changed only the Lua witness script (`aircraft-arrivals.lua`
`ed15c47c…6816cd91`); the binaries were not rebuilt at that revision and are not
presented as built there. The `lua/aircraft-arrivals.lua` copy inside the build
directory is the pre-repair script; the repository script was run.

| Check | Result |
|---|---|
| Linux unit tests in the pinned image | all suites ok, no failure, including `radius_arrival_keeps_the_same_physical_boundaries` (`linux-test.txt`, run at `044ea93a`) |
| Baseline `5469adf7` aircraft reference | red as expected: the script fails with "reference host received no non-SUPPLY fixed-wing arrival" |
| Candidate reference (original console) | fixed_wing 794579, helicopter 593649, pass_through 0, landed_reject 78484, helicopter_non_leader 322, ok |
| Candidate withhold (`withhold-identity-3`) | ok, below |
| SUPPLY causal runs, candidate vs baseline | 3 of 3 `.out` files byte-for-byte equal in content |
| One M4 pack, `ba892cbe` over the `044ea93a` binaries | PASS, exit 0; `regress.georgia.exact` and `regress.lebanon.exact` IDENTICAL to windows-retail v1 |
| Tacview | `recording.acmi` 117690861 bytes, SHA-256 `3bd17b64…2d19b`, identical in both runs and to the retained recording |

The pack `report.md` SHA-256 is `16fadbb0…3908` and `report.json` `6d2cdb2e…9b9e`
(under `pack-aircraft-arrivals/out/`). There is no baseline difference to
classify and nothing was re-recorded or widened. The Tacview recording was
generated and compared by bytes and structure; it was not visually reviewed. The
earlier loss pack (`pack-loss-logical`, 40/40) applies to other binaries and is
not evidence for this build.

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
and two task-key assertions that were invalid) are kept in `withhold-identity-1`
and `-2`.

Limits: the host answers synchronously in the same frame, so the ordering of
consequences is unchanged. The DLL still owns the aircraft and their movement.
The physical result is the same reference predicate, not a DCS adapter.

## Author review

One inline pass by the author, not an independent review. The eight code
principles are not defined in AGENT.md, PROJECT.md or REVIEW.md, so the definition
used is: clear naming; small single-purpose functions; no duplication; separation
of concerns; minimal explicit interfaces; no hidden state; tests that assert
behaviour; honest comments and docs.

- Met: `observe_reach` is shared by both targets instead of copied; the campaign
  keeps selection and thresholds while the host answers only the physical
  predicate; the witness asserts observable behaviour (same-identity movement and
  advancement), not a call graph; this document states the limits.
- Not met, recorded as debt: the `supply_reached` / `campaign_supply_*` /
  `SupplyReachDescriptor` names no longer describe the generic seam.
- Behavioural difference: the dropped legacy LAST_TO_REACH ASSERT (above).
- Coupling remaining: members with no roster host identity, and the three
  unmigrated criteria and ground vehicles, still use the legacy path.

## Commands

```
tools/build-windows.sh target/aircraft-arrivals-bin        # fresh EECH_WIN_TARGET volume
tools/aircraft-arrivals-windows.ps1 -Root <root> -Bin <bin> -Out <out>
tools/supply-arrivals-windows.ps1  -Root <root> -Bin <bin> -Out <out>
tools/regression-pack.py --georgia E:/eech-runs/roots/georgia --lebanon E:/eech-runs/m2/lebanon-root \
  --lebanon-repeat E:/eech-runs/m2/lebanon-m1-copy --bin target/aircraft-arrivals-bin --out <out>
```

Artifacts are under `E:/eech-runs/m5-d1/aircraft-arrivals-20261021/` (not committed).
Next bounded action: the ALTITUDE, HEADING and WEAPON_VECTOR cuts, then ground
vehicles, each with its own characterisation.
