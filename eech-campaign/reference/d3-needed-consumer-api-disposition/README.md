# D3 needed-consumer and DCS API disposition: evidence packet

Task EEDCS-49 (D3), 7 November 2026 (UTC). Second and final preparation batch (count 1 -> 2 of 2). Source before: `c6426dc310796795a0d8c694cb985b65b86d8e7d` (clean).
**Documentation and source disposition only.** No Rust, C, Lua, runtime, build or ABI change; no test, compiler, benchmark, native, Windows or war run; no agent, MCP, authentication,
model, cache or merge action. `physical_cutover_clearance` remains false. `ENV-ENTITY-POSE-LONG-RUN-MOTION-1` stays OPEN with `blocks_cutover=true`; the four original f734 natives are untouched.

| file | what it is |
|---|---|
| [`../../docs/d3-needed-consumer-api-disposition.md`](../../docs/d3-needed-consumer-api-disposition.md) | the narrative and per-group tables (15 groups), generated from the matrix |
| `needed-consumer-api-matrix.json` | the machine-readable matrix: resolved source anchors, declared API anchors, accepted reuse, OPEN dispositions, next moves, limits |
| `needed-consumer-api-reference-requests-20261107.json` | optional primary-file requests (Root feedback channel copy) |
| `D3-needed-consumer-api-disposition-CODE-FROZEN-20261107.json` | the freeze record: hashes, commands with actual exits, limits |
| `manifest.json` | SHA-256 of every file here except itself |

Generator and checks are `R:/d3-evidence/build-needed-consumer-api-disposition-pack-20261107.py` (Windows Python 3.12.14, `-I -B -X utf8`). It resolves each source anchor from the clean tree
(symbol text on the stated line), each API anchor from the verified bundle, recomputes the bundle SHA-256 values against `PROVENANCE.json`, and refuses duplicate ids and unknown references.
This is documentation hygiene, **not** a test framework and **not** a runtime or conformance result.

## What this settles and what it does not

It records, for 15 groups, the current source function and caller chain, branch and profile, present ownership, what the same DLL still needs when DCS supplies physical movement and combat,
the host contract that exists, the bounded required outcome, and the exact declared DCS call or a named versioned data workaround. Unknown needed APIs and workarounds stay **OPEN** with the
concrete missing fact; nothing was reduced in D3 scope and nothing is called dormant by a name, a generic count, a source grep, a missing export or one unreached run.

New source findings recorded as source-derived and unmeasured (none is approved or classified as a defect):

- `UPDATED`/`MOVED`/`ROTATED` getters clear every other entity's bit in the same 32-bit word (`mb_int.c`), the per-pass `memset` in `up_update.c` clears only the first quarter of the array words,
  and Tacview logging reads two of the flags destructively.
- A reseed of the CRT stream exists at session-entity creation (`parser.c` -> `ss_creat.c` -> `session.c` -> `random.c`), after the terrain loader's cloud draws and before the campaign's factions;
  this refines, and does not close, `core-tree-loader-forest-cloud-rng`.
- Campaign setup and task generation read EECH model sub-objects (airfield and troop routes, patrol points, regeneration type): a hierarchy consumer class outside the Task50/52 scopes.
- Root's optional later addition `Airbase.ts` (blob `d3d7dedd81babeb779c65ceab45933cc5e932139`, own provenance file; the original 8-file manifest is unchanged) is incorporated in D3G-09: it declares runway, parking,
  tower, category and point records and **no** route polylines or anchored sub-objects, so the airfield-route, patrol and regeneration-type contracts stay OPEN (no closure by analogy).
- The six draw files' `align_with_terrain` reads are dispatched only from `scan_local_entity_3d_objects` (draw scene); the headless frame calls none of the draw entries (event dispatch not enumerated).

## Limits

Source anchors are source/build-derived, not an exhaustive measured preprocessed, symbol or runtime census. DCS declarations are shapes of the pinned author-maintained commit, not measured
DCS contracts, exhaustive absence or mesh guarantees; no live DCS/M6 execution is claimed. Reported evidence (Task52 CRT capture, card blocker texts, accepted-scope proofs) is cited as reported.
The final actual compiled-consumer audit, the current final 40/all-8/final-4-inputs, CI, independent review, normal merge and whole-D3 acceptance remain required. No further preparation batch
starts until the production blocker is resolved by an actual complete corrected pack and independent review. No D4/D5/TypeScript.

## Review

One same independent reviewer is requested for the new delta only (documentation correctness, anchors, reuse boundaries). This packet is not an acceptance.
