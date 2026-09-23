# Slice 6b investigation: route, waypoint and guide construction through the ASSIGNED transition (issue #18)

Base: `master` at `81ed32e` (Slice 6a frozen). Investigation only: no
TypeScript campaign behaviour was written. Evidence is the original C
(`aphavoc/source`, `modules`), read in full on the SUPPLY path, plus a compile
and link probe of the candidate translation units against the current C
reference harness headers (Gate 9) and a rounding-mode probe of the inverse
square root table (Gate 10).

## Summary

- **The member-message boundary is a genuine source call boundary, but only
  at function granularity.** `assign_task_to_group` calls
  `push_task_onto_group_task_stack (group, task, valid_members)` and then
  `assign_task_to_group_members (group, guide, valid_members)`. At that call
  the task is already ASSIGNED, the guide is created, attached and on the
  group's guide stack, and the route exists. The member loop interleaves, per
  member, follower attachment → `ENTITY_MESSAGE_TASK_ASSIGNED` → helicopter
  preparation. The clean seam is therefore **before
  `assign_task_to_group_members`**, not "before the first `TASK_ASSIGNED`"
  (that would leave member 0 attached and the others not).
- **Assignment success is decided before that seam.** `assign_task_to_group_members`
  ignores every message result and returns TRUE; `assign_task_to_group`
  returns TRUE once the route exists. On the SUPPLY path reached from 6a the
  route is always created (`generate_biased_vec3d_route` always returns TRUE).
- **Route and guide are atomic with the ASSIGNED transition only in one
  direction.** The route is built first and is self-contained; the guide, its
  attachment and the UNASSIGNED → ASSIGNED switch are one function
  (`push_task_onto_group_task_stack`) whose source says the list switch "must
  be done AFTER guide is created and attached".
- **Escort creation and the remaining mutations cannot be in a slice that
  stops at the member seam.** `assign_primary_task_to_group` performs formation,
  re-parenting, the force `TASK_ASSIGNED` notification, escort assessment and
  creation, the start time and the expiry reset **after** `assign_task_to_group`
  returns, i.e. after the member loop. Every one of them therefore belongs with
  the member work.
- **Recommended 6b:** `assign_primary_task_to_group` → `assign_task_to_group`
  → `create_generic_waypoint_route` → `push_task_onto_group_task_stack`,
  ending at a boundary trap on `assign_task_to_group_members (group, guide,
  valid_members)`. 6b must not claim "assignment complete": the task is
  ASSIGNED in the middle of a synchronous transaction that EECH never leaves
  in that state, exactly as 6a's boundary is.
- **Environment:** two genuine source-shaped ports: terrain elevation at a
  point and the road node table. Everything else is campaign state, persisted
  state or generated database state.
- **Decision gates raised:** route-search floating point under the frozen
  contract (D1), the group `SECTOR_SIDE` default (D2), maps without roads
  (D3), the dead terrain read (D4), and one unreachable NULL dereference (D5).

## Gate 1: the SUPPLY assignment transaction, in source order

Entry: 6a's selected `(group, task)`, the task UNASSIGNED on the group's
keysite's `LIST_TYPE_UNASSIGNED_TASK` list, the group on that keysite's
`LIST_TYPE_KEYSITE_GROUP` list.

