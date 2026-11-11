# Sector-height public baseline observer: the residual JOIN correction (second correction of `ENV-SECTOR-HEIGHT-OBSERVER-JOIN-1`)

Task EEDCS-49 (D3), 7 November 2026 (UTC). **UNREVIEWED, source corrected, UNRUN natively.** Source before: `4901b6880a3126ca6ffe1cc99aa9ad9b61950438` (code `d74ca2e1b`). One bounded correction of the
necessary public baseline observer, inside the same REQUIRED sector-height extraction, before any physical move: no optional preparation, prototype or inventory (preparation count stays 2), no
production cutover, no Windows/native/campaign/war launch. The earlier packets ([`../sector-height-public-baseline-source/`](../sector-height-public-baseline-source/README.md),
[`../sector-height-public-baseline-fix/`](../sector-height-public-baseline-fix/README.md)) and their freezes, failures and maps are kept literally and are historical.

Input: the SAME reviewer's sealed report `independent-sector-height-public-baseline-correction-d74-20261107.json` (SHA-256 `636e0d38...`, copied literally here): `CHANGES_REQUIRED`,
`ENV-SECTOR-HEIGHT-OBSERVER-JOIN-1` remains the only open finding; `-COMPLETE-1` and `-BASELINE-RUN-1` are CLOSED by that report and were not reopened. Root's optional feedback file for this
correction was ABSENT before the first write and at the PRE map; it appeared at 05:20:25Z, while the final checks were starting, and was read in full at the freeze (a timing miss against the instruction to read it before the final checks, retained): it is a CI readback of the preceding head `4901b6880`, all nine jobs success, with no new scope; the new code needs its own CI. The progress guard ran first: `fix-blocker` for the JOIN finding and for
`sector-height-host-move-open-facts`, and `production-cutover`, all exit 0. P1 `ENV-ENTITY-POSE-LONG-RUN-MOTION-1` stays CLOSED at f734 (SAME `4de6d2e9`). The finding and
`sector-height-host-move-open-facts` stay OPEN until the SAME reviewer assesses this delta and a baseline has actually run.

## The residual defects, observed on the real funnel order first

`en_list.c` calls `campaign_place_insert (en)`; in `mb_hostplacemap.h` that call is `place_insert`, which registers the placement, declares it and then calls `place_fixed_state (en)` BEFORE it
returns; LINK_CHILD follows. The observer's insert hook (SH1) stood AFTER the call, so the first fixed or city statement met no record: `stmt_unjoined` and the mismatch latch made a baseline
report impossible to call whole. And `core_statement` classified EVERY city-building statement as child-only: a city building is listed in its city (`cb_creat.c`) and then in its sector, the placement
map prefers the sector key as soon as it exists, and it states under both keys, so the sector-stage statement of a building has a sector record that must join.

