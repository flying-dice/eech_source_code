# eech-core-ts architecture report

This is the architecture report for the frozen EECH campaign-core port. It was
written at bootstrap (issue #1, slice 1) and is kept current: slice 2 (issue #3)
added update timing, the `Clock` port and the first C-reference shim reduction.
The original C under `aphavoc/` and `modules/` is the behavioural authority. The
current `ee-dcs` implementation is not used as a reference anywhere.

```
Original EECH C ──(behavioural authority, executed by c-reference/)──┐
                                                                     v
                        +--------------------------------------+
                        | eech-core-ts: frozen campaign core   |
                        | TypeScript -> TSTL -> Lua 5.1        |
                        +------------------+-------------------+
                                           | src/ports (narrow, EECH-derived)
                         +-----------------+-----------------+
                         |                                   |
               test/adapters (deterministic)         DCS adapters (future)
```

## 1. Which EECH modules make up the likely campaign kernel?

These are the modules that own campaign state and decide campaign policy. Sizes
are for the `.c` files only.

| Area | Modules | Size | Role |
|---|---|---|---|
| Entity graph | `entity/special/session`, `force`, `division`, `group`, `keysite`, `task`, `guide`, `waypoint`, `landing`, `sector`, `regen` | ~66k lines | Campaign state: who exists, who owns what, supply levels, tasks, routes, regeneration. |
| High-level AI | `ai/highlevl` (highlevl.c, order.c, reaction.c, imaps.c, setup.c) | ~7k | Force strategy, influence maps, reactions. |
| Task generation and assignment | `ai/taskgen` (taskgen.c, assign.c, engage.c, croute.c) | ~7k | Creating tasks, suitability, assignment to groups. |
| Frontline | `ai/frontl` | 0.4k | Frontline geometry. |
| Faction and campaign data | `ai/faction` (faction.c, routegen.c, briefing.c, popread.c, parser.c) | ~12k | Population and campaign file loading, route generation. Much of this is data loading that a `CampaignStore` port will front. |
| Misc AI | `ai/ai_misc` (ai_misc.c, ai_route.c, ai_dbase.c) | ~3k | Shared helpers and AI databases. |
| Entity runtime | `entity/system/en_funcs`, `en_msgs`, `en_types`, `en_main` | | Typed values, lists, messages and function tables. The kernel is written against these. |

The entity graph is the campaign model. It is ported as an entity graph, with no
DCS objects standing in for it.

## 2. Which entity-system facilities does the kernel need?

Slices 1 and 2 needed these, and all of them are ported in `src/entity/system`
and `src/entity/special/update`:

- identity, type and raw data (`entity.ts`; C: `en_main.c`)
- per-type list roots and links, including **shared links**: a group's
  `LIST_TYPE_BUILDING_GROUP`, `INDEPENDENT_GROUP` and `KEYSITE_GROUP` all resolve
  to one `group_link` (`en_list.ts`; C: `en_list.c`, `en_list/get_prnt.h`)
- typed value dispatch through per-type function tables: int, float, vec3d and ptr
  (`en_values.ts`; C: `fn_get_local_entity_*_value[type][value_type]`)
- client/server dispatch, `set_client_server_entity_float_value [..][comms_model]`,
  with the server path "set local, then transmit" (`comms.ts`, `serverFloatValueSetter`)
- messages: `notify_local_entity` and the `message_responses` table (`en_msgs.ts`)
- list insertion and deletion **with** link notifications (`en_list.c`, slice 2).
  They can only be used on lists whose parent and child responses are ported:
  group, keysite, task, regen and waypoint respond to these messages, and an
  unported response fails loudly.
- the entity update loop and update dispatch (`up_update.c`, `en_updt.c`,
  `LIST_TYPE_UPDATE`, slice 2)
- the campaign side of time: `get_delta_time`, `set_manual_delta_time` and
  `locked_frame_rate` (`src/core/time.ts`, slice 2)

The kernel will need these later. They are not ported yet:

- creation and destruction with attribute lists (`en_creat.c`, `en_dstry.c`)
- update functions of entity types other than the group
- pack/unpack, i.e. save games (`en_pack.c`)
- string values

**Convention.** A C `switch` in `get_local_int_value` becomes one table entry per
`case`. The C `default:` arm (`debug_fatal_invalid_*_type`) is represented by the
table lookup failing. Every table entry starts **unported**, and reaching one
throws `UnportedBehaviourError`. The port never silently falls back to an EECH
default that the real overload would have replaced.

## 3. Which dependencies cross into physical simulation?

Measured over the kernel directories listed in question 1:

| Dependency | References | Classification |
|---|---|---|
| `get_local_entity_vec3d_ptr (.., VEC3D_TYPE_POSITION)` on mobiles | 178 in 35 files | Physical observation. The flight model or vehicle simulation writes `mob.position`. **Port: `MobilePhysicalState`.** |
| terrain (`get_3d_terrain_*`, `terrain_*`) | 158 in 17 files | Environmental. Becomes a future `Terrain` port. |
| `set_client_server_entity_*` / `transmit_entity_comms_message` | 381 / 255 | Network transport, which DCS replaces. The semantic contract is "authoritative value changed". **Port: `EntityReplication`.** |
| `create_/destroy_client_server_entity` | 109 in 31 files | Campaign-core entity lifecycle. It becomes a physical materialisation port where a mobile appears in or leaves the world. |
| `get_delta_time`, `get_system_time`, time acceleration | 40 in 6 files | Environmental measurement. **Port: `Clock`** (slice 2). The campaign's override of the delta (`set_manual_delta_time`) is core. |
| `frand1`, `sfrand1`, `rand16` | 48 in 14 files | Environmental. Future `RandomSource` port, which must reproduce EECH's generator sequence. |
| sound and speech (`play_client_server_*`, speech) | 59 in 7 files | Physical/presentation. Excluded, or an optional notification port. |
| flight dynamics, velocities | 20 in 5 files | Physical simulation, which DCS replaces. Excluded. Only the semantic observation (landed, taken off, arrived) crosses a port. |

The mobile entities themselves (`entity/mobile/**`: aircraft, vehicles, weapons)
are physical simulation. The port keeps only their campaign-visible surface:
membership, type, side and position. See `src/entity/mobile/mobile.ts`.

## 4. Which ports emerge naturally?

The ports introduced so far, each derived from C call sites:

| Port | C origin | Test adapter |
|---|---|---|
| `MobilePhysicalState.getMobilePosition` | `ac_vec3d.c` / `vh_vec3d.c` `get_local_vec3d_ptr (VEC3D_TYPE_POSITION)` | `InMemoryMobilePhysicalState` (positions come only from the scenario, never invented) |
| `EntityReplication.transmitEntityFloatValue` | `set_server_float_value` → `transmit_entity_comms_message (ENTITY_COMMS_FLOAT_VALUE, ..)` | `RecordingEntityReplication` (ordered log) |
| `Clock.getDeltaTime`, `Clock.isFrameRateLocked` (slice 2) | `time.c :: set_delta_time` (frame measurement) and `locked_frame_rate` | `ScriptedClock` (the frame driver states each frame) |
| `EntityReplication.transmitEntityCreate`, `transmitEntityDestroy` (slice 3) | `transmit_entity_comms_message (ENTITY_COMMS_CREATE / ENTITY_COMMS_DESTROY, ..)` | `RecordingEntityReplication` |
| `Object3DMetadata.getBoundingBox` (slice 4) | `3dobjvis.c :: get_object_3d_bounding_box (object_3d_index_numbers)`, read by `keysite.c :: update_keysite_cargo` (and `sc_msgs.c` for fixed entities) | `InMemoryObject3DMetadata` (bounds only as the scenario declares them) |

The game status (`global.c :: game_status`) is deliberately **not** a port: it is
the campaign's own lifecycle phase, moved by the host's game flow like the comms
model, so it is core state with a setter (`setGameStatus`; slice 4,
`docs/slices/keysite-cargo.md`, Investigation 3).

These are the next ports in order of need. None is created before a ported slice
calls it.

- `RandomSource`: `rand16`, `frand1`, `sfrand1`. The adapter must reproduce the
  EECH generator bit for bit, so the C harness can verify it.
- `Terrain`: elevation and terrain type queries.
- `WorldCommands`: create/destroy of physical members, and task/route hand-over
  to a physical group.
- `CombatObservation`, `LandingObservation`: the events the `mb_msgs.c` handlers
  react to (landed, damaged, killed). `assess_group_supplies` is itself called
  from two of them.
- `CampaignStore`: loading campaign/population files and save games (`ai/faction`
  parsers, `en_pack.c`).

**On scheduling (implemented in slice 2).** EECH has no callback scheduler.
Each frame the host measures time (`set_delta_time`), then calls
`update_client_server_entities ()` once per time-acceleration step. That walks
`LIST_TYPE_UPDATE`, calling each entity's update function, and splits the frame
into `(int) (delta * rate + 1)` equal float sub-steps. The deterministic
"virtual scheduler" is therefore:
- the `ScriptedClock` plus a host-loop driver (`test/scenarios/update-timeline.ts`);
- running the **ported** update loop, which executes the real campaign update
  functions in list order. That includes entities that remove themselves during
  the walk, and entities inserted at the head, which are not visited until the
  next pass.

The runaway guard is test-side (`validateTimeline`): it refuses frames that
would need more than 10,000 update passes, and never changes a result. Tests
never call private update functions directly.

**Slice 3** extends `EntityReplication` with `transmitEntityCreate` and
`transmitEntityDestroy` (EECH's `ENTITY_COMMS_CREATE` / `ENTITY_COMMS_DESTROY`).
It adds no port for the world map: its size is campaign data that EECH's
campaign script parser passes to `set_entity_world_map_size`, and the host does
the same. The 3D object dimensions (`get_object_3d_bounding_box`) are
environmental and become a port in slice 4.

**Calling convention for Lua hosts.** Ports are objects. The transpiled core
calls their functions as methods (`port:method (...)`), so a Lua host's port
functions take the port table first, for example
`transmitEntityDestroy = function (self, entityIndex) ... end`
(`test/lua/smoke.lua`).

## 5. Which function is the first vertical slice, and why?

**`group.c :: assess_group_supplies`**, with its callee
**`keysite.c :: get_closest_keysite`**. It meets every criterion in the issue:

1. **Genuine campaign behaviour.** Groups that resupply via mission ask their
   force for a supply mission. Idle air groups rearm and refuel from keysite stock.
2. **Exercises the entity model.** It uses int/float/vec3d/ptr dispatch,
   database-derived values (`group_database [sub_type].resupply_source`), a derived
   value (`INT_TYPE_GROUP_MODE` from the guide stack), shared list links, force
   lookup through the session, and a message to the force.
3. **Needs environmental ports.** The group's position is the leader's physical
   position, and every supply write is a replicated server write.
4. **Exhaustively testable.** The full behaviour matrix is 41 cases (see
   `docs/slices/assess-group-supplies.md`).
5. **Transpiles through TSTL.** It runs under Lua 5.1 against the same matrix.
6. **Bounded.** Five C functions plus the accessors they reach.

The force's reaction (`fc_msgs.c :: response_to_force_low_on_supplies`) creates
supply tasks and pulls in task creation. The slice stopped at that message
boundary, recording each delivery. Slice 5a ports the response behind the same
trace line, up to its own boundary, `create_supply_task`, which tests record
through a seam for that one function (`interceptCreateSupplyTask`); without it the call fails loudly.

## 6. What enables or blocks running the C implementation under a harness?

It works, and since slice 2 most of it is original code rather than shim.
`c-reference/extract.mjs` generates:
- **`build/c-reference/project.h`**: the harness environment, verbatim
  fragments, and **whole original headers**, including every entity dispatch
  macro and table declaration;
- **`eech_extracted.c`**: verbatim functions from files too large to compile
  whole, each with a `#line` directive back to its original file.

Nine original translation units then compile **unchanged**: `gp_int.c`,
`gp_float.c`, `gp_list.c`, `gp_vec3d.c`, `gp_ptr.c`, `gp_updt.c`, `gp_dbase.c`,
`up_list.c` and `up_msgs.c`. Dispatch goes through the original `fn_*` tables,
filled by the original `overload_*_functions ()`. The harness defines the
tables, the environment, and hand-written rows only for entity types whose files
are not compiled yet (see "Shrinking the C reference shim").

What enables this:

- EECH's headers are well layered; a curated include order plus a few fragments
  satisfies the original files. No reconstruction of `project.h` was needed
  (`docs/slices/group-update-timing.md`, investigation 3).
- `DEBUG_MODULE` / `DEBUG_SUPPLY` logging is compiled out.
- The extractor and the build fail loudly if a definition disappears or a
  prototype is missing (`-Werror=implicit-function-declaration`).

What limits it or blocks it next:

- **Keysite, force and session accessors are still hand-written.** Compiling
  `ks_*.c`, `fc_*.c` and `ss_*.c` needs their headers in the reduced
  `project.h`, and fail-loud stubs for their other dependencies. This is the
  next shim-reduction step.
- **Floating point.** The harness evaluates C float arithmetic at declared
  type (`FLT_EVAL_METHOD == 0`, SSE) with the rounding EECH sets on its
  campaign thread: toward zero (issue #7). `test/c-reference/fpu-environment.cref.test.ts`
  pins this. The historical x87 builds may have evaluated some intermediates
  beyond declared type; that question is unresolved and needs a Windows
  runtime trace. See "EECH numerical contract" in
  `docs/fidelity/fpu-semantics.md`.
- **Varargs messages.** They work unchanged. The shim installs the response for
  the one message it observes, and any other message aborts the harness.
- **Global state** (`session_entity`, the dispatch tables, `entities`) is
  defined by the harness. The frame delta (`set_delta_time`) is supplied from
  the same scenario as the TS `ScriptedClock`. Slices that use `rand16` or
  terrain need those environment functions in the harness, driven the same way.

## 7. Which coverage tooling enforces 100% reachable TS coverage?

Vitest with `@vitest/coverage-istanbul`. `vitest.config.ts` sets `thresholds` of
100 for statements, branches, functions and lines over `src/**`, with no exclusions. Istanbul counts
implicit `else` branches, `??` and default parameters. Type-only modules such as `src/ports/**`
emit no statements; if one starts emitting runtime code it is measured and must
be covered like anything else. There are **no**
`istanbul ignore` comments in campaign code. A future unreachable defensive path
must be excluded individually in the source and listed in
`docs/port-manifest.md`.

To avoid unreachable code in the first place, C `default:` arms that only guard
against invalid dispatch are represented by the function-table lookup (question
2). C `debug_assert` / `ASSERT` checks use the `ASSERT` helper, which keeps the
check without adding a branch to the caller.

## 8. How are TSTL/Lua semantics exercised rather than only JavaScript semantics?

- `npm run test:lua` transpiles `src/` plus the shared scenarios with TSTL
  (`tsconfig.lua-test.json`) and **executes them in a real Lua 5.1 interpreter**:
  both behaviour matrices (41 supply cases, 22 timeline cases), 250 scenarios
  and 150 timelines whose outcomes were recorded from the executed C, and the
  float32 edge cases. That is 485 checks. The runner refuses
  to run on anything but `_VERSION == "Lua 5.1"`, the version DCS embeds.
- `npm run smoke:lua` loads the production bundle `build/lua/eech-core.lua` from
  plain Lua with Lua-table ports, as a DCS host would, and runs a campaign
  operation.
- TSTL diagnostics fail the build (`scripts/lua.mjs build`). ESLint's
  `strict-boolean-expressions` forbids numeric and string truthiness, because
  `0` and `""` are true in Lua.
- C `float` semantics use pure arithmetic that exists in both runtimes
  (`src/core/float32.ts`). EECH's run-time float results round toward zero:
  `toFloat32RTZ`, `f32Add`, `f32Sub`, `f32Mul`, `f32Div` and `f32Sqrt`. They
  are verified bit for bit against the C oracle's float arithmetic on 40,000
  fresh operations. A recorded set of 3,000 is also replayed in JavaScript and
  in Lua 5.1. `toFloat32` (round to nearest) remains for compile-time constants
  and scenario input. It is verified against `Math.fround` on 400,000 inputs in
  JavaScript, and on the edge cases under Lua. `Math.fround` and `Math.log2`
  are lint-banned in `src`.
- The Lua runner has already found two real divergences during bootstrap. Both
  are now documented in the test data:
  1. Lua 5.1 merges the literal `-0` into `0` in the constant table.
  2. `{ ...defaults, leader: undefined }` keeps the default in Lua, because a
     table cannot hold `nil`. Optional scenario values are therefore explicit
     variants.
- Two mutants only fail under Lua: `in_use` (slice 1) and the timer setters'
  `value != 0.0` (slice 2), each tested by JavaScript truthiness. They prove the
  Lua run adds detection power beyond the JavaScript run.

## 9. How is C→TS migration tracked?

- `docs/port-manifest.md` is authoritative. It lists every touched C file and
  function with its status (`unported`, `ported`, `tested`, `100%-covered`,
  `C-reference-verified`, `blocked-engine-boundary`,
  `excluded-physical-simulation`). A module is only called ported when all of it
  is.
- Every TS module and function carries a `C provenance:` comment naming the file
  and function it came from.
- Enum ordinals, database columns and numeric `#define` constants are **generated** from the C sources
  (`npm run gen:c`, drift check `npm run check:c`). They are never typed by hand.
- Unported table entries fail at runtime with the name of the missing C overload.

## 10. What should the next slices be?

1. ~~**Group update and sleep timers**~~ Done in slice 2
   (`docs/slices/group-update-timing.md`).
2. ~~**The entity lifecycle, CARGO and sector membership**~~ Done in slice 3
   (`docs/slices/entity-lifecycle-cargo.md`). It was found underneath
   `update_keysite_cargo` when that was investigated as slice 3.
3. ~~**`keysite.c :: update_keysite_cargo`**~~ Done in slice 4
   (`docs/slices/keysite-cargo.md`): ported intact on slice 3's lifecycle, with
   the `Object3DMetadata` port keyed by `object_3d_index_numbers`
   (`get_object_3d_bounding_box`, also read by the sector link response for
   fixed entities) and the game status as core state.
4. **`fc_msgs.c :: response_to_force_low_on_supplies`** (slice 5a, issue #12;
   `docs/slices/force-low-on-supplies.md`): the duplicate-task decision and the
   supplier and cargo choice, up to the `create_supply_task` boundary. **Slice
   5b**: `create_supply_task`, `create_task` and the task entity, start-keysite
   scoring, difficulty and route construction, after the F1 investigation of
   the uninitialised route heights (`docs/slices/supply-task-investigation.md`).
5. **Pickup, transport and delivery** (the `mb_msgs.c` waypoint handlers, cargo
   movement), and the landing handlers that call `assess_group_supplies`. These
   introduce a `LandingObservation`-style port: the DCS adapter reports that a
   member landed at a keysite, and the campaign does the accounting.
6. **`ks_updt.c`**, composed from frozen parts once task assignment, repair and
   supply drift are ported.
7. **The readers of `sleep`**, e.g. task assignment and the `mb_msgs.c` landing
   handler that sets `sleep` after rearming. These give slice 2's timers their
   campaign meaning.
8. **The FPU rounding mode.** Slice 3 found that EECH runs the x87 FPU in
   round-toward-zero mode. Before more float-heavy slices, decide whether the
   harness and the port should model that mode (see the manifest's deviations).

## Shrinking the C reference shim

The C reference is only as strong as the part of it that is original EECH. Slice
1 executes the original campaign functions verbatim, but `c-reference/harness.c`
still implements, by hand, the accessor behaviour those functions call. The goal
is to replace each hand-written shim entry with the real EECH translation unit,
until the shim only supplies what is truly environmental: time, randomness,
terrain, physical positions and the comms transport.

### Current shim surface

The harness builds a reduced `project.h` from the harness environment, verbatim
fragments and **whole original headers**, and compiles original translation
units unchanged (`c-reference/extract.mjs`, `REAL_TRANSLATION_UNITS`). Dispatch
goes through the original `fn_*` tables, filled by the original
`overload_*_functions ()`.

**Retired** (now executed from the original C):

| Retired shim entry | Original code now executed | Slice |
|---|---|---|
| Group `INT_TYPE_RESUPPLY_SOURCE`, `INT_TYPE_GROUP_MODE`, `INT_TYPE_SIDE` | `gp_int.c` | 2 |
| Group `resupply_source` passed in by the scenario | `gp_dbase.c :: group_database` | 2 |
| Group supply level get/set, timer get/set | `gp_float.c` | 2 |
| Group position and leader | `gp_vec3d.c`, `gp_ptr.c` | 2 |
| Group list roots and links, including the shared `group_link` | `gp_list.c` with `en_list/*.h` | 2 |
| Hand-written NULL checks | the original unguarded code; a `SIGSEGV` is the reported outcome | 2 |
| Keysite `INT_TYPE_ENTITY_SUB_TYPE`, `INT_TYPE_IN_USE` | `ks_int.c` | 3 |
| Keysite supply level get and server set | `ks_float.c` | 3 |
| Keysite `VEC3D_TYPE_POSITION` | `ks_vec3d.c` | 3 |
| Keysite list roots and links | `ks_list.c` | 3 |
| Force `INT_TYPE_SIDE` | `fc_int.c` | 3 |
| Force list roots and links | `fc_list.c` | 3 |
| The harness's own entity array | `en_heap.c`: scenario entities come from the original heap | 3 |
| "Not supplied" default for int getters | `en_int.c :: default_get_entity_int_value`, extracted verbatim | 3 |
| Recording of `FORCE_LOW_ON_SUPPLIES` deliveries in place of the response | `fc_msgs.c` compiled whole; the original response runs behind the unchanged trace line | 5a |
| `task_database` defined without meaning | `ts_dbase.c :: task_database` | 5a |

**Remaining** (hand-written in `harness.c`):

| Shim entry | Original source that should replace it |
|---|---|
| List storage of session, guide and helicopter | `ss_list.c`, `gd_list.c`, `ac_list.c` |

**Environment** (legitimately hand-written, driven by the same scenario data as
the TS adapters):
- the frame delta and locked flag (`set_delta_time`);
- the host loop's time acceleration;
- the comms model and data flow (server, TX);
- the transport (`transmit_entity_comms_message`: float values, CREATE,
  DESTROY);
- the mobile position (physical);
- memory (`malloc_fast_mem`, `malloc_heap_mem`, `free_mem`);
- `convert_float_to_int` (x87 `fistp` under EECH's round-toward-zero mode:
  truncation);
- the player's gunship (`gunship_entity`, always NULL: the campaign core has
  none);
- debug output;
- tacview (never logging);
- the Windows SDK `min`/`max`.

The platform itself is part of the environment: the harness is built for 32-bit
x86 (`-m32`, SSE float arithmetic), because the original creation path
reinterprets a `va_list` as the argument stack (`c-reference/README.md`,
"Platform").

**Fail-loud stubs** for code that is reachable only outside the adopted
behaviour. They fail the run if reached.
- **Slice 1:** `add/remove_group_type_to/from_force_info`,
  `set_local_division_name`, and every dispatch-table default except the C
  default setters and the default int getter, which are extracted verbatim.
- **Slice 3:**
  - `pack_*` / `unpack_*` (saved games and network messages);
  - `set_comms_model`, `set_comms_data_flow`,
    `enable/disable_entity_comms_messages` (only
    `create_local_only_entities` calls them);
  - the creation and destruction of pylons, bridges, the camera and the update
    entity, plus `destroy_local_sector_entities`, `destroy_local_sound_effects`
    and `set_gunship_entity`;
  - `set_sector_fog_of_war_value` and `update_imap_surface_to_*_defence_level`;
  - `get_valid_current_game_session` and `get_current_game_session_type`
    (macros over session state in the original, declared as functions here so
    any use fails);
  - `get_local_group_member_landing_entity_from_keysite`.
- **Slice 4:** `keysite.c` is compiled whole (replacing the extract of
  `get_closest_keysite`). Its functions outside the port (FARP enabling,
  importance, attack notification, destruction, capture, repair, dumps,
  landing sites, speech, MFD names) are never called by the harness and are in
  no dispatch table; the 31 functions only they call are fail-loud stubs, and
  the data only they read (`random_number_seed`,
  `command_line_capture_aircraft`, `speech_sector_coordinates`, the side name
  tables, `task_database`, the string accessor table with a fail-loud default)
  is defined without meaning (`task_database` is the original since slice 5a).
- **Slice 5a:** `fc_msgs.c` is compiled whole, but only its
  `FORCE_LOW_ON_SUPPLIES` row is kept in the dispatch table; the functions only
  the other responses call (`campaign_completed`, the reactionary tasks,
  `engage_targets_in_group`, speech, sector defence levels, `get_sqr_2d_range`)
  and the waypoint list maintenance (`update_local_entity_waypoint_list_tags`,
  `get_formation_database_count`) are fail-loud stubs, and `aircraft_database`
  is defined without meaning. The task and waypoint accessors (`ts_int.c`,
  `ts_float.c`, `ts_list.c`, `wp_int.c`, `wp_list.c`, `wp_dbase.c`) are compiled
  whole; `entity_is_object_of_task` and `en_float.c ::
  default_get_entity_float_value` are extracted verbatim.
  `create_supply_task` (slice 5b) is the recording boundary.

**Environment entries driven by the scenario** (slice 4):
- `get_object_3d_bounding_box`: the scenario's `bounds` lines (the 3D object
  database), mirrored by `test/adapters/in-memory-object-3d-metadata.ts`; an
  undeclared object fails loudly.
- `game_status`: the original `global.h` macro over the host's global, set by
  the scenario's `game-status` line (zero-initialised, as in C).

### Order of work

1. ~~**Reduced `project.h`.**~~ Done in slice 2. It needed no reconstruction of
   `project.h`, only whole original headers plus a few fragments (see
   `docs/slices/group-update-timing.md`, investigation 3).
2. ~~**Value accessors.**~~ Group (slice 2), keysite and force (slice 3). The
   CARGO, mobile and sector accessors are compiled whole too (slice 3).
3. ~~**Database.**~~ `gp_dbase.c` is done (slice 2).
4. **Lists.** Group (slice 2), keysite, force, cargo, mobile and sector
   (slice 3). Next: `ss_list.c`, `gd_list.c`, and the aircraft lists with the
   first aircraft slice.
5. **Positions.** Group (slice 2) and keysite (slice 3). The aircraft position
   stays a scenario-supplied entry, because that value is physical. A cargo's
   position is campaign state, set by its creation attributes, and comes from
   `mb_vec3d.c`.

Each step lands with the slice that first needs it, or as its own small PR.
Every step must keep the existing C reference cases and the recorded random
fixtures passing unchanged. If an expectation changes, that is a finding about
the TS port, and must be investigated before anything is re-recorded. When
slice 2 replaced the group shim, slice 1's 41 cases passed unchanged, and
re-recording its 250-scenario fixture produced a byte-identical file. Slice 3
did the same for the keysite, force and heap shims, the 32-bit build and the
TX data flow: all Slice 1 and 2 fixtures re-recorded byte-identically.

### Rules for new slices

- **No new hand-written accessor behaviour in the harness** unless
  `docs/port-manifest.md` records why the real translation unit cannot be
  compiled yet, and what blocks it.
- **A manifest status moves from `source-read` to `C-reference-verified`** only
  when the harness executes the original function or overload for that entry.
- **The shim may only grow for truly environmental inputs** (physical state,
  time, randomness, terrain, transport). These inputs must be driven by the
  same scenario data as the TS port adapters.
- **This table is kept current.** A PR that adds or removes a shim entry
  updates it.

## Freeze policy

A slice is **frozen** when it has complete source mapping, 100% reachable
coverage, source-derived expectations, passing TSTL/Lua execution, and a C
reference comparison where practical. Frozen so far (see the manifest):
- slice 1: `assess_group_supplies` and `get_closest_keysite`;
- slice 2: group update timing (`update_server`, the timer setters, the update
  loop);
- slice 3: the campaign entity lifecycle (heap, attribute-driven creation,
  family destruction), CARGO, and sector membership;
- slice 4: `update_keysite_cargo`, with the `Object3DMetadata` port and the
  game status.

After freezing:

- A behavioural change needs evidence from the EECH C, and the behaviour matrix
  and C reference must agree with it.
- Refactors keep `npm run verify` green, including the mutation controls.
- DCS integration cannot change core behaviour. DCS limitations are solved in
  adapters, and any unavoidable deviation is an explicit adapter-level
  compatibility decision.
