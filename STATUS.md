# Current D3 test-helper correction — EEDCS-50 host model store at `61a80ee1a` (30 October 2026): SOURCE ONLY, UNBUILT, UNRUN

Actual first validation of the four-finding correction frozen at `61a80ee1a`: Linux format 0 and the eech-world LIBRARY tests pass, 235 of 235, including the five new `fs_tests` that drive the actual
resolver and enumerator against a fake filesystem (log `model-host-61a80ee1a-linux-20261030.log`). The integration test `tests/model_store.rs` did NOT compile: eight E0308 errors, all one cause. The test helper
`tri` in `the_legitimate_original_modes_are_accepted_and_compared_as_they_are` took `&Option<String>` while its callers pass `p0` and `p1`, which the lookup closure returns as owned `String`s (lines 1531, 1543,
1549, 1561, 1562). The old eight diagnostic guards and Clippy were therefore NOT RUN, and Windows had the same eight errors (build wrapper 1; no export, no Windows tests, no retail comparison). Neither the
235 library passes nor `61a80ee1a` is integration, Windows or acceptance evidence; the frozen source and the failed logs stay immutable.

Correction (test helper only, UNBUILT, UNRUN): `tri` now takes the nine words as `&str` and builds the `Some(owned string)` the report tuple schema requires, so all eight call sites type-check unchanged; no value, report
collaborator, negative case or assertion was changed, and the other new helpers (`host_group`, the alteration builders) already pass `Some(String)`. No product source (`models.rs`, `model_host.rs`,
`original_report.rs`, C) changed. The Root separately reproduced all four comparator defects against the immutable `2020436ce` host with exact corrupted retained reports (cut off after the first scene, a duplicated
scene, a missing geometry terminal, an original `done` with failures 1: each wrongly native 0 / done 0; healthy control 0); those inputs remain private for the corrected-binary green. Whole D3, the 59 original
custom and mixed geometry gaps and the production physical-loader removal are unchanged; no task, production or Tacview claim; the original Task 47 shipping run is live and untouched.

# Current D3 correction — EEDCS-50 host model store, four review findings (30 October 2026): SOURCE PREPARED, UNBUILT, UNRUN

Review of the tested `2020436ce` (`independent-host-model-component-review-2020436ce-20261030.json`, SHA-256 `4f387a639e452c9942e89d693d4de0b7a2e7a914a34bf6c68dcd38626f48c613`): CHANGES_REQUIRED,
exactly four P2s. This entry is the next source after publication `393539141`; it has not been formatted, compiled or run, and the tested `2020436ce` evidence (Linux format, 25 new and 8 old tests,
all-target Clippy; Windows build, 25 tests and the expected missing-host refusal; the three complete retail host reports, identical at 1,278,273 bytes SHA-256 `34e4b684…3e86`, with all 3025 scenes'
metadata and object numbers and 215 original calls matching and surface block 52,517; inputs, originals and the five host payloads unchanged) is retained as it was and is NOT relabelled as running
these corrections. The initial `0b5750f7a` failures and the skipped-number defect remain part of the history.

**Corrected (details and exact contracts: `eech-campaign/docs/m5-model-host-store.md`, "Corrections for the four review findings"):**
1. MODEL-NODE-PREALLOCATION: a sub-object count is checked against the bytes that remain, at the smallest record the format allows (derived from `initialise_3d_sub_object`: 96 bytes binary, 93 named,
   105 for version -3 and below), before anything is reserved or resized; the node total must fit the kernel's 32-bit counts; the reservation is checked. Traversal, order and layout unchanged.
2. MODEL-OVERRIDE-IO: enumeration and name resolution go through a small `Fs` boundary; only a genuinely absent optional `OBJECTS` is absent, every listing, entry, type, vanished-entry, wrong-kind
   and case-ambiguity failure is an error naming the path. Successful order unchanged. Unit tests drive the actual resolver against a fake filesystem; a real-filesystem test covers `OBJECTS` as a file.
3. MODEL-COMPARISON-COMPLETENESS: `compare_original` validates the report as a whole, coherent run of the closed original driver (new `src/original_report.rs`) before comparing a row: the envelope,
   the complete catalogue when declared, and every geometry group's pages, depths, terminal and call counts. Legitimate absent, refused, unbacked, capped, catalogue-off and partial-write observations pass;
   no physical outcome is required. Test collaborators are complete report envelopes built from the host's run; controls (cut off after the first scene, repeated scene, missing terminal, failed original
   done, and others) run through the real comparator and host-Lua driver and are runnable against the old `2020436ce` host through `EECH_WORLD_BIN`.
4. MODEL-SCENE-WIDTH: the signed scene is compared with the scene count before it is narrowed (`usize::try_from`); wide positive scenes that wrap on a 32-bit target are in the guard.

Changed: `src/models.rs`, `src/model_host.rs`, new `src/original_report.rs`, `src/lib.rs`, `tests/model_store.rs`, `docs/m5-model-host-store.md`, this file. Not changed: the C numeric kernel and floating-point
guard (no finding), the closed original diagnostic driver and fixture, any baseline, tolerance, MSRV or physical rule. No new original-provenance instrumentation in this batch.

**Next (Root):** format and freeze; Linux `model_store` (including the new unit and integration controls), the old `model_characterization` guards and all-target Clippy; Windows `model_store` with `EECH_WORLD_BIN` and
the invalid-override control; the old `2020436ce` host against the new comparator controls as the safe red where applicable (never the old unguarded allocation); the retail comparison again only if the Root judges the
loader change affects it (the physical path is unchanged, the report contract is new, so the three retained concrete reports should be re-checked against the new comparator); then the focused review of these changes.
Still open and unchanged: the 59 original custom and mixed-layout geometry gaps need actual original allocation and backing provenance (the host's counts for them are not an oracle); other safe models have metadata but not
exhaustive triangle proof; the production model loader and native physical service removal, hierarchy, viewpoint, device, muzzle and target-point services, entity collision, environment and host-time services remain required whole-D3
work (not deferred to D5); campaign policy, geography, routes, timers and consequences stay in EECH_DC. The original six Task 47 shipping processes were verified live and untouched at 00:15 UTC; the full 40-check /
eight-artifact acceptance and normal delivery are pending. D4 and D5 have not started. No task acceptance, production or Tacview claim. No Git, build, test or native action was taken in this lane.

# D3 checkpoint — host model component tested; four review corrections open (30 October 2026)

EEDCS-50 / draft PR110: source `2020436ce160ac3762f00f7ee8290f4e6247bce3`
implements host-owned disk loading and model queries independently of the campaign
DLL. Actual Linux formatting, 25 model-store tests, eight retained diagnostic
guards and all-target Clippy pass. The Windows build and all25 model-store tests
pass; the explicit missing-host override fails with native101 as required.

Actual repository runners complete0 on Georgia, Lebanon and Lebanon-repeat.
All3025 scene records/IDs and215 retained original accessor calls per root match,
with no named-ID skips and surface block52517. All three complete host comparison
reports have1278273 bytes and SHA256
34e4b68478a2b998411edc46f5e5ad565e8a2fc304decabdb2cb3b8747b03e86.
Pre/post inputs, original reports and five host build payloads are unchanged.
These are model-component comparisons, not campaign/Tacview shipping runs.
Initial0b JSON/Clippy failures and the separately discovered named-index error
remain retained; no baseline or tolerance changed.

The single independent combined review is CHANGES_REQUIRED, exactly four P2s:
bound node allocation before resizing; propagate override-directory I/O errors;
refuse incomplete/failed/duplicate original comparison reports; check wide scene
IDs before narrowing to usize. Report SHA256:
4f387a639e452c9942e89d693d4de0b7a2e7a914a34bf6c68dcd38626f48c613.
The review independently confirms the concrete complete retail measurements;
it does not approve the reusable comparator or current component for delivery.
Safe report and validation metadata are in PR110's
`eech-campaign/reference/d3-model-characterization/`.

Both Sonnet implementation invocations and their builds are terminal/collected;
no implementation writer is currently live. Next: one finite four-finding
correction batch in the relevant Sonnet5.5 High context, affected validation and
one review of the new changes. Then characterize the59 original custom/mixed
geometry gaps using actual original allocation/backing provenance before new
unsafe accessor calls. Other models have metadata coverage, not exhaustive
triangle coverage. Production model loading/queries remain in EECH_DC.

Task47 shipping115's original six process identities were verified live again
at this checkpoint; it was neither stopped nor restarted. Previously checked
Tacview/observation prefixes match only. Full40/eight complete files/post-inputs,
normal delivery and subsequent production handoffs remain pending. Model
hierarchy/viewpoint/device/muzzle/target points, entity collision, environment,
host-time and required physical solver/loader removal remain D3 work. Campaign
policy, geography, routes, timers and consequences stay in EECH_DC. D3 remains
In Progress; D4/D5 have not started. Jira10864/ledger69 are the last external
records and predate this component checkpoint.

# Current D3 correction — EEDCS-50 host model store, first-validation batch (29 October 2026): SOURCE ONLY, UNBUILT, UNRUN

Published source `0b5750f7a` (frozen by the Root as `model-host-source-0b5750f7a`) is the actual first host-owned model component; this entry corrects it after its first ACTUAL results and does not
claim the component accepted: it has had no independent review, and it is not Task Done, not whole-D3, not production. Authority and scope are unchanged (M5 goal Confluence 11534337, ledger
14548994, Epic EEDCS-21, Task EEDCS-50; D4 and D5 not started; the six-process Task 47 shipping run 115 and its full 40-check / eight-artifact acceptance still pending; the original
characterization closure `ab48e507…` for `366622e0c`/`309b3f050` is valid and unchanged).

**Actual first results at `0b5750f7a` (Root-run, not assumed):** formatting 0; Linux and Windows `model_store` 23 tests, 21 PASS and 2 FAIL (both in `lua/model-host-compare.lua`'s error paths: Lua `%q`
wrote a raw escaped newline, so a multi-line Rust/Lua error was not JSON and the strict reader refused the line); the separate all-target Clippy on the exact frozen source failed with six findings
(`models.rs`: two over-indented doc-list lines, four `chunks_exact` with a constant size). The first retail host-only comparisons completed 0 on all three verified roots: 3357 strict JSON rows and
1,278,301 bytes each, identical across Georgia, Lebanon and the repeat; 3025 scene rows compared with no evaluated scene or triangle mismatch and 215 original calls compared (208 TRUE, 6 end FALSE, 1
absence); no campaign DLL loaded and no engine booted; inputs, the original reports and the five host payloads unchanged. The load summary was 36 binary EEO, 144 EES and 811 named objects, with no
unsupported, unresolved or duplicate file. The host reports four original-REFUSED models as UNMEASURED (its own triangle counts for scenes 188, 1, 1412 and 1165): that is not equivalence evidence, and
all 59 custom and mixed-layout gaps remain required work.

**The core loader defect found by the Root's separate source and identity audit (`model-host-0b5750f7a-first-retail-identity-audit`):** `get_next_object_id` is `return total_number_of_objects++` and the
counter starts at the table's object COUNT N, so the FIRST named custom object is number N and REPLACES the last table object; the loader started at N + 1. Every one of the 61 retail named main and
collision numbers was therefore one too high and the consolidated surface block 52,518 against the original's 52,517. The comparator hid it: it skipped every named object number
(`ids_not_compared` 61). Corrected here from the original source, not to fit the output: the counter starts at N; the consolidation runs from 1 up to and INCLUDING the counter's final value (with no named
object that is the last table object; with M named objects the counter N + M is an unused zero slot that is in the loop and adds nothing); a scene's bounds are still computed when it is read, from the objects as they
then stand, and are not recomputed when a later named object overwrites one (query order as the original). `compare_original` no longer skips or normalises anything: main and collision object numbers are
compared as they are, the original's numbers for named custom objects included (a same-input reference comparison; DCS is not required to use EECH's private indices).

**Other corrections:** `lua/model-host-compare.lua` now writes dynamic strings and keys with real JSON escaping (quote, backslash, every control byte 0 to 31 and 127; UTF-8 kept), keeping the error and
detail text, the strict JSON, the checked writes, the no-overwrite and the non-zero exits; only the new driver changes. The six Clippy findings are fixed idiomatically (list items without over-indented
continuations; `chunks_exact(6)` and `(2)` replaced by index-based element reads, no `as_chunks`, no MSRV change, no `allow`). No C numeric kernel, FP guard, physical rule or tolerance changed.

**Guards added or corrected (written, not run):** the named-scene numbering test now expects the first named object to be number 4 for the four-object table (with source provenance), the second 5, the
overwritten table object read through the table scene that still names it, that scene's bounds NOT recomputed, objects_total 6 and a surface block of 4; a new test keeps the last table object consolidated
when nothing is named (total 4, block 4); the comparator now has main and collision number alterations in its list and a named-number-shift control (host 4/4 against a report that says 5/5 is reported and fails).
The retail 61-number and 52,517/52,518 differences are the RED proof; no expected triangle or number is computed by the new code.

