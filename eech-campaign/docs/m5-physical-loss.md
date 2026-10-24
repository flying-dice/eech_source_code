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

WITHHOLD is implemented for the characterized AI helicopter SUPPLY case and the
bounded shapes of the other five families (see the family table below).
The captured helicopter member must be one of at least two physical group members, airborne,
without movement cargo/parent, group task dependents, restricted guides, takeoff
queue state, or credited aggression. Its exact loss-derived work is four
unassigned, incomplete, non-primary ENGAGE tasks aimed at that member and one
helicopter landing-site reservation. Other member types, final-member loss,
players, CREDIT withholding, other task/landing shapes, and stale referenced
campaign entities fail at the boundary.

Fixed-wing adds one bounded landing shape. A normal in-flight fixed-wing member
whose primary task assesses landing carries exactly one `UNRESERVE_SITE` count-1
landing action, against the group's keysite landing (`FW` or `FW_TRANSPORT`), and
that landing must currently hold at least that many reservations
(`is_supported_fixed_wing_reservation`, `gp_roster.c`). The observe step stores the
landing generation key and count; WITHHOLD leaves the reservation in place. The
host-ID report validates the landing key (stale is fatal), type, count and current
reservation, advances the LANDING phase in its original position, and releases the
reservation once through `unreserve_local_landing_sites`. The receipt records
`landing_reserved_before/after`. The campaign landing accounting stays in the DLL;
this is D1 loss-consequence handoff, not D4 airfield migration. Landing,
takeoff, route/site unlock and any other landing-action shape, or a count other
than one, is still refused (`campaign loss: WITHHOLD does not support a member
with landing actions`). No task subtype is fabricated, no primary task is
detached, and the campaign task database is not adjusted.

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

## The other five families (EEDCS-41)

All six destroy files (helicopter included) hand their campaign consequences to
the DLL through one accepted loss-report transaction keyed by the opaque host
identity. Each `kill_local` calls `campaign_roster_accept_physical_loss (entity,
group, &ticket)`: the DLL asks the host for its loss report decision and fills a
pointer-free `campaign_loss_ticket {decision, record_index, host_id}`.

