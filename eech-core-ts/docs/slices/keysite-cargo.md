# Slice 4: `update_keysite_cargo` (issue #10)

This slice ports `keysite.c :: update_keysite_cargo` intact onto two frozen foundations:
- Slice 3's cargo lifecycle and sector membership (PR #6);
- the round-toward-zero numerical contract from #7 (PR #8, `796d5a7`).

The branch starts from `796d5a7`. The investigations below fixed the boundary before any TypeScript was written. Line numbers refer to `796d5a7`.

## Investigation 1: the function, traced onto the existing infrastructure

`aphavoc/source/entity/special/keysite/keysite.c:337`:

```c
void update_keysite_cargo (entity *en, float cargo_level, entity_sub_types sub_type, float cargo_size)
```

### Callers (outside this slice)

- `ks_creat.c:199/201`, while a keysite is being created ("RESOLVE DEFAULT VALUES"): once with `raw->supplies.ammo_supply_level, ENTITY_SUB_TYPE_CARGO_AMMO, CARGO_AMMO_SIZE`, then with fuel.
- `ks_updt.c:140/142`, in the keysite's server update whenever its sleep timer expires. The call comes after the supply drift and before the levels are bounded.

`CARGO_AMMO_SIZE` and `CARGO_FUEL_SIZE` are both `10` (`cargo.h:89/91`), passed as `float`. The callers themselves (keysite creation and the keysite update with its task assignment, repair and drift) are not in this slice.

### Step by step

| # | C (`keysite.c`) | What it depends on | Existing infrastructure |
|---|---|---|---|
| 1 | `if (get_game_status () == GAME_STATUS_INITIALISING) return;` (:361) | the game status, see Investigation 3 | new core state |
| 2 | `raw = get_local_entity_data (en); if ((!raw->alive) \|\| (!raw->in_use)) return;` (:368) | the keysite raw bit-fields `alive:1` and `in_use:1` | `KeysiteRaw` has `in_use`; **`alive` is new** |
| 3 | `memcpy (&position, get_keysite_supply_position (en), sizeof (vec3d));` (:375) | `get_keysite_supply_position` (:186) is `get_local_entity_vec3d_ptr (keysite, VEC3D_TYPE_POSITION)` | ported (`ks_vec3d.c`) |
| 4 | `bounding_box = get_object_3d_bounding_box (OBJECT_3D_SINGLE_CRATE);` (:377) | 3D object database, see Investigation 2 | **new port** |
| 5 | `position.y -= bounding_box->ymin;` | float − float | RTZ `f32Sub` |
| 6 | `position.z += sub_type * ((bounding_box->zmax - bounding_box->zmin) + 1);` | float − float, then + int `1` (converted to float), then int `sub_type` × float, then float + float. Ammo is sub-type 0 and fuel 1, so ammo crates stand in the row at `z` and fuel crates one crate depth + 1 further on. | RTZ `f32Sub` / `f32Add` / `f32Mul` |
| 7 | counting: `temp_cargo_level = cargo_level;` then walk `raw->cargo_root.first_child` along `LIST_TYPE_CARGO` | the cargo list, rooted at the keysite | ported (`ks_list.c`, `cg_list.c`) |
| 7a | only crates whose `INT_TYPE_ENTITY_SUB_TYPE == sub_type` take part: `temp_cargo_level -= cargo_size;` | float − float | RTZ `f32Sub` |
| 7b | if `temp_cargo_level < 0.0`: take the successor first, then `destroy_client_server_entity_family (destroy_cargo)`, and `continue` **without** advancing `position.x` | destruction and its replication | ported (Slice 3: `en_dstry.c`, `cg_dstry.c`, transmit-destroy) |
| 7c | otherwise `position.x += (bounding_box->xmax - bounding_box->xmin) + 1.0;` | float − float, then a double sum `+ 1.0`, then float + double stored as float | RTZ, see Numerics |
| 8 | creation: `if (temp_cargo_level > cargo_size) while (temp_cargo_level > cargo_size) { create_client_server_entity (ENTITY_TYPE_CARGO, ENTITY_INDEX_DONT_CARE, PARENT (LIST_TYPE_CARGO, en), INT (SIDE, keysite side), INT (ENTITY_SUB_TYPE, sub_type), VEC3D (POSITION, position), END); position.x += ...; temp_cargo_level -= cargo_size; }` | creation, its replication and sector membership | ported (Slice 3: `en_creat.c`, `cg_creat.c`, transmit-create); the keysite `INT_TYPE_SIDE` getter is **new** (`ks_int.c`) |
| 9 | `else if (cargo_level <= KEYSITE_SUPPLY_REQUEST_THRESHOLD)` (75.0, `en_suply.h:87`): for ammo or fuel, when `keysite_database [raw->sub_type].default_supply_usage.<ammo\|fuel>_supply_level < 0.0`, `notify_local_entity (ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES, get_local_force_entity (raw->side), en, sub_type)` | the keysite database (compiled in `ks_dbase.c`); the force lookup; the message system | the force lookup and messages are ported (Slice 1). **The supply-usage columns are new** (generated). The force response stays unported and is recorded at the boundary. |

