# Slice 5a: `response_to_force_low_on_supplies` (issue #12)

This slice ports the force's `ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES` response intact. It runs from the guard to the `create_supply_task` call, and that call is the new observable boundary.

The foundations are frozen:
- Slices 1–4, merged at `e6a74dc`;
- the round-toward-zero numerical contract from #7.

The investigation that fixed this boundary is `docs/slices/supply-task-investigation.md`. It traces the whole C path to its terminus. Task construction is Slice 5b: `create_supply_task` → `create_task` → the task entity → start-keysite scoring → difficulty and route construction. F1 (the uninitialised route heights) belongs to 5b.

## The seam

```
FORCE_LOW_ON_SUPPLIES          (keysite.c :: update_keysite_cargo, Slice 4; group.c :: assess_group_supplies, Slice 1)
  → guard                      game status INITIALISED, not a comms client
  → duplicate-task decision    entity_is_object_of_task + the TASK_USER_DATA walk
  → supplier selection         factory / oil refinery, then airbase; 10 km early exit; strict <
  → cargo selection            first crate of the sub type on the supplier's cargo list
  → create_supply_task (requester, supplier, cargo, MOVEMENT_TYPE_AIR, task_database[SUPPLY].task_priority, NULL, NULL)   ← boundary (Slice 5b)
```

The response decides whether a supply task should exist, and identifies its supplier and cargo. Everything after that is a separate subsystem with much more infrastructure.

## What is ported

| C | TS |
|---|---|
| `fc_msgs.c :: response_to_force_low_on_supplies`, and its row of `overload_force_message_responses` | `src/entity/special/force/fc_msgs.ts` |
| `task.c :: entity_is_object_of_task` | `entityIsObjectOfTask` (`src/entity/special/task/task.ts`) |
| `ts_int.c` (`ENTITY_SUB_TYPE`, `SIDE`, `TASK_STATE`), `ts_float.c` (`TASK_USER_DATA`), `ts_list.c` (`task_dependent_link`) | `overloadTaskFunctions` |
| `wp_int.c` (`ENTITY_SUB_TYPE`), `wp_list.c` (`task_dependent_link`), `en_float.c :: default_get_entity_float_value` for a waypoint's `TASK_USER_DATA` | `overloadWaypointFunctions` (`src/entity/special/waypoint/waypoint.ts`), `defaultGetEntityFloatValue` |
| `ks_list.c`, `gp_list.c`: `task_dependent_root` | `overloadKeysiteFunctions`, `overloadGroupFunctions` |
| `keysite.c :: get_keysite_supply_position` | `getKeysiteSupplyPosition` |
| `ts_dbase.c :: task_database [].task_priority` | `TASK_DATABASE_TASK_PRIORITY` (generated, drift-checked) |
| `taskgen.c :: create_supply_task` | `createSupplyTask`: the boundary. It fails loudly in production and is recorded in tests (`callUnportedFunction`) |

Tasks and waypoints are *read* here, never created. A scenario restores them raw onto their objective's `LIST_TYPE_TASK_DEPENDENT` list, the way keysites and forces are already restored. Their creation is 5b.

## Original behaviour, preserved and pinned by C-derived cases

All of the following is deterministic source behaviour. It is reproduced, not corrected (`test/scenarios/force-low-on-supplies.cases.ts`).

