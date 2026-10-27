# D2: host-owned physical weapon aiming and weapon-system readiness (EEDCS-44)

**Status: implementation lane, on PR103 (`codex/d2-weapon-aiming`). Not accepted. Weapon execution and combat resolution remain D2 after this Task.**
Scope authority is the live M5 goal (Confluence 11534337) and the migration ledger: move physical combat into the host and keep target and weapon
choice, engagement and the fire decision in EECH_DC. The characterisation of the original code that this move is held to is
[`m5-aim-characterization.md`](m5-aim-characterization.md) with its retained evidence in
[`../reference/d2-aim-characterization/`](../reference/d2-aim-characterization/README.md); the evidence of the move is in
[`../reference/d2-aim-extraction/`](../reference/d2-aim-extraction/README.md).

## What moved

The physical question "where does a launcher's weapon system point, is it ready, and where must it aim" left the campaign for the host. In the
reference host it is C in EECH's own arithmetic, expression for expression, like the other hosted steps (`crates/eech-world/csrc/aim_step.c`, driven by
`crates/eech-world/src/aim.rs`):

- the **weapon-system-ready keyframe** (`update_entity_reversable_keyframed_value` for the aircraft and vehicle producers, the forwards and reverse
  setters behind `open_entity_weapon_system_ready` and `close_entity_weapon_system_ready`): the state, its advance by rate and time, the wrap, and the phase
  it was in;
- the **pitch device's vectors** (`get_pitch_device_to_target_vector`, the pitch-device part of the vector update): the target vector, the guided
  weapon's intercept vector, the unguided weapon's lead vector from the lead angle, the tracking-point vector, the ranges, the `(0, 0, -1)` defaults;
- the **guided-weapon intercept prediction** (`get_intercept_point`, three refinement iterations, the 0.001 speed gates, the strict target-slower test) and the
  HUD's alone-call of it;
- the **lead and ballistic prediction** (`get_lead_and_ballistic_intercept_point_and_angle_of_projection`): the tiny-range, off-map, stationary and moving
  paths, the three or five iterations, the communications lag, the map test, and the **ballistic table interpolation** inside it
  (`get_ballistic_pitch_deflection`: the floor pitch index, the closeness weights, the drop limit, the velocity tests). The point-blank guard that was
  patch W1 in the legacy source moved with the code: it is a line of the host's kernel, not a patch of the campaign;
- the **offsets the devices are asked to reach** (the bearing of the intercept vector in the pitch device's unrotated frame, bounded by the package's
  limits, with the ground-launched missile's slight-climb floor) and the **rate-limited step** toward them (shortest-way heading wrap for a full-circle
  device, the rate times the elapsed time, the new offsets);
- the **target facing of an infantry member** (`ps_anim.c`): the heading from the person's own position to the target's, the heading matrix, now a
  `move_vehicle` mode (`person_face`) so that the host's retained D1 record of the person stays the one authority for its attitude.

