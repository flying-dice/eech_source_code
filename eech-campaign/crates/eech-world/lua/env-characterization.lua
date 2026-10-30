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
--   snapshot_every           a retail snapshot every this many simulated seconds: a plain decimal number from 0 to 3600 (default 60; 0 none). At most one is taken per frame, whatever the number of periods the frame covered; the next due time is reached in constant work, so a tiny period is bounded by the frames
-- A "plain decimal number" is digits with an optional single point (no sign, exponent, hex, space or text after it). The frames may not exceed 120000, and every loop stops at its bound.
--
-- Output, JSON Lines, one object per line with sorted keys: `boot`; `census` rows; for each report line an object whose `event` is the line's first word, `scope` says which call produced it
-- (`snapshot:initial`, `snapshot:frame:N`, `snapshot:periodic:T`, `snapshot:final`, `probe:g<group>:<index>`) and `at` is the simulated clock; `probe_noninterference`; `campaign` rows;
-- `coverage`; `validation` (reports accepted, witnesses, refused witnesses); `run`. A report the fixture sends is accepted only in the protocol shape documented at `validate_report`; a refusal or failure is a `refusal`/`failure` row followed by a non-zero exit.

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

-- THE EMITTED PROTOCOL. csrc/eech_env_fixture.c prints a fixed, ordered set of rows for each call. A report is parsed strictly (a truncated report, an empty or non-ASCII line, a field that is not
-- key=value, a duplicate key), written out exactly as it came, and then accepted ONLY if it has the protocol's shape for the group and index that were asked: the rows in order, the keys in order,
-- every value of its kind (an 8-digit lowercase hex word, a 16-digit fingerprint, an integer, a flag), the case identity, the declared supported or refused form, and for a witness the end marker
-- with restored=1 and a fingerprint after that equals the fingerprint before. The shape is checked, never the physical answer: no value is computed or compared with an expected number here.
local REPORT_LIMIT = 32 * 1024 -- the fixture's buffer (Engine::env_case); a report must be shorter than it
local clock

