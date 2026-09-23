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
  prints the outcome, with floats as bit patterns. NULL dereferences in the
  original code are reported as `result null-dereference` (`SIGSEGV`).
- `build.mjs` compiles everything. Our own files build with `-Werror`. Original
  files keep their historical warnings, but mismatches with the environment are
  errors (implicit declarations, pointer and int conversions). The build fails
  when no C compiler is available.

The remaining hand-written behaviour is listed, together with the plan to
retire it, in `docs/architecture.md`, "Shrinking the C reference shim".
