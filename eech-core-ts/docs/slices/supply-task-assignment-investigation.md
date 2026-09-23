# Slice 6 investigation: supply-task assignment (issue #16)

Status: **investigation only, nothing ported.** Traced from frozen master `f5ad93f` (Slice 5b). Line numbers are in `aphavoc/source`.

## Summary

- **The decision is separable.** It chooses which unassigned task and which group:
  - `assign_keysite_tasks` orders the tasks;
  - `get_suitable_registered_group` chooses the group.

  It is deterministic, has no randomness or clock, and ends in a single call, `assign_primary_task_to_group (group, task)`. That call is a natural boundary, like Slice 5a's `create_supply_task`.
- **The transaction is not separable.** The source only moves a task to `LIST_TYPE_ASSIGNED_TASK` after two things have succeeded:
  - building its waypoint route (`create_generic_waypoint_route`);
  - creating and attaching a guide (`push_task_onto_group_task_stack`).

  Straight after the list switch it attaches every member and sends each one `TASK_ASSIGNED`. For a landed helicopter, that starts the takeoff machinery.

  There is no point in the C where the supply task is ASSIGNED without a guide and waypoints. The preferred boundary ("task becomes ASSIGNED, then return") therefore does not exist in the source.
- **The transaction needs two external data sources the core does not have:**
  - terrain elevation, which the route search samples;
  - the road network, which gives every waypoint a road node.
- **Proposal:** a split.
  - **6a:** the assignment decision, up to the `assign_primary_task_to_group` boundary.
  - **6b and later:** the transaction, itself likely split (see "Proposed split").

## Gate 1: the entry point

`ks_updt.c :: update_server` (keysite, server) runs assignment:

```
alive? → assist_timer -= dt → assign_timer -= dt
  assign_timer <= 0:
    in_use: assign_keysite_tasks (RECON), (STRIKE), (SUPPORT)        ks_updt.c:113-115
    assign_timer = KEYSITE_TASK_ASSIGN_TIMER (3 min), ×2 unless USABLE
→ sleep timer: supply drift + update_keysite_cargo (Slice 4) + bound
→ repair branch / else a function-static task_timer (shared by all keysites) → create_repair_task
```

- **Which keysites run it:** every alive, in-use keysite, on its own 3-minute timer (6 minutes when not usable).
- **Supply tasks:** they are `TASK_CATEGORY_SUPPORT`, so the third call handles them.
- **The rest of `update_server` is not assignment.** It also covers supply drift, repair and the repair-task generator, and it shares a `static float task_timer` across all keysites. Porting the keysite update is therefore its own, later composition.
- **Proposed 6a entry point:** `assign_keysite_tasks (keysite, category)` itself. It is the adopted function, the way Slice 5a adopted the force response rather than the keysite update.

`assign.c :: assign_keysite_tasks` (`:96`):

1. It returns if the keysite has no unassigned task, or none of the category (`INT_TYPE_TASK_CATEGORY` from `task_database`).
2. It counts the force's idle groups by sub type over `LIST_TYPE_AIR_REGISTRY` (`GROUP_MODE_IDLE`). The count array is `static`, but it is `memset` on every call, so nothing carries over between calls.
3. It collects that category's unassigned tasks. The sort key is `FLOAT_TYPE_TASK_PRIORITY`, doubled for a critical task.
4. It sorts with `en_misc.c :: quicksort_entity_list` (`:212`). This is a hand-written, **non-stable**, **descending** quicksort with a middle pivot. Equal priorities come out in an order set by the algorithm, not by list order, so the port must use the same algorithm.
5. It walks the sorted tasks until `assign_count` reaches 0.
   - `assign_count = max (keysite_database [type].assign_task_count, 1u)`, a 2-bit field: the airbase has 3.
   - The walk skips a pilot-locked task.
   - It reserves `reserve_task_count` non-critical tasks whose expiry is above 3 minutes for the player. The airbase reserves 2. Supply tasks are **critical**, so they are never reserved.
6. For each remaining task, `get_suitable_registered_group`, then `assign_primary_task_to_group`. A TRUE result decrements `assign_count`; a FALSE result goes on to the next task.

**It stops without assigning when:**
- there is no task of the category;
- every task is pilot-locked or reserved;
- no group qualifies;
- or `assign_primary_task_to_group` fails.

That last call fails in three cases:
- the group has no members;
- the end keysite lacks landing sites;
- or the route cannot be built.

