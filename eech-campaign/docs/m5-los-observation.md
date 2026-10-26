# D2: host-owned line-of-sight observation (EEDCS-42)

**Status: implemented on PR101 (`codex/d2-perception-los`), corrected source `61e85664540770ab10e3ac7e9f5dfa916291c52f`. Not accepted.**
The independent source review and focused correction check are complete. The lead's clean final shipping build passed; the established
40-check candidate pack and supplied-binary destination baseline are running. Complete-file comparison, current CI and normal merge remain gates.
Scope authority is the live M5 goal (Confluence 11534337) and the
migration ledger (14548994): move physical combat and perception observation into the host and keep target selection, priorities and
engagement decisions in EECH_DC. Evidence: [`reference/d2-los-observation/`](../reference/d2-los-observation/README.md).

## What moved

The physical calculation of "does one point see another" left `check_position_line_of_sight` (`aphavoc/source/entity/mobile/mb_tgt.c`) for the
reference host, as a C kernel `crates/eech-world/csrc/los_step.c` in EECH's own arithmetic, expression for expression, like the other hosted steps:

- the direction, the horizontal range (x and z only) and the normalised **3D** direction, which together set the step increments;
- the coarse terrain loop (200 m), the source-end fine loop (40 m, counter capped at 8), the target-end fine loop (24 m, walking back from
  the target), the two object rays (each at most 200 m from its own end), the phase order and every early exit;
- the strict "farther than 12 m squared from the target" exclusion of a target-end hit, in three dimensions;
- the 1.5 m lift of an end that is a vehicle, which the entity form applied; the campaign now sends the fact (`source_eye`, `target_eye`, read from
  `INT_TYPE_IDENTIFY_VEHICLE`) and the host lifts, in the engine's own `y += 1.5` double expression.

The kernel is a resumable state machine (`eech_los_begin`, `eech_los_terrain`, `eech_los_objects`) driven by `crates/eech-world/src/los.rs`. It runs
under the round-toward-zero mode the engine entry sets, which a host callback inherits, as the other kernels do.

## The boundary

One new, required host callback, `line_of_sight`, in the same Lua -> `eech_dc` -> host path as the others (`dc.boot { line_of_sight = ... }`). It carries
a short synchronous dialogue with at most one query open (`aphavoc/source/entity/mobile/mb_hostlos.h`):

| Message | Direction | Contents |
|---|---|---|
| `begin` | campaign -> host | the two end points, `criteria` (the five original bits), `source_eye`, `target_eye`, `entity_form`, and the source and target **host identities** when the campaign roster has them (absent for a weapon, a position, an unbound end) |
| `need_terrain` | host -> campaign | the phase and the sample point (x, ray height there, z) |
| `terrain` | campaign -> host | the elevation the campaign's terrain service read at that point |
| `need_objects` | host -> campaign | the phase and the ray (`from`, `to`) |
| `objects` | campaign -> host | whether the collision scan found a blocker and, only then, its point |
| `done` | host -> campaign | `visible` |

A host that owns the terrain and the objects (a DCS host, later) answers `done` to `begin` and is never asked anything (`mode=blocked` and `mode=clear` of
`los-provider.lua` show that shape on the public path). No EECH pointer, entity index or list layout is in any message. Identities are the ones the roster
already holds (`gp_roster_link_member` binds every group member, which is why SITE members have them too); a weapon or unbound end is an explicit
absent identity, never refused. The reference host checks an identity's shape only and does not use it yet (collision is still the campaign's).

