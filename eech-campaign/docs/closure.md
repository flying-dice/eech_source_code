# The source closure and vendoring

## What the kernel is built from

The build generates the closure as `$OUT_DIR/eech/closure.txt`. It is the
union of three sets:
- the original translation units;
- the files the extracts come from;
- the compiler's own dependency scan (`-M`) of every unit.

`eech_campaign::conformance::build::closure_report ()` returns it. On x86-64
it is:

| | Files |
|---|---|
| total | 565 |
| `.c` | 75: 48 compiled whole, 2 patched, the rest the sources of verbatim extracts |
| `.h` | 490: mostly whole original headers included through the generated `project.h` |
| under `aphavoc/source` | 537 |
| under `modules` | 28 |

These are the translation units the kernel compiles:

| Area | Compiled whole | Extracted |
|---|---|---|
| group | `gp_dbase.c`, `gp_float.c`, `gp_int.c`, `gp_list.c`, `gp_ptr.c`, `gp_updt.c`, `gp_vec3d.c` | `group.c` (`assess_group_supplies`, `assess_group_task_locality_factor`), `gp_msgs.c` (link responses) |
| keysite | `keysite.c`, `ks_dbase.c`, `ks_float.c`, `ks_int.c`, `ks_list.c`, `ks_vec3d.c`, **`ks_updt.c`** (patched, P2) | `ks_msgs.c` (link child responses) |
| force | `fc_int.c`, `fc_list.c`, `fc_msgs.c` | `force.c` (`get_local_force_entity`) |
| task | `task.c`, `ts_creat.c`, `ts_dbase.c`, `ts_float.c`, `ts_int.c`, `ts_list.c`, `ts_ptr.c`, **`ts_updt.c`** | `ts_msgs.c` (link parent response) |
| waypoint, pilot | `wp_dbase.c`, `wp_int.c`, `wp_list.c`, `pi_list.c` | — |
| AI | `suitable.c` | `taskgen.c` (`create_task`, `create_supply_task`, `get_task_start_keysite`, `validate_task_generation`), `assign.c` (the assignment decision) |
| mobile, aircraft, cargo | `mb_int.c`, `mb_list.c`, `mb_vec3d.c`, `ac_float.c`, `ac_dbase.c`, `cg_creat.c`, `cg_dstry.c`, `cg_int.c`, `cg_list.c` | `ac_msgs.c` (link responses) |
| sector | `sc_int.c`, `sc_list.c`, `sc_msgs.c`, `sc_seccreat.c` | `sector.c` |
| update | `up_list.c`, `up_msgs.c` | `up_update.c` (the update loop) |
| entity system | `en_heap.c`, `en_attrs.c`, `en_creat.c` (patched, P1), `en_dstry.c` | `en_list.c`, `en_int.c`, `en_float.c`, `en_msgs.c`, `en_vec3d.c`, `en_misc.c`, `en_stats.c`, `en_valid.c`, `en_world.c` |
| engine modules | — | `time.c`, `range.c`, `vector.c`, `matrix.c`, `miscell.c`, `ca_msgs.c`, `soundeff.c` |

**Bold**: native additions beyond the C reference at eech-core-ts `81ed32e`.
They are what `Campaign::step` adds.

## How the closure was found

The dependency graph drove the extraction; nothing was copied speculatively:

1. The starting point is the C reference harness's closure, which eech-core-ts
   slices 1–6a grew one unit at a time. `spec.rs` transcribes
   `c-reference/extract.mjs` entry for entry. It is a Rust port, so the
   native build needs no Node.
2. It was compiled for x86-64 unchanged. That compile is where B1 was
   found: at run time, not at compile time.
3. The `step` path adds `ks_updt.c` and `ts_updt.c`. Linking then named the
   one missing symbol (`create_repair_task`, the once-a-minute repair check),
   and it became a fail-loud stub.
4. Running the corpus found G2 (`group_task_array`) on the second campaign in
   a process.

## Provenance

- An extract is preceded by `#line N "aphavoc/source/…"`.
- A patched file starts with `#line 1` of its original and re-syncs after each
  replacement.
- Compiler diagnostics, `__FILE__`/`__LINE__` in assertion details, and
  debuggers therefore always name the original file.
- A generated file carries a banner naming its generator.
- The host layer's files name their provenance in eech-core-ts in their
  headers.

## Vendoring

The spike builds from the repository's original tree (`../aphavoc`,
`../modules`, or `EECH_SOURCE_ROOT`) and does not copy it. Two reasons:

- **One source of truth.** The C reference, the TSTL port's generated
  constants and the native module all read the same bytes. A copy could drift
  from the source the corpus was recorded against.
- **The closure is still growing.** Every slice adds units. The
  build's report stays exact without any bookkeeping.

A standalone crate (for example, one published for a DCS integration built
outside this repository) needs the files listed in `closure.txt`, copied with
their paths. The extraction specification, `build/`, works unchanged against
such a copy through `EECH_SOURCE_ROOT`. Copying is mechanical, and it should
happen at the point of publishing, not during development.
