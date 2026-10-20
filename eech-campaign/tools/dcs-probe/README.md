# DCS probe tooling (M5 criterion 5)

These tools probe the risks in [m5-dcs-risks.md](../../docs/m5-dcs-risks.md). What has
actually run, and its results, is in
[reference/m5-dcs-probes](../../reference/m5-dcs-probes/README.md). They are harness
tooling, not the DCS adapter: the probe drives `engine:frame` from model time and reads
engine state. It never feeds DCS state into the campaign and never commands units.

| File | Purpose |
|---|---|
| `pe-lua-abi.py` | Static, link-level check of the module's PE imports against a host `lua.dll`'s exports, plus PE32+, `luaopen_eech_dc`, delay-import and MinGW-runtime checks. **It cannot see struct layouts.** The current candidate passes it, yet crashes on DCS's `lua_Debug` size. |
| `eech-dcs-probe.lua` | The mission-state probe, in cumulative stages: `env`, `load` (`require` plus one native error with a flushed begin marker), `boot`, `frames`. It writes `EECH_DCS_PROBE event=…` records to `dcs.log` and `log_file`. |
| `build-probe-miz.py` | Builds a byte-reproducible probe mission from an installed DCS mission: units and module requirements stripped, and one MISSION START `DO SCRIPT` that sets `EECH_DCS_PROBE` and `dofile`s the probe. |
| `eech-m5probe-hook.lua` | A GUI hook for the isolated profile only. It logs simulation callbacks, releases the briefing pause, and calls `exitProcess` a bounded time after the simulation starts. It does not start missions: that did not work from a hook. |
| `run-dcs-probe.ps1` | Runs one bounded stage. It starts `bin-mt\DCS.exe -w DCS.M5Probe --no-launcher --mission <miz>` hidden and stops only its own process tree. It records inputs, hashes, dialogs, the verdict and logs in `<Work>\runs\<id>`. It refuses to run if any DCS is already running, and aborts if the default profile's log changes. |

## Preconditions

- This installation's `Scripts/MissionScripting.lua` has its sanitisation commented out (found
  so, not changed). If an update restores it, stage `env` reports `require=nil`. Re-editing the
  installation needs explicit approval.
- A new profile has no stored login. Either pass `-CancelLogin`, which clicks Cancel and then OK
  on the owned process's two login dialogs, so DCS runs unauthorised: paid modules are skipped,
  free content and Caucasus load. Or have the owner log in once to `DCS.M5Probe` with
  "Save password". Copying credential files from another profile does **not** work, and must not
  be done.
- `boot` and `frames` are refused unless `-ErrorControlsEvidence <file>` names a passing
  DCS-Lua error-control record for the same candidate (risk L2).

## Commands

```text
python -I tools/dcs-probe/pe-lua-abi.py <candidate>/eech_dc.dll "<DCS>/bin-mt/lua.dll"
pwsh -File tools/dcs-probe/run-dcs-probe.ps1 -Stage load -Candidate <dir> -CancelLogin -TimeoutSeconds 240
pwsh -File tools/dcs-probe/run-dcs-probe.ps1 -Stage boot -Candidate <dir> -CancelLogin -ErrorControlsEvidence <record> -Root <EECH root copy>
pwsh -File tools/dcs-probe/run-dcs-probe.ps1 -Stage frames -Candidate <dir> -CancelLogin -ErrorControlsEvidence <record> -Root <EECH root copy> -RunSeconds 1800 -TimeoutSeconds 2400
```

Use a **copy** of the EECH root (`E:\eech-runs\m2\lebanon-root`): boot `chdir`s into it, and
`prepare_installation` writes into it.

## Reading results

- The runner's verdict is the process outcome only: `exited with code N (0xXXXXXXXX)`, `timeout`
  (the owned tree is stopped), or `aborted`. What the probe reached comes from `probe.log`.
  A crash leaves the last flushed record.
- `frame_*_ms` is the cost of one `engine:frame (frame_ms)` (100 ms simulated, as in the M4
  baseline). `callback_*` includes sub-steps. `gap_*` is the wall interval between callbacks.
  `lag_max_ms > frame_ms` means the campaign fell behind model time; lag is carried, never dropped.
  `os.clock` has 1 ms resolution, so means over many frames are meaningful below it, and single
  values are not.
- An unauthorised single-player run is a different environment from an authenticated server.
  A crash-free run proves only that scenario for that duration.
