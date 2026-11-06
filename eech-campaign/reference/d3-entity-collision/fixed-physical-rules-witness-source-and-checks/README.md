# D3 Task49: the changed_fixed_rule witness at the final placement question (UNREVIEWED CODE_FROZEN, test-only)

Code `96cc958d5fca9fdf847cf1853020583071dedde8` over metadata `33d1fc30e60477c26851204b88e9912d61726630`, over the sealed runtime `681ccfe5476a7c0a90036bc01cd054358c751162`. **Test driver and its Rust harness only.** Source and Linux checks only: no native actual of this material, no review of it, no CI of this head, no SourceClearance, no acceptance.
The machine-readable account is [fixed-physical-rules-witness-CODE-FROZEN-20261106.json](fixed-physical-rules-witness-CODE-FROZEN-20261106.json); the raw bindings are in [fixed-physical-rules-witness-raw-binding-96cc958d5-20261106.json](fixed-physical-rules-witness-raw-binding-96cc958d5-20261106.json); every Linux run record is in `direct-records/`; hashes are in `manifest.json`. The same reviewer's sealed report over the 681 runtime (`5fdab426...`, `findings: []`, conditional native clearance) is copied literally beside them.

## The blocker

Root's ONE real `changed_fixed_rule` run on the sealed 681 (outer, helper and native exit 1) completed its engine with no request or reply fault and 70040 keyed non-mobile questions, but `fixed_rule_eligible = 0` and `fixed_rule_changed = 0`: **UNREACHED**, no ordinary fixed placement was ever answered `need_placement`. That root is consumed and the attempt is preserved; it is not a pass and no actor, case, geometry or time was searched for. The old selector was an ordinary-need_placement precondition the retail case does not produce (the old `withdrawn_fixed_state` succeeded on the same case only because its predicate admits buildings listed under a city).

## The source reason for the new selector

`model_host.rs` `fixed_join`: for a building listed under a city (`of_city`) the flag and position are `(placement_given, true)`, and once the flag is given it is compared with `state.collision_fixed() && !mobile_flag` (`fixed_flag != expected_fixed` is `[candidate.mirror] the campaign's collision-fixed flag differs from the host's fixed state of the placement`). So a listed building whose placement question yields the final pose has consumed its held, sub-type-derived collision flag. An ordinary fixed candidate has the flag given from its fixed facts and the same predicate covers it. `fixed_join` returns before `candidate_in_store`, which takes the store by shared reference, so a refusal at the join runs no kernel, UPDATED getter, terrain read, random draw or movement step, and the join is integer-only (the floating-point state is untouched).

## The contract of the mode

At ONE actual non-mobile fixed-family final-placement question (placement and collision flag given, not a city, the served key held by an acknowledged report whose sub type is not UNKNOWN):

1. the UNMODIFIED payload is delegated to the real provider and its answer validated first: `test` with exactly twelve patterns. Only then is `fixed_rule_eligible` counted;
2. ONLY that record's sub type is restated as UNKNOWN, with the record's current other words (now kept from every acknowledged report in both windows, armed and before the launch), to the same provider under the fixed report's own acknowledgement contract: `placed`, the same key, no cursor. Anything else (a cursor, another key, another state, a provider that fails to answer) is `fixed_rule_changed = -1` with its own verdict and the payload is not re-asked;
3. the SAME payload (scene, depth, kind, definition and key identical) is asked of the same provider once more. This is a declared extra Host-only evaluation of one actual question across two controlled Host states, **not** a question the campaign asked and not counted as one (`candidate_after` stays 6);
4. the provider's own `[candidate.mirror] ... collision-fixed` refusal is handed back in place of the pose and must be the only fault (request faults 0, reply faults 1). A re-evaluation that is not that refusal is a failure of the claim, never a pass; no final placement question at all is `UNREACHED`.

The four `fixed_rule_*` row fields keep their names and are still emitted by every mode (zero or empty outside this one). Their meaning is declared in the driver header: `eligible` the final placement questions validated before the change; `changed` 1 acknowledged, -1 not; `changed_next` "placement" when the re-evaluated payload was the same placement's final placement question (never a next campaign question); `changed_cause` `fixed_rule_mirror_differs` when the failure is the mirror refusal. No default or control field was added or changed; the re-evaluation record (`scan.fixed.rule_reeval`) is internal and not emitted.

## Red, green

| run | result | meaning |
|---|---|---|
| `red-33d1-listed` | 87 passed, 3 failed | **setup accident**, not a RED: the harness listed the child as entity type 21 but reported type 6, so the controls failed at boot |
| `red-33d1-listed-b` | 88 passed, 2 failed | **setup accident** for the first test (mode `control` demands a mobile candidate); the second test already showed eligible 0 against 1 |
| `red-33d1-listed-c` | 88 passed, 2 failed | **the RED** on the unchanged 33d1 driver: the listed building reaches all six campaign questions and the old mode answers `UNREACHED: ... no staged candidate question of an ordinary fixed placement ... (1 fixed states acknowledged, 4 eligible lookups served, 6 keyed non-mobile questions)`; the positive control passed first |
| `green-a` | 90 passed | first GREEN after the driver change |
| `fmt-prelim`, `fmt-apply` | 1, 0 | layout of the new tests only (the RED ran on the pre-layout file) |
| `final-green-debug`, `final-fmt-check`, `final-clippy` | 90 passed / 0 / 0 | final, on the committed bytes (debug, x86_64, `--locked`; `eech-world` only) |

