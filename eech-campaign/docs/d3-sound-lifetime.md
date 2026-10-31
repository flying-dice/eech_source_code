# D3: original sound entity and lifetime characterization

## Controlled-data actuals — 31 October 2026

Seven cases4..10 now complete on original native116ffd5fb, with separately pinned Lua2a for4..8 and d4 for9/10. All seven successful native/controllers0,14successful pre/post inputs0,55payloads and driver post-hashes unchanged. Initial9fails witness/native/controller1 on a false total-child isolation guard; the retained C report/restoration is complete. Corrected d4 checks matching subtype1/returned1/subtype identity while permitting unrelated ambient children. Only replacement9 and previously unrun10 then execute; the five earlier successes are reused. Every old9 preterminal C raw row AND ending equals fresh9 exactly. Whole raw remains unequal at witness_failure versus result. Eight actual controls including failure have16pre/post inputs0; old failure is not a success.

Measured states: server life5→3→1→kill; client5→3→1→0 without kill; two-sample constructor4; ordinary/music caps40/300; both looping entities remain0/alive; invalid server/client validity lifetime1.5→0.5→0, then next-update unlink, no kill and effect lifetime5 unchanged. These words are outcomes, not fixture-side expected-value assertions. Freed slots have no sound data; their zero words are not a live lifetime. The new metadata/transition/cohort checks only join observations and declared inputs.

[Safe proposed actual evidence](../reference/d3-environment-characterization/sound-domains-original-native116-drivers2a-d4/README.md) records commands, eight controller identities, failed9, all script/native distinctions and retained source findings. The same combined independent actual/audit/safe-publication review has no findings, private-original SHA256 `89c26171dc9ec62080647db6a9b1b88b627837cb2bf32de694ce3bd902caa3d2`; original_domain_characterization_closed applies ONLY to these measured cases4..10. Raw coordinates/allocation slot IDs remain private; baseline/tolerances/measurement bytes unchanged. Only this measured original domain is prepared for closure, not host sound ownership, Task51/D3, shipping or Tacview. Remaining resume/device/destroy/network/pitch/amplitude and whole-D3 requirements below remain explicit.

## Measured checkpoint — 31 October 2026

The corrected original native `9429beefb9a8e143b648e01c390075882e6e6361` with pinned Lua-only driver `e1dc758074fba7fa8a4af99604ceecb85cf20483` has now run all four controls. Native/controller exits are all0, eight pre/post retail manifest checks pass, and all55 native payloads remain unchanged. Source review and the same independent actual/comparator/safe-publication review are clear (private-original review SHA256 `c0119559a4de76b6dde864dc50417dba53a2c90d4114830dabd41c266a044f0d`). Only the measured zero-start valid nonloop/no-handle scope and three named controls are closed; all nine e1dc CI jobs pass. [Safe evidence](../reference/d3-environment-characterization/sound-entity-original-native942-driver-e1dc-closure/README.md) distinguishes native and script identities and reused gates.

The natural 120simsecond/1200frame Lebanon report preserves **all17,257 old host-queue native7db raw lines and endings byte for byte**. The strict whole-file comparison remains FALSE/exit1 because the new report adds26 sound diagnostic lines. There are no other differences; no baseline, tolerance or measured file is rewritten. The three new sound primitives have no equivalent retained old sound reports.

Actual sound facts: enabled0, initialized0, paused0;2069 sample descriptors; lightning and thunder rate/original-rate22050 and size0. Four natural sound entities start at lifetime0 and each expires without a handle and is killed on its first update (four updates/four kills/sixteen records, no drops/untracked creations). Server control1 likewise kills on its first0.1s update, safely skips the second call and restores delta. Direct client control2 runs twice, clamps to0, remains alive and restores delta; it is not a real client/network delivery. Control3 creates nothing and reports prerequisites. These process facts do not establish missing retail audio assets.

