# M5 criterion 5: DCS hosting risk register

This register covers the DCS-only risks for hosting `eech_dc.dll` in the DCS **mission**
scripting environment: native module loading, frame budget and threading, save and restore,
and multiplayer. It feeds the migration plan (Confluence 12484650, still a draft). The
authoritative goal is Confluence 11534337 v1, criterion 5.

**Status: criterion 5 is not met.**
- One real DCS mission-state probe ran (2026-10-21). It proved that the module loads in the
  mission state, then DCS exited `0xC0000005` before another probe record. The first native
  error path is the next statement, so attribution remains inferred. The lead read the run
  independently (EEDCS-12 comment 10444).
- Campaign boot, frames, save/restore and multiplayer have **not** run in DCS.
- Retained evidence: [reference/m5-dcs-probes](../reference/m5-dcs-probes/README.md).
- Probe tooling: [tools/dcs-probe](../tools/dcs-probe/README.md).

| Input | Identity |
|---|---|
| Candidate | `eech_dc.dll` `a99d7a11…3f41`, built from `65892de1` (`x86_64-pc-windows-gnu`, mlua 0.10.5 / mlua-sys 0.6.8). The evidence-only merge `163e9158` leaves it unchanged. |
| DCS | openbeta `2.9.29.27468` MT (`autoupdate.cfg`). The retained `bin-mt/lua.dll` hash is `8a228b54…b755`; no `bin/lua.dll` hash is retained. |
| Environment | Profile `-w DCS.M5Probe`, single player, **unauthorised**: the login was cancelled, so paid modules are skipped. Launched hidden. This installation's `MissionScripting.lua` sanitisation was already commented out. |

Evidence classes:
- **R**: reproduced in actual DCS (the reference run named).
- **C**: standalone control outside DCS (lead or microprobe, by attribution).
- **S**: source- or static-proven (file cited).
- **V**: vendor text in the installation.
- **I**: inferred.
- **U**: unknown.

DCS paths are relative to the installation root. Repository paths are relative to
`eech-campaign/`, except `../`. `csrc/` means `crates/eech-engine-sys/csrc/`.

## The two Lua states that matter