```
assign_primary_task_to_group (group, task)                      assign.c:525
  ASSERT server; ASSERT task_database [SUPPLY].primary_task
  ASSERT (!get_local_group_primary_task (group))               task.c:947 (guide stack walk)
  assign_task_to_group (group, task, TASK_ASSIGN_ALL_MEMBERS)   assign.c:765
    ASSERT (!(primary task on group && task primary))
    member = first LIST_TYPE_MEMBER child;  no member → FALSE
    ASSAULT_SHIP and task != ENGAGE → FALSE
    LANDING / LANDING_HOLDING / TAKEOFF / TAKEOFF_HOLDING → FALSE (debug_fatal in DEBUG)
    start_keysite = keysite parent if INT_TYPE_GROUP_LIST_TYPE == KEYSITE_GROUP, else NULL
    pos = task VEC3D_TYPE_STOP_POSITION                          (read, unused)
    force = get_local_force_entity (task side)                   (read, unused)
    sub_type = group_database [group].default_landing_type
    ASSESS_LANDING (SUPPLY: TRUE):
      end_keysite = PTR_TYPE_RETURN_KEYSITE                      (5b leaves it NULL)
      NULL → start_keysite exists (6a) → end_keysite = start_keysite
             set_local_entity_ptr_value (task, RETURN_KEYSITE, end_keysite)     ← mutation before route
      landing = get_local_entity_landing_entity (end_keysite, landing type)     (read, unused)
    create_generic_waypoint_route (group, task, end_keysite, NULL, NULL, NULL, 0)   croute.c:200
      [Gate 2]
    push_task_onto_group_task_stack (group, task, ALL_MEMBERS)  assign.c:663
      create_client_server_guide_entity (task, NULL, ALL_MEMBERS)   guide.c:115   [Gate 3]
      attach_group_to_guide_entity (group, guide)                   guide.c:808   [Gate 3]
      UNASSIGNED → ASSIGNED switch                                                [Gate 4]
    assign_task_to_group_members (group, guide, ALL_MEMBERS)    ← RECOMMENDED 6b BOUNDARY [Gate 5]
      per member: attach_group_member_to_guide_entity; notify TASK_ASSIGNED;
                  helicopter: prepare_helicopter_for_task, set_helicopter_fuel_level
    return TRUE
  formation, re-parent, force TASK_ASSIGNED, escort, start time, expiry  [Gate 7]
  return TRUE
```

Every synchronous side effect before the first member `TASK_ASSIGNED`, in order:

1. `PTR_TYPE_RETURN_KEYSITE` := the group's keysite (local set, no transmission).
2. Route: `INT_TYPE_ROUTE_CHECK_SUM` set client-server (transmitted `INT_VALUE`),
   then every waypoint created locally in route order (each: task
   `LIST_TYPE_WAYPOINT` insertion with its `LINK_PARENT` retagging the whole
   list, sector `LIST_TYPE_SECTOR` insertion and the sector's `LINK_CHILD`
   response, optionally the dependent's `LIST_TYPE_TASK_DEPENDENT` list), the
   parser's position adjustments, then `ENTITY_COMMS_CREATE_WAYPOINT_ROUTE`.
3. Guide: `create_local` (task `LIST_TYPE_GUIDE`, first waypoint
   `LIST_TYPE_CURRENT_WAYPOINT`, `LIST_TYPE_UPDATE`), then `ENTITY_COMMS_CREATE`.
4. Guide onto the group's `LIST_TYPE_GUIDE_STACK` (head); the group becomes
   `GROUP_MODE_BUSY`; `ENTITY_COMMS_SWITCH_LIST (guide, GUIDE_STACK, group, GUIDE_STACK)`.
5. `initialise_guide_criteria`: `ENTITY_COMMS_SET_GUIDE_CRITERIA` for each
   criterion whose value changes from the zeroed raw state.
6. Guide `FLOAT_TYPE_VELOCITY` client-server (`FLOAT_VALUE`).
7. Task out of `LIST_TYPE_UNASSIGNED_TASK`, into `LIST_TYPE_ASSIGNED_TASK` (head):
   the task's `LINK_PARENT` sets `TASK_STATE_ASSIGNED` and notifies the campaign
   screen `CAMPAIGN_SCREEN_MISSION_ASSIGNED`; then
   `ENTITY_COMMS_SWITCH_LIST (task, UNASSIGNED_TASK, keysite, ASSIGNED_TASK)`.

## Gate 2: `create_generic_waypoint_route` for SUPPLY

Inputs and classification:

| Input | Source | Class |
|---|---|---|
| task route nodes, dependents, waypoint and formation types, route length | Slice 5b (`PTR_TYPE_ROUTE_*`, `INT_TYPE_ROUTE_LENGTH`) | campaign state (ported) |
| existing waypoints | task `LIST_TYPE_WAYPOINT` (non-empty → return TRUE at once) | campaign state |
| first member's position | `VEC3D_TYPE_POSITION` (`MobilePhysicalState`) | physical state (existing port) |
| first member's cruise velocity and cruise altitude | `ac_float.c` → `aircraft_database` | generated database (cruise altitude column new) |
| group `INT_TYPE_SECTOR_SIDE` | **not overloaded by any group file**: `en_int.c`'s default, 0 (`ENTITY_SIDE_NEUTRAL`) | **quirk D2** |
| group movement type, default entity type, default landing type | `gp_dbase.c` | generated |
| `task_route_search` (TRUE), `add_start_waypoint` (TRUE) | `ts_dbase.c` | generated (new columns) |
| `route_biasing_database [MOVEMENT_TYPE_AIR]`: elevation 5.0, range 0.5, side 1.0, min range 5000, deviation 3.0, samples 8.0, tolerance 0.94 | `croute.c` static | generated (new) |
| terrain elevation at sample points | `get_3d_terrain_elevation` → `get_3d_terrain_point_data (x, z, NULL)` | **environment port** |
| sector side at sample points | `get_local_sector_entity (point)` → `INT_TYPE_SECTOR_SIDE` | campaign state (ported in 5b) |
| map bounds | `MIN/MAX_MAP_X/Y/Z` | campaign state (world map) |
| road nodes | `get_closest_road_node (pos, 5.0)` (`ai_misc.c`) over `road_node_positions`, `road_nodes [].number_of_links`, `total_number_of_road_nodes` | **environment port** |
| waypoint database: position type (per mobile type), minimum previous waypoint distance, guide sub type, tag class | `wp_dbase.c`, `wp_char.c` | generated |

The construction:

1. **Specified route.** Slice 5b's four nodes (PICK_UP, PREPARE_FOR_DROP_OFF,
   DROP_OFF, FINISH_DROP_OFF), each coordinate `ceil`'d. F1's `prepare.y` and
   `finish.y` (0.0) become 0.
2. **Start waypoint** (SUPPLY adds one): the first member's position, `ceil`'d
   in x, y and z, NAVIGATION, `FORMATION_ROW_LEFT`, no dependent.
3. **Land waypoint** (a return keysite exists): the keysite position,
   `ceil`'d, `WAYPOINT_LAND`, `ROW_LEFT`, dependent = the return keysite.
4. **Route search** (`generate_biased_vec3d_route`) for each consecutive pair:
   - `create_route`: start and end (`ceil`), then recursive
     `generate_best_mid_point`: while a segment is longer than 5000 m
     (squared 2D range), `get_best_point` samples 8 points on the perpendicular
     through the midpoint (spacing = perpendicular / 24), bounded to the map;
     reads 8 terrain elevations (`ceil`); rates each point
     (`get_route_point_rating`: 3-sample average elevation × 5.0, plus
     |2s − 8| / 16 × 0.5 × max (avg, 1), plus 1.0 × (sector side != NEUTRAL) ×
     max (avg, 1)); the strictly lowest rating wins, the first of equals kept;
     the new node splits the segment and both halves recurse.
   - `second_past_route`: every interior node is re-placed by `get_best_point`
     between its (already moved) predecessor and its successor.
   - `optimise_route`: removes interior nodes with a zero-length leg or whose
     unit legs (from `get_inverse_square_root`, `invsqrt.c`) have
     |dot| > 0.94 — collinear either way.
   - Sub-routes are joined, dropping each later sub-route's first node; the
     joined nodes take the specified node's type and formation at each end;
     generated nodes are NAVIGATION.
   - It always returns TRUE (the FALSE arm of `create_generic_waypoint_route`
     is unreachable). No randomness, no clock.
