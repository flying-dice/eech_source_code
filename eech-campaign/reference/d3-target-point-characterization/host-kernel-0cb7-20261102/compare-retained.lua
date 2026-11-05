-- Read-only host geometry comparison. Deliberately never loads/boots eech_dc.
local args = host.args
local json = dofile(assert(args.bin) .. "/lua/json-lines.lua")
local file, why = host.create_exclusive(assert(args.out))
assert(file, why)
local out = json.wrap(file, args.out)
local store = host.model_store(assert(args.root))
local report = store:compare_target_points(assert(args.original))
local listed = report.listed
report.listed = nil
report.event = "comparison"
out.emit(report)
for index, text in ipairs(listed) do out.emit {event="difference", index=index, text=text} end
out.emit {event="done", listed=#listed}
out.close()
