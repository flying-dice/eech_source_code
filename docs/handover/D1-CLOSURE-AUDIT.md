# D1 closure audit — current evidence, not acceptance

23 October 2026. Source6e837440, PR98 draft. Sole scope authority M5 Confluence11534337v3 and migration ledger14548994v21. D1 moves forces, movement, arrivals and physical lifecycle; campaign decisions/consequences stay in DLL. D2 combat/perception, D3 terrain, D4 airfields/logistics and D5 final integration remain later. This audit does not redefine D1 around what happens to pass Lebanon.

| D1 responsibility | Evidence already delivered | Current disposition |
|---|---|---|
| Mobile identities, initial creation/placement, retirement | Roster host IDs; PR97 create_place at faction creation/regen sites, generic free retires ID; accepted40-check pack and causal placement witness | Delivered for declared supported paths; entity heap is a mirror, not removed in D1 |
| Routed, ship, person physical route movement | PR91/92 host kernels and connected call sites | Delivered; ship public movement witness was unavailable (Lebanon1h census had0ship_move calls), so baseline equality alone does not prove ship motion |
| Aircraft integration, rotor, death/crash motion and control | PR93–96 delivered; PR98 connects remaining control/avoidance/terrain shaping | PR98 runtime/census/CI and final approval pending, no completion claim |
| Arrival and proximity reports consumed by campaign | PR89/90 cover all6mobile kinds, action/recon/reached semantics, missing IDs fail; retained causal arrival witnesses | Delivered for recorded cases; player recon public path remains unavailable in headless AI scenario |
| Physical wreck lifetime and removal | PR97 host expire/retire plus PR96 ship sink | Delivered in supported server lifecycle. AA has no legacy expiry decision; dead timer is used for legacy rendering fade, not grounds to invent expiry behavior |
| Movement-dependent force positioning | Current vh_vec3d/ac_vec3d notifications and hc_msgs/rv_msgs/aa_msgs/ps_msgs handlers | Concrete remaining D1 movement gap, below |
| Host-reported loss lifecycle / campaign consequences | observe_physical_loss called by all6families; bounded helicopter report_loss receipt works | General all-family host-report handoff is incomplete, below |

## Remaining area 1 — movement-dependent force positioning

Concrete current source: vehicle position setter vh_vec3d.c:171–184 notifies MOVEMENT_DEPENDENT children before updating parent position; aircraft setter ac_vec3d.c does the same. Faction creation attaches members to a moving keysite parent (faction.c:432–437,731–736), so this is an actual configured path, not merely a leftover helper.

hc_msgs.c:477–520 computes child relative offset, rotated position, heading and velocity in DLL on SET_ENTITY_POSITION. Routed/AA/person handlers copy the supplied position through their setters. The host ship result in rv_hostmv.c:370–372 wraps the parent setter in global applying_host_result; campaign_movement_place:161 then skips the nested children's placements too. Thus carried child pose can change in the DLL without the corresponding host retained pose changing. This is a source-identified risk; no moving-parent runtime reproduction claimed yet.

One next extraction should move supported force-dependent positioning into the host, keyed by parent/child host IDs, preserve original notification order/arithmetic and keep campaign attachment membership in DLL. Characterize the real parent/child interaction before changing it and add one causal host/campaign consistency witness, then established regression/Tacview. Do not make this a general scene graph or move effects/keysite/airfield behavior out of their later epics. Inspect nested apply guard handling in that same extraction.

Not additional D1 movement work merely because text remains: basic_aircraft_movement has only definition/declaration and no production caller in aphavoc/source; helicopter_movement_calculate_new_position has its only call under !EECH_CAMPAIGN_HOST_MOVEMENT. Person target-facing animation is aiming (D2). Human network interpolation belongs to the unverified live-client integration path, not evidence of supported headless AI behavior.

## Remaining area 2 — host-report-driven loss handoff for all six families

