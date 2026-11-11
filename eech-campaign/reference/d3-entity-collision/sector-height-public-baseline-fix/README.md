# Sector-height public baseline observer: correction of the three findings of the SAME review of 9dad1b635

Task EEDCS-49 (D3), 7 November 2026 (UTC). **UNREVIEWED, source corrected, UNRUN natively.** Source before: `ba3ad01c3fbbaddbb0ec6725f211b933019ad3a5` (code `9dad1b635`). This is a bounded
correction of the necessary public baseline observer, inside the same REQUIRED sector-height extraction, before any physical sector-height move: no optional preparation, prototype or inventory
(preparation count stays 2), no production cutover, no Windows/native/campaign/war launch. The first source packet
[`../sector-height-public-baseline-source/`](../sector-height-public-baseline-source/README.md) and its freeze are kept literally and are historical, failures included.

Input: the SAME reviewer's sealed report `R:/d3-evidence/independent-sector-height-public-baseline-source-9dad-20261107.json` (SHA-256 `ad7e9a37...`, recorded in full in the freeze; copied literally here),
`CHANGES_REQUIRED`, three P2 findings, `native_launch_clearance=false`. The findings and `sector-height-host-move-open-facts` stay OPEN until that reviewer assesses this delta and a baseline has
actually run. P1 `ENV-ENTITY-POSE-LONG-RUN-MOTION-1` stays CLOSED at f734 (SAME `4de6d2e9`). The progress guard ran first: `--action fix-blocker --blocker sector-height-host-move-open-facts` and
`--action production-cutover`, both exit 0.

## What changed, finding by finding

