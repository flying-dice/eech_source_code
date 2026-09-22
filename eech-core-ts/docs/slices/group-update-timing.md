# Slice 2: group update timing (issue #3)

**Status:** investigation complete, boundary selected. Freezing waits on the
conformance gates at the end of this document.

The investigation is recorded first, because issue #3 makes it a gate before
the boundary is fixed.

## Investigation 1: `gp_updt.c :: update_server`

```c
if (raw->sleep > 0.0)        { raw->sleep -= get_delta_time ();        raw->sleep = max (raw->sleep, 0.0f); }
if (raw->assist_timer > 0.0) { raw->assist_timer -= get_delta_time (); raw->assist_timer = max (raw->assist_timer, 0.0f); }
if ((raw->sleep == 0.0) && (raw->assist_timer == 0.0)) delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);
```

- **Sleep and assist timer.** Two independent float countdowns. A timer is only
  decremented while it is `> 0.0`, so a negative value is never touched. After a
  decrement it is clamped with `max (x, 0.0f)`.
- **`max` comes from the platform.** It is not defined anywhere in the EECH
  repository: it is the Windows SDK macro `(((a) > (b)) ? (a) : (b))`. For a NaN
  timer it yields `0.0f`, where `Math.max` / `fmax` would not. The port must use
  the macro's expression.
- **`get_delta_time ()`** is the macro `(system_delta_time)` from
  `modules/system/time.h`. The update loop may have overridden it with a
  sub-step (investigation 2).
- **At or past the boundary.** A timer that reaches or crosses 0 becomes exactly
  0. When **both** timers are 0 (including one that was never set, or was
  negative and so never decremented and never 0), the group removes itself from
  `LIST_TYPE_UPDATE`. With a negative timer the group stays on the list for
  ever, and that is EECH's behaviour.
- **Downstream effects.** Expiry causes no guide, task or AI change. The only
  side effect is the list removal, which notifies:
  - `UNLINK_CHILD` to the update entity. `up_msgs.c` fixes the loop's successor
    pointer.
  - `UNLINK_PARENT` to the group. `gp_msgs.c :: response_to_unlink_parent` only
    calls `debug_log_entity_message`.
- **Entering the list.** This happens in `gp_float.c :: set_local_float_value`:
  setting `FLOAT_TYPE_SLEEP` or `FLOAT_TYPE_ASSIST_TIMER` to a non-zero value,
  when the group is not already on the update list, calls
  `insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL)`.
  That inserts at the **head** and notifies:
  - `LINK_CHILD` to the update entity: C default response, because `up_msgs.c`
    only overloads it under `DEBUG_MODULE`.
  - `LINK_PARENT` to the group: `gp_msgs.c :: response_to_link_parent`, which
    only acts for `LIST_TYPE_DIVISION`.
- **Readers of the timers are out of scope.** Task assignment, and the
  `mb_msgs.c` landing handler that sets `sleep` after rearming, are separate
  campaign behaviour. There is no natural boundary to cut *inside*
  `update_server`, because it is already self-contained.

**Classification.**
- Campaign core: the timers, the update-list membership and the list
  notifications.
- Environmental: the frame delta.
- Not ported: the debug logging.

## Investigation 2: the update-list semantics

`en_updt.c` only holds the dispatch table `fn_update_client_server_entity[type][comms_model]`,
whose default is a no-op. The loop is `update/up_update.c :: update_client_server_entities`,
called by the host flight loop (`flight.c`) **once per time-acceleration step**
each frame. It is called zero times while paused, and `set_delta_time ()` runs
at the end of each frame.

- **Traversal.** The loop walks `LIST_TYPE_UPDATE` under the update entity from
  its first child. Before each entity's update it stores the successor in the
  global `update_succ`, then continues from `update_succ`.
- **Ordering.** List order. Insertion is at the head, so an entity inserted
  during a pass is not visited in that pass. An entity that removes itself is
  safe, because its successor was saved beforehand. If the successor itself is
  unlinked, the update entity's `UNLINK_CHILD` response advances `update_succ`.
- **Frame subdivision.** When the frame rate is not locked, the loop calls
  `set_entity_update_frame_rate (command_line_entity_update_frame_rate)`. That
  asserts `1 <= rate <= 100`, computes
  `iterations = (int) (get_delta_time () * rate + 1.0)`, and sets the delta to
  `get_delta_time () / iterations` through `set_manual_delta_time`. The loop then
  makes `iterations` passes over the list, and finally restores the frame delta
  with `set_manual_delta_time (delta_time)`.
  - With a locked frame rate there is exactly one pass, and `set_manual_delta_time`
    is refused.
  - The rate is the EECH.INI setting "entity update frame rate". Its default is 2
    (`cmndline.c`).
- **`INT_TYPE_UPDATED`.** After each entity update the loop sets
  `INT_TYPE_UPDATED`. For groups this is the `en_int.c` default setter, a
  no-op; the mobiles' bitset is out of scope.
- **Global state:**
  - `update_entity` and `update_succ` (update module)
  - `system_delta_time`, `system_one_over_delta_time` and `locked_frame_rate`
    (time module)
  - `command_line_entity_update_frame_rate` (configuration)
- **Environment versus core.**
  - Environment: measuring the frame (`set_delta_time`: `timeGetTime`, the
    0.001 s floor, the history ring, the locked frame rate) and the host loop's
    time acceleration.
  - Campaign core: `get_delta_time` and `set_manual_delta_time`, because the
    update loop reads and overrides the delta.
  - Excluded: tacview logging (`tacview_is_logging`), which is off.
