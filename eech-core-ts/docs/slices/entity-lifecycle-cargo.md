# Slice 3: campaign entity lifecycle and CARGO foundations (issue #5)

**Status:** frozen. All conformance gates pass (see "Evidence" at the end).

Issue #5 first proposed porting `keysite.c :: update_keysite_cargo`. The first
investigation showed that it rests on machinery the port did not have: entity
creation from attribute lists, entity destruction, the entity heap, sector
membership, the CARGO entity and a 3D-object dimension query. Issue #5 was
rescoped to that foundation. `update_keysite_cargo` moves to Slice 4 and is
ported intact there. The original investigation is kept below because it
motivates the boundary.

## Investigation 1: `keysite.c :: update_keysite_cargo`

```c
void update_keysite_cargo (entity *en, float cargo_level, entity_sub_types sub_type, int cargo_size)
```

- It returns early while `get_game_status () == GAME_STATUS_INITIALISING`, and
  for a keysite that is not `alive` or not `in_use`.
- Crate positions come from
  `get_object_3d_bounding_box (OBJECT_3D_SINGLE_CRATE)`, a model dimension.
- It walks `raw->cargo_root`. Surplus crates are destroyed with
  `destroy_client_server_entity_family`. Missing crates are created with:

  ```c
  create_client_server_entity (ENTITY_TYPE_CARGO, ENTITY_INDEX_DONT_CARE,
      ENTITY_ATTR_PARENT (LIST_TYPE_CARGO, en),
      ENTITY_ATTR_INT_VALUE (INT_TYPE_SIDE, ...),
      ENTITY_ATTR_INT_VALUE (INT_TYPE_ENTITY_SUB_TYPE, sub_type),
      ENTITY_ATTR_VEC3D (VEC3D_TYPE_POSITION, x, y, z),
      ENTITY_ATTR_END);
  ```
- A separate `cargo_level <= KEYSITE_SUPPLY_REQUEST_THRESHOLD` arm notifies the
  force when the keysite type consumes that supply.
- The callers are `ks_creat.c` and `ks_updt.c`. `ks_updt.c :: update_server`
  also does task assignment, repair and drift, so it is out of scope.

**Conclusion.** Porting any fragment of this function would mean inventing the
creation, destruction and sector machinery underneath it, or faking it. The
proposed boundary was invalidated. Slice 3 ports that machinery through the
original cargo lifecycle instead: create a cargo entity through the real
construction path, establish its relationships and sector membership, destroy
it through the real family-destruction path, and verify the resulting graph.

## Investigation 2: the creation and destruction machinery

### Creation (`en_creat.c`, `en_attrs.c`, `en_heap.c`)

- `create_client_server_entity (type, index, ...)` and
  `create_local_entity (type, index, ...)`:
  1. `ASSERT ((type > ENTITY_TYPE_UNKNOWN) && (type < NUM_ENTITY_TYPES))`.
  2. Call `update_create_entity_statistics`.
  3. Turn the variadic arguments into a `char *` attribute buffer. On TX, or
     when `force_local_entity_create_stack_attributes` is set, the buffer is the
     stack itself (`pargs_buffer = (char *) pargs`). On RX, it is the first
     variadic argument: `en_comms.c` unpacks a received message into
     `attribute_buffer` with `unpack_entity_attributes`.
  4. Dispatch through `fn_create_client_server_entity[type][comms_model]` or
     `fn_create_local_entity[type]`. The default `default_create_entity` is
     `debug_fatal`.
  5. A NULL result is `debug_fatal ("... Limit of %d reached")` on the server,
     and always for `create_local_entity`.
- `set_local_entity_attributes (en, pargs)` walks the buffer with `get_list_item`
  (`modules/misc/listitem.h`). The attribute kinds are:
  - `END`
  - `ATTITUDE_ANGLES`: 3 doubles
  - `CHAR_VALUE`: type, int
  - `CHILD_PRED`: type, entity pointer
  - `FLOAT_VALUE`: type, double
  - `INT_VALUE`: type, int
  - `PARENT`: type, entity pointer
  - `PTR_VALUE`: type, pointer
  - `STRING`: type, pointer
  - `VEC3D`: type, 3 doubles

  Floats reach it promoted to double, and each attribute calls the entity
  type's *raw* setter (`set_local_entity_raw_int_value`, ...) or
  `set_local_entity_parent` / `child_pred`. An unknown attribute is
  `debug_fatal`.
