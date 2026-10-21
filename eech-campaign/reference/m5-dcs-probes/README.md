# M5 criterion 5: DCS probe evidence (EEDCS-12)

This is the retained evidence for the DCS-only risk register,
[docs/m5-dcs-risks.md](../../docs/m5-dcs-risks.md). It covers:
- nine bounded DCS startup and mission attempts, all in stage `load`;
- the lead's standalone Lua-compatibility controls;
- a snapshot of the separate ABI microprobe results.

The authoritative goal is Confluence 11534337 v1, criterion 5. **This evidence does not
complete criterion 5:**
- No campaign boot ran in DCS.
- No DCS frame was measured.
- No save/restore was exercised.
- No multiplayer session was run.

## Identities

| Item | Identity |
|---|---|
| Candidate | `eech_dc.dll` sha256 `a99d7a1118008f73a995fd7e4701a04bddb5fa51d1bf3b5a487297d78c383f41`. Built from `65892de1` (`pr59-candidate` BUILD-INFO), unchanged by the evidence-only merge `163e9158`. |
| DCS | openbeta `2.9.29.27468` MT. Run manifests retain `bin-mt/DCS.exe` sha256 `6bb5832c…1c41` and `bin-mt/lua.dll` sha256 `8a228b54…b755`; no `bin/` comparison hashes are retained here. `Scripts/MissionScripting.lua` sha256 `f212df7f…0f6d`, sanitisation commented out before this work. |
| Profile | `-w DCS.M5Probe` (`%USERPROFILE%\Saved Games\DCS.M5Probe`), new and isolated. The manifests record `default_profile_log_unchanged=true` based on file modification ticks; they do not record before/after log hashes. The default profile's credential-file timestamps were checked locally but are not retained. |
| Mission | Built by `tools/dcs-probe/build-probe-miz.py` from the installed `CoreMods/WWII Units/FW-190D-9/Missions/fw190.miz` (sha256 `346cb7f7…45ba`). All units and module requirements are removed. It has one MISSION START `DO SCRIPT`. The `.miz` is vendor-derived and not committed. Its whole-file hash is in each manifest; individual entry hashes are not retained. |
| As-run scripts | `as-run/eech-dcs-probe.7e699355.lua` (all runs). `as-run/eech-m5probe-hook.14cdf01e.lua` (runs 1–7). `as-run/eech-m5probe-hook.ba5e2626.lua` (runs 8–9). Each is byte-identical to the hash its manifests record. `as-run/build-probe-miz.54eb324f.py` is the builder as committed at `54eb324f`. The current builder also writes `scenario` into the `DO SCRIPT`, so its `mission` entry differs. The runner's as-run versions were not retained byte-exact. The manifest command omits the as-run `-WorkingDirectory` argument; B1's cwd observation comes from run 9's probe log, not reconstruction of that string. |

## DCS runs (all on 2026-10-20/21, all hidden, all ended with no owned process left)

Every run used stage `load`, the candidate above and the isolated profile.
`runs/<id>/manifest.json` gives:
- the inputs and their hashes;
- the command and PID;
- the verdict and elapsed time;
- the dialogs seen;
- the stopped PIDs;
- whether the default log was unchanged.

`dcs.log` and `probe.log` are copies with the user path replaced by `%USERPROFILE%`.
`ORIGINALS.sha256` lists the hashes of the uncommitted original bytes. The retained copies have
their own `RETAINED.sha256`; `python -I verify_retained.py` checks all 57 from this repository.
To compare the originals too, supply `--original-runs`, `--original-controls`,
`--original-microprobe` pointing to their source directories, and
`--original-user-profile` with the original Windows profile path. The verifier strips a UTF-8 BOM,
replaces five spellings of that path (JSON-escaped backslashes, literal backslashes, forward
slashes, lowercase forward slashes, lowercase backslashes) with `%USERPROFILE%`, then
normalises CRLF to LF. Isolated CR bytes remain. This sequence also covers JSON-escaped paths
and the lowercase path spellings in `dcs.log`. Original hashes cannot be reconstructed from
the retained copies alone.

