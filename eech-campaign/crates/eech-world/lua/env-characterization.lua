-- TEST-ONLY characterisation of the ORIGINAL session environment: weather, wind and the clock (EEDCS-51, first increment of D3). Needs a DLL built with the `env-fixture` feature; the shipping
-- module has no `env_case` method and this script then REFUSES (a `refusal` row, a non-zero exit), it never reports success without the fixture.
-- SOURCE PREPARED, UNBUILT, UNRUN, UNREVIEWED. Nothing here has been executed; docs/d3-environment-characterization.md says what must be run and what each result would and would not show.
--
-- It boots the campaign through the public path (require ("eech_dc")) with the reference host's callbacks, exactly as the other characterisation drivers do, runs the ordinary frames, and reads
-- the original code's environment through csrc/eech_env_fixture.c:
--   * the CENSUS (group 0): the case counts, enum values, constants and the `env-unsupported` items that cannot be witnessed;
--   * RETAIL snapshots (group 1, read-only, `tag=retail`): the unmodified session's clock, weather and wind words, the shared random seed word, the frame delta, the camera and the original
--     queries at fixed points of the live state. Taken before the first frame, after each of the first `snapshot_frames` frames, every `snapshot_every` simulated seconds and at the end;
--   * WITNESS cases (groups 2 to 8, `kind=witness`): finite controlled synthetic inputs to the original routines, run once at `probe_at`, each of which restores everything it touched and says so
--     (`env-witness-end restored=1`). A case that does not restore is a defect of the fixture and fails the run; it is never a result.
-- Float values are IEEE-754 words (keys ending `_w`, 8 hex digits) and stay STRINGS here; fingerprints too. Nothing is rounded, normalised or compared with a tolerance.
--
-- CONTROL PROTOCOL (probe noninterference). Run the same arguments twice, once with probes=0 (the control: the census and the read-only snapshots only) and once with the probes. The frames, the
-- seed and the schedule are identical, so every `snapshot` row of the two runs, the final retail snapshot, the `campaign_end` row and the frame-by-frame clock must be byte-identical
-- apart from the `probes`/`mode` fields of the boot row and the witness rows themselves. Inside one run, `probe_noninterference` compares the retail fingerprint and the campaign fingerprint
-- immediately before and after the whole witness set.
--
-- host.args (every one is VALIDATED before the output file is created, the installation prepared, the engine booted or anything changed; a malformed value is an error naming it, never a default):
--   root, out                required text (the installation root; the JSON Lines path)
--   scenario                 lebanon_retail (default) | georgia_retail
--   seed                     the boot random seed: an integer from 1 to 2147483647 (default 1)
--   seconds                  simulated seconds of ordinary frames: a plain decimal number from 0 to 21600 (default 120)
--   frame_ms                 an integer from 1 to 1000 (default 100). Zero never advances the clock, so it is refused
--   probes                   the most witness cases to run: an integer from 0 to 1024 (default 512). 0 is the control. The census says how many exist; the coverage row says how many ran
--   probe_at                 the simulated second at which the witness set runs: a plain decimal number from 0 to seconds (default 0)
--   snapshot_frames          retail snapshots after each of this many leading frames: an integer from 0 to 2000 (default 20)
--   snapshot_every           a retail snapshot every this many simulated seconds: a plain decimal number from 0 to 3600 (default 60; 0 none)
-- A "plain decimal number" is digits with an optional single point (no sign, exponent, hex, space or text after it). The frames may not exceed 120000, and every loop stops at its bound.
--
-- Output, JSON Lines, one object per line with sorted keys: `boot`; `census` rows; for each report line an object whose `event` is the line's first word, `scope` says which call produced it
-- (`snapshot:initial`, `snapshot:frame:N`, `snapshot:periodic:T`, `snapshot:final`, `probe:g<group>:<index>`) and `at` is the simulated clock; `probe_noninterference`; `campaign` rows;
-- `coverage`; `run`. A refusal or failure is a `refusal`/`failure` row followed by a non-zero exit.

io.stdout:setvbuf ("no")

local NAME = "env-characterization"
local MAX_FRAMES = 120000
local args = host.args

local function refuse (message)
	error (NAME .. ": " .. message, 0)
