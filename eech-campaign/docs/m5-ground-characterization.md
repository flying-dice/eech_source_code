# D3 G2: ground contact, original characterisation (EEDCS-48)

**Status: source written, NOT built, NOT compiled, NOT run. One independent source review of `7a4c12ca0` found two P2 and one P3 (no change to any original formula); this revision corrects them (section 7) and awaits Root validation and a review of that delta.** Task EEDCS-48 (parent EEDCS-21, D3), branch `codex/d3-ground-contact`, from accepted integration
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
| `crates/eech-world/tests/host_callbacks.rs`, `tests/lua/ground-caller.lua` | `the_ground_driver_*` tests: the real driver against a stand-in engine (loaded with `include_str!`), for argument refusal, flow, and a table of 39 single-defect reports that must each be refused by name |

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
   asked cold (record = sentinel) and once more warm, and the two answers compared. The warm call is the very next query after the cold one: the record is copied the instant the cold call returns,
   the (read-only) observation runs, and the warm call is made only after the record is checked to be that copy (`warm_start_intact`, which must equal the case count). The input vector is checked for mutation.
4. **Impact cases** (`ground-impact*`): eight types: 0 a valid crossing, 1 above endpoint actually below ground (first correction), 2 below endpoint actually above ground (second correction), 3 both,
   4 identical `x, z` (bisection on `y` only), 5 both exactly at the terrain elevation, 6 above endpoint exactly at its sector maximum, 7 endpoints straddling a sector boundary. Horizontal separations
   of 0.25, 0.9, 1, 1.1, 2, 10, 100, 1 000 and 10 000 m bracket the 1 m (squared 1.0) stop. Each case runs **three** calls: aliased as the caller does (`&p, &q, &q`), separate output, and the separate call
   again warm. Observed: class, the bits of every vector it can write, the record after each cold call, whether aliased and separate agree (`alias_equal`), whether warm agrees (`warm_same`), which inputs were
   corrected (`a_corrected`, `b_corrected`, and `xz_changed`, expected 0 and to be measured), and whether the impact equals the corrected below endpoint. The sequence is: aliased call, record re-armed,
   separate call, warm call, and **only then** the two extra diagnostic `point_above_ground` checks of the corrected endpoints (`ground-asserts`). Those checks are queries the original code does not make
   there, and `point_above_ground` can write the shared record, so they run after every call being characterised and cannot change what those calls saw. The original function's own `ASSERT` evaluations
   are untouched and happen inside it, in its own order. (The first committed form ran the diagnostics before the warm call and so measured a warm repeat on a different record; corrected after independent review.)
5. **Hashes and samples**: two order-dependent FNV-1a folds, reported on separate rows and never mixed (see "What can be compared" below): `ground-hash`, the legacy fold, and `ground-physical`, the physical-only
   fold. `ground-sample` lines carry **float bit patterns in hex only** (no printf of floats under round-toward-zero): two point samples per tag, four impact samples per type. The report size is independent of
   the counts and fits its 192 KiB buffer; a report that does not fit is refused, never cut short.

### What can be compared

The fixture was written against the accepted integration, where the shared terrain record is the original struct (`terrdata.h`) with sector and triangle indices and sector/face pointers. The G1 lane
(Task 47) replaces that record with semantic values and optional hints, and the record's layout is **never** a generic or DCS host requirement. So the report keeps three kinds of fact apart, and a later
candidate may be compared only as far as each kind allows:

