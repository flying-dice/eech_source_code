# M5: host-owned movement-dependent positioning (EEDCS-40)

**Status: source change on base `6e837440` (EEDCS-39 evidence head); R1 build gating fixed at
`7c61e13a`. Host, engine and Lua-boundary tests, clippy and rustfmt pass on Linux. The two 900 s
Lebanon witness runs through the public Lua -> `eech_dc` -> host path saw **0 carries**, which is
non-acceptance. R2 is closed only by the controlled fixture (commit `c9cb51a3`): genuine C handlers
reached through the public module, an artificial `link` arrangement, 34 checks green, and a
test-only old-guard build that fails the 3 guard checks. A final default-feature candidate was built
at `263b48a4` (section "Final default-feature candidate"). The established regression pack then
passed 40/40 on that candidate (exit 0, retained Lebanon Tacview and observations byte-identical in
both runs, `independent-reference-hashes.json` passed=true; section "Final pack"). PR98 (`ed38c8fb`,
CI/docs only) is merged locally at `b83c9699`. No visual Tacview claim. EEDCS-40, D1 (EEDCS-15) and M5
are not complete, and the technical lead has not approved.**

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
parent kind and shape, carries whose parent had itself been carried earlier (`parent_previously_carried`:
history, not proof of call-stack nesting), a campaign `place` of a carried child, and refusals; checks
each answer against its step). Scenario: `map5`, `camp01/LEBANON.CHC`, apache, seed 1.

| Run | Simulated | Exit | Wall | Carries | Refused | Previously carried | place after carry |
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

**Not achieved by those probes.** Zero carries is not acceptance: the probes show no carry occurred,
not that carries are correct. R2 was closed with the controlled fixture below instead.

## Controlled fixture (R2): genuine C handlers, test-only driver

**Route.** A shipped campaign never moves a carrier, so the fixture arranges the parent/child pair.
Scenario data was not edited. Test-only features `eech-dc/dependent-fixture` and
`eech-dc/dependent-fixture-old-guard` (never in the shipping build, never labelled as it) add one
gated C function and Lua method, `engine:dependent_fixture(op, parent, child, x, y, z)`, at the
existing module boundary; the production API and production C are unchanged without the feature.
Ops: `link` re-inserts an existing hosted entity of the booted Lebanon campaign into the parent's
original `LIST_TYPE_MOVEMENT_DEPENDENT` list; `move` calls the original
`set_local_entity_vec3d(parent, POSITION)`; `sink` calls the original `ship_vehicle_death_movement`.
Everything after that is the unmodified path: the parent setter's
`notify_local_entity(SET_ENTITY_POSITION)`, the original child handlers (`rv/aa/ps/hc_msgs.c`),
`campaign_movement_dependent_carry` and the reference host's callbacks (`dependent_carry` steps
answered by the real host arithmetic). No handler is replaced by Rust or Lua. The driver is called
from `crates/eech-world/lua/dependent-fixture.lua`, which records every step and compares
`engine:objects()` positions (DLL) with the positions the host last returned.

**Red.** `dependent-fixture-old-guard` rewrites only the staged copy of `rv_hostmv.c` to the old
boolean guard (`applying_host_result = 1` / `= 0`). The repository source, the PR98 candidate and
accepted references are untouched.

Builds at commit `c9cb51a3dd2fbf4130fc59323fa2e255d2905a28`, no uncommitted build inputs, pinned
image, new volumes `eech-dep-fix-green-g2` / `eech-dep-fix-red-old-guard-r2`, all exits 0: green
`eech_dc.dll` `5cae0903...34c7`, red `a09ade2a...f396` (test builds); `eech-world.exe` and `lua.dll`
as in the native build above. Summaries:
`reference/m5-dependent-positioning/native/fixture-{green,red-old-guard}.summary.txt`.

| Build | Exit | Carries | Refused | Checks | Failed |
|---|---|---|---|---|---|
| green (fixed guard) | 0 | 43 | 0 | 34 | 0 |
| red (old boolean guard) | 1 | 43 | 0 | 34 | 3 |

**Observed (green).** Parent = hosted ship 83133; each shape reached through the real handlers:
- position: a routed child under the ship is carried once to `parent_new` (host = DLL). Six
  pre-existing campaign dependents of that ship were carried too (real carries, included in totals).
- helicopter: horizontal offset kept to 0.0017, height to 0.00003, heading = parent heading. With a
  rotated parent (heading -2.07) the host result matches the original handler's formula to 0.001,
  the offset is turned rather than copied, heading = parent heading, host = DLL.
- chain: a grandchild under the carried child: the child's carry precedes the grandchild's, the
  grandchild lands on the child's new position, no campaign `place` of a carried id in the same
  operation (`chain.no_place_after_carry`), and a second move after the nested application still
  reaches the parent's own `place` and carries the chain again (`guard.restored_after_chain`).
