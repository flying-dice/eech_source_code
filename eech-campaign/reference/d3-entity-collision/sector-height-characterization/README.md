# Sector tallest-structure height: characterization (SECTOR-HEIGHT-CHARACTERIZATION-1)

**UNREVIEWED CODE_FROZEN, characterization only.** Task EEDCS-49 (D3), 7 November 2026 (UTC, the session began 6 November). Base `8a525b145c8c54f2b22863c20d908ee3389759e5`, code commit
`a8ef0ce662df0528dcae414287ffe45b2847cd8f` (six files, all tests, one feature, one document; **no path under `aphavoc/` or `modules/`, no production C/Rust/Lua, ABI, shipping macro or baseline changed**:
`git diff 8a525b145 a8ef0ce66 -- aphavoc modules` is empty). The pack is committed separately as metadata. The contract is
[`docs/d3-sector-height-contract.md`](../../../docs/d3-sector-height-contract.md); the external freeze report is `R:/d3-evidence/sector-height-characterization-CODE-FROZEN-20261106.json` (copied here).

`ENV-ENTITY-POSE-LONG-RUN-MOTION-1` stays **open** and `blocks_cutover=true`: the existing corrected f734 fullpack has no actual completion or independent review, and the finite CITY acceptance cannot
close it. Nothing here is a cutover, a delivery claim, a native result, a Windows result or a baseline change. The progress guard was run with the intended action first (`--action characterize`, exit 0);
this is the **first** of the two allowed preparation batches (count 0 → 1 of 2).

## What was found (details and the input classes: the contract document)

1. The scalar is a **high-water value**: raised only when a fixed entity is listed into the sector (strict `>` of `position.y + box ymax` over the zero word), never lowered, recomputed or reset by a removal, a
   sector change or a shape change. A destroyed or moved tall structure keeps its clearance.
2. The only computation is one float addition and one comparison. The engine runs toward zero; a sum that is not exact is stored rounded toward zero (`1 + 1.5·2^-24` → `1.0`, the largest finite doubled → the
   largest finite). Zero floor: a negative, ±0, NaN or -infinity sum never stores, and -0 is never stored. Identical words on x86-64 (SSE) and i686 (x87), debug, engine C at its default level 1.
