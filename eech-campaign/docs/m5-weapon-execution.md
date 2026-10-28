# D2: host-owned physical weapon execution (EEDCS-45)

**Status: implemented in source on `codex/d2-weapon-execution` (PR104, draft). The table-generation cut (stage 1) has native evidence equal to the retained originals (a complete 600-second trace and the controlled launch witness). The flight, contact, terminal, launch and release cuts (stages 2 to 4) have unit, contract and public-Lua tests and compile/Clippy results but, until the integrated candidate has run natively against the retained originals, NO equivalence with the original behaviour is claimed for them.** Evidence and results are recorded in [`reference/d2-weapon-characterization/`](../reference/d2-weapon-characterization/README.md) and the PR; this document records the design.

The behaviour being preserved is characterised in [`m5-weapon-characterization.md`](m5-weapon-characterization.md). The ownership rule is the migration ledger's: the host owns the physical world and the physical execution of a weapon; the campaign owns what to fire at, when, with what, the ammunition it accounts, and what follows.

## What moved

| Physical responsibility | Before (original C) | Now |
|---|---|---|
| The ballistic tables and the guided range runs | `generate_ballistics_tables` / `generate_guided_missiles_tables` simulated in the campaign | the host simulates from the weapon's figures and keeps the tables (`flight.rs`, `csrc/wn_flight.c`); stage 1 |
| A weapon's physical figures | read from the weapon database wherever needed | handed over once at boot (`CAMPAIGN_AIM_FIGURES`); the host flies, launches and releases by them |
| The release gate (rate of fire) | burst size and refusal computed in `create_client_server_entity_weapon` | the host decides the rounds or a refusal (`CAMPAIGN_AIM_RELEASE`) |
| The launch kinematics | velocity, lifetime, fuze timer, dispersion in `create_local` | the host's birth (`CAMPAIGN_AIM_WEAPON_BIRTH`) |
| A weapon's frame of flight | `weapon_movement`: lifetime, guidance, tracking, intercept, the guided and unguided step, contact, bounds, ground, kill code | the host's step (`CAMPAIGN_AIM_WEAPON_STEP`) |
| The weapon-specific collision algorithms | `collision_test_weapon_with_*` in `wn_cllsn.c` | removed: the host's step decides the contact from generic segment queries |

## What stays the campaign's