- **FORWARD** (the host reports now, in the caller's stack): the DLL opens a
  transaction on the member's record. Every campaign consequence then runs only
  through `campaign_roster_loss_step (&ticket, phase)` at the points where the
  original code ran it. The step verifies the ticket's record index and host
  identity against the open transaction, advances the phase (strictly ascending,
  each phase once), copies the private generation keys, re-resolves the member
  (and the group, only for the phases that need it) and applies the shared value
  leaf through `apply_member_loss_phase`. No entity pointer is stored in the
  ticket or the transaction. A second accept on an open transaction, an
  out-of-order or repeated phase, or a step with a mismatched identity is a
  `debug_fatal`.
- **WITHHOLD**: no transaction opens and a step with that ticket applies nothing;
  the pending record owns every consequence and `engine:report_loss(host_id)`
  runs them once, in the same enum order, through the same phase check.
- A CREDIT FORWARD opens no transaction. An answer that is neither forward nor
  withhold is a `debug_fatal`.

The physical work (death, sector removal, UI, Tacview, the ship's keysite kill,
the person's `TASK_COMPLETED` loop and the helicopter player paths) stays in the
destroy files unchanged, so the host report decides the campaign consequences while
the original interleaving is kept. The phases are `FORCE_INFO`, `REGEN`,
`MEMBER_TASKS`, `LANDING`, `GROUP_LOSS`, `TASK_LOSS`, `MEMBER_KILLED` and
`OBJECTIVES`; a family steps only those its original code ran:

| Family | Phases at the original interleave points |
|---|---|
| fixed-wing, routed | force info; (server) regen, member tasks, landing; group loss, task loss; (server) member-killed |
| ship | as above without regen |
| anti-air | force info; (server) member tasks, landing; group loss; (server) member-killed |
| person | group loss (early, before its unlinks); (server) member-killed |
| helicopter | force info; (server) regen; member tasks, landing, group loss, task loss; (server) member-killed, objectives |

The non-roster build maps the ticket calls straight to the original helper
(`group.h`), so the other builds are unchanged.

A withheld report for the bounded families applies the same leaves in the
original FORWARD order:

| Family | Report order after the host's identity loss report |
|---|---|
| fixed-wing, routed | force info, regen, logical unlink, landing reservation release (fixed-wing single `UNRESERVE_SITE` only), group losses, primary-task losses (when a primary task was recorded), group member-killed |
| ship | as above without regen |
| anti-air | force info, logical unlink, group losses, group member-killed |
| person | logical unlink, group losses, group member-killed (no force info) |

The helicopter SUPPLY case, its receipt, tombstone and stale/duplicate protections
are unchanged, and it keeps the force objective check. The bounded families skip
`check_all_force_campaign_objectives`.

Bounded shape: an AI, non-landed member of a group with at least two logical and
two physical members and no final-member, task-dependent, restricted-guide,
movement parent/dependent, takeoff-queue, member-task-dependent, unsupported
landing (only the fixed-wing single reservation above is supported), or
credited-aggression state. Anything else fails explicitly at the observe step
(`begin_bounded_pending_loss`), including a final member (`final member`), a
player, an invalid force category, and a duplicate or incomplete pending record.
CREDIT withholding stays fatal.

Report-final contract (`physical_loss_is_final`, `gp_roster.c`). A report is
accepted only when the loss is pending, the member is out of the group and the
physical loss is final: either the host released the identity
(`!physical_present`), or the record still maps the current incarnation
(`current_member_by_entity[index] == record + 1`) and that entity is of the
member's type, dead and unlinked from its group. A stale, reused or live entity
is fatal. This replaces the earlier rule that the host must have retired the
wreck first, which no host can satisfy for every family: fixed-wing,
helicopter, routed and person wrecks age out (5 minutes within 1000 m of a
keysite, up to 3 hours at 10000 m or more), ships sink and are destroyed, and
anti-air wrecks only age and are never retired. The helicopter path is
unchanged.

Force-info and regen effects are observable through the fixture-only
`loss_state(side)` (feature `loss-fixture`, absent from the default module):
loss and hardware counters per force-info category and the regen ring. The ring
is written at `(front + count) % size`; at capacity `front` advances and `count`
stays, so the effect is read as the last written slot, not as a count rise.

Setup fixtures (feature `loss-fixture`, never in the default build):
`loss_fixture`, `loss_state`, `loss_shape` (a read-only support screen),
`loss_group` (a read-only group shape diagnostic) and `loss_release`.
`loss_release(entity_id)` is an explicitly artificial setup step: it ends the tasks
whose objective is the member's group or one of its members (including member
target tasks) by sending the original `TASK_COMPLETED` /
`TASK_TERMINATED_OBJECTIVE_MESSAGE` that a group kill sends (`group.c`), and ends
engage guides. It changes no guard and no physics, and does not claim the
campaign would have released those tasks itself. The default Lua module must not
expose `loss_release`, `loss_group` or `loss_shape` (asserted by the shipping API
test). Airborne fixed-wing groups are usually
the objective of some task, so without it no fixed-wing member is in the bounded
shape. That is bounded evidence from one observed census (no eligible fixed-wing
member was found there), not a claim that none can exist; the fixed-wing witness
therefore rests on this artificial setup and is stated as such.

Fixed-wing witness evidence: the controlled run observes reservation 2, reads
report-time reservation 2 with no intervening frames (no FORWARD control exists for
a two-member group, so the preservation check is a direct state read, not a
same-landing comparison), and the report moves it 2 -> 1 once. The forward control
needs a group of at least three members, so it was not run for this case.

Not covered: save/restore, a group freed while a loss is pending, the delay
robustness of a late report, ship keysite/carrier exclusions, and the person's
variadic `TASK_COMPLETED` loop (unchanged).

## Known incomplete boundary

The physical readers above still observe physical removal before a withheld loss
is reported. The shared physical getter must remain available to world
simulation.

Save/restore of pending loss records and of logical membership is unsupported.
A FORWARD final kill of a group while another member's loss is still withheld
(a group freed with a pending loss) is uncharacterised and unsupported; the
`begin_pending_loss` guard covers only a WITHHOLD final member. Six entity
families retain different consequence sets; helicopter SUPPLY and the bounded
shape of the other five families (see above) enable WITHHOLD, and every other
shape rejects it. DCS mapping and
actual DCS operation remain future evidence. A passing reference-host run
demonstrates the bounded causal cut only.

An empty-root start crash in the lifecycle harness predates this change (it
reproduces on `e15830f8`) and is recorded as an existing defect, not fixed here.

## CI coverage and the slice 1a check

The slice 1a token-equivalence check (`eech-loss-structural`) is pinned to the
extraction commit `78dc796b`; this change deliberately differs from it (see
[`m5-loss-structural.md`](m5-loss-structural.md), "Revision boundary"). Hosted
CI has no automated forward, withhold, duplicate-report or lifetime test of the
current head. Those are covered only by the local Windows retail-host runs below.

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

### Provenance of the tested binaries after the `5917c7eb` merge

The evidence above was produced by the candidate binaries (not committed).
Commit `feaafaa5` committed those ten inputs; merge `d6c56c81` then added only
PR86 tool, CI, doc and Lua-script changes (no `aphavoc`, `csrc` or `.rs` file).
A clean-source build of `d6c56c81` (`BUILD-INFO.txt`: 0 uncommitted files, the
same image `bb10e44a…`, rustc 1.98.1, a new target volume) was compared with the
tested build (`tools/build-windows.sh`, reproduced locally 2026-10-21):

| File | Tested | Clean `d6c56c81` | Result |
|---|---|---|---|
| `eech_dc.dll` | `3ff6c7bb…9b02a` | `7432b839…bc829` | differs, metadata only (below) |
| `eech-world.exe` | `15bc66bb…c6f081` | `0994c768…6b146d` | differs, metadata only (below) |
| `lua.dll`, `lua/campaign.lua` and the other `lua/*` | equal | equal | identical bytes |
| `lua/supply-arrivals-lifecycle.lua` | `c9f1e082…` | `2bf7ecad…` | the PR86 repair (unknown case fails before setup) |

For both PEs the file size is equal and every loaded section (`.text`, `.data`,
`.rdata`, `.pdata`, `.xdata`, `.idata`, `.CRT`, `.tls`, `.reloc`) is byte-identical.
The differing bytes are the PE header timestamp and checksum, the export-table
timestamp (`.edata`, `eech_dc.dll` only), and the non-loaded COFF symbol/string
tables: the same symbols reordered, with exactly one pair of linker-generated
names differing by rustc's temporary directory (`..._rustcoDth8m_kernel32_dll_imports_lib`
versus `..._rustcruq1wZ_...`). No instruction, data or import differs. The tested
artifact identity above remains the one the evidence applies to. The pack was
therefore not rerun. Not rerun and not claimed: the arrival and host-identity
lifecycle scripts against the PR86-modified Lua driver (only the unknown-case
guard and argument quoting changed; PR86 carries its own self-test,
`tools/supply-arrivals-tools-selftest.py <lua> <pwsh>`).

