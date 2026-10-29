-- The JSON Lines writer and the report parser of lua/combat-characterization.lua (EEDCS-46), in a file of their own so that the host tests load the SAME code with a synthetic
-- report (tests/host_callbacks.rs), instead of a copy of it.
--   make (out)  ->  { emit = function (t), emit_report = function (scope, at, report) }
-- `out` is anything with write and flush (a file, or a table in a test).
return function (out)
	-- a string as a JSON string with every control character, quote, backslash and non-ASCII byte escaped losslessly (a byte >= 0x80 is written as the code point of the same value,
	-- i.e. the Latin-1 reading): nothing in the recorded text is replaced or dropped
	local escapes = { ['"'] = '\\"', ['\\'] = '\\\\', ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t' }
	local function json_string (s)
		return '"' .. (string.gsub (s, '[%c"\\\128-\255]', function (c) return escapes [c] or string.format ("\\u%04x", string.byte (c)) end)) .. '"'
	end

	local function number_or_string (v)
		-- a non-finite number is not valid JSON: keep it as the string it printed as, so the comparator sees it rather than a parse error
		if v ~= v or v == math.huge or v == -math.huge then
			return json_string (tostring (v))
		end
		return tostring (v)
	end

	local function emit (t)
		local parts = {}
		for k, v in pairs (t) do
			local value = type (v) == "string" and json_string (v) or number_or_string (v)
			parts [#parts + 1] = json_string (k) .. ":" .. value
		end
		table.sort (parts)
		out:write ("{" .. table.concat (parts, ",") .. "}\n")
		out:flush ()
	end

	-- the six hashes of the `cmb-hash` line are 16 hex digits and are strings, even when every digit is decimal (a number would lose the leading zeros and, above 2^53, the low
	-- digits).  They are hashes ONLY in that event: the word `blast` is also a COUNTER in a `cmb-kind` row, where a number is right.  So what is protected is a field of an event,
	-- never a word.  (The original f41 run wrote the cmb-kind counter as a string because this table used to be a set of words; that run's raw trace is kept as written.)
	local string_fields = {
		["cmb-hash"] = { damage = true, collision = true, blast = true, repair = true, timed = true, order = true },
	}

	-- one JSON object per report line: the first word is the event, `key=value` words are its fields; a sample line keeps its text whole
	local function emit_report (scope, at, report)
		for line in string.gmatch (report, "[^\n]+") do
			local event, rest = string.match (line, "^(%S+)%s*(.*)$")
			local t = { event = event, scope = scope, at = at }
			if event == "cmb-sample" then
				local key, text = string.match (rest, "^key=(%S+)%s+(.*)$")
				t.key = key
				t.text = text
			else
				local protected = string_fields [event] or {}
				for k, v in string.gmatch (rest, "(%S+)=(%S+)") do
					t [k] = (not protected [k] and tonumber (v)) or v
				end
			end
			emit (t)
		end
	end

	return { emit = emit, emit_report = emit_report }
end