| # | Run | Setup | What happened | End |
|---|---|---|---|---|
| 1 | `20261020-232347-load` | Credentials: none | Log stops after CPU info. A hidden native login dialog, identified in run 3. | Killed at the 420 s bound |
| 2 | `20261020-233118-load` | `network.vault` copied from the default profile | Same stall | Killed, 422 s |
| 3 | `20261020-233835-load` | Same as run 2 | An ad hoc window enumeration (not retained) found a hidden `#32770 "DCS Login"` | Killed, 121 s |
| 4 | `20261020-234042-load` | `network.vault` and `authdata.bin` copied | Same stall: the copied files do not authenticate | Killed, 301 s |
| 5 | `20261020-234601-load` | Run 4's files, plus an ad hoc `BM_CLICK` on the dialog's Cancel | `login was cancelled` / `offline auth is not available`. Paid plugins are skipped and free content loads. Then a second stall (`Authorization cancelled`, found in run 6) | Killed, 241 s |
| 6 | `20261020-235038-load` | Copied files deleted. Runner `-CancelLogin` (Cancel only) | Dialogs recorded: `DCS Login`, then `Authorization cancelled \| OK \| Cancel` | Killed, 302 s |
| 7 | `20261020-235602-load` | `-CancelLogin` (Cancel, then OK) | Hook loaded (log line 504), `END OF INIT`, dispatcher state 3. The hook's `UpdateManager` updater never ran, so no mission was requested. | Killed, 302 s |
| 8 | `20261021-000137-load` | Run 7, plus an `onShowMainInterface` handler in the hook | Same: neither callback was called | Killed, 241 s |
| 9 | `20261021-000601-load` | Run 7, plus `--mission <miz>` on DCS's command line | `Dispatcher: loadMission` at `END OF INIT` (log 473–475). Mission state reached; `probe.log` records below. | **Process exited by itself `0xC0000005`**; the 2-second runner poll does not establish a precise interval from the last record |

The copied credential files were in the isolated profile for runs 2–5 only. They were
deleted before run 6 and were never retained or committed.

### Run 9: the mission scripting environment (actual DCS, unauthorised single player)

