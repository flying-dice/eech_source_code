# EEDCS-47 terrain surface characterization (original solver)

Base `6bbaa8197a4843181d03cd355d562af998c75321`, branch `codex/d3-terrain-services`, 29 October 2026.
Census and group map: [`docs/handover/D3-SCOPE-AUDIT.md`](../../../docs/handover/D3-SCOPE-AUDIT.md).

**Status: original a8 Windows fixture controls completed; new f4 host backend
checkpoint fails compilation and is under review. No physical service has moved.**
[`original-controls-a8effa9f3.json`](original-controls-a8effa9f3.json) summarizes
the completed original controls and complete 900-second recording equality.
Raw recordings, retail data and binaries remain private. Source findings and
planned checks below retain their stated limits; no backend conformance,
fast-oracle result or new shipping acceptance is claimed.

## What is prepared

A `terrain-fixture` cargo feature (test-only, same pattern as `los-fixture`/`sensor-fixture`; never
enabled in the shipping build) that observes the **original, unmoved** terrain solver inside the real
campaign DLL, run through the public path (host Lua `require ("eech_dc")` -> native module):

| Piece | File | Role |
|---|---|---|
| Observer hooks | `modules/3d/terrain/terrelev.c`, `terrelev.h` | Under `EECH_TERRAIN_FIXTURE` only: the query macros pass `__FILE__`/`__LINE__` (the existing `DEBUG_TERRAIN` form, now keyed on `TERRAIN_CALL_SITES`), and each return calls `eech_terrain_record`. With the feature off the solver and its control logic are unchanged by inspection only (no preprocess, compile or runtime check yet); the added lines shift diagnostic source locations such as the off-map `ASSERT` (`__LINE__` 1268 -> 1281). No numerical drift is alleged. |
| Trace + sweep | `crates/eech-engine-sys/csrc/eech_terrain_fixture.c` | Run-wide trace, per-call-site table, deterministic probe sweep, text report. |
| Engine entries | `eech_engine.c/.h`, `eech-engine-sys` (`lib.rs`, `build/main.rs`, `Cargo.toml`), `eech-engine`, `eech-dc` | `terrain_trace`, `terrain_trace_reset`, `terrain_probe (seed, count, emit)` Lua methods. |
| Driver | `crates/eech-world/lua/terrain-characterization.lua` | Boots Lebanon or Georgia, reports boot-phase queries, runs frames, takes probes, asserts the probe leaves the run-wide trace unchanged. |
| Build wrapper | `tools/build-windows-terrain-fixture.sh` | Single-argument adaptation of `build-windows.sh`, as for the other fixtures. |

### Why this exercises real collaborating components

The queries counted are the ones the campaign's own code makes (`hc_move`, `fw_move`, `rv_move`, `ps_move`,
LOS terrain phases, weapon steps, targeting, the boot-time geography in `popread`/`faction`/`taskgen`)
against the real map loader and the real solver, in the real update order. Nothing is mocked and no campaign
decision is controlled. The only controlled input is the sweep's point set (a private xorshift, never the game's
random stream), which asks the *same solver* about points it would otherwise not meet in a run. The sweep
saves and restores the shared find-fast record (`eech_terrain_system_point_data`) so the campaign's own hint is
left as found, and keeps its own trace.

### What the report contains (all `key=value` lines)

- **Run-wide** (`scope=boot`, then `scope=run`): order-dependent FNV-1a hash over every query's x and z bits, the
  hint/fast/found/clamped/Cuba flags, the returned elevation, `last_elevation`, the normal and the type; counts of
  shared-record (`hint_null`) and private-record queries, hints tried (`fast_attempted`) and hit, full sector scans (`slow_scans`, whether or not a face was found), the
  queries that found no face (`not_found`, a subset of those scans), Cuba-area returns, clamped queries, and `partition_ok` (1 when `fast_hit + slow_scans + cuba == calls`); a surface-type histogram; and one row per call site (file, line, calls,
  shared-record calls, fast hits, not found). The boot report precedes the trace reset, so boot-time geography
  is counted too.
