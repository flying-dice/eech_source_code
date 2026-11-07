import sys

P = sys.argv[1] + "/eech-campaign/docs/d3-sector-height-production-contract.md"
raw = open(P, "rb").read()
assert b"\r\n" not in raw
d = raw.decode("utf-8")


def rep(old, new):
    global d
    assert d.count(old) == 1, (old[:90], d.count(old))
    d = d.replace(old, new)


rep("""Positive: the Host word equals the original's for every recorded raise sequence (the 13 patterns, strictness, no-lowering, rounding toward zero, a negative-restored raise, a NaN-restored no-raise); the Core consumer uses the
Host word (fact 12 carries it, the fixed-wing `min_height`, the pack bytes); a restore ACK drives the real consumer. Missing (a provider without the phases, a refusal), wrong (another sector, a short/long/non-matching reply, a word
not the stored one, a fact 12 that is not the Host's word), stale (a word that survives RESET, a restore for another sector, a restore after an entity insertion): each a named refusal with the legacy field never consulted.
The ordinary short, a `weapon-fixture` run and the baseline are Root's native gates after the SAME source clearance; a legitimate bounded controlled public path reaches the guarded fixed-wing consumer and the pack/unpack
(declared controlled fixtures, never natural engagement; no duration, actor or ray is tuned).""", """Positive: the Host word equals the original's for every recorded raise sequence (the 13 patterns, strictness, no-lowering, rounding toward zero, a negative-restored raise, a NaN-restored no-raise); the Core consumer uses the
Host word: in the real-engine control the getter and the pack read the scripted provider's word, and in the real campaign the scan's FIRST read is carried back as fact 12 (section 11 shows it natively). Missing (a provider that
refuses or withholds), wrong (another sector, a short/long/non-matching reply, a restore stored under another word, a fact 12 that is not the Host's word), stale (a word that survives a boot's RESET: a read after it is the zero word):
each a named refusal with the legacy field never consulted. **A restore (phase 18) is accepted by the provider whenever it is well formed, including while placements are live**: the provider deliberately has no insertion-time
rule for it. That a load restores the sectors before it inserts entities is the campaign's own session order (`en_sessn.c`, pinned by the source guard), not a provider rule, and no refusal for a restore after an insertion is claimed.
The scripted tests that make the campaign's getter read or restore (`entity_authority.rs`, the dialogue check, the real-engine control's scripted provider) are BOUNDARY CHECKS of the contract and the drivers: they manufacture phases 17
and 18 themselves and are not native campaign authority. The ordinary short, a `weapon-fixture` run and the baseline are Root's native gates after the SAME source clearance (section 11).""")

rep("""M5 (guarded clearance) and M6 (save/restore) were UNREACHED in the accepted baseline. The real guarded consumer expression and the real pack/unpack are controlled in this increment by the real-C control (extended in
place, running the real engine code in the hosted profile, and reused unchanged in the original-solver profile through `collision-fixture`), and by public-driver tests with the scripted host over the REAL roster. Their
native reach is a named Root witness to be taken after the SAME source clearance; unreached stays unreached until observed.""", """M5 (guarded clearance) and M6 (save/restore) were UNREACHED in the accepted baseline and stay so. **No control reaches the guarded fixed-wing execution**: `fixed_wing_movement_waypoint` is a static function of `fw_move.c`; its
statement text and hand-off are pinned by a source guard and its read is the Host's word through the getter, and nothing more is claimed. The real-engine control exercises the real getter and the real pack and unpack over a
SCRIPTED provider (hosted and original-solver profiles, x86-64 and i686): that is not a native Core server save/load, and a Host-only raw restore (phase 18 asked by a test or a driver) is not a Core save/load either.
`ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1` and `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1` stay required OPEN inside the height extraction until actual proof; the missing prerequisite of each is a named fixture entry
(a task-leader fixed wing whose guide follows terrain; a real server-session save and load), which does not exist in the public bridge and is not invented here.""")

rep("""Fresh source/profile gates, observer-off ordinary short against the accepted `80fd` files (the move changes only the order of nothing: the same words reach the same consumers), the declared window baseline compared word for
word (fact 12 offers, 63 FIRST reads), then the controlled witnesses of section 7.""", """See section 11: the executable plan (commands, features, modes, row schema, eligibility, expected outcomes). Native execution remains Root's, after the SAME clearance.""")

rep("""  profile; it observes no host word.""", """  profile, where its original-link hooks (SH2/SH3 raise, SH4/SH5 save/restore) live. In a hosted build those four hooks are compiled out, but the scan hooks SH10 and SH11 STILL capture the Host-returned FIRST word and every fact 12: the
  `hosted-scan` scope (section 11) reuses exactly that, compared by sector with the accepted original report at the same window, and nothing else; the observer is not globally unusable, and a whole original-link report comparison is invalid
  in a hosted build. Its clearance helper no longer reads the retired member in a hosted build: the record says `tallest=uncaptured` and the consumed word is never re-queried.""")

d = d.rstrip("\n") + """

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
"""
open(P, "wb").write(d.encode("utf-8"))
print("ok")
