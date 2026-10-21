# M5: the campaign's loss consequences, moved out of world destroy code (slice 1a)

**Scope.** This is structural slice 1a of the [host-boundary proposal](m5-host-boundary.md), and nothing more.
- **Authorisation:** the technical lead's decision on Jira [EEDCS-11, comment 10445](https://flying-dice.atlassian.net/browse/EEDCS-11?focusedCommentId=10445).
- **Scope authority:** the Confluence goal [EECH M5 Goal: EECH_DC into DCS World](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337), version 1.
- **The move:** campaign work done inline by the six world kill and wreck-removal files is moved into campaign C (`entity/special/group/gp_loss.c`). That work is the consequences of losing a group member and the detaching of a member from campaign lists.
- **What is preserved:** every per-type difference and the exact call order.

**What it is not.** No campaign rule, timing, lifecycle, random stream, host contract or baseline changes.
- **Same callers.** The world still calls this work, from the same places, and that remaining coupling is listed below.
- **No authority moved.** No physical authority leaves the DLL.
- **No criterion completed.** Criteria 2–4 of the M5 goal stay incomplete.

**Revisions.** The base is `163e9158`, the merge of #79. Its product tree is identical to `65892de1`'s: #79 changed only retained evidence and the CI workflow. Source lines cited without a revision are at the base.

## The change

**New campaign functions** (`aphavoc/source/entity/special/group/gp_loss.c`, declared in `gp_loss.h` and included from `group.h`). Each one is the body of one contiguous original chunk.

| Function | The original chunk |
|---|---|
| `notify_member_task_dependents_completed` | The loop sending `TASK_COMPLETED` with `TASK_TERMINATED_OBJECTIVE_MESSAGE` to each task (not waypoint) on the member's `LIST_TYPE_TASK_DEPENDENT` list |
| `increment_group_losses` | The group's `INT_TYPE_LOSSES` increment |
| `increment_group_primary_task_losses` | The group's primary task's `INT_TYPE_LOSSES` increment, when it has one |
| `notify_group_member_killed` | `MOBILE_KILLED` to the group, after the member leaves the group's member list |
| `notify_forces_check_campaign_objectives` | `CHECK_CAMPAIGN_OBJECTIVES` to every force |
| `unlink_member_task_dependents` | Unlinking the tasks and waypoints that depend on the member (`LIST_TYPE_TASK_DEPENDENT`) |
| `unlink_member_from_guide` | Removing the member from its guide's followers (`FOLLOWER`; inserted only under a guide, `guide.c:793`). This does not update the guide's valid members, which `delete_group_member_from_guide_entity` does. |
| `unlink_member_from_group` | Removing the member from its group (`MEMBER`) |
| `unlink_member_from_takeoff_queue` | Removing the member from a landing's takeoff queue (`TAKEOFF_QUEUE`, inserted under a landing at `mb_msgs.c:335`) |

**Where the calls sit.**
- Each call is placed where its chunk was. The world's `get_comms_model () == COMMS_MODEL_SERVER` checks stay at the call sites; the helpers check nothing.
- **Already campaign calls, unchanged:** `remove_from_force_info`, `add_entity_to_regen_queue` and `release_mobile_entity_landing_locks`.
- **Not moved, because the list is physical carriage:** `LIST_TYPE_MOVEMENT_DEPENDENT` unlinks stay inline. A parent's movement is applied to its dependents (`ac_vec3d.c:171-183`, `vh_vec3d.c:169-182`). Cargo (`mb_msgs.c:1895`) and aircraft riding a ship (`faction.c:426`) are on it.

**Deviation from the proposal's design.** The proposal described 1a as one function per phase with a kind argument. The lead rejected a two-phase wrapper that moves code across world effects, so 1a has one helper per chunk, called at that chunk's position.

**Build registration:**
- `eech-campaign/crates/eech-engine-sys/sources.txt` (1,217 → 1,218 files);
- the legacy `group/Makefile.am` and `EECH-MSVC.vcproj`.

The kernel crate's synthesised `project.h` (`crates/eech-sys/build/spec.rs`) includes `group.h`, so it now sees the new declarations. It compiles none of the six files and none of the helpers.

**Call sites.** There are 73: 13 in `hc_dstry.c`, 12 in `fw_dstry.c`, 12 in `rv_dstry.c`, 15 in `aa_dstry.c`, 9 in `ps_dstry.c` and 12 in `sh_dstry.c`. They are in the `destroy_local` and `kill_local` of all six files, and in `aa_dstry.c`'s `kill_local_restored_anti_aircraft_entity` (the restore path, `aa_pack.c:310`).
- **Where they are:** `git diff 163e9158 -- aphavoc/source/entity/mobile` shows them all.
- **Proof of position and order:** [`expand-check.py`](../reference/m5-loss-structural/expand-check.py), below.

### Per-type behaviour, kept as it was

These differences between the types are characterised here, not corrected:
- **Regen:** only helicopters, fixed wing and routed vehicles add to the regen queue.
- **Task losses:** none for anti-aircraft or people.
- **Objectives:** only helicopters notify the forces to check campaign objectives.
- **Force info:** people never update force info.
- **Landing locks:** people do not release them (commented out, base `ps_dstry.c:404`).
- **People's order:** people count the group loss before the task notification, and send it without a reason.
- **Unlink order** differs per type; in `kill_local`, routed vehicles, anti-aircraft, people and ships unlink from the group before the guide.
- **The restore path** sends `MOBILE_KILLED` without counting a loss. Its client branch still kills an empty group.

