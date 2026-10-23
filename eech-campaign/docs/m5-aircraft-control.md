# M5: host-owned aircraft control, fixed-wing and helicopter (EEDCS-39)

**Status: complete evidence for EEDCS-39 under D1 (EEDCS-15); not D1 closure, not M5 acceptance,
and not technical-lead approval.** Native runtime `b721ae5a`; reviewed equivalent lint delta
`f1371dda`; workflow-only head `10b02621`. The Windows regression pack passed 40/40 (exit 0), the
full Lebanon reference files are byte-equal to the retained references, the three-hour census
exited 0, and PR98 CI is green at `10b02621`. Scope authority is the live Confluence M5 goal and
the Jira ledger; this file records evidence only.

Evidence is in [`../reference/m5-aircraft-control/`](../reference/m5-aircraft-control/reference-summary.txt).

**Scope.** One physical extraction under D1, following
[`m5-fixed-wing-movement.md`](m5-fixed-wing-movement.md) and
[`m5-helicopter-movement.md`](m5-helicopter-movement.md). The host now computes the physical
control of aircraft that the DLL's `fw_move.c` and `hc_move.c` used to compute. The campaign keeps
intent. The legacy entity heap remains an explicit mirror; terrain and sector data remain transitional DLL
services. A live DCS run is M6 and is not claimed.
The parent is the one independent reviewer so far; the technical lead has not approved.

## What moved