5. **Checksum** (`generate_route_check_sum`): `unsigned char` sum of
   `(int) x + (int) y + (int) z` over every node except the first and the last
   (wraps mod 256). Includes the F1 heights (0) and the generated nodes' y (0).
   Set with `set_client_server_entity_int_value (task, ROUTE_CHECK_SUM)`.
6. **Waypoints**, in route order: position bounded to `[1, MAX − 1]` on each
   axis (so y = 0 becomes y = 1 — after the checksum), `ASSERT
   (point_inside_map_area)`, a terrain read whose value is discarded (D4), the
   closest road node (stored as `INT_TYPE_ROUTE_NODE`), the flight time from
   the previous waypoint (2D range / cruise velocity; the first waypoint 0).
   `create_local_entity (WAYPOINT)` with parent, child-pred, position,
   NAVIGATION, route node, `ROW_LEFT`, `FLOAT_TYPE_ALTITUDE` = cruise
   altitude, flight time. When the waypoint's x and z equal the next specified
   node's, the specified type, formation and position type (per mobile type)
   are set, and it joins its dependent's `LIST_TYPE_TASK_DEPENDENT` list
   (head).
7. **Parser** (`parser_task_waypoint_route`): for each consecutive
   (last, this, next): if this and next are closer than the waypoint database's
   minimum previous distance for next, a NAVIGATION `this` moves to the
   last-next midpoint (and, if still too close, `min_range` back from next
   along the normalised half-leg); otherwise a NAVIGATION `next` moves towards
   next-next, bounded to the adjusted map volume.
8. **Replication:** `ENTITY_COMMS_CREATE_WAYPOINT_ROUTE (task, group,
   return_keysite, start, stop, check_sum, node_count)`: task, group and return
   keysite pointers, start and stop positions (each through `pack_vec3d`, the
   5b packing check), the checksum, the waypoint count and every waypoint
   index. Clients re-run the whole construction from this message and
   compare checksums: determinism is part of the EECH contract.

Quirks found (to pin, not fix):

- **Waypoint tags are assigned at link time.** Every waypoint's
  `LINK_PARENT (LIST_TYPE_WAYPOINT)` re-tags the whole list
  (`update_local_entity_waypoint_list_tags`, global letter counters reset each
  time). A waypoint's specified type is set *after* its creation, so the last
  waypoint (LAND) is tagged as NAVIGATION and is never re-tagged.
- **Moved waypoints keep their sector.** The parser's `set_local_entity_vec3d`
  writes the raw position only; sector membership stays where creation put it.
- **Side bias ignores the group's side** (D2): the group has no
  `SECTOR_SIDE` row, so the default 0 (NEUTRAL) is compared with sector sides
  that are only ever BLUE or RED: the side term always applies.

## Gate 3: guide construction and attachment

- **Creation** (`create_client_server_guide_entity`, `gd_creat.c`): the first
  waypoint (the start waypoint); guide sub type =
  `waypoint_database [NAVIGATION].guide_sub_type`; attributes: task
  `LIST_TYPE_GUIDE` parent, waypoint `LIST_TYPE_CURRENT_WAYPOINT` parent, sub
  type, `VALID_GUIDE_MEMBERS` = `TASK_ASSIGN_ALL_MEMBERS` (0xffffffff), position
  = the waypoint's. `create_local` zeroes the raw guide (default sub type
  NAVIGATION_DIRECT, overwritten), links GUIDE, CURRENT_WAYPOINT and UPDATE;
  `create_remote` transmits `ENTITY_COMMS_CREATE`. The guide's and task's link
  responses are debug-only (default TRUE); the waypoint's `LINK_CHILD` is
  debug-only.
