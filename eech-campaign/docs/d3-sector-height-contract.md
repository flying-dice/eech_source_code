# D3: the sector's tallest-structure height — characterization and the concrete next move

Task EEDCS-49 (D3), characterization only, on the clean base `8a525b145`. **No production C, Rust or Lua, physical authority, ABI, shipping macro or baseline changed.**
The earlier ordinary long-run blocker `ENV-ENTITY-POSE-LONG-RUN-MOTION-1` stays open and `blocks_cutover`: nothing here is a cutover, and the finite CITY acceptance does
not close it. This batch is the one preparation batch (count 1 of 2) for the next production move named below.

## 1. What the value is

`struct SECTOR.tallest_structure_height` (sector.h) is ONE `float` per sector. It is **a high-water value, not the maximum of the current members**: it is raised when a
fixed entity is listed into the sector and is never lowered, recomputed or reset by anything except the sector's creation (zero fill) and a session load (unpack).

| | where | what |
|---|---|---|
| **zero fill** | `sc_seccreat.c` `memset (raw, 0, sizeof (sector))`, before `set_local_entity_attributes` | every sector starts at the all-zero word; a boot or `reinitialise_entity_system` creates new sectors |
| **raise** (the only computation) | `sc_msgs.c` `response_to_link_child`, `list_type == LIST_TYPE_SECTOR` | below |
| **restore** | `sc_pack.c` `unpack_local_sector_data`, `PACK_MODE_SERVER_SESSION`, last word of the sector's record | the saved word, bit for bit |
| **save** | `sc_pack.c` `pack_local_sector_data`, `PACK_MODE_SERVER_SESSION`, last word of the sector's record | the stored word as 32 raw bits (`FLOAT_TYPE_PROTOTYPE_1` is `FLOAT_PACK_TYPE_UNPACKED`) |
| **getter** | `sc_float.c` (`fn_get_local_entity_float_value [ENTITY_TYPE_SECTOR][FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT]`) | the only route to the value for every reader but the pack; the sector has no float setter |

The raise, exactly (`sc_msgs.c`): if `get_local_entity_int_value (sender, INT_TYPE_IDENTIFY_FIXED)` and the sender's `INT_TYPE_OBJECT_3D_SHAPE` is not
`OBJECT_3D_INVALID_OBJECT_INDEX`: `height = position->y + bounding_box->ymax` (a `float` local, one float addition; `position` is the sender's `VEC3D_TYPE_POSITION`, `bounding_box` is
`get_object_3d_bounding_box (shape)`, the box *with* lines), then `if (height > raw->tallest_structure_height) raw->tallest_structure_height = height`. A strict greater-than against a field that
starts at the zero word: a sum that is negative, +0, -0, NaN or -infinity never changes it, and a structure wholly below y = 0 is invisible to it. It is evaluated at the LINK_CHILD notification,
which the list funnel (`en_list.c`) sends after the roster hook and the list write. The tallest block runs BEFORE `add_mobile_values_to_sector`. The shape and the position are read AT ARRIVAL: a
later position setter in the same sector (no relist), a destroyed shape, a restore of a shape or a damage level never re-evaluate it.

**Removal** (`response_to_unlink_child`) only calls `remove_mobile_values_from_sector`; it never touches the height. A fixed entity that changes sector (`fx_vec3d.c`) is a removal, then
an insertion into the new list: the new sector is raised, the old one is NOT lowered. So a destroyed or moved tall structure keeps its clearance requirement for the rest of the run.

**Arithmetic facts.** No random draw. No table. The only floating-point operation is the one addition and its comparison. The engine entry runs with the rounding mode toward zero
(`set_fpu_rounding_mode_zero`), so a sum that is not exact in float is stored rounded toward zero (`1 + 1.5·2^-24` stores `1.0`; the largest finite plus itself stores the largest finite, not infinity).
Round-to-nearest gives `1 + 2^-23` and infinity; the control runs both and the stored word is the same on x86-64 (SSE) and i686 (x87, see the evidence pack).

## 2. Writer and reader census (SOURCE/BUILD-DERIVED, asserted by `tests/sector_height_guard.rs`)

The census is text patterns over `aphavoc/source` and `modules` and over the build's source list. It is **not** an exhaustive preprocessed-symbol or runtime census; the "compiled in" column below is inferred from the build
flags (`build/main.rs`) and the `#if` structure of the file, not measured, and the guard cannot close the whole-D3 final compiled-consumer disposition.

