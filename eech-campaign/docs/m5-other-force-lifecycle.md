# M5: host-owned wreck, fall, sink and crash motion of the other forces (EEDCS-37)

**Status: runtime commit `1a8080c3` is natively validated; delivery is a local branch
awaiting the independent review, publication and the lead's formal approval. D1 and M5
are not complete.**

**Scope.** One physical extraction under D1 (EEDCS-15), on base `11dcf588` (the PR95
merge), following [`m5-helicopter-lifecycle.md`](m5-helicopter-lifecycle.md). The dying
motion of the routed and anti-air ground vehicles, of ships and of fixed-wing aircraft
is stepped by the host, per host ID, through the same `move_vehicle` seam. The original C
arithmetic and its order are kept. The campaign still decides when a unit dies and what
its death means; the host's retained physical state drives the DLL's response. A live
DCS run is M6 and is not claimed.

## What moved

`move_vehicle` gains fifteen lifecycle modes (values 22 to 36; Lua snake_case names).
Each is a step of the original per-frame death code, called in the original order.

| Mode | Kind | Host does | `flags` |
|---|---|---|---|
| `gv_death_candidate` | routed, AA | wreck slide, before the terrain refresh: candidate x/z (no map bound) | 0 |
| `gv_death_decay` | routed, AA | after the refresh: speed decay, heading turn, `y = elevation + cg`, attitude | 0 |
| `gv_fall_candidate` | routed, AA | falling, before the refresh: candidate position, all axes (no map bound) | 0 |
| `gv_fall_gravity` | routed, AA | after the refresh: gravity, the vector and the speed | 0 |
| `gv_fall_settle` | routed, AA | contact, splash, sunk, ground impact; the settled position, vector and speed are **staged** host-side and nothing is applied | bit 0 contact, 1 splash, 2 sunk, 3 ground impact |
| `gv_fall_vector` | routed, AA | writes the staged vector (zero when sunk, `y` zero on ground) | 0 |
| `gv_fall_commit` | routed, AA | finalises the staged speed, then the staged settled position | 0 |
| `gv_impact` | routed, AA | impact shove; the DLL drew the float speed | 0 |
| `ship_sink` | ship | candidate position, drag, sink rate, pitch/roll drift, attitude; returns the height | 0 |
| `fw_death_attitude` | fixed wing | after the gravity step: heading, pitch and roll ease toward the velocity direction | -1 gate/draw mismatch, 0 nothing, 1 eased, 3 eased and the roll draw was consumed |
| `fw_settle` | fixed wing | settle by terrain class, double 3.0 river depth, no rotor; a bury zeroes the vector only | 0 |
| `fw_crash_candidate` | fixed wing | crash slide, phase one: candidate x/z, map-bounded, `y` held | 0 |
| `fw_crash_velocity` | fixed wing | phase two: candidate `y`, friction and slope push from the NEW normal; retains the post-slope speed | bit 0 stopped (post-friction speed under 1) |
| `fw_crash_attitude` | fixed wing | phase three: pitch and roll ease toward the NEW normal | bit 0 at rest (speed and both deltas at most 0.01) |
| `vehicle_vector_seed` | routed, AA, ship | ownership claim of the motion vector (see below) | 0 |

(`vehicle_vector_seed` is value 29; the table groups modes by kind, not by value.)
A fixed wing also uses the helicopter modes `fall`, `fall_gravity`, `contact` and
`impact`, which the host dispatches by member kind; the arithmetic is type-agnostic and
the rotor is ignored. Anti-air vehicles now take the routed movement kind
(`gp_roster.c`), so they use the routed path.

- **Original arithmetic, copied.** `crates/eech-world/csrc/death_step.c` (new) and
  `hc_step.c` copy the wreck, fall, sink and wing expressions; `movement.rs` holds the
  records and guards.
- **Host-retained authority.** The host keeps, per host ID: the motion vector (claimed
  once by `vehicle_vector_seed`, released only by live movement), the ground/AA wreck
  candidate, the staged fall results (settled position, vector, speed), the ship's sink
  state, the fixed wing's fall candidate, contact altitude and crash candidate with the
  post-slope speed, and the **attitude**. The DLL no longer sends or overwrites angles for
  these modes: the host derives heading, pitch and roll from its own retained attitude.
  Place, `rotor_set` and the vector seed are the explicit adoptions; every other step
  compares the DLL's copy bit-exactly with the retained record and refuses a difference
  before mutating anything.
- **Guards.** While a fall, a crash slide or a death slide is open, any request except
  Place and the next step of that sequence is refused. While a death candidate is open
  only `gv_death_decay` (and Place) is accepted. Live movement releases the vector claim
  and any death candidate. A step of the wrong kind is refused.