Correcting any of these would be an intentional rule change, to classify separately.

### The person's task notification

`ps_dstry.c` sends `TASK_COMPLETED` with no `task_terminated` argument. The response reads one (`va_arg (pargs, int)`, `ts_msgs.c:285`) and passes it to `assess_task_completeness`.
- In C this is undefined behaviour. On this Windows x86-64 build, the value is whatever the fourth-argument register holds at the call.
- This slice leaves the notification loop inline, and also the group-loss chunk before it (with a comment). So the code leading up to the call is unchanged. The defect is not fixed here; a fix would be a separately characterised change.

**Observed codegen evidence:** the evidence commit records it, from the same comparison as before: `objdump -d -r` of `kill_local` in `ps_dstry.o`, base against candidate, addresses normalised. This is narrow evidence for one compiler, flag set and target. It does not guarantee that undefined behaviour is preserved, and it is not provenance for the final binary. The Linux targets that CI builds have no such comparison.

## Evidence

### Source equivalence

[`expand-check.py`](../reference/m5-loss-structural/expand-check.py) runs from the repository root as `python -I eech-campaign/reference/m5-loss-structural/expand-check.py 163e9158`. Its docstring states exactly what it compares and how both sides are normalised. It checks:
- that `gp_loss.c` defines exactly the expected helpers;
- that each helper, with its parameters mapped to its call's arguments, is the statements it replaced;
- that, with every call expanded, each of the 67 destroy and kill functions in the six files, and the rest of each file, equals the base token for token.

Results are recorded in the evidence commit:
- [`expand-check.txt`](../reference/m5-loss-structural/expand-check.txt), the full output;
- [`selftest.txt`](../reference/m5-loss-structural/selftest.txt), six mutations, each applied alone, checked and restored: a reordered unlink, an added per-type consequence, a changed helper body, swapped helper parameters, a file-scope redefinition and `DEBUG_MODULE 1`.

The script does not compare the types of local declarations. The build's own type checks cover those.

### Candidate build, public-path checks and regression pack

The evidence commit records the candidate's build identity (built from the candidate commit with a clean tree, in a new cargo target volume), the probe and the M4 pack.

**Why a new volume.** `eech-engine-sys/build/main.rs` declares no rerun trigger for `aphavoc/` or `modules/`, so an existing volume silently reuses stale C objects. This was observed in this session: a rebuild in a used volume finished in 0.37 s with the previous DLL.

**The earlier candidate.** The pack also passed, unchanged and run by the lead, on the earlier working candidate `wip2`: `eech_dc.dll` `daea0ddf…`, 40 of 40 checks, Georgia and Lebanon IDENTICAL to `windows-retail v1` (Jira EEDCS-11 comment 10449). That candidate was revised after the pre-commit audits, which:
- put the `MOVEMENT_DEPENDENT` calls back inline;
- renamed a helper;
- edited comments.

So its results are kept as history only: [`wip2/`](../reference/m5-loss-structural/wip2/).

## Remaining coupling (explicit, not removed)

**In the six destroy files, the world still does campaign work:**
- **Helper calls:** it calls all nine helpers.
- **Other campaign calls:** `remove_from_force_info` after `get_local_force_entity`, `add_entity_to_regen_queue` (reading the group's sub-type), `release_mobile_entity_landing_locks` and `notify_campaign_screen` (presentation).
- **Sector values:** `remove_mobile_values_from_sector` (sectors are campaign).
- **Helicopter player paths:** the player's mission scoring and log (`notify_gunship_entity_mission_terminated`, `inc_player_log_missions_flown`, `update_player_log_mission_success_rate`), and clearing `WEAPONS_HOLD`/`POSITION_HOLD` on the player's group members.
- **Ships:** the carrier keysite search, and killing that keysite.
- **The anti-aircraft restore path:** its client branch reads the group's member count and kills an empty group.
- **Inline in the person file:** the group-loss count and the task notification.
- **Movement dependents:** destroying them with their carrier (`destroy_local_family`) and unlinking them (above).

**Outside these six files:** world files that edit campaign lists or send loss-related messages are not in this slice.

**Not separated yet.** The campaign still reads physical state through member pointers. Loss is still decided and applied inside one DLL, synchronously, in the middle of the update. Host identities, a host report, missing-host failure and causal loss through the public path are later, separately authorised work.

## Reproduce

```sh
python -I eech-campaign/reference/m5-loss-structural/expand-check.py 163e9158      # PASS
cd eech-campaign && EECH_WIN_TARGET=<new volume> tools/build-windows.sh target/<dir> # a new volume, always
<dir>/eech-world.exe reference/m5-host-boundary/probe.lua root=<Lebanon root copy> seconds=1800 observe=1 out=A.jsonl diag_at=1580,1590,1795 2> A.stderr
python tools/regression-pack.py --georgia <root> --lebanon <root> --lebanon-repeat <root> --bin <dir> --out <out>
```

**Rollback:** revert the commits. No baseline, lineage or pack inventory depends on them.