The Windows942 build and corrected C/World gate pass. Earlier c9 broad default/observer release and both strict checks are explicitly reused only under the retained source equivalence; its unsafe old fixture/binaries are unused on retail. The e1dc Lua-only production/build-input identity proof binds both script overrides to native942. Current fmt/5sound/45host-callback/World strict pass. Earlier failures remain retained; the first overwritten d1 raw development log remains unavailable, with only a labelled excerpt and three partial user tool-result failures. No recovered raw log is claimed.

Sound behavior remains original and coupled. Playing/idle handles, invalid/looping/unlink paths, nonzero sample data and audible/player/network delivery are uncovered. This checkpoint does not accept Task51/D3, shipping or a host sound extraction. The design and source account below record the original probe; statements about planned commands and unmeasured behavior describe its preparation, superseded only by the scoped actuals above.

EEDCS-51 / D3, the physical-environment responsibility chosen next after the lightning queue. This test-only probe measures what the ORIGINAL code creates and how it ages it, before moving that behavior to the host. The sibling probe for the strike queue is [d3-lightning-queue.md](d3-lightning-queue.md); the
shape is the same (a separate finite probe store, a fixture group, a natural-run block of the retained driver, a control driver, guards). Every statement under "What the source says" is a reading of the code,
not a measurement; the measured checkpoint above states exactly which facts are now known.

## What the source says (read, not measured)

* `ss_updt.c`, the hosted `create_lightning_effects` (active) and the legacy `update_lightning_effect` (`#else`) create two sound entities per lightning expiry through
  `create_client_server_sound_effect_entity` (the lightning sample, then the thunder sample), each with a pitch of `0.85 + 0.3 * frand1 ()` drawn **between** the creations: the draw order and the entity
  allocation order are part of what must be preserved. The observer already brackets each creation with `ENV_FX_SOUND_BEGIN` / `ENV_FX_SOUND_END` (the next free entity indices before and after).
* `soundeff.c`: every creator passes through `create_local_sound_effect_entity` (the real constructor). It creates the entity, links it into the parent's special-effect list and, when the effect is valid, the
  update list, and `set_local_sound_effect_sample_indices` gives it `effect_lifetime = sum(size / rate)` over its samples, clamped to 40 s (300 s for music).
* `se_updt.c`: the SERVER routine takes `get_delta_time ()` (the global `system_delta_time`) off a valid, non-looping effect each update; below zero it kills the entity when there is no playback handle or the
  handle is not playing, and leaves it alone while the handle is playing. The CLIENT routine takes the same delta, clamps the lifetime at zero and never kills; an invalid effect is unlinked from the update
  list and its handle destroyed after `valid_effect_lifetime` runs out.
* The playback handle (`sound_effect_data`) comes from `play_local_sound` (`se_snd.c`) through `create_single_system_sound_effect`, which returns NULL while `sound_system_initialised` is false. The headless
  frame never calls the play path. Whether the headless build initialises the sound system at all (it is enabled only when the command line does not say no-sound and the process is not a dedicated server) is
  **not known**; the probe prints the flags as facts.
* `application_sound_effects[]` supplies `rate`, `size` and `original_rate` per sample (`snd_data.c`; `size` from the retail `audio/sound.hdr` when present). A full retail boot may therefore load real data; do not
  assume it does not (the earlier texture-model assumption was wrong in exactly this way).
* The wind, rain and terrain ambient children that consume sound are the ambient routine already characterised (groups 11 and 12); this probe does not touch them.

## What is added (test-only; the scope of "feature off" is stated exactly)

