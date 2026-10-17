# Global state

EECH keeps the campaign in process globals: the entity heap, the dispatch
tables, the world map, the frame clock, host settings, and function-local
statics. The module does not hide this behind an API that implies independent
instances.

## What the module supports

| Mode | Supported | How |
|---|---|---|
| One campaign per process | yes | `Campaign::new` takes a process-wide token. A second `new` while one exists fails with `CampaignError::AlreadyRunning`, and so does a legacy replay. |
| Several campaigns one after another | yes | Opening the kernel resets every piece of state listed below. Proven by `a_sequential_campaign_starts_from_a_clean_state` (three runs, identical), and by the corpus: 8,401 scenarios replayed in one process, each identical to a fresh 32-bit C reference process. |
| Several campaigns at once | **no** | The globals make it impossible. The type system and the token enforce it. |
| Moving a campaign between threads | yes | `Campaign: Send`, not `Sync`. The floating-point mode is per thread, so every entry installs it itself. |

An id from one campaign is never valid in another. `EntityId` carries its
campaign instance, and the kernel's (index, generation) pair detects stale ids
within a campaign.

## The inventory

`crates/eech-sys/global-state.txt` classifies **every writable symbol** of the
compiled kernel: data and bss, function-local statics included. There are 201
at the time of writing. `tests/findings.rs :: every_writable_global_is_classified`
reads the symbols with `nm` from the actual objects and fails if a symbol is
unclassified, or if a classified one has disappeared. A slice that compiles a
new original unit therefore cannot add campaign state silently.

| Class | Meaning | Reset |
|---|---|---|
| `campaign` | original campaign state | by `eech_k_open` (`reset_all`, `eech_reset_environment`, `eech_reset_extracted_statics`, EECH's own `deinitialise_group_task_array`, load-time values) |
| `host-config` | the host's session settings (comms model, game type/status, EECH.INI) | reset, then set from `CampaignConfig` |
| `tables` | dispatch tables | rebuilt by the original `overload_*` functions on every open |
| `constant` | initialised data never written: the original databases, the name tables | verified: `database_digest` is compared before and after the whole corpus |
| `scratch` | written before every read, within one call (e.g. `assign.c`'s idle-count array) | not needed |
| `stale-unread` | survives between campaigns but is never read before it is rewritten | not needed (evidence per symbol) |
| `host-layer` | the native layer's own bookkeeping | reset |
| `replay` | the legacy replay's state | reset by `eech_legacy_reset` |

## Findings

| Id | Finding | Handling |
|---|---|---|
| G1 | `ks_updt.c :: update_server` keeps `static float task_timer` inside the function. A single timer is shared by **all** keysites: it paces the once-a-minute repair check across the whole campaign. It is not saved in save games, and nothing resets it. A second campaign would inherit the first one's timer. | Patch P2 (`docs/patches.md`) turns it into the named global `eech_ks_updt_task_timer`, reset with the campaign. The arithmetic is unchanged. |
| G2 | `suitable.c :: group_task_array` (static) is built by `initialise_group_task_array`, which `ASSERT`s it is still NULL. A second initialisation in the same process aborts. The first corpus run in one process found this. | Reset calls EECH's own `deinitialise_group_task_array` before freeing the arena. |
| G3 | `task.c :: completed_task_expire_time` (static, with a setter) and `ks_int.c :: keysite_icon_timer_flag` are initialised globals. Nothing in the slice writes them. | They are captured at first open and restored on every reset, so the module restates no original initialiser. |
| G4 | `sc_seccreat.c :: create_local_sector_entities` keeps the previous map's bounds in six function-local statics. | `stale-unread`: they are only read while `entity_sector_map` is non-NULL, and a new campaign starts with it NULL. |
| G5 | The original databases (`aircraft_database`, `group_database`, `keysite_database`, `task_database`, `waypoint_database`, ...) are writable C data. | `constant`: the digest test shows the kernel never writes them. In the full game, `wutcfg.c` overrides them from files. That is not in the kernel, and it becomes campaign configuration if it is ported. |
| G6 | Everything EECH allocates (`malloc_fast_mem`, `malloc_heap_mem`) belongs to the campaign. | An arena, freed whole on reset, whatever state an aborted call left behind. |

## After a failure

An `ASSERT`, `debug_fatal`, an unported dependency, the slice boundary, or a
world that cannot answer all end the C call through `longjmp` to the entry's
abort point. The C state is then whatever the aborted code left.
- The campaign is **poisoned**: further steps fail with
  `CampaignError::Poisoned`.
- The snapshot stays readable, for diagnosis.
- Dropping the campaign frees everything.
- The next campaign starts from a full reset.

A panic in a `World` implementation is caught in the trampoline, aborts the C
call, and is **resumed in Rust** after the C has returned. It keeps its
payload, poisons the campaign, and never unwinds through C frames
(`a_panicking_world_unwinds_in_rust_and_poisons_the_campaign`).

## Towards explicit state

The inventory is also the to-do list for moving globals into an explicit
campaign object, if multiple concurrent campaigns are ever wanted. The
`campaign` and `host-config` classes are the work. Today that is 43 symbols,
most of them in `en_heap.c`, `up_update.c` and the frame clock.