**What stays in the campaign** (declared transitional coupling): the terrain heights, read with the original `get_3d_terrain_point_data (x, z, &terrain_info)` and
`get_3d_terrain_point_data_elevation (&terrain_info)` (not the function's return value), with the same hint record kept for the whole query, and the
sample's `ASSERT (point_inside_map_area)` (D3); the object collision scan `get_line_of_sight_collision_entity`, with the source and target pointers kept
private to the one open query so that it excludes them (D5); the callers, which decide what to ask and act on the answer: target selection, weapon choice
and guide placement are untouched, and the aircraft and guide callers still build their own ray (lowered aircraft position, cover positions), which is
later D2 work, not policy moved here. The campaign never runs the solver: `mb_tgt.c`'s two public functions read the entity facts and call
`campaign_los_query` (`mb_hostlos.c`), which carries the dialogue and validates every host answer (state, finite fields, phase for the state, boolean
visibility, a bound on requests per query, no query inside another).

**No fallback.** A host build refuses a missing provider (`eech: missing host line_of_sight callback`; a C entry with no provider is a bad argument; a query
with none is a fatal). The original solver stays in the `#else` branch of `mb_tgt.c` for non-host builds of the legacy game and is not compiled into the DLL.

## Evidence

All results are from the runs recorded under `reference/d2-los-observation/` (private roots, none in Git; fresh copies, manifests IDENTICAL).

- **The original, frozen first** (`original-transcript/`): the unmoved solver with fixture-only hooks folded each terrain sample (phase, point, returned elevation),
  each object ray (phase, start, end, hit, collision point when hit) and a query-end marker into a lookup hash. The 40 s Lebanon run with probes at 30 s:
  4,567 queries, 107,162 lookups, run hash `9f8da359...`, lookup hash `30fc55f0733ea271`.
- **Same transcript after the move** (fixture candidate `becb9d5d...`, same hook definition taken where the campaign answers the host): the 40 s trace is **byte-identical**
  to the frozen file, outer hashes, lookup hashes, lookup count and the three probe sweeps included. The fixture records the entity form's effective (lifted) positions
  by the same expression, so no golden hash changed.
- **600 s, Lebanon and Georgia (`georgia_retail`)**: every compared field of the six probes and the run total is identical to the original characterisation
  (`d97f59bb364bb253`, 156,581 queries, 60,731 visible; `7fdc5ab0b8b5db5d`, 21,104 / 1,646). The 600 s runs of the *original* have no lookup transcript (it was added
  afterwards); the 40 s run is the lookup comparison. Candidate lookup counts: 3,032,664 (Lebanon), 234,681 (Georgia).
- **Whole-campaign short comparison** (not the pack): `campaign.lua scenario=lebanon_retail hours=0.25` with the supplied accepted-baseline binaries and with the shipping
  candidate: the Tacview recording (11,363,845 bytes), observations (6,629,273) and metrics are **byte-identical**.
- **Causal proof through the public path** (`los-provider.lua`, Lebanon, 300 s, seed 1; one run each, so a demonstration, not statistics): the provider decides, the DLL's policy acts on it.

  | Provider | queries | visible | requests | weapon births | losses |
  |---|---|---|---|---|---|
  | reference host | 71,668 | 20,516 | 1,298,696 | 185 | 6 |
  | host answers every query "blocked" at `begin` | 43,271 | 0 | 43,271 | 176 | 4 |
  | host answers every query "visible" at `begin` | 120,054 | 120,054 | 120,054 | 254 | 15 |
  | reference host, but every elevation the campaign reports is raised 10 km | 43,271 | 0 | 86,542 | 176 | 4 |

  The query volume, the engagements and the losses follow the host's visibility fact, so target selection and engagement go where the host says. The last row shows the
  campaign's environmental answers feed the host's solver. Blocking every query does not stop all weapons (176 births): launches that do not depend on a line-of-sight query
  (and the births counter's own limits) are not separated here.
- **Missing and malformed providers** (shipping candidate; `provider/failure-*.jsonl`): no provider fails at boot (`eech: missing host line_of_sight callback`); at the first query a nil answer,
  an unknown state, a non-boolean `done` visibility, a phase the state cannot have, a malformed point and a Lua error each stop the engine with `EECH fatal error: host line of sight: ...`.
  In these negative summaries, `frames=600` is the requested upper limit, not completed frames after the first-query failure.
  The baseline DLL has no such concept and boots without one, so this contract is new and red there by construction.
- **Source and symbols** (`extraction/checks/symbol-check.log`): in the shipping DLL's `mb_tgt.o` the only defined functions are the two public checks and the only
  undefined ones are `campaign_los_query`, the entity getters and the assert logger (no `sqrt`, no vector maths); `mb_hostlos.o` calls only the terrain, collision, roster and
  fatal services; `eech_los_*` kernel symbols are in `eech-world.exe` and absent from `eech_dc.dll`, as are the fixture symbols.
- **Focused contracts** (`crates/eech-world/src/los.rs`, 20 tests, plus `host_callbacks.rs::line_of_sight_round_trips_through_lua_and_refuses_what_it_was_not_asked`): derived from the original source as
  written, including its quirks: range 200 m gives no coarse sample and 400 m gives one; the fine counters make at most 7 queries; a sample level with the ray does not
  block; the exclusion at exactly 12 m is not a block and at 13 m it is, in three dimensions; the lift; a vertical ray and two coincident ends have range 0 (no terrain
  sample, zero-length object rays), characterised and not corrected; out-of-order, replaced, missing and malformed messages. Bit-for-bit equality is the transcript's job.
- Linux image: `cargo fmt --all --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings` (default features, and with `eech-dc/los-fixture`) and
  `cargo test --locked -p eech-engine -p eech-dc -p eech-world` (30, 3+2, 156, 18, 5, 2 tests) all exit 0, with cargo's own status.

## Limits and costs

- **Not run by the implementation lane**: the lead's final shipping build and established full pack, independent review, or live DCS operation (M6).
  The lead subsequently completed the clean corrected-head shipping build and independent source-review closure; the full pack remains running.
  The short comparisons above cover 900 s of one scenario and 600 s of two.
- **Cost**: the reference dialogue costs about 18 host crossings per query (1.3 M requests in 300 s of Lebanon). The 900 s whole-campaign comparison took 19 m 13 s against 18 m 06 s
  (+6%), run in parallel under the same load; that is an indication, not a benchmark. Batching would change the lookup order, so it is not done.
- **Known coupling, unchanged**: terrain (D3), collision and its entity-pointer exclusion (D5), the callers' own ray construction, and the host identities which the reference host does not use yet.
- **Unexplained, not investigated**: none new. The weapon-target (no identity) and SITE-target cases are carried as described above.
- The Lebanon A/B and the probe runs use the supplied default22 inputs; no tolerance, baseline or scenario was changed.