What is *not* here and stays in EECH_DC: which weapon is selected and which target it is aimed at; which package's devices are used; the ammunition and
damage state; whether the weapon system should be made ready (the open and close **commands** are the campaign's decision; the keyframe they act on is the
host's); the fire decision (the launch-angle threshold, burst and salvo timers, range, altitude and line of sight); the player and cockpit input; the
random generator and its seed; the projectile flight that produced the ballistic tables; and all of weapon execution and combat resolution.

## The boundary

One new, required host callback, `aim`, in the same Lua -> `eech_dc` -> host path as the other six (`dc.boot { aim = ... }`). Every request is a pure question; the
host keeps nothing between requests except the ballistic tables it is given once at boot (`aphavoc/source/entity/mobile/mb_hostaim.h`):

| Request | Campaign sends | Host answers |
|---|---|---|
| `table` | one weapon's range limit and its table as raw data (velocity test x pitch index x range, `(drop angle, flight time)` float pairs; a Lua string of little-endian IEEE-754 singles) | `done` |
| `ready_update` | the keyframe state, the rate (0.5), the elapsed time | `done`, the advanced state, the phase it was in (0 closed, 1 opening, 2 open, 3 closing) |
| `ready_command` | the state and `open` | `done`, the state |
| `required` | the pitch device's unrotated attitude, the launcher's intercept vector, the package's heading and pitch limits, `climb`; or, `unbounded` (no limits), the HUD helper's offsets, and with a `tracking` table (device position, tracking point, ballistic pitch) the player's cannon-tracking offsets | the required heading and pitch offsets |
| `step` | current and required offsets, the heading limits, both turn rates, the elapsed time | the deltas and the new offsets |
| `vectors` | the device viewpoint; the weapon's figures and the target's position, point and motion (or the tracking point); the map bounds; the communications lag | `valid`, the weapon, target and intercept vectors, the range; or `need_draw` |
| `lead` | the device position, the weapon, the parent speed, the target facts, the map bounds | `ok`, the intercept point, the angle, the time of flight (when `ok`); or `need_draw` |
| `ballistic` | the weapon, range, height or pitch, the two mode flags, the parent speed | `ok`, the aiming angle and the time of flight when `ok` |
| `intercept` | the device position, the weapon's speed figures, the target facts | the predicted point |

and one new `move_vehicle` mode, `person_face` (the person's pose as always, the target's position; the host answers the attitude and the heading in `angles[1]`).

**The shared random stream never leaves the campaign.** The lead prediction jitters its final range by `5 * sfrand1 ()`, a draw from the shared stream, once per
solve of a moving target. The host reaches that point and answers `need_draw`; the campaign takes ONE sample from its own stream at that point (a raw integer
in [0, 32767], which is `rand16 ()`) and asks the same question again with it; the host forms the double with the macro's own expression. The generator, its seed
and its state are never visible to the host, and the draw order is the original's. A host that has no such jitter never asks.

**What crosses are inputs of the host's model, not campaign ownership.** The device viewpoint, the target point, the positions and speeds, the weapon and package
figures and the map bounds are transitional mirrors of what a host that owns its world already knows; the reference (EECH) host aims with them as the inputs of its
own model and a host with another model (a DCS host) may answer from its own state and need none of them (`aim-provider.lua` mode `own_model` shows that shape on
the public path). The identities of the source and target are the host's own (absent when the roster holds none). No EECH pointer, entity index, list layout, seed or
campaign-internal table crosses the boundary. The ballistic tables are raw weapon data (the product of the campaign's still-core projectile simulation), handed over
once; a host with its own ballistics ignores them.

**No fallback.** A host build refuses a missing provider (`eech: missing host aim callback`; a C entry with no provider is a bad argument; a request with none is a
fatal). Every answer is validated in the campaign by mode (state, finite fields, the ready state in [0, 4], the phase in [0, 3], boolean results, no draw asked of a
request that cannot take one or already carries one, a bounded echo) and a malformed or failing answer stops the engine with the host's reason. The original
solvers are not in the DLL.

**Unsupported, refused rather than approximated:** the triangulated range of the player's gunship (a cockpit measurement); the campaign stops with a clear message
instead of asking. The player and cockpit paths are otherwise unchanged and were not exercised by the natural witnesses (see the characterisation).

## Evidence

All results are from the runs recorded under [`reference/d2-aim-extraction/`](../reference/d2-aim-extraction/README.md) (private roots, none in Git; fresh copies of the Lebanon inputs, the input manifest
IDENTICAL before and after each of the 31 runs, `checks/roots-manifest-*.log`). Every run is one native `eech-world.exe` process through the public path (`require ("eech_dc")`); every exit is the process's own
(`checks/native-exits.json`). Builds: the repository image in WSL2 `EECH-D2-Build`, fresh uniquely named cargo volumes, `tools/build-windows-aim-fixture.sh` and the unchanged `tools/build-windows.sh`, 0 uncommitted build inputs
(`builds/`). The reference the candidate is held to is the characterisation's: clean `0ee1fa46b`, Lebanon 600 s, 1,962 events, physical `8d5afe5ea932338f`, decision `4e542d029ba7362a`, raw SHA-256 `c4e9c2b1...ce62`.

| Source head | Role | Fixture build (`eech_dc.dll` / `eech-world.exe`) | Ordinary build |
|---|---|---|---|
| `f749878ce` | first complete extraction | `970b3fda...` / `78061571...` | `404fe213...` / `f0b6203f...` |
| `a0d3b6f9c` | + the HUD and player-tracking offsets through the host | `80ad327a...` / `ed61085e...` | `5b2be512...` / `1766f934...` |
| `d3c9a5c5e` | only the provider driver script and documentation differ from `a0d3b6f9c` (`git diff a0d3b6f9c d3c9a5c5e -- aphavoc modules eech-campaign/crates` is that one script) | `a8047384...` / `eaf8ccb3...` | `f24e664e...` / `349e9d88...` |

- **The complete original trace, reproduced from the host's own answers.** The fixture build replays the reference host's echo of its predictions, in its order, into the hooks the original solver bodies called (the
  statements the characterisation placed are unchanged), so the physical hash is a hash of what the host computed. Two fresh-root 600 s runs of `f749878ce` and two of `a0d3b6f9c`: native exit 0, 1,962 events,
  physical `8d5afe5ea932338f`, decision `4e542d029ba7362a`, and the **raw bytes** of each equal the original `r-a` / `r-b` traces (and one another): `comparisons/c-a-vs-r-a.json`, `c-b-vs-r-b.json`, `c-b-vs-c-a.json`,
  `c2-a-vs-r-a.json`, `c2-b-vs-r-b.json`, `c2-b-vs-c2-a.json`; the strict comparator exits 0 with 0 differing lines. The readiness counters (9,429,480 updates, 25,322 open and 3,387,517 close commands in 600 s) are inside that hash.
- **Failed experiments, kept.** The first development build `x-dev1` ran 167 s of the same scenario: the 60 s checkpoint's decision hash matched the original but the physical hash did not (`bfc92835...` against
  `b062306b...`): the fixture's lead hook read the prediction's own outputs (point, angle, time of flight) from the vectors answer, which does not carry them. The run was stopped by the lane (a native of its own, not the
  frozen pack); the cause was fixed by an echo event kind `lead_result` (fixture-only) and `x-dev2` reproduced the whole original trace (`comparisons/x2-a-vs-r-a.json`). Both are retained
  (`traces/lebanon-dev1-x1-a-killed-at-167s-physical-hash-differed.jsonl`, `lebanon-dev2-x2-a.jsonl`). A first fixture build of `f749878ce` counted 51 uncommitted inputs because the retained CRLF evidence logs
  are CRLF in the working copy and LF in Git for a git without `core.autocrlf=input`; it is kept as superseded and the clean-count build set that variable (`builds/`, `controls/build-aim-fixture-lf.sh`). The first
  ordinary 900 s attempt (`o2-ab`) stopped in its runner script (a backslash collapsed in the script) before it started a native process; nothing ran, its copied root is kept. The first provider runs on `a0d3b6f9c`
  stopped at a fixed frame count, one frame short of the characterisation's checkpoint, so their reports could not be equal by construction (the reference one differed by exactly one frame's calls); the driver was
  corrected (`5656422d0`), those runs stopped or set aside, and they were rerun.
