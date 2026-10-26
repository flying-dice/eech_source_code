-- Public-path checks of the host line-of-sight provider (EEDCS-42): what the campaign does with the host's answers, and how it
-- refuses a missing or malformed provider. Boots the campaign through require ("eech_dc") with a provider this script wraps around
-- (or replaces) the reference host's `host.line_of_sight`, runs frames, and prints one summary line.
--
-- It tests nothing about campaign policy itself; it shows that the policy callers act on whatever the host says, and that a bad host
-- is a visible failure. Modes, all of them in one process because the engine is one per process (run once per mode):
--   mode=reference         the reference host unchanged: the control
--   mode=blocked           the host answers every query "not visible" directly (a DCS-shaped visibility fact, no dialogue)
--   mode=clear             the host answers every query "visible" directly
--   mode=raise_terrain     the reference host, but every terrain elevation the campaign reports to it is raised by 10 km, so the
--                          campaign's environmental answers (not the host's arithmetic) block every ray that has a terrain sample
--   mode=missing           boot without a line_of_sight provider: must fail at boot with a clear error
--   mode=bad_*             a provider answering malformed values at its first query: the engine must stop with a clear error
--                          (bad_nil, bad_state, bad_visible, bad_phase, bad_point, bad_error)
-- host.args: root, scenario (lebanon_retail default, georgia_retail), seconds (default 120), frame_ms (default 100), mode
--            (default reference), out (JSON Lines summary path)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local seconds = tonumber (args.seconds or "120")
local frame_ms = tonumber (args.frame_ms or "100")
local mode = args.mode or "reference"
local out = assert (io.open (assert (args.out, "out=<JSON Lines path>"), "w"))
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario = assert (scenarios [args.scenario or "lebanon_retail"], "scenario=lebanon_retail|georgia_retail")

local function emit (t)
	local parts = {}
	for k, v in pairs (t) do
		local value = type (v) == "string" and string.format ("%q", v) or tostring (v)
		parts [#parts + 1] = string.format ("%q:%s", k, value)
	end
	table.sort (parts)
	out:write ("{" .. table.concat (parts, ",") .. "}\n")
	out:flush ()
end

local reference = host.line_of_sight
local counts = { queries = 0, visible = 0, blocked = 0, requests = 0 }

local function counted_reference (step)
	if step.mode == "begin" then counts.queries = counts.queries + 1 end
	counts.requests = counts.requests + 1
	local answer = reference (step)
	if answer.state == "done" then
		if answer.visible then counts.visible = counts.visible + 1 else counts.blocked = counts.blocked + 1 end
	end
	return answer
end

local provider
if mode == "reference" then
	provider = counted_reference
elseif mode == "blocked" or mode == "clear" then
	provider = function (step)
		assert (step.mode == "begin", "a host that answers at begin is never asked for terrain or objects")
		counts.queries = counts.queries + 1
		counts.requests = counts.requests + 1
		if mode == "clear" then counts.visible = counts.visible + 1 else counts.blocked = counts.blocked + 1 end
		return { state = "done", visible = mode == "clear" }
	end
elseif mode == "raise_terrain" then
	provider = function (step)
		if step.mode == "terrain" then
			local raised = {}
			for k, v in pairs (step) do raised [k] = v end
			raised.elevation = step.elevation + 10000
			step = raised
		end
		return counted_reference (step)
	end
elseif mode == "missing" then
	provider = nil
elseif mode:sub (1, 4) == "bad_" then
	-- the first query's first answer is malformed; the engine must stop there with a clear error
	provider = function (step)
		if mode == "bad_nil" then return nil
		elseif mode == "bad_state" then return { state = "later" }
		elseif mode == "bad_visible" then return { state = "done", visible = 1 }
		elseif mode == "bad_phase" then return { state = "need_terrain", phase = 9, point = { 0, 0, 0 } }
		elseif mode == "bad_point" then return { state = "need_terrain", phase = 1, point = { 0, 0 } }
		elseif mode == "bad_error" then error ("provider failed on purpose")
		end
		error ("unknown bad mode " .. mode)
	end
else
	error ("unknown mode " .. mode)
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local config = {
	install_root = root, map = scenario.map, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = provider,
}
local ok, engine = pcall (dc.boot, config)
if not ok then
	emit { event = "boot_failed", mode = mode, error = (tostring (engine):gsub ("\n.*", "")) }
	out:close ()
	io.stderr:write ("los-provider: boot failed: " .. (tostring (engine):gsub ("\n.*", "")) .. "\n")
	assert (mode == "missing", "boot failed")
	return
end
assert (mode ~= "missing", "boot succeeded without a line-of-sight provider")

local frames = math.floor (seconds * 1000 / frame_ms)
local seen, births, lost = {}, 0, 0
local last_alive = {}
local failure
for f = 1, frames do
	local frame_ok, err = pcall (engine.frame, engine, frame_ms)
	if not frame_ok then failure = (tostring (err):gsub ("\n.*", "")) break end
	if f % 10 == 0 then
		local present = {}
		for _, o in ipairs (engine:objects ()) do
			if o.kind == "weapon" then
				present [o.id] = true
				if not seen [o.id] then births = births + 1 end
			elseif o.kind == "ground_vehicle" or o.kind == "air_defence" or o.kind == "helicopter" or o.kind == "fixed_wing" then
				if last_alive [o.id] and not o.alive then lost = lost + 1 end
				last_alive [o.id] = o.alive
			end
		end
		seen = present
	end
end
emit { event = "summary", mode = mode, frames = frames, failure = failure or "", queries = counts.queries, visible = counts.visible, blocked = counts.blocked,
	requests = counts.requests, weapon_births = births, losses = lost }
out:close ()
io.stderr:write (string.format ("los-provider: mode=%s queries=%d visible=%d blocked=%d requests=%d weapon_births=%d losses=%d failure=%s\n", mode,
	counts.queries, counts.visible, counts.blocked, counts.requests, births, lost, failure or "none"))
if mode:sub (1, 4) == "bad_" then
	assert (failure, "a malformed provider answer was accepted: mode " .. mode)
elseif failure then
	error (failure)
end