| defect | correction |
|---|---|
| the record opened after the nested call | **SH1 stands BEFORE `campaign_place_insert`** (the hook only records; no production call, getter or order moves). |
| building classified by family | SH8 and SH9 pass `place_key_of_slot [slot][0] != 0` (the placement map's own choice of key). A statement under the SECTOR key joins the sector record, a building's second stage included; a statement under a city-list key is the building's first stage, counted (`stmt_child`); any other family stating under a child key is a failure (`stmt_child_unexpected`). A statement that finds no record is still `stmt_unjoined`; a statement of another type still `stmt_type_mismatch`; a second insert, a removal with no record and a freed slot with an open record are as before. |

Kept: the private bounded record identity (heap index, no production identifier or pointer), the explicit `unobserved` shape state, raw FIRST and fact-12 words and their per-sector relation, the complete-or-incomplete
rule for pack and unpack, and every strict counter. Nothing was weakened or made to ignore a record.

## The control: the real placement map in the staged order (`eech-engine/tests/sector_height_funnel.rs`)

It runs the real STAGED `mb_hostplacemap.h` (so its staged hooks are the ones compiled) over scripted entities and a scripted host (`csrc/eech_sho_funnel_check.c`, new test-only feature
`sector-height-funnel-check`; `eech_sho.c` gates its entity-reading hook bodies out under it), the real recorder core, the C report and the real Rust converters. **The order is not hard-coded**: the test
applies the real patch table (`build/patches.rs`, included as a module) to the real in-tree `en_list.c`, `sc_msgs.c`, `en_heap.c` and `mb_hostplacemap.h`, asserts the relations the control relies on and
passes the insert order to the C control, so the control and the code cannot disagree about placement. Scenarios: a fixed placement (INSERT, nested statement, LINK), the city, a city building in BOTH stages,
two same-family records of ONE sector with interleaved restated statements, a relink, a slot freed while listed, and genuinely unjoined placements (an insert the observer never saw and a restatement) that are
still refused and break completeness. Rows assert the per-entity callback order and the record each carries.

REDs (genuine, not setup failures; each reverts ONE defect, the file restored byte for byte and its hash verified):
- `m2`, exit 101: the `d74ca2e1b` placement of SH1 (after the call): `stmt_unjoined=6`, report incomplete.
- `m3`, exit 101: the `d74ca2e1b` family-only building rule: `stmt_child=3` where the real stages give 1.
Then GREEN on the corrected source (`m1`, and the final `p6`). The stale fixture run (`m4`) failed by design when the summary gained `stmt_child_unexpected`; the fixture was regenerated from its stdout block.

The recorder-only chain control keeps its preferred sequence but is now explicitly the recorder's, and gained the child-domain failure; it is not the public funnel. The pinned real C report was regenerated
(one counter added to the summary line only) and the Lua driver's guard still replays it unchanged.

## Actual checks and exact input join

Linux, ONE cache owner (volume `eech-linux-d3-collision-driver-5416adb3b-20261029`, image `sha256:4a5618cb...`, no container before or after), each check ONE Cargo child through
`R:/d3-evidence/run-sector-height-baseline-direct-20261107.py`. All records, failures included, are in [`runs/`](runs/). Format first (`n1` exit 1, `n2` applied), then the code was frozen and the
actual PRE map taken (`05:19:25Z`), the final checks `p1`..`p9` ran, and the POST map (`05:25:17Z`): 3967 files each, **every row equal**, fingerprint `ed55208c...` both, tool and script identity equal;
no source write in between. Seven inherited raw-versus-Git-filtered identity disagreements stay qualified, never normalised.

| final check | exit |
|---|---|
| p1 `cargo fmt ... -- --check` (four crates) | 0 |
| p2 strict clippy, default (observer OFF), four crates, all targets | 0 |
| p3 strict clippy `eech-engine-sys --features sector-height-funnel-check` | 0 |
| p4 strict clippy `eech-engine --features sector-height-funnel-check` | 0 |
| p5 strict clippy `eech-dc --features sector-height-observer` (the binding compiles) | 0 |
| p6 `eech-engine` funnel control (NEW) | 0 (1 passed) |
| p7 `eech-engine` chain + cap controls | 0 (1 + 1 passed) |
| p8 `eech-engine-sys` real-C observer control | 0 (179 expectations, 52 links, 17 packed, 17 unpacked) |
| p9 `eech-world` driver guards (replaying the regenerated real C report) | 0 (8 passed) |

Reuse is by exact changed-input join (`inputmaps-vs-d74-compare.json`): against the d74 final POST map, 9 rows changed (`eech-engine-sys` Cargo.toml, `build/main.rs`, `build/patches.rs`, `csrc/eech_sho.c`,
`csrc/eech_sho_hooks.h`, `src/lib.rs`, `eech-engine` Cargo.toml, the chain control, the pinned report) and 2 rows were added (the funnel C control and test); 3956 rows are byte-identical, including the Lua
driver, `host_callbacks.rs`, `eech-dc/src/module.rs`, the cap control, the real-C observer control file and the Windows wrapper, whose PowerShell AST results (`k10` green, `k11` red on the old wrapper) and the
`host_callbacks` run (82 passed, the inventory pin 63) therefore stand at their own identities and were not rerun. The COMPLETE and RUN closures of the review are untouched by this delta except that
`p7` and `p8` re-ran the controls whose hook code changed.

## Limits and what is still required

Source and Linux checks only; the funnel control runs the real placement map and the real staged patch relations but NOT the real `en_list.c`, the entity heap, the roster provider or the host, and not the
`eech-dc` Lua binding or the public entries' lock/state check (compiled and linted; the binding compiled proof is reused, native unrun). No Windows build, native run, retail data, campaign, war or Tacview
result; no physical move; the Core high-water writer, clocks, RNG and policy are unchanged; no baseline, input, seed, frame, duration or tolerance changed; engine C at its default level 1 (no -O2 claim); no
signalling-NaN, x87 control-word or exhaustive-census claim; the observer DLL is not assumed result-neutral (the observer-DLL ordinary-short comparison remains a required actual gate). The new native plan of the
previous packet stands unchanged (ordinary and observer builds on separate fresh volumes; the observer-DLL ordinary short against the three accepted short files; ONE finite baseline in the established
3.6-simulated-second window from the persistent monitor, never killed on a timeout; M1-M6 assessed only after completion, UNREACHED stays named).
All ten D3 requirements and the remaining hints, whole UPDATED/shared terrain, raw TREE/FOREST/CLOUD RNG, needed hierarchy and services, DCS API workarounds, natural moving, the final current
40/all-8/final-4-inputs, CI, review, normal merge and final actual census remain. No D4/D5/TypeScript. Author only: no self-review, approval, native clearance or delivery claim.
