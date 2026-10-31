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
--   observer                 on | off (default off): arm the read-only observer of the original environment BEFORE boot (needs a module built with the env-observer feature; refused otherwise)
--                            and record the boot and the frames' events in the file. Requires probes=0 (the witnesses call original routines on synthetic state and would mix into the record)
--   frame_schedule           instead of frame_ms: 1 to 16 comma-separated frame lengths in ms (each 1 to 1000), cycled (e.g. 100,250,50), so the previous-delta phase can be seen
--   controlled               none (default) | lightning_due | local_weather_on: ONE explicitly labelled controlled-weather case of the original setters (csrc group 9), right after the initial snapshots,
--                            for a fresh process. It changes the live session and restores nothing (the process is the isolation boundary); it needs probes=0 and is not a default run
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
local observer_text = args.observer == nil and "off" or args.observer
if observer_text ~= "on" and observer_text ~= "off" then refuse ("observer must be on or off, got '" .. tostring (observer_text) .. "'") end
local observer_on = observer_text == "on"
if observer_on and probes > 0 then refuse ("observer=on needs probes=0: the witnesses call original routines on synthetic state and would mix into the natural record") end
local controlled_cases = { lightning_due = 0, local_weather_on = 1 }
local controlled_name = args.controlled
local controlled_index
if controlled_name ~= nil and controlled_name ~= "none" then
	controlled_index = controlled_cases [controlled_name]
	if controlled_index == nil then refuse ("controlled must be none, lightning_due or local_weather_on, got '" .. tostring (controlled_name) .. "'") end
	if probes > 0 then refuse ("controlled=" .. controlled_name .. " changes the session and restores nothing: it needs probes=0 and a process of its own") end
