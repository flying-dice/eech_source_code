# Floating point in the native module

Read `eech-core-ts/docs/fidelity/fpu-semantics.md` first. It is the evidence
for EECH's numerical contract:
- the campaign thread runs **rounding toward zero**
  (`startup.c :: set_fpu_rounding_mode_zero`, re-asserted after every library
  initialisation);
- the C reference evaluates C float arithmetic **at declared type**, in SSE;
- **x87 intermediate precision** is open, and gated on a runtime trace of the
  shipped binary.

The native module does not reopen that decision. It reproduces the C
reference's contract and measures whether the native targets change anything.

## The environment the module installs

- **Rounding.** Every kernel entry (`eech_enter`, `csrc/eech_kernel.c`) saves
  the caller's floating-point environment (`fegetenv`), sets round toward zero
  (`fesetround (FE_TOWARDZERO)`), and restores the caller's environment on
  every exit, the aborted ones included. On x86-64 glibc and MinGW,
  `fesetround` sets both the SSE MXCSR and the x87 control word.
- **Drift.** An entry whose rounding mode changed under EECH fails with
  `EECH_STATUS_FPU_DRIFT`. The replay also checks the mode before each
  scenario line, as the C reference does.
- **Host callbacks.** They run under the **host's** environment: the
  trampoline reinstalls it around each call to `World`. A DCS host therefore
  never sees EECH's rounding mode, and EECH never sees the host's.
- **Declared-type evaluation.** SSE arithmetic (`FLT_EVAL_METHOD == 0`,
  checked at compile time) and `-ffp-contract=off` (no FMA contraction on
  targets that have it), without `-ffast-math`. On i686, `-msse2 -mfpmath=sse`,
  as in the C reference.
- **`convert_float_to_int`.** The C `(int)` cast, which truncates. That equals
  x87 `fistp` under EECH's round-toward-zero control word (eech-core-ts slice
  3), and it is portable.
- **Constant folding.** The original code is **not** compiled with
  `-frounding-math`, so its constant expressions fold at compile time, to
  nearest, as EECH's compiler folded them. (The generated aircraft database's
  knots-to-metres values depend on this; the corpus's `aircraft-database` entry
  checks all 33 of them bit for bit.) The host layer's own C *is* compiled with
  `-frounding-math`, because it switches rounding modes itself (scenario input
  is parsed to nearest).

## Results

All runs compare against the canonical 32-bit C reference: the recorded corpus
(1,401 scenarios and 5,000 float operations), plus the fresh corpus (7,000
scenarios and 28,000 float operations). The float operations cover narrowing,
sums, products, quotients, `sqrt`, double sums and the crate-row step, with
subnormals, infinities and NaNs.

| Build | Result |
|---|---|
| x86-64, gcc 13, `-O0` (default) | identical |
| x86-64, gcc 13, `-O2` / `-O3` | identical |
| x86-64, clang, `-O0` / `-O2` | identical |
| i686, gcc 13, `-O0`, SSE | identical |
| x86-64 Windows (MinGW gcc 13), under Wine | identical |
| x86-64, rounding **to nearest** (`EECH_FPU_ROUNDING=nearest`, investigation only) | **differs**: 200 of 1,401 recorded scenarios, 1,198 of 7,000 fresh; the float operations differ too |

What this shows:

1. **The native targets introduce no numerical difference** in the slice's
   code, at any optimisation level, with either compiler. `sqrt` included:
   x86-64 uses `sqrtsd` under the MXCSR mode, where the i686 C reference used
   libm's x87 path under the x87 control word, and the results are identical.
2. **The rounding mode matters, and the tests see it.** Round to nearest
   changes supply levels, ranges and crate positions in about one scenario in
   six, so the corpus is a working guard on the environment.
3. **NaN payloads** match the C reference bit for bit (x86's negative default
   NaN). The TSTL port deliberately does not model them
   (`test/c-reference/float32-rtz.cref.test.ts`).

## Modernising deliberately

Every numerical decision lives in one place and can be changed on purpose:
- the rounding mode: `EECH_CAMPAIGN_ROUNDING` in `csrc/eech_kernel.c`;
- the conversion: `convert_float_to_int` in `csrc/eech_env.c`;
- the evaluation flags: `build/main.rs`.

The corpus then measures what the change does. The x87 intermediate-precision
question stays where eech-core-ts left it (§9.3 there): if a trace of the
shipped binary shows extended-precision intermediates, the module can model it
the way the C reference's investigation variants do. Until then, declared-type
evaluation is the contract.
