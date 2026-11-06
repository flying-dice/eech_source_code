# D3 Task49: selected CITY removal record identity (UNREVIEWED CODE_FROZEN)

Code `215a8f7389dfc9ab1dac1df2f4e45344392967d9` over `71d92cf856e1340796fb6647a7cd70bc8319907b` (metadata `f21f21c1b`). The same reviewer's `CHANGES_REQUIRED` over `71d92cf85` (one P2 finding, `ENV-CITY-LIFECYCLE-SELECTED-REMOVE-FAULT-1`) is retained literally in this pack (`independent-city-lifecycle-selected-witness-source-71d92-20261106.json`); the 71d92, 4ef and d9bd packs and their reports stay unchanged. Compact report: `city-box-lifecycle-remove-identity-CODE-FROZEN-20261106.json` (byte-identical copy at `R:/d3-evidence/`). Two files changed: `eech-world/lua/entity-authority.lua` (5 lines) and `eech-world/tests/entity_authority.rs` (the one existing test).

## Defect and change

A current retaining REMOVE of the selected record that named ANOTHER syntactically valid record was excluded from the intervention (`real.name == life.selected`) but accepted by the generic `roster_reply_problem`, adopted by `roster_track` as a list binding, and the unintervened negative then reported the clean `UNREACHED` ("never retained"). That is a malformed current positive, not a missing prerequisite.

`roster_reply_problem` now compares a nonempty retained name with the removed placement's canonical record (`scan.city.anchor`, which resolves LINK relinks) when that record is the SELECTED one inside the lifecycle window; a different record is a reply fault. Both call sites therefore skip `roster_track` for it (no list binding adopted), the intervention predicate cannot be reached, nothing is credited, and the existing verdict order (fault before UNREACHED) reports the fault. No new store, API, solver or grammar; the generic removal grammar, live, draining and empty relink paths, the full private ACK checks, the strict report parser and the selected before/after counting are untouched.

A first, too general version (every retaining removal) failed the legacy pin `a_removal_that_names_the_record_the_host_keeps_for_a_citys_box_is_no_fault` (a scripted arbitrary record for a non-selected placement; `identity-green-focused`, exit 101). It was narrowed to the selected record the finding named; the pin is unchanged.

## Evidence (x86_64 Linux, direct runner records in `direct-records/`)

- **RED** (`identity-red-world`, exit 101): the strengthened EXISTING `a_bad_retaining_positive_or_release_acknowledgement_causes_no_intervention_and_no_credit` against the UNCHANGED 71d92 driver fails at `reply_faults >= 1` for `wrong_record_on_remove`: `reply_faults` 0, `first_fault` empty, `intervened` 0, `retained_answers` 0 (clean absence). The loop now also requires `retained_answers`/`releases_in_window` 0, a `scan reply:` first fault naming the fault, and an error that is neither `UNREACHED` nor a host-retention result (for the cursor fault as well).
- **GREEN** on the frozen bytes: `cargo fmt -p eech-world -- --check` 0, `cargo clippy --locked --all-targets -p eech-world -- -D warnings` 0, `cargo test ... -p eech-world --test entity_authority --test city_box_lifecycle` 0 (106 and 9 passed). Earlier attempts are retained: the general version (exit 101), a formatting failure of one new assertion (exit 1, then `cargo fmt -p eech-world`), and the pre-formatting clippy/test passes.
- **Maps** (`maps/`): the before and final after raw maps (3954 files) differ in exactly the two changed files; the three inherited raw-vs-Git test-Lua exceptions are identical and never normalised. The after-map taken before formatting is kept as evidence. Full ~2 MB maps stay external; hashes are in `input-map-summary.json`.

## Native plan

Unchanged from `city-box-lifecycle-witness-correction-source-and-checks/README.md` (actual selected CITY through real Core, standalone after-BOOT/warmup dedicated row, control, missing Host-retention and mirror-conformance negatives, no post-fatal getter, no invented physical fields). One wording consequence: a retaining removal of the selected record naming another record is a reply fault, so `request_faults 0` / `reply_faults 0` still holds for the control and a differing record is a failure, not `UNREACHED`.

## Limits

Lua driver and scripted-harness correction only; the witness C has not executed. No C, Engine, Sys, dc, Windows, native, i686 or COPT2 run (no such source changed over 71d92; the target map binds those trees to the parent). No independent review, current CI, SourceClearance or acceptance exists. `REQUIRED_CITY_BOX_LIFECYCLE_RETENTION`, whole Task49 and whole D3 stay OPEN, with all ten D3 requirements (required CITY before sector height, remaining coupled services, current final40/all8/final inputs, CI, review, normal merge, final census).
