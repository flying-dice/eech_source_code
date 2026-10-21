# M5: the DCS call map

This record answers M5 done-when criterion 1 (Confluence page 11534337, "EECH M5 Goal: EECH_DC into DCS World"):

> **DCS call map.** Every world interaction the campaign needs (creating and routing forces, tasking, events, time, terrain) is mapped to how DCS World's mission scripting environment provides it. Each gap has a named workaround.

It is a source-grounded proposal for review, not an acceptance. It changes no code, build, baseline or DCS configuration, and no DCS or EECH runtime was launched for it. DCS-only runtime risks (native module load, frame budget and threading, save and restore, multiplayer) are criterion 5, owned by `docs/m5-dcs-risks.md` (`eech-campaign/docs/`, in another session's worktree and not yet merged); this record cross-references them and does not probe them.

**In brief.**
- **Two passes over the 1,217 compiled C files plus the 11 headless host files, at `65892de1`.**
  - The first pass (`inventory.py`) measured direct calls between path classes and named world-service tokens.
  - The second pass closes the first pass's coverage holes:
    - **a reachability walk** from the seven public engine entry points, with translation-unit-local name resolution, macro expansion, and address-taken functions and file-scope data counted as references (`reach.py`);
    - **the dispatch tables:** message handlers, entity function tables, other function-pointer table forms, attribute accessors, and every campaign write to an attribute that a world entity type implements (`dispatch.py`);
    - **the shared global data** the campaign touches, directly or through a macro (`globals.py`).
  - The four scripts share one source model, `eechmap.py`.
- **The coverage ledger** (`ledger.py`, section 1) accounts for 1,126 items: 671 are mapped to an interaction row, 455 are excluded with a reason grounded in the headless product path, and **0 are outstanding**.
  - The items are:
    - campaign crossings to or from host, framework, presentation, world and environment code;
    - message types and waypoint message ids;
    - function tables;
    - attribute tokens;
    - campaign writes to world-implemented attributes;
    - global data;
    - names that resolved to no definition.
  - **Rules:** a rule's kind is derived from its regex, not set by hand. How many items each kind decides, and every pattern and fallback rule, are stated once, in section 10 ("Pattern and fallback rules").
  - No needed interaction found by these methods lacks a DCS mechanism or a named workaround.
  - The method's residual limits are in section 10. Accepting criterion 1 remains the technical lead's review decision.
- **The independent review of `a4cd7eea`** raised six findings, F1–F6, all resolved here:
  - F1: the side transfer on capture is now row I51, with workaround G28 and five named DCS probes.
  - F2: the globals pass grew from 31 to 95 items, and the season is now row I52.
  - F3: the catch-all rules were replaced, and the remaining pattern rules are counted.
  - F4: the UI table form is documented.
  - F5: the UI-caller scope is explained.
  - F6: the shared module replaces the `exec`.
- **The re-review of `ab830a04`** raised N1 and C1–C6, all fixed here:
  - N1: G28 now separates the conversion, which kills nothing, from the kills `destroy_keysite` makes in the same capture.
  - C1: the class X label now says disposition per item.
  - C2: rule kinds are derived from the regexes, and the counts are restated.
  - C3: G22 keeps the balance properties in EECH_DC.
  - C4/C5: the shadowed and unreachable alternatives were removed.
  - C6: dynamic sends are allowed by site.
- **53 interactions in 13 categories** are mapped to DCS: 8 fit directly (campaign-only work that needs no DCS call), 27 are adapted and 18 are gaps. Every gap, and every adapted row that needs one, has a named workaround (G1–G30).
- **The largest gaps:**
  - there is **no verified "waypoint reached" signal** in DCS (G7);
  - **movement is pulled from the campaign's guide every frame** in EECH, but DCS AI flies a route it is given (G4);
  - EECH's **loss, damage, target-detection and capture bookkeeping runs inside world code** (G8, G9);
  - **capture moves landed aircraft and keysite structures to the captor's side**, and no coalition setter for units, groups or statics was found (G28);
  - **player commands and player aircraft** reach the campaign through EECH's own UI, cockpit and radio code (G24, G27);
  - **no save of physical state** in the mission environment (G19);
  - **one engine per process** against DCS mission restarts (G20);
  - **the retail maps are not metric, and this PC has no Syria terrain** (G21).
- **DCS evidence:** of the 30 DCS evidence items (D1–D30), 14 are confirmed by this install's shipped files ([R-dcs]), 14 rest on tslua-dcs only ([R-ref]) and 2 are unknown ([U]); five items carry a named partial unknown. No shipped file gives a mission-API signature; every signature below is tslua-dcs's.

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
| **X** presentation files (disposition per item) | `ui_menu/`, `appsound/`, `app3d/`, `gunships/`, `entity/special/camera/`, `cdmusic.c`, `modules/{userint2,graphics,2d,sound}/` | 517 |
| **F** framework | everything else: the entity system, comms, events, maths, file and memory | 136 |

Choices a reviewer may contest: `guide/` is campaign, although it also steers the flight model (#74, "The guide straddles the line"). `landing/` (landing sites, takeoff routes) is campaign, although #74 leaves its ownership open. `cargo/` is campaign (crates are a derived campaign record, #74 stage 1). `gunships/` (the player's cockpit, avionics and flight model) is class X. The first pass left class X out of its edges. In the second pass every X item has its own disposition: of the 118 functions under X that reach the campaign, 97 are mapped (cockpit displays to I41, player commands to I44, the player aircraft to I45) and 21 are excluded (campaign screens).

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

Reproduce (with `eechmap.py` beside `inventory.py`):

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

### Second pass: reachability, dispatch, writes and globals

The scripts are in full at the end of this record. Save them side by side in one directory: `eechmap.py` is the shared source model, and the other four import it. `reach.py`, `dispatch.py` and `globals.py` take the repository root; `ledger.py` takes their JSON output.

**`reach.py`** [R-src]
- **Indexes** every function defined in the 1,217 `sources.txt` files and the 11 `csrc/*.c` files (class H, the headless host layer). This includes bodies that follow `#if` arms, and nested function-pointer parameters.
- **Resolves** each identifier in a body to the definition in the same file, else to the unique non-static global. Same-file `#ifdef` twins count as one definition.
- **Expands** macros from every header and C file under `aphavoc/source`, `modules`, `csrc` and `compat`, recursively.
- **Walks** from the seven entry points in `eech-campaign/crates/eech-engine-sys/csrc/eech_engine.h:36-102`: boot, frame, objects, clock, prepare_installation, diagnostics and fatal_message. It counts every reference, whether a call or an address taken, and the file-scope data of any file whose globals are referenced. This is an over-approximation by design: it reaches registered handlers, callbacks and data-table initialisers without modelling them.
- **Results at `65892de1`:**
  - Reachable/defined functions: C 997/1,139, W 1,603/1,813, E 799/889, F 914/1,086, H 266/294, X 5,677/6,114.
  - Ambiguous names left: 0. The first pass's 76 (for example `kill_local`, defined in many entity files) all resolve once a reference is resolved within its own file first.
  - Campaign call sites examined: 14,599. Call-shaped names with no definition: 3, all accounted for in the ledger.
  - Distinct non-campaign functions the campaign calls or references: C→F 122, C→W 38, C→E 25, C→X 10, C→H 6. Distinct non-campaign functions that call or reference the campaign: W 121, X 118, F 42, E 3, H 3.

**`dispatch.py`** [R-src]
- **Reads** every table registration `fn_*[TYPE]… = f` and `message_responses[TYPE][MSG] = f`. This includes the per-type `[type]` registrations of the shared mobile files.
- **Scans** every other array assignment whose right-hand side names a defined function, for table forms the registration pattern does not match (F4).
- **Classifies** each entity type by the files that register it.
- **Finds** every static message sent per class, with its handler classes, and every non-literal `notify_*_entity` site.
- For every attribute token in campaign files, it records which classes implement the token and in which context the campaign uses it: accessor, creation argument, pack tag, `case` label or registration.
- It lists every campaign `set_*` call on a token that a world entity type implements, with the receiver expression.
- **Results:** 2,879 registrations. 40 entity types: W 23, C 11, F 3, X 2, E 1. 30 static message types cross the boundary, and there are 3 dynamic sends. 22 entity function tables and 6 other table forms. 330 attribute tokens in campaign code. 116 campaign writes to world-implemented tokens.

**`globals.py`** [R-src]
- Finds every name declared `extern` in a header under the include trees, counting every declarator of a comma list. Prototypes and function-pointer declarations are left out; the latter are the dispatch tables.
- Keeps the names defined at file scope in a compiled non-campaign file and referenced in a campaign file, either by name or through the recursive expansion of a macro used there (F2).
- **Results:** 1,659 externs. 95 non-campaign globals used by the campaign: F 61, X 18, W 8, E 8. 36 are reached only through a macro: for example `system_delta_time` through `get_delta_time`, `random_number_seed` through `frand1`, and `current_map_info` through `set_global_season`.

**Cross-checks with the independent reviewer's scripts** (`g2.py`, `writes.py`, run on the same worktree):
- `g2.py` lists 64 globals beyond the first revision's 31. `globals.py` finds the same 64 names, with no difference either way; 31 + 64 = 95. The review text says 65, but its own script prints 64 names.
- `writes.py` lists 116 campaign writes to world-implemented tokens, in 24 tokens. `dispatch.py` finds the same 116.

### The coverage ledger

`ledger.py` gives every item from the three scripts one disposition: **mapped** to an interaction row, **excluded** with a reason grounded in the headless product path, or **outstanding**.
- Rules are ordered regular expressions over `path::function`, token names, global names or `file:line|token|receiver`.
- **A rule's kind is derived from its regex** by `rule_kind` in `ledger.py`, never set by hand.
  - A rule is **exact** when every top-level alternative names its keys literally, without wildcards:
    - a call key ending `::name$` (the path part may be a file-name suffix, so one exact name can match the same function in several files);
    - a global `^name$`;
    - a write site `file:line|TOKEN|receiver$`.
  - Any other rule is a **pattern**. A **fallback** decides attribute tokens by their usage context.
  - The fixed tables (messages, waypoint ids, named attributes, dynamic sends, unresolved names) are exact by construction: they are dictionaries of literal keys.
  - An item that no rule or entry matches is outstanding.
  - The dynamic-send entries are keyed on the reviewed site and argument (`gp_msgs.c:1244`, `wp_msgs.c:271`, `wp_msgs.c:440`). A new or moved non-literal send is therefore outstanding until reviewed (C6).

Result at `65892de1` [R-src]:

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
| Other function-pointer table forms | 6 | 0 | 6 | 0 |
| Dynamic message sends | 3 | 3 | 0 | 0 |
| `waypoint_database` message ids | 33 | 31 | 2 | 0 |
| Attribute tokens in campaign code | 330 | 188 | 142 | 0 |
| Campaign writes to world-implemented attributes | 116 | 52 | 64 | 0 |
| Non-campaign global data | 95 | 28 | 67 | 0 |
| Call-shaped names with no definition | 3 | 0 | 3 | 0 |
| **Total** | **1,126** | **671** | **455** | **0** |

**Exclusion reasons, with counts:**
- **Attributes:**
  - 86 are implemented only by campaign entity types.
  - 44 are the campaign entity's own implementation of an attribute (a `case` label, table registration or pack tag) and read no world value.
  - 4 are type constants that matched the accessor pattern.
  - 3 are list-table constants.
  - 2 are array-size constants.
  - 2 are visual-effect lists unlinked on destroy.
  - 1 is the player's view padlock list.
- **Calls:**
  - 31 entity-store functions.
  - 27 debug or test functions.
  - 27 campaign-screen, camera and influence-map image functions. The headless frame draws nothing (`eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c:253-286`), and the host layer has no display or DirectPlay connection (`eech-campaign/crates/eech-engine-sys/csrc/eech_headless_devices.c:5-10`).
  - 25 arithmetic functions.
  - 23 readers of campaign data files. They are read natively with C stdio, so no DCS API is involved; the files ship with the module (I39).
  - 8 visual and audio effects.
  - 6 multiplayer replication and client-join functions. The headless engine is `COMMS_MODEL_SERVER` (`eech_engine.c:144`) with DirectPlay replaced (`eech-campaign/crates/eech-engine-sys/build/main.rs:38-39`), and DCS replicates units itself.
  - 6 platform services.
  - 2 observers of the campaign: EECH's own Tacview writer and its shared-memory export.
- **Writes:**
  - 56 have a campaign entity as the receiver: a sector, waypoint, guide, keysite, group, cargo or landing entity. The token is also world-implemented, but these writes are campaign state. Examples: the attack guide's own sub-type and velocity (`aa_attck.c:347`), sector sides (`popread.c:446`).
  - 7 propagate `SET_ENTITY_POSITION` to campaign entities.
  - 1 sets a keysite's supply on drop-off.
- **Globals:**
  - 31 string tables.
  - 21 entity-store attribute type tables and their diagnostics, reached through the accessor macros.
  - 10 debug-drawing colours and 2D state.
  - 3 entity-store globals.
  - 2 PSD buffers for campaign data.
- **Messages:** 4 entity-store list messages, and 1 message sent only from debug keys.
- **Tables:**
  - 11 draw, sound and UI tables.
  - `campaign_screen_message_responses` (`ca_group.c:1584-1588`, 37 registrations, F4). It is the campaign-screen UI's own handler table, fed only through `notify_campaign_screen`, which is itself excluded as presentation.
  - 5 data arrays that the scan flagged only because a variable shares a function's name, for example `polygon_3d_vertices[0].v = v;` (`3dscon.c:993`).
- **Waypoint ids:** 2 with no registered handler, so no-ops (`END`, `WAIT`).
- **Names with no definition:**
  - `#endif` followed by a parenthesised expression (`ld_msgs.c:512`);
  - a call through a function-pointer field (`briefing.c:880`), whose targets are walked as address-taken references;
  - `ogre_scene_get_object`, which is in the `#else` arm of `#ifndef OGRE_EE` (`popread.c:2795`). The headless build does not define `OGRE_EE`.

**The 21 excluded presentation callers and the display scope (F5).**
- All 21 are campaign-screen functions: briefing, debriefing, group and task lists, statistics and the info bar. Their exclusion rests on the headless product path: the frame draws nothing, and there is no display.
- Two of them touch campaign state:
  - `notify_base_destroy_button` calls `capture_keysite` (`ca_base.c:521`), but it is registered only in a debug build (`ca_base.c:990-1008`).
  - `notify_member_accept_button` (`ca_group.c:594`) is the player picking a group member. That choice reaches the campaign through `ca_selct.c`, which is mapped (I44).
- Cockpit displays of campaign state are mapped instead (I41, 65 functions under `aphavoc/source/gunships/`). In DCS a flying player has no EECH campaign screen, so the campaign must push what the cockpit showed (waypoints, task text) through messages and markers. The campaign screens are EECH's planning UI, outside the DCS host; their player decisions reach the campaign through the mapped I44 paths.

`python -I ledger.py reach.json dispatch.json globals.json --items items.json` writes every item with its disposition, row and deciding rule.

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

| D30 | Change the coalition or country of an existing unit, group or static object | Not found. tslua-dcs declares only getters (`MT/Unit.ts:362`, `:425`; `MT/Group.ts:98`; `MT/StaticObject.ts:240`, `:286`). Its runtime-reflected method lists name `setCoalition` only for `Airbase` (`MTX/Airbase.export.ts:116`), not for Unit, Group or StaticObject (`MTX/Unit.export.ts:46`, `MTX/Group.export.ts:20`, `MTX/StaticObject.export.ts:10`). Probe P-CAP-1 settles it | [U] |

Totals: [R-dcs] 14 (D1–D5, D7–D14, D29; D11 and D14 by their verified part); [R-ref] 14 (D6, D15–D26, D28); [U] 2 (D27, D30). Partial unknowns inside rows with another label: D6's numeric ids, D11's trigger, D14's behaviour, D15's `groupData` schema and D18's route-task table.

## 3. Interactions by category

Columns: **Dir** is C→W (the campaign asks the world) or W→C (the world reports to, or reads from, the campaign). **Fit** is direct, adapted or gap. All EECH sites are [R-src].

### Forces and spawn

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I1 | Create a mobile member in a group | `create_mobile_member`, `faction.c:1678` (the one switch; creators at `faction.c:1757-1803`). Callers: campaign load `parsgen.c:1854`, regen `rg_updt.c:399`, frontline `faction.c:1550` | C→W | entity index, member sub-type, group entity, `vec3d` position; landed members get a landing site | `coalition.addGroup` D15, per group, not per member; parking start D18 | adapted (G1) |
| I2 | Population: cities, sites, scenery as entities with 3D objects | `popread.c:1575` `construct_3d_object`, `:1588` `insert_3d_object_into_terrain` | C→E | 3D object index, position | Map scenery is fixed; `addStaticObject` D15 | gap (G2) |
| I3 | Remove an entity | `destroy_client_server_entity` (`aphavoc/source/entity/system/en_funcs/en_dstry.h:75`) | C→W | entity | `Group:destroy` D8, `Object:destroy` D25 | direct |
| I43 | Campaign kills world units when a keysite is destroyed or captured | `destroy_keysite` (`keysite.c:945`). With the capture-aircraft setting on (the default, `keysite.c:972-1102`), it kills the losing side's untasked, landed, non-airborne aircraft (`keysite.c:1003`), and kills, through `group_kill_all_members` (`group.c:1021`), every non-aircraft group still linked to the keysite (`keysite.c:1097`; the `else` at `:1091` pairs with `if` aircraft group at `:1025`). With the setting off (`keysite.c:1103-1227`), it kills landed aircraft (`:1132`) and linked non-aircraft groups (`:1222`). It is reached when a keysite falls below minimum strength (`keysite.c:822` → `ks_dstry.c:346`) and on capture (`keysite.c:1423`). On capture, the ground groups listed as keysite groups are first moved to the losing force (`keysite.c:1397-1409`), so `:1097` then reaches only a non-aircraft group linked under another list type (the case logged at `:1411-1414`), and converted aircraft groups (I51) already carry the captor's side, so `:1003` spares them. Kill and destroy tables are in the ledger | C→W | entity | `Group:destroy` D8 (removal); `trigger.action.explosion` D28 (a physical kill). Whether either raises DEAD is [U] | adapted (G26) |
| I48 | Type properties read from world type databases | `aircraft_database` (`faction.c:999`), `vehicle_database`, `weapon_database`, `fixed_3d_object_database` (globals ledger); 28 type-property attribute tokens (cruise speed and altitude, fuel economy, threat values, identify flags, display names) | C reads W | per-sub-type records | Balance properties stay in EECH_DC. The host maps the sub-type to a DCS type; `Unit:getDesc` exists (D9), but its fields are not verified [U] | adapted (G22) |

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
| I52 | Season: the campaign file sets it (`parsgen.c:2196`, `set_global_season`, a macro writing `current_map_info.season`, `aphavoc/source/global.h:985`); the session's weather model reads it to choose snow or rain (`session.c:404`), and terrain and 3D object textures read it (`terrdata.c:219`, `3dobjid.c:930`) | as cited | C→E | season number | DCS season follows the mission date and terrain: `env.mission.date` D19; no runtime season setter verified | adapted (G29) |

### Airbases, keysites and supply

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I29 | Keysite supply levels, crates and requests | `ks_updt.c:126`; `update_keysite_cargo` `keysite.c:337` (#74 stage 1) | campaign-only | ammo and fuel percentages | Optional mirror to `Warehouse` D24 | direct |
| I30 | Landing sites, takeoff routes and locks | `response_to_lock_landing_site` `ld_msgs.c:599`; `insert_mobile_into_takeoff_route` `mobile.c:573`; reserve `mb_msgs.c:350` | both | site counts and locks | `Airbase:getParking (true)` D24; DCS ATC sequences | gap (G16) |
| I31 | Keysite structures destroyed or repaired | `st_dstry.c:293` → `keysite.c:739`; repair `keysite.c:1618` → `fixed.c:147` | W→C, C→W | importance and efficiency | DEAD event for a static or scenery initiator D4, D6 | gap (G17) |
| I32 | Capture of the keysite itself | `mb_msgs.c:2307` → `capture_keysite` `keysite.c:1289`; the keysite's own side `keysite.c:1340` | W→C | keysite, side | EECH's rule decides capture (`mb_msgs.c:2307`). The adapter then applies it with `Airbase:setCoalition`, with `autoCapture` off (D24). BASE_CAPTURED (D6) is only reconciled against EECH's decision, never used as the trigger | adapted |
| I51 | Capture converts forces: idle landed aircraft groups at the keysite change to the captor's side, with new callsigns (`keysite.c:1375-1390`, under `command_line_capture_aircraft`, on by default, `cmndline.c:151`), and the keysite's building and regen structures change side (`change_local_keysite_building_sides`, `keysite.c:1429` → `:1524-1568`; replayed on restore, `ks_pack.c:359`) | as cited; 6 side writes in the write ledger | C→W | entity side, group callsign | No coalition setter for units, groups or statics (D30 [U]); destroy D8 and respawn D15 under the captor's country | gap (G28) |
| I33 | Keysite identity | keysite name attribute (89 campaign sites) | both | name string | `Airbase.getByName` D24 | adapted (G3) |
| I34 | Moving keysites (ships) carry their landing sites | `ks_msgs.c:337` | W→C | position, heading, velocity | ship `Unit:getPoint` D16 | adapted |

### Sensing and visibility

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I35 | Fog of war per sector | `sector.c:510`, `sector.c:533`; recon arrival `mb_msgs.c:1951` | campaign-only, fed by arrivals | sector value | depends on I12 | direct |
| I36 | Target detection triggers a campaign engagement; the session's suppress-AI-fire setting stops campaign engagements (`engage.c:129`) | `ac_tgt.c:275` → `engage_targets_in_group` `engage.c:515` | W→C | group, target group; session flag | `Controller:getDetectedTargets` D17; ROE options through `setOption` D17, so that DCS AI does not choose targets itself, with weapon hold for suppress-AI-fire | gap (G9) |

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
| I53 | Campaign settings that steer decisions: the player pilot's difficulty (`engage.c:991`), campaign-commander mode (`session_camcom`, `reaction.c:859`), run mode (`faction.c:1696`), language and gunship side (`global_options`), and the campaign's aircraft requirement (`parsgen.c:970`) | as cited | host→C | integers from the command line and the campaign file | none: boot configuration (`eech-campaign/crates/eech-dc/src/module.rs:43-54`) | adapted (G30) |
| I47 | Campaign end: winner, session complete, player log | `campaign_completed`, `campaign.c:1096`, called from `fc_msgs.c:307` | C→X | side, completion type | `outText` D26; no mission-end call verified [U] | gap (G25) |

### Players and multiplayer

The headless product boots as a server, with no DirectPlay connection and no joystick or display devices (`eech_engine.c:144`; `eech-campaign/crates/eech-engine-sys/csrc/eech_headless_devices.c:5-10`). These paths are therefore not exercised there. They are still interactions that the campaign consumes, and the supported DCS destination is server-authoritative with human players.

| # | Interaction | EECH site(s) | Dir | EECH shape | DCS | Fit |
|---|---|---|---|---|---|---|
| I44 | Player commands and tasking: radio commands to wingmen and support (attack my target, hold, rejoin, return to base, airstrike, artillery, assistance); picking a task in the campaign screen or free flight; skipping a waypoint; key-bound campaign triggers; a client's task request | `process_radio_message` `msg_in.c:965`; `ca_selct.c:1083`; `fflight.c:504`; `co_wp.c:105`; `ev_fns.c:203`; `en_comms.c:5352` (43 ledger items) | player→C | message type and value; task; group | `missionCommands.addCommand` with a callback D29 | adapted (G24) |
| I45 | The player's aircraft as a campaign member: pilot entity, player control of a group member, its landing and takeoff against landing locks, mission results | `create_new_pilot_entity` `pilot.c:271`; `helicopter_assume_player_control` `helicop.c:1972`; `dynamics.c:2017-2021`; `gp_msgs.c:1034` (18 ledger items) | both | player flag, aircrew lists, landing entity | PLAYER_ENTER_UNIT and PLAYER_LEAVE_UNIT D6; `getPlayerName` D16; LAND and TAKEOFF D5 | adapted (G27) |
| — | Multiplayer replication of campaign state to clients, and client join | `transmit_entity_comms_message` (82 campaign functions, 93 raw `grep` call sites); `comm_man.c` client join (ledger) | C→clients | packed entity updates | none needed: DCS replicates units; campaign state stays on the server | excluded: server-authoritative destination; the headless engine has no clients (`eech-campaign/crates/eech-engine-sys/build/main.rs:38-39`) |
| — | Player displays of campaign state: HUD and MFD waypoints, task objective text, briefings | 64 cockpit, avionics and view functions under `aphavoc/source/gunships/` (ledger, I41) | C→player | strings, waypoint positions | `outText`, `markToAll` D26 | adapted (with I41) |

Totals: 53 rows. Direct 8 (I3, I5, I10, I17, I25, I29, I35, I37). Adapted 27. Gap 18 (I2, I6, I8, I12, I20, I23, I24, I26, I27, I30, I31, I36, I38, I39, I42, I47, I49, I51). The gap table (section 8) names a workaround for each gap and for the adapted rows that need one. The two un-numbered player rows record ledger groups and are not counted.

## 4. Identity and lifecycle

- **EECH:** an entity is a pointer into one heap. Its identity outside the C is its index (`en_heap.h:115`), and that index is **reused after destruction** (`eech-campaign/crates/eech-engine/src/lib.rs:106`). The SUPPLY map notes that guides and tasks can be destroyed mid-notification (#74 stage 4). [R-src]
- **DCS:** groups and units have names (`Group.getByName` D7, `Unit.getByName` D16) and ids (`Unit:getID (): number | string` D16). Events carry object references (`event.initiator`, `event.target`, D4). Whether names are unique per mission, and whether ids are reused, is [U]: a spawn/destroy/respawn probe would settle it.
- **The removal rule (R).** When the campaign removes DCS units without a combat outcome, it first marks their G3 names *removed*, then calls `Group:destroy` or `Object:destroy`. This covers troop pick-up (G23), units whose kill the campaign has already recorded (G26) and capture conversion (G28). G8 drops any DEAD, CRASH, UNIT_LOST or KILL event for a removed name before the loss accounting, so a removal never reaches it twice or as a new loss. Which events DCS raises on removal is probe P-CAP-2.
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
  - No campaign file reads the origin: `globals.py` finds the campaign's only use of `current_map_info` to be the season write (`parsgen.c:2196`, I52), so no campaign decision depends on the origin. [R-src]
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
| G9 Detection | I36 | **Detection observations, campaign targeting:** poll `getDetectedTargets` per campaign group into the campaign's engage decision. Target choice and priority stay in EECH_DC (as in G22). ROE is set so that DCS AI does not pick targets itself, and weapon hold covers suppress-AI-fire. DCS AI performs only the physical weapon release, after the campaign has assigned the target (I8, G6). SHOT, HIT and DEAD come back as observations (section 5) | D17, D6 [R-ref] |
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
| G22 Type database | I48 | **Campaign-side type properties, host-side type mapping.** The properties the campaign reads stay in EECH_DC, keyed by EECH sub-type, as EECH's own databases hold them today (`aircraft_database`, `vehicle_database`, `weapon_database`): cruise speed and altitude, fuel economy, threat values, target priorities and points. Threat, priority and points are campaign balance and policy, and are never given to the DCS adapter. The adapter translates only physical type: EECH sub-type → DCS type name. It reports capability facts from `Unit:getDesc` (whether the type exists, its category and its weapons) where those fields are verified. EECH_DC decides task eligibility from those facts. DCS flies its own type's performance; EECH keeps its planning figures (cruise speed and altitude, fuel economy) campaign-side, so a plan's ETA or fuel can differ from the DCS flight. That difference is classified as a world variation, not a regression | D9 [R-dcs]; fields [U] |
| G23 Troop transport | I49 | **Abstract troops:** troops stay a campaign count on the carrier. Insertion spawns an infantry group at the drop point (`coalition.addGroup`), and pick-up removes one (`Group:destroy`) under the removal rule R (section 4), so a pick-up is never a loss. No DCS embark is relied on | D15 [R-ref], D8 [R-dcs] |
| G24 Player commands | I44 | **F10 command menu:** each EECH radio command, task request and waypoint skip becomes a `missionCommands` entry. Its callback sends a player-command message to the campaign through the module's public interface: the player's group (resolved through G3), the command and its value. The campaign then routes the message to its own handling (today `process_radio_message`, `msg_in.c:965`); the adapter never calls an internal EECH function | D29 [R-dcs] |
| G25 Campaign end | I47 | **Host-owned end:** the campaign reports the winner through `outText` and an observable state. The host ends or keeps the mission; no mission-end API is relied on | D26 [R-ref] |
| G26 Kill or remove | I43 | **Campaign-recorded kills:** when the campaign itself kills a group, it records the losses directly, as EECH's own consequence, and removes the units with `Group:destroy`. It does not wait for a DEAD event that may not come. The units are removed under the removal rule R (section 4), so any DEAD, CRASH, UNIT_LOST or KILL that does arrive for those names is dropped at G8 and the kill is not counted twice | D8 [R-dcs]; DEAD on destroy [U], probe P-CAP-2 |
| G28 Respawn on capture | I51 | **Respawn under the captor, keeping EECH's capture policy.** The capture rules stay the campaign's: idle landed aircraft groups and the keysite's structures change side, under the capture-aircraft setting, which stays on by default. The workaround only carries out the result in DCS. On each conversion: **(1) Rebind identity:** in the G3 table, mark the old DCS names of the converted group or static as *converted at generation g*. Allocate generation g+1 names for the same EECH entity index, whose side and callsign EECH has already changed in place. **(2) Remove without loss:** remove the old objects with `Group:destroy` or `Object:destroy` under the removal rule R (section 4). The old names are marked *converted*, which counts as *removed*, so their DEAD, CRASH, UNIT_LOST or KILL events are dropped at G8 and a conversion is never reported as a combat loss. The conversion itself calls no kill and runs no loss bookkeeping (`keysite.c:1375-1390`, `:1524-1568`). The same capture can still kill: `capture_keysite` then calls `destroy_keysite` (`keysite.c:1423`). That kills the losing side's untasked, landed, non-airborne aircraft that were not converted (`keysite.c:1003`); converted groups already carry the captor's side and are skipped. `:1097` kills only a non-aircraft group still linked under a list type other than keysite group (the case logged at `keysite.c:1411-1414`). Standard ground groups were moved to the losing force first (`keysite.c:1397-1409`) and survive. The adapter executes only those kills, as I43 losses under G26, recorded by the campaign and never marked *converted*. **(3) Respawn:** with `coalition.addGroup` (cold, at the same parking spots, `Airbase:getParking`) or `addStaticObject`, under a country of the captor's coalition, with the generation g+1 names. **(4) Order:** set the airbase's coalition (`Airbase:setCoalition`, with `autoCapture` off) before respawning aircraft there. **Probes,** finite and named, settle the unknowns: **P-CAP-1** does any coalition or country setter exist for units, groups or statics (D30)? If one does, it replaces steps 2–3. **P-CAP-2** which events, if any, do `Group:destroy` and `Object:destroy` raise? **P-CAP-3** does a destroy followed by `addGroup` in the same scheduled call succeed on the same parking spots, and do the new units get fresh ids while `Group.getByName (old)` returns nil? **P-CAP-4** can a group of the captor's country spawn at an airbase just switched with `setCoalition`? **P-CAP-5** does a static respawned at the same position under the new country become a valid DEAD initiator for G17? | D8 [R-dcs], D15, D24 [R-ref]; D30 [U] |
| G29 Season as a host input | I52 | **Read, do not set:** the host's mission date and terrain give the season. The campaign file's season is checked against it at boot, and a difference is classified as an expected input or world variation, not a regression. The weather model's snow-or-rain choice uses the host's season | D19 [R-ref] |
| G30 Boot-configuration settings | I53 | **Settings at boot:** difficulty, campaign-commander mode, run mode, language and gunship side come from the module's boot configuration, as they come from the command line in EECH. No DCS API is involved | `eech-campaign/crates/eech-dc/src/module.rs:43-54` [R-src] |
| G27 Player aircraft binding | I45 | **Player-unit binding:** a player's DCS unit is bound to an EECH group member through G3 names on PLAYER_ENTER_UNIT. Its landings and takeoffs arrive as LAND and TAKEOFF events, applied with the campaign's landing-lock rules | D6 [R-ref], D5 [R-dcs], D16 [R-ref] |

## 9. Six-criterion acceptance matrix

| Criterion | This map contributes | Remains | Proposed acceptance check (where) |
|---|---|---|---|
| 1 DCS call map | Two-pass inventory and a 1,126-item coverage ledger (0 outstanding); 53 rows; 30 evidence items; G1–G30; review findings F1–F6 resolved | Lead review of the ledger rules: the pattern and fallback rules listed and counted in section 10, and the method's residual limits there. DCS-side unknowns that do not block the mapping but need probes: D6 ids, D11 trigger, D15 and D18 schemas, D27, and P-CAP-1 to P-CAP-5 (D30) | Review of this record; then a DCS run listing every D item used, as present or absent (DCS) |
| 2 No world inside the campaign | Names the world code that holds campaign rules (I6, I11, I12, I36) and the services to move behind host identities | A mechanical check and a loud failure with no host | Static check: `reach.py`'s campaign→world and campaign→environment callee count (today 38 and 25 functions) falls to the calls behind declared host ports, with the ledger re-run; and a boot with no host fails by name (EECH_WORLD) |
| 3 Host events drive the campaign | Event table (section 5); G7, G8 | The causal seam (#74, arrival first) | Withhold or delay one arrival or loss and show campaign state change (EECH_WORLD, then DCS) |
| 4 No regression | Lists which rows change semantics under DCS (G6, G9, G12, G18) | Running the M4 pack after each move | Lebanon under EECH_WORLD, M4 pack holds, differences classified (EECH_WORLD) |
| 5 DCS-only risks | G19, G20, D1, D12, section 6 | Owned by `docs/m5-dcs-risks.md` | Module load, frame budget and threading, save and restore, multiplayer (DCS) |
| 6 Plan ready to build | Interaction rows and workarounds for the steps of the existing working draft, [EECH M5 migration plan and acceptance evidence](https://flying-dice.atlassian.net/wiki/spaces/SD/pages/12484650) (Confluence 12484650, the current unaccepted working draft, not reviewed). Steps 2 and 3 use I1–I13 and G1–G7; step 4 uses section 5 and G7, G8; step 5 uses section 6; step 6 uses G19; step 7 uses I44, I45 and the excluded replication group | Attaching this map to the draft, mapping each step to its rows, and the lead's review and acceptance of the plan | That draft's in-DCS checks (DCS-load, DCS-bind, DCS-task, DCS-causal, DCS-clock-budget, DCS-restore, DCS-multiplayer, DCS-campaign) (DCS) |

## 10. Unknowns and limitations

- **Coverage.** The ledger has no outstanding item, but it rests on a regex index, not a compiler. These are the residual limits a reviewer should test:
  - **No preprocessor.** Both arms of `#ifdef` count, and only `#if 0` is removed. This over-approximates; it does not drop code.
  - **Function definitions** are found by pattern. The check that 14,599 call sites in reachable campaign functions resolve, with 3 explained exceptions, bounds the risk on the campaign side only.
  - **Registrations.** `dispatch.py` matches `fn_*[...] = f` and `message_responses[...][...] = f`. It also scans every other array assignment of a defined function's name. That scan found one real table form outside the pattern, `campaign_screen_message_responses` (UI, excluded, F4), and 5 data-array false positives. A function pointer stored through a struct field (for example `briefing_substitutions [loop].function`) is not a table: `reach.py` walks its targets as address-taken references.
  - **Message sends** are matched only as `notify_local_entity` and `notify_client_server_entity` calls; 3 sends with a non-literal message id are resolved in the ledger.
  - **The attribute disposition** uses the implementing entity classes and the usage context. Every world- or session-implemented token that the campaign uses in an accessor, a creation argument or another expression is named in `ledger.py` with its row, including suppress-AI-fire (I36) and difficulty (I53). Three world-implemented tokens appear in campaign code only as the campaign entity's own `case` labels and registrations, and fall to the own-implementation fallback: `PTR_TYPE_TERRAIN_FACE_NORMAL`, `FLOAT_TYPE_RECON_DISTANCE` and `INT_TYPE_MAP_ICON`. That assignment is a judgement the reviewer can check line by line.
  - **Globals** are found from `extern` declarations in headers only. A global declared only in a `.c` file, or used through a function rather than a macro, is not in this ledger; its accessor function is in the call ledger instead.
  - **Pattern and fallback rules.** This is the one place the rule totals are stated; they are reproduced by `ledger.py` ("items by rule kind", "decided by pattern or fallback rules") on the embedded scripts.
    - Of the 1,126 items, exact rules and fixed tables decide 376, 52 pattern rules decide 601, and 5 fallbacks decide 149: four by attribute usage context, and one for the other-table forms, where no regex is involved. So 57 pattern and fallback rules decide 750 items, each listed below.
    - Each rule's reason applies to every item it matches. The per-item list (`items.json`, with the deciding rule id) lets a reviewer check them.
    - Since F3, no rule assigns a default list of rows. Where a rule names two rows, both apply to every item it matches. The former catch-alls ("I6, I12, I30", "I12, I13, I30, I32", "I5 and G3" for all lists, "I18, I19, I27 … or framework default") are gone.
    - Since C2, every rule that was hand-labelled exact at `ab830a04` but matches whole files or wildcards is counted as a pattern: 28 rules over 145 items (calls, writes and globals). These include the reviewer's 15 call rules over 88 items.
    - **Removed alternatives:**
      - Since C4 and C5, two call alternatives that named campaign callees (never caller keys) are gone, and so are two global alternatives: `^object_3d_enumeration_names$`, shadowed by `_names$`, and `^dik_names$`, redundant with it.
      - Since the audit of `96650e9e`, three more redundant alternatives are gone: `common/co_force\.c::` (shadowed by `gunships/dynamics/`; `co_force.c::update_collision_dynamics` handles the player aircraft's crash, so it is I45), `common/co_undercarriage\.c::` (inside `gunships/dynamics/`), and `gunships/guns_sc\.c::` (the file is under `ui_menu/`). The unreachable "tool" table verb is gone too.
      - None of these removals changed a disposition.
    - `hc_updt.c::update_server` and `rv_updt.c::update_server` each get one row from their measured callee: the sector lookup (I25) and the current task (I6).

| Ledger | Rule id | Disposition | Reason or row | Items |
|---|---|---|---|---|
| attributes | `fallback:campaign-only` | excluded | campaign-only: implemented only by campaign entity types | 86 |
| attributes | `fallback:own-implementation` | excluded | the campaign entity's own implementation of the attribute (case label, table registration, pack tag); reads no world value | 44 |
| attributes | `fallback:pack-only` | mapped | I38: a pack/unpack field tag (save, restore, replication) | 12 |
| attributes | `fallback:size-constant` | excluded | an array-size constant, not an attribute | 2 |
| calls | `pattern:calls#2` | excluded | platform services (memory, strings, files, fatal unwinding); no world | 6 |
| calls | `pattern:calls#3` | mapped | I44 (player commands, key-bound triggers) | 14 |
| calls | `pattern:calls#4` | mapped | I44 (player task choice, route planning, waypoint skip) | 27 |
| calls | `pattern:calls#5` | mapped | I41 (messages to players) | 3 |
| calls | `pattern:calls#7` | excluded | multiplayer replication and client join: headless is COMMS_MODEL_SERVER (eech_engine.c:144) with DirectPlay replaced (eech-campaign/crates/eech-engine-sys/build/main.rs:38-39); DCS replicates units | 6 |
| calls | `pattern:calls#8` | mapped | I45 (player aircraft: control, flight dynamics, landing, crash) | 15 |
| calls | `pattern:calls#9` | mapped | I41 (player displays of campaign state) | 65 |
| calls | `pattern:calls#10` | mapped | I18, I40 (campaign callbacks on the frame's update list) | 2 |
| calls | `pattern:calls#11` | mapped | I39 (lifecycle) | 4 |
| calls | `pattern:calls#13` | mapped | I38 (save, restore) | 39 |
| calls | `pattern:calls#14` | mapped | I25 (map bounds) | 5 |
| calls | `pattern:calls#15` | mapped | I10 (doors) | 3 |
| calls | `pattern:calls#16` | excluded | entity store used for campaign records; crossings through it are the message, table, attribute and write ledgers | 31 |
| calls | `pattern:calls#17` | excluded | debug and test code; no world effect | 27 |
| calls | `pattern:calls#18` | excluded | pure arithmetic | 25 |
| calls | `pattern:calls#19` | excluded | campaign data files read natively by the DLL with C stdio; no DCS API (the files ship with the module, I39) | 23 |
| calls | `pattern:calls#20` | excluded | EECH's own Tacview writer and shared-memory export; observers of the campaign | 2 |
| calls | `pattern:calls#22` | excluded | campaign screens, camera and influence-map images: presentation: the headless frame draws nothing and plays no sound (eech_engine.c:253-286) | 27 |
| calls | `pattern:calls#26` | mapped | I49 (troops) | 4 |
| calls | `pattern:calls#43` | mapped | I41 (speech to players) | 9 |
| calls | `pattern:calls#44` | mapped | I1 (a spawned mobile registers with its force and sector) | 7 |
| calls | `pattern:calls#45` | excluded | visual and audio effects; DCS renders its own | 8 |
| calls | `pattern:calls#46` | mapped | I2, I25 (a world object registers in its sector) | 9 |
| calls | `pattern:calls#47` | mapped | I31 (keysite structures) | 5 |
| calls | `pattern:calls#48` | mapped | I24 (bridges break road links) | 2 |
| calls | `pattern:calls#49` | mapped | I14 (losses) | 11 |
| calls | `pattern:calls#50` | mapped | I36 (target detection triggers engagement) | 3 |
| calls | `pattern:calls#51` | mapped | I8 (weapon choice and release) | 4 |
| calls | `pattern:calls#52` | mapped | I16 (weapon fired at a target) | 2 |
| calls | `pattern:calls#54` | mapped | I9 (line of sight) | 2 |
| calls | `pattern:calls#55` | mapped | I29, I31 (campaign entity types reuse world handlers; campaign-only in DCS) | 4 |
| calls | `pattern:calls#56` | mapped | I6 (movement reads the guide) | 12 |
| calls | `pattern:calls#57` | mapped | I21, I22 (elevation, surface) | 6 |
| calls | `pattern:calls#58` | mapped | I23 (forest triangles) | 2 |
| calls | `pattern:calls#59` | mapped | I2 (population objects) | 5 |
| calls | `pattern:calls#60` | mapped | I26 (3D object data) | 7 |
| calls | `pattern:calls#61` | mapped | I19 (time of day) | 1 |
| calls | `pattern:calls#62` | mapped | I27 (weather) | 3 |
| globals | `pattern:globals#0` | excluded | string tables for parsing and logging | 31 |
| globals | `pattern:globals#1` | excluded | entity-store attribute type tables and their diagnostics, reached through the accessor macros | 21 |
| globals | `pattern:globals#3` | excluded | debug drawing colours and 2D state (gd_draw.c, imaps.c, gd_cover.c) | 10 |
| globals | `pattern:globals#4` | excluded | campaign data image read natively (popread.c:362) | 2 |
| globals | `pattern:globals#7` | mapped | I2, I26 (3D object data, G14) | 2 |
| other tables | `fallback:other-tables` | excluded | not a function table: a data array assigned from a variable that shares a function's name (for example `.v = v;`) | 5 |
| writes | `pattern:writes#2` | mapped | I1 (spawn placement and state) | 12 |
| writes | `pattern:writes#3` | mapped | I30 (landing and takeoff sequencing, G16) | 4 |
| writes | `pattern:writes#4` | mapped | I4 (group membership, G3) | 3 |
| writes | `pattern:writes#5` | mapped | I10 (hangar and building doors) | 2 |
| writes | `pattern:writes#9` | mapped | I8 (attack internals, G6) | 16 |
| writes | `pattern:writes#10` | mapped | I23 (cover, G12) | 3 |
| writes | `pattern:writes#11` | mapped | I2 (population objects) | 2 |
| writes | `pattern:writes#12` | excluded | the receiver is a campaign entity (sector, waypoint, guide, keysite, group, cargo, landing); the token is also world-implemented, but this write is campaign state | 56 |
| writes | `pattern:writes#14` | excluded | SET_ENTITY_POSITION propagation to a campaign entity (landing site, regen, task, waypoint, cargo); I34 covers the moving keysite | 7 |

- **The classification is a choice.** Moving `guide/` or `landing/` to W would move their interactions between rows, not remove them.
- **DCS signatures are tslua-dcs's.** No shipped file gives one, the DLL is packed, and no DCS runtime check was made. Unknowns that a DCS probe would settle: D6 numeric ids and event duplication; the `groupData` and route-task schemas; D27; when D11 runs; whether names and ids are unique or reused; D14 behaviour.
- **The coordinate chain is unmeasured against DCS.** The retail maps are distorted (section 7), and there is no Syria terrain on this PC.
- **Human players and multiplayer** are now mapped (I44, I45, the replication group). The player-skip and cohesion rules inside the helicopter arrival test (#74 stage 4) still sit in world code and move with G7. No player path runs in the headless product, so none of this is runtime-characterised.
- **Capture conversion in DCS** depends on P-CAP-1 to P-CAP-5 (G28). Until they run, G28 is the named workaround, not a verified mechanism.
- **Prior research:** `eech-campaign/docs/dcs.md` maps the earlier kernel-spike API, not the whole-engine product. Where they differ, this record is source-grounded at `65892de1`.

## Validation

- **Scope:** documentation only. This revision edits only this file.
- **First pass:** `inventory.py` at `65892de1`; the output is as in section 1, and the grep cross-checks were run as listed.
- **Second pass:** from one directory holding `eechmap.py` and the four scripts below, run:
  ```
  python -I reach.py <repo> --detail reach.json
  python -I dispatch.py <repo> --detail dispatch.json
  python -I globals.py <repo> --detail globals.json
  python -I ledger.py reach.json dispatch.json globals.json --items items.json
  ```
  These were run at `65892de1` with Python 3.13 on Windows and give the figures in section 1, including a ledger of 1,126 items with 0 outstanding. The reviewer's `g2.py` and `writes.py` were run on the same tree, and the differences are explained in section 1. The scripts were audited against the eight clean-code principles (SRP, DRY, NAMING, COUPLING, DEAD, KISS, BOUNDARY, PANIC); no finding above 0.5 remains.
- **DCS:** read-only `grep`, `sed -n` and `diff` over `DCS:Scripts/`, `DCS:API/Sim_ControlAPI.md`, `DCS:autoupdate.cfg` and `DCS:Mods/terrains/`, plus `grep -a` over `DCS:bin/Scripting.dll`. tslua-dcs files were fetched at `1179554f` with `gh api`.
- **Confluence:** the plan draft 12484650 (the current unaccepted working draft) was read; it was not edited.
- **Citations:** a checker kept outside the repository resolves every `file:line` outside code blocks (repository, `DCS:`, `MT/`, `MTX/`; a bare `:N` refers to the last file on the same line) to one file and checks the line is in range. It also checks every relative link and backticked repository path at the commit.
- **Not run:** no DCS or EECH runtime, no build and no regression pack. Nothing here changes behaviour.

<details>
<summary><code>eechmap.py</code></summary>

```python
"""Shared source model for the M5 DCS call-map scripts.

The compiled file list, the path classes, comment and string stripping, line
numbers and the macro index. The scripts import it from their own directory.
"""
import collections
import re
from pathlib import Path

SYS = "eech-campaign/crates/eech-engine-sys/"
APHAVOC = "aphavoc/source/"
CAMPAIGN_DIRS = ("force", "group", "task", "guide", "keysite", "waypoint", "division", "regen", "landing", "sector")

# first match wins; "" matches everything
CLASS_RULES = [
    ("H", [SYS + "csrc/"]),
    ("C", [APHAVOC + "ai/", APHAVOC + "ai_extrn.c", APHAVOC + "entity/mobile/cargo/"]
          + [APHAVOC + "entity/special/" + d + "/" for d in CAMPAIGN_DIRS]),
    ("W", [APHAVOC + "entity/mobile/", APHAVOC + "entity/fixed/", APHAVOC + "entity/special/bridge/",
           APHAVOC + "entity/special/effect/"]),
    ("E", ["modules/3d/", APHAVOC + "misc/terr_db", APHAVOC + "misc/tod", APHAVOC + "misc/timeaccl",
           APHAVOC + "entity/special/session/", "modules/maths/random.c", "modules/system/time.c",
           "modules/system/timer.c"]),
    ("X", [APHAVOC + "ui_menu/", "modules/userint2/", "modules/graphics/", "modules/2d/", "modules/sound/",
           APHAVOC + "appsound/", APHAVOC + "app3d/", APHAVOC + "gunships/", APHAVOC + "entity/special/camera/",
           APHAVOC + "cdmusic.c"]),
    ("F", [""]),
]

# the source trees whose headers and C files the build includes
INCLUDE_TREES = ("aphavoc/source", "modules", SYS + "csrc", SYS + "compat")

IDENT = re.compile(r"\b[A-Za-z_]\w*\b")
KEYWORDS = frozenset("if for while switch return sizeof else do case break continue goto default typedef struct "
                     "union enum static extern const volatile register unsigned signed int char float double long "
                     "short void".split())
_COMMENT = re.compile(r"/\*.*?\*/|//[^\n]*", re.S)
_STRING = re.compile(r'"(?:\\.|[^"\\\n])*"|\'(?:\\.|[^\'\\\n])*\'')
FUNCTION_DEFINITION = re.compile(
    r"^(?:static\s+|extern\s+|inline\s+|__inline\s+)*[A-Za-z_][\w \t\*]*?\b([A-Za-z_]\w*)\s*"
    r"\(([^;{}()]|\((?:[^()]|\([^()]*\))*\))*\)\s*(?:#[^\n]*\n\s*)*\{", re.M)
FILE_SCOPE_NAME = re.compile(r"\b([A-Za-z_]\w*)\s*(?:\[[^\]]*\]\s*)*(?:=|;|,)")
ACCESS = re.compile(r"\b(get|set)_(local|client_server)_entity_(\w+?)\s*\(\s*[^,()]+(?:\([^()]*\))?\s*,\s*"
                    r"([A-Z0-9_]+_TYPE_[A-Z0-9_]+)")
_MACRO = re.compile(r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)(\([^)]*\))?((?:[^\n\\]|\\\n|\\.)*)", re.M)


def path_class(path):
    """the class of a repository path; the last rule's "" prefix matches every path, so next() always finds one"""
    return next(k for k, prefixes in CLASS_RULES if any(path.startswith(p) for p in prefixes))


def compiled_units(root, host=True):
    """sources.txt, plus the headless host layer csrc/*.c when host is true"""
    root = Path(root)
    lines = (l.strip() for l in (root / SYS / "sources.txt").read_text().splitlines())
    units = [l for l in lines if l and not l.startswith("#")]  # as build/main.rs reads it
    if host:
        units += sorted(p.relative_to(root).as_posix() for p in (root / SYS / "csrc").glob("*.c"))
    return units


def read_source(root, rel):
    return (Path(root) / rel).read_text(encoding="latin-1")


def _blank(match):
    return re.sub(r"[^\n]", " ", match.group(0))


def strip_comments(text):
    """comments become spaces; line numbers survive"""
    return _COMMENT.sub(_blank, text)


def _drop_if0(text):
    out, depth, skipping = [], 0, False
    for line in text.split("\n"):
        s = line.strip()
        if skipping:
            if re.match(r"#\s*if", s):
                depth += 1
            elif re.match(r"#\s*endif", s):
                depth -= 1
                skipping = depth > 0
            elif re.match(r"#\s*(else|elif)", s) and depth == 1:
                skipping = False
            out.append("")
        elif re.match(r"#\s*if\s+0\b", s):
            skipping, depth = True, 1
            out.append("")
        else:
            out.append(line)
    return "\n".join(out)


def strip_code(text):
    """comments, string and character literals become spaces; #if 0 blocks are emptied"""
    return _drop_if0(_STRING.sub(_blank, strip_comments(text)))


def line_of(text, pos):
    return text.count("\n", 0, pos) + 1


class Macros:
    """#define bodies from INCLUDE_TREES: macro name -> identifiers in its body (all definitions united)"""

    def __init__(self, root, is_defined=lambda name: False):
        """is_defined(name) says whether a function of that name exists; it decides the hidden-function case"""
        self.is_defined = is_defined
        self.bodies = collections.defaultdict(set)
        for tree in INCLUDE_TREES:
            for p in (Path(root) / tree).rglob("*"):
                if p.suffix.lower() in (".h", ".c"):
                    for m in _MACRO.finditer(strip_comments(p.read_text(encoding="latin-1"))):
                        params = set(IDENT.findall(m.group(2) or ""))
                        self.bodies[m.group(1)] |= set(IDENT.findall(m.group(3))) - params - KEYWORDS
        self._expanded = {}  # complete top-level expansions only

    def __contains__(self, name):
        return name in self.bodies

    def expand(self, name, seen=()):
        """identifiers a macro use reaches, recursively; a nested macro whose expansion defines nothing is kept by
        name, because it hides a same-named function (e.g. an #ifdef'd _ex variant). Only top-level results are
        cached: a nested result is cut by its caller's cycle guard and would make the output depend on call order."""
        if not seen and name in self._expanded:
            return self._expanded[name]
        out = set()
        for n in self.bodies.get(name, ()):
            if n in self.bodies and n not in seen and n != name:
                sub = self.expand(n, seen + (name,))
                out |= sub
                if not any(self.is_defined(x) for x in sub):
                    out.add(n)
            else:
                out.add(n)
        if not seen:
            self._expanded[name] = out
        return out


def body_end(text, brace):
    """index of the brace closing the one at text[brace]"""
    depth = 0
    for j in range(brace, len(text)):
        if text[j] == "{":
            depth += 1
        elif text[j] == "}":
            depth -= 1
            if depth == 0:
                return j
    return len(text)


def index_functions(text):
    """(file, name) -> (body start, body end, is_static); name -> [(file, is_static)]; file -> file-scope text"""
    funcs, by_name, filescope = {}, collections.defaultdict(list), {}
    for f, t in text.items():
        spans = []
        for m in FUNCTION_DEFINITION.finditer(t):
            name = m.group(1)
            if name in KEYWORDS or (spans and m.start() < spans[-1][1]):  # a keyword, or a match inside a body
                continue
            brace = t.index("{", m.end() - 1)
            end = body_end(t, brace)
            static = bool(re.match(r"(?:inline\s+|__inline\s+)*static\b|static\b", t[m.start():m.start(1)].strip()))
            funcs[(f, name)] = (brace, end, static)
            by_name[name].append((f, static))
            spans.append((m.start(), end))
        pieces, last = [], 0
        for s0, e0 in spans:
            pieces.append(t[last:s0])
            last = e0 + 1
        pieces.append(t[last:])
        filescope[f] = "".join(pieces)
    return funcs, by_name, filescope
```

</details>

<details>
<summary><code>inventory.py</code></summary>

```python
"""First pass of the M5 DCS call map: direct calls between path classes, static
messages, attribute tokens and named world-service tokens in campaign files.

python -I inventory.py <repo-root> [--detail out.json]

Kept as the first pass's record; reach.py, dispatch.py and globals.py close
its coverage holes. Function definitions are found with the first pass's own
pattern on purpose, so that its published figures reproduce.
"""
import collections
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))  # eechmap.py sits beside this script
from eechmap import ACCESS, compiled_units, line_of, path_class, read_source, strip_comments

FIRST_PASS_DEF = re.compile(r"^(?:static\s+|extern\s+|inline\s+)*[A-Za-z_][\w \t\*]*?\b([A-Za-z_]\w*)\s*"
                            r"\(([^;{}()]|\([^()]*\))*\)\s*\{", re.M)
CALL = re.compile(r"\b([A-Za-z_]\w*)\s*\(")
MSG = re.compile(r"\bnotify_(?:local|client_server)_entity\s*\(\s*(ENTITY_MESSAGE_[A-Z0-9_]+)")
CREATE = re.compile(r"\bcreate_(?:local|client_server)_entity\s*\(\s*(ENTITY_TYPE_[A-Z0-9_]+)")
CONTROL_WORDS = {"if", "for", "while", "switch", "return", "sizeof", "else", "do", "case"}
CROSSINGS = (("C", "W"), ("C", "E"), ("W", "C"), ("E", "C"))
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


def site(f, t, pos):
    return f"{f}:{line_of(t, pos)}"


def main(root, detail):
    units = compiled_units(root, host=False)
    text = {f: strip_comments(read_source(root, f)) for f in units}
    classes = {f: path_class(f) for f in units}
    defs = collections.defaultdict(set)  # name -> classes defining it
    for f, t in text.items():
        for m in FIRST_PASS_DEF.finditer(t):
            if m.group(1) not in CONTROL_WORDS:
                defs[m.group(1)].add(classes[f])
    ambiguous = sorted(n for n, k in defs.items() if len(k) > 1)
    edges, attrs = collections.defaultdict(list), collections.defaultdict(list)
    msgs, creates = collections.defaultdict(lambda: collections.defaultdict(list)), collections.defaultdict(list)
    for f, t in text.items():
        k = classes[f]
        defined_here = {m.start(1) for m in FIRST_PASS_DEF.finditer(t)}
        for m in CALL.finditer(t):
            n = m.group(1)
            if m.start(1) in defined_here or n in CONTROL_WORDS or len(defs.get(n, ())) != 1:
                continue
            target = next(iter(defs[n]))
            if (k, target) in CROSSINGS:
                edges[(k, target, n)].append(site(f, t, m.start()))
        for m in MSG.finditer(t):
            msgs[k][m.group(1)].append(site(f, t, m.start()))
        if k == "C":
            for m in ACCESS.finditer(t):
                attrs[m.group(4)].append(site(f, t, m.start()))
            for m in CREATE.finditer(t):
                creates[m.group(1)].append(site(f, t, m.start()))
    tokens = {cat: {name: [site(f, t, m.start()) for f, t in text.items() if classes[f] == "C"
                           for m in re.finditer(r"\b" + name + r"\b", t)] for name in names.split()}
              for cat, names in TOKENS.items()}
    print(json.dumps({
        "files_by_class": dict(sorted(collections.Counter(classes.values()).items())),
        "functions_defined_once_per_class": dict(collections.Counter(next(iter(v)) for v in defs.values() if len(v) == 1)),
        "names_defined_in_several_classes_excluded": len(ambiguous),
        **{f"{a}->{b}": {"functions": sum(1 for e in edges if e[:2] == (a, b)),
                         "sites": sum(len(v) for e, v in edges.items() if e[:2] == (a, b))} for a, b in CROSSINGS},
        "C attribute tokens": {"distinct": len(attrs), "sites": sum(map(len, attrs.values()))},
        "messages sent": {k: {"distinct": len(v), "sites": sum(map(len, v.values()))} for k, v in sorted(msgs.items())},
        "C create_*_entity types": {k: len(v) for k, v in sorted(creates.items())},
        "C service tokens per category [sites, files]": {
            c: [sum(len(s) for s in v.values()), len({x.rsplit(":", 1)[0] for s in v.values() for x in s})]
            for c, v in tokens.items()},
    }, indent=1))
    if detail:
        Path(detail).write_text(json.dumps({
            "edges": {f"{a}->{b} {n}": v for (a, b, n), v in sorted(edges.items())},
            "attrs": dict(sorted(attrs.items())), "msgs": {k: dict(sorted(v.items())) for k, v in msgs.items()},
            "creates": dict(creates), "ambiguous": ambiguous, "tokens": tokens}, indent=1))


if __name__ == "__main__":
    main(Path(sys.argv[1]), sys.argv[sys.argv.index("--detail") + 1] if "--detail" in sys.argv else None)
```

</details>

<details>
<summary><code>reach.py</code></summary>

```python
"""Reachability walk over the headless eech_dc C, for the M5 DCS call map.

python -I reach.py <repo-root> [--detail out.json]

Nodes: every function defined in a compiled C file (sources.txt and csrc/*.c),
keyed by (file, name), plus one data node per file (its file-scope text).
Edges: every identifier in a body (a call or an address taken), resolved in
the same file first, then to the unique non-static global definition; macros
are expanded recursively. Comments, literals and #if 0 blocks are removed.
Over-approximate by design: an address taken counts as a call, and both arms
of #ifdef count.
"""
import collections
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))  # eechmap.py sits beside this script
from eechmap import (FILE_SCOPE_NAME, IDENT, KEYWORDS, SYS, Macros, compiled_units, index_functions, line_of,
                     path_class, read_source, strip_code)

ENTRIES = ["eech_engine_boot", "eech_engine_frame", "eech_engine_objects", "eech_engine_clock",
           "eech_engine_prepare_installation", "eech_engine_diagnostics", "eech_engine_fatal_message"]
DATA = "<file-scope data>"
LIBC = frozenset("""abs acos asin atan atan2 atof atoi atol ceil cos exp fabs fclose feof fflush fgetc fgets floor fmod
fopen fprintf fputc fputs fread free fscanf fseek ftell fwrite getc isalpha isdigit isspace log log10 malloc memcmp
memcpy memmove memset pow printf qsort rand realloc sin sprintf sqrt srand sscanf strcat strchr strcmp strcpy strcspn
strlen strncat strncmp strncpy strrchr strstr strtok tan tolower toupper va_arg va_end va_start vsprintf snprintf
vsnprintf time exit strdup stricmp strnicmp _stricmp _strnicmp sqrtf fabsf floorf ceilf powf atan2f sinf cosf""".split())


class Resolver:
    def __init__(self, funcs, by_name, macros):
        self.funcs, self.by_name, self.macros = funcs, by_name, macros

    def resolve(self, f, name):
        """the definition a name in file f binds to: [] none, [("?ambiguous", name)] several"""
        if (f, name) in self.funcs:
            return [(f, name)]
        hits = list({x[0]: x for x in self.by_name.get(name, ()) if not x[1]})  # #ifdef'd twins in one file count once
        if len(hits) == 1:
            return [(hits[0], name)]
        return [("?ambiguous", name)] if hits else []

    def references(self, f, t, lo, hi):
        """(definition, line, identifier used) for every identifier in t[lo:hi]"""
        out = []
        for m in IDENT.finditer(t, lo, hi):
            n = m.group(0)
            if n in KEYWORDS:
                continue
            line = line_of(t, m.start())
            names = self.macros.expand(n) if n in self.macros and (f, n) not in self.funcs else {n}
            got = [(r, line, n) for x in names for r in self.resolve(f, x)]
            if n in self.macros and not any(r[0] != "?ambiguous" for r, _, _ in got):
                got += [(r, line, n) for r in self.resolve(f, n)]  # a macro hiding a same-named function
            out += got
        return out


def walk(text, funcs, filescope, resolver):
    """breadth-first from ENTRIES; returns reached nodes, edges and ambiguous-name counts"""
    declared_in = collections.defaultdict(set)
    for f, fs in filescope.items():
        for m in FILE_SCOPE_NAME.finditer(fs):
            declared_in[m.group(1)].add(f)
    start = [(SYS + "csrc/eech_engine.c", e) for e in ENTRIES]
    reached, queue = set(start), collections.deque(start)
    edges, ambiguous = collections.defaultdict(list), collections.Counter()
    while queue:
        node = queue.popleft()
        f, n = node
        if n == DATA:
            refs = [(r, 0, via) for r, _, via in resolver.references(f, filescope[f], 0, len(filescope[f]))]
        else:
            lo, hi, _ = funcs[node]
            refs = resolver.references(f, text[f], lo, hi)
            for m in IDENT.finditer(text[f], lo, hi):  # global data used: walk the declaring file's file scope
                for g in declared_in.get(m.group(0), ()):
                    if (g, DATA) not in reached:
                        reached.add((g, DATA))
                        queue.append((g, DATA))
        for r, line, via in refs:
            if r[0] == "?ambiguous":
                ambiguous[r[1]] += 1
                continue
            edges[(node, r)].append((line, via))
            if r not in reached:
                reached.add(r)
                queue.append(r)
    return {k for k in reached if k[1] != DATA}, edges, ambiguous


def unresolved_campaign_calls(text, funcs, reach, resolver, cls):
    """call-shaped names in reachable campaign functions that resolve to no function, macro or C library name"""
    examined, unresolved = 0, collections.Counter()
    for f, n in reach:
        if cls[f] != "C":
            continue
        lo, hi, _ = funcs[(f, n)]
        body = text[f][lo:hi]
        for m in re.finditer(r"\b([A-Za-z_]\w*)\s*\(", body):
            x = m.group(1)
            examined += 1
            if x in KEYWORDS or x in resolver.macros or x in LIBC or resolver.resolve(f, x):
                continue
            if re.search(r"\(\s*\*?\s*" + x + r"\s*\)", body):  # a function-pointer parameter called by name
                continue
            unresolved[x] += 1
    return examined, unresolved


def main(root, detail):
    units = compiled_units(root)
    text = {f: strip_code(read_source(root, f)) for f in units}
    cls = {f: path_class(f) for f in units}
    funcs, by_name, filescope = index_functions(text)
    missing = [e for e in ENTRIES if (SYS + "csrc/eech_engine.c", e) not in funcs]
    if missing:
        sys.exit(f"reach.py: entry points not found in {SYS}csrc/eech_engine.c: {', '.join(missing)}")
    resolver = Resolver(funcs, by_name, Macros(root, is_defined=lambda name: name in by_name))
    reach, edges, ambiguous = walk(text, funcs, filescope, resolver)
    examined, unresolved = unresolved_campaign_calls(text, funcs, reach, resolver, cls)
    cross = collections.defaultdict(list)
    for (a, b), sites in edges.items():
        ka, kb = cls[a[0]], cls[b[0]]
        if ka != kb and "C" in (ka, kb):
            cross[(ka, kb)].append((a, b, sites[0][0], sites[0][1]))

    def by_class(keys):
        return dict(sorted(collections.Counter(cls[f] for f, _ in keys).items()))

    print(json.dumps({
        "units": dict(sorted(collections.Counter(cls.values()).items())),
        "functions": by_class(funcs),
        "reachable_functions": by_class(reach),
        "reachable_files": dict(sorted(collections.Counter(cls[f] for f in {f for f, _ in reach}).items())),
        "unresolved_ambiguous_names": len(ambiguous),
        "call_sites_examined_in_campaign": examined,
        "unresolved_call_names_in_campaign": sorted(unresolved),
        "campaign_cross_edges": {f"{a}->{b}": [len({(x[0], x[1]) for x in v}), len({x[1] for x in v})]
                                 for (a, b), v in sorted(cross.items())},
    }, indent=1))
    if detail:
        Path(detail).write_text(json.dumps({
            "cross": {f"{a}->{b}": sorted({(x[0][0] + "::" + x[0][1], x[1][0] + "::" + x[1][1], x[2], x[3]) for x in v})
                      for (a, b), v in cross.items()},
            "ambiguous": dict(ambiguous),
            "unresolved_names": sorted(unresolved),
        }, indent=1))


if __name__ == "__main__":
    main(Path(sys.argv[1]), sys.argv[sys.argv.index("--detail") + 1] if "--detail" in sys.argv else None)
```

</details>

<details>
<summary><code>dispatch.py</code></summary>

```python
"""Dispatch tables of the headless eech_dc: message handlers, entity function
tables, attribute accessors and campaign writes to world-implemented
attributes, with the classes that register or receive them.

python -I dispatch.py <repo-root> [--detail out.json]
"""
import collections
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))  # eechmap.py sits beside this script
from eechmap import ACCESS, compiled_units, index_functions, line_of, path_class, read_source, strip_code, strip_comments

REGISTRATION = re.compile(r"\b(fn_\w+|message_responses)\s*\[\s*(ENTITY_TYPE_\w+|[a-z_]\w*)\s*\]"
                          r"(?:\s*\[\s*(\w+)\s*\])?\s*=\s*(\w+)\s*;")
TABLE_ASSIGNMENT = re.compile(r"\b([A-Za-z_]\w*)\s*((?:\[[^\]]+\]\s*)+)(?:\.\s*\w+\s*)?=\s*&?\s*([A-Za-z_]\w*)\s*;")
SEND = re.compile(r"\bnotify_(?:local|client_server)_entity\s*\(\s*((?:\([^()]*\)\s*)?[^,()]+(?:\([^()]*\))?)\s*,")
WRITE = re.compile(r"\bset_(?:local|client_server)_entity_\w+?\s*\(\s*([^,]+?)\s*,\s*([A-Z0-9_]+_TYPE_[A-Z0-9_]+)")
ATTRIBUTE = re.compile(r"(?<![A-Za-z0-9_])((?:INT|FLOAT|VEC3D|STRING|PTR|LIST|CHAR|MATRIX3X3)_TYPE_[A-Z0-9_]+)"
                       r"(?![A-Za-z0-9_])")
KNOWN_TABLES = ("fn_", "message_responses")


def registrations(text, cls):
    """(table, entity type, key, handler, file, line); a variable type index is "var:<class of the file>" """
    regs = []
    for f, t in text.items():
        for m in REGISTRATION.finditer(t):
            et = m.group(2) if m.group(2).startswith("ENTITY_TYPE_") else "var:" + cls[f]  # overload_*(type)
            regs.append((m.group(1), et, m.group(3), m.group(4), f, line_of(t, m.start())))
    return regs


def entity_type_classes(regs, cls):
    """entity type -> the class of the files that register its handlers"""
    votes = collections.defaultdict(collections.Counter)
    for _, et, _, _, f, _ in regs:
        votes[et][cls[f]] += 1
    return {et: (et[4:] if et.startswith("var:") else c.most_common(1)[0][0]) for et, c in votes.items()}


def other_function_tables(root, units):
    """array assignments whose right-hand side names a defined function, in tables REGISTRATION does not match"""
    code = {f: strip_code(read_source(root, f)) for f in units}
    _, by_name, _ = index_functions(code)
    found = collections.defaultdict(list)
    for f, t in code.items():
        for m in TABLE_ASSIGNMENT.finditer(t):
            if m.group(3) in by_name and not m.group(1).startswith(KNOWN_TABLES):
                found[m.group(1)].append(f"{f}:{line_of(t, m.start())}")
    return {k: {"assignments": len(v), "first": v[0], "class": path_class(v[0].rsplit(":", 1)[0])}
            for k, v in sorted(found.items())}


def messages(text, cls, handlers, etype):
    """static message types crossing the campaign boundary, and non-literal sends"""
    static = collections.defaultdict(lambda: collections.defaultdict(list))
    dynamic = []
    for f, t in text.items():
        for m in SEND.finditer(t):
            arg = re.sub(r"^\(\s*entity_messages\s*\)\s*", "", m.group(1).strip())
            site = f"{f}:{line_of(t, m.start())}"
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
    return static, crossing, dynamic


def usage_context(token, line):
    if re.search(r"\bcase\s+" + token, line):
        return "case"
    if re.search(r"\bfn_\w+\s*\[", line):
        return "registration"
    if re.search(r"\b(un)?pack_\w+\s*\(", line):
        return "pack"
    if "ENTITY_ATTR_" in line:
        return "create"
    if re.search(r"\b(get|set)_\w*entity\w*\s*\(|_entity_\w+\s*\(", line):
        return "access"
    return "other"


def implementing_classes(regs, etype):
    """attribute token -> classes of the entity types that register a get or set accessor for it"""
    impl = collections.defaultdict(set)
    for table, et, key, _, _, _ in regs:
        if re.fullmatch(r"fn_(get|set)_(?:local|client_server)_entity_\w+", table) and key \
                and re.fullmatch(r"[A-Z0-9_]+_TYPE_[A-Z0-9_]+", key):
            impl[key].add(etype.get(et, "?"))
    return impl


def attribute_rows(text, cls, impl):
    """every attribute token in campaign code: sites, implementing classes and usage contexts"""
    sites, contexts, in_accessor = collections.defaultdict(list), collections.defaultdict(collections.Counter), set()
    for f, t in text.items():
        if cls[f] != "C":
            continue
        lines = t.split("\n")
        for m in ACCESS.finditer(t):
            sites[m.group(4)].append(f"{f}:{line_of(t, m.start())}")
            in_accessor.add(m.group(4))
        for m in ATTRIBUTE.finditer(t):
            i = line_of(t, m.start())
            sites[m.group(1)].append(f"{f}:{i}")
            contexts[m.group(1)][usage_context(m.group(1), lines[i - 1])] += 1
    return {a: {"sites": len(s), "first": s[0], "implemented_by": sorted(impl.get(a, ())),
                "in_accessor_call": a in in_accessor, "contexts": dict(contexts.get(a, {}))} for a, s in sites.items()}


def world_writes(text, cls, impl):
    """campaign set_* calls on attribute tokens that a world entity type implements, with the receiver expression"""
    return [{"site": f"{f}:{line_of(t, m.start())}", "token": m.group(2), "receiver": m.group(1)}
            for f, t in text.items() if cls[f] == "C"
            for m in WRITE.finditer(t) if "W" in impl.get(m.group(2), ())]


def main(root, detail):
    units = compiled_units(root)
    text = {f: strip_comments(read_source(root, f)) for f in units}
    cls = {f: path_class(f) for f in units}
    regs = registrations(text, cls)
    etype = entity_type_classes(regs, cls)
    handlers = collections.defaultdict(set)
    for table, et, key, _, _, _ in regs:
        if table == "message_responses":
            handlers[key].add(et)
    static, crossing, dynamic = messages(text, cls, handlers, etype)
    impl = implementing_classes(regs, etype)
    attrs, writes = attribute_rows(text, cls, impl), world_writes(text, cls, impl)
    tables = collections.defaultdict(lambda: collections.defaultdict(set))
    for table, et, _, _, _, _ in regs:
        if table != "message_responses" and not re.fullmatch(r"fn_(get|set)_.*", table):
            tables[table][etype.get(et, "?")].add(et)
    others = other_function_tables(root, units)
    print(json.dumps({
        "registrations": len(regs),
        "entity_types": dict(collections.Counter(etype.values())),
        "messages_with_handlers": len(handlers),
        "static_message_types_sent": len(static),
        "crossing_static_messages": len(crossing),
        "dynamic_send_sites": len(dynamic),
        "attribute_tokens_in_campaign_code": len(attrs),
        "campaign_writes_to_world_implemented_attributes": len(writes),
        "function_tables": len(tables),
        "other_function_table_forms": len(others),
    }, indent=1))
    if detail:
        Path(detail).write_text(json.dumps({
            "etype": etype, "crossing": crossing, "dynamic": dynamic, "attrs": attrs, "writes": writes,
            "tables": {k: {c: sorted(v) for c, v in d.items()} for k, d in tables.items()},
            "handlers": {k: sorted(v) for k, v in handlers.items()}, "other_tables": others,
        }, indent=1))


if __name__ == "__main__":
    main(Path(sys.argv[1]), sys.argv[sys.argv.index("--detail") + 1] if "--detail" in sys.argv else None)
```

</details>

<details>
<summary><code>globals.py</code></summary>

```python
"""Shared global data the campaign reads or writes, directly or through a macro.

python -I globals.py <repo-root> [--detail out.json]

A global is a name declared `extern` in a header under the include trees
(every declarator of a comma list; function prototypes and function-pointer
declarations excluded, the latter being the dispatch tables of dispatch.py),
defined at file scope in a compiled non-campaign file, and referenced in a
campaign file either by name or through the recursive expansion of a macro
used there.
"""
import collections
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))  # eechmap.py sits beside this script
from eechmap import FILE_SCOPE_NAME, IDENT, INCLUDE_TREES, Macros, compiled_units, path_class, read_source, strip_comments

EXTERN = re.compile(r"^\s*extern\s+(?!\"C\")([^;]*);", re.M)


def extern_names(root):
    names = set()
    for tree in INCLUDE_TREES:
        for p in (Path(root) / tree).rglob("*.h"):
            for m in EXTERN.finditer(strip_comments(p.read_text(encoding="latin-1"))):
                decl = m.group(1)
                if "(" in decl:  # a prototype or a function-pointer table
                    continue
                decl = re.sub(r"\[[^\]]*\]|=[^,]*", "", decl)  # array bounds and initialisers
                for part in decl.split(","):
                    ids = IDENT.findall(part)
                    if ids:
                        names.add(ids[-1])
    return names


def definitions(text, externs):
    """extern name -> files defining it at file scope"""
    defined = collections.defaultdict(set)
    for f, t in text.items():
        depth = 0
        for line in t.split("\n"):
            if depth == 0 and not line.lstrip().startswith(("extern", "#", "typedef")):
                for n in FILE_SCOPE_NAME.findall(line):
                    if n in externs:
                        defined[n].add(f)
            depth = max(depth + line.count("{") - line.count("}"), 0)
    return defined


def main(root, detail):
    units = compiled_units(root)
    text = {f: strip_comments(read_source(root, f)) for f in units}
    externs = extern_names(root)
    defined = definitions(text, externs)
    foreign = {n for n, files in defined.items() if not any(path_class(d) == "C" for d in files)}
    macros = Macros(root)
    refs = collections.defaultdict(list)  # global -> [(site, identifier written in the campaign file)]
    for f, t in text.items():
        if path_class(f) != "C":
            continue
        for i, line in enumerate(t.split("\n"), 1):
            for n in set(IDENT.findall(line)):
                for g in ({n} | (macros.expand(n) if n in macros else set())) & foreign:
                    refs[g].append((f"{f}:{i}", n))
    rows = {g: {"defined_in": sorted(defined[g]), "classes": sorted({path_class(d) for d in defined[g]}),
                "sites": len(s), "direct_sites": sum(1 for _, via in s if via == g), "first": s[0][0],
                "via": sorted({via for _, via in s if via != g})}
            for g, s in sorted(refs.items())}
    print(json.dumps({"externs": len(externs), "non_campaign_globals_used_by_campaign": len(rows),
                      "only_through_macros": sum(1 for r in rows.values() if r["direct_sites"] == 0),
                      "by_class": dict(collections.Counter("/".join(r["classes"]) for r in rows.values()))}, indent=1))
    if detail:
        Path(detail).write_text(json.dumps(rows, indent=1))


if __name__ == "__main__":
    main(Path(sys.argv[1]), sys.argv[sys.argv.index("--detail") + 1] if "--detail" in sys.argv else None)
```

</details>

<details>
<summary><code>ledger.py</code></summary>

```python
"""Coverage ledger for the M5 DCS call map: every campaign crossing found by
reach.py, dispatch.py and globals.py gets one disposition: mapped to an
interaction row, excluded with a reason, or outstanding (no rule).

python -I ledger.py <reach.json> <dispatch.json> <globals.json> [--items out.json]

A rule's kind is derived from its regex (rule_kind): "exact" when every
alternative names its keys literally (a call key anchored "::name$", a global
"^name$", a write site "file:line|TOKEN|receiver$"), otherwise "pattern".
The totals report how many items each pattern rule decides.
"""
import collections
import json
import re
import sys

MAPPED, EXCLUDED, OUTSTANDING = "mapped", "excluded", "outstanding"
NO_RULE = ("unclassified", OUTSTANDING, "no rule", "none")


WILDCARD = re.compile(r"\\[wdsS]|(?<!\\)[.\[*+?]")
LITERAL_NAMES = r"[A-Za-z0-9_|()]+"
EXACT_FORMS = {  # per rule list: the shape of an alternative that names its keys literally
    "calls": re.compile(r"::" + LITERAL_NAMES + r"\$$"),
    "globals": re.compile(r"\^" + LITERAL_NAMES + r"\$$"),
    "writes": re.compile(r":[0-9|()]+\\\|[A-Z0-9_]+\\\|" + LITERAL_NAMES + r"\$$"),
}


def alternatives(rx):
    """the top-level alternatives of a regex (| outside parentheses and not escaped)"""
    out, depth, start, i = [], 0, 0, 0
    while i < len(rx):
        c = rx[i]
        if c == "\\":
            i += 2
            continue
        depth += {"(": 1, ")": -1}.get(c, 0)
        if c == "|" and depth == 0:
            out.append(rx[start:i])
            start = i + 1
        i += 1
    return out + [rx[start:]]


def rule_kind(rx, ledger):
    """exact when every alternative names its keys literally in the ledger's exact shape"""
    form = EXACT_FORMS[ledger]
    for alt in alternatives(rx):
        if WILDCARD.search(alt.replace("\\.", "")) or not form.search(alt):
            return "pattern"
    return "exact"


def rule_list(ledger, specs):
    """(regex, category, disposition, ref) specs -> rules carrying their ledger name, derived kind and rule id"""
    return [{"rx": re.compile(rx), "category": category, "disposition": disposition, "ref": ref,
             "id": f"{rule_kind(rx, ledger)}:{ledger}#{i}"} for i, (rx, category, disposition, ref) in enumerate(specs)]


def kind_of(rule_id):
    """"exact", "pattern", "fallback" or "none" from a rule id such as "pattern:calls#3" """
    return rule_id.split(":")[0]


def decide(rules, item_key):
    """(category, disposition, ref, rule id) of the first rule matching item_key"""
    for r in rules:
        if r["rx"].search(item_key):
            return r["category"], r["disposition"], r["ref"], r["id"]
    return NO_RULE


SERVER_ONLY = ("multiplayer replication and client join: headless is COMMS_MODEL_SERVER (eech_engine.c:144) with "
               "DirectPlay replaced (eech-campaign/crates/eech-engine-sys/build/main.rs:38-39); DCS replicates units")
NO_DRAW = "presentation: the headless frame draws nothing and plays no sound (eech_engine.c:253-286)"

# ---- 1. calls and references across the campaign boundary, keyed "path::function" -----------------------
CALL_RULES = rule_list("calls", [
    # headless host layer
    (r"csrc/eech_engine\.c::(boot|force_data_dump)$", "host", MAPPED, "I39 (boot, diagnostics)"),
    (r"csrc/eech_observe\.c::fill_group$", "host", MAPPED, "I4 (host observation surface)"),
    (r"csrc/eech_(headless_\w+|win32)\.c::", "host", EXCLUDED, "platform services (memory, strings, files, fatal unwinding); no world"),
    # players and multiplayer
    (r"misc/msg_in\.c::|events/ev_fns\.c::|system/event\.c::", "player", MAPPED, "I44 (player commands, key-bound triggers)"),
    (r"ui_menu/.*/(ca_selct|fflight|map)\.c::|common/co_wp\.c::", "player", MAPPED, "I44 (player task choice, route planning, waypoint skip)"),
    (r"misc/msg_out\.c::", "player", MAPPED, "I41 (messages to players)"),
    (r"en_comms/en_comms\.c::process_received_entity_comms_messages$", "player", MAPPED, "I44 (client task requests)"),
    (r"en_comms/en_comms\.c::|comms/(comms|comm_man)\.c::", "player", EXCLUDED, SERVER_ONLY),
    (r"special/pilot/|gunships/dynamics/|helicop\.c::(get_local_entity_suitable_for_player|notify_gunship\w*|helicopter_assume_player_control|helicopter_release_player_control|assign_entity_to_user|set_gunship_entity_to_external_view_entity|transmit_player_recon_data)$", "player", MAPPED, "I45 (player aircraft: control, flight dynamics, landing, crash)"),
    (r"gunships/(avionics|cockpits|views)/|hc_move\.c::get_waypoint_display_position$", "player", MAPPED, "I41 (player displays of campaign state)"),
    # lifecycle, time, persistence
    (r"aphavoc/source/update\.c::", "framework", MAPPED, "I18, I40 (campaign callbacks on the frame's update list)"),
    (r"gameflow/gameflow\.c::|aphavoc/source/(flight|init)\.c::|ui_menu/.*/mission\.c::", "framework", MAPPED, "I39 (lifecycle)"),
    (r"campaign/campaign\.c::campaign_completed$", "presentation", MAPPED, "I47 (campaign end)"),
    (r"::(pack|unpack)_|en_sessn\.c::|session/ss_pack\.c::", "persistence", MAPPED, "I38 (save, restore)"),
    # framework
    (r"en_main/en_world\.c::", "framework", MAPPED, "I25 (map bounds)"),
    (r"en_anim/en_anim\.c::", "framework", MAPPED, "I10 (doors)"),
    (r"en_(funcs|main|types|msgs|misc|attrs)/", "framework", EXCLUDED, "entity store used for campaign records; crossings through it are the message, table, attribute and write ledgers"),
    (r"en_debug/|system/assert\.c::|events/ev_debug\.c::|aphavoc/source/test\.c::|::debug_visuals$|fw_draw\.c::|::draw_mobile_entity_debug_info$|::display_debug_kill_info$", "framework", EXCLUDED, "debug and test code; no world effect"),
    (r"modules/maths/(?!random)|system/fpu\.c::|misc/miscell\.c::", "framework", EXCLUDED, "pure arithmetic"),
    (r"misc/(file_tag|language)\.c::|system/files\.c::|graphics/psdread\.c::", "framework", EXCLUDED, "campaign data files read natively by the DLL with C stdio; no DCS API (the files ship with the module, I39)"),
    (r"entity/tacview/|aphavoc/source/sharedmem\.c::", "framework", EXCLUDED, "EECH's own Tacview writer and shared-memory export; observers of the campaign"),
    # presentation
    (r"graphics/textuser\.c::read_map_info_data$", "presentation", MAPPED, "I46, I52 (map info, origin, season)"),
    (r"graphics/psdwrite\.c::|special/camera/|ui_menu/", "presentation", EXCLUDED, "campaign screens, camera and influence-map images: " + NO_DRAW),
    # world and environment, by function first
    (r"_msgs\.c::response_to_check_mobile_reached_guide$", "world", MAPPED, "I12 (per-member arrival test)"),
    (r"_msgs\.c::response_to_guide_cover_position_reached$", "world", MAPPED, "I23 (cover)"),
    (r"mb_msgs\.c::response_to_waypoint_(landed|land|taken_off)_reached$", "world", MAPPED, "I13 (typed arrivals)"),
    (r"mb_msgs\.c::response_to_waypoint_troop_\w+_reached$", "world", MAPPED, "I49 (troops)"),
    (r"mb_msgs\.c::response_to_waypoint_attack_action$", "world", MAPPED, "I8 (attack)"),
    (r"mb_msgs\.c::response_to_waypoint_(defend|loop|sub_route_navigation)_reached$", "world", MAPPED, "I12, I50 (navigation arrivals)"),
    (r"mb_msgs\.c::response_to_task_assigned$", "world", MAPPED, "I6 (task assigned to a member)"),
    (r"mb_msgs\.c::(attach_divison_to_keysite|get_destination_keysite)$", "world", MAPPED, "I32 (capture by troops)"),
    (r"_msgs\.c::response_to_collision$", "world", MAPPED, "I15 (collision damage)"),
    (r"mobile\.c::credit_client_server_mobile_kill$", "world", MAPPED, "I14 (kill credit)"),
    (r"mobile\.c::mobile_check_all_members_taken_off$", "world", MAPPED, "I13 (takeoff)"),
    (r"mobile\.c::insert_mobile_into_takeoff_route$", "world", MAPPED, "I30 (takeoff route)"),
    (r"routed\.c::stop_vehicles_on_route$", "world", MAPPED, "I24 (broken road link stops vehicles)"),
    (r"fixwing\.c::update_fixed_wing_crate_dropping$", "world", MAPPED, "I29 (crate drop)"),
    (r"hc_updt\.c::update_fuel_weight$|wn_misc\.c::get_local_entity_weapon_load_weight$", "world", MAPPED, "I7 (fuel and load)"),
    (r"helicop\.c::helicopter_within_keysite_area$", "world", MAPPED, "I30 (keysite area)"),
    (r"rv_updt\.c::update_server$", "world", MAPPED, "I6 (the routed vehicle's update reads its current task)"),
    (r"_vec3d\.c::set_local_vec3d$|hc_updt\.c::update_server$", "world", MAPPED, "I25 (moving objects update their sector registration)"),
    (r"_int\.c::get_local_int_value$", "world", MAPPED, "I6 (a world attribute derived from the campaign task)"),
    (r"_creat\.c::create_(client_server|local)_entity_(anti_aircraft|fixed_wing|helicopter|person|routed_vehicle|ship)$", "world", MAPPED, "I1 (spawn)"),
    (r"special/effect/soundeff/speech\.c::|::play_mobile_under_attack_speech$", "world", MAPPED, "I41 (speech to players)"),
    (r"mobile/.*_creat\.c::create_local$", "world", MAPPED, "I1 (a spawned mobile registers with its force and sector)"),
    (r"special/effect/|session\.c::attach_session_sound", "world", EXCLUDED, "visual and audio effects; DCS renders its own"),
    (r"fixed/\w+/\w+_creat\.c::create_local$", "world", MAPPED, "I2, I25 (a world object registers in its sector)"),
    (r"(site|siteupdt|cbuild)/|fixed/fixed\.c::", "world", MAPPED, "I31 (keysite structures)"),
    (r"bridge/", "world", MAPPED, "I24 (bridges break road links)"),
    (r"_dstry\.c::", "world", MAPPED, "I14 (losses)"),
    (r"(ac|vh)_tgt\.c::\w+_target_scan$|vh_wpn\.c::update_vehicle_missile_defence$", "world", MAPPED, "I36 (target detection triggers engagement)"),
    (r"ac_wpn\.c::|weapon/weapon\.c::|wn_tgt\.c::", "world", MAPPED, "I8 (weapon choice and release)"),
    (r"wn_creat\.c::", "world", MAPPED, "I16 (weapon fired at a target)"),
    (r"aircraft\.c::assess_aircraft_damage_level$", "world", MAPPED, "I11, I15 (damage, return to base)"),
    (r"mb_tgt\.c::", "world", MAPPED, "I9 (line of sight)"),
    (r"_msgs\.c::overload_\w+|mb_funcs\.c::overload_mobile_functions$", "world", MAPPED, "I29, I31 (campaign entity types reuse world handlers; campaign-only in DCS)"),
    (r"_move\.c::", "world", MAPPED, "I6 (movement reads the guide)"),
    (r"modules/3d/terrain/terrelev\.c::|misc/terr_db\.c::", "environment", MAPPED, "I21, I22 (elevation, surface)"),
    (r"modules/3d/terrain/terrdata\.c::get_terrain_3d_\w+_in_sector$", "environment", MAPPED, "I23 (forest triangles)"),
    (r"modules/3d/terrain/terrobjs\.c::|modules/3d/3dobjdb\.c::(construct|destruct)_3d_object$", "environment", MAPPED, "I2 (population objects)"),
    (r"modules/3d/3d(objdb|subobj|objvis|objid)\.c::", "environment", MAPPED, "I26 (3D object data)"),
    (r"misc/tod\.c::", "environment", MAPPED, "I19 (time of day)"),
    (r"session/session\.c::set_session_\w+_weather_mode$|session/ss_updt\.c::", "environment", MAPPED, "I27 (weather)"),
    (r"session/session\.c::check_point_inside_population_area$", "environment", MAPPED, "I25 (map population area)"),
])

# ---- 2. static message types that cross the boundary --------------------------------------------------
MESSAGES = {
    "CHECK_CAMPAIGN_OBJECTIVES": (MAPPED, "I17"), "CHECK_MOBILE_REACHED_GUIDE": (MAPPED, "I12"),
    "COLLISION": (MAPPED, "I15"), "ENTITY_FIRED_AT": (MAPPED, "I16"), "ENTITY_TARGETED": (MAPPED, "I36"),
    "FORCE_DESTROYED": (MAPPED, "I14, I15"), "GUIDE_COVER_POSITION_REACHED": (MAPPED, "I23"),
    "LANDING_SITE_REQUEST": (MAPPED, "I30"), "LOCK_LANDING_ROUTE": (MAPPED, "I30, I38"),
    "LOCK_LANDING_SITE": (MAPPED, "I30, I38"), "LOCK_TAKEOFF_ROUTE": (MAPPED, "I30"), "MOBILE_KILLED": (MAPPED, "I14"),
    "REQUEST_ASSISTANCE": (MAPPED, "I44"), "RESERVE_LANDING_SITE": (MAPPED, "I30"), "SET_ENTITY_POSITION": (MAPPED, "I34"),
    "TASK_ASSIGNED": (MAPPED, "I6"), "TASK_COMPLETED": (MAPPED, "I14, I31"), "TASK_TERMINATED": (MAPPED, "I6, I8, I45"),
    "UNLOCK_LANDING_ROUTE": (MAPPED, "I30"), "UNLOCK_LANDING_SITE": (MAPPED, "I30"), "UNLOCK_TAKEOFF_ROUTE": (MAPPED, "I30"),
    "UNRESERVE_LANDING_SITE": (MAPPED, "I30"), "WAYPOINT_DROP_OFF_REACHED": (MAPPED, "I13, I29"),
    "WAYPOINT_PICK_UP_REACHED": (MAPPED, "I13, I29"), "WAYPOINT_TAKEN_OFF_REACHED": (MAPPED, "I13"),
    "GROUND_FORCE_ADVANCE": (EXCLUDED, "sent only from debug keys (ev_debug.c)"),
    "LINK_CHILD": (EXCLUDED, "entity-store list maintenance"), "LINK_PARENT": (EXCLUDED, "entity-store list maintenance"),
    "UNLINK_CHILD": (EXCLUDED, "entity-store list maintenance"), "UNLINK_PARENT": (EXCLUDED, "entity-store list maintenance"),
}

# ---- 3. entity function tables, by verb; 3b. other function-pointer tables -----------------------------
TABLE_VERBS = {"create": (MAPPED, "I1"), "destroy": (MAPPED, "I3, I43"), "kill": (MAPPED, "I43"),
               "update": (MAPPED, "I40"), "pack": (MAPPED, "I38"), "unpack": (MAPPED, "I38"),
               "draw": (EXCLUDED, NO_DRAW), "play": (EXCLUDED, NO_DRAW)}
OTHER_TABLES = {
    "campaign_screen_message_responses": (EXCLUDED, "campaign-screen UI handlers (ca_group.c:1584-1588); fed only by notify_campaign_screen, itself excluded: " + NO_DRAW),
}
NOT_A_TABLE = "not a function table: a data array assigned from a variable that shares a function's name (for example `.v = v;`)"

# ---- 4. waypoint_database message ids -------------------------------------------------------------------
WAYPOINT_IDS = [
    (r"START_UP|TAXI|LIFT_OFF|TAKEN_OFF|RAISE_UNDERCARRIAGE|LOWER_UNDERCARRIAGE_ACTION|APPROACH|TOUCH_DOWN|LAND|LANDED", "I13 (G10)"),
    (r"NAVIGATION|SUB_ROUTE_NAVIGATION|CONVOY|REVERSE_CONVOY|LOOP|DEFEND|REPAIR", "I12 (G7)"),
    (r"PICK_UP|PREPARE_FOR_DROP_OFF|DROP_OFF|FINISH_DROP_OFF", "I12, I29 (G7)"),
    (r"TROOP_\w+|PREPARE_FOR_INSERTION", "I49 (G23)"),
    (r"RECON|RECON_ACTION", "I35 (G7)"),
    (r"ATTACK_ACTION", "I8 (G6)"),
]
DYNAMIC = {  # "file:line|argument" of each reviewed non-literal send; any other send is outstanding
    "aphavoc/source/entity/special/group/gp_msgs.c:1244|message":
        "I6: TASK_TERMINATED forwarded to the members (response_to_task_terminated, gp_msgs.c:1185)",
    "aphavoc/source/entity/special/waypoint/wp_msgs.c:271|msg":
        "I50: the waypoint's action message from waypoint_database, resolved in the waypoint-id ledger",
    "aphavoc/source/entity/special/waypoint/wp_msgs.c:440|msg":
        "I50: the waypoint's reached message from waypoint_database, resolved in the waypoint-id ledger",
}

# ---- 5. attribute tokens: every token named once -----------------------------------------------------------
ATTRIBUTES = {
    MAPPED: {
        "I4, I6 (position, motion; read with getPoint, getPosition, getVelocity)": "VEC3D_TYPE_POSITION VEC3D_TYPE_MOTION_VECTOR FLOAT_TYPE_VELOCITY FLOAT_TYPE_HEADING FLOAT_TYPE_PITCH VEC3D_TYPE_XV VEC3D_TYPE_ZV VEC3D_TYPE_RELATIVE_POSITION",
        "I12 (arrival state)": "FLOAT_TYPE_DISTANCE INT_TYPE_LANDED INT_TYPE_AIRBORNE_AIRCRAFT INT_TYPE_OPERATIONAL_STATE INT_TYPE_MEMBERS_PURSUING INT_TYPE_MEMBERS_REACHED INT_TYPE_MEMBERS_WAITING",
        "I14, I15 (alive, damage)": "INT_TYPE_ALIVE INT_TYPE_DAMAGE_LEVEL",
        "I14 (kill counts)": "INT_TYPE_KILLS",
        "I1 (spawn state)": "FLOAT_TYPE_MAIN_ROTOR_RPM INT_TYPE_INSIDE_HANGAR FLOAT_TYPE_CENTRE_OF_GRAVITY_TO_GROUND_DISTANCE INT_TYPE_ALIGN_WITH_TERRAIN FLOAT_TYPE_FUSELAGE_ANGLE",
        "I10 (doors, undercarriage)": "FLOAT_TYPE_LOADING_DOOR_TIMER FLOAT_TYPE_UNDERCARRIAGE_STATE",
        "I8 (attack internals, G6)": "FLOAT_TYPE_WEAPON_BURST_TIMER FLOAT_TYPE_WEAPON_LAUNCH_DELAY FLOAT_TYPE_WEAPON_TO_INTERCEPT_POINT_RANGE VEC3D_TYPE_WEAPON_TO_INTERCEPT_POINT_VECTOR VEC3D_TYPE_WEAPON_VECTOR INT_TYPE_WEAPON_AND_TARGET_VECTORS_VALID INT_TYPE_SELECTED_WEAPON INT_TYPE_WEAPONS_HOLD FLOAT_TYPE_ATTACK_ALTITUDE",
        "I36 (sensing, ROE, G9)": "FLOAT_TYPE_AIR_SCAN_RANGE FLOAT_TYPE_SURFACE_SCAN_RANGE INT_TYPE_ENGAGE_ENEMY INT_TYPE_RADAR_ON LIST_TYPE_TARGET",
        "I36 (suppress AI fire: no campaign engagements, and ROE weapon hold for DCS AI, G9)": "INT_TYPE_SUPPRESS_AI_FIRE",
        "I53 (difficulty: a boot setting the campaign reads; DCS unit skill at spawn, G30)": "INT_TYPE_DIFFICULTY_LEVEL",
        "I53 (campaign session flags from the campaign file)": "INT_TYPE_CAMPAIGN_REQUIRES_APACHE_HAVOC INT_TYPE_AUTO_ASSIGN_GUNSHIP",
        "I47 (campaign end, medal)": "INT_TYPE_SESSION_COMPLETE INT_TYPE_CAMPAIGN_MEDAL",
        "I18, I19 (elapsed time, time of day, day, start time)": "FLOAT_TYPE_ELAPSED_TIME FLOAT_TYPE_TIME_OF_DAY INT_TYPE_DAY FLOAT_TYPE_START_TIME",
        "I35 (fog of war maximum, a campaign parameter held by the session)": "FLOAT_TYPE_FOG_OF_WAR_MAXIMUM_VALUE",
        "I25 (population area bounds)": "FLOAT_TYPE_POPULATION_X_MAX FLOAT_TYPE_POPULATION_X_MIN FLOAT_TYPE_POPULATION_Z_MAX FLOAT_TYPE_POPULATION_Z_MIN",
        "I48 (type properties, G22)": "FLOAT_TYPE_CRUISE_ALTITUDE FLOAT_TYPE_CRUISE_VELOCITY FLOAT_TYPE_FUEL_DEFAULT_WEIGHT FLOAT_TYPE_FUEL_ECONOMY FLOAT_TYPE_POTENTIAL_SURFACE_TO_AIR_THREAT FLOAT_TYPE_POTENTIAL_SURFACE_TO_SURFACE_THREAT FLOAT_TYPE_TARGET_PRIORITY_AIR_ATTACK FLOAT_TYPE_TARGET_PRIORITY_GROUND_ATTACK INT_TYPE_FORCE_INFO_CATAGORY INT_TYPE_IDENTIFY_AIRCRAFT INT_TYPE_IDENTIFY_FIXED INT_TYPE_IDENTIFY_MOBILE INT_TYPE_IDENTIFY_VEHICLE INT_TYPE_IDENTIFY_WEAPON INT_TYPE_MOVEMENT_TYPE INT_TYPE_POINTS_VALUE INT_TYPE_TARGET_TYPE INT_TYPE_TASK_TARGET_TYPE INT_TYPE_ENTITY_SUB_TYPE STRING_TYPE_FULL_NAME STRING_TYPE_LONG_DISPLAY_NAME STRING_TYPE_SHORT_DISPLAY_NAME",
        "I26 (3D shape, G14)": "INT_TYPE_OBJECT_3D_SHAPE INT_TYPE_APPROXIMATION_OBJECT_3D_SHAPE",
        "I23 (cover, G12)": "VEC3D_TYPE_COVER_POSITION",
        "I31 (structure importance and groups)": "FLOAT_TYPE_FIXED_OBJECT_IMPORTANCE LIST_TYPE_BUILDING_GROUP LIST_TYPE_BUILDING_GROUP_ROOT",
        "I2 (city buildings)": "LIST_TYPE_CITY_BUILDING",
        "I35 (recon)": "FLOAT_TYPE_RECON_RADIUS",
        "I4 (side read: getCoalition; side writes are the write ledger and I51)": "INT_TYPE_SIDE",
        "I4 (group membership, G3)": "INT_TYPE_FORMATION_POSITION INT_TYPE_GROUP_MEMBER_ID INT_TYPE_GROUP_MEMBER_NUMBER LIST_TYPE_MEMBER LIST_TYPE_MEMBER_ROOT",
        "I45 (player: pilots, aircrew, player tasks)": "INT_TYPE_PLAYER LIST_TYPE_AIRCREW LIST_TYPE_PILOT LIST_TYPE_PILOT_ROOT LIST_TYPE_PILOT_LOCK LIST_TYPE_PILOT_LOCK_LINK LIST_TYPE_PLAYER_TASK LIST_TYPE_PLAYER_TASK_ROOT",
        "I49 (troops, G23)": "INT_TYPE_TROOPS_ONBOARD",
        "I30 (landing sites, takeoff queue)": "LIST_TYPE_LANDING_SITE LIST_TYPE_LANDING_SITE_LINK LIST_TYPE_LANDING_SITE_ROOT LIST_TYPE_TAKEOFF_QUEUE LIST_TYPE_TAKEOFF_QUEUE_ROOT",
        "I29 (cargo and supply records)": "LIST_TYPE_CARGO LIST_TYPE_CARGO_LINK LIST_TYPE_CARGO_ROOT LIST_TYPE_MOVEMENT_DEPENDENT LIST_TYPE_MOVEMENT_DEPENDENT_LINK FLOAT_TYPE_FUEL_SUPPLY_LEVEL",
        "I18 (group sleep timer)": "FLOAT_TYPE_SLEEP",
        "I40 (frame update list)": "LIST_TYPE_UPDATE LIST_TYPE_UPDATE_LINK",
        "I5 (waypoint altitude at creation; route-point altitude in G4)": "FLOAT_TYPE_ALTITUDE",
        "I5, G3 (campaign links among tasks, waypoints, guides, groups, keysites, forces, divisions, sectors, regen)": "LIST_TYPE_AIR_REGISTRY LIST_TYPE_AIR_REGISTRY_ROOT LIST_TYPE_AIR_TASK LIST_TYPE_ASSIGNED_TASK LIST_TYPE_ASSIGNED_TASK_LINK LIST_TYPE_ASSIGNED_TASK_ROOT LIST_TYPE_CAMPAIGN_OBJECTIVE LIST_TYPE_CAMPAIGN_OBJECTIVE_LINK LIST_TYPE_CAMPAIGN_OBJECTIVE_ROOT LIST_TYPE_COMPLETED_TASK LIST_TYPE_COMPLETED_TASK_LINK LIST_TYPE_COMPLETED_TASK_ROOT LIST_TYPE_CURRENT_WAYPOINT LIST_TYPE_CURRENT_WAYPOINT_LINK LIST_TYPE_CURRENT_WAYPOINT_ROOT LIST_TYPE_DIVISION LIST_TYPE_DIVISION_HEADQUARTERS LIST_TYPE_DIVISION_HEADQUARTERS_LINK LIST_TYPE_DIVISION_HEADQUARTERS_ROOT LIST_TYPE_DIVISION_LINK LIST_TYPE_DIVISION_ROOT LIST_TYPE_FOLLOWER LIST_TYPE_FOLLOWER_ROOT LIST_TYPE_FORCE LIST_TYPE_FORCE_LINK LIST_TYPE_GROUND_REGISTRY LIST_TYPE_GROUND_REGISTRY_ROOT LIST_TYPE_GROUP_LINK LIST_TYPE_GUIDE LIST_TYPE_GUIDE_LINK LIST_TYPE_GUIDE_ROOT LIST_TYPE_GUIDE_STACK LIST_TYPE_GUIDE_STACK_LINK LIST_TYPE_GUIDE_STACK_ROOT LIST_TYPE_INDEPENDENT_GROUP LIST_TYPE_INDEPENDENT_GROUP_ROOT LIST_TYPE_KEYSITE_FORCE LIST_TYPE_KEYSITE_FORCE_LINK LIST_TYPE_KEYSITE_FORCE_ROOT LIST_TYPE_KEYSITE_GROUP LIST_TYPE_KEYSITE_GROUP_ROOT LIST_TYPE_REGEN LIST_TYPE_REGEN_LINK LIST_TYPE_REGEN_ROOT LIST_TYPE_REGISTRY_LINK LIST_TYPE_SEA_REGISTRY LIST_TYPE_SEA_REGISTRY_ROOT LIST_TYPE_SECTOR LIST_TYPE_SECTOR_LINK LIST_TYPE_SECTOR_ROOT LIST_TYPE_SECTOR_TASK LIST_TYPE_SECTOR_TASK_LINK LIST_TYPE_SECTOR_TASK_ROOT LIST_TYPE_TASK_DEPENDENT LIST_TYPE_TASK_DEPENDENT_LINK LIST_TYPE_TASK_DEPENDENT_ROOT LIST_TYPE_UNASSIGNED_TASK LIST_TYPE_UNASSIGNED_TASK_LINK LIST_TYPE_UNASSIGNED_TASK_ROOT LIST_TYPE_WAYPOINT LIST_TYPE_WAYPOINT_LINK LIST_TYPE_WAYPOINT_ROOT",
    },
    EXCLUDED: {
        "not an attribute: a type or sub-type constant matched the accessor pattern": "ENTITY_SUB_TYPE_LANDING_FIXED_WING ENTITY_SUB_TYPE_LANDING_FIXED_WING_TRANSPORT ENTITY_SUB_TYPE_LANDING_HELICOPTER ENTITY_TYPE_FORCE",
        "list-table constants, not links": "LIST_TYPE_INVALID LIST_TYPE_LINK_FUNCTIONS_REQUIRED LIST_TYPE_ROOT_FUNCTIONS_REQUIRED",
        "visual effects unlinked when a campaign entity is destroyed (cg_dstry.c:133); DCS renders its own": "LIST_TYPE_SPECIAL_EFFECT LIST_TYPE_SPECIAL_EFFECT_ROOT",
        "the player's view padlock, unlinked on destroy (cg_dstry.c:137); cockpit view state": "LIST_TYPE_PADLOCK",
    },
}
# fallback by usage context, for tokens not named above (counted in the output)
CONTEXT_FALLBACKS = [
    ("pack-only", lambda impl, ctx, a: ctx and ctx <= {"pack"}, MAPPED, "I38: a pack/unpack field tag (save, restore, replication)"),
    ("own-implementation", lambda impl, ctx, a: ctx and ctx <= {"case", "registration", "pack"}, EXCLUDED, "the campaign entity's own implementation of the attribute (case label, table registration, pack tag); reads no world value"),
    ("size-constant", lambda impl, ctx, a: ctx == {"other"} and a.endswith("_MAX_LENGTH"), EXCLUDED, "an array-size constant, not an attribute"),
    ("campaign-only", lambda impl, ctx, a: impl and impl <= {"C"}, EXCLUDED, "campaign-only: implemented only by campaign entity types"),
]

# ---- 6. campaign writes to world-implemented attributes, keyed "file:line|TOKEN|receiver" -------------------
WRITE_RULES = rule_list("writes", [
    (r"keysite\.c:(1383|1390)\|INT_TYPE_SIDE\|(member|current_group)$", "world", MAPPED, "I51 (capture converts landed aircraft groups, G28)"),
    (r"keysite\.c:(1540|1546|1562|1568)\|INT_TYPE_SIDE\|(group|member)$", "world", MAPPED, "I51 (capture converts keysite building and regen structures, G28)"),
    (r"faction\.c:\d+\|(VEC3D_TYPE_POSITION|INT_TYPE_OPERATIONAL_STATE|FLOAT_TYPE_MAIN_ROTOR_RPM|FLOAT_TYPE_UNDERCARRIAGE_STATE|INT_TYPE_FORMATION_POSITION)\|member$|popread\.c:1454\|FLOAT_TYPE_HEADING\|member$", "world", MAPPED, "I1 (spawn placement and state)"),
    (r"ld_msgs\.c:\d+\|INT_TYPE_OPERATIONAL_STATE\|", "world", MAPPED, "I30 (landing and takeoff sequencing, G16)"),
    (r"(group|ld_msgs)\.c:\d+\|(INT_TYPE_FORMATION_POSITION|INT_TYPE_GROUP_MEMBER_NUMBER)\|member$", "world", MAPPED, "I4 (group membership, G3)"),
    (r"(ld_msgs|rg_updt)\.c:\d+\|FLOAT_TYPE_LOADING_DOOR_TIMER\|", "world", MAPPED, "I10 (hangar and building doors)"),
    (r"assign\.c:1350\|FLOAT_TYPE_FUEL_SUPPLY_LEVEL\|en$", "world", MAPPED, "I7 (helicopter fuel load)"),
    (r"assign\.c:1321\|INT_TYPE_TROOPS_ONBOARD\|en$", "world", MAPPED, "I49 (troops, G23)"),
    (r"guide\.c:(509|801)\|FLOAT_TYPE_DISTANCE\|(leader|member)$", "world", MAPPED, "I12 (the guide writes the member's range, G7)"),
    (r"_attck\.c:\d+\|(FLOAT_TYPE_WEAPON_LAUNCH_DELAY|INT_TYPE_SELECTED_WEAPON)\|aggressor$", "world", MAPPED, "I8 (attack internals, G6)"),
    (r"(_attck|gd_cover)\.c:\d+\|VEC3D_TYPE_COVER_POSITION\|aggressor$", "world", MAPPED, "I23 (cover, G12)"),
    (r"popread\.c:\d+\|INT_TYPE_(APPROXIMATION_)?OBJECT_3D_SHAPE\|city$", "world", MAPPED, "I2 (population objects)"),
    (r"\|(sector|wp|this_wp|next_wp|keysite|get_parser_keysite \(\)|incoming_group|landing|guide)$|(_attck|guide)\.c:\d+\|\w+\|en$|(cg_\w+|gp_dstry|ks_dstry|group|wp_creat)\.c:\d+\|\w+\|en$|keysite\.c:(543|1340)\|", "campaign", EXCLUDED, "the receiver is a campaign entity (sector, waypoint, guide, keysite, group, cargo, landing); the token is also world-implemented, but this write is campaign state"),
    (r"ks_msgs\.c:238\|FLOAT_TYPE_FUEL_SUPPLY_LEVEL\|receiver$", "campaign", EXCLUDED, "a keysite's supply level set on drop-off: campaign state (I29)"),
    (r"(ks|ld|rg|ts|wp|cg)_msgs\.c:\d+\|(VEC3D_TYPE_POSITION|FLOAT_TYPE_HEADING)\|receiver$", "campaign", EXCLUDED, "SET_ENTITY_POSITION propagation to a campaign entity (landing site, regen, task, waypoint, cargo); I34 covers the moving keysite"),
])

# ---- 7. shared global data ---------------------------------------------------------------------------------
GLOBAL_RULES = rule_list("globals", [
    (r"_names$|^application_tag_strings$", "globals", EXCLUDED, "string tables for parsing and logging"),
    (r"^(debug_fatal_invalid|overload_invalid)_\w+_message$|^null_entity_pointer_message$|^\w+_type_database$", "globals", EXCLUDED, "entity-store attribute type tables and their diagnostics, reached through the accessor macros"),
    (r"^(entities|first_used_entity|entity_type_database|force_local_entity_create_stack_attributes)$", "globals", EXCLUDED, "the entity store itself"),
    (r"^sys_col(_\w+|our_table)$|^active_2d_environment$", "globals", EXCLUDED, "debug drawing colours and 2D state (gd_draw.c, imaps.c, gd_cover.c)"),
    (r"^(number_of_)?psd_layers$", "globals", EXCLUDED, "campaign data image read natively (popread.c:362)"),
    (r"^(aircraft|vehicle|weapon|effect)_database$|^fixed_3d_object_database$|^target_point_offsets$", "globals", MAPPED, "I48 (type database, G22)"),
    (r"^speech_sector_coordinates$", "globals", MAPPED, "I41 (speech)"),
    (r"^objects_3d_\w+$", "globals", MAPPED, "I2, I26 (3D object data, G14)"),
    (r"^(terrain_3d_max_map_z|terrain_3d_map_minimum_height|world_map)$", "globals", MAPPED, "I21, I25 (map extents and minimum height)"),
    (r"^system_delta_time$", "globals", MAPPED, "I18 (frame delta, through get_delta_time)"),
    (r"^random_number_seed$", "globals", MAPPED, "I37 (the shared random stream, through frand1, sfrand1, rand16)"),
    (r"^session_entity$", "globals", MAPPED, "I18, I19, I27 (session: time and weather)"),
    (r"^update_entity$", "globals", MAPPED, "I40 (frame update list)"),
    (r"^current_map_info$", "globals", MAPPED, "I52 (season written from the campaign file), I46 (map info)"),
    (r"^(gunship_entity|pilot_entity)$", "globals", MAPPED, "I45 (player aircraft and pilot)"),
    (r"^command_line_capture_aircraft$", "globals", MAPPED, "I51 (the capture-aircraft setting, on by default, cmndline.c:151)"),
    (r"^(command_line_run_mode|session_camcom|global_options)$", "globals", MAPPED, "I53 (settings: run mode, campaign-commander mode, language, gunship side)"),
    (r"^(current_game_session|game_type|game_status|system_comms_model|system_comms_data_flow)$", "globals", MAPPED, "I39 (session, game type and status, server comms model)"),
])

# ---- 8. call-shaped names with no definition -----------------------------------------------------------------
UNRESOLVED = {"endif": "not a call: a #endif line followed by a parenthesised expression (ld_msgs.c:512)",
              "function": "a call through a function-pointer field (briefing.c:880); its targets are address-taken functions, walked as references",
              "ogre_scene_get_object": "in the #else arm of #ifndef OGRE_EE (popread.c:2795); the headless build does not define OGRE_EE"}


def ledger_entry(ledger, key, category, disposition, ref, rule_id, **extra):
    return {"ledger": ledger, "item": key, "category": category, "disposition": disposition, "ref": ref,
            "rule": rule_id, **extra}


def call_items(reach):
    out = []
    for direction, edges in reach["cross"].items():
        side = 1 if direction.startswith("C->") else 0
        for key in sorted({e[side] for e in edges}):
            out.append(ledger_entry("calls", key, *decide(CALL_RULES, key), direction=direction))
    return out


def message_items(dispatch):
    out = []
    for m, by in sorted(dispatch["crossing"].items()):
        name = m[len("ENTITY_MESSAGE_"):]
        d, ref = MESSAGES.get(name, (OUTSTANDING, "no rule"))
        out.append(ledger_entry("messages", name, "dispatch", d, ref, "exact", senders={k: v["first"] for k, v in by.items()}))
    return out


def table_items(dispatch):
    out = []
    for t in sorted(dispatch["tables"]):
        verb = "ui" if "ui_object" in t else t.split("_")[1]
        d, ref = (EXCLUDED, NO_DRAW) if verb == "ui" else TABLE_VERBS.get(verb, (OUTSTANDING, "no rule"))
        out.append(ledger_entry("tables", t, "dispatch", d, ref, "exact"))
    for t, r in sorted(dispatch["other_tables"].items()):
        d, ref = OTHER_TABLES.get(t, (EXCLUDED, NOT_A_TABLE))
        out.append(ledger_entry("other tables", t, "dispatch", d, ref, "exact" if t in OTHER_TABLES else "fallback:other-tables", first=r["first"]))
    return out


def dynamic_items(dispatch):
    return [ledger_entry("dynamic", site, "dispatch",
                         *((MAPPED, DYNAMIC[f"{site}|{arg}"]) if f"{site}|{arg}" in DYNAMIC else (OUTSTANDING, "no rule")),
                         "exact")
            for site, _, arg in dispatch["dynamic"]]


def waypoint_id_items(dispatch):
    out = []
    names = {k[len("ENTITY_MESSAGE_WAYPOINT_"):] for k in dispatch["handlers"] if k.startswith("ENTITY_MESSAGE_WAYPOINT_")}
    names |= {"END_REACHED", "WAIT_REACHED"}  # in waypoint_database, with no registered handler
    names -= {"REACHED", "ACTION"}  # the guide's generic messages: static, campaign to campaign
    for name in sorted(names):
        if not dispatch["handlers"].get("ENTITY_MESSAGE_WAYPOINT_" + name):
            out.append(ledger_entry("waypoint ids", name, "dispatch", EXCLUDED, "no handler registered: the message is a no-op", "exact"))
            continue
        base = re.sub(r"_REACHED$", "", name)
        ref = next((r for k, r in WAYPOINT_IDS if re.fullmatch(k, base)), None)
        out.append(ledger_entry("waypoint ids", name, "dispatch", MAPPED if ref else OUTSTANDING, ref or "no rule", "exact"))
    return out


def attribute_items(dispatch):
    named = {n: (d, ref) for d, groups in ATTRIBUTES.items() for ref, names in groups.items() for n in names.split()}
    out = []
    for a, r in sorted(dispatch["attrs"].items()):
        impl, ctx = set(r["implemented_by"]), set(r["contexts"])
        if a in named:
            d, ref = named[a]
            out.append(ledger_entry("attributes", a, "attributes", d, ref, "exact", first=r["first"]))
            continue
        fallback = next((f for f in CONTEXT_FALLBACKS if f[1](impl, ctx, a)), None)
        if fallback:
            out.append(ledger_entry("attributes", a, "attributes", fallback[2], fallback[3], "fallback:" + fallback[0], first=r["first"]))
        else:
            out.append(ledger_entry("attributes", a, "attributes", OUTSTANDING, "no rule", "none", first=r["first"]))
    return out


def write_items(dispatch):
    out = []
    for w in dispatch["writes"]:
        key = f"{w['site'].rsplit('/', 1)[-1]}|{w['token']}|{w['receiver']}"
        out.append(ledger_entry("writes", key, *decide(WRITE_RULES, key)))
    return out


def global_items(global_data):
    return [ledger_entry("globals", n, *decide(GLOBAL_RULES, n), first=r["first"]) for n, r in sorted(global_data.items())]


def unresolved_items(reach):
    return [ledger_entry("unresolved names", n, "index", EXCLUDED if n in UNRESOLVED else OUTSTANDING,
                 UNRESOLVED.get(n, "no rule"), "exact") for n in reach["unresolved_names"]]


def main(reach_path, dispatch_path, globals_path, items_path):
    with open(reach_path) as a, open(dispatch_path) as b, open(globals_path) as c:
        reach, dispatch, global_data = json.load(a), json.load(b), json.load(c)
    items = (call_items(reach) + message_items(dispatch) + table_items(dispatch) + dynamic_items(dispatch)
             + waypoint_id_items(dispatch)
             + attribute_items(dispatch) + write_items(dispatch) + global_items(global_data) + unresolved_items(reach))
    totals = collections.defaultdict(collections.Counter)
    for i in items:
        key = f"calls {i['direction']}" if i["ledger"] == "calls" else i["ledger"]
        totals[key][i["disposition"]] += 1
    report = {k: dict(v) for k, v in sorted(totals.items())}
    report["ALL"] = dict(collections.Counter(i["disposition"] for i in items))
    kinds = collections.defaultdict(set)
    for i in items:
        kinds[kind_of(i["rule"])].add(i["rule"])
    report["items by rule kind"] = dict(sorted(collections.Counter(kind_of(i["rule"]) for i in items).items()))
    report["pattern and fallback rules that decide items"] = len(kinds["pattern"] | kinds["fallback"])
    report["decided by pattern or fallback rules"] = dict(sorted(collections.Counter(
        f"{i['rule']} | {i['ref']}" for i in items if i["rule"].startswith(("pattern", "fallback"))).items()))
    report["ambiguous names left"] = len(reach["ambiguous"])
    print(json.dumps(report, indent=1))
    for i in items:
        if i["disposition"] == OUTSTANDING:
            print("OUTSTANDING", i["ledger"], i["item"])
    if items_path:
        with open(items_path, "w") as f:
            json.dump(items, f, indent=1)


if __name__ == "__main__":
    main(*sys.argv[1:4], sys.argv[sys.argv.index("--items") + 1] if "--items" in sys.argv else None)
```

</details>