- **Probe**: six fixed points (four corners inset 1 cm, the centre, one sector-corner point) then `count` random
  points, one in eight snapped to a multiple or half-multiple of the sector side +-1 mm. Per point the reference is a
  fresh zeroed record; the same point is then asked (a) again on that record, (b) on a private record primed 5 cm to
  500 m away, (c) on the shared record primed the same way, (d) through `get_3d_terrain_elevation`,
  `get_3d_terrain_elevation_and_face_normal` and `get_3d_terrain_face_normal`. Reported: `sample` and `primed`
  hashes, mismatch counts for each of (a) to (d) against the reference, `result_ne_last` (Cuba-style
  difference between return value and `last_elevation`), the type, class and ground-vehicle suitability of every
  type met, the first `emit` samples as raw float bit patterns, and `shared_record_restored`.

### Limits of the fixture

- It counts and compares; it does not say which answer is *right*. A mismatch between a hinted and a fresh answer is
  reported as original behaviour, never corrected.
- Per-site attribution uses the macro call site. Queries through the wrappers in `terrelev.c` attribute to `terrelev.c`.
- Off-map input is not probed (the original asserts). Cuba flaw behaviour is only observed if the Cuba map is run.
- A fixture build is a different binary from the shipping module; its run hash is not comparable to a shipping
  Tacview byte file. Step R5 below checks that the fixture is observational by running the accepted scenario.

## Planned run matrix (for the Root; none executed)

Build, from a clean checkout of the commit that carries this directory, in the Git Bash used for the other fixtures
with Docker Desktop (Linux containers) running, from `eech-campaign/`:

    sh tools/build-windows-terrain-fixture.sh target/terrain-fixture/baseline-20261029

Then, with `root=` an installation root from `R:/d3-evidence/terrain-original-roots/{lebanon,georgia}` (private,
verified by the Root: 1179 and 1174 files) and outputs under `R:/d3-evidence/`:

| Run | Arguments | Purpose |
|---|---|---|
| R1 | `scenario=lebanon_retail seconds=600 frame_ms=100 probe_at=0,300 probe_seed=1 probe_count=20000 emit=256` | baseline run-wide trace, boot geography, sweep at the loaded and stepped world |
| R2 | same as R1 | determinism: every hash and count equal |
| R3 | `scenario=georgia_retail`, otherwise R1 | second map |
| R4 | R1 with `probe_at=` (empty) | no-probe control: the run-wide hash of R4 must equal R1's, showing the sweep does not disturb the campaign |
| R5 | the accepted default 900 s scenario with the fixture DLL, compared with the accepted baseline Tacview/observation/metric bytes | shows the hooks are observational |
| R6 | a second `probe_seed` and the `georgia` (map16) scenario | optional breadth |

Retain the JSON Lines outputs, the stderr, the exit codes, the BUILD-INFO and the post-run input manifests in
`R:/d3-evidence/`; commit only compact, non-retail summaries here. No retail asset, raw recording or credential
belongs in Git. Existing evidence to reuse without rerun: the frozen LOS terrain transcript
`reference/d2-los-observation/original-transcript/` and the D2 shipping pack.

## What this does not yet provide (needed before and after extraction)

- Controlled **changed and withheld host answers**: there is no host terrain callback yet, so nothing here can show a
  host answer changing a campaign state. Planned after the protocol is chosen: the reference host answers from the same
  solver (equal-input conformance, byte-identical regression), then an altered provider (as the LOS +10 km case),
  then a missing provider that must produce a clear error rather than a fallback.
- A **named DCS call or data workaround** per service is only listed as candidates in the audit (section 6).
- The **full byte-identical 40-check regression** with all eight artifacts after any behavioural move.

## Next

1. Independent review of the fixture source (never compiled; expect build fixes).
2. Root builds the fixture and runs R1 to R5; retains raw results.
3. Settle the open questions in the audit (section 5) with the technical lead.
4. Only then the smallest terrain surface extraction on EEDCS-47, with the red/green checks described in AGENT.md.

## Corrections after the first independent source review (pinned `257a18879`)

Source only; nothing built or run.