Writers of the member: `sc_seccreat.c` (zero fill), `sc_msgs.c` (raise, 2 lines), `sc_pack.c` (unpack, 1 line). Nothing else in `aphavoc/source` or `modules` names `->tallest_structure_height`
except the getter and the pack read. Readers of `FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT` (exactly five source files):

| reader | when | compiled in |
|---|---|---|
| `mb_hostscan.h` through `CAMPAIGN_SCAN_TALLEST` (`en_cllsn.c`) | once per sector the host names (the FIRST case), then passed unchanged into every offer of that sector | every build except the two that keep the original solver |
| `en_cllsn.c` `get_line_of_sight_collision_entity` original scan | once per sector of the window: `source_min_position.y > tallest` sets `ignore_all_fixed_entities_in_sector` | only the `#else` of `EECH_CAMPAIGN_HOST_ENTITY` (collision-fixture, model-fixture) |
| `fw_move.c` fixed-wing terrain follow | every waypoint step: `min_height = max (min_height, tallest - terrain_elevation)`, then `max (waypoint altitude, min_height)`; BEFORE the `EECH_CAMPAIGN_HOST_MOVEMENT` block hands `terrain->min_height` to the host | unconditional |
| `sc_pack.c` | save | every build |

The host already takes the value as a FACT: `mb_hostoffer.h` sends it as fact 12 (`CAMPAIGN_CANDIDATE_TALLEST`) of every candidate question and `entity_candidate_step.c` compares
`source_min_position.y > facts[12]` for every non-mobile candidate. `hc_step.c` (helicopter structure avoidance) walks the sector LIST, not the scalar. The ingredients of the raise are already host
data: the box top is the host-authored scene table (`mb_hostmodel.c`, installed once at boot, a non-finite bound is refused, a read before the install stops the engine; the shipping profiles stage
that accessor over `3dobjvis.c`, only `model-fixture` keeps the original), and the fixed entity's position and shape are stated to the host at the end of every insertion (`place_fixed_state`,
`mb_hostplacemap.h`; the eight fixed families, state words position, heading, shape, sub type).

## 3. Classification of the inputs

| input | class | note |
|---|---|---|
| finite position and finite box top; sums exact or rounded | supported, characterised | `sector_height` control, both rounding modes |
| subnormal, smallest normal, largest finite, +infinity stored | supported, characterised | exact words |
| negative, ±0 sums | supported, characterised | never store (zero floor); -0 is never stored |
| NaN or -infinity position | characterised, **not reachable from a healthy world** | never stored (comparison false) |
| box top | **always finite**: the host refuses a non-finite bound at install | NaN and infinite tops are unreachable in a hosted build |
| saved word: every finite word, ±infinity, a quiet NaN | supported, characterised | bit-exact through pack and unpack |
| saved signalling-NaN payload | **unavailable / not claimed** | an x87 load can quieten it; no test asserts it |
| client session (`PACK_MODE_CLIENT_SESSION`) and browse session | characterised | the record carries no height and restore leaves the client's own value |
| city entity (`ENTITY_TYPE_CITY`) | **unknown from source** | it has `IDENTIFY_FIXED`; its creation sets the invalid shape and the default-shape resolution is commented out, so it raises only if a creator gives it a shape. The host's city state carries no shape |
| mobile sender | characterised only as "not fixed" | the mobile side branch of the link response is out of scope here |

The classes above record what the original does and what a healthy world can reach; they are NOT a production policy (nothing here makes the move finite-only or refuses a word the original would keep).

Behaviours kept as they are, **not approved as correct for ever**: the value never lowers (a destroyed tall structure keeps its clearance); a structure wholly below y = 0 is ignored;
the fixed-wing clearance subtracts the terrain elevation from an absolute height (`tallest - terrain_elevation`), which looks like a unit mix and is preserved bit for bit; a client that
joins a server's save never receives the value and keeps what its own insertions made. Each is a candidate for an explicit, separate, reviewed semantic decision, never a side effect of the move.

## 4. What the evidence is, and what it is not

