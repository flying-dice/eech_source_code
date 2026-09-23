# Port manifest

This is the authoritative mapping from EECH C to `eech-core-ts`. A module is
only marked **ported** when all of it is. Otherwise the status is **partial** and
the functions below say what has moved.

Function statuses:

- `unported`
- `ported`
- `tested`
- `100%-covered`
- `C-reference-verified`: executed from the original C by `c-reference/` and
  compared with the TS
- `source-read`: verified against the C by reading only (not executed)
- `blocked-engine-boundary`
- `excluded-physical-simulation`

A slice is **frozen** when every function in it is `100%-covered`, has
source-derived expectations, runs under Lua 5.1, and is `C-reference-verified`
where practical.

## Modules

| C source | TS target | Module status |
|---|---|---|
| `aphavoc/source/entity/special/group/group.c` | `src/entity/special/group/group.ts` | partial |
| `aphavoc/source/entity/special/group/gp_int.c` | `src/entity/special/group/group.ts` (`overloadGroupFunctions`) | partial |
| `aphavoc/source/entity/special/group/gp_float.c` | `src/entity/special/group/group.ts` | partial |
| `aphavoc/source/entity/special/group/gp_ptr.c` | `src/entity/special/group/group.ts` | partial |
| `aphavoc/source/entity/special/group/gp_vec3d.c` | `src/entity/special/group/group.ts` | partial |
| `aphavoc/source/entity/special/group/gp_list.c` | `src/entity/special/group/group.ts` | partial |
| `aphavoc/source/entity/special/group/gp_updt.c` | `src/entity/special/group/group.ts` (`updateServer`) | ported |
| `aphavoc/source/entity/special/group/gp_msgs.c` | `src/entity/special/group/group.ts` (link/unlink parent responses; link child, slice 5b) | partial |
| `aphavoc/source/entity/special/update/up_update.c` | `src/entity/special/update/update.ts` | partial (tacview and mobile bitsets excluded) |
| `aphavoc/source/entity/special/update/up_msgs.c` | `src/entity/special/update/update.ts` | ported |
| `aphavoc/source/entity/special/update/up_list.c` | `src/entity/special/update/update.ts` | ported |
| `aphavoc/source/entity/system/en_funcs/en_updt.c` | `src/entity/system/en_updt.ts` | ported (default handler deliberately not installed) |
| `modules/system/time.c` | `src/core/time.ts`, `src/ports/clock.ts` (port) | partial (measurement is the `Clock` port; delta history not ported) |
| `aphavoc/source/cmndline.c` (`command_line_entity_update_frame_rate`) | `src/core/cmndline.ts` | partial |
| `aphavoc/source/entity/special/group/gp_dbase.c` | `src/generated/c-group-database.ts` (generated) | partial (resupply source; movement type, landing type, engage enemy and AI statistics, slice 5b; minimum idle count, registry list type, default entity type and default aircraft, slice 6a) |
| `aphavoc/source/entity/special/keysite/ks_dbase.c` (`default_supply_usage` ammo and fuel; `air_force_capacity`, slice 5b) | `src/generated/c-keysite-database.ts` (generated) | partial (compiled defaults; `wutcfg.c` overrides not ported) |
| `aphavoc/source/global.c`, `global.h` (`game_status`, `set_game_status`, `get_game_status`) | `src/core/game-status.ts` | partial (`game_status_string` not ported: no campaign reader) |
| `aphavoc/source/ui_menu/gametype/gametype.c`, `gametype.h` (`game_type`, `get_game_type`) | `src/core/game-type.ts` | partial (the front end's assignments are `setGameType`) |
| `aphavoc/source/ui_menu/ingame/campaign/ca_msgs.c` (`notify_campaign_screen`) | `src/ui_menu/campaign/ca_msgs.ts`, `src/ports/campaign-events.ts` (port) | partial (the guard is core; the screen's responses are the `CampaignEvents` port; `MISSION_CREATED` only) |
| `modules/3d/3dobjvis.c :: get_object_3d_bounding_box`, `objects.h :: struct OBJECT_3D_BOUNDS` | `src/ports/object-3d-metadata.ts` (port) | blocked-engine-boundary (the 3D object database) |
| `modules/3d/3dmodels.h` (`OBJECT_3D_SINGLE_CRATE`) | `src/generated/c-constants.ts` (generated) | partial (the indices the port names) |
| `aphavoc/source/entity/special/keysite/keysite.c` | `src/entity/special/keysite/keysite.ts` | partial |
| `aphavoc/source/entity/special/keysite/ks_int.c`, `ks_float.c`, `ks_vec3d.c`, `ks_list.c`, `ks_msgs.c` | `src/entity/special/keysite/keysite.ts` | partial |
| `aphavoc/source/entity/special/force/force.c` | `src/entity/special/force/force.ts` | partial |
| `aphavoc/source/entity/special/force/fc_int.c`, `fc_list.c` | `src/entity/special/force/force.ts` | partial |
| `aphavoc/source/entity/special/force/fc_msgs.c` | `src/entity/special/force/fc_msgs.ts` | partial (`response_to_force_low_on_supplies`, slice 5a) |
| `aphavoc/source/entity/special/task/task.c` (`entity_is_object_of_task`, slice 5a; `get_local_task_list_type`, `find_most_suitable_keysite_for_task`, `assess_task_difficulty`, `assess_task_sector_difficulty`, slice 5b), `ts_int.c`, `ts_float.c`, `ts_ptr.c`, `ts_list.c` | `src/entity/special/task/task.ts` | partial (the values, pointers and lists task construction sets and reads) |
| `aphavoc/source/entity/special/task/ts_creat.c` | `src/entity/special/task/ts_creat.ts` | partial (server model; the client model is not ported) |
| `aphavoc/source/entity/special/task/ts_msgs.c` (`response_to_link_parent`) | `src/entity/special/task/task.ts` | partial (the unassigned arm; assigned and completed throw unported) |
| `aphavoc/source/entity/special/task/ts_dbase.c` (`task_priority`, slice 5a; `primary_task`, `engage_enemy`, `movement_type`, `keysite_air_force_capacity`, `landing_types`, `ai_stats`, slice 5b) | `src/generated/c-task-database.ts` (generated) | partial |
| `aphavoc/source/entity/special/waypoint/wp_int.c`, `wp_list.c` | `src/entity/special/waypoint/waypoint.ts` | partial (a restored waypoint's sub type and `task_dependent_link`, slice 5a) |
| `aphavoc/source/ai/taskgen/taskgen.c` (`create_supply_task`, `create_task`, `get_task_start_keysite`, `validate_task_generation`, `terminator_point`) | `src/ai/taskgen/taskgen.ts` | partial (slice 5b; the other task generators are not ported) |
| `aphavoc/source/ai/taskgen/assign.c` (`assign_keysite_tasks`, `suitable_group_task_specific_checks`, `get_suitable_registered_group`, `check_group_members_awake`; `assign_primary_task_to_group` as the boundary) | `src/ai/taskgen/assign.ts` | partial (slice 6a: the decision; the transaction is not ported) |
| `aphavoc/source/entity/en_misc/en_misc.c` (`qs`, `quicksort_entity_list`) | `src/entity/en_misc/en_misc.ts` | partial (slice 6a) |
| `aphavoc/source/entity/mobile/aircraft/ac_float.c` (`CRUISE_VELOCITY`; the default `SLEEP`) | `src/entity/mobile/aircraft/ac_float.ts` | partial (slice 6a) |
| `aphavoc/source/entity/mobile/aircraft/ac_dbase.c` (`cruise_velocity`) | `src/generated/c-aircraft-database.ts` (generated) | partial (slice 6a) |
| `aphavoc/source/entity/special/pilot/pi_list.c` (`pilot_lock_root`) | `src/entity/special/pilot/pilot.ts` | partial (slice 6a) |
| `aphavoc/source/ai/highlevl/suitable.c` | `src/ai/highlevl/suitable.ts` | ported (`deinitialise_group_task_array` is the next initialisation's reset) |
| `aphavoc/source/entity/special/session/session.h`, `ss_list.c` | `src/entity/special/session/session.ts`, `src/entity/system/entity.ts` | partial |
| `aphavoc/source/entity/special/guide/gd_list.c` | `src/entity/special/guide/guide.ts` | partial |
| `aphavoc/source/entity/mobile/aircraft/ac_list.c`, `ac_vec3d.c`; `vehicle/vh_list.c`, `vh_vec3d.c` | `src/entity/mobile/mobile.ts` | partial (campaign surface only) |
| `aphavoc/source/entity/mobile/mb_int.c`, `mb_vec3d.c`, `mb_list.c` | `src/entity/mobile/mobile.ts` (`overloadMobileRawStateFunctions`) | partial (the rows cargo reaches) |
| `aphavoc/source/entity/mobile/aircraft/ac_msgs.c` | `src/entity/mobile/aircraft/ac_msgs.ts` | partial (link / unlink parent responses) |
| `aphavoc/source/entity/mobile/cargo/cg_creat.c`, `cg_dstry.c`, `cg_list.c`, `cg_funcs.c`, `cg_msgs.c` | `src/entity/mobile/cargo/cargo.ts` | partial (kill, movement, update, draw, pack and `cg_int.c` not ported) |
| `aphavoc/source/entity/special/sector/sector.c`, `sc_seccreat.c`, `sc_int.c`, `sc_list.c`, `sc_msgs.c` | `src/entity/special/sector/sector.ts` | partial (slice 5b adds sector side and the enemy surface-to-air defence level) |
| `aphavoc/source/entity/special/effect/soundeff/soundeff.c` | `src/entity/special/effect/soundeff.ts` | partial (`destroy_client_server_sound_effects`) |
| `aphavoc/source/entity/mobile/**` (flight models, movement, weapons, damage, drawing) | none | excluded-physical-simulation |
| `aphavoc/source/entity/system/en_funcs/en_list.c`, `en_list/*.h` | `src/entity/system/en_list.ts` | partial |
| `aphavoc/source/entity/system/en_funcs/en_int.c`, `en_float.c`, `en_vec3d.c`, `en_ptr.c` | `src/entity/system/en_values.ts`, `function-table.ts` | partial |
| `aphavoc/source/entity/system/en_msgs/en_msgs.c` | `src/entity/system/en_msgs.ts` | partial |
| `aphavoc/source/entity/system/en_main/en_heap.c`, `en_heap.h` | `src/entity/system/en_heap.ts`, `entity.ts` | partial (downwash heap and packing not ported) |
| `aphavoc/source/entity/system/en_main/en_world.c`, `en_world.h`; `misc/miscell.c :: int_bit_count` | `src/entity/system/en_world.ts` | partial (slice 5b adds `point_inside_map_volume`, `MAP_PERIMETER_SIZE`, `bound_position_to_adjusted_map_area`) |
| `aphavoc/source/entity/system/en_main/*` (other files) | `src/entity/system/entity.ts` | partial |
| `aphavoc/source/entity/system/en_attrs/en_attrs.c` | `src/entity/system/en_attrs.ts` | partial (`set_local_entity_attributes`; pack/unpack not ported) |
| `aphavoc/source/entity/system/en_funcs/en_creat.c`; `en_debug/en_valid.c` (create index checks) | `src/entity/system/en_creat.ts` | partial |
| `aphavoc/source/entity/system/en_funcs/en_dstry.c` | `src/entity/system/en_dstry.ts` | partial (kill and whole-heap destruction not ported) |
| `modules/system/fpu.c :: convert_float_to_int` | `toCInt` (`src/core/cint.ts`) | ported (truncation: EECH's round-toward-zero FPU mode) |
| `aphavoc/source/entity/system/en_comms/en_comms.c` | `src/entity/system/en_comms.ts` (the single player trap; `pack_vec3d`'s position check for task pointers), `src/ports/entity-replication.ts` (port) | partial: the transport is blocked-engine-boundary |
| `aphavoc/source/comms/comms.c` (`get_comms_model`) | `src/entity/system/comms.ts` | partial |
| `modules/maths/range.c` | `src/core/maths/range.ts` | partial |
| `modules/maths/miscmath.h`, `constant.h`, `vector.h` | `src/core/maths/miscmath.ts`, `vec3d.ts` | partial |
| `modules/maths/vector.c` (`normalise_any_3d_vector`) | `src/core/maths/vector.ts` | partial |
| `aphavoc/source/entity/system/en_types/en_suply.h` (`FUEL_USAGE_ACCELERATOR`, `AMMO_USAGE_ACCELERATOR`, `KEYSITE_SUPPLY_REQUEST_THRESHOLD`); `cargo.h` (`CARGO_AMMO_SIZE`, `CARGO_FUEL_SIZE`) | `src/generated/c-constants.ts` (generated) | partial |
| `modules/system/assert.h` | `src/core/assert.ts` | partial |
| enum headers (`en_types.h`, `en_side.h`, `en_list.h`, `en_int.h`, `en_float.h`, `en_vec3d.h`, `en_ptr.h`, `en_msgs.h`, `en_sbtyp.h`, `ai_extrn.h`, `comms.h`, `en_suply.h`) | `src/generated/c-enums.ts` (generated) | ported (selected enums, generated verbatim) |
| `ai/highlevl/*` (except `suitable.c`), `ai/taskgen/*` (except the functions above), `ai/frontl/*`, `ai/faction/*`, `ai/ai_misc/*` | none | unported |
| `entity/special/division`, `landing`, `regen`; the rest of `task` and `waypoint` | none | unported |
| `wutcfg.c`, `gwutcfg.c` (runtime overrides of `group_database` and `keysite_database`) | none | unported |

## Functions

### Frozen slice: supply assessment

| C function | TS | Status |
|---|---|---|
| `group.c :: assess_group_supplies` | `assessGroupSupplies` | ported, tested, 100%-covered, C-reference-verified |
| `keysite.c :: get_closest_keysite` | `getClosestKeysite` | ported, tested, 100%-covered, C-reference-verified |
| `force.c :: get_local_force_entity` | `getLocalForceEntity` | ported, tested, 100%-covered, C-reference-verified |
| `range.c :: get_2d_range` | `get2dRange` | ported, tested, 100%-covered, C-reference-verified (bit exact) |
| `range.c :: get_approx_2d_range` | `getApprox2dRange` | ported, tested, 100%-covered, C-reference-verified (bit exact) |
| `en_msgs.c :: notify_local_entity` | `notifyLocalEntity` | ported, tested, 100%-covered, C-reference-verified |
| `miscmath.h :: bound` | `bound` | ported, tested, 100%-covered, C-reference-verified (through the slice) |

### Frozen slice: group update timing (slice 2)

| C function | TS | Status |
|---|---|---|
| `gp_updt.c :: update_server`, `overload_group_update_functions` | `updateServer` | ported, tested, 100%-covered, C-reference-verified |
| `gp_float.c :: set_local_float_value` (`FLOAT_TYPE_SLEEP`, `FLOAT_TYPE_ASSIST_TIMER`), `set_server_float_value` | `setLocalTimerValue` via `serverFloatValueSetter` | ported, tested, 100%-covered, C-reference-verified |
| `gp_float.c :: get_local_float_value` (`FLOAT_TYPE_SLEEP`, `FLOAT_TYPE_ASSIST_TIMER`) | `overloadGroupFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `up_update.c :: update_client_server_entities` | `updateClientServerEntities` | ported, tested, 100%-covered, C-reference-verified (tacview excluded) |
| `up_update.c :: set_entity_update_frame_rate` | `setEntityUpdateFrameRate` | ported, tested, 100%-covered, C-reference-verified |
| `up_update.c :: set_update_entity`, `get_update_entity`, `get/set_update_succ` | `update.ts` | ported, tested, 100%-covered |
| `up_msgs.c :: response_to_unlink_child` | `responseToUnlinkChild` | ported, tested, 100%-covered, C-reference-verified (successor fix-up branch: unit test only; no ported update function unlinks its successor) |
| `up_msgs.c` C-default link responses (`LINK_CHILD`, `LINK_PARENT`, `UNLINK_PARENT`) | `defaultMessageResponse` | ported, tested, 100%-covered, C-reference-verified |
| `gp_msgs.c :: response_to_link_parent` | `responseToLinkParent` | partial: the `LIST_TYPE_DIVISION` case (`set_local_division_name`) is unported and throws |
| `gp_msgs.c :: response_to_unlink_parent` | `responseToUnlinkParent` | ported, tested, 100%-covered, C-reference-verified |
| `en_list.c :: insert_local_entity_into_parents_child_list` | `insertLocalEntityIntoParentsChildList` | ported, tested, 100%-covered, C-reference-verified (the `#ifdef DEBUG` validation is kept, with debug-build meaning) |
| `en_list.c :: delete_local_entity_from_parents_child_list` | `deleteLocalEntityFromParentsChildList` | ported, tested, 100%-covered, C-reference-verified |
| `en_updt.c` dispatch (`update_client_server_entity`) | `updateClientServerEntity` | ported, tested, 100%-covered |
| `en_int.c :: default_set_entity_int_value` (group `INT_TYPE_UPDATED`) | `defaultSetEntityIntValue` | ported, tested, 100%-covered, C-reference-verified |
| `time.c :: set_manual_delta_time`, `get_delta_time`, `locked_frame_rate` | `src/core/time.ts` | ported, tested, 100%-covered, C-reference-verified (history and `system_one_over_delta_time` not ported: no ported reader) |
| `time.c :: set_delta_time`, `lock_frame_rate` | `Clock` port | blocked-engine-boundary (frame measurement) |
| Windows SDK `max` | `max` in `miscmath.ts` | ported, tested, 100%-covered |

### Frozen slice: keysite cargo (slice 4, issue #10)

| C function | TS | Status |
|---|---|---|
| `keysite.c :: update_keysite_cargo` | `updateKeysiteCargo` | ported, tested, 100%-covered, C-reference-verified (the original `keysite.c` compiled whole; DEBUG logging compiled out in EECH, not ported) |
| `keysite.c :: get_keysite_supply_position` | inlined in `updateKeysiteCargo` (`get_local_entity_vec3d_ptr (keysite, VEC3D_TYPE_POSITION)`) | ported, C-reference-verified |
| `ks_int.c :: get_local_int_value` (`INT_TYPE_SIDE`) | `overloadKeysiteFunctions` | ported, tested, C-reference-verified |
| `global.c :: set_game_status`, `global.h :: get_game_status` | `setGameStatus`, `getGameStatus` (core state, reset on `initialiseCampaignCore`) | ported, tested, 100%-covered, C-reference-verified |
| `3dobjvis.c :: get_object_3d_bounding_box` | `Object3DMetadata.getBoundingBox` | port (blocked-engine-boundary); deterministic adapter in `test/adapters` |
| `fc_msgs.c :: response_to_force_low_on_supplies` | recorded at the boundary when slice 4 froze; ported in slice 5a (below), behind the same trace line | see slice 5a |
| RTZ double sum `(xmax - xmin) + 1.0` | `f64AddRTZ` (`src/core/float32.ts`) | ported, bit-exact against C (20,000 fresh sums, 1,000 recorded, replayed in JS and Lua 5.1) |

### Slice 5a: the force's low-on-supplies response (issue #12)

| C function | TS | Status |
|---|---|---|
| `fc_msgs.c :: response_to_force_low_on_supplies` | `responseToForceLowOnSupplies` (`fc_msgs.ts`) | ported, tested, 100%-covered, C-reference-verified (the original `fc_msgs.c` compiled whole; DEBUG logging compiled out in EECH, not ported; the switch without a case is `EechUndefinedBehaviourError`) |
| `fc_msgs.c :: overload_force_message_responses` | `overloadForceMessageResponses` | partial: the `FORCE_LOW_ON_SUPPLIES` row only |
| `task.c :: entity_is_object_of_task` | `entityIsObjectOfTask` | ported, tested, 100%-covered, C-reference-verified (extracted verbatim) |
| `ts_int.c :: get_local_int_value` (`ENTITY_SUB_TYPE`, `SIDE`, `TASK_STATE`), `ts_float.c :: get_local_float_value` (`TASK_USER_DATA`), `ts_list.c :: task_dependent_link` | `overloadTaskFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `wp_int.c :: get_local_int_value` (`ENTITY_SUB_TYPE`), `wp_list.c :: task_dependent_link`, `en_float.c :: default_get_entity_float_value` (waypoint `TASK_USER_DATA`) | `overloadWaypointFunctions`, `defaultGetEntityFloatValue` | ported, tested, 100%-covered, C-reference-verified |
| `ks_list.c`, `gp_list.c :: task_dependent_root` | `overloadKeysiteFunctions`, `overloadGroupFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `keysite.c :: get_keysite_supply_position` | `getKeysiteSupplyPosition` | ported, C-reference-verified |
| `ts_dbase.c :: task_database [].task_priority` | `TASK_DATABASE_TASK_PRIORITY` | ported (generated from C, drift-checked), C-reference-verified (the harness compiles `ts_dbase.c`) |
| `taskgen.c :: create_supply_task` | `createSupplyTask` | the slice 5a boundary: ported in slice 5b (below). Tests observe its calls through `observeCreateSupplyTask`, a seam for this one function that cannot change what it does, is not public API, and is removed by `initialiseCampaignCore` |
| waypoint creation, `croute.c` | `createLocalEntityRaw` restores waypoints in tests | unported |

### Slice 5b: supply task construction (issue #14)

`create_supply_task` → `create_task` through its return. Assignment, waypoint
materialisation, expiry, packing and destruction of tasks are later slices.
`docs/slices/supply-task-construction.md` maps every step.

| C function | TS | Status |
|---|---|---|
| `taskgen.c :: create_supply_task` | `createSupplyTask` | ported, tested, 100%-covered, C-reference-verified (extracted verbatim; **F1**: see the compatibility decision below) |
| `taskgen.c :: create_task` | `createTask` | ported, tested, 100%-covered, C-reference-verified (extracted verbatim; the variable arguments are a route array; the paths other task types take are unit-tested and fail loudly where their callees are unported) |
| `taskgen.c :: get_task_start_keysite` | `getTaskStartKeysite` | ported, tested, 100%-covered, C-reference-verified (the ground and given-keysite paths: unit tests) |
| `taskgen.c :: validate_task_generation` | inlined: it returns `TRUE` (the rest is `#if 0`) | ported, C-reference-verified |
| `task.c :: find_most_suitable_keysite_for_task` | `findMostSuitableKeysiteForTask` | ported, tested, 100%-covered, C-reference-verified (`task.c` compiled whole) |
| `task.c :: assess_task_difficulty`, `assess_task_sector_difficulty` | `assessTaskDifficulty`, `assessTaskSectorDifficulty` | ported, tested, 100%-covered, C-reference-verified (`route_length` 0 is `EechUndefinedBehaviourError`) |
| `task.c :: get_local_task_list_type` | `getLocalTaskListType` | ported, tested, 100%-covered (unit tests: `create_task` never gives a `task_link` parent attribute) |
| `suitable.c :: calculate_group_to_task_suitability`, `get_group_to_task_suitability`, `initialise_group_task_array` | `suitable.ts` | ported, tested, 100%-covered (one justified exclusion, below), C-reference-verified (`suitable.c` compiled whole) |
| `ts_creat.c :: create_local`, `create_remote`, `create_server`, `overload_task_create_functions` (server) | `ts_creat.ts` | ported, tested, 100%-covered, C-reference-verified (compiled whole) |
| `ts_int.c :: set_local_int_value` (`ENTITY_SUB_TYPE`, `TASK_STATE`, `TASK_ID` 12 bits, `CRITICAL_TASK` 1, `MOVEMENT_TYPE` 3, `ROUTE_LENGTH` 16, `SIDE` 2, `TASK_DIFFICULTY` 4) | `overloadTaskFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `ts_float.c :: set_local_float_value`, `set_server_float_value` (`EXPIRE_TIMER`, `STOP_TIMER`, `TASK_PRIORITY`, `TASK_USER_DATA`) | `overloadTaskFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `ts_ptr.c :: set_local_ptr_value` (route nodes, dependents, waypoint and formation types, return keysite) | `overloadTaskFunctions` | ported, tested, 100%-covered, C-reference-verified (compiled whole) |
| `ts_list.c` (all roots and links; `task_link` serves the unassigned, assigned and completed lists) | `overloadTaskFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `ts_msgs.c :: response_to_link_parent` | `responseToLinkParent` (`task.ts`) | partial: the unassigned arm (state, `MISSION_CREATED` for a primary task); the assigned and completed arms throw unported |
| `ks_list.c :: unassigned_task_root`; `ks_int.c` (`LANDING_TYPES`, `KEYSITE_USABLE_STATE`) | `overloadKeysiteFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `gp_int.c` (`ENTITY_SUB_TYPE`, `ALIVE`); `gp_msgs.c :: response_to_link_child` | `overloadGroupFunctions` | ported, tested, 100%-covered, C-reference-verified (`response_to_link_child`: the member arm throws unported) |
| `sector.c :: get_local_sector_entity_enemy_defence_level`, `get_local_sector_entity_enemy_surface_to_air_defence_level`; `sc_int.c` (`SECTOR_SIDE`) | `sector.ts` | ported, tested, 100%-covered, C-reference-verified |
| `en_list.c :: set_client_server_entity_parent` (server) | `setClientServerEntityParent` | ported, tested, 100%-covered, C-reference-verified (extracted; the client model throws unported) |
| `en_comms.c :: transmit_entity_comms_message` (the single player trap; `ENTITY_COMMS_CREATE`, `FLOAT_VALUE`, `DESTROY`, `SET_TASK_POINTERS`, `SWITCH_PARENT`) | `en_comms.ts` → `EntityReplication` | ported, tested, 100%-covered, C-reference-verified (the harness runs the original `pack_vec3d` on the route) |
| `en_vec3d.c :: pack_vec3d` (`VEC3D_PACK_TYPE_POSITION`: the check) | `packPosition` (`en_comms.ts`) | ported with debug-build meaning: the `ASSERT` fails; the release build's in-place `bound_position_to_map_volume` is not reachable |
| `ca_msgs.c :: notify_campaign_screen` (`MISSION_CREATED`) | `notifyCampaignScreenMissionCreated` → `CampaignEvents.missionCreated` | ported, tested, 100%-covered, C-reference-verified (extracted; the harness's response table records the event) |
| `vector.c :: normalise_any_3d_vector` | `normaliseAny3dVector` | ported, tested, 100%-covered, C-reference-verified (extracted) |
| `en_world.c :: bound_position_to_adjusted_map_area`; `en_world.h :: point_inside_map_volume` | `en_world.ts` | ported, tested, 100%-covered, C-reference-verified (extracted) |
| Windows SDK `min` | `min` in `miscmath.ts` | ported, tested, 100%-covered |

### Slice 6a: supply-task assignment decision (issue #16)

`assign_keysite_tasks` up to `assign_primary_task_to_group`, the boundary.
`docs/slices/supply-task-assignment.md` maps every step.

| C function | TS | Status |
|---|---|---|
| `assign.c :: assign_keysite_tasks` | `assignKeysiteTasks` | ported, tested, 100%-covered (two exclusions past the boundary, below), C-reference-verified (extracted verbatim) |
| `assign.c :: get_suitable_registered_group` | `getSuitableRegisteredGroup` | ported, tested, 100%-covered (one exclusion, below), C-reference-verified (extracted; the `NULL` idle count of `msg_in.c` is unit-tested) |
| `assign.c :: suitable_group_task_specific_checks` | `suitableGroupTaskSpecificChecks` | ported, tested, 100%-covered, C-reference-verified (extracted) |
| `assign.c :: check_group_members_awake` | `checkGroupMembersAwake` | ported, tested, 100%-covered (one exclusion, below), C-reference-verified (extracted) |
| `assign.c :: assign_primary_task_to_group` | `assignPrimaryTaskToGroup` | **the boundary**: unported. It always throws `UnportedBoundaryError` (an `UnportedBehaviourError`) carrying the group and task; the C harness traps the same call |
| `en_misc.c :: qs`, `quicksort_entity_list` | `quicksortEntityList` | ported, tested, 100%-covered, C-reference-verified (extracted) |
| `group.c :: assess_group_task_locality_factor` | `assessGroupTaskLocalityFactor` | ported, tested, 100%-covered (two exclusions, below), C-reference-verified (extracted) |
| `ac_float.c :: get_local_float_value (CRUISE_VELOCITY)`; `en_float.c` default (`SLEEP`) | `overloadAircraftFloatValueFunctions` (helicopter and fixed wing) | ported, tested, 100%-covered, C-reference-verified (`ac_float.c` compiled whole) |
| `ac_dbase.c :: aircraft_database [].cruise_velocity` | `AIRCRAFT_DATABASE_CRUISE_VELOCITY` | ported (generated from C, drift-checked), C-reference-verified bit for bit |
| `ts_int.c` (`TASK_CATEGORY`, `MINIMUM_MEMBER_COUNT` from `ts_dbase.c`; `CRITICAL_TASK`); `ts_float.c` (`EXPIRE_TIMER`, `TASK_PRIORITY`) getters | `overloadTaskFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `gp_int.c` (`MEMBER_COUNT` getter); `gp_list.c` (`pilot_lock_link`, `registry_link`); `fc_list.c` (`air_registry_root`); `pi_list.c` (`pilot_lock_root`) | `overloadGroupFunctions`, `overloadForceFunctions`, `overloadPilotFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `gp_msgs.c :: response_to_link_child (LIST_TYPE_MEMBER)` (live `member_count` maintenance) | none | unported, fail-loud. Tests restore `member_count` raw, as `gp_pack.c :: unpack_local_data` does |
| `ts_dbase.c` (`task_category`, `minimum_member_count`), `ks_dbase.c` (`assign_task_count`, `reserve_task_count`), `gp_dbase.c` (`minimum_idle_count`) | generated columns | ported (generated from C, drift-checked), C-reference-verified |

### Frozen slice: entity lifecycle, CARGO, sector membership (slice 3)

| C function | TS | Status |
|---|---|---|
| `en_heap.c :: initialise_entity_heap`, `reset_entity_heap` | `initialiseEntityHeap`, lazily materialised records (`getLocalEntityPtr`) | ported, tested, 100%-covered, C-reference-verified |
| `en_heap.c :: get_free_entity` (`ENTITY_INDEX_DONT_CARE` and a specific index) | `getFreeEntity` | ported, tested, 100%-covered, C-reference-verified (client creation, the run-time caller with a specific index, stays unported at the create tables) |
| `en_heap.c :: set_free_entity` | `setFreeEntity` | ported, tested, 100%-covered, C-reference-verified |
| `en_creat.c :: create_client_server_entity`, `create_local_entity` | `createClientServerEntity`, `createLocalEntity` | ported, tested, 100%-covered, C-reference-verified (TX stack attributes and RX buffer both become the attribute array) |
| `en_valid.c :: assert_local_create_entity_index`, `assert_remote_create_entity_index` (server) | `validateLocalCreateEntityIndex`, `validateRemoteCreateEntityIndex` | ported, tested, 100%-covered, C-reference-verified |
| `en_attrs.c :: set_local_entity_attributes` (`INT_VALUE`, `FLOAT_VALUE`, `VEC3D`, `PARENT`, `CHILD_PRED`) | `setLocalEntityAttributes` | ported, tested, 100%-covered, C-reference-verified (`FLOAT_VALUE`: unit test only, no ported raw float setter); the other attribute kinds are not representable |
| `en_attrs.c :: pack_entity_attributes` (what `ENTITY_COMMS_CREATE` carries) | `replicatedEntityAttributes` → `EntityReplication.transmitEntityCreate` | ported, tested, 100%-covered, C-reference-verified |
| `en_dstry.c :: destroy_local_entity`, `destroy_client_server_entity`, `destroy_client_server_entity_family`, `default_destroy_entity*` (`ENTITY_TYPE_UNKNOWN`) | `en_dstry.ts` | ported, tested, 100%-covered, C-reference-verified |
| `en_list.c :: unlink_local_entity_children` | `unlinkLocalEntityChildren` | ported, tested, 100%-covered, C-reference-verified (with children: unit test only) |
| `en_list/set_frst.h`, `set_prnt.h`, `set_succ.h`, `set_pred.h` :: `ASSERT (en != ...)` | the `en_list.ts` setters | ported, tested, 100%-covered, C-reference-verified (added in slice 3; the slice 2 setters lacked it) |
| `en_world.c :: set_entity_world_map_size`; `en_world.h :: point_inside_map_area`, `get_x_sector`, `get_z_sector`; `miscell.c :: int_bit_count` | `en_world.ts` | ported, tested, 100%-covered, C-reference-verified |
| `sc_seccreat.c :: create_local_sector_entities`, `create_local` | `createLocalSectorEntities`, `overloadSectorFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `sector.c :: get_local_sector_entity`, `get_local_raw_sector_entity` | `getLocalSectorEntity`, `getLocalRawSectorEntity` | ported, tested, 100%-covered, C-reference-verified |
| `sector.c :: add_mobile_values_to_sector`, `remove_mobile_values_from_sector` | `addMobileValuesToSector`, `removeMobileValuesFromSector` | partial: the vehicle arm (imap defence levels) throws unported |
| `sc_msgs.c :: response_to_link_child`, `response_to_unlink_child` | `overloadSectorFunctions` | partial: the fixed-entity arm (object dimensions, Slice 4) and the aircraft / vehicle arm (fog of war, `FORCE_ENTERED_SECTOR`) throw unported |
| `sc_int.c` (`X_SECTOR`, `Z_SECTOR`, 8-bit fields), `sc_list.c` | `overloadSectorFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `mb_int.c` (`ALIVE` 1 bit, `SIDE` 2 bits, `ENTITY_SUB_TYPE`), `mb_vec3d.c` (`POSITION`), `mb_list.c`; `en_int.c` default 0 for `IDENTIFY_FIXED` / `AIRCRAFT` / `VEHICLE` | `overloadMobileRawStateFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `ac_msgs.c :: response_to_link_parent`, `response_to_unlink_parent` (for cargo) | `overloadAircraftLinkParentResponses` | partial: the target, gunship-target and update arms throw unported |
| `cg_creat.c :: create_local`, `create_remote`, `create_server` | `cargo.ts` | ported, tested, 100%-covered, C-reference-verified |
| `cg_dstry.c :: destroy_local`, `destroy_remote`, `destroy_server`, `destroy_server_family` | `cargo.ts` | ported, tested, 100%-covered, C-reference-verified |
| `cg_list.c` | `overloadCargoFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `soundeff.c :: destroy_client_server_sound_effects` | `destroyClientServerSoundEffects` | ported, tested, 100%-covered (with special effects: unit test only; destroying a sound effect throws unported) |
| `ks_msgs.c :: response_to_link_child`, `response_to_unlink_child`; `ks_list.c :: LIST_TYPE_CARGO_ROOT` | `overloadKeysiteFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `cg_dstry.c :: kill_local` and friends, `cg_move.c`, `cg_updt.c`, `cg_draw.c`, `cg_pack.c`, `cg_int.c` | none | unported |
| `en_stats.c :: update_create_entity_statistics`, `update_destroy_entity_statistics` | none | excluded (debug display counters; executed in the harness) |

### Accessor overloads reached by the slices

Since slice 2 the harness compiles the original group translation units, and
since slice 3 the keysite and force ones, so these accessors are
`C-reference-verified`. Session, guide and aircraft accessors are still supplied
by the harness shim, and are verified by source reading only. The plan and the rules for new slices are in
`docs/architecture.md`, "Shrinking the C reference shim".

| C | TS | Status |
|---|---|---|
| `gp_int.c :: get_local_int_value` (`INT_TYPE_GROUP_MODE`, `INT_TYPE_RESUPPLY_SOURCE`, `INT_TYPE_SIDE`) | `overloadGroupFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `gp_int.c` (all other int types) | none | unported |
| `gp_float.c :: set_local_float_value`, `set_server_float_value` (`AMMO_SUPPLY_LEVEL`, `FUEL_SUPPLY_LEVEL`) | `overloadGroupFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `gp_float.c :: set_client_float_value`, the remaining float types | none | unported |
| `gp_ptr.c :: get_local_ptr_value (PTR_TYPE_GROUP_LEADER)` | `overloadGroupFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `gp_ptr.c :: set_local_ptr_value (PTR_TYPE_GROUP_LEADER)` | none | unported |
| `gp_vec3d.c :: get_local_vec3d_ptr (VEC3D_TYPE_POSITION)` | `overloadGroupFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `gp_list.c` / `en_list/*.h`: `member_root`, `guide_stack_root`, `group_link` (BUILDING/INDEPENDENT/KEYSITE_GROUP), `update_link` | `overloadGroupFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `gp_dbase.c :: group_database[].resupply_source` | `GROUP_DATABASE_RESUPPLY_SOURCE` | ported (generated from C, drift-checked), C-reference-verified (the harness reads the compiled `group_database`) |
| `ks_int.c :: get_local_int_value` (`ENTITY_SUB_TYPE`, `IN_USE`) | `overloadKeysiteFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `ks_float.c :: get_local_float_value`, `set_server_float_value` (`AMMO_SUPPLY_LEVEL`, `FUEL_SUPPLY_LEVEL`) | `overloadKeysiteFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `ks_vec3d.c :: get_local_vec3d_ptr (VEC3D_TYPE_POSITION)` | `overloadKeysiteFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `ks_list.c`: `keysite_group_root`, `building_group_root`, `cargo_root`, `keysite_force_link` | `overloadKeysiteFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `fc_int.c :: get_local_int_value (INT_TYPE_SIDE)` | `overloadForceFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `fc_list.c`: `keysite_force_root`, `independent_group_root`, `force_link` | `overloadForceFunctions` | ported, tested, 100%-covered, C-reference-verified |
| `fc_msgs.c :: response_to_force_low_on_supplies` | slice 1 stopped at this message boundary; ported in slice 5a (below) | see slice 5a |
| `ss_list.c`: `force_root` | `overloadSessionListFunctions` | ported, tested, 100%-covered, source-read |
| `gd_list.c`: `guide_stack_link` | `overloadGuideFunctions` | ported, tested, 100%-covered, source-read |
| `ac_list.c` / `vh_list.c`: `member_link` | `overloadMobileFunctions` | ported, tested, 100%-covered, source-read |
| `ac_vec3d.c` / `vh_vec3d.c :: get_local_vec3d_ptr (VEC3D_TYPE_POSITION)` | `overloadMobileFunctions` → `MobilePhysicalState` port | blocked-engine-boundary (physical state through a port) |

### Entity runtime

| C | TS | Status |
|---|---|---|
| `en_list.c :: get_local_entity_first_child`, `get_local_entity_parent`, `get_local_entity_child_succ` | `en_list.ts` | ported, tested, 100%-covered |
| `en_list.c :: get_local_entity_child_pred` | `getLocalEntityChildPred` | ported, tested, 100%-covered |
| `en_list.c :: insert_local_entity_into_parents_child_list` | `insertLocalEntityIntoParentsChildList` (slice 2), and `insertLocalEntityIntoParentsChildListRaw` for restoring state without notifications | ported |
| `en_list.c :: set_local_entity_parent`, `set_local_entity_child_pred` (used by attributes) | `setLocalEntityParent`, `setLocalEntityChildPred` | ported, tested, 100%-covered, C-reference-verified |
| the other `en_list.c` functions | none | unported |
| `en_funcs.h :: get/set_local_entity_type`, `get/set_local_entity_data` | `entity.ts` | ported, tested, 100%-covered |
| `en_pack.c` | `createLocalEntityRaw` (restore primitive: the next heap entry with its raw data) | unported |
| `en_int.c`, `en_float.c`, `en_vec3d.c`, `en_ptr.c` dispatch macros | `en_values.ts` | ported, tested, 100%-covered |
| `en_*.c :: default_*` handlers | the unported sentinel (throws) instead of EECH defaults | deliberate: see architecture question 2. Installed only where the C table is known to keep the default: `default_set_entity_int_value` (group `UPDATED`), `default_get_entity_int_value` (cargo `IDENTIFY_*`), `default_get_entity_float_value` (waypoint `TASK_USER_DATA`, slice 5a), `default_destroy_entity*` (`ENTITY_TYPE_UNKNOWN`) |
| `en_msgs.c :: default_message_response` | `defaultMessageResponse` | ported; installed only where the C table keeps the default (the update entity's link responses) |
| `en_valid.h :: validate_client_server_local_fn / remote_fn` | none | excluded (debug-build dispatch validation) |
| client comms model overloads (`set_client_float_value`) | none | unported (the core is the server authority) |

## Deviations and exclusions

- **Coverage exclusions (slice 6a):** these branches are excluded narrowly
  (`/* istanbul ignore */`), each backed by a test. Each is listed in
  `docs/slices/supply-task-assignment.md`.
  - **Past the boundary**, in `assign.ts`: the `assign_count == 0` break and the result arms of `assign_primary_task_to_group`.
  - **The sleep rejection** of `check_group_members_awake`, and its caller's else arm. An aircraft member's sleep is the default 0.0 (**6a-F2**).
  - **Two arms of `assess_group_task_locality_factor`** (`group.ts`) that the only ported caller cannot reach: no member, and a task off a keysite's unassigned list.
- **6a-F1 (finding).** Every nonzero group-to-task suitability is exactly
  1.0, so "least suitable wins" is "the first qualifying group wins".
  **6a-F2 (finding).** On the assignment path the ETA divisor is never 0: every
  aircraft's cruise velocity is positive, and only aircraft groups join the
  air registry. `test/unit/supply-task-assignment.test.ts` holds both invariants.
- **Coverage exclusions (slice 5b):** one. `suitable.c`'s movement stealth rejection
  (`src/ai/highlevl/suitable.ts`, `/* istanbul ignore if */`) cannot fire with
  EECH's databases: only BDA and RECON need stealth, and every group that has
  passed the checks before it for them has it.
  `test/unit/supply-task-construction.test.ts` recomputes this from the
  generated databases, so a database change that makes it reachable fails a
  test. Type-only modules (`src/ports/**`) emit no statements.
- **ASSERT and NULL dereference.** A failed EECH `ASSERT` / `debug_assert` throws
  `EechAssertionError` quoting the C expression. Where the C dereferences NULL
  without an assert (`get_closest_keysite` with no force for the side), the port
  throws `EechNullDereferenceError`. EECH itself would crash in both cases.
- **Undefined behaviour the port refuses.** Where the original reads an
  uninitialised value on a path no caller takes
  (`response_to_force_low_on_supplies` for a cargo sub type its switch has no
  case for), the port throws `EechUndefinedBehaviourError` instead of inventing
  a value.
- **Unported behaviour always fails loudly.** There is no policy that turns
  unported campaign code into a no-op, and observation never alters control
  flow. Slice 6a's boundary, `assign_primary_task_to_group`, throws
  `UnportedBoundaryError`, which carries the selection; runners report it and
  cannot suppress the call. Slice 5a's boundary, `taskgen.c :: create_supply_task`, is ported in
  slice 5b; its test seam is now an observer (`observeCreateSupplyTask`), which
  sees the arguments and cannot change what the function does or returns. Its
  interceptor (slice 5a) is gone. The `unportedMessagePolicy` option of slices 1 and 4 is
  gone too.
- **F1: a compatibility decision, not a translation** (slice 5b, issue #14).
  `create_supply_task` never initialises `prepare.y` or `finish.y`, and
  `create_task` stores `ceil ()` of them in the route. EECH defines no value:
  the heights are whatever the build's stack or registers held, and they
  reach the saved route, the multiplayer transmission (where `pack_vec3d`
  asserts in debug builds and clamps in release builds) and the campaign's
  route checksum. **The port sets both to 0.0 by decision.** This resolves
  undefined behaviour in the EECH source. It does not reproduce a value EECH
  defined, and nothing claims EECH produced 0.0. The C reference pins the
  otherwise unchanged original to the same choice by compiling
  `eech_extracted_taskgen.c`, and only that unit, with
  `-ftrivial-auto-var-init=zero` (`c-reference/extract.mjs`, `UNIT_FLAGS`).
  The F1 probe (`npm run probe:f1`) keeps the evidence. The same code built
  pattern- or un-initialised stores other heights (0xfefefefe, a NaN, a code
  address), and multiplayer packing then fails where single player does not.
  With 0.0, single player and multiplayer construct the same route (an
  acceptance case in every runner).
- **`max`** is the Windows SDK macro, not `Math.max`: `max (NaN, 0.0f)` is `0.0f`.
- **Findings about the original C** (recorded, not fixed): `time.c ::
  set_manual_delta_time` has an undefined-behaviour history index update, and
  `up_update.c` clears only a quarter of its entity bitsets. Neither affects
  ported behaviour; see `docs/slices/group-update-timing.md`.
- **Float arithmetic: the EECH numerical contract** (issue #7,
  `docs/fidelity/fpu-semantics.md`):

  | Aspect | Rule | Status |
  |---|---|---|
  | Rounding direction | toward zero (`startup.c :: set_fpu_rounding_mode_zero`, re-asserted after every library initialisation) | established, canonical |
  | Declared float operations | IEEE binary32, rounded toward zero (`toFloat32RTZ`, `f32Add/Sub/Mul/Div/Sqrt`) | canonical |
  | float → int | truncation (`toCInt`; `convert_float_to_int` is `fistp` under RTZ) | established |
  | Compile-time constants (e.g. `time.c`'s `0.1`) | round to nearest (`toFloat32`) | established |
  | x87 intermediate evaluation precision (24, 53 or 64 bits) and when values are rounded back to declared type | not modelled (declared type) | **unresolved**: needs the Windows runtime control-word trace plus the shipped compiler's instruction behaviour |

  The `get_2d_range` sqrt argument is the canary for the unresolved row. It is
  the one ported operation whose results depend on intermediate precision in
  the corpora (spike: 15/1500 Slice 1 scenarios, all closest-keysite ranges).
- **`debug_fatal`** throws `EechFatalError`, carrying the C format string
  (compared with the C reference) and a formatted message.
- **Undefined behaviour** that crashes EECH (integer division by zero, e.g. a
  cargo created before any world map) throws `EechUndefinedBehaviourError`.
  Slice 5b adds three more cases. `create_task` reading past its last route
  argument is one, and `assess_task_difficulty` reading `route_nodes [-1]` is
  another. The third is `get_local_raw_sector_entity` indexing outside the
  sector map.
- **Bit-fields.** Stores into `unsigned int` bit-fields keep the low bits
  (`storeUnsignedBitfield`): mobile `alive` (1), `side` (2), sector
  `x_sector` / `z_sector` (8), and (slice 5b) task `task_id` (12),
  `critical_task` (1), `movement_type` (3), `difficulty` (4), `route_length`
  (16), `side` (2).
- **`debug_log`** under `DEBUG_MODULE` / `DEBUG_SUPPLY` is compiled out in EECH
  and not ported.
