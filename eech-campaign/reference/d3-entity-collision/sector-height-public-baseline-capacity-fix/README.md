# Sector-height public baseline observer: the event-table capacity correction (`ENV-SECTOR-HEIGHT-BASELINE-CAPACITY-1`)

Task EEDCS-49 (D3), 7 November 2026 (UTC). **UNREVIEWED, source corrected, native UNRUN for the corrected source.** Source before: `1ef730a40ecd56c10f1a8262e0990ac71bd40102` (code `eff3b4b33`). One minimal,
storage-only correction of the necessary public baseline observer, inside the same REQUIRED sector-height extraction: no optional preparation, prototype or inventory (preparation count stays 2), no
physical move, no Windows/native/campaign/war launch here. Earlier packets, freezes, failures and maps are kept literally.

Input: the SAME reviewer's sealed report `independent-sector-height-public-baseline-eff3-actuals-20261107.json` (SHA-256 `e60093c8...`, copied literally; read in full before the first write and again before
the freeze, unchanged): `FINITE_BUILD_PROFILE_SHORT_PASS_PUBLIC_BASELINE_FAILED_CAPACITY`, one P2 finding. It confirms (and this correction does not redo): the eff3 ordinary and observer Windows builds (exit 0, 89
payloads each, feature/COFF/staged/export isolation PASS), the observer ordinary short (native and controller 0, four input checks all 1179 literal, all three COMPLETE files literal to the accepted `80fd`),
and, from the real baseline, zero join faults. The closed JOIN, COMPLETE and RUN findings stay CLOSED (the SAME `04b2f588` source review closes JOIN; `636e0d38` closes COMPLETE and RUN). Root's optional
feedback file for this correction was absent before the first write, before the PRE map and at the freeze. The progress guard ran first: `fix-blocker` for `sector-height-host-move-open-facts` and
`production-cutover`, both exit 0.

## What the actual run showed, and the only change

The finite public baseline (unchanged `hours=0.001`, `frame_ms=100`, `seed=1`; the floor yields **35 frames and 3.5 simulated seconds**, not rounded to 36) attempted **304811** event records: **262144**
stored and **42667** omitted. The report correctly latched `sho-complete 0`, emitted no whole verdict, and native, wrapper and outer exited 1. Mismatches, pending and every unjoined counter were 0. That failure is
the RED of this correction: the table was too small for the declared window; the refusal was right.

