# M5: host-owned live fixed-wing movement (EEDCS-34)

**Scope.** One physical extraction under D1 (EEDCS-15) toward criteria 2 and 3 of
the [M5 goal](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337)
(all six criteria remain unaccepted; the goal page is live and versioned, so read
its current text), on base `f7aa3207887761f3652ba62dccf443bdda91a2cd` (the PR #92
merge). The alive physical movement of **fixed-wing aircraft** (taxi, flight,
landing, and the absolute flight variant used by the airborne branch) is now
stepped by the host, per existing host ID, through the `move_vehicle`
host-callback seam of [`m5-convoy-movement.md`](m5-convoy-movement.md) and
[`m5-surface-movement.md`](m5-surface-movement.md). **This does not finish D1 or
the aircraft work.** Helicopters, death/crash/impact motion, creation/removal and
lifecycle stay in the DLL; collision avoidance and pilot-control calculations
also stay (see Remaining coupling). Terrain is D3, combat is D2. A live DCS run
is M6 and is not claimed.

## What moved

`move_vehicle` gains `fixed_wing_move` (mode 7, kind 4). The DLL (`fw_move.c`)
still gathers the guidance and decides the branch; the host owns the pose and
motion vector per host ID and steps them with the original C maths, expression
for expression (`crates/eech-world/csrc/fw_step.c`, shared `eech_arith.h`):

| Phase | Source | Host does |
|---|---|---|
| `taxi` (0) | `fw_move.c` ground branch | ground acceleration, heading and terrain-following placement |
| `flight` (1) | airborne branch | follows its motion vector; speed clamped to 0.7x to 2x `cruise_velocity`; new vector scaled by the frame delta |
| `land` (2) | airborne branch, landing | takes the waypoint speed at once |
| `absolute` (3) | `fw_move.c:1868` | integrates from the attitude before its turn; never reports rotation |

- **Four ABI layers change in lockstep:** C `campaign_movement_step`/`result`
  (`rv_hostmv.h`), `eech_movement_step` (`eech-engine-sys`), serde `MovementStep`
  (`eech-engine`, Lua key `motion_vector_in`), and `movement_from` (`eech-world`).
  `movement_kind` 4 selects the step in `bind_mobile`.
- **Phase parsing is exact.** `fixed_wing_phase` accepts only `0`, `1`, `2`, `3`;
  a non-integer, negative, out-of-range or NaN value is refused with
  `host.move_vehicle: unknown fixed-wing phase <v>` (no float-to-int
  truncation). Test: `a_fixed_wing_phase_must_be_exactly_one_of_the_four_integers`.
- **Engine delta and frame delta stay distinct,** as for ships.

### Host-retained motion vector (lead blocker 1)

The previous motion vector is physical state, not campaign guidance. The host
now keeps it per host ID:

- **Seed.** `Movers::bind(host_id, kind, pose, Option<[f32;3]>)` takes the vector
  through the explicit bind descriptor chain (`gp_roster.c`
  `campaign_member_descriptor` -> `eech-engine-sys` -> `MemberDescriptor` -> `eech-dc` Lua table -> `eech-world`
  `bind_mobile`). An aircraft bound without a seed is refused; no other kind
  accepts one. There is no zero default. Test:
  `an_aircraft_needs_a_motion_vector_seed_and_no_other_kind_takes_one`.
- **Flight uses the host's vector.** The DLL still sends `motion_vector_in` as a
  mirror, but the step flies from the host's own value.
- **Bit-exact validation.** `FixedWingMove` compares the DLL mirror with the
  retained vector by `to_bits` **before** any mutation; a difference is refused.
  Test: `an_aircraft_whose_only_difference_is_the_campaign_motion_vector_is_refused_before_the_host_moves`
  (the pose is identical, only the vector differs; the host record is unchanged
  after the refusal). A pose difference is refused as before
  (`an_aircraft_whose_campaign_copy_differs_from_the_record_is_refused`).
- **Result.** The host stores the returned vector. The Lua result must carry
  `motion_vector`; `eech-dc` refuses a fixed-wing result without it, so a missing
  field can never be read as zero. The next step flies from the answered vector
  (`the_next_step_flies_from_the_vector_the_host_answered`).