- **Attachment** (`attach_group_to_guide_entity`): guide onto the group's
  guide stack (head), `SWITCH_LIST`; `initialise_guide_criteria`: every
  criterion from `guide_database [guide type]` (`gd_dbase.c`), then RADIUS,
  TRANSMIT_DATA and LAST_TO_REACH from the waypoint database for the group's
  default entity type; each changed criterion transmits
  `SET_GUIDE_CRITERIA`. Velocity = waypoint database velocity (NAVIGATION,
  entity type) × first member's cruise velocity, client-server.
- **What the assignment graph needs** vs. **guide execution:** the guide
  entity with its links, criteria and velocity are all set synchronously and
  are needed. Guide execution (`gd_updt.c`, `gd_nav.c`, attack and cover
  guides, waypoint reached/action messages) runs later from the update list and
  is out of scope. The follower list and `FLOAT_TYPE_DISTANCE` on members are
  member work (Gate 5).
- No formation state is touched by the guide.

## Gate 4: the ASSIGNED transition

Order inside `push_task_onto_group_task_stack`:

1. guide creation (local links, then `ENTITY_COMMS_CREATE`);
2. group attachment (`SWITCH_LIST` guide), criteria, velocity;
3. `delete_local_entity_from_parents_child_list (task, UNASSIGNED)`: the task's
   `UNLINK_PARENT` and the keysite's `UNLINK_CHILD` are default TRUE;
