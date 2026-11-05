-- Private own-model probe around the unchanged verified controlled fixture.
-- PERSON starts with 3HP in this retail witness; reference shots kill it, so
-- the host instead reports one-point damage to leave a living wounded target.
local args = host.args
local case_script = assert(args.case_script)
local case_dir = assert(case_script:match("^(.*[/\\])"))
local summary_file = assert(io.open(assert(args.probe_summary), "w"))
local writer = dofile(case_dir .. "combat-report.lua")(summary_file)
local original = host.aim
local counts = { person_damage_answers = 0, person_damage_writes = 0,
                 generic_draw_requests = 0, generic_samples = 0,
                 generic_hits = 0, generic_fatals = 0 }
host.aim = function(step)
    if step.mode == "damage" and step.critical_model == 2 and
       not (step.target_not_ai and step.invulnerable) then
        local capability = step.damage_capability > 0 and 1 or 0
        local written = capability > 0
        local level = written and math.max(0, step.damage_level - 1) or step.damage_level
        counts.person_damage_answers = counts.person_damage_answers + 1
        if written then counts.person_damage_writes = counts.person_damage_writes + 1 end
        counts.last_person_damage_level = level
        return { state = "done", exited = false, capability = capability,
                 written = written, new_level = level,
                 critical_before = 0, critical_after = 0, bad_modifier = false }
    end
    if step.mode == "health" and step.op == "timed" and
       step.critical_model == 2 and step.damage_level > 0 and
       step.damage_level < step.initial_damage_level then
        if step.draw == nil then
            counts.generic_draw_requests = counts.generic_draw_requests + 1
            return { state = "need_draw" }
        end
        counts.generic_samples = counts.generic_samples + 1
        local hit = step.draw < 4096 -- this host's stochastic model: one eighth
        local level = hit and math.max(0, step.damage_level - 1) or step.damage_level
        local fatal = hit and level <= 0
        if hit then counts.generic_hits = counts.generic_hits + 1 end
        if fatal then counts.generic_fatals = counts.generic_fatals + 1 end
        counts.last_generic_level = level
        return { state = "done", critical_class = 0, crossed = false,
                 recovered = false, eject = false, eligible = true,
                 rolled = true, hit = hit, new_level = level, fatal = fatal }
    end
    return original(step)
end
local ok, failure = pcall(dofile, case_script)
host.aim = original
local summary = { event = "timed_generic_person_probe",
                  script_completed = ok, failure = ok and "" or tostring(failure),
                  scope = "own DAMAGE model plus stochastic PERSON TIMED model; not a timed-only or default-byte comparison" }
for key, value in pairs(counts) do summary[key] = value end
writer.emit(summary)
summary_file:close()
if not ok then error(failure, 0) end
