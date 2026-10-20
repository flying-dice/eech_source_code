//! The host loads the dynamic campaign DLL the way a simulator does: require.
use mlua::Lua;

#[test]
fn require_loads_the_dc_dll() {
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
    let name: String = lua.load("local dc = require('eech_dc'); return dc.name").eval().unwrap();
    assert_eq!(name, "eech-dc 0.1.0");
}