- **`ENTITY_ATTR_PARENT` only stores the link's parent pointer.** The entity's
  `create_local` inserts it into the list afterwards.
- `get_free_entity (index)`: the heap is a doubly linked free list and used list
  threaded through `entity.succ` / `pred`.
  - With `ENTITY_INDEX_DONT_CARE` it takes the head of the free list. An
    exhausted heap is logged and yields NULL.
  - Otherwise it takes that exact index, and an index already in use is
    `debug_fatal`.
  - The entity is pushed at the head of the used list.
- `set_free_entity` clears `type` and `data` and pushes the entity at the head
  of the free list, so the most recently freed index is reused first.
- `initialise_entity_heap (n)` threads `entities[0..n-1]` in order.
- The downwash local-only heap (`command_line_downwash`,
  `ENTITY_INDEX_CREATE_LOCAL`) is a separate mode. It is off by default and not
  ported.
- `validate_local_create_entity_index` / `validate_remote_create_entity_index`
  are debug-build `ASSERT`s (`en_valid.c`). A server local create requires
  `ENTITY_INDEX_DONT_CARE`, and a server remote create requires a real index.

### CARGO (`entity/mobile/cargo`, `entity/mobile/mb_*.c`)

- `struct CARGO` is `mobile mob`, `task_dependent_root`, `cargo_link`,
  `movement_dependent_link` and `terrain_info`. `struct MOBILE` carries:
  - `sub_type`, `position`, `attitude`, `motion_vector`, `velocity`;
  - the `special_effect` and `target` roots;
  - the `padlock`, `sector`, `target` and `update` links;
  - the bit-fields `alive : 1` and `side : 2`.

  **Stores into the bit-fields truncate.**
- `cg_creat.c :: create_local`:
  1. Validate the index.
  2. `get_free_entity`, then `malloc`, then `memset`.
  3. Set defaults: `sub_type = ENTITY_SUB_TYPE_UNINITIALISED (-1)`, position
     `MID_MAP_X/Y/Z`, the identity attitude, `alive = TRUE`,
     `side = ENTITY_SIDE_UNINITIALISED (3)`.
  4. `set_local_entity_attributes`.
  5. If `cargo_link.parent` is set, insert into `LIST_TYPE_CARGO` at the head.
  6. Always insert into `LIST_TYPE_SECTOR` of
     `get_local_sector_entity (&raw->mob.position)`. The update-list insertion
     is commented out in the original.
- `create_server` runs `create_local`, then `create_remote`. `create_remote`
  calls `transmit_entity_comms_message (ENTITY_COMMS_CREATE, NULL, type, index, pargs)`
  with the created entity's index.
- `cg_dstry.c`:
  - `destroy_server_family` calls `destroy_client_server_sound_effects`, then
    `destroy_client_server_entity`.
  - `destroy_server` transmits `ENTITY_COMMS_DESTROY` first, then
    `destroy_local_entity`.
  - `destroy_local` does the following, then `free_mem` and `set_free_entity`:
    - unlinks the `TASK_DEPENDENT`, `SPECIAL_EFFECT` and `TARGET` children;
    - deletes the cargo from `CARGO`, `MOVEMENT_DEPENDENT`, `PADLOCK`, `SECTOR`
      and `UPDATE`.
- Value overloads:
  - The mobile ones (`mb_int.c`, `mb_vec3d.c`, `mb_list.c`) provide `ALIVE`,
    `SIDE`, `ENTITY_SUB_TYPE`, `IDENTIFY_MOBILE`, `POSITION` and the mobile
    lists.
  - `cg_int.c` adds `IDENTIFY_CARGO` and `TASK_TARGET_TYPE`.
  - `cg_list.c` adds the cargo lists.
  - Other `IDENTIFY_*` values keep the `en_int.c` default of 0.
