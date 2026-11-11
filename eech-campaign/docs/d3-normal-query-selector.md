# Signed normal-query selector correction (pending SAME review)

Named blocker `ENV-NORMAL-FIRE-QUERY-SELECTOR-DOMAIN-1` remains under open
GenericNative1. This is a diagnostic codec correction, with no physical change.

## Actual failure

Reviewed56d1b healthy native/controller/Root33010 ended1 at reported frame1497,
on `normal-fire: integer domain`. B was never prepared/launched. Root binding
`normal-fire-56d1b-terminal-Root-binding-20261108.json` (`4f1f331c…`) binds the
private complete raw stream, inputs/configurations and unchanged97/10 artifacts.
The report's zero traffic counters are fallback after latched `finish` failure,
not evidence of no physical traffic. Raw314092 consecutive NF records include
296 fire/detach/birth triples,155096 step pairs,8 queries and2700 model entries;
last tick149600. Six queries legitimately state `which=-1`, two state0.
Prior boot-retention sourcef0, exact launch8a14,56d1b all9CI and the accepted
default3COMPLETE literal-equality control retain their scopes. No replay.

## Complete producing path

The actual D2 producer is `eech-world/csrc/wn_flight.c`, not just a comment:

- Its sole `wf_q(WL_COLLISION, ...)` call at2600 takes `asked_which` from
  `wf_target_which` (1500–1502). `wf_q` copies that integer directly to `q.a`.
- `wf_target_which` returns-1 when `!has_target`; otherwise `target_which`.
  The only writes initialize that state to0 (2923), or set it to1 after acquiring
  the launcher's target (1852). There is no producing value outside {-1,0,1}.
- `wn_move.c::CAMPAIGN_STEP_LOOKUP_COLLISION` (454–475) forwards `lookup->a`
  unchanged to `NF_QUERY_BEGIN`. `weapon_step_target` (157–174) resolves0 as
  the weapon's original target,1 as the launcher's target and none for-1.
  The collision path explicitly permits a weapon with no current target.
- `eech_nf_query_begin` prints that signed integer with `%d`. Lua/Python must
  accept its exact canonical domain, not every nonnegative integer or arbitrary
  signed integers. `-0`, fractions and nonnumeric tokens are not produced.

Both Lua checks (`M.event` and `N.feed`) now accept -1..1; the query parser also
rejects textual-0 explicitly (Lua's negative-zero `tostring` otherwise permits
it). Independent Python uses the same exact domain with its canonical grammar.
The previous faulty '-1' negative fixture is now '-2'. No selected birth/target,
moving-pose, identity, model-call/op, semantic, common-prefix, flag or failure-
precedence gate changes, and no alternative actor/window or geometry substitution.

## Focused evidence and next gate

Offline old-codec RED1 and corrected GREEN0 scan the SAME complete private raw
file to EOF:315244 lines,314092 NF records,313796 selected-codec records. Python
intentionally excludes296 `normal_end` rows; no observer errors were present.
Safe numeric projections/hashes are published; physical tuples/raw100MB remain
private. This is selected-event codec replay, not a full live provider-order replay.
Portable Lua has0/1/-1 positives through both parser and contextual collector,
three real reference-provider contexts, and seven invalid-domain negatives.
Independent Python covers the same bounds and full synthetic reference gates.
Genuine old Lua RED101 and the intermediate '-0' diagnostic failure are retained.

Whole3816+2 compiled inputs/default reuse remain joined to immutable2fc. Corrected
literals/validator are separately bound; no C/Rust product/build change, Windows
rebuild, native/default replay, physical input/baseline/seed/window change or
runtime relabel. STOP for SAME changed-material review, then Root owns any later
materialization/launch review. All ten/preparation2/GenericNative1, fullpacks,
physical cutover, Task49 and whole D3 remain OPEN.
