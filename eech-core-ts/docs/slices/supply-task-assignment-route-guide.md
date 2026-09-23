# Slice 6b: supply-task assignment, route and guide (issue #18)

Slice 6b ports the SUPPLY assignment transaction from Slice 6a's boundary to
the member seam:

`assign_primary_task_to_group` → `assign_task_to_group` →
`create_generic_waypoint_route` → `push_task_onto_group_task_stack` → **the
boundary**, `assign_task_to_group_members (group, guide, valid_members)`.

No member is attached, messaged or prepared. The investigation, the gates and
decisions D1–D5 are in `supply-task-assignment-route-guide-investigation.md`.
The migration of the Slice 6a corpus is reported in
`supply-task-assignment-fixture-migration.md`.

## The path

The primary social case runs Slices 1–6a unchanged, then this slice.
Everything after this list is synchronous, inside one `assign_keysite_tasks`
call:

1. **`assign_primary_task_to_group (group, task)`.** Only a SUPPLY task gets past the entry (see *The scope gate*). It ASSERTs the server comms model, that the task is primary, and that the group has no primary task (`get_local_group_primary_task`, whose own ASSERT is `count <= 1`).
2. **`assign_task_to_group (group, task, TASK_ASSIGN_ALL_MEMBERS)`:**
   - It returns FALSE for:
     - a group without members;
     - an assault ship given anything but ENGAGE;
     - the LANDING, LANDING_HOLDING, TAKEOFF and TAKEOFF_HOLDING tasks.
   - The start keysite is the group's keysite, when the group is on a keysite's group list.
   - SUPPLY assesses landing:
     - The return keysite is read from the task. When it is NULL, the start keysite takes its place and is stored in the task. With no start keysite either, `get_closest_keysite` is needed, and it is not ported.
     - A return keysite other than the start keysite needs the free landing sites (`landing.c`), which are not ported.
     - The group's default landing type is looked up at the keysite (`get_local_entity_landing_entity`). A keysite with LANDING_SITE children is not ported.
3. **`create_generic_waypoint_route (group, task, return_keysite)`:**
   - Sets the start point (the leader's position) and the stop point (the return keysite's).
   - Runs the route search, `generate_biased_vec3d_route`: `create_route`, `generate_best_mid_point`, `get_best_point`, `second_past_route` and `optimise_route`. Its inputs are:
     - the terrain (the `TerrainElevation` port);
     - the sector sides;
     - the route biasing database;
     - `invsqrt.c`.
   - Computes the `unsigned char` checksum, which is set client-server.
   - Creates the waypoints, in route order:
     - one discarded terrain read each (D4, through the port);
     - the closest road node (the `RoadNetwork` port; D3 ASSERTs a map without a road table);
     - the flight time and the tags.
   - Runs the parser (`parser_task_waypoint_route`).
   - Sends `ENTITY_COMMS_CREATE_WAYPOINT_ROUTE`.
4. **`push_task_onto_group_task_stack`:**
   - Creates the guide client-server.
   - Attaches the group: the guide goes onto its guide stack, the list switch is sent, the criteria are set (`initialise_guide_criteria`) and the velocity is set.
   - Moves the task from the keysite's unassigned list to the head of its assigned list. The task's `LINK_PARENT` makes it ASSIGNED and notifies `CAMPAIGN_SCREEN_MISSION_ASSIGNED`. `SWITCH_LIST` follows.
5. **`assign_task_to_group_members (group, guide, valid_members)`** is **the boundary**.

## The boundary

- **In production,** `assignTaskToGroupMembers` always throws `UnportedBoundaryError("assign.c :: assign_task_to_group_members", [group, guide, valid_members])`.
- **State at the throw.** The task is ASSIGNED and the guide exists, is attached and is on the group's guide stack. EECH never leaves this state: it is the middle of one synchronous transaction, exactly as 6a's boundary was. 6b does not claim "assignment complete".
- **The conformance runners** end the scenario with `result boundary assign_task_to_group_members <group> <guide> <valid members, hex>`. They then print the assignment graph (`observe-assignment`) and the entity graph.
- **The C harness** traps the same call and prints the same lines.

## The scope gate

The port assigns SUPPLY tasks only. For any other task type,
`assignPrimaryTaskToGroup` throws Slice 6a's boundary,
`UnportedBoundaryError("assign.c :: assign_primary_task_to_group", [group, task])`,
before anything else. The C harness does the same by linking with
`--wrap=assign_primary_task_to_group`:

