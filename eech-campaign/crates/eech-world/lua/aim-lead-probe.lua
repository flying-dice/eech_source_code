-- TEST-ONLY (EEDCS-44): the partial-output contract of the lead prediction's time of flight, through the campaign's own function and the real `aim` callback.
-- Needs a DLL built with the `aim-fixture` feature. It boots the campaign through the public path (require ("eech_dc")) with the reference host's callbacks, steps a short
-- warm-up, picks two live host-identified entities of opposite sides, and calls `engine:aim_lead_probe (source, target, weapon, sentinel)`: the campaign's own
-- get_lead_and_ballistic_intercept_point_and_angle_of_projection (the function weapon launch calls for the decoy and fuze timer) is asked once, with the caller's
-- time-of-flight variable set to a sentinel, and the report says whether the variable still holds the sentinel afterwards. The host that answers is a wrapper this
-- script gives the engine for `aim`: every request but `lead` goes to the reference host unchanged (so the ballistic tables reach it at boot); the `lead` answer is the
-- scenario's. One scenario per call, all in one process, except the two malformed ones, which stop the engine and so run alone (mode=bad_nan, mode=bad_string).
--
--   reference         the reference host answers; the campaign must store the flight time it produced (the sentinel is replaced by that value)
--   off_map           the request is rewritten so the target lies off the map and the device is within reach of it: the reference's off-map path succeeds and produces NO
--                     flight time, and the campaign must keep the sentinel (the original's behaviour on that branch)
--   present           a host with a model of its own answers ok with a flight time (12.5): stored
--   absent_ok         the same host answers ok with none: the sentinel is kept
--   absent_failed     the same host answers a failure with none: the sentinel is kept
--   present_failed    the same host answers a failure WITH a flight time (3.0), as a solve that produced one before it failed: stored (a partial output is stored when it exists)
--   bad_nan, bad_string   a flight time that is not a finite number: the engine must stop with a clear error
--
-- host.args: root, scenario (lebanon_retail default), warmup (simulated seconds, default 20), frame_ms (default 100), weapon (default 13), mode (default all), out (JSON Lines path)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local warmup = tonumber (args.warmup or "20")
local frame_ms = tonumber (args.frame_ms or "100")
local weapon = tonumber (args.weapon or "13")
local mode = args.mode or "all"
local out = assert (io.open (assert (args.out, "out=<JSON Lines path>"), "w"))
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario = assert (scenarios [args.scenario or "lebanon_retail"], "scenario=lebanon_retail|georgia_retail")
local SENTINEL = -123.5

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
local failed = 0
local function check (name, ok, detail)
	if not ok then failed = failed + 1 end
	emit { event = "check", name = name, ok = ok and true or false, detail = detail or "" }
end

local descriptors, order = {}, {}
local function bind_mobile (descriptor)
	local id = host.bind_mobile (descriptor)
	descriptors [id] = { kind = descriptor.movement_kind, side = descriptor.side }
	order [#order + 1] = id
	return id
end

local current, last_reference_reply, lead_requests = nil, nil, 0
local function copy (t) local c = {} for k, v in pairs (t) do c [k] = v end return c end

-- the host this script gives the engine for `aim`
local function aim (step)
	if step.mode ~= "lead" or not current then return host.aim (step) end
	lead_requests = lead_requests + 1
	if current == "reference" then
		last_reference_reply = host.aim (step)
		return last_reference_reply
	elseif current == "off_map" then
		local s = copy (step)
		local point = { step.map_max [1] + 1000, step.target.point [2], step.target.point [3] }
		s.target = copy (step.target)
		s.target.point = point
		s.target.position = point
		s.target.motion_vector = { 0, 0, 0 }
		s.device_position = { point [1] - 1500, point [2], point [3] }
		s.parent_velocity = 500
		last_reference_reply = host.aim (s)
		return last_reference_reply
	end
	local point = step.target.point
	local answer = { state = "done", ok = true, intercept_point = { point [1], point [2], point [3] }, angle = 0.25 }
	if current == "present" then answer.time_of_flight = 12.5
	elseif current == "absent_failed" then answer.ok = false
	elseif current == "present_failed" then answer.ok = false answer.time_of_flight = 3.0
	elseif current == "bad_nan" then answer.time_of_flight = 0 / 0
	elseif current == "bad_string" then answer.time_of_flight = "soon" end
	return answer
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = scenario.map, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, model = host.model, sense = host.sense, aim = aim,
}
emit { event = "boot", scenario = args.scenario or "lebanon_retail", mode = mode, warmup = warmup, weapon = weapon, sentinel = SENTINEL }
for _ = 1, math.floor (warmup * 1000 / frame_ms) do engine:frame (frame_ms) end
emit { event = "warmed_up", elapsed = engine:clock ().elapsed_seconds, bound = #order }

local function alive_state (id)
	local ok, text = pcall (engine.aim_person_query, engine, id)
	if not ok then return nil end
	local alive = string.match (text, " alive=(%d+) ")
	return alive == "1"
end
-- the entities are routed vehicles (movement kind 1): a site or a unit without a movement record is not a valid subject of the query
local source, target
for _, a in ipairs (order) do
	if not source and descriptors [a].kind == 1 and (descriptors [a].side == "blue" or descriptors [a].side == "red") and alive_state (a) then
		for _, b in ipairs (order) do
			if b ~= a and descriptors [b].kind == 1 and descriptors [b].side ~= descriptors [a].side and (descriptors [b].side == "blue" or descriptors [b].side == "red") and alive_state (b) then
				source, target = a, b
				break
			end
		end
	end
end
assert (source and target, "no pair of live host-identified entities of opposite sides")
emit { event = "chosen", source = source, target = target }

local function probe (name)
	current, last_reference_reply = name, nil
	local before = lead_requests
	local ok, text = pcall (engine.aim_lead_probe, engine, source, target, weapon, SENTINEL)
	current = nil
	if not ok then return nil, tostring (text), lead_requests - before end
	local line = string.match (text, "aim%-lead%-probe [^\n]+")
	assert (line, "no probe report in: " .. tostring (text))
	local r = {}
	for k, v in string.gmatch (line, "(%S+)=(%S+)") do r [k] = v end
	r.result, r.kept_sentinel, r.time_of_flight = tonumber (r.result), tonumber (r.kept_sentinel), tonumber (r.time_of_flight)
	return r, line, lead_requests - before
end

local function expect (name, wanted)
	local r, line, asked = probe (name)
	if not r then check (name .. ": the probe ran", false, line) return end
	emit { event = "probe", scenario = name, report = line, lead_requests = asked }
	check (name .. ": the host was asked at least one lead request (one logical solve; a random-draw handshake is two physical requests)", asked >= 1, tostring (asked))
	wanted (r, line)
end

if mode == "all" then
	expect ("reference", function (r)
		local reply = last_reference_reply
		check ("reference: the host produced a flight time", reply and reply.ok == true and reply.time_of_flight ~= nil, reply and tostring (reply.time_of_flight))
		check ("reference: the campaign returned the success", r.result == 1)
		check ("reference: the sentinel was replaced by the host's flight time", r.kept_sentinel == 0 and reply and r.time_of_flight == tonumber (string.format ("%.9g", reply.time_of_flight)),
			string.format ("caller %s host %s", tostring (r.time_of_flight), tostring (reply and reply.time_of_flight)))
	end)
	expect ("off_map", function (r)
		local reply = last_reference_reply
		check ("off_map: the reference solver's off-map path succeeded and produced NO flight time (it invents none)", reply and reply.ok == true and reply.time_of_flight == nil,
			reply and string.format ("ok=%s time_of_flight=%s", tostring (reply.ok), tostring (reply.time_of_flight)))
		check ("off_map: the campaign returned the success and KEPT the caller's sentinel", r.result == 1 and r.kept_sentinel == 1 and r.time_of_flight == SENTINEL,
			string.format ("result %s caller %s", tostring (r.result), tostring (r.time_of_flight)))
	end)
	expect ("present", function (r) check ("present: a produced flight time replaces the sentinel", r.result == 1 and r.kept_sentinel == 0 and r.time_of_flight == 12.5, tostring (r.time_of_flight)) end)
	expect ("absent_ok", function (r) check ("absent_ok: success without a flight time keeps the sentinel", r.result == 1 and r.kept_sentinel == 1, tostring (r.time_of_flight)) end)
	expect ("absent_failed", function (r) check ("absent_failed: failure without a flight time keeps the sentinel", r.result == 0 and r.kept_sentinel == 1, tostring (r.time_of_flight)) end)
	expect ("present_failed", function (r) check ("present_failed: a flight time produced before the failure is stored", r.result == 0 and r.kept_sentinel == 0 and r.time_of_flight == 3.0, tostring (r.time_of_flight)) end)
elseif mode == "bad_nan" or mode == "bad_string" then
	local r, text = probe (mode)
	check (mode .. ": the engine stopped with a clear error and no result", r == nil and string.find (text, "host aim", 1, true) and string.find (text, "time_of_flight", 1, true), tostring (text):gsub ("\n.*", ""))
else
	error ("unknown mode " .. mode)
end

emit { event = "verdict", mode = mode, failed = failed }
out:close ()
io.stderr:write (string.format ("aim-lead-probe: mode=%s failed=%d\n", mode, failed))
assert (failed == 0, "a check failed")