3. The save is the stored word as the **last 32 raw bits** of the sector record, `PACK_MODE_SERVER_SESSION` only; sectors are unpacked before every entity (a load's insertions can only raise it). A client or
   browse session carries and restores nothing. Zero, -0, subnormals, extremes, both infinities and a quiet NaN round-trip bit for bit; a signalling NaN is not claimed (x87).
4. The box top is **already host-owned** in the shipping profiles (the staged accessor serves the table `campaign_model_install_scenes` fills; the host refuses a non-finite bound), and the fixed entity's position
   and shape are already stated to the host at every insertion. The host already takes the value as fact 12 of every candidate question.
5. Source facts the move must respect: the hosted scan reads it once per served sector; the original scan reads it only in the build that keeps the original solver; the fixed-wing terrain follow reads it
   unconditionally before the hosted movement block and subtracts the terrain elevation from an absolute height (preserved, a candidate defect, not approved); nine fixed families can raise it but the host's
   fixed state holds eight (the **city** has its own state with no shape, and its creation sets the invalid shape with the default-shape resolution commented out: whether a real city ever raises it is **unknown**).

## Evidence (every command a direct Cargo child, exit read from the runner's own receipt, no pipeline)

Runner `run-fixed-rules-correction-direct-20261106.py` (x86_64 Linux debug, one cache owner, image `sha256:4a5618cb…`) and, for i686, `run-tree-i686-direct.py` (image `eech-d3-tree-i686-20261106:2`).
All records are under `direct-records/` (receipt, stdout, stderr per run); `direct-records/runs.json` is the machine summary.

| final run | command | exit | result |
|---|---|---|---|
| shf1 | `cargo fmt -p eech-engine-sys -- --check` | 0 | |
| shf2 | `cargo test --locked -p eech-engine-sys --features sector-height-check --test sector_height -- --nocapture` | 0 | 1 test, **179 expectations** |
| shf3 | `cargo test --locked -p eech-engine-sys --test sector_height_guard` | **101** | 5 of 6: the new route test asserted an English comment of `build/main.rs` — **retained, failed** |
| shf4 | existing guards `entity_roster_guard entity_host_guard tree_host_guard entity_pose_lifecycle_guard` | 0 | all pass (default features, unchanged) |
| shf5 / shf6 | `cargo clippy --locked -p eech-engine-sys [--features sector-height-check] --all-targets -- -D warnings` | 0 / 0 | |
| shf7 | the control on `i686-unknown-linux-gnu` (x87) | 0 | **179 expectations**, same words |
| shf8 | the same with `env EECH_C_OPT=2` | 0 | **NO-OP DUPLICATE**: Cargo reused the shf7 binary (2.4 s, no `Compiling`): `EECH_C_OPT` is read only by `eech-sys` and `eech-world`; `eech-engine-sys` reads `EECH_ENGINE_OPT` (default 1) and the runner grammar does not pass it. **No -O2 claim for the engine C.** |
| shg1 | `cargo fmt -p eech-engine-sys -- --check` (after the guard fix) | 0 | |
| shg2 | `cargo test --locked -p eech-engine-sys --test sector_height_guard` | 0 | **6 of 6** (the test now pins the actual patch-selection code, not a comment) |
| shg3 / shg4 | clippy as shf5 / shf6, after the guard fix | 0 / 0 | |

The shf3 failure was a guard-only text assertion; Root's feedback named it and asked for the selection code to be pinned instead. Only guard-only checks were rerun (shg1 to shg4); the real-C control
(shf2, shf7) was **not** rerun because its inputs are literal across the two maps (below).

**Input maps** (`maps/`; rows of the second and the post map are external, `R:/d3-evidence/sector-height-FINAL-*.rows.tsv`): scope is every tracked and untracked-non-ignored file under `aphavoc/`, `modules/` and
`eech-campaign/` except `docs`, `reference`, `target`, `tools` (the cargo inputs), raw working-tree bytes, 4080 files each.

| map | taken | aggregate sha256 |
|---|---|---|
| FINAL-PRE-CHECK (before shf1 to shf8) | 00:16:23Z | `b41eb0e9ffa8a9c3000c07eabde893bdfea1db845e8079e69cc0b4c5b11d2f52` |
| FINAL-PRE-CHECK2 (before shg1 to shg4) | 00:23:56Z | `c365ecb7e10dee238815e322897bdf44fa573594115372e90038f579928a8792` |
| FINAL-POST-CHECK | 00:24:24Z | `c365ecb7e10dee238815e322897bdf44fa573594115372e90038f579928a8792` (**identical to PRE2**) |

PRE → PRE2 binding: `git status --porcelain` and the file count are equal; the five changed files' hashes are equal except `tests/sector_height_guard.rs`; every other file is its Git blob at the base in both
maps. So `Cargo.toml`, `build/main.rs`, `csrc/eech_sector_height_check.c`, `tests/sector_height.rs` and all production/build/target inputs are literal for shf2 and shf7. The committed bytes are the POST map's bytes.

**Sensitivity of the control** (`mutants/`; harness-sensitivity evidence, not a product defect and no required RED): four one-line mutants of the REAL sources, each run through the same runner, all exit
101 with named failures — last arrival wins (`sc_msgs.c`, 20 failures), removal lowers (4), unpack skips the word (`sc_pack.c`, 17), box top ignored (20). Each file was restored from a saved copy and
`sc_msgs.c` `383b5571…eed13` and `sc_pack.c` `3f23c5ab…fedd30` **equal their Git blobs at the base**; all **112** staged copies of either file in the Linux cache (every build directory, i686 included) equal the same
blobs, and the feature configuration's copy was re-staged after the restore (00:16:48Z) by the shf2 build, so the final control ran on restored inputs, not the last mutant's staged cache.

## Declared qualifications (bound by the freeze; Root's guidance)

- The feature `sector-height-check` is dedicated and TEST-ONLY: off by default and in every shipping build, its C entirely under `EECH_SECTOR_HEIGHT_CHECK`, its test binary `required-features`; no unrelated
  feature is coupled. The default build compiled the empty file (shf4 to shf6, shg2 to shg4 ran without the feature).
- **Hand-made and scripted data is declared**: the entity structs, the sector map and ONE scripted model-provider callback (a box per distinct top, installed by the real `campaign_model_install_scenes`).
  **The message table is not the public `en_list.c` funnel**: the roster hook, the list write and the notification order were not run. That gap is explicit and has a named public trigger (contract §5).
- The control saves and restores the dispatch tables, `message_responses`, `world_map`, `entity_sector_map`, the pack buffer pointers and the rounding mode; it cannot restore `mb_hostmodel.c`'s private
  installed scenes and callback. The safety bound is the process: its own test binary with one test, and the module's own rule that every boot reinstalls them.
- The census guard is a **source/build-derived census** (text patterns over the sources and the build's source list), not an exhaustive preprocessed-symbol or runtime census; compiled inclusion is inferred from
  the build flags and the `#if` structure. It cannot close the whole-D3 final compiled-consumer disposition.
- Float and save semantics are recorded as the original has them; nothing becomes a finite-only production policy.

## Process record (honest)

- Root's optional feedback `R:/d3-evidence/Root-sector-height-review-feedback-20261106.json` did **not exist** at the first check, appeared at 23:55:24Z after my first source writes, was read
  immediately, grew at 00:09 and 00:21 (re-read both times) and was re-read before the final checks and before the freeze. Its guidance was applied after the fact: the feature qualification,
  declared hand-made data, restoration of the changed globals, the census wording, the mutation restore proof and the guard fix. A copy is in this pack, literal.
- Iteration failures are retained (`direct-records/`): sh1 the first control abort (the hosted bounding-box accessor needs installed scenes: that is how the original's accessor was replaced), sh2 14 failures
  (an arithmetic slip of mine, 200 + 5 is 205, and a decode that consumed the stream the engine's unpack shifts in place), sh4 three guard failures (assumptions about the text). All were bugs in my new test code.
- shf8 was the no-op duplicate above; it is kept as a failed *bookkeeping* result, not evidence.

## Limitations and open items

No retail data, native campaign, Windows build, Tacview or full-war evidence; no Windows/native source clearance or authority claim; no host-sourced value. The city's shape question, the native observation
of the campaign's word against the host's high-water, the DCS consumer/API decisions, and everything listed as still required in the contract remain open. A DCS host answers clearance from its own obstacle
knowledge; nothing here claims render dormancy.

## Next bounded action

SAME independent reviewer over ONLY this change (the control's honesty about its hand-made data, the guard, the contract), before Root decides the native observation through the named public trigger. The
physical production move (host-owned per-sector scalar, contract §6) waits for `ENV-ENTITY-POSE-LONG-RUN-MOTION-1` to close by an actual corrected fullpack and review.
