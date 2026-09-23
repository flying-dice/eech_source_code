# Slice 6a: supply-task assignment decision (issue #16)

Slice 6a ports which unassigned task a keysite assigns and to which group:
`assign.c :: assign_keysite_tasks (keysite, category)` up to its call of
`assign_primary_task_to_group (group, task)`. That call is the boundary.
Nothing inside it is ported. The investigation, the lead's decisions and the
6b / 6c decomposition are in `supply-task-assignment-investigation.md`.

## The path

The primary social case runs Slices 1–5b unchanged, then this slice:

1. Keysite cargo (Slice 4) → `FORCE_LOW_ON_SUPPLIES` (Slice 5a) →
   `create_supply_task` → `create_task` (Slice 5b). An UNASSIGNED supply task
   lands on its start airbase's `LIST_TYPE_UNASSIGNED_TASK` list.
2. `assign_keysite_tasks (airbase, TASK_CATEGORY_SUPPORT)`, which does the following:
   - It returns if the keysite has no unassigned task, or none of the category (`TASK_CATEGORY` comes from `ts_dbase.c`).
   - It counts idle groups per sub type over the force's `LIST_TYPE_AIR_REGISTRY`. The array is function-static in C and cleared on every call.
   - It collects the category's tasks with their sort key: `TASK_PRIORITY`, doubled (`*= 2.0`, exact) for a critical task.
   - It sorts them with `en_misc.c :: quicksort_entity_list`. This is the original `qs`, ported as written. It is descending and not stable, with a middle pivot.
   - It sets `assign_count = max (assign_task_count, 1u)` and the reserve to `reserve_task_count` (`ks_dbase.c`).
   - It walks the sorted tasks. It skips a pilot-locked task. It reserves non-critical tasks whose expiry is over `KEYSITE_TASK_ASSIGN_TIMER` (180 s) while the reserve lasts. For every other task it calls `get_suitable_registered_group`.
3. `get_suitable_registered_group (task, idle_group_count)` walks the task's keysite's `LIST_TYPE_KEYSITE_GROUP` list. A group qualifies when all of these hold:
   - it is not pilot locked;
   - it is not an assault ship;
   - it is idle;
   - its raw sleep is 0.0;
   - its side is the task's side;
   - the idle count is over `minimum_idle_count` (`gp_dbase.c`);
   - `MEMBER_COUNT` is at least the task's minimum (`ts_dbase.c`);
   - `check_group_members_awake` passes;
   - its suitability is over 0;
   - `suitable_group_task_specific_checks` passes (the ESCORT and TROOP_INSERTION arms);
   - `group.c :: assess_group_task_locality_factor` passes: the first member's approximate 2D range to the keysite, divided by its `CRUISE_VELOCITY`, gives an ETA that must not exceed the task's expiry.

   The qualifying group with the lowest non-zero suitability wins (`result < best_result`). On a tie, the first group encountered in the list wins. Under the frozen EECH databases every qualifying score is 1.0, so the observable result today is that the first qualifying group wins (6a-F1).
4. `assign_primary_task_to_group (group, task)` is **the boundary**.

## The boundary

- **In production,** `assignPrimaryTaskToGroup` always throws `UnportedBoundaryError`. It is an `UnportedBehaviourError` whose `args` are `[group, task]`. There is no observer seam, interceptor or configurable no-op. The error cannot be turned into a return value, so the loop after the call is never reached. Its `assign_count` bookkeeping and the `assign_count == 0` break are ported as written and excluded from coverage (below).
- **The conformance runners** catch this one error and end the scenario with `result boundary assign_primary_task_to_group <group> <task>`. They then print the entity graph.
- **The C harness** defines `assign_primary_task_to_group` as a trap. It prints the same line and `longjmp`s out of the operation, exactly as a failed `ASSERT` does. The harness prints the graph the same way.

## Restored state

These restore operations were added. Each sets state that EECH persists in saved games:

