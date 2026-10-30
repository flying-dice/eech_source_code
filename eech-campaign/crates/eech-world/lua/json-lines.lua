-- Strict JSON Lines output for the model witness scripts (EEDCS-50, D3): model-authority.lua, model-provider-check.lua and json-lines-check.lua load it with
-- dofile (here .. "json-lines.lua"), the way campaign.lua loads metrics.lua.
--
-- Lua 5.1's string.format ("%q") is Lua quoting, not JSON: it writes a newline as a backslash and a raw newline and every other control byte as a decimal escape, which a
-- JSON parser refuses. A native report (the diagnostic probe's rows hold newlines) and an error message (control bytes, quotes, backslashes, UTF-8) must survive intact, so
-- the quote, the backslash and every control byte (0..31 and 127) are escaped here and every other byte, UTF-8 included, is kept as it is: the same encoder as
-- model-host-compare.lua. A number must be finite. Every write, flush and close is CHECKED: an output failure raises, so a script never reports success after it lost or
-- truncated its evidence. Nothing is normalised: a string is written as the bytes it holds.

local M = {}

local escapes = { ['"'] = '\\"', ['\\'] = '\\\\', ['\b'] = '\\b', ['\f'] = '\\f', ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t' }

function M.string (text)
	local escaped = string.gsub (text, '[%c"\\]', function (c) return escapes [c] or string.format ("\\u%04x", string.byte (c)) end)
	return '"' .. escaped .. '"'
end

local function value (v)
	local kind = type (v)
	if kind == "string" then
		return M.string (v)
	elseif kind == "boolean" then
		return tostring (v)
	elseif kind == "number" then
		if v ~= v or v == math.huge or v == -math.huge then error ("json-lines: a non-finite number cannot be written", 0) end
		if v == math.floor (v) and math.abs (v) < 2 ^ 53 then return string.format ("%.0f", v) end
		return string.format ("%.17g", v)
	end
	error ("json-lines: a " .. kind .. " cannot be written", 0)
end

-- a writer around an open file (or anything with write, flush and close that report failure as Lua's file methods do: nil and a reason)
function M.wrap (file, path)
	local writer = { path = path, closed = false }

	function writer.emit (t)
		if writer.closed then error ("json-lines: the output " .. tostring (path) .. " is closed", 0) end
		local parts = {}
		for k, v in pairs (t) do parts [#parts + 1] = M.string (tostring (k)) .. ":" .. value (v) end
		table.sort (parts)
		local ok, why = file:write ("{" .. table.concat (parts, ",") .. "}\n")
		if not ok then error ("json-lines: cannot write " .. tostring (path) .. ": " .. tostring (why), 0) end
		ok, why = file:flush ()
		if not ok then error ("json-lines: cannot flush " .. tostring (path) .. ": " .. tostring (why), 0) end
	end

	function writer.close ()
		if writer.closed then return end
		writer.closed = true
		local ok, why = file:close ()
		if not ok then error ("json-lines: cannot close " .. tostring (path) .. ": " .. tostring (why), 0) end
	end

	return writer
end

-- a new file: an existing one is never overwritten
function M.open (path)
	local existing = io.open (path, "r")
	if existing then
		existing:close ()
		error ("the output file already exists and is not overwritten: " .. tostring (path), 0)
	end
	local file, why = io.open (path, "w")
	if not file then error ("cannot open the output file " .. tostring (path) .. ": " .. tostring (why), 0) end
	return M.wrap (file, path)
end

return M