end
-- the frame lengths: one (frame_ms) or a cycled schedule; a schedule is not combined with frame_ms
local schedule = { frame_ms }
if args.frame_schedule ~= nil then
	if args.frame_ms ~= nil then refuse ("frame_ms and frame_schedule are alternatives: give one") end
	local text = tostring (args.frame_schedule)
	if #text > 80 then refuse ("frame_schedule is too long: at most 80 characters") end
	schedule = {}
	for token in (text .. ","):gmatch ("([^,]*),") do
		if not token:match ("^%d+$") or #token > 4 or tonumber (token) < 1 or tonumber (token) > 1000 then refuse ("frame_schedule must be 1 to 16 integers from 1 to 1000 separated by commas, got '" .. text .. "'") end
		schedule [#schedule + 1] = tonumber (token)
	end
	if #schedule < 1 or #schedule > 16 then refuse ("frame_schedule must name 1 to 16 frame lengths, got '" .. text .. "'") end
end
local frames = math.ceil (seconds * 1000 / frame_ms)
if args.frame_schedule ~= nil then
	frames = 0
	local elapsed_ms = 0
	while elapsed_ms < seconds * 1000 and frames <= MAX_FRAMES do
		frames = frames + 1
		elapsed_ms = elapsed_ms + schedule [(frames - 1) % #schedule + 1]
	end
end
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
	if string_keys [key] or key:match ("_w$") or key:match ("^w%d$") then return value end
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
local function emit_lines (scope, lines, decorate)
	local rows = {}
	for _, line in ipairs (lines) do
		local t = { event = line.event, scope = scope, at = clock.elapsed_seconds }
		for _, kv in ipairs (line.fields) do
			t [reserved [kv [1]] and ("field_" .. kv [1]) or kv [1]] = field_value (kv [1], kv [2])
		end
		if decorate ~= nil then decorate (line, t) end
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
	n = { "a counter", function (v) return #v <= 18 and v:match ("^%d+$") ~= nil end },
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

-- one original query at one point: the weather answer and either the wind answer or (where the printed state can leave the gust domain: the retail snapshot and the initialisers, which leave the gust phase untouched; the context guard in validate_report judges which one is right) the stated absence of it
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
		local function tail () return { state_rows ("env-init-state", witness), { query_row ("env-init-query", witness, "origin", true), query_row ("env-init-query", witness, "map_centre", true) } } end
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
	if group == 9 then
		local head = "kind=controlled i:u what=lightning_due|local_weather_on"
		return {
			{ "controlled", { row ("env-controlled", head .. " local_before:b radius_before_w:w timer_before_w:w local_after:b radius_after_w:w timer_after_w:w"), row ("env-controlled-end", "group=9 index:u changed=1 process_isolated=required") }, false },
			{ "controlled_refused_server", { row ("env-controlled", head .. " refused=not_the_server comms_model:i"), row ("env-controlled-end", "group=9 index:u changed=0 process_isolated=required") }, true },
			{ "controlled_refused_weather", { row ("env-controlled", head .. " refused=weather_not_local_or_radius_zero local_weather_model:b weather_radius_w:w"), row ("env-controlled-end", "group=9 index:u changed=0 process_isolated=required") }, true },
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
			if group == 9 then
				-- a controlled case restores nothing and says so; its row is judged only by the inputs it echoes and by the guard it printed
				local c, last = lines [1].map, lines [#lines].map
				if tonumber (last.index) ~= index then return nil, "the controlled end marker is for index " .. last.index end
				if c.what ~= (index == 0 and "lightning_due" or "local_weather_on") then return nil, "the controlled row is for what=" .. c.what .. ", not case " .. index end
				local zero = { ["00000000"] = true, ["80000000"] = true }
				if verdict.shape == "controlled_refused_server" and tonumber (c.comms_model) == context.comms_server then return nil, "the controlled case was refused as not the server although comms_model equals the census comms_server" end
				if verdict.shape == "controlled_refused_weather" and not (index == 0 and (c.local_weather_model == "0" or zero [c.weather_radius_w])) then return nil, "the controlled case was refused as not local weather although the printed weather is local with a non-zero radius (or the case is not 0)" end
				if not verdict.refused then
					if c.timer_after_w ~= "3f000000" then return nil, "the lightning timer written is " .. c.timer_after_w .. ", not the 0.5 s input" end
					if index == 0 and (c.local_before ~= "1" or zero [c.radius_before_w]) then return nil, "case 0 ran although the weather was not local with a non-zero radius" end
					if index == 1 and (c.local_after ~= "1" or c.radius_after_w ~= "471c4000") then return nil, "case 1 did not leave the local model with the 40 km input radius" end
				end
			end
			if group >= 2 and group <= 8 then
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
	local boot_row = { event = "boot", scenario = scenario_name, seed = seed, seconds = seconds, frame_ms = frame_ms, probes = probes, probe_at = probe_at, snapshot_frames = snapshot_frames,
		snapshot_every = snapshot_every, mode = probes == 0 and "control" or "probe", feature = "env-fixture" }
	-- the new arguments appear only when given, so a run without them writes the boot row it always wrote
	if observer_on then boot_row.observer = "on" end
	if args.frame_schedule ~= nil then boot_row.frame_schedule = tostring (args.frame_schedule) end
	if controlled_index ~= nil then boot_row.controlled = controlled_name end
	emit (boot_row)

	dc = require ("eech_dc")
	dc.prepare_installation (root)
	if observer_on then
		-- the genuine refusal for a module without the observer; the observer is armed BEFORE boot, so the random start time, the reseed and the ten weather draws are in the record
		if type (dc.env_observer_arm) ~= "function" then
			emit { event = "refusal", reason = "env_observer_unavailable", detail = "the module was built without the env-observer feature" }
			refuse ("the module has no env_observer_arm function: build eech-dc with the env-observer feature")
		end
		dc.env_observer_arm (true)
	end
	engine = dc.boot {
		install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
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
	local function step_frame (ms)
		local before = clock.elapsed_seconds
		frames_run = frames_run + 1
		if frames_run > MAX_FRAMES + 2 then refuse ("more than " .. MAX_FRAMES .. " frames were run") end
		engine:frame (ms)
		clock = engine:clock ()
		if not (clock.elapsed_seconds > before) then refuse ("the clock did not advance in a frame of " .. ms .. " ms") end
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

	-- THE OBSERVER (armed before boot; csrc/eech_env_observer.c). Its events are drained in order, written as they came (with the name of the kind added), and checked: the row shape, an unbroken
	-- sequence from the boot, the shared seed word and its signed form being one word, frames that begin and end in pairs. At the end the report must say armed, nothing dropped, everything read, and
	-- the frame and update counts the driver and the original's own counters give must agree. An observer that dropped events, or was not armed before boot, is refused, never reported as measured.
	local OBS_KINDS = { "boot_begin", "boot_end", "seed_set", "seed_source", "start_reseed", "start_index", "start_time", "start_result", "session_created", "parser_session_begin", "parser_session_end",
		"rain_tag", "wind_tag", "weather_begin", "weather_draw", "weather_end", "fixed_weather", "frame_begin", "frame_end", "tod_begin", "tod_end", "lightning_expired", "lightning_new_timer",
		"lightning_timer_set", "lightning_env", "strike_begin", "strike_end", "sound_begin", "sound_end", "sound_facts" }
	local obs = { next_seq = 0, counts = {}, phase = "pre", boot = {}, frames_seen = 0, clock = nil, last_delta = nil, frame = nil, tod_open = false, lightning = nil, facts_pending = false }
	local OBS_EVENT = compile ("seq:u kind:u a:i b:i rng:i rng_w:w w0:w w1:w w2:w w3:w w4:w w5:w")
	local OBS_REPORT = compile ("armed:b capacity:u stored:u read:u dropped:u overflow:b next_seq:u update_server:n lightning_calls:n sound_calls:n resync_calls:n weather_calls:n sound_records:u")

	local function observer_call (method, scope, event, spec, decorate)
		local ok, text = pcall (function () return engine [method] (engine) end)
		if not ok then refuse ("engine:" .. method .. " failed: " .. tostring (text)) end
		if text == "" then return {} end
		local lines, why = parse_report (text)
		if lines == nil then refuse ("engine:" .. method .. " returned a malformed report: " .. why) end
		emit_lines (scope, lines, decorate)
		for _, line in ipairs (lines) do
			if line.event ~= event then refuse ("engine:" .. method .. " returned the row '" .. line.event .. "' where '" .. event .. "' was expected") end
			local problem = spec_error (line, spec)
			if problem ~= nil then refuse ("engine:" .. method .. " row " .. event .. " " .. problem) end
		end
		return lines
	end

	-- THE TRANSCRIPT GRAMMAR. The events are checked as the source orders them, never against an expected physical result. BOOT (boot_begin ... boot_end): the engine's own reseed (seed_source then
	-- seed_set from it, 0 becoming 1), the boot seed set, the campaign loader's session (parser_session_begin ... parser_session_end: a start-time episode, session_created carrying that start time, a
	-- weather episode of draws 1 to 10), then ANY number of further weather and start-time episodes (the session setup options run them again; the real boot has two of each) and the optional tag and
	-- fixed-weather events. FRAMES (each boot_end-ordered, none before it): frame_begin asks the scheduled ms at the clock the previous frame (or boot) ended on and the delta word the previous
	-- frame_end set (the previous-delta phase, recorded not corrected); tod_begin/tod_end pairs and the sound-fact pairs sit inside the frame; a lightning episode is its twelve events in order; frame_end
	-- is begin + ms on the clock. Anything else, anywhere else, is refused.
	local function obs_fail (e, message) refuse ("observer transcript, event " .. e.seq .. " (" .. e.kind .. "): " .. message) end

	local function check_boot (ev)
		local i = 1
		local function fail (message) obs_fail (ev [i] or ev [#ev], message) end
		local function take (kind)
			local e = ev [i]
			if e == nil or e.kind ~= kind then fail ("expected " .. kind .. ", found " .. (e and e.kind or "the end of the boot")) end
			i = i + 1
			return e
		end
		local function skip_free ()
			while ev [i] ~= nil and (ev [i].kind == "rain_tag" or ev [i].kind == "wind_tag" or ev [i].kind == "fixed_weather") do
				if ev [i].kind == "fixed_weather" and (ev [i].a < 1 or ev [i].a > 4) then fail ("fixed weather mode " .. ev [i].a .. " is not one of the four") end
				i = i + 1
			end
		end
		-- initialise_random_number_generator: the clock value it seeded from, then the seed set from it (zero becomes one)
		local function reseed ()
			local source = take ("seed_source")
			local set = take ("seed_set")
			if set.a ~= (source.a == 0 and 1 or source.a) then fail ("seed_set a=" .. set.a .. " does not follow the seed_source a=" .. source.a) end
			return set
		end
		local function start_episode ()
			local set = reseed ()
			local reseeded = take ("start_reseed")
			if reseeded.rng ~= set.a then fail ("the word after the reseed is " .. reseeded.rng .. ", not the seed " .. set.a) end
			local index = take ("start_index")
			if index.a < 0 or index.a > 5 then fail ("the start-time table index " .. index.a .. " is outside the six entries") end
			local drawn = take ("start_time")
			local result = take ("start_result")
			if result.rng ~= drawn.rng then fail ("the seed word changed between start_time and start_result") end
			return result
		end
		local function weather_episode ()
			take ("weather_begin")
			local last
			for ordinal = 1, 10 do
				local draw = take ("weather_draw")
				if draw.a ~= ordinal then fail ("weather draw " .. draw.a .. " where draw " .. ordinal .. " was expected") end
				-- draws 2 and 7 are the two flag draws (the value 0 or 1); the others carry no flag
				if draw.b < 0 or draw.b > ((ordinal == 2 or ordinal == 7) and 1 or 0) then fail ("weather draw " .. ordinal .. " carries the flag " .. draw.b) end
				last = draw
			end
			local finish = take ("weather_end")
			if finish.rng ~= last.rng then fail ("the weather episode ended on another seed word than its tenth draw") end
		end

		local begin = take ("boot_begin")
		if begin.a ~= seed then fail ("boot_begin carries seed " .. begin.a .. ", the run was given " .. seed) end
		reseed ()
		local engine_seed = take ("seed_set")
		if engine_seed.a ~= begin.a then fail ("the engine's own seed_set is " .. engine_seed.a .. ", not the boot seed " .. begin.a) end
		take ("parser_session_begin")
		local start = start_episode ()
		local created = take ("session_created")
		if created.w [0] ~= start.w [0] then fail ("session_created carries the start time " .. string.format ("%08x", created.w [0]) .. ", not the start_result " .. string.format ("%08x", start.w [0])) end
		if created.a < 0 or created.a > 1 or created.b < 0 or created.b > 3 then fail ("session_created carries the weather model " .. created.a .. " and the day segment " .. created.b) end
		skip_free ()
		weather_episode ()
		skip_free ()
		take ("parser_session_end")
		while ev [i] ~= nil and ev [i].kind ~= "boot_end" do
			skip_free ()
			if ev [i] == nil or ev [i].kind == "boot_end" then break end
			if ev [i].kind == "weather_begin" then
				weather_episode ()
			elseif ev [i].kind == "seed_source" then
				start_episode ()
			else
				fail ("the boot has " .. ev [i].kind .. " after the session was created")
			end
		end
		local finish = take ("boot_end")
		if finish.w [0] < begin.w [0] then fail ("the clock went backwards across the boot") end
		return finish
	end

	local LIGHTNING_ORDER = { { "lightning_expired" }, { "lightning_new_timer" }, { "lightning_timer_set" }, { "lightning_env" }, { "strike_begin", 1 }, { "strike_end", 1 }, { "strike_begin", 2 },
		{ "strike_end", 2 }, { "sound_begin", 1 }, { "sound_end", 1 }, { "sound_begin", 2 }, { "sound_end", 2 } }
	local LIGHTNING_KIND = { lightning_expired = true, lightning_new_timer = true, lightning_timer_set = true, lightning_env = true, strike_begin = true, strike_end = true, sound_begin = true, sound_end = true }

	local function frame_event (e)
		local kind = e.kind
		-- a lightning episode is its events in the source's order; without the server's sound entities it ends after the second strike
		if obs.lightning ~= nil and not LIGHTNING_KIND [kind] then
			if obs.lightning ~= 8 then obs_fail (e, "interrupts the lightning episode after its step " .. obs.lightning) end
			obs.lightning = nil
		end
		if LIGHTNING_KIND [kind] then
			if obs.frame == nil or obs.tod_open or obs.facts_pending then obs_fail (e, "a lightning event outside the frame's weather update") end
			local step = (obs.lightning or 0) + 1
			local want = LIGHTNING_ORDER [step]
			if want == nil or kind ~= want [1] or (want [2] ~= nil and e.a ~= want [2]) then obs_fail (e, "the lightning episode expected " .. (want and want [1] or "nothing more") .. " at step " .. step) end
			if step == 1 then
				if e.a ~= 1 then obs_fail (e, "the lightning timer expired in a branch other than the server's") end
				obs.expired = (obs.expired or 0) + 1
			elseif kind == "lightning_timer_set" then
				if e.w [0] ~= obs.lightning_timer then obs_fail (e, "the timer set is not the timer drawn") end
			elseif kind == "lightning_env" and (e.a < 0 or e.a > 1) then
				obs_fail (e, "the active-environment flag is " .. e.a)
			elseif kind == "strike_begin" then
				if e.a == 1 then obs.strike = { e.w [0], e.w [1] } elseif e.w [0] ~= obs.strike [1] or e.w [1] ~= obs.strike [2] then obs_fail (e, "the second strike is not at the first strike's position") end
			end
			if kind == "lightning_new_timer" then obs.lightning_timer = e.w [0] end
			obs.lightning = step
			if step == #LIGHTNING_ORDER then obs.lightning = nil end
			return
		end
		if obs.facts_pending and not (kind == "sound_facts" and e.a == -1) then obs_fail (e, "the sound-fact pair was not completed") end
		if kind == "frame_begin" then
			if obs.frame ~= nil then obs_fail (e, "a frame began inside a frame") end
			local ms = schedule [obs.frames_seen % #schedule + 1]
			if e.a ~= ms then obs_fail (e, "frame_begin asks " .. e.a .. " ms where the schedule says " .. ms) end
			if e.b < 0 then obs_fail (e, "the time acceleration count is " .. e.b) end
			if e.w [0] ~= obs.clock then obs_fail (e, "the frame began at clock " .. e.w [0] .. " ms, where the previous frame (or the boot) ended at " .. tostring (obs.clock)) end
			if obs.last_delta ~= nil and e.w [1] ~= obs.last_delta then obs_fail (e, "the frame began with the delta word " .. string.format ("%08x", e.w [1]) .. ", not the previous delta " .. string.format ("%08x", obs.last_delta)) end
			obs.frame = { ms = ms, clock = e.w [0], delta = e.w [1] }
		elseif kind == "frame_end" then
			if obs.frame == nil then obs_fail (e, "a frame ended that had not begun") end
			if obs.tod_open then obs_fail (e, "the frame ended inside a time-of-day update") end
			if e.w [0] ~= (obs.frame.clock + obs.frame.ms) % 4294967296 then obs_fail (e, "the clock advanced to " .. e.w [0] .. " ms, not " .. obs.frame.clock .. " + " .. obs.frame.ms) end
			obs.clock, obs.last_delta, obs.frame = e.w [0], e.w [1], nil
			obs.frames_seen = obs.frames_seen + 1
		elseif kind == "tod_begin" then
			if obs.frame == nil then obs_fail (e, "a time-of-day update outside a frame") end
			if obs.tod_open then obs_fail (e, "a time-of-day update began inside a time-of-day update") end
			if e.w [1] ~= obs.frame.delta then obs_fail (e, "the update read the delta word " .. string.format ("%08x", e.w [1]) .. ", not the frame's " .. string.format ("%08x", obs.frame.delta)) end
			obs.tod_open = true
		elseif kind == "tod_end" then
			if not obs.tod_open then obs_fail (e, "a time-of-day update ended that had not begun") end
			obs.tod_open = false
		elseif kind == "sound_facts" then
			if obs.frame == nil or obs.tod_open then obs_fail (e, "sound facts outside a frame's sound update") end
			if e.a == -1 then
				if not obs.facts_pending then obs_fail (e, "the second sound-fact row has no first row") end
				obs.facts_pending = false
			else
				if e.a < 0 or e.a > 1 then obs_fail (e, "in_cockpit is " .. e.a) end
				obs.facts_pending = true
			end
		else
			obs_fail (e, "is a boot or setup event after the boot ended")
		end
	end

	local BOOT_KINDS = { boot_begin = true, boot_end = true, seed_set = true, seed_source = true, start_reseed = true, start_index = true, start_time = true, start_result = true, session_created = true,
		parser_session_begin = true, parser_session_end = true, rain_tag = true, wind_tag = true, weather_begin = true, weather_draw = true, weather_end = true, fixed_weather = true }

	local function observer_drain (scope)
		local lines = observer_call ("env_observer_events", scope, "env-obs", OBS_EVENT, function (line, t) t.kind_name = OBS_KINDS [tonumber (line.map.kind) or 0] or "unknown" end)
		for _, line in ipairs (lines) do
			local m = line.map
			local seq, kind = tonumber (m.seq), tonumber (m.kind)
			if seq ~= obs.next_seq then refuse ("observer event " .. m.seq .. " follows " .. (obs.next_seq - 1) .. ": an event is missing or repeated") end
			if kind < 1 or kind > #OBS_KINDS then refuse ("observer event " .. m.seq .. " has kind " .. m.kind .. ", which is not one of the " .. #OBS_KINDS .. " kinds") end
			if (tonumber (m.rng) % 4294967296) ~= tonumber (m.rng_w, 16) then refuse ("observer event " .. m.seq .. ": rng " .. m.rng .. " and rng_w " .. m.rng_w .. " are not one word") end
			if seq == 0 and kind ~= 1 then refuse ("the first observer event is kind " .. m.kind .. ", not boot_begin: the observer was not armed before boot") end
			local name = OBS_KINDS [kind]
			local e = { seq = seq, kind = name, a = tonumber (m.a), b = tonumber (m.b), rng = tonumber (m.rng), w = {} }
			for k = 0, 5 do e.w [k] = tonumber (m ["w" .. k], 16) end
			obs.counts [name] = (obs.counts [name] or 0) + 1
			obs.next_seq = seq + 1
			if obs.phase == "pre" then obs.phase = "boot" end
			if obs.phase == "boot" then
				if not BOOT_KINDS [name] then obs_fail (e, "is a frame event before boot_end") end
				obs.boot [#obs.boot + 1] = e
				if name == "boot_end" then
					obs.clock = check_boot (obs.boot).w [0]
					obs.phase = "frames"
				end
			else
				frame_event (e)
			end
		end
	end

	local function observer_finish ()
		observer_drain ("observer:final")
		local lines = observer_call ("env_observer_report", "observer:report", "env-obs-report", OBS_REPORT)
		if #lines ~= 1 then refuse ("the observer report has " .. #lines .. " rows, not 1") end
		local r, c = lines [1].map, obs.counts
		local function n (key) return tonumber (r [key]) end
		if r.armed ~= "1" then refuse ("the observer report says it is not armed") end
		if r.dropped ~= "0" or r.overflow ~= "0" then refuse ("the observer dropped " .. r.dropped .. " events (capacity " .. r.capacity .. "): the record is incomplete") end
		if n ("capacity") < 1 or n ("capacity") < n ("stored") then refuse ("the observer report has capacity " .. r.capacity .. " for " .. r.stored .. " stored events") end
		if n ("stored") ~= obs.next_seq or n ("read") ~= obs.next_seq or n ("next_seq") ~= obs.next_seq then
			refuse ("the observer stored " .. r.stored .. ", read " .. r.read .. " and numbered " .. r.next_seq .. " events, but the driver received " .. obs.next_seq)
		end
		if obs.phase ~= "frames" or (c.boot_begin or 0) ~= 1 or (c.boot_end or 0) ~= 1 then refuse ("the observer record does not hold exactly one complete boot") end
		if obs.frame ~= nil or obs.tod_open or obs.facts_pending or (obs.lightning ~= nil and obs.lightning ~= 8) then refuse ("the observer record ends inside a frame, an update or an episode") end
		if obs.frames_seen ~= frames_run or (c.frame_begin or 0) ~= frames_run or (c.frame_end or 0) ~= frames_run then
			refuse ("the observer saw " .. (c.frame_begin or 0) .. " frame_begin and " .. (c.frame_end or 0) .. " frame_end events for " .. frames_run .. " frames the driver ran")
		end
		if (c.tod_begin or 0) ~= (c.tod_end or 0) or (c.tod_begin or 0) ~= n ("update_server") then
			refuse ("the observer saw " .. (c.tod_begin or 0) .. " tod_begin and " .. (c.tod_end or 0) .. " tod_end events for " .. r.update_server .. " session updates")
		end
		-- the per-frame routines the server update calls unconditionally are called once per update; the lightning routine only in the local model; the resync only when it fires
		if n ("weather_calls") ~= n ("update_server") or n ("sound_calls") ~= n ("update_server") then
			refuse ("the observer counted " .. r.weather_calls .. " weather and " .. r.sound_calls .. " sound updates for " .. r.update_server .. " session updates")
		end
		if n ("lightning_calls") > n ("update_server") or n ("resync_calls") > n ("update_server") or (obs.expired or 0) > n ("lightning_calls") then
			refuse ("the observer's lightning (" .. r.lightning_calls .. ", expiries " .. (obs.expired or 0) .. ") or resync (" .. r.resync_calls .. ") counts exceed the session updates")
		end
		if (c.sound_facts or 0) ~= 2 * n ("sound_records") then refuse ("the observer counted " .. r.sound_records .. " sound records but " .. (c.sound_facts or 0) .. " sound-fact rows") end
		local summary = { event = "observer_summary", events = obs.next_seq, update_server = tonumber (r.update_server), lightning_calls = tonumber (r.lightning_calls), sound_calls = tonumber (r.sound_calls),
			resync_fired = tonumber (r.resync_calls), weather_calls = tonumber (r.weather_calls), sound_records = tonumber (r.sound_records) }
		for name, count in pairs (c) do summary ["n_" .. name] = count end
		emit (summary)
	end

	if observer_on then
		for _, method in ipairs ({ "env_observer_events", "env_observer_report" }) do
			local present, fn = pcall (function () return engine [method] end)
			if not present or type (fn) ~= "function" then
				emit { event = "refusal", reason = "env_observer_unavailable", detail = "the engine has no " .. method .. " method" }
				refuse ("the engine has no " .. method .. " method: build eech-dc with the env-observer feature")
			end
		end
		observer_drain ("observer:boot")
	end

	campaign_row ("campaign:initial")
	snapshot ("snapshot:initial")
	if controlled_index ~= nil then
		-- ONE controlled-weather case of the original setters, after the unmodified initial snapshots; it restores nothing and says so
		case (9, controlled_index, "controlled:" .. controlled_name)
		snapshot ("snapshot:controlled")
	end
	if probes > 0 then probe_state = "waiting" end
	local next_periodic = snapshot_every > 0 and snapshot_every or nil
	for n = 1, frames do
		if probes > 0 and probe_state == "waiting" and clock.elapsed_seconds >= probe_at then run_probes () end
		step_frame (schedule [(n - 1) % #schedule + 1])
		if observer_on and n % 200 == 0 then observer_drain ("observer:frame:" .. n) end
		if n <= snapshot_frames then snapshot ("snapshot:frame:" .. n) end
		if next_periodic ~= nil and clock.elapsed_seconds >= next_periodic then
			snapshot ("snapshot:periodic:" .. string.format ("%.3f", clock.elapsed_seconds))
			next_periodic = advance_periodic (next_periodic, snapshot_every, clock.elapsed_seconds)
		end
	end
	if probes > 0 and probe_state == "waiting" then run_probes () end
	snapshot ("snapshot:final")
	campaign_row ("campaign_end")
	if observer_on then observer_finish () end

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
