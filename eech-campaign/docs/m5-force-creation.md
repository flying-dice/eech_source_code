# M5: host-owned force creation, initial placement, lifetime and retirement (EEDCS-38)

**Status: source change on base `8ddce83d` (the PR96 merge), revised after the source review.
Runtime source frozen at `4cf0bd99`; native build, public witness and the single regression pack ran
at `5aa6ca54` (runtime source unchanged since `4cf0bd99`). Host/Rust tests and clippy pass, one
pinned MinGW build is clean, the corrected public witness passed, and the pack is 40 of 40 PASS,
exit 0. These are the runner's results for the pack's own scenarios; the parent verifies the full
reference hashes before approval. EEDCS-38, D1 (EEDCS-15) and M5 are not complete, and nothing
here is initial source delivery or ownership beyond code.**

Evidence is in [`../reference/m5-force-creation/`](../reference/m5-force-creation/reference-summary.txt).

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
| `create_place` (new `move_vehicle` mode) | Takes the campaign's physical input facts (base position, terrain normal, centre-of-gravity offset, fuselage angle, heading), computes the final height (`y + cg`, refused if non-finite) and the attitude, and retains both as that host id's state. A later `create_place` replaces the pose bound at link time | Chooses the intent (formation x/z, route, keysite heading, group, task), samples the terrain (D3), then writes the host's returned position and angles to the entity heap (`faction.c`, 4 creation sites) |
| Retirement (`campaign_movement_retire`) | `set_free_entity` releases the entity, the roster hook retires the host id. Any later step or reuse of that id is refused, and any open sequence state is dropped | Calls the hook before heap-free finalisation |
| `expire` (new lifecycle mode) | Ages a wreck by the frame, owns the dead-frame gate `(int)(timer+1) % 300` and the squared-range threshold, resets the timer for an occupied helicopter, and ends a ship once it has sunk half its diagonal below the waterline. Answers yes or no in `flags`; on a gated step with no range it answers "needs range" (flag 2) and records nothing | Supplies the timer, the policy bits and, only when the host asks, the keysite range. Destroys the entity only on a yes |

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
- **Input validation (before the unsafe C call).** `death_timer`, `delta_seconds` and a supplied
  `death_range` must be finite and nonnegative; the summed timer must stay inside the gate's
  `int` range; `death_policy` bits other than 1, 2 and 4 are refused. A refusal leaves the
  record untouched (`an_invalid_expiry_input_is_refused_and_changes_nothing`).
- **Lazy keysite range, host-owned gate.** The DLL first sends a step without a range. Only if
  the host's own gate is open does it answer flag 2, and the DLL then samples
  `calculate_mobile_nearest_keysite_sqr_range` once and re-issues the step with policy bit 4
  and `death_range`. The original evaluated the range only on the gated frame; so does this.
  There is no second gate in the DLL (`a_step_off_the_gate_never_asks_for_the_range`).
- **Unchanged.** A link without a creation request is not made fatal. Initial state is hooked
  only when the member is fully initialised. RNG draws, pending loss receipts and the Tacview
  phase are not touched by these paths.

## Evidence

Run on this worktree, cache volume `eech-d1-creation-target2`, full logs in
`E:/eech-runs/m5-d1/creation-placement-removal-20261022/logs/`:

Initial run (superseded, kept): `build-workspace.log`, `test-workspace.log`, `fmt.log`.
`eech-world` lib 84, `eech-world` main 18, `eech-engine` 9 passed (an earlier note said engine
18; that was wrong). The passing evidence and first checkpoint were archived by the parent in
`source-2b5cfa5a`.

Review-fix run (current), all exit 0, unique logs:

- `cargo fmt --all` / `--check`: clean.
- `cargo build --workspace --all-targets` and `cargo test --workspace` in Docker
  (`build-test-fixes.log`, `test-fixes-final.log`): `eech-world` lib 88, `eech-world` main 18,
  `eech-engine` 9, `eech-campaign` 11, `eech-dc` 3 (+2 `lua_debug_layout`), harness corpus 3,
  scenarios 1, `eech-map` 1, `eech-sys` findings 2 and smoke 2, `host_callbacks` 2, `load_dc` 2,
  doc-test 1. No failures.
