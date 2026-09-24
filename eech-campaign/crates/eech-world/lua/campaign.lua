-- A dynamic campaign run, driven from Lua: boots EECH's campaign in the
-- eech_dc module, runs it frame by frame, reports progress and hands the
-- observed world to the host for its Tacview recording.
--
-- host.args: root (installation root), scenario (georgia, georgia_retail), map, campaign, hours, frame_ms,
--            record_every (frames), acmi (output path), seed

local args = host.args
local root = assert (args.root, "root=<installation root>")
local hours = tonumber (args.hours or "1")
local frame_ms = tonumber (args.frame_ms or "100")
local record_every = tonumber (args.record_every or "10")
-- simulated seconds between force-state diagnostics (0: none)
local diagnostics_every = tonumber (args.diagnostics_every or "0")

-- the maps eech-map builds: game path, campaign file, map origin, title
local scenarios = {
	-- the retail Georgia campaign ("Caspian Black Gold") on retail map3 data
	-- map3 is not metric: EECH's own Tacview origin (textuser.c: 41.16, 40.185) is
	-- off by up to 50 km. The projection is fitted by correlating the retail
	-- terrain heights with SRTM (r = 0.978): x is stretched 1.22 east-west.
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC", title = "EECH dynamic campaign: Caspian Black Gold (Georgia)",
		affine = { m = { 1.2167375, -0.02315, -0.0035625, 0.9910375 }, t = { 258100.5, 94261.0 }, latitude = 42.0, longitude = 43.0 } },
	georgia = { map = "..\\common\\maps\\map16", campaign = "georgia.chc", latitude = 41.0, longitude = 40.4, title = "EECH dynamic campaign: Georgia" },
}
local scenario = assert (scenarios[args.scenario or "georgia"], "scenario=georgia|georgia_retail")

local dc = require ("eech_dc")
host.log (dc.name .. " loaded")

dc.prepare_installation (root)

local engine = dc.boot {
	install_root = root,
	map = args.map or scenario.map,
	campaign_directory = scenario.campaign_directory or "camp01",
	campaign = args.campaign or scenario.campaign,
	gunship = "apache",
	seed = tonumber (args.seed or "1"),
}
host.log ("campaign booted")

host.open_recording {
	path = args.acmi or "campaign.acmi",
	title = args.title or scenario.title,
	reference_time = args.reference_time or "2026-09-24T06:00:00Z",
	affine = scenario.affine,
	latitude = tonumber (args.latitude) or scenario.latitude,
	longitude = tonumber (args.longitude) or scenario.longitude,
}

-- campaign state summary: live units per side and kind, and keysites held
local function summary (objects)
	local count = {}
	local keysites = { blue = 0, red = 0 }
	local weapons, destroyed = 0, { blue = 0, red = 0 }
	for _, o in ipairs (objects) do
		if o.kind == "keysite" then
			keysites[o.side] = (keysites[o.side] or 0) + 1
		elseif o.kind == "weapon" then
			weapons = weapons + 1
		elseif o.alive then
			local key = o.side .. " " .. o.kind
			count[key] = (count[key] or 0) + 1
		else
			destroyed[o.side] = (destroyed[o.side] or 0) + 1
		end
	end
	local parts = {}
	for k, v in pairs (count) do parts[#parts + 1] = k .. "=" .. v end
	table.sort (parts)
	-- the tasks groups are on, per side (each group counted once)
	local tasks, seen = {}, {}
	for _, o in ipairs (objects) do
		if o.task and o.group_id and not seen[o.group_id] and o.task ~= "TASK_NOTHING" and (o.kind == "helicopter" or o.kind == "fixed_wing") then
			seen[o.group_id] = true
			local key = o.side .. " " .. o.task:gsub ("^TASK_", "") .. "/" .. (o.state or "?"):gsub (" ", "_")
			tasks[key] = (tasks[key] or 0) + 1
		end
	end
	local task_parts = {}
	for k, v in pairs (tasks) do task_parts[#task_parts + 1] = k .. "=" .. v end
	table.sort (task_parts)
	return string.format ("keysites blue=%d red=%d; wrecks blue=%d red=%d; weapons in flight %d; %s; tasks: %s",
		keysites.blue, keysites.red, destroyed.blue, destroyed.red, weapons, table.concat (parts, " "), table.concat (task_parts, " "))
end

local frames = math.floor (hours * 3600 * 1000 / frame_ms)
local last_owner = {}
for frame = 1, frames do
	engine:frame (frame_ms)
	if frame % record_every == 0 then
		local clock = engine:clock ()
		local objects = engine:objects ()
		host.record (clock.elapsed_seconds, objects)
		-- keysites changing hands
		for _, o in ipairs (objects) do
			if o.kind == "keysite" then
				if last_owner[o.name] and last_owner[o.name] ~= o.side then
					host.event (clock.elapsed_seconds, "Message", o.name .. " captured by " .. o.side)
					host.log (string.format ("%.0f s: %s captured by %s", clock.elapsed_seconds, o.name, o.side))
				end
				last_owner[o.name] = o.side
			end
		end
		if diagnostics_every > 0 and frame % math.floor (diagnostics_every * 1000 / frame_ms) == 0 then
			engine:diagnostics ()
		end
		if frame % (record_every * 600) == 0 then
			host.log (string.format ("%.0f s (day %d, %.0f s of day): %s", clock.elapsed_seconds, clock.day, clock.time_of_day_seconds, summary (objects)))
		end
	end
end
host.log ("campaign run complete")
