# D3 G2: ground contact, reference-host preparation (EEDCS-48)

**Status: kernel Windows conformance on all three retail roots passed and independent kernel review closed without findings at `14bfd3ed6`. The newer Lua transport and handoff harness passed Linux build/guards/lint; their Windows/retail execution and independent review are pending.** Branch `codex/d3-ground-host-preparation`, base `1b6218b06a0d0ca61f884a4b913c0c1e5ded3b01`. This prepares the HOST side of
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

## Host provider operations and the public handoff checks (second host-preparation step; Linux guards passed, retail execution pending)

**Still HOST preparation, not a delivered DLL capability.** The campaign DLL does not send these requests; nothing about shipping behaviour, `wn_move.c`, `mb_hostterrain.c` or the public module bridge changes.

### The provider operations (`src/terrain.rs`, the existing Lua provider; point, fast and map branches and the exact-bits coordinate reader are unchanged)

```
{ op = "ground_below",  world = { min_x, min_z, max_x, max_z }, x, y, z, hint? }
{ op = "ground_impact", world = { min_x, min_z, max_x, max_z }, above = { x, y, z }, below = { x, y, z }, hint? }

ground_below  -> { below = bool, queried = bool, record = nil | { last_elevation, normal = { nx, ny, nz }, type, stored, hint? } }
ground_impact -> { above = { x, y, z }, below = { x, y, z }, impact = { x, y, z }, type, record = { last_elevation, normal = { nx, ny, nz }, type, stored, hint? } }
```

- **The physics is the reviewed kernel's.** Each arm only parses, calls `Terrain::ground_below` / `ground_impact` and shapes the reply; there is no second Lua solver and no precomputed core off-map boolean. `world` is campaign geography data; the
  host performs the physical off-map predicate with it and never assumes it equals the terrain's extents.
- **Numbers.** Every coordinate, nested ones included, is read with the same `coordinate` reader: a Lua number only (a numeric string, boolean, table or nil is refused, naming where, e.g. `world.min_x: min_x must be a number, got string`),
  an exact single-precision value, the exact bits Lua holds (signed zero and subnormals survive in both directions), one field lookup per field. A missing nested table is `<field> must be a table, got nil`.
  The kernel's policy for non-finite input stays what it is (`terrain ground: a coordinate is not finite`, `... the world area is missing or not finite`); G1's point and fast handling is not changed.
- **Record.** `record` is absent exactly when `queried` is false, and then the future caller keeps its WHOLE existing record. When present it is the complete last record: `stored` is independent of the hint, and a host that keeps no hint omits it.
  `type` is an EECH terrain type id; the type -> class mapping stays the campaign's. The kernel's assertion counters are not part of the reply, and no hint, cache index or record layout is required of a generic host.
- **Hint.** Its shape is parsed here (host-private, `terrain hint: not a hint this host issued` for a malformed one); its tag is checked by the kernel only when the terrain is first READ, so a well-formed altered hint changes nothing for an answer
  decided outside the world area or above the sector maximum, and is refused (`terrain hint:`) on a read.

### Checks (Root owns every run)

Actual working-tree Linux execution completed with exit0 at 2026-10-29T17:18:24Z:
format/check, six terrain-library tests, six wrapper/verdict guards, two map-free
transport guards and host all-target Clippy passed. The real-map transport and
public native handoff functions explicitly SKIPPED without their required
environment. They are not retail or boundary proof. See the committed
`reference/d3-ground-host-preparation/lua-linux-working-completion.json` and log
for exact source hashes, invocation and scope. Windows execution and independent
review of this new Lua step remain pending.

| File | What it is | Needs |
|---|---|---|
| `lua/ground-provider-transport.lua` + `tests/ground_provider.rs` | DIRECT mode, in the actual host code (`terrain::install`) on a verified retail map: reply shapes, complete-or-absent record, caller-owned world, cold/warm hint flow, corrected endpoints and impact, signed zero and `math.ldexp (1, -149)` round trips, one getter read, about 20 strict refusals, non-finite and off-terrain refusals. Inputs are exact singles (integers, `math.ldexp`) | `EECH_TERRAIN_DIR` (else the map test prints SKIPPED; two map-free guards always run) |
| `lua/ground-handoff-verdict.lua`, `lua/ground-handoff.lua` + `tests/ground_handoff.rs` (guards) | the wrapper and its verdict against a stand-in DRIVER that asks chosen operations: delegation unchanged (answers and refusals), operation counts, its own arguments stripped, control fails on a ground request, no traffic is a setup failure, the RED marker only after a healthy run | nothing (always run) |
| `tests/ground_handoff.rs` (public control and RED) | runs the ESTABLISHED `lua/ground-characterization.lua` unchanged, under the wrapper, in a real host process against a real DLL with the `ground-fixture` feature | `EECH_WORLD_BIN`, `EECH_GROUND_ROOT_CONTROL`, `EECH_GROUND_ROOT_REQUIRED` (two fresh verified roots: the engine writes into its root); optional `EECH_GROUND_SCENARIO`, `EECH_HANDOFF_DIR` |

**Public control and RED.** Root packages a TEST-ONLY hybrid: the new host executable with the unchanged frozen ground-fixture DLL (`.../target/ground-fixture/coordinate-fixed-febbb7cc5`); its per-component identities are explicit and it is never shipping
evidence. The test writes the three scripts (embedded with `include_str!`, so the source named by the build is what runs) beside its output and starts the host hidden (`CREATE_NO_WINDOW` on Windows) with
`scenario=lebanon_retail points=200 impacts=50 repeats=1 seconds=3 after=1` (healthy 100 ms frames before and after the probe), `handoff=control|required`, `handoff_out=<summary>`.
The wrapper counts the provider's requests by operation and delegates every one to the host's own provider unchanged.

- **Control** (`handoff=control`, expected GREEN on the unmoved DLL): the driver completes (a `state_digest` row, `ground-semantic-record restored=1`), `point` > 0 (real original traffic), `ground_below` = `ground_impact` = 0.
  Summary row: `{"event":"ground_handoff","handoff":"control","healthy":true,"passed":true,"point":N,"fast":N,"map":N,"ground_below":0,"ground_impact":0,...}`.
- **Required** (`handoff=required`, expected RED on the unmoved DLL): the same healthy run, then exit non-zero with the stderr marker **`GROUND-HANDOFF-RED`** ("no ground_below or ground_impact request reached the host terrain provider (N point request(s)): the
  DLL still computes the ground predicates and the impact search itself"); summary `healthy:true, passed:false, ground_below:0, ground_impact:0`, written before the failure. A driver that fails first reports
  `ground-handoff: the healthy driver failed before the boundary could be judged: ...` (never the marker), and a run that asked no point question is a `SETUP FAILURE` (never the marker).
- **Limits.** The `ground_probe` seam exercises the ORIGINAL queries through the public Lua -> module path; it does not exercise `wn_move.c`'s launch glue (the weapon fixture build is a different DLL), so a later pass here is not the weapon
  authority/refusal witness, which remains a required cutover test. Real original traffic here is the G1 point/map/fast service only.

### Still coupled
The DLL's `wn_move` glue, a new `mb_hostterrain` operation and its client, the public bridge (`TERRAIN_BRIDGE` carries only map/point/fast), the DLL-side sector rows used by G2, the other `point_below_ground`/`point_above_ground` consumers
(avionics, particles), G3, loader/solver removal, and the spatial/model, environment and time inputs remain D3 scope. D4/D5 have not started. Nothing here is a cutover, a completed extraction, shipping evidence or Task48/D3 closure.
