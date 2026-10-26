# D2 — EEDCS-43 sensor detection, stacked on EEDCS-42 line of sight — 26 October 2026

Branch `codex/d2-sensor-detections`, from reverified integration
`a7934aa7d1bcffc2ea5057bb3f1b0af332606c61`. Draft delivery PR:
https://github.com/flying-dice/eech_source_code/pull/102.
Authority: live M5 goal 11534337 v3 and migration ledger 14548994.
D1 remains accepted; D2 (EEDCS-18) is active and **incomplete**; D3 has not started. EEDCS-43 is incomplete.

## Current state in one place

* **EEDCS-42 (PR101) is NOT accepted or merged.** Its reviewed frozen head `a42f1cfad39e5b29ce18bfd2d4ff6cf0f5e0cf12` (runtime source `61e856645`; later commits docs/evidence only) is merged
  into this branch as **development stacking** so the sensor work can be built on the host-owned line of sight. Its supplied-binary baseline pack and its final-source candidate full pack are still
  running (the lead's). Delivery of EEDCS-42 must precede EEDCS-43: nothing here claims LOS accepted, a sensor predicate moved, or D2 complete, and this branch is not merged anywhere.
* **EEDCS-43 so far:** original-behaviour characterisation of the four AI scan functions with a test-only gated observer (below); no physical sensor behaviour has moved, every sensor formula,
  filter, ranking and random-input order is the original text.
* **Source composition stage** (see the section after this one): the observer is composed with the LOS extraction, and its public entries are given the required LOS callback context.

## Composition stage: observer + host-owned LOS (development stacking only)

Merge of `a42f1cfad` into `f22269490` (the sensor characterisation and comparator-fix head). The ten textual conflicts were resolved by keeping both sides: the `los-fixture`
and `sensor-fixture` features, defines, FFI and engine entries side by side. The sensor probe entry now takes the complete host (the surface-to-air scan reaches the required `line_of_sight`
callback) and the sensor driver boots with `line_of_sight = host.line_of_sight`. Build, checks and the exact 600 s witness against the retained original are recorded in later commits;
until then nothing about this composition is claimed as built or run.

## Stage: original-behaviour characterisation; evidence-tool correction; no host extraction

Nothing moved. The four AI scan functions (`ac_tgt.c` air-to-air and air-to-surface, `vh_tgt.c`
surface-to-air and surface-to-surface) are the original text; the only source additions are
test-only hooks that are empty macros unless the cargo feature `sensor-fixture` is on.

* Source: observer `60f6a4876e02d4ce4e6d32436930b69d82214260`; counters added in
  `e0d58d818661620ddfa7e386118551de8c191da0` (hooks unchanged, nothing that enters a hash).
  Fixture DLL at `e0d58d818`: `352f8f8563dc1f59266d06ec804de1420749e1dd5651c6d1091d686d99b693e8`;
  both fixture builds exit 0 from a clean tree (0 uncommitted inputs); fmt and clippy (default and
  `sensor-fixture`) exit 0 at both commits.
* Lebanon 600 s through the public Lua -> DLL path: physical hash `d60c462829c9714e`, decision hash
  `2611b04d99f5c689`, identical on two builds, two roots, with and without the read-only probe
  (75 run-wide events, 0 differences). Per-function tables, probes and samples are in
  `eech-campaign/docs/m5-sensor-characterization.md` and `eech-campaign/reference/d2-sensor-detections/`.
* Feature off preprocesses to the original text (`checks/preprocess-equivalence.log`).
* Hooks on, 900 s whole-campaign Lebanon run: Tacview recording, observations and metrics are
  byte-identical to the supplied-baseline-binary run the lead recorded (that side not re-run here);
  `observation-check.py` PASS. Not a visual Tacview inspection; not the 3 h pack.

## Evidence-tool correction (review feedback)

`tools/sensor-trace-compare.py` at `e0d58d818` was NOT acceptable: it exited 0 for empty files and for probe
snapshots with no final run report (two real red controls, retained in
`reference/d2-sensor-detections/controls/comparator-inputs/`). It now fails closed: exit 2 with a reason for a
missing, empty, malformed or incomplete file (the terminal `sensor-samples` event of scope `run` is required,
whatever scopes are selected and with `--skip-samples`) and for a selection that matches no event; exit 1 for a
difference; exit 0 only for complete files with a non-empty, equal selection. Actual Python exits are in
`controls/comparator-controls-result.json`: both red controls exit 2; the retained complete v2 pair `--scope run`
exits 0 (75 events, 0 differences); a complete probe trace against itself exits 0 (`--scope probe`, 182 events);
a changed-value copy exits 1; truncated, malformed, missing and empty-selection cases exit 2. No traces or
baselines were altered, no rebuild or campaign was rerun. The comparison of the original characterisation is not
claimed from zero or partial reports.

The 900 s control's configuration is now stated as verified versus inferred (docs section "Non-perturbation"):
verified from retained records that both runs used the same argument list (`hours=0.25 record_every=10
checkpoint_every=3600 observe_every=60 observe_supply=0`; seed 1 and 100 ms by default for both) and
byte-identical `campaign.lua`/`metrics.lua`/`observations.lua`; inferred that the original root was an
unmodified Lebanon copy. Identical bytes are the non-perturbation result for this configuration only.

## Boundary discovered (measured, not designed)

Physical facts per candidate: planar squared range (aircraft scans) or approximate 3-D range
(vehicle scans); target radar altitude against floor and ceiling (surface-to-air only); the
target's radar-stealth factor (a function of helicopter configuration); emitter and jammer state
(only for player-controlled targets); line of sight (already host-owned). Everything else in the
loops — side, airborne, type, group, best-group, alive, self filters, sector enumeration, ranking
and engagement — stays in EECH_DC. Two properties constrain extraction: the surface-to-air scan
floor is mutated cumulatively across candidates (288 of 2,444 evaluations differ from an
independent per-target floor; 606 calls evaluated it more than once), and the random tolerance
is a fixed per-source draw derived from a heap slot.

## Corrections to the task text, found in source and confirmed by measurement

* `cos_angle` is **not** a raw unnormalised dot product: `get_3d_vector_dot_product` normalises both
  vectors, so it is a true cosine (22 values in the run, all within [-1, 1]).
* The aircraft scans square the range before the sector bounds as stated: the rectangle is the whole map
  (1,676 of 1,676 air-to-surface calls), and for scan ranges above about 46.3 km (80 km observed, 179 of
  1,726 air-to-air calls) the squared range overflows `int` and the rectangle is empty, so the source
  finds nothing. Characterised, not fixed.

## Preconditions for the extraction

* Keep the required LOS entry wired: after EEDCS-42 the surface-to-air scan's LOS is the host-owned
  `campaign_los_query` dialogue (required `line_of_sight` callback, no fallback); this baseline predates it.
* The tolerance seed is a private heap index (`EN - entities`); never expose it or a seed derived from it publicly.

## Unexercised edges

Player-controlled targets (difficulty factor, emitter-off adjustments, all-criteria LOS form), altitude
above the ceiling, Georgia and other seeds, durations beyond 900 s. No player aircraft exists in the
headless world (0 of 265 aircraft entities at every probe).

## Handover

One Sonnet 5.5 High CLI lane, local context `67f40663-046f-435e-be4b-902073b9a69d`, worked here.
The lead manages publication, independent review, final full pack, normal merge and Jira/ledger.
The existing logged-in GitHub identity is used as directed; no authentication or credential
changes and no fabricated GitHub self-approval.

Predecessor EEDCS-42 / PR101 is source-complete and frozen in full validation; its source, binaries,
roots, logs and processes were not touched. This branch will incorporate accepted LOS integration
before moving sensor behaviour. Next bounded action: after that integration, choose the first
physical predicate to extract (range, altitude envelope, stealth, emitter/jammer), keep campaign
filters, sequence-dependent floor handling and ranking in EECH_DC, and compare against these
traces with `tools/sensor-trace-compare.py`. No accepted baseline or tolerance was changed.

## Predecessor EEDCS-42 / PR101 status (verbatim from its frozen head `a42f1cfad`; the gates it names still apply)

D1 / EEDCS-15 is complete at integration a7934aa7d1bcffc2ea5057bb3f1b0af332606c61.
All 15 D1 tasks are Done. D2 / EEDCS-18 is now In Progress; first extraction
EEDCS-42 moves AI line-of-sight observations to the host while keeping target
selection and engagement policy in EECH_DC. Live authority: M5 goal 11534337 v3
and migration ledger 14548994. All five epic report macros are preserved.

Delivery PR: https://github.com/flying-dice/eech_source_code/pull/101 (draft),
authored as fd-starscream-bot following the user's logged-in-identity direction.
The LOS physical solver and entity eye lift are implemented in the host at
`61e85664540770ab10e3ac7e9f5dfa916291c52f` (corrected candidate). No merge or full regression acceptance yet.

Accepted D1 evidence: native 40/40, full Tacview and observations unchanged, metrics
equal; nine CI checks and independent exact-head approval before PR100 merge.
Coverage limits are preserved in the acceptance record and closure audit.

Workspace: codex/d2-perception-los from verified integration a7934aa7, plus the
documentation-only handover cherry-pick 756ed6326. Local regression bundle:
C:/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/eech-regression-20261025.
All 3,581 payload hashes pass; Georgia 1,174 and Lebanon/repeat 1,179 input files match
their checked-in manifests exactly. Read docs/handover/HOST-READINESS-20261025.md.

Now: one Claude CLI 2.1.310 context `ae34a3d7-0931-4638-b06a-a2d399277929`, exact
`claude-sonnet-5-5` High via `-p`, completed and exited cleanly. Both original/candidate
600-second characterizations match: Lebanon 156,581 queries / 60,731 visible / hash
`d97f59bb364bb253`; Georgia 21,104 / 1,646 / `7fdc5ab0b8b5db5d`. The frozen original40s
ordered lookup transcript and candidate are identical (4,567 queries / 107,162 lookups).
Public-path 300s reference/blocked/clear host visibility changes actual weapon births and losses
(185/6, 176/4, 254/15). Missing/malformed providers fail clearly. Exact source, builds, inputs,
commands and limitations are retained under `eech-campaign/reference/d2-los-observation`.

One independent Codex delivery source pass found no blocking shipping-source defect; its P3
duplicate callback setup in `dependent-fixture` is resolved in 61e856645. The same reviewer checked
the two-line correction and actual all-fixture/old-guard Clippy results (cargo/Docker exits 0).
See `docs/handover/D2-LOS-DELIVERY-REVIEW.md`. Default/LOS-feature tests, Clippy and fmt passed.
The lead built ordinary-feature shipping binaries from an immutable clean 61e856645 snapshot,
fresh target volume and unchanged recipe (exit 0, no dirty build inputs). Exact hashes are in
`eech-campaign/reference/d2-los-observation/extraction/checks/final-shipping-BUILD-INFO.txt`. Shipping symbols place the LOS kernel in
World, with no trace/probe fixture symbols in the DLL. The full supplied-binary destination pack
is still RUNNING at `R:/d2-evidence/baseline-host-20261026-shortpath`; checked prefixes match,
but no completed full-pack pass is claimed. The separate 900-second A/B comparison completed:
Tacview, observations and metrics are byte-identical, independently checked by the lead. The
final shipping candidate's full pack is RUNNING at `R:/d2-evidence/candidate-pack-los-61e856645`.
The short A/B comparison does not replace it.

Build setup is resolved through isolated WSL2 distro `EECH-D2-Build` and the unchanged repository
Dockerfile (Rust/cargo1.99.1, MinGW12-posix, Lua5.1.5; exact image and hashes recorded per build).
This Rust is newer than the supplied baseline toolchain; equivalence is checked at runtime.
Docker Desktop is not needed. Windows11x64 and Python3.12.14 are available. No authentication
change made. Both GitHub connections identify fd-starscream-bot. On 26 October,
the user directed that the PR author match the logged-in GitHub identity; use
fd-starscream-bot for this task's PR publication. That task-specific direction
supersedes the earlier request for a distinct author connection. No authentication
change is needed. Do not attempt GitHub self-approval; keep the independent
review and actual remaining delivery gates explicit.

Next: finish the established full baseline/candidate packs and complete-file Tacview/observation/
metric comparisons, current-head CI and normal merge. The lead owns these steps autonomously.
No baseline or tolerance changed. Terrain elevation and object collision services remain coupled;
target selection and engagement policy remain in EECH_DC. EEDCS-42 and the rest of D2 are incomplete.

Later: remaining D2 work, then D3-D5 and M6 live DCS operation. Call-map PR83 is
unfinished historical inventory and needs a current-source refresh. Historical
ee-dcs TypeScript-port work is unrelated preservation, not the current roadmap.

### Historical implementation checkpoints (current status above)

#### EEDCS-42 implementation lane (Claude, Sonnet 5.5) — updated 25 October 2026

Branch `codex/d2-perception-los`. **EEDCS-42 and D2 are not Done.** This is the earlier implementation checkpoint;
the current PR publication status above supersedes its at-the-time lack of a PR. The lead owns publication, Jira and
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

#### Source characterisation (read, not run)

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

#### Harness (uncompiled)

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

#### Planned boundary (not implemented)

Host-owned solver, DLL-held environment inputs. A `line_of_sight` host callback carries a small dialogue: the DLL sends
`begin {source position, target position, criteria}`; the host answers `need_terrain {x, z}`, `need_objects {from, to}` or
`done {visible}`; the DLL replies with an elevation, or `{blocked, collision point}` from
`get_line_of_sight_collision_entity` (it holds the entity pointers; none cross). The host owns criteria gating, direction, step
counts, increments, comparison and the target-end exclusion, in the original order, so the DLL's lookup order is unchanged.
Payloads are positions, criteria, elevations and booleans. Terrain and collision answers are the declared transitional
coupling; when the host owns them (D3/D5) it answers `done` at `begin`. Open points to settle by measurement: queries and
samples per simulated hour (one Lua crossing per sample), and the cost of a required boot callback (one line in each of the ~17
host driver scripts).

#### Pending validation (none run)

Harness build and baseline outputs; the same on the candidate; sociable checks that a blocking host and a clear host change
target selection and weapon choice against the baseline; refusal of malformed host answers; fmt/clippy/test in the pinned
image. The final shipping build and 40/40 pack are the lead's, not to be run concurrently with source edits.

#### Next bounded action

When `docker info` succeeds: build the harness commit in the pinned image on a fresh target volume, run the baseline
characterisation, retain outputs under `eech-campaign/reference/d2-los-observation/`, fix compile errors in the harness only,
then begin the extraction.

#### Update 26 October 2026 (implementation lane)

- Baseline: the lead is running the pack with the unchanged supplied Windows binaries
  (`../d2-evidence/baseline-host-20261026`). No extraction or solver change until the lead confirms it passes; no second
  pack is started from this lane. Delivery is user-directed, autonomous under the lead (PR101, fd-starscream-bot author);
  no self-approval and no authentication change.
- PR101 CI: the one `cargo fmt --check` diff (the `eech_engine_los_probe` declaration) is applied alone in `7ef45829a`.
- Fixture build (not run; Docker unavailable): from a clean checkout in Git Bash with Docker Linux containers,
  `sh eech-campaign/tools/build-windows-los-fixture.sh target/los-fixture/<dir>` (run from `eech-campaign/`). It runs the
  unchanged `tools/build-windows.sh` and Dockerfile image from a temporary copy in the ignored `target/` directory whose
  only two differences are `--features eech-dc/los-fixture` on the cargo line and a `FIXTURE BUILD` stamp in `BUILD-INFO.txt`
  (the wrapper refuses otherwise). Fresh cargo volume `eech-win-target-los-fixture` (override with `EECH_WIN_TARGET`); the
  shipping volume and flags are untouched. Whether `--features eech-dc/los-fixture` resolves with `-p eech-world` in the
  workspace is unverified until it runs.
- Next, when the lead says baseline and Docker are ready: build with that command, run `los-characterization.lua` from the
  fixture output on the copied Lebanon and Georgia roots (see the harness section), retain outputs, then extract.

#### Update 26 October 2026, fixture compiled (implementation lane)

- Docker now works in the lead's isolated WSL2 distro `EECH-D2-Build` (same `tools/Dockerfile` image). The fixture build of
  `da8b8c327` compiled first time: exit 0, `Finished release in 1m 11s`, no compile errors, so no source fix was needed.
  Output `eech-campaign/target/los-fixture/baseline-20261026-wsl` (ignored; `BUILD-INFO.txt` and notes retained under
  `eech-campaign/reference/d2-los-observation/`). Toolchain recorded by the build: rustc/cargo 1.99.1, MinGW gcc 12-posix
  (`12.2.0-14+deb12u1+25.2+b1`). FIXTURE BUILD: never a shipping artifact. `cargo fmt --all --check` clean; clippy with the
  feature, no diagnostics; the driver parses. **Not run: any characterisation, campaign, `cargo test` or the pack.**
- The lead's baseline pack (unchanged supplied binaries) is still pending its byte-equality result. No characterisation is run,
  no solver or policy changed, until the lead confirms it. Next: run `los-characterization.lua` from the fixture output on
  copied roots when told, retain outputs, then extract.

#### Update 26 October 2026, unchanged-solver characterisation done (implementation lane)

- Clippy result settled fail-closed (cargo exit 0, no pipeline). Fixture B `ee7f6f79b` adds entity-kind counters and
  `los-controls.lua`; fixture A `da8b8c327` and B both built in the WSL image, no compile error.
- Characterised the ORIGINAL solver on private root copies (manifests IDENTICAL before and after; none used by the lead's pack):
  Lebanon 600 s with and without probes and a third copy with kinds, and Georgia (`georgia_retail`, map3). Lebanon run-wide fold
  identical across all three (156,581 queries, 60,731 visible, hash `d97f59bb364bb253`); Georgia 21,104 / 1,646,
  `7fdc5ab0b8b5db5d`. The probe leaves the fold untouched. The campaign requests only masks 1 (coarse, 70%) and 31 (all, 29%); the
  fine-terrain and single-object phases are reachable only by the probe. The two probe times gave identical pairs and hashes, so
  three distinct sweeps per map. Boundary controls: 8 refused, 0 wrongly accepted, valid probe works after; weapon-as-source
  unavailable; unreached time and no-pair fail the driver (exit 1). Weapon targets (397 queries) have no host id. Details,
  identities and limits: `eech-campaign/reference/d2-los-observation/characterization-20261026.md`.
- Still pending and not claimed: the lead's full 3 h baseline pack and its byte comparison. No extraction started.
- Next bounded extraction (after the lead confirms the gate): a host `line_of_sight` callback carrying positions and a criteria
  mask only (no entity identity while collision stays in the DLL), the host-owned coarse and fine terrain arithmetic in C
  beside `hc_step.c`, the DLL answering terrain and collision lookups in the original order; candidate checked by the same fold
  hash, probe and a blocking-versus-clear host change of weapon choice.

#### Update 26 October 2026, LOS extraction candidate (implementation lane)

- The physical line-of-sight calculation is moved behind a required host callback: `a7863a73fa8bfba56cfa6bac4bb1602e9b9b88eb` (code and tests; the evidence
  commit that follows records the results). Kernel in `eech-world/csrc/los_step.c`, dialogue in `mb_hostlos.[ch]`, record in `docs/m5-los-observation.md`.
  Policy callers are untouched; terrain (D3) and object collision (D5) stay the campaign's declared coupling.
- Original lookup transcript frozen first (source `1387331fd`, `original-transcript/`). After the move the 40 s trace is byte-identical (fixture candidate `becb9d5d...`);
  600 s Lebanon and Georgia probes and run totals equal; a 900 s whole-campaign comparison against the supplied baseline binaries is byte-identical in Tacview, observations
  and metrics (about +6% wall). Causal runs: blocked / reference / clear host give 4 / 6 / 15 losses; missing and six malformed providers fail clearly.
- Shipping candidate built (not the lead's final build): `eech_dc.dll` `37502d73...6c86`, `eech-world.exe` `d280f2cb...5f47`, source `a7863a73f`, rustc 1.99.1, MinGW gcc 12-posix.
  fmt, clippy (default and fixture features) and tests exit 0. Symbol check: no solver in `mb_tgt.o` or the DLL; kernel only in `eech-world.exe`.
- Pending, the lead's: final shipping build, the established pack with complete-file comparison against the baseline run, one independent review, normal merge and Jira/ledger.
  Not claimed here: M6 live DCS operation, D3-D5.

- Review finding P3 (duplicate `line_of_sight` and `sighter` declarations in the `dependent-fixture` method of `eech-dc`, from my replace matching the
  inner indentation) is fixed in the commit after `815b07aee`: both declarations are single again. It sits entirely behind the `dependent-fixture` cargo
  feature, so the default shipping DLL and host are unchanged by it. Checked with cargo's own exit status: `cargo clippy --locked -p eech-engine -p eech-dc -p eech-world
  --all-targets -- -D warnings` with `eech-dc/dependent-fixture,eech-dc/loss-fixture,eech-dc/los-fixture` and separately with `eech-dc/dependent-fixture-old-guard`: both
  exit 0; `cargo fmt --all --check` exit 0. The lead's final shipping build should be of the corrected head, not `a7863a73f`.
