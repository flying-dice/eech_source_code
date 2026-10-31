# D3 G2: ground contact, production host handoff (EEDCS-48)

**Status: implemented in source; the first source (`02ccf06cc`) was built and measured by the Root and independently reviewed (CHANGES_REQUIRED, five P2 groups); the corrections below are written
in source and UNBUILT and UNRUN by the implementing lane.** This note describes what the change does, the contracts it fixes, the witnesses that were written for it, the corrections of the review and
the focused commands the Root needs to produce evidence. It reports no result of its own: nothing in the corrections has been compiled, built or executed by the author. No pass, G2 cutover or D3
completion is claimed. The Root's measured results for `02ccf06cc` (three-map finite production comparisons exact, public handoff healthy, direct provider checks) are recorded by the Root, not here.
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
| Authority/refusal | a REAL weapon launched through the campaign's own launch call: a changed valid predicate/impact reply changes the real result of the launch's frozen cohort (terminal fact; terminal position = the exact replied single); withheld/malformed replies stop the engine at the named ground marker after a healthy MATCHED launch; the same-DLL control is the unaltered provider. See the corrections below | `lua/ground-authority.lua`, `tests/ground_authority.rs` (env) |
| Physical characterisation | the established finite ground fixture, unchanged physical cases and folds, now measures the moved boundary (host-semantic profile; the original-only cache-index hashes are not relabelled comparable) | `lua/ground-characterization.lua`, `eech_ground_fixture.c` (comment only) |

The `ground_probe` seam exercises the ground queries, not `wn_move.c`'s launch glue; the weapon-authority witness is the launch-through-glue run. Neither the unit tests nor the guards of the harnesses
prove the DLL; only the environment-driven runs on a fresh production build do.

## Corrections after the independent source review of `02ccf06cc` (five P2 groups; source only, unbuilt, unrun)

The review (`independent-ground-production-source-review-02ccf06cc-20261031.json`) found no physical-kernel, copy, ABI, pointer or fallback defect and no physical mismatch; the Root's three-map finite
comparisons and public handoff stand and are not reopened. It required five things. Tests passing do not close them; these are the corrections and what each is judged by.

1. **GROUND-AUTHORITY-COHORT.** The case cohort is the births of the launch call itself, read before any frame runs and FROZEN (`cohort` row). A weapon the natural war creates later is a `later_birth`
   row, a terminal of a weapon outside the cohort is a `foreign_terminal` row, a second terminal of a member a `duplicate_terminal` row; none is ever a case result. The judge refuses a run with no
   complete evidence: every terminal field (`type code steps pos velocity lifetime`) present and non-empty with the numbers numeric, one terminal per member, no terminal row outside the cohort,
   the authority counters equal to the rows, a launch equal to the selected launcher/target/weapon. A control or a valid change must have ALL its cohort ended (`case_weapons_pending` 0), not merely not
   have raised at the settle deadline; a refusal may stop with an unfinished cohort but only after a launch MATCHED to the control (configuration, selected launch, launch text, cohort identities) and with
   its alteration actually sent at the named marker. The terminal of a changed run is compared per member, never the first terminal that arrives.
2. **GROUND-AUTHORITY-SINGLE-RECEIPT.** The provider's double is not the consumed single unless it is an exact single, and `+ 37.5` is not always one (`255.99998474121094 + 37.5` lies between two
   singles). The shift now moves the impact y to a multiple of 1/16 m (about 37.5 m up): such a number below 2^20 needs at most 24 bits, and the driver checks with its own exactness test that both
   the original and the moved y are exact singles (otherwise a witness failure, never a receipt). Every recorded impact reply carries `exact` (all three coordinates exact singles) and `altered`; the
   judge refuses any inexact replied point and requires a cohort weapon to end exactly at an ALTERED replied point that differs from the predicate-only run's. No tolerance, no rounding of artifacts.