| | Mission scripting state (where the campaign must run) | GUI / hooks state |
|---|---|---|
| Entry | A mission trigger `DO SCRIPT`, run under `Scripts/MissionScripting.lua` | `$WRITE_DIR/Scripts/Hooks/*.lua`, isolated environments (V: `API/Sim_ControlAPI.md:22-37`) |
| Observed globals | `require`, `package`, `io`, `os`, `lfs`, `net`, `world`, `timer`. **`DCS` and `Sim` are nil.** (R: run 9 `probe.log`) | `DCS.*`/`Sim.*`, `net.*`, `log.*`, `lfs` (V) |
| Time | `timer.getTime ()` (model time) and `timer.scheduleFunction`. MISSION START ran at model time 0. (R/S) | `DCS.getModelTime`, `getRealTime` (V) |
| cwd | Installation root `D:\Program Files\Eagle Dynamics\DCS World OpenBeta\`. Relative `Scripts/…` paths are visible. (R) | n/a |

Units, tasking and persistence live only in the mission state. Starting a mission from a GUI hook
did not work (see L4). A launcher therefore uses DCS's command line `--mission <miz>`, or a human
starts the mission.

## Register

### Native module loading

| ID | Risk | Evidence | Open | Next probe and what it decides |
|---|---|---|---|---|
| L1 | Binding to the right Lua | **R:** `require ("eech_dc")` in the mission state with `dll_dir` prepended to `package.cpath`: `ok`, `eech-dc 0.1.0`, 1.0 ms. **S:** the 83 `lua.dll` imports all resolve against DCS's 130 exports, the module is PE32+ x86-64, exports `luaopen_eech_dc`, has no delay imports and needs no MinGW runtime (`reference/m5-dcs-probes/link-check.stdout.txt`: PASS, link level, input hashes retained). **S:** `.cargo/config.toml:8` points `LUA_LIB` at `crates/eech-dc/lua5.1`, which has no `lua.lib` (`*.lib` is git-ignored), so Windows builds depend on the Docker override. | None for loading. | Closed for load. **Plan:** deploy `eech_dc.dll` alone, never the candidate's PUC `lua.dll`, and fix or document the `LUA_LIB` input. |
| L2 | **The first native error crashes the host** (blocking) | **R:** run 9. After `load ok`, the next statement is `pcall (dc.boot, "not a table")`. DCS exits `0xC0000005` with no further record. **C (lead, EEDCS-12 comment 10439):** on DCS `lua.dll`, the same call crashes `eech-world` at `0x0`. `host.log ({})`, a native **host** callback error, also crashes without the module. Plain `pcall (error)` passes. The candidate PUC Lua passes all of these. **C (microprobe, comment 10440):** DCS Lua is built with `LUA_IDSIZE 256` (`lua_Debug` 320 bytes). mlua-sys 0.6.8 hard-codes 60 (120 bytes). DCS writes 316 bytes during mlua's traceback, a 196-byte stack overrun. Patching only IDSIZE to 256 passes 60/60 per compiler, runtime and host. **C (actual product, lead-verified, comment 10446; `reference/m5-dcs-probes/product-abi/`):** `eech_dc` and `eech-world` were built from an archive of `65892de1` with only `LUA_IDSIZE` 60→256 in mlua-sys 0.6.8 (`mlua-sys-idsize256.diff`; no compiler change) in the pinned image `0c60c75d…`. Patched DLL `34ad0c2b…4557c`, host `d4190538…7c62e`. **Standalone controls:** all 36 patched runs exit 0 (6 controls × DCS and PUC Lua × 3), covering a bad module argument, 100 module errors, a host-callback error, plain errors and teardown. The stock build still exits `0xC0000005` in `first-error`, `host-log` and `lua-compat` on DCS Lua. **The unchanged full M4 pack passes** on the patched build (`pack-patched.report.json` `75ae2cd2…`: 17 directly protected, 12 baseline-sensitive, 8 evidence only, 8 not covered; all 40 checks PASS, with `regress.georgia.exact` and `regress.lebanon.exact` IDENTICAL to the baselines). | **I:** that the crash in run 9 is inside that call (that probe version had no begin marker; it has now). **Open:** the fix is a disposable archive-plus-patch experiment (`pack_revision` blank), not yet a reviewed product-source change, and **no corrected build has run in a DCS mission (N2)**. | **N1: done, PASS** (standalone, the actual product; above). The plan therefore adopts the `lua_Debug` size correction (a DCS-gated mlua-sys patch) as a bounded product Task before any DCS campaign run; no other `luaconf` deviation was found in the layouts checked (`lua_Debug`, `luaL_Buffer`, `lua_Integer`, `TValue`); others are unknown. **N2** (prerequisites: that product change built, and a DCS session past the login: `-CancelLogin` needs native dialog automation, unavailable to the current agent, or a stored login): `run-dcs-probe.ps1 -Stage load -Candidate <corrected> -CancelLogin`, expecting `error_path_begin` followed by `error_path raised=true`, and a run that ends by the hook's `exitProcess` or the runner bound, not by `0xC0000005`. |
| L3 | Compiler or unwinding hypothesis | **Superseded.** This register formerly named MSVC `longjmp` crossing MinGW frames as "the earliest crash-class unknown" and asked whether an MSVC build was required. The microprobe found stock MSVC modules also fail (wrong error values, heap corruption, `0xC0000005`), and the patched GNU module crosses DCS `longjmp` cleanly. **A compiler port is not a remedy and is not recommended.** | | None. |
| L4 | Starting a mission for an unattended probe | **R:** with no stored login, DCS stops at a hidden native `DCS Login` dialog. Copying `network.vault` or `authdata.bin` into a new profile does not authenticate (runs 2–5). Cancel followed by OK continues unauthorised (runs 7–9; run 6 records Cancel but no OK click). A GUI hook loads, but neither an `UpdateManager` updater nor `onShowMainInterface` was ever called at the menu (runs 7–8). `--mission <miz>` on the command line loads the mission at `END OF INIT` (run 9). **S/V:** the hooks' documented callbacks include none for the menu or idle (`Sim_ControlAPI.md:573-698`). The GUI itself starts missions through `START_PARAMS.command='--mission'` (`MissionEditor/modules/me_mission.lua:8464-8491`). | Whether an authenticated profile behaves the same; why the hook updater does not run. | **Decision for the lead:** stay unauthorised (free content, demonstrated for mission load only; the native error path has not completed in DCS), or have the owner log in once to `DCS.M5Probe` with "Save password". Paid assets or multiplayer need the latter. |

### Process-global state and lifecycle (needed before boot or frames)

| ID | Risk | Evidence | Open | Next probe and what it decides |
|---|---|---|---|---|
| B1 | Boot changes DCS's working directory and never restores it | **S:** `chdir ("<root>/cohokum")` at `csrc/eech_engine.c:232-238`. EECH resolves its data from there (`crates/eech-engine/src/lib.rs:70-72`). This already broke a relative load in eech-world (`docs/m3-resupply-loop.md:93`). **R:** the mission state's cwd is the installation root, and `MissionScripting.lua:3` loads `Scripts/ScriptingSystem.lua` relatively. | Whether DCS, the next mission load or other DCS threads depend on the cwd after boot. | **N3** `-Stage boot` (only after N1/N2; the runner refuses it without `-ErrorControlsEvidence`). Read `cwd_after` and `relative_scripts`, then load a second mission in the same process. If it is not visible or the load fails, the plan's first campaign change is absolute EECH paths, or save and restore the cwd around each entry (not thread-safe if DCS threads use relative paths). |
| B2 | One boot per process, no shutdown | **S:** `BOOTED` is set before validation (`lib.rs:168-171`). A fatal error poisons the engine for good (`csrc/eech_headless_system.c:131`). There is no `Drop`/shutdown (`lib.rs:140-265`). | Whether DCS closes the mission `lua_State` (and so unloads the DLL) between missions. | N3 run twice in one DCS process. The result chooses the M6 lifecycle contract: refuse visibly, or re-initialise explicitly (lead agreement needed). |
| B3 | Logs are lost | **S:** the engine log sink is stderr (`csrc/eech_headless.h:37-40`), and `eech_dc` installs none. | | Plan: route the sink to `env.info`. |
| B4 | Hidden threads or timers | **S:** the headless shims start none (`csrc/eech_win32.c:884-967`). | | Low risk. |

### Frame budget and threading

| ID | Risk | Evidence | Open | Next probe and what it decides |
|---|---|---|---|---|
| F1 | Cost of `engine:frame` on the simulation thread | **S:** `frame` is synchronous (`csrc/eech_engine.c:252-286`). The M4 baseline steps 100 ms (`crates/eech-world/lua/campaign.lua:17,143`). | **U: nothing measured in DCS.** No budget figure is proposed until the distribution is measured. | **N4** `-Stage frames` for at least 30 min of model time, then ×4 compression. The probe reports, per window, frame cost, callback cost through its last measured operation (`callback_max_ms`) beside its stepping part (`step_max_ms`), callback gaps, lag and `objects()` cost; definitions and terminal-callback limits are in [tools/dcs-probe](../tools/dcs-probe/README.md#reading-results) (`os.clock`, 1 ms resolution). The plan budget is set from the measured callback distribution, not the stepping part alone. Two measurement defects found by the lead before any DCS frames run (the full callback cost was not captured; the gap mean divided by the wrong count) are fixed and covered by the self-test. |
| F2 | Cadence | **S/R:** `timer.scheduleFunction`/`timer.getTime` exist in the mission state. `DCS.getRealTime` does not. | Callback cadence; pause and compression behaviour. | N4. |
| F3 | Thread affinity | **S:** `Engine` is `!Send`. The FP environment is saved and restored per entry (`csrc/eech_engine.c:47-68`). | Whether DCS calls one mission state from more than one thread. | Not observable from Lua without a product change; **I:** calls are serialised through one `lua_State`. |
| F4 | Boot stalls the mission start | **S:** synchronous terrain and campaign load (`csrc/eech_engine.c:77-219`). | Duration in DCS. | N3 `boot ms`. |

### Save and restore

| ID | Risk | Evidence | Open | Next probe and what it decides |
|---|---|---|---|---|
| P1 | DCS persistence hook | **R:** in the mission state `world.getPersistenceData` and `world.setPersistenceHandler` are functions. A fresh mission returns `nil`, and registering a handler succeeds (run 9). **S:** `Scripts/World/PersistenceHandlers.lua:1-18`; the GUI's `saveMissionState` calls `DCS.exportToMiz` (`Scripts/Hooks/webGUI.lua:472-481`). | When DCS calls `runPersistenceHandlers`; which value types survive; single player versus server. | **N5** (needs a small hook addition, not yet written): in a running `env` or `load` mission, call `DCS.exportToMiz (<path>)` from the profile hook, relaunch with `--mission <saved miz>`, and read `persistence_restore` (string, then `persistence_value="table"`). Decides whether a campaign blob can ride DCS's save, or needs its own file. |
| P2 | The campaign cannot serialise itself | **S:** no save/restore on the `eech_dc` surface (`crates/eech-dc/src/module.rs:57-67`). Legacy `store_session` is compiled in but not called headless (`../aphavoc/source/ui_menu/session/uisession.c:752`, `crates/eech-engine-sys/sources.txt:955`). | | Per Confluence 11534337, M5 only probes and bounds persistence. A live campaign save/restore implementation belongs to the migration plan unless an M5 criterion needs it. |
| P3 | Joint consistency of the campaign and the DCS world after restore | **S:** EECH ids are reused entity indices (`lib.rs:106-107`). | Identity mapping across a restore. | A host-contract question for the plan. Not probed. |

### Multiplayer and replay

| ID | Risk | Evidence | Open | Next probe and what it decides |
|---|---|---|---|---|
| M1 | One authoritative process | **R:** the mission state has `net` (`get_my_player_id ()=0`, `get_server_id ()=0` in single player) but no `DCS.isServer`. | Whether mission triggers run on clients; whether an unauthorised DCS can host. | **N6** (after N4; needs the L4 auth decision): a loopback local server plus one client, `env` stage on both, comparing their `probe.log` presence. The result decides the server-only enforcement rule. |
| M2 | Desanitised scripting on a server | **S:** this installation already exposes `io`/`os`/`require` to every mission (`MissionScripting.lua:15-22`, with the vendor warning at `:5-8`). | Whether updates restore the file. | Plan: an operator security decision; server-only deployment. |
| R1 | Track replay and determinism | **S:** the campaign advances only from host milliseconds (`csrc/eech_engine.c:283`). Wall-clock readers appear only in Tacview, the server log and UI (`../modules/system/time.c:261,301` callers). | Whether DCS replays re-run mission scripts with identical model time. | After N4: compare the `checkpoint` digests of a live run and its track replay. |

### Crash containment

| ID | Risk | Evidence | Open | Next probe |
|---|---|---|---|---|
| C1 | A fault in the module kills DCS | **R:** run 9 loaded the module, then DCS exited `0xC0000005` with no crash file in the profile. **I:** the next native error-path call caused that exit; the as-run probe had no begin marker. **S:** `debug_fatal` outside an entry calls `abort ()` (`csrc/eech_headless_system.c:137`). | Legacy NULL-dereference frequency in DCS. | N4 soak. In-process versus sidecar remains an explicit architectural decision; this register does not make it. |

## Decisions needed

1. **Authorise and own** the product Task for the DCS `lua_Debug` fix (L2). The size-only correction
   passed the actual product controls and the unchanged M4 pack as a disposable experiment; it
   still needs a reviewed product-source change. This register asks for no compiler change.
2. **Auth mode** for further DCS probes (L4): unauthorised is enough for N2/N3/N4 on free content;
   a one-time owner login is needed for N6 and any paid assets.
3. **Lifecycle contract** after N3 (B2): refuse or re-initialise.

## Smallest next step

N1 has passed outside DCS (L2). The next step is the reviewed product change, then N2 with it:

`pwsh -File tools/dcs-probe/run-dcs-probe.ps1 -Stage load -Candidate <corrected dir> -CancelLogin -TimeoutSeconds 240`

That is N2: the same mission-state probe as run 9, now with the `error_path_begin` marker. Pass:
`error_path raised=true`, and the run ends by the hook's `exitProcess` or the runner bound, not by
a crash exit code.
