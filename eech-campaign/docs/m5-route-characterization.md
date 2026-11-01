# D3: original route-geometry observer and retained raw-model-reader disposition (EEDCS-50)

**Status: observer SOURCE ONLY - not built into any engine, not run against any retail data, not accepted.** Nothing in this document is a measurement of the original: no original
route has been observed, no count, hash or coverage figure of real routes exists, and none may be quoted from here. Only the Lua driver and the output helper have been exercised,
locally in a working tree against invented C-shaped reports (section 2). The first REAL captures (source `d10dd8cc0`, Windows, 1 November 2026) completed the boot and every route query but FAILED
completeness on identity alone (section 2, "First real captures"); they are retained private partial originals, not accepted measurements. Base `2ec202036`. This is the **original characterisation only**: it moves nothing out of the DLL, changes no
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
unmoved parser, not the collision accessors). Files: `csrc/eech_route_hooks.h`, `csrc/eech_route_fixture.c`, `ROUTE_FIXTURE_PATCHES` R1..R8 in `build/patches.rs`
(staged only with the feature, strict count-checked like the others), `eech_engine_route_summary` and `eech_engine_route_call` (`csrc/eech_engine.c`), `Engine::route_summary`
and `route_call`, the Lua methods `engine:route_summary ()` and `engine:route_call (index)`, `tools/build-windows-route-fixture.sh`, and the driver
`lua/route-characterization.lua` with its local guards `tests/route_characterization.rs`.

What the patches do: they ADD a call and change no original statement, argument, order or value, do no floating-point arithmetic (floats cross as pointers and are copied as
bit patterns), draw no random number and keep no pointer. R3 records the call's arguments and the raw object's counts and bounds; R4 each node's raw `int16` point and the
position the parser has just decoded; R5 each line face as the first pass reads it (surface, face, the two face-local references, the two resolved nodes, the surface colour,
flags and point count); R6 calls the static observer R2 once at the parse end, which hands the final node, link (by node-array index), start and per-depth position and offset
tables to the hooks; R7a/b record the route-object lookup (and the entry it returned); R8 records the caller context at every parse call site (see "Route identity"). It observes values the original already read; it builds no parallel parser and walks nothing the original did not.

What it can NOT show: a route object that was not looked up or parsed in this boot is absent; reaching routes in one boot is not proof that every route object of an
installation is parsed. It does not observe `match_end_slots`' inputs beyond the `slots` argument or `get_waypoint_inhangar_bits` (the airfield code that consumes the tables);
the end-to-end consumption is covered by the existing accepted campaign recordings, which a later cutover must keep byte-exact. Legacy numbering (`legacy_object`, node
numbers) is diagnostic only; comparison across the move is by scene name, kind name, call order and the physical rows.

### Completeness is explicit, and joins every cohort a call promises