- Clippy ran in a throwaway `rust:trixie` container after `rustup component add clippy`
  (not preinstalled): clippy 0.1.99 (58cf8f9c35 2026-10-13), rustc/cargo 1.99.1.
  `clippy-fixes.log` failed (component missing), `clippy-fixes-2.log` reported one
  `manual_range_patterns` at `eech-engine/src/lib.rs:407` (fixed as `0..=2`),
  `clippy-fixes-3.log` is clean.

**Red evidence.** None is claimed, and no compile-red was manufactured. The one retained test
failure (the initial expiry test) was an **incorrect test setup**: it opened a helicopter fall,
which does not enter the ground-fall phase, so nothing was open. It is not a production red. The
retained `linux/test-initial-failure-transcript.txt` is a parent-recovered excerpt, not the
original full log.

**Host and bridge tests.** The two `movement.rs` tests vary the campaign's base height and
centre-of-gravity offset and check the host's returned position and angles; the `eech-engine`
wire test carries `cg_to_ground`, `death_range` and `death_policy` to the host request. These
are host-side and bridge-level tests only. They make no claim about what the DLL does with the
answer; the public witness below covers position only.

### Native build, public witness and pack

- **Build.** One pinned MinGW build at `5aa6ca54` (0 dirty inputs), exit 0, 39.83 s, with no
  cargo warnings. `eech-engine-sys/build/main.rs` compiles every listed legacy C source
  (`faction.c`, `rv_hostmv.c`, `gp_roster.c`, the family updaters) with
  `EECH_CAMPAIGN_HOST_MOVEMENT` on Linux and MinGW, so the Linux cargo builds compiled them too.
  An earlier statement in this branch that they did not was wrong and is withdrawn.
