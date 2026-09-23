# eech-core-ts

This is a mechanically faithful port of the EECH dynamic campaign core from C to
TypeScript, compiled to **Lua 5.1** with
[TypeScriptToLua](https://typescripttolua.github.io/).

The original C in `../aphavoc` and `../modules` is the behavioural authority. The
campaign core runs without DCS World. The environment (physical positions,
replication, the frame clock, and later randomness, terrain and 3D object
dimensions) reaches it only through narrow ports in `src/ports`.

- `docs/architecture.md`: the bootstrap report (kernel boundary, ports, harness,
  coverage, Lua semantics, next slices).
- `docs/port-manifest.md`: the authoritative C→TS mapping and status.
- `docs/slices/assess-group-supplies.md`: slice 1, frozen, with its dependency
  trace and behaviour matrix.
- `docs/slices/group-update-timing.md`: slice 2, frozen, with its investigation,
  boundary and behaviour matrix.
- `docs/slices/entity-lifecycle-cargo.md`: slice 3, frozen, with its
  investigation, findings, boundary and behaviour matrix.

## Requirements

- Node.js ≥ 20
- a **Lua 5.1** interpreter (`lua5.1`, or set `LUA=/path/to/lua`). For example,
  `apt-get install lua5.1`.
- a C compiler with 32-bit x86 support (`cc`, or set `CC`; for example
  `apt-get install gcc gcc-multilib`) for the C reference gate

## Commands

```sh
npm ci
npm run verify        # every gate below, in order
```

| Gate | Command | What it proves |
|---|---|---|
| C-derived sources are current | `npm run check:c` | Enum ordinals, database columns and numeric constants in `src/generated` match the EECH C |
| Types | `npm run typecheck` | Node-side and Lua-side projects typecheck cleanly |
| Lint | `npm run lint` | No numeric/string truthiness (0 is true in Lua); no DCS, Node or host globals in `src` |
| Tests + coverage | `npm run coverage` | 100% statements, branches, functions and lines over `src` |
| Lua build + smoke | `npm run smoke:lua` | `build/lua/eech-core.lua` builds without TSTL diagnostics and runs in Lua 5.1 as a host would use it |
| Lua conformance | `npm run test:lua` | The shared behaviour matrix and C-recorded scenarios pass under Lua 5.1 |
| C reference | `npm run test:cref` | The original EECH C, extracted verbatim and executed, agrees with the expectations and with the TS port |
| Negative controls | `npm run mutation` | Meaningful behavioural regressions are detected |

CI runs the same gates step by step in `.github/workflows/eech-core-ts.yml` for pull requests and pushes to `master` that touch `eech-core-ts/`, `aphavoc/` or `modules/`.

`npm run cref:record` re-records the fixtures in `test/scenarios/generated/`
from the executed C. `npm run gen:c` regenerates `src/generated` from the C sources.

## Layout

```
src/
  core/            ASSERT, C float and integer semantics (float32: RTZ arithmetic, cint), maths, time (get_delta_time), configuration
  entity/system/   entity runtime: heap, lists (with shared links), value function tables, messages, comms model,
                   creation attributes, creation / destruction dispatch, the world map
  entity/special/  session, force, keysite, group, guide, update, sector, effect: the ported overloads, campaign functions and the update loop
  entity/mobile/   campaign-visible surface of aircraft and vehicles (position comes from a port), and cargo
  generated/       enums, database columns and constants generated from the EECH C (never hand-edited)
  ports/           what the campaign needs from the environment
test/
  adapters/        deterministic port implementations (no campaign policy)
  scenarios/       shared scenario model and behaviour matrix (TSTL-compatible)
  unit/            vitest suites (JavaScript semantics, coverage)
  lua/             Lua 5.1 conformance runner and bundle smoke test
  c-reference/     C-vs-TS differential tests
c-reference/       extractor, harness environment and harness that execute the original C
```

## Porting rules (short form)

- Preserve EECH behaviour. Every ported function names its C provenance.
  Expectations come from the C, never from the TypeScript or from `ee-dcs`.
- Model C `float` results with the round-toward-zero helpers of
  `src/core/float32.ts` (`toFloat32RTZ`, `f32Add`, ...), because EECH runs its
  FPU rounding toward zero. `toFloat32` (to nearest) is only for compile-time
  constants. See `docs/fidelity/fpu-semantics.md`.
- Do not use truthiness on numbers or strings. Compare explicitly.
- Do not use `undefined` properties as data in code that must run under Lua. A
  table cannot hold `nil`.
- Function-table entries start unported and throw. Install only overloads that
  have been ported.
- No DCS concept enters `src`. DCS belongs in adapters.
