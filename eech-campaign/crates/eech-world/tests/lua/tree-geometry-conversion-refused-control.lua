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



local function save (suffix, text)
 local f=assert(io.open(out_path..suffix,'wb'));assert(f:write(text));assert(f:close())
end
local function main ()
 engine:collision_trace_arm(true)
 local before=engine:collision_probe(20261029,5,5)
 save('.before-report.txt',before)
 engine:collision_geometry_arm(true)
 local ok,err=pcall(function() return engine:collision_probe("not-a-seed",5,5) end)
 emit('conversion_refused',{{'refused',not ok},{'error',tostring(err)}})
 assert(not ok,'invalid case count was accepted')
 local after=engine:collision_probe(20261029,5,5)
 save('.after-report.txt',after)
 local witness=string.find(after,'col-geometry-witness',1,true)~=nil
 emit('next_default',{{'report_equal',before==after},{'witness_present',witness}})
 assert(not witness,'geometry arming survived a refused public probe')
 assert(before==after,'default report changed after refused probe')
end
local ok,err=pcall(main)
pcall(function() engine:collision_geometry_arm(false) end)
pcall(function() engine:collision_trace_arm(false) end)
if not ok then emit('failed',{{'error',tostring(err)}});out:close();error(err,0) end
emit('done');out:close();print('TREE_GEOMETRY_REFUSAL_CONSUMES_ARM_OK')