- **Messages:** `cg_msgs.c :: overload_cargo_message_responses` calls
  `overload_aircraft_message_responses (ENTITY_TYPE_CARGO)`. A cargo's
  `LINK_PARENT` / `UNLINK_PARENT` responses are therefore the **aircraft**
  ones (`ac_msgs.c`). They only act for `LIST_TYPE_TARGET`,
  `LIST_TYPE_GUNSHIP_TARGET` and `LIST_TYPE_UPDATE`, and return TRUE otherwise.
- **Keysite messages:** `ks_msgs.c`'s `LINK_CHILD` / `UNLINK_CHILD` responses
  only log, under `DEBUG_MODULE`. They return TRUE. `ks_list.c` owns
  `LIST_TYPE_CARGO_ROOT`.

### Sectors (`entity/special/sector`, `en_world.c`)

- `set_entity_world_map_size (x_sectors, z_sectors, side_length)` sets the map:
  - `side_length` must be a power of 2 (`ASSERT`);
  - `MAX_MAP_X = (float) (x_sectors * side_length) - 1.0`, and likewise Z;
  - `MIN_MAP_Y = -8000`, `MAX_MAP_Y = 65535`;
  - `MID_MAP_* = min + (max - min) * 0.5`, in float.

  The campaign script parser calls it with the terrain's dimensions.
- `create_local_sector_entities` allocates `entity_sector_map` and creates one
  local SECTOR entity per cell, z-major, with
  `create_local_entity (ENTITY_TYPE_SECTOR, ENTITY_INDEX_DONT_CARE, INT X_SECTOR, INT Z_SECTOR, END)`.
  It runs under SERVER/TX with stack attributes
  (`en_creat.c :: create_local_only_entities`). `struct SECTOR` stores
  `x_sector` and `z_sector` in 8-bit fields.
- `get_local_sector_entity (pos)`:
  - `debug_fatal` unless `point_inside_map_area (pos)`, i.e.
    `MIN <= x <= MAX` and `MIN <= z <= MAX`;
  - `get_x_sector`, which is `convert_float_to_int (x)` then
    `/= SECTOR_SIDE_LENGTH` (integer division);
  - `entity_sector_map[x + z * NUM_MAP_X_SECTORS]`, with `ASSERT (en)`.
- `sc_msgs.c :: response_to_link_child`, for `LIST_TYPE_SECTOR`:
  - Checks `INT_TYPE_IDENTIFY_FIXED`: a fixed entity updates
    `tallest_structure_height` from `get_object_3d_bounding_box`.
  - Calls `add_mobile_values_to_sector`: on the server, for a live **vehicle**,
    this updates the imap defence levels.
  - Aircraft and vehicles also get fog of war and `FORCE_ENTERED_SECTOR`
    notifications.

  A cargo is none of these, so for cargo the response only reads values.
  `response_to_unlink_child` calls `remove_mobile_values_from_sector`, which
  mirrors `add_mobile_values_to_sector`.

## Investigation 3: compiling the keysite, force, cargo, mobile, sector and entity-system files

The following original translation units compile **unchanged** against the
generated `project.h`, once whole original headers and a few verbatim fragments
are added:
- `ks_int.c`, `ks_float.c`, `ks_vec3d.c`, `ks_list.c`, `ks_dbase.c`, `fc_int.c`, `fc_list.c`
- `cg_creat.c`, `cg_dstry.c`, `cg_list.c`, `cg_int.c`
- `mb_int.c`, `mb_vec3d.c`, `mb_list.c`
- `sc_seccreat.c`, `sc_msgs.c`, `sc_list.c`, `sc_int.c`
- `en_heap.c`, `en_attrs.c`, `en_creat.c`, `en_dstry.c`

