# eech-core-ts architecture report

This is the bootstrap report for the frozen EECH campaign-core port (issue #1).
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

The first slice needed these, and all of them are ported in `src/entity/system`:

- identity, type and raw data (`entity.ts`; C: `en_main.c`)
- per-type list roots and links, including **shared links**: a group's
  `LIST_TYPE_BUILDING_GROUP`, `INDEPENDENT_GROUP` and `KEYSITE_GROUP` all resolve
  to one `group_link` (`en_list.ts`; C: `en_list.c`, `en_list/get_prnt.h`)
- typed value dispatch through per-type function tables: int, float, vec3d and ptr
  (`en_values.ts`; C: `fn_get_local_entity_*_value[type][value_type]`)
- client/server dispatch, `set_client_server_entity_float_value [..][comms_model]`,
  with the server path "set local, then transmit" (`comms.ts`, `serverFloatValueSetter`)
- messages: `notify_local_entity` and the `message_responses` table (`en_msgs.ts`)

The kernel will need these later. They are not ported yet:

- creation and destruction with attribute lists (`en_creat.c`, `en_dstry.c`)
- list insertion with link notifications: `insert_local_entity_into_parents_child_list`
  sends `ENTITY_MESSAGE_LINK_CHILD` / `LINK_PARENT`, and group, keysite, task,
  regen and waypoint respond to those
- the update list and sleep timers (`en_updt.c`, `LIST_TYPE_UPDATE`)
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
| `get_delta_time`, `get_system_time`, time acceleration | 40 in 6 files | Environmental. Future `Clock` port. |
| `frand1`, `sfrand1`, `rand16` | 48 in 14 files | Environmental. Future `RandomSource` port, which must reproduce EECH's generator sequence. |
| sound and speech (`play_client_server_*`, speech) | 59 in 7 files | Physical/presentation. Excluded, or an optional notification port. |
| flight dynamics, velocities | 20 in 5 files | Physical simulation, which DCS replaces. Excluded. Only the semantic observation (landed, taken off, arrived) crosses a port. |

The mobile entities themselves (`entity/mobile/**`: aircraft, vehicles, weapons)
are physical simulation. The port keeps only their campaign-visible surface:
membership, type, side and position. See `src/entity/mobile/mobile.ts`.

## 4. Which ports emerge naturally?

The first slice introduced exactly two ports, both derived from C call sites:

| Port | C origin | Test adapter |
|---|---|---|
| `MobilePhysicalState.getMobilePosition` | `ac_vec3d.c` / `vh_vec3d.c` `get_local_vec3d_ptr (VEC3D_TYPE_POSITION)` | `InMemoryMobilePhysicalState` (positions come only from the scenario, never invented) |
| `EntityReplication.transmitEntityFloatValue` | `set_server_float_value` → `transmit_entity_comms_message (ENTITY_COMMS_FLOAT_VALUE, ..)` | `RecordingEntityReplication` (ordered log) |

These are the next ports in order of need. None is created before a ported slice
calls it.

- `Clock`: `get_delta_time`. The update loop decrements `FLOAT_TYPE_SLEEP` and
  `assist_timer` (`gp_updt.c`).
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

**On scheduling.** EECH has no callback scheduler. Time advances through a
per-frame `get_delta_time ()`. Entities on `LIST_TYPE_UPDATE` count down
`FLOAT_TYPE_SLEEP` and act when it expires, and the high-level AI runs from
session and force updates. The deterministic "virtual scheduler" is therefore a
`Clock` adapter driving the ported update list, not a callback queue. It is
introduced with the first slice that contains an update function (see question 10).
Its runaway guard will bound the frame count. Tests must not call the private
update functions directly.

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
supply tasks and pulls in task creation. The slice stops at that message
boundary: the response is declared unported. Tests observe the delivery through
the `record` policy. Production uses `throw`.

## 6. What enables or blocks running the C implementation under a harness?

It works today, for this slice. `c-reference/extract.mjs` copies the original
definitions **verbatim** into `build/c-reference/eech_extracted.c`, with `#line`
directives back to the original files: the enums, the supply struct, the macros,
and the five functions `assess_group_supplies`, `get_closest_keysite`,
`get_local_force_entity`, `get_2d_range`, `get_approx_2d_range` and
`notify_local_entity`. `c-reference/harness.c` supplies the environment: entity
records, lists, the accessor values and a recording message response. It then
calls the original functions. It compiles with `-Wall -Werror`.

What enables this:

- the target functions reach the entity system only through accessor macros, so
  a shim can stand in for it;
- `DEBUG_MODULE` / `DEBUG_SUPPLY` logging is compiled out;
- the extractor fails loudly if a signature or enum disappears.

What limits it or blocks it next:

- **Accessor overloads are shim-provided.** Files such as `gp_int.c` are `static`
  switches inside translation units that include `project.h`, the whole game. The
  harness therefore verifies the slice functions, not `gp_int.c`/`ks_int.c`
  themselves. Those accessors are small and are verified by source reading. To
  execute them too, the next step is to compile the real `xx_int.c`/`xx_float.c`
  files against a reduced `project.h`.
- **Floating point.** The harness requires `FLT_EVAL_METHOD == 0`, i.e. IEEE
  single precision evaluated at the declared type (x86-64 SSE, arm64). The
  historical 32-bit MSVC x87 builds could differ in the last bit where
  intermediates stayed in extended precision. The port follows the IEEE model.
- **Varargs messages.** They work unchanged. The shim installs the response for
  the one message it observes, and any other message aborts the harness.
- **Global state** (`session_entity`, `message_responses`) is defined by the
  harness. Slices that use `get_delta_time`, `rand16` or terrain need those
  environment functions in the shim, driven by the same scenario as the TS
  adapters.

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
  the 41-case matrix, 250 scenarios whose outcomes were recorded from the
  executed C, and the float32 edge cases. That is 313 checks. The runner refuses
  to run on anything but `_VERSION == "Lua 5.1"`, the version DCS embeds.
- `npm run smoke:lua` loads the production bundle `build/lua/eech-core.lua` from
  plain Lua with Lua-table ports, as a DCS host would, and runs a campaign
  operation.
- TSTL diagnostics fail the build (`scripts/lua.mjs build`). ESLint's
  `strict-boolean-expressions` forbids numeric and string truthiness, because
  `0` and `""` are true in Lua.
- C `float` semantics use `toFloat32`, pure arithmetic that exists in both
  runtimes. It is verified against `Math.fround` on 400,000 inputs in JavaScript,
  and on the edge cases under Lua. `Math.fround` and `Math.log2` are lint-banned
  in `src`.
- The Lua runner has already found two real divergences during bootstrap. Both
  are now documented in the test data:
  1. Lua 5.1 merges the literal `-0` into `0` in the constant table.
  2. `{ ...defaults, leader: undefined }` keeps the default in Lua, because a
     table cannot hold `nil`. Optional scenario values are therefore explicit
     variants.
- One mutant only fails under Lua: `in_use` tested by JavaScript truthiness. It
  proves the Lua run adds detection power beyond the JavaScript run.

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

1. **Group update and sleep timers** (`gp_updt.c :: update_server`,
   `en_updt.c`). This introduces the `Clock` port and the deterministic update
   loop, i.e. the virtual scheduler, with a runaway guard. It is small, and it is
   the prerequisite for anything time-driven.
2. **`keysite.c :: update_keysite_cargo`** (the second
   sender of `FORCE_LOW_ON_SUPPLIES`). It reuses this slice's runtime and extends
   keysite accessors.
3. **`fc_msgs.c :: response_to_force_low_on_supplies`**. It closes the supply
   loop by finding a supplier and creating the supply task. It requires task
   creation (`en_creat.c` for tasks, `entity_is_object_of_task`) and
   `get_game_status`.
4. **The `mb_msgs.c` landing handlers that call `assess_group_supplies`**, which
   introduce the `LandingObservation` port. The DCS adapter reports "member
   landed at keysite", and the campaign does the accounting.
5. **Harness depth.** Compile the real `gp_int.c`/`ks_int.c`/`ks_float.c` against
   a reduced `project.h`, so the accessors are C-verified as well.

## Freeze policy

A slice is **frozen** when it has complete source mapping, 100% reachable
coverage, source-derived expectations, passing TSTL/Lua execution, and a C
reference comparison where practical. `assess_group_supplies` and
`get_closest_keysite` meet all of these (see the manifest). After freezing:

- A behavioural change needs evidence from the EECH C, and the behaviour matrix
  and C reference must agree with it.
- Refactors keep `npm run verify` green, including the mutation controls.
- DCS integration cannot change core behaviour. DCS limitations are solved in
  adapters, and any unavoidable deviation is an explicit adapter-level
  compatibility decision.