- `__wrap_assign_primary_task_to_group` prints `result boundary assign_primary_task_to_group <group> <task>` for a task that is not SUPPLY;
- for SUPPLY it calls `__real_`.

This is not an interceptor. The TS gate is the production code, and it
changes no decision: the C would enter the same function with the other
types' routes, which are not ported. The non-SUPPLY cases of Slice 6a
therefore keep their 6a result line unchanged. This is checked by the
migration below.

## State invariants (a test-oracle correction)

Executing past 6a's boundary reads state that 6a's corpus never needed and
restored incompletely. Every change below corrects invalid test state and
changes no campaign behaviour. Slice 6a remains valid for the boundary it
tested.

- **Tasks are persisted tasks, with their route.**
  - `unassigned-task` is replaced by `persisted-task`, with `route <n> (<x> <y> <z> <waypoint type> <formation> <dependent>) × n` and `return <keysite | NULL>`. These are the fields `ts_pack.c :: unpack_local_data` restores (`route_length`, `route_nodes`, `route_waypoint_types`, `route_formation_types`, `route_dependents`, `return_keysite`).
  - Both parts are required. A line without them, or with `n < 1`, fails immediately: `harness_fail` in the C, a thrown error in the TS runner.
  - A route-less task was the uninitialised `specified_route` of `croute.c:445`. The port surfaces it as `EechUndefinedBehaviourError`, and a unit test pins it.
  - The social cases construct their task through Slice 5b's real `create_supply_task`.
- **Aircraft are inside the map.**
  - The generators constrain aircraft positions to the world map, and every 6b scenario has a map, a terrain and a road node.
  - An off-map leader makes the route search recurse without end. EECH has no guard for that, and the port adds none.
  - `the Slice 6b corpus holds every aircraft inside the map` checks every hand case, every recorded fixture and fresh generated scenarios.
- **The world exists before the comms model changes.** The migration adds a missing world map, terrain and road table before the first `assign-tasks`, and before any `comms-model` switch. EECH builds them at campaign load, as the server. A map created later, under the client model, is not a campaign state. The C ASSERTs `assert_local_create_entity_index` there, and the port now does the same (below).
- **6b transaction scenarios select SUPPLY only.** `generateRandomSupplyTaskTransaction` restores or constructs only SUPPLY tasks.
- **The Slice 6a corpus is migrated mechanically, not re-recorded by hand.** `test/c-reference/slice-6a-migration.cref.test.ts` checks every one of the 186 Slice 6a scenarios (36 hand, 150 random) against their output from the original C at `81ed32e`:
  - the lines before the former boundary are identical;
  - a non-SUPPLY result is identical;
  - a SUPPLY selection ends at the new boundary for the same group, with the same task, and only that task, ASSIGNED.

  The results are 53 SUPPLY transactions, 111 without a selection, 11 non-SUPPLY selections (which stay at the 6a boundary), and 11 ASSERTs. The non-SUPPLY and no-selection scenarios remain the 6a selection corpus (121 recorded).

## Restored state and environment

| Line | Meaning | EECH source |
|---|---|---|
| `persisted-task …` | above | `ts_pack.c :: unpack_local_data` |
| `terrain <default> <cell size> <cells x> <cells z> <elevations…>` | the `TerrainElevation` port: a grid, and a default outside it | the theatre's terrain (`terrelev.h :: get_3d_terrain_elevation`) |
| `road-node <x> <y> <z> <links>` | the `RoadNetwork` port: the next node of the node table | the theatre's road table (`ai_route.h`) |
| `observe-environment` | print every terrain and road lookup, in call order | — |
| `observe-assignment` | print the assignment graph at the boundary | — |

Both ports are narrow and source-shaped:

- **`TerrainElevation.getTerrainElevation (x, z)`.** The core narrows the result to float.
- **`RoadNetwork`.** It exposes `hasRoadNodeTable`, `getTotalNumberOfRoadNodes`, `getRoadNodePosition` and `getRoadNodeNumberOfLinks`.

The route search and the closest-node search are campaign code and stay in
the core.

## Decisions (as taken)

- **D1.** The declared-type RTZ contract, with route-choice canaries for #9 (below).
- **D2.** A group's `SECTOR_SIDE` is `en_int.c`'s default 0. It is compared with sector sides that are only ever BLUE or RED, so the side bias always applies.
- **D3.** A map without a road table ASSERTs in `get_closest_road_node`. This happens after the checksum and before the first waypoint.
- **D4.** The discarded terrain lookup per waypoint goes through the port. A matrix case records it.
- **D5.** The parser's NULL dereference is `EechNullDereferenceError`. It is unreachable, and its exclusion is below.

