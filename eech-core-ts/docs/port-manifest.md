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
| `aphavoc/source/entity/special/group/gp_dbase.c` | `src/generated/c-group-database.ts` (generated) | partial |
| `aphavoc/source/entity/special/keysite/keysite.c` | `src/entity/special/keysite/keysite.ts` | partial |
| `aphavoc/source/entity/special/keysite/ks_int.c`, `ks_float.c`, `ks_vec3d.c`, `ks_list.c` | `src/entity/special/keysite/keysite.ts` | partial |
| `aphavoc/source/entity/special/force/force.c` | `src/entity/special/force/force.ts` | partial |
| `aphavoc/source/entity/special/force/fc_int.c`, `fc_list.c`, `fc_msgs.c` | `src/entity/special/force/force.ts` | partial |
| `aphavoc/source/entity/special/session/session.h`, `ss_list.c` | `src/entity/special/session/session.ts`, `src/entity/system/entity.ts` | partial |
| `aphavoc/source/entity/special/guide/gd_list.c` | `src/entity/special/guide/guide.ts` | partial |
| `aphavoc/source/entity/mobile/aircraft/ac_list.c`, `ac_vec3d.c`; `vehicle/vh_list.c`, `vh_vec3d.c` | `src/entity/mobile/mobile.ts` | partial (campaign surface only) |
| `aphavoc/source/entity/mobile/**` (flight models, movement, weapons, damage, drawing) | none | excluded-physical-simulation |
| `aphavoc/source/entity/system/en_funcs/en_list.c`, `en_list/*.h` | `src/entity/system/en_list.ts` | partial |
| `aphavoc/source/entity/system/en_funcs/en_int.c`, `en_float.c`, `en_vec3d.c`, `en_ptr.c` | `src/entity/system/en_values.ts`, `function-table.ts` | partial |
| `aphavoc/source/entity/system/en_msgs/en_msgs.c` | `src/entity/system/en_msgs.ts` | partial |
| `aphavoc/source/entity/system/en_main/*` | `src/entity/system/entity.ts` | partial |
| `aphavoc/source/entity/system/en_comms/en_comms.c` | `src/ports/entity-replication.ts` (port) | blocked-engine-boundary |
| `aphavoc/source/comms/comms.c` (`get_comms_model`) | `src/entity/system/comms.ts` | partial |
| `modules/maths/range.c` | `src/core/maths/range.ts` | partial |
| `modules/maths/miscmath.h`, `constant.h`, `vector.h` | `src/core/maths/miscmath.ts`, `vec3d.ts` | partial |
| `aphavoc/source/entity/system/en_types/en_suply.h` (`FUEL_USAGE_ACCELERATOR`, `AMMO_USAGE_ACCELERATOR`) | `src/generated/c-constants.ts` (generated) | partial |
| `modules/system/assert.h` | `src/core/assert.ts` | partial |
| enum headers (`en_types.h`, `en_side.h`, `en_list.h`, `en_int.h`, `en_float.h`, `en_vec3d.h`, `en_ptr.h`, `en_msgs.h`, `en_sbtyp.h`, `ai_extrn.h`, `comms.h`, `en_suply.h`) | `src/generated/c-enums.ts` (generated) | ported (selected enums, generated verbatim) |
| `ai/highlevl/*`, `ai/taskgen/*`, `ai/frontl/*`, `ai/faction/*`, `ai/ai_misc/*` | none | unported |
| `entity/special/division`, `task`, `waypoint`, `landing`, `sector`, `regen` | none | unported |
| `wutcfg.c`, `gwutcfg.c` (runtime overrides of `group_database`) | none | unported |

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

### Accessor overloads reached by the slice

In the C harness these accessors are supplied by the shim, so they are verified
by source reading only (see architecture question 6). The plan for replacing
the shim with the real translation units, and the rules for new slices, are in
`docs/architecture.md`, "Shrinking the C reference shim".