- **Required public fields.** The vehicle lifecycle modes must return `motion_vector`;
  the four flag-bearing modes (`gv_fall_settle`, `fw_death_attitude`,
  `fw_crash_velocity`, `fw_crash_attitude`) must return `flags`. `eech-dc` `module.rs`
  rejects an answer without them. Valid `flags`: `gv_fall_settle` in {0, 1, 3, 5, 7, 9},
  `fw_death_attitude` in {-1, 0, 1, 3}, `fw_crash_velocity` and `fw_crash_attitude` in
  {0, 1}; every other mode 0.
- **ABI layers change in lockstep:** C `campaign_movement_*` (`rv_hostmv.h/.c`),
  `eech-engine-sys`, `eech-engine`, `eech-dc` `call_move_vehicle`, and `eech-world`
  (`lifecycle_from`, `movement.rs`, `death_step.c`, `hc_step.c`, `build.rs`).
- **DLL side.** `rv_move.c` (routed), `aa_move.c`, `sh_move.c`, `fw_move.c` call
  `campaign_movement_*` under `EECH_CAMPAIGN_HOST_MOVEMENT`; the original bodies stay in
  the `#else` branch, so builds without the flag are unchanged. A hosted unit without a
  host ID fatals rather than falling back to DLL physics.

### Phase ordering

- **Routed/AA wreck slide:** `vehicle_vector_seed`; `gv_death_candidate`; the DLL's
  terrain refresh; `gv_death_decay`. Heading turns by `-0.1 * pi * dt`.
- **Routed/AA fall:** `vehicle_vector_seed`; `gv_fall_candidate`; the refresh;
  `gv_fall_gravity`; `gv_fall_settle` (flags, staged); the DLL's splash, sunk and impact
  effects; `gv_fall_vector`; `gv_fall_commit` last. Nothing of the settle is applied
  until the commit. `gv_impact` is the separate impact shove.
- **Ship sink:** `vehicle_vector_seed`; `ship_sink` (returns the new height; the fire
  clears and the destroy test stay with the campaign and read it). Pitch and roll drift.
- **Fixed wing dying:** `contact(false)` on land crashes (below); otherwise
  `fall`, the DLL's terrain sample, `fall_gravity`, `contact(true)`, `fw_settle` if in
  contact, then `fw_death_attitude` (with its optional roll draw).
- **Fixed-wing crash slide:** `fw_crash_candidate`, then `fw_crash_velocity` (stopped
  flag), then `fw_crash_attitude` (rest flag). The post-slope speed is retained even after
  a Place clears the scalar, so a 0.5 m/s slide is not "at rest".
- **Fixed-wing impact:** `impact` along the attitude x axis.

## Draw precision

`frand1 ()` returns a double. PR95 carried the helicopter tumble draws as doubles end to
end after a float transport broke one observation. This task adds one more draw, the
fixed-wing roll draw of `fw_death_attitude`, and keeps it double end to end the same way
(`hc_draws[2]`, last in the C step; `eech_fw_death_attitude` takes `double roll_draw`; an
exact `%.17g` f64 Lua read). The branch merged PR95 and kept the already-ported precision
edits; the merged-in duplicate tumble tests were removed (below).

## Evidence

All for runtime commit `1a8080c376558932fc9e524390dca73818ba52df` (tree `74b7c81b`,
0 dirty inputs), base `11dcf588`. The compact portable record is
[`../reference/m5-other-force-lifecycle/`](../reference/m5-other-force-lifecycle/)
(`reference-summary.txt`, `identities.txt`, `BUILD-INFO.txt`, pack report, the parent's
reference hashes, Linux logs). The Tacview recording and observations are identified by
hash there, not copied.