| Item | Host does | DLL does |
|---|---|---|
| Fixed-wing steering | Collision avoidance, waypoint attain/follow, guide pursuit and the acceleration request (`eech_fw_navigate`, `csrc/fw_navigator.h`, `fw_step.c`) from retained poses and the intent sent per step | Resolves the waypoint, formation and task leader, group membership order, turn allowance and avoidance radius, and sends them as intent |
| Fixed-wing facing/turn | Facing-away test, `turn_speed_multi` and the toggle hold, retained per aircraft (`eech_fw_turn`); the multiplier is no longer sent by the DLL | Sends the cannon policy verdict (`aiming_cannon`) and `elapsed_time` |
| Fixed-wing terrain-follow climb shaping (R1) | Computes the shaped waypoint height from its own pose, speed and attitude (`shape_fixed_wing_waypoint`), then adds `site_y` and `cg` | Records `follow`, `min_height`, the unshaped guide position and the terrain elevation at the host's look-ahead point; no longer calls `mobile_terrain_adjust_waypoint_height` in a host build |
| Helicopter guidance | Plan, guide, begin, end and abandon as one retained flow (new step modes 40 to 44): structure avoidance, terrain fold, demand and steering, and the desired heading, pitch and roll | Sends intent, the environmental lookups the host asks for (structures in the aircraft's sector, terrain samples) and the cached terrain elevation |
| Look-ahead query (new mode 45, `FIXED_WING_LOOKAHEAD`) | Answers the point `pos + (velocity*1.0 + 10.0) * zv` from its retained pose, without mutating the record | Samples the terrain at that point and sends it back with the flight step |

- **Original arithmetic, kept.** The host C copies the original expressions, including double
  literals and float stores. The DLL's original code stays in `#else` branches of
  `#ifdef EECH_CAMPAIGN_HOST_MOVEMENT`. No bug fixes or rebalancing are included.
- **Campaign policy retained.** Waypoint and formation selection, task and guide intent, weapon
  and intercept inputs (D2), the helicopter's required heading as intent, and the `site_y` and
  `cg` addends stay in the campaign (DLL) side.
- **Transitional DLL services and mirror.** The terrain lookup (`get_3d_terrain_fast_elevation`,
  the cached `terrain_info` elevation) and the sector structure data stay in the DLL as
  transitional services, because the terrain service is D3. The entity heap is a mirror: the DLL
  still allocates and frees entities. Neither is host authority.
- **Lua keys.** The terrain block rides in `fixed_wing_terrain`, because `terrain` is already the
  routed sample.

## Review findings R1 to R5 and their fixes

| Id | Finding | Fix |
|---|---|---|
| R1 | Fixed-wing terrain-follow climb shaping still ran in the DLL and reached host navigation as campaign intent | Moved to the host; DLL sends the campaign's part and the look-ahead sample; host refuses a `sample_point` not bit-equal to its own look-ahead. Disclosed limits in `reference/m5-aircraft-control/prepush.md` and the source notes |
| R2 | Helicopter `dispatch` mutated retained flow before validating; end after a hosted begin was not caught; a repeated guide dropped the flow | Flow is cleared only after a successful non-flow request; guide validates before taking the flow; successful place/create clear the flow before the early return |
| R3 | The query-plan and guide contracts were mis-described (sector, max fold) | Contract corrected: the aircraft's own sector is enumerated and the actual running maximum and final cache ordering are preserved |
| R4 | Helicopter steer accepted `leader_heading` as a DLL physical observation | The host resolves the task leader by host id and derives heading from the leader's retained attitude, only on the demand path that reads it; the campaign's required heading stays intent |
| R5 | The bridge handoff said begin advances position and prescribed a post-begin terrain sample | Corrected: begin changes vector and speed only, and end consumes the cached terrain elevation with no new getter between them |

## Native source versus the lint commit

- **Native-built source:** `b721ae5ac8e0a7fa86aa47e79a0ca19884c0710f`, tree
  `1b0a2ca99b64656a9ca053c1e4eea6853b2eab59`, 0 dirty build inputs, pinned image
  `sha256:bb10e44a…e562be`. The three binaries are hashed in
  `reference/m5-aircraft-control/BUILD-INFO.txt`.
- **Lint/source checkpoint:** `f1371dda9a7884ee60601bf68a870d093368d23e` (not the branch head;
  the docs/evidence commits are later), tree
  `65a8fab735946e39deb970481734d637280a4c85`, parent `b721ae5a`. It changes four files:
  `eech-dc/src/module.rs`, `eech-engine/src/lib.rs`, `eech-world/src/main.rs`,
  `eech-world/src/movement.rs`.
- **What differs from the native-built source:** rustfmt layout; `#[cfg(test)]` code in
  `eech-engine` (two helpers, `refusal` and `accepted`, replacing `expect_err`/`unwrap_err`/
  `unwrap`, and one `redundant_closure` fix); and **one production expression**, the range
  predicate in `module.rs` (`terrain_sample_count`, `LuaValue::Number` arm):
  `count >= 0.0 && count <= 1.0e6` became `(0.0..=1.0e6).contains(&count)`. The parent reviewed it
  as behaviour-equivalent, including NaN, signed zero and infinities. No native rebuild was done
  for it, so the native binaries are identified by `b721ae5a`, not by the lint commit.
  `git diff -w b721ae5a f1371dda` shows the non-layout delta.

## Checks and results (logs in `reference/m5-aircraft-control/linux/`)

- `cargo fmt --all --check`: clean (`prepush-fmt-check.log`, `EXIT=0`). This was run **locally**
  with rustfmt 1.9.0-stable, not in Linux or the pinned image, which has no rustfmt. Before the lint
  commit the same check reported 85 diffs in these four files.
- `cargo clippy --locked --all-targets -- -D warnings` (the CI step), in the pinned image:
  `prepush-clippy1.log` failed with 21 errors, all new in `b721ae5a`; `prepush-clippy2.log` is
  clean after the fixes. Clippy 0.1.98 was installed transiently in `--rm` containers
  (`rustup component add clippy`), since the image has no clippy.
- `cargo test --offline --locked -p eech-world -p eech-engine -p eech-dc --no-fail-fast` on the
  lint tree (`prepush-test.log`): eech-dc 3 and `lua_debug_layout` 2; eech-engine 28; eech-world
  lib 131, bin 18, `host_callbacks` 3, `load_dc` 2; 0 failed. `r1-green.log` is the same run
  before the lint commit with identical counts. No test was added, removed or loosened by the
  lint commit. These ran in the pinned image with `--offline`, which relies on a populated
  `eech-cargo` registry cache.
- One pinned MinGW build of the native binaries at `b721ae5a`: exit 0 (`build.exit`) in 54.21 s.
  `build.log` is the full, unfiltered, non-verbose `cargo build` output (279 B, four lines;
  `cargo build ... > /evidence/build.log 2>&1` in `build-native.sh`, retained here). The exit code
  is in `build.exit`, not in `build.log`. The C flags include `-w` (`eech-engine-sys/build/main.rs`),
  so no C warning-clean claim is made. See also `BUILD-INFO.txt`.
- **Red evidence is limited for R1.** The R1 host tests were written after the R1 host
  implementation. The only red recorded here for R1 is a compile break from a test-authoring slip
  (`r1-red1.log`, `Kind::Vehicle` does not exist); it is not behavioural red evidence, and the new
  R1 tests were not run against the pre-R1 code. Earlier stage 3 and R2 behavioural red exists in
  the source notes and is not retained in this directory.
- **Integrated flight behaviour is not established by these Linux tests.** The new control tests,
  plus the limited real-module `load_dc` checks (`require('eech_dc')` loads the native module, and
  `pcall(dc.boot, 'not a table')` returns the invalid-boot-argument error), do not establish
  integrated flight behaviour. The native pack and census below are the integrated evidence.

## Reproduction

From `eech-campaign/`. The pinned image (`tools/Dockerfile`, digest above) has neither rustfmt nor
clippy, so install both in a disposable (`--rm`) container first:

```
rustup component add rustfmt clippy
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --offline --locked -p eech-world -p eech-engine -p eech-dc --no-fail-fast
```

`--offline` needs a populated cargo registry cache (here the `eech-cargo` volume). For a fresh
setup, use `cargo test --locked -p eech-world -p eech-engine -p eech-dc --no-fail-fast`. The fmt
record here came from a local rustfmt 1.9.0-stable, not from the image.

The native build is the release build for `x86_64-pc-windows-gnu` at `b721ae5a`
(`reference/m5-aircraft-control/build-native.sh`, which holds local paths and the image digest);
the regression pack is run per [`m4-regression-pack.md`](m4-regression-pack.md).

## Native results (Windows retail host, 2026-10-23)

Retained in [`../reference/m5-aircraft-control/`](../reference/m5-aircraft-control/reference-summary.txt)
(`native/`, `ci/`); the run directory also holds the traces, roots and logs, which are not in Git.

| Item | Result |
|---|---|
| Provenance | Binaries `b721ae5a` (tree `1b0a2ca9`, 0 dirty build inputs, pinned image `sha256:bb10e44a…e562be`). The pack tool and its report revision are the `10b02621` worktree; the binaries are the same ones reported by the pack's `build` block. No claim that the binaries are byte-equal to a build of `f1371dda` or `10b02621`. |
| Regression pack | `native/report.json`, `report.md`: verdict **PASS**, `pack.exit` 0, **40/40 checks PASS** (`regress.georgia.exact`, `regress.lebanon.exact`, `reference.repeatable`, `reference.observed_equals_unobserved`, the retained recording/observation/check/chain/loss checks and the lifecycle checks). Claims: 17 PASS directly protected, 12 PASS baseline-sensitive, 8 NOT COVERED and 8 EVIDENCE ONLY, the pack's declared gaps, unchanged. Baseline `windows-retail v1`, not modified. |
| Full-file comparison | `native/independent-reference-hashes.json`, both runs: `recording.acmi` (117,690,861 B, sha256 `3bd17b64…d19b`) and `observations.jsonl` (85,716,447 B, sha256 `da26bab0…77cf`) are byte-equal to the retained `reference/lebanon-3h` Tacview and `reference/m3-resupply-loop` supply-detail observations; `metrics.json` is equal as JSON values and after CRLF normalisation. The earlier `completed-reference-comparison.json` covered the completed reference processes only and is kept as that partial record. |
| Tacview | "Unchanged" means byte-identical, a structural/hash claim. **No visual inspection in Tacview is claimed.** |
| Census | `native/census.out`: 3 h, exit 0, `ended_at 10800.0`, same binaries, isolated Lebanon root copy. Fixed-wing: 127 identities, 104 moved at least 10 m; taxi 809,413 / flight 1,996,817 / land 22,294 steps; `fixed_wing_lookahead` 1,230,916; `fixed_wing_move` 2,828,524. Helicopter plan/guide 3,488,618 and guided begin/end 3,408,914 steps. |
| Census limits | Helicopter steps print as `unbound` because the script has no helicopter kind label (a missing label, not a registration failure). Displacement is tracked for fixed-wing only, so `moved_10m=0` for other kinds is not "stationary". No `ship_move` observed. Counts show exercised modes, not every avoidance conditional or numeric boundary. It is coverage evidence, not acceptance evidence. |
| CI at `10b02621` | `ci/pr98-checks-10b02621.json`: all 8 checks SUCCESS (test x86_64, test i686, i686 with `EECH_ORIGINAL_STACK_ATTRIBUTES=1`, x86_64 with `EECH_C_OPT=2`, evidence-selftest, dcs-probe-tools, windows-cross, eech-core-ts verify). The `windows-cross` apt-source workflow fix is confirmed on the real runner; earlier attempts had timed out in apt. CI is not evidence for the retail pack. |

Reproduce (local retail roots and the built `target/aircraft-control` required; paths are inside
the scripts): `reference/m5-aircraft-control/native/run-pack.ps1` runs
`tools/regression-pack.py --georgia … --lebanon … --lebanon-repeat … --bin target/aircraft-control --out …`
(see [`m4-regression-pack.md`](m4-regression-pack.md)); `run-census.py` runs
`eech-world.exe lua/fixed-wing-movement.lua root=… mode=census hours=3`;
`verify-complete-reference.py` re-hashes the run outputs against the retained references.

## Author self-review of the docs-only closeout

Author self-review, not independent review. The closeout changes this file and
`reference/m5-aircraft-control/` only: no source, workflow, test, baseline or golden change, and no
retail data or binaries (hashes, counts and reports only). Results are copied from the run
directory unedited; earlier unsuccessful or partial records (the failing clippy listing, the
partial reference comparison, the pre-pack statements in the build, pre-push and census-assessment
notes) are kept with their original wording and marked as dated in the reference summary. The
parent is the independent reviewer and publishes, approves and merges.

## Limits and what remains

- Not shown: airborne coverage beyond the pack's scenarios and the census, per-conditional
  avoidance and numeric-boundary coverage, host arithmetic equivalence beyond the exact Lebanon and
  Georgia comparison, the C/aphavoc build outside the pinned MinGW and Linux builds, any visual
  Tacview review, and any live DCS behaviour (M6). No native rebuild exists for `f1371dda` or
  `10b02621`.
- Declared coupling remains: campaign policy, terrain (lookups, sector structure data and the
  waypoint altitude feeding `min_height`), the entity heap (a mirror), RNG and effects stay in the
  DLL. TAXI, ABSOLUTE and LAND re-fetch the waypoint, so they issue extra non-mutating look-ahead
  queries. `fw_draw` debug shows the unadjusted y in a host build.
- D1 follow-ups, not part of this Task: dependent-force positioning and all-family report-loss
  work. D1 closure needs its own reconciliation and review.
- Red evidence for R1 remains limited, as described above.
- Technical-lead approval (@fd-starscream-bot) has not been given.