| C | TS | Status |
|---|---|---|
| `gp_int.c :: get_local_int_value` (`INT_TYPE_GROUP_MODE`, `INT_TYPE_RESUPPLY_SOURCE`, `INT_TYPE_SIDE`) | `overloadGroupFunctions` | ported, tested, 100%-covered, source-read |
| `gp_int.c` (all other int types) | none | unported |
| `gp_float.c :: set_local_float_value`, `set_server_float_value` (`AMMO_SUPPLY_LEVEL`, `FUEL_SUPPLY_LEVEL`) | `overloadGroupFunctions` | ported, tested, 100%-covered, source-read |
| `gp_float.c :: set_client_float_value`, all other float types | none | unported |
| `gp_ptr.c :: get_local_ptr_value (PTR_TYPE_GROUP_LEADER)` | `overloadGroupFunctions` | ported, tested, 100%-covered, source-read |
| `gp_ptr.c :: set_local_ptr_value (PTR_TYPE_GROUP_LEADER)` | none | unported |
| `gp_vec3d.c :: get_local_vec3d_ptr (VEC3D_TYPE_POSITION)` | `overloadGroupFunctions` | ported, tested, 100%-covered, source-read |
| `gp_list.c` / `en_list/*.h`: `member_root`, `guide_stack_root`, `group_link` (BUILDING/INDEPENDENT/KEYSITE_GROUP) | `overloadGroupFunctions` | ported, tested, 100%-covered, source-read |
| `gp_dbase.c :: group_database[].resupply_source` | `GROUP_DATABASE_RESUPPLY_SOURCE` | ported (generated from C, drift-checked) |
| `ks_int.c :: get_local_int_value` (`ENTITY_SUB_TYPE`, `IN_USE`) | `overloadKeysiteFunctions` | ported, tested, 100%-covered, source-read |
| `ks_float.c :: get_local_float_value`, `set_server_float_value` (`AMMO_SUPPLY_LEVEL`, `FUEL_SUPPLY_LEVEL`) | `overloadKeysiteFunctions` | ported, tested, 100%-covered, source-read |
| `ks_vec3d.c :: get_local_vec3d_ptr (VEC3D_TYPE_POSITION)` | `overloadKeysiteFunctions` | ported, tested, 100%-covered, source-read |
| `ks_list.c`: `keysite_group_root`, `building_group_root`, `keysite_force_link` | `overloadKeysiteFunctions` | ported, tested, 100%-covered, source-read |
| `fc_int.c :: get_local_int_value (INT_TYPE_SIDE)` | `overloadForceFunctions` | ported, tested, 100%-covered, source-read |
| `fc_list.c`: `keysite_force_root`, `independent_group_root`, `force_link` | `overloadForceFunctions` | ported, tested, 100%-covered, source-read |
| `fc_msgs.c :: response_to_force_low_on_supplies` | declared unported message response | unported (next slice; the slice stops at this message boundary) |
| `ss_list.c`: `force_root` | `overloadSessionListFunctions` | ported, tested, 100%-covered, source-read |
| `gd_list.c`: `guide_stack_link` | `overloadGuideFunctions` | ported, tested, 100%-covered, source-read |
| `ac_list.c` / `vh_list.c`: `member_link` | `overloadMobileFunctions` | ported, tested, 100%-covered, source-read |
| `ac_vec3d.c` / `vh_vec3d.c :: get_local_vec3d_ptr (VEC3D_TYPE_POSITION)` | `overloadMobileFunctions` → `MobilePhysicalState` port | blocked-engine-boundary (physical state through a port) |

### Entity runtime

| C | TS | Status |
|---|---|---|
| `en_list.c :: get_local_entity_first_child`, `get_local_entity_parent`, `get_local_entity_child_succ` | `en_list.ts` | ported, tested, 100%-covered |
| `en_list.c :: get_local_entity_child_pred` | none | unported (no adopted caller) |
| `en_list.c :: insert_local_entity_into_parents_child_list` | `insertLocalEntityIntoParentsChildListRaw` (pointer updates only, no `LINK_CHILD`/`LINK_PARENT` notifications) | partial |
| `en_list.c :: delete_local_entity_from_parents_child_list` and the other list functions | none | unported |
| `en_main.c :: get_local_entity_type`, `get_local_entity_data` | `entity.ts` | ported, tested, 100%-covered |
| `en_creat.c`, `en_dstry.c`, `en_pack.c`, `en_updt.c` | `createLocalEntityRaw` (restore primitive only) | unported |
| `en_int.c`, `en_float.c`, `en_vec3d.c`, `en_ptr.c` dispatch macros | `en_values.ts` | ported, tested, 100%-covered |
| `en_*.c :: default_*` handlers | the unported sentinel (throws) instead of EECH defaults | deliberate: see architecture question 2 |
| `en_msgs.c :: default_message_response` | none | unported (no response proven to use it yet) |
| `en_valid.h :: validate_client_server_local_fn / remote_fn` | none | excluded (debug-build dispatch validation) |
| client comms model overloads (`set_client_float_value`) | none | unported (the core is the server authority) |

## Deviations and exclusions

- **Coverage exclusions:** none. Type-only modules (`src/ports/**`) emit no
  statements. There are no `istanbul ignore` comments in campaign code.
- **ASSERT and NULL dereference.** A failed EECH `ASSERT` / `debug_assert` throws
  `EechAssertionError` quoting the C expression. Where the C dereferences NULL
  without an assert (`get_closest_keysite` with no force for the side), the port
  throws `EechNullDereferenceError`. EECH itself would crash in both cases.
- **Float arithmetic** follows IEEE single precision at declared type
  (`FLT_EVAL_METHOD == 0`). Historical x87 builds may differ in the last bit.
- **`debug_log`** under `DEBUG_MODULE` / `DEBUG_SUPPLY` is compiled out in EECH
  and not ported.
