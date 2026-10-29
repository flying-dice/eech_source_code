-- TEST-ONLY original collision-model characterisation (EEDCS-50). Needs a DLL built with the `model-fixture` feature; the shipping module has no `model_*`
-- method and this script stops at the first call.
--
-- It boots the campaign through the public path (require ("eech_dc")) with the reference host's six callbacks, as every other scenario does, and then ASKS, never
-- advances: no frame is stepped and nothing is written to the campaign. Through the fixture it reads, from the ORIGINAL compiled model code the retail data loaded:
--   * a finite catalogue of every scene: the original scene accessors' bounds and radius (as float bit patterns), the effective collision-object index, the
--     collision object's loaded metadata, the safety verdict of the original decoder (safe / refused with a reason / absent) and how many loaded entities use the scene;
--   * for the scenes named in `scenes=`, the original collision-triangle accessor called once per depth, ascending, from depth 0 until it first returns false (or
--     the safety verdict's bound, or `max_triangles`), reported with the buffer's partial-write facts. A scene the verdict refuses is reported refused; the original
--     is not called for it. A model with more triangles than `max_triangles` is reported `capped`, which is NOT a full capture.
-- Before and after, the fixture hashes every table it reads and the script compares them, with the clock; a difference fails the run.
-- Nothing here intersects, decodes or counts a triangle; the answers are the original's.
--
-- Every reply is also checked as a REPORT before it is believed (shape, identity, order, cardinality, completion; never geometry): a catalogue page must echo its request and
-- hold exactly its scenes once, in order, each with the metadata the fixture always writes; a geometry page must echo its request, keep one model's identity from page to page,
-- hold contiguous depths with well-formed return/write/coordinate fields, and end in a terminal row that names the same scene, the depth reached and a reason the rows support
-- (`original_false` only straight after an original call that returned false), with the reported call count equal to the rows. A reply that fails is written (its rows are
-- already in the output), a `report-refused` row says why, and the run stops with an error BEFORE any `done`. Genuine explicit outcomes (an absent or refused model, an
-- unbacked end probe, a capped walk) are accepted as such: this never makes an unsupported model succeed, and `fan_bound` stays safety information, never an expected answer.
--
-- host.args: root, scenario (lebanon_retail default, georgia_retail, georgia), out (JSON Lines path; refused if it exists),
--            catalogue (1 default, 0 to skip), scenes (comma-separated scene numbers from a catalogue, optional), max_triangles (default 4096, at most 65536)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local out_path = assert (args.out, "out=<JSON Lines path>")

local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
	georgia = { map = "..\\common\\maps\\map16", campaign = "georgia.chc" },
}
local scenario_name = args.scenario or "lebanon_retail"
local scenario = assert (scenarios [scenario_name], "scenario=lebanon_retail|georgia_retail|georgia")

-- argument checks, before the engine is booted or a file is touched
local function whole (text, name, min, max)
	local number = tonumber (text)
	if not number or number ~= number or number == math.huge or number == -math.huge or number % 1 ~= 0 or number < min or number > max then
		error (string.format ("%s must be a whole number within %d..%d, got %s", name, min, max, tostring (text)), 0)
	end
	return number
