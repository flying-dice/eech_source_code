-- Structured observations of the world, from the same samples the Tacview
-- recorder receives (campaign.lua observe=<file>). JSON Lines, one record per
-- line, in time order:
--
--   header   the run's scenario, seed, sampling and map projection
--   appear   an entity index reported for the first time, or again after it
--            was gone: every field below
--   retype   an index reported with a different kind or sub-type than at the
--            last sample (EECH reused it): every field
--   alive    the alive flag changed (a death, or an index reused by a live
--            entity of the same type): id, alive, position
--   side     the side changed (a keysite captured): id, side, from
--   usable   a keysite's usable state changed: id, usable, from
--   gone     an index reported at the last sample and not at this one:
--            id and its last kind, sub-type, side and position
--   snap     every object's full state: at the first sample, every `every`
--            samples, and whenever the caller asks (campaign.lua: at each
--            metrics checkpoint)
--
-- With the supply detail on (campaign.lua observe_supply=1), also:
--
--   task     a unit's group, primary task or operational state changed (or
--            the unit is new): id, group, task, state, position
--   supply   a keysite's ammo or fuel moved by 0.5 points or more since the
--            last sample: id, ammo, fuel, from_ammo, from_fuel
--   track    every sample, the position and state of each live unit whose
--            group's primary task is the tracked task (TASK_SUPPLY)
--
-- Every sample is compared with the one before, so an event's time is the
-- first sample that shows the change. `t` is EECH's session elapsed time (a
-- 32-bit float the engine accumulates frame by frame), printed with two
-- decimals as the Tacview frames are. Positions are EECH world metres (x east,
-- y up, z north, map coordinates); heading, pitch and roll are radians, as
-- engine:objects () reports them. Raw facts only: how the Tacview recorder
-- turns them into objects is checked against them (tools/observation-check.py).
local observations = {}
observations.__index = observations

local function str (s)
	return '"' .. s:gsub ('[%c"\\]', function (ch) return string.format ("\\u%04x", ch:byte ()) end) .. '"'
end