`DEBUG_MODULE` and `DEBUG_SUPPLY` are both 0 (`keysite.c:78`, `en_suply.h:93`), so the `debug_log` calls, and the `get_local_entity_string` they would make, are compiled out.

### Behaviour the table implies (all checked against the C in the matrix)

- **Crate counting.**
  - A crate of the requested sub-type is kept while the running remainder stays `>= 0`. A crate that takes it below 0 is destroyed, and so is every later one. Existing crates therefore keep ⌊level / size⌋ of them.
  - Creation needs the remainder to be **strictly** above `cargo_size`. From no crates, a level of exactly `n · size` creates `n − 1` crates, while `n` existing crates at the same level are all kept. Creation and retention are asymmetric.
  - Crates of the other sub-type are skipped entirely: they are not counted and do not move `position.x`.
- **Destroy and create ordering.**
  - `cg_creat.c:184` inserts each new crate at the **head** of the keysite's cargo list, so the walk meets the newest crate first.
  - Surplus destruction therefore removes the **oldest** crates, which are at the tail.
  - Each destruction takes the successor before destroying the family, so the walk survives removal.
  - Within one call, every destruction happens before any creation.
- **Positions.**
  - New crates are placed by count: `x0 + kept · (width + 1)`, where `width = xmax − xmin`. This does not depend on where the surviving crates actually are.
  - After oldest-first destruction, the survivors are the newer crates, further along x. A later creation can therefore place a crate on the position of a surviving one.
  - This is original behaviour, reproduced and tested, not corrected.
- **Ammo against fuel.** They share one algorithm. They differ in the z row (`sub_type` × (depth + 1)), in the database column that gates the notification, and in the message argument (`sub_type`).
- **Threshold notification.**
  - It is the `else` of the creation test, so a call that creates even one crate never notifies, however low the level. This is the "missing crate materialised first" case.
  - A call that only destroys crates can notify.
  - The comparison is `cargo_level <= 75.0`, on the level passed in, not on the remainder.
  - Keysite types whose usage for that supply is `>= 0` never notify.
  - There is no latch: every qualifying call notifies again.
- **Force lookup.** `get_local_force_entity (raw->side)` can return NULL (no force for the side, or no session). `notify_local_entity` then fails its `ASSERT (receiver)` (`en_msgs.c:197`), exactly as `assess_group_supplies` does in Slice 1.

### Numerics (the RTZ contract, `src/core/float32.ts`)

| Expression | C evaluation (declared type) | Port |
|---|---|---|
| `position.y -= ymin` | float − float | `f32Sub` |
| `(zmax - zmin) + 1` | float − float; `+ 1` is float + (float) 1 | `f32Add (f32Sub (zmax, zmin), 1)` |
| `sub_type * (...)` | (float) sub_type × float | `f32Mul` |
| `position.z += ...` | float + float | `f32Add` |
| `(xmax - xmin) + 1.0` | float − float, then a **double** sum | the double sum is exact unless the width is below 2^-29 or at least 2^53, and those need round toward zero at 53 bits: `f64AddRTZ` |
| `position.x += <double>` | double sum, stored as float | `f32Add (x, <double>)`: a double rounded toward zero and then narrowed toward zero equals one truncation |

