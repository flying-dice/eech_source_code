# M5: the first host-boundary cut (proposal for the lead's decision)

**Status: proposal and characterisation (commit `f7a3b756`).** That commit changed no product code, build input, baseline or expectation. The lead later authorised structural slice 1a (§0), recorded in [`m5-loss-structural.md`](m5-loss-structural.md), and a separate host-identity/durable-roster prerequisite, recorded in [`m5-host-identities.md`](m5-host-identities.md). This proposal remains a proposal; neither implemented prerequisite completes a goal criterion. The sole authority for scope is the Confluence goal [EECH M5 Goal: EECH_DC into DCS World](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337) (read live on 2026-10-21, version 1). The working plan, [Confluence 12484650](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/12484650) (v3 when this proposal was written, v6 when slice 1a was recorded), is explicitly incomplete. Jira EEDCS-11 owns the boundary, the host-driven events and Lebanon preservation.

- **Branch:** `codex/m5-host-boundary`, worktree `.claude/worktrees/m5-host-boundary`, based on the remote integration head `163e9158`, the merge of PR #79.
- **Product tree:** #79 changes `eech-campaign/reference/pr59-candidate-identity/` and the CI workflow `.github/workflows/eech-campaign.yml` (`git diff --name-only 65892de1 163e9158`). Neither is a product build input (`aphavoc/`, `modules/`, `eech-campaign/crates/`, `Cargo.lock`, `tools/Dockerfile`, `tools/build-windows.sh`). So every source citation below is at the unchanged product tree `65892de1`, and so is the characterised candidate build (`eech_dc.dll` `a99d7a11…`, #79's `BUILD-INFO.txt`).
- **Inputs used:**
  - the call map at `a4cd7eea` (`codex/m5-dcs-call-map`, under independent review). Its local revision `ab830a04` resolves review findings F1–F6: 53 interactions, and a corrected globals inventory of 95 items;
  - #74 at `31204591` (partial SUPPLY ownership), read when this proposal was written. #74 has since merged (head `f153ffe8`, merge `6f222b1b`), with documentation changes only;
  - #77 at `ab279d44` and #78 at `cf11eb82` (arrival characterisation, not merged).

This record is criteria 2–4 progress planning. **It completes no criterion.**

## 0. Lead decisions on this proposal (Starscream, recorded 2026-10-21)

The source is the lead's scope decision, recorded on Jira [EEDCS-11, comment 10445](https://flying-dice.atlassian.net/browse/EEDCS-11?focusedCommentId=10445).

- **Accepted as sequencing:** loss first. The reasons are the natural Lebanon loss and the verified shape of DCS's loss events.
- **Authorised now:** slice 1a only, the structural move. Its record is [`m5-loss-structural.md`](m5-loss-structural.md).
- **Not approved:**
  - the 1b step-boundary timing shift, the internal queue and its draining;
  - deferring wreck removal;
  - any baseline replacement;
  - any presumed eventual M4 divergence.
- **Identity:** host identities must eventually be issued or confirmed by the actual host, and must be opaque to the campaign. Private pointers stay private.
- **Guards:** the proposed runtime guard and symbol checks (§4) are hypotheses for the later host-routing cut. They are not evidence at 1a.
- **No criterion is accepted by the first cut.**

The lead subsequently approved the host-issued identity and durable-roster
prerequisite with the constraints recorded in
[`m5-host-identities.md`](m5-host-identities.md): identity belongs to a physical
incarnation and survives membership transfer, the issuer is an invoked callable
outside the DLL, legacy observation and ACMI identities remain unchanged, and
the work must not be described as host-owned physical authority or loss
causality.

## 1. Answer: can host physics be linked only into EECH_WORLD?

**Not at `65892de1`.** It is achievable only as the end state of the whole-campaign cut, not as a first step. The campaign and the reference world are one program over one entity heap. The coupling, from source:

