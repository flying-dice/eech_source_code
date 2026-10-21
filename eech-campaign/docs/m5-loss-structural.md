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

**Observed codegen evidence.**
- `x86_64-w64-mingw32-objdump -d --no-show-raw-insn -r` was run on `kill_local` from `ps_dstry.o`, in two cargo target volumes: the #59 build's `eech-win-target-pr59` (source `65892de1`) and this candidate's `eech-win-target-m5hb-c2` (`78dc796b`).
- With addresses normalised ([`ps-kill_local-base.s`](../reference/m5-loss-structural/ps-kill_local-base.s), [`ps-kill_local-candidate.s`](../reference/m5-loss-structural/ps-kill_local-candidate.s)), the first 106 lines are identical. That includes the `TASK_COMPLETED` call at line 94 and its argument setup.
- The first difference is at line 107, after it ([`ps-kill_local.diff`](../reference/m5-loss-structural/ps-kill_local.diff)): the unlink helpers and `MOBILE_KILLED`.

This is narrow evidence for one compiler, flag set and target. It does not guarantee that undefined behaviour is preserved, and it is not provenance for the final binary. The Linux targets that CI builds have no such comparison.

## Evidence

### Source equivalence

[`expand-check.py`](../reference/m5-loss-structural/expand-check.py) runs from the repository root as `python -I eech-campaign/reference/m5-loss-structural/expand-check.py 163e9158 78dc796b` (base, then the candidate revision; without a candidate argument it reads the working tree). Its docstring states exactly what it compares and how both sides are normalised. It checks:
- that `gp_loss.c` defines exactly the expected helpers;
- that each helper, with its parameters mapped to its call's arguments, is the statements it replaced;
- that, with every call expanded, each of the 67 destroy and kill functions in the six files, and the rest of each file, equals the base token for token.

Results (Python 3.13):
- [`expand-check.txt`](../reference/m5-loss-structural/expand-check.txt): 9 helpers SAME and 67 functions SAME, PASS.
- [`selftest.txt`](../reference/m5-loss-structural/selftest.txt): seven mutations, each applied alone, checked and restored, and each reported: a reordered unlink, an added per-type consequence, a changed helper body, swapped helper parameters, a file-scope redefinition, `DEBUG_MODULE 1`, and an unlisted helper. The restored tree passes.

The portable [`eech-loss-structural` workflow](../../.github/workflows/eech-loss-structural.yml) checks the pinned slice 1a extraction `78dc796b` against base `163e9158` (not the PR head; see "Revision boundary") and runs [`expand-check-selftest.py`](../reference/m5-loss-structural/expand-check-selftest.py) with the same pinned candidate. The current selftest applies twelve mutations, each alone, to an isolated temporary Git tree and requires the checker to reject each one: the seven above, plus a `DEBUG_MODULE` block with an `#else`, conditional compilation in `gp_loss.c`, a missing helper, a helper with an extra parameter, and an added world function. It matches each expected diagnostic as a whole output line, not a substring, requires exit status 1 and no `PASS`, reads the `DEBUG_MODULE` alternative-branch rejection, which exits through `SystemExit`, from stderr, and requires that line to be all of stderr (every other case requires empty stderr), compares each restored file with the candidate's source bytes recorded before any mutation, and finally requires the restored candidate to pass again. Adding a function also adds its signature to the file-scope text, so that case reaches the shared function-list/file-scope rejection line without separating the two conditions. This guards the equivalence claim for slice 1a. The sensitivity run is unchanged by pinning: the mutations are applied to isolated copies of the pinned sources, and the isolated tree's own base is the named `163e9158` sources. The full Windows M4 pack still uses private retail roots and is retained as the local run below.

#### Revision boundary

The token-for-token claim holds for `78dc796b` and no later revision. Reviewed semantic cuts after it change `gp_loss.c` and the six `*_dstry.c` files on purpose, so the checker deliberately fails on them. The first was `3a1da638` (forward-only loss observation seam), then `157905a3` (withheld bounded physical loss) and the logical-membership cut (`feaafaa5`) on branch `codex/m5-physical-loss`. At the last of those it reports `FAIL: 8`: the helper set gains `check_all_force_campaign_objectives` and `increment_task_losses`, `increment_group_primary_task_losses` differs, and `kill_local` differs in all six files. No normalisation was added to hide this.

The CI job therefore runs the checker and its selftest at the pinned `78dc796b` against `163e9158` (both fetched by SHA), which reproduces [`expand-check.txt`](../reference/m5-loss-structural/expand-check.txt) line for line and keeps all twelve mutations detected. It is a historic proof of the slice 1a extraction, and its path triggers still cover the files it concerns. It says nothing about the current head.

For the current head, the behaviour is defined by the reviewed semantic change ([`m5-physical-loss.md`](m5-physical-loss.md)). Hosted CI has no automated test of forward, withhold, duplicate-report or lifetime behaviour: `cargo test` (`eech-campaign` workflow) has no test that reaches `observe_loss` or `report_loss`, and the host-identity and supply-arrival jobs check retained hashes and runner inputs only. That behaviour is checked by the Windows retail-host lifecycle runs (`tools/supply-arrivals-lifecycle-windows.ps1`, `tools/host-identities-lifecycle-windows.ps1`), the receipt run and the M4 pack, recorded in `m5-physical-loss.md`. They are local, not CI.

