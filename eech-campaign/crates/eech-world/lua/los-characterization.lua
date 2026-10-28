-- TEST-ONLY line-of-sight characterisation (EEDCS-42). Needs a DLL built with the `los-fixture` feature; the shipping module
-- has no `los_probe` or `los_trace` method.
--
-- It boots the campaign through the public path (require ("eech_dc")) with the reference host's callbacks, runs frames, and
-- reads two things from the fixture:
--   * the run-wide trace: every line-of-sight query the campaign's own AI made (vehicle target selection, weapon choice,
--     weapon launch and guide placement), folded into a count, a visible count, per-criteria counts and an order-dependent
--     hash over each query's inputs and answer. The campaign decides what to query; the fixture only observes.
--   * a deterministic probe sweep at chosen simulated times, between pairs of live entities chosen from the world by id order,
--     over the loaded map's real terrain and objects (see csrc/eech_los_fixture.c). Its points come from a private generator,
--     so it never touches the game's random stream.
-- The same script is run on the baseline DLL and on the extraction candidate; a difference in a hash, count or answer is a
-- difference in behaviour, and nothing here is tuned to make a run pass.
--
-- host.args: root, scenario (lebanon_retail default, georgia, georgia_retail), seconds (default 600), frame_ms (default 100),
--            probe_at (comma list of simulated seconds, default 30,300; an explicitly empty probe_at= is the no-probe control, and any
--            requested time that finds no opposed pair or is never reached fails the run), count (sweep points per pair, default 500),
--            pairs (probe pairs per time, default 3), seed (sweep seed, default 20261025), out (JSON Lines path)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local seconds = tonumber (args.seconds or "600")
local frame_ms = tonumber (args.frame_ms or "100")
local count = tonumber (args.count or "500")
local pair_limit = tonumber (args.pairs or "3")
local sweep_seed = tonumber (args.seed or "20261025")
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

local function criteria_text (trace)
	local keys = {}
	for criteria in pairs (trace.by_criteria) do keys [#keys + 1] = criteria end
	table.sort (keys)
	local parts = {}
	for _, criteria in ipairs (keys) do
		local row = trace.by_criteria [criteria]
		parts [#parts + 1] = string.format ("%d=%d/%d", criteria, row.visible, row.calls)
	end
	return table.concat (parts, " ")
end

local function emit_trace (event, extra, trace)
	local t = { event = event, hash = trace.hash, calls = trace.calls, visible = trace.visible, skipped = trace.skipped,
		lookup_hash = trace.lookup_hash, lookups = trace.lookups, by_criteria = criteria_text (trace) }
	for k, v in pairs (extra) do t [k] = v end
	emit (t)
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
emit { event = "boot", scenario = args.scenario or "lebanon_retail", seconds = seconds, frame_ms = frame_ms, count = count,
	seed = sweep_seed }
engine:los_trace_reset ()

-- pairs of live mobiles in id order: a ground vehicle with a ground vehicle, a helicopter with a ground vehicle, an air defence
-- with a helicopter. Different sides first, so a pair is a plausible attacker and target.
local function pick_pairs ()
	local by_kind = { ground_vehicle = {}, helicopter = {}, air_defence = {}, fixed_wing = {} }
	for _, o in ipairs (engine:objects ()) do
		local list = by_kind [o.kind]
		if list and o.alive then list [#list + 1] = o end
	end
	for _, list in pairs (by_kind) do table.sort (list, function (a, b) return a.id < b.id end) end
	local function first_opposed (a_list, b_list, skip)
		for _, a in ipairs (a_list) do
			for _, b in ipairs (b_list) do
				if a.id ~= b.id and a.side ~= b.side and not (skip and skip [a.id .. ":" .. b.id]) then return a, b end
			end
		end
	end
	local chosen, used = {}, {}
	for _, spec in ipairs { { "ground_vehicle", "ground_vehicle" }, { "helicopter", "ground_vehicle" }, { "air_defence", "helicopter" },
			{ "fixed_wing", "ground_vehicle" } } do
		if #chosen < pair_limit then
			local a, b = first_opposed (by_kind [spec [1]], by_kind [spec [2]], used)
			if a then
				used [a.id .. ":" .. b.id] = true
				chosen [#chosen + 1] = { a = a, b = b, label = spec [1] .. "->" .. spec [2] }
			end
		end
	end
	return chosen
end

local frames = math.floor (seconds * 1000 / frame_ms)
local next_probe = 1
local probes_taken = 0
for f = 1, frames do
	engine:frame (frame_ms)
	local clock = engine:clock ()
	if next_probe <= #probe_at and clock.elapsed_seconds >= probe_at [next_probe] then
		local before = engine:los_trace ()
		local chosen = pick_pairs ()
		-- a requested probe with no supported opposed pair is an unavailable characterisation, not a quiet success
		if #chosen == 0 then
			emit { event = "probe_unavailable", at = probe_at [next_probe], frame = f, reason = "no live opposed pair of supported mobiles" }
			error (string.format ("los-characterization: probe at %s s found no supported opposed pair; characterisation unavailable", probe_at [next_probe]))
		end
		emit { event = "probe_pairs", at = probe_at [next_probe], requested = pair_limit, found = #chosen }
		for i, pair in ipairs (chosen) do
			local trace = engine:los_probe (pair.a.id, pair.b.id, sweep_seed + i, count)
			probes_taken = probes_taken + 1
			emit_trace ("probe", { at = probe_at [next_probe], frame = f, pair = pair.label, source = pair.a.id, target = pair.b.id,
				source_kind = pair.a.kind, target_kind = pair.b.kind }, trace)
		end
		local after = engine:los_trace ()
		-- the probe must leave the run-wide fold exactly as it found it
		local untouched = before.hash == after.hash and before.calls == after.calls
		emit { event = "run_fold_untouched_by_probe", ok = untouched }
		assert (untouched, "los-characterization: the probe changed the run-wide fold")
		next_probe = next_probe + 1
	end
end

-- every requested probe time must have been reached; an explicitly empty probe_at is the no-probe control
assert (next_probe > #probe_at, string.format ("los-characterization: only %d of %d requested probes were taken before the run ended", next_probe - 1, #probe_at))

local final = engine:los_trace ()
local clock = engine:clock ()
emit_trace ("run_trace", { elapsed = clock.elapsed_seconds, frames = frames, probes = probes_taken }, final)
out:close ()
io.stderr:write (string.format ("los-characterization: %d run queries, %d visible, hash %s, %d probes\n", final.calls, final.visible, final.hash,
	probes_taken))
