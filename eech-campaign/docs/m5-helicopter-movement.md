# M5: host-owned live AI helicopter movement (EEDCS-35)

**Status: implemented; settled retail build, public witness and 40/40 regression pack
run at runtime commit `e5ec3b69`.** Nothing here is accepted: it awaits independent
review and the technical lead's approval. D1 and the aircraft work are not complete.

**Scope.** One physical extraction under D1 (EEDCS-15) toward criteria 2 and 3 of
the [M5 goal](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337)
(all six criteria remain unaccepted; the goal page is live and versioned, so read
its current text), on base `03e89ff6` (the PR #93 merge; runtime commit
`f4155e24`). The alive physical movement of **AI helicopters** (motion vector,
speed, moved flag, attitude, position, map bound and floor clamp) is stepped by
the host, per host ID, through the `move_vehicle` seam of
[`m5-fixed-wing-movement.md`](m5-fixed-wing-movement.md). **This does not finish
D1 or the aircraft work.** See Remaining coupling. A live DCS run is M6 and is not
claimed.

## What moved

`move_vehicle` gains two modes around the campaign's attitude guidance, for
`movement_kind` 5:

| Mode | Value | Host does |
|---|---|---|
| `helicopter_begin` | 8 | `mv.x += ax*dt; mv.y = wp_vec_y; mv.z += az*dt`; speed = magnitude; `moved = fabs(speed) > 0.1`; answers `pending = true` |
| `helicopter_end` | 9 | requires a pending begin; wraps `desired_heading - current` to [-pi, pi] and bounds it to `max_turn_rate`; 0.5*dt attitude blend; `rotated` at `rad(0.1)`; position integration; map bound; floor clamp |

The heading wrap and turn-rate bound are **physical inputs and retained coupling, not
campaign decisions**: the DLL passes `desired_heading` and `max_turn_rate` (guidance
inputs) and the host applies the original expressions. Begin answers `pending = true`;
that is the final handshake and the `eech_engine.h` documentation says so.

- **Original arithmetic, copied.** `crates/eech-world/csrc/hc_step.c`
  (`eech_hc_begin`, `eech_hc_end`) copies `helicopter_movement_calculate_new_position`
  expression for expression. The floor clamp may change vector-y **after** speed is
  computed; speed is not recomputed. Floor: `min_height = elevation + cg_to_ground
  (+2.0 if terrain_follow)`; if `pos.y <= min_height`, pitch is taken from the new
  attitude, `min_height += 5.0*sin(fabs(pitch))`, `mv.y = max(mv.y, 0)`,
  `pos.y = max(pos.y, min_height)`.
- **Two phases, because guidance reads the vector.** Begin is applied to the entity
  immediately (inside the apply bracket, so the setter's `campaign_movement_place`
  is a no-op) and the campaign's guidance then reads the host's vector and speed.
  End applies the attitude, position and vector-y.
- **ABI layers change in lockstep:** C `campaign_movement_step`/`result`
  (`rv_hostmv.h`, new `hc_*` step fields and result `moved`), `eech_movement_step`
  (`eech-engine-sys`), serde `MovementStep` (`eech-engine`), `eech-dc`
  `call_move_vehicle` (parses `moved`, required for begin) and `movement_from`
  (`eech-world`).
- **DLL side.** `hc_move.c` calls `campaign_movement_require_hosted` then
  `campaign_movement_helicopter_begin/end` under `EECH_CAMPAIGN_HOST_MOVEMENT`; the
  original bodies stay in `#else`/`#ifndef`, so builds without the flag are
  unchanged.

### Host-retained motion vector

Extends the fixed-wing contract coherently:

- **Seed.** `gp_roster.c` `campaign_member_descriptor` carries the helicopter's
  pose pointer and motion-vector seed; `bind` refuses a seedless aircraft
  (`a_helicopter_is_never_bound_without_its_motion_vector`).
- **Mirror validation before mutation.** The DLL's vector (`hc_motion_vector`) and
  pose are compared bit-exactly with the host record; a difference is an error, never
  an adoption (`a_helicopter_mirror_that_differs_is_refused_and_nothing_moves`).
