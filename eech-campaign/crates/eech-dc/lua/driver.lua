-- eech_dc mission driver: eech.start (options) and eech.place (config, origin).
--
-- Embedded in eech_dc.dll and loaded by luaopen; receives the module table
-- and returns `start` and `place`. It runs the EECH campaign inside DCS's mission scripting
-- state:
--
--   * the campaign map is placed on the theatre by `origin` (the DCS point of
--     EECH (0, 0)); keysites may name a DCS airbase (`airbase = "Batumi"`),
--     whose point then gives their position;
--   * the World the campaign asks during a step is DCS itself: an aircraft's
--     position is Unit.getByName (name):getPoint (), the configuration name
--     being the DCS unit name; object bounds come from `options.object_bounds`;
--   * the step is pumped on model time (timer.scheduleFunction), with the
--     model time elapsed since the last pump as the frame delta;
--   * every campaign event goes to `options.on_event` (if any) and dcs.log;
--   * the campaign is released on S_EVENT_MISSION_END, while the state is
--     still whole, and stops pumping.
--
-- Every function DCS calls from here (the scheduled pump, the event handler)
-- is protected: DCS calls them from its C++ dispatcher, which has nothing to
-- catch a raise (the reference DCS module crate's rule). A campaign failure
-- stops the pump and is reported; it never escapes into DCS.
--
-- options = {
--   campaign = <CampaignConfig>,                 required
--   origin = { x = <north>, z = <east> },        DCS point of EECH (0, 0); default { x = 0, z = 0 }
--   interval = 1.0,                              seconds of model time between steps
--   object_bounds = { [model] = { min = {x,y,z}, max = {x,y,z} } },
--   on_event = function (event) end,
-- }
--
-- Returns a handle: { campaign = <campaign>, stop = function (why) end, stopped = function () end }.
local eech = ...

local function report(level, message)
  local text = "EECH: " .. message
  if type(env) == "table" then
    if level == "error" and env.error then
      env.error(text, false)
    elseif env.info then
      env.info(text)
    end
  end
end

local function describe(event)
  local parts = {}
  for k, v in pairs(event) do
    if type(v) ~= "table" then
      parts[#parts + 1] = k .. "=" .. tostring(v)
    end
  end
  table.sort(parts)
  return table.concat(parts, " ")
end

-- eech.place (config, origin): the campaign configuration with every keysite
-- that names a DCS airbase (`airbase = "Batumi"`) positioned on it. Returns
-- the configuration (edited in place).
local function place(config, origin)
  for _, keysite in ipairs(config.keysites or {}) do
    if keysite.airbase then
      local airbase = Airbase.getByName(keysite.airbase)
      assert(airbase, "eech: no DCS airbase named " .. tostring(keysite.airbase))
      keysite.position = eech.to_eech(airbase:getPoint(), origin)
      keysite.airbase = nil
    end
  end
  return config
end

local function start(options)
  assert(type(options) == "table" and type(options.campaign) == "table", "eech.start needs options.campaign")
  local origin = options.origin or { x = 0, z = 0 }
  local interval = options.interval or 1.0

  place(options.campaign, origin)

  local campaign = eech.new({ campaign = options.campaign, origin = origin })

  local bounds = options.object_bounds or {}

  local dcs_world = {
    position = function(name)
      local unit = Unit.getByName(name)
      if unit and unit:isExist() then
        return unit:getPoint()
      end
      return nil
    end,
    object_bounds = function(model)
      return bounds[model]
    end,
  }

  local stopped = false
  local last = timer.getTime()

  local handle = { campaign = campaign }

  function handle.stop(why)
    if stopped then
      return
    end
    stopped = true
    local closed = campaign:close()
    report("info", "campaign " .. (closed and "closed" or "was already closed") .. " (" .. tostring(why) .. ")")
  end

  function handle.stopped()
    return stopped
  end

  local function deliver(events)
    for _, event in ipairs(events) do
      report("info", describe(event))
      if options.on_event then
        local ok, err = pcall(options.on_event, event)
        if not ok then
          report("error", "on_event raised: " .. tostring(err))
        end
      end
    end
  end

  -- the pump: one campaign step per interval of model time
  local function pump(_, now)
    if stopped then
      return nil
    end
    local dt = now - last
    last = now
    if dt <= 0 then
      return now + interval
    end
    local ok, events = pcall(campaign.step, campaign, dt, dcs_world)
    if ok then
      deliver(events)
      return now + interval
    end
    -- a failed step poisons the campaign: report it, with what it did first, and stop
    report("error", "campaign step failed: " .. tostring(events))
    local drained, before = pcall(campaign.drain_events, campaign)
    if drained then
      deliver(before)
    end
    handle.stop("step failed")
    return nil
  end

  timer.scheduleFunction(pump, nil, timer.getTime() + interval)

  -- teardown while the state is still whole
  if type(world) == "table" and world.addEventHandler and world.event then
    world.addEventHandler({
      onEvent = function(_, event)
        local ok, err = pcall(function()
          if event.id == world.event.S_EVENT_MISSION_END then
            handle.stop("mission end")
          end
        end)
        if not ok then
          report("error", "event handler error: " .. tostring(err))
        end
      end,
    })
  end

  report("info", "campaign started (eech_dc " .. eech.version .. ")")
  return handle
end

return { start = start, place = place }
