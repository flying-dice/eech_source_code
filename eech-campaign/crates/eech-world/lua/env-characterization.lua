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
--   queue                    on | off (default off; needs observer=on): after everything else, APPEND the rows of the lightning strike queue probe (csrc group 13 index 0, ss_envfx.h ENV_FXQ; a module built with the
--                            env-observer feature): what the original lightning routine found and did at each add_3d_lightning_strike. It adds rows after `observer_summary` only (the `lightning_queue_*` events
--                            and the `env-lightning-*`/`env-lq*` rows of scope `queue:report`); with queue=off the driver is exactly as it was, and with it every earlier row is unchanged
--   sound                    on | off (default off; needs observer=on): after the queue rows, APPEND the rows of the sound entity probe (csrc group 14 index 0, ss_envfx.h ENV_FXE): the sound effect entities the ORIGINAL
--                            lightning routine creates (what the real constructor built: sample, lifetime, pitch, position, list membership, playback handle present or absent, the sample table's rate and size) and
--                            how the real server update routine ages, expires and kills them in the frames, with a per-entity summary derived from the rows. Rows only after the queue rows (`sound_entity`,
--                            `sound_summary` and the `env-snd*` rows of scope `sound:report`); with sound=off the driver is exactly as it was
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
local queue_text = args.queue == nil and "off" or args.queue
if queue_text ~= "on" and queue_text ~= "off" then refuse ("queue must be on or off, got '" .. tostring (queue_text) .. "'") end
local queue_on = queue_text == "on"
if queue_on and not observer_on then refuse ("queue=on needs observer=on: the lightning queue probe records only while the observer is armed") end
local sound_text = args.sound == nil and "off" or args.sound
if sound_text ~= "on" and sound_text ~= "off" then refuse ("sound must be on or off, got '" .. tostring (sound_text) .. "'") end
local sound_on = sound_text == "on"
if sound_on and not observer_on then refuse ("sound=on needs observer=on: the sound entity probe records only while the observer is armed") end
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
-- THE OBSERVER STREAM is not one fixture report: Engine::env_observer_events concatenates as many native 32 KiB chunks as the unread events need. Its bound comes from the native source: the array
-- holds ENVOBS_CAPACITY events in all (csrc/eech_env_observer.c), and the widest event line (every integer at its widest: `env-obs seq=4294967295 kind=30 a=-2147483648 b=-2147483648
-- rng=-2147483648 rng_w=ffffffff w0=ffffffff ... w5=ffffffff`) is 161 bytes, so 170 per line is the bound used. Longer rows, more rows, or a longer stream than that are refused, never truncated.
local OBS_CAPACITY = 16384
local OBS_MAX_LINE = 170
local OBS_STREAM_LIMIT = OBS_CAPACITY * (OBS_MAX_LINE + 1) + 1
local clock

local function parse_report (report, limit, max_line)
	if type (report) ~= "string" or report == "" then return nil, "no report" end
	if limit ~= nil then
		if #report >= limit then return nil, "an observer stream of " .. #report .. " bytes exceeds the bound of " .. limit .. " bytes (" .. OBS_CAPACITY .. " events of at most " .. OBS_MAX_LINE .. " bytes)" end
	elseif #report >= REPORT_LIMIT then
		return nil, "a report of " .. #report .. " bytes does not fit the fixture's " .. REPORT_LIMIT .. "-byte buffer"
	end
	if report:sub (-1) ~= "\n" then return nil, "the report is truncated: it does not end with a newline" end
	local lines = {}
	for line in report:gmatch ("([^\n]*)\n") do
		if line == "" then return nil, "the report has an empty line" end
		if max_line ~= nil and #line > max_line then return nil, "an observer row of " .. #line .. " bytes exceeds the " .. max_line .. "-byte bound" end
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
		install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
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

	local function observer_call (method, scope, event, spec, decorate, stream)
		local ok, text = pcall (function () return engine [method] (engine) end)
		if not ok then refuse ("engine:" .. method .. " failed: " .. tostring (text)) end
		if text == "" then return {} end
		-- the event stream is bounded by the native capacity and line width; the report row by the fixture's buffer
		local lines, why = parse_report (text, stream and OBS_STREAM_LIMIT or nil, stream and OBS_MAX_LINE or nil)
		if lines == nil then refuse ("engine:" .. method .. " returned a malformed report: " .. why) end
		if stream and #lines > OBS_CAPACITY then refuse ("engine:" .. method .. " returned " .. #lines .. " events, more than the observer's capacity of " .. OBS_CAPACITY) end
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
	-- frame_end set (the previous-delta phase, recorded not corrected); tod_begin/tod_end pairs and the sound-fact pairs sit inside the frame; a lightning episode is all twelve events in order, after that update's time of day and before its sound facts; frame_end
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
		-- the file's later tags and the session setup options (parsgen.c: the tags are cases of the file loop, AFTER the session block returns; rain tag 0 alone calls fixed weather DRY, any other
		-- value calls nothing; sparm_sc.c may call a fixed weather of its own). A fixed weather straight after a non-zero rain tag cannot be told from the tag's own call by the events, so it is refused:
		-- the real runtime has no rain tag at all.
		local function tag_event ()
			local e = ev [i]
			i = i + 1
			if e.kind == "rain_tag" then
				local after = ev [i]
				local fixed = after ~= nil and after.kind == "fixed_weather"
				if e.a == 0 and not (fixed and after.a == 1) then obs_fail (e, "rain tag 0 must be followed by the fixed weather DRY it calls") end
				if e.a ~= 0 and fixed then obs_fail (after, "rain tag " .. e.a .. " is followed by a fixed weather, which only rain tag 0 calls") end
				if fixed then i = i + 1 end
			elseif e.kind == "fixed_weather" and (e.a < 1 or e.a > 4) then
				obs_fail (e, "fixed weather mode " .. e.a .. " is not one of the four")
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
		weather_episode ()
		take ("parser_session_end")
		while ev [i] ~= nil and ev [i].kind ~= "boot_end" do
			if ev [i].kind == "weather_begin" then
				weather_episode ()
			elseif ev [i].kind == "seed_source" then
				start_episode ()
			elseif ev [i].kind == "rain_tag" or ev [i].kind == "wind_tag" or ev [i].kind == "fixed_weather" then
				tag_event ()
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
		-- a lightning episode is all twelve events in the source's order (the server expiry makes both sound entities unconditionally; a client's shape is not characterized and not accepted)
		if obs.lightning ~= nil and not LIGHTNING_KIND [kind] then
			obs_fail (e, "interrupts the lightning episode after its step " .. obs.lightning)
		end
		if LIGHTNING_KIND [kind] then
			if obs.frame == nil or obs.tod_open or obs.facts_pending then obs_fail (e, "a lightning event outside the frame's weather update") end
			local step = (obs.lightning or 0) + 1
			-- the update's order is time of day, weather/lightning, sound: an episode before the update's time of day, after its sound facts, or a second one in the update is out of place
			if step == 1 then
				if obs.upd == nil or not obs.upd.tod then obs_fail (e, "a lightning episode before the update's time of day") end
				if obs.upd.facts then obs_fail (e, "a lightning episode after the update's sound facts") end
				if obs.upd.lightning then obs_fail (e, "a second lightning episode in one update") end
				obs.upd.lightning = true
			end
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
			if kind == "sound_begin" or kind == "sound_end" then
				-- the ordered pairs the sound probe's creations are matched to (the probe counts a frame at each frame_begin, which is frames_seen + 1 while the frame is open)
				obs.sound_events = obs.sound_events or {}
				obs.sound_events [#obs.sound_events + 1] = { kind = kind, n = e.a, frame = obs.frames_seen + 1, entity = e.w [0] }
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
			obs.clock, obs.last_delta, obs.frame, obs.upd = e.w [0], e.w [1], nil, nil
			obs.frames_seen = obs.frames_seen + 1
		elseif kind == "tod_begin" then
			if obs.frame == nil then obs_fail (e, "a time-of-day update outside a frame") end
			if obs.tod_open then obs_fail (e, "a time-of-day update began inside a time-of-day update") end
			if e.w [1] ~= obs.frame.delta then obs_fail (e, "the update read the delta word " .. string.format ("%08x", e.w [1]) .. ", not the frame's " .. string.format ("%08x", obs.frame.delta)) end
			obs.tod_open = true
			obs.upd = { tod = false, lightning = false, facts = false }
		elseif kind == "tod_end" then
			if not obs.tod_open then obs_fail (e, "a time-of-day update ended that had not begun") end
			obs.tod_open = false
			obs.upd.tod = true
		elseif kind == "sound_facts" then
			if obs.frame == nil or obs.tod_open then obs_fail (e, "sound facts outside a frame's sound update") end
			if e.a == -1 then
				if not obs.facts_pending then obs_fail (e, "the second sound-fact row has no first row") end
				obs.facts_pending = false
				obs.upd.facts = true
			else
				if e.a < 0 or e.a > 1 then obs_fail (e, "in_cockpit is " .. e.a) end
				if obs.upd == nil or not obs.upd.tod then obs_fail (e, "sound facts before the update's time of day") end
				if obs.upd.facts then obs_fail (e, "a second sound-fact pair in one update") end
				obs.facts_pending = true
			end
		else
			obs_fail (e, "is a boot or setup event after the boot ended")
		end
	end

	local BOOT_KINDS = { boot_begin = true, boot_end = true, seed_set = true, seed_source = true, start_reseed = true, start_index = true, start_time = true, start_result = true, session_created = true,
		parser_session_begin = true, parser_session_end = true, rain_tag = true, wind_tag = true, weather_begin = true, weather_draw = true, weather_end = true, fixed_weather = true }

	local function observer_drain (scope)
		local lines = observer_call ("env_observer_events", scope, "env-obs", OBS_EVENT, function (line, t) t.kind_name = OBS_KINDS [tonumber (line.map.kind) or 0] or "unknown" end, true)
		for _, line in ipairs (lines) do
			local m = line.map
			local seq, kind = tonumber (m.seq), tonumber (m.kind)
			if seq >= OBS_CAPACITY then refuse ("observer event " .. m.seq .. " is beyond the native capacity of " .. OBS_CAPACITY .. " events") end
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
		if n ("capacity") > OBS_CAPACITY then refuse ("the observer report has capacity " .. r.capacity .. ", beyond the " .. OBS_CAPACITY .. " the driver's stream bound is derived from") end
		if n ("capacity") < 1 or n ("capacity") < n ("stored") then refuse ("the observer report has capacity " .. r.capacity .. " for " .. r.stored .. " stored events") end
		if n ("stored") ~= obs.next_seq or n ("read") ~= obs.next_seq or n ("next_seq") ~= obs.next_seq then
			refuse ("the observer stored " .. r.stored .. ", read " .. r.read .. " and numbered " .. r.next_seq .. " events, but the driver received " .. obs.next_seq)
		end
		if obs.phase ~= "frames" or (c.boot_begin or 0) ~= 1 or (c.boot_end or 0) ~= 1 then refuse ("the observer record does not hold exactly one complete boot") end
		if obs.frame ~= nil or obs.tod_open or obs.facts_pending or obs.lightning ~= nil then refuse ("the observer record ends inside a frame, an update or an episode") end
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

	-- THE LIGHTNING QUEUE PROBE (queue=on, an addition that changes nothing before it: every row above and every row the driver wrote without it is as it was; this block only APPENDS rows). The
	-- ORIGINAL lightning routine stores one record at each of six points per lightning event in a SEPARATE store (ss_envfx.h ENV_FXQ, csrc/eech_env_observer.c; it is not the event stream and adds no event
	-- to it), and csrc/eech_env_fixture.c group 13 index 0 reports them with the queue as it stands. The report is parsed strictly, written out as it came, and accepted only with this shape: the plan, two
	-- unsupported rows, the report, the records, the occupied slots, in that order; the records complete (six per event, numbered in sequence, point by point), nothing dropped, the occupancy counts agreeing
	-- with the masks, and the events agreeing with the observer's own stream (one lightning_env and two strike_begin per event). What the original did with each add is DERIVED from the masks the records
	-- hold, never assumed: a slot became occupied (queued), nothing changed with no active environment, nothing changed with every slot in use, or nothing changed for a reason the records do not show.
	local LQ_SPECS = {
		["env-lightning-plan"] = compile ("version=1 cases:u main_present:b active:u queue_capacity:i mask_w:w update_calls:n process_isolated=required"),
		["env-lightning-unsupported"] = compile ("item:s reason:s"),
		["env-lq-report"] = compile ("armed:b capacity:u stored:u dropped:u overflow:b lightning_events:u update_calls:n queue_capacity:i final_mask_w:w final_occupancy:u"),
		["env-lq"] = compile ("seq:u point:u n:u active:u main:b time_present:b time_w:w mask_w:w occupancy:u slot:i w0:w w1:w w2:w w3:w w4:w w5:w w6:w w7:w"),
		["env-lq-slot"] = compile ("slot:u w0:w w1:w w2:w w3:w w4:w w5:w w6:w w7:w"),
	}
	local LQ_RANK = { ["env-lightning-plan"] = 1, ["env-lightning-unsupported"] = 2, ["env-lq-report"] = 3, ["env-lq"] = 4, ["env-lq-slot"] = 5 }
	local function lq_bit (word, i) return math.floor (word / 2 ^ i) % 2 end
	local function lq_popcount (word)
		local count = 0
		for i = 0, 31 do count = count + lq_bit (word, i) end
		return count
	end
	local function queue_probe ()
		local ok, report = pcall (engine.env_case, engine, 13, 0)
		if not ok then refuse ("env_case(13, 0) failed: " .. tostring (report)) end
		local lines, why = parse_report (report)
		if lines == nil then refuse ("env_case(13, 0) returned a malformed report: " .. why) end
		emit_lines ("queue:report", lines)
		local counts, rank, records, slot_rows = {}, 0, {}, {}
		for _, line in ipairs (lines) do
			local spec, r = LQ_SPECS [line.event], LQ_RANK [line.event]
			if spec == nil then refuse ("env_case(13, 0) returned the unexpected row '" .. line.event .. "'") end
			local problem = spec_error (line, spec)
			if problem ~= nil then refuse ("env_case(13, 0) row " .. line.event .. " " .. problem) end
			if r < rank then refuse ("env_case(13, 0) row " .. line.event .. " is out of order") end
			rank = r
			counts [line.event] = (counts [line.event] or 0) + 1
			if line.event == "env-lq" then records [#records + 1] = line.map elseif line.event == "env-lq-slot" then slot_rows [#slot_rows + 1] = line.map end
		end
		if counts ["env-lightning-plan"] ~= 1 or counts ["env-lightning-unsupported"] ~= 2 or counts ["env-lq-report"] ~= 1 then refuse ("env_case(13, 0) did not return exactly one plan, two unsupported rows and one report") end
		local plan, rep
		for _, line in ipairs (lines) do
			if line.event == "env-lightning-plan" then plan = line.map elseif line.event == "env-lq-report" then rep = line.map end
		end
		local events, capacity = tonumber (rep.lightning_events), tonumber (rep.queue_capacity)
		if rep.armed ~= "1" then refuse ("the lightning queue probe says it is not armed") end
		if rep.dropped ~= "0" or rep.overflow ~= "0" then refuse ("the lightning queue probe dropped " .. rep.dropped .. " records (capacity " .. rep.capacity .. "): the record is incomplete") end
		if capacity < 1 or capacity > 32 then refuse ("the lightning queue has " .. rep.queue_capacity .. " slots: the accessor did not answer") end
		if tonumber (rep.stored) ~= #records or #records ~= 6 * events then refuse ("the probe stored " .. rep.stored .. " records for " .. events .. " lightning events: six per event are expected") end
		if plan.update_calls ~= rep.update_calls or plan.mask_w ~= rep.final_mask_w or plan.queue_capacity ~= rep.queue_capacity then refuse ("the plan and the report describe different queues") end
		local final_mask = tonumber (rep.final_mask_w, 16)
		if lq_popcount (final_mask) ~= tonumber (rep.final_occupancy) or #slot_rows ~= tonumber (rep.final_occupancy) then
			refuse ("the final queue has occupancy " .. rep.final_occupancy .. " and mask " .. rep.final_mask_w .. " but " .. #slot_rows .. " occupied slot rows")
		end
		-- the occupied slots are the final mask exactly: in range, in use, strictly ascending (so unique), as many as the mask has bits
		local previous_slot = -1
		for _, s in ipairs (slot_rows) do
			local slot = tonumber (s.slot)
			if slot >= capacity or tonumber (s.w0, 16) == 0 or lq_bit (final_mask, slot) ~= 1 then refuse ("occupied slot row " .. s.slot .. " is not an occupied slot of the final mask " .. rep.final_mask_w) end
			if slot <= previous_slot then refuse ("occupied slot row " .. s.slot .. " repeats or precedes slot " .. previous_slot .. ": the rows are the occupied slots once each, in ascending order") end
			previous_slot = slot
		end
		if tonumber (plan.cases) ~= 5 or tonumber (plan.active) > 2 then refuse ("the plan has " .. plan.cases .. " cases and the active environment category " .. plan.active .. ": the fixture has five controls (the four original cases and the forked-age control 5) and three categories") end
		if (obs.counts.lightning_env or 0) ~= events or (obs.counts.strike_begin or 0) ~= 2 * events or (obs.counts.strike_end or 0) ~= 2 * events then
			refuse ("the probe holds " .. events .. " lightning events but the observer stream holds " .. (obs.counts.lightning_env or 0) .. " environment switches and " .. (obs.counts.strike_begin or 0) .. " strike calls")
		end
		local tally = { queued = 0, no_active_environment = 0, queue_full = 0, unexplained = 0 }
		-- the facts the C records satisfy by construction, checked as the shape of a coherent state view (none of them says what an add does): the active category is 0 (none), 1 (the main one) or 2 (another); the active
		-- time word exists exactly when an environment is active; the main environment either exists in every record or in none, and exists when it is the active one; the point's slot and words are -1 and zeros
		-- except after an add that occupied a slot, whose slot is then in use. The masks of points with no add between them are equal (entry and first before; first after and second before; second after and
		-- restored); a later event can only have lost slots since the previous restore (no other caller adds); the final queue is the last restored mask exactly if the update never ran, and a subset of it otherwise.
		local main_seen, previous_restored_mask, last_restored_mask
		for ev = 1, events do
			local base = (ev - 1) * 6
			local function rec (point)
				local r = records [base + point + 1]
				if tonumber (r.seq) ~= base + point or tonumber (r.point) ~= point or tonumber (r.n) ~= ev then
					refuse ("probe record " .. (base + point) .. " is not point " .. point .. " of lightning event " .. ev .. " (seq " .. r.seq .. ", point " .. r.point .. ", n " .. r.n .. ")")
				end
				local mask = tonumber (r.mask_w, 16)
				if lq_popcount (mask) ~= tonumber (r.occupancy) then refuse ("probe record " .. r.seq .. " has occupancy " .. r.occupancy .. " for the mask " .. r.mask_w) end
				local active, main, time_present, slot = tonumber (r.active), tonumber (r.main), tonumber (r.time_present), tonumber (r.slot)
				if active > 2 then refuse ("probe record " .. r.seq .. " has the active environment category " .. r.active .. ", which is not none, the main one or another") end
				if time_present ~= (active ~= 0 and 1 or 0) then refuse ("probe record " .. r.seq .. " has time_present " .. r.time_present .. " with the active category " .. r.active) end
				if time_present == 0 and r.time_w ~= "00000000" then refuse ("probe record " .. r.seq .. " has a time word without an active environment") end
				if active == 1 and main ~= 1 then refuse ("probe record " .. r.seq .. " has the main environment active but no main environment") end
				if main_seen ~= nil and main ~= main_seen then refuse ("probe record " .. r.seq .. " disagrees with the others on whether the main environment exists") end
				main_seen = main
				local words_zero = true
				for w = 0, 7 do
					if tonumber (r ["w" .. w], 16) ~= 0 then words_zero = false end
				end
				if point == 2 or point == 4 then
					if slot >= capacity or slot < -1 then refuse ("probe record " .. r.seq .. " names slot " .. r.slot .. " of a queue of " .. capacity) end
					if slot >= 0 and tonumber (r.w0, 16) == 0 then refuse ("probe record " .. r.seq .. " reports slot " .. r.slot .. " occupied by the add but its captured slot is not in use (w0 = 0)") end
					if slot < 0 and not words_zero then refuse ("probe record " .. r.seq .. " names no slot but carries slot words") end
				elseif slot ~= -1 or not words_zero then
					refuse ("probe record " .. r.seq .. " is point " .. point .. ", which follows no add, but names a slot or carries slot words")
				end
				return r, mask
			end
			local entry, entry_mask = rec (0)
			if previous_restored_mask ~= nil then
				for i = 0, 31 do
					if lq_bit (entry_mask, i) == 1 and lq_bit (previous_restored_mask, i) == 0 then refuse ("lightning event " .. ev .. " begins with slot " .. i .. " occupied that the previous event's restored queue did not hold: an add no record shows") end
				end
			end
			local before_1, restored
			local carried_mask = entry_mask
			for k = 1, 2 do
				local before, mask_before = rec (2 * k - 1)
				local after, mask_after = rec (2 * k)
				if mask_before ~= carried_mask then refuse ("lightning event " .. ev .. ": the queue before add " .. k .. " (" .. before.mask_w .. ") is not the queue the previous point left (" .. string.format ("%08x", carried_mask) .. "), and no add lies between") end
				carried_mask = mask_after
				if k == 1 then before_1 = before end
				local new_bits, kept, new_slot = 0, true, -1
				for i = 0, 31 do
					local b, a = lq_bit (mask_before, i), lq_bit (mask_after, i)
					if b == 1 and a == 0 then kept = false end
					if b == 0 and a == 1 then
						new_bits = new_bits + 1
						if new_slot < 0 then new_slot = i end
					end
				end
				local outcome
				if not kept then
					outcome = "unexplained"
				elseif new_bits == 1 then
					outcome = "queued"
				elseif new_bits == 0 and before.active == "0" then
					outcome = "no_active_environment"
				elseif new_bits == 0 and lq_popcount (mask_before) >= capacity then
					outcome = "queue_full"
				else
					outcome = "unexplained"
				end
				if tonumber (after.slot) ~= new_slot then refuse ("probe record " .. after.seq .. " names slot " .. after.slot .. " but the masks show slot " .. new_slot) end
				tally [outcome] = tally [outcome] + 1
				local row = { event = "lightning_queue_outcome", lightning = ev, call = k, outcome = outcome, active_before = tonumber (before.active), main_present = tonumber (before.main), time_present = tonumber (before.time_present),
					time_w = before.time_w, mask_before_w = before.mask_w, mask_after_w = after.mask_w, occupancy_after = tonumber (after.occupancy), slot = new_slot }
				if outcome == "queued" then
					for w = 0, 7 do row ["w" .. w] = after ["w" .. w] end
				end
				emit (row)
			end
			local restored_mask
			restored, restored_mask = rec (5)
			if restored_mask ~= carried_mask then refuse ("lightning event " .. ev .. ": the restored queue (" .. restored.mask_w .. ") is not the queue after the second add (" .. string.format ("%08x", carried_mask) .. "), and no add lies between") end
			previous_restored_mask, last_restored_mask = restored_mask, restored_mask
			emit { event = "lightning_queue_environment", lightning = ev, entry_active = tonumber (entry.active), entry_main_present = tonumber (entry.main), before_first_add_active = tonumber (before_1.active),
				restored_active = tonumber (restored.active), switched_to_main = entry.active ~= "1" and before_1.active == "1", restored_to_entry = restored.active == entry.active, entry_mask_w = entry.mask_w }
		end
		if main_seen ~= nil and main_seen ~= tonumber (plan.main_present) then refuse ("the records say the main environment " .. (main_seen == 1 and "exists" or "is absent") .. " but the plan says main_present " .. plan.main_present) end
		if last_restored_mask ~= nil then
			for i = 0, 31 do
				if lq_bit (final_mask, i) == 1 and lq_bit (last_restored_mask, i) == 0 then refuse ("the final queue holds slot " .. i .. " that the last event's restored queue did not: an add no record shows") end
			end
			if tonumber (rep.update_calls) == 0 and final_mask ~= last_restored_mask then refuse ("the update never ran, so the final queue (" .. rep.final_mask_w .. ") must be the last event's restored queue (" .. string.format ("%08x", last_restored_mask) .. ")") end
		end
		emit { event = "lightning_queue_summary", lightning_events = events, records = #records, queued = tally.queued, no_active_environment = tally.no_active_environment, queue_full = tally.queue_full,
			unexplained = tally.unexplained, update_calls = tonumber (rep.update_calls), final_occupancy = tonumber (rep.final_occupancy), queue_capacity = capacity, main_present = tonumber (plan.main_present) }
	end

	-- THE SOUND ENTITY PROBE (sound=on, an addition that changes nothing before it: this block only APPENDS rows after the queue rows). The ORIGINAL code stores a record when the real constructor has built
	-- each sound effect entity of the lightning routine and at each call of the real update routines on it, in a SEPARATE store (ss_envfx.h ENV_FXE, csrc/eech_env_observer.c; not the event stream), and
	-- csrc/eech_env_fixture.c group 14 index 0 reports them. The report is parsed strictly, written out as it came and accepted only with this shape: the plan, the facts of the sound database and
	-- the sound system, two unsupported rows, the report, the records; nothing dropped, the sequence unbroken, every tracked entity followed from its creation through its updates to at most one
	-- expiry branch and one kill, and the creations agreeing with the observer's own stream (one sound_begin and one sound_end for each). What the original DID with the entities is the content of the rows
	-- and of the per-entity summaries derived from them; no outcome is expected.
	local SND_SPECS = {
		["env-snd-plan"] = compile ("version=1 cases:u process_isolated=required"),
		["env-snd-facts"] = compile ("tag=plan comms:i application_sound_system_enabled:i sound_system_initialised:i sound_system_paused:i samples:i lightning_rate:i lightning_size:i lightning_original_rate:i thunder_rate:i thunder_size:i thunder_original_rate:i"),
		["env-snd-unsupported"] = compile ("item:s reason:s"),
		["env-snd-report"] = compile ("armed:b capacity:u stored:u dropped:u overflow:b creations:u updates:u kills:u frames:u tracked:i open:u track_all:b untracked:u"),
		["env-snd"] = compile ("seq:u point:u n:u tracked:i frame:u upd:u entity:i etype:i sub:i channel:i locality:i valid:i looping:i panning:i seqcount:i handle:i session_parent:b update_member:b sample:i rate:i size:i orate:i life_w:w vlife_w:w amp_w:w pitch_w:w px_w:w py_w:w pz_w:w delta_w:w"),
	}
	local SND_RANK = { ["env-snd-plan"] = 1, ["env-snd-facts"] = 2, ["env-snd-unsupported"] = 3, ["env-snd-report"] = 4, ["env-snd"] = 5 }
	-- the next points a tracked entity may show after a point: 0 created; 1 server enter; 2 / 3 / 4 expired (no handle / idle handle / playing handle); 5 killed; 6 client enter; 7 client exit
	local SND_NEXT = { [0] = { [1] = true, [6] = true }, [1] = { [1] = true, [2] = true, [3] = true, [4] = true }, [2] = { [5] = true }, [3] = { [5] = true }, [4] = { [1] = true, [3] = true, [4] = true }, [5] = {}, [6] = { [7] = true }, [7] = { [6] = true } }
	local function sound_probe ()
		local ok, report = pcall (engine.env_case, engine, 14, 0)
		if not ok then refuse ("env_case(14, 0) failed: " .. tostring (report)) end
		local lines, why = parse_report (report)
		if lines == nil then refuse ("env_case(14, 0) returned a malformed report: " .. why) end
		emit_lines ("sound:report", lines)
		local counts, rank, records, rep = {}, 0, {}, nil
		for _, line in ipairs (lines) do
			local spec, r = SND_SPECS [line.event], SND_RANK [line.event]
			if spec == nil then refuse ("env_case(14, 0) returned the unexpected row '" .. line.event .. "'") end
			local problem = spec_error (line, spec)
			if problem ~= nil then refuse ("env_case(14, 0) row " .. line.event .. " " .. problem) end
			if r < rank then refuse ("env_case(14, 0) row " .. line.event .. " is out of order") end
			rank = r
			counts [line.event] = (counts [line.event] or 0) + 1
			if line.event == "env-snd" then records [#records + 1] = line.map elseif line.event == "env-snd-report" then rep = line.map end
		end
		if counts ["env-snd-plan"] ~= 1 or counts ["env-snd-facts"] ~= 1 or counts ["env-snd-unsupported"] ~= 2 or counts ["env-snd-report"] ~= 1 then
			refuse ("env_case(14, 0) did not return exactly one plan, one facts row, two unsupported rows and one report")
		end
		if rep.armed ~= "1" then refuse ("the sound entity probe says it is not armed") end
		if rep.dropped ~= "0" or rep.overflow ~= "0" then refuse ("the sound entity probe dropped " .. rep.dropped .. " records (capacity " .. rep.capacity .. "): the record is incomplete") end
		if tonumber (rep.stored) ~= #records then refuse ("the sound entity probe stored " .. rep.stored .. " records but reported " .. #records) end
		local creations, updates, kills = tonumber (rep.creations), tonumber (rep.updates), tonumber (rep.kills)
		if (obs.counts.sound_begin or 0) ~= creations or (obs.counts.sound_end or 0) ~= creations then
			refuse ("the probe holds " .. creations .. " sound creations but the observer stream holds " .. (obs.counts.sound_begin or 0) .. " sound_begin and " .. (obs.counts.sound_end or 0) .. " sound_end events")
		end
		-- the declared sampling of the native probe (csrc/eech_env_observer.c ENVE_ENTERS_STORED): the first three updates of an entity are stored whole, later ones are only counted by `upd`
		local ENTERS_STORED = 3
		local untracked = tonumber (rep.untracked)
		local events = obs.sound_events or {}
		local created, chains, untracked_seen = 0, {}, 0
		for n, r in ipairs (records) do
			if tonumber (r.seq) ~= n - 1 then refuse ("sound probe record " .. n - 1 .. " has seq " .. r.seq) end
			local point, tracked, upd = tonumber (r.point), tonumber (r.tracked), tonumber (r.upd)
			if point > 7 then refuse ("sound probe record " .. r.seq .. " has the point " .. r.point) end
			if n > 1 and tonumber (r.frame) < tonumber (records [n - 1].frame) then refuse ("sound probe record " .. r.seq .. " goes back in frames") end
			if point == 0 then
				created = created + 1
				if upd ~= 0 then refuse ("sound probe creation record " .. r.seq .. " has the update count " .. r.upd) end
				if tracked >= 0 and chains [tracked] ~= nil then refuse ("sound probe record " .. r.seq .. " creates the tracked entity " .. tracked .. " twice") end
				if tonumber (r.n) < 1 or tonumber (r.n) > 2 then refuse ("sound probe record " .. r.seq .. " is a creation outside the lightning routine's two sound entities (n " .. r.n .. ")") end
				-- the creation is the one the observer's own ordered stream brackets: the created-th sound_begin / sound_end pair, with the same ordinal in the routine, in the same frame, and (the constructor takes the
				-- first free entity slot, se_creat.c get_free_entity with ENTITY_INDEX_DONT_CARE, which is what get_next_free_entity_index () read before it) the entity that was next to be given. The helper
				-- create_client_server_sound_effect_entity has no branch that skips the constructor, so each pair holds exactly one creation record.
				local begin_event, end_event = events [2 * created - 1], events [2 * created]
				if begin_event == nil or end_event == nil or begin_event.kind ~= "sound_begin" or end_event.kind ~= "sound_end" then
					refuse ("sound probe creation " .. created .. " (record " .. r.seq .. ") has no sound_begin / sound_end pair in the observer's stream")
				end
				if begin_event.n ~= tonumber (r.n) or end_event.n ~= begin_event.n then refuse ("sound probe creation " .. created .. " is the lightning routine's sound " .. r.n .. " but the observer's pair is sound " .. begin_event.n .. " / " .. end_event.n) end
				if begin_event.frame ~= tonumber (r.frame) or end_event.frame ~= begin_event.frame then refuse ("sound probe creation " .. created .. " is in frame " .. r.frame .. " but the observer's pair is in frame " .. begin_event.frame .. " / " .. end_event.frame) end
				if begin_event.entity ~= tonumber (r.entity) then refuse ("sound probe creation " .. created .. " is entity " .. r.entity .. " but the observer saw entity " .. begin_event.entity .. " next to be given") end
				if tracked >= 0 then
					chains [tracked] = { entity = r.entity, last = 0, n = tonumber (r.n), n_text = r.n, created_frame = tonumber (r.frame), first_life_w = r.life_w, last_life_w = r.life_w, enters = 0, upd = 0, expiry = "none", killed = false, handle_ever = r.handle ~= "0" }
				else
					untracked_seen = untracked_seen + 1
				end
			else
				local chain = chains [tracked]
				if tracked < 0 or chain == nil then refuse ("sound probe record " .. r.seq .. " (point " .. point .. ") follows no tracked creation") end
				if chain.entity ~= r.entity then refuse ("sound probe record " .. r.seq .. " is of entity " .. r.entity .. " but tracked entity " .. tracked .. " is " .. chain.entity) end
				if r.n ~= chain.n_text then refuse ("sound probe record " .. r.seq .. " has n " .. r.n .. " where its creation has " .. chain.n_text) end
				if not SND_NEXT [chain.last] [point] then refuse ("sound probe record " .. r.seq .. ": point " .. point .. " cannot follow point " .. chain.last .. " of tracked entity " .. tracked) end
				-- the update counter: the j-th stored entry into an update is update j (only the first ENTERS_STORED are stored); an exit carries its update's count; an expiry and a kill carry the count of the update they end
				if point == 1 or point == 6 then
					chain.enters = chain.enters + 1
					if chain.enters > ENTERS_STORED or upd ~= chain.enters then refuse ("sound probe record " .. r.seq .. " is update " .. upd .. " where it is the " .. chain.enters .. "th stored entry of an update of tracked entity " .. tracked .. " (only the first " .. ENTERS_STORED .. " are stored)") end
				elseif point == 7 then
					if upd ~= chain.upd then refuse ("sound probe record " .. r.seq .. " leaves update " .. upd .. " that began as update " .. chain.upd .. " for tracked entity " .. tracked) end
				else
					-- an expiry or a kill belongs to the update that is running. Until all ENTERS_STORED entries have been stored no update has been left out, so it carries the CURRENT stored entry count (the count may run ahead only
					-- after the last mandatory entry, where the later updates are only counted); a kill carries the update of the expiry straight before it. This is bookkeeping coherence, not an expected lifetime.
					if chain.enters < 1 then refuse ("sound probe record " .. r.seq .. " ends an update that no record began for tracked entity " .. tracked) end
					if point == 5 and upd ~= chain.upd then refuse ("sound probe record " .. r.seq .. " is a kill in update " .. upd .. " but the expiry straight before it is in update " .. chain.upd .. " for tracked entity " .. tracked) end
					if upd < chain.upd then refuse ("sound probe record " .. r.seq .. " shows the update count going down from " .. chain.upd .. " to " .. upd .. " for tracked entity " .. tracked) end
					if chain.enters < ENTERS_STORED and upd ~= chain.enters then refuse ("sound probe record " .. r.seq .. " is in update " .. upd .. " but only " .. chain.enters .. " of the " .. ENTERS_STORED .. " mandatory entries have been stored for tracked entity " .. tracked .. ": the running update is the " .. chain.enters .. "th") end
				end
				chain.upd = upd
				chain.last = point
				if point == 2 or point == 3 or point == 4 then chain.expiry = ({ [2] = "no_handle", [3] = "idle_handle", [4] = "playing_handle" }) [point] end
				if point == 5 then chain.killed = true end
				if point ~= 5 then chain.last_life_w = r.life_w end
				if r.handle ~= "0" then chain.handle_ever = true end
			end
		end
		if created ~= creations then refuse ("the probe says " .. creations .. " creations but holds " .. created .. " creation records") end
		if untracked_seen ~= untracked then refuse ("the probe says it lost track of " .. untracked .. " creations but holds " .. untracked_seen .. " creation records without a tracked entity") end
		if tonumber (rep.tracked) + untracked ~= creations then refuse ("the probe tracked " .. rep.tracked .. " entities and lost " .. untracked .. " of " .. creations .. " creations") end
		local entities, killed, expired, sum_last, inexact = 0, 0, 0, 0, 0
		for tracked = 0, tonumber (rep.tracked) - 1 do
			local chain = chains [tracked]
			if chain == nil then refuse ("tracked entity " .. tracked .. " has no creation record") end
			-- an operation that began is finished: an expiry with no or an idle handle is followed by its kill, and an entry into the client routine by its exit
			if chain.last == 2 or chain.last == 3 then refuse ("tracked entity " .. tracked .. " ends at an expiry (point " .. chain.last .. ") with no kill record after it") end
			if chain.last == 6 then refuse ("tracked entity " .. tracked .. " ends inside the client routine: its exit record is missing") end
			entities = entities + 1
			if chain.killed then killed = killed + 1 end
			sum_last = sum_last + chain.upd
			-- the count at the last record is the entity's whole count if it was killed, or if fewer updates than are stored have run (a further one would have been stored); otherwise it is a lower bound
			local exact = chain.killed or chain.upd < ENTERS_STORED
			if not exact then inexact = inexact + 1 end
			if chain.expiry ~= "none" then expired = expired + 1 end
			emit { event = "sound_entity", tracked = tracked, n = chain.n, entity = tonumber (chain.entity), created_frame = chain.created_frame, updates_at_last_record = chain.upd, updates_exact = exact, expiry = chain.expiry, killed = chain.killed, handle_ever = chain.handle_ever,
				first_life_w = chain.first_life_w, last_life_w = chain.last_life_w }
		end
		if sum_last > updates then refuse ("the tracked entities show " .. sum_last .. " updates at their last records but the probe counted only " .. updates) end
		if inexact == 0 and sum_last ~= updates then refuse ("every tracked entity's update count is exact, they sum to " .. sum_last .. " but the probe counted " .. updates) end
		if killed ~= kills then refuse ("the probe counted " .. kills .. " kills but the records show " .. killed) end
		if untracked > 0 then
			emit { event = "sound_untracked", creations = untracked, reason = "the probe's tracking table was full: these creations are recorded as creations, but their updates, expiry and kills are not followed, so their lifetimes are NOT characterised" }
		end
		emit { event = "sound_summary", creations = creations, updates = updates, kills = kills, entities = entities, expired = expired, records = #records, frames = tonumber (rep.frames), untracked_creations = untracked, lifetimes_complete = untracked == 0 }
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
	if queue_on then queue_probe () end
	if sound_on then sound_probe () end

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
