# Sector high-water height: the two proof-guard false passes (`ENV-SECTOR-HEIGHT-SCOPED-REPORT-COMPLETE-1`, `ENV-SECTOR-HEIGHT-REFUSAL-INTERVENTION-1`)

Task EEDCS-49 (D3), 7 November 2026 (UTC). **UNREVIEWED; native UNRUN; both findings stay OPEN until the SAME reviewer closes them** (and `ENV-SECTOR-HEIGHT-PUBLIC-PROOF-PLAN-1`, partially addressed, stays OPEN with them).
Source before: the checkpoint `fe1bf6723` (both findings registered OPEN in the card) over the previous correction `5aae2256f02ca3d79e762d71c9b79c20dc34856b` and the production source
`0be1dd8465d9bf38da69cd3b303206cf5a64574d`. This is a required bounded correction inside the height extraction, not a preparation batch (the count stays 2). It is **test-only** (diagnostic Lua, driver tests,
prose): no production solver, protocol, kernel or compiled campaign input changed. No Windows, native, campaign or war launch happened here; the Root-prepared `3f` snapshot was not touched.

## Inputs (literal copies here; `literal-bytes-check.json` compares each to its ACTUAL Git index blob bytes)

- `independent-sector-height-proofplan-5aae-source-20261107.json` (SHA-256 `409a2622...aa27b`): the SAME review of `5aae2256f`, `CHANGES_REQUIRED`, `native_launch_clearance=false`, no new production defect; the two P2 findings are under `decision.findings`.
- `sector-height-proofplan-3f-CI-readback-20261107.json` (`bb46f014...`): the exact `3f` CI, all nine jobs success. It arrived **after the reviewer's seal**, so the report's own 8/9 observation stays historical. Green CI is no physical or native acceptance.
- `sector-height-proofplan-Root-binding-5aae-20261107.json` (`6ec510ae...`): Root's binding of `5aae`. No Root feedback file newer than the production feedback (06:40Z) existed before the PRE map.
- Byte qualifications carried unchanged: exactly **3 inherited Lua CRLF raw-versus-Git exceptions** and **4 Core filter-only qualifications** (raw bytes equal the Git blob, only the filtered label differs); no file was normalised.

## The two false passes and the correction

| finding | before (RED retained) | after |
|---|---|---|
| `...SCOPED-REPORT-COMPLETE-1` | the scoped comparison parsed identity/totals/sector maps without checking the accepted report's terminal verdict or the declared scan rows; a truncated, duplicated or missing-field report compared `EQUAL_AT_SCOPE` with `scan_whole=1` (`g1`) | `sector-height-baseline.lua` validates **both sides as a whole scoped scan** before comparing: accepted original = exactly one identity, one `sho-scan` summary, one `sho-tables`, and the terminal `sho-verdict whole=1 ... sectors=N` last; both = exact fields/domains, unique contiguous row indices from 0, unique sector keys, declared sector count, `firsts=reads`, no fact-12 offer before a FIRST, `scan_out_of_range=0`, rows summing to the totals, words consistent with changes. The hosted side is not asked for `sho-complete`, events, raise or session. New statuses `INVALID_ACCEPTED` / `INVALID_HOSTED` (never EQUAL_AT_SCOPE, SCAN_ONLY or `scan_whole=1`); `UNREACHED` only for a clean whole scan with no read; unreadable accepted stays `NOT_COMPARABLE` |
| `...REFUSAL-INTERVENTION-1` | an expected-marker fatal with `height_altered=0` bypassed both the other-marker failure and the clean UNREACHED branches, and the generic alteration sum included the configured forced-hit input: an unrelated fatal bearing the mode's marker after a configured hit and before any eligible read was credited (`g4`, `g7`) | `entity-authority.lua` credits a height refusal only with **exactly the one eligible intervention (`height_altered=1`)**, independent of the secondary counters and of marker equality; any fatal before it is a FAILURE (the mode's own marker included); only a clean run with no eligible read is UNREACHED; `wrong_word` keeps its same-entry fact-12 consumption and exact candidate refusal |

## Controls (positive first, single fault from a coherent positive)

- Baseline driver: equal-at-scope credited with no original-link record; differing valid report refused with every difference written; another window / unreadable report `NOT_COMPARABLE`, only a clean absence `UNREACHED`;
  **29 single-fault damaged-accepted** and **17 single-fault damaged-live-hosted** cases (truncated before the verdict, verdict not last, duplicated or omitted row, duplicate sector key, non-contiguous index, wrong count, totals/word
  inconsistency, malformed token, malformed live row or total, and so on): none compares equal, none is `SCAN_ONLY`, none has `scan_whole=1`.
