# Deliver bounded migration increments

These controls implement [the D3 five-whys review](D3-FIVE-WHYS-20261102.md)
requested by the owner under EEDCS-53. Live Confluence still defines scope.

1. Start from the active extraction card, current task/review and ONE compact
   current runtime checkpoint. Read historical evidence only for the next unmet
   gate. Never resume a stale PID, tool session or old-machine context by assumption.
2. Name the next **production** change before preparing another diagnostic. Its
   contract covers the actual inputs, outputs, side effects, RNG/FPU/order and
   immediate consumer. Capture these together rather than discovering one missing
   field after each replay. A real correctness uncertainty remains a valid blocker.
3. Preparation is limited to two completed batches without a production cutover.
   At that point, make the bounded cutover or fix a named blocking finding. A new
   diagnostic must identify exactly which production change it unblocks and what
   missing fact it supplies. This is a decision by the lead, not a new user approval
   step. Do not cut over unsafely to satisfy the limit.
4. Batch known review findings in the same implementation lane. Run focused checks
   and formatting before one clean freeze; then run the applicable existing build,
   deterministic scenario and comparison. Reuse equivalent accepted evidence.
   Prose, publication and checker-only edits do not justify rebuilding unchanged C
   or rerunning a war. Changed C retains its own identity and needs the affected
   actual checks. Never change a baseline, tolerance or scenario to obtain green.
5. One implementation lane and one independent reviewer cover correctness, clean
   code and boundary safety. Re-review changed material or new actuals, not closed
   evidence. Numeric prototypes and diagnostic counts are preparation, not host
   authority, a production cutover or delivery. Finish the current increment before
   adding another optional prototype.
6. Keep one owner per build cache, one writer per combined log, and immutable
   source/bin/root identities. Inspect actual PID/birth/argv and completion before
   restarting. Keep the established pack running; quiet output is not failure.
7. Report production changes, complete/partial recording equality, remaining Core
   coupling and the next gate. Separate unavoidable regression wall time from
   agent/build rework. Estimate run progress from simulated time, not file bytes;
   label extrapolations and programme ETA uncertainty. Do not infer an end date
   from a growing prefix.

Before starting work, run from the repository root:

```powershell
python eech-campaign/tools/migration-progress.py
python eech-campaign/tools/migration-progress.py --action fix-blocker --blocker rng-evidence
```

The guard rejects another unmotivated preparation batch and rejects cutover or
delivery with unresolved required gates. Delivery also rejects every open finding
in the card, including findings that need not prevent starting a cutover.
It never requests permission or authorizes
a merge. Update the card when evidence actually changes, not to bypass a failure.

Keep the live runtime checkpoint outside Git: at most 12 KiB, with objective,
current task/branch/source, active jobs and their receipts, closed evidence,
blockers and next action. Archive useful prior state before replacing it. Older
checkpoint names must point to this one record rather than contain contradictory
live state. The tracked card holds stable decisions; the checkpoint holds transient
PIDs, paths and tool IDs. On this host the canonical checkpoint is
`R:/d3-evidence/D3-CURRENT.json`, mirrored on D:.

For the current Task52 the RNG-evidence group is closed and the offset initializer is cut over
(`9b916ac650`: seeker policy once per invocation, all three host-authored local offsets
for the direct guidance consumer). The next implementation is the host routing of
`get_local_entity_target_point` (instance chain and live placement, fixed position), with its
native authority run as Root's gate; it is not another optional characterization. Valid
retained numeric evidence is not rerun for it, and the Task50 pack stays untouched.

STATUS.md keeps ONE current summary, first in the file and dated; every later entry is
history that keeps its own date and scope, and none of them is live state even if an
older heading once said "current". A fresh context reads the active card, that summary
and the runtime checkpoint, and opens an older entry only for the evidence of the next
unmet gate. When the state changes, replace the top summary and demote it to history
instead of adding a second "current" heading.
