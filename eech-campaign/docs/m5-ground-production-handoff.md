# D3 G2: ground contact, production host handoff (EEDCS-48)

**Status: implemented in source, UNBUILT and UNRUN by the implementing lane.** This note describes what the change does, the contracts it fixes, the witnesses that were written for it and the
focused commands the Root needs to produce evidence. It reports no result: nothing here has been compiled, built or executed by the author. No pass, G2 cutover or D3 completion is claimed.
It builds on the reviewed kernels and Lua transport of `m5-ground-host-preparation.md` and measures against the original characterisation of `m5-ground-characterization.md` (retained c119
raws), neither of which is changed.

## What moved

Above/below-ground and the segment-against-ground search are physical questions about a world. The campaign module no longer answers them. For every compiled caller (not only the weapon flight):

| Original (campaign module) | Now | Callers |
|---|---|---|
| `point_above_ground` (`en_world.h` macro: sector maximum, then elevation) | `campaign_terrain_ground_above` → provider op `ground_above` | `pa_updt.c` (particles), the impact search's own uses (host-side now) |
| `point_below_ground` (`en_world.h` macro: world-area clause, then sector maximum and elevation) | `campaign_terrain_ground_below` → provider op `ground_below` | `wn_move.c` (weapon ground probe), `co_eo.c`, `co_hms.c`, `co_radar.c` (avionics) |
| `get_ground_impact_point` (`en_cllsn.c`: ±0.001 corrections, bisection, final point query, class) | `campaign_terrain_ground_impact` → provider op `ground_impact`; the class is still looked up by the campaign | `wn_move.c` (weapon ground impact lookup) |

`en_world.h` and `en_cllsn.c` swap under `#ifdef EECH_CAMPAIGN_HOST_TERRAIN` (always defined for the engine build, `eech-engine-sys/build/main.rs`); the original expressions stay compiled only
for a build without it. Nothing behind the new wrappers is a physical fallback: no predicate, sector maximum, elevation comparison or bisection exists in `mb_hostterrain.c`.

`eech_engine.c` is untouched (the request/reply structs live in `mb_hostterrain.h`; the callback type is an alias), so its embedded NUL is preserved by construction.

## The contract

Callback protocol (`mb_hostterrain.h`, mirrored in `eech-engine-sys/src/lib.rs`; fields are appended, no existing field moved):

- Request: `op` = `GROUND_ABOVE | GROUND_BELOW | GROUND_IMPACT`; `x, y, z` (position, or the above endpoint); `below[3]` (the below endpoint of an impact); `world[4]` = the campaign's
  world-map area `(MIN_MAP_X, MIN_MAP_Z, MAX_MAP_X, MAX_MAP_Z)` read when the question is asked; the shared record's `hint`; `winter` as on every request.
- Reply: `answer` (0/1) and `queried` (0/1) for the predicates, `above[3] below[3] impact[3]` for the impact, and the record in the fields a point reply already uses (`last_elevation`, `normal`,
  `type`, `stored`, `hint`).

Semantics, all preserved from the original and kept in the host's reviewed kernel (`terrain_ref.c`, one `ground_above` helper shared by the predicate and the bisection):

- **Order and short circuits.** `ground_above` reads the sector maximum first and the elevation only when it does not decide; `ground_below` answers before any read outside the world area.
- **Lazy record.** The campaign always sends the shared record's hint; the host validates it only when the terrain is first read. A decided answer (`queried` 0) carries no record and the campaign
  leaves its whole shared record exactly as it was.
- **The shared record** (`system_point_data`, what a `NULL` point record means) is rewritten completely from a queried reply: values, `stored`, hint (empty when the host keeps none; no-hint hosts stay
  valid). It carries no pointer, index or loader state.
