-- TEST-ONLY controlled witness of the ORIGINAL bridge and segment collapse (EEDCS-46, R5). Needs a DLL built with the `weapon-fixture` feature. RUN, ON THE ORIGINAL: the root built it and ran it twice
-- on the original code (native original99734/PID10104 and repeat81402/PID17440, both exit 0, 320598 bytes with the identical SHA d4c198af..., the survey of 118 bridges, bridge 0 of 8 segments chosen by
-- the criterion; the root's root-original-bridge-pair-completion.json and the independent reviews hold the evidence). Its flow and argument safety are also checked against a stand-in engine
-- (tests/host_callbacks.rs). It moves and changes no bridge physics: it observes the campaign's own code. The protocol, the default arguments and the output are PROTECTED: the host's collapse (the R5
-- extraction, docs/m5-combat-extraction.md Step 5) is judged by running this same script, unchanged, against the new path and comparing with that original baseline.
--
-- WHAT IT DOES. From a verified retail boot (Lebanon by default, Georgia with scenario=georgia_retail), it surveys the bridges the world holds, chooses one that meets the FIRST-COLLAPSE criterion
-- (an explicit test-only criterion applied inside C: the bridge is alive, and one of its segments is alive, is not an unsupported mid section and is next to an unsupported mid section that is
-- ALIVE; no pointer, heap index or seed crosses), records its state, then applies a DECLARED DIRECT STRUCTURAL FAILURE to that segment (the campaign's own kill of one segment: this is NOT a weapon hit
-- and is never reported as one), and records, for a bounded window of frames, the bridge's and every segment's state after each frame and the ordered events the original code produced
-- (csrc/eech_cmb_case.c, csrc/eech_cmb_fixture.c: the read-only CMB_FX_BRIDGE hooks of special/bridge/br_msgs.c and fixed/segment/sg_updt.c, armed only for this case). The war runs on as it does;
-- its events are kept apart from the bridge's by scope and by their `kind=bridge-*` label.
--
-- WHAT RAN IS JUDGED FROM THE EVENTS, NEVER FROM `done`. `done` says only that the precondition was met. A path ran if and only if its `bridge-*` events are in the frame logs; `bridge_summary`
-- counts them (the unsupported-span kill, the link break, the repair task, the falls, the landings) and says how many segments were still in the update list when the window ended: a window of
-- finite length does not guarantee that a segment at an unknown height has landed, and the summary says whether it did. If the world has no bridge that meets the criterion the script says so
-- (`bridge_case` done=false, reason) and does nothing else, so the same script can be run on the other verified scenario; it never invents a bridge.
--
-- Declared and unreached: a weapon hit on a segment, a player or multiplayer cause, a client, a bridge with no unsupported span, and the vehicles on the route beyond the world's counts of routed
-- vehicles alive and repair tasks.
--
-- Output, JSON Lines (the writer and parser of lua/combat-report.lua): boot; `bridge:survey`; `bridge:before` (state); `bridge_case` (the failure's own report and whether it was done);
-- `bridge:after` (the state immediately after the failure and before any frame); for each frame n: `frame:n:state` and `frame:n:log` (the events of that frame, in order, with the engine's
-- sequence numbers); `bridge_summary`; `bridge:disarm`; the observer's whole report before and after (`bridge:trace:before`, `bridge:trace:after`); `run`.
--
-- host.args (every one is VALIDATED before the output file is created, the installation prepared, the engine booted or anything changed; a malformed value is an error naming it, never a default):
--   root, out                       required text (the installation root; the JSON Lines path)
--   scenario                        lebanon_retail (default) | georgia_retail
--   warmup                          simulated seconds before the survey: a plain decimal number >= 0, at most 600 (default 5)
--   window_seconds                  the frames recorded after the failure: a plain decimal number >= 0, at most 600 (default 12)
--   frame_ms                        an integer from 1 to 1000 (default 100). Zero never advances the clock, so it is refused
--   bridge                          the bridge's ordinal in the entity list: an integer from 0 to 100000; absent means the first with a candidate
--   segment                         the segment's index along the bridge: an integer from 0 to 255; absent means the first by the criterion (there is no other automatic sentinel)
--   criterion                       0, the only one (default 0)
-- A "plain decimal number" is digits with an optional single point (no sign, exponent, hex, space or text after it). The frames of warmup and window together may not exceed 10000, and every loop
-- below stops at its bound whatever the clock does.

io.stdout:setvbuf ("no")

local args = host.args

local MAX_FRAMES = 10000

local function refuse (message)
	error ("bridge-case: " .. message, 0)
end

-- a plain non-negative decimal (digits and at most one point) or integer (digits only), at most `maximum`; anything else is refused by name
local function number_arg (name, value, default, maximum, integer)
	if value == nil then return default end
	local text = tostring (value)
	local plain
	if integer then
		plain = text:match ("^%d+$") ~= nil
	else
		plain = text:match ("^%d+%.?%d*$") ~= nil or text:match ("^%.%d+$") ~= nil
	end
	if not plain or #text > 12 then
		refuse (name .. " must be a plain non-negative " .. (integer and "integer" or "decimal number") .. " of at most twelve characters, got '" .. text .. "'")
	end
	local number = tonumber (text)
	if number == nil or number ~= number or number == math.huge or number > maximum then
		refuse (name .. " must be at most " .. tostring (maximum) .. ", got '" .. text .. "'")
	end
	return number
end

local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario_name = args.scenario == nil and "lebanon_retail" or args.scenario
local scenario = scenarios [scenario_name]
if scenario == nil then refuse ("scenario must be lebanon_retail or georgia_retail, got '" .. tostring (scenario_name) .. "'") end
local frame_ms = number_arg ("frame_ms", args.frame_ms, 100, 1000, true)
if frame_ms < 1 then refuse ("frame_ms must be at least 1, got '" .. tostring (args.frame_ms) .. "'") end
local warmup = number_arg ("warmup", args.warmup, 5, 600, false)
local window_seconds = number_arg ("window_seconds", args.window_seconds, 12, 600, false)
local explicit_bridge = number_arg ("bridge", args.bridge, nil, 100000, true)
local explicit_segment = number_arg ("segment", args.segment, nil, 255, true)
local criterion = number_arg ("criterion", args.criterion, 0, 0, true)
local warmup_frames = math.ceil (warmup * 1000 / frame_ms)
local window_frames = math.ceil (window_seconds * 1000 / frame_ms)
if warmup_frames + window_frames > MAX_FRAMES then
	refuse ("warmup and window_seconds need " .. (warmup_frames + window_frames) .. " frames of " .. frame_ms .. " ms; the most this witness records is " .. MAX_FRAMES)
end
local seed = 1

local out = assert (io.open (out_path, "w"))

local here = (debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or "")
local writer = dofile (here .. "combat-report.lua") (out)
local emit, emit_report = writer.emit, writer.emit_report

local dc = require ("eech_dc")
dc.prepare_installation (root)
engine = dc.boot {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = seed, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = scenario_name, case = "bridge_collapse", warmup = warmup, frame_ms = frame_ms, window_seconds = window_seconds, criterion = criterion, seed = seed }

local clock = engine:clock ()
local frames_run = 0
-- one frame, with the bound and the progress both CHECKED: the clock must move forward, and no more than the budget of frames may ever be run
local function step_frame ()
	local before = clock.elapsed_seconds
	frames_run = frames_run + 1
	if frames_run > MAX_FRAMES + 4 then error ("bridge-case: more than " .. MAX_FRAMES .. " frames were run", 0) end
	engine:frame (frame_ms)
	clock = engine:clock ()
	if not (clock.elapsed_seconds > before) then error ("bridge-case: the clock did not advance in a frame of " .. frame_ms .. " ms", 0) end
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

-- the bridge: the one asked for, else the first that meets the first-collapse criterion
local chosen
for line in string.gmatch (survey, "[^\n]+") do
	local f = fields (line)
	if line:find ("^cmb%-bridge%-survey") and f.first_candidate and tonumber (f.first_candidate) >= 0 then
		local ordinal = tonumber (f.bridge)
		if explicit_bridge == nil and chosen == nil then chosen = ordinal end
		if explicit_bridge ~= nil and ordinal == explicit_bridge then chosen = ordinal end
	end
end

if chosen == nil then
	emit { event = "bridge_case", done = false, reason = explicit_bridge and "the_asked_bridge_does_not_meet_the_first_collapse_criterion" or "no_bridge_meets_the_first_collapse_criterion", scenario = scenario_name }
	emit_report ("run", clock.elapsed_seconds, engine:cmb_trace ())
	out:close ()
	print ("BRIDGE_CASE_NOT_DONE")
	return
end

-- an absent segment is the automatic choice (-1 to the case entry); a segment that was given has been validated above and is passed as it is
local segment = explicit_segment == nil and -1 or explicit_segment

emit_report ("bridge:trace:before", clock.elapsed_seconds, engine:cmb_trace ())
emit_report ("bridge:before", clock.elapsed_seconds, bridge_case (1, chosen, -1))

drain ("pre:failure")
local failure = bridge_case (2, chosen, segment)
local f = fields (failure)
emit { event = "bridge_case", done = f.done == "1", reason = f.reason or "", bridge = chosen, segment = tonumber (f.index) or -1, first_collapse = f.first_collapse == "1", scenario = scenario_name,
	text = (failure:gsub ("\n", " ")) }
-- the state immediately after the failure is KEPT: with a window of zero seconds no frame runs, and it is then the last state there is (the killed segment is in the update list at once)
local after_state = bridge_case (1, chosen, -1)
emit_report ("bridge:after", clock.elapsed_seconds, after_state)

-- the events and the last state are tallied as they are written, so that the summary says what the ORIGINAL code actually did in the window
local tally, last_state = {}, after_state
local function count_events (text)
	for kind in string.gmatch (text, "kind=bridge%-([%a%-]+)") do tally [kind] = (tally [kind] or 0) + 1 end
end
local first_events = engine:cmb_events ()
count_events (first_events)
emit_report ("frame:0:log", clock.elapsed_seconds, first_events)

if f.done == "1" then
	for n = 1, window_frames do
		step_frame ()
		last_state = bridge_case (1, chosen, -1)
		emit_report ("frame:" .. n .. ":state", clock.elapsed_seconds, last_state)
		local events = engine:cmb_events ()
		count_events (events)
		emit_report ("frame:" .. n .. ":log", clock.elapsed_seconds, events)
	end
end

-- the summary is read from the last state that was actually returned (the one after the last frame, or, with no frame, the one right after the failure). A state with no segment line in it is not a
-- finished state: it is reported as not valid, with in_update_at_end -1 and landed_all false, and never counted as "nothing left in the update list"
local in_update_at_end, segment_lines = 0, 0
for line in string.gmatch (last_state, "[^\n]+") do
	if line:find ("^cmb%-bridge%-segment ") then
		segment_lines = segment_lines + 1
		if line:find ("in_update=1", 1, true) then in_update_at_end = in_update_at_end + 1 end
	end
end
local state_valid = segment_lines > 0
if not state_valid then in_update_at_end = -1 end
local summary = { event = "bridge_summary", done = f.done == "1", first_collapse = f.first_collapse == "1", frames = f.done == "1" and window_frames or 0, in_update_at_end = in_update_at_end,
	state_valid = state_valid, landed_all = f.done == "1" and state_valid and in_update_at_end == 0 }
for kind, count in pairs (tally) do summary ["n_" .. kind:gsub ("%-", "_")] = count end
emit (summary)

emit_report ("bridge:disarm", clock.elapsed_seconds, bridge_case (3, 0, -1))
emit_report ("bridge:trace:after", clock.elapsed_seconds, engine:cmb_trace ())
emit_report ("run", clock.elapsed_seconds, engine:cmb_trace ())
out:close ()
print ("BRIDGE_CASE_DONE")
