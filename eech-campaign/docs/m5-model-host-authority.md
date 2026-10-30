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
  covers), `SCENE` (the scene's name, whether the host has a model for it, whether that model has a collider, the radius, the two bounds) and `TRIANGLE` (scene, depth). No EECH pointer, offset, layout, entity index or legacy object number crosses it, and no hash
  of physical values: a scene is the EECH scene index, which the host and the campaign derive from the same data. `eech_engine_host` gained the field `model`.
* **Install, refresh, retain** (the Task 47 terrain pattern, with its lifecycle stated as it is): every entry that carries a callback refreshes it (`campaign_model_set_host`, called
  from `ENTER`), an entry without one leaves it (boot requires one: `EECH_ENGINE_MODEL_MISSING`, in `eech_engine.h`; the Rust engine offers the callback only when a provider is
  installed). The entry's `user` pointer names that entry's own context, which ends with the entry, so it is **never retained**: the native code always calls the callback with a
  NULL `user`, and a host must not depend on it. The Rust provider is a process-lifetime closure held in thread-local storage by `Engine::boot`; it is not reached through `user`.
  Re-entry and a second boot are refused by the engine's own state guards, so a retained function pointer is never called after the closure it reaches is gone.
* **The public contract is the scene, not the legacy loader's numbering.** A scene is the EECH scene index, whose name is compiled from the same header into every build. What a
  host declares about it is semantic: its compiled `name` (an echo that host and campaign mean the same scene by the same index; empty for the invalid index 0), whether the host
  has a model for it (`loaded`), whether that model has a collider (`collider`, only for a loaded scene), the radius and the two bounds. The numbers the legacy loader gives a
  scene's main and collision objects are private numbering: they are not sent, not asked for and not compared, so a host chooses its own storage and does not reproduce the EECH
  object table. The reference adapter adds its own store's numbers as `reference_main` and `reference_collision` for diagnostics only; the module ignores them, and the
  diagnostic build counts (never enforces) how many scenes' declared presence differs from what the campaign's own arrays imply.
* **Boot** (`campaign_model_install_scenes`, called by a patch at the end of `initialise_3d_objects`): asks the catalogue, then every scene 0 to `OBJECT_3D_LAST - 1`; refuses a
  catalogue of a different size, a non-finite number, a name that is not the scene's, a `loaded`/`collider` that is not 0 or 1, a collider on a scene that is not loaded, and a
  loaded invalid scene 0. The campaign's own scene arrays are **not read for acceptance**; the host's radius, bounds and triangles are never checked against them, so a host
  that authors different geometry is believed, not rejected.
* **Pointer lifetime**: callers keep the bounds pointer across loop iterations (`en_cllsn.c`, `fw_move.c`, ...). The tables are static, written once at boot from the host's
  answers, never written again and never a rotating cache, so every pointer is stable for the life of the process. They are a snapshot of host-authored data, not a second
  authority: the code that could produce a value of its own is not compiled.
* **Triangles are live and checked against the host's own declaration BEFORE any caller word is written**: the cursor is asked for every call, in the caller's order. `FOUND`
  (nine finite floats, exactly the host's bits) and `END` need a loaded scene with a collider; `ABSENT` needs a loaded scene without one; any of the three for a scene declared not
  loaded is a contradiction, and a `FOUND` for a scene declared without a collider stops the engine with `[model-provider:triangle.answer]` exactly as `END` and `ABSENT` do.
  `END` and `ABSENT` are the original's FALSE and leave the buffer **untouched**, distinct from every refusal, which is an error. No FALSE is ever substituted for a malformed or
  missing answer.
* **Rust**: `eech-engine` (`ModelQuery`, `ModelAnswer`, `ModelScene`, `ModelProvider`, `model_callback`; `Engine::boot` takes the provider), `eech-dc` (`boot { model = function (op, scene,
  depth) }`; the Lua bridge below), `eech-world` (`model_host.rs`: `host.model` and `host.model_provider (store)`). The Lua contract is documented at `MODEL_BRIDGE` in
  `eech-dc/src/module.rs`: `"catalogue"` returns `{ scenes }`; `"scene"` returns `{ name, loaded, collider, radius, bb, bb2 }`; `"triangle"` returns `{ result = "triangle", p } | "end"
  | "absent" | "refused"`. A float is a finite number (nearest single) or an exact `0x%08x` bit pattern. A script supplies **its own** function, so a host that is not the reference
  host (a DCS host) authors its own models and reproduces neither EECH's private array layout nor any physical-value hash.
* **Exact numbers, one read, bounded lists** (`MODEL_BRIDGE`). mlua turns any Lua number within `f64::EPSILON` of an integer into an integer (the patched Lua 5.1 `lua_isinteger`), so a
  tiny or subnormal physical value read as a `LuaValue` would arrive as 0 and a negative zero would lose its sign. The provider's answer is therefore never read as `LuaValue`s: a Lua
  bridge built once at boot reads every field ONCE (a stateful getter is not asked twice), checks its raw Lua type, and returns text: the number at 17 significant digits (an exact
  image of the double, -0 and subnormals included) that `nearest_f32` narrows, or the bit pattern. The catalogue count is a whole number tested in Lua (`1e-20` is not 0), `loaded` and
  `collider` are booleans, and the name is text of at most 127 bytes without a NUL. A list of six or nine is a table holding exactly the numbers 1..N (a number at every index, nothing at
  N+1, no key outside 1..N) or a text of exactly `11 * N - 1` bytes of N bit patterns; the length is tested before anything is parsed and the N slots are filled one token at a time, so no
  input can make the engine allocate more than N floats. This is the protected pattern `exact_numbers` uses for poses: nothing longjmps across Rust. What is NOT claimed: that the
  provider code leaves the x87 or MXCSR words untouched (Lua and libc formatting may raise exception flags); the narrowing's RESULT does not depend on the rounding mode, which is all the
  unit tests show, and they check only the rounding mode (`fegetround`). The reference host's requests are checked the same way (`STRICT_REQUEST` in `model_host.rs`): a scene
  or depth must be exactly a whole number.
* **The reference host** (`host.model`) is the f9 `ModelStore` (immutable, exact, with its reviewed numeric kernel; no new math), loaded from `host.args.root` when the campaign first
  asks (at boot); a root with no model assets refuses the boot as `NOT RUN: model assets absent`, never an empty database. Every campaign script of the repository passes
  `model = host.model` (31 scripts; the closed original-characterisation driver `model-characterization.lua` does not: see the variants).
* **FPU, RNG, order**: the callback runs inside the engine entry (round-toward-zero set); the host's kernel (kernel-only evidence, unchanged) requires the exception masks, sets and restores the caller's complete
  x87/SSE state around its arithmetic (reviewed, unchanged), and nothing in the exchange draws a random number, reads the clock or reorders the caller's calls (the accessors are
  replaced in place; the caller's `depth++` loops are untouched).
* **The complete outer floating-point state across the PUBLIC entries is NOT yet evidenced for this boundary, and is measured, not assumed.** The kernel's complete-state tests cover the
  kernel only. The shipped `ENTER`/`LEAVE` macros save the caller's environment with `fegetenv`, set round-toward-zero and restore with `fesetenv` (on the setjmp fatal path too);
  whether that gives back the x87 control and status words and the SSE word (MXCSR, flags included) on each toolchain, with the Lua model callback and its parser in between, is what the
  diagnostic build's `eech_engine_model_host_fpu` measures from the caller's side, on a successful entry and on a provider-refused one, at each initial rounding mode with every exception
  masked and nonzero exception flags set in both units. Nothing about the macros is repaired or assumed here; a difference is a finding to characterise.


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

* `crates/eech-dc/src/module.rs` unit tests (feature `fpu-test-support` of the engine, a dev-dependency): the REAL Lua bridge, asked under round-toward-zero as the engine's callback
  runs it, with the code under test required to leave the ROUNDING MODE (`fegetround` only; not the flags, masks or MXCSR) as it found it. A scene with `1e-20` (within EPSILON of 0), subnormal `1e-45`, `5e-324` and negative zero keeps every
  bit; bit patterns are exact (a subnormal pattern; NaN and infinity patterns and numbers refused, a `0X` prefix refused); a triangle is a table or a text of exactly nine and an
  extra at 10, an extra at 11 after a hole at 10, an extra far away, a string key, a key 0, eight numbers, a hole, ten patterns, six patterns, comma-and-space, semicolons, a hundred
  thousand patterns, a NaN, a bit pattern inside a table, a boolean and no `p` are each refused by name (upper-case digits and the exact text are accepted); every field of an
  answer, and every list index plus the bound, is read from a stateful getter exactly once (twelve reads for a triangle, thirteen for a scene); the catalogue count `1e-20`, `3026.5`,
  `-1`, `2^31`, text, NaN and nothing are refused, `0` and `3026` accepted; names of 127 and 128 bytes, an embedded NUL, a number for a boolean and a missing field are decided;
  `end`, `absent`, `refused` (with scene, depth and reason), an unknown result, a missing result, a non-table, a raised error and nil are each named.
* `crates/eech-engine/src/lib.rs` tests: the C-callback crossing with no engine booted. Every answer crosses bit for bit (including subnormals and `-0.0`); a scene crosses with
  its name, `loaded` and `collider`; a name of 128 bytes or with a NUL is refused; `END` and `ABSENT` write no float; a missing provider (`[model-provider:missing]`), a declining one, an
  answer of the wrong kind, an unknown operation and a panicking provider are each refused by name and write no reply field.
* `crates/eech-world/tests/model_store.rs` (appended) with `lua/model-provider-check.lua`: the real host binary on the synthetic installation; `host.model` (lazy, from the root) and
  `host.model_provider (store)` give exactly the store's answers for the catalogue, scene 0 (the zeroed, not-loaded null scene), the scenes (name, `loaded`, `collider`) and the triangle
  cursor (a triangle, an end, an absence and a refusal are all present and different), and refuse twenty-two malformed requests each by name (including `1e-20`, `-1e-20`, `1e-300`, NaN,
  infinity and a boolean, none taken for the integer 0); a root with no model assets is `NOT RUN`.
* The witness scripts, under the real host binary (`model_store.rs`): `json-lines.lua` (strict JSON: every control byte 0..31 and 127, the quote, the backslash, a multi-line report and
  UTF-8 round-trip through a real JSON parser; a non-finite number, a value that is not text, number or boolean, a failed write, a failed flush, a failed close, an existing output and a
  closed writer all raise); `model-authority.lua` with a stand-in `eech_dc` that asks the real provider what the engine asks at boot: thirty-seven malformed arguments (`frame_ms=0`,
  `seconds=inf`/`nan`/`1e9`/`0x10`, a duration that is more than 1000000 frames, `sweeps`, `seed`, `poison`, `los`, `probe_scenes` lists with holes, zero, duplicates, 65 items or more than
  1024 probes, `probe_depths` alone, an unknown mode or scenario) each refused by name **before any output is created or anything boots**; a healthy run whose four requested probes are
  all reported, with the native newlines intact in strict JSON; a probe that raises, and a probe that reports another scene, each fail the run (`requested probes failed`) while the
  evidence stays whole; a stuck clock (`no progress`), a requested sweep with no fixture (`los-fixture`), probes asked of a DLL with no probe method, an output that cannot be created and an
  output that exists are each failures, never coverage.
* `lua/model-authority.lua` natively (needs the real DLL and a retail root; the Root's): modes `reference` (healthy control, counted), `vanish`, `inflate` (host-authored geometry changes),
  `bounds_zero` (host-authored bounds change), `absent_all` (a host with no colliders, legitimately), `missing`, `decline`, `refused`, and `bad_nil`, `bad_catalogue`, `bad_name`,
  `bad_scene_nan`, `bad_collider_unloaded`, `bad_scene0_loaded`, `bad_kind`, `bad_nan`, `bad_count`, `bad_hex`, `bad_absent`, `bad_end_absent`, `bad_found_declared_absent` (every
  scene declared without a collider, then a finite FOUND: the native contradiction control) and `bad_end_declared_absent`. Each bad mode must stop the engine with its
  `[model-provider:...]` marker; a mode whose precondition was not reached is `UNREACHED` and fails the run. With `los=1` and `los-fixture` the script folds a line-of-sight sweep
  between the first opposed pair of live mobiles; the changed modes must move that fold relative to `reference`. Where it does not (no ray met a model), that is reported, not
  asserted. That the native code checks the presence declaration before it writes the caller's buffer is a property of the order of the checks in `campaign_model_triangle` (read, and
  exercised by the control above stopping the engine); the stop unwinds the engine, so the buffer cannot be inspected afterwards.
* **Which stage refused (`model-authority.lua`).** The first failure of a run is the PRIMARY one, recorded with its stage (boot, frame, probe, los or fpu) and message
  (`primary_stage`, `primary_message`, `primary_matched`). A negative mode is covered only when its answer was given AND the primary failure, at whatever stage, is the engine's stop on
  that mode's marker: a guard raised by a requested probe on healthy frames, a triangle refusal at boot, and a frame refusal are all recognised; a failure after the primary one (the
  engine has stopped, every later call fails), an unrelated failure, an accepted answer and an answer never given (`UNREACHED`) are not coverage. The `model_store.rs` stand-in controls
  play boot-first, frame-first and probe-first refusals (`decline` and `refused`), the never-asked, swallowed-then-unrelated and swallowed-and-accepted cases, and `missing`.
* **The complete outer floating-point state (`fpu=1`, diagnostic DLL).** For each `fpu_rounding` (0 nearest, 1 down, 2 up, 3 toward zero) the caller state is set (rounding in both units,
  every exception masked, divide-by-zero, underflow and inexact flags raised in the x87 status word and MXCSR) and the counters entry (ENTER/LEAVE alone) and the probe entry
  (accessors, model callback, Lua provider and bridge) are called; x87 control word, x87 status word and MXCSR are compared before and after, and the process's own environment is
  restored afterwards. In a mode that makes the host refuse, the first probe entry takes the fatal path: that is the refused entry measured (run once per rounding; the entries
  after it are POISONED and not judged). A changed word fails the run and nothing is repaired. The stand-in controls test only the script's asking and judging (every rounding on
  both entries, each word, the refused entry as the primary failure at its own stage, the unjudged entries after it, a changed word, a DLL that cannot measure, out-of-range arguments).
  The native measurement is the Root's.
