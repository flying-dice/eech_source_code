# D3 physical flags: the per-entity UPDATED / MOVED / ROTATED word (characterisation, EEDCS-49)

Status: **characterisation only. No production flag moves. UNREVIEWED.** This records what the ORIGINAL code does, from the source and from a real-C control, so the next production move is concrete. It is the named required blocker `ENV-PHYSICAL-FLAGS-RAWWORD-CONTRACT-1`
(the "UPDATED truth/domain/guide contract" that `needed-consumer-api-disposition-open-rows` and `entity-object-collision` already declare missing). Nothing below is a design, an ABI, a trait or a layout; unknown facts stay unknown.

Evidence: `csrc/eech_physical_flags_check.c` + `tests/physical_flags.rs` (feature `physical-flags-check`, 199 expectations over the REAL getter, setter, flag arrays and `update_client_server_entities`), and the source census `tests/physical_flags_guard.rs`
(text patterns, not behaviour). The control loads no retail data, boots no campaign and runs no consumer body.

## 1. What the word is (source facts, verified against the code, not the English labels)

- **Not a per-entity field.** It is three GLOBAL bit arrays in `special/update/up_update.c`: `moved_entities`, `rotated_entities`, `updated_entities`, each `unsigned int [MAX_NUM_ENTITIES / 32]` with `MAX_NUM_ENTITIES = 131072 * 4 = 524288` (16384 words, 65536 bytes each).
  The "per-entity 32-bit word" is the 32-bit WORD OF THE ARRAY that holds the entity's bit: word = `entity index >> 5`, mask = `1 << (entity index & 31)` (`mobile/mb_int.c`). The key is the **entity heap index alone** (`entity - entities`): no type, identity, host id or life is part of it.
- **Setter** (`INT_TYPE_UPDATED|MOVED|ROTATED`, local setter only; no client/server setter is installed): `ASSERT (value == TRUE)` then `array [word] |= mask`. TRUE-only; the commented-out FALSE path of `up_update.c` was replaced by the reset below. With assertions off a FALSE value would still OR the bit (source fact, not run).
- **Getter**: `value = array [word] &= mask`. Observed with the real code: it returns the entity's **MASK** (a power of two; `INT_MIN`, i.e. negative, for bit 31), not 1; it **clears every OTHER entity's bit in the same 32-bit word** (so a read of one entity destroys up to 31 neighbours' flags and the answer depends on read order);
  it does **not** clear the entity's own bit (a second read returns the mask again). A getter of an UNSET entity returns 0 and clears its set neighbours. To an `if (getter)` consumer any non-zero value, negative included, is true; only bit 0 equals `TRUE`.
- **Reset**: `update_client_server_entities` does `memset (updated_entities, 0, MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8))` once per update ITERATION before the update loop. The count is a number of BYTES: 16384 of the 65536 bytes, **a quarter of the array**. Entity indices **0..131071 are reset; 131072..524287 are NEVER reset** (once set, `UPDATED` stays true for the life of the
  process). MOVED and ROTATED are reset by the same quarter-memset only behind `tacview_is_logging () && tacview_reset_frame ()` (not exercised; with logging off nothing resets them). This is a LEGACY DEFECT preserved as a compatibility fact; it is not repaired here.
- **Set sites**: UPDATED is set once per update-list member AFTER `update_client_server_entity (en)` (`up_update.c`), so during an entity's own update its bit is still clear (below index 131072). MOVED/ROTATED are set by the movement code after a step (`hc_move.c`, `fw_move.c` rotated, `rv_move.c` rotated, the hosted `rv_hostmv.c` after the host's step, the gunship `dynamics.c`).
- **Domain**: installed by `overload_mobile_functions`, called for the aircraft (fixed wing, helicopter), vehicle (routed, ship, anti-air, person), cargo and weapon families. A fixed OBJECT keeps the default function (a read of it would stop the engine). `INT_TYPE_MOBILE_MOVING` (read by the fixed-wing and routed update for the Tacview `moved` argument) is a DIFFERENT field (installed by the aircraft initialiser and read for vehicles too), not this word.

## 2. Consumers (the census is exact, pinned by `physical_flags_guard.rs`)

| class | where | use |
|---|---|---|
| **physical (UPDATED read)** | `en_cllsn.c` (the moving-mobile collision pass-through: if updated, old position = new - motion * dt, else new = old + motion * dt), `fw_msgs.c` (the pass-through check of a message receiver, the same arithmetic), `gd_supply_arrival.c` (the supply-arrival descriptor field `updated`) | decides a POSITION used by collision and arrival decisions: **physical** |
| **observational (MOVED/ROTATED read)** | `hc_updt.c`, `fw_updt.c`, `rv_updt.c`, `aa_updt.c`: `write_tacview_unit_update (en, moved, rotated, ...)` only under `tacview_is_logging ()` (`moved`/`rotated` are used as booleans there) | the Tacview recording: **observational only**; recording must stay observational |
| **writers** | the movement code and the update loop (table in `physical_flags_guard.rs`) | record that a physical step happened |

No campaign DECISION reads the words (no task, guide-policy or scheduling code). The scheduling (which entity is updated, in which order, how many iterations) is the Core's `update_client_server_entities`; the word is the physical RECORD that an entity was stepped this iteration.

