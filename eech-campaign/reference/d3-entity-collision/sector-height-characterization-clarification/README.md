# Sector-height characterization: clarification of the three P2 precision findings

**UNREVIEWED correction, within the completed characterization batch (preparation 1 of 2, not a new batch), comment and prose only.** 7 November 2026 UTC. The SAME independent reviewer's consolidated report
(`independent-sector-height-characterization-source-a8ef-20261107.json`, sha256 `7471e653366196803eb7a66dbc9b12287aacc140c1b19388a3b805c27a3904b9`, literal in this packet) decided
`CHARACTERIZATION_CONTROLS_VERIFIED_CONTRACT_CHANGES_REQUIRED`: the linked controls are verified, three P2 findings are contract/evidence precision. Closure of all three is **pending that reviewer**; the
card carries them as OPEN (registered before the named `fix-blocker` guard, which exited 0 for each). `ENV-ENTITY-POSE-LONG-RUN-MOTION-1` is untouched and stays `blocks_cutover=true`. Nothing here is a new test,
mutation, build, native or Windows run, a source clearance, a physical cutover, or whole Task49/D3 acceptance.

## The three corrections (current wording is [docs/d3-sector-height-contract.md](../../../docs/d3-sector-height-contract.md))

1. `ENV-SECTOR-HEIGHT-SAVED-WORD-PLAN-1`: the zero floor holds for zero-initialised histories only; the strict `>` is against the word ACTUALLY held. Source-derived (not measured): after a raw restore a
   larger negative sum can raise a negative word, and after a restored NaN every comparison is false. The next host plan compares against the actual held or restored word, never clamps to zero or `fmax`-es a
   NaN away, and separates a malformed or mismatched reply from a valid raw saved word (negative, infinity, quiet NaN), which is restored, not refused.
2. `ENV-SECTOR-HEIGHT-MEASURED-DOMAIN-PRECISION-1`: `save_words` is **13 representative raw patterns** (zero, -0, both subnormal edges, the smallest normal, 1.0, 110.0, 10000.0, both extremes, both infinities,
   one quiet NaN), not every finite word. The wider unpacked 32-bit domain is source-derived. Signalling-NaN, -O2 / non-default engine-C level, the Windows game and any FP state beyond the restored rounding mode stay
   explicit limits.
3. `ENV-SECTOR-HEIGHT-RAW-BINDING-PRECISION-1`: of the 4080 raw final rows **4077 are literal Git blobs and 3 are inherited `eech-world` test-Lua CRLF-vs-Git-LF files** (exact CRLF→LF only, identical at base and
   code, raw specimens retained at `D:/EECH-Migration-Evidence/sector-height-source-binding-a8ef0ce66-20261107/inherited-lua-raw`; Root's proof is literal here). PRE2 and POST rows are literally equal
   (TSV sha256 `622b349f…`). **PRE1 was reconstructed** from the final rows plus the recorded old guard digest; its aggregate `b41eb0e9…` matches, but it is not a retained original full-row capture. The constructor script
   `inputmap-construction.js` (sha256 `deb82b37…`, copied from that D: folder) is the one Root captured now; it is **not** an attestation of the original PRE1 script's identity. No input, source or baseline byte was normalised.

## What changed, and the proofs (`clarification-proofs.json`)

| | old (`a8ef0ce66`) | new (`3fac41f04`) |
|---|---|---|
| `csrc/eech_sector_height_check.c` raw sha256 | `7b80c8e323dca3ef7cd797d6e8c192163db18bc4087bc8c2eb45e9a8b4e3e20f` (28381 B) | `84f4c9e6039fb5f95ad6007719b5f41b7c83b5bd750fb16460affcfbc4f0f89c` (28962 B) |
| suffix from `#ifdef EECH_SECTOR_HEIGHT_CHECK` (23689 B) | `df20a3b0…47f4f` | `df20a3b0…47f4f` (**byte-identical**) |
| `#ifdef` line / line count | 41 / 859 | 41 / 859 |
| `docs/d3-sector-height-contract.md` sha256 | `3f1fdea5…cc80c` | `678d36af…bf7af` |

Only comment lines 22, 27, 29 and 30 of the C header changed (before the `#ifdef`; no statement, helper, expectation, test case, build, Rust, ABI or protocol change), so every `__LINE__` constant is unchanged.
`git diff a8ef0ce66 3fac41f04` over the code paths is exactly those two files. **The raw C-file hash changed and is named above.** The 179-expectation x86-64 and i686 controls, the 6 guard tests, the existing guards and the
lints ran on the ORIGINAL source `a8ef0ce66` under its 4080-row maps (`b41eb0e9…` / `c365ecb7…`); they were **not** rerun and are not claimed for the new raw bytes. The equivalence offered is the byte-identical code body
and start line, not a new run. Source and binary identities are not relabelled. The identification map in the proofs file is a read-only description of the new inputs, not a check.

The whole existing packet `sector-height-characterization/` (82 files: README, freeze report, manifest, failed and no-op records, maps, mutants) is **literally unchanged**: every file equals its Git blob at `12a223556`
and at HEAD, and `git diff` over that directory is empty. The historical statement "every other file is its base Git blob" and the README's "every finite word" style remarks stay as written and are qualified by this packet
and the current contract.

## Commands and exits

Read-only or Git only: the named `migration-progress.py --action fix-blocker` guard for each of the three ids (exit 0, 0, 0, run after registering the findings in the card); the header patch script (aborts unless the
suffix, start line and line count are identical; exit 0); the contract patch script; `git commit` and normal `git push` of `3fac41f04`. **No Cargo, build, test, mutation, runner, cache or native command ran.**
Optional `Root-sector-height-review-feedback-20261106.json` was reread before the freeze: unchanged (4754 bytes, mtime 00:21:14Z, equal to the copy in the historical packet).

## Next

Root hands the changed delta (the four header comment lines with the proof of the identical body, the contract prose and this packet) to the SAME reviewer. No next batch is started. The physical move still waits for
`ENV-ENTITY-POSE-LONG-RUN-MOTION-1`.
