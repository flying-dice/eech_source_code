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

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local seconds = tonumber (args.seconds or "600")
local frame_ms = tonumber (args.frame_ms or "100")
local probe_seed = tonumber (args.probe_seed or "1")
local probe_count = tonumber (args.probe_count or "20000")
local emit_samples = tonumber (args.emit or "64")
local out = assert (io.open (assert (args.out, "out=<JSON Lines path>"), "w"))

local probe_at = {}
for s in string.gmatch (args.probe_at or "0,300", "[^,]+") do probe_at [#probe_at + 1] = tonumber (s) end

local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
	georgia = { map = "..\\common\\maps\\map16", campaign = "georgia.chc" },
}
local scenario = assert (scenarios [args.scenario or "lebanon_retail"], "scenario=lebanon_retail|georgia_retail|georgia")

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
	file = true, name = true,
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
emit { event = "boot", scenario = args.scenario or "lebanon_retail", seconds = seconds, frame_ms = frame_ms, probe_seed = probe_seed, probe_count = probe_count }

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

local frames = math.floor (seconds * 1000 / frame_ms)
for f = 1, frames do
	engine:frame (frame_ms)
	local clock = engine:clock ()
	if next_probe <= #probe_at and clock.elapsed_seconds >= probe_at [next_probe] then
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
