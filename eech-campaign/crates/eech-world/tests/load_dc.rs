//! The host loads the dynamic campaign DLL the way a simulator does: require.
use mlua::Lua;

/// A Lua state that finds the DC DLL with require, as a simulator host's does.
fn host_lua() -> Lua {
    // the test binary runs from <target>[/<triple>]/<profile>/deps, where cargo
    // also leaves the cdylib; the profile directory holds it after `cargo build`
    let deps = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let cpath: String = [deps.as_path(), deps.parent().unwrap()]
        .iter()
        .map(|d| {
            let d = d.display().to_string().replace('\\', "/");
            format!("{d}/lib?.so;{d}/?.dll;")
        })
        .collect();
    // loading native modules is what a simulator host allows and mlua's safe mode refuses
    let lua = unsafe { Lua::unsafe_new() };
    lua.load(format!("package.cpath = '{cpath}' .. package.cpath")).exec().unwrap();
    lua
}

#[test]
fn require_loads_the_dc_dll() {
    let name: String = host_lua().load("local dc = require('eech_dc'); return dc.name").eval().unwrap();
    assert_eq!(name, "eech-dc 0.1.0");
}

/// A failing DLL callback (mlua builds its traceback, docs/m5-dcs-lua-abi.md) returns its error to
/// the caller, repeatedly, and leaves a state that still collects and closes.
#[test]
fn a_failing_dc_callback_returns_its_error_to_pcall() {
    let lua = host_lua();
    let errors: Vec<String> = lua
        .load(
            "local dc = require('eech_dc')
            local errors = {}
            for i = 1, 100 do
              local ok, err = pcall(dc.boot, 'not a table')
              assert(not ok, 'boot accepted a string')
              errors[i] = tostring(err)
            end
            collectgarbage('collect')
            return errors",
        )
        .eval()
        .unwrap();
    assert_eq!(errors.len(), 100);
    for e in &errors {
        assert!(e.contains("table"), "{e}");
        assert!(e.contains("stack traceback:"), "{e}");
    }
}