| Coupling | Evidence at `65892de1` | What has to change before a link split |
|---|---|---|
| **The campaign holds world entities by pointer.** Group → member is the `LIST_TYPE_MEMBER` list of world mobiles, and task dependents, takeoff queues and movement dependents are entity lists too. (Movement dependents turned out to be physical carriage, not a campaign list: see [`m5-loss-structural.md`](m5-loss-structural.md).) | Unlinked by world code: `hc_dstry.c:505` (kill) and `:159` (wreck removal), and the same in `fw_`, `rv_`, `aa_`, `ps_` and `sh_dstry.c` | The campaign keeps its own member records keyed by a host identity. World code never edits a campaign list. |
| **The campaign reads physical state through those pointers,** dispatched per entity type by the attribute tables. | Token counts in the 159 campaign files: `VEC3D_TYPE_POSITION` 329 sites in 50 files, `INT_TYPE_ALIVE` 76 in 20, `FLOAT_TYPE_FUEL_SUPPLY_LEVEL` 33, `FLOAT_TYPE_VELOCITY` 28 (raw `grep -w`; the receiver may be a campaign entity, so these are upper bounds). Call map: 330 attribute tokens, 194 mapped to world rows. | Reads of a member's physical state come from the last host observation. A campaign-side member type can serve the same attributes, which leaves the campaign C unchanged at the read sites. |
| **World code applies campaign policy and bookkeeping.** | Loss: §3 below. Arrival criteria inside the world's message handlers: `hc_msgs.c:529-700` (leader radius, altitude, heading, landed, rotor spin-up, player skip); `fw_msgs.c:390`; `vh_msgs.c:339`. Call map: W→C 121 functions, 25 static message types. | Each piece of policy moves into campaign C behind a host observation. |
| **World code is steered by campaign objects every frame.** | The guide's formation offset and criteria are read by the flight model (`hc_move.c:1145`; `get_guide_criteria_valid`, 14 W sites); call map I6 and G4 | The host receives routes and tasks through the contract (G4), and the world follows them itself. |
| **The campaign calls world and environment code directly.** | Call map `reach.py`: C→W 38 functions, C→E 25 (for example `get_3d_terrain_elevation`, 26 sites), C→X 10 | Each call becomes a host query (terrain, line of sight) or campaign-owned data (type tables G22, object bounds G14). |
| **Shared runtime state.** | One random LCG for both halves (`random.h:96`, call map I37); 95 non-campaign globals used by the campaign (call map `ab830a04`, local and unreviewed; corrected from the 31 first counted at `a4cd7eea`, which missed comma-list and macro-mediated externs); one `system_delta_time` (`eech_engine.c:283-284`) | Separate streams (G18) and globals per binary, with time given to the campaign explicitly |
| **The entity system's defaults are silent.** | A missing attribute handler returns 0.0, or a default vector (`en_float.c:1572-1611`, `en_vec3d.c:229-240`); an unhandled message returns FALSE (`en_msgs.c:218-221`). | If world types were simply left out of the campaign build, the campaign would read zeros and stall with no error. A split must make every handler the contract does not supply **fatal** (§4). |

**With pointers kept private:** once the rows above are done, two binaries in one process work on Windows. Each PE module has its own copy of the framework's globals and its own heap. ELF needs `-Bsymbolic` or hidden visibility (inferred, not tested). No `entity *` may cross the boundary. Only identities, observations and requests cross. EECH_WORLD then links the W and E classes (332 + 73 files) plus a framework copy, and `eech_dc.dll` links C plus F. Until then, a link split would leave dangling or duplicate entity pointers.

## 2. Characterisation of the public path today (red for criteria 2 and 3)

Run with the #59 candidate build (`eech_dc.dll` `a99d7a1118008f73…`, `eech-world.exe` `13cc2a6f…`, `lua.dll` `d0c799fe…`). These hashes were checked against #79's `BUILD-INFO.txt`, and the build was copied to `E:/eech-runs/m5-host-boundary/bin`. Inputs:
- a private copy of the accepted Lebanon root, checked before and after with `tools/input-manifest.py --compare reference/lebanon-3h/inputs.json`: IDENTICAL, 1,179 files, `121a7890…`;
- retail Lebanon (map5, `LEBANON.CHC`), seed 1, 100 ms frames, 1,800 simulated seconds;
- Windows 11, 2026-10-20 23:06 UTC.

