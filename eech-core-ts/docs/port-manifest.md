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
| `aphavoc/source/entity/special/group/gp_msgs.c` | `src/entity/special/group/group.ts` (link/unlink parent responses) | partial |
| `aphavoc/source/entity/special/update/up_update.c` | `src/entity/special/update/update.ts` | partial (tacview and mobile bitsets excluded) |
| `aphavoc/source/entity/special/update/up_msgs.c` | `src/entity/special/update/update.ts` | ported |
| `aphavoc/source/entity/special/update/up_list.c` | `src/entity/special/update/update.ts` | ported |
| `aphavoc/source/entity/system/en_funcs/en_updt.c` | `src/entity/system/en_updt.ts` | ported (default handler deliberately not installed) |
| `modules/system/time.c` | `src/core/time.ts`, `src/ports/clock.ts` (port) | partial (measurement is the `Clock` port; delta history not ported) |
| `aphavoc/source/cmndline.c` (`command_line_entity_update_frame_rate`) | `src/core/cmndline.ts` | partial |
| `aphavoc/source/entity/special/group/gp_dbase.c` | `src/generated/c-group-database.ts` (generated) | partial |
| `aphavoc/source/entity/special/keysite/ks_dbase.c` (`default_supply_usage` ammo and fuel) | `src/generated/c-keysite-database.ts` (generated) | partial (compiled defaults; `wutcfg.c` overrides not ported) |
| `aphavoc/source/global.c`, `global.h` (`game_status`, `set_game_status`, `get_game_status`) | `src/core/game-status.ts` | partial (`game_status_string` not ported: no campaign reader) |
| `modules/3d/3dobjvis.c :: get_object_3d_bounding_box`, `objects.h :: struct OBJECT_3D_BOUNDS` | `src/ports/object-3d-metadata.ts` (port) | blocked-engine-boundary (the 3D object database) |
| `modules/3d/3dmodels.h` (`OBJECT_3D_SINGLE_CRATE`) | `src/generated/c-constants.ts` (generated) | partial (the indices the port names) |
| `aphavoc/source/entity/special/keysite/keysite.c` | `src/entity/special/keysite/keysite.ts` | partial |
| `aphavoc/source/entity/special/keysite/ks_int.c`, `ks_float.c`, `ks_vec3d.c`, `ks_list.c`, `ks_msgs.c` | `src/entity/special/keysite/keysite.ts` | partial |
| `aphavoc/source/entity/special/force/force.c` | `src/entity/special/force/force.ts` | partial |
| `aphavoc/source/entity/special/force/fc_int.c`, `fc_list.c` | `src/entity/special/force/force.ts` | partial |
| `aphavoc/source/entity/special/force/fc_msgs.c` | `src/entity/special/force/fc_msgs.ts` | partial (`response_to_force_low_on_supplies`, slice 5a) |
| `aphavoc/source/entity/special/task/task.c` (`entity_is_object_of_task`), `ts_int.c`, `ts_float.c`, `ts_list.c` | `src/entity/special/task/task.ts` | partial (a restored task's sub type, side, state, user data and `task_dependent_link`, slice 5a; creation is slice 5b) |
| `aphavoc/source/entity/special/task/ts_dbase.c` (`task_priority`) | `src/generated/c-task-database.ts` (generated) | partial |
| `aphavoc/source/entity/special/waypoint/wp_int.c`, `wp_list.c` | `src/entity/special/waypoint/waypoint.ts` | partial (a restored waypoint's sub type and `task_dependent_link`, slice 5a) |
| `aphavoc/source/ai/taskgen/taskgen.c` (`create_supply_task`) | `src/ai/taskgen/taskgen.ts` | boundary: the slice 5a call is recorded in tests and fails loudly in production; ported in slice 5b |
| `aphavoc/source/entity/special/session/session.h`, `ss_list.c` | `src/entity/special/session/session.ts`, `src/entity/system/entity.ts` | partial |
| `aphavoc/source/entity/special/guide/gd_list.c` | `src/entity/special/guide/guide.ts` | partial |
| `aphavoc/source/entity/mobile/aircraft/ac_list.c`, `ac_vec3d.c`; `vehicle/vh_list.c`, `vh_vec3d.c` | `src/entity/mobile/mobile.ts` | partial (campaign surface only) |
| `aphavoc/source/entity/mobile/mb_int.c`, `mb_vec3d.c`, `mb_list.c` | `src/entity/mobile/mobile.ts` (`overloadMobileRawStateFunctions`) | partial (the rows cargo reaches) |
| `aphavoc/source/entity/mobile/aircraft/ac_msgs.c` | `src/entity/mobile/aircraft/ac_msgs.ts` | partial (link / unlink parent responses) |
| `aphavoc/source/entity/mobile/cargo/cg_creat.c`, `cg_dstry.c`, `cg_list.c`, `cg_funcs.c`, `cg_msgs.c` | `src/entity/mobile/cargo/cargo.ts` | partial (kill, movement, update, draw, pack and `cg_int.c` not ported) |
| `aphavoc/source/entity/special/sector/sector.c`, `sc_seccreat.c`, `sc_int.c`, `sc_list.c`, `sc_msgs.c` | `src/entity/special/sector/sector.ts` | partial |
| `aphavoc/source/entity/special/effect/soundeff/soundeff.c` | `src/entity/special/effect/soundeff.ts` | partial (`destroy_client_server_sound_effects`) |
| `aphavoc/source/entity/mobile/**` (flight models, movement, weapons, damage, drawing) | none | excluded-physical-simulation |
| `aphavoc/source/entity/system/en_funcs/en_list.c`, `en_list/*.h` | `src/entity/system/en_list.ts` | partial |
| `aphavoc/source/entity/system/en_funcs/en_int.c`, `en_float.c`, `en_vec3d.c`, `en_ptr.c` | `src/entity/system/en_values.ts`, `function-table.ts` | partial |
| `aphavoc/source/entity/system/en_msgs/en_msgs.c` | `src/entity/system/en_msgs.ts` | partial |
| `aphavoc/source/entity/system/en_main/en_heap.c`, `en_heap.h` | `src/entity/system/en_heap.ts`, `entity.ts` | partial (downwash heap and packing not ported) |
| `aphavoc/source/entity/system/en_main/en_world.c`, `en_world.h`; `misc/miscell.c :: int_bit_count` | `src/entity/system/en_world.ts` | partial |
| `aphavoc/source/entity/system/en_main/*` (other files) | `src/entity/system/entity.ts` | partial |
| `aphavoc/source/entity/system/en_attrs/en_attrs.c` | `src/entity/system/en_attrs.ts` | partial (`set_local_entity_attributes`; pack/unpack not ported) |
| `aphavoc/source/entity/system/en_funcs/en_creat.c`; `en_debug/en_valid.c` (create index checks) | `src/entity/system/en_creat.ts` | partial |
| `aphavoc/source/entity/system/en_funcs/en_dstry.c` | `src/entity/system/en_dstry.ts` | partial (kill and whole-heap destruction not ported) |
| `modules/system/fpu.c :: convert_float_to_int` | `toCInt` (`src/core/cint.ts`) | ported (truncation: EECH's round-toward-zero FPU mode) |
| `aphavoc/source/entity/system/en_comms/en_comms.c` | `src/ports/entity-replication.ts` (port) | blocked-engine-boundary |
| `aphavoc/source/comms/comms.c` (`get_comms_model`) | `src/entity/system/comms.ts` | partial |
| `modules/maths/range.c` | `src/core/maths/range.ts` | partial |
| `modules/maths/miscmath.h`, `constant.h`, `vector.h` | `src/core/maths/miscmath.ts`, `vec3d.ts` | partial |
| `aphavoc/source/entity/system/en_types/en_suply.h` (`FUEL_USAGE_ACCELERATOR`, `AMMO_USAGE_ACCELERATOR`, `KEYSITE_SUPPLY_REQUEST_THRESHOLD`); `cargo.h` (`CARGO_AMMO_SIZE`, `CARGO_FUEL_SIZE`) | `src/generated/c-constants.ts` (generated) | partial |
| `modules/system/assert.h` | `src/core/assert.ts` | partial |
| enum headers (`en_types.h`, `en_side.h`, `en_list.h`, `en_int.h`, `en_float.h`, `en_vec3d.h`, `en_ptr.h`, `en_msgs.h`, `en_sbtyp.h`, `ai_extrn.h`, `comms.h`, `en_suply.h`) | `src/generated/c-enums.ts` (generated) | ported (selected enums, generated verbatim) |
| `ai/highlevl/*`, `ai/taskgen/*` (except the `create_supply_task` boundary), `ai/frontl/*`, `ai/faction/*`, `ai/ai_misc/*` | none | unported |
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
| `taskgen.c :: create_supply_task` | `createSupplyTask` | boundary (slice 5b): throws `UnportedBehaviourError`; conformance tests replace it through `interceptCreateSupplyTask`, a seam for this one function that is not public API and that `initialiseCampaignCore` removes |
| task and waypoint creation, `ts_creat.c`, `create_task`, `croute.c` | `createLocalEntityRaw` restores them in tests | unported (slice 5b) |

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

- **Coverage exclusions:** none. Type-only modules (`src/ports/**`) emit no
  statements. There are no `istanbul ignore` comments in campaign code.
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
  unported campaign code into a no-op. Slice 5a's boundary,
  `taskgen.c :: create_supply_task`, throws `UnportedBehaviourError`;
  conformance tests replace that one function through `interceptCreateSupplyTask`
  (not public API, removed by `initialiseCampaignCore`), which records the call
  and returns what the test decides. The unported message responses and the
  `unportedMessagePolicy` option of slices 1 and 4 are gone: the response they
  stood for is ported.
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
- **Bit-fields.** Stores into `unsigned int` bit-fields keep the low bits
  (`storeUnsignedBitfield`): mobile `alive` (1), `side` (2), sector
  `x_sector` / `z_sector` (8).
- **`debug_log`** under `DEBUG_MODULE` / `DEBUG_SUPPLY` is compiled out in EECH
  and not ported.