- **The controlled person case, the same case.** `lua/aim-person-case.lua` (target, control, repeat) on the same 1,179-file inputs: native exit 0 each, **0 failed checks** (target 11 checks, 20 facing steps, 60 host steps;
  control 3 checks, 0 facings), the target repeat byte-identical to the target. Against the original `74ae6a96d` traces all 213 (194 control) events are equal except the named entry-point differences, which
  `controls/compare-person.py` maps exactly: the original asked the host to *place* the pose the campaign had turned (driver keys `place_in`/`place_out`, step mode `place`); the candidate asks the host to turn the person
  (`person_face`; keys `face_in`/`face_out`), so on every frame `face_out == place_out`, and the only other differences are the three check names that name the entry point and the verdict's `place=20`/`person_face=20`.
  The facing hook's counters and hashes, the selection, the headings and the 40 later normal steps (attitude in == the attitude the host last answered) are equal. This is a changed test entry point, explained here; it is
  not claimed to be byte-identical to the original.
- **Ordinary 900 s against the retained supplied-binary baseline** (`ab-base`, not rerun; ordinary shipping binaries `d3c9a5c5e`, `hours=0.25 record_every=10 checkpoint_every=3600 observe_every=60
  observe_supply=0`, seed 1, 100 ms): native exit 0; `recording.acmi` (11,363,845 bytes), `observations.jsonl` (6,629,273) and `metrics.json` (11,777) are **byte-identical** to the baseline's
  (`comparisons/o3-ab-vs-ab-base.json`); the observation structure check passes (900 frames, 0 failures).
