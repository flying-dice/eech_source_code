# M5: host-owned ground convoy movement (EEDCS-32)

**Scope.** One physical extraction toward criteria 2 and 3 of the
[M5 goal](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337)
(all six criteria remain unaccepted; the goal page is live and versioned, so read
its current text), on base `fd3a759c` (the PR #90 merge). The alive physical
movement of **routed vehicles** (ground convoys) is now stepped by the host, per
existing host ID, through the same host-callback seam as
[`m5-remaining-arrivals.md`](m5-remaining-arrivals.md). It claims no complete D1:
other mobile families, death motion and full creation/removal stay in the DLL.

## What moved

A new host callback, `move_vehicle`, answers each alive routed-vehicle movement
step. The campaign still owns routes, formation, hold/engage policy and waypoint
progression. The host owns the pose of the vehicle (position, velocity,
orientation) per host ID, steps it with the original C maths
(`crates/eech-world/csrc/convoy_step.c`, verbatim from `rv_move.c`/`vh_vec3d.c`),
and returns the stepped pose, which the DLL applies.

| Request | Meaning | Host does |
|---|---|---|
| `move` | an alive vehicle with a guide waypoint | steps the pose toward the waypoint |
| `stop` | the original stop branches (no guide, engage/not moving, invalid waypoint) | zeroes velocity, per branch |
| `place` | a pose setter wrote the vehicle (spawn, teleport, deferred death-motion writers) | adopts the DLL pose |

The original branch distinctions are preserved: the no-guide stop does not
recompute the motion vector, the other stops do; within planar squared distance
<= 1.0 the original skips acceleration, desired velocity, heading/terrain reads and
attitude change but still bounds the position and recomputes the motion vector;
`ROTATED` is set only when logged angular differences exceed the threshold and is
never cleared unconditionally; map bounding follows the active move path.
Result application order is: velocity, attitude, `ROTATED` (Tacview logging only),
position (inside the apply bracket), then the motion-vector recompute.

**Inputs carried to the host** are copied primitives: the host ID, mode, the
desired and maximum velocity and acceleration, the waypoint-derived target, the
terrain sample and normal, map bounds and the elapsed time. No pointer crosses the
boundary and legacy layouts stay private. Terrain is still sampled by the DLL
(D3) and passed as input; the sample is taken at the original site, only in the
distance > 1 branch with terrain-follow enabled.

## Transitional design and its limits

- **The DLL keeps a copy of the pose.** The host owns the authoritative record per
  host ID. This is transitional coupling: the DLL still has the full physical
  entity, and the host record is seeded from the `bind_mobile` descriptor.
- **Divergence guard.** On `move` and `stop` the DLL sends its pose only for
  bitwise (`to_bits`) comparison against the host record. Any mismatch is an
  error, which makes the DLL `debug_fatal`. The guard never overwrites host
  authority from DLL state; `place` is the only request that does, and it is the
  explicit transitional path. It is suppressed while the DLL applies a host
  result, and for non-routed entities and entities without a host ID.
- **PLACE is not complete D1.** The generic pose setters also cover the deferred
  death-motion writers. This cut is limited to alive convoy motion; it does not
  make every placement source campaign policy or every lifecycle step host-owned.
- A routed vehicle **without a host ID is fatal** at the boundary
  (`debug_fatal`), as for arrivals. In the retained Lebanon run none occurred.
- A **missing `move_vehicle` callback** refuses to boot with
  `eech: missing host move_vehicle callback`; an **unknown host ID** fails with
  `host.move_vehicle: unknown host id <id>`.
- The 2D predicate `(wp.x^2 + wp.z^2) > 1.0` is duplicated in the DLL (to decide
  whether to sample terrain) and in the host step. Which STOP branches fire in the
  headless scenario is not known; they are covered by focused host tests only.
- The host step runs under the rounding mode the engine entry sets
  (`FE_TOWARDZERO`), inherited across the synchronous callback exactly as the
  legacy code ran. It is not re-established by the host.
- Host records persist and are never removed (removal is later D1).
- `eech-engine`'s `checked_floats` still reports "non-finite supply arrival
  {name}" for movement inputs routed through it; movement has its own
  "non-finite movement {name}" and "move_vehicle host returned a non-finite pose".
  The stale message is cosmetic and was left alone.
- `crates/eech-world/tests/load_dc.rs` has two Docker-only failures (not caused by
  this change; probably a missing `libeech_dc.so` precondition). Not confirmed.

## Number transport

The divergence guard compares bits, so every physical numeric input must cross
Lua without change. Two root causes were found by running it (see the failed
iterations below):

1. mlua (Lua 5.1) turns a Lua number within `f64::EPSILON` of an integer into
   `Value::Integer` for `Value` and `Vec<f32>` reads, so `-0.0` and tiny values
   (for example `-6.71e-17`) became `0`. Direct scalar `get::<f32>` is exact.
2. `as f32` follows MXCSR. Under `FE_TOWARDZERO` it truncates, giving 1 ulp
   narrowing errors.

Fix, safe Rust (`eech-dc` keeps `unsafe_code = "forbid"`): a Lua-side `%.17g`
formatter cached in the registry looks the field up **inside Lua**, so no lossy
`LuaValue` read precedes the conversion; `nearest_f32` then takes the nearest of
the cast and its two neighbours. All numeric physical step inputs (pose, terrain,
bounds, velocities) use it. Zero canonicalisation, tried at iter3, was removed:
signed zero can change `atan2` at the axes, so it is not bit-equivalent to the
original. **No universal bit-exactness is claimed from `%.17g`**: the msvcrt
formatter was not proved exact; `nearest_f32` absorbs a 1 ulp f64 truncation only
because the values originate as f32. The helper is duplicated between
`eech-world` and `eech-dc` (a limitation). Focused tests cover signed zero, tiny
values, 20k finite samples and edges, scalar and array reads, under both the
nearest and toward-zero rounding modes.

## Evidence

Revisions; they must not be conflated:

| Revision | Full SHA | Role |
|---|---|---|
| `fd3a759c` | `fd3a759cff1b7cf06f2d37724db5ae4fd94837d8` | base (PR #90 merge); baseline runtime `cdcbd72e` |
| `03110344` | `0311034417e4a6a931bada33a532260ef1a28ecc` | the only native build under test; tree `f1c8a1a63fcbfae13772f75ee0fd61b8f28bb289` |
| iter1 to iter8 | n/a (uncommitted trees) | diagnostic builds; **not evidence of acceptance**, retained as failure evidence |

Settled native build: `BUILD-INFO.txt` reports commit `03110344`, 0 uncommitted
build inputs, image
`sha256:bb10e44a20e9c6304b3a318902bcf0f8ec8175c162862240ce25c34bd7e562be`,
rustc 1.98.1, Lua 5.1.5, `x86_64-pc-windows-gnu` release, on a fresh
`EECH_WIN_TARGET` volume.

| Artifact | SHA-256 |
|---|---|
| `eech_dc.dll` | `90428eb4d2f9756452898439f7a721e56778aa24bff70bae361fdea89da7b821` |
| `eech-world.exe` | `174e396b3c72980e92667ecf1240a3742a3d00263c8c36f8152342877f6fddd4` |
| `lua.dll` | `b224cb2eacdea06fcb7fe273979e660fbef0c1a9c65be53c7be9fc0a4571daca` |
| `lua/convoy-movement.lua` | `d83927a3edefb844c220c701b82a3dace7221a7b7cfbf235aeb09da7b061c8e5` |
| baseline `recording.acmi` (PR #90 runtime), 117690861 bytes (local only) | `3bd17b647665b558c7378206a3717c94e9cf771d4781d78e0b2aaae9a7d2d19b` |

### Public witness: held convoy (settled build)

`tools/convoy-movement-windows.ps1`, Lebanon root, seed 1, 100 ms frames, exit 0.
The scenario goes through host Lua -> `eech_dc` -> `move_vehicle`.

- **missing:** `dc.boot` without `move_vehicle` is refused: `runtime error: eech: missing host move_vehicle callback`.
- **control:** the reference host moves every vehicle. The subject is the first routed vehicle that moved at least 10 m and then passed the reference arrival gate: `eech-world:3574`, first moved at 156.6 s, arrived at 163.4 s.
- **hold:** from its first move step the host declines to move `eech-world:3574` for 67 s (the control span plus a 60 s margin) and answers with the pose it holds. 671 steps were held; displacement in the campaign was exactly 0.000 m; the waypoint did not change while held. After release it moved 41.916 m and the campaign advanced the waypoint `4295047215` to `4295047214` 6.8 s after release (230.5 s).

This shows that host state affects the campaign's arrival: with the host not
moving the vehicle the campaign does not arrive, and with it moving again it
does. The hold is **a host declining to step**, not a physical obstruction: it
proves the seam is causal, not that the world models blockage. The expectation
"a released convoy always advances" is not universal; the witness also accepts a
host-observed `physical` loss of that identity after release (and prints which
occurred). A real physical loss intervening would not be a failure of the cut.

The failure paths are covered by `crates/eech-world/tests/host_callbacks.rs`
(`move_vehicle_refuses_an_unknown_id_a_foreign_pose_and_an_unknown_mode`) and the
`convoy.rs` unit tests (7 tests: stepping, stop branches, adoption, bit-exact
divergence rejection, unknown id).

### Regression pack and Tacview

Pack revision `0311034417e4a6a931bada33a532260ef1a28ecc` (0 dirty build inputs),
run 2026-10-22, accepted baseline `windows-retail v1`. Command:

```
tools/regression-pack.py --georgia E:/eech-runs/roots/georgia --lebanon E:/eech-runs/m2/lebanon-root --lebanon-repeat E:/eech-runs/m2/lebanon-m1-copy --bin target/conv-settled --out E:/eech-runs/m5-d1/convoy-movement-20261022/settled-pack
```

Result: **REGRESSION PACK: PASS**, exit 0, 40 checks, 40 PASS, 0 failed. Claims:
17 PASS directly protected, 12 PASS baseline-sensitive, 8 NOT COVERED and 8
evidence-only, all unchanged from the accepted pack.

- Georgia and Lebanon metrics at every checkpoint: IDENTICAL to `windows-retail v1`.
- Lebanon reference (two identical roots, 1179 input files, run 1 and run 2):
  `recording.acmi`, `observations.jsonl` and `metrics.json` byte-identical across
  both runs; 39 of 39 campaign expectations hold; 31 of 32 supply deliveries
  complete (0 unattributed); observation checks PASS.
- Tacview: `recording.acmi` is 117,690,861 bytes, SHA-256
  `3bd17b647665b558c7378206a3717c94e9cf771d4781d78e0b2aaae9a7d2d19b`, equal to the
  PR90 baseline recording. The comparison is by bytes and structure; no recording
  was reviewed visually.
- **Classification of differences against the baseline: none.** There is nothing
  to classify as regression, approved defect correction, intentional semantic
  change or input/world variation. No golden output was re-recorded and no
  tolerance was widened.

The pack was run on the settled candidate only (iterations are in the next
section). The report is kept at
`E:/eech-runs/m5-d1/convoy-movement-20261022/settled-pack/report.md` (SHA-256
`9f39b443e29b7f05bea49e97dbd53ca3f4b003088dd3a28aa1f9b0be47d57bb8`), not committed.

### Retained failed iterations

Each failure was reproduced in the host's divergence guard on the Lebanon 1 h
census, with the output kept under `E:/eech-runs/m5-d1/convoy-movement-20261022/`
(not committed):

| Iteration | Failure | Cause and fix |
|---|---|---|
| iter1, iter2 | `attitude[2] (0 against -0)` | signed zero lost in transport |
| iter3 | `(-0 against 0)` | zero canonicalisation made it worse and is not bit-equivalent to the original; removed |
| iter4 | `attitude[3] (0 against -6.71e-17)` | mlua collapses values within EPSILON of an integer on `Value`/`Vec<f32>` reads |
| iter5 | build failure | raw `mlua::ffi` needs `unsafe`; replaced by the safe Lua formatter |
| iter6 | 1 ulp trip | `as f32` follows `FE_TOWARDZERO`; fixed by `nearest_f32` |
| iter7 | passed the census | the lossy `LuaValue` read remained (lead review) |
| iter8 | passed | lookup moved inside Lua for all inputs; `census-iter8.out` is byte-identical to the baseline census |

The first held-convoy run (`witness-iter8/`) selected `eech-world:3399`, which was
physically lost at 149.5 s before any waypoint query; the hold run then failed on
a nil waypoint (exit 1). That is an unusable selection, not a defect of the cut;
the subject is now chosen from a control run as a vehicle that demonstrably moves
and passes the arrival gate. The failed run is retained, not counted.

### Limits

The host steps the vehicle synchronously in the same frame, so the order of
consequences is unchanged. The DLL still owns the entity and its physical
simulation (D1 coupling), the terrain sample (D3), and every non-routed mobile
family. The result is the same reference physics, not a DCS adapter. Visual
inspection of Tacview and a live DCS run are not claimed.

## Remaining coupling

D1 still to do: other mobile families, death motion, force creation/removal and
complete lifecycle and loss handling. D2 (combat, perception) and D3 (terrain) are
separate. The DLL's pose copy and the `place` request exist only because the
entity still lives in the DLL.

## Author handoff

One inline author pass against SRP, DRY, naming, coupling, dead code, KISS,
boundary and panic safety. It is not an independent review.

- **SRP:** the DLL decides route, hold/engage and progression and gathers inputs;
  the host holds the pose record and steps it. `convoy.rs` keeps the record and the
  guard; `convoy_step.c` is the maths only.
- **DRY:** one request path (`Request`) for move/stop/place and one exact-number
  read path in each crate. `exact_numbers` and `nearest_f32` are duplicated between
  `eech-world` and `eech-dc`; sharing them was not attempted, and the duplication
  is noted as a limitation.
- **Naming:** `rv_hostmv` and `move_vehicle` are new and consistent with
  `bind_mobile`/`observe_loss`.
- **Coupling:** no pointer crosses the boundary; the descriptor is copied
  primitives and an opaque host ID. The `place` path and pose copy are the
  transitional coupling named above.
- **Dead code:** the legacy bodies remain under `#else` for builds without
  `EECH_CAMPAIGN_HOST_MOVEMENT`; none is newly dead in the host build. The raw-FFI
  and canonicalisation drafts were removed.
- **KISS:** no framework, ABI project or per-class mock; the witness adds a
  control, a hold and a missing mode.
- **Boundary and panic safety:** a missing callback, unknown ID, foreign pose,
  unknown mode and non-finite input or result each fail loudly; callbacks are
  cleared on leave and on error, and callback panics are contained and returned
  as errors rather than unwinding through C.

Retained text evidence is in `reference/m5-convoy-movement/`: `BUILD-INFO.txt`,
`identities.txt`, the three witness stdout files, `pack-report.md/json` and
`reference-summary.txt`. No retail data, binaries or recordings are committed.

## Commands

```
EECH_WIN_TARGET=<fresh volume> tools/build-windows.sh target/conv-settled
powershell -File tools/convoy-movement-windows.ps1 -Root <lebanon root> -Bin <bin> -Out <out>
python -I tools/regression-pack.py --georgia E:/eech-runs/roots/georgia \
  --lebanon E:/eech-runs/m2/lebanon-root --lebanon-repeat E:/eech-runs/m2/lebanon-m1-copy \
  --bin target/conv-settled --out <out>
cargo clippy --locked --all-targets -p eech-world -- -D warnings   # Windows host
cargo test --locked -p eech-world --bin eech-world                 # Linux image: 16/16
```

## Next bounded action

D1: the remaining mobile families, death motion, force creation/removal and
complete lifecycle and loss handling in the host.
