# EEDCS-42: characterisation of the ORIGINAL line-of-sight solver (26 Oct 2026)

Observed through the public module (`eech-world.exe` -> `require ("eech_dc")` -> campaign) with a test-only fixture build.
**Nothing moved; the solver and its callers are unchanged.** This is baseline behaviour of unextracted code, recorded before
any extraction. It is not a delivery regression result, not a Tacview comparison, and not a pass of the full 3 h pack (which
the lead is running separately and which remains a required gate).

## Identities

| | |
|---|---|
| Solver source | `mb_tgt.c` at base `a7934aa7d1bcffc2ea5057bb3f1b0af332606c61` (D1 integration); only the gated `EECH_LOS_FIXTURE` shim differs |
| Fixture A | commit `da8b8c327008ce720c95e9f48971ff6b4aab0321`, `eech_dc.dll` `11294e6c7c2197f47ce8e7c94c4b7c70a475c6143a008533a52314d12a9a206e` (`fixture-BUILD-INFO.txt`); used for the Lebanon probe, Lebanon no-probe and Georgia runs and the controls |
| Fixture B | commit `ee7f6f79b9c3c43ad34ccf8c049306c67689a50b` (adds entity-kind counters, not in the hash), `eech_dc.dll` `99f189b1e5672e4abf44e0636800e0ad3be3193125970d0de2dfd8150dec6ce5`, `eech-world.exe` `67906e08...69e2`; used for one extra Lebanon run |
| Toolchain | repository `tools/Dockerfile` image (id `sha256:61999cc3...61c2`) in WSL2 `EECH-D2-Build`; rustc/cargo 1.99.1, MinGW gcc 12-posix `12.2.0-14+deb12u1+25.2+b1`; wrapper `tools/build-windows-los-fixture.sh` |
| Inputs | private copies under `R:/d2-evidence/los-characterization-baseline-20261026/roots/` (not in Git): `lebanon-probe`, `lebanon-noprobe`, `lebanon-kinds` (each IDENTICAL to `reference/lebanon-3h/inputs.json`, 1,179 files, `121a7890...`) and `georgia` (map3, IDENTICAL to `reference/m4-s3-mutation/inputs-georgia.json`, 1,174 files). All re-compared after the runs: still IDENTICAL. None is a root used by the lead's pack |
| Run | `eech-world.exe lua/los-characterization.lua root=<copy> scenario=<lebanon_retail or georgia_retail> seconds=600 frame_ms=100 seed=20261025 probe_at=<30,300 or empty> count=500 pairs=3 out=<trace.jsonl>`; the driver boots the campaign with seed 1 |
| Load | each run concurrent with the others and the lead's pack; about 10 min wall for 600 simulated seconds (6,000 frames) |

## Results (artifacts: `traces/`, `controls/`)

| Run | Run-wide queries | Visible | By criteria (visible/calls) | Hash | Probes |
|---|---|---|---|---|---|
| Lebanon, probe_at=30,300 (A) | 156,581 | 60,731 | `1`: 53,162/110,712; `31`: 7,569/45,869 | `d97f59bb364bb253` | 6 |
| Lebanon, probe_at= empty (A) | 156,581 | 60,731 | identical | `d97f59bb364bb253` | 0 |
| Lebanon, probe_at=30,300 (B) | 156,581 | 60,731 | identical | `d97f59bb364bb253` | 6 |
| Georgia (map3), probe_at=30,300 (A) | 21,104 | 1,646 | `1`: 912/20,166; `31`: 734/938 | `7fdc5ab0b8b5db5d` | 6 |

- **The probe is inert.** The run-wide fold with and without probes is identical in hash, count, visible count and per-criteria
  distribution, and `run_fold_untouched_by_probe` was `ok:true` at both probe times. Three independent Lebanon copies (and two DLL
  builds) gave the same run hash, so the run is deterministic at this level and a candidate's hash is a meaningful comparison.
