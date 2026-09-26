# M2 reference scenario: retail Lebanon, observed

This record is part of M2 ([#23](https://github.com/flying-dice/eech_source_code/issues/23)). It answers one question: **can we trust EECH World's observations of this reference scenario as physical-world evidence?**

**Answer: yes, within the limits listed below.**
- **Two runs agree byte for byte.** The two runs started from separately assembled, hash-identical inputs and produced byte-identical Tacview, observations and metrics.
- **The two forms of evidence agree exactly.** The Tacview recording and the structured observations agree frame by frame on every object's identity, creation, disappearance and death. They agree on its position to within the recorder's stated threshold, and with the metrics on losses and keysite state.
- **Observing doesn't disturb the campaign.** The run's metrics equal the M1 baseline, which was recorded with neither form of observation.

The limits are real and specific:
- EECH reuses entity indices, which merges some weapon tracks;
- Tacview's attitude and keysite state are incomplete;
- EECH's session clock drifts;
- what "physical" means here is EECH's own simulation, inside `eech_dc.dll`.

## The scenario

| | |
|---|---|
| Campaign | Retail Comanche vs Hokum Lebanon (map5, `LEBANON.CHC`), `scenario=lebanon_retail`, seed 1 |
| Duration | 3 simulated hours: 108,000 frames of 100 ms |
| Sampling | every 10 frames (1 s), 10,800 samples |
| Binaries | The accepted M1 candidate, byte for byte (`0288ece2`): `eech_dc.dll` `da72f128…`, `eech-world.exe` `314fde3d…`, `lua.dll` `d0c799fe…` |
| Scripts | Commit `a1941f23`: `campaign.lua` `8c14062f…`, `observations.lua` `20d41fb0…`, `metrics.lua` unchanged (`dd28274e…`) |
| Inputs | 1,179 files, 102,239,559 bytes, combined SHA-256 `121a789003b68f9e…` ([`reference/lebanon-3h/inputs.json`](../reference/lebanon-3h/inputs.json)) |

**Where the inputs come from.** `tools/retail-cvh.sh <install> map5 <root>` assembles them from a retail Comanche vs Hokum install (here the GOG install), plus the repository's `setup/` data. It drops the retail script's 30-minute failure trigger.

**Two separately assembled roots were used:**
- a fresh one, built from the GOG install on 2026-09-26;
- the root the M1 baselines were recorded on, built on 2026-09-24 and copied.

`tools/input-manifest.py` finds them identical.

**Files the engine writes itself** are listed apart in the manifest:
- **written at every boot:** `ballistics-data.txt`, `guided-missiles-data.txt`, `game.cfg`, and the briefing texts;
- **written at the first boot:** `EECH.INI`. Its floats are printed by the C runtime, so a Linux and a Windows first boot write three settings differently: night light, a cockpit view position, and a joystick curve. The two roots hold one of each. The runs are nevertheless byte-identical, so those settings do not affect this scenario.

## What EECH World simulates, and what `eech_dc` owns

Today the physical world runs **inside `eech_dc.dll`**, together with the campaign. That is the whole EECH engine, and this is the transitional coupling M1 recorded. The EECH World host observes that world; it does not simulate it.

| Where | What it does |
|---|---|
| `eech_dc.dll`: the EECH engine's physical simulation | Flight and ground movement, weapons in flight, guidance and impact, damage and death, terrain, keysite damage and repair, the movement of supply cargo, and regenerated units appearing |
| `eech_dc.dll`: the EECH campaign | Task generation and assignment, supply requests, captures, regen scheduling, and group tasks and states |
| `eech-world.exe`: the host | Owns the Lua state and the time step, asks `engine:objects ()` what exists, and writes Tacview (`src/tacview.rs`, `Recording::record`) |
| `lua/campaign.lua`, `metrics.lua`, `observations.lua` | Advance time, sample, and write metrics and observations. No physics and no campaign decisions |

So "EECH World" here means the EECH engine's physical simulation, hosted and observed by `eech-world`. Moving it out of the DLL is M5.

Among the observed facts:
- **Physical:** positions, attitude, movement, deaths, weapons in flight, keysite damage.
- **Campaign state:** group task and state, keysite ownership, and supply levels. Both kinds come through the same `engine:objects ()` call.

## The evidence and its semantics

Both forms come from **the same samples**. At each sample `campaign.lua` calls `engine:objects ()` once and hands the same table to the Tacview recorder and to `observations.lua`.

| | Tacview (`recording.zip.acmi`) | Structured observations (`observations.jsonl.gz`) |
|---|---|---|
| Writer | Rust: `eech-world` `Recording::record` and `tacview.rs` | Lua: `observations.lua` |
| Identity | A Tacview object per EECH entity *lifetime*. A new object appears when an index is first seen, changes kind or sub-type, or comes back to life. | The EECH entity index, with raw events (`appear`, `retype`, `alive`, `gone`) |
| Creation / disappearance | Declaration; removal (`-id`) when an object is no longer reported | `appear` / `retype`; `gone` |
| Losses | `Event=Destroyed` when alive becomes false | `alive` false |
| Position | Latitude and longitude through the scenario's fitted map projection, altitude = EECH y, written only when the object moved more than 0.5 m on an axis or turned more than 1° | EECH map metres (x east, y up, z north): the full state of every object every 60 s and at each metrics checkpoint |
| Attitude | Heading (0 = north, clockwise), pitch, and roll (sign flipped: right positive), in degrees, written with the position | Heading, pitch and roll in radians, as EECH reports them |
| Keysites | Declared once, with their position and coalition. Captures appear only as `Message` events. | Position, side, usable state, ammo, fuel and efficiency; `side` and `usable` events |
| Time | `#t`: EECH's session elapsed time, with two decimals | `t`: the same value, printed the same way |

**What is reported:**
- aircraft, ground vehicles, air defence, ships and infantry;
- missiles, rockets and bombs in flight;
- keysites.

Gun and artillery rounds, decoys, cargo and debris are not (`csrc/eech_observe.c`, following EECH's own Tacview logger). A weapon that lives less than one sample can be missed.

## Checks and results

`tools/reference-windows.ps1` ran on Windows 11 on 2026-09-26. It started the two runs in parallel (1,070 s) and checked each with `tools/observation-check.py`.

| Check | Result |
|---|---|
| **Identity and lifecycle.** The observations, replayed through the recorder's rules, must give the recording's exact declarations (with Type, Name, Coalition and Group), removals and Destroyed events, frame by frame. | **Exact in all 10,800 frames.** Declared: 7,498 objects (689 air defence, 127 fixed wing, 524 ground vehicles, 253 helicopters, 152 infantry, 133 keysites, 36 ships, 5,584 weapons). Removed: 5,919 in all. 5,821 are disappearances. The other 98 are replacements: EECH reused an index for an entity of a different type within one sample (97 weapons, 1 ground vehicle), and the recorder correctly started a new object. Destroyed: 530. |
| **Position.** At each snapshot, the recorded position projected back to map metres (by the checker's own projection code) must be within the recorder's threshold plus the file's rounding: 0.52 m horizontally, 0.56 m in altitude, 1.06° of heading. | **308,368 comparisons, all within.** Largest differences: x 0.493 m, z 0.506 m, y 0.53 m, heading 1.004°. Keysites and air defence within 1 cm. |
| **Losses**, three ways | **Identical** in the observations, the recording and the metrics: blue 285 (64 air defence, 25 fixed wing, 105 ground vehicles, 65 helicopters, 14 infantry, 12 ships), red 245 (51, 27, 88, 73, 6). |
| **Keysite state** at the last checkpoint | **Identical to the metrics**: 133 keysites, their usable states, and mean ammo and fuel per side and type. 21 usable-state changes were observed: 9 usable → out of action, 5 usable → repairing, 3 out of action → repairing, 4 repairing → usable. No keysite moved. |
| **Captures**, three ways | None in these 3 hours, in any form. **Not exercised.** |
| **Repeatability.** Two runs on the two roots. | **Byte-identical:** `recording.acmi` (117,690,861 bytes, `3bd17b64…`), `observations.jsonl` (69,586,021 bytes, `31360275…`), `metrics.json` (`fa09d562…`) |
| **Observation does not perturb.** The metrics against `regression/windows/lebanon_retail.json`, which was recorded with neither form of observation. | **IDENTICAL**; 39/39 campaign expectations |
| **The checker detects discrepancies** (`tools/observation-check-selftest.py`, on a 3-minute run) | It passes the unmodified evidence and fails each deliberate discrepancy: a shifted position, a dropped declaration, a dropped Destroyed event, a changed coalition, a dropped disappearance. |
| **M1 evidence preserved**, with the new scripts | `tools/regress-windows.ps1 -Exact`: Georgia and Lebanon IDENTICAL to their baselines (30/30, 39/39). `tools/lifecycle-windows.ps1`: unchanged (5 PASS, 1 KNOWN DEFECT). `docs/m1-baseline.md` and the baselines are untouched. |

Each run's report is `reference/lebanon-3h/check.json`. The two runs' reports are identical.

## Fidelity limits found

1. **Identity: EECH reuses entity indices.** A reuse by an entity of a *different* type within one sample is recognised: 98 times in this run, and the recorder started a new object each time. But when a weapon expires and another of the same kind and sub-type takes its index within one sample, both forms of evidence (which key on the index) merge the two into one track.
   - The checker's plausibility test found 23 one-second jumps of 34–100 km: 16 M26A1 MLRS rockets, 4 S-8KOM, 2 MIM-72G Chaparral and 1 AGM-114R Hellfire.
   - At 12 snapshots, a weapon index was held by a different launcher than the one that fired it.
   - These are lower bounds. A merge at a short distance, or from the same launcher, isn't detected.
   - The metrics' `launched` counts undercount by the same mechanism. No unit other than a weapon showed a jump.
   - A per-entity generation, which the engine does not expose, would remove this.
2. **Tacview attitude can be stale.** The recorder writes attitude only with a position or heading change, so pitch and roll that change on their own are not refreshed. The largest staleness observed was 5.1° of pitch and 79.5° of roll.
3. **Tacview keysites are incomplete.** A keysite's coalition is fixed at declaration: a capture shows only as a `Message` event (by code reading; no capture happened in this run). Usable state and supply are only in the observations and the metrics.
4. **Time.** `t` is EECH's session elapsed time, a 32-bit float the engine accumulates. At the 10,800th sample it reads 10,770.37 s, against 10,800 s of host frame time: **−29.63 s (−0.27%)**, the same in both runs.
   - Summing 0.1 s in 32-bit floats accounts for about −4.6 s of that. The rest has not been explained.
   - The samples are 0.99–1.00 s apart on this clock.
   - Tacview's `ReferenceTime` is a fixed label (`2026-09-24T06:00:00Z`), while the campaign starts at 08:50:16 on day 1. So Tacview's wall clock and lighting are not the campaign's.
5. **Position on the globe.** Retail map5 is not metric. The projection is an affine fit to eight real airbases, with a median residual of 1.1 km (reported in `campaign.lua`; not re-verified here). The check verifies the recorder's arithmetic, not the fit.
6. **Sampling.** At one sample per second, anything shorter-lived than a second can be missed, and the order of events within a second is unknown.
7. **EECH's physical model is EECH's.** For example, the MV-22 Osprey, a tiltrotor EECH classes as a helicopter, cruises at up to 177 m/s over one second (161 m/s averaged over a minute). It accounts for all 1,961 helicopter plausibility flags; every other helicopter type stays at or below 82 m/s. Nothing here validates EECH's physics against reality.
8. **Names.** The engine converts EECH's strings to UTF-8 lossily. All 587,492 names and types in this run are plain ASCII, so nothing was lost here.

## What this establishes

- **Reproducible from identified inputs.** The reference scenario is reproducible from identified inputs (the file manifest; two separately assembled roots) and an identified candidate and scripts: repeated runs are byte-identical.
- **Both forms represent the same activity.** For this scenario, Tacview and the structured observations represent the same world activity. Identity, creation, disappearance and deaths match exactly, positions match within the recorder's threshold, and both agree with the metrics on losses and keysite state.
- **Observing is non-perturbing** for this scenario.
- **The limits are characterised:** the fidelity limits and the time semantics above.
- **Responsibility is split:** the host observes; the physical simulation and the campaign both run in `eech_dc.dll`.

## What this does not establish

- **A physical world separate from the campaign.** It runs inside `eech_dc.dll` (M5).
- **Real-world accuracy** of EECH's physics or of the map projection.
- **Keysite captures in the evidence.** None occurred in this run.
- **Other scenarios, longer runs, or Linux.** Linux fights a different war.
- **A visual review of the recording in Tacview.** The recording was generated and structurally checked, not visually reviewed, in this slice.
- **The cause of most of the session-clock drift.**
- **Anything about the campaign/world feedback loop** (M3).

## Retained artifacts

In [`reference/lebanon-3h/`](../reference/lebanon-3h/):

| File | What it is |
|---|---|
| `recording.zip.acmi` | 30.3 MB; Tacview opens it directly |
| `observations.jsonl.gz` | 8.2 MB |
| `metrics.json` | identical to `regression/windows/lebanon_retail.json` |
| `inputs.json` | the input manifest |
| `check.json` | the checker's report on these files |
| `summary.txt` | the runner's summary |
| `BUILD-INFO.txt` | the binaries' and scripts' hashes |

The checker runs on the committed files with no retail data:

```sh
cd reference/lebanon-3h
python ../../tools/observation-check.py recording.zip.acmi observations.jsonl.gz metrics.json
```

The full outputs of both runs, and of the M1 preservation runs, are kept locally under `E:\eech-runs\m2\`.

## Reproduce

```sh
# Git Bash: the inputs, twice, and the candidate
tools/retail-cvh.sh "<Comanche vs Hokum install>" map5 <root>
tools/input-manifest.py <root> --compare reference/lebanon-3h/inputs.json
tools/build-windows.sh target/m1-candidate        # at 0288ece2: the binaries; at this commit: the scripts
```
```powershell
tools\reference-windows.ps1 -Root <root> -RepeatRoot <second root> -Bin <binaries with this commit's lua/> -Out <dir>
python tools\observation-check-selftest.py <short run's recording.acmi> <observations.jsonl> <metrics.json>
```
