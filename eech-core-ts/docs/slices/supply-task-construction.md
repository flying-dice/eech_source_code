# Slice 5b: supply task construction (issue #14)

This slice ports `taskgen.c :: create_supply_task` → `create_task` through its
return: the supply task a force's low-on-supplies response (Slice 5a) asks for.
It rests on frozen foundations:
- Slices 1–4 and 5a;
- the round-toward-zero numerical contract (#7).

The investigation that fixed the boundary and decided F1 is
`docs/slices/supply-task-construction-investigation.md`. The user accepted all
of the following:
- **F1:** pin `prepare.y = finish.y = 0.0` as a compatibility decision (below).
- **Boundary:** `create_supply_task` → `create_task` through return. In scope:
  - start-keysite scoring and the suitability table;
  - task creation, state and list membership;
  - the difficulty calculation;
  - task ID semantics;
  - route-node construction;
  - replication ordering and the parent switch.
- **Outside:** assignment and waypoint materialisation, expiry, packing and
  destruction.
- **The campaign screen:** a semantic port, `CampaignEvents.missionCreated`,
  not UI concepts.
- **An extra acceptance criterion:** single player and multiplayer construct
  the same semantic route under the compatibility rule.

## The path

```
fc_msgs.c :: response_to_force_low_on_supplies                         (Slice 5a)
  → create_supply_task (requester, supplier, cargo, AIR, 4.0, NULL, NULL)
      stop = requester position, start = cargo position, side = requester's
      get_task_start_keysite (SUPPLY)
        → find_most_suitable_keysite_for_task (check_capacity: SUPPLY is not a ground task)
            per in-use keysite of the side with groups, suiting the task's landing types and capacity:
            score = min (Σ idle 5.0 / busy 0.5 over alive suitable groups, 12)
                  × 2 x⁴ (x = range / 100 km − 1; range ≥ 100 km skips)
                  × max (1 − 0.2 × unassigned SUPPLY tasks, 0.2)
                  × (0.5 unless usable); best strictly greater wins
      NULL, or start keysite == requester → return NULL                 (F2)
      direction = normalise (stop − start)            (x, y and z)
      prepare = stop − 4 km × direction, finish = stop + 2 km × direction, x and z only,
        each bound to the map less its 5 km perimeter; y: F1
      create_task (SUPPLY, side, AIR, start keysite, NULL, NULL, critical, 1200 s, 0, requester, 4.0,
                   start/supplier/PICK_UP, prepare/-/PREPARE_FOR_DROP_OFF,
                   stop/requester/DROP_OFF, finish/-/FINISH_DROP_OFF, terminator)
        force task_generation [SUPPLY].created ++
        route: ceil () of each node up to the terminator (compared by value); route_length = nodes − 1
        id = created, less 4095 while above 4095
        create_client_server_entity (TASK, …)                           → ENTITY_COMMS_CREATE
          ts_creat.c :: create_local: update list, the requester's task dependent list
        route pointers                                                  → ENTITY_COMMS_SET_TASK_POINTERS
        set_client_server_entity_parent (UNASSIGNED_TASK, start keysite)
          → LINK_PARENT: TASK_STATE_UNASSIGNED, notify_campaign_screen (MISSION_CREATED)
                                                                        → ENTITY_COMMS_SWITCH_PARENT
        difficulty = assess_task_difficulty (Bresenham over sectors from the start keysite)
        the objective's sector's task list
      TASK_USER_DATA = cargo sub type
```

| C | TS |
|---|---|
| `taskgen.c :: create_supply_task`, `create_task`, `get_task_start_keysite`, `validate_task_generation` (returns `TRUE`), `terminator_point` | `src/ai/taskgen/taskgen.ts` |
| `task.c :: find_most_suitable_keysite_for_task`, `assess_task_difficulty`, `assess_task_sector_difficulty`, `get_local_task_list_type` | `src/entity/special/task/task.ts` |
| `suitable.c` (the group-to-task suitability table, built at start-up) | `src/ai/highlevl/suitable.ts` |
| `ts_creat.c` (server), `ts_int.c`, `ts_float.c`, `ts_ptr.c`, `ts_list.c`, `ts_msgs.c :: response_to_link_parent` (unassigned arm) | `ts_creat.ts`, `task.ts` |
| `ts_dbase.c`, `gp_dbase.c`, `ks_dbase.c` columns | `src/generated/*` (generated from the C, drift-checked) |
| `en_list.c :: set_client_server_entity_parent` (server) | `setClientServerEntityParent` |
| `en_comms.c :: transmit_entity_comms_message` (single player trap; `SET_TASK_POINTERS`, `SWITCH_PARENT`); `en_vec3d.c :: pack_vec3d` (position check) | `src/entity/system/en_comms.ts` → `EntityReplication` |
| `ca_msgs.c :: notify_campaign_screen` | `src/ui_menu/campaign/ca_msgs.ts` → `CampaignEvents` |
| `gametype.c :: game_type` | `src/core/game-type.ts` (core state, like the game status) |
| `sector.c` enemy defence levels, `sc_int.c :: SECTOR_SIDE`; `vector.c :: normalise_any_3d_vector`; `en_world.c :: bound_position_to_adjusted_map_area` | `sector.ts`, `vector.ts`, `en_world.ts` |
| `gp_int.c` (`ENTITY_SUB_TYPE`, `ALIVE`), `gp_msgs.c :: response_to_link_child`; `ks_int.c` (`LANDING_TYPES`, `KEYSITE_USABLE_STATE`), `ks_list.c :: unassigned_task_root` | `group.ts`, `keysite.ts` |

`create_task` takes the route as an array (`RouteNodeArgument`), in place of
the C variable arguments. The paths other task types take are ported with it:
- no start keysite;
- an objective of another side (it notifies the enemy force; `TASK_CREATED` is unported);
- no objective;
- a NULL node;
- a stop timer.

Unit tests cover these paths, and they fail loudly where their callees are
unported.

## F1: a compatibility decision

`create_supply_task` sets only the x and z of `prepare` and `finish`, which
are automatic variables. `create_task` stores `ceil (position->y)` of each in
the route. EECH defines no value, and the heights reach several places:
- the saved route;
- the multiplayer `ENTITY_COMMS_SET_TASK_POINTERS`, where `pack_vec3d` asserts
  in debug builds or clamps the task's own node in place in release builds;
- the route checksum.

The investigation found that no compiler, calling convention or downstream
step makes the value reproducible or normalises it before it is observable.

**Decision: the port sets both to 0.0.** This resolves undefined behaviour in
the EECH source. It does not translate a value EECH defined, and nothing claims
EECH produced 0.0. The C reference pins the otherwise unmodified original to
the same choice. It compiles `eech_extracted_taskgen.c` with
`-ftrivial-auto-var-init=zero`, and nothing else is built that way. The same
distinction is recorded in `docs/port-manifest.md`.

**The evidence is kept.** `npm run probe:f1` (`test/f1-probe`) builds that one
unit zero-, pattern- and un-initialised, at -O0 and -O2. It runs the
acceptance pair below through each:

| build | stored `prepare.y`, `finish.y` | single player | multiplayer |
|---|---|---|---|
| zero -O0 (the harness's pinning) | `00000000 00000000` (0.0) | ok | ok |
| zero -O2 | `00000000 00000000` | ok | ok |
| pattern -O0 / -O2 | `fefefefe fefefefe` (−1.7e38) | ok | `ASSERT (point_inside_map_volume (v) \|\| v->y == -10000)` |
| uninitialised -O0 | `ffe82e68 565d8f54` (a NaN; a code address) | ok | the same `ASSERT` |
| uninitialised -O2 | `f7cc81a9 4e15c900` (−8.3e33; 6.3e8) | ok | the same `ASSERT` |

The uninitialised heights are stack or register residue of earlier calls. One
is a code address, which address-space randomisation changes from run to run.
Single player never packs, so it keeps whatever height it got. A transmitting
session checks the route (debug) or clamps it in place (release). **The modes
could diverge.**

**Acceptance case** (`f1CompatibilityPair`, in the JS, Lua and C runners). The
base scenario runs once as a transmitting server and once as single player.
Under 0.0:
- the two entity graphs are identical, route and all;
- the multiplayer `transmit-task-pointers` carries exactly the stored nodes;
- single player transmits nothing.

A keysite whose own height is outside the map volume still makes multiplayer
packing assert where single player does not. That is EECH behaviour, not F1,
and a matrix case pins it (`multiplayer-packing-asserts-on-a-drop-off-above-the-map`,
`single-player-keeps-a-drop-off-above-the-map`).

## Ports and core state

- **`CampaignEvents.missionCreated (taskIndex)`**: the semantic event of
  `notify_campaign_screen (CAMPAIGN_SCREEN_MISSION_CREATED, task)`. The guard
  is core: game status INITIALISED and game type CAMPAIGN or SKIRMISH
  (`DEMO_VERSION` 0). The screen's response table is the UI and stays outside.
- **`EntityReplication.transmitTaskPointers`, `transmitSwitchParent`**: what
  `ENTITY_COMMS_SET_TASK_POINTERS` and `ENTITY_COMMS_SWITCH_PARENT` carry.
- **`setEntityCommsTransmission (active)`**: `transmit_entity_comms_message`'s
  single player trap
  (`direct_play_get_comms_mode () == DIRECT_PLAY_COMMS_MODE_NONE`). Every
  transmission, including the three call sites of earlier slices, now goes
  through `src/entity/system/en_comms.ts`. Transmission starts on, as every
  earlier slice assumed. Packing a task's route checks each node the way
  `pack_vec3d` does, with debug-build meaning (the `ASSERT` fails).
- **`setGameType`**: the front end's `game_type`, core state reset by
  `initialiseCampaignCore`.
- **`observeCreateSupplyTask`** replaces Slice 5a's interceptor. It is a test
  seam for this one function: it sees each call's arguments before the ported
  function runs, and cannot change what it does or returns. The review rule
  holds: unported campaign behaviour always fails loudly, and observation does
  not alter control flow.

## Original behaviour, preserved and pinned by C-derived cases

`test/scenarios/supply-task-construction.cases.ts` has 38 cases. Each keeps
what the source does, including:

- **Replication order.** `ENTITY_COMMS_CREATE`, then
  `ENTITY_COMMS_SET_TASK_POINTERS`, then the campaign screen, then
  `ENTITY_COMMS_SWITCH_PARENT`. The difficulty, `TASK_USER_DATA` and the
  sector task list are set locally only. The task pointers carry
  `route_length` nodes: the terminator is not sent.
- **Task ids.** The force's counter increments before anything else, even for
  a task whose later steps fail. The id is the counter less 4095 while above
  4095: it wraps by 4095, not 4096. 4095 becomes 1 and 8190 becomes 1, so 0
  never occurs (four cases).
- **F2 settled.** A self-supplying airbase gets a task only when another
  keysite is the start keysite. When it would start the task itself,
  `start_ks == requester` returns NULL. The response's call happens, and
  nothing is counted or created.
- **Scoring:**
  - groups count only when alive and suitable; a busy group adds 0.5 against
    an idle group's 5.0;
  - the group score caps at 12;
  - equal scores keep the first keysite in the force's list;
  - unassigned tasks of the type discount a keysite (0.8 each step, floor
    0.2), and tasks of other types do not;
  - a keysite that is not usable scores half;
  - only an airbase has the LARGE capacity supply tasks need;
  - a keysite 100 km or more away is out of range, measured by
    `get_approx_2d_range`;
  - ground tasks (e.g. ADVANCE) ignore capacity and landing types, and score
    every keysite with groups at 12 (unit test).
- **Difficulty:**
  - the walk starts at the start keysite (the task link parent), because the
    parent switch comes before the assessment;
  - it visits sectors with Bresenham's algorithm, both arms and diagonal
    steps, and counts the last node once more;
  - enemy surface-to-air levels exclude the task's side and neutral;
  - a sector is RED unless blue presence is strictly greater, so a tie is
    RED;
  - each term is `min (n >> 1, 5)`;
  - `task_database [].difficulty_rating` is read into an unused local.
- **Geometry:**
  - the direction is normalised in three dimensions, so a height difference
    shortens the horizontal offsets;
  - cargo on the requester gives a zero direction, and prepare and finish
    collapse onto the drop-off;
  - the derived points are kept 5 km inside the map edge.
- **Sector list.** The task goes on the objective's sector's task list, not
  the last route node's: a FARP just inside sector 1 files its task there,
  though its finish point is in sector 0. A front line group's task files
  under the group's (its leader's) position.
- **The campaign screen** hears about primary tasks only, and only in a
  running campaign or skirmish.

## C reference

- **Compiled whole:** `task.c`, `ts_creat.c`, `ts_ptr.c` and `suitable.c`.
- **Extracted verbatim:**
  - from `taskgen.c`: `create_task`, `create_supply_task`,
    `get_task_start_keysite`, `validate_task_generation` and
    `terminator_point`, into `eech_extracted_taskgen.c` (the rest of the file
    reaches the 3D engine);
  - `pack_vec3d` and `vec3d_type_database`;
  - `notify_campaign_screen`;
  - `set_client_server_entity_parent`;
  - `normalise_any_3d_vector` and `bound_position_to_adjusted_map_area`;
  - the sector enemy defence levels;
  - the task's `response_to_link_parent` and the group's
    `response_to_link_child`.
- **The `--wrap`.** `fc_msgs.c` reaches the original `create_supply_task`
  through `--wrap=create_supply_task`, which prints Slice 5a's boundary line
  first.
- **Transmissions.** The transmit handler packs a task's route with the
  original `pack_vec3d` before it prints anything. A failed check therefore
  ends the operation before the line starts.

New scenario lines, mirrored by `test/scenarios/lifecycle-scenario.ts`:

| Line | Meaning |
|---|---|
| `observe-tasks` | print tasks (values, route, list parents), the forces' supply task counters, the keysites' unassigned and task dependent lists, and sectors' task lists |
| `single-player` | a single player session: nothing is transmitted or packed |
| `game-type <n>` | the front end's game type |
| `keysite-landing <keysite> <landing types> <usable state>` | a keysite's raw bit-fields, as a saved game holds them |
| `group-alive <group> <alive>` | a restored group's raw alive bit |
| `sector-state <sector> <blue> <red> <SAM neutral> <SAM blue> <SAM red>` | a sector's raw side presence and surface-to-air defence levels |
| `task-counter <force> <sub type> <created>` | a force's raw task generation counter |

Tasks the original creates are labelled `task<index>` when they are first
printed.

## The frozen fixtures

Every frozen fixture is byte-identical, 5a's included, and
`npm run cref:record` rewrites them unchanged. The frozen corpora do now run
the real construction whenever they reach the boundary. None of them gives a
keysite landing types, which a restored keysite leaves at 0, so no start
keysite exists, `create_supply_task` returns NULL, and nothing is created,
counted or transmitted.

## Conformance

| Check | Where | Size |
|---|---|---|
| Hand-derived matrix | `test/scenarios/supply-task-construction.cases.ts` | 38 cases, run in JS, Lua 5.1 and against the C (TS == C on the whole output) |
| F1 acceptance: single player = multiplayer route | `f1CompatibilityPair` | JS, Lua 5.1, C (both outputs also equal the TS) |
| Paths other task types take; loud failures; the observer | `test/unit/supply-task-construction.test.ts` | unit tests |
| Fresh randomised scenarios | `differential.cref.test.ts` (`generateRandomSupplyTaskConstruction`, seed `0x5b14`) | 1000, C == TS line for line. It must reach tasks with and without difficulty, ids at the wrap, two unassigned tasks at a keysite, single player tasks, calls without a task, and the packing `ASSERT` |
| Recorded randomised scenarios | `c-reference-random-supply-task-construction.cases.ts` (seed 20261014) | 150, replayed in JS and Lua 5.1 |
| Coverage | `npm run coverage` | 100% statements, branches, functions and lines. One justified exclusion: `suitable.c`'s movement stealth rejection, unreachable with EECH's databases, recomputed by a unit test |
| Mutations | `scripts/mutation-check.mjs`, "Slice 5b" | 43 (40 JS, 3 Lua), all killed. They include "fixing" the 4095 wrap, F2, the F1 height, the single player trap and the packing check, and turning unported arms into returns |
| F1 probe | `npm run probe:f1` (investigation only) | 6 builds of the one unit |

## Handed on

- **Task assignment:** `assign.c`, the assigned list and `TASK_STATE_ASSIGNED`,
  which throws unported in `response_to_link_parent`.
- **Waypoint materialisation:** `croute.c`.
- **Also unported:** expiry, task completion, destruction (a task's
  `UNLINK_PARENT` is unported), and packing for saved games.
- **The other task generators** in `taskgen.c`.