* **Counters, named for what they are.** `reference_*` count what the reference host answered, before a mode rewrote it; `replied_*` count what the provider function handed to the
  campaign; `native_counters` (diagnostic build) are what the native code ACCEPTED (and the questions asked, the table reads and the presence diagnostic). None of them is authority
  proof on its own; with the poison and the LOS fold they are.
* With the diagnostic build: `poison=1` overwrites, right after boot, every campaign-side array the ORIGINAL accessors read (radius, both bounds, collision object index) with NaN
  and an out-of-table index; a poisoned run that gives the same observations as the unpoisoned reference run took them from the host. `probe_scenes=` drives the REAL public
  accessors for given scenes in the original fixture's row format (bit patterns, the return, how many of the nine words were written, the rounding mode before and after, the
  bounds asked twice for pointer and bit stability); every requested probe must succeed and report its own scene and depth or the run fails.

## Exact validation required (the Root's)

1. Build: the shipping module and the `model-host-fixture,los-fixture` variant (Windows and Linux), and the `model-fixture` variant (unchanged purpose). Confirm the staged
   `3dobjdb.c` and `3dobjvis.c` carry exactly the `D1a, D1b, D2, D3a, D3b, D3c` hunks (count-checked) with `model-fixture` off, none with it on, and that the two features together
   fail the build. `eech_engine.c`: exactly two hunks plus the appended diagnostic block, the one NUL and every other byte unchanged (the byte-safe splice was verified against the
   prior file).
2. Run the Rust unit tests (`eech-dc` needs the engine's `fpu-test-support` feature, enabled by its dev-dependency), `model_store.rs` (which drives the real host binary with the witness scripts and the stand-in engine), the existing `host_callbacks` and `load_dc` tests, clippy (`eech-dc` denies unwrap/expect/panic/indexing). Confirm the staged build still carries the strict patches and that a `model-fixture` build is untouched.
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