Every pool is bounded and counted. `route-complete value=1` requires at least one recorded call, no dropped call or lookup, no hook outside an open call, no interrupted or open
call, and every call complete. A call is complete only if it ended, has totals, lost no row, has ONE route identity, and **every cohort it promises was captured whole**: the
requested slots (stored = requested); the first-pass faces (the rows equal the sum of the raw object's surfaces' face counts, the parser's own loop bound); every final node; every
parent and child link the final rows count (a list clipped at the parser's `MAX_WAYPOINT_LINKS`, or a target outside the node table, is incomplete); every start; and every depth
with all the offsets its count declares (a positive count with no offsets is a shortfall). The expectations are the original's own counts, never a second route solver's answer.
A report that does not fit its buffer is refused (never cut short).

**Route identity is the caller's context, not an inference.** The first real captures showed that the earlier rule (the scene and kind of the lookups that returned the parse's raw
number, accepted only if exactly one) cannot work: several keys legitimately share one model, so a raw number does not name a key. The observer therefore records the ACTUAL parser
caller's context. At every original parse call site popread holds the route entry it selected (the result of its own `get_airfield_waypoint_route_object` for a scene and kind) and has
just taken the raw number it passes from it (`raw_object_index = <entry>->index;`, 26 sites, or the inline `route->index` of the two troop-route calls, where `route->index` is evaluated once as the argument of a private identity-returning helper, `eech_rf_caller_index`, whose result is the parser's argument). Patch R8 hands the fixture
that entry and number at each of those places (a block-local `eech_rf_caller (entry, number)`); R7a/R7b also hand it the entry each lookup returned, kept as a private comparison
token that is never reported. The next `eech_rf_begin` consumes the context and looks the entry up among the lookups' entries, which gives the (scene, kind) THE CALLER CHOSE; keys that
share a model have separate entries and are each `unique`. Nothing is recomputed, no original statement, argument, order or value changes, and the raw number remains diagnostic
(`legacy_object`, `caller_raw`). The states are exact: `unique` (the entry was returned by lookups of one key and the number agrees), `missing` (no context was registered for the
parse), `mismatch` (the caller's number is not the parse's argument, or not the lookup's), `unmatched` (no lookup returned the entry), `conflict` (lookups returned the entry under two
keys). A context never consumed by a parse is `stale`: the summary counts `contexts` and `stale`, requires every context to be consumed by one parse or be stale, and is not complete
while any is stale. All of these are recorded in full and are incomplete, so the measurement fails explicitly instead of a key being guessed. The lookup table holds distinct lookups.

**First real captures (source `d10dd8cc0`; private audit, raw data never committed).** One natural boot each of Georgia and Lebanon completed the boot and answered every route query with
no malformed report and no dropped, capped, orphan, open or interrupted capture, all four full retail input checks 0 and all 57 payloads unchanged; both runs exited 1 on completeness
alone. Georgia: 704 parser calls, 430 unique and 274 ambiguous identities. Lebanon: 528 calls, 394 unique and 134 ambiguous. All other cohorts (slots, nodes, faces, finals, links, starts,
depths, offsets) were retained. These are useful FAILED partial originals: the ambiguous identity came from the reverse-inference rule above, not from the geometry, and the
corrected observer must be re-run on fresh installation roots. The expectation, to be verified by the Root after a corrected capture, is that corresponding full geometry and graph rows are
byte-identical to those partial captures and only the documented identity and completeness metadata differ; no such comparison exists yet.

### The driver validates the whole native grammar

`lua/route-characterization.lua` checks the summary and every call line by line against the grammar `csrc/eech_route_fixture.c` emits (name, key set and order, single spacing;
exact decimal integers and lower-case 8-digit words in bounded domains; compiled names; the identity state as the whole literal token `unique`, `missing`, `mismatch`, `unmatched` or `conflict`), the row counts against the counts each report declares, and the cross-report joins (the
summary's counters, pools and verdict against the calls and against the lookup rows; the pinned table capacities). `tests/route_characterization.rs` checks that its capacities and
its row grammar are the C source's. It distinguishes three outcomes:

* **measured**: well formed, the observer's verdicts complete, and every complete call's rows support its claim (numbering, order, the faces' surface sequence, link counts per
  node, starts, depths and offsets against the call's own counts; each face-local reference below its surface's point count; the repeated faces of one surface carrying one surface
  record (colour, flags, polygon bit, point count, face count); and a face-local reference resolving to the same node wherever it recurs within a surface). These are consistency
  relations among captured records, not a recomputed route. Only this passes. No route answer is recomputed and no numerical geometry oracle exists.
* **well-formed but incomplete** (an empty boot, a full table, a missing, mismatched, unmatched or conflicting caller context, a stale context, a clipped list, an unended call): the engine's honest answer, not a failure of the
  engine. The driver therefore still reads the remaining recorded calls once each, for diagnosis, then writes `done complete=false` and FAILS. This draining is deliberate.
* **malformed** (a line the grammar cannot produce, an out-of-domain or contradictory number, rows that do not match their declared counts, a summary that disagrees with the
  calls, a `complete=1` the rows do not support): the text is already written verbatim; a `malformed` row and `done complete=false` follow, the driver FAILS, and the engine is
  not asked again. An engine call that itself fails stops the questions at once.

### The output is created exclusively

`host.create_exclusive` (`src/exclusive_output.rs`, registered next to the other host functions; `json-lines.open` and every existing client are unchanged) uses the platform's own
create-new open (`O_CREAT | O_EXCL`, `CREATE_NEW`), which atomically refuses ANY existing target: a file whatever its permissions, a directory, a link (also a dangling one). Lua
5.1's `io.open` cannot (`"w"` truncates; a read-open check races and misses unreadable files) and an `x` mode is not assumed. Writes and flushes are checked, and the close is checked in two separate steps, both attempted and both reported: the data sync (`sync_all`, a durability step that is NOT
the close) and then the descriptor / handle release by the platform's own close (`close`, `CloseHandle`), after the raw descriptor is taken out of the `File` so that its drop never
closes a second time (a close interrupted by a signal has still released the descriptor on Linux and is not a failure). The release is attempted even if the sync failed. The owned output
is closed on every exit, and a failed close is reported together with any primary failure. The helper's unit tests exercise the real platform close with a value that is not a
descriptor / handle (the operating system refuses it); the driver tests' "close failed" cases use a Lua stand-in and show only how the driver reacts to a failed close.

### How it is run (the Root's, in order; none has been done)

1. Build a `route-fixture` module (`tools/build-windows-route-fixture.sh target/route-fixture/<dir>`; it adds exactly the one cargo argument, like the ground wrapper).
2. `eech-world lua/route-characterization.lua root=<retail root> out=<new file> scenario=lebanon_retail` and again with `georgia_retail`, each in a fresh process (the engine is
   one per process). The boot is campaign.lua's (same preparation, reference arithmetic profile, seed 1); no frame is run, so the accepted 100 ms step and seed are untouched.
3. Run twice and compare the files byte for byte (determinism), and compare the first call's `route-bounds`/`route-node` rows with a direct read of the same retail object
   only if a reviewed, independent oracle exists; none is introduced here.
4. Only then a cutover Task may use these rows as the original side of an original-versus-host comparison (same scenes, kinds, call order, node bit patterns, face order
   and final tables); any difference is classified under the baseline governance, never absorbed by a tolerance.

Local guards (`cargo test -p eech-world --test route_characterization`, and the helper's unit tests `--lib exclusive_output`) cover only the DRIVER and the output helper: argument
refusal before any side effect, prepare -> select -> boot order, verbatim reports, a coherent C-shaped positive and 51 damaged-report controls plus 4 near-miss identity states, the well-formed-incomplete drain,
no query after a malformed report or a failed engine call, the exclusive create and its IO failures, and drift guards against the C source. The reports are invented C-shaped text
with no real route geometry; the guards prove nothing about the observer or the parser.

### Retained evidence of the old driver's false acceptance (working-tree checks, not a frozen-commit run)

The driver of `dc4c7c41b` (`route-characterization.lua` blob `cedefbc48ec1...`) was exercised for the first time by a focused local Linux run on the Root's free Rust/Lua-only cache
(`eech-linux-d3-collision-driver-5416adb3b-20261029`, image `sha256:4a5618cb...`, `cargo test --release --locked -p eech-world --test route_characterization`). Step 1, the test
file as committed plus one added test: **0 passed, 5 failed**, a test-setup defect (the test's Lua state had no `debug` library, which the driver uses to find its own path), so the
committed positive had never run. Step 2, only that harness state repaired (blob `69ef0d2d...`, retained as `reference/d3-route-characterization/old-driver-red-test-69ef0d2d.rs.txt`):
**4 passed, 1 failed**, the failure being the old driver ACCEPTING a call that declares `points=2` and `faces=1` and carries no node, face or final row (the old positive fixture did
exactly that). This is the actual red for the missing-body acceptance; the new driver rejects it as malformed (`the_report_that_declares_geometry_but_carries_none_is_malformed_not_measured`).
Details and the post-correction results are in `reference/d3-route-characterization/route-driver-focused-checks-20261101.txt`.

## 3. Retained readers of the raw model array: finite disposition (read, not run)

Root's static census (`model-core-raw-object-source-census-1399f9b76-20261101.json`) finds `objects_3d_data` in 20 compiled files with 455 identifier hits. It is a **source
identifier census, not reachability**: compiled presence is not a runtime read, staged patches and guarded entry bodies are not resolved by it, and a headless boot that
shows zero counters proves only the paths it ran. The disposition below says what each group needs when the loader stops filling the array; every group must end as one of
(a) a host query, (b) true host metadata consumed by Core, or (c) an explicit stop before any unsupported physical read. **Zeroing the rows and calling present models absent
is none of these and is not allowed.**

| Group (census hits) | Reads | Reached (as read) | At cutover |
|---|---|---|---|
| `aphavoc/.../routegen.c` (1) | points, bounds, surfaces, line faces | campaign load, popread, per airfield | **(a), MOVED** (section 4): the parser takes its nodes and line faces from the host, keyed by scene and kind. Observed in both profiles |
| `modules/3d/3dsubobj.c` (4), scene bounds built in `3dobjdb.c` (`initialise_scene_bounding_sub_objects`, `:1386`) | `bounding_box`, `bounding_box2` of each main and sub-object | at scene load (hierarchy construction) | **(b)**: the host's scene reply already carries the same bounds; the scene database must take its bounds from it. Part of the hierarchy service, still core-resident, visible |
| `3dobjdb.c` remainder (loader 101 hits; `add_animation_object` surface texture-animation flags; `debug_object`; custom EEO/EES import) | the loader's own fills; surface texture flags | load | The loader itself is what leaves (host-owned model loading). Texture-animation lists are rendering data: **(c)** or a host render-metadata answer, decided with the renderer boundary, not here |
| render and visibility: `3dobject.c` (19), `3dobjdrh/drw/drz.c` (64/53/12), `3dobjzdr.c` (20), `3dobjvb.c` (5), `3dobjvis.c` (33), `3dexplos.c` (32), `3dshadow.c` (51), `3dhoriz.c` (15), `3dwface1/2.c` (5/16), `3dfaces.c` (6), `3dscon.c` (9), `clouds/3dtrump.c` (5), `3dclouds.c` (3) | vertices, faces, surfaces, bounds for transform, draw, outcodes, shadows | renderer entry points (`draw_*`, `transform_*`, `get_object_3d_outcode`, instance visibility); the dedicated-server boot path calls no drawing, but that is **not proved dormant** for every campaign path | **(c)** by construction: these entries are not part of the hosted module's surface (the host owns rendering). The cutover Task must either remove them from the build or stop each entry clearly before reading an absent array; no silent empty-geometry draw. Which of them a boot actually reaches is established from evidence, by a method chosen at the cutover; none is assumed dormant |
| `hi_hud.c:328` (1) | `hud_display_model` pointer into the array | the player's Mi-24 avionics init | **(c)** (player-gunship UI, not the campaign), or a host display-model answer with the avionics boundary; the legacy pointer stays private either way |

Not in this census (separately read): `objects_3d_scene_database` readers (`popread.c` scene link objects, `damage.c` `succeeded`, `main_sc.c` menu links) and the
poly-line accessors, per the host-authority audit table; the viewpoint, device, muzzle, target-point and camera services of `3dobjdb.c`. They remain whole-D3 work and none
is assigned to another phase here.

## 4. The first production extraction: the host owns the route data (source; NOT built, NOT run)

With the original route characterisation closed (the unmoved parser, 704 Georgia and 528 Lebanon natural-boot calls with exact caller identities, four actual native runs 0,
complete repeats byte-identical), the airfield route parser no longer reads the campaign DLL's model arrays. Core keeps what is campaign: which route its own hierarchy lookup
selected for an airfield (`get_airfield_waypoint_route_object`, `popread.c`) and every route-planning statement of `routegen.c` (tree, primary and start nodes, end-slot matching,
the per-depth positions), which are textually unchanged. The host owns the model data behind it.

* **Host store** (`eech-world/src/models.rs`, the reference host; DCS would feed the same questions from its own data). The loader now keeps what it used to skip: each scene's and each
  sub-object's (kind, offset) records (`read_indices`), with `get_subobject`'s name-to-kind resolution for `.EES` scenes (`data/subobject-names.txt`, generated from the original
  header exactly as `scene-names.txt` is), and each surface's colour. `route_node (scene, kind, index)` and `route_face (scene, kind, index)` answer from them: the lookup is
  `initialise_scene_quick_sub_object_search` (the scene's own records first, then each top-level sub-object's own records before its children, depth first) at depth 0; the node is
  the stored short scaled by the object's bounds by a new C kernel entry (`model_step.c eech_model_route_node`: `max (fabs (min), fabs (max))` in float, then `short * (max / 32767)`
  with the division in FLOAT, the original's expressions, under the same entry guard that restores the caller's complete floating-point state); the line faces come surface by surface,
  each face's two entries of the face point list resolved through its own surface's run of the surface-point references, with the surface's colour and polygon bit. True absence
  (a scene never read, no record of the kind), the end of a cursor, an argument error and a refusal by name (an unbacked read) are four different answers.
* **Public operation** (`mb_hostmodel.h`): `ROUTE_NODE` and `ROUTE_FACE`, each (scene, kind, cursor) with the kind's compiled-name echo checked as the scene's is, through the same
  callback, Lua bridge and strict request gate as the triangle (a fourth provider argument, `kind`). No raw short, bound or legacy object number crosses.
* **Core** (`mb_hostmodel.c`, staged patches `ROUTE_HOST_PATCHES` H1..H5 in `build/patches.rs`): `get_airfield_waypoint_route_object` tells the campaign which entry was found for which
  (scene, kind) (H1/H2); at the 28 places that took the legacy number from a route entry (26 `raw_object_index = <entry>->index;` sites and the two inline troop-route calls) the
  parse now receives that entry's KEY (H3), evaluated once, in the same place and order; the parse's object is built from the host's ordered nodes and line faces (H4) and each node's
  position is the host's (H5). The line faces are packed into surfaces of at most 128 faces of one colour and polygon bit so that the unchanged loops walk them: a packaging of the host's
  flat answers in the same order, not a model. A route the hierarchy found and the host calls absent, a refusal, a malformed or out-of-domain reply, a kind name that disagrees, or more
  nodes or faces than the parser's own tables hold stops the engine with a named `[model-provider:route.*]` marker; nothing is invented and nothing falls back to the campaign's arrays.
* **Candidate characterisation** (feature `route-fixture` WITHOUT `route-original`): the observer of the HOST profile. It records the host's ACTUAL answers (decoded node bit patterns, the
  ordered line faces with the colour and polygon bit) and the unchanged Core's final graph (finals, links, starts, per-depth positions and offsets), under the caller identity the production
  key table gives (scene, kind). The report starts with `route-profile value=host` and has no raw stored shorts, face-local references or surface records: the campaign never holds them
  and none is invented. The original profile is built with `route-fixture` AND `route-original` (`EECH_ROUTE_PROFILE=original tools/build-windows-route-fixture.sh ...`), which keeps the
  unmoved parser. **What the Root compares across the move**: the declared unchanged cohorts, row for row and byte for byte, per scene and kind in the call order: the slots, the decoded
  node bit patterns, the ordered face point pairs with colour and polygon bit, the finals, links, starts, per-depth positions and offsets. What differs by construction: the raw shorts and
  the face-local columns, the surface columns and bounds (the host profile has none), `legacy_object` and `caller_raw` (now the key number), and the profile line. No whole-file equality is claimed.
* **Controls prepared** (the Root runs them): the host store's contract tests (`tests/model_route.rs`: kind lookup order, the colour, the ordered faces through each surface's own
  references, absence versus end versus argument error versus refusal), the driver's host-profile shapes, the bridge tests in `eech-dc`, the model-host-fixture probe now also asks one route
  node and face under each caller rounding mode (`model-host-route` line), and the existing authority modes (`model-authority.lua` forwards the new `kind` argument; the route questions
  pass through every mode unchanged). Poisoning the campaign's own geometry rows for the route objects is possible only after the loader detachment below, because the route no longer reads them
  but other readers still do.

**The next batch, the loader detachment, is NOT done**: the common `3dobjs.*` loading and the custom `.EEO` / `.EES` loading (`initialise_3d_objects`, `read_object`, `read_custom_scene` in
`3dobjdb.c`) still fill `objects_3d_data` in the campaign DLL. It needs true counts, bounds and radius for the readers that remain (scene bounds, the render and visibility group, the Mi-24
HUD, the texture-animation scan, section 3), NULL geometry pointers, and for each live reader a host query, true metadata or a named refusal BEFORE a buffer is read. The hierarchy,
viewpoint, device, muzzle and target-point services of `3dobjdb.c` stay Core and are required whole-D3 work (Task49 owns collision, not all hierarchy).

