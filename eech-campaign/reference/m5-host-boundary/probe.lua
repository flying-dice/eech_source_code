-- M5 host-boundary characterisation probe (no product change).
-- Run by eech-world.exe through the public path: require ("eech_dc").
-- The host gives the module only boot configuration and elapsed milliseconds.
--
-- host.args: root, seconds (simulated), observe (1: read engine:objects () every
-- 10 frames, as campaign.lua does; 0: never), out (JSON Lines path),
-- unit and group (entity ids to follow), diag_at (comma-separated simulated
-- seconds at which to write the campaign's force-state diagnostics to stderr)

local args = host.args
local root = assert (args.root, "root=")
local seconds = tonumber (args.seconds or "1800")
local observe = args.observe ~= "0"
local unit, group = tonumber (args.unit or "87568"), tonumber (args.group or "87567")
local diag_at = {}
for s in string.gmatch (args.diag_at or "", "[^,]+") do diag_at[#diag_at + 1] = tonumber (s) end

local out = assert (io.open (assert (args.out, "out="), "w"))
local function emit (t)
	local parts = {}
	for k, v in pairs (t) do
		local value = type (v) == "string" and string.format ("%q", v) or tostring (v)
		parts[#parts + 1] = string.format ("%q:%s", k, value)
	end
	table.sort (parts)
	out:write ("{" .. table.concat (parts, ",") .. "}\n")
	out:flush ()
end

local dc = require ("eech_dc")

-- the public surface, as the host sees it
local exports = {}
for k, v in pairs (dc) do exports[#exports + 1] = k .. ":" .. type (v) end
table.sort (exports)
emit { event = "surface", module = dc.name, exports = table.concat (exports, " ") }

dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = "..\\common\\maps\\map5", campaign_directory = "camp01",
	campaign = "LEBANON.CHC", gunship = "apache", seed = 1, bind_mobile = host.bind_mobile,
	supply_reached = host.supply_reached,
}

-- Retain the original guessed event/report surface checks. Host binding now
-- enters through the required boot callback above; these unrelated names stay absent.
for _, name in ipairs { "event", "report", "observe", "on_event", "host" } do
	local ok, err = pcall (function () return engine[name] end)
	emit { event = "engine_index", name = name, ok = ok, value = ok and type (err) or tostring (err) }
end

local frame_ms, frames = 100, math.floor (seconds * 1000 / 100)
local next_diag, unit_alive, unit_seen, members_before = 1, nil, false, nil
local function members_of (objects)
	local m = {}
	for _, o in ipairs (objects) do
		if o.group_id == group and o.kind ~= "weapon" and o.alive then m[#m + 1] = o.id end
	end
	table.sort (m)
	return table.concat (m, " ")
end

for f = 1, frames do
	engine:frame (frame_ms)
	local clock = engine:clock ()
	if next_diag <= #diag_at and clock.elapsed_seconds >= diag_at[next_diag] then
		io.stderr:write (string.format ("=== DIAGNOSTICS at elapsed %.2f s (frame %d) ===\n", clock.elapsed_seconds, f))
		engine:diagnostics ()
		io.stderr:write ("=== END DIAGNOSTICS ===\n")
		emit { event = "diagnostics", elapsed = clock.elapsed_seconds, frame = f }
		next_diag = next_diag + 1
	end
	if observe and f % 10 == 0 then
		local objects = engine:objects ()
		local alive
		for _, o in ipairs (objects) do
			if o.id == unit then alive = o.alive; unit_seen = true end
		end
		local members = members_of (objects)
		if alive ~= unit_alive or members ~= members_before then
			emit { event = "change", elapsed = clock.elapsed_seconds, frame = f, unit_alive = tostring (alive), group_members = members }
			unit_alive, members_before = alive, members
		end
	end
end
local clock = engine:clock ()
emit { event = "end", elapsed = clock.elapsed_seconds, frames = frames, observed = observe, unit_seen = unit_seen }
out:close ()