**Next (Root):** format and freeze, rerun Linux `model_store` and the old `model_characterization` guards with all-target Clippy, Windows `model_store` with `EECH_WORLD_BIN` plus the invalid-override control,
then the three retail comparisons (expect all 3025 scene fields, 215 original calls, no skipped number, surface block 52,517), and inspect every difference without adding tolerance, normalisation or skips;
then ONE initial combined independent review of the new component. Later: resolve the 14 custom and 45 mixed-layout gaps through minimal original backing/provenance instrumentation, not by treating the host's new numbers
as the oracle; the production hand-off; and the whole remaining D3 scope (model hierarchy, viewpoint, device, muzzle and target-point services, entity collision, terrain, spatial, environment and host-time services, removal of the
physical solvers and loaders from EECH_DC). Campaign policy, geography, routes, timers and consequences stay in EECH_DC. No Git, build, test or native action was taken in this lane.

# Current D3 component — EEDCS-50 host-owned collision-model store (29 October 2026): SOURCE PREPARED, UNBUILT, UNRUN

Base: branch `codex/d3-model-geometry` at the reviewed original characterization (`366622e0c` fixture and measurements; driver `309b3f050`, independent closure with no open findings),
integration ancestry `6bbaa8197`. Live authority: the M5 goal (Confluence 11534337) and the migration ledger (14548994); Epic EEDCS-21 (D3) and Task EEDCS-50 In Progress. D4 and D5 have not
started. The six-process Task 47 shipping run (115) is still live elsewhere (59 MB Tacview / 41 MB OBS prefixes equal only; no complete 40-check / eight-artifact acceptance); production G2/G3
still wait for validated preceding delivery. This entry makes no claim about any of that, and it is not Task Done, not whole-D3, not production.

