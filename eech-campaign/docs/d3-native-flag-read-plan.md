# Public/native flag READ plan — nr7e6 (UNREVIEWED, NO LAUNCH CLEARANCE)

Resolve one unmet part of `ENV-PHYSICAL-FLAGS-HOST-STORE-NATIVE-ACCEPTANCE-1`:
an actual public Lua → native → host-store READ, a changed word carried back
into Core's next candidate question, and immediate READ refusal. This is a
plan only. No driver implementation, build, native process or campaign was run.
The adjacent [identity record](../reference/d3-entity-collision/native-flag-read-plan/identities.json)
pins the current source, existing binary inputs and accepted old Lua literal.

## Concrete supported transaction

`mb_hostentity.h` accepts `CAMPAIGN_MODEL_CANDIDATE_NEED_UPDATED` when candidate
fact6 MOBILE is nonzero, fact10 OF_CITY is zero, and fact20 UPDATED is zero
(zero-based indices). It does **not** require speed greater than one. The
reference `entity_candidate_step.c` asks naturally only inside its
`sqr_velocity > 1` branch. Thus a stationary real mobile can legally be asked
for this service by a diagnostic host. This is an extra destructive READ,
distinct from the ordinary reference host and unchanged-default conformance.

The proposed opt-in Lua strategy is `request_updated=first_eligible`, OFF in
ordinary scripts. At the FIRST real candidate meeting all of the following:

1. The request parses as exactly44 words plus its nonempty semantic host ID;
   scene is positive, MOBILE is exactly1.0, OF_CITY and UPDATED are exactly0.0.
2. Delegate that exact request to `host.model` ONCE first. It must return a
   valid non-refused candidate answer. This establishes the provider's actual
   canonical movement record, not an invented actor or pose. Missing, retired,
   unknown, incomplete or busy records retain their real refusal unchanged.
3. Record its existing result/name; return `{result='need_updated',name=name}`
   with NO `p`. Preserve a natural `need_updated` as natural; otherwise label
   this one answer as diagnostic. No other candidate decision is changed.

The current Lua bridge expressly accepts that result/name with no pose. The
real C offer loop then invokes UPDATED exactly once, which calls
`campaign_model_flag_read(UPDATED,slot)` and the host's phase21 FLAG_READ via
the shipped model callback. The native bridge → reference `flag_answer` →
FlagStore performs its real destructive `word &= lane_mask`. Core converts
the returned raw word to fact20=1.0 for zero or2.0 for nonzero, and immediately
re-asks the same candidate. Delegate that re-ask unchanged to `host.model`.
No object creation, target linkage, firing, motion override, scan reorder,
campaign policy change, fake geometry, private entry or new shipping API.

Selection is first-in-observed-order, not actor/scene/distance/frame search.
The prior35-frame receipt contains no eligible-candidate census; an eligible
candidate's presence or first frame is UNKNOWN. Fixed-wing M5 frame2277 is
context only, not READ evidence, and supplies no window choice.

## Bind the candidate and the carried word

Hold one local diagnostic selection tag, `1`, with entry/frame, candidate-call
ordinal, scene, semantic host ID, all44 pre-request words, delegated name/result,
and the existing bind_mobile registration tuple (issued host ID, member_key,
group_key, registration ordinal). The reference host issues monotonically
increasing `eech-world:n` IDs; no raw heap slot/pointer is exposed. The candidate
protocol has NO separate generation field: do not invent one or claim a new
generation guarantee. Match the actual registration tuple and host identity;
unavailable or inconsistent correlation is FAILURE, not a substituted record.

Allow only the resulting optional GROUP and the selected UPDATED READ before
the re-ask. Match the READ's issued flag-group key, array0 and lane to the
observed group/traffic transcript. This key identifies a word group, not an
entity ID. Require the next candidate query to match the selection's scene,
identity and every word except20; fact20 was0 and must now equal1 or2 as the
answered raw word requires. Another callback, record/identity change, another
candidate, malformed request or lost correlation invalidates the transaction.
The old driver's `need_updated` latch clears on the next model callback, so
an intervening GROUP would lose it. The new driver must carry the selection
through exactly that optional GROUP and bind its issued key to the READ;
preserving the old code verbatim is insufficient for this new strategy.
Children must match the healthy reference's complete pre-intervention tuple:
selection/registration tag, frame/entry, candidate ordinal, flag ordinal, and
all four phase counters before the READ. No later eligible actor is substituted.

## Seven bounded processes, healthy first

All are ordinary HOSTED3abf binaries, same new opt-in Lua literal and fixed
Lebanon retail setup: seed1,100ms, hours=`0.001`, at most the established35
frames. One fresh verified retail root, immutable binary copy and exclusive
report per process. No duration extension, scenario sweep or read-window hunt.
No new Original run: the old Original/control evidence is reused only at its
actual020/5d2c scope; it is NOT a cross-source paired reference for3abf.

| Order | Proposed mode | Required result |
|---|---|---|
|1|`read_probe_control`|Actual eligible selection, accepted READ, zero oracle faults, exact unaltered carried fact; no fault intervention. Otherwise UNREACHED or FAILURE as below.|
|2|`read_changed`|After the real provider READ, return0 instead of the lane mask, or mask instead of0, keeping array/lane/state valid. Require changed fact20 on the exact re-ask, different from the real word's fact.|
|3|`read_withheld`|After observing the real provider answer, withhold it; exact immediate `entity_scan.callback` fatal.|
|4|`read_malformed`|Two patterns instead of three; exact immediate `entity_scan.answer` fatal.|
|5|`read_wrong_state`|Valid `height` reply to a flag question; exact immediate `entity_scan.state` fatal.|
|6|`read_wrong_array`|Echo another array; exact immediate `flag.answer` fatal.|
|7|`read_wrong_word`|Raw word neither0 nor this lane's mask; exact immediate `flag.word` fatal.|

