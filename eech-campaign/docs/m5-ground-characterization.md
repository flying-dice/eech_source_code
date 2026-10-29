# D3 G2: ground contact, original characterisation (EEDCS-48)

**Status: source written, NOT built, NOT compiled, NOT run, NOT reviewed.** Task EEDCS-48 (parent EEDCS-21, D3), branch `codex/d3-ground-contact`, from accepted integration
`6bbaa8197a4843181d03cd355d562af998c75321`. This is the **original characterisation only**: a test-only fixture and a driver that exercise the unchanged ground-contact code through the public
Lua -> `require ("eech_dc")` -> `Engine` path. New engine hooks are test-feature guarded; ordinary shipping flags, behavior and physical expressions remain unchanged. Nothing here moves behavior to a host. Every result named below as
"to be measured" is unmeasured; no golden, hash, count or coverage figure exists yet, and none may be quoted from this document.

Scope authority is the live M5 goal (Confluence 11534337), the migration ledger (14548994) and Epic EEDCS-21: physical terrain and spatial queries move to the host; geography, routes, timers and
scheduling stay in the campaign. The finite D3 audit is research, not authority. The group-1 point/fast/map surface (Task 47) is a separate, undelivered lane and is not used here; G2 extraction and
production cutover wait for Task 47's delivery and an integration re-verification.

## 1. The behaviour, as read from source (not yet measured)

Everything in this section is what the source says; the probe exists to turn it into measured, order-dependent facts.

**`point_above_ground (POS)`** (`aphavoc/source/entity/system/en_main/en_world.h:201`):
`(y >= get_terrain_3d_point_sector_max_elevation (x, z)) || (y >= get_3d_terrain_elevation (x, z))`.
- The sector maximum is read first. When `y` reaches it, the second clause is **not evaluated**: the shared terrain point record is not written.
- There is **no map-area guard**. The sector-maximum function stops the engine (`debug_fatal "Sector maximum terrain elevation off map"`, `modules/3d/terrain/terrdata.c:1642-1654`) for `x` or `z`
  outside the terrain's own extents. Off-map input is therefore an unsupported precondition, never probed.
- `get_3d_terrain_elevation` is `get_3d_terrain_point_data (x, z, NULL)` (`terrelev.h:110`), whose `NULL` record is the file-static `system_point_data` (`terrelev.c:99`). That one record is the shared
  state every `NULL` terrain query writes and every `NULL` accessor (`get_3d_terrain_point_data_type (NULL)` and the others) reads.

**`point_below_ground (POS)`** (`en_world.h:207`):
`(!point_inside_map_area) || ((y < sector_max (x, z)) && (y < elevation (x, z)))`, the inner `&&` binding tighter than the `||`.
- Off the map area it is true before any terrain read (record untouched). Inside, `y >= sector max` is false at the first clause and the elevation is not read.
- It is **written separately** from `point_above_ground`; it is not defined as its negation. Whether the two are complements, and whether they read the shared record in the same cases, is **to be
  measured** (`ground-complement`, and `record_touched` for each). Do not infer either.

