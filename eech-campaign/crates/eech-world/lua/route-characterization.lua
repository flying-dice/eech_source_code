-- Original route-geometry observer (EEDCS-50, D3). TEST-ONLY: it needs a module built with the `route-fixture` feature (the ORIGINAL airfield route parser and its route-object
-- lookup, observed by csrc/eech_route_fixture.c) and the reference host (EECH_WORLD), and it measures what the DLL's own loader and parser did for ONE natural retail boot.
--
-- It boots the campaign exactly as lua/campaign.lua does (the same preparation, the reference arithmetic profile, the same boot configuration, seed 1) and asks for NOTHING
-- else: no frame is run (the airfield populations, and with them every route, are read while the campaign loads), no entity is created and no setting is changed. After the
-- boot it reads the observer's summary and then every recorded parser call, once each, and writes them as JSON Lines, the native report text verbatim in each row.
--
-- host.args: root (installation root, required), out (the JSON Lines file, required; created exclusively, see OUTPUT), scenario (lebanon_retail, the default, or georgia_retail)
--
-- OUTPUT. The file is created with host.create_exclusive (src/exclusive_output.rs): atomic, exclusive, refusing ANY existing target (a readable or unreadable file, a directory, a
-- link), and every write, flush and close is checked. io.open "w" truncates and a read-open check races and misses unreadable files, so neither is used. The owned output is
-- closed on EVERY exit after it was created, success or failure; a failure to close is reported (together with the primary failure, if there is one).
--
-- THREE KINDS OF OUTCOME, kept apart.
--   * MEASURED: the summary and every call are well formed AND the observer's own verdicts are complete (route-complete value=1, every call complete=1) AND every cohort a complete
--     call promises is present in its rows (checked here against the call's OWN declared counts and row order; no route answer is recomputed). Only this passes.
--   * WELL-FORMED BUT INCOMPLETE: the native grammar and every count are coherent but the observer says the capture is not whole (an empty boot, a full table, a missing, mismatched, unmatched or
--     conflicting caller context, a clipped list, an unended call ...). That is the engine's honest answer, not an engine failure, so the remaining recorded calls ARE still read
--     (once each, for diagnosis); then the run writes done complete=false and FAILS. It is never reported as a measured original.
--   * MALFORMED: a line the C grammar cannot produce, an out-of-domain or contradictory number, rows that do not match their declared counts or each other, a summary that
--     disagrees with the calls, or a complete=1 claim the rows do not support. The text is already written verbatim; a `malformed` row and done complete=false follow, the
--     script FAILS, and the engine is NOT asked again.
-- An engine call that itself fails (an error raised by route_summary or route_call) stops the questions at once. After a malformed report or a failed engine call the engine is
-- not asked again; the only continued questioning is the well-formed-incomplete diagnosis above.
--
-- The caps below repeat the C tables (csrc/eech_route_fixture.c RF_*); tests/route_characterization.rs checks that they are the same numbers. A reached route object is not proof
-- that every route object of the installation was parsed, and this script says nothing about the host's model service or about anything but the parser's own tables.

io.stdout:setvbuf ("no")

local here = debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or ""
local json = dofile (here .. "json-lines.lua")

local args = host.args

-- ---------------------------------------------------------------------------------------------------------------------------------
-- every argument is checked, strictly, BEFORE the output is created, the installation is prepared or anything is booted

local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}

local function fail_args (message) error ("route-characterization: " .. message, 0) end

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
-- the native report grammar (csrc/eech_route_fixture.c), checked line by line

local CAP = { calls = 1024, lookups = 4096, slots = 8192, nodes = 32768, faces = 32768, finals = 32768, links = 131072, starts = 8192, depths = 8192, offsets = 65536 }
local INT_MAX = 2147483647

local function malformed (message) error ({ malformed = message }, 0) end

-- a decimal integer exactly as C's %d writes it (no plus sign, no leading zero, no "-0"), within lo..hi
local function int (text, lo, hi, what)
	if type (text) ~= "string" or #text > 11 or not (text == "0" or string.match (text, "^%-?[1-9]%d*$")) then malformed (what .. " is not a decimal integer: " .. tostring (text)) end
	local n = tonumber (text)
	if n < lo or n > hi then malformed (string.format ("%s %s is outside %d..%d", what, text, lo, hi)) end
	return n
