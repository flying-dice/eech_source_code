# C reference harness

This directory runs the **original** EECH functions against the same scenarios
as the TypeScript port.

- `extract.mjs` copies the enums, structs, macros and functions listed in
  `EXTRACTS` verbatim from `../aphavoc` and `../modules` into
  `build/c-reference/eech_extracted.c`. Each block carries a `#line` directive to
  its origin. The extractor fails if an original definition cannot be found.
- `eech_shim_prelude.h` and `eech_shim_types.h` supply the environment those
  functions expect: `ASSERT` with debug-build meaning, entity records, lists
  (including the shared `group_link`), and the accessor values of the scenario's
  entities. The shim contains no campaign algorithm.
- `harness.c` reads a scenario on stdin (written by
  `test/scenarios/campaign-scenario.ts :: serialiseScenario`), runs one operation
  and prints the outcome. Floats are printed as bit patterns.
- `build.mjs` extracts and compiles with `-Wall -Werror`, and fails when no C
  compiler is available.

The comparison covers the extracted functions. The accessor overloads
(`gp_int.c`, `ks_int.c`, ...) are shim-provided and are verified by source
reading. See `docs/architecture.md`, question 6.

The shim is meant to shrink over time. `docs/architecture.md`, "Shrinking the C
reference shim", lists the current shim entries, the order in which real EECH
translation units replace them, and the rules that stop the shim growing.