- **The world area is campaign geography.** It is supplied by value with every ground question and is never assumed equal to the terrain's extents; the host performs the physical comparison with it.
- **Impact writes.** Inputs are read once by value; outputs are written above, below, then the record, then the impact point last, so `get_ground_impact_point (&p, &q, &q)` keeps the original aliasing.
  Distinct above/below buffers are required (aliasing them would change the original's reads) and are refused clearly.
- **Class.** The host returns the EECH terrain type of the final read; `get_terrain_type_class (get_3d_terrain_point_data_type (NULL))` stays in the campaign (G3 vocabulary).
- **No partial commit.** A ground reply is checked completely (finite numbers, predicate/`queried` flags 0 or 1, type in range, NUL-terminated hint) before any caller buffer or the shared record is written.
- **Refusals.** A missing, failing or malformed provider stops the engine at a named marker (`[terrain-provider:ground_above|ground_below|ground_impact.callback|answer|position|record|type]` from the Lua
  bridge, `[terrain-provider:ground.answer]` from the C client); there is no fallback.

Lua bridge (`eech-dc/src/module.rs`, `GROUND_BRIDGE`): requests reach the provider as
`{ op, world = { min_x, min_z, max_x, max_z }, x, y, z | above = { x, y, z }, below = { x, y, z }, hint?, winter }` and the reply is read once, field by field: Lua numbers only (strings are refused, not
coerced), finite, exact (`%.17g` text parsed to the nearest single, so signed zero and subnormals survive), booleans as booleans, `record` present exactly when `queried`, an impact's `type` equal to its
record's `type`, `stored` a boolean (nil means true, as for a point answer). The reference host serves them (`eech-world/src/terrain.rs`: `ground_above` added beside `ground_below` and
`ground_impact`, all over the same kernel; no second solver).

## Source findings (call graph, remaining dependencies)

- Compiled ground callers: `wn_move.c:417` (`point_below_ground`), `wn_move.c:431` (`get_ground_impact_point (&p, &q, &q)`), `co_eo.c`, `co_hms.c`, `co_radar.c` (`point_below_ground`),
  `pa_updt.c:210` (`point_above_ground`). All go through the macros/function swapped above. Headless inactivity of the avionics and particle callers is **not** assumed: their reference-host
  coverage is the same provider contract, exercised only where the war reaches them.
