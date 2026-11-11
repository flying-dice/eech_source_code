"""ENV-SECTOR-HEIGHT-SCOPED-REPORT-COMPLETE-1: the scoped comparison validates both sides as a WHOLE scoped scan before it compares anything. Byte-exact, count-checked."""
import sys

P = sys.argv[1] + "/eech-campaign/crates/eech-world/lua/sector-height-baseline.lua"
raw = open(P, "rb").read()
assert b"\r\n" not in raw
d = raw.decode("utf-8")

# ---- 1. the helpers: from `local function pairs_of` up to (not including) `local function run ()`
a = d.index("local function pairs_of (line)")
b = d.index("local function run ()")
assert d.count("local function pairs_of (line)") == 1 and d.count("local function run ()") == 1
helpers = r'''local function pairs_of (line)
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

'''
d = d[:a] + helpers + d[b:]

# ---- 2. the hosted block
a = d.index("\tif hosted then\n\t\t-- the hosted-scan scope: no event row")
b = d.index("\tlocal last_seq = 0\n\tfor i = 0, events - 1 do")
block = r'''	if hosted then
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

'''
d = d[:a] + block + d[b:]
open(P, "wb").write(d.encode("utf-8"))
print("ok")

d = open(P, "rb").read().decode("utf-8")
old = """	local events, clearances, sectors = number_of ("sho-tables", "events_used"), number_of ("sho-clearance", "stored"), number_of ("sho-scan", "sectors")
"""
new = """	-- the hosted scope reads its counts from the strictly parsed summary below (a malformed summary is INVALID_HOSTED, not an error of this lookup)
	local events, clearances, sectors
	if not hosted then
		events, clearances, sectors = number_of ("sho-tables", "events_used"), number_of ("sho-clearance", "stored"), number_of ("sho-scan", "sectors")
	end
"""
assert d.count(old) == 1
open(P, "wb").write(d.replace(old, new).encode("utf-8"))
print("ok2")
