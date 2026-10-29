-- TEST-ONLY ground-contact characterisation (EEDCS-48, D3 G2). Needs a DLL built with the `ground-fixture` feature; the shipping module
-- has no `ground_probe` method.
--
-- It boots the campaign through the public path (require ("eech_dc")) with the reference host's callbacks, optionally runs frames, and
-- then calls the one fixture entry, engine:ground_probe (seed, points, impacts), which runs the ORIGINAL point_above_ground,
-- point_below_ground and get_ground_impact_point against the terrain the campaign's own boot loaded (see csrc/eech_ground_fixture.c).
-- The report is `key=value` text lines; each becomes one JSON Lines event. Raw map coordinates and heights appear only in that
-- runtime output (float bit patterns in hex), which is kept outside Git.
--
-- The probe is repeated (`repeats`, default 2) in the same process with no frames between: the reports must be byte identical, which
-- shows the probe is a pure function of the seed and the loaded map, and that the shared terrain record is left as it was found (the
-- record's value words are reported before and after each probe and must match). A difference fails the run; nothing is tuned to pass.
-- `probe=no` is the control: the same frames, no probe call, and the same state digest at the end, so a probe run and a control run can be
-- compared offline. The digest is the object list's identity, kind, side and alive flag in id order after the frames: a coarse witness
-- that the probe did not disturb the campaign, not a proof (see docs/m5-ground-characterization.md).
--
-- host.args (every one checked before anything is booted; an unknown key, a malformed number or an out-of-range value is refused):
--   root=<installation root>  out=<JSON Lines path>                                              required
--   scenario  lebanon_retail (default), georgia_retail or georgia
--   seed      1 to 4294967295 (default 20261029): the private generator's seed; 0 is refused because the generator treats it as 1
--   points    0 to 100000 (default 2000): point cases (point_above_ground and point_below_ground)
--   impacts   0 to 100000 (default 500): impact cases (get_ground_impact_point); points and impacts are not both 0
--   repeats   1 to 5 (default 2): how many times the probe is run in a row
--   seconds   0 to 3600 (default 0): simulated seconds of frames before the probe (0 probes the freshly booted world)
--   after     0 to 3600 (default 0): simulated seconds of frames after the probe, before the digest
--   frame_ms  1 to 1000 (default 100)
--   probe     yes (default) or no (the control)

io.stdout:setvbuf ("no")

local args = host.args

local function refuse (message)
	error ("ground-characterization: " .. message, 0)
end

local known = { root = true, out = true, scenario = true, seed = true, points = true, impacts = true, repeats = true, seconds = true,
	after = true, frame_ms = true, probe = true }
for key in pairs (args) do
	if not known [key] then refuse ("unknown argument '" .. tostring (key) .. "'") end
end

local function whole_arg (name, value, default, minimum, maximum)
	if value == nil then return default end
	if type (value) ~= "string" or not string.match (value, "^%d+$") then
		refuse (name .. " must be a whole number from " .. minimum .. " to " .. maximum .. ", got '" .. tostring (value) .. "'")
	end
	local n = tonumber (value)
	if n < minimum or n > maximum then
		refuse (name .. " must be a whole number from " .. minimum .. " to " .. maximum .. ", got '" .. value .. "'")
	end
	return n
end

local function choice_arg (name, value, default, allowed)
	if value == nil then return default end
	for _, option in ipairs (allowed) do if value == option then return value end end
	refuse (name .. " must be one of " .. table.concat (allowed, ", ") .. ", got '" .. tostring (value) .. "'")
end

local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end

local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
	georgia = { map = "..\\common\\maps\\map16", campaign = "georgia.chc" },
}
local scenario_name = args.scenario == nil and "lebanon_retail" or args.scenario
local scenario = scenarios [scenario_name]
if scenario == nil then refuse ("scenario must be lebanon_retail, georgia_retail or georgia, got '" .. tostring (scenario_name) .. "'") end

local seed = whole_arg ("seed", args.seed, 20261029, 1, 4294967295)
local points = whole_arg ("points", args.points, 2000, 0, 100000)
local impacts = whole_arg ("impacts", args.impacts, 500, 0, 100000)
local repeats = whole_arg ("repeats", args.repeats, 2, 1, 5)
local seconds = whole_arg ("seconds", args.seconds, 0, 0, 3600)
local after = whole_arg ("after", args.after, 0, 0, 3600)
local frame_ms = whole_arg ("frame_ms", args.frame_ms, 100, 1, 1000)
local probe = choice_arg ("probe", args.probe, "yes", { "yes", "no" })
if probe == "yes" and points + impacts == 0 then refuse ("points and impacts are both 0: there is nothing to probe") end

local MAX_FRAMES = 72000
local frames_before = math.floor (seconds * 1000 / frame_ms)
local frames_after = math.floor (after * 1000 / frame_ms)
if frames_before + frames_after > MAX_FRAMES then
	refuse ("seconds and after need " .. (frames_before + frames_after) .. " frames of " .. frame_ms .. " ms; the most this driver runs is " .. MAX_FRAMES)
end

local out = assert (io.open (out_path, "w"))

