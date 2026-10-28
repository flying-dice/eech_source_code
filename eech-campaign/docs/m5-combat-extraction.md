# D2: physical combat resolution extraction (EEDCS-46): step 1, the damage FORECAST; step 2, the damage of a hit (below)

Status: first candidate moved, NOT accepted. Only the forecast is moved; real weapon damage, the health application, the critical predicate, the timed rolls, the explosion and
blast walk and the bridge collapse are the original campaign code, unchanged, and each stays behind its own characterization and coverage gate (see
`m5-combat-characterization.md`). Nothing here claims that combat resolution has moved.

## What moved

`weapon_damage_capability (NULL, target, damage_modifier, weapon_type, velocity)` in `aphavoc/source/entity/mobile/weapon/damage.c`: the damage a weapon is CAPABLE of against a
target with no weapon in flight. The target selection asks it (`wn_tgt.c`, six call sites, always with a modifier of 0.0 and a velocity of 1.0) to decide whether a package can
hurt a target at all (a zero is "not capable") and to weigh packages against each other.

The physical arithmetic now runs in the HOST: `eech-world/csrc/cmb_step.c` (EECH's own expressions, moved verbatim: the conventional-munitions relationship, the target's armour
as its table level, the pierce test, the two high-explosive branches with their `pow` and `min`, the integer truncations, and the player's damage setting applied last), driven by
`eech-world/src/cmb.rs` and answered through the existing aim step protocol (`CAMPAIGN_AIM_FORECAST`, `mb_hostaim.h`). It executes OUTSIDE the DLL; the DLL does not call a private
solver and has no fallback (no host aim callback stops the engine at the first request, as for every aim request).

## The contract (by value)

Sent by the campaign (`campaign_aim_forecast`): the weapon sub type; its damage figures (`campaign_damage_figures`: the warhead type of its own definition, and the warhead,
damage, armour penetration and burst duration the forecast is made from); the target's armour level (a table value, never negative); whether the target is not an AI and, then, the
player's damage setting; and the two arguments of the original function. Answered by the host: the capability, an integer, never negative. No pointer, entity index, list layout,
seed or database array layout crosses, and no target identity is sent at all (a forecast needs none, so a fixed structure, which has no roster identity, is forecast like any
other target).

* **Conventional munitions.** A conventional-munitions weapon carries its damage in the munition it scatters, the next entry of the weapon database (the same relationship
  `wn_creat.c` relies on). The campaign resolves that relationship and sends the munition's figures; the host applies the arithmetic (the munition's damage scaled by its burst
  duration over three). An entry with no munition after it stops the campaign with a clear message (the original read past the table).
* **No random stream.** A forecast takes no sample of the shared stream, as the original took none: a host that asks for a draw or a lookup is refused (C, Rust and Lua layers).
* **Refusal, never a default.** A missing host, a failing host, a state that is not done, a negative capability, an echo that is not exactly one forecast event agreeing with the
  answer, a Lua reply whose capability is not an integer, and a request with a non-finite figure, an armour level below zero or a non-boolean flag each stop the engine with a clear
  message before the answer is used. The reference kernel also refuses an intermediate value the original could not convert to an integer (a non-finite or out-of-range product;
  the original's result there is undefined), for example a splash against armour 0 with no damage figure (0 / 0).
* **Who keeps what.** The campaign keeps which weapons and targets are weighed, the salvo and rate-of-fire multiplier and the comparison (`wn_tgt.c`), the reading of the target's
  armour level and player setting, the player-setting table (`get_player_damage_modifier`: difficulty to setting, including its fatal error for an unknown difficulty), and the
  definition relationship above. The REAL weapon's capability (its draw, its velocity, the aspect armour of the target, the multiplayer multiplier and the debug display) is not part of
  this request and is unchanged in the campaign.

## Declared couplings that remain

* The player-setting table and the conventional-munitions definition relationship are campaign data, sent as values.
* The fixture observer (`CMB_FX_*`, test build only): the reference host echoes the original's hooks of the call in ONE event (`CAMPAIGN_AIM_EVENT_FORECAST`: the path, the capability
  before the player setting, the warhead used, the final capability; the modifier, penetration, armour thickness and random modifier as floats) and the fixture replays them into the
  observer in the order the original made them (begin, path, raw, end), so the capability accounting, the damage hash and the ordered event log are produced by the same hooks. A host
  that echoes nothing is not observed (its model is its own); an echo that is not exactly one agreeing event is refused.