- **Pending guard.** `helicopter_end` without a begin, or a second begin while one
  is pending, is refused before any change
  (`a_helicopter_end_without_a_begin_or_a_second_begin_is_refused_before_any_change`).
  Pairing, kind and state are validated up front.
- **Place** adopts the pose and the vector for helicopters, preserving raw-position
  setter alias semantics (`placing_a_helicopter_adopts_the_campaign_motion_vector`).
  **`fw_motion_vector` is the one aircraft placement vector** through C, the sys
  struct, the engine callback and Lua: the engine reads `hc_motion_vector` only for
  `helicopter_begin`/`helicopter_end` and `fw_motion_vector` for every other mode.
- **Flags and results.** `ask_host` validates `pending` and `moved` as booleans;
  begin fatals unless pending is TRUE and end fatals if pending is left set.
  `eech-dc` requires `motion_vector` in aircraft results, so a missing field is
  never read as zero.
- **Early returns** in `hc_move.c` before the begin leave no pending state.

### No fallback to the DLL under a host build

`campaign_movement_require_hosted(en)` stops with a clear error (`debug_fatal`) if
an alive AI helicopter in the server path has no host ID or callback. There is no
silent fallback to DLL physics, and no host mismatch is silently adopted.
**Limitation:** an unhosted helicopter (client/multiplayer path) fatals by design
in a host build. Only the AI server-path `helicopter_movement` call is exercised by
the headless campaign.

## Evidence (local; this stage)

