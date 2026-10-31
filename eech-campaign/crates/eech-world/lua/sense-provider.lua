-- Public-path checks of the host sensing provider (EEDCS-43): what the campaign does with the host's physical sensing answers, and how
-- it refuses a missing or malformed provider. Boots the campaign through require ("eech_dc") with a provider this script wraps around
-- (or replaces) the reference host's `host.sense`, runs frames, and prints one summary line.
--
-- It tests nothing about campaign policy itself: it shows that the campaign's scans act on whatever physical sensing the host reports, and that
-- a bad host is a visible failure. One mode per process (the engine is one per process):
--   mode=reference    the reference host unchanged: the control
--   mode=blocked      the host reports every candidate undetectable (reject, reason "range") at its first message
--   mode=not_perceived  the host reports every candidate not perceived with the generic reason "not_perceived" (a verdict of its own model, no EECH
--                     predicate): the same campaign behaviour as `blocked`, whatever the scan, because the campaign acts on the verdict and not on its reason
--   mode=own_model    a host with a model of its own that is not EECH's: it perceives a candidate when a plain horizontal distance (from the positions it was
--                     given) is under its own limit, answers that distance as the measured range, and otherwise reports "not_perceived". It needs
--                     no draw and no signature, and no seed or entity index reaches it
--   mode=far          the reference host, but every scan's raw scan range is tripled before the host's own arithmetic sees it
--   mode=missing      boot without a sense provider: must fail at boot with a clear error
--   mode=bad_*        a provider that answers the first candidate message malformed: the engine must stop with a clear error
--                     (bad_nil, bad_state, bad_reason, bad_accept_range, bad_draw_for_range, bad_error); bad_begin answers the first begin wrongly
-- host.args: root, scenario (lebanon_retail default, georgia_retail), seconds (default 300), frame_ms (default 100), mode (default reference),
--            out (JSON Lines summary path)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local seconds = tonumber (args.seconds or "300")
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

local reference = host.sense
-- calls per message mode, and the answers by state: what the campaign asked and what the host said
local counts = { begin = 0, candidate = 0, draw = 0, signature = 0, ["end"] = 0, accept = 0, reject = 0 }
-- the candidates the host was consulted about, by scan kind, and the rejections of the generic reason by scan kind
local consulted, not_perceived_by_scan = {}, {}

local function consult (step)
	counts [step.mode] = counts [step.mode] + 1
	if step.mode == "candidate" then consulted [step.scan] = (consulted [step.scan] or 0) + 1 end
end

local function counted (step)
	consult (step)
	local answer = reference (step)
	if answer.state == "accept" then counts.accept = counts.accept + 1 end
	if answer.state == "reject" then counts.reject = counts.reject + 1 end
	return answer
end

local provider
if mode == "reference" then
	provider = counted
elseif mode == "blocked" then
	-- a DCS-shaped host that decides detectability itself: it needs no dialogue, only the verdict, so it answers each candidate at once
	provider = function (step)
		consult (step)
		if step.mode == "begin" then return { state = "open" } end
		if step.mode == "end" then return { state = "closed" } end
		assert (step.mode == "candidate", "a host that rejects at the candidate is never asked for a draw or a signature")
		counts.reject = counts.reject + 1
		return { state = "reject", reason = (step.scan == "surface_to_air" and "altitude") or (step.scan == "air_to_air" and "range_adjusted") or "range" }
	end
