# D3: the host-owned collision-model store (EEDCS-50)

**Status: SOURCE PREPARED, UNBUILT, UNRUN.** Nothing here has been compiled, linted, formatted, run or measured; the Rust, C and Lua were checked by reading only. This is the first
actual host-owned model component of D3. It is not production (the campaign DLL's tree and collision queries still run the original `initialise_3d_objects` and accessors), not the
Task's Done and not D3 acceptance. Authority: the live M5 goal (Confluence 11534337) and the migration ledger (14548994); Jira EEDCS-50 under Epic EEDCS-21. Base: branch
`codex/d3-model-geometry` after the reviewed original characterisation (`366622e0c` fixture, driver `309b3f050`, review closure recorded in `reference/d3-model-characterization/`).

## What it is

`eech_world::models::ModelStore` loads the collision-model data **from the retail disk assets, by the host**, not from the campaign DLL's loaded tables, and answers the
questions the original's entity/tree collision code asks of it:

* `scene (n)`: name, whether a scene was read, main object and effective collision object indices, the scene's radius and two bounding boxes (`bb` = `bounding_box`,
  `bb2` = `bounding_box2`, the one `get_object_3d_bounding_box_without_lines` returns) as float bit patterns, and the collision object's own counts, radius and bounding box;
* `triangle (scene, depth)`: the original accessor `get_object_3d_collision_object_geometry_triangle`'s answer for one depth: the nine float bit patterns, the end, the absence
  (collision object -1) or an explicit refusal naming why (never a guess);
* `summary ()` and `compare_original (path)`.

It is exposed to Lua as `host.model_store (root)` (src/model_host.rs). The returned store is an immutable, shared (`Arc`) userdata; every value it returns is a plain copy, so
nothing a script keeps refers into it; no pointer, offset or legacy layout is visible. Arguments are whole numbers within range or the call is refused before the data is touched.
A missing asset is `NOT RUN` (never a pass); a truncated, incompatible or inconsistent one is a refusal naming the file and the reason. There is no synthetic or empty fallback.

## Files and ownership

| File | |
|---|---|
| `crates/eech-world/src/models.rs` | the store: bounds-checked reader, loader (table, scenes, overrides), the decoder's integer walk, the query API |
| `crates/eech-world/csrc/model_step.c` | the original's numeric code, expression for expression, under its round-toward-zero guard: the object-bounding-box scaling of a triangle, and 3dsubobj.c's scene bounds and radius |
| `crates/eech-world/src/model_host.rs` | the Lua face and the comparison with a retained original report |
| `crates/eech-world/data/scene-names.txt` | the 3025 compiled scene names (line n = scene n), derived from `modules/3d/3dmodels.h` (SHA-256 `cdacdb32…caed9`) by `sed -n 's/^[[:space:]]*OBJECT_3D_INDEX(\([A-Za-z0-9_]*\)).*$/\1/p'` over its enum; a test regenerates the list from the header when the header is present |
| `crates/eech-world/lua/model-host-compare.lua`, `tools/model-host-compare-windows.ps1` | the driver and its runner: host process alone, no campaign DLL, no boot |
| `crates/eech-world/tests/model_store.rs` | loader/store/driver guards on minimal format fixtures |

The campaign still owns policy, selection and consequences; this is physical environment data only. The legacy layouts (the common face, point-list, surface-point and point arrays,
the consolidated surface block, the per-object spans) are private to `models.rs`.

## Formats implemented (derived from the original source, not guessed)

* **`bininfo.bin`**: the scene-name count (must equal `OBJECT_3D_OLD_LAST` = 2762) and each name, which must equal the compiled name (the original asserts this).
* **`3dobjs.bin` + `.pts` + `.sp`** (`initialise_3d_objects`): the seven header counts; per object a null flag or the counts (stored in the original as unsigned shorts: a larger file value is
  truncated there, the cursors advance by the truncated value, and the store does the same and counts it as `oversize_counts`), radius, the three reference counts, the two bounding
  boxes, the surfaces as the loader reads them (an **18-byte** record read into a 16-byte structure: flags word with the polygon bit and the point count in its top byte, the face
  count as the low half of an int read into a 16-bit field), the skipped normals, the face array, the point list and the skipped face normals. The point and surface-point arrays are the
  mapped files, indexed by the cumulative declared counts. Declared totals the objects exceed, or mapped files shorter than the cursors, are refused (the original would overrun).
* **`3dobjdb.bin` scenes** (`read_scene`, binary mode): the 16 magic words, the scene index, lights, cameras (two words each), links, sprite lights, the sub-object and animation
  counts, the main object, approximations, shadow data, **the collision object with the accepted C1 mapping (0 means -1)**, the main keyframes, dissolve and displacement records, the
  named indices and the recursive sub-object tree (keyframe 0 of each). Every record is walked by its own counts with checked sizes; the camera database after the scenes is not read.
  The tree must not hold more nodes or indices than the scene header declares (the original allocates exactly that), nor be deeper than 64.
* **Scene bounds and radius** are not stored on disk: `initialise_scene_bounding_sub_objects` computes them from the main object and the tree. `model_step.c` ports that code verbatim
  (rotation by keyframe 0, scale, the eight-corner boxes, the double-precision radius).
* **`OBJECTS/XXXX.EEO`** (binary-hex overrides, `read_object`): versions 1 and 2, replacing table object `XXXX` with its own allocations (`custom`). An unsupported version is left alone and counted,
  as the original returns before touching the object. A face of fewer than 3 points or a smoothed non-polygon surface is what the original rejects after reading: the object is kept and
  counted `eeo_invalid`, and decoding it is refused. Two files for one object are refused.