**Determinism.** No randomness or clock is involved. The result is a pure function of the entity graph and the databases.

## Gate 2: task selection

For a supply task the inputs are:
- category SUPPORT;
- `task_priority` 4.0, doubled because it is critical: 8.0;
- the pilot lock (`LIST_TYPE_PILOT_LOCK` parent);
- the expire timer, for the reserve rule; a critical task skips it;
- membership of the keysite's unassigned list.

Nothing else is read: not the objective, the movement type or the landing types.

**Ties** between equal sort keys (every supply task at the keysite scores 8.0) come out in `qs`'s order, not in list order. That quirk must be pinned.

## Gate 3: group selection

`assign.c :: get_suitable_registered_group` (`:401`) considers **only the groups of the task's own unassigned-list keysite** (`LIST_TYPE_KEYSITE_GROUP`), in list order. A group qualifies only if all of these hold:

1. it is not pilot-locked;
2. it is not `ASSAULT_SHIP`;
3. it is idle (no guide on its stack);
4. its raw `sleep == 0.0`;
5. it is on the task's side;
6. `idle_group_count [sub type] > group_database [].minimum_idle_count`. That is 0 for the medium lift helicopter, so at least one idle group of the type must be registered;
7. `INT_TYPE_MEMBER_COUNT` is at least the task's minimum member count (1);
8. `check_group_members_awake`: no member has `FLOAT_TYPE_SLEEP > 0`;
9. the suitability table (Slice 5b, reused) gives more than 0.0;
10. `suitable_group_task_specific_checks` passes. It only has ESCORT and TROOP_INSERTION arms; supply passes;
11. `group.c :: assess_group_task_locality_factor` passes. It uses the first member's position and `FLOAT_TYPE_CRUISE_VELOCITY`, divides the approximate range to the keysite by that velocity, and rejects if the ETA exceeds the task's expire timer (1200 s).

**Quirks to pin:**
- **The least suitable group wins.** The comparison is `result < best_result` (`:497`), with `best_result` starting at `FLT_MAX`. The first group in list order wins ties.
- **Alive is never checked.** A dead group is still a candidate. Slice 5b's start-keysite scoring did check alive.
- **The idle count is force-wide, from the air registry.** A keysite group that is not itself registered can still qualify, as long as another idle group of its type is registered. A registered group elsewhere counts too.
- **The member count is raw.** It is `raw->member_count`, maintained by `gp_msgs.c`'s `LINK_CHILD (LIST_TYPE_MEMBER)` arm, which is unported. A restore sets it raw.
- **The member count and the members can disagree.** `assess_group_task_locality_factor` rejects a group with no members. So does `assign_task_to_group` later.
- **Division by a cruise velocity of 0.** The locality check divides by the member's cruise velocity. With velocity 0, a nonzero distance gives an infinite ETA, and the group is rejected. A zero distance gives NaN, `eta > expire` is false, and the group passes.

**New state this needs.** The group's raw member count and the members' sleep, cruise velocity and position. The members are aircraft: position is already a port, but sleep and cruise velocity are raw mobile/aircraft state (`aircraft_database`). The force also needs its `LIST_TYPE_AIR_REGISTRY` root and the groups their link.

## Gate 4: the assignment transaction, in source order

`assign_primary_task_to_group` (`:524`) calls `assign_task_to_group (group, task, TASK_ASSIGN_ALL_MEMBERS)` (`:765`):

1. It returns FALSE with no first member, for an assault ship, or for LANDING/TAKEOFF tasks.
2. `start_keysite` is the group's keysite, when its list type is `KEYSITE_GROUP`.
3. **Landing assessment.** Supply sets `ASSESS_LANDING` TRUE and has no end keysite:
   - `end_keysite = start_keysite`, written into `PTR_TYPE_RETURN_KEYSITE`;
   - `landing = get_local_entity_landing_entity (end_keysite, landing type)`;
   - a group without a keysite takes the closest keysite and then checks its landing sites.
