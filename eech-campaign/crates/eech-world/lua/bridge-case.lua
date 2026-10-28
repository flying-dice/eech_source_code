-- TEST-ONLY controlled witness of the ORIGINAL bridge and segment collapse (EEDCS-46, R5). Needs a DLL built with the `weapon-fixture` feature. NOT RUN: this script has been parsed and its flow
-- checked against a stand-in engine (tests/host_callbacks.rs); no build or native slot has been given to it. It moves and changes no bridge physics: it observes the campaign's own code.
--
-- WHAT IT DOES. From a verified retail boot (Lebanon by default, Georgia with scenario=georgia_retail), it surveys the bridges the world holds, chooses one that has a segment next to an
-- unsupported mid section (an explicit test-only criterion, applied inside C: no pointer, heap index or seed crosses), records its state, then applies a DECLARED DIRECT STRUCTURAL FAILURE to
-- that segment (the campaign's own kill of one segment: this is NOT a weapon hit and is never reported as one), and records, for a bounded window of frames, the bridge's and every segment's
-- state after each frame and the ordered events the original code produced (csrc/eech_cmb_case.c, csrc/eech_cmb_fixture.c: the read-only CMB_FX_BRIDGE hooks of special/bridge/br_msgs.c and
-- fixed/segment/sg_updt.c, armed only for this case).  The war runs on as it does; its events are kept apart from the bridge's by scope and by their `kind=bridge-*` label.
--
-- THE PRECONDITIONS ARE THE REPORT'S: if the world has no bridge with a segment next to an unsupported span, the script says so (`bridge_case` done=false, reason) and does nothing else, so the
-- same script can be run on the other verified scenario; it never invents a bridge.  Declared and unreached: a weapon hit on a segment, a player or multiplayer cause, a client, a bridge with
-- no unsupported span, and the vehicles on the route beyond the world's counts of routed vehicles alive and repair tasks.
--
-- Output, JSON Lines (the writer and parser of lua/combat-report.lua): boot; `bridge:survey`; `bridge:before` (state); `bridge_case` (the failure's own report and whether it was done);
-- `bridge:after` (the state immediately after the failure and before any frame); for each frame n: `frame:n:state` and `frame:n:log` (the events of that frame, in order, with the engine's
-- sequence numbers); `bridge:disarm`; the observer's whole report before and after (`bridge:trace:before`, `bridge:trace:after`); `run`.
--
-- host.args: root, out (JSON Lines path), scenario (lebanon_retail default, georgia_retail), warmup (simulated seconds before the survey, default 5), frame_ms (default 100), window_seconds
--            (the frames recorded after the failure, default 12), bridge (the bridge's ordinal in the entity list; default the first with a candidate), segment (the segment's index along it;
--            default the first by the criterion), criterion (0, the only one: a segment that is alive, not an unsupported mid section, next to one)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local out = assert (io.open (assert (args.out, "out=<JSON Lines path>"), "w"))
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario_name = args.scenario or "lebanon_retail"
local scenario = assert (scenarios [scenario_name], "scenario=lebanon_retail|georgia_retail")
local warmup = tonumber (args.warmup or "5")
local frame_ms = tonumber (args.frame_ms or "100")
local window_seconds = tonumber (args.window_seconds or "12")
local criterion = tonumber (args.criterion or "0")
local seed = 1

local here = (debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or "")
local writer = dofile (here .. "combat-report.lua") (out)
local emit, emit_report = writer.emit, writer.emit_report

local dc = require ("eech_dc")
dc.prepare_installation (root)
engine = dc.boot {
	install_root = root, map = scenario.map, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = seed, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = scenario_name, case = "bridge_collapse", warmup = warmup, frame_ms = frame_ms, window_seconds = window_seconds, criterion = criterion, seed = seed }

local clock = engine:clock ()
local function step_frame ()
	engine:frame (frame_ms)
	clock = engine:clock ()
end

local function fields (text)
	local t = {}
	for k, v in string.gmatch (text, "(%S+)=(%S+)") do t [k] = v end
	return t
end

-- the war before the case (kept, under its own scope, so that the events read after the failure are the case's and none is lost)
local function drain (scope)
	emit_report (scope, clock.elapsed_seconds, engine:cmb_events ())
end

while clock.elapsed_seconds < warmup do step_frame () end
drain ("pre:warmup")

-- the CASE ENTRY: op, ordinal, index, criterion
local function bridge_case (op, ordinal, index)
	return engine:cmb_case_bridge (op, ordinal, index, criterion)
end

local survey = bridge_case (0, 0, -1)
emit_report ("bridge:survey", clock.elapsed_seconds, survey)

-- the bridge: the one asked for, else the first with a segment next to an unsupported mid section
local chosen
for line in string.gmatch (survey, "[^\n]+") do
	local f = fields (line)
	if line:find ("^cmb%-bridge%-survey") and f.first_candidate and tonumber (f.first_candidate) >= 0 then
		local ordinal = tonumber (f.bridge)
		if args.bridge == nil and chosen == nil then chosen = ordinal end
		if args.bridge ~= nil and ordinal == tonumber (args.bridge) then chosen = ordinal end
	end
end

if chosen == nil then
	emit { event = "bridge_case", done = false, reason = args.bridge and "the_asked_bridge_has_no_segment_next_to_an_unsupported_span" or "no_bridge_with_a_segment_next_to_an_unsupported_span", scenario = scenario_name }
	emit_report ("run", clock.elapsed_seconds, engine:cmb_trace ())
	out:close ()
	print ("BRIDGE_CASE_NOT_DONE")
	return
end

local segment = args.segment and tonumber (args.segment) or -1

emit_report ("bridge:trace:before", clock.elapsed_seconds, engine:cmb_trace ())
emit_report ("bridge:before", clock.elapsed_seconds, bridge_case (1, chosen, -1))

drain ("pre:failure")
local failure = bridge_case (2, chosen, segment)
local f = fields (failure)
emit { event = "bridge_case", done = f.done == "1", reason = f.reason or "", bridge = chosen, segment = tonumber (f.index) or -1, scenario = scenario_name, text = (failure:gsub ("\n", " ")) }
emit_report ("bridge:after", clock.elapsed_seconds, bridge_case (1, chosen, -1))
emit_report ("frame:0:log", clock.elapsed_seconds, engine:cmb_events ())

if f.done == "1" then
	local frames = math.ceil (window_seconds * 1000 / frame_ms)
	for n = 1, frames do
		step_frame ()
		emit_report ("frame:" .. n .. ":state", clock.elapsed_seconds, bridge_case (1, chosen, -1))
		emit_report ("frame:" .. n .. ":log", clock.elapsed_seconds, engine:cmb_events ())
	end
end

emit_report ("bridge:disarm", clock.elapsed_seconds, bridge_case (3, 0, -1))
emit_report ("bridge:trace:after", clock.elapsed_seconds, engine:cmb_trace ())
emit_report ("run", clock.elapsed_seconds, engine:cmb_trace ())
out:close ()
print ("BRIDGE_CASE_DONE")
