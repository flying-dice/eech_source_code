# Changes to the original code

The kernel runs the original EECH text, with the exceptions listed here.
- The patches are declared in `crates/eech-sys/build/patches.rs`.
- The compile definitions are declared in `crates/eech-sys/build/spec.rs` and `build/main.rs`.

A patch replaces whole lines, and those lines must occur exactly as many times
as declared, otherwise the build fails. That keeps a patch from silently
applying to changed code. The generated file starts with `#line 1` of the
original and re-syncs the line numbers after each replacement. Compiler
messages, debuggers and `__FILE__`/`__LINE__` in assertion diagnostics
therefore all name the original file and line.

## Patches

### P1: en_creat.c, the stack attribute list (64-bit blocker B1)

```c
	char
		*pargs_buffer;
+	EECH_STACK_ATTRIBUTES_STORAGE (eech_stack_attributes);
...
-		pargs_buffer = (char *) pargs;
+		pargs_buffer = EECH_STACK_ATTRIBUTES (eech_stack_attributes, pargs);
```

This applies to both `create_local_entity` and `create_client_server_entity`.
- **The marshaller** (`csrc/eech_attrs.c`) reads each `ENTITY_ATTR_*` item
  with `va_arg`, at its promoted type. It writes the item in the layout
  `get_list_item` reads: `sizeof (TYPE)` per item, doubles for floats, ints for
  chars.
- **The storage** is 1 KiB in the creating function's frame. That matches the
  lifetime of the i386 argument area it replaces, and it is safe for nested
  creations.
- **Evidence** (`docs/64-bit.md`, B1):
  - the probe shows the original line reads the va_list descriptor on x86-64;
  - on i686, a build with the original line (`EECH_ORIGINAL_STACK_ATTRIBUTES=1`)
    and the patched build pass the same corpus;
  - the patch is behaviour-neutral.

### P2: ks_updt.c, the shared repair-check timer (global state G1)

```c
-		static float task_timer = 0.0;
+		extern float eech_ks_updt_task_timer;
+#define task_timer eech_ks_updt_task_timer
```

The timer becomes a named global that the kernel resets with the campaign. The
code that uses it is unchanged.

## Controlled compile definitions

| Unit | Definition | Why |
|---|---|---|
| `force/fc_msgs.c` | `create_supply_task=eech_observe_create_supply_task` | Its call of `create_supply_task` passes through the host layer. The layer reports `SupplyMissionRequested` (and the C reference's `create-supply-task` trace line), then calls the original. This replaces the C reference's `ld --wrap`, which does not exist outside GNU ld. |
| `en_main/en_heap.c` | `get_free_entity=eech_original_get_free_entity` | Every allocation of the original code passes through `csrc/eech_kernel.c :: get_free_entity`, which counts the index's generation for `EntityId`. `en_heap.c` does not call the function itself. |
| all original units, GCC-family Windows only | `-UWIN32` | T1: `highlevl.h`'s WIN32 release `ai_log` needs MSVC's preprocessor. EECH's own non-WIN32 branch is used instead. |
| `eech_extracted_taskgen.c` | `-ftrivial-auto-var-init=zero` | F1 (eech-core-ts `docs/slices/supply-task-construction.md`): `create_supply_task` reads the uninitialised `prepare.y` / `finish.y`. The C reference and the TSTL port share a compatibility decision, 0.0. The native module inherits it, as a decision, not a claim about what EECH produced. MSVC has no such flag: an MSVC build must make the same decision another way. |

## Extraction instead of whole files

Files that reach far beyond the slice are not compiled whole: the same
functions are extracted, verbatim, as the C reference extracts them (`spec.rs`,
forked from `extract.mjs`). These files are `taskgen.c`, `assign.c`,
`en_misc.c`, `group.c`, `gp_msgs.c`, `up_update.c`, `ts_msgs.c`, `en_vec3d.c`, `ac_msgs.c`, `ks_msgs.c`,
`sector.c`, `en_list.c`, `en_int.c`, `en_float.c`, `en_msgs.c`, `time.c`,
`range.c`, `vector.c`, `en_world.c`, `ca_msgs.c`, `force.c`, `soundeff.c`,
`en_stats.c`, `en_valid.c`, `miscell.c` and `matrix.c`.

An extract is the original text: nothing between its `#line` directive and its
closing brace is changed.

## The host layer is not EECH

`csrc/` is new code, and holds no campaign policy:
- dispatch tables;
- the environment (transport, allocator, 3D object database, positions);
- fail-loud stubs;
- the lifecycle;
- the legacy replay.

Its provenance is eech-core-ts's `c-reference/harness.c` and
`eech_harness_env.h`, as stated at the top of each file.
