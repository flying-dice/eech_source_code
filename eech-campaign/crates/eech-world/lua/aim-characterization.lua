-- TEST-ONLY physical weapon-aiming characterisation (EEDCS-44). Needs a DLL built with the `aim-fixture` feature; the shipping module
-- has no `aim_trace` or `aim_trace_reset` method.
--
-- It boots the campaign through the public path (require ("eech_dc")) with the reference host's callbacks (all six: the host is complete, as in
-- every other scenario), runs frames, and reads the run-wide report from the fixture:
--   * the aiming code the campaign's own update loop ran (the weapon-system update, the vector update, the pitch-device and lead/ballistic solvers,
--     the ballistic table, the fire decision's reads of the launch-angle error and the person target-facing), as a physical hash, a decision hash,
--     counters, measured value ranges, the entity kinds and weapons that aimed, and sample records. The campaign decides what aims; the fixture only
--     observes. There is no probe: no aiming function is called from outside the game's own order.
-- The report is also taken at fixed simulated checkpoints (default every 60 s), so a later candidate that diverges can be dated.
-- The same script is run on the unchanged code and on a later candidate; a difference in a hash, count, range or sample is a difference in
-- behaviour, and nothing here is tuned to make a run pass.
--
-- host.args: root, scenario (lebanon_retail default, georgia_retail, georgia), seconds (default 600), frame_ms (default 100),
--            checkpoint_every (simulated seconds, default 60; 0 disables), out (JSON Lines path)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local seconds = tonumber (args.seconds or "600")
local frame_ms = tonumber (args.frame_ms or "100")
local checkpoint_every = tonumber (args.checkpoint_every or "60")
local out = assert (io.open (assert (args.out, "out=<JSON Lines path>"), "w"))

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
		if event == "aim-sample" then
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

local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
	line_of_sight = host.line_of_sight,
	sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = args.scenario or "lebanon_retail", seconds = seconds, frame_ms = frame_ms, checkpoint_every = checkpoint_every }
engine:aim_trace_reset ()

local frames = math.floor (seconds * 1000 / frame_ms)
local next_checkpoint = checkpoint_every
local checkpoints = 0
for f = 1, frames do
	engine:frame (frame_ms)
	if checkpoint_every > 0 then
		local clock = engine:clock ()
		if clock.elapsed_seconds >= next_checkpoint then
			emit { event = "checkpoint", at = next_checkpoint, frame = f, elapsed = clock.elapsed_seconds }
			emit_report ("checkpoint", next_checkpoint, engine:aim_trace ())
			checkpoints = checkpoints + 1
			next_checkpoint = next_checkpoint + checkpoint_every
		end
	end
end

local final = engine:aim_trace ()
local clock = engine:clock ()
emit { event = "run", elapsed = clock.elapsed_seconds, frames = frames, checkpoints = checkpoints }
emit_report ("run", clock.elapsed_seconds, final)
out:close ()
local physical, decision = string.match (final, "aim%-hash physical=(%x+) decision=(%x+)")
io.stderr:write (string.format ("aim-characterization: physical %s decision %s, %d checkpoints\n", tostring (physical), tostring (decision), checkpoints))
