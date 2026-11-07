# D3 needed-consumer and DCS API disposition (second and final preparation batch)

Task EEDCS-49 (D3), 7 November 2026. Source `c6426dc310796795a0d8c694cb985b65b86d8e7d` (clean). **Documentation and source disposition only**: no physical implementation, numeric prototype, Rust/C/Lua/runtime/build/ABI change, test, compiler or native run. `physical_cutover_clearance` stays false and `ENV-ENTITY-POSE-LONG-RUN-MOTION-1` stays OPEN with `blocks_cutover=true`. The machine-readable matrix is [`needed-consumer-api-matrix.json`](../reference/d3-needed-consumer-api-disposition/needed-consumer-api-matrix.json); every file/line below is resolved from that clean tree by the generator, not typed. Authority: live M5 goal 11534337 v3 and ledger 14548994 v171, which outrank Jira, the historical roadmap and the old W07/W08 labels.

Labels: **source-derived** = read in the current tree (inclusion and reachability inferred from `sources.txt`, `#if` structure and call chains, not a measured preprocessed/symbol/runtime census); **declared** = a pinned `flying-dice/tslua-dcs` declaration (commit `1179554f`), a shape and never live DCS behaviour; **reported** = from a retained record, not reproduced here.

## Overview

| id | group | disposition | DCS status |
|---|---|---|---|
| D3G-01 | Sector high-water height: writers, readers, save and restore | REQUIRED_NEXT_PHYSICAL_AFTER_BLOCKER | OPEN |
| D3G-02 | Terrain point record, sector hint token and the face-normal question (effectful shared terrain state) | ACCEPTED_REUSE_WITH_RESIDUAL_OPEN | OPEN_WORKAROUND_NAMED_UNPROVEN |
| D3G-03 | Destructive whole-word UPDATED / MOVED / ROTATED flags and their real immediate consumers | REQUIRED_OPEN | OPEN |
| D3G-04 | Raw TREE / FOREST / CLOUD loader, season, and the boot CRT stream | REQUIRED_OPEN_BOUNDED_CAPTURE_NEXT | NOT_REQUIRED_FOR_DATA_OPEN_FOR_RNG |
| D3G-05 | Scene constructors (skin_random) and the later CRT consumers | REQUIRED_OPEN | NOT_A_DCS_API_QUESTION |
| D3G-06 | Entity and object collision primitive: narrow phase, candidate decision and which Core callers need a DCS-facing primitive | REQUIRED_OPEN | OPEN |
| D3G-07 | Tree, forest and scenery blocking (weapon TREES lookup, LOS through trees) | REQUIRED_OPEN | OPEN |
| D3G-08 | Model hierarchy: weapon device, station and muzzle sub-objects and world viewpoints | REQUIRED_OPEN | OPEN |
| D3G-09 | Campaign-setup model consumers: airfield waypoint routes, troop landing/takeoff/patrol routes, regeneration type | REQUIRED_OPEN | PARTIAL_DECLARED_SHAPES_ROUTE_AND_SUBOBJECT_CONTRACTS_OPEN |
| D3G-10 | Rotor damage, blade fragments and smoke attachment (physical services with shared RNG) | REQUIRED_OPEN | NOT_A_DCS_API_QUESTION |
| D3G-11 | Default shape and ground-radar-clutter columns | REQUIRED_OPEN | OPEN_WORKAROUND_NAMED |
| D3G-12 | Six compiled draw files reading fixed_database[].align_with_terrain | CENSUS_OPEN_SOURCE_DERIVED_UNREACHED | NOT_REQUIRED_PENDING_FINAL_CENSUS |
| D3G-13 | Natural moving and public authority | BLOCKED_BY_ENV_ENTITY_POSE_LONG_RUN_MOTION_1 | DECLARED_SHAPES_PRESENT_BEHAVIOUR_UNVERIFIED |
| D3G-14 | Final active-consumer and compile census: player, view, HUD, cockpit, camera, UI and gunship dynamics | CENSUS_OPEN | NOT_REQUIRED_PENDING_FINAL_CENSUS |
| D3G-15 | Accepted scopes reused by exact proof (not reopened) | ACCEPTED_REUSE | ACCEPTED_REUSE_PER_SCOPE |

The CRT draw consumers are listed once (D3G-04/05). Groups 12 and 14 are census rows: nothing is called dormant by name, by one unreached run, by a generic count, by source grep or by a missing export.

## D3G-01. Sector high-water height: writers, readers, save and restore