- **Driver argument validation (P2).** `terrain-characterization.lua` refuses a bad argument with a
  `terrain-characterization:` error *before* it opens the output file, requires the module or boots. Bounds (documented in the
  script header): `frame_ms` integer 10..1000; `seconds` decimal 0..36000 (positive: 1..1,000,000 frames and the clock must advance
  every frame; exactly `0`: an instantaneous boot-and-sweep that runs no frame and so has no clock-progress check, with every requested
  probe still required to be taken); `probe_seed` integer 0..4294967295 (a u32; the C sweep treats 0 as 1, so seeds 0 and 1 give the
  same sweep); `probe_count` 0..200000; `emit` 0..256; `scenario` one of the three; `root` and `out` non-empty. `probe_at` is empty
  (`probe_at=`, the no-probe control) or a strictly ascending list of plain non-negative decimals, none beyond the simulated total
  (so the default `0,300` is refused with `seconds=0`); malformed, empty, repeated, descending, negative and non-finite tokens are
  refused. Defaults are unchanged (600 s, 100 ms, `0,300`, seed 1, 20000 points, 64 samples). Unknown argument names are not
  rejected. Root's real-Lua-5.1 guard helper covers only argument setup under an I/O tripwire and never boots EECH or runs a frame:
  v1 was RED on `257a18879`; the corrected working tree was 11/11 at an earlier hash (`577ffab4`); the v2 cases (which add the two
  zero modes) were 4/14 on `257a18879` and 12/14 on the first correction, failing only `valid_probe_only` and
  `valid_zero_seed_alias`, which this preservation fix restores. The re-run is the Root's; none of it is native characterization
  acceptance.
- **Observer fidelity (P3).** `slow_scans` counts every non-fast, non-Cuba query, including a full scan that finds no face.
  Added `partition_ok`; the driver refuses a report where it is not 1. No claim is made that a no-face query occurs.
- **Evidence wording (P3).** See "Observer hooks" above: feature-off logic unchanged by inspection only.
- **Census (P2).** The audit now lists active model/device/target-point geometry as D3 (G3b, G3c) and no longer calls any active
  service dormant without evidence; see `docs/handover/D3-SCOPE-AUDIT.md`.

Root verification after the preservation fix: the real Lua 5.1 v2 setup guard
completed with actual exit 0, all 14 cases passing; syntax parsing also passed.
Tested driver SHA-256: 4400e7eded209f3cd721f3dfa290f7853011eddd9fcca99cb8c9a104d9e0749f.
The helper uses an output-opening tripwire and never boots EECH or runs a frame.
This supersedes the pending guard re-run statements above; C/Rust compilation,
original native characterization and observation controls remain unrun.

## Delta pass on `f05005832`

- **Due-probe draining.** The driver now takes *every* due probe at each frame (`while`, bounded by the finite probe list), in
  order and each at its requested time, instead of one per frame; the old behaviour lost accepted times that fall in the same
  frame. Defaults and the zero-duration mode are unchanged. The Root's local scheduling helper
  (`R:/d3-evidence/terrain-driver-schedule.py`: real Lua 5.1, one controlled engine-clock collaborator) was 3/5 on `f05005832`;
  the re-run is the Root's. It is not native evidence: a native one-frame, two-due-probe smoke remains required after the build.
- The audit's G7 label in section 7 is now "unproven reachability", not "dormant".

Root verification after the due-probe correction: actual Lua 5.1 syntax passes,
all 14 setup guard cases pass and all five local scheduling cases pass.
Driver SHA-256: 0a42d3c6928c1da65d2861979e1a4024cd2a9d390457343aeadd822d8af6d035.
The scheduling test controls only the fixture clock collaborator; it does not
boot EECH or implement terrain. The actual native one-frame/two-probe smoke
and all original characterization/recording controls remain required.

## Increment 1 of the extraction: independent host terrain backend and public-provider RED preparation (source only)

Base `a8effa9f3` (frozen original source `R:/d3-evidence/terrain-original-source-a8effa9f3`, original native results under
`R:/d3-evidence/original-*-a8effa9f3`). **Nothing below has been compiled or run** (no C compiler or cargo in the author's tool
environment); the Root builds and executes. **No production dispatch changed**: the campaign DLL still answers every terrain
question from its own solver, so this increment moves no behaviour and claims no D3 delivery.

