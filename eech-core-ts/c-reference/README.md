# C reference harness

This directory runs the **original** EECH C against the same scenarios as the
TypeScript port.

- `extract.mjs` generates two files into `build/c-reference/`:
  - `project.h`: `eech_harness_env.h`, verbatim fragments and **whole original
    headers**, in the order listed in `PROJECT_H`;
  - `eech_extracted.c`: verbatim functions from files too large to compile whole
    (`EXTRACTED_C`).

  Each fragment carries a `#line` directive to its origin. Extraction fails if
  an original definition cannot be found.
- `REAL_TRANSLATION_UNITS` (in `extract.mjs`) lists the original files compiled
  **unchanged** against that `project.h`, for example `gp_int.c`, `gp_float.c`
  and `gp_updt.c`.
- `eech_harness_env.h` is the environment: the C library, the Windows SDK
  `min`/`max`, `ASSERT` with debug-build meaning, debug output declarations, and
  opaque engine types. `eech_harness_decls.h` declares the harness's own
  globals and extracted functions.
- `harness.c` defines the dispatch tables and fills them through the original
  `overload_*_functions ()`. It adds hand-written rows only for entity types
  whose files are not compiled yet, and supplies the environment: frame delta,
  comms, transport and mobile positions. It then reads a scenario on stdin and
  prints the outcome, with floats as bit patterns. `stdout` is unbuffered.
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
  errors (implicit declarations, pointer and int conversions). The build fails
  when no C compiler is available.

The remaining hand-written behaviour is listed, together with the plan to
retire it, in `docs/architecture.md`, "Shrinking the C reference shim".