- **Requirements:** 1, 2, 6. **Disposition:** `REQUIRED_NEXT_PHYSICAL_AFTER_BLOCKER`.
- **Branch/profile:** Writers and restore in every build; hosted scan and candidate fact in every build except the two original-solver profiles; fixed-wing clearance unconditional (source/build-derived, docs/d3-sector-height-contract.md section 2).
- **Source chain (function, caller or reader):**
  - `aphavoc/source/entity/special/sector/sector.h:81` `tallest_structure_height` - one float per sector
  - `aphavoc/source/entity/special/sector/sc_seccreat.c:236` `memset (raw, 0, sizeof (sector))` - zero fill at creation
  - `aphavoc/source/entity/system/en_funcs/en_list.c:506` `ENTITY_MESSAGE_LINK_CHILD, parent, en, type` - list funnel notifies the sector
  - `aphavoc/source/entity/special/sector/sc_msgs.c:140` `if (height > raw->tallest_structure_height)` - the only raise: strict > over the held word; fixed entities with a valid shape
  - `aphavoc/source/entity/special/sector/sc_pack.c:287` `pack_float_value (en, FLOAT_TYPE_PROTOTYPE_1, raw->tallest_structure_height)` - save as raw 32 bits (server session)
  - `aphavoc/source/entity/special/sector/sc_pack.c:407` `raw->tallest_structure_height = unpack_float_value` - restore of the raw word
  - `aphavoc/source/entity/special/sector/sc_float.c:201` `value = raw->tallest_structure_height` - the typed getter
  - `aphavoc/source/entity/mobile/mb_hostscan.h:164` `CAMPAIGN_SCAN_TALLEST (sector)` - hosted scan: once per served sector
  - `aphavoc/source/entity/mobile/mb_hostoffer.h:133` `facts [CAMPAIGN_CANDIDATE_TALLEST]` - fact 12 of every candidate question
  - `eech-campaign/crates/eech-world/csrc/entity_candidate_step.c:446` `source_min_position.y > facts[12]` - host kernel compares it
  - `aphavoc/source/entity/en_misc/en_maths/en_cllsn.c:1384` `FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT))` - original scan reader (the #else, original-solver profiles only)
  - `aphavoc/source/entity/mobile/aircraft/fixwing/fw_move.c:1243` `get_local_entity_float_value (sector, FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT) - terrain_elevation` - fixed-wing clearance, unconditional, before the hosted movement block (:1247)
- **Ownership now:** Core physical state (writer, save/restore, getter); Host already consumes the word as candidate fact 12 and already owns the box top (Task50) and the fixed position/shape statements.
- **What the same DLL needs when DCS supplies physical movement and combat:** The fixed-wing waypoint-altitude clearance (a campaign movement input) and, only if the DCS profile asks the EECH narrow phase at all (D3G-06), the candidate skip fact. The save needs the word to round-trip raw (13 measured patterns; wider domain source-derived; 033defb4).
- **Host contract already present:** ENTITY_SCAN grammar and ENTITY_CANDIDATE fact 12 carry the word from Core to the host today. No host-owned scalar, no restore acknowledgement phase, no host answer for the clearance read yet.
- **Bounded required outcome:** One host-owned per-sector high-water scalar, raised by the host's fixed-state statements under the characterised strict-greater / toward-zero / raw-restore rules, read through the existing scan grammar plus two new exact phases (clearance/save, restore acknowledgement), Core's field kept as a conformance mirror until only the save reads it (contract section 6). Not built here.
- **DCS:** status `OPEN`; declared calls: `land.getHeight` (land.ts:41).
  - Missing fact: No verified call returns the tallest obstacle in an EECH sector. land.getHeight is terrain only; world.searchObjects needs unverified Category/volume constants and only UnitDesc.box (units) is declared. Either a verified static/scenery obstacle declaration or a versioned per-map per-sector clearance table with provenance (not defined), or an approved semantic change of the clearance policy (technical lead), is required. The `tallest - terrain_elevation` unit mix is preserved, not approved.
- **Reach evidence (source/build-derived):** list funnel en_list.c:506 -> sc_msgs.c response_to_link_child; fixed-wing movement step fw_move.c:1243; session pack/unpack sc_pack.c:287/407. Not proved: native observation of campaign word vs a host high-water through the public driver; whether any real CITY is created with a valid shape; engine C at -O2.
- **Reuses (exact scope):** R-T47, R-T50, R-FINITE.
- **Open items kept:** sector-height-host-move-open-facts; city shape unknown; no public-driver observation.

## D3G-02. Terrain point record, sector hint token and the face-normal question (effectful shared terrain state)

- **Requirements:** 1, 6. **Disposition:** `ACCEPTED_REUSE_WITH_RESIDUAL_OPEN`.
- **Branch/profile:** EECH_CAMPAIGN_HOST_TERRAIN builds (all but the original-solver profiles). Hint, last_elevation, stored type and normal are the host's answer kept in Core's record; the query order and which record each query writes are the original's.
- **Source chain (function, caller or reader):**
  - `aphavoc/source/entity/mobile/mb_hostterrain.c:22` `system_point_data;` - Core shared value cache for queries that pass no record
  - `aphavoc/source/entity/mobile/mb_hostterrain.c:135` `system_point_data.hint[0] = '\0';` - hint forgotten at map load, as the original
  - `aphavoc/source/entity/mobile/mb_hostterrain.c:193` `strcpy (point_data->hint, reply.hint);` - the host's hint token stored in the caller's record
  - `aphavoc/source/entity/mobile/mb_hostterrain.c:35` `season_captured;` - season captured where terrdata.c read it
  - `aphavoc/source/entity/en_misc/en_maths/en_cllsn.c:494` `#define CAMPAIGN_OFFER_TERRAIN_NORMAL` - hosted candidate dialogue: Core performs the face-normal question once when the host names NEED_NORMAL
  - `aphavoc/source/entity/en_misc/en_maths/en_cllsn.c:586` `if (get_local_entity_int_value (target, INT_TYPE_ALIGN_WITH_TERRAIN))` - original branch reads the align flag then the normal
  - `modules/3d/terrain/terrelev.c:108` `system_point_data;` - original shared record (not compiled in hosted builds: symbol-ownership-251bfe15b.json)
- **Ownership now:** Host solver (Task47/48); Core keeps the shared value cache and the order of queries as declared transitional coupling. The hint is an opaque host token.
- **What the same DLL needs when DCS supplies physical movement and combat:** Elevation, surface type and ground contact come from the host. The face normal feeds the aligned-fixed candidate pose and ground vehicle orientation inputs in the EECH reference path only; a DCS host leaves the hint empty (its own business).
- **Host contract already present:** Terrain point/fast/map-fact provider with hint carriage (Task47); the NEED_NORMAL stage of ENTITY_CANDIDATE.
- **Bounded required outcome:** Reuse. Residual: a DCS host must state a normal for NEED_NORMAL/align, or the campaign must state that no DCS-profile consumer needs one. No Core terrain solver remains to move.
- **DCS:** status `OPEN_WORKAROUND_NAMED_UNPROVEN`; declared calls: `land.getHeight` (land.ts:41), `land.getSurfaceType` (land.ts:97), `land.getSurfaceHeightWithSeabed` (land.ts:87), `land.getIP` (land.ts:73), `land.profile` (land.ts:122).
  - Bounded workaround: terrain-normal by differences of land.getHeight samples or host data, as named in docs/handover/D3-SCOPE-AUDIT.md section 6 and the terrain README; its numerical difference from the EECH face normal is unmeasured and unapproved
  - Missing fact: No declared normal call (land.ts, declared shapes only). Which DCS-profile consumer, if any, still needs a normal once DCS resolves ground and weapon physics; EECH-to-DCS surface class table is campaign data, not yet defined.
- **Reach evidence (source-derived):** mb_hostoffer.h NEED_NORMAL stage; terrain queries throughout movement and targeting (D3-SCOPE-AUDIT.md section 2 G1). Not proved: off-map clamp path; Cuba map flaw area; fast-grid generation (explicit compatibility gap).
- **Reuses (exact scope):** R-T47, R-T48.
- **Open items kept:** DCS normal; surface-class table; off-map; fast-grid generation.

## D3G-03. Destructive whole-word UPDATED / MOVED / ROTATED flags and their real immediate consumers

- **Requirements:** 2, 6. **Disposition:** `REQUIRED_OPEN`.
- **Branch/profile:** Flag storage and getter in every build. The collision-sweep reader is the original in original-solver profiles and the host-named stage in hosted builds. The Tacview readers run only while tacview_is_logging().
- **Source chain (function, caller or reader):**
  - `aphavoc/source/entity/mobile/mb_int.c:398` `updated_entities [index] &= mask` - the getter clears every other entity's bit in the same 32-bit word and returns the masked word
  - `aphavoc/source/entity/mobile/mb_int.c:178` `updated_entities [index] |= mask` - setter may only set TRUE
  - `aphavoc/source/entity/special/update/up_update.c:157` `memset (updated_entities` - per-pass clear; the byte count MAX_NUM_ENTITIES/32 clears only the first quarter of the array words (array at :86-88)
  - `aphavoc/source/entity/system/en_main/en_heap.h:95` `#define MAX_NUM_ENTITIES` - (131072 * 4)
  - `aphavoc/source/entity/special/update/up_update.c:183` `set_local_entity_int_value (en, INT_TYPE_UPDATED, TRUE)` - set after update_client_server_entity
  - `aphavoc/source/entity/en_misc/en_maths/en_cllsn.c:1448` `if (get_local_entity_int_value (target, INT_TYPE_UPDATED))` - original sweep ends (hosted: CAMPAIGN_OFFER_UPDATED en_cllsn.c:493, NEED_UPDATED in mb_hostoffer.h)
  - `aphavoc/source/entity/mobile/aircraft/fixwing/fw_msgs.c:455` `if (get_local_entity_int_value (receiver, INT_TYPE_UPDATED))` - response_to_check_mobile_reached_guide (:396): pass-through check
  - `aphavoc/source/entity/special/guide/gd_supply_arrival.c:305` `descriptor.updated = get_local_entity_int_value (member, INT_TYPE_UPDATED)` - fixed-wing supply-arrival descriptor, already host-visible
  - `aphavoc/source/entity/mobile/aircraft/fixwing/fw_updt.c:199` `rotated = get_local_entity_int_value(en, INT_TYPE_ROTATED)` - Tacview update, guarded by tacview_is_logging() (:195); same destructive getter
  - `aphavoc/source/entity/mobile/aircraft/helicop/hc_updt.c:911` `moved = get_local_entity_int_value(en, INT_TYPE_MOVED)` - Tacview update, same idiom
- **Ownership now:** Core state with an effect of its own; the host names when the Core getter must run (the finite candidate dialogue, 216 expectations). Recording reads of MOVED/ROTATED are reference-recording behaviour.
- **What the same DLL needs when DCS supplies physical movement and combat:** Three decisions use UPDATED to choose between pos and pos -/+ velocity*dt (collision sweep, fixed-wing guide-reached check, fixed-wing supply-arrival descriptor). Under DCS the pose is sampled after DCS's own step; the meaning of 'already updated this pass' must be stated by the host explicitly. MOVED/ROTATED are not needed by DCS (EECH Tacview logging only).
- **Host contract already present:** Candidate fact [20] carries 0/1/2 for the sweep; the supply descriptor carries `updated`. No contract for the guide-reached read (fw_msgs.c:455).
- **Bounded required outcome:** A host-stated explicit boolean for each of the three decisions (and none for Tacview), with the 32-bit-word neighbour clearing either reproduced for the reference backend or declared; the partial-memset domain (first quarter of the array) must be measured and carried or reviewed. Semantics under a DCS-sampled pose need technical-lead decision.
- **DCS:** status `OPEN`; declared calls: `Object.getPoint` (Object.ts:34), `Object.getVelocity` (Object.ts:61), `timer.getTime` (timer.ts:112).
  - Bounded workaround: host states `updated` per query; which value a DCS-sampled pose carries is a semantic decision, not derivable from the declarations
  - Missing fact: The truth value to state for a DCS-sampled position, approved by the technical lead; whether any entity index >= 131072 occurs in supported runs (partial memset reaches); no public-path observation of the neighbour-clearing effect.
- **Reach evidence (source-derived):** update_client_server_entities up_update.c:112 sets flags per entity; collision scan, guide check and supply descriptor read them in the same pass; Tacview readers need tacview_is_logging(). Not proved: which of the three decisions a DCS profile still ticks in Core; measured behaviour of the neighbour-clearing side effect.
- **Reuses (exact scope):** R-FINITE.
- **Open items kept:** UPDATED truth value under DCS; partial memset domain; fw_msgs.c guide-reached host contract.

## D3G-04. Raw TREE / FOREST / CLOUD loader, season, and the boot CRT stream

- **Requirements:** 1, 2, 6. **Disposition:** `REQUIRED_OPEN_BOUNDED_CAPTURE_NEXT`.
- **Branch/profile:** Ordinary hosted boot (eech_engine.c:251-320). The zone reload path (gameflow.c:787, comm_man.c:2289) is a native-game flow, not the headless boot (source-derived).
- **Source chain (function, caller or reader):**
  - `eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c:251` `set_random_number_seed (config->random_seed);` - boot seed (srand of the CRT)
  - `eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c:300` `session_cloud_puffs = command_line_cloud_puffs;` - default 1 (cmndline.c:163)
  - `eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c:308` `load_3d_terrain_game_data ();` - terrain load in the ordinary boot
  - `aphavoc/source/ui_menu/czone/zone_sc.c:235` `if (!load_3d_terrain (filename))` - reaches load_3d_terrain
  - `modules/3d/terrain/terrdata.c:777` `construct_3d_object_by_name ( "FOREST_TREE_OBJECT" )` - forest object constructor
  - `modules/3d/terrain/terrdata.c:816` `construct_3d_object_by_name ( "CLOUD" )` - cloud object constructor
  - `modules/3d/terrain/terrdata.c:818` `if (session_cloud_puffs > 1 || session_cloud_puffs == 1` - cloud puffs generated unless desert at level 1
  - `modules/3d/terrain/terrdata.c:840` `base = rand() % 8;` - ten rand() sites 840-878 per sector/puff
  - `eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c:314` `reinitialise_entity_system ();` - after the load
  - `eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c:320` `create_campaign (get_current_game_session ());` - parser creates the session entity before the factions
  - `aphavoc/source/ai/faction/parser.c:408` `ENTITY_TYPE_SESSION,` - session created at parse start
  - `aphavoc/source/entity/special/session/ss_creat.c:141` `raw->start_time = get_session_random_start_time_of_day ();` - start-of-day draw
  - `aphavoc/source/entity/special/session/session.c:277` `initialise_random_number_generator ();` - reseeds from the simulated clock (-> random.c:120 srand)
  - `modules/maths/random.c:120` `srand (seed);` - the only srand of the build
  - `modules/3d/terrain/terrgeom.c:933` `terrain_cloud_puff_sectors &&` - cloud data reader: a draw function
  - `aphavoc/source/entity/en_misc/en_maths/en_cllsn.c:1893` `#ifdef EECH_CAMPAIGN_HOST_TREE` - tree collision is hosted: Core's tree data is not read there
- **Ownership now:** Core transitional loader and constructors; tree collision authority is the Host (TREE finite scope closed); tree/cloud draw readers are renderer code.
- **What the same DLL needs when DCS supplies physical movement and combat:** No tree, forest or cloud DATA: DCS owns its scenery and the headless frame does not draw. What the same DLL needs is only that campaign-visible CRT state is not changed by removing the loader. The reseed at session creation (parser.c:408 -> ss_creat.c:141 -> session.c:277 -> random.c:120) is source evidence that the loader draws precede a reseed, and the parser creates the session before its factions (parser.c:408 then 'create factions'), so before the campaign's keysites and forces exist; the card text 'no later reseed found' (core-tree-loader-forest-cloud-rng) is therefore refined, NOT closed: consumers inside reinitialise_entity_system and the effective season/flags per map are unmeasured.
- **Host contract already present:** TREE_BEGIN/TREE_TERRAIN host kernel and provider (finite authority). No contract is needed for CRT draws.
- **Bounded required outcome:** One bounded capture per supported map with the existing counting header (csrc/eech_tp_rand.h): CRT rand() calls inside load_3d_terrain, effective session_cloud_puffs/season, forest/cloud skin draws, and the first CRT consumer after the load; then remove the loader/constructor/cloud block for hosted profiles only if no draw is consumed before the reseed (or the removal is baseline-approved). Original profile retained. No dummy draws.
- **DCS:** status `NOT_REQUIRED_FOR_DATA_OPEN_FOR_RNG`; declared calls: none.
  - Missing fact: Measured draw counts and ordering per map; this is not a DCS API question.
- **Reach evidence (source-derived):** eech_engine.c:308 -> zone_sc.c:235 -> terrdata.c load_3d_terrain; reseed chain above. Not proved: boot-time ordering of CRT consumers between :308 and parser.c:408; forest/cloud skin metadata.
- **Reuses (exact scope):** R-T47, R-FINITE.
- **Open items kept:** core-tree-loader-forest-cloud-rng; forest-geometry-retail-source.

## D3G-05. Scene constructors (skin_random) and the later CRT consumers

- **Requirements:** 2, 6. **Disposition:** `REQUIRED_OPEN`.
- **Branch/profile:** Every build compiles all 19 CRT rand sites (3dobjid.c 2, terrdata.c 10, camera.c 3, ap_cpg.c 2, explosn.c 1, co_undercarriage.c 1; text census, the same count as the Task52 audit) and one srand. skin_random draws only for scenes with a skin animation.
- **Source chain (function, caller or reader):**
  - `modules/3d/3dobjdb.c:2950` `skin_random ( object );` - construct_3d_object ends with it
  - `modules/3d/3dobjid.c:911` `void skin_random ( object_3d_instance* scene )` - returns at once if skin.animation < 0
  - `modules/3d/3dobjid.c:950` `rand() % number_of_textures` - one draw, season-dependent branch (:954 the other)
  - `aphavoc/source/entity/mobile/aircraft/fixwing/fw_creat.c:282` `construct_3d_object` - every mobile creator constructs its instance (hc/aa/ps/rv/sh_creat likewise; pa_creat.c:217 particles)
  - `aphavoc/source/entity/special/regen/regen.c:173` `object_instance = construct_3d_object ( object );` - regen type per shape
  - `aphavoc/source/ai/taskgen/taskgen.c:2105` `object = construct_3d_object ( get_local_entity_int_value (keysite, INT_TYPE_OBJECT_INDEX) );` - patrol route model
  - `aphavoc/source/entity/special/camera/camera.c:449` `!(rand() % one_over_delta_time)` - add_turbulence (:428): camera, three draws
  - `aphavoc/source/entity/special/effect/explosn/explosn.c:695` `rand()/(RAND_MAX+1.0)` - explosion sound sample choice
  - `aphavoc/source/gunships/cockpits/apache/ap_cpg.c:68` `rand() % 3` - player CPG animation
  - `aphavoc/source/gunships/dynamics/common/co_undercarriage.c:654` `rand() * PI / RAND_MAX` - player gear
- **Ownership now:** Core. The campaign's own generator (rand16/frand1) is a different stream, so none of these draws is a campaign-random decision; they matter only through stream position.
- **What the same DLL needs when DCS supplies physical movement and combat:** Nothing from DCS. Under DCS, EECH instances need not exist at all for units DCS owns; the question is whether constructor draws, if kept, can alter a campaign-visible value (camera/explosion-sound/player-gunship consumers are view or effect state) and whether a reseed always separates them.
- **Host contract already present:** Task52 removed the initialiser's constructor loop from ordinary builds; its observer and reported capture (every initialiser pass 5 draws, seeds at the same draw total) are REPORTED, not reproduced here.
- **Bounded required outcome:** Disposition per consumer class by the same bounded capture as D3G-04: draws per creator class (mobile, particle, regen/taskgen/popread constructors) and the first campaign-visible CRT consumer after each; decide per class: kept in Core (reference), erased by reseed (proved), or approved change.
- **DCS:** status `NOT_A_DCS_API_QUESTION`; declared calls: none.
  - Missing fact: Measured per-class draw counts and consumer order.
- **Reach evidence (source-derived):** construct_3d_object callers listed; camera update via update_client_server_entity (eech_engine.c count==0 branch); explosn via effect creation. Not proved: which consumers run in the supported headless flows.
- **Reuses (exact scope):** R-T52.
- **Open items kept:** per-class CRT draw census; consumer order.

## D3G-06. Entity and object collision primitive: narrow phase, candidate decision and which Core callers need a DCS-facing primitive

- **Requirements:** 2, 5, 6. **Disposition:** `REQUIRED_OPEN`.
- **Branch/profile:** Hosted in every build except collision-fixture and model-fixture. Finite stationary, candidate, scan, narrow-phase scopes are closed; the moving public path, natural moving and every unreached state are not.
- **Source chain (function, caller or reader):**
  - `aphavoc/source/entity/mobile/mb_tgt.c:455` `collision_en = get_line_of_sight_collision_entity` - check_position_line_of_sight (:218): campaign target-LOS decision (second call :494)
  - `aphavoc/source/entity/mobile/mb_hostlos.c:275` `collision_en = get_line_of_sight_collision_entity (source, target` - ask_host (:79): hosted LOS, accepted D2 phases
  - `aphavoc/source/entity/mobile/weapon/wn_move.c:466` `hit = get_line_of_sight_collision_entity` - answer_weapon_step_lookup (:213): the lookup callback handed to the host's weapon step
  - `aphavoc/source/entity/mobile/weapon/wn_move.c:776` `campaign_aim_weapon_step (&step, answer_weapon_step_lookup, &context)` - the host's weapon step asks the lookups; Core only answers
  - `aphavoc/source/gunships/avionics/common/co_radar.c:683` `get_line_of_sight_collision_entity` - player radar (also :705, :733); co_force.c:1266/1334 player dynamics
  - `aphavoc/source/entity/en_misc/en_maths/en_cllsn.c:448` `#ifdef EECH_CAMPAIGN_HOST_ENTITY` - hosted body; original kept in the #else (collision-fixture, model-fixture only)
  - `aphavoc/source/entity/mobile/mb_hostmodel.h:103` `ENTITY_COLLISION` - narrow-phase operation
- **Ownership now:** Host reference implements the decision and the narrow phase; Core offers candidates in stages and keeps the effectful getters (D3G-03) and the consequences of a hit.
- **What the same DLL needs when DCS supplies physical movement and combat:** Two classes of caller. (1) Campaign decisions that need line of sight through objects: mb_tgt.c:455/494 and the hosted LOS question mb_hostlos.c:275. (2) The weapon-flight lookup wn_move.c:466 is asked from inside the host's weapon step (wn_move.c:776 hands answer_weapon_step_lookup to campaign_aim_weapon_step); when DCS flies its own weapons, no EECH weapon step runs and these lookups never arrive, so it is reference-backend-only PROVIDED the DCS profile does not instantiate EECH weapon-flight entities: that architectural fact is not yet recorded. Player radar/dynamics callers: see D3G-14.
- **Host contract already present:** ENTITY_CANDIDATE, ENTITY_SCAN, ENTITY_COLLISION, ENTITY_POSE with exact refusals (reference host); LOS host phases (D2).
- **Bounded required outcome:** Name the DCS profile's rule for each caller class. For class (1) a verified object-blocking LOS declaration or a bounded, versioned per-type geometry workaround with provenance; unknown objects and unsupported child state refuse by name, never miss. A UnitDesc box is a different physical model and is not an approved substitute.
- **DCS:** status `OPEN`; declared calls: `world.searchObjects` (world.ts:285), `Object.getPosition` (Object.ts:43), `Object.getVelocity` (Object.ts:61), `Object.getTypeName` (Object.ts:52), `Unit.getDesc` (Unit.ts:482), `UnitDesc.box` (Unit.ts:115), `land.isVisible` (land.ts:111).
  - Missing fact: A declared ray/LOS-against-objects call or per-type geometry data with provenance and supported types; the weapon-entity ownership rule of the DCS profile; the Object.Category and volume constants for world.searchObjects. land.isVisible is terrain-only by its own declaration (land.ts:103).
- **Reach evidence (source-derived):** caller list above; hosted under EECH_CAMPAIGN_HOST_ENTITY; kernel in eech-world csrc. Not proved: which callers a DCS profile ticks; the moving public path; per-weapon attribution.
- **Reuses (exact scope):** R-T50, R-FINITE, R-D1D2.
- **Open items kept:** entity-object-collision; dcs-entity-collision-mapping-unresolved; entity-host-sourced-live-pose.

## D3G-07. Tree, forest and scenery blocking (weapon TREES lookup, LOS through trees)

- **Requirements:** 1, 2, 5. **Disposition:** `REQUIRED_OPEN`.
- **Branch/profile:** Hosted under EECH_CAMPAIGN_HOST_TREE (with HOST_ENTITY); finite TREE authority closed (report 7ad1a698, 11 cases, 87 payloads each).
- **Source chain (function, caller or reader):**
  - `aphavoc/source/entity/mobile/weapon/wn_move.c:493` `get_line_of_sight_collision_tree (&p, &q` - weapon-step TREES lookup (host-initiated)
  - `aphavoc/source/entity/en_misc/en_maths/en_cllsn.c:1893` `#ifdef EECH_CAMPAIGN_HOST_TREE` - hosted tree body
  - `aphavoc/source/gunships/avionics/common/co_radar.c:688` `get_line_of_sight_collision_tree` - player radar (also co_eo.c:681, co_hms.c:733)
- **Ownership now:** Host reference (host tree loader reads installation-layout directory); Core retains the loader (D3G-04).
- **What the same DLL needs when DCS supplies physical movement and combat:** Campaign LOS decisions that today see trees (through D3G-06 callers). The weapon TREES lookup is reference-kernel only (same condition as D3G-06 class 2).
- **Host contract already present:** TREE_BEGIN/TREE_TERRAIN; a DCS host must refuse tree_begin by name until a source exists.
- **Bounded required outcome:** A verified DCS tree/forest/scenery declaration or a versioned per-map dataset with provenance; until then refusal by name. done{hit=false} for an unsupported query was withdrawn: an unknown service is not a measured absence.
- **DCS:** status `OPEN`; declared calls: `world.searchObjects` (world.ts:285).
  - Missing fact: A declaration of scenery/tree objects (Object.Category constants and the volume schema live in exports/*.export.ts, not in the bundle) or a bounded versioned map dataset with provenance.
- **Reach evidence (source-derived):** wn_move.c:493; co_* player callers. Not proved: retail forest store fact (forest-geometry-retail-source).
- **Reuses (exact scope):** R-FINITE, R-T50.
- **Open items kept:** dcs-tree-capability-unresolved; forest-geometry-retail-source.

## D3G-08. Model hierarchy: weapon device, station and muzzle sub-objects and world viewpoints

- **Requirements:** 2, 5. **Disposition:** `REQUIRED_OPEN`.
- **Branch/profile:** update_entity_weapon_systems and the vector update run in the campaign entity update (source-derived); recoil, HUD and muzzle-flash callers are player or draw files.
- **Source chain (function, caller or reader):**
  - `aphavoc/source/entity/mobile/weapon/weapon.c:207` `if (find_object_3d_sub_object (search_weapon_system_heading)` - get_viewpoint_from_weapon (:197), static
  - `aphavoc/source/entity/mobile/weapon/weapon.c:421` `get_3d_sub_object_world_viewpoint (&search_weapon_system_weapon, vp)` - detach_local_entity_weapon (:323): station viewpoint; muzzle :447
  - `aphavoc/source/entity/mobile/weapon/weapon.c:1765` `AIM_FX_WS_EXIT (source, EECH_AIM_WS_EXIT_NO_PITCH_DEVICE` - update_entity_weapon_systems (:1265): device lookup gate; returns without stepping when a device is missing
  - `aphavoc/source/entity/mobile/weapon/weapon.c:1592` `aim_result = campaign_aim_ask (&aim_step, FALSE);` - the device step itself is the host's (D2)
  - `aphavoc/source/entity/mobile/weapon/weapon.c:2360` `get_3d_sub_object_world_viewpoint (&search_weapon_system_pitch, &vp);` - update_entity_weapon_system_weapon_and_target_vectors (:2214)
  - `aphavoc/source/entity/mobile/weapon/weapon.c:2404` `int get_local_entity_selected_weapon_to_target_offsets` - callers are HUD/satellite-view files only (ah_hud.c:2234 ...; vm_satinfo.c:551)
  - `aphavoc/source/entity/mobile/weapon/weapon.c:2538` `int get_local_entity_selected_weapon_viewpoint` - sole caller co_wpn.c:155 apply_weapon_recoil_effect (player)
  - `aphavoc/source/entity/mobile/weapon/weapon.c:2836` `void suppress_ineffective_ship_weapons` - caller wn_tgt.c:137
  - `aphavoc/source/entity/mobile/weapon/wn_creat.c:423` `detach_local_entity_weapon` - launch path
  - `aphavoc/source/entity/mobile/aircraft/fixwing/fw_updt.c:182` `update_entity_weapon_systems (en);` - per-entity update callers: fw/hc/aa/ps/rv_updt.c
  - `aphavoc/source/entity/mobile/weapon/weapon.c:2023` `void animate_and_draw_entity_muzzle_flash_effect` - draw-file callers only (fw/hc/aa/ps/rv/sh_draw.c)
- **Ownership now:** Core model hierarchy (3dsubobj.c, 3dobjdb.c) feeds device presence, station and muzzle viewpoints; the device step arithmetic is the host's; target point and bounds are Host (Task50/52).
- **What the same DLL needs when DCS supplies physical movement and combat:** Device presence (a missing heading or pitch device stops the aim step) and the launch/muzzle viewpoint. When DCS aims and fires through Controller tasks, EECH device stepping and muzzle viewpoints are not needed for DCS-driven units; the DLL needs a rule for which units' weapon systems the DCS profile still ticks in Core.
- **Host contract already present:** D2 aim step receives device angles from Core; Task52 target-point offsets and live point; no hierarchy service (sub-object search, relative/world/station/muzzle viewpoints with actual pose).
- **Bounded required outcome:** A semantic host hierarchy service (sub-object presence per package, world viewpoints with actual pose) shared by the callers above, or the explicit decision that the DCS profile ticks none of them; do not copy solvers into adapters. Animated joint state is unsupported unless a verified call exists.
- **DCS:** status `OPEN`; declared calls: `Unit.getDrawArgumentValue` (Unit.ts:537), `Object.getPosition` (Object.ts:43), `Controller.setTask` (Controller.ts:165), `Controller.pushTask` (Controller.ts:85), `Controller.ControllerAction` (Controller.ts:8).
  - Bounded workaround: per-type device-presence and rigid muzzle/station offsets as versioned host data (the Task52 target-point catalogue pattern), with provenance and supported types stated per type
  - Missing fact: No declared per-sub-object pose; Unit.getDrawArgumentValue is a normalised draw argument. ControllerAction.params has no per-task schema in the bundle (Fire/Attack task ids and fields). The profile rule for Core-ticked weapon systems.
- **Reach evidence (source-derived):** fw_updt.c:182/327, hc_updt.c:314..., aa/ps/rv_updt.c; wn_creat.c:423; HUD callers are player-only. Not proved: which callers a DCS profile reaches.
- **Reuses (exact scope):** R-T50, R-T52, R-D1D2.
- **Open items kept:** hierarchy service; Controller task schemas.

## D3G-09. Campaign-setup model consumers: airfield waypoint routes, troop landing/takeoff/patrol routes, regeneration type

- **Requirements:** 2, 5. **Disposition:** `REQUIRED_OPEN`.
- **Branch/profile:** Campaign boot and task generation (source-derived); the manual-route branch of taskgen is taken only for a keysite with an object index (unmeasured for Lebanon and Georgia).
- **Source chain (function, caller or reader):**
  - `aphavoc/source/ai/faction/popread.c:719` `read_population_airfield_placements ( fp );` - campaign population read; definition :1476, route object :1778, :2789
  - `aphavoc/source/ai/faction/popread.c:4360` `int get_object_3d_troop_landing_route` - model sub-object WAYPOINT_ROUTES (:4390); takeoff :4441; position/heading :4522
  - `aphavoc/source/ai/taskgen/taskgen.c:2123` `get_3d_sub_object_viewpoint(&child, &wp_viewpoint, TRUE);` - create_troop_movement_patrol_task (:2089): patrol waypoints from model sub-objects plus sfrand1 and terrain elevation
  - `aphavoc/source/entity/special/regen/regen.c:216` `enum ENTITY_SUB_TYPE_REGEN get_object_3d_regeneration_type` - regeneration type from RADAR/GROUND_CREW/... sub-objects (callers popread.c:4001/4102/4185)
  - `aphavoc/source/entity/mobile/vehicle/ship/sh_creat.c:295` `OBJECT_3D_SUB_OBJECT_WAYPOINT_ROUTES` - hides route lines on the instance (visual state)
- **Ownership now:** Core reads EECH model data to build campaign geography (landing positions, routes, patrol points, regen type). Ownership split: geography decisions are campaign policy; the data is model-derived and Core-loaded, not host-owned.
- **What the same DLL needs when DCS supplies physical movement and combat:** Airfield and troop route geometry for DCS airbases. DCS has no EECH models; the data must arrive as campaign data or from DCS airbase information.
- **Host contract already present:** None for these model reads (Task50 moved only collision geometry, bounds and radius).
- **Bounded required outcome:** Decide per consumer (landing position/heading, troop landing and takeoff routes, patrol points, regeneration type) whether it is a campaign dataset (per-keysite route table keyed by campaign names, versioned, with provenance) or a host airbase service built on the declared Airbase records under a reviewed mapping rule; measure which keysites of the supported maps actually reach the model-route branches before building either. Do not close the anchored sub-object contracts by analogy.
- **DCS:** status `PARTIAL_DECLARED_SHAPES_ROUTE_AND_SUBOBJECT_CONTRACTS_OPEN`; declared calls: `world.getAirbases` (world.ts:338), `Airbase.getByName` (Airbase.ts:372), `Airbase.getPoint` (Airbase.ts:283), `Airbase.getRunways` (Airbase.ts:213), `AirbaseRunway` (Airbase.ts:15), `Airbase.getParking` (Airbase.ts:204), `AirbaseParkingSpot` (Airbase.ts:6), `Airbase.getDispatcherTowerPos` (Airbase.ts:219), `Airbase.getCategoryEx` (Airbase.ts:338), `land.getClosestPointOnRoads` (land.ts:57), `land.findPathOnRoads` (land.ts:18), `land.getHeight` (land.ts:41).
  - Bounded workaround: per consumer: landing/takeoff position and heading candidates from declared runway and parking records only if a reviewed mapping states the rule; troop landing/takeoff routes, patrol points and regeneration type as campaign-owned versioned data per keysite (not defined)
  - Missing fact: Declared airbase data (Airbase.ts, declared shapes only) holds runways, parking spots, tower position and category, but no route polylines, approach paths or anchored sub-objects, and no declaration says how they correspond to the EECH WAYPOINT_ROUTES, PATROL_ROUTE, RADAR/GROUND_CREW anchors or to carrier/helipad/FARP/parking universality; geometry equivalence is unproven. Which supported keysites reach the model-route branches.
- **Reach evidence (source-derived):** popread.c:719 -> :1476; taskgen.c:2089; regen.c via popread. Not proved: keysites reaching the model branches; whether the campaign data contains manual routes.
- **Reuses (exact scope):** R-T50.
- **Open items kept:** airbase geometry source; route-branch reach per map.

## D3G-10. Rotor damage, blade fragments and smoke attachment (physical services with shared RNG)

- **Requirements:** 2, 6. **Disposition:** `REQUIRED_OPEN`.
- **Branch/profile:** Kill dispatch is registered for helicopters (hc_dstry.c); guards: ALIVE, inst3d, sub-object availability, valid map position (source-derived). Reaching create_rotor_blade_fragment in a supported run is unmeasured.
- **Source chain (function, caller or reader):**
  - `aphavoc/source/entity/mobile/aircraft/helicop/hc_dstry.c:554` `damage_helicopter_3d_object (en);` - kill_local runs it BEFORE ALIVE is cleared (:559)
  - `aphavoc/source/entity/mobile/aircraft/helicop/hc_int.c:159` `damage_helicopter_main_rotors (en, -2);` - typed setter path (tail :295)
  - `aphavoc/source/entity/mobile/aircraft/helicop/hc_anim.c:984` `void damage_helicopter_3d_object (entity *en)` - calls main (:1009) and tail (:1215) damage functions
  - `aphavoc/source/entity/mobile/aircraft/helicop/hc_anim.c:1145` `get_3d_sub_object_viewpoint(&search_main_rotor_blade_section_static, &blade_vp, TRUE);` - blade world geometry; fragment :1149
  - `aphavoc/source/entity/mobile/aircraft/helicop/hc_anim.c:1304` `void create_rotor_blade_fragment (entity *en, viewpoint blade_vp, int main_rotor)` - effect creation using the shared stream (frand1 at ~1387)
  - `aphavoc/source/entity/special/effect/smokelst/sl_meta.c:215` `get_sub_object_relative_position (inst3d, sub_object_type` - smoke attachment relative position
  - `aphavoc/source/entity/en_misc/en_anim/sub_anim.c:169` `int get_sub_object_relative_position` - searches the current model and computes a world point
- **Ownership now:** Core (effects, shared RNG, loss flags); campaign loss consequences stay Core policy; the geometry is Core model state.
- **What the same DLL needs when DCS supplies physical movement and combat:** DCS destroys its own units; the EECH-side fragments and smoke are cosmetic. They still draw from the SHARED campaign stream, so removing them changes later campaign draws unless the count is reproduced or the change is declared.
- **Host contract already present:** None.
- **Bounded required outcome:** Technical-lead decision on shared-RNG accounting for the damage and smoke effect paths (keep the draw count through a host-reported fact, keep Core code for the EECH reference profile only, or an approved declared change), after measuring whether supported flows reach them.
- **DCS:** status `NOT_A_DCS_API_QUESTION`; declared calls: none.
  - Missing fact: Reach and draw counts in supported flows; the RNG-accounting decision.
- **Reach evidence (source-derived):** hc_dstry.c kill dispatch; hc_int.c typed setters; sl_meta.c via smoke-list creation. Not proved: native reach of fragment creation.
- **Reuses (exact scope):** R-T50.
- **Open items kept:** RNG accounting decision; native reach.

## D3G-11. Default shape and ground-radar-clutter columns

- **Requirements:** 2, 5. **Disposition:** `REQUIRED_OPEN`.
- **Branch/profile:** Default-shape resolution runs in every creator when no shape is supplied; the clutter column has readers only in gunship avionics files, co_radar.c and test.c by the text census of aphavoc/source and modules.
- **Source chain (function, caller or reader):**
  - `aphavoc/source/entity/fixed/fx_dbase.c:88` `// default_3d_shape` - fixed_database columns; ground_radar_clutter beside it
  - `aphavoc/source/entity/fixed/object/ob_creat.c:175` `raw->fix.object_3d_shape = fixed_database[raw->fix.sub_type].default_3d_shape;` - creator resolution (cb 173, st 173, su 173, sn 180, sg 181, cr 182; city ct_creat.c:179 commented out in a block comment :175-181)
  - `aphavoc/source/entity/mobile/aircraft/fixwing/fw_creat.c:252` `aircraft_database[raw->ac.mob.sub_type].default_3d_shape` - mobiles likewise (hc 264, aa 247, ps 245, rv 251, sh 242)
  - `aphavoc/source/entity/fixed/fx_int.c:265` `value = fixed_database[raw->sub_type].default_3d_shape;` - typed getter (ac_int.c:501, vh_int.c:450)
  - `aphavoc/source/entity/mobile/weapon/wn_creat.c:805` `INT_TYPE_DEFAULT_3D_SHAPE` - weapon launch bounding box (:1007)
  - `aphavoc/source/entity/fixed/fx_int.c:297` `value = fixed_database[raw->sub_type].ground_radar_clutter;` - getter of the clutter column
  - `aphavoc/source/gunships/avionics/common/co_radar.c:778` `INT_TYPE_GROUND_RADAR_CLUTTER` - readers: 11 gunship MFD files, co_radar.c (:778/:958/:1707), test.c:2155 (sources.txt:908)
- **Ownership now:** Campaign data tables (EECH sub type -> EECH scene). The default shape is the INPUT of the host fixed state's shape word, tallest height and collision bounds; it is not declared campaign-owned or Host-owned by any closed scope.
- **What the same DLL needs when DCS supplies physical movement and combat:** Default shape: only as the EECH-side key of host statements; a DCS host needs a typeName-to-scene mapping table, not a shape. Clutter: nothing for DCS (player radar belongs to DCS's cockpit).
- **Host contract already present:** Fixed-state statements carry the shape word (finite fixed scope). Nothing for the clutter column.
- **Bounded required outcome:** State default shape as campaign-owned data with a versioned DCS typeName mapping (bounded per-type table with provenance) or as a Host fact; the clutter column is dispositioned by the final compile census (readers are gunship avionics and test.c by text census), not dormant by name.
- **DCS:** status `OPEN_WORKAROUND_NAMED`; declared calls: `Object.getTypeName` (Object.ts:52), `Unit.getDescByName` (Unit.ts:133), `StaticObject.getDesc` (StaticObject.ts:301).
  - Bounded workaround: versioned per-type mapping DCS typeName -> EECH scene index with provenance, supported types and named unsupported types
  - Missing fact: The mapping data itself and its provenance. StaticObjectDesc declares no box (box only in a doc example); UnitDesc.box is static bounds, not a mesh.
- **Reach evidence (source-derived text census):** creators and getters above; player-avionics readers of the clutter column. Not proved: headless reach of the clutter readers; no compile/symbol census.
- **Reuses (exact scope):** R-FINITE, R-T50.
- **Open items kept:** default-shape ownership; clutter final census.

## D3G-12. Six compiled draw files reading fixed_database[].align_with_terrain

- **Requirements:** 1, 2. **Disposition:** `CENSUS_OPEN_SOURCE_DERIVED_UNREACHED`.
- **Branch/profile:** Seven direct reads in six files; each then calls get_3d_terrain_face_normal, which writes the shared terrain record. All six files are in sources.txt (43-131).
- **Source chain (function, caller or reader):**
  - `aphavoc/source/entity/fixed/cbuild/cb_draw.c:108` `fixed_database[raw->fix.sub_type].align_with_terrain` - draw_local_city_building_object (:79) <- ct_draw.c:147
  - `aphavoc/source/entity/fixed/object/ob_draw.c:98` `fixed_database[raw->fix.sub_type].align_with_terrain` - registered ob_draw.c:224
  - `aphavoc/source/entity/fixed/pylon/py_draw.c:175` `align_with_terrain` - and :328; registered :398
  - `aphavoc/source/entity/fixed/scenic/sn_draw.c:101` `align_with_terrain` - registered :157
  - `aphavoc/source/entity/fixed/site/st_draw.c:102` `align_with_terrain` - registered :147
  - `aphavoc/source/entity/fixed/siteupdt/su_draw.c:110` `align_with_terrain` - registered :155
  - `aphavoc/source/entity/system/en_funcs/en_draw.c:222` `draw_local_entity_3d_object (en, range);` - scan_local_entity_3d_objects (:115): the only dispatcher
  - `aphavoc/source/app3d/3d_draw.c:293` `scan_local_entity_3d_objects (&main_vp);` - draw_application_3d_scene (:196); also :557, :616, :702
  - `eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c:381` `static void frame (uint32_t milliseconds)` - headless frame: update_client_server_entities and update_update_functions only (:403-405), comment 'without drawing'
- **Ownership now:** Renderer code reading a Core data table; no campaign decision consumes the draw result.
- **What the same DLL needs when DCS supplies physical movement and combat:** Nothing: the draw functions are dispatched only from the application draw scene, which the headless entry does not call and which a DCS host never calls. Residual: process_events (modules/system/event.c:509) dispatch was not enumerated.
- **Host contract already present:** Not applicable.
- **Bounded required outcome:** Final compile/link and entry census shows no path from the DCS-profile entry points to scan_local_entity_3d_objects; until then 'source-derived unreachable in the headless frame', not dormant. The default-shape and clutter columns stay in D3G-11.
- **DCS:** status `NOT_REQUIRED_PENDING_FINAL_CENSUS`; declared calls: none.
  - Missing fact: process_events dispatch enumeration and the final compile/symbol census.
- **Reach evidence (source-derived entry/dispatch/compile chain):** draw table en_draw.c:247 (default :270) <- six overload_* registrations <- scan_local_entity_3d_objects <- 3d_draw.c (4 sites) <- app3d draw; headless frame excludes them (eech_engine.c:381-418). Not proved: event-dispatch reach; measured census; a name or one unreached run is not claimed as dormancy.
- **Reuses (exact scope):** R-FINITE.
- **Open items kept:** fixed-draw-align-direct-reads-final-census.

## D3G-13. Natural moving and public authority

- **Requirements:** 2, 6, 7. **Disposition:** `BLOCKED_BY_ENV_ENTITY_POSE_LONG_RUN_MOTION_1`.
- **Branch/profile:** Public finite scopes cover stationary candidates, configured moving queries and the person-motion correction; ENV-ENTITY-POSE-LONG-RUN-MOTION-1 (blocks_cutover=true) is open until the corrected full ordinary pack completes with independent review (f734, running).
- **Source chain (function, caller or reader):**
  - `aphavoc/source/entity/mobile/mb_hostoffer.h:193` `case CAMPAIGN_MODEL_CANDIDATE_NEED_UPDATED:` - moving candidate stage; sweep needs UPDATED (D3G-03)
  - `aphavoc/source/entity/mobile/mb_hostscan.h:164` `sector_tallest_structure_height = CAMPAIGN_SCAN_TALLEST (sector);` - scan over the host's roster order
- **Ownership now:** Host Movers record owns mobile poses; Core mirror is a conformance assertion (entity_pose.mirror). No legitimately reached MOVING candidate has been observed on the public path.
- **What the same DLL needs when DCS supplies physical movement and combat:** Poses of real campaign movers sampled from DCS (Object.getPosition/getVelocity) feeding the same candidate, LOS and arrival decisions.
- **Host contract already present:** ENTITY_POSE and the Movers record; DCS side of pose sampling is declared (Object.getPoint/getPosition/getVelocity) but its coordinate/basis conversion needs an actual DCS check (M6).
- **Bounded required outcome:** After the blocker closes: one witness driven by actual campaign engagement and host callbacks that reaches a moving candidate (candidate_need_updated and moving counters positive), plus disposition of every unreached state (no identity, unrecorded, retired, wreck).
- **DCS:** status `DECLARED_SHAPES_PRESENT_BEHAVIOUR_UNVERIFIED`; declared calls: `Object.getPoint` (Object.ts:34), `Object.getPosition` (Object.ts:43), `Object.getVelocity` (Object.ts:61), `Object.isExist` (Object.ts:79), `Object.getTypeName` (Object.ts:52).
  - Missing fact: Coordinate/basis conversion and the UPDATED truth value (D3G-03); live behaviour is M6.
- **Reach evidence (reported (card)):** entity-host-sourced-live-pose blocker text. Not proved: moving public path, natural coverage.
- **Reuses (exact scope):** R-FINITE.
- **Open items kept:** ENV-ENTITY-POSE-LONG-RUN-MOTION-1; entity-host-sourced-live-pose.

## D3G-14. Final active-consumer and compile census: player, view, HUD, cockpit, camera, UI and gunship dynamics

- **Requirements:** 2, 9. **Disposition:** `CENSUS_OPEN`.
- **Branch/profile:** Everything under gunships/, views and ui_menu is compiled into the DLL (sources.txt). Source suggests player-only guards (get_gunship_entity, draw scene entry); nothing is measured.
- **Source chain (function, caller or reader):**
  - `aphavoc/source/entity/mobile/aircraft/helicop/hc_updt.c:382` `update_flight_dynamics ();` - player-gunship branch (AUTO_PILOT and EJECTED guards); also :741
  - `aphavoc/source/entity/mobile/aircraft/helicop/helicop.c:627` `set_gunship_entity (en);` - gunship assignment lives in the player selection flow (:598-627), not in the headless boot
  - `eech-campaign/crates/eech-engine-sys/sources.txt:864` `aphavoc/source/gunships/dynamics/common/co_force.c` - compiled
  - `eech-campaign/crates/eech-engine-sys/sources.txt:691` `aphavoc/source/gunships/avionics/common/co_radar.c` - compiled
- **Ownership now:** Reference-game player code compiled transitionally; not a campaign decision owner.
- **What the same DLL needs when DCS supplies physical movement and combat:** Nothing from DCS: the player aircraft and cockpit are DCS's. The DLL still contains this code; the question is only whether any DCS-profile entry reaches it.
- **Host contract already present:** Not applicable.
- **Bounded required outcome:** Final actual compiled-consumer audit after the last code change: link map or symbol census plus entry/dispatch analysis for the DCS-profile entry points (boot, frame, host callbacks, save/restore); every residual compiled consumer named with a disposition. Not an exhaustive measured census here.
- **DCS:** status `NOT_REQUIRED_PENDING_FINAL_CENSUS`; declared calls: none.
  - Missing fact: The final actual compiled-consumer audit.
- **Reach evidence (source-derived):** hc_updt.c:382/741; helicop.c:627; sources.txt. Not proved: dispatch reach, symbol census.
- **Open items kept:** final active-consumer/compile census.

## D3G-15. Accepted scopes reused by exact proof (not reopened)

- **Requirements:** 1, 2, 3, 4, 5. **Disposition:** `ACCEPTED_REUSE`.
- **Branch/profile:** See accepted_reuse in this file for each scope's proof and what it does not cover.
- **Ownership now:** Host (reference) for terrain, ground contact, environment/time, collision-model geometry and target points; campaign geography, timers and scheduling stay Core.
- **What the same DLL needs when DCS supplies physical movement and combat:** As in the referenced accepted DCS mapping documents (terrain README Mapping to DCS, d3-environment-time-DCS-mapping.md, d3-model-target-point-DCS-mapping.md, dcs-api/MAPPING-NOTES.md).
- **Host contract already present:** Delivered and accepted per the scope index; the final audit must still join accepted review/merge/full40all8/input/source evidence.
- **Bounded required outcome:** None here beyond the final audit.
- **DCS:** status `ACCEPTED_REUSE_PER_SCOPE`; declared calls: `land.getHeight` (land.ts:41), `land.getSurfaceType` (land.ts:97), `land.getIP` (land.ts:73), `land.isVisible` (land.ts:111), `atmosphere.getWind` (atmosphere.ts:25), `timer.getTime` (timer.ts:112), `Object.getPosition` (Object.ts:43), `Unit.getDesc` (Unit.ts:482).
  - Bounded workaround: as recorded in those documents
  - Missing fact: Their own qualifications stand (for example weather mapping rows, normal workaround, live behaviour at M6).
- **Reach evidence (reported accepted evidence):** n/a. Not proved: n/a.
- **Reuses (exact scope):** R-D1D2, R-T47, R-T48, R-T51, R-T50, R-T52, R-FINITE.

## Accepted scopes reused (not reopened)

| id | scope | proof | does not cover |
|---|---|---|---|
| R-D1D2 | D1 and D2 (aim, LOS, sensor, weapon hosted steps; accepted M5 predecessor epics) | accepted per D3-ACCEPTED-SCOPE-INDEX-20261106.json and D3-CURRENT.json accepted.D1_D2 | D3 terrain/model/environment answers behind those steps; they were campaign-answered lookups before D3 |
| R-T47 | EEDCS-47 host terrain point, fast-elevation and map-fact services | Done PR107 b6575594c114cf38c0206af24920e40f0e250b77; eech-campaign/reference/d3-terrain-characterization/normal-delivery-b6575594c; symbol-ownership-251bfe15b.json | the Core shared value cache system_point_data (mb_hostterrain.c), the transitional loader and legacy grid generation, off-map, fast-grid generation, DCS face normal |
| R-T48 | EEDCS-48 ground-contact and ground-impact predicates | Done PR108 3fd; accepted40/all8 (D3-CURRENT.json accepted.Task48) | object, tree or entity collision |
| R-T51 | EEDCS-51 weather, wind and environment time inputs | Done PR111 a42d86e758812d7aed75c1bb5f8dc09d4aea1544; eech-campaign/docs/d3-environment-time-DCS-mapping.md | terrain, objects, model hierarchy; the DCS weather row mapping qualifications stay in that document |
| R-T50 | EEDCS-50 collision-model loading, triangle decoding, bounds and radius | Done PR110 normal merge 873a172d9d26d0f2e437aa491187ea3cc3ef6cb2; Task50-actual-delivery-873a172d9-20261105.json | model hierarchy, sub-objects, viewpoints, devices, muzzles, rotor and smoke services |
| R-T52 | EEDCS-52 model target-point offsets and the live target point | Done PR112 normal merge 84775522f9eb9c7182ff89f75387115b9e3883fd; Task52-actual-delivery-84775522f-20261105.json; eech-campaign/docs/d3-model-target-point-DCS-mapping.md | device, muzzle, station, rotor-damage and smoke-attachment consumers; the gd_attck.c direct offset read stays Core policy |
| R-FINITE | Finite closed Task49 scopes: stationary entity handoff, candidate, scan, narrow phase, TREE collision, sector roster, city own state/list/box/lifecycle, fixed physical rules, constructor math, configured moving | closed_evidence of D3-CURRENT.json and the reference/d3-entity-collision and reference/d3-tree-collision-production finite-authority manifests | all entity types, natural moving, sector height, UPDATED/shared terrain, loader RNG, hierarchy, whole Task49 or D3 |

## Declared DCS shapes used (pinned `1179554f`)

| call | file:line | note |
|---|---|---|
| `land.getHeight` | land.ts:41 |  |
| `land.getSurfaceHeightWithSeabed` | land.ts:87 |  |
| `land.getSurfaceType` | land.ts:97 |  |
| `land.isVisible` | land.ts:111 | doc line says terrain-only: buildings and other objects do not block (land.ts:103) |
| `land.profile` | land.ts:122 |  |
| `land.getIP` | land.ts:73 |  |
| `land.getClosestPointOnRoads` | land.ts:57 |  |
| `land.findPathOnRoads` | land.ts:18 |  |
| `atmosphere.getWind` | atmosphere.ts:25 |  |
| `atmosphere.getWindWithTurbulence` | atmosphere.ts:32 | doc: zero vector at or below terrain (atmosphere.ts:28-30) |
| `atmosphere.getTemperatureAndPressure` | atmosphere.ts:13 |  |
| `timer.getTime` | timer.ts:112 |  |
| `timer.getAbsTime` | timer.ts:42 |  |
| `timer.getPause` | timer.ts:27 |  |
| `timer.scheduleFunction` | timer.ts:86 |  |
| `world.searchObjects` | world.ts:285 | category is a bare number or number[]; the Object.Category values are NOT in the bundle |
| `world.WorldVolume` | world.ts:7 | id: number and opaque params only; the volume-type constants and param schemas are NOT in the bundle |
| `world.getAirbases` | world.ts:338 | returns l_Airbase[]; Airbase.ts is the later additional file (Root addition, blob d3d7dedd8), not one of the original 8 |
| `Object.getPoint` | Object.ts:34 |  |
| `Object.getPosition` | Object.ts:43 |  |
| `Object.getVelocity` | Object.ts:61 |  |
| `Object.getTypeName` | Object.ts:52 |  |
| `Object.getCategory` | Object.ts:135 |  |
| `Object.isExist` | Object.ts:79 |  |
| `Unit.getDesc` | Unit.ts:482 |  |
| `Unit.getDescByName` | Unit.ts:133 |  |
| `UnitDesc.box` | Unit.ts:115 | static descriptor data, not a mesh or normal oracle |
| `Unit.getDrawArgumentValue` | Unit.ts:537 | a normalised draw argument, not a joint pose |
| `StaticObject.getDesc` | StaticObject.ts:301 |  |
| `StaticObjectDesc` | StaticObject.ts:26 | the declared type (StaticObject.ts:26-51) has NO box field; a box appears only in the doc example (StaticObject.ts:6-24) |
| `Airbase.getRunways` | Airbase.ts:213 | runway records only |
| `AirbaseRunway` | Airbase.ts:15 | course, Name, position, length, width |
| `Airbase.getParking` | Airbase.ts:204 | parking spot records; not a route or sub-object anchor |
| `AirbaseParkingSpot` | Airbase.ts:6 | Term_Index, vTerminalPos, TO_AC, Term_Index_0, Term_Type, fDistToRW |
| `Airbase.getDispatcherTowerPos` | Airbase.ts:219 |  |
| `Airbase.getPoint` | Airbase.ts:283 | reference point; follows a moving ship |
| `Airbase.getCategoryEx` | Airbase.ts:338 | airdrome, helipad/FARP or ship; Airbase.Category values not declared in the file |
| `Airbase.getByName` | Airbase.ts:372 |  |
| `AirbaseDesc` | Airbase.ts:23 | life, attributes, category, typeName, displayName: no geometry |
| `Controller.ControllerAction` | Controller.ts:8 | id plus params: Record<string, unknown>; each task id's schema is not declared here |
| `Controller.setTask` | Controller.ts:165 |  |
| `Controller.pushTask` | Controller.ts:85 |  |
| `Controller.setCommand` | Controller.ts:120 |  |
| `Controller.setSpeed` | Controller.ts:155 |  |
| `Controller.getDetectedTargets` | Controller.ts:26 |  |

Verified bundle files (Git blob, SHA-256 and URL) are listed in the matrix `api_bundle.files`. Declared absences within the bundle (not exhaustive):

- land.ts declares no terrain-normal call (not an exhaustive absence proof: other files of the repository were not read).
- No file of the bundle declares a tree, forest or scenery query; `Object.Category` and the `world` volume-type constants live in files not fetched (exports/*.export.ts).
- Object.ts, Unit.ts and StaticObject.ts declare no collision mesh, hit/intercept/normal, named child pose or sub-object transform.
- Airbase.ts (the later additional file) declares runway records, parking spots, a tower position, category and a point, but no landing or takeoff route polylines, approach paths or named anchored sub-objects; it does not establish carrier, helipad or parking universality, geometry equivalence or live behaviour.
- The per-task parameter schemas behind ControllerAction are not in the bundle; Airbase.Category, Object.Category and the volume constants are not declared in the files read.

## Next physical moves (high level, gated; not a backlog)

1. Host-owned per-sector high-water scalar with restore acknowledgement and the clearance read (D3G-01), after ENV-ENTITY-POSE-LONG-RUN-MOTION-1 closes by complete corrected full pack plus independent review. Gate: technical-lead decision on the DCS clearance source (D3G-01 DCS row) before any DCS claim.
2. Boot CRT census and loader removal decision for hosted profiles (D3G-04/05): one bounded capture with the existing counting header, then remove or approve. Gate: measured draw order; baseline-approved if any draw is consumed.
3. UPDATED/shared-state host statements for the three decisions (D3G-03), including the memset-domain measurement. Gate: lead decision on the truth value under a DCS-sampled pose.
4. Hierarchy facts: device presence, station/muzzle viewpoints, campaign-setup routes and regen type, rotor/smoke RNG decision (D3G-08/09/10), only for callers the DCS profile still ticks. Gate: weapon-entity ownership rule of the DCS profile; Airbase and Controller task declarations.
5. Final active-consumer/compile census and whole-D3 audit after the last code change (D3G-12/14, accepted-scope join). Gate: after final code; current final 40/all8/inputs/CI/review/normal merge.

## Primary files that would settle OPEN rows

- `flying-dice/tslua-dcs` @ `1179554f`: `packages/tslua-dcs-mission-types/src/exports/world.export.ts` - world volume-type constants and params schema; Object.Category constants (a sibling exports/Object.export.ts) so world.searchObjects category/volume can be named exactly (D3G-01, 06, 07; Airbase.ts names Object.Category.BASE without declaring the values)
- `flying-dice/tslua-dcs` @ `1179554f`: `UNKNOWN: directory listing of packages/tslua-dcs-mission-types/src and the file declaring Controller task/command ids and params` - per-task parameter schemas behind ControllerAction (attack/fire task ids) for D3G-08

## Limits

- Documentation and source disposition only: no compiler, test, native, Windows, war, physical change; physical_cutover_clearance remains false.
- Source anchors are resolved from the clean tree at the recorded HEAD; inclusion/reachability statements are source/build-derived and labelled, not an exhaustive measured preprocessed/symbol/runtime census.
- DCS declarations are declared shapes of the pinned author-maintained commit, not measured live DCS contracts, exhaustive absence, universal bounds or collision-mesh guarantees; no live DCS/M6 execution is claimed.
- OPEN rows stay open and reduce no D3 scope; accepted scopes are reused at their stated scope only.
- ENV-ENTITY-POSE-LONG-RUN-MOTION-1 stays OPEN/blocks_cutover=true; the four original f734 natives are untouched.
- Reported evidence (Task52 CRT capture, card blocker texts, accepted-scope proofs) is cited as reported and not reproduced here.
- The original 8-file bundle manifest is unchanged; Airbase.ts is a later optional addition from Root with its own provenance file and hash.

Preparation counts 2 of 2 on completion of this audit. No further preparation batch starts until the production blocker is resolved by an actual complete corrected pack and independent review. Whole D3 and Task49 remain open; no D4/D5/TypeScript.
