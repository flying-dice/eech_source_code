# Core terrain CRT characterization — source/plan candidate, UNREVIEWED

Required blocker: `core-tree-loader-forest-cloud-rng`; source starts at1516f5a9279dbfbb3a7d8ac309a8bb722c58e40e.
This is characterization BEFORE any loader removal. All10/GenericNative1/Task49/D3
and both full shipping packs retain their gates. No Windows/native/capture ran here.

## Supported entry and finite scope

Ordinary headless `eech_engine.c` boot sets seed/configuration, calls
`load_3d_terrain_game_data` → zone_sc.c → REAL `load_3d_terrain`, then map facts,
population/reset, `reinitialise_entity_system`, and `create_campaign`/session seed.
FOREST construction requires its real tree-data branch; CLOUD construction precedes
the puff guard. `skin_random` may draw when effective skin.animation is nonnegative;
lexical absence of rand in a constructor does not establish zero draws.

Other load callers are gameflow UI initialization and the network MISSION_DATA
client-session restoration branch. The public headless API has no reload operation;
normal server boot/frame does not run gameflow UI. Supported headless compatibility
has no established client packet prerequisite (accepted Tacview source disposition).
RELOAD IS SOURCE-BLOCKED/UNMEASURED: no injected packet, fake API, second boot,
season override or duration hunt. The observer freezes at the FIRST seed after the
completed load; loads=1 therefore describes that window, never all-session dormancy.

## Dedicated observer, never the TP or shipping runtime

New nondefault feature `eech-dc/core-terrain-crt-observer` forwards engine/sys and
force-includes its own header into Core translation units only. TARGET_POINT_FIXTURE
is explicitly incompatible: its forwarding PATTERN is reused, not its Original
target-point hooks/profile, old measurements or binary identity. The sole leased
Linux9d247 build image/cache retain their separately recorded identities.
The unchanged raw aphavoc/modules files are staged with eight strict additive
load/FOREST/CLOUD/skin hooks (`build/core_crt_observer.rs`). Actual compiled staged
bytes are checked against shipping text plus those hooks; original RNG expressions
remain. Feature-only C/Rust/Lua report methods are absent from default shipping.

The wrappers call underlying rand/srand exactly once with original arguments and
return rand's int unchanged. Bookkeeping uses integers/static storage, preserves
underlying errno, and performs no I/O/allocation/time/random call. No extra seed,
dummy draws, RNG solver, flag normalization or campaign decision is introduced.
The GNU feature namespace precedes force-included libc on Linux to preserve the
existing cookie-I/O shim. Native Windows has its own future build/provenance gate.

Capture covers instrumented Core call expressions, not Host CRT or opaque library
internals, even if a platform shares a CRT DLL. Ordered seed/milestone marks,
per-site counts/first ordinals/first-last values and a32-bit FNV fingerprint are
compact observation, NOT a collision-free full per-draw trace or full RNG proof.
No reset/arming API is exposed; later calls still forward after capture freezes.

## Exactly two required maps, unchanged configuration

| Case | Pristine input | Existing scenario | Existing map/campaign |
| --- | --- | --- | --- |
| Georgia | `R:/eech-regression-20261025/roots/georgia` | georgia_retail | map3 / GEORGIA.CHC |
| Lebanon | `R:/eech-regression-20261025/roots/lebanon` | lebanon_retail | map5 / LEBANON.CHC |

Root must first obtain SAME source+this finite-plan clearance, then build a NEW
explicit observer-profile source snapshot/bundle with actual compiler flags,
features, staged sources, COFF/build exits and full payload hashes. No retained3abf
binary is relabelled; OGRE_EE must be absent for the declared constructor scope.
Root then supplies a concrete immutable build/bundle/plan identity and fresh roots
and outputs before any capture launch; no nonexistent binary SHA is pinned here.

