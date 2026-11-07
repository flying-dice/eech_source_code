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

M5 (guarded clearance) and M6 (save/restore) were UNREACHED in the accepted baseline and in the finite native gates of `1b89` (which did not claim them). **Neither is closed by source, a scripted test or a census guard.** The real-engine
control exercises the real getter and the real pack and unpack over a SCRIPTED provider (hosted and original-solver profiles, x86-64 and i686): that is not a native Core server save/load, and a Host-only raw restore (phase 18 asked by a
test or a driver) is not a Core save/load either; `fixed_wing_movement_waypoint` is a static function of `fw_move.c` and its text and hand-off are pinned by a source guard, which is not the branch running.
`ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1` and `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1` stay required OPEN inside the height extraction until actual native proof. **Section 12 is their executable witness**: a host-owned driver
(`lua/sector-height-witness.lua`) and, for the save, one feature-gated fixture METHOD (`session:sector_height_witness_save`, `eech-dc/sector-height-witness`) that makes the campaign's OWN `store_session` run; the facts that fixed those
designs (and why a created leader or a hand-made world was not built) are recorded there, and the native runs are Root's, after the SAME source review.

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
- **Named blockers, OPEN and required** (not closed by this source): `ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1` (the guarded fixed-wing expression lives in a static function of `fw_move.c`: its text and its hand-off are pinned and its read
  is the host's through the getter; no native run has reached the guards; the witness driver of section 12 observes it at the host's own boundary at a declared window) and `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1` (the real pack
  and unpack run in the real-C control over a scripted provider in both profiles, x86-64 and i686; a native server-session save and restore through the campaign is not reached; section 12 makes the campaign's own save run and observes its own load).

## 11. The executable public proof plan (Root runs it; native execution is not done here)

Source-bound: all of it needs a build of the production source (or its correction), the established finite window (`lebanon_retail`, `hours=0.001`, `frame_ms=100`, `seed=1`: 35 frames and 3.5 s as the accepted baseline floored; never
rounded, extended or searched), a fresh verified root per run and an immutable output directory. Nothing is tuned to reach a green: **UNREACHED, NOT_COMPARABLE, DIFFERENT and FAILED are distinct and each keeps its partial output.**

**A. Scoped baseline comparison (M4 at the accepted window).** Build the HOSTED observer DLL as for the accepted baseline (`tools/build-windows-sector-height-observer.sh`, feature `sector-height-observer`, no funnel check), then

`powershell -File tools/sector-height-baseline-windows.ps1 -Root <fresh root> -Bin <observer bin> -Out <new dir> -Scenario lebanon_retail -Hours 0.001 -FrameMs 100 -Seed 1 -Scope hosted-scan -Accepted <accepted report>`

(the accepted report is the retained f01 observation, read and never copied; the wrapper never terminates a process). The driver `lua/sector-height-baseline.lua` writes the identity line with `scope=hosted-scan`, the observer's summary
lines, the clearance guard records (`tallest=uncaptured`) and the per-sector `sho-scan` rows, then `sho-compare` and `sho-verdict scope=hosted-scan scan_whole=<0|1> status=<...>`. **Compared, by sector (sx, sz) and at the same window
identity:** `reads`, `first_word`, `last_word`, `changes`, `fact_offers`, `fact_equal_first`, `fact_differ_first`, `fact_first_word`, `fact_last_word`, `fact_changes` and the totals of the `sho-scan` summary line (the accepted
evidence reused at its true scope: 63 FIRST reads over 2 sectors and 27615 fact-12 offers, each equal to its FIRST). **Both sides are validated as a WHOLE scoped scan before anything is compared:** the accepted original must carry
exactly one identity, one `sho-scan` summary, one `sho-tables` and one terminal `sho-verdict whole=1 ... sectors=N` (the real f01 report does: 2 sectors, 63 reads, 27615 offers, checked by an independent port of the same rules); both
sides need the exact scoped fields and domains, unique contiguous row indices, unique sector keys, the declared sector count and totals and words consistent with the rows; the hosted side is NOT asked for the original-link
`sho-complete=1`, events, raise or session. Damaged input is `INVALID_ACCEPTED` or `INVALID_HOSTED`, always a non-pass. `scan_whole` describes only the live hosted scan: it can remain 1 with `INVALID_ACCEPTED` when the live scan is valid, but that comparison status/exit still fails; `INVALID_HOSTED` cannot claim a whole live scan. **Not compared:** sequence numbers, INSERT/LINK/statement joins, shape states, the original raise, the original
save and restore, the clearance word (not captured) and the accepted 304811-event original-LINK diagnostics. Credited only with exit 0 and `status=EQUAL_AT_SCOPE` (every compared field equal); `DIFFERENT` lists each field
(`sho-compare-diff`) and fails; `NOT_COMPARABLE` (another window or an unreadable report), `INVALID_ACCEPTED`, `INVALID_HOSTED` and `UNREACHED` (a clean, whole scan with no FIRST) are not passes. A hosted run that is equal shows the Host's word reached the campaign's scan at every
read of the window as the original's did; it does not show the Host's raise at every one of the 77531 arrivals, and it is not a native save, a clearance or a natural weapon engagement.

**B. Authority and refusal witnesses (the public driver, modes of `lua/entity-authority.lua`; hosted `weapon-fixture` build, one fresh root per mode).**
`eech-world.exe <literal lua/entity-authority.lua> root=<fresh root> mode=<mode> out=<new .jsonl>` (the established command of the public authority witnesses). Eligibility, credited only for a REAL consuming campaign request: a phase-17
read that is ARMED (after the case launch), joined to the real scan FIRST that just named that very sector (the cursor words equal, once), whose served entry is a NON-MOBILE placement of the Host's roster (the fixed family or a city),
with a complete reply (the state, three patterns, no name). The healthy `sector_height_control` is credited only when a candidate question of that served entry then carried the answered word back as fact 12 (consumption), no word
differed (`height_mismatch=0`) and nothing was altered; the changed-input and refusal modes each alter ONE eligible read and credit the NAMED outcome, **only with exactly that one intervention (`height_altered=1`), independent of the configured secondary hit and of marker
equality: any fatal before it is FAILURE (the mode's own marker included) and only a clean run with no eligible read is UNREACHED**:

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
min_height after the original statement when they are reached, never the consumed word. `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1`: no real Core server-session save and load has been made natively. Neither is closed by A or B; their
executable witnesses are section 12 (source frozen for the SAME review, native UNRUN).

## 12. The two required witnesses (M5 guarded clearance, M6 native save and load): what the source taught, the executable plan, the limits

Source-bound and Root-run after the SAME source review; nothing here was run natively and none of it is claimed natively. **UNREACHED, FAILURE and PASS are distinct, UNREACHED is never a pass, and a witness is never tuned by what a run reached.**
The driver is `lua/sector-height-witness.lua` (one bounded process per mode, a fresh verified root each, its report created exclusively and never overwritten); its scripted tests (`tests/sector_height_witness_driver.rs`) are boundary checks of
the DRIVER only; the C refusals of the save method are run for real without a boot (`eech-engine-sys/tests/sector_height_witness.rs`) and the source guards (`sector_height_witness_guard.rs`) pin that the fixture exists only behind its feature.

### 12.1 M5, the guarded clearance (`ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1`)

**What the source taught.** The clearance is `fixed_wing_movement_waypoint` (static, `fw_move.c`), run for a TASK LEADER whose guide's `terrain_follow_mode` is not NONE. That mode is read from the static `guide_database[sub_type]` (`gd_int.c`, `gd_dbase.c`: NONE for the routed, altitude, landing and landed guides; CRUISE_ALTITUDE for the direct and virtual navigation guides and some attack-move guides; ATTACK_ALTITUDE for others) and has no setter; the leader is the guide's own task-leader pointer; the host step needs the member's host identity, which the roster holds
only for members the campaign inserted into a real group. The static function is reached through `basic_fixed_wing_movement` (and its absolute variant) and, with no terrain out-parameter, through the public wrapper
`fixed_wing_movement_get_waypoint_position`. In a hosted build each task leader's step therefore reads the sector word TWICE with no other host callback between (the wrapper, whose answer is discarded, then the navigation, which
hands `min_height` to the host), asks `fixed_wing_lookahead`, and sends `fixed_wing_move` carrying `fixed_wing_terrain` (`min_height`, the guide position, the sample point, the terrain elevation answered there). **No test entry was built that creates a
leader**: the campaign has no creation path for an airborne task leader with a guide except its own task and guide machinery, so building one by hand would be an override of campaign decisions disguised as natural behaviour, a hand-made
world cannot reach the host hand-off (the look-ahead needs the roster's host identity), and re-implementing the guide's position to hand-make one would copy the campaign into the test. The branch is therefore observed at the HOST'S OWN boundary on a
DECLARED window: the accepted regression-pack identity (`lebanon_retail`, 3 simulated hours, 100 ms frames, seed 1, the pack's scenario), named here before any launch and never chosen by what a run reached. A leader that never meets the branch in that window
is UNREACHED.

**Eligibility, derived from the campaign's own call structure and pinned by the census guard of `tests/sector_height_guard.rs`** (the readers of the word are the scan, this clearance and the pack): a phase 17 question that arrives while no scan enumeration
is open (a scan opens at phase 0 and closes at its `done_*` reply) and during no save is the guarded clearance asking. A WINDOW is a maximal run of such questions with no other host callback between; the HAND-OFF is the next `fixed_wing_move` carrying
`fixed_wing_terrain` after a `fixed_wing_lookahead`. No C hook is added: the consumed word is the host's own answer, the hand-off is the step the host receives, and its physical result is the reference host's answer.

| mode | what it does | credited |
|---|---|---|
| `clearance_original` | the same window in the ORIGINAL-solver profile (`tools/build-windows-collision-fixture.sh`): the campaign consumes its legacy word and the host is never asked | the first hand-off reported; FAILURE if the host WAS asked (not that profile) |
| `clearance_control` | hosted, nothing altered, stops at the first window followed by a hand-off; `original=<report>` compares its first hand-off with the original profile's, field for field | one such window; `min_height >= max(elevation, word - elevation)` in single precision; equal to the original's row when given |
| `clearance_changed_word` | `window=<N>` from the control: every word read in window N is answered 100000.0 (0x47c35000) | the hand-off equals the single precision value of 100000.0 - elevation exactly; with `control_min`/`control_answer_y` the hand-off and the host's answered height are compared (`physical=changed\|unchanged`, stated either way) |
| `clearance_withheld` / `_malformed` / `_wrong_state` / `_wrong_sector` | `window=<N>`: ONE fault at the first read of window N | the engine stops at `entity_scan.callback` / `.answer` / `.state` / `.sector_height`, and only after that intervention; a fatal before it, no fatal, or another marker is FAILURE; window N never reached is UNREACHED |

Commands (each a fresh root; the ordinary hosted DLL for every row but the first):
`eech-world.exe lua/sector-height-witness.lua root=<fresh root> out=<new report> mode=<mode> hours=3 [window=N] [original=<clearance_original report>] [control_min=0x.. control_answer_y=0x..]`
Order: `clearance_original` (original-solver build), `clearance_control original=...`, `clearance_changed_word window=<the control's window>`, then the four refusals with the same window. The campaign is deterministic at a fixed seed, so window N of the control
is the same step in every run; a run that reaches a different step shows it in its own counts. The word term can be absent from a hand-off (an altitude or terrain term may dominate): the changed word is chosen to dominate and a hand-off that does not follow it is reported UNREACHED,
never credited.

### 12.2 M6, the native server-session save and the campaign's own load (`ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1`)

**The real lifecycle (traced from source before any code).** SAVE: `store_session` (`uisession.c`, the in-game save) writes the saved script `<data_path>\<campaign_directory>\<name>.S<ext>` (the extension's letters shifted as `store_session` does: `.Sch` for `.chc`; its
`FILE_TAG_SAVED_CAMPAIGN` names the data) and the data `<name>.sav`: the session version int, then `pack_session(buffer, &size, PACK_MODE_SERVER_SESSION)` into a 1 MiB buffer which `store_session` itself doubles and re-packs while `pack_session` reports an overflow.
`pack_session` packs the sectors FIRST (`pack_local_sector_data`: each sector's record ends with the 32 raw bits of the word the sector getter returns, i.e. the host's phase 17 answer), then the pylons, segments, bridges, the session, the forces, every
entity, the keysites, regen, city buildings, scenics, sites, groups and the databases. A rotation of `_bakN` backups happens only when BOTH files exist (it renames and deletes). LOAD: the engine's boot (`eech_engine_boot`) resets the host's roster
(`campaign_place_reset`, `campaign_roster_reset`: a RESET clears the host's words) BEFORE `create_campaign` parses the script; reaching `FILE_TAG_SAVED_CAMPAIGN` in `parse_campaign_script` (`parsgen.c`) reads `<script name>.sav`, checks the version int and calls
`unpack_session(buffer, size, PACK_MODE_SERVER_SESSION)`, which unpacks the sectors first: each sector states its saved word RAW to the host (phase 18, `campaign_model_sector_height_restore`: the word in fact 0, the host holding exactly those bits and
acknowledging them), then creates the entities, whose sector-list insertions raise the host's word by the strict-greater rule. **An unpack onto live entities is not a load and is never attempted**: the load is a fresh boot of the saved script in its own process.

**The fixture method** (feature `eech-dc/sector-height-witness`; with it off no definition, entry or export exists): `session:sector_height_witness_save(name)` calls the original `store_session` once and nothing else (`csrc/eech_hw_witness.c`). It refuses a name outside 1 to
24 of `A-Z a-z 0-9 _ -`, a session that is not the booted campaign's server session of the campaign game type, and any name whose saved script OR saved data exists (it never overwrites, rotates or deletes: `store_session` would); afterwards the data file must exist and begin with the
session version. A fatal inside `store_session` or the pack stops the engine through the entry's own fatal path, as any campaign fatal does. It reads and writes no sector word itself.

**Seeds** are the host's explicit raw HISTORIES, an INPUT: before the save the driver states them to its OWN provider as a restore (a host whose world holds a history), and the pack's phase 17 reads then carry them. Negative, infinite and quiet-NaN words are valid history and are
preserved bit for bit (no finite classifier, no canonicalisation); a SIGNALLING NaN is refused (unmeasured), as is engine `-O2`.

| mode | what it does | credited |
|---|---|---|
| `save` | `name=` `seeds=sx:sz:0xWORD,...` (`frames=` default 0): boot, seed the provider, the campaign saves; every word the pack asked for is recorded (`sh-save-word`) | the save line (`sh-witness-save name= bytes= version=`), at least one sector, every seeded sector packed with its seed's bits (a seed outside the map is FAILURE) |
| `save_withheld` | the host raises at the FIRST pack read | the engine stops at `entity_scan.callback` after it; a fatal before it is FAILURE |
| `load` | `campaign=<saved script>` `expect=<the save report>` `name=<a new name>`: the campaign boots the saved script, then saves again | every restore states exactly the word the save packed for that sector (none extra, none missing, none twice), acknowledged bit for bit, every restore BEFORE the first entity insertion (restore-before-insert) and at least one insertion after (else UNREACHED), and no sector's word read back by the second pack is LOWER than its restored word (a NaN unchanged; a history raised only upward by a greater arrival): `relation=held\|raised` per sector |
| `load_withheld` / `load_wrong_sector` / `load_wrong_state` / `load_stale_word` | ONE fault at the FIRST restore of the campaign's own boot | stops at `entity_scan.callback` / `.sector_height` (another sector) / `.state` / `.sector_height` (the host stored a word that is not the one stated), after that intervention only |

Order: `save` (seeds on sectors inside the map: a negative word, minus infinity, a quiet NaN, plus the zero history that needs no seed) on the witness build (`tools/build-windows-sector-height-witness.sh`), then each `load*` mode on the same build and a COPY of the root that holds the saved files
(a boot of the saved script reads them). A load that fails the first fault is FAILURE; absent insertions after the restores are UNREACHED.

### 12.3 What this does NOT establish

Nothing in section 12 has been run natively. The scripted tests prove the driver credits only what it intervened on and refuses every single-fault variant of its controls; they never reach `fw_move.c`, `store_session`, `pack_session` or `unpack_session`. The C refusals test proves the method's
refusals and that a refused call touches no file, not that a save works. A passing native `clearance_*` run would show the branch asked the host and used its word at ONE declared window for the first eligible leader, not every leader or every window; a passing `save`/`load` pair shows the
campaign's own save and load of the words the driver seeded and the campaign's arrivals produced on one map, not natural save histories, a signalling NaN, engine `-O2` or a server session of another game type. Neither witness is an acceptance of the height extraction, Task49, D3 or delivery.