| Kind | Where it is reported | What it is | Comparing a later candidate |
|---|---|---|---|
| **Physical** | `ground-physical` (`combined`, `above`, `below`, `impact`); the counters `ground-points`, `ground-above` and `-below` (`cases`, `true`, `false`), `ground-complement`, `ground-impact` (`a_corrected`, `b_corrected`, `xz_changed`, `alias_equal`, `alias_unequal`, `impact_equals_corrected_below`), `-classes`, `-types`; the `x y z`, `a0 b0 pa qa pb qb rb`, `class_*`, `above`, `below`, `inside` fields of `ground-sample` | what each query was given and answered: input vectors as given and after the call (so an input mutation shows), the booleans, the terrain class, the endpoint corrections and the impact point, for the aliased and the separate output shapes. `ground-physical` folds exactly these, from the cold calls only | the whole-query comparison that needs no private index or pointer. A difference is a regression, an approved defect correction, an intentional semantic change or expected input variation, and is classified, never normalised |
| **Short-circuit and cache** | `record_touched`, `true_record_untouched`, `off_map_record_touched`, `record_touched/record_untouched` (impact); `warm_same`, `warm_different`, `warm_start_intact`; `above_touched`, `below_touched` and `touched` in samples | whether a call read the terrain at all (the sector-maximum clause, the off-map clause) and whether a repeat on the state the first call left agrees. Counters only; **not** in `ground-physical` | a candidate with no shared record has no "record touched"; it is compared through its own declared hint/cache facts (G1's semantic cache comparison) where they exist, and a short-circuit that the candidate does not reproduce is reported as a difference, not dropped |
| **Legacy record diagnostics** | `ground-hash` (all four hashes), `ground-record` (`words_before`, `words_after`), and the `elev`/`type` words of `ground-sample` (`above_elev`, `above_type`, `below_elev`, `below_type`, `elev`, `type_word`) | the original shared record's value words and its private bookkeeping: x and z sector indices, the triangle offset and whether a sector and a face are held, folded by `fold_view`. They exist only because the original record exists | not interchangeable physical answers and not a provider requirement. A `ground-hash` difference against a candidate with a different record is expected and is **classified, never normalised into physical equality** and never "fixed" by forcing indices or pointers into a host contract |

`ground-hash` is unchanged by the physical fold (the legacy folds are the same expressions over the same words, and the physical fold is a second, independent accumulator). The legacy hashes may differ from
the first committed form of the fixture, because the warm repeat now starts from the correct record (above); no oracle was ever recorded from that form.

**Comparison scope after G1/G2 composition.** The original oracle stays as recorded, with its legacy diagnostics, as the characterisation of the unchanged code. Composing this fixture with the G1 branch
needs a **deliberate fixture adaptation**: the record access (`shared_record`, `view_record`, `fold_view`, `record_words_hash`, the sentinel) reads the legacy struct and must be replaced by whatever G1
exposes (its semantic value record), and that adaptation is itself reviewed. After it, the comparison is: `ground-physical` and the physical counters and samples must match the original oracle under the
declared classification; the short-circuit and cache facts are compared as G1's semantic cache comparison permits; the legacy rows are reported for the original side only. This document does not define a host
API for any of it. The physical fold folds the same cold-call inputs and outputs in the same order, so a candidate that reproduces the answers reproduces the digest without exposing a single private index.

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
before the output file is opened); the probe is refused; the report is not a complete result (below); the record is not restored; or a repeat differs from the first.

**A report is accepted only as a whole.** Every row of the fixture's schema (`ground-probe`, `-map`, `-record`, `-points`, `-above`, `-below`, `-complement`, `-impact`, `-impact-classes`, `-impact-types`,
`-asserts`, `-hash`, `-physical`) must be present exactly once with exactly its keys: a missing key, an unexpected key, a repeated key, a word that is not `key=value`, an unknown row or a repeated row is
refused by name. Every value must have its exact type: a decimal count (digits only, so not `0x5`, `-1`, `1.5` or `1e3`), a 0/1 flag, or lowercase hex of the exact width (16 for record words and folds, 8 per
float word in a sample, three comma-joined for a vector). The counts must agree with the request and with each other (cases equal the points or impacts asked, `true + false` equals the cases, the class and
type tallies sum to the impact cases, `warm_start_intact` equals the cases, a repeat's `restored` is 1 and its `words_before` equals `words_after`), a family that ran no case folded nothing (its hash is the
empty-fold value) and a family that did is not empty, and the samples are complete as the C emitter takes them (next paragraph). A zero-case family is a valid report. A report that fails any of
this records `report_invalid` with the reason, fails the run, and **no `repeat_identical` or `state_digest` row follows it**. The reason is the driver's own message for that one defect; the tests compare it
exactly, not by a shared word. (The first committed driver only checked that a few rows existed, so a report with no record words, or a bare `ground-hash`, could pass, because `nil == nil`.)
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

## 7. Corrections after the independent source review of `7a4c12ca0` (unbuilt, unrun)

Review: `R:/d3-evidence/independent-ground-original-source-review-7a4c12ca0.json` (SHA-256 `1e6f5d4b2753b123db2bec6152a04aaac4a154110b4a59cdb6967d8ab245edf4`). No original production expression, shipping flag, data, root,
oracle, baseline or tolerance is touched by any of the three.

