# D3 Task49: the public entity_scan answer bridge and the record a city removal retains (UNREVIEWED CODE_FROZEN)

Code `354abd9c04a53f1935919abc4fde1172b438c2b3` over metadata `6a6a74b61ec4ffe9af0b4578107e003edf2a1a62` (runtime code before: `215a8f7389dfc9ab1dac1df2f4e45344392967d9`). One file changed: `eech-campaign/crates/eech-dc/src/module.rs`. Compact report: `city-box-lifecycle-scan-reply-CODE-FROZEN-20261106.json` (byte-identical copy at `R:/d3-evidence/`). Root's actual native blocker `ENV-CITY-LIFECYCLE-NATIVE-REMOVED-NAME-1` is retained byte for byte in `Root-city-lifecycle-native-215a-boundary-failure-20261106.json`; the failed 215a control receipt and raw data and the 37/51/73/69/17-file packs stay unchanged where they are.

## Defect

In the actual 215a control the selected record 11193 was consumed once, the host answered one retaining REMOVE, and the run died at `entity_scan.answer: name is only for first, next and placed` (request and reply faults 0, no continuation, AFTER 0). The Lua function every host answer crosses (`MODEL_BRIDGE`, entity_scan branch) refused any name on `removed`. Letting the name through was not enough either: the bridge returned only the result, so `call_model` would have handed the engine a plain removal. The unit tests and the C and Lua harnesses never exercised the retaining answer through this converter (the C dialogue check uses a C callback; the driver tests script the provider), which is why it first showed in the native actual.

## Change (one file)

- Bridge: a `removed` answer may carry a name and the name is returned; a cursor (`p`) on a removal stays refused.
- `call_model`: `check_retained_record` admits a non-empty name on a removal only (1) at phase 5 (roster REMOVE) or 8 (city CHILD_REMOVE), the phase word of the question compared bit for bit, (2) as a roster key (one to twenty decimal digits, no leading zero, as the C `roster_key_is_valid` reads one) and (3) at phase 8 only as the list the removal was asked about (first word of `<list> <child>`). The release (16), RESET (6), every other phase, a neighbouring float, -0, a non-finite word or a missing phase fact, and a name of the wrong type, shape or with a tail, are malformed answers by the existing `entity_scan.answer` marker.
- Nothing else is loosened: `offer`, `done_none`, `done_hit` and `cleared` still refuse any name; first, next, placed, expanded and refused are untouched; the C checks (`check_roster_reply`, `check_city_reply`) are unchanged and still run after this conversion. No C, host provider, driver, roster, CITY or fixed math, campaign policy, scenario, baseline or tolerance changed.

## Evidence (x86_64 Linux debug, strict direct runner, records in `direct-records/`)

- **RED** (`removedname-red1`, exit 101): four new tests over the unchanged 215a bridge; 2 of 70 fail. The positive retaining removal fails with the exact text of the native fatal; the wrong-relation test fails on the same text. The plain-removal and outside-domain tests pass on the unchanged bridge (it refused every name): they pin the unchanged domain and are not RED.
- **GREEN** (`removedname-green1`, `removedname-final-test`, exit 0): 70 passed (66 existing lib tests plus the four new ones).
- **Mutation** (`removedname-mutant1`, exit 101): the validator made to accept everything; the two negative tests fail (68 passed, 2 failed). The source was restored byte for byte (sha256 compared) before any final map.
- **Lints** (`removedname-final-fmt`, `-final-clippy`, `-final-clippyfx`, all exit 0): `cargo fmt -p eech-dc --check`, clippy `--all-targets -- -D warnings` for the ordinary build and with `--features weapon-fixture`.
- **Maps**: FINAL-PRE-CHECK captured on the frozen, formatted bytes, then the final checks, then FINAL-POST-CHECK: 3954 files, every entry equal, the one change from the parent is `module.rs` (plus the three known checkout-filter files). The parent baseline map is not used as a pre-check map. Details and the full-map SHA-256s: `maps/final-map-summary.json`; the 2 MB maps stay at `R:/d3-evidence/city-box-scan-reply-20261106/`.

## Not claimed

No Windows build, native run, current CI, independent review, SourceClearance or actual CITY authority for this source. Every 215a profile build embeds the old bridge. No local harness composes this bridge with the real C city check for a real city; the C grammar stays pinned by the existing `entity_dialogue` check, not rerun. Only `eech-dc` (ordinary and `weapon-fixture`) was linted and tested; the other crates and the Original-like profile were not rerun for a one-file change. The card guard reported no named open blocker for this finding before work; this change adds one.

## Next

The SAME independent reviewer over only this change, then Root validates the new exact source (a fresh snapshot and profile builds, the control first). The 215a control and short roots are CONSUMED; the healthy, missing and changed roots remain UNUSED and may join the new source only by identical input, tool and reference identity and full manifest revalidation. `REQUIRED_CITY_BOX_LIFECYCLE_RETENTION`, all ten requirements and whole D3 remain OPEN.
