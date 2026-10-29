# Original forest-geometry witness — 537c36faf measured (physical, default and host agree); three REAL state-contract defects found; corrections PREPARED, UNBUILT, UNRUN (EEDCS-49, D3)

## Measured on frozen source `537c36faf` (Root; preserved, not repeated or rewritten here)

Source tree `tree-geometry-original-source-537c36faf`. The ordinary wrapper's Windows fixture, driver and tree tests build 71228 exit 0 with all 49 exports verified. Linux 58033: format, all 18 collision driver
guards and host all-target Clippy 0. Windows 32952: all 18 guards 0, plus the missing-host override control refused with 101 as it must; all 5 test-export hashes unchanged. Native 45228: controller 1 for the
three boundary failures below, NOT for a physical answer. Every pre/post input manifest passed and all 49 payloads are unchanged. Summary
`R:/d3-evidence/tree-geometry-original-537c36faf-first-measurements.json`; immutable raw `R:/d3-evidence/tree-geometry-original-537c36faf-native-20261029`.

* All 3 default 206-row reports and the complete original transcripts are BYTE IDENTICAL to the reviewed f3 ones (witness off changes nothing).
* The controlled ORIGINAL decoder produced 6 triangles; all restoration flags 1; all four flag combinations, geometry hits, hits at depth above 0, misses, partial writes and exhausted walks were observed; geometry
  coverage gaps `none`.
* Georgia 4802 queries / 9598 ordered lookups and each Lebanon 5126 / 10148 match EVERY output bit through BOTH the actual Windows library and the real-host Lua replay. The host tree C, library and Lua API are
  UNCHANGED. This is new characterization of the ORIGINAL geometry branch and decoder plus agreement of the existing host kernel on it; it is not the model-loader extraction and not production delivery. The new
  witness itself is unreviewed and not accepted; the old 8c452f197 component review remains closed.

## The three real RED controls (each exit 1; preserved unchanged, to be re-run GREEN)

Real public host-Lua to native C witnesses, byte-copied into the repository with the same contents and SHA-256:
`tests/lua/tree-geometry-one-shot-control.lua` (614efb209ed37da6e432189fa58227d8164b45d25d89d62f62f936ce18297c15), `tests/lua/tree-geometry-refused-probe-control.lua`
(ef470bd924110fb81a8fcce1c4190845fefb53af94791ecd7b4bdb7e8ab2f895), `tests/lua/tree-geometry-conversion-refused-control.lua` (b3a13b108b22b18c138aea10b0a5cab14afdd9bb51158d1b7d0aebd0966217c0).
Their originals under `R:/d3-evidence/` and their saved outputs are not rewritten.

1. DEFAULT then CONTROLLED then DEFAULT on ONE trace arming: the default report and facts came back byte identical, but the next default record pages kept `g_calls g_true g_last g_depth`. Cause:
   `trace_witness` (how the held records are written) was set to 1 while the facts were built for a controlled probe and cleared only by `collision_trace_arm`, never per probe.
2. `collision_geometry_arm(true)` then `collision_probe(20261029, -1, 5)`: the range refusal happened in `Engine::collision_probe` before any C, so the arming survived and the next valid probe was controlled.
3. `collision_geometry_arm(true)` then `collision_probe("not-a-seed", 5, 5)`: the typed tuple conversion refused before the Lua method body ran, so the arming survived likewise.

## The corrections (source only; no mesh, vertex, original math, call order, baseline, tolerance or host-kernel change)

* State representation (1): in `probe_run_body`, `trace_witness = witness_active` is set immediately AFTER `trace_reset_records()` (which does not touch it) in the same `if (trace_armed)` block that rebuilds the facts, and the
  assignment inside `trace_build_facts` is removed. The flag therefore always describes the records held: a controlled probe's pages carry the geometry fields; a default probe on the same arming writes the old
  bytes again; arming still clears it. A probe that does not touch the transcript leaves records and flag together.
