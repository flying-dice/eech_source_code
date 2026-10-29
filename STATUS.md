# D3 ground-contact — original characterization and G1 compatibility complete

EEDCS-48 / PR108, branch codex/d3-ground-contact. Original characterization and
probe/no-probe controls atc119 are retained. The separate G1 composition is now
physically compatible on the declared cases after its reader fix. This main branch
has no G2 production extraction. G2 cutover follows Task47 delivery/base verification.

29 October 2026. Authority: M5 goal11534337v3 and ledger14548994v57 (live).
D1/D2 are accepted. D3/EEDCS-21 is In Progress; D4/D5 have not started. Campaign
geography, routes, planning, timers, scheduling and consequences remain in EECH_DC.
Other physical spatial/model, environment, host-time and required DLL solver/loader
removal remain whole-D3 scope; Task47/48 do not exhaust the epic.

The ground-query fixture found an extraction regression in G1: LuaValue conversion
erased signed zero and tiny request coordinates. Unchanged-binary prefix bisection
located case14. Two clean diagnostic builds/native runs reproduced the unchanged
prefix15 hashes exactly and traced a terrain-height/inputY difference at the
smallest positive Z float. Directed rounding means this lost input can affect height.
The old one-test library RED (executed1/exit101) and every raw failure are retained;
the earlier zero-test invocation is explicitly setup-only, never pass evidence.

The focused numeric-only reader preserves exact bits with one field lookup, keeps
finite exact-f32 validation and prior NaN/infinity behavior, and changes no physical
solver, arithmetic formula, RNG order, case generator, baseline or tolerance.
Actual Linux terrain library6/6, host callback40/40, host all-target Clippy and fmt
pass0; these40 callback guards are not the shipping40.

Clean test-only compositionfebbb7cc5 built0/all46 manifest entries. Georgia,
Lebanon and separate Lebanon-repeat each completed original20k points/5k impacts,
two repeats, native0/pre/post-input0 and119 strict rows. The unchanged comparator
exits0 on all three: all four physical hashes, physical counters/emitted samples,
and common cache observations match immutable originalc119 exactly. Both complete
Lebanon raw files are byte identical. Legacy/semantic diagnostic hashes remain
separate. This closes the bounded physical RED; it is not shipping acceptance.

Safe proof: eech-campaign/reference/d3-ground-characterization/
coordinate-native-conformance-febbb7cc5.json. Source-only independent review has
no actionable finding; verification of the newly completed native evidence is in
progress in the same report. No formal GitHub approval or merge is claimed.

The original frozen06 shipping pack remains live on its same four native processes.
Latest retained23.4MB Tacview/14.56MB observation prefixes match accepted bytes;
this is partial evidence only. Its source predates the production reader fix and
cannot establish final-source acceptance. Prepare a new clean ordinary shipping
build and fresh-root preflight/40-check/eight-complete-artifact comparison; preserve
and explicitly classify the old run when superseding it. Do not infer termination
or restart from a quiet console. No G2 production cutover or task/epic closure yet.

Private inputs, raw coordinates, recordings, source/binary snapshots and logs stay
under R:/d3-evidence. Keep useful RED/partial evidence; remove only disposable
closed caches after verification. Execution checkpoint: R:/d3-evidence/D3-NEXT-ACTION.json.