end

-- a plain decimal (digits and at most one point) or integer (digits only) within [minimum, maximum]; anything else is refused by name
local function number_arg (name, value, default, minimum, maximum, integer)
	if value == nil then return default end
	local text = tostring (value)
	local plain
	if integer then
		plain = text:match ("^%d+$") ~= nil
	else
		plain = text:match ("^%d+%.?%d*$") ~= nil or text:match ("^%.%d+$") ~= nil
	end
	if not plain or #text > 12 then
		refuse (name .. " must be a plain non-negative " .. (integer and "integer" or "decimal number") .. " of at most twelve characters, got '" .. text .. "'")
	end
	local number = tonumber (text)
	if number == nil or number ~= number or number == math.huge then
		refuse (name .. " is not a finite number, got '" .. text .. "'")
	end
	if number < minimum or number > maximum then
		refuse (name .. " must be from " .. tostring (minimum) .. " to " .. tostring (maximum) .. ", got '" .. text .. "'")
	end
	return number
end

local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario_name = args.scenario == nil and "lebanon_retail" or args.scenario
local scenario = scenarios [scenario_name]
if scenario == nil then refuse ("scenario must be lebanon_retail or georgia_retail, got '" .. tostring (scenario_name) .. "'") end
local seed = number_arg ("seed", args.seed, 1, 1, 2147483647, true)
local seconds = number_arg ("seconds", args.seconds, 120, 0, 21600, false)
local frame_ms = number_arg ("frame_ms", args.frame_ms, 100, 1, 1000, true)
local probes = number_arg ("probes", args.probes, 512, 0, 1024, true)
local probe_at = number_arg ("probe_at", args.probe_at, 0, 0, 21600, false)
local snapshot_frames = number_arg ("snapshot_frames", args.snapshot_frames, 20, 0, 2000, true)
local snapshot_every = number_arg ("snapshot_every", args.snapshot_every, 60, 0, 3600, false)
local frames = math.ceil (seconds * 1000 / frame_ms)
if frames > MAX_FRAMES then
	refuse ("seconds=" .. seconds .. " at frame_ms=" .. frame_ms .. " needs " .. frames .. " frames; the most this driver runs is " .. MAX_FRAMES)
end
if probes > 0 and probe_at > seconds then
	refuse ("probe_at=" .. probe_at .. " is later than seconds=" .. seconds .. ", so the witness set would never run")
end

-- the checked writer: every write, every flush and the close are tested, and a failure is an error naming the file
local out = assert (io.open (out_path, "w"))
local function checked (ok, err, what)
	if not ok then error (NAME .. ": " .. what .. " of " .. out_path .. " failed: " .. tostring (err), 0) end
end

