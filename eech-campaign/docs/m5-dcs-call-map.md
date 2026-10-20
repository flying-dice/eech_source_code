# M5: the DCS call map

This record answers M5 done-when criterion 1 (Confluence page 11534337, "EECH M5 Goal: EECH_DC into DCS World"):

> **DCS call map.** Every world interaction the campaign needs (creating and routing forces, tasking, events, time, terrain) is mapped to how DCS World's mission scripting environment provides it. Each gap has a named workaround.

It is a source-grounded proposal for review, not an acceptance. It changes no code, build, baseline or DCS configuration, and no DCS or EECH runtime was launched for it. DCS-only runtime risks (native module load, frame budget and threading, save and restore, multiplayer) are criterion 5, owned by `docs/m5-dcs-risks.md` (`eech-campaign/docs/`, in another session's worktree and not yet merged); this record cross-references them and does not probe them.

**In brief.**
- The campaign side of the whole-engine `eech_dc` was inventoried by a reproducible script over the 1,217 compiled C files at `65892de1`. It measured 64 campaign→world functions (161 call sites), 66 world→campaign functions (265 sites), the entity messages each side sends and 10 categories of world-service tokens read by campaign code (method and coverage below).
- 42 interactions in 12 categories are mapped to DCS. 8 fit directly (campaign-only work that needs no DCS call), 19 are adapted and 15 are gaps. Every gap, and every adapted row that needs one, has a named workaround (G1–G21).
- The largest gaps: there is **no verified "waypoint reached" signal** in DCS (G7); **movement is pulled from the campaign's guide every frame** in EECH, but DCS AI flies a route it is given (G4); EECH's **loss, damage, target-detection and capture bookkeeping runs inside world code** (G8, G9); **no save of physical state** in the mission environment (G19); **one engine per process** against DCS mission restarts (G20); and **this PC has no Syria terrain**, so Lebanon cannot be placed in DCS here (G21).
- Of the 27 DCS evidence items (D1–D27), 13 are confirmed by this install's shipped files ([R-dcs]), 13 rest on tslua-dcs only ([R-ref]) and 1 is unknown ([U]); five items carry a named partial unknown. No shipped file gives a mission-API signature; every signature below is tslua-dcs's.

## Evidence labels

| Label | Meaning |
|---|---|
| **[R-src]** | Reproduced from EECH source at `65892de1` in this session (`grep -n`, `sed -n`, or the inventory script below). A statement about code, not a runtime observation. |
| **[R-dcs]** | Verified in this DCS install's shipped files: Lua under `Scripts/`, `DCS:API/Sim_ControlAPI.md`, or export strings in `DCS:bin/Scripting.dll`. Read-only. |
| **[R-ref]** | From tslua-dcs only, at commit `1179554fcafcef92d71313336cc09500febf8245`. Its `src/exports/*.export.ts` files say they were reflected from a live DCS 2.9.29.27468 (reported by tslua-dcs, not reproduced here). The hand-written `src/*.ts` files give the signatures. |
| **[I]** | Inferred. |
| **[U]** | Unknown. The text says what would settle it. |

Citation prefixes: a bare `file.c:N` is under `aphavoc/source/` or `modules/` (unique basename) unless a fuller repository path is given. `DCS:` is relative to `D:\Program Files\Eagle Dynamics\DCS World OpenBeta\`. `MT/` is tslua-dcs `packages/tslua-dcs-mission-types/src/`, `MTX/` is its `src/exports/`.

## 1. Inventory method

### What the product is

At `65892de1` the product is the whole-engine module: `eech-world` → `require ("eech_dc")` → `crates/eech-engine` → the original C (`eech-campaign/README.md`). The campaign and the reference world are compiled into one DLL and share one entity system (see the SUPPLY map, `m5-supply-ownership.md`, on PR #74 at `31204591`, "Coupling inside `eech_dc.dll`"). So "the world interactions the campaign needs" are found inside the C, not at the public Lua surface. That surface takes only boot configuration and elapsed time (`eech-campaign/crates/eech-dc/src/module.rs:20-29`, `:43-54`).

### Classification

Every file in `eech-campaign/crates/eech-engine-sys/sources.txt` (the compiled C list, 1,217 files) gets one class by path. The first matching rule wins:

| Class | Paths | Files |
|---|---|---|
| **C** campaign | `aphavoc/source/ai/`, `ai_extrn.c`, `entity/mobile/cargo/`, `entity/special/{force,group,task,guide,keysite,waypoint,division,regen,landing,sector}/` | 159 |
| **W** world entities | `entity/mobile/` (except cargo), `entity/fixed/`, `entity/special/{bridge,effect}/` | 332 |
| **E** environment services | `modules/3d/` (terrain, 3D objects, clouds), `misc/{terr_db,tod,timeaccl}`, `entity/special/session/` (time of day, weather), `modules/maths/random.c`, `modules/system/{time,timer}.c` | 73 |
| **X** presentation (excluded) | `ui_menu/`, `appsound/`, `app3d/`, `gunships/`, `entity/special/camera/`, `cdmusic.c`, `modules/{userint2,graphics,2d,sound}/` | 517 |
| **F** framework | everything else: the entity system, comms, events, maths, file and memory | 136 |

Choices a reviewer may contest: `guide/` is campaign, although it also steers the flight model (#74, "The guide straddles the line"). `landing/` (landing sites, takeoff routes) is campaign, although #74 leaves its ownership open. `cargo/` is campaign (crates are a derived campaign record, #74 stage 1). `gunships/` (the player's cockpit, avionics and flight model) is excluded as player presentation.

### Measures

`inventory.py` (in full below) strips comments, keeps line numbers and indexes top-level function definitions per class. It counts:
1. **Direct cross-class calls** to functions defined in exactly one class.
2. **Static entity messages** sent with `notify_*_entity (ENTITY_MESSAGE_X, ...)`, per sending class.
3. **Entity attribute tokens** in campaign files (`get_/set_*_entity_*(en, X_TYPE_Y)`).
4. **World-service tokens** in campaign files: a named list of functions, macros and attributes in 10 categories.

Results at `65892de1`, with Python 3.13, Windows. Run time is about 7 s. [R-src]

| Measure | Result |
|---|---|
| Functions defined once per class | C 644, W 858, E 853, F 1,059, X 4,429; 76 names defined in several classes are excluded from edges |
| C → W calls | 39 functions, 86 sites |
| C → E calls | 25 functions, 75 sites |
| W → C calls | 64 functions, 261 sites |
| E → C calls | 2 functions, 4 sites |
| Static messages sent | C 29 types, 81 sites; W 24 types, 89 sites; F 15, 29; X 4, 31 |
| Attribute tokens in C accessor calls | 208 distinct, 2,698 sites |
| World-service tokens in C (sites, files) | time 41/11; random 30/12; terrain 58/13; map grid 199/27; roads 21/8; 3D objects 75/8; weather 4/3; sensing 27/9; physical state 511/66; range maths 48/20 |

Reproduce:

```sh
git rev-parse HEAD                      # 65892de17bf5c4bb9ab6e9f8af70779e35b0106b
python -I inventory.py . --detail detail.json      # from the repository root; prints the table above
# grep cross-checks (raw grep counts include comments, so they can exceed the script's)
S=eech-campaign/crates/eech-engine-sys/sources.txt
C='^aphavoc/source/(ai/|ai_extrn\.c|entity/mobile/cargo/|entity/special/(force|group|task|guide|keysite|waypoint|division|regen|landing|sector)/)'
W='^aphavoc/source/entity/(mobile/|fixed/|special/(bridge|effect)/)'
E='^(modules/3d/|aphavoc/source/misc/(terr_db|tod|timeaccl)|aphavoc/source/entity/special/session/|modules/maths/random\.c|modules/system/timer?\.c)'
X='^(aphavoc/source/(ui_menu/|appsound/|app3d/|gunships/|entity/special/camera/|cdmusic\.c)|modules/(userint2|graphics|2d|sound)/)'
wc -l < $S                                             # 1217
grep -cE "$C" $S                                       # 159
grep -E "$W" $S | grep -cvE "$C"                       # 332
grep -cE "$E" $S; grep -cE "$X" $S                     # 73, 517
grep -vE "$C|$W|$E|$X" $S | wc -l                      # 136
grep -E "$C" $S | xargs grep -wn get_delta_time | wc -l            # 14 raw (13 outside comments)
grep -E "$C" $S | xargs grep -wn get_3d_terrain_elevation | wc -l  # 27 raw (26)
grep -E "$C" $S | xargs grep -wnE 'frand1|sfrand1|rand16' | wc -l  # 34 raw (30)
```

### Coverage: measured against sampled

- **Measured exhaustively** for the stated rules: the class of every compiled file; every direct call between uniquely defined C and W/E functions; every static message name; every named service token in campaign files.
- **Not measured:**
  - **Macro-expanded calls** other than the named tokens. For example `get_3d_terrain_elevation` and `frand1` are macros (`modules/3d/terrain/terrelev.h:108`, `modules/maths/random.h:153`). They are covered because they are named.
  - **Function-pointer dispatch** through the entity type tables (for example `kill_client_server_entity`, `aphavoc/source/entity/system/en_funcs/en_dstry.h:107`). This is covered only where a message or attribute names the interaction.
  - **Dynamic message ids:** the waypoint-reached messages come from `waypoint_database` (`wp_dbase.c:846`), not a literal.
  - **Calls from F and X**, and the 76 ambiguous names.
  - **The headless host layer** (`eech-campaign/crates/eech-engine-sys/csrc/`), which is not in `sources.txt`.
- **Classified, not exhaustively read:** 21 physical-state attribute tokens were named and counted (section 1 table). The rest of the 208 attribute tokens found in campaign accessor calls (task, group, list and keysite links, names, sides) were read by name only and judged campaign or framework state [I]. The rows below cite representative sites; the full site lists are in the script's `--detail` output.
- **Reused:** the SUPPLY rows lean on #74's map, which is supporting evidence only. The #74 citations reused here were spot-checked at `65892de1` (`assign.c:983`, `keysite.c:377`, `mb_msgs.c:1822`).

So this record claims a complete map **for the measured categories**. It does not claim that no other interaction exists.

## 2. DCS sources

- **Install:** `DCS:autoupdate.cfg` gives `"branch": "openbeta"`, `"version": "2.9.29.27468"`. Installed terrains are Caucasus, Mariana Islands, Nevada, Normandy and Persian Gulf (`DCS:Mods/terrains/`). **There is no Syria terrain.** [R-dcs]
- **Desanitised.** `DCS:Scripts/MissionScripting.lua:16-21` has all six sanitising lines commented out (`os`, `io`, `lfs`, `require`, `loadlib`, `package`). The stock copy next to it, `MissionScripting.lua.dcsstudio.bak`, has them active; `diff` shows only `16,21c16,21`. This is not stock. Interactions that rely on it: loading the native module with `require` (D1), and file persistence or logging through `io` or `lfs` (D2). The edit touches only those six names, so the other mapped interactions should be present in a stock install too [I]; D12–D13 are in the hooks environment, not the mission one. [R-dcs]
- **Mission environment:** `DCS:Scripts/ScriptingSystem.lua` declares the classes (`:12-21`, including `Warehouse`), builds `country.id` (`:24-33`) and loads `DCS:Scripts/World/EventHandlers.lua`, `DCS:Scripts/World/PersistenceHandlers.lua` and `rtsScripting.lua` (`:117-119`). The bindings are native: `DCS:bin/Scripting.dll` holds the strings `regLuaCoalition`, `regLuaGroup`, `regLuaUnit`, `regLuaController`, `regLuaWorld`, `regLuaTime`, `regLuaLand`, `regLuaCoord`, `regLuaAtmosphere`, `regLuaAirbase`, `regLuaEnv`, `regLuaTrigger`, `regLuaWeapon`, `regLuaMath`, `regLuaStaticObject`, `regLuaSceneryObject` and others, plus `createWorldEvent`, `clearLuaTimers` and `luaMathRandomInit` (`grep -a`). These strings confirm that the modules exist, not their method names. The DLL is UPX-packed, so method-name strings were not found. [R-dcs]
- **Hooks (GUI) environment**, a separate Lua state: `DCS:API/Sim_ControlAPI.md` (hooks are loaded into the GUI state, `:22-23`; callbacks via `setUserCallbacks`, `:28`; standard `io` and `os`, `:61-68`; `lfs.writedir`, `:78`). [R-dcs]
- **tslua-dcs** at `1179554f`, read-only through `gh api`. [R-ref]

### DCS evidence items

| ID | DCS capability | Evidence | Label |
|---|---|---|---|
| D1 | Load a native Lua module in the mission environment | `require`, `loadlib` and `package` remain only because `DCS:Scripts/MissionScripting.lua:19-21` are commented; DCS-gRPC uses the same route (`eech-campaign/docs/dcs.md`) | [R-dcs] (desanitised only) |
| D2 | File I/O in the mission environment | `io`, `lfs`, `os`: `DCS:Scripts/MissionScripting.lua:16-18` commented | [R-dcs] (desanitised only) |
| D3 | Event subscription | `world.addEventHandler`, `removeEventHandler`, `onEvent` (Lua dispatch to `handler:onEvent (event)`): `DCS:Scripts/World/EventHandlers.lua:3`, `:7`, `:11-15` | [R-dcs] |
| D4 | Loss events | `S_EVENT_CRASH`, `S_EVENT_DEAD`, `S_EVENT_UNIT_LOST` (initiator) and `S_EVENT_KILL` (target) handled in the mission environment: `DCS:Scripts/rtsScripting.lua:16`, `:21` | [R-dcs] |
| D5 | Takeoff and landing events | `S_EVENT_BIRTH`, `S_EVENT_TAKEOFF`, `S_EVENT_RUNWAY_TAKEOFF`, `S_EVENT_LAND` in the radio-dialog Lua state: `DCS:Scripts/UI/RadioCommandDialogPanel/Config/Common/ATC.lua:647`, `:670`, `:673` | [R-dcs] (names; another Lua state) |
| D6 | Other events: `SHOT`, `HIT`, `RUNWAY_TOUCH`, `ENGINE_STARTUP`, `BASE_CAPTURED`, `MISSION_END`, `UNIT_TASK_STAGE` and others, 63 names | `MTX/world.export.ts:24-86`; numeric ids not given anywhere read | [R-ref]; ids [U] |
| D7 | Find a group, and task it | `Group.getByName`, `Group:getID`, `Group:getController`, `Controller:pushTask ({id = 'AttackGroup', params = {groupId = ...}})`: `DCS:Scripts/rtsScripting.lua:112`, `:118-128` | [R-dcs] |
| D8 | Activate, remove and stop a group | `Group:activate`, `Group:destroy`, `Controller:setOnOff`, `Controller:setCommand ({id = 'StopRoute', ...})`: `DCS:Scripts/ScriptingSystem.lua:62`, `:66`, `:70`, `:84` | [R-dcs] |
| D9 | Unit health ceiling | `Unit:getLife0` = `self:getDesc ().life`: `DCS:Scripts/ScriptingSystem.lua:112` (so `getDesc` exists) | [R-dcs] |
| D10 | Country ids | `country.id` from `DCS:Scripts/Database/db_countries.lua`: `DCS:Scripts/ScriptingSystem.lua:24-33` | [R-dcs] |
| D11 | Persistence hooks | `world.setPersistenceHandler (name, fn)`, `world.runPersistenceHandlers (storageFunc)`: `DCS:Scripts/World/PersistenceHandlers.lua:5`, `:14`. When DCS calls them is not documented [U]. `world.getPersistenceData`: a comment at `:3` and `MT/world.ts:310` | [R-dcs]; trigger [U] |
| D12 | Per-frame callback | Only in the hooks environment: `onSimulationFrame` (`DCS:API/Sim_ControlAPI.md:591`); `DCS.getModelTime ()` used at `DCS:Scripts/Hooks/webGUI.lua:276`. No mission-environment frame callback was found in shipped Lua (absence, not proof) | [R-dcs] (hooks only) |
| D13 | Hooks → mission bridge | `net.dostring_in` documented (`DCS:API/Sim_ControlAPI.md:303`), gated by `autoexec` (`:318`); `a_do_script` is recommended instead (`:322-325`) | [R-dcs] |
| D14 | DCS's own mission random generator | `regLuaMath`, `luaMathRandomInit` in `DCS:bin/Scripting.dll` | [R-dcs] (exists; behaviour [U]) |
| D15 | Spawn | `coalition.addGroup (countryId, groupCategory, groupData)`: `MT/coalition.ts:71`; `addStaticObject (countryId, objectData)`: `:100`. The `groupData` schema is not declared [U] | [R-ref] |
| D16 | Unit reads | `Unit.getByName` `MT/Unit.ts:396`; `getID (): number \| string` `:301`; `getName` `:311`; `getPoint` `:321` (X north, Y up, Z east, `:316`); `getPosition` `:332`; `getVelocity` `:342`; `inAir` `:284`; `isExist` `:292`; `getLife` `:589`; `getFuel` `:471`; `getAmmo` `:498`; `getGroup` `:416`; `getPlayerName` `:443` | [R-ref] |
| D17 | Controller | `setTask` `MT/Controller.ts:165`; `setOption (optionId, value)` `:141`; `getDetectedTargets` `:29`; `popTask` `:75`; `resetTask` `:94`; `hasTask` `:38`. ROE and reaction-on-threat option names `MTX/AI.export.ts:31`, `:30` | [R-ref] |
| D18 | Route points | `AI.WaypointType` LAND, TAKEOFF, TAKEOFF_PARKING, TAKEOFF_PARKING_HOT, TURNING_POINT: `MTX/AI.export.ts:154-159`; `AI.VehicleFormation.ON_ROAD`: `:150`. The "Mission" route task table is not declared [U] | [R-ref] |
| D19 | Scheduling and clocks | `timer.scheduleFunction (fn (arg, time), arg, modelTime) → id` `MT/timer.ts:86`; `getTime` `:112`; `getAbsTime` (seconds since midnight) `:42`; `getTime0` `:122`; `removeFunction` `:102`; `setFunctionTime` `:60`; `env.mission.date`, `start_time`, `weather`: `MTX/env.export.ts:157`, `:251`, `:278` | [R-ref] |
| D20 | Terrain | `land.getHeight (Vec2)` `MT/land.ts:41`; `getSurfaceType` `:100`; `isVisible` `:111`; `profile` `:122`; `getIP` `:76`. Surface classes LAND, ROAD, RUNWAY, SHALLOW_WATER, WATER: `MTX/land.export.ts:7-11` | [R-ref] |
| D21 | Roads | `land.findPathOnRoads` `MT/land.ts:21`; `getClosestPointOnRoads` `:60` | [R-ref] |
| D22 | Coordinates | `coord.LLtoLO (lat, lon, alt)` `MT/coord.ts:36`; `LOtoLL` `:20`; axes x north, z east (`:108-109`) | [R-ref] |
| D23 | Weather reads | `atmosphere.getWind` `MT/atmosphere.ts:25`; `getTemperatureAndPressure` `:16` | [R-ref] |
| D24 | Airbases and warehouses | `Airbase.getByName` `MT/Airbase.ts:372`; `getParking (available?)` `:204`; `getRunways` `:213`; `setCoalition` `:251`; `autoCapture` `:156`; `getWarehouse` `:355`. Warehouse `getItemCount` `MT/Warehouse.ts:105`; `addItem` `:73`; `removeItem` `:145`; `setItem` `:167`; `getLiquidAmount` `:115`. The `Warehouse` class itself is shipped (`DCS:Scripts/ScriptingSystem.lua:21`) | [R-ref] (methods) |
| D25 | Weapons, search, removal | `Weapon:getLauncher` `MT/Weapon.ts:137`; `getTarget` `:146`; `world.searchObjects` `MT/world.ts:285`; `world.getAirbases` `:338`; `Object:destroy` `MT/Object.ts:16` | [R-ref] |
| D26 | Player messages | `trigger.action.outText (text, displayTime, clearView?)` `MT/trigger.ts:402`; `markToAll` `:495` | [R-ref] |
| D27 | A per-waypoint "reached" callback or script action on an AI route | Not found in the shipped files or the tslua files read. A probe would settle it: a route with a script command at one waypoint, and a check of whether it fires | [U] |

Totals: [R-dcs] 13 (D1–D5, D7–D14; D11 and D14 by their verified part); [R-ref] 13 (D6, D15–D26); [U] 1 (D27). Partial unknowns inside rows with another label: D6's numeric ids, D11's trigger, D14's behaviour, D15's `groupData` schema and D18's route-task table.

## 3. Interactions by category

Columns: **Dir** is C→W (the campaign asks the world) or W→C (the world reports to, or reads from, the campaign). **Fit** is direct, adapted or gap. All EECH sites are [R-src].

### Forces and spawn

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I1 | Create a mobile member in a group | `create_mobile_member`, `faction.c:1678` (the one switch; creators at `faction.c:1757-1803`). Callers: campaign load `parsgen.c:1854`, regen `rg_updt.c:399`, frontline `faction.c:1550` | C→W | entity index, member sub-type, group entity, `vec3d` position; landed members get a landing site | `coalition.addGroup` D15, per group, not per member; parking start D18 | adapted (G1) |
| I2 | Population: cities, sites, scenery as entities with 3D objects | `popread.c:1575` `construct_3d_object`, `:1588` `insert_3d_object_into_terrain` | C→E | 3D object index, position | Map scenery is fixed; `addStaticObject` D15 | gap (G2) |
| I3 | Remove an entity | `destroy_client_server_entity` (`aphavoc/source/entity/system/en_funcs/en_dstry.h:75`) | C→W | entity | `Group:destroy` D8, `Object:destroy` D25 | direct |

### Identity

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I4 | Name and find an entity across the boundary | `get_local_entity_index`, `aphavoc/source/entity/system/en_main/en_heap.h:115`; observed id at `eech-campaign/crates/eech-engine-sys/csrc/eech_observe.c:49` | both | `int` index into the entity heap, reused after destruction (`eech-campaign/crates/eech-engine/src/lib.rs:106`) | Group and unit names D7, D16; `getID (): number \| string` D16 | adapted (G3) |

### Routing, tasking and movement

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I5 | Create, prioritise and assign tasks; build routes | `create_supply_task` `taskgen.c:1638`; `assign_primary_task_to_group` `assign.c:524`; `assign_task_to_group` `assign.c:765`; `create_route` `croute.c:1242` | campaign-only | task, waypoint and guide entities | none needed | direct |
| I6 | Move along the plan | The world reads the guide every frame: formation offset `hc_move.c:1145`; current waypoint and criteria (W→C calls `get_guide_criteria_valid` 14 sites, `get_local_entity_formation_position_offset` 10 sites); the plan advances in `set_guide_new_waypoint` `guide.c:401` | W reads C | guide position, velocity, formation offset, criteria | `Controller:setTask` with a route D17, D18; the route-task table is [U] | gap (G4) |
| I7 | Helicopter fuel load set at assignment | `assign.c:983`; fuel weight `assign.c:1339` | C→W | fuel level from route flight time | Spawn-time group data D15 (field [U]) | adapted (G5) |
| I8 | Attack: weapon choice and release | `ag_attck.c:481` → `aircraft_fire_weapon` `ac_wpn.c:164`; `get_best_weapon_for_target` `engage.c:1138` | C→W | aggressor, target, weapon sub-type | `pushTask ({id = 'AttackGroup'})` D7; ROE option D17 | gap (G6) |
| I9 | Line of sight for attack and cover positions | `gd_cover.c:762` → `check_position_line_of_sight` `mb_tgt.c:138` | C→W | two positions → bool | `land.isVisible` D20 | adapted |
| I10 | Cargo doors and undercarriage | `mb_msgs.c:1822-1843` (#74); `ld_msgs.c:839` | C→W | articulation timers | none needed | direct (dropped; no accounting effect, #74) |
| I11 | Return to base when damaged | `aircraft.c:332`, inside world damage code | W→C | damage threshold → group order | HIT event D6, `getLife`/`getLife0` D16, D9; the campaign applies the rule | adapted (G8) |

### Arrival and waypoints

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I12 | Has the required member reached the waypoint? | `check_guide_reached_waypoint` `guide.c:260` asks each member (`guide.c:302`) → `hc_msgs.c:529`, `fw_msgs.c:390`; verdict `gd_updt.c:108` → `gd_nav.c:79` → `wp_msgs.c:285` | C→W→C | member, guide target, radius → bool | No reached event verified (D27 [U]); `Unit:getPoint`, `inAir` D16 | gap (G7) |
| I13 | Typed arrivals that carry events: landed, lift-off, taken-off, touch-down | `wp_dbase.c:846`, `:900`, `:1494`, `:1656`; handlers `mb_msgs.c:1308` (landed), `mb_msgs.c:2099` (taken off) | W→C | waypoint-reached message per type | LAND, TAKEOFF, RUNWAY_TAKEOFF D5; RUNWAY_TOUCH D6 | adapted (G10) |

### Events (loss, damage, fire)

Section 5 maps the event sources named in the brief (loss, waypoint, landed, takeoff, fire, damage); these rows are the ones not listed under another category. All are raised inside world code today.

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I14 | Member destroyed: force, regen, task losses, group notice | `kill_local` `hc_dstry.c:316` (`:358`, `:410`, `:623`); `mobile.c:427`, `:440` | W→C | entity, aggressor | DEAD, CRASH, UNIT_LOST, KILL D4 | adapted (G8) |
| I15 | Damage | `damage_client_server_entity` `damage.c:97`; `FORCE_DESTROYED` `hc_msgs.c:334` | W→C | damage level, aggressor | HIT D6; `Unit:getLife` D16 | adapted (G8) |
| I16 | Fired at | `wn_creat.c:617` → `fc_msgs.c:317` | W→C | launcher, target | SHOT D6; `Weapon:getTarget` D25 | adapted |
| I17 | Campaign objectives re-checked after a loss | `hc_dstry.c:639` | campaign-only once I14 is observed | force | none needed | direct |

### Time

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I18 | Frame delta driving campaign timers | `get_delta_time` (`modules/system/time.h:106`), 13 campaign sites, e.g. `gp_updt.c:92`, `ks_updt.c:107`, `ts_updt.c:107`; set from the host's ms at `eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c:283-284` | host→C | `float` seconds | `timer.scheduleFunction` and `timer.getTime` differences D19; or the hooks `onSimulationFrame` D12 | adapted (section 6) |
| I19 | Time of day and day | `task.c:1374` (session time of day); `keysite.c:1789` | E→C | seconds of day, day | `timer.getAbsTime`, `env.mission.date` D19 | adapted |
| I20 | Time acceleration | `eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c:255` loops the update `get_time_acceleration ()` times | host | integer multiple | The user controls acceleration; no mission-API call verified | gap (G11) |

### Terrain and geography

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I21 | Ground elevation | `get_3d_terrain_elevation`, 26 sites, e.g. `croute.c:637`, `faction.c:437` | C→E | (x, z) → y | `land.getHeight` D20, after conversion (section 7) | adapted |
| I22 | Surface type (land or water) | `faction.c:1141` | C→E | EECH terrain type → class | `land.getSurfaceType`, 5 classes D20 | adapted |
| I23 | Forest triangles in a sector (seek cover) | `gd_cover.c:1329`, `:1355` | C→E | triangle count and list | none found | gap (G12) |
| I24 | Road graph, route search and broken links | `read_road_route_data` `ai_route.c:293`; `get_road_sub_route` `ai_route.c:898`; `get_closest_road_node` `ai_misc.c:480`; bridge loss `br_msgs.c:131-135` | campaign data from map files; W→C on bridge loss | node and link graph | `land.findPathOnRoads`, `getClosestPointOnRoads` D21; ON_ROAD formation D18 | gap (G13) |
| I25 | Map grid, sectors and map bounds | `aphavoc/source/entity/special/sector/sector.h:105`; `aphavoc/source/entity/system/en_main/en_world.h:141`; `en_world.c:166`; 199 sites | campaign-only | sector indices from (x, z) | none needed, given map extents in DCS coordinates | direct |
| I26 | 3D object data: crate bounds, cover bounds, object index | `keysite.c:377`; `gd_cover.c:544`; `popread.c:807` | C→E | bounding box, radius | none | gap (G14) |

### Environment and weather

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I27 | Set the weather mode at campaign start | `parsgen.c:1205` → `session.c:538` | C→E | weather mode | Mission weather `env.mission.weather` D19; runtime change not verified [U] | gap (G15) |
| I28 | Wind for ATC speech | `landing.c:1080` | C→E | wind speed | `atmosphere.getWind` D23 | adapted |

### Airbases, keysites and supply

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I29 | Keysite supply levels, crates and requests | `ks_updt.c:126`; `update_keysite_cargo` `keysite.c:337` (#74 stage 1) | campaign-only | ammo and fuel percentages | Optional mirror to `Warehouse` D24 | direct |
| I30 | Landing sites, takeoff routes and locks | `response_to_lock_landing_site` `ld_msgs.c:599`; `insert_mobile_into_takeoff_route` `mobile.c:573`; reserve `mb_msgs.c:350` | both | site counts and locks | `Airbase:getParking (true)` D24; DCS ATC sequences | gap (G16) |
| I31 | Keysite structures destroyed or repaired | `st_dstry.c:293` → `keysite.c:739`; repair `keysite.c:1618` → `fixed.c:147` | W→C, C→W | importance and efficiency | DEAD event for a static or scenery initiator D4, D6 | gap (G17) |
| I32 | Capture | `mb_msgs.c:2307` → `capture_keysite` `keysite.c:1289` | W→C | keysite, side | `Airbase:setCoalition`, `autoCapture` D24; BASE_CAPTURED D6 | adapted |
| I33 | Keysite identity | keysite name attribute (89 campaign sites) | both | name string | `Airbase.getByName` D24 | adapted (G3) |
| I34 | Moving keysites (ships) carry their landing sites | `ks_msgs.c:337` | W→C | position, heading, velocity | ship `Unit:getPoint` D16 | adapted |

### Sensing and visibility

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I35 | Fog of war per sector | `sector.c:510`, `sector.c:533`; recon arrival `mb_msgs.c:1951` | campaign-only, fed by arrivals | sector value | depends on I12 | direct |
| I36 | Target detection triggers a campaign engagement | `ac_tgt.c:275` → `engage_targets_in_group` `engage.c:515` | W→C | group, target group | `Controller:getDetectedTargets` D17; DCS AI engages by ROE | gap (G9) |

### Random

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I37 | Random draws | one LCG shared by world and campaign, `modules/maths/random.h:96`; 30 campaign sites, e.g. `taskgen.c:2154`; seeded at `eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c:141`, `:246` | campaign-internal | 32-bit seed | none needed; DCS has its own D14 | direct (G18 classifies the divergence) |

### Persistence, lifecycle, players

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I38 | Save and restore | `pack_session` `en_sessn.c:197`, called at `uisession.c:1113`; restore `parsgen.c:2292`. The pack includes world entities | both | one binary session | `setPersistenceHandler` D11; `io` and `lfs` only through D2 | gap (G19) |
| I39 | Load and boot | `eech-campaign/crates/eech-dc/src/module.rs:57-66`; one boot per process, `eech-campaign/crates/eech-engine/src/lib.rs:169` | host→C | module table, boot configuration | `require` D1 (desanitised) | gap (G20) |
| I40 | Step the campaign | `eech_engine_frame`, `eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c:288` | host→C | ms | D19 or D12 | adapted |
| I41 | Messages and speech to players; gunship mission results | `fc_msgs.c:431` (speech); `gp_msgs.c:1034` | C→W | speech and message ids | `trigger.action.outText` D26 | adapted (multiplayer: `docs/m5-dcs-risks.md`) |
| I42 | Map placement for the reference scenario | Lebanon, map5 (`m2-reference.md`) | host | EECH map | No Syria terrain on this PC | gap (G21) |

Totals: 42 rows. Direct 8 (I3, I5, I10, I17, I25, I29, I35, I37). Adapted 19. Gap 15 (I2, I6, I8, I12, I20, I23, I24, I26, I27, I30, I31, I36, I38, I39, I42). The gap table (section 8) names a workaround for each gap and for the adapted rows that need one.

## 4. Identity and lifecycle

- **EECH:** an entity is a pointer into one heap. Its identity outside the C is its index (`en_heap.h:115`), and that index is **reused after destruction** (`eech-campaign/crates/eech-engine/src/lib.rs:106`). The SUPPLY map notes that guides and tasks can be destroyed mid-notification (#74 stage 4). [R-src]
- **DCS:** groups and units have names (`Group.getByName` D7, `Unit.getByName` D16) and ids (`Unit:getID (): number | string` D16). Events carry object references (`event.initiator`, `event.target`, D4). Whether names are unique per mission, and whether ids are reused, is [U]: a spawn/destroy/respawn probe would settle it.
- **G3, the workaround:** the campaign names every DCS group and unit it spawns as `EECH_<kind>_<index>_<generation>`, keeps a two-way table, and resolves events by name, never by a reused index. Unknown names (players, mission-editor units) are reported as foreign, not guessed.

Lifetimes that DCS lacks, all **campaign-only** [I]: tasks, waypoints, guides, keysites as supply and capacity records, divisions, regen queues, sectors, landing sites and crates. DCS sees only the groups, units and statics they act through. A guide's physical half (the position the flight model steers to) is replaced by the route given to DCS (G4). An airbase keysite maps onto a DCS `Airbase` by name (I33). Other keysites (factories, refineries, SAM sites, cities) have no DCS object and stay campaign records, with statics only where destruction must be observed (G17).

## 5. Events

| EECH source | Where | DCS replacement | Fit |
|---|---|---|---|
| `kill_local`: forces, regen, task losses, `MOBILE_KILLED`, `CHECK_CAMPAIGN_OBJECTIVES` | `hc_dstry.c:316`, `:358`, `:410`, `:623`, `:639`; `fw_dstry.c:306`; `mobile.c:427`, `:440` | DEAD, CRASH, UNIT_LOST, KILL D4 | adapted: G8 |
| Destroyed by damage: `FORCE_DESTROYED` to both forces | `hc_msgs.c:307-336` → `fc_msgs.c:510` | KILL (target and initiator) D4 | adapted: G8 |
| Damage | `damage_client_server_entity`, `damage.c:97`; collision `hc_msgs.c:169` | HIT D6; `Unit:getLife` D16 | adapted |
| Weapon fired at a target | `ENTITY_FIRED_AT` `wn_creat.c:617`, `:999` → `fc_msgs.c:317`, `gp_msgs.c:295` | SHOT D6; `Weapon:getTarget`, `getLauncher` D25 | adapted |
| Waypoint reached | `gd_updt.c:108`, `gd_nav.c:93` | none verified (D27) | gap: G7 |
| Landed | `wp_dbase.c:846` → `mb_msgs.c:1308` | LAND D5 | adapted: G10 |
| Takeoff | `wp_dbase.c:1494` → `mb_msgs.c:2099` | TAKEOFF, RUNWAY_TAKEOFF D5 | adapted: G10 |
| Target detected | `ac_tgt.c:275` | `getDetectedTargets` D17 (polled) | gap: G9 |
| Structure destroyed | `st_dstry.c:293` | DEAD with a static initiator D4 | gap: G17 |
| Capture | `mb_msgs.c:2307` | BASE_CAPTURED D6 (as an observation only) | adapted |

How DCS events are ordered and duplicated (for example, whether one loss yields both DEAD and UNIT_LOST) is [U]. G8 handles that by deduplicating per unit name.

## 6. Time and frame

- **EECH:** the host gives milliseconds; `frame` runs `update_client_server_entities ()` once per time-acceleration step (`eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c:255`). It then advances the headless clock and sets `system_delta_time` (`:283-284`; `modules/system/time.c:162`). So **a frame's delta applies to the next frame's update**. Campaign timers subtract `get_delta_time ()` (13 sites). [R-src]
- **DCS mission environment:** no frame callback was found. `timer.scheduleFunction` reschedules by returning a model time (D19 [R-ref]), and `timer.getTime ()` gives model time. The per-frame `onSimulationFrame` was found only in the hooks environment (D12 [R-dcs]), which is a different Lua state, reachable from the mission side only through D13.
- **Proposal (adapted):** a scheduled function steps the campaign with `dt = getTime () − last`, split into fixed quanta of the size the EECH_WORLD baseline uses (100 ms), and records the dt sequence. Fixed quanta keep campaign timers comparable across hosts. Frame cost and threading are criterion 5 (`docs/m5-dcs-risks.md`).
- **Determinism:** DCS gives no equivalent-input determinism with EECH_WORLD. The world physics differ, so outcomes are comparable, not equal (`AGENT.md`, "Baselines and evidence"). Within DCS, run-to-run repeatability is [U]. The campaign's own random sequence stays deterministic only if world-side draws leave the shared stream (G18).

## 7. Terrain and geography

- **Coordinates.** EECH uses metres with x east, y up, z north (`eech-campaign/crates/eech-engine/src/lib.rs:119`) from a map origin. EECH converts to latitude and longitude with its Tacview series about the origin in `mapinfo.txt` (`tacview.c:171-177`; the same code is in `eech-campaign/crates/eech-map/src/geo.rs`). DCS uses x north, y up, z east (D16, D22) and converts with `coord.LLtoLO` (D22 [R-ref]). The proposed chain is EECH (x, z) → latitude/longitude (EECH geodesy) → `coord.LLtoLO` → DCS. **The retail maps are not metric.** The reference host already fits an affine projection per retail map: Lebanon map5 to eight real airbases, with a median residual of 1.1 km and z compressed to 0.84; Georgia map3 by correlating terrain heights with SRTM, with x stretched 1.22 (`eech-campaign/crates/eech-world/lua/campaign.lua:43-55`). So the chain for a retail map is EECH (x, z) → that affine → latitude/longitude → `coord.LLtoLO` → DCS. Kilometre residuals mean an EECH keysite position and a DCS airbase position will not coincide, and campaign ranges and ETAs are in EECH's own map geometry, not DCS's [I]. Its error against DCS's projection is [U]; G21 measures it at airbases.
- **Elevation and surface:** `land.getHeight` and `land.getSurfaceType` (D20). EECH's many terrain types collapse to DCS's five classes; nothing on the measured campaign paths needs more than land or water except cover search (G12) [I].
- **Roads:** EECH's campaign owns a road graph read from map files (`ai_route.c:293`), with nodes, links and break counts. DCS exposes only path and nearest-point queries (D21), not a graph [R-ref]. See G13.
- **Unknown:** how well the fitted retail affines match DCS's own terrain (they were fitted to real-world data, not to DCS); DCS's road-network coverage of the same area; and the vertical datum difference between EECH's and DCS's elevation.

## 8. Gaps and named workarounds

| Gap | Rows | Workaround | Rests on |
|---|---|---|---|
| G1 Group-level spawn | I1 | **Batched group spawn:** the campaign collects a group's members and spawns them with one `coalition.addGroup`, with names from G3; landed members start at free parking (`getParking (true)`) | D15, D18, D24 [R-ref]; `groupData` schema [U] |
| G2 Population objects | I2 | **Campaign-only population:** cities and sites stay campaign records with positions; no DCS objects | — |
| G3 Identity | I4, I33 | **Generated names with generations** (section 4) | D7 [R-dcs], D16 [R-ref] |
| G4 Guide-pulled movement | I6 | **Route push on waypoint change:** each waypoint change sends the remaining route through `Controller:setTask`; DCS AI handles the formation | D17, D18 [R-ref]; route table [U] |
| G5 Fuel load | I7 | **Spawn-time fuel:** fuel set in the spawn data; no in-flight change | D15 [U] |
| G6 Attack micro-control | I8 | **Target-level tasks:** the campaign chooses the target and issues AttackGroup; DCS AI chooses weapons and releases | D7 [R-dcs], D17 [R-ref] |
| G7 No reached signal | I12, I35 | **Host polling of arrival geometry:** a scheduled tick tests the campaign-named member's `getPoint` and `inAir` against the campaign's target and radius. The campaign keeps waypoint, member and combination rules (#74 seam A) | D16, D19 [R-ref]; D27 [U] |
| G8 Loss and damage inside world code | I11, I14, I15 | **One loss entry point:** separate EECH's campaign bookkeeping from `kill_local` behind a "member destroyed" observation (the #74 second slice), fed from DEAD/CRASH/UNIT_LOST/KILL deduplicated by name, plus an `isExist` sweep as reconciliation | D4 [R-dcs], D16 [R-ref] |
| G9 Detection | I36 | **Detection observations:** poll `getDetectedTargets` per campaign group into the campaign's engage decision; set ROE so DCS AI does not pre-empt campaign tasking. Or accept DCS AI engagement as host behaviour and observe SHOT, HIT and DEAD | D17, D6 [R-ref] |
| G10 Typed arrivals | I13 | **Event-backed arrivals:** LAND and TAKEOFF events, matched to the unit's current campaign waypoint type | D5 [R-dcs] |
| G11 Acceleration | I20 | **Host-owned time rate:** the campaign sees only larger dt values, quantised as in section 6 | D19 [R-ref] |
| G12 Forest cover | I23 | **Drop or precompute:** ground units seek cover from a precomputed per-map table, or the cover search is disabled for DCS. The change is classified as a world variation | — |
| G13 Road graph | I24 | **Offline road graph per DCS map** (as `eech-map` builds Georgia's from OSM), checked against `findPathOnRoads`; ground moves use ON_ROAD route points; bridge breaks stay campaign state | D21, D18 [R-ref] |
| G14 3D object data | I26 | **Adapter object table:** bounds and radii shipped as data with the adapter (`eech-campaign/docs/dcs.md`) | — |
| G15 Weather | I27 | **Weather as a host input:** read the mission weather at start; the campaign does not set it | D19, D23 [R-ref] |
| G16 Landing and takeoff slots | I30 | **Capacity records only:** landing sites stay campaign capacity counts; DCS ATC sequences physical movements; free parking is read at spawn | D24 [R-ref] |
| G17 Keysite structures | I31 | **Statics for structures that matter:** spawn the structures the campaign scores as statics with G3 names, and map their DEAD events to the importance update; repairs respawn them | D15 [R-ref], D4 [R-dcs] |
| G18 Shared random stream | I37 | **Campaign-owned stream:** world draws no longer consume it, so sequences diverge from EECH_WORLD. Classify that as `expected-input-world-variation` | — |
| G19 Save and restore | I38 | **Campaign-state save:** the campaign serialises its own records and G3 table (through D11 if its trigger is confirmed, else a file through D2), and respawns units from them on load. In-flight physical state is approximated (respawn at base or last position). Probe: `docs/m5-dcs-risks.md` | D11 [R-dcs], D2 [R-dcs, desanitised] |
| G20 Load and restart | I39 | **Desanitised `require` for M5 probes, and one boot per DCS process;** a mission restart needs a process restart, or a future reset entry. Probe: `docs/m5-dcs-risks.md` | D1 [R-dcs, desanitised] |
| G21 No Syria terrain here | I42 | **Georgia on Caucasus first:** retail Georgia (map3, with its fitted affine) or the `eech-map` Georgia map (an OSM georeference). Then Lebanon on Syria once that terrain is available. Snap each airbase keysite to its DCS `Airbase` by name (G3) and record the residuals, rather than trusting the affine alone | `DCS:Mods/terrains/` [R-dcs] |

## 9. Six-criterion acceptance matrix

| Criterion | This map contributes | Remains | Proposed acceptance check (where) |
|---|---|---|---|
| 1 DCS call map | The inventory, 42 rows, 27 evidence items, G1–G21 | Lead review; settle D27 and the D6 ids | Review of this record, then a DCS run listing every D item used, as present or absent (DCS) |
| 2 No world inside the campaign | Names the world code that holds campaign rules (I6, I11, I12, I36) and the services to move behind host identities | A mechanical check and a loud failure with no host | Static check: this script's C→W/E edge count, today 64 functions, falls to the calls behind declared host ports; and a boot with no host fails by name (EECH_WORLD) |
| 3 Host events drive the campaign | Event table (section 5); G7, G8 | The causal seam (#74, arrival first) | Withhold or delay one arrival or loss and show campaign state change (EECH_WORLD, then DCS) |
| 4 No regression | Lists which rows change semantics under DCS (G6, G9, G12, G18) | Running the M4 pack after each move | Lebanon under EECH_WORLD, M4 pack holds, differences classified (EECH_WORLD) |
| 5 DCS-only risks | G19, G20, D1, D12, section 6 | Owned by `docs/m5-dcs-risks.md` | Module load, frame budget and threading, save and restore, multiplayer (DCS) |
| 6 Plan ready to build | The gap order and workarounds give the M6 steps | Writing and accepting the plan | Each M6 step has one in-DCS check, reviewed and accepted (DCS) |

## 10. Unknowns and limitations

- **Coverage** is as stated in section 1. Function-pointer dispatch, dynamic messages, calls from framework code and the attribute tokens judged by name are not exhaustively read.
- **The classification is a choice.** Moving `guide/` or `landing/` to W would move their interactions between rows, not remove them.
- **DCS signatures are tslua-dcs's.** No shipped file gives one, the DLL is packed, and no DCS runtime check was made. Unknowns that a DCS probe would settle: D6 numeric ids and event duplication; the `groupData` and route-task schemas; D27; when D11 runs; whether names and ids are unique or reused; D14 behaviour.
- **The coordinate chain is unmeasured against DCS.** The retail maps are distorted (section 7), and there is no Syria terrain on this PC.
- **Human players** (the player-skip and cohesion rules, #74 stage 4) and **multiplayer** are not mapped beyond I41.
- **Prior research:** `eech-campaign/docs/dcs.md` maps the earlier kernel-spike API, not the whole-engine product. Where they differ, this record is source-grounded at `65892de1`.

## Validation

- Scope: documentation only; one new file.
- Inventory: `inventory.py` (below) at `65892de1`, output as in section 1; grep cross-checks run as listed.
- DCS: read-only `grep`, `sed -n` and `diff` over `DCS:Scripts/`, `DCS:API/Sim_ControlAPI.md`, `DCS:autoupdate.cfg` and `DCS:Mods/terrains/`, plus `grep -a` over `DCS:bin/Scripting.dll`. tslua-dcs files were fetched at `1179554f` with `gh api`.
- Citations: a checker kept outside the repository resolves every `file:line` outside code blocks (repository, `DCS:`, `MT/`, `MTX/`; a bare `:N` refers to the last file on the same line) to one file and checks the line is in range; it also checks every relative link and backticked repository path at the commit.
- Not run: no DCS or EECH runtime, no build, no regression pack. Nothing here changes behaviour.

<details>
<summary><code>inventory.py</code></summary>

```python
"""Campaign <-> world call inventory for the M5 DCS call map.
Run from the repository root: python -I inventory.py . [--detail out.json]"""
import collections, json, re, sys
from pathlib import Path

root = Path(sys.argv[1])
detail = "--detail" in sys.argv
sources = [l.strip() for l in (root / "eech-campaign/crates/eech-engine-sys/sources.txt").read_text().splitlines() if l.strip()]

S = "aphavoc/source/"
RULES = [  # first match wins
    ("C", [S + "ai/", S + "ai_extrn.c", S + "entity/mobile/cargo/"] + [S + "entity/special/" + d + "/" for d in
          ("force", "group", "task", "guide", "keysite", "waypoint", "division", "regen", "landing", "sector")]),
    ("W", [S + "entity/mobile/", S + "entity/fixed/", S + "entity/special/bridge/", S + "entity/special/effect/"]),
    ("E", ["modules/3d/", S + "misc/terr_db", S + "misc/tod", S + "misc/timeaccl", S + "entity/special/session/",
           "modules/maths/random.c", "modules/system/time.c", "modules/system/timer.c"]),
    ("X", [S + "ui_menu/", "modules/userint2/", "modules/graphics/", "modules/2d/", "modules/sound/", S + "appsound/",
           S + "app3d/", S + "gunships/", S + "entity/special/camera/", S + "cdmusic.c"]),
    ("F", [""]),
]

def klass(path):
    for k, prefixes in RULES:
        if any(path.startswith(p) for p in prefixes):
            return k

COMMENT = re.compile(r"/\*.*?\*/|//[^\n]*", re.S)
DEF = re.compile(r"^(?:static\s+|extern\s+|inline\s+)*[A-Za-z_][\w \t\*]*?\b([A-Za-z_]\w*)\s*\(([^;{}()]|\([^()]*\))*\)\s*\{", re.M)
CALL = re.compile(r"\b([A-Za-z_]\w*)\s*\(")
ACCESS = re.compile(r"\b(get|set)_(local|client_server)_entity_(\w+?)\s*\(\s*[^,()]+(?:\([^()]*\))?\s*,\s*([A-Z0-9_]+_TYPE_[A-Z0-9_]+)")
MSG = re.compile(r"\bnotify_(?:local|client_server)_entity\s*\(\s*(ENTITY_MESSAGE_[A-Z0-9_]+)")
CREATE = re.compile(r"\bcreate_(?:local|client_server)_entity\s*\(\s*(ENTITY_TYPE_[A-Z0-9_]+)")
KEYWORDS = {"if", "for", "while", "switch", "return", "sizeof", "else", "do", "case"}

def strip(text):  # keep line structure so line numbers survive
    return COMMENT.sub(lambda m: "\n" * m.group(0).count("\n"), text)

text, classes = {}, {}
for f in sources:
    text[f] = strip((root / f).read_text(encoding="latin-1"))
    classes[f] = klass(f)

defs = collections.defaultdict(set)  # name -> classes
for f, t in text.items():
    for m in DEF.finditer(t):
        if m.group(1) not in KEYWORDS:
            defs[m.group(1)].add(classes[f])
ambiguous = sorted(n for n, k in defs.items() if len(k) > 1)

def line(t, pos):
    return t.count("\n", 0, pos) + 1

edges = collections.defaultdict(list)
attrs = collections.defaultdict(list)
msgs = collections.defaultdict(lambda: collections.defaultdict(list))
creates = collections.defaultdict(list)
for f, t in text.items():
    k = classes[f]
    defined_here = {m.start(1) for m in DEF.finditer(t)}
    for m in CALL.finditer(t):
        n = m.group(1)
        if m.start(1) in defined_here or n in KEYWORDS or n not in defs or len(defs[n]) != 1:
            continue
        target = next(iter(defs[n]))
        if (k, target) in {("C", "W"), ("C", "E"), ("W", "C"), ("E", "C")}:
            edges[(k, target, n)].append(f"{f}:{line(t, m.start())}")
    for m in MSG.finditer(t):
        msgs[k][m.group(1)].append(f"{f}:{line(t, m.start())}")
    if k == "C":
        for m in ACCESS.finditer(t):
            attrs[m.group(4)].append(f"{f}:{line(t, m.start())}")
        for m in CREATE.finditer(t):
            creates[m.group(1)].append(f"{f}:{line(t, m.start())}")

TOKENS = {  # world-service tokens (functions, macros, attributes) read in campaign files
    "time": "get_delta_time get_entity_movement_delta_time get_system_time get_time_acceleration FLOAT_TYPE_ELAPSED_TIME FLOAT_TYPE_TIME_OF_DAY get_digital_clock_values get_digital_clock_int_values INT_TYPE_DAY",
    "random": "frand1 sfrand1 rand16 rand8 rand32 get_random_number",
    "terrain": "get_3d_terrain_elevation get_3d_terrain_point_data get_3d_terrain_point_data_type get_3d_terrain_point_data_normal get_3d_terrain_face_normal get_3d_terrain_elevation_and_face_normal get_terrain_type_class get_terrain_3d_number_of_type_triangles_in_sector get_terrain_3d_type_triangles_in_sector",
    "map_grid": "get_x_sector get_z_sector get_local_raw_sector_entity get_local_sector_entity point_inside_map_area point_inside_adjusted_map_area bound_position_to_map_area bound_position_to_adjusted_map_area",
    "roads": "get_road_sub_route get_road_link_breaks set_road_link_breaks get_closest_road_node get_closest_side_road_node get_road_nodes_loaded get_road_link_data read_road_route_data",
    "objects_3d": "get_object_3d_bounding_box get_object_3d_bounding_box_without_lines get_object_3d_radius get_object_3d_index_from_name find_object_3d_sub_object construct_3d_object destruct_3d_object insert_3d_object_into_terrain get_object_3d_troop_landing_position_and_heading get_object_3d_troop_takeoff_route INT_TYPE_OBJECT_3D_SHAPE",
    "weather": "set_session_random_weather_mode set_session_fixed_weather_mode get_speech_atc_wind_speed",
    "sensing": "check_position_line_of_sight check_entity_line_of_sight get_sector_fog_of_war_value update_sector_fog_of_war FLOAT_TYPE_AIR_SCAN_RANGE FLOAT_TYPE_SURFACE_SCAN_RANGE INT_TYPE_HIDDEN_BY_FOG_OF_WAR",
    "physical_state": "VEC3D_TYPE_POSITION FLOAT_TYPE_VELOCITY VEC3D_TYPE_MOTION_VECTOR FLOAT_TYPE_HEADING FLOAT_TYPE_PITCH VEC3D_TYPE_XV VEC3D_TYPE_ZV FLOAT_TYPE_DISTANCE INT_TYPE_LANDED INT_TYPE_AIRBORNE_AIRCRAFT FLOAT_TYPE_MAIN_ROTOR_RPM FLOAT_TYPE_CENTRE_OF_GRAVITY_TO_GROUND_DISTANCE FLOAT_TYPE_UNDERCARRIAGE_STATE FLOAT_TYPE_LOADING_DOOR_TIMER FLOAT_TYPE_FUSELAGE_ANGLE INT_TYPE_ALIVE INT_TYPE_DAMAGE_LEVEL FLOAT_TYPE_FUEL_DEFAULT_WEIGHT FLOAT_TYPE_FUEL_ECONOMY INT_TYPE_INSIDE_HANGAR INT_TYPE_RADAR_ON",
    "range_math": "get_2d_range get_3d_range get_approx_2d_range get_approx_3d_range get_sqr_2d_range get_sqr_3d_range",
}
tokens = {}
for cat, names in TOKENS.items():
    for name in names.split():
        rx = re.compile(r"\b" + name + r"\b")
        sites = [f"{f}:{line(t, m.start())}" for f, t in text.items() if classes[f] == "C" for m in rx.finditer(t)]
        tokens.setdefault(cat, {})[name] = sites

count = collections.Counter(classes.values())
summary = {
    "files_by_class": dict(sorted(count.items())),
    "functions_defined_once_per_class": dict(collections.Counter(next(iter(v)) for v in defs.values() if len(v) == 1)),
    "names_defined_in_several_classes_excluded": len(ambiguous),
    **{f"{a}->{b}": {"functions": sum(1 for e in edges if e[:2] == (a, b)),
                     "sites": sum(len(v) for e, v in edges.items() if e[:2] == (a, b))}
       for a, b in (("C", "W"), ("C", "E"), ("W", "C"), ("E", "C"))},
    "C attribute tokens": {"distinct": len(attrs), "sites": sum(map(len, attrs.values()))},
    "messages sent": {k: {"distinct": len(v), "sites": sum(map(len, v.values()))} for k, v in sorted(msgs.items())},
    "C create_*_entity types": {k: len(v) for k, v in sorted(creates.items())},
    "C service tokens per category [sites, files]": {c: [sum(len(s) for s in v.values()), len({x.rsplit(':', 1)[0] for s in v.values() for x in s})] for c, v in tokens.items()},
}
print(json.dumps(summary, indent=1))
if detail:
    out = {"edges": {f"{a}->{b} {n}": v for (a, b, n), v in sorted(edges.items())},
           "attrs": dict(sorted(attrs.items())), "msgs": {k: dict(sorted(v.items())) for k, v in msgs.items()},
           "creates": dict(creates), "ambiguous": ambiguous, "tokens": tokens}
    Path(sys.argv[sys.argv.index("--detail") + 1]).write_text(json.dumps(out, indent=1))
```

</details>