The new wrapper delegates the EXACT unchanged `lua/campaign.lua` once, retaining
the same real Engine. That client resolves its own metrics/observations beside
itself, prepares once and selects the reference profile once; census stays66/2.
Fixed arguments for BOTH cases: seed=1, frame_ms=100, hours=0.001, stop=hours,
record_every=10, observe_every=1, observe_supply=1, ordinary accepted recording/
metrics settings, default verified INI/cloud/season (no overrides). Typical argv:
`eech-world.exe <frozen-core-terrain-crt-observer.lua> root=<fresh> scenario=<case> campaign_lua=<verified-bundle>/lua/campaign.lua crt_out=<new>/crt.txt seed=1 frame_ms=100 hours=0.001 stop=hours record_every=10 observe_every=1 observe_supply=1 acmi=<new>/recording.acmi observe=<new>/observations.jsonl metrics=<new>/metrics.json`.
All paths/three literal hashes/controller/PID/birth/direct exits/full pre/post
input rows/full payload hashes must be bound by Root's later materialized plan.
Failed client/boot/frame returns before any report query through a poisoned engine.

Noninvasiveness prerequisite: complete raw ACMI/observations/metrics equality to a
same-config ordinary baseline, plus unchanged full input rows/payloads. Reuse the
accepted3abf Lebanon short ONLY if every argument/input/literal identity matches;
otherwise a specifically reviewed matched ordinary control is needed. Georgia's
matched control is the same finite new observation, not a golden-baseline rewrite.
At most one matched control+one observer per map; no sweep/retry/window extension.

## Required report interpretation and refusal

Report-gate correction (review P2 `ENV-CORE-CRT-OBSERVER-REPORT-COMPLETE-1`):
the driver parses every exact ordered summary/mark/site/skin field and canonical
integer domain before writing any report. It joins capacities, seed count, mark
order=draws+mark-index, load/phase totals, per-site first/count/last and draw-prefix
bounds, constructor balance/scene/skin calls, active versus inactive skin draws,
and gap skin identities. Families occur in the C emitter's order; unknown kinds,
missing load bounds, malformed fields and contradictions fail. The actual first
post-load seed closes the ordered boot-boundary prefix, including an empty prefix.
These are internal consistency bounds on compact evidence, not a reconstruction
of every draw, independent metadata truth, digest verification or consumer identity.
The compiled C/Rust observer/staging remains exact552ee; its prior controls are
reused at their own scope. Corrected Lua/report controls require SAME review
before native capture; the earlier build-only disposition is not capture acceptance.

`core-crt-summary schema1` must be complete1/problem0/one begin+end; seed before
load and actual first seed after load bound the window. Record effective clouds,
season, observed map/dimensions (never infer map number from directory), balanced
FOREST/CLOUD marks and scene, skin.animation/season/default-desert-winter counts,
calls/draws and all Core site/phase totals. The CRT report adds no frame counter;
the unchanged client completion/metrics retain their own bounded-duration scope.
Boot phase marks before/after reinitialization and before create are an ordered
PREFIX if a real earlier reseed closes capture. Later absent marks are outside the
window, not0 work; no assumption that the closing seed must occur in create.
The first gap draw is the minimum nonzero first_gap site, with observed skin scene
if applicable. Attribute its source/value use before declaring a needed consumer
erased by reseed; ambiguous consumer/order remains OPEN, never silently dormant.

Positive: real completed load/seed bounds and real constructor/skin/site traffic.
Inactive skin.animation<0 with recorded call/zero draw is a matched negative;
no FOREST branch means NOT_REACHED, not proof of active-skin zero or permission
to substitute another actor/configuration. Cloud guard is interpreted from the
frozen source enum/effective metadata and loop-site counts/dimensions, not guessed.
Overflow, missing begin/end/seed, malformed/count/order contradiction, setup or
input/payload mismatch FAIL before any absence credit. Seed/value/output difference
is investigated, never normalized or accepted by widening tolerances. Linux controls
are isolated forwarding/wiring/delegation evidence, not actual retail boot, FPU-wide,
all constructor/reload or shipping acceptance. STOP for SAME review before Root build.
