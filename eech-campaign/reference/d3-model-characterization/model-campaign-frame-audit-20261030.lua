-- Diagnostic only: run the EXACT shipped campaign script with a delegating engine
-- proxy that counts real frame calls. No world reply, input, RNG, time or output
-- is changed. Record its final real clock after the driver has completed.
local dc = require ('eech_dc')
local boot = dc.boot
local real_engine
local frames = 0
dc.boot = function (...)
    assert (not real_engine, 'audit permits one boot')
    real_engine = boot (...)
    return setmetatable ({}, { __index = function (_, key)
        local value = real_engine [key]
        if type (value) ~= 'function' then return value end
        return function (_, ...)
            if key == 'frame' then frames = frames + 1 end
            return value (real_engine, ...)
        end
    end })
end
dofile (assert (host.args.driver, 'driver=the unchanged shipped campaign.lua'))
local clock = assert (real_engine, 'driver did not boot'):clock ()
local output = assert (io.open (assert (host.args.audit), 'w'))
assert (output:write (string.format ('{"actual_frame_calls":%d,"final_elapsed_seconds":%.17g}\n', frames, clock.elapsed_seconds)))
assert (output:close ())