The whole crate-row statement is one production function, `advanceCrateRow (x, xmin, xmax)`, used at both C sites (`keysite.c:428` and `:464`). Its comment records that the transcription follows the frozen #7 declared-type contract pending #9.

**Review (#11): the statement checked as a whole.**
- The harness `f32 crate-row` command executes the original statement verbatim (`struct OBJECT_3D_BOUNDS *`, `vec3d`, float x, xmin and xmax, through the stored `position.x`).
- `test/c-reference/keysite-crate-row.cref.test.ts` compares it with `advanceCrateRow`, bit for bit, on:
  - 40,000 fresh operands aimed at each rounding point: realistic rows, widths below 2^-29 or at least 2^53, cancellation, zero and negative widths, float overflow, large x, subnormals;
  - targeted cases;
  - 1,000 recorded operands, replayed in JS and Lua 5.1.
- The corpus is sensitive to the decomposition. Against the recorded C results:
  - a double `+ 1.0` rounded to nearest disagrees in 10 cases;
  - a float `+ 1.0` in 71;
  - an unrounded double width in 117.
  Mutation controls cover all three.
- **#9 canary.** `c-reference/fpu-probes` runs the same statement under the x87 variants (`crate-row`). For `x 0.1, xmin -0.025, xmax 0.025`, 24-bit precision control (x87 RTZ PC24) stores `3f933332` where declared type and x87 RTZ PC53 store `3f933333`. The C evaluates `+ 1.0` in double, so this statement separates PC24 from PC53, which the Slice 1–3 corpora could not.
| `temp_cargo_level -= cargo_size` | float − float | `f32Sub` |
| `< 0.0`, `> cargo_size`, `<= 75.0` | exact comparisons | direct |

Rounding toward zero is observable here in two ways:
- **Positions:** fractional bounding boxes and keysite coordinates change position bits.
- **Crate counting:** a level that is not a multiple of the size accumulates `temp −= size`, which truncates. Near a multiple, that can decide whether one more crate is kept or created.

x87 intermediate precision is not modelled (#9). No expression in this function combines several float operations without a store, except `(xmax − xmin) + 1.0`, which C evaluates in double anyway.

## Investigation 2: object metadata, defined from the call site

**The C interface.** EECH asks `get_object_3d_bounding_box (object_3d_index_numbers object)` (`modules/3d/3dobjvis.h:100`) and gets a `struct OBJECT_3D_BOUNDS *`: `float xmin, xmax, ymin, ymax, zmin, zmax` (`modules/3d/objects.h:432`). The value comes from the 3D object database loaded at run time (`objects_3d_data [...]`).

**Where the geometry lives.**
- `OBJECT_3D_SINGLE_CRATE` is index 2698 (`0x0A8A`, `modules/3d/3dmodels.h:2779`, name in `3dmodels.c:2775`).
- The repository ships no geometry for it. `setup/cohokum/3ddata/objects` holds 83 mod replacement objects, and none has index `0A8A`. The original crate's dimensions are only available from a game installation's object database.
- **Open item:** they are not needed for fidelity of the port, which is parameterised by them, but a deployment must supply them.

**A second consumer.** `sc_msgs.c :: response_to_link_child` reads the bounding box of every fixed entity that enters a sector (Slice 3 finding). So the interface is a generic, index-keyed query and nothing cargo-specific.

**Port (`src/ports/object-3d-metadata.ts`):**

```ts
interface Object3DMetadata {
	// C: get_object_3d_bounding_box (object) -> struct OBJECT_3D_BOUNDS *
	getBoundingBox(objectIndex: number): Object3DBounds; // { xmin, xmax, ymin, ymax, zmin, zmax }
}
```

- The core reads the six floats and narrows them as float storage.
- The core exports the index constant it asks for, `OBJECT_3D_SINGLE_CRATE = 2698`, generated from `3dmodels.h`, so an adapter can map it.
- The deterministic test adapter (`test/adapters/in-memory-object-3d-metadata.ts`) returns the bounds a scenario declares per index, and throws for an undeclared index, just as the C harness's `get_object_3d_bounding_box` returns the scenario's bounds and fails loudly otherwise.
- No DCS shape and no cargo shape: a DCS adapter maps an EECH object index to whatever model it uses.

## Investigation 3: why `update_keysite_cargo` reads the game status

**The global.** `get_game_status ()` is `global.h:91`, `#define get_game_status() (game_status)`. `game_status` is a zero-initialised global (`global.c:83`, so `GAME_STATUS_UNINITIALISED`), written only by `set_game_status` (`global.c:278`). The host's game flow calls it:
- `init.c:354`: UNINITIALISED;
- `gameflow.c:713`: INITIALISING, at the start of `GAME_INITIALISATION_PHASE_SETUP`, before the session is created or joined;
- `flight.c:240`: INITIALISED, when the flight loop starts;
- `flight.c:524`: UNINITIALISING;
- `gameflow.c:322/977`: UNINITIALISED again.

**Why this function needs it.** Crates are part of a saved or joined session: `cg_pack.c :: pack_local_data` packs every cargo in `PACK_MODE_SERVER_SESSION` and `PACK_MODE_CLIENT_SESSION`, including its `LIST_TYPE_CARGO` link. While a session is being built or restored (INITIALISING), keysites are created or unpacked, `ks_creat.c` calls `update_keysite_cargo`, and the crates arrive through their own packed entities. Materialising crates at that moment would duplicate them. The guard means "the session's entity graph is still being assembled; do not derive crates from supply levels".

**Other campaign readers.** The same phase gates `force.c:238`, the force message responses in `fc_msgs.c` (Slice 5: `!= GAME_STATUS_INITIALISED` makes them ignore messages), `sc_msgs.c:186`, `sc_int.c:112` (an ASSERT), `ld_msgs.c` and `fw_dstry.c`.

**Decision: campaign state, not a port.**
- The game status is not a query about the environment. It is the campaign's own lifecycle phase, which the host moves through as it builds and runs a session.
- The core owns it the way it owns the comms model: `setGameStatus (status)` and `getGameStatus ()` in the core (C provenance `global.c :: set_game_status`, `global.h :: get_game_status`).
- `initialiseCampaignCore` resets it to the C static value `GAME_STATUS_UNINITIALISED`, and the host sets it when its own flow changes phase.
- The name `game_status_string` is not ported: nothing in the campaign reads it.
- There is no GameStatusPort: nothing would ever implement it differently from "return what the host last set".

## Investigation 4: executing the original in the C harness

- **`keysite.c` compiles whole.** It replaces the verbatim extract of `get_closest_keysite` (`c-reference/extract.mjs`), so no crate algorithm exists in harness code.
- **What its other functions name.** Those are FARP enabling, importance, attack notification, destruction, capture, repair, player suitability, dumps, landing sites, speech and MFD names. They are provided by original headers (`cmndline.h`, `misc/message.h`, `misc/msg_out.h`, `misc/tod.h`, `soundeff/speech.h`, `task/task.h`, `landing/landing.h`, `regen/rg_updt.h`, `session/session.h`, `fixed/fixed.h`) and by verbatim enum, typedef, define and prototype extracts. Whole headers are used where they are self-contained. `soundeff.h` and `effect.h` pull in the explosion and dynamics modules, so their few needed lines are extracted instead.
- **What they call.** 31 functions only those unreached functions call are fail-loud stubs.
- **What they read.** Seven data objects only they read are defined without meaning: `random_number_seed`, `command_line_capture_aircraft`, `speech_sector_coordinates`, the side name tables, `task_database`, and the string accessor table. The last has a fail-loud default. The harness never calls those functions, and none of them is in a dispatch table.
- **`get_game_status`.** It is now the original macro over an extracted `extern game_status_types game_status`. The harness defines `game_status` (host state, zero-initialised as in C), and scenarios set it. The former fail-loud `get_game_status` function stub is gone.
- **`get_object_3d_bounding_box`.** It returns the scenario's declared bounds for the requested index, and fails loudly for any other index. It used to fail loudly for every index.
- **Frozen outputs.** The existing 127 C-reference tests and all recorded fixtures are unchanged with `keysite.c` compiled whole.

## Selected boundary

**Ported to TypeScript:**
- `keysite.c :: update_keysite_cargo` as `updateKeysiteCargo` (`src/entity/special/keysite/keysite.ts`), statement for statement.
- The keysite `alive` bit-field and the `INT_TYPE_SIDE` getter (`ks_int.c`).
- The game status as core state (`src/core/game-status.ts`), reset to `GAME_STATUS_UNINITIALISED` by `initialiseCampaignCore`.
- `f64AddRTZ`: the double sum `(xmax − xmin) + 1.0` under round toward zero.
- Generated from the C:
  - `KEYSITE_DATABASE_AMMO_SUPPLY_USAGE` and `_FUEL_` (`ks_dbase.c`);
  - `KEYSITE_SUPPLY_REQUEST_THRESHOLD`, `CARGO_AMMO_SIZE` and `CARGO_FUEL_SIZE`;
  - `OBJECT_3D_SINGLE_CRATE`, whose index is computed from the `3dmodels.h` X-macro list and cross-checked by the harness, which looks the scenario's bounds up under the C value;
  - `GameStatusType`.

**Port:** `Object3DMetadata.getBoundingBox (objectIndex)` (`src/ports/object-3d-metadata.ts`).

**Boundary:** `ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES` reaches the unported force response (`fc_msgs.c`, Slice 5).
- In production it throws.
- Test runners use the existing `unportedMessagePolicy: "record"`, and the C harness records it the same way (`record_force_low_on_supplies`).
- Both print `message <force> <keysite> <message> <sub_type>`.
- Since Slice 5a (issue #12) the response is ported. It runs behind the same trace line, so this slice's fixtures are unchanged; its own boundary is `create_supply_task` (`docs/slices/force-low-on-supplies.md`).

**Not in this slice:**
- the callers (`ks_creat.c`, `ks_updt.c`);
- the rest of `keysite.c`;
- WUT overrides of the keysite database (`wutcfg.c`), as for the group database;
- the original crate geometry, which a deployment supplies through the port.

## Conformance

Every run exercises the Slice 3 lifecycle socially: a real keysite, sector grid and cargo list, with creation and destruction through the original cargo path. The observable output is the entity graph (heap order, cargo values, keysite cargo lists, sector lists), the replication operations, and the recorded `FORCE_LOW_ON_SUPPLIES` deliveries.

- **Behaviour matrix** (`test/scenarios/keysite-cargo.cases.ts`, 32 cases), organised by crate-count transition:
  - **creation:** none at exactly one crate; one; several, newest first; one fewer at an exact multiple;
  - **retention:** all at an exact multiple; no change above the threshold; steady state notifying on every call;
  - **destruction:** oldest first; several in list order; all; a later crate landing on a survivor, with index reuse; a negative size that refills the remainder after destroying (the only input under which a destroyed crate's x could matter);
  - **notification:**
    - a missing crate materialised instead of notifying;
    - exactly at the threshold, and just above it;
    - fuel's argument;
    - keysite-type gating: factory, power station;
    - no force for the side, which trips the receiver assert;
  - **ammo and fuel independently**, and the third (supplies) row;
  - **guards:** initialising, uninitialised, dead, not in use, the alive bit-field;
  - **positions:** zero-size crate, fractional bounds under RTZ, a row crossing a sector boundary;
  - **RTZ-sensitive counting:** 0.3/0.1 and 7.7/1.1, where rounding to nearest would create one crate more.

  Each case has hand-derived expected lines and absent prefixes. It runs in JavaScript and Lua 5.1, and against the executed original C, where the TypeScript output must equal the C's line for line.
- **Random differential** (`generateRandomKeysiteCargo`):
  - 1,000 fresh scenarios, TypeScript against the C, line for line. The generator must keep reaching creation, destruction, notification, create-and-notify, the receiver assert and heap exhaustion.
  - 150 recorded (`c-reference-random-keysite-cargo.cases.ts`) and replayed under Lua 5.1.
- **RTZ double sum:** 20,000 fresh sums plus boundary values against C, and 1,000 recorded and replayed in JS and Lua 5.1. The recording is separate, so the #7 float fixture stays frozen.
- **Frozen fixtures:** Slices 1–3 and #7 are byte-identical. `npm run cref:record` rewrites the three scenario fixtures unchanged.
- **Crate-row statement:** checked whole against the executed C expression; see Numerics.
- **Coverage:** 100%. **Mutation controls:** 23 for this slice, including three alternative decompositions of the crate-row statement (JS) and one under Lua; see `scripts/mutation-check.mjs`.
