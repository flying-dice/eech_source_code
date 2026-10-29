# D3 terrain and environment — source census and original-characterization plan

29 October 2026. Branch `codex/d3-terrain-services`, integration base
`6bbaa8197a4843181d03cd355d562af998c75321` (verified HEAD, clean at session start).
Authority: the live [M5 goal](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337)
(v3), the [migration ledger](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/14548994),
Epic [EEDCS-21](https://flying-dice.atlassian.net/browse/EEDCS-21) and Task
[EEDCS-47](https://flying-dice.atlassian.net/browse/EEDCS-47). Old W07/W08 wording, the
TypeScript experiments and earlier inventories are not authority.

**Status: original a8 fixture controls completed; corrected 6f host backend
compiles and passes both retail-map conformance cases. Production DLL dispatch
is unchanged: no physical service has moved.** Sections 2 to 6 record the initial source survey; section 7 gives
current execution evidence. Original controls do not establish host ownership.
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
| G3a | **Object/tree collision**: first object or tree a segment meets, object windows | `get_line_of_sight_collision_entity` (`en_cllsn.c:1144`), `get_line_of_sight_collision_tree` (`:1747`); campaign-side callers `mb_tgt.c:455`, `mb_hostlos.c:275`, `wn_move.c` COLLISION and TREES lookups; D2 blast/bridge object windows | Host service; entity-pointer exclusion is coupling D2 declared | later D3 Task |
| G3b | **Model sub-object and device viewpoints** (active): the world position and attitude of a weapon device's heading, pitch, weapon-station and muzzle sub-objects, found by walking the entity's 3D model (`find_object_3d_sub_object`, `get_3d_sub_object_world_viewpoint`/`get_3d_sub_object_viewpoint`) | `weapon.c`: `get_viewpoint_from_weapon` (`:197`, world viewpoint at `:221`), `detach_local_entity_weapon` (`:323`, station viewpoint `:421`, muzzle `:447`, `:585`), `update_entity_weapon_systems` (`:1265`, sub-object search `:1756`), `update_entity_weapon_system_weapon_and_target_vectors` (`:2214`, `:2360`), `get_local_entity_selected_weapon_to_target_offsets` (`:2404`, `:2516`), `get_local_entity_selected_weapon_viewpoint` (`:2538`, `:2628`), `suppress_ineffective_ship_weapons` (`:2836`, `:2962`); `sub_anim.c:169 get_sub_object_relative_position`; model database `modules/3d/3dobjdb.c`, `3dsubobj.c`. D2's hosted aim, launch and device-step paths receive the results of these reads as campaign-supplied step inputs (per `D2-SCOPE-AUDIT.md`; the exact step fields were not re-audited here) | **D3 physical model service** (host reports the model geometry; the campaign must not read its own model for a physical solve). Not deferred to D5 | later D3 Task |
| G3c | **Model target points, offsets and bounds** (active): the aim point of a target entity from the model's TARGET_POINT sub-object, or for a fixed object its position plus the per-shape `target_point_offsets` rotated by heading | `en_maths.c:166 get_local_entity_target_point` (sub-object search `:199`, world viewpoint `:205`, offsets `:228-245`); callers `weapon.c:748` and `:957 fill_aim_target` (hosted aim step input, commented "transitional lookup: the host may know better"), `wn_move.c:379` (weapon-step TARGET_VEC lookup), `wn_msgs.c:339`; the offset table `target_point_offsets` is declared in the weapon database (`wn_dbase.c`/`.h`) and also read in `wn_funcs.c` and `gd_attck.c` | **D3**: the model sub-object lookup is a host model service; the per-shape offset table is campaign data that a DCS host would need a named replacement for (DCS object types differ from EECH shape indexes) | later D3 Task |
| G4 | **Weather and environment**: wind at a point, weather model position and mode | `session.c:595 get_session_wind_velocity_at_point`; the only campaign-reachable consumer found is the weapon step lookup `wn_move.c:399-411`. The session entity also *advances* wind gusting and weather position with `get_delta_time` (`ss_updt.c:484`, `:357-390`) and draws its random start values from the shared random stream (`set_session_random_weather_mode`, `session.c`, `frand1`/`rand16`) | Host service for wind; the RNG draws must be accounted for, see 5.4 | later D3 Task |
| G5 | **Host time inputs**: frame milliseconds, simulated clock, session elapsed time and time of day | `eech_engine.c:296` seeds the simulated clock from the run seed, `:332-334` advances it by the host's milliseconds and calls `set_delta_time` (which keeps a delta-time average and history); `ss_updt.c:79-119` applies elapsed time and time acceleration to time of day; `get_session_random_start_time_of_day` (`ss_creat.c:139`) draws the start | Host supplies the elapsed time; the campaign keeps update semantics, timers and scheduling | later D3 Task |
| G6 | **Campaign-owned consumers of terrain**: setup geography, route planning, task generation | `popread.c` 14, `faction.c` 9, `taskgen.c` 5, `croute.c` 2, `ai_route.c` 3, `keysite.c`, `wp_creat.c` | **Stay in the campaign.** Only the source of the terrain value changes, through G1 | no extraction; verified through G1 |
| G7 | **Candidate render, view and player-only users** of the same terrain and model APIs (not yet proven dormant): gunship flight dynamics (13 models, 188 occurrences in 14 files), cameras (`cm_*.c`, many call `get_local_entity_target_point`), cockpit/avionics/EO/radar/HUD/MFD (`co_eo.c`, `co_hms.c`, `co_radar.c`, `*_hud.c`, `*_mfd.c`, `*_vckpt.c`), `vm_*.c` views, UI maps, drawing (`*_draw.c`, `3dshadow.c`, `3dstrike.c`), `animate_and_draw_entity_muzzle_flash_effect` (`weapon.c:2023`), helicopter rotor animation and damage-fragment geometry (`hc_anim.c`: `damage_helicopter_main_rotors` `:1009`, `..._tail_rotors` `:1215`), Tacview overlay | compiled. The headless frame does not draw (`eech_engine.c:306-336`) and no Lua export reaches the player and view code, **by source reading only**. No compiled-symbol or call-graph audit has been done. The rotor-damage functions change model visibility state and read sub-object positions on a damage path and are *not* classified here: whether any campaign path calls them is open | A function is moved to "dormant" only with source and dispatch evidence of that function (as D2 did for the cockpit sight); until then an active-looking user stays in scope. An active service is never inferred dormant, and never deferred to D5 by a comment | a closing audit |
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
   record. The original fixture sweep subsequently found changed physical tuples
   at 375 Lebanon and 385 Georgia points; same-record exact repeats matched.
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
| Objects / collision, model viewpoints, target points | not covered by these four files; to be mapped (G3a to G3c) |

## 7. Executed versus prepared

- **Executed original behavior:** clean a8 Windows fixture build/controller 0;
  both 20,006-point retail probes and the one-frame/two-due-probe smoke passed.
  Four 600-second controls completed with native/controller 0. Complete
  Lebanon/repeat traces match; all 137 ordinary boot/run lines match the no-probe
  control. The ordinary 900-second control completed with native/controller 0;
  complete Tacview, observations and metrics match the verified D2 shipping900
  donor. All five post-input checks and 43 build entries match. Safe summary:
  `eech-campaign/reference/d3-terrain-characterization/original-controls-a8effa9f3.json`.
  The combined f4 review independently checked these bounded original results,
  input hashes and build files; no visual/performance claim.
- **Prepared backend, not accepted:** f4b61a475 adds the host terrain backend,
  public-provider harness and original fast-elevation oracle. Its clean Windows
  fixture compile exited 1 at `terrain_ref.c:1444` (unmatched `#endif`). No backend
  native run, fast-oracle result or host authority is established. Production DLL
  dispatch remains unchanged. The new parser/FFI/harness delta is under review.
- **Corrected kernel validation:** clean 6f13beae1 Windows fixture/focused-test
  builds exited 0. Seven synthetic production-loader/query/sweep/concurrency tests
  and three real-map conformance tests on each map pass, including new original
  fast-elevation hashes. All old original smoke rows remain exact except one
  source-diagnostic line shift per map (558 -> 585, same statement verified).
  Both post-input manifests and all 44 exported fixture files match. See
  `backend-validation-6f13beae1.json` in the reference directory. This does not
  establish public campaign authority or shipping acceptance. Five review
  findings close by source; stricter refusal markers and word-reference test
  coverage are the next tiny delta before actual production handoff.
- **Feature-off limit:** the old solver/control logic is unchanged by inspection,
  but exact preprocessed identity is not claimed: diagnostic source locations
  shift. A new shipping build and regression remain delivery requirements.
- **Reused as baseline:** accepted D2 pack (40 checks, eight complete artifacts byte identical).
- **Unproven reachability and uncovered paths, retained:** G7 above; the off-map path; the Cuba flaw area unless the Cuba map is run;
  player, multiplayer and save/restore paths.

## 8. Blockers and next actions

Next: correct the combined f4 source findings, freeze and build corrected source,
run quick original fast-oracle probes and both real-map conformance cases, then
prove the new public contract fails on the old DLL and implement the required
host handoff. Preserve shared/private hints, query order and original quirks.
The missing fast-grid generation path is an explicit compatibility gap. Final
shipping regression, independent review, checks and normal merge remain required.
The lead's backend decisions and remaining D3 coupling are recorded in section 10.

## 9. Corrections after the first independent source review (pinned `257a18879`)

Two P2 and two P3 findings; nothing here was built or run.

1. **P2, driver arguments** (`terrain-characterization.lua`): arguments are now validated, with `terrain-characterization:`
   errors, before the output file is opened, the module is required or anything boots. Bounds and the
   rules for `probe_at` are in the driver's header; defaults (600 s, 100 ms, `0,300`, 20000, 64, seed 1) are unchanged,
   and an explicit empty `probe_at=` is still the no-probe control. The clock must advance on every frame. Two existing valid
   modes are preserved and not narrowed: `seconds=0` (with `probe_at=0`) is an instantaneous boot-and-sweep that runs no frame,
   so no clock-progress check applies and every requested probe must still be taken (the default `probe_at=0,300` with
   `seconds=0` is still refused, since 300 is unreachable); `probe_seed=0` is accepted as a u32 and the existing C sweep
   treats 0 as 1, so seeds 0 and 1 give the same sweep. A positive duration still needs at least one bounded frame.
   Guard history, all through the Root's real Lua 5.1 argument-setup helper with an I/O tripwire (never boots EECH or
   runs a frame, so it does not show that a frame loop terminates in a run): v1 was RED on `257a18879` (1 valid control
   passed, all 10 invalid cases touched the output file); the Root's corrected working tree was 11/11 at an earlier
   hash (`577ffab4`); the v2 cases, which add the two positive modes above, were 4/14 on `257a18879` and 12/14 on the first
   correction (only `valid_probe_only` and `valid_zero_seed_alias` failed). The re-run on the bytes carrying this
   preservation fix is the Root's; none of this is native characterization acceptance.
2. **P2, census gap**: G3 omitted active model geometry. It is now G3a (collision), G3b (device and sub-object
   viewpoints) and G3c (model target points, offsets, bounds), all classed D3; G7 is narrowed to candidate render, view and
   player-only users that are *not yet proven* dormant. No active service is inferred dormant or deferred to D5.
3. **P3, observer fidelity** (`eech_terrain_fixture.c`): `slow_scans` now counts every full sector scan, including one
   that finds no face (`not_found` stays a separate subset). Each report line carries `partition_ok`, which is 1 when
   `fast_hit + slow_scans + cuba == calls`, and the driver refuses a report where it is 0. This adds no claim that a
   no-face query occurs at runtime.
4. **P3, evidence wording**: "preprocessed output unchanged" was removed from this audit and the README (section 7).

Root verification after the preservation fix: the real Lua 5.1 v2 setup guard
completed with actual exit 0, all 14 cases passing; syntax parsing also passed.
Tested driver SHA-256: 4400e7eded209f3cd721f3dfa290f7853011eddd9fcca99cb8c9a104d9e0749f.
The helper uses an output-opening tripwire and never boots EECH or runs a frame.
This supersedes the pending guard re-run statements above; C/Rust compilation,
original native characterization and observation controls remain unrun.

### Delta pass on `f05005832`

The census, slow-scan and feature-off-wording findings were closed; one residual of the argument finding and one label remain,
both corrected in source only.

- **Due-probe draining** (`terrain-characterization.lua`): the frame loop took at most one due probe per frame, so accepted
  ascending times inside one frame (for example `seconds=0.1 frame_ms=100 probe_at=0.01,0.02`) lost the later ones and failed the
  reached-probe assertion. It now takes every due probe at each frame in order, with `while`, bounded by the finite probe list
  (`next_probe` only increases). Default `0,300`, the zero-duration mode, report order, one probe per requested time and the
  actual frame and clock values are unchanged. The Root's local scheduling check (`R:/d3-evidence/terrain-driver-schedule.py`,
  real Lua 5.1 with one controlled engine-clock collaborator, not EECH) was 3/5 on `f05005832` (single_due, zero_duration and
  no_probe passed; two_due_same_frame and last_frame_burst failed). It is a local workflow test, not native characterization; a
  one-frame, two-due-probe native smoke stays required after the fixture build. No result for the corrected driver is claimed
  here; the re-run is the Root's.
