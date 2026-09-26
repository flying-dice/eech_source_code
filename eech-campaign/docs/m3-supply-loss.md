# M3: a SUPPLY aircraft destroyed, and what the campaign does next

This record is part of M3 ([#27](https://github.com/flying-dice/eech_source_code/issues/27)). It answers one question: **when a SUPPLY aircraft is destroyed during an active supply task, does that physical loss alter campaign progression and lead to a subsequent campaign response?**

**Answer: yes, the loss altered campaign progression, and the campaign carried on with what it had left:**
- the SUPPLY group permanently lost half its strength;
- its task ended, with the survivor turning back at the moment of the loss and landing back at its base. No keysite delivery by the group was observed, and a delivery to a group would not be observable in any case;
- the campaign later re-tasked the reduced group, which delivered fuel elsewhere on its next task.

**What the public observations cannot show:** what happened to the cargo aboard the lost aircraft, and who the task's receiver was. By elimination the receiver was a group in the field, not a keysite. So whether that receiver's shortfall was later made good is not observable.

The loss occurred naturally in the accepted reference run; no adverse scenario was manufactured. The evidence is that run's retained observations ([`reference/m3-resupply-loop/observations.jsonl.gz`](../reference/m3-resupply-loop/observations.jsonl.gz), from #66): the same file, byte for byte. No new run was needed and no runtime file changed. Everything was read through `eech-world.exe` → `campaign.lua` → `require ("eech_dc")` → `engine:objects ()`.

## The chain

Group 87567 is callsign **"Wolfpack"**: two red **Mi-17 Hip** helicopters, units 87568 and 87581, based at Beirut International.

| t (s) | Fact | Status |
|---|---|---|
| 899.83 | No red keysite has fuel at or below 75 (the request threshold) | observed (keysite snapshot) |
| 921.82 | Both aircraft's primary task becomes **TASK_SUPPLY** | observed |
| 1,043.79, 1,044.79 | Beirut's fuel drops one crate in each of two consecutive samples (63.2 → 53.2 → 43.2). 87581 is 54 m and then 19 m over Beirut; 87568 is 423 m and then 363 m away. | observed |
| 1,044.79 – 1,584.66 | Both fly south-southwest, *Performing Task*, at about 60 m | observed |
| 1,576.66, 1,577.66 | Two blue **MIM-72G Chaparral** missiles are launched: by M48A1 "Banshee" (air defence 82114) and M48A1 "Apollo" (vehicle 84435) | observed |
| **1,584.66** | **Mi-17 87568 dies**, 32 km from Beirut, 62 m up. In the same sample it leaves its group, and both Chaparrals vanish with their last positions 351–354 m from it | death observed; the hit inferred |
| 1,584.66 | The survivor, 87581, is at its farthest point from Beirut: it turns back at the moment of the loss, still *Performing Task* | observed |
| 1,619.65 | Group 87567 has one member, 87581. **No replacement joins it for the rest of the run.** | observed |
| 2,116.45 | 87581's task ends (no task, *Landing*), 455 m from Beirut. **No keysite delivery by the group** between the loss and this point. | observed (a drop-off to a group leaves no observable trace) |
| 2,187.34 | The wreck of 87568 is removed | observed |
| **3,619.24** | The campaign assigns the one-aircraft group **TASK_SUPPLY** again | observed |
| **4,013.67** | 87581 delivers fuel to **Power Station 3**: 66.5 → 100 (the complete chain from #66) | observed |

In the 30 minutes after the loss, four other SUPPLY groups (87621, 87686, 87714, 87742) pass within 1.4–4.1 km of the loss point. The campaign kept routing SUPPLY traffic through the area; nothing more is claimed about that.

## Observed, inferred and not observable

| Observed: in the public observations | Inferred: from observations and the code, or by elimination | Not observable through the public path |
|---|---|---|
| The aircraft, its group, TASK_SUPPLY, *Performing Task*, its position and the sample of its death | That the Chaparrals hit it: they vanish at it in the sample it dies | Crates, so whether 87568 carried one. Two were picked up, but crates are not reported. |
| The two Chaparrals' types, launch times and launchers, and their disappearance in the death sample | **The receiver was a group in the field.** When the task was assigned, no red keysite had fuel at or below 75, and keysites request only then (`keysite.c`); the other requester kind is a group (`group.c` `assess_group_supplies`) | Which group requested it, and that group's supply level: group supply is not reported |
| The two one-crate pickups at Beirut, and how far each aircraft was | | The task's own supplier and receiver: the task record is not reported |
| | | Whether any crate reached the receiver: a drop-off to a group changes nothing that is reported. The survivor turned back at the loss point, but whether a drop-off waypoint lay there is unknown. |
| The group shrinking from two to one, with no replacement | | The request message; why the survivor turned back (the AI's reason) |
| The survivor turning back, its task ending at Beirut, no delivery | | Whether the receiver's shortfall was later made good |
| The wreck's removal | | |
| The group's next SUPPLY assignment and its delivery to Power Station 3 | | |

**Code reading:**
- a crate goes aboard the aircraft that reaches the pick-up waypoint (`mb_msgs.c` `response_to_waypoint_pick_up_reached`);
- a crate aboard a destroyed helicopter is destroyed with its wreck (`hc_dstry.c` `destroy_local_family`, when the wreck is removed).

So if 87568 carried a crate, it was never delivered and was destroyed at 2,187.34 s. The observations cannot say whether it did.

## What the loss changed in campaign progression

- **The group's strength:** Wolfpack flew all its later SUPPLY tasks with one aircraft (observed), where it had two.
- **The task:** it ended with the aircraft back at its base and no keysite delivery (observed). Whether anything reached the group requester is not observable.
- **Supply:** two crates of Beirut's fuel left its stock (observed). Neither is observed being delivered on this task. Which aircraft carried them is not observable. If the survivor landed still carrying one, the carried-over-crate finding from #66 would apply.
- **The next decision:** the campaign re-tasked the reduced group about 25 minutes after its return (observed). Its next delivery went to Power Station 3, a keysite, not to the group that had requested the lost task's supply. Whether that group was resupplied later is not observable.

## Checks and results

| Check | Command | Result |
|---|---|---|
| Loss chain | `tools/loss-chain-check.py reference/m3-resupply-loop/observations.jsonl.gz --unit 87568` | PASS: the loss, the group's response and its next assignment are all observed ([`reference/m3-supply-loss/loss-chain.json`](../reference/m3-supply-loss/loss-chain.json)) |
| The checker requires those links | `tools/loss-chain-check-selftest.py … --unit 87568` | Removing the death, the survivor's task events after it, or its later SUPPLY assignment each makes the check fail |
| The run's other SUPPLY losses | same report | Four more are listed, not analysed: a red Mi-6 en route (1,790.61 s, 14.8 km away, no weapon found at it) and three blue transports destroyed on the ground while *Waiting* (10,011–10,234 s) |
| M1, M2 and #66 evidence | — | Unaffected: this slice changes only `tools/`, `docs/` and `reference/m3-supply-loss/`. No script or binary the runs use changed, and no run was repeated. |

## What this does not establish

- **Any other adverse outcome.** A delivery to a *keysite* lost in flight, where the receiver's shortfall would be observable, did not occur in this run. Supplier or receiver captures and destroyed keysites did not occur either.
- **A campaign response aimed at the lost supply.** No re-request or re-tasking toward the same receiver is observable.
- **That the campaign adapts.** SUPPLY traffic kept passing the loss area; no avoidance is claimed or denied.
- **Host interchangeability.** The loss, the campaign and the physical world are all inside `eech_dc.dll` (M5).
- **Cargo physics.** Crates are not observed. What happens to them comes from the code.
- **M3 acceptance.** This is one slice.

## Retained evidence

In [`reference/m3-supply-loss/`](../reference/m3-supply-loss/):

| File | What it is |
|---|---|
| `loss-chain.json` | The checker's report on the retained observations |
| `loss-events.jsonl` | The records behind each link (2,674 lines): the two aircraft's task and track events from 900 to 4,300 s; the death and the wreck's removal; the two Chaparrals and their launchers; Beirut's pickups; Power Station 3's delivery; and the red keysites' snapshot at 899.83 s |

The run's Tacview recording is M2's [`reference/lebanon-3h/recording.zip.acmi`](../reference/lebanon-3h/recording.zip.acmi) (byte-identical). It was not visually reviewed for this chain.

```sh
python tools/loss-chain-check.py reference/m3-resupply-loop/observations.jsonl.gz --unit 87568 --report loss-chain.json
python tools/loss-chain-check-selftest.py reference/m3-resupply-loop/observations.jsonl.gz --unit 87568
```