3. **GROUND-ABOVE-CALLER-FP.** The three public ground entries (`host_terrain_ground_above`, `_below`, `_impact`) are now thin wrappers that save the caller's COMPLETE floating-point state (the x87
   environment by `fnstenv`/`fldenv`, and MXCSR by `stmxcsr`) before their first statement, run the unchanged entry body, and restore both on EVERY return path, including the early refusals whose finite
   validation raises flags (`infinity - infinity`). The internal round-toward-zero arithmetic, the original algorithms, the conformance probes' own use of the kernels and the generic campaign DLL
   rounding are unchanged: this only returns the caller's state. The previously retained actual MinGW-w64 12 posix control (fenv-only restores x87 and leaves the SSE rounding control where the kernel put
   it, sha256 `ef118cf1...`) is the reason and is reused, not rerun. Effect on validation scope: no change to any numerical result is expected; the finite three-map comparisons were measured on the code
   before this guard, so the Root must re-run them (or classify a guard-only change) before relying on them for the guarded code. Controls (`tests/terrain_backend.rs`, module `caller_floating_point`, x86 only,
   synthetic map, no asset): the state is set and read with the instructions themselves in each of the four caller rounding modes (both registers) with every x87/SSE exception flag and the x87 condition
   codes raised, and the x87 control and status words and MXCSR read back after a read answer, a decided answer, infinite-coordinate and infinite-world refusals (early) and off-terrain refusals (after the
   kernel began), for all three entries, must equal what was read immediately before the call; the answer must not depend on the caller's mode. Another architecture has no such test (the guard falls back
   to `fegetenv`/`fesetenv` there).
4. **GROUND-AUTHORITY-FINITE-IO.** The driver checks `warmup`, `settle` and `frame_ms` are finite and in range, requires the simulated clock to be a finite number that moves forward on EVERY frame
   (`STALLED CLOCK`), and bounds the frames by four times what warm-up and settle ask for plus a margin (`FRAME BUDGET`). Every `write`, `flush` and the final `close` of the evidence is checked
   (`OUTPUT FAILURE`), including a close that fails after the success row. A failure of the witness itself raises an error beginning `ground-authority:` after a best-effort `witness_failure` row; the
   engine-failure handlers re-raise it (also when the engine has wrapped it in its own marker text), so it is never recorded as an expected provider refusal and the process fails. The Rust parent now waits
   for each mode's child with a wall limit (default 3600 s: a retail boot, warm-up and settle cap fit with a wide margin; `EECH_AUTHORITY_WALL_SECONDS`, finite, 1 to 14400), drains both pipes while it
   runs, keeps the last 64 KiB of each, and on expiry kills the child and reports how long it ran and what it had said. Ordinary healthy query and update order is unchanged.
5. **GROUND-STRICT-LINT.** The thirteen indexed numeric-vector accesses in `eech-dc/src/module.rs` (`ground_record`, `ground_triple`, `call_ground`) are replaced by fixed arrays and destructuring. All
   finite/single, boolean/schema and single-field-read validation and every marker is as it was; no `allow`, no panic, no unchecked unwrap.

Controls for 1, 2 and 4 execute the REAL `lua/ground-authority.lua` in a real Lua state (`tests/ground_authority.rs`) against scripted collaborators (a scripted `eech_dc` engine whose events, clock and
provider traffic the test chooses, and a scripted output file). They are fixtures for the driver's bookkeeping: they invent no physics and say nothing about the DLL. They cover a later birth and a foreign
terminal that arrive before the case terminal, a missing and an empty terminal field, a duplicate terminal, an incomplete cohort (refused for a control, legitimate for a refusal after a matched launch), a
stalled clock before and after the launch, an exhausted frame budget, a write/flush/close failure (including a refusal mode and a close failing after the success row), the shift at 255.99998474121094 and a
point that cannot be an exact single. The bounded wait is exercised with real children (this test executable re-run): a hung child killed at the wall limit, a failing child and a chatty child that fills both pipes.
The judge itself is exercised on synthetic rows for every refusal above and for mismatched launches.

## Proposed Root verification (nothing below has been run)

Build identity to record: the head SHA of the PR, the feature set of each DLL, and the `eech-world` executable beside it.

1. Unit, guard and driver-control tests, no assets: `cargo test -p eech-dc -p eech-engine --lib` (bridge and callback tests), `cargo test -p eech-world --test ground_handoff --test ground_authority --test ground_provider --test terrain_backend`
   (the guards, the real-driver controls with scripted collaborators, the bounded-wait child controls and the caller floating-point controls run everywhere x86; the environment-driven parts print SKIPPED without
   their variables), then the strict gate that failed: `cargo clippy --locked --all-targets -- -D warnings` (the CI command), which must not be satisfied by an `allow`.
   The caller floating-point controls need the native kernel: run them where `eech-world` builds with its C sources (Linux and the actual Windows MinGW build).
