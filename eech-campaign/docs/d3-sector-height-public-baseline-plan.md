# D3 sector-height extraction, first substage: the public-funnel baseline observation (source plan)

Task EEDCS-49 (D3), 7 November 2026. Source before: `d23882dc28225744a953f3e9335eca4bd7ae05fa`. This is the FIRST substage of the REQUIRED host-owned sector high-water extraction
([contract](d3-sector-height-contract.md), section 6), not a standalone preparation and not a production move. It prepares ONE test-only observer and ONE public Lua scenario so that the
exact missing facts of `sector-height-host-move-open-facts` are observed on the real path BEFORE any production behaviour moves. Nothing here changes the Core writer, a clock, the random
stream, a policy, a shipping build or any accepted baseline; the observer is read-only and exists only in a feature build.

## The missing facts, and what observes each

The numeric controls (a8ef/12a, 179 expectations on x86-64 and i686; clarification 3fac/7559; reviewed `033defb4`) drive hand-made entities through the message table. They did not run the
public funnel. Still unobserved, and observed by this substage:

| # | missing fact | observed by (hook, staged copy only) | where |
|---|---|---|---|
| M1 | the ACTUAL order of sector-list INSERT (roster), LINK_CHILD (the raise) and the fixed-state statement to the host, for each fixed family | `eech_sho_sector_insert`, `eech_sho_link_begin/end`, `eech_sho_statement_fixed/city`, one global sequence number | `en_list.c` after `campaign_place_insert`; `sc_msgs.c` around the tallest block; `mb_hostplacemap.h` `place_fixed_state` |
| M2 | the words each raise used (shape, position y, box top) against the shape and position the host is told in the fixed-state statement, and the sector word before and after | the same link and statement records (raw bit patterns, never arithmetic) | same |
| M3 | whether a REAL city arrives at the sector list with a valid shape (its state carries no shape) | the link record carries entity type, the fixed flag, the shape index and its validity; the city statement is recorded beside it | `sc_msgs.c`, `mb_hostplacemap.h` |
| M4 | fact 12 and the FIRST-sector reads of the scan | `eech_sho_scan_first` (per sector: reads, first and last word, changes, first/last sequence) and `eech_sho_candidate_fact` (offers, non-zero words) | `mb_hostscan.h`, `mb_hostoffer.h` |
| M5 | whether the guarded fixed-wing clearance consumer is reached, with which words | `eech_sho_clearance_leader`, `eech_sho_clearance_follow` (the two runtime guards), `eech_sho_clearance_tallest`, `eech_sho_clearance_final` (the sector word, the terrain elevation and `min_height` after each statement) | `fw_move.c` |
| M6 | whether a save or restore of the word is reached by the public scenario | `eech_sho_pack`, `eech_sho_unpack` (counts and the raw words) | `sc_pack.c` |

An unreached case is reported with its counter at zero and stays named UNREACHED. A configured query is never credited as a natural engagement, and no actor, ray, seed, frame or duration is
tuned: the scenario is the ordinary campaign driver's own boot and frame loop with the established defaults, observed.

## All immediate consumers, inputs and state effects of every hook