**Scope of the feature-off claim.** Only the two PRODUCTION hook bodies, `soundeff.c create_local_sound_effect_entity` and `se_updt.c update_server` / `update_client`, keep their executable tokens when
`ENV_FXE` is off (the macro expands to nothing, no argument is evaluated; the added `#include` and the macro definitions are not body tokens and the hook lines shift `__LINE__`). That is a source reading, not an
object-file, numeric, runtime or shipping equivalence, and it says nothing about the other files. `eech_env_fixture.c` is a TEST fixture entry point and changes even when the observer is off: the group catalogue
bound `ENV_GROUP_LAST` moves from 13 to 14, and group 14 is dispatched to a diagnostic refusal (`sound_lifetime_probe_needs_the_env_observer_build`) in that build. `eech_env_observer.c` is test-only code compiled
into the env-observer build. The Lua entry point `env-characterization.lua` has new optional code (`sound=on`); that is not C preprocessor identity and is judged by the driver's own guards. The C changes of this
checkpoint (the fixture's skip of a second update after an early kill, the probe's untracked-creation counter, the report and end-row fields below) require a NEW native build: no earlier binary is reused for them.

1. **A separate probe store, not the event stream** (`ENV_FXE`, `ss_envfx.h`; the observer's kinds, sequence numbers and transcript grammar are untouched). Eight points: `CREATED` (end of the real
   constructor), `SERVER_ENTER`, `SERVER_EXPIRED_NO_HANDLE` / `_IDLE` / `_PLAYING` (the three branches below zero), `SERVER_KILLED` (straight after `kill_client_server_entity`; only the entity slot is read, the sound
   data is gone), `CLIENT_ENTER` and `CLIENT_EXIT`. The hooks are in `soundeff.c` and `se_updt.c`, which are not under the host-env alternatives: they compile on the current hosted path. `ss_envfx.h` is now
   included unconditionally by both (the macros expand to nothing when the observer is off, as in `ss_updt.c`).
