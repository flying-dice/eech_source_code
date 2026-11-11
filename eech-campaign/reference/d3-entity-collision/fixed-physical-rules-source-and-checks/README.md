# D3 Task49: the host derives the physical fixed/CITY collision-fixed and align rules (UNREVIEWED CODE_FROZEN)

Code `d2f5027c9154841c32f70ed87747b9d908e08e48`, parent `90b38189601bacf57684e944a3d0f35b5ad0e69e`. One finite cut, source and Linux checks only: **no native actual, no Windows build, no review, no acceptance.**
The machine-readable account is [fixed-physical-rules-CODE-FROZEN-20261106.json](fixed-physical-rules-CODE-FROZEN-20261106.json); every run record is in `direct-records/`; hashes are in `manifest.json`.

## The cut

The original derives the two physical flags of every fixed entity from its fixed record's sub type: `fixed_database[sub_type]` (`fx_dbase.c`), read by `fx_int.c` for `INT_TYPE_COLLISION_TEST_FIXED` and `INT_TYPE_ALIGN_WITH_TERRAIN`.
The table is constant data with 15 rows in the order of `enum ENTITY_SUB_TYPE_FIXED`; nothing writes it (a test reads every use under `aphavoc/source` and `modules`), and it holds two further columns that are the campaign's (the default shape, the ground radar's clutter flag).

Before this cut the campaign derived the flags and stated them to the host's record, so storing them had moved nothing. Now the FIXED_STATE and CITY_STATE reports state the **sub type** (fact 10 of both; the flag facts 7 and 8 must be zero) at every point that already stated a fixed entity,
and the host derives collision-fixed and align from it with its own 15-row table (`roster.rs`, `FIXED_RULES`), holds the sub type in the record the placement already has, and decides the narrow phase's candidate from it. The campaign's own getters are the conformance mirror compared with the host's flags.
`ENTITY_SUB_TYPE_FIXED_UNKNOWN` is one of the 15 declared values and derives no collision and no alignment; only a value outside the 15 is refused by name (`[roster.subtype]`), before anything changes.

A city carries the fixed record, so its report states its sub type too. Once the campaign has read a city's fixed facts the host needs its record: a city the campaign calls not fixed that the host derives fixed is refused (it used to be skipped on the campaign's word), and an absent or withdrawn record is refused at that stage, whatever the campaign's fixed, city and building words say.
The stages that read nothing of the city still need none. The city's box is the host's and does not depend on the sub type.

## Timing and lifecycle (verified in source)

The sub type is written by the creators' defaults and by creation attributes (before the entity is inserted: the insertion states it), by the live crater session unpack (before its insertion; the object unpack that also calls `unpack_fixed_data` is inside a block comment, and `ct_pack.c`'s unpack is an empty function),
and by the one setter in `fx_int.c`, which states the change at once. A relink is a removal and a new insertion that states the sub type again. No report exists before the entity is placed, the host never asks the campaign for a sub type and no identity is allocated, so the lazy object-ID timing is unchanged.
No floating-point computation, no RNG, and every report is stated from the same call sites in the same order.

## What did not move

The six draw files that read `align_with_terrain` straight from the table are compiled into the hosted engine; whether a headless run reaches them is not proved, so they stay a required final-census disposition. The default shape and clutter columns, `INT_TYPE_IDENTIFY_FIXED` (the campaign's task policy and the tallest-structure computation) and the keysite database are not part of this rule.
**`REQUIRED_CITY_BOX_LIFECYCLE_RETENTION` stays OPEN and untouched** (an empty city's relink or restore still seeds the new record's box from Core's mirror); the city's sub type neither seeds nor depends on the box.

## Evidence

RED on `90b38189` (provider tests through the public provider, source guards, the `eech-sys` build) and GREEN on the cut: see the report. The provider RED is an assertion failure for the missing capability, not a compile failure; the C dialogue canaries and the scripted driver tests name the new words and cannot be shown RED without a compile failure (declared).
Final chain on the frozen tree (all exit 0): fmt check, every `eech-world` test (55 binaries), every `eech-engine-sys` test with `entity-dialogue-check`, `eech-engine` and `eech-dc`, the four-crate all-targets check, the three fixture-feature builds, `eech-sys`, strict clippy on the four crates this cut touches.
Input maps before and after the final checks are literal equal (3953 files); 3950 are raw-equal to the commit's blobs and the three that differ are the inherited legacy tree-geometry Lua controls this cut did not change; the three changed files outside the map's roots are bound separately.

## The current CI repair (same batch, minimal)

`eech-sys`'s generated `eech_extracted.c` and the historical C harness's carry the extracted `en_list.c` insert/delete bodies, which call `campaign_place_*` through `mb_hostplace.h` — the include the extraction drops (implicit declarations at `en_list.c:498`, `503`, `571`, `575`, reproduced on the unchanged tree).
Both extractors now name the header; with no `EECH_CAMPAIGN_HOST_ENTITY` it compiles every report away (the original no-host list semantics). A source guard pins it. Not verified here: the Windows cross job, the historical harness build itself (`-m32`, the image has no 32-bit libc; all 55 of its units compile `-fsyntax-only` at `-m64`), and the CI lint step, which fails on a latent strict-clippy finding in `eech-dc/src/module.rs` (a file this cut does not change; reported, not fixed).

## The planned native witness

`changed_fixed_rule` (`lua/entity-authority.lua`, TEST-ONLY, host-side): after the first eligible lookup (an ordinary fixed placement answered `need_placement` over its held record, counted as `fixed_rule_eligible` before anything changes), the host restates ONLY that record's fixed state with the sub type UNKNOWN through the same provider, and the next, unmodified question of the same candidate meets the campaign's true collision flag against the host's derived false and is refused by name (`[candidate.mirror] ... collision-fixed`). New row fields appear only in that mode. The payload words of the two reports change (protocol only); counts, recordings, the terminal and default outputs are expected exact.