elseif mode == "not_perceived" or mode == "own_model" then
	-- a DCS-shaped host: it decides at the candidate from its own model, so it answers begin and end at once and never asks for a draw or a signature
	local limit = 4000.0
	local source
	provider = function (step)
		consult (step)
		if step.mode == "begin" then source = step.source_position return { state = "open" } end
		if step.mode == "end" then return { state = "closed" } end
		assert (step.mode == "candidate", "a host that decides at the candidate is never asked for a draw or a signature")
		if mode == "own_model" then
			local dx, dz = step.target_position [1] - source [1], step.target_position [3] - source [3]
			local distance = math.sqrt (dx * dx + dz * dz)
			if distance < limit then
				counts.accept = counts.accept + 1
				return { state = "accept", target_range = distance }
			end
		end
		counts.reject = counts.reject + 1
		not_perceived_by_scan [step.scan] = (not_perceived_by_scan [step.scan] or 0) + 1
		return { state = "reject", reason = "not_perceived" }
	end
elseif mode == "far" then
	provider = function (step)
		if step.mode == "begin" then
			local tripled = {}
			for k, v in pairs (step) do tripled [k] = v end
			tripled.scan_range = step.scan_range * 3
			step = tripled
		end
		return counted (step)
	end
elseif mode == "missing" then
	provider = nil
elseif mode:sub (1, 4) == "bad_" then
	-- the reference host answers until the first candidate (or begin for bad_begin); that answer is malformed and the engine must stop there
	local armed = true
	provider = function (step)
		-- bad_draw_for_range asks for a draw on a scan that has no tolerance, so it waits for the first candidate that is not a surface-to-air one
		local first = (mode == "bad_begin" and step.mode == "begin") or (mode ~= "bad_begin" and step.mode == "candidate" and (mode ~= "bad_draw_for_range" or step.scan ~= "surface_to_air"))
		if armed and first then
			armed = false
			if mode == "bad_nil" then return nil
			elseif mode == "bad_state" then return { state = "later" }
			elseif mode == "bad_reason" then return { state = "reject", reason = "because" }
			elseif mode == "bad_accept_range" then return { state = "accept", target_range = 0 / 0 }
			elseif mode == "bad_draw_for_range" then return { state = "need_draw", target_range = 1 }
			elseif mode == "bad_begin" then return { state = "accept", target_range = 1 }
			elseif mode == "bad_error" then error ("provider failed on purpose")
			end
			error ("unknown bad mode " .. mode)
		end
		return counted (step)
	end
else
	error ("unknown mode " .. mode)
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local config = {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = provider, aim = host.aim,
}
local ok, engine = pcall (dc.boot, config)
if not ok then
	emit { event = "boot_failed", mode = mode, error = (tostring (engine):gsub ("\n.*", "")) }
	out:close ()
	io.stderr:write ("sense-provider: boot failed: " .. (tostring (engine):gsub ("\n.*", "")) .. "\n")
	assert (mode == "missing", "boot failed")
	return
end
assert (mode ~= "missing", "boot succeeded without a sense provider")

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
local by_scan = {}
for _, scan in ipairs { "air_to_air", "air_to_surface", "surface_to_air", "surface_to_surface" } do
	by_scan [#by_scan + 1] = string.format ("%s=%d/%d", scan, consulted [scan] or 0, not_perceived_by_scan [scan] or 0)
end
by_scan = table.concat (by_scan, " ")
emit { event = "summary", mode = mode, frames = frames, failure = failure or "", begin = counts.begin, candidate = counts.candidate, draw = counts.draw,
	signature = counts.signature, ["end"] = counts ["end"], accept = counts.accept, reject = counts.reject, weapon_births = births, losses = lost,
	candidates_by_scan_and_not_perceived = by_scan }
out:close ()
io.stderr:write (string.format ("sense-provider: mode=%s begin=%d candidate=%d draw=%d signature=%d end=%d accept=%d reject=%d weapon_births=%d losses=%d candidates/not_perceived by scan: %s failure=%s\n", mode,
	counts.begin, counts.candidate, counts.draw, counts.signature, counts ["end"], counts.accept, counts.reject, births, lost, by_scan, failure or "none"))
if mode:sub (1, 4) == "bad_" then
	assert (failure, "a malformed provider answer was accepted: mode " .. mode)
elseif failure then
	error (failure)
end
