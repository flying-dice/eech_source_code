# D3 Task49: i686 enumeration pose, x87 evaluation model (UNREVIEWED CODE_FROZEN)

Code `681ccfe5476a7c0a90036bc01cd054358c751162` over metadata `e442946e5589ae9b91a1522bf4b35afec490cc63` over tree raw-copy code `0a7330f860c19a45a415f8a6cbf77513a9db84be` over `cd97e0b723e4f6710622ba9c6c19a16516fa4174`. Source and Linux checks only: **no native actual, no Windows/Wine run, no independent review, no SourceClearance, no acceptance, no CI success.**
The machine-readable account is [i686-enumeration-pose-CODE-FROZEN-20261106.json](i686-enumeration-pose-CODE-FROZEN-20261106.json); run records are in `direct-records/`, the scratch controls and their logs in `env/`, the CI snapshot in `ci/`, hashes in `manifest.json`. The tree raw-copy pack and report (`03168a4a...`), the `cd97e0b72` report (`62445b90...`) and every earlier pack are preserved and not rewritten.

## The failure

`tests/entity_candidate_host.rs:907` (world 1) fails on i686 and passes on x86_64: the ordered UPDATED/Normal/Narrow/refusal events agree, but five words of the terrain-aligned placement of a fixed candidate differ by one ulp. It was reproduced on the pristine `d45221d95` and on the tree-fixed `e442946e5` with identical text (receipts in the tree pack, referenced by hash in the report; not replayed), and the `e442946e5` CI shows it in both i686 jobs after the tree failure went away. CI could not show it earlier because cargo stops at the first failing test binary.

## Which side is wrong (measured, not assumed)

A scratch control extracts the ORIGINAL face-normal functions by name from `modules/maths/matrix.c` and `vector.c` and drives them, and the production file's own statics, with 2000 raw-word inputs that are identical on every target, under round-toward-zero and the production flags (`env/pose-control.sh`, `env/original_extract.sha256`).

- **x64 (SSE)** is the oracle: original and production text agree on all 2000 (the Windows x64 original, the project's declared policy for the rotor kernel).
- **i686 x87, production flags:** the original text differs from x64 on 1990 of 2000 inputs and the production text on 1999, and the two differ from EACH OTHER on 1657: the same original expressions are not one answer on i686. Extended precision flows through whole expressions and, once a static helper is inlined, between statements (`-std=gnu99` implies fast excess precision). Even `-fexcess-precision=standard` (context-independent: O0, O1 and the production text agree) differs from x64 on 1954, because x87 still evaluates each expression extended.
- **i686 with SSE float math** (diagnostic only, never used in production) reproduces the x64 words on all 2000 at three optimisation levels: the libraries agree and the divergence is purely the evaluation model.
- **Classification:** a production defect on x87 targets. The Rust transcription in the test is the x64 original's per-operation single-precision evaluation; built for i686 (SSE2) it computes the x64 words, so it is not the defect.

## The correction

`csrc/entity_candidate_step.c` only: `NARROW` (a volatile float store and read-back, the identity where `__FLT_EVAL_METHOD__` is 0) at each float operation of the face-normal placement, local copies of the normalise and magnitude helpers, the speed test, the sweep ends and the broad-phase and city boxes. After it, the patched production text equals the x64 words on all 2000 at i686 O0, O1, O2 and standard excess precision. No build flag, oracle, tolerance, skip or golden change; `eech_arith.h`, the other kernels and the FP guard are untouched. The test file gets a comment only. The workflow gets `--no-fail-fast` on its two `cargo test` steps (matrix and windows-cross) so every binary's result is reported; targets, features, environment, lock, flags and required jobs are unchanged and the job still fails when any binary fails. Cause and control: CI stopped in the lib binary and hid this failure; the local no-fail-fast run exposed both.

## Checks

On the clean committed tree, maps literal equal before the first check and after the last (3953 files, 3950 raw-equal to the committed blobs, three inherited CRLF Lua files qualified; the workflow file, outside the map roots, is bound directly: blob bytes equal the working tree): `entity_candidate_host` 13/13 on i686 default, on the i686 original-stack row (the SAME compiled binary: the variable changes no artifact of this scoped target) and on x86_64; `cargo fmt -p eech-world --check`; `cargo clippy --locked --all-targets -p eech-world -- -D warnings`. Two development runs before the commit are retained and not claimed as final. All runs went through the scoped runner `run-d3-pose-focused-direct-20261106.py` (guard not bypassed). No whole-workspace, release, C_OPT=2, native, Windows or Wine run was made; the unchanged material keeps its earlier qualifications.

## Limits and open items

Other kernels with float expressions (the scan window, convoy, ship, person, fw, hc, los, sense, aim, cmb, tree arithmetic) were not audited for the same class. The control covers the placement; the other corrected sites are covered by the 400-world differential. No retail 32-bit MSVC behaviour is known or claimed. e442 CI: x86_64 default and `EECH_C_OPT=2`, windows-cross, evidence and dcs-probe jobs pass, the structural run passes; this correction has not been run in CI and Windows exact-head CI remains required.

Root's latest read-only f734 prefix capture (16:57:57Z on 6 November 2026, `tree-x64-controls-default-prefix-f734-20261106.json`) has all four prefixes literal at 3963.74/3945.76 simulated seconds, PARTIAL ONLY: a dated capture, not live, not complete-pack, 40/all8 or D3 acceptance; earlier prefixes in older records are history. `REQUIRED_CITY_BOX_LIFECYCLE_RETENTION` stays OPEN and is the mandatory next CITY cut after the finite flags; default shape/clutter and draw-align dispositions, sector height, UPDATED terrain, raw TREE/FOREST/CLOUD RNG, hierarchy services, DCS mapping, natural moving, final 40/all8, current CI, normal merge, the final census and all ten D3 requirements stay open.

## Pack accounting

54 listed manifest records plus `manifest.json` itself = 55 Git files in this pack.
