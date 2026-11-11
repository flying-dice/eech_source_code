# Normal-fire correlation prerequisite — source candidate, UNREVIEWED

SAME36b24964 accepted the d419 direction only, not code, a producer, native launch
or cutover. This opt-in prerequisite changes no campaign decision, physical answer,
shipping API, baseline, configuration or tolerance. GenericNative1 stays OPEN and
blocks_cutover; all10/prep2/Task49/D3 and D3G08 remain required.

## Producer erratum and actual path

The d419 plan/review's rv_wpn example is disabled by `#ifdef 0` and absent from
`eech-engine-sys/sources.txt`. Preserve those original literals as direction-only
history. The compiled producer is `rv_updt.c::update_server` → readiness →
`update_entity_weapon_systems` → vectors → **vh_wpn.c::update_vehicle_weapon_fire**.
Its real sleep/target/suppression, best-weapon selection, readiness, range, radar
altitude, vector, angle, LOS, fire/reload/burst timer gates all remain unchanged.
The observation begins only at its existing successful launch call; it neither
calls fire nor credits a desired vector, direct fixture launch or global count.

That call runs the actual release gate, creation, `weapon.c::detach_local_entity_weapon`
and host D2 birth in `wn_creat.c`. Detach records the actual configuration/package,
weapon/burst, heading/pitch/station/muzzle presence, raw pose, ammunition,
station visibility, conditional flash timer, map-volume and ground-lift branches.
Birth binds actual launcher/target identities, weapon type and the existing
birth ordinal to that detach and normal-fire interval. Detach and final birth
poses remain separate: the host may legitimately change attitude/launch state.

## Read-only capture contract

`normal-fire-correlation` explicitly enables the existing weapon fixture through
sys/engine/dc. Off, all new producer macros evaluate no arguments. The ordinary
source body is compared to d419 after removing only the additive hook/include
lines; no range, LOS, timer, angle, selection, RNG or solver expression is changed.

Pointers remain private inside the one owner-thread fire/detach stack. Identity
uses pure `campaign_roster_host_id`, never lazy identification or registration.
Hooks perform integer bookkeeping/raw-word copies and bounded integer formatting;
no I/O, allocation, random draw, provider call, initializer or reset is invoked.
The clock getter only returns the existing `simulated_ms` word (`eech_log.c`).

The normal stream has its own monotonic sequence, 8,192 fixed record slots and
drain cursor. Existing `ready()/trace_reset` never owns it. It is appended through
the existing `session:wpn_events()` read surface as `normal-ev`, independently of
legacy `wpn-ev` order. Whole records drain once; reads do not erase capture or
reset sequence. Overflow/truncation is an explicit error, not UNREACHED credit.
No reset, replay, provider replacement or new launch capability is exposed.

`normal-fire-correlation.lua` is a pure collector, not a preparing driver. The
caller supplies a capture label, frame and expected previous simulated-clock word
for the supported 100ms profile. The real engine advances that clock at frame END;
a tick is not an invented native frame ordinal. Exact schema/domains, sequence,
frame/clock relation, source/target/weapon/configuration/package/detach/birth joins,
actual effects and closure are checked before correlation. Fire=0 diagnostics
are validated too. Missing devices retain their actual fallback; zero births
retain no-eligible status. Native/observer/malformed/context failures latch before
UNREACHED and reject every later callable continuation. No actor is selected.

## Actual private controls and limits

The first complete private GREEN is d17: one engine and one sys Rust test. The
normal positive uses the compiled Bradley TOW package, a stationary Abrams target
1km ahead, the real synthetic-model writer/Original loader/constructor/search/
viewpoints, actual readiness/systems/vectors/vehicle selection/fire/release/create/
detach/birth, and this engine instance's real aim/environment converters with the
existing reference D2 kernels. Three fixed updates respect previous-frame vectors;
no package angle or firing verdict is assigned. One actual birth consumes 2→1
rounds with ready/vectors1. Real request counts are retained in the direct log.

Declared hand-made environment: heap/records, sector/session/update/group/roster,
initial loadout/engagement/ready animation/timer, flat terrain/clear LOS, a private
sound descriptor and imported reference environment field. Sound/wind effects run
through the real kernels. This is the `model-fixture` Original model profile,
not a retail installation, Engine boot, current host-model authority or native case.
The fixture is a dedicated process; global/model/table/heap cleanup and broad
FPU/lifetime restoration are not claimed. Aim callback pointers are cleared
before their Rust context ends; the environment seam refuses an existing binding.

Five real detach cases preserve the original full-device, absent heading, absent
pitch, absent station and absent muzzle branches. Missing heading/pitch keeps
launcher origin but still decrements ammo; absent station uses muzzle, absent
muzzle keeps station. No universal absent-device refusal is invented.

Final affected controls also feed the ACTUAL C-produced positive stream to real
Lua, then single-field identity, clock, unrelated-weapon, extra-field and malformed
ineligible-record faults, plus no-eligible/failure/frame precedence. Four NEW
hook-only modes compare x87 control/status, MXCSR with nonzero flags, errno and
the shared RNG word; missing-identity lookup runs with no aim provider. Drain
controls prove late legacy initialization/read does not erase or duplicate capture.
These are selected finite x86 controls, not all-platform/signalling-NaN/FPU proof.

Development receipts remain literal: loader/provider/file-map/texture/callback/
roster/environment/link setup failures are not product RED. d8 is behavior-positive
Original/no-observer missing-correlation RED; d10 exposes candidate legacy-reset
record loss. d21 is the genuine malformed-ineligible adapter RED with its exact
Lua/test bytes retained; d22 is GREEN. These are unsealed working-tree development
runs, not retrospectively frozen shipping identities. Final current-source checks
and their timed PRE/POST map are the freeze evidence.

## Remaining concrete native gap

This prerequisite selects no native case and supplies no new controller. After
SAME code review, Root must materialize a distinctly identified observer binary,
Lua/capture/frame binding and fresh inputs, then obtain launch review. The existing
Lebanon seed1/100ms/30s warmup/120s settle bounds remain fixed. Healthy actual normal
fire/cohort plus the SAME moving target's lawful need_updated/store/pose effect
must exist before the existing eligible terrain-input variation is considered.
Old hit_land underreach and stationary READ400394 remain reused at their scopes.
No native/natural moving success, collision outcome, D3G08 ownership move, loader
removal, full40/all8/fourinputs/census/CI/formal review/merge or whole-D3 acceptance
is claimed. Both original fullpacks remain untouched and prefix-PARTIAL only.