Tacview (reproduced in the pack run, `pack-loss-logical/out`): `recording.acmi`
SHA-256 `3bd17b64…d19b` is byte-identical in both runs and to the retained
recording (committed in compressed form as `reference/lebanon-3h/recording.zip.acmi`,
`3b6d81d8…47`); `regress.georgia.exact` and `regress.lebanon.exact` report
IDENTICAL to windows-retail v1. `report.json` `ad407cec…caf69`. The recording was
generated and compared structurally and by bytes; it was not visually reviewed in
Tacview.

Real public loss receipt (Lebanon seed 1, 2300 s, WITHHOLD; raw JSON from
`receipt-v4.jsonl`, SHA-256 `577d1198…5d5e`):
`{"elapsed":2187.3444824219,"event":"report_loss","group_logical_members_after":1,"group_logical_members_before":2,"group_losses_after":1,"group_losses_before":0,"group_physical_members_before":1,"host_id":"eech-world:3069","landing_reserved_after":5,"landing_reserved_before":6,"primary_task_losses_after":1,"primary_task_losses_before":0,"result":"success","tasks_completed_on_report":0,"tasks_retired_expire":4}`
followed by `receipt_assertions` `passed:true`; the duplicate report was rejected.

Commands: `tools/build-windows.sh <out>` (fresh `EECH_WIN_TARGET` volume),
`tools/supply-arrivals-lifecycle-windows.ps1`, `tools/host-identities-lifecycle-windows.ps1`,
`tools/regression-pack.py`. Remaining coupling: campaign and the reference world
still share one `eech_dc.dll`; the physical readers listed above stay physical.
