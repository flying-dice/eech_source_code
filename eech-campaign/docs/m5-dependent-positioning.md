# M5: host-owned movement-dependent positioning (EEDCS-40)

**Status: source change on base `6e837440` (EEDCS-39 evidence head); R1 build gating fixed at
`7c61e13a`. Host, engine and Lua-boundary tests, clippy and rustfmt pass on Linux. The host-build DLL
C compiled and linked in an isolated pinned native build and `eech-world.exe` ran two Lebanon
witness runs through the public Lua -> `eech_dc` -> host path, but **no `dependent_carry` step
occurred in either run (0 carries)**, so the real causal witness the independent review asked for
(R2) is NOT achieved and no carry shape (position, helicopter, nested, guard) has been observed on
the real C path. The Windows deterministic/Tacview pack is PENDING; no result of it is claimed. Not
to be merged before PR98. EEDCS-40, D1 (EEDCS-15) and M5 are not complete, and the technical lead
has not approved.**

Evidence is in [`../reference/m5-dependent-positioning/`](../reference/m5-dependent-positioning/linux/).
Author self-review (not independent):
[`../reference/m5-dependent-positioning/self-review.md`](../reference/m5-dependent-positioning/self-review.md).

**Scope.** When a parent that has a movement record moves, the position of its roster
`LIST_TYPE_MOVEMENT_DEPENDENT` children (inserted under the keysite's movement parent in
`faction.c`) is now computed by the host, keyed by host id. Membership of the dependent list, and
every non-roster dependent (keysite, cargo, receiver, task/waypoint, effects), stays in the
campaign on the original path. No scene graph, no loss handoff (Area 2), no airfield, effects or
keysite policy.

## The defect, and what is and is not characterised

The parent setters (`vh_vec3d.c`, `ac_vec3d.c`) send `ENTITY_MESSAGE_SET_ENTITY_POSITION` (new
position, heading, velocity) to each dependent *before* writing the parent position. The child
handler writes the child position, which re-enters `campaign_movement_place`. During a host apply
that call was suppressed by the non-reentrant `applying_host_result` flag, so a child written
inside its parent's host step could leave the host's child record stale: a write the host never saw.
This is **inferred from reading the C**, not observed on the base.

`base_a_child_pose_the_campaign_wrote_during_its_parents_step_is_a_write_the_host_never_saw`
(`base-witness.log`) is a **host-consistency unit test**. It fabricates the child's changed pose and
calls `Movers::Stop` by hand: it does not invoke a C parent setter and does not exercise the message
handlers, the notification order or the guard. It also models a routed child keeping an offset,
whereas the real routed handler copies the parent position. It is not the causal or red
characterisation of the real defect and is not native acceptance. The real path is exercised only by
the native witness below.

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
  is false without a host callback). The four call sites are compiled only under
  `EECH_CAMPAIGN_HOST_MOVEMENT`, like the other host extraction sites, because `routed.h` exposes
  `rv_hostmv.h` only in that build. A non-host (retail) build of the DLL C was **not** compiled for
  this Task.
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
- **Red evidence is limited.** No test was run red against the real C path on the base. The base
  unit test is a host-consistency check, not that. The carry tests were written with the implementation.

## Reproduction

From `eech-campaign/`, in the pinned image:

```
cargo test --offline --locked -p eech-world -p eech-engine -p eech-dc --no-fail-fast
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
```

`rustfmt` and `clippy` need `rustup component add` in a disposable container, as in
[`m5-aircraft-control.md`](m5-aircraft-control.md).

## Native build and witness (observed)

Build (`tools/build-native.sh`, pinned image `sha256:bb10e44a...562be`, fresh volume
`eech-dep-pos-native-target`, `cargo build --release --target
x86_64-pc-windows-gnu -p eech-dc -p eech-world`, MinGW gcc 12-posix, Lua 5.1.5, C flags `-w`): all
exits 0 at commit `7c61e13a951c4139e8e136dbf392e4dbf1a544ec`, no uncommitted build inputs.
`eech_dc.dll` sha256 `69caef0aade5fbb2602157ad09d7868a79a9298958cb00d2bf472a6f06471b93`,
`eech-world.exe` `f4ed25238319cde17f903be32d095f9bbe704733081cac3db267d029fe0d7d3d`, `lua.dll`
`b224cb2eacdea06fcb7fe273979e660fbef0c1a9c65be53c7be9fc0a4571daca`. `-w` suppresses warnings, so no
warning-clean claim. The non-host (retail) DLL C build was not compiled. PR98's worktree, binaries,
volume and data roots were not touched; the run used a copied retail root.

Witness (`crates/eech-world/lua/dependent-carry.lua`, observational: wraps the reference
`move_vehicle`, answers every step with the real host, counts `dependent_carry` steps by child and
parent kind and shape, nested depth, a campaign `place` of a carried child, and refusals; checks each
answer against its step). Scenario: `map5`, `camp01/LEBANON.CHC`, apache, seed 1.

| Run | Simulated | Exit | Wall | Carries | Refused | Nested | place after carry |
|---|---|---|---|---|---|---|---|
| probe 1 | 900 s | 0 | 10m31s | 0 | 0 | 0 | 0 |
| probe 2 (adds a per-step-mode census) | 900 s | 0 | 10m32s | 0 | 0 | 0 | 0 |

**Observed (probe 2 census).** 36 ships were bound (`create_place/ship`). The ship modes seen were
`expire`, `ship_sink` (2877) and `vehicle_vector_seed` (2877); there was **no `ship_move` and no
`dependent_carry` step**. Ships bearing dependents were not identified, so whether any `ship_sink`
step was of a dependents-bearing parent is not established; no carry resulted.

**Inferred, not observed.** By reading `assign.c:451` and `:809`, carriers (the roster parents of
`LIST_TYPE_MOVEMENT_DEPENDENT`) are excluded from task assignment, so a shipped campaign does not
move them and no natural carry is expected. The run is consistent with that and does not prove it.

**Not achieved.** Producing a real carry needs either a scenario edit (a moving parent) or a
test-only driver hook, both of which change scenario data or add an API and were not done without a
lead decision. Until then, the position, helicopter, nested and guard shapes are covered only by the
Rust host tests and the Lua boundary test, which emulate the notification, and must not be presented
as native acceptance. Raw logs are kept outside git.

## Pending and not claimed

- R2: a real public Lua -> native module -> world parent/child carry through the actual C handlers
  (needs a lead decision, above).
- The non-host (retail) DLL C build.
- The established deterministic regression pack, Tacview evidence (`windows-retail v1` baseline)
  and the census are required before delivery and are pending; PR98's runs are live. No baseline
  or tolerance was changed.
- Not claimed: carry under an unhosted parent, a general scene graph, the loss handoff, DCS
  behaviour, the untested `debug_fatal` path for an unsupported hosted child type.
- Ordering: do not merge before PR98. Technical-lead approval has not been given.