| Line | Meaning | EECH source of the state |
|---|---|---|
| `member-count <group> <n>` | a group's raw `member_count` (6-bit) | `gp_pack.c :: unpack_local_data` (:430, :560): `raw->member_count = unpack_int_value (en, INT_TYPE_MEMBER_COUNT)` |
| `group-sleep <group> <v>` | a group's raw sleep timer | `gp_pack.c` (:406): `raw->sleep = unpack_float_value (en, FLOAT_TYPE_SLEEP)` |
| `air-register <group>` | the group on its force's `LIST_TYPE_AIR_REGISTRY` (appended) | the force's registry list and the group's `registry_link` |
| `aircraft-type <member> <sub type>` | a helicopter member's `mob.sub_type` | the saved member's sub type |
| `add-member <label> <group> <entity type> <sub type> <x> <z>` | an aircraft member, helicopter or fixed wing, appended to a group | saved members |
| `unassigned-task <label> <keysite> <objective or NULL> <sub type> <side> <critical> <priority> <expire>` | an UNASSIGNED task on a keysite's unassigned list and its objective's dependent list | saved tasks |
| `pilot <label>`, `pilot-lock <entity> <pilot>` | a pilot, and a task or group on its `LIST_TYPE_PILOT_LOCK` list | `pi_pack.c` (:240), `ts_pack.c` (:1035), `gp_pack.c` (:534) |
| `assign-tasks <keysite> <category>` | `assign_keysite_tasks (keysite, category)` | — |

- **`member_count` is restored, not maintained.** Its live maintenance is `gp_msgs.c :: response_to_link_child` (`LIST_TYPE_MEMBER`). That code is reached only through helicopter creation (`hc_creat.c`), and its dependencies explode, so it is not ported and still fails loudly. The scenario generators keep `member_count` equal to the number of members they restore.

## Findings

- **6a-F1: the algorithm, and what the frozen databases make of it.**
  - **The algorithm.** `get_suitable_registered_group` keeps the qualifying group with the lowest non-zero suitability (`result < best_result`, starting from `FLT_MAX`).
  - **Ties.** The comparison is strict, so the first group encountered in `LIST_TYPE_KEYSITE_GROUP` order wins.
  - **The frozen EECH databases.** Every qualifying non-zero `get_group_to_task_suitability` is exactly 1.0. `suitable.c` rejects a group weaker than any of the task's strengths, so each `min (a / b, 1.0)` is 1.0.
  - **The observable result today.** The first qualifying group wins.
  - **The port.** It keeps the generic comparison. `wutcfg.c` / WUT database overrides are not ported yet and may expose different non-zero scores. A unit invariant proves the current database property over every group and task pair, so porting those overrides makes it a deliberate fidelity checkpoint rather than a hidden assumption.
- **6a-F2: the ETA division never divides by zero, and aircraft never sleep.**
  - **Cruise velocity.** `CRUISE_VELOCITY` is `aircraft_database [mob.sub_type].cruise_velocity`, from `ac_float.c`. `src/generated/c-aircraft-database.ts` generates it from `ac_dbase.c`: `knots_to_metres_per_second (K)`, folded at float precision with round to nearest, as the compiler does. It matches the compiled C database bit for bit (`aircraft-database.cref.test.ts`). All 33 entries are positive (70–450 knots).
  - **Which groups get this far.** A group reaches locality only if at least one idle group of its sub type is on the air registry (`idle_count > minimum_idle_count >= 0`). Only aircraft group types join the air registry (`gp_dbase.c` `registry_list_type` and `default_entity_type`, generated and checked by an invariant). Every supply-compatible group's default aircraft has a positive cruise velocity.
  - **Sleep.** No aircraft file overloads `FLOAT_TYPE_SLEEP`, so an aircraft member's sleep is `en_float.c`'s default 0.0, and `check_group_members_awake` never rejects one. Vehicles do overload sleep (`vh_float.c`), but their groups are on the ground or sea registry.
  - **What is pinned instead.** The reachable boundary is pinned. At 85 knots (UH-60, 43.80644… m/s) from consecutive float distances: ETA 1200 − 2⁻¹³ is eligible; ETA exactly 1200 is eligible, because `eta > expire` is strict; ETA 1200 + 2⁻¹³ is rejected. Zero distance with a real cruise velocity is eligible.

## Coverage exclusions

Four branches are excluded, each narrowly and each backed by a test:

