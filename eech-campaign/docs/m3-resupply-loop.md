# M3: one campaign → world → campaign resupply loop

This record is part of M3 ([#27](https://github.com/flying-dice/eech_source_code/issues/27)). It answers one question: **can we demonstrate, from the public integration path, one real campaign decision causing physical world activity whose outcome feeds back into campaign state?**

**Answer: yes, for resupply, within the limits below.**
- **One chain in full.** The campaign assigns an Il-76 transport a SUPPLY task. It flies 30 km, continuously, from the supplier to the receiver. At the receiver, the airbase's fuel goes from 10 to 100. The campaign then uses that fuel: groups draw on it, another SUPPLY task picks up from it, and when it falls below the request threshold again the campaign sends another SUPPLY task there.
- **Across the whole run.** Every one of the run's 32 deliveries is attributed to the SUPPLY group overhead, and 31 chains are complete.
- **Seen only from outside.** Everything is observed through `eech-world.exe` → `lua/campaign.lua` → `require ("eech_dc")` → `engine:objects ()`, with no private inspection.

The loop runs **inside `eech_dc.dll`**: the campaign and the physical world both live there, and no host carries the outcome between them. This shows the loop works and can be observed through the public path. It does not show that a host could supply the world.

## Setup

| | |
|---|---|
| Scenario | The M2 reference scenario: retail Lebanon, seed 1, 3 simulated hours, sampled every 1 s |
| Binaries | The accepted M1 candidate, byte for byte (`0288ece2`): `eech_dc.dll` `da72f128…`, `eech-world.exe` `314fde3d…` |
| Scripts | `0ea49a4a` and later: `campaign.lua` `4b1fa47d…`, `observations.lua` `68527ae6…` |
| Inputs | The two M2 roots: 1,179 files, combined SHA-256 `121a7890…` |
| Command | `tools\reference-windows.ps1 -Root <root> -RepeatRoot <second root> -Bin <binaries with these scripts> -ObserveSupply` |

`-ObserveSupply` passes `observe_supply=1`, which adds three kinds of events to the M2 structured observations. It is Lua only, and off by default.
- **`task`:** a unit's group, primary task or operational state changed.
- **`supply`:** a keysite's ammo or fuel moved by 0.5 or more in one sample.
- **`track`:** every sample, the position and state of each aircraft whose group's primary task is `TASK_SUPPLY`.

All three come from the fields `engine:objects ()` already reported. Nothing new is exposed by the engine.

## The chain

Group 88235 is callsign **"Jester"**: one red **Il-76MD Candid-B** (unit 88236, Tacview object `243` in [`reference/lebanon-3h/recording.zip.acmi`](../reference/lebanon-3h/recording.zip.acmi)). It is based at **Wujah Al Hajar** airbase. The run's first delivery is its chain.

| t (s) | Fact | Kind | Evidence |
|---|---|---|---|
| — | Beirut International's fuel is at the floor (10); it has been ≤ 75, the request threshold, so it asks its force for fuel | campaign | `supply` events; `keysite.c` `update_keysite_cargo` |
| 1,160.77 | Jester's primary task becomes **TASK_SUPPLY** (state *Taxiing*) | campaign decision | `task` event |
| 1,296.73 – 1,369.71 | *Preparing for takeoff* → *Take off* → *En route*; it climbs to about 1,500 m | campaign state and physical movement | `task` and `track` events |
| **1,555.67** | Jester is **85 m** (horizontal) over **Halat** airbase, at 1,487 m, *Performing Task*. In the same sample, Halat's fuel drops **89.6 → 79.6**: one fuel crate (`CARGO_FUEL_SIZE`, 10) | physical position; campaign accounting | `track`; `supply`; `ks_msgs.c` pick-up |
| 1,555.67 – 1,904.58 | It flies from Halat to Beirut: 350 one-second samples, 30.2 km flown (29.5 km direct), up to 87.5 m/s, at 1,478–1,508 m, with no gap | physical | `track` events |
| **1,904.58** | Jester is **43 m** over **Beirut International**, at 1,507 m. In the same sample, Beirut's fuel goes **10.0 → 100.0** | physical position; campaign state | `track`; `supply`; `ks_msgs.c` drop-off |
| 2,323.15 | The task ends (primary task none, state *Landing*); it returns to base and lands at 2,505.88 | campaign and physical | `task` events |

**What the campaign then does with the fuel**, all at Beirut:

| t (s) | Beirut fuel | What happens |
|---|---|---|
| 2,356.1 | 97.2 → **55.8** | A draw of 41.4. No SUPPLY aircraft is over Beirut, so by the code this is groups refuelling from the keysite's stock (`group.c` `assess_group_supplies`). The level is now **≤ 75**: by `update_keysite_cargo`, Beirut requests fuel again. |
| 2,420.0 | — | Group 88227 **"Hawkeye"** (Il-76) is assigned TASK_SUPPLY, 64 s after Beirut fell below the threshold. |
| 2,455.95 | −10 | One crate is **picked up from Beirut** by another SUPPLY group, 87746, which is 92 m overhead. The delivered fuel has become a source for the campaign's logistics. |
| 2,570.78, 2,629.7, 3,143.94 | −10, −10.8, −10 | Further draws with no SUPPLY aircraft overhead |
| **3,324.68** | 10.0 → **100.0** | **Hawkeye delivers to Beirut**, after picking up at Halat at 2,970.2. It is chain 3 of the run. |

So one campaign decision caused physical activity (the flight to the two waypoints). Its outcome, a new supply level, was used by the campaign's own rules: rearm/refuel draws, pickups, the request threshold. That led to a new campaign decision and a second delivery to the same keysite.

## Which facts are campaign and which are physical

| Campaign facts: EECH_DC's state and decisions | Physical facts: the world EECH simulates |
|---|---|
| A group's primary task (TASK_SUPPLY) and operational state | Aircraft positions, altitude and heading every second |
| Keysite ammo and fuel levels, and their changes: pick-up −10, drop-off to 100, draws | Flying from the base to the pick-up waypoint to the drop-off waypoint: distance, speed, continuity |
| The request threshold (75) and the assignment of SUPPLY tasks | Arriving over the supplier and over the receiver at the sample the levels change |

Both kinds are produced inside `eech_dc.dll` and read through the same public call.

**Inferred, not observed:**
- **The task's record:** which supplier and receiver it names. The campaign keeps it on the task entity, which `engine:objects ()` does not report. The chain is attributed from where the group is at the one-second sample in which the levels change: every pick-up is within 336 m and every drop-off within 78 m, against a 500 m radius.
- **The link from a threshold crossing to a particular assignment:** Hawkeye's assignment coincides with Beirut's request, but the request itself is not observed.
- **Which group drew from a keysite.** Group supply levels are not reported.

## Checks and results

| Check | Command | Result |
|---|---|---|
| Resupply chains | `tools/supply-chain-check.py observations.jsonl metrics.json` | **32 deliveries, all 32 attributed to exactly one SUPPLY group, 31 complete chains.** They cover red airbases, military bases, power stations and a FARP, and a blue military base, carrying ammo and fuel, by Il-76, An-12, C-130J, Mi-6 and Mi-17. The deliveries equal the metrics' `resupplied` counts exactly. |
| The one incomplete chain | same | An-12 "Python" (group 87686) delivers to Power Station 1 at 4,971.11, on a SUPPLY task assigned at 4,158.3. On that task it overflew its supplier, Beirut, at 4,635.43 (86 m), when Beirut had no fuel and so no crate. EECH's pick-up then takes nothing (`mb_msgs.c`: no cargo), so no step appears. The crate it delivered came from its **previous** SUPPLY task: picked up at Beirut at 1,390.71, then dropped off at 1,892.59 to a group in the field, 8.6 km from any keysite. A drop-off to a group restocks the group but leaves the crate aboard (`mb_msgs.c`), so the same crate was delivered again (see Findings). The checker is right to report the chain incomplete: there was no pick-up on this task. |
| The chain checker detects broken chains | `tools/supply-chain-check-selftest.py` (on a one-hour run) | Removing the group's SUPPLY observations before the pick-up, the pick-up itself, 60 s of the flight, or the aircraft at the drop-off each makes the chain incomplete. |
| Repeatability | two runs, two roots | Recording, observations (85,716,447 bytes, `da26bab0…`) and metrics **byte-identical**; the chain reports are identical |
| M2's check | `tools/observation-check.py` | PASS on both runs; the new events are ignored by it |
| Observation does not perturb | metrics against `regression/windows/lebanon_retail.json` | **IDENTICAL** (39/39 expectations). The recording is byte-identical to M2's retained one (`3bd17b64…`). |
| M2 preserved | the M2 settings (without `observe_supply`), new scripts, a third root | Recording, observations and metrics **byte-identical to M2's retained evidence** (`3bd17b64…`, `31360275…`, `fa09d562…`) |
| M1 preserved | `tools\regress-windows.ps1 -Exact` with the new scripts | Georgia and Lebanon **IDENTICAL** (30/30, 39/39) |

## Findings

- **Waypoints are overflown, not landed at.** EECH's supply tasks pick up and drop off when the aircraft reaches the waypoint: the Il-76 was at about 1,500 m, 43–85 m horizontally from the keysite. Cargo crates are not among the observed objects, so the physical transfer is the aircraft's arrival over the keysite. No landing or loading was observed.
- **One crate can resupply two requesters.** `response_to_waypoint_drop_off_reached` (`mb_msgs.c`) handles requesters two ways:
  - a **keysite** requester gets the carried crate moved into its cargo;
  - a **group** requester gets its own level raised, but the crate stays aboard the aircraft.

  The aircraft then delivers that crate again on its next task: the An-12 above did. This appears to be an original EECH defect. It is recorded here, not changed; reviewing it belongs with the corrections (`docs/corrections.md`).
- **One crate refills a keysite to 100.** A pick-up takes one crate (10 points) from the supplier, and a keysite drop-off sets the receiver to 100 whatever its level (`ks_msgs.c`). That is EECH's own rule.
- **SUPPLY tasks can run back to back.** A group can fly one SUPPLY task after another without its primary task changing, so the observations cannot separate successive tasks. The checker starts each chain after the group's previous delivery.
- **The engine's boot changes the process's working directory.** It calls `chdir` to `<root>/cohokum` (`csrc/eech_engine.c`). `campaign.lua` loaded a script after boot by a relative path, and that failed. It now loads it before boot. An in-process host (DCS) would see its own working directory change. This is recorded here, not changed.

## What this does not establish

- **Adverse outcomes.** This is one successful loop, and 31 like it. No chain was followed where the transport is shot down, the supplier is captured or destroyed, the delivery fails, or the receiver is lost. Whether and how those outcomes feed back is not shown.
- **Host interchangeability.** The campaign and the physical world are both in `eech_dc.dll`; the outcome reaches the campaign inside the DLL, not through the host. That the loop would work with another host's world (EECH World outside the DLL, or DCS) is not shown (M5 and later).
- **The task's own record.** Supplier, receiver and the request message are inferred from position and supply-level changes, not read.
- **Other activity types.** Only resupply. Combat, repair, captures and troop insertion are not traced here.
- **Longer runs, other scenarios, or Linux.**
- **A visual review in Tacview.** The recording is M2's, structurally checked, not visually reviewed for this chain.
- **The rest of M3.** This is one bounded slice, not the M3 acceptance.

## Retained evidence

In [`reference/m3-resupply-loop/`](../reference/m3-resupply-loop/):

| File | What it is |
|---|---|
| `observations.jsonl.gz` | The full run with the supply detail (10.5 MB) |
| `chain-jester.jsonl` | The featured chain's own events: the group's `task` and `track` events, and Halat's and Beirut's `supply` events from 1,100 to 3,400 s |
| `chains.json` | The chain checker's report, with every chain |
| `check.json` | M2's check of these observations against the recording |
| `summary.txt`, `BUILD-INFO.txt` | The runner's summary; the binaries' and scripts' hashes |

The Tacview recording and the metrics of this run are byte-identical to M2's retained [`reference/lebanon-3h/`](../reference/lebanon-3h/) files, so they are not duplicated.

Both checks run on the committed files with no retail data:

```sh
cd reference/m3-resupply-loop
python ../../tools/supply-chain-check.py observations.jsonl.gz ../lebanon-3h/metrics.json
python ../../tools/observation-check.py ../lebanon-3h/recording.zip.acmi observations.jsonl.gz ../lebanon-3h/metrics.json
```

## Reproduce

```powershell
# the M2 roots (tools/retail-cvh.sh), and the M1 candidate's binaries with this commit's lua/
tools\reference-windows.ps1 -Root <root> -RepeatRoot <second root> -Bin <dir> -Out <dir> -ObserveSupply
python tools\supply-chain-check-selftest.py <1-hour run's observations.jsonl> <its metrics.json>
```
