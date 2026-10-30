# D3: host authority over the collision-model queries (EEDCS-50)

Status: **SOURCE PREPARED, UNBUILT, UNRUN, UNREVIEWED.** The shipping module's four model-data accessors are replaced by requests to the host; the default candidate boot and
query path is wired (nothing here is an optional, unused callback). Nothing in this document is a measured result. Deployment and merge stay gated by the validated
preceding delivery (Task 47 / PR107 and the integration it composes); this is preparation on the existing Task 50 / PR 110 branch, not a delivery, and Task 47's callback
source is used only as a pattern (nothing from it is cherry-picked or merged).

## What changed in the contract

The campaign DLL used to answer four questions from the model arrays it loads itself:

| Accessor | File | Now |
|---|---|---|
| `get_object_3d_collision_object_geometry_triangle` | `modules/3d/3dobjdb.c` | one step of the host's triangle cursor, asked live on every call |
| `get_object_3d_bounding_box` | `modules/3d/3dobjvis.c` | a pointer into an immutable table of the host's answers (filled at boot) |
| `get_object_3d_bounding_box_without_lines` | same | same |
| `get_object_3d_radius` | same | the host's radius from the same table |

The original array-reading bodies are **not compiled** into the module: strict, count-checked build patches (`eech-engine-sys/build/patches.rs`, `MODEL_HOST_PATCHES`,
applied by `build/main.rs`) replace the three small bodies and wrap the long one in `#if 0` (kept in the staged text only so the patch is auditable against the source). There
is no fallback to the arrays and no always-miss: a host that cannot answer, or answers wrongly, stops the engine with its reason.

* **C side**: `aphavoc/source/entity/mobile/mb_hostmodel.{h,c}` (new, in `sources.txt`). One synchronous callback, three operations: `CATALOGUE` (how many scene indices the host
  covers), `SCENE` (main and collision object index, radius, the two bounds) and `TRIANGLE` (scene, depth). No EECH pointer, offset, layout or entity index crosses it, and no hash
  of physical values: a scene is the EECH scene index, which the host and the campaign derive from the same data. `eech_engine_host` gained the field `model`.
* **Install-once, required at boot** (the Task 47 terrain pattern): the first entry that carries a callback installs it for the process (`campaign_model_set_host`, called from
  `ENTER`); boot refuses a host without one (`EECH_ENGINE_MODEL_MISSING`, in `eech_engine.h`); the Rust engine offers the callback only when a provider is installed.
* **Boot** (`campaign_model_install_scenes`, called by a patch at the end of `initialise_3d_objects`): asks the catalogue, then every scene 0 to `OBJECT_3D_LAST - 1`; refuses a
  catalogue of a different size, a non-finite number, and a scene whose **identity** (main object index, collision object index) differs from the scene database the campaign just
  loaded. Only the identity is compared. The host's radius, bounds and triangles are never checked against the campaign's arrays, so a host that authors different geometry is
  believed, not rejected.
* **Pointer lifetime**: callers keep the bounds pointer across loop iterations (`en_cllsn.c`, `fw_move.c`, ...). The tables are static, written once at boot from the host's
  answers, never written again and never a rotating cache, so every pointer is stable for the life of the process. They are a snapshot of host-authored data, not a second
  authority: the code that could produce a value of its own is not compiled.
* **Triangles are live**: the cursor is asked for every call, in the caller's order. `FOUND` writes the caller's buffer (nine floats, exactly the host's bits); `END` (a collision
  object is present and the list is over) and `ABSENT` (no collision object) are the original's FALSE and leave the buffer **untouched**. The two are distinct from every refusal,
  which is an error. `ABSENT` for a scene the host's own catalogue gives a collision object (and `END` for one it gives none) is refused as a contradiction.
* **Rust**: `eech-engine` (`ModelQuery`, `ModelAnswer`, `ModelProvider`, `model_callback`; `Engine::boot` takes the provider), `eech-dc` (`boot { model = function (op, scene, depth) }`,
  `call_model`), `eech-world` (`model_host.rs`: `host.model` and `host.model_provider (store)`). The Lua contract is documented at `call_model` in `eech-dc/src/module.rs`:
  `"catalogue"` returns `{ scenes }`; `"scene"` returns `{ main, collision, radius, bb, bb2 }`; `"triangle"` returns `{ result = "triangle", p } | "end" | "absent" | "refused"`.
  A float is a `0x%08x` bit pattern (exact) or a finite number (nearest single). A script supplies **its own** function, so a host that is not the reference host (a DCS host)
  authors its own models and does not reproduce EECH's private array layout or any physical-value hash.