2. Host transport: the same `ground_provider` test with `EECH_TERRAIN_DIR` set to a verified retail `map<N>/terrain` directory.
3. Public handoff GREEN on the production DLL built with `--features ground-fixture` (the driver's probe needs it): `EECH_GROUND_DLL=moved EECH_WORLD_BIN=<exe> EECH_GROUND_ROOT_REQUIRED=<fresh root>
   EECH_GROUND_ROOT_CONTROL=<any value>` with `cargo test -p eech-world --test ground_handoff`. The preserved RED is re-run with `EECH_GROUND_DLL=unmoved` against the frozen DLL, unchanged.
4. Authority/refusal on the production DLL built with `--features weapon-fixture`: `EECH_WORLD_BIN=<exe> EECH_AUTHORITY_ROOTS=<nine fresh roots>` with
   `cargo test -p eech-world --test ground_authority` (the modes can be restricted with `EECH_AUTHORITY_MODES`; `EECH_AUTHORITY_WALL_SECONDS` sets the per-process wall limit, default 3600). Judge the
   run by its printed `judged (cohort ..., N terminal(s), M impact repl(ies))` lines and by the retained `<mode>.jsonl` rows; a SKIPPED line proves nothing. The first retail run is also the first time the
   cohort, the exact-single shift and the settle cap meet the real weapon: a setup finding (no ground question after launch, a cohort that does not end, a terminal position that is not a replied point) is
   a result to report, not a reason to loosen a check.
5. Because the FP guard is in the kernel entry, repeat what the first source's evidence rested on, on the corrected source: the three-map finite production comparison (`ground-physical` fold, counters,
   restoration, byte identity of the repeat) and the public handoff (`EECH_GROUND_DLL=moved`), plus the direct provider checks (`EECH_TERRAIN_DIR`); classify any difference under `REVIEW.md`.
6. Regression and conformance: the M4 regression pack and the retained physical ground comparison (`ground-physical` fold, counters, `state_digest`) against the c119 raws, with any difference handled by
   `REVIEW.md` baseline governance; Tacview evidence if the war's weapon activity is affected. The host-semantic cache rows (`ground-profile`, `ground-semantic-record`, `ground-semantic-hash`) are never
   compared with the original `ground-record`/`ground-hash` rows.

## Limitations and unknowns

- The corrections are not built, not run, not reviewed: expect compile-level surprises in the new C inline assembly, Rust inline assembly and Lua/Rust driver controls (written without a compiler). The
  inline assembly is x86/x86-64 GCC syntax (the shipping MinGW-w64 and the Linux CI); no other compiler or architecture is covered.
- The caller floating-point controls observe the x87 control and status words and MXCSR; they do not compare the x87 tag word or exception pointers, and they cannot observe a leak inside the Rust glue
  after the call returns (all sticky flags are raised beforehand on purpose, so a flag cannot change). The other native entries of `terrain_ref.c` (point query, sweep, conformance) still restore with
  `fegetenv`/`fesetenv` only, as before: the finding and the guard concern the three ground entries.
- The authority witness depends on the case weapon actually asking the ground after the launch and on its terminal position being the replied impact point; when it does not, the run reports a setup
  finding (it never passes). Which unguided weapon and launcher it picks follows the war's roster at the chosen warm-up and is a precondition, not a claim about campaign choice.
- Particle and avionics ground callers are covered by the contract and the characterisation fixture, not by a dedicated war scenario; whether a given reference war reaches them is evidence to be read from the
  provider counts, not assumed.
- The terrain-fixture trace (`eech_engine_terrain_trace`, the EEDCS-47 pack) records the point queries the DLL makes. Ground questions that formerly made the DLL query its own record are now the
  host's, so a war in which weapons or avionics asked the ground will show fewer client point queries than the frozen EEDCS-47 baseline: a change of observer coverage, to be classified under
  `REVIEW.md` baseline governance, never re-recorded silently.
- Each ground question is now a host round trip (a Lua call): the cost was not measured. A hot caller such as an avionics scan may need a measured assessment before it is called acceptable.
