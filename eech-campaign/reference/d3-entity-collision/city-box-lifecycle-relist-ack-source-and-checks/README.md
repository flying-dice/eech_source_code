# D3 Task49: a relisted building's new city-list key and the fixed state its record holds (UNREVIEWED CODE_FROZEN)

Code `eb3861ea8a5e2103db7bc8c3d3f97cf1f15b0dc5` over metadata `a1ef80b05612f4b8f2d355ad024cef6037426e55`; sealed runtime code `354abd9c04a53f1935919abc4fde1172b438c2b3`. Three files changed: the driver script `eech-campaign/crates/eech-world/lua/entity-authority.lua`, its scripted test `tests/entity_authority.rs` and the TEST-ONLY C dialogue check `csrc/eech_entity_dialogue_check.c`. No production C or Rust changed. Compact report: `city-box-lifecycle-relist-ack-CODE-FROZEN-20261106.json`.

## Blocker

Root's actual native control over the sealed 354abd9c0 runtime (`Root-city-lifecycle-354ab-relist-ack-failure-20261106.json`, byte copy here) completed the real CITY lifecycle (selected 11193 BEFORE 1 / AFTER 1, retained 2, continuations 2, six expansions answered, query_equal, list_order_restored and mirror_box_equal 1) but exited 1 with `request_faults` 6: first `scan request: a city expansion names the building 137970, whose fixed state the provider never acknowledged`. Completion and tuple equality are not acceptance, and the faults are not suppressed.

## Diagnosis (reproduced, not assumed)

State is **not** missing; the observer's bookkeeping missed a legitimate rebound record.

- The real C relist (`place_child_insert`) lists the building under a NEW key, declares `LINK(sector key, new child key)` (`place_declare`), then states the building (`place_fixed_state`) under its **sector** key because it holds one. No report ever names the new child key. At boot the child is listed first and stated under the child key, which is why only the relist failed. The C dialogue check now pins this emission.
- The host keeps both placements in ONE record with ONE fixed state (`roster.rs` `link` / `merge_fixed`); the scripted fixture asserts in-engine that the real roster holds the state for the new child key (`held`) before the expansion. A building listed again WITHOUT a LINK holds `never` for the new key (probe `relist-probe1`), and the real provider refuses its expansion by name.
- The driver (`request_join_problem`) required `scan.fixed.held[child]`, keyed by the key a report named, so the new child key looked unacknowledged. The earlier scripted relist stated the building under the child key, an order the real C path does not emit.

RED over the unchanged driver with the real provider (`relist-red-final`): `request_faults=2 reply_faults=0`, the same text, lifecycle otherwise complete.

## Change

An acknowledged LINK of a live sector placement and a building listed under a city records the two keys of one record (`scan.fixed.peer`); a state acknowledged under a key marks its peer, and a LINK joins whatever state either key already held (`scan.fixed.shared`). The expansion check asks `scan.fixed.holds(child)`: acknowledged under that key, or shared through an acknowledged LINK. `scan.fixed.held` keeps its meaning and its count. An unlinked key, an unacknowledged LINK, a non-live sector key and a state of another record establish nothing; a missing LINK stays a request fault even when another building's LINK succeeded. Not changed: the row, its counters, the candidate-lookup joins (they still ask the strict `held`), any production C or Rust, and the faults-before-UNREACHED order.

## Evidence (x86_64 Linux debug, strict direct runner, all 17 records in `direct-records/`, failed iterations included)

- **RED** `relist-red-final` (exit 101, 2 failed) after the first-version `relist-red-driver`; **GREEN** `relist-final-world` (entity_authority 108, entity_candidate_host 13, entity_roster_host 18, city_box_lifecycle 9) and `relist-final-c` (entity_dialogue 1, entity_host_guard 5, entity_roster_guard 20), all exit 0.
- **Fixture fidelity** (found during the work, kept visible in `green-driver1/2`): the unchanged query walks past the buildings' sector entries the real scan serves first, and the scripted expansion refuses a building the real roster holds no state for, as the real provider does.
- **C characterisation + mutation**: the new block passes on unchanged production C; flipping one expectation fails at its line (`relist-c-mutant`), source restored byte for byte.
- **Lints** `cargo fmt -p eech-world --check` and clippy `--locked --all-targets -p eech-world -- -D warnings` exit 0.
- **Maps** (`maps/final-map-summary.json`): FINAL-PRE on the frozen bytes, final checks, FINAL-POST: 3954 files, every entry equal; the three code files are the change from the parent (plus the three known checkout-filter Lua controls); each committed blob equals its mapped raw bytes (`env/bind-committed-blobs-to-map.py`).

## Not claimed

No Windows build, native run, current CI, independent review, SourceClearance or actual CITY authority for this source; whether the native control now reaches zero faults is unproved until Root's fresh run. No per-request trace of the native run was retained, so the six native requests are inferred from the real C source plus the reproduction. Not run locally: Original-like and hosted feature builds, the i686 and `EECH_C_OPT=2` matrices of the C check, eech-dc/eech-engine (unchanged), the full workspace.

## Next

The SAME independent reviewer over only this change. Root needs a NEW fresh control root (the 354ab control root is CONSUMED; short, healthy, missing and changed roots stay UNUSED). If Root confirms no compiled input differs, a separate Git literal can be bound to the sealed 354abd9c0 runtime after the same review, with no new Windows or C rebuild. No negative before a control reaches the selected record before and after with zero faults. All ten requirements, the CITY lifecycle before sector height, remaining physical services, DCS mapping, natural moving integration, final current 40/all8, final 4 inputs, CI, review, normal merge, final census and whole D3 stay OPEN.