2. **`csrc/eech_env_observer.c`** records a creation only while a lightning sound creation is open (`ENV_FX_SOUND_BEGIN` to `ENV_FX_SOUND_END` of the observer's own stream) or while a fixture control has switched
   tracking on, and follows at most eight entities. It READS: the stored fields are copied **as bits** with `memcpy` (no float is loaded by value, so no x87 narrowing), the handle is a present/absent flag, the list
   links are present/absent flags, and the sample table's `rate`, `size`, `original_rate` for the first sample are read. It draws nothing, allocates nothing, formats nothing at the point and writes no game
   variable. At most 80 records are stored; each entity's first three updates are stored whole and the rest are only COUNTED (`upd` on every later record says how many ran). Records beyond the capacity are counted
   as dropped and the report says so; the driver refuses a report that dropped anything. A creation after the eighth tracked entity is still recorded as a creation (`tracked=-1`) and COUNTED (`untracked=` in the
   report); its updates, expiry and kill are not followed, so its lifetime is NOT characterised. The cap is not raised to hide this.
3. **`csrc/eech_env_fixture.c` group 14** (explicit only, observer build only). Index 0: the plan, the facts of the sound database and the sound system, two UNSUPPORTED rows and the probe's report and records. Indices 1 to 3
   are PROCESS-ISOLATED controls (an entity cannot be put back): the real constructor builds one entity with the lightning routine's arguments, tracking is switched on, and the real compiled update routine is
   called through the entity function table with `system_delta_time` set to a value the control names (the delta is restored): **1** the server routine at 0.1 s then, if the entity is still a sound effect, at a
   delta far beyond the lifetime; **2** the client routine at the large delta, then at 0.1 s (clamp, no kill); **3** nothing is created: the facts and the explicit statement that the playback-handle branches are not
   exercised. The lifetime is whatever the loaded sample data gives and is never lengthened for the fixture. The real server kill frees the sound data and clears the slot's type and data, and the sample may be short,
   empty or missing, so the FIRST call can end the entity: after each real call only the stable entity-slot type and data are read, a terminal entity is not updated again, and the end row says `calls=` (1 or 2),
   `second_skipped=` and the slot's `etype_after` / `data_after`. A genuine early kill is therefore a legitimate one-update report, not an error and not a preferred two-update oracle. All rows that can fail to fit
   are written before the entity exists and before the delta or the tracking is touched, so no return path leaves either changed. A process that is not the server, has no session, or has no free entity slot for
   the constructor (`get_free_entity` would return NULL and the constructor would dereference it) REFUSES cases 1 and 2 and says why.
4. **Drivers.** `lua/env-characterization.lua` gains an optional `sound=on` (needs `observer=on`; default off). It only APPENDS rows after the queue rows. `lua/env-sound-lifetime.lua` (new, one control per fresh process,
   `case=1..3`) arms the observer before boot, writes the fixture's rows verbatim with every value kept as text and judges **nothing physical**: it checks the report's shape and that each tracked entity's chain of points
   can occur, so any lifetime, any expiry branch and any kill outcome is equally accepted.

### Natural-run rows (`sound=on`)

Raw (scope `sound:report`): `env-snd-plan`, `env-snd-facts`, `env-snd-unsupported`, `env-snd-report`, `env-snd` (one per point; words as text). Derived by the driver from those rows, never assumed:
`sound_entity` per tracked entity (`n` 1 lightning / 2 thunder, `created_frame`, `updates_at_last_record`, `expiry` = `none` / `no_handle` / `idle_handle` / `playing_handle`, `killed`, `handle_ever`, first and last
lifetime words), `sound_summary` and, only when a creation was not tracked, `sound_untracked`. A report is refused, never truncated or repaired, if it is out of order, dropped records, has a record sequence with a
gap, a chain whose points cannot follow one another, an operation that began and did not finish (an expiry with no or an idle handle without its kill record; an entry into the client routine without its exit), a
record of another entity or of another `n` than its creation, an update counter that is not what the declared sampling allows (the j-th stored entry of an update is update j and only the first three are stored; an
exit carries the update of its entry; an expiry or a kill belongs to the RUNNING update, so until all three mandatory entries have been stored it carries exactly the current stored-entry count and may not run
ahead across an unstored entry; only after the third entry may it carry a later, only-counted update, and it never goes below the update before it; a kill carries the update of the expiry straight before it; the
two-call control stores every entry, so there an expiry and a kill carry exactly the current entry count), a per-entity count above the probe's total, or a total that differs from the entities' counts when every one of them is exact (a killed entity's count
is its whole count; a count below three is whole because a further update would have been stored; so is the count at or after an observed expiry, because in the supported journal every later update stores another expiry, alive or not; only a count of three or more with NO expiry is a lower bound). Each creation is matched to the observer's own ordered `sound_begin` /
`sound_end` pair, not only counted: the same ordinal of the routine (1 lightning, 2 thunder), the same frame, and the entity index the observer saw next to be given (the constructor takes the first free slot,
`se_creat.c get_free_entity`). The creator `create_client_server_sound_effect_entity` has no branch that skips the constructor (only assertions and the message that follows it), so each pair holds exactly one
creation; heap exhaustion would fault, not skip. Tracking loss is classified, never hidden: `sound_summary` carries `untracked_creations` and `lifetimes_complete`, and a `sound_untracked` row says that those
lifetimes are not characterised.

**The playing-handle journal (source-only; no handle is ever created by this work, and the stand-ins are canned protocol checks, not backend availability).** In the supported journal every record that reads the
entity is valid and non-looping (as `create_lightning_effects` passes them; a record outside it is refused, not recomputed, because an unrecorded change of `valid`, or of time, would make the rules below
unprovable). There the server routine goes below zero once and stays below (the delta is positive), and the probe samples only the entries and the exits after the third entry: **every** expiry point and kill is
stored. So once an expiry with a playing handle has been observed at update u, the next record is the entry of update u+1 (while entries are still stored) or the expiry of update u+1, and an entry that follows an
expiry is followed by the expiry of its own update; a journal with a playing expiry at update 3 and then at 7 and 9 is NOT coherent and is refused. A gap before the FIRST expiry is legitimate (the lifetime was still
positive) and stays accepted (`sound_long`), as do alive lower bounds and one-update early kills. The handle is what the branch says: none for an expiry with no handle, a handle for the two others; the real
constructor leaves none at creation; once a record shows a handle it stays until the kill; an expiry sees the handle of the entry of its own update. The accepted positive form (`sound_playing`) is the complete
consecutive journal 3 to 8 with the handle from update 3, in frames inside the run (every record is in a frame the probe saw begin, which must equal the observer stream's frame_begin count); its last counter 8 is EXACT while alive and the totals say 8 per entity and 16 in all, with no unseen update 9. A total above the exact counters (a hidden trailing update) or a record beyond the run's frames is refused. What the lifetime or the playing time is, and whether the headless process ever gets a handle, is not predicted.

## Controlled-data lifetime domains (source prepared 31 October 2026; UNBUILT natively, UNRUN, UNREVIEWED)

**Why.** The measured checkpoint above covers ONLY what the retail headless descriptors gave: every sample size measured 0, so every natural sound entity started with lifetime 0 and expired with no handle on its first update. That
says nothing about the positive-lifetime aging, the sequence sum, the 40 s / 300 s cap, looping, or the invalid-effect countdown and unlink of the ORIGINAL code, all of which are active physical branches that a lifetime
extraction would otherwise have to leave behind silently. Nothing from the zero-size runs is claimed for them, and the existing probe, drivers and the three original controls are not rerun for this.

**What is added (test-only, observer build only; no production sound code, hook body or probe store changed).** `csrc/eech_env_fixture.c` group 14 indices 4 to 10, and the same driver `lua/env-sound-lifetime.lua`
(`case=4..10`). Index 0 (the plan and the natural report) and cases 1 to 3 are unchanged, so the natural run and the primitive controls stay comparable (the plan still says `cases=3`). Each case is PROCESS-ISOLATED, named as a
CONTROLLED-DATA case and distinct from the retail baseline. It sets the sample DESCRIPTOR numbers (`application_sound_effects[].rate` and `.size`, plain ints: no audio is allocated and no handle exists) to named
controlled values, builds the entity with the REAL constructor and its normal flags, and calls the REAL compiled update routine of the entity function table, only while the entity is a live sound effect AND a member of
the update list (the condition under which the original update loop calls it at all; an unlinked invalid entity is not called again). The descriptors, the frame delta and the tracking switch are restored BEFORE any row is
printed, so no return path leaves them changed (`descriptor_restored`, `delta_restored` in the end row). Nothing computes a lifetime: every word is read from the stored fields by `memcpy` (no float is loaded by value) and is
what the original did; the end state after each call is read from the entity slot only while it is still a sound effect with data (a killed entity's data is freed). The existing probe store supplies the creation, entry,
expiry and kill records; the fixture adds only the missing end-of-call state rows (`env-sound-state`) and, for the invalid cases, the pause accounting row. Constructor sample order, pitch and the shared RNG are untouched.

| index | label | controlled data | what is run |
|---|---|---|---|
| 4 | `data_positive_server_ages_and_expires` | lightning rate 4 size 20 | server routine, 3 calls of delta 2.0 |
| 5 | `data_positive_client_clamps` | the same | client routine, 3 calls of delta 2.0 |
| 6 | `data_sequence_accumulates` | two samples (rate 4 size 6; rate 2 size 5) | construction only |
| 7 | `data_caps_nonmusic_and_music` | rate 1 size 400 | an ordinary and a MUSIC sound, construction only |
| 8 | `data_looping_server_and_client_not_aged` | rate 4 size 20 | two LOOPING entities, one under each routine, 3 calls of delta 2.0 each |
| 9 | `data_invalid_server_countdown_and_unlink` | rate 4 size 20 | made invalid by the REAL `pause_local_entity_sound_type` (delay 1.5), server routine, 3 calls of delta 1.0 |
| 10 | `data_invalid_client_countdown_and_unlink` | the same | the same under the client routine |

The zero crossing of the invalid countdown and the unlink on the NEXT update are OBSERVED original results, not changed behavior. The real pause function walks the session's special-effect children: the fixture counts the
sound children (and those of the paused sub type) before calling it and prints the counts. The real pause changes ONLY a valid sound child of the paused sub type (it filters on the sub type), so the session's sound children of OTHER sub types are listed and untouched: the actual case-9 run showed 11 sound children, 1 of the paused type, 1 paused, and the first driver wrongly refused that as a lack of isolation (a witness bug, not a physical result; the C comment that says a fresh process holds exactly the control's own entity is superseded by this). The driver now requires exactly one child of the paused type and exactly one paused, a total that is at least the matching count, and the pause sub type to be the control's own entity's; another same-type target or a self-contradicting count is refused.
**The driver judges nothing physical.** It checks the three unsupported rows, the restoration flags, the descriptor/delta/pause rows, the state rows (tag order, the case's normal constructor
flags, a slot that reads as free after a kill), the probe (armed, nothing dropped, nothing untracked, as many tracked entities as built, as many updates as calls, a chain whose points belong to the routine, entries equal to the
calls of the entity, a kill exactly when the final slot is free), and writes every row as it came. It does NOT expect any lifetime, countdown, clamp, cap or unlink value.

**Bounded compiled-reference census of the sound-lifetime code (read from the compiled sources listed in `eech-campaign/crates/eech-engine-sys/sources.txt`, SHA-256 `d1aa1c0f0b7dd88c1483687915cbd53f8d94e9e495b88b0a11ac97d027dbca88`; nothing measured).** These are
files that CONTAIN an identifier, not independently listed or reached callsites, and nothing here infers that a file or a caller is dormant, headless-unreachable or irrelevant from a trace or from being compiled. The list is what a lifetime
extraction must disposition, group by group, and it is not a claim that the groups are exhaustive of all indirect routes:

* *Creators*: the creator identifiers (`create_client_server_sound_effect_entity`, `create_local_sound_effect_entity`) occur in **24** C files, all in `sources.txt`. One of the 24 is the helper's own `soundeff.c`, so **23 external files** contain a creator reference (the
  count of callsites is larger and is not enumerated here): `session/ss_updt.c`, `session/session.c` (lightning, measured above); `cdmusic.c` (music, looping or not, with the 300 s cap and pause/resume); `soundeff/speech.c` (the constructor's RADIO/CPG/WARNING amplitude and
  panning cases); `mobile/aircraft/` (`ac_msgs`, `fw_creat`, `fw_dstry`, `fw_move`, `fw_msgs`, `hc_creat`, `hc_dstry`, `hc_move`, `hc_msgs`, `helicop.c`: looping engine/rotor sounds and one-shots); `mobile/vehicle/` (`rv_creat`, `sh_creat`, `sh_msgs`);
  `mobile/weapon/` (`weapon.c`, `damage.c`); `special/effect/explosn/explosn.c`; `fixed/siteupdt/su_msgs.c`; the network creation path (`en_comms/en_comms.c`, `en_sessn.c`).
* *Pause and resume*: `pause_local_entity_sound_type` and `resume_local_entity_sound_type` EACH occur in the same **40** compiled C files that CONTAIN the identifier (not 40 callsites), all in `sources.txt`, including their implementation `soundeff.c` (so 39 external files; `pause_client_server_entity_sound_type` and its resume twin are
  in the same file; the pause variant TRANSMITS the type AND the delay over the network, the resume variant only the type). The 39 external files are in four groups, none of which may be dismissed: (1) `cdmusic.c`; (2) the aircraft/vehicle state code `entity/mobile/aircraft/fixwing/fw_anim.c`, `fw_int.c`, `helicop/helicop.c` and the
  vehicle interaction code `entity/mobile/vehicle/vh_int.c` (engine/rotor/APU sound types, delay 0.5 or 0); (3) the NETWORK receive path `entity/system/en_comms/en_comms.c`, which unpacks the sub type AND the delay from the wire for the pause (the delay becomes the valid lifetime of a remote
  entity) and only the sub type for the resume; (4) the gunship AVIONICS families under `gunships/avionics/` (11 families: ah64a, apache, blackhawk, comanche, default, havoc, hind, hokum, ka50, kiowa, viper), three files each (`*_avlmp`, `*_thrt` and `*_wpn`, or `*_cowpn` in place of `*_wpn` for comanche, kiowa and viper), 33 files in all,
  pausing and resuming avionics sound types (for example `ENTITY_SUB_TYPE_EFFECT_SOUND_MCA` with delay 0.5 in `ap_avlmp.c`).
* *Writers of the lifetime state*: the constructor (`soundeff.c` sample sum and cap), the two update routines (`se_updt.c`), pause/resume (`soundeff.c`), the destroy path (`se_dstry.c` sets `valid` false and zeroes the valid lifetime), the network pack/unpack of both lifetimes (`se_pack.c`), the per-vehicle
  sound updaters that set amplitude and pitch (`*_snd.c`), and the playback code that reads `effect_lifetime` (`se_snd.c`).
* *Resume is TWO separate state changes around a device-bound call.* `resume_local_entity_sound_type` sets `valid_sound_effect` TRUE, calls `play_local_entity_sound` (which needs the sound system and a device), and then re-inserts the entity into the UPDATE list if it is absent. The validity change and the update-list
  reinsertion are lifetime-state domains of the original and are NOT inherently unavailable because a device is: a missing playback handle makes only the play call unmeasurable. They are still UNMEASURED here, with the follow-up stated below; the play-call portion is the device-bound one.
* *Covered by a controlled-data case (source prepared, not yet run):* positive lifetime with server aging/expiry and client clamp, sequence accumulation, the 40 s / 300 s cap, looping not aged on either routine, the invalid-effect valid-lifetime countdown, zero crossing and update-list unlink on both routines.
* *Not covered, with the follow-up (none of these is deferred out of required D3; each needs its own disposition before a generic lifetime move):* (a) `resume` validity and update-list reinsertion state (a follow-up control that calls the real resume on a paused, unlinked entity and observes the state, accounting for the play call it makes), and the device-bound play portion (a device control with a real handle,
  playing and idle); (b) a playing or idle handle after expiry; (c) the independent destroy path (reached here only through the server's kill); (d) the network pack/unpack and transmission of lifetimes and the remote pause/resume receive path with a wire-supplied delay (real client/network delivery is not exercised: the client control is a server-created entity under the client routine); (e) the speech
  amplitude cases; (f) the per-vehicle amplitude/pitch updaters (a different responsibility from lifetime, to be dispositioned separately); (g) the avionics, vehicle and aircraft pause/resume callers as callers of (a) with their own delays; (h) a `valid=false` entity created that way from the start (it is never in the update list and is never updated).

**Two-entity prerequisite (cases 7 and 8).** The constructor takes the first free entity slot and dereferences the result, so EVERY constructor call needs a slot that exists. The fixture reads the free list before the case (`planned_entities` slots, refusal otherwise) AND before each constructor call (`env_snd_data_make` returns without calling the constructor when no slot is left); a control that finds itself short stops where it is, restores the descriptors, delta and tracking, and the end row says
`entities`, `planned_entities` and `short_of_slots=1` (a partial creation is reported, not hidden). Allocation order and the constructor itself are unchanged.

**Report coherence of the controlled-data cases.** The driver applies the same per-entity transition and update-pair rules as the primitive controls (one shared `walk_chain`), binds the typed rows to the case's declared inputs (the descriptors, the entities, the planned calls and the delta; the pause delay and sub type), allows a skipped call only after
an entity that is dead or unlinked, reconciles the kill and update totals, keeps stable slots and roles, and joins the independent captures of the same moment (constructor state to CREATED; each state before a call to that call's ENTER record, after the real pause the state straight after it; a client state after a call to that call's EXIT record) without calculating any lifetime.

**Constructor metadata and per-cohort call limits (review of 116ffd5fb).** Every probe record that reads the entity (all but a kill, which reads only its slot) must carry the declared constructor input: the sample of the first descriptor row, its controlled rate and size, the observed original rate, and the case's sequence count (2 for case 6, which builds a two-sample sequence; 1 otherwise). The canned positives
are generated from the declared descriptors, not from generic fallback numbers. Nothing is computed from these words: no lifetime or cap is derived or expected. Separately, each cohort is checked against the plan BEFORE any aggregate accounting: an entity that makes more calls than the case plans for it is refused, so a cohort's overcall can never cancel another cohort's legitimate skip in the
planned, made and skipped totals (a coherent 4+2 report in case 8 is refused). Early kills, early unlinks and a partial creation are still accepted.

**Actual WIP scope (my own, Linux container, no network, exclusive-new logs; fixture C compiled with `env-observer`):** see STATUS. The cases themselves have NOT run: the fixture needs a new native build and fresh roots, then the same reviewer on this delta only.

## Entry-point changes, stated

Source entry points changed (Root's feature-off check applies to the two production hook bodies only, as scoped above; the fixture and observer are test code): `aphavoc/source/entity/special/session/ss_envfx.h`, `.../effect/soundeff/soundeff.c`, `.../effect/soundeff/se_updt.c`,
`eech-campaign/crates/eech-engine-sys/csrc/eech_env_observer.c`, `.../csrc/eech_env_fixture.c`, `eech-campaign/crates/eech-world/lua/env-characterization.lua` (one optional argument and an appended block), plus the new
`lua/env-sound-lifetime.lua` and the guards in `tests/env_sound_lifetime.rs` and `tests/host_callbacks.rs`. `eech_engine.c` (one NUL byte) is **not** touched. The raw natural reports are unchanged when `sound=off`; the
retained observer and queue reports stay comparable (the new records are in a separate store, and no event, kind or sequence number was added to the stream). A new engine build is needed.

## Original probe recipe (now executed within the measured scope above)

`cargo fmt --check`; `cargo test -p eech-world --test env_sound_lifetime`; `cargo test -p eech-world --test host_callbacks sound` and `... environment` (each filter must report a NON-ZERO count: a filter that
selects nothing proves nothing); `cargo test -p eech-world --test env_lightning_queue`; clippy; build `eech-dc` with `env-observer`; the feature-off token check for the two production hook bodies (the default-feature
`cargo check` of `eech-engine`/`eech-dc` compiled the hooks away in the WIP check, which is not that check); then **one** natural fixed-Lebanon 120 s run with exactly the arguments of the retained fixed-Lebanon
observer control plus `queue=on sound=on` (the retained controls are not rerun), compared on every non-sound row with the retained summary (a difference is classified, never normalised); then the three controls, each in its
own fresh process and root. Private coordinates and raw outputs stay outside Git.

## Honest limits

* The natural run can only report what the retained-style configuration reaches. The previously recorded fixed run holds 2 lightning events and 4 sound creations, the variable Lebanon/Georgia runs 1 event and 2
  creations; those counts do not say what the creations produced or how long the entities lived, which is what this probe is for.
* The CLIENT routine is not reachable in the headless server process; it is observed only by control 2, on one entity built by the real constructor. The playback-handle branches (a handle that is playing, or is
  not) are NOT exercised and the plan says so: they need a sound-system device, no handle is faked and no backend callback is invented. The `valid = false` unlink path is not used by the lightning routine and is
  not covered. Audible output, network delivery and the player are not exercised or claimed.
* The probe does not decide what a host should do with a sound. What stays coupled for now: the generic entity heap, ids, list links and mirrors (the constructor is the engine's own), the sound-system and device
  state, and the campaign's time inputs. Campaign policy, scheduling and consequences stay in the Core.
* Remaining whole-D3 work: this characterisation run and comparison; moving the sound responsibility to the host only after it; the remaining time inputs; the required terrain, spatial and model services; the full
  shipping 40 and all 8 complete byte identities; current checks, review and the normal merges. D4 and D5 are unstarted. Choosing sound next does not close Task 51 or narrow D3. The probe, its drivers and its guards
  were originally prepared without runtime evidence; the checkpoint above supersedes that preparation status. No host API, schema or ABI is proposed.