- **Actual host authority (the fixture observes what the host answered).** `lua/aim-provider.lua`, Lebanon 300 s, seed 1, one run each, fresh roots, candidate fixture `d3c9a5c5e`:

  | Host | ready updates | ready commands | vectors (valid) | device steps | draws | weapon births | losses | report against the original at 300 s |
  |---|---|---|---|---|---|---|---|---|
  | reference host | 4,741,792 | 1,718,114 | 46,226 (42,710) | 15,304 | 3,516 | 185 | 6 | **equal**: 196 lines, 0 differing, physical `79efa18d48328638`, decision `cca4b35b00b2672c` |
  | ready keyframe advances at a quarter rate | 4,741,792 | 1,718,114 | 46,194 (42,678) | 15,033 | 3,516 | 182 | 6 | differs (51 lines): moving ready phases 755 against 215, vehicle not-ready rejections 243 against 63 |
  | devices step at a tenth of the turn rate | 4,741,089 | 1,716,633 | 46,792 (43,317) | 21,944 | 3,475 | 177 | 9 | differs (104 lines): launch-angle rejections of vehicles 10,207 against 6,484 |
  | every vectors request answered invalid | 4,746,584 | 1,719,573 | 57,516 (0) | 0 | 3,582 | **0** | **0** | differs (153 lines): no required offset or device step is ever asked, no weapon is born |
  | **own model** (below) | 4,746,061 | 1,719,573 | 54,441 (8,102) | 7,554 | 0 | 5 | 1 | differs (144 lines) |
  | the **original** build (`0ee1fa46b`, a private copy with the provider script added), quarter-rate host | 0 | 0 | 0 | 0 | 0 | 185 | 6 | **equal**: 0 differing lines (the negative control: a build that computes aiming itself never consults the host) |
  | the original build, every-vectors-invalid host | 0 | 0 | 0 | 0 | 0 | 185 | 6 | equal |

  The campaign's weapons, rejections and losses follow the physical facts the host reports, and an unmoved build is unmoved by the same host. The **own model** is a valid host that does not recompute EECH's: instant
  readiness (closed 0 or open 2, no animation), devices that snap to the required offsets within their limits, a plain bearing for the required offsets, a guided weapon intercepting the target's point, no solution
  for an unguided weapon, no ballistic table and no draw (`need_draw` 0). The fixture recorded its answers as ordinary valid responses (7,554 steps, 0 opening or closing phases, no draw), with no EECH kernel detail
  imposed on it. The reference counts are the crossing volume: about 6.5 M `aim` requests per 300 s of Lebanon, almost all of them the keyframe.
- **Missing and malformed hosts** (shipping binaries `d3c9a5c5e`, Lebanon, up to 300 s; `providers/ship-pv-*`): no provider fails at boot (`eech: missing host aim callback`); a nil answer, an unknown state, a ready state
  of 5, a phase of 7, a non-finite weapon vector, a second request for a draw, a negative range and a Lua error each stop the engine at the first such request with `EECH fatal error: host aim: ...` and the host's
  reason (the driver's controlled `pcall` then exits 0). **One mode did not trigger**: `bad_draw_for_ballistic` waits for a standalone ballistic request, and none occurred in 300 s (the natural witnesses make no
  standalone `ballistic`, `lead` or `intercept` request at all; see the limits), so the driver's own assertion that a malformed answer must fail stopped it (native exit 1, the mode's expected failure to fire, recorded as
  such). That refusal is covered by the engine unit test instead (`an_aim_draw_is_asked_for_only_by_a_vectors_or_lead_request...`). The shipping reference provider run (120 s) completes with failure none.
