# D2 scope and implementation delivery record

29 October 2026. Authority: live [M5 goal](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337)
v3 and the [migration ledger](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/14548994),
reconciled at v49 before its final acceptance append. PR105 delivered the
physical D2 increment at a44e2b3483aab5f5b51aad47dd67e3ece5874da9. The fetched
merge tree equals reviewed head 6220bd5d9. This finite matrix preserves the
whole declared scope and explicit coverage limits. D3 has not started.

| D2 responsibility | Boundary and evidence | Current disposition |
|---|---|---|
| Physical visibility observations | EEDCS-42/PR101, merge3ce17432e: host eye lift, ray arithmetic and stepping; original/candidate traces and causal visibility witnesses | Delivered; target choice and engagement remain core; terrain/object lookups remain declared services |
| Physical sensing/detection facts | EEDCS-43/PR102, merge04bf645a: range, altitude envelope, stealth and emitter/jammer calculations through the host | Delivered; campaign filtering, ranking and awareness remain core |
| Physical aiming and readiness geometry | EEDCS-44/PR103, merge83539982: intercept/lead/ballistics, device vectors, offsets and device stepping; actual host authority plus focused optional-flight-time contracts | Delivered within supported campaign runtime; selection/fire decisions and cockpit inputs remain core; legacy sight disposition below |
| Physical weapon execution | EEDCS-45/PR104, merge3b91ca347: launches, flight, guided interception, animation/decoys and physical lifecycle; reviewed public-host witnesses and final40/full8/post3 | Delivered; accepted Task45 evidence is reused, including its explicit uncovered cases |
| Physical combat resolution | EEDCS-46/PR105: forecast and real hit capability/health/fatal decisions, critical and timed/self-hit rules, impact/blast traversal/radius/attenuation/rocket interception, bridge collapse/fall decisions | Source and bounded runtime groups independently closed; Delivered by normal PR105 merge a44e2b348; combined final review closed, nine CI checks pass, shipping 40 checks/eight complete artifacts/post-input integrity/build identity pass |
| Persistent physical subject identities and factual reports | Shipping64cf: six structure kinds, existing mobile roster IDs, incarnation retirement/collision guards, subject identities across C/Rust/Lua, real report application | Six public native report/withhold/refusal cases closed; stable63136-object census, actual site state and dependent segment death; no all-kind mutation or arbitrary-delay claim |
| Campaign decisions and consequences | Target/weapon/package choice, priorities, engagement, awareness, credit, loss accounting, route/repair-task policy and D1 transactions | Remain core; report application uses real existing handlers and does not claim campaign credit. Reuse accepted D1 causal evidence with its timing/final-member/save-restore limits |

Accepted42–45 merge ancestry and final composition were checked by the same
independent reviewer. Exact Task46 component evidence and failed attempts are in
`eech-campaign/reference/d2-combat-characterization/`, including
`r3-source-reviews/` and `r3-corrected-runtime/`. API declarations and default
recording equality alone are not the authority proof; the causal host/report
witnesses establish the characterized boundary.

## Legacy EECH cockpit ballistic sight: explicit supported-runtime boundary

Task44's at-the-time coupling paragraph names `modules/maths/projtile.c` and the
`get_angle_of_projection*` helper. The remaining compiled caller is
`co_wpn.c:get_weapon_drop` (893–931, helper call924). Its consumers are the
Havoc/Hokum/Ka50 HUD drawing paths (`ha_hud.c:2501`, `hk_hud.c:1925`,
`hm_hud.c:1835`). They position a ballistic pipper and can also slew the EO sight.
This is real legacy avionics arithmetic/coupling, not merely general maths or
proof that a player path ran in the headless witnesses.

The technical-lead disposition is to keep that dormant EECH cockpit/HUD path
outside the supported campaign-DLL runtime. The native public frame explicitly
runs without drawing (`eech_engine.c:306–336`); there is no campaign Lua export or
product caller for `get_weapon_drop`, nor an EECH cockpit/HUD rendering operation
in the host API. The old AI helper calls in `weapon.c:707–865` are disabled by
`#if 0`; active campaign ballistic/intercept and tracking calculations use the
host boundary below898. This disposition follows M5's restriction to work needed
for DCS integration, not lack of headless-player coverage. It does not authorize
an active campaign physical solver to remain core. D5 may remove dormant legacy
code; it is not being assigned a missing D2 physical extraction. Supporting an
EECH cockpit rendering loop would require an explicit new scope decision.

