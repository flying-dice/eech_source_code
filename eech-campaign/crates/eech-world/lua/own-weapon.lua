-- The weapon model of mode=own_model of lua/aim-provider.lua: a host with a model of its own that is not EECH's, for the weapons the campaign launches (EEDCS-45).  It
-- needs none of EECH's tables, figures or diagnostics, takes no random sample and asks the campaign for one thing: the terrain under a weapon.  Loaded by the provider and by
-- the host tests (tests/host_callbacks.rs).  Lookup kinds and action kinds are those of the campaign's protocol (aphavoc/.../mb_hostaim.h).
local own = {}

local LOOKUP_GROUND_PROBE = 8
local ACTION_STATE, ACTION_ATTITUDE, ACTION_POSITION, ACTION_BIRTH, ACTION_DECOY, ACTION_SEDUCE, ACTION_REACQUIRE = 1, 2, 5, 8, 9, 10, 11

-- The module carries the arguments of a lookup as single-precision numbers, and the campaign answers by repeating them; a model that asked about a point it computed in
-- double precision would never recognise its own question.  Every coordinate this model asks about is therefore snapped to a grid of 1/64 m, every point of which a
-- single-precision number represents exactly (to 262144 m), so the question that goes out and the answer that comes back are the same by value.
local function snap (x) return math.floor (x * 64 + 0.5) / 64 end

function own.figures () return { state = "done" } end

-- a launch of its own: the weapon leaves at 300 m/s along the launcher's heading, for twenty seconds, with no dispersion, no fuze and nothing asked of the campaign; the
-- launcher lets one round go at every request (no rate of fire)
function own.weapon_birth (step)
	local a = step.attitude
	return { state = "done", actions = {
		{ kind = ACTION_BIRTH, a = 0, b = 0, c = 0, d = 0, f = { 300, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 } },
		{ kind = ACTION_ATTITUDE, a = 0, b = 0, c = 0, d = 0, f = { a [1], a [2], a [3], a [4], a [5], a [6], a [7], a [8], a [9], 0, 0, 0 } },
	} }
end

function own.release () return { state = "done", refuse = false, burst_size = 1 } end

-- the damage forecast (EEDCS-46): a weapon with a damage figure can do 5 points of damage to anything, whatever its armour, its warhead or the player's setting; one with
-- none does nothing.  It needs none of EECH's arithmetic, takes no random sample and asks the campaign for nothing.
-- the damage of a hit (EEDCS-46): the player's invulnerability stops it; otherwise a weapon with a damage figure does 10 points to anything, whatever its armour, warhead,
-- speed or the settings (and takes no sample of the campaign's stream), and the health is written unless the cheat guard stops it.  A vehicle is critical at a quarter of its
-- initial health or less; an aircraft is mortal at half and critical at a tenth; a person and a fixed structure are never classified.
local function critical_class (step, level)
	local initial = step.initial_damage_level
	if step.critical_model == 1 then return level * 4 <= initial and 1 or 0 end
	if step.critical_model == 3 then
		if level * 10 <= initial then return 2 end
		return level * 2 <= initial and 1 or 0
	end
	return 0
end

function own.damage (step)
	local class = critical_class (step, step.damage_level)
	if step.target_not_ai and step.invulnerable then
		return { state = "done", exited = true, capability = 0, written = false, new_level = step.damage_level, critical_before = class, critical_after = class, bad_modifier = false }
	end
	local capability = step.damage_capability > 0 and 10 or 0
	local written = capability > 0 and not step.cheat_guard
	local level = written and math.max (step.damage_level - capability, 0) or step.damage_level
	return { state = "done", exited = false, capability = capability, written = written, new_level = level, critical_before = class, critical_after = critical_class (step, level), bad_modifier = false }
end

-- the health questions that are not a hit (EEDCS-46): the classes are this model's own (above), nothing ever burns or degrades by itself (no timed roll is eligible, so it takes no
-- sample), a change of health crosses a threshold when it raises the class, and a unit may hurt itself
local function health_reply (step, fields)
	local reply = { state = "done", critical_class = 0, crossed = false, recovered = false, eject = false, eligible = false, rolled = false, hit = false, new_level = step.damage_level, fatal = false }
	for k, v in pairs (fields or {}) do reply [k] = v end
	return reply
end

function own.health (step)
	if step.op == "classify" then return health_reply (step, { critical_class = critical_class (step, step.damage_level) }) end
	if step.op == "assess" then
		local before, after = critical_class (step, step.old_damage_level), critical_class (step, step.damage_level)
		if step.critical_model == 3 then return health_reply (step, { crossed = after > before, recovered = after < before }) end
		return health_reply (step, { crossed = (step.explosive or step.flammable) and after > before })
	end
	if step.op == "timed" then return health_reply (step, { critical_class = critical_class (step, step.damage_level) }) end
	return health_reply (step, { eligible = true })
end

function own.forecast (step) return { state = "done", capability = step.damage_capability > 0 and 5 or 0 } end

-- after the flight: this model has no parachute (a falling crate is as it was), a decoy's timer restarts at half a second with no sample taken, a seeker is never drawn
-- to a decoy and, if it ever followed one, drops the lock when the decoy expires (it does not go back to the launcher; the campaign unlinks it)
function own.animate (step) return { state = "done", parachute_status = step.parachute_status } end

function own.weapon_decoy (step)
	local timer = step.decoy_timer - step.delta_time
	local restarted = timer < 0
	if restarted then timer = 0.5 end
	return { state = "done", actions = { { kind = ACTION_DECOY, a = restarted and 1 or 0, b = 0, c = 0, d = 0, f = { timer, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 } } } }
end

