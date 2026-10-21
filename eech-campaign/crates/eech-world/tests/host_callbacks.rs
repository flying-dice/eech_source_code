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
