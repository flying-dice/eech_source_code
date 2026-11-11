# D3 Task49: correction of the fixed-rules review and current CI (UNREVIEWED CODE_FROZEN)

Code `4fb81c673bd9ca82c9ff3fa801e553e56c1b6453` over `bfce56e3c9e56352c1f2c6beb6c53d930dbee649` (metadata) over code `d2f5027c9154841c32f70ed87747b9d908e08e48`. Source and Linux checks only: **no native actual, no Windows/Wine run, no review, no SourceClearance, no acceptance.**
The machine-readable account is [fixed-rules-review-correction-CODE-FROZEN-20261106.json](fixed-rules-review-correction-CODE-FROZEN-20261106.json); every run record is in `direct-records/`; hashes are in `manifest.json`.
The original pack [fixed-physical-rules-source-and-checks](../fixed-physical-rules-source-and-checks/README.md) and the same reviewer's `CHANGES_REQUIRED` report (`18b2faf8...`, one P2) are preserved literally and are not rewritten; where this pack corrects them it says so.

## What was corrected

1. **ENV-FIXED-RULES-RESTATEMENT-ACK-1.** The witness's `changed_fixed_rule` restatement was credited for any `placed` reply with the same name, even one that also carried a cursor, which the campaign's own fixed-report contract forbids. It now uses `scan.fixed.reply_problem` (placed, this placement's key, no cursor). A damaged acknowledgement is a reply fault with its own verdict line and is never credited. RED: the damaged control was credited against the unchanged bfce Lua; the positive control passes before and after.
2. **Current CI, x86_64/Windows/C_OPT=2: a signaling NaN phase raised the invalid flag in unoptimized builds.** `entity_host::scan_step` ran the floating `check_scan_facts` (whose first test is `f32::is_finite`, a compare that signals on a signaling NaN unless the optimizer folds it into a bit test). Release passed; CI's required debug job did not. Reproduced on the clean unchanged bfce, located to the flag level by an untracked control (source retained), corrected by an integer classifier (`classify_scan_words`) in the same order with the same messages that runs before any word is a float. No assertion, tolerance, flag, profile or baseline was touched. The defect is not new in d2f: the failing path and tests are byte-identical from `90b38189`, and a probe before the rules cut failed identically.
3. **Current CI, i686: a different failure.** Two city-box tests used the x86_64-only `fpu` helper without the `cfg` their siblings carry (compile error, 14 sites). They are now gated the same way. They therefore do not run on i686; i686 city-box parity is not claimed by them. The i686 build could not be run here (no 32-bit libc in the pinned image).
4. **Workspace clippy.** The two `expect_err` uses in `eech-dc` tests are rewritten with the file's own `let ... else` idiom. The workflow's own `cargo clippy --locked --all-targets -- -D warnings` (workspace) and `cargo fmt --check` pass on the final bytes.

## Chronology erratum for the d2f pack

The original freeze key `red_before_production_edit` is chronologically false for the provider and source-guard REDs: implementation edits began 13:37, the provider RED ran at 13:45 and the guard RED at 14:20, both after the initial GREEN, by controlled restoration of the parent's production files. The provider failures at `90b38189` mostly exercise the missing new subtype-wire capability; they are not 27 independent defects, and the old Core-false CITY bypass is pinned separately. The original text above (including "New row fields appear only in that mode") is superseded by the next section.

## The four added diagnostic fields

`fixed_rule_eligible`, `fixed_rule_changed`, `fixed_rule_changed_next` and `fixed_rule_changed_cause` are emitted by every mode's authority row, the healthy control included. A native comparison declares all four as added fields and requires every existing shared field, the complete physical terminal and the default recordings to be exact.

## What this does not change or claim

The 15 physical rules (including the valid UNKNOWN), the fixed/CITY Host records, the original math, the finite constructor proof and every C, sys and extractor input are unchanged (equal Git tree ids with d2f). The default-shape and clutter columns were outside the cut and are not declared campaign-owned; their consumers need the census disposition. The six draw files' direct `align_with_terrain` reads stay an unproved census item. `IDENTIFY_FIXED` policy stays Core. **`REQUIRED_CITY_BOX_LIFECYCLE_RETENTION` stays OPEN and is the next CITY production** (an empty-CITY relink/restore must retain/authorize the Host box without Core's mirror as the sole copy); sector height does not come before it. All ten D3 requirements, the natural moving case, the final 40/all8, current CI, the normal merge and the whole-D3 acceptance remain open. The f734 ordinary full pack was not touched.

## Evidence

Run records in `direct-records/`: the debug RED on bfce (`fpu-red-bfce-debug-a`), the flag-location controls (`fpu-diag-debug`, `fpu-diag-release`, `fpu-diag-test-source.rs.txt`), the damaged-ACK RED (`ack-red-bfce-lua-debug`), the GREEN runs, the retained `cargo fmt --check` exit 1 and its repair, the C_OPT=2 profile, the unavailable i686 attempt, and the final chain (whole-workspace debug, workspace clippy, whole `eech-world` release). Concise CI excerpts are in `ci-bfce-failure-excerpts.txt`; the raw CI logs stay outside Git (hashes in the report). Input maps before and after the final checks are literal equal (3953 files, 3950 raw-equal to the commit's blobs plus the three inherited CRLF Lua controls, listed in the raw binding).
