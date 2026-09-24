//! The Lua surface: `require ("eech_dc")`.

use std::cell::RefCell;
use std::time::Duration;

use eech_campaign::{Bounds, Campaign, CampaignConfig, CampaignError, EntityId, ObjectModel, Position, World};
use mlua::prelude::{LuaError, LuaFunction, LuaResult, LuaTable, LuaValue};
use mlua::{Lua, LuaSerdeExt, SerializeOptions, UserData, UserDataMethods};

use crate::coords::{to_dcs, to_eech, DcsPoint, Origin};
use crate::names::{name_ids, Names};

/// The embedded mission driver (`eech.start`): scheduling, the DCS world,
/// events and teardown, written in Lua against DCS's scripting API; and
/// `eech.place`, which positions keysites on DCS airbases.
const DRIVER_SOURCE: &str = include_str!("../lua/driver.lua");

/// The `luaopen_eech_dc` entry point DCS's `require` calls.
///
/// # Errors
///
/// Any `mlua` error while building the module table or loading the embedded
/// driver; `require` then fails with it, and DCS logs it (the mission runs on
/// without the campaign, which is recoverable; a panic would not be).
#[mlua::lua_module]
pub fn eech_dc(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("name", "eech-dc")?;
    exports.set("version", env!("CARGO_PKG_VERSION"))?;
    exports.set("new", lua.create_function(new_campaign)?)?;
    exports.set(
        "to_eech",
        lua.create_function(|lua, (point, origin): (LuaTable, Option<LuaTable>)| position_to_lua(lua, to_eech(dcs_point(&point)?, origin_of(origin)?)))?,
    )?;
    exports.set(
        "to_dcs",
        lua.create_function(|lua, (position, origin): (LuaTable, Option<LuaTable>)| {
            dcs_point_to_lua(lua, to_dcs(eech_position(&position)?, origin_of(origin)?))
        })?,
    )?;
    let driver: LuaTable = lua.load(DRIVER_SOURCE).set_name("=eech_dc_driver").call(exports.clone())?;
    exports.set("start", driver.get::<LuaFunction>("start")?)?;
    exports.set("place", driver.get::<LuaFunction>("place")?)?;
    Ok(exports)
}

fn serialize_options() -> SerializeOptions {
    SerializeOptions::new().serialize_none_to_null(false).serialize_unit_to_null(false)
}

fn number(table: &LuaTable, key: &str) -> LuaResult<f64> {
    table
        .get::<Option<f64>>(key)?
        .ok_or_else(|| LuaError::runtime(format!("eech: expected a number field `{key}`")))
}

fn dcs_point(table: &LuaTable) -> LuaResult<DcsPoint> {
    Ok(DcsPoint {
        x: number(table, "x")?,
        y: table.get::<Option<f64>>("y")?.unwrap_or(0.0),
        z: number(table, "z")?,
    })
}

#[allow(clippy::cast_possible_truncation)] // EECH positions are C floats
fn eech_position(table: &LuaTable) -> LuaResult<Position> {
    Ok(Position::new(
        number(table, "x")? as f32,
        table.get::<Option<f64>>("y")?.unwrap_or(0.0) as f32,
        number(table, "z")? as f32,
    ))
}

fn origin_of(table: Option<LuaTable>) -> LuaResult<Origin> {
    match table {
        None => Ok(Origin::default()),
        Some(t) => Ok(Origin {
            x: number(&t, "x")?,
            z: number(&t, "z")?,
        }),
    }
}

fn position_to_lua(lua: &Lua, p: Position) -> LuaResult<LuaTable> {
    let t = lua.create_table()?;
    t.set("x", f64::from(p.x))?;
    t.set("y", f64::from(p.y))?;
    t.set("z", f64::from(p.z))?;
    Ok(t)
}

fn dcs_point_to_lua(lua: &Lua, p: DcsPoint) -> LuaResult<LuaTable> {
    let t = lua.create_table()?;
    t.set("x", p.x)?;
    t.set("y", p.y)?;
    t.set("z", p.z)?;
    Ok(t)
}

/// A campaign error as the Lua error DCS's scripting sandbox logs.
fn campaign_error(e: &CampaignError) -> LuaError {
    let kind = match e {
        CampaignError::AlreadyRunning => "already-running",
        CampaignError::InvalidConfig(_) => "invalid-config",
        CampaignError::InvalidEntity(_) => "invalid-entity",
        CampaignError::Assertion { .. } => "assertion",
        CampaignError::Fatal { .. } => "fatal",
        CampaignError::Unported { .. } => "unported",
        CampaignError::Boundary { .. } => "boundary",
        CampaignError::World(_) => "world",
        CampaignError::Poisoned => "poisoned",
        _ => "internal",
    };
    LuaError::runtime(format!("eech: {kind}: {e}"))
}

/// `eech.new { campaign = <CampaignConfig>, origin = { x = <north>, z = <east> } }`
fn new_campaign(lua: &Lua, spec: LuaTable) -> LuaResult<LuaCampaign> {
    let config: CampaignConfig = lua.from_value(spec.get::<LuaValue>("campaign")?)?;
    let origin = origin_of(spec.get::<Option<LuaTable>>("origin")?)?;
    let campaign = Campaign::new(config.clone()).map_err(|e| campaign_error(&e))?;
    let names: Names = config
        .keysites
        .iter()
        .map(|k| k.name.clone())
        .chain(
            config
                .groups
                .iter()
                .flat_map(|g| std::iter::once(g.name.clone()).chain(g.members.iter().map(|m| m.name.clone()))),
        )
        .filter_map(|n| campaign.entity(&n).map(|id| (id, n)))
        .collect();
    Ok(LuaCampaign {
        campaign: Some(campaign),
        names,
        origin,
    })
}