* **`OBJECTS/<dir>/*.EES`** (named scenes, versions -1 to -3, `read_custom_scene`): the same scene record in named mode (names instead of indices, `skip_schem` for version -3, ambient and
  distant lights for -2 and below). Names resolve as `get_object` does: the last path component upper-cased, `XXXX.LWO` a retail object by number, otherwise a per-directory history and a new
  object numbered after the table (room for 2000), loaded from `<dir>/<NAME>.EEO` found case-insensitively. A scene replaces the table's. A missing or rejected object, a truncated file
  or an exhausted room is a refusal (the original's load is fatal). An unsupported version, and a name no scene has, are skipped as in the original and **counted** (`ees_unsupported_version`,
  `ees_unresolved_names`); two files defining one scene are listed (`ees_duplicate_scenes`).
* **Surface consolidation** (the original's "reallocates surfaces memory"): every object's surfaces (the table's, then the overrides', then the new objects') in one block in object order.

## The decoder, and what "backed" means

`decode` is the original's integer walk, translated exactly: the unconditional first-surface read, the loop on `faces[count].number_of_points - 3 < depth`, the depth and reference
stepping, the surface advance on exhausted face counts with 0 points meaning 256, then the fan point 0 and the two points of the remaining depth through the point list, the surface-point
reference and the object's points. The float part is `model_step.c`'s `eech_model_triangle`.

The original reads the common arrays by pointer without limit; an object's own counts are not the limit of a read (when `number_of_faces` includes line surfaces the walk goes on into the next object's
faces: the 45 mixed-layout models). The store keeps each array as bounds-checked blocks and a read is allowed exactly as far as the **backed** block extends: the whole filled face block, the
whole point list, the whole mapped point and surface-point files, the consolidated surface block, or a custom object's own exact allocation. A read past that is `Refused (unbacked_*)`. So
a mixed or custom object is decoded where the original's read is inside memory the files back, refused where it would not be, and **never silently fixed**: the legacy semantics, including the walk into
the next object's data, are reproduced, not repaired. The original's one-past-the-end surface read at the end of a walk is not performed because its value is unused after the loop ends; if a
walk would use it (inconsistent counts) and it is unbacked, the query is refused.

## Floating point

The original runs under round-toward-zero. `model_step.c` requires the x87 and SSE exception masks to be all-masked (63 and 63; it never masks or unmasks), saves the caller's complete state (x87
environment with `fegetenv`, the whole MXCSR with `stmxcsr`), sets round-toward-zero for both units and reads it back, runs the arithmetic in a body that is not inlined into the entry, and restores the
caller's state on every exit (`fesetenv`, `ldmxcsr`, a read-back that must equal what was saved, else the answer is withheld). No guard state is held between calls. The load itself runs the scene-bounds
kernel, so a process with an exception unmasked cannot load the store (`Limit`). The tests show a query hands back the caller's control and status bits unchanged, and returns the same bits whatever
rounding mode the caller was in.

## What the original measurements are reused for, and what they do not cover

The retained original report (fixture `366622e0c`, three verified retail roots, 3025 scenes each: 769 safe, 2009 absent, 14 custom-object refusals, 45 face-mismatch refusals, 188 no-scene; the
selected real models MI24 8, KRIVAK 2643, Ka-50 2543, pipe 1213, Lebanon buildings 1424/1426, pylon 1241 absence) is the oracle `compare_original` reads. Nothing is computed from the store as an
expectation. **Not covered by any original triangle**: the 14 custom-object scenes and 45 mixed-layout scenes (the original refused them: it could not show their allocations backed). The store
decodes them; `compare_original` lists the host's answer for each as `compare-original-refused`, which is NOT a match. Resolving it needs a minimal feature-only original probe that calls the original
accessor for exactly the scene/depth ranges the store reports backed (the Root measures before any equivalence is claimed); no such instrumentation is added here.

## Commands for the Root (not run)

```
# the portable guards (loader refusals, caller FP state, overrides, the comparator, the driver); EECH_WORLD_BIN selects the host for a test binary built elsewhere
cargo test -p eech-world --test model_store

# the retail comparison (PowerShell; ordinary build output; a private retail copy that is only read; a new empty out directory)
tools\model-host-compare-windows.ps1 -Bin target\windows -Root <retail root> -Out <new dir> -Original <retained original catalogue report> -Scenes 8,2643,2543,1213,1424,1426,1241,188,1,1412,1165
```

Exit 3 with `NOT RUN: <reason>` is a prerequisite absent; any mismatch, refused store or empty comparison is non-zero.

## Not done

Compiling, running and reviewing anything here. The retail comparison, for all 3025 scenes and the selected geometry, on Georgia, Lebanon and a repeat. The custom and mixed-layout original
probe above. Anything in the production path: the campaign DLL's `initialise_3d_objects`, collision queries and tree queries are untouched and the DLL still runs the original loader. Still
whole-D3 scope: model hierarchy, viewpoint, device, muzzle and target-point services; entity collision; terrain, spatial, environment and host-time services; the removal of the physical
solvers and loaders from EECH_DC. Campaign policy, geography, routes, timers and consequences stay in EECH_DC. The textures, animations, cameras and lights the loaders also read are not physical
and are walked or skipped, not stored.