Working tree on `03e89ff6` (uncommitted when run). Output:
`E:/eech-runs/m5-d1/helicopter-impl-20261022/` (preliminary) and `post-fix/` (after the
independent review's corrections: pending TRUE, one placement vector, host
wrap/clamp).

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` (`post-fix/fmt-check-final`) | exit 0 |
| `cargo clippy --all-targets -p eech-world -p eech-engine -p eech-dc -D warnings` and `cargo test` for the same packages (pinned image, ephemeral clippy; `post-fix/clippy-and-test-final`) | exit 0; `eech-world` lib 40, bin 17, `host_callbacks` 2, `load_dc` 2; `eech-engine` 3; `eech-dc` `lua_debug_layout` 2 |
| Lua syntax of `helicopter-movement.lua` (Lua 5.1.5 `loadfile`, `post-fix/lua-syntax.log`) | ok (rough check; the repo embeds a different Lua) |
| Earlier full-workspace `cargo test` / `clippy --workspace` (`test-workspace1`, `clippy-workspace`) | exit 0 **before** the review corrections; superseded for the three packages above, not rerun for the rest |

Failed attempts are kept with a `-failed` suffix (compile errors in the new
transport tests, an `expect_used` clippy denial, a formatting diff); each was fixed
and rerun. `clippy-host-workspace` failed on the `eech-engine-sys` windows-msvc
refusal (environment limit).

**Compile provenance** (from build files, not memory):

- `eech-world/build.rs` compiles `csrc/hc_step.c` with `cc` on every target.
- `eech-engine-sys/build/main.rs` supports Linux and windows-gnu, defines
  `EECH_CAMPAIGN_HOST_MOVEMENT`, `EECH_CAMPAIGN_HOST_IDENTITIES` and
  `EECH_CAMPAIGN_HOST_ARRIVALS`, and compiles all of `sources.txt` (including
  `hc_move.c`, `rv_hostmv.c`, `gp_roster.c`) with `-w` plus
  `-Werror=implicit-function-declaration`, `-Werror=implicit-int` and
  `-Werror=int-conversion`.
  The DLL C therefore **compiled under gcc in the Linux container** with the host
  flags in the post-fix runs.

### Native evidence (runtime commit `e5ec3b69`, clean tree: 0 uncommitted build inputs)

Output: `E:/eech-runs/m5-d1/helicopter-impl-20261022/native/` (`build.log`,
`witness/`, `pack/`, `pack.log`).

- **Build.** `tools/build-windows.sh target/hc-settled` with `EECH_WIN_TARGET=eech-hc-settled-target`
  (fresh volume), pinned image `sha256:bb10e44a20e9c6304b3a318902bcf0f8ec8175c162862240ce25c34bd7e562be`
  (rustc 1.98.1, MinGW gcc 12-posix): exit 0, one build. This is the first
  compile of the DLL C under the retail (MinGW) image. `eech_dc.dll`
  `8e14e68170df097aa250c84964940b6a7d9ea724553641d07c2c865a3462e02a`,
  `eech-world.exe` `ac6e9668f225b1c7b3b22979bd1472e0c4b6e54be2f2a72b4bae85ea7498365f`
  (full list in `BUILD-INFO.txt`).
- **Public witness.** `tools/helicopter-movement-windows.ps1 -Root E:/eech-runs/m2/lebanon-root`
  (Lebanon retail root, 1 h): `PASS`, one run.
  - `missing`: `runtime error: eech: missing host move_vehicle callback`.
  - `control`: helicopter `eech-world:2975` selected at 79.3 s.
  - `hold`: the host declined to move it for 140.1 s (1401 held steps,
    `held_displacement 0.000`), then released it (`displacement_after_release 0.764`
    m, measured over the 0.2 s to the end of the run).
  - Census: helicopters bound 174, 27 moved at least 10 m; 17,994 begin and 17,994
    end steps, `pairing_errors 0`, `helicopter_pairs 17994 moved_begins=16592
    places=348`. Fixed-wing, routed, ship and person steps also flowed.
  - This is a causal hold/release witness of the Lua -> `eech_dc` -> `move_vehicle`
    path. It asserts no waypoint completion.
- **Regression pack.** `tools/regression-pack.py --georgia E:/eech-runs/roots/georgia
  --lebanon E:/eech-runs/m2/lebanon-root --lebanon-repeat E:/eech-runs/m2/lebanon-m1-copy
  --bin target/hc-settled`: `REGRESSION PACK: PASS`, 40/40 checks PASS.
  `regress.georgia.exact` and `regress.lebanon.exact` are `IDENTICAL to the baseline`;
  inputs are identical to the retained manifests. The Tacview recording
  (`pack/reference/run1`, `run2`, `retained`) has SHA-256
  `3bd17b647665b558c7378206a3717c94e9cf771d4781d78e0b2aaae9a7d2d19b`, equal to the
  accepted PR #93 baseline. Baseline differences: **none**; no golden, tolerance or
  scenario was changed.

**Not claimed:**

- **No Tacview visual review.** The recording is byte-identical and structurally
  checked, not visually inspected (the pack's own `tacview-visual` limitation).
- **One seed per scenario and a coupled engine** (the pack's `one-war` and
  `coupled-engine` limits): identity to the baseline shows no change at that seed
  and does not prove the helicopter step bit-identical for all inputs, nor
  attribute behaviour to the host world rather than the DLL.
- **Hold/release moved by only 0.764 m after release**, a small displacement
  measured over a short window. It demonstrates the host-controlled pose is what
  the campaign sees, not a flight profile.
- **No live DCS run** (M6) and no separate long census, baseline or ABI rerun.
- Only the AI server-path helicopter call is exercised.

### Tests

Focused in `eech-world/src/movement.rs` (they characterise the ported maths and the
guards; bit-identity is the regression pack's job, not these tests'):

- `a_helicopter_step_is_two_phases_and_the_second_flies_from_the_vector_the_first_answered`
- `a_hovering_helicopter_has_not_moved`
- `a_helicopter_reports_a_rotation_only_beyond_a_tenth_of_a_degree`
- `a_helicopter_is_bounded_to_the_map`
- `a_helicopter_is_never_bound_without_its_motion_vector`
- `a_helicopter_mirror_that_differs_is_refused_and_nothing_moves`
- `a_helicopter_end_without_a_begin_or_a_second_begin_is_refused_before_any_change`
- `placing_a_helicopter_adopts_the_campaign_motion_vector`
- `a_helicopter_turns_the_shortest_way_to_the_desired_heading`
- `a_helicopter_turns_no_faster_than_its_turn_rate`

Transport-boundary tests in `eech-engine/src/lib.rs`. They call the production
`move_vehicle` callback through `with_host` and check the wire-field mapping; they
complement inspection of the C producer and are **not** proof of the full public
path:

- `an_aircraft_placement_carries_its_nonzero_vector_across_the_callback_boundary`
- `a_placement_reads_the_placement_field_not_the_helicopter_step_field`
- `the_helicopter_step_modes_read_the_helicopter_vector_not_the_placement_field`

**Red/green.** No red run was recorded for these: the physical-step tests are
characterisation of ported maths and the guard tests are error-path checks written
with the guard. Stated as a limitation, not as red evidence.

### Public witness (design; result under Native evidence)

`lua/helicopter-movement.lua` through `tools/helicopter-movement-windows.ps1` (modes
`missing`, `control`, `hold`, plus `census` counts folded into `hold`), Lebanon
retail root. The control run finds a helicopter the host moves at least 10 m; the
hold run has the host decline to move it (begin answers the vector it already holds,
`moved=false`, `pending=true`; end answers the held pose) for longer than that, then
move it again. It asserts that the campaign's view of the pose does not change while
held and does change after release, and that every begin is followed by its end. It
also prints bound/moved counts per kind, begin/end step counts and
`helicopter_pairs` (paired ends, begins that moved, and placements, the declared
transitional writers). It makes no waypoint-advance claim. Paths: host Lua
-> `eech_dc` -> `move_vehicle`. Output: `result ok` and `PASS`.

## Remaining coupling

D1/aircraft is **not** finished. Left in the DLL, each needing its own boundary
decision; none is relabelled campaign policy:

- **Rotor RPM and spin-up** (`helicopter_movement_calculate_rotor_rpm`,
  `main_rotor_spin_up_timer`, `update_rotors`): physical, deferred to D1
  death/lifecycle.
- **Pilot control and avoidance:** physical coupling. LOS/structure avoidance and
  guide/leader/formation likewise.
- **D2** pitch/roll/heading guidance and **D3** terrain look-ahead.
- **Death/crash movement and other direct writers:** `clear_helicopter_velocity`,
  VELOCITY writes in death/crash, `hc_updt.c:979`, `hc_msgs.c:514-518`,
  `hc_pack.c:751-757`. These are **explicitly transitional placement**, announced to
  the host only through the setters' `place`; they are not migrated.
- The DLL's pose and vector copy remain, because the entity still lives in the DLL.
- Client/multiplayer helicopters are unsupported in a host build (above).
- No claim is made about other aircraft kinds or that D1 is complete.

## Author handoff

One inline author pass against SRP, DRY, naming, coupling, dead code, KISS, boundary
and panic safety. It is not an independent review; the parent is the sole
independent reviewer. The C diff of `hc_step.c` was compared with the original
`helicopter_movement_calculate_new_position`: matches. The DLL C compiled under gcc
in the Linux container; the retail image compile is pending.

- **SRP:** the DLL gathers guidance and decides; the host holds pose/vector and
  steps. `hc_step.c` is the maths; `movement.rs` the records and guards.
- **DRY:** one `Movers::step` path; helicopter reuses the aircraft seed, Place and
  mirror-validation paths.
- **Coupling:** no pointer crosses the boundary; the mirror is validated, not trusted.
- **Dead code:** the legacy bodies are reachable only without the flag.
- **KISS:** no framework or per-class mock; the witness follows the existing driver.
- **Boundary and panic safety:** unknown id, kind mismatch, missing seed, foreign
  pose, vector-only mismatch, unpaired begin/end, bad boolean and a missing callback
  fail loudly before mutation.

## Commands

```
cargo fmt --all -- --check
cargo clippy --locked --all-targets -p eech-world -- -D warnings     # Windows host
# pinned image eech-build, volumes eech-fw-target / eech-cargo
cargo clippy --workspace --all-targets -- -D warnings                # after rustup component add clippy
cargo test --locked                                                  # full workspace
EECH_WIN_TARGET=eech-hc-settled-target tools/build-windows.sh target/hc-settled
tools/helicopter-movement-windows.ps1 -Root <lebanon root> -Bin target/hc-settled -Out <out>
python tools/regression-pack.py --georgia <georgia> --lebanon <lebanon> --lebanon-repeat <lebanon copy> --bin target/hc-settled --out <out>
```

## Next bounded action

Independent review of this runtime and evidence by the parent, then publication
(branch, PR on the Task, handover). The PR needs the technical lead's formal approval
before merge. If a later review changes C or Rust source, the settled build, witness
and pack must be rerun against the new commit; do not re-record goldens.
