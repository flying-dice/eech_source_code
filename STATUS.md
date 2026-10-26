# D2 — EEDCS-43 sensor detection, stacked on EEDCS-42 line of sight — 26 October 2026

Branch `codex/d2-sensor-detections`, from reverified integration
`a7934aa7d1bcffc2ea5057bb3f1b0af332606c61`. Draft delivery PR:
https://github.com/flying-dice/eech_source_code/pull/102.
Authority: live M5 goal 11534337 v3 and migration ledger 14548994.
D1 remains accepted; D2 (EEDCS-18) is active and **incomplete**; D3 has not started. EEDCS-43 is incomplete.

## Current state in one place

**Current lead gate:** all source findings are resolved at reviewed runtime
`a2a041fecf52e18cbc16b33f3dc75a9c9b245c9d`. Same reviewer completed only focused
new deltas. New generic fixture300s and three short cases exit0; all4 scan counts
agree with shipping. Root independently checked83 original30s checkpoint events
byte for byte. The final ordinary build exited0, clean/fresh, and all26 manifest
hashes match. **Full pack is RUNNING**, exec89376 at
`R:/d2-evidence/candidate-pack-sensing-a2a041fec`, with explicit native R: cwd and
frozen snapshot/binaries/roots. No40/40 or full-byte acceptance yet. Earlier
fixture-finding/runtime-build pending paragraphs below are superseded by this gate.

**Accepted predecessor update:** PR101 merged normally at
`3ce17432e4026f85b19ebf06cc582314817ddac2` on26October16:27:19UTC after all9CI passes.
EEDCS-42 is Done (Jira comment10773); Confluence ledgerv34 preserves all5 epic reports.
All40 checks passed and all8 complete scenario artifacts match the supplied baseline.
The original exit1/report-only recovery0 distinction remains recorded and independently checked.
This worktree still has the earlier reviewed development stack; root will incorporate the
accepted merge after this CLI actually finishes. Do not mutate a valid run or final frozen
snapshot. Root's immutable cleana2a041fec final shipping build is running (exec2774);
source follow-up found no remaining issue, while new green/check records are being inspected.
The earlier predecessor-pending paragraphs below are historical and superseded here.

Lead update after the focused5c00 source follow-up: P1 resolved and P2 shipping
transport resolved, but a new **P2 fixture issue remains**. A valid `not_perceived`
result must not `debug_fatal` merely because observational `sensor-fixture` is on.
Read the latest disposition at the end of `docs/handover/D2-SENSOR-DELIVERY-REVIEW.md`.
Observe generic nondetection separately without faking a legacy predicate or
changing the existing443-event reference oracle. Preserve/finish the in-progress
shipping controls; only a short focused fixture check is needed for this new fix.
The same reviewer will check only its new delta. Do not declare source closure
or launch the final pack until that issue is resolved.

* **EEDCS-42 (PR101) is NOT accepted or merged.** Its reviewed frozen head `a42f1cfad39e5b29ce18bfd2d4ff6cf0f5e0cf12` (runtime source `61e856645`; later commits docs/evidence only) is merged
  into this branch as **development stacking**. The supplied-binary destination baseline completed all 40 checks, runner exit 0, and all 6 complete reference files match accepted D1 bytes.
  Its final-source LOS candidate full pack is still running. Baseline evidence is published at `de987c471`; all 9 checks pass there, and ledger v33 preserves all 5 epic reports.
  Delivery of EEDCS-42 must precede EEDCS-43; neither task nor D2 is accepted by this status.
* **Independent source review has happened; its findings are corrected and await the same reviewer's follow-up on the delta** (see "Review corrections" below): P1 `647558f8b`, P2 `5c00ab4fa`, follow-up P2 (fixture) `a2a041fec`. Not accepted; no source-closure claim until that follow-up.
* **EEDCS-43 is source-complete as extracted (pre-review pin) and validated on the bounded evidence below, not accepted:** `37989fa5f` moves the physical sensing of the four AI target scans (planar and approximate 3-D range, the surface-to-air envelope with its
  cumulative floor, the tolerance limit, the emitter and stealth adjustment) to the host behind a sixth required callback `sense`; campaign filters, enumeration, the random draw, ranking and engagement remain core.
  The original 600-second oracle holds across the move (below). The final full pack, the independent delivery review and merge are the lead's; EEDCS-42 must be delivered first; D2 is incomplete.