- **F2: an airbase supplies itself.**
  - `get_closest_keysite (AIRBASE, side, pos, 10 km, …, exclude = NULL)` searches from the requester's own position, so an in-use airbase requester finds itself at range 0 on the early exit.
  - It then replaces any factory that is not also at range exactly 0.
  - Its own newest crate of the type is the cargo. With no such crate (level below one crate's worth), cargo is NULL and nothing is created, even beside a stocked factory.
- **The duplicate-task guard is asymmetric.**
  - `entity_is_object_of_task` counts only tasks that are `ENTITY_TYPE_TASK`, of the sender's side, and not `TASK_STATE_COMPLETED`.
  - The walk it gates checks none of these. It matches any entity on the list whose `ENTITY_SUB_TYPE` equals `ENTITY_SUB_TYPE_TASK_SUPPLY` and whose `TASK_USER_DATA` equals the sub type.
  - So a completed ammo task, or another side's ammo task, blocks a new ammo request, but only once any own-side supply task that is not completed is counted.
- **A RECON waypoint reads as an ammo supply task** (found while covering the entity-type check).
  - `croute.c:725` links route waypoints into their dependent's `LIST_TYPE_TASK_DEPENDENT` list.
  - `ENTITY_SUB_TYPE_WAYPOINT_RECON` is 21, the value of `ENTITY_SUB_TYPE_TASK_SUPPLY`.
  - `wp_float.c` does not overload `FLOAT_TYPE_TASK_USER_DATA`, so `en_float.c`'s default answers 0.0, which is `ENTITY_SUB_TYPE_CARGO_AMMO`.
  - A recon waypoint therefore blocks an ammo request whenever the guard lets the walk run.
  - A supply route's own waypoints (pick-up 19, prepare, drop-off 6, finish) never match.
- **The first keysite within 10 km wins, not the nearest.** `get_closest_keysite` returns on the first keysite of `LIST_TYPE_KEYSITE_FORCE` order within the early-exit range.
- **Approximate against exact range.**
  - On the early exit, the reported range is `get_approx_2d_range`; otherwise it is `get_2d_range`.
  - The factory/airbase comparison mixes them. One case keeps a farther factory: the airbase's approximate 10000 is not below the factory's exact 9850.9, though the airbase is really 9708 away.
  - Another case picks a farther airbase: its approximate 9875 is below the factory's exact 10500, though the airbase is really 11172 away.
- **Supplier fallbacks and ties:**
  - ammo: factory, else oil refinery; fuel: oil refinery, else factory;
  - the airbase wins only when strictly closer (an exact tie keeps the factory);
  - with nothing found, both ranges are `FLT_MAX` and `factory` is NULL.
- **The cargo walk** takes the first crate of the sub type, skipping newer crates of the other type.
- **Senders.** Both senders pass `AMMO` or `FUEL`, and a group's position is its leader's. A group without members has no position; `get_closest_keysite` hands the NULL to `get_approx_2d_range`, which fails `ASSERT (v2)`, but only if a keysite of a searched type exists.
- **The switch has no default.** Any other sub type would read `factory` uninitialised; no sender does. The port refuses with `EechUndefinedBehaviourError` (unit-tested with `CARGO_SUPPLIES`).

## C reference

- **Compiled whole.** `fc_msgs.c` is compiled whole. So are the task accessors `ts_int.c`, `ts_float.c` and `ts_list.c`, the task database `ts_dbase.c` (it replaces the meaning-free `task_database` definition Slice 4 had), and the waypoint accessors and database `wp_int.c`, `wp_list.c` and `wp_dbase.c`.
- **Extracted verbatim.** `entity_is_object_of_task` is extracted from `task.c`, and `default_get_entity_float_value` from `en_float.c`.
- **Only the one force response is live.** The harness calls the original `overload_force_message_responses ()`, keeps only the `FORCE_LOW_ON_SUPPLIES` row, and resets every other force row to its fail-loud default.
- **Stubs.** The functions only the other responses call are fail-loud stubs: `campaign_completed`, the reactionary tasks, `engage_targets_in_group`, speech, sector defence levels, and `get_sqr_2d_range`. So are the waypoint list maintenance functions. `aircraft_database` is defined without meaning.
- **The boundary.** `create_supply_task` is the harness's recording stub (`create-supply-task …`). It returns NULL: the response reads the result only in its compiled-out `DEBUG_SUPPLY` log.

New scenario lines, mirrored by `test/scenarios/lifecycle-scenario.ts`:

| Line | Meaning |
|---|---|
| `observe-supply-tasks` | print the boundary from here on |
| `comms-model <n>` | the host's `set_comms_model` |
| `restore-group <label> …` | a group by label: sub type, side, supplies, parent keysite or independent, guide, leader position |
| `task <label> <objective> <sub_type> <side> <state> <user data>` | a restored task on its objective's task-dependent list |
| `waypoint <label> <dependent> <sub_type>` | a restored route waypoint on its dependent's task-dependent list |
| `assess-group <label>` | `assess_group_supplies` inside a lifecycle scenario |

A NULL dereference in a lifecycle scenario (reachable through `assess_group_supplies`, Slice 1) now ends the TS output the way the C fault handler does: with `result null-dereference` and the keysites' final supply levels.

## The frozen fixtures

**Every delivery keeps its trace line.** C and TS both keep the line Slices 1 and 4 recorded at this boundary (`message <force> <sender> <message> <sub type>`), and then run the real response:
- in C, a wrapper around the original function pointer;
- in TS, `test/scenarios/supply-boundary.ts`, the same wrapper over the ported table entry.

**The frozen corpus does reach the new boundary**, as the issue asked us to check:
- 6 of the 32 Slice 4 matrix cases (7 calls);
- 3 of the 150 recorded Slice 4 random scenarios (4 calls: 2 self-supplying airbases and 2 other suppliers).

**Why the fixtures still hold.** The boundary call has no other effect, so the boundary is printed only after an `observe-supply-tasks` line. Scenarios recorded before 5a do not have it, so every frozen fixture is byte-identical, with the real response installed on both sides:
- Slice 1 and 4 matrices;
- 250 + 150 + 150 + 150 recorded scenarios;
- `npm run cref:record` rewrites them unchanged.

Slice 1 never sets the game status, so the response returns at its guard there, and the Slice 1 runner asserts that it never reaches the boundary.

**What the gate cannot hide.** `test/c-reference/supply-boundary-frozen.cref.test.ts` replays the whole frozen Slice 4 corpus with the boundary observed. C and TS agree line for line, and removing the `create-supply-task` lines gives back the frozen output exactly. The response adds calls and changes nothing else.

## Conformance

| Check | Where | Size |
|---|---|---|
| Hand-derived matrix: guard, F2, fallbacks, ties, early exit, approximate against exact range, duplicate asymmetry, waypoints, group senders | `test/scenarios/force-low-on-supplies.cases.ts` | 37 cases, run in JS, Lua 5.1 and against the C (TS == C on the whole output) |
| Fresh randomised scenarios | `differential.cref.test.ts` (`generateRandomForceLowOnSupplies`, seed `0x5a12`) | 1000, C == TS line for line. It must reach self-supply, other suppliers, group senders, requests without a call, and `assert v2` / `assert receiver` |
| Recorded randomised scenarios | `c-reference-random-force-low-on-supplies.cases.ts` (seed 20261013) | 150, replayed in JS and Lua 5.1 |
| Frozen Slice 4 corpus, observed | `supply-boundary-frozen.cref.test.ts` | 32 + 150 |
| Undefined behaviour | `test/unit/force-low-on-supplies.test.ts` | the switch without a case |
| Mutations | `scripts/mutation-check.mjs`, "Slice 5a" | 23 (21 JS, 2 Lua), all killed. They include "fixing" F2, the duplicate asymmetry and the RECON quirk |

## Handed to Slice 5b

- **Everything past the boundary:** `create_supply_task`, `create_task`, the task entity's creation, its other lists and route pointers, `find_most_suitable_keysite_for_task`, `assess_task_difficulty`, and the two replication messages.
- **F1: uninitialised route heights.** Before 5b, investigate whether the historical compiler or calling convention constrained `prepare.y` and `finish.y`, or whether downstream route processing normalises them. If neither, make an explicit compatibility decision rather than presenting a chosen value as source fidelity.
- **Whether a self-supplying airbase ever gets a task** is decided past the boundary, by `create_supply_task`'s `start_ks == requester` test (5b).
