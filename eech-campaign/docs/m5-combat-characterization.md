# D2: original characterization of physical combat resolution (EEDCS-46)

**Status: characterization before extraction. The observer below is source-checked (syntax, strict feature-off preprocess equality, Clippy, tests). No native run of it has been made, so nothing here is yet a measured fact about the original; the source reading is labelled as reading.** Nothing is moved into the host by this document or by this commit: it adds a feature-gated, read-only observer and a scenario script, and describes where a seam can honestly be cut.

The ownership rule is the migration ledger's: the host owns the physical world and the physical execution of combat (what a hit does to a target's health, how far a blast reaches and how it falls off, when a target is destroyed); the campaign owns who fought whom and why, credit, loss accounting and consequences. The composition this task is built on is the reviewed Task45 runtime (`a137fc3ec`), composed as unaccepted development: Task45 has not been delivered, and nothing in the damage path was changed by it.

## What the source does (reading, with anchors)

| Step | Where | Kind |
|---|---|---|
| A weapon ends (hit target, hit land, overshot, self destruct); the explosion effect decides what is blasted | `special/effect/explosn/explosn.c` `~880-950` (kill-code switch) | physical decision in an effects file |
| A direct hit sends the target an `ENTITY_MESSAGE_COLLISION` with modifier `0.0` | `explosn.c ~936-946` | physical |
| Every living object (the six mobile kinds, seven fixed kinds, and weapons) inside `sqrt(capability)` of the blast, in sector order, except the direct-hit target, gets a collision with `modifier = range / radius`; a weapon in range may instead be destroyed (rocket, a threshold of the splash curve) | `damage_objects_within_explosion_damage_radius`, `explosn.c 973-1110` | physical |
| `weapon_damage_capability`: warhead figures, armour thickness, velocity ratio, one draw of the shared stream, armour penetration or high-explosive splash, the player and multiplayer modifiers; with no real weapon it is the no-draw forecast | `damage.c 338-486` | physical arithmetic (the no-weapon callers `wn_tgt.c 165-166, 435-439, 500-504` are campaign weapon-selection forecasts: the answer is physical, the selection is core) |
| `damage_client_server_entity`: damage sound effects (by weapon heap-index parity), player invulnerability, capability, subtract, clamp at 0, write `INT_TYPE_DAMAGE_LEVEL`; the debug-cheat guard changes whether it is written | `damage.c 97-244` | physical apply, with sound coupling before it |
| The thirteen collision handlers: damage, read the level, a critical-damage predicate for credit (aircraft and vehicles), destruction at level <= 0, force-destroyed notifications, speech, player logs, keysite under-attack | `hc/fw` (`*_msgs.c`), `rv/aa/ps/sh`, and fixed `st/su/ob/sg/cb/sn/py` | the health and fatal FACTS are physical; credit, force notifications, speech, logs and the under-attack notification are campaign consequences, interleaved |
| `mb_msgs.c ~1466`: repair restores the initial damage level | | the other health writer; repair is D4 coupling, observed and classified, not removed |

Two things the reading found and the observer now measures: (1) a weapon that hits land runs the blast **twice** (the crater branch calls it, then falls through into the case that calls it again), so a land hit blasts twice with the same capability; (2) the blast skips the direct-hit target only on the hit-target path. The observer's `repeats_same_weapon` counter says whether either occurs in the scenarios.

## The observer (feature `weapon-fixture`, never shipping)

`entity/mobile/mb_cmbfx.h` hooks (empty without the feature) in `damage.c`, `explosn.c`, `mb_msgs.c` and `en_msgs.c`, and `csrc/eech_cmb_fixture.c`. Every hook only reads. The thirteen collision handlers are wrapped once, when the message responses are initialised, by one wrapper that reads the receiver, runs the original on the same arguments (the modifier read from a copy of the argument list) and reads the receiver again; `notify_local_entity` (which the `eech-sys` crate extracts verbatim) is untouched. An entity the handler destroyed is reported as destroyed and read for nothing else. The shared random stream is observed by the number of draws only.

It reports four ordered hashes (`damage`, `collision`, `blast`, `repair`), counters per entity kind (collisions, direct versus blast, applied, no damage, invulnerable, fatal, destroyed, already dead, damage sum and maximum, blast candidates, in range, notified, rocket kills), capability counters (real weapon versus forecast, armour-penetration versus splash versus none, draws, player targets, multiplier moved the result), per-warhead rows, blast calls by kill code, and an ordered event log of every collision, blast and repair. Through `engine:cmb_trace ()` and `engine:cmb_events ()` of the fixture module; `lua/combat-characterization.lua` runs the campaign through the public host path and writes the checkpoints (default every 60 s) and the final report as JSON lines.

## What is checked so far (source only)

* C syntax with and without the fixture macro: the four edited sources, the observer and the engine glue compile (`checks/syntax-check-46-*.log`; a subset of the engine, `init.c` and the other callers are not in it).
* Strict preprocess equality: with the macro off the four edited sources preprocess identically to `HEAD` (`checks/strict-preprocess-46-*.log`); the control with the macro on differs for all four.
* Default and fixture Clippy, and the engine, module and world tests: actual Cargo and Docker 0.

## What this does not yet show

* No native result. The natural 600-second run (`lua/combat-characterization.lua`) will say which of the thirteen collision kinds, which warheads and which blast cases the campaign itself reaches; whatever it does not reach needs a controlled witness (the Task45 controlled-launch case can launch a weapon at a host-identified target; a blast near a fixed object needs its own), and an unreached path stays declared as unreached, not covered.
* Player-targeted damage (invulnerability, difficulty modifier) and the multiplayer multiplier are reached only if a scenario has a player target; the observer counts them.
* The sound effects created inside `damage_client_server_entity` and the heap-index parity that picks them are not measured; whether creating them draws from the shared stream has not been read or measured. The observer counts the draws `weapon_damage_capability` itself takes, so it does not claim to see a draw made elsewhere in the damage path.
* No seam is chosen here. A seam proposal comes after the natural and controlled counters say what the campaign actually exercises.