* **Original characterization and composition are complete:** the original 600-second reports, observer 900-second recording control and combined LOS/observer 600-second trace match as recorded below.
  Root's live process/source checkpoint is `../d2-evidence/ACTIVE-RUNS.json`. Sole active implementation context: `67f40663-046f-435e-be4b-902073b9a69d`, exact Sonnet 5.5 High via CLI `-p`.

## Review corrections (EEDCS-43), head `a2a041fec` — development stacking, not accepted

Account: `eech-campaign/docs/m5-sensor-characterization.md` ("Review corrections"). Evidence, hashes, commands: `eech-campaign/reference/d2-sensor-detections/review-corrections/README.md`. The review record with its original findings, the lead's dispositions and the resolution status is `docs/handover/D2-SENSOR-DELIVERY-REVIEW.md`.

* **P1 resolved (`647558f8b`).** `eech-engine-sys/build/main.rs` registers `modules/` and `aphavoc/source` under the resolved `EECH_SOURCE_ROOT`. On one never-cleaned cargo target: the old script ignores an original-file edit (staged copy stale, RED); the new one reruns on an edit of a `.c`, a `.h` and `modules/3d/textname.h`, and not on an unchanged build (GREEN).
* **P2 resolved for shipping (`5c00ab4fa`).** Generic reason `not_perceived`, accepted for every scan; the EECH predicates remain the reference host's diagnostics; unknown reasons still refused; comments say the supplied facts are transitional mirrors, not a campaign-owned solver. Public-path shipping runs (Lebanon, 300 s, one each): `not_perceived` for every candidate equals the retained `blocked` counters
  (17,047 begins, 324,946 candidates, 176 births, 1 loss; all four scan kinds); a host model of its own (distance only, no draw/signature/seed/entity index) gives 199 detections, 180 births, 9 losses. fmt 0, clippy 0 (default, sensor-fixture), `cargo test --workspace` 0.
* **Follow-up P2 (fixture) corrected (`a2a041fec`).** Red: the `5c00ab4fa` fixture stops after 6 s on a valid `not_perceived`. Green: the fixture counts it apart (rows and samples only when a host answered it), skips the reference-only hooks of that exchange, folds nothing into either hash; 60 s and 300 s runs exit 0 (300 s: 6,245 / 312,160 / 520 / 6,021 generic verdicts,
  every table verdict 0); a reference run records no such row and its 83 events at the 30 s probe equal the retained original's byte for byte (ad-hoc text comparison, not the strict tool). The 443-event oracle and expected counters are unchanged and were not re-run. Shipping objects unchanged except four `debug_fatal` line-number immediates. fmt 0; clippy 0 for default, all fixtures, old-guard+sensor, sensor alone; symbol check clean.
* **Builds:** clean fixture and shipping builds of `a2a041fec` (0 uncommitted inputs, fresh volumes `-5`): fixture `eech_dc.dll` `e5f3674b...6ef3`, shipping `ab42d053...968b`. The final shipping build, full pack and the reviewer's follow-up on this delta are the lead's.
* **Not repeated, by decision:** the 600 s witness, the 900 s control, the retained reference/blocked/far/malformed records. Not claimed: a DCS host (the host models here are mine), Georgia, player-controlled targets, visual Tacview inspection.

## Extraction result (EEDCS-43), pinned `37989fa5f` — development stacking on EEDCS-42, not accepted

Account: `eech-campaign/docs/m5-sensor-characterization.md` Part 1. Evidence, hashes and commands: `eech-campaign/reference/d2-sensor-detections/extraction/README.md`.

* **Red first.** The retained composed original `1fc0cdc62` booted with the new provider (`blocked`, 300 s), ran, and consulted it 0 times (185 weapon births, 6 losses): a concrete missing behaviour. Pure relocation itself was held to passing characterisation.
* **What moved / stayed / is transitional coupling:** see the document. No second authoritative physical predicate remains in the DLL (symbol check: the scan objects no longer reference the range functions; the kernel is in `eech-world.exe`).
  The draw is the campaign's and is taken at the original conditional point; the heap-index seed never crosses. Positions, radar altitude, helicopter configuration, emitter states, the difficulty factor and the unit database figures are declared facts; terrain and collision for line of sight remain D3/D5.