The added headers are `keysite.h`, `force.h`, `en_world.h`, `en_attrs.h`,
`mobile.h` (with `cargo.h`), `sector.h`, `3d/3dmodels.h` and `3d/textanim.h`.
The added fragments are:
- the `double_vec3d`, `object_3d_bounds` and `terrain_3d_point_data` structs;
- the enums `GUNSHIP_TYPES`, `GAME_STATUS_TYPES`, `SESSION_LIST_TYPES` and
  `DEBUG_COLOURS`;
- `modules/misc/listitem.h`;
- typedef-only forward declarations of the engine structs that the headers only
  use through pointers.

The link surface beyond Slice 2 falls into four groups:
- **Environment:** memory (`malloc_heap_mem`, `malloc_fast_mem`, `free_mem`),
  `convert_float_to_int`, `get_identity_matrix3x3`, debug output, and the
  transport (`transmit_entity_comms_message`), which gains CREATE and DESTROY.
- **Extracted verbatim:**
  - `unlink_local_entity_children`;
  - `destroy_client_server_sound_effects`;
  - `get_local_sector_entity`, `add_mobile_values_to_sector` and
    `remove_mobile_values_from_sector`;
  - `set_entity_world_map_size`;
  - the create-index assertions;
  - the statistics counters;
  - the aircraft link/unlink parent responses and the keysite link/unlink child
    responses.
- **Fail-loud stubs, never reached by this slice:**
  - `pack_*` / `unpack_*` (save games and network messages);
  - the pylon, bridge and camera creation and destruction;
  - `get_object_3d_bounding_box` (Slice 4);
  - fog of war, imap and game session state.
- **Hand-written rows that the original files now replace:** the harness's own
  entity storage, and the keysite and force shims.

### Finding: the original creation path requires the 32-bit x86 calling convention

`pargs_buffer = (char *) pargs` is only meaningful where `va_list` is a pointer
into the argument stack. EECH was built for 32-bit x86 (Watcom has its own
`#ifdef` for the same line). On x86-64 System V, `va_list` is an array of a
register-save descriptor, and the conversion yields garbage.

Every creation that runs on the server's normal TX data flow depends on this,
including `update_keysite_cargo` and `create_local_sector_entities`. The C
reference harness is therefore now built for 32-bit x86 (`-m32`), with SSE
float arithmetic so that `FLT_EVAL_METHOD` stays 0 (commit `1f59409`). The
Slice 1 and 2 fixtures re-record byte-identically, and the signal-safety test's
negative controls still fail it. CI installs `gcc-multilib`.

The port has no varargs. Both calling conventions (stack attributes on TX, a
received buffer on RX) deliver the same ordered attribute list, and the port
takes that list as an array of tagged attributes.

### Finding: EECH runs the FPU in round-toward-zero mode

`modules/system/startup.c` calls `set_fpu_rounding_mode_zero ()` at start-up,
with the comment "The graphics / 3d / 2d systems need the maths fpu to round to
zero". `init.c` and `terrdata.c` repeat the call.
`convert_float_to_int` is x87 `fistp`, so under that mode it truncates. The
sector lookup therefore truncates positions, which is what the port and the
harness do.

**Open, cross-cutting:** the x87 control word also governs the rounding of all
x87 float arithmetic in the original executable. Slices 1 and 2 model IEEE
single precision with round-to-nearest, and so does the harness (SSE, default
rounding). No slice has yet been checked against round-toward-zero arithmetic.
This is recorded for a decision and is not changed silently in this slice.
Slice 3's own arithmetic is limited to:
- the map extents (exact for power-of-two side lengths and realistic sector
  counts);
- the midpoint (`* 0.5`, exact);
- the double-to-float narrowing of attribute positions. The corpus uses
  float-representable positions, for which the rounding mode cannot matter.

### Finding: `OBJECT_3D_SINGLE_CRATE` is in the repository

An earlier note in issue #5 said this index came from a generated database
outside the repository. That is wrong. `modules/3d/3dmodels.h` defines it
through the `OBJECT_3D_INDEX (SINGLE_CRATE)` macro, and `3dmodels.c` holds its
name. What is **not** in the source is the model's geometry. The bounding box is
loaded at run time into the 3D object database (`3dobjdb.h`,
`objects_3d_data[...].bounding_box`), and `get_object_3d_bounding_box` returns
it.