## Remaining coupling and acceptance gates

Terrain, environment, model/geometry services remain D3; airfields and physical
logistics/repair quantities D4; final integration and dormant/world-code removal
D5; live DCS operation M6. Entity mirrors/heap, shared random stream, effects and
sounds are declared transitional coupling. Those labels do not defer any active
D2 physical solver. Reference-host Lua callbacks reuse original C arithmetic;
this is neither a second campaign implementation nor a claim of a full DCS host.

The six focused report probes produce no Tacview. They use controlled reported
facts, and the own-ID case overrides identify only. Natural, controlled, altered
provider and bridge evidence retain their exact recorded family/player/MP/RNG
coverage limits; none is promoted into a universal coverage claim.

## Final native and delivery evidence

The ordinary shipping pack completed on 28 October 2026 at 23:53:56 UTC,
with the original controller and runner exiting 0. Wrapper post-checks completed
at 23:54:10 UTC. The exact accepted set of 40 checks passed, all eight complete
artifacts equal the completed destination baseline, and all three external
post-input checks passed. The additional internal Lebanon copy matched its
1179-file manifest. All 42 shipping file-manifest entries and BUILD-INFO SHA-256
3410ac4174a259e0ee211e4bb05361a819e1c1da54a300abe93968eff08e169d are unchanged.
These are full-file comparisons; visual Tacview inspection is not claimed.
Exact metadata is in:
eech-campaign/reference/d2-combat-characterization/final-shipping-fullpack/.

The separate final 9000-frame, 15-window cost control is independently closed
(runtime review a3a2ee76, publication review 0109735d). Its three complete
artifacts, 1179 post-input files and shipping manifest match. Raw timing,
statistics and loaded elapsed-time/DCS-budget limits remain in
final-shipping-frame-cost/. At daf3960d8 all nine CI checks passed, including
Windows host_callbacks portability confirmation. The implementation merge tree exactly matches that reviewed and checked
candidate after the final publication at 6220bd5d9.

The finite matrix accounts for all D2 responsibilities. No active D2 physical
solver is assigned to D5. Prior source/runtime reviews, accepted Task42–45
ancestry and D1 consequences are reused within their exact limits. One combined
independent final completed-pack/publication/composition review at 6220bd5d9
found no blocking findings, SHA-256
d5a44c3dd104923a323dc6d0f69b904356c3bf3e96b764f6990af4aed247c000.
All nine checks passed at that exact head. Default recording equality alone is
not the authority proof; the controlled host/report witnesses remain necessary.

Normal PR105 merge a44e2b3483aab5f5b51aad47dd67e3ece5874da9 was fetched and
verified: parents 3b91ca347 and 6220bd5d9, with the exact reviewed tree. Safe
review, CI, merge-tree and cache-cleanup records are in
eech-campaign/reference/d2-combat-characterization/final-delivery/.
The completed named cache was removed only after all exported shipping files
and zero container references were verified. Useful raw and failed evidence,
source snapshots and shipping binaries remain preserved.

Authoritative task/whole-epic acceptance is recorded in
[EEDCS-46](https://flying-dice.atlassian.net/browse/EEDCS-46),
[EEDCS-18](https://flying-dice.atlassian.net/browse/EEDCS-18) and the migration
ledger, preserving all five epic reports. This document records the delivered
implementation and its evidence; it does not start D3.

The independent source/publication matrix at 09253829b was provisional.
The actual runtime build remains 64cf0f1c5551a2e59309a92917c32e6189d8ae60.
Seven coarse frontiers match the later candidate; crates differs only in the
integration test host_callbacks.rs. Production compilation inputs are unchanged.
Later test-only and metadata commits neither relabel the build nor establish
equality of all eight coarse source trees.