`eech-engine-sys` feature `sector-height-check` (TEST-ONLY, default and shipping builds unchanged; its own test binary `sector_height`) runs `csrc/eech_sector_height_check.c` against code the engine
library links: the sector's LINK_CHILD/UNLINK_CHILD responses (through `message_responses`), its float getter, the fixed entity's int and position getters, the hosted bounding-box accessor, and the
session's `pack_local_sector_data` / `unpack_local_sector_data` over the real pack stream. **Declared hand-made data**: the entity structs, one scripted model-provider callback (a box per distinct top, installed
by the real `campaign_model_install_scenes`) and the sector map. **The message table is not the public funnel**: `insert_local_entity_into_parents_child_list` (the roster hook, the list write, then these
notifications) needs a host roster and is not run. The control gives back the dispatch tables, `world_map`, `entity_sector_map`, the pack buffer pointers and the rounding mode on its only path;
`mb_hostmodel.c`'s installed scenes are private statics with no reset, so the safety bound is the process (own test binary, one test) and the module's own rule that every boot reinstalls them.
Four mutants of the real sources (last arrival wins, removal lowers, unpack skips the word, box top ignored) turn it RED; the sources were restored byte for byte. The source/build-derived census is
the guard test (6 tests). No retail data, native campaign, Windows build or Tacview recording is involved: nothing here is native evidence, provider-only replay is not either, and no Windows/native source
clearance or authority is claimed.

## 5. The real public trigger and consumer for Root's later native run (named, not run)

- **Trigger**: the existing public driver `crates/eech-world/lua/entity-authority.lua` (Lua host → `require ("eech_dc")` → the weapon-fixture DLL → the campaign) on a retail map, launching
  a weapon step whose segment crosses populated sectors; and, for the second consumer, a fixed-wing mover in the ordinary scenario.
- **Consuming phases**: `ENTITY_CANDIDATE` fact 12 on every offer of a non-mobile candidate and the FIRST-sector read of `ENTITY_SCAN` (the sector word the host sees), and the fixed-wing
  `terrain->min_height` carried by the fixed-wing movement question.
- **Observation to take**: per sector key, the word the campaign sends against the high-water the host computes from the fixed-state statements and its own scene table. The campaign's word is
  the reference; the host's is the candidate. A provider replay of the same words proves the provider, not the engine.

## 6. The concrete next production move (after the long-run blocker closes; NOT built here)

**One host-owned scalar per sector, written by the host's roster, read through the existing scan grammar.** No new framework, no second world, no mirror as sole authority.

1. **Writer**: the host's roster raises `sector_height[sector]` at the FIRST fixed-state statement that follows an INSERT of a fixed-family key (a relink is a removal and a new key, so it is a new arrival;
   later statements for the same key never raise it), with `h = fl32 (y + ymax (shape))` from its own record and its own scene table, under the same strict `>` against the zero word, with the original's
   per-operation float and the toward-zero profile (NARROW on x87, as the pose kernels do). Removal never touches it. The roster's RESET (phase 6) zeroes it. **Open input for this step**: the city
   (its state carries no shape) — settle by a source/native observation of whether any real city is created with a valid shape before deciding whether its state gains a word.
2. **Reads** (all through `ENTITY_SCAN`, no new op): the FIRST reply carries the sector's word beside the cursor (the scan stops calling `CAMPAIGN_SCAN_TALLEST`); a new phase (next free after 16,
   to be confirmed) asks it by sector coordinates for the fixed-wing clearance and for the save.
3. **Save and restore**: the pack asks the same phase for every sector in the original's order (z outer, x inner) and writes the host's word unchanged; the unpack states each word to the host
   (a second new phase, an exact acknowledgement) BEFORE it writes the legacy field, sectors before entities as `en_sessn.c` already orders them, so the load's insertions can only raise it.
   Client and browse sessions stay as they are.
4. **Transition**: the campaign's own field stays as the conformance MIRROR until nothing but the save reads it; a read where host and mirror differ is a named refusal, never a fallback.
5. **Refusal checks, built together on the same fixture** (`entity-authority.lua` scenario flags, the city-cut pattern): positive (host word equals mirror, bit for bit); missing (a provider without the
   phase: a named refusal, never the campaign's word); wrong (a word the mirror does not hold, a NaN, a sector outside the map); stale (a word that survives a RESET, a restore acknowledged for another
   sector, a restore after an entity insertion). Plus the original's order, FPU and rounding profile through the real C control, extended in place.

Core keeps the campaign's geography, route planning, timers, selection and awareness decisions; the height is physical environmental data the host supplies, and a DCS host answers it from its own
obstacle knowledge. **Still required and undecided here**: the broad DCS API and needed-consumer decisions, the remaining physical sector hints, the effectful whole-32-bit UPDATED,
shared terrain, the raw TREE/FOREST/CLOUD CRT rand/season/skin_random, the needed hierarchy services, the DCS workarounds, natural moving integration, the draw/align census, the final current
40/all-8/final-4-inputs, CI, independent review and the normal merge. No blanket render-dormancy claim is made. Any semantic change above needs technical-lead agreement first.