4. **`croute.c :: create_generic_waypoint_route`** (`:200`, `:927`):
   - it builds the specified route from the task's route nodes, re-`ceil`ing each. **F1's 0.0 heights enter here;**
   - `add_start_waypoint` (TRUE for supply) prepends the first member's position;
   - it appends a LAND waypoint at the return keysite;
   - `task_route_search` (TRUE for supply) runs **`generate_biased_vec3d_route`** (`:538`). That builds a sub-route per leg, choosing points by **terrain elevation** samples (`get_3d_terrain_elevation`) and **sector side** bias (`route_biasing_database`);
   - it sets `INT_TYPE_ROUTE_CHECK_SUM`, a checksum over the node positions, with `set_client_server`, so it is transmitted;
   - it creates one **waypoint entity** per node (`create_local_entity (WAYPOINT, …)`). Each waypoint is bounded to `[1, MAX_MAP − 1]`, so F1's 0.0 height becomes 1. Each gets `get_3d_terrain_elevation` and **`get_closest_road_node`** (road network) and a flight time from the member's cruise velocity. The specified nodes get their types, and they are **inserted into their dependents' `LIST_TYPE_TASK_DEPENDENT` lists** (`:725`). After 6b, supply routes therefore feed Slice 5a's duplicate-task walk;
   - `parser_task_waypoint_route` merges waypoints that are too close (waypoint database);
   - it sends `ENTITY_COMMS_CREATE_WAYPOINT_ROUTE` (`:823`).
5. **`push_task_onto_group_task_stack`** (`:659`):
   - `create_client_server_guide_entity` → `ENTITY_COMMS_CREATE`;
   - `attach_group_to_guide_entity`: the guide stack insert and `ENTITY_COMMS_SWITCH_LIST`. The **group becomes BUSY**. It also runs `initialise_guide_criteria` and sets the guide velocity from the waypoint database and the cruise velocity (a transmitted float);
   - **then** the task moves off the unassigned list and onto the assigned list. The insert fires `LINK_PARENT (ASSIGNED)`, which sets `TASK_STATE_ASSIGNED` and sends `MISSION_ASSIGNED` to the campaign screen (the arm Slice 5b leaves fail-loud). Then `ENTITY_COMMS_SWITCH_LIST` (`:754`).
6. **`assign_task_to_group_members`**. For each valid member:
   - `attach_group_member_to_guide_entity`: the follower list and `SWITCH_LIST`;
   - `notify TASK_ASSIGNED` to the member: **`mb_msgs.c :: response_to_task_assigned`** (`:160`). For a landed aircraft this covers:
     - the landing entity from its keysite and the landing entity's persistent TAKEOFF task;
     - the weapon configuration;
     - `LOCK_TAKEOFF_ROUTE`, then inserting into the takeoff route or the takeoff queue;
     - `RESERVE_LANDING_SITE` at the end keysite;
     - the selected weapon;
     - the view interest level;
   - for a helicopter, `prepare_helicopter_for_task` (troop insertion only) and `set_helicopter_fuel_level`. The latter covers the estimated route duration, fuel economy and default weight, and a transmitted fuel level.

Back in `assign_primary_task_to_group`:

7. The group's default formation (transmitted when it changes).
8. The group is re-parented to the return keysite, which for supply is its own keysite (`set_client_server_entity_parent` → `SWITCH_PARENT`).
9. `notify TASK_ASSIGNED` to the force: `create_task_assigned_reactionary_tasks`, **a no-op for SUPPLY**.
10. The escort check. The supply threshold is 6: `assess_task_difficulty` (Slice 5b), and at 6 or more **`create_escort_task`**, another task generator. It is reachable: the difficulty goes up to 10.
11. `FLOAT_TYPE_START_TIME` = the session's elapsed time (transmitted).
12. The expire timer is set to 0 (local).

**Replication order:**
1. the route checksum;
2. `CREATE_WAYPOINT_ROUTE`;
3. the guide `CREATE`;
4. the guide-stack `SWITCH_LIST`;
5. the guide velocity;
6. the task `SWITCH_LIST`;
7. per member: `SWITCH_LIST`, the takeoff and landing messages, the weapon, the fuel;
8. the formation;
9. `SWITCH_PARENT`;
10. the escort task's messages;
11. the start time.

The campaign-screen `MISSION_ASSIGNED` comes from `LINK_PARENT`, before the task's `SWITCH_LIST`.

## Gate 5: classification

