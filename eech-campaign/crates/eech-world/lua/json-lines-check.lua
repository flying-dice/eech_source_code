-- Checks of the strict JSON Lines writer (json-lines.lua) the model witness scripts use. It boots nothing and needs no DLL. host.args: out (a new JSON Lines path).
-- It writes rows whose strings hold every control byte, quotes, backslashes, newlines and UTF-8, so the test that drives it can parse the file with a real JSON parser and compare
-- every string byte for byte, and it shows that each failed write, flush or close, an existing output and a non-finite number are RAISED, never reported as success.

local here = debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or ""
local json = dofile (here .. "json-lines.lua")

local out = json.open (assert (host.args.out, "out=<JSON Lines path>"))

-- every control byte 0..31 and 127, the quote and the backslash, a multi-line native-style report, and UTF-8 (e, acute accent; a check mark; a 4-byte sequence)
local controls = {}
for byte = 0, 31 do controls [#controls + 1] = string.char (byte) end
controls [#controls + 1] = string.char (127)
out.emit { event = "controls", text = table.concat (controls), quote_backslash = 'a"b\\c', report = "model-host-scene scene=1 radius=0x3f800000\nmodel-host-tri scene=1 depth=0 return=1\n",
	utf8 = "caf\195\169 \226\156\147 \240\159\152\128" }
out.emit { event = "numbers", zero = 0, negative = -3, fraction = 1.5, tiny = 1e-20, big = 2 ^ 40, yes = true, no = false }

-- failures are raised
local function raised (f) local ok, why = pcall (f) return (not ok) and tostring (why) or "" end

out.emit { event = "nonfinite", refused = raised (function () out.emit { bad = 0 / 0 } end) ~= "" and raised (function () out.emit { bad = 1 / 0 } end) ~= "" }
out.emit { event = "unwritable_value", refused = raised (function () out.emit { bad = {} } end) ~= "" }

local function failing (which)
	local file = {}
	function file:write () if which == "write" then return nil, "disk full" end return self end
	function file:flush () if which == "flush" then return nil, "flush failed" end return true end
	function file:close () if which == "close" then return nil, "close failed" end return true end
	local w = json.wrap (file, "fake")
	if which == "close" then w.emit { a = 1 } return raised (function () w.close () end) end
	return raised (function () w.emit { a = 1 } end)
end
out.emit { event = "output_failures", write = failing ("write"), flush = failing ("flush"), close = failing ("close") }

-- an existing output is not overwritten, and a closed writer refuses a row
out.emit { event = "existing", refused = raised (function () json.open (host.args.out) end) }
local second = json.wrap (io.tmpfile (), "tmp")
second.close ()
out.emit { event = "closed", refused = raised (function () second.emit { a = 1 } end) }

out.close ()
