# D3 needed-consumer/API disposition: documentation clarification packet

Task EEDCS-49 (D3), 7 November 2026 (UTC). **Documentation correction inside the completed preparation batch 2 of 2; not another preparation and not a physical extraction.** Source before:
`1dbf945281dc818d96491e5743e0e6237b07992f` (clean). The runtime sources are unchanged since the original disposition's source-before `c6426dc31` (`git diff` over `aphavoc`, `modules` and
`eech-campaign/crates` is empty, checked by the generator). No Rust/C/Lua/runtime/build/ABI file, test, compiler, native run, baseline, input, duration, seed, frame or tolerance changed.

The original packet [`../d3-needed-consumer-api-disposition/`](../d3-needed-consumer-api-disposition/README.md) (code `2c6652f4d`, freeze `7d33d600...`) is kept literally unchanged and its facts and results are
historical, never relabelled. This sibling carries the corrected matrix. The narrative [`d3-needed-consumer-api-disposition.md`](../../docs/d3-needed-consumer-api-disposition.md) is regenerated from it and points here.

| file | what it is |
|---|---|
| `needed-consumer-api-matrix.json` | the corrected 15-row matrix (113 resolved source anchors, 49 declared API anchors) with a `correction` block |
| `independent-needed-consumer-api-disposition-2c6652f4d-20261107.json` | the SAME reviewer's report, copied literally (SHA-256 `cc70a2ce...`) |
| `Root-needed-consumer-api-reference-response-20261107.json` | Root's post-freeze primary-source response, copied literally (SHA-256 `394d31fd...`) |
| `needed-consumer-api-reference-requests-clarified-20261107.json` | outstanding primary-file wording after the responses |
| `D3-needed-consumer-api-clarification-CODE-FROZEN-20261107.json` | the freeze record |
| `manifest.json` | SHA-256 of every file here except itself |

## The two findings (both stay OPEN until the SAME reviewer examines this delta)

- **ENV-D3-DISPOSITION-CALLER-PROFILE-1.** Inclusion, legacy versus active source, runtime guards, measurement and source-derived reach are now stated separately. `HOST_LOS`, `HOST_TERRAIN` and `HOST_ENV`
  are defined unconditionally by the current builders (`build/main.rs:271-273`); `HOST_TREE` is excluded only by `collision-fixture` (`:323-328`); `HOST_ENTITY` is excluded by `collision-fixture` OR
  `model-fixture` (`:338-339`). The two `mb_tgt.c` object-collision calls (`:455`, `:494`) are the legacy `#else` of `HOST_LOS` (`mb_tgt.c:94/128`), distinct from the active `campaign_los_query` ->
  `mb_hostlos.c:275`. The fixed-wing clearance (`fw_move.c:1243`) is compiled outside a feature macro and runtime-reached only for a task leader (`:1208`) with `terrain_follow != NONE` (`:1218`), not at
  every waypoint; the one repeated sentence in the current `docs/d3-sector-height-contract.md` is corrected (that document only; no C header edit). Rows D3G-01, 02, 06 and 07 changed.
- **ENV-D3-DISPOSITION-MOVING-COVERAGE-1.** The accepted CONFIGURED moving query-service scope (`reference/d3-entity-collision/configured-speed-query-authority`: real public path, HostCandidate,
  NeedUpdated, moving16, matched controlled input and Original at its retained identity) is reused as `R-CONFIGURED-MOVING`, not rerun and not broadened. A NATURAL campaign weapon engagement reaching a
  moving candidate and every unreached state remain OPEN (D3G-06, D3G-13). `ENV-ENTITY-POSE-LONG-RUN-MOTION-1` is untouched and still blocks cutover.

## Post-freeze primary response

`Object.export.ts` declares `Category` properties (BASE, CARGO, SCENERY, STATIC, UNIT, VOID, WEAPON) and `world.export.ts` declares `VolumeType` properties (BOX, PYRAMID, SEGMENT, SPHERE), each typed `number`.
Neither declares numeric values, volume parameter structures, object coverage or runtime behaviour, so no number is hard-coded here. `AI.ts`, `index.ts` and `AI.export.ts` declare constant groups, not typed
attack/fire/route payload schemas. They have their own provenance (Git blobs and SHA-256 in the matrix `api_bundle.files`); the original 8-file manifest and the separate `Airbase.ts` addition stay distinct.
No OPEN row is closed by a declared name, and no exhaustive absence is claimed.

## Checks and limits

The generator (`R:/d3-evidence/build-needed-consumer-api-clarification-pack-20261107.py`, adapted from the original) resolves every source anchor from the clean tree and recomputes API file hashes against
their provenance. That is documentation hygiene (parse, uniqueness, hash, anchor), not a test, compiler, runtime reachability or conformance result. Inclusion and reachability stay source/build-derived,
not a measured preprocessed/symbol/runtime census. This is UNREVIEWED: no independent review, approval or delivery is claimed by the author. All ten D3 requirements, every remaining physical service, the
final current 40/all-8/final-4-inputs, CI, review, normal merge, final compiled-consumer audit and whole-D3 acceptance remain required. No further preparation. No D4/D5/TypeScript.