| Branch | Why it is unreachable | Backed by |
|---|---|---|
| `assign.ts`: `if (assign_count === 0) break` | `assign_count` only falls after `assign_primary_task_to_group` succeeds, which is past the boundary | the boundary unit test |
| `assign.ts`: both arms of `if (assignPrimaryTaskToGroup (...) !== 0)` | the call always throws | the boundary unit test |
| `assign.ts`: `check_group_members_awake`'s `return FALSE` and its caller's else arm | aircraft members' sleep is the default 0.0 (6a-F2) | the sleep invariant test |
| `group.ts`: `assess_group_task_locality_factor`'s no-member return, and its else arm (a task not on a keysite's unassigned list) | the only ported caller has already required `member_count >= 1` (every task's minimum, invariant test) and takes tasks from a keysite's unassigned list. Its `START_POSITION` row is unported and would fail loudly | the minimum member count invariant |

The paths the adopted callers do not take, but other callers can, are unit-tested rather than excluded:
- the `NULL` idle count of `msg_in.c` (`INT_MAX`);
- a `NULL` `return_distance`;
- the ASSERTs.

## C reference

- **Extracted verbatim:**
  - from `assign.c`: `assign_keysite_tasks`, `suitable_group_task_specific_checks`, `get_suitable_registered_group` and `check_group_members_awake`, into `eech_extracted_assign.c`, with the release `ai_log` macro;
  - `en_misc.c`'s `qs` and `quicksort_entity_list`, into `eech_extracted_en_misc.c`;
  - `group.c :: assess_group_task_locality_factor`.
- **Compiled whole:** `ac_float.c`, `ac_dbase.c` and `pi_list.c`. Members, both helicopter and fixed wing, are the original `aircraft` struct.
- **Stubbed:** `ac_float.c`'s other rows name the terrain (`get_3d_terrain_point_data_elevation`) and the 3D object database. Nothing ported reads them; they are fail-loud stubs or `NULL`.
- **The trap:** `assign_primary_task_to_group` (above).
- **`aircraft-cruise-velocity`** prints the compiled `aircraft_database [].cruise_velocity` bits for the generated column's check.

## Conformance

| Check | Where | Size |
|---|---|---|
| Hand-derived matrix | `test/scenarios/supply-task-assignment.cases.ts` | 36 cases, run in JS, in Lua 5.1 and against the C (TS == C on the whole output) |
| Cruise velocity column | `aircraft-database.cref.test.ts` | 33 entries, bit for bit |
| Boundary, unadopted paths, invariants | `test/unit/supply-task-assignment.test.ts` | unit tests |
| Fresh randomised scenarios | `differential.cref.test.ts` (`generateRandomSupplyTaskAssignment`, seed `0x6a16`) | 1000, C == TS line for line |
| Recorded randomised scenarios | `c-reference-random-supply-task-assignment.cases.ts` (seed 20261016) | 150, replayed in JS and Lua 5.1 |
| Coverage | `npm run coverage` | 100% statements, branches, functions and lines, with the exclusions above |

**The generator's required outcomes.** It must reach every one of these:
- `result ok`;
- a boundary with a restored task, and with a task the real 5b path constructed;
- the `force`, `objective` and comms-model ASSERTs.

**The matrix pins:**
- **Locality:** zero distance; ETA below, at and above the expiry.
- **Group choice:**
  - the first of equals wins;
  - a dead group is a candidate;
  - the force-wide registered idle count qualifies an unregistered local group, and no registered idle group of the type means no group;
  - a busy registered group is not counted;
  - the minimum idle count's strict `>`: 2 and 3 idle attack groups.
- **Rejections:** busy, asleep, the other side, below the minimum member count, unsuitable, an assault ship, a pilot-locked group.
- **The ESCORT arms:** a fast objective, a slow objective, and the slower-than-objective check.
- **The TROOP_INSERTION arm:** an enemy airbase against a friendly one.
- **Task choice:**
  - category filtering;
  - no task of the category, and no task at all;
  - the non-stable quicksort order of 2, 3 and 5 equal-priority tasks (t1, t2 and t3 first);
  - critical doubling;
  - airbase and FARP reservation, and the strict `> 180` expiry;
  - a pilot-locked task;
  - a task with no group does not stop the loop.

## The frozen fixtures

Every frozen fixture is byte-identical. The harness's member struct changed from a position-only shim to the original `aircraft` struct. This is not observable, because no frozen corpus reads a member beyond its position.

## Handed on (provisional, not created)

- **6b:** `TerrainElevation` and `RoadNetwork` ports; route, waypoint and guide; the ASSIGNED transition. It stops before the member `TASK_ASSIGNED` message.
- **6c:** members, takeoff and landing.
