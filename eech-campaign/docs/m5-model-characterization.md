# D3: characterisation of the original collision-model data and accessors (EEDCS-50)

**Status: SOURCE PREPARED, UNBUILT, UNRUN.** Nothing in this document has been compiled, linked, run or measured. It describes a test-only probe and what its first run must show.
It is not a production capability, not D3 acceptance and not evidence about any retail model. Authority: the live M5 goal (Confluence 11534337) and the migration ledger
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

`status` is computed from the loaded tables only, before any call: `absent`, `no_scene`, `invalid_index`, `null_surfaces`, `surfaces_outside_block`, `custom_object`,
`face_mismatch` (`number_of_faces` ≠ the sum of surface faces ≠ the sum of polygon-surface faces), `zero_face_surface`, `null_arrays`, `degenerate_face` (< 3 points),
`capacity_unobservable` / `capacity_short` (the object's faces, point list or surface points are not shown to cover what the decoder will read; the extent is the distance to the next
object's array, which the last object of a block does not have), `index_violation`, `empty`, `safe`. `fan_bound` is the sum of (face points - 2) and is a **safety bound for
how deep to call**, never compared with or substituted for the original's answer; the driver reports where the original actually first returned false. The end probe is made only when the
one-past surface read is inside the block (`end_read_backed = 1`); otherwise the row says `end_probe_unbacked` and that scene's end is **not** observed (explicit gap, not a pass).
The verdict's face walk uses the correct surface association, which equals the decoder's only when no surface is empty (those are refused). Refused models are reported with their
metadata; the retained raw output is the evidence. Nothing is repaired.

### Pure observer

No frame is stepped; no entity, table, pointer, model byte, retail file or random stream is written or drawn from. The use count reads each entity's shape through the ordinary getter
(the default for a type without a shape is the invalid index); it does not use the legacy updating getter. The probe makes no physical query other than the four accessors named.

## Commands for the Root (not run)

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
* every `model-tri` row has `return` 0 or 1; `return=1` rows have `writes=9`; `return=0` rows have `writes=0`; depths ascend from the page start by one; the walk ends in `model-end`
  (`original_false`, `end_probe_unbacked` or `beyond_safe_bound`) or is explicitly `model-capped`; a `capped` or `unbacked` model is not a full capture.
* `model tables unchanged` and `clock unchanged` checks pass; `model-state` before and after are identical.
* run 1 and a repeated run on a fresh root produce identical catalogue and geometry rows (the float bit patterns), or the difference is reported.

## Not done / next unmet measurements

Not built, not run, not reviewed. Next, in order: (1) the Root builds the fixture on a clean tree and verifies BUILD-INFO and flags; (2) the catalogue run on fresh verified Georgia and
Lebanon roots, with input post-checks; (3) the Root selects real vehicle and building scenes and records them; (4) the selected geometry runs, full triangle lists, repeated;
(5) only then host-owned read-only model data and the loader/geometry implementation (private legacy layout inside, public Lua semantic only), reusing these scenarios against the new
path. Remaining D3 scope outside this step: model hierarchy, viewpoint, device, muzzle and target-point services, and the whole terrain/spatial/environment/host-time set.
The whole-engine DLL still couples D3 to the original C model code; that coupling is not removed by this probe.
