-- Original scene TEXTURE ANIMATION LIST characterisation (EEDCS-50, D3). TEST-ONLY: it needs a module built with the `model-fixture` feature (the ORIGINAL loader and accessors, observed
-- by csrc/eech_model_fixture.c and its loader hooks) and the reference host (EECH_WORLD). The shipping module, a `model-host-fixture` module and a module built without the feature have no
-- `model_animations` method: that is NOT RUN, never a zero-success.
--
-- WHAT IT RECORDS. For ONE natural retail boot (the same preparation, reference arithmetic profile, boot configuration and seed 1 as lua/campaign.lua; no frame is run), the FINAL
-- texture animation list of every scene, as the original loader (read_scene, add_animation_object, add_animation) left it: the scene's name, its disposition (`listed`, `empty`, `absent`, or
-- a named unsupported outcome), where the list came from (`file` for a declared list, `built` for a scene replaced by an `.EES`, whose list is built from the animated surfaces of its
-- main object, approximations and sub-objects in that order without repeats), the scene's success flag, the count, the allocation, the entries known to be filled, and the legacy animation
-- ids in list order. The ids and counts are PRIVATE diagnostic data of the original loader, not an identity of the host protocol. Nothing is recomputed here: no list is derived from a
-- surface and no order is assumed; this is what a later semantic host move is compared to, literally.
--
-- THE REPORT IS CHECKED BEFORE IT IS BELIEVED. Each page must be exactly one `model-anim-page` row, `count` `model-anim` rows for scenes first .. first+count-1 in order, and one
-- `model-anim-end` row, in the C grammar (names, field order, decimal integers, bounded ids). Every row must be coherent with its disposition, as the emitter's own states make it (a `listed`
-- row has 1..4096 ids, exactly `count` of them, `filled` equal to `count`, an allocation of at least `count` and, for a declared (`file`) list, exactly `count`; an `empty` row has no count,
-- allocation or filled entry; an `absent` or `unobserved` row has no source and no content; every other outcome is a list the loader made). The page headers must agree with each other and
-- with the rows: the same generation, digest and `scenes_read` on every page, `unknown_scene` 0 on every page (a scene pointer the hooks could not place is an unsupported observer, not a
-- note), and after the last page `scenes_read` equal to the scenes whose rows are not `absent` or `unobserved` (the scenes the loader finished reading). The clock is read before anything is
-- asked and after the last answer: a table with finite `elapsed_seconds` and `time_of_day_seconds` (single-precision range) and a whole `day` (32-bit), equal both times.
-- The digest covers the recorded state and the verified list projection (status, count, filled entries, ids), not every emitted field; the first page is also compared byte for byte when it is
-- read again, and that is what the run relies on for "nothing moved".
-- A malformed report stops the questions at once. A well-formed report with an unsupported outcome (short_read, overflow, unbacked, stale, unobserved, capacity) is written in full and the run
-- fails: the lists are not measured. A run that read no scene, or found no list at all, is not measured either.
--
-- host.args: root (installation root, required), out (the JSON Lines file, required; created exclusively), scenario (lebanon_retail, the default, or georgia_retail)

io.stdout:setvbuf ("no")

local here = debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or ""
local json = dofile (here .. "json-lines.lua")

local args = host.args

local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}

local function fail_args (message) error ("model-animation-characterization: " .. message, 0) end

local known = { root = true, out = true, scenario = true }
for key in pairs (args) do
	if not known [key] then fail_args ("unknown argument " .. tostring (key)) end
end
local root = args.root
if type (root) ~= "string" or root == "" then fail_args ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then fail_args ("out=<JSON Lines file> is required") end
local scenario_name = args.scenario or "lebanon_retail"
local scenario = scenarios [scenario_name]
if not scenario then fail_args ("scenario=lebanon_retail|georgia_retail") end
if type (host.create_exclusive) ~= "function" then fail_args ("this host has no create_exclusive: an output that may never overwrite anything cannot be made safely") end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the report grammar (csrc/eech_model_fixture.c eech_model_animation_run); tests/model_animation_characterization.rs checks these field lists and statuses against the C

