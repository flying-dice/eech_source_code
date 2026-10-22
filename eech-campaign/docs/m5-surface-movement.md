# M5: host-owned ship and infantry movement (EEDCS-33)

**Scope.** One physical extraction toward criteria 2 and 3 of the
[M5 goal](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337)
(all six criteria remain unaccepted; the goal page is live and versioned, so read
its current text), on base `5f951122` (the PR #91 merge). The alive physical
movement of **ships** and **infantry (persons)** is now stepped by the host, per
existing host ID, through the same `move_vehicle` host-callback seam as
[`m5-convoy-movement.md`](m5-convoy-movement.md). It claims no complete D1:
death motion, creation/removal and lifecycle stay in the DLL (later D1), combat
and perception are D2, terrain is D3. A live DCS run is M6 and is not claimed.

## What moved

`move_vehicle` gains three request modes next to the routed `move` / `stop` /
`place`. The campaign still owns guide, formation, hold/engage and animation
choices and the RNG order; the host owns the pose per host ID and steps it with
the original C maths, kept expression for expression:

| Mode | Source | Host does |
|---|---|---|
| `ship_move` | `sh_move.c` | heading, velocity and position from the engine delta, the motion vector from the frame delta |
| `person_begin` | `ps_move.c` | horizontal advance toward the waypoint; `pending` only when terrain-following |
| `person_end` | `ps_move.c` | applies the final pose from the terrain height the DLL sampled (host adds +0.01) |

(`crates/eech-world/csrc/ship_step.c`, `person_step.c`, the shared
`eech_arith.h`, and `convoy_step.c`, which now shares the same arithmetic.)

- **Engine delta and frame delta stay distinct.** Ships use the engine delta
  (`get_entity_movement_delta_time`) for heading, velocity and position and the
  frame delta (`get_one_over_delta_time`) for the motion vector. Persons use the
  engine delta throughout. Early returns, floating-point order, the legacy
  repeated `z` assignment near a ship's waypoint, the turn cap and the
  asymmetric acceleration are preserved.
- **Bounded two-phase infantry terrain seam.** The host moves horizontally
  (`person_begin`); the DLL asserts `point_inside_map_area` and samples
  `get_3d_terrain_elevation` (still D3, unchanged); the host places the final
  pose (`person_end`). While a person step is pending the host holds the
  committed pose unchanged, and every other request for that ID is refused. No
  callback leaves the host into the DLL during a step.
- **`movement_kind`** (1 routed, 2 ship, 3 person) in the `bind_mobile`
  descriptor selects the physical step. `host.bind_mobile` with a pose now
  **requires** `movement_kind`: an intentional contract change, seeded in
  `gp_roster.c/.h`.
- `ps_anim.c` still writes the targeting attitude directly; it now announces
  that with `campaign_movement_place`. That is a declared combat coupling, not
  extracted movement physics.
- The Lua `move_vehicle` result is flat: `position`, `velocity`, `attitude`,
  `rotated`, `motion_vector`, `pending`. A response without `motion_vector`
  (for example a host that holds a subject and answers with its own pose)
  defaults the motion vector to zero; the held witness relies on this.

Host refusals. These are raised in different places:

- In `Movers::step` (`crates/eech-world/src/movement.rs`), where the guard checks run before
  the host record is mutated:
  - unknown host id: `host.move_vehicle: unknown host id <id>`;
  - a request for another kind: `... is not a request for <Kind>`;
  - `person_end` without a pending `person_begin`;
  - any request for an ID `while its person step is pending`;
  - a campaign pose that differs bitwise from the host record.
- An unknown mode is rejected earlier, when the host parses the request in
  `movement_from` (`eech-world/src/main.rs`): `host.move_vehicle: unknown mode <m>`.
- A missing callback is rejected in the campaign module at boot
  (`eech-dc/src/module.rs`): `eech: missing host move_vehicle callback`.

The bit-exact (`to_bits`) divergence guard of PR #91 runs on every non-`place`
request; `place` alone adopts DLL state and is suppressed while the DLL applies a
host result. Number transport is unchanged (see `m5-convoy-movement.md`).

## Transitional design and its limits

- The DLL keeps a copy of the pose; the host record is seeded from `bind_mobile`.
  `place` is the explicit transitional adoption path and also covers deferred
  death-motion writers: it is **not** complete D1.
- A mobile of these kinds without a host ID is fatal at the boundary
  (`debug_fatal`). In the Lebanon census none occurred.
- Host records persist and are never removed (removal is later D1).
- The legacy bodies remain under `#else` for builds without
  `EECH_CAMPAIGN_HOST_MOVEMENT`.
- `person_begin` keeps the retained distance calculation that serves guide and
  animation policy in `ps_move.c`; that policy stays in the DLL.

## Evidence

Revisions; they must not be conflated:

| Revision | Full SHA | Role |
|---|---|---|
| `5f951122` | `5f951122fd477bf0b8e3b5dc1ae4647f48a4b85b` | base (PR #91 merge) |
| `03110344` | `0311034417e4a6a931bada33a532260ef1a28ecc` | valid PR #91 baseline; Tacview `3bd17b64...` reused, no baseline rerun |
| `b4399344` | `b4399344edd1859b517b526604d1d923d7b40318` | code commit and the settled native build; tree `31ab16de59e744f0b1d28fff0308a715950deff5` |
| iteration build | n/a (uncommitted tree, `target/sm-iter1`) | diagnostic build used for the census and first witness; **not evidence of acceptance** |

Settled native build: `BUILD-INFO.txt` reports commit `b4399344`, 0 uncommitted
build inputs, image
`sha256:bb10e44a20e9c6304b3a318902bcf0f8ec8175c162862240ce25c34bd7e562be`,
rustc 1.98.1, cargo 1.98.1, mingw gcc 12-posix, Lua 5.1.5,
`x86_64-pc-windows-gnu` release, on the fresh volume `eech-sm-settled-20261022`.

| Artifact | SHA-256 |
|---|---|
| `eech_dc.dll` | `ac29398ee59401199ced6906892480cf535c5bce8a83d886683d3fddf6f95361` |
| `eech-world.exe` | `648f9788c832ac3abe1a73eea94a0c0bdb841753b1f778f61f23434d549aa0c1` |
| `lua.dll` | `b224cb2eacdea06fcb7fe273979e660fbef0c1a9c65be53c7be9fc0a4571daca` |
| `lua/surface-movement.lua` | `e5610fd442ca00e6c87e340fdc65ce2c8a46cd590ce14d0883d767711d6ae71f` |
| baseline `recording.acmi` (PR #91 runtime, local only) | `3bd17b647665b558c7378206a3717c94e9cf771d4781d78e0b2aaae9a7d2d19b` |

### Public witness: held infantryman

`lua/surface-movement.lua` through `tools/surface-movement-windows.ps1`'s modes,
Lebanon root (a private copy, `lebanon-private`), seed 1, 100 ms frames. The
scenario goes host Lua -> `eech_dc` -> `move_vehicle`.

- **missing:** `dc.boot` without `move_vehicle` is refused: `runtime error: eech: missing host move_vehicle callback`.
- **control:** the subject is `eech-world:3460` (a person), selected at 18.7 s.
- **hold:** from its first step the host declines to move `eech-world:3460` for
  79 s and answers with the pose it holds. 1582 held steps; held displacement in
  the campaign exactly 0.000 m; no waypoint change while held. After release it
  moved 13.142 m and the campaign advanced the waypoint `4295058708` to
  `4295058709` at 97.9 s; no loss intervened.

The result is identical on the iteration build and on the settled `b4399344`
build (settled run: `missing` from the wrapper, then `control` and `hold` invoked
directly with `select=eech-world:3460`, hold 79 s). The hold is a host declining
to step, not a physical obstruction; it shows the seam is causal (host state
changes the campaign's arrival), not that the world models blockage. The witness
also accepts a host-observed `physical` loss after release for a person.

### Census (iteration build, Lebanon 1 simulated hour)

```
census routed bound=489 moved_10m=0   arrived=0
census ship   bound=36  moved_10m=0   arrived=0
census person bound=140 moved_10m=140 arrived=37
steps person_begin 5018584, person_end 4934037, routed:move 1702001, routed:stop 183
```

- **Infantry:** 140 bound, all 140 moved at least 10 m, 37 arrived.
- **Ships: no public ship-movement witness or observed runtime coverage is
  claimed.** In the Lebanon 1 h iteration census 36 ships are bound and zero
  `ship_move` steps occurred. That census does not show whether a ship steps in
  the longer Georgia or Lebanon pack runs, and no independent evidence on that is
  held. Ship coverage is the focused
  `movement.rs` tests only (acceleration ahead with the frame delta used only
  for the motion vector, the g-limited turn towards an abeam waypoint, the
  near-waypoint legacy branch, map bounds, adoption). No public ship coverage
  was invented.
- `routed moved_10m=0` is a census artefact, not a finding: the witness records
  displacement only for `ship_move` and person steps, never for routed ones
  (1,702,001 routed `move` steps ran). Routed motion is covered by the PR #91
  witness and the pack.
- The census was run once, on the iteration build. The lead deliberately stopped
  a second, redundant census on the settled build; it was **not** repeated (see
  Interruption record).

### Tests

- Linux, pinned image, **full workspace** `cargo test --locked` (after
  `cargo fmt`): all pass, including `load_dc.rs` 2/2, `host_callbacks.rs` 2,
  `eech-world` lib 20 and bin 16 (`reference/m5-surface-movement/linux-workspace-postfmt.log`).
  An earlier `-p eech-world`-only run reported 2 `load_dc` failures because the
  `eech_dc` cdylib was not built in that target directory; that is an environment
  gap of that invocation, not an unsupported platform.
- Windows host: `cargo clippy --locked --all-targets -p eech-world -- -D warnings` clean.
- **No genuine red phase** for the physical-step unit tests: they passed on their
  first run. They characterise ported maths, so the settled regression pack, not
  those tests, is the bit-identity proof for the routed refactor. The new
  error-path tests (unknown id, kind mismatch, out-of-order and pending
  requests, foreign pose) are direct.

### Regression pack and Tacview

Pack revision `b4399344edd1859b517b526604d1d923d7b40318` (0 dirty build inputs),
run 2026-10-22, accepted baseline `windows-retail v1`. The ONE settled pack
(not restarted, no additional ABI or baseline run). Command:

```
tools/regression-pack.py --georgia E:/eech-runs/roots/georgia --lebanon E:/eech-runs/m2/lebanon-root --lebanon-repeat E:/eech-runs/m2/lebanon-m1-copy --bin target/sm-settled --out E:/eech-runs/m5-d1/surface-movement-20261022/settled-pack
```

Result: **REGRESSION PACK: PASS**, exit 0, 40 checks, 40 PASS, 0 failed. Claims:
17 PASS directly protected, 12 PASS baseline-sensitive, 8 NOT COVERED and 8
evidence-only, all unchanged from the accepted pack.

- Georgia and Lebanon metrics at every checkpoint: IDENTICAL to `windows-retail v1`.
- Lebanon reference (two identical roots, 1179 input files, run 1 and run 2, 2,517 s):
  `recording.acmi`, `observations.jsonl` and `metrics.json` byte-identical across
  both runs; 39 of 39 campaign expectations hold; 31 of 32 supply deliveries
  complete (0 unattributed); observation checks PASS.
- Tacview: `recording.acmi` is 117,690,861 bytes, SHA-256
  `3bd17b647665b558c7378206a3717c94e9cf771d4781d78e0b2aaae9a7d2d19b`, equal to the
  PR #91 baseline recording (both runs hashed locally). The comparison is by bytes
  and structure; no recording was reviewed visually.
- **Classification of differences against the baseline: none.** Nothing to classify
  as regression, approved defect correction, intentional semantic change or
  input/world variation. No golden output was re-recorded and no tolerance was
  widened. Because no ship step was observed in the 1 h census, byte identity is not
  claimed as evidence for `ship_move`; it is the proof for the routed refactor and for the infantry
  path (about 5.0 M `person_begin` steps in the 1 h census; the pack runs 3 h).

The report is committed byte-exact as `reference/m5-surface-movement/pack-report.md`
(SHA-256 `1d6d10e20391c2a7756364ebff45600b529216824744e64cf22c653fa28221ed`) with
`pack-report.json` and `reference-summary.txt`; the full pack output tree stays
local.


### Retained failed iterations

| Iteration | Failure | Cause and fix |
|---|---|---|
| iter1 build | `rv_hostmv.c:81:36` compile error | the `vehicle` parameter shadowed the `vehicle` typedef in `begin_step`; the parameter was renamed `en`. Log kept: `build-iter1-failed-vehicle-shadow.log` (local) |
| first test run | `settled.pose.position` nil | the result is flat; the assertion read `settled.position`. The error was in the embedded Lua fixture in `crates/eech-world/tests/host_callbacks.rs`, not in `lua/surface-movement.lua` |
| first test run | wrong count | `stop` on a Person now reports "is not a request for", so the expected count went 6 to 5 |
| wiring | field mismatch | `ship_move` read a different field than `MovementStep` serialised; it now reads `desired_velocity` (found in review, fixed before the commit) |

### Interruption record

During the settled run the parent deliberately stopped the Claude CLI process
(33040, the implementation session, not the witness wrapper) and the redundant
private census (23672) after verifying identities, to avoid duplicate checking.
This is neither quota exhaustion nor a failed scenario. The PowerShell witness
wrapper (45092) then exited 1 with `census exited -1` because its child was
terminated; its partial output is kept locally as
`settled-witness/census-stopped-by-lead.*` and is not evidence. The ONE settled
regression pack was left untouched and its result is the one reported here. An earlier settled wrapper run on the shared
`lebanon-root` was aborted by the author to avoid a writable-root conflict with
the concurrent pack (`settled-witness-aborted-shared-root.log`).

### Limits

The host steps the entity synchronously in the same frame, so the order of
consequences is unchanged. Alive ship and person motion is host-owned; the DLL
still owns the entity, the terrain sample (D3), death/lifecycle, combat (D2) and
remaining aircraft physical movement. The result is the same reference physics, not a DCS
adapter. Visual inspection of Tacview and a live DCS run (M6) are not claimed.

## Remaining coupling

- The DLL's pose copy and the `place` request exist because the entity still
  lives in the DLL.
- `ps_anim.c`'s targeting attitude write is a combat coupling (D2).
- The terrain sample (`get_3d_terrain_elevation`) and `point_inside_map_area`
  stay in the DLL (D3).
- Death motion, force creation/removal and lifecycle/loss handling (later D1).
- No public ship-movement witness exists; runtime ship coverage is not claimed
  (the Lebanon 1 h census saw zero ship steps).

## Author handoff

One inline author pass against SRP, DRY, naming, coupling, dead code, KISS,
boundary and panic safety. It is not an independent review; the parent is the
sole independent reviewer.

- **SRP:** the DLL decides guide, formation, hold/engage, animation and gathers
  inputs; the host holds the pose record and steps it. `movement.rs` keeps the
  records and the guard; `ship_step.c`, `person_step.c` and `convoy_step.c` are
  the maths only; `rv_hostmv.c` is the one DLL-side seam.
- **DRY:** one `Movers::step` request path for all kinds, one set of exact-number
  reads, and the C arithmetic shared in `eech_arith.h`. `exact_numbers` and
  `nearest_f32` remain duplicated between `eech-world` and `eech-dc` (inherited
  from PR #91).
- **Naming:** `movement.rs` (renamed from `convoy.rs`), `ship_move`,
  `person_begin`/`person_end` follow `bind_mobile`/`move_vehicle`.
- **Coupling:** no pointer crosses the boundary; the step and result are copied
  primitives and an opaque host ID. `place`, the pose copy and the `ps_anim`
  write are the transitional coupling named above.
- **Dead code:** none newly dead in the host build; legacy bodies stay under
  `#else`.
- **KISS:** no framework, ABI project or per-class mock; the witness adds
  `missing`, `census`, `control` and `hold` modes. The `build.rs` was normalised
  to LF only to match the index.
- **Boundary and panic safety:** unknown id, kind mismatch, out-of-order or
  pending requests, foreign pose, unknown mode and a missing callback each fail
  loudly before mutation; callbacks are cleared on leave and error and panics are
  contained as errors.

Retained text evidence is in `reference/m5-surface-movement/`. No retail data,
binaries or recordings are committed.

## Commands

```
EECH_WIN_TARGET=<fresh volume> tools/build-windows.sh target/sm-settled
powershell -File tools/surface-movement-windows.ps1 -Root <lebanon root> -Bin <bin> -Out <out>
python -I tools/regression-pack.py --georgia E:/eech-runs/roots/georgia \
  --lebanon E:/eech-runs/m2/lebanon-root --lebanon-repeat E:/eech-runs/m2/lebanon-m1-copy \
  --bin target/sm-settled --out <out>
cargo clippy --locked --all-targets -p eech-world -- -D warnings   # Windows host
# Linux image, full workspace (builds the eech_dc cdylib that load_dc needs):
docker run --rm -v <worktree>:/src -v <target volume>:/target -e CARGO_TARGET_DIR=/target \
  -e LUA_LIB=/opt/lua-mingw -e LUA_LIB_NAME=lua -w /src/eech-campaign eech-build \
  sh -c 'cargo test --locked'
```

## Next bounded action

D1: remaining aircraft physical movement, creation/placement/removal and
death/lifecycle (including loss handling) in the host. The lead chooses the next
extraction.