The probe is [`reference/m5-host-boundary/probe.lua`](../reference/m5-host-boundary/probe.lua). It runs through `eech-world.exe` → `require ("eech_dc")`.

```sh
eech-world.exe probe.lua root=<copy> seconds=1800 observe=1 out=A.jsonl diag_at=1580,1590,1795 2> A.stderr   # 3 min 12 s
eech-world.exe probe.lua root=<copy> seconds=1800 observe=0 out=B.jsonl diag_at=1580,1590,1795 2> B.stderr   # 2 min 03 s
```

| Observation | Result | Label |
|---|---|---|
| The module's exports | `boot`, `name`, `prepare_installation`. Indexing `event`, `report`, `observe`, `on_event` or `host` on the engine gives `nil`. The host can give the engine only boot configuration and milliseconds (`A.jsonl`). | [R-run] |
| **No host.** Booting needs nothing from a host world | `dc.boot` succeeds and the war runs. The DLL never asks the host for anything. | [R-run] |
| **The natural M3 loss is reproduced** (run A, observing every 10 frames as `campaign.lua` does) | Mi-17 87568 of Wolfpack (group 87567) dies at **1,584.66 s**, and the group drops to 87581. This matches the accepted `M3.supply_loss_reproduced`. | [R-run] |
| **The campaign books the loss inside the DLL** | Diagnostics at 1,580.06 s against 1,590.06 s ([`loss-delta.txt`](../reference/m5-host-boundary/loss-delta.txt)): red `ARMED_HELICOPTER` 92 → 91; `REGEN_HELICOPTER` 3 → 4; Wolfpack `members=2` → `1`. | [R-run] |
| **Withholding has no effect** (run B: the host never reads or reports anything) | The campaign diagnostics at 1,580, 1,590 and 1,795 s are **byte-identical** to run A's (SHA-256 `70b59f20…`, `5a4c40aa…`, `65f82373…`). B books the same loss. | [R-run] |

**What fails today, with no interface name assumed:**
- **Criterion 3** needs this: when the host withholds a loss report, the loss is not booked, so the campaign state differs between A and B at 1,590 s. Today it does not differ.
- **Criterion 2** needs this: a boot without a host fails loudly. Today it succeeds.

The red is meaningful because the inputs are the accepted ones and the loss is natural. Nothing in it is manufactured or test-only. It is retained as the A/B pair and the hashes above.

## 3. Recommended first cut: a member loss becomes a host observation

**Why loss first, not arrival:**
- **DCS supports it.** DCS has a verified loss event in the mission environment: `S_EVENT_DEAD`, `CRASH`, `UNIT_LOST` and `KILL` (call map D4 [R-dcs], `DCS:Scripts/rtsScripting.lua:16,21`). DCS has no verified "waypoint reached" signal (D27 [U]).
- **One-way and minimal.** The host reports one fact, an identity that died. Arrival needs the campaign to publish its targets and criteria first (G7), and that commits a route and waypoint contract early.
- **The consequences are pure campaign bookkeeping.** They are identical in shape across six world files, and campaign C already implements them all: `remove_from_force_info` `force.c:249`; `add_entity_to_regen_queue` `rg_updt.c:656`; `release_mobile_entity_landing_locks` `landing.c:867`; the group's `response_to_mobile_killed` `gp_msgs.c:1281`, registered at `:1419`.
- **An accepted red scenario already exists:** §2.

