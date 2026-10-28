-- TEST-ONLY physical combat-resolution characterisation (EEDCS-46). Needs a DLL built with the `weapon-fixture` feature; the shipping module has no `cmb_trace` or `cmb_events` method.
--
-- It boots the campaign through the public path (require ("eech_dc")) with the reference host's callbacks (all six: the host is complete, as in every other scenario), runs frames,
-- and reads from the fixture what the campaign's own update loop did with damage: every weapon damage-capability computation (armour penetration, high-explosive splash, the
-- no-weapon forecasts the weapon selection makes), every collision delivered to a mobile or fixed entity (the damage applied, the health before and after, whether the receiver
-- was destroyed), every explosion blast (the radius, each object examined in order, who was notified, rockets destroyed) and every repair write of the damage level; as four
-- hashes, counters per entity kind, and an ORDERED event log drained at every checkpoint and at the end, one JSON line each. The campaign decides what is fired and when; the
-- fixture only observes. There is no probe: no combat function is called from outside the game's own order.
--
-- host.args: root, scenario (lebanon_retail default, georgia_retail, georgia), seconds (default 600), frame_ms (default 100), checkpoint_every (simulated seconds, default 60;
--            0 disables), out (JSON Lines path)

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
local seed = 1

-- the JSON Lines writer and the report parser are a file of their own, next to this one, so that the host tests load the same code (lua/combat-report.lua)
local here = (debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or "")
local report_writer = dofile (here .. "combat-report.lua") (out)
local emit, emit_report = report_writer.emit, report_writer.emit_report

local function emit_events (at)
	emit_report ("log", at, engine:cmb_events ())
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
engine = dc.boot {
	install_root = root, map = scenario.map, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = seed, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
	line_of_sight = host.line_of_sight,
	sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = args.scenario or "lebanon_retail", seconds = seconds, frame_ms = frame_ms, checkpoint_every = checkpoint_every, seed = seed }
-- the boot-time ballistic table simulation has run by now: its facts are in the report taken here (and in every later one: the report is run-wide, never reset)
emit_report ("boot", 0, engine:cmb_trace ())
emit_events (0)

local expected_frames = math.ceil (seconds * 1000 / frame_ms)
local frame_limit = expected_frames + 10
local next_checkpoint = checkpoint_every
local checkpoints = 0
local frames = 0
local clock = engine:clock ()
while clock.elapsed_seconds < seconds do
	if frames >= frame_limit then
		error (string.format ("the clock reached %s s after %d frames, not the %s s endpoint within %d frames", tostring (clock.elapsed_seconds), frames, tostring (seconds), frame_limit))
	end
	engine:frame (frame_ms)
	frames = frames + 1
	clock = engine:clock ()
	if checkpoint_every > 0 and clock.elapsed_seconds >= next_checkpoint then
		emit { event = "checkpoint", at = next_checkpoint, frame = frames, elapsed = clock.elapsed_seconds }
		emit_events (next_checkpoint)
		emit_report ("checkpoint", next_checkpoint, engine:cmb_trace ())
		checkpoints = checkpoints + 1
		next_checkpoint = next_checkpoint + checkpoint_every
	end
end

emit_events (clock.elapsed_seconds)
local final = engine:cmb_trace ()
emit { event = "run", elapsed = clock.elapsed_seconds, frames = frames, expected_frames = expected_frames, checkpoints = checkpoints, seconds = seconds, frame_ms = frame_ms }
emit_report ("run", clock.elapsed_seconds, final)
out:close ()
local damage_hash, collision_hash, blast_hash, repair_hash = string.match (final, "cmb%-hash damage=(%x+) collision=(%x+) blast=(%x+) repair=(%x+)")
io.stderr:write (string.format ("combat-characterization: damage %s collision %s blast %s repair %s, %d frames, %d checkpoints, clock %s\n", tostring (damage_hash), tostring (collision_hash),
	tostring (blast_hash), tostring (repair_hash), frames, checkpoints, tostring (clock.elapsed_seconds)))
