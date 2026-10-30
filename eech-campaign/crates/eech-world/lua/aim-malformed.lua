-- The malformed providers of lua/aim-provider.lua (mode=bad_*): the reference host answers until the first request of the kind the mode corrupts, and that answer is
-- malformed; the engine must stop there with a clear error.  A file of its own so that the host tests can drive a provider directly (tests/host_callbacks.rs).
--   make (mode, reference, counted)  ->  provider (step)
-- `reference` is the reference host's `host.aim`; `counted (step, answer)` records an answer the provider lets through and returns it.
return function (mode, reference, counted)
	local kinds = { bad_nil = "ready_update", bad_state = "ready_update", bad_ready_state = "ready_update", bad_phase = "ready_update", bad_nan_vector = "vectors",
		bad_draw_for_ballistic = "ballistic", bad_second_draw = "vectors", bad_negative_range = "vectors", bad_error = "ready_command",
		bad_step_state = "weapon_step", bad_step_lookup_kind = "weapon_step", bad_step_action_kind = "weapon_step", bad_step_nonfinite = "weapon_step",
		bad_step_repeat = "weapon_step", bad_step_error = "weapon_step", bad_step_draw = "weapon_step", bad_birth_action = "weapon_birth", bad_release_burst = "release",
		bad_decoy_flag = "weapon_decoy", bad_seduce_flag = "weapon_seduce", bad_reacquire_flag = "weapon_reacquire", bad_reacquire_none = "weapon_reacquire",
		bad_forecast_negative = "forecast", bad_forecast_draw = "forecast", bad_forecast_fraction = "forecast",
		bad_damage_level = "damage", bad_damage_draw_twice = "damage", bad_damage_exit = "damage", bad_damage_class = "damage", bad_damage_unwritten = "damage", bad_damage_missing = "damage",
		bad_health_class = "health", bad_health_missing = "health", bad_health_classify_draw = "health", bad_health_second_draw = "health", bad_health_hit_ineligible = "health", bad_health_fatal = "health",
		bad_impact_crater = "impact", bad_impact_draw = "impact", bad_impact_blasts = "impact",
		bad_blast_handle = "blast", bad_blast_radius = "blast", bad_blast_draw = "blast", bad_blast_sector = "blast", bad_blast_repeat = "blast", bad_blast_modifier = "blast", bad_blast_rocket = "blast",
		bad_bridge_kill_dead = "bridge", bad_bridge_variant = "bridge", bad_bridge_notify_absent = "bridge", bad_bridge_landed_not_fell = "bridge", bad_bridge_nonfinite = "bridge",
		bad_bridge_draw = "bridge", bad_bridge_handle = "bridge", bad_bridge_terrain_twice = "bridge", bad_bridge_rest_alive = "bridge", bad_bridge_rest_nonfinite = "bridge",
		bad_bridge_identity = "bridge", bad_blast_identity = "blast", bad_blast_identity_missing = "blast",
		bad_identify_empty = "identify", bad_identify_long = "identify", bad_identify_missing = "identify", bad_identify_control = "identify", bad_identify_draw = "identify", bad_identify_repeat = "identify" }
	local kind = assert (kinds [mode], "unknown bad mode " .. mode)
	local armed = true
	local issued = nil
	return function (step)
		-- the health providers corrupt the first answer of their kind: pure-question and repeated draws are invalid; a generic timed draw for a person is valid
		-- the impact and blast providers corrupt the first answer that can carry their fault; a mode whose fault needs a walk that reaches an object or a second sector may not be
		-- reached in a short run, and the summary's counters say so
		-- the bridge providers corrupt the first answer that can carry their fault; a fault that needs a walked bridge (a dead segment's collision, a neighbour that is not there, a vehicle in reach) may not
		-- be reached in a plain war, and the summary's counters say so
		-- the identity providers corrupt the first answer that can carry their fault: the identity a host issues for a structure (empty, too long, missing, with a control character, a sample asked for,
		-- or one it issued before), and the identity it repeats for the object it names in a blast or a vehicles walk (another, or none for an object that was offered with one)
		if step.mode == "identify" and mode:find ("^bad_identify_") then
			local answer = reference (step)
			if mode == "bad_identify_repeat" then
				if armed and issued ~= nil then
					armed = false
					return { state = "done", host_id = issued }
				end
				issued = issued or answer.host_id
			elseif armed then
				armed = false
				if mode == "bad_identify_empty" then return { state = "done", host_id = "" }
				elseif mode == "bad_identify_long" then return { state = "done", host_id = string.rep ("x", 129) }
				elseif mode == "bad_identify_missing" then return { state = "done" }
				elseif mode == "bad_identify_control" then return { state = "done", host_id = "bad\nidentity" }
				elseif mode == "bad_identify_draw" then return { state = "need_draw" } end
			end
			return counted (step, answer)
		end
		if step.mode == "bridge" and mode:find ("^bad_bridge_") then
			local answer = reference (step)
			if armed then
				if mode == "bad_bridge_draw" then
					armed = false
					return { state = "need_draw" }
				elseif mode == "bad_bridge_kill_dead" and step.op == "kill" and not step.alive then
					armed = false
					return { state = "done", kill = true }
				elseif mode == "bad_bridge_variant" and step.op == "shape" then
					armed = false
					return { state = "done", variant = 9 }
				elseif mode == "bad_bridge_notify_absent" and step.op == "neighbour" and not step.neighbour_present then
					armed = false
					return { state = "done", notify = true }
				elseif mode == "bad_bridge_landed_not_fell" and step.op == "fall" and answer.state == "done" and not answer.fell then
					armed = false
					return { state = "done", fell = false, landed = true, remove = true, landed_y = 1 }
				elseif mode == "bad_bridge_nonfinite" and step.op == "fall" and answer.state == "done" and answer.fell then
					armed = false
					answer.dropped_y = 0 / 0
					return answer
				elseif mode == "bad_bridge_terrain_twice" and step.op == "fall" and step.terrain ~= nil then
					armed = false
					return { state = "need_lookup", need = "terrain" }
				elseif mode == "bad_bridge_rest_alive" and step.op == "rest" and step.alive and answer.state == "done" then
					-- a segment that is alive moved when a saved world is unpacked
					armed = false
					return { state = "done", landed = true, landed_y = 1 }
				elseif mode == "bad_bridge_rest_nonfinite" and step.op == "rest" and answer.state == "done" and answer.landed then
					armed = false
					answer.landed_y = 0 / 0
					return answer
				elseif mode == "bad_bridge_identity" and step.op == "vehicles" and answer.state == "done" and answer.action == "act" then
					armed = false
					answer.host_id = "x-" .. (answer.host_id or "none")
					return answer
				elseif mode == "bad_bridge_handle" and step.op == "vehicles" and answer.state == "done" and answer.action == "act" then
					armed = false
					answer.handle = 99999
					return answer
				end
			end
			return counted (step, answer)
		end
		if (step.mode == "impact" and mode:find ("^bad_impact_")) or (step.mode == "blast" and mode:find ("^bad_blast_")) then
			local answer = reference (step)
			if armed then
				if mode == "bad_impact_crater" and step.terrain_draws_crater == nil then
					-- a crater on a terrain the campaign has not read for it
					armed = false
					return { state = "done", invalid = false, crater = true, direct_hit = false, blasts = 1, exclude_target = false }
				elseif mode == "bad_impact_draw" then
					armed = false
					return { state = "need_draw" }
				elseif mode == "bad_impact_blasts" and answer.state == "done" and not answer.invalid then
					armed = false
					answer.blasts = 3
					return answer
				elseif mode == "bad_blast_draw" then
					armed = false
					return { state = "need_draw" }
				elseif mode == "bad_blast_sector" and answer.state == "need_lookup" then
					armed = false
					answer.need = { sx = step.max_x_sector + 1, sz = step.max_z_sector + 1, after = 0 }
					return answer
				elseif mode == "bad_blast_repeat" and answer.state == "need_lookup" and answer.need.after == 0 and step.window ~= nil then
					-- asks for the start of the sector it has just been given
					armed = false
					answer.need = { sx = step.window.sx, sz = step.window.sz, after = 0 }
					return answer
				elseif mode == "bad_blast_radius" and step.window ~= nil and answer.radius > 0 then
					-- the radius changes at the first window the host is given; the optional echo of the candidates examined (the fixture build's) says the NEW radius too, so the diagnostic envelope is
					-- consistent and the campaign's refusal of a changed radius is the primary one, not a refusal of a malformed echo.  A supplied window and a positive radius are all it needs
					armed = false
					answer.radius = answer.radius + 1
					for _, event in ipairs (answer.echo or {}) do event.f [2] = answer.radius end
					return answer
				elseif mode == "bad_blast_handle" and answer.state == "done" and answer.action ~= "none" then
					armed = false
					answer.handle = 99999
					return answer
				elseif mode == "bad_blast_identity" and answer.state == "done" and answer.action ~= "none" then
					armed = false
					answer.host_id = "x-" .. (answer.host_id or "none")
					return answer
				elseif mode == "bad_blast_identity_missing" and answer.state == "done" and answer.action ~= "none" and answer.host_id ~= nil then
					armed = false
					answer.host_id = nil
					return answer
				elseif mode == "bad_blast_modifier" and answer.state == "done" and answer.action ~= "none" then
					armed = false
					answer.modifier = 2
					return answer
				elseif mode == "bad_blast_rocket" and answer.state == "done" and answer.action == "collision" then
					armed = false
					answer.action = "intercept"
					return answer
				end
			end
			return counted (step, answer)
		end
		-- the reference host answers requests that the selected mode does not corrupt
		if step.mode == "health" and mode:find ("^bad_health_") then
			if mode == "bad_health_classify_draw" then
				-- a classification is a pure question: it may not ask for the campaign's sample
				if armed and step.op == "classify" then
					armed = false
					return { state = "need_draw" }
				end
				return counted (step, reference (step))
			end
			if mode == "bad_health_second_draw" then
				-- a timed roll takes one sample: the host that has been given it and asks again is refused
				if armed and step.op == "timed" and step.draw ~= nil then
					armed = false
					return { state = "need_draw" }
				end
				return counted (step, reference (step))
			end
			local answer = reference (step)
			local corrupted = false
			if armed and answer.state == "done" then
				if mode == "bad_health_class" and step.op == "classify" then corrupted = true answer.critical_class = 7
				elseif mode == "bad_health_missing" and step.op == "classify" then corrupted = true answer.critical_class = nil
				elseif mode == "bad_health_hit_ineligible" and step.op == "timed" and not answer.eligible then corrupted = true answer.hit = true answer.new_level = math.max (0, step.damage_level - 1)
				elseif mode == "bad_health_fatal" and step.op == "timed" and answer.hit then corrupted = true answer.fatal = not answer.fatal end
			end
			if corrupted then
				armed = false
				return answer
			end
			return counted (step, answer)
		end
		-- the overdraw provider is not satisfied by a sample: it asks for another at every pass of every weapon step, without consulting the reference host, until the
		-- campaign's room for samples is gone and the engine refuses (a legal draw is a continuation, not a refusal)
		if mode == "bad_step_draw" and step.mode == kind then
			return { state = "need_draw" }
		end
		local answer = reference (step)
		if armed and step.mode == kind and (mode ~= "bad_second_draw" or step.draw ~= nil) and (mode ~= "bad_negative_range" or answer.state == "done")
			and (mode ~= "bad_step_lookup_kind" or answer.state == "need_lookup") and (mode ~= "bad_step_action_kind" or answer.state == "done")
			and (mode ~= "bad_step_nonfinite" or (answer.state == "done" and #answer.actions > 0)) and (mode ~= "bad_step_repeat" or #step.answers > 0)
			and (mode ~= "bad_birth_action" or answer.state == "done")
			and (not mode:find ("^bad_damage_") or (mode == "bad_damage_draw_twice" and step.draw ~= nil) or (mode ~= "bad_damage_draw_twice" and answer.state == "done" and answer.written)) and (not mode:find ("^bad_%a+_flag$") and mode ~= "bad_reacquire_none" or answer.state == "done") then
			armed = false
			if mode == "bad_nil" then return nil
			elseif mode == "bad_state" then return { state = "later" }
			elseif mode == "bad_ready_state" then return { state = "done", ready_state = 5.0, phase = 0 }
			elseif mode == "bad_phase" then return { state = "done", ready_state = 1.0, phase = 7 }
			elseif mode == "bad_nan_vector" then answer.weapon_vector = { 0 / 0, 0, 0 } return answer
			elseif mode == "bad_draw_for_ballistic" then return { state = "need_draw" }
			elseif mode == "bad_second_draw" then return { state = "need_draw" }
			elseif mode == "bad_negative_range" then answer.intercept_range = -1 return answer
			elseif mode == "bad_error" or mode == "bad_step_error" then error ("provider failed on purpose")
			elseif mode == "bad_birth_action" then answer.actions [1].kind = 1 return answer
			elseif mode == "bad_decoy_flag" then answer.actions [1].a = 2 return answer
			elseif mode == "bad_seduce_flag" then answer.actions [1].b = 3 return answer
			elseif mode == "bad_reacquire_flag" then answer.actions [1].a = -1 return answer
			elseif mode == "bad_reacquire_none" then answer.actions = {} return answer
			elseif mode == "bad_release_burst" then return { state = "done", refuse = false, burst_size = 0 }
			elseif mode == "bad_damage_level" then answer.new_level = -1 return answer
			elseif mode == "bad_damage_draw_twice" then return { state = "need_draw" }
			elseif mode == "bad_damage_exit" then answer.exited = true return answer
			elseif mode == "bad_damage_class" then answer.critical_after = 5 return answer
			elseif mode == "bad_damage_unwritten" then answer.written = false return answer
			elseif mode == "bad_damage_missing" then answer.capability = nil return answer
			elseif mode == "bad_forecast_negative" then return { state = "done", capability = -1 }
			elseif mode == "bad_forecast_draw" then return { state = "need_draw" }
			elseif mode == "bad_forecast_fraction" then return { state = "done", capability = 1.5 }
			elseif mode == "bad_step_state" then return { state = "later" }
			elseif mode == "bad_step_lookup_kind" then answer.lookups [1].kind = 99 return answer
			elseif mode == "bad_step_action_kind" then if #answer.actions == 0 then answer.actions = { { kind = 99, a = 0, b = 0, c = 0, d = 0, f = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 } } } else answer.actions [1].kind = 99 end return answer
			elseif mode == "bad_step_nonfinite" then answer.actions [1].f [1] = 0 / 0 return answer
			elseif mode == "bad_step_repeat" then
				-- asks again for a lookup the campaign has just answered
				local given = step.answers [1]
				return { state = "need_lookup", lookups = { { kind = given.kind, a = given.a, b = given.b, c = given.c, d = given.d, f = { given.f [1], given.f [2], given.f [3], given.f [4], given.f [5], given.f [6], 0, 0, 0, 0, 0, 0 } } } }
			end
		end
		return counted (step, answer)
	end
end
