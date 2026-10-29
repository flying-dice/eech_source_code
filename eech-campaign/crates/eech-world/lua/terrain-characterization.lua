-- TEST-ONLY terrain surface characterisation (EEDCS-47). Needs a DLL built with the `terrain-fixture` feature; the shipping module
-- has no `terrain_trace`, `terrain_trace_reset` or `terrain_probe` method.
--
-- It boots the campaign through the public path (require ("eech_dc")) with the reference host's callbacks, runs frames, and reads
-- two things from the fixture (see csrc/eech_terrain_fixture.c):
--   * the run-wide report: every terrain query the campaign's own code made, first for the boot (campaign and force loading), then for
--     the frames: counts by shared or private find-fast record, hint tried and hit, sector scans, no face found, surface-type
--     histogram and the call-site table. The campaign decides what to ask; the fixture only observes.
--   * a read-only deterministic sweep of the loaded map at chosen simulated times, with the answer of a fresh record as the reference
--     and the same points asked again with a hint left by an earlier query (the order dependence of the original solver), through the
--     wrappers the campaign calls, and the surface type, class and ground-vehicle suitability of every type met. Its report is
--     separate from the run-wide one and it leaves the shared find-fast record as it found it.
-- The terrain is the campaign's own, loaded from the installation: no host callback is involved in the sweep. The same script is run on
-- the unchanged solver and on a later extraction candidate; a difference in a hash, count or sample is a difference in behaviour, and
-- nothing here is tuned to make a run pass.
--
-- host.args: root, scenario (lebanon_retail default, georgia_retail, georgia), seconds (default 600), frame_ms (default 100),
--            probe_at (comma list of simulated seconds, default 0,300; 0 is the loaded, unstepped world; an explicitly empty probe_at= is
--            the no-probe control, and any requested time that is never reached fails the run), probe_seed (default 1), probe_count
--            (default 20000), emit (default 64, at most 256: samples written in full), out (JSON Lines path)

-- Every argument is validated here, before the output file is opened, the module is required or anything is booted, so a refused
-- run has no side effect. Bounds of the fixture (defaults are inside them and unchanged):
--   frame_ms     integer 10 .. 1000                         seconds     decimal 0 .. 36000 (10 h of simulated time); 0 is the instantaneous mode
--   frames       floor (seconds * 1000 / frame_ms): 1 .. 1000000 for a positive duration; exactly 0 for seconds=0, where no frame is run
--                and so no clock-progress check applies (the boot and the probes at time 0 still run, and every requested probe must be taken)
--   probe_at     "" (no probe) or a strictly ascending list of non-negative decimals, each at most frames * frame_ms / 1000;
--                0 is only the first and is taken before any frame
--   probe_seed   integer 0 .. 4294967295 (a u32; the C sweep deliberately treats a seed of 0 as 1, so 0 and 1 give the same sweep)
--   probe_count  integer 0 .. 200000 (the native entry refuses more)     emit  integer 0 .. 256 (the native report holds at most 256)
--   scenario     one of the three below                      root, out   non-empty strings
local function refuse (message)
	error ("terrain-characterization: " .. message, 0)
end

local args = host.args
if type (args) ~= "table" then refuse ("host.args is not a table") end

local function text_arg (name, default)
	local value = args [name]
	if value == nil then value = default end
	if type (value) ~= "string" then refuse (name .. " is not a string") end
	return value
end

-- a plain non-negative decimal: digits, optionally one point and digits; no sign, exponent, hex, spaces, inf or nan
local function decimal (name, text)
	if not (string.match (text, "^%d+$") or string.match (text, "^%d+%.%d+$")) then
		refuse (string.format ("%s=%q is not a plain non-negative decimal number", name, text))
	end
	local value = tonumber (text)
	if value == nil or value ~= value or value == math.huge then refuse (string.format ("%s=%q is not finite", name, text)) end
	return value
end

local function integer_arg (name, default, minimum, maximum)
	local text = text_arg (name, default)
	if not string.match (text, "^%d+$") then refuse (string.format ("%s=%q is not a plain non-negative integer", name, text)) end
	local value = tonumber (text)
	if value == nil or value ~= value or value == math.huge or value < minimum or value > maximum then
		refuse (string.format ("%s=%q is outside %s .. %s", name, text, tostring (minimum), tostring (maximum)))
	end
	return value
end

local root = text_arg ("root", "")
if root == "" then refuse ("root=<installation root> is required and not empty") end
local out_path = text_arg ("out", "")
if out_path == "" then refuse ("out=<JSON Lines path> is required and not empty") end

local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
	georgia = { map = "..\\common\\maps\\map16", campaign = "georgia.chc" },
}
local scenario_name = text_arg ("scenario", "lebanon_retail")
local scenario = scenarios [scenario_name]
if scenario == nil then refuse (string.format ("scenario=%q is not lebanon_retail, georgia_retail or georgia", scenario_name)) end

local seconds = decimal ("seconds", text_arg ("seconds", "600"))
if seconds > 36000 then refuse (string.format ("seconds=%s is outside 0 .. 36000", args.seconds or "600")) end
local frame_ms = integer_arg ("frame_ms", "100", 10, 1000)
local frames = math.floor (seconds * 1000 / frame_ms + 1e-9)
-- a positive duration needs at least one bounded frame; only a duration of exactly 0 runs none (a boot and an instantaneous sweep)
if seconds == 0 then frames = 0 end
if (seconds > 0 and frames < 1) or frames > 1000000 then refuse (string.format ("seconds / frame_ms gives %d frames, outside 1 .. 1000000", frames)) end
local simulated_total = frames * frame_ms / 1000