A second campaign consumer exists: `sc_msgs.c :: response_to_link_child`
reads the bounding box of every fixed entity that enters a sector. This supports
a generic object-dimensions port keyed by `object_3d_index_numbers`, not a
crate-specific one. It lands in Slice 4 with `update_keysite_cargo`.

### Other original-code observations (recorded, not changed)

- `cg_dstry.c :: kill_local` deletes the cargo from `LIST_TYPE_TARGET` where its
  comment says "sector_link". Kill is outside this slice.
- `create_local_sector_entities` allocates `sizeof (entity) * N` bytes for an
  array of `entity *`, which over-allocates harmlessly.
- `overload_aircraft_message_responses (type)` always writes the HELICOPTER
  `ARTICULATE_UNDERCARRIAGE` row, whatever `type` is.

## Selected boundary

**Ported to TypeScript:**
- **Entity heap** (`en_heap.c`): `initialise_entity_heap`, `get_free_entity`
  (both arms: the free-list head for `ENTITY_INDEX_DONT_CARE`, and a specific
  index, unlinked from wherever it sits in the free list), `set_free_entity`.
  The heap size becomes a campaign option. Client creation, which is what
  passes a specific index at run time, stays unported one level up, at the
  create tables.
- **Creation** (`en_creat.c`):
  - `create_local_entity` and `create_client_server_entity`, with the type
    `ASSERT` and fatal NULL results;
  - the `fn_create_*` tables, with an unported default;
  - the create-index assertions (`en_valid.c`).
- **Attributes** (`en_attrs.c`): `set_local_entity_attributes` over a typed
  attribute list. Every attribute kind dispatches to the entity type's raw
  setter, and setters that are not ported fail as unported.
- **Destruction** (`en_dstry.c`, `en_list.c`):
  - `destroy_local_entity`, `destroy_client_server_entity` and
    `destroy_client_server_entity_family`, with the C default no-op for types
    whose tables keep it;
  - `unlink_local_entity_children`.
- **World map** (`en_world.c`): `set_entity_world_map_size`,
  `point_inside_map_area`, `get_x_sector` / `get_z_sector` (truncation).
- **Sector:**
  - `create_local_sector_entities` and the sector's `create_local`;
  - `get_local_sector_entity`;
  - `add_mobile_values_to_sector` / `remove_mobile_values_from_sector`: the
    vehicle arm stays unported;
  - the `LIST_TYPE_SECTOR` link and unlink child responses: the fixed-entity,
    aircraft and vehicle arms stay unported;
  - the sector lists and the `X_SECTOR` / `Z_SECTOR` values (8-bit fields).
- **Mobile, for CARGO:**
  - the `ALIVE`, `SIDE`, `ENTITY_SUB_TYPE` and `IDENTIFY_MOBILE` values, with
    bit-field truncation;
  - `POSITION`;
  - the mobile lists;
  - the aircraft link/unlink parent responses: the target, gunship-target and
    update arms stay unported.
- **CARGO:** `cg_creat.c` (local and server create), `cg_dstry.c` (local,
  server, server family), `cg_list.c`. `cg_int.c` has no reader in this slice
  and stays unported.
  `destroy_client_server_sound_effects` walks the special-effect children;
  destroying a sound effect stays unported.
- **Keysite:** `LIST_TYPE_CARGO_ROOT` and the link/unlink child responses.
- **Port `EntityReplication`** gains `transmitEntityCreate (type, index, attributes)`
  and `transmitEntityDestroy (index)`: EECH's `ENTITY_COMMS_CREATE` and
  `ENTITY_COMMS_DESTROY`. The attributes are what `pack_entity_attributes`
  carries: floats narrowed, entities by index.
- **List setters:** the `ASSERT (en != ...)` of `en_list/set_*.h`, which the
  Slice 2 setters lacked (found by the random differential, see "Evidence").
- **Options:** `numberOfEntities`, the heap size (`init.c`: 125000). Heap
  records are materialised on first use, so the default costs nothing.
