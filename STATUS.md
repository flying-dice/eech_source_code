# D3 ground-contact characterization — sample-checker checkpoint

29 October 2026. EEDCS-48 is In Progress under EEDCS-21. Branch
codex/d3-ground-contact starts from verified accepted integration
6bbaa8197a4843181d03cd355d562af998c75321. Authority is the live M5 goal
11534337 v3 and migration ledger 14548994 v55. D3 remains incomplete;
D4/D5 have not started.

This checkout contains ORIGINAL characterization only for point_above_ground,
point_below_ground and get_ground_impact_point. The ground-fixture hooks and
public Lua/module driver exercise the existing implementation. Ordinary shipping
behavior and physical expressions are unchanged. Task48 has moved no physical
responsibility. Read the characterization notes in
eech-campaign/docs/m5-ground-characterization.md.

The original test-only fixture compiled successfully from clean snapshot
a14cbdda1d59fbe8a88bb7437e21a98f82052c04 at 14:16:45 UTC. This does not validate
the subsequent sample-checker correction. No original retail ground probe has run,
no original oracle exists, and no shipping or Task48 acceptance is claimed.

The first independent review found two P2 fixture issues and one P3 comparison
issue. The next delta review closed warm-record contamination and qualified the
legacy diagnostic comparison, but found incomplete/invalid samples still accepted.
The latest sole Sonnet 5.5 High correction requires distinct ordered case IDs,
valid tag/type domains, per-class caps, exact impact sample counts from reported
tallies, and consistency with counters. It adds 22 negative cases (61 total) and
positive cap controls. The original C fixture is unchanged. Rust formatting and
check, both changed Lua syntax parses, and diffcheck pass. The new driver/tests
remain uncompiled/unrun and require independent delta review. Earlier reports and
current static/build results are retained in the reference folder.

Task47 surface services are separately implemented/reviewed on
R:/d2-sensor-detections, pushed head f03a680c9, with all nine hosted CI checks green.
Its final ordinary shipping regression remains RUNNING from immutable runtime
06d425578 on four separate roots. Previously checked live Tacview/observation
prefixes match accepted bytes; complete 40-check/eight-file/input acceptance and
normal delivery remain pending. Do not restart or modify those processes or their
frozen source, binaries and roots.

Next: independently review the new sample-checker delta, build a clean corrected
snapshot, execute actual driver guards, then run original retail controls and
repeats. No production cutover before Task47 delivery and integration verification.
Campaign geography, routes, timers, scheduling and consequences remain in EECH_DC.
The remaining D3 spatial/model/environment/time services and whole-D3 closure stay
in scope. Same-host continuity: R:/d3-evidence/D3-NEXT-ACTION.json.

D1/D2 acceptance remains in the migration ledger and accepted repository evidence.
HOST-HANDOVER.md is the historical D1 handover. Private retail inputs, raw runtime
recordings and disposable build caches are local artifacts, not source to publish.