| Evidence | Result | Source |
|---|---|---|
| Linux container, `rust:trixie` (logs record no version; a later version query of the local image printed rustc 1.99.1): `fmt` clean; `clippy --locked --all-targets -D warnings`, no warnings; `cargo test --locked -p eech-engine -p eech-world -p eech-dc`. **Run at `e55fbe1e`, not repeated at `1a8080c3`** | `eech-dc` 3 and 2; `eech-engine` 6; `eech-world` library 68, binary 18, `host_callbacks` 2, `load_dc` 2; 0 failed | `linux-fmt.log`, `linux-clippy.log`, `linux-test-final.log` |
| Production parity `e55fbe1e` to `1a8080c3` | `git diff e55fbe1e 1a8080c3 -- aphavoc modules eech-campaign/crates` is empty; `1a8080c3` only removes the two merged-in duplicate tumble tests from `movement.rs` | git |
| Crash-speed mutation (intentional) | **Expected red**: `a_wing_crash_slide_keeps_its_post_slope_speed_through_the_clear_and_commits_the_staged_candidate` fails at `movement.rs:3438`; the final run above is the green. Not a valid final check | `linux-test-crash-speed-mutation-EXPECTED-FAIL.log` |
| Run after the PR95 merge | **Failed, compile error E0428**: two PR95 tumble tests duplicated the hand-ported copies. Corrected by source parity in `1a8080c3`; **no Linux rerun followed** | `linux-test-after-merge-E0428-FAILED.log` |
| Retail MinGW build, fresh volume, pinned image `sha256:bb10e44a…562be`, rustc 1.98.1, gcc 12-posix, Lua 5.1.5 | exit 0, 0 dirty inputs; `eech_dc.dll` `30bb9e0b…`, `eech-world.exe` `b9e4d564…`, `lua.dll` `b224cb2e…` | `BUILD-INFO.txt` |
| Full regression pack, one run | exit 0, PASS, 40 of 40 checks, `windows-retail v1`, Georgia and Lebanon exact PASS, pack revision `1a8080c3` | `pack-report.json`, `pack-report.md` |
| Reference runs (the parent's independent hashing) | Tacview `3bd17b64…2d19b` (117,690,861 bytes) and observations `da26bab0…77cf` (85,716,447 bytes) match the retained decompressed bytes exactly, in both runs. `metrics.json` (35,942 bytes, `fa09d562…2587`) is identical in both runs and equals the baseline's JSON values (and its bytes after CRLF normalisation) | `independent-reference-hashes.json` |

The pack reports the build as differing from the accepted one in `eech_dc.dll`,
`eech-world.exe`, `lua.dll`, `lua/campaign.lua` and `lua/lifecycle.lua`; there is no
output difference to classify. No golden, tolerance or scenario changed. The baseline's
raw `metrics.json` file uses CRLF and **is not** byte-equal to the candidate's; the
comparison is by values and by CRLF-normalised bytes. The parent first asserted raw
equality, found the line-ending difference and corrected its checker; this is a checker
correction, not a candidate or baseline change. The Tacview recording was checked by hash
and was not visually reviewed.

## Coverage limits

- **No new public witness exists or is required by this task.** The behaviour is
  exercised through the host-owned Lua, public module and world path by the regression
  pack's three-hour wars (Georgia, Lebanon twice), which are outcome-equivalence checks:
  they show the migrated motion reproduces the accepted Tacview, observations and metrics.
  They do not hold or delay a single phase, so they do not isolate that host answers
  drive the DLL for each mode. Per-mode hit counts were not extracted from the pack.
- The Linux unit tests characterise the ported maths, guards and ordering (for example
  staged fall results applied only at the commit, the open-sequence refusals, the
  DLL-cannot-set-attitude rule and the crash-slide speed retention). Bit identity of the
  full war is the pack's job.
- The pack's blind spots stand (`coupled-engine`, `three-hours`, `one-war`, `captures`,
  `tacview-visual`, `linux`; 17 are listed in the report, 8 claims are evidence-only and
  8 not covered).
- Rare paths are not shown to be hit by the references; unobserved cases are unproven,
  not asserted.

## Remaining coupling

D1 and M5 are **not** complete. The following remain, and each needs its own boundary
decision:

- **D1:** creation, placement and removal authority; aircraft physical control and
  avoidance.
- **D3:** terrain sampling and its cache, including the refreshed sample the wreck, fall
  and crash steps consume.
- **Presentation coupling in the DLL:** smoke, fire, effects and sounds raised during
  dying, splash, sinking and crash, and the shared `frand1` RNG (only the draw values
  cross the seam).
- The DLL's copy of each vector and attitude remains because the entity still lives in the
  DLL, and the campaign and the physical world still share `eech_dc.dll` (the
  `coupled-engine` blind spot).
- Other direct writers are announced to the host only through the setters' `place`.
- Client/multiplayer entities are unsupported in a host build.

D2 (combat, targeting, perception) is not changed by this Task and comes later. The
campaign keeps deciding when a unit dies and the death consequences (group state,
removal, logistics).

## Commands

```
cargo fmt --all -- --check
cargo clippy --locked --all-targets -p eech-engine-sys -p eech-engine -p eech-world -p eech-dc -- -D warnings
cargo test --locked -p eech-engine -p eech-world -p eech-dc        # rust:trixie, liblua5.1-0-dev
EECH_WIN_TARGET=<fresh volume> tools/build-windows.sh target/other-force-lifecycle
python tools/regression-pack.py --georgia <georgia> --lebanon <lebanon> --lebanon-repeat <lebanon copy> --bin target/other-force-lifecycle --out <out>
```
