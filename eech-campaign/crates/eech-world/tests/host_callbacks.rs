//! The host's own native callbacks fail through the same mlua error path as the DLL's
//! (docs/m5-dcs-lua-abi.md): run the host binary on a script that makes host.log fail.
use std::process::Command;

const FAILING_HOST_LOG_SCRIPT: &str = "host.log('a good host callback')
for _ = 1, 100 do
  local ok, err = pcall(host.log, {})
  assert(not ok, 'host.log accepted a table')
  assert(tostring(err):find('string', 1, true), tostring(err))
end
collectgarbage('collect')
print('HOST_LOG_ERRORS=100')
";

#[test]
fn a_failing_host_callback_returns_its_error_to_pcall() {
    let script = std::env::temp_dir().join(format!("eech-world-host-callbacks-{}.lua", std::process::id()));
    std::fs::write(&script, FAILING_HOST_LOG_SCRIPT).unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_eech-world")).arg(&script).output();
    std::fs::remove_file(&script).unwrap();
    let run = run.unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run.status.success(), "{}\n{stdout}\n{stderr}", run.status);
    assert!(stdout.contains("HOST_LOG_ERRORS=100"), "{stdout}\n{stderr}");
    assert!(stderr.contains("[campaign] a good host callback"), "{stderr}");
}

const MOVE_VEHICLE_SCRIPT: &str = "local pose = { position = { 0, 0, 0 }, velocity = 0, attitude = { 1, 0, 0, 0, 1, 0, 0, 0, 1 } }
local id = host.bind_mobile({ member_key = 1, group_key = 1, entity_id = 1, entity_type = 1, sub_type = 1, side = 1, pose = pose, movement_kind = 1 })
local function stop(host_id, from) return host.move_vehicle({ host_id = host_id, mode = 'stop', pose = from }) end
assert(stop(id, pose).velocity == 0)
local ok, err = pcall(stop, 'eech-world:999', pose)
assert(not ok and tostring(err):find('unknown host id eech-world:999', 1, true), tostring(err))
local moved = { position = { 1, 0, 0 }, velocity = 0, attitude = pose.attitude }
ok, err = pcall(stop, id, moved)
assert(not ok and tostring(err):find('differs from the host', 1, true), tostring(err))
ok, err = pcall(host.move_vehicle, { host_id = id, mode = 'glide', pose = pose })
assert(not ok and tostring(err):find('unknown mode glide', 1, true), tostring(err))
local man = host.bind_mobile({ member_key = 2, group_key = 1, entity_id = 2, entity_type = 2, sub_type = 1, side = 1, pose = pose, movement_kind = 3 })
local function step(host_id, mode, extra)
  local request = { host_id = host_id, mode = mode, pose = pose }
  for key, value in pairs(extra or {}) do request[key] = value end
  return host.move_vehicle(request)
end
local begin = { delta_seconds = 0.1, ps_vel = 1.5, waypoint = { 10, 0, 0 }, terrain_follow = true }
ok, err = pcall(step, id, 'ship_move', { delta_seconds = 0.1, frame_delta = 0.1, desired_velocity = 1, max_acceleration = 1, g_max = 1, waypoint = { 10, 0, 0 }, map_min = { -1e6, 0, -1e6 }, map_max = { 1e6, 0, 1e6 } })
assert(not ok and tostring(err):find('is not a request for', 1, true), tostring(err))
ok, err = pcall(step, id, 'person_begin', begin)
assert(not ok and tostring(err):find('is not a request for', 1, true), tostring(err))
ok, err = pcall(step, man, 'person_end', { elevation = 5 })
assert(not ok and tostring(err):find('without a pending person_begin', 1, true), tostring(err))
local advanced = step(man, 'person_begin', begin)
assert(advanced.pending == true)
ok, err = pcall(step, man, 'stop')
assert(not ok and tostring(err):find('is not a request for', 1, true), tostring(err))
ok, err = pcall(step, man, 'person_begin', begin)
assert(not ok and tostring(err):find('while its person step is pending', 1, true), tostring(err))
local settled = step(man, 'person_end', { elevation = 5 })
assert(settled.pending == false and settled.position[2] > 5 and settled.position[2] < 5.1, tostring(settled.position[2]))
print('MOVE_VEHICLE_ERRORS=3')
print('SURFACE_MOVEMENT_ERRORS=5')
";

#[test]
fn move_vehicle_refuses_an_unknown_id_a_foreign_pose_and_an_unknown_mode() {
    let script = std::env::temp_dir().join(format!("eech-world-move-vehicle-{}.lua", std::process::id()));
    std::fs::write(&script, MOVE_VEHICLE_SCRIPT).unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_eech-world")).arg(&script).output();
    std::fs::remove_file(&script).unwrap();
    let run = run.unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run.status.success(), "{}\n{stdout}\n{stderr}", run.status);
    assert!(stdout.contains("MOVE_VEHICLE_ERRORS=3"), "{stdout}\n{stderr}");
}

const HELICOPTER_HOSTED_SCRIPT: &str = "local pose = { position = { 5000, 0, 5000 }, velocity = 0, attitude = { 1, 0, 0, 0, 1, 0, 0, 0, 1 } }
local function bind(key)
  return host.bind_mobile({ member_key = key, group_key = 1, entity_id = key, entity_type = 5, sub_type = 1, side = 1, pose = pose, movement_kind = 5, motion_vector = { 0, 0, 100 }, rotor = { 100, 100, 0 } })
