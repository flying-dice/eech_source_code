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
local function gun(name) return { name = name, start_weight = 0.1, final_weight = 0.1, max_range = 1000, muzzle_velocity = 900, cruise_velocity = 900, g_max = 0, boost_time = 0, cruise_time = 4, inhibit_time = 0, drag_coefficient = 0.3, flightpath_deviation = 0, boost_power = 0, sustain_power = 0, sustain_time = 0, diameter = 0.02, max_altitude = 0, ignore_gravity = false, flight_profile_or_self_destr = 0, parachute_type = 0, weapon_class = 8, cruise_time_max_error = 0, detonation_radius = 0, max_seeker_limit = 0, max_launch_angle_error = 0, survive_ground_impact = 0, in_flight_collision_check = 0, guidance_type = 0, aiming_type = 3, decoy_type = 0, warhead_type = 3, hellfire_ii = 0, smoke_grenade = 0, submunition_count = 0, submunition_velocity_factor = 0, muzzle_velocity_max_error = 0, circular_error_probable = 0, rate_of_fire = 0, inverse_rate_of_fire = 0, acquire_forward = 0 } end
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
-- after the flight: a crate's parachute, a decoy's timer and a seeker's seduction
local crate = gun('crate') crate.parachute_type = 2
crate.start_weight, crate.final_weight, crate.boost_time, crate.sustain_time, crate.boost_power, crate.sustain_power = 5, 5, 1, 2, 1, 1
host.aim({ mode = 'figures', weapon = 27, figures = crate })
r = host.aim({ mode = 'animate', weapon = 27, parachute_status = 0, weapon_on_ground = false, weapon_lifetime = -0.6, motion_vector_y = -3.0 })
assert(r.state == 'done' and r.parachute_status == 1 and near(r.weapon_lifetime, 0), 'a falling crate opens its parachute and resets its lifetime')
r = host.aim({ mode = 'animate', weapon = 27, parachute_status = 3, weapon_on_ground = true, weapon_lifetime = -9, motion_vector_y = 0 })
assert(r.parachute_status == 4 and r.weapon_lifetime == nil, 'on the ground it collapses and nothing else is written')
refused({ mode = 'animate', weapon = 27, parachute_status = 9, weapon_on_ground = false, weapon_lifetime = 0, motion_vector_y = 0 }, 'parachute state')
refused({ mode = 'animate', weapon = 28, parachute_status = 0, weapon_on_ground = false, weapon_lifetime = 0, motion_vector_y = 0 }, 'holds no figures')
local chaff = gun('chaff') chaff.decoy_type = 1
local seeker = gun('seeker') seeker.guidance_type = 2
host.aim({ mode = 'figures', weapon = 29, figures = chaff })
host.aim({ mode = 'figures', weapon = 30, figures = seeker })
local function decoy_step(timer) local request = weapon_step(29) request.mode = 'weapon_decoy' request.decoy_timer = timer request.delta_time = 0.1 return request end
r = host.aim(decoy_step(0.5))
assert(r.state == 'done' and r.actions[1].kind == 9 and r.actions[1].a == 0 and near(r.actions[1].f[1], 0.4), 'a decoy running down takes no sample')
assert(host.aim(decoy_step(0.05)).state == 'need_draw', 'and when it runs out it asks for one')
local drawn = decoy_step(0.05) drawn.draws = { 32767 }
r = host.aim(drawn)
assert(r.state == 'done' and r.actions[1].a == 1 and near(r.actions[1].f[1], 0.6), 'the restart is 0.4 plus a fifth of the sample')
local function seduce_step(sample) local request = weapon_step(30) request.mode = 'weapon_seduce' request.other_weapon = 29 request.coefficient = 1.0 request.effectiveness = { 0.5, 0.5, 1.0 } request.draws = sample and { sample } or {} return request end
assert(host.aim(seduce_step(nil)).state == 'need_draw', 'a seeker asked about a decoy takes a sample')
r = host.aim(seduce_step(1000))
assert(r.state == 'done' and r.actions[1].kind == 10 and r.actions[1].a == 1 and r.actions[1].b == 1, 'a radar seeker drawn to chaff goes for it')
r = host.aim(seduce_step(20000))
assert(r.actions[1].a == 0 and r.actions[1].b == 0, 'a sample over the chance does not')
bad = seduce_step(1000) bad.effectiveness = { 0.5, 0.5 }
refused(bad, 'effectiveness must be 3 finite numbers')
bad = seduce_step(1000) bad.other_weapon = 31
refused(bad, 'holds no figures')
local function reacquire_step() local request = weapon_step(30) request.mode = 'weapon_reacquire' request.other_weapon = 29 return request end
r = host.aim(reacquire_step())
assert(r.state == 'done' and #r.actions == 1 and r.actions[1].kind == 11 and r.actions[1].a == 1 and r.actions[1].b == 0, 'a seeker whose decoy expired goes back to the launcher')
local sampled = reacquire_step() sampled.draws = { 5 }
assert(host.aim(sampled).state == 'done', 'and the answer takes no sample')
bad = reacquire_step() bad.other_weapon = 31
refused(bad, 'holds no figures')
bad = reacquire_step() bad.other_weapon = nil
refused(bad, 'other_weapon must be an integer')
local own_model = dofile('@LUA_DIR@/own-weapon.lua')
r = own_model.weapon_reacquire(reacquire_step())
assert(r.state == 'done' and #r.actions == 1 and r.actions[1].kind == 11 and r.actions[1].a == 0, 'a generic host may tell the seeker to drop the lock')
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
-- a host with a model of its own is not made to know its own question by an exactness it does not have: the module carries the arguments of a lookup as single
-- precision, so the model asks about points on a grid that single precision carries exactly, and recognises the campaign's answer to them
local own = dofile('@LUA_DIR@/own-weapon.lua')
local function carried(x)
  -- a number as the module carries it, through the host's own conversion: the wind lookup of a step is asked at the weapon's position
  local probe = weapon_step(21) probe.position = { x, 500, 0 }
  for _, lookup in ipairs(host.aim(probe).lookups) do if lookup.kind == 7 then return lookup.f[1] end end
  error('no wind lookup')
end
local dt = carried(0.1)
assert(dt ~= 0.1, 'a tenth is not a single-precision number')
local moving = weapon_step(21) moving.position = { carried(1000), carried(500), 0 } moving.delta_time = dt moving.velocity = 300 moving.attitude = LEVEL
local asked = own.weapon_step(moving)
assert(asked.state == 'need_lookup' and #asked.lookups == 1, 'the model asks for the ground once')
local ground = asked.lookups[1]
for i = 1, 3 do assert(carried(ground.f[i]) == ground.f[i], 'what the model asks is what the module carries: ' .. tostring(ground.f[i])) end
moving.answers = { { kind = ground.kind, a = 0, b = 0, c = 0, d = 0, f = { carried(ground.f[1]), carried(ground.f[2]), carried(ground.f[3]), 0, 0, 0, 0, 0, 0, 0, 0, 0 } } }
assert(own.weapon_step(moving).state == 'done', 'and the model recognises the answer to it')
-- the malformed providers: the overdraw provider asks for a sample at every pass of a weapon step (a legal draw is a continuation, not a refusal), the others corrupt the
-- first answer of their kind once and let the reference host answer the rest
local malformed = dofile('@LUA_DIR@/aim-malformed.lua')
local passed = function(_, answer) return answer end
local overdraw = malformed('bad_step_draw', host.aim, passed)
for n = 0, 12 do
  local request = weapon_step(21) request.draws = {} for i = 1, n do request.draws[i] = 100 end
  assert(overdraw(request).state == 'need_draw', 'the overdraw provider never settles: ' .. n .. ' samples given')
end
local once = malformed('bad_step_state', host.aim, passed)
assert(once(weapon_step(21)).state == 'later', 'the first weapon step is corrupted')
assert(once(weapon_step(21)).state == 'need_lookup', 'and the reference host answers the rest')
assert(malformed('bad_step_state', host.aim, passed)({ mode = 'ready_update', state = 1.0, rate = 0.5, delta_time = 0.5 }).state == 'done', 'a request of another kind is answered as the reference does')
-- the decision providers corrupt the first decision of their kind, and only a decision (a seeker that has yet to draw a sample is not corrupted)
local flagged = malformed('bad_decoy_flag', host.aim, passed)(decoy_step(0.5))
assert(flagged.state == 'done' and flagged.actions[1].kind == 9 and flagged.actions[1].a == 2, 'a decoy decision that is neither 0 nor 1')
local seduced = malformed('bad_seduce_flag', host.aim, passed)
assert(seduced(seduce_step(nil)).state == 'need_draw', 'a seduction that asks for its sample is not yet a decision')
flagged = seduced(seduce_step(1000))
assert(flagged.state == 'done' and flagged.actions[1].kind == 10 and flagged.actions[1].b == 3, 'a seduction decision that is neither 0 nor 1')
flagged = malformed('bad_reacquire_flag', host.aim, passed)(reacquire_step())
assert(flagged.state == 'done' and flagged.actions[1].kind == 11 and flagged.actions[1].a == -1, 'a reacquisition decision that is neither 0 nor 1')
flagged = malformed('bad_reacquire_none', host.aim, passed)(reacquire_step())
assert(flagged.state == 'done' and #flagged.actions == 0, 'a reacquisition with no decision')
local reacquirer = malformed('bad_reacquire_flag', host.aim, passed)
assert(reacquirer(weapon_step(21)).state == 'need_lookup', 'a request of another kind is answered as the reference does')
assert(reacquirer(reacquire_step()).actions[1].a == -1 and reacquirer(reacquire_step()).actions[1].a == 1, 'the first decision only')
-- malformed requests
refused({ mode = 'glide' }, 'unknown mode')
refused({ mode = 'ready_update', state = 0 / 0, rate = 0.5, delta_time = 1 }, 'state must be a finite number')
refused({ mode = 'ready_command', state = 0, open = 1 }, 'open must be a boolean')
refused({ mode = 'ready_command', source_host_id = '', state = 0, open = true }, 'source_host_id must be 1 to 128 bytes')
refused({ mode = 'required', device_attitude = { 1, 0, 0 }, intercept_vector = { 0, 0, 1 }, heading_min = -1, heading_max = 1, pitch_min = -1, pitch_max = 1, climb = false }, 'device_attitude must be 9 finite numbers')
refused({ mode = 'lead', weapon = 7, device_position = { 0, 0, 0 }, parent_velocity = 0, target = { position = { 0, 0, 0 }, point = { 0, 0, 0 }, motion_vector = { 0, 0, 0 }, mobile = false }, map_min = { 0, 0, 0 }, map_max = { 1, 1, 1 }, comms_weapon_lag = 0, source_gunship = false, draw = 40000 }, 'draw must be an integer in [0, 32767]')
-- the damage forecast (EEDCS-46): a pure question answered from the figures it is sent, with no draw.  The expected values are worked out by hand from the original
-- expressions of weapon_damage_capability (the `wpn == NULL` path); the kernel's own tests are in src/cmb.rs
local function forecast(over)
  local request = { mode = 'forecast', weapon = 54, weapon_warhead = 5, warhead = 5, damage_capability = 100, armor_penetration_capability = 1, burst_duration = 0.0,
    armour_level = 4, target_not_ai = false, player_modifier = 1.0, damage_modifier = 0.5, velocity = 1.0, echo = false }
  for k, v in pairs(over or {}) do request[k] = v end
  return request
end
r = host.aim(forecast())
assert(r.state == 'done' and r.capability == 31 and r.echo == nil, 'dual-purpose splash against armour 4: 100 * (1 - 0.5^((2/3)^1.5)) = 31.4')
r = host.aim(forecast({ echo = true }))
local seen = r.echo[1]
assert(r.capability == 31 and #r.echo == 1 and seen.kind == 'forecast' and seen.a == 2 and seen.b == 31 and seen.c == 5 and seen.d == 31, 'the echo is the original observer values in one event')
assert(near(seen.f[1], 0.31429072) and seen.f[2] == 1 and seen.f[3] == 4 and seen.f[4] == 1, 'the modifier after the splash, the penetration, the armour thickness and the random modifier (1)')
r = host.aim(forecast({ weapon_warhead = 3, warhead = 3, damage_capability = 12, armor_penetration_capability = 20, armour_level = 10, damage_modifier = 0.0 }))
assert(r.capability == 12, 'a solid shot that pierces does its damage figure')
r = host.aim(forecast({ weapon_warhead = 3, warhead = 3, damage_capability = 12, armor_penetration_capability = 5, armour_level = 10, damage_modifier = 0.0 }))
assert(r.capability == 0, 'and one that cannot pierce does nothing')
r = host.aim(forecast({ weapon_warhead = 3, warhead = 3, armor_penetration_capability = 50, armour_level = 10, damage_modifier = 0.0, target_not_ai = true, player_modifier = 0.33 }))
assert(r.capability == 33, 'a player target is scaled by the setting after the arithmetic: 100 * 0.33')
r = host.aim(forecast({ weapon_warhead = 8, warhead = 3, damage_capability = 30, burst_duration = 6.0, armor_penetration_capability = 50, armour_level = 10, damage_modifier = 0.0 }))
assert(r.capability == 60, 'a conventional-munitions weapon takes the munition figures scaled by the burst: 30 * 6 / 3')
refused(forecast({ armour_level = -1 }), 'below zero')
refused(forecast({ weapon = 0 }), 'not a weapon')
refused(forecast({ target_not_ai = 1 }), 'target_not_ai must be a boolean')
refused(forecast({ echo = 1 }), 'echo must be a boolean')
refused(forecast({ velocity = 0 / 0 }), 'velocity')
refused(forecast({ damage_capability = 0, armour_level = 0 }), 'not an integer')
local missing = forecast() missing.warhead = nil
refused(missing, 'warhead must be an integer')
-- a host with a model of its own answers the same question without any of that arithmetic, and a provider that corrupts the answer is told apart by the first one
assert(own_model.forecast(forecast()).capability == 5 and own_model.forecast(forecast({ damage_capability = 0 })).capability == 0, 'the plain model: a damage figure does 5, none does nothing')
assert(malformed('bad_forecast_negative', host.aim, passed)(forecast()).capability == -1, 'the first forecast answered with a negative capability')
assert(malformed('bad_forecast_draw', host.aim, passed)(forecast()).state == 'need_draw', 'or with a request for a sample')
assert(malformed('bad_forecast_fraction', host.aim, passed)(forecast()).capability == 1.5, 'or with a capability that is not an integer')
local negative = malformed('bad_forecast_negative', host.aim, passed)
assert(negative(forecast()).capability == -1 and negative(forecast()).capability == 31, 'the first only; the reference host answers the rest')
-- the damage of a hit (EEDCS-46, a real weapon): the question is asked twice, the second time with the campaign's one sample; the answer is the whole of what the hit does.  The
-- expected values are worked out by hand from the original expressions; the kernel's own tests are in src/cmb.rs
local function hit(over)
  local request = { mode = 'damage', weapon = 54, weapon_warhead = 3, warhead = 3, damage_capability = 12, armor_penetration_capability = 20, burst_duration = 0.0,
    weapon_speed = 250.0, weapon_cruise_velocity = 250.0, damage_modifier = 0.0, armour_level = 10, armour_easy = false, armour_model = 0, armour_front = 0.0,
    armour_rear = 0.0, armour_side = 0.0, weapon_position = { 0, 0, 0 }, target_position = { 0, 0, 0 }, target_attitude = { 1, 0, 0, 0, 1, 0, 0, 0, 1 },
    target_not_ai = false, player_modifier = 1.0, has_launcher = false, multiplier = 1, invulnerable = false, cheat_guard = false, damage_level = 50,
    initial_damage_level = 100, critical_model = 0, echo = false }
  for k, v in pairs(over or {}) do request[k] = v end
  return request
end
r = host.aim(hit())
assert(r.state == 'need_draw', 'the arithmetic asks for the campaign sample first')
r = host.aim(hit({ draw = 0 }))
assert(r.state == 'done' and r.exited == false and r.capability == 9 and r.written == true and r.new_level == 41 and r.critical_before == 0 and r.critical_after == 0 and r.bad_modifier == false and r.echo == nil,
  'a solid shot that pierces, with the random modifier 0.75: 12 * 0.75 = 9 points off a health of 50')
r = host.aim(hit({ draw = 0, echo = true }))
assert(#r.echo == 1 and r.echo[1].kind == 'damage' and r.echo[1].a == 1 and r.echo[1].b == 9 and r.echo[1].c == 3 and r.echo[1].d == 9 and near(r.echo[1].f[4], 0.75), 'the original observer values in one event')
r = host.aim(hit({ target_not_ai = true, invulnerable = true }))
assert(r.state == 'done' and r.exited == true and r.capability == 0 and r.written == false and r.new_level == 50, 'an invulnerable player is not hit, and no sample was asked for')
r = host.aim(hit({ draw = 0, target_not_ai = true, invulnerable = true, echo = true }))
assert(r.exited == true and r.echo == nil, 'and a stopped hit echoes nothing')
r = host.aim(hit({ draw = 0, critical_model = 1, damage_level = 23 }))
assert(r.new_level == 14 and r.critical_before == 0 and r.critical_after == 1, 'a vehicle at 14 percent is critical (below 15)')
r = host.aim(hit({ draw = 0, critical_model = 3, damage_level = 13 }))
assert(r.new_level == 4 and r.critical_before == 1 and r.critical_after == 2, 'an aircraft goes from mortal (below 30) to critical (below 5)')
r = host.aim(hit({ draw = 0, cheat_guard = true }))
assert(r.capability == 9 and r.written == false and r.new_level == 50, 'the cheat guard keeps the health')
r = host.aim(hit({ draw = 0, armour_model = 1, armour_front = 2.0, armour_rear = 0.5, armour_side = 1.0, weapon_position = { 0, 0, 100 }, echo = true }))
assert(r.capability == 0 and r.echo[1].f[3] == 20, 'a vehicle hit from the front meets 10 * 2.0 and the penetration 15 does not go through it')
-- a weapon at the target's own position has no direction: the aspect armour is not a number, the hit does nothing and writes nothing (the original's comparison with it is false), and the
-- diagnostic echo of the fixture build still carries every intermediate value, as raw bits, with every number the reply holds finite
local centred = host.aim(hit({ draw = 0, armour_model = 1, armour_front = 2.0, armour_rear = 0.5, armour_side = 1.0, echo = true }))
assert(centred.state == 'done' and centred.capability == 0 and centred.written == false and centred.new_level == 50 and centred.exited == false, 'a centred solid hit does nothing and writes nothing')
assert(#centred.echo == 1 and centred.echo[1].kind == 'damage' and centred.echo[1].a == 0 and centred.echo[1].b == 0 and centred.echo[1].c == 3 and centred.echo[1].d == 0, 'and is still echoed, with no arithmetic path')
for i = 1, 12 do
  local v = centred.echo[1].f[i]
  assert(v == v and v ~= math.huge and v ~= -math.huge, 'echo value ' .. i .. ' is a finite number: ' .. tostring(v))
end
-- the armour (the third value; its halves are f[9], f[10]) is the quiet not-a-number of the aspect arithmetic, as raw bits; the random modifier 0.75 is 0x3F400000 (f[11], f[12])
local armour_high = centred.echo[1].f[9]
assert(armour_high >= 32704 and armour_high < 65536 and armour_high % 32768 >= 32704, 'the armour is a quiet NaN as raw bits: ' .. tostring(armour_high))
assert(centred.echo[1].f[11] == 16192 and centred.echo[1].f[12] == 0, 'and the random modifier is 0.75 bit for bit')
assert(centred.echo[1].f[3] == 0 and centred.echo[1].f[4] == 0.75, 'the readable copy of the value that is not finite is 0, and of a finite one the value')
r = host.aim(hit({ draw = 0, target_not_ai = true, player_modifier = 0.33 }))
assert(r.capability == 2 and r.new_level == 48, 'a player is scaled by the setting after the arithmetic: 9 * 0.33 = 2.97')
local missing = hit({ draw = 0, target_not_ai = true }) missing.player_modifier = nil
assert(host.aim(missing).bad_modifier == true, 'a player with no damage setting is reported for the campaign to stop on')
refused(hit({ draw = 40000 }), 'draw')
refused(hit({ weapon = 0 }), 'not a weapon')
refused(hit({ armour_easy = 1 }), 'armour_easy must be a boolean')
refused(hit({ initial_damage_level = 0, critical_model = 1 }), 'cannot be classified')
refused(hit({ weapon_position = { 0, 0 / 0, 0 } }), 'position')
refused(hit({ echo = 1 }), 'echo must be a boolean')
local no_level = hit() no_level.damage_level = nil
refused(no_level, 'damage_level must be an integer')
-- a host with a model of its own answers the same question without any of that arithmetic or the sample, and a provider that corrupts the first answer is told apart by it
r = own_model.damage(hit({ critical_model = 1, damage_level = 30 }))
assert(r.capability == 10 and r.written == true and r.new_level == 20 and r.critical_before == 0 and r.critical_after == 1 and r.exited == false, 'the plain model: ten points, no sample')
assert(own_model.damage(hit({ target_not_ai = true, invulnerable = true })).exited == true, 'and it honours the invulnerability')
assert(own_model.damage(hit({ cheat_guard = true })).written == false, 'and the cheat guard')
assert(malformed('bad_damage_level', host.aim, passed)(hit({ draw = 0 })).new_level == -1, 'the first hit answered with health below zero')
assert(malformed('bad_damage_exit', host.aim, passed)(hit({ draw = 0 })).exited == true, 'or stopped by an invulnerability the target has not')
assert(malformed('bad_damage_class', host.aim, passed)(hit({ draw = 0 })).critical_after == 5, 'or with a class the target cannot have')
assert(malformed('bad_damage_unwritten', host.aim, passed)(hit({ draw = 0 })).written == false, 'or with health changed and not written')
assert(malformed('bad_damage_missing', host.aim, passed)(hit({ draw = 0 })).capability == nil, 'or with a field missing')
assert(malformed('bad_damage_draw_twice', host.aim, passed)(hit({ draw = 0 })).state == 'need_draw', 'or asking for a second sample')
local armed = malformed('bad_damage_level', host.aim, passed)
assert(armed(hit()).state == 'need_draw' and armed(hit({ draw = 0 })).new_level == -1 and armed(hit({ draw = 0 })).new_level == 41, 'the first completed hit only; the reference host answers the rest')
-- the health questions that are not a hit (EEDCS-46): the reference host answers them by value; the values are worked out by hand from the original expressions (the kernel's own tests are in
-- src/cmb.rs).  A timed roll asks for the campaign's one sample only when the target is eligible
local function health(over)
  local request = { mode = 'health', op = 'classify', critical_model = 1, damage_level = 14, initial_damage_level = 100, old_damage_level = 14, explosive = true, flammable = false,
    delta_time = 0.1, aircraft_class = 0, self_hit = false, target_not_ai = false }
  for k, v in pairs(over or {}) do request[k] = v end
  return request
end
r = host.aim(health())
assert(r.state == 'done' and r.critical_class == 1 and r.crossed == false and r.eligible == false and r.hit == false and r.new_level == 14, 'a vehicle at 14 percent is critical')
assert(host.aim(health({ damage_level = 15 })).critical_class == 0 and host.aim(health({ critical_model = 2, damage_level = 0 })).critical_class == 0, 'at 15 it is not, and a person never is')
assert(host.aim(health({ critical_model = 3, damage_level = 29 })).critical_class == 1 and host.aim(health({ critical_model = 3, damage_level = 4 })).critical_class == 2, 'an aircraft is mortal then critical')
r = host.aim(health({ op = 'assess', old_damage_level = 16, damage_level = 14 }))
assert(r.crossed == true and r.recovered == false and r.eject == false, 'a vehicle that can burn falls below 15 percent')
assert(host.aim(health({ op = 'assess', old_damage_level = 16, damage_level = 14, explosive = false })).crossed == false, 'one that cannot burn does not')
r = host.aim(health({ op = 'assess', critical_model = 3, initial_damage_level = 1000, old_damage_level = 60, damage_level = 30 }))
assert(r.crossed == true and r.eject == true, 'an aircraft at 3 percent and 30 points: a threshold, and the health that ejects an AI crew')
r = host.aim(health({ op = 'assess', critical_model = 3, initial_damage_level = 1000, old_damage_level = 290, damage_level = 310 }))
assert(r.recovered == true and r.crossed == false, 'and recovery above 30 percent')
assert(host.aim(health({ op = 'timed' })).state == 'need_draw', 'an eligible vehicle (critical, can burn, has health) asks for the sample')
r = host.aim(health({ op = 'timed', draw = 6553 }))
assert(r.state == 'done' and r.eligible == true and r.rolled == true and r.hit == true and r.new_level == 13 and r.fatal == false and r.critical_class == 1, 'the sample 6553 is under 2 * 0.1: a hit, one point less')
r = host.aim(health({ op = 'timed', draw = 6554 }))
assert(r.rolled == true and r.hit == false and r.new_level == 14, 'and 6554 is not')
r = host.aim(health({ op = 'timed', damage_level = 1, draw = 0 }))
assert(r.hit == true and r.new_level == 0 and r.fatal == true, 'the last point is fatal')
r = host.aim(health({ op = 'timed', damage_level = 50 }))
assert(r.state == 'done' and r.eligible == false and r.rolled == false and r.new_level == 50, 'a healthy vehicle is not eligible: no sample')
r = host.aim(health({ op = 'timed', critical_model = 2, damage_level = 3 }))
assert(r.state == 'done' and r.eligible == false and r.critical_class == 0, 'a person never bleeds: no sample whatever its health')
r = host.aim(health({ op = 'timed', critical_model = 3, damage_level = 5, aircraft_class = 1 }))
assert(r.state == 'need_draw', 'an aircraft in a class asks for the sample')
assert(host.aim(health({ op = 'timed', critical_model = 3, damage_level = 5, aircraft_class = 1, draw = 3276 })).hit == true and host.aim(health({ op = 'timed', critical_model = 3, damage_level = 5, aircraft_class = 1, draw = 3277 })).hit == false, 'class 1 gives odds of 0.1 at a frame of 0.1 s')
assert(host.aim(health({ op = 'timed', critical_model = 3, damage_level = 5, aircraft_class = 2, draw = 32767 })).hit == true, 'class 2 always hits, and the sample was taken')
assert(host.aim(health({ op = 'timed', critical_model = 3, damage_level = 5, aircraft_class = 0 })).eligible == false, 'in no class: nothing')
assert(host.aim(health({ op = 'hit_eligible', critical_model = 0, damage_level = 0, initial_damage_level = 0, self_hit = true })).eligible == false, 'an AI helicopter ignores its own hit')
assert(host.aim(health({ op = 'hit_eligible', critical_model = 0, damage_level = 0, initial_damage_level = 0, self_hit = true, target_not_ai = true })).eligible == true, 'a player does not')
refused(health({ op = 'timed', draw = 40000 }), 'draw')
refused(health({ op = 'bleed' }), 'unknown health op')
refused(health({ explosive = 1 }), 'explosive must be a boolean')
refused(health({ delta_time = 0 / 0 }), 'delta_time')
refused(health({ initial_damage_level = 0 }), 'cannot be answered')
local no_class = health() no_class.critical_model = nil
refused(no_class, 'critical_model must be an integer')
-- a host with a model of its own answers the same questions without any of that arithmetic or the sample, and a provider that corrupts the first answer of its kind is told apart by it
assert(own_model.health(health()).critical_class == 1 and own_model.health(health({ op = 'timed' })).eligible == false and own_model.health(health({ op = 'timed' })).rolled == false, 'the plain model: its own class, and nothing burns by itself')
assert(own_model.health(health({ op = 'assess', old_damage_level = 50, damage_level = 20 })).crossed == true and own_model.health(health({ op = 'hit_eligible', self_hit = true })).eligible == true, 'its own crossing, and a unit may hurt itself')
assert(malformed('bad_health_class', host.aim, passed)(health()).critical_class == 7, 'the first classification answered with a class no vehicle has')
assert(malformed('bad_health_missing', host.aim, passed)(health()).critical_class == nil, 'or with the class missing')
assert(malformed('bad_health_classify_draw', host.aim, passed)(health()).state == 'need_draw', 'or a sample asked for a classification, which is a pure question')
assert(malformed('bad_health_second_draw', host.aim, passed)(health({ op = 'timed', draw = 6553 })).state == 'need_draw', 'or a second sample for a timed roll that has been given its one')
local second = malformed('bad_health_second_draw', host.aim, passed)
assert(second(health({ op = 'timed' })).state == 'need_draw' and second(health({ op = 'timed', draw = 6553 })).state == 'need_draw' and second(health({ op = 'timed', draw = 6553 })).state == 'done', 'the first only; the reference host asks for the sample first and the provider corrupts the answer to the given one')
-- the reference host takes no sample for a person or an aircraft in no class (its eligibility, the host\'s), which the campaign accepts; a host with a model of its own may
assert(host.aim(health({ op = 'timed', critical_model = 2, damage_level = 3 })).state == 'done' and host.aim(health({ op = 'timed', critical_model = 3, damage_level = 40, aircraft_class = 0 })).state == 'done', 'the reference host rolls for neither')
assert(malformed('bad_health_hit_ineligible', host.aim, passed)(health({ op = 'timed', damage_level = 50 })).hit == true, 'or a hit on an ineligible target')
assert(malformed('bad_health_fatal', host.aim, passed)(health({ op = 'timed', draw = 0 })).fatal == true, 'or a hit whose fatal flag disagrees with the health it leaves')
local classified = malformed('bad_health_class', host.aim, passed)
assert(classified(health()).critical_class == 7 and classified(health()).critical_class == 1, 'the first only; the reference host answers the rest')
-- the impact dispatch and the area blast (EEDCS-46): the reference host answers them by value; the expected values are worked out by hand (the kernel's own tests are in src/cmb.rs)
local function impact(over)
  local request = { mode = 'impact', kill_code = 5, has_target = true, target_alive = true, position = { 250, 12, 750 } }
  for k, v in pairs(over or {}) do request[k] = v end
  return request
end
r = host.aim(impact())
assert(r.state == 'done' and r.direct_hit == true and r.blasts == 1 and r.exclude_target == true and r.crater == false and r.invalid == false, 'a hit on the target: the direct hit, then one blast that leaves it out')
r = host.aim(impact({ target_alive = false }))
assert(r.direct_hit == false and r.blasts == 1 and r.exclude_target == true, 'a target that is dead gets no direct hit but is still left out')
r = host.aim(impact({ kill_code = 3 }))
assert(r.state == 'need_lookup' and r.need == 'terrain', 'a land hit needs the terrain at the impact point')
r = host.aim(impact({ kill_code = 3, terrain_draws_crater = true }))
assert(r.state == 'done' and r.crater == true and r.blasts == 2 and r.direct_hit == false, 'a land hit on terrain that takes a crater: the crater, then two blasts')
assert(host.aim(impact({ kill_code = 3, terrain_draws_crater = false })).crater == false and host.aim(impact({ kill_code = 3, terrain_draws_crater = false })).blasts == 2, 'and on terrain that does not, two blasts and no crater')
assert(host.aim(impact({ kill_code = 1 })).blasts == 0 and host.aim(impact({ kill_code = 2 })).blasts == 0 and host.aim(impact({ kill_code = 4 })).blasts == 0, 'a weapon that ran out, left the map or hit water blasts nothing')
assert(host.aim(impact({ kill_code = 6 })).blasts == 1 and host.aim(impact({ kill_code = 7 })).blasts == 1 and host.aim(impact({ kill_code = 6 })).direct_hit == false, 'an overshot or self-destroyed weapon blasts once')
assert(host.aim(impact({ kill_code = 0 })).invalid == true and host.aim(impact({ kill_code = 8 })).invalid == true, 'a code the dispatch does not know')
refused(impact({ kill_code = 'x' }), 'kill_code')
refused(impact({ terrain_draws_crater = 1 }), 'terrain_draws_crater must be a boolean')
refused(impact({ has_target = 1 }), 'has_target must be a boolean')
r = own_model.impact(impact({ kill_code = 3 }))
assert(r.blasts == 1 and r.crater == false and r.state == 'done', 'a plain model blasts a land hit once, with no crater, and never asks for the terrain')
assert(own_model.impact(impact({ kill_code = 5 })).direct_hit == true and own_model.impact(impact({ kill_code = 1 })).blasts == 0 and own_model.impact(impact({ kill_code = 9 })).invalid == true, 'its other deaths')
assert(malformed('bad_impact_draw', host.aim, passed)(impact()).state == 'need_draw', 'a sample asked for an impact')
local crater = malformed('bad_impact_crater', host.aim, passed)(impact({ kill_code = 3 }))
assert(crater.state == 'done' and crater.crater == true, 'a crater on a terrain that has not been read')
assert(malformed('bad_impact_blasts', host.aim, passed)(impact()).blasts == 3, 'or three blasts')
local function blast(over)
  local request = { mode = 'blast', position = { 250, 0, 250 }, damage_capability = 400, warhead_type = 7, has_target = false, sector_side_length = 100,
    min_x_sector = 0, max_x_sector = 99, min_z_sector = 0, max_z_sector = 99, echo = false }
  for k, v in pairs(over or {}) do request[k] = v end
  return request
end
local function object(handle, kind, y, extra)
  local o = { handle = handle, entity_type = kind, readable = true, alive = true, has_position = true, rocket = false, excluded = false, position = { 250, y, 250 } }
  for k, v in pairs(extra or {}) do o[k] = v end
  return o
end
local function window(objects, ends, sx, sz, after) return { sx = sx or 2, sz = sz or 2, after = after or 0, list_ends = ends, objects = objects } end
r = host.aim(blast())
assert(r.state == 'need_lookup' and r.radius == 20 and r.need.sx == 2 and r.need.sz == 2 and r.need.after == 0, 'capability 400 is a radius of 20 and, from (250, 250), the one sector 2, 2')
r = host.aim(blast({ damage_capability = 0 }))
assert(r.state == 'done' and r.radius == 0 and r.action == 'none', 'a capability of zero is a radius of zero: nothing to walk')
r = host.aim(blast({ damage_capability = 10000 }))
assert(r.need.sx == 1 and r.need.sz == 1 and r.radius == 100, 'capability 10000 is a radius of 100: sectors 1 to 3, from the first')
r = host.aim(blast({ window = window({ object(1, 6, 0), object(2, 6, 10) }, true) }))
assert(r.state == 'done' and r.action == 'collision' and r.handle == 2 and r.modifier == 0.5 and r.radius == 20, 'range 0 is never reached; 10 of 20 is a collision with the modifier 0.5')
r = host.aim(blast({ echo = true, window = window({ object(1, 6, 0), object(2, 6, 10) }, true) }))
assert(#r.echo == 2 and r.echo[1].kind == 'blast' and r.echo[1].a == 0 and r.echo[1].b == 0 and r.echo[2].a == 1 and r.echo[2].b == 1 and r.echo[2].f[1] == 10 and r.echo[2].f[2] == 20 and r.echo[2].f[3] == 0.5, 'the echo is the candidates examined, in order, with their range, radius and modifier')
r = host.aim(blast({ window = window({ object(1, 6, 10, { excluded = true }), object(2, 3, 5), object(3, 6, 5, { alive = false }), object(4, 6, 20), object(5, 6, 5, { has_position = false }) }, true) }))
assert(r.state == 'done' and r.action == 'none', 'the excluded target, a kind not examined, a dead object, one exactly at the radius and one with no position are not reached')
r = host.aim(blast({ window = window({ object(5, 6, 30) }, false) }))
assert(r.state == 'need_lookup' and r.need.sx == 2 and r.need.sz == 2 and r.need.after == 5, 'a list that did not end is continued after its last object')
r = host.aim(blast({ damage_capability = 10000, window = window({}, true, 1, 1) }))
assert(r.need.sx == 2 and r.need.sz == 1 and r.need.after == 0, 'the next sector of the row')
r = host.aim(blast({ window = window({ object(3, 37, 5, { rocket = true }) }, true) }))
assert(r.action == 'intercept' and r.handle == 3 and r.modifier == 0.25, 'a rocket a quarter of the radius from a high-explosive anti-aircraft blast is destroyed (1 - 0.25^1.54 = 0.88)')
r = host.aim(blast({ window = window({ object(3, 37, 10, { rocket = true }) }, true) }))
assert(r.action == 'none', 'and not at half the radius (1 - 0.5^1.54 = 0.66)')
r = host.aim(blast({ window = window({ object(3, 37, 5) }, true) }))
assert(r.action == 'none', 'a weapon that is not a rocket is examined and left alone')
refused(blast({ window = window({}, true, 9, 9) }), 'cannot be answered')
refused(blast({ damage_capability = -4 }), 'cannot be answered')
refused(blast({ position = { 0 / 0, 0, 0 } }), 'position')
refused(blast({ has_target = 0 }), 'has_target must be a boolean')
local plain = own_model.blast(blast())
assert(plain.state == 'need_lookup' and plain.radius == 30 and plain.need.sx == 2 and plain.need.sz == 2, 'a plain model: thirty metres, the sector of the explosion')
r = own_model.blast(blast({ window = window({ object(1, 3, 10) }, true) }))
assert(r.action == 'collision' and r.handle == 1 and r.modifier == 341 / 1024, 'it reaches an object of any kind that is alive and placed')
assert(own_model.blast(blast({ window = window({ object(1, 3, 40) }, true) })).action == 'none', 'and none beyond its radius')
assert(malformed('bad_blast_draw', host.aim, passed)(blast()).state == 'need_draw', 'a sample asked for a blast')
assert(malformed('bad_blast_sector', host.aim, passed)(blast()).need.sx == 100, 'a window of a sector outside the map')
assert(malformed('bad_blast_handle', host.aim, passed)(blast({ window = window({ object(1, 6, 10) }, true) })).handle == 99999, 'an object that is not in the window')
assert(malformed('bad_blast_modifier', host.aim, passed)(blast({ window = window({ object(1, 6, 10) }, true) })).modifier == 2, 'a modifier above one')
assert(malformed('bad_blast_rocket', host.aim, passed)(blast({ window = window({ object(1, 6, 10) }, true) })).action == 'intercept', 'a destruction of an object that is not a rocket')
assert(malformed('bad_blast_radius', host.aim, passed)(blast({ window = window({ object(1, 6, 30) }, true) })).radius == 21, 'a radius that changes within a blast')
local radius_echo = malformed('bad_blast_radius', host.aim, passed)(blast({ echo = true, window = window({ object(1, 6, 30) }, true) }))
assert(radius_echo.radius == 21 and #radius_echo.echo == 1 and radius_echo.echo[1].f[2] == 21, 'with the echo on, the diagnostic envelope says the new radius too (a supplied window and a positive radius are all it needs), so the changed radius is the primary refusal')
assert(host.aim(blast({ echo = true, window = window({ object(1, 6, 30) }, true) })).echo[1].f[2] == 20, 'and the unmodified reference says the old one')
local again = malformed('bad_blast_repeat', host.aim, passed)(blast({ damage_capability = 10000, window = window({}, true, 1, 1) }))
assert(again.need.sx == 1 and again.need.sz == 1 and again.need.after == 0, 'the start of a sector asked for again')
-- the physical collapse of a bridge segment (EEDCS-46, R5): the reference host answers by value; the expected values are worked out by hand (the kernel's own tests are in src/cmb.rs)
local function bridge(over)
  local request = { mode = 'bridge', op = 'kill', segment_kind = 3, alive = true, server = true, pred_present = false, pred_alive = false, succ_present = false, succ_alive = false, which = 0,
    neighbour_present = false, neighbour_kind = 0, neighbour_alive = false, position = { 250, 120, 250 }, delta_time = 0.1, segment_length = 10, sector_side_length = 100,
    min_x_sector = 0, max_x_sector = 99, min_z_sector = 0, max_z_sector = 99 }
  for k, v in pairs(over or {}) do request[k] = v end
  return request
end
assert(host.aim(bridge()).kill == true, 'a living unsupported span of a server has nothing to hold it: it is killed')
assert(host.aim(bridge({ alive = false })).kill == false and host.aim(bridge({ server = false })).kill == false and host.aim(bridge({ segment_kind = 2 })).kill == false, 'a dead one, a client and a supported one are not')
r = host.aim(bridge({ op = 'shape', alive = true, pred_present = true, pred_alive = false, succ_present = true, succ_alive = true }))
assert(r.state == 'done' and r.variant == 2, 'the predecessor is dead: variant 2')
assert(host.aim(bridge({ op = 'shape', alive = false })).variant == 1 and host.aim(bridge({ op = 'shape', alive = true })).variant == 0, 'a dead segment is destroyed; one with no neighbour is normal')
assert(host.aim(bridge({ op = 'shape', pred_present = true, pred_alive = true, succ_present = true, succ_alive = false })).variant == 3 and host.aim(bridge({ op = 'shape', pred_present = true, succ_present = true })).variant == 4, 'successor dead: 3; both dead: 4')
assert(host.aim(bridge({ op = 'neighbour', neighbour_present = true })).notify == true and host.aim(bridge({ op = 'neighbour' })).notify == false, 'a neighbour is told if it is there')
r = host.aim(bridge({ op = 'fall' }))
assert(r.state == 'need_lookup' and r.need == 'terrain', 'an unsupported segment asks for the terrain under it')
r = host.aim(bridge({ op = 'fall', terrain = { height = 100, water = false } }))
assert(r.state == 'done' and r.fell == true and r.dropped_y == 119 and r.floor_height == 100 and r.landed == false and r.remove == false and r.landed_y == nil, 'from 120 a frame of 0.1 drops to 119, above a floor of 100')
r = host.aim(bridge({ op = 'fall', position = { 250, 100.5, 250 }, terrain = { height = 100, water = false } }))
assert(r.landed == true and r.remove == true and r.dropped_y == 99.5 and r.landed_y == 100, 'from 100.5 it lands at the floor and leaves the list')
r = host.aim(bridge({ op = 'fall', position = { 250, 100.5, 250 }, terrain = { height = 100, water = true } }))
assert(r.fell == true and r.landed == false and r.floor_height == 99, 'a water floor is a metre lower: still above it')
r = host.aim(bridge({ op = 'fall', segment_kind = 2 }))
assert(r.state == 'done' and r.fell == false and r.remove == true and r.dropped_y == nil, 'a supported segment leaves the list at once, with no terrain asked')
r = host.aim(bridge({ op = 'rest', alive = false }))
assert(r.state == 'need_lookup' and r.need == 'terrain', 'a dead unsupported segment asks for the terrain it rests on')
r = host.aim(bridge({ op = 'rest', alive = false, terrain = { height = 100.25, water = false } }))
assert(r.state == 'done' and r.landed == true and r.landed_y == 100.25 and r.fell == nil and r.remove == nil, 'it rests on the ground and has not fallen')
r = host.aim(bridge({ op = 'rest', alive = false, terrain = { height = 100.25, water = true } }))
assert(r.landed == true and r.landed_y == 99.25, 'a metre lower over water')
r = host.aim(bridge({ op = 'rest', alive = true }))
assert(r.state == 'done' and r.landed == false and r.landed_y == nil, 'a living segment stays where it was saved, with no terrain asked')
r = host.aim(bridge({ op = 'rest', alive = false, segment_kind = 2 }))
assert(r.state == 'done' and r.landed == false, 'a dead supported segment stays')
r = host.aim(bridge({ op = 'vehicles' }))
assert(r.state == 'need_lookup' and r.need.sx == 2 and r.need.sz == 2 and r.need.after == 0, 'the segment own sector is walked')
local function vehicle(handle, offset, extra)
  local o = { handle = handle, entity_type = 23, readable = true, alive = true, has_position = true, rocket = false, excluded = false, position = { 250 + offset, 5, 250 } }
  for k, v in pairs(extra or {}) do o[k] = v end
  return o
end
r = host.aim(bridge({ op = 'vehicles', position = { 250, 5, 250 }, window = { sx = 2, sz = 2, after = 0, list_ends = true, objects = { vehicle(1, 30), vehicle(2, 14, { alive = false }), vehicle(3, 13) } } }))
assert(r.state == 'done' and r.action == 'act' and r.handle == 3, 'the first living vehicle inside 10 * root two by the approximate range is killed')
r = host.aim(bridge({ op = 'vehicles', position = { 250, 5, 250 }, window = { sx = 2, sz = 2, after = 0, list_ends = true, objects = { vehicle(1, 15) } } }))
assert(r.state == 'done' and r.action == 'none', 'one outside is not')
assert(host.aim(bridge({ op = 'vehicles', segment_length = 0 })).action == 'none', 'a segment of no length walks nothing')
refused(bridge({ op = 'sink' }), 'unknown bridge op')
refused(bridge({ alive = 1 }), 'alive must be a boolean')
refused(bridge({ position = { 0 / 0, 0, 0 } }), 'position')
refused(bridge({ op = 'vehicles', window = { sx = 9, sz = 9, after = 0, list_ends = true, objects = {} } }), 'cannot be answered')
-- a host with a model of its own answers the same questions from nothing of EECH's, and the provider that corrupts the first answer of its kind is told apart by it
assert(own_model.bridge(bridge()).kill == false and own_model.bridge(bridge({ op = 'neighbour', neighbour_present = true })).notify == false, 'the plain model holds every span up and tells no neighbour')
assert(own_model.bridge(bridge({ op = 'shape', alive = true, pred_present = true, pred_alive = false })).variant == 0 and own_model.bridge(bridge({ op = 'shape', alive = false })).variant == 1, 'and shapes a segment by itself alone')
r = own_model.bridge(bridge({ op = 'fall', terrain = { height = 100, water = true }, position = { 250, 100.1, 250 } }))
assert(r.fell == true and r.landed == true and r.landed_y == 100 and r.floor_height == 100, 'it falls at two metres a second to the height of the terrain (no water offset of its own)')
assert(own_model.bridge(bridge({ op = 'vehicles' })).action == 'none', 'and kills no vehicle')
r = own_model.bridge(bridge({ op = 'rest', alive = false, terrain = { height = 100, water = true } }))
assert(r.landed == true and r.landed_y == 100 and own_model.bridge(bridge({ op = 'rest', alive = false })).need == 'terrain' and own_model.bridge(bridge({ op = 'rest', alive = true })).landed == false, 'its dead segments lie on the terrain, none moved if alive')
assert(malformed('bad_bridge_draw', host.aim, passed)(bridge()).state == 'need_draw', 'a sample asked for a bridge question')
assert(malformed('bad_bridge_kill_dead', host.aim, passed)(bridge({ alive = false })).kill == true, 'a dead segment killed')
assert(malformed('bad_bridge_variant', host.aim, passed)(bridge({ op = 'shape' })).variant == 9, 'a shape variant that does not exist')
assert(malformed('bad_bridge_notify_absent', host.aim, passed)(bridge({ op = 'neighbour' })).notify == true, 'a neighbour that is not there told')
assert(malformed('bad_bridge_landed_not_fell', host.aim, passed)(bridge({ op = 'fall', segment_kind = 2 })).landed == true, 'a landing without a fall')
local nonfinite = malformed('bad_bridge_nonfinite', host.aim, passed)(bridge({ op = 'fall', terrain = { height = 100, water = false } }))
assert(nonfinite.dropped_y ~= nonfinite.dropped_y, 'a drop that is not a number')
assert(malformed('bad_bridge_terrain_twice', host.aim, passed)(bridge({ op = 'fall', terrain = { height = 100, water = false } })).need == 'terrain', 'the terrain asked for again')
local handle = malformed('bad_bridge_handle', host.aim, passed)(bridge({ op = 'vehicles', position = { 250, 5, 250 }, window = { sx = 2, sz = 2, after = 0, list_ends = true, objects = { vehicle(1, 3) } } }))
local alive_moved = malformed('bad_bridge_rest_alive', host.aim, passed)(bridge({ op = 'rest', alive = true }))
assert(alive_moved.landed == true and alive_moved.landed_y == 1, 'a living segment moved')
local rest_nonfinite = malformed('bad_bridge_rest_nonfinite', host.aim, passed)(bridge({ op = 'rest', alive = false, terrain = { height = 100, water = false } }))
assert(rest_nonfinite.landed_y ~= rest_nonfinite.landed_y, 'a resting height that is not a number')
assert(handle.handle == 99999, 'an object that is not in the window')
-- the identities of the physical objects that are not rostered mobiles (EEDCS-46, R3): the host issues one for a structure from its own count, an offered object carries the identity it was given, and the host repeats it for the object it reaches
local function identify(over)
  local request = { mode = 'identify', entity_type = 5, sub_type = 41, side = 2, alive = true, damage_level = 80, initial_damage_level = 100, position = { 100, 5, 250 }, serial = 1 }
  for k, v in pairs(over or {}) do request[k] = v end
  return request
end
r = host.aim(identify())
assert(r.state == 'done' and r.host_id == 'eech-world:object:1', 'the reference host issues an identity from its own count')
assert(host.aim(identify({ serial = 9 })).host_id == 'eech-world:object:2' and host.aim(identify({ serial = 9 })).host_id == 'eech-world:object:3', 'a new one each time, whatever the campaign sent')
refused(identify({ position = { 0 / 0, 0, 0 } }), 'position')
refused(identify({ alive = 1 }), 'alive must be a boolean')
refused(identify({ serial = 'x' }), 'serial')
r = host.aim(blast({ window = window({ object(1, 6, 0), object(2, 6, 10, { host_id = 'obj-2' }) }, true) }))
assert(r.action == 'collision' and r.handle == 2 and r.host_id == 'obj-2', 'the identity the object was offered under is repeated for the object the blast reaches')
r = host.aim(blast({ window = window({ object(1, 6, 10) }, true) }))
assert(r.action == 'collision' and r.handle == 1 and r.host_id == nil, 'an object offered with none is repeated as none')
r = host.aim(bridge({ op = 'vehicles', position = { 250, 5, 250 }, window = { sx = 2, sz = 2, after = 0, list_ends = true, objects = { vehicle(7, 3, { host_id = 'veh-7' }) } } }))
assert(r.action == 'act' and r.handle == 7 and r.host_id == 'veh-7', 'and for the vehicle a collapse kills')
r = own_model.blast(blast({ window = window({ object(1, 3, 10, { host_id = 'obj-1' }) }, true) }))
assert(r.action == 'collision' and r.host_id == 'obj-1', 'a plain model repeats the identity too')
r = own_model.identify(identify({ serial = 4 }))
assert(r.state == 'done' and r.host_id == 'own-world/structure/4', 'a plain model issues its own kind of identity')
assert(malformed('bad_identify_empty', host.aim, passed)(identify()).host_id == '', 'a provider that issues an empty identity')
assert(#malformed('bad_identify_long', host.aim, passed)(identify()).host_id == 129, 'one that is too long')
assert(malformed('bad_identify_missing', host.aim, passed)(identify()).host_id == nil, 'one that issues none')
assert(malformed('bad_identify_control', host.aim, passed)(identify()).host_id:find('\\n', 1, true), 'one with a control character')
assert(malformed('bad_identify_draw', host.aim, passed)(identify()).state == 'need_draw', 'one that asks for a sample')
local repeating = malformed('bad_identify_repeat', host.aim, passed)
local issued_first = repeating(identify()).host_id
assert(repeating(identify()).host_id == issued_first, 'one that issues the same identity twice')
assert(malformed('bad_blast_identity', host.aim, passed)(blast({ window = window({ object(2, 6, 10, { host_id = 'obj-2' }) }, true) })).host_id == 'x-obj-2', 'a blast answer with the identity of another object')
assert(malformed('bad_blast_identity_missing', host.aim, passed)(blast({ window = window({ object(2, 6, 10, { host_id = 'obj-2' }) }, true) })).host_id == nil, 'and with none for an object offered with one')
assert(malformed('bad_bridge_identity', host.aim, passed)(bridge({ op = 'vehicles', position = { 250, 5, 250 }, window = { sx = 2, sz = 2, after = 0, list_ends = true, objects = { vehicle(7, 3, { host_id = 'veh-7' }) } } })).host_id == 'x-veh-7', 'a vehicles answer with the identity of another object')
-- the identity of the object a hit, a health fact and a weapon's death are about (EEDCS-46, R3): the reference host accepts it (the host's own, checked for shape) and answers exactly as it does without; a malformed one is refused
local function same_answer(a, b)
  for k, v in pairs(a) do if type(v) ~= 'table' and b[k] ~= v then return false end end
  return true
end
for _, make in ipairs({ { 'damage', hit }, { 'health', health }, { 'impact', impact } }) do
  local name, build = make[1], make[2]
  local plain = host.aim(build())
  local with = host.aim(build({ target_host_id = 'eech-world:object:5' }))
  assert(same_answer(plain, with) and same_answer(with, plain), name .. ': the identity of the subject changes no answer')
  refused(build({ target_host_id = 5 }), 'target_host_id must be a string')
  refused(build({ target_host_id = string.rep('x', 129) }), 'target_host_id must be 1 to 128 bytes')
  refused(build({ target_host_id = '' }), 'target_host_id must be 1 to 128 bytes')
end
print('AIM_HOST=ok')";

#[test]
fn aim_round_trips_through_lua_and_refuses_what_it_was_not_asked() {
    let script = std::env::temp_dir().join(format!("eech-world-aim-{}.lua", std::process::id()));
    let lua_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("lua")
        .display()
        .to_string()
        .replace('\\', "/");
    std::fs::write(&script, AIM_SCRIPT.replace("@LUA_DIR@", &lua_dir)).unwrap();
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

#[test]
fn the_provider_and_the_controlled_witness_find_the_model_files_beside_them_when_the_binary_runs_them_as_the_main_script() {
    // the binary names the main script's chunk by its path alone, so a script that looks for a file beside itself must accept a source with no leading '@'; the root does not
    // exist, so each run ends at the boot, after it has read the model files it needs and before it flies anything
    let lua_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("lua")
        .display()
        .to_string()
        .replace('\\', "/");
    for (script, own_model) in [("aim-provider.lua", "mode=own_model"), ("weapon-case.lua", "aim_model=own_model")] {
        let out = std::env::temp_dir().join(format!("eech-world-main-script-{}-{script}.out", std::process::id()));
        let run = Command::new(env!("CARGO_BIN_EXE_eech-world"))
            .arg(format!("{lua_dir}/{script}"))
            .arg("root=/nonexistent-root-for-a-script-test")
            .arg("scenario=lebanon_retail")
            .arg(format!("out={}", out.display().to_string().replace('\\', "/")))
            .arg(own_model)
            .output()
            .unwrap();
        let _ = std::fs::remove_file(&out);
        let text = format!("{}\n{}", String::from_utf8_lossy(&run.stdout), String::from_utf8_lossy(&run.stderr));
        assert!(!text.contains("cannot open"), "{script} could not find a model file beside it: {text}");
        assert!(!text.contains("own-model.lua"), "{script}: {text}");
    }
}

// The combat report's six hashes are sixteen-digit strings. A hash of decimal digits only must not be read as a number (that would drop its leading zeros and, above 2^53,
// its low digits): the test loads the SAME writer and parser the characterisation script uses (lua/combat-report.lua) with a synthetic report and reads what it wrote.
const COMBAT_REPORT_SCRIPT: &str = r#"
local make = dofile('@LUA_DIR@/combat-report.lua')
local written = {}
local sink = { write = function(_, text) written[#written + 1] = text end, flush = function() end }
local writer = make(sink)
writer.emit_report('run', 600, table.concat({
  'cmb-hash damage=0123456789012345 collision=9007199254740993 blast=0000000000000255 repair=1000000000000000 timed=0000000000000001 order=12345678901234567890',
  'cmb-hash damage=0123456789abcdef collision=ffffffffffffffff blast=00000000000000ff repair=0000000000000000 timed=0000000000000000 order=1',
  'cmb-integrity depth_max=2 depth_overflow=0 trusted=1',
  'cmb-kind type=HELICOPTER collisions=3 damage_sum=5.5',
  -- the SAME word `blast` is a hash in cmb-hash (above) and a COUNTER in a cmb-kind row: its type follows the event, not the word
  'cmb-kind type=CITY_BUILDING collisions=70 direct=1 blast=69 applied=69 candidates=105876',
  'cmb-kind type=SEGMENT collisions=8 direct=0 blast=0000000000000008 applied=8',
  'cmb-repair resupply=5 fixed_restore=0 other=0 zero_to_positive=0',
  'cmb-timed kind=1 attempts=626574 draws=364 hits=43 fatal=0',
  'cmb-ev seq=7 kind=timed type=1 receiver=ENTITY_TYPE_HELICOPTER draws=1 new_level=0',
}, '\n'))
local text = table.concat(written)
local function has(fragment) assert(text:find(fragment, 1, true), 'missing ' .. fragment .. ' in ' .. text) end
local function lacks(fragment) assert(not text:find(fragment, 1, true), 'unexpected ' .. fragment .. ' in ' .. text) end
has('"damage":"0123456789012345"')
has('"collision":"9007199254740993"')
has('"blast":"0000000000000255"')
has('"repair":"1000000000000000"')
has('"timed":"0000000000000001"')
has('"order":"12345678901234567890"')
has('"timed":"0000000000000000"')
has('"order":"1"')
-- a count stays a number and a name stays a name
has('"depth_max":2')
has('"trusted":1')
has('"collisions":3')
has('"type":"HELICOPTER"')
-- the counter named like a hash is a number in a cmb-kind row, and the hash named like a counter is a string in the cmb-hash line, in the same report
has('"blast":69,')
lacks('"blast":"69"')
has('"blast":8,')
has('"blast":"0000000000000255"')
lacks('"blast":255')
-- other counters stay numbers (including the ones that share a word with a hash key elsewhere) and a name that looks like one stays what the emitter wrote
has('"resupply":5')
has('"hits":43')
has('"kind":"timed"')
has('"type":1}')
print('COMBAT_REPORT=ok')
"#;

#[test]
fn the_combat_report_keeps_every_hash_a_string_and_the_script_finds_its_report_module_beside_it() {
    let lua_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("lua")
        .display()
        .to_string()
        .replace('\\', "/");
    let script = std::env::temp_dir().join(format!("eech-world-combat-report-{}.lua", std::process::id()));
    std::fs::write(&script, COMBAT_REPORT_SCRIPT.replace("@LUA_DIR@", &lua_dir)).unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_eech-world")).arg(&script).output();
    std::fs::remove_file(&script).unwrap();
    let run = run.unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run.status.success(), "{}\n{stdout}\n{stderr}", run.status);
    assert!(stdout.contains("COMBAT_REPORT=ok"), "{stdout}\n{stderr}");

    // run as the main script, the characterisation script finds the module beside it (its root does not exist, so it ends at the boot)
    let out = std::env::temp_dir().join(format!("eech-world-combat-main-{}.jsonl", std::process::id()));
    let run = Command::new(env!("CARGO_BIN_EXE_eech-world"))
        .arg(format!("{lua_dir}/combat-characterization.lua"))
        .arg("root=/nonexistent-root-for-a-script-test")
        .arg(format!("out={}", out.display().to_string().replace('\\', "/")))
        .output()
        .unwrap();
    let _ = std::fs::remove_file(&out);
    let text = format!("{}\n{}", String::from_utf8_lossy(&run.stdout), String::from_utf8_lossy(&run.stderr));
    assert!(!text.contains("combat-report.lua"), "the module was not found beside the script: {text}");
}

// The controlled witness reads what each case did as the deltas of the observer's counters between two reports. A report has INDEXED rows (a kill code, a warhead, an entity kind,
// a timed roll's kind): each row's counter must keep its own key, or a later row overwrites an earlier one and a change disappears from the delta. The test loads the SAME
// flatten-and-delta code the witness uses (lua/combat-counters.lua) with two reports that differ in ONE kill code's calls.
const COMBAT_COUNTERS_SCRIPT: &str = r#"
local counting = dofile('@LUA_DIR@/combat-counters.lua')
local function report(code3, code7, warhead4, helicopter, timed4_draws)
  return table.concat({
    'cmb-hash damage=0123456789012345 collision=9007199254740993 blast=0000000000000255 repair=1000000000000000 timed=0000000000000001 order=12345678901234567890',
    'cmb-blast calls=' .. (code3 + code7) .. ' zero_radius=70 candidates=134614 max_candidates=1519',
    'cmb-blast-kill code=3 calls=' .. code3,
    'cmb-blast-kill code=7 calls=' .. code7,
    'cmb-warhead type=4 real=' .. warhead4 .. ' forecast=81613 penetrated=56906 splash=24756',
    'cmb-warhead type=5 real=88 forecast=39485 penetrated=37506 splash=2067',
    'cmb-kind type=ENTITY_TYPE_HELICOPTER collisions=' .. helicopter .. ' direct=45 blast=3 max_damage=17',
    'cmb-kind type=ENTITY_TYPE_ROUTED_VEHICLE collisions=81 direct=20 blast=61 max_damage=500',
    'cmb-timed kind=1 attempts=1026989 draws=364 hits=43 fatal=0',
    'cmb-timed kind=4 attempts=4134689 draws=' .. timed4_draws .. ' hits=0 fatal=0',
    'cmb-unobserved type=ENTITY_TYPE_BRIDGE deliveries=31 note=counted_only',
  }, '\n')
end
local before = report(1054, 3, 49, 48, 0)
local after = report(1056, 3, 50, 49, 7)

-- each indexed row keeps its own raw value in the flattened counters (never a sum over the rows)
local b, a = counting.counters(before), counting.counters(after)
assert(a['cmb-blast-kill.code=3.calls'] == 1056 and a['cmb-blast-kill.code=7.calls'] == 3, 'both kill-code rows keep their own value')
assert(a['cmb-blast-kill.calls'] == nil, 'no ambiguous key that a later kill-code row could overwrite')
assert(a['cmb-warhead.type=4.real'] == 50 and a['cmb-warhead.type=5.real'] == 88)
assert(a['cmb-kind.type=ENTITY_TYPE_HELICOPTER.collisions'] == 49 and a['cmb-kind.type=ENTITY_TYPE_ROUTED_VEHICLE.collisions'] == 81)
assert(a['cmb-timed.kind=1.draws'] == 364 and a['cmb-timed.kind=4.draws'] == 7)
assert(a['cmb-unobserved.type=ENTITY_TYPE_BRIDGE.deliveries'] == 31)
assert(a['cmb-blast.calls'] == 1059 and b['cmb-blast.calls'] == 1057)

-- the delta of a case that changed code 3 (and not the later code 7) shows code 3
local d = counting.deltas(before, after)
assert(d['cmb-blast-kill.code=3.calls'] == 2, 'the kill-code 3 change is visible: ' .. tostring(d['cmb-blast-kill.code=3.calls']))
assert(d['cmb-blast-kill.code=7.calls'] == nil, 'the unchanged later kill-code row is not in the delta')
assert(d['cmb-warhead.type=4.real'] == 1 and d['cmb-warhead.type=5.real'] == nil)
assert(d['cmb-kind.type=ENTITY_TYPE_HELICOPTER.collisions'] == 1 and d['cmb-kind.type=ENTITY_TYPE_ROUTED_VEHICLE.collisions'] == nil)
assert(d['cmb-timed.kind=4.draws'] == 7 and d['cmb-timed.kind=1.draws'] == nil)
assert(d['cmb-blast.calls'] == 2)
-- maxima are not counters; hashes are not counters; and nothing else moved
assert(d['cmb-blast.max_candidates'] == nil and d['cmb-kind.type=ENTITY_TYPE_HELICOPTER.max_damage'] == nil)
for k in pairs(d) do assert(not string.find(k, 'cmb-hash', 1, true), 'a hash is not a counter: ' .. k) end
local n = 0
for _ in pairs(d) do n = n + 1 end
assert(n == 5, 'exactly the five changed counters: ' .. n)
-- the reverse change (code 7 up, code 3 unchanged) is visible too, so neither row hides the other
local d2 = counting.deltas(before, report(1054, 5, 49, 48, 0))
assert(d2['cmb-blast-kill.code=7.calls'] == 2 and d2['cmb-blast-kill.code=3.calls'] == nil)
print('COMBAT_COUNTERS=ok')
"#;

#[test]
fn the_case_deltas_keep_every_indexed_report_row_apart() {
    let lua_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("lua")
        .display()
        .to_string()
        .replace('\\', "/");
    let script = std::env::temp_dir().join(format!("eech-world-combat-counters-{}.lua", std::process::id()));
    std::fs::write(&script, COMBAT_COUNTERS_SCRIPT.replace("@LUA_DIR@", &lua_dir)).unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_eech-world")).arg(&script).output();
    std::fs::remove_file(&script).unwrap();
    let run = run.unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run.status.success(), "{}\n{stdout}\n{stderr}", run.status);
    assert!(stdout.contains("COMBAT_COUNTERS=ok"), "{stdout}\n{stderr}");
}

// The controlled witness spends its carrier's ammunition (a launch per case), and the first controlled original run took the FIRST launcher carrying each class whatever it held: the
// high-explosive carrier launched six weapons and was then refused thirteen times. The test loads the SAME carrier choice the witness uses (lua/combat-plan.lua) with a synthetic
// inventory shaped like the measured one (first carrier of the class holds six, a later one holds plenty).
const COMBAT_PLAN_SCRIPT: &str = r#"
local planning = dofile('@LUA_DIR@/combat-plan.lua')
local function he(w) return w == 4 end
local launchers = {
  { id = 'eech-world:2690', inventory = { { weapon = 54, count = 6, warhead = 4 } } },
  { id = 'eech-world:2700', inventory = { { weapon = 11, count = 2, warhead = 1 }, { weapon = 54, count = 40, warhead = 4 } } },
  { id = 'eech-world:2784', inventory = { { weapon = 76, count = 12, warhead = 1 } } },
}
-- the plan: a direct hit and a blast-range collision per living kind, a land-hit explosion per kind and one hit on target
assert(planning.need(0) == 0 and planning.need(1) == 4 and planning.need(7) == 22, 'need ' .. planning.need(7))
-- a carrier that cannot afford the plan is passed over, even though it is first
local id, weapon, warhead, count, shortfall = planning.choose(launchers, he, planning.need(7))
assert(id == 'eech-world:2700' and weapon == 54 and warhead == 4 and count == 40 and shortfall == 0, 'the first affordable carrier: ' .. tostring(id) .. ' ' .. tostring(count))
-- when the first one does afford it, it is kept (the choice is stable, in launcher order)
id, weapon, _, count, shortfall = planning.choose(launchers, he, 6)
assert(id == 'eech-world:2690' and count == 6 and shortfall == 0, 'the first carrier that affords the plan is kept')
-- when none affords it, the one with the most ammunition is taken and the shortfall is reported, not hidden
id, weapon, _, count, shortfall = planning.choose(launchers, he, 100)
assert(id == 'eech-world:2700' and count == 40 and shortfall == 60, 'the richest carrier with the shortfall: ' .. tostring(shortfall))
-- a class nobody carries is nothing
assert(planning.choose(launchers, function(w) return w == 7 end, 4) == nil, 'no carrier for a class nobody holds')
-- the solid class is chosen by its own warhead, and a carrier that holds both is read per weapon
id, weapon, warhead, count, shortfall = planning.choose(launchers, function(w) return w == 1 end, 12)
assert(id == 'eech-world:2784' and weapon == 76 and count == 12 and shortfall == 0)
-- the class of a role: the first class that is not estimated short, else the one with the most ammunition
local dual = { class = 'he_dual_purpose', count = 6, shortfall = 4 }
local plain = { class = 'he_plain', count = 100, shortfall = 0 }
local aa = { class = 'he_anti_aircraft', count = 30, shortfall = 0 }
local c, why = planning.choose_class({ dual, plain, aa })
assert(c == plain and why == 'first_affordable_alternate', 'the affordable alternate, not the preferred class that is short')
c, why = planning.choose_class({ plain, dual })
assert(c == plain and why == 'preferred', 'the preferred class when it is affordable')
c, why = planning.choose_class({ { class = 'a', count = 3, shortfall = 7 }, { class = 'b', count = 5, shortfall = 5 }, { class = 'c', count = 5, shortfall = 5 } })
assert(c.class == 'b' and why == 'none_affordable_most_ammunition', 'none affordable: the most ammunition, the first on a tie')
assert(planning.choose_class({}) == nil, 'no class, no choice')
-- the carrier a case launches from NOW is read from the live count and the rounds one launch cost
local role = { { key = 'A', class = 'he_dual_purpose' }, { key = 'B', class = 'he_plain' } }
local live = { A = 10, B = 100 }
local debit = {}
local function count(carrier) return live[carrier.key] end
local p = planning.pick(role, count, debit)
assert(p.key == 'A', 'the chosen carrier while it has ammunition')
planning.note_debit(debit, p, 10, 6)
assert(debit.A == 4, 'a salvo that spent four rounds is remembered: ' .. tostring(debit.A))
planning.note_debit(debit, p, 6, 5)
assert(debit.A == 4, 'a cheaper launch does not lower the learned cost')
planning.note_debit(debit, p, 5, 5)
planning.note_debit(debit, p, -1, 3)
assert(debit.A == 4, 'a launch that spent nothing, or an unreadable count, changes nothing')
live.A = 2
p = planning.pick(role, count, debit)
assert(p.key == 'B', 'two rounds left cannot pay four: the next carrier of the role')
live.A, live.B = 0, 0
assert(planning.pick(role, count, debit) == nil, 'no carrier with ammunition: nothing is picked (the case is not attempted)')
live.A, live.B = -1, 3
assert(planning.pick(role, count, debit).key == 'B', 'an unreadable count is not ammunition')
print('COMBAT_PLAN=ok')
"#;

#[test]
fn the_case_carrier_is_one_that_can_afford_the_plan() {
    let lua_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("lua")
        .display()
        .to_string()
        .replace('\\', "/");
    let script = std::env::temp_dir().join(format!("eech-world-combat-plan-{}.lua", std::process::id()));
    std::fs::write(&script, COMBAT_PLAN_SCRIPT.replace("@LUA_DIR@", &lua_dir)).unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_eech-world")).arg(&script).output();
    std::fs::remove_file(&script).unwrap();
    let run = run.unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run.status.success(), "{}\n{stdout}\n{stderr}", run.status);
    assert!(stdout.contains("COMBAT_PLAN=ok"), "{stdout}\n{stderr}");
}

// CALLER-LEVEL test of the controlled witness's carrier choice (EEDCS-46, the 871 review): the REAL lua/combat-case.lua and its shared modules run against a small stand-in for the
// engine (inventory, launch cost, refusal and the report texts only; the C driver and its guards are checked elsewhere). A launch spends `@DEBIT@` rounds (a salvo of weapons, each
// spending its burst size), and one that cannot be afforded is refused (`refused=1`, ammunition unchanged), as the C driver reports it. Nothing here is the campaign's physics.
const COMBAT_CASE_CALLER_SCRIPT: &str = r#"
local LAUNCHERS = @LAUNCHERS@
local DEBIT = @DEBIT@
local ammo, launches, refused = {}, {}, {}
for _, l in ipairs(LAUNCHERS) do for _, i in ipairs(l.items) do ammo[l.id .. ':' .. i.weapon] = i.count end end
local time = 0
local engine = {}
function engine:clock() return { elapsed_seconds = time } end
function engine:frame(ms) time = time + ms / 1000 end
function engine:cmb_events() return '' end
function engine:cmb_trace() return 'cmb-blast calls=0\n' end
local LIVE = { ENTITY_TYPE_PERSON = 1, ENTITY_TYPE_PYLON = 1, ENTITY_TYPE_ROUTED_VEHICLE = 1 }
function engine:cmb_case_state(op, kind, ordinal, percent)
  if op == 0 then
    local lines = {}
    for _, k in ipairs({ 'ANTI_AIRCRAFT', 'OBJECT', 'PERSON', 'PYLON', 'ROUTED_VEHICLE', 'SCENIC', 'SITE', 'SITE_UPDATABLE' }) do
      lines[#lines + 1] = 'cmb-case-census kind=ENTITY_TYPE_' .. k .. ' live=' .. (LIVE['ENTITY_TYPE_' .. k] or 0)
    end
    return table.concat(lines, '\n') .. '\n'
  end
  return 'cmb-case-health kind=' .. kind .. ' found=1 eligible=0\n'
end
function engine:wpn_case_query(id, w)
  for _, l in ipairs(LAUNCHERS) do
    if l.id == id then
      for _, i in ipairs(l.items) do
        if i.weapon == w then return 'wpn-case-state host_id=' .. id .. ' alive=1 weapon=' .. w .. ' weapon_count=' .. ammo[id .. ':' .. w] .. ' warhead=' .. i.warhead .. '\n' end
      end
      return 'wpn-case-state host_id=' .. id .. ' alive=1 weapon=' .. w .. ' weapon_count=0 warhead=-1\n'
    end
  end
  error('no such launcher ' .. id)
end
function engine:cmb_case_act(op, kind, ordinal, launcher, weapon, amount, kill_code, weaponless)
  if op == 1 and weaponless then
    return 'cmb-case-collide kind=' .. kind .. ' ordinal=0 live=1 found=1 weapon=0 weaponless=1 launched=0 refused=0 modifier=0 level_before=1 alive_before=1 launch=none\n'
  end
  local key = launcher .. ':' .. weapon
  launches[key] = (launches[key] or 0) + 1
  local head = op == 1 and ('cmb-case-collide kind=' .. kind .. ' ordinal=0 live=1 found=1 weapon=' .. weapon .. ' weaponless=0') or ('cmb-case-blast kind=' .. kind .. ' ordinal=0 live=1 found=1 placed=1 weapon=' .. weapon .. ' kill_code=' .. kill_code .. ' offset=5')
  if (ammo[key] or 0) < DEBIT then
    refused[key] = (refused[key] or 0) + 1
    return head .. ' launched=0 refused=1 launch=wpn-case-launch|ok\n'
  end
  ammo[key] = ammo[key] - DEBIT
  return head .. ' launched=1 refused=0 launch=wpn-case-launch|ok\n'
end
local bound = 0
host.bind_mobile = function() bound = bound + 1; return LAUNCHERS[bound].id end
package.loaded['eech_dc'] = {
  prepare_installation = function() end,
  boot = function(cfg) for _ = 1, #LAUNCHERS do cfg.bind_mobile({}) end return engine end,
}
dofile('@LUA_DIR@/combat-case.lua')
local keys = {}
for k in pairs(launches) do keys[#keys + 1] = k end
table.sort(keys)
for _, k in ipairs(keys) do print('LAUNCHES ' .. k .. ' calls=' .. launches[k] .. ' refused=' .. (refused[k] or 0) .. ' left=' .. ammo[k]) end
print('CALLER_DONE')
"#;

fn run_combat_case_caller(launchers: &str, debit: u32) -> (Vec<serde_json::Value>, String) {
    let lua_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("lua")
        .display()
        .to_string()
        .replace('\u{5c}', "/");
    let tag = format!("{}-{}", std::process::id(), debit);
    let script = std::env::temp_dir().join(format!("eech-world-combat-caller-{tag}.lua"));
    let out = std::env::temp_dir().join(format!("eech-world-combat-caller-{tag}.jsonl"));
    let text = COMBAT_CASE_CALLER_SCRIPT
        .replace("@LAUNCHERS@", launchers)
        .replace("@DEBIT@", &debit.to_string())
        .replace("@LUA_DIR@", &lua_dir);
    std::fs::write(&script, text).unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_eech-world"))
        .arg(&script)
        .arg("root=none")
        .arg(format!("out={}", out.display().to_string().replace('\u{5c}', "/")))
        .arg("warmup=1")
        .output();
    std::fs::remove_file(&script).unwrap();
    let run = run.unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout).to_string();
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run.status.success(), "{}\n{stdout}\n{stderr}", run.status);
    assert!(stdout.contains("CALLER_DONE"), "{stdout}\n{stderr}");
    let rows: Vec<serde_json::Value> = std::fs::read_to_string(&out)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    std::fs::remove_file(&out).unwrap();
    (rows, stdout)
}

fn rows_of<'a>(rows: &'a [serde_json::Value], event: &str) -> Vec<&'a serde_json::Value> {
    rows.iter().filter(|r| r["event"] == event).collect()
}

// the preferred high-explosive class (dual purpose) is held by a carrier of six rounds while a plain high-explosive carrier holds a hundred: the caller must keep the estimate through
// the class choice and take the affordable alternate of the SAME role, instead of the first class and then thirteen refusals
#[test]
fn the_case_caller_takes_the_affordable_alternate_class_of_its_role() {
    let launchers = "{ { id = 'eech-world:2690', items = { { weapon = 54, count = 6, warhead = 4 } } },\
 { id = 'eech-world:2700', items = { { weapon = 55, count = 100, warhead = 5 } } },\
 { id = 'eech-world:2784', items = { { weapon = 76, count = 40, warhead = 1 } } } }";
    let (rows, stdout) = run_combat_case_caller(launchers, 1);
    // BEHAVIOUR first: no launch the campaign refused for want of ammunition
    assert!(
        stdout.lines().filter(|l| l.starts_with("LAUNCHES")).all(|l| l.contains("refused=0")),
        "a launch was refused for want of ammunition: {stdout}"
    );
    let chosen = rows_of(&rows, "chosen_weapon_class");
    let he = chosen.iter().find(|r| r["role"] == "high_explosive").expect("a high explosive class is chosen");
    assert_eq!(he["class"], "he_plain", "the affordable alternate, not the preferred class of six rounds: {he}");
    assert_eq!(he["reason"], "first_affordable_alternate");
    assert_eq!(he["preferred"], "he_dual_purpose");
    assert_eq!(he["estimated_shortfall"], 0);
    // the per-class rows keep the count and the estimated shortfall of the unaffordable class
    let classes = rows_of(&rows, "weapon_class");
    let dual = classes.iter().find(|r| r["class"] == "he_dual_purpose").unwrap();
    assert_eq!(
        (dual["count"].as_i64(), dual["planned"].as_i64(), dual["estimated_shortfall"].as_i64()),
        (Some(6), Some(10), Some(4)),
        "{dual}"
    );
    // no launch from the six-round carrier, no refusal, every launching case done
    assert!(!stdout.contains("LAUNCHES eech-world:2690:54"), "{stdout}");
    let cases = rows_of(&rows, "case");
    assert!(cases.iter().all(|r| r["refused"] == false), "no refusal: {stdout}");
    assert_eq!(cases.iter().filter(|r| r["done"] == true).count(), cases.len(), "every case done");
    assert!(stdout.contains("LAUNCHES eech-world:2700:55 calls=10 refused=0 left=90"), "{stdout}");
}

// a launch spends a SALVO: here four rounds. A carrier of ten rounds is not estimated short of the ten launch calls of the plan, so it is chosen, but only two launches fit; the cases
// must then pick the other carrier of the role from the LIVE count (and not be refused), and never launch from anything outside the role
#[test]
fn the_case_caller_follows_the_live_ammunition_when_a_launch_spends_several_rounds() {
    let launchers = "{ { id = 'eech-world:2690', items = { { weapon = 54, count = 10, warhead = 4 } } },\
 { id = 'eech-world:2700', items = { { weapon = 55, count = 100, warhead = 5 } } },\
 { id = 'eech-world:2784', items = { { weapon = 76, count = 40, warhead = 1 } } },\
 { id = 'eech-world:2800', items = { { weapon = 77, count = 500, warhead = 2 } } } }";
    let (rows, stdout) = run_combat_case_caller(launchers, 4);
    // BEHAVIOUR first: no launch the campaign refused for want of ammunition
    assert!(
        stdout.lines().filter(|l| l.starts_with("LAUNCHES")).all(|l| l.contains("refused=0")),
        "a launch was refused for want of ammunition: {stdout}"
    );
    let chosen = rows_of(&rows, "chosen_weapon_class");
    let he = chosen.iter().find(|r| r["role"] == "high_explosive").unwrap();
    assert_eq!(he["class"], "he_dual_purpose", "ten rounds are not estimated short of ten launch calls: {he}");
    assert_eq!(he["reason"], "preferred");
    let changed = rows_of(&rows, "carrier_changed");
    assert_eq!(
        changed.len(),
        1,
        "exactly one change, once the live count (2) cannot pay the learned cost (4): {stdout}"
    );
    assert_eq!(
        (changed[0]["from"].as_str(), changed[0]["to"].as_str()),
        (Some("he_dual_purpose"), Some("he_plain"))
    );
    let cases = rows_of(&rows, "case");
    assert!(cases.iter().all(|r| r["refused"] == false), "no case was refused: {stdout}");
    // the cases record the ammunition they spent
    let first = cases.iter().find(|r| r["name"].as_str().unwrap().ends_with("he_dual_purpose")).unwrap();
    assert_eq!((first["ammo_before"].as_i64(), first["ammo_after"].as_i64()), (Some(10), Some(6)), "{first}");
    assert!(stdout.contains("LAUNCHES eech-world:2690:54 calls=2 refused=0 left=2"), "{stdout}");
    assert!(stdout.contains("LAUNCHES eech-world:2700:55 calls=8 refused=0 left=68"), "{stdout}");
    // the carrier of an unrelated class (warhead 2, not part of any role) was never fired
    assert!(!stdout.contains("eech-world:2800"), "{stdout}");
}

// The controlled bridge witness (EEDCS-46, R5; lua/bridge-case.lua) against a small stand-in for the engine: only its flow and its argument safety are checked here (the survey, the choice of a bridge by
// the explicit criterion, the order of the operations, the frames recorded, the disarming, the plain statement that there was no suitable bridge, and that a malformed argument is refused BEFORE the
// installation is prepared, the engine booted, a frame run or a segment killed); the C driver and the observer are compiled and checked elsewhere, and the original code's behaviour is the ROOT's
// native run. The script under test is the real file, run by the real host's Lua. The stand-in notes every call it receives in a file (so that a refusal can be shown to have touched nothing) and stops
// the process with an error after 3000 frames or 200 case calls, so that a script without a bound cannot hang a test; this is a proof of argument validation, not of native physics.
const BRIDGE_CASE_CALLER_SCRIPT: &str = r#"
local SURVEY = [==[@SURVEY@]==]
-- the frame count from which the stand-in's segment has left the update list (-1: it never does); and whether the state it returns is empty
local FINISH = @FINISH@
local EMPTY = @EMPTY@
local calls = {}
local time = 0
local frames = 0
local function note(text)
  local f = io.open('@TRACE@', 'a')
  f:write(text, '\n')
  f:close()
end
local engine = {}
function engine:clock() return { elapsed_seconds = time } end
function engine:frame(ms)
  frames = frames + 1
  if frames > 3000 then error('stand-in cap: more than 3000 frames were run') end
  note('frame ' .. ms)
  time = time + ms / 1000
end
function engine:cmb_events() return 'cmb-ev seq=1 kind=bridge-segment-update bridge=0 seg=1 of=3 sub=2 alive=0 shape=5 y=10 a=0 b=0 f=0\n' end
function engine:cmb_trace() return 'cmb-blast calls=0\n' end
function engine:cmb_case_bridge(op, ordinal, index, criterion)
  if #calls > 200 then error('stand-in cap: more than 200 case calls') end
  calls[#calls + 1] = op .. ':' .. ordinal .. ':' .. index .. ':' .. criterion
  note('case ' .. op .. ':' .. ordinal .. ':' .. index .. ':' .. criterion)
  if op == 0 then return SURVEY end
  if op == 1 then
    if EMPTY then return '' end
    local pending = (FINISH >= 0 and frames >= FINISH) and 0 or 1
    return 'cmb-bridge-state bridge=' .. ordinal .. ' alive=1 breaks=0\ncmb-bridge-segment bridge=' .. ordinal .. ' index=0 kind=E alive=1 in_update=' .. pending .. '\ncmb-bridge-segment bridge=' .. ordinal .. ' index=1 kind=S alive=0 in_update=0\n'
  end
  if op == 2 then return 'cmb-bridge-fail bridge=' .. ordinal .. ' found=1 segments=3 index=' .. (index < 0 and 1 or index) .. ' criterion=0 done=1 bridge_alive=1 neighbour=2 neighbour_alive=1 first_collapse=1 armed=1\n' end
  return 'cmb-bridge-disarm armed=0\n'
end
package.loaded['eech_dc'] = {
  prepare_installation = function() note('prepare') end,
  boot = function() note('boot') return engine end,
}
dofile('@LUA_DIR@/bridge-case.lua')
print('CALLS ' .. table.concat(calls, ' '))
print('BRIDGE_CALLER_DONE')
"#;

struct BridgeRun {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
    trace: String,
    out_exists: bool,
    rows: Vec<serde_json::Value>,
}

fn run_bridge_case_raw(survey: &str, extra: &[&str], base: &[&str]) -> BridgeRun {
    run_bridge_case_with(survey, extra, base, -1, false)
}

// `finish`: the frame count from which the stand-in's segment is out of the update list (-1: never); `empty`: the state the stand-in returns is empty
fn run_bridge_case_with(survey: &str, extra: &[&str], base: &[&str], finish: i32, empty: bool) -> BridgeRun {
    let lua_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("lua")
        .display()
        .to_string()
        .replace('\u{5c}', "/");
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let tag = format!("{}-{}", std::process::id(), COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
    let script = std::env::temp_dir().join(format!("eech-world-bridge-caller-{tag}.lua"));
    let out = std::env::temp_dir().join(format!("eech-world-bridge-caller-{tag}.jsonl"));
    let trace = std::env::temp_dir().join(format!("eech-world-bridge-caller-{tag}.trace"));
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&trace);
    let text = BRIDGE_CASE_CALLER_SCRIPT
        .replace("@SURVEY@", survey)
        .replace("@FINISH@", &finish.to_string())
        .replace("@EMPTY@", if empty { "true" } else { "false" })
        .replace("@TRACE@", &trace.display().to_string().replace('\u{5c}', "/"))
        .replace("@LUA_DIR@", &lua_dir);
    std::fs::write(&script, text).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_eech-world"));
    command
        .arg(&script)
        .arg("root=none")
        .arg(format!("out={}", out.display().to_string().replace('\u{5c}', "/")));
    // a base argument is replaced by an extra one of the same name (the host does not say which of two it keeps)
    let key = |arg: &str| arg.split('=').next().unwrap_or_default().to_string();
    let overridden: Vec<String> = extra.iter().map(|a| key(a)).collect();
    for arg in base.iter().filter(|a| !overridden.contains(&key(a))).chain(extra) {
        command.arg(arg);
    }
    let run = command.output().unwrap();
    std::fs::remove_file(&script).unwrap();
    let out_exists = out.exists();
    let rows = if out_exists {
        std::fs::read_to_string(&out)
            .unwrap()
            .lines()
            // every line of the output must parse: a malformed row is a failure here, never silently dropped
            .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("malformed output row {l:?}: {e}")))
            .collect()
    } else {
        Vec::new()
    };
    let trace_text = std::fs::read_to_string(&trace).unwrap_or_default();
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&trace);
    BridgeRun {
        status: run.status,
        stdout: String::from_utf8_lossy(&run.stdout).to_string(),
        stderr: String::from_utf8_lossy(&run.stderr).to_string(),
        trace: trace_text,
        out_exists,
        rows,
    }
}

const BRIDGE_BASE: [&str; 3] = ["warmup=0.2", "frame_ms=100", "window_seconds=0.25"];

fn run_bridge_case_caller(survey: &str, extra: &[&str]) -> (Vec<serde_json::Value>, String) {
    let run = run_bridge_case_raw(survey, extra, &BRIDGE_BASE);
    assert!(run.status.success(), "{}\n{}\n{}", run.status, run.stdout, run.stderr);
    assert!(
        run.stdout.contains("BRIDGE_CASE_") || run.stdout.contains("BRIDGE_CALLER_DONE"),
        "{}\n{}",
        run.stdout,
        run.stderr
    );
    (run.rows, run.stdout)
}

const BRIDGE_SURVEY_ONE: &str = "cmb-bridge-census bridges=2 routed_alive=3 repair_tasks=0 comms_server=1\n\
cmb-bridge-survey bridge=0 sub_type=1 start=4 end=5 alive=1 breaks=0 segments=3 unsupported=0 first_candidate=-1 first_neighbour=-1 x=1 y=2 z=3 pattern=ESE\n\
cmb-bridge-survey bridge=1 sub_type=2 start=6 end=7 alive=1 breaks=0 segments=3 unsupported=1 first_candidate=0 first_neighbour=2 x=1 y=2 z=3 pattern=ESU\n";

#[test]
fn the_bridge_witness_records_a_declared_failure_and_a_bounded_window_of_frames_in_order() {
    let (rows, stdout) = run_bridge_case_caller(BRIDGE_SURVEY_ONE, &[]);
    // the survey, then the state, the failure of the explicit choice (the first bridge with a candidate, the first segment by the criterion), the state after, three frames of state, and the disarming
    assert!(
        stdout.contains("CALLS 0:0:-1:0 1:1:-1:0 2:1:-1:0 1:1:-1:0 1:1:-1:0 1:1:-1:0 1:1:-1:0 3:0:-1:0"),
        "{stdout}"
    );
    let case = rows.iter().find(|r| r["event"] == "bridge_case").unwrap();
    assert_eq!(
        (
            case["done"].as_bool(),
            case["bridge"].as_i64(),
            case["segment"].as_i64(),
            case["first_collapse"].as_bool()
        ),
        (Some(true), Some(1), Some(1), Some(true)),
        "{case}"
    );
    let scopes: Vec<&str> = rows.iter().filter_map(|r| r["scope"].as_str()).collect();
    for scope in [
        "bridge:survey",
        "bridge:before",
        "bridge:after",
        "frame:0:log",
        "frame:1:state",
        "frame:3:log",
        "bridge:disarm",
        "bridge:trace:before",
        "bridge:trace:after",
    ] {
        assert!(scopes.contains(&scope), "{scope} missing: {scopes:?}");
    }
    assert!(!scopes.contains(&"frame:4:state"), "the window is bounded at three frames: {scopes:?}");
    // the order: the failure is recorded before the first frame, and the disarming after the last
    let position = |scope: &str| scopes.iter().position(|s| *s == scope).unwrap();
    assert!(
        position("bridge:before") < position("frame:0:log")
            && position("frame:0:log") < position("frame:1:state")
            && position("frame:3:log") < position("bridge:disarm")
    );
    let event = rows.iter().find(|r| r["scope"] == "frame:1:log" && r["event"] == "cmb-ev").unwrap();
    assert_eq!(event["kind"], "bridge-segment-update");
    // what the original did is COUNTED from the events and the last state, not read from `done`: four logs of one segment-update each, no landing, one segment still in the update list
    let summary = rows.iter().find(|r| r["event"] == "bridge_summary").unwrap();
    assert_eq!(summary["n_segment_update"], 4, "{summary}");
    assert_eq!(
        (
            summary["in_update_at_end"].as_i64(),
            summary["landed_all"].as_bool(),
            summary["frames"].as_i64()
        ),
        (Some(1), Some(false), Some(3)),
        "{summary}"
    );
    assert!(summary.get("n_landed").is_none(), "no landing was logged, so none is claimed: {summary}");
}

#[test]
fn the_bridge_witness_says_so_and_does_nothing_when_no_bridge_meets_the_first_collapse_criterion() {
    let survey = "cmb-bridge-census bridges=1 routed_alive=3 repair_tasks=0 comms_server=1\n\
cmb-bridge-survey bridge=0 sub_type=1 start=4 end=5 alive=1 breaks=0 segments=3 unsupported=0 first_candidate=-1 first_neighbour=-1 x=1 y=2 z=3 pattern=ESE\n";
    let (rows, stdout) = run_bridge_case_caller(survey, &[]);
    assert!(
        // the whole logical line (LF or CRLF, as the platform writes it): the survey was the only call
        stdout.lines().any(|line| line == "CALLS 0:0:-1:0") && stdout.contains("BRIDGE_CASE_NOT_DONE"),
        "only the survey was asked, and the script said it did nothing: {stdout}"
    );
    let case = rows.iter().find(|r| r["event"] == "bridge_case").unwrap();
    assert_eq!(case["done"], false);
    assert_eq!(case["reason"], "no_bridge_meets_the_first_collapse_criterion");
    // and an explicit bridge that has none is not replaced by another
    let (rows, stdout) = run_bridge_case_caller(BRIDGE_SURVEY_ONE, &["bridge=0"]);
    let case = rows.iter().find(|r| r["event"] == "bridge_case").unwrap();
    assert_eq!(case["done"], false, "{stdout}");
    assert_eq!(case["reason"], "the_asked_bridge_does_not_meet_the_first_collapse_criterion");
}

#[test]
fn the_bridge_witness_accepts_an_explicit_valid_selector_and_the_edge_values_and_passes_them_on_exactly() {
    // an explicit bridge and segment are passed to the case as given; zero warmup and a zero window are valid (no frame at all)
    let run = run_bridge_case_raw(
        BRIDGE_SURVEY_ONE,
        &["bridge=1", "segment=2", "frame_ms=50", "warmup=0", "window_seconds=0", "criterion=0"],
        &[],
    );
    assert!(run.status.success(), "{}\n{}", run.stdout, run.stderr);
    assert!(run.stdout.contains("CALLS 0:0:-1:0 1:1:-1:0 2:1:2:0 1:1:-1:0 3:0:-1:0"), "{}", run.stdout);
    assert!(!run.trace.contains("frame"), "no frame was run: {}", run.trace);
    // with no frame, the summary is read from the state right after the failure, in which the killed segment is still in the update list: nothing has landed, and it must not say so
    let summary = bridge_summary(&run.rows);
    assert_eq!(
        (
            summary["done"].as_bool(),
            summary["frames"].as_i64(),
            summary["in_update_at_end"].as_i64(),
            summary["landed_all"].as_bool(),
            summary["state_valid"].as_bool()
        ),
        (Some(true), Some(0), Some(1), Some(false), Some(true)),
        "{summary}"
    );
    // the largest values the witness accepts (a frame of a second, ten minutes of both budgets within the frame limit)
    let run = run_bridge_case_raw(
        BRIDGE_SURVEY_ONE,
        &["bridge=100000", "segment=255", "frame_ms=1000", "warmup=600", "window_seconds=600"],
        &[],
    );
    assert!(run.status.success(), "{}\n{}", run.stdout, run.stderr);
}

// A malformed argument is refused BEFORE the output file is created, the installation prepared, the engine booted, a frame run or a segment killed, and is never turned into a default (the stand-in
// stops a script that does not stop). `segment=abc` used to become the automatic choice and kill the first qualifying segment.
#[test]
fn the_bridge_witness_refuses_a_malformed_argument_before_anything_is_prepared_booted_run_or_killed() {
    let cases: &[(&[&str], &str)] = &[
        (&["frame_ms=0", "warmup=5"], "frame_ms"),
        (&["frame_ms=-100"], "frame_ms"),
        (&["frame_ms=100.5"], "frame_ms"),
        (&["frame_ms=abc"], "frame_ms"),
        (&["frame_ms="], "frame_ms"),
        (&["frame_ms=1001"], "frame_ms"),
        (&["frame_ms=10x"], "frame_ms"),
        (&["warmup=-1"], "warmup"),
        (&["warmup=nan"], "warmup"),
        (&["warmup=inf"], "warmup"),
        (&["warmup=1e3"], "warmup"),
        (&["warmup=1e999"], "warmup"),
        (&["warmup=0x10"], "warmup"),
        (&["warmup=5s"], "warmup"),
        (&["warmup=601"], "warmup"),
        (&["warmup=abc"], "warmup"),
        (&["window_seconds=-1"], "window_seconds"),
        (&["window_seconds=nan"], "window_seconds"),
        (&["window_seconds=inf"], "window_seconds"),
        (&["window_seconds=100000"], "window_seconds"),
        (&["window_seconds=12.5.1"], "window_seconds"),
        (&["window_seconds=abc"], "window_seconds"),
        (&["frame_ms=1", "window_seconds=600"], "frames"),
        (&["frame_ms=1", "warmup=600"], "frames"),
        (&["segment=abc"], "segment"),
        (&["segment=-1"], "segment"),
        (&["segment=1.5"], "segment"),
        (&["segment=256"], "segment"),
        (&["segment=99999999999999"], "segment"),
        (&["segment="], "segment"),
        (&["segment=1x"], "segment"),
        (&["bridge=abc"], "bridge"),
        (&["bridge=-1"], "bridge"),
        (&["bridge=2.5"], "bridge"),
        (&["bridge=100001"], "bridge"),
        (&["bridge="], "bridge"),
        (&["criterion=1"], "criterion"),
        (&["criterion=abc"], "criterion"),
        (&["criterion=0.0"], "criterion"),
        (&["criterion=-0"], "criterion"),
        (&["scenario=mars"], "scenario"),
    ];
    for (extra, name) in cases {
        let run = run_bridge_case_raw(BRIDGE_SURVEY_ONE, extra, &BRIDGE_BASE);
        assert!(!run.status.success(), "{extra:?} was accepted:\n{}\n{}", run.stdout, run.stderr);
        assert!(
            run.stderr.contains("bridge-case:") && run.stderr.contains(name),
            "{extra:?}: the refusal does not name {name}: {}",
            run.stderr
        );
        assert!(run.trace.is_empty(), "{extra:?}: something ran before the refusal: {}", run.trace);
        assert!(!run.out_exists, "{extra:?}: the output file was created");
        assert!(!run.stdout.contains("CALLS"), "{extra:?}: the script ran on: {}", run.stdout);
    }
}

// The defect in its own words: an explicit segment that is not a number used to become the automatic choice, and the first qualifying segment of the first bridge was killed instead of the one asked
// for. The stand-in records the case calls, so the kill (operation 2) is what must not happen.
#[test]
fn the_bridge_witness_never_turns_a_segment_that_is_not_a_number_into_the_automatic_choice() {
    let run = run_bridge_case_raw(BRIDGE_SURVEY_ONE, &["segment=abc"], &BRIDGE_BASE);
    assert!(
        !run.trace.contains("case 2:"),
        "a segment was killed for segment=abc (status {}):\n{}\n{}",
        run.status,
        run.trace,
        run.stdout
    );
    assert!(!run.status.success() && run.stderr.contains("segment"), "{}\n{}", run.stdout, run.stderr);
}

// the one `bridge_summary` row of a run, found in the whole of its (fully parsed) output
fn bridge_summary(rows: &[serde_json::Value]) -> &serde_json::Value {
    let found: Vec<&serde_json::Value> = rows.iter().filter(|r| r["event"] == "bridge_summary").collect();
    assert_eq!(found.len(), 1, "exactly one bridge_summary row: {rows:?}");
    found[0]
}

// The summary says the decks are finished only when a state that was actually returned says so: a window of zero seconds, a window in which the segment is still in the update list, a window in which
// it has left it (the positive control), and a state that came back empty (which is no state at all, and counts as nothing).
#[test]
fn the_bridge_summary_counts_a_segment_as_finished_only_from_a_state_that_really_says_so() {
    let args = ["bridge=1", "frame_ms=100", "warmup=0"];
    // window, finish after, empty state, then the expected frames, in_update_at_end, landed_all, state_valid
    let cases: &[(&str, i32, bool, i64, i64, bool, bool)] = &[
        ("window_seconds=0", 5, false, 0, 1, false, true),
        ("window_seconds=0.3", -1, false, 3, 1, false, true),
        ("window_seconds=0.3", 2, false, 3, 0, true, true),
        ("window_seconds=0.3", -1, true, 3, -1, false, false),
        ("window_seconds=0", -1, true, 0, -1, false, false),
    ];
    for (window, finish, empty, frames, pending, landed, valid) in cases {
        let mut extra = args.to_vec();
        extra.push(window);
        let run = run_bridge_case_with(BRIDGE_SURVEY_ONE, &extra, &[], *finish, *empty);
        assert!(run.status.success(), "{window} {finish} {empty}:\n{}\n{}", run.stdout, run.stderr);
        let summary = bridge_summary(&run.rows);
        assert_eq!(
            (
                summary["frames"].as_i64(),
                summary["in_update_at_end"].as_i64(),
                summary["landed_all"].as_bool(),
                summary["state_valid"].as_bool()
            ),
            (Some(*frames), Some(*pending), Some(*landed), Some(*valid)),
            "{window} finish {finish} empty {empty}: {summary}"
        );
    }
}

// The controlled object-report witness (EEDCS-46, R3; lua/object-report-case.lua) against a small stand-in for the engine: only its flow and its argument safety are checked here (the census, the choice of the
// first alive structure of the kind, the report made once and by its identity, the bounded window, the state read afterwards, the control that ends the run, and that a malformed argument is refused BEFORE
// the engine is booted).  Whether a real report changes the campaign's state is the native run's to show, through the real handlers.
const OBJECT_CASE_CALLER_SCRIPT: &str = r#"
local time = 0
local function note(text)
  local f = io.open('@TRACE@', 'a')
  f:write(text, '\n')
  f:close()
end
local structures = {
  { host_id = 'obj-1', kind = 'pylon', sub_type = 2, side = 'neutral', alive = true, damage_level = 40, initial_damage_level = 40, position = { 1, 2, 3 } },
  { host_id = 'obj-2', kind = 'site', sub_type = 5, side = 'blue', alive = false, damage_level = 0, initial_damage_level = 90, position = { 4, 5, 6 } },
  { host_id = 'obj-3', kind = 'site', sub_type = 5, side = 'blue', alive = true, damage_level = 90, initial_damage_level = 90, position = { 7, 8, 9 } },
  { host_id = 'obj-4', kind = 'site', sub_type = 6, side = 'red', alive = true, damage_level = 60, initial_damage_level = 60, position = { 7, 8, 9 } },
}
local engine = {}
function engine:clock() return { elapsed_seconds = time } end
function engine:frame(ms) time = time + ms / 1000 note('frame') end
function engine:structures()
  note('structures')
  local copy = {}
  for i, s in ipairs(structures) do
    local c = {}
    for k, v in pairs(s) do c[k] = v end
    copy[i] = c
  end
  return copy
end
function engine:report_object(report)
  local found
  for _, s in ipairs(structures) do if s.host_id == report.host_id then found = s end end
  if found == nil then error('host objects: the identity "' .. report.host_id .. '" was never issued', 0) end
  if not found.alive then error('host objects: "' .. report.host_id .. '" is already destroyed: a fact is applied once', 0) end
  local before = found.damage_level
  if report.damage_level ~= nil then
    if report.damage_level > before then error('host objects: a damage level of ' .. report.damage_level .. ' cannot be repaired by a report', 0) end
    found.damage_level = report.damage_level
  end
  local killed = report.destroyed == true or (report.damage_level ~= nil and report.damage_level <= 0)
  if killed then found.alive = false end
  note('report ' .. report.host_id .. ' ' .. tostring(report.damage_level) .. ' ' .. tostring(report.destroyed))
  return { entity_type = 5, rostered = false, damage_level_before = before, damage_level_after = found.damage_level, killed = killed }
end
package.loaded['eech_dc'] = {
  prepare_installation = function() note('prepare') end,
  boot = function() note('boot') return engine end,
}
dofile('@LUA_DIR@/object-report-case.lua')
print('OBJECT_CALLER_DONE')
"#;

fn run_object_case(extra: &[&str]) -> (std::process::ExitStatus, String, String, String, Vec<serde_json::Value>) {
    let lua_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("lua")
        .display()
        .to_string()
        .replace('\u{5c}', "/");
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let tag = format!("{}-{}", std::process::id(), COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
    let script = std::env::temp_dir().join(format!("eech-world-object-caller-{tag}.lua"));
    let out = std::env::temp_dir().join(format!("eech-world-object-caller-{tag}.jsonl"));
    let trace = std::env::temp_dir().join(format!("eech-world-object-caller-{tag}.trace"));
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&trace);
    let text = OBJECT_CASE_CALLER_SCRIPT
        .replace("@TRACE@", &trace.display().to_string().replace('\u{5c}', "/"))
        .replace("@LUA_DIR@", &lua_dir);
    std::fs::write(&script, text).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_eech-world"));
    command
        .arg(&script)
        .arg("root=none")
        .arg(format!("out={}", out.display().to_string().replace('\u{5c}', "/")))
        .arg("warmup=0.2")
        .arg("frame_ms=100")
        .arg("window_seconds=0.25");
    for arg in extra {
        command.arg(arg);
    }
    let run = command.output().unwrap();
    std::fs::remove_file(&script).unwrap();
    let rows = if out.exists() {
        std::fs::read_to_string(&out)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("malformed output row {l:?}: {e}")))
            .collect()
    } else {
        Vec::new()
    };
    let trace_text = std::fs::read_to_string(&trace).unwrap_or_default();
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&trace);
    (
        run.status,
        String::from_utf8_lossy(&run.stdout).to_string(),
        String::from_utf8_lossy(&run.stderr).to_string(),
        trace_text,
        rows,
    )
}

#[test]
fn the_object_witness_reports_a_destruction_by_identity_once_and_reads_the_state_afterwards() {
    let (status, stdout, stderr, trace, rows) = run_object_case(&["kind=site", "report=destroyed"]);
    assert!(status.success() && stdout.contains("OBJECT_CASE_DONE"), "{stdout}\n{stderr}");
    let census = rows.iter().find(|r| r["event"] == "object_census").unwrap();
    assert_eq!((census["structures"].as_i64(), census["duplicate_identities"].as_i64()), (Some(4), Some(0)));
    // the first ALIVE structure of the kind: obj-2 is dead and is passed over
    let before = rows.iter().find(|r| r["event"] == "object_before").unwrap();
    assert_eq!((before["host_id"].as_str(), before["alive"].as_bool()), (Some("obj-3"), Some(true)));
    assert_eq!(trace.matches("report obj-3 nil true").count(), 1, "{trace}");
    assert_eq!(trace.matches("report ").count(), 1, "{trace}");
    let receipt = rows.iter().find(|r| r["event"] == "object_receipt").unwrap();
    assert_eq!(receipt["killed"].as_bool(), Some(true));
    let summary = rows.iter().find(|r| r["event"] == "object_summary").unwrap();
    assert_eq!(
        (
            summary["done"].as_bool(),
            summary["changed"].as_bool(),
            summary["alive_before"].as_bool(),
            summary["alive_after"].as_bool()
        ),
        (Some(true), Some(true), Some(true), Some(false)),
        "{summary}"
    );
    // the report is made before the window of frames, and the state is read after it
    let first_report = trace.find("report obj-3").unwrap();
    let first_frame_after = trace[first_report..].find("frame").unwrap();
    assert!(first_frame_after > 0, "{trace}");
    // the last logical line of the trace (LF or CRLF, as the platform writes it) is the state read
    assert_eq!(trace.lines().last(), Some("structures"), "the state is read last: {trace}");
}

#[test]
fn the_object_witness_with_no_report_is_the_control_that_changes_nothing() {
    let (status, stdout, stderr, trace, rows) = run_object_case(&["kind=site", "report=none"]);
    assert!(status.success(), "{stdout}\n{stderr}");
    assert_eq!(trace.matches("report ").count(), 0, "a withheld report reports nothing: {trace}");
    let summary = rows.iter().find(|r| r["event"] == "object_summary").unwrap();
    assert_eq!(
        (summary["changed"].as_bool(), summary["alive_after"].as_bool()),
        (Some(false), Some(true)),
        "{summary}"
    );
    assert!(rows.iter().all(|r| r["event"] != "object_receipt"));
}

#[test]
fn the_object_witness_reports_a_damage_level_and_ends_a_run_with_the_control_that_must_be_refused() {
    let (status, stdout, stderr, trace, rows) = run_object_case(&["kind=pylon", "report=damage", "level=15"]);
    assert!(status.success(), "{stdout}\n{stderr}");
    assert!(trace.contains("report obj-1 15 nil"), "{trace}");
    let summary = rows.iter().find(|r| r["event"] == "object_summary").unwrap();
    assert_eq!(
        (
            summary["damage_before"].as_i64(),
            summary["damage_after"].as_i64(),
            summary["alive_after"].as_bool()
        ),
        (Some(40), Some(15), Some(true)),
        "{summary}"
    );
    // a default level is half the current one
    let (_, _, _, trace, _) = run_object_case(&["kind=pylon", "report=damage"]);
    assert!(trace.contains("report obj-1 20 nil"), "{trace}");
    for (control, fragment) in [
        ("unknown", "was never issued"),
        ("duplicate", "already destroyed"),
        ("raise", "cannot be repaired"),
    ] {
        let report = if control == "duplicate" {
            "report=destroyed"
        } else if control == "raise" {
            "report=damage"
        } else {
            "report=destroyed"
        };
        let (status, stdout, stderr, _, rows) = run_object_case(&["kind=site", report, &format!("control={control}")]);
        assert!(status.success(), "{control}: {stdout}\n{stderr}");
        let row = rows
            .iter()
            .find(|r| r["event"] == "object_control")
            .unwrap_or_else(|| panic!("{control}: no control row"));
        assert_eq!(row["refused"].as_bool(), Some(true), "{control}: {row}");
        assert!(row["error"].as_str().unwrap().contains(fragment), "{control}: {row}");
        // the control is the last row: nothing is recorded after a refusal
        assert_eq!(rows.last().unwrap()["event"], "object_control", "{control}");
    }
}

#[test]
fn the_object_witness_says_so_when_there_is_no_such_structure_and_refuses_a_malformed_argument_before_booting() {
    let (status, _, _, trace, rows) = run_object_case(&["kind=scenic"]);
    assert!(status.success(), "{trace}");
    let summary = rows.iter().find(|r| r["event"] == "object_summary").unwrap();
    assert_eq!(summary["done"].as_bool(), Some(false));
    assert_eq!(trace.matches("report ").count(), 0, "nothing was reported: {trace}");
    for (bad, what) in [
        ("kind=bridge", "kind must be one of"),
        ("report=heal", "report must be one of"),
        ("control=stale", "control must be one of"),
        ("level=-1", "level must be a whole number"),
        ("index=1.5", "index must be a whole number"),
        ("identity=magic", "identity must be one of"),
    ] {
        let (status, _, stderr, trace, rows) = run_object_case(&[bad]);
        assert!(!status.success(), "{bad} was accepted");
        assert!(stderr.contains(what), "{bad}: {stderr}");
        assert!(trace.is_empty() && rows.is_empty(), "{bad}: the engine was booted or a row written: {trace}");
    }
    let (status, _, stderr, trace, _) = run_object_case(&["control=duplicate", "report=none"]);
    assert!(
        !status.success() && stderr.contains("control=duplicate needs report=destroyed") && trace.is_empty(),
        "{stderr}"
    );
}

// The environment characterisation driver (EEDCS-51, lua/env-characterization.lua) run as the host binary runs it, against a stand-in engine. The checks protect its CONTRACT: arguments refused before
// anything runs, a genuine refusal without the fixture, a control run that asks for no witness, strings that stay strings, the emitted protocol checked for EVERY family (a marker-only, wrong-family,
// incomplete, duplicated, mistyped, mis-identified or fingerprint-inconsistent report is refused), a bounded periodic schedule, and the portable wiring below. The stand-in answers well-formed
// canned reports; nothing here says anything about the original code's weather, wind or clock. No retail input is used.
//
// PORTABILITY. The driver under test is embedded in this executable (include_str) and written to a per-run temporary directory, so a test executable cross-built in a container needs no source
// tree beside it. The host binary is the ordinary Cargo one (CARGO_BIN_EXE_eech-world) unless EECH_WORLD_TEST_HOST names another: a native Windows run of a cross-built test sets it to the verified
// host executable. A supplied value that is empty, relative, missing or not a file is REFUSED (the test panics naming it); it never falls back to the Cargo path, and a Cargo path that does not exist
// here is refused with the same instruction rather than skipped.
const ENV_DRIVER_LUA: &str = include_str!("../lua/env-characterization.lua");
const ENV_HOST_OVERRIDE: &str = "EECH_WORLD_TEST_HOST";

fn resolve_env_host(supplied: Option<std::ffi::OsString>, cargo_default: &str) -> Result<std::path::PathBuf, String> {
    match supplied {
        Some(value) => {
            if value.to_string_lossy().trim().is_empty() {
                return Err(format!("{ENV_HOST_OVERRIDE} is set but empty"));
            }
            let path = std::path::PathBuf::from(&value);
            if !path.is_absolute() {
                return Err(format!("{ENV_HOST_OVERRIDE}={} is not an absolute path", path.display()));
            }
            if !path.is_file() {
                return Err(format!("{ENV_HOST_OVERRIDE}={} is not an existing file", path.display()));
            }
            Ok(path)
        }
        None => {
            let path = std::path::PathBuf::from(cargo_default);
            if path.is_file() {
                Ok(path)
            } else {
                Err(format!(
                    "the Cargo-built host {} does not exist here (a cross-built test executable?): set {ENV_HOST_OVERRIDE} to the verified eech-world executable",
                    path.display()
                ))
            }
        }
    }
}

fn env_host() -> std::path::PathBuf {
    resolve_env_host(std::env::var_os(ENV_HOST_OVERRIDE), env!("CARGO_BIN_EXE_eech-world")).unwrap_or_else(|why| panic!("{why}"))
}

#[test]
fn the_environment_host_override_is_strict_and_never_falls_back() {
    let real = std::env::current_exe().unwrap();
    let real_text = real.display().to_string();
    // no override: the Cargo path, if it is here
    assert_eq!(resolve_env_host(None, &real_text).unwrap(), real);
    assert!(resolve_env_host(None, "/no/such/eech-world").unwrap_err().contains(ENV_HOST_OVERRIDE));
    // a valid override wins over any default
    assert_eq!(resolve_env_host(Some(real.clone().into_os_string()), "/no/such/eech-world").unwrap(), real);
    // an invalid override is refused even when the Cargo default is perfectly good
    let temp = std::env::temp_dir();
    let missing = temp.join("eech-world-no-such-host-executable");
    for bad in [
        std::ffi::OsString::new(),
        std::ffi::OsString::from("   "),
        std::ffi::OsString::from("eech-world"),
        std::ffi::OsString::from("./eech-world"),
        missing.clone().into_os_string(),
        temp.clone().into_os_string(),
    ] {
        let why = resolve_env_host(Some(bad.clone()), &real_text).unwrap_err();
        assert!(why.contains(ENV_HOST_OVERRIDE), "{bad:?}: {why}");
    }
}

// the stand-in engine, in three parts: the clock and frames, the fixture's per-family reports, and the wiring. The mutations damage ONE report after it is made, so every other report stays well formed.
const ENV_STAND_IN_BASE: &str = r#"
local time = 0
local JUMP = @JUMP@
local frames = 0
local function note(text)
  local f = io.open('@TRACE@', 'a')
  f:write(text, '\n')
  f:close()
end
local engine = {}
function engine:clock() return { elapsed_seconds = time } end
function engine:frame(ms)
  frames = frames + 1
  if frames > 60 then error('stand-in cap: more than 60 frames') end
  note('frame ' .. ms)
  time = time + ms * JUMP / 1000
end
function engine:objects() return { { id = 1, kind = 'helicopter', sub_type = 2, side = 'blue', alive = true, position = { 1.5, 2.5, 3.5 }, heading = 0.5, pitch = 0, roll = 0, ammo = 0, fuel = 0 } } end
"#;

const ENV_STAND_IN_FIXTURE: &str = r#"
local MUTATION = '@MUTATION@'
local TARGET_GROUP = @TARGET_GROUP@
local TARGET_INDEX = @TARGET_INDEX@
local O = '3f800000'
local FP = '0123456789abcdef'
-- ' key=word ...' for the names given; 'name:word' picks the word
local function ws(...)
  local parts = {}
  for _, k in ipairs({ ... }) do
    local name, word = k:match('^(.-):(%x+)$')
    parts[#parts + 1] = (name or k) .. '=' .. (word or O)
  end
  return table.concat(parts, ' ')
end
local function state(prefix, tag)
  return prefix .. '-time tag=' .. tag .. ' ' .. ws('start_time_w', 'elapsed_time_w', 'time_of_day_w') .. ' day=1 day_segment_type=1 ' .. ws('time_of_day_acceleration_w') .. ' skip_night_time=0 ' .. ws('time_of_day_resync_w') .. '\n'
    .. prefix .. '-weather tag=' .. tag .. ' local_weather_model=1 weather_increasing=1 ' .. ws('weather_radius_w') .. ' weather_mode=1 target_weather_mode=1 '
    .. ws('transitional_status_w', 'transitional_period_w', 'lightning_timer_w', 'weather_pos_x_w', 'weather_pos_y_w', 'weather_pos_z_w', 'weather_vel_x_w', 'weather_vel_y_w', 'weather_vel_z_w') .. '\n'
    .. prefix .. '-wind tag=' .. tag .. ' wind_increasing=0 ' .. ws('wind_effect_radius_w', 'gusting_value_w:3f000000', 'minimum_speed_w', 'maximum_speed_w', 'direction_x_w', 'direction_y_w', 'direction_z_w',
      'effect_pos_x_w', 'effect_pos_y_w', 'effect_pos_z_w', 'effect_vel_x_w', 'effect_vel_y_w', 'effect_vel_z_w') .. '\n'
    .. prefix .. '-globals tag=' .. tag .. ' season=2 rng=-5 ' .. ws('rng_w', 'delta_time_w') .. ' comms_model=1 in_cockpit=0 camera=1 ' .. ws('camera_x_w', 'camera_y_w', 'camera_z_w') .. '\n'
end
local function query(event, tag, point)
  return event .. ' tag=' .. tag .. ' point=' .. point .. ' ' .. ws('x_w', 'y_w', 'z_w') .. ' cwm=1 twm=1 ' .. ws('ts_w') .. ' simple=1 ' .. ws('wind_speed_w', 'wind_x_w', 'wind_y_w', 'wind_z_w') .. '\n'
end
local function finish(group, index)
  return 'env-witness-end group=' .. group .. ' index=' .. index .. ' restored=1 fp_before=' .. FP .. ' fp_after=' .. FP .. '\n'
end
local function report(group, index)
  if group == 0 then
    return 'env-census version=1 session_bytes=1024 weather_cases=2 wind_cases=2 gust_cases=2 time_cases=3 init_cases=2 update_time_cases=1 update_weather_cases=2\n'
      .. 'env-census-enums dry=1 light_rain=2 heavy_rain=3 snow=4 season_summer=2 season_winter=3 season_desert=4 segment_dawn=0 segment_day=1 segment_dusk=2 segment_night=3 comms_server=1\n'
      .. 'env-census-constants ' .. ws('min_weather_radius_w', 'max_weather_radius_w', 'weather_expansion_rate_w', 'min_wind_radius_w', 'max_wind_radius_w', 'wind_expansion_rate_w', 'wind_gust_frequency_w',
        'session_resync_frequency_w', 'default_min_wind_speed_w', 'default_max_wind_speed_w', 'one_hour_w', 'one_day_w') .. '\n'
      .. 'env-census-configured ' .. ws('weather_rain_radius_min_w', 'weather_rain_radius_max_w', 'weather_rain_speed_w', 'weather_wind_radius_min_w', 'weather_wind_radius_max_w', 'weather_wind_speed_w') .. '\n'
      .. 'env-census-map ' .. ws('min_map_x_w', 'max_map_x_w', 'min_map_z_w', 'max_map_z_w', 'mid_map_x_w', 'mid_map_z_w') .. '\n'
      .. 'env-unsupported item=get_session_random_start_time_of_day reason=a_reason\n'
      .. 'env-unsupported item=update_lightning_effect reason=a_reason\n'
      .. 'env-unsupported item=update_session_sound_effects reason=a_reason\n'
      .. 'env-unsupported item=update_time_of_day_resync reason=a_reason\n'
      .. 'env-unsupported item=wind_gust_outside_0_1 reason=a_reason\n'
      .. 'env-unsupported item=session_state_loading_and_serialisation reason=a_reason\n'
  end
  if group == 1 then
    return state('env-state', 'retail') .. 'env-state-fingerprint tag=retail fp=' .. FP .. '\n' .. query('env-query', 'retail', 'camera') .. query('env-query', 'retail', 'map_centre')
      .. query('env-query', 'retail', 'weather_centre') .. query('env-query', 'retail', 'wind_centre')
  end
  if group == 2 then
    return 'env-weather kind=witness i=' .. index .. ' model=local season=2 ' .. ws('radius_w') .. ' in_cwm=1 in_twm=1 ' .. ws('in_ts_w', 'range_w') .. '\n' .. query('env-weather-answer', 'witness', 'case') .. finish(group, index)
  end
  if group == 3 then
    return 'env-wind kind=witness i=' .. index .. ' ' .. ws('radius_w', 'centre_x_w', 'centre_z_w', 'minimum_w', 'maximum_w', 'gust_w:3f000000', 'dir_x_w', 'dir_z_w', 'x_w', 'z_w', 'range_w', 'modulation_w',
      'speed_w:00000000', 'vx_w', 'vy_w', 'vz_w', 'null_speed_w:00000000') .. ' null_same=1\n' .. finish(group, index)
  end
  if group == 4 then
    if index == 1 then return 'env-gust kind=witness i=1 gust_w=3f800000 refused=original_asserts_outside_0_1\n' .. finish(group, index) end
    return 'env-gust kind=witness i=' .. index .. ' ' .. ws('gust_w:3f000000', 'modulation_w', 'calm_speed_w', 'calm_vx_w') .. '\n' .. finish(group, index)
  end
  if group == 5 then
    if index == 1 then return 'env-time kind=witness i=1 what=segment ' .. ws('tod_w') .. ' segment=1\n' .. finish(group, index) end
    if index == 2 then
      return 'env-time kind=witness i=2 what=constants ' .. ws('segment_time_dawn_w', 'segment_time_day_w', 'segment_time_dusk_w', 'segment_time_night_w', 'preset_dawn_w', 'preset_midday_w', 'preset_afternoon_w',
        'preset_dusk_w', 'preset_midnight_w') .. '\n' .. finish(group, index)
    end
    return 'env-time kind=witness i=0 what=calculate ' .. ws('start_w', 'elapsed_w', 'tod_w') .. ' rolled_days=0 ' .. ws('accessor_tod_w') .. ' accessor_day=1 segment=1\n' .. finish(group, index)
  end
  if group == 6 then
    local head = index == 0 and ('env-init kind=witness i=0 what=fixed mode=1 rng=-5\n') or ('env-init kind=witness i=1 what=random seed_in=1 rng=-5 rng_w=fffffffb\n')
    return head .. state('env-init-state', 'witness') .. query('env-init-query', 'witness', 'origin') .. query('env-init-query', 'witness', 'map_centre') .. finish(group, index)
  end
  if group == 7 then
    return 'env-update-time kind=witness i=0 label=midday_real_time ' .. ws('delta_w', 'accel_w') .. ' skip_night=0 ' .. ws('tod_before_w') .. ' day_before=1\n' .. state('env-update-time-state', 'before_0')
      .. 'env-update-time-result kind=witness i=0 ' .. ws('tod_after_w') .. ' day_after=1\n' .. state('env-update-time-state', 'after_0') .. finish(group, index)
  end
  if group == 8 then
    if index == 1 then
      return 'env-update-weather kind=witness i=1 refused=unavailable_in_cockpit_or_no_camera_or_not_server in_cockpit=1 camera=1 comms_model=1\n' .. finish(group, index)
    end
    return 'env-update-weather kind=witness i=0 label=local_interior ' .. ws('delta_w') .. ' camera_dependent_fields=weather_mode,target_weather_mode,transitional_status_when_local_and_radius_nonzero camera=1\n'
      .. state('env-update-weather-state', 'before_0') .. state('env-update-weather-state', 'after_0') .. finish(group, index)
  end
  error('stand-in: no such group ' .. group)
end
local mutations = {
  marker_only = function(t) return t:match('[^\n]*\n$') end,
  wrong_family = function() return (report(2, 0):gsub('group=2', 'group=3')) end,
  missing_field = function(t) return (t:gsub(' vz_w=%x+', '', 1)) end,
  duplicate_field = function(t) return (t:gsub('gust_w=(%x+)', 'gust_w=%1 gust_w=00000000', 1)) end,
  bad_word = function(t) return (t:gsub('speed_w=%x+', 'speed_w=3F80ABCD', 1)) end,
  short_word = function(t) return (t:gsub('speed_w=%x+', 'speed_w=3f80', 1)) end,
  fp_mismatch = function(t) return (t:gsub('fp_after=%x+', 'fp_after=fedcba9876543210')) end,
  restored_zero = function(t) return (t:gsub('restored=1', 'restored=0')) end,
  wrong_index = function(t) return (t:gsub('(env%-wind kind=witness i=)%d+', '%19')) end,
  wrong_end_index = function(t) return (t:gsub('index=%d+', 'index=7')) end,
  extra_row = function(t) return t:match('^[^\n]*\n') .. t end,
  unknown_field = function(t) return (t:gsub('(env%-wind [^\n]*)\n', '%1 extra=1\n', 1)) end,
  truncated = function(t) return t:sub(1, -2) end,
  empty_line = function(t) return (t:gsub('\n', '\n\n', 1)) end,
  gust_in_domain_refused = function(t) return (t:gsub('gust_w=%x+', 'gust_w=3f000000', 1)) end,
  gust_out_of_domain_called = function(t) return (t:gsub('gust_w=%x+', 'gust_w=3f800000', 1)) end,
  census_zero = function(t) return (t:gsub('weather_cases=%d+', 'weather_cases=0')) end,
  census_huge = function(t) return (t:gsub('weather_cases=%d+', 'weather_cases=100000')) end,
  census_enum = function(t) return (t:gsub('dry=1', 'dry=9')) end,
  census_missing_unsupported = function(t) return (t:gsub('env%-unsupported item=session[^\n]*\n', '')) end,
  snapshot_bad_fingerprint = function(t) return (t:gsub('fp=%x+', 'fp=abc')) end,
  -- guard-context contradictions: each claims a branch the state printed in the same report cannot have taken
  init_false_refusal = function() return 'env-init kind=witness i=1 refused=the_original_initialisers_assert_the_server_comms_model comms_model=1\n' .. finish(6, 1) end,
  init_false_run = function(t) return (t:gsub('comms_model=1', 'comms_model=2')) end,
  update_false_refusal = function(t) return (t:gsub('in_cockpit=1', 'in_cockpit=0')) end,
  update_false_run = function(t) return (t:gsub('in_cockpit=0', 'in_cockpit=1')) end,
  snapshot_false_unavailable = function(t) return (t:gsub('wind_speed_w=%x+ wind_x_w=%x+ wind_y_w=%x+ wind_z_w=%x+', 'wind=unavailable_gust_outside_0_1')) end,
  snapshot_gust_outside_with_wind = function(t) return (t:gsub('gusting_value_w=%x+', 'gusting_value_w=3f800000')) end,
  wind_flag_says_unequal = function(t) return (t:gsub('null_same=1', 'null_same=0')) end,
  wind_flag_says_equal = function(t) return (t:gsub('null_speed_w=%x+', 'null_speed_w=3f800000')) end,
  -- coherent ALTERNATIVE forms the emitter can print (accepted): unequal words with flag 0; a snapshot whose gust is outside [0,1) and so has no wind answers; refusals by each other guard;
  -- a controlled update whose state shows a gust phase of 1.0 before it wraps (not a query context, so not judged)
  wind_words_unequal_flag_zero = function(t) return ((t:gsub('null_speed_w=%x+', 'null_speed_w=3f800000'):gsub('null_same=1', 'null_same=0'))) end,
  snapshot_gust_outside_no_wind = function(t) return ((t:gsub('gusting_value_w=%x+', 'gusting_value_w=3f800000'):gsub('wind_speed_w=%x+ wind_x_w=%x+ wind_y_w=%x+ wind_z_w=%x+', 'wind=unavailable_gust_outside_0_1'))) end,
  init_refused_nonserver = function() return 'env-init kind=witness i=1 refused=the_original_initialisers_assert_the_server_comms_model comms_model=2\n' .. finish(6, 1) end,
  update_refused_no_camera = function() return 'env-update-weather kind=witness i=0 refused=unavailable_in_cockpit_or_no_camera_or_not_server in_cockpit=0 camera=0 comms_model=1\n' .. finish(8, 0) end,
  update_refused_nonserver = function() return 'env-update-weather kind=witness i=0 refused=unavailable_in_cockpit_or_no_camera_or_not_server in_cockpit=0 camera=1 comms_model=2\n' .. finish(8, 0) end,
  update_state_gust_phase_one = function(t) return (t:gsub('gusting_value_w=%x+', 'gusting_value_w=3f800000')) end,
}
function engine:env_case(group, index)
  note('case ' .. group .. ':' .. index)
  local text = report(group, index)
  if MUTATION ~= 'none' and group == TARGET_GROUP and (TARGET_INDEX < 0 or index == TARGET_INDEX) then text = mutations[MUTATION](text) end
  return text
end
"#;

const ENV_STAND_IN_WIRING: &str = r#"
package.loaded['eech_dc'] = {
  prepare_installation = function() note('prepare') end,
  boot = function() note('boot') return engine end,
}
dofile('@DRIVER@')
"#;

struct EnvStandIn {
    fixture: bool,
    jump: f64,
    mutation: &'static str,
    group: i32,
    index: i32,
}

impl EnvStandIn {
    const WELL_FORMED: EnvStandIn = EnvStandIn {
        fixture: true,
        jump: 1.0,
        mutation: "none",
        group: -1,
        index: -1,
    };
}

struct EnvRun {
    status: std::process::ExitStatus,
    stderr: String,
    trace: String,
    out_exists: bool,
    out_text: String,
    rows: Vec<serde_json::Value>,
}

// a per-run temporary directory, removed when the run is over, however it ends
struct EnvTempDir(std::path::PathBuf);

impl Drop for EnvTempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn forward(path: &std::path::Path) -> String {
    path.display().to_string().replace('\u{5c}', "/")
}

// the host runs for at most this long; a driver that does not finish is killed and the test fails (a loop that does not end is a failure, not a hang)
const ENV_DRIVER_DEADLINE: std::time::Duration = std::time::Duration::from_secs(90);

fn run_env_scenario(stand_in: &EnvStandIn, args: &[&str]) -> EnvRun {
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let tag = format!("{}-{}", std::process::id(), COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
    let dir = EnvTempDir(std::env::temp_dir().join(format!("eech-world-env-driver-{tag}")));
    std::fs::create_dir_all(&dir.0).unwrap();
    let driver = dir.0.join("env-characterization.lua");
    let script = dir.0.join("stand-in.lua");
    let out = dir.0.join("out.jsonl");
    let trace = dir.0.join("trace.txt");
    std::fs::write(&driver, ENV_DRIVER_LUA).unwrap();
    let mut text = String::from(ENV_STAND_IN_BASE);
    if stand_in.fixture {
        text.push_str(ENV_STAND_IN_FIXTURE);
    }
    text.push_str(ENV_STAND_IN_WIRING);
    let text = text
        .replace("@JUMP@", &format!("{}", stand_in.jump))
        .replace("@MUTATION@", stand_in.mutation)
        .replace("@TARGET_GROUP@", &stand_in.group.to_string())
        .replace("@TARGET_INDEX@", &stand_in.index.to_string())
        .replace("@TRACE@", &forward(&trace))
        .replace("@DRIVER@", &forward(&driver));
    std::fs::write(&script, text).unwrap();

    let mut command = Command::new(env_host());
    command.arg(&script).arg("root=none").arg(format!("out={}", forward(&out)));
    // an argument given by the test replaces the same-named one of the defaults
    let key = |arg: &str| arg.split('=').next().unwrap_or_default().to_string();
    let defaults = ["seconds=1", "frame_ms=100", "snapshot_frames=2", "snapshot_every=0"];
    let overridden: Vec<String> = args.iter().map(|a| key(a)).collect();
    for arg in defaults.iter().filter(|a| !overridden.contains(&key(a))).chain(args.iter()) {
        command.arg(arg);
    }
    command.stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
    let mut child = command.spawn().unwrap();
    let started = std::time::Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if started.elapsed() > ENV_DRIVER_DEADLINE {
            let _ = child.kill();
            let _ = child.wait();
            panic!("the environment driver did not finish within {ENV_DRIVER_DEADLINE:?} and was killed");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    let output = child.wait_with_output().unwrap();
    let out_exists = out.exists();
    let out_text = if out_exists { std::fs::read_to_string(&out).unwrap() } else { String::new() };
    // every line must parse: a malformed row is a failure here, never silently dropped
    let rows = out_text
        .lines()
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("malformed output row {l:?}: {e}")))
        .collect();
    let trace_text = std::fs::read_to_string(&trace).unwrap_or_default();
    EnvRun {
        status,
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        trace: trace_text,
        out_exists,
        out_text,
        rows,
    }
}

fn run_env_driver(args: &[&str]) -> EnvRun {
    run_env_scenario(&EnvStandIn::WELL_FORMED, args)
}

fn env_rows<'a>(rows: &'a [serde_json::Value], event: &str) -> Vec<&'a serde_json::Value> {
    rows.iter().filter(|r| r["event"] == event).collect()
}

fn env_witness_calls(trace: &str) -> usize {
    trace
        .lines()
        .filter(|l| l.starts_with("case ") && !l.starts_with("case 0:") && !l.starts_with("case 1:"))
        .count()
}

#[test]
fn the_environment_driver_refuses_a_malformed_argument_before_anything_is_prepared_booted_or_run() {
    let cases: &[(&[&str], &str)] = &[
        (&["frame_ms=0"], "frame_ms"),
        (&["frame_ms=1001"], "frame_ms"),
        (&["frame_ms=10.5"], "frame_ms"),
        (&["seconds=-1"], "seconds"),
        (&["seconds=1e3"], "seconds"),
        (&["seconds=21601"], "seconds"),
        (&["seconds=nan"], "seconds"),
        (&["seed=0"], "seed"),
        (&["seed=2147483648"], "seed"),
        (&["seed=abc"], "seed"),
        (&["probes=1025"], "probes"),
        (&["probes=-1"], "probes"),
        (&["probes="], "probes"),
        (&["probe_at=2", "seconds=1"], "probe_at"),
        (&["probe_at=x"], "probe_at"),
        (&["snapshot_frames=2001"], "snapshot_frames"),
        (&["snapshot_every=3601"], "snapshot_every"),
        (&["snapshot_every=abc"], "snapshot_every"),
        (&["scenario=mars"], "scenario"),
        (&["frame_ms=1", "seconds=200"], "frames"),
    ];
    for (args, name) in cases {
        let run = run_env_driver(args);
        assert!(!run.status.success(), "{args:?} was accepted");
        assert!(
            run.stderr.contains("env-characterization:") && run.stderr.contains(name),
            "{args:?}: the refusal does not name {name}: {}",
            run.stderr
        );
        assert!(run.trace.is_empty(), "{args:?}: something ran before the refusal: {}", run.trace);
        assert!(!run.out_exists, "{args:?}: the output file was created");
    }
}

#[test]
fn the_environment_driver_refuses_a_module_without_the_fixture_and_runs_no_frame() {
    let run = run_env_scenario(
        &EnvStandIn {
            fixture: false,
            ..EnvStandIn::WELL_FORMED
        },
        &["probes=3"],
    );
    assert!(
        !run.status.success(),
        "a module without the fixture was reported as measured:\n{}",
        run.out_text
    );
    assert!(run.stderr.contains("env_case"), "{}", run.stderr);
    assert_eq!(env_rows(&run.rows, "refusal").len(), 1, "{}", run.out_text);
    assert!(env_rows(&run.rows, "run").is_empty(), "a refused run wrote a run row");
    assert!(!run.trace.contains("frame"), "a frame ran without the fixture: {}", run.trace);
}

#[test]
fn the_environment_driver_control_asks_for_no_witness_and_the_probe_run_keeps_words_as_strings_and_reports_its_coverage() {
    let control = run_env_driver(&["probes=0"]);
    assert!(control.status.success(), "{}\n{}", control.stderr, control.out_text);
    assert_eq!(env_witness_calls(&control.trace), 0, "the control asked for a witness: {}", control.trace);
    assert_eq!(env_rows(&control.rows, "boot")[0]["mode"], "control");

    // 14 witness cases exist in the stand-in's census; the cap of 4 truncates, and the coverage row says so
    let probe = run_env_driver(&["probes=4"]);
    assert!(probe.status.success(), "{}\n{}", probe.stderr, probe.out_text);
    assert_eq!(env_witness_calls(&probe.trace), 4, "{}", probe.trace);
    let winds = env_rows(&probe.rows, "env-wind");
    assert_eq!(winds.len(), 2);
    // a float word that is all decimal digits, and the all-zero word, stay strings; a plain integer is a number
    assert_eq!(winds[0]["speed_w"], "00000000");
    assert_eq!(winds[0]["vx_w"], "3f800000");
    let states = env_rows(&probe.rows, "env-state-globals");
    assert!(
        !states.is_empty() && states.iter().all(|r| r["rng"] == -5 && r["rng_w"] == "3f800000"),
        "{states:?}"
    );
    let snapshots = env_rows(&probe.rows, "env-state-fingerprint");
    assert!(snapshots.iter().all(|r| r["fp"] == "0123456789abcdef"), "{snapshots:?}");
    let noninterference = env_rows(&probe.rows, "probe_noninterference");
    assert_eq!(noninterference.len(), 1);
    assert_eq!(noninterference[0]["retail_equal"], true);
    let run = env_rows(&probe.rows, "run");
    assert_eq!(run.len(), 1);
    assert_eq!(run[0]["probes_run"], 4);
    assert_eq!(run[0]["cases_available"], 14);
    assert_eq!(run[0]["complete"], false);
    let coverage = env_rows(&probe.rows, "coverage");
    assert_eq!(coverage.len(), 7);
    assert!(coverage.iter().any(|c| c["truncated"] == true));
    assert!(probe.out_text.ends_with('\n'));
}

#[test]
fn the_environment_driver_accepts_every_family_in_its_declared_form_including_the_genuine_refusals_and_counts_them() {
    let run = run_env_driver(&["probes=512"]);
    assert!(run.status.success(), "{}\n{}", run.stderr, run.out_text);
    assert_eq!(env_witness_calls(&run.trace), 14, "{}", run.trace);
    let summary = &env_rows(&run.rows, "run")[0];
    assert_eq!(summary["complete"], true);
    assert_eq!(summary["probes_run"], 14);
    // the stand-in's gust case 1 and update-weather case 1 are refused in their declared forms: the report is accepted as such, and counted
    let validation = &env_rows(&run.rows, "validation")[0];
    assert_eq!(validation["witnesses"], 14);
    assert_eq!(validation["refused_witnesses"], 2);
    assert_eq!(env_rows(&run.rows, "env-gust").len(), 2);
    assert_eq!(env_rows(&run.rows, "env-gust").iter().filter(|r| r.get("refused").is_some()).count(), 1);
    assert_eq!(env_rows(&run.rows, "env-witness-end").iter().filter(|r| r["restored"] == 1).count(), 14);
}

#[test]
fn the_environment_driver_refuses_every_report_that_is_not_the_fixtures_protocol() {
    // (mutation, group, index of the damaged report, what the refusal must say). Every other report of the run is well formed, so only the damaged one can be the reason.
    let cases: &[(&'static str, i32, i32, &str)] = &[
        ("marker_only", 3, 0, "expected '"),
        ("wrong_family", 3, 0, "expected 'env-wind'"),
        ("missing_field", 3, 0, "fields"),
        ("duplicate_field", 3, 0, "duplicate field"),
        ("bad_word", 3, 0, "8-digit lowercase hex word"),
        ("short_word", 3, 0, "8-digit lowercase hex word"),
        ("fp_mismatch", 3, 0, "fingerprint changed"),
        ("restored_zero", 3, 0, "restored"),
        ("wrong_index", 3, 0, "for case i=9"),
        ("wrong_end_index", 3, 0, "end marker is for group"),
        ("extra_row", 3, 0, "expected '"),
        ("unknown_field", 3, 0, "fields"),
        ("truncated", 3, 0, "truncated"),
        ("empty_line", 3, 0, "empty line"),
        ("marker_only", 2, 0, "expected '"),
        ("marker_only", 6, 1, "expected '"),
        ("marker_only", 7, 0, "expected '"),
        ("marker_only", 8, 0, "expected '"),
        ("gust_in_domain_refused", 4, 1, "was refused"),
        ("gust_out_of_domain_called", 4, 0, "reported as called"),
        ("census_zero", 0, 0, "outside 1"),
        ("census_huge", 0, 0, "outside 1"),
        ("census_enum", 0, 0, "not a value the protocol allows"),
        ("census_missing_unsupported", 0, 0, "is missing"),
        ("snapshot_bad_fingerprint", 1, 0, "16-digit"),
        ("marker_only", 1, 0, "expected '"),
        // the printed guards: an impossible refusal hides the state rows, and an unavailable wind hides the answers, so each is judged by the context of the same report
        ("init_false_refusal", 6, 1, "refused although comms_model equals"),
        ("init_false_run", 6, 0, "ran although comms_model"),
        ("update_false_refusal", 8, 1, "refused although in_cockpit=0, camera=1"),
        ("update_false_run", 8, 0, "ran although in_cockpit"),
        ("snapshot_false_unavailable", 1, 0, "says wind unavailable although"),
        ("snapshot_gust_outside_with_wind", 1, 0, "has a wind answer although"),
        ("wind_flag_says_unequal", 3, 0, "null_same=0 contradicts"),
        ("wind_flag_says_equal", 3, 0, "null_same=1 contradicts"),
    ];
    for (mutation, group, index, says) in cases {
        let stand_in = EnvStandIn {
            mutation,
            group: *group,
            index: *index,
            ..EnvStandIn::WELL_FORMED
        };
        let run = run_env_scenario(&stand_in, &["probes=512"]);
        let what = format!("{mutation} on group {group} index {index}");
        assert!(!run.status.success(), "{what} was accepted:\n{}", run.out_text);
        assert!(
            run.stderr.contains("did not return the fixture's protocol") || run.stderr.contains("malformed report"),
            "{what}: {}",
            run.stderr
        );
        assert!(run.stderr.contains(*says), "{what}: the refusal does not say '{says}': {}", run.stderr);
        assert_eq!(env_rows(&run.rows, "failure").len(), 1, "{what}: {}", run.out_text);
        assert!(env_rows(&run.rows, "run").is_empty(), "{what}: a refused run reported a run row");
        assert!(env_rows(&run.rows, "validation").is_empty(), "{what}: a refused run reported validation");
    }
}

#[test]
fn the_environment_driver_bounds_its_periodic_schedule_whatever_the_period_and_the_clock_jump() {
    // a period of 1e-11 s and a clock that jumps 50 s in a frame: one snapshot per frame and no catch-up loop (the old loop needed about 5e12 steps per frame and never finished)
    let jump = EnvStandIn {
        jump: 500.0,
        ..EnvStandIn::WELL_FORMED
    };
    let run = run_env_scenario(
        &jump,
        &["probes=0", "snapshot_frames=0", "snapshot_every=.00000000001", "seconds=1", "frame_ms=100"],
    );
    assert!(run.status.success(), "{}\n{}", run.stderr, run.out_text);
    let periodic: Vec<_> = env_rows(&run.rows, "env-state-fingerprint")
        .into_iter()
        .filter(|r| r["scope"].as_str().unwrap_or("").starts_with("snapshot:periodic:"))
        .collect();
    assert_eq!(periodic.len(), 10, "{}", run.out_text);

    // the ordinary schedule is unchanged: the clock moves 0.25 s a frame; every 0.5 s gives a snapshot at 0.5, 1.0, 1.5, 2.0, 2.5; every 0.25 s one after every frame; 0 none
    let steady = EnvStandIn {
        jump: 2.5,
        ..EnvStandIn::WELL_FORMED
    };
    let scopes = |every: &str| -> Vec<String> {
        let run = run_env_scenario(&steady, &["probes=0", "snapshot_frames=0", every, "seconds=1", "frame_ms=100"]);
        assert!(run.status.success(), "{}\n{}", run.stderr, run.out_text);
        env_rows(&run.rows, "env-state-fingerprint")
            .into_iter()
            .filter_map(|r| r["scope"].as_str().map(str::to_string))
            .filter(|s| s.starts_with("snapshot:periodic:"))
            .collect()
    };
    assert_eq!(
        scopes("snapshot_every=0.5"),
        [
            "snapshot:periodic:0.500",
            "snapshot:periodic:1.000",
            "snapshot:periodic:1.500",
            "snapshot:periodic:2.000",
            "snapshot:periodic:2.500"
        ]
    );
    assert_eq!(scopes("snapshot_every=0.25").len(), 10);
    assert!(scopes("snapshot_every=0").is_empty());
}

#[test]
fn the_environment_driver_accepts_the_coherent_alternative_forms_the_emitter_can_print() {
    // (mutation, group, index, refused witnesses the run must then count). The well-formed stand-in refuses two witnesses (the gust and the update-weather case 1).
    let cases: &[(&'static str, i32, i32, u64)] = &[
        ("none", -1, -1, 2),
        // the emitter's null_same is the equality of its own two words: unequal words with flag 0 are as coherent as equal words with flag 1
        ("wind_words_unequal_flag_zero", 3, 0, 2),
        // a snapshot whose gust is outside [0,1) has no wind answers (the emitter's own branch)
        ("snapshot_gust_outside_no_wind", 1, 0, 2),
        // the initialiser and the weather update are refused by each guard that can hold
        ("init_refused_nonserver", 6, 1, 3),
        ("update_refused_no_camera", 8, 0, 3),
        ("update_refused_nonserver", 8, 0, 3),
        // a state record is not a query context: a gust phase of 1.0 before the wrap is accepted there
        ("update_state_gust_phase_one", 8, 0, 2),
    ];
    for (mutation, group, index, refused) in cases {
        let stand_in = EnvStandIn {
            mutation,
            group: *group,
            index: *index,
            ..EnvStandIn::WELL_FORMED
        };
        let run = run_env_scenario(&stand_in, &["probes=512"]);
        let what = format!("{mutation} on group {group} index {index}");
        assert!(run.status.success(), "{what} was refused: {}\n{}", run.stderr, run.out_text);
        assert!(env_rows(&run.rows, "failure").is_empty(), "{what}: {}", run.out_text);
        let validation = &env_rows(&run.rows, "validation")[0];
        assert_eq!(validation["witnesses"], 14, "{what}");
        assert_eq!(validation["refused_witnesses"], *refused, "{what}");
        assert_eq!(env_rows(&run.rows, "run")[0]["complete"], true, "{what}");
    }
}
