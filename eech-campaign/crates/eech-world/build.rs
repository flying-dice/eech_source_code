// The host embeds a real Lua 5.1 (PUC liblua5.1), and the DC DLL it loads
// resolves its lua_* symbols against that same library, as a module DLL does
// against its host's Lua.
fn main() {
    // The convoy, ship, person and fixed-wing steps are EECH's own arithmetic: it takes the engine's numeric flags
    // (eech-engine-sys build) so the host and the original reference produce the same bits.
    println!("cargo::rerun-if-env-changed=EECH_ENGINE_OPT");
    for file in [
        "eech_arith.h",
        "convoy_step.c",
        "ship_step.c",
        "person_step.c",
        "fw_step.c",
        "hc_step.c",
        "fw_navigator.h",
        "hc_guidance.h",
        "death_step.c",
        "place_step.c",
        "dependent_step.c",
        "expire_step.c",
        "los_step.c",
        "sense_step.c",
        "aim_step.c",
    ] {
        println!("cargo::rerun-if-changed=csrc/{file}");
    }
    let opt = std::env::var("EECH_ENGINE_OPT").unwrap_or_else(|_| "1".into());
    cc::Build::new()
        .file("csrc/convoy_step.c")
        .file("csrc/ship_step.c")
        .file("csrc/person_step.c")
        .file("csrc/fw_step.c")
        .file("csrc/hc_step.c")
        .file("csrc/death_step.c")
        .file("csrc/place_step.c")
        .file("csrc/dependent_step.c")
        .file("csrc/expire_step.c")
        .file("csrc/los_step.c")
        .file("csrc/sense_step.c")
        .file("csrc/aim_step.c")
        .opt_level_str(&opt)
        .debug(true)
        .pic(true)
        .warnings(false)
        .flag("-std=gnu99")
        .flag("-w")
        .flag("-fno-strict-aliasing")
        .flag("-fwrapv")
        .flag("-ftrivial-auto-var-init=zero")
        .compile("eech_steps");
    if std::env::var_os("CARGO_CFG_WINDOWS").is_none() {
        // a link argument (at the end of every link line of this package: the
        // binary and the tests), after mlua-sys, which references lua_*
        println!("cargo::rustc-link-arg=-llua5.1");
    } else if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu") {
        // EECH runs on the main thread: Linux gives it 8 MB of stack, Windows 1 MB
        println!("cargo::rustc-link-arg-bins=-Wl,--stack,16777216");
    }
}
