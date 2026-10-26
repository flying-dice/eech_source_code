# D2 combat and perception — 25 October 2026

D1 / EEDCS-15 is complete at integration a7934aa7d1bcffc2ea5057bb3f1b0af332606c61.
All 15 D1 tasks are Done. D2 / EEDCS-18 is now In Progress; first extraction
EEDCS-42 moves AI line-of-sight observations to the host while keeping target
selection and engagement policy in EECH_DC. Live authority: M5 goal 11534337 v3
and migration ledger 14548994 v26. All five epic report macros are preserved.

Accepted D1 evidence: native 40/40, full Tacview and observations unchanged, metrics
equal; nine CI checks and independent exact-head approval before PR100 merge.
Coverage limits are preserved in the acceptance record and closure audit.

Workspace: codex/d2-perception-los from verified integration a7934aa7, plus the
documentation-only handover cherry-pick 756ed6326. Local regression bundle:
C:/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/eech-regression-20261025.
All 3,581 payload hashes pass; Georgia 1,174 and Lebanon/repeat 1,179 input files match
their checked-in manifests exactly. Read docs/handover/HOST-READINESS-20261025.md.

Now: one Claude CLI 2.1.310 implementation context
ae34a3d7-0931-4638-b06a-a2d399277929, exact claude-sonnet-5-5 High via -p, has
stopped at uncompiled characterization harness commit 676ab794c after one
independent Codex source review and corrections. Both print-mode processes exited;
no implementation/build/regression process remains running. No physical behavior
has moved and no D2 Tacview comparison has run. D1 experiments were not repeated
for readiness. See docs/handover/D2-LOS-HARNESS-REVIEW.md for evidence and limits.

Build prerequisite: existing Docker Desktop first-run Subscription Service
Agreement awaits user handling; Linux daemon is not ready. Windows 11 x64 and
Python 3.12.14 are available. No numerical toolchain substitution or authentication
change made. Both GitHub connections identify fd-starscream-bot; normal delivery
also needs a distinct authorized PR-author identity, without self-approval.

Next: compile the prepared los-fixture harness and exercise original-behavior
characterization with the documented toolchain,
then bounded LOS extraction, focused boundary evidence, clean shipping build,
established deterministic pack and complete Tacview/observation/metric comparison,
independent delivery review and normal merge. EEDCS-42 and D2 remain incomplete.

Later: remaining D2 work, then D3-D5 and M6 live DCS operation. Call-map PR83 is
unfinished historical inventory and needs a current-source refresh. Historical
ee-dcs TypeScript-port work is unrelated preservation, not the current roadmap.

## EEDCS-42 implementation lane (Claude, Sonnet 5.5) — updated 25 October 2026

Branch `codex/d2-perception-los`. **EEDCS-42 and D2 are not Done; no PR exists.** The lead owns publication, Jira and
ledger status, the final shipping build and 40/40 pack, and the independent review.

**Stage:** characterisation harness written; **nothing compiled or run; extraction not started** (rule: only after the
baseline behaviour is exercised).

| Step | State |
|---|---|
| Source characterisation of `mb_tgt.c` and every caller | done (read, below) |
| Test-only harness, feature `los-fixture` | written, **uncompiled** |
| Baseline build with the harness, baseline run, retained outputs | **blocked: Linux Docker daemon not ready** |
| Extraction, focused boundary tests, candidate comparison | not started |

**Blocking prerequisite (exact).** `docker info` / `docker ps` return `500 Internal Server Error` on
`//./pipe/dockerDesktopLinuxEngine` (checked repeatedly 25 Oct 2026; first-run agreement pending per the lead). The host has
no gcc, rustc, cargo, clang or WSL distribution, and none was installed. Every C and Rust edit below is unverified until the
`tools/Dockerfile` image can run.

### Source characterisation (read, not run)

- `check_entity_line_of_sight` (`mb_tgt.c:111`) reads both positions, raises a position 1.5 m when
  `INT_TYPE_IDENTIFY_VEHICLE`, and calls `check_position_line_of_sight`, which is the solver (`mb_tgt.c:138`).
- Criteria are the five bits of `mobile_los_check_criteria`. Order, with an immediate FALSE on the first failure:
  1. direction = target − source; `target_range = sqrt (dx²+dz²)` (double sqrt, narrowed to float); `normalise_3d_vector`.
  2. COARSE terrain: 200 m steps while the float counter `target_range/200 > 1`, from the source; `terrain_info` is zeroed here only.
  3. SOURCE-END terrain: 40 m steps from the source; the counter is `min (range/40, 8)` and the loop runs while it is `> 1`,
     decremented by 1, so at most 7 terrain queries.
  4. TARGET-END terrain: 24 m steps walking back from the target; counter `min (range/24, 8)`, same loop, at most 7 queries.
  5. SOURCE-END objects: one `get_line_of_sight_collision_entity` over `min (range, 200)` from the source.
  6. TARGET-END objects: the same from the target; a hit blocks only when `get_sqr_3d_range (collision_point, target) > 12²`.
  A terrain sample is `ASSERT (point_inside_map_area)`, `get_3d_terrain_point_data (x, z, &terrain_info)`,
  `get_3d_terrain_point_data_elevation`; blocked when the elevation exceeds the sample height.
- The step counter is a float decremented in the loop. The engine runs under round-toward-zero (`FE_TOWARDZERO`) and a host
  callback inherits that mode (docs/fpu.md, docs/m5-convoy-movement.md), so any host arithmetic must reproduce each
  expression at declared float type (as `crates/eech-world/csrc/hc_step.c` does), not use Rust floating point.
