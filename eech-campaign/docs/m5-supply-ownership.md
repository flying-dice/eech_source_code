# M5: who owns what in the SUPPLY loop

This record is part of M5 ([EEDCS-5](https://flying-dice.atlassian.net/browse/EEDCS-5)), Task [EEDCS-10](https://flying-dice.atlassian.net/browse/EEDCS-10). It answers one question: **in the accepted Lebanon SUPPLY loop, which state and rules belong to the campaign, which facts and actions belong to the physical world, and which single dependency should be handed to a host first?**

**Answer, from the source and the retained M3 evidence:**
- **The campaign owns the loop's state and rules.** That covers keysite and group supply levels, the request threshold, supplier choice, the task and its route plan, group selection, the pick-up and drop-off accounting, task success, and the loss bookkeeping.
- **The physical world decides when things happen.** It decides when an aircraft has *reached* a waypoint, where every aircraft is, whether one is destroyed, and the terrain and 3D data that routes and crate layouts use.
- **The two meet in one place for arrival.** Every "this waypoint has been reached" verdict enters campaign accounting through one call, `navigation_guide_waypoint_reached` (`gd_nav.c:79`). Everything upstream of it is a physical test; everything downstream is campaign rule.
- **Recommended first seam: the arrival verdict.** A host-supplied arrival, the campaign applying its own pick-up, drop-off and completion rules unchanged. A causal test for EEDCS-11 is proposed below.

Nothing here is a contract or an ABI. This is a map from which EEDCS-11 can be refined. No code, script, baseline or campaign rule changed.

## Sources and how to read this record

| Mark | Meaning |
|---|---|
| **[O]** | Observed through the public path in the retained M3 evidence (`reference/m3-resupply-loop/`, `reference/m3-supply-loss/`) |
| **[S]** | Read in the original C at `a02d489c`. Line numbers are of the unpatched tree in `aphavoc/source/`; where a build patch changes the code, the patch is named |
| **[I]** | Inferred from [O] and [S] together, not directly seen |

No new run was made, and no tracing was added. Where the source is ambiguous, it is listed under "Open questions" rather than resolved by guesswork.

## The loop, stage by stage

### 1. Supply need

| | |
|---|---|
| Campaign state and rule | Keysite supply levels change on a one-minute keysite tick by `keysite_database` usage rates (`ks_updt.c:126-146`; airbase −0.2 ammo, −0.4 fuel per tick). They are clamped to [10, 100]. Crates are a derived representation of the level, rebuilt each tick (`keysite.c:337-467`). A consuming keysite at or below 75 sends `FORCE_LOW_ON_SUPPLIES` to its force (`keysite.c:469-495`). [S] |
| Physical reads | The keysite position, and the 3D bounding box of `OBJECT_3D_SINGLE_CRATE`, used only to lay crates out on the ground (`keysite.c:375-385`). [S] |
| Group requests | Groups request and draw supply only when a member reaches a DEFEND or LANDED waypoint (`group.c:619-765`, called from `mb_msgs.c:897` and `:1457`). The draw is a transfer between two campaign scalars. [S] |
| Time | `get_delta_time ()` drives both timers. The request repeats every tick while the level is at or below 75; nothing on the keysite remembers it asked. [S] |

### 2. Supplier and task

| | |
|---|---|
| Campaign rule | The force's response (`fc_msgs.c:670-893`) refuses a request when the requester already has a SUPPLY task for that cargo (`fc_msgs.c:722-745`). It then picks a factory or refinery, or the nearest airbase if that is closer (S1 patched), and creates the task (`fc_msgs.c:846-848`). |
| Task | `create_supply_task` (`taskgen.c:1638-1725`) builds PICK_UP at the crate (dependent: supplier), PREPARE_FOR_DROP_OFF 4 km short, DROP_OFF at the requester (dependent: requester) and FINISH_DROP_OFF 2 km past. It records only the cargo *type* (`taskgen.c:1718`), not the crate. The task waits 20 minutes for assignment (`ts_updt.c:105-119`). [S] |
| Physical reads | Positions of the requester, supplier and crate, as approximate 2D ranges. The start keysite is chosen by a score that falls to nothing at 100 km from the crate (`task.c:1168-1186`). [S] |

### 3. Assignment

| | |
|---|---|
| Campaign rule | Every three minutes a keysite assigns its unassigned support tasks (`ks_updt.c:107-122`, `assign.c:96-300`) to an idle group *based at that keysite* (`assign.c:401-516`). The force must keep its minimum idle count of that group type. |
| Physical reads | The first member's position and cruise speed give an ETA, which must beat the task's remaining expiry (`group.c:245-330`). [S] |
| What assignment creates | `assign_primary_task_to_group` (`assign.c:524`) → `assign_task_to_group`. That builds the waypoint route (`assign.c:927` → `croute.c:200-826`), creates the guide and attaches the group (`assign.c:943`; `guide.c:115-200`, `808-860`), and attaches each member as a follower (`assign.c:945`, `957-990`). [S] |
| Physical reads and actions | Route search over terrain elevation and road nodes, with map bounds (`croute.c`). Helicopter fuel load is set from the route's flight time (`assign.c:983`, `1326-1355`). Members get takeoff-route locks and landing-site reservations (`mb_msgs.c:160-420`). [S] |

### 4. Movement and arrival: where physical authority decides

| | |
|---|---|
| Physical | Each aircraft flies toward its guide's position and writes its own 2D range to the guide into `FLOAT_TYPE_DISTANCE` (helicopter: `hc_move.c`; fixed wing: `fw_move.c`). [S] |
| The arrival test | Every frame, on the server, the guide asks whether it has been reached (`gd_updt.c:105-108` → `check_guide_reached_waypoint`, `guide.c:260`). That asks the task leader (or every member, for last-to-reach waypoints) through `CHECK_MOBILE_REACHED_GUIDE`. Fixed wing: range against the waypoint's reached radius, with a pass-through test over the last frame's motion (`fw_msgs.c:389-470`). Helicopters add altitude and heading tests (`hc_msgs.c:529-700`). The reached radius is 100 m for fixed wing and 40 m for helicopters at both PICK_UP and DROP_OFF (`wp_dbase.c:1112-1140`, `410-440`). [S] |
| Where the verdict enters the campaign | `navigation_guide_waypoint_reached` (`gd_nav.c:79-93`) sends `WAYPOINT_REACHED` to the current waypoint. `response_to_waypoint_reached` (`wp_msgs.c:285`) then: looks up the waypoint's reached message (`:327`), advances the guide (`:413`), notifies the force (`:432`), and **notifies only the task leader** with `PICK_UP_REACHED`, `DROP_OFF_REACHED` and so on (`:440`). [S] |
| Identity | The task leader is not stored; it is recomputed on every read (`gd_ptr.c:144-175`). The guide advances before the leader's response runs, and the source warns that this can destroy the task (`wp_msgs.c:396-398`). [S] |
| Time | The test runs once per frame against whatever range the aircraft last wrote, so the order of entity updates matters. The fixed-wing pass-through test uses the frame delta. [I] |

### 5. Pick-up and drop-off: campaign accounting

| | |
|---|---|
| Pick-up | `response_to_waypoint_pick_up_reached` (`mb_msgs.c:1857`) takes a crate of the task's type from the supplier (S3; the original takes the first crate). With no crate, the task carries on empty (`:1885-1889`). The supplier's level drops by 10 (`ks_msgs.c:281`, `289`), and the crate moves onto the leader aircraft's `MOVEMENT_DEPENDENT` list (`mb_msgs.c:1893-1897`). [S] |
| Drop-off | `response_to_waypoint_drop_off_reached` (`mb_msgs.c:906`). A keysite receiver gets the crate and its level set to 100 whatever it was (`ks_msgs.c:230`, `238`). A group receiver's level rises per crate, and the crate stays aboard (`mb_msgs.c:975-1010`). [S] |
| Completion | Reaching FINISH_DROP_OFF completes the task with `OBJECTIVE_MESSAGE` (`mb_msgs.c:1112`), which counts as SUCCESS for SUPPLY (`task.c:435-455`). Points, the force's reactionary tasks and the group's return to base follow (`ts_msgs.c:261-365`). [S] |
| Physical | None. These rules run on the arrival verdict alone. [S] |

**Correlation with the retained evidence.** Jester's chain [O]: Halat −10 in the sample Jester is 85 m over Halat (1,555.67 s), and Beirut 10 → 100 in the sample it is 43 m over Beirut (1,904.58 s). That matches the pick-up (−10 per crate) and keysite drop-off (set to 100) rules above, within one 1 s sample. The run's pick-ups were up to 336 m from the keysite's position [O]. That fits a pick-up waypoint placed at the *crate*, which is laid out in a grid around the keysite (`keysite.c:383-385`), rather than at the keysite itself [I].

### 6. Loss: physical outcome, campaign bookkeeping

| | |
|---|---|
| Physical | Damage kills the aircraft. `kill_local` (`hc_dstry.c:316-720`; fixed wing `fw_dstry.c`) marks it dead and unlinks it from its guide and group. [S] |
| Campaign bookkeeping, interleaved with that teardown | `remove_from_force_info` (`:358`), the regen queue (`:410`), group and task losses (`:594`, `:608`), `MOBILE_KILLED` to the group (`:623`), and `CHECK_CAMPAIGN_OBJECTIVES` to every force (`:639`). [S] |
| Group response | A group that has lost all its members is killed, and its task fails. Otherwise a SUPPLY task, assigned with `TASK_ASSIGN_ALL_MEMBERS` (`assign.c:553`), keeps its guide, and the survivor becomes leader by recomputation (`gp_msgs.c:1340-1360`). [S] Wolfpack shrank to one aircraft and was re-tasked at 3,619.24 s. [O] |
| Cargo | A crate aboard a dead aircraft stays on it until the wreck is removed, then is destroyed with it (`hc_dstry.c:277`, `290`). The supplier is never credited back. [S] |

### 7. Afterwards

Delivered stock is campaign state that the campaign's own rules then use [O, `m3-resupply-loop.md`]. Draws by landed groups (stage 1), later pick-ups from the same keysite (stage 5), and the next request when it falls to 75 (stage 1) are all campaign rules.

## Ownership map

| Concern | Campaign policy | Physical-world authority | Host translation today |
|---|---|---|---|
| Supply levels, threshold, crates | owns them | crate layout uses 3D bounds | none (all inside `eech_dc.dll`) |
| Supplier choice, task creation | owns them | positions and ranges | none |
| Group selection | owns it | member position, cruise speed (ETA) | none |
| Route | owns the waypoint plan and dependents | terrain, roads, map bounds, flight time | none |
| Takeoff, flight, landing | owns the task, guide and timing | flight model, keysite routes, landing slots | none |
| **Arrival at a waypoint** | applies pick-up, drop-off and completion | **decides it was reached** (`gd_updt.c` → `guide.c:260`) | none |
| Cargo aboard | accounts for it | carried on the aircraft entity | none |
| Loss | losses, regen, objectives, task outcome | decides the aircraft died | none |
| Time | timers in campaign units | frame delta from the host clock | the host's `engine:frame (ms)` is the only input |

**Co-resident state.** Campaign and world share one entity system in one process.
- The aircraft entity carries campaign links (group member, guide follower, cargo) and flight-model state (position, motion, `DISTANCE`) side by side.
- The guide and waypoint entities straddle the line: they hold the campaign's plan and the physical target position.
- Every link is a raw entity pointer or an intrusive list. Destroying a supplier, requester or crate can leave route dependents dangling. [S]

**The public path today.** The host can only boot, step time and read objects (`crates/eech-dc/src/module.rs`: `boot`, `frame`, `objects`, `clock`, `diagnostics`). **No host-owned value reaches the campaign except elapsed time.** Any host authority therefore needs a new, host-owned input on that path.

## Invisible records

These remain unobservable through the public path. None of them blocks choosing the seam, but EEDCS-11's evidence must not depend on them silently.

| Record | Status |
|---|---|
| Crates (keysite stock and aboard aircraft) | not reported; derived from levels by code [S] |
| A task's supplier, receiver and cargo type | on waypoint dependents and task user data; not reported [S] |
| `FORCE_LOW_ON_SUPPLIES` requests | messages; not reported [S] |
| Group supply levels | not reported [S] |
| Arrival verdicts themselves | not reported; inferred from position plus a level change in the same sample [O, I] |

## Choosing the first seam

| Candidate | What the host would supply | Causal test available? | Breadth of change | Verdict |
|---|---|---|---|---|
| **A. Arrival verdict** (`gd_nav.c:79` / `wp_msgs.c:285`) | "Guide G's leader has reached its current waypoint" | Yes: withhold, delay or alter a host verdict and predict the supply levels | One call site; the physical test stays as the reference provider | **Recommended** |
| B. Position query (ETA, supplier search) | Member and keysite positions | Weak: a different position changes an ETA gate or a nearest-keysite choice, rarely observable | Many readers of `VEC3D_TYPE_POSITION` | Later |
| C. Loss report (`MOBILE_KILLED`) | "Member M of group G was destroyed" | Yes, but the bookkeeping is interleaved with teardown inside `kill_local` | Needs `kill_local` split and a cargo rule EECH lacks | Second slice |
| D. Route geometry | Terrain, roads, flight time | Indirect | Wide (route search) | Later |

**Why A.**
- **It is the narrowest point where a physical outcome becomes campaign state.** Upstream (range, radius, altitude, pass-through) is physics. Downstream (pick-up, drop-off, completion, guide advance) is campaign rule and stays in EECH_DC unchanged.
- **It is causally testable through the public path** with evidence M3 already uses: the supplier's −10 and the receiver's 100 are observed keysite levels.
- **The reference host can supply it faithfully.** EECH's own arrival test becomes the EECH World reference provider's answer, so `windows-retail v1` can be preserved.

**Constraints the boundary must respect** (for EEDCS-11, not decided here):
- **Synchronous.** EECH consumes the verdict inside the frame, before the guide advances. A verdict delivered one frame later changes ordering and, at 100 ms frames, timing. That would be a baseline difference to classify, not a refactor. A query-shaped provider, asked during the frame, is the faithful shape, as `docs/ports.md` found for positions. Callbacks and reentrancy are currently unverified (`M1.lifecycle.unverified` in the pack).
- **Identity.** The verdict names a guide or waypoint and its task leader. Both are recomputed or recreated in EECH, so the boundary needs stable references with generations (as `eech_kernel.h` already does) and a lifetime rule for a task destroyed mid-notification.
- **Failure contract.** If no provider is registered, or the provider fails, the result must be a named error or a stated "not reached". The embedded test must never answer silently.
- **Scope.** Arrival is used by every waypoint type, not only SUPPLY. EEDCS-11 must either take all types through the provider (preferred: one rule) or state which types stay embedded.

## Proposed causal test for EEDCS-11

The test, not yet built:
1. **Reference provider.** The host answers each arrival query with EECH's own test. Prediction: the M4 pack is unchanged, and `windows-retail v1` holds exactly.
2. **Withheld verdict.** For one chosen SUPPLY task's PICK_UP, the host answers "not reached". Prediction: the supplier's level does not drop at the overflight, and no delivery reaches the receiver on that task. The task stays assigned, and the receiver's re-request is refused while it does (`fc_msgs.c:722-745`). Whatever ends the task comes from the campaign's own timers.
3. **Moved verdict.** For a chosen DROP_OFF, the host answers "reached" at a chosen later time. Prediction: the receiver's 10 → 100 moves to that time, and the task completes later.
4. **No provider.** Prediction: the declared failure, not EECH's embedded answer.

Steps 2 and 3 control a genuine physical input, not a campaign decision. The consequences are read through `engine:objects ()` and checked by the existing chain checker.

## Wider dependencies beyond this loop

Before claiming the campaign hostable (EEDCS-12), these must move behind the boundary or be named as remaining coupling:
- **Movement:** guide-following and the flight model (`hc_move.c`, `fw_move.c`), and the helicopter fuel load set at assignment.
- **Airfield operations:** takeoff and landing routes, slots and queues (keysite 3D objects, `mb_msgs.c:160-420`).
- **Combat and damage:** kills and losses (`kill_local`), the regen queue, campaign objectives.
- **Geography:** positions in supplier search, the ETA and sector lookup; terrain and roads in route search; 3D bounds in crate layout.
- **Time:** `get_delta_time ()` everywhere; the host clock is already the only time source (`docs/engine.md`).
- **Process-global state:** for example the function-static `task_timer` (`ks_updt.c:192`) and the static idle-group counts (`assign.c:116-117`), with the inventory in `crates/eech-sys/global-state.txt`.

## Source facts found on the way

All were read in source, not confirmed at runtime, and are not changed here. Each is a candidate for `docs/corrections.md` review, not an M5 change.
- **A keysite drop-off delivers only the first crate aboard.** Moving the crate off `MOVEMENT_DEPENDENT` clears its successor, so the loop ends (`mb_msgs.c:932-1031`, `en_list.c:570-583`). Its type is not checked against the task's cargo.
- **SUPPLY success does not depend on delivery.** FINISH_DROP_OFF completes it with SUCCESS whether or not anything was dropped (`mb_msgs.c:1112`, `task.c:435-455`).
- **Losing the leader can lose the cargo.** If the leader carrying the crate dies, the survivor carries nothing, yet the task can still succeed [I, from the above].
- **The duplicate check's second walk ignores task state** (`fc_msgs.c:722-745`). A completed task of the same cargo, still linked, can block a new one [I].

## Open questions

- Why Wolfpack's survivor turned back at the moment of the loss (`m3-supply-loss.md`) is not explained by the source read here.
- A runtime confirmation that every arrival passes `gd_nav.c:79`. The only other `WAYPOINT_REACHED` sender found, `ac_basmv.c:159`, is in `basic_aircraft_movement`, which has no caller and is not in the engine's source list (`crates/eech-engine-sys/sources.txt`) [S]. EEDCS-11's reference run should still count verdicts through the new boundary.

## Validation

- **Scope:** documentation only. No runtime file, script, binary or baseline changed, and no tracing was added, so non-perturbation holds trivially.
- **M4 pack, retained tier:** `tools/regression-pack.py --retained-only` at this commit gives `M4.s3_detected` PASS, with the lineage `windows-retail v1` verified and unchanged.
- **Runtime tier: not run.** It needs a Windows host and the retail roots, which this environment does not have. Nothing it checks could change from a documentation edit.