end
local leader, follower = bind(1), bind(2)
local function step(host_id, mode, extra)
  local request = { host_id = host_id, mode = mode, pose = pose }
  for key, value in pairs(extra or {}) do request[key] = value end
  return host.move_vehicle(request)
end
local function plan(members)
  return { hc_plan_members = members, hc_group_leader = false, hc_task_leader = true, hc_task_leader_is_self = true, hc_position_hold = false,
    terrain_follow = true, hc_terrain_follow_attack_altitude = false, hc_waypoint_kind = 1, hc_guide_altitude_valid = false,
    waypoint = { 5000, 0, 6000 }, hc_waypoint_velocity = { 0, 0, 0 }, hc_distance = 1000, elevation = 0, hc_fuel_supply_level = 100,
    hc_cruise_velocity = 50, power_output = 1, hc_cruise_altitude = 50, hc_attack_altitude = 30, hc_leader_radar_altitude = 0,
    map_min = { -1e6, 0, -1e6 }, map_max = { 1e6, 0, 1e6 }, delta_seconds = 0.5, motion_vector_in = { 0, 0, 100 } }
end
-- a member the roster lacks travels as a table without a host id and is refused by position, not skipped
local ok, err = pcall(step, follower, 'helicopter_plan', plan({ { host_id = leader }, {} }))
assert(not ok and tostring(err):find('group member #1 names no host id', 1, true), tostring(err))
local planned = step(follower, 'helicopter_plan', plan({ { host_id = leader } }))
local query = planned.query
assert(query and planned.guidance == nil, 'a plan answers the query only')
assert(type(query.structure_scan) == 'boolean' or type(query.structure_scan) == 'number', type(query.structure_scan))
assert(query.terrain_sample_count >= 1 and query.terrain_sample_count <= 2, query.terrain_sample_count)
assert(#query.terrain_sample == 4 and #query.structure_position == 3)
local function samples(count)
  local list, running = {}, 0
  for i = 1, count do
    local first = 10 - 5 * (i - 1)
    if running > first then list[i] = { first = first } else running = first; list[i] = { first = first, second = first } end
  end
  return list
end
-- a terrain-sample count that differs from the plan is refused and leaves the plan open
ok, err = pcall(step, follower, 'helicopter_guide', { motion_vector_in = { 0, 0, 100 }, hc_structures = {}, hc_terrain_samples = samples(query.terrain_sample_count + 1) })
assert(not ok and tostring(err):find('terrain samples answered', 1, true), tostring(err))
local structures = { { position = { 5000, 0, 5500 }, avoidance_radius = 20, height = 15 } }
local guided = step(follower, 'helicopter_guide', { motion_vector_in = { 0, 0, 100 }, hc_structures = structures, hc_terrain_samples = samples(query.terrain_sample_count) })
local g = guided.guidance
assert(g and guided.query == nil and type(g.lift) == 'number' and #g.wp_vec == 3 and #g.world_acceleration == 3 and #g.waypoint == 3, 'a guide answers the guidance only')
-- abandoning the open step discards it: its guidance cannot be begun
assert(step(follower, 'helicopter_abandon').pending == false)
ok, err = pcall(step, follower, 'helicopter_guided_begin', { delta_seconds = 0.5, motion_vector_in = { 0, 0, 100 } })
assert(not ok and tostring(err):find('without host guidance', 1, true), tostring(err))
print('HELICOPTER_HOSTED_LUA=ok')
";

#[test]
fn the_hosted_helicopter_plan_guide_and_abandon_round_trip_through_lua() {
    let script = std::env::temp_dir().join(format!("eech-world-hc-hosted-{}.lua", std::process::id()));
    std::fs::write(&script, HELICOPTER_HOSTED_SCRIPT).unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_eech-world")).arg(&script).output();
    std::fs::remove_file(&script).unwrap();
    let run = run.unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run.status.success(), "{}\n{stdout}\n{stderr}", run.status);
    assert!(stdout.contains("HELICOPTER_HOSTED_LUA=ok"), "{stdout}\n{stderr}");
}

const DEPENDENT_CARRY_SCRIPT: &str = "local level = { 1, 0, 0, 0, 1, 0, 0, 0, 1 }
local function at(x, z) return { position = { x, 0, z }, velocity = 0, attitude = level } end
local function bind(key, kind, x, z, extra)
  local request = { member_key = key, group_key = 1, entity_id = key, entity_type = key, sub_type = 1, side = 1, pose = at(x, z), movement_kind = kind }
  for name, value in pairs(extra or {}) do request[name] = value end
  return host.bind_mobile(request)
end
local ship, child, grandchild = bind(1, 1, 0, 0), bind(2, 1, 10, 0), bind(3, 1, 20, 0)
local function carry(host_id, pose, extra)
  local request = { host_id = host_id, mode = 'dependent_carry', pose = pose, dc_helicopter = false, dc_heading = 0, dc_velocity = 0 }
  for name, value in pairs(extra) do request[name] = value end
  return host.move_vehicle(request)
end
local function from_parent(parent, old, new) return { dc_parent = parent, dc_parent_old = old, dc_parent_new = new } end
-- the parent is written and placed after its dependents are told, so the host still holds its old position
local moved = carry(child, at(10, 0), from_parent(ship, { 0, 0, 0 }, { 5, 0, 5 }))
assert(moved.position[1] == 5 and moved.position[3] == 5, 'a routed child adopts the parent position')
-- the host moved the child, so the grandchild (told with the child's old position) is carried from the child's new record
local nested = carry(grandchild, at(20, 0), from_parent(child, { 10, 0, 0 }, { 5, 0, 5 }))
assert(nested.position[1] == 5 and nested.position[3] == 5, 'a nested child adopts its parent position')
-- the host's records now agree with what the campaign wrote: a later step is checked against the carried pose
assert(host.move_vehicle({ host_id = child, mode = 'stop', pose = at(5, 5) }).velocity == 0)
local function refused(fragment, host_id, pose, extra)
  local ok, err = pcall(carry, host_id, pose, extra)
  assert(not ok and tostring(err):find(fragment, 1, true), fragment .. ' / ' .. tostring(err))
end
refused('is the child itself', child, at(5, 5), from_parent(child, { 5, 5, 5 }, { 6, 5, 5 }))
refused('is not a host id with a movement record', child, at(5, 5), from_parent('eech-world:999', { 0, 0, 0 }, { 1, 0, 0 }))
refused('a move the host did not make', child, at(5, 5), from_parent(ship, { 9, 0, 9 }, { 1, 0, 0 }))
refused('dc_parent_new must be 3 finite numbers', child, at(5, 5), from_parent(ship, { 0, 0, 0 }, { 0 / 0, 0, 0 }))
refused('differs from the host', child, at(6, 5), from_parent(ship, { 0, 0, 0 }, { 6, 0, 5 }))
refused('Lua nil to String', child, at(5, 5), { dc_parent_old = { 0, 0, 0 }, dc_parent_new = { 6, 0, 5 } })
-- a helicopter child is turned by the parent's heading change and adopts its velocity; a routed child is not a helicopter shape
local pilot = bind(4, 5, 100, 100, { motion_vector = { 0, 0, 100 }, rotor = { 100, 100, 0 } })
local turned = carry(pilot, at(100, 100), { dc_parent = ship, dc_helicopter = true, dc_parent_old = { 0, 0, 0 }, dc_parent_new = { 6, 0, 5 }, dc_heading = 0, dc_velocity = 7 })
assert(turned.velocity == 7 and #turned.attitude == 9, 'a helicopter child adopts the parent velocity and rebuilt attitude')
assert(math.abs(turned.position[1] - 106) < 1e-4 and math.abs(turned.position[3] - 105) < 1e-4, turned.position[1] .. ',' .. turned.position[3])
refused('is not a request for', child, at(5, 5), { dc_parent = ship, dc_helicopter = true, dc_parent_old = { 0, 0, 0 }, dc_parent_new = { 6, 0, 5 } })
print('DEPENDENT_CARRY_LUA=ok')
";

#[test]
fn a_dependent_carry_round_trips_through_lua_and_refuses_what_the_host_did_not_move() {
    let script = std::env::temp_dir().join(format!("eech-world-dependent-carry-{}.lua", std::process::id()));
    std::fs::write(&script, DEPENDENT_CARRY_SCRIPT).unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_eech-world")).arg(&script).output();
    std::fs::remove_file(&script).unwrap();
    let run = run.unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run.status.success(), "{}\n{stdout}\n{stderr}", run.status);
    assert!(stdout.contains("DEPENDENT_CARRY_LUA=ok"), "{stdout}\n{stderr}");
}

const LINE_OF_SIGHT_SCRIPT: &str = "local function begin(extra)
  local step = { mode = 'begin', entity_form = false, criteria = 1, source_eye = false, target_eye = false, source_position = { 0, 100, 0 }, target_position = { 1000, 100, 0 } }
  for key, value in pairs(extra or {}) do step[key] = value end
  return host.line_of_sight(step)
end
local function refused(step, text)
  local ok, err = pcall(host.line_of_sight, step)
  assert(not ok and tostring(err):find(text, 1, true), text .. ' / ' .. tostring(err))
end
-- a clear ray: four coarse samples, asked one at a time, then the visibility
local r = begin()
assert(r.state == 'need_terrain' and r.phase == 1 and math.abs(r.point[1] - 200) < 0.01 and math.abs(r.point[2] - 100) < 0.01, 'first sample')
for i = 2, 4 do
  r = host.line_of_sight({ mode = 'terrain', elevation = 0 })
  assert(r.state == 'need_terrain' and math.abs(r.point[1] - 200 * i) < 0.01, 'sample ' .. i)
end
r = host.line_of_sight({ mode = 'terrain', elevation = 0 })
assert(r.state == 'done' and r.visible == true, 'clear')
-- a blocked ray ends at the first blocker
r = begin()
r = host.line_of_sight({ mode = 'terrain', elevation = 150 })
assert(r.state == 'done' and r.visible == false, 'blocked at the first sample')
-- identities pass through by shape only: a source and a target id, and none
r = begin({ criteria = 0, source_host_id = 'eech-world:1', target_host_id = 'eech-world:2' })
assert(r.state == 'done' and r.visible == true)
refused({ mode = 'begin', entity_form = false, criteria = 0, source_eye = false, target_eye = false, source_host_id = string.rep('x', 129), source_position = { 0, 0, 0 }, target_position = { 1, 0, 0 } }, 'source_host_id must be 1 to 128 bytes')
refused({ mode = 'begin', entity_form = false, criteria = 0, source_eye = false, target_eye = false, target_host_id = '', source_position = { 0, 0, 0 }, target_position = { 1, 0, 0 } }, 'target_host_id must be 1 to 128 bytes')
refused({ mode = 'begin', entity_form = false, criteria = 0, source_eye = false, target_eye = false, source_host_id = 5, source_position = { 0, 0, 0 }, target_position = { 1, 0, 0 } }, 'source_host_id must be a string')
-- malformed requests
refused({ mode = 'begin', entity_form = false, criteria = 32, source_eye = false, target_eye = false, source_position = { 0, 0, 0 }, target_position = { 1, 0, 0 } }, 'outside the five check bits')
refused({ mode = 'begin', entity_form = false, criteria = 1, source_eye = 1, target_eye = false, source_position = { 0, 0, 0 }, target_position = { 1, 0, 0 } }, 'source_eye must be a boolean')
refused({ mode = 'begin', entity_form = 'no', criteria = 1, source_eye = false, target_eye = false, source_position = { 0, 0, 0 }, target_position = { 1, 0, 0 } }, 'entity_form must be a boolean')
refused({ mode = 'begin', entity_form = false, criteria = 1, source_eye = false, target_eye = false, source_position = { 0, 0 }, target_position = { 1, 0, 0 } }, 'source_position must be 3 finite numbers')
refused({ mode = 'begin', entity_form = false, criteria = 1, source_eye = false, target_eye = false, source_position = { 0, 0, 0 / 0 }, target_position = { 1, 0, 0 } }, 'source_position must be 3 finite numbers')
refused({ mode = 'begin', entity_form = false, criteria = 1.5, source_eye = false, target_eye = false, source_position = { 0, 0, 0 }, target_position = { 1, 0, 0 } }, 'criteria must be an integer')
refused({ mode = 'glide' }, 'unknown mode')
-- answers that were not asked for
begin()
refused({ mode = 'objects', hit = false }, 'none was asked for')
refused({ mode = 'terrain', elevation = 1 / 0 }, 'elevation must be')
r = host.line_of_sight({ mode = 'terrain', elevation = 0 })
assert(r.state == 'need_terrain' and math.abs(r.point[1] - 400) < 0.01, 'the refused answers changed nothing')
r = begin({ criteria = 0 })
assert(r.state == 'done')
refused({ mode = 'terrain', elevation = 0 }, 'no query open')
refused({ mode = 'objects', hit = false }, 'no query open')
-- object rays: a hit needs its point, a miss carries none, and the 12 m exclusion is the host's
r = begin({ criteria = 16, target_position = { 1000, 0, 0 }, source_position = { 0, 0, 0 } })
assert(r.state == 'need_objects' and r.phase == 5 and math.abs(r.from[1] - 1000) < 0.01 and math.abs(r.to[1] - 800) < 0.01, 'target-end ray')
refused({ mode = 'objects', hit = true }, 'needs its collision point')
refused({ mode = 'objects', hit = false, collision_point = { 0, 0, 0 } }, 'must not carry a collision point')
refused({ mode = 'objects', hit = 'yes' }, 'hit must be a boolean')
r = host.line_of_sight({ mode = 'objects', hit = true, collision_point = { 988, 0, 0 } })
assert(r.state == 'done' and r.visible == true, 'a hit exactly 12 m from the target is excluded')
r = begin({ criteria = 16, target_position = { 1000, 0, 0 }, source_position = { 0, 0, 0 } })
r = host.line_of_sight({ mode = 'objects', hit = true, collision_point = { 987, 0, 0 } })
assert(r.state == 'done' and r.visible == false, 'a hit 13 m from the target blocks')
print('LINE_OF_SIGHT_HOST=ok')
";

#[test]
fn line_of_sight_round_trips_through_lua_and_refuses_what_it_was_not_asked() {
    let script = std::env::temp_dir().join(format!("eech-world-line-of-sight-{}.lua", std::process::id()));
    std::fs::write(&script, LINE_OF_SIGHT_SCRIPT).unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_eech-world")).arg(&script).output();
    std::fs::remove_file(&script).unwrap();
    let run = run.unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run.status.success(), "{}\n{stdout}\n{stderr}", run.status);
    assert!(stdout.contains("LINE_OF_SIGHT_HOST=ok"), "{stdout}\n{stderr}");
}

const SENSE_SCRIPT: &str = "local function refused(step, text)
  local ok, err = pcall(host.sense, step)
  assert(not ok and tostring(err):find(text, 1, true), text .. ' / ' .. tostring(err))
end
local SRC = { 1000, 100, 2000 }
-- a surface-to-air scan: the floor, the draw, the signature, the end, over the Lua boundary
local r = host.sense({ mode = 'begin', scan = 'surface_to_air', source_position = SRC, scan_range = 6000, scan_ceiling = 6000, scan_floor = 18, source_host_id = 'eech-world:1' })
assert(r.state == 'open' and r.echo and r.echo.ceiling == 6100, 'begin')
r = host.sense({ mode = 'candidate', scan = 'surface_to_air', target_position = { 4000, 100, 2000 }, target_player = false, radar_altitude = 15, target_host_id = 'eech-world:2' })
assert(r.state == 'need_draw' and r.target_range == 3000 and r.echo.floor == 9, 'the floor is lowered by the half-range candidate')
r = host.sense({ mode = 'draw', draw = 0.0 })
assert(r.state == 'need_signature' and r.echo.tolerance_limit == 9, 'the limit is floor * (1 + 0.5 * draw)')
r = host.sense({ mode = 'signature', scan = 'surface_to_air', helicopter = { weapon_bay_closed = false, stub_wings_attached = true, undercarriage_up = false } })
assert(r.state == 'accept' and r.target_range == 3000 and r.echo.stealth_factor == 1, 'accepted')
-- the next candidate meets the lowered floor
r = host.sense({ mode = 'candidate', scan = 'surface_to_air', target_position = { 7000, 100, 2000 }, target_player = false, radar_altitude = 8 })
assert(r.state == 'reject' and r.reason == 'altitude' and r.echo.floor == 9, 'the floor stayed lowered')
r = host.sense({ mode = 'end' })
assert(r.state == 'closed', 'end')
-- an aircraft scan rejects with its reason
host.sense({ mode = 'begin', scan = 'air_to_surface', source_position = SRC, scan_range = 4000 })
r = host.sense({ mode = 'candidate', scan = 'air_to_surface', target_position = { 5001, 0, 2000 }, target_player = false })
assert(r.state == 'reject' and r.reason == 'range' and r.target_range > 16000000, 'out of range')
host.sense({ mode = 'end' })
-- malformed messages
refused({ mode = 'glide' }, 'unknown mode')
refused({ mode = 'begin', scan = 'sideways', source_position = SRC, scan_range = 1 }, 'unknown scan')
refused({ mode = 'begin', scan = 'air_to_air', source_position = { 0, 0 }, scan_range = 1 }, 'source_position must be 3 finite numbers')
refused({ mode = 'begin', scan = 'air_to_air', source_position = SRC, scan_range = 0 / 0 }, 'scan_range must be a finite number')
refused({ mode = 'begin', scan = 'air_to_air', source_position = SRC, scan_range = 1, source_host_id = '' }, 'source_host_id must be 1 to 128 bytes')
host.sense({ mode = 'begin', scan = 'surface_to_air', source_position = SRC, scan_range = 6000, scan_ceiling = 6000, scan_floor = 18 })
refused({ mode = 'candidate', scan = 'surface_to_air', target_position = SRC, target_player = 1, radar_altitude = 1 }, 'target_player must be a boolean')
refused({ mode = 'candidate', scan = 'surface_to_air', target_position = SRC, target_player = true, radar_altitude = 1 }, 'difficulty factor')
refused({ mode = 'candidate', scan = 'surface_to_air', target_position = SRC, target_player = false, radar_altitude = 1 / 0 }, 'radar_altitude must be a finite number')
refused({ mode = 'candidate', scan = 'air_to_air', target_position = SRC, target_player = false }, 'arrived in a scan of kind')
refused({ mode = 'draw', draw = 0.5 }, 'none was asked for')
refused({ mode = 'signature', scan = 'surface_to_air' }, 'none was asked for')
host.sense({ mode = 'candidate', scan = 'surface_to_air', target_position = { 7000, 100, 2000 }, target_player = false, radar_altitude = 25 })
refused({ mode = 'draw', draw = 1.5 }, 'draw')
refused({ mode = 'signature', scan = 'surface_to_air', emitters = { radar_on = 1 } }, 'radar_on must be a boolean')
r = host.sense({ mode = 'draw', draw = 1.0 })
assert(r.state == 'reject' and r.reason == 'tolerance', 'the refused answers changed nothing')
host.sense({ mode = 'end' })
refused({ mode = 'end' }, 'none open')
print('SENSE_HOST=ok')
";

#[test]
fn sense_round_trips_through_lua_and_refuses_what_it_was_not_asked() {
    let script = std::env::temp_dir().join(format!("eech-world-sense-{}.lua", std::process::id()));
    std::fs::write(&script, SENSE_SCRIPT).unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_eech-world")).arg(&script).output();
    std::fs::remove_file(&script).unwrap();
    let run = run.unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run.status.success(), "{}\n{stdout}\n{stderr}", run.status);
    assert!(stdout.contains("SENSE_HOST=ok"), "{stdout}\n{stderr}");
}

