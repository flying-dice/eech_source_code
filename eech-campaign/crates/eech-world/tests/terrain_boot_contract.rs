//! The terrain provider is a required part of the module's boot contract (EEDCS-47): through the real module, loaded the way a simulator host loads it,
//! a boot with every other callback but no terrain provider is refused before anything boots, with the stable first-line marker
//! `[terrain-provider:missing]`, and so is a terrain provider that is not a function. A missing other callback is still reported as itself, so the marker
//! is never the answer to some other mistake. These cases return before the engine's one boot per process is used; they need no installation or asset.

use mlua::Lua;

/// A Lua state that finds the DC DLL with require, as a simulator host's does (see load_dc.rs).
fn host_lua() -> Lua {
    let deps = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let cpath: String = [deps.as_path(), deps.parent().unwrap()]
        .iter()
        .map(|d| {
            let d = d.display().to_string().replace('\\', "/");
            format!("{d}/lib?.so;{d}/?.dll;")
        })
        .collect();
    let lua = unsafe { Lua::unsafe_new() };
    lua.load(format!("package.cpath = '{cpath}' .. package.cpath")).exec().unwrap();
    lua
}

const SCRIPT: &str = r#"
    local dc = require ('eech_dc')
    local function config (overrides)
        local c = {
            install_root = 'unused', map = 'unused', campaign = 'unused',
            bind_mobile = function () end, observe_loss = function () end, supply_reached = function () end, move_vehicle = function () end,
            line_of_sight = function () end, sense = function () end, aim = function () end,
        }
        for key, value in pairs (overrides) do c [key] = value end
        return c
    end
    local function first_line (value) return (tostring (value):gsub ('\n.*', '')) end
    local results = {}
    local function refused (name, c)
        local ok, err = pcall (dc.boot, c)
        assert (not ok, name .. ': the boot was accepted')
        results [name] = first_line (err)
    end
    refused ('no_terrain', config {})
    refused ('terrain_number', config { terrain = 5 })
    refused ('terrain_table', config { terrain = { op = 'point' } })
    refused ('no_aim_and_no_terrain', config { aim = false })
    refused ('no_aim_with_terrain', config { aim = false, terrain = function () end })
    return results
"#;

#[test]
fn a_boot_without_a_terrain_provider_is_refused_with_the_missing_marker() {
    let lua = host_lua();
    let results: mlua::Table = lua.load(SCRIPT).eval().unwrap();
    let get = |name: &str| results.get::<String>(name).unwrap();
    for name in ["no_terrain", "terrain_number", "terrain_table"] {
        let message = get(name);
        assert!(message.contains("[terrain-provider:missing]"), "{name}: {message}");
        assert!(!message.contains("terrain.ffp"), "{name}: {message}");
    }
    // another missing callback is reported as itself, with or without a terrain provider, and never as the terrain's
    for name in ["no_aim_and_no_terrain", "no_aim_with_terrain"] {
        let message = get(name);
        assert!(message.contains("aim"), "{name}: {message}");
        assert!(!message.contains("[terrain-provider:missing]"), "{name}: {message}");
    }
}
