-- TEST-ONLY public baseline observation of the sector tallest-structure word (EEDCS-49, D3, first substage of the host-owned sector-height extraction; plan:
-- docs/d3-sector-height-public-baseline-plan.md). Needs a DLL built with `eech-dc/sector-height-observer`; the observer is read-only and changes no campaign behaviour.
--
-- Boundary under observation (the product path, not a private test): this host-owned Lua -> the public native module (`require ("eech_dc")`, `dc.boot`, `engine:frame`) -> the real en_list
-- funnel, the sector response, the host roster statements, the hosted scan and candidate questions, the fixed-wing clearance and the session pack, all of the ordinary campaign.
-- The scenario is the ordinary campaign driver's own boot and frame loop (lua/campaign.lua, the same dc.boot arguments), without recording or metrics. Nothing is staged or tuned: which
-- fixed families insert and in what order, whether a real city arrives with a valid shape, whether the scan and candidate questions are asked, whether the guarded clearance is reached and
-- whether a save or restore happens are the OBSERVATION. A case that did not happen is reported with its counter at zero (UNREACHED), never credited. A configured query is not a natural
-- engagement and none is made here.
--
-- host.args: root (installation root), scenario (georgia, georgia_retail, lebanon_retail), map, campaign, hours (default 0.001: the established finite ordinary short window, 3.6 simulated seconds;
--            a larger value needs its own named, bounded plan and is never chosen by what the observation reached), frame_ms (default 100), seed (default 1),
--            out (the observation file, required; created EXCLUSIVELY before the boot, which changes the working directory)
--            scope (default `original`: the observation of the campaign-held word, whose hooks live in the ORIGINAL profile; or `hosted-scan`, the scoped observation of a HOSTED build, below)
--            accepted (only with scope=hosted-scan: the accepted original report to compare the scan rows with, read-only; never copied)
--
-- SCOPE hosted-scan (Task49, the host-owned sector height). In a hosted build the original raise (SH2/SH3), the original save and restore (SH4/SH5) and the campaign-held word are gone: the link, event, shape, clearance-word and
-- session records of the original are NOT comparable and are not written. What the observer still sees of the HOST's word is the scan: the word the scan's getter was answered at each FIRST (SH10) and the fact 12 of every
-- offer (SH11), both per sector. This scope writes ONLY: the identity line (with `scope=hosted-scan`), the observer's summary lines (the original-link counters among them, labelled by the verdict as not compared),
-- the clearance records (the guard counters and the min_height AFTER the original statement; the consumed word is `tallest=uncaptured`, never re-queried) and the per-sector `sho-scan` rows. With `accepted=` it compares, by sector
-- (sx, sz) and at the SAME window (scenario, seed, frame_ms, hours, frames), these fields of the hosted rows with the accepted report's: reads, first_word, last_word, changes, fact_offers, fact_equal_first, fact_differ_first,
-- fact_first_word, fact_last_word, fact_changes, and the totals of the `sho-scan` summary line. Not compared: sequence numbers, insert/link/statement joins, shape states, the session and anything the hooks of the original raise
-- recorded. Statuses (`sho-compare`): EQUAL_AT_SCOPE, DIFFERENT, NOT_COMPARABLE (the window differs or the accepted report cannot be read), UNREACHED (no scan FIRST in the window). Anything but EQUAL_AT_SCOPE (or, without
-- `accepted`, a scan that is whole with at least one read) ends the process non-zero after the observation is closed; nothing is searched, extended or retried to reach a green.
-- BOTH SIDES ARE VALIDATED AS A WHOLE SCOPED SCAN BEFORE ANYTHING IS COMPARED (ENV-SECTOR-HEIGHT-SCOPED-REPORT-COMPLETE-1): the accepted ORIGINAL report must hold exactly one identity line, one `sho-scan` summary, one
-- `sho-tables` line and one `sho-verdict whole=1 events= clearances= sectors=` line, that verdict LAST and its sector count the rows'; both sides need the exact scoped fields and domains, the declared sector count, unique
-- contiguous row indices from 0, unique sector keys, `firsts` equal to `reads`, no fact-12 offer before a FIRST, `scan_out_of_range=0`, the rows' reads and fact-12 counts summing to the totals and each row's words
-- consistent with its changes. Never required of the hosted side: the original-link `sho-complete value=1`, the events, the raise or the session. A malformed or incomplete accepted report is INVALID_ACCEPTED (an
-- unreadable one NOT_COMPARABLE); a malformed or incomplete live hosted scan is INVALID_HOSTED and `scan_whole=0`; UNREACHED only for a clean, whole scan with no read. Duplicates are refused, never collapsed.
--
-- Output: one `key=value` line grammar. The first line is the run's identity; then the observer's summary lines, every recorded event, every fixed-wing clearance record and every sector
-- aggregate, then one verdict line. The driver REFUSES (non-zero exit) an observation that is not whole: the observer's own completeness verdict, one row per counted record, increasing
-- sequence numbers. That is integrity of the record only; it asserts nothing about the campaign's heights.