const AIM_SCRIPT: &str = "local function refused(step, text)
  local ok, err = pcall(host.aim, step)
  assert(not ok and tostring(err):find(text, 1, true), text .. ' / ' .. tostring(err))
end
local function near(a, b) return math.abs(a - b) < 1e-6 end
-- the weapon-system-ready keyframe: closed, commanded open, opening, open, commanded closed, closing, wrapped
local r = host.aim({ mode = 'ready_command', source_host_id = 'eech-world:1', state = 0.0, open = true })
assert(r.state == 'done' and r.ready_state == 1.0, 'commanded open from closed starts the opening')
r = host.aim({ mode = 'ready_update', state = 1.0, rate = 0.5, delta_time = 0.5 })
assert(r.state == 'done' and r.ready_state == 1.25 and r.phase == 1, 'opening advances by rate * time')
r = host.aim({ mode = 'ready_update', state = 1.75, rate = 0.5, delta_time = 1.0 })
assert(r.ready_state == 2.0 and r.phase == 1, 'opening ends at open')
r = host.aim({ mode = 'ready_update', state = 2.0, rate = 0.5, delta_time = 1.0 })
assert(r.ready_state == 2.0 and r.phase == 2, 'open holds')
r = host.aim({ mode = 'ready_command', state = 2.0, open = false })
assert(r.ready_state == 3.0, 'commanded closed from open starts the closing')
r = host.aim({ mode = 'ready_update', state = 3.5, rate = 0.5, delta_time = 2.0 })
assert(r.ready_state == 0.0 and r.phase == 3, 'closing wraps to closed')
r = host.aim({ mode = 'ready_update', state = 0.0, rate = 0.5, delta_time = 1.0 })
assert(r.ready_state == 0.0 and r.phase == 0, 'closed holds')
-- the device step: nothing to reach, nothing taken
r = host.aim({ mode = 'step', current_heading = 0.25, current_pitch = -0.1, required_heading = 0.25, required_pitch = -0.1, heading_min = -1, heading_max = 1, heading_rate = 1, pitch_rate = 1, delta_time = 0.1 })
assert(r.state == 'done' and r.delta_heading == 0 and r.delta_pitch == 0 and r.new_heading == 0.25 and near(r.new_pitch, -0.1), 'a device at its required offsets stays')
-- the offsets with no package limit (the HUD's) and of a gun aimed at a tracking point (the player's cannon tracking)
local LEVEL = { 1, 0, 0, 0, 1, 0, 0, 0, 1 }
r = host.aim({ mode = 'required', device_attitude = LEVEL, intercept_vector = { 1, 0, 0 }, unbounded = true })
assert(r.state == 'done' and near(r.required_heading, math.pi / 2) and near(r.required_pitch, 0), 'unbounded offsets are the bearing as it is')
r = host.aim({ mode = 'required', device_attitude = LEVEL, tracking = { device_position = { 0, 0, 0 }, point = { 100, 7, 0 }, pitch = 0 }, unbounded = true })
assert(near(r.required_heading, math.pi / 2) and near(r.required_pitch, 0), 'a tracking point is aimed at in the horizontal plane, lowered by the ballistic pitch')
refused({ mode = 'required', device_attitude = LEVEL, tracking = { device_position = { 0, 0, 0 }, point = { 1, 0, 1 }, pitch = 0 }, heading_min = -1, heading_max = 1, pitch_min = -1, pitch_max = 1, climb = false }, 'tracking request has no package limits')
refused({ mode = 'required', device_attitude = LEVEL, intercept_vector = { 0, 0, 1 }, unbounded = 1 }, 'unbounded must be a boolean')
-- the host generates its own ballistic table from the weapon's figures (no cell is sent), and a request for a weapon without one is refused
local function gun(name) return { name = name, start_weight = 0.1, final_weight = 0.1, max_range = 1000, muzzle_velocity = 900, cruise_velocity = 900, g_max = 0, boost_time = 0, cruise_time = 4, inhibit_time = 0, drag_coefficient = 0.3, flightpath_deviation = 0, boost_power = 0, sustain_power = 0, sustain_time = 0, diameter = 0.02, max_altitude = 0, ignore_gravity = false, flight_profile_or_self_destr = 0, parachute_type = false, weapon_class = 8, cruise_time_max_error = 0, detonation_radius = 0, max_seeker_limit = 0, max_launch_angle_error = 0, survive_ground_impact = 0, in_flight_collision_check = 0, guidance_type = 0, aiming_type = 3, decoy_type = 0, warhead_type = 3, hellfire_ii = 0, smoke_grenade = 0, submunition_count = 0, submunition_velocity_factor = 0, muzzle_velocity_max_error = 0, circular_error_probable = 0, rate_of_fire = 0, inverse_rate_of_fire = 0, acquire_forward = 0 } end
r = host.aim({ mode = 'table', weapon = 7, figures = gun('test gun'), first = false, last = false, echo = false, echo_cells = 0 })
assert(r.state == 'done' and r.echo_cells == nil and r.echo == nil, 'the table is generated and nothing comes back')
-- with an echo (the test fixture build's) the host gives back its flights and the cells of the table it generated, in the model's shape
r = host.aim({ mode = 'table', weapon = 9, figures = gun('echo gun'), first = false, last = false, echo = true, echo_cells = 5 * 85 * 52 * 2 })
assert(r.state == 'done' and #r.echo_cells == 5 * 85 * 52 * 2 * 4, 'the echo carries every cell')
assert(#r.echo == 83 and r.echo[1].kind == 'table_flight' and r.echo[1].a == 0 and r.echo[1].d > 0 and r.echo[1].f[2] == 52, 'one record per simulated flight')
-- the lead's flight time is a partial output: sent when the prediction produced one, absent when it did not (it is never invented)
local MAP = { map_min = { 0, 0, 0 }, map_max = { 100000, 100000, 100000 }, comms_weapon_lag = 0, source_gunship = false, echo = false }
local function lead(device, point, parent_velocity)
  local request = { mode = 'lead', weapon = 7, device_position = device, parent_velocity = parent_velocity, target = { position = point, point = point, motion_vector = { 0, 0, 0 }, mobile = true } }
  for k, v in pairs(MAP) do request[k] = v end
  return host.aim(request)
end
r = lead({ 0, 0, 0 }, { 500, 0, 0 }, 0)
assert(r.state == 'done' and r.ok == true and type(r.time_of_flight) == 'number', 'a ballistic lookup produced a flight time')
r = lead({ 198500, 0, 0 }, { 200000, 0, 0 }, 500)
assert(r.state == 'done' and r.ok == true and r.time_of_flight == nil, 'the off-map success produces no flight time: it is absent, not zero')
r = lead({ 10, 0, 10 }, { 10, 0, 10 }, 0)
assert(r.state == 'done' and r.ok == false and r.time_of_flight == nil, 'a tiny range produces none')
refused({ mode = 'ballistic', weapon = 8, range = 50, height_diff_or_pitch = 0, simplified = false, fixed_pitch = false, parent_velocity = 0 }, 'no ballistic table for weapon 8')
refused({ mode = 'table', weapon = 7, first = false, last = false, echo = false }, 'figures must be a table')
refused({ mode = 'table', weapon = 7, figures = gun(''), first = false, last = false, echo = false }, 'weapon name must be 1 to 63 bytes')
refused({ mode = 'table', weapon = 0, figures = gun('test gun'), first = false, last = false, echo = false }, 'not a weapon')
refused({ mode = 'table', weapon = 7, figures = gun('test gun'), first = 1, last = false, echo = false }, 'first must be a boolean')
local rangeless = gun('rangeless gun') rangeless.max_range = 0
refused({ mode = 'table', weapon = 7, figures = rangeless, first = false, last = false, echo = false }, 'range limit')
local slow = gun('infinite gun') slow.drag_coefficient = 1 / 0
refused({ mode = 'table', weapon = 7, figures = slow, first = false, last = false, echo = false }, 'drag_coefficient')
-- the host keeps every weapon's figures and flies a weapon by them (the weapon step): a pass that needs the campaign's world stops and changes nothing, and is asked
-- again with the answers; it ends with the weapon's state, attitude and position as ordered actions
-- (a weapon that lives has a motor: the original's weight is defined for non-zero boost and sustain times)
local flown = gun('flown gun')
flown.start_weight, flown.final_weight, flown.diameter, flown.boost_time, flown.sustain_time, flown.boost_power, flown.sustain_power = 5, 5, 0.1, 1, 2, 1, 1
r = host.aim({ mode = 'figures', weapon = 21, figures = flown })
assert(r.state == 'done', 'the figures are kept')
local function weapon_step(weapon)
  return { mode = 'weapon_step', weapon = weapon, echo = false, delta_time = 0.1, system_time = 1000, position = { 0, 500, 0 }, attitude = LEVEL, motion_vector = { 0, 0, 0 }, velocity = 900, weapon_lifetime = 3.0, decoy_timer = 0,
    missile_phase = 0, loal_mode = false, weapon_on_ground = false, parachute_status = 0, kill_code = 0, entity_samples = { 100, 200, 300, 400 }, has_launcher = false, has_target = false, server = true,
    map_min = { -100000, -1000, -100000 }, map_max = { 100000, 10000, 100000 }, answers = {}, draws = {} }
end
local function fly(step, answer)
  local answers, draws = {}, {}
  for pass = 1, 20 do
    local request = {}
    for k, v in pairs(step) do request[k] = v end
    request.answers, request.draws = answers, draws
    local out = host.aim(request)
    if out.state == 'done' then return out end
    assert(out.state == 'need_lookup' and out.actions == nil and #out.lookups > 0, 'a step that is not done leaves no action: ' .. tostring(out.state))
    for _, lookup in ipairs(out.lookups) do
      local a = { kind = lookup.kind, a = lookup.a, b = lookup.b, c = lookup.c, d = lookup.d, f = { unpack(lookup.f) } }
      for i = 7, 12 do a.f[i] = 0 end
      answers[#answers + 1] = answer(a) or a
    end
  end
  error('the step did not finish')
end
local function last(out, kind) local found for _, action in ipairs(out.actions) do if action.kind == kind then found = action end end return found end
r = host.aim(weapon_step(21))
assert(r.state == 'need_lookup' and r.actions == nil and #r.lookups >= 1, 'with nothing answered the step stops for lookups and leaves nothing')
local out = fly(weapon_step(21), function() end)
local state, position = last(out, 1), last(out, 5)
assert(state.c == 0 and near(state.f[5], 2.9) and state.f[1] < 900 and state.f[1] > 800, 'a frame of free flight: no kill code, the lifetime run down by the frame, the speed drag-reduced')
assert(near(position.f[1], 0) and near(position.f[2], 500) and position.f[3] > 80 and position.f[3] < 90, 'and the position moved along the heading')
-- the ground the campaign answers for decides the contact: land ends the weapon where the impact lookup says
out = fly(weapon_step(21), function(a)
  if a.kind == 8 then a.f[7] = 1 elseif a.kind == 9 then a.f[7], a.f[8], a.f[9], a.f[10] = 3, 1, 2, 3 end
end)
state, position = last(out, 1), last(out, 5)
assert(state.c == 3 and near(position.f[1], 1) and near(position.f[2], 2) and near(position.f[3], 3), 'a weapon over land ends at hit-land at the impact point')
out = fly(weapon_step(21), function(a)
  if a.kind == 8 then a.f[7] = 1 elseif a.kind == 9 then a.f[7], a.f[8], a.f[9], a.f[10] = 1, 1, 2, 3 end
end)
assert(last(out, 1).c == 4, 'and over water at hit-water')
-- a weapon whose lifetime has run out is over before it moves, and asks the campaign nothing
local expired = weapon_step(21) expired.weapon_lifetime = -4.0
out = host.aim(expired)
assert(out.state == 'done' and last(out, 1).c == 1 and last(out, 5) == nil and last(out, 4) == nil, 'an exhausted weapon leaves its state and no position')
refused(weapon_step(22), 'holds no figures')
-- the birth of a weapon: the velocity it leaves with, its lifetime and its attitude, from the pose the launcher let it go in
local born = gun('born gun')
born.start_weight, born.final_weight, born.diameter, born.boost_time, born.sustain_time, born.boost_power, born.sustain_power, born.muzzle_velocity, born.acquire_forward = 5, 5, 0.1, 1, 2, 1, 1, 900, 1
host.aim({ mode = 'figures', weapon = 23, figures = born })
local function birth(weapon) local request = weapon_step(weapon) request.mode = 'weapon_birth' request.submunition = false return request end
out = fly(birth(23), function(a) if a.kind == 14 then a.f[7] = 100 end end)
local started = last(out, 8)
assert(near(started.f[1], 1000) and started.f[2] == 3 and started.f[3] == 0, 'a weapon leaves with its launcher velocity and its muzzle velocity, for its motor lifetime')
assert(last(out, 2).f[5] == 1 and last(out, 2).f[9] == 1 and last(out, 2).f[1] == 1, 'at the attitude the launcher let it go in')
refused(birth(24), 'holds no figures')
bad = birth(23) bad.submunition = 1
refused(bad, 'submunition must be a boolean')
-- the release of a weapon: the rate-of-fire gate of a launcher
local cannon = gun('rated gun') cannon.rate_of_fire, cannon.inverse_rate_of_fire = 600, 1 / 600
host.aim({ mode = 'figures', weapon = 25, figures = cannon })
r = host.aim({ mode = 'release', weapon = 25, delta_time = 0.1, current_time = 1000, last_shot = 0, not_ai = false })
assert(r.state == 'done' and r.refuse == false and r.burst_size == 1 and r.last_shot == 1000, 'the first shot goes and starts the launcher clock')
r = host.aim({ mode = 'release', weapon = 25, delta_time = 0.1, current_time = 1050, last_shot = 1000, not_ai = false })
assert(r.refuse == true and r.last_shot == nil, 'too soon: nothing goes and nothing is written')
r = host.aim({ mode = 'release', weapon = 25, delta_time = 0.1, current_time = 1300, last_shot = 1000, not_ai = false })
assert(r.refuse == false and r.burst_size == 3 and r.last_shot == 1300, 'three rounds owed go together')
r = host.aim({ mode = 'release', weapon = 25, delta_time = 0.1, current_time = 1300, not_ai = false })
assert(r.refuse == false and r.burst_size == 1 and r.last_shot == nil, 'a launcher with no clock lets one round go')
refused({ mode = 'release', weapon = 26, delta_time = 0.1, current_time = 1000, not_ai = false }, 'holds no figures')
refused({ mode = 'release', weapon = 25, delta_time = 0.1, current_time = 1000, not_ai = 1 }, 'not_ai must be a boolean')
local bad = weapon_step(21) bad.entity_samples = { 1, 2, 3 }
refused(bad, 'entity_samples must be 4 integers')
bad = weapon_step(21) bad.position = { 0, 0 / 0, 0 }
refused(bad, 'position')
bad = weapon_step(21) bad.draws = { 40000 }
refused(bad, 'outside [0, 32767]')
bad = weapon_step(21) bad.answers = { { kind = 8, a = 0, b = 0, c = 0, d = 0, f = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 } } }
refused(bad, 'f must be 12')
refused({ mode = 'figures', weapon = 0, figures = gun('no weapon') }, 'not a weapon')
-- malformed requests
refused({ mode = 'glide' }, 'unknown mode')
refused({ mode = 'ready_update', state = 0 / 0, rate = 0.5, delta_time = 1 }, 'state must be a finite number')
refused({ mode = 'ready_command', state = 0, open = 1 }, 'open must be a boolean')
refused({ mode = 'ready_command', source_host_id = '', state = 0, open = true }, 'source_host_id must be 1 to 128 bytes')
refused({ mode = 'required', device_attitude = { 1, 0, 0 }, intercept_vector = { 0, 0, 1 }, heading_min = -1, heading_max = 1, pitch_min = -1, pitch_max = 1, climb = false }, 'device_attitude must be 9 finite numbers')
refused({ mode = 'lead', weapon = 7, device_position = { 0, 0, 0 }, parent_velocity = 0, target = { position = { 0, 0, 0 }, point = { 0, 0, 0 }, motion_vector = { 0, 0, 0 }, mobile = false }, map_min = { 0, 0, 0 }, map_max = { 1, 1, 1 }, comms_weapon_lag = 0, source_gunship = false, draw = 40000 }, 'draw must be an integer in [0, 32767]')
print('AIM_HOST=ok')
";

#[test]
fn aim_round_trips_through_lua_and_refuses_what_it_was_not_asked() {
    let script = std::env::temp_dir().join(format!("eech-world-aim-{}.lua", std::process::id()));
    std::fs::write(&script, AIM_SCRIPT).unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_eech-world")).arg(&script).output();
    std::fs::remove_file(&script).unwrap();
    let run = run.unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run.status.success(), "{}\n{stdout}\n{stderr}", run.status);
    assert!(stdout.contains("AIM_HOST=ok"), "{stdout}\n{stderr}");
}

#[test]
fn every_shipped_lua_script_parses_under_the_hosts_lua() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("lua");
    let lua = mlua::Lua::new();
    let mut parsed = 0;
    for entry in std::fs::read_dir(&directory).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|extension| extension == "lua") {
            let source = std::fs::read(&path).unwrap();
            lua.load(&source[..])
                .set_name(path.display().to_string())
                .into_function()
                .unwrap_or_else(|error| panic!("{} does not parse: {error}", path.display()));
            parsed += 1;
        }
    }
    assert!(parsed >= 20, "only {parsed} scripts were found in {}", directory.display());
}
