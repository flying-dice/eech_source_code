-- Private focused host-answer probe. The original verified combat-case.lua and
-- its preconditions, observers, seed, warmup and frame timing remain unchanged.
local args = host.args
local case_script = assert(args.case_script, "case_script=<verified combat-case.lua>")
local summary_path = assert(args.probe_summary, "probe_summary=<path>")
local mode = assert(args.timed_probe_mode, "timed_probe_mode=<mode>")
assert(mode == "timed_never" or mode == "timed_rate150" or
       mode == "bad_health_second_draw" or mode == "bad_health_fatal", "unsupported timed probe mode")
local case_dir = assert(case_script:match("^(.*[/\\])"))
local summary_file = assert(io.open(summary_path, "w"))
local writer = dofile(case_dir .. "combat-report.lua")(summary_file)
local original = host.aim
local counts = { requests_with_sample = 0, draw_requests = 0,
                 eligible_answers = 0, hit_answers = 0,
                 second_draw_requests = 0, inconsistent_fatal_answers = 0 }
local function observe(step, answer)
    if step.mode == "health" and step.op == "timed" then
        if step.draw ~= nil then counts.requests_with_sample = counts.requests_with_sample + 1 end
        if answer.state == "need_draw" then
            counts.draw_requests = counts.draw_requests + 1
            if step.draw ~= nil then counts.second_draw_requests = counts.second_draw_requests + 1 end
        elseif answer.state == "done" then
            if answer.eligible then counts.eligible_answers = counts.eligible_answers + 1 end
            if answer.hit then
                counts.hit_answers = counts.hit_answers + 1
                if answer.fatal ~= (answer.new_level <= 0) then
                    counts.inconsistent_fatal_answers = counts.inconsistent_fatal_answers + 1
                end
            end
        end
    end
    return answer
end
local provider
if mode == "bad_health_second_draw" or mode == "bad_health_fatal" then
    local malformed = dofile(case_dir .. "aim-malformed.lua")
    -- The existing malformed provider owns its one-shot corruption. Observe the
    -- returned answer outside it too, including answers it deliberately corrupts.
    local mutate = malformed(mode, original, function(_, answer) return answer end)
    provider = function(step) return observe(step, mutate(step)) end
else
    provider = function(step)
        local request = step
        if step.mode == "health" and step.op == "timed" then
            request = {}
            for key, value in pairs(step) do request[key] = value end
            if mode == "timed_never" then
                request.explosive, request.flammable, request.aircraft_class = false, false, 0
            else
                request.delta_time = request.delta_time * 1.5
            end
        end
        return observe(step, original(request))
    end
end
host.aim = provider
local ok, failure = pcall(dofile, case_script)
host.aim = original
local summary = { event = "timed_aa_probe", mode = mode, script_completed = ok,
                  failure = ok and "" or tostring(failure),
                  scope = "changed host answers around the unchanged controlled fixture; not a default byte comparison or fullpack" }
for key, value in pairs(counts) do summary[key] = value end
writer.emit(summary)
summary_file:close()
-- Keep the actual failed native disposition for malformed answers. A summary
-- cannot turn an unexpected error or a poisoned engine into a healthy run.
if not ok then error(failure, 0) end
