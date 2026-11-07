# D3 Task49: valid-subnormal pre-save floating-point correction of the scan classifier (UNREVIEWED CODE_FROZEN)

Code `cd97e0b723e4f6710622ba9c6c19a16516fa4174` over metadata `870a243fba4d0e683697994617aacf60eedfd994` over code `4fb81c673bd9ca82c9ff3fa801e553e56c1b6453`. Source and Linux checks only: **no native actual, no Windows/Wine run, no review, no SourceClearance, no acceptance.**
The machine-readable account is [fixed-rules-subnormal-CODE-FROZEN-20261106.json](fixed-rules-subnormal-CODE-FROZEN-20261106.json); the same reviewer's sealed `CHANGES_REQUIRED` report (`d6b5df9f...`, one P2) is copied literally beside it; every run record is in `direct-records/`; hashes are in `manifest.json`. Earlier packs and reports (including `18b2faf8...`, `c926479a...`, `d960ae7d...`) are preserved and not rewritten.

## The finding and its disposition

`ENV-SCAN-PRE-SAVE-SUBNORMAL-FPU-1`: after the integer `classify_scan_words`, `scan_step` still ran the floating `check_scan_facts` before `fp_state` captured the caller. A valid subnormal coordinate passes the raw classifier and then its floating `abs`/range comparisons set the SSE denormal status (or trap if unmasked) before the snapshot, so the later `after == state` guard compared an already-modified state with itself.

- **RED first, on the unchanged 870a source:** a new control in the existing FPU harness (`entity_roster_host`) with the real provider, roster and kernel and raw subnormal coordinate words failed at its first masked, DAZ-clear context: MXCSR 8066 against 8064. The helper's `exception_flags()` read 0 both times, so only the complete-state comparison shows it.
- **Fix:** `classify_scan_words` is the only production validator. `scan_step` classifies, then captures `fp_state` and applies the existing unsupported-environment refusal, and only then turns the words into floats for the kernel. The floating predicate is `#[cfg(test)]`, the oracle of the differential test (whose corpus now has subnormal words of both signs). Domains, order, messages, error priority, masks, rounding and sticky flags are unchanged; no input is flushed or rejected and nothing is masked or cleared.
- **Control:** four rounding modes, flags clear and pre-raised, six denormal contexts (DAZ/FTZ, two with the denormal exception unmasked with DAZ set). Masked contexts are served and the whole state is unchanged; unmasked contexts get the existing named refusal and an unchanged state. A context that would have trapped before the fix (unmasked, DAZ clear) is deliberately not entered.
- **Checks on the committed code** (clean tree, maps literal equal before and after): `cargo fmt --check`, whole `eech-world` in debug and in release (1173 tests each: the provider path is global to every scan question), and the workflow's workspace `cargo clippy --locked --all-targets -- -D warnings`.

## Scope and what is reused, not claimed

Git tree ids of every C, sys, engine, DC, Lua and Cargo input equal those of `4fb81c673`; the code commit differs in exactly `entity_host.rs` and `entity_roster_host.rs`. The whole-workspace debug (1410/79), `EECH_C_OPT=2` (1410/79) and release `eech-world` (1172) results of `4fb81c673` are reviewed PARENT checks, not new-head runs; no whole-workspace, C_OPT=2, native-sys, C, Windows/Wine or i686 run was made at the new head. Standard current-head CI must still pass. No i686 build is claimed: the retained local attempt failed on two missing prerequisites, the Rust `i686` target and the 32-bit libc; the cfg-gated city-box tests do not run on i686.

## Metadata provenance erratum

The earlier statement that the external report `d960ae7d...` and the tracked copy `c926479a...` differ only by the publication block and `identity.meta_commit` was incomplete. Strict deep equality needs exactly three adjustments to the external copy: remove `publication`, remove the external-only `review_sha256` (the tracked key is absent, not null; its value equals the nested `18b2faf8...`), and reset `identity.meta_commit` to the tracked null. Re-verified here on the literal files. The earlier pack has 51 listed manifest records plus the manifest itself, 52 Git files.

## Unchanged

The four `fixed_rule_*` fields stay declared in every mode, healthy included; every existing shared field, the complete physical terminal and the default recordings must stay exact. `REQUIRED_CITY_BOX_LIFECYCLE_RETENTION` stays OPEN and is the mandatory next CITY cut. Default shape/clutter and the six draw-file align readers stay open for the needed-caller/DCS disposition; Core keeps campaign policy. All ten D3 requirements, the natural moving case, the final 40/all8, current CI, the normal merge and the final census stay open; D4/D5/TypeScript are unstarted. The f734 ordinary pack and every consumed native root are untouched.
