# D2 bridge and segment collapse (R5): the test-only characterisation of the ORIGINAL behaviour (EEDCS-46)

Status: **RUN ON THE ORIGINAL, CLOSED** (the root ran it twice: native `original99734` / PID 10104 and `repeat81402` / PID 17440, both exit 0, 320,598 bytes, identical SHA `d4c198af...`, 118 bridges surveyed, bridge 0 of 8 segments; the bridge physics has since moved to the host, `m5-combat-extraction.md` Step 5, and this same script is the comparison). This is a test-only fixture for the ORIGINAL bridge and segment collapse. The fixture itself moves and changes no bridge physics, adds no host request and no persistent-structure or report API, and had
no native run or build when it was written; the root built and ran it on the original and accepted its actual behaviour before any bridge rule was extracted. The surroundings it runs in are the current code (the combat callbacks already
moved are existing surroundings); the later old/new comparison must share that same source and configuration.

## The original code under observation (read, not changed)

| piece | where | what it is |
|---|---|---|
| the structural failure of a segment | `fixed/segment/sg_dstry.c kill_local` | alive set false; the object-killed explosion effect; `kill_routed_vehicles_on_segment` (routed vehicles within the segment's length times root two, in its sector, die); then the bridge is sent a collision (the sender is the killed segment) and then child-killed |
| the bridge's collision response | `special/bridge/br_msgs.c response_to_collision` | a sender that is an UNSUPPORTED mid section and alive is killed (server) with its explosion effect: **the unsupported-span kill**; then the sender's shape is chosen from its neighbours (`get_bridge_segment_damaged_object`) |
| the shape choice | `fixed/segment/sg_dbase.c get_bridge_segment_damaged_object` | a rule over the segment's and its neighbours' alive flags (a missing neighbour counts as alive), selecting one of five shapes of the segment database (**data**: normal, destroyed, predecessor-, successor- and both-destroyed) |
| the bridge's child-killed response | `br_msgs.c response_to_child_killed` | only while the bridge itself is alive: it is marked not alive, the route link's break count is incremented, the vehicles on the route are stopped, a repair task is asked for (**campaign actions**); in every case the killed segment is put in the update list |
| the update of a segment in the list | `sg_updt.c update_server` | each frame: the successor and then the predecessor are notified (collision to the bridge with that neighbour as sender: the propagation); an unsupported segment drops `10 * dt` (a water floor is one metre lower) until it reaches the terrain, then the collision effect and it leaves the list; a supported one leaves the list at once |

Physical rules: the unsupported-span kill, the neighbour propagation, the shape choice, the fall, the kill of vehicles on the segment. Data: the segment database (shapes, lengths, damage levels, sub types) and the
bridge database's patterns. Campaign actions, which stay core: the route link break, the stopping of vehicles, the repair task, credit, awareness and accounting. Nothing here is deferred without a reason:
the physical rules above are the next extraction once this behaviour is accepted. Persistent structure ownership and report correlation (R3) and the physical factual reports are next D2 work and are not
touched by this fixture.

## The fixture

* **Entry**: `eech_engine_cmb_case_bridge (op, ordinal, index, criterion, text, capacity, host)` (`csrc/eech_engine.c`, in the `EECH_WEAPON_FIXTURE` section), implemented by `eech_cmb_case_bridge_run` in `csrc/eech_cmb_case.c`;
  Rust `Engine::cmb_case_bridge` and the Lua method `engine:cmb_case_bridge (op, ordinal, index, criterion)`, all behind the `weapon-fixture` feature. It takes the **complete host** as a frame does (the
  failure runs the campaign's kill, explosion effects and consequences), the missing-host guard is unchanged, and no pointer, heap index, layout or seed crosses: a bridge is its ordinal in the entity list, a
  segment its index along the bridge.
* **Operations**: 0 survey (every bridge: sub type, nodes, alive, segments, a pattern of `E` end, `S` supported mid, `U` unsupported mid, the first segment meeting the criterion, the world's counts of routed
  vehicles alive and repair tasks); 1 state (the bridge and each segment: sub type, type, alive, damage, shape, position, whether in the update list, the neighbours' alive flags, the route link's break count); 2 fail
  (the declared structural failure); 3 disarm.
* **The precondition is declared, not a weapon hit.** Operation 2 calls the campaign's own `kill_client_server_entity` on one segment, chosen by an explicit test-only criterion applied inside C
  (criterion 0, the FIRST-COLLAPSE preconditions: the bridge is alive, and the segment is alive, is not an unsupported mid section and is next to an unsupported mid section that is ALIVE) or by index. It is
  refused as NOT DONE, with the reason, if there is no such segment, the segment is not alive or the campaign is not a server. The facts the original's first-collapse branches depend on are REPORTED, read
  before the failure (the survey: `alive`, `first_candidate`, `first_neighbour`; the failure: `bridge_alive`, `neighbour`, `neighbour_alive`, `first_collapse`), and an explicit index that does not meet them is applied
  all the same and reported `first_collapse=0`.
* **The observer**: read-only `CMB_FX_BRIDGE` hooks (`mb_cmbfx.h`) in `br_msgs.c` and `sg_updt.c` record, in the original's own order, the segment update, the collision with its sender's sub type and alive flag, the
  unsupported-span kill, the shape chosen with the neighbours' state, the child-killed, the link break and its count, the repair task, the update-list insertion, each fall step with the terrain height compared, the
  landing and the removal of a supported segment, as ordered `cmb-ev ... kind=bridge-*` lines (bridge ordinal, segment index and count, sub type, alive, shape, the segment's height, two integers and a float). The
  hooks only read; they are armed only by operation 2, so outside the case they return before reading anything, record no event and take no sequence number: no natural run's report, hash or event log can change.
* **Gating**: the C driver and the observer are inside `#ifdef EECH_WEAPON_FIXTURE`, the Rust methods behind `cfg(feature = "weapon-fixture")`, and the hooks expand to nothing without the fixture. The ordinary view
  of the original sources is proven identical: the feature-off preprocessed text of `br_msgs.c`, `sg_updt.c` and (as a control) `sg_dstry.c` equals the baseline's byte for byte
  (`checks/strict-preprocess-bridge-1.log`).
* **The witness script** `lua/bridge-case.lua`: boot from the verified retail root, `warmup` seconds of the war, the survey, the choice of the first bridge that meets the criterion (or `bridge=`), the state, the
  failure, the state immediately after, then `window_seconds` of frames, each followed by the state and the events of that frame, `bridge_summary`, the disarming and the observer's whole report before and after.
  If no bridge meets the criterion it says so (`bridge_case done=false` with the reason) and does nothing else, so the same script is run on the other verified scenario; it never invents a bridge.
* **Arguments are validated first.** Every argument is checked BEFORE the output file is created, the installation prepared, the engine booted, a frame run or a segment killed, and a malformed value is an error
  naming it, never a default: `root` and `out` non-empty text; `scenario` `lebanon_retail` (default) or `georgia_retail`; `frame_ms` an integer from 1 to 1000 (default 100; zero never advances the clock);
  `warmup` and `window_seconds` plain decimal numbers (digits and at most one point: no sign, exponent, hex, space or trailing text) from 0 to 600 (defaults 5 and 12); `bridge` an integer from 0 to 100000 and
  `segment` an integer from 0 to 255, each absent for the automatic choice (an invalid `segment` is never the automatic one); `criterion` 0 (the only one). The frames of warmup and window together may not exceed
  10,000, and the loops stop at their bounds whatever the clock does (a frame that does not advance the clock is an error). The defaults (5, 100, 12, criterion 0) are what the root's fixed driver uses.
* **What ran is judged from the events.** `bridge_case done` and `first_collapse` say only that the preconditions were met. `bridge_summary` counts the `bridge-*` events seen (`n_unsupported_kill`,
  `n_link_broken`, `n_repair_task`, `n_fall`, `n_landed`, ...) and reports `in_update_at_end` (the segments still in the update list when the window ended, read from the last state actually returned: with a window of zero seconds that is the state right after the failure, in which the killed segment is already in the list; a state with no segment line is `state_valid=false`, `in_update_at_end` -1) and `landed_all`: a window of finite length does not
  guarantee that a segment at an unknown height has landed, and the summary says whether it did.

## Expected preconditions (not evidence)

Bridge types whose segment database has an unsupported deck: boat (`DECK MID DECK`), girder (`DECK MID DECK`), concrete arch and suspension (decks between supports). Which of these the Lebanon or Georgia retail map holds is
what the survey reports; nothing here claims either does. The retail boot is the engine's own world creation (`create_local_bridge_entities`), so the bridge, its segments and their neighbours are the real
collaborating entities, not constructed state. When the criterion is met, the chosen segment's neighbour is a living unsupported deck of a living bridge, so the original's unsupported-span kill, the first break
of the route link and the first repair task are REACHABLE on the first updates of the killed segment; whether each ran, and whether the decks landed within the window, is read from the events.

## Reachable original paths, and what is declared unreached

Reachable by one run when the first-collapse criterion is met (and shown to have run only by the events): the killed-segment path (effects, routed vehicles on it, collision then child-killed to the bridge), the first-kill route link break and repair task, the update-list insertion, the neighbour
notification in both directions every frame, the unsupported-span kill by a neighbour's collision, the shape choice with the neighbours' state, the fall at `10 * dt` until the terrain (or a metre below a water
floor) and the landing with its collision effect, the removal of a supported segment, and the bridge's second and later child-killed (the bridge is no longer alive, so no second break). The fall and the landing depend on the height of the bridge above the terrain, which no source fixes: the landing is reached only if the logs show `bridge-landed`.

Declared unreached: a weapon hit on a segment (the segment's collision response is the damage path already characterised), a player or multiplayer cause, a client (the fixture refuses to run on one), a bridge with no
unsupported span (the script reports none and stops), the vehicles on the route beyond the world's counts (the stopping is not reported per vehicle), a bridge that is already dead or whose unsupported neighbours are already dead (the criterion refuses them), and the data cases no
retail bridge holds.

## Source checks run (no native run)

Real-header syntax of `br_msgs.c`, `sg_updt.c`, `sg_dstry.c`, `eech_cmb_fixture.c`, `eech_cmb_case.c` and `eech_engine.c` with the fixture on and off (`checks/syntax-bridge-2.log`, exit 0); the strict feature-off
preprocess equality above; `cargo test` for the engine, DC module and world crates, including the stand-in tests of the witness script (the flow and its order, the bounded window, the disarming, the summary counted from the events, the plain statement that no bridge was suitable, and the
validation of every argument: 40 malformed values refused before anything was prepared, booted, run or killed, and the edge values accepted and passed on exactly). The malformed-argument tests were shown RED
against the previous script (`checks/bridge-args-red-1.log`: `frame_ms=0` ran the stand-in to its cap of 3000 frames; `checks/bridge-args-red-2.log`: `segment=abc` killed a segment, `case 2:1:-1:0`, with exit 0) and GREEN against
the corrected one (`checks/cargo-test-bridge2.log`); these prove the argument validation in the real script under the real host''s Lua, not any native physics. rustfmt; default and fixture Clippy (`checks/*-bridge*.log`). The first rustfmt check (`fmt-bridge-failed-layout.log`) found only the layout of the new binding.

## The ROOT native command (run on the original; the same command, unchanged, is the comparison for the new path)

With the same fixture build used for `lua/combat-case.lua` (the `weapon-fixture` feature) and the verified retail install root:

```
eech-world lua/bridge-case.lua root=<verified retail root> out=<bridge-lebanon.jsonl> scenario=lebanon_retail warmup=5 frame_ms=100 window_seconds=12
```

and, if the survey reports no suitable bridge, or in addition, the same with `scenario=georgia_retail`. The output is JSON Lines (scopes `bridge:survey`, `bridge:before`, `bridge:after`, `frame:N:state`,
`frame:N:log`, `bridge:disarm`, `bridge:trace:*`); `done` in `bridge_case` says only that the precondition was met, and the path ran if and only if the frame logs show the `bridge-*` events. The seed is fixed (1), so two runs of the same
command on the same build are expected to be identical, and a difference is itself a finding; the root accepts or rejects the recorded behaviour before the bridge physics is extracted, and a later old/new comparison reuses this script and source.
