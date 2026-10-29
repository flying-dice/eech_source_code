-- The verdict of lua/ground-handoff.lua, kept apart so that it can be exercised alone (EEDCS-48, D3 G2 HOST PREPARATION). It judges counts of the requests the host terrain provider
-- received, by operation, during a healthy run of the established ground characterisation driver; it measures nothing itself.
--
--   control    the UNMOVED campaign DLL: it computes the ground predicates and the impact search itself and asks the host only point/map/fast questions. Expected: ground_below and
--              ground_impact requests == 0 AND point requests > 0 (real original traffic: a run that asked nothing proves nothing about the boundary).
--   required   the boundary the extraction must create: ground_below or ground_impact requests > 0. On the unmoved DLL this FAILS with `red_marker`: a missing boundary, not a setup failure.
--
-- Both modes fail with a message that does NOT contain the marker when the run was not healthy enough to judge (no point traffic).
return {
	red_marker = "GROUND-HANDOFF-RED",
	verdict = function (mode, counts)
		local marker = "GROUND-HANDOFF-RED"
		local ground = (counts.ground_below or 0) + (counts.ground_impact or 0)
		local point = counts.point or 0
		if mode ~= "control" and mode ~= "required" then
			error ("ground-handoff: handoff must be control or required, got '" .. tostring (mode) .. "'", 0)
		end
		if point == 0 then
			return false, "ground-handoff: " .. mode .. " SETUP FAILURE: the healthy driver asked the host terrain provider no point question, so no boundary was observed"
		end
		if mode == "control" then
			if ground ~= 0 then
				return false, "ground-handoff: control UNEXPECTED: " .. ground .. " ground request(s) reached the host provider, so this is not the unmoved campaign DLL"
			end
			return true, "ground-handoff: control: zero ground requests with " .. point .. " point request(s): the unmoved DLL computes the ground predicates and the impact search itself"
		end
		if ground == 0 then
			return false, marker .. ": no ground_below or ground_impact request reached the host terrain provider (" .. point
				.. " point request(s)): the DLL still computes the ground predicates and the impact search itself"
		end
		return true, "ground-handoff: required: " .. (counts.ground_below or 0) .. " ground_below and " .. (counts.ground_impact or 0) .. " ground_impact request(s) reached the host provider"
	end,
}
