# The full engine, headless

The spike (`eech-sys`, `eech-campaign`) compiled a 40k-line closure of the
campaign C around one vertical slice. This part takes **all of EECH**: the
dynamic campaign, AI, pathfinding, flight, weapons, damage, comms and the
entity system. It compiles headless for x86-64 Linux, puts the engine behind
a Lua module (a DLL), and runs it from a Rust host process.

```
   eech-world (host binary)       Rust; embeds a Lua 5.1 state; records Tacview
        │  lua: require ("eech_dc") → boot { ... }, engine:frame (ms), engine:objects ()
        ▼
   eech-dc (cdylib: libeech_dc.so)     the Lua module; mlua 0.10 (lua51, module)
        │  eech-engine: Engine::boot / frame / objects / clock      (safe Rust)
        ▼
   eech-engine-sys                     native build + raw FFI
        │  csrc/: headless platform layer, boot and frame, observation, synthetic data
        ▼
   original EECH C                     ../aphavoc/source, ../modules: 1,217 files as written
```

Data comes from `eech-map`, which builds a map and a dynamic campaign from an
OpenStreetMap extract and SRTM elevation. Georgia (map16) is the one built so far. The retail Georgia campaign (map3) runs on retail data instead.

## What is compiled

The source list (`crates/eech-engine-sys/sources.txt`) is the maintained
Windows build's: every `.c` file of the `aphavoc` and `modules` Visual Studio
projects, 1,225 files. 1,217 compile as they are. csrc/ replaces the other
eight, which need a real Windows machine:

| Replaced | Headless replacement (csrc/) |
|---|---|
| `system/startup.c` (WinMain, window, message pump) | `eech_headless_system.c`: application globals and exit functions; the host drives the engine |
| `system/debug.c` (debug windows) | `debug_fatal` records the message and unwinds to the entry point |
| `system/joystick.c`, `userint2/ui_draw/uifont.c` | no joysticks; no glyphs (`string_to_utf8` is ported: it is text) |
| `graphics/dirdraw.c`, `graphics/f3d.c` | a memory-backed video screen and textures; no Direct3D device |
| `multi/directp.c`, `multi/dpguid.c` | no DirectPlay connection, so EECH runs as a local server |

The compiled code is EECH's `WIN32` path, built against `compat/`: the Win32
and DirectX names EECH uses, declared with LLP64 widths. COM calls are
declared without prototypes, as the originals were called, and fail with
`E_FAIL` (`csrc/eech_com_null.c`). EECH's sound, input and 3D backends
already treat that as "no device". Every Win32 and CRT call has a prototype,
so no pointer result passes through an implicit `int`.
`-Werror=implicit-function-declaration`, `implicit-int` and `int-conversion`
enforce that.

The build (`build/main.rs`) stages the tree into `OUT_DIR`, applies
`build/patches.rs` and compiles in parallel, in about two minutes. The source
tree is never modified.

### Source patches

| Id | File | Why |
|---|---|---|
| E1, E2 | `system/debug.h`, `ai/highlevl/highlevl.h` | T1 (`docs/64-bit.md`): the WIN32 release `debug_log`/`ai_log` macros are `#define x();` and are called with arguments. Only MSVC's preprocessor accepts that. |
| E3 | `entity/special/effect/explosn/xp_dbase.h` | Two anonymous structs of one union both declare `frequency` and `smoke_lifetime`, at the same offsets. MSVC's C++ front end accepts the repeated names; C does not. The second pair is renamed; the layout and every access are unchanged. |
| P1 | `entity/system/en_funcs/en_creat.c` | 64-bit blocker B1 (the spike's P1): the entity attribute `va_list` is read as the i386 argument stack. `csrc/eech_attrs.c` marshals it. |
| **B2** | `userint2/ui_sys/ui_attrs/ui_attrs.c` | **New 64-bit blocker.** Five UI attributes pass a `ui_object *` and read it back as `va_arg (pargs, int)`. Four graphic attributes read a pointer into an `int`. On x86-64 the pointer is truncated, and the first UI screen built crashes. The fix reads the arguments as pointers. |
| **X1** | `system/fpu.h` | **New 64-bit blocker.** The GNU `convert_float_to_int` / `convert_double_to_int` are `fistp (%1)`. In AT&T syntax an unsuffixed `fistp` is the 16-bit store, so only the low 16 bits of the int are written and values from 32,768 up saturate. Every world coordinate past 32.7 km then fell into the wrong sector (`get_x_sector`). On the Luxembourg map (since removed), the eastern half of the map shared one sector column; on Georgia, every keysite came out blue. The asm also popped the x87 stack without declaring it. The fix converts in C: `cvttss2si` truncates, which is the rounding EECH sets. |
| S1 | `entity/special/force/fc_msgs.c` | A keysite low on ammo or fuel is resupplied from the closest factory or refinery, or from the closest airbase if that is nearer. The airbase lookup excludes no keysite, so an airbase asking finds itself at 0 km and becomes its own supplier. It has no cargo of what it lacks, so no SUPPLY task is created and no airbase is ever resupplied. The patch passes the requester as `get_closest_keysite`'s exclude argument. |
| C1 | `3d/3dobjdb.c` | A scene's collision object 0 is the null object, meaning none. The `.EES` path maps 0 to −1, but the `3dobjdb.bin` path keeps it, and the retail database stores 0 (`RS_MANPAD`, for one). The first weapon tested against such a scene read the null object's NULL surface list. |
| T1, T2 | `graphics/textuser.c`, `3d/3dobjid.c` | Headless only. The community objects over a retail texture set disagree on which textures are camouflaged, and name texture animations the retail set lacks. Nothing is drawn headless, so both are logged instead of fatal. |
| W1 | `entity/mobile/weapon/wn_move.c` | `get_ballistic_pitch_deflection` takes `asin (height / range)`, and the aiming loop jitters the range by up to 5 m. At point blank the height can exceed the range: the pitch is NaN, its table index `INT_MIN`, and the ballistics table read faults. The patch returns "no solution". |

`-ftrivial-auto-var-init=zero` keeps the spike's F1 decision (`taskgen.c`
reads uninitialised locals).

