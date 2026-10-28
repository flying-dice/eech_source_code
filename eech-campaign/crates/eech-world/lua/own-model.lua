-- The plain model of mode=own_model of lua/aim-provider.lua: a host with a model of its own that is not EECH's (instant readiness, snapping devices, a bearing in the device
-- frame, guided weapons intercept the target's point; the weapons the campaign launches fly, launch and are released by lua/own-weapon.lua).  It needs none of EECH's tables,
-- figures or diagnostics.  Loaded by the provider, by the controlled launch witness (lua/weapon-case.lua with aim_model=own_model) and by the host tests.
--   own [request mode] (step) -> the answer
local function normalise (v)
	local length = math.sqrt (v [1] * v [1] + v [2] * v [2] + v [3] * v [3])
	if length == 0 then return { 0, 0, -1 }, 0 end
	return { v [1] / length, v [2] / length, v [3] / length }, length
end

-- the plain model of mode=own_model: instant readiness, snapping devices, a bearing in the device frame, guided weapons intercept the target's point
local own = {}
function own.ready_update (step)
	return { state = "done", ready_state = (step.state >= 1.5 and step.state < 2.5) and 2.0 or (step.state < 0.5 and 0.0 or step.state), phase = (step.state >= 1.5 and step.state < 2.5) and 2 or 0 }
end
function own.ready_command (step) return { state = "done", ready_state = step.open and 2.0 or 0.0 } end
function own.required (step)
	-- the intercept vector in the device frame by the transpose of its attitude (rows are the device axes), then a bearing; a gun aimed at a tracking point
	-- takes the plain direction to it
	local m, v = step.device_attitude, step.intercept_vector
	if step.tracking then
		local d, p = step.tracking.device_position, step.tracking.point
		v = normalise { p [1] - d [1], p [2] - d [2], p [3] - d [3] }
	end
	local x = m [1] * v [1] + m [2] * v [2] + m [3] * v [3]
	local y = m [4] * v [1] + m [5] * v [2] + m [6] * v [3]
	local z = m [7] * v [1] + m [8] * v [2] + m [9] * v [3]
	local heading = math.atan2 (x, z)
	local pitch = math.atan2 (y, math.sqrt (x * x + z * z))
	if not step.unbounded then
		heading = math.max (step.heading_min, math.min (step.heading_max, heading))
		pitch = math.max (step.pitch_min, math.min (step.pitch_max, pitch))
	end
	return { state = "done", required_heading = heading, required_pitch = pitch }
end
function own.step (step)
	local heading = math.max (step.heading_min, math.min (step.heading_max, step.required_heading))
	return { state = "done", delta_heading = heading - step.current_heading, delta_pitch = step.required_pitch - step.current_pitch, new_heading = heading, new_pitch = step.required_pitch }
end
function own.vectors (step)
	local nothing = { 0, 0, -1 }
	if step.tracking_point or step.weapon.aiming_type ~= "intercept_point" then
		-- the model has no solution for an unguided weapon or a tracking point
		return { state = "done", valid = false, weapon_vector = nothing, target_vector = nothing, intercept_vector = nothing, intercept_range = 0 }
	end
	local d, a, p = step.device_position, step.device_attitude, step.target.point
	local to_point, range = normalise { p [1] - d [1], p [2] - d [2], p [3] - d [3] }
	local to_target = normalise { step.target.position [1] - d [1], step.target.position [2] - d [2], step.target.position [3] - d [3] }
	return { state = "done", valid = true, weapon_vector = { a [7], a [8], a [9] }, target_vector = to_target, intercept_vector = to_point, intercept_range = range }
end
function own.lead (step) return { state = "done", ok = false, intercept_point = step.target.point, angle = 0 } end
function own.ballistic () return { state = "done", ok = false } end
function own.intercept (step) return { state = "done", intercept_point = step.target.point } end
function own.table () return { state = "done" } end
function own.range () return { state = "done" } end

-- the weapon model of this mode (the figures, the launch, the release, the flight and what follows it) is a file of its own, next to this one
local here = (debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or "")
for name, weapon_model in pairs (dofile (here .. "own-weapon.lua")) do own [name] = weapon_model end

return own