From `runs/20261021-000601-load/probe.log`. That probe version wrote whitespace in values as `_`.
- `lua=Lua 5.1`; `require`, `package`, `package.loadlib`, `io`, `os`, `lfs` are all present
  (because of this installation's desanitised `MissionScripting.lua`).
- `DCS=nil Sim=nil`: the GUI-state API is **absent** in the mission state. `net` is a table:
  `net.get_my_player_id()=0`, `net.get_server_id()=0`.
- `cwd=D:\Program Files\Eagle Dynamics\DCS World OpenBeta\`, the installation root, not `bin-mt`.
  `relative_scripts=visible`: `Scripts/ScriptingSystem.lua` opens by a relative path.
- The default `package.cpath` is
  `.\lua-?.dll;.\?.dll;<install>\bin-mt\lua-?.dll;<install>\bin-mt\?.dll;`.
  `lfs.writedir()` is the isolated profile.
- `world.getPersistenceData` and `world.setPersistenceHandler` are functions.
  `getPersistenceData("eech_dc_probe")` returns `nil` in a fresh mission, and registering the
  handler succeeds.
- MISSION START ran at `model_time=0`, `abs_time=43200`, about 10.5 s after `loadMission`.
- **`require ("eech_dc")` with `dll_dir` prepended to `package.cpath`: `ok=true`, `name=eech-dc 0.1.0`,
  1.0 ms.** The module loads into the mission state and binds to DCS's `lua.dll`. Its 83 imports
  resolve: `link-check.stdout.txt` retains the link-level PASS with both input hashes.
- The next statement is `pcall (dc.boot, "not a table")`, the first native error path. No
  `error_path` record follows. The process exits `0xC0000005` (exit code `-1073741819`).
  `dcs.log` ends with the probe's `load` record, and the profile's `Logs` holds no crash file.
  **Inferred, not proven in DCS:** the crash is inside that call. That probe version wrote no begin
  marker; the current probe flushes `error_path_begin` first. The inference rests on the
  identical standalone signature in the lead's controls and the microprobe below.
- No simulation start was reached, so the hook's run bound and unpause were not exercised.
  Campaign boot and frames were not attempted, and are now refused by the runner without
  error-control evidence.

### Why the GUI hook could not start the mission (runs 7–8)

- The hook loaded each time (`EECH_M5PROBE_HOOK … loaded`), and DCS reached `END OF INIT`.
- **Observed:** over at least 180 s idle, the hook's `UpdateManager` updater (run 7) and
  `onShowMainInterface` (run 8) were never called, and nothing requested a mission.
- **Shipped evidence:**
  - User hooks are loaded "into an isolated environment" and receive the callbacks set by
    `Sim.setUserCallbacks` (`API/Sim_ControlAPI.md:22-37`). The documented simulation and GUI
    callbacks (`:573-698`) include no main-menu or idle callback.
  - `onShowMainInterface` appears only in the GameGUI state's own list of simulator callbacks
    and as a global function there (`MissionEditor/GameGUI.lua:8, 57-73, 787-798`), not as a
    documented hook callback.
  - `UpdateManager.update` is attached to `Gui.AddUpdateCallback` in `Scripts/UserHooks.lua:15-19`.
    Whether those updates run in the hooks' state at the main menu (`mmw.show (true)`,
    `MissionEditor/MissionEditor.lua:638-643`) with a hidden window was not established.
- **What works:** the GUI starts a mission by setting `START_PARAMS.missionPath` and
  `START_PARAMS.command = '--mission'`, then calling `Gui.doQuit ()`
  (`MissionEditor/modules/me_mission.lua:8464-8491`, `me_menubar.lua:328`, `me_toolbar.lua:955`).
  The same `--mission <miz>` on the command line is accepted (run 9). The runner now always uses
  it, and the hook no longer tries.

## Lead controls (Starscream, Jira EEDCS-12 comment 10439; `lead-controls/`)

These are standalone runs of `eech-world.exe` (`13cc2a6f…a228`) with the candidate module, outside
DCS. Scripts, stdout/stderr and manifests are retained; the DLLs and EXEs are not.
- **Candidate PUC Lua `d0c799fe…` (`candidate-lua-control/`):** `lua-compat.lua` passes 100
  invalid `dc.boot` calls (`bad argument #1: error converting Lua string to table`). Plain Lua
  errors and host-callback errors pass. Every control exits 0.
- **DCS `lua.dll` `8a228b54…`, no module (`dcs-lua-empty-control/`):** an empty script and
  `pcall (error, …)` pass. `pcall (host.log, {})`, a native **host** callback error, crashes
  `0xc0000005 at 0x0` before returning.
- **DCS `lua.dll` plus module (top level):**
  - `load-only.lua`: require, then GC, exit 0.
  - `first-error.lua`: prints `BEFORE_BAD_ARGUMENT`, then crashes `0xc0000005 at 0x0`.
  - `lua-compat.lua`: crashes.

## ABI microprobe (separate Claude microprobe session for the lead; Jira EEDCS-12 comment 10440)

`microprobe/RESULTS.md` and `microprobe/SHA256SUMS.txt` are snapshots, taken on 2026-10-21, of
`E:/eech-runs/dcs-probe/abi-microprobe/`. That tree belongs to another worker and may have changed
since. Summary by attribution:
- DCS `lua.dll` uses `LUA_IDSIZE = 256`, so its `lua_Debug` is 320 bytes. mlua-sys 0.6.8
  hard-codes 60, so its `lua_Debug` is 120 bytes (`src/lua51/lua.rs:355`).
- A disassembly and a canary agree: DCS writes 316 bytes into it. That is a 196-byte overrun
  during mlua's traceback on a callback error.
- Stock GNU and MSVC tiny modules both fail on DCS Lua. Changing only IDSIZE to 256 passes 60/60
  per compiler, per Lua runtime and in two hosts.
- **A compiler port is not a remedy.** The actual product was then rebuilt
  with the change: see "Product ABI experiment" below.

## Product ABI experiment (lead-verified; Jira EEDCS-12 comment 10446; `product-abi/`)

These files are byte-for-byte copies of the text artefacts in
`E:/eech-runs/dcs-probe/abi-microprobe/product/`. Their hashes are in `product-abi/SHA256SUMS.txt`.
Built from an archive of `65892de1` with only `LUA_IDSIZE` 60→256 in mlua-sys 0.6.8
(`mlua-sys-idsize256.diff`, `cargo-toml.diff`, `cargo-lock.diff`; no compiler change):
- **Controls** (`controls-{stock,patched}.summary.txt`): patched, all 36 runs exit 0 on DCS and PUC
  Lua. Stock exits `0xc0000005` in `first-error`, `host-log` and `lua-compat` on DCS Lua only.
- **M4 pack** (`pack-patched.report.{json,md}`): unchanged pack, all 40 checks PASS. Georgia and
  Lebanon metrics are IDENTICAL to their baselines.
- **Not shown:** `pack_revision` is blank (a disposable archive plus patch, not a product commit),
  and the corrected build has not run in a DCS mission.

## Reproduce

- Mission:
  `python -I tools/dcs-probe/build-probe-miz.py "<DCS>/CoreMods/WWII Units/FW-190D-9/Missions/fw190.miz" <out.miz> load lebanon_retail <probe.lua> <dll_dir> <root> <log_file>` (run 9's entries: the as-run builder, without `lebanon_retail`; command shown from `eech-campaign/`)
- Run 9's launch (the runner now always does this):
  `pwsh -File tools/dcs-probe/run-dcs-probe.ps1 -Stage load -Candidate <dir> -CancelLogin -TimeoutSeconds 240`
  This starts `bin-mt\DCS.exe -w DCS.M5Probe --no-launcher --mission "<miz>"` hidden, and stops
  only its own process tree.
- `-CancelLogin` automates Cancel/OK on the owned process's login dialogs. DCS then runs
  unauthorised (paid modules skipped), and that is a different environment from an authenticated
  session.
