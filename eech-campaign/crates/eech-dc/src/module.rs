//! The Lua surface of the module.

use eech_engine::{Engine, EngineConfig, EngineError, Gunship};
use mlua::prelude::*;
use std::path::PathBuf;

/// Rust None becomes Lua nil (not mlua's null sentinel), so scripts can test fields with `if o.task then`
fn options() -> LuaSerializeOptions {
    LuaSerializeOptions::new().serialize_none_to_null(false)
}

fn lua_error(e: EngineError) -> LuaError {
    LuaError::runtime(format!("eech: {e}"))
}

struct LuaEngine(Engine);

impl LuaUserData for LuaEngine {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("frame", |_, this, milliseconds: u32| this.0.frame(milliseconds).map_err(lua_error));
        methods.add_method("objects", |lua, this, ()| {
            let objects = this.0.objects().map_err(lua_error)?;
            lua.to_value_with(&objects, options())
        });
        methods.add_method("clock", |lua, this, ()| {
            let clock = this.0.clock().map_err(lua_error)?;
            lua.to_value_with(&clock, options())
        });
        methods.add_method("diagnostics", |_, this, ()| this.0.diagnostics().map_err(lua_error));
    }
}

fn gunship(name: &str) -> LuaResult<Gunship> {
    match name.to_ascii_lowercase().as_str() {
        "apache" => Ok(Gunship::Apache),
        "havoc" => Ok(Gunship::Havoc),
        "comanche" => Ok(Gunship::Comanche),
        "hokum" => Ok(Gunship::Hokum),
        other => Err(LuaError::runtime(format!("eech: bad argument: unknown gunship {other:?}"))),
    }
}

fn boot(_: &Lua, config: LuaTable) -> LuaResult<LuaEngine> {
    let arguments: Vec<String> = config.get::<Option<Vec<String>>>("arguments")?.unwrap_or_default();
    let config = EngineConfig {
        install_root: PathBuf::from(config.get::<String>("install_root")?),
        map_path: config.get("map")?,
        campaign_directory: config.get::<Option<String>>("campaign_directory")?.unwrap_or_else(|| "camp01".into()),
        campaign_filename: config.get("campaign")?,
        gunship: gunship(&config.get::<Option<String>>("gunship")?.unwrap_or_else(|| "apache".into()))?,
        random_seed: config.get::<Option<u32>>("seed")?.unwrap_or(1),
        arguments,
    };
    Engine::boot(&config).map(LuaEngine).map_err(lua_error)
}

#[mlua::lua_module]
pub fn eech_dc(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("name", concat!("eech-dc ", env!("CARGO_PKG_VERSION")))?;
    exports.set("boot", lua.create_function(boot)?)?;
    exports.set(
        "prepare_installation",
        lua.create_function(|_, root: String| eech_engine::prepare_installation(std::path::Path::new(&root)).map_err(lua_error))?,
    )?;
    Ok(exports)
}