Each child is NOT CREATED OR LAUNCHED until the complete healthy control is
validated as PASS with a carried tuple. If none is eligible within35frames,
record control UNREACHED and all six children NOT_LAUNCHED_UNREACHED; STOP.
The old17PASS and old six UNREACHED cases remain immutable, not overwritten
or converted to passes by this new strategy. No collision outcome is promised:
a stationary candidate can legally carry UPDATED while its reference solver
does not use it in its physical result. Natural moving-weapon integration is
separately required and receives no credit from this diagnostic request.

## Store effects and causal refusal

Delegate the selected READ once before changing/withholding its returned reply.
Consequently the real store's destructive read has already happened in every
mode, including refusals. Maintain the existing observed-traffic oracle from
boot: before-word W, mask M, real reply must equal W&M; after-word is W&M and
the other lanes are destroyed. Report W/M/real word/answered word/post-read
oracle word separately. A changed returned reply does NOT write a second word
into the store; the next Core candidate fact follows the answered word while
the real store retains the real read result. Do not normalize masks or repair
the neighbour effect. The oracle and actual provider reply are observations
of traffic plus the accepted store semantics, not a new direct whole-store
snapshot capability. No additional host-only read or reset is inserted to
inspect the store, and no broad postfatal cache/atomicity claim is made.

Reuse the existing continuation latch across ALL callable surfaces, including
the actual terrain/environment/sample providers. A refusal PASS needs exactly
one matched intervention and the SAME engine entry stopping at the specified
marker, with no intervening host callback and no normal entry return. A fatal
before intervention, wrong marker, other entry, later callback, malformed
positive, request/reply fault or oracle difference is FAILURE, taking precedence
over UNREACHED. Read nothing from the poisoned engine afterwards.

## Identity, schema and commands before any launch

Runtime source is EXACT `3abf03e5133279ae6ed53b657e76d4f7724f7dd3`, not the
plan/source-review HEAD `7e6bed4c702869611ead639610c47abc3d86aa08`. Six protocol
files in identities.json have identical Git blobs at both commits. Current
test-gate/include raw changes elsewhere do not relabel retained binaries.
Reuse a NEW immutable copy of the complete91-payload ordinary bundle; never
stage Lua into the running pack's source/bin/root/output. Existing hashes:

| File | SHA256 |
|---|---|
|eech_dc.dll|`208ebca3033481dcf5474bc9343beacdb4bf2eeaa6e6abd43b09d9c2bbaf8a43`|
|eech-world.exe|`2b4d06b9083e9cbc9268501b2ed565e4e3ee4e73bcc250234d67945d6f82b65a`|
|lua.dll|`ad9a305565905b4ea1421f7c053f9ebd6ebec9bb5439b89f117451f1e90ece36`|

Derivation input Lua is the current UNCHANGED physical-flags-witness.lua,
Git blob `add513356a6fd4eb00f7138dd95b93b5863c2145`, SHA256
`30adcae241d43deddb710d014ac4585b3c0f282a7309b833c77c08cc71881ad7`.
It DOES NOT implement this strategy. Proposed new `physical-flags-read-witness.lua`
is UNCREATED, with NO hash or launch clearance. Do not pass a nonexistent
`request_updated` option to the old driver and mistake ignored arguments for
coverage. First review this plan, then implement/test/freeze that opt-in Lua,
its strict report validator and controller. Root/SAME must review the literal
new source/hash and exact controller/bundle/input manifests before native.
No compiled capability is presently missing; no rebuild is proposed unless
that later review identifies an actual binary/source incompatibility.

New report identity is `pf-read-identity schema=1` (a NEW schema family,
not relabelled old pf-witness schema1), with exact canonical key sets/order.
Require runtime commit, Lua hash, plan hash, strategy, mode, HOSTED profile,
scenario/map/campaign/seed/frame_ms/hours and healthy-reference file hash.
Records must include phase attempts/answered/accepted, oracle counts/faults,
the selection/registration tuple, full request/re-ask44 words, READ key/array/
lane, real/answered word and carried fact, entry/intervention/first-following-
callback/fatal, and one final PASS/FAILURE/UNREACHED verdict. Validate references
COMPLETELY before output creation, root preparation, boot or provider access.
The future frozen validator must refuse wrong runtime/Lua/strategy/setup,
missing tuples, duplicate/out-of-order records and false healthy counts.

After those gates, Root materializes commands of this form (NOT executable
authorization now):

```text
<new-copy-of-pinned-3abf>/eech-world.exe <FROZEN-new-Lua> root=<fresh-root>
 out=<exclusive-report> mode=read_probe_control request_updated=first_eligible
 scenario=lebanon_retail hours=0.001 frame_ms=100 seed=1
<same-binary> <same-frozen-Lua> ... mode=<one-child> control=<complete-healthy>
```

Select the reference arithmetic profile using the existing exact helper before
boot; preserve Lua5.1.5/x86_64 release and input identities. Each process needs
direct exits, full pre/post retail manifests, complete immutable payload/Lua
checks and exact control hashes; caught expected engine refusal may give the
diagnostic native exit0, not healthy campaign completion. No run/build now.

All10, generic NativeAcceptance1, public READ until observed, whole FPU/lifetime/
DCS/height/sharedterrain/RNG/hierarchy/services/natural moving/final census,
CURRENT40/all8/fourinputs/currentCI/formalreview/normalmerge/Task49/D3 stay OPEN.
Both existing fullpacks and all old evidence remain untouched. STOP for SAME
correctness, clean-code and boundary-safety PLAN review.
