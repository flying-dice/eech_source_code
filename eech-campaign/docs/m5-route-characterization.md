# D3: original route-geometry observer and retained raw-model-reader disposition (EEDCS-50)

**Status: source written, NOT built, NOT compiled, NOT run, NOT reviewed.** Nothing in this document is a measurement. No original route has been observed, no count,
hash or coverage figure exists, and none may be quoted from here. Base `1399f9b76`. This is the **original characterisation only**: it moves nothing out of the DLL, changes no
production statement and prepares no cutover.

Scope authority is the live M5 goal (Confluence 11534337), the migration ledger and Epic EEDCS-21: physical model loading, triangle decoding and bounds services belong to the
host; campaign shape selection, target decisions, consequences and route-planning policy stay in EECH_DC. Hierarchy, viewpoint, device, muzzle and target-point services
remain required whole-D3 work and are listed, not assigned elsewhere.

## 1. Why routes

The DLL's model loader (`modules/3d/3dobjdb.c:1737`, `initialise_3d_objects`) still opens `3dobjs.bin/.pts/.sp/.spn/.spt` and fills `objects_3d_data`. The collision
accessors no longer read it (host-owned, closed). The earliest *campaign-required* reader left is the airfield route parser: `parse_waypoint_routes_from_object`
(`aphavoc/source/ai/faction/routegen.c:190`) decodes a line-face object into a waypoint tree, called from 30 call sites in `popread.c` (landing, take-off and holding routes of the airfield populations, and a single-route
function near `:2849`) while the campaign loads. It reads, from the raw object: the stored `int16` points and the bounds (`*= max / 32767`, the intrinsic coordinate decode), the
surface records (colour, point count, face count) and the line faces' point references through the surface-point indirection. Everything after that (primary and start
nodes, tree building, matching end slots, per-depth positions) is route-planning policy and stays core.

**The seam this implies** (a finding for the later extraction, not a design): the *intrinsic* decode (stored `int16` times bound over 32767, and the surface-point
indirection that turns a face-local reference into a node) is physical model data and must be host-owned; handing Core the raw `int16` and leaving `max / 32767` and the
indirection in Core would leave the authoritative array in the DLL. What Core needs from a host is, per route object, the decoded node positions, and the line faces
**in the parser's iteration order** (surface order, then face order) as resolved node pairs with their true material colour (Core interprets black/green; the colour is data).
The tree building iterates the faces repeatedly and its child order depends on that order, so the order is part of the contract. The identity of a route object is the airfield
**scene** plus the route **sub-object kind** (compiled names), not the legacy raw object number: Core's hierarchy search (`get_airfield_waypoint_route_object`,
`popread.c:2776`, `find_object_3d_sub_object`) returns `search.object_scene->index`, which indexes the array that is leaving. That lookup is hierarchy and stays visible
whole-D3 work; the observer records it (scene, kind, found, legacy number) so a host feed can be keyed and compared by scene and kind.

## 2. The observer (`route-fixture`)

Feature `eech-dc/route-fixture` (off in every shipping and every other fixture build; it composes with the host model and terrain providers because it observes the
unmoved parser, not the collision accessors). Files: `csrc/eech_route_hooks.h`, `csrc/eech_route_fixture.c`, `ROUTE_FIXTURE_PATCHES` R1..R7b in `build/patches.rs`
(staged only with the feature, strict count-checked like the others), `eech_engine_route_summary` and `eech_engine_route_call` (`csrc/eech_engine.c`), `Engine::route_summary`
and `route_call`, the Lua methods `engine:route_summary ()` and `engine:route_call (index)`, `tools/build-windows-route-fixture.sh`, and the driver
`lua/route-characterization.lua` with its local guards `tests/route_characterization.rs`.

What the patches do: they ADD a call and change no original statement, argument, order or value, do no floating-point arithmetic (floats cross as pointers and are copied as
bit patterns), draw no random number and keep no pointer. R3 records the call's arguments and the raw object's counts and bounds; R4 each node's raw `int16` point and the
position the parser has just decoded; R5 each line face as the first pass reads it (surface, face, the two face-local references, the two resolved nodes, the surface colour,
flags and point count); R6 calls the static observer R2 once at the parse end, which hands the final node, link (by node-array index), start and per-depth position and offset
tables to the hooks; R7a/b record the route-object lookup. It observes values the original already read; it builds no parallel parser and walks nothing the original did not.

What it can NOT show: a route object that was not looked up or parsed in this boot is absent; reaching routes in one boot is not proof that every route object of an
installation is parsed. It does not observe `match_end_slots`' inputs beyond the `slots` argument or `get_waypoint_inhangar_bits` (the airfield code that consumes the tables);
the end-to-end consumption is covered by the existing accepted campaign recordings, which a later cutover must keep byte-exact. Legacy numbering (`legacy_object`, node
numbers) is diagnostic only; comparison across the move is by scene name, kind name, call order and the physical rows.

### Completeness is explicit

