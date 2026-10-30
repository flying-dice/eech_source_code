# D3: characterisation of the original collision-model data and accessors (EEDCS-50)

**Current status (30 October 2026): all 828 present models in the verified retail catalogue are characterized and match the prepared host loader/decoder.** Original fixture source `debf965cbff9346a2ada3e2cfb6691b572dba059` has closed its independent surface-safety correction review; Windows compilation and the Linux isolated Rust test pass, with 69 synthetic verifier expectations. The earlier six positive models and one absence retain all 215 original call rows byte for byte. The 59 former custom/mixed gaps add 1,662 TRUE and 59 original FALSE calls per root. Twelve bounded batches cover the remaining 763 models, adding 28,739 TRUE and 763 original FALSE calls per root, with no overlap or cap. All 3,025 scene metadata records and all observed returned bits match unchanged host component `f9c6a1e4c`. Across the combined evidence, present models account for 30,609 TRUE plus 828 end FALSE calls per root; the retained absence adds one FALSE. This is finite current-input coverage, not arbitrary/future model support.

All three case roots pass repository input checks before/after; before/after model state and payloads remain unchanged. Original triangle rows and complete host reports are byte identical across the three roots within each selected set/batch. Two Root aggregation errors occurred after successful native/input stages; both failed controllers remain exit 1, and separate reviewed read-only recoveries complete 0 without repeating native runs or changing raw files. See `../reference/d3-model-characterization/model-all-present-geometry-debf965cb-validation-20261030.json` and the independent evidence review (SHA256 `b8712c4bc6c0630e74eecea25bcec5bcdfd31a48d228f09483b46d4701037821`). The 2,009 absent scenes have metadata plus the retained absence-FALSE path; the 188 `no_scene` cases were not unsafely called. Historical preparation checkpoints below retain their original limits.

Production model loading/query dispatch, public host/native authority and refusal integration, required remaining physical collision solvers and hierarchy/viewpoint/device/muzzle/target-point services still need migration within D3. No production, full campaign/Tacview, Task50 or D3 acceptance is claimed. Campaign shape selection and targeting decisions/consequences remain core. Authority: the live M5 goal (Confluence 11534337) and the migration ledger
(Confluence 14548994); Jira [EEDCS-50](https://flying-dice.atlassian.net/browse/EEDCS-50) under Epic EEDCS-21 (D3). Candidate base: integration `6bbaa8197`
(accepted D2), branch `codex/d3-model-geometry`. The Task 49 files are not in this base and are not assumed.

## What this step is

D3 moves the physical terrain, spatial, model and environment services (and the native loaders and solvers they need) out of EECH_DC. The collision-model data is the model-data
dependency of the object/tree query migration. This step does the first thing the rules require, **characterise before moving**: it asks the ORIGINAL compiled accessors, through
host Lua -> public `eech_dc` -> original C, what the retail model data answers. It implements no loader, no parser, no geometry algorithm and removes no production dispatch.

## Evidence reused, and its limits

The Task 49 forest-geometry witness (original native `47a4af0d1`, stricter driver `b44d2c96f`; the closure records in `R:/d3-evidence/`) decoded **six triangles of a controlled,
scoped-down mesh** (quad, triangle, pentagon, surface switch, fan, depth, scaling) through the real original accessor, and matched every bit of the tree queries. It is a decoder
witness. It is NOT proof that a disk loader reproduces retail model data, and it covers no real vehicle or building. The native `3dobjdb.c`, `objects.h`, `3dobjvis.c` and
`buildpatches.rs` blobs are unchanged between the accepted base and those measured trees (verified by the Root, not re-measured here).

## What the source shows (read, not measured)

* **Consumers.** `en_cllsn.c` asks `get_object_3d_collision_object_geometry_triangle (INT_TYPE_OBJECT_3D_SHAPE, p, depth++)` ascending until it returns false, else falls back to
  `get_object_3d_bounding_box_without_lines`; the radius test reads `get_object_3d_radius`. The shape index is the entity's `INT_TYPE_OBJECT_3D_SHAPE`, a *scene* index (enum
  `OBJECT_3D_*`, 1 .. `OBJECT_3D_LAST`-1; the compiled label list is `object_3d_scene_names`).
