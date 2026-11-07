# Sector high-water height: the Host-owned production move, source and Linux checks

Task EEDCS-49 (D3), 7 November 2026 (UTC). **UNREVIEWED; native UNRUN; the move is built in source, not accepted.** Source before: `93495d1b6ddc519f8ec4fee9c6b1147d2c292f39` (the published checkpoint of the accepted finite baseline,
packet `sector-height-public-baseline-reviewed`, pushed BEFORE any production edit). This is the REQUIRED production move of the per-sector `tallest_structure_height` scalar, not a preparation batch (the count stays 2). Design and the
corrections the source taught: [`docs/d3-sector-height-production-contract.md`](../../../docs/d3-sector-height-production-contract.md). No Windows, native, campaign or war launch happened here.

Inputs read in full before the first write and again before the freeze: the SAME reviewer's sealed report `3665b264...` (copied literally), Root's production-start disposition (`Root-sector-height-production-start-disposition-20261107.json`,
in the reviewed packet) and the live goal/ledger as Root cited them. Root's optional feedback file `Root-sector-height-production-feedback-20261107.json` (copied literally) **did not exist at my preflight listing and at the checkpoint guard run; it was
written at 06:40Z while I worked and I read it at 07:21Z, before the final PRE map and every final check** (a timing miss I record, as before): it is a publication readback and asks for no scope, test or recapture.

## What the move is

| | before | now (hosted builds) |
|---|---|---|
| the word | `sector.tallest_structure_height`, Core's, raised in `sc_msgs.c` | one word per sector in the HOST's roster (`roster.rs`, `heights`), absent = the zero word, cleared by a boot's RESET |
| the raise | LINK_CHILD, fixed sender with a valid shape: `y + box.ymax`, strict `>` | the first FIXED_STATE or CITY_STATE statement under a sector placement's KEY after its insertion (`model_host.rs`); the box top is the host's own scene table; one C kernel (`model_step.c`, expression `csrc/sector_height_raise.h`) under the model kernels' environment guard |
| every read | the field, through the getter | the getter (`sc_float.c`) asks the host by the sector's coordinates (new scan phase 17): the scan FIRST read, the original-solver body, the fixed-wing clearance (their text unchanged) and the pack |
| the save and the restore | raw 32 bits of the field | the pack writes the host's word (the getter's, raw); the unpack states each saved word to the host (phase 18), checks the acknowledgement before the load goes on |
| the legacy field | written and read | never written or read in a hosted build (census guard); kept only in the build that keeps the original entity solver |
| candidate fact 12 | the campaign's word | the host's word passed on; the host compares its BITS with its own word for the candidate's sector and refuses a difference; both candidate classifiers exempt fact 12 from the finite test (any raw word is valid history) |
| a city | no shape word | CITY_STATE states the shape (fact 9); a city raises only with a valid one (the accepted baseline's 14395 cities were all invalid) |

Wire: scan phases `17` (`0x41880000`, read: sector in the cursor facts, reply `height`, three bit patterns `x`, `z`, word) and `18` (`0x41900000`, restore: the word RAW in fact 0, reply `restored`); numbers are never accepted for the word in the bridge. Every refusal is named and changes nothing.

## Controls (final set t1 to t16, all exit 0, between the PRE and POST maps)

| check | result |
|---|---|
| t1 `cargo fmt --check`; t2 strict clippy (workspace, all targets); t3 strict clippy of the engine-sys features; t4 strict clippy of the funnel feature | 0 |
| t5 workspace tests x86-64 | 1482 passed, 0 failed |
| t11 the same with `EECH_C_OPT=2` | 1482 passed, 0 failed |
| t6 real-engine control, HOSTED profile (the host owns the word) | 150 expectations |
| t7 real-engine control, ORIGINAL-SOLVER profile (`collision-fixture`): the accepted 179 UNCHANGED plus the differential of the host's raise expression over 21 domain cases under both rounding modes | 221 expectations (179 + 42) |
| t8 observer control in the original-solver profile; t10 observer chain, funnel and cap | pass |
| t9 the campaign's half of the dialogue (raw words, wrong state/sector/word/extra words, refusal) | pass |
| t12 to t14: the same three on i686 | 150, 221, pass |
| t15 workspace tests i686; t16 the same with `EECH_ORIGINAL_STACK_ATTRIBUTES=1` | 1461 passed, 0 failed each |

New or changed controls: `tests/sector_height_host.rs` (the real provider over the real roster, store and kernel: strict greater, no lowering, zero history, restored negative/NaN/infinite words, reset, arrival by key, a city and a building through its sector placement,
refusals, fact 12 by bits), the kernel tests in `tests/model_target_point.rs` (toward-zero sum whatever the caller's rounding, state handed back, unmasked exception refused, every held word), the roster unit tests, one converter test per layer (`eech-engine`, `eech-dc` bridge), the driver tests with the
real roster (`entity_authority.rs`, with a minimal installation root), and the census guards in `eech-engine-sys/tests` (intentionally updated: the pack and getter statements, the readers, the city state words, plus a new guard that no hosted-build code touches the legacy field).
Changed ENTRY POINTS (not scenarios): the accepted 179-expectation control and the observer control now run with `--features sector-height-check,collision-fixture`, where the original body is compiled.

## RED, recoveries and retained failures (all in [`runs/`](runs/), none relabelled)

`c5-world-tests` is the RED of the fixture changes: the whole eech-world suite over the first Host source failed 32 tests (every fact-12 value the old tests carried was not the host's word; the driver tests' real provider had no installation; the city state's new shape fact). `c6`..`c8`, `c11`, `c12`
are the recovery steps; `c14`/`c15` the guards pinning the old census (the intentional change); `c17`/`c18` the observer's width pin and its 179 count; `f1`, `f3`, `f4` formatting and strict lint of my own new tests. The freeze file lists each exit and why.

## Equivalence of the inputs

PRE `2026-11-07T07:21:37Z`, POST `2026-11-07T08:05:13Z` (recorded by `date -u` immediately before each capture, not reconstructed): 3970 rows each, every row equal, tool identity equal; seven inherited raw-versus-Git-filtered identity disagreements stay qualified, never normalised.
No source file was written between them; this packet's files are not inputs of any check.

## Limits and open

Source and Linux checks only. **`sector-height-host-move-open-facts` stays OPEN.** Named OPEN blockers: `ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1` (the guarded fixed-wing clearance is a static function of `fw_move.c` no control reaches: its text and hand-off are pinned and its read is the host's through the getter)
and `ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1` (the real pack and unpack run in the real-C control over a scripted provider in both profiles, x86-64 and i686; no native server-session save and restore). The host does not know the world map (a sector it never saw answers the zero word; the campaign asks only for sectors it holds);
no signalling-NaN, engine `-O2` or exhaustive claim; the test-only sector-height observer instruments the original word and observes no host word. Next: ONE SAME combined correctness/clean-code/boundary-safety review, then Root's native gates and the two controlled witnesses.
Author only: no self-review, approval, native clearance or whole Task49/D3 claim. All ten D3 requirements and the remaining hints, whole UPDATED/shared terrain, raw TREE/FOREST/CLOUD RNG, hierarchy/services, DCS API workarounds, natural moving, the final current 40/all-8/final-4-inputs, CI, review, normal merge and final actual census remain. No D4/D5/TypeScript.
