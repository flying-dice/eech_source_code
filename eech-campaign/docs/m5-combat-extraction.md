# D2: physical combat resolution extraction (EEDCS-46), step 1: the damage FORECAST

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
