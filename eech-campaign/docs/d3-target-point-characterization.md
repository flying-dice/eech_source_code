# D3: original model-derived target-point observer (EEDCS-52)

**Status (2 November 2026): SOURCE-ONLY PREPARATION. Observer, hooks, plumbing, strict driver and local guards are written; nothing here has been formatted, built, run, reviewed or measured.** No target-point responsibility has moved, no host
query exists, no production code path was touched, and the shipping build contains none of it. Task EEDCS-52 (parent EEDCS-21) is stacked above checked Task 50 (`bce0c3514`); Task 50's ordinary shipping validation is still running and
is not accepted, so this does not claim that dependency delivered. Scope authority is the live M5 goal (Confluence 11534337): the physical world moves to the host; campaign policy, consequences, geography, routes, timers and scheduling stay in
EECH_DC. D3 is active; D4 and D5 are unstarted.

## 1. What the original does (verified in source; unmeasured)

`get_local_entity_target_point (en, &point)` (`aphavoc/source/entity/en_misc/en_maths/en_maths.c:166`) answers four ways:

| branch | condition | result | writes |
|---|---|---|---|
| 1 | the entity has a 3D instance with a `TARGET_POINT` sub-object | the sub-object's world position | **overwrites `inst3d->vp` (position, then attitude) from the entity pose** before composing the world viewpoint |
| 2 | an instance without one | the entity position | none |
| 3 | no instance, `INT_TYPE_IDENTIFY_FIXED`, shape not `OBJECT_3D_INVALID_OBJECT_INDEX` | position, then `target_point_offsets[shape]`: **x and z only when the offset has a non-zero x or z AND the heading is non-zero** (rotated by the heading); `y` is always added | none |
| 4 | no instance and not a fixed object, or a fixed object with the invalid shape | the entity position | none |

**The heading-zero quirk is legacy behaviour and stays**: a fixed shape with a horizontal offset and a heading of exactly zero gets its `y` offset but not its x/z offsets. It is recorded and checked as a contract of the report, not corrected.

`target_point_offsets[]` is filled once, at boot, by `initialise_target_point_offsets` (`wn_dbase.c:6748`, called from `wn_funcs.c:105`): per model index, the world position of the model's `TARGET_POINT` sub-object, or, when it has none, the bounding-box
centre with `y = max (centre y, 0)`; the invalid index is zero. These offsets are **physical model-derived facts** and belong to the host under D3. The same function first calls `calculate_seeker_limits_from_launch_angle`, a separate campaign/weapon-policy
calculation that **stays Core, in its order and executed once**; the function must not be moved as one piece, and the observer's census hook sits before that call and changes nothing about it.

### Callers

| caller | reached how | observed |
|---|---|---|
| `weapon.c` `fill_aim_target` (line 957) | the host aim operation's `vectors`, `lead` and `intercept` modes; the **only** place the aim code asks | yes, by aim mode |
| `wn_move.c` weapon step, `CAMPAIGN_STEP_LOOKUP_TARGET_VEC` question 1 | the host weapon-step operation | yes |
| `weapon.c:748` `get_ballistic_intercept_point_and_angle_of_projection` | inside `#if 0`: compiled out, never runs | no (dead) |
| `wn_msgs.c` (weapon-explosion camera), the `cm_*.c` cameras, `ah_hud.c` | player view and HUD | not hooked; counted as `unattributed` if the headless campaign ever reaches them |
| `gd_attck.c:172` | AI attack guidance **reads `target_point_offsets[]` directly**, not through the function | not observed: a second consumer of the table that a host answer must also serve |

## 2. The observer (`target-point-fixture`)