local function full (o)
	local parts = {
		string.format ('"id":%d,"kind":%s,"sub":%d,"side":%s,"alive":%s,"type":%s', o.id, str (o.kind), o.sub_type, str (o.side),
			o.alive and "true" or "false", str (o.type_name)),
	}
	if o.name then parts[#parts + 1] = '"name":' .. str (o.name) end
	if o.group_id then parts[#parts + 1] = string.format ('"group":%d', o.group_id) end
	parts[#parts + 1] = string.format ('"x":%.2f,"y":%.2f,"z":%.2f', o.position[1], o.position[2], o.position[3])
	if o.kind == "keysite" then
		parts[#parts + 1] = string.format ('"ammo":%.3f,"fuel":%.3f,"eff":%.3f,"usable":%s', o.ammo, o.fuel, o.efficiency, str (o.usable or "unknown"))
	else
		parts[#parts + 1] = string.format ('"h":%.5f,"p":%.5f,"r":%.5f', o.heading, o.pitch, o.roll)
	end
	return table.concat (parts, ",")
end

-- path: the JSON Lines file; every: samples between full snapshots; header: a
-- table of strings and numbers describing the run; track_task: the supply
-- detail's tracked task (nil: no supply detail)
function observations.open (path, every, header, track_task)
	local f = assert (io.open (path, "wb"))
	local keys, parts = {}, {}
	for k in pairs (header) do keys[#keys + 1] = k end
	table.sort (keys)
	for _, k in ipairs (keys) do
		local v = header[k]
		parts[#parts + 1] = str (k) .. ":" .. (type (v) == "number" and string.format ("%.10g", v) or str (tostring (v)))
	end
	f:write ('{"ev":"header",', table.concat (parts, ","), "}\n")
	return setmetatable ({ f = f, every = every, samples = 0, last = {}, track_task = track_task }, observations)
end

local function opt (s)
	return s and str (s) or "null"
end

-- the supply detail's events for one object (was: its record at the last sample, or nil)
function observations:supply_detail (t, o, was)
	local f = self.f
	if o.kind == "keysite" then
		if was and was.kind == "keysite" and (math.abs (o.ammo - was.ammo) >= 0.5 or math.abs (o.fuel - was.fuel) >= 0.5) then
			f:write (string.format ('{"t":%s,"ev":"supply","id":%d,"ammo":%.3f,"fuel":%.3f,"from_ammo":%.3f,"from_fuel":%.3f}\n', t, o.id, o.ammo, o.fuel,
				was.ammo, was.fuel))
		end
	elseif o.kind ~= "weapon" then
		if not was or was.kind ~= o.kind or was.task ~= o.task or was.state ~= o.state or was.group ~= o.group_id then
			f:write (string.format ('{"t":%s,"ev":"task","id":%d,"group":%s,"task":%s,"state":%s,"x":%.2f,"y":%.2f,"z":%.2f}\n', t, o.id,
				o.group_id and string.format ("%d", o.group_id) or "null", opt (o.task), opt (o.state), o.position[1], o.position[2], o.position[3]))
		end
		if o.alive and o.task == self.track_task then
			f:write (string.format ('{"t":%s,"ev":"track","id":%d,"group":%s,"state":%s,"x":%.2f,"y":%.2f,"z":%.2f,"h":%.5f}\n', t, o.id,
				o.group_id and string.format ("%d", o.group_id) or "null", opt (o.state), o.position[1], o.position[2], o.position[3], o.heading))
		end
	end
end

-- snapshot: also write a full snapshot at this sample
function observations:sample (seconds, objects, snapshot)
	self.samples = self.samples + 1
	local f, t, now = self.f, string.format ("%.2f", seconds), {}
	for _, o in ipairs (objects) do
		local was = self.last[o.id]
		if not was then
			f:write ('{"t":', t, ',"ev":"appear",', full (o), "}\n")
		elseif was.kind ~= o.kind or was.sub_type ~= o.sub_type then
			f:write ('{"t":', t, ',"ev":"retype",', full (o), "}\n")
		else
			if was.alive ~= o.alive then
				f:write (string.format ('{"t":%s,"ev":"alive","id":%d,"alive":%s,"x":%.2f,"y":%.2f,"z":%.2f}\n', t, o.id,
					o.alive and "true" or "false", o.position[1], o.position[2], o.position[3]))
			end
			if was.side ~= o.side then
				f:write (string.format ('{"t":%s,"ev":"side","id":%d,"side":%s,"from":%s}\n', t, o.id, str (o.side), str (was.side)))
			end
			if o.kind == "keysite" and was.usable ~= o.usable then
				f:write (string.format ('{"t":%s,"ev":"usable","id":%d,"usable":%s,"from":%s}\n', t, o.id, str (o.usable or "unknown"),
					str (was.usable or "unknown")))
			end
		end
		if self.track_task then self:supply_detail (t, o, was) end
		now[o.id] = { kind = o.kind, sub_type = o.sub_type, side = o.side, alive = o.alive, usable = o.usable, position = o.position,
			ammo = o.ammo, fuel = o.fuel, task = o.task, state = o.state, group = o.group_id }
	end
	local gone = {}
	for id in pairs (self.last) do if not now[id] then gone[#gone + 1] = id end end
	table.sort (gone)
	for _, id in ipairs (gone) do
		local was = self.last[id]
		f:write (string.format ('{"t":%s,"ev":"gone","id":%d,"kind":%s,"sub":%d,"side":%s,"x":%.2f,"y":%.2f,"z":%.2f}\n', t, id, str (was.kind),
			was.sub_type, str (was.side), was.position[1], was.position[2], was.position[3]))
	end
	self.last = now
	if snapshot or self.samples == 1 or self.samples % self.every == 0 then
		for _, o in ipairs (objects) do f:write ('{"t":', t, ',"ev":"snap",', full (o), "}\n") end
	end
end

function observations:close ()
	self.f:close ()
end

return observations
