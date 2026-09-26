// mlua `module` mode never links Lua into the cdylib: DCS provides the symbols
// at load time (on Windows through the LUA_LIB import-lib pin in
// .cargo/config.toml). The tests are ordinary executables that create real
// Lua states, so elsewhere they must link a real Lua 5.1 themselves: PUC
// liblua5.1 (the same 5.1 ABI DCS ships; Debian/Ubuntu: liblua5.1-0-dev).
// Same arrangement as the reference crate (dcs-studio bridge-core/build.rs).
fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_none() {
        println!("cargo::rustc-link-lib=dylib=lua5.1");
    }
}
