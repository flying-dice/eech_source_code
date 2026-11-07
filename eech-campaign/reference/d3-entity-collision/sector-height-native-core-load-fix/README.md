# Core load of the generated saved script: the bounded loader fix (`ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1`) — UNREVIEWED, native UNRUN for this source

Task EEDCS-49 (D3), 7 November 2026 (UTC). **`ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1` stays OPEN** until the SAME reviewer accepts this correction AND a real native positive pair (the real Core save, then the real fresh Core boot of its generated script, every restored word equal to the saved word in order) loads. M5, M6, `sector-height-host-move-open-facts` and all ten requirements stay OPEN; `ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1` and the other three witness findings stay CLOSED_SOURCE (reviews 507f49b7 and f61dac32, literal in [`../sector-height-reference-fix-reviewed/`](../sector-height-reference-fix-reviewed/)).
Source before: checkpoint `b23c06c16` over the reviewed code `9cb480fbb` (metadata `beb57c879`).

## Cause (diagnosis packet: [`../sector-height-native-core-load/`](../sector-height-native-core-load/), committed at the first checkpoint)

Root's first native M6 run (exact 17f save-feature profile, external Lua 9cb): the real Core save passed native 0; a fresh Core boot of its generated script stopped native 1 after 1748 restores at `en_list.c:356` (`Invalid list type`, sector index 0, `LIST_TYPE_FORCE`). The saved sector section decodes exactly to the save report's 1748 words (1027 non-zero) but the load stated ZERO for every sector: the unpack input was zero where the file holds data; an all-zero stream reads the session's force root as entity index 0 (the sector 0 entity). **The list fatal is a symptom.** The legacy loader sized the data with the text-mode `file_size ()`, read it with a one-element `fread (size + 1)` and ignored the unpack's overflow flag; the data holds Ctrl-Z bytes from offset 167. The exact C-runtime mechanism is NOT observed.

## The fix (product C: `aphavoc/source/ai/parser/parsgen.c`, `FILE_TAG_SAVED_CAMPAIGN` only)

Size from the binary handle opened for the data, `fread (buffer, 1, size)` with every byte required, and a `debug_fatal` naming the file and sizes on a short read or on an `unpack_session` overflow (`pack_session` sizes its data to the last partly used byte, so a correct unpack never overflows). The data still goes to the Core's own unpack: no bypass, no hand-made restore, no pack-format, host-contract, sector-height-rule or campaign-decision change.

## Evidence

- `join-vs-reference.json`: exactly ONE compiled product input changed against Root's reference map (`parsgen.c`; every other compiled or linked row, 3759 of 3760, is raw-identical); the new guard test is the only added row. **The sealed 17f Windows binaries are NOT reusable for this source**: new binaries need this new source, a new immutable plan and Root-owned native runs.
- `runs/`: development runs `r1`..`r5` (`r1` a test-setup slip, retained; `r3` the RED on the PREVIOUS loader: 2 of 3 guard tests fail for the right reasons) and the final set `s1`..`s9` (fmt, workspace and witness-feature clippy with `-D warnings`, engine-sys default 45 tests incl. the 3 loader-guard tests, hosted and original-solver controls, witness C refusals and guards, engine/dc default 194 and witness 195), all exit 0 with nonzero counts, between equal timed PRE (13:50:24Z) and POST (14:06:33Z) maps (3977 rows and extras equal), on Root's explicit 9d247 image in the fresh volume (measured in the runs, not claimed equal to the old producer).
- Readers were final before PRE (SHA-256 recorded and verified by the freeze). Literal Git bytes by `git cat-file`; the 3 Lua CRLF and 4 Core filter-only qualifications unchanged; older packets unchanged.

## Limits

A saved-campaign load cannot run on Linux (no retail data; the Windows C runtime's text-mode and partial-element behaviour is the open question): the guard (`eech-engine-sys/tests/saved_campaign_load_guard.rs`, 3 tests) pins the loader's contract and its premises and does not load a campaign; the RED/GREEN is the guard's, the validation is the native positive pair. The M4 regression pack and native profiles for this source are Root's. The `ai/faction/parser.c` duplicate of the loader is not on this boot path and was not changed. Not run: native, Windows, retail, the 179/150/221 characterisations, the 16 matrices, campaign replays. Nothing here is a review, approval, native clearance or closure.
