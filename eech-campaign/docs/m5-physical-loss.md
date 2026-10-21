# Experimental physical-loss boundary

This M5 experiment separates one captured campaign loss from the reference
world's physical death. It does not establish a complete replaceable-world
boundary or accept an M5 criterion.

`boot` requires an `observe_loss` callback. Returning `forward` preserves each
original destroy path and remains the reference default. Returning `withhold`
never prevents damage, `ALIVE = FALSE`, physical list detachment, wreck motion,
death timing, or entity free. A later `engine:report_loss(host_id)` applies the
retained campaign consequences once for that opaque physical incarnation.
Unknown, stale, duplicate, or unsupported reports fail visibly.

## Bounded supported case

WITHHOLD is implemented only for the characterized AI helicopter SUPPLY case.
The captured member must be one of at least two physical group members, airborne,
without movement cargo/parent, group task dependents, restricted guides, takeoff
queue state, or credited aggression. Its exact loss-derived work is four
unassigned, incomplete, non-primary ENGAGE tasks aimed at that member and one
helicopter landing-site reservation. Other member types, final-member loss,
players, CREDIT withholding, other task/landing shapes, and stale referenced
campaign entities fail at the boundary.

The loss record copies force, regeneration, group, primary-task, task and landing
values before physical unlink. Group, task and landing entities use private
generation keys; no entity index or raw pointer remains authority after free.
The later report preserves the original campaign-action order:

1. force current hardware and loss statistics;
2. regeneration queue insertion;
3. loss-derived ENGAGE completion when the task is still live;
4. landing reservation release;
5. group and primary-task loss counters;
6. group member-killed policy;
7. force objective checks.

The report holds only keys and values across campaign actions. Task completion
can cascade through guides and groups, and `apply_group_member_killed` frees the
group when its physical `MEMBER_COUNT` is already zero at report time (the
observe-time preflight guaranteed two physical members only at observe time).
So each group, landing and primary-task entity is re-resolved from its
generation key immediately before its use, the pending ledger is re-looked-up
before each state write, and the audit values are read before the one call that
can free the group. A stale or freed key is a loud failure, never a skip.

The four observed ENGAGE timers expire before the reference wreck is freed.
Those timers and their guide/group cascades continue normally while loss is
withheld. Immediately before an affected task's existing termination mutates or
destroys it, the ledger records the same generation and
`TASK_TERMINATED_EXPIRE_TIME_REACHED`. At report, only that explicit tombstone is
an already-retired action. A generic missing task, a reused entity slot, or a
different termination reason is an error. This avoids a ghost task, paused timer,
or silent skip.

`engine:report_loss(host_id)` returns a table copied from the actions that
report performed: `landing_reserved`, `group_losses` and primary-task loss counts
before and after (`*_before`, `*_after`), `tasks_retired_expire` (tombstones
verified by generation), `tasks_completed_on_report`, and the group's logical
member count before and after the report plus its physical count before it
(`group_logical_members_before`, `group_logical_members_after`,
`group_physical_members_before`). It exists because the
original `debug_log` audit is compiled out of release builds. It is an
observation only: nothing reads it back, it is not stored, and callers may
ignore it. It is produced only for a successful exactly-once report; duplicate,
stale and unsupported reports raise and leave the C output untouched. It does not
cover other loss families, physical count/list/alive consumers or save/restore.

Campaign-screen physical member removal remains at physical death. Critical-hit
credit remains at its original site and cannot be withheld by this experiment.
FORWARD adds no simulation-time advance, queue, random draw, or changed physical
ordering.

## Campaign logical membership

The roster keeps a campaign-owned, durable member record beside the physical
binding. A record is `active` from its first bind. It counts toward its group's
logical membership while it is `active` and either physically linked (`in_group`)
or awaiting a host loss report (`loss_pending`). Physical `present`/`in_group`
are unchanged and still describe the world only.

- A live transfer unlinks and relinks the same member key, so the logical count
  is unchanged and no loss is implied. Reordering is likewise not a loss.
- A FORWARD loss retires the member at the original unlink phase
  (`campaign_roster_unlink_member` in `kill_local`), so FORWARD observes the
  same count at each original phase as before.
- A WITHHOLD loss leaves the member counted through physical unlink, death and
  free. The host report retires it once, at the logical-unlink phase: after the
  landing release and before the group and primary-task loss counters.
- A freed incarnation with no report owed leaves the campaign at release.

`campaign_roster_group_member_count (group)` is the private campaign view. For a
campaign group it returns the logical count and fails closed (`debug_fatal`)
unless `logical == physical + withheld`, where `withheld` is the group's records
with a pending loss and no physical link. Any other gap is an uncharacterised
birth or transfer order and is not hidden. It falls back to the physical
`INT_TYPE_MEMBER_COUNT` for non-groups and for entities with no campaign group
key, and in the non-roster build stub. The shared `gp_int` getters are unchanged.

