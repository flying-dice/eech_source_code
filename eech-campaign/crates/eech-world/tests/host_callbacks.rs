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
