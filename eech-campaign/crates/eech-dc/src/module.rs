//! The Lua surface of the module.

use eech_engine::{
    AimEvent, AimReply, AimRequest, AimStepKind, AimTarget, AimWeapon, AimWorld, ArrivalAnswer, Engine, EngineConfig, EngineError, Gunship,
    HelicopterGuidanceAnswer, HelicopterQuery, LosMessage, LosReply, LossDecision, LossDescriptor, MemberDescriptor, MovementMode, MovementResult,
    MovementStep, SenseMessage, SenseReason, SenseReply, SenseScan, SenseState, Side, SupplyReachDescriptor, VehiclePose, WeaponFigures,
};
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
    move_vehicle: LuaFunction,
    line_of_sight: LuaFunction,
    sense: LuaFunction,
    aim: LuaFunction,
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
    if let (Some(pose), Some(kind)) = (descriptor.pose, descriptor.movement_kind) {
        table.set("movement_kind", kind as i32).map_err(|e| e.to_string())?;
        table
            .set("pose", lua.to_value_with(&pose, options()).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }
    if let Some(motion_vector) = descriptor.motion_vector {
        table
            .set("motion_vector", lua.to_value_with(&motion_vector, options()).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }
    if let Some(rotor) = descriptor.rotor {
        table
            .set("rotor", lua.to_value_with(&rotor, options()).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }
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
        LuaValue::Boolean(gates_passed) => Ok(ArrivalAnswer {
            gates_passed,
            launch_angle_error: None,
            heading_target_missing: false,
        }),
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
            let heading_target_missing = match table.get::<LuaValue>("heading_target_missing").map_err(|e| e.to_string())? {
                LuaValue::Nil => false,
                LuaValue::Boolean(missing) => missing,
                other => return Err(format!("supply_reached heading_target_missing must be a boolean, got {}", other.type_name())),
            };
            Ok(ArrivalAnswer {
                gates_passed,
                launch_angle_error,
                heading_target_missing,
            })
        }
        other => Err(format!("supply_reached must return a boolean or a result table, got {}", other.type_name())),
    }
}

/// The `N` numbers a table field holds (a number for `N == 1`, else an array), exactly as Lua
/// holds them. mlua's own number reads turn any value within `f64::EPSILON` of an integer into
/// that integer, so a physical value such as -6.7e-17 would arrive as 0 and the two sides'
/// copies of a pose would differ. Lua looks the field up and formats the numbers at 17 significant digits instead,
/// and `nearest_f32` narrows them.
fn exact_numbers<const N: usize>(lua: &Lua, table: &LuaTable, field: &str) -> Result<[f32; N], String> {
    let values = exact_f64s::<N>(lua, table, field)?;
    let mut numbers = [0.0f32; N];
    for (slot, value) in numbers.iter_mut().zip(values) {
        *slot = nearest_f32(value).ok_or_else(|| shape_text(field, N))?;
    }
    Ok(numbers)
}

/// The scalars `names` (comma-separated field names of `table`, `N` of them) exactly as Lua holds them, in one call: the formatting of
/// `exact_f64s` for several scalar fields at once, which the aiming exchange (a request per entity per update) makes worth having. The Lua function
/// for a name list is built once and kept in the registry.
fn exact_fields<const N: usize>(lua: &Lua, table: &LuaTable, names: &'static str) -> Result<[f32; N], String> {
    let key = format!("eech_exact_fields:{names}");
    let format: LuaFunction = match lua.named_registry_value(&key) {
        Ok(function) => function,
        Err(_) => {
            let fields: Vec<&str> = names.split(',').collect();
            assert_eq!(fields.len(), N, "exact_fields: {names} is not {N} names");
            let mut code = String::from("return function(t)\n");
            for (index, field) in fields.iter().enumerate() {
                code += &format!(
                    "local v{index} = t.{field}\nif type(v{index}) ~= 'number' or v{index} ~= v{index} or v{index} == math.huge or v{index} == -math.huge then return nil, '{field}' end\n"
                );
            }
            let vars: Vec<String> = (0..N).map(|index| format!("v{index}")).collect();
            code += &format!("return string.format('{}', {})\nend", "%.17g ".repeat(N), vars.join(", "));
            let function: LuaFunction = lua.load(code).eval().map_err(|e| e.to_string())?;
            lua.set_named_registry_value(&key, function.clone()).map_err(|e| e.to_string())?;
            function
        }
    };
    let (text, bad): (Option<String>, Option<String>) = format.call(table).map_err(|e| e.to_string())?;
    let Some(text) = text else {
        return Err(shape_text(&bad.unwrap_or_default(), 1));
    };
    let mut parts = text.split_whitespace();
    let mut numbers = [0.0f32; N];
    for (slot, field) in numbers.iter_mut().zip(names.split(',')) {
        *slot = parts
            .next()
            .and_then(|part| part.parse::<f64>().ok())
            .and_then(nearest_f32)
            .ok_or_else(|| shape_text(field, 1))?;
    }
    Ok(numbers)
}

fn shape_text(field: &str, n: usize) -> String {
    format!(
        "{field} must be {}",
        if n == 1 {
            "a finite number".to_owned()
        } else {
            format!("{n} finite numbers")
        }
    )
}

#[cfg(feature = "sensor-fixture")]
/// A finite double exactly as Lua holds it, whatever the FPU rounding mode. `exact_f64s` formats at 17 significant digits, and the engine
/// runs a callback under round-toward-zero, where the C library's formatting truncates instead of rounding: the parsed double can then be one
/// ulp low. This reads the double in exact steps instead (`math.frexp`, a multiplication by a power of two, a floor, a subtraction: all exact in
/// any mode) as three integers and rebuilds it with exact scalings.
fn exact_f64_bits(lua: &Lua, table: &LuaTable, field: &str) -> Result<f64, String> {
    const KEY: &str = "eech_exact_f64_bits";
    let split: LuaFunction = match lua.named_registry_value(KEY) {
        Ok(function) => function,
        Err(_) => {
            let function: LuaFunction = lua
                .load(
                    "return function(t, f)
                       local v = t[f]
                       if type(v) ~= 'number' or v ~= v or v == math.huge or v == -math.huge then return nil end
                       local m, e = math.frexp(v)
                       local mi = m * 9007199254740992
                       local hi = math.floor(mi / 67108864)
                       local lo = mi - hi * 67108864
                       return string.format('%d %d %d', hi, lo, e)
                     end",
                )
                .eval()
                .map_err(|e| e.to_string())?;
            lua.set_named_registry_value(KEY, function.clone()).map_err(|e| e.to_string())?;
            function
        }
    };
    let text: Option<String> = split.call((table, field)).map_err(|e| e.to_string())?;
    let shape = || format!("{field} must be a finite number");
    let text = text.ok_or_else(shape)?;
    let mut parts = text.split_whitespace().map(|part| part.parse::<i64>().ok());
    let (Some(Some(hi)), Some(Some(lo)), Some(Some(exponent))) = (parts.next(), parts.next(), parts.next()) else {
        return Err(shape());
    };
    // the 53-bit significand is exact in an f64; the scaling is split so that a subnormal result is not rounded twice
    let significand = (hi * 67_108_864 + lo) as f64;
    let exponent = exponent - 53;
    let (first, second) = if exponent < -1000 { (-1000, exponent + 1000) } else { (exponent, 0) };
    Ok(significand * 2f64.powi(first as i32) * 2f64.powi(second as i32))
}

/// The `N` finite numbers of `exact_numbers`, kept as the `f64` Lua holds.
fn exact_f64s<const N: usize>(lua: &Lua, table: &LuaTable, field: &str) -> Result<[f64; N], String> {
    const KEY: &str = "eech_exact_numbers";
    let format: LuaFunction = match lua.named_registry_value(KEY) {
        Ok(function) => function,
        Err(_) => {
            let function: LuaFunction = lua
                .load(
                    "return function(t, f, n)
                       local v = t[f]
                       if n == 1 and type(v) == 'number' then v = { v }
                       elseif type(v) ~= 'table' or #v ~= n then return nil end
                       for i = 1, n do
                         local x = v[i]
                         if type(x) ~= 'number' or x ~= x or x == math.huge or x == -math.huge then return nil end
                       end
                       return string.format(string.rep('%.17g ', n), unpack(v))
                     end",
                )
                .eval()
                .map_err(|e| e.to_string())?;
            lua.set_named_registry_value(KEY, function.clone()).map_err(|e| e.to_string())?;
            function
        }
    };
    let text: Option<String> = format.call((table, field, N)).map_err(|e| e.to_string())?;
    let shape = || shape_text(field, N);
    let text = text.ok_or_else(shape)?;
    let mut parts = text.split_whitespace();
    let mut numbers = [0.0f64; N];
    for slot in &mut numbers {
        *slot = parts
            .next()
            .and_then(|part| part.parse::<f64>().ok())
            .filter(|value| value.is_finite())
            .ok_or_else(shape)?;
    }
    Ok(numbers)
}

/// The `f32` nearest `value`, or `None` beyond the `f32` range. A plain `as f32` follows the
/// FPU rounding mode, and the engine runs a callback under round-toward-zero, where a value a
/// hair under an `f32` narrows to its neighbour. The cast only picks the neighbourhood; the
/// nearest of it and the two adjacent `f32`s is taken, the cast's own result winning a tie so a
/// zero keeps its sign.
fn nearest_f32(value: f64) -> Option<f32> {
    if !value.is_finite() || value.abs() > f64::from(f32::MAX) {
        return None;
    }
    let guess = value as f32;
    [guess, guess.next_down(), guess.next_up()]
        .into_iter()
        .filter(|candidate| candidate.is_finite())
        .min_by(|a, b| (f64::from(*a) - value).abs().total_cmp(&(f64::from(*b) - value).abs()))
}