Which target, weapon and package; the decision to fire; the ammunition count and what happens when it runs out (`current_weapon_count`, `new_weapon_count`, reloads, speech); the launcher's burst timer (it gates the launch sound, not the physics); the FIRED_AT notification; sound, smoke trails and effects; the launcher's pylon geometry and the accounting of what it carries (`detach_local_entity_weapon`, run before the host's birth); the entity links; Tacview recording; combat damage (the next D2 task). The campaign applies the host's physical result and never chooses it.

## Declared coupling (not D2 ownership)

Generic terrain and object geometry services stay behind the campaign's lookups, declared as D3 coupling: whether a point is below the ground, the terrain class and impact point of a segment, the face normal and elevation at a point, the first object or tree a segment meets (`get_line_of_sight_collision_entity`/`_tree`), the launcher's forward velocity and tracking point, a target's position, point, motion vector and flags, the session wind. General effect creation (the smoke-grenade killed effect), sound, and the lead prediction a remote fuze is set from (already host-owned aiming) are asked of the campaign too. These are data and services; the weapon-specific decisions made from them are the host's.

## The protocol

Every request to the host is a **pure question**: the host keeps nothing between requests except the figures and tables it was given. A step or a birth that needs the campaign's world stops and says what it needs; the campaign answers and asks again.

* **Lookups.** A step answers `need_lookup` with up to 16 lookups (a kind, small integers, up to six floats). The campaign answers each one **by value** (the lookup repeated with its result in `f[6..11]`) and asks again with every answer so far; the step matches an answer by its arguments, never by position. A step that has not finished leaves no action. The host batches the lookups it can already name; a host may ask a lookup speculatively and not use the answer. A lookup the host asks again after it was answered is a host error.
* **Random samples.** A step that needs a draw of the campaign's **shared** stream answers `need_draw`; the campaign takes one raw `rand16 ()` sample and asks again with every sample so far. The host consumes them in order, so the stream is advanced exactly as the original advanced it. The **entity-local** stream (a seed that is a heap index in the original) never leaves the campaign: the request carries the first four outputs of a copy of it, which is all any one call of the original uses.
* **Actions.** A finished step answers with an ordered list the campaign applies to its mirror of the weapon in the order the original wrote and called them: the state (kill code, phase, on-ground, velocity, motion vector, lifetime, decoy timer), the attitude, the position, a change of the target link, the submunitions spawned, the Tacview update, the ground effect. The state is written before anything that looks at the weapon, so a partial write of the original (a velocity bound, a lifetime decrement before an early return) is a write here too.
* **Echo (fixture build only).** The test fixture build asks the host for the hooks of the original it passed, in order; the observer replays them into the same hooks. Shipping builds never ask.

Validation is at every layer: C checks the request and the answer (`mb_hostaim.c`), Rust checks sizes, ranges and finiteness before and after the kernel, and an unsupported, malformed or missing physical answer stops the engine with a message; nothing is defaulted.

## What is preserved, and how it is known

* **Arithmetic and the FPU.** The kernel (`csrc/wn_flight.c`) copies the original expressions, with only the data access changed (the database becomes the figures, the entity the request); it is compiled with the same C semantics as the engine and runs under the same rounding mode the engine runs host callbacks in. Mixed float and double expressions are kept as written.
* **Random streams.** The shared stream is consumed by the host's draws in the original order (the muzzle error and the cruise-time error use the entity-local samples; the spiral, the fuze jitter and the dispersion use shared samples). The fixture counts draws per update and per birth.
* **Update order, partial writes and early returns.** The order of the original's checks (fuze timer, expiry, resting, guidance, movement, contact, bounds, ground) is the kernel's; every path ends by writing the state the original had written.
* **Unsupported paths fail rather than invent.** A guidance that asks for a launcher the weapon does not have (the original would dereference it) is a step error here, not a default.

What was **not** measured natively for the original and is therefore only source-reviewed: player-launched laser and wire guidance (the retained campaigns launch from AI), the Hellfire lock-on-after-launch dive, and weapons with a submunition fuze that the retained scenarios did not reach. The coverage of each retained scenario is stated with its evidence.

## Substitutability (the test of the boundary)

A host with a model of its own answers the same requests and needs none of EECH's tables, figures or diagnostics: `lua/aim-provider.lua` `mode=own_model` flies weapons straight for twenty seconds, ends them on the ground it meets, lets one round go at every request, and launches at a fixed speed, asking the campaign one lookup (the ground under the weapon) and nothing else. Other modes change only what the host is told (a weapon's drag, cruise time, muzzle velocity or rate of fire) and show that the campaign's weapons then fly, end, launch and release differently. Malformed answers (`bad_step_*`, `bad_birth_action`, `bad_release_burst`) stop the engine with a clear error.

## Tests

* Unit tests of the kernel through the Rust driver (`aim.rs`): a free round for a frame, the lookups it asks and does not ask, atomic stops, ground, water, rest, target contact, the map edge, expiry, the spiral's samples in order, a semi-active weapon without a launcher, the launch velocity and error, the dispersion samples, the fuze set from the lead prediction, the rate-of-fire gate including a write kept when the gate then refuses.
* Contract tests of every layer (`eech-engine`, `eech-dc`): the request and the answer cross the boundary exactly, and what the campaign cannot hold never reaches the host.
* Public Lua tests (`tests/host_callbacks.rs`): the host's modes through `host.aim`, and that every shipped Lua script parses.
* Native fixture comparison (the original behaviour): see the PR and `reference/d2-weapon-characterization/`.