One compact read-only Opus Medium plan completed10turns, contextb24cbad1-7c5d-4381-9e05-41e1ed9c2a7c; source findings independently grounded by parent's reads. Full plan is D1-LOSS-PLAN.md. No implementation or new agents.

Current gp_roster.c:446–508 restricts pending loss to nonfinal AI SUPPLY helicopter with exact characterized dependencies; the other5destroy handlers reject WITHHOLD. All6call observe_physical_loss, but FORWARD still applies campaign consequence blocks synchronously inside family kill_local. A callback verdict alone is not proof that an external host-ID report can drive the lifecycle for every family.

Lead disposition: retain this as a real D1 handoff gap; do not relabel it D2 merely to close D1. D2 owns causes of death, damage, CREDIT, targeting/perception. Campaign loss bookkeeping stays campaign-owned; move it behind the host-report entry, not into host physics. Preserve immediate-report timing/order for the accepted default. Bound delayed-report witnesses to characterized cases; do not claim arbitrary delay/final-member/save-restore support from those cases.

One finite task after PR98: make host-ID loss reports reach the campaign's family-specific consequences for all6families, preserving synchronous FORWARD ordering, and establish minimal causal report/withhold evidence for the missing5families. Keep unsupported shapes explicit. Reuse the existing helicopter receipt and accepted ABI evidence; one final established native/Tacview comparison is required even if an advisory plan says Tacview only when event order changes (the user's working loop requires it for every extraction).

## Sequencing and closure

Finish PR98 first. Then take these two bounded areas sequentially, raising one simple task when each starts. Do not pre-create a sprawling backlog or start D2. After their delivery, reconcile this matrix with actual current source/evidence and check remaining writers/report paths before D1 closure. The existence of this plan, all-green narrow tests, and the absence of other findings do not prove D1 complete.

## Evidence update 23 October, 23:33 BST

The matrix above is the earlier source audit, not current delivery status. PR98/EEDCS39 is delivered at ed38c8fb with exact-head independent approval, eight CI passes, native40/40 and unchanged full Tacview/observations. PR99/EEDCS40 is open at6771b002, independently approved; native40/40/full-reference comparison passes, five CI checks pass and three remain live. Its controlled nested carry fixture proves the changed boundary; normal full-pack equality is not a carry census. EEDCS41 is active at6a252b08 with report-owned per-host transactions across all six families. Final causal family coverage, source/binary provenance, native regression and delivery remain unproven. These updates do not close D1 or supersede the final source/report-path reconciliation required above.

## Delivery reconciliation — 24 October

Live migration ledger v23 confirms PR99/EEDCS40 delivered at 2fadceb2, with native40/40, unchanged full Tacview/observations and campaign metrics, plus controlled nested carry evidence. Thus remaining area1 above is delivered, not an additional open extraction. PR98/EEDCS39 is also delivered as recorded previously.

Remaining area2 (EEDCS41) is not accepted: current integrated source checkpoint6438416c has clean shipping/fixture builds and focused checks, but final all-family causal coverage and shipping native/Tacview regression remain pending. Routed/AA/person have successful bounded receipts; ship causal receipt passed but its separate negative-control setup failed; fixed-wing candidates remain unavailable under the production guard. Diagnostic fixture work is in progress in the same author context. These are validation gaps, not justification to remove fixed-wing scope. After41 delivery, reconcile remaining physical writers and report paths against the whole D1 scope before closing EEDCS15. D2 remains unstarted.

## Current reconciliation — 24 October, eefd0bb0 (pack still running)

The preceding fixed-wing-unavailable status is superseded. EEDCS41 now has a final clean runtime source (9b4a668b), a successful fixed-wing report including reservation preservation and single release, and final-member refusal evidence. Retained routed/AA/person/ship receipts and the accepted helicopter evidence are mapped in all-family-loss-20261023/PROVENANCE-AND-SELF-REVIEW-41.md; the parent cumulative review records why unchanged-family evidence is reused and its partial-tree provenance limits. This is validation progress, not delivery: final pack, publication, CI, exact-head approval and merge remain pending.

