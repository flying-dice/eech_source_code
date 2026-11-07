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

	emit (string.format ("sho-baseline schema=1 scenario=%s seed=%d frame_ms=%s hours=%s frames=%d elapsed_seconds=%.3f observer=read-only", scenario_name, seed, tostring (frame_ms), tostring (hours), frames, clock.elapsed_seconds))
	local summary = engine:sector_height_summary ()
	for line in summary:gmatch ("[^\n]+") do emit (line) end

	local function number_of (prefix, key)
		for line in summary:gmatch ("[^\n]+") do
			if line:sub (1, #prefix) == prefix then return tonumber (field (line, key)) end
		end
		error ("the observer summary has no " .. prefix .. " " .. key, 0)
	end
	local events, clearances, sectors = number_of ("sho-tables", "events_used"), number_of ("sho-clearance", "stored"), number_of ("sho-scan", "sectors")

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
