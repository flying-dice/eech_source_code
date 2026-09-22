# Slice 1: `assess_group_supplies` (frozen)

**C:** `aphavoc/source/entity/special/group/group.c :: assess_group_supplies`, and its callees
**TS:** `src/entity/special/group/group.ts :: assessGroupSupplies`

## Callers in EECH

Both callers are physical-observation handlers in `entity/mobile/mb_msgs.c`:

- line 897: a ground force defending reduces its members' fuel, then assesses.
- line 1457: a member landed at a keysite, was refuelled and rearmed, then assesses.

In the port these become the `LandingObservation` / `CombatObservation` slices
(architecture question 10). The campaign accounting stays in the core.

## Dependency trace and classification

| Dependency | Where | Class | Port or ported as |
|---|---|---|---|
| `get_local_entity_data`, `get_local_entity_type` | en_main.c | entity core | `entity.ts` |
| `INT_TYPE_RESUPPLY_SOURCE` → `group_database[sub_type].resupply_source` | gp_int.c, gp_dbase.c | campaign database | generated `c-group-database.ts` |
| `INT_TYPE_GROUP_MODE` → guide stack non-empty ? BUSY : IDLE | gp_int.c | entity core | group overload |
| `INT_TYPE_SIDE`, `raw->side` | gp_int.c | entity core | group overload / raw |
| `get_local_force_entity` → session `LIST_TYPE_FORCE` | force.c, session.h | entity core | `getLocalForceEntity` |
| `notify_local_entity (ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES, force, group, cargo)` | en_msgs.c | entity message | `notifyLocalEntity`; the response is unported (next slice) |
| `get_local_entity_parent (group, LIST_TYPE_KEYSITE_GROUP)` → shared `group_link` | gp_list.c, get_prnt.h | entity core | shared link in `en_list.ts` |
| `get_closest_keysite` | keysite.c | campaign core | `getClosestKeysite` |
| `get_local_entity_vec3d_ptr (group, VEC3D_TYPE_POSITION)` → leader's `mob.position` | gp_vec3d.c, gp_ptr.c, ac_vec3d.c | **physical observation** | `MobilePhysicalState` port |
| `get_approx_2d_range`, `get_2d_range`, `bound`, `KILOMETRE` | modules/maths | pure maths | `src/core/maths` |
| keysite `FLOAT_TYPE_*_SUPPLY_LEVEL` get/set | ks_float.c | entity core | keysite overload |
| `set_client_server_entity_float_value` → `set_server_float_value` → `transmit_entity_comms_message` | xx_float.c, en_comms.c | **network transport** | local set + `EntityReplication` port |
| `debug_log` | | compiled out | not ported |

## Behaviour matrix

Every row is a case in `test/scenarios/assess-group-supplies.cases.ts`. Each case
runs three ways: under JavaScript, under Lua 5.1, and through the executed
original C.

| Resupply source | Condition | EECH behaviour | Cases |
|---|---|---|---|
| GROUP | ammo < 100 | notify the side's force: `LOW_ON_SUPPLIES`, `CARGO_AMMO` | `group-low-ammo-requests-ammo`, `group-just-below-100-requests` |
| GROUP | ammo ≥ 100, fuel < 100 | notify `CARGO_FUEL` | `group-low-fuel-requests-fuel` |
| GROUP | both low | ammo only (`else if`) | `group-low-ammo-and-fuel-requests-ammo-only` |
| GROUP | both ≥ 100 | nothing | `group-full-requests-nothing`, `group-over-full-requests-nothing` |
| GROUP | several forces | first force whose side matches | `group-request-goes-to-force-of-own-side` |
| GROUP | busy | still requests (mode not consulted) | `group-busy-still-requests` |
| GROUP | no force for side / no session | `ASSERT (receiver)` | `group-without-force-asserts`, `group-without-session-asserts` |
| NONE | any | nothing | `none-source-does-nothing` |
| KEYSITE | busy | nothing | `keysite-busy-group-not-resupplied` |
| KEYSITE | idle, both ≥ 100 | nothing | `keysite-full-group-not-resupplied` |
| KEYSITE | idle, ammo and/or fuel < 100 | `required = bound (100 - level, 0, keysite level)`; the keysite is set first, then the group | `keysite-rearm-and-refuel-limited-by-stock`, `keysite-rearm-only`, `keysite-refuel-only` |
| KEYSITE | keysite stock 0 | values still set and transmitted | `keysite-empty-stock-still-transmits` |
| KEYSITE | keysite stock < 0 | `bound` returns the negative level: the group **loses** supplies and the keysite returns to 0 (EECH quirk, preserved) | `keysite-negative-stock-drains-group` |
| KEYSITE | group level < 0 | required > 100, the group is topped up to 100 | `keysite-negative-group-level-tops-up-to-100` |
| KEYSITE | float values | float narrowing on each assignment | `keysite-float-arithmetic` |
| KEYSITE, no keysite parent | a keysite within 1 km | **first** in-use keysite within 1 km in list order (not the nearest) | `closest-keysite-within-1km`, `closest-keysite-early-out-is-first-in-list-not-nearest` |
| KEYSITE, no keysite parent | none within 1 km | nearest by approximate range; a tie keeps the first | `closest-keysite-beyond-1km-picks-nearest`, `closest-keysite-tie-keeps-first` |
| KEYSITE, no keysite parent | unused / other side | skipped / not searched | `closest-keysite-skips-unused`, `closest-keysite-only-own-side` |
| KEYSITE | independent group (parent is the force through the shared link) | parent type ≠ KEYSITE → search | `independent-group-parent-is-force-not-keysite` |
| KEYSITE | rearm and refuel | the keysite is resolved twice | `closest-keysite-rearm-and-refuel-look-up-twice` |
| KEYSITE | nothing found | `ASSERT (keysite)` | `no-keysite-found-asserts`, `no-leader-no-candidates-asserts-keysite` |
| KEYSITE | group without members | NULL position: `ASSERT (v2)` in `get_approx_2d_range` | `no-leader-asserts-in-range` |
| KEYSITE | no force for side | NULL dereference in `get_closest_keysite` | `no-force-for-side-dereferences-null` |
| `get_closest_keysite` | sub type, exclude, `actual_range` (approximate on early out, exact otherwise, `FLT_MAX` when none), inside-only, NULL position, `ASSERT (min_range > 0.0 \|\| outside_of_range)` | as in C | `closest-*` |

## EECH invariants represented

- `ASSERT (receiver)` in `notify_local_entity`, and its message range assertion.
- `ASSERT (keysite)` after the keysite lookup.
- `ASSERT (v1)` / `ASSERT (v2)` in the range functions.
- `ASSERT (min_range > 0.0 || outside_of_range)` in `get_closest_keysite`.
- `debug_assert (get_local_entity_type (force) == ENTITY_TYPE_FORCE)` in
  `get_local_force_entity`.
- `ASSERT (pred != parent)` and the en_list.c validation "entity already in list"
  in list insertion.

The assertion expressions are the C `#E` text, and the C reference compares them
literally.

## Evidence

| Check | Command |
|---|---|
| 41 matrix cases, JavaScript | `npm test` |
| 100% statements, branches, functions and lines | `npm run coverage` |
| Matrix + 250 C-recorded scenarios + float edge cases under Lua 5.1 | `npm run test:lua` |
| Matrix expectations hold for the executed C; 1,500 fresh random scenarios TS == C; ranges bit exact | `npm run test:cref` |
| 13 behavioural mutants, including one Lua-only mutant, all killed | `npm run mutation` |
