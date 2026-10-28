-- The carrier choice of lua/combat-case.lua (EEDCS-46), in a file of its own so that the host tests load the SAME code with a synthetic inventory (tests/host_callbacks.rs).
--   need (living_kinds)                -> the launches one carrier of a class has to afford: a direct hit and a blast-range collision per living kind, a land-hit explosion per
--                                          living kind and one hit on target (3 * kinds + 1, the upper bound over the two roles a class can play)
--   choose (launchers, wanted, needed) -> launcher id, weapon, warhead, count, shortfall   (nothing when no launcher carries the class)
-- WHY: the first controlled original run took the FIRST launcher carrying each class.  The high-explosive carrier it took launched six weapons and was then refused again and again
-- (13 rejections): a case run spends the carrier's ammunition, so a carrier that cannot afford the plan is not a plan.  A carrier whose count covers the need is taken (the first, in
-- launcher order, so the choice is stable); when none does, the one with the MOST ammunition is taken and the shortfall is reported rather than hidden.
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

return module