| Finding | Correction |
|---|---|
| P2 warm impact state: diagnostic `point_above_ground` checks ran between the separate impact call and the warm call and could overwrite the shared record | the two diagnostic checks now run **after** the warm call; the record is copied when the separate call returns and checked to be that copy immediately before the warm call (`warm_start_intact`, per family, required to equal the case count). The aliased sequence is unaffected (the record is re-armed to the sentinel before the separate call, which is the cold reference by design) and the point cases already repeat immediately; the anchor check at probe start writes then restores its own snapshot. No query was added to the behaviour being characterised |
| P2 driver truthfulness: absent `words_before`/`words_after` compared `nil == nil`; a bare `ground-hash` passed; row names alone satisfied the family checks | the strict whole-report schema of section 3, with 39 single-defect negative controls through the real driver (absent, short and upper-case record words; bare, short and missing hash and physical rows; missing, hex, negative, fractional and inconsistent counts; duplicate, unknown and malformed rows; a fold that is empty for a family that ran or full for one that did not; bad, missing and out-of-range samples) plus the refused probe and a repeat that differs. Each is compared with its own exact message |
| P3 legacy record comparability | section 2, "What can be compared": physical, short-circuit/cache and legacy-record facts are separated; a new `ground-physical` fold (inputs as given and after the call, answers, classes, endpoint corrections, the impact point in both output shapes; no record bookkeeping, no short-circuit or cache facts) is reported next to the unchanged legacy `ground-hash`; the driver requires both; a deliberate, reviewed fixture adaptation is required when composing with G1 |

Remaining limits of this revision: the physical fold is only as meaningful as the cases that run (it is a digest of what was probed, not of the whole function); the fixture still reads the legacy record
directly and so still cannot be built against the G1 record without the adaptation above; `warm_start_intact` is an invariant check on the fixture's own sequence, not evidence about the original code; none of
the C, Rust or Lua has been compiled or run.

### Sample completeness (second review of the driver; unbuilt, unrun)

The first corrected checker accepted any non-empty sample set under a global cap, so an impact tally of five cases of one type with a single sample was accepted although the emitter takes four, and duplicate
case numbers or out-of-domain tags and types passed. The driver now checks what follows from **how `eech_ground_fixture.c` emits samples**, and nothing it would have to re-derive from the private generator:

- cases are visited in order and a case is sampled while fewer than `samples_per_point_tag` (of its tag) or `samples_per_impact_type` (of its type) earlier cases were, so within a family the sampled case
  numbers are **distinct, strictly increasing and below the case count**, case 0 is **always** the first sample (nothing precedes it), and all point samples precede all impact samples;
- a point tag is 0 to 5 and an impact type 0 to 7 (the impact-type tally has exactly eight members), and a tag or type has **at most** its cap of samples;
- because the impact type tally (`ground-impact-types`) is reported, each type has **exactly `min(tally, samples_per_impact_type)`** samples. The per-tag tally of point cases is not reported, so a point tag is
  bounded only from above; the driver does not guess the generator's tags;
- the sample values are the values the counters count: point samples inside the map area are at most `in_map` and those outside at most `off_map`; a sample outside the map area has `above=-1` (it was not asked) and
  one inside has `above` 0 or 1; samples with `above` 1 or 0, `below` 1 or 0, `alias_equal` 1 or 0 and `warm_same` 1 or 0 are at most the matching counter (`true`, `false`, `alias_equal`, `alias_unequal`,
  `warm_same`, `warm_different`). Distinct case numbers make each sampled case a case that was counted once;
- the header's quotas are the emitter's compile-time constants, `POINT_SAMPLES_PER_TAG` 2 and `IMPACT_SAMPLES_PER_TYPE` 4 in `eech_ground_fixture.c`, written into every report's header. The driver accepts
  **only those values** (a header naming another is refused as `samples_per_point_tag is N, expected 2`), because every completeness check below assumes the quota and a larger one would excuse cases the emitter takes;
- **no case may be skipped before a class is full** (see the next subsection).

Tests (second review): 22 more single-defect reports through the real driver (61 in all at that revision, each compared with its exact message; the zero-quota case now lives in the forced-gap test below, so the table holds 60), and positive runs with the caps binding (40 cases each: 12 point and 32 impact samples), not binding (5 each:
every case sampled) and with one type holding all five impact cases (four samples). The stand-in writes a self-consistent complete sample set; it is not the C generator, and these tests are scoped to what the driver
accepts and refuses, not to the ground queries. No C, numeric, query or digest change is made.

