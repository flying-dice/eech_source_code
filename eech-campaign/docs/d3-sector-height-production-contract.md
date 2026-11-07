# D3: the Host-owned sector high-water height — production contract and plan

Task EEDCS-49 (D3), 7 November 2026. This is the compact, complete contract of the REQUIRED production move of the per-sector `tallest_structure_height` word from the campaign to the Host. It
revises section 6 of [d3-sector-height-contract.md](d3-sector-height-contract.md) (a *proposed* design) from the actual source and the accepted finite baseline
(`reference/d3-entity-collision/sector-height-public-baseline-reviewed/`); the characterized behaviour of that document (sections 1 to 4) is preserved, not re-derived. Written BEFORE the structural edits;
corrected at the freeze where the source taught something (see the end).

## 1. What moves, what stays

**Moves (hosted builds, `EECH_CAMPAIGN_HOST_ENTITY`)**: the one physical word per sector (a high-water float: strict `>`, never lowered, raw 32-bit save and restore, one float addition under
round-toward-zero). The Host's roster, its existing canonical world state, holds it; the campaign neither computes, holds nor answers it.

**Stays in Core**: geography, route policy and timers; the guarded fixed-wing clearance policy `min_height = max (terrain_elevation, word - terrain_elevation)`, then `max (waypoint altitude, min_height)`
(the unit mix is preserved bit for bit, not approved); the scan's offer of the word as fact 12; the session pack order. The Core *policy inputs* are those expressions; the *physical scalar* is the word.

**Original solver profiles keep the legacy body explicitly** (`collision-fixture`, `model-fixture`: no `EECH_CAMPAIGN_HOST_ENTITY`): the raise, the field read and the pack lines are the original code, so the
accepted 179-expectation real-C characterization is reused unchanged in that profile (its entry point, not its scenarios, changes: see section 8).

## 2. Writers (all of them)

| writer | original | now (hosted) |
|---|---|---|
| zero fill | `sc_seccreat.c` `memset` at sector creation, in `reinitialise_entity_system`, immediately AFTER `campaign_place_reset` (`eech_engine.c`) | the roster RESET (scan phase 6) clears every height: a sector the host holds no word for is the all-zero word |
| arrival raise | `sc_msgs.c` LINK_CHILD, `LIST_TYPE_SECTOR`, fixed sender with a valid shape: `h = position.y + box.ymax`, `if (h > held) held = h` | the Host, at the FIRST fixed-family state statement (FIXED_STATE) or city state statement (CITY_STATE) under a SECTOR key after that key was issued by INSERT; position and shape are the statement's words, the box top the Host's own scene store (the box WITH lines, the accessor the original reads). Nothing else raises. The Core block is compiled out |
| relink / sector change | a removal and a new insertion into the new list | the same: a new key, a new arrival; the old sector is never lowered |
| removal | never touches the height | the roster's removal never touches it |
| restore | `sc_pack.c` `unpack_local_sector_data`, SERVER_SESSION, last word of the record, raw | scan phase 18 RESTORE states the unpacked word to the Host; the Host stores it raw; the campaign takes the ACK (same sector, same word, bit for bit) before it goes on. Sectors are unpacked before entities (`en_sessn.c`), so later insertions can only raise it |
| the legacy field | written by all of the above | **retired** in hosted builds: never written, never read (a stale mirror invites a fallback; no mirror is kept) |

