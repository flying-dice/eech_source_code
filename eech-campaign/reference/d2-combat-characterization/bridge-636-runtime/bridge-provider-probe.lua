-- ROOT focused provider control around the unchanged bridge-case entry point.
-- Default equality uses bridge-case.lua directly. This is altered-host evidence.
local args = host.args
local mode = assert(args.bridge_mode, 'bridge_mode required')
assert(mode == 'own' or mode == 'own_fall' or mode == 'missing' or mode == 'bad_shape', 'unknown bridge_mode')
local case_script = assert(args.bridge_script, 'bridge_script required')
local own = dofile(assert(args.own_script, 'own_script required'))
local output_path = assert(args.provider_out, 'provider_out required')
local exists = io.open(output_path, 'rb')
if exists then exists:close(); error('provider_out already exists') end
local file = assert(io.open(output_path, 'wb'))
local reference = assert(host.aim)
local stats = { requests = 0, kills = 0, shapes = 0, neighbours = 0, falls = 0, rests = 0, vehicles = 0, other = 0, own = 0, changed_falls = 0, malformed = 0 }
host.aim = function(step)
  if step.mode ~= 'bridge' then return reference(step) end
  stats.requests = stats.requests + 1
  if step.op == 'kill' then stats.kills = stats.kills + 1
  elseif step.op == 'shape' then stats.shapes = stats.shapes + 1
  elseif step.op == 'neighbour' then stats.neighbours = stats.neighbours + 1
  elseif step.op == 'fall' then stats.falls = stats.falls + 1
  elseif step.op == 'rest' then stats.rests = stats.rests + 1
  elseif step.op == 'vehicles' then stats.vehicles = stats.vehicles + 1
  else stats.other = stats.other + 1 end
  if mode == 'missing' then
    stats.malformed = stats.malformed + 1
    return nil
  elseif mode == 'bad_shape' and step.op == 'shape' then
    stats.malformed = stats.malformed + 1
    return { state = 'done', variant = 99 }
  elseif mode == 'own' then
    stats.own = stats.own + 1
    return own.bridge(step)
  elseif mode == 'own_fall' and step.op == 'fall' then
    if step.terrain ~= nil and step.delta_time > 0 then stats.changed_falls = stats.changed_falls + 1 end
    return own.bridge(step)
  end
  return reference(step)
end
local ok, reason = pcall(dofile, case_script)
-- Values come from this wrapper, not engine predictions. Only whitelisted mode
-- names and finite integer counters enter JSON; the original error is rethrown
-- unchanged for the native process log.
file:write(string.format('{"mode":"%s","script_completed":%s,"requests":%d,"kills":%d,"shapes":%d,"neighbours":%d,"falls":%d,"rests":%d,"vehicles":%d,"other":%d,"own":%d,"changed_falls":%d,"malformed":%d}\n', mode, tostring(ok), stats.requests, stats.kills, stats.shapes, stats.neighbours, stats.falls, stats.rests, stats.vehicles, stats.other, stats.own, stats.changed_falls, stats.malformed))
assert(file:close())
if not ok then error(reason, 0) end
assert(stats.requests > 0, 'bridge provider never triggered')
assert(stats.other == 0, 'unrecognized bridge operation in probe')
if mode == 'own' then assert(stats.own > 0, 'own model never used') end
if mode == 'own_fall' then assert(stats.changed_falls > 0, 'own fall never used') end
assert(mode ~= 'missing' and mode ~= 'bad_shape', 'malformed provider unexpectedly completed')
