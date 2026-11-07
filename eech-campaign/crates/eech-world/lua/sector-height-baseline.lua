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

local WINDOW_FIELDS = { "scenario", "seed", "frame_ms", "hours", "frames" }
local ROW_FIELDS = { "reads", "first_word", "last_word", "changes", "fact_offers", "fact_equal_first", "fact_differ_first", "fact_first_word", "fact_last_word", "fact_changes" }
local TOTAL_FIELDS = { "reads", "firsts", "sectors", "fact_offers", "fact_nonzero", "fact_equal_first", "fact_differ_first", "fact_without_first" }

-- the scan rows of a report file, streamed (the accepted report is large: only its identity line, its `sho-scan` summary and its per-sector rows are kept): the identity, the totals, the rows by sector, or nil and why
local function read_scan_report (path)
	local file, why = io.open (path, "rb")
	if not file then return nil, "cannot open " .. tostring (path) .. ": " .. tostring (why) end
	local identity, totals, rows = nil, nil, {}
	for line in file:lines () do
		if line:sub (1, 13) == "sho-baseline " then identity = pairs_of (line)
		elseif line:sub (1, 15) == "sho-scan reads=" then totals = pairs_of (line)
		elseif line:sub (1, 11) == "sho-scan i=" then
			local row = pairs_of (line)
			rows [(row.sx or "?") .. "," .. (row.sz or "?")] = row
		end
	end
	file:close ()
	if not identity or not totals then return nil, "the report has no identity line or no sho-scan summary" end
	return { identity = identity, totals = totals, rows = rows }
end

-- the scoped comparison of the hosted scan with the accepted report: returns the status, the reason and the list of differences (field lines); see the header of this file for exactly what is compared
local function compare_scan (hosted_report, accepted)
	local report, why = read_scan_report (accepted)
	if not report then return "NOT_COMPARABLE", why, {} end
	for _, key in ipairs (WINDOW_FIELDS) do
		if tostring (hosted_report.identity [key]) ~= tostring (report.identity [key]) then
			return "NOT_COMPARABLE", "the window differs at " .. key .. ": hosted " .. tostring (hosted_report.identity [key]) .. ", accepted " .. tostring (report.identity [key]), {}
		end
	end
	if (tonumber (hosted_report.totals.reads) or 0) == 0 then return "UNREACHED", "the hosted run made no scan FIRST read in the window", {} end
	local differences = {}
	for _, key in ipairs (TOTAL_FIELDS) do
		if hosted_report.totals [key] ~= report.totals [key] then
			differences [#differences + 1] = string.format ("sho-compare-diff total=%s hosted=%s accepted=%s", key, tostring (hosted_report.totals [key]), tostring (report.totals [key]))
		end
	end
	for sector, row in pairs (hosted_report.rows) do
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
		if not hosted_report.rows [sector] then
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
	local events, clearances, sectors = number_of ("sho-tables", "events_used"), number_of ("sho-clearance", "stored"), number_of ("sho-scan", "sectors")

	if hosted then
		-- the hosted-scan scope: no event row (the original-link diagnostics are not comparable and are not written); the clearance records and the scan rows only
		local hosted_rows = {}
		for i = 0, clearances - 1 do emit ((engine:sector_height_clearance (i):gsub ("\n$", ""))) end
		for i = 0, sectors - 1 do
			local line = engine:sector_height_scan (i):gsub ("\n$", "")
			emit (line)
			local row = pairs_of (line)
			hosted_rows [(row.sx or "?") .. "," .. (row.sz or "?")] = row
		end
		local identity = pairs_of (identity_line)
		identity.frames = tostring (frames)
		local totals
		for line in summary:gmatch ("[^\n]+") do if line:sub (1, 15) == "sho-scan reads=" then totals = pairs_of (line) end end
		local scan_whole = totals ~= nil and (tonumber (field (summary:match ("sho%-tables[^\n]+"), "scan_out_of_range") or "1") or 1) == 0 and (tonumber (totals.reads) or 0) >= 1
		local status, reason, differences = "SCAN_ONLY", "no accepted report was given: the scan record is whole, and nothing was compared", {}
		if accepted_path then
			status, reason, differences = compare_scan ({ identity = identity, totals = totals or {}, rows = hosted_rows }, accepted_path)
		elseif (tonumber (totals and totals.reads or "0") or 0) == 0 then
			status, reason = "UNREACHED", "the hosted run made no scan FIRST read in the window"
		end
		emit (string.format ("sho-compare scope=scan-reads-and-fact12 status=%s sectors_hosted=%d reason=%s differences=%d", status, sectors, (reason:gsub ("%s", "_")), #differences))
		for _, line in ipairs (differences) do emit (line) end
		emit (string.format ("sho-verdict scope=hosted-scan scan_whole=%d status=%s clearances=%d sectors=%d original_link_pack_hooks=not_compared", scan_whole and 1 or 0, status, clearances, sectors))
		host.log (string.format ("sector-height-baseline: hosted-scan %s; %d clearance records, %d sectors", status, clearances, sectors))
		if status ~= "EQUAL_AT_SCOPE" and status ~= "SCAN_ONLY" then error ("hosted-scan comparison " .. status .. ": " .. reason, 0) end
		if not scan_whole then error ("the hosted scan record is not whole (scan_out_of_range or no read)", 0) end
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
