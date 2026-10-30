-- TEST-ONLY physical weapon-execution characterisation (EEDCS-45). Needs a DLL built with the `weapon-fixture` feature; the shipping module has no
-- `wpn_trace`, `wpn_events`, `wpn_case_query` or `wpn_case_launch` method.
--
-- It boots the campaign through the public path (require ("eech_dc")) with the reference host's callbacks (all six: the host is complete, as in every other
-- scenario), runs frames, and reads from the fixture:
--   * the run-wide report of the weapon-execution code the campaign's own update loop ran: the boot-time ballistic table simulation and its tables, every weapon
--     birth (launch kinematics), every weapon_movement call (guidance, branch, contact, terrain, lifetime), the per-update order and the terminal facts, as five
--     hashes (table, launch, flight, terminal, decision), counters, value ranges, per-weapon and per-table rows and sample records;
--   * the ORDERED event log (table summaries at boot, launch outcomes, births, terminals, destroys) drained at every checkpoint and at the end, one JSON line each.
-- The campaign decides what is launched and when; the fixture only observes. There is no probe: no weapon function is called from outside the game's own order.
-- The report is also taken at fixed simulated checkpoints (default every 60 s), so a later candidate that diverges can be dated.
--
-- The run ends at the FIRST frame whose clock reaches the requested endpoint (not after a precomputed frame count): the clock accumulates in float, so a fixed
-- count can stop one frame short of or past the endpoint. A run that has not reached the endpoint after the expected frames plus a margin is an error.
--
-- host.args: root, scenario (lebanon_retail default, georgia_retail, georgia), seconds (default 600), frame_ms (default 100), checkpoint_every (simulated seconds,
--            default 60; 0 disables), out (JSON Lines path)

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

-- a string as a JSON string with every control character, quote, backslash and non-ASCII byte escaped losslessly (a byte >= 0x80 is written as the code point of the same value, i.e.
-- the Latin-1 reading): nothing in the recorded text is replaced or dropped
local escapes = { ['"'] = '\\"', ['\\'] = '\\\\', ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t' }
local function json_string (s)
	return '"' .. (string.gsub (s, '[%c"\\\128-\255]', function (c) return escapes [c] or string.format ("\\u%04x", string.byte (c)) end)) .. '"'
end

local function number_or_string (v)
	-- a non-finite number is not valid JSON: keep it as the string it printed as, so the comparator sees it rather than a parse error
	if v ~= v or v == math.huge or v == -math.huge then
		return json_string (tostring (v))
	end
	return tostring (v)
end

local function emit (t)
	local parts = {}
	for k, v in pairs (t) do
		local value = type (v) == "string" and json_string (v) or number_or_string (v)
		parts [#parts + 1] = json_string (k) .. ":" .. value
	end
	table.sort (parts)
	out:write ("{" .. table.concat (parts, ",") .. "}\n")
	out:flush ()
end

-- hashes (16 hex digits) and the track hash are always strings, even when every digit is decimal
local string_keys = { table = true, launch = true, flight = true, terminal = true, decision = true, hash = true, track = true }

-- one JSON object per report line: the first word is the event, `key=value` words are its fields; a sample line keeps its text whole
local function emit_report (scope, at, report)
	for line in string.gmatch (report, "[^\n]+") do
		local event, rest = string.match (line, "^(%S+)%s*(.*)$")
		local t = { event = event, scope = scope, at = at }
		if event == "wpn-sample" then
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

local function emit_events (at)
	emit_report ("log", at, engine:wpn_events ())
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
host.use_reference_arithmetic () -- the reference host's arithmetic profile, selected right after the installation is prepared (src/reference_arithmetic.rs)
engine = dc.boot {
	install_root = root, map = scenario.map, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = seed, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
	line_of_sight = host.line_of_sight, model = host.model,
	sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = args.scenario or "lebanon_retail", seconds = seconds, frame_ms = frame_ms, checkpoint_every = checkpoint_every, seed = seed }
-- the boot-time ballistic table simulation has run by now: its facts are in the report taken here (and in every later one: the report is run-wide, never reset)
emit_report ("boot", 0, engine:wpn_trace ())
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
		emit_report ("checkpoint", next_checkpoint, engine:wpn_trace ())
		checkpoints = checkpoints + 1
		next_checkpoint = next_checkpoint + checkpoint_every
	end
end

emit_events (clock.elapsed_seconds)
local final = engine:wpn_trace ()
emit { event = "run", elapsed = clock.elapsed_seconds, frames = frames, expected_frames = expected_frames, checkpoints = checkpoints, seconds = seconds, frame_ms = frame_ms }
emit_report ("run", clock.elapsed_seconds, final)
out:close ()
local table_hash, launch_hash, flight_hash, terminal_hash, decision_hash =
	string.match (final, "wpn%-hash table=(%x+) launch=(%x+) flight=(%x+) terminal=(%x+) decision=(%x+)")
io.stderr:write (string.format ("weapon-characterization: table %s launch %s flight %s terminal %s decision %s, %d frames, %d checkpoints, clock %s\n", tostring (table_hash),
	tostring (launch_hash), tostring (flight_hash), tostring (terminal_hash), tostring (decision_hash), frames, checkpoints, tostring (clock.elapsed_seconds)))