* **Two levels.** A *scene* (`objects_3d_scene_database[i]`) holds `collision_object_index`, a scene radius and two scene bounds. The collision object is an entry of `objects_3d_data`
  (`object_3d`: points, faces, surfaces, its own radius and `bounding_box`). The decoder scales by the OBJECT's `bounding_box`, not the scene's. The probe reports both.
* **Effective absence.** `collision_object_index == -1` is absent. Binary `3dobjdb.bin` stored 0 for some scenes; the accepted C1 patch maps it to -1 (as `.EES` scenes always did).
  A remaining 0 on a loaded scene, or an index outside `1 .. total_number_of_raw_3d_objects`, is an invalid index.
* **Data layout.** After load, all surfaces are ONE reallocated block of exactly the sum of every object's surfaces. Faces, point lists and surface points are laid out back to back
  in the mapped / allocated files in object order; objects replaced by `.EEO` files carry `custom = 1` and separate allocations. The sizes of those blocks are `static` in
  `3dobjdb.c` and cannot be read by name.
* **The decoder is not defensive** (read in `3dobjdb.c`):
  1. it reads `obj->surfaces[0]` before testing anything (a NULL surface list is a crash; this is why the C1 patch exists);
  2. when the walk consumes the last face it advances `surface` and reads `obj->surfaces[number_of_surfaces]`, **one surface past the object's own**: the end probe (`depth == total
     triangles`, which `en_cllsn.c` always makes) reads memory the object does not own. It is safe only where another surface follows in the block;
  3. it trusts the face point counts, point-list references and surface-point indices; a surface with zero faces breaks its surface advance; a negative depth reads before the arrays.
  The probe therefore never calls it blind (below).
* **Synthetic data.** `eech_engine_prepare_installation` keeps a retail 3D database only if `cohokum/3ddata/textures.pal` exists; otherwise `eech_synth3d.c` writes a synthetic
  one in which every scene is a polygon-less box with collision object -1. A run on that data would show only absence. The driver and runner refuse it (NOT RUN) before
  anything is written.

Unknown from source (left unknown, not guessed): which scenes, if any, come from `.EES` / `.EEO` overrides in the private retail copy; the declared sizes of the last object in each
mapped block; how many triangles any real vehicle or building has.

## The probe (test-only, feature `model-fixture`)

Files changed or added (all gated; the production build, module and API are untouched):

| File | Change |
|---|---|
| `crates/eech-engine-sys/csrc/eech_model_fixture.c` | new: summary, catalogue, geometry, state digest, safety verdict |
| `crates/eech-engine-sys/csrc/eech_engine.c` | 71 lines appended under `#ifdef EECH_MODEL_FIXTURE` (four `ENTER (OPERATION_OBSERVE, NULL)` wrappers); the existing bytes, including the file's one NUL, are a verified prefix |
| `crates/eech-engine-sys/csrc/eech_engine.h` | declarations under the same gate |
| `crates/eech-engine-sys/csrc/{eech_model_walk.h,eech_model_hooks.h,eech_model_walkcheck.c}`, `csrc/eech_win32.c` (`eech_view_extent`), `build/{patches.rs,main.rs}`, `tests/model_walk.rs` | the backing observer and decoder-read safety walk described below (source prepared, unbuilt, unrun, unreviewed) |
| `crates/eech-engine-sys/{Cargo.toml,src/lib.rs,build/main.rs}` | feature `model-fixture`, FFI externs, `EECH_MODEL_FIXTURE` define |
| `crates/eech-engine/{Cargo.toml,src/lib.rs}`, `crates/eech-dc/{Cargo.toml,src/module.rs}` | feature chain; `model_summary`, `model_catalogue`, `model_geometry`, `model_state` on the Lua engine object; whole-number argument checks before the engine is entered |
| `crates/eech-world/lua/model-characterization.lua` | the driver |
| `tools/build-windows-model-fixture.sh`, `tools/model-characterization-windows.ps1` | build wrapper (same single-line adaptation as the aim/los wrappers) and the one-process runner |

