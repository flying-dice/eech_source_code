-- TEST-ONLY native boundary witness for the tree-query transcript's paging (EEDCS-49). Needs a DLL built with the `collision-fixture` feature and a real installation root; it is run
-- by Root against the actual built host and DLL, never by a stand-in: a mock cannot show that the C paging arithmetic never overflows.
--
--   eech-world.exe collision-trace-paging.lua root=<installation root> out=<JSON Lines path> [scenario=lebanon_retail|georgia_retail|georgia]
--
-- It boots the campaign as the driver does, arms the transcript, runs one small probe (seed 20261029, 5 entity cases, 5 tree cases, at the freshly booted world), and then pages the
-- transcript through the public Lua -> module -> native path:
--   * NORMAL pages: the whole transcript in pages of 256 from offset 0; every page has min(256, total - offset) records, starts at the expected ordinal, and the total is reported
--     unchanged by every read;
--   * the LAST record alone (offset total - 1, count 256);
--   * END pages, which are SUCCESSFUL EMPTY pages with the total reported: offsets total, total + 1, 2147483647 (INT_MAX), 2147483646, 2147483392 (INT_MAX - 255, where offset + 256 is the
--     first sum above INT_MAX) and 2147483391 (INT_MAX - 256, where offset + 256 is exactly INT_MAX), each with count 256;
--   * REFUSALS (an error, never a page): a negative offset, count 0, count 257, kind 2, kind -1, and an offset beyond 32-bit range.
-- The contract under test (csrc/eech_collision_fixture.c, eech_collision_trace_read_run): bad arguments are refused; an offset at or past the end is an empty page; otherwise the page is
-- min(count, total - offset) records. The C no longer computes offset + count, so no addition can overflow whatever offset is passed. (A build compiled with -fwrapv would have wrapped
-- the old sum to a negative number and returned an empty page by accident; this witness tests the contract, not that accident.)
--
-- It prints one line `COLLISION_TRACE_PAGING_OK total=... normal_pages=... end_pages=... refusals=...` and exits 0 only if every check holds; any failure is an error. Its argument
-- check runs before anything is opened or booted. A run that did not print that line is not a pass and is never counted as a skip.

io.stdout:setvbuf ("no")

local args = host.args

local function refuse (message)
	error ("collision-trace-paging: " .. message, 0)
end

local known = { root = true, out = true, scenario = true }
for key in pairs (args) do
	if not known [key] then refuse ("unknown argument '" .. tostring (key) .. "'") end
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
local existing = io.open (out_path, "rb")
if existing then
	existing:close ()
	refuse ("out '" .. out_path .. "' already exists: an existing file is never overwritten")
end

local out = assert (io.open (out_path, "w"))

local function json_string (text)
	return "\"" .. string.gsub (text, "[%z\1-\31\"\\]", function (c) return string.format ("\\u%04x", string.byte (c)) end) .. "\""
end

local function emit (event, fields)
	local parts = { "\"event\":" .. json_string (event) }
	for _, item in ipairs (fields or {}) do
		local key, value = item [1], item [2]
		parts [#parts + 1] = json_string (key) .. ":" .. (type (value) == "string" and json_string (value) or tostring (value))
	end
	out:write ("{" .. table.concat (parts, ",") .. "}\n")
	out:flush ()
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
emit ("boot", { { "scenario", scenario_name } })

local armed = false

local function main ()
	armed = true
	engine:collision_trace_arm (true)
	engine:collision_probe (20261029, 5, 5)

	local state, total = engine:collision_trace_read (0, 0, 1)
	local state_total = string.match (state, "^tr%-state armed=1 records=(%d+) ")
	if state_total == nil or tonumber (state_total) ~= total then
		refuse ("the state row does not agree with the total: " .. tostring (state_total) .. " against " .. tostring (total))
	end
	if total < 4 then refuse ("the transcript holds " .. tostring (total) .. " records; the structured edge cases alone make at least 4") end

	local function lines_of (text)
		local _, n = string.gsub (text, "\n", "")
		return n
	end

	-- normal pages
	local normal_pages, seen = 0, 0
	local offset = 0
	while offset < total do
		local text, again = engine:collision_trace_read (1, offset, 256)
		local want = math.min (256, total - offset)
		if again ~= total then refuse ("a page changed the total: " .. tostring (total) .. " then " .. tostring (again)) end
		if lines_of (text) ~= want then refuse ("the page at " .. offset .. " held " .. lines_of (text) .. " records, expected " .. want) end
		local first = string.match (text, "^tq ordinal=(%d+) ")
		if tonumber (first) ~= offset then refuse ("the page at " .. offset .. " starts at ordinal " .. tostring (first)) end
		emit ("page", { { "offset", offset }, { "records", want }, { "total", again } })
		seen = seen + want
		normal_pages = normal_pages + 1
		offset = offset + want
	end
	if seen ~= total then refuse ("the pages held " .. seen .. " records of " .. total) end

	-- the last record alone
	local last, again = engine:collision_trace_read (1, total - 1, 256)
	if again ~= total or lines_of (last) ~= 1 or tonumber (string.match (last, "^tq ordinal=(%d+) ")) ~= total - 1 then
		refuse ("the page at the last record is not exactly that record")
	end
	emit ("last", { { "offset", total - 1 }, { "records", 1 } })

	-- end pages: successful, empty, total reported
	local end_offsets = { total, total + 1, 2147483647, 2147483646, 2147483392, 2147483391 }
	for _, end_offset in ipairs (end_offsets) do
		local ok, text, end_total = pcall (function () return engine:collision_trace_read (1, end_offset, 256) end)
		if not ok then refuse ("offset " .. end_offset .. " was refused, expected an empty page: " .. tostring (text)) end
		if text ~= "" or end_total ~= total then
			refuse ("offset " .. end_offset .. " returned " .. #text .. " bytes and total " .. tostring (end_total) .. ", expected an empty page and total " .. total)
		end
		emit ("end", { { "offset", end_offset }, { "bytes", 0 }, { "total", end_total } })
	end

	-- refusals: an error, never a page
	local refusals = {
		{ "negative offset", 1, -1, 10 },
		{ "count 0", 1, 0, 0 },
		{ "count 257", 1, 0, 257 },
		{ "kind 2", 2, 0, 1 },
		{ "kind -1", -1, 0, 1 },
		{ "offset beyond 32 bits", 1, 2147483648, 1 },
	}
	for _, case in ipairs (refusals) do
		local ok, text = pcall (function () return engine:collision_trace_read (case [2], case [3], case [4]) end)
		if ok then refuse (case [1] .. " was accepted") end
		emit ("refused", { { "case", case [1] }, { "error", tostring (text) } })
	end

	armed = false
	engine:collision_trace_arm (false)
	return total, normal_pages, #end_offsets, #refusals
end

local ok, total, normal_pages, end_pages, refusal_count = pcall (main)
if not ok then
	emit ("failed", { { "error", tostring (total) } })
	if armed then pcall (function () engine:collision_trace_arm (false) end) end
	out:close ()
	error (total, 0)
end
emit ("done", { { "total", total }, { "normal_pages", normal_pages }, { "end_pages", end_pages }, { "refusals", refusal_count } })
out:close ()
print (string.format ("COLLISION_TRACE_PAGING_OK total=%d normal_pages=%d end_pages=%d refusals=%d", total, normal_pages, end_pages, refusal_count))