/// One message of the host-owned line-of-sight dialogue as a Lua table: `{ mode = "begin" | "terrain" | "objects", ... }`. Positions are
/// arrays of three numbers; a host identity is absent (nil) when the campaign roster has none for that end.
fn los_message_table(lua: &Lua, message: &LosMessage) -> Result<LuaTable, String> {
    let table = lua.create_table().map_err(|e| e.to_string())?;
    match message {
        LosMessage::Begin(begin) => {
            table.set("mode", "begin").map_err(|e| e.to_string())?;
            table.set("entity_form", begin.entity_form).map_err(|e| e.to_string())?;
            table.set("criteria", begin.criteria).map_err(|e| e.to_string())?;
            table.set("source_eye", begin.source_eye).map_err(|e| e.to_string())?;
            table.set("target_eye", begin.target_eye).map_err(|e| e.to_string())?;
            table.set("source_position", begin.source_position.to_vec()).map_err(|e| e.to_string())?;
            table.set("target_position", begin.target_position.to_vec()).map_err(|e| e.to_string())?;
            if let Some(id) = &begin.source_host_id {
                table.set("source_host_id", id.as_str()).map_err(|e| e.to_string())?;
            }
            if let Some(id) = &begin.target_host_id {
                table.set("target_host_id", id.as_str()).map_err(|e| e.to_string())?;
            }
        }
        LosMessage::Terrain { elevation } => {
            table.set("mode", "terrain").map_err(|e| e.to_string())?;
            table.set("elevation", *elevation).map_err(|e| e.to_string())?;
        }
        LosMessage::Objects { hit, collision_point } => {
            table.set("mode", "objects").map_err(|e| e.to_string())?;
            table.set("hit", *hit).map_err(|e| e.to_string())?;
            if let Some(point) = collision_point {
                table.set("collision_point", point.to_vec()).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(table)
}

/// The host's answer to a line-of-sight message: `{ state = "need_terrain", phase, point }`, `{ state = "need_objects", phase, from, to }` or
/// `{ state = "done", visible }`. Every field is mandatory for its state and exact; anything else is an error, never a default.
fn call_line_of_sight(lua: &Lua, function: &LuaFunction, message: LosMessage) -> Result<LosReply, String> {
    let argument = los_message_table(lua, &message)?;
    let answer: LuaValue = function.call(argument).map_err(|e| e.to_string())?;
    let LuaValue::Table(table) = answer else {
        return Err(format!("line_of_sight must return a result table, got {}", answer.type_name()));
    };
    let state: LuaValue = table.get("state").map_err(|e| e.to_string())?;
    let LuaValue::String(state) = state else {
        return Err(format!("line_of_sight result state must be a string, got {}", state.type_name()));
    };
    let phase = |table: &LuaTable| -> Result<i32, String> {
        match table.get::<LuaValue>("phase").map_err(|e| e.to_string())? {
            LuaValue::Integer(phase) => i32::try_from(phase).map_err(|_| format!("line_of_sight result phase {phase} is out of range")),
            other => Err(format!("line_of_sight result phase must be an integer, got {}", other.type_name())),
        }
    };
    let state_text = state.to_str().map_err(|e| e.to_string())?;
    match &*state_text {
        "need_terrain" => Ok(LosReply::NeedTerrain {
            phase: phase(&table)?,
            point: exact_numbers(lua, &table, "point").map_err(|e| format!("line_of_sight result {e}"))?,
        }),
        "need_objects" => Ok(LosReply::NeedObjects {
            phase: phase(&table)?,
            start: exact_numbers(lua, &table, "from").map_err(|e| format!("line_of_sight result {e}"))?,
            end: exact_numbers(lua, &table, "to").map_err(|e| format!("line_of_sight result {e}"))?,
        }),
        "done" => match table.get::<LuaValue>("visible").map_err(|e| e.to_string())? {
            LuaValue::Boolean(visible) => Ok(LosReply::Done { visible }),
            other => Err(format!("line_of_sight result visible must be a boolean, got {}", other.type_name())),
        },
        other => Err(format!("line_of_sight result state {other:?} is not need_terrain, need_objects or done")),
    }
}

fn aiming_type_name(value: i32) -> Result<&'static str, String> {
    match value {
        0 => Ok("none"),
        1 => Ok("intercept_point"),
        2 => Ok("angle_of_projection"),
        3 => Ok("lead_and_ballistic"),
        other => Err(format!("aim received unknown aiming type {other}")),
    }
}

fn aim_world_into(table: &LuaTable, world: &AimWorld) -> Result<(), String> {
    let e = |error: LuaError| error.to_string();
    table.set("map_min", world.map_min.to_vec()).map_err(e)?;
    table.set("map_max", world.map_max.to_vec()).map_err(e)?;
    table.set("comms_weapon_lag", world.comms_weapon_lag).map_err(e)?;
    Ok(())
}

fn aim_weapon_into(lua: &Lua, table: &LuaTable, weapon: &AimWeapon) -> Result<(), String> {
    let e = |error: LuaError| error.to_string();
    let facts = lua.create_table().map_err(e)?;
    facts.set("weapon", weapon.weapon).map_err(e)?;
    facts.set("present", weapon.present).map_err(e)?;
    facts.set("aiming_type", aiming_type_name(weapon.aiming_type)?).map_err(e)?;
    facts.set("cruise_velocity", weapon.cruise_velocity).map_err(e)?;
    facts.set("acquire_parent_forward_velocity", weapon.acquire_forward).map_err(e)?;
    facts.set("source_velocity", weapon.source_velocity).map_err(e)?;
    table.set("weapon", facts).map_err(e)?;
    Ok(())
}

fn aim_target_into(lua: &Lua, table: &LuaTable, target: &AimTarget) -> Result<(), String> {
    let e = |error: LuaError| error.to_string();
    let facts = lua.create_table().map_err(e)?;
    facts.set("position", target.position.to_vec()).map_err(e)?;
    facts.set("point", target.point.to_vec()).map_err(e)?;
    facts.set("motion_vector", target.motion_vector.to_vec()).map_err(e)?;
    facts.set("mobile", target.mobile).map_err(e)?;
    table.set("target", facts).map_err(e)?;
    Ok(())
}

fn aim_ids_into(table: &LuaTable, source: &Option<String>, target: &Option<String>) -> Result<(), String> {
    let e = |error: LuaError| error.to_string();
    if let Some(id) = source {
        table.set("source_host_id", id.as_str()).map_err(e)?;
    }
    if let Some(id) = target {
        table.set("target_host_id", id.as_str()).map_err(e)?;
    }
    Ok(())
}

fn weapon_figures_table(lua: &Lua, figures: &WeaponFigures) -> Result<LuaTable, String> {
    let e = |error: LuaError| error.to_string();
    let table = lua.create_table().map_err(e)?;
    table.set("name", figures.name.as_str()).map_err(e)?;
    for (field, value) in [
        ("start_weight", figures.start_weight),
        ("final_weight", figures.final_weight),
        ("max_range", figures.max_range),
        ("muzzle_velocity", figures.muzzle_velocity),
        ("cruise_velocity", figures.cruise_velocity),
        ("g_max", figures.g_max),
        ("boost_time", figures.boost_time),
        ("cruise_time", figures.cruise_time),
        ("inhibit_time", figures.inhibit_time),
        ("drag_coefficient", figures.drag_coefficient),
        ("flightpath_deviation", figures.flightpath_deviation),
        ("boost_power", figures.boost_power),
        ("sustain_power", figures.sustain_power),
        ("sustain_time", figures.sustain_time),
        ("diameter", figures.diameter),
        ("max_altitude", figures.max_altitude),
        ("cruise_time_max_error", figures.cruise_time_max_error),
        ("detonation_radius", figures.detonation_radius),
        ("max_seeker_limit", figures.max_seeker_limit),
        ("max_launch_angle_error", figures.max_launch_angle_error),
        ("muzzle_velocity_max_error", figures.muzzle_velocity_max_error),
        ("circular_error_probable", figures.circular_error_probable),
        ("rate_of_fire", figures.rate_of_fire),
        ("inverse_rate_of_fire", figures.inverse_rate_of_fire),
    ] {
        table.set(field, value).map_err(e)?;
    }
    table.set("ignore_gravity", figures.ignore_gravity != 0).map_err(e)?;
    table.set("flight_profile_or_self_destr", figures.flight_profile_or_self_destr).map_err(e)?;
    table.set("parachute_type", figures.parachute_type).map_err(e)?;
    table.set("weapon_class", figures.weapon_class).map_err(e)?;
    table.set("survive_ground_impact", figures.survive_ground_impact).map_err(e)?;
    table.set("in_flight_collision_check", figures.in_flight_collision_check).map_err(e)?;
    table.set("guidance_type", figures.guidance_type).map_err(e)?;
    table.set("aiming_type", figures.aiming_type).map_err(e)?;
    table.set("decoy_type", figures.decoy_type).map_err(e)?;
    table.set("warhead_type", figures.warhead_type).map_err(e)?;
    table.set("hellfire_ii", figures.hellfire_ii).map_err(e)?;
    table.set("smoke_grenade", figures.smoke_grenade).map_err(e)?;
    table.set("submunition_count", figures.submunition_count).map_err(e)?;
    table.set("submunition_velocity_factor", figures.submunition_velocity_factor).map_err(e)?;
    table.set("acquire_forward", figures.acquire_forward).map_err(e)?;
    Ok(table)
}

/// One aiming request as a Lua table: `{ mode = "table" | "range" | "ready_update" | "ready_command" | "required" | "step" | "vectors" | "lead" | "ballistic" |
/// "intercept", ... }`. Vectors are arrays of numbers; a host identity is absent (nil) when the campaign roster has none for that end. A `table` or `range` request
/// carries the weapon's physical `figures` and the `first`/`last` flags of the boot pass: the host generates its own table (no cell is sent).
fn aim_request_table(lua: &Lua, request: &AimRequest) -> Result<LuaTable, String> {
    let e = |error: LuaError| error.to_string();
    let table = lua.create_table().map_err(e)?;
    match request {
        AimRequest::Table(data) => {
            table.set("mode", "table").map_err(e)?;
            table.set("weapon", data.weapon).map_err(e)?;
            table.set("figures", weapon_figures_table(lua, &data.figures)?).map_err(e)?;
            table.set("first", data.first).map_err(e)?;
            table.set("last", data.last).map_err(e)?;
            table.set("echo", data.echo).map_err(e)?;
            table.set("echo_cells", data.echo_cells).map_err(e)?;
        }
        AimRequest::Range(data) => {
            table.set("mode", "range").map_err(e)?;
            table.set("weapon", data.weapon).map_err(e)?;
            table.set("figures", weapon_figures_table(lua, &data.figures)?).map_err(e)?;
            table.set("first", data.first).map_err(e)?;
            table.set("last", data.last).map_err(e)?;
            table.set("echo", data.echo).map_err(e)?;
        }
        AimRequest::Figures(data) => {
            table.set("mode", "figures").map_err(e)?;
            table.set("weapon", data.weapon).map_err(e)?;
            table.set("figures", weapon_figures_table(lua, &data.figures)?).map_err(e)?;
        }
        AimRequest::WeaponStep(data) => {
            let state = &data.state;
            table
                .set(
                    "mode",
                    match data.kind {
                        AimStepKind::Step => "weapon_step",
                        AimStepKind::Birth => "weapon_birth",
                        AimStepKind::Decoy => "weapon_decoy",
                        AimStepKind::Seduce => "weapon_seduce",
                        AimStepKind::Reacquire => "weapon_reacquire",
                    },
                )
                .map_err(e)?;
            if data.kind == AimStepKind::Birth {
                table.set("submunition", state.submunition).map_err(e)?;
            }
            if matches!(data.kind, AimStepKind::Seduce | AimStepKind::Reacquire) {
                table.set("other_weapon", state.other_weapon).map_err(e)?;
            }
            if data.kind == AimStepKind::Seduce {
                table.set("coefficient", state.coefficient).map_err(e)?;
                table.set("effectiveness", state.effectiveness.to_vec()).map_err(e)?;
            }
            table.set("echo", data.echo).map_err(e)?;
            table.set("weapon", data.weapon).map_err(e)?;
            table.set("delta_time", data.delta_time).map_err(e)?;
            table.set("system_time", data.system_time).map_err(e)?;
            table.set("map_min", data.map_min.to_vec()).map_err(e)?;
            table.set("map_max", data.map_max.to_vec()).map_err(e)?;
            table.set("position", state.position.to_vec()).map_err(e)?;
            table.set("attitude", state.attitude.to_vec()).map_err(e)?;
            table.set("motion_vector", state.motion_vector.to_vec()).map_err(e)?;
            table.set("velocity", state.velocity).map_err(e)?;
            table.set("weapon_lifetime", state.weapon_lifetime).map_err(e)?;
            table.set("decoy_timer", state.decoy_timer).map_err(e)?;
            table.set("missile_phase", state.missile_phase).map_err(e)?;
            table.set("loal_mode", state.loal_mode).map_err(e)?;
            table.set("weapon_on_ground", state.weapon_on_ground).map_err(e)?;
            table.set("parachute_status", state.parachute_status).map_err(e)?;
            table.set("kill_code", state.kill_code).map_err(e)?;
            table.set("entity_samples", state.entity_samples.to_vec()).map_err(e)?;
            table.set("has_launcher", state.has_launcher).map_err(e)?;
            table.set("has_target", state.has_target).map_err(e)?;
            table.set("server", state.server).map_err(e)?;
            let answers = lua.create_table().map_err(e)?;
            for (index, answer) in data.answers.iter().enumerate() {
                answers.set(index + 1, aim_event_table(lua, answer)?).map_err(e)?;
            }
            table.set("answers", answers).map_err(e)?;
            table.set("draws", data.draws.clone()).map_err(e)?;
        }
        AimRequest::Animate(data) => {
            table.set("mode", "animate").map_err(e)?;
            table.set("weapon", data.weapon).map_err(e)?;
            table.set("parachute_status", data.parachute_status).map_err(e)?;
            table.set("weapon_on_ground", data.weapon_on_ground).map_err(e)?;
            table.set("weapon_lifetime", data.weapon_lifetime).map_err(e)?;
            table.set("motion_vector_y", data.motion_vector_y).map_err(e)?;
        }
        AimRequest::Release(data) => {
            table.set("mode", "release").map_err(e)?;
            table.set("weapon", data.weapon).map_err(e)?;
            table.set("delta_time", data.delta_time).map_err(e)?;
            table.set("current_time", data.current_time).map_err(e)?;
            if let Some(last_shot) = data.last_shot {
                table.set("last_shot", last_shot).map_err(e)?;
            }
            table.set("not_ai", data.not_ai).map_err(e)?;
        }
        AimRequest::Forecast(data) => {
            // the damage a weapon is capable of against a target, with no weapon in flight: everything by value, nothing to keep
            table.set("mode", "forecast").map_err(e)?;
            table.set("weapon", data.weapon).map_err(e)?;
            table.set("weapon_warhead", data.weapon_warhead).map_err(e)?;
            table.set("warhead", data.warhead).map_err(e)?;
            table.set("damage_capability", data.damage_capability).map_err(e)?;
            table.set("armor_penetration_capability", data.armor_penetration_capability).map_err(e)?;
            table.set("burst_duration", data.burst_duration).map_err(e)?;
            table.set("armour_level", data.armour_level).map_err(e)?;
            table.set("target_not_ai", data.target_not_ai).map_err(e)?;
            table.set("player_modifier", data.player_modifier).map_err(e)?;
            table.set("damage_modifier", data.damage_modifier).map_err(e)?;
            table.set("velocity", data.velocity).map_err(e)?;
            table.set("echo", data.echo).map_err(e)?;
        }
        AimRequest::Damage(data) => {
            // one hit that reached a damage handler, everything by value; the sample of the campaign's stream is present once the host has asked for it
            table.set("mode", "damage").map_err(e)?;
            table.set("weapon", data.weapon).map_err(e)?;
            table.set("weapon_warhead", data.weapon_warhead).map_err(e)?;
            table.set("warhead", data.warhead).map_err(e)?;
            table.set("damage_capability", data.damage_capability).map_err(e)?;
            table.set("armor_penetration_capability", data.armor_penetration_capability).map_err(e)?;
            table.set("burst_duration", data.burst_duration).map_err(e)?;
            table.set("weapon_speed", data.weapon_speed).map_err(e)?;
            table.set("weapon_cruise_velocity", data.weapon_cruise_velocity).map_err(e)?;
            table.set("damage_modifier", data.damage_modifier).map_err(e)?;
            table.set("armour_level", data.armour_level).map_err(e)?;
            table.set("armour_easy", data.armour_easy).map_err(e)?;
            table.set("armour_model", data.armour_model).map_err(e)?;
            table.set("armour_front", data.armour_front).map_err(e)?;
            table.set("armour_rear", data.armour_rear).map_err(e)?;
            table.set("armour_side", data.armour_side).map_err(e)?;
            table.set("weapon_position", data.weapon_position.to_vec()).map_err(e)?;
            table.set("target_position", data.target_position.to_vec()).map_err(e)?;
            table.set("target_attitude", data.target_attitude.to_vec()).map_err(e)?;
            table.set("target_not_ai", data.target_not_ai).map_err(e)?;
            if let Some(modifier) = data.player_modifier {
                table.set("player_modifier", modifier).map_err(e)?;
            }
            table.set("has_launcher", data.has_launcher).map_err(e)?;
            table.set("multiplier", data.multiplier).map_err(e)?;
            table.set("invulnerable", data.invulnerable).map_err(e)?;
            table.set("cheat_guard", data.cheat_guard).map_err(e)?;
            table.set("damage_level", data.damage_level).map_err(e)?;
            table.set("initial_damage_level", data.initial_damage_level).map_err(e)?;
            table.set("critical_model", data.critical_model).map_err(e)?;
            if let Some(draw) = data.draw {
                table.set("draw", draw).map_err(e)?;
            }
            table.set("echo", data.echo).map_err(e)?;
        }
        AimRequest::Health(data) => {
            // a health fact that is not a hit, by value; the sample of the campaign's stream is present once a timed roll has asked for it
            table.set("mode", "health").map_err(e)?;
            table
                .set(
                    "op",
                    match data.op {
                        1 => "classify",
                        2 => "assess",
                        3 => "timed",
                        _ => "hit_eligible",
                    },
                )
                .map_err(e)?;
            table.set("critical_model", data.critical_model).map_err(e)?;
            table.set("damage_level", data.damage_level).map_err(e)?;
            table.set("initial_damage_level", data.initial_damage_level).map_err(e)?;
            table.set("old_damage_level", data.old_damage_level).map_err(e)?;
            table.set("explosive", data.explosive).map_err(e)?;
            table.set("flammable", data.flammable).map_err(e)?;
            table.set("delta_time", data.delta_time).map_err(e)?;
            table.set("aircraft_class", data.aircraft_class).map_err(e)?;
            table.set("self_hit", data.self_hit).map_err(e)?;
            table.set("target_not_ai", data.target_not_ai).map_err(e)?;
            if let Some(draw) = data.draw {
                table.set("draw", draw).map_err(e)?;
            }
        }
        AimRequest::Impact(data) => {
            // what a weapon's death does to the world: the kill code and the facts the dispatch reads, by value.  `terrain_draws_crater` is present once the host has asked for the terrain
            table.set("mode", "impact").map_err(e)?;
            table.set("kill_code", data.kill_code).map_err(e)?;
            table.set("has_target", data.has_target).map_err(e)?;
            table.set("target_alive", data.target_alive).map_err(e)?;
            table.set("position", data.position.to_vec()).map_err(e)?;
            if let Some(draws) = data.terrain_draws_crater {
                table.set("terrain_draws_crater", draws).map_err(e)?;
            }
        }
        AimRequest::Blast(data) => {
            // one area blast: the explosion, the map's sector figures and, once the host has asked for one, the window of a sector's list it was given.  Each object is named for this
            // blast only by its handle
            table.set("mode", "blast").map_err(e)?;
            table.set("position", data.position.to_vec()).map_err(e)?;
            table.set("damage_capability", data.damage_capability).map_err(e)?;
            table.set("warhead_type", data.warhead_type).map_err(e)?;
            table.set("has_target", data.has_target).map_err(e)?;
            table.set("sector_side_length", data.sector_side_length).map_err(e)?;
            table.set("min_x_sector", data.min_x_sector).map_err(e)?;
            table.set("max_x_sector", data.max_x_sector).map_err(e)?;
            table.set("min_z_sector", data.min_z_sector).map_err(e)?;
            table.set("max_z_sector", data.max_z_sector).map_err(e)?;
            table.set("echo", data.echo).map_err(e)?;
            if let Some(window) = &data.window {
                let window_table = lua.create_table().map_err(e)?;
                window_table.set("sx", window.sx).map_err(e)?;
                window_table.set("sz", window.sz).map_err(e)?;
                window_table.set("after", window.after).map_err(e)?;
                window_table.set("list_ends", window.end).map_err(e)?;
                let objects = lua.create_table().map_err(e)?;
                for (index, entry) in window.entries.iter().enumerate() {
                    let object = lua.create_table().map_err(e)?;
                    object.set("handle", entry.handle).map_err(e)?;
                    object.set("entity_type", entry.entity_type).map_err(e)?;
                    object.set("readable", entry.readable).map_err(e)?;
                    object.set("alive", entry.alive).map_err(e)?;
                    object.set("has_position", entry.has_position).map_err(e)?;
                    object.set("rocket", entry.rocket).map_err(e)?;
                    object.set("excluded", entry.excluded).map_err(e)?;
                    object.set("position", entry.position.to_vec()).map_err(e)?;
                    objects.set(index + 1, object).map_err(e)?;
                }
                window_table.set("objects", objects).map_err(e)?;
                table.set("window", window_table).map_err(e)?;
            }
        }
        AimRequest::ReadyUpdate(data) => {
            table.set("mode", "ready_update").map_err(e)?;
            aim_ids_into(&table, &data.source, &None)?;
            table.set("state", data.state).map_err(e)?;
            table.set("rate", data.rate).map_err(e)?;
            table.set("delta_time", data.delta_time).map_err(e)?;
        }
        AimRequest::ReadyCommand(data) => {
            table.set("mode", "ready_command").map_err(e)?;
            aim_ids_into(&table, &data.source, &None)?;
            table.set("state", data.state).map_err(e)?;
            table.set("open", data.open).map_err(e)?;
        }
        AimRequest::Required(data) => {
            table.set("mode", "required").map_err(e)?;
            aim_ids_into(&table, &data.source, &None)?;
            table.set("device_attitude", data.device_attitude.to_vec()).map_err(e)?;
            match &data.limits {
                Some(limits) => {
                    table.set("heading_min", limits.heading_min).map_err(e)?;
                    table.set("heading_max", limits.heading_max).map_err(e)?;
                    table.set("pitch_min", limits.pitch_min).map_err(e)?;
                    table.set("pitch_max", limits.pitch_max).map_err(e)?;
                    table.set("climb", limits.climb).map_err(e)?;
                }
                None => table.set("unbounded", true).map_err(e)?,
            }
            match &data.tracking {
                Some(track) => {
                    let facts = lua.create_table().map_err(e)?;
                    facts.set("device_position", track.device_position.to_vec()).map_err(e)?;
                    facts.set("point", track.point.to_vec()).map_err(e)?;
                    facts.set("pitch", track.pitch).map_err(e)?;
                    table.set("tracking", facts).map_err(e)?;
                }
                None => table.set("intercept_vector", data.intercept_vector.to_vec()).map_err(e)?,
            }
        }
        AimRequest::Step(data) => {
            table.set("mode", "step").map_err(e)?;
            aim_ids_into(&table, &data.source, &None)?;
            table.set("current_heading", data.current_heading).map_err(e)?;
            table.set("current_pitch", data.current_pitch).map_err(e)?;
            table.set("required_heading", data.required_heading).map_err(e)?;
            table.set("required_pitch", data.required_pitch).map_err(e)?;
            table.set("heading_min", data.heading_min).map_err(e)?;
            table.set("heading_max", data.heading_max).map_err(e)?;
            table.set("heading_rate", data.heading_rate).map_err(e)?;
            table.set("pitch_rate", data.pitch_rate).map_err(e)?;
            table.set("delta_time", data.delta_time).map_err(e)?;
        }
        AimRequest::Vectors(data) => {
            table.set("mode", "vectors").map_err(e)?;
            table.set("echo", data.echo).map_err(e)?;
            aim_ids_into(&table, &data.source, &data.target_id)?;
            aim_world_into(&table, &data.world)?;
            table.set("device_position", data.device_position.to_vec()).map_err(e)?;
            table.set("device_attitude", data.device_attitude.to_vec()).map_err(e)?;
            table.set("source_gunship", data.source_gunship).map_err(e)?;
            if let Some(point) = &data.tracking {
                table.set("tracking_point", point.to_vec()).map_err(e)?;
            }
            if let Some(weapon) = &data.weapon {
                aim_weapon_into(lua, &table, weapon)?;
            }
            if let Some(target) = &data.target {
                aim_target_into(lua, &table, target)?;
            }
            if let Some(draw) = data.draw {
                table.set("draw", draw).map_err(e)?;
            }
        }
        AimRequest::Lead(data) => {
            table.set("mode", "lead").map_err(e)?;
            table.set("echo", data.echo).map_err(e)?;
            aim_ids_into(&table, &data.source, &data.target_id)?;
            aim_world_into(&table, &data.world)?;
            table.set("weapon", data.weapon).map_err(e)?;
            table.set("source_gunship", data.source_gunship).map_err(e)?;
            table.set("parent_velocity", data.parent_velocity).map_err(e)?;
            table.set("device_position", data.device_position.to_vec()).map_err(e)?;
            aim_target_into(lua, &table, &data.target)?;
            if let Some(draw) = data.draw {
                table.set("draw", draw).map_err(e)?;
            }
        }
        AimRequest::Ballistic(data) => {
            table.set("mode", "ballistic").map_err(e)?;
            table.set("echo", data.echo).map_err(e)?;
            table.set("weapon", data.weapon).map_err(e)?;
            table.set("range", data.range).map_err(e)?;
            table.set("height_diff_or_pitch", data.height_diff_or_pitch).map_err(e)?;
            table.set("simplified", data.simplified).map_err(e)?;
            table.set("fixed_pitch", data.fixed_pitch).map_err(e)?;
            table.set("parent_velocity", data.parent_velocity).map_err(e)?;
        }
        AimRequest::Intercept(data) => {
            table.set("mode", "intercept").map_err(e)?;
            table.set("echo", data.echo).map_err(e)?;
            aim_ids_into(&table, &data.source, &data.target_id)?;
            aim_weapon_into(lua, &table, &data.weapon)?;
            table.set("device_position", data.device_position.to_vec()).map_err(e)?;
            aim_target_into(lua, &table, &data.target)?;
        }
    }
    Ok(table)
}

/// A weapon step's lookup, answer or action as a Lua table: `{ kind, a, b, c, d, f = { 12 numbers } }`.
fn aim_event_table(lua: &Lua, event: &AimEvent) -> Result<LuaTable, String> {
    let e = |error: LuaError| error.to_string();
    let table = lua.create_table().map_err(e)?;
    table.set("kind", event.kind).map_err(e)?;
    table.set("a", event.a).map_err(e)?;
    table.set("b", event.b).map_err(e)?;
    table.set("c", event.c).map_err(e)?;
    table.set("d", event.d).map_err(e)?;
    table.set("f", event.f.to_vec()).map_err(e)?;
    Ok(table)
}

/// A list of a weapon step's events from the host's answer: tables `{ kind, a, b, c, d, f }`, each integer exact and each float exact and finite.
fn aim_event_list(lua: &Lua, table: &LuaTable, field: &str) -> Result<Vec<AimEvent>, String> {
    let list = match table.get::<LuaValue>(field).map_err(|e| e.to_string())? {
        LuaValue::Nil => return Ok(Vec::new()),
        LuaValue::Table(list) => list,
        other => return Err(format!("aim result {field} must be a list, got {}", other.type_name())),
    };
    let mut events = Vec::new();
    for value in list.sequence_values::<LuaValue>() {
        let LuaValue::Table(event) = value.map_err(|e| e.to_string())? else {
            return Err(format!("aim result {field} holds a value that is not an event table"));
        };
        events.push(AimEvent {
            kind: aim_integer(&event, "kind")?,
            a: aim_integer(&event, "a")?,
            b: aim_integer(&event, "b")?,
            c: aim_integer(&event, "c")?,
            d: aim_integer(&event, "d")?,
            f: exact_numbers::<12>(lua, &event, "f").map_err(|e| format!("aim result {field} {e}"))?,
        });
    }
    Ok(events)
}

fn aim_integer(table: &LuaTable, field: &str) -> Result<i32, String> {
    match table.get::<LuaValue>(field).map_err(|e| e.to_string())? {
        LuaValue::Integer(value) => i32::try_from(value).map_err(|_| format!("aim result {field} {value} is out of range")),
        LuaValue::Number(value) if value.fract() == 0.0 && value.abs() < 2147483648.0 => Ok(value as i32),
        other => Err(format!("aim result {field} must be an integer, got {}", other.type_name())),
    }
}

fn aim_boolean(table: &LuaTable, field: &str) -> Result<bool, String> {
    match table.get::<LuaValue>(field).map_err(|e| e.to_string())? {
        LuaValue::Boolean(value) => Ok(value),
        other => Err(format!("aim result {field} must be a boolean, got {}", other.type_name())),
    }
}

fn aim_echo(lua: &Lua, table: &LuaTable) -> Result<Option<Vec<AimEvent>>, String> {
    let list = match table.get::<LuaValue>("echo").map_err(|e| e.to_string())? {
        LuaValue::Nil => return Ok(None),
        LuaValue::Table(list) => list,
        other => return Err(format!("aim result echo must be a list, got {}", other.type_name())),
    };
    let mut events = Vec::new();
    for value in list.sequence_values::<LuaValue>() {
        let LuaValue::Table(event) = value.map_err(|e| e.to_string())? else {
            return Err("aim result echo holds a value that is not an event table".into());
        };
        let kind = match event.get::<LuaValue>("kind").map_err(|e| e.to_string())? {
            LuaValue::String(text) => match &*text.to_str().map_err(|e| e.to_string())? {
                "intercept" => 1,
                "lead_path" => 2,
                "lead_range" => 3,
                "lead_iteration" => 4,
                "ballistic" => 5,
                "lead_result" => 6,
                "table_flight" => 7,
                "range_result" => 8,
                "forecast" => 9,
                "damage" => 10,
                "blast" => 11,
                "step_path" => 20,
                "step_submunitions" => 21,
                "step_tracking" => 22,
                "step_guidance" => 23,
                "step_guidance_source" => 24,
                "step_tgt_branch" => 25,
                "step_tgt_end" => 26,
                "step_intercept" => 27,
                "step_guided" => 28,
                "step_precision" => 29,
                "step_ground" => 30,
                "step_impact" => 31,
                "step_contact_any" => 32,
                "step_contact_trees" => 33,
                "step_birth_fuze_begin" => 34,
                "step_birth_fuze_solved" => 35,
                "step_birth_dispersion" => 36,
                other => {
                    return Err(format!(
                        "aim result echo kind {other:?} is not intercept, lead_path, lead_range, lead_iteration, ballistic, lead_result, table_flight, range_result, forecast, damage, blast or a step_* kind"
                    ))
                }
            },
            other => return Err(format!("aim result echo kind must be a string, got {}", other.type_name())),
        };
        events.push(AimEvent {
            kind,
            a: aim_integer(&event, "a")?,
            b: aim_integer(&event, "b")?,
            c: aim_integer(&event, "c")?,
            d: aim_integer(&event, "d")?,
            f: exact_numbers::<12>(lua, &event, "f").map_err(|e| format!("aim result echo {e}"))?,
        });
    }
    Ok(Some(events))
}

/// The host's answer to an aiming request: a table with `state = "done"` (or, to a vectors or lead request, `"need_draw"`) and the fields of that
/// request, each mandatory, exact and finite; anything else is an error, never a default.
fn call_aim(lua: &Lua, function: &LuaFunction, request: AimRequest) -> Result<AimReply, String> {
    let argument = aim_request_table(lua, &request)?;
    let answer: LuaValue = function.call(argument).map_err(|e| e.to_string())?;
    let LuaValue::Table(table) = answer else {
        return Err(format!("aim must return a result table, got {}", answer.type_name()));
    };
    let state: LuaValue = table.get("state").map_err(|e| e.to_string())?;
    let LuaValue::String(state) = state else {
        return Err(format!("aim result state must be a string, got {}", state.type_name()));
    };
    let state = state.to_str().map_err(|e| e.to_string())?.to_owned();
    let number = |field: &str| -> Result<f32, String> { Ok(exact_numbers::<1>(lua, &table, field).map_err(|e| format!("aim result {e}"))?[0]) };
    let vector = |field: &str| -> Result<[f32; 3], String> { exact_numbers::<3>(lua, &table, field).map_err(|e| format!("aim result {e}")) };
    let mut reply = AimReply::default();
    match state.as_str() {
        "need_draw" => {
            if !matches!(
                request,
                AimRequest::Vectors(_) | AimRequest::Lead(_) | AimRequest::WeaponStep(_) | AimRequest::Damage(_) | AimRequest::Health(_)
            ) {
                return Err("aim result need_draw answers a request that cannot take a draw".into());
            }
            reply.need_draw = true;
            return Ok(reply);
        }
        "need_lookup" => {
            match &request {
                AimRequest::WeaponStep(_) => {
                    reply.need_lookup = true;
                    reply.lookups = aim_event_list(lua, &table, "lookups")?;
                    return Ok(reply);
                }
                AimRequest::Impact(_) => {
                    // the only thing an impact question can ask for is the terrain fact at the impact position
                    match table.get::<LuaValue>("need").map_err(|e| e.to_string())? {
                        LuaValue::String(need) if &*need.to_str().map_err(|e| e.to_string())? == "terrain" => {}
                        _ => return Err("aim result need_lookup answers an impact question with a need that is not the terrain".into()),
                    }
                    reply.need_lookup = true;
                    reply.impact_need_terrain = true;
                    return Ok(reply);
                }
                AimRequest::Blast(_) => {
                    // the only thing a blast question can ask for is the window of a sector's list: { sx, sz, after }
                    let LuaValue::Table(need) = table.get::<LuaValue>("need").map_err(|e| e.to_string())? else {
                        return Err("aim result need_lookup answers a blast question with no window to supply".into());
                    };
                    reply.need_lookup = true;
                    reply.blast_radius = number("radius")?;
                    reply.blast_need = Some((aim_integer(&need, "sx")?, aim_integer(&need, "sz")?, aim_integer(&need, "after")?));
                    reply.echo = aim_echo(lua, &table)?;
                    return Ok(reply);
                }
                _ => return Err("aim result need_lookup answers a request that cannot take lookups".into()),
            }
        }
        "done" => {}
        other => return Err(format!("aim result state {other:?} is not done, need_draw or need_lookup")),
    }
    match &request {
        AimRequest::Table(data) => {
            // the host's table is its own: the answer carries nothing, except the diagnostic echo the test fixture build asked for (the table's cells, little-endian
            // single-precision pairs). The echo is optional: a host with a model of its own has no EECH table to echo, and its absence (nil) is explicit, not an error
            if data.echo {
                let bytes = match table.get::<LuaValue>("echo_cells").map_err(|e| e.to_string())? {
                    LuaValue::Nil => None,
                    LuaValue::String(bytes) => Some(bytes),
                    other => return Err(format!("aim result echo_cells must be a string of cells or absent, got {}", other.type_name())),
                };
                if let Some(bytes) = bytes {
                    let guard = bytes.as_bytes();
                    let (words, rest) = guard.as_chunks::<4>();
                    if !rest.is_empty() {
                        return Err("aim result echo_cells is not a whole number of single-precision cells".into());
                    }
                    reply.table_cells = Some(words.iter().map(|word| f32::from_le_bytes(*word)).collect());
                }
            }
        }
        AimRequest::Range(_) | AimRequest::Figures(_) => {}
        AimRequest::WeaponStep(_) => {
            reply.actions = aim_event_list(lua, &table, "actions")?;
        }
        AimRequest::Animate(_) => {
            reply.animate_status = aim_integer(&table, "parachute_status")?;
            reply.animate_lifetime = match table.get::<LuaValue>("weapon_lifetime").map_err(|e| e.to_string())? {
                LuaValue::Nil => None,
                _ => Some(number("weapon_lifetime")?),
            };
        }
        AimRequest::Release(_) => {
            reply.release_refuse = aim_boolean(&table, "refuse")?;
            reply.release_burst_size = aim_integer(&table, "burst_size")?;
            reply.release_last_shot = match table.get::<LuaValue>("last_shot").map_err(|e| e.to_string())? {
                LuaValue::Nil => None,
                LuaValue::Integer(value) => Some(u32::try_from(value).map_err(|_| format!("aim result last_shot {value} is out of range"))?),
                LuaValue::Number(value) if value.fract() == 0.0 && (0.0..4_294_967_296.0).contains(&value) => Some(value as u32),
                other => return Err(format!("aim result last_shot must be an unsigned integer or absent, got {}", other.type_name())),
            };
        }
        AimRequest::Forecast(_) => {
            // the capability is the host's own integer; the echo (the test fixture build's) is read below
            reply.forecast_capability = aim_integer(&table, "capability")?;
        }
        AimRequest::Damage(_) => {
            // the host's whole answer to the hit, each field mandatory and exact: a missing or mistyped one is an error, never a default
            reply.damage_exited = aim_boolean(&table, "exited")?;
            reply.damage_capability = aim_integer(&table, "capability")?;
            reply.damage_written = aim_boolean(&table, "written")?;
            reply.damage_new_level = aim_integer(&table, "new_level")?;
            reply.damage_critical_before = aim_integer(&table, "critical_before")?;
            reply.damage_critical_after = aim_integer(&table, "critical_after")?;
            reply.damage_bad_modifier = aim_boolean(&table, "bad_modifier")?;
        }
        AimRequest::Health(_) => {
            // the host's whole answer, each field mandatory and exact: a missing or mistyped one is an error, never a default
            reply.health_critical_class = aim_integer(&table, "critical_class")?;
            reply.health_crossed = aim_boolean(&table, "crossed")?;
            reply.health_recovered = aim_boolean(&table, "recovered")?;
            reply.health_eject = aim_boolean(&table, "eject")?;
            reply.health_eligible = aim_boolean(&table, "eligible")?;
            reply.health_rolled = aim_boolean(&table, "rolled")?;
            reply.health_hit = aim_boolean(&table, "hit")?;
            reply.health_new_level = aim_integer(&table, "new_level")?;
            reply.health_fatal = aim_boolean(&table, "fatal")?;
        }
        AimRequest::Impact(_) => {
            // the plan, each field mandatory and exact: a missing or mistyped one is an error, never a default
            reply.impact_invalid = aim_boolean(&table, "invalid")?;
            reply.impact_crater = aim_boolean(&table, "crater")?;
            reply.impact_direct_hit = aim_boolean(&table, "direct_hit")?;
            reply.impact_blasts = aim_integer(&table, "blasts")?;
            reply.impact_exclude_target = aim_boolean(&table, "exclude_target")?;
        }
        AimRequest::Blast(_) => {
            // the radius with every answer, then the action: nothing more to do ("none"), a collision for the object, or a rocket destroyed.  An action names its object (the handle it
            // was offered under) and the modifier; the walk's end names neither
            reply.blast_radius = number("radius")?;
            let action = match table.get::<LuaValue>("action").map_err(|e| e.to_string())? {
                LuaValue::String(text) => text.to_str().map_err(|e| e.to_string())?.to_owned(),
                other => return Err(format!("aim result action must be a string, got {}", other.type_name())),
            };
            reply.blast_action = match action.as_str() {
                "none" => 0,
                "collision" => 1,
                "intercept" => 2,
                other => return Err(format!("aim result action {other:?} is not none, collision or intercept")),
            };
            if reply.blast_action != 0 {
                reply.blast_handle = aim_integer(&table, "handle")?;
                reply.blast_modifier = number("modifier")?;
            }
        }
        AimRequest::ReadyUpdate(_) => {
            reply.ready_state = number("ready_state")?;
            reply.phase = aim_integer(&table, "phase")?;
        }
        AimRequest::ReadyCommand(_) => {
            reply.ready_state = number("ready_state")?;
        }
        AimRequest::Required(_) => {
            reply.required_heading = number("required_heading")?;
            reply.required_pitch = number("required_pitch")?;
        }
        AimRequest::Step(_) => {
            [reply.delta_heading, reply.delta_pitch, reply.new_heading, reply.new_pitch] =
                exact_fields::<4>(lua, &table, "delta_heading,delta_pitch,new_heading,new_pitch").map_err(|e| format!("aim result {e}"))?;
        }
        AimRequest::Vectors(_) => {
            reply.result = aim_boolean(&table, "valid")?;
            reply.weapon_vector = vector("weapon_vector")?;
            reply.target_vector = vector("target_vector")?;
            reply.intercept_vector = vector("intercept_vector")?;
            reply.intercept_range = number("intercept_range")?;
        }
        AimRequest::Lead(_) => {
            reply.result = aim_boolean(&table, "ok")?;
            reply.intercept_point = vector("intercept_point")?;
            reply.angle = number("angle")?;
            // the time of flight is a partial output: absent (nil) when the host produced none, whatever `ok` says; present it must be a finite number
            reply.time_of_flight = match table.get::<LuaValue>("time_of_flight").map_err(|e| e.to_string())? {
                LuaValue::Nil => None,
                _ => Some(number("time_of_flight")?),
            };
        }
        AimRequest::Ballistic(_) => {
            reply.result = aim_boolean(&table, "ok")?;
            if reply.result {
                reply.angle = number("angle")?;
                reply.time_of_flight = Some(number("time_of_flight")?);
            }
        }
        AimRequest::Intercept(_) => {
            reply.intercept_point = vector("intercept_point")?;
        }
    }
    reply.echo = aim_echo(lua, &table)?;
    Ok(reply)
}

fn sense_scan_name(scan: SenseScan) -> &'static str {
    match scan {
        SenseScan::AirToAir => "air_to_air",
        SenseScan::AirToSurface => "air_to_surface",
        SenseScan::SurfaceToAir => "surface_to_air",
        SenseScan::SurfaceToSurface => "surface_to_surface",
    }
}

/// One message of the host-owned sensing dialogue as a Lua table: `{ mode = "begin" | "candidate" | "draw" | "signature" | "end", scan, ... }`.
/// Positions are arrays of three numbers; a host identity is absent (nil) when the campaign roster has none for that end; the emitters and the
/// helicopter facts are tables, present only when the campaign has them.
fn sense_message_table(lua: &Lua, message: &SenseMessage) -> Result<LuaTable, String> {
    let e = |error: LuaError| error.to_string();
    let table = lua.create_table().map_err(e)?;
    match message {
        SenseMessage::Begin(begin) => {
            table.set("mode", "begin").map_err(e)?;
            table.set("scan", sense_scan_name(begin.scan)).map_err(e)?;
            table.set("source_position", begin.source_position.to_vec()).map_err(e)?;
            table.set("scan_range", begin.scan_range).map_err(e)?;
            if begin.scan == SenseScan::SurfaceToAir {
                table.set("scan_ceiling", begin.scan_ceiling).map_err(e)?;
                table.set("scan_floor", begin.scan_floor).map_err(e)?;
            }
            if let Some(id) = &begin.source_host_id {
                table.set("source_host_id", id.as_str()).map_err(e)?;
            }
        }
        SenseMessage::Candidate(candidate) => {
            table.set("mode", "candidate").map_err(e)?;
            table.set("scan", sense_scan_name(candidate.scan)).map_err(e)?;
            table.set("target_position", candidate.target_position.to_vec()).map_err(e)?;
            table.set("target_player", candidate.target_player).map_err(e)?;
            if let Some(factor) = candidate.difficulty_factor {
                table.set("difficulty_factor", factor).map_err(e)?;
            }
            if let Some(altitude) = candidate.radar_altitude {
                table.set("radar_altitude", altitude).map_err(e)?;
            }
            if let Some(id) = &candidate.target_host_id {
                table.set("target_host_id", id.as_str()).map_err(e)?;
            }
        }
        SenseMessage::Draw { draw } => {
            table.set("mode", "draw").map_err(e)?;
            table.set("draw", *draw).map_err(e)?;
        }
        SenseMessage::Signature(signature) => {
            table.set("mode", "signature").map_err(e)?;
            table.set("scan", sense_scan_name(signature.scan)).map_err(e)?;
            if let Some(emitters) = signature.emitters {
                let emitter_table = lua.create_table().map_err(e)?;
                emitter_table.set("radar_on", emitters.radar_on).map_err(e)?;
                emitter_table.set("radar_jammer_on", emitters.radar_jammer_on).map_err(e)?;
                emitter_table.set("infra_red_jammer_on", emitters.infra_red_jammer_on).map_err(e)?;
                table.set("emitters", emitter_table).map_err(e)?;
            }
            if let Some(helicopter) = signature.helicopter {
                let helicopter_table = lua.create_table().map_err(e)?;
                helicopter_table.set("weapon_bay_closed", helicopter.weapon_bay_closed).map_err(e)?;
                helicopter_table.set("stub_wings_attached", helicopter.stub_wings_attached).map_err(e)?;
                helicopter_table.set("undercarriage_up", helicopter.undercarriage_up).map_err(e)?;
                table.set("helicopter", helicopter_table).map_err(e)?;
            }
        }
        SenseMessage::End { scan } => {
            table.set("mode", "end").map_err(e)?;
            table.set("scan", sense_scan_name(*scan)).map_err(e)?;
        }
    }
    Ok(table)
}

/// The host's answer to a sensing message: `{ state = "open" | "accept" | "reject" | "need_draw" | "need_signature" | "closed", reason, target_range, echo }`.
/// A rejection names why the candidate was not perceived (`reason`): `not_perceived`, the verdict of a host's own model, which every scan accepts, or
/// one of the original EECH model's predicates (range, range_adjusted, altitude, tolerance) that the reference host reports, each legal only for the
/// scans whose original had it. An acceptance carries the measured `target_range`; every number is exact. The optional `echo` table is read only by
/// the test fixture build.
fn call_sense(lua: &Lua, function: &LuaFunction, message: SenseMessage) -> Result<SenseReply, String> {
    let argument = sense_message_table(lua, &message)?;
    let answer: LuaValue = function.call(argument).map_err(|e| e.to_string())?;
    let LuaValue::Table(table) = answer else {
        return Err(format!("sense must return a result table, got {}", answer.type_name()));
    };
    let state: LuaValue = table.get("state").map_err(|e| e.to_string())?;
    let LuaValue::String(state) = state else {
        return Err(format!("sense result state must be a string, got {}", state.type_name()));
    };
    let state_text = state.to_str().map_err(|e| e.to_string())?;
    let state = match &*state_text {
        "open" => SenseState::Open,
        "accept" => SenseState::Accept,
        "need_draw" => SenseState::NeedDraw,
        "need_signature" => SenseState::NeedSignature,
        "closed" => SenseState::Closed,
        "reject" => {
            let reason: LuaValue = table.get("reason").map_err(|e| e.to_string())?;
            let LuaValue::String(reason) = reason else {
                return Err(format!("sense result reason must be a string, got {}", reason.type_name()));
            };
            let reason_text = reason.to_str().map_err(|e| e.to_string())?;
            SenseState::Reject(match &*reason_text {
                "range" => SenseReason::Range,
                "range_adjusted" => SenseReason::RangeAdjusted,
                "altitude" => SenseReason::Altitude,
                "tolerance" => SenseReason::Tolerance,
                "not_perceived" => SenseReason::NotPerceived,
                other => {
                    return Err(format!(
                        "sense result reason {other:?} is not range, range_adjusted, altitude, tolerance or not_perceived"
                    ))
                }
            })
        }
        other => {
            return Err(format!(
                "sense result state {other:?} is not open, accept, reject, need_draw, need_signature or closed"
            ))
        }
    };
    let target_range = match table.get::<LuaValue>("target_range").map_err(|e| e.to_string())? {
        LuaValue::Nil => None,
        _ => Some(exact_numbers::<1>(lua, &table, "target_range").map_err(|e| format!("sense result {e}"))?[0]),
    };
    #[cfg(feature = "sensor-fixture")]
    let echo = match table.get::<LuaValue>("echo").map_err(|e| e.to_string())? {
        LuaValue::Table(echo) => {
            let one = |name: &str| -> Result<f32, String> { Ok(exact_numbers::<1>(lua, &echo, name).map_err(|e| format!("sense result echo {e}"))?[0]) };
            Some(eech_engine::SenseEcho {
                ceiling: one("ceiling")?,
                floor: one("floor")?,
                adjusted_range: one("adjusted_range")?,
                stealth_factor: one("stealth_factor")?,
                stealth_limit: one("stealth_limit")?,
                tolerance_limit: exact_f64_bits(lua, &echo, "tolerance_limit").map_err(|e| format!("sense result echo {e}"))?,
            })
        }
        LuaValue::Nil => None,
        other => return Err(format!("sense result echo must be a table, got {}", other.type_name())),
    };
    #[cfg(not(feature = "sensor-fixture"))]
    let echo = None;
    Ok(SenseReply { state, target_range, echo })
}

fn call_move_vehicle(lua: &Lua, function: &LuaFunction, step: MovementStep) -> Result<MovementResult, String> {
    let mode = step.mode;
    let value = lua.to_value_with(&step, options()).map_err(|e| e.to_string())?;
    let answer: LuaValue = function.call(value).map_err(|e| e.to_string())?;
    let LuaValue::Table(table) = answer else {
        return Err(format!("move_vehicle must return a result table, got {}", answer.type_name()));
    };
    read_move_result(lua, mode, &table)
}

/// The host's answer to one `mode` step, every field exact; the fields a mode's answer must carry are mandatory, so an
/// absent answer is an error rather than a default of zero or false.
fn read_move_result(lua: &Lua, mode: MovementMode, table: &LuaTable) -> Result<MovementResult, String> {
    // the helicopter lifecycle modes (shared by the fixed wing's dying steps): every one answers with the rotor and the
    // motion vector the host retains
    let lifecycle_step = matches!(
        mode,
        MovementMode::RotorSet
            | MovementMode::RotorAlive
            | MovementMode::RotorDown
            | MovementMode::RotorIdle
            | MovementMode::Fall
            | MovementMode::FallGravity
            | MovementMode::Contact
            | MovementMode::Settle
            | MovementMode::Tumble
            | MovementMode::CrashBegin
            | MovementMode::CrashEnd
            | MovementMode::Impact
    );
    // the fixed-wing lifecycle modes that are new: they answer with the motion vector but carry no rotor
    let fixed_wing_step = matches!(
        mode,
        MovementMode::FwSettle | MovementMode::FwDeathAttitude | MovementMode::FwCrashCandidate | MovementMode::FwCrashVelocity | MovementMode::FwCrashAttitude
    );
    // the vehicle lifecycle modes (routed, anti-air, ship): every one answers with the vector the host has claimed, which
    // the campaign applies, and carries no rotor
    let vehicle_step = matches!(
        mode,
        MovementMode::GvDeathCandidate
            | MovementMode::GvDeathDecay
            | MovementMode::GvFallCandidate
            | MovementMode::GvFallGravity
            | MovementMode::GvFallSettle
            | MovementMode::GvImpact
            | MovementMode::ShipSink
            | MovementMode::VehicleVectorSeed
            | MovementMode::GvFallVector
            | MovementMode::GvFallCommit
    );
    // the modes whose answer carries bits the campaign acts on: an absent answer is an error, never 0
    let flags_step = matches!(
        mode,
        MovementMode::GvFallSettle | MovementMode::FwDeathAttitude | MovementMode::FwCrashVelocity | MovementMode::FwCrashAttitude | MovementMode::Expire
    );
    // the hosted helicopter flow: every step answers with the vector the host holds
    let hosted_helicopter_step = matches!(
        mode,
        MovementMode::HelicopterPlan
            | MovementMode::HelicopterGuide
            | MovementMode::HelicopterGuidedBegin
            | MovementMode::HelicopterGuidedEnd
            | MovementMode::HelicopterAbandon
    );
    let aircraft_step = lifecycle_step
        || fixed_wing_step
        || hosted_helicopter_step
        || matches!(
            mode,
            MovementMode::FixedWingMove | MovementMode::FixedWingLookahead | MovementMode::HelicopterBegin | MovementMode::HelicopterEnd
        );
    // the lifecycle modes whose answer decides something in the campaign
    let decides = matches!(
        mode,
        MovementMode::RotorAlive | MovementMode::Contact | MovementMode::Settle | MovementMode::CrashEnd | MovementMode::FwSettle
    );
    let [velocity] = exact_numbers::<1>(lua, table, "velocity").map_err(|e| format!("move_vehicle result {e}"))?;
    let rotated = match table.get::<LuaValue>("rotated").map_err(|e| e.to_string())? {
        LuaValue::Nil => false,
        LuaValue::Boolean(rotated) => rotated,
        other => return Err(format!("move_vehicle result rotated must be a boolean, got {}", other.type_name())),
    };
    let pending = match table.get::<LuaValue>("pending").map_err(|e| e.to_string())? {
        LuaValue::Nil => false,
        LuaValue::Boolean(pending) => pending,
        other => return Err(format!("move_vehicle result pending must be a boolean, got {}", other.type_name())),
    };
    // a helicopter's first phase reports whether it moved; no other step has an answer for it
    let moved = match table.get::<LuaValue>("moved").map_err(|e| e.to_string())? {
        LuaValue::Nil if matches!(mode, MovementMode::HelicopterBegin | MovementMode::HelicopterGuidedBegin) || decides => {
            return Err("move_vehicle result for this step must carry moved".into())
        }
        LuaValue::Nil => false,
        LuaValue::Boolean(moved) => moved,
        other => return Err(format!("move_vehicle result moved must be a boolean, got {}", other.type_name())),
    };
    // a routed or held member's live step returns no motion vector; an aircraft's or a wreck's answer must carry the one it holds
    let motion_vector = match table.get::<LuaValue>("motion_vector").map_err(|e| e.to_string())? {
        LuaValue::Nil if aircraft_step => return Err("move_vehicle result for an aircraft step must carry its motion_vector".into()),
        LuaValue::Nil if vehicle_step => return Err("move_vehicle result for a vehicle lifecycle step must carry its motion_vector".into()),
        LuaValue::Nil => [0.0; 3],
        _ => exact_numbers(lua, table, "motion_vector").map_err(|e| format!("move_vehicle result {e}"))?,
    };
    let rotor = match table.get::<LuaValue>("rotor").map_err(|e| e.to_string())? {
        LuaValue::Nil if lifecycle_step => return Err("move_vehicle result for a helicopter lifecycle step must carry its rotor".into()),
        LuaValue::Nil => [0.0; 3],
        _ => exact_numbers(lua, table, "rotor").map_err(|e| format!("move_vehicle result {e}"))?,
    };
    let altitude = match table.get::<LuaValue>("altitude").map_err(|e| e.to_string())? {
        LuaValue::Nil if mode == MovementMode::Contact => return Err("move_vehicle result for a contact step must carry its altitude".into()),
        LuaValue::Nil => 0.0,
        _ => exact_numbers::<1>(lua, table, "altitude").map_err(|e| format!("move_vehicle result {e}"))?[0],
    };
    // the per-mode bits are checked by the engine against the mode; here only an exact integer (or none) is accepted
    let flags = match table.get::<LuaValue>("flags").map_err(|e| e.to_string())? {
        LuaValue::Nil if flags_step => return Err("move_vehicle result for this step must carry its flags".into()),
        LuaValue::Nil => 0,
        LuaValue::Integer(flags) => i32::try_from(flags).map_err(|_| format!("move_vehicle result flags {flags} is out of range"))?,
        LuaValue::Number(flags) if flags.fract() == 0.0 && flags.abs() <= f64::from(i32::MAX) => flags as i32,
        other => return Err(format!("move_vehicle result flags must be an integer, got {}", other.type_name())),
    };
    // a creation placement answers the Euler angles the host placed the member at; no other step has them
    let angles = match table.get::<LuaValue>("angles").map_err(|e| e.to_string())? {
        LuaValue::Nil if mode == MovementMode::CreatePlace => return Err("move_vehicle result for a create_place step must carry its angles".into()),
        LuaValue::Nil if mode == MovementMode::HelicopterGuidedEnd => {
            return Err("move_vehicle result for a helicopter_guided_end step must carry its angles".into())
        }
        LuaValue::Nil if mode == MovementMode::PersonFace => return Err("move_vehicle result for a person_face step must carry its angles".into()),
        LuaValue::Nil => [0.0; 3],
        _ => exact_numbers(lua, table, "angles").map_err(|e| format!("move_vehicle result {e}"))?,
    };
    // the hosted helicopter's two answers: the transitional lookup plan after the plan step (and, for a fixed-wing look-ahead,
    // the one terrain sample the host asks for), the guidance after the guide step. Each is mandatory for its mode and absent
    // from every other, so a stray table is an error rather than ignored
    let plans_query = matches!(mode, MovementMode::HelicopterPlan | MovementMode::FixedWingLookahead);
    let query = match table.get::<LuaValue>("query").map_err(|e| e.to_string())? {
        LuaValue::Nil if plans_query => return Err(format!("move_vehicle result for a {mode:?} step must carry its query")),
        LuaValue::Nil => None,
        LuaValue::Table(_) if !plans_query => return Err("move_vehicle result carries a query for a step that plans none".into()),
        LuaValue::Table(query) => {
            let structure_scan = match query.get::<LuaValue>("structure_scan").map_err(|e| e.to_string())? {
                LuaValue::Boolean(scan) => scan,
                other => return Err(format!("move_vehicle result query structure_scan must be a boolean, got {}", other.type_name())),
            };
            let count = match query.get::<LuaValue>("terrain_sample_count").map_err(|e| e.to_string())? {
                LuaValue::Integer(count) => usize::try_from(count).ok(),
                LuaValue::Number(count) if count.fract() == 0.0 && (0.0..=1.0e6).contains(&count) => Some(count as usize),
                _ => None,
            }
            .ok_or("move_vehicle result query terrain_sample_count must be a non-negative integer")?;
            // the engine refuses a count above the sample capacity; the array always carries every slot's x and z
            let [x0, z0, x1, z1] = exact_numbers::<4>(lua, &query, "terrain_sample").map_err(|e| format!("move_vehicle result query {e}"))?;
            Some(HelicopterQuery {
                structure_scan,
                structure_position: exact_numbers(lua, &query, "structure_position").map_err(|e| format!("move_vehicle result query {e}"))?,
                terrain_sample_count: count,
                terrain_sample: [[x0, z0], [x1, z1]],
            })
        }
        other => return Err(format!("move_vehicle result query must be a table, got {}", other.type_name())),
    };
    let guidance = match table.get::<LuaValue>("guidance").map_err(|e| e.to_string())? {
        LuaValue::Nil if mode == MovementMode::HelicopterGuide => return Err("move_vehicle result for a helicopter_guide step must carry its guidance".into()),
        LuaValue::Nil => None,
        LuaValue::Table(_) if mode != MovementMode::HelicopterGuide => return Err("move_vehicle result carries guidance for a step that guides none".into()),
        LuaValue::Table(guidance) => {
            let field = |name: &str| {
                exact_numbers::<1>(lua, &guidance, name)
                    .map(|[value]| value)
                    .map_err(|e| format!("move_vehicle result guidance {e}"))
            };
            Some(HelicopterGuidanceAnswer {
                lift: field("lift")?,
                waypoint: exact_numbers(lua, &guidance, "waypoint").map_err(|e| format!("move_vehicle result guidance {e}"))?,
                required_height: field("required_height")?,
                world_acceleration: exact_numbers(lua, &guidance, "world_acceleration").map_err(|e| format!("move_vehicle result guidance {e}"))?,
                wp_vec: exact_numbers(lua, &guidance, "wp_vec").map_err(|e| format!("move_vehicle result guidance {e}"))?,
            })
        }
        other => return Err(format!("move_vehicle result guidance must be a table, got {}", other.type_name())),
    };
    // an expiry answers the host's timer, which the campaign mirrors; no other step has one
    let death_timer = match table.get::<LuaValue>("death_timer").map_err(|e| e.to_string())? {
        LuaValue::Nil if mode == MovementMode::Expire => return Err("move_vehicle result for an expire step must carry its death_timer".into()),
        LuaValue::Nil => 0.0,
        _ => exact_numbers::<1>(lua, table, "death_timer").map_err(|e| format!("move_vehicle result {e}"))?[0],
    };
    Ok(MovementResult {
        pose: VehiclePose {
            position: exact_numbers(lua, table, "position").map_err(|e| format!("move_vehicle result {e}"))?,
            velocity,
            attitude: exact_numbers(lua, table, "attitude").map_err(|e| format!("move_vehicle result {e}"))?,
        },
        rotated,
        motion_vector,
        pending,
        moved,
        rotor,
        altitude,
        flags,
        angles,
        death_timer,
        query,
        guidance,
    })
}

impl LuaUserData for LuaEngine {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("frame", |lua, this, milliseconds: u32| {
            let bind_mobile = this.bind_mobile.clone();
            let observe_loss = this.observe_loss.clone();
            let supply_reached = this.supply_reached.clone();
            let move_vehicle = this.move_vehicle.clone();
            let line_of_sight = this.line_of_sight.clone();
            let sense = this.sense.clone();
            let aim = this.aim.clone();
            let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
            let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
            let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
            let mut mover = |step| call_move_vehicle(lua, &move_vehicle, step);
            let mut sighter = |message| call_line_of_sight(lua, &line_of_sight, message);
            let mut sensor = |message| call_sense(lua, &sense, message);
            let mut aimer = |request| call_aim(lua, &aim, request);
            this.engine
                .frame(
                    milliseconds,
                    &mut binder,
                    &mut observer,
                    &mut reach,
                    &mut mover,
                    &mut sighter,
                    &mut sensor,
                    &mut aimer,
                )
                .map_err(lua_error)
        });
        // TEST-ONLY (feature `loss-fixture`, EEDCS-41): not part of the shipping module.
        #[cfg(feature = "loss-fixture")]
        methods.add_method("loss_fixture", |lua, this, entity_id: i32| {
            let bind_mobile = this.bind_mobile.clone();
            let observe_loss = this.observe_loss.clone();
            let supply_reached = this.supply_reached.clone();
            let move_vehicle = this.move_vehicle.clone();
            let line_of_sight = this.line_of_sight.clone();
            let sense = this.sense.clone();
            let aim = this.aim.clone();
            let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
            let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
            let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
            let mut mover = |step| call_move_vehicle(lua, &move_vehicle, step);
            let mut sighter = |message| call_line_of_sight(lua, &line_of_sight, message);
            let mut sensor = |message| call_sense(lua, &sense, message);
            let mut aimer = |request| call_aim(lua, &aim, request);
            this.engine
                .loss_fixture(
                    entity_id,
                    &mut binder,
                    &mut observer,
                    &mut reach,
                    &mut mover,
                    &mut sighter,
                    &mut sensor,
                    &mut aimer,
                )
                .map_err(lua_error)
        });
        // TEST-ONLY (feature `loss-fixture`, EEDCS-41): releases the tasks targeting a member's group (setup); not part of the shipping module.
        #[cfg(feature = "loss-fixture")]
        methods.add_method("loss_release", |lua, this, entity_id: i32| {
            let bind_mobile = this.bind_mobile.clone();
            let observe_loss = this.observe_loss.clone();
            let supply_reached = this.supply_reached.clone();
            let move_vehicle = this.move_vehicle.clone();
            let line_of_sight = this.line_of_sight.clone();
            let sense = this.sense.clone();
            let aim = this.aim.clone();
            let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
            let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
            let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
            let mut mover = |step| call_move_vehicle(lua, &move_vehicle, step);
            let mut sighter = |message| call_line_of_sight(lua, &line_of_sight, message);
            let mut sensor = |message| call_sense(lua, &sense, message);
            let mut aimer = |request| call_aim(lua, &aim, request);
            this.engine
                .loss_release(
                    entity_id,
                    &mut binder,
                    &mut observer,
                    &mut reach,
                    &mut mover,
                    &mut sighter,
                    &mut sensor,
                    &mut aimer,
                )
                .map_err(lua_error)
        });
        // TEST-ONLY (feature `loss-fixture`, EEDCS-41): read-only force-info and regen state; not part of the shipping module.
        #[cfg(feature = "loss-fixture")]
        methods.add_method("loss_state", |lua, this, side: i32| {
            let values = this.engine.loss_state(side).map_err(lua_error)?;
            loss_state_table(lua, &values)
        });
        // TEST-ONLY (feature `loss-fixture`, EEDCS-41): read-only withhold eligibility of one hosted member; not part of the shipping module.
        #[cfg(feature = "loss-fixture")]
        methods.add_method("loss_shape", |_, this, entity_id: i32| this.engine.loss_shape(entity_id).map_err(lua_error));
        // TEST-ONLY (feature `loss-fixture`, EEDCS-41): read-only task dependents and guide stack of a hosted member's group; not part of the shipping module.
        #[cfg(feature = "loss-fixture")]
        methods.add_method("loss_group", |_, this, entity_id: i32| this.engine.loss_group(entity_id).map_err(lua_error));
        // TEST-ONLY (feature `los-fixture`, EEDCS-42): the run-wide fold of line-of-sight queries; not part of the shipping module.
        #[cfg(feature = "los-fixture")]
        methods.add_method("los_trace", |lua, this, ()| {
            let trace = this.engine.los_trace().map_err(lua_error)?;
            los_trace_table(lua, &trace)
        });
        // TEST-ONLY (feature `los-fixture`, EEDCS-42): clears that fold; not part of the shipping module.
        #[cfg(feature = "los-fixture")]
        methods.add_method("los_trace_reset", |_, this, ()| this.engine.los_trace_reset().map_err(lua_error));
        // TEST-ONLY (feature `los-fixture`, EEDCS-42): the deterministic line-of-sight sweep between two live entities; not part of the shipping module.
        #[cfg(feature = "los-fixture")]
        methods.add_method("los_probe", |lua, this, (source, target, seed, count): (i32, i32, u32, i32)| {
            let bind_mobile = this.bind_mobile.clone();
            let observe_loss = this.observe_loss.clone();
            let supply_reached = this.supply_reached.clone();
            let move_vehicle = this.move_vehicle.clone();
            let line_of_sight = this.line_of_sight.clone();
            let sense = this.sense.clone();
            let aim = this.aim.clone();
            let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
            let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
            let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
            let mut mover = |step| call_move_vehicle(lua, &move_vehicle, step);
            let mut sighter = |message| call_line_of_sight(lua, &line_of_sight, message);
            let mut sensor = |message| call_sense(lua, &sense, message);
            let mut aimer = |request| call_aim(lua, &aim, request);
            let trace = this
                .engine
                .los_probe(
                    source,
                    target,
                    seed,
                    count,
                    &mut binder,
                    &mut observer,
                    &mut reach,
                    &mut mover,
                    &mut sighter,
                    &mut sensor,
                    &mut aimer,
                )
                .map_err(lua_error)?;
            los_trace_table(lua, &trace)
        });
        // TEST-ONLY (feature `aim-fixture`, EEDCS-44): the run-wide report of the physical weapon-aiming code; not part of the shipping module.
        #[cfg(feature = "aim-fixture")]
        methods.add_method("aim_trace", |_, this, ()| this.engine.aim_trace().map_err(lua_error));
        // TEST-ONLY (feature `aim-fixture`, EEDCS-44): the pose and target link of one host-identified entity; not part of the shipping module.
        #[cfg(feature = "aim-fixture")]
        methods.add_method("aim_person_query", |_, this, host_id: String| {
            this.engine.aim_person_query(&host_id).map_err(lua_error)
        });
        // TEST-ONLY (feature `aim-fixture`, EEDCS-44): the controlled person case, given the same complete host callbacks as a frame; not part of the shipping module.
        #[cfg(feature = "aim-fixture")]
        methods.add_method("aim_person_case", |lua, this, (person_id, target_id): (String, String)| {
            let bind_mobile = this.bind_mobile.clone();
            let observe_loss = this.observe_loss.clone();
            let supply_reached = this.supply_reached.clone();
            let move_vehicle = this.move_vehicle.clone();
            let line_of_sight = this.line_of_sight.clone();
            let sense = this.sense.clone();
            let aim = this.aim.clone();
            let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
            let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
            let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
            let mut mover = |step| call_move_vehicle(lua, &move_vehicle, step);
            let mut sighter = |message| call_line_of_sight(lua, &line_of_sight, message);
            let mut sensor = |message| call_sense(lua, &sense, message);
            let mut aimer = |request| call_aim(lua, &aim, request);
            this.engine
                .aim_person_case(
                    &person_id,
                    &target_id,
                    &mut binder,
                    &mut observer,
                    &mut reach,
                    &mut mover,
                    &mut sighter,
                    &mut sensor,
                    &mut aimer,
                )
                .map_err(lua_error)
        });
        // TEST-ONLY (feature `aim-fixture`, EEDCS-44): the campaign's own lead-and-ballistic function asked once, with the caller's time-of-flight variable set to a
        // sentinel; the report says whether it still holds it. Given the same complete host callbacks as a frame; not part of the shipping module.
        #[cfg(feature = "aim-fixture")]
        methods.add_method(
            "aim_lead_probe",
            |lua, this, (source_id, target_id, weapon, sentinel): (String, String, i32, f64)| {
                let bind_mobile = this.bind_mobile.clone();
                let observe_loss = this.observe_loss.clone();
                let supply_reached = this.supply_reached.clone();
                let move_vehicle = this.move_vehicle.clone();
                let line_of_sight = this.line_of_sight.clone();
                let sense = this.sense.clone();
                let aim = this.aim.clone();
                let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
                let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
                let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
                let mut mover = |step| call_move_vehicle(lua, &move_vehicle, step);
                let mut sighter = |message| call_line_of_sight(lua, &line_of_sight, message);
                let mut sensor = |message| call_sense(lua, &sense, message);
                let mut aimer = |request| call_aim(lua, &aim, request);
                this.engine
                    .aim_lead_probe(
                        &source_id,
                        &target_id,
                        weapon,
                        sentinel as f32,
                        &mut binder,
                        &mut observer,
                        &mut reach,
                        &mut mover,
                        &mut sighter,
                        &mut sensor,
                        &mut aimer,
                    )
                    .map_err(lua_error)
            },
        );
        // TEST-ONLY (feature `aim-fixture`, EEDCS-44): clears that trace; not part of the shipping module.
        #[cfg(feature = "aim-fixture")]
        methods.add_method("aim_trace_reset", |_, this, ()| this.engine.aim_trace_reset().map_err(lua_error));
        // TEST-ONLY (feature `weapon-fixture`, EEDCS-46): the run-wide report of the physical combat-resolution code; not part of the shipping module.
        #[cfg(feature = "weapon-fixture")]
        methods.add_method("cmb_trace", |_, this, ()| this.engine.cmb_trace().map_err(lua_error));
        // TEST-ONLY (feature `weapon-fixture`, EEDCS-46): controlled combat cases of the original campaign that launch nothing (op 0 census, op 1 set a health percentage), given the
        // same complete host as a frame because the health case sets a health and asks the critical predicate, which the host answers; not part of the shipping module.
        #[cfg(feature = "weapon-fixture")]
        methods.add_method("cmb_case_state", |lua, this, (op, kind, ordinal, percent): (i32, String, i32, i32)| {
            let bind_mobile = this.bind_mobile.clone();
            let observe_loss = this.observe_loss.clone();
            let supply_reached = this.supply_reached.clone();
            let move_vehicle = this.move_vehicle.clone();
            let line_of_sight = this.line_of_sight.clone();
            let sense = this.sense.clone();
            let aim = this.aim.clone();
            let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
            let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
            let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
            let mut mover = |step| call_move_vehicle(lua, &move_vehicle, step);
            let mut sighter = |message| call_line_of_sight(lua, &line_of_sight, message);
            let mut sensor = |message| call_sense(lua, &sense, message);
            let mut aimer = |request| call_aim(lua, &aim, request);
            this.engine
                .cmb_case_state(
                    op,
                    &kind,
                    ordinal,
                    percent,
                    &mut binder,
                    &mut observer,
                    &mut reach,
                    &mut mover,
                    &mut sighter,
                    &mut sensor,
                    &mut aimer,
                )
                .map_err(lua_error)
        });
        // TEST-ONLY (feature `weapon-fixture`, EEDCS-46): controlled combat cases that launch a real weapon (op 1 collide, op 2 blast), given the same complete host as a frame; not part
        // of the shipping module.
        #[cfg(feature = "weapon-fixture")]
        methods.add_method(
            "cmb_case_act",
            |lua, this, (op, kind, ordinal, launcher_id, weapon, amount, kill_code, weaponless): (i32, String, i32, String, i32, f64, i32, bool)| {
                let bind_mobile = this.bind_mobile.clone();
                let observe_loss = this.observe_loss.clone();
                let supply_reached = this.supply_reached.clone();
                let move_vehicle = this.move_vehicle.clone();
                let line_of_sight = this.line_of_sight.clone();
                let sense = this.sense.clone();
                let aim = this.aim.clone();
                let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
                let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
                let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
                let mut mover = |step| call_move_vehicle(lua, &move_vehicle, step);
                let mut sighter = |message| call_line_of_sight(lua, &line_of_sight, message);
                let mut sensor = |message| call_sense(lua, &sense, message);
                let mut aimer = |request| call_aim(lua, &aim, request);
                this.engine
                    .cmb_case_act(
                        op,
                        &kind,
                        ordinal,
                        &launcher_id,
                        weapon,
                        amount,
                        kill_code,
                        weaponless,
                        &mut binder,
                        &mut observer,
                        &mut reach,
                        &mut mover,
                        &mut sighter,
                        &mut sensor,
                        &mut aimer,
                    )
                    .map_err(lua_error)
            },
        );
        // TEST-ONLY (feature `weapon-fixture`, EEDCS-46): the ordered combat event log, drained from the cursor; not part of the shipping module.
        #[cfg(feature = "weapon-fixture")]
        methods.add_method("cmb_events", |_, this, ()| this.engine.cmb_events().map_err(lua_error));
        // TEST-ONLY (feature `weapon-fixture`, EEDCS-45): the run-wide report of the physical weapon-execution code; not part of the shipping module.
        #[cfg(feature = "weapon-fixture")]
        methods.add_method("wpn_trace", |_, this, ()| this.engine.wpn_trace().map_err(lua_error));
        // TEST-ONLY (feature `weapon-fixture`, EEDCS-45): the ordered weapon event log, drained from the cursor; not part of the shipping module.
        #[cfg(feature = "weapon-fixture")]
        methods.add_method("wpn_events", |_, this, ()| this.engine.wpn_events().map_err(lua_error));
        // TEST-ONLY (feature `weapon-fixture`, EEDCS-45): the state of one host-identified entity and its count of one weapon type; not part of the shipping module.
        #[cfg(feature = "weapon-fixture")]
        methods.add_method("wpn_case_query", |_, this, (host_id, weapon): (String, i32)| {
            this.engine.wpn_case_query(&host_id, weapon).map_err(lua_error)
        });
        // TEST-ONLY (feature `weapon-fixture`, EEDCS-45): the entity kind of a roster host id; not part of the shipping module.
        #[cfg(feature = "weapon-fixture")]
        methods.add_method("wpn_case_kind", |_, this, host_id: String| {
            this.engine.wpn_case_kind(&host_id).map_err(lua_error)
        });
        // TEST-ONLY (feature `weapon-fixture`, EEDCS-45): one physical launch attempt through the campaign's own launch call, given the same complete host callbacks as a frame; not
        // part of the shipping module.
        #[cfg(feature = "weapon-fixture")]
        methods.add_method(
            "wpn_case_launch",
            |lua, this, (launcher_id, target_id, weapon, salvo_count): (String, String, i32, i32)| {
                let bind_mobile = this.bind_mobile.clone();
                let observe_loss = this.observe_loss.clone();
                let supply_reached = this.supply_reached.clone();
                let move_vehicle = this.move_vehicle.clone();
                let line_of_sight = this.line_of_sight.clone();
                let sense = this.sense.clone();
                let aim = this.aim.clone();
                let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
                let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
                let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
                let mut mover = |step| call_move_vehicle(lua, &move_vehicle, step);
                let mut sighter = |message| call_line_of_sight(lua, &line_of_sight, message);
                let mut sensor = |message| call_sense(lua, &sense, message);
                let mut aimer = |request| call_aim(lua, &aim, request);
                this.engine
                    .wpn_case_launch(
                        &launcher_id,
                        &target_id,
                        weapon,
                        salvo_count,
                        &mut binder,
                        &mut observer,
                        &mut reach,
                        &mut mover,
                        &mut sighter,
                        &mut sensor,
                        &mut aimer,
                    )
                    .map_err(lua_error)
            },
        );
        // TEST-ONLY (feature `sensor-fixture`, EEDCS-43): the run-wide report of the AI target scans; not part of the shipping module.
        #[cfg(feature = "sensor-fixture")]
        methods.add_method("sensor_trace", |_, this, ()| this.engine.sensor_trace().map_err(lua_error));
        // TEST-ONLY (feature `sensor-fixture`, EEDCS-43): clears that trace; not part of the shipping module.
        #[cfg(feature = "sensor-fixture")]
        methods.add_method("sensor_trace_reset", |_, this, ()| this.engine.sensor_trace_reset().map_err(lua_error));
        // TEST-ONLY (feature `sensor-fixture`, EEDCS-43): the read-only scan sweep; not part of the shipping module. The surface-to-air scan asks the host
        // for line of sight, so the sweep is given the same complete host callbacks as a frame.
        #[cfg(feature = "sensor-fixture")]
        methods.add_method("sensor_probe", |lua, this, ()| {
            let bind_mobile = this.bind_mobile.clone();
            let observe_loss = this.observe_loss.clone();
            let supply_reached = this.supply_reached.clone();
            let move_vehicle = this.move_vehicle.clone();
            let line_of_sight = this.line_of_sight.clone();
            let sense = this.sense.clone();
            let aim = this.aim.clone();
            let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
            let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
            let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
            let mut mover = |step| call_move_vehicle(lua, &move_vehicle, step);
            let mut sighter = |message| call_line_of_sight(lua, &line_of_sight, message);
            let mut sensor = |message| call_sense(lua, &sense, message);
            let mut aimer = |request| call_aim(lua, &aim, request);
            this.engine
                .sensor_probe(&mut binder, &mut observer, &mut reach, &mut mover, &mut sighter, &mut sensor, &mut aimer)
                .map_err(lua_error)
        });
        // TEST-ONLY (feature `dependent-fixture`, EEDCS-40): not part of the shipping module.
        #[cfg(feature = "dependent-fixture")]
        methods.add_method(
            "dependent_fixture",
            |lua, this, (op, parent, child, x, y, z): (String, i32, i32, Option<f64>, Option<f64>, Option<f64>)| {
                let code = match op.as_str() {
                    "link" => 1,
                    "move" => 2,
                    "sink" => 3,
                    _ => return Err(mlua::Error::runtime("dependent_fixture: op is link, move or sink")),
                };
                let bind_mobile = this.bind_mobile.clone();
                let observe_loss = this.observe_loss.clone();
                let supply_reached = this.supply_reached.clone();
                let move_vehicle = this.move_vehicle.clone();
                let line_of_sight = this.line_of_sight.clone();
                let sense = this.sense.clone();
                let aim = this.aim.clone();
                let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
                let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
                let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
                let mut mover = |step| call_move_vehicle(lua, &move_vehicle, step);
                let mut sighter = |message| call_line_of_sight(lua, &line_of_sight, message);
                let mut sensor = |message| call_sense(lua, &sense, message);
                let mut aimer = |request| call_aim(lua, &aim, request);
                this.engine
                    .dependent_fixture(
                        code,
                        parent,
                        child,
                        [x.unwrap_or(0.0), y.unwrap_or(0.0), z.unwrap_or(0.0)],
                        &mut binder,
                        &mut observer,
                        &mut reach,
                        &mut mover,
                        &mut sighter,
                        &mut sensor,
                        &mut aimer,
                    )
                    .map_err(lua_error)
            },
        );
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
    let move_vehicle = match config.get::<LuaValue>("move_vehicle")? {
        LuaValue::Function(function) => function,
        LuaValue::Nil => return Err(LuaError::runtime("eech: missing host move_vehicle callback")),
        value => return Err(LuaError::runtime(format!("eech: move_vehicle must be a function, got {}", value.type_name()))),
    };
    let line_of_sight = match config.get::<LuaValue>("line_of_sight")? {
        LuaValue::Function(function) => function,
        LuaValue::Nil => return Err(LuaError::runtime("eech: missing host line_of_sight callback")),
        value => return Err(LuaError::runtime(format!("eech: line_of_sight must be a function, got {}", value.type_name()))),
    };
    let sense = match config.get::<LuaValue>("sense")? {
        LuaValue::Function(function) => function,
        LuaValue::Nil => return Err(LuaError::runtime("eech: missing host sense callback")),
        value => return Err(LuaError::runtime(format!("eech: sense must be a function, got {}", value.type_name()))),
    };
    let aim = match config.get::<LuaValue>("aim")? {
        LuaValue::Function(function) => function,
        LuaValue::Nil => return Err(LuaError::runtime("eech: missing host aim callback")),
        value => return Err(LuaError::runtime(format!("eech: aim must be a function, got {}", value.type_name()))),
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
    let mut mover = |step| call_move_vehicle(lua, &move_vehicle, step);
    let mut sighter = |message| call_line_of_sight(lua, &line_of_sight, message);
    let mut sensor = |message| call_sense(lua, &sense, message);
    let mut aimer = |request| call_aim(lua, &aim, request);
    let engine = Engine::boot(
        &config,
        &mut binder,
        &mut observer,
        &mut reach,
        &mut mover,
        &mut sighter,
        &mut sensor,
        &mut aimer,
    )
    .map_err(lua_error)?;
    Ok(LuaEngine {
        engine,
        bind_mobile,
        observe_loss,
        supply_reached,
        move_vehicle,
        line_of_sight,
        sense,
        aim,
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

/// TEST-ONLY (feature `los-fixture`): lays out a line-of-sight fold as a Lua table. The hash is a 16-digit hex string, because a
/// Lua number cannot hold 64 bits; `by_criteria` maps each criteria value that was queried to `{ calls, visible }`.
#[cfg(feature = "los-fixture")]
fn los_trace_table(lua: &Lua, trace: &eech_engine::LosTrace) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;
    table.set("hash", format!("{:016x}", trace.hash))?;
    table.set("calls", trace.calls)?;
    table.set("visible", trace.visible)?;
    table.set("skipped", trace.skipped)?;
    table.set("lookup_hash", format!("{:016x}", trace.lookup_hash))?;
    table.set("lookups", trace.lookups)?;
    let by_criteria = lua.create_table()?;
    for (criteria, (calls, visible)) in trace.by_criteria_calls.iter().zip(trace.by_criteria_visible.iter()).enumerate() {
        if *calls > 0 {
            let row = lua.create_table()?;
            row.set("calls", *calls)?;
            row.set("visible", *visible)?;
            by_criteria.set(criteria as i64, row)?;
        }
    }
    table.set("by_criteria", by_criteria)?;
    Ok(table)
}

/// TEST-ONLY (feature `loss-fixture`): lays out `Engine::loss_state` values as a Lua table. Every read is bounds-checked, so a
/// short or inconsistent buffer is an error and not a panic.
#[cfg(feature = "loss-fixture")]
fn loss_state_table(lua: &Lua, values: &[i32]) -> mlua::Result<mlua::Table> {
    let short = || mlua::Error::runtime("loss_state: the engine returned a short state buffer");
    let at = |i: usize| values.get(i).copied().ok_or_else(short);
    let span = |from: usize, len: usize| values.get(from..from + len).map(<[i32]>::to_vec).ok_or_else(short);
    let (categories, regens) = (at(1)? as usize, at(2)? as usize);
    let queues = 3 + categories;
    let state = lua.create_table()?;
    state.set("losses", at(0)?)?;
    state.set("hardware", span(3, categories)?)?;
    state.set("regen", span(queues, regens)?)?;
    state.set("regen_front", span(queues + regens, regens)?)?;
    state.set("regen_size", span(queues + 2 * regens, regens)?)?;
    // the slot written last in each queue, as { type, sub_type, group }, or false for an empty queue
    let last = lua.create_table()?;
    for r in 0..regens {
        let slot = span(queues + 3 * regens + 3 * r, 3)?;
        let empty = slot.first().is_none_or(|t| *t < 0);
        last.set(
            r + 1,
            if empty {
                mlua::Value::Boolean(false)
            } else {
                mlua::Value::Table(lua.create_sequence_from(slot)?)
            },
        )?;
    }
    state.set("regen_last", last)?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answer(lua: &Lua, fields: &str) -> mlua::Result<LuaTable> {
        lua.load(format!(
            "return {{ position = {{0, 1, 2}}, velocity = 3, attitude = {{1, 0, 0, 0, 1, 0, 0, 0, 1}}, {fields} }}"
        ))
        .eval()
    }

    // The fixture build's echo of a hit carries the observer's four intermediate floats as raw bits in finite halves (a hit with no direction has a not-a-number aspect armour); the
    // reader keeps them exactly, and a not-a-number sent as a float is still refused: finite validation is not weakened.
    #[test]
    fn a_damage_echo_of_raw_bit_halves_is_read_exactly_and_a_not_a_number_float_is_still_refused() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let table: LuaTable = lua
            .load("return { echo = { { kind = 'damage', a = 0, b = 0, c = 3, d = 0, f = { 0, 15, 0, 0.75, 0, 0, 16752, 0, 65472, 0, 16192, 0 } } } }")
            .eval()?;
        let events = aim_echo(&lua, &table)?.ok_or("no echo")?;
        let [event] = events.as_slice() else {
            return Err("the echo is not one event".into());
        };
        assert_eq!((event.kind, event.a, event.b, event.c, event.d), (10, 0, 0, 3, 0));
        assert_eq!(event.f, [0.0, 15.0, 0.0, 0.75, 0.0, 0.0, 16752.0, 0.0, 65472.0, 0.0, 16192.0, 0.0]);
        let bits = |index: usize| Some(((*event.f.get(4 + 2 * index)? as u32) << 16) | (*event.f.get(5 + 2 * index)? as u32));
        assert_eq!((bits(1), bits(2), bits(3)), (Some(0x4170_0000), Some(0xFFC0_0000), Some(0x3F40_0000)));
        // the old representation, a NaN as a float, is refused at this boundary
        let bad: LuaTable = lua
            .load("return { echo = { { kind = 'damage', a = 0, b = 0, c = 3, d = 0, f = { 0, 15, 0 / 0, 0.75, 0, 0, 0, 0, 0, 0, 0, 0 } } } }")
            .eval()?;
        assert!(aim_echo(&lua, &bad).is_err());
        Ok(())
    }

    // The health question reaches the host's Lua as a table with its op by name and its facts by value; the answer is mandatory field by field and a timed roll may ask for its sample.
    #[test]
    fn a_health_question_reaches_lua_by_value_and_its_answer_is_read_field_by_field() -> Result<(), Box<dyn std::error::Error>> {
        use eech_engine::AimHealth;
        let lua = Lua::new();
        let request = || {
            AimRequest::Health(AimHealth {
                op: 3,
                critical_model: 1,
                damage_level: 14,
                initial_damage_level: 100,
                old_damage_level: 14,
                explosive: true,
                flammable: false,
                delta_time: 0.5,
                aircraft_class: 0,
                self_hit: false,
                target_not_ai: false,
                draw: Some(6553),
            })
        };
        let answering: LuaFunction = lua
            .load(
                "return function(r) assert(r.mode == 'health' and r.op == 'timed' and r.critical_model == 1 and r.damage_level == 14 and r.initial_damage_level == 100 and r.explosive == true \
                 and r.flammable == false and r.delta_time == 0.5 and r.draw == 6553 and r.self_hit == false)
                 return { state = 'done', critical_class = 1, crossed = false, recovered = false, eject = false, eligible = true, rolled = true, hit = true, new_level = 13, fatal = false } end",
            )
            .eval()?;
        let reply = call_aim(&lua, &answering, request())?;
        assert_eq!(
            (
                reply.health_critical_class,
                reply.health_eligible,
                reply.health_rolled,
                reply.health_hit,
                reply.health_new_level,
                reply.health_fatal
            ),
            (1, true, true, true, 13, false)
        );
        // a field missing is an error, never a default
        let missing: LuaFunction = lua
            .load("return function(r) return { state = 'done', critical_class = 1, crossed = false, recovered = false, eject = false, eligible = true, rolled = true, hit = true, fatal = false } end")
            .eval()?;
        assert!(call_aim(&lua, &missing, request()).err().ok_or("accepted")?.contains("new_level"));
        // a timed roll may ask for the sample; a classification may not
        let asking: LuaFunction = lua.load("return function(r) return { state = 'need_draw' } end").eval()?;
        assert!(call_aim(&lua, &asking, request())?.need_draw);
        let mut classify = request();
        if let AimRequest::Health(ref mut h) = classify {
            h.op = 1;
            h.draw = None;
        }
        assert!(
            call_aim(&lua, &asking, classify)?.need_draw,
            "the module carries the request; the campaign refuses a sample asked for a classification"
        );
        Ok(())
    }

    // The impact question reaches the host's Lua with its kill code and the target's facts by value, and the terrain only once the campaign has read it; the plan is read field by field
    // and the one lookup it may ask for is the terrain.
    #[test]
    fn an_impact_question_reaches_lua_by_value_and_its_plan_is_read_field_by_field() -> Result<(), Box<dyn std::error::Error>> {
        use eech_engine::AimImpact;
        let lua = Lua::new();
        let request = |terrain: Option<bool>| {
            AimRequest::Impact(AimImpact {
                kill_code: 3,
                has_target: true,
                target_alive: false,
                terrain_draws_crater: terrain,
                position: [250.0, 12.0, 750.0],
            })
        };
        let planning: LuaFunction = lua
            .load(
                "return function(r) assert(r.mode == 'impact' and r.kill_code == 3 and r.has_target == true and r.target_alive == false and r.position[1] == 250 and r.position[3] == 750)
                 if r.terrain_draws_crater == nil then return { state = 'need_lookup', need = 'terrain' } end
                 assert(r.terrain_draws_crater == true)
                 return { state = 'done', invalid = false, crater = true, direct_hit = false, blasts = 2, exclude_target = false } end",
            )
            .eval()?;
        let first = call_aim(&lua, &planning, request(None))?;
        assert!(first.need_lookup && first.impact_need_terrain);
        let plan = call_aim(&lua, &planning, request(Some(true)))?;
        assert_eq!(
            (
                plan.impact_invalid,
                plan.impact_crater,
                plan.impact_direct_hit,
                plan.impact_blasts,
                plan.impact_exclude_target
            ),
            (false, true, false, 2, false)
        );
        // a field missing is an error, never a default; and only the terrain may be asked for
        let missing: LuaFunction = lua
            .load("return function(r) return { state = 'done', invalid = false, crater = true, direct_hit = false, exclude_target = false } end")
            .eval()?;
        assert!(call_aim(&lua, &missing, request(Some(true))).err().ok_or("accepted")?.contains("blasts"));
        let other: LuaFunction = lua.load("return function(r) return { state = 'need_lookup', need = 'wind' } end").eval()?;
        assert!(call_aim(&lua, &other, request(None)).is_err());
        Ok(())
    }

    // The blast question reaches the host's Lua with the explosion, the map's sector figures and the window of the sector's list it was given (each object by value, named by its handle);
    // the host's walk comes back as a window it needs, or as the action on one object, and the echo of the candidates it examined is read when it is sent.
    #[test]
    fn a_blast_question_reaches_lua_by_value_and_the_walk_is_read_field_by_field() -> Result<(), Box<dyn std::error::Error>> {
        use eech_engine::{AimBlast, AimBlastEntry, AimBlastWindow};
        let lua = Lua::new();
        let request = |window: bool| {
            AimRequest::Blast(AimBlast {
                position: [150.0, 5.0, 150.0],
                damage_capability: 400,
                warhead_type: 7,
                has_target: true,
                sector_side_length: 100,
                min_x_sector: 0,
                max_x_sector: 2,
                min_z_sector: 0,
                max_z_sector: 2,
                window: window.then(|| AimBlastWindow {
                    sx: 1,
                    sz: 1,
                    after: 7,
                    end: true,
                    entries: vec![AimBlastEntry {
                        handle: 8,
                        entity_type: 37,
                        readable: true,
                        alive: true,
                        has_position: true,
                        rocket: true,
                        excluded: false,
                        position: [160.0, 5.0, 150.0],
                    }],
                }),
                echo: true,
            })
        };
        let walking: LuaFunction = lua
            .load(
                "return function(r) assert(r.mode == 'blast' and r.damage_capability == 400 and r.warhead_type == 7 and r.has_target == true and r.sector_side_length == 100 and r.max_x_sector == 2 \
                 and r.position[1] == 150 and r.echo == true)
                 if r.window == nil then return { state = 'need_lookup', radius = 20, need = { sx = 1, sz = 1, after = 0 } } end
                 local w = r.window
                 assert(w.sx == 1 and w.sz == 1 and w.after == 7 and w.list_ends == true and #w.objects == 1)
                 local o = w.objects[1]
                 assert(o.handle == 8 and o.entity_type == 37 and o.readable and o.alive and o.has_position and o.rocket and not o.excluded and o.position[1] == 160)
                 return { state = 'done', radius = 20, action = 'intercept', handle = 8, modifier = 0.25,
                          echo = { { kind = 'blast', a = 0, b = 3, c = 0, d = 0, f = { 5, 20, 0.25, 0, 0, 0, 0, 0, 0, 0, 0, 0 } } } } end",
            )
            .eval()?;
        let need = call_aim(&lua, &walking, request(false))?;
        assert!(need.need_lookup);
        assert_eq!((need.blast_radius, need.blast_need), (20.0, Some((1, 1, 0))));
        let reached = call_aim(&lua, &walking, request(true))?;
        assert_eq!(
            (reached.blast_radius, reached.blast_action, reached.blast_handle, reached.blast_modifier),
            (20.0, 2, 8, 0.25)
        );
        let echo = reached.echo.ok_or("no echo")?;
        let [event] = echo.as_slice() else {
            return Err("the echo is not one event".into());
        };
        assert_eq!((event.kind, event.a, event.b), (11, 0, 3));
        assert_eq!(event.f.get(..3), Some(&[5.0, 20.0, 0.25][..]));
        // the walk's end, with no object
        let over: LuaFunction = lua
            .load("return function(r) return { state = 'done', radius = 20, action = 'none' } end")
            .eval()?;
        let done = call_aim(&lua, &over, request(true))?;
        assert_eq!((done.blast_action, done.blast_handle, done.blast_modifier), (0, 0, 0.0));
        // a field missing, an action that is not one, a modifier that is not exact, and a lookup that is not a window are errors
        for (what, body) in [
            ("radius", "return { state = 'done', action = 'none' }"),
            ("action", "return { state = 'done', radius = 20 }"),
            ("is not none", "return { state = 'done', radius = 20, action = 'explode' }"),
            ("handle", "return { state = 'done', radius = 20, action = 'collision', modifier = 0.5 }"),
            ("modifier", "return { state = 'done', radius = 20, action = 'collision', handle = 8 }"),
            ("window", "return { state = 'need_lookup', radius = 20 }"),
            ("sx", "return { state = 'need_lookup', radius = 20, need = { sz = 1, after = 0 } }"),
        ] {
            let f: LuaFunction = lua.load(format!("return function(r) {body} end")).eval()?;
            assert!(call_aim(&lua, &f, request(true)).err().ok_or("accepted")?.contains(what), "{what}");
        }
        Ok(())
    }
    #[test]
    fn a_vehicle_lifecycle_answer_without_its_vector_is_an_error() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        for mode in [
            MovementMode::GvDeathCandidate,
            MovementMode::GvDeathDecay,
            MovementMode::GvFallCandidate,
            MovementMode::GvFallGravity,
            MovementMode::GvFallSettle,
            MovementMode::GvImpact,
            MovementMode::ShipSink,
            MovementMode::VehicleVectorSeed,
            MovementMode::GvFallVector,
            MovementMode::GvFallCommit,
        ] {
            let table = answer(&lua, "flags = 0")?;
            let error = read_move_result(&lua, mode, &table).err().ok_or("accepted")?;
            assert!(error.contains("motion_vector"), "{mode:?}: {error}");
        }
        Ok(())
    }

    #[test]
    fn an_explicit_zero_vector_and_zero_flags_are_accepted() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let table = answer(&lua, "motion_vector = {0, 0, 0}, flags = 0")?;
        let result = read_move_result(&lua, MovementMode::GvFallSettle, &table)?;
        assert_eq!(result.motion_vector, [0.0; 3]);
        assert_eq!(result.flags, 0);
        // a live step still answers without a vector
        read_move_result(&lua, MovementMode::Move, &answer(&lua, "")?)?;
        Ok(())
    }

    #[test]
    fn an_answer_that_decides_on_bits_must_carry_its_flags() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        for mode in [
            MovementMode::GvFallSettle,
            MovementMode::FwDeathAttitude,
            MovementMode::FwCrashVelocity,
            MovementMode::FwCrashAttitude,
        ] {
            let table = answer(&lua, "motion_vector = {0, 0, 0}")?;
            let error = read_move_result(&lua, mode, &table).err().ok_or("accepted")?;
            assert!(error.contains("flags"), "{mode:?}: {error}");
            let table = answer(&lua, "motion_vector = {0, 0, 0}, flags = 0")?;
            assert_eq!(read_move_result(&lua, mode, &table)?.flags, 0, "{mode:?}");
        }
        // a mode that carries no bits still reads an absent answer as 0
        let table = answer(&lua, "motion_vector = {0, 0, 0}")?;
        assert_eq!(read_move_result(&lua, MovementMode::GvImpact, &table)?.flags, 0);
        Ok(())
    }
}