The script does not compare the types of local declarations. The build's own type checks cover those.

### Candidate build, public-path checks and regression pack

**Build.** The candidate is commit `78dc796b`, with a clean tree.
- **How it was built:** `tools/build-windows.sh`'s own build step, run in the pinned image `sha256:0c60c75d…` (rustc 1.98.1, MinGW GCC 12.2.0), with the new cargo target volume `eech-win-target-m5hb-c2`.
- **Why not the script as is:** after this machine restarted, its `docker build` step rebuilt `eech-build` as another image (`bb10e44a…`, with a different `lua.dll`). That build was discarded. The script was run from a copy that skips `docker build` and names the pinned image; no other line changed.
- **Identity** ([`BUILD-INFO.txt`](../reference/m5-loss-structural/BUILD-INFO.txt)): `commit: 78dc796b…`, `uncommitted changes in the build inputs: 0 files`. `eech_dc.dll` is `de1a52f9…`, and `lua.dll` and the Lua scripts are identical to the #59 build's.

**Probe.** The [proposal's](../reference/m5-host-boundary/README.md) `probe.lua` was run through `eech-world.exe` → `require ("eech_dc")`.
- **Inputs:** retail Lebanon, seed 1, 100 ms frames, 1,800 s, on a private root copy. The copy was IDENTICAL to `reference/lebanon-3h/inputs.json` before and after.
- **Result:** Mi-17 87568 dies at **1,584.66 s** and Wolfpack drops to 87581 ([`probe-A.jsonl`](../reference/m5-loss-structural/probe-A.jsonl)).
- **Diagnostics:** the campaign diagnostics at 1,580, 1,590 and 1,795 s equal the retained diag-A hashes of the `65892de1` candidate ([`probe-diag-sha256.txt`](../reference/m5-loss-structural/probe-diag-sha256.txt)).

**M4 regression pack.** Unchanged, run on the candidate's binaries; finished 2026-10-21 08:51 London.
- **Command:** `python -I tools/regression-pack.py --georgia E:/eech-runs/roots/georgia --lebanon E:/eech-runs/m2/lebanon-root --lebanon-repeat E:/eech-runs/m2/lebanon-m1-copy --bin E:/eech-runs/m5-host-boundary/c1/bin --out E:/eech-runs/m5-host-boundary/pack-1a-c1`
- **Result: PASS, 40 of 40 checks** ([`report.md`](../reference/m5-loss-structural/report.md), [`report.json`](../reference/m5-loss-structural/report.json)):
  - Georgia and Lebanon are **IDENTICAL** to `windows-retail v1`;
  - the three roots match their manifests;
  - the M2 recording is reproduced byte for byte, the two reference runs are byte-identical, and observed equals unobserved;
  - the M3 resupply chains and the loss chain hold, including `M3.supply_loss_reproduced`.
- **About the label:** the report says "with uncommitted changes" because this record and its script were being edited during the run. None of those files is a pack input or a build input.
- **Baseline:** unchanged, with no difference to classify.

**Tacview.** The pack generated and structurally checked its recordings, and reproduced M2's byte for byte. They were not visually inspected.

**Red/green.** None is claimed. This is a pure structural move, and the characterisation was in place before it: the proposal's A/B probe on the `65892de1` candidate and the accepted M4 pack. No red was manufactured.

**CI-equivalent checks** ([`linux-test.txt`](../reference/m5-loss-structural/linux-test.txt)): `cargo test --locked --target x86_64-unknown-linux-gnu` with gcc-12, in the pinned image, on a read-only mount of the candidate tree. Exit 0, 23 tests passed, none failed.
- **Not run locally:** `cargo fmt` and `cargo clippy` (not installed in the pinned image; no Rust file changed), the i686 and C-optimisation-level matrix entries, and the Wine job. CI runs them on publication.

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

**Movement dependents.** They are physical carriage, not campaign work, but campaign keysites and crates ride on them. The world destroys them with their carrier (`destroy_local_family`) and unlinks them inline (above).

**Outside these six files:** world files that edit campaign lists or send loss-related messages are not in this slice.

**Not separated yet.** The campaign still reads physical state through member pointers. Loss is still decided and applied inside one DLL, synchronously, in the middle of the update. Host identities, a host report, missing-host failure and causal loss through the public path are later, separately authorised work.

## Reproduce

```sh
python -I eech-campaign/reference/m5-loss-structural/expand-check.py 163e9158 78dc796b  # PASS at the extraction; the head fails by design
cd eech-campaign && EECH_WIN_TARGET=<new volume> tools/build-windows.sh target/<dir> # a new volume, always
<dir>/eech-world.exe reference/m5-host-boundary/probe.lua root=<Lebanon root copy> seconds=1800 observe=1 out=A.jsonl diag_at=1580,1590,1795 2> A.stderr
python tools/regression-pack.py --georgia <root> --lebanon <root> --lebanon-repeat <root> --bin <dir> --out <out>
```

**Rollback:** revert the commits. No baseline, lineage or pack inventory depends on them.
