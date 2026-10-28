# Final D2 native frame-cost evidence: scope and DCS follow-up

This note defines interpretation of the final 900-second measurement. The run
and its acceptance proof must supply actual completion, recomputed raw statistics,
complete artifact comparisons and post-run identities; this note supplies no
timing result by itself.

The measurement runs the ordinary shipping64cf DLL in EECH_WORLD. An observational
copy of its actual shipping campaign script times the sole `engine:frame` call,
including nested host callbacks. Recording, object enumeration, metrics, outer
Lua work and per-frame sample serialization are outside the timed span. That
serialization can affect subsequent load, so the result describes this instrumented
run. The four full-pack campaigns execute concurrently. Local targeted Rust tests
also ran during the probe; their timestamped logs and process records retain that
context. The Task45-parent measurement used a different source and workload;
differences between those measurements are not causal performance attribution.

All 9,000 raw samples must be retained and independently recomputed, including
the 15 complete 600-frame windows and nearest-rank quantiles. The Windows Lua DLL
is identified by its actual SHA256. Its retained idle calibration applies only
because those bytes match: `os.clock` advances during idle and is treated as
elapsed time with timer quantization, rather than CPU-only work. The 100ms
simulated frame step does not establish a permitted wall-time budget.

The required control comparison is all bytes of Tacview, observations and metrics
against retained `ab-base`, with the original baseline-root provenance still
declared inferred. Current inputs are separately verified against the repository
retail manifest. Equal recordings establish observational transparency for this
control; they are separate from the full40-check pack and its eight artifacts.

The unanswered DCS question is the elapsed cost and thread affinity of the
complete mission callback using the intended DCS host, including object reads,
report delivery and scheduling. This EECH_WORLD run does not execute that path.
In the supported DCS version, load the same accepted DLL through the mission-Lua
entry, exercise the representative force/observation workload, and record the
callback's total elapsed time separately from the DLL frame span and host API
calls. Record mission/server/client configuration, callback thread, scheduling
interval, overruns, errors and campaign outcomes. Verify the permitted thread
for each DCS API and the DLL's reentrancy contract before evaluating a worker
arrangement. Compare results against an explicitly agreed scheduling budget;
recording a simulated step of100ms is not approval of such a budget. Live DCS
operation and its acceptance remain M6 work under the authoritative M5/M6 plan.
