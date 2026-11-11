-- Replays the REAL captured tree queries of the original solver through the reference host's Lua transport (host.tree_collision), which calls the
-- host-owned tree loader and kernel (EEDCS-49, D3 PREPARATION). It is the Lua-path twin of tests/tree_conformance.rs: the same private transcript, the
-- same map files, the same per-query checks, but every number crosses Lua -> host -> C kernel -> Lua, so the host's exact number reads and bit echoes
-- are in the proof. The campaign DLL is not involved and does not call this path.
--
--   eech-world lua/tree-collision-replay.lua transcript=<tree-transcript.jsonl> map_dir=<the map's terrain directory>
--
-- Every query is checked and the script FAILS (a Lua error, so a non-zero exit) on any difference, after listing the first ones. Nothing is normalised
-- and no tolerance is applied: the floats are compared as the hex bits the host echoes. Not passing is the default; the final line
-- TREE_LUA_REPLAY_DONE queries=N lookups=M hits=H misses=X mismatches=0 is printed only when everything matched.
--
-- Forest geometry: the retail facts record no forest tree collision triangles (tf-forest geometry 0), and then the host is told so explicitly ("absent"). A transcript
-- that records triangles (the CONTROLLED forest geometry witness of the original: tf-forest geometry 1 with its tf-triangle rows) is replayed with those recorded triangles, in order,
-- as the host-owned mesh. Anything else (a capped, miscounted or out-of-order triangle list) is refused: it is never counted as a pass of the absent case. The geometry branch of the
-- host kernel is NOT considered validated by this script unless a controlled transcript was replayed and every query matched.

local transcript_path = host.args.transcript
local map_dir = host.args.map_dir
if not transcript_path or not map_dir then
	error ("tree-collision-replay: transcript=<path> and map_dir=<path> are required (a run without them proves nothing)")
end

-- An f32 bit word as the number it is, exactly (a double holds every f32 exactly); a signed zero keeps its sign. Inputs here are finite.
local function f32_from_hex (hex)
	local n = tonumber (hex, 16)
	local sign = (n >= 2147483648) and -1 or 1
	local exponent = math.floor (n / 8388608) % 256
	local mantissa = n % 8388608
	if exponent == 255 then
		error ("tree-collision-replay: a non-finite input " .. hex)
	elseif exponent == 0 then
		return sign * mantissa * 2 ^ -149
	end
	return sign * (1 + mantissa / 8388608) * 2 ^ (exponent - 127)
end

local function triple (text)
	local a, b, c = string.match (text, "^(%x%x%x%x%x%x%x%x),(%x%x%x%x%x%x%x%x),(%x%x%x%x%x%x%x%x)$")
	if not a then error ("tree-collision-replay: not a triple of f32 words: " .. tostring (text)) end
	return { a, b, c }
end

local function numbers (words)
	return { f32_from_hex (words[1]), f32_from_hex (words[2]), f32_from_hex (words[3]) }
end

-- one transcript line: flat JSON of strings and integers
local function parse (line)
	local row = {}
	for key, value in string.gmatch (line, '"([%w_]+)":"([^"]*)"') do row[key] = value end
	for key, value in string.gmatch (line, '"([%w_]+)":(-?%d+)') do row[key] = tonumber (value) end
	return row
end