- **Label**: section 7 called G7 "dormant and uncovered", against the table's not-yet-proven disposition. It now reads "Unproven
  reachability and uncovered paths". No dispatch census was repeated and no dormant proof is claimed.

Root verification after the due-probe correction: actual Lua 5.1 syntax passes,
all 14 setup guard cases pass and all five local scheduling cases pass.
Driver SHA-256: 0a42d3c6928c1da65d2861979e1a4024cd2a9d390457343aeadd822d8af6d035.
The scheduling test controls only the fixture clock collaborator; it does not
boot EECH or implement terrain. The actual native one-frame/two-probe smoke
and all original characterization/recording controls remain required.

## 10. Task 47 implementation, increment 1: independent host terrain backend (source only; nothing built or run)

Scope per the lead's decisions: the required point-surface queries, `get_3d_terrain_fast_elevation` and the boot map facts move to a host that
owns and loads the terrain itself, with EECH's own solver kept in C in `eech-world` and the legacy find-fast record carried across the public Lua
boundary as an opaque value. This increment adds the backend and the RED preparation only; **production DLL dispatch is unchanged** and nothing is
delivered. Details, inputs, validation and the exact checks to run are in `eech-campaign/reference/d3-terrain-characterization/README.md`.

- New: `eech-world/csrc/terrain_ref.c|h`, `src/terrain.rs`, `tests/terrain_conformance.rs`, `lua/terrain-provider.lua`; wiring in `build.rs`, `lib.rs`,
  `main.rs`. The solver body is verbatim from `terrelev.c` at `6bbaa8197` with the engine's own patches N1, N3 and X1 applied.