The three new tests: the positive-first and same-payload test (six questions and answers equal the control's, a seventh call that is the sixth's payload byte for byte, refused with the mirror reason, both reports `placed` under the building's key, no engine, movement, query or pose log, no roster removal); the strict-acknowledgement test (cursor, wrong key, wrong state, failed provider, each credited as nothing and never re-asked); and the negative controls (a dialogue that ends in `skip`, a building with no held state, an engine stop first, a reply fault first). Two of the three fail on 33d1; the third passes there as characterisation (the old driver was `UNREACHED` or failed first in those cases too). The 87 existing tests, including the ordinary-fixed `changed_fixed_rule` control (which now reaches the same mode through its final question), pass unchanged.

## Scope, reuse and what is NOT claimed

- Only `lua/entity-authority.lua` and `tests/entity_authority.rs` differ from the 33d1 head by raw bytes; against the 681 runtime they differ by those two plus the 33d1 metadata (`STATUS.md`, the card). The tree ids of every C, sys, engine, DC, Rust-production, Cargo, toolchain, `aphavoc`, `modules` and workflow root are identical, and the shipping `campaign.lua` is unchanged (`raw-binding`). The raw input maps before and after the final checks are file-for-file equal (9769 files; the maps themselves are external: 3.6 MB each, hashes in the report).
- The sealed 681 Windows builds (88 payloads per profile) are reused unchanged; a test-only Lua driver needs no rebuild and is passed separately (`--literal-driver` with the Git blob `917bff921c48c1aeed4d7b27f268d4a2c0e9c64a`, sha256 `25fe9bbf83c600a9c092cce8c5d3f0006821adcfed55a44bdce578cd1bdff959`, bound by Root to Git at this code commit). The 88 payloads are not a build of this source. The old literal (blob `55d808c1...`) is the driver of the UNREACHED attempt.
- Reused, not replayed: the combined source review `5fdab426...`, the exact 33d1 CI (all nine checks), and the native positive actuals (ordinary short literal 80fd, healthy comparison with only the four declared additions, 180 B physical terminal). Not run: workspace, `EECH_C_OPT=2`, i686, release, other `eech-world` binaries (including `model_target_point`), original-C, math, tree, scan or profile experiments.
- The harness's candidate answers are scripted (the real roster is read through a probe), so it proves the driver contract and the strict acknowledgement, not the refusal bytes of the real provider; the real provider's refusal after a restatement is pinned for an ordinary fixed placement by `tests/model_target_point.rs`, and for a listed building only in the opposite direction (campaign flag false against a colliding record). The restated-UNKNOWN direction for a listed building is the fact of the native run.
- A listed building is an actual required fixed-family consumer; no natural ordinary-sector coverage and no claim that all fifteen sub types are natively proven. Only x86_64 debug ran locally: the new tests have no CI yet, including both i686 jobs.

## Blockers

The seven source/CI blockers (`ENV-FIXED-RULES-RESTATEMENT-ACK-1`, `ENV-CI-DEBUG-SCAN-FPU-1`, `ENV-SCAN-PRE-SAVE-SUBNORMAL-FPU-1`, `ENV-CI-I686-FPU-TEST-GATE-1`, `ci-lints-latent-eech-dc-expect-err`, `ENV-TREE-NOHIT-RAW-WORDS-I686-1`, `ENV-I686-ENTITY-CANDIDATE-NARROW-ULP-1`) are recorded closed **only** to the scope of the same reviewer's `5fdab426...` and the exact 33d1 CI success, for the 681 runtime. `fixed-physical-rules-source-review-native` stays OPEN for this counterfactual. `REQUIRED_CITY_BOX_LIFECYCLE_RETENTION` stays OPEN and is the mandatory next production after the finite flags; the draw-file align census, sector height, UPDATED terrain, raw tree/forest/cloud RNG, hierarchy, DCS mapping, natural moving, final 40/all8, current CI, normal merge and whole-D3 acceptance remain. All ten D3 requirements stand; D4, D5 and TypeScript are not started.

## Metadata provenance

This tracked report holds `identity.meta_commit` and `publication` as `null` placeholders. Root's external copy `R:/d3-evidence/fixed-rules-witness-CODE-FROZEN-20261106.json` is written after the metadata commit and differs from it in exactly those two places (the metadata commit id, and the top-level `publication` block). `manifest.json` lists every other file of this pack with size and SHA-256; the pack is the listed files plus `manifest.json` itself.

## Next

The same independent reviewer follows only the new Lua and test material; Root observes the CI of the committed head and then runs ONE negative actual on a fresh root with the sealed 681 binary and this commit's Lua.