- Authority driver: per mode (the five refusal modes) the coherent positive, then an unrelated fatal bearing the mode's own marker injected after the valid configured secondary hit with no eligible read must FAIL with
  `height_altered=0` and never read UNREACHED; the four read-time modes also with the eligible read still to come (late read positive, then a fatal between hit and read); a clean absence is UNREACHED; a stray request fault is a failure
  (never credited, never UNREACHED); a different marker after the real intervention names both markers.
- **Real accepted report:** `check-real-accepted-shape-20261107.py` (an independent Python port of the same rules; the Lua loader itself is exercised by the scripted tests) over the retained private accepted original report
  (`runs/shp3-real-accepted-sha-20261107.txt`): 1 identity, 1 totals, 1 tables, 1 verdict, 2 rows; VALID: 2 sectors, 63 reads, 27615 fact offers, window `lebanon_retail`, seed 1, frame_ms 100, hours 0.001, frames 35.
  Only safe aggregates are recorded; the report itself, retail data and geometry are not copied.

## Checks (all in [`runs/`](runs/); the failures are retained as evidence)

RED / recovery (nonzero by design): `g1` the baseline matrices against the previous comparator; `g3` the first late-read control (incoherent for `wrong_word`: a stale fact 12 is refused before the read, so the control was restructured);
`g4` the injection controls against the `5aae2256f` verdict; `g5` an intermediate failure after the verdict fix (the exactly-once check was placed before the clean UNREACHED branch, and the late scripted read made the candidate refuse; both fixed);
`g7` the previous driver verdict credited an unrelated fatal bearing `sector_height_withheld`'s marker. GREEN: `g2`, `g6`, `g8` (cargo fmt applied, which reformatted the test sources; the final set below re-ran every test on the formatted source).
**Final set** between the PRE (`2026-11-07T09:18:33Z`) and POST (`2026-11-07T09:19:05Z`, recorded by `date -u` before each capture) maps, which are equal: `v1` `cargo fmt --check`; `v2` strict clippy of `eech-world` (all targets);
`v3` `entity_authority` + `sector_height_baseline_driver` (116 + 13 passed); `v4` `host_callbacks` (82 passed); `v5` the real-report shape check. All exit 0.
**No C, x87, legacy-workspace or whole-World rerun**: no C source, protocol, kernel, non-test crate source or Cargo file changed; the `0be1dd846` matrices (16 final checks, 190 files, 49 runs) and the `5aae2256f` affected runs (11 checks,
19 runs) are reused ONLY through `join-vs-5aae-final-post.json`, which compares this POST map to the `5aae` final POST map and classifies every changed row.

## Audit-reader timing (stated, not smoothed)

All readers of this batch (the freeze generator, the join, the extra-identity script, the final-set script and the real-report check) were final before the PRE identity capture. In the previous (`5aae`) batch the join reader
`shp2-join-20261107.py` was corrected at `08:54:55Z`, **after** its POST: its old PRE/POST extra-identity files list the 4836-byte version (SHA-256 prefix `2dd277db`), while the freeze's final reader of that batch is
4852 bytes (prefix `ca27dda8`). That was a post-processing audit reader, not a test or source input (no `u1`-`u11` or source drift), no replay was demanded, and the 13 extras of that batch are therefore **not** all current-literal. Old packets stay literal.

## Lessons recorded

`docs/governance/D3-FIVE-WHYS-20261102.md` gains two lines: an expected error string needs the actual eligible intervention; scoped whole data needs unique complete rows and a terminal verdict on both sides (a green prefix is not whole).

## Limits

Source and Linux checks only; no native result, clearance or review is claimed. The scripted tests are boundary checks of the diagnostics, not native campaign authority; the real accepted report is checked by an independent port, not
by the Lua loader. `ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1` and `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1` stay required OPEN; no baseline, input, seed, frame, duration or tolerance changed. Author only: no self-review,
approval, finding closure or whole Task49/D3 claim. All ten D3 requirements and the remaining hints, whole UPDATED/shared terrain, raw TREE/FOREST/CLOUD RNG, hierarchy/services, DCS API workarounds, natural moving, the final
current 40/all-8/final-4-inputs, CI, review, normal merge and final actual census remain. No D4/D5/TypeScript.