local rows, tqs, one = {}, {}, {}
local file = assert (io.open (transcript_path, "rb"))
for line in file:lines () do
	local row = parse (line)
	if not row.event then error ("tree-collision-replay: a line without an event: " .. line) end
	rows[#rows + 1] = row
	if row.event == "tq" then tqs[#tqs + 1] = row else one[row.event] = (one[row.event] or 0) + 1 end
end
file:close ()

local function single (event)
	if one[event] ~= 1 then error ("tree-collision-replay: " .. tostring (one[event] or 0) .. " " .. event .. " rows, expected 1") end
	for _, row in ipairs (rows) do if row.event == event then return row end end
end

-- the transcript is complete and the capture was healthy
assert (rows[1].event == "trace-start", "the transcript does not start with trace-start")
assert (rows[#rows].event == "trace-end", "the transcript does not end with trace-end")
local state = single ("tr-state")
assert (state.armed == 1 and state.overflow == 0 and state.facts_overflow == 0 and state.extra_lookups == 0, "the capture was not healthy")
assert (#tqs == state.records and #tqs == rows[#rows].records and #tqs > 0, "tq rows against tr-state/trace-end records")

local layout = single ("tf-layout")
assert (layout.record_size == 8 and layout.x_offset == 0 and layout.z_offset == 2 and layout.y_offset == 4 and layout.colour_offset == 6, "tf-layout")

local forest = single ("tf-forest")
assert (forest.capped == 0, "the capture capped the forest triangles")
local forest_geometry
if forest.geometry == 0 then
	assert (forest.triangles == 0 and (one["tf-triangle"] or 0) == 0, "tf-forest says no geometry but triangles are recorded")
	forest_geometry = "absent"
else
	assert (forest.geometry == 1 and forest.triangles > 0 and one["tf-triangle"] == forest.triangles, "tf-forest geometry and its tf-triangle rows disagree")
	forest_geometry = {}
	local expected_index = 0
	for _, row in ipairs (rows) do
		if row.event == "tf-triangle" then
			assert (row.i == expected_index, "tf-triangle rows out of order")
			expected_index = expected_index + 1
			local flat = {}
			for _, key in ipairs { "p0", "p1", "p2" } do
				for _, value in ipairs (numbers (triple (row[key]))) do flat [#flat + 1] = value end
			end
			forest_geometry [#forest_geometry + 1] = flat
		end
	end
end

local facts = single ("tf-map")
assert (facts.terrain_sector_side == 2048 and facts.max_tree_height == 35, "tf-map")
local map = {
	min_x = f32_from_hex (facts.map_min_x), max_x = f32_from_hex (facts.map_max_x),
	min_z = f32_from_hex (facts.map_min_z), max_z = f32_from_hex (facts.map_max_z),
	sectors_x = facts.terrain_sectors_x, sectors_z = facts.terrain_sectors_z,
	minimum_height = f32_from_hex (facts.height_minimum), scaled_height_difference = f32_from_hex (facts.height_scale),
}

local census = host.tree_collision { mode = "load", directory = map_dir, map = map, forest_geometry = forest_geometry }
assert (census.loaded == true, "the map's trees were not loaded: " .. tostring (census.no_trees))
assert (census.tree_total == facts.tree_total, "tree_total " .. tostring (census.tree_total) .. " against " .. tostring (facts.tree_total))
assert (census.sectors_with_trees == facts.sectors_with_trees, "sectors_with_trees " .. tostring (census.sectors_with_trees))
print (string.format ("TREE_LUA_REPLAY_LOADED sector_file=%s data_file=%s tree_total=%d sectors_with_trees=%d records_in_file=%d", tostring (census.sector_file),
	tostring (census.data_file), census.tree_total, census.sectors_with_trees, census.records_in_file))

local mismatches, listed = 0, 0
local function mismatch (ordinal, what)
	mismatches = mismatches + 1
	if listed < 40 then
		listed = listed + 1
		print (string.format ("MISMATCH tq %s: %s", tostring (ordinal), what))
	end
end

local lookups_made, hits, misses = 0, 0, 0
for _, q in ipairs (tqs) do
	local count = q.lookups
	assert (count >= 0 and count <= 2 and q.extra_lookups == 0, "lookups")
	local recorded = {}
	if count >= 1 then recorded[1] = triple (q.l0) end
	if count >= 2 then recorded[2] = triple (q.l1) end
	local old, new = triple (q.a), triple (q.b)
	local answer = host.tree_collision {
		mode = "begin",
		old_position = numbers (old), new_position = numbers (new),
		intercept_point = numbers (triple (q.init_point)), face_normal = numbers (triple (q.init_normal)),
		ignore_forest_edge_trees = (q.ignore_edge == 1), skip_tree_top_test = (q.skip_top == 1),
	}
	local asked = 0
	while answer.state == "need_terrain" do
		asked = asked + 1
		local want = recorded[asked]
		if not want then
			mismatch (q.ordinal, "the host asked for lookup " .. asked .. " but the original made " .. count)
			host.tree_collision { mode = "abandon" }
			break
		end
		if answer.x_bits ~= want[1] or answer.z_bits ~= want[2] then
			mismatch (q.ordinal, string.format ("lookup %d is %s,%s, the original's was %s,%s", asked, answer.x_bits, answer.z_bits, want[1], want[2]))
		end
		answer = host.tree_collision { mode = "terrain", elevation = f32_from_hex (want[3]) }
	end
	lookups_made = lookups_made + asked
	if answer.state == "done" then
		if asked ~= count then mismatch (q.ordinal, "the host made " .. asked .. " lookups, the original " .. count) end
		if answer.hit ~= (q.hit == 1) then mismatch (q.ordinal, "hit is " .. tostring (answer.hit) .. ", the original's was " .. tostring (q.hit == 1)) end
		if answer.point_bits ~= q.point then mismatch (q.ordinal, "point is " .. answer.point_bits .. ", the original's was " .. q.point) end
		if answer.normal_bits ~= q.normal then mismatch (q.ordinal, "normal is " .. answer.normal_bits .. ", the original's was " .. q.normal) end
		if q.hit == 1 then hits = hits + 1 else misses = misses + 1 end
	end
end

if mismatches > 0 then
	error (string.format ("tree-collision-replay: %d mismatches over %d queries (the first %d are listed above)", mismatches, #tqs, listed))
end
assert (hits > 0 and misses > 0, "a replay that never hit or never missed proves little")
print (string.format ("TREE_LUA_REPLAY_DONE queries=%d lookups=%d hits=%d misses=%d mismatches=0", #tqs, lookups_made, hits, misses))
