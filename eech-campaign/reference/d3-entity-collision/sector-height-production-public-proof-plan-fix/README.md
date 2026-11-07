# Sector high-water height: the public proof plan correction (`ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1`)

Task EEDCS-49 (D3), 7 November 2026 (UTC). **UNREVIEWED; native UNRUN; the finding stays OPEN until the SAME reviewer closes it.** Source before: the checkpoint `4a228c305` (the finding registered OPEN in the card) over the production source
`0be1dd8465d9bf38da69cd3b303206cf5a64574d`. This is a required bounded correction inside the height extraction, not a preparation batch (the count stays 2). It is **test-only**: no production solver, protocol, kernel or
compiled campaign input changed. No Windows, native, campaign or war launch happened here; the Root-prepared `1cf5` clone was not touched.

## Inputs (literal copies here; `literal-bytes-check.json` compares each to its ACTUAL Git blob bytes)

- `independent-sector-height-production-0be-source-20261107.json` (SHA-256 `49d7739e...ce58c`): the SAME combined review, `CHANGES_REQUIRED_PUBLIC_VALIDATION_PLAN`, `native_launch_clearance=false`, ONE P2, no new functional/math/protocol defect.
- `sector-height-production-Root-binding-0be-v2-20261107.json` (`ec7015b6...`): Root's corrected raw-byte binding. `sector-height-production-Root-binding-0be-20261107.json` (`7b5cc4a7...`): the **initial** binding, retained with its reader error (it confused
  filtered identities with raw bytes). Direct Git-byte comparison: exactly **3 unchanged Lua CRLF raw-versus-Git exceptions** (the tree-geometry conversion-refused, one-shot and refused-probe controls) and **4 Core filter-only
  qualifications** (their raw bytes equal the Git blob; only the filtered label differs). No file was normalised.
- `sector-height-production-1cf5-CI-readback-20261107.json` (`03abecc4...`): the exact `1cf5` CI, all nine jobs success. Root's optional feedback `Root-sector-height-production-feedback-20261107.json` (literal; present before the PRE map, unchanged).

## What the finding said, and the correction

| finding evidence | correction |
|---|---|
| the promised public comparison and Host authority/refusal gate is not executable; the scripted tests manufacture phases 17/18 | `entity-authority.lua` modes `sector_height_control`, `_wrong_word`, `_withheld`, `_malformed`, `_wrong_state`, `_wrong_sector`; eligibility, consumption and named outcomes (below); the scripted tests are labelled **boundary checks**, never native authority |
| the observer cannot be compared whole, yet is not unusable: SH10/SH11 still capture the Host-returned FIRST/fact 12 | `sector-height-baseline.lua` `scope=hosted-scan` (+ `accepted=`): writes ONLY the scan rows and the clearance guard records and compares by sector at the same window with the accepted original report; the wrapper gains `-Scope`/`-Accepted` (parse-only AST check passes) |
| the clearance helper reads retired `raw->tallest` storage | in a hosted build the helper reads nothing retired and never re-queries: the record says `tallest=uncaptured` (RED against the old helper: `n6`) |
| section 8 claimed the guarded fw execution was controlled | corrected: it was not; no control reaches it; the real-C scripted-provider pack/unpack is not a native Core save/load; a Host-only raw restore is not one either |
| section 7 claimed a restore-after-insertion refusal | corrected: the provider deliberately ACCEPTS a well-formed phase 18 with live placements; no such rule is invented |

## The executable plan (contract section 11; Root runs it)