* **Oracle held.** The 600 s Lebanon witness (probes 30/300/590, seed 1) on the fresh-volume fixture build `37989fa5f`: native exit 0 after 763 s; physical `d60c462829c9714e`, decision `2611b04d99f5c689`; the raw trace is byte-identical to the composed original's;
  the corrected comparator exits 0 with 0 differences for run (75), probe (182), run-at-probe (186) and default (257) events. Probe invariant true at all three probes.
* **First causes of the two earlier differences** (retained as failed evidence): a one-ulp-low tolerance limit from the existing `exact_f64s` reader under round-toward-zero (4,010 of 60,000 sampled doubles, 3,199 of 32,768 draws; fixed for the sensing dialogue by `exact_f64_bits`, the D1 reader untouched and reported), then a stale fixture hook
  linked from a reused cargo volume after an `aphavoc/`-only edit. Fresh volumes are used for every identity build.
* **Causal and refusals (ordinary-feature shipping build, 300 s):** reference host 185 births / 6 losses (equals the original); every candidate undetectable 176 / 1; scan ranges tripled 208 / 10. A missing provider fails at boot; seven malformed providers stop the engine with `host sense: ...` and the specific reason.
* **Whole campaign:** the shipping candidate's 900 s Lebanon recording, observations and metrics are byte-identical to the retained supplied-binary reference (native exit 0; `observation-check.py` PASS; not visually inspected in Tacview).
* **Checks:** fmt 0; clippy 0 for default, all fixtures, old-guard+sensor and sensor alone (fresh volume); `cargo test` (default features) all pass: 3+2, 30, 165 (nine new sensing contracts), 19 (exact-double transport under both rounding modes), 6 (Lua `host.sense` round trip), 2.
* **Not claimed:** player-controlled targets, the difficulty factor and emitter-off adjustment in a campaign run (the headless world has none; kernel contracts only); altitude above the ceiling; Georgia; other seeds; the final full pack; batching (none was needed or introduced; 316,847 crossings in 300 s showed no visible wall-time penalty, not a benchmark).

### Lead QA before accepting the sensing witness

Root independently compared all 443 events of the completed `d588656e3` witness2 against
the immutable original: physical/decision hashes and every other field agree, but **5
`sensor-tolerance.recompute_mismatch` records differ**, so the comparison still exits 1.
The final run counter is 1079 instead of 0 (exactly 1398 draws minus 319 tolerance rejects).
Evidence: `../d2-evidence/independent-sensor-d588-comparison.txt`. Keep the counter and old
expected values; investigate its bookkeeping rather than dropping it or accepting only hashes.
**Resolved in the fresh third witness:** root independently compared `out/extract-witness3/trace.jsonl`
at `37989fa5f` with `out/lebanon-probe/trace.jsonl`: all 443 events agree, comparator exit 0,
including all five formerly differing counters. Evidence: `../d2-evidence/independent-sensor-379-comparison.txt`.
No further unchanged 600-second rerun is needed. `37989fa5f` itself adds only lint attributes;
the important build difference is the fresh fixture-2 volume. Root read-only inspection found
fixture-1's staged `mb_hostsense.c` still SHA-256 `77fd5ce446b522f8732689c63e811b6c6dfc20b56aa9125e85a25f6efbed6c09`,
while fixture-2 and the corrected source both have `51ca46e22ec1ea6220db32fe2de21429d8977375cd3191897492c90cde47528d`.
The d588 fixture build finished in 5.12 seconds: its source label did not establish recompilation
of the external C input. Investigate/fix incremental invalidation for the newly edited external C
source (build/main.rs watches build/compat/csrc/sources.txt and cc's staged files, not the original
aphavoc tree). Preserve the stale witness/bin as failed evidence. Final acceptance uses verified
freshly compiled C, not a stale incremental artifact. This is lead evidence QA during development,
not the independent delivery source-review pass.

### Independent review of 37989fa5f (the lead's notes as written; the pass is complete and its findings are corrected, see "Review corrections")

