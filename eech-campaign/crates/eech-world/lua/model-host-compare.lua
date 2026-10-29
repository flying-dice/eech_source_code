-- TEST-ONLY comparison of the HOST-OWNED collision-model store with the retained ORIGINAL model report (EEDCS-50, D3). No campaign DLL is loaded and no engine is booted:
-- host.model_store reads the retail disk assets itself (3dobjs.bin/.pts/.sp, 3dobjdb.bin, bininfo.bin and the active OBJECTS overrides), and answers with the numeric part of
-- the original's own code (csrc/model_step.c). Everything the original reported is the oracle; nothing here computes an expected value.
--
-- It writes, as JSON Lines (never overwriting, every write checked):
--   * provenance and the store's load summary (what overrides were applied, what was skipped or unresolved);
--   * with catalogue=1 (default), a `host-model-scene` row for every scene: name, loaded, indices, radius and bounds as float bit patterns, the collision object's counts;
--   * for each scene named in scenes=, `host-model-tri` rows from depth 0 until the host's end / absence / refusal (or max_triangles, reported `host-model-capped`);
--   * with original=<path to a retained original model-driver report>, the comparison: every `model-scene` row's name, indices (where both number objects alike), radius,
--     bounds and collision-object counts, and every `model-tri` row's return and nine float bit patterns, against the store. Mismatches are listed (the totals are complete).
--     A scene the original REFUSED (a custom object, a mixed layout) has no original triangles; the host's answer for it is listed as such, not counted as a match.
-- A missing asset is NOT RUN (an error before any output beyond the provenance row), a refused store is an error, any mismatch fails the run, and a comparison that compared
-- nothing is a failure, never a pass.
--
-- host.args: root (the installation root holding cohokum/3ddata), out (JSON Lines path), original (optional), catalogue (1 default, 0 to skip),
--            scenes (comma-separated scene numbers, optional), max_triangles (default 4096, at most 65536)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local out_path = assert (args.out, "out=<JSON Lines path>")

local function whole (text, name, min, max)
	local number = tonumber (text)
	if not number or number ~= number or number == math.huge or number == -math.huge or number % 1 ~= 0 or number < min or number > max then
		error (string.format ("%s must be a whole number within %d..%d, got %s", name, min, max, tostring (text)), 0)
	end
	return number
end

-- the arguments are checked before the output is opened or the store is loaded
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
if args.original ~= nil and args.original == "" then error ("original= names no file", 0) end
if not catalogue and #scenes == 0 and not args.original then error ("nothing is asked: catalogue=0, no scenes, no original", 0) end

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

local function stop (message)
	assert (out:close ())
	io.stderr:write ("model-host-compare: " .. message .. "\n")
	error (message, 0)
end

emit { event = "provenance", component = "host-owned collision-model store", campaign_dll = "not loaded", catalogue = catalogue and 1 or 0, scenes = #scenes, original = args.original and "given" or "none" }

local loaded, store = pcall (host.model_store, (string.gsub (root, "\\", "/")))
if not loaded then
	local message = tostring (store)
	emit { event = "store-refused", message = message }
	stop (message)
end

local summary = store:summary ()
do
	local row = { event = "host-model-summary" }
	for key, value in pairs (summary) do
		if type (value) == "table" then
			row [key .. "_count"] = #value
			row [key] = table.concat (value, ";")
		else
			row [key] = value
		end
	end
	emit (row)
end

local failures = 0

if catalogue then
	local counts = {}
	for scene = 1, summary.scene_last - 1 do
		local view = store:scene (scene)
		local status = (not view.loaded) and "no_scene" or (view.collision == -1 and "absent" or "present")
		counts [status] = (counts [status] or 0) + 1
		view.event = "host-model-scene"
		view.status = status
		emit (view)
	end
	for status, n in pairs (counts) do emit { event = "host-catalogue-status", status = status, scenes = n } end
end

for _, scene in ipairs (scenes) do
	if scene >= summary.scene_last then
		emit { event = "host-model-skipped", scene = scene, reason = "scene_out_of_range" }
		failures = failures + 1
	else
		local depth, finished = 0, false
		while not finished do
			local answer = store:triangle (scene, depth)
			local row = { event = "host-model-tri", scene = scene, depth = depth, result = answer.result }
			if answer.p then row.p = answer.p end
			if answer.reason then row.reason = answer.reason end
			emit (row)
			if answer.result ~= "triangle" then
				finished = true
			else
				depth = depth + 1
				if depth >= max_triangles then
					emit { event = "host-model-capped", scene = scene, calls = depth, max_triangles = max_triangles }
					finished = true
				end
			end
		end
	end
end

if args.original then
	local ok, report = pcall (store.compare_original, store, (string.gsub (args.original, "\\", "/")))
	if not ok then
		emit { event = "compare-refused", message = tostring (report) }
		stop (tostring (report))
	end
	local row = { event = "compare" }
	for key, value in pairs (report) do
		if type (value) ~= "table" then row [key] = value end
	end
	emit (row)
	for _, text in ipairs (report.listed) do emit { event = "compare-mismatch", text = text } end
	for _, text in ipairs (report.original_refused_host_answers) do emit { event = "compare-original-refused", text = text } end
	failures = failures + report.scene_mismatches + report.triangle_mismatches
	if report.scene_rows_compared == 0 and report.triangles_compared == 0 then
		emit { event = "compare-empty", text = "the original report held no model-scene or model-tri row to compare" }
		failures = failures + 1
	end
end

emit { event = "done", failures = failures }
assert (out:close ())
io.stderr:write (string.format ("model-host-compare: %d failures\n", failures))
if failures > 0 then error (string.format ("%d failures", failures), 0) end
