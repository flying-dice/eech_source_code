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
verified by generation) and `tasks_completed_on_report`. It exists because the
original `debug_log` audit is compiled out of release builds. It is an
observation only: nothing reads it back, it is not stored, and callers may
ignore it. It is produced only for a successful exactly-once report; duplicate,
stale and unsupported reports raise and leave the C output untouched. It does not
cover other loss families, physical count/list/alive consumers or save/restore.

Campaign-screen physical member removal remains at physical death. Critical-hit
credit remains at its original site and cannot be withheld by this experiment.
FORWARD adds no simulation-time advance, queue, random draw, or changed physical
ordering.

## Known incomplete boundary

Physical MEMBER lists and `INT_TYPE_MEMBER_COUNT` still serve many campaign
policy consumers. Assignment minimums, task generation, ENGAGE selection,
completeness and some objective checks can therefore observe physical removal
before a withheld loss is reported. This experiment does not relabel the roster
snapshot as campaign authority. Those consumers need a durable campaign-member
view plus explicit handling for a campaign-active member with no physical
binding. The shared physical getter must remain available to world simulation.

Save/restore of pending loss records is unsupported. Six entity families retain
different consequence sets; only helicopter SUPPLY withholding is enabled and
the other five families continue to reject WITHHOLD. DCS mapping and actual DCS
operation remain future evidence. A passing reference-host run demonstrates the
bounded causal cut only.