function own.weapon_reacquire () return { state = "done", actions = { { kind = ACTION_REACQUIRE, a = 0, b = 0, c = 0, d = 0, f = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 } } } } end

function own.weapon_seduce () return { state = "done", actions = { { kind = ACTION_SEDUCE, a = 0, b = 0, c = 0, d = 0, f = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 } } } } end

-- a weapon with a model of its own: it flies straight along its heading at the speed it has, for twenty seconds, and ends on the ground it meets; it needs one lookup (the
-- terrain under its new position), no figure of EECH's, no draw and no wind
function own.weapon_step (step)
	local dt = step.delta_time
	local p, a = step.position, step.attitude
	local new = { snap (p [1] + a [7] * step.velocity * dt), snap (p [2] + a [8] * step.velocity * dt), snap (p [3] + a [9] * step.velocity * dt) }
	local kill = step.kill_code
	local inside = new [1] >= step.map_min [1] and new [1] <= step.map_max [1] and new [3] >= step.map_min [3] and new [3] <= step.map_max [3]
	if inside then
		local below
		for _, answer in ipairs (step.answers) do
			if answer.kind == LOOKUP_GROUND_PROBE and answer.f [1] == new [1] and answer.f [2] == new [2] and answer.f [3] == new [3] then below = answer.f [7] ~= 0 end
		end
		if below == nil then
			return { state = "need_lookup", lookups = { { kind = LOOKUP_GROUND_PROBE, a = 0, b = 0, c = 0, d = 0, f = { new [1], new [2], new [3], 0, 0, 0, 0, 0, 0, 0, 0, 0 } } } }
		end
		if below and kill == 0 then kill = 3 end
	elseif kill == 0 then
		kill = 2
	end
	local lifetime = step.weapon_lifetime - dt
	if lifetime <= -20 and kill == 0 then kill = 1 end
	return { state = "done", actions = {
		{ kind = ACTION_STATE, a = step.missile_phase, b = step.weapon_on_ground and 1 or 0, c = kill, d = 0, f = { step.velocity, a [7] * step.velocity, a [8] * step.velocity, a [9] * step.velocity, lifetime, step.decoy_timer, 0, 0, 0, 0, 0, 0 } },
		{ kind = ACTION_ATTITUDE, a = 0, b = 0, c = 0, d = 0, f = { a [1], a [2], a [3], a [4], a [5], a [6], a [7], a [8], a [9], 0, 0, 0 } },
		{ kind = ACTION_POSITION, a = 0, b = 0, c = 0, d = 0, f = { inside and new [1] or p [1], inside and new [2] or p [2], inside and new [3] or p [3], 0, 0, 0, 0, 0, 0, 0, 0, 0 } },
	} }
end

-- what a weapon's death does to the world (EEDCS-46): a plain model that is not EECH's.  A weapon that ran out, left the map or hit water does nothing; any other death (a land hit,
-- an overshot or self-destroyed weapon, a hit on its target) blasts ONCE, without a crater, so it never asks for the terrain; a hit on the target sends it a direct hit when it is alive
-- and leaves it out of the blast.  A kill code it does not know is not a death it knows.  The codes are the campaign's (weapon.h): 1 exhausted, 2 out of bounds, 4 water.
function own.impact (step)
	local code = step.kill_code
	if code == 1 or code == 2 or code == 4 then return { state = "done", invalid = false, crater = false, direct_hit = false, blasts = 0, exclude_target = false } end
	if code == 3 or code == 6 or code == 7 then return { state = "done", invalid = false, crater = false, direct_hit = false, blasts = 1, exclude_target = false } end
	if code == 5 then return { state = "done", invalid = false, crater = false, direct_hit = step.has_target and step.target_alive, blasts = 1, exclude_target = step.has_target } end
	return { state = "done", invalid = true, crater = false, direct_hit = false, blasts = 0, exclude_target = false }
end

-- one area blast of a plain model: a fixed radius of thirty metres whatever the weapon, reaching only the sector the explosion is in (whole sectors of the campaign's map), in the
-- order of the list, every object that is alive and placed, whatever its kind, strictly inside the radius and not the excluded target; it never destroys a rocket.  It needs none of
-- EECH's tables: the window it asks for is the campaign's list of the sector, which is all it knows of the world.
local BLAST_RADIUS = 30
function own.blast (step)
	local sx = math.min (math.max (math.floor (step.position [1] / step.sector_side_length), step.min_x_sector), step.max_x_sector)
	local sz = math.min (math.max (math.floor (step.position [3] / step.sector_side_length), step.min_z_sector), step.max_z_sector)
	local window = step.window
	if window == nil then return { state = "need_lookup", radius = BLAST_RADIUS, need = { sx = sx, sz = sz, after = 0 } } end
	local p = step.position
	for index, object in ipairs (window.objects) do
		if object.readable and object.alive and object.has_position and not object.excluded then
			local q = object.position
			local range = math.sqrt ((p [1] - q [1]) ^ 2 + (p [2] - q [2]) ^ 2 + (p [3] - q [3]) ^ 2)
			if range > 0 and range < BLAST_RADIUS then
				-- this plain model quantizes its modifier to 1/1024, its own choice: the module reads the modifier as a single-precision number, so any finite double is accepted and rounded to the nearest one
				return { state = "done", radius = BLAST_RADIUS, action = "collision", handle = object.handle, modifier = math.floor (range / BLAST_RADIUS * 1024) / 1024 }
			end
		end
	end
	if not window.list_ends then
		local last = window.objects [#window.objects]
		return { state = "need_lookup", radius = BLAST_RADIUS, need = { sx = window.sx, sz = window.sz, after = last.handle } }
	end
	return { state = "done", radius = BLAST_RADIUS, action = "none" }
end

return own