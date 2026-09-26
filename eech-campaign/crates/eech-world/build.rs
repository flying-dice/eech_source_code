// The host embeds a real Lua 5.1 (PUC liblua5.1), and the DC DLL it loads
// resolves its lua_* symbols against that same library, as a module DLL does
// against its host's Lua.
fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_none() {
        // a link argument (at the end of every link line of this package: the
        // binary and the tests), after mlua-sys, which references lua_*
        println!("cargo::rustc-link-arg=-llua5.1");
    } else if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu") {
        // EECH runs on the main thread: Linux gives it 8 MB of stack, Windows 1 MB
        println!("cargo::rustc-link-arg-bins=-Wl,--stack,16777216");
    }
}