local args = host.args
local root = assert (args.root, "root=<installation root>")
local out_path = assert (args.out, "out=<observation file>")
local hours = tonumber (args.hours or "0.001")
local frame_ms = tonumber (args.frame_ms or "100")
local seed = tonumber (args.seed or "1")
assert (hours and hours >= 0 and frame_ms and frame_ms > 0 and seed, "hours, frame_ms and seed must be numbers")
local scope = args.scope or "original"
assert (scope == "original" or scope == "hosted-scan", "scope=original|hosted-scan")
local hosted = scope == "hosted-scan"
local accepted_path = args.accepted
assert (hosted or accepted_path == nil, "accepted=<report> is only for scope=hosted-scan")

local scenarios = {
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia = { map = "..\\common\\maps\\map16", campaign = "georgia.chc" },
}
local scenario_name = args.scenario or "georgia"
local scenario = assert (scenarios[scenario_name], "scenario=georgia|georgia_retail|lebanon_retail")

-- the observation is created exclusively (src/exclusive_output.rs): an existing observation, partial report or log of any kind is never overwritten or truncated
if type (host.create_exclusive) ~= "function" then error ("this host has no create_exclusive: an output that may never overwrite anything cannot be made safely", 0) end
local out, why = host.create_exclusive (out_path)
if not out then error ("sector-height-baseline: cannot create " .. out_path .. " exclusively: " .. tostring (why), 0) end

local function emit (line)
	assert (out:write (line .. "\n"))
end

local function field (line, key)
	return line:match ("%f[%w_]" .. key .. "=([^%s]+)")
end

local function pairs_of (line)
	local t = {}
	for k, v in line:gmatch ("%f[%w_]([%w_]+)=([^%s]+)") do t [k] = v end
	return t
end

