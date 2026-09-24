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
OpenStreetMap extract and SRTM elevation. Luxembourg is the one built so far.

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
  - EECH names files with Windows paths (`..\common\maps\map15\terrain`). `eech_native_path` converts `\` to `/` and resolves every component case-insensitively.
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

`eech-map <extract.osm.pbf> <srtm dir> <installation root>` writes
`common/maps/map15`.

| File | From |
|---|---|
| `terrain/terrain.ffp`, `default.sec`, `default.rgb` | 30 × 44 sectors of 2,048 m. SRTM heights on a 256 m grid; one fan face per cell (SW, NW, NE, SE, clockwise as the elevation lookup's inside test requires). Faces are typed from OSM land cover (fields, forest, built-up, industrial, military, water). Per-point normals index EECH's 254-normal table. |
| `route/ROADS.dat/.nde/.wp` | OSM motorway to secondary roads. Graph nodes are junctions, degree-2 chains are contracted, and only the largest component is kept: 3,946 nodes and 5,864 links, within EECH's 14-bit and 7-bit limits. |
| `route/popname.dat`, `bridge.pop` | OSM towns; aerodrome names |
| `camp01/luxembourg.sid` | the AI-sector side map (a Photoshop file): blue west of 6.07° E, red east |
| `camp01/luxembourg.pop` | Airfields: the largest named aerodrome on each side (Useldange for blue, Luxembourg Findel for red). Two FARPs per side, 6 km behind the front. Two air-defence sites per airbase. |
| `camp01/luxembourg.chc` | Campaign data and, for each force, its reserves, task generation, division numbers, frontline forces and the groups at its airbase |
| `mapinfo.txt` | the origin, `coordinate=49.4,5.7`: the same geodesy as EECH's Tacview writer, so map and recording agree |

## The DLL and the host

```lua
local dc = require ("eech_dc")           -- the host must allow C modules
dc.prepare_installation (root)
local engine = dc.boot { install_root = root, map = "..\\common\\maps\\map15",
                         campaign = "luxembourg.chc", gunship = "apache", seed = 1 }
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
4. **The campaign runs.** Luxembourg boots, generates and assigns tasks from the first minutes (BAI, CAS, recon, CAP, SEAD, ground and OCA strikes, OCA sweeps, advance and retreat, patrols, supply, troop insertion, transfers), and runs for simulated hours without a fault. Runs are deterministic.
5. **Combat runs.** Aircraft and ground units choose weapons, aim, launch, guide, hit and kill, and wrecks and weapons appear in the recording. Three pieces of data gate combat, and each fails silently: the weapon-system sub-objects (no fire at all), the GWUT table (NaN missiles), and regen sites plus reserves (air tasking stops once losses bring each group type down to EECH's minimum idle count).
6. **The campaign keeps reserves.** `assign.c` tasks a group only while more than `group_database[type].minimum_idle_count` idle groups of its type remain at the keysite: attack helicopters 2, recon-attack 3, fighters and CAS 1. A campaign therefore needs more groups than that per type, and it needs regen sites to replace losses.

## Limits

- **Linux only.** csrc/ is POSIX. A Windows build would compile EECH against the real SDK and replace only csrc/'s platform half.
- **No geometry in the synthetic objects:** only their bounding boxes and routes. Line of sight and weapon hits use bounding boxes and terrain; object meshes do not take part.
- **No player.** The engine is a dedicated server. The player's gunship and cockpit code is compiled but not driven.
- **Performance:** about four minutes of wall time per simulated hour for Luxembourg's roughly 2,300 units, in a release build.