- **World map data** is campaign data from the campaign script parser, not
  measured environment, so the host sets it through the ported
  `setEntityWorldMapSize`; there is no port.

**Not in this slice:**
- `update_keysite_cargo` and the object-dimensions port (Slice 4);
- `ks_updt.c`;
- kill;
- cargo movement, update, draw and pack;
- entity statistics (debug display counters, with no campaign effect);
- the downwash local heap;
- client-side creation and destruction (the port is the server);
- any other entity type's create or destroy function (reaching one fails as
  unported).

**Shim reduction:** `ks_int.c`, `ks_float.c`, `ks_vec3d.c`, `ks_list.c`,
`ks_dbase.c`, `fc_int.c` and `fc_list.c` replace the hand-written keysite and
force rows. `en_heap.c` replaces the harness's entity storage. The cargo,
mobile, sector, attribute, creation and destruction files above are compiled
whole. `docs/architecture.md`, "Shrinking the C reference shim", lists what
remains.

## Behaviour matrix

Every row is a case in `test/scenarios/entity-lifecycle.cases.ts`. Each case
runs three ways:
- under JavaScript (`test/unit/entity-lifecycle.test.ts`);
- under Lua 5.1 (`test/lua/conformance.ts`);
- through the executed original C (`test/c-reference/entity-lifecycle.cref.test.ts`).

A case's hand-derived lines must appear in the output. The whole TypeScript
output must also equal the C's, line for line. The output covers the
transmitted create and destroy messages, the created indices, the result, and
the entity graph: the heap's free and used order, the cargo values, each
keysite's cargo list and each sector's list.

| Area | EECH behaviour | Cases |
|---|---|---|
| Creation | the real construction path: heap entry, working defaults, attributes, `LIST_TYPE_CARGO` insert when a parent is given, sector insert always, then `ENTITY_COMMS_CREATE` with the created index | `crate-created-through-the-real-construction-path`, `no-parent-attribute-means-no-cargo-list`, `a-null-parent-attribute-means-no-cargo-list` |
| List order | inserts at the head of the keysite and sector lists | `crates-are-inserted-at-the-head-of-the-keysite-and-sector-lists` |
| Defaults and attributes | `ENTITY_SUB_TYPE_UNINITIALISED`, `MID_MAP_*`, alive, `ENTITY_SIDE_UNINITIALISED`; later attributes win; a parent attribute only sets the link; a child-pred attribute is overwritten by the head insertion | `absent-attributes-keep-the-working-defaults`, `later-attributes-overwrite-earlier-ones`, `a-parent-attribute-for-the-sector-list-is-overwritten`, `a-child-pred-attribute-is-overwritten-by-the-head-insertion` |
| Bit-fields | `side : 2`, `alive : 1`, full-int `sub_type`, sector `x_sector : 8` | `side-is-a-two-bit-field`, `alive-is-a-one-bit-field`, `sub-type-is-a-full-int`, `sector-coordinates-are-eight-bit-fields` |
| Sector lookup | truncation to cells; the map edges are inside; past the edge or negative is `debug_fatal`, after the cargo list insert | `sector-cells-truncate-positions`, `the-map-edges-are-inside`, `a-position-past-the-map-edge-is-fatal`, `a-negative-position-is-off-the-map` |
| Map setup | z-major sector creation; float extents; power-of-two and positive-size `ASSERT`s; recreating over live sectors is fatal; heap exhaustion during setup is fatal | `sector-entities-are-created-z-major`, `the-map-extents-are-floats`, `a-side-length-that-is-not-a-power-of-two-asserts`, `a-map-without-sectors-asserts`, `creating-the-map-twice-is-fatal`, `running-out-of-heap-while-creating-sectors-is-fatal` |
| Validation | server creates need `ENTITY_INDEX_DONT_CARE`; the entity type range `ASSERT` | `a-server-create-with-an-index-asserts`, `an-entity-type-out-of-range-asserts`, `num-entity-types-is-out-of-range` |
| Heap | exhaustion is fatal on the server; the most recently freed index is reused first | `running-out-of-heap-while-creating-cargo-is-fatal`, `the-most-recently-freed-index-is-reused-first` |
| Allocation at an index | `get_free_entity (index)` unlinks the entry from the head, middle or tail of the free list (also after reuse reordered it) and pushes it on the used list; an entry in use is `debug_fatal` before any list changes; outside the heap is the `ASSERT`; `ENTITY_INDEX_DONT_CARE` takes the head | `allocating-the-free-list-head-by-index`, `allocating-a-middle-free-entry-by-index`, `allocating-the-free-list-tail-by-index`, `allocating-by-index-after-reuse-reordered-the-free-list`, `allocating-the-freed-head-by-index`, `allocating-the-only-free-entry-by-index-empties-the-heap`, `allocating-an-entity-in-use-is-fatal`, `allocating-a-sector-index-is-fatal`, `allocating-past-the-heap-asserts`, `allocating-a-negative-index-asserts`, `allocating-with-entity-index-dont-care-takes-the-head` |
| Destruction | `ENTITY_COMMS_DESTROY` first; unlinked from the keysite and sector lists (head, middle, tail); freed; destroying a freed entity does nothing | `destroying-the-only-crate-restores-the-graph`, `destroying-the-head-crate`, `destroying-a-middle-crate`, `destroying-the-tail-crate`, `destroying-a-freed-entity-does-nothing`, `crates-of-two-keysites-in-one-sector` |

