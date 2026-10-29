-- Native one-shot geometry-state control; original paging witness boot reused verbatim.
io.stdout:setvbuf ("no")

local args = host.args

local function refuse (message)
	error ("collision-trace-paging: " .. message, 0)
end

local known = { root = true, out = true, scenario = true }
for key in pairs (args) do
	if not known [key] then refuse ("unknown argument '" .. tostring (key) .. "'") end
end
local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
	georgia = { map = "..\\common\\maps\\map16", campaign = "georgia.chc" },
}
local scenario_name = args.scenario == nil and "lebanon_retail" or args.scenario
local scenario = scenarios [scenario_name]
if scenario == nil then refuse ("scenario must be lebanon_retail, georgia_retail or georgia, got '" .. tostring (scenario_name) .. "'") end
local existing = io.open (out_path, "rb")
if existing then
	existing:close ()
	refuse ("out '" .. out_path .. "' already exists: an existing file is never overwritten")
end

local out = assert (io.open (out_path, "w"))

local function json_string (text)
	return "\"" .. string.gsub (text, "[%z\1-\31\"\\]", function (c) return string.format ("\\u%04x", string.byte (c)) end) .. "\""
end

local function emit (event, fields)
	local parts = { "\"event\":" .. json_string (event) }
	for _, item in ipairs (fields or {}) do
		local key, value = item [1], item [2]
		parts [#parts + 1] = json_string (key) .. ":" .. (type (value) == "string" and json_string (value) or tostring (value))
	end
	out:write ("{" .. table.concat (parts, ",") .. "}\n")
	out:flush ()
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = scenario.map, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
	line_of_sight = host.line_of_sight,
	sense = host.sense, aim = host.aim,
}
emit ("boot", { { "scenario", scenario_name } })


-- One trace arming, three probes: default -> controlled -> default. No trace
-- re-arming can hide state left by the one-shot geometry control.
local function capture ()
    local facts, total = engine:collision_trace_read (0, 0, 1)
    local pages = {}
    local offset = 0
    while offset < total do
        local page, again = engine:collision_trace_read (1, offset, 256)
        assert (again == total, "trace total changed")
        pages[#pages + 1] = page
        offset = offset + math.min (256, total - offset)
    end
    return facts, table.concat (pages), total
end

local function save (suffix, text)
    local f = assert (io.open (out_path .. suffix, "wb"))
    assert (f:write (text))
    assert (f:close ())
end

local function main ()
    engine:collision_trace_arm (true)
    local before = engine:collision_probe (20261029, 5, 5)
    local before_facts, before_records, before_total = capture ()
    save (".before-report.txt", before)
    save (".before-facts.txt", before_facts)
    save (".before-records.txt", before_records)
    assert (not string.find (before, "col%-geometry%-witness"), "default report includes witness")
    assert (not string.find (before_facts, "tf%-witness"), "default facts include witness")
    assert (not string.find (before_records, "g_calls="), "default records include witness")
    emit ("default_before", { { "records", before_total } })

    engine:collision_geometry_arm (true)
    local controlled = engine:collision_probe (20261029, 5, 5)
    local controlled_facts, controlled_records, controlled_total = capture ()
    save (".controlled-report.txt", controlled)
    save (".controlled-facts.txt", controlled_facts)
    save (".controlled-records.txt", controlled_records)
    assert (string.find (controlled, "col%-geometry%-witness"), "controlled report missing witness")
    assert (string.find (controlled_facts, "tf%-witness"), "controlled facts missing witness")
    assert (string.find (controlled_records, "g_calls="), "controlled records missing witness")
    emit ("controlled", { { "records", controlled_total } })

    local after = engine:collision_probe (20261029, 5, 5)
    local after_facts, after_records, after_total = capture ()
    save (".after-report.txt", after)
    save (".after-facts.txt", after_facts)
    save (".after-records.txt", after_records)
    emit ("default_after", { { "records", after_total }, { "report_equal", before == after },
        { "facts_equal", before_facts == after_facts }, { "records_equal", before_records == after_records },
        { "witness_fields_present", string.find (after_records, "g_calls=", 1, true) ~= nil } })
    assert (before == after, "default report changed after one-shot witness")
    assert (before_facts == after_facts, "default facts changed after one-shot witness")
    assert (before_records == after_records, "default records changed after one-shot witness")
    assert (before_total == after_total, "default record total changed")
end

local ok, err = pcall (main)
pcall (function () engine:collision_geometry_arm (false) end)
pcall (function () engine:collision_trace_arm (false) end)
if not ok then
    emit ("failed", { { "error", tostring (err) } })
    out:close ()
    error (err, 0)
end
emit ("done")
out:close ()
print ("TREE_GEOMETRY_ONE_SHOT_OK")
