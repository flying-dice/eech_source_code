-- The carrier choice of lua/combat-case.lua (EEDCS-46), in a file of its own so that the host tests load the SAME code with a synthetic inventory (tests/host_callbacks.rs).
--
-- WHY: the first controlled original run took the FIRST launcher carrying each class.  The high-explosive carrier it took launched six weapons and was then refused again and again
-- (13 rejections, ammunition exhaustion inferred): a case run spends its carrier's ammunition, so a carrier that cannot afford the plan is not a plan.
--
-- WHAT IS AN ESTIMATE AND WHAT IS LIVE.  The campaign's launch call fires a SALVO (an AI launcher fires its weapon's whole salvo size; wn_creat.c launch_client_server_weapon), and every
-- weapon created consumes its burst size of rounds (weapon.c), so one launch call can create several weapons and spend several rounds.  The plan therefore does NOT promise that a
-- carrier's round count covers N launch calls:
--   need (living_kinds)                  -> an ESTIMATE of the launch calls the plan makes (a direct hit and a blast-range collision per living kind, a land-hit explosion per living
--                                           kind and one hit on target: 3 * kinds + 1).  A carrier needs at least one ROUND per call, so a count below it can NOT cover the plan (a
--                                           necessary condition, never a sufficient one).
--   choose (launchers, wanted, needed)   -> launcher id, weapon, warhead, count, estimated shortfall for the best carrier of ONE warhead class (nothing when none carries it)
--   choose_class (candidates)            -> the class to use among the classes of one ROLE, in the caller's preference order: the first that is not estimated short, else the one
--                                           with the most ammunition; and why
--   pick (role, live_count, debit)       -> the carrier a case launches from NOW: the first of the role's carriers whose LIVE count (a read-only query made for this case) covers the
--                                           rounds its last launch cost (at least one).  The actual count change governs, not the estimate.
--   note_debit (debit, carrier, before, after) -> remembers the largest number of rounds a launch of that carrier cost
-- A carrier is a table with `key` (launcher and weapon) and whatever the caller keeps; classes are never changed by the plan except through the caller's explicit role list.
local module = {}

function module.need (living_kinds)
	if living_kinds <= 0 then return 0 end
	return 3 * living_kinds + 1
end

-- `launchers` is a list of { id = , inventory = { { weapon = , count = , warhead = }, ... } }; `wanted (warhead)` says whether a warhead belongs to the class
function module.choose (launchers, wanted, needed)
	local best
	for _, l in ipairs (launchers) do
		for _, item in ipairs (l.inventory) do
			if wanted (item.warhead) then
				if item.count >= needed then return l.id, item.weapon, item.warhead, item.count, 0 end
				if not best or item.count > best.count then best = { id = l.id, weapon = item.weapon, warhead = item.warhead, count = item.count } end
			end
		end
	end
	if best then return best.id, best.weapon, best.warhead, best.count, needed - best.count end
end

-- `candidates` is the role's usable classes in the caller's preference order, each { class =, count =, shortfall = , ... } (shortfall is the estimate of `choose`)
function module.choose_class (candidates)
	local best
	for index, c in ipairs (candidates) do
		if c.shortfall == 0 then return c, index == 1 and "preferred" or "first_affordable_alternate" end
		if not best or c.count > best.count then best = c end
	end
	if best then return best, "none_affordable_most_ammunition" end
end

-- `role` is the ordered carriers of one role (the chosen one first); `live_count (carrier)` reads its weapon count now (-1 when unreadable); `debit` maps key -> rounds one launch cost
function module.pick (role, live_count, debit)
	for _, c in ipairs (role) do
		local n = live_count (c)
		if n >= math.max (1, debit [c.key] or 1) then return c, n end
	end
end

function module.note_debit (debit, carrier, before, after)
	if before >= 0 and after >= 0 and before > after then debit [carrier.key] = math.max (debit [carrier.key] or 1, before - after) end
end

return module