Re-read live scope: Confluence M5 v3 and migration ledger v23 remain authoritative; Jira15 remains In Progress and describes physical creation/removal, route movement, arrivals and lifecycle reports, with campaign composition, task/route decisions, membership and loss accounting retained. Its W01/W02/W03/W05 labels are historical references absent from the current ledger body, not new scope authority.

Current source anchors checked for the final reconciliation:
- faction.c has four campaign_movement_create_place call sites (469, 778, 1080, 1433); force-creation documentation explicitly retains the entity heap as a mirror and terrain as D3.
- gp_roster.c:1106 calls campaign_movement_retire. Routed, helicopter, fixed-wing and person update paths call campaign_movement_expire; ship sinking has its separate host mode and AA has no legacy expiry to invent.
- Six destroy families are connected to the report transaction; the cumulative EEDCS41 review covers their per-family phase ordering. Loss causes/combat remain D2, and bounded report support must not be described as arbitrary-delay/final-member support.
- Retained surface-movement evidence explicitly lacks a public ship_move witness (zero ship steps in its one-hour census). Its host kernel/dispatch checks and source wiring are evidence of extraction, but unchanged Tacview is not evidence that ship_move executed. This limitation must remain visible in the final D1 review.

The final whole-epic disposition is still pending; these anchors do not claim an exhaustive proof of every physical writer. No new extraction or D2 task has been started.

## Additional source reconciliation — person movement

Inspected current ps_move.c movement branch and rv_hostmv.c person begin/end bridge while the final pack remains live. In the host-movement build, walking/running translation and pose come from PERSON_BEGIN/PERSON_END host results; the old translation/heading arithmetic is under the opposite preprocessor branch. The intervening terrain elevation query remains an explicit D3 dependency. Task-based animation/speed selection is still campaign-side input and must remain visible when judging the eventual physical/campaign boundary; this inspection does not prove every writer has been extracted. No source changed and no new run was started.

## Closure reconciliation in progress — candidate 382c3ab3 / PR100

Current EEDCS15 and41 were reread live after publication. The epic still requires physical creation/removal, route execution, arrival observations and lifecycle reporting through host calls, with campaign composition, task/route decisions, membership and loss accounting retained. PR100 has exact-head independent approval and final native40/40/full-reference equality, but hosted CI and merge remain outstanding; EEDCS41 is not delivered yet.

Additional source inspection: sh_move.c:297-301 dispatches live ship movement to campaign_movement_ship_move under EECH_CAMPAIGN_HOST_MOVEMENT. The old turn-rate/acceleration/pose arithmetic is in the opposing preprocessor branch. Death movement similarly dispatches ship_sink at508 and ship_sunk at586; the old speed writer at526 belongs to the legacy branch. The earlier lack of a public ship_move witness remains a coverage limitation, not evidence that the physical solver still executes in the DLL.

Person movement: current ps_move.c selects requested walk/run speed from campaign task type, then PERSON_BEGIN/PERSON_END owns physical translation and returned pose; terrain is the declared D3 input. ps_anim.c's remaining attitude write at332 faces a target while firing, which belongs to D2 combat/perception rather than route motion. This classification follows responsibility, not simply the presence of a pose writer. These additional reads do not claim the whole D1 audit finished; remaining creation/report and supported-family call-path reconciliation still needs a final consolidated disposition after PR100 delivery.

## Creation, identity retirement and arrival dispatch reconciliation

At382c3ab3, gp_msgs.c:163 binds members through campaign_roster_link_member. The roster switch explicitly maps all six D1 families; bind_mobile is required, returns the host identity, and invalid/reused identities fail. The four faction.c creation-placement calls remain present. This supports host physical identity/placement ownership while campaign formation/composition and the entity heap mirror stay declared inputs/coupling.

