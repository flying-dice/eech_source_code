# Load-order diagnostic correction (`ENV-SECTOR-HEIGHT-NATIVE-LOAD-ORDER-DIAGNOSTIC-1`) — UNREVIEWED, test-only, native UNRUN

Task EEDCS-49 (D3), 7 November 2026 (UTC). **`ENV-SECTOR-HEIGHT-NATIVE-LOAD-ORDER-DIAGNOSTIC-1` and `ENV-SECTOR-HEIGHT-NATIVE-CORE-LOAD-LIST-1` stay OPEN** until the SAME combined review of the loader fix `90e9547ee` and this correction, and a real native positive pair with NEW Windows binaries. M5, M6, `sector-height-host-move-open-facts` and all ten requirements stay OPEN. Source before: checkpoint `44afa7a6a` over the loader fix `90e9547ee`.
The three same-reviewer constraint records and the diagnosis addendum are literal in [`../sector-height-native-load-followup/`](../sector-height-native-load-followup/); the earlier diagnosis and loader-fix packets are unchanged.

## What changed (contract section 12.7; Lua, scripted tests and contract only, no C, Rust product, build or Cargo input)

The `load` verdict refused any insertion before the first restore, but a supported generated script holds `CAMPAIGN_DATA` before `SAVED_CAMPAIGN`, so the boot builds a baseline first (282933 insertions before the first restore in the actual native load; 5001 after the last): the gate could never pass. Insertion phases are now told apart by POSITION relative to the restores only (no caller attribution): before the first restore is baseline telemetry (reported, never zeroed or hidden), between two restores stays a failure, after the last restore are the saved entities and must exist (else UNREACHED). Every restored raw word, ACK, coverage and order check, the real second pack and its per-sector no-lowering/NaN relation stay strict; a fatal is always a failure. The real load remains a native failure.

## Evidence

- `runs/`: development runs `t1`..`t6` (`t1` a wording slip against an existing source guard, retained; `t4` the RED on the PREVIOUS Lua after the no-baseline load passed on it: the load with three legitimate baseline insertions was refused as a defect) and the final set `u1`..`u3` (fmt, World clippy `-D warnings`, the four World driver targets: witness driver 27 tests incl. the new baseline test, entity_authority 116, host_callbacks 82, baseline 13), all exit 0 with nonzero counts, between equal timed PRE and POST maps, on Root's explicit 9d247 image in the fresh volume.
- `join-vs-loader-fix.json`: compiled and linked inputs raw-identical to the loader fix's final map (only the Lua driver and its scripted test moved), so the loader fix's retained C checks keep their true identity and no unchanged C check is replayed. The corrected Lua needs its own literal identity; the existing M5 Root runs keep their 9cb/17f identity and remain valid only at that finite scope.
- Every `Root-height-native-load-*.json` constraint file present at the start was read and published; the freeze records the list at freeze time.

## Limits

Source and Linux checks only; the scripted collaborator checks the DRIVER, never `parsgen.c`, `unpack_session` or the Windows C runtime. Not run: native, Windows, retail, any replay of the failed load, unchanged C checks, the 179/150/221 characterisations, the 16 matrices. Nothing here is a review, approval, native clearance or closure.
