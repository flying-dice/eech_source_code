-- Campaign metrics: what the war did, abstracted from second-by-second events,
-- for the regression test (tools/regress.sh) and run summaries.
--
-- Fed every sample with the observed objects, it keeps cumulative counts:
--   sorties     a group taking a new task, per side, kind and task, and its members
--   airframes   aircraft sorties per side and type
--   spawned     units appearing after the first minute (regen, reinforcements); the
--               campaign still creates units in its first seconds, which are its start
--   lost        units going from alive to dead, per side and kind
--   launched    weapons launched, per side and weapon
--   captures    keysites changing hands, per new owner and keysite type
-- and, at each checkpoint, a snapshot of the state: keysites held and their
-- usable states per side and type, mean keysite ammo and fuel per side and
-- type, and units alive per side and kind.

local metrics = {}
metrics.__index = metrics

local function add (t, key, n)
	t[key] = (t[key] or 0) + (n or 1)
end

function metrics.new ()
	return setmetatable ({
		samples = 0,
		units = {},		-- id -> { alive, side, kind }
		weapons = {},		-- weapon ids in flight at the last sample
		group_task = {},	-- group id -> task at the last sample
		owner = {},		-- keysite name .. type -> side
		counts = { sorties = {}, sortie_members = {}, airframes = {}, spawned = {}, lost = {}, launched = {}, captures = {} },
		checkpoints = {},
	}, metrics)
end

-- the first simulated seconds build the campaign's starting world
local WARM_UP = 60

-- one sample of the observed world at `seconds`
function metrics:sample (seconds, objects)
	local c = self.counts
	local warming_up = seconds < WARM_UP
	self.samples = self.samples + 1
	local weapons, groups, present = {}, {}, {}
	for _, o in ipairs (objects) do
		if o.kind == "weapon" then
			weapons[o.id] = true
			if not self.weapons[o.id] and self.samples > 1 then add (c.launched, o.side .. " " .. o.type_name) end
		elseif o.kind == "keysite" then
			local key = o.name .. " " .. o.type_name
			local was = self.owner[key]
			if was and was ~= o.side then add (c.captures, o.side .. " " .. o.type_name) end
			self.owner[key] = o.side
		else
			present[o.id] = true
			local u = self.units[o.id]
			if o.alive then
				-- a new entity, or an index reused after a destruction
				if (not u or not u.alive or u.kind ~= o.kind) and not warming_up then add (c.spawned, o.side .. " " .. o.kind) end
			elseif u and u.alive then
				add (c.lost, o.side .. " " .. o.kind)
			end
			self.units[o.id] = { alive = o.alive, side = o.side, kind = o.kind }
			if o.alive and o.group_id and o.task and o.task ~= "TASK_NOTHING" then
				local g = groups[o.group_id]
				if not g then g = { n = 0, task = o.task, side = o.side, kind = o.kind, types = {} }; groups[o.group_id] = g end
				g.n = g.n + 1
				add (g.types, o.type_name)
			end
		end
	end
	-- entities gone from the world while alive (landed transfers, removed wrecks) are forgotten
	for id in pairs (self.units) do if not present[id] then self.units[id] = nil end end
	self.weapons = weapons
	for id, g in pairs (groups) do
		if self.group_task[id] ~= g.task then
			local key = g.side .. " " .. g.kind .. " " .. g.task:gsub ("^TASK_", "")
			add (c.sorties, key)
			add (c.sortie_members, key, g.n)
			if g.kind == "helicopter" or g.kind == "fixed_wing" then
				for t, n in pairs (g.types) do add (c.airframes, g.side .. " " .. t, n) end
			end
		end
	end
	self.group_task = {}
	for id, g in pairs (groups) do self.group_task[id] = g.task end
end

local function copy (t)
	local r = {}
	for k, v in pairs (t) do r[k] = type (v) == "table" and copy (v) or v end
	return r
end

-- a snapshot of the state, with the counts so far
function metrics:checkpoint (seconds, objects)
	local keysites, usable, supply, alive = {}, {}, {}, {}
	for _, o in ipairs (objects) do
		if o.kind == "keysite" then
			local key = o.side .. " " .. o.type_name
			add (keysites, key)
			add (usable, key .. " " .. (o.usable or "unknown"))
			local s = supply[key] or { n = 0, ammo = 0, fuel = 0 }
			s.n, s.ammo, s.fuel = s.n + 1, s.ammo + o.ammo, s.fuel + o.fuel
			supply[key] = s
		elseif o.kind ~= "weapon" and o.alive then
			add (alive, o.side .. " " .. o.kind)
		end
	end
	local mean = {}
	for key, s in pairs (supply) do
		-- one decimal: the regression compares effects, not rounding
		mean[key .. " ammo"] = math.floor (s.ammo / s.n * 10 + 0.5) / 10
		mean[key .. " fuel"] = math.floor (s.fuel / s.n * 10 + 0.5) / 10
	end
	self.checkpoints[#self.checkpoints + 1] = {
		seconds = math.floor (seconds + 0.5),
		counts = copy (self.counts),
		keysites = keysites,
		keysite_states = usable,
		supply = mean,
		alive = alive,
	}
end

-- JSON with sorted keys, so baselines diff cleanly
local function encode (v, indent)
	indent = indent or ""
	local t = type (v)
	if t == "number" then
		if v == math.floor (v) then return string.format ("%d", v) end
		return string.format ("%.10g", v)
	elseif t == "string" then
		return '"' .. v:gsub ('[%c"\\]', function (ch) return string.format ("\\u%04x", ch:byte ()) end) .. '"'
	elseif t == "table" then
		local inner = indent .. "  "
		if #v > 0 then
			local parts = {}
			for _, x in ipairs (v) do parts[#parts + 1] = inner .. encode (x, inner) end
			return "[\n" .. table.concat (parts, ",\n") .. "\n" .. indent .. "]"
		end
		local keys = {}
		for k in pairs (v) do keys[#keys + 1] = tostring (k) end
		if #keys == 0 then return "{}" end
		table.sort (keys)
		local parts = {}
		for _, k in ipairs (keys) do parts[#parts + 1] = inner .. encode (k) .. ": " .. encode (v[k], inner) end
		return "{\n" .. table.concat (parts, ",\n") .. "\n" .. indent .. "}"
	end
	return "null"
end

function metrics:write (path, info)
	local f = assert (io.open (path, "w"))
	f:write (encode ({ run = info, checkpoints = self.checkpoints }), "\n")
	f:close ()
end

-- the sortie table, for the run log
function metrics:sortie_lines ()
	local lines = {}
	for key, n in pairs (self.counts.sorties) do
		lines[#lines + 1] = string.format ("%s: %d sorties, %d aircraft/vehicles", key, n, self.counts.sortie_members[key])
	end
	for key, n in pairs (self.counts.airframes) do lines[#lines + 1] = string.format ("by type %s: %d", key, n) end
	table.sort (lines)
	return lines
end

return metrics