- **Source and symbols** (`checks/symbol-check-shipping-d3c9a5c5e.log`, `controls/symcheck-aim.sh`): `mb_hostaim.o` and `rv_hostmv.o` reference no maths function; the kernel `eech_aim_*` symbols (ready update and command,
  required, step, ballistic, lead, vectors, intercept, person face) are in `eech-world.exe` and absent from `eech_dc.dll`; `ballistic_pitch_deflection` is absent from the DLL (`get_ballistic_pitch_deflection` remains as
  the thin campaign-side call); the fixture symbols are in no shipping binary. The `weapon.o` references to `atan2` and `get_3d_transformation_matrix` that remain are the rocket-pod tube roll and the package-arc
  bearing (below). The DLL's `get_intercept_point` is `wn_move.c`'s different function for a guided weapon in flight (weapon execution).
- **Focused contracts** (all in the image, cargo's own status): `cargo fmt --all --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings` (default features, and `--features eech-dc/aim-fixture`) and
  `cargo test --locked -p eech-engine -p eech-dc -p eech-world` all `CARGO_EXIT=0` (`checks/verify-v4.log`). New: seven engine tests of the `aim` trampoline and the `person_face` mode (shape, ids, draw rules,
  refusals, echo room), seven kernel tests in `aim.rs` (keyframe open/hold/close/wrap, offsets and limits and the climb floor, the unbounded and tracked forms, the shortest-way step, the table shape, the point-blank
  guard, the tiny-range lead, the person facing), two movement-record tests of `person_face`, and `host_callbacks.rs::aim_round_trips_through_lua_and_refuses_what_it_was_not_asked`. Bit-for-bit equality is the
  complete trace's job, not these.

## Limits and costs

- **Not run by the implementation lane**: the lead's final clean ordinary build and full deterministic pack, the independent source review of this implementation, or live DCS (M6). The runs above are the lane's own.
- **Cost.** About 12.9 M extra host crossings per 600 s of Lebanon (almost all of them the keyframe: 9,429,480 updates and 3,387,517 close commands). The 600 s fixture runs took 808 to 914 s against the original 677 s (+19% for the
  808 to 809 s runs; the 914 s run overlapped builds), and the ordinary 900 s run 1,260 s against 1,080 s for the baseline (+17%); other work was running on the machine, so these are indications, not a benchmark. The
  reference host reads a request's scalars in one Lua call; batching would change the update order and is not done.
- **Not reached by the natural witnesses, so covered only by the focused contracts and the unmoved arithmetic**: the standalone `lead` (decoy and fuze timer at launch), `ballistic` (AI salvo, the pipper) and `intercept`
  (the HUD) requests (0 in 300 s of Lebanon); the lead prediction's tiny-range, off-map and stationary paths; the fixed-pitch ballistic mode; the point-blank guard; the unbounded HUD offsets and the player's cannon
  tracking; the person branch outside the controlled case; any other seed, duration or scenario than Lebanon (Georgia was not rerun). **One accepted difference, unreachable in the witnesses**: an off-map *success* of the
  lead prediction left the caller's time of flight unwritten (an uninitialised local at weapon launch); the host answers 0 and the campaign stores it.
- **Known coupling, unchanged and declared**: the device viewpoint (`get_3d_sub_object_world_viewpoint`, the 3-D model and subobject lookup) is still read by the campaign; the target point
  (`get_local_entity_target_point`); the ballistic tables are generated by the still-core projectile simulation (`generate_ballistics_tables`, `move_unguided_weapon`: weapon execution) and handed over once; the shared
  `get_angle_of_projection*` maths helper (`modules/maths/projtile.c`) is used by the player's HUD sight in `co_wpn.c` and stays; the rocket-pod launch-tube roll and the package-arc bearing in `weapon.c`
  (weapon execution and package choice); `wn_move.c`'s guided-weapon intercept in flight; the triangulated range of the player's gunship, which is refused; the fire decision, whether to make a system ready, the cockpit
  and head-tracking inputs, and the shared random stream stay in EECH_DC by design.
- **Weapon execution and combat resolution/reports remain D2** after this Task; nothing here claims D2 complete.