**What is new (actual implementation, not another fixture):** the first host-owned model component. `eech_world::models::ModelStore` loads the collision-model data from the retail DISK assets by
the host, independently of the campaign DLL: `bininfo.bin` names (checked against the compiled list), `3dobjs.bin/.pts/.sp` objects, every `3dobjdb.bin` scene record (walked by its own counts),
binary-hex `OBJECTS/????.EEO` overrides and named `OBJECTS/<dir>/*.EES` scenes (versions -1 to -3, per-directory `.EEO` resolution). Private legacy layout; Lua sees only semantic values
(`host.model_store`, `scene`, `triangle`, `summary`, `compare_original`). The original's numeric code (object-bounding-box triangle scaling; 3dsubobj.c scene bounds and radius) is reused in
`csrc/model_step.c` under a round-toward-zero guard that restores the caller's complete x87/SSE state on every path. The decoder's integer walk is the original's, translated exactly, over
bounds-checked arrays: a read is allowed as far as the BACKED array extends (not the object's own counts), so the 45 mixed-layout and 14 custom-object models are decoded where backed and
refused (`unbacked_*`) where not, with the legacy semantics reproduced, not repaired. Missing assets are NOT RUN; truncated or inconsistent files are refusals; no synthetic fallback.
Files and formats: `eech-campaign/docs/m5-model-host-store.md`. Changed: `src/models.rs`, `src/model_host.rs`, `csrc/model_step.c`, `data/scene-names.txt` (derived from `modules/3d/3dmodels.h`),
`build.rs`, `src/lib.rs`, `src/main.rs` (one registration line), `lua/model-host-compare.lua`, `tools/model-host-compare-windows.ps1`, `tests/model_store.rs`, the docs. The existing model driver,
its tests and the fixture are unchanged. No production dispatch moved; the campaign DLL still runs the original loader and queries.

**Original measurements reused (not repeated):** `compare_original` reads the retained original report (3025 scenes per root; 769 safe, 2009 absent, 14 custom, 45 face-mismatch, 188 no-scene; the
selected MI24, KRIVAK, Ka-50, pipe, building and pylon rows) as the oracle. Nothing is derived from the store as an expectation. The 14 custom and 45 mixed-layout scenes have NO original triangles
(the original refused them); the host's answers for them are listed as such, not as matches, and a minimal feature-only original probe over the ranges the store reports backed is the next
characterisation if the Root wants them equivalence-measured. Raw assets and triangle coordinates stay private.

**Guards written (not run):** loader refusals at every truncation, bad counts and magic, scene and object indices, header totals, missing assets, name disagreement; the C1 mapping; overrides
(applied, ignored version, invalid, truncated, ambiguous); named scenes (object numbering, missing object, unsupported version, unresolved name); the mixed-layout backing refusal; arguments and
depths; caller floating-point control and status restored bit for bit and the answer independent of the caller's rounding mode, and an unmasked caller refused; the store is `Send + Sync`
immutable data; the driver and comparator on a fixture installation (match, nine kinds of alteration found and counted, unreadable or empty reports refused, NOT RUN, no overwrite). The
fixture installations are written by the tests in the original's formats; the only numeric expectations are identities (scale exactly 1.0; caller-independence), not a physical oracle.

**Next (Root):** format, build, run the guards (Linux and Windows, with `EECH_WORLD_BIN` for the physical host), fix what the compiler and Clippy find; then the retail comparison with
`tools/model-host-compare-windows.ps1` on fresh verified Georgia, Lebanon and a repeat root against the retained original report, with input post-checks; investigate every mismatch (a
mismatch is a finding, never a tolerance); then one combined independent review. After that: the custom and mixed-layout original probe, the production hand-off of the model data to the object and
tree queries, and the remaining whole-D3 scope: model hierarchy, viewpoint, device, muzzle and target-point services, entity collision, the terrain, spatial, environment and host-time services,
and the removal of the physical solvers and loaders from EECH_DC. Campaign policy, geography, routes, timers and consequences stay in EECH_DC. No Git, build, test or native action was taken in
this lane.

# D3 checkpoint — original retail model characterization reviewed; host loader next

Task50 original-model preparation is independently closed with no open findings.
Original native measurements remain366622e0c; the corrected diagnostic driver and
portable guards are reviewed at309b3f050. Closure report SHA256:
ab48e5079bcc07f119236d31555f70c082f733ae5a979956071bd6d6b8a62bce.
Linux format/all8 guard groups/all-target Clippy pass. Windows host/test build0,
all8 guard groups pass; an explicit invalid-host override is correctly refused.
All3 new-driver native catalogue/selected-geometry runs complete0 and preserve
exact original catalogue and geometry row bytes, including line endings. State,
clock, input and original43 payload checks remain unchanged. Native366 is not
relabeled as a new DLL build. C441 Clippy and Windows harness failures, and Root's
aggregate log-label error with reviewed read-only recovery, remain retained.

No physical responsibility has moved in this preparation. Collision-model
loading/queries remain in the DLL. Next is the actual host-owned collision model
loader/query component within EEDCS-50, preserving original fan/depth/scaling and
required binary/EEO/EES input semantics. The active custom/mixed-layout gaps must
be investigated; no always-absent substitute is accepted. Model hierarchy,
viewpoint/device/muzzle/target-point, entity collision, environment, host-time and
required physical solver/loader removal remain whole-D3 work. Campaign decisions,
consequences, geography, routes and timers stay in EECH_DC. Task47 original
shipping115 continues; full40/eight complete files/post-input and normal delivery
remain pending. D3 stays In Progress; D4/D5 have not started.

# Current D3 test-harness fix — EEDCS-50 model driver guards (29 October 2026): SOURCE ONLY, UNBUILT, UNRUN

Driver source `c44173556` (frozen) passed the Linux guards (8/8), formatting and the Root's actual-native checks of the corrected driver with the original `366622e0c` DLL; Linux all-target Clippy
failed on the unused `Run.stdout`, and the Windows guards failed before launching the host (OS error 3) because the test used the container-baked Cargo binary and manifest paths. That is a test
set-up failure, not a production red, and no expectation was relaxed. This change touches only `crates/eech-world/tests/model_characterization.rs` (and a factual doc note): an explicit
`EECH_WORLD_BIN` selects the host (a missing or non-file value is an error, never a fallback; unset uses Cargo's binary; the environment is only read), the reviewed driver is compiled into the
test with `include_str!` and written unchanged into each run's temporary directory, and `Run.stdout` now appears in the failure diagnostics. Guard cases, stand-in protocol, driver, C, Rust
product, FFI and API are unchanged; no new DLL. Not formatted, built or run in this lane. Next (Root): Linux 8/Clippy, Windows 8 with the physical same-source host plus the invalid-override
control, then the focused changed-source review of the driver closure. Whole-D3 scope, the custom / mixed-layout gaps, production coupling and the live Task 47 run 115 are unchanged; the host
loader/geometry move follows review.

# Current D3 correction — EEDCS-50 diagnostic-driver response validation (29 October 2026): SOURCE ONLY, UNBUILT, UNRUN

Measured (Root, fixture `366622e0c08b2caaa5a5317f5197c0ee3d7050e9`, original DLL): three retail catalogues of 3,025 scenes each and eleven selected scenes per root run through the
real host Lua → public `eech_dc` → original accessors, with the first-measurement results in the checkpoint below and in `eech-campaign/reference/d3-model-characterization/`. The sole
independent reviewer confirmed one P2 on the diagnostic Lua driver and no other defect in the C, FFI or observer on the verified loaded-table contract. That P2 was reproduced against the actual
original DLL by corrupting only the report text (a `safe` header with an invented zero-call `original_false` end; a count-correct catalogue page with a repeated scene id): both were accepted with
`done failures=0`.

This correction (new, not measured, not reviewed): `lua/model-characterization.lua` now validates every reply as a report (shape, identity, order, cardinality, completion; never geometry
mathematics) and refuses, with a `report-refused` row and an error before any `done`, what it cannot support; genuine absent / refused / unbacked / capped / partial-write outcomes are kept as
what they are. New portable guards `crates/eech-world/tests/model_characterization.rs` run the real driver under the real host Lua against a response stand-in (healthy multi-page, absent,
refused, empty, unbacked, capped and anomaly cases accepted; the two measured controls and about 60 other malformed replies refused; argument / synthetic-database / existing-output refusals
before boot). They are response-contract tests, not physics and not loader proof. Changed in this correction: the driver, the new test file, `docs/m5-model-characterization.md` and this record.
No C, Rust product, FFI, feature API, parser or host-loader source changed; `eech_engine.c` (one NUL) is untouched. Nothing was compiled, linted or run in this lane, and the new Lua and Rust
are checked only by reading.

Source identity: the driver and test are not native build inputs, so the `366622e0c` DLL is reusable only if the Root verifies the exact C/Rust native-source identity first; it must not be relabelled
as a new build. Next (Root): Linux and Windows guards; the same two corruption controls red → green against the corrected driver; the three retail measurements rerun and compared with the first;
a focused review of the changed source. No independent closure of this correction exists yet. Original Task 47 shipping run 115 remains live and unaccepted. Remaining whole-D3 requirements are
unchanged: custom-object and mixed-layout original characterisation (14 custom and 45 layout-mismatch scenes were refused, not decoded), the host-owned read-only model data and loader/geometry
move (next, not dropped), model hierarchy / viewpoint / device / muzzle / target-point services, entity collision, environment and host-time services, and removal of the physical solvers and
loaders from EECH_DC. Campaign policy, geography, routes, timers and consequences stay in EECH_DC. D4 and D5 have not started. The earlier source-preparation paragraphs below are historical.

# D3 checkpoint — model fixture built; three retail catalogues measured

EEDCS-50 / draft PR110, measured source `366622e0c08b2caaa5a5317f5197c0ee3d7050e9`.
The Windows model-fixture build completed with actual exit 0 and 43 verified
exports. The repository PowerShell runner completed with actual exit 0 on
Georgia, Lebanon and Lebanon-repeat; all three pre/post input checks pass,
all payloads remain unchanged, and each before/after native state digest matches.
Each catalogue enumerates 3,025 scenes: 769 safe-to-probe, 2,009 absent,
14 custom-object refusals, 45 face-layout mismatches and 188 missing scenes.
These are catalogue results, not measured retail triangle equivalence. The full
raw catalogue hashes differ and are retained; no cross-process byte equality
is claimed. Safe metadata and exact build identity are recorded in
`eech-campaign/reference/d3-model-characterization/`.

The sole Sonnet High invocation completed with exit 0; no model implementation
writer is running. Next: select campaign-used safe models with backed end reads,
measure original triangles and refusal/absence/end behavior, then independent
review before host loader extraction. No host loader or production dispatch has
moved. Task50 and D3 remain In Progress; D4/D5 have not started. The original
Task47 shipping processes remain running, untouched by this commit/push request;
full-pack acceptance remains pending.

# Current D3 checkpoint — EEDCS-50 (29 October 2026): original collision-model probe PREPARED, UNBUILT, UNRUN

Live authority: the M5 goal (Confluence 11534337) and the migration ledger (Confluence 14548994), read by the Root; Epic [EEDCS-21](https://flying-dice.atlassian.net/browse/EEDCS-21)
(D3) In Progress; Task [EEDCS-50](https://flying-dice.atlassian.net/browse/EEDCS-50) "M5 D3: Host collision-model loading and geometry" In Progress. Branch `codex/d3-model-geometry`,
base accepted integration `6bbaa8197a4843181d03cd355d562af998c75321`. The Task 49 object/tree-query files and features are NOT in this base. D4 and D5 have not started.
This is preparation: it is not production, not the Task's Done, and not D3 acceptance. The six-process Task 47 ordinary shipping run (115) is still live elsewhere; this
worktree makes no claim about it (its latest Tacview/OBS prefixes are exact prefixes only; no complete 40-check / eight-artifact acceptance exists).

Scope of D3 (unchanged): the physical terrain, spatial, model and environment services and the native loaders and solvers they need leave EECH_DC. Campaign geography, routes, timers,
scheduling, selection, decisions and consequences stay native in EECH_DC. EECH_WORLD may reuse the original C physics; no language rewrite, renderer, DCS clone or second campaign.
Required DCS calls, or named bounded data workarounds, and the model loader / mapping definitions are D3, not deferred to D5 or M6 (only live DCS execution is M6).

Evidence reused, not redone: Task 49 forest-geometry witness (original native `47a4af0d1`, stricter driver `b44d2c96f`; one combined review and one focused correction, closure
`4a97eb54…` CLOSED, no open findings). Its six triangles came from the real original accessor on a controlled, scoped-down mesh: a decoder witness, **not** proof of a disk loader
or of any retail model. Its raw reports and the source blobs are retained and untouched.

This step (see [`eech-campaign/docs/m5-model-characterization.md`](eech-campaign/docs/m5-model-characterization.md)): a test-only, feature-gated (`model-fixture`) probe that asks the
ORIGINAL compiled model accessors a finite catalogue and selected geometry queries through host Lua → public `eech_dc` → original C, plus a read-only safety verdict (the decoder reads
one surface past the object on its last step, trusts every index and crashes on a NULL surface list, so it is called only where the loaded tables show the memory backed) and a
before/after table-state digest. It implements no loader, parser or geometry algorithm and removes no production dispatch. Changed: `eech_model_fixture.c` (new), `eech_engine.{c,h}`
(gated tail / declarations; the existing bytes and the file's single NUL are an unchanged prefix), the `eech-engine-sys`, `eech-engine` and `eech-dc` feature chains and Lua methods,
`lua/model-characterization.lua`, `tools/build-windows-model-fixture.sh`, `tools/model-characterization-windows.ps1`, and this record. Nothing was compiled, linted, run or
measured in this lane (no toolchain was invoked); the C and Rust are unreviewed by a compiler.

Next unmet measurements (Root): clean fixture build with BUILD-INFO / payload / flag checks; the catalogue on fresh verified Georgia and Lebanon roots (retail database required;
a synthetic one is refused); selection of real campaign-used vehicle and building models from the catalogue; full original triangle runs for them, repeated; then the host-owned
read-only model data and loader, reusing the same scenarios. Remaining D3 gates: model hierarchy, viewpoint, device, muzzle and target-point services, and all terrain / spatial /
environment / host-time services; the whole-engine DLL coupling is still present. The older checkpoints below describe D2 and are unchanged.

# D2 combat and perception — implementation delivery record

29 October 2026. The physical D2 increment is delivered on integration through
PR105, merge a44e2b3483aab5f5b51aad47dd67e3ece5874da9. Its fetched merge tree
exactly matches reviewed candidate 6220bd5d9. The sole independent combined
final technical review found no blocking findings, and all nine CI checks
passed at that candidate. Exact review, CI, merge and cleanup records:
eech-campaign/reference/d2-combat-characterization/final-delivery/.

The original shipping pack exited 0: all 40 accepted checks pass, all eight
complete Tacview/observation/metric artifacts remain byte identical, and three
external plus one internal post-input checks pass. The 42-entry build manifest
and BUILD-INFO are unchanged. Runtime source remains
64cf0f1c5551a2e59309a92917c32e6189d8ae60; metadata commits never relabel it.
Full raw evidence and retail data remain private and preserved. The completed
named Windows build cache was removed after export and zero-reference checks.

The whole-D2 responsibility and coverage matrix is
docs/handover/D2-SCOPE-AUDIT.md. Campaign decisions and consequences stay in
EECH_DC. Terrain/model services, entity mirrors/heap, shared RNG, effects/sounds
and later-epic repair/logistics coupling retain their explicit dispositions.
No active D2 solver is assigned to D5. D3 has not started.

The authoritative acceptance and live work-item statuses are
[EEDCS-46](https://flying-dice.atlassian.net/browse/EEDCS-46),
[D2/EEDCS-18](https://flying-dice.atlassian.net/browse/EEDCS-18) and the
[migration ledger](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/14548994).
This implementation-delivery record supersedes the older at-time checkpoints
below without removing their failed experiments, provenance or limitations.

# Current D2 lead checkpoint — final native validation complete

The original shipping regression completed on 28 October 2026 at 23:53:56 UTC.
D2/EEDCS-18 and EEDCS-46/PR105 remain delivery candidates, pending final
independent review, checks on the publication commit, normal merge and
Jira/Confluence acceptance. D3 has not started.

The original controller and runner exited 0. The exact accepted set of 40 checks
passed, and all eight complete Tacview, observation and metric artifacts match
the completed destination baseline byte for byte. All three external post-input
checks and the additional internal Lebanon copy passed. The 42-entry shipping
file manifest and BUILD-INFO identity are unchanged. Baselines and tolerances
were preserved. Exact reports and hashes are in:
eech-campaign/reference/d2-combat-characterization/final-shipping-fullpack/.
Complete recordings, retail roots and binaries remain private.

The actual runtime build remains 64cf0f1c5551a2e59309a92917c32e6189d8ae60.
All nine CI checks passed at daf3960d8. Seven coarse source frontiers match the
runtime source; crates differs only in the integration test host_callbacks.rs.
Production compilation inputs are unchanged. The build is never relabelled as
a later metadata commit. The final 9000-frame cost control and its reviews are
closed, with their loaded elapsed-time and DCS-budget limits retained.

The finite D2 matrix is in docs/handover/D2-SCOPE-AUDIT.md. Prior source, causal
host/report and component runtime reviews retain their declared coverage.
Campaign decisions and consequences remain in EECH_DC; terrain/model services,
entity mirrors/heap, RNG, effects and sounds remain explicit coupling.
Next: final evidence/publication/composition review, current-head CI, normal
PR105 merge and fetched merge-tree verification; then Task46 Done, whole-epic
acceptance, the final ledger append preserving all five reports, and completed
disposable-cache cleanup. Older checkpoints below retain their at-time state
and failed experiments and are superseded by this entry.

# Current D2 lead checkpoint — 28 October 2026, 16:53 UTC

D2/EEDCS-18 and Task46/PR105 remain ACTIVE/UNACCEPTED; noD3. Final ordinary
shipping64cf frame-cost/900s control is independently CLOSEDa3a2ee76: original
8557/native17720 actual0 (completed16:48:18UTC), all9000rawsamples/15windows
recomputed, mean260.2627778ms/max564/p50254/p95370/p99429, all3COMPLETE
Tacview/observation/metric files byte-identical, post1179inputs/all42exports and
frozen source unchanged. Exact safe proofs and timing-only raw samples:
`eech-campaign/reference/d2-combat-characterization/final-shipping-frame-cost/`.
This is loaded, quantized elapsed engine:frame; no causal parent comparison or
DCSbudget pass. Scope/follow-up is retained with the evidence.

All9CI passed at aa19, including actualWindows24host_callbacks confirmation;
source/scope/metadata reviews are closed. Seven coarse frontiers match actual
runtime64cf, with only the integration-test target differing in crates; production
inputs are unchanged and the build is never relabelled. This new publication
needs passing current-head checks before delivery. Full originalpack38810 /
runner23828 (created16:06:29UTC) andfourcampaigns remain ACTIVE. Complete40/full8/
post3, finalcompletepack review, normalmerge46 and wholeEpic18acceptance remain
gates. Campaign decisions/consequences staycore; declared terrain/model/mirror/
RNG/effects/sounds coupling remains. Historical paragraphs below retain their
at-the-time scope and are superseded by this checkpoint.

# Current D2 lead checkpoint — 28 October 2026, 16:23 UTC

D2/EEDCS-18 and Task46/PR105 remain ACTIVE/UNACCEPTED; noD3. Provisional finite
D2 source-scope reconciliation is independently closed without a new active D2
responsibility outstanding: `docs/handover/D2-SCOPE-AUDIT.md`, exact reviewed
draft8906a8c4. The dormant EECH cockpit/HUD ballistic-avionics path is explicitly
outside the supported campaign-DLL runtime; no active physical solver is deferred
toD5. Provisional source review is not whole-epic acceptance.

Current test-only63ea6cfa4 fixes two Windows-CI LF-only assertions using exact
logical lines; raw stdout/trace/readers/baselines are unchanged. Local24callbacks
andfmt pass, independent492732ef closed; new WindowsCI confirmation remains open.
The frozen ordinary shipping build used by finalpack38810/PID23828 and raw-cost
8557/native17720 stays64cf. Both runs remain ACTIVE. Production inputs are unchanged;
seven coarse frontiers match, with the crates tree differing only in the integration
test file. Do not claim all eight coarse trees equal after63ea or relabel the build.
The prior six native report cases remain closed; complete40/full8/post3, finalcost,
currentCI/finalreview/normalmerge and Task46/wholeD2 acceptance remain gates.
Scope/equivalence/reviews: `eech-campaign/reference/d2-combat-characterization/final-scope-progress/`.
Historical at-the-time paragraphs below retain their original limits.

# Current author note — 28 October 2026: TEST-ONLY Windows-CI portability correction (the Root's lead checkpoint below is otherwise current)

GitHub run 40723326838 (job 122939836511, raw log kept at `R:/d2-evidence/combat-characterization-20261028/ci-092-windows-cross-job-122939836511-attempt2.log`) was red at `09253829b` for two
`eech-world` test assertions only (`host_callbacks` 22 passed, 2 failed); the product build and the other suites passed. Both failures printed the semantically correct rows with Windows CRLF line ends:
the bridge witness's `CALLS 0:0:-1:0` line (the case correctly said `BRIDGE_CASE_NOT_DONE` and acted not at all) and the object witness's last trace row `structures` (after the report and the frames). The
correction is in the test target `crates/eech-world/tests/host_callbacks.rs` ONLY: the first now requires a whole logical line equal to `CALLS 0:0:-1:0`, the second that the last logical line of the trace equal
`structures` (Rust `lines()`, which accepts LF and CRLF); the raw stdout and trace are not normalised, no reader or product code changed, no ordering assertion was weakened and no Windows test was skipped. Local
check (Linux toolchain): `cargo test --locked -p eech-world --test host_callbacks` 24 passed, `cargo fmt --check` exit 0 (`checks/cargo-test-ci-crlf-host-callbacks.log`, `checks/fmt-ci-crlf-check.log`). That is NOT
Windows coverage: the remote Windows CI of the pushed commit is the confirmation of the CRLF cases. Production build inputs (all C, Rust, Lua and shipping scripts) are unchanged from `64cf0f1c5`, and the shipping
BUILD-INFO of the Root's active runs remains `64cf0f1c5`; but this test file lies in the `crates` tree, so the eight coarse source frontiers (equal through `09253829b`) are NOT claimed equal after it: the one
exception is this test-only file. The Root's active full pack and final raw-frame-cost runs on the immutable clean `64cf0f1c5` are untouched and unaccepted. D2/EEDCS-18 and Task46/PR105 remain ACTIVE and UNACCEPTED, no D3.

# Current D2 lead checkpoint — 28 October 2026, 16:10 UTC

D2/EEDCS-18 remains ACTIVE; Task46/PR105 is UNACCEPTED; noD3. The later
at-the-time author/history paragraphs below are superseded by this checkpoint.
Physical combat/R3 runtime64cf0f1c5 has a clean ordinary Windows shipping build
(original build/outer0, all42 exports). Test-only candidatea89a668ae has identical
eight runtime/build/scenario frontiers; the build is never relabelled. The same
reviewer closed production capacity/guard-order and harness sentinel corrections.
Six native report cases now complete originaldriver/native0: actual site health
2000→1000 under damage, destruction changes actualalive, withholding preserves
state, own-issued identity works, and three primary invalid-report guards refuse.
Segment destruction reaches real bridge handlers (alive segments1312→1310).
All6post1179 manifests/all42 exports match. Runtimeb3f34540 and corrected evidence
auditor6f2acae1 are independently closed. Exact safe evidence and declared failures:
`eech-campaign/reference/d2-combat-characterization/r3-corrected-runtime/`.

Final ordinary shipping pack38810/PID23828 creation16:06:29UTC is RUNNING on
immutable64cf after actual preflight0/40expected checks; allfour native campaigns
started16:08:29–32UTC. Complete40/full8/post3 recording equality is NOT passed.
Final raw frame-cost probe8557/native17720 creation16:08:47UTC is also RUNNING,
with fresh1179 inputs and actual shipping Lua DLL calibration reused only by exact
hash equality. No final cost/900s equality claim yet. CurrentCI/finalreview/normal
merge46 and whole-D2 reconciliation/closure remain. Campaign decisions and
consequences staycore; terrain/model, entity mirrors, shared RNG/effects/sounds
remain declared coupling. No accepted baseline or tolerance changed.

# Current author checkpoint — 28 October 2026 (the lines below are historical unless they say otherwise)

The newest SOURCE candidate is the shipping source of `64cf0f1c5551a2e59309a92917c32e6189d8ae60` (product C, Rust and Lua, shipping scripts, runtime quantities and all eight build/scenario source frontiers are identical in the commit that carries this paragraph, which changes only a TEST harness: the focused review closed the guard-order P2 and found one test-only P3, that the fake heap's harmless slot for a forbidden index was subtracted from `fake_entities` before being rejected, which is undefined C on the old-red report path; `object_checks.c` now recognises that slot explicitly before any pointer subtraction). It orders the structure registry's guards so that the entity heap, the identity map and the
entity's slot are validated BEFORE any entity read, cached identity, report resolution or host question (the focused review of `30fbdfcbc` found `campaign_object_host_id`, `campaign_object_identify` and the report
resolution reading or returning first), and corrects two sentences of the previous correction's evidence wording. Source checks only: the real registry and driver against a fake heap allocated exactly the run's capacity with
instrumented entity access (94 checks, ASan/UBSan, 26 mutants caught; against `30fbdfcbc` the 12 guard-order checks fail), the real campaign tree compiled, workspace and fixture `cargo test`, fmt and syntax (the C-harness figures were rerun with the corrected harness under NEW logs `object-checks-14-dummy-safe-harness.log` 94/0 green under ASan/UBSan, `object-checks-15-red-pre-guard-order-30fbdfcbc-dummy-safe-harness.log` 94 checks with 12 failed assertions against the `30fbdfcbc` registry, compiled and run, no sanitizer or setup failure, and `object-mutation-9-dummy-safe-harness.log` 26 caught, both setup controls as expected; the workspace, fixture, fmt and syntax runs from `64cf0f1c5` are reused, together with earlier unchanged-Rust Clippy evidence (Clippy was not repeated at `64cf0f1c5`), because no product or Rust code changed since; the earlier logs are kept, and the earlier harness's guarded-access instrumentation, like this one, counts forbidden accesses and does not let a sanitizer observe an out-of-bounds read). NO Windows build, snapshot
or native run of this source; the six `f4700bbab` report roots crashed (0xC0000005, boot row only) and no report or guard has run natively at any SHA; the older-build compatibility runs do not show that the report path works.
D2/EEDCS-18 and Task46/PR105 remain ACTIVE and UNACCEPTED, with no D3; the Root's fresh six-case rerun, final comparison, shipping gates and the whole-D2 audit are all still required.

# Current D2 lead checkpoint — 28 October 2026, 14:55 UTC

D2/EEDCS-18 ACTIVE; Task46/PR105 UNACCEPTED; noD3. Sole Sonnet correction24899/PID7704 actually exited0 at14:44:52UTC, sourcef4700bbab0e28c367c660b8c3bc353c89ad0dac9 clean/pushed. Same independent reviewer CLOSED c398ba49... allP1reverse-IDcollision/P2actual-subjectcorrelation/P3borrowlifetime corrections, no newfindings. Exact initial/focused source reviews: reference/d2-combat-characterization/r3-source-reviews. Actual production binding calls namespace acceptance before commit; IDs for real DAMAGE/HEALTH/IMPACT subjects cross C/Rust/Lua, no numerical/RNG/policy change. Red52/5 vs finalgreen56 is scoped extracted acceptance, not a whole-roster native run; subjectmutants have3passing baseline harnesses, not3negative setupcontrols.

Fresh isolated source R:/d2-evidence/combat-reports-corrected-source-f4700bbab is detachedclean/noalternates and matches all8 sourcefrontiers. Prebuild proof root-clean-r3-corrected-source-prebuild-f4700bbab.json predates any build. Windows fixture78030/controller22108 creation14:52:48UTC is ACTIVE on fresh namedtarget; compile/manifest/sourcepostcheck remain pending. Ordinaryshipping build is NOT STARTED. No R3 native run has started. Do not restart a quiet controller; check actual process/creation/command and record. The author checkout is separate from immutable build/runtime source.

Six fresh1179 report/withhold/guard roots, natural600+matchedbridgepair3roots, finalshipping Georgia1174/Lebanon1179/repeat1179 roots and finalcost1179 root are verified UNRUN. Newcompatibility drivers differ only provenance labels and are independently reviewed. Corrected636 prior900 full3bytes remain CLOSED/unchanged and published; never relabel them f470. Final40/full8/post3/finalnativecost/currentCI/independentfinalreview/normalmerge/Jira46Done/wholeEpic18auditDone still gate completion. Campaign decisions/credit/awareness/accounting/route/repair policy/D1 consequences staycore; data/mirrors/effects/sound/sharedRNG remain declared coupling. Ledger47 publishedONCE/all5reports+oldHTML preserved; never replay. Actual current state/next gates: R:/d2-evidence/D2-LIVE-CHECKPOINT.json. Earlier entries below are historical.

---

# Current D2 lead checkpoint — 28 October 2026, 14:12 UTC

D2/EEDCS-18 ACTIVE; Task46/PR105 UNACCEPTED; noD3. Sole Sonnet5.5High65623/PID24340 actually exited0 at14:11:14UTC. R3 source0ca067d1ef94e51264a4f4a9da40e38c8a14e10a is clean/pushed: host-issued per-incarnation structure identities, window identity correlation and public physical report_object/structures path. Default+fixture Cargo/Clippy/fmt, layout, C guards/mutants/differentials/header checks pass. NEW combined independent source/probe review is NEXT. No R3 Windows build or native run yet; source checks and receipt assertions do not establish campaign-state effects.

Corrected636 R5 default900 is independently CLOSED6a8a15ff: original39137/native24704 actually0, all3 COMPLETE Tacview/observation/metric files byte identical; post1179/all41/sourceclean. Six exact safe copies: reference/d2-combat-characterization/bridge-636-control900. This supersedes earlier pending labels without changing original records. It is not R3, final shipping or wholeD2 evidence. Ledger47 was publishedONCE/read back with all5 reports and allpreviousHTML preserved; never replay.

Six fresh1179 report/withhold/guard roots and finalshipping Georgia1174/Lebanon1179/repeat1179 roots are verified and UNRUN. Root private native wrapper parses0; review precedes a fresh clean shipping build and actual report-state/guard/default comparisons. Final ordinaryshipping40/full8/post3, finalnativeframecost/currentCI/independentfinalreview/normalmerge/Jira46Done/wholeEpic18auditDone remain. New raw-sample timing helpers independently CLOSED803f6eb7 (setup-only, no final native result). Campaign decisions/credit/awareness/accounting/route/repair policy and D1 consequences stay core. Current process/evidence/next gate: R:/d2-evidence/D2-LIVE-CHECKPOINT.json; whole-D2 audit draft explicitly incomplete. Earlier entries below retain at-the-time history.

---

# Current D2 lead checkpoint — 28 October 2026, 13:00 UTC

D2/EEDCS-18 remains ACTIVE; Task46/PR105 UNACCEPTED, noD3. R5 corrected source636ef798a is clean/pushed, Sonnet10691 actually0/PID23596gone. P1 repr(C)/all19inner offsets and P2 destructive-target continuation are independently CLOSED(cabde367...); RootP3 scopedCR metadata preserves raw copies. The rejected b268 build was NEVER RUN; its disposablecache was removed after zero references, all41exports/source/evidence retained.

Corrected clean Windows fixture92864 actually0/dirty0/all41(BINF9fc034cc...). Both default bridge runs31083/17264/native0 match all320598 originalbb bytes exactly. Own physical model leaves unsupported neighbour1 alive; own-fall lands it atframe21 vsdefault5. Nil/variant99 providers trigger specificprimary native1 refusals. Allsixpost1179/all41/source identities match. Independent focused runtime review319c633f... CLOSED/no findings. Exact safe source/correction/runtime proofs and executed helpers: reference/d2-combat-characterization/bridge-636-runtime. This is direct structural failure evidence, not weapon-hit/nativeREST/occupiedvehicle/player/MP/client/wholeD2 coverage.

Separate frozen636 default900 control ACTIVE original39137/native24704 creation12:53:50UTC; source/bin/root/out immutable, actual completion/full3byte comparison/postinput still pending. Do not restart on a quiet console. NEXT coding responsibility is R3 persistent structure ownership/report correlation AND physical factual reports under existingTask46, preserving campaign decisions/accounting/credit/awareness/route/repair policy/D1 consequences core. Do not defer D2 responsibilities to D3/D5 or change accepted baselines/tolerances/scenarios. Earlier accepted damage/impact/timed/bridge behavior is reusable characterization, not a reason to repeat healthy original runs.

Final ordinaryshipping40/full8/post3/currentCI/independent final review/normalmerge/wholeD2 audit and finalshipping native framecost still required. Accepted Task45-parent cost is independently closed49a254.../complete3bytes unchanged; it is not a DCS budget or causal benchmark. Ledger45 publishedONCE/all5reports+oldHTML preserved; never replay. Current process identities and next gates: R:/d2-evidence/D2-LIVE-CHECKPOINT.json. Earlier paragraphs below are historical at-the-time checkpoints.

---

# Current D2 lead checkpoint — 28 October 2026, 12:15 UTC

D2/EEDCS-18 remains ACTIVE. Task46/PR105 is UNACCEPTED; no D3. Accepted integration3b91ca347 includes delivered LOS42, sensing43, aiming44 and weapon45. No authentication change or credential copy; logged-in author and autonomous normal merges are human-directed, without fabricated self-approval.

Impact/blast e0 source and ALL focused runtime groups are independently CLOSED: default600 complete984994bytes, four focused traces, default900 complete Tacview/observation/metric files, and all16 provider profiles (six healthy/five active altered models and ten primary refusals). Inputs/all40/BINF/source identities match; full original legacyf41raw equality remains FALSE for the124 declared counterquote removals. ORIGINAL bridge bb pair is CLOSED:320598raw bytes identical/direct structural failure, not weaponhit; allpost2/all41/sourceclean. Exact safe copies and prior/closing reviews: reference/d2-combat-characterization/impact-bridge-original-runtime.

R5 candidate f5a360199 is committed/pushed, sole Sonnet20280 actual0/PID18760 gone. Host owns collapse/shape/neighbour/fall/vehicle selection and unpack rest-placement. Source checks pass (57production guards,84036differential,12+12mutants with setup controls, Cargo/default+fixture/Clippy/fmt/header checks); source review, clean Windows build and native default/authority/refusal validation are NEXT, not passed. Six focused roots and one900s control root are fresh verified1179 inputs. Root private probe helpers are syntax-checked only. R3 persistent structure ownership/report correlation and physical factual reports remain NEXT D2; do not defer them to D3/D5. Campaign route accounting/repair-task policy and consequences remain core; terrain/model data, mirrors, effects/sound/sharedRNG remain declared coupling.

NEW Task45-parent R4 cost measurement is independently CLOSED: original7184/native0,9000frames/15windows; complete3artifacts byteidentical/post1179/all35/sourcea137clean. Mean207.120222ms/max554/p50194/p95334/p99424 are elapsed native engine:frame-only statistics under recorded background load; quantiles are source-algorithm evidence (RAMsamples not exported), not a DCS budget or matched/causal benchmark. Exact safe reports/executed helpers: reference/d2-combat-characterization/task45-parent-frame-cost. Final ordinary shipping D2 cost measurement still required.

Ledger14548994 v45 was published ONCE/read back: alloldv44HTML exactprefix/all5epic reports unchanged. Never replay v45/v44 or prior writes; fresh full published+draft reads gate the next update. Private snapshot R:/d2-evidence/ledger-combat-progress-v45-20261028.json. Final46/wholeD2 still requires bridge validation, R3/reports, final ordinaryshipping40/full8/post3/currentCI/independentreview/normalmerge and whole-epic audit. No baseline/tolerance/scenario change. R:/d2-evidence/D2-LIVE-CHECKPOINT.json records actual current process identities and next gates.

All frozen source/bin/roots/raws/failures retained. Only completed, independently closed Cargo target caches (including e0impact and bbOriginalBridge) were removed after zero container references; exported binaries were reverified. Keep active author/shared Cargo, WSLdaemon37778 and R: mapping. Older paragraphs below are historical at-the-time checkpoints.

---

# D2 — EEDCS-46 combat resolution and physical reports

Lead checkpoint, 28 October 09:56 UTC: Task45 is DELIVERED by normal PR104 merge3b91ca34730a88e944700af1d0b7d777e071c73e, fetched tree equal reviewedc6b85318d/all8CI. Original runner+launcher0, exact40PASS/full8complete byte identity/post3/all35 unchanged; independent final evidence closed. Jira45Done/comment10813. Ledger44 published ONCE09:49:01.337UTC and read back with all5report wrappers/allold43HTML byte exact. Never replay its publication. Accepted45 is composed into Task46 by metadata merge9236f1bab; all8 runtime/build/scenario trees equal premerge9318e0465. Safe delivery note: docs/handover/D2-WEAPON-DELIVERY-REVIEW.md.

Task46 remains UNACCEPTED. SoleSonnet5.5High correction7651/PID9828 ACTUAL0 at09:53:46UTC, clean/pushed9318e0465 before lead metadata. It corrects all3P2+groupedP3 findings from0400af: host-owned sector order with independent safety/progress guards; changed-radius primary-refusal probe with consistent echo; mutation build/timeout classification; explicit next-D2/R3 structure/report scope and numeric wording. Production blast60/forecast25/damage66/health59,6000worlds/255003events differential, mutations,fmt,Cargo,default+fixtureClippy/header checks pass with retained actual exits. SAME independent correction review and native impact/blast validation still gate; no native impact candidate acceptance yet. Original bridge characterization must be added and run BEFORE its physics moves. Persistent structure identity/report correlation, physical factual reports and whole-D2 audit remain; repair quantities stayD4, general terrain/model servicesD3. Do not defer physical D2 responsibilities on that basis.

All timed evidence is independently CLOSED: frozen366natural600 all984994forecast891/real225bytes/6440events/12trusted/27auditPASS;366default900 all3complete retained artifacts exact; corrected511controlled all403523original35bytes/39cases10done29validrefused/79trusted/74events, prior366scope failure retained. AA authority reference201samples41corehits versus never0/0 andrate150201/55; actual second-draw/fatal-consistency guards refuse. Generic PERSON native/controller0 under explicitly combined ownDAMAGE+stochasticTIMED leaves3->2->1 then4samples/1hit/1fatal1->0,79trusted/75events; not default or timed-only. The unchanged180queue failed on an UNTRIGGERED second-draw final assertion:10completed cases are5healthyuntriggered,4specificrefusals,1untriggeredassertion, fatal180unrun/separatelyAAguardcovered. Noall11pass/unsafeacceptance claim. All used post-inputs/all40BINF/source identities match. Safe timed publication: reference/d2-combat-characterization/timed-366-511-runtime,64exact copies; new publication review/Gitblob verification required. Wholef41raw equality remainsFALSE onlyfor124declared counterquote removals; retained900root provenance inferred; byte comparison is not visual inspection. No defaultPERSONbleeding/selfhit/fatalAA TIMED/player/MP/easy native coverage claimed.

Campaign choices, accounting and consequences remain in EECH_DC. FinalTask46 still needs ordinary shipping40/all8/post3/currentCI/final independent review/normalmerge, then wholeD2scope acceptance. No baseline/tolerance/scenario-definition change. D3 has not started. Current private compactcheckpoint R:/d2-evidence/D2-LIVE-CHECKPOINT.json supersedes historical at-the-time entries below. Full sources/exports/roots/raws retained; only completed reviewed Cargo target caches removed, shared/active511/WSLdaemon/Rmapping kept.

# D2 — EEDCS-46 physical combat resolution: original characterization

Lead checkpoint, 28 October 07:58 UTC: real-hit225 source and ALL focused runtime groups are independently closed: centred-solid94973bytes, SITE113688/SITE_UPDATABLE116036 original35bytes; default600 controller37651/native0/all984994bytes exactly forecast891/6440events/audit0; default900 controller50679/native0/all3 complete files exactab-base; providerqueue24801actual0/all8native0, matched180reference61writes/40fatalanswers versus no-damage0/0 and six genuinely reached DAMAGE refusals before writes. All post1179inputs/all40exports/BINF/sourceclean match. Declared124 originalf41 blast-counter quote removals remain, so fullraw equality to f41 is FALSE; baseline900root provenance inferred; sampled losses not totals; no-damage varies model AND RNG policy. Safe proof real-225-runtime contains67 exact copies, no raw retail/binaries/fullrecordings. Completed real225 Docker target cache alone removed; all exports/evidence retained. Timed/critical/self-hit source1c943ffb5 is independently corrected: P2 host eligibility closed, sourcechecks59health/66damage/25forecast/Cargo/Clippy0. Root closes residual two P3 comments only, preserving all executable lines and linecounts; no source-test/native rerun for comments. NEW timed clean build/native600/900/controlled/hostauthority/refusal/performance gates remain, with11fresh provider roots and reviewed drivers. Task45fullpack83709 remains active; all8 currentCI passed at52b0325ce but complete40/all8/post3/finalevidence/currentCI/normalmerge45 still gate. Impact/blast, bridge collapse, physical factual reports/R3/R5 and wholeD2 reconciliation remain; campaign decisions/consequences core. Ledger42 published ONCE/preservesall5reports, no replay, no D3/baseline/tolerance change. This supersedes historical state below.

Lead checkpoint, 28 October: the same independent reviewer closed forecast-only source35c3..91bc07e07 with no blocking findings. Contract/layout tests,25 production C bridge checks and default/fixture Clippy pass; the preserved failed compiler log has a scoped whitespace attribute. No forecast candidate native validation is claimed. The original SITE and SITE_UPDATABLE probes are also independently closed: each has two HE health writes, two solid capability calls without health writes, and two explosion invocations/three blasts. Both actual native/controller exits0, post1179 inputs/all40 exports/BUILD-INFO match. These are bounded original witnesses, not all13-kind or candidate acceptance. Exact safe copies and limits are in eech-campaign/reference/d2-combat-characterization/original-runtime. Earlier controlled refusals remain visible. Task45's ordinary fullpack remains active; its delivery precedes46. No D3 or baseline/tolerance change.

Task46 (branch `codex/d2-combat-resolution`, Jira EEDCS-46, PR105, In Progress) has moved its FIRST physical piece into the host, as an UNACCEPTED candidate: the damage FORECAST (`weapon_damage_capability` with no weapon, asked by the target selection), and nothing else; the original characterization of the other paths continues behind its own gates. Task46 cannot merge before Task45 is delivered. The branch composes the reviewed Task45 weapon-execution runtime (`a137fc3ec`, ancestry-merged with accepted integration `83539982`) as UNACCEPTED development.

* **First physical extraction, the damage forecast (candidate, NOT accepted):** the physical arithmetic of the forecast now runs in the host (`eech-world/csrc/cmb_step.c`, EECH's own expressions moved verbatim; `src/cmb.rs`), asked through the aim step protocol as `CAMPAIGN_AIM_FORECAST` (`mb_hostaim.h`), by value, with no draw and no fallback; the campaign keeps which weapons and targets are weighed, the salvo and rate-of-fire multiplier, the player-setting table and the conventional-munitions definition relationship, and sends everything else. Real weapon damage, the health application, the critical predicate, the timed rolls, the explosion and blast walk and the bridge collapse are UNCHANGED original code and remain Task46 work (D2) behind their own characterization and coverage gates; the fixed-structure and collapse responsibilities (R3, R5) are not hidden as a D1 limit or D3 data and stay open. See `eech-campaign/docs/m5-combat-extraction.md`. Source checks only (kernel values worked out by hand from the original, the host contract, the engine boundary, 25 checks of the real `mb_hostaim.c`, Clippy, tests); NOTHING was launched or built natively for it. ROOT-owned gates before acceptance: the default native candidate against the retained f41 natural 600 measurements (574,382 forecasts, six hashes, counters, ordered log) and the 900-second control, then default shipping 40, all 8, post 3, current CI and the normal merge after the accepted Task45 composition.
* **Second physical extraction, the damage of a hit (candidate, NOT accepted; after the closed forecast review of `91bc07e07`):** the physical result of a hit that reaches a damage handler is now the host's (`CAMPAIGN_AIM_DAMAGE`): invulnerability, the real weapon's capability with the campaign's one sample asked for as a draw, the aspect armour (`get_local_entity_armour_thickness` left the campaign), the health the capability leaves (subtraction and clamp) and the critical classes before and after, by value, with the forecast's reviewed bridge unchanged. The campaign keeps the sounds, the setting of the health and its side effects, the credit, the kill and the consequences. STILL COUPLED and unaccepted: the six timed health rolls and the other readers of the critical predicate (R1: the mobility gates and the helicopter branch are physical and are NOT relabelled policy), the helicopter's AI self-hit gate (R2, the one physical handler gate not moved), the explosion and blast walk, the bridge collapse (R5) and the factual-report work. Fixed structures (R3) are damaged by value with no identity. The first review of this candidate (`dc3113e4b`) found two P2 and one P3, corrected afterwards: a nonpositive capability may not write health; the fixture echo of a hit carries its four intermediate floats as raw bits, so a hit with no direction (not-a-number aspect armour, no damage) is no longer fatal in the diagnostic path; and the coverage wording (the retained natural 600 has five fatal anti-aircraft collisions; the unreached case is the fatal anti-aircraft TIMED FIRE roll). The corrected candidate is still unaccepted. See `eech-campaign/docs/m5-combat-extraction.md`. Source checks only; nothing was launched or built natively; Task45 full pack 83709 is ACTIVE and delivery 45 precedes 46; the whole of Task46 needs the established 40/all 8/post 3, current CI, independent review and the normal merge.
* **Third physical extraction, the critical class, the six timed rolls and the helicopter's own-hit eligibility (candidate, NOT accepted; after the closed real-hit correction `225ff4abd`):** `CAMPAIGN_AIM_HEALTH` makes the host the owner of the critical class of a vehicle and an aircraft for every reader, of what a change of health crosses, of the six timed health rolls (eligibility, the campaign's one sample taken only by an eligible target, the roll, the health a hit leaves, fatal) and of an AI helicopter's refusal of its own hit. The campaign keeps the setting of the health, the observer hooks at their original places, the kill and every consequence; a person never rolls. STILL OPEN and unaccepted: the explosion and blast walk, the impact switch, the bridge collapse (R5), the physical factual reporting, R3 for a host that owns structure state, the D4 repair quantities and the whole-scope audit. Cost (about 8,600 requests a second for two kinds alone in the retained natural run) is a ROOT measurement; no class cache was added. Source checks only; nothing launched or built natively; the real-hit 225 immutable build's default 600 and 900 are root-owned and still running. Task45 full pack 83709 is active and delivery 45 precedes 46. See `eech-campaign/docs/m5-combat-extraction.md` (step 3).
* **Correction of the timed/critical candidate (`d6c878c7d`, one P2 and one P3; still unaccepted):** timed physical eligibility is the host's: the DLL no longer vetoes a timed sample by the reference rules and both aircraft updates ask TIMED at the existing point whatever the class, while the reference host's eligibility, the sample sequence and the observer's counters stay exactly as before; the provider's malformed cases and comments are corrected (`bad_health_person_draw` replaced; `critical_late` changes DAMAGE classes too). Full shipping 40, all 8, post 3, current CI and the normal merge after the accepted Task45 delivery still gate Task46; impact/blast, bridge collapse (R5), physical reporting, R3 and the whole-scope audit remain.
* **Bridge/segment collapse characterisation fixture (EEDCS-46 R5; test-only; the original has since been run twice by the root and the characterisation is CLOSED, see the bridge collapse bullet below):** `lua/bridge-case.lua` with `engine:cmb_case_bridge` (C `eech_engine_cmb_case_bridge`, full host scope, weapon-fixture only) validates every argument before anything is prepared, booted or killed, surveys the retail world's bridges, applies a DECLARED direct structural failure (the campaign's own kill of one segment of an alive bridge that is next to a LIVING unsupported deck; not a weapon hit) and records the original's ordered changes over a bounded frame window through read-only hooks in `br_msgs.c` and `sg_updt.c` that are armed only by the case; the ordinary view of those sources is byte-identical to the baseline after preprocessing. Root builds and runs it (Lebanon, then Georgia if there is no suitable bridge) and accepts the actual behaviour BEFORE the bridge physics is extracted. What ran is judged from the observed events (`bridge_summary` counts them and says whether the decks landed within the window), never from `done`. See `eech-campaign/docs/m5-bridge-characterization.md` for the entry point, arguments, preconditions and the declared-unreached cases. Persistent structure ownership/report correlation (R3) and the physical factual reports remain next D2 work.
* **Impact dispatch and area blast (EEDCS-46 step 4; candidate, unaccepted, nothing native run):** the dispatch on a weapon's kill code (crater, direct hit, the number of blasts and whether they leave the target out) and the whole physical selection of an area blast (the radius, the sector rectangle, the order of the walk, the candidates, the strict range test, the modifier, the exclusion and the rocket interception) are the host's, through two new requests (IMPACT, BLAST) with the campaign supplying windows of the sectors' lists and acting on each object the host reaches before the walk continues; the campaign keeps the notifications, the kill code of a destroyed rocket and the effects. An object a blast reports is named by a handle that exists for that blast only (a transitional mirror correlation, not a persistent host-owned structure identity: that, the physical factual reports and the structure and bridge responsibilities are explicit next D2 work, and the lead has not approved closing the whole of D2 before they are complete). Source evidence: a differential of the original walk against the new path over 6,000 random worlds (identical events, mutations caught), 60 checks of the campaign bridge (after the first review's three P2 and one P3: the walk order is the host's, the radius probe's echo is consistent, build failures and timeouts are not catches), the kernel and boundary tests, fmt/Clippy, real-header syntax and strict preprocess. The bridge collapse (R5) needs its own original-behaviour characterisation first; physical reporting, a host-owned structure identity, D4 quantities and the whole-scope audit remain. Full shipping 40, all 8, post 3, current CI, independent review and the normal merge after the accepted Task45 delivery still gate Task46.
* **Controlled-fixture callback scope (after `366d55914`; still unaccepted):** the first native controlled run of the timed candidate failed in `cmb_case_state` because that test-only entry had no host callback while its health case now asks the host; the entry now takes the complete host like the collide/blast entries. Every changed line is under the weapon-fixture gate, the strict missing-host guard is unchanged, nothing native was run here, and the corrected controlled rerun is root-owned. Impact/blast, bridge characterization and collapse, physical reporting, R3 and the whole-scope audit remain.
* **Bridge and segment collapse, physical execution (EEDCS-46 R5; candidate, NOT accepted; nothing native run or built here):** after the original characterisation closed (two native runs exit 0, 320,598 bytes, identical SHA `d4c198af...`, 118 bridges surveyed, bridge 0 of 8 segments), the physical collapse is the host's through `CAMPAIGN_AIM_BRIDGE` (`mb_hostaim.h`), six small by-value questions asked at the original decision points so the synchronous interleaving and every observer hook are kept: KILL (an unsupported span with nothing to hold it), SHAPE (the damaged shape from the neighbours; both callers `br_msgs.c` and `sg_pack.c`), NEIGHBOUR (successor, then predecessor), FALL (`10 * dt`, water floor, landing; the terrain asked for once), REST (where a dead unsupported segment lies when a saved world is unpacked) and VEHICLES (the routed vehicles a killed segment kills, a window walk like the blast's). The reference kernel (`eech-world csrc/cmb_step.c eech_cmb_bridge`) is outside the DLL; `lua/own-weapon.lua own.bridge` is a plain model that is not EECH's; providers `bridge_withhold`, `bridge_slow`, `bridge_own` and malformed `bad_bridge_*` (including `_rest_alive`, `_rest_nonfinite`) exercise the real production bridge. The campaign keeps the route link's break and accounting, stopping the route's vehicles, the repair task and its quantities (D4), credit, awareness, the D1 loss consequences, the kill itself and the effects; the segment database and the terrain stay data until D3; the vehicle handle is the transitional correlation of step 4. Source evidence: workspace and fixture `cargo test`, default and fixture Clippy, fmt, 57 checks of the real driver (ASan/UBSan), a differential of 84,036 bit-for-bit comparisons against the transcribed originals, 12+12 mutants caught with build/timeout controls, the other four guards rerun, syntax (22 OK) and strict preprocess. See `eech-campaign/docs/m5-combat-extraction.md` (step 5). STILL OPEN: persistent structure ownership/report correlation (R3), the physical factual reports, D4 repair quantities, the whole-scope D2 audit and the shipping gates; the ROOT's native comparison of this path against the original baseline is the acceptance evidence. Whole D2 is not complete.
* **Correction of the bridge/segment candidate after its first independent review (`b268c7e5a`; one P1, one P2; still unaccepted, nothing native run):** the review rejected the source (and the clean `b268` fixture build, UNRUN, never to be launched or relabelled) before any native launch. P1: the bridge declarations had swallowed `#[repr(C)]` of `eech_weapon_state` in `eech-engine-sys` into a doc comment; restored, with the inner size, alignment and all 19 field offsets compared with the campaign as compiled and a control that a reordered mirror is reported; the red WAS observable (rustc reordered the mirror before the fix). P2: the vehicles walk resumed through an entity its own kill had released (a weapon is destroyed by its kill); the campaign now refuses, BEFORE applying anything, a target whose kill is its destruction (`campaign_blast_walk.resumable`, answered by `sg_dstry.c vehicle_resumable` from the dispatch tables), keeps the successor read after a kill, leaves the closed blast flow untouched, and distinguishes this lifetime guard from the reference's physical rule. Destruction and slot-reuse stand-in cases, red against the `f5a360199` driver (2 failures) then green (64 checks), with controls and a 13th mutant. The publication P3 was fixed by the root. R3, the physical factual reports, D4 quantities, the whole-scope audit and the shipping gates remain; whole D2 is not complete. See `eech-campaign/docs/m5-combat-extraction.md`.
* **Structure identity and physical factual reports (EEDCS-46 step 6, R3; candidate, NOT accepted; nothing native run or built here):** built on the independently closed corrected R5 source `636ef798a`. The structures (site, site updatable, city building, pylon, scenery, bridge segment) now have a real host-issued opaque identity with a per-incarnation lifecycle (`CAMPAIGN_AIM_IDENTIFY`, `mb_hostobj.c`; ended by `set_free_entity`; never a heap index, pointer or seed; refused if empty, overlong, with a control character, issued before or a roster member's), reusing the roster's identity for mobiles (no double binding). Blast and vehicle window entries carry the identity and the host must repeat it for the object it names (a stale or misdirected action cannot affect another object). The host reports a physical fact by identity (`engine:report_object { host_id, damage_level?, destroyed? }`, `engine:structures()`): the campaign applies it once through the object's own collision steps (set the level, then kill), so the real handlers produce the consequences (a segment's kill asks the host about the collapse; a rostered mobile's goes through the D1 loss observation); unknown, ended, reused-slot, duplicate, repairing and client reports are refused. Campaign choices, credit, awareness, route/repair policy and D1 accounting stay in EECH_DC; a report carries no credit. Evidence: the REAL registry and report code against a fake entity heap with slot reuse (37 checks, ASan/UBSan, 15 mutants caught), window identity checks through the real driver (bridge 70, blast 63; 15 and 14 mutants caught), the differentials rerun, workspace and fixture `cargo test`, default and fixture Clippy, fmt, the C-vs-Rust size/alignment/field-offset probes of every new DTO (cases 130 to 165) and a stand-in-engine test of the new controlled witness `lua/object-report-case.lua`. LIMITS: lazy identities (first physical need), no save/restore of identities, no identity for weapons or unrostered mobiles, no bridge identity; the layout probes had no observed red. ROOT-owned: the default/controlled non-regression, the new native witnesses (control plan in `eech-campaign/docs/m5-combat-extraction.md` Step 6) and the independent review. STILL OPEN in D2: that native validation, the whole-scope audit and the shipping gates; whole D2 is not complete.
* **Correction of the structure identity and report candidate after its first independent review (`0ca067d1e`/`aff4904fd`; one P1, one P2, one P3; still unaccepted, no Windows or native run of any of that source):** P1 the host's identities are now ONE never-reused namespace in BOTH directions: a group member bound later may not take an identity ever issued for a structure, live or ended (`gp_rosterid.c campaign_roster_accept_host_id`, the roster's unchanged acceptance moved out of `campaign_roster_link_member` and called before it commits; no mobile numbering, RNG or D1 change). Behavioural red: the current both-order controls run against the acceptance as it was at `0ca067d1e` fail exactly the reverse checks (5 failures), green after (56 checks, ASan/UBSan; 20 mutants caught). P2 the persistent identity now reaches the actual physical subjects: the common aim source/target envelope uses the shared registry (a rostered mobile keeps the roster's identity; a structure is identified at its first physical need), and DAMAGE, HEALTH and IMPACT carry the identity of the object they are about in the existing envelope field (no new DTO, solvers and numbers untouched, subjectless forecast/table queries unchanged), through the C producer, the Rust engine, the DC Lua request and the reference host; covered by the real helpers in their harnesses (damage 70, health 63, blast 67 checks; 6 mutants caught), the engine crossing test, the DC test and the public Lua host. LOS and sensor envelopes keep their roster-only fill (declared). P3 the structure identity string is documented as borrowed for the callback only. Workspace and fixture `cargo test`, default and fixture Clippy, fmt, syntax 22 OK; failed attempts retained. The Root builds a fresh clean source and runs the six planned shipping roots of `lua/object-report-case.lua` (now also counting subject-identity questions) and the default/controlled comparisons. STILL OPEN in D2: that native validation, the whole-scope audit and the shipping gates; whole D2 is not complete. See `eech-campaign/docs/m5-combat-extraction.md` Step 6.
* **Windows runtime memory-safety defect of the Step 6 registry, repaired in source (`f4700bbab` run; candidate, NOT accepted; no native validation of the repair):** the Root's first native run of the six shipping `object-report-case` roots on `f4700bbab` actually terminated all six at native -1073741819 (0xC0000005) with only the boot row in each trace (`root-r3-f470-six-report-crashes.json`, `root-r3-f470-structure-crash-disassembly.log`); none establishes report influence, withholding, generic-host authority or a refusal. Cause, verified in source: `campaign_object_structures` walked the entity heap to the compile-time ceiling `MAX_NUM_ENTITIES` (524,288) while the run allocated `number_of_entities` slots (125,000 or 135,000) and `get_local_entity_ptr` is unchecked. Every slot index of the registry (enumeration, identity map allocation, lookup, identification, release) is now bounded by the capacity actually allocated, and a heap that changes size under live identities is refused; nothing else changed (kinds, order, identities, lifecycle, report semantics, handlers, allocation, numbering, physics). The fake heap that hid it is replaced by one allocated EXACTLY the run's capacity below the ceiling with a counting accessor: against the `f4700bbab` registry the capacity checks fail (19 of 82, at five runtime capacities), against the fix 82 pass under ASan/UBSan; 23 mutants caught; real-tree compile, workspace and fixture `cargo test`, fmt and syntax pass. The Root rebuilds a clean source and reruns the six roots; the repaired source is not claimed to work natively until then. See `eech-campaign/docs/m5-combat-extraction.md` Step 6. D2 remains active, whole D2 not complete.
* **Guard order of the structure registry after the focused review of the heap-capacity fix (`30fbdfcbc`; one P2, one P3; candidate, NOT accepted; no Windows or native run of this source):** the heap, the map and the slot are validated first in `campaign_object_host_id`, `campaign_object_identify` (before the cache, the type read and the host question), the report resolution (before any record is followed; a rostered target's slot before it is read) and the enumeration, so a heap that is not the registry's is refused by name without returning an identity, asking the host or applying anything. 94 checks (exact-capacity heap, instrumented entity access, pointer at the capacity, cached and fresh identity and reports after a capacity change, both stale slots, restore), 26 mutants caught, 12 failures against `30fbdfcbc`. Evidence wording corrected (an out-of-bounds read faulted, not "the first"; the old map was sized for the ceiling, not "2 MiB excess"; the old red was guarded-access instrumentation, not an ASan reproduction). See `eech-campaign/docs/m5-combat-extraction.md` Step 6. Whole D2 is not complete.
* **Evidence note (35c3d6777 housekeeping):** `checks/case-helper-status-controls-2.txt` is a run of the helper status controller with `871b51362` as the previous driver: it FAILED one assertion (the one specific to the `450ecf60a` mutant's off-map stop) and its trailing `CONTROLLER_EXIT=0` is the logging wrapper's, not the controller's; it is retained as a failed attempt and is never a pass. The passing record is `checks/case-helper-status-controls-3.txt` (every assertion passed, default previous driver).
* **Observer:** a feature-gated, read-only combat observer (`weapon-fixture` only; empty macros otherwise) records damage capability (real and forecast), damage application, the collision handlers, the explosion blast, the repair writers and the six per-frame timed health rolls, as SIX ordered hashes (`damage collision blast repair timed order`), counters per entity kind and an ordered event log. Sixteen entity kinds have a collision handler and all are wrapped; thirteen hold a health and are read, three (bridge, cargo, weapon) are only counted. `wrapped_collision_handlers=32` is cumulative: `initialise_entity_system` and `reinitialise_entity_system` each re-register and re-wrap the sixteen (see the design note).
* **Reviews:** the same independent reviewer closed the five findings of the first observer review, the report-parser and guard follow-up (`f41fd9202`) and the evidence-helper correction (`f5430bf1e`). Disposition of the typed-parser issue (root, from the real f41 trace; closed by the review of `450ecf60a` as sound): `lua/combat-report.lua` protected hash WORDS, so the `cmb-kind` COUNTER `blast=69` was written as the JSON string `"69"` while `cmb-hash blast` correctly stays a 16-hex string; no value was lost and no physics is involved. FIXED in the shared parser (a hash is protected only inside the `cmb-hash` event), proved red then green by a test that loads that same parser on both `cmb-hash blast=0000000000000255` and `cmb-kind blast=69` (plus the six hashes losslessly), and the audit now declares the legacy typing of the kept original run instead of claiming every counter numeric. The earlier observer findings are not reopened.
* **Build and native evidence:** clean fixture build of `f41fd9202` (BUILD and OUTER 0, dirty 0, all 37 exports verified by root; `f5430bf1e` changed no runtime input). Original natural run `natural-01` (Lebanon, seed 1, 100 ms, 600 s, checkpoints 60 s): native 11296 exit 0 and the unified driver 90660 exit 0 (root), ended 03:54:24 UTC, all 1,179 post-run inputs and all 37 exports and the build manifest unchanged (root-natural01-completion.json), 985,242-byte trace, audited by `eech-campaign/tools/combat-trace-audit.py` (PASS: six string hashes everywhere, trusted with no overflow, damage entries equal exits, event log contiguous and consistent with the counters; its self-test passes 63 cases on that real trace: 3 accepted, 43 targeted corruptions or incomplete runs rejected (including the required report structure, the emitter order and the typing of the cmb-kind `blast` counter) and 17 bounded-reader and limit controls). Its raw trace is the LEGACY schema (the `cmb-kind` `blast` counter is a decimal string in all 124 such rows; every other counter is a number) and is kept exactly as written; it is not rerun for a format correction, and the audit reads that counter as an integer and says so. Measured (the event log is exactly 6,440 events, sequence 1..6,440 without gap or duplicate): 234 weapon collisions on seven of the thirteen health-bearing kinds (the other six have none), 574,382 forecast computations, 1,475 explosions (982 blast twice), four of the six timed health rolls fired. NOT reached: collisions on OBJECT, PERSON, PYLON, SCENIC, SITE, SITE_UPDATABLE; a person's bleeding and an anti-aircraft vehicle's fire; a player target, the multiplayer multiplier, the no-weapon exit, invulnerability, a rocket killed by a blast, a fixed restore. A canonical 900-second observational control is separate and its result is not claimed here. One seed and one run are what this is.
* **Reviews of the controlled witness, audit and parser:** the combined pass on `450ecf60a` found five P2 and one P3; the focused pass on `f35388cee` CLOSED the original timed eligibility (a person's bleeding is source-unreachable and declared, the anti-aircraft case is bounded by the original predicate), the complete weapon snapshot and overflow cleanup, the pre-launch placement validation, the missing-row rejection, bounded reading with valid limits and the natural-only wording. FOUR items remained and are FIXED in this change set, awaiting the same reviewer's focused closure on the new diff: (1) the audit checks the emitter's ORDER, so a `run` marker moved after its whole final report, or a foreign row between a checkpoint marker and its report, fails; (2) the shared reader validates its limits before ANY read (a negative line limit made `readline(-1)`); (3) the witness's counter flattening keeps every indexed row identifier (kind code, warhead, entity kind, timed kind), so a later `cmb-blast-kill` row can no longer overwrite an earlier one and hide a changed counter; (4) the new mutation helper no longer masks its status. Not reopened: the C, game, Rust and observer source, which are unchanged from `f35388cee`.
**First controlled original run, 13 rejections (EEDCS-46, after `9d7cd106a`):** see `eech-campaign/reference/d2-combat-characterization/README.md` (section "The 13 rejections of controlled-original-01"). Leading explanation, INFERRED not proven: the high-explosive carrier ran out of ammunition after six launches (a plan defect, not production physics); the launch reason is now returned instead of discarded, and nothing native was run for this. The review of `871b51362` found two P2, fixed after it: the ammunition figure is an ESTIMATE of launch calls (a launch fires a salvo and spends several rounds), so the class of a role keeps its affordability and every case picks its carrier from the LIVE count; and the case driver validates the launcher kind and the weapon sub type BEFORE the launch (an invalid argument stays an error, a dead launcher and a launch that creates nothing are data). The first partial original run (`9d7cd106a`) was independently supported: six high-explosive health writes; twelve solid-shot capability calls without any health write; six weaponless exits; anti-aircraft 201 draws, 46 hits, 0 fatal; it is NOT claimed that all five newly reached families got health writes. See `eech-campaign/docs/m5-combat-characterization.md`.

* **Controlled witness:** written (`csrc/eech_cmb_case.c`, `lua/combat-case.lua`, engine and module glue), NOT built, NOT run, awaiting the focused review and then a clean fixture build of its own identity. Source checks: C syntax with the real game headers both ways; 27 checks of the REAL driver over a stand-in entity list under AddressSanitizer and UndefinedBehaviorSanitizer (the previous driver fails 16 of them; 32 checks after the refusal-as-data change, see the 13-rejections section); Clippy default and fixture; tests including the Lua parse. A controlled acceptance will read the per-case counter deltas (each indexed report row under its own key) and eligible draws, not the process exit or `preconditions_met`. Its runtime inputs are the two Lua files `lua/combat-case.lua` and the new `lua/combat-counters.lua` (and `eech_cmb_case.c` unchanged since `f35388cee`): a clean fixture build of the head is a new identity.
* **Limits kept:** the first sanitizer-guard flood log was discarded (only 4,096 bytes survive) and its cause is not asserted; the later guard runs use test-only non-PIE flags. General terrain and model data remain D3 coupling and repair and logistics D4; neither defers the physical hit, blast and fatal decisions. D3 has not started.

Design, source map and the measured census: [`eech-campaign/docs/m5-combat-characterization.md`](eech-campaign/docs/m5-combat-characterization.md); evidence: [`eech-campaign/reference/d2-combat-characterization/`](eech-campaign/reference/d2-combat-characterization/README.md).

# Composed Task45 status (a SNAPSHOT of the Task45 branch as composed; Task45's own STATUS.md on PR104 is current for Task45, and the statements below are as of 28 October 02:10 UTC)

Lead checkpoint, 28 October 09:40 UTC: Task45 original ordinary shipping pack83709 and launcher completed with actual exit0. All40 accepted checks PASS; all8 complete Tacview, observation and metrics files are byte identical to the verified completed destination baseline. All3 post-run retail manifests and35 exports/BUILD-INFO remain unchanged. Safe final proof is reference/d2-weapon-characterization/final-pack (22 exact private copies plus verification/manifest). Frozen clean runtimea137 and all8 build/scenario frontiers equal candidate093; subsequent documentation/evidence changes do not alter runtime. Focused independent reviews remain closed; NEW final-evidence review, current-head CI and normalmerge45 still gate delivery. Task46 impact/blast source0400af is under the same independent reviewer. Timed366 default600 and900 and corrected511 controlled/AA authority/refusal gates are independently closed. Generic PERSON native0 reaches4samples/1hit/1fatal under explicitly combined own DAMAGE plus stochastic TIMED variation; post/audit/review pending, not default or timed-only evidence. The short180s provider queue ended1 at an untriggered second-draw assertion; no unsafe answer or actual refusal is established by that case, and partials remain. Physical factual reports/R3, original bridge characterization and extraction, and whole-D2 reconciliation remain. Campaign decisions/accounting/consequences staycore; noD3. Ledger43 remains current with all5reports preserved. The private D2-LIVE-CHECKPOINT.json is refreshed; historical at-the-time entries below are superseded.

Current checkpoint, 28 October: the same independent reviewer closed the a137 natural600, controlled, generic-host and matched600s reacquisition evidence without blocking findings. Natural600 is raw-byte identical to original. Controlled native/driver0/all11 has exactly two Lua source-position diagnostic differences; all other5,760 lines/all5,718 physical records match, so whole-trace byte equality is explicitly false. Generic native0/all11 reaches14birth/14release/9,724flight/400animation/2,000decoy calls; the unchanged reference checker rejects its absent EECH-private table diagnostic, while the separate scoped generic audit passes. Matched host reacquisition override changes actual births1,700 to1,109 and losses50 to47. Post-input/build identities match. Safe proof: reference/d2-weapon-characterization/integrated-a137-bounded.

The final900s weapon control completed native/controller0 with all three complete artifacts byte-identical to retained ab-base, post1179 inputs/all35 exports unchanged; original baseline-root provenance remains inferred. All21 B providers, their13 specific refusals, all21 post-inputs and35 exports are independently closed. Matched180s fast/drag/short and600s reacquisition/seduction/release controls establish host authority. Final Ccontroller38392 and all3 natives exited0, all3 post-inputs/all35 exports unchanged; slow release rounds3525→1194, birth actions1700→1098, losses50→51. The same reviewer closed all focused evidence. Public safe proof at reference/d2-weapon-characterization/integrated-a137-final-focused (62 copies verified against committed Git bytes), plus the earlier bounded dossier. Fullpack83709 remains ACTIVE on immutable clean a137; complete40/all8/post3/source identity/currentCI/normal merge45 remain gates.

Task46's original 600-second characterization and 900-second control completed with actual original-driver and native exit 0. The natural run has 6,440 contiguous events and twelve complete trusted groups; all three complete control artifacts match retained bytes. Input and build identities match, and independent review is closed. The first controlled run (9d7cd106a) provides six HE health writes, twelve solid-shot capability calls without health writes, six weaponless exits and 201 AA draws/46 hits. Its thirteen rejections remain visible. The corrected second run (35c3d6777, clean build84109, all40 exports verified) completed native/controller0, but remains PARTIAL: ten preconditions succeeded and29 valid launches were refused. Its two first shots wrote health; ample remaining ammunition and calls at the same clock require release-gate investigation. AA took201 draws/41 hits. All79 integrity reports, post1179 inputs and40 exports match. The same reviewer is examining this new evidence; no all-family damage coverage is claimed.

The first physical damage-forecast candidate is clean and pushed at91bc07e075ad4d58061d0304f3d90389997ee6bc on codex/d2-combat-resolution (PR105). The sole Sonnet5.5 High CLI10153 actually exited0 and PID6068 is gone; its relevant local context remains f8561f85-1ce4-4047-97df-48f777bbc752. Only weapon==NULL capability arithmetic moves to the host; campaign target/weapon/salvo choices remain core. The independently closed original574,382 forecasts and900-second purity control support this bounded extraction. Source contract/layout tests and default/fixture Clippy pass with actual Cargo/Docker0; the first precision failure is retained. One failed C compiler diagnostic has trailing spaces in a preserved raw log, to be classified with scoped evidence metadata. The same reviewer is examining the new source; no candidate native run or acceptance is claimed. Real damage, critical/timed damage, impact/blast, bridge collapse and factual reports remain D2 work. Task45 delivery precedes46; development continues while its fullpack runs. No D3 or baseline/tolerance change. Older sections below retain their at-the-time history and are superseded by this checkpoint.

Two original35c3 selector probes are now independently closed without blocking findings: kinds=SITE or SITE_UPDATABLE, control=OBJECT, timed_seconds=0, separate verified retail roots. Both actual native/controller exits0; nine preconditions, zero refusals, nineteen trusted reports and31/19 contiguous events. Both choose HE54 and solid76. Each has two HE direct/modifier health writes, two solid capability calls without health writes, one weaponless exit and two explosion invocations/three blasts (including land double blast). SITE has7 selected-kind applications/5fatal/4 area-blast notifications; SITE_UPDATABLE has8/3/5. Full-run health writes are20/8 and real capabilities22/10. All1179 inputs per root,40exports and BUILD-INFO match. All six pre-case hashes match original checkpoint60; controlled actual timestamp is60.099273681641. No timed frames/draws were added, although the PERSON predicate and AA health setter still execute. HIT_TARGET excludes its linked target only from the area-blast walk; both cases also record direct collisions. P3 wording correction preserves before-proof copies and hashes. Original02's no-create diagnosis remains source-supported but unproven; no all13-kind coverage is claimed.

The latest Task45 live prefix comparisons exited0: approximately56MB Tacview and38.5MB observations per reference run match accepted bytes. This is partial evidence only; fullpack83709 remains active. The portable eech-regression skill and actual-run records remain the execution/handover entry points. Inactive completed original9d7/35c3 Cargo target volumes were removed after no-container and export-preservation checks; exported binaries, sources, roots, raw traces and current shared/Task45 caches remain retained.

Accepted integration `83539982a5b19b3e5636454ef36066505065f3ab` (Aim44, PR103) is merged into this branch as an unchanged-runtime ancestry merge: before and after the merge the git tree objects of `aphavoc` (`2fa320502f5f`), `modules` (`cb40c4563825`), `eech-campaign/crates` (`2acc1c6fbf08`), `eech-campaign/tools` (`a294c835bd13`), `Cargo.toml` (`6651665cec5e`), `Cargo.lock` (`bef9a5e432cb`), `scenarios` (`d9a521c0db90`) and `regression` (`090bd08a2685`) are identical, so the frozen `a137fc3ec` build and evidence describe this runtime exactly. The only textual conflict was this file (the integration copy is the Task44 status); this branch's version is kept. The merge adds the Aim44 delivery evidence under `reference/d2-aim-extraction/` and the handover review note.

Lead checkpoint 28 October: aiming44 is DELIVERED by normal PR103 merge83539982a5b19b3e5636454ef36066505065f3ab at02:10:28UTC. All9 CI checks passed at reviewed headb0a6b2a6b; fetched merge tree equals that head. Original runner and launcher80926 exited0; exact40checks/all8 complete artifacts/post3 retail manifests/source-build identity pass. The same independent reviewer closed the new final proof without findings. Jira44Done/comment10798; ledgerv41 persisted ONCE02:11:32.979UTC and read back with all5 report wrappers and all old HTML byte-identical. Do not replay that update. Accepted integration is now83539982.

Weapon execution45 remains In Progress. Same independent review througha137fc3ec has no blocking source finding; helper-status correction is closed. Clean fixturea137fc3ec-clean and ordinary shipping build2720 completed BUILD/OUTER0/dirty0; all35 exported hashes verified. Final h1 natural600 completed02:10:04UTC with actual native0 captured through a held process handle: all10,532 events and complete3,097,859 bytes equal the original, post1179inputs/all35 exports unchanged. Its original driver was intentionally detached, so driver completion is not claimed. The earlier wrong-warmup20 controlled attempt was intentionally stopped with partials retained; corrected h1 controlled cases explicitly use original60/700 and continue healthy. Generic positive-activity/refusal and controlled gates remain.

The ordinary Task45 fullpack is ACTIVE exec83709/runnerPID14296 from02:12:10UTC, using frozen cleana137 source R:/d2-evidence/shipping-source-d2-weapon-a137fc3ec and fresh candidate-roots-d2-weapon-execution/{georgia,lebanon,lebanon-repeat}. Skillpreflight passed all3 input manifests, exact40 expected checks and childPython3.12.14. Out R:/d2-evidence/candidate-pack-weapon-a137fc3ec. Preserve source/bin/roots and actual run.json/outerexit; no full result is claimed. Root owns full40/all8/post3/finalproof/CI/normaldelivery after focused gates. Combat resolution/factual reports remain D2; D3 has not started. No baseline/tolerance changed. Details: private ACTIVE-RUNS.json. These current checkpoints supersede older at-the-time running/build statuses below.

Task45 / PR104 remains In Progress on `codex/d2-weapon-execution`. The ORIGINAL characterization is accepted at fixture `aaceb0e7d69fa405941cf3078a52d3438fb62a8f`: controlled witness, two complete 600-second traces, and the 900-second campaign control all completed with actual native/driver exit 0. The two traces match raw bytes; complete control Tacview, observations and metrics equal the retained baseline. Post-run retail manifests match. Original failures and exact provenance remain in `eech-campaign/docs/m5-weapon-characterization.md` and `reference/d2-weapon-characterization/`.

The first physical extraction now moves ballistic-table generation and guided range simulation into the host, sharing the original unguided integration expressions. Source through `abc6d35bd2e2dddda15e7383f1ff206f69be8fb6` has no outstanding independent source-review finding. Its clean Windows fixture build completed with BUILD_EXIT0, zero uncommitted build inputs, and all 32 exported manifest entries independently verified. This is fixture evidence, not shipping acceptance.

Candidate `s2-600-a` completed 27Oct23:38:54UTC with actual native/driver exit0 after839s (sourceabc6, PID23956). Root's unchanged strict comparator returned0: all10,532 events match across every scope; full3,097,859 raw bytes are identical to ORIGINAL (SHA256 faa94591c299ddb3ae20fe8b8101a4317f431139871297e65703c042d6289c0b). Post-run1179-file retail manifest is identical. `R:/d2-evidence/weapon-characterization-20261027/root-s2-600-a-completion.json` records this subset.

Sequential queue22876 continues on the unchanged fixture: controlled `s2-case-a` native15312 started23:38:54.416217UTC; provider authority/refusal cases and900s control follow. Those results remain pending. Earlier test/build failures are retained; corrected affected-package tests and default/fixture Clippy completed with actual Cargo/Docker0. Current abc6 CI has seven successes and two i686 compile failures caused by LuaInteger width at main.rs125; the conversion fix is in the working tree and still needs current CI. No original baseline or tolerance changed.

Integrated candidate source (flight at `ac9b5a6d8`, flight/launch/release at `d71fec4fb`, then the review corrections and the post-flight seam below): the host owns a weapon's frame of flight (`weapon_movement`: lifetime and expiry, guidance and lock source, physical target tracking and intercept, the guided and unguided step, in-flight contact selection, map bounds, ground contact, the terminal kill code), its birth (launch velocity, lifetime, decoy or fuze timer, dispersion and attitude), the rate-of-fire gate that decides the rounds a launcher releases or a refusal, and, since the combined review's P1, what happens to the weapon after its movement: a falling crate's or ejector seat's parachute (it sets the drag and resets the crate's lifetime), a decoy's timer and its restart draw, and whether a seeker locked on a decoy's launcher goes for the decoy. The campaign hands the host the weapon's state and answers, by value, pure questions of its world (launcher and target facts, wind, the terrain below a point, the first object or tree a segment meets: generic geometry and data services, declared D3 coupling); the host answers with ordered actions the campaign applies and takes the campaign's shared random samples in the order it asks; the entity-local stream never leaves the campaign. The weapon-specific collision algorithms (`collision_test_weapon_with_*`) and the physical decoy and parachute functions were removed from the campaign. Whether a seeker whose decoy expired goes back to the decoy's launcher is the host's too (`CAMPAIGN_AIM_WEAPON_REACQUIRE`; the campaign still unlinks the seeker whatever the host answers, so the decoy's destruction always makes progress), and the decisions the campaign acts on from a decoy or a seduction are refused unless exactly 0 or 1. The combined independent review of `d71fec4fb` found six things, all corrected in source: the post-flight seam (P1), a batch that named the same ground question ten times for a weapon that did not move, a target-link action that applied the last contact lookup instead of the one that found the target, the generic Lua model's exact comparison of a double against a single-precision transport, a clockless release that could be told to write a clock, and an overdraw provider that disarmed before its first reply. Design: `eech-campaign/docs/m5-weapon-execution.md`. Source checks, actual Cargo/Docker 0: affected-package tests (`cargo-test-p2-1.log`: eech-engine 48, eech-world 197 library, 19 binary, 8 host/Lua, load_dc 2, eech-dc 3), default and fixture Clippy (`clippy-p2-1-*.log`), a C-level check of the strict decisions against the real `mb_hostaim.c` (`hostaim-validation-1.log`: only 0 and 1 accepted), a C syntax compile of the weapon-execution sources feature on and off (a subset of the engine: `init.c` and the other callers are not in it). DIAGNOSTIC native run of `d71fec4fb` (clean build, before those corrections and the post-flight move; not final acceptance): `s4-600-a` native 19380 exit 0 after 1,210 s, trace byte-identical to the retained original (3,097,859 bytes, SHA-256 `faa94591…c0b`), strict comparator 0 differences in all 10,532 events. The latest source (decoy-expiry reacquisition, strict decisions, and the sibling-file lookup fix found by the first native provider run, which the first provider chain hit as `g1-pref` mode=reference native exit 1 before boot) is the final candidate `a137fc3ec`, built clean as `a137fc3ec-clean` (BUILD_EXIT 0, OUTER_EXIT 0, 0 uncommitted inputs; the first build of the same commit recorded 1 uncommitted input, a CRLF log the WSL Git saw as modified, and is kept, not used). Native evidence on it so far: the natural 600-second trace (`h1-600-a`) is byte-identical to the retained original (3,097,859 bytes, strict comparator 0 differences in 10,532 events; native exit 0 captured through a process handle root held across a controller transfer, the original driver was detached and its completion is not claimed), and the reference-host provider run (`h1-pref`, 600 s, native exit 0 after 1,139 s) reproduces the original's counts (342,014 weapon steps, 18,710 decoy updates, 3,017 restarts) with 1,700 births, 2,150 releases, 25,089 seductions and 57 reacquisitions asked and returned, no failure. Still pending: the controlled witness (warmup 60, settle 700, as the original; an earlier run with the queue's wrong default of 20 was stopped by root and is retained, not equivalent), the generic-host witness with positive counts, the altered-host and malformed-provider cohort (`h1-queue-b`), and the ordinary 40-check pack. Combat resolution and factual reports follow within D2; campaign choices, fire intent, accounting and consequences stay in EECH_DC. Terrain/model lookup services remain declared coupling for D3; this does not defer D2 contact/hit decisions. D3 has not started.

Authority: live M5 goal11534337v3 and migration ledger14548994v40 (progress appended 28Oct 01:20:56 UTC by root; table proof complete, full Aim40 awaiting Georgia, Task45 expiry/boolean corrections under way; scope and acceptance unchanged) reread 28Oct. Accepted integration remains04bf645a; Task45 composes reviewed aiming source as development, not accepted delivery. Keep the aiming frozen source/binaries/roots untouched.

## Accepted increments

D1a7934aa7/all15tasks accepted; destination baseline87915 actual0/40checks and complete accepted bytes reused. Do not rerun healthy baselines for handover.
LOS42/PR101 delivered3ce17432e; sensing43/PR102 delivered04bf645a96f8277ca54d4a575b77f35b5ee68052. Both40checks/all8complete artifacts/post3inputs match; original checker/reporting exit distinctions remain in their dossiers. Jira42/43Done. All5 ledger epic reports remain intact.

## Aiming44 — frozen full validation, not delivered

PR103 branchcodex/d2-weapon-aiming, reviewed physical source52206fef322dbb2f4db18620d44da1a57408ad7f. Root review record docs/handover/D2-AIM-DELIVERY-REVIEW.md. Person-checker/TOFpartial-write/W1test findings resolved; no new independent findings. Earlier600s and900s witnesses retain their original sources and byte equality. Corrected native sentinel probe sixscenarios/15checks/native0; v8fmt/default+fixtureClippy/tests actual0. MalformedTOF native modes unrun; focused contracts cover them. Failed attempts remain retained.

Clean ordinary source R:/d2-evidence/shipping-source-d2-aiming-52206fef3; bin eech-campaign/target/windows-d2-aiming-final. Fresh build54527 actual0/all30manifest hashes match. DLL162c9920d829ee5e457cad01897e4f66a98b62f6543d2f576631f6a54fe26e48; World16de0cd0ca07d97a59a6b6e15b2caa1ad0ff1449fd5dfc396ffb6fc96f2f77f9. Skillpreflight62398 actual0/threeinput manifests/childPython3.12.14 verified.

Full pack ACTIVE exec80926/runnerPID25444, started27Oct19:46:22UTC. Out R:/d2-evidence/candidate-pack-aiming-52206fef3; run.json records actualstatus/exit, pack.stdout.log and pack.stderr.log retain originals. No complete40-check/full-file result yet. Keep source/bin and candidate-roots-d2-aiming/{georgia,lebanon,lebanon-repeat} untouched. No duplicate run after a quiet console or timeout: inspect actual PIDs/creation/commands and progress. Root owns complete8-file comparison/post3inputs/actualexits/currentCI/normalmerge/JiraConfluence. Delivery44 precedes45.

## Working directories and execution

R:/eech_source_code is Task45 author checkout. R:/d2-sensor-detections is now the Task44 delivery checkout on codex/d2-weapon-aiming (completed sensing checkout reused; ignored tested bins preserved). Frozen sensing/aiming snapshots and private evidence remain separate/untouched. Keep R: mapping while processes depend on it.

One implementation lane: ClaudeCLI-p exactclaude-sonnet-5-5 High, noAgent/Task/nestedagents. Same independent Codex reviewer d2_los_review. Task44author73613 actuallyexit0/PID2936gone; Task45 starts a fresh localcontext recorded in externalACTIVE-RUNS.json. Only realClaudequota exhaustion permits oneGPTSolHigh implementation fallback; preserve checkpoint and return toClaude whenavailable. Never infer process exit from a result event alone.

Use installed ~/.codex/skills/eech-regression/SKILL.md and runbook for full packs: unchanged repo runner, explicitPython3.11+, process-local childPython resolver, -B/bytecode disabled, fresh short paths, --preflight-only then distinct fresh runout. Actual pack result, exact40checkset, complete8files and post3input manifests are separate gates. No baselines/tolerances changed.

Existing WSL2 EECH-D2-Build/localUnixDocker daemon exec37778 stays running; inspect before restart/cleanup. Use physicalC paths forWSL. Preserve Latin1/embeddedNUL C sources, exported binaries and useful failures before removing disposable caches. Private retail assets/binaries/full rawrecordings stay outsideGit. Logged-in GitHubidentity fd-starscream-bot and autonomous normalmerges are human-directed; no self-approval/authchanges/adminbypass.

Detailed live checkpoint: R:/d2-evidence/ACTIVE-RUNS.json. WholeD2 goal remains ACTIVE. After weaponexecution, complete combatresolution/factualreports and whole-scope audit beforeD3. No historicalTypeScript port/call-map scope.