## Findings

These are corrections to the investigation, found by executing the C:

- **6b-F1: the LAND waypoint is re-tagged.** `croute.c:716` sets the specified type through `set_local_entity_int_value`, whose sub-type row (`wp_int.c`) re-tags the whole list. The last waypoint (LAND) therefore ends with tag `W`, not a navigation letter. The investigation said it was never re-tagged.
- **6b-F2: waypoints are not in a sector in release builds.** `wp_creat.c:196` links the sector only under `DEBUG_MODULE`.
- **6b-F3: `second_past_route` can read `best_point` uninitialised.**
  - `best_point` is one local for the whole walk. A FALSE from `get_best_point` (the neighbours are at most 5000 m apart) keeps the previous iteration's point. On the first iteration there is none.
  - The port throws `EechUndefinedBehaviourError` there; a later failure reuses the point, which is defined.
  - No scenario in the corpus reaches it. A hand-built route pins it.
- **6b-F4: D5's arm is unreachable for every route, not only SUPPLY's.** The parser's `else if (next is NAVIGATION)` needs a range below NAVIGATION's minimum previous waypoint distance. That distance is 0 in every mobile column (`wp_dbase.c`), so no route of any task reaches the arm.
  - Only ATTACK and RECON have a spacing.
  - No waypoint type of a SUPPLY route has one, so the parser never moves a supply waypoint.
- **6b-F5: `initialise_guide_criteria` transmits RADIUS and LAST_TO_REACH twice.** The guide database loop clears every criterion the guide type lacks, these two included. The waypoint database then sets them. Both are sent: cleared, then set.
- **6b-F6: normalisation under RTZ is not exact.** The parser's back-off from an ATTACK waypoint normalises the half leg with `1.0 / length`, truncated. A 1500 m leg gives 1 − 2⁻²⁴, so a 5000 m back-off from x 13000 lands at `ceil (8000.00049)` = 8001.

