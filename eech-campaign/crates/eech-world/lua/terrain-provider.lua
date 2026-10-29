-- Public-path checks of the host terrain provider (EEDCS-47, D3): what the campaign does with the host's terrain answers, and how it refuses a
-- missing or malformed provider. Boots the campaign through require ("eech_dc") with a `terrain` provider this script supplies, runs frames,
-- and prints one summary line. Needs an eech-world with `host.terrain_open` (src/terrain.rs); the shipping module is not changed by it.
--
-- It tests nothing about campaign policy; it shows that the campaign's physical terrain facts come from the host, and that a bad host is a
-- visible failure. Modes (one per process, as the engine is one per process):
--   mode=reference   the reference host's own terrain, unchanged: the control
--   mode=raised      the reference terrain with every elevation (point, stored, fast) raised by `raise` metres (default 10000)
--   mode=generic     a host with no EECH data: an analytic surface, no hints, the reference map's extents only
--   mode=missing     boot without a terrain provider: must fail at boot with a clear error
--   mode=bad_*       a provider answering a malformed value at its first point query: the engine must stop with a clear error
--                    (bad_nil, bad_elevation, bad_normal, bad_type, bad_hint, bad_error)
--
-- What it observes, from the public surface only (no engine internals):
--   * boot placement: the elevation (pose.position[2]) of every mobile the campaign hands to host.bind_mobile, as a count and a sum;
--   * the campaign's environment answers to the host's line-of-sight dialogue: the count and sum of the `elevation` it reports for
--     each terrain sample of a query (its own terrain service today, the provider's after the extraction);
--   * how often the provider was consulted, by operation.
--
-- EXPECTED on the UNMOVED baseline (the DLL still answers from its own terrain), which is the red state for the new contract:
--   reference, raised, generic: provider calls = 0 and the boot placement and line-of-sight sums are identical in all three;
--   missing: boot SUCCEEDS (no refusal); bad_*: nothing fails (the provider is never asked).
-- EXPECTED after the extraction: reference calls > 0 and the same sums as the baseline; raised changes both sums (by `raise` per sample);
--   generic boots and runs with provider calls > 0; missing and every bad_* fail with a clear error naming the terrain provider.
--
-- host.args: root, scenario (lebanon_retail default, georgia_retail), seconds (default 30), frame_ms (default 100), mode (default reference),
--            raise (default 10000), out (JSON Lines summary path)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local seconds = tonumber (args.seconds or "30")
local frame_ms = tonumber (args.frame_ms or "100")
local mode = args.mode or "reference"
local raise = tonumber (args.raise or "10000")
assert (seconds and seconds > 0 and seconds <= 3600 and frame_ms and frame_ms >= 10 and frame_ms <= 1000, "seconds and frame_ms are out of range")
assert (raise and raise == raise and raise ~= math.huge and raise ~= -math.huge, "raise is not finite")
local out = assert (io.open (assert (args.out, "out=<JSON Lines path>"), "w"))
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC", terrain = "/common/maps/map5/terrain" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC", terrain = "/common/maps/map3/terrain" },
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

assert (host.terrain_open, "this eech-world has no host.terrain_open")
local reference = host.terrain_open { terrain_dir = root .. scenario.terrain }
local facts = reference { op = "map" }

local calls = { map = 0, point = 0, fast = 0 }

local function counted (inner)
	return function (request)
		calls [request.op] = (calls [request.op] or 0) + 1
		return inner (request)
	end
end

local first_point = true
local provider
if mode == "reference" then
	provider = counted (reference)
elseif mode == "raised" then
	provider = counted (function (request)
		local answer = reference (request)
		if request.op == "point" then
			answer.elevation = answer.elevation + raise
			answer.last_elevation = answer.last_elevation + raise
		elseif request.op == "fast" then
			answer.elevation = answer.elevation + raise
		end
		return answer
	end)
