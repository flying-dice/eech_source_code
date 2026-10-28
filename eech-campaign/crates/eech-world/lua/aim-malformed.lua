-- The malformed providers of lua/aim-provider.lua (mode=bad_*): the reference host answers until the first request of the kind the mode corrupts, and that answer is
-- malformed; the engine must stop there with a clear error.  A file of its own so that the host tests can drive a provider directly (tests/host_callbacks.rs).
--   make (mode, reference, counted)  ->  provider (step)
-- `reference` is the reference host's `host.aim`; `counted (step, answer)` records an answer the provider lets through and returns it.
return function (mode, reference, counted)
	local kinds = { bad_nil = "ready_update", bad_state = "ready_update", bad_ready_state = "ready_update", bad_phase = "ready_update", bad_nan_vector = "vectors",
		bad_draw_for_ballistic = "ballistic", bad_second_draw = "vectors", bad_negative_range = "vectors", bad_error = "ready_command",
		bad_step_state = "weapon_step", bad_step_lookup_kind = "weapon_step", bad_step_action_kind = "weapon_step", bad_step_nonfinite = "weapon_step",
		bad_step_repeat = "weapon_step", bad_step_error = "weapon_step", bad_step_draw = "weapon_step", bad_birth_action = "weapon_birth", bad_release_burst = "release",
		bad_decoy_flag = "weapon_decoy", bad_seduce_flag = "weapon_seduce", bad_reacquire_flag = "weapon_reacquire", bad_reacquire_none = "weapon_reacquire" }
	local kind = assert (kinds [mode], "unknown bad mode " .. mode)
	local armed = true
	return function (step)
		-- the overdraw provider is not satisfied by a sample: it asks for another at every pass of every weapon step, without consulting the reference host, until the
		-- campaign's room for samples is gone and the engine refuses (a legal draw is a continuation, not a refusal)
		if mode == "bad_step_draw" and step.mode == kind then
			return { state = "need_draw" }
		end
		local answer = reference (step)
		if armed and step.mode == kind and (mode ~= "bad_second_draw" or step.draw ~= nil) and (mode ~= "bad_negative_range" or answer.state == "done")
			and (mode ~= "bad_step_lookup_kind" or answer.state == "need_lookup") and (mode ~= "bad_step_action_kind" or answer.state == "done")
			and (mode ~= "bad_step_nonfinite" or (answer.state == "done" and #answer.actions > 0)) and (mode ~= "bad_step_repeat" or #step.answers > 0)
			and (mode ~= "bad_birth_action" or answer.state == "done") and (not mode:find ("^bad_%a+_flag$") and mode ~= "bad_reacquire_none" or answer.state == "done") then
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