- **Out-of-step writes are accounted for, never adopted silently.** The only
  DLL paths that write the vector outside a step go through the
  `MOTION_VECTOR` setter in `ac_vec3d.c`, which now calls
  `campaign_movement_place` (this covers `clear_fixed_wing_velocity`'s direct
  write in `fw_move.c` and the `fw_pack.c` unpack). `place` is the explicit, declared transitional
  coupling that may update the retained vector (and the pose); it is the only
  path that does. Tests:
  `placing_an_aircraft_adopts_the_campaign_motion_vector_and_other_kinds_ignore_it`
  (a stale mirror after a place is refused), 
  `placing_a_ship_a_person_or_an_aircraft_adopts_the_campaign_pose`.
- **Hold witness coherence.** When the host declines to step, the Lua answer
  returns its own `position`, `velocity`, `attitude` and
  `motion_vector = step.motion_vector_in`, `rotated = false`, so held movement
  and the release stay coherent.

### No fallback to the DLL under a host build (lead blocker 2)

Under `EECH_CAMPAIGN_HOST_MOVEMENT`, `campaign_movement_require_hosted(en)`
(`rv_hostmv.c`; `debug_fatal` unless a host callback and identity exist) is
called in the alive fixed-wing taxi (`fw_move.c:1717`), flight (`:1876`) and
absolute (`:2016`) branches, then the hosted step returns. An ordinary active
aircraft with no host ID or callback fails loudly instead of running the legacy
DLL physics. The legacy bodies stay in place after each `#ifdef
EECH_CAMPAIGN_HOST_MOVEMENT` early return, so builds without the flag are
unchanged. The absolute branch also `debug_fatal`s if reached on the ground.

- **Creation-time exception: none identified.** No guessed exception was added.
  The settled pack (3 simulated hours per scenario) and the witness hit no
  `campaign_movement_require_hosted` fatal; that is absence of a hit in those
  runs, not proof that no creation path exists.
- **Absolute ground variant: omitted.** Its only caller is `fw_move.c:1868`,
  inside the airborne branch (the host build `debug_fatal`s if it is reached on
  the ground), so a ground absolute step is never requested.

## Evidence

Revisions; they must not be conflated:

| Revision | Full SHA | Role |
|---|---|---|
| `f7aa3207` | `f7aa3207887761f3652ba62dccf443bdda91a2cd` | base (PR #92 merge) |
| baseline Tacview | n/a | PR #92 / M2 recording; SHA-256 `3bd17b647665b558c7378206a3717c94e9cf771d4781d78e0b2aaae9a7d2d19b` (the lead's earlier note `3bd17b66` was a truncation; the full SHA is this one). Reused, no baseline rerun |
| `f4155e24` | `f4155e24d4e3cc6069f541f071057a46d7bf7283` | runtime code commit and the settled native build; tree `645a0a2de7df2cc064cf8346d1f2bc8a74871f0b` |

The documentation and reference evidence are committed after `f4155e24` and do
not change any build input.

Settled native build: `BUILD-INFO.txt` reports commit `f4155e24`, 0 uncommitted
build inputs, image
`sha256:bb10e44a20e9c6304b3a318902bcf0f8ec8175c162862240ce25c34bd7e562be`,
rustc 1.98.1, cargo 1.98.1, mingw gcc 12-posix, Lua 5.1.5,
`x86_64-pc-windows-gnu` release, fresh volume.

| Artifact | SHA-256 |
|---|---|
| `eech_dc.dll` | `0b4757f1be747f0c4235a84054942bc89379dbd20f73fa9feeab6e50dccd23cc` |
| `eech-world.exe` | `fd7d75bd594e3586264004662c236e2277c73fa35d5dfc09e315f518d5546d49` |
| `lua.dll` | `b224cb2eacdea06fcb7fe273979e660fbef0c1a9c65be53c7be9fc0a4571daca` |
| `lua/fixed-wing-movement.lua` | `ca1fafe7fc4511239ccf94814914d828a7e3698faf098a58b8f3046acf89b188` |
| `recording.acmi` (local only) | `3bd17b647665b558c7378206a3717c94e9cf771d4781d78e0b2aaae9a7d2d19b` |

### Public witness: held fixed-wing aircraft

`lua/fixed-wing-movement.lua` through `tools/fixed-wing-movement-windows.ps1`
(modes `missing`, `control`, `hold`), retail Lebanon root, `hours=1` (the run ended at 221.6 s of simulated time).
The path is host Lua -> `eech_dc` -> `move_vehicle`. Coverage counts were
combined into the hold run rather than run as a separate census.

- **missing:** `dc.boot` without `move_vehicle` is refused:
  `runtime error: eech: missing host move_vehicle callback`.
- **control:** subject `eech-world:2696`, selected at 81.9 s.
- **hold:** the same subject selected at 79.3 s; the host declined to move it for
  142 s and answered with the pose it holds. 1421 held steps; held displacement
  in the campaign exactly **0.000 m**; after release it moved **0.030 m** in the
  remaining 0.2 s.
- **Coverage in the window (221.6 s):** `fixed_wing bound=91`, `moved_10m=9`;
  6438 `fixed_wing_move` steps, **all phase 0 (taxi)**; also person_begin 303,240,
  person_end 302,804, routed move 161,159, routed stop 10.

**Limit.** The witness window only reached taxi. Flight, landing and absolute
have **no public runtime witness**: their coverage is the focused `movement.rs`
tests plus the unchanged results of the pack below, for which no per-phase step
count was recorded (no separate long census was run, by instruction). The hold is
a host declining to step, not a physical obstruction; it shows the seam is causal.

### Tests

- Linux, pinned image, **full workspace** `cargo test --locked`, exit 0
  (`reference/m5-fixed-wing-movement/linux-workspace-test.log`): `eech-world` lib
  29, bin 17, `host_callbacks.rs` 2, `load_dc.rs` 2, all pass.
- New focused tests in `movement.rs`: seed, taxi, flight, landing, absolute,
  place adoption, pose-mismatch refusal, **vector-only-mismatch refusal before
  host mutation**, and next-step-from-answered-vector; in `main.rs`: exact phase
  parsing.
- Windows host: `cargo clippy --locked --all-targets -p eech-world -- -D warnings`
  exit 0; `cargo fmt --all` clean.
- **Red/green.** The physical-step tests characterise ported maths and passed
  once set up correctly; the regression pack, not those tests, is the
  bit-identity proof. The vector-only refusal and the seed/phase refusals are
  direct error-path checks.

### Regression pack and Tacview

Pack revision `f4155e24d4e3cc6069f541f071057a46d7bf7283` (0 dirty build inputs),
run 2026-10-22T08:05:02, accepted baseline `windows-retail v1`, seed 1, 3
simulated hours, 100 ms frames. The ONE settled pack (no ABI or baseline rerun).
Command, as recorded in the report:

```
tools/regression-pack.py --georgia E:/eech-runs/roots/georgia --lebanon E:/eech-runs/m2/lebanon-root --lebanon-repeat E:/eech-runs/m2/lebanon-m1-copy --bin target/fw-settled --out E:/eech-runs/m5-d1/fixed-wing-movement-20261022/settled-pack
```

Result: **REGRESSION PACK: PASS**, exit 0, 40 checks. Claims: 17 PASS directly
protected, 12 PASS baseline-sensitive, 8 NOT COVERED, 8 evidence-only, all
unchanged from the accepted pack.

- Georgia and Lebanon exact comparisons (`regress.georgia.exact`,
  `regress.lebanon.exact`) PASS against `windows-retail v1`.
- Lebanon reference (two roots with identical inputs, 1179 files, run 1 and 2; runs took 2,849 s):
  `recording.acmi`, `observations.jsonl` (85,716,447 bytes, `da26bab0…`) and
  `metrics.json` (`fa09d562…`) byte-identical across both runs and equal to the
  PR #92 values; 39 of 39 campaign expectations hold; 31 of 32 supply deliveries
  complete (0 unattributed).
- Tacview: `recording.acmi` 117,690,861 bytes, SHA-256
  `3bd17b647665b558c7378206a3717c94e9cf771d4781d78e0b2aaae9a7d2d19b` in the
  retained artifact and in both reference runs, **equal to the baseline**. Byte
  and structure comparison only; no recording was reviewed visually.
- **Classification of differences against the baseline: none.** No golden was
  re-recorded and no tolerance widened.

The report is committed as `reference/m5-fixed-wing-movement/pack-report.md`
(SHA-256 `a228626026d27cb3e33747740f1836850b2c8e1b2bfe4768c628247f598308d7`) with
`pack-report.json` and `reference-summary.txt`; the pack output tree stays local.

### Retained failed iterations

| Iteration | Failure | Cause and fix |
|---|---|---|
| first test run | flight/land/absolute tests `10.0 vs 0.0` | test setup: the default elevation 10 triggered the terrain clamp; tests set `elevation = 0.0`. Log: `test-failure-1-elevation-clamp-test-setup.log` |
| second and third test runs | placing test `70 vs 40` | test setup: flight clamps speed to 0.7x to 2x `cruise_velocity`; the test set `cruise_velocity = 40.0`. Log: `test-failure-2-cruise-clamp-test-setup.log` |
| first clippy run | 4 x `redundant_guards` in `fixed_wing_phase` | literal match patterns. Log: `clippy-host-first-failure-redundant-guards.log` |

No tolerance or golden changed; none of these was a product defect.

### Limits

The host steps the entity synchronously in the same frame, so the order of
consequences is unchanged. The result is the same reference physics, not a DCS
adapter. Visual inspection of Tacview and a live DCS run (M6) are not claimed.
Flight/landing/absolute have no public runtime witness (above).

## Remaining coupling

D1/aircraft is **not** finished. Left in the DLL:

- Collision avoidance, navigator and pilot-control calculations, `avoid_enemy_fire`
  and the toggles/magitek. These are remaining **physical coupling**, not
  automatically campaign policy, and each needs its own boundary decision.
- Waypoint fetch and the `wp_vel` fallback; the MOBILE_MOVING/guide gates.
- Terrain elevation sampling (D3).
- Death, crash and impact movement (direct-pointer writes), creation/removal and
  lifecycle (later D1); helicopters.
- The DLL's pose and vector copy, and `place` updating the host vector, remain
  declared transitional coupling because the entity still lives in the DLL.

## Author handoff

One inline author pass against SRP, DRY, naming, coupling, dead code, KISS,
boundary and panic safety. It is not an independent review; the parent is the
sole independent reviewer.

- **SRP:** the DLL gathers guidance and picks the phase; the host holds pose and
  vector and steps them. `fw_step.c` is the maths only; `movement.rs` the records
  and guards; `rv_hostmv.c` the one DLL-side seam.
- **DRY:** one `Movers::step` path for all kinds; shared `eech_arith.h`; the
  fixed-wing vectors reuse `exact_numbers` in `eech-dc`.
- **Naming:** `fixed_wing_move`, `fixed_wing_phase`, `fw_*` fields follow
  `ship_move`/`person_begin`.
- **Coupling:** no pointer crosses the boundary. The vector mirror is validated,
  not trusted; `place` and the DLL copies are named above.
- **Dead code:** none new in the host build; the legacy bodies stay reachable only in builds without the flag.
- **KISS:** no framework or per-class mock; the witness adds modes to the
  existing driver pattern.
- **Boundary and panic safety:** unknown id, kind mismatch, missing seed, foreign
  pose, vector-only mismatch, bad phase, unknown mode and a missing callback each
  fail loudly before mutation; a result without `motion_vector` is refused.

Retained text evidence is in `reference/m5-fixed-wing-movement/`. No retail
data, binaries or recordings are committed.

## Commands

```
tools/build-windows.sh target/fw-settled          # Git Bash, pinned eech-build image, fresh target volume
tools/fixed-wing-movement-windows.ps1 -Root <lebanon root> -Bin target/fw-settled -Out <out>
python tools/regression-pack.py --georgia E:/eech-runs/roots/georgia --lebanon E:/eech-runs/m2/lebanon-root --lebanon-repeat E:/eech-runs/m2/lebanon-m1-copy --bin target/fw-settled --out <out>
cargo clippy --locked --all-targets -p eech-world -- -D warnings   # Windows host
cargo test --locked                                                 # full workspace, in the eech-build image
```

The exact container invocation of the Linux workspace test is in
`reference/m5-fixed-wing-movement/linux-workspace-test.log`'s producing run
(`test-ws5` in the local output directory); it is not reproduced here.

## Next bounded action

The lead chooses the next D1 extraction: helicopters, the collision-avoidance and
pilot-control boundary, or creation/removal and death/lifecycle. A runtime
flight/landing witness would also close the coverage limit above.
