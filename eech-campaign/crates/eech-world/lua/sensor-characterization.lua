-- TEST-ONLY sensor-detection characterisation (EEDCS-43). Needs a DLL built with the `sensor-fixture` feature; the shipping module
-- has no `sensor_trace`, `sensor_trace_reset` or `sensor_probe` method.
--
-- It boots the campaign through the public path (require ("eech_dc")) with the reference host's callbacks, including the required
-- line_of_sight provider (EEDCS-42; the read-only sweep is handed the same complete host as a frame), runs frames, and reads
-- two things from the fixture:
--   * the run-wide report: every AI target scan the campaign's own update loop made (aircraft and vehicle scans, see
--     csrc/eech_sensor_fixture.c), as a physical-fact hash, a decision hash, per-scan-function counters, which edges the run
--     exercised and sample candidates. The campaign decides what to scan; the fixture only observes.
--   * a read-only sweep at chosen simulated times: the same scan group functions run once for every eligible live source against the
--     live world, with no engagement, speech or timer change. Its report is separate from the run-wide one.
-- The same script is run on the unchanged scans and on a later extraction candidate; a difference in a hash, count or sample is a
-- difference in behaviour, and nothing here is tuned to make a run pass.
--
-- host.args: root, scenario (lebanon_retail default, georgia_retail, georgia), seconds (default 600), frame_ms (default 100),
--            probe_at (comma list of simulated seconds, default 30,300; an explicitly empty probe_at= is the no-probe control, and any
--            requested time that is never reached fails the run), out (JSON Lines path),
--            sense (default: the reference host's `host.sense`, the characterisation and its oracle; `not_perceived` or `own_model` boot a host
--            of its own model instead, as in lua/sense-provider.lua, to show that the fixture observes a host's generic nondetection verdict
--            instead of stopping on it: not an oracle run)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local seconds = tonumber (args.seconds or "600")
local frame_ms = tonumber (args.frame_ms or "100")
local out = assert (io.open (assert (args.out, "out=<JSON Lines path>"), "w"))

local probe_at = {}
for s in string.gmatch (args.probe_at or "30,300", "[^,]+") do probe_at [#probe_at + 1] = tonumber (s) end

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

-- the hashes are 16 hex digits: always a string, even when every digit is decimal
local string_keys = { physical = true, decision = true }

-- one JSON object per report line: the first word is the event, `key=value` words are its fields; a sample line keeps its text whole
local function emit_report (scope, at, report)
	for line in string.gmatch (report, "[^\n]+") do
		local event, rest = string.match (line, "^(%S+)%s*(.*)$")
		local t = { event = event, scope = scope, at = at }
		if event == "sensor-sample" then
			local key, text = string.match (rest, "^key=(%S+)%s+(.*)$")
			t.key = key
			t.text = text
		else
			for k, v in string.gmatch (rest, "(%S+)=(%S+)") do
				t [k] = (not string_keys [k] and tonumber (v)) or v
			end
		end
		emit (t)
	end
end

-- the reference host unless the run asks for a host of its own model (never in an oracle run)
local sense = host.sense
if args.sense == "not_perceived" or args.sense == "own_model" then
	local source
	sense = function (step)
		if step.mode == "begin" then source = step.source_position return { state = "open" } end
		if step.mode == "end" then return { state = "closed" } end
		assert (step.mode == "candidate", "a host that decides at the candidate is never asked for a draw or a signature")
		if args.sense == "own_model" then
			local dx, dz = step.target_position [1] - source [1], step.target_position [3] - source [3]
			local distance = math.sqrt (dx * dx + dz * dz)
			if distance < 4000.0 then return { state = "accept", target_range = distance } end
		end
		return { state = "reject", reason = "not_perceived" }
	end
else
	assert (args.sense == nil, "sense=not_perceived|own_model, or omit it for the reference host")
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
	line_of_sight = host.line_of_sight,
	sense = sense, aim = host.aim,
}
emit { event = "boot", scenario = args.scenario or "lebanon_retail", seconds = seconds, frame_ms = frame_ms }
engine:sensor_trace_reset ()

local frames = math.floor (seconds * 1000 / frame_ms)
local next_probe = 1
local probes_taken = 0
for f = 1, frames do
	engine:frame (frame_ms)
	local clock = engine:clock ()
	if next_probe <= #probe_at and clock.elapsed_seconds >= probe_at [next_probe] then
		local before = engine:sensor_trace ()
		local report = engine:sensor_probe ()
		probes_taken = probes_taken + 1
		emit { event = "probe", at = probe_at [next_probe], frame = f, elapsed = clock.elapsed_seconds }
		emit_report ("probe", probe_at [next_probe], report)
		emit_report ("run-at-probe", probe_at [next_probe], before)
		-- the probe must leave the run-wide report exactly as it found it
		local after = engine:sensor_trace ()
		local untouched = before == after
		emit { event = "run_trace_untouched_by_probe", at = probe_at [next_probe], ok = untouched }
		assert (untouched, "sensor-characterization: the probe changed the run-wide trace")
		next_probe = next_probe + 1
	end
end

-- every requested probe time must have been reached; an explicitly empty probe_at is the no-probe control
assert (next_probe > #probe_at, string.format ("sensor-characterization: only %d of %d requested probes were taken before the run ended", next_probe - 1, #probe_at))

local final = engine:sensor_trace ()
local clock = engine:clock ()
emit { event = "run", elapsed = clock.elapsed_seconds, frames = frames, probes = probes_taken }
emit_report ("run", clock.elapsed_seconds, final)
out:close ()
local physical, decision = string.match (final, "sensor%-hash physical=(%x+) decision=(%x+)")
io.stderr:write (string.format ("sensor-characterization: physical %s decision %s, %d probes\n", tostring (physical), tostring (decision), probes_taken))
