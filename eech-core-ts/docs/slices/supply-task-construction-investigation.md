# Slice 5b investigation: supply task construction (issue #14)

Status: **investigation only, nothing ported.** Traced against master at `e014e6c`, where Slice 5a (PR #13) is frozen.

The issue sets two gates, in order:
- **Gate 1:** F1, the uninitialised route heights.
- **Gate 2:** the whole `create_supply_task` → `create_task` path, traced before scope is fixed.

This document records both. The F1 compatibility choice at the end is a **proposal for review**. Nothing is implemented until it is agreed.

## Gate 1: F1, `prepare.y` and `finish.y`

### The source facts

- `create_supply_task` declares `vec3d finish, prepare, direction` as automatic variables (`taskgen.c:1644`).
- It writes `prepare.x`, `prepare.z`, `finish.x` and `finish.z` (`:1688–1695`). `bound_position_to_adjusted_map_area` also touches only x and z (`en_world.c:166`). Nothing writes `.y`. (`direction.y` *is* computed, from `stop->y - start->y`, but it is not copied into either point.)
- Both points are passed by address to `create_task` as route nodes 1 and 3.
- The addresses are taken, so C does not make the read itself undefined. It reads an **indeterminate value**: an unspecified float bit pattern, possibly a signalling NaN.

### 1. Can the historical build give the value a reproducible meaning?

No meaning can be recovered from the source, and there is no single historical build to recover it from.

- **Where the bits come from.**
  - `prepare` and `finish` live in `create_supply_task`'s own frame, and x86 stacks grow downwards.
  - So the calls `create_supply_task` makes before writing the points cannot overwrite that y slot: `get_task_start_keysite` → `find_most_suitable_keysite_for_task`, and `normalise_any_3d_vector`. Their frames sit at lower addresses.
  - The bits are therefore whatever an *earlier sibling* call left at that address. That means the frames of `response_to_force_low_on_supplies`'s own callees: `entity_is_object_of_task`, the `get_closest_keysite` calls, and their callees.
  - Which local of which callee overlaps the slot is a property of the compiled frames. The bits it holds are that callee's data: a range, a pointer, a saved register.
- **The build decides the layout, and there is no one build.**
  - `EECH-MSVC.vcproj` Release uses `/O2` with **whole-program optimisation** (LTCG). With LTCG, inlining and frame layout across `fc_msgs.c`, `taskgen.c` and `keysite.c` are link-time decisions: `create_supply_task` may be inlined into the response, and locals may share slots.
  - The Debug configuration has `BasicRuntimeChecks="0"`, so there is no `/RTC` fill either.
  - The repository also carries an autotools/GCC build (`configure.in`, `Makefile.am`).
  - The original Razorworks releases are another toolchain again (`docs/fidelity/fpu-semantics.md` §2.1).
  - No shipped executable is in the repository.
- **Conclusion.** Even within one executable, the value is a function of unrelated earlier computations on the same call path. Across executables it is not even that. There is no stable EECH semantic value to reproduce.

The planned demonstration, once `taskgen.c` is compiled in the harness, is to build the original `create_supply_task` with GCC's `-ftrivial-auto-var-init=pattern`, with `=zero`, and with neither, at `-O0` and `-O2`. The route nodes 1 and 3 it emits should differ between those builds. That is evidence that the value is build-dependent. It is never presented as the historical value.

### 2. Is it overwritten or normalised before it becomes campaign-observable?

No. It is observable, and partly observable on a build-dependent path.

| Where | What happens to y | Observable? |
|---|---|---|
| `create_task` (`taskgen.c:240`) | `route_node_list [n].y = ceil (position->y)`; copied to the task's heap route arrays | stored task state |
| terminator test (`:271`) | compared with −1.0 | see 3 below |
| `assess_task_difficulty` (`task.c:708`) | not read: sector walk on x and z only | no |
| `ENTITY_COMMS_SET_TASK_POINTERS` (`en_comms.c:2409`) | **single player:** `transmit_entity_comms_message` returns before packing (`DIRECT_PLAY_COMMS_MODE_NONE`), so the node keeps its value. **Multiplayer:** `pack_vec3d (VEC3D_TYPE_POSITION)` asserts `point_inside_map_volume (v) \|\| v->y == -10000`, which fails in a debug build. A release build **clamps the route node in place** to `[-8000, 65535]` (`bound_position_to_map_volume`; a NaN passes through unclamped), then sends `y * 16` truncated to 21 signed bits | yes, sent to clients; and the server's stored value now depends on whether it was a multiplayer session |
| saved game (`ts_pack.c:137/406`) | the same `pack_vec3d`: clamp in place, then pack | yes, saved and restored |
| task assignment: `croute.c :: create_generic_waypoint_route` (`:355`) | `ceil` again, then route generation between nodes | yes: see the next two rows |
| route checksum (`croute.c:1074`) | `check_sum += (int) route->position.y` for the interior nodes. Converting an out-of-range float or a NaN to `int` is **undefined** in C (x87 `fistp` gives `0x80000000`) | yes: `INT_TYPE_ROUTE_CHECK_SUM` is replicated, and clients compare it with their own |
| waypoint creation (`croute.c:623`) | `bound (y, 1, MAX_MAP_Y - 1)` becomes the waypoint's `VEC3D_TYPE_POSITION` y (a NaN passes through) | yes: replicated and saved |
| helicopter guidance (`hc_move.c :: get_waypoint_display_position`) | replaces y with the terrain elevation | no effect on flight |
| campaign map preview (`map.c:851 → croute.c :: temp_create_generic_waypoint_route`) | the same route construction from the stored nodes, for drawing an unassigned task's route | UI |

So the only consumer that throws the value away is flight guidance. The route checksum, the replicated and saved route nodes, and the waypoints all carry it. The multiplayer pack even rewrites the stored value.

### 3. Can the terminator test depend on it?

The loop in `create_task` ends when a node compares equal to `terminator_point {-1, -1, -1}` in x **and** y **and** z (`:271`).

- **In practice, no.** `prepare.x` and `finish.x` come out of `bound_position_to_adjusted_map_area`. They are either their computed value, or `MIN_MAP_X + MAP_PERIMETER_SIZE` = 5000, or `MAX_MAP_X − 5000`. The same holds for z.
- **Only on a pathological map.** They can equal −1 only when `MAX_MAP_X − 5000 = −1`, that is when `num_map_x_sectors × sector_side_length = 5000`, and the same for z. Only on such a map does the indeterminate y decide whether the route ends early.
- No EECH campaign map has that size, so on every real map termination does not depend on y. The port should still keep the three-way comparison as written.

### Classification and proposed compatibility choice

**Classification.** `prepare.y` and `finish.y` are **undefined source behaviour**: indeterminate values whose bits depend on the build and the call history. They are campaign-observable through:
- the route checksum (where the int conversion can itself be undefined);
- the replicated and saved route nodes (clamped in place only in multiplayer);
- the waypoints.

There is no C oracle for them, and none should be manufactured.

**Proposed compatibility choice:** `prepare.y = finish.y = 0.0`, as if the frame were zero-initialised.

- It is a documented deviation, recorded in the port manifest and the slice doc as a compatibility value, not source fidelity.
- It keeps every downstream use defined:
  - `ceil (0) = 0`;
  - inside the map volume, so the multiplayer pack neither clamps nor diverges from single player;
  - `(int) 0.0` adds 0 to the checksum with no undefined conversion;
  - waypoints bound it to 1;
  - flight ignores it.
- **How the C reference agrees without claiming history.** The harness compiles `taskgen.c` with `-ftrivial-auto-var-init=zero`. That is a compiler setting on unmodified source, labelled in the harness and the docs as *pinning the compatibility choice*.
- **How the choice stays honest.** A separate investigation probe (above) shows that other builds give other values.
- **Alternatives considered:**
  - **`stop->y`** (the drop-off height). It would invent a semantic the source never states, and the C reference could only agree by editing the source.
  - **Keeping the value "indeterminate" and failing loudly when observed.** Slice 5b itself replicates the route nodes (`SET_TASK_POINTERS`), so every multiplayer supply task would fail. It only defers the choice.

## Gate 2: `create_supply_task` → `create_task`, traced whole

```
taskgen.c:1638 create_supply_task (requester, supplier, cargo, MOVEMENT_TYPE_AIR, 4.0, NULL, NULL)
├─ stop = requester VEC3D_TYPE_POSITION; start = cargo VEC3D_TYPE_POSITION; side = requester INT_TYPE_SIDE
├─ taskgen.c:2583 get_task_start_keysite (SUPPLY, side, start, &start_ks)
│    └─ task_database[SUPPLY].primary_task, landing types (no GROUND) → check_capacity TRUE
│       task.c:1030 find_most_suitable_keysite_for_task (SUPPLY, side, crate position, TRUE)
│         ASSERT server; for each keysite of the side's force (LIST_TYPE_KEYSITE_FORCE order):
│           IN_USE (ks_int), has LIST_TYPE_KEYSITE_GROUP children, keysite INT_TYPE_LANDING_TYPES (raw) & task landing types,
│           keysite_database[].air_force_capacity >= task_database[SUPPLY].keysite_air_force_capacity (LARGE)
│           group score: suitable.c get_group_to_task_suitability (group_task_array, built at start-up by
│             calculate_group_to_task_suitability from group_database and task_database: movement type, default
│             landing type, ai_stats, engage enemy), group INT_TYPE_ALIVE, INT_TYPE_GROUP_MODE (guide stack):
│             idle 5.0, busy 0.5, capped at 12.0; no score → skip
│           range bias: get_approx_2d_range; >= 100 km → skip; score *= 2 (range / 100 km − 1)^4
│           task bias: count of this task type on LIST_TYPE_UNASSIGNED_TASK; 1 − 0.2 n, floor 0.2
│           KEYSITE_USABLE_STATE != USABLE → × 0.5; strictly greater score wins
│    (all float, under the #7 RTZ contract; #9 caveat)
├─ no start keysite, or start_ks == requester → return NULL                 ← F2's usual end
├─ direction = stop − start (float); vector.c normalise_any_3d_vector (float sqrt, 1 / length; zero vector → 0)
├─ prepare / finish x, z = stop ∓ direction · 4 / 2 km (double, stored float); bound_position_to_adjusted_map_area
│    prepare.y, finish.y: indeterminate (Gate 1)
└─ taskgen.c:114 create_task (SUPPLY, side, AIR, start_ks, NULL, NULL, TRUE, 20 min, 0.0, requester, 4.0,
                              start/supplier/PICK_UP, &prepare/NULL/PREPARE_FOR_DROP_OFF, stop/requester/DROP_OFF,
                              &finish/NULL/FINISH_DROP_OFF, &terminator_point/NULL/NUM…/FORMATION_NONE)
     ├─ debug_assert server; validate_task_generation → TRUE (#if 0)
     ├─ force_raw->task_generation [SUPPLY].created ++                     [force state, never replicated here]
     ├─ route arrays: ceil of every x, y, z; dependents supplier, NULL, requester, NULL; waypoint and formation types;
     │    4 heap arrays of route_length + 1 (terminator included); route_length = 4
     ├─ id = created, wrapped by 4095 (not 4096: ids never return to 0) into the 12-bit task_id
     ├─ create_client_server_entity (ENTITY_TYPE_TASK, DONT_CARE, PARENT (TASK_DEPENDENT, requester),
     │    SUB_TYPE, TASK_ID, EXPIRE_TIMER 1200, TASK_PRIORITY 4, CRITICAL_TASK 1, MOVEMENT_TYPE, ROUTE_LENGTH 4, SIDE)
     │    ts_creat.c create_server → create_local: memset, side UNINITIALISED, state UNASSIGNED, attributes;
     │      insert LIST_TYPE_UPDATE (update entity), LIST_TYPE_TASK_DEPENDENT (requester)
     │      LINK_CHILD to the update entity (default), keysite (DEBUG-only log) or group (MEMBER only): no effect
     │      LINK_PARENT to the task: only the ASSIGNED / UNASSIGNED / COMPLETED cases act: no effect
     │    → create_remote: transmit ENTITY_COMMS_CREATE                     [replication 1]
     ├─ stop_timer 0.0 → not set
     ├─ set_local_entity_ptr_value ×5 (ts_ptr.c): route dependents, nodes, waypoint types, formation types, return keysite NULL
     ├─ transmit ENTITY_COMMS_SET_TASK_POINTERS (route nodes 0..3 packed as positions; see Gate 1) [replication 2]
     ├─ primary_task → set_client_server_entity_parent (task, LIST_TYPE_UNASSIGNED_TASK, start_ks) (en_list.c:841)
     │    delete (not yet in a list: no-op), insert into start_ks: LINK_CHILD (no effect);
     │    LINK_PARENT → ts_msgs.c: INT_TYPE_TASK_STATE = UNASSIGNED, and
     │      ca_msgs.c notify_campaign_screen (CAMPAIGN_SCREEN_MISSION_CREATED, task)   [UI boundary]
     │        returns unless game status INITIALISED and game type CAMPAIGN or SKIRMISH; then every
     │        campaign-screen target's response (UI lists)
     │    transmit ENTITY_COMMS_SWITCH_PARENT                                [replication 3]
     ├─ task.c:708 assess_task_difficulty → INT_TYPE_TASK_DIFFICULTY (local set, not replicated)
     │    from start_ks (task_link.parent) through nodes 0..3, Bresenham over sectors (get_x/z_sector: truncation);
     │    per sector: enemy surface-to-air defence level (sum over the other sides) > 0, INT_TYPE_SECTOR_SIDE != side;
     │    then the last node again; min (n >> 1, 5) each; the task-type rating is computed but not used
     ├─ ENTITY_MESSAGE_TASK_CREATED only if the objective's side != task side: never for supply
     └─ get_local_sector_entity (requester position); insert LIST_TYPE_SECTOR_TASK (local, no transmit; link responses no-ops)
  set_local_entity_float_value (TASK_USER_DATA, cargo sub type)            (local: never replicated)
```

### What the trace settles

- **The natural boundary is `create_task`'s return.** Nothing past it is synchronous. Waypoints are created only when the task is assigned (`croute.c`, from `assign.c`, and on clients from `en_comms.c` when they receive the assignment). The task's own update (`ts_updt.c`: the 20-minute expiry) runs later from the update list.
- **Route-node ownership.**
  - The task owns four heap arrays; `ts_dstry.c` frees them. `route_length` is 4, and the terminator slot is allocated but excluded from the length.
  - The dependents are the supplier (pick-up node) and the requester (drop-off node).
  - The drop-off *waypoint* that later sits on the requester's `LIST_TYPE_TASK_DEPENDENT` list, beside the task, is created at assignment. That is where 5a's RECON finding meets real data.
- **Only one new boundary.** Inside the path the only new external dependency is `notify_campaign_screen`. It is UI and belongs behind a port (a `CampaignScreen` observer), reached only under the game-status and game-type guard. Everything else is campaign state, database tables, or replication through the existing port.
- **Replication ordering.** `ENTITY_COMMS_CREATE` → `ENTITY_COMMS_SET_TASK_POINTERS` → `ENTITY_COMMS_SWITCH_PARENT`. The difficulty, the sector list and `TASK_USER_DATA` are local-only, so clients never receive the task's cargo sub type or difficulty on this path. That is original behaviour.
- **Force task counters.** Only `task_generation [SUPPLY].created` changes. It drives the task id, and the id wraps by 4095.
- **The self-supplying airbase (F2)** ends at `start_ks == requester` whenever the requester scores best from its own crate. That happens when it has a suitable alive group, capacity LARGE, and a matching landing type. Otherwise another keysite gets the task, and `direction` is the (vertical or zero) vector from the crate to the requester. For the ammo crate at x0, with the requester as its own supplier, `direction` is purely vertical, so prepare and finish sit at the stop position in x and z (after bounding).
- **Another database-driven table.** `group_task_array` is computed once at start-up (`highlevl.c :: initialise_group_task_array`). It becomes generated or initialised core state.

## Proposed scope for Slice 5b (for review)

This follows the C: everything from `create_supply_task` to `create_task`'s return, with no further split. Every piece is on the one synchronous path, and none of it can be observed separately. The pieces:

- **`create_supply_task`.** Including `normalise_any_3d_vector` and `bound_position_to_adjusted_map_area`, and F1 as decided above.
- **`get_task_start_keysite` and `find_most_suitable_keysite_for_task`.** Plus `group_task_array` and `calculate_group_to_task_suitability`, and the group and task database columns they read (generated).
- **`create_task` and task entity creation.** `ts_creat.c`; the `ts_int.c`, `ts_float.c` and `ts_ptr.c` setters it reaches; the task's other list links (update, unassigned, sector task); and the id and force counter.
- **`assess_task_difficulty`.** Plus the sector state it reads: the surface-to-air defence levels and the sector side.
- **Replication.** `ENTITY_COMMS_SET_TASK_POINTERS` and `ENTITY_COMMS_SWITCH_PARENT`, through the existing `EntityReplication` port.
- **A `CampaignScreen` port** for `notify_campaign_screen`. It is UI: the core reports the message, and the host decides. Plus the host's game type as core state beside the game status.

**Out of scope:**
- task assignment (`assign.c`) and route and waypoint construction (`croute.c`);
- expiry and termination (`ts_updt.c`, `ts_msgs.c`);
- `ts_pack.c` and task destruction.