**`get_ground_impact_point (above, below, impact)`** (`aphavoc/source/entity/en_misc/en_maths/en_cllsn.c:107-171`), in order:
1. `if (!point_above_ground (above))`: assert in map area, then `above->y = get_3d_terrain_elevation (above->x, above->z) + 0.001` (a `double` constant, stored to `float`).
2. `if (point_above_ground (below))`: assert in map area, then `below->y = get_3d_terrain_elevation (below->x, below->z) - 0.001`. Both corrections edit the caller's inputs in place and touch only `y`.
3. Asserts: `above` and `below` in map area, `!point_above_ground (below)`. **`ASSERT` here is `write_debug_log`** (`modules/system/assert.h`: the fatal `process_assert` needs `DEBUG` or `DEBUG_ASSERT`;
   I found neither defined in the build's `-D` set or the headers searched, only an unrelated `#define DEBUG 0` in `malloc.c.h`: confirm in the compiled objects). It is a no-op unless
   `command_line_debug_log` is set, so a violated assert does not stop the engine in this build; the probe therefore re-asks the one assert condition that can fail after correction
   (`ground-asserts original_below_endpoint_still_above`) rather than trusting it.
4. Bisection: `length = 100.0` is set **before** the loop, so the loop body **always runs at least once**, whatever the endpoints. Each pass: midpoint by `(a + b) * 0.5` per axis; `point_above_ground (c)`
   moves `a` to `c`, otherwise `b`; then `length = get_sqr_3d_range (a, b)`. It stops when `length <= 1.0` (squared range, so endpoints at most 1 m apart; the compare is float against a double constant).
5. `b.y = get_3d_terrain_point_data (b.x, b.z, NULL)` (the terrain elevation at the final below endpoint's `x, z`, which also **writes the shared record**), `*impact = b`, and the return is
   `get_terrain_type_class (get_3d_terrain_point_data_type (NULL))`: **the class is read back from the shared record the line above just wrote**. The class depends on that record, not on a value
   returned by the elevation call.

**The caller** (`aphavoc/source/entity/mobile/weapon/wn_move.c:411-437`, the hosted weapon lookups; the only caller of `get_ground_impact_point` in the tree):
- `CAMPAIGN_STEP_LOOKUP_GROUND_PROBE`: `r[0] = point_below_ground (&p) ? 1.0 : 0.0`.
- `CAMPAIGN_STEP_LOOKUP_GROUND_IMPACT`: `terrain_class = get_ground_impact_point (&p, &q, &q)` then `r[0] = (float) terrain_class`, `r[1..3] = q`. The impact output is **aliased with the below endpoint**. The function copies
  `*below` to a local before the loop and writes `*impact` last, so on return `q` is the impact point and the corrected below endpoint is lost to the caller; `p` (the corrected above endpoint) is
  a caller-local copy that is never returned. The corrections matter only through the bisection start and through the shared record.
- The campaign-side weapon flight (`eech-world/csrc/wn_flight.c`) bounds the new position into the map volume first (`:2687`, `:2709`) and asks `WL_GROUND_PROBE` only for points inside the map area, so
  the off-map clause of `point_below_ground` is not reachable by this path. `WL_GROUND_IMPACT` is asked with the old and new positions; an endpoint outside the map area would reach the
  fatal sector-maximum read, which the bounding appears to prevent (read from source, not exercised).
- Other users of the macros (read only, not characterised here): `pa_updt.c:210` (`point_above_ground`), and `co_eo.c`, `co_hms.c`, `co_radar.c` (`point_below_ground`).

## 2. The fixture

Compiled only with `--features eech-dc/ground-fixture` (`EECH_GROUND_FIXTURE`); the shipping module has no `ground_probe` method, and no shipping flag, source expression or default changes.

| File | Role |
|---|---|
| `crates/eech-engine-sys/csrc/eech_ground_fixture.c` | the probe: case generation, the three original calls, the shared-record observation, the report |
| `crates/eech-engine-sys/csrc/eech_engine.[ch]` | one entry, `eech_engine_ground_probe (seed, points, impacts, text, capacity)`, inside the engine's `ENTER`/`LEAVE` (round-toward-zero as the campaign sees it); needs no host |
| `crates/eech-engine-sys/{Cargo.toml,build/main.rs,src/lib.rs}`, `crates/eech-engine`, `crates/eech-dc` | the feature, its `-D`, the binding, `Engine::ground_probe`, the Lua method `engine:ground_probe (seed, points, impacts)` |
| `crates/eech-world/lua/ground-characterization.lua` | the one driver |
| `tools/build-windows-ground-fixture.sh` | the unchanged `build-windows.sh` with the one cargo argument added (the same two-line adaptation check as the LOS wrapper) |
| `crates/eech-world/tests/host_callbacks.rs` | `the_ground_driver_*` tests: the real driver against a stand-in engine, for argument refusal and flow only |

### What one probe does

A private xorshift32 generator (never the game's random stream; no trigonometry) places `points` point cases and `impacts` impact cases on the **map the campaign's own boot loaded**.
Terrain heights used to place points come from a private point-data record. Every case then calls the original macros or function, and:

1. **Shared record**: before each call the record is set to a known sentinel (a full `memcpy`); afterwards it is compared with the sentinel byte for byte. Unchanged means the call never queried the
   terrain; changed means it did, and the record's value words (elevation bits, terrain type, face-normal bits, sector and face indices, whether a sector and a face are held) are folded and sampled.
   Pointers are tested for presence only and never emitted. The record is located through the public accessor `get_3d_terrain_point_data_normal (NULL)` less `offsetof (face_normal)`, and the address is
   proven by writing two distinct words through it and reading them back through `get_3d_terrain_point_data_elevation (NULL)` and `..._type (NULL)`; otherwise the probe is refused.
2. **Restoration**: the record is snapshotted before the first case and written back byte for byte after the last. Restoration is by construction; the independent evidence is that repeat 2's
   `words_before` (and so its whole report) equals repeat 1's, and the probe/control digest below. If the engine stops mid-probe (`debug_fatal`) the record is not restored; the engine is then failed anyway.
3. **Point cases** (`ground-above`, `ground-below`, `ground-complement`): tags 0 and 1 interior points, 2 sector boundaries (`k * 2048`, one float below, one above), 3 and 4 map edges (`0`, `-0.0`, the
   next float above 0, `1`, the maximum, one float below it, `maximum - 1`, `2048`), 5 **outside the map area**. The height is one of sixteen offsets from the terrain elevation `e` and the sector maximum `M`:
   `e-500, e-1, e-0.001, nextbelow(e), e, nextabove(e), e+0.001, e+1, e+500, nextbelow(M), M, nextabove(M), M-1, M+1, MIN_MAP_Y, MAX_MAP_Y`.
   `point_above_ground` is asked **only** when `point_inside_map_area` holds (it stops the engine otherwise); `point_below_ground` is asked for every point, including outside the map area. Each is
   asked cold (record = sentinel) and once more warm, and the two answers compared. The input vector is checked for mutation.
4. **Impact cases** (`ground-impact*`): eight types: 0 a valid crossing, 1 above endpoint actually below ground (first correction), 2 below endpoint actually above ground (second correction), 3 both,
   4 identical `x, z` (bisection on `y` only), 5 both exactly at the terrain elevation, 6 above endpoint exactly at its sector maximum, 7 endpoints straddling a sector boundary. Horizontal separations
   of 0.25, 0.9, 1, 1.1, 2, 10, 100, 1 000 and 10 000 m bracket the 1 m (squared 1.0) stop. Each case runs **three** calls: aliased as the caller does (`&p, &q, &q`), separate output, and the separate call
   again warm. Observed: class, the bits of every vector it can write, the record after each cold call, whether aliased and separate agree (`alias_equal`), whether warm agrees (`warm_same`), which inputs were
   corrected (`a_corrected`, `b_corrected`, and `xz_changed`, expected 0 and to be measured), and whether the impact equals the corrected below endpoint.
5. **Hashes and samples**: an order-dependent FNV-1a over every case's inputs and outputs (`combined`, and one per call family). `ground-sample` lines carry **float bit patterns in hex only** (no
   printf of floats under round-toward-zero): two point samples per tag, four impact samples per type. The report size is independent of the counts and fits its 192 KiB buffer; a report that does not fit is
   refused, never cut short.

### Preconditions, refusals and limits

A probe that does not hold these writes `ground-refused reason=...` and fails; nothing is altered first. The driver records `probe_refused` and fails the run (no digest, no `repeat_identical`).

| Reason | Meaning |
|---|---|
| `fpu_not_round_toward_zero` | not inside an engine entry |
| `no_terrain_loaded` | no terrain or no sectors |
| `world_map_is_not_the_terrain_extents` | the macros read the world map and the terrain functions read the terrain's: they must agree on sector side, counts and the four extents (bit-exact) |
| `empty_map` | |
| `shared_record_not_reachable_through_accessors` | the anchor check failed |
| `report_does_not_fit_the_capacity` | |

Engine entry: `points` and `impacts` each 0 to 100 000, not both 0; seed 0 is accepted by the engine (treated as 1) but **refused by the driver** so that it cannot alias seed 1. Unsupported and never
probed: any off-map input to `point_above_ground` or `get_ground_impact_point` (fatal), NaN or infinite coordinates, the Cuba terrain-flaw path (`map_number == 2`, not a retail map used here).
Not observed: the bisection iteration count (it would need instrumenting `en_cllsn.c`; it is visible only through the outputs); the `answer_weapon_step_lookup` wrapper itself (static, entered only by a
launched weapon's flight step) and its two glue lines (`bool -> 1.0/0.0`, `class -> float`). Terrain content is the real map's, so any rate (how often a point is above, a class is water) is a property of
the map and seed, not a requirement.

## 3. The driver

`ground-characterization.lua` boots with the reference host's callbacks exactly as the LOS driver does, optionally runs `seconds` of frames, calls the probe `repeats` times with no frames between, runs
`after` seconds of frames, and writes one JSON Lines event per report line plus a `state_digest`. It fails (not green) when: an argument is unknown, malformed or out of range (checked **before boot**,
before the output file is opened); the probe is refused; the report lacks a required line or is not for the seed and counts asked; the record is not restored; or a repeat differs from the first.
`probe=no` is the control: the same frames, no call, and the same digest line (object ids, kind, side and alive in id order, a modular hash below 2^31 because Lua 5.1 formats through a 32-bit `long`
on Windows). Hash words and samples are kept as text so a hex word of decimal digits is never turned into a number.

Arguments: `root`, `out` (required); `scenario` (`lebanon_retail` default, `georgia_retail`, `georgia`); `seed` (1 to 4294967295, default 20261029); `points` (default 2000); `impacts` (default 500);
`repeats` (1 to 5, default 2); `seconds`, `after` (0 to 3600, default 0); `frame_ms` (1 to 1000, default 100); `probe` (`yes`, `no`). All whole numbers.

## 4. Proposed commands (Root runs these; none has been run)

Build the test-only fixture binaries (Git Bash, Docker Desktop with Linux containers, from a clean checkout of the commit to characterise):

```
tools/build-windows-ground-fixture.sh target/ground-fixture/<name>
```

Cheap checks that need no fixture feature (the driver against the stand-in engine, formatting, lints), exactly as the other `host_callbacks` tests:

```
cargo test -p eech-world --test host_callbacks the_ground_driver
cargo fmt --check
cargo clippy -p eech-engine -p eech-dc --features eech-dc/ground-fixture -- -D warnings
```

One native run (the engine writes into its install root, so each run needs its own private copy of a verified retail root; the verified roots are under
`R:/eech-regression-20261025/roots/` (`lebanon`, `georgia`, `lebanon-repeat`). Copy a root per run, do not run against the originals; paths without spaces):

```
<bin>\eech-world.exe <bin>\lua\ground-characterization.lua root=<private root copy> scenario=lebanon_retail out=<dir>\ground.jsonl seed=20261029 points=2000 impacts=500 repeats=2
```

## 5. Follow-up original runs (all unrun; each answers one question)

| # | Run | Question it answers |
|---|---|---|
| R1 | `scenario=lebanon_retail`, defaults, freshly booted world | the first measured facts: short-circuit and record-touch pattern, complement counts, alias and warm agreement, assert conditions, classes seen |
| R2 | `scenario=georgia_retail`, defaults | the same on the other retail map (different extents, sector counts and classes) |
| R3 | R1 again in a second process on a fresh copy of `lebanon-repeat` | cross-process determinism: `ground.jsonl` byte identical to R1 (compare the whole files; no line carries a path, time or address) |
| R4 | `seconds=300 after=60` with `probe=yes`, then the identical `probe=no` control | the shared record holds the campaign's own last query at probe time (`words_before` differs from R1, informative, not a defect); and `state_digest` of the probe run equals the control's |
| R5 | `points=20000 impacts=5000` | counts large enough for the rare branches (equality, sector maximum, each correction, water and forest classes); report size unchanged |
| R6 | the **shipping** DLL (no fixture) with the same driver | the driver must fail with `probe_refused` and a missing-method error, never print a result |
| R7 | the three argument refusals by hand (`points=-1`, `seed=0`, `point=5`) | each fails before boot with a message naming the argument |

Whether the sample draw reaches every branch is itself a result to read from R1/R5, not to assume: a branch that no case reaches stays "unsupported or unmeasured" in the record.
Per AGENT.md the first result is a **characterisation of the unchanged code**, which may correctly pass before any move; there is no red/green here. A later G2 extraction reuses the same driver and seed
against the candidate and compares `ground-hash`, the counters and the samples; a difference is classified, never re-recorded (REVIEW.md).

## 6. Risks and open points

- **Nothing here has been compiled.** The C is gnu99 against the staged tree and uses only constructs already in the neighbouring fixtures; `-Werror=implicit-function-declaration` is on, so a missing
  header or declaration (`nextafterf`, `fegetround`, `get_ground_impact_point`) fails the build rather than the run.
- The record is located by layout: the public struct in `terrdata.h` and an accessor that returns the address of a member. The anchor check refuses a mismatch; it does not prove the accessor returns a
  member of the record `get_3d_terrain_point_data (x, z, NULL)` writes (it does not need to: `NULL` is `system_point_data` in every accessor of `terrelev.c`).
- `ASSERT` is assumed non-fatal (section 1). If a compiled object shows `process_assert`, an assert-violating case would stop the engine, not log; `ground-asserts` is meant to show none exist.
- Floating-point: the probe runs under the engine's round-toward-zero entry with the MinGW SSE build, not the MSVC retail binary; equality cases (`y == e`, `y == M`) are exact in `float` but any x87
  difference from the retail build is out of scope here.
- `edge_coord` and `boundary_coord` reach the exact map maximum, where the terrain sector index equals the last sector; this is intended (it is inside the map area) and is the case most likely to expose a
  surprise. If it stops the engine, the run fails with the engine's message and the case is recorded as unsupported, not skipped.
- The probe/no-probe digest is a coarse witness (object identity, kind, side, alive) after the same frames; it is not proof that the campaign's state is untouched. A fuller control is Root's call.
