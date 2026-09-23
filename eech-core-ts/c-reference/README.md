# C reference harness

This directory runs the **original** EECH C against the same scenarios as the
TypeScript port.

- `extract.mjs` generates these files into `build/c-reference/`:
  - `project.h`: `eech_harness_env.h`, verbatim fragments and **whole original
    headers**, in the order listed in `PROJECT_H`;
  - `eech_extracted*.c`: verbatim functions from files too large to compile
    whole (`EXTRACTED_UNITS`). There is one generated file per group of
    extracts that share a C scope, so that two original `static` functions of
    the same name (for example `response_to_link_parent` in `gp_msgs.c` and
    `ac_msgs.c`) never meet.

  Each fragment carries a `#line` directive to its origin. Extraction fails if
  an original definition cannot be found.
- `REAL_TRANSLATION_UNITS` (in `extract.mjs`) lists the original files compiled
  **unchanged** against that `project.h`, for example `gp_int.c`, `gp_float.c`
  and `gp_updt.c`.
- `eech_harness_env.h` is the environment: the C library, the Windows SDK
  `min`/`max`, `ASSERT` with debug-build meaning, debug output declarations, and
  opaque engine types. `eech_harness_decls.h` declares the harness's own
  globals and extracted functions.
- `harness.c` defines the value and list dispatch tables and fills them through
  the original defaults and `overload_*_functions ()`. It adds hand-written
  rows only for entity types whose files are not compiled yet (session, guide,
  helicopter). It supplies the environment: frame delta, comms, transport,
  mobile positions, memory and `convert_float_to_int`. Functions that the
  compiled files reference but the adopted paths never reach are fail-loud
  stubs.
- **Scenarios.** `harness.c` reads a scenario on stdin and prints the outcome,
  with floats as bit patterns. `stdout` is unbuffered. Scenario entities come
  from the original entity heap (`en_heap.c`), in scenario order, after the
  session and update entities.
  - Lifecycle scenarios (`heap`, `map`, `create`, `destroy` lines) run the
    original construction path. `create` lays its attributes out as i386
    argument-stack words and calls `create_client_server_entity`, which reads
    them through its TX path. The scenario ends with the entity graph.
  - `ASSERT` and `debug_fatal` are EECH outcomes (`result assert ...`,
    `result fatal <format>`) and end the scenario.
- **Faults.** A `SIGSEGV` inside the NULL page is EECH's unguarded NULL
  dereference. The handler reports `result null-dereference` and the final
  state, then ends the process; it never resumes it. Any other fault kills the
  process by signal, and the test driver treats that as a failed run, never as
  an outcome (`test/c-reference/harness-faults.cref.test.ts`).
- **Signal safety is checked on the binary.** The handler's call graph may reach
  no library call except `write` and `_exit`: fixed strings have compile-time
  lengths, float bits are hex-encoded by hand, and `harness.c` is built with
  `-fno-stack-protector`. `test/c-reference/harness-signal-safety.cref.test.ts`
  walks the call graph in the compiled object with `objdump` and fails on
  anything else, including calls the compiler inserts.
- **Platform.** The harness is built for 32-bit x86 (`-m32`), EECH's platform,
  with SSE float arithmetic (`-msse2 -mfpmath=sse`, `FLT_EVAL_METHOD == 0`).
  The original code depends on the 32-bit calling convention: `en_creat.c`
  converts a `va_list` into the `char *` attribute buffer
  (`pargs_buffer = (char *) pargs`), which only works where `va_list` points
  into the argument stack. On x86-64 that pointer is meaningless. A compiler
  with 32-bit support is required (`gcc-multilib` on Debian and Ubuntu).
- `build.mjs` compiles everything. Our own files build with `-Werror`. Original
  files keep their historical warnings, but mismatches with the environment are
  errors (implicit declarations, pointer and int conversions). Before Slice 3
  the original-code flags also carried `-w`, which silently disables those
  promoted errors. It was removed; the Slice 1 and 2 build was clean without
  it. The build fails when no C compiler is available.

The remaining hand-written behaviour is listed, together with the plan to
retire it, in `docs/architecture.md`, "Shrinking the C reference shim".