local escapes = { ['"'] = '\\"', ['\\'] = '\\\\', ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t' }
local function json_string (s)
	return '"' .. (string.gsub (s, '[%c"\\\128-\255]', function (c) return escapes [c] or string.format ("\\u%04x", string.byte (c)) end)) .. '"'
end
local function json_value (v)
	if type (v) == "string" then return json_string (v) end
	if type (v) == "boolean" then return v and "true" or "false" end
	if v ~= v or v == math.huge or v == -math.huge then return json_string (tostring (v)) end
	if v == math.floor (v) and math.abs (v) < 1e15 then return string.format ("%d", v) end
	return string.format ("%.17g", v)
end
local function emit (t)
	local parts = {}
	for k, v in pairs (t) do parts [#parts + 1] = json_string (k) .. ":" .. json_value (v) end
	table.sort (parts)
	local ok, err = out:write ("{" .. table.concat (parts, ",") .. "}\n")
	checked (ok, err, "write")
	ok, err = out:flush ()
	checked (ok, err, "flush")
end

-- the keys that stay strings: every float word, every fingerprint. Anything else that is a plain decimal number becomes a number.
local string_keys = { fp = true, fp_before = true, fp_after = true }
local reserved = { event = true, scope = true, at = true }
local function field_value (key, value)
	if string_keys [key] or key:match ("_w$") then return value end
	if value:match ("^-?%d+$") or value:match ("^-?%d+%.%d+$") then return tonumber (value) end
	return value
end

-- one object per report line; returns them in order
local clock
local function emit_report (scope, report)
	local rows = {}
	for line in string.gmatch (report, "[^\n]+") do
		local event, rest = string.match (line, "^(%S+)%s*(.*)$")
		local t = { event = event, scope = scope, at = clock.elapsed_seconds }
		for k, v in string.gmatch (rest, "(%S+)=(%S+)") do
			t [reserved [k] and ("field_" .. k) or k] = field_value (k, v)
		end
		emit (t)
		rows [#rows + 1] = t
	end
	return rows
end

local dc, engine
local finished = false

local function run ()
	emit { event = "boot", scenario = scenario_name, seed = seed, seconds = seconds, frame_ms = frame_ms, probes = probes, probe_at = probe_at, snapshot_frames = snapshot_frames,
		snapshot_every = snapshot_every, mode = probes == 0 and "control" or "probe", feature = "env-fixture" }

	dc = require ("eech_dc")
	dc.prepare_installation (root)
	engine = dc.boot {
		install_root = root, map = scenario.map, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
		seed = seed, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
		supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = host.aim,
	}
	clock = engine:clock ()

	-- the genuine refusal: a DLL built without the feature has no such method, and nothing is then reported as measured
	local present, method = pcall (function () return engine.env_case end)
	if not present or type (method) ~= "function" then
		emit { event = "refusal", reason = "env_case_unavailable", detail = "the module was built without the env-fixture feature", at = clock.elapsed_seconds }
		refuse ("the engine has no env_case method: build eech-dc with the env-fixture feature")
	end

	local function case (group, index, scope)
		local ok, report = pcall (engine.env_case, engine, group, index)
		if not ok then refuse ("env_case(" .. group .. ", " .. index .. ") failed: " .. tostring (report)) end
		if type (report) ~= "string" or report == "" then refuse ("env_case(" .. group .. ", " .. index .. ") returned no report") end
		return emit_report (scope, report)
	end

	local function first (rows, event)
		for _, row in ipairs (rows) do
			if row.event == event then return row end
		end
	end

	-- a fingerprint of the campaign's observed objects in the order the engine lists them: a polynomial checksum that stays inside a double's exact integers
	local function campaign_fingerprint ()
		local objects = engine:objects ()
		local h, alive = 7, 0
		local function mix (text)
			for i = 1, #text do h = (h * 131 + string.byte (text, i)) % 2147483647 end
		end
		for _, o in ipairs (objects) do
			if o.alive then alive = alive + 1 end
			local p = o.position or {}
			mix (table.concat ({ tostring (o.id), tostring (o.kind), tostring (o.sub_type), tostring (o.side), tostring (o.alive), string.format ("%.9g|%.9g|%.9g", p [1] or 0, p [2] or 0, p [3] or 0),
				string.format ("%.9g|%.9g|%.9g|%.9g|%.9g", o.heading or 0, o.pitch or 0, o.roll or 0, o.ammo or 0, o.fuel or 0), tostring (o.usable), tostring (o.task), tostring (o.state) }, "|"))
		end
		return { objects = #objects, alive = alive, fp = string.format ("%d", h) }
	end
	local function campaign_row (scope)
		local f = campaign_fingerprint ()
		emit { event = "campaign", scope = scope, at = clock.elapsed_seconds, objects = f.objects, alive = f.alive, fp = f.fp }
		return f
	end

	local census_rows = case (0, 0, "census")
	local census = first (census_rows, "env-census")
	if census == nil then refuse ("the census report has no env-census line") end
	local groups = {
		{ 2, "weather", "weather_cases" }, { 3, "wind", "wind_cases" }, { 4, "gust", "gust_cases" }, { 5, "time", "time_cases" },
		{ 6, "init", "init_cases" }, { 7, "update_time", "update_time_cases" }, { 8, "update_weather", "update_weather_cases" },
	}
	local available = 0
	for _, g in ipairs (groups) do
		local n = census [g [3]]
		if type (n) ~= "number" or n < 0 or n ~= math.floor (n) then refuse ("the census has no valid " .. g [3]) end
		g.count, g.ran, g.refused = n, 0, 0
		available = available + n
	end
	local unsupported = 0
	for _, row in ipairs (census_rows) do
		if row.event == "env-unsupported" then unsupported = unsupported + 1 end
	end

	local frames_run = 0
	local function step_frame ()
		local before = clock.elapsed_seconds
		frames_run = frames_run + 1
		if frames_run > MAX_FRAMES + 2 then refuse ("more than " .. MAX_FRAMES .. " frames were run") end
		engine:frame (frame_ms)
		clock = engine:clock ()
		if not (clock.elapsed_seconds > before) then refuse ("the clock did not advance in a frame of " .. frame_ms .. " ms") end
	end

	local function snapshot (scope)
		local rows = case (1, 0, scope)
		local fp = first (rows, "env-state-fingerprint")
		if fp == nil or type (fp.fp) ~= "string" then refuse ("the snapshot has no fingerprint line") end
		return fp.fp
	end

	local probes_run = 0
	local probe_state = "not_requested"
	local function run_probes ()
		probe_state = "ran"
		local retail_before, campaign_before = snapshot ("snapshot:before_probes"), campaign_row ("campaign:before_probes")
		for _, g in ipairs (groups) do
			for index = 0, g.count - 1 do
				if probes_run >= probes then break end
				local rows = case (g [1], index, "probe:g" .. g [1] .. ":" .. index)
				local last = rows [#rows]
				if last == nil or last.event ~= "env-witness-end" or last.restored ~= 1 or last.group ~= g [1] or last.index ~= index then
					refuse ("witness g" .. g [1] .. ":" .. index .. " did not end with a restored witness: the fixture's own invariant failed")
				end
				for _, row in ipairs (rows) do
					if row.refused ~= nil then g.refused = g.refused + 1 break end
				end
				g.ran, probes_run = g.ran + 1, probes_run + 1
			end
		end
		local retail_after, campaign_after = snapshot ("snapshot:after_probes"), campaign_row ("campaign:after_probes")
		emit { event = "probe_noninterference", retail_before = retail_before, retail_after = retail_after, retail_equal = retail_before == retail_after,
			campaign_before = campaign_before.fp, campaign_after = campaign_after.fp, campaign_equal = campaign_before.fp == campaign_after.fp, probes_run = probes_run }
		if retail_before ~= retail_after or campaign_before.fp ~= campaign_after.fp then
			refuse ("the witness set changed the retail environment or the campaign: noninterference failed")
		end
	end

	campaign_row ("campaign:initial")
	snapshot ("snapshot:initial")
	if probes > 0 then probe_state = "waiting" end
	local next_periodic = snapshot_every > 0 and snapshot_every or nil
	for n = 1, frames do
		if probes > 0 and probe_state == "waiting" and clock.elapsed_seconds >= probe_at then run_probes () end
		step_frame ()
		if n <= snapshot_frames then snapshot ("snapshot:frame:" .. n) end
		if next_periodic ~= nil and clock.elapsed_seconds >= next_periodic then
			snapshot ("snapshot:periodic:" .. string.format ("%.3f", clock.elapsed_seconds))
			while next_periodic <= clock.elapsed_seconds do next_periodic = next_periodic + snapshot_every end
		end
	end
	if probes > 0 and probe_state == "waiting" then run_probes () end
	snapshot ("snapshot:final")
	campaign_row ("campaign_end")

	for _, g in ipairs (groups) do
		emit { event = "coverage", group = g [1], name = g [2], available = g.count, ran = g.ran, refused = g.refused, truncated = g.ran < g.count }
	end
	emit { event = "run", frames = frames_run, probes_requested = probes, probes_run = probes_run, cases_available = available, unsupported_items = unsupported, complete = probes_run == available,
		probe_state = probe_state, elapsed_seconds = clock.elapsed_seconds }
	finished = true
end

local ok, problem = pcall (run)
if not ok then
	-- a failure is recorded as a row when the file can still take one, then the run fails
	pcall (function () emit { event = "failure", message = tostring (problem) } end)
	pcall (function () out:close () end)
	error (tostring (problem), 0)
end
local closed, close_error = out:close ()
checked (closed, close_error, "close")
print ("ENV_CHARACTERIZATION_DONE")
