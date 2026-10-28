-- The weapon model of mode=own_model of lua/aim-provider.lua: a host with a model of its own that is not EECH's, for the weapons the campaign launches (EEDCS-45).  It
-- needs none of EECH's tables, figures or diagnostics, takes no random sample and asks the campaign for one thing: the terrain under a weapon.  Loaded by the provider and by
-- the host tests (tests/host_callbacks.rs).  Lookup kinds and action kinds are those of the campaign's protocol (aphavoc/.../mb_hostaim.h).
local own = {}

local LOOKUP_GROUND_PROBE = 8
local ACTION_STATE, ACTION_ATTITUDE, ACTION_POSITION, ACTION_BIRTH, ACTION_DECOY, ACTION_SEDUCE = 1, 2, 5, 8, 9, 10

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

-- after the flight: this model has no parachute (a falling crate is as it was), a decoy's timer restarts at half a second with no sample taken, and a seeker is never drawn
-- to a decoy
function own.animate (step) return { state = "done", parachute_status = step.parachute_status } end

function own.weapon_decoy (step)
	local timer = step.decoy_timer - step.delta_time
	local restarted = timer < 0
	if restarted then timer = 0.5 end
	return { state = "done", actions = { { kind = ACTION_DECOY, a = restarted and 1 or 0, b = 0, c = 0, d = 0, f = { timer, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 } } } }
end

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

return own