Migrated policy readers (eight reads, all through the accessor):

| Site | Policy |
|---|---|
| `ai/taskgen/assign.c:386` | enemy-airbase task needs a group of at least two |
| `ai/taskgen/assign.c:476` | assignment checks the task's minimum member count |
| `ai/taskgen/taskgen.c:589, 972, 1115, 1624` | member count stored in the new task's user data |
| `entity/special/task/task.c:226` | ESCORT completeness |
| `entity/special/task/task.c:272` | BAI/CAS/SEAD completeness |

`observe_loss`'s descriptor `campaign_member_count` also reads the accessor.

WITHHOLD now additionally refuses the final physical member of a group
(`begin_pending_loss` fatal): the final-member consequences (group free,
`MEMBER_COUNT == 0`, empty-group task effects) are uncharacterised.

### Physical readers deliberately not migrated

Criterion 2 stays open: these still read the physical list or count, and each is
a physical or awareness question that has not been decided as campaign policy.

- `special/task/task.c:1128` (`INT_TYPE_ALIVE` on a group: keysite capacity bias)
- `ai/highlevl/reaction.c:534, 561, 577, 931, 952`
- `mobile/mb_msgs.c:746, 2293`; `aa_dstry.c:641`; `landing/ld_msgs.c:1402`
- `group.c:418, 420, 507`; `gp_pack.c`; `gp_int.c`; `test.c:1283`
- `gp_msgs.c:1297` (`apply_group_member_killed`: `MEMBER_COUNT == 0` frees the
  group; it remains physical because freeing is a world-lifetime fact)

### Decision witness: not obtained

No assignment or completeness decision was observed to differ. In the supported
shape the group keeps at least one physical member, and the Wolfpack SUPPLY
minimum is 1, so 2 logical versus 1 physical does not flip an assignment. The
completeness readers apply to ESCORT/BAI/CAS/SEAD tasks whose objective is the
withheld group; in the receipt scenario the four tasks aimed at the member are
ENGAGE tasks that expire before the free. The `assign.c:386` rule needs a
two-member group assigned against an enemy airbase inside the window, which this
scenario does not produce. The roster count (receipt before/after fields) is a
membership observation, not proof that a decision changed. No policy constant,
force outcome or scenario was altered to manufacture one.

## Known incomplete boundary

The physical readers above still observe physical removal before a withheld loss
is reported. The shared physical getter must remain available to world
simulation.

Save/restore of pending loss records and of logical membership is unsupported.
A FORWARD final kill of a group while another member's loss is still withheld
(a group freed with a pending loss) is uncharacterised and unsupported; the
`begin_pending_loss` guard covers only a WITHHOLD final member. Six entity
families retain different consequence sets; only helicopter SUPPLY withholding is
enabled and the other five families continue to reject WITHHOLD. DCS mapping and
actual DCS operation remain future evidence. A passing reference-host run
demonstrates the bounded causal cut only.

An empty-root start crash in the lifecycle harness predates this change (it
reproduces on `e15830f8`) and is recorded as an existing defect, not fixed here.

## Evidence for logical membership (local, 2026-10-21)

Candidate: commit `b7e63267` plus the ten uncommitted build-input files of this
change, built on image `sha256:bb10e44a20e9c6304b3a318902bcf0f8ec8175c162862240ce25c34bd7e562be`
in a fresh target volume. `eech_dc.dll` `3ff6c7bb…9b02a`, `eech-world.exe`
`15bc66bb…c6f081`. Artifacts are under `E:/eech-runs/m5-host-boundary/loss-logical-run/`
and `pack-loss-logical/` (not committed).

- Public lifecycles: arrival 4/4 PASS; host identities 13 PASS plus the known,
  unchanged `missing_map` defect.
- Lebanon seed 1, 2300 s WITHHOLD (`probe-receipt-v4.lua`, fail-closed
  `receipt_assertions`: passed): physical free at 2187.344482 precedes the
  report; the group's logical members are 2 (physical 1 plus the withheld one)
  until the report and 1 after; landing 6 to 5; group and primary-task losses
  0 to 1; four verified expired-task tombstones; zero live completions; a
  duplicate report is rejected. No `diverges` or final-member fatal occurred.
- One full M4 pack (`tools/regression-pack.py`, windows-retail v1, 3 h Georgia
  and Lebanon, seed 1): PASS, no FAIL; both `regress.*.exact` PASS, so no baseline
  difference. SHA-256 `report.md` `ddd965db…38cdc`.
- Not shown: a changed assignment or completeness decision (see above).