elseif mode == "generic" then
	provider = counted (function (request)
		if request.op == "map" then return facts end
		local x, z = request.x, request.z
		local elevation = 200 + 80 * math.sin (x / 5000) * math.cos (z / 7000)
		if request.op == "fast" then return { elevation = elevation } end
		-- a host with no EECH data: land (type 9), an upward normal, no hint
		return { elevation = elevation, last_elevation = elevation, normal = { 0, 1, 0 }, type = 9 }
	end)
elseif mode == "missing" then
	provider = nil
elseif mode:sub (1, 4) == "bad_" then
	provider = counted (function (request)
		if request.op ~= "point" or not first_point then return reference (request) end
		first_point = false
		local answer = reference (request)
		if mode == "bad_nil" then return nil
		elseif mode == "bad_elevation" then answer.elevation = 0 / 0 return answer
		elseif mode == "bad_normal" then answer.normal = { 0, 1 } return answer
		elseif mode == "bad_type" then answer.type = 99999 return answer
		elseif mode == "bad_hint" then answer.hint = 12345 return answer
		elseif mode == "bad_error" then error ("provider failed on purpose")
		end
		error ("unknown bad mode " .. mode)
	end)
else
	error ("unknown mode " .. mode)
end

-- the public observations
local placed = { count = 0, sum = 0 }
local los = { count = 0, sum = 0 }

local function bind_mobile (descriptor)
	local pose = descriptor.pose
	if pose and pose.position then
		placed.count = placed.count + 1
		placed.sum = placed.sum + pose.position [2]
	end
	return host.bind_mobile (descriptor)
end

local function line_of_sight (step)
	if step.mode == "terrain" then
		los.count = los.count + 1
		los.sum = los.sum + step.elevation
	end
	return host.line_of_sight (step)
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local config = {
	install_root = root, map = scenario.map, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = line_of_sight, sense = host.sense, aim = host.aim,
	terrain = provider,
}
local ok, engine = pcall (dc.boot, config)
if not ok then
	emit { event = "boot_failed", mode = mode, error = (tostring (engine):gsub ("\n.*", "")), terrain_calls = calls.map + calls.point + calls.fast }
	out:close ()
	io.stderr:write ("terrain-provider: boot failed: " .. (tostring (engine):gsub ("\n.*", "")) .. "\n")
	assert (mode == "missing" or mode:sub (1, 4) == "bad_", "boot failed in a mode that must boot")
	return
end

local frames = math.floor (seconds * 1000 / frame_ms)
local failure
for f = 1, frames do
	local frame_ok, err = pcall (engine.frame, engine, frame_ms)
	if not frame_ok then failure = (tostring (err):gsub ("\n.*", "")) break end
end
emit {
	event = "summary", mode = mode, frames = frames, failure = failure or "",
	map_calls = calls.map, point_calls = calls.point, fast_calls = calls.fast,
	placed_count = placed.count, placed_sum = string.format ("%.9g", placed.sum),
	los_terrain_count = los.count, los_terrain_sum = string.format ("%.9g", los.sum),
}
out:close ()
io.stderr:write (string.format ("terrain-provider: mode=%s map=%d point=%d fast=%d placed=%d/%.9g los=%d/%.9g failure=%s\n", mode, calls.map, calls.point, calls.fast,
	placed.count, placed.sum, los.count, los.sum, failure or "none"))
-- a run that must not complete (missing provider, a malformed answer) is a failure of the check when it completes
if mode == "missing" then
	error ("terrain-provider: a campaign without a terrain provider booted and ran")
elseif mode:sub (1, 4) == "bad_" then
	assert (failure, "a malformed terrain answer was accepted: mode " .. mode)
elseif failure then
	error (failure)
else
	-- the new contract: the campaign's physical terrain comes from the provider (RED on the unmoved baseline, where nothing is asked)
	assert (calls.point + calls.fast > 0, "terrain-provider: the campaign never consulted the terrain provider")
end