/// The campaign, owned by the Lua state that created it: `close`, garbage
/// collection or `lua_close` ends it (and frees the process's one campaign).
struct LuaCampaign {
    campaign: Option<Campaign>,
    names: Names,
    origin: Origin,
}

impl LuaCampaign {
    fn campaign(&mut self) -> LuaResult<&mut Campaign> {
        self.campaign.as_mut().ok_or_else(|| LuaError::runtime("eech: the campaign is closed"))
    }

    fn to_lua<T: serde::Serialize>(&self, lua: &Lua, value: &T) -> LuaResult<LuaValue> {
        let mut json = serde_json::to_value(value).map_err(LuaError::external)?;
        name_ids(&mut json, &self.names);
        lua.to_value_with(&json, serialize_options())
    }
}

/// `World`, answered by the Lua table the step is given:
/// `position (name) -> DCS point | nil` and `object_bounds (model) -> { min, max } | nil`.
struct LuaWorld<'a> {
    table: &'a LuaTable,
    names: &'a Names,
    origin: Origin,
    /// the first Lua error a query raised (the step fails with it)
    failure: RefCell<Option<LuaError>>,
}

impl LuaWorld<'_> {
    fn fail(&self, e: LuaError) {
        let mut failure = self.failure.borrow_mut();
        if failure.is_none() {
            *failure = Some(e);
        }
    }

    fn position_of(&self, name: &str) -> LuaResult<Option<Position>> {
        let f: LuaFunction = self.table.get("position")?;
        match f.call::<LuaValue>(name)? {
            LuaValue::Nil => Ok(None),
            LuaValue::Table(t) => Ok(Some(to_eech(dcs_point(&t)?, self.origin))),
            other => Err(LuaError::runtime(format!("eech: world.position ({name}) returned a {}", other.type_name()))),
        }
    }

    fn bounds_of(&self, model: ObjectModel) -> LuaResult<Option<Bounds>> {
        let f: LuaFunction = self.table.get("object_bounds")?;
        match f.call::<LuaValue>(model.0)? {
            LuaValue::Nil => Ok(None),
            LuaValue::Table(t) => Ok(Some(Bounds {
                min: eech_position(&t.get("min")?)?,
                max: eech_position(&t.get("max")?)?,
            })),
            other => Err(LuaError::runtime(format!(
                "eech: world.object_bounds ({}) returned a {}",
                model.0,
                other.type_name()
            ))),
        }
    }
}

impl World for LuaWorld<'_> {
    fn position(&self, entity: EntityId) -> Option<Position> {
        let Some(name) = self.names.get(&entity) else {
            self.fail(LuaError::runtime(format!(
                "eech: the campaign asked for the position of {entity}, which names no DCS unit"
            )));
            return None;
        };
        self.position_of(name).unwrap_or_else(|e| {
            self.fail(e);
            None
        })
    }

    fn object_bounds(&self, model: ObjectModel) -> Option<Bounds> {
        self.bounds_of(model).unwrap_or_else(|e| {
            self.fail(e);
            None
        })
    }
}

impl UserData for LuaCampaign {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // campaign:step (dt, world) -> { events }
        methods.add_method_mut("step", |lua, this, (dt, world): (f64, LuaTable)| {
            if !(dt > 0.0 && dt.is_finite()) {
                return Err(LuaError::runtime(format!("eech: step needs a positive frame delta, not {dt}")));
            }
            let origin = this.origin;
            let names = std::mem::take(&mut this.names);
            let mut lua_world = LuaWorld {
                table: &world,
                names: &names,
                origin,
                failure: RefCell::new(None),
            };
            let result = this
                .campaign()
                .and_then(|c| c.step(&mut lua_world, Duration::from_secs_f64(dt)).map_err(|e| campaign_error(&e)));
            let failure = lua_world.failure.into_inner();
            this.names = names;
            match (result, failure) {
                (Ok(report), _) => this.to_lua(lua, &report.events),
                // the world's own Lua error is the cause: keep it
                (Err(e), Some(cause)) => Err(LuaError::runtime(format!("{e}: {cause}"))),
                (Err(e), None) => Err(e),
            }
        });

        // campaign:drain_events () -> { events a failed step produced }
        methods.add_method_mut("drain_events", |lua, this, ()| {
            let events = this.campaign()?.drain_events();
            this.to_lua(lua, &events)
        });

        // campaign:snapshot () -> { forces, keysites, groups, tasks }
        methods.add_method_mut("snapshot", |lua, this, ()| {
            let snapshot = this.campaign()?.snapshot();
            this.to_lua(lua, &snapshot)
        });

        methods.add_method_mut("is_poisoned", |_, this, ()| Ok(this.campaign()?.is_poisoned()));

        // campaign:is_live (name): the configured entity still exists
        methods.add_method_mut("is_live", |_, this, name: String| Ok(this.campaign()?.entity(&name).is_some()));

        methods.add_method("to_eech", |lua, this, point: LuaTable| {
            position_to_lua(lua, to_eech(dcs_point(&point)?, this.origin))
        });
        methods.add_method("to_dcs", |lua, this, position: LuaTable| {
            dcs_point_to_lua(lua, to_dcs(eech_position(&position)?, this.origin))
        });

        // campaign:close (): ends the campaign now (idempotent)
        methods.add_method_mut("close", |_, this, ()| {
            let closed = this.campaign.take().is_some();
            Ok(closed)
        });
    }
}
