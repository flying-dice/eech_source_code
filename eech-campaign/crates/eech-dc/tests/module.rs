//! The module as DCS loads it: a real Lua 5.1 state, a stub of the mission
//! scripting state's API (`env`, `timer`, `world`, `Unit`, `Airbase`), the
//! module's `luaopen` and its embedded driver.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)] // idiomatic in tests

use std::sync::Mutex;

use mlua::prelude::{LuaFunction, LuaTable};
use mlua::Lua;

/// one campaign per process: the tests take turns
fn serially() -> std::sync::MutexGuard<'static, ()> {
    static SERIAL: Mutex<()> = Mutex::new(());
    SERIAL.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The supply chain (eech-campaign's tests/campaign.rs) placed in a theatre:
/// EECH (0, 0) is the DCS point x (north) = -300000, z (east) = 600000. The
/// airbase is a DCS airbase; the group's members are DCS units.
const MISSION: &str = r#"
REPORTED = {}
EVENTS = {}
SCHEDULED = nil
MODEL_TIME = 0
env = {
  info = function(m) table.insert(REPORTED, m) end,
  error = function(m) table.insert(REPORTED, "ERROR " .. m) end,
}
timer = {
  getTime = function() return MODEL_TIME end,
  scheduleFunction = function(fn, arg, at) SCHEDULED = { fn = fn, at = at } end,
}
world = {
  event = { S_EVENT_MISSION_END = 27 },
  addEventHandler = function(h) HANDLER = h end,
}
-- DCS points: x north, z east
local function point(east, north) return { x = -300000 + north, y = 0, z = 600000 + east } end
UNITS = {
  ["lift-1"] = point(22000, 16000),
  ["lift-2"] = point(22010, 16000),
}
Unit = {
  getByName = function(name)
    local p = UNITS[name]
    if not p then return nil end
    return { isExist = function() return true end, getPoint = function() return p end }
  end,
}
Airbase = {
  getByName = function(name)
    if name == "Airbase One" then return { getPoint = function() return point(22000, 16000) end } end
    return nil
  end,
}
function keysite(name, kind, east, ammo)
  return { name = name, kind = kind, side = "blue", position = { x = east, y = 0, z = 16000 }, supplies = { ammo = ammo, fuel = 100 } }
end
CAMPAIGN = {
  map = { sectors_x = 4, sectors_z = 4, sector_size = 8192 },
  session = "single-player",
  sides = { "blue" },
  keysites = {
    keysite("farp", "FARP", 8000, 5),
    keysite("factory", "FACTORY", 13000, 35),
    { name = "airbase", kind = "AIRBASE", side = "blue", airbase = "Airbase One", supplies = { ammo = 100, fuel = 100 }, landing = { "HELICOPTER" } },
  },
  groups = {
    {
      name = "lift", kind = "MEDIUM_LIFT_TRANSPORT_HELICOPTER", side = "blue", base = "airbase",
      supplies = { ammo = 100, fuel = 100 },
      members = { { name = "lift-1", aircraft = "UH60_BLACK_HAWK" }, { name = "lift-2", aircraft = "UH60_BLACK_HAWK" } },
      registered = true,
    },
  },
}
OPTIONS = {
  campaign = CAMPAIGN,
  origin = { x = -300000, z = 600000 },
  interval = 0.5,
  object_bounds = { [2698] = { min = { x = -1, y = -0.5, z = -1.5 }, max = { x = 1, y = 0.5, z = 1.5 } } },
  on_event = function(e) table.insert(EVENTS, e) end,
}
-- run the scheduled pump like DCS: at its due time, until it unschedules itself
function RUN(limit)
  local n = 0
  while SCHEDULED and n < limit do
    local due = SCHEDULED.at
    MODEL_TIME = due
    local next_at = SCHEDULED.fn(nil, due)
    if next_at then SCHEDULED.at = next_at else SCHEDULED = nil end
    n = n + 1
  end
  return n
end
"#;

fn mission() -> (Lua, LuaTable) {
    let lua = Lua::new();
    lua.load(MISSION).exec().expect("stub the mission state");
    let eech = eech_dc::eech_dc(&lua).expect("luaopen_eech_dc");
    lua.globals().set("eech", eech.clone()).unwrap();
    (lua, eech)
}

fn reported(lua: &Lua) -> String {
    lua.load("return table.concat(REPORTED, '\\n')").eval().unwrap()
}

#[test]
fn the_driver_runs_the_supply_chain_in_a_mission() {
    let _s = serially();
    let (lua, _) = mission();
    lua.load("HANDLE = eech.start(OPTIONS)").exec().expect("eech.start");

    // pump until the campaign stops itself at the slice boundary
    let pumps: u32 = lua.load("return RUN(2000)").eval().unwrap();
    assert_eq!(pumps, 361, "the assignment pass at 180.5 s of model time ends the run");

    let log = reported(&lua);
    assert!(log.contains("campaign started"), "{log}");
    // events cross with the DCS names
    assert!(log.contains("event=low-on-supplies requester=farp side=blue supply=ammo"), "{log}");
    assert!(log.contains("event=supply-mission-requested requester=farp supplier=factory"), "{log}");
    assert!(log.contains("event=mission-created task=#"), "{log}");
    // the boundary is a reported failure that stops the pump, never an escape
    assert!(
        log.contains("ERROR EECH: campaign step failed: runtime error: eech: boundary: slice boundary reached: assign_primary_task_to_group"),
        "{log}"
    );
    assert!(log.contains("campaign closed (step failed)"), "{log}");
    assert!(lua.load("return HANDLE.stopped()").eval::<bool>().unwrap());

    // on_event saw them too, as Lua tables
    let first: String = lua.load("return EVENTS[1].event .. ' ' .. EVENTS[1].requester").eval().unwrap();
    assert_eq!(first, "low-on-supplies farp");
}

#[test]
fn the_world_is_asked_in_eech_coordinates() {
    let _s = serially();
    let (lua, _) = mission();
    // the lift group's units stand elsewhere: the campaign's ETA check reads them through the adapter
    lua.load(
        r#"
        local c = eech.new({ campaign = eech.place(CAMPAIGN, OPTIONS.origin), origin = OPTIONS.origin })
        ASKED = {}
        local w = {
          position = function(name) table.insert(ASKED, name); return Unit.getByName(name):getPoint() end,
          object_bounds = function(model) return OPTIONS.object_bounds[model] end,
        }
        local ok, err
        for i = 1, 400 do
          ok, err = pcall(c.step, c, 0.5, w)
          if not ok then break end
        end
        ERR = err
        SNAP = c:snapshot()
        c:close()
        "#,
    )
    .exec()
    .unwrap();
    let asked: String = lua.load("return table.concat(ASKED, ',')").eval().unwrap();
    assert_eq!(asked, "lift-1", "the locality check reads the first member");
    let err: String = lua.load("return tostring(ERR)").eval().unwrap();
    assert!(err.contains("eech: boundary"), "{err}");
    // the airbase took its position from the DCS airbase, converted
    let airbase: (f64, f64) = lua
        .load("for _, k in ipairs(SNAP.keysites) do if k.name == 'airbase' then return k.supplies.ammo, SNAP.tasks[1].route_length end end")
        .eval()
        .unwrap();
    assert!(airbase.0 > 99.0 && airbase.1 == 4.0, "{airbase:?}");
    let conv: (f64, f64, f64) = lua
        .load("local p = eech.to_eech({ x = -284000, y = 10, z = 622000 }, OPTIONS.origin); return p.x, p.y, p.z")
        .eval()
        .unwrap();
    assert_eq!(conv, (22000.0, 10.0, 16000.0));
}

#[test]
fn a_world_error_reaches_the_mission_log_with_its_cause() {
    let _s = serially();
    let (lua, _) = mission();
    lua.load(
        r#"
        local c = eech.new({ campaign = eech.place(CAMPAIGN, OPTIONS.origin), origin = OPTIONS.origin })
        local w = {
          position = function(name) error("the unit " .. name .. " is not spawned", 0) end,
          object_bounds = function(model) return OPTIONS.object_bounds[model] end,
        }
        for i = 1, 400 do
          OK, ERR = pcall(c.step, c, 0.5, w)
          if not OK then break end
        end
        POISONED = c:is_poisoned()
        c:close()
        "#,
    )
    .exec()
    .unwrap();
    let err: String = lua.load("return tostring(ERR)").eval().unwrap();
    assert!(err.contains("the unit lift-1 is not spawned"), "{err}");
    assert!(err.contains("eech: world"), "{err}");
    assert!(lua.load("return POISONED").eval::<bool>().unwrap());
}

#[test]
fn mission_end_releases_the_campaign_and_the_next_mission_can_start_one() {
    let _s = serially();
    let (lua, eech) = mission();
    lua.load("HANDLE = eech.start(OPTIONS)").exec().unwrap();
    lua.load("RUN(10)").exec().unwrap();
    // one campaign per process: a second mission's campaign waits for this one
    let second: LuaFunction = eech.get("new").unwrap();
    let err = second
        .call::<mlua::Value>(
            lua.load("return { campaign = eech.place(CAMPAIGN, OPTIONS.origin) }")
                .eval::<LuaTable>()
                .unwrap(),
        )
        .unwrap_err();
    assert!(err.to_string().contains("eech: already-running"), "{err}");

    // an unrelated event changes nothing; the mission end releases
    lua.load("HANDLER:onEvent({ id = 1 })").exec().unwrap();
    assert!(!lua.load("return HANDLE.stopped()").eval::<bool>().unwrap());
    // a raising event is contained
    lua.load(r#"HANDLER:onEvent(setmetatable({}, { __index = function() error("unreadable", 0) end }))"#)
        .exec()
        .unwrap();
    assert!(reported(&lua).contains("event handler error: unreadable"));
    lua.load("HANDLER:onEvent({ id = world.event.S_EVENT_MISSION_END })").exec().unwrap();
    assert!(lua.load("return HANDLE.stopped()").eval::<bool>().unwrap());
    assert!(reported(&lua).contains("campaign closed (mission end)"));
    // the pump unschedules itself
    assert!(lua.load("return SCHEDULED.fn(nil, MODEL_TIME + 1) == nil").eval::<bool>().unwrap());

    // the next mission's state
    let (next, _) = mission();
    next.load("HANDLE = eech.start(OPTIONS)").exec().expect("a new campaign after the release");
    next.load("HANDLE.stop('test end')").exec().unwrap();
}

#[test]
fn a_campaign_is_released_when_its_state_closes() {
    let _s = serially();
    {
        let (lua, _) = mission();
        lua.load("KEEP = eech.new({ campaign = eech.place(CAMPAIGN, OPTIONS.origin) })").exec().unwrap();
        // no close, no mission end: lua_close collects the userdata
    }
    let (lua, _) = mission();
    lua.load("local c = eech.new({ campaign = eech.place(CAMPAIGN, OPTIONS.origin) }); c:close()")
        .exec()
        .expect("the dropped state released the campaign");
}

#[test]
fn a_state_the_driver_cannot_run_in_fails_the_call_not_the_process() {
    let _s = serially();
    let lua = Lua::new();
    let eech = eech_dc::eech_dc(&lua).expect("luaopen");
    lua.globals().set("eech", eech).unwrap();
    // no mission API at all: start raises a Lua error with the cause
    lua.load(MISSION).exec().unwrap();
    lua.load("Airbase = nil").exec().unwrap();
    let err = lua.load("eech.start(OPTIONS)").exec().unwrap_err().to_string();
    assert!(err.contains("Airbase"), "{err}");
    // invalid configurations are refused as Lua errors
    let err = lua
        .load("eech.new({ campaign = { map = { sectors_x = 1, sectors_z = 1, sector_size = 1 }, sides = {} } })")
        .exec()
        .unwrap_err()
        .to_string();
    assert!(err.contains("eech: invalid-config"), "{err}");
    let err = lua.load("eech.new({ campaign = { sides = 'blue' } })").exec().unwrap_err().to_string();
    assert!(!err.is_empty());
}