A test-only cargo feature (`eech-dc/target-point-fixture`), off in every shipping and every other fixture build, built by `tools/build-windows-target-point-fixture.sh` (the unchanged `build-windows.sh` with one added cargo argument, the same wrapper pattern as the
aim, route and other fixtures). It follows the route fixture's mechanism: hooks are **staged patches** (`TARGET_POINT_FIXTURE_PATCHES` TP1..TP20 in `build/patches.rs`, strict occurrence-count checked), applied only with the feature to the staged copies of
`en_maths.c`, `wn_dbase.c`, `weapon.c` and `wn_move.c`. **The in-tree sources are not edited**, so with the feature off the staged files are the shipping ones and no hook, definition or entry exists. Other files: `csrc/eech_tp_hooks.h`, `csrc/eech_tp_fixture.c`,
the three entries in `csrc/eech_engine.c` (appended after the existing text; the file's literal NUL byte is untouched) and `eech_engine.h`, `eech-engine-sys` externs, `Engine::target_point_summary / target_point_offsets / target_point_event`, the Lua methods of the same names, the driver
`lua/target-point-characterization.lua` and its local guards `tests/target_point_characterization.rs`.

What each hook does: it **adds a call and changes no original statement, argument, order or value**, does no floating-point arithmetic (floats are copied as bit patterns), draws no random number, reads no clock, keeps no pointer on the wire and writes no campaign or model byte.
The query hooks record: the caller (TP18/TP20) and its detail; the branch (TP3/TP4, and the no-instance default of 4 promoted to 3 by TP6 for a valid shape); the entity type; the host identity; the semantic model (the instance's scene, or the shape's, by compiled scene name); the shape; whether a horizontal
offset exists (TP7), the heading the original read (TP8) and whether the rotated offsets were added (TP9); the returned point (TP10); and the **instance viewpoint, position and attitude (twelve words), before and after**. The census hooks (TP12..TP16) record, per model index, which producer
branch filled the entry (sub-object, bounding box with its six bounds, or the invalid index's zero) and the offset bits; at report time each entry is compared with the live table, so a later write to it shows as `live=0`.

**Host identity is read, never allocated.** The fixture calls `campaign_object_host_id`, which returns the identity the host already issued (a structure's, or a rostered mobile's) and never asks the host. `campaign_object_identify`, which the aim path's `fill_host_id` uses and which *asks the host for
a structure's first identity*, is deliberately not used, so the observer cannot issue an identity as a side effect. `host_id_state` is `present` or `absent` (an object with none yet); the identity text is the last field of the line and may hold spaces. This helper validates the entity heap and, like the aim
path that calls it first, would stop the engine on an invalid entity; that is the only extra campaign function the fixture calls. It gives host-identity *availability* only; it does not decide an answer shape for a host query.

### Reports and the driver

Three read-only entries, bounded and refused rather than truncated: the **summary** (counters, a branch x caller matrix, the case counts, the census totals and two verdicts), the **census** (one row per model index), and **one recorded query** by index (capacity 4096; calls beyond it are counted as `dropped` and
make the capture not whole). The driver boots exactly like `campaign.lua`, reads the summary and census straight after the boot (the census must be whole before any frame runs), runs `seconds` (default 3.6) in `frame_ms` (default 100) frames, samples the summary at the requested cadence, reads the summary and
the census again, requires the census byte-equal, and reads every recorded query once. It **selects no target, creates no entity and calls the original from nowhere**: only queries the campaign's own update loop makes are reported.

Safeguards, each exercised by a local guard with the real driver: strict argument checking before any side effect and an exclusive output closed on every exit; the native grammar line by line and the summary's arithmetic (every begun call is exactly one of stored, dropped, abandoned or open); a verdict is believed only if
the counters say the same, both ways; the event rows must reproduce the summary's branch and caller matrix; a monotone counter, a clock that advances, and a census that does not change; a query of branch 3 must agree with its own offset and heading (`xz_applied` exactly when the heading is non-zero, `+0.0` and `-0.0` alike); a
branch 2 call must not change the instance viewpoint; sequence numbers strictly increase; an engine call that fails, a malformed report or a module without the feature (`NOT RUN`) stops the questions and is never a zero-success.

**No query reached is stated, not hidden**: a whole capture with no query passes with `natural_queries_reached=false`, `branches_reached=none`, an `unreached` row and a standard-error line; nothing is claimed covered.

## 3. How Root runs it (not yet done)

1. Build on a clean checkout: `sh tools/build-windows-target-point-fixture.sh target/target-point-fixture/<dir>` (a fresh, uniquely named `EECH_WIN_TARGET` volume).
2. `eech-world lua/target-point-characterization.lua root=<fresh retail root> out=<new file> scenario=lebanon_retail` (defaults `seconds=3.6 frame_ms=100`, 36 frames) and again with `georgia_retail`, each in a fresh process on a fresh root with the input manifest checked before and after. Then, **separately labelled as natural longer runs**, `seconds=60` and
   `seconds=600` (the D2 aim witness reached 112,621 pitch-device solves in 600 s on Lebanon). The shipped-profile build's census is the model-host profile's bounding boxes and sub-objects; it is not claimed equal to the original-model-accessor profile.
3. Compare nothing against an oracle: record the census, the branches and callers reached, and the bits. A short run may reach no query; that is the finding, not a failure to make coverage by forcing a target.

## 4. Unproven and not covered

* **Nothing has run.** The C, the patches' anchors against a real staged tree, the Rust, the Lua driver and its guards are unbuilt and unrun here; static counts of every anchor in the four sources were checked (each occurs exactly once) and nothing else. The guards use C-shaped *synthetic* reports and prove the driver, not the observer.
* **Natural reach is unknown.** Which branches the 3.6 s controls reach, and whether any query is reached at all, is Root's first measurement. Branches 3 and 4 depend on targets having **no** 3D instance, which in a headless campaign is the common case far from any view, but that is unmeasured. Branch 1 and 2 need an instance.
* **Controlled coverage is not implemented.** If the natural runs leave a branch unreached, the options are an already established aim scenario (the D2 person-case pattern, `lua/aim-person-case.lua`) or a separately labelled controlled option that calls the original on a chosen entity. The second would *perturb the campaign* (branch 1 overwrites the instance viewpoint, which this observer
  deliberately does not restore) and would claim neither natural coverage nor restoration; it is not built, and no geometry is invented for it.
* **Downstream readers of the written viewpoint are not proven.** The sibling code in `weapon.c` (lines 1343, 2116, 2340, 2501, 2613, 2931) writes `inst3d->vp` from the entity pose (the same write branch 1 makes) at each of those sites, evidently so that the viewpoint reads that follow see the pose (whether every one of them is preceded by such a write was not checked), and `get_sub_object_relative_position` (`sub_anim.c`) subtracts the instance position from a world position, so it is
  base-independent; about 50 references to `get_3d_sub_object_world_viewpoint` exist in all (gunship avionics and cockpits, dynamics), and a reader of `inst3d->vp` that does not refresh it first has not been ruled out. The report records whether the write *changes* the words (`vp_changed`), which is the evidence that bears on it; a host answer that does not write the viewpoint is a semantic claim to be justified after that
  measurement, not assumed.
* **FPU and ordering.** No RNG is visible in the function. Whether the caller's FPU state or the update order matters needs the actual runs; the hooks run inside the engine entry under the same mode and add no arithmetic. The observer's call to `campaign_object_host_id` runs before the original body, outside its FPU use. TP8 takes the address of the local `heading` so the hook can copy its bits; on the x86-64 SSE single-precision target that can change code generation but not a value, and that is argued here, not shown for any other target.
* **Not observed**: the camera, HUD and player callers (counted only as unattributed), `gd_attck.c`'s direct table read, the population of `target_point_offsets[]` under the original model accessors (this feature composes with the host model profile that ships), and any lazily constructed model effect (the census records what the boot already computed; it constructs and measures no model).
* **No host answer shape is chosen, no extraction is made, and no baseline, tolerance, input, scenario, flag or oracle changed.** D3/Task 52 stay open until a real original characterization, an extraction, the actual authority and regression comparisons, review, current CI and normal delivery.