- **Witness.** The first witness run crashed at boot (exit 139, msvcrt `0xc0000005`). The cause
  was a wrongly escaped map path in the witness script (three literal backslashes), a setup error,
  not a candidate defect. Its script and logs are kept unchanged. The corrected script changed
  only that line (plus a pre-boot assertion) and ran on the same binaries and root: exit 0 in
  2.1 s. Through the public Lua, `eech_dc.dll` and `eech-world` path, one routed member's
  `create_place` answer was shifted by (+250, 0, -125) m and the entity heap held that position
  (0.000 m difference; 279.508 m from the DLL's input), while 1443 of 1443 controls matched
  within 0.01 m. Limits: boot only, position only, `y` unchanged so the CG offset is not
  discriminated, one shifted subject. It makes no height, attitude, expiry, retirement or
  full-lifecycle claim.
- **Pack.** One regression pack run (windows-retail v1, unchanged): exit 0, 40 of 40 checks PASS
  (17 directly protected, 12 baseline-sensitive, 8 NOT COVERED, 8 EVIDENCE ONLY as the report
  states). The parent-prepared verifier, run once unmodified, found the complete Tacview
  (117,690,861 B, `3bd17b64…2d19b`), observations (85,716,447 B, `da26bab0…77cf`) and metrics
  JSON equal to the retained references in both runs. An earlier prefix check reported a
  mismatch because its input was the older `lebanon-3h` observation format; that was a verifier
  input error, corrected against the accepted supply-detail observations, with no candidate or
  baseline change.
- The pack validates its scenarios only. It is not a claim that rare paths (helicopter,
  fixed-wing and person terminal ordering, the fatal guards) were exercised.

## Corrections made in the review pass

- **Undefined normal (Finding 2).** `create_faction_members` previously read `face_normal`
  uninitialised when `!place_at_ground_height` in the legacy path
  (`popread.c:1439` creates static infantry with `FALSE, FALSE`). The hosted branch now sets an
  explicit flat normal `(0, 1, 0)` first. This is a source-level initialisation correction: the
  legacy source left the value undefined. In the accepted build, `-ftrivial-auto-var-init=zero`
  (`eech-engine-sys/build/main.rs`) makes the old unset normal zero, and the heading-0 zero-normal
  fallback and the explicit flat normal give the same base attitude, so baseline semantics did
  not change. The `#else` path is unchanged. Every defined path, RNG draw and order is
  unchanged, and no baseline or tolerance was touched.
- **Physical placement (Finding 1).** The host now receives the base position, normal, CG offset,
  fuselage angle and heading and returns `position = [x, y + cg, z]` (f32 add) and the angles;
  the DLL no longer adds `cg_to_ground` before the hosted call.

## Creation path coverage

| Site (`faction.c`) | Hosted placement | Fallback |
|---|---|---|
| `create_landed_faction_members` | `create_place`, CG from the aircraft database; host position and angles applied | `#else`: `pos.y += cg_to_ground`, original setters |
| `create_landed_faction_group` | same | same |
| `create_landing_faction_members` | `create_place`; CG is 0 unless `MOVEMENT_TYPE_AIR`; applied with `set_client_server_entity_*` | `#else` unchanged, including the dead `member_pos` |
| `create_faction_members` | `create_place` only if `campaign_movement_is_hosted`; flat normal when not placed on ground | `#else`: `pos.y += cg_to_ground` first |

Traced in source. Boot and regen creation reach these same four functions. The Lebanon pack
scenario and the witness create members through them (1444 `create_place` answers in the
witness boot); which of the four sites and which family branches that reaches is not itemised.

## Terminal call-ordering analysis (expiry vs open host sequences)

Expire is refused while a fall or slide is open, so a legitimate dead-frame flow must have
closed the host's sequence before the entity is DEAD.

- **Ground vehicles, traced:** `rv_move.c:695-743` calls the host `gv_fall_settle`; on
  `flags & 0x04` it calls `campaign_movement_gv_fall_vector` and sets DEAD in the same call,
  and `campaign_movement_gv_commit` (`rv_move.c:747`) runs after DEAD is marked, closing the
  host's sequence in that call. The expire branch (DEAD only) therefore follows the host's
  terminal answer and closure. Anti-air has the same
  shape (`aa_move.c:317-365`) and never issues expire.
- **Not verified:** helicopter (`hc_move.c` DEAD at 565/762/865/1024/1052/1099, using
  `campaign_movement_fall` and `crash_begin`/`crash_end`), fixed-wing (`fw_move.c`
  2548/2797/2821/2866/3045/3210, `fw_fall` and the crash functions) and person (`ps_`) paths
  were located but their open-sequence closure before DEAD was not traced per site.
- **Not verified:** that the dispatch guard "refuse while a helicopter step is pending" is
  unreachable on a dead frame.

If any unverified path reaches a guard the result is a fatal `ask_lifecycle` error, not a
silent wrong answer. The pack passed without reaching one, but it does not enumerate these
paths, so this is not a claim about the unverified sites.

## Remaining coupling and limitations

- **The native result validates only its scenarios.** The pack passed 40 of 40 and the witness
  covers position only. Rare paths are not claimed as exercised, and the Tacview was checked by
  hash, not visually reviewed.
- **Fatal-guard reachability** is untraced per site for helicopter, fixed-wing and person (see
  the ordering analysis above); only ground vehicles are traced.
- **Keysite range is campaign-computed, now lazily.** `calculate_mobile_nearest_keysite_sqr_range`
  runs once, only after the host's gate opens and asks for it, as in the original. It has no side
  effects and draws no RNG.
- **Terrain sampling (base y)** stays a campaign input from D3. CG offset, final height and
  attitude are host-owned. The entity heap remains a mirror the DLL writes.
- **Anti-air** (routed kind) never issues Expire, and the client-side sites still only
  accumulate the timer.
- **Not covered, remaining work:** D1 control and avoidance, D2 combat and perception, D3
  terrain inputs. No live DCS run (M6). Boot and regen creation are traced to the four sites.
