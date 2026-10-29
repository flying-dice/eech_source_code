# D3 terrain and environment — source census and original-characterization plan

29 October 2026. Branch `codex/d3-terrain-services`, integration base
`6bbaa8197a4843181d03cd355d562af998c75321` (verified HEAD, clean at session start).
Authority: the live [M5 goal](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337)
(v3), the [migration ledger](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/14548994),
Epic [EEDCS-21](https://flying-dice.atlassian.net/browse/EEDCS-21) and Task
[EEDCS-47](https://flying-dice.atlassian.net/browse/EEDCS-47). Old W07/W08 wording, the
TypeScript experiments and earlier inventories are not authority.

**Status: source census and a prepared, not yet built or run, original-solver fixture.
Nothing has moved. No D3 native run exists.** Everything in sections 2 to 6 is *source
reading* (reported by inspection); only section 7 lists what is executed, and it is empty.
Accepted D2 evidence (`D2-SCOPE-AUDIT.md`, `reference/d2-combat-characterization/final-shipping-fullpack`,
baseline `R:/d2-evidence/baseline-host-20261026-shortpath`, bundle `R:/eech-regression-20261025`,
shipping source `64cf0f1c5`) is reused as the regression baseline and is not rerun.

## 1. Instruction-discovery record

Working directory `R:\d2-sensor-detections` (a worktree), model Sonnet 5.5, High effort. Read in
this session: `CLAUDE.md` (imports `AGENT.md`), `AGENT.md`, `PROJECT.md`, `docs/governance/REVIEW.md`,
`STATUS.md` (first 214 of 306 lines, the current D2 delivery entries), and live: M5 goal, ledger
(current body; its D3 row is only the epic link, so the scope text is the epic and Task descriptions), EEDCS-47, EEDCS-21.
No project memory file was found in the memory directory. Gaps: the ledger body exceeded the tool limit and was
read from the saved result, not paginated; Jira showed EEDCS-47 and EEDCS-21 as **To Do** at the time of
reading (the Root's notes say In Progress; the live state was not re-read later); `STATUS.md` lines 215 to 306 were
not read (older Task45 history); Root private notes (`R:/d3-evidence/D3-REQUIREMENTS.json`,
`D3-NEXT-ACTION.json`, `terrain-source-occurrences.json`, `dcs-api-reference/`) were read after the
live records and are used only as inputs, not as authority.

## 2. Finite D3 responsibility groups

Counts are textual occurrences of the terrain/spatial API names in files listed in
`eech-campaign/crates/eech-engine-sys/sources.txt` (the compiled set); they include declarations and
comments and are not runtime call counts. The same ripgrep pass over all compiled files finds
451 occurrences of 13 names in 99 compiled files; the Root's separate pass
(`R:/d3-evidence/terrain-source-occurrences.json`) counts `get_3d_terrain_point_data` 157,
`..._elevation` 67, `..._point_data_elevation` 140, `..._point_data_type` 49, `..._point_data_normal` 17,
`..._face_normal` 13, `get_terrain_type_class` 44.

| # | Group | Active source and call sites (read, not run) | Owner after D3 | Task |
|---|---|---|---|---|
| G1 | **Terrain surface at a point**: elevation, surface type, class and ground-vehicle suitability, face normal, plus the find-fast record that carries a hint between queries | `modules/3d/terrain/terrelev.c` (`get_3d_terrain_point_data` and the wrappers); type tables `aphavoc/source/misc/terr_db.c`. Movement: `hc_move.c` 33, `fw_move.c` 27, `rv_move.c` 17, `rv_hostmv.c` 4, `ps_move.c`, `aa_move.c`; targeting `mb_tgt.c` 6; `weapon.c` 3; `wn_move.c` lookups ELEVATION and FACE_NORMAL; LOS terrain phases `mb_hostlos.c` 2; `en_cllsn.c` 7; guides `ag_attck.c` 7, `gd_cover.c` 5, `aa_attck.c` 2; creation and update paths (`rv_creat`, `ps_creat`, `aa_creat`, `hc_creat`, `fw_creat`, `hc_updt` 6, `fw_updt`, `cg_updt`) | Host service. The type-to-class table and ground-vehicle suitability are campaign-owned data (`m5-host-boundary.md`) | **EEDCS-47** (first) |
| G2 | **Terrain-derived physical geometry**: ground probe (`point_below_ground`, `en_world.h`), ground impact point (`get_ground_impact_point`, `en_cllsn.c:107`), terrain-following look-ahead samples, crater/bridge/person terrain facts that D2 already asks of the campaign | `wn_move.c:399-470` (lookup kinds ELEVATION, FACE_NORMAL, GROUND_PROBE, GROUND_IMPACT); the helicopter and fixed-wing plan lookups in `hc_move.c`/`fw_move.c`; `mb_hostaim.c` impact/bridge `need_terrain`; LOS `need_terrain` | Host service (the answer), campaign keeps what it does with it | later D3 Task, after G1 |
| G3 | **Object/model collision and geometry**: first object or tree a segment meets, object bounds and windows | `get_line_of_sight_collision_entity` (`en_cllsn.c:1144`), `get_line_of_sight_collision_tree` (`:1747`); campaign-side callers `mb_tgt.c:455`, `mb_hostlos.c:275`, `wn_move.c` COLLISION and TREES lookups; D2 blast/bridge object windows | Host service; entity-pointer exclusion and object bounds data are coupling D2 declared | later D3 Task |
| G4 | **Weather and environment**: wind at a point, weather model position and mode | `session.c:595 get_session_wind_velocity_at_point`; the only campaign-reachable consumer found is the weapon step lookup `wn_move.c:399-411`. The session entity also *advances* wind gusting and weather position with `get_delta_time` (`ss_updt.c:484`, `:357-390`) and draws its random start values from the shared random stream (`set_session_random_weather_mode`, `session.c`, `frand1`/`rand16`) | Host service for wind; the RNG draws must be accounted for, see 5.4 | later D3 Task |
| G5 | **Host time inputs**: frame milliseconds, simulated clock, session elapsed time and time of day | `eech_engine.c:296` seeds the simulated clock from the run seed, `:332-334` advances it by the host's milliseconds and calls `set_delta_time` (which keeps a delta-time average and history); `ss_updt.c:79-119` applies elapsed time and time acceleration to time of day; `get_session_random_start_time_of_day` (`ss_creat.c:139`) draws the start | Host supplies the elapsed time; the campaign keeps update semantics, timers and scheduling | later D3 Task |
| G6 | **Campaign-owned consumers of terrain**: setup geography, route planning, task generation | `popread.c` 14, `faction.c` 9, `taskgen.c` 5, `croute.c` 2, `ai_route.c` 3, `keysite.c`, `wp_creat.c` | **Stay in the campaign.** Only the source of the terrain value changes, through G1 | no extraction; verified through G1 |
| G7 | **Not in the supported campaign-DLL runtime (to be proven)**: gunship flight dynamics (13 models, 188 occurrences in 14 files), cameras, cockpit/avionics/EO/radar, UI maps, drawing (`*_draw.c`, `3dshadow.c`, `3dstrike.c`), Tacview overlay | compiled, but the headless frame does not draw (`eech_engine.c:306-336`) and there is no Lua export for them | Disposition needs a compiled-symbol and dispatch audit like D2's cockpit-sight case; none done yet | a closing audit |
| G8 | **The terrain solver and loader themselves**: `terrelev.c`, `terrdata.c`, `terrmap.c`, `terrgeom.c`, `terrsel.c`, `terrtype.c`, `terrobjs.c`, `terrre.c`, `terrbuf.c`, `terrcons.c`, `terrdr*.c` | listed in `sources.txt` 1016-1027 | Leaves the DLL only when no active consumer remains (the epic's done-when) | last |

A candidate "G1 only" cut is a **large** change: G1 has dozens of consumers across movement,
targeting and guides, all of which read through the one shared record. This audit does not choose the
host protocol; section 6 lists what the characterisation must settle first.

## 3. Terrain surface service: source findings (read from `terrelev.c`)

1. **Outputs.** One query returns an elevation and, as a side effect, writes the caller's record (or
   the shared static `system_point_data` when the caller passes NULL): `last_elevation`, `face_normal`,
   `terrain_type`, the sector, face and triangle hint.
2. **Return is not always `last_elevation`.** The Cuba map (`map_number == 2`), sectors x 71 to 72 by z 58
   to 60, returns the sector *minimum* height but stores the *maximum* in `last_elevation`, type INVALID,
   normal (0,1,0) and clears the hint. Callers that read `get_3d_terrain_point_data_elevation` (e.g. the
   LOS solver, `mb_tgt.c`) get a different number from callers that use the return value.
3. **Find-fast hint.** If the record already holds a face in the *same sector* the query tries that
   triangle first and, on a hit, keeps the record's *stored* normal and type. A full scan tests all fans in
   face order, then strips in face order, skipping hedge, wall and forest-side types in the strip pass. So
   where faces overlap in x/z the answer can depend on the hint left by the previous query on that
   record. Whether this ever changes a value on the shipped maps is **unmeasured**; the fixture sweep
   measures it.
4. **Shared state.** Every query that passes NULL (`get_3d_terrain_elevation` and most movement code)
   reads and writes the one `system_point_data`; the answer of one subsystem leaves the hint for the next.
   Order of calls across subsystems therefore matters for the *path* taken (fast or slow), and the
   campaign's update order must be preserved by any extraction.
5. **Wrapper quirks.** `get_3d_terrain_elevation_and_face_normal` ignores its record argument (it queries with
   NULL and copies the shared normal). Its only compiled non-comment caller is `faction.c:1397`, with a NULL
   record; the `ai_route.c:1221` call that passes a record sits in a commented-out block. It and
   `get_3d_terrain_face_normal` `ASSERT` strictly inside the map.
6. **Off-map.** The query `ASSERT`s, then clamps x and z to the map extent minus or plus 0.0001. In
   a build where `ASSERT` is not fatal the clamp is the behaviour; this is **not probed** (the original asserts).
7. **Numerics.** Elevation is the stored vertex height of the triangle's second point minus a plane
   correction `(nx*dx + nz*dz) / ny`; the normal is the normalised cross product of two edges with the
   y components rescaled by `TERRAIN_3D_XZ_SCALE`; an elevation outside +-32000 becomes 0. The
   commented-out guard for `ny` near 0 is not active, so a near-vertical face divides by a small `ny`.
8. **Failure.** No face found: a debug log line, elevation = the sector's maximum height, normal (0,1,0), type
   INVALID, hint cleared; the query does not fail.
9. **FPU and RNG.** The query uses `float` arithmetic under the engine's round-toward-zero entry mode; any
   host-side replacement must reproduce or declare the difference. No random draw was seen in the query.

## 4. Representative consumers and what the existing evidence already holds

- **Movement and plans** (`hc_move`, `fw_move`, `rv_move`, `ps_move`): D1/D2 moved the arithmetic to the host and left the
  *terrain read* in the campaign, answering host lookups (`terrain_sample`, `fixed_wing_terrain`,
  `hc_terrain_samples`, `PersonEnd.elevation`). D3 reverses who answers.
- **Line of sight** (`mb_hostlos.c`): terrain samples are asked by the host at phases 1 to 3; the frozen original
  lookup transcript is `reference/d2-los-observation/original-transcript/` (40 s, per-sample point and elevation).
- **Weapon steps** (`wn_move.c`): ELEVATION, FACE_NORMAL, GROUND_PROBE, GROUND_IMPACT, COLLISION, TREES, WIND are
  already crossed lookups, answered by the DLL from its own terrain, trees, entities and session.
- **Boot geography** (`popread.c` places keysites and checks water class; `faction.c` places forces and tests land
  class; `taskgen.c`, `croute.c`, `ai_route.c` set route heights): campaign algorithms that need terrain values.

## 5. Open questions the characterisation must settle before choosing a protocol

1. **Volume and order**: how many queries per frame and at boot, from which call sites, and with how
   many through the shared record (the fixture's run-wide report answers this).
2. **Order dependence**: does a primed hint ever change a value on Lebanon or Georgia (the sweep answers this)?
3. **Representation**: the host answers an EECH surface-type id, or a DCS surface type that the campaign
   maps with a named table. Unresolved; needs the technical lead.
4. **RNG**: weather/wind setup consumes the shared random stream at session creation; an extraction that stops
   calling it changes every later draw unless the draws are kept or the change is declared and reviewed.
5. **Where the reference host's terrain lives**: the EECH World host does not hold the map. It must reuse the
   original terrain loader and solver (transitional, as D2 reused EECH arithmetic) or read map data itself.
   Not decided here.
6. **Normal**: DCS has no face-normal call. A normal would be an explicit data workaround (differences of
   `land.getHeight` samples, or host-supplied data); its numerical difference from EECH's face normal must be
   declared, not hidden.

## 6. DCS reference map (reference only; not live-verified)

From the Root's retained copy of `tslua-dcs` at `1179554f` (`R:/d3-evidence/dcs-api-reference/PROVENANCE.json`).
These are candidates for the D3 services, not claims of DCS behaviour; M6 verifies live.

| D3 service | DCS call or workaround |
|---|---|
| Elevation | `land.getHeight(vec2)`; `land.getSurfaceHeightWithSeabed(vec2)` for water depth |
| Surface type | `land.getSurfaceType(vec2)` (DCS classes) mapped by a campaign table to EECH types and classes |
| Face normal | none; named workaround (see 5.6) |
| Terrain segment / ground impact | `land.getIP(origin, direction, distance)`; terrain profile `land.profile` |
| Terrain line of sight | `land.isVisible(a, b)` is terrain-only; object blocking is a separate service |
| Roads (route data) | `land.getClosestPointOnRoads`, `land.findPathOnRoads` (campaign route planning stays core) |
| Wind | `atmosphere.getWind(vec3)`, `getWindWithTurbulence`; temperature and pressure `getTemperatureAndPressure` |
| Time | `timer.getTime`, `timer.getAbsTime`, `timer.getPause`; scheduling stays in the campaign |
| Objects / collision | not covered by these four files; to be mapped |

## 7. Executed versus prepared

- **Executed:** none for D3. The tool environment has no C compiler or cargo on its path; no syntax check of
  the new C was possible, and no build, native run or test was run.
- **Prepared (source only, uncompiled):** the `terrain-fixture` feature and the characterisation script, described in
  `eech-campaign/reference/d3-terrain-characterization/README.md`. Fixture-gated lines were added to
  `modules/3d/terrain/terrelev.c` and `terrelev.h` (preprocessed output is unchanged without the define).
- **Reused as baseline:** accepted D2 pack (40 checks, eight complete artifacts byte identical).
- **Dormant and uncovered, retained:** G7 above; the off-map path; the Cuba flaw area unless the Cuba map is run;
  player, multiplayer and save/restore paths.

## 8. Blockers and next actions

Blockers: the fixture has never been compiled; a Windows fixture build (Docker image) is the Root's.
The questions in section 5.3 and 5.5 need the lead before any extraction. Next bounded action: independent review
of the fixture source, then the build and runs in the README plan.