Lead source/provenance note: that pass is complete with P1/P2 below (the correction
stage is active). The eventual focused follow-up must also include the small
`cee800898` test-driver delta. That commit is not docs-only: it amends the malformed
draw control and documents wrapper exits. The private bin's sense-provider.lua
was amended after BUILD-INFO was captured (build-time hash be4d916f..., amended
script/source hash5cfade67...). Preserve the original manifest and explicitly
record binary-source379 plus driver amendmentcee; see
`../d2-evidence/sensor-driver-provenance-note.md`. This does not require repeating
the unchanged reference/blocked/far or campaign.lua900s runs.

The same independent Codex reviewer is doing the single source pass. Two findings
confirmed so far (final report still pending):

* Track the original source/header inputs in `eech-engine-sys/build/main.rs`, not
  only its local files and staged `OUT_DIR/tree` copies. Verify a representative
  original-input incremental edit triggers recompilation without cleaning.
* `call_sense` and `mb_hostsense.c` currently require every rejected detection to
  name a scan-specific legacy predicate (range, adjusted range, altitude or
  tolerance). The host needs a generic physical nondetection result that does not
  pretend one of those predicates failed. Preserve existing fixture reasons for
  exact characterization, and keep target selection/engagement in the campaign.

Lead will pass the complete consolidated review back through the same Sonnet
context after actual CLI completion. This is not permission to create another
implementation lane or independent review fleet.

## Composition stage: observer + host-owned LOS (development stacking only)

Merge `1fc0cdc621fc0e643b1b78373c2fa230f8ef20c3` of the reviewed frozen LOS head `a42f1cfad` into `f22269490` (the sensor characterisation and comparator-fix head); then `544debee3`
(rustfmt line breaks only). Ten textual conflicts resolved by keeping both sides (`los-fixture` and `sensor-fixture` features, defines, FFI and engine entries side by side; `eech_engine.c` edited byte-safely).

* **Entry wiring.** The surface-to-air scan asks the host for line of sight, which exists only inside an `ENTER (..., host)` context. `eech_engine_sensor_probe` now takes and requires the complete host
  (five callbacks, as a frame), `Engine::sensor_probe` builds it with `with_host`, the Lua `sensor_probe` hands over all five callbacks, and the sensor driver boots with `line_of_sight = host.line_of_sight`.
  The trace entries run no scan and stay host-less. No fallback; missing or malformed provider semantics are EEDCS-42's, unchanged.
* **Checks** (cargo's own exits): `544debee3`: `cargo fmt --all --check` 0; clippy 0 for default, all fixtures (dependent+loss+los+sensor), old-guard+sensor, sensor alone. A clean fixture build of `1fc0cdc62`
  (0 uncommitted inputs, exit 0): `eech_dc.dll` `917fe0f2...7e5c`. `cargo test` not run.
* **Witness.** One 600 s Lebanon run of that build through the public Lua -> DLL path with the host-owned LOS provider, frame 100 ms, seed 1, probes at 30/300/590, fresh private root (manifest identical before
  and after), native exit 0: the trace file is **byte-identical** to the retained original (SHA-256 `dec1daf0...f753`); the corrected comparator reports 0 differences, exit 0, for run (75 events), probe (182),
  run-at-probe (186) and all scopes (443). Physical `d60c462829c9714e`, decision `2611b04d99f5c689`; the run-wide report was unchanged by each probe. Evidence and limits:
  `eech-campaign/reference/d2-sensor-detections/composition/README.md`.
* Not claimed: LOS accepted, any sensor predicate moved, a new 900 s or no-probe case, the full pack, D2 or EEDCS-43 complete. The `544debee3` binary differs in hash and was not run.
* Next (root's): confirm the complete baseline, then the physical sensor extraction in this lane. For that scope the host owns the physical model and the campaign selection/ranking; the heap-index seed stays private
  (a random draw fact at the original conditional point is allowed, never the seed).

## Original characterization and evidence-tool correction (completed before extraction)

At that original-characterization stage, the four AI scan functions (`ac_tgt.c` air-to-air and air-to-surface, `vh_tgt.c`
surface-to-air and surface-to-surface) retained the original text; the only source additions were
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