end

local function flag (text, what) return int (text, 0, 1, what) end

-- a 32-bit word exactly as C's %08x writes it
local HEX8 = "^" .. string.rep ("[0-9a-f]", 8) .. "$"
local function word (text, what)
	if type (text) ~= "string" or not string.match (text, HEX8) then malformed (what .. " is not a lower-case 8-digit word: " .. tostring (text)) end
	return text
end

local function words (text, n, what)
	local list = {}
	for item in string.gmatch (text .. ",", "([^,]*),") do list [#list + 1] = word (item, what) end
	if #list ~= n then malformed (string.format ("%s has %d words, expected %d", what, #list, n)) end
	return list
end

local function ints (text, n, lo, hi, what)
	local list = {}
	for item in string.gmatch (text .. ",", "([^,]*),") do list [#list + 1] = int (item, lo, hi, what) end
	if #list ~= n then malformed (string.format ("%s has %d integers, expected %d", what, #list, n)) end
	return list
end

-- a compiled scene or sub-object name (an identifier's text), or the "-" the observer writes for an index it has no name for
local function name (text, what)
	if type (text) ~= "string" or #text > 127 or not string.match (text, "^[%w_%.%-]+$") then malformed (what .. " is not a name: " .. tostring (text)) end
	return text
end

-- the text as a list of lines: only complete lines, none empty, no carriage return, no NUL
local function lines_of (text, what)
	if text == "" or string.sub (text, -1) ~= "\n" then malformed (what .. " does not end in a complete line") end
	if string.find (text, "[%z\r]") then malformed (what .. " holds a NUL or a carriage return") end
	local list = {}
	for line in string.gmatch (text, "([^\n]*)\n") do
		if line == "" then malformed (what .. " holds an empty line") end
		list [#list + 1] = line
	end
	return list
end

-- A reader over the lines. expect (line name, ordered key list) consumes the next line, which must be single-space separated with exactly those keys in that order, and returns the
-- raw value of each key.
local function reader (list, what)
	local at = 0
	local r = {}
	function r.expect (line_name, keys)
		at = at + 1
		local line = list [at]
		if not line then malformed (what .. " ends before a '" .. line_name .. "' line") end
		local tokens = {}
		for token in string.gmatch (line, "[^ ]+") do tokens [#tokens + 1] = token end
		if table.concat (tokens, " ") ~= line then malformed (what .. " line " .. at .. " has irregular spacing") end
		if tokens [1] ~= line_name then malformed (string.format ("%s line %d is '%s', expected '%s'", what, at, tostring (tokens [1]), line_name)) end
		if #tokens ~= #keys + 1 then malformed (string.format ("%s line %d ('%s') has %d fields, expected %d", what, at, line_name, #tokens - 1, #keys)) end
		local values = {}
		for i, key in ipairs (keys) do
			local k, v = string.match (tokens [i + 1], "^([%w_]+)=(.+)$")
			if k ~= key then malformed (string.format ("%s line %d ('%s') field %d is '%s', expected '%s'", what, at, line_name, i, tostring (k), key)) end
			values [key] = v
		end
		return values
	end
	-- the next line, not consumed
	function r.next_line () return list [at + 1] end
	function r.finish ()
		if list [at + 1] then malformed (what .. " has extra lines after its last expected one: " .. list [at + 1]) end
	end
	return r
end

-- The summary: counters, pools, the lookups and the verdict. Returns what the calls are joined against.
local function check_summary (text)
	local r = reader (lines_of (text, "the summary"), "the summary")
	-- A report of the HOST profile (the airfield route parser takes its nodes and line faces from the host) says so first; a report of the original parser has no such line.
	local profile = "original"
	if string.match (r.next_line () or "", "^route%-profile ") then
		local v = r.expect ("route-profile", { "value" })
		if v.value ~= "host" then malformed ("the profile is not the literal host") end
		profile = "host"
	end
	local s = r.expect ("route-summary", { "calls", "call_capacity", "dropped_calls", "lookups", "lookup_capacity", "dropped_lookups", "orphans", "interrupted", "open", "incomplete_calls", "contexts", "stale" })
	local summary = {
		calls = int (s.calls, 0, CAP.calls, "calls"), dropped_calls = int (s.dropped_calls, 0, INT_MAX, "dropped_calls"),
		lookups = int (s.lookups, 0, CAP.lookups, "lookups"), dropped_lookups = int (s.dropped_lookups, 0, INT_MAX, "dropped_lookups"),
		orphans = int (s.orphans, 0, INT_MAX, "orphans"), interrupted = int (s.interrupted, 0, INT_MAX, "interrupted"), open = flag (s.open, "open"),
		incomplete_calls = int (s.incomplete_calls, 0, CAP.calls, "incomplete_calls"), contexts = int (s.contexts, 0, INT_MAX, "contexts"), stale = int (s.stale, 0, INT_MAX, "stale"),
	}
	if summary.stale > summary.contexts then malformed ("stale contexts exceed the contexts registered") end
	if int (s.call_capacity, 0, INT_MAX, "call_capacity") ~= CAP.calls then malformed ("call_capacity is not the observer's " .. CAP.calls) end
	if int (s.lookup_capacity, 0, INT_MAX, "lookup_capacity") ~= CAP.lookups then malformed ("lookup_capacity is not the observer's " .. CAP.lookups) end
	summary.profile = profile
	if summary.incomplete_calls > summary.calls then malformed ("incomplete_calls exceeds calls") end
	local p = r.expect ("route-pools", { "slots", "nodes", "faces", "finals", "links", "starts", "depths", "offsets" })
	summary.pools = {}
	for _, key in ipairs { "slots", "nodes", "faces", "finals", "links", "starts", "depths", "offsets" } do
		local used, capacity = string.match (p [key], "^(%d+)/(%d+)$")
		if not used then malformed ("pool " .. key .. " is not used/capacity: " .. p [key]) end
		if int (capacity, 0, INT_MAX, "pool " .. key .. " capacity") ~= CAP [key] then malformed ("pool " .. key .. " capacity is not the observer's " .. CAP [key]) end
		summary.pools [key] = int (used, 0, CAP [key], "pool " .. key .. " used")
	end
	summary.lookup_rows = {}
	for i = 0, summary.lookups - 1 do
		local l = r.expect ("route-lookup", { "index", "scene", "scene_name", "kind", "kind_name", "found", "legacy_object" })
		if int (l.index, 0, INT_MAX, "lookup index") ~= i then malformed ("lookup rows are not in order at " .. i) end
		local row = { scene = int (l.scene, 0, INT_MAX, "lookup scene"), kind = int (l.kind, 0, INT_MAX, "lookup kind"), found = flag (l.found, "lookup found"),
			scene_name = name (l.scene_name, "lookup scene_name"), kind_name = name (l.kind_name, "lookup kind_name"), legacy = int (l.legacy_object, -1, INT_MAX, "lookup legacy_object") }
		if (row.found == 0) ~= (row.legacy == -1) then malformed ("lookup " .. i .. ": legacy_object -1 is exactly the not-found case") end
		summary.lookup_rows [i] = row
	end
	local verdict = r.expect ("route-complete", { "value" })
	summary.complete = flag (verdict.value, "route-complete value")
	r.finish ()
	if summary.complete == 1 and not (summary.calls > 0 and summary.dropped_calls == 0 and summary.dropped_lookups == 0 and summary.orphans == 0 and summary.interrupted == 0 and
			summary.open == 0 and summary.incomplete_calls == 0 and summary.stale == 0) then
		malformed ("route-complete value=1 contradicts the summary's own counters")
	end
	return summary
end

-- One call. `index` is the number asked for. The report must be a well-formed whole; a complete=1 claim must also be supported by the rows. Returns the call's facts.
local function check_call (text, index, summary)
	local what = "call " .. index
	local r = reader (lines_of (text, what), what)
	local function mine (values, row) if int (values.index, 0, INT_MAX, row .. " index") ~= index then malformed (row .. " of " .. what .. " carries another call's index") end end

	local h = r.expect ("route-call", { "index", "legacy_object", "lookup", "slots", "slots_stored", "points", "surfaces", "faces", "ended", "interrupted", "complete" })
	mine (h, "route-call")
	local c = {
		legacy = int (h.legacy_object, -INT_MAX, INT_MAX, "legacy_object"), lookup = int (h.lookup, -1, CAP.lookups - 1, "lookup"), slots = int (h.slots, -INT_MAX, INT_MAX, "slots"),
		slots_stored = int (h.slots_stored, 0, CAP.slots, "slots_stored"), points = int (h.points, 0, 65535, "points"), surfaces = int (h.surfaces, 0, 65535, "surfaces"),
		faces = int (h.faces, 0, 65535, "faces"), ended = flag (h.ended, "ended"), interrupted = flag (h.interrupted, "interrupted"), complete = flag (h.complete, "complete"),
	}
	if c.slots_stored > math.max (c.slots, 0) then malformed (what .. " stored more slots than were requested") end

	-- The identity row is the CALLER's context and has three shapes: unique (the key the caller chose, and the number it took from its entry), a context that is not an identity (mismatch,
	-- unmatched, conflict: only the number the caller took) or no context at all (missing). The state is the WHOLE token after state= and must be one of the five literals the emitter
	-- writes (a prefix such as "unique-INVALID" is not "unique").
	local state = string.match (r.next_line () or "", "^route%-identity index=%d+ state=([^ ]*)")
	if state == "unique" then
		local i = r.expect ("route-identity", { "index", "state", "scene", "scene_name", "kind", "kind_name", "caller_raw" })
		mine (i, "route-identity")
		if i.state ~= "unique" then malformed (what .. " has an identity state that is not the literal unique") end
		c.identity, c.scene, c.kind = "unique", int (i.scene, 0, INT_MAX, "identity scene"), int (i.kind, 0, INT_MAX, "identity kind")
		c.scene_name, c.kind_name = name (i.scene_name, "identity scene_name"), name (i.kind_name, "identity kind_name")
		c.caller_raw = int (i.caller_raw, 0, INT_MAX, "identity caller_raw")
		if c.caller_raw ~= c.legacy then malformed (what .. " names a caller whose number is not the parse's argument") end
		-- the unique identity IS the lookup row it names: the same key and names, found, and the raw number of that lookup is the parse's argument (other rows may share that number)
		local row = summary.lookup_rows [c.lookup]
		if not row or row.found ~= 1 or row.legacy ~= c.legacy or row.scene ~= c.scene or row.kind ~= c.kind or row.scene_name ~= c.scene_name or row.kind_name ~= c.kind_name then
			malformed (what .. " has a unique identity that is not the lookup row it names")
		end
	elseif state == "mismatch" or state == "unmatched" or state == "conflict" then
		local i = r.expect ("route-identity", { "index", "state", "caller_raw" })
		mine (i, "route-identity")
		if i.state ~= state then malformed (what .. " has an identity state that is not the literal " .. state) end
		c.identity = state
		c.caller_raw = int (i.caller_raw, 0, INT_MAX, "identity caller_raw")
		-- the number is compared with the parse's argument before any lookup is searched, so only a mismatch may carry a number that differs from it
		if state ~= "mismatch" and c.caller_raw ~= c.legacy then malformed (what .. " is " .. state .. " with a caller number that is not the parse's argument") end
		if c.lookup ~= -1 then malformed (what .. " has no unique identity and still names a lookup row") end
	elseif state == "missing" then
		mine (r.expect ("route-identity", { "index", "state" }), "route-identity")
		c.identity = "missing"
		if c.lookup ~= -1 then malformed (what .. " has no caller context and still names a lookup row") end
	else
		malformed (what .. " has no route-identity row with a known state")
	end

	-- The bounds row is the original parser's object's bounding box: the original grammar has it and the HOST profile has none (the campaign never holds the bounds), so a host
	-- report that carries one is refused here, as the reader expects the next row to be route-dropped.
	if summary.profile ~= "host" then
		local b = r.expect ("route-bounds", { "index", "bits" })
		mine (b, "route-bounds")
		words (b.bits, 6, "bounds")
	end
	local d = r.expect ("route-dropped", { "index", "slots", "nodes", "faces", "finals", "links", "starts", "depths", "offsets" })
	mine (d, "route-dropped")
	c.dropped = 0
	for _, key in ipairs { "slots", "nodes", "faces", "finals", "links", "starts", "depths", "offsets" } do c.dropped = c.dropped + int (d [key], 0, INT_MAX, "dropped " .. key) end
	local t = r.expect ("route-totals", { "index", "have", "nodes", "starts", "depths", "maximum_depth", "rows_nodes", "rows_faces", "rows_finals", "rows_links", "rows_starts", "rows_depths" })
	mine (t, "route-totals")
	c.have, c.total_nodes, c.total_starts, c.total_depths = flag (t.have, "totals have"), int (t.nodes, 0, INT_MAX, "totals nodes"), int (t.starts, 0, INT_MAX, "totals starts"), int (t.depths, 0, INT_MAX, "totals depths")
	c.maximum_depth = int (t.maximum_depth, -INT_MAX, INT_MAX, "maximum_depth")
	local rows = { nodes = int (t.rows_nodes, 0, CAP.nodes, "rows_nodes"), faces = int (t.rows_faces, 0, CAP.faces, "rows_faces"), finals = int (t.rows_finals, 0, CAP.finals, "rows_finals"),
		links = int (t.rows_links, 0, CAP.links, "rows_links"), starts = int (t.rows_starts, 0, CAP.starts, "rows_starts"), depths = int (t.rows_depths, 0, CAP.depths, "rows_depths"), slots = c.slots_stored, offsets = 0 }
	c.rows = rows
	local k = r.expect ("route-capture", { "index", "expected_faces", "expected_links", "bad_links", "offset_shortfall" })
	mine (k, "route-capture")
	local expected_faces, expected_links = int (k.expected_faces, 0, INT_MAX, "expected_faces"), int (k.expected_links, 0, INT_MAX * 2, "expected_links")
	local bad_links, shortfall = int (k.bad_links, 0, INT_MAX * 2, "bad_links"), int (k.offset_shortfall, 0, INT_MAX, "offset_shortfall")

	-- the rows, in the order the observer writes them; each is read against the count declared for it
	for i = 0, rows.slots - 1 do
		local v = r.expect ("route-slot", { "index", "slot", "bits" })
		mine (v, "route-slot")
		if int (v.slot, 0, INT_MAX, "slot") ~= i then malformed (what .. " slot rows are not in order at " .. i) end
		words (v.bits, 3, "slot bits")
	end
	local node_numbers = {}
	local host = summary.profile == "host"
	for i = 1, rows.nodes do
		-- the host profile has no stored short: the node is the host's decoded position
		local v
		if host then v = r.expect ("route-node", { "index", "node", "bits" }) else v = r.expect ("route-node", { "index", "node", "raw", "bits" }) end
		mine (v, "route-node")
		node_numbers [i] = int (v.node, 0, INT_MAX, "node")
		if not host then ints (v.raw, 3, -32768, 32767, "raw point") end
		words (v.bits, 3, "decoded bits")
	end
	local faces = {}
	for i = 1, rows.faces do
		if host then
			-- the host profile has the flat ordered faces only: the two resolved node numbers, the colour and the polygon bit
			local v = r.expect ("route-face", { "index", "face", "point", "colour", "polygons" })
			mine (v, "route-face")
			faces [i] = { face = int (v.face, 0, INT_MAX, "face number"), point = ints (v.point, 2, -INT_MAX, INT_MAX, "face resolved nodes"),
				colour = ints (v.colour, 3, 0, 255, "face colour"), polygons = flag (v.polygons, "face polygons") }
		else
			local v = r.expect ("route-face", { "index", "surface", "face", "local", "point", "colour", "polygons", "flags", "surface_points", "surface_faces" })
			mine (v, "route-face")
			faces [i] = { surface = int (v.surface, 0, INT_MAX, "face surface"), face = int (v.face, 0, INT_MAX, "face number"), local_refs = ints (v ["local"], 2, -INT_MAX, INT_MAX, "face local refs"),
				colour = ints (v.colour, 3, 0, 255, "face colour"), polygons = flag (v.polygons, "face polygons"), flags = word (v.flags, "face flags"),
				point = ints (v.point, 2, -INT_MAX, INT_MAX, "face resolved nodes"), surface_points = int (v.surface_points, 0, 256, "surface_points"), surface_faces = int (v.surface_faces, 0, 65535, "surface_faces") }
		end
	end
	local finals = {}
	for i = 1, rows.finals do
		local v = r.expect ("route-final", { "index", "node", "depth", "tree", "node_index", "primary", "start", "possible_start", "references", "in_route", "processed", "children", "parents" })
		mine (v, "route-final")
		finals [i] = { node = int (v.node, 0, INT_MAX, "final node"), children = int (v.children, 0, INT_MAX, "final children"), parents = int (v.parents, 0, INT_MAX, "final parents") }
		for _, key in ipairs { "depth", "tree", "node_index" } do int (v [key], -INT_MAX, INT_MAX, "final " .. key) end
		for _, key in ipairs { "primary", "start", "possible_start", "in_route", "processed" } do flag (v [key], "final " .. key) end
		int (v.references, 0, INT_MAX, "final references")
	end
	local links = {}
	for i = 1, rows.links do
		local v = r.expect ("route-link", { "index", "node", "link", "slot", "target" })
		mine (v, "route-link")
		if v.link ~= "child" and v.link ~= "parent" then malformed (what .. " has a link that is neither child nor parent: " .. tostring (v.link)) end
		links [i] = { node = int (v.node, 0, INT_MAX, "link node"), parent = v.link == "parent", slot = int (v.slot, 0, INT_MAX, "link slot"), target = int (v.target, -1, INT_MAX, "link target") }
	end
	local starts = {}
	for i = 1, rows.starts do
		local v = r.expect ("route-start", { "index", "slot", "node" })
		mine (v, "route-start")
		starts [i] = { slot = int (v.slot, 0, INT_MAX, "start slot"), node = int (v.node, -1, INT_MAX, "start node") }
	end
	local depths = {}
	for i = 1, rows.depths do
		local v = r.expect ("route-position", { "index", "depth", "count", "stored", "bits" })
		mine (v, "route-position")
		local depth = { depth = int (v.depth, 0, INT_MAX, "position depth"), count = int (v.count, -INT_MAX, INT_MAX, "position count"), stored = int (v.stored, 0, CAP.offsets, "position stored") }
		words (v.bits, 3, "position bits")
		for j = 0, depth.stored - 1 do
			local o = r.expect ("route-offset", { "index", "depth", "slot", "bits" })
			mine (o, "route-offset")
			if int (o.depth, 0, INT_MAX, "offset depth") ~= depth.depth or int (o.slot, 0, INT_MAX, "offset slot") ~= j then malformed (what .. " offset rows are not in order under depth " .. depth.depth) end
			words (o.bits, 3, "offset bits")
		end
		rows.offsets = rows.offsets + depth.stored
		depths [i] = depth
	end
	mine (r.expect ("route-end", { "index" }), "route-end")
	r.finish ()

	-- A complete=1 claim is only believed if every cohort it promises is in the rows, against the call's own counts and the order the observer writes.
	if c.complete == 1 then
		local function unsupported (reason) malformed (what .. " claims complete=1 but " .. reason) end
		if c.ended ~= 1 or c.interrupted ~= 0 or c.have ~= 1 or c.dropped ~= 0 then unsupported ("it is unended, interrupted, without totals or dropped rows") end
		if c.identity ~= "unique" then unsupported ("its route identity is " .. c.identity) end
		local lookup = summary.lookup_rows [c.lookup]
		if not lookup or lookup.found ~= 1 or lookup.legacy ~= c.legacy or lookup.scene ~= c.scene or lookup.kind ~= c.kind or lookup.scene_name ~= c.scene_name or lookup.kind_name ~= c.kind_name then
			unsupported ("its identity is not the lookup row it names")
		end
		if c.slots < 0 or c.slots_stored ~= c.slots then unsupported ("the requested slots were not all captured") end
		if rows.nodes ~= c.points or c.total_nodes ~= c.points then unsupported ("the node rows are not the object's points") end
		for i = 1, rows.nodes do if node_numbers [i] ~= i - 1 then unsupported ("the node rows are not numbered in order") end end
		if rows.faces ~= expected_faces then unsupported ("the first-pass faces are not the faces its surfaces promise") end
		if host then
			-- the host's faces are numbered in order and name nodes of the host's own table (there is no surface structure to join: the host resolved it)
			for i = 1, rows.faces do
				local f = faces [i]
				if f.face ~= i - 1 then unsupported ("the line faces are not numbered in order") end
				for _, node in ipairs (f.point) do if node < 0 or node >= c.points then unsupported ("a face names a node outside the table") end end
			end
		else
			local previous, resolved
			for i = 1, rows.faces do
				local f = faces [i]
				if f.surface >= c.surfaces or f.surface_points < 1 then unsupported ("a face is outside its object") end
				for _, ref in ipairs (f.local_refs) do if ref < 0 or ref >= f.surface_points then unsupported ("a face-local reference is outside its surface's point range") end end
				for _, node in ipairs (f.point) do if node < 0 or node >= c.points then unsupported ("a face names a node outside the table") end end
				if previous and previous.surface == f.surface then
					if f.face ~= previous.face + 1 or f.surface_faces ~= previous.surface_faces then unsupported ("the faces of a surface are not in order") end
					-- the hook records the SAME surface record for every face of a surface: its colour, flags, polygon bit and point count cannot differ between faces
					if f.flags ~= previous.flags or f.polygons ~= previous.polygons or f.surface_points ~= previous.surface_points or f.colour [1] ~= previous.colour [1] or
							f.colour [2] ~= previous.colour [2] or f.colour [3] ~= previous.colour [3] then
						unsupported ("two faces of one surface carry different surface records")
					end
				else
					if previous and (f.surface <= previous.surface or previous.face + 1 ~= previous.surface_faces) then unsupported ("the surfaces are not in order or a surface was cut short") end
					if f.face ~= 0 then unsupported ("a surface does not begin at its first face") end
					resolved = {}
				end
				-- within one surface a face-local reference names ONE node: the same reference seen again must have resolved to the same node
				for k = 1, 2 do
					local seen = resolved [f.local_refs [k]]
					if seen ~= nil and seen ~= f.point [k] then unsupported ("a face-local reference resolves to two different nodes within one surface") end
					resolved [f.local_refs [k]] = f.point [k]
				end
				previous = f
			end
			if previous and previous.face + 1 ~= previous.surface_faces then unsupported ("the last surface was cut short") end
		end
		if rows.finals ~= c.total_nodes then unsupported ("the final rows are not the totals' nodes") end
		local promised = 0
		for i = 1, rows.finals do
			if finals [i].node ~= i - 1 then unsupported ("the final rows are not numbered in order") end
			promised = promised + finals [i].children + finals [i].parents
		end
		if bad_links ~= 0 or rows.links ~= promised or promised ~= expected_links then unsupported ("the links are not the links the final rows count") end
		local at = 0
		for i = 1, rows.finals do
			for _, which in ipairs { false, true } do
				for slot = 0, (which and finals [i].parents or finals [i].children) - 1 do
					at = at + 1
					local l = links [at]
					if l.node ~= i - 1 or l.parent ~= which or l.slot ~= slot or l.target < 0 or l.target >= c.total_nodes then unsupported ("a link is out of order or points outside the table") end
				end
			end
		end
		if rows.starts ~= c.total_starts then unsupported ("the start rows are not the totals' starts") end
		for i = 1, rows.starts do
			if starts [i].slot ~= i - 1 or starts [i].node < 0 or starts [i].node >= c.total_nodes then unsupported ("a start is out of order or outside the table") end
		end
		if rows.depths ~= c.total_depths or c.total_depths ~= c.maximum_depth + 1 then unsupported ("the depth rows are not the totals' depths") end
		if shortfall ~= 0 then unsupported ("a depth lacks offsets") end
		for i = 1, rows.depths do
			if depths [i].depth ~= i - 1 or depths [i].count ~= depths [i].stored then unsupported ("a depth is out of order or lacks offsets its count declares") end
		end
	end
	return c
end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the run. The output is created here and is closed on every exit.

local out_file, why = host.create_exclusive (out_path)
if not out_file then error ("route-characterization: " .. tostring (why), 0) end
local out = json.wrap (out_file, out_path)
local function emit (row) out.emit (row) end

local function first_line (message) return (string.match (tostring (message), "^[^\n]*")) end

-- a protocol error (malformed evidence) is written, the run is closed as not complete, and it fails; any other error (an output failure) is passed on as it is
local function protocol_failure (err)
	if type (err) == "table" and err.malformed then
		emit { event = "malformed", reason = err.malformed }
		emit { event = "done", complete = false, malformed = true }
		error ("route-characterization: MALFORMED REPORT: " .. err.malformed .. " (the engine is not asked again)", 0)
	end
	error (err, 0)
end

local function run ()
	local dc = require ("eech_dc")
	dc.prepare_installation (root)
	-- the reference host's arithmetic profile, selected right after the installation is prepared and before the boot (src/reference_arithmetic.rs)
	host.use_reference_arithmetic ()

	emit { event = "header", scenario = scenario_name, seed = 1, frames = 0, observer = "route-fixture", note = "natural boot only; no frame is run" }

	local ok, engine = pcall (dc.boot, {
		install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
		seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
		supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, model = host.model, sense = host.sense, aim = host.aim,
	})
	if not ok then
		local message = first_line (engine)
		emit { event = "boot_failed", error = message }
		error ("route-characterization: boot failed: " .. message, 0)
	end
	emit { event = "boot", booted = true }

	-- one question to the engine; a failure stops all questions (the engine may be poisoned)
	local function ask (what, call)
		local good, text = pcall (call)
		if not good then
			local message = first_line (text)
			emit { event = "report_failed", what = what, error = message }
			error ("route-characterization: " .. what .. " failed: " .. message .. " (the engine is not asked again)", 0)
		end
		if type (text) ~= "string" then
			emit { event = "report_failed", what = what, error = "the report is not a string" }
			error ("route-characterization: " .. what .. " returned a " .. type (text), 0)
		end
		return text
	end

	local summary_text = ask ("route_summary", function () return engine:route_summary () end)
	emit { event = "route_summary", text = summary_text }
	local good, summary = pcall (check_summary, summary_text)
	if not good then protocol_failure (summary) end

	local calls, incomplete = {}, 0
	for index = 0, summary.calls - 1 do
		local text = ask ("route_call " .. index, function () return engine:route_call (index) end)
		emit { event = "route_call", index = index, text = text }
		local checked, call = pcall (check_call, text, index, summary)
		if not checked then protocol_failure (call) end
		calls [index] = call
		if call.complete == 0 then incomplete = incomplete + 1 end
	end

	-- the summary and the calls must agree with each other
	local joined, reason = pcall (function ()
		-- every context registered is consumed by exactly one parse or is stale; a parse without a context is the missing identity (calls that were dropped for capacity are not visible, so
		-- they widen the bound)
		local missing = 0
		for index = 0, summary.calls - 1 do if calls [index].identity == "missing" then missing = missing + 1 end end
		local consumed = summary.contexts - summary.stale
		if consumed + missing < summary.calls or consumed + missing > summary.calls + summary.dropped_calls then
			malformed (string.format ("the summary registered %d contexts (%d stale) for %d calls (%d without a context, %d dropped)", summary.contexts, summary.stale, summary.calls, missing, summary.dropped_calls))
		end
		if incomplete ~= summary.incomplete_calls then malformed (string.format ("the summary counts %d incomplete calls, the calls say %d", summary.incomplete_calls, incomplete)) end
		local sum = { slots = 0, nodes = 0, faces = 0, finals = 0, links = 0, starts = 0, depths = 0, offsets = 0 }
		for index = 0, summary.calls - 1 do
			for key in pairs (sum) do sum [key] = sum [key] + calls [index].rows [key] end
		end
		for key, total in pairs (sum) do
			if summary.pools [key] ~= total then malformed (string.format ("the summary's %s pool holds %d rows, the calls hold %d", key, summary.pools [key], total)) end
		end
	end)
	if not joined then protocol_failure (reason) end

	local measured = summary.complete == 1 and summary.calls > 0 and incomplete == 0
	emit { event = "done", complete = measured, calls = summary.calls, incomplete_calls = incomplete, summary_complete = summary.complete == 1, malformed = false }
	if not measured then
		if summary.calls == 0 then error ("route-characterization: NOT MEASURED: the boot recorded no route parser call", 0) end
		error (string.format ("route-characterization: NOT COMPLETE: summary complete=%s, %d of %d recorded calls incomplete", tostring (summary.complete == 1), incomplete, summary.calls), 0)
	end
end

local ran, err = pcall (run)
local closed, close_err = pcall (out.close)
if not ran then
	if not closed then err = tostring (err) .. "; additionally the output could not be closed: " .. first_line (close_err) end
	error (err, 0)
end
if not closed then error ("route-characterization: the output could not be closed: " .. first_line (close_err), 0) end