- Criteria at the callers: `vh_tgt.c:709/716` and both `ac_wpn.c` sites (`:336`, `:594`) pass `MOBILE_LOS_CHECK_ALL` for a
  non-AI target and `MOBILE_LOS_CHECK_COURSE_TERRAIN` (coarse terrain only) for an AI target; `wn_tgt.c:302` always passes
  COURSE_TERRAIN; `vh_wpn.c:319`, `ag_attck.c:702,1097` and `gd_cover.c:762,816` pass ALL. So AI-versus-AI target
  selection and aircraft weapon choice use only the coarse terrain phase. Policy callers, which stay in the DLL: `vh_tgt.c:709,716` (vehicle target scan),
  `vh_wpn.c:319`, `wn_tgt.c:302`, `ac_wpn.c:336,594` (aircraft weapons), `ag_attck.c:702,1097` and `gd_cover.c:762,816` (guide
  placement). `eech_engine_debug_engagements` calls it but is never invoked.
- No existing Lua-visible entry exposes a line-of-sight answer, so the supplied bundle DLL cannot characterise it without a
  harness build. The bundle binaries remain the unmodified baseline `a7934aa7`.

### Harness (uncompiled)

Test-only feature `los-fixture`, in the pattern of `loss-fixture` and `dependent-fixture`. Everything sits behind
`EECH_LOS_FIXTURE` or the cargo feature, so the runtime paths of a build without the feature are unmodified (the files differ in
text only inside those gates). **No build, no run and no Tacview comparison exists for this harness.**

- `mb_tgt.c` (gated): the public `check_position_line_of_sight` forwards to the unchanged solver (renamed by macro) and
  records the inputs and the answer. Entity and position queries both pass through it.
- `csrc/eech_los_fixture.c`: a run-wide trace (count, visible, per-criteria counts, order-dependent FNV-1a over criteria, six
  position float bit patterns and the answer) and a deterministic probe sweep (private xorshift generator and private terrain
  point-data record, so neither the game's RNG nor shared terrain state is touched): segments of 25 m to 15 km whose lengths
  bracket the 200 m coarse step (150, 199.5, 200, 201, 399, 450 m among them) but **not** the 40 m or 24 m fine steps, at five
  heights over the real terrain, each with one of **eight chosen criteria masks** (ALL, each of the five bits alone, the three
  terrain bits together, NONE; not every subset of the five bits), then the entity form for each mask both ways round. The
  fine-terrain and object phases are therefore exercised only through ALL, their single bits, and the terrain-bit mask, with
  no designed threshold coverage of the fine steps.
- Review corrections (a0b624646 review): the probe entry refuses an index outside the allocated `number_of_entities` (not
  `MAX_NUM_ENTITIES`), a NULL heap, and any entity that is not one of the six mobile families (bad argument, visible to Lua);
  the driver asserts the run-fold invariant, fails when a requested probe time finds no opposed pair or is not reached, and
  treats an explicitly empty `probe_at=` as the only no-probe control. Pending negative cases (out-of-range index, weapon or keysite
  index, free slot, same entity twice, no pair, unreached time): the refusals are implemented but **no executable check for them
  exists yet and none has run**; they need the build.
- `eech_engine.c/.h` (`eech_engine_los_trace`, `_reset`, `_probe`; `eech_engine.c` holds one raw NUL byte, so it was edited
  binary-safely), `eech-engine-sys`, `eech-engine`, `eech-dc` (`engine:los_trace ()`, `:los_trace_reset ()`,
  `:los_probe (source, target, seed, count)`), and the driver `crates/eech-world/lua/los-characterization.lua`.
- Planned baseline run, on a build of this commit with `--features eech-dc/los-fixture`: `eech-world.exe
  los-characterization.lua root=<copied Lebanon root> scenario=lebanon_retail seconds=600 probe_at=30,300 out=<dir>/baseline.jsonl`;
  repeated with `probe_at=` empty to show the probe leaves the run-wide fold unchanged; and on the generated Georgia root.

### Planned boundary (not implemented)

Host-owned solver, DLL-held environment inputs. A `line_of_sight` host callback carries a small dialogue: the DLL sends
`begin {source position, target position, criteria}`; the host answers `need_terrain {x, z}`, `need_objects {from, to}` or
`done {visible}`; the DLL replies with an elevation, or `{blocked, collision point}` from
`get_line_of_sight_collision_entity` (it holds the entity pointers; none cross). The host owns criteria gating, direction, step
counts, increments, comparison and the target-end exclusion, in the original order, so the DLL's lookup order is unchanged.
Payloads are positions, criteria, elevations and booleans. Terrain and collision answers are the declared transitional
coupling; when the host owns them (D3/D5) it answers `done` at `begin`. Open points to settle by measurement: queries and
samples per simulated hour (one Lua crossing per sample), and the cost of a required boot callback (one line in each of the ~17
host driver scripts).

### Pending validation (none run)

Harness build and baseline outputs; the same on the candidate; sociable checks that a blocking host and a clear host change
target selection and weapon choice against the baseline; refusal of malformed host answers; fmt/clippy/test in the pinned
image. The final shipping build and 40/40 pack are the lead's, not to be run concurrently with source edits.

### Next bounded action

When `docker info` succeeds: build the harness commit in the pinned image on a fresh target volume, run the baseline
characterisation, retain outputs under `eech-campaign/reference/d2-los-observation/`, fix compile errors in the harness only,
then begin the extraction.
