# D3 G2: ground contact, reference-host preparation (EEDCS-48)

**Status: Linux build, existing terrain-library guards, vocabulary check, formatting and host Clippy pass. Retail Windows conformance and independent review are pending.** Branch `codex/d3-ground-host-preparation`, base `1b6218b06a0d0ca61f884a4b913c0c1e5ded3b01`. This prepares the HOST side of
the below-ground and ground-impact extraction. It binds nothing: the campaign DLL (`wn_move.c`, `mb_hostterrain.c`, the public module bridge) is unchanged and still computes both
itself, and the shipping behaviour is unchanged. Production cutover follows Task47 delivery and integration re-verification (ledger 14548994; Epic EEDCS-21). No completed extraction, shipping
pass, G2 cutover or D3 completion is claimed. The original characterisation (`m5-ground-characterization.md`) and its retained c119 raws are the oracle and are not touched.

## What the host now has

`eech-world/csrc/terrain_ref.c` (and `.h`) gains two operations, copied from the original expression for expression, statement for statement, with the same float/double literals, over
the host's own loaded sectors and the point solver the host already has (`host_get_3d_terrain_point_data`, `record_from_hint`, `hint_from_record`, `FE_TOWARDZERO`, the process-wide solver lock):

| Operation | Original | Result |
|---|---|---|
| `host_terrain_ground_below` (`Terrain::ground_below`) | `point_below_ground` (`en_world.h`), with `point_above_ground` as the internal helper the impact search needs | `below`; `queried` (the terrain was read); when queried, the complete record the last read left (`host_terrain_answer`: values, `stored`, optional opaque hint) |
| `host_terrain_ground_impact` (`Terrain::ground_impact`) | `get_ground_impact_point` (`en_cllsn.c`) | both endpoints after the original's `+0.001` / `-0.001` corrections, the impact point, the surface TYPE the last query left, the record, and a count of the original's ASSERT conditions that were false |

Kept exactly: the `||`/`&&` short circuits (`point_above_ground` reads the sector maximum first and skips the elevation when it decides; `point_below_ground` answers before any read outside the
world area); the double-literal corrections; the float midpoint order with the double `0.5`; `get_sqr_3d_range`; `length = 100.0` set before the loop; the final `get_3d_terrain_point_data`; and
**the one shared record carried through the whole sequence**. The original's `ASSERT (!point_above_ground (below_ground))` is inert in the characterised build (a log, not a stop) but its
condition is EVALUATED and reads the terrain, moving the record between the corrections and the bisection; it is evaluated here in the same place and a false one is counted, never fatal.

## Contracts (the data shape a later client binding needs; nothing is bound now)

- **World area is campaign data, supplied.** `point_inside_map_area` compares x and z with the campaign's world-map extents, which are not assumed equal to the terrain's (the fixtures prove equality for
  the three retail roots only). The caller passes `{min_x, min_z, max_x, max_z}` (`host_ground_world`); a later client request would carry the four floats the DLL reads from its world map at the call.
  The host never decides "off the map" from its own extents for the below test, and no world Y or height value is needed or invented.
- **The class mapping stays in the campaign.** The impact returns the EECH `terrain_types` id the final query left in the record, not a class; `get_terrain_type_class` stays with the DLL (as
  `mb_hostterrain.h` already says: the campaign keeps terrain class).
- **The aliasing of the impact output with the below endpoint is caller glue.** Both corrected endpoints and the impact point are returned, so `wn_move.c`'s `(&p, &q, &q)` shape and the separate-output shape are
  both reconstructible; the conformance probe rebuilds both and compares them.
- **The record crosses by value.** Not an index, pointer or layout: the point answer's existing fields, plus `queried`. When `queried` is false the answer carries no record and the caller's record must be
  left entirely as it is (a true short circuit). A generic provider with no hint is valid: a call with no hint starts from a freshly cleared record and a result with no hint is valid.
