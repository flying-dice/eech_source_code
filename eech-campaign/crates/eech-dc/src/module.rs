//! The Lua surface of the module.

use eech_engine::{ArrivalAnswer, Engine, EngineConfig, EngineError, Gunship, LossDecision, LossDescriptor, MemberDescriptor, Side, SupplyReachDescriptor};
use mlua::prelude::*;
use std::path::PathBuf;

/// Rust None becomes Lua nil (not mlua's null sentinel), so scripts can test fields with `if o.task then`
fn options() -> LuaSerializeOptions {
    LuaSerializeOptions::new().serialize_none_to_null(false)
}

fn lua_error(e: EngineError) -> LuaError {
    LuaError::runtime(format!("eech: {e}"))
}

struct LuaEngine {
    engine: Engine,
    bind_mobile: LuaFunction,
    observe_loss: LuaFunction,
    supply_reached: LuaFunction,
}

fn call_bind_mobile(lua: &Lua, function: &LuaFunction, descriptor: MemberDescriptor) -> Result<String, String> {
    let table = lua.create_table().map_err(|e| e.to_string())?;
    table.set("member_key", descriptor.member_key).map_err(|e| e.to_string())?;
    table.set("group_key", descriptor.group_key).map_err(|e| e.to_string())?;
    table.set("entity_id", descriptor.entity_id).map_err(|e| e.to_string())?;
    table.set("entity_type", descriptor.entity_type).map_err(|e| e.to_string())?;
    table.set("sub_type", descriptor.sub_type).map_err(|e| e.to_string())?;
    table
        .set(
            "side",
            match descriptor.side {
                Side::Blue => "blue",
                Side::Red => "red",
                Side::Neutral => "neutral",
            },
        )
        .map_err(|e| e.to_string())?;
    let value: LuaValue = function.call(table).map_err(|e| e.to_string())?;
    match value {
        LuaValue::String(identity) => identity
            .to_str()
            .map(|value| value.to_owned())
            .map_err(|_| "bind_mobile returned a non-UTF-8 identity".into()),
        other => Err(format!("bind_mobile must return a string, got {}", other.type_name())),
    }
}

fn call_observe_loss(lua: &Lua, function: &LuaFunction, descriptor: LossDescriptor) -> Result<LossDecision, String> {
    let value: LuaValue = function
        .call(lua.to_value_with(&descriptor, options()).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    match value {
        LuaValue::String(decision) if decision.as_bytes() == b"forward" => Ok(LossDecision::Forward),
        LuaValue::String(decision) if decision.as_bytes() == b"withhold" => Ok(LossDecision::Withhold),
        LuaValue::String(decision) => Err(format!("observe_loss returned unknown decision {:?}", decision.to_string_lossy())),
        other => Err(format!("observe_loss must return 'forward' or 'withhold', got {}", other.type_name())),
    }
}

fn call_supply_reached(lua: &Lua, function: &LuaFunction, descriptor: SupplyReachDescriptor) -> Result<ArrivalAnswer, String> {
    let value = lua.to_value_with(&descriptor, options()).map_err(|e| e.to_string())?;
    let answer: LuaValue = function.call(value).map_err(|e| e.to_string())?;
    match answer {
        LuaValue::Boolean(gates_passed) => Ok(ArrivalAnswer { gates_passed, launch_angle_error: None }),
        LuaValue::Table(table) => {
            let gates_passed = match table.get::<LuaValue>("gates_passed").map_err(|e| e.to_string())? {
                LuaValue::Boolean(passed) => passed,
                other => return Err(format!("supply_reached gates_passed must be a boolean, got {}", other.type_name())),
            };
            let launch_angle_error = match table.get::<LuaValue>("launch_angle_error").map_err(|e| e.to_string())? {
                LuaValue::Nil => None,
                LuaValue::Number(angle) => Some(angle as f32),
                LuaValue::Integer(angle) => Some(angle as f32),
                other => return Err(format!("supply_reached launch_angle_error must be a number, got {}", other.type_name())),
            };
            Ok(ArrivalAnswer { gates_passed, launch_angle_error })
        }
        other => Err(format!("supply_reached must return a boolean or a result table, got {}", other.type_name())),
    }
}

impl LuaUserData for LuaEngine {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("frame", |lua, this, milliseconds: u32| {
            let bind_mobile = this.bind_mobile.clone();
            let observe_loss = this.observe_loss.clone();
            let supply_reached = this.supply_reached.clone();
            let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
            let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
            let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
            this.engine.frame(milliseconds, &mut binder, &mut observer, &mut reach).map_err(lua_error)
        });
        methods.add_method("report_loss", |lua, this, host_id: String| {
            let receipt = this.engine.report_loss(&host_id).map_err(lua_error)?;
            lua.to_value_with(&receipt, options())
        });
        methods.add_method("objects", |lua, this, ()| {
            let objects = this.engine.objects().map_err(lua_error)?;
            lua.to_value_with(&objects, options())
        });
        methods.add_method("members", |lua, this, ()| {
            let members = this.engine.members().map_err(lua_error)?;
            lua.to_value_with(&members, options())
        });
        methods.add_method("supply_tasks", |lua, this, ()| {
            let tasks = this.engine.supply_tasks().map_err(lua_error)?;
            lua.to_value_with(&tasks, options())
        });
        methods.add_method("roster_stats", |lua, this, ()| {
            let stats = this.engine.roster_stats().map_err(lua_error)?;
            lua.to_value_with(&stats, options())
        });
        methods.add_method("clock", |lua, this, ()| {
            let clock = this.engine.clock().map_err(lua_error)?;
            lua.to_value_with(&clock, options())
        });
        methods.add_method("diagnostics", |_, this, ()| this.engine.diagnostics().map_err(lua_error));
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

fn boot(lua: &Lua, config: LuaTable) -> LuaResult<LuaEngine> {
    let bind_mobile = match config.get::<LuaValue>("bind_mobile")? {
        LuaValue::Function(function) => function,
        LuaValue::Nil => return Err(LuaError::runtime("eech: missing host bind_mobile callback")),
        value => return Err(LuaError::runtime(format!("eech: bind_mobile must be a function, got {}", value.type_name()))),
    };
    let observe_loss = match config.get::<LuaValue>("observe_loss")? {
        LuaValue::Function(function) => function,
        LuaValue::Nil => return Err(LuaError::runtime("eech: missing host observe_loss callback")),
        value => return Err(LuaError::runtime(format!("eech: observe_loss must be a function, got {}", value.type_name()))),
    };
    let supply_reached = match config.get::<LuaValue>("supply_reached")? {
        LuaValue::Function(function) => function,
        LuaValue::Nil => return Err(LuaError::runtime("eech: missing host supply_reached callback")),
        value => return Err(LuaError::runtime(format!("eech: supply_reached must be a function, got {}", value.type_name()))),
    };
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
    let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
    let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
    let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
    let engine = Engine::boot(&config, &mut binder, &mut observer, &mut reach).map_err(lua_error)?;
    Ok(LuaEngine {
        engine,
        bind_mobile,
        observe_loss,
        supply_reached,
    })
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
