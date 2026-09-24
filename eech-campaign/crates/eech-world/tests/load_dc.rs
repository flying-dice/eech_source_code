//! The host loads the dynamic campaign DLL the way a simulator does: require.
use mlua::Lua;

#[test]
fn require_loads_the_dc_dll() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/debug");
    // loading native modules is what a simulator host allows and mlua's safe mode refuses
    let lua = unsafe { Lua::unsafe_new() };
    lua.load(format!("package.cpath = '{}/lib?.so;' .. package.cpath", dir.display()))
        .exec()
        .unwrap();
    let name: String = lua.load("local dc = require('eech_dc'); return dc.name .. ' ' .. dc.version").eval().unwrap();
    assert_eq!(name, "eech-dc 0.1.0");
}