4. `insert_local_entity_into_parents_child_list (task, ASSIGNED, keysite, NULL)`
   (head of the keysite's assigned list);
5. inside step 4, the task's `LINK_PARENT (LIST_TYPE_ASSIGNED_TASK)`:
   `set_local_entity_int_value (TASK_STATE, ASSIGNED)`, then (primary task)
   `notify_campaign_screen (CAMPAIGN_SCREEN_MISSION_ASSIGNED)`;
6. `ENTITY_COMMS_SWITCH_LIST (task, UNASSIGNED_TASK, keysite, ASSIGNED_TASK)`.

Slice 5b's fail-loud ASSIGNED arm of `response_to_link_parent` is replaced by
the state store and the `MISSION_ASSIGNED` event (a new `CampaignEvents`
method); the COMPLETED arm stays fail-loud. Cases that distinguish the order:
the transmission sequence (checksum `INT_VALUE`, `CREATE_WAYPOINT_ROUTE`,
guide `CREATE`, guide `SWITCH_LIST`, criteria, `FLOAT_VALUE`, task
`SWITCH_LIST`) with `mission-assigned` printed between the velocity and the
task `SWITCH_LIST`; the task at the head of the assigned list; the group
`GROUP_MODE_BUSY` at the boundary.

## Gate 5: the member-message boundary

- Members are attached to the guide **inside** `assign_task_to_group_members`,
  per member and interleaved: `attach_group_member_to_guide_entity` (follower
  list, `SWITCH_LIST`, `FLOAT_TYPE_DISTANCE` 32000) → `TASK_ASSIGNED` → for a
  helicopter, `prepare_helicopter_for_task` (troops, TROOP_INSERTION only) and
  `set_helicopter_fuel_level` (route duration, fuel economy).
- The follower link itself has no campaign behaviour (aircraft `LINK_PARENT`
  has no `FOLLOWER` arm; the guide's `LINK_CHILD` is debug-only).
- No member state is needed for a valid guide/task graph:
  `validate_local_guide_entity` requires followers only for ENGAGE.
- The member message's return value is ignored and
  `assign_task_to_group_members` returns TRUE: it cannot change the outcome.
- **The member `TASK_ASSIGNED` response is the takeoff machinery**
  (`mb_msgs.c :: response_to_task_assigned`): for a LANDED member, the keysite
  landing entity, its TAKEOFF task, AI weapon configuration,
  `LOCK_TAKEOFF_ROUTE` → takeoff route or takeoff queue, and
  `RESERVE_LANDING_SITE` at the return keysite. There is no narrower seam
  inside it for keysite-based (landed) supply groups.

Verdict: the source-valid boundary is the call
`assign_task_to_group_members (group, guide, valid_members)`. A boundary
between member 0's attachment and its `TASK_ASSIGNED` is also a call
boundary, but it splits a per-member loop and gains nothing.

## Gate 6: escort creation

- Threshold: `task_database [SUPPLY].escort_required_threshold` = 6
  (`ESCORT_NEVER` 15, `ESCORT_CRITICAL` 6): an escort is critical whenever one
  is created.
- The threat is re-assessed with `assess_task_difficulty` (Slice 5b, ported)
  after the member loop. It starts at `raw->task_link.parent`, which is still
  the same keysite (now through the assigned list), over the unchanged route
  nodes. The task's stored `TASK_DIFFICULTY` is not updated.
- `create_escort_task (group, TRUE, task_database [ESCORT].task_priority,
  NULL, NULL)` (`taskgen.c:1038`): expiry 15 minutes; start = the group's
  position (the leader's); `get_task_start_keysite (ESCORT, …)` and
  `create_task (ESCORT, …)` with one ESCORT route node dependent on the group,
  both Slice 5b code; then `TASK_USER_DATA` = the group's member count. It
  creates an UNASSIGNED escort task; nothing assigns it synchronously.
- It runs after the ASSIGNED transition and after member assignment. It cannot
  be bounded into 6b without the member work; with the member work it is cheap
  (mostly 5b code).

## Gate 7: the remaining mutations of `assign_primary_task_to_group`

All run after `assign_task_to_group` returns TRUE, i.e. after the member loop:

| Effect | Detail | Classification |
|---|---|---|
| default formation | `group_database [].default_group_formation` vs raw `group_formation`; set client-server if different | 2 (with members) |
| re-parent | group `INT_TYPE_GROUP_LIST_TYPE` (raw, persisted) == KEYSITE_GROUP → `set_client_server_entity_parent (group, KEYSITE_GROUP, RETURN_KEYSITE)`. For SUPPLY that is its own keysite: the group is removed and re-inserted **at the head** of the keysite's group list (observable in 6a's first-of-equals order) and `SWITCH_PARENT` is transmitted | 2 |
| force `TASK_ASSIGNED` | `create_task_assigned_reactionary_tasks`: SUPPLY falls to the default arm (nothing) | 2 |
| escort | Gate 6 | 2 |
| start time | `FLOAT_TYPE_START_TIME` := session `FLOAT_TYPE_ELAPSED_TIME` (persisted session state), client-server | 2 |
| expiry | `set_local_entity_float_value (EXPIRE_TIMER, 0.0)` (no transmission) | 2 |

None is separable before the member seam (category 3 would need a boundary
after members, which is 6c's end). Under the recommended split, 6b ends with a
task that is ASSIGNED mid-transaction and makes no "assignment complete"
claim; 6c completes `assign_primary_task_to_group` and resumes 6a's loop
(`assign_count--` and further tasks).

## Gate 8: dependencies not yet in the core

| Dependency | Class | Notes |
|---|---|---|
| terrain elevation at (x, z) | **environment → port** | `TerrainElevation.getElevation (x, z)`; called only with map-bounded points; values `ceil`'d before use |
| road nodes | **environment → port** | `RoadNetwork`: node count, position (vec3d) and link count per node; the closest-node scan (`get_closest_road_node`, first-within-5 m early exit, strict `<`) is core. A map with no road table ASSERTs (D3) |
| member position | existing `MobilePhysicalState` port | start waypoint (x, y, z `ceil`'d) |
| member cruise altitude | generated (`ac_dbase.c`) | new column |
| group `group_list_type`, `group_formation` | persisted raw group state (`gp_pack.c`: `INT_TYPE_GROUP_LIST_TYPE`, `INT_TYPE_GROUP_FORMATION`) | new restore fields; `group_list_type` decides the start keysite |
| keysite landing entities | persisted state | 6b only reads `LIST_TYPE_LANDING_SITE` (value unused on the SUPPLY path); an empty list is valid |
| session elapsed time | persisted session state | 6c (start time) |
| waypoint database, guide database, route biasing table, task `task_route_search` / `add_start_waypoint` / `escort_required_threshold` | generated | new generator columns |
| inverse square root table | deterministic derived state (`initialise_inverse_square_root_table`) | rounding-mode independent (Gate 10) |
| route search scratch (`fast_route`, `best_point_terrain_elevations`, waypoint tag counters) | per-call scratch | no cross-call state |
| world map bounds, sectors | ported | `MAX_MAP_Y` now read |

No DCS concept appears in either port: both are the C call sites' semantics.

## Gate 9: C reference feasibility

Probe (scratch, not committed): each candidate unit compiled unchanged with
the harness's flags against the harness's reduced `project.h`.

| Unit | Result |
|---|---|
| `ai/taskgen/croute.c` | compiles unchanged |
| `entity/special/waypoint/wp_creat.c`, `wp_vec3d.c`, `wp_float.c`, `wp_char.c`, `wp_msgs.c`, `wp_ptr.c` | compile unchanged |
| `entity/special/guide/guide.c`, `gd_creat.c`, `gd_int.c`, `gd_float.c`, `gd_vec3d.c`, `gd_list.c`, `gd_ptr.c`, `gd_dbase.c`, `gd_msgs.c` | compile once `guide.h` is added to `PROJECT_H` with an opaque `terrain_3d_triangle` |
| `entity/special/landing/landing.c`, `task/ts_vec3d.c`, `task/ts_msgs.c`, `ai/highlevl/reaction.c` | compile unchanged |
| `ai/ai_misc/ai_misc.c` | compiles with `limits.h` |
| `modules/maths/invsqrt.c` | pulls `system.h` (SDL): extract `initialise_inverse_square_root_table` and `get_inverse_square_root` verbatim |

Against the current harness plus these units, 58 symbols stay unresolved. All
are outside the 6b path and become fail-loud stubs: the attack-guide and
navigation reached handlers (`gd_msgs.c`), the other task generators
(`reaction.c`), speech, formation lookup, `get_imap_value`, char-value tables,
`reassign_group_members_to_valid_tasks`, and a few maths helpers
(`get_approx_3d_range`, `normalise_3d_vector`, `check_zero_3d_vector`,
`get_3d_vector_dot_product`, `bound_position_to_adjusted_map_volume`,
`bound_position_to_map_area`), which are extracted verbatim instead. The
environment globals (`road_node_positions`, `road_nodes`,
`total_number_of_road_nodes`) and `get_3d_terrain_point_data` are supplied by
the harness from scenario data (a per-cell table: environment data, no
campaign semantics). `assign_primary_task_to_group`, `assign_task_to_group`
and `push_task_onto_group_task_stack` are extracted from `assign.c`;
`assign_task_to_group_members` becomes the trap, printing group, guide, task
and valid members.

The oracle would execute the actual route, waypoint, guide and assignment
functions. Nothing campaign-shaped is reimplemented in the shim.

## Gate 10: compatibility and undefined behaviour

- **F1 propagates consistently.** The 0.0 heights reach the route as `ceil
  (0.0)` = 0, the checksum as `(int) 0`, the waypoints as y = 1 after the
  `[1, MAX_MAP_Y − 1]` bound. Single player and multiplayer build the same
  route. No new uninitialised read is on the SUPPLY path: `flight_time` is
  initialised; `best_point_terrain_elevations [0]` is never written or read;
  `specified_route` is uninitialised only when `route_length` is 0 (not
  SUPPLY).
- **Overflow and truncation:** the checksum is an `unsigned char` sum
  (defined wrap); `(int)` of `ceil`'d floats is exact; `VALID_GUIDE_MEMBERS`
  0xffffffff. No signed overflow found.
- **D1 — floating point in route search.** Terrain ratings, averages,
  perpendicular increments and the optimiser's dot products mix float and
  double arithmetic, and **strict comparisons of ratings select route points**.
  Under the frozen contract (declared types, RTZ per operation, compile-time
  constants to nearest) every operation is modelled as in earlier slices, and
  the harness executes the same contract. The x87 intermediate-precision row
  of the numerical contract is still unresolved, and this is the first path
  where it could change a *decision* rather than one canary value. Decision
  asked: continue under the frozen declared-type RTZ contract (recommended,
  consistent with Slices 1–6a), with the route-search differential as its
  largest exercise.
- **Inverse square root:** a 512-entry seed table plus two Newton steps in
  double. The table built under round-to-nearest and under round-toward-zero
  is identical (probe: 0 differences), so its initialisation order relative to
  `set_fpu_rounding_mode_zero` does not matter. The Newton steps run under RTZ.
- **D2 — group `SECTOR_SIDE` default.** Not undefined behaviour: `en_int.c`
  returns 0 for an unoverloaded int type. Preserved as an EECH quirk (the
  side-bias term applies to every sample). Confirmation asked.
- **D3 — no road table.** `get_closest_road_node` ASSERTs
  `road_node_positions`; with nodes but none linked it returns node 0. The
  port should ASSERT the same way (a terrain without roads is not a supported
  EECH map). Confirmation asked.
- **D4 — the dead terrain read.** `create_generic_waypoint_route` reads the
  terrain elevation at every waypoint and discards it. Recommendation: make the
  port call anyway (faithful call sequence, observable to a recording port);
  alternatively omit it. Decision asked.
- **D5 — a NULL dereference off the SUPPLY path.** `parser_task_waypoint_route`
  reads `next_next_wp`'s position before checking it is non-NULL, reachable
  only when the last waypoint is NAVIGATION. SUPPLY routes always end with
  LAND. The port throws `EechNullDereferenceError` there.
- **Compiler and stack layout:** `get_inverse_square_root` type-puns through a
  pointer cast (strict aliasing; well-defined at `-O0` and in the port as a
  bit reinterpretation). No stack-layout dependency found.

## Recommended scope

**6b** — `assign_primary_task_to_group` (entry and ASSERTs),
`assign_task_to_group` (validity checks, start/return keysite, landing
lookups), `create_generic_waypoint_route` with the route search, checksum,
waypoint entities (creation, tags, sector and dependent lists), parser and
`CREATE_WAYPOINT_ROUTE`, and `push_task_onto_group_task_stack` (guide entity,
attachment, criteria, velocity, UNASSIGNED → ASSIGNED, `MISSION_ASSIGNED`,
`SWITCH_LIST`). Boundary: a trap on `assign_task_to_group_members (group,
guide, valid_members)`. Ports: `TerrainElevation`, `RoadNetwork`.
Acceptance: the social chain → 6a selection → the route/guide transaction →
the boundary, with the transmission and event order asserted.

**6c (not created)** — `assign_task_to_group_members` (follower attachment,
member `TASK_ASSIGNED` → takeoff/landing machinery, helicopter preparation and
fuel), then the rest of `assign_primary_task_to_group` (formation, re-parent,
force notification, escort, start time, expiry) and the resumption of 6a's
loop. Landing entities, takeoff routes and the session clock enter there.

**Narrower alternative:** route construction alone (boundary on
`push_task_onto_group_task_stack`). The seam is valid, but the push step is
small, has no environment dependency, and is what makes the assignment graph
exist, so it adds little risk to keep it in 6b.

**Decisions needed:** the scope above; D1 (floating-point contract for route
search); D2, D3 and D5 (preserve as found); D4 (call or omit the dead terrain
read).
