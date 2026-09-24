// The harness is a Lua host: it links a real Lua 5.1 (PUC liblua5.1, the ABI
// DCS World embeds), and the DC DLL it loads resolves its lua_* symbols
// against that same library, as a module DLL does against its host's Lua.
fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_none() {
        // a link argument (at the end of every link line of this package: the
        // binary and the tests), after mlua-sys, which references lua_*
        println!("cargo::rustc-link-arg=-llua5.1");
    }
}