- Remaining physical dependencies, named honestly: the campaign still loads the terrain sectors itself (G3: trees, per-sector surface types, `terrain_3d_sector_*`, the map-facts agreement check),
  the terrain type → class vocabulary, `get_terrain_3d_point_sector_max_elevation` (no campaign caller remains; the ground fixture's own case generator and its placement helpers still read it, test-only),
  and the line-of-sight/derived/spatial services. This change does not claim removal of any of these.
- Other provider implementations: `lua/terrain-provider.lua` `generic` answers no ground operation (the first ground question stops the campaign at the ground boundary, deliberately: an analytic stand-in
  would be a second ground kernel), `raised` passes ground operations through unraised (documented as physically inconsistent for ground consumers), `nohint` strips the cache state from ground records too;
  `terrain-has-face.lua` `generic` does likewise.

## Witnesses (source written; nothing run)

| Witness | Claim it can carry | Where |
|---|---|---|
| Bridge unit tests | the real Lua bridge and parser: requests by value, exact numbers, record-iff-queried, strict shapes, every malformed reply refused at its marker | `eech-dc/src/module.rs` (tests `a_ground_question_reaches…`, `a_malformed_ground_answer…`) |
| Callback unit tests | every request field reaches the provider; reply fields exact; a wrong kind of answer, or an unrepresentable hint, writes nothing | `eech-engine/src/lib.rs` (tests `a_ground_question_reaches…`, extended `a_terrain_answer_of_the_wrong_kind…`) |
| Host transport | `ground_above` over the real retail map: decided/queried, record, warm/cold hints, refusals | `lua/ground-provider-transport.lua`, `tests/ground_provider.rs` (env: `EECH_TERRAIN_DIR`) |
| Public handoff | the moved DLL sends above, below AND impact questions to the provider after a healthy driver run (`required` is green); the unmoved DLL keeps its preserved RED (`GROUND-HANDOFF-RED`); a partial boundary is `GROUND-HANDOFF-INCOMPLETE` | `lua/ground-handoff*.lua`, `tests/ground_handoff.rs` (env, `EECH_GROUND_DLL=unmoved|moved`) |
| Authority/refusal | a REAL weapon launched through the campaign's own launch call: a changed valid predicate/impact reply changes the weapon's real result (terminal fact, terminal position = the replied point); withheld/malformed replies stop the engine at the named ground marker after a healthy launch; the same-DLL control is the unaltered provider | `lua/ground-authority.lua`, `tests/ground_authority.rs` (env) |
| Physical characterisation | the established finite ground fixture, unchanged physical cases and folds, now measures the moved boundary (host-semantic profile; the original-only cache-index hashes are not relabelled comparable) | `lua/ground-characterization.lua`, `eech_ground_fixture.c` (comment only) |

The `ground_probe` seam exercises the ground queries, not `wn_move.c`'s launch glue; the weapon-authority witness is the launch-through-glue run. Neither the unit tests nor the guards of the harnesses
prove the DLL; only the environment-driven runs on a fresh production build do.

## Proposed Root verification (nothing below has been run)

Build identity to record: the head SHA of the PR, the feature set of each DLL, and the `eech-world` executable beside it.

1. Unit and guard tests, no assets: `cargo test -p eech-dc -p eech-engine --lib` (bridge and callback tests), `cargo test -p eech-world --test ground_handoff --test ground_authority --test ground_provider`
   (the guards run everywhere; the environment-driven parts print SKIPPED without their variables).
2. Host transport: the same `ground_provider` test with `EECH_TERRAIN_DIR` set to a verified retail `map<N>/terrain` directory.
3. Public handoff GREEN on the production DLL built with `--features ground-fixture` (the driver's probe needs it): `EECH_GROUND_DLL=moved EECH_WORLD_BIN=<exe> EECH_GROUND_ROOT_REQUIRED=<fresh root>
   EECH_GROUND_ROOT_CONTROL=<any value>` with `cargo test -p eech-world --test ground_handoff`. The preserved RED is re-run with `EECH_GROUND_DLL=unmoved` against the frozen DLL, unchanged.
4. Authority/refusal on the production DLL built with `--features weapon-fixture`: `EECH_WORLD_BIN=<exe> EECH_AUTHORITY_ROOTS=<nine fresh roots>` with
   `cargo test -p eech-world --test ground_authority` (the modes can be restricted with `EECH_AUTHORITY_MODES`).
5. Regression and conformance: the M4 regression pack and the retained physical ground comparison (`ground-physical` fold, counters, `state_digest`) against the c119 raws, with any difference handled by
   `REVIEW.md` baseline governance; Tacview evidence if the war's weapon activity is affected. The host-semantic cache rows (`ground-profile`, `ground-semantic-record`, `ground-semantic-hash`) are never
   compared with the original `ground-record`/`ground-hash` rows.

## Limitations and unknowns

- Not built, not run, not reviewed: expect compile-level surprises in the new C and Rust (written against the reviewed sources without a compiler).
- The authority witness depends on the case weapon actually asking the ground after the launch and on its terminal position being the replied impact point; when it does not, the run reports a setup
  finding (it never passes). Which unguided weapon and launcher it picks follows the war's roster at the chosen warm-up and is a precondition, not a claim about campaign choice.
- Particle and avionics ground callers are covered by the contract and the characterisation fixture, not by a dedicated war scenario; whether a given reference war reaches them is evidence to be read from the
  provider counts, not assumed.
- The terrain-fixture trace (`eech_engine_terrain_trace`, the EEDCS-47 pack) records the point queries the DLL makes. Ground questions that formerly made the DLL query its own record are now the
  host's, so a war in which weapons or avionics asked the ground will show fewer client point queries than the frozen EEDCS-47 baseline: a change of observer coverage, to be classified under
  `REVIEW.md` baseline governance, never re-recorded silently.
- Each ground question is now a host round trip (a Lua call): the cost was not measured. A hot caller such as an avionics scan may need a measured assessment before it is called acceptable.