The arrival is identified by the key, not by `Never -> Held`: a city building's sector record is already merged with its list record when its first statement arrives, so a first-statement test would
miss most arrivals. Each sector entry carries one `arrived` flag, false when INSERT issues the key and true after its first statement; later statements (setters, a kill's shape, a restore) never raise.

**City.** A city has `IDENTIFY_FIXED` (source: exactly the nine types that call `overload_fixed_functions`, the eight of the fixed family plus the city). Its shape is created invalid; the generic fixed setter exists
for it, so a creator or a restore COULD give it a valid one. The accepted baseline observed 14395 reached city links, all invalid, no raise. Not a universal claim and not a blanket skip: the CITY_STATE
report gains the shape word (fact 9, which no city report used) and the Host raises for a city only when that shape is valid, with the city's own position and the same box accessor.

## 3. Readers (all of them)

| reader | original read | now |
|---|---|---|
| the getter `sc_float.c` (`FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT`) | the field | a scan-phase 17 HEIGHT question to the Host by the sector's own coordinates. The only read route for every reader but the pack, so the scan FIRST read (`CAMPAIGN_SCAN_TALLEST`), the fixed-wing clearance and the original-solver body (non-hosted) need no edit of their own |
| scan FIRST -> fact 12 of every offer | the word | the Host's word, passed on; the Host's candidate question compares fact 12 bit for bit with ITS word of the candidate's sector and refuses a difference by name (`[candidate.sector_height]`); the sector skip is decided over the Host's word |
| guarded fixed-wing clearance (`fw_move.c`: only a task leader whose guide has `terrain_follow != NONE`) | the field | the getter (above), then Core's unchanged policy expression, then the unchanged hand-off of `terrain->min_height` |
| save (`pack_local_sector_data`, SERVER_SESSION) | `raw->tallest_structure_height` | the getter's Host word, packed as the same raw 32 bits (`FLOAT_PACK_TYPE_UNPACKED`); client and browse sessions carry no height and stay as they are |
| observer hooks (test-only `sector-height-observer`) | read-only | the original raise and save/restore hooks (SH2 to SH5) are compiled out of a hosted build; the scan hooks SH10 and SH11 still capture the Host-returned FIRST word and every fact 12, reused by the `hosted-scan` scope (section 11); the clearance helper captures no word in a hosted build (`tallest=uncaptured`) |

## 4. Wire grammar (one new question pair; the next free phases)

`ENTITY_SCAN` phases 17 and 18 (`0x41880000`, `0x41900000`, classified from their bits like every phase), every fact zero but the stated ones, identity place empty.

- **17 HEIGHT**: the sector `x` and `z` in facts 12 and 13 (whole numbers; the scan's own cursor facts). Reply `height`, `p` = three words: `x`, `z` echoed, then the word.
- **18 RESTORE**: the same sector, the word raw in fact 0. Reply `restored`, `p` = `x`, `z`, the word as stored.
- Both are refused by name for a malformed fact, a sector outside the whole-number domain, a non-zero stray fact or a non-empty identity place. The campaign checks the whole reply before one word is used: the right state, exactly
  three words, the echoed sector equal to the asked one, the word as stored equal to the word stated (restore). A refusal, a missing provider or a wrong answer stops the engine with the host's reason. **No fallback to the legacy
  field and no repair of a mismatch.** The word itself is NEVER validated as finite, non-negative or non-NaN: a restored negative, infinite or quiet-NaN word is valid history and crosses exactly.
- The CITY_STATE report's new fact 9 (the city's shape: a whole scene index or the invalid index) is the only change to an accepted report.

## 5. State, ACK, lifetime, order

The Host's word lives in the roster (`SectorRoster`), keyed by the sector index pair, absent = the zero word; it ends with the roster's RESET. Order that must hold and is controlled: RESET before the new sectors; INSERT, then
the nested state statement (the arrival), then the original's list write and LINK; restore before entities. The original's per-arrival order inside `sc_msgs.c` (height, then `add_mobile_values_to_sector`) is preserved because the
Host raise happens inside the nested statement, before the list write. A statement that fails its own checks (a refusal) raises nothing: a raise only follows a statement the roster accepted.

## 6. Arithmetic, FPU, RNG, transport

- The raise is ONE C kernel (`csrc/model_step.c`, beside `city_box_expand_body`) under the same `enter_environment`/`leave_environment` guard as the other model kernels: the exception masks are checked, the rounding is set to
  toward-zero and the caller's state restored. It is the original expression (`float h = y + ymax; if (h > held) held = h;`) on raw words, not a Rust float transcription. The comparison is against the word the Host ACTUALLY
  holds: a restored negative word can be raised, a restored NaN is never exceeded, a zero or negative sum over a zero history never stores, `-0` is never stored. The store is the raw word. x86-64 (SSE) and i686 (x87) are both checked.
- No RNG draw, no clock read, no order change: the Core read path is a `memcpy` of the answered word into the float the original returned; the Core adds no floating-point operation.
- Raw transport end to end: C float field -> `memcpy` -> request/reply words -> `eech-engine` converter -> `eech-dc` Lua bridge (hex words, never `tonumber`) -> host Lua -> `model_host.rs` (integer words) -> roster; and back.
  Every converter keeps the 32 bits (a controlled test per layer carries the 13 measured save patterns plus a restored negative, `+-inf` and quiet NaN). Signalling-NaN and engine `-O2` behaviour stay unmeasured.

## 7. Controls (built together)

Positive: the Host word equals the original's for every recorded raise sequence (the 13 patterns, strictness, no-lowering, rounding toward zero, a negative-restored raise, a NaN-restored no-raise); the Core consumer uses the
Host word: in the real-engine control the getter and the pack read the scripted provider's word, and in the real campaign the scan's FIRST read is carried back as fact 12 (section 11 shows it natively). Missing (a provider that
refuses or withholds), wrong (another sector, a short/long/non-matching reply, a restore stored under another word, a fact 12 that is not the Host's word), stale (a word that survives a boot's RESET: a read after it is the zero word):
each a named refusal with the legacy field never consulted. **A restore (phase 18) is accepted by the provider whenever it is well formed, including while placements are live**: the provider deliberately has no insertion-time
rule for it. That a load restores the sectors before it inserts entities is the campaign's own session order (`en_sessn.c`, pinned by the source guard), not a provider rule, and no refusal for a restore after an insertion is claimed.
The scripted tests that make the campaign's getter read or restore (`entity_authority.rs`, the dialogue check, the real-engine control's scripted provider) are BOUNDARY CHECKS of the contract and the drivers: they manufacture phases 17
and 18 themselves and are not native campaign authority. The ordinary short, a `weapon-fixture` run and the baseline are Root's native gates after the SAME source clearance (section 11).