Some primitives cannot be reached by the cargo corpus by itself. They are
covered by isolated unit tests with C-derived expectations
(`test/unit/entity-lifecycle-runtime.test.ts`):
- the fail-loud arms of the sector, aircraft and sound-effect code;
- integer division by zero (a cargo created before any world map);
- the unchecked read of a missing sector map;
- the raw float setter reached through an attribute;
- allocating by index into an empty used list;
- freeing into an empty free list, and freeing the used list's tail;
- recreating the map after its sectors were freed.

## Evidence

| Check | Result |
|---|---|
| 44 lifecycle cases, JavaScript (`npm test`) | pass |
| the same 44 cases against the executed original C, plus TS output == C output line for line | pass: expectations were derived by hand and all matched on the first run (the 11 allocation cases were added after review) |
| 1,000 fresh random lifecycles, TS == C | pass. The generator must reach ok, destroy, two crates in one keysite, allocation at an index, every fatal (off map, heap exhausted on create and on map, map recreated, entity in use) and every assert (create index, type, side length, sector count, heap index) |
| 150 random lifecycles recorded from the C, replayed in JavaScript and Lua 5.1 | pass |
| Slices 1 and 2: all cases and recorded fixtures, against the 32-bit, TX, less shimmed oracle | pass **unchanged**; re-recording their fixtures was byte-identical |
| Lua 5.1 conformance (`npm run test:lua`) | 679 / 679 |
| coverage (statements / branches / functions / lines) | 100 / 100 / 100 / 100, no exclusions |
| mutation controls (`npm run mutation`) | 40 / 40 killed, including 17 new slice 3 mutants |

Review of PR #6 found that this slice first left the specific-index arm of
`get_free_entity` unported while claiming the heap primitive as ported. The arm
is now ported and verified against the executed C. The lifecycle scenarios
have an `allocate` operation that calls the original `get_free_entity (index)`,
as restoring a saved group does. The random generator uses it too.

The random differential found one gap in the port, not in this slice's new
code. Every original list setter (`en_list/set_frst.h`, `set_prnt.h`,
`set_succ.h`, `set_pred.h`) starts with `ASSERT (en != ...)`. The Slice 2 port
of the setters had no such check. A random lifecycle reached it by naming a
destroyed crate as the child predecessor of a new crate that reused its index.
The setters now keep the assertion.

Two harness defects were found and fixed on the way:
- `-w` in the original-code flags had silently disabled the
  `-Werror=implicit-function-declaration`, pointer and int-conversion checks.
  The Slice 1 and 2 build turned out clean without it.
- A lifecycle dump after a failed second `map` operation read the new map size
  over the old sector map.