### What now exists, and who owns what

| Path | Role |
|---|---|
| `eech-campaign/crates/eech-world/csrc/terrain_ref.c`, `.h` | The host's terrain kernel: loader, point-surface query, fast grid, map facts, and a conformance-only sweep |
| `eech-campaign/crates/eech-world/src/terrain.rs` | Rust ownership of the map bytes, FFI, opaque hint codec, the Lua provider `host.terrain_open` |
| `eech-campaign/crates/eech-world/build.rs`, `src/lib.rs`, `src/main.rs` | Wiring only (the kernel is compiled with the other host kernels, same flags; `main.rs` gains one `terrain::install` call) |
| `eech-campaign/crates/eech-world/tests/terrain_conformance.rs` | Compares the kernel with the original probe's retained results (gated on the private data) |
| `eech-campaign/crates/eech-world/tests/terrain_backend.rs` | Synthetic-map tests of the loader, hints, sweep bounds, fast grid and two maps on two threads (no assets) |
| `eech-campaign/crates/eech-world/lua/terrain-provider.lua` | Public-path RED harness for the provider contract |

The host owns and loads the terrain itself from `terrain.ffp`, `default.sec` (`default-W.sec` in winter) and `terrain.sim`, through no
DLL callback, sidecar or second campaign. All kernel public symbols carry the `host_` prefix, so the archive cannot interpose on the
engine DLL's symbols.

### Source-backed inputs (verified by reading `terrdata.c`, `terrsel.c`, `terrmap.c`, `zone_sc.c`, `parsgen.c`)

- Files: `<install>/common/maps/map<N>/terrain/`. `load_3d_terrain` also maps `default.rgb` (colours, tree files) which the solver does not
  use; the winter rule needs both `default-W.sec` and `default-W.rgb` to exist. **Season is an explicit input** (`winter`), set in the
  original from the campaign file's SEASON tag; the production wiring must derive it from the campaign data. The map number is the trailing
  digits of the directory above `terrain` (the Cuba flaw rule is `map_number == 2`).
- `default.sec`: version `u32`, width `u16`, height `u16`, min and max height `f32`, then `width*height` 48-byte sector records (counts,
  radius, three heights and six offsets into `terrain.ffp`, two unused), then the approximation list the solver does not read. Checked against
  the Lebanon file exactly: 16 + 76*92*48 + 4 = 335,636 bytes.
- `terrain.sim` (the fast grid, `get_3d_terrain_fast_elevation`): `i32` width and height, `f32` x and z grid sizes, then the `f32` samples:
  16 + 513*513*4 = 1,052,692 bytes for Lebanon. It is *not* derived from the point loader: if the file is absent the original generates it by
  sampling the point query on the shared record (`generate_terrain_simple_elevation`, 256 m) and saves it. Both retail maps ship the file,
  so the host refuses a map without a grid rather than generate one (a named gap).
- The derived facts (`max_x = width*2048 - 1`, `scaled_height_difference`, grid reciprocals) are computed under round-toward-zero, as the engine
  computes them.

### How the characterised behaviour is kept

- The triangle tests and `get_3d_terrain_point_data` are **verbatim copies** from base `6bbaa8197` (`terrelev.c` lines 145-1193 and 1208-2005),
  with only: the `host_` names; the engine's own build patches applied (N1 NaN position to the map corner, N3 degenerate face takes the up vector,
  X1 float-to-int by truncation); the ASSERT and `debug_log` not compiled; the off-map ASSERT replaced by a flag on the clamp it falls through to
  (its effect in the original is uncharacterised and source-only); and observation flags for the conformance record. The polygon-type table is
  taken from `terrtype.c` (every assignment agrees across both rendering modes; the 15 temporary generation types are explicitly unknown).