| Behaviour | Class |
|---|---|
| Task ordering, reserve/lock rules, group selection | 1: the decision (separable up to the `assign_primary_task_to_group` call) |
| Waypoint route: specified nodes, start and land waypoints, checksum, waypoint entities, task-dependent links, `CREATE_WAYPOINT_ROUTE` | 1: required before the task can become ASSIGNED |
| Route search (`generate_biased_vec3d_route`) | 1 for supply (`task_route_search`), and it needs **terrain elevation**: class 3, a port |
| Road node per waypoint | 3: the road network is map data, so a port for the data with the search in the core |
| Guide creation, guide stack, velocity, criteria | 1: required; the list switch comes after it |
| `LINK_PARENT (ASSIGNED)`: state and `MISSION_ASSIGNED` | 1 |
| Member attachment and member `TASK_ASSIGNED`: takeoff routes and queue, landing reservation, weapons, fuel | 1 in the C (same call), but a large subsystem of its own (landing entities, takeoff tasks). The candidate for a further split |
| Group formation, re-parent, start time, expire | 1 (small) |
| Force `TASK_ASSIGNED` reaction | 4 for supply (no-op) |
| Escort task creation | 1 when difficulty ≥ 6 (reachable); otherwise 4 |
| Keysite update timers, supply drift, repair | 2: separate composition |

## Gate 6: C-reference feasibility

- **6a (the decision):**
  - `assign.c` is large and mixes the transaction with player requests and reassignment. Extract `assign_keysite_tasks`, `get_suitable_registered_group`, `suitable_group_task_specific_checks` and `check_group_members_awake`, and wrap `assign_primary_task_to_group` as the recording boundary, as 5a did.
  - Extract `en_misc.c :: quicksort_entity_list` and `qs`, and `group.c :: assess_group_task_locality_factor`.
  - The member state needs original accessors for sleep and cruise velocity. The aircraft and helicopter TUs (`ac_float.c`, `hc_*`) or verbatim extracts would have to be compiled; the harness currently shims members. This is the main shim-reduction item.
  - The `ks_dbase.c` and `gp_dbase.c` columns are already compiled whole.
  - Feasible.
- **The transaction:**
  - `croute.c` (2048 lines) needs `get_3d_terrain_elevation` and road nodes. They would be scenario-driven harness environment entries, as `bounds` is for the 3D object database;
  - `guide.c` and `gd_*.c` (about 3,000 lines, with attack and cover guides beside them);
  - the waypoint creation and database TUs;
  - `mb_msgs.c` (about 2,900 lines) and the landing entity (`entity/special/landing`), with takeoff tasks, routes and queues;
  - aircraft and helicopter fuel data;
  - `create_escort_task`.
  - Feasible in stages, but several times Slice 5b's size.

## Undefined or compatibility-sensitive behaviour

- **F1 propagates.** `create_generic_waypoint_route` re-reads the route heights for the checksum and the waypoints. Under the accepted 0.0 decision the heights are defined. The waypoint bound makes 0 into 1, and the checksum sees 0. The investigation found **no new uninitialised read** on the decision path.
- **The cruise velocity divisor** in the locality check can be 0 for a stationary member type. That gives IEEE infinity (rejected) or, at zero distance, NaN (accepted). Neither is undefined behaviour. Pin both if reachable.
- **The keysite update's function-static `task_timer`** is shared by all keysites. It is deterministic but order-dependent. It is outside 6a.

## Proposed split (for the technical lead)

**6a: the assignment decision.**
- **Entry point:** `assign_keysite_tasks (keysite, category)`, faithful up to the `assign_primary_task_to_group (group, task)` call.
- **The boundary:** in production it fails loudly; tests observe it with a narrowly scoped interceptor for that one call, which returns FALSE as the harness stub would. With FALSE, the loop goes on to the next task, which the cases pin.
- **What it adds:**
  - task ordering (the non-stable quicksort), the reserve and pilot-lock rules and the assign count;
  - the group filters with their quirks (least suitable wins, no alive check, the force-wide idle count);
  - the minimum raw member, sleep and cruise-velocity state.
- **Acceptance:** the social path of Slices 1–5b → a supply task at an airbase → `assign_keysite_tasks (SUPPORT)` → the observed (task, group) choice.

**6b and later: the transaction**, possibly split again:
- **6b:** route and guide. This covers the `TerrainElevation` and `RoadNetwork` ports, waypoints, the checksum, the guide, the list switch and `MISSION_ASSIGNED`, formation, re-parent, start time and escort. Its new boundary would be the member `TASK_ASSIGNED` message.
- **6c:** member attachment and the takeoff/landing machinery.

The alternative is one slice covering the whole transaction. That goes against the issue's "do not silently broaden", so it is only listed here.

**The decision needed:** adopt the 6a / 6b / 6c split with the boundaries above, or say where the boundary should be instead.