* R1 (the other readers of the critical predicate), R2 (the handlers' early returns), R3 (fixed structures have no roster identity) and R5 (collapse physics against shape data) are
  not touched by the forecast and stay open for the modes that need them; the forecast needs no identity, so R3 does not arise here.
* R4 (cost): the natural 600-second run made 574,382 forecasts, about 960 requests per simulated second, each a pure question carrying about a hundred bytes of figures. The frame cost
  of the exchange is a ROOT-owned measurement with the existing M5 risk metrics, not claimed here.

## Evidence and gates

Source checks (no native run, no build of the fixture or of the shipping module): the kernel's own tests against values worked out by hand from the original expressions
(`eech-world/src/cmb.rs`: pierce, no pierce, the velocity argument, the four high-explosive warheads against two armours, conventional munitions, the player's setting and its
truncation, and the refusals); the reference host's public contract through `host.aim` and the plain model of a host with another model (`tests/host_callbacks.rs`,
`lua/own-weapon.lua`); the engine boundary (`eech-engine` request and reply translation, refusal of a draw, a lookup and a negative capability); a layout test (`csrc/eech_aim_layout.c` reports the sizes and the offsets of the aim exchange as the campaign compiled them, and the engine test compares the hand-written Rust mirrors with them, because the structures were extended on both sides), and 25 checks of the REAL
`mb_hostaim.c` (`c-checks/forecast_checks.c`, `run-forecast-checks.sh`) for what the campaign sends, that a changed host's capability is used as given, that no sample is taken and
that missing, failing and malformed hosts and echoes stop the engine. `lua/aim-provider.lua` gains the changed-host modes `forecast_armoured` and `forecast_zero` and the malformed
modes `bad_forecast_negative`, `bad_forecast_draw` and `bad_forecast_fraction` for the ROOT-owned runtime gates.

ROOT-owned runtime gates (nothing was launched for this change): the default native candidate must reproduce the retained original physical measurements of the f41 natural 600
run (the six hashes, the counters, the 574,382 forecasts and the ordered event log; the legacy string-typed `blast` counter of that raw is a known schema-only difference and the raw
originals stay immutable) and the 900-second control; then the default shipping full 40, all 8 and post 3 and current CI. Task45 delivery precedes Task46 delivery.

# Step 2: the damage of a hit (a real weapon)

Status: candidate moved, NOT accepted. The physical result of a hit that reaches a damage handler is the host's; the timed health rolls, the explosion and its blast walk, the
bridge collapse, the other readers of the critical predicate and one handler gate (below) are still the original campaign code and stay behind their own gates. Nothing here claims
that combat resolution has moved.

## What moved

`damage_client_server_entity` and the real-weapon half of `weapon_damage_capability` (`damage.c`), with `get_local_entity_armour_thickness` (`wn_cllsn.c`, deleted from the campaign) and
the critical-state classification after a hit. One request of the aim step protocol, `CAMPAIGN_AIM_DAMAGE` (`mb_hostaim.h`), answered by `eech-world/csrc/cmb_step.c` (`cmb.rs`):

* whether the player's invulnerability stops the hit (`INT_TYPE_INVULNERABLE_FROM_WEAPONS`, a player's setting): nothing is computed and no sample is taken;
* the capability of the real weapon: the conventional-munitions relationship, the random modifier from the campaign's ONE sample (`frand1 ()`), the two high-explosive branches, the
  weapon's speed over its own cruise velocity for a solid shot, the penetration test, the integer truncations, the player's damage setting and the multiplayer multiplier last;
