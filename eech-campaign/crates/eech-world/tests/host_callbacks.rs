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
local id = host.bind_mobile({ member_key = 1, group_key = 1, entity_id = 1, entity_type = 1, sub_type = 1, side = 1, pose = pose })
local function stop(host_id, from) return host.move_vehicle({ host_id = host_id, mode = 'stop', pose = from }) end
assert(stop(id, pose).velocity == 0)
local ok, err = pcall(stop, 'eech-world:999', pose)
assert(not ok and tostring(err):find('unknown host id eech-world:999', 1, true), tostring(err))
local moved = { position = { 1, 0, 0 }, velocity = 0, attitude = pose.attitude }
ok, err = pcall(stop, id, moved)
assert(not ok and tostring(err):find('differs from the host', 1, true), tostring(err))
ok, err = pcall(host.move_vehicle, { host_id = id, mode = 'glide', pose = pose })
assert(not ok and tostring(err):find('unknown mode glide', 1, true), tostring(err))
print('MOVE_VEHICLE_ERRORS=3')
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