### Forced samples before any class is full (third review of the driver; unbuilt, unrun)

The previous completeness checks still accepted `points=5` with the samples for cases 0, 2, 3 and 4 (the case-1 sample dropped), with case 4 dropped, or five impact cases of one type with samples for cases 0, 2, 3 and 4
(the exact count of four, the order, the domain and the caps all hold). The emitter samples a case **unless its own tag (or type) already holds its quota**, so while no tag has two samples (no type four) there is no
class a skipped case could belong to: every consecutive case must be sampled. The driver now applies this to **both** families, after the exact impact-type counts:

- walking a family's samples in case order, it counts samples per tag (type); the family is **full** from the first moment any one class reaches the quota (2 for a point tag, 4 for an impact type);
- a gap between consecutive sample numbers, or a tail that stops before the last case, is refused **while not full**: `point case 1 is not sampled, but no tag has reached 2 samples (the gap is cases 1 to 1)` and
  `point samples stop at case 3 of 5, but no tag has reached 2 samples (cases 4 to 4 are unsampled)` (and the `impact`/`type` forms);
- once a class is full, a gap or tail is accepted: the skipped cases may belong to it. The driver does not guess which; the point cases' per-tag population is not reported and the generator is not reproduced here.
  A gap that falls **before** the first full class is refused even if a class fills afterwards.

Tests (named in `host_callbacks.rs`): `the_ground_driver_refuses_a_skipped_sample_case_before_any_class_is_full_and_a_quota_the_emitter_does_not_take` (interior gaps at cases 1 and 2, an unfinished tail, a gap not excused by
a later saturation, the concentrated-impact gap, and the three header quotas 0, 3 and 8 of the point/impact kind) and
`the_ground_driver_accepts_the_omissions_the_emitter_makes_once_a_class_is_full_and_complete_small_runs` (zero and one case of a family, small complete runs, five points of one tag that stop after the tag's two
samples, and a saturated tag followed by other tags), beside the earlier positives (caps binding at 40 cases each, 5 each, and the concentrated impact complete at four samples). The stand-in is a driver-truthfulness
collaborator, not the C generator, and these tests say nothing about the ground queries.

**Validation status.** None of this is executed. The Lua parse, the `host_callbacks` tests with the real driver, the fixture build and the original retail probes are Root's to run; the earlier immutable
06 native build result, if it completes, is a separate fact and does not validate this driver change. No runtime PASS is claimed here. The C fixture, its numerics, queries and digests are unchanged by this revision.

**Test-assertion correction (Linux run of `06ce4b7bc`: 6 passed, 1 failed).** The driver records every repeat's report, so the sample-cardinality positives that asserted 12/32, 5/5 and 4 sample rows from a default two-repeat run
saw 24/64 and so on. That was a test defect, not a driver or emitter one: those positives now request `repeats=1`, and the caps-binding case also runs with the default two repeats and checks the doubled totals and the
12/32 counts per `repeat_index`, so separate repeat behaviour stays covered. The forced-gap positives count the same way. Still not re-run after this edit.

### Executed checker checkpoint — 29 October 2026

Root executed the corrected working-tree driver and embedded stand-in through the
real Linux host and Lua 5.1: all nine ground-driver tests pass, including the
single-defect controls, small/zero families, legitimate saturated omissions,
exact sample cardinality and repeat separation. The affected host_callbacks
Clippy target passes with warnings denied; formatting/check and both Lua 5.1
syntax parses pass. The execution record retains the exact tested file hashes;
this is driver validation, not original C query or shipping acceptance.

A retained red/green replay uses the exact same corrupted old collaborator,
missing point case1: the immutable06 driver wrongly exits0 and emits a digest;
the corrected driver exits1 with the named missing-case error, one report_invalid
and no digest. Its before/after driver hashes match. Separately, the first actual06
suite had six passes and one test-assertion failure: flat sample totals ignored
the default second repeat. Single-report assertions now request repeats1; an
explicit default-two-repeat case also checks each repeat's12/32 samples. No
emitter, query or expected physical answer was changed to correct that assertion.

The immutable06 Windows original fixture built0 and all43 exported manifest entries
verify. Its separate Windows guard executable compiled0 but has not run. A clean
snapshot of this corrected source must be built before the original retail probe,
repeat and probe/control gates. No original oracle exists yet; final independent
source-delta review and all host-extraction/shipping delivery gates remain pending.