- **No callback scheduler.** EECH's temporal behaviour is this per-frame walk of
  the update list. The deterministic test boundary is therefore:
  - a scripted `Clock`, which supplies each frame's delta and locked flag;
  - a host-loop driver, which calls the real update loop `count` times per frame.

  The runaway guard is test-side only: it refuses a scenario whose frame would
  need more sub-steps than a fixed limit, and it never changes a result.

**Findings about the original C** (recorded, not "fixed"):
- **`time.c :: set_manual_delta_time`** writes
  `system_delta_time_history_position = (++system_delta_time_history_position) % DELTA_TIME_HISTORY_SIZE`,
  which is undefined behaviour in C. It only affects the delta history (read by
  `get_delta_time_average`), which is not ported.
- **`up_update.c`** clears its bitsets with
  `memset (x, 0, MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8))`, a byte count
  equal to the element count. Only a quarter of each array is cleared. This only
  affects the mobiles' `INT_TYPE_UPDATED` / moved / rotated bits, which are out of
  scope.

## Investigation 3: compiling the real `gp_int.c`

**Result: tractable, and done.** No reconstruction of `project.h` was needed.

`build/c-reference/project.h` is generated by `c-reference/extract.mjs` from
four kinds of content:
- the harness environment, `c-reference/eech_harness_env.h`: C library, the
  Windows SDK `min`/`max`, `ASSERT`, `debug_fatal`/`debug_log` declarations, two
  engine types only named in prototypes (`matrix3x3`, `viewpoint`), and no-op
  debug-build comms validation;
- verbatim fragments:
  - `struct VEC3D`, `bound`, `METRE`, `KILOMETRE` (`modules/maths`)
  - the time globals and `get_delta_time` (`modules/system/time.h`)
  - the comms model and data-flow enums, globals and accessors (`comms.h`;
    including the whole file would pull in the DirectPlay transport)
  - two map enums used as database values (`ui_menu/.../map.h`)
  - the session global and a few prototypes (`force.h`, `keysite.h`);
- **whole original headers**: `en_types.h`, `en_heap.h`, `en_msgs.h`,
  `en_funcs.h` (all value and list dispatch macros and tables), `en_comms.h`,
  `en_dbmsg.h`, `ai_extrn.h`, `up_update.h`, `division.h`, `group.h`,
  `tacview.h`;
- `c-reference/eech_harness_decls.h`: the configuration global and the
  prototypes of extracted functions.

Nine original translation units then compile **unchanged**: `gp_int.c`,
`gp_float.c`, `gp_list.c`, `gp_vec3d.c`, `gp_ptr.c`, `gp_updt.c`, `gp_dbase.c`,
`up_list.c`, `up_msgs.c`.
- **Link surface:** the dispatch tables (defined by the harness, filled by the
  original `overload_*_functions ()`), name databases read only on fatal paths,
  `entities` (so `get_local_entity_index` is genuine pointer arithmetic), and
  stubs that fail loudly: `add/remove_group_type_to/from_force_info`,
  `set_local_division_name`, and tacview.
- **Needs extraction instead of whole-file compilation:** `en_list.c`
  insert/delete, the `en_int.c`/`en_float.c` default setters, `en_msgs.c`, the
  update loop in `up_update.c`, `time.c :: set_manual_delta_time`, and two
  responses from `gp_msgs.c`.
- **Why those files aren't compiled whole:** they drag in sizeable unrelated
  subsystems, such as the name databases of `en_int.c`, mobile tacview frame
  handling, and the whole message table of `gp_msgs.c`.

Slice 1's 41 C reference cases and 250 recorded scenarios passed **unchanged**
against this oracle (commit `e5c34da`).

## Selected boundary

**Ported to TypeScript:**
- `gp_updt.c :: update_server` and `overload_group_update_functions`.
- `gp_float.c` `FLOAT_TYPE_SLEEP` / `FLOAT_TYPE_ASSIST_TIMER`: get, local set
  (with insertion into the update list), and server set.
- `up_update.c :: update_client_server_entities` and
  `set_entity_update_frame_rate`, with `update_entity` / `update_succ`.
- `up_msgs.c :: response_to_unlink_child`, the C-default link responses of the
  update entity, and `up_list.c` (`LIST_TYPE_UPDATE_ROOT`).
- `en_list.c :: insert_local_entity_into_parents_child_list` and
  `delete_local_entity_from_parents_child_list`, with their notifications. Only
  list types whose responses are ported can be used.
- `gp_msgs.c :: response_to_link_parent` (the `LIST_TYPE_DIVISION` case stays
  unported) and `response_to_unlink_parent`.
- `en_updt.c` dispatch, and the `en_int.c` default setter for
  `INT_TYPE_UPDATED` on groups.
- `modules/system/time.c`: `get_delta_time`, `set_manual_delta_time` and
  `locked_frame_rate`, without the delta history, which has no ported reader.
- **New port `Clock`:** the frame delta and the locked flag, which is what
  `set_delta_time` measures.
- **New option:** `entityUpdateFrameRate`, EECH.INI "entity update frame rate",
  default 2.

**Not in this slice:**
- whatever reads `sleep` / `assist_timer` (task assignment, landing and combat
  handlers);
- update functions of any other entity type: reaching one fails as unported;
- tacview;
- the delta-time history and average.

**C reference work:** the timeline scenarios run the original loop, update
function, setters, list code and time override. The shim entries retired by
this slice are listed in `docs/architecture.md`, "Shrinking the C reference
shim".
