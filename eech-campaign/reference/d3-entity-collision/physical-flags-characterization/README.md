# Physical flags: the per-entity UPDATED / MOVED / ROTATED word (characterisation) — UNREVIEWED, no production flag moves, native UNRUN

Task EEDCS-49 (D3), 7 November 2026 (UTC). **`ENV-PHYSICAL-FLAGS-RAWWORD-CONTRACT-1` is OPEN** (a named required correctness blocker; `blocks_cutover` for the flag move) until the SAME review decides the first extraction step. M5/M6 finite closures, the whole height extraction, all ten requirements, Task49 and the Epic are unchanged and OPEN as recorded. Preparation count stays 2 (the named-blocker exception; no scope deletion).
The full contract is `eech-campaign/docs/d3-physical-flags-contract.md`; Root's 15:50 feedback (`Root-physical-flags-feedback-20261107-1550.json`) was read before the freeze and shaped it.

## What was learned (original code, verified in source and by a real-C control)

- Three GLOBAL bit arrays (16384 words each), keyed by the entity HEAP INDEX alone; the "per-entity word" is the array word holding the entity's bit. The getter is `array[word] &= mask`: it returns the entity's MASK (INT_MIN for bit 31), clears EVERY OTHER entity's bit in the word and keeps its own. The reset of UPDATED is a quarter-memset (bytes vs words): indices 0..131071 are reset, 131072.. never; MOVED/ROTATED only behind the Tacview frame reset. A reused slot inherits the bit. All preserved as compatibility facts, not repaired, not declared outside observable behaviour.
- UPDATED is physical (collision pass-through, fixed-wing pass-through, supply descriptor); MOVED/ROTATED are Tacview-observational and stay in scope; no campaign decision reads the words. Census exact (`tests/physical_flags_guard.rs`).
- A source-derived, UNCONFIRMED, SEPARATE defect candidate: the supply descriptor copies the raw mask into a field the Rust side validates as 0/1. Not fixed, not normalised, reachability unknown.

## Evidence

- Real-C control `csrc/eech_physical_flags_check.c` (feature `physical-flags-check`; 199 expectations over the REAL mobile getter/setter via the real dispatch tables, the real flag arrays and the real `update_client_server_entities` over an empty update list; hand-made entities of the engine's own struct; nothing else scripted) and the 3-test census guard. Final set `f1`..`f5` (fmt check, engine-sys clippy with and without the feature, engine-sys default tests, the control + guard), all exit 0 with nonzero counts, between equal timed PRE (15:52:33Z) and POST maps, in the sole fresh volume `eech-linux-d3-physical-flags-9d247-20261107` on the retained 9d247 image (cargo 1.99.1, GCC 12.2 inside; not the Root 3a3 Windows producer).
- `join-vs-reference.json`: every compiled-input change is a test-only feature-gated ADDITION (one new translation unit wholly inside `#ifdef EECH_PHYSICAL_FLAGS_CHECK`, added manifest lines, one added define block); no original-campaign C and no other compiled input changed. Retained: `p1` (a compile slip), `p2` (6 sign-comparison slips of the control's own expectations), `p3` (the placeholder count pin); a first PRE captured before the 15:50 feedback is retained as SUPERSEDED.

## The bounded next production responsibility and what remains REQUIRED

The whole UPDATED/MOVED/ROTATED store with its set, destructive masked read and reset as one unit, Core keeping scheduling and every decision; a simple has-been-stepped truth service plus Core mirror is NOT demonstrated sufficient and is not proposed. **Remaining required blocker, recorded not met: the real immediate-consumer control** (the smallest identified is `response_to_check_mobile_reached_guide`, `fw_msgs.c`, through the real message table); the source census does not meet that gate and no consumer body ran. Unresolved: where the word's whole store lives and how its heap-index key crosses the boundary without a second store.

Not run: native, Windows, retail, any consumer body, the Tacview reset path, the setter with FALSE, -O2, the 179/150/221 characterisations, the 16 matrices, campaign replays. Root's 764 full-40 job and every Root artifact were untouched. Nothing here is a review, approval or acceptance.
