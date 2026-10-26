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