* the armour the weapon meets: the easiest-difficulty armour (0.1), the table level, and for a vehicle or an aircraft the aspect armour from the impact vector in the target's frame
  (the original's matrix product, normalisation and face weights, with a vehicle's three figures sent from its definition and an aircraft's fixed ones);
* whether the capability is applied to the health (a capability above zero and no cheat guard) and the health it leaves (the subtraction and its clamp at zero);
* the critical classes before and after (`vehicle_critically_damaged`: critical below 15 percent, never a person; `aircraft_critically_damaged`: mortal below 30, critical below 5; the
  original's integer percentage), none for a fixed structure.

The reference kernel executes OUTSIDE the DLL with the original's expressions; no host callback calls a private solver in the DLL; there is no fallback.

## The contract (by value)

The campaign sends the weapon's damage figures (as the forecast's), its speed and its own definition's cruise velocity, the blast fraction, the armour data (table level, the
launcher-is-an-easy-player flag, the model, a vehicle's three figures, the weapon's and the target's positions and the target's attitude matrix, read only for a vehicle or an
aircraft), the settings (whether the target is not an AI and the player's damage setting, or that none could be resolved; whether the weapon has a launcher and the multiplayer
multiplier; the player's invulnerability; the cheat guard) and the health (level, initial level and the classification model). No pointer, entity index, identity, seed or
database layout crosses. The host answers NEED_DRAW once (after the invulnerability test), is given the raw integer in [0, 32767] and answers: exited, capability, written, new
level, the two classes, and whether a damage setting it needed was missing.

The campaign refuses, before anything is applied: a second sample; a sample for a hit that invulnerability stops; a hit on an invulnerable player, or a stop of a target that is not
invulnerable; a stopped hit that still changed something; health written under the cheat guard, below zero or above what it was; health or a class changed without being written; a
class the target's model cannot have (a person and a fixed structure have none, a vehicle 0 or 1, an aircraft 0 to 2); a missing-setting report that the request did not allow; an
echo that is not exactly one agreeing event; a lookup; a failing, absent or malformed host; and a request with a non-finite figure, a model that does not exist or a classified
health with no initial level. It does NOT refuse a capability of zero or less (the original's arithmetic can give one, and such a hit does nothing) or an armour level of any sign; but it DOES refuse any write of health
for such a hit, because the original writes health only for a capability above zero (a hit that does nothing may not kill). An unwritten zero or negative answer is the valid no-op.
A host with a model of its own answers from it (`lua/own-weapon.lua` is a plain one: ten points, no sample, the guards honoured) and needs none of EECH's arithmetic.

## Order, and what the campaign keeps

The sounds of a hit to a player come first, as they did (they create entities and are the campaign's), then the request, then the setting of the health through the campaign's
setter with its side effects, then the handler's consequences. The relative order of the sounds, the one random sample and the setter is the original's, and so is the number of
samples (one per hit, none for an invulnerable player). The campaign keeps the sounds, the debug display of a hit, the player-setting table and its fatal error (raised if the host
needed a setting that was missing, as the original raised it when it got that far), the multiplayer multiplier's record, the setting of the health, and in the handlers the credit,
the speech, the force notifications and the kill. The handlers of the aircraft, anti-aircraft, person, routed and ship vehicles read the two classes from the host's answer
(`damage_client_server_entity_assess`) instead of calling the predicates around the hit; a hit that was not asked of the host (no weapon, or stopped by invulnerability) changed
nothing and gives equal classes, so no handler sees a transition.

## R1, R2, R3 from the source

* **R2, the 13 collision handlers.** Twelve (the seven fixed kinds, the fixed-wing aircraft and the four vehicle kinds) go straight into `damage_client_server_entity` with no
  physical gate of their own. One has a gate before it: the helicopter ignores a hit by itself when it is an AI (`hc_msgs.c`, "fix to stop helicopters from killing themselves through
  firing missiles into the ground"). That gate is physical eligibility and is NOT moved in this phase (it precedes the damage call, and moving it needs the request to be asked
  before the fixture's damage hooks begin, which the retained original does not do); it is the next bounded step and is not relabelled campaign policy. The ship handler's smoke on a
  strong high-explosive direct hit (`damage_caused > 100`, a roll of the weapon's own seed) is presentation.
* **R1, the other readers of the critical predicate.** Six kinds of reader remain on the campaign's own predicates, which read the stored health: the timed fire and bleeding rolls
  (`hc_updt`, `fw_updt`, `aa_updt`, `rv_updt`, `sh_updt`, `ps_updt`: physical eligibility and the aircraft class's power in the probability, and they draw), the per-frame gate that
  stops updating a critically damaged vehicle (`aa_updt`, `ps_updt`, `rv_updt`, `sh_updt`: a physical consequence of the state, mobility), the helicopter's critical-damage branch
  (`hc_updt`, two sites: whether a crippled helicopter keeps flying or lands), the aircraft's damaged smoke (`aircraft.c`: presentation) and the victim's speech (`aircraft.c`:
  presentation). The timed rolls are the next phase and will take the host's class; the mobility consequences are declared coupled until then, NOT classified as policy to close the
  audit. The predicate functions therefore still exist in the DLL for these readers, as the stored-level reading they always were; what moved is the classification that follows a hit.
* **R3, fixed structures.** A hit on a fixed structure needs no identity: the calculation is pure and by value (level, armour level, the weapon and the settings), and the host answers
  the capability, whether it is written and the level, which the campaign sets and, at zero, kills. So the physical damage of a fixed structure is the host's without a private heap
  index and without declaring static combat a D1 limit. A host that wants to OWN a structure's state would need an identity the roster does not give a fixed structure; that is not
  needed by this exchange and is recorded as a limit for such a host.

## Evidence, and what is not available

Source checks (nothing launched or built natively): the kernel against values worked out by hand from the original expressions (`eech-world/src/cmb.rs`: the sample and the random
modifier, high-explosive and solid-shot speed scaling, conventional munitions, the aspect armour on each axis and a diagonal and a turned target for a vehicle and an aircraft, the easy
armour, the invulnerability and cheat guard, the clamp, the classes at their boundaries and in the original's integer arithmetic, a person and a fixed structure, the player's setting
and the multiplier, the missing setting); the reference host's public contract through `host.aim`, a plain own-model host and the malformed providers (`tests/host_callbacks.rs`); the
engine boundary and the layout of the extended structures (`eech-engine`); and 50 checks of the REAL `mb_hostaim.c` (`c-checks/damage_checks.c`, `run-damage-checks.sh`: what is sent,
the one sample, a changed host, every refusal above; removing the cheat-guard, invulnerability-consistency and second-sample refusals fails 4 of them). The fixture build replays the original observer hooks of the capability from one echoed event
in their original order (the hooks begin before the host takes the sample, as the original's did), so the capability accounting, the damage hash and the ordered log come from the same
hooks; an independent host needs to emit nothing EECH-private. `lua/aim-provider.lua` gains `damage_armoured`, `damage_double` and `damage_none` (only real damage changes: the forecasts and
every other mode are the reference's) and `bad_damage_level`, `bad_damage_draw_twice`, `bad_damage_exit`, `bad_damage_class`, `bad_damage_unwritten`, `bad_damage_missing` for the
ROOT-owned native authority and refusal runs, which report the hits, samples, losses and criticals.

Not available and not claimed: any native run of this candidate. The original characterization it must reproduce is the retained one: the f41 natural 600 (234 real capability draws,
234 balanced damage entries and exits, six hashes, 6,440 ordered events) and the 900 control, the 9d7 and 35c3 controlled runs (the HE writes, the PERSON writes, the solid-shot
capability calls without writes, the weaponless exits) and the two SITE probes. The player's own damage, invulnerability, the multiplayer multiplier and
the easy-difficulty armour were not reached by any retained original run (the retained natural 600 DID record five fatal anti-aircraft COLLISIONS, which are covered; what no retained run reached is the fatal anti-aircraft TIMED FIRE roll, which is a later phase, and the controlled anti-aircraft cases drew 201 times with 41 or 46 hits and no timed fatal); they are covered here by the source arithmetic and the contract checks only.

## Correction of the first review of the damage candidate (`dc3113e4b`)

* **A nonpositive capability may not write health (P2).** `mb_hostaim.c` now refuses a DONE answer that writes health for a capability of zero or less (a zero capability with a write to level
  0 had passed the range checks and the handler would have killed a healthy target). Unwritten zero and negative answers stay legal. The production C guard has the cases both ways (RED
  before the fix: four refusals missing; GREEN after).
* **The diagnostic echo carries a legitimate not-a-number (P2).** A weapon at the target's own position has no direction: its aspect armour is a not-a-number, the original's comparison with it is
  false and the hit does nothing (capability 0, nothing written, kernel outcome unchanged). The fixture build asks for the echo of the original's observer values, and the exchange validates
  echo floats as finite, so that legitimate diagnostic became fatal. The echo of a hit now carries each of the four intermediate floats (the damage modifier as it ends, the penetration, the armour
  thickness, the random modifier) as its RAW 32-bit pattern, as two exact 16-bit halves in `f[4..11]` (sign, payload and order preserved; `f[0..3]` repeat the finite ones for a reader and are 0
  otherwise). Validation of the physical request and answer is not weakened: a float that is not finite is still refused everywhere, and the halves must be integers in [0, 65535]. The campaign
  rebuilds the floats with `campaign_aim_damage_echo_float` and the fixture observer folds the exact bits, so the damage hash and accounting of a centred hit are the original's. An independent
  host sends no echo at all. No layout changed (the event already had twelve floats); the only public representation change is this documented use of `f[4..11]` of the DAMAGE echo event.
  Checked at each hop with the production code: the reference kernel and its Lua reply (`tests/host_callbacks.rs`, RED before the fix with `echo value 3 ... -nan`), the DC module reader and the engine's
  copy into the campaign's room (`eech-dc`, `eech-engine` tests), and the production `mb_hostaim.c` validation and decoder (`c-checks/damage_checks.c`, every bit pattern that matters). The hops are
  not driven in one process: that needs a native run, which is not claimed.
* **Coverage wording (P3).** Corrected above: the unreached anti-aircraft case is the fatal TIMED FIRE roll, not every fatal hit.