Interface (no native pointer ever reaches Lua; all values are text rows):

* `engine:model_summary ()`: `scene_last`, `raw_objects`, `surface_block`, struct sizes, `page_scenes` (96), `page_triangles` (256).
* `engine:model_catalogue (first, count)`: one `model-scene` row per scene: label (compiled enum), main object, `succeeded`, effective `collision`, `status`/`detail` (the safety verdict),
  the **original** accessors' radius, `bb2` (`get_object_3d_bounding_box_without_lines`) and `bb` as `0x%08x` bit patterns, `uses` (loaded entities of the booted campaign whose shape
  is this scene, by entity type, up to four types), and, where a collision object exists, its loaded counts, flags, radius and `bounding_box` bits, surface/face/point sums, face point
  min/max, `fan_bound`, `end_read_backed`, the observed capacities and `max_depth`.
* `engine:model_geometry (scene, first_depth, count)`: one original call per depth, ascending, each into a fresh buffer pre-filled with a quiet-NaN sentinel; per call `return`, `writes`
  (how many of the nine floats changed) and, when written, the nine float bit patterns. Stops at the first false. A scene whose verdict is not `safe` / `empty` is `model-refused` and
  the original is **not called**; `absent` is answered by one original call at the requested depth (the original returns false before reading any object).
* `engine:model_state ()`: FNV-1a over the whole scene table, whole object table, whole surface block, the three base addresses and bound, and the arrays the decoder reads for every
  `safe` scene. The driver compares it before and after, with the clock.

### The safety verdict is not an expected answer

(This describes the verdict the `366622e0c` DLL measured. The source now prepared replaces its extent inference and face walk; see the next section. The retained raw output stays the
evidence for what that DLL measured.)