**A. Scoped baseline comparison** (window `lebanon_retail`, `hours=0.001`, `frame_ms=100`, `seed=1`; never extended or searched): build the hosted observer DLL as for the accepted baseline, then
`powershell -File tools/sector-height-baseline-windows.ps1 -Root <fresh root> -Bin <observer bin> -Out <new dir> -Scenario lebanon_retail -Hours 0.001 -FrameMs 100 -Seed 1 -Scope hosted-scan -Accepted <the f01 report>`.
Compared by sector `(sx, sz)`: `reads`, `first_word`, `last_word`, `changes`, `fact_offers`, `fact_equal_first`, `fact_differ_first`, `fact_first_word`, `fact_last_word`, `fact_changes`, and the `sho-scan` summary totals (the accepted
63 FIRST reads over 2 sectors and 27615 fact-12 offers, at that scope only). Not compared: sequence numbers, INSERT/LINK/statement joins, shape states, the original raise and save/restore, the clearance word, the 304811-event
original-LINK diagnostics. Credited only with exit 0 and `status=EQUAL_AT_SCOPE`; `DIFFERENT`, `NOT_COMPARABLE` (another window or an unreadable report) and `UNREACHED` (no scan FIRST) are distinct non-passes with their output kept.

**B. Authority and refusal witnesses** (`eech-world.exe <literal lua/entity-authority.lua> root=<fresh root> mode=<mode> out=<new .jsonl>`, hosted `weapon-fixture` build, one fresh root per mode). ELIGIBLE = a phase-17 read that is armed,
joined by sector to the real scan FIRST that just named it, whose served entry is a non-mobile placement of the roster, with a complete reply. Control credited only with `height_eligible>=1`, `height_consumed>=1` (a candidate question of that
entry carried the answered word as fact 12), `height_mismatch=0`, `height_altered=0`. Named outcomes: `sector_height_wrong_word` -> `entity_candidate.refused` ([candidate.sector_height], with `height_consumed_altered=1`);
`_withheld` -> `entity_scan.callback`; `_malformed` -> `entity_scan.answer`; `_wrong_state` -> `entity_scan.state`; `_wrong_sector` -> `entity_scan.sector_height`. A window with no eligible read is UNREACHED; no mode credits a restore.

**C. Still required OPEN:** `ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1` (the hosted-scan report counts the guards and shows the min_height after the original statement, never the consumed word; a controlled witness needs a fixture entry that does not
exist) and `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1` (needs a real server-session save and load entry, also absent).

## Checks (affected final set u1 to u11, between PRE and POST; all in [`runs/`](runs/))

`u1` fmt; `u2` strict clippy of the workspace; `u3`/`u4` strict clippy of the engine-sys features (hosted, original-solver); `u5` the whole eech-world suite; `u6` the hosted real-engine control (150); `u7` the hosted clearance boundary test;
`u8` the observer control and the 179+42 control in the original-solver profile; `u9` the observer chain, funnel and cap; `u10`/`u11` the same on i686. RED/recovery runs retained: `n2` (the new driver tests against the previous driver),
`n3` (a one-character prefix length in the first hosted-scan driver, fixed), `n6` (the clearance boundary test against the old helper). **Reuse:** the `0be1dd846` matrices (16 final checks, 190 packet files, 49 runs) are reused ONLY through
`join-vs-0be-final-post.json`: against its final POST map, no `aphavoc/` or `modules/` path and no non-test crate source changed (the changed rows are tests, the two host Lua drivers, documentation and two feature-gated translation units
that preprocess to nothing without their macro), and each changed row has its own affected check here. PRE `2026-11-07T08:43:19Z`; POST time in the freeze file (recorded by `date -u` before the capture).

## Older claims superseded (the older packets stay literal)

In `sector-height-production-source-and-checks` (and the card/STATUS text that cited it): "the test-only sector-height observer ... observes no host word" (SH10/SH11 do), the controlled-guard wording of section 8, and the restore-after-insertion
refusal of section 7. The contract (sections 7 to 11), this packet and the card carry the corrected wording.

## Limits

Source and Linux checks only; no native result, clearance or review is claimed. The scripted tests that make the campaign read or restore a sector height are boundary checks, not native campaign authority. No signalling-NaN or `-O2` claim.
Author only: no self-review, approval or whole Task49/D3 claim. All ten D3 requirements and the remaining hints, whole UPDATED/shared terrain, raw TREE/FOREST/CLOUD RNG, hierarchy/services, DCS API workarounds, natural moving, the final
current 40/all-8/final-4-inputs, CI, review, normal merge and final actual census remain. No D4/D5/TypeScript.
