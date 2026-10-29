# D3 terrain and environmental services — active

29 October2026. Authority is the live M5 goal11534337v3 and migration ledger14548994v55.
D1/EEDCS-15 and D2/EEDCS-18 are accepted. D3/EEDCS-21 is In Progress; D4/D5 have not started.
Campaign geography, planning, timers, scheduling and consequences stay in EECH_DC.
The finite remaining scope is recorded in docs/handover/D3-SCOPE-AUDIT.md. Old status
entries remain in Git history; HOST-HANDOVER.md is the historical D1 handover.

## Current extraction

EEDCS-47, PR107, branch codex/d3-terrain-services, verified integration base
6bbaa8197a4843181d03cd355d562af998c75321. Point, fast-elevation and map-fact queries
now dispatch to the host. The DLL retains semantic value records and optional opaque
hints; the numerical point/fast solver is in the reference host. DLL2D loading/grid
generation, derived terrain geometry, spatial/model services, environment and host-time
coverage remain whole-D3 work. Task47 and the epic are not delivered yet.

## Verified evidence

- Original controls include both maps, deterministic20k probes, repeated600s runs and
  a900s control whose complete Tacview/observations/metrics match the accepted D2 donor.
- Both complete generated grids equal independent original-generated files byte for
  byte; point/primed/fast conformance passes on both generated and provided grids.
- Public reference placement/LOS match original; raised terrain changes both. Missing
  and all six malformed replies refuse with the specific expected markers.
- Public moved-client probes match all256 emitted physical point/fast samples and
  complete20006-fast-call hashes on both maps. Primed/run diagnostic folds are qualified.
- Six real face-predicate cases pass for reference/nohint providers and native season,
  Summer assertion and Winter fallback. Retail has no winter pair; distinct seasonal
  file selection is tested separately. Shipping nohint preserves original observations.
- Frozen runtime06d42557843aba5728ffafae937451413115fb6c: ordinary Windows build0,
  all45 manifest entries verified, backend17/17, refusal-driver7/7, face-driver5/5,
  actual DLL boot-contract1/1 and both final whole-grid comparisons pass0. Prior
  engine9/world18 terrain checks and independently reviewed a9 public cases are reused.
- The analytic all-land Lebanon case remains FAILED1: changed WATER geography removes
  the required USS TARAWA anchorage. Its failure is preserved, never swallowed as a
  provider refusal. No baseline, input manifest, oracle or tolerance was changed.

Safe evidence and independent reports: eech-campaign/reference/d3-terrain-characterization/.
Raw retail assets, coordinates, binaries and recordings remain outside Git. Accepted D2
runtime64cf0f1c5551a2e59309a92917c32e6189d8ae60 and its40/eight-artifact proof are reused.

## Final regression and next gate

The unchanged40-check shipping pack is RUNNING from clean frozen runtime06, seed1,
100ms frames and3simhours per scenario, using three fresh manifest-verified roots.
Controller tool55606; runner PID23548 created2026-10-29T13:03:32.911797Z. Four native
processes21112/21948/6588/9768 are live on distinct roots; verify creation/argv before
acting. Both Tacview and observation LIVE PREFIXES match accepted bytes; this is
partial evidence only. Complete all8 files, actual runner exit and post-input checks
remain required. Do not restart from a quiet console or observation timeout.

Source06 has an independent delta review with no findings. Its hosted runtime tests
pass, but workspace lint exposed additional tuple/test-style issues. Those corrections
are prepared without numerical/ABI/contract changes. Actual affected Linux engine9/9
and ALL module13/13 tests pass0. Actual FULL-workspace strict Linux Clippy passes0; new-delta review/currentCI remain pending;
the frozen regression continues unchanged. No full-pack or task/epic acceptance claimed.

Use the eech-regression skill and existing repo runner/checkers. Same-host process,
source/bin/root/output paths and next actions: R:/d3-evidence/D3-NEXT-ACTION.json.
After final comparison, applicableCI and independent new-delta/evidence review, perform
a normal Task47 merge and update Jira/Confluence, preserving all five epic reports.
Continue all remaining D3 services and audit the whole scope before closing the epic.