- The legacy find-fast **record** (pointers into sector memory) crosses as a plain `host_terrain_hint` value: sector, face index, offset of the
  face's references, triangle, type and stored normal. The query rebuilds the record the verbatim solver expects, runs it, and encodes the record it
  leaves. A fresh or cleared record is "no hint". The shared NULL record and the private records stay the campaign's, in the campaign's order, which is
  the intended way to preserve the cache and order behaviour (the 375 Lebanon and 385 Georgia primed-hint differences); that is a design intent,
  **not yet shown**: the conformance run is the evidence. A hint is checked against the map on every use (sector, face, reference offset, type,
  triangle, normal) so that a forged or stale one is refused; these guards are implemented by inspection only. Empirical conformance and a review of
  the unsafe parsing and pointer arithmetic are pending, and no memory-safety claim is made before them.
- The loader is written to check the files against their length before use (by inspection; not yet exercised): every offset and count, every point reference, every face of at least three points,
  every surface id. A file the original would read unchecked is an error here. This is stricter than the original, deliberately (a declared difference); the retail maps are
  expected to pass and the conformance run is what shows it.
- Rounding: the query sets round-toward-zero (as `get_terrain_3d_sector` does) and restores the caller's environment. Whether that is bitwise the
  engine's environment on the real build is exactly what the conformance test checks; it is not assumed.
- The Lua boundary (`host.terrain_open { terrain_dir, winter, map_number }` returns `provider(request)`): `{op="map"}`, `{op="point",x,z,hint}`, `{op="fast",x,z}`.
  Elevation, stored elevation, normal (3 numbers) and surface type (EECH `terrain_types` ids) cross by value; the hint is an opaque string `e2` + 88 hex
  digits that the campaign only stores and returns. An answer's `notes` table is diagnostic. A host that keeps no state returns no hint and ignores one it
  is given, and needs none of the EECH layout or kernel. Requests whose numbers are not exactly single precision are refused (nothing is rounded silently).
- The campaign's class, ground-vehicle suitability and route policy (`terr_db.c`, `ai_route.c`, ...) stay in the core.

### Checks to run (Root), none run yet

    cargo test --locked -p eech-world terrain               # hint codec and map-number unit tests
    EECH_TERRAIN_DIR=<private>/common/maps/map5/terrain \
    EECH_TERRAIN_ORACLE=R:/d3-evidence/original-smokes-a8effa9f3/lebanon/trace.jsonl \
      cargo test --locked -p eech-world --test terrain_conformance -- --nocapture

and the same with the Georgia directory (`map3`) and `.../georgia/trace.jsonl`. Without the two variables the conformance tests print `SKIPPED` and
pass; a SKIPPED run is not evidence. Expected from the oracle: sample hash `5019c81ccaa3bf8d` / primed `42669569d4a35a25` (Lebanon) and `0199bfae4fbe4ff5` /
`6f61a1158a8ab759` (Georgia), the 256 raw samples bit for bit, `private_primed` 375/385, `elevation_macro` 92/128, `repeat` 0. Limits: the sweep starts
with a cleared shared record (the oracle started with the campaign's, which could differ only in the first iteration's discarded primer); there is
**the a8 oracle has no fast-elevation data** (that fixture did not sample it); a trace from the fast-extended fixture (below) is needed for that comparison, and the older
trace only prints `FAST ORACLE ABSENT`. The node-value test checks the file parser only, not the interpolation. Cuba and off-map
behaviour remain source-only.

### Public-provider RED harness

`lua/terrain-provider.lua` boots the real campaign with a `terrain` provider (`reference`, `raised`, `generic`, `missing`, `bad_nil`, `bad_elevation`,
`bad_normal`, `bad_type`, `bad_hint`, `bad_error`) and observes only the public surface: the elevation of every mobile handed to `bind_mobile` (boot
placement), the elevations the campaign reports in the host's line-of-sight dialogue, and the provider's call counts. On the **unmoved** baseline the
`terrain` key is ignored (the boot reads only the keys it knows), so the run is the contract's RED: the provider is never consulted, `raised` equals
`reference`, `missing` boots, and no `bad_*` mode fails. After the extraction the script header lists the expected effects. Run it with an external
`eech-world` built from this source (the shipped a8 bundle has no `host.terrain_open`) against the immutable a8 DLL; the frozen 43-file bundle must
not be modified.

### Risks and what remains

