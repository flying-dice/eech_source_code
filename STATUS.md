# D2 — EEDCS-44 physical aiming and readiness

Current task: EEDCS-44, In Progress. Branch `codex/d2-weapon-aiming` was created from
verified accepted integration `3ce17432e4026f85b19ebf06cc582314817ddac2`.
Characterization comes first: no aiming responsibility has moved and no aiming
Tacview comparison is claimed. Draft PR103 contains characterization harness
b7f907894; its fresh clean-build witnesses are running. D2 is the only active
delivery epic; D3 has not started.

Authority: live M5 goal11534337v3 and migration ledger14548994v34. D2 covers physical
sensing/visibility, physical aiming/weapon execution, and combat resolution/reports.
Campaign target and weapon choice, priorities, engagement, awareness and consequences
remain in EECH_DC. Historical TypeScript and old call-map inventory are not scope.

## Accepted and frozen predecessors

- D1 is accepted at a7934aa7; all15 tasks Done,40/40/full-byte evidence reused.
- EEDCS-42 is Done: PR101 merged normally at3ce17432e after all9 CI passes at8bd28294b.
  All40 checks passed; all8 complete artifacts and all3 post-run input manifests match.
  Original runner exit1 was only a final cross-drive report display; unchanged report-only
  recovery exited0, independently checked, originals preserved. Ledgerv34 keeps all5 reports.
- EEDCS-43 is not accepted. Reviewed runtimea2a041fec has resolved source findings,
  passing characterization/causal/refusal/fixture controls and unchanged900s records.
  Root's fresh ordinary build passed, all26 hashes match, and full pack89376 is RUNNING
  at `R:/d2-evidence/candidate-pack-sensing-a2a041fec`. Python18836; native9648/22368/23396/8024.
  Snapshot/bin/roots are immutable. Its full40-check/full-byte/CI/merge gates remain.
- Reviewed sensing branche7e46dcc2 is composed before the first aiming build. This is
  development stacking of runtimea2a041fec, not accepted43 delivery. Delivery43 precedes44;
  any later source correction must be incorporated and its affected evidence renewed.

## Current bounded work

Characterize active physical aiming, intercept/vector production, readiness/device
orientation and person target-facing paths through the real host Lua -> native module.
Current source anchors: weapon.c around700/755/943/1102/1257/2213/2635, ac_wpn.c,
vh_wpn.c, ps_anim.c around311, en_maths.c target-point geometry, and all6 family updates.
Those are starting observations, not a predetermined ABI or proof of runtime coverage.
Observe existing computed values without changing calls, RNG/FPU order or campaign policy.
No physical cut before a passing named characterization and observational control.

One Sonnet5.5 High CLI `-p` implementation lane, one independent Codex reviewer.
No nested agents or review fleet. Root owns Jira/Confluence, final builds/full packs,
publication and normal merges. The user's logged-in identity direction authorizes
fd-starscream-bot publication/normal merging for this work; do not reopen that question,
change authentication, copy credentials, fabricate GitHub self-approval or use admin merge.

Use the unchanged Docker recipe in isolated WSL2 EECH-D2-Build/local Unix-socket daemon.
Current toolchain is recorded per build: Rust/cargo1.99.1, MinGW12-posix, Lua5.1.5.
Keep R: (workspace subst) live while native runs exist; native arguments must avoid spaces.
The final pack launcher explicitly establishes a native R: cwd for report formatting.
Private retail inputs/binaries stay outside Git. Preserve actual source/bin/script/input
identities and exits. No accepted baseline or tolerance changes and no D1 reruns for handover.

Next: characterize, then move the source-derived physical responsibility, run the same
scenario and compare complete Tacview/observations/metrics, investigate differences,
independent review, normal merge, and delivery records. Terrain/model lookup, entity
mirrors, shared RNG, effects and sounds remain honest transitional coupling; authoritative
aiming/ballistic calculations must not be left in core behind an echo callback. Remaining
weapon execution and combat resolution are still D2; do not start D3.

Exact live processes/checkpoints: `../d2-evidence/ACTIVE-RUNS.json`.