local INT_MAX = 2147483647
local IDS_MAX = 4096     -- MF_ANIM_IDS_MAX
local ID_MAX = 65535     -- a surface texture index is an unsigned short
local PAGE_MAX = 96      -- PAGE_SCENES

local PAGE_KEYS = { "first", "count", "scene_last", "generation", "scenes_read", "unknown_scene", "content" }
local ROW_KEYS = { "scene", "name", "status", "source", "succeeded", "count", "capacity", "filled", "ids" }
local END_KEYS = { "first", "count", "rows" }

local SUPPORTED = { listed = true, empty = true, absent = true }
local STATUSES = { listed = true, empty = true, absent = true, short_read = true, overflow = true, unbacked = true, stale = true, unobserved = true, capacity = true }
local SOURCES = { file = true, built = true, none = true }

local function malformed (message) error ({ malformed = message }, 0) end

local function int (text, lo, hi, what)
	if type (text) ~= "string" or #text > 11 or not (text == "0" or string.match (text, "^%-?[1-9]%d*$")) then malformed (what .. " is not a decimal integer: " .. tostring (text)) end
	local number = tonumber (text)
	if number < lo or number > hi then malformed (string.format ("%s %s is outside %d..%d", what, text, lo, hi)) end
	return number
end

-- one row in the exact C grammar: the name, then exactly `keys` in order, one space between words; the raw words are returned (the caller checks each domain)
local function row (line, name, keys)
	local fields = {}
	local n = 0
	if string.sub (line, 1, #name + 1) ~= name .. " " then malformed ("expected a " .. name .. " row, got: " .. tostring (line)) end
	for key, value in string.gmatch (string.sub (line, #name + 1), "%s+([%w_]+)=(%S+)") do
		n = n + 1
		if keys [n] ~= key then malformed (string.format ("%s field %d is %s, expected %s", name, n, key, tostring (keys [n]))) end
		fields [key] = value
	end
	if n ~= #keys then malformed (string.format ("%s has %d fields, expected %d", name, n, #keys)) end
	local rebuilt = name
	for _, key in ipairs (keys) do rebuilt = rebuilt .. " " .. key .. "=" .. fields [key] end
	if rebuilt ~= line then malformed (name .. " row is not in the exact grammar: " .. line) end
	return fields
end

-- the ids of a `listed` row: exactly `count` decimal entries, each 0..65535
local function parse_ids (text, count)
	local ids = {}
	for item in string.gmatch (text .. ",", "([^,]*),") do
		ids [#ids + 1] = int (item, 0, ID_MAX, "an animation id")
	end
	if #ids ~= count then malformed (string.format ("the row lists %d ids and counts %d", #ids, count)) end
	return ids
end

-- one page. `request` is { first, count, scene_last }; `seen` the digest state of the run so far (content, generation, scenes_read) or nil for the first page. Returns the page's rows.
local function check_page (text, request, seen)
	if type (text) ~= "string" or text == "" then malformed ("the page is missing or empty") end
	local lines = {}
	for line in string.gmatch (text, "([^\n]*)\n") do lines [#lines + 1] = line end
	if string.sub (text, -1) ~= "\n" or select (2, string.gsub (text, "\n", "")) ~= #lines then malformed ("the page does not end in a newline") end
	if #lines ~= request.count + 2 then malformed (string.format ("the page has %d lines, expected %d", #lines, request.count + 2)) end

	local header = row (lines [1], "model-anim-page", PAGE_KEYS)
	local page = {
		first = int (header.first, 1, INT_MAX, "page first"), count = int (header.count, 1, PAGE_MAX, "page count"), scene_last = int (header.scene_last, 1, INT_MAX, "scene_last"),
		generation = int (header.generation, 0, 4294967295, "generation"), scenes_read = int (header.scenes_read, 0, INT_MAX, "scenes_read"),
		unknown_scene = int (header.unknown_scene, 0, INT_MAX, "unknown_scene"),
	}
	if not string.match (header.content, "^%x%x%x%x%x%x%x%x%x%x%x%x%x%x%x%x$") or string.match (header.content, "%u") then malformed ("content is not 16 lowercase hexadecimal digits: " .. header.content) end
	page.content = header.content
	if page.first ~= request.first or page.count ~= request.count then malformed ("the page does not echo its request") end
	if page.scene_last ~= request.scene_last then malformed ("the page's scene_last is not the summary's") end
	if page.scenes_read > page.scene_last - 1 then malformed ("scenes_read exceeds the scenes") end
	if page.unknown_scene ~= 0 then malformed (string.format ("the loader hooks saw %d scene pointers outside the scene table", page.unknown_scene)) end
	if seen and (seen.content ~= page.content or seen.generation ~= page.generation or seen.scenes_read ~= page.scenes_read or seen.unknown_scene ~= page.unknown_scene) then
		malformed ("the recorded state changed between pages")
	end

	local rows = {}
	for i = 1, request.count do
		local fields = row (lines [i + 1], "model-anim", ROW_KEYS)
		local r = { scene = int (fields.scene, 1, INT_MAX, "scene"), name = fields.name, status = fields.status, source = fields.source, ids_text = fields.ids }
		if r.scene ~= request.first + i - 1 then malformed (string.format ("row %d is scene %d, expected %d", i, r.scene, request.first + i - 1)) end
		if not string.match (r.name, "^[%w_]+$") then malformed ("scene " .. r.scene .. " has no genuine name: " .. r.name) end
		if not STATUSES [r.status] then malformed ("scene " .. r.scene .. " has an unknown status " .. r.status) end
		if not SOURCES [r.source] then malformed ("scene " .. r.scene .. " has an unknown source " .. r.source) end
		r.succeeded = int (fields.succeeded, 0, 1, "succeeded")
		r.count = int (fields.count, -1, INT_MAX, "count")
		r.capacity = int (fields.capacity, 0, INT_MAX, "capacity")
		r.filled = int (fields.filled, 0, INT_MAX, "filled")
		local at = "scene " .. r.scene .. ": "
		if r.status == "listed" then
			-- the emitter says `listed` only for a list that is the allocated array, within its allocation and the bound, with every entry filled; a declared list is allocated at its count
			if r.source == "none" or r.count < 1 or r.count > IDS_MAX or r.filled ~= r.count or r.capacity < r.count or (r.source == "file" and r.capacity ~= r.count) then
				malformed (at .. "a listed row without the backing its count needs")
			end
			r.ids = parse_ids (fields.ids, r.count)
		else
			if fields.ids ~= "-" then malformed (at .. "only a listed row has ids") end
			if r.status == "empty" and (r.source == "none" or r.count ~= 0 or r.capacity ~= 0 or r.filled ~= 0) then malformed (at .. "an empty row with a count, allocation or filled entry, or no source") end
			if (r.status == "absent" or r.status == "unobserved") and (r.source ~= "none" or r.count ~= 0 or r.capacity ~= 0 or r.filled ~= 0 or r.succeeded ~= 0) then
				malformed (at .. "an " .. r.status .. " row with a source or content")
			end
			if r.status ~= "absent" and r.status ~= "unobserved" and r.source == "none" then malformed (at .. "a " .. r.status .. " row with no source") end
		end
		rows [#rows + 1] = r
	end
	local ending = row (lines [#lines], "model-anim-end", END_KEYS)
	if int (ending.first, 1, INT_MAX, "end first") ~= request.first or int (ending.count, 1, PAGE_MAX, "end count") ~= request.count or int (ending.rows, 0, PAGE_MAX, "end rows") ~= request.count then
		malformed ("the closing row does not echo the page")
	end
	return rows, page
end

-- the clock the engine reports (eech_clock: two single-precision seconds and a 32-bit day): present, numeric, finite and whole where it must be; nothing is rounded here
local F32_MAX = 3.4028234663852886e38

local function check_clock (c, when)
	if type (c) ~= "table" then malformed ("the " .. when .. " clock is not a table") end
	for _, key in ipairs { "elapsed_seconds", "time_of_day_seconds" } do
		local v = c [key]
		if type (v) ~= "number" or v ~= v or v > F32_MAX or v < -F32_MAX then malformed (string.format ("the %s clock's %s is not a finite single-precision number: %s", when, key, tostring (v))) end
	end
	local day = c.day
	if type (day) ~= "number" or day % 1 ~= 0 or day < -2147483648 or day > INT_MAX then malformed (string.format ("the %s clock's day is not a 32-bit whole number: %s", when, tostring (day))) end
	return { elapsed_seconds = c.elapsed_seconds, time_of_day_seconds = c.time_of_day_seconds, day = day }
end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the run. The output is created here and is closed on every exit.

local out_file, why = host.create_exclusive (out_path)
if not out_file then error ("model-animation-characterization: " .. tostring (why), 0) end
local out = json.wrap (out_file, out_path)
local function emit (row_) out.emit (row_) end

local function first_line (message) return (string.match (tostring (message), "^[^\n]*")) end

local function file_size (path)
	local file = io.open (path, "rb")
	if not file then return -1 end
	local size = file:seek ("end")
	file:close ()
	return size or -1
end

local function protocol_failure (err)
	if type (err) == "table" and err.malformed then
		emit { event = "malformed", reason = err.malformed }
		emit { event = "done", complete = false, malformed = true }
		error ("model-animation-characterization: MALFORMED REPORT: " .. err.malformed .. " (the engine is not asked again)", 0)
	end
	error (err, 0)
end

local function run ()
	-- which database is this? The installation is only a retail characterisation when the retail 3D database is there; without it the engine would write a synthetic one with no lists
	local data_directory = string.gsub (root, "\\", "/") .. "/cohokum/3ddata/"
	local textures_pal = file_size (data_directory .. "textures.pal")
	emit { event = "provenance", textures_pal = textures_pal, objdb_bin = file_size (data_directory .. "3dobjdb.bin"), database = textures_pal >= 0 and "retail" or "synthetic" }
	if textures_pal < 0 then
		emit { event = "done", complete = false, malformed = false, not_run = true }
		error ("model-animation-characterization: NOT RUN: no retail 3D database (cohokum/3ddata/textures.pal) under the installation root", 0)
	end

	local dc = require ("eech_dc")
	dc.prepare_installation (root)
	-- the reference host's arithmetic profile, selected right after the installation is prepared and before the boot (src/reference_arithmetic.rs)
	host.use_reference_arithmetic ()

	emit { event = "header", scenario = scenario_name, seed = 1, frames = 0, observer = "model-fixture", note = "natural boot only; no frame is run" }

	local ok, engine = pcall (dc.boot, {
		install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
		seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
		supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, model = host.model, sense = host.sense, aim = host.aim,
	})
	if not ok then
		local message = first_line (engine)
		emit { event = "boot_failed", error = message }
		error ("model-animation-characterization: boot failed: " .. message, 0)
	end
	emit { event = "boot", booted = true }

	if type (engine.model_animations) ~= "function" or type (engine.model_summary) ~= "function" or type (engine.clock) ~= "function" then
		emit { event = "not_run", reason = "the booted module has no model-fixture entries" }
		error ("model-animation-characterization: NOT RUN: this module was not built with the model-fixture feature", 0)
	end

	local function ask (what, call)
		local good, text = pcall (call)
		if not good then
			emit { event = "report_failed", what = what, error = first_line (text) }
			error ("model-animation-characterization: " .. what .. " failed: " .. first_line (text) .. " (the engine is not asked again)", 0)
		end
		return text
	end

	local function clock_of (when)
		local raw = ask ("clock", function () return engine:clock () end)
		local checked, clock = pcall (check_clock, raw, when)
		if not checked then protocol_failure (clock) end
		emit { event = "clock", when = when, elapsed_seconds = clock.elapsed_seconds, time_of_day_seconds = clock.time_of_day_seconds, day = clock.day }
		return clock
	end

	local clock_before = clock_of ("before")
	local summary_text = ask ("model_summary", function () return engine:model_summary () end)
	emit { event = "model_summary", text = tostring (summary_text) }
	local scene_last = type (summary_text) == "string" and string.match (summary_text, "^model%-summary scene_last=(%d+) ")
	scene_last = scene_last and tonumber (scene_last)
	if not scene_last or scene_last < 2 or scene_last > INT_MAX then protocol_failure ({ malformed = "the model summary does not name scene_last" }) end

	local tally = { scenes = 0, ids = 0, by_status = {}, by_source = {} }
	local unsupported = {}
	local seen
	local first = 1
	local first_page_text
	while first < scene_last do
		local count = math.min (PAGE_MAX, scene_last - first)
		local text = ask ("model_animations " .. first, function () return engine:model_animations (first, count) end)
		emit { event = "page", first = first, count = count, text = tostring (text) }
		local checked, rows, page = pcall (check_page, text, { first = first, count = count, scene_last = scene_last }, seen)
		if not checked then protocol_failure (rows) end
		if not seen then
			seen = { content = page.content, generation = page.generation, scenes_read = page.scenes_read, unknown_scene = page.unknown_scene }
			first_page_text = text
		end
		for _, r in ipairs (rows) do
			emit { event = "anim", scene = r.scene, name = r.name, status = r.status, source = r.source, succeeded = r.succeeded, count = r.count, capacity = r.capacity, filled = r.filled, ids = r.ids_text }
			tally.scenes = tally.scenes + 1
			tally.by_status [r.status] = (tally.by_status [r.status] or 0) + 1
			tally.by_source [r.source] = (tally.by_source [r.source] or 0) + 1
			if r.ids then tally.ids = tally.ids + #r.ids end
			if not SUPPORTED [r.status] then unsupported [#unsupported + 1] = string.format ("%d %s %s", r.scene, r.name, r.status) end
		end
		first = first + count
	end
	if tally.scenes ~= scene_last - 1 then protocol_failure ({ malformed = string.format ("%d scene rows for %d scenes", tally.scenes, scene_last - 1) }) end
	-- the headers' count of finished scene reads is the rows': a scene the loader finished is every one whose row is not `absent` (never read) or `unobserved` (a list the hooks did not see)
	local finished = tally.scenes - (tally.by_status.absent or 0) - (tally.by_status.unobserved or 0)
	if finished ~= seen.scenes_read then
		protocol_failure ({ malformed = string.format ("the page headers count %d scenes read and the rows show %d", seen.scenes_read, finished) })
	end

	-- the observer left everything where it was: the first page again (byte for byte), and the clock
	local again = ask ("model_animations 1 (again)", function () return engine:model_animations (1, math.min (PAGE_MAX, scene_last - 1)) end)
	local clock_after = clock_of ("after")
	local unchanged = again == first_page_text and clock_before.elapsed_seconds == clock_after.elapsed_seconds and clock_before.time_of_day_seconds == clock_after.time_of_day_seconds
		and clock_before.day == clock_after.day
	emit { event = "unchanged", pass = unchanged }

	local listed = tally.by_status.listed or 0
	local measured = #unsupported == 0 and seen.scenes_read > 0 and listed > 0 and unchanged
	local statuses = {}
	for status, n in pairs (tally.by_status) do statuses [#statuses + 1] = status .. "=" .. n end
	table.sort (statuses)
	emit { event = "done", complete = measured, malformed = false, scenes = tally.scenes, scenes_read = seen.scenes_read, ids = tally.ids, statuses = table.concat (statuses, " "), unsupported = #unsupported,
		content = seen.content }
	if #unsupported > 0 then error (string.format ("model-animation-characterization: NOT MEASURED: %d scene lists are an unsupported outcome (first: %s)", #unsupported, unsupported [1]), 0) end
	if seen.scenes_read == 0 then error ("model-animation-characterization: NOT MEASURED: the loader hooks saw no scene read", 0) end
	if listed == 0 then error ("model-animation-characterization: NOT MEASURED: no scene has a list, so there is nothing to compare", 0) end
	if not unchanged then error ("model-animation-characterization: the observer's read-only state or the clock changed", 0) end
end

local ran, err = pcall (run)
local closed, close_err = pcall (out.close)
if not ran then
	if not closed then err = tostring (err) .. "; additionally the output could not be closed: " .. first_line (close_err) end
	error (err, 0)
end
if not closed then error ("model-animation-characterization: the output could not be closed: " .. first_line (close_err), 0) end