`status` is computed from the loaded tables only, before any call: `absent`, `no_scene`, `invalid_index`, `null_surfaces`, `surfaces_outside_block`, `custom_object`,
`face_mismatch` (`number_of_faces` ≠ the sum of surface faces ≠ the sum of polygon-surface faces), `zero_face_surface`, `null_arrays`, `degenerate_face` (< 3 points; no longer produced by the source prepared below),
`capacity_unobservable` / `capacity_short` (the object's faces, point list or surface points are not shown to cover what the decoder will read; the extent is the distance to the next
object's array, which the last object of a block does not have), `index_violation`, `empty`, `safe`. `fan_bound` is the sum of (face points - 2) and is a **safety bound for
how deep to call**, never compared with or substituted for the original's answer; the driver reports where the original actually first returned false. The end probe is made only when the
one-past surface read is inside the block (`end_read_backed = 1`); otherwise the row says `end_probe_unbacked` and that scene's end is **not** observed (explicit gap, not a pass).
The verdict's face walk uses the correct surface association, which equals the decoder's only when no surface is empty (those are refused). Refused models are reported with their
metadata; the retained raw output is the evidence. Nothing is repaired.

### Backing observation and the decoder's actual reads (SOURCE PREPARED, UNBUILT, UNRUN, UNREVIEWED)

The verdict described above is what the DLL `366622e0c` measured. It inferred an array's extent from the distance to the next object's array and so refused the last object of every
block, every custom object (14) and every model whose `number_of_faces` differs from its surfaces' face counts (45): 59 models whose original behaviour was therefore not characterised.
That inference is replaced, in source only, by observation of the real allocations plus an exact replay of the decoder's integer reads. This is a distinct **original-memory-safety change**:
it must receive its own independent safety review before the new fixture or its native accessors are executed.

What is observed (nothing is guessed from a neighbouring pointer; nothing is read before it is shown backed):

| Backing | Source | Fixture knowledge |
|---|---|---|
| shared face block, shared point-list block | feature-only hooks the staged `3dobjdb.c` calls (`csrc/eech_model_hooks.h`; strict count-checked patches `M1`..`M22` in `build/patches.rs`, applied by `build/main.rs` only with `model-fixture`) | the loader's own allocation size, every cursor position at every `fread`, the items each `fread` actually returned (a short read leaves a hole: bytes after it are allocated, not initialised), and the final cursor, reported before any override file mutates an object |
| file-mapped points, file-mapped surface points | `eech_view_extent` in `csrc/eech_win32.c` (mutex-protected `views[]`, exact base only, read-only look, no address leaves the process) | the live view's exact length; an interior pointer, released or unknown base is unobserved |
| surface records (common, custom, and the consolidated block) | patches `M12`..`M22`: the seven per-field `fread`s of each common record, the one 16-byte-per-surface `fread` of `read_object`, and the three points of the final consolidation (block allocated, one call immediately before each object's `memcpy`, block bound) | **which record bytes were actually filled, and the identity of the block the walk reads.** See "Surface provenance" below. The recomputed sum of the objects' surface counts (`surface_block_length`) is no longer used to size or trust any read; it is only reported |
| each custom (override) object | hooks inside `read_object`: the `fread` counts and, after `obj->custom = 1`, the counts it allocated by (points, polygon faces, point references, surface-point references) | exact allocation identity and filled extent, matched to the object's **current** pointers; one slot per **object index** (a static table of `MF_OBJECT_MAX` = 32768 slots, found in constant time, valid only in the generation that wrote it). The loader's table is (objects in `3dobjs.bin`) + 2000 + 1 entries, 11442 objects plus slot zero for the supported data (36 hex-named override files + 811 named objects = 847 custom loads, named ids reaching 10252), so every needed record fits with more than twofold margin and none is skipped for arriving late. A repeat read into the same object, an object that is not an element of the table, a changed pointer or an index past the fixed bound is a refusal **of that object alone** (`custom_object` with detail `ambiguous_record`, `unrecorded`, `unknown_object`, `stale_record`, `record_capacity`, `load_unobserved`); counts of each are reported and no other record is affected. Never a guess. The consolidated surface pointer of a custom object is not trusted from the record (consolidation frees and rebinds it); it is the block's |

#### Surface provenance (review finding P2 of `874aa5ae6`; SOURCE PREPARED, UNBUILT, UNRUN, UNREVIEWED)

The first version treated the surface block as fully readable and initialised from the recomputed sum of the objects' surface counts. That was not provenance: a short `fread` leaves
uninitialised bytes, the consolidation's `malloc` and `memcpy` copy whatever the source held, and the typed metadata reads happened before any alignment check. Now:

* **Common records** (`M13`..`M19`): each of the seven `fread`s that fill a record reports its result. A field counts only if `fread` returned 1. The fields overlap (`number_of_faces` is
  read as an `int` into the 16-bit member at byte 8, so it also writes bytes 10 and 11), so each field has a byte mask and a record is initialised only if the masks of the successful
  fields cover all 16 bytes. Per object the fixture keeps the number of LEADING initialised records; a hole never extends it. A field reported twice, out of range, or after the loop was
  reported over, or a changing surfaces pointer, marks that object (or the whole surface observation) inconsistent.
* **Custom records** (`M12`, and the extended `M10`): the surfaces allocation pointer, its size (16 bytes per surface) and the number of 16-byte records `fread` returned.
* **Consolidation** (`M20`..`M22`): `begin` records the new block and the record count it was allocated for; `copy`, called immediately before each object's `memcpy`, finds that
  object's source provenance (a custom object: its exact record, matched to the pointer and size; a common object: its tracked prefix, matched to the pointer the loop bound and to the
  common block the loop reported in `M5`) and marks the destination records initialised only for the source records that were; it also checks the destination is exactly where the running
  total says, the order, and the bound; `end` checks the block and the count the loader stored. A source that cannot be matched contributes **no** initialised record.
* **The gate.** Every surface read, in the walk, in the metadata statistics, and in the state digest, passes `mw_surface_gate` first: the surface block must have been observed (bound,
  not rebound, this object's pointer and count exactly where the consolidation put them, within `MF_SURFACE_MAX` = 262144 records, bookkeeping consistent), the record wholly inside the
  block, on a 16-byte record boundary, 4-byte aligned, and its bit set in the initialised bitmap. Only then is it dereferenced. The state digest hashes only initialised records of the
  observed block. The 52,517 surfaces of the supported data fit 5 times over; a block observed larger is refused for every object (`surface_capacity`), explicitly.
* **Refusals** use the existing vocabulary and name the cause in the new optional `surfaces=` field (`observed`, or `object_capacity`, `inconsistent`, `not_consolidated`,
  `surface_capacity`, `block_rebound`, `object_rebound`): an unobserved surface read is `capacity_unobservable` (detail `surfaces`), a misaligned or uninitialised one `capacity_short`,
  one outside the block `surfaces_outside_block`. A walk that is whole except for the next-surface read after the last face is reported as before (`end_probe_unbacked`). The summary row
  adds `surface_phase`, `surface_bad`, `surface_records`, `surface_initialised`, `surface_capacity`, `surface_common_records`, `surface_common_consumed`, `surface_dropped` and
  `surface_partial_objects` (objects with fewer initialised records than surfaces; Root expects 0 on healthy data, and `surface_records` to equal `surface_block`).
* **Cross-object walks** still work: the bitmap is per record of the final block, so a walk from one object's last record into the next object's records is allowed exactly when those
  records are initialised, and the per-object distinctions (each object's own prefix) are kept.

The hooks only record. They allocate nothing, free nothing, write no model byte, draw no random number, read no clock, change no entity and do no floating-point arithmetic. Every
`fread` they wrap is evaluated exactly as before and its result handed back unchanged. Each patch adds a call and changes no original statement, argument, order or value. With the feature off
none of the hooks, patches or tables exists. `get_object_3d_collision_object_geometry_triangle` and every other original numeric body are byte-identical.

The replay (`csrc/eech_model_walk.h`, a pure header with no engine type) follows the accessor's integer path exactly: the unconditional read of `surfaces[0]`; the loop on
`faces[count].number_of_points - 3 < depth`; refs, surface-point and surface stepping; the number of points 0 meaning 256; the **read of the next surface even after the last face** (the end
call's); then, for each triangle of a face, the three point-list entries, the three surface-point references and the three points. A walk may leave the object's own extent into the
following objects' shared arrays (a mixed layout) exactly when those reads are inside the allocation and initialised; otherwise it stops there. Checked integer arithmetic (no wrap), 2-byte
and 4-byte alignment, `allocated` against `initialised` bytes, and a step budget bound every walk. It computes **no triangle**.

Result per model: the longest run of depths `0 .. safe_depths-1` all of whose reads are backed, whether the end call is backed, and why the walk stopped if it did. It is reported in the
existing fields: `fan_bound` is the whole fan when the walk is complete and otherwise the safe depth count, `end_read_backed` is 0 when the end call (depth == `fan_bound`) is not backed,
so the existing gate rows apply unchanged: depth `fan_bound` is `end_probe_unbacked`, deeper is `beyond_safe_bound`. A model with nothing callable is refused with the existing finite
vocabulary (`null_surfaces`, `surfaces_outside_block`, `null_arrays`, `capacity_unobservable` for an unobserved array or the budget, `capacity_short` for an unbacked, uninitialised,
misaligned or wrapping read, `custom_object` with a detail as above). `face_mismatch`, `zero_face_surface`, `index_violation` and `degenerate_face` are no longer produced: a mismatch is walked
(the accessor does walk it), an index is judged by whether the read it causes is backed, not by whether it looks plausible, and a face of few points is not memory-unsafe in itself.

Faces of 0, 1 and 2 points are walked **exactly as the original steps them**, not refused and not given a safer interpretation. The accessor subtracts `number_of_points - 2` from its running
depth for every face it consumes, so a face of 2 points subtracts 0 and a face of 1 or 0 points ADDS 1 or 2; the loop is bounded by `number_of_faces`. With C_k the running sum of
(points - 2) before face k, a call at depth D stops at the first face k with D < C_{k+1} (that face always has 3 or more points, and the triangle it reads is D - C_k), or consumes
every face and returns FALSE. So the depths that return TRUE are exactly 0 .. P-1, P being the running maximum of C (never below 0); a face claims the depths from the maximum so far
up to its own C_{k+1} - 1 (none when it does not exceed it); and the first FALSE is depth P. The walk computes P and each claimed depth's reads from the face counts alone, with signed 64-bit
arithmetic, no assumption that an increment is positive, and the same bound by `number_of_faces`, the backing and the step budget; the `fan_bound` it reports is P (or, when a read is
unbacked, the first depth whose reads are not backed), a non-negative bound for how deep to call that is never compared with the original's answer. The previous refusal of such faces
was an implementation preference, not a property of the original source, and is removed. Any read these faces lead to is still checked against the backing, and a run of them is bounded
by what is backed and by the step budget.
`capacity_*` values now mean elements from the object's own pointer to the end of its observed allocation (-1: unobserved).

Optional fields (the closed driver and comparator tolerate extra `key=value`; no row type is added and the 15-status vocabulary is unchanged): catalogue rows and geometry headers carry
`backing` (`common`, `custom`, `-`), `walk` (`complete`, `end_unbacked`, `partial`, `refused`, `-`), `stop`, `stop_array`, and the extents the walk read (`faces_read`, `list_read`, `sp_read`,
`points_read`, in elements, all backed); the summary row carries the hooks' record (generation, phase, the shared blocks' allocated / consumed / initialised bytes, the views' lengths,
custom slots, records recorded / ambiguous / dropped for index capacity / unknown / late); the state row carries a `backing` digest of those records.

Gaps this makes SAFELY PROBEABLE (after the safety review and a build): the last object of each shared block; the 45 mixed-layout models up to the first unbacked read (including walks into
following objects' data that the loader initialised); the custom objects with an exact record. Still refused or explicit gaps: a model whose backing the hooks did not observe; a read
inside an allocation past the filled bytes (uninitialised); a custom object with an ambiguous, stale, unknown or missing record or an index past the fixed bound (Root will verify actual recorded count 847 expected and dropped 0 on the three roots); a walk that exhausts the step budget;
anything beyond the first unbacked read (reported as `partial`, never extrapolated). A `partial` walk is a bound for how deep to call, not a measurement of the model's whole fan.

Controls prepared (none run): the isolated `eech_model_walkcheck_run` (`csrc/eech_model_walkcheck.c`, 69 expectations over synthetic arrays in its own memory, no original code; the last 25 protect the surface pre-read contract: unobserved, outside, misaligned, off-boundary, missing or short bitmap, uninitialised first / next / end record with a poisoned unreadable pointer and poisoned bytes, the fread-result bookkeeping (short field, the spilling int read, a hole, duplicates) and a simulated consolidation feeding a cross-object walk, healthy and refused) driven by
`crates/eech-engine-sys/tests/model_walk.rs` (`--features model-fixture`) covers missing, short, uninitialised, below-region, misaligned and wrapping backing, the unbacked end read, the
256-point surface, surface advance and the budget, and faces of 0, 1 and 2 points: a 1-point face after a triangle, 1- and 3-point faces that leave no TRUE depth (no list read, backed end),
a 1-point face before a quad and a 0-point face before a 5-point face (the depths they claim and the exact list entries read), the unbacked list and face reads those lead to, and a
2^40-face run of 0-point faces stopped by the backing and by the budget. It proves the verifier's own bounds logic only; every expected value was derived by hand from the accessor's source,
not from a run.

Limits stated plainly: the number of items `fread` returned is taken as the filled extent (the bytes are not re-read); the 18-byte on-disk surface record's mapping to the 16-byte struct and
the layout of `face_surface_description` (points byte at offset 3, `number_of_faces` as the u16 at offset 8) are the walk's contract, checked by static size assertions and by the
retained evidence, not re-proved; the observer assumes the loader and the fixture entries run on the same engine thread; no part of this has been compiled or executed.

### Pure observer

No frame is stepped; no entity, table, pointer, model byte, retail file or random stream is written or drawn from. The use count reads each entity's shape through the ordinary getter
(the default for a type without a shape is the invalid index); it does not use the legacy updating getter. The probe makes no physical query other than the four accessors named.

## Commands and preparation-time expectations

```
# build (Git Bash, Docker with Linux containers), from a clean checkout of the exact commit to be characterised; a fresh, uniquely named volume gives a clean build
EECH_WIN_TARGET=eech-win-target-model-fixture-<tag> tools/build-windows-model-fixture.sh target/model-fixture/<dir>

# run (PowerShell; paths without spaces; a private retail copy; a new empty out directory)
tools\model-characterization-windows.ps1 -Bin target\model-fixture\<dir> -Root <private retail root> -Out <new dir> -Scenario lebanon_retail
# then, with scene numbers taken from that catalogue (the Root chooses; none is named here):
tools\model-characterization-windows.ps1 -Bin ... -Root <fresh root> -Out <new dir> -Scenario lebanon_retail -Scenes <n>[,<n>...] -MaxTriangles 4096
```

Exit 3 with `NOT RUN: <reason>` means a prerequisite is absent (no binaries, no fixture DLL, no retail database); it is never a pass. The Lua driver exits non-zero on any failed check.

## Expected shape and restoration (guards only; no physical answer is asserted)

* `provenance database=retail`; `catalogue` pages each carry exactly the requested number of `model-scene` rows, and the pages cover `1 .. scene_last-1` once.
* `catalogue-status` rows show positive (`safe`) and absent cases; the Root chooses real campaign-used vehicle and building scenes from `uses > 0` and the status, and records the
  families and counts actually found. If none is `safe`, that is a finding with the refusal reasons, not a fixture failure.
* every `model-tri` row has `return` 0 or 1, `writes` 0..9 agreeing with its nine coordinate words (`p` present exactly when `writes` > 0, `writes` equal to the words that differ from
  the buffer sentinel); depths ascend from the page start by one; a false return is followed at once by its `original_false` end. A false that nevertheless wrote, or a true that wrote
  fewer than nine words, is the original's own behaviour: it is recorded and left for the Root to read, not refused. The walk ends in `model-end` (`original_false`,
  `end_probe_unbacked` or `beyond_safe_bound`) or is explicitly `model-capped`; a `capped` or `unbacked` model is not a full capture.
* `model tables unchanged` and `clock unchanged` checks pass; `model-state` before and after are identical.
* run 1 and a repeated run on a fresh root produce identical catalogue and geometry rows (the float bit patterns), or the difference is reported.

## Diagnostic report validation (correction after the first original measurement; UNBUILT, UNRUN)

The first measurement (fixture `366622e0c`, three retail catalogues and selected geometry) showed the driver trusting its replies: two controls, made against the original DLL by
corrupting the reply text only, were accepted with `done failures=0`: a `safe` header followed by an invented `original_false` end with zero calls and no triangles, and a catalogue
page of the right size whose scene identities repeated. The native code, the safety analysis and the numeric readers were not at fault; the DLL's own argument checks reject
non-integral values (including `1.0000000000000002`), and that was not changed.

`lua/model-characterization.lua` now reads each reply as a report before believing it. A reply that fails writes a `report-refused` row (the offending rows are already in the output
before it), closes the file and stops with an error before any `done`. What is checked is shape, identity, order, cardinality and completion, never the geometry:

* **summary / state / clock**: exactly one well-formed row (whole-number sizes, 16-digit hashes, a clock with its three fields); an empty or missing reply is refused.
* **catalogue page**: the header echoes `first`, `count` and `scene_last`; exactly `count` `model-scene` rows follow, scene `first` upward each once and in order, none repeated, none foreign,
  each with the metadata the fixture always writes (name, whole numbers, a verdict from the finite set the fixture can write, float bit patterns for radius and bounds, `uses`/`use_types`
  agreeing, `absent` exactly when the collision index is -1, and for a callable model its `fan_bound`, `end_read_backed` and `max_depth`).
* **geometry page**: the header echoes `scene`, `first` and `count`, is well formed, and keeps the model's identity from page to page. An `absent` model is exactly one false, writeless call
  at the requested depth; a refused verdict is exactly one `model-refused` row naming the same scene, status and detail and no call. For a callable model the depths are contiguous from
  the page start; the reply ends in one terminal row for the same scene at the depth reached, with a reason its rows support: `original_false` only immediately after an original call
  that returned false at that depth (an invented one with zero calls is refused), `end_probe_unbacked` only for a model whose end read is unbacked at `fan_bound`,
  `beyond_safe_bound` only beyond the backed range and not at `fan_bound`; the closing row names the same scene and its call count equals the rows. A page with no terminal row must hold
  exactly the requested number of calls. Truncated, repeated, skipped, reordered, post-false, extra and contradictory rows are refused.

Explicit genuine outcomes are accepted as what they are (absence, refusal, an unbacked end, a capped walk, an original false at any depth, a partial write), with their limitations
unchanged. `fan_bound` is used only to check that the fixture's own gate rows are self-consistent; it is never compared with where the original returned false.

Portable guards (`crates/eech-world/tests/model_characterization.rs`) run the REAL driver under the real host Lua against a stand-in `eech_dc` that formats this report protocol from a
table of ten invented scenes and corrupts one reply per case. They are response-contract tests only: they say nothing about what a retail model contains and prove neither the C
analysis nor a loader. They cover a healthy run (three catalogue pages with a short last, multi-page geometry, absent, refused, empty, an unbacked end, a false on a later page's first row,
an end inside the first page, a capped walk), the two measured controls, about 60 malformed replies (catalogue, summary, state, clock, geometry; see the test table), accepted original
anomalies, a changed model state (still a failed check, as before), a refusal that stops the run before it asks anything further, and the argument / synthetic-database / existing-output
refusals before boot. (When written they had not been compiled or run; the Root's later runs are recorded in STATUS.md.)

Test-harness note (set-up only, after the first cross-machine runs): the guards run the host named by `EECH_WORLD_BIN` when it is set (a test binary built in a container bakes in a Cargo path
that does not exist on the machine that runs it); an explicit value that is not an existing file is an error, never a silent fall back to Cargo's binary, and without it Cargo's own binary is used.
The driver under test is compiled into the test (`include_str!`) and written unchanged into each run's own temporary directory, so no source path of the build machine is needed at run time.
The guard expectations, the stand-in protocol and the driver are unchanged by this. Unbuilt and unrun in this lane; the driver itself was measured natively by the Root (original `366622e0c` DLL).

## Not done / next unmet measurements

Original measurements exist for fixture `366622e0c` (see `reference/d3-model-characterization/` and STATUS.md); the diagnostic-driver correction above is UNBUILT and UNRUN, and no
independent review has closed it. Next: the Root runs the portable guards on Linux and Windows and replays the same two corruption controls against the corrected driver (red, then
green), reruns the three retail measurements with it and compares them with the first, and verifies the exact C/Rust native-source identity before reusing the `366622e0c` DLL (a driver-only
change is not a new native build and must not be relabelled as one). Still open, whole-D3 requirements: the custom-object and mixed-layout original cases (14 custom objects and 45
face-layout mismatches were refused, not decoded); the host-owned read-only model data and the loader/geometry implementation (private legacy layout inside, public Lua semantic only),
reusing these scenarios against the new path; model hierarchy, viewpoint, device, muzzle and target-point services; entity collision and the terrain / environment / host-time services
and the physical solver/loader removal from EECH_DC. Campaign policy, geography, routes, timers and consequences stay in EECH_DC. The whole-engine DLL still couples D3 to the
original C model code; this probe does not remove that coupling.

The first host-owned model component that follows this characterisation (source prepared, unbuilt, unrun) is described in [`m5-model-host-store.md`](m5-model-host-store.md).
