-- GUI hook for the isolated probe profile only (Saved Games\DCS.M5Probe\Scripts\Hooks).
-- It never touches the mission state or the campaign. It starts the probe mission
-- from the main menu (net.load_mission, from onShowMainInterface or a GUI update), dismisses the single-player briefing pause,
-- logs the simulation callbacks, and ends its own DCS process a bounded time later.
--
-- Environment (set by run-dcs-probe.ps1): EECH_M5PROBE_MISSION (absolute .miz),
-- EECH_M5PROBE_RUN_SECONDS (real seconds after simulation start, default 20),
-- EECH_M5PROBE_MENU_TIMEOUT (real seconds without a simulation start, default 240).

local mission = os.getenv ("EECH_M5PROBE_MISSION")
local run_seconds = tonumber (os.getenv ("EECH_M5PROBE_RUN_SECONDS") or "20")
local menu_timeout = tonumber (os.getenv ("EECH_M5PROBE_MENU_TIMEOUT") or "240")

local function say (fmt, ...)
	log.write ("EECH_M5PROBE_HOOK", log.INFO, string.format (fmt, ...))
end

local started_real, exiting, requested
local callbacks = {}

local function request (how)
	if requested or not mission then return end
	requested = true
	local loaded, result = pcall (net.load_mission, mission)
	say ("net.load_mission via %s ok=%s result=%s", how, tostring (loaded), tostring (result))
end

local function exit (why)
	if exiting then return end
	exiting = true
	say ("exitProcess: %s", why)
	DCS.exitProcess ()
end

-- GameGUI.lua lists onShowMainInterface among the callbacks the simulator calls
function callbacks.onShowMainInterface () say ("onShowMainInterface") request ("onShowMainInterface") end
function callbacks.onShowBriefing () say ("onShowBriefing") end
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
	if not started_real then return end
	local elapsed = DCS.getRealTime () - started_real
	-- the single-player briefing holds the simulation paused; release it during the first seconds only
	if elapsed < 10 and DCS.getPause () then
		say ("setPause(false) at model_time=%.3f", DCS.getModelTime ())
		DCS.setPause (false)
	end
	if elapsed > run_seconds then
		exit (string.format ("run bound %.0f s reached, model_time=%.3f", run_seconds, DCS.getModelTime ()))
	end
end

DCS.setUserCallbacks (callbacks)
say ("loaded: mission=%s run_seconds=%s menu_timeout=%s writedir=%s", tostring (mission), tostring (run_seconds), tostring (menu_timeout), lfs.writedir ())

if mission then
	local ok, updates = pcall (require, "UpdateManager")
	if not ok then
		say ("UpdateManager unavailable: %s", tostring (updates))
		return
	end
	local t0 = os.clock ()
	updates.add (function ()
		local t = os.clock () - t0
		if t > 3 then request ("UpdateManager") end
		if not started_real and t > menu_timeout then
			exit ("no simulation start within the menu timeout")
			return true
		end
		return started_real ~= nil
	end)
end
