# M5: who owns what in the SUPPLY loop

This record is part of M5 ([EEDCS-5](https://flying-dice.atlassian.net/browse/EEDCS-5)), Task [EEDCS-10](https://flying-dice.atlassian.net/browse/EEDCS-10). It answers one question: **in the accepted Lebanon SUPPLY loop, which state, decisions and rules belong to the campaign, which physical attempts and observations belong to the host world, and which single dependency should be characterised first?**

**Answer, from the source and the retained M3 evidence:**
- **The campaign owns the loop's state, decisions, task policy, logistics and progression.** That covers keysite and group supply levels, the request threshold, supplier choice, the task and its waypoint plan, group selection, which member must arrive and how members' arrivals combine, the pick-up and drop-off accounting, task success, and the loss bookkeeping. Campaign algorithms that consume environmental data (route search over terrain, crate layout over 3D bounds, an ETA from a position) stay campaign algorithms.
- **The host world owns physical attempts and the observations they produce:** where each aircraft is and how it moved, whether it is landed or spinning up, whether it has been destroyed, and the terrain and 3D data.
- **Arrival is not a single physical verdict.** It is a campaign rule applied to per-member physical state. The campaign picks the current waypoint, sets its criteria (reached radius, last-to-reach), chooses the required member(s) (the task leader, or every valid member) and combines their answers (`check_guide_reached_waypoint`, `guide.c:260-336`). Only the per-member physical test (range against the radius, pass-through, landed; for helicopters also rotor spin-up) is world authority, and even that function holds some campaign rules today (see stage 4).
- **Recommended first seam: arrival at the four SUPPLY task waypoint types** (PICK_UP, PREPARE_FOR_DROP_OFF, DROP_OFF, FINISH_DROP_OFF). For these, the source's rule is *task leader only* (stage 4). Every other waypoint and action path, including the rest of a SUPPLY task's route, is named below as still coupled.

Nothing here is a contract, an ABI or a callback shape. This is a map from which EEDCS-11 can be refined. No code, script, baseline or campaign rule changed.

## Evidence labels

| Label | Meaning |
|---|---|
| **[R-run]** | Reproduced from runtime evidence: observed through the public path (`eech-world.exe` → `lua/campaign.lua` → `require ("eech_dc")` → `engine:objects ()`) in the retained M3 artifacts (`reference/m3-resupply-loop/`, `reference/m3-supply-loss/`), and re-read in those artifacts for this record. No new run was made. |
| **[R-src]** | Reproduced from source: the cited original C says this. Lines are of the unpatched tree in `aphavoc/source/`, identical at `a02d489c` and at the current integration head `d2d4792a` (`git diff --stat a02d489c d2d4792a -- aphavoc` is empty). Where a build patch changes the code, the patch is named. This is a statement about the code, not a runtime observation. |
| **[I]** | Inferred from [R-run] and [R-src] together, not directly seen. |
| **[U]** | Unknown. Not established by the source read or the evidence; left open rather than guessed. |

## The loop, step by step

Each step names the campaign side, the host/world side, and what crosses between them. "Crosses" means the observation the host supplies or the action the campaign requests, *in ownership terms*. Today nothing crosses a host boundary: both sides are compiled into `eech_dc.dll` and share one entity system (see "Coupling inside `eech_dc.dll`").

### 1. Supply need

| | |
|---|---|
| Campaign | Keysite supply levels change on a one-minute keysite tick by `keysite_database` usage rates (`ks_updt.c:126-146`; airbase −0.2 ammo, −0.4 fuel per tick), clamped to [10, 100]. A consuming keysite at or below 75 sends `FORCE_LOW_ON_SUPPLIES` to its force (`keysite.c:469-495`). Crates are a derived representation of the level, rebuilt each tick (`keysite.c:337-467`). [R-src] |
| Campaign | Groups draw supply only when a member's DEFEND or LANDED waypoint is reached: `response_to_waypoint_defend_reached` (`mb_msgs.c:779`, draw at `:897`) and `response_to_waypoint_landed_reached` (`mb_msgs.c:1308`, draw at `:1457`) call `assess_group_supplies` (`group.c:619-765`). The draw is a transfer between two campaign scalars. [R-src] |
| Host / world | Keysite position, and the 3D bounding box of `OBJECT_3D_SINGLE_CRATE`, read to lay crates out on the ground (`keysite.c:375-385`). Elapsed time. [R-src] |
| Crosses | Elapsed time (host → campaign; today `engine:frame (ms)`, the only host input). Crate layout is a campaign algorithm consuming 3D-object data. Group draws depend on LANDED/DEFEND arrivals, which are **outside** the first slice (stage 4). [R-src] |
| Time | `get_delta_time ()` drives both timers. The request repeats every tick while the level is at or below 75; nothing on the keysite remembers it asked. [R-src] |

### 2. Supplier and task

| | |
|---|---|
| Campaign | The force's response (`fc_msgs.c:670-893`) refuses a request when the requester already has a SUPPLY task for that cargo type (`fc_msgs.c:720-745`). It picks a factory or refinery, or the nearest airbase if closer (S1 patched), and creates the task (`fc_msgs.c:846-848`). `create_supply_task` (`taskgen.c:1638-1725`) builds PICK_UP at the crate (dependent: supplier), PREPARE_FOR_DROP_OFF 4 km short, DROP_OFF at the requester (dependent: requester) and FINISH_DROP_OFF 2 km past (`taskgen.c:1685-1712`). It records only the cargo *type* (`taskgen.c:1718`), not the crate. The task waits 20 minutes for assignment (`taskgen.c:1696`; `ts_updt.c:105-119`). [R-src] |
| Host / world | Positions of requester, supplier and crate; map bounds (`bound_position_to_adjusted_map_area`). [R-src] |
| Crosses | Positions (observation). Supplier choice and the start-keysite score (`task.c:1168-1186`) are campaign algorithms consuming them. [R-src] |

### 3. Assignment

| | |
|---|---|
| Campaign | Every three minutes a keysite assigns its unassigned support tasks (`ks_updt.c:107-122`, `assign.c:96-300`) to an idle group based at that keysite (`assign.c:401-516`), keeping the force's minimum idle count. `assign_primary_task_to_group` assigns with `TASK_ASSIGN_ALL_MEMBERS` (`assign.c:553`; `0xffffffff`, `taskgen.h:80`). `assign_task_to_group` builds the route (`assign.c:927` → `croute.c:200-826`), pushes the guide with that valid-members mask (`assign.c:943`) and attaches each valid member as a follower (`assign.c:945`, `957-990`). Route search inserts NAVIGATION waypoints (`croute.c:437`, `687`). [R-src] |
| Host / world | The first member's position and cruise speed (ETA gate, `group.c:245-330`); terrain elevation, road nodes and map bounds (route search). [R-src] |
| Crosses | Positions, cruise speed and terrain/road data (observations). The route search itself is campaign behaviour that consumes terrain (PROJECT.md: ownership is not decided by data source). Helicopter fuel load is a campaign decision from the route's flight time (`assign.c:983`, `1326-1355`) applied to a physical quantity: a requested world action [I]. Takeoff-route locks and landing-site reservations (`mb_msgs.c:160-420`): whether these are campaign records of keysite capacity or host airfield operation is **not decided here** [U]. |

### 4. Movement and arrival

This step is where the previous revision of this record (`add1d5f`) said "everything upstream of `gd_nav.c:79` is a physical test". That was wrong. The source divides it as follows.

| Part | Owner | Source | Label |
|---|---|---|---|
| Flying toward the guide; writing each aircraft's 2D range to its guide position into `FLOAT_TYPE_DISTANCE` | Host / world | `hc_move.c:1264`, `fw_move.c:1410` | [R-src] |
| Which waypoint is current; setting the guide's target position and velocity from it | Campaign | `set_guide_new_waypoint`, `guide.c:401-541`; `set_guide_next_waypoint`, `guide.c:547` | [R-src] |
| The waypoint's criteria: reached radius, last-to-reach flag, transmit-recon flag, copied onto the guide from `waypoint_database` per waypoint type and mobile type | Campaign (task policy data) | `initialise_guide_criteria`, `guide.c:1453-1529`, called on every waypoint change (`guide.c:538`) and group attach (`guide.c:829`) | [R-src] |
| **Who must arrive.** The task leader is the group leader if it follows this guide, else the first follower (`PTR_TYPE_TASK_LEADER`, `gd_ptr.c:144-175`, recomputed on every read). If `GUIDE_CRITERIA_LAST_TO_REACH` is set, every group member whose bit is in `INT_TYPE_VALID_GUIDE_MEMBERS` must arrive; otherwise only the task leader | Campaign (group/task criteria) | `check_guide_reached_waypoint`, `guide.c:260-336` (leader `:275`, last-to-reach branch `:282-316`, leader-only branch `:317-333`) | [R-src] |
| **How answers combine.** All required members must pass; the first failure ends the check for that frame | Campaign | `guide.c:296-314` | [R-src] |
| Per-member physical test, fixed wing: not landed; range against the guide's radius; else a pass-through test over the last frame's motion | Host / world, applying a campaign-supplied radius | `response_to_check_mobile_reached_guide`, `fw_msgs.c:390-520` | [R-src] |
| Per-member physical test, helicopter, task leader: not landed, rotor spun up, range against radius; altitude and heading only when those criteria are set | Host / world, applying campaign-supplied criteria | `hc_msgs.c:529-765` (spin-up `:589`, radius `:600`) | [R-src] |
| Rules embedded in the helicopter test: a player's aircraft passes any waypoint whose type is `player_skip_waypoint` (`hc_msgs.c:560-570`); a **non-leader** passes a last-to-reach waypoint by being within 750 m of the *leader*, not of the waypoint (`hc_msgs.c:527`, `766-788`) | Campaign rules (player policy; group cohesion) inside a world-side function | as cited | [R-src]; the classification is [I] |
| The WAYPOINT_ACTION path: a separate leader-only test against the action radius, for waypoints with "Check Waypoint Action" | Campaign rule over a physical range | `gd_updt.c:98-103`; `check_guide_waypoint_action_reached`, `guide.c:342-395`; `gd_msgs.c:139-165`; `wp_msgs.c:232-279` | [R-src] |
| The verdict entering campaign accounting: guide `WAYPOINT_REACHED` (`gd_updt.c:105-108`) → navigation guides call `navigation_guide_waypoint_reached` (`gd_msgs.c:188-197` → `gd_nav.c:79-93`) → `response_to_waypoint_reached` (`wp_msgs.c:285-445`). Attack guides go elsewhere (`gd_msgs.c:204-277`) | Campaign | as cited | [R-src] |

**What the SUPPLY waypoint types select.** PICK_UP (`wp_dbase.c:1112-1165`), PREPARE_FOR_DROP_OFF (`wp_dbase.c:1058-1110`), DROP_OFF (`wp_dbase.c:410-462`) and FINISH_DROP_OFF (`wp_dbase.c:572-624`) all have: guide sub-type `NAVIGATION_DIRECT`; no waypoint action message and "Check Waypoint Action" FALSE; "Criteria Last To Reach" FALSE for both fixed wing and helicopter; reached radius 100 m (fixed wing) and 40 m (helicopter). `NAVIGATION_DIRECT` enables no guide-database criteria (`gd_dbase.c:86-101`), so the only criterion is the waypoint's radius. [R-src]

So **for the four SUPPLY task waypoints the campaign's rule is "the task leader has reached", not "all required members have reached".** [R-src] The helicopter non-leader branch is not used there; the rotor spin-up gate and `player_skip_waypoint` (FALSE for these four types) are. [R-src]

**The rest of a SUPPLY task's route is different.** Helicopter NAVIGATION waypoints are last-to-reach (`wp_dbase.c:1032`, in the helicopter block that starts at `:1027`), as are helicopter Holding, Insert Troops and Prepare for Insertion (`wp_dbase.c:654`, `1842`, `2112`). For ground vehicles, Defend, the troop pick-up start and end points, the troop put-down point and Wait are last-to-reach (`wp_dbase.c:394`, `1960`, `1906`, `2014`, `2068`). An earlier reply on #74 (`01f4519`) said last-to-reach was on only for ground vehicles at the troop pick-up points; the helicopter entries above show that was incomplete. Route search inserts NAVIGATION waypoints (stage 3). A two-member helicopter SUPPLY group such as Wolfpack therefore uses all-valid-members arrival on its navigation legs and leader-only arrival at its four task waypoints [I]. Which waypoint types a given SUPPLY route actually contains at runtime is [U]: nothing counts them.

**What the evidence can and cannot distinguish.** Jester, the featured M3 chain, is a one-member fixed-wing group (unit 88236, Il-76MD; it is the only member in `chain-jester.jsonl`) [R-run]. For a one-member group, leader-only and all-members semantics give the same verdict, so that chain does not distinguish them [I]. Wolfpack (two Mi-17s) picked up at Beirut at 1,043.79 s while it had two members (`m3-supply-loss/loss-chain.json`) [R-run]. Which of the two was task leader at that moment is not reported [U].

| | |
|---|---|
| Crosses (first slice, SUPPLY task waypoints only) | *Observation, host → campaign:* whether the physical state of a **campaign-named member** (the task leader) satisfies the reach geometry for the **campaign-named target and radius**. The campaign keeps: choosing the waypoint, the radius, the required member, the combination rule, and every consequence. Whether the host also owns the pass-through and spin-up refinements, or reports raw state for the campaign to test, is a hypothesis for EEDCS-11 [U]. |
| Still coupled (outside the first slice) | The all-valid-members path and the helicopter non-leader cohesion test; `player_skip_waypoint`; the WAYPOINT_ACTION path (Attack, Lower Undercarriage, Recon: `wp_dbase.c:149`, `959`, `1229`; `:959` is the Lower Undercarriage entry, whose action message is `WAYPOINT_LOWER_UNDERCARRIAGE_ACTION`, not Lift Off as `01f4519` said); attack-guide arrivals (`gd_msgs.c:204-277`); the RECON and CAP_LOOP special cases (`wp_msgs.c:339-391`); vehicle and ship tests (`vh_msgs.c:366-380`); and every non-SUPPLY-task waypoint on the SUPPLY route, including LANDED (whose arrival drives group draws, stage 1). [R-src] |
| Identity | The task leader is recomputed on every read (`gd_ptr.c:144-175`; the member's `INT_TYPE_TASK_LEADER` flag is derived from it, `ac_int.c:995-1010`). `response_to_waypoint_reached` reads the leader (`wp_msgs.c:313`), advances the guide (`:413`) and notifies the force (`:432`) *before* notifying the leader (`:440`); the source warns the notification can destroy the task (`wp_msgs.c:404-405`) and the guide (`gd_nav.c:92`). [R-src] |
| Time | The guide check runs once per frame on the server against whatever range the aircraft last wrote. The fixed-wing pass-through uses the frame delta. Whether guides update before or after their followers in a frame is [U]. |

### 5. Pick-up, drop-off and completion: campaign accounting

| | |
|---|---|
| Campaign | `response_to_waypoint_reached` sends the waypoint's reached message **only to the task leader** (`wp_msgs.c:438-441`). The force's `FORCE_WAYPOINT_REACHED` handler does nothing (its counting is commented out, `fc_msgs.c:978-1014`). [R-src] |
| Campaign: pick-up | `response_to_waypoint_pick_up_reached` (`mb_msgs.c:1857`) takes a crate from the supplier (S3 selects one of the task's type; the original takes the first, `:1877-1882`). With no crate, it returns and the task carries on empty (`:1885-1888`). The supplier's level drops by 10 (`ks_msgs.c:281`, `289`). The crate moves onto the **leader's** `MOVEMENT_DEPENDENT` list (`mb_msgs.c:1893-1897`). [R-src] |
| Campaign: drop-off | `response_to_waypoint_drop_off_reached` (`mb_msgs.c:906`) walks only the **receiving leader's** `MOVEMENT_DEPENDENT` list (`:927-1032`). A keysite receiver gets the crate and its level set to 100 whatever it was (`ks_msgs.c:230`, `238`). A group receiver's level rises, and the crate stays aboard (`mb_msgs.c:975-1008`). The physical crate drop is commented out (`mb_msgs.c:1033-1075`). [R-src] |
| Campaign: completion | FINISH_DROP_OFF completes the task with `TASK_TERMINATED_OBJECTIVE_MESSAGE` (`mb_msgs.c:1096-1112`), which is SUCCESS for SUPPLY (`task.c:435-455`) **whether or not anything was delivered**. Points, reactionary tasks and return to base follow (`ts_msgs.c:261-365`). [R-src] |
| Host / world | None needed for the accounting. [R-src] |
| Crosses | *Actions, campaign → host:* open cargo doors on every member at PREPARE_FOR_DROP_OFF (`mb_msgs.c:1822-1843`) and close them on the leader at FINISH_DROP_OFF (`mb_msgs.c:1108`). They are requested world actions with no effect on the accounting [I]. |

**Correlation with the retained evidence.** In `chain-jester.jsonl` [R-run]: at 1,555.67 s Jester is *Performing Task* and, in the same sample, Halat's fuel goes 89.6 → 79.6; at 1,904.58 s Beirut's fuel goes 10.0 → 100.0. That matches the pick-up (−10 per crate) and keysite drop-off (set to 100) rules within one 1 s sample [I]. The run's pick-ups were up to 336 m from the keysite position (`m3-resupply-loop.md`) [R-run], which fits a PICK_UP placed at the crate, laid out in a grid around the keysite (`keysite.c:383-385`), rather than at the keysite itself [I]. Neither the arrival verdict, the task leader, the task identity nor the crate is in the observations, so the link from a specific arrival to a specific level change is attributed by position and time, not observed [R-run].

### 6. Loss

| | |
|---|---|
| Host / world | Combat resolution and damage decide that the aircraft died. `kill_local` (`hc_dstry.c:316-720`; fixed wing `fw_dstry.c`) marks it dead and removes it from its guide's followers (`hc_dstry.c:503`). [R-src] |
| Campaign, interleaved with that teardown | `remove_from_force_info` (`hc_dstry.c:358`), the regen queue (`:410`), group and task losses (`:594`, `:608`), `MOBILE_KILLED` to the group (`:623`), `CHECK_CAMPAIGN_OBJECTIVES` to every force (`:639`). [R-src] |
| Campaign: group response | On `MOBILE_KILLED` the group removes the member from guides with a **restricted** valid-members mask, terminating the task when none remain (`gp_msgs.c:1335-1378`, `guide.c:870-923`). A guide assigned with `TASK_ASSIGN_ALL_MEMBERS`, as SUPPLY is, is skipped (`gp_msgs.c:1353`): the task continues and the survivor becomes leader by recomputation. [R-src] Wolfpack shrank to one aircraft at the loss (1,584.66 s) and flew its later SUPPLY tasks with one [R-run]. |
| Campaign: cargo | A crate aboard a dead aircraft stays on it until the wreck is removed, then is destroyed with it (`hc_dstry.c:277`, `290`). The supplier is never credited back. [R-src] |
| Crosses | *Observation, host → campaign:* "member M was destroyed". Today the campaign consequences run inside `kill_local`, so this observation has no single entry point [R-src]. |

### 7. Afterwards

Delivered stock is campaign state that the campaign's own rules then use [R-run, `m3-resupply-loop.md`]: draws by landed groups (stage 1, driven by LANDED arrivals outside the first slice), later pick-ups from the same keysite (stage 5), and the next request at 75 (stage 1).

## Ownership map

| Step | Campaign (state, decisions, policy, logistics, progression) | Host / world (physical attempt, observation) | What crosses | Label |
|---|---|---|---|---|
| Supply levels, threshold, crates | owns levels, rates, threshold, crate records and layout rule | keysite position, crate 3D bounds | time; 3D-object data | [R-src] |
| Supplier choice, task creation | owns choice, task, waypoint plan, dependents, cargo type | positions, map bounds | positions | [R-src] |
| Group selection, route | owns idle-group policy, ETA gate, route search, valid-members mask | member position, cruise speed, terrain, roads | positions, speed, terrain/road data; fuel load requested | [R-src]; fuel [I] |
| Airfield slots and locks | — | — | not classified | [U] |
| Flight | owns the guide's target, velocity and timing | flight model, movement, range to guide | position/range (observation) | [R-src] |
| **Arrival rule** | owns current waypoint, criteria, required member(s), combination | — | — | [R-src] |
| **Arrival geometry** | supplies member, target, radius | member's position/motion against target and radius; landed; spin-up | per-member reach observation | [R-src]; boundary shape [U] |
| Pick-up, drop-off, completion | owns all accounting and success | none | cargo-door actions out | [R-src] |
| Loss | owns losses, regen, objectives, valid-members, task outcome | decides the aircraft died | destruction observation | [R-src] |
| Time | timers in campaign units | frame delta from the host clock | `engine:frame (ms)` | [R-src] |

**The public path today.** The host can only boot, step time and read objects: `crates/eech-dc/src/module.rs` exports `boot` (`:43`, `:61`) and the methods `frame`, `objects`, `clock` and `diagnostics` (`:20-29`). **No host-owned value reaches the campaign except elapsed time.** [R-src] Any host authority therefore needs a new, host-owned input on that path; this record does not choose its form.

## Coupling inside `eech_dc.dll`

Stated so that no later slice mistakes a separation for one that exists. All [R-src] unless marked.
- **One process, one entity system.** Campaign and reference world are compiled into the same DLL and share entities. The aircraft entity carries campaign links (group member, guide follower, cargo) and flight-model state (position, motion, `DISTANCE`) side by side. Every link is a raw pointer or an intrusive list.
- **The guide straddles the line.** It holds the campaign's plan (current waypoint, criteria, valid members) and the physical target position the flight model steers to.
- **Campaign code writes physical fields.** `set_guide_new_waypoint` writes the leader's `FLOAT_TYPE_DISTANCE` (`guide.c:509`), the field the flight model otherwise writes (`hc_move.c:1264`, `fw_move.c:1410`); attaching a member writes 32 km (`guide.c:801`).
- **World-side handlers read campaign data and hold campaign rules.** The per-member tests read guide criteria and `waypoint_database` directly, and the helicopter test holds player-skip and leader-cohesion rules (stage 4).
- **Shared random stream.** The weapon-vector branch of both per-member tests calls `frand1 ()` (`fw_msgs.c:505`, `hc_msgs.c:743`); it is not reached at SUPPLY waypoints [R-src]. Whether the campaign and world share one random sequence elsewhere in the loop is [U].
- **Ordering.** The consequences run synchronously inside the arrival check's frame, with the guide advanced before the leader is notified (stage 4). Whether entity update order within a frame matters to the verdict is [U].
- **Loss is not separable today.** Campaign bookkeeping is interleaved with world teardown inside `kill_local` (stage 6).

## Invisible records

Unobservable through the public path. None blocks choosing the seam, but EEDCS-11's evidence must not depend on them silently. [R-src], and confirmed absent from the M3 observation fields [R-run].

| Record | Status |
|---|---|
| Crates (keysite stock and aboard aircraft) | not reported; derived from levels by code |
| A task's supplier, receiver, cargo type and identity | on waypoint dependents and task user data; not reported |
| A guide's current waypoint, criteria, valid members and task leader | not reported |
| `FORCE_LOW_ON_SUPPLIES` requests | messages; not reported |
| Group supply levels | not reported |
| Arrival verdicts themselves | not reported; inferred from position plus a level change in the same sample |

## Choosing the first seam

| Candidate | Host would supply | Causal test available? | Breadth | Verdict |
|---|---|---|---|---|
| **A. Arrival at the four SUPPLY task waypoints** | the per-member physical reach observation for the campaign-named task leader | Yes: withhold or delay it and predict supply levels | one aggregation point (`check_guide_reached_waypoint`) restricted to four waypoint types; the embedded test is the reference provider's answer | **Recommended** (lead accepted arrival as the first seam to characterise, #74 review at `add1d5f`) |
| B. Position query (ETA, supplier search) | member and keysite positions | Weak | many readers of `VEC3D_TYPE_POSITION` | Later |
| C. Loss report | "member M destroyed" | Yes, but bookkeeping is interleaved with teardown | needs `kill_local` split | Second slice |
| D. Route geometry | terrain, roads | Indirect | wide | Later |

**Why A, and why narrowed.** It is the narrowest point where a physical outcome becomes campaign state, and it is causally testable through observed keysite levels. Narrowing to the four SUPPLY task types keeps one combination rule (leader-only) and leaves the all-valid-members path, the helicopter cohesion rule and the action path in the campaign untouched. Whether EEDCS-11 later takes other waypoint types through the same boundary is a scope decision for the lead, not made here.

**Requirements the boundary must meet** (for EEDCS-11; the mechanism is not chosen here):
- **Campaign policy stays in the campaign.** Waypoint selection, criteria, required-member selection and combination remain EECH_DC behaviour. The host must not decide which member counts or how answers combine, and an adapter must not grant arrival to compensate for missing behaviour.
- **Synchronous-equivalent.** Today the verdict is consumed inside the frame, before the guide advances. A later verdict would change ordering and, at 100 ms frames, timing: a baseline difference to classify, not a refactor. Callbacks and reentrancy are unverified (`M1.lifecycle.unverified` in the pack); any mechanism is a hypothesis to test.
- **Generation-safe identity.** The observation concerns a member, a guide and a waypoint, all recomputed or recreated in EECH; the leader is recomputed each read and a task can be destroyed mid-notification. References need generations (as `eech_kernel.h` already does for entities) and a lifetime rule.
- **Failure contract.** With no provider, or a failing one, the result must be a named error or a stated "not reached", never EECH's embedded answer.

## First-seam acceptance gates (from the lead's review of `add1d5f`)

Recorded here so EEDCS-11 starts from them. None is met by this record.
1. **Readiness first.** After #73 and the repository protection blocker are resolved, characterise the named current baseline before restructuring. Preserve the M4 pack and rebuilt `windows-retail v1` Georgia/Lebanon `-Exact` runs; classify every meaningful difference under REVIEW.md without replacing the baseline silently.
2. **Public path, causal.** Exercise host-owned Lua → public EECH_DC → host observations. Show reference-provider equivalence, a withheld PICK_UP, a delayed DROP_OFF and missing-provider behaviour. The controlled observations must actually determine the campaign consequences, with no hidden embedded answer; an interface declaration or an unchanged recording is insufficient.
3. **Correlation.** Correlate the selected task, waypoint generation, required member(s), arrival and supply effect. `engine:objects ()` and the current chain checker expose none of those identities, cargo or task links; add bounded observational evidence where needed, rather than treating proximity or a whole-keysite level as task-specific proof.
4. **Ordering and lifetimes.** Preserve ordering and task/guide lifetimes, and count the supported arrival path at runtime.
5. **Completion is not delivery.** The source can mark SUPPLY successful without cargo (stage 5). Do not fix that, or the other source facts below, in the structural slice.

**Proposed causal test (not built).**
1. *Reference provider* answers with EECH's own per-member test. Prediction: M4 pack unchanged; `windows-retail v1` holds exactly. [I]
2. *Withheld PICK_UP* for one chosen task's leader. Prediction: the supplier's level does not drop at the overflight; no delivery on that task; the task stays assigned and the receiver's re-request is refused while it does (`fc_msgs.c:720-745`); whatever ends the task comes from the campaign's own timers. [I]
3. *Delayed DROP_OFF.* Prediction: the receiver's 10 → 100 moves to the later time and the task completes later. [I]
4. *No provider.* Prediction: the declared failure, not the embedded answer. [I]

Steps 2 and 3 control a genuine physical observation, not a campaign decision. The required member stays the campaign's choice in all four.

## Wider dependencies beyond this loop

Before claiming the campaign hostable (EEDCS-12), these must move behind the boundary or be named as remaining coupling:
- **Movement:** guide-following and the flight model (`hc_move.c`, `fw_move.c`), and the helicopter fuel load set at assignment.
- **Arrival outside the first slice:** listed in stage 4.
- **Airfield operations:** takeoff and landing routes, slots and queues (`mb_msgs.c:160-420`); ownership [U].
- **Combat and damage:** kills and losses (`kill_local`), the regen queue, campaign objectives.
- **Geography:** positions in supplier search, the ETA and sector lookup; terrain and roads consumed by route search; 3D bounds consumed by crate layout.
- **Time:** `get_delta_time ()` everywhere; the host clock is already the only time source (`docs/engine.md`).
- **Process-global state:** for example the function-static `task_timer` (`ks_updt.c:192`) and the static idle-group counts (`assign.c:116-117`), with the inventory in `crates/eech-sys/global-state.txt`.

## Source facts found on the way

Read in source, not confirmed at runtime, and not changed here. Each is a candidate for `docs/corrections.md` review, not an M5 change.
- **A keysite drop-off delivers only the first crate aboard.** Moving the crate off `MOVEMENT_DEPENDENT` clears its successor, so the loop ends (`mb_msgs.c:943`, `1031`; `en_list.c:570-583`). Its type is not checked against the task's cargo. [R-src]
- **SUPPLY success does not depend on delivery** (`mb_msgs.c:1096-1112`, `task.c:435-455`). [R-src]
- **Cargo is leader-bound.** Pick-up loads the leader; drop-off searches only the receiving leader. If the leader carrying the crate dies, or leadership otherwise changes, the new leader carries nothing, yet the task can still succeed. [I]
- **The duplicate check's walk ignores task state** (`fc_msgs.c:720-745`). A completed task of the same cargo, still linked, can block a new one. [I]

## Open questions

- Why Wolfpack's survivor turned back at the moment of the loss (`m3-supply-loss.md`). [U]
- Whether every SUPPLY-task arrival passes `check_guide_reached_waypoint` → `navigation_guide_waypoint_reached` at runtime. The only other `WAYPOINT_REACHED` sender found, `ac_basmv.c:159`, is in `basic_aircraft_movement`, which has no caller and is not in the engine's source list (`crates/eech-engine-sys/sources.txt`) [R-src]. Runtime completeness is [U]; EEDCS-11's reference run must count it.
- Which waypoint types a SUPPLY route contains at runtime, and who was task leader at each multi-member arrival. [U]
- Ownership of airfield slots and takeoff/landing locks. [U]

## Validation

- **Scope:** documentation only. No runtime file, script, binary or baseline changed, and no tracing was added.
- **This revision:** every stage 4-6 citation, and every citation added or changed here, was re-read against `aphavoc/source/` on the PR branch (identical to `d2d4792a`); three line numbers from `add1d5f` were corrected (`wp_msgs.c:404-405`, `taskgen.c:1696`, `fc_msgs.c:720-745`). Stage 1-3 citations carried from `add1d5f` were spot-checked (`fc_msgs.c`, `ks_msgs.c`, `task.c`, `taskgen.c`, `assign.c`), not each re-read. The M3 retained artifacts were re-read for the Jester and Wolfpack facts. No new run.
- **M4 pack, retained tier:** `python tools/regression-pack.py --retained-only` (Python 3.13.6, Windows) gives `M4.s3_detected` PASS and `governance.lineage` PASS (`windows-retail v1`, unchanged); 28 runtime claims NOT RUN, overall INCOMPLETE (retained tier only), as expected.
- **Runtime tier: not run.** It needs a Windows host and the retail roots. Nothing it checks could change from a documentation edit.