end
local catalogue = whole (args.catalogue or "1", "catalogue", 0, 1) == 1
local max_triangles = whole (args.max_triangles or "4096", "max_triangles", 1, 65536)
local scenes, seen = {}, {}
if args.scenes and args.scenes ~= "" then
	for item in string.gmatch (args.scenes, "[^,]+") do
		local scene = whole (item, "scenes item", 1, 2147483646)
		if seen [scene] then error ("scenes names " .. scene .. " twice", 0) end
		seen [scene] = true
		scenes [#scenes + 1] = scene
	end
	if #scenes > 64 then error ("scenes names more than 64 scenes", 0) end
end
if not catalogue and #scenes == 0 then error ("catalogue=0 with no scenes asks nothing", 0) end

-- the output: never overwrites, every write checked, closed and checked
do
	local existing = io.open (out_path, "r")
	if existing then
		existing:close ()
		error ("the output file already exists and is not overwritten", 0)
	end
end
local out = assert (io.open (out_path, "w"))

local function emit (t)
	local parts = {}
	for k, v in pairs (t) do
		local value = type (v) == "string" and string.format ("%q", v) or tostring (v)
		parts [#parts + 1] = string.format ("%q:%s", k, value)
	end
	table.sort (parts)
	assert (out:write ("{" .. table.concat (parts, ",") .. "}\n"))
	assert (out:flush ())
end

-- one JSON object per report line: the first word is the event, `key=value` words are its fields; whole decimal values are numbers, everything else
-- (names, hex bit patterns, lists) is text
local string_keys = { tables = true, pointers = true, content = true } -- 16-hex-digit hashes: text even when every digit is decimal
local function parse_report (text, extra)
	local rows = {}
	for line in string.gmatch (text, "[^\n]+") do
		local event, rest = string.match (line, "^(%S+)%s*(.*)$")
		local row = { event = event }
		for k, v in string.gmatch (rest, "(%S+)=(%S+)") do
			if row [k] ~= nil then row.duplicate_key = k end -- the report names a field twice: the validators refuse it
			row [k] = (not string_keys [k] and string.match (v, "^-?%d+$")) and tonumber (v) or v
		end
		for k, v in pairs (extra or {}) do row [k] = v end
		rows [#rows + 1] = row
	end
	return rows
end
local function emit_report (text, extra)
	local rows = parse_report (text, extra)
	for _, row in ipairs (rows) do emit (row) end
	return rows
end

local failures = 0
local function check (name, ok, detail)
	emit { event = "check", name = name, pass = ok and "true" or "false", detail = detail or "" }
	if not ok then failures = failures + 1 end
end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- report validation: the replies are checked as reports (see the header), then believed

-- a reply that does not hold up: say so in the output, close it, and stop before any success
local function refuse (what, reason)
	emit { event = "report-refused", what = what, reason = reason }
	assert (out:close ())
	io.stderr:write ("model-characterization: report refused: " .. what .. ": " .. reason .. "\n")
	error ("report refused: " .. what .. ": " .. reason, 0)
end

local function reply (what, text)
	if type (text) ~= "string" or text == "" then refuse (what, "missing or empty reply") end
	return text
end

local function whole_number (v) return type (v) == "number" and v % 1 == 0 end
local function hex32 (v) return type (v) == "string" and string.match (v, "^0x%x%x%x%x%x%x%x%x$") ~= nil end
local function hex32_list (v, n)
	if type (v) ~= "string" then return false end
	local count = 0
	for item in string.gmatch (v .. ",", "([^,]*),") do
		if not hex32 (item) then return false end
		count = count + 1
	end
	return count == n
end

-- the verdicts the fixture can write (eech_model_fixture.c): a name outside this set is a malformed report, not a new kind of model
local statuses = {
	absent = true, no_scene = true, invalid_index = true, null_surfaces = true, surfaces_outside_block = true, custom_object = true, face_mismatch = true,
	zero_face_surface = true, null_arrays = true, degenerate_face = true, capacity_unobservable = true, capacity_short = true, index_violation = true,
	empty = true, safe = true,
}
local function probeable (status) return status == "safe" or status == "empty" end
local SENTINEL = "0x7fc0a5a5" -- the word the fixture pre-fills each triangle buffer with

local function validate_summary (rows)
	local s = rows [1]
	if #rows ~= 1 or not s or s.event ~= "model-summary" or s.duplicate_key then refuse ("summary", "not exactly one well-formed model-summary row") end
	for _, key in ipairs { "scene_last", "raw_objects", "surface_block", "page_scenes", "page_triangles" } do
		if not whole_number (s [key]) then refuse ("summary", key .. " is missing or not a whole number") end
	end
	if s.scene_last < 2 or s.page_scenes < 1 or s.page_triangles < 1 then refuse ("summary", "scene_last or a page size is out of range") end
	return s
end

local function validate_state (rows, phase)
	local s = rows [1]
	if #rows ~= 1 or not s or s.event ~= "model-state" or s.duplicate_key then refuse ("state " .. phase, "not exactly one well-formed model-state row") end
	for _, key in ipairs { "tables", "pointers", "content" } do
		if type (s [key]) ~= "string" or not string.match (s [key], "^%x%x%x%x%x%x%x%x%x%x%x%x%x%x%x%x$") then refuse ("state " .. phase, key .. " is not a 16-digit hash") end
	end
	if not whole_number (s.content_objects) then refuse ("state " .. phase, "content_objects is not a whole number") end
end

local function validate_clock (clock, phase)
	if type (clock) ~= "table" or type (clock.elapsed_seconds) ~= "number" or type (clock.time_of_day_seconds) ~= "number" or type (clock.day) ~= "number" then
		refuse ("clock " .. phase, "the clock is missing a field")
	end
end

-- one catalogue page: the request echoed, then exactly `count` scene rows, scene `first` .. `first + count - 1`, each once and in order, each with the metadata the fixture
-- always writes. Returns how many scenes were counted (the statuses are tallied by the caller).
local function validate_catalogue_page (rows, first, count, scene_last)
	local what = "catalogue page " .. first
	if #rows == 0 then refuse (what, "empty reply") end
	local page = rows [1]
	if page.event ~= "model-page" or page.duplicate_key or page.first ~= first or page.count ~= count or page.scene_last ~= scene_last then
		refuse (what, "the page header does not echo the request (first, count, scene_last)")
	end
	if not whole_number (page.entities_scanned) or not whole_number (page.entities_with_shape) or page.entities_with_shape > page.entities_scanned then
		refuse (what, "the entity counts are missing or inconsistent")
	end
	if #rows ~= count + 1 then refuse (what, "expected " .. count .. " scene rows, found " .. (#rows - 1)) end
	for i = 1, count do
		local row, scene = rows [i + 1], first + i - 1
		local function bad (reason) refuse (what, "scene row " .. scene .. ": " .. reason) end
		if row.event ~= "model-scene" or row.duplicate_key then bad ("not a well-formed model-scene row") end
		if row.scene ~= scene then bad ("wrong or out-of-order scene identity " .. tostring (row.scene)) end
		if type (row.name) ~= "string" or row.name == "" or type (row.detail) ~= "string" or type (row.use_types) ~= "string" then bad ("name, detail or use_types is missing") end
		for _, key in ipairs { "main", "succeeded", "collision", "uses", "use_other" } do
			if not whole_number (row [key]) then bad (key .. " is missing or not a whole number") end
		end
		if not statuses [row.status] then bad ("unknown status " .. tostring (row.status)) end
		if not hex32 (row.radius) or not hex32_list (row.bb2, 6) or not hex32_list (row.bb, 6) then bad ("radius or bounds are not float bit patterns") end
		if (row.status == "absent") ~= (row.collision == -1) then bad ("status and collision index contradict each other") end
		if (row.uses == 0) ~= (row.use_types == "-") then bad ("uses and use_types contradict each other") end
		if probeable (row.status) then
			if row.collision < 1 or not whole_number (row.fan_bound) or row.fan_bound < 0 or not whole_number (row.max_depth) or row.max_depth < 0
				or (row.end_read_backed ~= 0 and row.end_read_backed ~= 1) then
				bad ("a callable model lacks its collision index, fan_bound, end_read_backed or max_depth")
			end
		end
	end
	return count
end

-- one geometry page for `scene` from depth `first`, `count` asked. `known` carries the model's identity from its earlier pages. Returns { finished, calls, outcome }.
local function validate_geometry_page (rows, scene, first, count, known)
	local what = "geometry scene " .. scene .. " depth " .. first
	local function bad (reason) refuse (what, reason) end
	if #rows == 0 then bad ("empty reply") end
	local header = rows [1]
	if header.event ~= "model-geometry" or header.duplicate_key then bad ("the first row is not a well-formed model-geometry header") end
	if header.scene ~= scene or header.first ~= first or header.count ~= count then bad ("the header does not echo the request (scene, first, count)") end
	if type (header.name) ~= "string" or header.name == "" or type (header.detail) ~= "string" or not statuses [header.status] then bad ("the header lacks a name, detail or known status") end
	for _, key in ipairs { "collision", "fan_bound" } do
		if not whole_number (header [key]) then bad ("the header's " .. key .. " is not a whole number") end
	end
	if header.fan_bound < 0 or (header.end_read_backed ~= 0 and header.end_read_backed ~= 1) then bad ("the header's fan_bound or end_read_backed is out of range") end
	if (header.status == "absent") ~= (header.collision == -1) then bad ("status and collision index contradict each other") end
	if probeable (header.status) and header.collision < 1 then bad ("a callable model without a collision object") end
	if known.header then
		for _, key in ipairs { "name", "collision", "status", "detail", "fan_bound", "end_read_backed" } do
			if known.header [key] ~= header [key] then bad ("the model's " .. key .. " changed between pages") end
		end
	else
		known.header = header
	end

	if header.status == "absent" then
		-- the original answers the absence itself, once, at the requested depth
		local t = rows [2]
		if #rows ~= 2 or t.event ~= "model-tri" or t.duplicate_key or t.depth ~= first or t["return"] ~= 0 or t.writes ~= 0 or t.p ~= nil then
			bad ("an absent model must be exactly one false, writeless original call at the requested depth")
		end
		return { finished = true, calls = 1, outcome = "absent" }
	end
	if not probeable (header.status) then
		local r = rows [2]
		if #rows ~= 2 or r.event ~= "model-refused" or r.duplicate_key or r.scene ~= scene or r.status ~= header.status or r.detail ~= header.detail then
			bad ("a refused model must be exactly one model-refused row naming the same scene, status and detail (and no original call)")
		end
		return { finished = true, calls = 0, outcome = "refused" }
	end

	local max_depth = (header.end_read_backed == 1) and header.fan_bound or (header.fan_bound - 1)
	local depth, calls, terminal = first, 0, nil
	local n = #rows
	for i = 2, n do
		local row = rows [i]
		if row.duplicate_key then bad ("row " .. i .. " names a field twice") end
		if row.event == "model-tri" then
			if terminal or i == n then bad ("row " .. i .. ": a triangle row after the end, or with no closing row") end
			if calls >= count then bad ("more triangle rows than were asked for") end
			if row.depth ~= depth then bad ("row " .. i .. ": depth " .. tostring (row.depth) .. " is repeated, skipped or out of order (expected " .. depth .. ")") end
			if row["return"] ~= 0 and row["return"] ~= 1 then bad ("row " .. i .. ": return is neither 0 nor 1") end
			if not whole_number (row.writes) or row.writes < 0 or row.writes > 9 or (row.writes > 0) ~= (row.p ~= nil) then bad ("row " .. i .. ": writes and the coordinate words disagree") end
			if row.p ~= nil then
				if not hex32_list (row.p, 9) then bad ("row " .. i .. ": the coordinates are not nine float bit patterns") end
				local changed = 0
				for word in string.gmatch (row.p, "[^,]+") do
					if word ~= SENTINEL then changed = changed + 1 end
				end
				if changed ~= row.writes then bad ("row " .. i .. ": writes does not match the words that changed") end
			end
			if row["return"] == 0 then
				-- the original said false: the fixture stops here, so the very next row must say so at this depth
				local nextrow = rows [i + 1]
				if not nextrow or nextrow.event ~= "model-end" or nextrow.reason ~= "original_false" or nextrow.depth ~= row.depth then
					bad ("row " .. i .. ": a false return is not followed by its original_false end at the same depth")
				end
			end
			calls = calls + 1
			depth = depth + 1
		elseif row.event == "model-end" then
			if terminal then bad ("a second end row") end
			if row.scene ~= scene or not whole_number (row.depth) then bad ("the end row names another scene or has no depth") end
			if rows [n].event ~= "model-geometry-end" or i ~= n - 1 then bad ("the end row is not followed by exactly the closing row") end
			if row.reason == "original_false" then
				local previous = rows [i - 1]
				if i == 2 or previous.event ~= "model-tri" or previous["return"] ~= 0 or previous.depth ~= row.depth then
					bad ("original_false without an original call that returned false at that depth")
				end
			elseif row.reason == "end_probe_unbacked" then
				if row.depth ~= depth or header.end_read_backed ~= 0 or row.depth ~= header.fan_bound then bad ("end_probe_unbacked contradicts the header or the depth reached") end
			elseif row.reason == "beyond_safe_bound" then
				-- depth == fan_bound beyond the backed range is the unbacked end probe, which the fixture names as such
				if row.depth ~= depth or depth <= max_depth or depth == header.fan_bound then bad ("beyond_safe_bound contradicts the header or the depth reached") end
			else
				bad ("unknown end reason " .. tostring (row.reason))
			end
			terminal = row
		elseif row.event == "model-geometry-end" then
			if i ~= n then bad ("rows after the closing row") end
			if row.scene ~= scene or row.calls ~= calls then bad ("the closing row names another scene or its call count (" .. tostring (row.calls) .. ") is not the rows' (" .. calls .. ")") end
			if not terminal and calls ~= count then bad ("the page stopped after " .. calls .. " of " .. count .. " calls with no end row") end
		else
			bad ("unexpected row " .. i .. " (" .. tostring (row.event) .. ")")
		end
	end
	if rows [n].event ~= "model-geometry-end" then bad ("the reply is truncated: no closing row") end
	return { finished = terminal ~= nil, calls = calls, outcome = terminal and terminal.reason or "page" }
end

-- Which database is this? prepare_installation keeps a retail 3D database only when cohokum/3ddata/textures.pal exists and otherwise WRITES a synthetic one
-- (every scene one polygon-less box, every collision object absent). The marker is read BEFORE prepare_installation can create anything; without it
-- this is not a retail characterisation and nothing is booted or written.
local function file_size (path)
	local file = io.open (path, "rb")
	if not file then return -1 end
	local size = file:seek ("end")
	file:close ()
	return size or -1
end
local data_directory = string.gsub (root, "\\", "/") .. "/cohokum/3ddata/"
local provenance = {
	event = "provenance", textures_pal = file_size (data_directory .. "textures.pal"), objs_bin = file_size (data_directory .. "3dobjs.bin"),
	objdb_bin = file_size (data_directory .. "3dobjdb.bin"), objs_pts = file_size (data_directory .. "3dobjs.pts"), objs_sp = file_size (data_directory .. "3dobjs.sp"),
}
provenance.database = (provenance.textures_pal >= 0) and "retail" or "synthetic"
emit (provenance)
if provenance.database ~= "retail" then
	assert (out:close ())
	error ("NOT RUN: no retail 3D database (cohokum/3ddata/textures.pal) under the installation root; the engine would write a synthetic one with no collision geometry", 0)
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
emit { event = "boot", scenario = scenario_name, catalogue = catalogue and 1 or 0, scenes = #scenes, max_triangles = max_triangles }

local clock_before = engine:clock ()
validate_clock (clock_before, "before")
local state_before = reply ("state before", engine:model_state ())
validate_state (emit_report (state_before, { phase = "before" }), "before")

local summary = validate_summary (emit_report (reply ("summary", engine:model_summary ())))

-- the catalogue: every scene, in pages; each page is validated before its scenes are tallied
local tally = {}
if catalogue then
	local first = 1
	while first < summary.scene_last do
		local count = math.min (summary.page_scenes, summary.scene_last - first)
		local rows = emit_report (reply ("catalogue page " .. first, engine:model_catalogue (first, count)))
		local scenes_in_page = validate_catalogue_page (rows, first, count, summary.scene_last)
		for _, row in ipairs (rows) do
			if row.event == "model-scene" then tally [row.status] = (tally [row.status] or 0) + 1 end
		end
		check ("catalogue page " .. first .. " has " .. count .. " scenes", scenes_in_page == count, "found " .. scenes_in_page)
		first = first + count
	end
	for status, n in pairs (tally) do emit { event = "catalogue-status", status = status, scenes = n } end
end

-- the geometry: the original accessor, depth by depth; each page is validated before the next is asked for
for _, scene in ipairs (scenes) do
	if scene >= summary.scene_last then
		emit { event = "model-geometry-skipped", scene = scene, reason = "scene_out_of_range" }
		check ("scene " .. scene .. " is within the catalogue", false, "scene_last " .. summary.scene_last)
	else
		local depth, finished, captured, known = 0, false, 0, {}
		while not finished do
			local count = math.min (summary.page_triangles, max_triangles - depth)
			local rows = emit_report (reply ("geometry scene " .. scene .. " depth " .. depth, engine:model_geometry (scene, depth, count)))
			local page = validate_geometry_page (rows, scene, depth, count, known)
			captured = captured + page.calls
			if page.finished then
				finished = true -- an absent or refused model, or the end the original (or the fixture's gate, said so) reached
			else
				depth = depth + count -- a full page: every one of its `count` calls was a true return
				if depth >= max_triangles then
					emit { event = "model-capped", scene = scene, calls = captured, max_triangles = max_triangles }
					finished = true
				end
			end
		end
	end
end

-- the observer left everything where it was
local clock_after = engine:clock ()
validate_clock (clock_after, "after")
local state_after = reply ("state after", engine:model_state ())
validate_state (emit_report (state_after, { phase = "after" }), "after")
check ("model tables unchanged", state_before == state_after, "hashes of the scene table, object table, surface block, addresses and decoder arrays")
check ("clock unchanged", clock_before.elapsed_seconds == clock_after.elapsed_seconds and clock_before.time_of_day_seconds == clock_after.time_of_day_seconds
	and clock_before.day == clock_after.day, "no frame was stepped")
emit { event = "done", failures = failures }
assert (out:close ())
io.stderr:write (string.format ("model-characterization: %d failed checks\n", failures))
if failures > 0 then error (string.format ("%d checks failed", failures), 0) end
