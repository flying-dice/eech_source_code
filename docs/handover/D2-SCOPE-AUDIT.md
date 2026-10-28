# D2 scope reconciliation — provisional, not acceptance

28 October 2026. Authority: live [M5 goal](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/11534337)
v3 and [migration ledger](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/14548994)
v48. D2/EEDCS-18 is the only active epic. Final regression, cost, CI, review and
delivery gates remain open. This matrix accounts for the finite scope; it does
not redefine that scope around the paths observed in Lebanon.

| D2 responsibility | Boundary and evidence | Current disposition |
|---|---|---|
| Physical visibility observations | EEDCS-42/PR101, merge3ce17432e: host eye lift, ray arithmetic and stepping; original/candidate traces and causal visibility witnesses | Delivered; target choice and engagement remain core; terrain/object lookups remain declared services |
| Physical sensing/detection facts | EEDCS-43/PR102, merge04bf645a: range, altitude envelope, stealth and emitter/jammer calculations through the host | Delivered; campaign filtering, ranking and awareness remain core |
| Physical aiming and readiness geometry | EEDCS-44/PR103, merge83539982: intercept/lead/ballistics, device vectors, offsets and device stepping; actual host authority plus focused optional-flight-time contracts | Delivered within supported campaign runtime; selection/fire decisions and cockpit inputs remain core; legacy sight disposition below |
| Physical weapon execution | EEDCS-45/PR104, merge3b91ca347: launches, flight, guided interception, animation/decoys and physical lifecycle; reviewed public-host witnesses and final40/full8/post3 | Delivered; accepted Task45 evidence is reused, including its explicit uncovered cases |
| Physical combat resolution | EEDCS-46/PR105 candidate: forecast and real hit capability/health/fatal decisions, critical and timed/self-hit rules, impact/blast traversal/radius/attenuation/rocket interception, bridge collapse/fall decisions | Source and bounded runtime groups independently closed; final combined shipping/delivery still pending |
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

Before D2 acceptance: finish the actual unchanged shipping40-check run, compare
all eight complete artifacts byte for byte and all three post-input manifests;
complete the raw9000-frame native cost probe, recompute its statistics and compare
all three complete control artifacts; resolve current CI (including actual
Windows test portability), obtain final independent review, normal-merge PR105
and verify the merge tree. Only then close Task46 and reconcile/accept the whole
epic in Jira and Confluence, preserving all five ledger reports. No D3 starts.

The independent source/publication matrix at09253829b is PROVISIONAL; runtime
build identity remains64cf. Later test-only portability and metadata commits
must record their exact production-input equivalence rather than relabeling that
build or claiming coarse source-tree equality where tests changed.