Arrival (#77, #78) follows as the second cut, using the same identity and observation semantics (§5).

### Current call sites: the campaign consequences inside world `kill_local`

| File (`entity/mobile/…`) | `kill_local` | Force info | Regen queue | Task dependents completed | Landing locks | Unlink `MEMBER` (kill / wreck) | Group losses | Task losses | `MOBILE_KILLED` | Objectives check |
|---|---|---|---|---|---|---|---|---|---|---|
| `aircraft/helicop/hc_dstry.c` | 316 | 358 | 410 | 422-440 | 448 | 505 / 159 | 594 | 608 | 623 | 639 |
| `aircraft/fixwing/fw_dstry.c` | 306 | 347 | 368 | 376-394 | 402 | 427 / 160 | 502 | 516 | 531 | — |
| `vehicle/routed/rv_dstry.c` | 301 | 345 | 369 | 381-399 | 407 | 434 / 155 | 499 | 513 | 528 | — |
| `vehicle/anti_air/aa_dstry.c` | 295 (+ a second path, 600-657) | 339 | — | 365-383 | 391 | 412, 624 / 151 | 470 | — | 484, 657 | — |
| `vehicle/person/ps_dstry.c` | 293 | — | — | 378-396 (no reason argument) | commented out, 404 | 425 / 149 | 358 (before the unlink) | — | 471 | — |
| `vehicle/ship/sh_dstry.c` | 301 | 348 | — | 364-380 | 388 | 425 / 155 | 490 | 504 | 519 | — |

The uneven coverage is behaviour to preserve, not to tidy up: regen only for helicopters, fixed wing and routed vehicles; the objectives check only for helicopters; no force info for people. Unifying it would be an intentional rule change, to classify separately. The world parts in between (effects, sounds, unlinking world lists, `INT_TYPE_ALIVE`, `DEATH_TIMER`) stay in the world.

### The bounded implementation, in three reviewable slices

| Slice | Change | Expected M4 result | Proves |
|---|---|---|---|
| **1a. Structural** | Move the campaign consequences out of the six `kill_local` bodies (and the campaign unlinks out of wreck removal) into campaign C, with one function per phase. **Pre:** force info, regen, task dependents, landing locks. **Post:** campaign unlinks, group and task losses, `MOBILE_KILLED`, objectives. Each takes the member's kind, and the world calls them at exactly the current points, so the per-type differences and the ordering are kept. **As authorised and built:** a two-phase wrapper is not used. Instead there is one helper per contiguous campaign chunk, called at that chunk's original position, with no kind argument; the per-type differences stay at the call sites. See [`m5-loss-structural.md`](m5-loss-structural.md). | **IDENTICAL** to `windows-retail v1` (pack runtime tier, both maps). §2's A/B diagnostics equal the retained hashes. | Pure move: a characterisation test, with no red manufactured |
| **1b. Semantic: the step boundary** | The world records "member died" (identity, model time) instead of calling the campaign. The campaign applies the 1a consequences at the start of its next step, in report order. For now the queue is drained inside the DLL, so this slice isolates the timing change from the host routing. Campaign lists stay linked until then; **wreck removal waits while a campaign record still binds the entity.** | Unmeasured. Any difference (consequences up to one frame later could also shift the random stream) is to be measured and classified through the M4 change record. **Not approved:** see §0. Claims `M3.supply_loss`, `M1.*.mechanics` and `M2.tacview_observations` must still hold. **No baseline is re-recorded without approval.** | The one semantic change, measured on its own |
| **1c. Host authority** | Remove the internal drain. The world's death record leaves only through the public module, and the host's Lua forwards it back into the campaign's inbound entry. That entry is the only caller of the 1a consequences (guard, §4). EECH_WORLD's `campaign.lua` forwards every event. Boot fails without a declared host. | Equal to the 1b candidate when the host forwards everything. The DCS-shaped scenarios in the table below differ as specified. | Criterion 3 for loss; the criterion 2 contract and no-host failure for this interaction |

The names of the outbound and inbound calls are left to review. The shape is DCS's: an event table `{id = DEAD, initiator = <host identity>, time = <model time>}`, delivered between campaign steps, as `world.addEventHandler` plus a timer-stepped campaign would deliver it (D3, D19).

### Contract semantics to decide (proposal)

- **Host identity.** An opaque string issued by the host for each physical unit, never reused within a host session. The campaign stores it in its member record and never parses it.
  - **EECH_WORLD:** the world issues `"<entity index>.<creation serial>"` at creation. The index is reused (`eech_observe.c:49`; #77 saw waypoint generations up to 567), so the serial is required.
  - **DCS:** the unit name the campaign supplies in its spawn request (G3, `EECH_<kind>_<index>_<generation>`), with `Unit:getID` recorded. Whether DCS names are unique and whether ids are reused is [U]; the DCS-bind probe settles it.
- **Duplicate.** Idempotent per identity: the first of DEAD, CRASH or UNIT_LOST applies, and later ones are counted, with no consequence. KILL adds only the aggressor. (Whether DCS raises several of these per loss is [U]; call map §5.)
- **Late or stale.** A report for an identity whose record is gone is counted and ignored. It never resolves to a reused index. Examples: the campaign already removed the member itself (G26, `group_kill_all_members` `group.c:1021` on capture), or the identity was never bound.
- **Missing.** A loss that is never reported leaves the member alive in the campaign; the campaign infers nothing. In DCS the reconciliation is an `isExist` sweep (G8), a later slice. In EECH_WORLD the harness counts world deaths not forwarded, so a withheld run is visible in the evidence, not an error.
- **Unknown identity** (a player or mission-editor unit). Reported as foreign and counted, with no consequence.
- **Arrival and loss.** No arrival is accepted for an identity after its loss. A loss ends the record's observation stream.
- **Order and time.** Reports are applied in host order at the start of the next campaign step. The campaign uses its own clock for consequences and records the host's time.
- **No host.** Boot without a declared host is an `eech:` error. Calling the inbound entry before boot, or after a fatal error, is an `eech:` error as well.

### Checks for 1c (by affected behaviour)

| Check | Input | Pass |
|---|---|---|
| Forwarded | The §2 probe, the host forwarding everything | The loss delta at 1,590 s equals 1b's |
| **Withheld** (criterion 3) | The host drops 87568's report | At 1,590 s Wolfpack has `members=2`, red `ARMED_HELICOPTER` 92 and `REGEN_HELICOPTER` 3. Any loss consequence without a report fails. This is also the detector for hidden channels (below). |
| Delayed | Forwarded 30 s late | The consequences appear in the first step after the forward |
| Duplicate | Forwarded twice | Equal to forwarded |
| Unknown or stale | A forged identity, or 87568's identity after it is unbound | Counted; no state change |
| No host | Boot without a host declaration | An `eech:` error naming the missing host |
| Regression | M4 pack runtime tier, Georgia and Lebanon; `loss-chain-check.py` | 1b's classified differences only |

**A limit stated in advance.** In one shared heap, campaign code can still notice a death by reading `INT_TYPE_ALIVE` or position through a member pointer before the host reports it. The withheld check measures that directly for the loss bookkeeping. Other decisions that read physical state through pointers belong to later interactions, and only the link split (§1) removes the channel completely.

## 4. Smallest mechanical enforcement against hidden physical fallback

Matching names in a regex ledger does not stop a fallback. These three checks do, and each rests on what the compiler or the runtime actually does:

1. **The host-context guard (runtime).** The campaign's loss-consequence functions call `debug_fatal` unless they are running inside the inbound host-report entry. That entry sets a flag; nothing else does. Any internal path, old or new, renamed or reached through a function table, fails as `eech: EECH fatal error …` the first time it runs. This is the existing poisoning path (`eech-engine/src/lib.rs:41-53`).
2. **Partition symbol closure (link time).** Classify the compiled objects by the call map's `sources.txt` classes. The world partition's undefined symbols (`nm -u`) must not include the campaign's loss entry points. The reviewed list of remaining W→C and C→W symbols may only shrink. This uses the toolchain's symbol tables, so macros, statics and renames cannot hide a call.
3. **Fatal defaults for the contract's entity types (split stage).** When world types leave the campaign build, every attribute and message cell for those types that the host contract does not supply gets a fatal handler, not the silent 0.0 or FALSE (§1, last row).

## 5. How the whole-campaign cut closes the rest

Each later cut has the same pattern: characterise, structural move, separate semantic change, host routing, then the guard and the symbol list shrink. The rows are the call map's.

| Order | Cut | Call map rows | Host contract (DCS) |
|---|---|---|---|
| 1 | **Loss** (this proposal), then damage and return-to-base, and campaign-initiated kills | I14, I15, I17, I11, I43 | DEAD, CRASH, UNIT_LOST, KILL, HIT; `Group:destroy` (G8, G26) |
| 2 | **Arrival:** the four SUPPLY types (leader-only, #77), then last-to-reach cohesion (#78), then typed LAND and TAKEOFF arrivals | I12, I13, I50, I35 | Host geometry polling against published criteria (G7); LAND and TAKEOFF events (G10) |
| 3 | **Spawn and identity:** members created by host request; campaign member records | I1, I3, I4, I33, I48 | `coalition.addGroup`; names (G1, G3, G22) |
| 4 | **Movement and tasking:** routes pushed instead of the guide pulled by the world | I6, I7, I8, I9, I10, I36, I49 | `Controller:setTask` / `pushTask` (G4–G6, G9, G23) |
| 5 | **Environment queries** | I21–I28, I46 | `land.*`, `atmosphere.*` (G12–G15, G21) |
| 6 | **Keysites and structures** | I29–I34 | Airbases, statics, BASE_CAPTURED (G16, G17) |
| 7 | **Time, random and lifecycle** | I18–I20, I37–I40, I47 | Timer stepping, a campaign-owned stream, a campaign-state save (G11, G18–G20, G25) |
| 8 | **Players** | I44, I45 | F10 menu, PLAYER_ENTER_UNIT (G24, G27) |
| 9 | **The link split:** world W and E into EECH_WORLD, C and F into `eech_dc.dll`, with fatal defaults (§4.3) | §1 | — |

Criterion 2 is complete only at step 9, when the guard and symbol checks hold with no campaign→world symbol left. Criterion 3 is complete when arrival and loss are both host-driven. Criterion 4 needs the M4 pack on the separated candidate, with every difference classified. Whether the separated candidate can still match `windows-retail v1` exactly, with equivalent inputs and semantics, is unproven either way. Moving the physics does not by itself show that identity is impossible. The requirement is to preserve exact accepted behaviour wherever inputs and semantics are equivalent, and to classify only measured differences, under the existing governance.

## 6. Decisions needed from the lead

Items 1, 2, 4 and 5 are decided in §0; 3 and 6 remain open.

1. **Decided (§0): loss first, accepted as sequencing.** Is loss first acceptable (§3), or must arrival come first as the earlier "#74, arrival first" note had it?
2. **Decided (§0): not approved.** The step-boundary semantic change (1b). Consequences can no longer run in the middle of an update. Approve classifying the resulting M4 differences as `intentional-semantic-change`, or say what standard applies.
3. **Identity issuance:** host-issued `index.serial` in EECH_WORLD; in DCS, campaign-supplied names that the host owns (§3).
4. **Decided (§0): not approved.** Wreck removal is deferred while the campaign still binds the entity (1b). This is a world lifecycle change made for safety in the shared heap.
5. **Decided (§0).** The criterion 4 standard after the link split: preserve exact accepted behaviour wherever inputs and semantics are equivalent, and classify only measured differences. No divergence is presumed.
6. **The enforcement set** (§4): the runtime guard plus partition symbol closure now, and fatal defaults at the split.

## 7. Limitations

- One seed, 1,800 s, Lebanon only, and one loss (a helicopter). Fixed-wing, vehicle, ship and person losses are characterised by source reading only.
- The attribute counts are raw token counts with the receiver unresolved.
- The two-PE-module claim (§1) is inferred; it is not built.
- DCS event duplication, name uniqueness and id reuse are [U].
- No DCS run was made.
