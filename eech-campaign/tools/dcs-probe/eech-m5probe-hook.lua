-- GUI hook for the isolated probe profile only (Saved Games\DCS.M5Probe\Scripts\Hooks).
-- It never touches the mission state or the campaign. It logs the simulation
-- callbacks, releases the single-player briefing pause, and ends its own DCS
-- process a bounded time after the simulation starts.
--
-- The probe mission is started by DCS's own command line (run-dcs-probe.ps1 passes
-- --mission <miz>; dcs.log then shows "Dispatcher: loadMission" right after END OF
-- INIT). Starting it from this hook did not work in the observed runs: neither an
-- UpdateManager updater nor onShowMainInterface was called at the main menu
-- (reference/m5-dcs-probes/README.md). The runner's process timeout is the outer
-- bound; this hook's bound applies only once the simulation has started.
--
-- Environment (set by run-dcs-probe.ps1): EECH_M5PROBE_RUN_SECONDS (real seconds
-- after simulation start, default 20).

local run_seconds = tonumber (os.getenv ("EECH_M5PROBE_RUN_SECONDS") or "20")

local function say (fmt, ...)
	log.write ("EECH_M5PROBE_HOOK", log.INFO, string.format (fmt, ...))
end

local started_real, exiting
local callbacks = {}

function callbacks.onMissionLoadBegin () say ("onMissionLoadBegin") end
function callbacks.onMissionLoadEnd () say ("onMissionLoadEnd model_time=%.3f", DCS.getModelTime ()) end
function callbacks.onSimulationStart ()
	started_real = DCS.getRealTime ()
	say ("onSimulationStart model_time=%.3f paused=%s mission=%s", DCS.getModelTime (), tostring (DCS.getPause ()), tostring (DCS.getMissionName ()))
end
function callbacks.onSimulationPause () say ("onSimulationPause model_time=%.3f", DCS.getModelTime ()) end
function callbacks.onSimulationResume () say ("onSimulationResume model_time=%.3f", DCS.getModelTime ()) end
function callbacks.onSimulationStop () say ("onSimulationStop model_time=%.3f", DCS.getModelTime ()) end
function callbacks.onSimulationFrame ()
	if not started_real or exiting then return end
	local elapsed = DCS.getRealTime () - started_real
	-- the single-player briefing holds the simulation paused; release it during the first seconds only
	if elapsed < 10 and DCS.getPause () then
		say ("setPause(false) at model_time=%.3f", DCS.getModelTime ())
		DCS.setPause (false)
	end
	if elapsed > run_seconds then
		exiting = true
		say ("exitProcess: run bound %.0f s reached, model_time=%.3f", run_seconds, DCS.getModelTime ())
		DCS.exitProcess ()
	end
end

DCS.setUserCallbacks (callbacks)
say ("loaded: run_seconds=%s writedir=%s", tostring (run_seconds), lfs.writedir ())