* **The reference host** (`host.model`) is the f9 `ModelStore` (immutable, exact, with its reviewed numeric kernel; no new math), loaded from `host.args.root` when the campaign first
  asks (at boot); a root with no model assets refuses the boot as `NOT RUN: model assets absent`, never an empty database. Every campaign script of the repository passes
  `model = host.model` (31 scripts; the closed original-characterisation driver `model-characterization.lua` does not: see the variants).
* **FPU, RNG, order**: the callback runs inside the engine entry (round-toward-zero set); the host's kernel requires the exception masks, sets and restores the caller's complete
  x87/SSE state around its arithmetic (reviewed, unchanged), and nothing in the exchange draws a random number, reads the clock or reorders the caller's calls (the accessors are
  replaced in place; the caller's `depth++` loops are untouched).

## Variants (explicit)

| Variant | Features | Accessors |
|---|---|---|
| shipping module and every campaign run | none | host-owned (this change) |
| host-model diagnostic | `eech-dc/model-host-fixture` (+ `los-fixture` for the LOS witness; `tools/build-windows-model-host-fixture.sh`) | the same host-owned code, plus counters, the poison and the probe below. Never a shipping artifact |
| original characterisation | `eech-dc/model-fixture` (unchanged purpose, `tools/build-windows-model-fixture.sh`) | the ORIGINAL accessors, on purpose, to ask them what they answer; the host patches are not applied, no host is required, and the two model features exclude each other at build time |

## Source audit of direct readers of the model arrays (read, not run)

The accessors cover every use of **collision** data: `en_cllsn.c` reads bounds and triangles only through them (`:1313`, `:1565`, `:1651` bounds; `:1386`, `:1452`, `:1571`,
`:1657` radius; `:502/533`, `:751/841`, `:1976/2009` triangles, the last for forest trees), and the other ~40 callers of the bounds accessors (movement, floats, weapon
creation, cameras, the weapon database) are in the same position. `modules/physics/body.c` calls them too but is not in `sources.txt` (not compiled). The direct reads outside
`modules/3d` are none of them a collision bypass:

| Reader | Compiled / reached | Reads | Class |
|---|---|---|---|
| `ai/faction/routegen.c:219` (`parse_waypoint_routes_from_object`, called from `popread.c:2328..2426`) | compiled; reached when a population is read (landing and waypoint routes of keysites) | the raw object's points, faces and surfaces: a route network stored as line faces | **geography/route data in model form**; campaign route-planning policy stays core. **D3-open**: needs a host model-data service for line-object route networks before the DLL loader can go. Not a collision read |
| `ai/faction/popread.c:1344..1447, 2728, 2748, 3959` | compiled; reached when a population is read | `number_of_scene_link_objects` and `scene_link_objects[]` (linked sub-scene, offset, heading) | **hierarchy/instance data**. D3-open |
| `entity/mobile/weapon/damage.c:715` | compiled; reached when the damage tables are built | `objects_3d_scene_database[].succeeded` (does the destroyed variant exist) | scene-existence flag, not collision. D3-open (the host catalogue could answer it next) |
| `gunships/avionics/hind/hi_hud.c:324..328` | compiled; the player's Mi-24 avionics | the HUD display model | rendering/avionics. D3-open |
| `ui_menu/main/main_sc.c:916` | compiled; the main-menu scene | scene links for the menu | UI. Not reached by the boot path as far as read (not proved dormant). D3-open |
| `modules/graphics/textuser.c:2171` | compiled; terrain initialisation | the address of the forest-tree object (no read of its data in the compiled code; no other reader of `terrain_init.tree`) | terrain tree drawing; moves with the terrain solver (G3). D3-open |
| `get_object_3d_outcode`, `get_scaled_object_3d_outcode`, `get_object_3d_instance_visibility` (3dobjvis.c) | compiled; render paths | bounding data for frustum tests | rendering. D3-open |
| `get_object_3d_number_of_poly_lines`, `get_object_3d_poly_line_data` (3dobjdb.c) | compiled; Comanche/Hokum/Ka-50 MFD | line geometry of cockpit scenes | avionics. D3-open |

Headless does not prove any of these dormant; this table says what each reads and why none is a collision-bounds, radius or triangle bypass. If one turns out to be, it must
switch in this change.

## Controls PREPARED, NOT RUN

* `crates/eech-engine/src/lib.rs` tests: the C-callback crossing with no engine booted. Every answer crosses bit for bit (including subnormals and `-0.0`); `END` and `ABSENT` write
  no float; a missing provider (`[model-provider:missing]`), a declining one, an answer of the wrong kind, an unknown operation and a panicking provider are each refused by name
  and write no reply field.
