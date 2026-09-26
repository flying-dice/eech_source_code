# The boundaries

The architecture has three boundaries, and they must not be confused:

```
PUBLIC RUST API            eech-campaign     semantic campaign concepts
    │                                        Campaign, CampaignConfig, World, EntityId, Position,
    │                                        CampaignEvent, Replication, CampaignSnapshot, CampaignError
    ▼
PRIVATE RUST/C FFI         eech-sys          implementation mechanics
    │                                        eech_kernel.h: (index, generation) references, ints,
    │                                        floats, strings; host callback table; status codes
    ▼
LEGACY EECH INTERNALS      original C        entity *, list_root, list_link, dispatch tables,
                                             messages, va_lists, raw structs
```

No EECH type crosses the FFI. `eech_kernel.h` is the whole contract, and
`crates/eech-sys/src/ffi.rs` mirrors it. Nothing outside `eech-sys` uses it:
`eech-campaign` has `#![forbid(unsafe_code)]`, and the harness only depends on
`eech-campaign`.

## World: derived from the C call sites the slice reaches

The table lists every environmental input and output the running slice has.
The kernel fails loudly for any environmental function it does not provide, so
this list is complete: everything else is a `NOT_REACHED` stub
(`csrc/eech_env.c`).

| C call site | What EECH needs | Public boundary | eech-core-ts port it corresponds to |
|---|---|---|---|
| `ac_vec3d.c` `VEC3D_TYPE_POSITION` of a helicopter or fixed-wing member, read by `gp_vec3d.c` (a group's position is its leader's), `assess_group_supplies`, `assess_group_task_locality_factor` (the mission ETA) | the flight model's current position | `World::position (EntityId) -> Option<Position>` | `MobilePhysicalState` |
| `3dobjvis.c :: get_object_3d_bounding_box`, read by `keysite.c :: update_keysite_cargo` (crate rows) and the sector link response | the 3D object database | `World::object_bounds (ObjectModel) -> Option<Bounds>` | `Object3DMetadata` |
| `time.c :: set_delta_time` (host frame measurement), `get_delta_time` | the frame delta | `step (&mut world, delta)` | `Clock` |
| `en_comms.c :: transmit_entity_comms_message` (FLOAT_VALUE, CREATE, DESTROY, SET_TASK_POINTERS, SWITCH_PARENT, DESTROY_LOCAL_FAMILY) | a multiplayer server's authoritative changes | `CampaignEvent::Replicated (Replication)` (server sessions only) | `EntityReplication` |
| `ca_msgs.c :: notify_campaign_screen` (MISSION_CREATED) | the campaign screen | `CampaignEvent::MissionCreated` | `CampaignEvents` |
| `fc_msgs.c`: the force's `FORCE_LOW_ON_SUPPLIES` response | (a semantic notification the campaign produces) | `CampaignEvent::LowOnSupplies`, `SupplyMissionRequested` | test seams in eech-core-ts |
| `global.c` `game_status`, `gametype.c` `game_type`, `comms.c` comms model, EECH.INI `entity_update_frame_rate` | the host's session | `CampaignConfig` (`session`, `update_rate`); the campaign host is always EECH's server | core state with setters |

### Why `World` has no `execute (WorldCommand)` yet

No call site in the slice commands the physical world. The replication messages
are not commands: they describe changes for clients, and a single-player
session suppresses them (`en_comms.c` traps single player). EECH's physical
effects begin inside `assign_primary_task_to_group`, where a group gets its
route, guide and members' tasks. That is eech-core-ts 6b/6c, and the slice's
boundary today. The first `WorldCommand` will be derived there, from those call
sites. The architecture has room for it: `step` already takes `&mut W`.

### Why `World` is queried, not fed

EECH asks for positions and object bounds *during* its update, when its logic
needs them: the ETA check runs in the middle of the assignment decision, and
the bounds are read in the middle of laying out crates. A `World` fed before
`step` would have to guess which entities EECH will ask about. That is
possible for positions (all aircraft), but it is wasteful and it hides the
dependency. The synchronous query is the faithful shape. The harness's
`HeadlessWorld` records every question and whether it could answer, so a
scenario shows exactly what its campaign needed from the world. The supply
chain asks for one position and 22 object bounds over 180 seconds.

Queries are `&self`. A `World` cannot mutate itself or call back into the
campaign during a step: the campaign is mutably borrowed.

## Errors: fail loudly, with names

| Kernel status | Public error | Example |
|---|---|---|
| `ASSERT` | `Assertion { expression, detail }` | `ASSERT (v2)` in `get_approx_2d_range` |
| `debug_fatal` | `Fatal { message }` | an invalid entity attribute |
| unported dispatch row / stub | `Unported { dependency, detail }` | `message_responses [ENTITY_TYPE_TASK] [ENTITY_MESSAGE_TASK_TERMINATED]`: an unassigned supply mission expires after 20 minutes (`scenarios/supply-chain-no-registered-group.expected.json`) |
| the slice boundary | `Boundary { name, entities, detail }` | `assign_primary_task_to_group` with the group and the mission |
| a `World` answered `None` | `World (message)` | `World::position knows no position for #23.1` |
| a panic in `World` | the panic, resumed in Rust | — |

## The semantic model

- `CampaignConfig` is a campaign as a saved game holds it: map, session, sides,
  keysites (kind, side, position, supplies, in use, usable, landings), groups
  (kind, side, base, supplies, members, registration).
- Kinds are EECH database names without the implementation prefix
  (`KeysiteKind::AIRBASE`, `GroupKind::named ("MEDIUM_LIFT_TRANSPORT_HELICOPTER")`,
  `AircraftKind::named ("UH60_BLACK_HAWK")`). The build generates the name
  tables from the original enums, and names are checked before the kernel is
  touched.
- The restore follows the C reference's method: raw state and list membership
  as `gp_pack.c`/`ks_pack.c` hold them. The difference is that keysites and
  groups join the update list, so that `step` runs them.
- `CampaignSnapshot` is read from raw state without running EECH code. It
  holds forces (supply missions created), keysites (supplies, crates, waiting
  missions), groups (supplies, members) and missions (kind, status, priority,
  expiry, objective, keysite, route length).