local function emit (t)
	local parts = {}
	for k, v in pairs (t) do
		-- a line break inside a string would not be JSON (%q writes it as a backslash and a newline): an error text may have several lines
		local value = type (v) == "string" and string.format ("%q", (string.gsub (v, "[\r\n]+", " "))) or tostring (v)
		parts [#parts + 1] = string.format ("%q:%s", k, value)
	end
	table.sort (parts)
	out:write ("{" .. table.concat (parts, ",") .. "}\n")
	out:flush ()
end

-- one JSON object per report line: the first word is the event and the `key=value` words are its fields. Sample and hash lines, and the
-- hex words of the record line, are strings (a hex word of decimal digits must not become a number); every counter is a number.
local function report_events (report, repeat_index)
	local events = {}
	for line in string.gmatch (report, "[^\n]+") do
		local event, rest = string.match (line, "^(%S+)%s*(.*)$")
		local t = { event = event, repeat_index = repeat_index }
		for k, v in string.gmatch (rest, "(%S+)=(%S+)") do
			local as_text = event == "ground-sample" or event == "ground-hash" or event == "ground-refused" or k == "words_before" or k == "words_after"
			t [k] = (not as_text and tonumber (v)) or v
		end
		events [#events + 1] = t
	end
	return events
end

local function find_event (events, name)
	for _, t in ipairs (events) do if t.event == name then return t end end
	return nil
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = scenario.map, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
	line_of_sight = host.line_of_sight,
	sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = scenario_name, campaign_seed = 1, seed = seed, points = points, impacts = impacts, repeats = repeats, seconds = seconds, after = after,
	frame_ms = frame_ms, probe = probe }

local function run_frames (count)
	for _ = 1, count do engine:frame (frame_ms) end
end

run_frames (frames_before)

local first_report, last_events
if probe == "yes" then
	for r = 1, repeats do
		local ok, report = pcall (function () return engine:ground_probe (seed, points, impacts) end)
		if not ok then
			-- a refused probe (a precondition not held, a report that does not fit) is an unavailable characterisation, never a quiet success
			emit { event = "probe_refused", repeat_index = r, error = tostring (report) }
			error ("ground-characterization: the probe was refused: " .. tostring (report), 0)
		end
		local events = report_events (report, r)
		for _, t in ipairs (events) do emit (t) end
		-- a report that does not describe the request, or lacks a part, is not a result
		local header = find_event (events, "ground-probe")
		local record = find_event (events, "ground-record")
		local hash = find_event (events, "ground-hash")
		for _, name in ipairs { "ground-points", "ground-above", "ground-below", "ground-complement", "ground-impact", "ground-asserts" } do
			if not find_event (events, name) then refuse ("the report has no " .. name .. " line") end
		end
		if not (header and record and hash) then refuse ("the report has no ground-probe, ground-record or ground-hash line") end
		if header.seed ~= seed or header.points ~= points or header.impacts ~= impacts then
			refuse (string.format ("the report is for seed %s points %s impacts %s, not the %s %s %s asked", tostring (header.seed), tostring (header.points),
				tostring (header.impacts), tostring (seed), tostring (points), tostring (impacts)))
		end
		-- the shared terrain record is left exactly as it was found
		if record.restored ~= 1 or record.words_before ~= record.words_after then
			refuse ("the probe did not restore the shared terrain record: restored=" .. tostring (record.restored) .. " words " .. tostring (record.words_before) ..
				" -> " .. tostring (record.words_after))
		end
		if first_report == nil then
			first_report = report
		else
			-- the same seed on the same loaded map and the same record: the same report, byte for byte
			local same = report == first_report
			emit { event = "repeat_identical", repeat_index = r, ok = same }
			if not same then refuse ("repeat " .. r .. " differs from repeat 1: the probe is not a pure function of the seed and the loaded map") end
		end
		last_events = events
	end
else
	emit { event = "probe_skipped", reason = "probe=no (the control)" }
end

run_frames (frames_after)

-- the coarse state digest: the object list's identity, kind, side and alive flag in id order (a modular polynomial hash, exact in doubles).
-- The modulus keeps it below 2^31: Lua 5.1 formats %d and %x through a C long, which is 32 bits on Windows.
local objects = engine:objects ()
table.sort (objects, function (a, b) return a.id < b.id end)
local digest, alive = 5381, 0
for _, o in ipairs (objects) do
	local text = tostring (o.id) .. ":" .. tostring (o.kind) .. ":" .. tostring (o.side) .. ":" .. tostring (o.alive) .. ";"
	for i = 1, #text do digest = (digest * 31 + string.byte (text, i)) % 2147483647 end
	if o.alive then alive = alive + 1 end
end
local clock = engine:clock ()
emit { event = "state_digest", objects = #objects, alive = alive, digest = string.format ("%08x", digest), elapsed = clock.elapsed_seconds,
	frames = frames_before + frames_after, probed = probe == "yes" }
out:close ()
local combined = last_events and find_event (last_events, "ground-hash") and find_event (last_events, "ground-hash").combined or "none"
io.stderr:write (string.format ("ground-characterization: probe %s, %d repeats, combined hash %s, state digest %08x over %d objects\n", probe, probe == "yes" and repeats or 0,
	tostring (combined), digest, #objects))