local function parse_report (report)
	if type (report) ~= "string" or report == "" then return nil, "no report" end
	if #report >= REPORT_LIMIT then return nil, "a report of " .. #report .. " bytes does not fit the fixture's " .. REPORT_LIMIT .. "-byte buffer" end
	if report:sub (-1) ~= "\n" then return nil, "the report is truncated: it does not end with a newline" end
	local lines = {}
	for line in report:gmatch ("([^\n]*)\n") do
		if line == "" then return nil, "the report has an empty line" end
		if line:find ("[^\32-\126]") then return nil, "the report has a byte that is not printable ASCII" end
		local event, rest = line:match ("^([%a][%w%-_]*)(.*)$")
		if event == nil or (rest ~= "" and rest:sub (1, 1) ~= " ") then return nil, "a report line has no event name: " .. line:sub (1, 60) end
		local fields, seen = {}, {}
		for token in rest:gmatch ("%S+") do
			local k, v = token:match ("^([%a_][%w_]*)=([^=]+)$")
			if k == nil then return nil, "malformed field '" .. token .. "' in " .. event end
			if seen [k] ~= nil then return nil, "duplicate field '" .. k .. "' in " .. event end
			seen [k] = v
			fields [#fields + 1] = { k, v }
		end
		lines [#lines + 1] = { event = event, fields = fields, map = seen }
	end
	return lines
end

-- one object per parsed line, in order; the same objects as ever for a well-formed report
local function emit_lines (scope, lines)
	local rows = {}
	for _, line in ipairs (lines) do
		local t = { event = line.event, scope = scope, at = clock.elapsed_seconds }
		for _, kv in ipairs (line.fields) do
			t [reserved [kv [1]] and ("field_" .. kv [1]) or kv [1]] = field_value (kv [1], kv [2])
		end
		emit (t)
		rows [#rows + 1] = t
	end
	return rows
end

-- field kinds: w a float word, f a fingerprint, i an integer, u a non-negative integer, b a flag, s a token
local kinds = {
	w = { "an 8-digit lowercase hex word", function (v) return #v == 8 and v:match ("^%x+$") ~= nil and v == v:lower () end },
	f = { "a 16-digit lowercase hex fingerprint", function (v) return #v == 16 and v:match ("^%x+$") ~= nil and v == v:lower () end },
	i = { "an integer", function (v) return #v <= 11 and v:match ("^-?%d+$") ~= nil end },
	u = { "a non-negative integer", function (v) return #v <= 9 and v:match ("^%d+$") ~= nil end },
	b = { "0 or 1", function (v) return v == "0" or v == "1" end },
	s = { "a token", function (v) return #v <= 200 and v:match ("^[%w_,]+$") ~= nil end },
}

-- "key:kind" is a typed value, "key=a|b" one of the literal values; the keys are in the order the C prints them
local function compile (text)
	local spec = {}
	for token in text:gmatch ("%S+") do
		local k, kind = token:match ("^([%w_]+):(%a)$")
		if k ~= nil and kinds [kind] ~= nil then
			spec [#spec + 1] = { key = k, kind = kind }
		else
			local key, literal = token:match ("^([%w_]+)=(.+)$")
			if key == nil then error ("env-characterization: bad protocol spec token '" .. token .. "'", 0) end
			local set = {}
			for alternative in literal:gmatch ("[^|]+") do set [alternative] = true end
			spec [#spec + 1] = { key = key, set = set }
		end
	end
	return spec
end

local function spec_error (line, spec)
	if #line.fields ~= #spec then return "has " .. #line.fields .. " fields, the protocol has " .. #spec end
	for n, want in ipairs (spec) do
		local k, v = line.fields [n] [1], line.fields [n] [2]
		if k ~= want.key then return "field " .. n .. " is '" .. k .. "', the protocol has '" .. want.key .. "'" end
		if want.set ~= nil then
			if not want.set [v] then return k .. "='" .. v .. "' is not a value the protocol allows" end
		elseif not kinds [want.kind] [2] (v) then
			return k .. "='" .. v .. "' is not " .. kinds [want.kind] [1]
		end
	end
end

-- a template is an ordered list of rows, each an event name and the alternative field specs it may have
local function row (event, ...)
	local alts = {}
	for _, text in ipairs ({ ... }) do alts [#alts + 1] = compile (text) end
	return { event = event, alts = alts }
end

local function template_error (lines, template)
	for n, want in ipairs (template) do
		local line = lines [n]
		if line == nil then return "row " .. n .. " is missing, expected '" .. want.event .. "'" end
		if line.event ~= want.event then return "row " .. n .. " is '" .. line.event .. "', expected '" .. want.event .. "'" end
		local why
		for _, spec in ipairs (want.alts) do
			why = spec_error (line, spec)
			if why == nil then break end
		end
		if why ~= nil then return "row " .. n .. " (" .. line.event .. ") " .. why end
	end
	if #lines > #template then return "has " .. #lines .. " rows, expected " .. #template end
end

-- the session state: four rows (time, weather, wind, globals) under an event prefix and a tag; camera 0 or 1 pins the globals row's camera field, nil allows either
local function state_rows (prefix, tag, camera)
	local globals = "tag=" .. tag .. " season:i rng:i rng_w:w delta_time_w:w comms_model:i in_cockpit:u camera="
	local alts = {}
	if camera ~= 1 then alts [#alts + 1] = globals .. "0" end
	if camera ~= 0 then alts [#alts + 1] = globals .. "1 camera_x_w:w camera_y_w:w camera_z_w:w" end
	return {
		row (prefix .. "-time", "tag=" .. tag .. " start_time_w:w elapsed_time_w:w time_of_day_w:w day:i day_segment_type:u time_of_day_acceleration_w:w skip_night_time:b time_of_day_resync_w:w"),
		row (prefix .. "-weather", "tag=" .. tag .. " local_weather_model:b weather_increasing:b weather_radius_w:w weather_mode:u target_weather_mode:u transitional_status_w:w transitional_period_w:w"
			.. " lightning_timer_w:w weather_pos_x_w:w weather_pos_y_w:w weather_pos_z_w:w weather_vel_x_w:w weather_vel_y_w:w weather_vel_z_w:w"),
		row (prefix .. "-wind", "tag=" .. tag .. " wind_increasing:b wind_effect_radius_w:w gusting_value_w:w minimum_speed_w:w maximum_speed_w:w direction_x_w:w direction_y_w:w direction_z_w:w"
			.. " effect_pos_x_w:w effect_pos_y_w:w effect_pos_z_w:w effect_vel_x_w:w effect_vel_y_w:w effect_vel_z_w:w"),
		row (prefix .. "-globals", unpack (alts)),
	}
end

-- one original query at one point: the weather answer and either the wind answer or (only where the state can leave the gust domain, the retail snapshot) the stated absence of it
local function query_row (event, tag, point, allow_unavailable)
	local base = "tag=" .. tag .. " point=" .. point .. " x_w:w y_w:w z_w:w cwm:u twm:u ts_w:w simple:u"
	if allow_unavailable then return row (event, base .. " wind_speed_w:w wind_x_w:w wind_y_w:w wind_z_w:w", base .. " wind=unavailable_gust_outside_0_1") end
	return row (event, base .. " wind_speed_w:w wind_x_w:w wind_y_w:w wind_z_w:w")
end

local function concat (...)
	local all = {}
	for _, part in ipairs ({ ... }) do
		for _, r in ipairs (part) do all [#all + 1] = r end
	end
	return all
end

local function end_row ()
	return row ("env-witness-end", "group:u index:u restored=1 fp_before:f fp_after:f")
end

-- the census: bounded counts (the capacities of the C tables), the enum values of the headers, and the six unsupported items in order
local census_caps = { weather_cases = 96, wind_cases = 48, gust_cases = 48, time_cases = 48, init_cases = 64, update_time_cases = 64, update_weather_cases = 64 }
local unsupported_items = { "get_session_random_start_time_of_day", "update_lightning_effect", "update_session_sound_effects", "update_time_of_day_resync", "wind_gust_outside_0_1",
	"session_state_loading_and_serialisation" }

local function census_template ()
	local t = {
		row ("env-census", "version=1 session_bytes:u weather_cases:u wind_cases:u gust_cases:u time_cases:u init_cases:u update_time_cases:u update_weather_cases:u"),
		row ("env-census-enums", "dry=1 light_rain=2 heavy_rain=3 snow=4 season_summer=2 season_winter=3 season_desert=4 segment_dawn=0 segment_day=1 segment_dusk=2 segment_night=3 comms_server:u"),
		row ("env-census-constants", "min_weather_radius_w:w max_weather_radius_w:w weather_expansion_rate_w:w min_wind_radius_w:w max_wind_radius_w:w wind_expansion_rate_w:w wind_gust_frequency_w:w"
			.. " session_resync_frequency_w:w default_min_wind_speed_w:w default_max_wind_speed_w:w one_hour_w:w one_day_w:w"),
		row ("env-census-configured", "weather_rain_radius_min_w:w weather_rain_radius_max_w:w weather_rain_speed_w:w weather_wind_radius_min_w:w weather_wind_radius_max_w:w weather_wind_speed_w:w"),
		row ("env-census-map", "min_map_x_w:w max_map_x_w:w min_map_z_w:w max_map_z_w:w mid_map_x_w:w mid_map_z_w:w"),
	}
	for _, item in ipairs (unsupported_items) do t [#t + 1] = row ("env-unsupported", "item=" .. item .. " reason:s") end
	return t
end

-- the float word of a gust is inside the original's domain [0,1) when it is below the word of 1.0 or is exactly -0.0 (the original asserts anywhere else)
local function gust_word_in_domain (word)
	local n = tonumber (word, 16)
	return n < 0x3f800000 or n == 0x80000000
end

-- the possible shapes of the report for (group, index): each is { name, template, refused }
local function shapes (group, index)
	local who = "kind=witness i:u"
	local witness = "witness"
	if group == 0 then return { { "census", census_template (), false } } end
	if group == 1 then
		local function snapshot_shape (camera)
			local rows = state_rows ("env-state", "retail", camera)
			rows [#rows + 1] = row ("env-state-fingerprint", "tag=retail fp:f")
			if camera == 1 then rows [#rows + 1] = query_row ("env-query", "retail", "camera", true) end
			rows [#rows + 1] = query_row ("env-query", "retail", "map_centre", true)
			rows [#rows + 1] = query_row ("env-query", "retail", "weather_centre", true)
			rows [#rows + 1] = query_row ("env-query", "retail", "wind_centre", true)
			return rows
		end
		return { { "snapshot_with_camera", snapshot_shape (1), false }, { "snapshot_without_camera", snapshot_shape (0), false } }
	end
	if group == 2 then
		return { { "weather", { row ("env-weather", who .. " model=local|global season:i radius_w:w in_cwm:u in_twm:u in_ts_w:w range_w:w"),
			query_row ("env-weather-answer", witness, "case", false), end_row () }, false } }
	end
	if group == 3 then
		return {
			{ "wind", { row ("env-wind", who .. " radius_w:w centre_x_w:w centre_z_w:w minimum_w:w maximum_w:w gust_w:w dir_x_w:w dir_z_w:w x_w:w z_w:w range_w:w modulation_w:w speed_w:w vx_w:w vy_w:w vz_w:w"
				.. " null_speed_w:w null_same:b"), end_row () }, false },
			{ "wind_refused", { row ("env-wind", who .. " refused=gust_outside_0_1"), end_row () }, true },
		}
	end
	if group == 4 then
		return {
			{ "gust", { row ("env-gust", who .. " gust_w:w modulation_w:w calm_speed_w:w calm_vx_w:w"), end_row () }, false },
			{ "gust_refused", { row ("env-gust", who .. " gust_w:w refused=original_asserts_outside_0_1"), end_row () }, true },
		}
	end
	if group == 5 then
		return {
			{ "time_calculate", { row ("env-time", who .. " what=calculate start_w:w elapsed_w:w tod_w:w rolled_days:i accessor_tod_w:w accessor_day:i segment:u"), end_row () }, false },
			{ "time_segment", { row ("env-time", who .. " what=segment tod_w:w segment:u"), end_row () }, false },
			{ "time_constants", { row ("env-time", who .. " what=constants segment_time_dawn_w:w segment_time_day_w:w segment_time_dusk_w:w segment_time_night_w:w preset_dawn_w:w preset_midday_w:w"
				.. " preset_afternoon_w:w preset_dusk_w:w preset_midnight_w:w"), end_row () }, false },
		}
	end
	if group == 6 then
		local function tail () return { state_rows ("env-init-state", witness), { query_row ("env-init-query", witness, "origin", false), query_row ("env-init-query", witness, "map_centre", false) } } end
		local t1, t2 = tail (), tail ()
		return {
			{ "init_fixed", concat ({ row ("env-init", who .. " what=fixed mode=1|2|3|4 rng:i") }, t1 [1], t1 [2], { end_row () }), false },
			{ "init_random", concat ({ row ("env-init", who .. " what=random seed_in:i rng:i rng_w:w") }, t2 [1], t2 [2], { end_row () }), false },
			{ "init_refused", { row ("env-init", who .. " refused=the_original_initialisers_assert_the_server_comms_model comms_model:i"), end_row () }, true },
		}
	end
	if group == 7 then
		return { { "update_time", concat ({ row ("env-update-time", who .. " label:s delta_w:w accel_w:w skip_night:b tod_before_w:w day_before:i") }, state_rows ("env-update-time-state", "before_" .. index),
			{ row ("env-update-time-result", who .. " tod_after_w:w day_after:i") }, state_rows ("env-update-time-state", "after_" .. index), { end_row () }), false } }
	end
	if group == 8 then
		return {
			{ "update_weather", concat ({ row ("env-update-weather", who .. " label:s delta_w:w camera_dependent_fields=weather_mode,target_weather_mode,transitional_status_when_local_and_radius_nonzero camera=1") },
				state_rows ("env-update-weather-state", "before_" .. index), state_rows ("env-update-weather-state", "after_" .. index), { end_row () }), false },
			{ "update_weather_refused", { row ("env-update-weather", who .. " refused=unavailable_in_cockpit_or_no_camera_or_not_server in_cockpit:u camera:b comms_model:i"), end_row () }, true },
		}
	end
	return {}
end

-- the rows of one event, in order
local function rows_of (lines, event)
	local found = {}
	for _, line in ipairs (lines) do
		if line.event == event then found [#found + 1] = line end
	end
	return found
end

-- accept a parsed report only if it has one of the protocol's shapes for (group, index); returns { shape, refused } or nil and the reasons. `context` carries what an earlier validated report fixed:
-- the census' comms_server value, by which the initialisers' and the weather update's own guards (printed by the emitter in the refusal and in the state rows) are judged.
local function validate_report (group, index, lines, context)
	local why = {}
	for _, shape in ipairs (shapes (group, index)) do
		local problem = template_error (lines, shape [2])
		if problem == nil then
			local verdict = { shape = shape [1], refused = shape [3] }
			-- the case identity: every row that names its case names this one
			for _, line in ipairs (lines) do
				if line.map.i ~= nil and tonumber (line.map.i) ~= index then return nil, "row " .. line.event .. " is for case i=" .. line.map.i .. ", not " .. index end
			end
			if group == 0 then
				local c, total = lines [1].map, 0
				local bytes = tonumber (c.session_bytes)
				if bytes < 16 or bytes > 1000000 then return nil, "the census session_bytes=" .. c.session_bytes .. " is outside 16..1000000" end
				for name, cap in pairs (census_caps) do
					local n = tonumber (c [name])
					if n < 1 or n > cap then return nil, "the census " .. name .. "=" .. c [name] .. " is outside 1.." .. cap end
					total = total + n
				end
				if total > 1024 then return nil, "the census declares " .. total .. " cases, more than 1024" end
			end
			if group >= 2 then
				local last = lines [#lines].map
				if tonumber (last.group) ~= group or tonumber (last.index) ~= index then return nil, "the end marker is for group " .. last.group .. " index " .. last.index end
				if last.fp_before ~= last.fp_after then return nil, "the state fingerprint changed across the witness: " .. last.fp_before .. " then " .. last.fp_after end
			end
			if group == 0 then verdict.comms_server = tonumber (lines [2].map.comms_server) end
			if group == 3 and not verdict.refused and not gust_word_in_domain (lines [1].map.gust_w) then return nil, "a supported wind row has a gust outside [0,1)" end
			-- the emitter defines null_same as the equality of its own two printed words; the speeds themselves are never compared with an expected value
			if group == 3 and not verdict.refused then
				local w = lines [1].map
				if w.null_same ~= ((w.speed_w == w.null_speed_w) and "1" or "0") then
					return nil, "null_same=" .. w.null_same .. " contradicts speed_w=" .. w.speed_w .. " and null_speed_w=" .. w.null_speed_w
				end
			end
			-- a query's wind answer exists exactly when the gust of the SAME report's state is inside [0,1) (the emitter's own branch); other state rows may show any gust word (a phase of 1.0 before it wraps)
			local state = (group == 1 or group == 6) and rows_of (lines, group == 1 and "env-state-wind" or "env-init-state-wind") [1] or nil
			if state ~= nil then
				local inside = gust_word_in_domain (state.map.gusting_value_w)
				for _, line in ipairs (lines) do
					if line.map.wind_speed_w ~= nil and not inside then return nil, "row " .. line.event .. " has a wind answer although the report's gusting_value_w=" .. state.map.gusting_value_w .. " is outside [0,1)" end
					if line.map.wind ~= nil and inside then return nil, "row " .. line.event .. " says wind unavailable although the report's gusting_value_w=" .. state.map.gusting_value_w .. " is inside [0,1)" end
				end
			end
			-- the printed guards of the initialisers (refused unless the server comms model) and of the weather update (refused if in_cockpit, no camera, or not the server): a refusal needs a guard that holds, a run its complement
			if group == 6 or group == 8 then
				local server = context.comms_server
				if server == nil then return nil, "there is no validated census comms_server to judge the guard by" end
				if group == 6 and verdict.refused and tonumber (lines [1].map.comms_model) == server then return nil, "the initialiser was refused although comms_model equals the census comms_server" end
				if group == 6 and not verdict.refused then
					for _, g in ipairs (rows_of (lines, "env-init-state-globals")) do
						if tonumber (g.map.comms_model) ~= server then return nil, "the initialiser ran although comms_model=" .. g.map.comms_model .. " is not the census comms_server" end
					end
				end
				if group == 8 and verdict.refused then
					local m = lines [1].map
					if tonumber (m.in_cockpit) == 0 and tonumber (m.camera) == 1 and tonumber (m.comms_model) == server then
						return nil, "the weather update was refused although in_cockpit=0, camera=1 and the model is the census comms_server"
					end
				end
				if group == 8 and not verdict.refused then
					for _, g in ipairs (rows_of (lines, "env-update-weather-state-globals")) do
						local m = g.map
						if tonumber (m.in_cockpit) ~= 0 or tonumber (m.camera) ~= 1 or tonumber (m.comms_model) ~= server then
							return nil, "the weather update ran although in_cockpit=" .. m.in_cockpit .. ", camera=" .. m.camera .. ", comms_model=" .. m.comms_model .. " holds a guard"
						end
					end
				end
			end
			if group == 4 then
				local inside = gust_word_in_domain (lines [1].map.gust_w)
				if verdict.refused and inside then return nil, "a gust inside [0,1) was refused" end
				if not verdict.refused and not inside then return nil, "a gust outside [0,1) was reported as called" end
			end
			return verdict
		end
		why [#why + 1] = shape [1] .. ": " .. problem
	end
	return nil, table.concat (why, "; ")
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

	-- one call: the report is parsed, written out as it came, and accepted only with the protocol's shape for this group and index (a rejected report is on the file before the run fails)
	local reports_validated = 0
	local context = {}
	local function case (group, index, scope)
		local name = "env_case(" .. group .. ", " .. index .. ")"
		local ok, report = pcall (engine.env_case, engine, group, index)
		if not ok then refuse (name .. " failed: " .. tostring (report)) end
		local lines, why = parse_report (report)
		if lines == nil then refuse (name .. " returned a malformed report: " .. why) end
		local rows = emit_lines (scope, lines)
		local verdict, reason = validate_report (group, index, lines, context)
		if verdict == nil then refuse (name .. " did not return the fixture's protocol: " .. reason) end
		if group == 0 then context.comms_server = verdict.comms_server end
		reports_validated = reports_validated + 1
		return rows, verdict
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
				-- the shape, the identity, restored=1 and equal fingerprints are all checked by `case`
				local _, verdict = case (g [1], index, "probe:g" .. g [1] .. ":" .. index)
				if verdict.refused then g.refused = g.refused + 1 end
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

	-- the next scheduled snapshot time after `now`, by whole periods in one step however many were passed (one snapshot is taken per frame, whatever the number of periods the frame covered):
	-- the same times as stepping one period at a time (for the default 60 s exactly), in constant work. The result must lie after `now` and be finite, or the run is refused.
	local function advance_periodic (due, every, now)
		local periods = math.floor ((now - due) / every) + 1
		local advanced = due + periods * every
		if not (advanced > now) then advanced = advanced + every end
		if not (advanced > now) or advanced ~= advanced or advanced == math.huge then
			refuse ("snapshot_every=" .. every .. " is too small to schedule after " .. now .. " seconds (the period is below the clock's resolution)")
		end
		return advanced
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
			next_periodic = advance_periodic (next_periodic, snapshot_every, clock.elapsed_seconds)
		end
	end
	if probes > 0 and probe_state == "waiting" then run_probes () end
	snapshot ("snapshot:final")
	campaign_row ("campaign_end")

	for _, g in ipairs (groups) do
		emit { event = "coverage", group = g [1], name = g [2], available = g.count, ran = g.ran, refused = g.refused, truncated = g.ran < g.count }
	end
	local refused_total = 0
	for _, g in ipairs (groups) do refused_total = refused_total + g.refused end
	emit { event = "validation", protocol = "env-fixture/1", reports = reports_validated, witnesses = probes_run, refused_witnesses = refused_total }
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