- **Cost.** The original Lebanon 600 s run makes 46,387,862 terrain queries (about 7,700 per 100 ms frame; 77,953 at boot), 92% through the shared NULL
  record, and 82% of the hint-using ones hit the hint. A Lua provider call per query, each carrying a 74-character hint, is a real frame-cost question;
  batching or campaign-side caching without reordering is the likely follow-up and needs measurement.
- Uncompiled C and Rust (see above). The C solver keeps process-global statics, so the Rust wrapper serialises every query and sweep, on any map, under one lock (see the correction batch below).
- Production wiring is **not done**: the DLL client (replacing the solver body with a provider call), the `terrain` boot provider and its refusal when
  missing, the boot map facts and `get_3d_terrain_fast_elevation` consumers (`fw_move.c`, `mobile.c`), the hint stored in the legacy record (including
  the two `terrain_face == NULL` tests in `fw_move.c`), and removing the solver from `sources.txt`.
- **Named transitional DLL terrain users that remain** until G2/G3 are extracted in D3, and are removed by whole-D3 closure, not deferred to D5: the legacy
  loader and sector data (`terrdata.c`, `terrmap.c`), per-sector type triangles (`gd_cover.c`), `get_terrain_3d_types_in_sector` (`ss_updt.c`), the tree
  data (`terrobjs`/`en_cllsn.c`), `get_ground_impact_point`, and the model/device geometry of audit groups G3a to G3c.
- DCS: `land.getHeight`, `land.getSurfaceType` (mapped to EECH ids by a named table) and an explicit named workaround for the face normal and the
  stored-elevation distinction are the intended adapter; none is built or verified here.

## Fast-elevation oracle addition to the original fixture (source only; not built or run)

The original `get_3d_terrain_fast_elevation` (`terrsel.c`) had no recorded original behaviour, so the host kernel's fast grid had nothing to be compared
with except its own file. `eech_terrain_fixture.c` now also samples the **original** fast function in the original DLL at each of the sweep's
20,006 points (the six fixed and `count` generated), after everything the point queries do for that point:

- It is source-verified read-only: the function reads the loaded grid and the grid's header values, and its one write is a private test counter that
  nothing reads; it uses no find-fast record and no random number. It therefore cannot change the point queries, so the point hashes, counters, order
  and raw samples are expected to be exactly those of a8 (to be confirmed by the Root's comparison of the new build against the a8 point results).
- New report lines, separate from the old ones: `terrain-probe-fast-hash available=<0|1> count=<n> fast=<16 hex>` (an order-dependent FNV-1a over the
  bit patterns of x, z and the answer) and, for the first `emit` points, `terrain-fast-sample i= x= z= fast=` (hex bit strings; the driver keeps
  `fast` a string key). With no grid loaded nothing is sampled and `available` is 0.
- The conformance test compares the kernel's fast elevation with these samples and hash, bit for bit, when the trace has them.

Identity of the inputs the Root should record with the new oracle run, so the fast comparison is attributable: for each fresh private root, the size and
SHA-256 of `terrain.ffp`, `default.sec` (or `default-W.sec`) and `terrain.sim` (Lebanon: 17,043,748 / 335,636 / 1,052,692 bytes as read here), the
season used (summer), and the grid header (513 x 513, grid sizes 304 and 368 for Lebanon).

**Named compatibility gap, not hidden by this addition:** a map without `terrain.sim` is, in the original, given a generated grid (the point query sampled
on a 256 m lattice, saved to disk). The host does not do that, and no retail map exercises it, so full supported G1 behaviour is not claimed until this
is reconciled (generate it in the host from the same kernel, or state it unsupported with a review). Rebaselining is not the way to close it.

## Correction batch after the independent review of `f4b61a475` (source only; not built or run)

The review (`independent-host-backend-review-f4b61a475.json`) requested changes: one P1 and five grouped P2 findings. The Root's compile of the
unchanged recipe had already stopped at the first. All six are corrected below in source; **none is shown fixed until the Root builds and runs the
tests**, and no conformance, native or acceptance claim is made. Production DLL dispatch is unchanged. `rustfmt` is absent in the build image, so formatting
is unchecked.

1. **Build blocker.** A stray `#endif` after the copied function declaration (an artefact of the extraction from `terrelev.c`, where it closed the
   original's `DEBUG_TERRAIN` branch) is removed; the preprocessor conditionals are balanced again (17 `#endif` for 17 openers). No statement of the
   numerical body changed.
2. **Geometry views.** Every span of the sector record is now checked in 64-bit arithmetic with `offset <= total && length <= total - offset` before any
   pointer into the file is formed. The sector's points, references, faces and surface list are copied into one 4-aligned block owned by the kernel, so the
   solver's typed 16- and 32-bit reads are never made through a misaligned pointer into the file, whatever offsets the file holds. The kernel no longer
   keeps pointers into the sector and geometry files (the Rust map owns only the fast grid, which is read with `memcpy`). The copy assumes a little-endian host,
   as the original does.
3. **Concurrency.** Every query and every conformance sweep, on any map, runs under one process-wide lock in `terrain.rs` (`C_SOLVER`), as does closing
   a map. Maps are `!Send`, so each is created on the thread that uses it; the floating-point environment is per thread and is saved and restored by the
   kernel as before. Query order and arithmetic are unchanged.
4. **Hint provenance.** A hint now carries a 64-bit tag, a keyed FNV-1a over a per-instance random key and every field of the hint (sector, face, reference
   offset, triangle, type and the stored normal). A query refuses a hint whose tag does not match: one issued by another map, by an earlier instance of the same
   files, or edited in any field, including a normal that is still finite. A stored normal must also be finite with `y > 0.00001` (the solver guarantees it),
   so a forged zero normal can never reach the `0/0` of a fast hit. Legitimate hints are untouched: the cached normal, type and elevation bits come from the
   solver and travel unchanged. There is no table of issued hints. The tag is an integrity check against accident and staleness, not a secret; the structural
   checks still run on a hint whose tag matches. The Lua string is now `e2` + 88 hex digits. A host that returns no hint, or ignores one, is unaffected.
5. **Sweep bounds.** The sweep refuses a count outside 0..200000 or a negative emit before any arithmetic on them (the original probe's bounds), caps emit at
   256 and at the number of points, and the Rust wrapper returns only the samples that were produced (no invented zeros).
6. **Provider harness.** `lua/terrain-provider.lua` now accepts a refusal only for what it is: `missing` must fail at boot with an error that names the terrain
   provider; each `bad_*` mode must fail after its malformed answer was actually sent (`malformed_sent`), with an error that names the terrain provider
   (and `bad_error` with the provider's own message); healthy modes fail on any boot or frame error. Completed and requested frames are reported separately.
   On the unmoved DLL the outcome is a RED for the right reason: the provider is never consulted, so a healthy run asserts "never consulted", `missing` asserts
   that it booted, and a `bad_*` run asserts that the malformed answer was never sent or that nothing failed.

New tests (all in `crates/eech-world/tests/terrain_backend.rs`, on synthetic one-sector maps built from bytes, through the real loader, query and sweep):
odd offsets give bit-identical answers; offsets near `u32::MAX`, a wrapping reference offset, too many polygons, a two-point face, a bad reference and a bad
surface type are all refused; an edited valid hint (zero normal, another finite normal, another triangle or type, no or another tag) and a valid hint presented to
another map instance with identical structure are refused while the real one is accepted; the sweep bounds, including `i32::MAX`, negatives and `count 0, emit
256` returning six samples; the fast grid's checks and exact interpolation; and two maps created in two threads giving the single-thread answers over repeated
interleaved queries and sweeps. The private-data checks in `terrain_conformance.rs` are unchanged and still gated.

Proposed commands (Root): `cargo test --locked -p eech-world --test terrain_backend` and `cargo test --locked -p eech-world terrain`, then the gated conformance run as
above with `EECH_TERRAIN_DIR=R:/d3-evidence/backend-test-roots/lebanon/common/maps/map5/terrain` (Georgia: `map3`), and the Lua harness against the unmoved DLL.
Not done: the missing-grid generation of the original remains a named compatibility gap to reconcile; it is not declared unsupported here to ease acceptance.