| item | before | after |
|---|---|---|
| `SHO_EVENTS` (C table, `eech_sho.c`) | 262144 | **524288** (219477 slots of headroom over the measured 304811) |
| `SHO_EVENTS_MAX` (Rust, `eech-engine`, the converter's index bound) | 262_144 | **524_288** |
| the report | `events_capacity` | adds `event_record_bytes` (the measured `sizeof` of one record: 136) |

Memory cost, honestly: 524288 x 136 B = 71,303,168 B (68.0 MiB) of zero-initialised static storage in the observer DLL (it was 34.0 MiB), **diagnostic feature only**; it is resident only as far as it is touched (the
measured demand touches about 41 MB, 39.5 MiB: 304811 x 136 B). The ordinary profile has no such table. The demand is the measured attempted record count, **not** derived from the sequence number (which also counts aggregate reads).

Unchanged and exact: total, stored and omitted accounting, the overflow latch, whole rejection, every join, shape and fact-12 rule, the incomplete-as-failure rule, the actual hook order, word copies, clocks, RNG,
policy and physical state. No truncation waiver, sampled whole, counts-only substitute or streaming framework was added; no event kind is dropped; no input, window, seed, frame or geometry changed.

## Controls (the bound, at the new value)

- `eech-engine/tests/sector_height_observer_cap.rs`: the table is filled to EXACTLY the bound (whole, nothing omitted, `events_capacity` equals `SHO_EVENTS_MAX`, so C and Rust cannot disagree); the LARGEST valid index
  (`SHO_EVENTS_MAX - 1`) reads through the real converter; the FIRST invalid index is refused by the converter and by the C report (`-1`); one more word is visibly omitted, the overflow latches and the report is
  incomplete; a later word cannot restore completeness; raw patterns are stored as bits. A compile-time assertion states that the bound exceeds the measured 304811 (the old 262144 fails it).
- The recorder chain, the funnel control and the real-C observer control (179 expectations) re-ran because the C file and the pinned report changed; the pinned real C report was regenerated (the `sho-tables` line
  gained `event_record_bytes` and the new capacity) and the Lua driver's guard replays it line for line.
- No old native replay and no artificial mutant: the actual failed run is the RED.

## Actual checks and exact input join

Linux, ONE cache owner (volume `eech-linux-d3-collision-driver-5416adb3b-20261029`, image `sha256:4a5618cb...`, no container before or after), each check ONE Cargo child through the same direct runner. All records,
failures included, are in [`runs/`](runs/): a stale-fixture run (`q2`, 101, by design), and a FIRST final set whose strict clippy of the cap control failed on my own new assertion
(`s0-superseded-r3`, exit 101, `clippy::assertions_on_constants`); that set and its PRE map are **superseded** and labelled so, the assertion became a compile-time one, format was applied, and the PRE map was retaken.
The FINAL set `t1`..`t7` ran between the final PRE (`05:51:26Z`) and POST maps of 3967 files each, **every row equal**, fingerprint `13c2b2c2...` both, tool identity equal, no source write in between; seven inherited
raw-versus-Git-filtered identity disagreements stay qualified, never normalised.

| final check | exit |
|---|---|
| t1 `cargo fmt ... -- --check` | 0 |
| t2 strict clippy, default (observer OFF), four crates, all targets | 0 |
| t3 strict clippy `eech-engine --features sector-height-funnel-check` | 0 |
| t4 strict clippy `eech-engine-sys --features sector-height-check,sector-height-observer` | 0 |
| t5 `eech-engine` funnel + chain + cap controls | 0 (1 + 1 + 1 passed) |
| t6 `eech-engine-sys` real-C observer control | 0 (179 expectations, 52 links, 17 packed, 17 unpacked) |
| t7 `eech-world` driver guards | 0 (8 passed) |

Reuse by exact changed-input join (`inputmaps-vs-eff3-compare.json`): against the eff3 final POST map exactly 5 rows changed (`eech_sho.c`, `eech-engine/src/lib.rs`, the cap control, the pinned report, the
driver test) and none was added; 3962 rows are byte-identical (the hook patches, the funnel control and C file, the Lua driver, the Windows wrapper, `host_callbacks.rs`, `eech-dc`). The closed ordinary,
179-expectation, driver, AST and full-pack results therefore stand at their own identities and were not replayed.

## The ordinary eff3 binary at its true identity (feature-off equivalence join)

`offprofile-join.json` (source-derived, structural; **not** an object comparison and not a relabel): every path that differs from `eff3b4b33` is either not compiled into the DLL (tests, the pinned report, documentation,
STATUS and the card), or `eech_sho.c`, which is wholly inside one `#ifdef EECH_SECTOR_HEIGHT_OBSERVER ... #endif` pair outside comments in BOTH versions (so it preprocesses to nothing in the ordinary profile), or
the single `#[cfg(feature = "sector-height-observer")]` constant. No path under `aphavoc/` or `modules/` and no `build/`, `Cargo.toml`, patch or other `src` file changed. So the compiled inputs of the ORDINARY profile
are unchanged in effect, and the sealed ordinary eff3 binary keeps its identity (it stays "built from `eff3b4b33`"; it is not described as built from this commit). The observer binary does change: a NEW observer binary on a fresh
cache, its profile audit (funnel-check absent), the exact ordinary-short comparison and ONE new finite baseline on a fresh verified root at the unchanged window are Root's next actual gates, after the SAME
reviewer's source clearance.

## Limits

Source and Linux checks only; no Windows build, native run, retail data, campaign, war or Tacview result here. The report of the new baseline will hold private identities and words and stays outside public
projections; only safe aggregates are published here. A partial prefix is not a whole baseline; clearance and save/restore were UNREACHED in the failed window and stay named; natural LOS offers are not a natural
weapon engagement. The observer DLL is not assumed result-neutral beyond the finite ordinary short already compared; no -O2, signalling-NaN or exhaustive-census claim. The Core high-water word, clocks, RNG and
policy are unchanged; `sector-height-host-move-open-facts` stays OPEN. All ten D3 requirements and the remaining hints, whole UPDATED/shared terrain, raw TREE/FOREST/CLOUD RNG, needed hierarchy and services, DCS API
workarounds, natural moving, the final current 40/all-8/final-4-inputs, CI, review, normal merge and final actual census remain. No D4/D5/TypeScript. Author only: no self-review, approval, native clearance or delivery claim.
