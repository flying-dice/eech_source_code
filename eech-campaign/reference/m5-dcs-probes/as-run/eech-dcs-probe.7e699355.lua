-- EECH DCS probe: loading, lifecycle and frame cost of eech_dc.dll in the DCS
-- mission scripting environment. Regression-harness tooling, not the adapter:
-- it drives engine:frame from DCS model time and reads the engine's own state.
-- It never passes DCS state into the campaign and never commands DCS units, so
-- every campaign decision in a run is the campaign's.
--
-- Load it from a mission trigger (DO SCRIPT), configuration first:
--
--   EECH_DCS_PROBE = { stage = "frames", dll_dir = [[E:\eech-runs\dcs-probe\bin]],
--                      root = [[E:\eech-runs\dcs-probe\lebanon-root]] }
--   dofile ([[E:\eech-runs\dcs-probe\eech-dcs-probe.lua]])
--
-- stage:   env     the environment only (no require)
--          load    + require ("eech_dc") and one Lua error raised inside the module
--          boot    + prepare_installation and boot (campaign.lua's Lebanon configuration)
--          frames  + engine:frame from timer.scheduleFunction, with cost statistics
-- Other fields (defaults): scenario "lebanon_retail", seed 1, frame_ms 100,
--          interval 0.05 (model s between callbacks), max_substeps 10,
--          summary_every 60 (model s), prepare true, log_file nil (absolute path,
--          needs io), persistence_value "string" | "table".
--
-- Output: dcs.log lines "EECH_DCS_PROBE event=<name> key=value ...", whitespace in
-- values replaced by "_". Costs are os.clock differences (milliseconds).

local config = EECH_DCS_PROBE or {}
local stages = { env = 1, load = 2, boot = 3, frames = 4 }
local stage = stages[config.stage or "env"]
local scenarios = {
	-- as crates/eech-world/lua/campaign.lua and lifecycle.lua boot them
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}

local log_file = config.log_file and io and io.open (config.log_file, "a")
local clock = os and os.clock

local function record (event, fields)
	local keys = {}
	for k in pairs (fields or {}) do keys[#keys + 1] = k end
	table.sort (keys)
	local parts = { "EECH_DCS_PROBE", "event=" .. event }
	for _, k in ipairs (keys) do
		parts[#parts + 1] = k .. "=" .. (tostring (fields[k]):gsub ("%s", "_"))
	end
	local line = table.concat (parts, " ")
	env.info (line)
	if log_file then
		log_file:write (line, "\n")
		log_file:flush ()
	end
end

local function ms_since (t0)
	return clock and t0 and string.format ("%.1f", (clock () - t0) * 1000) or "unavailable"
end

local function first_line (message)
	return (tostring (message):match ("^[^\n]*"))
end

local function cwd ()
	return lfs and lfs.currentdir and lfs.currentdir () or "unavailable"
end

-- the relative path MissionScripting.lua loads its next mission's environment from
local function relative_scripts ()
	if not io then return "no_io" end
	local f = io.open ("Scripts/ScriptingSystem.lua", "r")
	if f then f:close () return "visible" end
	return "not_visible"
end

if not stage then
	record ("config_error", { stage = config.stage, expected = "env|load|boot|frames" })
	return
end

-- stage env ---------------------------------------------------------------------------------------

local function call (f, ...)
	if type (f) ~= "function" then return "absent" end
	local ok, v = pcall (f, ...)
	return ok and tostring (v) or "error:" .. first_line (v)
end

record ("env", {
	stage = config.stage or "env",
	lua = _VERSION,
	require = type (require), package = type (package), io = type (io), os = type (os), lfs = type (lfs),
	loadlib = type (package and package.loadlib), clock = type (clock),
	net = type (net), DCS = type (DCS), Sim = type (Sim),
	my_player_id = call (net and net.get_my_player_id), server_id = call (net and net.get_server_id),
	set_persistence = type (world and world.setPersistenceHandler),
	get_persistence = type (world and world.getPersistenceData),
	model_time = timer.getTime (), abs_time = call (timer.getAbsTime),
	cwd = cwd (), relative_scripts = relative_scripts (),
	writedir = call (lfs and lfs.writedir),
	cpath = package and package.cpath,
	loaded_before = package and package.loaded and type (package.loaded.eech_dc),
})

-- persistence: what a restored mission hands back, and when DCS asks for state
local frames_done = 0
if world and world.getPersistenceData then
	local ok, v = pcall (world.getPersistenceData, "eech_dc_probe")
	record ("persistence_restore", { ok = ok, type = type (v),
		value = type (v) == "table" and tostring (v.marker) .. "@" .. tostring (v.model_time) or first_line (v) })
end
if world and world.setPersistenceHandler then
	local ok, err = pcall (world.setPersistenceHandler, "eech_dc_probe", function ()
		local now = timer.getTime ()
		record ("persistence_save", { model_time = now, frames = frames_done })
		if config.persistence_value == "table" then
			return { marker = "eech_dc_probe", model_time = now, frames = frames_done }
		end
		return string.format ("eech_dc_probe@%.3f frames=%d", now, frames_done)
	end)
	record ("persistence_handler", { ok = ok, error = not ok and first_line (err) or nil })
end

if stage < stages.load then return end

-- stage load --------------------------------------------------------------------------------------

if not require or not package then
	record ("load", { ok = false, error = "require_or_package_absent" })
	return
end
if config.dll_dir then
	package.cpath = config.dll_dir .. "\\?.dll;" .. package.cpath
end
local t0 = clock and clock ()
local loaded, dc = pcall (require, "eech_dc")
record ("load", { ok = loaded, ms = ms_since (t0), name = loaded and dc.name or nil,
	error = not loaded and first_line (dc) or nil, cwd = cwd () })
if not loaded then return end

-- a Lua error raised from inside the module: mlua -> lua_error -> the host Lua's unwinding.
-- The argument is rejected before Engine::boot, so the one boot per process is not used.
local raised, message = pcall (dc.boot, "not a table")
record ("error_path", { raised = not raised, message = first_line (message) })

if stage < stages.boot then return end

-- stage boot --------------------------------------------------------------------------------------

local scenario = scenarios[config.scenario or "lebanon_retail"]
if not scenario or not config.root then
	record ("config_error", { scenario = config.scenario, root = config.root, expected = "root=<installation root>, scenario=lebanon_retail|georgia_retail" })
	return
end
local cwd_before = cwd ()
if config.prepare ~= false then
	t0 = clock and clock ()
	local ok, err = pcall (dc.prepare_installation, config.root)
	record ("prepare", { ok = ok, ms = ms_since (t0), error = not ok and first_line (err) or nil })
end
t0 = clock and clock ()
local booted, engine = pcall (dc.boot, {
	install_root = config.root, map = scenario.map, campaign_directory = "camp01",
	campaign = scenario.campaign, gunship = "apache", seed = config.seed or 1,
})
record ("boot", { ok = booted, ms = ms_since (t0), error = not booted and first_line (engine) or nil,
	model_time = timer.getTime (), cwd_before = cwd_before, cwd_after = cwd (), relative_scripts = relative_scripts () })
if not booted then return end

if stage < stages.frames then return end

-- stage frames ------------------------------------------------------------------------------------

local frame_ms = config.frame_ms or 100
local interval = config.interval or 0.05
local max_substeps = config.max_substeps or 10
local summary_every = config.summary_every or 60
local bounds = { 1, 2, 4, 8, 16, 33, 66 }

local stats
local function reset ()
	stats = { callbacks = 0, frames = 0, frame_total = 0, frame_max = 0, callback_max = 0,
		gap_total = 0, gap_max = 0, lag_max = 0, buckets = {} }
	for i = 1, #bounds + 1 do stats.buckets[i] = 0 end
end
reset ()

local pending_ms = 0
local last_model = timer.getTime ()
local last_wall
local next_summary = last_model + summary_every
local stopped = false

local function histogram ()
	local parts = {}
	for i, b in ipairs (bounds) do parts[#parts + 1] = "le" .. b .. ":" .. stats.buckets[i] end
	parts[#parts + 1] = "gt" .. bounds[#bounds] .. ":" .. stats.buckets[#bounds + 1]
	return table.concat (parts, ",")
end

-- a digest of the campaign's observable state, for comparing a live run with its track replay
local function checkpoint (now)
	local t = clock and clock ()
	local ok, objects = pcall (engine.objects, engine)
	local objects_ms = ms_since (t)
	if not ok then
		record ("checkpoint_error", { model_time = now, message = first_line (objects) })
		return
	end
	local alive, kinds, sx, sz = 0, {}, 0, 0
	for _, o in ipairs (objects) do
		if o.alive then
			alive = alive + 1
			kinds[o.kind] = (kinds[o.kind] or 0) + 1
			sx, sz = sx + o.position[1], sz + o.position[3]
		end
	end
	local names = {}
	for k, n in pairs (kinds) do names[#names + 1] = k .. ":" .. n end
	table.sort (names)
	local clocked, c = pcall (engine.clock, engine)
	if not clocked then c = {} end
	record ("checkpoint", { model_time = string.format ("%.3f", now), frames = frames_done,
		campaign_elapsed = c.elapsed_seconds and string.format ("%.1f", c.elapsed_seconds) or "error", day = c.day,
		objects = #objects, alive = alive, kinds = table.concat (names, ","),
		position_digest = string.format ("%.0f/%.0f", sx, sz), objects_ms = objects_ms })
end

local function summary (event, now)
	record (event, { model_time = string.format ("%.3f", now), callbacks = stats.callbacks, frames = stats.frames,
		frames_total = frames_done, frame_mean_ms = stats.frames > 0 and string.format ("%.3f", stats.frame_total / stats.frames) or "-",
		frame_max_ms = string.format ("%.1f", stats.frame_max), callback_max_ms = string.format ("%.1f", stats.callback_max),
		gap_mean_ms = stats.callbacks > 1 and string.format ("%.1f", stats.gap_total / (stats.callbacks - 1)) or "-",
		gap_max_ms = string.format ("%.1f", stats.gap_max), lag_max_ms = string.format ("%.0f", stats.lag_max),
		histogram = histogram () })
end

local function step (_, _)
	if stopped then return nil end
	local now = timer.getTime ()
	local wall = clock and clock ()
	if wall and last_wall then
		local gap = (wall - last_wall) * 1000
		stats.gap_total = stats.gap_total + gap
		stats.gap_max = math.max (stats.gap_max, gap)
	end
	last_wall = wall
	-- sub-steps follow model time only; time not yet run carries over (lag), it is never dropped
	pending_ms = pending_ms + (now - last_model) * 1000
	last_model = now
	local n = 0
	while pending_ms >= frame_ms and n < max_substeps do
		local f0 = clock and clock ()
		local ok, err = pcall (engine.frame, engine, frame_ms)
		if not ok then
			stopped = true
			record ("frame_error", { model_time = now, frames = frames_done, message = first_line (err) })
			summary ("frames_final", now)
			return nil
		end
		if f0 then
			local cost = (clock () - f0) * 1000
			stats.frame_total = stats.frame_total + cost
			stats.frame_max = math.max (stats.frame_max, cost)
			local i = 1
			while bounds[i] and cost > bounds[i] do i = i + 1 end
			stats.buckets[i] = stats.buckets[i] + 1
		end
		pending_ms = pending_ms - frame_ms
		frames_done = frames_done + 1
		stats.frames = stats.frames + 1
		n = n + 1
	end
	stats.lag_max = math.max (stats.lag_max, pending_ms)
	stats.callbacks = stats.callbacks + 1
	if wall then stats.callback_max = math.max (stats.callback_max, (clock () - wall) * 1000) end
	if now >= next_summary then
		summary ("frames_summary", now)
		checkpoint (now)
		reset ()
		next_summary = now + summary_every
	end
	return now + interval
end

world.addEventHandler ({ onEvent = function (_, event)
	if world.event and event.id == world.event.S_EVENT_MISSION_END and not stopped then
		stopped = true
		local now = timer.getTime ()
		summary ("frames_final", now)
		checkpoint (now)
	end
end })

record ("frames_start", { frame_ms = frame_ms, interval = interval, max_substeps = max_substeps, summary_every = summary_every })
checkpoint (last_model)
timer.scheduleFunction (step, nil, last_model + interval)