| finding | correction |
|---|---|
| `ENV-SECTOR-HEIGHT-OBSERVER-JOIN-1` | An INSERT of a fixed entity or city opens an **observer record** under the entity's heap index (a private bookkeeping number; no production semantic identifier is allocated, no pointer or ABI is exposed). LINK, the host statement and the removal of the same placement find that record; a freed slot closes it (patches `SH12` removal in `en_list.c`, `SH13` release in `en_heap.c`). Rows now carry `rec`, `ent`, `family`. A LINK or sector-placed statement with no open record, a second INSERT of an open record, a statement of another type and a removal with no record are counted (`links_unjoined`, `stmt_unjoined`, `stmt_type_mismatch`, `double_insert`, `remove_unjoined`) and make the report incomplete; nothing is joined by family or sector. A city building's statement (a city's list, never a sector's) is counted apart (`stmt_child`). The shape is read only at the original block's own gate: `shape_state` is `valid`, `invalid` (both read) or `unobserved` (not read, never counted invalid). Fact 12 keeps its RAW word and is related to the raw word of the last FIRST read of the scan by bit equality, per sector (`fact_equal_first`, `fact_differ_first`; an offer with no FIRST is a join failure), so distinct nonzero words are not collapsed. No extra getter was added: the shape, position and box top are read where the original block reads them, the entity index is the entity's own field. |
| `ENV-SECTOR-HEIGHT-OBSERVER-COMPLETE-1` | The 64-event pack/unpack sample is removed: every word is stored in the event table, whose bound is the only bound. A word that does not fit is `pack_omitted` / `unpack_omitted` and `sho-complete` is 0. Raw words (negative, infinite, quiet NaN) are stored as bits; nothing is refused, clamped or computed. |
| `ENV-SECTOR-HEIGHT-BASELINE-RUN-1` | `tools/sector-height-baseline-windows.ps1` no longer terminates anything: a wait that ends while the native runs returns "still running" (exit 2) with the process receipt (pid, birth, command line, argument hash) already on disk and writes no completion; every output (observation, stdout, stderr, both receipts) is created NEW and atomically, and the script refuses to start if any exists. The Lua driver creates its observation with `host.create_exclusive`. The default window is the **established finite ordinary short window** (`hours=0.001`, 3.6 simulated seconds, `frame_ms=100`, `seed=1`), not two one-hour wars. |
| CI (Root's feedback `3170b6b5...`, run 42338676317) | `host_callbacks` pins the number of reference scripts that select the profile (62); the new driver is the 63rd. The semantic guard was characterised first (run `i1`, exit 101, `left: (63, 2) right: (62, 2)` AFTER every per-script structural assertion passed: the new script prepares, selects once right after, and boots like `campaign.lua`); only the inventory was updated, the guard is not weakened. |

## Controls (what they do and do not show)

- `eech-engine/tests/sector_height_observer_chain.rs`: the recorder core is driven with a known coherent sequence (two same-family records in ONE sector with interleaved statements, an unread city shape,
  an observed-invalid city, a mobile, distinct nonzero FIRST and fact-12 words, 70 pack and 70 unpack words) and read back through the REAL Rust converters (`sector_height_summary_text`,
  `sector_height_row_text`, the functions the public `Engine::sector_height_*` methods call). Then a removal, an unjoined statement, an unjoined link and a second insert are shown to be counted and
  to break completeness. The real C report of the coherent part is pinned as `eech-world/tests/data/sector-height-observer-report.txt` (its first run failed by design against an empty fixture and the
  fixture was taken from that run's stdout block: `g1`, retained).
- `eech-engine/tests/sector_height_observer_cap.rs`: the event table is filled to EXACTLY its bound (whole), one more pack word is visibly omitted and the report is incomplete, a later unpack cannot
  restore completeness, and the raw patterns are stored as bits.
- `eech-engine-sys/tests/sector_height_observer.rs`: the real-C control with the hooks compiled in still makes the same 179 expectations, and the observer refuses to call it whole (that control drives
  the message table with no en_list INSERT, so every fixed or city link is unjoined by design).
- `eech-world/tests/sector_height_baseline_driver.rs`: the REAL driver under real Lua 5.1 with the real exclusive-output function: the finite default window, order, refusal of an incomplete
  observation, non-increasing sequences, a module without the observer, arguments before any side effect, an existing output never overwritten with nothing booted, and the real C report replayed
  line for line through the driver.
- `check-sho-windows-wrapper-ast-20261107.ps1` (PowerShell AST, parse only, no process started): GREEN (`k10`, exit 0) on the new wrapper; RED (`k11`, exit 1: `$process.Kill()`, no still-running branch,
  an open that is not CreateNew, the receipt not before the wait) on the wrapper of `9dad1b635`.

These controls do NOT run the public funnel, the entity heap or the host, and not the Lua binding in `eech-dc/src/module.rs` (pinned by compilation and lint until the native run). The observer build
takes the address of two locals in `fw_move.c` and is NOT assumed result-neutral; the observer DLL's ordinary-short comparison remains a required actual gate.

## Actual checks and input identity

Linux, ONE cache owner (volume `eech-linux-d3-collision-driver-5416adb3b-20261029`, image `sha256:4a5618cb...`, no container before or after), each check ONE Cargo child through
`R:/d3-evidence/run-sector-height-baseline-direct-20261107.py` (separate stdout and stderr, exit from the runner's record, no pipeline). All records, failures included, are in [`runs/`](runs/).
Development runs `g1`..`j1` came first (retained; `g1` and `h1` and `i1` are failures at their stages). The FINAL set `k1`..`k11` ran between two actual input maps taken with
`R:/d3-evidence/sector-roster-inputmap-batched-20261106.py` and `sho-extra-identity-20261107.py` (the tools, wrapper, check and runner scripts):

- PRE `04:54:01Z`, POST `04:59:01Z`: 3965 files each, **every row equal**, fingerprint `3ebef2d4...` both; the 17 files changed from `ba3ad01c3` are the observer files and the inventory pin; no deletions; seven
  raw-versus-Git-filtered identity disagreements are the inherited ones (two `aphavoc` headers, three test Lua CRLF files, two `modules` files) and stay qualified, never normalised. Tool identity
  aggregate `f0cc14aa...` equal before and after. A first PRE (taken before Root's CI feedback was read and the inventory pin updated) is **superseded**, labelled as such, and was not used.
- No source write happened between the PRE and the POST map (git status equal).

| final check | exit |
|---|---|
| k1 `cargo fmt -p eech-engine-sys -p eech-engine -p eech-dc -p eech-world -- --check` | 0 |
| k2 strict clippy, default (feature OFF), four crates, all targets | 0 |
| k3 strict clippy `eech-engine-sys --features sector-height-check,sector-height-observer` | 0 |
| k4 strict clippy `eech-engine --features sector-height-observer` | 0 |
| k5 strict clippy `eech-dc --features sector-height-observer` | 0 |
| k6 `eech-engine` chain + cap tests | 0 (1 + 1 passed) |
| k7 `eech-engine-sys` observer control | 0 (179 expectations; 52 links, 17 packed, 17 unpacked) |
| k8 `eech-world` driver guards | 0 (8 passed) |
| k9 `eech-world --test host_callbacks` (the whole file, including the inventory test) | 0 (82 passed) |
| k10 AST check, new wrapper | 0 |
| k11 AST check, wrapper of 9dad1b635 | 1 (the expected RED) |

Not rerun (unchanged inputs): the closed 179/033 characterization, the source/API disposition, the f734 full pack, the finite CITY proofs, the in-tree `sector_height_guard` and entity guards.

## The new native plan (for Root; nothing here has been run)

1. Clean frozen checkout of the correction's code commit. Build the ordinary DLL (`tools/build-windows.sh`) and the observer DLL (`tools/build-windows-sector-height-observer.sh`) with SEPARATE
   FRESH `EECH_WIN_TARGET` volumes; record image, commit, BUILD-INFO and the SHA-256 of every file. The observer feature is the only difference.
2. Control for read-only-ness: the established ordinary short (the accepted `campaign.lua` short window, `hours=0.001`, the three accepted short files) run with the OBSERVER DLL against all three
   complete accepted short files literally, with the input manifests before and after and the actual process exit. A difference is a finding about the observer; no accepted byte is normalised.
3. ONE finite baseline on an unused verified retail root copy (never a consumed f734 root): `tools/sector-height-baseline-windows.ps1 -Scenario lebanon_retail` with its defaults (the same
   3.6-simulated-second window), started from the persistent monitor with `-WaitSeconds 0`. Keep the process and completion receipts, stdout, stderr and the observation; never terminate on a timeout.
   Georgia, or any longer window, only on Root's explicit bounded plan naming the unresolved fact; never selected by what the first observation reached. The report holds private identities and words and stays outside public safe projections.
4. Assess M1-M6 only after actual completion: joined order of INSERT, LINK, statement and removal per record; raise words against statement words; the city's `shape_state`; FIRST and fact-12 words and
   their equality counts; whether the guarded clearance is reached and with which words; whether a save or restore happens. A zero counter stays UNREACHED. By source, FIRST and fact 12 need a
   `get_line_of_sight_collision_entity` call (a campaign LOS check or a weapon step), the clearance needs a task-leader fixed-wing with terrain following, and a save needs a session pack: none is
   assured in 3.6 simulated seconds. If a needed case is UNREACHED, the next bounded actual is chosen from those source facts (the existing public `entity-authority.lua` control needs a build with both
   `weapon-fixture` and `sector-height-observer`, which this correction does not build or check), not found by extending the duration.

## Limits

Source and Linux checks only. No Windows build, native run, retail data, campaign, war or Tacview result; no physical move; the Core high-water writer, clocks, RNG and policy are unchanged; no
baseline, input, seed, frame, duration or tolerance changed; engine C at its default level 1 (no -O2 claim); no signalling-NaN, x87 control-word or exhaustive runtime-census claim. The observer's
records will be evidence about one scenario, not the supported domain. All ten D3 requirements and the remaining hints, whole UPDATED/shared terrain, raw TREE/FOREST/CLOUD RNG, needed hierarchy and
services, DCS API workarounds, natural moving, the final current 40/all-8/final-4-inputs, CI, review, normal merge and final actual census remain. No D4/D5/TypeScript. Author only: no self-review,
approval, native clearance or delivery claim.