- **6b-F7: a port gap in the entity runtime (Slice 3's `validateLocalCreateEntityIndex`).** It checked only the server case, so a local create under the client comms model passed where the C ASSERTs. It now follows `en_valid.c`:
  - a client never accepts `ENTITY_INDEX_DONT_CARE`;
  - a client's given index depends on the comms data flow, which is not ported and fails loudly.

  No frozen fixture changed.

## Route-choice canaries (#9)

Three matrix cases place terrain so that sample 1 and sample 8 of one
`get_best_point` call rate within their last bits. The route then depends on
the order and rounding of the float operations.

The canonical oracle's choice is the expectation, and it is checked in JS,
Lua 5.1 and C. `test/fpu-spike/route-choice.fpu.test.ts` checks the choice of
each investigation variant (`c-reference/fpu-variants.mjs`):

| Canary | canonical | sse-rn | x87-rtz-pc53 | x87-rn-pc53 |
|---|---|---|---|---|
| `rounding-mode-decides` | 1 | 8 | 1 | 8 |
| `x87-intermediate-precision-decides` | 1 | 1 | 8 | 1 |
| `every-other-model-disagrees` | 8 | 1 | 1 | 1 |

When #9 resolves the x87 row, these cases show which routes change.

## Coverage exclusions

There are seven exclusions, each narrow and each backed by a test in
`test/unit/supply-task-transaction.test.ts`:

| Branch | Why it is unreachable | Backed by |
|---|---|---|
| `assign.ts`: the else of `if (ASSESS_LANDING)` | the scope gate admits only SUPPLY, which assesses landing | database facts: SUPPLY's `assess_landing` |
| `croute.ts`: the else of `if (start_point_count > 0)` | SUPPLY adds a start waypoint | SUPPLY's `add_start_waypoint` |
| `croute.ts`: the else of `if (return_keysite)` | `assign_task_to_group` passes a return keysite for a task that assesses landing | SUPPLY's `assess_landing` |
| `croute.ts`: the else of `if (generate_route)` | SUPPLY searches its route | SUPPLY's `task_route_search` |
| `croute.ts`: the else of `if (cruise velocity > 0.0)` | every aircraft's cruise velocity is positive | every `AIRCRAFT_DATABASE_CRUISE_VELOCITY` > 0 |
| `croute.ts`: the parser's `if (next is NAVIGATION)` arm (D5 inside it) | 6b-F4 | NAVIGATION's minimum previous distance is 0 in every column |
| `croute.ts`: the `: 0` of the side bias | every sector is BLUE or RED and the group's side is 0 (D2) | sector side is never NEUTRAL; group side is 0 |

Slice 6a's exclusions stay as they were, with their comments moved to this
slice's boundary.

The paths no valid supply selection takes, but other callers or other states
can, are unit-tested rather than excluded:

- the FALSE returns of `assign_task_to_group`;
- the unported landing and closest-keysite lookups;
- a pre-routed task;
- the client's rebuild;
- the empty-route undefined behaviour;
- a single-point route's NULL checksum;
- a full heap;
- every waypoint setter, getter and tag class;
- `optimise_route`'s zero-length legs;
- `get_inverse_square_root` at subnormal and extreme floats (bits from the C);
- `bound_position_to_adjusted_map_volume`.

## C reference

- **Compiled whole:**
  - `croute.c`;
  - `wp_creat.c`, `wp_vec3d.c`, `wp_float.c`, `wp_char.c`, `wp_msgs.c` and `wp_ptr.c`, alongside Slice 5b's `wp_dbase.c`, `wp_int.c` and `wp_list.c`;
  - `gd_creat.c`, `gd_int.c`, `gd_float.c`, `gd_vec3d.c`, `gd_list.c`, `gd_ptr.c`, `gd_dbase.c` and `gd_msgs.c`;
  - `landing.c`.
- **Extracted verbatim:**
  - `assign_primary_task_to_group`, `assign_task_to_group` and `push_task_onto_group_task_stack`, into `eech_extracted_assign_transaction.c`;
  - the route maths (`invsqrt.c`, the range helpers), into `eech_extracted_route_maths.c`.
- **Link-time wraps:** `create_supply_task` (Slice 5b's observer) and `assign_primary_task_to_group` (the scope gate).
- **The trap:** `assign_task_to_group_members`.
- **Database checks:**
  - `database-6b` dumps the route biasing, waypoint, guide and group columns the slice reads; `route-databases.cref.test.ts` compares them with the generated TS.
  - `invsqrt <bits…>` runs `get_inverse_square_root`. The same test compares 4096 arguments across the positive float range bit for bit, subnormals included. The investigation's probe showed that the 512-entry seed table is the same whether it is built to nearest or toward zero, so its start-up order does not matter.

## Conformance

| Check | Where | Size |
|---|---|---|
| Hand-derived matrix | `test/scenarios/supply-task-transaction.cases.ts` | 11 cases (social chain, short route in order, D2, D3, D4, single player, campaign guard, fixed wing, 3 canaries), run in JS, in Lua 5.1 and against the C |
| 6a matrix, migrated | `test/scenarios/supply-task-assignment.cases.ts` | 36 cases; 24 run the transaction |
| Migration check | `slice-6a-migration.cref.test.ts` | 186 scenarios |
| Fresh randomised transactions | `differential.cref.test.ts` (`generateRandomSupplyTaskTransaction`, seed `0x6b18`) | 1000, C == TS line for line |
| Recorded randomised transactions | `c-reference-random-supply-task-transaction.cases.ts` (seed 20261023) | 150, replayed in JS and Lua 5.1 |
| Migrated 6a SUPPLY transactions | `c-reference-migrated-supply-task-transaction.cases.ts` | 29, replayed in JS and Lua 5.1 |
| 6a selection corpus | `c-reference-random-supply-task-assignment.cases.ts` | 121, replayed in JS and Lua 5.1 |
| Databases and invsqrt | `route-databases.cref.test.ts` | the columns; 4096 arguments |
| Canary variants | `route-choice.fpu.test.ts` (`npm run spike:fpu`) | 3 canaries × 4 variants |
| Boundary, unadopted paths, invariants | `test/unit/supply-task-transaction.test.ts` | unit tests |
| Coverage | `npm run coverage` | 100% statements, branches, functions and lines, with the exclusions above |

## Frozen fixtures

- **Slices 1–5b:** byte-identical.
- **Slice 6a:** its fixtures are migrated as above. No line before its boundary changed.

## Handed on (6c, not created)

- `assign_task_to_group_members`: follower attachment, the member `TASK_ASSIGNED` message → takeoff and landing, helicopter preparation and fuel.
- The rest of `assign_primary_task_to_group`: formation, re-parenting, force notification, escort, start time and expiry.
- The resumption of 6a's loop.
