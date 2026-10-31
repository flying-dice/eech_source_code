//! Transport of the reference host's GROUND operations through its Lua terrain provider (EEDCS-48, D3 G2; the campaign DLL sends them: mb_hostterrain.c).
//!
//! `lua/ground-provider-transport.lua` is run IN the actual host code (`eech_world::terrain::install` is what the `eech-world` executable installs as `host.terrain_open`) against a
//! verified private retail map. Its requests are answered by the reviewed `Terrain::ground_above` / `ground_below` / `ground_impact` over the host's own sectors and point solver; the script judges
//! the reply shapes, the complete-or-absent record, the caller-owned world area, the cold and warm hint flow, the strict number reads (signed zero and a subnormal go in and come
//! back with their bits; one field lookup) and the refusals. It computes no ground predicate of its own and mocks no answer.
//!
//! The retail map is private and never in Git, so the map test is driven by the environment and does nothing, loudly, without it: a run that printed "SKIPPED" proved nothing.
//!
//!   EECH_TERRAIN_DIR            the `map<N>/terrain` directory of a verified private installation
//!   EECH_GROUND_TRANSPORT_OUT   optional: a path for the JSON Lines of the checks

use mlua::Lua;
use std::path::PathBuf;

// Carry the actual source into cross-compiled test executables: the build machine's manifest path is unavailable when they run on Windows.
const TRANSPORT: &str = include_str!("../lua/ground-provider-transport.lua");

fn run(terrain_dir: Option<String>, out: Option<String>) -> Result<(), String> {
    let lua = Lua::new();
    let host = lua.create_table().expect("a table");
    let args = lua.create_table().expect("a table");
    if let Some(dir) = terrain_dir {
        args.set("terrain_dir", dir).expect("an argument");
    }
    if let Some(out) = out {
        args.set("out", out).expect("an argument");
    }
    host.set("args", args).expect("the arguments");
    eech_world::terrain::install(&lua, &host).expect("the provider installs");
    lua.globals().set("host", host).expect("the host");
    lua.load(TRANSPORT).set_name("ground-provider-transport.lua").exec().map_err(|e| e.to_string())
}

#[test]
fn the_provider_transports_the_ground_operations_on_the_real_map() {
    let Some(dir) = std::env::var_os("EECH_TERRAIN_DIR").map(PathBuf::from) else {
        eprintln!("ground_provider: SKIPPED (EECH_TERRAIN_DIR is not set): nothing was checked");
        return;
    };
    let out = std::env::var_os("EECH_GROUND_TRANSPORT_OUT").map(|p| PathBuf::from(p).to_string_lossy().replace('\\', "/"));
    if let Err(error) = run(Some(dir.to_string_lossy().replace('\\', "/")), out) {
        panic!("the ground transport check failed: {error}");
    }
}

/// The script's own guard, which needs no map: it refuses to run without the map it is meant to judge, rather than passing.
#[test]
fn the_transport_check_refuses_to_run_without_a_map() {
    let error = run(None, None).expect_err("a run with no terrain_dir must fail");
    assert!(
        error.contains("ground-provider-transport: terrain_dir=<map terrain directory> is required"),
        "{error}"
    );
    let error = run(Some(String::new()), None).expect_err("an empty terrain_dir must fail");
    assert!(error.contains("terrain_dir=<map terrain directory> is required"), "{error}");
}

/// A map directory that does not exist is a visible refusal from the host, never a pass.
#[test]
fn the_transport_check_fails_for_a_missing_map_directory() {
    let missing = std::env::temp_dir()
        .join(format!("no-such-terrain-{}", std::process::id()))
        .to_string_lossy()
        .replace('\\', "/");
    let error = run(Some(missing), None).expect_err("a missing map must fail");
    assert!(error.contains("host.terrain_open"), "{error}");
}