- The fast grid is a separate file (`terrain.sim`) with its own format, not derived from the point loader; absent, the original generates and saves
  it, which the host does not do.
- Measured on the original (R1, Lebanon 600 s): 46.4 M terrain queries (about 7,700 per frame), 92% on the shared record. A provider call per query is
  an open cost question for the production design.
- Named D3 transitional coupling still in the DLL after this increment (removed by whole-D3 closure): the legacy terrain loader and sector data,
  `gd_cover.c` type triangles, `ss_updt.c` sector types, tree data, ground-impact geometry, and the model/device geometry of G3a to G3c.
- Not claimed: compilation, any conformance result, the RED run, the DLL client, host registration, or any regression. The original fixture does
  not probe the fast grid in the a8 build; the fixture source now samples the original fast elevation at the sweep points (README, "Fast-elevation oracle
  addition"), which the Root builds and runs before the comparison exists. The missing-grid generation of the original is a named compatibility gap.

### Correction batch after the independent review of `f4b61a475` (source only)

Six grouped findings (one build blocker, five P2) on the new host terrain backend are corrected in source: the stray `#endif`, overflow-safe geometry spans with
aligned kernel-owned copies, one lock around the C solver's statics, hints bound to the issuing map instance and to their fields, the sweep's original bounds
with only produced samples returned, and a provider harness that accepts only the right refusal. Details and the new synthetic-map tests are in the README of
`eech-campaign/reference/d3-terrain-characterization/`. Nothing is built or run; no conformance or acceptance claim; production DLL dispatch unchanged; the
missing-grid generation gap stays open.

## 11. G1 production handoff (source only; not built or run by the author)

The physical terrain surface (point query, fast elevation, record getters, boot facts) is now the host's in the headless module: a required `terrain` provider at boot
(`[terrain-provider:missing]` otherwise), the original solver excluded from the shipping DLL by `EECH_CAMPAIGN_HOST_TERRAIN` (`terrelev.c`, the fast function of
`terrsel.c`), `mb_hostterrain.c` as the only terrain source, opaque bounded hints stored in a pointer-free record, stable first-line refusal markers, the reference host's
`host.terrain_provider` registered in every reference script that boots the DLL, and the original's missing-grid generation reproduced in the host kernel. Campaign class,
suitability, geography, route planning and consequences are unchanged. Named transitional coupling remains and is not deferred to D5: the DLL's own load of
`default.sec`/`terrain.ffp` for the sector users (G2: trees, per-sector types, ground impact, the 2D map), the object/model/device geometry (G3a to G3c), weather and time.
Contract, DCS mapping and workarounds, the consumer audit, the tests added and the risks (notably the cost of about 7,700 provider round trips per frame) are in
`eech-campaign/reference/d3-terrain-characterization/README.md`. Not claimed: compilation, any run, the public GREEN, the ordinary shipping regression, or D3 delivery.
