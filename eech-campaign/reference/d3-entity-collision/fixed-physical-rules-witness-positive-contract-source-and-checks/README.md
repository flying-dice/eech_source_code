# D3 Task49: the changed_fixed_rule witness validates its intercepted final positive (UNREVIEWED CODE_FROZEN, test-only)

Code `879b0ebde110250169513dfd9fecc4c83f3b7b01` over metadata `18f86267ffd81c44f9abb003a16ebc0089da8421` (the first witness pack, code `96cc958d5`, preserved and not accepted) over the sealed runtime `681ccfe5476a7c0a90036bc01cd054358c751162`. Only `lua/entity-authority.lua` and `tests/entity_authority.rs` changed. Source and Linux checks only: no native actual, no review of this material, no CI of this head, no SourceClearance, no acceptance. The compact account is [fixed-physical-rules-witness-positive-contract-CODE-FROZEN-20261106.json](fixed-physical-rules-witness-positive-contract-CODE-FROZEN-20261106.json); the same reviewer's sealed `CHANGES_REQUIRED` report (`870453bc...`, one P2) is copied literally beside it; every run record is in `direct-records/`, the two input maps in `maps/`, hashes in `manifest.json`. Older math, profile, source and native histories are referred to by their sealed identities, not repeated.

## The finding (ENV-FIXED-RULE-WITNESS-POSITIVE-ACCEPTANCE-1, P2)

The mode holds the real `TEST` answer back from the campaign (it meets the provider's refusal of the same payload instead). The shared `candidate_reply_problem` checks only grammar and "12 or 16 words", so a positive with a non-finite word, scene zero or another scene's name was still counted eligible, restated and hidden behind the later refusal; the campaign could never apply its own contract to it.

## The correction

Before eligible credit, any state change or the extra evaluation, the intercepted positive must satisfy what the campaign's consumer demands (`mb_hostentity.h` `check_candidate_reply`, `mb_hostmodel.c` `check_target_point_identity`): a nonzero scene; exactly twelve grammar-valid pose words, each classified as **text and integer arithmetic on its exponent field** (no word is loaded as a float, so a signaling NaN or subnormal word is neither damaged nor alters the floating-point state); and the scene's own name.

**The expected name** is what the campaign accepted. Its identity check is fatal, so a reply is accepted exactly when the campaign asks its next question. In this mode only, non-refused replies of `entity_candidate`, `object`, `custom_object`, `animation` and `target_point_offset` (the operations whose provider reply echoes `compiled_scene_name(scene)`) are noted per scene and promoted at the next question. A scene with no accepted reply, or two different accepted names, has no binding: its positive is **not validated** (counted, never credited, never guessed; the verdict is `UNREACHED` naming the count). No production API, store or independent model image is used; the name table is not guessed.

A malformed positive is a reply fault with eligible 0, changed 0, **zero restatement and zero re-evaluation**; the campaign meets the original reply itself and the verdict is a failed-first run, never `UNREACHED` and never a pass. Unchanged: the same provider and unmodified actual payload, the subtype-only restatement under the strict `placed`/same key/no cursor acknowledgement, the declared extra Host-only evaluation (not a next campaign question), the real named refusal returned to the campaign, the four `fixed_rule_*` fields, and every default and other mode (name tracking is installed only for `changed_fixed_rule`; the scripted consumer gained the campaign's finite-word check).

## Red, green

| run | result |
|---|---|
| `red-96cc-positive-a` | first looping formulation, RED at its first fault; superseded |
| `red-96cc-positive-b` | **the RED** on the unchanged 96cc driver (tests only changed): 91 passed, 5 failed. Infinite/negative-infinite word, signaling NaN, scene zero, wrong name and unbound scene were each credited (eligible 1, changed 1) and hidden; the valid-positive control, including a subnormal word and a negative zero, passed |
| `green-positive-a` | 96 passed |
| `positive-fmt-apply` | layout only (the RED ran on the pre-layout file) |
| `positive-final-green-debug`, `-fmt-check`, `-clippy` | 96 passed / 0 / 0 on the committed bytes (debug, x86_64, `--locked`, `eech-world` only) |

Each fault test asserts eligible 0, changed 0, empty next and cause, request faults 0, reply faults 1 (the unbound scene: 0 and `UNREACHED`), exactly one fixed report (no restatement) and exactly six provider candidate calls (no re-evaluation, no refusal). The 90 existing tests, including the first witness's positive-first, strict-acknowledgement and negative controls, pass unchanged.

## Inputs, scope and what is not claimed

- The maps are targeted (not the 9769-file map, which the 96cc pack holds): the two changed paths by raw bytes, git status (only those two) and the tree ids of every production root and the shipping `campaign.lua` against 681, all equal; before and after the final checks equal except label and time. Against 681 the only non-reference paths that differ are `STATUS.md`, the card and the two files above.
- The Lua is passed separately to the sealed 681 binary (Git blob `e27c2da83ee59dda1ac614a36d0755698f302722`, sha256 `8cb88ddf2ab548728abbbd99e64bdf4c41258fe144cf9607e9ba6efbf1867892`; the 96cc literal was `917bff92...`); the 88 payloads are not a build of this source.
- The accepted-name oracle can leave a native run honestly `UNREACHED` if no accepted reply named the final scene. "Accepted" is inferred from the campaign's next question. The scripted provider proves the driver contract, not the real refusal bytes or retail eligibility; no coverage of all fifteen sub types or ordinary sectors is claimed. Only x86_64 debug ran; no workspace, matrix, i686, release, other binary or native run.
- Pending, no self-clearance: the same reviewer over only this material, the exact-head CI, and Root's ONE new negative on a fresh root (the prepared `fixed-rule-witness-input-20261106` root stays unused until a new source clearance). `fixed-physical-rules-source-review-native` stays OPEN; `REQUIRED_CITY_BOX_LIFECYCLE_RETENTION` stays the mandatory next production after the finite flags; all ten D3 requirements remain; no D4, D5 or TypeScript.

## Metadata provenance

This tracked report holds `identity.meta_commit` and `publication` as `null` placeholders. Root's external copy `R:/d3-evidence/fixed-rule-witness-positive-contract-CODE-FROZEN-20261106.json` is written after the metadata commit and differs from it in exactly those two places. `manifest.json` lists every other file of this pack with size and SHA-256; the pack is the listed files plus `manifest.json`.