local probe_seed = integer_arg ("probe_seed", "1", 0, 4294967295)
local probe_count = integer_arg ("probe_count", "20000", 0, 200000)
local emit_samples = integer_arg ("emit", "64", 0, 256)

-- an explicitly empty probe_at= is the no-probe control; any other empty, malformed, repeated, descending or unreachable token is refused
local probe_at = {}
local probe_text = text_arg ("probe_at", "0,300")
if probe_text ~= "" then
	for token in string.gmatch (probe_text .. ",", "([^,]*),") do
		local at = decimal ("probe_at token", token)
		if #probe_at > 0 and at <= probe_at [#probe_at] then
			refuse (string.format ("probe_at=%q is not strictly ascending at %q", probe_text, token))
		end
		if at > simulated_total then
			refuse (string.format ("probe_at time %s is beyond the %g simulated seconds the run covers", token, simulated_total))
		end
		probe_at [#probe_at + 1] = at
	end
end

io.stdout:setvbuf ("no")
local out = assert (io.open (out_path, "w"))

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

-- hashes and bit patterns are hex: always strings, even when every digit is decimal; names and paths are strings too
local string_keys = {
	run = true, sample = true, primed = true, x = true, z = true, result = true, last = true, nx = true, ny = true, nz = true,
	file = true, name = true, fast = true,
}

-- one JSON object per report line: the first word is the event, `key=value` words are its fields
local function emit_report (scope, at, report)
	for line in string.gmatch (report, "[^\n]+") do
		local event, rest = string.match (line, "^(%S+)%s*(.*)$")
		local t = { event = event, report = scope, at = at }
		for k, v in string.gmatch (rest, "(%S+)=(%S+)") do
			t [k] = (not string_keys [k] and tonumber (v)) or v
		end
		emit (t)
		-- every query is a hint hit, a Cuba-area return or a full sector scan (found or not): the three counts add up to the calls
		if event == "terrain-calls" and t.partition_ok ~= 1 then
			refuse ("the " .. scope .. " report's fast_hit + slow_scans + cuba does not equal calls")
		end
	end
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = scenario.map, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
	line_of_sight = host.line_of_sight,
	sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = scenario_name, seconds = seconds, frame_ms = frame_ms, probe_seed = probe_seed, probe_count = probe_count }

-- the queries made while the campaign and forces loaded, before the first frame, then a clean run-wide trace for the frames
emit_report ("boot", 0, engine:terrain_trace ())
engine:terrain_trace_reset ()

local probes_taken = 0
local function take_probe (at, frame, elapsed)
	local before = engine:terrain_trace ()
	local report = engine:terrain_probe (probe_seed, probe_count, emit_samples)
	probes_taken = probes_taken + 1
	emit { event = "probe", at = at, frame = frame, elapsed = elapsed }
	emit_report ("probe", at, report)
	-- the probe must leave the run-wide trace, and so the shared find-fast record's observable effect on it, exactly as it found it
	local after = engine:terrain_trace ()
	local untouched = before == after
	emit { event = "run_trace_untouched_by_probe", at = at, ok = untouched }
	assert (untouched, "terrain-characterization: the probe changed the run-wide trace")
	assert (string.find (report, "shared_record_restored=1", 1, true), "terrain-characterization: the probe did not restore the shared record")
end

local next_probe = 1
if probe_at [next_probe] == 0 then
	take_probe (0, 0, engine:clock ().elapsed_seconds)
	next_probe = next_probe + 1
end

local previous_elapsed = engine:clock ().elapsed_seconds
for f = 1, frames do
	engine:frame (frame_ms)
	local clock = engine:clock ()
	-- the simulated clock must move forward with every frame, or the run is not the one that was asked for
	if not (clock.elapsed_seconds > previous_elapsed) then
		refuse (string.format ("the simulated clock did not advance at frame %d (%s after %s)", f, tostring (clock.elapsed_seconds), tostring (previous_elapsed)))
	end
	previous_elapsed = clock.elapsed_seconds
	-- every probe now due is taken at this frame, in order, each at its requested time; the list is finite and next_probe only grows
	while next_probe <= #probe_at and clock.elapsed_seconds >= probe_at [next_probe] do
		take_probe (probe_at [next_probe], f, clock.elapsed_seconds)
		next_probe = next_probe + 1
	end
end

-- every requested probe time must have been reached; an explicitly empty probe_at is the no-probe control
assert (next_probe > #probe_at, string.format ("terrain-characterization: only %d of %d requested probes were taken before the run ended", next_probe - 1, #probe_at))

local final = engine:terrain_trace ()
local clock = engine:clock ()
emit { event = "run", elapsed = clock.elapsed_seconds, frames = frames, probes = probes_taken }
emit_report ("run", clock.elapsed_seconds, final)
out:close ()
local hash = string.match (final, "terrain%-hash scope=run run=(%x+)")
io.stderr:write (string.format ("terrain-characterization: run hash %s, %d probes\n", tostring (hash), probes_taken))
