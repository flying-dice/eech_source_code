-- A dynamic campaign run, driven from Lua: boots EECH's campaign in the
-- eech_dc module, runs it frame by frame, reports progress and hands the
-- observed world to the host for its Tacview recording.
--
-- host.args: root (installation root), scenario (georgia, georgia_retail, lebanon_retail), map, campaign, hours, frame_ms,
--            record_every (frames), acmi (output path), seed, diagnostics_every and log_every (simulated
--            seconds), stop (hours: run all of them; conclusion: also stop when the war is decided or
--            stalls), stalemate_hours, metrics (a JSON file of campaign metrics, metrics.lua),
--            checkpoint_every (simulated seconds between metrics snapshots), record (0: no recording)

local args = host.args
local root = assert (args.root, "root=<installation root>")
local hours = tonumber (args.hours or "1")
local frame_ms = tonumber (args.frame_ms or "100")
local record_every = tonumber (args.record_every or "10")
-- simulated seconds between force-state diagnostics (0: none)
local diagnostics_every = tonumber (args.diagnostics_every or "0")
-- simulated seconds between progress lines
local log_every = tonumber (args.log_every or "600")
-- stop=conclusion ends the run early (hours stays the cap) when a side holds no
-- airbase or FARP left, or when nothing is captured or destroyed for
-- stalemate_hours (longer than the slowest regen interval, so reinforcements
-- still get their turn)
local stop_at_conclusion = args.stop == "conclusion"
local stalemate_seconds = tonumber (args.stalemate_hours or "48") * 3600
local checkpoint_every = tonumber (args.checkpoint_every or "3600")
local recording = args.record ~= "0"

-- metrics.lua, beside this script
local here = debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or ""
local metrics = dofile (here .. "metrics.lua").new ()

-- the maps eech-map builds: game path, campaign file, map origin, title
local scenarios = {
	-- the retail Georgia campaign ("Caspian Black Gold") on retail map3 data
	-- map3 is not metric: EECH's own Tacview origin (textuser.c: 41.16, 40.185) is
	-- off by up to 50 km. The projection is fitted by correlating the retail
	-- terrain heights with SRTM (r = 0.978): x is stretched 1.22 east-west.
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC", title = "EECH dynamic campaign: Caspian Black Gold (Georgia)",
		affine = { m = { 1.2167375, -0.02315, -0.0035625, 0.9910375 }, t = { 258100.5, 94261.0 }, latitude = 42.0, longitude = 43.0 } },
	-- the retail Lebanon campaign (map5). map5 is not metric either: EECH's Tacview
	-- origin (textuser.c: 32.86, 35.01) puts airbases up to 17 km out. The
	-- projection is fitted to eight real airbases (Beirut, Damascus, Mezzeh, Riyaq,
	-- Marj Ruhayyil, Khalkhalah, Kleiat, Rosh Pina; median residual 1.1 km): z
	-- is compressed 0.84 north-south.
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC", title = "EECH dynamic campaign: Lebanon",
		affine = { m = { 0.9769436, 0.0474611, -0.0187441, 0.8438513 }, t = { 89434.7, 78512.0 }, latitude = 33.5, longitude = 36.0 } },
	georgia = { map = "..\\common\\maps\\map16", campaign = "georgia.chc", latitude = 41.0, longitude = 40.4, title = "EECH dynamic campaign: Georgia" },
}
local scenario = assert (scenarios[args.scenario or "georgia"], "scenario=georgia|georgia_retail|lebanon_retail")

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

if recording then host.open_recording {
	path = args.acmi or "campaign.acmi",
	title = args.title or scenario.title,
	reference_time = args.reference_time or "2026-09-24T06:00:00Z",
	affine = scenario.affine,
	latitude = tonumber (args.latitude) or scenario.latitude,
	longitude = tonumber (args.longitude) or scenario.longitude,
} end

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
local last_alive, last_activity = nil, 0
local conclusion
for frame = 1, frames do
	engine:frame (frame_ms)
	if frame % record_every == 0 then
		local clock = engine:clock ()
		local objects = engine:objects ()
		if recording then host.record (clock.elapsed_seconds, objects) end
		metrics:sample (clock.elapsed_seconds, objects)
		if metrics.samples == 1 or frame % math.floor (checkpoint_every * 1000 / frame_ms) < record_every then metrics:checkpoint (clock.elapsed_seconds, objects) end
		-- keysites changing hands
		for _, o in ipairs (objects) do
			if o.kind == "keysite" then
				if last_owner[o.name] and last_owner[o.name] ~= o.side then
					if recording then host.event (clock.elapsed_seconds, "Message", o.name .. " captured by " .. o.side) end
					host.log (string.format ("%.0f s: %s captured by %s", clock.elapsed_seconds, o.name, o.side))
				end
				if last_owner[o.name] ~= o.side then last_activity = clock.elapsed_seconds end
				last_owner[o.name] = o.side
			end
		end
		if stop_at_conclusion then
			-- air bases (airbases and FARPs) held, and units alive, per side
			local bases, alive = { blue = 0, red = 0 }, { blue = 0, red = 0 }
			for _, o in ipairs (objects) do
				if o.kind == "keysite" then
					if o.type_name == "KEYSITE_AIRBASE" or o.type_name == "KEYSITE_FARP" then bases[o.side] = (bases[o.side] or 0) + 1 end
				elseif o.kind ~= "weapon" and o.alive then
					alive[o.side] = (alive[o.side] or 0) + 1
				end
			end
			if last_alive and (alive.blue < last_alive.blue or alive.red < last_alive.red) then last_activity = clock.elapsed_seconds end
			last_alive = alive
			if bases.blue == 0 or bases.red == 0 then
				conclusion = string.format ("%s holds no airbase or FARP: %s wins", bases.blue == 0 and "blue" or "red", bases.blue == 0 and "red" or "blue")
			elseif clock.elapsed_seconds - last_activity >= stalemate_seconds then
				conclusion = string.format ("stalemate: nothing captured or destroyed for %g hours (airbases and FARPs: blue %d, red %d)",
					stalemate_seconds / 3600, bases.blue, bases.red)
			end
		end
		if diagnostics_every > 0 and frame % math.floor (diagnostics_every * 1000 / frame_ms) == 0 then
			engine:diagnostics ()
		end
		if frame % math.floor (log_every * 1000 / frame_ms) < record_every or conclusion then
			host.log (string.format ("%.0f s (day %d, %.0f s of day): %s", clock.elapsed_seconds, clock.day, clock.time_of_day_seconds, summary (objects)))
		end
		if conclusion then
			if recording then host.event (clock.elapsed_seconds, "Message", "Campaign over: " .. conclusion) end
			host.log (string.format ("%.0f s: campaign over: %s", clock.elapsed_seconds, conclusion))
			break
		end
	end
end
for _, line in ipairs (metrics:sortie_lines ()) do host.log ("sorties " .. line) end
if args.metrics then
	local clock = engine:clock ()
	if metrics.checkpoints[#metrics.checkpoints] == nil or metrics.checkpoints[#metrics.checkpoints].seconds < clock.elapsed_seconds - 1 then
		metrics:checkpoint (clock.elapsed_seconds, engine:objects ())
	end
	metrics:write (args.metrics, { scenario = args.scenario or "georgia", hours = hours, seed = tonumber (args.seed or "1"),
		frame_ms = frame_ms, sample_every = record_every, conclusion = conclusion or "none" })
	host.log ("metrics written to " .. args.metrics)
end
host.log ("campaign run complete")
