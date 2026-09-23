# Slice 5 investigation: `response_to_force_low_on_supplies` and the supply-task chain

Status: **investigation only, nothing ported.** The trace was done against master at `e6a74dc`, where Slice 4 (PR #11) is frozen. Line numbers refer to that commit.

The question: where does the C path that starts at `ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES` actually end?

## Answer

- **The synchronous path is bounded.** It ends when `create_task` returns (`taskgen.c:438`) and control passes back through `create_supply_task`, which sets `TASK_USER_DATA`.
  - It sends no further messages for a supply task.
  - It assigns no group, and makes no recursive task creation.
- **Everything after that is time-driven, from other update loops:**
  - task expiry in `ts_updt.c :: update_server` (20 minutes unassigned, then `ENTITY_MESSAGE_TASK_TERMINATED`);
  - assignment in `ks_updt.c:115 :: assign_keysite_tasks (en, TASK_CATEGORY_SUPPORT)`.

  Neither of these is on the call path, so a slice can stop at the created, unassigned task.

- **The path's breadth is the problem, not its depth.** It brings in:
  - a new entity type (`ENTITY_TYPE_TASK`: creation, attributes, lists, pointers);
  - the task database;
  - the group-to-task suitability table;
  - a keysite-selection scorer;
  - a sector-walking difficulty assessment;
  - two new replication messages.
- **It also has two fidelity problems that need a decision before porting.** See F1 (uninitialised route heights) and F2 (an airbase picking itself as its own supplier).

## The call graph

Senders of the message:
- `keysite.c:483/494` (Slice 4, ammo or fuel);
- `group.c:653/666` (Slice 1, ammo or fuel, `RESUPPLY_SOURCE_GROUP`).

In both cases the argument is `ENTITY_SUB_TYPE_CARGO_AMMO` or `_FUEL`, so the unguarded `switch (sub_type)` (`fc_msgs.c:752`) always assigns `factory`.

```
fc_msgs.c:672  response_to_force_low_on_supplies (receiver = force, sender = keysite | group, sub_type)
├─ get_game_status () != INITIALISED || get_comms_model () == CLIENT  → return FALSE        [ported: core state; comms]
├─ task.c:618  entity_is_object_of_task (sender, TASK_SUPPLY, sender side)                     [NEW]
│    └─ walk sender LIST_TYPE_TASK_DEPENDENT; count tasks of type/side with TASK_STATE != COMPLETED
├─ walk sender LIST_TYPE_TASK_DEPENDENT for a SUPPLY task with TASK_USER_DATA == sub_type → return FALSE
├─ get_keysite_supply_position (sender)  = sender VEC3D_TYPE_POSITION                         [ported: keysite and group position]
├─ keysite.c:88  get_closest_keysite (FACTORY | OIL_REFINERY, side, pos, 10 km, &range, TRUE, NULL)   [ported, Slice 1]
├─ get_closest_keysite (AIRBASE, …)  → airbase replaces factory if strictly closer
├─ walk supplier LIST_TYPE_CARGO for the first crate of sub_type                              [ported, Slice 3]
└─ taskgen.c:1638  create_supply_task (sender, supplier, cargo, MOVEMENT_TYPE_AIR, task_database[SUPPLY].task_priority = 4, NULL, NULL)
     ├─ taskgen.c:2583  get_task_start_keysite                                               [NEW]
     │    └─ task.c:1030  find_most_suitable_keysite_for_task (SUPPLY, side, crate pos, check_capacity = TRUE)
     │         ├─ force LIST_TYPE_KEYSITE_FORCE: IN_USE, has LIST_TYPE_KEYSITE_GROUP, LANDING_TYPES & task landing types
     │         ├─ keysite_database[].air_force_capacity >= task_database[SUPPLY].keysite_air_force_capacity (LARGE)
     │         ├─ per group: suitable.c:212 get_group_to_task_suitability  (group_task_array, built by
     │         │   calculate_group_to_task_suitability from group_database and task_database ai_stats), ALIVE, GROUP_MODE
     │         ├─ get_approx_2d_range; range bias (x^4), LIST_TYPE_UNASSIGNED_TASK count bias, KEYSITE_USABLE_STATE
     │         └─ float scoring: all declared float, so the #7 RTZ contract applies
     ├─ start_ks == requester → return NULL
     ├─ vector.c:208  normalise_any_3d_vector (stop − crate)                                 [NEW: float sqrt, 1/length]
     ├─ prepare / finish = stop ∓ direction · 4 / 2 km (x, z only), en_world.c:166 bound_position_to_adjusted_map_area
     └─ taskgen.c:114  create_task (… 4 route nodes + terminator {-1,-1,-1})
          ├─ validate_task_generation → TRUE (body compiled out, #if 0)
          ├─ force_raw->task_generation[SUPPLY].created ++  → task id (wraps at NUM_TASK_ID_BITS)   [NEW force field]
          ├─ route nodes: ceil () of x, y, z of every node  → heap copies (4 arrays)
          ├─ create_client_server_entity (ENTITY_TYPE_TASK, PARENT (TASK_DEPENDENT, requester), INT/FLOAT attrs)   [NEW type]
          │    └─ ts_creat.c :: create_local: memset, side UNINITIALISED, state UNASSIGNED; joins LIST_TYPE_UPDATE
          │       and LIST_TYPE_TASK_DEPENDENT of the requester
          ├─ ENTITY_COMMS_SET_TASK_POINTERS (route nodes, formations, waypoint types, dependents, return keysite)   [NEW message]
          ├─ task_database[SUPPLY].primary_task → set_client_server_entity_parent (LIST_TYPE_UNASSIGNED_TASK, start_ks)
          ├─ task.c:708  assess_task_difficulty → INT_TYPE_TASK_DIFFICULTY                   [NEW]
          │    └─ Bresenham over sectors from start_ks along the route; sector.c
          │       surface_to_air_defence_level summed over the other sides, and INT_TYPE_SECTOR_SIDE
          ├─ ENTITY_MESSAGE_TASK_CREATED only if the objective's side != task side  → never for supply (same side)
          └─ insert into LIST_TYPE_SECTOR_TASK of the objective's sector                     ← end of the synchronous path
     └─ set_local_entity_float_value (TASK_USER_DATA, cargo sub type)
```

## Fidelity findings (need a decision before porting)

### F1: route heights are read uninitialised (proven by source)

`create_supply_task` declares `vec3d finish, prepare` (`taskgen.c:1644`) and assigns only `.x` and `.z` (`:1688–1695`). `bound_position_to_adjusted_map_area` also touches only x and z (`en_world.c:166`).

`create_task` then reads `position->y` for every node:
- `ceil (position->y)` at `taskgen.c:240`;
- the terminator comparison at `:271`.

The indeterminate heights are stored as route nodes 1 and 3, packed into `ENTITY_COMMS_SET_TASK_POINTERS` and sent to clients.

The historical value is whatever the stack held, which depends on the executable. No C oracle can reproduce it meaningfully. The harness would give a value that means nothing, or one the build happens to leave.

The port needs a declared choice. Options:
- a deterministic stand-in, recorded as a fidelity deviation;
- a value the harness can be made to agree on, for example by zeroing the frame and documenting that.

The same x87 caveat as #9 applies to how `ceil` receives the float.

### F2: an airbase supplies itself (proven by source; behaviour to preserve)

`get_closest_keysite (AIRBASE, side, pos, 10 km, …, exclude = NULL)` searches from the requester's own position. An in-use airbase requester is therefore always found by the early exit at range 0 (`keysite.c:127`), so `airbase_actual_range = 0`. It replaces any factory that is not also at range exactly 0, and the requester becomes its own supplier.

What happens next:
1. The cargo search takes the head of the requester's own cargo list for that type, which is its newest crate (Slice 4). A requester below one crate's worth (level < 10) holds no crate of that type, so the search finds nothing.
2. `create_supply_task` picks a start keysite near that crate. The requester's own range factor is the maximum (range 0), so when it has a suitable idle group it usually scores best (group count, unassigned-task count and usable state also weigh in), which gives `start_ks == requester` and returns NULL.
3. Only when the requester has no suitable transport group does another keysite get a task that flies the requester's own crate back to it.
4. With no crate of the type, nothing is created.

So in practice an airbase almost never raises a supply task through this path. This is original behaviour: it must be reproduced and tested, not corrected.

Requesters, from the generated keysite database, are those with negative usage:
- airbase (ammo and fuel);
- FARP (ammo and fuel);
- military base (ammo and fuel);
- power station (fuel);
- plus ground groups with `RESUPPLY_SOURCE_GROUP`.

### Smaller original behaviours to preserve

- **Duplicate-task guard asymmetry.** The outer test (`entity_is_object_of_task`) counts supply tasks that are *not* completed, of the sender's side. The inner loop matches *any* supply task by `TASK_USER_DATA == sub_type`, completed ones included, of any side. Example: a completed ammo task plus an active fuel task blocks a new ammo request.
- **Mixed range metrics.** `get_closest_keysite` returns the approximate range on its ≤ 10 km early exit, but the exact `get_2d_range` otherwise. The factory/airbase comparison `airbase_actual_range < factory_actual_range` can therefore compare an approximate range with an exact one.
- **FLT_MAX when nothing is found.** When no keysite of a type exists, `actual_range` is `FLT_MAX`. Two misses leave `factory` NULL, and nothing is created.
- **Return value ignored.** Both senders discard the response's return value.
- **Zero-length direction.** When the crate and the stop position have equal x, y and z, `normalise_any_3d_vector` zeroes the direction and prepare = finish = stop, after bounding. When the picked crate is the ammo crate at x0 (the keysite holds exactly one ammo crate, and the supplier is the requester), x and z are equal and only y differs (`− ymin`). The unit vector is then purely vertical, so again prepare = finish = stop in x and z.

## Existing infrastructure against what is new

| Needed | State |
|---|---|
| game status, comms model, force lookup, message dispatch | ported (Slices 1, 4) |
| `get_closest_keysite`, `get_approx_2d_range`, `get_2d_range` | ported (Slice 1) |
| keysite/group position, cargo list, sector lookup, world map bounds | ported (Slices 1, 3) |
| `ENTITY_TYPE_TASK`: raw struct, `ts_creat`, `ts_int`, `ts_float`, `ts_list`, `ts_ptr` (route pointers) | **new** |
| `task_database` (supply row, plus the fields the scorer reads for any task) | **new**, generate from `ts_dbase.c` |
| `group_task_array` / `calculate_group_to_task_suitability` (group and task `ai_stats`) | **new**; the group database is partly generated already |
| keysite `LANDING_TYPES`, `KEYSITE_USABLE_STATE`, `air_force_capacity`; group `GROUP_MODE`, `ALIVE` | **new** accessors or columns |
| force `task_generation[]` counters | **new** force field |
| `normalise_any_3d_vector`, `bound_position_to_adjusted_map_area`, `ceil` | **new**, small |
| `assess_task_difficulty` (sector SAM levels, sector side, Bresenham) | **new** |
| sector `LIST_TYPE_SECTOR_TASK`, keysite `LIST_TYPE_UNASSIGNED_TASK`, `LIST_TYPE_TASK_DEPENDENT` | **new** list roots and links |
| `ENTITY_COMMS_SET_TASK_POINTERS`, client-server parent switch for tasks | **new** replication |
| harness: `fc_msgs.c`, `taskgen.c`, `task.c`, `ts_*.c`, `suitable.c` compiled whole | **new**; today the harness overrides the response with `record_force_low_on_supplies` |

## Consequence for the frozen fixtures

Slices 1 and 4 record the message at the boundary: the harness prints `message force0 keysite0 …`, and TypeScript overloads an unported response. Porting the real response means changing that boundary. The recorded message line is the input to this slice, so keeping it as a trace line alongside the real response leaves every frozen fixture byte-identical. That is the approach proposed here.

## Suggested split

The path is bounded but broad. A proposed split, to agree before any porting:
- **5a.** The response's decision logic, up to the `create_supply_task` call:
  - guard, duplicate check, supplier selection and cargo pick, including F2;
  - `create_supply_task` recorded at a boundary with its arguments.
- **5b.** Task creation: `create_supply_task` / `create_task` / the task entity, with F1 decided first, then `find_most_suitable_keysite_for_task` and `assess_task_difficulty`.