### 64-bit audit of the whole tree

- **Implicit declarations:** after `compat/`, only Win32, DirectX and CRT names were implicit. They are now all declared.
- **Pointer/int casts** (`-Wpointer-to-int-cast`, `-Wint-to-pointer-cast`): 79 and 51 sites. Outside B2 they are:
  - UI callbacks that carry small integers in a `void *`;
  - `ks_creat.c` and `py_draw.c`, which take a pointer modulo 100 or 5 for a fallback name or a draw pattern;
  - `timer.c`, which passes a function through `timeSetEvent` (unused headless).
- **`long`** in the simulation code: only `_findfirst` handles and file lengths, both valid at 64 bits. The binary loaders read fixed-size fields.

## The platform layer (csrc/)

- **Files.**
  - EECH names files with Windows paths (`..\common\maps\map16\terrain`). `eech_native_path` converts `\` to `/` and resolves every component case-insensitively.
  - `fopen` in `"r"` mode reads as Windows text mode does: CR LF becomes LF, and Ctrl-Z ends the file. EECH's tag parser depends on it.
  - `CreateFileMapping` and `MapViewOfFile` are `mmap`, copy-on-write.
  - `EECH_TRACE_FILES=1` logs every file opened or missed.
- **Time** is simulated. `timeGetTime`, and every EECH clock with it, reads a millisecond counter that only the host advances (`eech_engine_frame (ms)`). Runs are reproducible: two runs of the same campaign and seed write byte-identical recordings.
- **Registry:** no keys, which is EECH's own "not installed" path. The installation path is set from the host's root.
- **Artwork:** the retail UI artwork is not in the repository. A read of a missing `.psd`, `.bmp` or `.tga` returns a valid 1×1 image, and each substitution is logged.
- **Failures:** `debug_fatal` unwinds (`longjmp`) to the entry point, which returns `EECH_ENGINE_FATAL` with the message. The engine is then poisoned: EECH state after a fatal error is not trusted again. Every entry saves the caller's FPU environment, runs EECH under round-toward-zero, and restores it.

## Boot and frame (`csrc/eech_engine.c`)

The boot follows EECH's own dedicated-server path and calls the same
functions in the same order:

1. `WinMain` (event, timer and file systems).
2. `application_main`: options, command line, `EECH.INI`, and the graphics system on memory surfaces.
3. `brief_initialise_game`, then the init screen, whose function runs `full_initialise_game`: maths, comms, sound, 3D objects and terrain, tags, entities, AI, players, ballistics.
4. The game-initialisation phases for a dedicated server. The campaign is parsed and created (`create_campaign`, `setup_campaign`, `create_server_pilot`).
5. `flight ()` up to its loop.

A frame is one iteration of `flight ()`'s loop without drawing:

- `receive_comms_data`
- `update_client_server_entities`
- `update_update_functions`, which runs the AI task generators
- the in-flight timers
- `set_delta_time` from the host's clock

## Observation (`csrc/eech_observe.c`)

`eech_engine_objects` reports what a viewer needs, classified as EECH's own
Tacview logger (`entity/tacview/tacview.c`) classifies it:

- helicopters, fixed wing, ground vehicles, air defence, ships and infantry;
- missiles, rockets and bombs in flight (not rounds);
- keysites.

Each object carries its EECH database name, side, position, attitude, alive
flag, group callsign, and the group's primary task.

## Data

### Generated by the engine (`eech_engine_prepare_installation`)

The retail data EECH cannot start without is not in the repository.

**The 3D object database** (`cohokum/3ddata`: `bininfo.bin`, `3dobjs.*`,
`3dobjdb.bin`) is written by `csrc/eech_synth3d*.c`.

- It uses EECH's own formats, and every write mirrors a read in `3dobjid.c` and `3dobjdb.c`.
- The name tables are the ones the engine compiled (`3dmodels.h`, `textanim.h`).
- Content:
  - Every scene has an object with no polygons and a bounding box sized from its name (aircraft, vehicle, building, and so on).
  - Airport and FARP scenes carry route sub-objects built as `routegen.c` reads them: line meshes; black start edges; the green primary route; one tree per slot; depth 0 the far end, the deepest level the slots on the ground. The routes are landing, takeoff and holding routes for fixed wing, helicopters and vehicles.
  - Airport scenes link hangars and a control tower, which `fx_objdb.c` rates as important. The hangars carry `REGEN_FIXED_WING`, `REGEN_HELICOPTER` and `REGEN_ROUTED_VEHICLE` sub-objects, so they become regen sites (`popread.c`, `regen.c`). Regen sites rebuild lost aircraft and vehicles from the force's hardware reserves.
  - Every aircraft scene carries a `WAYPOINT_ROUTES` sub-object with troop takeoff and landing routes: the path from the cabin door to where the troops stand. Without them, the first troop insertion is fatal ("Can't find proper troops take off route", `taskgen.c`).
  - Every aircraft and vehicle scene carries a weapon-system tree: `WEAPON_SYSTEM_HEADING`, then `PITCH`, then `MUZZLE` and `WEAPON`. The tree is as deep as the deepest `heading_depth`, `pitch_depth` and `muzzle_depth`, and as wide as the largest package count, over the `weapon_config_database` packages the type can carry (`csrc/eech_synth3d_weapons.c`). EECH aims through these devices. Without them, `WEAPON_AND_TARGET_VECTORS_VALID` is never set and no AI unit ever fires: `aircraft_fire_weapon` returns `WEAPON_SYSTEM_NOT_READY`.
- What the engine demanded, in order:
  1. landing and takeoff routes, or no landing sites;
  2. at least three route depths, so a primary node exists;
  3. takeoff routes that end on the landing slots;
  4. holding routes: a group whose members take off at different times sends the early ones to the takeoff-holding task, which is empty without them.

**Texture names** (`textures.bin`) are every system texture as a reserved
slot. The names are generated at build time from `modules/3d/textname.h`.

**Briefing texts** (`common/data/brief_en.dat`): one per task type.

### From the repository (`setup/common/data`)

Formation databases, the language database and the suspension tables.
`eech-map` also copies `setup/cohokum/GWUT1162.CSV`, the weapon and unit
tuning table (`eechini.c` `DEFAULT_GWUT_FILE`), into `cohokum/`. Weapon
weights, drag and motor power come only from it: the compiled weapon database
leaves them at zero, and a missile launched without the table flies with a
NaN velocity. It copies the explosion and smoke tables (`EXPLOS.CSV`,
`METASMOK.CSV`, `SMOKES.CSV`) too. Without `EXPLOS.CSV`, EECH exports its
compiled explosion database and runs on it, and that database declares
components it never sets: `XSMALL_HE_META_EXPLOSION` declares 5 and
initialises 3. The first small explosion then reads uninitialised heap.

### The map and campaign (`eech-map`)

`eech-map <extract.osm.pbf> <srtm dir> <installation root> [georgia]`
writes `common/maps/map16` (Georgia). The map is chosen by name or by the
extract's file name.

**Georgia** (`georgia-latest.osm.pbf`, SRTM N41–N43 × E040–E046) covers
40.4–46.7° E and 41.0–43.6° N: 256 × 142 terrain sectors, 524 × 291 km. EECH's
campaign map holds at most 128 campaign sectors a side (`map.c`
`MAP_OVERLAY_TEXTURE_SIZE`), which at 4 km is 524 km. The country's 560 km
therefore lose Abkhazia's north-west corner and a sliver of Lagodekhi.

- The front runs at 44.0° E, through the Shida Kartli plain by Gori and Tskhinvali, where the roads that frontline forces are placed on cross it.
- There are four FARPs a side along the front (41.45–42.25° N). The airbases are 70–160 km back, beyond most helicopter tasking, so the front's helicopters fly from the FARPs. The population load marks FARPs in use (`initialise_keysite_farp_enable`), which the script's keysite lookup skips, so only `FRONTLINE_FORCES` can populate them, with two groups each.
- Each side has a factory and a refinery on OSM industrial areas: blue near Ozurgeti and Supsa (Guria), red at Rustavi and east Tbilisi.
- Airbases: blue has Kutaisi and Senaki; red has Vaziani and Marneuli.
- Roads include tertiary: 8,723 nodes.
- Names use OSM's `name:en` where the local name is not in Latin script.
- The Black Sea (SRTM 0 m, with no OSM polygon) is sea terrain.

| File | From |
|---|---|
| `terrain/terrain.ffp`, `default.sec`, `default.rgb` | 256 × 142 sectors of 2,048 m. SRTM heights on a 256 m grid; one fan face per cell (SW, NW, NE, SE, clockwise as the elevation lookup's inside test requires). Faces are typed from OSM land cover (fields, forest, built-up, industrial, military, water). Per-point normals index EECH's 254-normal table. |
| `route/ROADS.dat/.nde/.wp` | OSM roads down to the spec's smallest class (Georgia: tertiary). Graph nodes are junctions, degree-2 chains are contracted, and only the largest component is kept (Georgia: 8,723 nodes), within EECH's 14-bit and 7-bit limits. |
| `route/popname.dat`, `bridge.pop` | OSM towns; aerodrome names |
| `camp01/georgia.sid` | the AI-sector side map (a Photoshop file): blue west of 44.0° E, red east |
| `camp01/georgia.pop` | Airbases: the two largest named aerodromes per side, at least 10 km apart (blue: Kutaisi and Senaki; red: Vaziani and Marneuli). A side with fewer gets one synthesised at its town farthest from its other airbase, off water. FARPs at the spec's latitudes, 0.085° behind the front. A factory and a refinery per side. Two air-defence sites per airbase. |
| `camp01/georgia.chc` | Campaign data and, for each force, its reserves, task generation, division numbers, frontline forces and the groups at its airbase |
| `mapinfo.txt` | the origin, `coordinate=41.0,40.4`: the same geodesy as EECH's Tacview writer, so map and recording agree |

### Retail data: the Georgia campaign (map3, "Caspian Black Gold")

`tools/retail-map3.sh <data> <root>` assembles an installation from retail
data kept outside the repository:

- a retail `cohokum/3ddata`, plus the community objects from
  `setup/cohokum/3ddata/objects`. The Steam Apache vs Havoc `3dobjdb.bin` holds
  2,761 of the 3,026 scenes this source defines, and a modern install adds the
  rest as `.EES` scenes;
- map3's campaign files, population, roads (`ROADDATA.*`, which EECH reads when
  `ROADS.*` is absent) and terrain (`terrain.ffp`, `default.sec`,
  `default.rgb`);
- the repository's formation, language and GWUT/explosion tables.

The retail `GEORGIA.SCR` ends the campaign as a FAIL after 30 minutes (a
`TIME_DURATION` trigger), and the installed copy drops that trigger.
`prepare_installation` keeps a retail 3D database (one that has
`textures.pal`).

Map3 is not metric. Correlating its terrain heights with SRTM (r = 0.978)
gives x = 1.217 e − 0.023 n + 258,100 and z = −0.0036 e + 0.991 n + 94,261,
where e and n are metres east and north of 42° N 43° E. The map is stretched
22% east–west. EECH's own Tacview origin for map3 (`textuser.c`: 41.16, 40.185,
treated as metric) is off by up to 50 km. The recorder takes the fitted
projection instead (`affine` in `campaign.lua`'s `georgia_retail`).

`tools/offsite.sh` keeps large install data and run outputs in one Google
Drive folder through rclone, configured from environment variables.

## The DLL and the host

```lua
local dc = require ("eech_dc")           -- the host must allow C modules
dc.prepare_installation (root)
local engine = dc.boot { install_root = root, map = "..\\common\\maps\\map16",
                         campaign = "georgia.chc", gunship = "apache", seed = 1 }