* `crates/eech-world/tests/model_store.rs` (appended) with `lua/model-provider-check.lua`: the real host binary on the synthetic installation; `host.model` (lazy, from the root)
  and `host.model_provider (store)` give exactly the store's answers for the catalogue, scene 0 (the zeroed null scene), the scenes, and the triangle cursor (a triangle, an end,
  an absence and a refusal are all present and different), and refuse fifteen malformed requests each by name; a root with no model assets is `NOT RUN`.
* `lua/model-authority.lua` (needs the real DLL and a retail root; native, the Root's): modes `reference` (healthy control, counted), `vanish`, `inflate` (host-authored geometry
  changes), `bounds_zero` (host-authored bounds change), `missing`, `decline`, `refused`, and `bad_nil`, `bad_catalogue`, `bad_identity`, `bad_scene_nan`, `bad_kind`, `bad_nan`,
  `bad_count`, `bad_hex`, `bad_absent`, `bad_end_absent`. Each bad mode must stop the engine with its `[model-provider:...]` marker; a mode whose precondition was not reached is
  `UNREACHED` and fails the run, never counted as covered. With `los-fixture` the script folds a line-of-sight sweep between the first opposed pair of live mobiles; the changed
  modes must move that fold relative to `reference`. Where it does not (no ray met a model), that is reported, not asserted.
* With the diagnostic build: `poison=1` overwrites, right after boot, every campaign-side array the ORIGINAL accessors read (radius, both bounds, collision object index) with NaN
  and an out-of-table index; a poisoned run that gives the same observations as the unpoisoned reference run took them from the host. `probe_scenes=` drives the REAL public
  accessors for given scenes in the original fixture's row format (bit patterns, the return, how many of the nine words were written, the rounding mode before and after, the
  bounds asked twice for pointer and bit stability). The counters (`catalogue`, `scene`, `triangle`, `found`, `end`, `absent`, bounds/radius table reads) are read together with
  the poison; a counter alone is not authority proof.

## Exact validation required (the Root's)

1. Build: the shipping module and the `model-host-fixture,los-fixture` variant (Windows and Linux), and the `model-fixture` variant (unchanged purpose). Confirm the staged
   `3dobjdb.c` and `3dobjvis.c` carry exactly the `D1a, D1b, D2, D3a, D3b, D3c` hunks (count-checked) with `model-fixture` off, none with it on, and that the two features together
   fail the build. `eech_engine.c`: exactly two hunks plus the appended diagnostic block, the one NUL and every other byte unchanged (the byte-safe splice was verified against the
   prior file).
2. Run the Rust unit tests, `model_store.rs`, the existing `host_callbacks` and `load_dc` tests, clippy (`eech-dc` denies unwrap/expect/panic/indexing).
3. Natively, on the three fresh roots: boot with `reference` (zero unexplained differences from the frozen baseline, deterministic pack, M4 regression pack), the `missing`, `decline`,
   `refused` and `bad_*` modes, and the changed modes against `reference` with `los-fixture`; the poisoned run against the unpoisoned one; the probe rows for the six-positive scenes
   and a sample of the 828 against the retained original rows (the f9 oracle, not relabelled as original). Classify every difference under REVIEW.md; no golden is re-recorded.
4. Boot time and per-query cost of the Lua provider (the triangle cursor is O(depth) in the store as it is in the original; an LOS sweep asks it per ray): measure, do not assume.

## Remaining coupling and uncertainties (D3 stays open)

* The physical solvers that consume these answers (sector scans, the attitude/position transform, the line-triangle decision in `en_cllsn.c`) are still campaign code: this is an
  **intermediate** boundary. They move to the host under Task 49 / G3 within D3; there is no duplicate-loader end state and no D5 deferral.
* The DLL still loads the same files for hierarchy (sub-objects, viewpoints, mounts, devices, muzzles, target points), the readers in the audit table, and the instance
  construction. Removing the native loaders from the DLL is the next D3 step; D3 cannot be accepted before it.
* Whether the synthetic installation `prepare_installation` writes (`eech_synth3d.c`) is accepted by `ModelStore::load` is **not established**: the writer says it mirrors the
  loaders' reads, but the host store has never been given such a root. Scripts that boot a synthetic root now fail at boot (`NOT RUN` or a store refusal) until that is shown or the
  script supplies a provider of its own; nothing substitutes an empty database. The Root's first build run answers it.
* A script that already boots with a provider of its own (DCS) must supply `model`; there is no default in the module.
