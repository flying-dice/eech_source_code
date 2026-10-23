# M5: host-owned movement-dependent positioning (EEDCS-40)

**Status: source change on base `6e837440` (EEDCS-39 evidence head). Host, engine and Lua-boundary
tests, clippy and rustfmt pass on Linux. The DLL C changes were NOT compiled (no Windows/MinGW
build was run for this Task) and the real Lua -> native module -> world parent/child path was NOT
run. The Windows deterministic/Tacview pack and the census are PENDING; no result of either is
claimed. Not to be merged before PR98. EEDCS-40, D1 (EEDCS-15) and M5 are not complete, and the
technical lead has not approved.**

Evidence is in [`../reference/m5-dependent-positioning/`](../reference/m5-dependent-positioning/linux/).
Author self-review (not independent):
[`../reference/m5-dependent-positioning/self-review.md`](../reference/m5-dependent-positioning/self-review.md).

**Scope.** When a parent that has a movement record moves, the position of its roster
`LIST_TYPE_MOVEMENT_DEPENDENT` children (inserted under the keysite's movement parent in
`faction.c`) is now computed by the host, keyed by host id. Membership of the dependent list, and
every non-roster dependent (keysite, cargo, receiver, task/waypoint, effects), stays in the
campaign on the original path. No scene graph, no loss handoff (Area 2), no airfield, effects or
keysite policy.

## The defect, characterised on the base

The parent setters (`vh_vec3d.c`, `ac_vec3d.c`) send `ENTITY_MESSAGE_SET_ENTITY_POSITION` (new
position, heading, velocity) to each dependent *before* writing the parent position. The child
handler writes the child position, which re-enters `campaign_movement_place`. During a host apply
that call was suppressed by the non-reentrant `applying_host_result` flag, so a child written
inside its parent's host step left the host's child record stale: a write the host never saw.
`base_a_child_pose_the_campaign_wrote_during_its_parents_step_is_a_write_the_host_never_saw`
(`base-witness.log`) is the causal witness on the base, before any implementation edit.

## What moved

| Item | Host does | DLL does |
|---|---|---|
| Carry of a routed, AA or person child | `Request::DependentCarry` (mode 46): validates the parent by host id and the child's pose against its record, writes the child's new position | Decides membership (list order and insertion unchanged); sends parent host id, parent old and new position, heading, velocity |
| Carry of a helicopter child | `eech_dependent_helicopter` (`csrc/dependent_step.c`): the original `hc_msgs.c` arithmetic (rotate the offset from the parent's OLD position by `heading - en_heading`, then heading and attitude, then velocity) | Applies the position, attitude and velocity the host returns |
| Nested application | Each carry validates against the host's own record of the parent, so a grandchild is carried from the child's new record | `applying_host_result` is a depth counter; nested `campaign_movement_place` is suppressed only while a host result is applied, and the host record stays consistent |
| Unsupported shapes | Refused with an error; nothing changes | `debug_fatal` for a hosted child of an unsupported type |

- **Original behaviour kept.** Message send order (before the position write), the sender's OLD
  position at handler time, effects notification and `campaign_movement_place` order are
  unchanged. The original handler code runs for every unhosted child (`campaign_movement_is_hosted`
  is false without a host callback), so a non-host build is unchanged.
- **Parent validation.** The parent's old position must equal the host record's position (before
  the parent's own write), or its `previous_position` with new equal to the record position
  (after the host's step). Anything else is "a move the host did not make" and is refused. Also
  refused: non-finite input, parent equal to child, retired parent, parent with no movement
  record, and a child pose that differs from the host record.
- **Documented limitation.** A hosted child with an *unhosted* parent takes the original path.
- **Shape difference.** The position shape does not apply `result.attitude`; the original handler
  copies only the position.

Mode `CAMPAIGN_MOVEMENT_DEPENDENT_CARRY` = 46 (`rv_hostmv.h`, `eech-engine-sys`) with new step
fields `dc_parent_host_id`, `dc_helicopter`, `dc_parent_old[3]`, `dc_parent_new[3]`, `dc_heading`,
`dc_velocity`. The Lua mode is `dependent_carry` with `dc_*` fields; a missing `dc_parent` or a
vector that is not 3 finite numbers is refused at the boundary.

## Checks and results (logs in `reference/m5-dependent-positioning/linux/`)

Pinned `eech-build` image, volumes `eech-cargo` and `eech-dep-pos-target` (isolated; PR98's volume
and binaries were not touched), run through `dock.sh`.

- `dep-final-test.log` (`cargo test --offline --locked -p eech-world -p eech-engine -p eech-dc`):
  eech-dc 3 and `lua_debug_layout` 2; eech-engine 30; eech-world lib 136, bin 18,
  `host_callbacks` 4, `load_dc` 2; 0 failed. Counts at the base were 28 / 131 / 18 / 3 / 2.
- New tests: five host tests in `movement.rs` (the base witness; a child and grandchild following a
  host-moved parent; a child following a parent the campaign is about to write; refusal of a carry
  the host cannot vouch for; the helicopter arithmetic), two in `eech-engine` (the step crosses the callback;
  a missing parent or non-finite position is refused) and
  `a_dependent_carry_round_trips_through_lua_and_refuses_what_the_host_did_not_move`
  (`host_callbacks`, prints `DEPENDENT_CARRY_LUA=ok`): a routed carry, a nested grandchild, a later
  stop with the carried pose, refusals (self-parent, unknown parent, stale or foreign parent move,
  NaN, child pose differing from the host, missing parent, helicopter shape for a routed child),
  and a helicopter carry.
- `dep-carry-boundary.log` to `dep-carry-boundary3.log` are failed iterations (wrong expectations
  in my Lua test, fixed in the test; the host was not changed); `boundary4` is green. They are
  retained, not hidden.
- `dep-fmt-clippy.log`: initial rustfmt diffs in my edits; clippy 0 with `-D warnings`.
  `dep-fmt-apply.log`: `cargo fmt --all`, then `--check` clean. **Formatting was applied after the
  final test run; tests were not rerun.**
- **Red evidence is limited.** Only the base witness is red-before-fix evidence for the defect. The
  carry tests were written with the implementation and were not run against the base.

## Reproduction

From `eech-campaign/`, in the pinned image:

```
cargo test --offline --locked -p eech-world -p eech-engine -p eech-dc --no-fail-fast
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
```

`rustfmt` and `clippy` need `rustup component add` in a disposable container, as in
[`m5-aircraft-control.md`](m5-aircraft-control.md).

## Pending and not claimed

- The DLL C (`rv_hostmv.c`/`.h`, `hc_msgs.c`, `rv_msgs.c`, `aa_msgs.c`, `ps_msgs.c`) is verified by
  reading only; it has not been compiled.
- Host-owned Lua -> public native module -> world acceptance of a real parent/child move needs the
  retail data root (Windows host) and has not run.
- The established deterministic regression pack, Tacview evidence (`windows-retail v1` baseline)
  and the census are required before delivery and are pending; PR98's runs are live. No baseline
  or tolerance was changed.
- Not claimed: carry under an unhosted parent, a general scene graph, the loss handoff, DCS
  behaviour.
- Ordering: do not merge before PR98. Technical-lead approval has not been given.