- **What the campaign itself queries.** Only two criteria values occur in a run: `1` (coarse terrain only, the AI-target form) and
  `31` (all five phases). The fine-terrain bits (`2`, `4`) and the single object bits (`8`, `16`) are never requested by the
  campaign; only the probe reaches them. About 70% of Lebanon queries are coarse-only, so a campaign-level comparison constrains the
  fine and object phases only through `31` (29% of Lebanon queries, 4% of Georgia's).
- **Probe coverage.** Each probe takes 3 live opposed pairs (ground vehicle to ground vehicle, helicopter to ground vehicle, air
  defence to helicopter): 500 position queries plus 16 entity-form queries (8 masks, both ways round) = 516 queries each, none
  skipped. **The probes at 30 s and at 300 s chose the same pairs and returned identical hashes** (the units had not moved), so
  the six probes are 3 distinct sweeps (1,548 distinct queries), not 6. Mask `0` (NONE) was always visible (67/67, 74/74). On
  every pair the terrain masks (`1`, `2`, `4`, `7`) and ALL (`31`) returned both visible and blocked answers, so the terrain phases
  and the combined mask are exercised in both outcomes on all three pairs per map. The object-only masks are weaker: they block
  rarely and **not on every pair**. Lebanon, per pair: mask `16` (target-end objects) blocked 0 of 62, 0 of 62 and 2 of 67, so only
  the third pair ever blocked; mask `8` (source-end objects) blocked 3 of 77, 0 of 63 and 3 of 70, so the second pair never did.
  Georgia: mask `16` blocked 0, 0 and 7 and mask `8` blocked 0, 0 and 8, so only the third pair blocked for either. Whether any probe hit fell within the 12 m target-end
  exclusion distance was not recorded, so that exclusion is not designed probe coverage; it is covered by the focused kernel contracts
  instead (`los.rs`). The probe never uses a
  fixed-wing, ship or person as source or target (the driver takes the first three pair kinds), and does not probe a zero-length
  segment or a weapon target.
- **Call kinds (fixture B, Lebanon, 600 s; `traces/lebanon-kinds.los-kinds-final-block.txt`).** Every source is a host-identified
  mobile. Targets are mostly host-identified mobiles, with two exceptions. **WEAPON targets have no host id** (ship vehicle -> weapon,
  397 queries). **SITE targets** (routed vehicle -> site, 3,070 queries) are reported by the roster as having a host id, which is
  unexplained and not investigated here. Rows (final read): routed -> anti-air 50,301; routed -> routed about 37,508; ship ->
  anti-air 11,670; anti-air -> fixed-wing 10,473; helicopter -> routed 21,632; routed -> helicopter 7,259; anti-air -> helicopter
  5,865; helicopter -> anti-air 4,095; routed -> site 3,070; ship -> fixed-wing 2,716; helicopter -> helicopter 775; ship -> routed
  567; ship -> weapon 397; routed -> fixed-wing 241; fixed-wing -> ship 5; fixed-wing -> helicopter 4; fixed-wing -> anti-air 2;
  anti-air -> routed 1. No overflow. The routed -> routed final count was lost to stderr interleaving (the last observed value was
  29,873), so 37,508 is **inferred by difference** from the 156,581 total, not observed. Georgia kinds were not collected (fixture A).

## Boundary controls (existing fixture entry; not campaign policy)

`controls/lebanon-controls.jsonl` (20 frames) and `lebanon-controls-2500.jsonl` (2,500 frames), both Lebanon: 8 refused, 0 accepted
wrongly, 1 unavailable. A valid probe then still worked (`valid_after_refusals ok:true`), so no refusal poisoned the engine.

| Case | Result |
|---|---|
| source index 500000 (below `MAX_NUM_ENTITIES`, above the allocated heap); target index 500000; 1000000; 2147483647; -1 | each refused: `eech: bad argument: rejected by the engine` |
| same entity twice | refused |
| keysite as source | refused |
| an index the object list does not name (0) | refused |
| weapon as source | **unavailable**: none in flight after 20 or 2,500 frames (weapon targets do occur later, see above); not forced |
| unreached probe time (`seconds=10 probe_at=700`) | driver failed, exit 1, "only 0 of 1 requested probes were taken" |
| no opposed pair (`pairs=0`, an artificial trigger: the real world always supplied a pair) | driver failed, exit 1, `probe_unavailable` event and error |
| an unsupported mobile family | not exercised: all six mobile families are accepted by the probe |

## Source observations for the boundary design (no API implemented)

- Position-based callers (`ac_wpn.c`, `ag_attck.c`, `gd_cover.c`) pass positions that are not any entity's pose (an aircraft position
  lowered by its centre-of-gravity height, hypothetical cover positions). A query is therefore a function of two positions and a
  criteria mask plus, for the object phases, the two entities to exclude from the collision scan.
- While collision stays in the DLL (declared transitional coupling) the host needs no entity identity: the DLL keeps the pointers
  for its own `get_line_of_sight_collision_entity` calls. When D5 moves object collision to the host, exclusion needs host ids for the
  source and target. Weapons have none, so a target without an id must be an expressible case, not an error. No pointer or index may
  be put in a public payload.
- The entity form adds 1.5 m to each position when `INT_TYPE_IDENTIFY_VEHICLE`. That reads an entity attribute, so it stays with the
  DLL caller, ahead of any host query.
- `terrain_info` is zeroed only inside the coarse phase. Masks `2` and `4` alone read a hint that was never memset and rely on the
  build's `-ftrivial-auto-var-init=zero`. That is baseline behaviour of this build and is exercised by the probe; a host solver must
  start from an equivalent zeroed hint.
- Original-source defects: none observed that change results. Candidate questions recorded, not fixed: the SITE host id above, and the
  uninitialised-hint dependency.

## Limits

One seed, 600 simulated seconds, two maps; Georgia without kinds; 3 distinct probe pairs per map. The full 3 h pack and its byte
comparison are the lead's and are not claimed here. Runs were concurrent with other work, so wall times are not performance data. The
fixture C was compiled with `-w` (no warning-clean claim). The hash folds float bit patterns and answers, so it is equality
evidence, not an independent oracle of correctness.