-- one report line as strict key=value pairs (the first token is the line's name): a token that is not key=value, or a key given twice, is refused (nil and why), never collapsed
local function strict_pairs (line)
	local t, first = {}, true
	for token in line:gmatch ("%S+") do
		if first then
			first = false
		else
			local k, v = token:match ("^([%w_]+)=(.+)$")
			if not k then return nil, "the token \"" .. token .. "\" is not key=value" end
			if t [k] ~= nil then return nil, "the key " .. k .. " is given twice" end
			t [k] = v
		end
	end
	return t
end

local function whole_number (text)
	if type (text) ~= "string" or not text:match ("^%d+$") or #text > 15 then return nil end
	return tonumber (text)
end

local function word_pattern (text)
	return type (text) == "string" and text:match ("^0x%x%x%x%x%x%x%x%x$") ~= nil
end

local WINDOW_FIELDS = { "scenario", "seed", "frame_ms", "hours", "frames" }
local ROW_INTS = { "i", "sx", "sz", "reads", "first_seq", "last_seq", "changes", "fact_offers", "fact_equal_first", "fact_differ_first", "fact_changes" }
local ROW_WORDS = { "first_word", "last_word", "fact_first_word", "fact_last_word" }
local ROW_FIELDS = { "reads", "first_word", "last_word", "changes", "fact_offers", "fact_equal_first", "fact_differ_first", "fact_first_word", "fact_last_word", "fact_changes" }
local TOTAL_FIELDS = { "reads", "firsts", "sectors", "fact_offers", "fact_nonzero", "fact_equal_first", "fact_differ_first", "fact_without_first" }

-- THE WHOLE SCOPED SCAN of one side (identity, the `sho-scan` totals, the `sho-tables` line and the per-sector rows in the order they were written): the exact scoped fields and domains, the declared sector count, unique contiguous
-- row indices, unique sector keys, and the totals and words consistent with the rows. It asks for nothing the HOSTED scope does not keep: no original-link completeness, no event, raise or session. Returns the checked scan
-- (numbers and words), or nil and why. A scan that fails here is never compared and never counted whole.
local function validate_scan (identity, totals, tables, rows)
	if type (identity) ~= "table" or identity.schema ~= "1" or identity.observer ~= "read-only" then return nil, "the identity line is not schema 1 and read-only" end
	if type (identity.scenario) ~= "string" or not identity.scenario:match ("^[%w_]+$") then return nil, "the identity names no scenario" end
	if not (whole_number (identity.seed) and whole_number (identity.frame_ms) and whole_number (identity.frames)) then return nil, "the identity's seed, frame_ms or frames is not a whole number" end
	if type (identity.hours) ~= "string" or not identity.hours:match ("^%d+%.?%d*$") then return nil, "the identity's hours is not a number" end
	local t = {}
	for _, key in ipairs (TOTAL_FIELDS) do
		t [key] = type (totals) == "table" and whole_number (totals [key]) or nil
		if t [key] == nil then return nil, "the sho-scan total " .. key .. " is missing or not a whole number" end
	end
	if type (tables) ~= "table" or whole_number (tables.scan_out_of_range) ~= 0 then return nil, "the sho-tables line does not state scan_out_of_range=0" end
	if t.firsts ~= t.reads then return nil, "the scan totals state " .. t.reads .. " reads but " .. t.firsts .. " firsts" end
	if t.fact_without_first ~= 0 then return nil, "the scan totals state " .. t.fact_without_first .. " fact-12 offers made before any FIRST" end
	if t.fact_nonzero > t.fact_offers then return nil, "more nonzero fact-12 offers than offers" end
	if t.fact_offers ~= t.fact_equal_first + t.fact_differ_first + t.fact_without_first then return nil, "the fact-12 offers are not equal + different + without a first" end
	if #rows ~= t.sectors then return nil, "the sho-scan total declares " .. t.sectors .. " sectors but " .. #rows .. " rows were written" end
	local by_sector, sum = {}, { reads = 0, fact_offers = 0, fact_equal_first = 0, fact_differ_first = 0 }
	for position, row in ipairs (rows) do
		local n = {}
		for _, key in ipairs (ROW_INTS) do
			n [key] = whole_number (row [key])
			if n [key] == nil then return nil, "row " .. (position - 1) .. ": the field " .. key .. " is missing or not a whole number" end
		end
		for _, key in ipairs (ROW_WORDS) do
			if not word_pattern (row [key]) then return nil, "row " .. (position - 1) .. ": the field " .. key .. " is missing or not a 0x%08x pattern" end
		end
		if n.i ~= position - 1 then return nil, "row index " .. n.i .. " stands at position " .. (position - 1) .. ": the indices are not unique and contiguous from 0" end
		local key = row.sx .. "," .. row.sz
		if by_sector [key] then return nil, "the sector " .. key .. " is written twice" end
		if n.reads < 1 or n.first_seq > n.last_seq or n.changes > n.reads - 1 then return nil, "row " .. n.i .. ": reads, sequence numbers and changes are inconsistent" end
		if n.changes == 0 and row.first_word ~= row.last_word then return nil, "row " .. n.i .. ": no change is stated but the first and last words differ" end
		if n.fact_equal_first + n.fact_differ_first ~= n.fact_offers then return nil, "row " .. n.i .. ": the fact-12 offers are not equal + different" end
		if n.fact_offers == 0 and (n.fact_changes ~= 0 or row.fact_first_word ~= "0x00000000" or row.fact_last_word ~= "0x00000000") then return nil, "row " .. n.i .. ": a row with no fact-12 offer states fact words or changes" end
		if n.fact_offers > 0 and (n.fact_changes > n.fact_offers - 1 or (n.fact_changes == 0 and row.fact_first_word ~= row.fact_last_word)) then return nil, "row " .. n.i .. ": the fact-12 changes and words are inconsistent" end
		sum.reads, sum.fact_offers = sum.reads + n.reads, sum.fact_offers + n.fact_offers
		sum.fact_equal_first, sum.fact_differ_first = sum.fact_equal_first + n.fact_equal_first, sum.fact_differ_first + n.fact_differ_first
		by_sector [key] = row
	end
	if sum.reads ~= t.reads then return nil, "the rows hold " .. sum.reads .. " reads, the total states " .. t.reads end
	if sum.fact_offers ~= t.fact_offers or sum.fact_equal_first ~= t.fact_equal_first or sum.fact_differ_first ~= t.fact_differ_first then return nil, "the rows' fact-12 counts are not the totals'" end
	return { identity = identity, totals = totals, rows = by_sector, reads = t.reads }
end

-- the accepted ORIGINAL report, streamed (it is large: only the lines this scope uses are kept): exactly one identity line, one `sho-scan` summary, one `sho-tables` line and one `sho-verdict` line, the verdict LAST
-- and `whole=1` with the declared sector count, the rows validated as a whole scan. Returns the checked scan, or nil and why ("open" when the file cannot be opened at all)
local function load_accepted (path)
	local file, why = io.open (path, "rb")
	if not file then return nil, "cannot open " .. tostring (path) .. ": " .. tostring (why), true end
	local identity, totals, tables, verdicts, rows, last, bad = {}, {}, {}, {}, {}, nil, nil
	for line in file:lines () do
		line = line:gsub ("\r$", "")
		if line ~= "" then last = line end
		local bucket
		if line:sub (1, 13) == "sho-baseline " then bucket = identity
		elseif line:sub (1, 15) == "sho-scan reads=" then bucket = totals
		elseif line:sub (1, 11) == "sho-scan i=" then bucket = rows
		elseif line:sub (1, 11) == "sho-tables " then bucket = tables
		elseif line:sub (1, 12) == "sho-verdict " then bucket = verdicts
		end
		if bucket then
			if bucket == verdicts then
				verdicts [#verdicts + 1] = line
			else
				local pairs_line, problem = strict_pairs (line)
				if not pairs_line then bad = bad or (line:sub (1, 20) .. ": " .. problem) else bucket [#bucket + 1] = pairs_line end
			end
		end
	end
	file:close ()
	if bad then return nil, "a scoped line is malformed: " .. bad end
	if #identity ~= 1 or #totals ~= 1 or #tables ~= 1 or #verdicts ~= 1 then
		return nil, string.format ("the report holds %d identity, %d sho-scan summary, %d sho-tables and %d sho-verdict lines, not one of each", #identity, #totals, #tables, #verdicts)
	end
	if identity [1].scope ~= nil then return nil, "the accepted report is not an original report (it states a scope)" end
	local events, clearances, sectors = verdicts [1]:match ("^sho%-verdict whole=1 events=(%d+) clearances=(%d+) sectors=(%d+)$")
	if not sectors then return nil, "the report has no complete terminal verdict (sho-verdict whole=1 events= clearances= sectors=): it is truncated or not whole" end
	if last ~= verdicts [1] then return nil, "the verdict is not the last line of the report" end
	local scan, why_scan = validate_scan (identity [1], totals [1], tables [1], rows)
	if not scan then return nil, why_scan end
	if tonumber (sectors) ~= #rows then return nil, "the verdict declares " .. sectors .. " sectors but " .. #rows .. " rows were written" end
	return scan
end

-- the scoped comparison of the hosted scan (already validated) with the accepted report: returns the status, the reason and the list of differences (field lines); see the header of this file for exactly what is compared
local function compare_scan (hosted_scan, accepted)
	local report, why, unreadable = load_accepted (accepted)
	if not report then return unreadable and "NOT_COMPARABLE" or "INVALID_ACCEPTED", why, {} end
	for _, key in ipairs (WINDOW_FIELDS) do
		if tostring (hosted_scan.identity [key]) ~= tostring (report.identity [key]) then
			return "NOT_COMPARABLE", "the window differs at " .. key .. ": hosted " .. tostring (hosted_scan.identity [key]) .. ", accepted " .. tostring (report.identity [key]), {}
		end
	end
	local differences = {}
	for _, key in ipairs (TOTAL_FIELDS) do
		if hosted_scan.totals [key] ~= report.totals [key] then
			differences [#differences + 1] = string.format ("sho-compare-diff total=%s hosted=%s accepted=%s", key, tostring (hosted_scan.totals [key]), tostring (report.totals [key]))
		end
	end
	for sector, row in pairs (hosted_scan.rows) do
		local other = report.rows [sector]
		if not other then
			differences [#differences + 1] = string.format ("sho-compare-diff sector=%s field=row hosted=present accepted=absent", sector)
		else
			for _, key in ipairs (ROW_FIELDS) do
				if row [key] ~= other [key] then
					differences [#differences + 1] = string.format ("sho-compare-diff sector=%s field=%s hosted=%s accepted=%s", sector, key, tostring (row [key]), tostring (other [key]))
				end
			end
		end
	end
	for sector in pairs (report.rows) do
		if not hosted_scan.rows [sector] then
			differences [#differences + 1] = string.format ("sho-compare-diff sector=%s field=row hosted=absent accepted=present", sector)
		end
	end
	table.sort (differences)
	if #differences > 0 then return "DIFFERENT", #differences .. " differences at scope", differences end
	return "EQUAL_AT_SCOPE", "every compared field of every sector and of the totals is equal at the same window", {}
end

local function run ()
	local dc = require ("eech_dc")
	dc.prepare_installation (root)
	host.use_reference_arithmetic ()

	local engine = dc.boot {
		install_root = root,
		map = args.map or scenario.map,
		terrain = host.terrain_provider (root, args.map or scenario.map), environment = host.env_provider (),
		campaign_directory = scenario.campaign_directory or "camp01",
		campaign = args.campaign or scenario.campaign,
		gunship = "apache",
		seed = seed,
		bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle,
		observe_loss = host.observe_loss,
		supply_reached = host.supply_reached,
		line_of_sight = host.line_of_sight, model = host.model,
		sense = host.sense, aim = host.aim,
	}
	if type (engine.sector_height_summary) ~= "function" then
		error ("the module has no sector_height_summary: it was built without feature eech-dc/sector-height-observer", 0)
	end
	host.log ("sector-height-baseline: campaign booted")

	local frames = math.floor (hours * 3600 * 1000 / frame_ms)
	for _ = 1, frames do
		engine:frame (frame_ms)
	end
	local clock = engine:clock ()

	local identity_line = string.format ("sho-baseline schema=1 scenario=%s seed=%d frame_ms=%s hours=%s frames=%d elapsed_seconds=%.3f observer=read-only", scenario_name, seed, tostring (frame_ms), tostring (hours), frames, clock.elapsed_seconds)
	emit (hosted and (identity_line .. " scope=hosted-scan") or identity_line)
	local summary = engine:sector_height_summary ()
	for line in summary:gmatch ("[^\n]+") do emit (line) end

	local function number_of (prefix, key)
		for line in summary:gmatch ("[^\n]+") do
			if line:sub (1, #prefix) == prefix then return tonumber (field (line, key)) end
		end
		error ("the observer summary has no " .. prefix .. " " .. key, 0)
	end
	-- the hosted scope reads its counts from the strictly parsed summary below (a malformed summary is INVALID_HOSTED, not an error of this lookup)
	local events, clearances, sectors
	if not hosted then
		events, clearances, sectors = number_of ("sho-tables", "events_used"), number_of ("sho-clearance", "stored"), number_of ("sho-scan", "sectors")
	end

	if hosted then
		-- the hosted-scan scope: no event row (the original-link diagnostics are not comparable and are not written); the clearance records and the scan rows only. The summary and every row are validated as a WHOLE
		-- scoped scan BEFORE anything is compared; a scan that is malformed or incomplete is INVALID_HOSTED and never scan_whole=1 or EQUAL_AT_SCOPE
		local totals, tables, clearance_line, problem
		local seen = { scan = 0, tables = 0, clearance = 0 }
		for line in summary:gmatch ("[^\n]+") do
			local name
			if line:sub (1, 15) == "sho-scan reads=" then name = "scan" elseif line:sub (1, 11) == "sho-tables " then name = "tables" elseif line:sub (1, 14) == "sho-clearance " then name = "clearance" end
			if name then
				seen [name] = seen [name] + 1
				local parsed, why = strict_pairs (line)
				if not parsed then problem = problem or ("the " .. name .. " line is malformed: " .. why)
				elseif name == "scan" then totals = parsed elseif name == "tables" then tables = parsed else clearance_line = parsed end
			end
		end
		if not problem and (seen.scan ~= 1 or seen.tables ~= 1 or seen.clearance ~= 1) then problem = string.format ("the summary holds %d sho-scan, %d sho-tables and %d sho-clearance lines, not one of each", seen.scan, seen.tables, seen.clearance) end
		local declared_sectors = totals and whole_number (totals.sectors)
		local declared_clearances = clearance_line and whole_number (clearance_line.stored)
		if not problem and (declared_sectors == nil or declared_clearances == nil) then problem = "the declared sector or clearance count is not a whole number" end
		local scan_rows = {}
		if not problem then
			for i = 0, declared_clearances - 1 do emit ((engine:sector_height_clearance (i):gsub ("\n$", ""))) end
			for i = 0, declared_sectors - 1 do
				local line = engine:sector_height_scan (i):gsub ("\n$", "")
				emit (line)
				local row, why = strict_pairs (line)
				if not row or line:sub (1, 11) ~= "sho-scan i=" then problem = problem or ("row " .. i .. " is malformed: " .. tostring (why or "it is not a sho-scan row")) else scan_rows [#scan_rows + 1] = row end
			end
		end
		local identity = pairs_of (identity_line)
		identity.frames = tostring (frames)
		local hosted_scan, why_scan
		if not problem then hosted_scan, why_scan = validate_scan (identity, totals, tables, scan_rows) end
		local status, reason, differences
		if problem or not hosted_scan then
			status, reason, differences = "INVALID_HOSTED", problem or why_scan, {}
		elseif hosted_scan.reads == 0 then
			status, reason, differences = "UNREACHED", "the hosted run made no scan FIRST read in the window (a clean, whole scan with no read)", {}
		elseif accepted_path then
			status, reason, differences = compare_scan (hosted_scan, accepted_path)
		else
			status, reason, differences = "SCAN_ONLY", "no accepted report was given: the scan record is whole, and nothing was compared", {}
		end
		local scan_whole = hosted_scan ~= nil and hosted_scan.reads >= 1
		emit (string.format ("sho-compare scope=scan-reads-and-fact12 status=%s sectors_hosted=%d reason=%s differences=%d", status, declared_sectors or 0, (tostring (reason):gsub ("%s", "_")), #differences))
		for _, line in ipairs (differences) do emit (line) end
		emit (string.format ("sho-verdict scope=hosted-scan scan_whole=%d status=%s clearances=%d sectors=%d original_link_pack_hooks=not_compared", scan_whole and 1 or 0, status, declared_clearances or 0, declared_sectors or 0))
		host.log (string.format ("sector-height-baseline: hosted-scan %s; %d clearance records, %d sectors", status, declared_clearances or 0, declared_sectors or 0))
		if status ~= "EQUAL_AT_SCOPE" and status ~= "SCAN_ONLY" then error ("hosted-scan comparison " .. status .. ": " .. tostring (reason), 0) end
		if not scan_whole then error ("the hosted scan record is not whole", 0) end
		return
	end

	local last_seq = 0
	for i = 0, events - 1 do
		local line = engine:sector_height_event (i):gsub ("\n$", "")
		local seq = tonumber (field (line, "seq"))
		if not seq or seq <= last_seq then error ("event " .. i .. ": the sequence number does not increase: " .. line, 0) end
		last_seq = seq
		emit (line)
	end
	for i = 0, clearances - 1 do emit ((engine:sector_height_clearance (i):gsub ("\n$", ""))) end
	for i = 0, sectors - 1 do emit ((engine:sector_height_scan (i):gsub ("\n$", ""))) end

	if not summary:find ("sho-complete value=1", 1, true) then
		error ("the observation is not whole (the observer's own verdict is sho-complete value=0): a table overflowed or a hook pairing broke", 0)
	end
	emit (string.format ("sho-verdict whole=1 events=%d clearances=%d sectors=%d", events, clearances, sectors))
	host.log (string.format ("sector-height-baseline: whole; %d events, %d clearance records, %d sectors", events, clearances, sectors))
end

local ran, failure = xpcall (run, function (e) return e end)
local closed, close_error = out:close ()
if not ran then
	if not closed then io.stderr:write ("sector-height-baseline: OUTPUT FAILURE while closing after the failure below: " .. tostring (close_error) .. "\n") end
	error (failure, 0)
end
if not closed then error ("sector-height-baseline: OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error), 0) end