The generic en_heap.c:497 release hook reaches campaign_roster_release_entity; for a physically present roster incarnation this calls campaign_movement_retire before clearing the heap mapping. This is the common identity-removal path rather than a per-family optional cleanup. Physical expiry decisions remain in the previously reviewed host expire/sink kernels; campaign consequences and membership stay in the DLL.

Arrival dispatch: guide.c:check_guide_reached_waypoint returns FALSE before member evaluation if there is no task leader. With a leader, check_mobile_reached_waypoint dispatches SUPPLY or the all-six-family campaign_radius_arrival_check; the legacy notify fallback is not selected for those family/leader combinations. The host returns physical gates; campaign selects criteria and combines task/member results. Player recon remains a documented headless runtime coverage limitation. These source checks strengthen the D1 closure matrix; they do not override the outstanding PR100 delivery gate or prove arbitrary external report timing.

## PR100 delivery verified — 24 October
PR100 merged normally at a7934aa7d1bcffc2ea5057bb3f1b0af332606c61 after all9 CI SUCCESS and independent exact382c3ab3 approval. EEDCS41 Done; ledger14548994v24 verified with5charts intact. Earlier pending-delivery statements are superseded. Remaining work is the consolidated whole-D1 completion audit against current source and retained evidence, including explicit ship/player-recon runtime coverage limitations and bounded loss support. No D2 work started.

## Post-merge lifecycle audit — a7934aa7 / 24 October
Verified fetch and empty tree diff from reviewed382c3ab3 to mergea7934aa7. origin/HEAD is the historical default branch, not the agreed integration branch; the first comparison against origin/HEAD was irrelevant and is not acceptance evidence. Live Jira query returned all15 D1 children Done with isLast=true.

Traced two previously unverified terminal sequences in the current hosted branches:
- Helicopter ground crash: hc_move.c calls crash_begin then crash_end before returning a stopped result; movement.rs CrashEnd clears candidate and crash_pending before the caller can mark DEAD. Helicopter burial marks DEAD but still executes campaign_movement_tumble before returning; no new runtime claim.
- Fixed-wing crash: fw_move.c calls candidate, velocity and attitude in order; FwCrashAttitude clears candidate/fw_crash before returning at_rest, which precedes DEAD. Burial continues to campaign_movement_fw_death_attitude at2764 before returning; successful host attitude processing commits/clears candidate, contact_altitude and fw_speed. Thus the inspected DEAD paths do not leave those crash sequences open for next-frame expiry.

This resolves the specific crash-order uncertainty by source inspection, not by inventing runtime coverage. Person movement/expiry and live-helicopter-step terminal flow still need concise reconciliation before the consolidated acceptance record. Historical documents contain pre-delivery status paragraphs; final acceptance must distinguish those snapshots from current delivered state. No code changed, no new build/run/agent started.

## Final acceptance — 24 October 2026
D1 accepted at a7934aa7. Authoritative durable acceptance is Confluence14548994v25 and JiraEEDCS15 comment10714; epic transition returned Done. All5report macros verified preserved. All15 child tasks delivered.
Final remaining source checks: ps_move.c355-364 always calls person_end on a pending begin, with no intervening normal return; rv_hostmv.c1736-1783 enforces matching pending flags; movement.rs end_person takes/clears pending state. hc_move.c807-809 calls guided_begin then hosted_guided_end with no intervening return, helper424-504 calls the host end unconditionally, and movement.rs2113 clears helicopter_pending. Earlier rotor/landed/wait returns precede begin; abandon clears retained guidance. This closes the documented pending-step/expiry concern by source inspection.
Acceptance covers the full declared D1 increment (creation/removal, movement/route execution, arrivals, lifecycle reports; campaign policy/consequences retained), with evidence mapped in the ledger. Known coverage limits are recorded there without promoting them to runtime passes. D2-D5 and M6 remain separate, uncompleted scope. No code changes/new runtime experiments were necessary for this acceptance audit. Goal complete.