* One-shot at every entered refusal (2 and 3), at each layer, with no ambient/session framework:
  * C engine entry `eech_engine_collision_probe`: the `ENGINE_RUNNING` check and the argument check now call `eech_collision_geometry_arm_run(0)` before refusing (byte-safe edit; the file's single NUL byte preserved).
  * `Engine::collision_probe` (Rust): the body moved to `collision_probe_entered`; ANY `Err` (the range refusal before C, the engine's refusals, the report refusals) clears the arming with `collision_geometry_arm(false)`
    (idempotent; its own failure is ignored because an engine that is not running cannot be armed).
  * Lua method `collision_probe` (eech-dc): it now takes the raw argument list (`mlua::MultiValue`) and converts it INSIDE the method with `FromLuaMulti` into the same `(u32, i32, i32)`, the same scalar conversions and
    coercions the typed signature used; any error (conversion, engine) clears the arming and the original error is returned unchanged. The error TEXT of a conversion refusal can differ in its argument-position prefix from the
    typed signature's, since mlua adds that prefix only for typed arguments; the refusal itself and the engine's reasons are unchanged. No `RefCell` or other borrow is held across the conversion, which can run Lua metamethods.
  A completed probe still consumes the arming in C; a successful probe never clears anything extra.
* No new test double: the three real scripts above are the boundary tests (a mock would only prove its own setup). The 18 portable driver guards and the 3 argument refusals are unchanged.

## Root: expected next steps (NOT RUN)

Freeze the corrected source; rebuild the Windows fixture, driver and tree tests and Linux format/Clippy; re-run the three EXACT script bytes natively against the fresh binaries (expect `TREE_GEOMETRY_ONE_SHOT_OK` and
`TREE_GEOMETRY_REFUSAL_CONSUMES_ARM_OK` twice, exit 0, with the saved before/controlled/after sidecars: default before == default after; controlled with the witness fields; refused-probe and conversion-refused next probe
equal to the earlier default); re-run the default passivity (206-row reports and transcripts still byte identical to f3), the controlled capture and the Georgia/Lebanon library and Lua replays; then ONE independent review of
this witness before the required model-loader extraction. Full D3 remains incomplete: model loader, entity collision, G1 integration, production authority, the DCS mapping, shipping. Shipping115 is untouched.

---

# Original forest-tree GEOMETRY witness — source PREPARED, UNBUILT, UNRUN, unreviewed (EEDCS-49, D3 characterization before the model-loading extraction)

This completes the partial checkpoint 5154bc7e7 (the feature-only observation hook in `en_cllsn.c`) into a bounded, controlled witness of the ORIGINAL forest-tree geometry branch and the ORIGINAL
model-triangle decoder. Nothing here was built, formatted, tested or run. The measured host tree preparation at `8c452f197449d3a08b8f7bfbad5c34ae652c22c1` (below) is unchanged and not re-reviewed;
no host kernel, loader or Lua API file was touched. Production dispatch is unchanged; the shipping expansion of the new macro is exactly the original call. Task47 shipping115 is untouched.
Task49 and D3 remain In Progress; the model loader, the host geometry branch's agreement with the original, entity collision, G1 integration, production authority and the DCS mapping remain open.

## Why

The measured retail maps give the forest tree object NO collision triangles (`tf-forest geometry=0`), so the original's type-1 geometry branch of `get_line_of_sight_collision_tree` has never run against
them: the 554/600/600 exact replays prove the quad path only. The host kernel carries the geometry branch as the original's text but unexercised, and the model-loading responsibility cannot be moved
before the ORIGINAL's decoder and branch have been measured on a known input.

## What the witness is (all answers are the original code's)

* Input (environment, constants, labelled controlled): a fixture-owned model of 12 points, 3 faces (a quad, a triangle and a pentagon, so fan order and depth order), 2 surfaces whose per-surface point tables are
  not the identity (surface 0 permutes its 7 entries, surface 1 restarts local indexes at 0 with 5), 16-bit points scaled by a bounding box of x/z plus or minus 0.25 and y 0 to 1, and the zero SENTINEL surface the original
  reads after the last face of the last surface. It makes 6 triangles (4-gon: 2, triangle: 1, 5-gon: 3). Decoded coordinates, fan order, scaling and the walk are produced ONLY by the original
  `get_object_3d_collision_object_geometry_triangle`; the fixture never decodes, intersects or predicts.
* Seam: the original reads the model through `objects_3d_data[objects_3d_scene_database[FOREST_TREE_OBJECT].collision_object_index]`. Index 0 (the retail null object, which the accepted C1 patch maps to -1) is NOT used.
  The witness copies the real object table (`total_number_of_raw_3d_objects + 1` entries, content identical, checked), appends its own object after the last real index, points `objects_3d_data` at the copy and
  sets only the forest scene's `collision_object_index` to the appended index, for exactly one probe. Entity queries see an identical table; no real object, tree, file, list or position is changed; the real type-1
  trees of the loaded map are the trees.
* Restoration (every path: normal, the body's own refusals, report too big): the scene index and the `objects_3d_data` pointer are put back FIRST, the copy is freed AFTER. The report then states `data_restored`
  (pointer), `scene_restored` (index), `copy_identical`, the hash of the WHOLE real object table before/after, the hash of the forest scene entry before/after and the scene index before/after; the driver
  requires every one, and a failed restoration is a failed run with the raw report kept (`report_invalid`).
* Observation: `en_cllsn.c`'s two reads of the decoder go through `TREE_QUERY_GEOMETRY_TRIANGLE`; under `EECH_COLLISION_FIXTURE` it calls the fixture wrapper, which calls the original decoder once, with the same
  arguments in the same order, and records by value (calls, yes answers, last answer, depth of the last yes). No extra decoder, terrain or intersection query exists. From it: per stage (0 primary cold, 1 primary warm,
  2 nearer-check cold, 3 nearer-check warm) the queries made, those that ENTERED the branch, those that HIT through it (the first successful triangle returns at once), those that hit through the quad path although geometry
  was present, and those that walked a tree's triangles to the end; and for stage 0 which triangle depth hit.
* One-shot arming: `engine:collision_geometry_arm(true)` (new, feature-only, mirroring `collision_trace_arm`) applies to the NEXT `collision_probe` and is consumed by it whatever its outcome, so no disarm path exists.
  The driver gets `geometry_witness=yes` (requires `probe=yes` and `trace_out`; anything else is refused before booting).

## Default (witness off) is unchanged

The unarmed probe is the original probe body byte for byte (it was only renamed and wrapped): same report rows, same transcript rows and bytes. The new row, fact row and per-query fields exist only when armed. In
the driver, without `geometry_witness=yes` a `col-geometry-witness` row, a `tf-witness` row or a `g_*` field is an UNKNOWN row/key and is refused, so the reviewed f3 default reports (205 ordinary values, 206 trace-report
rows) must still compare byte for byte; Root's native default run is the proof, not this statement.

## New report/transcript shape (checks, not physical answers)

* Report row `col-geometry-witness`: `label=controlled_forest_geometry source=fixture_owned_mesh installed restored data_restored scene_restored copy_identical objects_count scene_index_before scene_index_after
  objects_before objects_after entry_before entry_after mesh_points=12 mesh_faces=3 mesh_surfaces=2 mesh_triangles=6 mesh_fold` then `q0 b0 g0 o0 e0 ... q3 b3 g3 o3 e3` and `d0..d7 dbeyond`.
* Transcript: `trace-start` carries `geometry_witness=controlled_forest_geometry`; a `tf-witness` facts row; `tf-forest geometry=1 triangles=6` with 6 `tf-triangle` rows (the ORIGINAL's decoded output); every `tq` row gains
  `g_calls g_true g_last g_depth`. Everything else (tf-map, tf-sample, caller bits, ordered terrain lookups, final hit/point/normal bits) is the existing format, so `tests/tree_conformance.rs` and
  `lua/tree-collision-replay.lua` replay the SAME transcript (the Lua replay now accepts recorded triangles; it still refuses a capped, miscounted or out-of-order list).
* Driver checks: exactly the labels/mesh constants above; `restored` and the four identity elements; `tf-forest geometry=1`, `triangles=6`; per query `g_true <= g_calls`, a never-answered branch has no last answer or depth,
  a last yes is a hit, `g_depth < 6`; and the report's per-stage `q/b/g/o/e` and depth counts must EQUAL the same counts recomputed from the transcript's per-query facts.
* Named coverage gaps (never a pass, `require_coverage=yes` fails on them): `geometry_branch_never_entered`, `no_geometry_hit`, `no_geometry_hit_after_the_first_triangle`, `geometry_walk_never_exhausted`,
  `no_geometry_query_miss`, `no_geometry_miss_with_a_partial_write`, `geometry_branch_not_seen_with_flags_<ignore_edge>_<skip_top>` for each of the four flag combinations. Event `geometry_witness_coverage` carries the counts.

## Changed files

`aphavoc/.../en_cllsn.c` (already in 5154bc7e7); `eech-campaign/crates/eech-engine-sys/csrc/eech_collision_fixture.c` (model, install/remove, wrapper, accounting, rows, one-shot arm, wrapped entry),
`eech_engine.c` (byte-safe append of `eech_engine_collision_geometry_arm`; its single NUL byte preserved), `eech_engine.h`, `eech-engine-sys/src/lib.rs`, `eech-engine/src/lib.rs`, `eech-dc/src/module.rs`
(`collision_geometry_arm`), `eech-world/lua/collision-characterization.lua`, `lua/tree-collision-replay.lua`, `tests/lua/collision-caller.lua` (stand-in: arm method, witness row, geometry fields, 20 single-defect modes),
`tests/host_callbacks.rs` (3 new tests: acceptance + default isolation, 20 single-defect controls + the unarmed fact control, 3 argument refusals); this file and the collision doc. No host tree kernel/loader/API file changed.

## Root: commands and expected shapes (NOT RUN)

1. Build a fresh clean collision-fixture engine on Windows as for the d5 fixture (`tools/build-windows-collision-fixture.sh`), then verify exports include `eech_engine_collision_geometry_arm`.
2. DEFAULT passivity (witness off): the same run as the reviewed f3 smokes (seed 20261029, `entity_cases=200 tree_cases=200 seconds=0 repeats=2`, `trace_out=<new>`) on Georgia and Lebanon retail roots; the report
   rows and trace bytes must equal the reviewed f3 default ones.
3. WITNESS: `eech-world lua/collision-characterization.lua root=<fresh verified retail root> out=<new.jsonl> scenario=<lebanon_retail|georgia_retail> seed=20261029 entity_cases=200 tree_cases=2000 seconds=0
   repeats=2 geometry_witness=yes trace_out=<new.trace.jsonl>` (add `require_coverage=yes` to make any named gap fatal). Expect: exit 0, `col-geometry-witness` with `installed=restored=data_restored=scene_restored=copy_identical=1`,
   equal before/after hashes and scene index (`-1` expected on the retail DB but not assumed), `mesh_triangles=6`, `tf-forest geometry=1 triangles=6`, `b0>0`, `g0>0`, hits at more than one depth, `e0>0`, misses,
   all four flag combinations seen, a stable transcript across the two repeats, and the world/RNG/record/flags guards as before. These are shape checks; no physical answer is expected here.
4. Cross-check the host (a FINDING about the host kernel's geometry branch, not about the original): `EECH_TREE_REQUIRED=1 EECH_TREE_TRANSCRIPT=<witness trace> EECH_TREE_MAP_DIR=<map terrain dir>
   cargo test -p eech-world --test tree_conformance -- --nocapture` and `eech-world lua/tree-collision-replay.lua transcript=<…> map_dir=<…>`. A mismatch must be reported as such, never normalised.
5. Portable: `cargo test -p eech-world --test host_callbacks collision` (the driver against the stand-in), then format and Clippy; then the ONE independent review of the new witness before any model-loader extraction.

## Remaining unknowns

Whether the retail DB's forest scene index is -1 (expected from C1) or an empty object; whether the 128 KiB report buffer and the 160 KiB facts buffer hold the witness additions (a clean `report_does_not_fit` refusal
if not); whether real type-1 trees fall in the structured target classes (the random cases draw from all trees); how many random cases hit at depth above 0 and miss with partial writes (named gaps otherwise); the exact
hit/miss/point/normal bits (they are the original's and are what the host must later match); `acos`/`sqrt` library agreement between the original build and the host build on those cases.

---

# Host tree loader/kernel preparation — reviewed, exact original-query replay; production still coupled

Measured frozen source `8c452f197449d3a08b8f7bfbad5c34ae652c22c1` passes the
focused Linux and actual Windows checks. Linux format,17 tree units,1 real-host
Lua test and all-host-target Clippy with warnings denied complete0. Windows
build0/all8 exports verified;17 units and1 real-host Lua test pass. All554 Georgia
and600 each Lebanon/repeat original queries match every answer bit and all
1102/1120/1120 ordered terrain lookups through both library and real host Lua.
Library Up/Down/Zero replays also match, with caller x87/SSE control/status and
nonzero exception flags restored. Every pre/post-input check passes; all native
payloads and immutable original transcripts remain unchanged.

The same six protocol and three getter controls reproduce old defects and pass
on this source: sparse geometry refuses, failed ordinary or reentrant begin and
terrain requests cannot leave a query answerable, and healthy controls remain
valid. The one combined review and focused correction follow-up are CLOSED with
no open findings. Final review SHA256:
`4258f1b94e9eeb0821a1dc692190451ee1606454dc2631e148261605d47876ca`.
Safe validation SHA256:
`4687c6a8297188b58c17e72acac3797cc92b4f45a7bd759a73adf238a7c50243`.
Reports are under `eech-campaign/reference/d3-collision-characterization/`.
Original/intermediate failures and the initial C5 review remain immutable.

Ledger14548994v65 readback preserves all prior HTML and all five epic reports
exactly; Jira10860 records closure of preparation. Four closed Windows Cargo
caches were removed only after all8 exports, all3 cached/exported logs and zero
container owners verified. Frozen sources, tested binaries, roots, successes and
failures remain retained. Current8c and shared caches and shipping115 are untouched.
The latest checked shipping prefixes are43.4MB Tacview/28.7MB observations exact;
all six original process identities remain live. Full-pack acceptance is pending.

This is host-component preparation, not production ownership or Task49/D3
acceptance. Campaign queries still run the original DLL service. Next: establish
a controlled ORIGINAL forest-geometry/model-decoder witness before moving the
remaining required model loading/geometry services. Native forest geometry was
absent from these retail captures. Entity collision, G1 integration, public
authority/refusal, DCS mapping/workarounds and shipping remain required. Actual
unmasked-exception entries and induced environment-operation failures are not
claimed exercised. Task47 shipping115 remains live; only partial recording
equality is available. ProductionG2/G3 delivery gates remain. D4/D5 have not started.


# Host tree kernel — f81eb31bc runtime controls PASS; Linux Clippy 101 on two test-only lints; the two edits PREPARED, UNBUILT, UNRUN (EEDCS-49, D3 preparation)

Measured by Root on f81eb31bc454492c57d64dbd8cf0c0105041b1e5 (retained, not re-derived here): Windows build 0 with 8 exports; Linux 17 tree unit tests, 1 real-host Lua transport test and fmt 0; Root's same 6 protocol controls
and same 3 re-entrant getter controls all PASS on the new host (an erroring or malformed getter can no longer continue a nested query; the healthy getter is unchanged); the 3-map native original replay (controller 42087)
was still running, Georgia already 0 on both paths with every original bit exact, the full results being Root's to collect. The reviewer inspected only the f81 delta and has no further correction finding; closure awaits the
actual checks. There is no overall acceptance yet. Linux Clippy now gets past the previous three lints and fails with exactly three errors that are two tiny edits in `tests/tree_conformance.rs` only
(raw `R:/d3-evidence/tree-host-final-linux-f81eb31bc-20261029.log`, Linux invocation 101).

Prepared (source only; no host library, `main.rs`, C, FPU, physics, loader or protocol change; no lint allow or suppression, no dependency or toolchain change):
1. `doc_lazy_continuation` (lines 17 and 18): a blank `//!` line now separates the bullet list from the paragraph "The loader is checked ..."; the prose is unchanged.
2. `manual_is_multiple_of` (line 54): `text.len() % 2 != 0` is now `!text.len().is_multiple_of(2)`, the same odd-length hex refusal; parsing and expected bits are unchanged. (`is_multiple_of` on unsigned integers needs a
   recent stable Rust, as Clippy's own suggestion implies; Root's builds show whether every toolchain has it.)

Root will format and freeze the new source, run `cargo clippy --keep-going` to expose any remaining independent target issue, the appropriate checks, and then the same reviewer closes only these edits and the evidence. The allocation
wording that Root qualified is kept. Forest geometry (model loader and native witness), entity collision, G1 integration, production authority, the DCS mapping or documented workaround (live DCS execution is M6) and shipping
remain D3 requirements. This does not complete Task49, production or D3.

---

# Host tree kernel — reviewer's remaining pending-query seam (re-entrant `__index` during `begin`): measured RED, cleanup PREPARED, UNBUILT, UNRUN (EEDCS-49, D3 preparation)

## Measured state of the 684fb6981 source (retained; this does NOT cover the new code)

684fb6981 already measured: Windows 17/17 units, 1/1 real-host Lua transport, ALL 554/600/600 original queries with 1102/1120/1120 lookups exact through the library (default and Up/Down/Zero callers with non-zero
exception status restored) AND Lua, all 3 pre/post manifests 0 and all 8 payloads unchanged; Root's unchanged protocol script passing all 6 cases. Linux 17/17 units, Lua transport and fmt passed and Clippy exit 101 on three
style lints; those three fixes (as_chunks, expect_err, a local `Case` alias) are PREPARED in the working tree, UNCOMMITTED, UNBUILT, UNRUN and are preserved here.

## The seam (found by the same reviewer's closure of the five pending-query/protocol fixes only)

`begin` abandoned the OLD pending query before it read its Lua fields, but reading a field can run Lua (`__index`). A getter can re-enter `host.tree_collision` with a VALID begin and then raise an error or return a
malformed `old_position`: the OUTER begin is refused, but the NESTED query it created survives and a later `terrain` continues it. Root's actual RED on the frozen 684 host (SHA256
706f4b39b005381a447421a03531297f425c0c2162938decded246db3615fd43; private `R:/d3-evidence/tree-host-reentrant-red-684fb6981-20261029/controller.json`; script
`R:/d3-evidence/tree-host-reentrant-begin-control-20261029.lua` SHA256 2bb59f7553686b0edc9a3d253d6c7b05866effc04e8b1055aff46a193b058244, to be replayed UNCHANGED against the next host): each real getter ran
exactly once; `getter_error` and `getter_malformed` each gave outer accepted=false but continued=true / `need_terrain` (ghost continuation); `healthy_reentry` gave outer accepted=true, continued=true. All 3 diagnostic
native processes exited 0. These observe the bug; they are not pass claims.

## The cleanup (source only; no formula, FPU, loader, physical or protocol change)

`src/main.rs`, `host.tree_collision{mode='begin'}`: the whole argument parse AND the call into the kernel now run inside one small `Result` scope; EVERY `Err` leaving that scope abandons whatever is pending AT THAT
MOMENT (including a nested query the getter started) before the error is returned. The earlier pre-parse abandon stays (it still closes the old query on an ordinary malformed request). No `RefCell` borrow is held
while Lua runs: each borrow is taken for one call and dropped, so a re-entering getter never meets a held borrow. There is no recursion ban and no transaction framework, so the healthy control is unchanged: a getter
that starts a nested query and then returns a valid vector still yields an accepted outer begin that replaces the nested query and continues. `terrain` already abandoned after a parse error and still does.

Real-host controls added to `tests/tree_collision_host.rs` (all old protocol, tiny / negative-zero and geometry checks kept): a metatable getter that starts a valid nested begin and then (a) errors, (b) returns a 2-element
`old_position`: the outer begin is refused, the getter ran exactly once and the next `terrain` is refused with "no query open"; (c) returns a valid vector: the outer begin is accepted and the request continues to the new
end point.

## Root's next step (expected, NOT RUN)

Format; freeze the combined source (three lint fixes + this cleanup); Linux fmt and Clippy and the unit tests; Windows build, the 17 contracts plus the new getter control, the library and Lua replays of all 554/600/600
original queries, Root's protocol script and the re-entrant script old -> new (the RED must become: both bad cases refused AND not continued, the healthy case unchanged); then the same reviewer checks only the new fixes
and evidence. Original oracles, baselines and tolerances are untouched. Geometry (model loader, native witness), entity collision, G1 integration, production authority, the DCS mapping or documented workaround (live DCS
execution is M6), shipping and the rest of D3 remain open; Task49, D3 and the extraction are not complete.

---

# Host tree kernel — 684fb6981 measured GREEN on Windows and protocol controls; Linux Clippy 101 on three style lints; the three fixes PREPARED, UNBUILT, UNRUN (EEDCS-49, D3 preparation)

## Actual 684fb6981 results (Root-measured; retained)

* Windows: build 0, all 8 exports verified; native controller 23989 terminal, 0 collected: 17/17 tree unit tests, 1/1 real-host Lua transport test, and ALL 554 Georgia / 600 Lebanon / 600 repeat original queries with
  1102 / 1120 / 1120 recorded lookups exact through the library AND the Lua path, every exit 0, no skips. The library replays in Up, Down and Zero were bit-exact with the caller's x87/SSE control and status words and its
  NON-ZERO exception flags restored; the default replay is exact too. All 3 pre/post manifests 0, all 8 payloads and every original transcript hash unchanged. Raw
  `R:/d3-evidence/tree-host-boundary-684fb6981-native-20261029/controller.json` and logs.
* Root's UNCHANGED protocol script (SHA256 fa77370fbd1cfd20fe1bf43378474b775deea189a85978813be6e361fae149de) now passes all 6 cases: valid dense list and healthy pending unchanged; sparse and holey lists refused; a
  pending request cannot continue after a bad begin or a bad terrain answer. Green controller `R:/d3-evidence/tree-host-boundary-green-684fb6981-20261029/controller.json`; the d726 RED is retained.
* Linux: 17/17 units, 1/1 real-host Lua transport and format PASS; Clippy exit 101 on exactly three style lints (below). Raw `R:/d3-evidence/tree-host-boundary-linux-684fb6981-20261029.log`. Linux is NOT overall passed.
  No review of 684fb6981 has been performed yet; the same reviewer will check only the changed fixes and this evidence.

## The three Clippy fixes (source only; no allow/suppression, no dependency or toolchain change; no physical, FPU or protocol change)

1. `chunks_exact_to_as_chunks` in the loader: the data file's whole records are now taken with `data_file.as_chunks::<RECORD_BYTES>().0.iter()` and decoded with `TreeRecord::from_bytes(*chunk)`: the same whole 8-byte
   records in the same order, no `try_into`/`expect`; the bytes after the last whole record are the remainder and are still counted as the census's trailing bytes (computed from the length, as before).
2. `err_expect` in the map-metadata test: `result.err().expect("accepted")` became `result.expect_err("accepted")` (TreeMap already implements Debug, so no new Debug implementation was needed). It still panics if the
   forged map is accepted and still asserts the diagnostic text. The other `.err()` calls are on `TreeScene` results, which have no Debug and were not flagged.
3. `type_complexity` in the rounding test: a local `type Case = (Arc<TreeScene>, [f32; 3], [f32; 3], (bool, bool));` replaces the long tuple type. Nothing else changed.

Everything else of 684fb6981 is kept: the 17 unit tests, the thread-affinity compile-time contract, the map revalidation, the complete caller state (x87 environment and MXCSR) restoration, the dense-geometry and fail-closed
protocol checks, and all expected data, goldens and tolerances. Note: `as_chunks` needs a recent stable Rust (Clippy's own suggestion implies the Linux toolchain has it); Root's Windows rebuild shows whether that
toolchain does as well. Root will freeze a new source, rerun Linux fmt/Clippy and the Windows proof (original queries, Lua, protocol controls), then the same reviewer closes only these fixes.

Still D3 requirements, not done: forest geometry / model loader and a native original-geometry witness, entity collision, G1 terrain integration, production authority, the DCS mapping or documented workaround (live DCS
execution is M6), and shipping. Task47 shipping115 is untouched. This does not complete Task49, D3 or the extraction.

---

# SUPERSEDED (kept for the record): the review-batch section for 684fb6981 as prepared, before its measurement

# Host tree kernel — d72610f85 measured: default library and Lua replays bit-exact, boundary checks FAILED; independent-review batch PREPARED, UNBUILT, UNRUN (EEDCS-49, D3 preparation)

## What the d72610f85 native measurements actually showed (terminal; NOT an acceptance and NOT "overall passed")

* Windows host and test build 0, all 8 exports verified. Linux: 15/15 tree unit tests passed; the real-host Lua transport test FAILED (exit 101) at the newly reached negative-zero assertion; Clippy not reached.
  Windows: 14 unit tests passed and 1 FAILED (the new environment-restoration test); the Lua transport test failed 101.
* ALL captured original queries (554 Georgia / 600 Lebanon / 600 repeat; 1102 / 1120 / 1120 recorded terrain replies) replay through the default library path with ZERO mismatches of lookups and ZERO answer-bit
  mismatches, and each full real-host Lua replay exits 0 with zero mismatches. The rounding correction therefore made the default-caller replays bit-exact. This is evidence for those replays only.
* The library replay under a caller in Up also had zero bit mismatches, and then FAILED the environment assertion: before `63/63`, SSE rounding 2, x87 Up, flags 0; after `63/63`, SSE rounding 3, x87 Up, flags 32.
  The Down and Zero replays were not reached. Controller 11796, terminal, 1 collected. Raw `R:/d3-evidence/tree-host-rounding-d72610f85-native-20261029/controller.json` and case logs. All 3 pre/post input
  manifests 0, all 8 payloads and every original transcript hash unchanged.
* Negative zero diagnosed by Root without source changes (`R:/d3-evidence/tree-host-negative-zero-d72610f85-diagnostic-20261029/controller.json`): the test's `local negative_zero = 0 * -1` is folded by this Lua to a
  POSITIVE zero (reciprocal +inf, `%.17g` "0"). Constructing it as `-1 / math.huge` gives a genuine negative zero (reciprocal -inf, "-0") and the whole synthetic transport script then exits 0 with the tiny expected bits.
* Root's protocol RED controls against the same host (`R:/d3-evidence/tree-host-boundary-red-d72610f85-20261029/controller.json`, script
  `R:/d3-evidence/tree-host-boundary-controls-20261029.lua` SHA256 `fa77370fbd1cfd20fe1bf43378474b775deea189a85978813be6e361fae149de`, to be replayed UNCHANGED against the next host): a dense valid geometry list was
  accepted, and so were sparse `{[2]=tri}` and holey `{[1]=tri,[3]=tri}` lists (bad acceptances); a pending request continued after a malformed `begin` was refused and after a non-finite `terrain` answer was refused
  (ghost continuations). A healthy pending request continues. These are measured defects, not pass claims.

## The single independent review's batch, and what the prepared source does (source only; arithmetic untouched — the reviewer found NO discrepancy between the C and the original arithmetic)

1. P2 map invariant. `TreeMap`'s public fields let a forged value bypass `new()` (for example min_x = max_x = f32::MAX with 2 sectors could reach an undefined C float-to-int conversion; the reviewer did not execute that malformed native query). The invariant is
   now REVALIDATED: `TreeMap::validate()` (the checks of `new`, which now calls it) is called first by EVERY public scene constructor: `without_trees`, `from_bytes`, and `load` including its absent-file path.
   Valid metadata and all math unchanged. Unit test `a_tree_map_built_by_hand_is_revalidated_by_every_scene_constructor` feeds nine forged literals to all four paths (none is executed in C) and checks the valid map
   and an empty triangle list; an empty `Triangles(vec![])` is itself refused (it is not a statement about the geometry).
2. P2 complete caller floating-point state. On the Windows toolchain `fesetenv` did not give back MXCSR (the SSE rounding stayed round-toward-zero and the inexact flag stayed raised). The C entry now saves the x87
   environment (`fegetenv`) AND the whole MXCSR (`stmxcsr`: control and status bits), sets round-toward-zero in both units and reads both back, runs the original arithmetic, and on EVERY exit restores with
   `fesetenv` then `ldmxcsr` and reads MXCSR back; a restore (or set) that does not read back as expected is `TREE_ENVIRONMENT`: the answer is withheld and the query closed. The exception masks are saved/restored
   exactly and never weakened; unmasked callers are still refused up front. `ScopedRounding` uses the same save/restore. `FpState` now also carries the raw x87 control word and raw MXCSR. The library and the
   real-query tests now enter from callers in each rounding mode with a NON-CLEAR exception status (inexact and divide-by-zero raised before the call; asserted non-zero) and compare masks, rounding, raw control
   words, raw MXCSR and raised flags before and after. An actual unmasked-exception entry and an actual `fesetround`/`fesetenv`/`ldmxcsr` failure are NOT exercised and not claimed.
3. P2 thread affinity. `ScopedRounding` carries a `PhantomData<*const ()>`, so it is neither `Send` nor `Sync`. A compile-time contract in the unit tests (the `AmbiguousIfSend`/`AmbiguousIfSync` technique of the
   static_assertions crate, no new dependency) fails the test build if either bound is ever implemented.
4. P2 dense geometry. `host.tree_collision{mode='load'}` requires `forest_geometry` to be `"absent"` or a DENSE ordered list (`dense_length`: every key an integer 1..n, no hole or extra key) of triangles that are
   each a dense list of exactly nine numbers; a sparse, holey, empty or malformed list is refused BEFORE any geometry is discarded or absence is assumed, and a refused load leaves the loaded scene untouched. Host
   script controls: `{[2]=tri}`, `{[1]=tri,[3]=tri}`, `{}`, an eight-number triangle, a holey triangle and a non-table item are refused; `{tri, tri}` and `"absent"` are accepted. No physical oracle was added and
   native geometry is not claimed validated.
5. P2 pending-query cleanup. The Lua `begin` abandons any pending query BEFORE parsing its arguments (a begin replaces the pending query anyway), and a `terrain` whose elevation cannot be read abandons it; in Rust a
   non-finite `terrain` answer closes the pending query. Contract: a refused begin or terrain answer ends the pending query, so a later valid `terrain` is refused with "no query open"; a refused `load` changes
   neither the scene nor a pending query. The real-host script covers pending -> malformed begin, pending -> non-boolean flag, pending -> non-finite elevation, pending -> non-numeric elevation, and the healthy
   pending control that continues. Root's unchanged protocol script above is the second check.
6. P3 negative zero. The synthetic script constructs it as `-1 / math.huge` and asserts `1 / negative_zero == -math.huge`; the expected `80000000` and the numeric reader are unchanged.
7. P3 loader wording. The module documentation no longer says "before anything is allocated": it says the two files are read whole, each size is first checked against `MAX_SECTOR_FILE_BYTES` and
   `MAX_DATA_FILE_BYTES` (a bound on what `fs::read` allocates; retail Georgia is 226 KB and 6.8 MB), the forest triangles are flattened as given, and every length, multiplication and span is checked in `u64`
   with sector-table byte sizes checked before allocating that table and each sector's record span checked before tree-record decoding or indexing. No format change.

## Changed files in this batch (under `eech-campaign/crates/eech-world/`)

`csrc/tree_step.c`, `src/trees.rs`, `src/main.rs`, `tests/tree_collision_host.rs`, `tests/tree_conformance.rs`, plus this file and `eech-campaign/docs/m5-collision-characterization.md`.

## Root's next step (expected, NOT RUN)

Format; freeze a NEW source; build Windows and Linux; `cargo test -p eech-world --lib trees`, `--test tree_collision_host`, then for EACH of Georgia / Lebanon / Lebanon-repeat the library replay
(`--test tree_conformance`, `EECH_TREE_REQUIRED=1`, which repeats in Up / Down / Zero with a non-clear caller status) and `lua/tree-collision-replay.lua`; replay Root's unchanged protocol controls old -> new;
then the SAME reviewer checks only the new fixes and evidence. Nothing here claims D3, Task49 or extraction complete: geometry (model loader, native witness), entity collision, G1 integration, production authority,
shipping, and the DCS mapping or documented workaround (a D3 requirement; live DCS execution is M6) remain open. Task47 shipping115 is untouched.

---

# SUPERSEDED (kept for the record): rounding-environment correction d72610f85 — see the measurements above

# Host tree kernel — first native checks FAILED (real RED); rounding-environment correction PREPARED, UNBUILT, UNRUN (EEDCS-49, D3 preparation)

## What the first native checks of source 0db20ab3a46582110bd7c4706c26edaaa9335478 actually showed (retained; terminal; NOT passed)

* Windows build 0, all 8 exported manifest entries verified; frozen binary `…/eech-campaign/target/tree-host/0db20ab3a`; controller `tree-host-first-windows-0db20ab3a-20261029-completion.json`.
* Linux: 14/14 tree library unit tests pass; the real-host Lua transport test FAILED at its assertion `both outside: no lookups at all` (Linux invocation exit 101; Clippy not reached). Windows: 14/14 tree
  library units pass; the same Lua transport test FAILED. Raw log `tree-host-first-linux-0db20ab3a-20261029.log`.
* Windows replay of ALL 554 Georgia / 600 Lebanon / 600 repeat captured queries with their 1102 / 1120 / 1120 recorded terrain replies, through BOTH the library path (exit 101) and the host Lua path
  (exit 1) on each scenario: the loader, census and native samples agreed, but OUTPUT BITS DIFFERED. The retained diagnostic list is bounded (its 41 entries are not the number of failing queries).
  First actual tq 0 Georgia point `45e0dec9,41ed10b2,48584704` against the original `45e0dec9,41ed10b2,48584703`; other retained examples differ by one bit in x, y or z. Raw per-case logs and
  controller JSON under `R:/d3-evidence/tree-host-first-0db20ab3a-native-20261029`; all 3 pre/post manifests 0, all 8 payloads and every original transcript hash unchanged. The raw expected data is
  unchanged and is the oracle.

## Diagnosis (two separate defects; neither is a change to the original's physics)

1. MISSING ENVIRONMENT BOUNDARY (the numerical failure). The original runs under round-toward-zero: `eech_engine.c` ENTER saves the caller's `fenv` and sets `FE_TOWARDZERO`, LEAVE and the fatal path restore
   it, the collision fixture refuses captures unless `fegetround() == FE_TOWARDZERO`, and the original `get_terrain_3d_sector` sets it again. The first source read and required only the exception masks
   (63/63) and explicitly left rounding alone, so it executed under the caller's ordinary round-to-nearest, which is not equivalent. Masks of 63/63 do not establish rounding.
2. WRONG SYNTHETIC EXPECTATION in `tests/tree_collision_host.rs` (a test defect, not a kernel defect). The case `{-1,10,1024} -> {5000,10,1024}` was expected to miss because "both outside: no lookups". The
   segment runs along z = 1024 at y = 10 and actually CROSSES the tree at the middle of sector (0,0). An end point outside the map only avoids the terrain lookup for that end (its radial altitude is
   taken as 0 / the old one's); it never makes the collision FALSE, and a `done` answer with no `need_terrain` first simply means no lookup was needed. The test now asserts the real contract: that
   segment gives `done` immediately with a HIT at the hand-derived point (1024, 10, 1024), normal +x; and a both-outside segment that crosses no tree (z = 3000) gives `done`, a miss, and the caller's own
   point and normal. The original behaviour was not changed to suit the old expectation. The assertions after that point in the script had not been reached by the failed run; they were re-read against the
   source and the kernel text (ordering, refusals, exact tiny / negative-zero reads) and no further defect was found by reading, which is not a run.

## The correction (source only; math, loader, query and scan order, partial writes, tie rules, triangles and recorded data untouched)

`csrc/tree_step.c`: the former `eech_tree_begin` / `eech_tree_terrain` bodies are now private `noinline` functions, and each public entry wraps them (the pattern of the host-private terrain_ref.c of G1, not imported):
1. read the x87 and SSE exception masks and REQUIRE 63 and 63 BEFORE any arithmetic; on failure return `TREE_MASKS`, compute nothing, change nothing, and the query is closed. The kernel never masks exceptions.
2. `fegetenv` the caller's environment; `fesetround(FE_TOWARDZERO)`; either failing is `TREE_ENVIRONMENT` (the caller's environment is put back if it was touched).
3. run the original's arithmetic (begin: the in-map tests, radial altitudes, scan; terrain: the radial altitude and scan).
4. `fesetenv` the caller's environment on EVERY exit: lookup request, final answer, scene fault, masks refusal path (nothing set), environment failure. If the restore itself fails the answer is withheld,
   the query is closed and the call returns `TREE_ENVIRONMENT`. An out-of-sequence `terrain` returns before touching the environment.
No guard state exists between calls, so none is held across the Lua/campaign callbacks that supply elevations. The bodies are not inlined into the entries so no arithmetic is scheduled outside the guard. The file is
deliberately NOT built with `-frounding-math`: the original's compile-time constant folds (e.g. `19/127.0f`, `PI2 - 0.01`) happened at the default rounding; the host must fold the same constants the same way.
Whether this reproduces the original is established only by Root's replay of the captured queries, not by reasoning.
`src/trees.rs`: the Rust-side mask check is removed (the C entry is authoritative); codes -3 / -4 map to clear errors; new public environment helpers `rounding()`, `Rounding`, `exception_flags()`,
`clear_exception_flags()` and the RAII `ScopedRounding` (saves and restores the whole caller environment) exist only to enter the kernel from a caller in another mode in tests and reports.

## New boundary checks (all source only)

* Library unit test `the_answer_does_not_depend_on_the_callers_rounding_and_the_callers_environment_is_restored`: five hand-derived-class cases (plane hit with division and normalisation, top-quad hit, a miss,
  a both-outside segment that still hits with no lookup, a geometry-branch triangle hit) entered from a caller in Nearest, Up, Down and Zero; answers and lookup requests compared as bits across all four,
  the caller's rounding checked after EVERY call into the kernel (including after a lookup request), and masks, rounding and raised exception flags compared before and after.
* `tests/tree_conformance.rs` (the real queries): the unchanged full replay, then the same replay against the same recorded bits from callers in Up, Down and Zero, with the environment compared before and after
  each; no assertion relaxed and no oracle touched.
* The real-host Lua path starts in the host process's own caller rounding (it cannot change it) and is the compatibility path for that mode only; other modes are covered by the library path above. Preserved: the
  tiny / negative-zero transport checks and the real malformed, refusal and out-of-sequence guards.
* NOT covered: entry with exceptions UNMASKED (the masks refusal is a source guard only; no test enables exceptions) and an actual `fesetround` / `fesetenv` failure (not inducible safely). Neither is claimed.

## Root's next step (expected, NOT RUN)

Freeze a NEW source; build Linux and Windows; run `cargo test -p eech-world --lib trees`, `--test tree_collision_host`, and for EACH of Georgia / Lebanon / Lebanon-repeat both the library replay
(`--test tree_conformance`, with `EECH_TREE_REQUIRED=1`, which now also repeats under Up / Down / Zero) and `lua/tree-collision-replay.lua`; then format, Clippy, and the ONE independent review of this component.
Any remaining difference is investigated from the raw logs; nothing is normalised. Production dispatch is unchanged; geometry (model loader and native geometry witness), entity collision, G1 integration, production
authority and shipping stay open. Task47 shipping115 is untouched.

---

# SUPERSEDED (kept for the record): first source 0db20ab3a — PREPARED, UNBUILT, UNRUN

# Host tree loader and tree segment kernel — source PREPARED, UNBUILT, UNRUN (EEDCS-49, D3 preparation; not Task49/D3 completion)

This completes the preserved partial work (checkpoint f0bd165c9) into the first host-owned physical component, as source only. Nothing was compiled, formatted, tested or run by the
implementation writer; every statement below describes what the source is meant to do and what Root must run, not a result. The campaign DLL's tree queries still run the ORIGINAL solver; no
production dispatch, accepted baseline, tolerance, capture, oracle or original physical file changed. Production G2/G3 cutover stays gated on the preceding delivery and verified integration;
Task47 frozen shipping115 is untouched; D4/D5 have not started; whole D3 remains open.

## Changed files (all under `eech-campaign/crates/eech-world/`)

* `csrc/tree_step.c` — NEW. The host-private C kernel: EECH's own `get_line_of_sight_collision_tree` (en_cllsn.c) with `get_3d_line_quad_intersection` and
  `get_3d_line_triangle_intersection` (maths/vector.c), expression for expression, as a resumable NeedTerrain/Done state machine. Built with the existing `eech_steps` flags and `eech_arith.h`.
* `src/trees.rs` — NEW. Checked ownership and API: `TreeRecord` (explicit 8-byte decode), `TreeMap` (checked terrain metadata), `ForestGeometry` (must be stated), `TreeScene` (loader, immutable owned
  storage, census), `TreeCollision` (one open query, `begin`/`terrain`/`abandon`), `fp_state` (read-only), plus focused unit tests with hand-derived expectations.
* `src/lib.rs` (`pub mod trees`), `build.rs` (`tree_step.c` in the `cc` file list and `rerun-if-changed`), `src/main.rs` (`host.tree_collision`, the reference host's Lua transport).
* `tests/tree_conformance.rs` — NEW. Replays EVERY captured original query through the library path (needs the retail inputs below; says plainly "NOT RUN" without them).
* `tests/tree_collision_host.rs` — NEW. Starts the real `eech-world` host on a Lua script against a tiny map written by the test (ordered lookups, refusals, out-of-sequence, exact number reads).
* `lua/tree-collision-replay.lua` — NEW. The same real-query replay through the host's Lua transport (Lua -> host -> C kernel -> Lua), with the host's bit echoes compared as strings.
* `STATUS.md`, `docs/m5-collision-characterization.md` — this record.

## Component ownership (what moved under the host and what did not)

Host-owned in this source: the tree_pos.sec/.dat file reading and checking, the decoded tree records and sector tables, the map metadata the tree query reads (supplied explicitly, see below), the
optional forest-tree triangles, and the whole tree segment calculation including its normals, intercepts, flags, nearest-tree choice and the early exits. Campaign-owned and unchanged: every decision
about when and why a tree query is made. Still the CAMPAIGN DLL's, supplied as values through NeedTerrain: the two terrain elevations the original reads (one at the old end point, then one at the
new one, only for end points inside the map). The kernel makes no terrain call of its own and no extra or reordered lookup, so the shared terrain point hint/record effects of the original stay
visible for the later G1 composition. The surrounding ENTITY collision (the UPDATED/delta-time coupling and the `&=` getter) is the next component and is not touched.

## Lifetime model

`TreeScene` owns every table (records, sector `count`/`first` pairs, flat triangle floats) and is immutable after construction. `TreeCollision` holds an `Arc<TreeScene>` and one opaque query block.
Each kernel call is handed a descriptor built from that `Arc` for the duration of the call; the C side retains no pointer between calls, so a pending query can never outlive its storage and
dropping the holder drops the query. Raw pointers exist only inside the module's `unsafe` calls; the public Rust API and the Lua contract carry values only. The Rust and C layouts are compared at
`TreeCollision::new` (record 10 bytes, sector 8, scene descriptor size) and a mismatch is an error.

## Loader behaviour

Files are found by listing the map directory and matching `tree_pos.sec`/`tree_pos.dat` ignoring ASCII case, then opening the entry's OWN spelling (Georgia/map3 lower case, Lebanon/map5 upper case);
nothing is renamed and case-insensitivity is never assumed; two spellings of one name are refused. Both files absent (or one) is a known "no trees loaded" in the census; a header that differs from the
terrain's sector counts is a known "no trees loaded" (the original ditches the trees there) and the census records the header. Anything else wrong (file shorter than its header, negative count or
offset, a sector reaching past the data file, offset+count overflow) is an error, with every length computed in 64 bits before allocation. Trailing bytes are counted in the census and not interpreted.
Records are decoded explicitly: x i16 LE at 0, z i16 LE at 2, y u16 LE at 4, colour at 6, last byte height (low 7 bits) and type (high bit); SEC is LE i32 width and height then row-major (z, x)
pairs of count and record offset. The original's scan order is kept (x outer, z inner, each sector's trees in file order).

## Map metadata (G1 dependency)

This branch predates the unaccepted G1 terrain host module, so `TreeMap` (min/max x and z, terrain sector counts, terrain minimum height and scaled height difference) is an EXPLICIT constructor:
the conformance test and replay script feed the facts the original recorded (`tf-map`) as controlled inputs. Production will reuse the accepted host terrain module's metadata; it must not borrow
DLL pointers. No second world, no core-owned scene API and no terrain loader were added.

## Geometry (remaining D3 dependencies, explicit)

The kernel includes the original's geometry branch (type-1 tree and a forest-tree collision triangle at depth 0: the identity-matrix transform, the height/position scaling, the do/while over the
triangles, and an immediate return on the first triangle hit). `ForestGeometry` has no default: it is `KnownAbsent` (the original's triangle function returns FALSE at depth 0, the retail facts) or
the decoded triangles. Retail Georgia and Lebanon record geometry 0 / 0 triangles, so ONLY the quad path is numerically compared with the original by the retail data; the geometry branch is the
original's text carried over, unrun against any native witness, and its hand-derived unit test is a worked example, not a conformance claim. REMAINING for D3: the model geometry LOADER (decoding the
forest-tree collision object's faces to triangles, 3dobjdb.c) and a controlled native geometry witness before complete query delivery.

## Floating point (SUPERSEDED by the section at the top of this file)

The first source only READ the exception masks and ran under the caller's rounding; that was the defect the first native replay exposed. The current source reads and requires the masks (63 / 63), saves the
caller's environment, runs the original's arithmetic under FE_TOWARDZERO and restores the caller's environment on every exit. Division by `dx`/`dz` is unguarded and IEEE results are kept; the host converts
a position to a sector with a C truncation, as the patched 64-bit engine does.

## Declared limits and numerical unknowns

* End points must be finite and terrain elevations finite; the original accepts NaN/infinity and the host refuses them (no captured query contains one).
* `acos` and `sqrt` come from the host C library; whether they match the original build's last bit near the 2*pi-0.01 threshold is verified only through the conformance replay, never assumed.
* The sector scan is bounds-checked against the table (a second line of defence); the original reads without a check. Consistent metadata makes it unreachable.
* A DCS tree-raycast equivalent is NOT designed or claimed here. The needed DCS mapping or documented workaround remains a D3 acceptance requirement; live DCS execution belongs to M6.
* The conformance claim exists only after Root runs the replay below and every one of the 554/600/600 queries and 1102/1120/1120 lookups matches.

## What Root runs (expected, NOT YET RUN)

Retail inputs (private, unmodified; raw reviewed captures in `R:/d3-evidence/tree-input-f3fd93dfb-smokes-20261029/<scenario>/tree-transcript.jsonl`, capture review SHA d71fcaea…2d8 at f3fd93dfb,
Git LF copy SHA 5c48a1cd…c22; map trees in `R:/d3-evidence/tree-input-roots-f3fd93dfb-20261029/`):

| scenario | EECH_TREE_TRANSCRIPT | EECH_TREE_MAP_DIR | expected counts |
| --- | --- | --- | --- |
| georgia | `…/georgia/tree-transcript.jsonl` | `…/tree-input-roots-f3fd93dfb-20261029/georgia/common/maps/map3/terrain` | 554 queries, 1102 lookups, tree_total 848187, sectors 8689 |
| lebanon | `…/lebanon/tree-transcript.jsonl` | `…/lebanon/common/maps/map5/terrain` | 600 queries, 1120 lookups, tree_total 213955, sectors 1338 |
| lebanon-repeat | `…/lebanon-repeat/tree-transcript.jsonl` | `…/lebanon-repeat/common/maps/map5/terrain` | 600 queries, 1120 lookups |

1. Library path, per scenario (set `EECH_TREE_REQUIRED=1`; a run without the two paths prints `TREE_CONFORMANCE NOT RUN` and proves nothing):
   `cargo test -p eech-world --test tree_conformance -- --nocapture` — prints `TREE_CONFORMANCE … mismatches=0` and passes only if every query's lookups and answer bits match.
2. Lua path, per scenario: `eech-world lua/tree-collision-replay.lua transcript=<…> map_dir=<…>` — prints `TREE_LUA_REPLAY_LOADED …` then `TREE_LUA_REPLAY_DONE queries=… mismatches=0`, or errors.
3. Synthetic and unit contracts: `cargo test -p eech-world --lib trees` and `cargo test -p eech-world --test tree_collision_host` (set `EECH_WORLD_BIN` to run a separately built host).
4. Then format, host Clippy, the existing eech-world suite (nothing else was edited), and the one independent review of this new component.

Any difference is investigated as a regression, an unsupported case or a harness defect; the transcripts and expectations are never normalised, re-recorded or loosened.

---

The earlier "User-requested Git checkpoint" block (partial files preserved at f0bd165c9, UNBUILT/UNRUN) is superseded by the section above; the reviewed capture evidence below is unchanged.

# Current tree-input capture — reviewed and measured; host kernel preparation next

Capture review is CLOSED with no open findings at f3fd93dfb2ad81b6cbe4d89174953376efd07611.
Review SHA256 d71fcaea939a6207b2267a45ad7183d05eae74a6621e0ff329e3ac71a5d82fd8.
The three findings (page arithmetic, capped facts, paired-query/lookup coherence)
are resolved. Clean Windows fixture build0/all43 exports; Linux and actualWindows
15/15 driver/report/transcript/I-O tests, format and host Clippy0. Real public
Lua/module/C paging passes normal108-record/last pages, six EOF/extreme-offset
empty pages and six invalid-argument refusals, with pre/post inputs0 and no skips.

Three corrected retail trace probes complete0/pre/post0. Every old205 report value
matches541; all six complete206-row reports/transcripts match the initial41 bytes.
Georgia captures554 queries/1102 actual terrain lookups; Lebanon600/1120. Warm
pairs, lookup coordinates/order, native8-byte packing and separate repeat bytes
are independently verified. No raw sample or oracle changed. Forest geometry is0
on these maps; native geometry/cap branches remain unexercised and are not counted
as full tree geometry coverage. Bounded checker red/green has10 actual old0/new1
malformed cases and three valid boundary cases passing both.

The next Task49 work prepares the host-owned tree file loader and physical kernel
against these immutable inputs. Production entity/tree collision, model geometry
loaders, public authority/refusal, shipping and normal delivery remain required.
Task47 shipping115 is still live; checked31.2MB Tacview/19.7MB observation prefixes
match, with no complete-pack acceptance. ProductionG2/G3 gates and the full D3
model/environment/time/required-physical-solver/loader scope stay open. D4/D5 have
not started. The following records earlier checkpoints with their at-time status.

# Third capture P2 (warm-pair and lookup coherence) — correction UNBUILT, UNRUN; all three P2 source corrections now prepared (working tree over 41f94952e)

The one independent source sweep of the NEW capture (41f94952e) is complete with exactly three P2s and no others: (1) paging arithmetic and (2) capped forest-geometry acceptance, both already
prepared, and (3) pair/lookup coherence, addressed here in the real driver only (no C, physical, solver, query, generator, hook or fold change; no actual C wrong lookup was observed).
The checker counted a stage-1 warm record as warm agreement from its hit/point/normal without requiring it to be the same query as its stage-0 record (a changed warm endpoint was accepted), did the
same for stage 3 against stage 2, and never tied the supplied terrain lookups to the recorded endpoints (the valid stand-in had A.z=1 but l0.z=2). Now:
* A stage-1 record must equal its preceding stage-0 record, and a stage-3 record its preceding stage-2 record, in group, kind, role, ignore_edge, skip_top, a, b, initial point and initial normal
  (the existing fixed initial-output checks remain). Answers (hit, intercept, normal) and terrain ELEVATIONS are NOT compared across cold/warm, because cached hints can legitimately differ, and no
  physical answer is recomputed.
* The recorded lookups must be exactly what the original does: the x and z (bit for bit) of the old endpoint then the new endpoint, in that order, each only if it lies inside the map by
  point_inside_map_area (en_world.h 149: x and z only, min and max INCLUSIVE, y ignored), using the captured tf-map bounds and an exact f32-bit-to-value conversion (signed zero, infinities and NaN
  handled as the original's float comparisons would; actual data is never sanitised). The lookup count must equal the number inside; unused-lookup zeroing, the overflow guard and the strict schema are kept.
* The valid stand-in's lookups are now derived from its endpoints (old, then new, inside the stand-in map), so the feasible positive is coherent; its counters, answers and the 77 report controls are unchanged.
New real-driver controls beside the 11 earlier transcript controls (20 now): trace_warm_endpoint, trace_warm_flags, trace_warm_context (stage 1) and trace_near_warm_context (stage 3), and
trace_lookup_x, trace_lookup_z, trace_lookup_order, trace_lookup_count, trace_outside_wrong; each is a single defect that the old 41 driver accepts (so Root can reproduce the actual false pass with the same
stand-in and host) and the new driver refuses with, respectively: "tq 1 against its stage 0 record 0: b is 40000001,41000000,40400000, expected 40000000,41000000,40400000"; "... ignore_edge is 1,
expected 0"; "... kind is 5, expected 0"; "tq 3 against its stage 2 record 2: role is 7, expected 0"; "tq 0: lookup 1 x is 3f800001, expected 3f800000"; "tq 0: lookup 2 z is 40400001, expected
40400000"; "tq 0: lookup 1 x is 40000000, expected 3f800000"; "tq 0: lookups is 1, expected 2"; "tq 0: lookups is 2, expected 1". Three ACCEPTING controls (trace_outside_ok, trace_boundary_inclusive,
trace_boundary_outside_ok) show the checker is not stricter than the original's inclusive rule. The cap-512 control, the I/O, alias, default trace-off and capture-positive tests are retained.

Valid actual data must remain valid, and Root's independent audit (tree-query-context-41f94952e-audit.json) says it does against these same rules: Georgia 554 records / 277 matched warm pairs / 1102
ordered lookups; Lebanon and the repeat 600 / 300 / 1120; every pair matches all nine signature fields, and lookup counts, x, z and order match the inclusive endpoint-in-map predicate, with no
height answer derived. The raw transcripts and the 205 old report rows are immutable and are not rewritten. Source 41 remains NOT finally accepted: its three P2s are corrected in source only, UNBUILT and
UNRUN. Root formats, freezes and builds the combined source once and supplies the actual C paging witness, guards, traces and old/new red/green to the same reviewer.

The 5416adb3b / 54e5ece85 review stays closed within its old scope. No host tree kernel or loader has started; it waits for Root's validation of the new capture. All required whole-D3 entity, tree
geometry, environment, time, physical solver and loader-removal work remains; Task47 shipping115 is live and untouched; the production G2/G3 gates remain; D4/D5 have not started.

# Capped forest-facts P2 and the actual source 41f94952e capture results — correction UNBUILT, UNRUN (working tree over 41f94952e)

Second P2 from the same ongoing combined capture review: check_trace validated tf-forest.geometry and the triangle count but never required capped == 0. The emitter stops at 512 forest
triangles and writes capped=1 when another exists, so a coherent truncated facts block (512 indexed rows, triangles=512, capped=1) could reach trace_written, trace-end and done. A truncated
forest geometry is not a complete input for a host loader or kernel. Corrected in the real driver only: tf-forest must have capped == 0 ("tf-forest: capped is 1, expected 0") or the transcript
is not complete; geometry is not dropped, baselines are not widened and the capped set is never presented as whole. No physical C, solver, trajectory, query, generator or hook change. The current
retail data (forest geometry 0, capped 0) stays valid and unchanged. Real-driver control trace_forest_capped (a coherent 512-row truncation the emitter can actually produce, not a one-triangle
capped flag, which the emitter cannot produce) joins the transcript controls (11); the 77 report controls, I/O fault, alias and capture-positive tests are retained. Root's bounded red/green: the old
41 driver must falsely accept this stand-in transcript, the new driver must refuse it with exactly that diagnostic. UNBUILT/UNRUN until Root executes it.

Actual results of frozen source 41f94952e (Root's, collected): Windows fixture build exit0 with all 43 exports; Linux 14/14 plus format and Clippy exit0; actual Windows 14/14 with the missing-host
control exit101 as expected, controller exit0 and all four guard payloads unchanged. Three fresh retail trace probes (seed20261029, entity200/tree200/repeats2/seconds0/trace_out) native and controller
exit0 with all three pre/post input checks0; EVERY old 205 report-row value equals source 5416adb3b exactly, the only addition being trace_written (206). Georgia: 568 transcript rows, 554 actual tree
queries, 1102 terrain lookups; Lebanon and the separate repeat: 614 rows, 600 queries, 1120 lookups; all ordinal and overflow guards pass; the separate Lebanon whole report AND transcript are byte
equal. Native facts: tf-layout record_size 8, offsets x0 z2 y4 colour6, height_only 000000000000007f, type_only 0000000000000080, probe feff3412cdabefd5; forest geometry 0, capped 0. Exact audit
R:/d3-evidence/tree-input-41f94952e-audit.json; raw tree-input-41f94952e-smokes-20261029. This shows passivity for the tested cases; it is NOT acceptance of source 41f94952e, which carries the known
paging and capped-check defects. The 5416adb3b / 54e5ece85 review stays closed within its old scope.

No host tree kernel or loader has started. The one combined new-capture review stays open until the paging bound, the capped check, the native paging witness and new actual tests are verified.
Task47 shipping115 is live and untouched. All required D3 geometry, entity, environment, time and loader solvers remain required; D4/D5 have not started. Root formats, freezes and rebuilds the
combined new source after this follow-up.

# Transcript paging P2 (signed overflow before clipping) — bounded SOURCE correction, UNBUILT, UNRUN (working tree over 41f94952e)

One combined independent review of the NEW tree capture found a concrete P2: eech_collision_trace_read_run computed the signed int end = offset + count before clipping it to the record count,
while the Rust wrapper and the C entry accept any non-negative i32 offset with count up to 256, so an offset near INT_MAX reached signed overflow in the C itself. (A build with -fwrapv would have
wrapped it to a negative sum and returned an empty page by accident, which is why the old behaviour was not observably different; the defect was the unsafe arithmetic, not a wrong page.)
Corrected in the C read function only, independent of the Rust and entry guards: it now refuses a missing buffer or total pointer, a kind other than 0 or 1, a negative offset, or a count outside
1 to 256 (col-trace-refused reason=bad_arguments); an offset at or beyond the record count is the END of the transcript, a SUCCESSFUL EMPTY page with the total still reported; otherwise the page is
min(count, records - offset) records from offset, computed from a value already known to be below TRACE_MAX, so no addition can overflow. Normal paging, the reported total and the refusal of
bad arguments keep their documented meaning. This is paging only: no captured record, physical C, fold, case set, query order or report changed, and no output expectation changed.
Native witness (not a stand-in; a mock cannot show the C arithmetic is safe): tests/lua/collision-trace-paging.lua, run by Root with the actual built host and DLL on a fresh verified root:
  eech-world.exe collision-trace-paging.lua root=<root> out=<new .jsonl> [scenario=lebanon_retail|georgia_retail|georgia]
It arms the transcript, runs one small probe (seed 20261029, 5+5 cases, seconds0), then checks the normal pages of 256 (counts, starting ordinals, unchanged total), the last record alone,
END pages at offsets total, total+1, 2147483647, 2147483646, 2147483392 (INT_MAX-255) and 2147483391, each count 256 and each a successful empty page with the total, and six refusals (negative
offset, count 0, count 257, kind 2, kind -1, offset beyond 32 bits). A pass is exactly the printed line COLLISION_TRACE_PAGING_OK total=.. normal_pages=.. end_pages=6 refusals=6; anything else, or
no line, is a failure and never a skip. Linux/Windows driver guards and the existing 14 stand-in tests are unaffected by this change.

Frozen 41f94952e stays the known initial paging-defect source identity, not an accepted source. Actual results to date (Root's, not this edit's): its Windows fixture and driver guard build exit0 at
19:27:56Z (image 87cd7e37766a7253713f125b4866ad915115728ccb4551883a92d8dfdc8d509a, physical exports verified); Linux format, 14 driver/report/transcript/I-O tests and all-target Clippy passed;
the first retail trace probes (three fresh roots, seed20261029, entity200/tree200/repeats2/seconds0/trace_out, tool86719) and the Windows 14-test run (98355) were LIVE when this was written and are
not recorded as passed. The one combined new-capture review is still in progress; the source 5416adb3b / 54e5ece85 correction review stays closed and all prior raw is preserved. Task47 shipping115
is live and untouched; there is no host kernel or loader and no cutover; whole D3 stays active; D4/D5 have not started. Root formats, pins, rebuilds and runs the combined next source and supplies the
completed results to the same reviewer.

# Trace instrumentation prebuild correction (driver I/O and resource cleanup) — SOURCE ONLY, UNBUILT, UNRUN (working tree over 03413ccd3)

Root's focused prebuild inspection found a false-success path in the NEW transcript driver code: the trace writer ignored the results of file write and flush, the final close was ignored,
so trace_written and a normal done could follow a disk error; and the armed transcript and open file were cleaned up only on the happy path. Also trace_out=dir/./name could alias a just-created
output file that a string comparison does not catch. Corrected in the real driver only (no C, arithmetic, generator, order, terrain hook, physical fold or report change; trace off is unchanged):
* every trace write and flush is checked (new trace_write, with encode_line shared by the unchanged report writer): "could not be written"/"could not be flushed" fail the run before trace_written;
  the final close is checked ("could not be closed"); a file is accepted only when the output holds trace_written, whatever its last line says. Partial output is kept, never marked accepted.
* with a trace file the probe/read/check/repeat/write path runs under one handler: on any failure it records trace_failed, disarms the transcript if armed, closes the file, and re-raises the
  ORIGINAL error unchanged; a disarm or close error during cleanup is recorded as trace_cleanup_failed and never replaces the primary diagnostic or turns the failure into a pass.
* the existing-file check is repeated immediately after the output is created and before the trace file is created, so an alias of the output (dir/./name, and on Windows case or drive spellings)
  is resolved by the operating system and refused, never truncated; the output is closed on refusal. This is a check then a create, NOT claimed race-proof; Root uses fresh unique names.
Real-script stand-in controls (the stand-in proxies only the new trace file): write, flush, close and disarm failures (non-zero, no trace_written/coverage/done, trace_failed recorded, disarm and
close accounted, a cleanup failure recorded without hiding the first error) and a dir/./name alias control. The 10 transcript controls now also require disarm and a trace_failed record. All 77
report controls, trace-off output and the positive capture test are unchanged. UNRUN until Root executes it; no physical or query source change.

Read-only file-format facts from Root's audit (tree-file-format-audit-20261029.json, exit0; they do NOT replace the native sizeof and per-record height/type verification the trace performs):
Georgia tree_pos.sec/.dat (lowercase names) 848187 eight-byte records, 2894 with the high bit of the last byte set; Lebanon TREE_POS.SEC/.DAT (uppercase) 213955 records, 371 high-bit; stride, counts
and header dimensions match the measured native census. The host loader and kernel are NOT started: they wait for Root's run of the trace and its review.

# Next G3a step: private TREE-QUERY TRANSCRIPT instrumentation prepared — UNBUILT, UNRUN (working tree over 03413ccd3)

Inherited, closed boundaries (not re-reviewed, not re-run): the correction phase is closed. One independent pass found no open findings at native 5416adb3b and final driver 54e5ece85
(report SHA-256 92ad84e8764c6a0b4068712a3faf9f82bb7ca4bfbcd46861193bb50776931bff, reference/d3-collision-characterization/independent-correction-review-54e5ece85.json); all five
moving 120s+30s native cases and the controller exited0 with input checks0 and the original queries' flag effects restored; Linux 10/10 with format and Clippy, Windows 10/10 with the
invalid-override control101 expected and the four old 438 false-pass cases reproduced; fresh 54e driver with the unchanged 5416adb3b native binary gave all 205 boot-report rows byte-identical.
The old d5 moving failures stay failed with no valid oracle. Live authority read by Root: M5 goal v3, ledger v62 (all prior HTML and the five epic reports exact), Task49 In Progress under
D3/EEDCS-21, draft PR109. All D3 terrain/spatial/model/environment/time, required physical solver and loader removal remain D3 scope (not deferred to D5; an unsupported or refused query is not
completion). Campaign geography, routes, timers, scheduling, decisions and consequences stay in EECH_DC. D4/D5 have not started. Task47 corrected shipping115 (tool59652) is LIVE; Task48
host kernel/Lua prep is closed but production G2 is NOT cut over. No G2/G3 production cutover before validated preceding delivery and verified integration.

What was prepared (fixture-only, feature-gated, no host kernel, loader, production dispatch or framework): the missing INPUT characterization the host-owned tree kernel and tree-file loader
need. Every tree query the probe makes (primary cold and warm, shortened nearer-check cold and warm) can be recorded in execution order in a bounded private transcript: ordinal, group, kind,
role, stage, the two flags, f32 endpoint bits, the caller's initial intercept (zero) and normal (sentinel) bits, the ORDERED terrain lookups the original actually made (x, z, the elevation
it returned) and the final hit, intercept and normal bits including a miss's partial writes. Lookups are captured passively: en_cllsn.c's two terrain-elevation calls in
get_line_of_sight_collision_tree go through a TREE_QUERY_TERRAIN_ELEVATION macro that, without EECH_COLLISION_FIXTURE, expands to exactly the original get_3d_terrain_elevation call (14 lines
added to en_cllsn.c; the division lines move from 2021/2024 to 2035/2038), and with it calls a fixture function that makes the identical call, returns the identical float and records.
No extra terrain query, no solver arithmetic, control flow, generator, call count or order, getter, FPU mask, RNG, quad/triangle helper or early return is changed; the flag/terrain snapshot,
effects report and restoration are untouched. A facts block records the tree record layout measured natively (size, member offsets and a known-pattern byte dump that fixes the packed
height/type bit order: the declaration is eight bytes on these builds, not six, but nothing is assumed), the scales and bounds, the forest tree object's collision triangles and the first live
records as raw bytes. Raw output is private and out of Git. Interface: Lua driver argument trace_out=<path> (default off; the report is identical either way), engine:collision_trace_arm(on)
and engine:collision_trace_read(kind, offset, count) (kind0 state+facts, kind1 records, count 1..256). Bounds: 32768 records (about 4 per random tree case plus about 160 structured, so the
driver requires tree_cases <= 8000 with trace_out), 160 KiB of facts, 512 forest triangles; overflow, an extra lookup or a facts overflow is flagged and the driver FAILS rather than truncate.
Cleanup/safety: an existing trace file is never overwritten (refused before anything is opened or booted), the file is created before the run and written only after every repeat's
transcript has passed, so a failed run leaves it empty; a transcript without trace-end is incomplete; the transcript statics are cleared on arm and disarmed after the repeats.
The driver parses the transcript strictly and checks it against its own report (record counts per group and stage, hits, warm agreement, miss-time writes, derived zero-extent rays, tree
census and map facts, initial outputs, stage order, no unrecorded lookup) and requires repeated transcripts to be identical; 10 transcript negative controls, 3 argument guards and a
complete-transcript positive test were added to the real-script tests. The 77 report controls, strict-JSON retention and the host-path override are unchanged.

Not done: nothing here is built or run. Root formats, freezes and builds ONE new named fixture, runs seconds0 with the existing seed20261029/entity200/tree200/repeats2 and trace_out on
fresh verified Georgia, Lebanon and separate-repeat roots, compares all 205 existing report rows/folds/samples/counters with 5416adb3b (new metadata separate), inspects the lookup
counts, pair order and raw transcripts, and obtains one independent review of the NEW instrumentation and evidence. Only then does the host-owned tree loader/kernel start. That work must
preserve the 8-byte tree file layout, float/double promotions and FPU, map gates and sector/tree order, the 35 m and 0.1 early returns, the flags, strict-nearest quad and first-triangle
rules, partial writes and the ordered terrain cache effects, with the full remaining D3 scope. A first quad-path-with-geometry refusal would be preparation only; no permanent unsupported or
miss substitute is approved and no unverified DCS API is claimed. Unresolved risk: the transcript is captured at the C wrapper, so the elevation values are the original's return values as
floats; a build whose float evaluation differed would show it as a changed old fold, which Root compares.

# Current collision checkpoint — correction review closed; host preparation next

Independent correction review is closed with no open findings at native source
5416adb3b and final driver/helper source54e5ece85. Retained review SHA256:
92ad84e8764c6a0b4068712a3faf9f82bb7ca4bfbcd46861193bb50776931bff.
All three original boot probes and five moving/control cases complete0; actual
query flag effects are reported and restored, repeated reports agree and coarse
campaign digest pairs match. Coverage gaps and the old failed d5 runs remain.

Final actual-driver checks pass10/10 on both Linux and Windows; Linux format and
host all-target Clippy pass. Four impossible-flag reports actually pass the old
driver and fail the corrected one; legitimate added-bit counts pass both. A
missing Windows host path fails explicitly. Fresh54e driver/unchanged541 native
boot reports match all previous205 rows byte for byte. All input checks pass;
the retained runtime remains541, without a relabelled native build or oracle.

No collision responsibility has moved to production. The next bounded Task49
step captures tree-query inputs, ordered terrain lookups and outputs without
altering original queries, then prepares a host tree loader/kernel. The source
tree record is eight bytes (not six); file parsing and bit order require measured
verification. Entity collision, model geometry and the full tree geometry path
remain required withinD3. No unsupported tree path may become a completed query.
Task47 shipping115 is still live; latest23.4MB Tacview/14.6MB observation prefixes
match accepted bytes, with no full-pack acceptance yet. Production G2/G3 delivery
and the remaining whole-D3 environment/time/solver/loader work remain required.

Safe evidence and the independent review are under
eech-campaign/reference/d3-collision-characterization/. Two closed d5 Cargo
caches were removed after full build-log export, payload and ownership checks;
all source, binaries, successful/failed raw runs and retail roots are retained.
The following sections preserve earlier checkpoints with their at-time status.

# Current collision state: source 5416adb3b moving witnesses complete; col-flags feasibility follow-up UNBUILT, UNRUN (working tree over 438940d39)

Actual results (all by Root, none by this edit). Source 5416adb3b: Windows physical build exit0 with 43 entries; three boot probes exit0 with every old d5 199-row value identical (six new
diagnostic rows separate). Its five moving 120s+30s native cases (controller tool68773) have all TERMINATED exit0: all five pre/post checks0; probe and control coarse identity/kind/side/alive digests
equal; the separate Lebanon entire reports are byte identical and both probe-report repeats identical; FPU masks 63/63 and record/RNG/world/flag restoration hold. Georgia: 10 changed updated-flag
words, 62 cleared bits, restored1, 5 moving targets; Lebanon and its repeat: 6 selected moving targets. Exact raw and controller output: collision-observer-5416adb3b-controls-20261029 (private).
These are original-behaviour observations with explicit gaps: no new physical or Tacview claim; the old d5 failed moving probes stay failed with no valid oracle. Source 438940d39: Linux 9/9, format
and all-target Clippy exit0; Windows guard and matching host built exit0 with the four payloads hashed. The actual Windows override run now passes all9 tests using a verified host path
containing spaces, and the invalid-host control fails101 with the explicit no-fallback diagnostic. All4 payloads remain unchanged. All12 retained native flag rows satisfy the stronger bounds.

New bounded P2 from the one combined independent review (driver feasibility validation of col-flags, not a C snapshot defect): zero changed words with a nonzero cleared bit was accepted, counts
could exceed 32 bits per changed word, and one array's changed words with no bits could be hidden by another array's clears. Source follow-up in the REAL driver, UNBUILT/UNRUN, no change to C,
production code, generation, folds, masks or restoration: (1) each array's cleared_bits <= 32 x its changed_words; (2) the minimum additions, sum over arrays of max(0, changed_words - cleared_bits),
must be <= added_bits (a changed word with no cleared bit in its own array differs only by an added bit); (3) sum(cleared_bits) + added_bits <= 32 x sum(changed_words). added_bits is NOT forced to 0
and the existing nonnegative, capacity, lower-bound and hash checks stay. Four one-defect stand-in controls (bits with no changed word; per-array overflow; aggregate overflow; a changed array
lacking bits despite another's clears) and one positive control (realisable aggregated added bits are accepted); 77 negative controls now. Dynamic bit-31/refusal coverage is not claimed.
Root audits the stronger rules against all retained native flag rows without changing them, runs new Linux and Windows guards, and may run fresh boot probes through the new driver with the
unchanged 5416adb3b C binary, clearly labelled. One combined independent review remains pending. No collision query is extracted; production G2 remains gated and Task47 shipping115 is live and untouched;
whole D3 stays active; D4 and D5 have not started.

# Collision driver Windows setup defect and host-path override — follow-up UNBUILT, UNRUN (working tree over d16c08bec)

Actual results to date. Source 5416adb3b: Windows physical build exit0 with 43 entries; three native boot probes exit0 with 205 rows each, every old d5 199-row value matching (the six new
diagnostic rows are separate); Lebanon raw reports identical; all pre/post checks0. Its moving 120s+30s probes/control/repeat (tool68773, controller13108) are LIVE and not accepted.
Source d16c08bec: Linux 9/9 driver tests, format and all-target Clippy PASS (exit0). Its Windows driver guard binary built (exit0; the three exported entries match before and after), but the
actual Windows run is TERMINAL exit101 with 0/9 passed: every test failed identically with OS error 3 at the command.output unwrap in run_collision_case. Cause: the helper still started
Command::new(env!("CARGO_BIN_EXE_eech-world")), the Linux build path baked in at compile time; the earlier include_str driver embedding removed the source-path dependence but not the executable-path
one. No driver or physical test passed on Windows, no baseline changed; the failed log and completion are preserved.

Follow-up (test helper only; production, fixture C, driver validation, generation, folds, order, masks and restoration untouched): run_collision_case now starts collision_host_exe(). The runtime
override is EECH_WORLD_BIN (the name the ground handoff tests already use): when set it names the eech-world executable to start, read as an OsString/PathBuf so spaces work; a set value that is not an existing
file FAILS the test with a clear message (no fallback, skip or pass); unset, the ordinary cargo path is used. Only the collision driver tests use it; the other old host_callbacks tests are not touched.
The 73 malformed-report controls and the strict-JSON retention assertions are unchanged. This change is UNBUILT and UNRUN until Root exports the matching same-source eech-world.exe beside the test and lua.dll,
sets the override for that single native process and runs all 9 on Windows. One combined new-delta independent review is pending. No collision query is extracted; whole D3 stays active; Task47
shipping tool59652 is untouched; no D4 or D5.

# Current collision checkpoint — corrected native boot reports match retained original values

The clean Windows fixture at 5416adb3b built successfully (actual exit 0), with all
43 exported payloads verified. Georgia, Lebanon and the separate Lebanon repeat
boot probes each exit 0 with 205 strict JSON rows, restored terrain/update flags,
unchanged game RNG and both FP exception masks 63. Every value in all 199 retained
d5 boot rows matches, including physical/legacy folds and samples; the six added
rows are new census/FP/flag diagnostics. Full raw Lebanon repeat reports match.
All three pre/post input manifests pass. This is bounded boot evidence with the
original coverage gaps; it is neither moving-case conformance nor shipping proof.

Fresh 120s+30s original-query probes and matched controls are running on immutable
5416adb3b. The test-only expectation correction below changes two strings, with no
native/driver behavior change; its Linux/Windows checks and independent delta
review remain pending. Task47 shipping115 remains running unchanged. No collision
responsibility has moved; production G2 and the rest of D3 remain required.

The following sections preserve earlier results and their at-the-time status.

# Source 5416adb3b Linux guard result and test-expectation follow-up — follow-up UNRUN

Actual first Linux run of corrected source 5416adb3b is TERMINAL (exit 101; completion collision-driver-linux-5416adb3b-completion.json, 18:40:16Z): format passed, 8 driver
tests passed, 1 failed, and Clippy did not run (the script stops at the first failure). The failure is in the_collision_driver_names_the_one_defect_in_a_report_and_writes_no_success_row:
the validator correctly reported "col-entity-common struct: hit + miss is 71, expected 44" for mode common_cases_wrong (the stand-in sets cases 44), while the test still expected "expected 72".
That is a stale test expectation left over from the coherent 15-target valid stand-in, not a physical regression and not a validator or stand-in defect.

Follow-up (test source only; production source, native C and driver semantics unchanged; UNRUN until Root executes it): every one of the 73 negative-mode expectations was re-derived from the
CURRENT stand-in values and validator order. Two were stale and are corrected: common_cases_wrong (expected 44, as the stand-in sets) and census_paths_wrong (the stand-in sets box_path 7, so
"geometry_path + box_path is 16, expected 18", not 17). All other expectations match. Nothing was weakened; all 73 negative controls, the strict-JSON report retention and the positive valid-world
constraints remain. Root formats and runs this corrected test source, cross-builds its Windows guard binary and pins every measured source; the initial failure is preserved.

The Windows physical build of 5416adb3b is LIVE and has not passed. The old d5 failed moving probes stay failed with no valid oracle. The same independent review remains CHANGES REQUIRED pending the
new source and actual results. Whole D3 stays active; no production G2/G3, D4 or D5.

# Fixture observer P1 — SOURCE CORRECTION PREPARED, UNBUILT, UNRUN (working tree over f8092241d)

The terminal long-run failure (world_unchanged0 after1200 frames; both controls complete) is explained: the d5 fingerprint called the ORIGINAL destructive INT_TYPE_UPDATED getter
(mb_int.c398 `array[index] &= mask`), which clears the other bits of the entity's 32-entity word in the bounded up_update.c arrays. Old d5 zero-frame smokes, the failed moving probes
and the coarse-only controls stay as measured d5 identities with their limited scope; there is no valid moving oracle and none is normalised. The production getter and the original
collision code (which itself calls the getter) are untouched.

Source correction, none of it built or run: the fingerprint reads the RAW updated bit (no getter); the three bounded flag arrays are copied in full before anything is observed, the
ORIGINAL queries' effect on them is measured against the copy and reported as LEGACY diagnostics (col-flags: hashes and counts only), the copy is written back on every exit and before the
FINAL fingerprint (so any other change is still caught; the terrain record is restored as before; the game RNG is not reset). The driver requires restoration and coherent counts; a rejected
real report is now retained whole as raw_report in the private output (JSON-escaped), as is a differing repeat. Tests: seven col-flags negative controls and a strict-JSON round trip of
a retained report. The fingerprint still excludes CITY children and identities and the warm counters still cover primary queries only; no state-purity claim. Root builds ONE new clean
source, runs Linux and cross-built Windows guards, zero-frame conformance with masks, and fresh moving probe/control witnesses before any of this is accepted. The same review remains
CHANGES REQUIRED. Whole D3 stays active; no production G2/G3, D4 or D5.

# Current collision gate — observed destructive-flag observer defect; no moving oracle

The original long run is TERMINAL1 (tool84252;18:18:29Z). All three probes fail
native1 after1200frames with world_unchanged0, before measurements/digest/coverage
are accepted. Both controls complete1500frames/native0; all5post-inputs0 and43
exports unchanged. Pair digest comparison is UNAVAILABLE, not an observed mismatch;
failed Lebanon raw equality is failure reproducibility only. Old d5 boot-smoke
results retain their limited scope. Safe failure proof and the full independent
review are in reference/d3-collision-characterization/ under eech-campaign.

The same combined review requires changes:1P1/4P2/1P3. Source-only corrections for
bucket/target completeness, verified FP masks/derived-axis accounting, portable
actual-script guards and precise scope wording are prepared but UNBUILT/UNRUN.
The NEW P1 remains OPEN: world_fold calls the original destructive UPDATED getter
(mb_int.c398 uses &=), changing shared flags before measurement. Original moving
collision also legitimately invokes it. Before ANY new build, the sole Sonnet lane
must add nondestructive fixture observation, bounded flag snapshot/effects report/
restore on all exits, preserving production &= and all original queries/order.
Do not relax the assertion or normalize a moving oracle. The production queries,
case generator/fold algorithms, baselines/tolerances remain unchanged.

The following records earlier checkpoints and the first source-correction batch.

# Collision fixture reviewer corrections — SOURCE ONLY, UNBUILT, UNRUN (working tree over publication bd2a31949)

One combined independent review of pinned d5ca5ee5b found four P2 defects and one P3 scope issue; all are corrected in source after the pinned d5 build and smokes.
The d5 build, its three seconds0 smokes and the five live 120s+30s processes remain measured identities of the d5 source: this is NOT a new corrected-source pass, and nothing
below was compiled or executed. Original queries, the structured and random case generators and order, and the physical folds are unchanged.

1. Exact hit-type conservation: summed col-entity-type hits must equal each group's hits (was <=); deleted-bucket and all-omitted negative controls.
2. Exact target selection: new row col-census-class (class census, not only selection); col-targets must equal min(class census, 3) for all ten classes and the class census must
   agree with col-census-entity/-tree; the positive stand-in was impossible and is now coherent (quota-binding classes included); under-selection controls added.
3. FP mask contract: the probe reads (fnstcw, stmxcsr; never writes) the x87 and SSE masks and requires all six in both (63/63) or is refused; reported as col-fp. d5 did not check
   this and nothing infers it did. The primary random tree rays with zero x or z extent are still skipped and counted (skipped_axis); the DERIVED shortened nearer-check ray, which
   can reach en_cllsn.c 2021/2024 with a zero extent, is queried unscreened and counted (derived_axis, new field, must be <= nearer_checked). Source axis checks alone are not a full physics statement.
4. Portable guards: the Rust tests embed the actual driver with include_str! and copy it to the private test directory (@DRIVER@); no source-machine path at run time. Root cross-builds and runs it on Windows.
5. P3: the world fingerprint covers sector-list entities' type/flags/position and mobile motion/updated flag plus delta time, not city buildings or identities; warm counters cover primary
   queries only; repeated reports plus a coarse digest are repeatability and bounded observations, not candidate-state purity or Tacview equivalence. Messages and docs say so.

Still required: Root builds a NEW clean named correction, runs Linux and cross-built Windows guards and native smokes with masks, compares physical folds/samples/counters with retained d5
(separating the new rows/fields), and obtains same-review delta verification. D3 stays active; production G2/G3 switching, remaining model/weather/time/loader services, D4 and D5 are unchanged.

# Latest original collision native checkpoint — smokes completed; controls live

Clean measured source d5ca5ee5bb1f784318ce9c76903169cb97bbe362 Windows fixture
build0; all43 exports verified. Actual Georgia/Lebanon/separate-repeat seconds0
probes (200 random cases per family, two reports) each native0/pre/post-input0,
199 strict rows, report-repeat equality and restoration/RNG/candidate-state flags1.
Separate Lebanon raw reports are byte identical. Source/original samples stay
immutable. Safe aggregate: reference/d3-collision-characterization/ under eech-campaign.

Coverage is GAPPED: no moving target at boot, no terrain-aligned fixed object,
no forest collision geometry or geometry-tree target; Georgia also has no two-object
ray. These remain limits, not full characterization. Axis-aligned tree rays and
player/gunship reachability are still unresolved. No extraction/shipping claim.

Five fresh original probe/control processes are LIVE on the SAME frozen d5 build:
Georgia pair, Lebanon pair and separate Lebanon probe;120s before +30s after,
100ms/1500frames,2000random per family/two reports. Controller21768/tool84252,
natives2840/9504/25404/20896/16772 born18:12:03Z; exact identities recorded privately.
Full controls/moving coverage/post-inputs and one combined new-source/native review
are in progress. No Tacview is generated by these probes; the digest control is
identity/kind/side/alive only, not positions/all-state/recording equality.

PR109 continues this Task49. Task47 shipping11559652 remains live and untouched;
production G2 delivery and remaining model/weather/time/loader services stay D3.
The following records the preceding checkpoints with their at-time limitations.

# Latest collision characterization checkpoint — driver guards passed

Actual working-tree Linux format/check, eight the_collision_driver_* tests and
host all-target Clippy passed0 at 2026-10-29T18:02:22Z. These are stand-in/driver
flow and schema tests, not original C/retail collision evidence. Exact tested
hashes and log are in reference/d3-collision-characterization/ under eech-campaign.
Three fresh reserved Georgia/Lebanon/separate-repeat roots match the repository
input manifests; no native collision campaign has run on them yet.

Next: clean named Windows collision-fixture build and verified exports; first
seconds=0 native probes, then seconds=120 paired probe/control runs and separate
repeat, all on verified private roots. Record actual exits, differences and coverage
gaps, independently review new source/results, then continue host extraction.
Production switching waits validated preceding deliveries and freshly verified
integration. Ground host kernel/Lua preparation reviews are closed; production G2
handoff/shipping delivery and Task47 final shipping115 still remain. Whole D3 scope
is retained; D4/D5 not started. Current ledger is14548994v60; the following records
the author's earlier live v59 read and source-preparation checkpoint.

# D3 object and tree collision queries (EEDCS-49) — original characterisation source prepared

29 October 2026. **Source only: unbuilt, unrun, unreviewed.** Task [EEDCS-49](https://flying-dice.atlassian.net/browse/EEDCS-49) (parent
[D3/EEDCS-21](https://flying-dice.atlassian.net/browse/EEDCS-21)) on branch `codex/d3-object-tree-collision`, base accepted integration
`6bbaa8197a4843181d03cd355d562af998c75321` (clean when work began). Live authority read 29 October: M5 goal v3, migration ledger v59, EEDCS-49 and
EEDCS-21 (no tool was unavailable). D1 and D2 are accepted; D3 is the only active epic; D4/D5 have not started.

Gate advanced: original characterisation of `get_line_of_sight_collision_entity` and `get_line_of_sight_collision_tree` before any ownership change.
**No physical responsibility has moved, no production dispatch changed, and nothing here completes EEDCS-49, G3a or D3.** Production G3 switching still
waits for validated preceding delivery (EEDCS-47 shipping regression, the G2 host-Lua review) and freshly verified integration.

Prepared (all feature-gated test-only; none in the shipping module): `collision-fixture` feature chain `eech-dc` → `eech-engine` → `eech-engine-sys`;
`csrc/eech_collision_fixture.c` (calls the ORIGINAL functions with the consumers' call shape against the loaded sector lists, 3D object database and tree
data); `eech_engine_collision_probe` entry in `eech_engine.c/.h`; `Engine::collision_probe`; Lua method `engine:collision_probe`;
`lua/collision-characterization.lua` (arguments checked before anything is opened or booted, strict JSON, report accepted only as a whole, coverage
stated); `tools/build-windows-collision-fixture.sh`; a stand-in and `the_collision_driver_*` tests; and
[docs/m5-collision-characterization.md](eech-campaign/docs/m5-collision-characterization.md) with the source findings, finite case set, report schema,
unsupported inputs, run command and risks.

Source findings worth the lead's attention (read from the source; **not runtime evidence**): the entity query is first-found in a fixed scan order,
not nearest-hit; exclusion tests only the sector-list entity, so excluding a city building does nothing and excluding its city excludes all of its
buildings; both intersection helpers write the face normal (and the plane-crossing point) before deciding hit or miss, so a miss can leave written
outputs that the weapon-step lookup reports to the host; the tree query has two tie rules (first triangle hit versus nearest quad) and divides by the
segment's x and z extents; both queries write the shared terrain point record. The gunship consumers (`co_force.c` uses the face normal) are compiled; their
reachability disposition is not decided here.

Not done and still required: compile the clean named fixture build; execute the stand-in tests (their expected messages were derived by reading the
driver); run the native probes on fresh verified Georgia, Lebanon and Lebanon-repeat roots at `seconds=0` and `120`, with and without probe, and
`repeats>=2`; learn whether the retail worlds offer tree data, moving targets, buildings and geometry targets (the `coverage` event says); classify
differences; independent review. Adding `lua/collision-characterization.lua` adds one entry to the build file manifest; the C and Rust changes outside the
feature are `#ifdef`/`cfg` only and are expected, but not verified, to leave shipping bytes unchanged. No baseline, tolerance, scenario or seed changed;
all prior records below are preserved.

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