## 8. Disposition of what is not natively reached (M5, M6)

M5 (guarded clearance) and M6 (save/restore) were UNREACHED in the accepted baseline and stay so. **No control reaches the guarded fixed-wing execution**: `fixed_wing_movement_waypoint` is a static function of `fw_move.c`; its
statement text and hand-off are pinned by a source guard and its read is the Host's word through the getter, and nothing more is claimed. The real-engine control exercises the real getter and the real pack and unpack over a
SCRIPTED provider (hosted and original-solver profiles, x86-64 and i686): that is not a native Core server save/load, and a Host-only raw restore (phase 18 asked by a test or a driver) is not a Core save/load either.
`ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1` and `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1` stay required OPEN inside the height extraction until actual proof; the missing prerequisite of each is a named fixture entry
(a task-leader fixed wing whose guide follows terrain; a real server-session save and load), which does not exist in the public bridge and is not invented here.

## 9. Native plan (Root-owned, not run here)

See section 11: the executable plan (commands, features, modes, row schema, eligibility, expected outcomes). Native execution remains Root's, after the SAME clearance.

## 10. What the source taught (corrections to this plan at the freeze) and what stays open

- **No mirror.** The plan allowed a legacy mirror with ACK rules. The source showed a mirror can only go stale (the raise is the host's), so the legacy field is *retired* in hosted builds: every access of the member is compiled
  only where the original entity solver is kept, pinned by a census guard (`tests/sector_height_guard.rs`), not left as a field that could be read as a fallback.
- **The getter is the one read.** `sc_float.c` asks the host, so the scan's FIRST read (`mb_hostscan.h`, textually unchanged), the original-solver body, the fixed-wing clearance (`fw_move.c`, textually unchanged) and the pack
  (`sc_pack.c`, through the getter) need no edit of their own. No question of the dialogue was added for the scan: fact 12 is the host's word passed on and the host compares its bits.
- **Fact 12 is a raw word.** Both candidate classifiers (the integer one and its floating mirror) demanded every fact be finite; they now exempt fact 12 only, with the exact words pinned (`tests/sector_height_host.rs`).
- **The key, not the record, detects the arrival** (records merge at a city building's LINK before its sector placement's statement); one `arrived` flag per sector entry.
- **The city's shape** is stated (CITY_STATE fact 9). The accepted baseline's 14395 reached cities were all invalid; whether any real city ever arrives with a valid shape stays unknown from source and is decided by the stated word.
- **The accepted original characterization keeps its scenarios and changes its entry point:** the 179 expectations run unchanged with `--features sector-height-check,collision-fixture` (221 with the 42 differential expectations), the hosted
  profile adds 150 of its own. The test-only sector-height observer instruments the ORIGINAL campaign-held word (its raise and save/restore hooks sit in the compiled-out branches of a hosted build): it still builds and its controls pass in the original-solver
  profile, where its original-link hooks (SH2/SH3 raise, SH4/SH5 save/restore) live. In a hosted build those four hooks are compiled out, but the scan hooks SH10 and SH11 STILL capture the Host-returned FIRST word and every fact 12: the
  `hosted-scan` scope (section 11) reuses exactly that, compared by sector with the accepted original report at the same window, and nothing else; the observer is not globally unusable, and a whole original-link report comparison is invalid
  in a hosted build. Its clearance helper no longer reads the retired member in a hosted build: the record says `tallest=uncaptured` and the consumed word is never re-queried.
- **Limits stated, not hidden.** The host does not know the world map, so a sector it never saw answers the zero word (the campaign only asks for sectors it holds); the word is never validated; signalling-NaN payloads and engine `-O2`
  are unmeasured; the host's raise expression is the original's text compiled at the engine's default level and compared with the original response on x86-64 and i686 over the 21 domain cases under both rounding modes.
- **Named blockers, OPEN and required** (not closed by this source): `ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1` (the guarded fixed-wing expression lives in a static function of `fw_move.c` whose guards no control reaches: its text and its
  hand-off are pinned and its read is the host's through the getter, but no real-C or native run reaches the guards; a controlled public witness, a task-leader fixed wing whose guide follows terrain, is Root's to take and is never natural reach)
  and `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1` (the real pack and unpack run in the real-C control over a scripted provider in both profiles, x86-64 and i686; a native server-session save and restore through the campaign is not reached).

## 11. The executable public proof plan (Root runs it; native execution is not done here)

Source-bound: all of it needs a build of the production source (or its correction), the established finite window (`lebanon_retail`, `hours=0.001`, `frame_ms=100`, `seed=1`: 35 frames and 3.5 s as the accepted baseline floored; never
rounded, extended or searched), a fresh verified root per run and an immutable output directory. Nothing is tuned to reach a green: **UNREACHED, NOT_COMPARABLE, DIFFERENT and FAILED are distinct and each keeps its partial output.**

**A. Scoped baseline comparison (M4 at the accepted window).** Build the HOSTED observer DLL as for the accepted baseline (`tools/build-windows-sector-height-observer.sh`, feature `sector-height-observer`, no funnel check), then

`powershell -File tools/sector-height-baseline-windows.ps1 -Root <fresh root> -Bin <observer bin> -Out <new dir> -Scenario lebanon_retail -Hours 0.001 -FrameMs 100 -Seed 1 -Scope hosted-scan -Accepted <accepted report>`

(the accepted report is the retained f01 observation, read and never copied; the wrapper never terminates a process). The driver `lua/sector-height-baseline.lua` writes the identity line with `scope=hosted-scan`, the observer's summary
lines, the clearance guard records (`tallest=uncaptured`) and the per-sector `sho-scan` rows, then `sho-compare` and `sho-verdict scope=hosted-scan scan_whole=<0|1> status=<...>`. **Compared, by sector (sx, sz) and at the same window
identity:** `reads`, `first_word`, `last_word`, `changes`, `fact_offers`, `fact_equal_first`, `fact_differ_first`, `fact_first_word`, `fact_last_word`, `fact_changes` and the totals of the `sho-scan` summary line (the accepted
evidence reused at its true scope: 63 FIRST reads over 2 sectors and 27615 fact-12 offers, each equal to its FIRST). **Not compared:** sequence numbers, INSERT/LINK/statement joins, shape states, the original raise, the original
save and restore, the clearance word (not captured) and the accepted 304811-event original-LINK diagnostics. Credited only with exit 0 and `status=EQUAL_AT_SCOPE` (every compared field equal); `DIFFERENT` lists each field
(`sho-compare-diff`) and fails; `NOT_COMPARABLE` (another window or an unreadable report) and `UNREACHED` (no scan FIRST) are not passes. A hosted run that is equal shows the Host's word reached the campaign's scan at every
read of the window as the original's did; it does not show the Host's raise at every one of the 77531 arrivals, and it is not a native save, a clearance or a natural weapon engagement.

**B. Authority and refusal witnesses (the public driver, modes of `lua/entity-authority.lua`; hosted `weapon-fixture` build, one fresh root per mode).**
`eech-world.exe <literal lua/entity-authority.lua> root=<fresh root> mode=<mode> out=<new .jsonl>` (the established command of the public authority witnesses). Eligibility, credited only for a REAL consuming campaign request: a phase-17
read that is ARMED (after the case launch), joined to the real scan FIRST that just named that very sector (the cursor words equal, once), whose served entry is a NON-MOBILE placement of the Host's roster (the fixed family or a city),
with a complete reply (the state, three patterns, no name). The healthy `sector_height_control` is credited only when a candidate question of that served entry then carried the answered word back as fact 12 (consumption), no word
differed (`height_mismatch=0`) and nothing was altered; the changed-input and refusal modes each alter ONE eligible read and credit the NAMED outcome:

| mode | the host's answer at the first eligible read | expected stop (`failure_marker`) | what else is credited |
|---|---|---|---|
| `sector_height_control` | unchanged | none (completes) | `height_eligible>=1`, `height_consumed>=1`, `height_mismatch=0`, `height_altered=0` |
| `sector_height_wrong_word` | the word one unit away (a changed Host input) | `entity_candidate.refused` | the campaign CARRIED the changed word (`height_consumed_altered=1`) and the provider's `[candidate.sector_height]` refusal is the only fault |
| `sector_height_withheld` | the provider raises | `entity_scan.callback` | no other fault |
| `sector_height_malformed` | two patterns | `entity_scan.answer` | no other fault |
| `sector_height_wrong_state` | `restored` for a read | `entity_scan.state` | no other fault |
| `sector_height_wrong_sector` | the sector echoed one unit away | `entity_scan.sector_height` | no other fault |

Row schema (the `authority` row): `height_reads`, `height_scan_joined`, `height_eligible`, `height_other_callers` (the clearance or the pack), `height_altered`, `height_consumed`, `height_consumed_altered`, `height_mismatch`,
`sector_height_total`, `sector_height_answered`, `sector_restore_total`, `sector_restore_answered`, `failure_marker`, `failure_line`, `request_faults`, `reply_faults`, `first_fault`. A window with no eligible read is
`UNREACHED` (the verdict names the counts); a run that failed first, or stopped at another marker, is `FAILED`, never an absent prerequisite; nothing is read from the engine after its fatal stop. **No mode credits a restore
(phase 18):** a real Core save/load is not reachable in this window, a Host-only raw restore is not one, and `sector_restore_total` is only reported.

**C. Still required OPEN (named, inside the height extraction, not deferred away).** `ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1`: the hosted-scan report counts the guards (`sho-clearance leader_reached`, `follow_reached`) and shows the
min_height after the original statement when they are reached, never the consumed word; a controlled witness that REACHES them needs a named fixture entry that does not exist yet. `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1`:
needs a real Core server-session save and load entry, also not present. Neither is closed by A or B.
