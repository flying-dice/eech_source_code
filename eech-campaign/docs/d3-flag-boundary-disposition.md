# Supported flag boundary: disposition candidate (UNREVIEWED)

Task49/D3, source b4ddd9ca2dd05c79b847ef6a8e7a32edb36c949a; family fpuB4.
Live M5 goal v3 and migration ledger v195 were read. This is source/evidence
reuse only: no code, API, semantics, test, build, native, cache or baseline change.

## Current route and accepted raw behavior

`mb_int.c` hosted setters/getters and `up_update.c` resets call
`campaign_model_flag_set/read/reset`; `flag_group_key` opens GROUP as needed.
`mb_hostentity.h::campaign_entity_flag_exchange` sends entity_scan phases19..22
through C model exchange → `eech-engine::model_callback`/owned MODEL →
`eech-dc::model_provider`/MODEL_BRIDGE/call_model → reference
`model_host.rs::entity_scan_answer/flag_event/flag_answer` → the ONE FlagStore.
Replies take the same route back, with raw words and issued key checked.
The real GROUP/SET/READ/RESET store is not a boolean pose-freshness approximation.
[R-KEY] accepts the raw199, guide56 and collision15/16 controls at their stated
Original/Hosted TEST-bridge scope; it does not make them full native measurements.
[R-NATIVE] adds actual public boot/UPDATED READ/carry/five immediate refusals only:
initialword0, array0/lane10/frame16, no optionalGROUP, no populated-neighbour proof.

## FPU: reuse the common outer restoration, with explicit inference

**Accepted measurement:** [R-FPU] records16 measured public entries at runtime80fd:
all four initial rounding controls, nonzero seeded status, exact x87 control/status
and full MXCSR words on healthy counter/callback and fresh-process fatal entries.
**Current source join:** `eech_engine.c::ENTER` saves fenv and complete MXCSR,
sets toward-zero; normal LEAVE and fatal setjmp return both restore fenv then MXCSR.
Current flag calls run inside those entries and use the existing model callback;
they introduce no alternative outer exit. Reference FlagStore operations are
integer/key bookkeeping; the Lua bridge transports three exact bit patterns.
Thus reuse of the common supported x86 outer-word restoration is a SOURCE
INFERENCE from [R-FPU] and the current route, not a new flags-specific FPU reading.
It does not claim arbitrary provider inner-state preservation, unmasked traps,
other architectures, register-stack contents, or every consumer/global/RNG property.
No repeat wrapper/word experiment is required for this supported mechanism alone.

## Ownership/lifetime: one persistent owner, not replacement safety

`Engine::boot` uses BOOTED.swap before assigning the owned static ModelProvider
to thread-local MODEL; Engine's PhantomData prevents Send/Sync. `with_host` lends
an entry context, but model_callback reads MODEL, not that borrowed user pointer.
`campaign_model_set_host` retains callback with user=NULL; LEAVE/fatal do not clear it.
`model_provider` owns its bridge LuaFunction. Reference main registers ONE
SharedRoster; SectorRoster::clear preserves flags, and C group keys persist.
[R-LIFE] accepts actual owning-engine/preboot normal/NULL/error/fatal/poison controls
(six callbacks/one group/word32/zero postpoison calls), with no global resets.
[R-NATIVE] now establishes successful public boot and reached READ for this binding.
The proposed supported disposition is therefore one boot attempt, one persistent
provider/store/VM binding on its owner thread. VM/thread/session destruction or
replacement, reentrancy and cross-thread transfer remain unsupported/unproved.
A fresh store can reuse fN and alias an old group: no guaranteed refusal, epoch,
restart/reset architecture or universal lifetime promise follows from this join.

## Needed DCS contract: internal metadata, separate physical decisions

GROUP/SET/READ/RESET need an owned key→three-word host store reproducing the
accepted OR/destructive-mask/reset-extent semantics. No simulator flag query is
required by those operations: the source consumes group eligibility, array, lane
and issued key, not DCS geometry, clock or an object property. This is a source
contract inference, not an assertion that an external DCS API is absent or tested.
Physical pose/time/arrival semantics remain their separate needed host contracts.
For recognised supported mobile members with a task leader, guide.c first routes
through hosted supply/radius arrival (HOST_ARRIVALS is unconditional); its legacy
fw_msgs pass-through fallback still exists. This is NOT blanket fallback dormancy.
The older D3G-03 boolean/DCS-sampled-UPDATED proposal predates the whole-word move;
do not implement it as a second truth/store or silently close all its consumer rows.
No new DCS API declaration or live DCS/runtime/thread guarantee is claimed here.

## Decision and first remaining physical coupling

Candidate: reuse the accepted mechanisms above for the supported flag boundary;
no genuinely new flag FPU/lifetime experiment is identified by this source join.
Generic NativeAcceptance1 stays OPEN: this is not all consumers/arrays/nonzero
native behavior, natural moving-weapon integration, final current40/all8/fourinputs,
final census/CI/formal review/normal merge or whole D3/all10 acceptance.
The next concrete production responsibility is the already-open hosted raw
TREE/FOREST/CLOUD loader removal, NOT another flag prototype. Its FIRST missing
fact is effective constructor/cloud-loop CRT draws and the first consumer before
the session reseed: `eech_engine.c` boot → `zone_sc.c::load_3d_terrain_game_data` →
`terrdata.c::load_3d_terrain`/FOREST_TREE_OBJECT/CLOUD → `skin_random`, then session
creation/reseed. Existing D3G-04/card retain the bounded capture requirement;
effective season/skin/consumer ordering is unmeasured, so no removal, dummy draws,
new observer or empirical case is implemented here. Stop for SAME disposition review.
Shared terrain/height/hierarchy/services/DCS/natural moving and both fullpacks keep
their independent open gates; checked prefix equality remains partial. Prep2 stays;
no D4/D5/TypeScript. Exact evidence/source identities are in [the compact receipt][ID].

[R-FPU]: ../reference/d3-model-characterization/independent-reference-host-profile-closeout-e48808f53-20261030.json
[R-KEY]: ../reference/d3-entity-collision/physical-flags-keyfix-020-reviewed/README.md
[R-LIFE]: ../reference/d3-entity-collision/physical-flags-lifetime-reviewed/independent-flag-lifetime-0c7b-source-20261108.json
[R-NATIVE]: ../reference/d3-entity-collision/native-flag-read-accepted/independent-native-flag-read-nr602-actual-20261108-v2.json
[ID]: ../reference/d3-entity-collision/flag-boundary-disposition/evidence-identities.json