## 3. A real contract mismatch found at the boundary (source-derived, reachability UNKNOWN)

`gd_supply_arrival.c` copies the RAW getter value into the supply-arrival descriptor (`descriptor.updated = get_local_entity_int_value (member, INT_TYPE_UPDATED)`, a `c_int`), and the Rust side validates it with `checked_bool` (`eech-engine/src/lib.rs`), which accepts exactly 0 and 1
and otherwise returns `invalid updated flag N`. The neighbouring flag of the same function IS normalised (`... ? TRUE : FALSE`). So a member whose UPDATED bit is set with a mask other than 1 (any entity index not divisible by 32) is a descriptor error. Whether this is reached: during the member's own update its bit is clear below index 131072, but at index >= 131072 the bit is
permanent after the first update (the quarter-memset), so such a member reads its mask on every later iteration. The shipping runs that passed may simply never have met an index >= 131072 mask != 1 on this path: **UNKNOWN**. This is a defect candidate, not fixed and not claimed reached.

## 4. Boundary: what is physical, what is Core, and what is NOT yet shown

Root's feedback of 15:50 (`Root-physical-flags-feedback-20261107-1550.json`, read before the freeze) fixes the constraints this section obeys: the accepted byte-identical behaviour stays mandatory; **the destructive `&=` getter and the quarter-memset cannot be declared outside the observable behaviour to simplify a move**; no boolean normalisation and no supply-descriptor fix is authorised
by this work; MOVED and ROTATED are NOT scoped out of whole D3; no preselected design.

- **Physical truth**: the three words record whether a mobile entity was stepped (UPDATED, read by the collision pass-through, the fixed-wing pass-through and the supply descriptor) or moved/rotated (MOVED/ROTATED, read by the Tacview writers). They are the physical RECORD of a step, and a move to the host may not leave a second authoritative store inside the Core. MOVED/ROTATED are observational but remain part of the required responsibility.
- **Core**: update scheduling (which entities, in which order, how many iterations), when each update starts and ends, the decision to call a consumer, the supply/arrival POLICY and the Tacview writers' own decisions.
- **What a host-owned version must reproduce, because the current facts make each observable** (none may be simplified away): (1) the complete RAW return of the getter (the mask, negative for bit 31), not a boolean; (2) the CROSS-ENTITY destructive read: a getter of one entity clears the other entities' bits of its 32-bit word, so any entity's answer depends on the order and extent of other entities' reads, and the reads of the three consumers are interleaved with other entities' updates;
  (3) the reset EXTENT (UPDATED: indices 0..131071 once per update iteration, never above; MOVED/ROTATED only behind the Tacview frame reset, with the same quarter extent); (4) INDEX REUSE (the key is the heap index alone and a reused slot inherits the bit); (5) the ORDER of the set (after the entity's own update) against the reads of the consumers.
- **A simple "has this entity been stepped" truth service plus a Core mirror is NOT demonstrated sufficient** by these facts: it cannot carry the raw mask, the neighbour effect, the reset extent or the index-keyed reuse. It is not proposed and nothing here preselects it. The key question the review must decide is where the word's WHOLE store (arrays, setter, getter, reset) lives and how its heap-index key (the legacy key; the host's identity is the roster host id) crosses the boundary without a second store.
- **Source-derived defect candidate, explicitly UNCONFIRMED and SEPARATE**: the supply-arrival descriptor copies the raw mask into a field the Rust side validates as 0/1 (section 3). It is not fixed, not normalised and not part of any proposed move; its reachability is unknown.

## 5. The bounded next production responsibility (for the SAME review; nothing moves now)

**Responsibility**: the whole per-entity UPDATED / MOVED / ROTATED store with its three operations (set after a step, the destructive masked read, the reset), as one unit, with the Core keeping the scheduling and every decision. Whether that is a host-answered read with a Core-informed set, or the store itself on the host side, is NOT chosen here; both must satisfy the five reproduction requirements of section 4 and the existing byte-identical
conformance, and no boolean, no mirror-as-answer and no descriptor change may be used to get there.
**The remaining REQUIRED blocker before any production move** (recorded, not met): the real immediate-consumer control. The smallest real consumer of the raw word identified from the source is `response_to_check_mobile_reached_guide` (`fw_msgs.c`, reached through the real `message_responses` table for a fixed wing): it reads UPDATED, then computes the previous position and the perpendicular distance to the guide line, and returns TRUE/FALSE.
Running it needs a hand-made fixed-wing record, a guide with a radius criterion, the delta time and the vector getters; it is the next bounded step, not done here, and the source-text census of section 2 does NOT meet this gate.

## 6. Limits (what this does NOT establish)

The consumer gate is NOT met (no consumer body ran: the pass-through arithmetic, the supply policy and the Tacview writers are census facts here); not exhaustive (representative indices and bit offsets, not every one of 524288); the Tacview reset path, the setter with FALSE, `-O2`, other platforms and other entity types were not run; reachability of section 3 and the entity-index range reached by real campaigns are not measured; the control uses hand-made
entities (the engine's own struct and tables) and an empty update list, so the update loop's order effect with real members is not shown. No numeric, RNG or geometry claim is made or needed (the accepted proofs are reused unchanged). Preparation count stays 2: this is the named required blocker's characterisation, not a third optional preparation.