engine:frame (100)                        -- milliseconds of simulated time
for _, o in ipairs (engine:objects ()) do ... end
local clock = engine:clock ()
```

`eech-world <script.lua> key=value ...` embeds Lua 5.1 and lets the script
`require` the DLL. It exposes `host.args`, `host.log`, `host.open_recording`,
`host.record (time, objects)` and `host.event`, and writes Tacview ACMI 2.2:

- positions are written only when they change;
- a `Destroyed` event is written when an object dies;
- an entity index that EECH reuses for another object becomes a new ACMI object.

`lua/campaign.lua` runs a campaign for a number of hours and logs, every
simulated ten minutes:

- the units alive per side;
- the keysites each side holds;
- the tasks groups are on.

## Findings

1. **All of the maintained EECH builds and runs headless on x86-64.** It needs 3 source patches and 2 64-bit fixes. B2 is new: the whole tree has one more `va_arg` pointer truncation, in the UI.
2. **The dedicated-server path is the headless route.** It needed no new game logic. The one ordering constraint is that the dedicated-server flag must be set after the init screen, whose function would otherwise enter `flight ()` itself.
3. **Retail data is the real dependency.** The 3D database drives keysites (routes, landing sites, buildings), so it cannot be skipped. It can be synthesised from EECH's own formats and name tables.
4. **The campaign runs.** A generated campaign boots, generates and assigns tasks from the first minutes (BAI, CAS, recon, CAP, SEAD, ground and OCA strikes, OCA sweeps, advance and retreat, patrols, supply, troop insertion, transfers), and runs for simulated hours without a fault. Runs are deterministic. EECH seeds its random numbers from the system clock when the session starts. The simulated clock's epoch is set from the boot seed, so the seed selects the run.
5. **Combat runs.** Aircraft and ground units choose weapons, aim, launch, guide, hit and kill, and wrecks and weapons appear in the recording. Three pieces of data gate combat, and each fails silently: the weapon-system sub-objects (no fire at all), the GWUT table (NaN missiles), and regen sites plus reserves (air tasking stops once losses bring each group type down to EECH's minimum idle count).
6. **The campaign keeps reserves.** `assign.c` tasks a group only while more than `group_database[type].minimum_idle_count` idle groups of its type remain in the force's air registry, across the whole map: attack helicopters 2, recon-attack 3, fighters and CAS 1. Landed groups of one type merge into groups of up to four, which lowers the group count further. A campaign therefore needs more groups than the minimum per type, and it needs regen sites to replace losses.
7. **A campaign needs supply producers.** Airbases and FARPs only consume ammo and fuel (`ks_dbase.c`: an airbase uses 0.2 ammo and 0.4 fuel per keysite tick). Supply comes from factories (ammo), oil refineries (fuel) and ports, and SUPPLY missions fly it as cargo to the keysites that run low. Without producers every airbase drains to the 10% floor, and the air war stalls on rearming. `eech-map` puts a factory and a refinery on each side's largest OSM industrial areas behind the front. They are key templates in the population file, which `popread.c` turns into keysites only in version 2 files (negative template count, a routes object per template): the version 1 branch is commented out, but its `if` still guards the version 2 code.
8. **A side needs two airbases.** A REPAIR or SUPPLY task never starts from the keysite it serves (`taskgen.c`). A side whose only airbase is struck out of action can therefore never repair or resupply it, and its air groups stay "Repairing" for the rest of the campaign. The same happens when there are too few idle groups elsewhere to fly the task.
9. **Campaigns run for hours without a fault.** The recordings (`recordings/`) cover 12 simulated hours of Georgia: task generation and assignment, combat and attrition, regeneration from reserves, production, supply, repair and troop insertion. Before X1, a run on a generated map showed red capturing both blue airbases, but its sector lookups beyond 32.7 km were wrong. With correct maths, no generated war has changed a keysite's hands within those hours; the retail campaign does capture.
10. **The retail Georgia campaign barely reinforces, and has no victory condition.** Its `GEORGIA.CHC` sets `:REGEN_FREQUENCY 60000` (seconds between a regen site's attempts, `rg_updt.c`): 16.7 hours, where the other retail campaigns use 180–600 s. Its script only ends the campaign as a FAIL after 30 minutes, which the installed copy drops. `campaign.lua`'s `stop=conclusion` ends a run when a side holds no airbase or FARP, or after `stalemate_hours` with nothing captured or destroyed. Run that way, red wins in 61.8 hours (`recordings/README.md`).

## Limits

- **Linux only.** csrc/ is POSIX. A Windows build would compile EECH against the real SDK and replace only csrc/'s platform half.
- **No geometry in the synthetic objects:** only their bounding boxes and routes. Line of sight and weapon hits use bounding boxes and terrain; object meshes do not take part.
- **No player.** The engine is a dedicated server. The player's gunship and cockpit code is compiled but not driven.
- **Performance:** about four minutes of wall time per simulated hour for a generated map of roughly 2,300 units, in a release build.
