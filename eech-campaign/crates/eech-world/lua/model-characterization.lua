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
local state_before = engine:model_state ()
emit_report (state_before, { phase = "before" })

local summary_rows = emit_report (engine:model_summary ())
local summary = summary_rows [1]
assert (summary and summary.event == "model-summary" and summary.scene_last and summary.page_scenes and summary.page_triangles, "no model-summary")

-- the catalogue: every scene, in pages
local tally = {}
if catalogue then
	local first = 1
	while first < summary.scene_last do
		local count = math.min (summary.page_scenes, summary.scene_last - first)
		local rows = emit_report (engine:model_catalogue (first, count))
		local scenes_in_page = 0
		for _, row in ipairs (rows) do
			if row.event == "model-scene" then
				scenes_in_page = scenes_in_page + 1
				tally [row.status] = (tally [row.status] or 0) + 1
			end
		end
		check ("catalogue page " .. first .. " has " .. count .. " scenes", scenes_in_page == count, "found " .. scenes_in_page)
		first = first + count
	end
	for status, n in pairs (tally) do emit { event = "catalogue-status", status = status, scenes = n } end
end

-- the geometry: the original accessor, depth by depth
for _, scene in ipairs (scenes) do
	if scene >= summary.scene_last then
		emit { event = "model-geometry-skipped", scene = scene, reason = "scene_out_of_range" }
		check ("scene " .. scene .. " is within the catalogue", false, "scene_last " .. summary.scene_last)
	else
		local depth, finished, captured = 0, false, 0
		while not finished do
			local count = math.min (summary.page_triangles, max_triangles - depth)
			local rows = emit_report (engine:model_geometry (scene, depth, count))
			for _, row in ipairs (rows) do
				if row.event == "model-tri" then
					captured = captured + 1
				elseif row.event == "model-end" or row.event == "model-refused" then
					finished = true
				elseif row.event == "model-geometry" and row.status == "absent" then
					finished = true -- the original answered the absence once; asking deeper adds nothing
				end
			end
			depth = depth + count
			if not finished and depth >= max_triangles then
				emit { event = "model-capped", scene = scene, calls = captured, max_triangles = max_triangles }
				finished = true
			elseif not finished and #rows == 0 then
				finished = true
			end
		end
	end
end

-- the observer left everything where it was
local clock_after = engine:clock ()
local state_after = engine:model_state ()
emit_report (state_after, { phase = "after" })
check ("model tables unchanged", state_before == state_after, "hashes of the scene table, object table, surface block, addresses and decoder arrays")
check ("clock unchanged", clock_before.elapsed_seconds == clock_after.elapsed_seconds and clock_before.time_of_day_seconds == clock_after.time_of_day_seconds
	and clock_before.day == clock_after.day, "no frame was stepped")
emit { event = "done", failures = failures }
assert (out:close ())
io.stderr:write (string.format ("model-characterization: %d failed checks\n", failures))
if failures > 0 then error (string.format ("%d checks failed", failures), 0) end
