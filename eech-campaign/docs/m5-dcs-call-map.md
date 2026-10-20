# M5: the DCS call map

This record answers M5 done-when criterion 1 (Confluence page 11534337, "EECH M5 Goal: EECH_DC into DCS World"):

> **DCS call map.** Every world interaction the campaign needs (creating and routing forces, tasking, events, time, terrain) is mapped to how DCS World's mission scripting environment provides it. Each gap has a named workaround.

It is a source-grounded proposal for review, not an acceptance. It changes no code, build, baseline or DCS configuration, and no DCS or EECH runtime was launched for it. DCS-only runtime risks (native module load, frame budget and threading, save and restore, multiplayer) are criterion 5, owned by `docs/m5-dcs-risks.md` (`eech-campaign/docs/`, in another session's worktree and not yet merged); this record cross-references them and does not probe them.

**In brief.**
- **Two passes over the 1,217 compiled C files plus the 11 headless host files, at `65892de1`.**
  - The first pass (`inventory.py`) measured direct calls between path classes and named world-service tokens.
  - The second pass closes the first pass's coverage holes:
    - **a reachability walk** from the seven public engine entry points, with translation-unit-local name resolution, macro expansion, and address-taken functions and file-scope data counted as references (`reach.py`);
    - **the dispatch tables:** message handlers, entity function tables and attribute accessors (`dispatch.py`);
    - **the shared global data** the campaign touches (`globals.py`).
- **The coverage ledger** (`ledger.py`, section 1) accounts for 940 items: 607 are mapped to an interaction row, 333 are excluded with a reason grounded in the headless product path, and **0 are outstanding**.
  - The items are campaign crossings to or from host, framework, presentation, world and environment code; message types and waypoint message ids; function tables; attribute tokens; global data; and names that resolved to no definition.
  - No needed interaction found by these methods lacks a DCS mechanism or a named workaround.
  - The method's residual limits are in section 10. Accepting criterion 1 remains the technical lead's review decision.
- **50 interactions in 13 categories** are mapped to DCS: 8 fit directly (campaign-only work that needs no DCS call), 25 are adapted and 17 are gaps. Every gap, and every adapted row that needs one, has a named workaround (G1–G27).
- **The largest gaps:**
  - there is **no verified "waypoint reached" signal** in DCS (G7);
  - **movement is pulled from the campaign's guide every frame** in EECH, but DCS AI flies a route it is given (G4);
  - EECH's **loss, damage, target-detection and capture bookkeeping runs inside world code** (G8, G9);
  - **player commands and player aircraft** reach the campaign through EECH's own UI, cockpit and radio code (G24, G27);
  - **no save of physical state** in the mission environment (G19);
  - **one engine per process** against DCS mission restarts (G20);
  - **the retail maps are not metric, and this PC has no Syria terrain** (G21).
- **DCS evidence:** of the 29 DCS evidence items (D1–D29), 14 are confirmed by this install's shipped files ([R-dcs]), 14 rest on tslua-dcs only ([R-ref]) and 1 is unknown ([U]); five items carry a named partial unknown. No shipped file gives a mission-API signature; every signature below is tslua-dcs's.

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

### First pass: measures (`inventory.py`)

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

### What the first pass did not measure

The first pass (revision `18ab088e`) was measured exhaustively only for its own rules: file classes, direct calls between uniquely defined C and W/E functions, static message names and the named service tokens. It left six holes, and the lead's review found a concrete omission through one of them. `create_campaign` calls `read_map_info_data` (`faction.c:156`), which is in a presentation file, `modules/graphics/textuser.c:3255`, and was therefore excluded. The holes were:
1. **Calls to and from F and X files, and the host layer.** `eech-campaign/crates/eech-engine-sys/csrc/` is not in `sources.txt`.
2. **Function-pointer dispatch.** The message handler tables, the entity function tables (for example `kill_client_server_entity`, `aphavoc/source/entity/system/en_funcs/en_dstry.h:107`) and registered callbacks.
3. **Dynamic message ids.** For example the waypoint messages that come from `waypoint_database` (`wp_dbase.c:846`), not from a literal.
4. **The 76 names defined in more than one class**, dropped from the edges.
5. **Attribute tokens** other than the 21 named physical-state ones.
6. **Human-player and multiplayer interactions.**

### Second pass: reachability, dispatch and globals

The scripts are in full at the end; run them side by side from one directory. `reach.py` and `dispatch.py` take the repository root. `ledger.py` takes their JSON output.

| Script | What it does | Result at `65892de1` [R-src] |
|---|---|---|
| `reach.py` | **Indexes** every function defined in the 1,217 `sources.txt` files and the 11 `csrc/*.c` files (class H, the headless host layer), including the bodies that follow `#if` arms and nested function-pointer parameters. **Resolves** each identifier in a body to the definition in the same file, else to the unique non-static global; same-file `#ifdef` twins count as one definition. **Expands** macros from every header and C file under `aphavoc/source`, `modules`, `csrc` and `compat`, recursively. **Walks** from the seven entry points in `eech-campaign/crates/eech-engine-sys/csrc/eech_engine.h:36-102` (boot, frame, objects, clock, prepare_installation, diagnostics, fatal_message). It counts every reference, call or address taken, and the file-scope data of any file whose globals are referenced. This is an over-approximation by design: it reaches registered handlers, callbacks and data-table initialisers without modelling them. | **Functions reachable/defined:** C 997/1,139, W 1,603/1,813, E 799/889, F 914/1,086, H 266/294, X 5,677/6,114. **Ambiguous names left:** 0. The first pass's 76 (for example `kill_local`, defined in many entity files) all resolve once a reference is resolved within its own file first. **Campaign call sites examined:** 14,599. **Call-shaped names in campaign code with no definition:** 3, all accounted for in the ledger. **Distinct non-campaign functions** the campaign calls or references: C→F 122, C→W 38, C→E 25, C→X 10, C→H 6. Distinct non-campaign functions that call or reference the campaign: W 121, X 118, F 42, E 3, H 3 |
| `dispatch.py` | **Reads** every table registration `fn_*[TYPE]… = f` and `message_responses[TYPE][MSG] = f`, including the per-type `[type]` registrations of the shared mobile files. **Classifies** each entity type by the files that register it. **Finds** every static message sent per class with its handler classes, and every non-literal `notify_*_entity` site. For every attribute token in campaign files, it records which classes implement it and in which context the campaign uses it (accessor, creation argument, pack tag, `case` label, registration). | **2,879 registrations; 40 entity types:** W 23, C 11, F 3, X 2, E 1. **30 static message types cross the campaign boundary;** 3 dynamic sends. **22 function tables.** **330 attribute tokens appear in campaign code:** the first pass's 208 from accessor calls, plus 122 in other contexts (creation argument lists, pack tags, `case` labels) |
| `globals.py` | Finds every `extern` variable declared in a header, defined at file scope outside the campaign, and referenced in a campaign function. | 716 externs; 31 non-campaign globals used by the campaign: F 13, W 7, X 7, E 4 |

### The coverage ledger

`ledger.py` gives every item from the three scripts one disposition: **mapped** to an interaction row, **excluded** with a reason grounded in the headless product path, or **outstanding**. Rules are ordered regular expressions over `path::function` or token names; an item that no rule matches is outstanding. Result at `65892de1` [R-src]:

| Ledger | Items | Mapped | Excluded | Outstanding |
|---|---|---|---|---|
| Campaign → host layer (H) | 6 | 0 | 6 | 0 |
| Host layer → campaign | 3 | 3 | 0 | 0 |
| Campaign → framework (F) | 122 | 33 | 89 | 0 |
| Framework → campaign | 42 | 22 | 20 | 0 |
| Campaign → presentation files (X) | 10 | 3 | 7 | 0 |
| Presentation files → campaign | 118 | 97 | 21 | 0 |
| Campaign → world (W) | 38 | 35 | 3 | 0 |
| World → campaign | 121 | 113 | 8 | 0 |
| Campaign → environment (E) | 25 | 24 | 1 | 0 |
| Environment → campaign | 3 | 3 | 0 | 0 |
| Static message types crossing the boundary | 30 | 25 | 5 | 0 |
| Entity function tables | 22 | 11 | 11 | 0 |
| Dynamic message sends | 3 | 3 | 0 | 0 |
| `waypoint_database` message ids | 33 | 31 | 2 | 0 |
| Attribute tokens in campaign code | 330 | 194 | 136 | 0 |
| Non-campaign global data | 31 | 10 | 21 | 0 |
| Call-shaped names with no definition | 3 | 0 | 3 | 0 |
| **Total** | **940** | **607** | **333** | **0** |

**Exclusion reasons, with counts:**
- **Attributes:**
  - 86 are implemented only by campaign entity types.
  - 44 are the campaign entity's own implementation of an attribute (a `case` label, a table registration or a pack tag) and read no world value.
  - 4 are type constants that matched the accessor pattern.
  - 2 are array-size constants.
- **Calls:**
  - 31 entity-store functions used for campaign records. Crossings through the store are the message, table and attribute ledgers.
  - 27 debug or test functions.
  - 27 presentation and debug-output functions. The headless frame draws nothing (`eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c:253-286`), and the host layer has no display or DirectPlay connection (`eech-campaign/crates/eech-engine-sys/csrc/eech_headless_devices.c:5-10`).
  - 25 arithmetic functions.
  - 23 readers of campaign data files. They are read natively by the DLL with C stdio, so no DCS API is involved; the files ship with the module (I39).
  - 8 visual and audio effects.
  - 6 multiplayer replication and client-join functions. The headless engine is `COMMS_MODEL_SERVER` (`eech_engine.c:144`), with DirectPlay replaced (`eech-campaign/crates/eech-engine-sys/build/main.rs:38-39`), and DCS replicates units itself.
  - 6 platform services (memory, strings, fatal unwinding).
  - 2 observers of the campaign: EECH's own Tacview writer and its shared-memory export.
- **Globals:** 16 string tables, 2 PSD image buffers for campaign data, 2 debug-drawing globals, 1 entity-store flag.
- **Messages:** 4 entity-store list messages, and 1 message sent only from debug keys.
- **Tables:** 11 draw, sound and UI tables.
- **Waypoint ids:** 2 with no registered handler, so no-ops (`END`, `WAIT`).
- **Names with no definition:**
  - `#endif` followed by a parenthesised expression (`ld_msgs.c:512`);
  - a call through a function-pointer field (`briefing.c:880`), whose targets are walked as address-taken references;
  - `ogre_scene_get_object`, which is in the `#else` arm of `#ifndef OGRE_EE` (`popread.c:2795`). The headless build does not define `OGRE_EE`.

`python -I ledger.py reach.json dispatch.json globals.json --items items.json` writes every item with its disposition and row. The mapped items cite their rows (I1–I50 below), so every row is backed by a measured site list, not by a sample.

**The SUPPLY rows** lean on #74's map, which is supporting evidence only. The #74 citations reused here were spot-checked at `65892de1` (`assign.c:983`, `keysite.c:377`, `mb_msgs.c:1822`).

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
| D28 | Explosion at a point | `trigger.action.explosion (point, power)` `MT/trigger.ts:281` | [R-ref] |
| D29 | Player menu commands | `missionCommands.addCommand (name, nil, callback)` and `removeItem`, used by the shipped wrappers at `DCS:Scripts/ScriptingSystem.lua:38`, `:42`; per-coalition and per-group variants at `:46`, `:54` | [R-dcs] |

Totals: [R-dcs] 14 (D1–D5, D7–D14, D29; D11 and D14 by their verified part); [R-ref] 14 (D6, D15–D26, D28); [U] 1 (D27). Partial unknowns inside rows with another label: D6's numeric ids, D11's trigger, D14's behaviour, D15's `groupData` schema and D18's route-task table.

## 3. Interactions by category

Columns: **Dir** is C→W (the campaign asks the world) or W→C (the world reports to, or reads from, the campaign). **Fit** is direct, adapted or gap. All EECH sites are [R-src].

### Forces and spawn

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I1 | Create a mobile member in a group | `create_mobile_member`, `faction.c:1678` (the one switch; creators at `faction.c:1757-1803`). Callers: campaign load `parsgen.c:1854`, regen `rg_updt.c:399`, frontline `faction.c:1550` | C→W | entity index, member sub-type, group entity, `vec3d` position; landed members get a landing site | `coalition.addGroup` D15, per group, not per member; parking start D18 | adapted (G1) |
| I2 | Population: cities, sites, scenery as entities with 3D objects | `popread.c:1575` `construct_3d_object`, `:1588` `insert_3d_object_into_terrain` | C→E | 3D object index, position | Map scenery is fixed; `addStaticObject` D15 | gap (G2) |
| I3 | Remove an entity | `destroy_client_server_entity` (`aphavoc/source/entity/system/en_funcs/en_dstry.h:75`) | C→W | entity | `Group:destroy` D8, `Object:destroy` D25 | direct |
| I43 | Campaign kills world units: all members of a group when its keysite falls or is captured | `group_kill_all_members` `group.c:1021`, called at `keysite.c:1097`, `keysite.c:1222`; a keysite below minimum strength `keysite.c:822`; kill and destroy tables (ledger) | C→W | entity | `Group:destroy` D8 (removal); `trigger.action.explosion` D28 (a physical kill). Whether either raises DEAD is [U] | adapted (G26) |
| I48 | Type properties read from world type databases | `aircraft_database` (`faction.c:999`), `vehicle_database`, `weapon_database`, `fixed_3d_object_database` (globals ledger); 28 type-property attribute tokens (cruise speed and altitude, fuel economy, threat values, identify flags, display names) | C reads W | per-sub-type records | `Unit:getDesc` exists (D9); its fields are not verified [U] | adapted (G22) |

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
| I49 | Troop transport: troops on board, insertion, pick-up and capture by troops | `INT_TYPE_TROOPS_ONBOARD` set at `assign.c:1321`; troop waypoint handlers such as `mb_msgs.c:2370` (7 troop waypoint ids, ledger) | both | troop count; waypoint messages | No embark or disembark API verified [U] | gap (G23) |

### Arrival and waypoints

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I12 | Has the required member reached the waypoint? | `check_guide_reached_waypoint` `guide.c:260` asks each member (`guide.c:302`) → `hc_msgs.c:529`, `fw_msgs.c:390`; verdict `gd_updt.c:108` → `gd_nav.c:79` → `wp_msgs.c:285` | C→W→C | member, guide target, radius → bool | No reached event verified (D27 [U]); `Unit:getPoint`, `inAir` D16 | gap (G7) |
| I13 | Typed arrivals that carry events: landed, lift-off, taken-off, touch-down | `wp_dbase.c:846`, `:900`, `:1494`, `:1656`; handlers `mb_msgs.c:1308` (landed), `mb_msgs.c:2099` (taken off) | W→C | waypoint-reached message per type | LAND, TAKEOFF, RUNWAY_TAKEOFF D5; RUNWAY_TOUCH D6 | adapted (G10) |
| I50 | The waypoint's own message, chosen at runtime from `waypoint_database` and handled in world files | Dynamic sends `wp_msgs.c:271` (action message) and `wp_msgs.c:440` (reached message, to the task leader); 33 ids in `wp_dbase.c`, 31 with handlers registered by the mobile types | C→W | message id per waypoint type | Per id (ledger): aircraft sequence ids → G10; navigation, convoy, defend and supply ids → G7; troop ids → G23; recon → G7 with I35; attack action → G6 | adapted |

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
| I46 | Map info and the map's geographic origin, read when the campaign is created | `create_campaign` → `read_map_info_data` (`faction.c:156` → `textuser.c:3255`) → `initialize_known_coordinates` (`textuser.c:3291`). Per-map source defaults, e.g. Lebanon map5 at 32.86, 35.01 (`textuser.c:3187-3189`), overridden by a `coordinate` line in `mapinfo.txt` (`textuser.c:3370-3376`). The only reader of the origin is EECH's Tacview writer (`get_current_map_latitude_offset`, `textuser.c:3153-3161` → `tacview.c:171-172`) | C→X (map data) | map number, latitude and longitude of map (0, 0) | none for campaign decisions; the adapter converts positions (section 7) | adapted (G21) |

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
| I47 | Campaign end: winner, session complete, player log | `campaign_completed`, `campaign.c:1096`, called from `fc_msgs.c:307` | C→X | side, completion type | `outText` D26; no mission-end call verified [U] | gap (G25) |

### Players and multiplayer

The headless product boots as a server, with no DirectPlay connection and no joystick or display devices (`eech_engine.c:144`; `eech-campaign/crates/eech-engine-sys/csrc/eech_headless_devices.c:5-10`). These paths are therefore not exercised there. They are still interactions that the campaign consumes, and the supported DCS destination is server-authoritative with human players.

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I44 | Player commands and tasking: radio commands to wingmen and support (attack my target, hold, rejoin, return to base, airstrike, artillery, assistance); picking a task in the campaign screen or free flight; skipping a waypoint; key-bound campaign triggers; a client's task request | `process_radio_message` `msg_in.c:965`; `ca_selct.c:1083`; `fflight.c:504`; `co_wp.c:105`; `ev_fns.c:203`; `en_comms.c:5352` (43 ledger items) | player→C | message type and value; task; group | `missionCommands.addCommand` with a callback D29 | adapted (G24) |
| I45 | The player's aircraft as a campaign member: pilot entity, player control of a group member, its landing and takeoff against landing locks, mission results | `create_new_pilot_entity` `pilot.c:271`; `helicopter_assume_player_control` `helicop.c:1972`; `dynamics.c:2017-2021`; `gp_msgs.c:1034` (18 ledger items) | both | player flag, aircrew lists, landing entity | PLAYER_ENTER_UNIT and PLAYER_LEAVE_UNIT D6; `getPlayerName` D16; LAND and TAKEOFF D5 | adapted (G27) |
| — | Multiplayer replication of campaign state to clients, and client join | `transmit_entity_comms_message` (82 campaign functions, 93 raw `grep` call sites); `comm_man.c` client join (ledger) | C→clients | packed entity updates | none needed: DCS replicates units; campaign state stays on the server | excluded: server-authoritative destination; the headless engine has no clients (`eech-campaign/crates/eech-engine-sys/build/main.rs:38-39`) |
| — | Player displays of campaign state: HUD and MFD waypoints, task objective text, briefings | 64 cockpit, avionics and view functions under `aphavoc/source/gunships/` (ledger, I41) | C→player | strings, waypoint positions | `outText`, `markToAll` D26 | adapted (with I41) |

Totals: 50 rows. Direct 8 (I3, I5, I10, I17, I25, I29, I35, I37). Adapted 25. Gap 17 (I2, I6, I8, I12, I20, I23, I24, I26, I27, I30, I31, I36, I38, I39, I42, I47, I49). The gap table (section 8) names a workaround for each gap and for the adapted rows that need one. The two un-numbered player rows record ledger groups and are not counted.

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

- **Coordinates: what the source does.**
  - EECH uses metres with x east, y up, z north (`eech-campaign/crates/eech-engine/src/lib.rs:119`) from a map origin.
  - The origin is set on the campaign-creation path (I46). `read_map_info_data` takes the map number from the data path and calls `initialize_known_coordinates` (`textuser.c:3291`), which sets per-map **source defaults**: map5, Lebanon, 32.86 N 35.01 E (`textuser.c:3187-3189`). A `coordinate=lat,lon` line in the map's `mapinfo.txt` overrides them (`textuser.c:3370-3376`).
  - In the compiled code, the origin's only reader is EECH's Tacview writer (`tacview.c:171-177`), which converts with its own series about the origin. The same code is in `eech-campaign/crates/eech-map/src/geo.rs`.
  - `grep` finds no campaign file that reads `current_map_info` or the origin getters, so no campaign decision depends on the origin. [R-src]
- **Coordinates: how accurate the conversion is.** These are separate facts from the source defaults.
  - DCS uses x north, y up, z east (D16, D22) and converts with `coord.LLtoLO` (D22 [R-ref]).
  - The naive chain is EECH (x, z) → latitude/longitude (EECH geodesy about the default origin) → `coord.LLtoLO` → DCS. **The retail maps are not metric**, so the naive chain is wrong by kilometres.
  - The comments in the reference host's scenario table say that EECH's own Tacview origin puts map5 airbases up to 17 km out, and map3's up to 50 km. They say the host therefore fits an affine projection per retail map instead (`eech-campaign/crates/eech-world/lua/campaign.lua:43-55`):
    - Lebanon map5 is fitted to eight real airbases, with a median residual of 1.1 km and z compressed to 0.84.
    - Georgia map3 is fitted by correlating terrain heights with SRTM, with x stretched 1.22.

    These figures are reported in those source comments. They were not reproduced here.
  - So the chain for a retail map is EECH (x, z) → that affine → latitude/longitude → `coord.LLtoLO` → DCS.
  - Kilometre residuals mean an EECH keysite position and a DCS airbase position will not coincide. Campaign ranges and ETAs stay in EECH's own map geometry, not DCS's [I].
  - The chain's error against DCS's terrain is [U]; G21 measures it at airbases.
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
| G21 Map placement and coordinates | I42, I46 | **Georgia on Caucasus first:** retail Georgia (map3, with its fitted affine) or the `eech-map` Georgia map (an OSM georeference). Then Lebanon on Syria once that terrain is available. Snap each airbase keysite to its DCS `Airbase` by name (G3) and record the residuals, rather than trusting the affine alone. Never use the source-default origin (`textuser.c:3187-3189`) for placement | `DCS:Mods/terrains/` [R-dcs]; D22 [R-ref] |
| G22 Type database | I48 | **Adapter type table:** each EECH sub-type maps to a DCS type name and the properties the campaign reads (cruise speed and altitude, fuel, threat, points), kept as data next to G14. Where `Unit:getDesc` fields are verified, they cross-check the table | D9 [R-dcs]; fields [U] |
| G23 Troop transport | I49 | **Abstract troops:** troops stay a campaign count on the carrier. Insertion spawns an infantry group at the drop point (`coalition.addGroup`), and pick-up removes one (`Group:destroy`). No DCS embark is relied on | D15 [R-ref], D8 [R-dcs] |
| G24 Player commands | I44 | **F10 command menu:** each EECH radio command, task request and waypoint skip becomes a `missionCommands` entry whose callback calls the campaign's existing handler (for example `process_radio_message`) with the player's group, resolved through G3 | D29 [R-dcs] |
| G25 Campaign end | I47 | **Host-owned end:** the campaign reports the winner through `outText` and an observable state. The host ends or keeps the mission; no mission-end API is relied on | D26 [R-ref] |
| G26 Kill or remove | I43 | **Campaign-recorded kills:** when the campaign itself kills a group, it records the losses directly, as EECH's own consequence, and removes the units with `Group:destroy`. It does not wait for a DEAD event that may not come | D8 [R-dcs]; DEAD on destroy [U] |
| G27 Player aircraft binding | I45 | **Player-unit binding:** a player's DCS unit is bound to an EECH group member through G3 names on PLAYER_ENTER_UNIT. Its landings and takeoffs arrive as LAND and TAKEOFF events, applied with the campaign's landing-lock rules | D6 [R-ref], D5 [R-dcs], D16 [R-ref] |

## 9. Six-criterion acceptance matrix

| Criterion | This map contributes | Remains | Proposed acceptance check (where) |
|---|---|---|---|
| 1 DCS call map | Two-pass inventory and a 940-item coverage ledger (0 outstanding); 50 rows; 29 evidence items; G1–G27 | Lead review of the ledger rules and the method's residual limits (section 10: macro-heavy or `#if` code the regex index might misread; registrations or sends written in forms the scripts do not match). DCS-side unknowns that do not block the mapping but need probes: D6 ids, D11 trigger, D15 and D18 schemas, D27 | Review of this record; then a DCS run listing every D item used, as present or absent (DCS) |
| 2 No world inside the campaign | Names the world code that holds campaign rules (I6, I11, I12, I36) and the services to move behind host identities | A mechanical check and a loud failure with no host | Static check: `reach.py`'s campaign→world and campaign→environment callee count (today 38 and 25 functions) falls to the calls behind declared host ports, with the ledger re-run; and a boot with no host fails by name (EECH_WORLD) |
| 3 Host events drive the campaign | Event table (section 5); G7, G8 | The causal seam (#74, arrival first) | Withhold or delay one arrival or loss and show campaign state change (EECH_WORLD, then DCS) |
| 4 No regression | Lists which rows change semantics under DCS (G6, G9, G12, G18) | Running the M4 pack after each move | Lebanon under EECH_WORLD, M4 pack holds, differences classified (EECH_WORLD) |
| 5 DCS-only risks | G19, G20, D1, D12, section 6 | Owned by `docs/m5-dcs-risks.md` | Module load, frame budget and threading, save and restore, multiplayer (DCS) |
| 6 Plan ready to build | Interaction rows and workarounds for the steps of the existing working draft, [EECH M5 migration plan and acceptance evidence](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/12484650) (Confluence 12484650, version 1, not reviewed). Steps 2 and 3 use I1–I13 and G1–G7; step 4 uses section 5 and G7, G8; step 5 uses section 6; step 6 uses G19; step 7 uses I44, I45 and the excluded replication group | Attaching this map to the draft, mapping each step to its rows, and the lead's review and acceptance of the plan | That draft's in-DCS checks (DCS-load, DCS-bind, DCS-task, DCS-causal, DCS-clock-budget, DCS-restore, DCS-multiplayer, DCS-campaign) (DCS) |

## 10. Unknowns and limitations

- **Coverage.** The ledger has no outstanding item, but it rests on a regex index, not a compiler. These are the residual limits a reviewer should test:
  - **No preprocessor.** Both arms of `#ifdef` count, and only `#if 0` is removed. This over-approximates; it does not drop code.
  - **Function definitions** are found by pattern. The check that 14,599 call sites in reachable campaign functions resolve, with 3 explained exceptions, bounds the risk on the campaign side only.
  - **Registrations and message sends** written in forms other than `fn_*[...] = f`, `message_responses[...][...] = f` and `notify_*_entity (...)` would be missed.
  - **The attribute disposition** uses the implementing entity classes and the usage context. Of the world-implemented tokens used in accessors, each was assigned to a row by name in `ledger.py`; that assignment is a judgement the reviewer can check line by line.
  - **Over-approximation:** reachability makes almost all code reachable, because the headless boot initialises the in-game UI (`start_ingame_ui_system`, `eech_engine.c:212`). Exclusions therefore rest on the stated product-path reasons, not on unreachability.
- **The classification is a choice.** Moving `guide/` or `landing/` to W would move their interactions between rows, not remove them.
- **DCS signatures are tslua-dcs's.** No shipped file gives one, the DLL is packed, and no DCS runtime check was made. Unknowns that a DCS probe would settle: D6 numeric ids and event duplication; the `groupData` and route-task schemas; D27; when D11 runs; whether names and ids are unique or reused; D14 behaviour.
- **The coordinate chain is unmeasured against DCS.** The retail maps are distorted (section 7), and there is no Syria terrain on this PC.
- **Human players and multiplayer** are now mapped (I44, I45, the replication group). The player-skip and cohesion rules inside the helicopter arrival test (#74 stage 4) still sit in world code and move with G7. No player path runs in the headless product, so none of this is runtime-characterised.
- **Prior research:** `eech-campaign/docs/dcs.md` maps the earlier kernel-spike API, not the whole-engine product. Where they differ, this record is source-grounded at `65892de1`.

## Validation

- **Scope:** documentation only. This revision edits only this file.
- **First pass:** `inventory.py` at `65892de1`; the output is as in section 1, and the grep cross-checks were run as listed.
- **Second pass:** from one directory holding the four scripts below, run:
  ```
  python -I reach.py <repo> --detail reach.json
  python -I dispatch.py <repo> --detail dispatch.json
  python -I globals.py <repo> --detail globals.json
  python -I ledger.py reach.json dispatch.json globals.json --items items.json
  ```
  These were run at `65892de1` with Python 3.13 on Windows and give the figures in section 1, including a ledger of 940 items with 0 outstanding.
- **DCS:** read-only `grep`, `sed -n` and `diff` over `DCS:Scripts/`, `DCS:API/Sim_ControlAPI.md`, `DCS:autoupdate.cfg` and `DCS:Mods/terrains/`, plus `grep -a` over `DCS:bin/Scripting.dll`. tslua-dcs files were fetched at `1179554f` with `gh api`.
- **Confluence:** the plan draft 12484650 (version 1) was read; it was not edited.
- **Citations:** a checker kept outside the repository resolves every `file:line` outside code blocks (repository, `DCS:`, `MT/`, `MTX/`; a bare `:N` refers to the last file on the same line) to one file and checks the line is in range. It also checks every relative link and backticked repository path at the commit.
- **Not run:** no DCS or EECH runtime, no build and no regression pack. Nothing here changes behaviour.

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

<details>
<summary><code>reach.py</code></summary>

```python
"""Reachability walk over the headless eech_dc C, for the M5 DCS call map.

python -I reach.py <repo-root> [--detail out.json]

Nodes: every function defined in a compiled C file (sources.txt + csrc/*.c),
keyed by (file, name); plus one data node per file (its file-scope text).
Edges: any identifier reference in a body (call or address taken), resolved
same-file first, then to the unique non-static global definition; a
function-like or object macro is expanded (identifiers in its body,
recursively). #if 0 blocks and comments are removed. Over-approximate by
design: an address taken counts as a call, both arms of #ifdef count.
"""
import collections, json, re, sys
from pathlib import Path

root = Path(sys.argv[1])
SYS = "eech-campaign/crates/eech-engine-sys/"
sources = [l.strip() for l in (root / SYS / "sources.txt").read_text().splitlines() if l.strip()]
csrc = sorted(p.relative_to(root).as_posix() for p in (root / SYS / "csrc").glob("*.c"))
units = sources + csrc

S = "aphavoc/source/"
RULES = [
    ("H", [SYS + "csrc/"]),
    ("C", [S + "ai/", S + "ai_extrn.c", S + "entity/mobile/cargo/"] + [S + "entity/special/" + d + "/" for d in
          ("force", "group", "task", "guide", "keysite", "waypoint", "division", "regen", "landing", "sector")]),
    ("W", [S + "entity/mobile/", S + "entity/fixed/", S + "entity/special/bridge/", S + "entity/special/effect/"]),
    ("E", ["modules/3d/", S + "misc/terr_db", S + "misc/tod", S + "misc/timeaccl", S + "entity/special/session/",
           "modules/maths/random.c", "modules/system/time.c", "modules/system/timer.c"]),
    ("X", [S + "ui_menu/", "modules/userint2/", "modules/graphics/", "modules/2d/", "modules/sound/", S + "appsound/",
           S + "app3d/", S + "gunships/", S + "entity/special/camera/", S + "cdmusic.c"]),
    ("F", [""]),
]
def klass(p):
    return next(k for k, ps in RULES if any(p.startswith(x) for x in ps))

COMMENT = re.compile(r"/\*.*?\*/|//[^\n]*", re.S)
STRING = re.compile(r'"(?:\\.|[^"\\\n])*"|\'(?:\\.|[^\'\\\n])*\'')
IDENT = re.compile(r"\b[A-Za-z_]\w*\b")
DEF = re.compile(r"^(?:static\s+|extern\s+|inline\s+|__inline\s+)*[A-Za-z_][\w \t\*]*?\b([A-Za-z_]\w*)\s*\(([^;{}()]|\((?:[^()]|\([^()]*\))*\))*\)\s*(?:#[^\n]*\n\s*)*\{", re.M)
KEYWORDS = set("if for while switch return sizeof else do case break continue goto default typedef struct union enum static extern const volatile register unsigned signed int char float double long short void".split())

def blank(m):
    return re.sub(r"[^\n]", " ", m.group(0))

def if0(text):
    out, depth, skipping = [], 0, 0
    for line in text.split("\n"):
        s = line.strip()
        if skipping:
            if re.match(r"#\s*if", s): depth += 1
            elif re.match(r"#\s*endif", s):
                depth -= 1
                if depth == 0: skipping = 0
            elif re.match(r"#\s*(else|elif)", s) and depth == 1: skipping = 0
            out.append("")
            continue
        if re.match(r"#\s*if\s+0\b", s):
            skipping, depth = 1, 1
            out.append("")
            continue
        out.append(line)
    return "\n".join(out)

def clean(text):
    text = COMMENT.sub(blank, text)
    text = STRING.sub(blank, text)
    return if0(text)

# macros from every header and C file under the trees the build includes
macros = collections.defaultdict(set)
MACRO = re.compile(r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)(\([^)]*\))?((?:[^\n\\]|\\\n|\\.)*)", re.M)
for top in ("aphavoc/source", "modules", SYS + "csrc", SYS + "compat"):
    for p in (root / top).rglob("*"):
        if p.suffix.lower() in (".h", ".c"):
            t = COMMENT.sub(blank, p.read_text(encoding="latin-1"))
            for m in MACRO.finditer(t):
                params = set(IDENT.findall(m.group(2) or ""))
                macros[m.group(1)] |= set(IDENT.findall(m.group(3))) - params - KEYWORDS
_mx = {}
def expand(name, seen=()):
    if name in _mx: return _mx[name]
    out = set()
    for n in macros.get(name, ()):
        if n in macros and n not in seen and n != name:
            sub = expand(n, seen + (name,))
            out |= sub
            if not any(x in by_name for x in sub):
                out.add(n)  # the macro hides a same-named function whose variant is not defined
        else:
            out.add(n)
    _mx[name] = out
    return out

text, cls = {}, {}
for f in units:
    text[f] = clean((root / f).read_text(encoding="latin-1"))
    cls[f] = klass(f)

def body_end(t, i):
    depth = 0
    for j in range(i, len(t)):
        if t[j] == "{": depth += 1
        elif t[j] == "}":
            depth -= 1
            if depth == 0: return j
    return len(t)

funcs = {}          # (file, name) -> (start, end, line, static)
by_name = collections.defaultdict(list)
filescope = {}
for f, t in text.items():
    spans = []
    for m in DEF.finditer(t):
        name = m.group(1)
        if name in KEYWORDS: continue
        if spans and m.start() < spans[-1][1]: continue  # nested match inside a body
        b = t.index("{", m.end() - 1)
        e = body_end(t, b)
        static = bool(re.match(r"(?:inline\s+|__inline\s+)*static\b|static\b", t[m.start():m.start(1)].strip()))
        funcs[(f, name)] = (b, e, t.count("\n", 0, m.start(1)) + 1, static)
        by_name[name].append((f, static))
        spans.append((m.start(), e))
    fs, last = [], 0
    for s0, e0 in spans:
        fs.append(t[last:s0]); last = e0 + 1
    fs.append(t[last:])
    filescope[f] = "".join(fs)

def resolve(f, n):
    if (f, n) in funcs: return [(f, n)]
    g = list({x[0]: x for x in by_name.get(n, ()) if not x[1]}.values())  # #ifdef'd twins in one file are one definition
    if len(g) == 1: return [(g[0][0], n)]
    if len(g) > 1: return [("?ambiguous", n)]
    return []

def refs(f, t, lo, hi):
    out = []
    for m in IDENT.finditer(t, lo, hi):
        n = m.group(0)
        if n in KEYWORDS: continue
        line = t.count("\n", 0, m.start()) + 1
        names = expand(n) if n in macros and (f, n) not in funcs else {n}
        got = [(r, line, n) for x in names for r in resolve(f, x)]
        if n in macros and not any(r[0] != "?ambiguous" for r, _, _ in got):
            got += [(r, line, n) for r in resolve(f, n)]  # a macro hiding a same-named function (e.g. #ifdef'd _ex variants)
        out += got
    return out

data_decl = {f: set(IDENT.findall(fs)) for f, fs in filescope.items()}
def data_node(f): return (f, "<file-scope data>")

ENTRIES = ["eech_engine_boot", "eech_engine_frame", "eech_engine_objects", "eech_engine_clock",
           "eech_engine_prepare_installation", "eech_engine_diagnostics", "eech_engine_fatal_message"]
start = [(SYS + "csrc/eech_engine.c", e) for e in ENTRIES]
parent = {s: None for s in start}
queue = collections.deque(start)
edges = collections.defaultdict(list)   # (from, to) -> sites
ambiguous = collections.Counter()
globals_by_file = collections.defaultdict(set)
for f, fs in filescope.items():
    for m in re.finditer(r"\b([A-Za-z_]\w*)\s*(?:\[[^\]]*\]\s*)*(?:=|;|,)", fs):
        globals_by_file[m.group(1)].add(f)
while queue:
    node = queue.popleft()
    f, n = node
    t = text[f]
    if n == "<file-scope data>":
        rs = refs(f, filescope[f], 0, len(filescope[f]))
        rs = [(r, 0, x) for r, _, x in rs]
    else:
        b, e, _, _ = funcs[node]
        rs = refs(f, t, b, e)
        # data referenced: the file-scope data node of the file that declares it
        for m in IDENT.finditer(t, b, e):
            for g in globals_by_file.get(m.group(0), ()):
                d = data_node(g)
                if d not in parent:
                    parent[d] = node; queue.append(d)
    for r, line, via in rs:
        if r[0] == "?ambiguous":
            ambiguous[r[1]] += 1; continue
        edges[(node, r)].append((line, via))
        if r not in parent:
            parent[r] = node; queue.append(r)

reach = {k for k in parent if k[1] != "<file-scope data>"}
# call-shaped names in reachable campaign functions that resolve to no function and no macro
LIBC = set("""abs acos asin atan atan2 atof atoi atol ceil cos exp fabs fclose feof fflush fgetc fgets floor fmod fopen fprintf fputc fputs fread free fscanf fseek ftell fwrite getc isalpha isdigit isspace log log10 malloc memcmp memcpy memmove memset pow printf qsort rand realloc sin sprintf sqrt srand sscanf strcat strchr strcmp strcpy strcspn strlen strncat strncmp strncpy strrchr strstr strtok tan tolower toupper va_arg va_end va_start vsprintf snprintf vsnprintf time exit strdup stricmp strnicmp _stricmp _strnicmp sqrtf fabsf floorf ceilf powf atan2f sinf cosf""".split())
unresolved = collections.Counter()
examined = 0
for f, n in reach:
    if cls[f] != "C": continue
    b, e, _, _ = funcs[(f, n)]
    for m in re.finditer(r"\b([A-Za-z_]\w*)\s*\(", text[f][b:e]):
        x = m.group(1)
        examined += 1
        if x in KEYWORDS or x in macros or x in LIBC or resolve(f, x): continue
        if re.search(r"\(\s*\*?\s*" + x + r"\s*\)", text[f][b:e]): continue
        unresolved[x] += 1
def cnt(keys): return collections.Counter(cls[f] for f, _ in keys)
cross = collections.defaultdict(list)
for (a, b), sites in edges.items():
    if a[1] == "<file-scope data>" or b[1] == "<file-scope data>": pass
    ka, kb = cls[a[0]], cls[b[0]]
    if ka != kb and "C" in (ka, kb):
        cross[(ka, kb)].append((a, b, sites[0][0], sites[0][1]))
summary = {
    "units": dict(sorted(collections.Counter(cls.values()).items())),
    "functions": dict(sorted(cnt(funcs).items())),
    "reachable_functions": dict(sorted(cnt(reach).items())),
    "reachable_files": dict(sorted(collections.Counter(cls[f] for f in {f for f, _ in reach}).items())),
    "unresolved_ambiguous_names": len(ambiguous),
    "call_sites_examined_in_campaign": examined,
    "unresolved_call_names_in_campaign": sorted(unresolved),
    "campaign_cross_edges": {f"{a}->{b}": [len({(x[0], x[1]) for x in v}), len({x[1] for x in v})] for (a, b), v in sorted(cross.items())},
}
print(json.dumps(summary, indent=1))
if "--detail" in sys.argv:
    def path(n):
        p = []
        while n is not None:
            p.append(f"{n[0]}::{n[1]}"); n = parent.get(n)
        return p[::-1]
    out = {
        "cross": {f"{a}->{b}": sorted({(x[0][0] + "::" + x[0][1], x[1][0] + "::" + x[1][1], x[2], x[3]) for x in v}) for (a, b), v in cross.items()},
        "ambiguous": dict(ambiguous),
        "unresolved_names": sorted(unresolved),
        "reachable": sorted(f"{f}::{n}" for f, n in reach),
        "unreachable": sorted(f"{f}::{n}" for f, n in funcs if (f, n) not in reach),
        "paths": {f"{f}::{n}": path((f, n)) for f, n in reach if cls[f] in "XH" or n in ("read_map_info_data", "initialize_known_coordinates")},
        "funcs": {f"{f}::{n}": v[2] for (f, n), v in funcs.items()},
    }
    Path(sys.argv[sys.argv.index("--detail") + 1]).write_text(json.dumps(out, indent=1))
```

</details>

<details>
<summary><code>dispatch.py</code></summary>

```python
"""Dispatch tables of the headless eech_dc: message handlers, entity function
tables and attribute accessors, and which classes register them.

python -I dispatch.py <repo-root> [--detail out.json]
Reuses reach.py's classification and comment stripping (imported by path).
"""
import collections, json, re, sys, importlib.util
from pathlib import Path

root = Path(sys.argv[1])
SYS = "eech-campaign/crates/eech-engine-sys/"
spec_src = (Path(__file__).parent / "reach.py").read_text()
klass_src = spec_src[spec_src.index("S = \"aphavoc/source/\""):spec_src.index("COMMENT = ")]
ns = {"SYS": SYS}
exec(klass_src, ns)
klass = ns["klass"]
COMMENT = re.compile(r"/\*.*?\*/|//[^\n]*", re.S)

sources = [l.strip() for l in (root / SYS / "sources.txt").read_text().splitlines() if l.strip()]
units = sources + sorted(p.relative_to(root).as_posix() for p in (root / SYS / "csrc").glob("*.c"))
text = {f: COMMENT.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), (root / f).read_text(encoding="latin-1")) for f in units}
cls = {f: klass(f) for f in units}
def ln(t, i): return t.count("\n", 0, i) + 1

REG = re.compile(r"\b(fn_\w+|message_responses)\s*\[\s*(ENTITY_TYPE_\w+|[a-z_]\w*)\s*\](?:\s*\[\s*(\w+)\s*\])?\s*=\s*(\w+)\s*;")
regs = []          # table, entity type, key, handler, file, line
for f, t in text.items():
    for m in REG.finditer(t):
        et = m.group(2) if m.group(2).startswith("ENTITY_TYPE_") else "var:" + cls[f]  # overload_*(type) in a shared file
        regs.append((m.group(1), et, m.group(3), m.group(4), f, ln(t, m.start())))

# entity type -> class: the class of the files that register its message handlers and functions
type_cls = collections.defaultdict(collections.Counter)
for table, et, key, h, f, l in regs:
    type_cls[et][cls[f]] += 1
etype = {et: (et[4:] if et.startswith("var:") else c.most_common(1)[0][0]) for et, c in type_cls.items()}

# message handlers per message
handlers = collections.defaultdict(set)
for table, et, key, h, f, l in regs:
    if table == "message_responses":
        handlers[key].add(et)

# static senders per message and class, dynamic sends
SEND = re.compile(r"\bnotify_(?:local|client_server)_entity\s*\(\s*((?:\([^()]*\)\s*)?[^,()]+(?:\([^()]*\))?)\s*,")
static = collections.defaultdict(lambda: collections.defaultdict(list))
dynamic = []
for f, t in text.items():
    for m in SEND.finditer(t):
        arg = m.group(1).strip()
        site = f"{f}:{ln(t, m.start())}"
        arg = re.sub(r"^\(\s*entity_messages\s*\)\s*", "", arg)
        if re.fullmatch(r"ENTITY_MESSAGE_\w+", arg):
            static[arg][cls[f]].append(site)
        elif not t[max(0, m.start() - 40):m.start()].rstrip().endswith(("define", "int", "void")):
            dynamic.append((site, cls[f], arg))

crossing = {}
for msg, by in static.items():
    hcls = sorted({etype.get(et, "?") for et in handlers.get(msg, ())})
    for sc, sites in by.items():
        if (sc == "C" and set(hcls) - {"C"}) or (sc != "C" and "C" in hcls):
            crossing.setdefault(msg, {})[sc] = {"sites": len(sites), "first": sites[0], "handler_classes": hcls}

# attribute accessors: which entity classes implement get/set per attribute
attr_impl = collections.defaultdict(lambda: collections.defaultdict(set))
for table, et, key, h, f, l in regs:
    m = re.fullmatch(r"fn_(get|set)_(?:local|client_server)_entity_(\w+?)(?:_value)?", table)
    if m and key and re.fullmatch(r"[A-Z0-9_]+_TYPE_[A-Z0-9_]+", key):
        attr_impl[key][m.group(1)].add(etype.get(et, "?"))

ACCESS = re.compile(r"\b(get|set)_(local|client_server)_entity_(\w+?)\s*\(\s*[^,()]+(?:\([^()]*\))?\s*,\s*([A-Z0-9_]+_TYPE_[A-Z0-9_]+)")
used = collections.defaultdict(list)
TOKEN = re.compile(r"[A-Z0-9_]+_TYPE_[A-Z0-9_]+")
ANY = re.compile(r"(?<![A-Za-z0-9_])((?:INT|FLOAT|VEC3D|STRING|PTR|LIST|CHAR|MATRIX3X3)_TYPE_[A-Z0-9_]+)(?![A-Za-z0-9_])")
accessor = set()
contexts = collections.defaultdict(collections.Counter)
for f, t in text.items():
    if cls[f] == "C":
        for m in ACCESS.finditer(t):
            used[m.group(4)].append(f"{f}:{ln(t, m.start())}")
            accessor.add(m.group(4))
        lines = t.split(chr(10))
        for m in ANY.finditer(t):  # every attribute token anywhere in campaign code, not only in matched accessor calls
            i = ln(t, m.start())
            used.setdefault(m.group(1), []).append(f"{f}:{i}")
            line = lines[i - 1]
            ctx = ("case" if re.search(r"\bcase\s+" + m.group(1), line) else
                   "registration" if re.search(r"\bfn_\w+\s*\[", line) else
                   "pack" if re.search(r"\b(un)?pack_\w+\s*\(", line) else
                   "create" if "ENTITY_ATTR_" in line else
                   "access" if re.search(r"\b(get|set)_\w*entity\w*\s*\(|_entity_\w+\s*\(", line) else "other")
            contexts[m.group(1)][ctx] += 1
attr_rows = {}
for a, sites in used.items():
    impl = attr_impl.get(a, {})
    classes = sorted(set().union(*impl.values())) if impl else []
    attr_rows[a] = {"sites": len(sites), "first": sites[0], "implemented_by": classes, "in_accessor_call": a in accessor, "contexts": dict(contexts.get(a, {}))}
bucket = collections.Counter("world-implemented" if "W" in r["implemented_by"] else
                             ("campaign-only" if r["implemented_by"] and set(r["implemented_by"]) <= {"C"} else
                              ("environment/framework" if r["implemented_by"] else "no registration found"))
                             for r in attr_rows.values())

# entity function tables (non-attribute, non-message) that campaign entity types and world types both register
tables = collections.defaultdict(lambda: collections.defaultdict(set))
for table, et, key, h, f, l in regs:
    if table != "message_responses" and not re.fullmatch(r"fn_(get|set)_.*", table):
        tables[table][etype.get(et, "?")].add(et)

summary = {
    "registrations": len(regs),
    "entity_types": dict(collections.Counter(etype.values())),
    "messages_with_handlers": len(handlers),
    "static_message_types_sent": len(static),
    "crossing_static_messages": len(crossing),
    "dynamic_send_sites": len(dynamic),
    "attribute_tokens_in_campaign_accessors": len(attr_rows),
    "attribute_buckets": dict(bucket),
    "function_tables": len(tables),
}
print(json.dumps(summary, indent=1))
if "--detail" in sys.argv:
    Path(sys.argv[sys.argv.index("--detail") + 1]).write_text(json.dumps({
        "etype": etype, "crossing": crossing, "dynamic": dynamic, "attrs": attr_rows,
        "tables": {k: {c: sorted(v) for c, v in d.items()} for k, d in tables.items()},
        "handlers": {k: sorted(v) for k, v in handlers.items()},
    }, indent=1))
```

</details>

<details>
<summary><code>globals.py</code></summary>

```python
"""Shared global data the campaign reads or writes: every `extern` variable
declared in a header, defined (file scope) in a non-campaign compiled file,
and referenced in a campaign function body.

python -I globals.py <repo-root> [--detail out.json]
"""
import collections, json, re, sys
from pathlib import Path

root = Path(sys.argv[1])
src = (Path(__file__).parent / "reach.py").read_text()
ns = {"SYS": "eech-campaign/crates/eech-engine-sys/"}
exec(src[src.index('S = "aphavoc/source/"'):src.index("COMMENT = ")], ns)
klass, SYS = ns["klass"], ns["SYS"]
COMMENT = re.compile(r"/\*.*?\*/|//[^\n]*", re.S)
def clean(t): return COMMENT.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), t)

sources = [l.strip() for l in (root / SYS / "sources.txt").read_text().splitlines() if l.strip()]
units = sources + sorted(p.relative_to(root).as_posix() for p in (root / SYS / "csrc").glob("*.c"))
text = {f: clean((root / f).read_text(encoding="latin-1")) for f in units}

EXTERN = re.compile(r"^\s*extern\s+(?!\"C\")[^;(]*?\b([A-Za-z_]\w*)\s*(?:\[[^\]]*\]\s*)*;", re.M)
externs = set()
for top in ("aphavoc/source", "modules", SYS + "csrc"):
    for p in (root / top).rglob("*.h"):
        externs |= set(EXTERN.findall(clean(p.read_text(encoding="latin-1"))))

# definition site: a file-scope line starting with a type and naming the variable, not extern, not a prototype
defined = collections.defaultdict(set)
for f, t in text.items():
    depth = 0
    for i, line in enumerate(t.split("\n"), 1):
        if depth == 0 and not line.lstrip().startswith(("extern", "#", "typedef")):
            for n in re.findall(r"\b([A-Za-z_]\w*)\s*(?:\[[^\]]*\]\s*)*(?:=|;)", line):
                if n in externs:
                    defined[n].add(f)
        depth += line.count("{") - line.count("}")
        depth = max(depth, 0)

refs = collections.defaultdict(list)
for f, t in text.items():
    if klass(f) != "C":
        continue
    for i, line in enumerate(t.split("\n"), 1):
        for n in set(re.findall(r"\b[A-Za-z_]\w*\b", line)):
            if n in defined and not any(klass(d) == "C" for d in defined[n]):
                refs[n].append(f"{f}:{i}")
rows = {n: {"defined_in": sorted(defined[n]), "classes": sorted({klass(d) for d in defined[n]}), "sites": len(s), "first": s[0]}
        for n, s in sorted(refs.items())}
print(json.dumps({"externs": len(externs), "non_campaign_globals_used_by_campaign": len(rows),
                  "by_class": dict(collections.Counter("/".join(r["classes"]) for r in rows.values()))}, indent=1))
if "--detail" in sys.argv:
    Path(sys.argv[sys.argv.index("--detail") + 1]).write_text(json.dumps(rows, indent=1))
```

</details>

<details>
<summary><code>ledger.py</code></summary>

```python
"""Coverage ledger for the M5 DCS call map: every campaign crossing found by
reach.py, dispatch.py and globals.py gets a category and a disposition
(mapped to a row / excluded with a reason / outstanding).

python -I ledger.py <reach.json> <dispatch.json> <globals.json> [--items out.json]
"""
import collections, json, re, sys

reach, disp, glob = (json.load(open(p)) for p in sys.argv[1:4])
M, X, O = "mapped", "excluded", "outstanding"

# ---- 1. direct calls and references across the campaign boundary --------------------------
# unit: each distinct non-campaign function the campaign calls or references (C->k),
# and each distinct non-campaign function that calls or references the campaign (k->C)
RULES = [  # (regex on "path::function", category, disposition, row or reason)
    # headless host layer
    (r"csrc/eech_engine\.c::", "host", M, "I39, I40 (boot, frame, diagnostics)"),
    (r"csrc/eech_observe\.c::", "host", M, "I4 (host observation surface)"),
    (r"csrc/eech_(headless_\w+|win32)\.c::", "host", X, "platform services (memory, strings, fatal unwinding); no world"),
    # player and multiplayer (before the generic framework and presentation rules)
    (r"misc/msg_in\.c::|events/ev_fns\.c::|ui_menu/.*/(ca_selct|fflight|map)\.c::|common/co_wp\.c::", "player", M, "I44 (player commands and tasking)"),
    (r"misc/msg_out\.c::", "player", M, "I41 (messages to players)"),
    (r"en_comms/en_comms\.c::(?!transmit|enable|disable)", "player", M, "I44 (client requests); replica updates excluded: server-authoritative"),
    (r"en_comms/en_comms\.c::|comms/(comms|comm_man)\.c::", "player", X, "multiplayer replication and client join: headless is COMMS_MODEL_SERVER (eech_engine.c:144) with DirectPlay replaced (build/main.rs:38-39); DCS replicates units"),
    (r"special/pilot/|gunships/dynamics/|common/co_undercarriage\.c::|helicop\.c::(get_local_entity_suitable_for_player|notify_gunship)", "player", M, "I45 (player aircraft, landing, takeoff, results)"),
    (r"gunships/(avionics|cockpits|views)/|common/co_force\.c::", "player", M, "I41 (player displays of campaign state)"),
    # lifecycle, persistence
    (r"aphavoc/source/update\.c::", "framework", M, "I18, I40 (campaign callbacks run by the frame's update list)"),
    (r"gameflow/|aphavoc/source/(flight|init|deinit|game)\.c::|ui_menu/.*/mission\.c::", "framework", M, "I39, I40 (lifecycle)"),
    (r"campaign/campaign\.c::campaign_completed", "presentation", M, "I47 (campaign end)"),
    (r"::(pack|unpack)_|en_sessn\.c::|session/ss_pack\.c::", "persistence", M, "I38 (save, restore)"),
    # framework, not a world interaction
    (r"en_main/en_world\.c::", "framework", M, "I25 (map bounds)"),
    (r"en_anim/en_anim\.c::", "framework", M, "I10 (doors)"),
    (r"en_(funcs|main|types|msgs|misc|attrs)/", "framework", X, "entity store used for campaign records; crossings through it are the dispatch rows below"),
    (r"en_debug/|system/assert\.c::|events/ev_debug\.c::|aphavoc/source/test\.c::|hc_move\.c::debug_visuals|fw_draw\.c::", "framework", X, "debug and test code; no world effect"),
    (r"modules/maths/(?!random)|system/fpu\.c::|misc/miscell\.c::", "framework", X, "pure arithmetic"),
    (r"misc/(file_tag|language)\.c::|system/files\.c::|graphics/psdread\.c::", "framework", X, "campaign data files read natively by the DLL; no DCS API (C stdio)"),
    (r"system/event\.c::", "player", M, "I44 (key-bound campaign triggers)"),
    (r"entity/tacview/|aphavoc/source/sharedmem\.c::", "framework", X, "EECH's own recorder and shared-memory export; observers of the campaign"),
    # presentation files
    (r"graphics/textuser\.c::read_map_info_data", "presentation", M, "I46 (map info, coordinate origin)"),
    (r"graphics/psdwrite\.c::|special/camera/|ui_menu/|gunships/guns_sc\.c::", "presentation", X, "presentation and debug output (screens, camera, influence-map images); the headless frame draws nothing (eech_engine.c:253-286)"),
    # world and environment
    (r"session/ss_updt\.c::", "environment", M, "I27 (weather update looks up sectors)"),
    (r"_msgs\.c::response_to_collision$", "world", M, "I14, I15 (collision damage)"),
    (r"helicop\.c::(helicopter_assume_player_control|helicopter_release_player_control|assign_entity_to_user|set_gunship_entity_to_external_view_entity|transmit_player_recon_data)$", "player", M, "I45 (player aircraft)"),
    (r"mobile\.c::draw_mobile_entity_debug_info$|damage\.c::display_debug_kill_info$", "framework", X, "debug and test code; no world effect"),
    (r"mobile\.c::credit_client_server_mobile_kill$", "world", M, "I14 (kill credit)"),
    (r"mobile\.c::mobile_check_all_members_taken_off$", "world", M, "I13 (takeoff)"),
    (r"fixwing\.c::update_fixed_wing_crate_dropping$", "world", M, "I29 (crate drop)"),
    (r"hc_updt\.c::update_fuel_weight$|wn_misc\.c::get_local_entity_weapon_load_weight$", "world", M, "I7 (fuel and load)"),
    (r"helicop\.c::helicopter_within_keysite_area$", "world", M, "I30, I32 (keysite area)"),
    (r"_creat\.c::create_(client_server|local)_entity_(anti_aircraft|fixed_wing|helicopter|person|routed_vehicle|ship)", "world", M, "I1 (spawn)"),
    (r"special/effect/soundeff/speech\.c::|mobile\.c::play_mobile_under_attack_speech", "world", M, "I41 (speech to players)"),
    (r"mobile/.*_creat\.c::create_local$", "world", M, "I1 (a spawned mobile registers with its force and sector)"),
    (r"(fixed|effect|bridge|cbuild|city|crater|object|pylon|scenic|segment|site|siteupdt)/\w+_creat\.c::|(cr|xp|pa|sl|sp|wn|br)_pack\.c::", "world", M, "I1, I25 (world objects register in sectors and forces)"),
    (r"special/effect/(?!soundeff/speech)|session\.c::attach_session_sound", "world", X, "visual and audio effects; DCS renders its own"),
    (r"(site|siteupdt|cbuild)/|fixed/fixed\.c::", "world", M, "I31 (keysite structures)"),
    (r"bridge/", "world", M, "I24 (bridges break road links)"),
    (r"_dstry\.c::", "world", M, "I14 (losses)"),
    (r"(ac|vh|wn)_tgt\.c::|vh_wpn\.c::|ac_wpn\.c::|weapon/weapon\.c::|wn_tgt\.c::", "world", M, "I8, I36 (attack, detection)"),
    (r"weapon/(wn_\w+|damage)\.c::|aircraft/aircraft\.c::", "world", M, "I11, I15, I16 (damage, fire)"),
    (r"mb_tgt\.c::", "world", M, "I9 (line of sight)"),
    (r"_msgs\.c::overload|mb_funcs\.c::overload", "world", M, "I29, I31 (campaign types reuse world handlers; campaign-only in DCS)"),
    (r"_(move|updt)\.c::|mobile\.c::|fixwing\.c::|routed\.c::|_vec3d\.c::|_int\.c::|helicop\.c::", "world", M, "I6, I12, I30 (movement reads the guide; arrival; takeoff routes)"),
    (r"_msgs\.c::", "world", M, "I12, I13, I30, I32 (arrival tests, typed handlers, landing locks, capture)"),
    (r"modules/3d/terrain/(terrelev|terrdata)\.c::|misc/terr_db\.c::", "environment", M, "I21-I23 (terrain)"),
    (r"modules/3d/terrain/terrobjs\.c::|modules/3d/3d(objdb|subobj|objvis|objid)\.c::", "environment", M, "I2, I26 (3D objects)"),
    (r"misc/tod\.c::", "environment", M, "I19 (time of day)"),
    (r"session/session\.c::(set_session_\w+_weather_mode)|session/ss_updt\.c::", "environment", M, "I27 (weather)"),
    (r"session/session\.c::check_point_inside_population_area", "environment", M, "I25 (map population area)"),
]

def classify(key):
    for rx, cat, disp_, ref in RULES:
        if re.search(rx, key):
            return cat, disp_, ref
    return "unclassified", O, "no rule"

items = []
for direction, edges in reach["cross"].items():
    side = 1 if direction.startswith("C->") else 0
    for key in sorted({e[side] for e in edges}):
        cat, d, ref = classify(key)
        items.append({"ledger": "calls", "direction": direction, "item": key, "category": cat, "disposition": d, "ref": ref})

# ---- 2. message dispatch: static message types that cross the boundary ---------------------
MSG = {
    "CHECK_CAMPAIGN_OBJECTIVES": "I17", "CHECK_MOBILE_REACHED_GUIDE": "I12", "COLLISION": "I15",
    "ENTITY_FIRED_AT": "I16", "ENTITY_TARGETED": "I36", "FORCE_DESTROYED": "I14, I15",
    "GUIDE_COVER_POSITION_REACHED": "I23", "LANDING_SITE_REQUEST": "I30", "LOCK_LANDING_ROUTE": "I30, I38",
    "LOCK_LANDING_SITE": "I30, I38", "LOCK_TAKEOFF_ROUTE": "I30", "MOBILE_KILLED": "I14", "REQUEST_ASSISTANCE": "I44",
    "RESERVE_LANDING_SITE": "I30", "SET_ENTITY_POSITION": "I34", "TASK_ASSIGNED": "I6", "TASK_COMPLETED": "I14, I31",
    "TASK_TERMINATED": "I6, I8, I45", "UNLOCK_LANDING_ROUTE": "I30", "UNLOCK_LANDING_SITE": "I30",
    "UNLOCK_TAKEOFF_ROUTE": "I30", "UNRESERVE_LANDING_SITE": "I30", "WAYPOINT_DROP_OFF_REACHED": "I13, I29",
    "WAYPOINT_PICK_UP_REACHED": "I13, I29", "WAYPOINT_TAKEN_OFF_REACHED": "I13",
}
MSG_X = {"GROUND_FORCE_ADVANCE": "sent only from debug keys (ev_debug.c)",
         "LINK_CHILD": "entity-store list maintenance", "LINK_PARENT": "entity-store list maintenance",
         "UNLINK_CHILD": "entity-store list maintenance", "UNLINK_PARENT": "entity-store list maintenance"}
for m, by in sorted(disp["crossing"].items()):
    name = m[len("ENTITY_MESSAGE_"):]
    if name in MSG:
        d, ref = M, MSG[name]
    elif name in MSG_X:
        d, ref = X, MSG_X[name]
    else:
        d, ref = O, "no rule"
    items.append({"ledger": "messages", "item": name, "category": "dispatch", "disposition": d, "ref": ref,
                  "senders": {k: v["first"] for k, v in by.items()}})

# ---- 3. entity function tables -------------------------------------------------------------
TABLE = {"create": (M, "I1"), "destroy": (M, "I3, I43"), "kill": (M, "I43"), "update": (M, "I40"),
         "pack": (M, "I38"), "unpack": (M, "I38")}
for t, by in sorted(disp["tables"].items()):
    verb = t.split("_")[1] if t.startswith("fn_") else t
    if "ui_object" in t or verb in ("draw", "play", "tool"):
        d, ref = X, "presentation: the headless frame draws nothing and plays no sound (eech_engine.c:253-286)"
    elif verb in TABLE:
        d, ref = TABLE[verb]
    else:
        d, ref = O, "no rule"
    items.append({"ledger": "tables", "item": t, "category": "dispatch", "disposition": d, "ref": ref, "classes": sorted(by)})

# ---- 4. dynamic message ids ------------------------------------------------------------------
for site, c, arg in disp["dynamic"]:
    ref = {"message": "I6: TASK_TERMINATED forwarded to members (gp_msgs.c:1185 handler)",
           "msg": "I50: waypoint_database messages (wp_dbase.c), each resolved below"}[arg]
    items.append({"ledger": "dynamic", "item": site, "category": "dispatch", "disposition": M, "ref": ref})
WP = {  # waypoint_database message -> row
    "START_UP|TAXI|LIFT_OFF|TAKEN_OFF|RAISE_UNDERCARRIAGE|LOWER_UNDERCARRIAGE_ACTION|APPROACH|TOUCH_DOWN|LAND|LANDED": "I13 (G10)",
    "NAVIGATION|SUB_ROUTE_NAVIGATION|CONVOY|REVERSE_CONVOY|LOOP|DEFEND|REPAIR": "I12 (G7)",
    "PICK_UP|PREPARE_FOR_DROP_OFF|DROP_OFF|FINISH_DROP_OFF": "I12, I29 (G7)",
    "TROOP_CAPTURE|TROOP_DEFEND|TROOP_INSERT|TROOP_PICKUP_POINT_END|TROOP_PICKUP_POINT_START|TROOP_PUTDOWN_POINT|PREPARE_FOR_INSERTION": "I49 (G23)",
    "RECON|RECON_ACTION": "I35 (G7)", "ATTACK_ACTION": "I8 (G6)",
}
for m in sorted(k for k in disp["handlers"] if k.startswith("ENTITY_MESSAGE_WAYPOINT_")) + ["ENTITY_MESSAGE_WAYPOINT_END_REACHED", "ENTITY_MESSAGE_WAYPOINT_WAIT_REACHED"]:
    name = m[len("ENTITY_MESSAGE_WAYPOINT_"):]
    if name in ("REACHED", "ACTION"):
        continue  # the guide's generic WAYPOINT_REACHED / WAYPOINT_ACTION: static campaign-to-campaign messages, not waypoint_database ids
    base = re.sub(r"_REACHED$", "", name)
    if not disp["handlers"].get(m):
        d, ref = X, "no handler registered: the message is a no-op"
    else:
        ref = next((r for k, r in WP.items() if re.fullmatch(k, base)), None)
        d = M if ref else O
    items.append({"ledger": "waypoint ids", "item": name, "category": "dispatch", "disposition": d, "ref": ref or "no rule"})
seen = set()
items = [i for i in items if not (i["ledger"] == "waypoint ids" and (i["item"] in seen or seen.add(i["item"])))]

# ---- 5. attribute tokens in campaign accessor calls ----------------------------------------
A = {
    M: {
        "I4, I6 (position, motion)": "VEC3D_TYPE_POSITION VEC3D_TYPE_MOTION_VECTOR FLOAT_TYPE_VELOCITY FLOAT_TYPE_HEADING FLOAT_TYPE_PITCH VEC3D_TYPE_XV VEC3D_TYPE_ZV VEC3D_TYPE_RELATIVE_POSITION",
        "I12 (arrival)": "FLOAT_TYPE_DISTANCE INT_TYPE_LANDED INT_TYPE_AIRBORNE_AIRCRAFT INT_TYPE_OPERATIONAL_STATE INT_TYPE_MEMBERS_PURSUING INT_TYPE_MEMBERS_REACHED INT_TYPE_MEMBERS_WAITING",
        "I14, I15 (alive, damage)": "INT_TYPE_ALIVE INT_TYPE_DAMAGE_LEVEL",
        "I1 (spawn state)": "FLOAT_TYPE_MAIN_ROTOR_RPM INT_TYPE_INSIDE_HANGAR FLOAT_TYPE_CENTRE_OF_GRAVITY_TO_GROUND_DISTANCE INT_TYPE_ALIGN_WITH_TERRAIN FLOAT_TYPE_FUSELAGE_ANGLE",
        "I10 (doors, undercarriage)": "FLOAT_TYPE_LOADING_DOOR_TIMER FLOAT_TYPE_UNDERCARRIAGE_STATE",
        "I8 (attack internals, G6)": "FLOAT_TYPE_WEAPON_BURST_TIMER FLOAT_TYPE_WEAPON_LAUNCH_DELAY FLOAT_TYPE_WEAPON_TO_INTERCEPT_POINT_RANGE VEC3D_TYPE_WEAPON_TO_INTERCEPT_POINT_VECTOR VEC3D_TYPE_WEAPON_VECTOR INT_TYPE_WEAPON_AND_TARGET_VECTORS_VALID INT_TYPE_SELECTED_WEAPON INT_TYPE_WEAPONS_HOLD FLOAT_TYPE_ATTACK_ALTITUDE",
        "I36 (sensing, ROE, G9)": "FLOAT_TYPE_AIR_SCAN_RANGE FLOAT_TYPE_SURFACE_SCAN_RANGE INT_TYPE_ENGAGE_ENEMY INT_TYPE_RADAR_ON LIST_TYPE_TARGET",
        "I48 (type properties, G22)": "FLOAT_TYPE_CRUISE_ALTITUDE FLOAT_TYPE_CRUISE_VELOCITY FLOAT_TYPE_FUEL_DEFAULT_WEIGHT FLOAT_TYPE_FUEL_ECONOMY FLOAT_TYPE_POTENTIAL_SURFACE_TO_AIR_THREAT FLOAT_TYPE_POTENTIAL_SURFACE_TO_SURFACE_THREAT FLOAT_TYPE_TARGET_PRIORITY_AIR_ATTACK FLOAT_TYPE_TARGET_PRIORITY_GROUND_ATTACK INT_TYPE_FORCE_INFO_CATAGORY INT_TYPE_IDENTIFY_AIRCRAFT INT_TYPE_IDENTIFY_FIXED INT_TYPE_IDENTIFY_MOBILE INT_TYPE_IDENTIFY_VEHICLE INT_TYPE_IDENTIFY_WEAPON INT_TYPE_MOVEMENT_TYPE INT_TYPE_POINTS_VALUE INT_TYPE_TARGET_TYPE INT_TYPE_TASK_TARGET_TYPE INT_TYPE_ENTITY_SUB_TYPE STRING_TYPE_FULL_NAME STRING_TYPE_LONG_DISPLAY_NAME STRING_TYPE_SHORT_DISPLAY_NAME",
        "I26 (3D shape, G14)": "INT_TYPE_OBJECT_3D_SHAPE INT_TYPE_APPROXIMATION_OBJECT_3D_SHAPE",
        "I23 (cover, G12)": "VEC3D_TYPE_COVER_POSITION",
        "I31 (structure importance)": "FLOAT_TYPE_FIXED_OBJECT_IMPORTANCE LIST_TYPE_BUILDING_GROUP",
        "I35 (recon)": "FLOAT_TYPE_RECON_RADIUS",
        "I3, I4 (identity: side, group membership; G3)": "INT_TYPE_SIDE INT_TYPE_FORMATION_POSITION INT_TYPE_GROUP_MEMBER_ID INT_TYPE_GROUP_MEMBER_NUMBER LIST_TYPE_MEMBER",
        "I45 (player)": "INT_TYPE_PLAYER LIST_TYPE_AIRCREW LIST_TYPE_PILOT_LOCK",
        "I49 (troops, G23)": "INT_TYPE_TROOPS_ONBOARD",
        "I30 (landing sites, takeoff queue)": "LIST_TYPE_LANDING_SITE LIST_TYPE_TAKEOFF_QUEUE",
        "I29 (cargo records)": "LIST_TYPE_CARGO LIST_TYPE_MOVEMENT_DEPENDENT FLOAT_TYPE_FUEL_SUPPLY_LEVEL",
        "I18 (group sleep timer)": "FLOAT_TYPE_SLEEP",
        "I5 (waypoint altitude at creation; route-point altitude in G4)": "FLOAT_TYPE_ALTITUDE",
    },
    X: {"not an attribute: a type or sub-type constant matched the accessor pattern": "ENTITY_SUB_TYPE_LANDING_FIXED_WING ENTITY_SUB_TYPE_LANDING_FIXED_WING_TRANSPORT ENTITY_SUB_TYPE_LANDING_HELICOPTER ENTITY_TYPE_FORCE"},
}
amap = {}
for d, groups in A.items():
    for ref, names in groups.items():
        for n in names.split():
            amap[n] = (d, ref)
for a, r in sorted(disp["attrs"].items()):
    impl = set(r["implemented_by"])
    ctx = set(r.get("contexts", {}))
    if a in amap:
        d, ref = amap[a]
    elif ctx and ctx <= {"pack"}:
        d, ref = M, "I38: a pack/unpack field tag (save, restore, replication)"
    elif ctx and ctx <= {"case", "registration", "pack"}:
        d, ref = X, "the campaign entity's own implementation of the attribute (case label, table registration, pack tag); reads no world value"
    elif ctx == {"other"} and a.endswith("_MAX_LENGTH"):
        d, ref = X, "an array-size constant, not an attribute"
    elif a.startswith("LIST_TYPE_"):
        d, ref = M, "I5 and G3: campaign link list (generic en_list.c storage); links to world members go through G3"
    elif impl and impl <= {"C"}:
        d, ref = X, "campaign-only: implemented only by campaign entity types"
    elif impl and "W" not in impl:
        d, ref = M, "I18, I19, I27 (session: time, weather, flags) or framework default"
    else:
        d, ref = O, "no rule"
    items.append({"ledger": "attributes", "item": a, "category": "attributes", "disposition": d, "ref": ref,
                  "implemented_by": sorted(impl), "first": r["first"]})

# ---- 6. shared global data -----------------------------------------------------------------
G = [
    (r"_names$|^application_tag_strings$|^dik_names$", X, "string tables for parsing and logging"),
    (r"^(aircraft|vehicle|weapon|effect)_database$|^fixed_3d_object_database$|^target_point_offsets$", M, "I48 (type database, G22)"),
    (r"^speech_sector_coordinates$", M, "I41 (speech)"),
    (r"^objects_3d_\w+$|^object_3d_enumeration_names$", M, "I2, I26 (3D object data, G14)"),
    (r"^terrain_3d_max_map_z$", M, "I25 (map extents)"),
    (r"^(number_of_)?psd_layers$", X, "campaign data image read natively (popread.c:362)"),
    (r"^active_2d_environment$|^sys_colour_table$", X, "debug drawing (wp_draw.c)"),
    (r"^force_local_entity_create_stack_attributes$", X, "entity-store flag"),
]
for n, r in glob.items():
    d, ref = next(((d, ref) for rx, d, ref in G if re.search(rx, n)), (O, "no rule"))
    items.append({"ledger": "globals", "item": n, "category": "globals", "disposition": d, "ref": ref, "first": r["first"]})

# ---- 7. call-shaped names in campaign code that resolve to no definition -------------------
U = {"endif": "not a call: a #endif line followed by a parenthesised expression (ld_msgs.c:512)",
     "function": "a call through a function-pointer field (briefing.c:880); its targets are address-taken functions, walked as references",
     "ogre_scene_get_object": "in the #else arm of #ifndef OGRE_EE (popread.c:2795); the headless build does not define OGRE_EE"}
for n in reach.get("unresolved_names", []):
    items.append({"ledger": "unresolved names", "item": n, "category": "index", "disposition": X if n in U else O, "ref": U.get(n, "no rule")})

# ---- totals -------------------------------------------------------------------------------
tot = collections.defaultdict(collections.Counter)
for i in items:
    key = i["ledger"] if i["ledger"] != "calls" else f"calls {i['direction']} [{i['category']}]"
    tot[key][i["disposition"]] += 1
out = {k: dict(v) for k, v in sorted(tot.items())}
out["ALL"] = dict(collections.Counter(i["disposition"] for i in items))
out["ambiguous names left"] = len(reach["ambiguous"])
print(json.dumps(out, indent=1))
for i in items:
    if i["disposition"] == O:
        print("OUTSTANDING", i["ledger"], i["item"])
if "--items" in sys.argv:
    json.dump(items, open(sys.argv[sys.argv.index("--items") + 1], "w"), indent=1)
```

</details>
