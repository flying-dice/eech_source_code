# DCS probe tooling (M5 criterion 5)

These tools probe the risks in [m5-dcs-risks.md](../../docs/m5-dcs-risks.md). What has
actually run, and its results, is in
[reference/m5-dcs-probes](../../reference/m5-dcs-probes/README.md). They are harness
tooling, not the DCS adapter: the probe drives `engine:frame` from model time and reads
engine state. It never feeds DCS state into the campaign and never commands units.

| File | Purpose |
|---|---|
| `pe-lua-link-check.py` | Static, link-level check of the module's PE imports against a host `lua.dll`'s exports, plus PE32+, `luaopen_eech_dc`, delay-import and MinGW-runtime checks. **It cannot see struct layouts.** The current candidate passes it, yet crashes on DCS's `lua_Debug` size. |
| `eech-dcs-probe.lua` | The mission-state probe, in cumulative stages: `env`, `load` (`require` plus one native error with a flushed begin marker), `boot`, `frames`. It writes `EECH_DCS_PROBE event=…` records to `dcs.log` and `log_file`. |
| `build-probe-miz.py` | Builds a byte-reproducible probe mission from an installed DCS mission: units and module requirements stripped, and one MISSION START `DO SCRIPT` that sets `EECH_DCS_PROBE` and `dofile`s the probe. |
| `eech-m5probe-hook.lua` | A GUI hook for the isolated profile only. It does nothing unless the runner set `EECH_M5PROBE_RUN_SECONDS`. It logs simulation callbacks, releases the briefing pause, and calls `exitProcess` a bounded time after the simulation starts. It does not start missions: that did not work from a hook. |
| `dcs-probe-selftest.py` | Tests without DCS: mission serialization and Lua 5.1 compilation of unusual paths; synthetic PE link and malformed cases; PowerShell prelaunch guards and fake process-tree cleanup through `runner-lifecycle-selftest.ps1`; and the probe and hook under fake DCS globals. It exercises multiple frame windows, scheduling returns, log open/write, require failures, frame errors, mission end, hook unpause/exit, and controlled callback-cost/gap statistics. CI runs it (`dcs-probe-tools` job). |
| `run-dcs-probe.ps1` | Runs one bounded stage. It starts `bin-mt\DCS.exe -w DCS.M5Probe --no-launcher --mission <miz>` hidden, and stops its owned process tree, including descendants if the root exits first (children must have started after their parent). It records inputs, hashes, dialogs, the verdict and logs in `<Work>\runs\<id>`. It refuses to run if any DCS is already running, if the canonical probe profile path equals the default profile (`-DefaultWriteDirName`, default `DCS.openbeta`) or if that profile's `dcs.log` is missing, and aborts if that log changes. |

## Preconditions

- This installation's `Scripts/MissionScripting.lua` has its sanitisation commented out (found
  so, not changed). If an update restores it, stage `env` reports `require=nil`. Re-editing the
  installation needs explicit approval.
- A new profile has no stored login. Either pass `-CancelLogin`, which clicks Cancel and then OK
  on the owned process's two login dialogs, so DCS runs unauthorised: paid modules are skipped,
  free content and Caucasus load. Or have the owner log in once to `DCS.M5Probe` with
  "Save password". Copying credential files from another profile does **not** work, and must not
  be done.
- `boot` and `frames` are refused unless `-ErrorControlsEvidence <file>` names a JSON record
  with matching SHA-256 hashes for the candidate `eech_dc.dll` and the installed
  `bin-mt/lua.dll`, plus completed zero-exit `first_error` and `repeated_errors` controls on
  that pair (risk L2). An operator must retain the actual control outputs beside this record;
  the guard validates the record's content and hashes, not the provenance of a claimed test.
  An arbitrary existing file, failed control, or mismatched DLL is rejected before profile
  writes. The record shape is:

  ```json
  {
    "candidate_eech_dc_sha256": "<64 hex digits>",
    "dcs_lua_sha256": "<64 hex digits>",
    "controls": {
      "first_error": { "completed": true, "exit_code": 0 },
      "repeated_errors": { "completed": true, "exit_code": 0 }
    }
  }
  ```

## Commands

```text
python -I tools/dcs-probe/dcs-probe-selftest.py <lua5.1 interpreter>
python -I reference/m5-dcs-probes/verify_retained.py
python -I tools/dcs-probe/pe-lua-link-check.py <candidate>/eech_dc.dll "<DCS>/bin-mt/lua.dll"
pwsh -File tools/dcs-probe/run-dcs-probe.ps1 -Stage load -Candidate <dir> -CancelLogin -TimeoutSeconds 240
pwsh -File tools/dcs-probe/run-dcs-probe.ps1 -Stage boot -Candidate <dir> -CancelLogin -ErrorControlsEvidence <record> [-Root <EECH root copy>] [-Scenario lebanon_retail]
pwsh -File tools/dcs-probe/run-dcs-probe.ps1 -Stage frames -Candidate <dir> -CancelLogin -ErrorControlsEvidence <record> [-Root <EECH root copy>] -RunSeconds 1800 -TimeoutSeconds 2400
```

`-Root` (default `E:\eech-runs\dcs-probe\lebanon-root`) must be a **copy** of a prepared EECH
root such as `E:\eech-runs\m2\lebanon-root`: boot `chdir`s into it, and `prepare_installation`
writes into it. `-Scenario` must match the root (`lebanon_retail` or `georgia_retail`).

## Reading results

- The runner's verdict is `exited with code N (0xXXXXXXXX)`, `timeout` (the owned tree is
  stopped), `aborted` (the default profile log changed), or `runner error: <message>` (the runner
  itself failed; the owned tree is still stopped). What the probe reached comes from `probe.log`.
  A crash leaves the last flushed record.
- `frame_*_ms` is the cost of one `engine:frame (frame_ms)` (100 ms simulated, as in the M4
  baseline). `step_max_ms` is one callback's campaign stepping (sub-steps and bookkeeping).
  `callback_max_ms` measures each completed scheduled callback through its last measured
  operation, over the window's `callbacks_completed`; a callback that writes a summary finishes
  after it, so its cost counts in the next window. If DCS exits without a later summary or final
  record, that last summary writer has no completed-cost report. A failing frame callback is included in
  `frames_final` through its `frame_error` write; the subsequent `callback_final cost_ms` includes
  that final summary write. A mission-end event writes `event_handler_cost cost_ms` after its final
  summary and checkpoint. These terminal records cannot include their own log-write cost.
  Budget against the measured callback cost with this limit. `gap_*` is the wall interval between
  consecutive callback starts, over the `gaps` sampled in the window. `-` means no sample for
  measured fields; counters and histogram buckets remain zero. Probe 7e699355 (the retained
  runs) wrote the stepping-only value as `callback_max_ms`.
  `lag_max_ms > frame_ms` means the campaign fell behind model time; lag is carried, never dropped.
  `os.clock` has 1 ms resolution, so means over many frames are meaningful below it, and single
  values are not.
- An unauthorised single-player run is a different environment from an authenticated server.
  A crash-free run proves only that scenario for that duration.