Each hook ADDS one call and changes no original statement, argument, order or value. Floats cross as `const float *` (never by value, so no x87 load can quieten a signalling NaN) and are copied
to `uint32_t`; the hooks do no floating-point arithmetic, allocate nothing, draw no random number, read no clock and write no entity or campaign byte. The only reads are the getters the
original statement next to the hook reads itself (`INT_TYPE_IDENTIFY_FIXED`, `INT_TYPE_OBJECT_3D_SHAPE`, `VEC3D_TYPE_POSITION`, `get_object_3d_bounding_box`, the entity type, the sector
record's coordinates). State: bounded private tables in `csrc/eech_sho.c` (events, clearance records, one cell per sector); a table that fills latches an overflow counter and the report says
`complete=0`; nothing is silently truncated. Single-threaded bookkeeping (the engine runs behind one entry lock). One boot per process (the tables are not reset).

| hook | inputs | state effect |
|---|---|---|
| `eech_sho_sector_insert (en, parent)` | the entity, its sector | counts every insert; records an event for a fixed-flagged entity or a city |
| `eech_sho_link_begin (sector, sender, word)` | the sector entity, the sender, a pointer to the held word | remembers the word's bits |
| `eech_sho_link_end (sector, sender, word)` | the same | counts; for a fixed sender or a city records type, fixed flag, shape and validity, position and box-top bits when the original read them, before and after bits |
| `eech_sho_statement_fixed (en, type, key, state)` / `_city (en, key, city)` | the statement the host is about to receive | records the key and the raw state words |
| `eech_sho_scan_first (x, z, word)` | the sector the host named and its word | per-sector aggregate |
| `eech_sho_candidate_fact (word)` | fact 12 of one candidate question | two counters |
| `eech_sho_clearance_*` | the guard reached, the sector entity, `terrain_elevation`, `min_height` | two counters; up to 4096 records |
| `eech_sho_pack / unpack (word, x, z)` | the raw word and the sector coordinates | counters; up to 64 events in the event table |

The report is read through three read-only entries (`eech_engine_sho_summary`, `_event`, `_clearance`, `_scan`) in the same pattern as the route and target-point fixtures, through the
public Lua module methods of the same names (feature builds only).

## How the observer is attached

Staged build patches (`build/patches.rs`, `SECTOR_HEIGHT_OBSERVER_PATCHES`), applied by `build/main.rs` only when feature `sector-height-observer` is on, exactly as the route and target-point
fixtures do: exact anchors with a strict occurrence count, on the staged copy only. **No path under `aphavoc/` or `modules/` is edited**; with the feature off the staged files are the shipping
ones. The hook header is force-included only in the feature build. Existing guards (`sector_height_guard` and the entity guards) keep pinning the in-tree sources.

## The public scenario

`lua/sector-height-baseline.lua`: the ordinary campaign driver's boot and frame loop (same `dc.boot` arguments as `lua/campaign.lua`, no recording and no metrics), then the observer report is
written to an output file in a strict `key=value` line grammar, and the driver refuses an observation that is not whole (`complete=0`). Natural combat and line-of-sight checks of the ordinary
war reach `get_line_of_sight_collision_entity`, which is where fact 12 and the FIRST reads occur; whether and how often is the observation, not a precondition. For each profile Root runs it once
on a DLL built with the observer (`tools/build-windows-sector-height-observer.sh`, fresh `EECH_WIN_TARGET` volume) on a fresh verified root copy with the established ordinary defaults, through
`tools/sector-height-baseline-windows.ps1`. The wrapper refuses to overwrite an observation, runs ONE driver process, and accepts only a whole verdict. A duration is never extended to make a
case reach. The observer DLL is not the shipping DLL: the staged `fw_move.c` hooks take the address of two locals (`terrain_elevation`, `min_height`), which can change that build's code generation
for them, so the observer is NOT assumed result-neutral by construction. Root's control is the ordinary short check run with the observer DLL against the three accepted short artifacts; a
difference is a finding about the observer, not a baseline change.

## Checks prepared (focused, Linux)

A real-C control of the observer itself (feature combination `sector-height-check` + `sector-height-observer`): the existing control's real `response_to_link_child`, float getter and pack/unpack
run with the staged hooks, and the observer's records are compared with the control's own known words (before/after, raised or not, the raw saved and restored words). It proves the
RECORDER, not the funnel: the funnel observation is the native run. Plus `cargo fmt`, strict clippy for the feature set, and the existing sector-height guards unchanged.

## Not claimed

No production move, no host-owned scalar, no native or Windows result, no measurement of the engine C at -O2 (`EECH_C_OPT` does not reach `eech-engine-sys`; the engine default is -O1), no
signalling-NaN or x87 control-word claim, no exhaustive runtime census. The observer's records are evidence about one scenario, not the supported domain.