- sink: original `ship_vehicle_death_movement`, 9 carries, every carry's parent the ship or the
  carried child, no spurious `place`, host and DLL agree for child, helicopter, grandchild, parent.

**Observed (red).** The same script fails exactly the guard checks: `chain.no_place_after_carry`
(1), `guard.restored_after_chain`, `sink.no_place_after_carry` (1). The position, helicopter,
rotation and agreement checks still pass under the old guard, so the failures are the guard defect
and not a fixture artefact. A first red attempt failed for a setup reason (the feature did not
expose the driver method: `attempt to call method 'dependent_fixture' (a nil value)`); fixed in
`c9cb51a3` and not counted.

**Limits.**
- `link` is an artificial arrangement (routed under a ship, helicopter under a routed vehicle,
  grandchild under the routed child): the relationships exist in the original list types but the
  campaign would not make them. It mutates the copied campaign's lists only.
- Depth beyond two levels, a hosted child of an unhosted parent, the `debug_fatal`
  unsupported-child path and the non-host (retail) C build were not exercised or compiled.
- Nested application is shown by order and by the guard outcome, not by a call-stack depth reading.
- The fixture is a bounded 30-frame warm-up of the Lebanon campaign, not the established
  deterministic pack or Tacview evidence (see "Final pack" for those).
- The dev build used while writing the fixture came from a dirty tree and is not evidence.

## Final default-feature candidate

Built at `263b48a4d65914f0383cd098b364847e6497b51c` (no uncommitted build inputs), pinned image,
new volume `eech-dep-cand-c1`, default features, exits 0: `eech_dc.dll` `a5b9dedd...bafe`,
`eech-world.exe` `bc91c27c...7cb5`, `lua.dll` `b224cb2e...daca`. Both hashes differ from the
`7c61e13a` native build; byte identity is not claimed and the cause is not isolated. Full hashes,
checks and the pack command: `reference/m5-dependent-positioning/native/candidate-normal.summary.txt`.

- `cargo fmt --all --check`, and `cargo clippy --locked --all-targets -- -D warnings` for default,
  `dependent-fixture` and `dependent-fixture-old-guard` (default checks skip the gated Rust and C):
  clean. Default tests: eech-dc 3, lua_debug_layout 2, eech-engine 30, eech-engine-sys 0; all pass.
  The only finding was the `[features]` block splitting the lints comment in `eech-dc/Cargo.toml`.
- Public Lua check: the normal DLL has no `engine:dependent_fixture` (index nil; call "attempt to
  call method 'dependent_fixture' (a nil value)"); the fixture-green DLL does, as a positive control.
  This shows the Lua method is absent, not a byte-level symbol absence proof.
- The fixture green/red runs were not repeated: the only source change since `c9cb51a3` is a
  comment move in `Cargo.toml`.

## Final pack

Normal c1 (`263b48a4`) passed the established `tools/regression-pack.py` (copied roots, `windows-retail v1`,
no baseline or tolerance change): **40 checks PASS, exit 0**. The parent's complete-file check
(`verify-complete.exit` 0, `independent-reference-hashes.json` passed=true) shows both runs' Tacview
(`3bd17b64...d19b`, 117690861 B) and observations (`da26bab0...77cf`, 85716447 B)
byte-identical to the retained references. Command, hashes and counts:
`reference/m5-dependent-positioning/native/final-pack.summary.txt`.

This is a no-regression result for the normal build. It is a bytes-equal-to-reference claim, not a visual
inspection of a Tacview recording. It proves reference / no-regression equality, not independent carry-path
coverage: no census was run on the full pack, so no carry count was measured in it. The controlled fixture
proves carry (with the limits above); the earlier 900 s probes measured zero carries (non-acceptance).

PR98 merge: `ed38c8fb` brings only the CI workflow and docs/evidence (16 files, no runtime path, no overlap);
runtime inputs of the merged tree equal those c1 was built from, so no second native pack was run. The merged
commit was not itself built.

## Pending and not claimed

- R2: closed for the fixture route above (genuine handlers through the public module, artificial
  linking, bounded scenario). The `link` arrangement is artificial and the old-guard is a test-only
  mutant of the staged copy of `rv_hostmv.c`, not the pre-change C.
- The non-host (retail) DLL C build.
- The census, and the pack's own NOT COVERED rows, are unchanged and not claimed.
- Not claimed: carry under an unhosted parent, a general scene graph, the loss handoff, DCS
  behaviour, the untested `debug_fatal` path for an unsupported hosted child type.
- Technical-lead approval has not been given. D1 stays incomplete until EEDCS-41 is delivered and the
  closure audit passes.
