-- The observer-report flattening and delta code of lua/combat-case.lua (EEDCS-46), in a file of its own so that the host tests load the SAME code with synthetic reports
-- (tests/host_callbacks.rs) instead of a copy of it.
--   counters (text)                  -> { ["event[.type=..][.code=..][.kind=..].key"] = number }
--   deltas (before_text, after_text) -> { key = after - before } for every counter that changed (maxima are not counters)
local module = {}

-- The observer's counters as a flat table of numbers.  A report has singleton rows (`cmb-blast calls=...`) and INDEXED rows, one per entity kind (`cmb-kind type=`), warhead
-- (`cmb-warhead type=`), kill code (`cmb-blast-kill code=`), timed-roll kind (`cmb-timed kind=`) and unobserved kind (`cmb-unobserved type=`).  Every identifier a row carries is part
-- of its counters' keys, in the order type, code, kind, so no later row can overwrite an earlier one: `cmb-blast-kill.code=3.calls`, `cmb-warhead.type=4.real`,
-- `cmb-kind.type=ENTITY_TYPE_HELICOPTER.collisions`, `cmb-timed.kind=4.draws`.  Each key holds that row's own value (never a sum over rows); hashes are not counters.
local IDENTIFIERS = { "type", "code", "kind" }

function module.counters (text)
	local c = {}
	for line in string.gmatch (text, "[^\n]+") do
		local event = string.match (line, "^(%S+)")
		if event ~= "cmb-hash" then
			local fields, identifiers = {}, {}
			for k, v in string.gmatch (line, "(%S+)=(%S+)") do fields [k] = v end
			local prefix = event
			for _, id in ipairs (IDENTIFIERS) do
				if fields [id] then
					prefix = prefix .. "." .. id .. "=" .. fields [id]
					identifiers [id] = true
				end
			end
			for k, v in pairs (fields) do
				local n = tonumber (v)
				if n and not identifiers [k] then c [prefix .. "." .. k] = n end
			end
		end
	end
	return c
end

-- every counter that changed between two reports, as a flat table of after - before
function module.deltas (before_text, after_text)
	local b, a, d = module.counters (before_text), module.counters (after_text), {}
	for k, v in pairs (a) do
		local diff = v - (b [k] or 0)
		if diff ~= 0 and not string.find (k, "max_", 1, true) then d [k] = diff end
	end
	return d
end

return module
