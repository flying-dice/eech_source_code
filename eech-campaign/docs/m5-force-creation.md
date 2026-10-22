# M5: host-owned force creation, initial placement, lifetime and retirement (EEDCS-38)

**Status: source change committed locally on base `8ddce83d` (the PR96 merge). Host/Rust
tests pass; the native pack has NOT been run against it. EEDCS-38, D1 (EEDCS-15) and M5
are not complete.**

**Scope.** One physical extraction under D1, following
[`m5-other-force-lifecycle.md`](m5-other-force-lifecycle.md). The host now calculates the
initial physical placement of a created member, decides when a dead wreck's physical
lifetime ends, and refuses every step for a released identity. The campaign keeps force
intent and the group, task and logistics consequences. The legacy entity heap remains an
explicit mirror: the DLL still allocates the entity and frees it. A live DCS run is M6 and
is not claimed.

## What moved

| Item | Host does | DLL does |
|---|---|---|
| `create_place` (new `move_vehicle` mode) | Places a created member from the constructor's requested pose and retains it as that host id's state. A later `create_place` replaces the pose bound at link time | Writes the pose to the entity heap from the host's answer (`faction.c`, 4 creation sites) |
| Retirement (`campaign_movement_retire`) | `set_free_entity` releases the entity, the roster hook retires the host id. Any later step or reuse of that id is refused, and any open sequence state is dropped | Calls the hook before heap-free finalisation |
| `expire` (new lifecycle mode) | Ages a wreck by the frame, applies the original dead-frame gate and the squared-range threshold, resets the timer for an occupied helicopter, and ends a ship once it has sunk half its diagonal below the waterline. Answers yes or no in `flags` | Supplies the timer, the keysite range and the policy constants. Destroys the entity only on a yes |

- **Original arithmetic, kept.** `crates/eech-world/csrc/place_step.c` and
  `expire_step.c` copy the original expressions (`mobile.c` lifetime code, including the
  unparenthesised-macro divisor of 1e8). The per-family removal and effect order is not
  changed; the original code stays in `#else` branches of `#ifdef EECH_CAMPAIGN_HOST_MOVEMENT`.
- **Physical expiry and explicit campaign removal are distinct.** Expire is the host's
  physical end; a campaign remove still goes through the existing release path.
- **Mirror rule.** The wreck timer is seeded from the DLL while the host retains none, then
  compared bit-exactly (`check_mirror`); a difference is refused before anything mutates.
- **Guards.** Expire is refused while a ground fall, death slide or crash slide is open, and
  for a retired id. A refused request changes nothing.
- **Unchanged.** A link without a creation request is not made fatal. Initial state is hooked
  only when the member is fully initialised. RNG draws, pending loss receipts and the Tacview
  phase are not touched by these paths.

## Evidence

Run on this worktree, cache volume `eech-d1-creation-target2`, full logs in
`E:/eech-runs/m5-d1/creation-placement-removal-20261022/logs/`:

- `cargo fmt --all` (host): exit 0.
- `cargo build --workspace --all-targets` (Docker, `bash` with `pipefail`): exit 0, no warnings
  (`build-workspace.log`).
- `cargo test --workspace`: exit 0 (`test-workspace.log`); `eech-world` 84 passed, `eech-engine`
  18 passed, no failures. New tests: 5 retirement, 6 placement and 5 expire in `movement.rs`;
  an expire-step wire test and NaN/infinite refusal checks in `eech-engine`.
- Red evidence is compile-red only (the new modes and fields did not exist). One test failed
  once on a test bug (it opened a helicopter fall, which leaves nothing open) and was corrected.

## Remaining coupling and limitations

- **Native pack not run.** The affected M4 regression pack and a Tacview comparison are still
  required before this can be called validated. Until then the DLL paths (`faction.c`,
  `gp_roster.c`, the six family updaters) are compile-checked only.
- **Possible fatal.** Expire is refused while a fall or slide is open, and dispatch refuses
  requests while a helicopter step is pending. A legitimate dead-frame flow reaching either
  guard would be a fatal `ask_lifecycle` error. Only the native pack can show whether one does.
- **Keysite range is campaign-computed.** `calculate_mobile_nearest_keysite_sqr_range` is
  evaluated by the DLL and sent. It runs on every dead frame, where the original evaluated it
  only on the gated frame; it has no side effects and draws no RNG.
- **Terrain sampling** stays with D3. Anti-air (routed kind) never issues Expire, and the
  client-side sites still only accumulate the timer.
- **Not covered:** the remaining D1 aircraft control and avoidance, D2, D3. Boot and regen
  creation paths were read and compile but are not exercised. Clippy is not available in the
  build image and was not run.