Every pool is bounded and counted. `route-complete value=1` requires at least one recorded call, no dropped call or lookup, no hook outside an open call, no interrupted or open
call, and every call complete (ended, totals recorded, node rows = the object's point count, final rows = the totals, no row dropped). An empty boot, a full table or a cut
call is `complete=0`: the driver writes it, writes `done complete=false`, then FAILS. A report that does not fit its buffer is refused (never cut short); after any failure
the engine is not asked again.

### How it is run (the Root's, in order; none has been done)

1. Build a `route-fixture` module (`tools/build-windows-route-fixture.sh target/route-fixture/<dir>`; it adds exactly the one cargo argument, like the ground wrapper).
2. `eech-world lua/route-characterization.lua root=<retail root> out=<new file> scenario=lebanon_retail` and again with `georgia_retail`, each in a fresh process (the engine is
   one per process). The boot is campaign.lua's (same preparation, reference arithmetic profile, seed 1); no frame is run, so the accepted 100 ms step and seed are untouched.
3. Run twice and compare the files byte for byte (determinism), and compare the first call's `route-bounds`/`route-node` rows with a direct read of the same retail object
   only if a reviewed, independent oracle exists; none is introduced here.
4. Only then a cutover Task may use these rows as the original side of an original-versus-host comparison (same scenes, kinds, call order, node bit patterns, face order
   and final tables); any difference is classified under the baseline governance, never absorbed by a tolerance.

Local guards (`cargo test -p eech-world --test route_characterization`, the Root's to run) cover only the DRIVER: argument refusal before any side effect, no overwrite,
prepare -> select -> boot order, verbatim reports, writing then failing an empty or incomplete measurement, and no query after a failure. They use invented protocol text and
prove nothing about the observer or the parser.

## 3. Retained readers of the raw model array: finite disposition (read, not run)

Root's static census (`model-core-raw-object-source-census-1399f9b76-20261101.json`) finds `objects_3d_data` in 20 compiled files with 455 identifier hits. It is a **source
identifier census, not reachability**: compiled presence is not a runtime read, staged patches and guarded entry bodies are not resolved by it, and a headless boot that
shows zero counters proves only the paths it ran. The disposition below says what each group needs when the loader stops filling the array; every group must end as one of
(a) a host query, (b) true host metadata consumed by Core, or (c) an explicit stop before any unsupported physical read. **Zeroing the rows and calling present models absent
is none of these and is not allowed.**

| Group (census hits) | Reads | Reached (as read) | At cutover |
|---|---|---|---|
| `aphavoc/.../routegen.c` (1) | points, bounds, surfaces, line faces | campaign load, popread, per airfield | **(a)/(b), next**: host route-data service keyed by scene and kind (section 1). Observed by this change |
| `modules/3d/3dsubobj.c` (4), scene bounds built in `3dobjdb.c` (`initialise_scene_bounding_sub_objects`, `:1386`) | `bounding_box`, `bounding_box2` of each main and sub-object | at scene load (hierarchy construction) | **(b)**: the host's scene reply already carries the same bounds; the scene database must take its bounds from it. Part of the hierarchy service, still core-resident, visible |
| `3dobjdb.c` remainder (loader 101 hits; `add_animation_object` surface texture-animation flags; `debug_object`; custom EEO/EES import) | the loader's own fills; surface texture flags | load | The loader itself is what leaves (host-owned model loading). Texture-animation lists are rendering data: **(c)** or a host render-metadata answer, decided with the renderer boundary, not here |
| render and visibility: `3dobject.c` (19), `3dobjdrh/drw/drz.c` (64/53/12), `3dobjzdr.c` (20), `3dobjvb.c` (5), `3dobjvis.c` (33), `3dexplos.c` (32), `3dshadow.c` (51), `3dhoriz.c` (15), `3dwface1/2.c` (5/16), `3dfaces.c` (6), `3dscon.c` (9), `clouds/3dtrump.c` (5), `3dclouds.c` (3) | vertices, faces, surfaces, bounds for transform, draw, outcodes, shadows | renderer entry points (`draw_*`, `transform_*`, `get_object_3d_outcode`, instance visibility); the dedicated-server boot path calls no drawing, but that is **not proved dormant** for every campaign path | **(c)** by construction: these entries are not part of the hosted module's surface (the host owns rendering). The cutover Task must either remove them from the build or stop each entry clearly before reading an absent array; no silent empty-geometry draw. A reach counter is needed to know which, and is a separate bounded step |
| `hi_hud.c:328` (1) | `hud_display_model` pointer into the array | the player's Mi-24 avionics init | **(c)** (player-gunship UI, not the campaign), or a host display-model answer with the avionics boundary; the legacy pointer stays private either way |

Not in this census (separately read): `objects_3d_scene_database` readers (`popread.c` scene link objects, `damage.c` `succeeded`, `main_sc.c` menu links) and the
poly-line accessors, per the host-authority audit table; the viewpoint, device, muzzle, target-point and camera services of `3dobjdb.c`. They remain whole-D3 work and none
is assigned to another phase here.

**Next concrete D3 step after the Root's measurement:** the host route-data service and the loader detachment for the route readers, as one cohesive change, with the
reach counter above run first for the render group (the smallest measurement that turns "census" into "reached or not" for this build).