- **Unsupported, refused with a message:** a coordinate that is not finite; any read of the sector maximum off the terrain's own extents (the original stops the engine there); an endpoint of an impact outside
  the terrain. **A hint the map did not issue is refused when the terrain is first READ**, never earlier: a below test decided by the world area or by the sector maximum never uses the hint, as the original
  never reads its record there. A refusal says nothing about whether the original would have queried.

## Conformance (host-only; patterned on `terrain_conformance`)

`host_terrain_ground_conformance` (`Terrain::ground_conformance`, CONFORMANCE ONLY) regenerates the original fixture's cases (`eech_ground_fixture.c` at c119a7370: the private xorshift, `make_point`,
`make_impact`, `point_height`, the coordinate helpers and the case tables are reproduced in draw order and arithmetic), makes the same cold and warm calls, and folds the same physical words in the same order.
Its inputs are given, not invented: the world area is the map's extents (the fixture refuses a map whose two differ; the three roots it ran on are equal), `MIN_MAP_Y`/`MAX_MAP_Y` are the
`en_world.c` literals (-8000.0, 65535.0) used only to place two of the sixteen height choices, and the terrain-type -> class table is parsed by the test from `terr_db.c`/`terrtype.h`, embedded at compile time with `include_str!` (crate-relative path, so a cross build ties it to the named build source and no runtime source root is needed; a hand-written table or any private data is never embedded). The host holds no class.
`tests/ground_conformance.rs` runs it against `EECH_TERRAIN_DIR` and the retained raw `EECH_GROUND_ORACLE`, and compares: the four physical folds; every physical counter; the emitted point and impact samples
word for word; and as separate semantic cache facts the fixture's "record touched" (here "queried"), `true_record_untouched`, `off_map_record_touched` and the warm repeats. The legacy `ground-hash` and
`ground-record` rows are original-only and are not compared. A run without its environment prints SKIPPED and proved nothing. The other tests there need only the map: the off-world and sector-maximum short
circuits, the lazy hint refusal, the caller-owned world area, the non-finite and off-terrain refusals, and the impact's cold = warm answer.

## Not done, and what stays coupled in D3

- **Binding still required:** the campaign supplies its world-area values as data; the host performs the physical off-map predicate. Do not retain a core physical predicate or assume world/terrain extents universally equal. The protocol binding and removal of the DLL sector rows used by G2 are cutover work after Task47 delivery.
- **Other consumers are not dormant.** `point_below_ground` is also used by the avionics (`co_eo.c` 578/618/654, `co_hms.c` 365/418/456/490, `co_radar.c` 493/546/584/618) and `point_above_ground` by the particle
  update (`pa_updt.c` 210); neither is covered by "no hosted caller", and `point_above_ground` stays an internal helper here. They, G3 (trees, per-sector surface types), loader and solver removal, and the remaining spatial,
  model, environment and time inputs are still whole-D3 scope. D4/D5 have not started.
- **Public RED for missing host authority** (a launched weapon's ground request on the healthy old DLL) is a later step, after green; not here.

## Execution checkpoint and remaining checks

1. Actual Linux C/Rust build, formatting/check, six existing terrain-library tests and all-target host Clippy pass0. The tested source hashes are retained. No retail map was supplied for this stage.
2. Actual no-environment ground_conformance execution: one vocabulary test ran and passed; THREE retail-dependent tests printed SKIPPED and checked nothing. Cargo reports four passing functions because those three return early; that total is not physical conformance evidence.
3. With a verified private root per map: `EECH_TERRAIN_DIR=<root>/common/maps/map<N>/terrain EECH_GROUND_ORACLE=<c119 ground.jsonl for that map> cargo test -p eech-world --test ground_conformance -- --nocapture`
   for Lebanon, Georgia and the separate Lebanon repeat. Expected: no difference lines; the printed physical folds equal the oracle's `ground-physical` row.
4. Then the existing `terrain_conformance`, `terrain_backend`, `terrain_provider_harness` tests, unchanged, to show the point solver itself is undisturbed.
