//! The Lua surface of the module.

use eech_engine::{
    AimEvent, AimReply, AimRequest, AimStepKind, AimTarget, AimWeapon, AimWorld, ArrivalAnswer, CandidateDecision, Engine, EngineConfig, EngineError,
    EnvProvider, EnvReply, EnvRequest, EnvState, Gunship, HelicopterGuidanceAnswer, HelicopterQuery, LosMessage, LosReply, LossDecision, LossDescriptor,
    MemberDescriptor, ModelAnimation, ModelAnswer, ModelObject, ModelObjectFacts, ModelOrder, ModelProvider, ModelQuery, ModelRouteFace, ModelRouteNode,
    ModelScene, ModelTriangle, MovementMode, MovementResult, MovementStep, ObjectReport, PoseClass, ScanDecision, SenseMessage, SenseReason, SenseReply,
    SenseScan, SenseState, Side, SupplyReachDescriptor, TerrainAnswer, TerrainMapFacts, TerrainProvider, TerrainQuery, TerrainRecord, TreeEnd, TreeStep,
    VehiclePose, WeaponFigures, EECH_ENV_AMBIENT, EECH_ENV_EXPORT, EECH_ENV_FLOATS, EECH_ENV_GUST, EECH_ENV_IMPORT, EECH_ENV_INIT_FIXED, EECH_ENV_INIT_RANDOM,
    EECH_ENV_INTS, EECH_ENV_LIGHTNING_ADD, EECH_ENV_LIGHTNING_UPDATE, EECH_ENV_MODE_COUNT, EECH_ENV_ROTOR_SOUND, EECH_ENV_SOUND_LIFETIME_INIT,
    EECH_ENV_SOUND_LIFETIME_UPDATE, EECH_ENV_SOUND_PAUSE_RESUME, EECH_ENV_STEP, EECH_ENV_WEATHER, EECH_ENV_WIND, EECH_LIGHTNING_SLOTS, EECH_ROTOR_ACTIONS,
    EECH_ROTOR_CHANNELS, EECH_ROTOR_PLAYER_AI, EECH_ROTOR_PLAYER_LOCAL, EECH_ROTOR_PLAYER_REMOTE, EECH_SOUND_SAMPLES,
};
use eech_engine::{SoundBackendProvider, SoundBackendReply, SoundBackendRequest, SOUND_BACKEND_PHASE_INITIALISE};
use eech_engine::{SoundCatalogueEntry, SoundCatalogueProvider, SoundCatalogueReply, SoundCatalogueRequest};
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

/// The host's terrain provider as the campaign calls it (EEDCS-47). The campaign hands the provider a request table
/// `{ op = "map" | "point" | "fast", x, z, hint }` and reads a table back. Everything about the shape of the answer is checked here, in Lua, where the
/// values are, and reaches Rust as plain text and integers: the numbers are formatted exactly (`%.17g`) and parsed with `nearest_f32`, as every other
/// number the engine takes from a host callback (the engine calls a callback under round-toward-zero, so a plain read could be one ulp off). A refusal is
/// returned as a stable marker and a message; the provider function's own error is returned with its original text.
///
/// Markers (the first line of the error the engine stops with carries `[terrain-provider:<marker>]`):
///   map.callback, point.callback, fast.callback   the provider raised an error (its text follows)
///   map.answer, point.answer, fast.answer         the answer is not a table
///   map.facts                                     a map fact is missing, not finite, or not an integer where one is required
///   point.elevation                               `elevation` or `last_elevation` is missing or not a finite number
///   point.normal                                  `normal` is not three finite numbers
///   point.type                                    `type` is not an integer in the EECH terrain-type range
///   point.hint                                    `hint` is neither nil nor a string of 1 to the limit of printable ASCII bytes
///   point.stored                                  `stored` is neither nil nor a boolean (nil means true: a provider that returns a surface for the point has stored one)
///   fast.elevation                                `elevation` is missing or not a finite number
///   sector.callback, sector.answer                the provider raised an error, or the answer is not a table
///   sector.flags                                  `flags` is not a string of exactly one `0` or `1` for each terrain type
///   ambient.callback, ambient.answer              the provider raised an error, or the answer is not a table
///   ambient.amplitudes                            `amplitudes` is not a list of exactly one finite number for each child asked about
///   sector.returned                               no flag is set (the sector's first surface is always flagged), or `returned` is not an integer of at least the number of flags set
const TERRAIN_BRIDGE: &str = r#"
local provider, types, hint_max = ...
local huge = math.huge
local function finite (v) return type (v) == 'number' and v == v and v ~= huge and v ~= -huge end
local function integer (v) return finite (v) and v == math.floor (v) end
local map_floats = { 'min_x', 'min_z', 'max_x', 'max_z', 'min_height', 'max_height', 'scaled_height_difference' }
return function (op, x, z, hint, winter, extra)
	-- THE CONTRACT IS CAPTURED BEFORE the provider is called (it is handed the request table and may change it): for an ambient question, the number of children asked about
	local children_asked = extra and #extra.children or 0
	local request = { op = op, x = x, z = z, hint = hint, winter = winter }
	if extra then
		request.alt, request.delta, request.in_cockpit, request.children = extra.alt, extra.delta, extra.in_cockpit, extra.children
	end
	local ok, answer = pcall (provider, request)
	if not ok then return op .. '.callback', tostring (answer) end
	if type (answer) ~= 'table' then return op .. '.answer', 'the ' .. op .. ' answer must be a table, got ' .. type (answer) end
	if op == 'point' then
		if not finite (answer.elevation) then return 'point.elevation', 'elevation must be a finite number' end
		if not finite (answer.last_elevation) then return 'point.elevation', 'last_elevation must be a finite number' end
		local n = answer.normal
		if type (n) ~= 'table' or #n ~= 3 or not (finite (n [1]) and finite (n [2]) and finite (n [3])) then
			return 'point.normal', 'normal must be three finite numbers'
		end
		local kind = answer.type
		if not integer (kind) or kind < 0 or kind >= types then
			return 'point.type', 'type must be an integer from 0 to ' .. (types - 1)
		end
		local h = answer.hint
		if h ~= nil and (type (h) ~= 'string' or #h < 1 or #h > hint_max or string.find (h, '[^\32-\126]') ~= nil) then
			return 'point.hint', 'hint must be nil or a string of 1 to ' .. hint_max .. ' printable ASCII bytes'
		end
		local stored = answer.stored
		if stored == nil then stored = true end
		if type (stored) ~= 'boolean' then return 'point.stored', 'stored must be a boolean or nil' end
		return nil, string.format ('%.17g %.17g %.17g %.17g %.17g', answer.elevation, answer.last_elevation, n [1], n [2], n [3]), kind, h, stored
	elseif op == 'fast' then
		if not finite (answer.elevation) then return 'fast.elevation', 'elevation must be a finite number' end
		return nil, string.format ('%.17g', answer.elevation)
	elseif op == 'ambient' then
		-- the new amplification of each terrain ambient sound child asked about (EEDCS-51), in the order asked: exactly that many finite numbers
		local a = answer.amplitudes
		if type (a) ~= 'table' or #a ~= children_asked then return 'ambient.amplitudes', 'amplitudes must be a list of exactly ' .. children_asked .. ' numbers, one for each child' end
		local out = {}
		for i = 1, children_asked do
			if not finite (a [i]) then return 'ambient.amplitudes', 'amplitude ' .. i .. ' must be a finite number' end
			out [#out + 1] = string.format ('%.17g', a [i])
		end
		return nil, table.concat (out, ' ')
	elseif op == 'sector' then
		-- the surface enumeration of a sector (EEDCS-51): the original's return value (1 + the surface transitions, NOT the number of distinct surfaces) and the whole scratch, one 0 or 1 per type
		local flags, returned = answer.flags, answer.returned
		if type (flags) ~= 'string' or #flags ~= types or string.find (flags, '[^01]') ~= nil then
			return 'sector.flags', 'flags must be a string of exactly ' .. types .. ' characters, each 0 or 1'
		end
		local _, distinct = string.gsub (flags, '1', '')
		if distinct < 1 then return 'sector.returned', 'the first surface of a sector is always flagged: at least one flag must be 1' end
		if not integer (returned) or returned < distinct or returned > 2147483647 then
			return 'sector.returned', 'returned must be an integer of at least the ' .. distinct .. ' surfaces flagged'
		end
		return nil, flags, returned
	end
	for _, field in ipairs { 'sector_x_max', 'sector_z_max' } do
		if not integer (answer [field]) or answer [field] < 1 then return 'map.facts', field .. ' must be an integer of at least 1' end
	end
	for _, field in ipairs { 'fast_width', 'fast_height' } do
		if not integer (answer [field]) or answer [field] < 0 then return 'map.facts', field .. ' must be an integer of at least 0' end
	end
	for _, field in ipairs (map_floats) do
		if not finite (answer [field]) then return 'map.facts', field .. ' must be a finite number' end
	end
	return nil, string.format ('%.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g', answer.sector_x_max, answer.sector_z_max, answer.fast_width,
		answer.fast_height, answer.min_x, answer.min_z, answer.max_x, answer.max_z, answer.min_height, answer.max_height, answer.scaled_height_difference)
end
"#;

/// The ground-contact operations of the same provider (EEDCS-48, D3 G2): `ground_above` (point_above_ground), `ground_below` (point_below_ground) and
/// `ground_impact` (get_ground_impact_point). The campaign hands the provider
///
///   { op = "ground_above" | "ground_below", world = { min_x, min_z, max_x, max_z }, x, y, z, hint?, winter }
///   { op = "ground_impact", world = { ... }, above = { x, y, z }, below = { x, y, z }, hint?, winter }
///
/// (`world` is the campaign's world-map area, geography it supplies; every number is an exact single) and reads back, by value:
///
///   ground_above:  { above = boolean, queried = boolean, record = nil | { last_elevation, normal = { nx, ny, nz }, type, stored, hint? } }
///   ground_below:  { below = boolean, queried = boolean, record = ... }
///   ground_impact: { above = { x, y, z }, below = { x, y, z }, impact = { x, y, z }, type, record = { ... } }
///
/// `record` is present exactly when `queried` is true: an answer that read no terrain leaves the campaign's record as it is. `stored` is a boolean or nil (nil means true, as in
/// a point answer: a host that keeps no per-record state reports a surface and no hint) and `type` of an impact must be the record's. Every number must be a Lua number (a string is refused, not coerced),
/// finite, and is read once; the shape is checked here, before anything reaches the engine. Markers: `<op>.callback` (the provider raised), `<op>.answer` (not a table, or a
/// field of the wrong type), `<op>.position` (a coordinate), `<op>.record` (the record), `<op>.type`.
const GROUND_BRIDGE: &str = r#"
local provider, types, hint_max = ...
local huge = math.huge
local function finite (v) return type (v) == 'number' and v == v and v ~= huge and v ~= -huge end
local function integer (v) return finite (v) and v == math.floor (v) end
local function point (op, name, t)
	if type (t) ~= 'table' then return nil, op .. '.position', name .. ' must be a table of x, y and z' end
	local x, y, z = t.x, t.y, t.z
	if not (finite (x) and finite (y) and finite (z)) then return nil, op .. '.position', name .. ' must be three finite numbers x, y and z' end
	return { x, y, z }
end
-- the record: its numbers as text, the type, the hint and the stored flag; nil and a marker when it is malformed
local function record (op, r)
	if type (r) ~= 'table' then return nil, op .. '.record', 'record must be a table' end
	local last, n, kind, h, stored = r.last_elevation, r.normal, r.type, r.hint, r.stored
	if not finite (last) then return nil, op .. '.record', 'record.last_elevation must be a finite number' end
	local n1, n2, n3
	if type (n) == 'table' and #n == 3 then n1, n2, n3 = n [1], n [2], n [3] end
	if not (finite (n1) and finite (n2) and finite (n3)) then return nil, op .. '.record', 'record.normal must be three finite numbers' end
	if not integer (kind) or kind < 0 or kind >= types then return nil, op .. '.type', 'record.type must be an integer from 0 to ' .. (types - 1) end
	if h ~= nil and (type (h) ~= 'string' or #h < 1 or #h > hint_max or string.find (h, '[^\32-\126]') ~= nil) then
		return nil, op .. '.record', 'record.hint must be nil or a string of 1 to ' .. hint_max .. ' printable ASCII bytes'
	end
	if stored == nil then stored = true end -- as a point answer: a record that holds a surface and does not say otherwise stored one
	if type (stored) ~= 'boolean' then return nil, op .. '.record', 'record.stored must be a boolean or nil' end
	return { text = string.format ('%.17g %.17g %.17g %.17g', last, n1, n2, n3), kind = kind, hint = h, stored = stored }
end
return function (op, winter, hint, min_x, min_z, max_x, max_z, x, y, z, bx, by, bz)
	local request = { op = op, winter = winter, hint = hint, world = { min_x = min_x, min_z = min_z, max_x = max_x, max_z = max_z } }
	if op == 'ground_impact' then
		request.above, request.below = { x = x, y = y, z = z }, { x = bx, y = by, z = bz }
	else
		request.x, request.y, request.z = x, y, z
	end
	local ok, answer = pcall (provider, request)
	if not ok then return op .. '.callback', tostring (answer) end
	if type (answer) ~= 'table' then return op .. '.answer', 'the ' .. op .. ' answer must be a table, got ' .. type (answer) end
	if op == 'ground_impact' then
		local a, b, c, kind, r = answer.above, answer.below, answer.impact, answer.type, answer.record
		local pa, ma, ta = point (op, 'above', a)
		if not pa then return ma, ta end
		local pb, mb, tb = point (op, 'below', b)
		if not pb then return mb, tb end
		local pc, mc, tc = point (op, 'impact', c)
		if not pc then return mc, tc end
		local rec, mr, tr = record (op, r)
		if not rec then return mr, tr end
		if kind ~= rec.kind then return op .. '.type', 'type must be the record\'s type' end
		return nil, string.format ('%.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g %.17g ', pa [1], pa [2], pa [3], pb [1], pb [2], pb [3], pc [1], pc [2], pc [3]) .. rec.text,
			rec.kind, rec.hint, rec.stored, true, true
	end
	local value, queried, r = answer [op == 'ground_above' and 'above' or 'below'], answer.queried, answer.record
	if type (value) ~= 'boolean' then return op .. '.answer', (op == 'ground_above' and 'above' or 'below') .. ' must be a boolean' end
	if type (queried) ~= 'boolean' then return op .. '.answer', 'queried must be a boolean' end
	if not queried then
		if r ~= nil then return op .. '.record', 'record must be absent when no terrain was read' end
		return nil, '', nil, nil, nil, value, false
	end
	local rec, mr, tr = record (op, r)
	if not rec then return mr, tr end
	return nil, rec.text, rec.kind, rec.hint, rec.stored, value, true
end
"#;

/// The tree segment operations of the same provider (EEDCS-49, D3): `tree_begin` (get_line_of_sight_collision_tree) and `tree_terrain`, the campaign's answer to the elevation the host asked for.
/// The campaign hands the provider
///
///   { op = "tree_begin", winter, old = "0x..,0x..,0x..", new = "...", point = "...", normal = "...", ignore_forest_edge_trees = boolean, skip_tree_top_test = boolean }
///   { op = "tree_terrain", winter, elevation = "0x.." }
///
/// Every float is its 32-bit PATTERN as `0x` and eight hexadecimal digits (three comma separated for a position): the caller's own point and normal buffers may hold anything (a signed
/// zero, a NaN) and a miss gives them back as the original leaves them, so no number is converted anywhere on the way, and Lua never sees a float it could round or normalise. It reads back
///
///   { state = "need_terrain", end = "old" | "new", x = "0x..", z = "0x.." }   the host needs the campaign's terrain elevation at (x, z) for that end of the segment
///   { state = "done", hit = boolean, point = "0x..,0x..,0x..", normal = "0x..,0x..,0x.." }   the final intercept point and face normal, as the original leaves them
///
/// The shape is checked here, before anything reaches the engine. Markers: `<op>.callback` (the provider raised) and `<op>.answer` (not a table, or a field of the wrong shape).
const TREE_BRIDGE: &str = r#"
local provider = ...
local function word (v) return type (v) == 'string' and string.find (v, '^0x%x%x%x%x%x%x%x%x$') ~= nil end
local function triple (v) return type (v) == 'string' and string.find (v, '^0x%x%x%x%x%x%x%x%x,0x%x%x%x%x%x%x%x%x,0x%x%x%x%x%x%x%x%x$') ~= nil end
return function (op, winter, old, new, point, normal, ignore, skip, elevation)
	local request = { op = op, winter = winter }
	if op == 'tree_begin' then
		request.old, request.new, request.point, request.normal = old, new, point, normal
		request.ignore_forest_edge_trees, request.skip_tree_top_test = ignore, skip
	else
		request.elevation = elevation
	end
	local ok, answer = pcall (provider, request)
	if not ok then return op .. '.callback', tostring (answer) end
	if type (answer) ~= 'table' then return op .. '.answer', 'the ' .. op .. ' answer must be a table, got ' .. type (answer) end
	local state = answer.state
	if state == 'need_terrain' then
		local e = answer['end']
		if e ~= 'old' and e ~= 'new' then return op .. '.answer', 'end must be old or new' end
		if not word (answer.x) or not word (answer.z) then return op .. '.answer', 'x and z must each be 0x and eight hexadecimal digits' end
		return nil, 'need_terrain ' .. e .. ' ' .. answer.x .. ' ' .. answer.z
	elseif state == 'done' then
		if type (answer.hit) ~= 'boolean' then return op .. '.answer', 'hit must be a boolean' end
		if not triple (answer.point) or not triple (answer.normal) then return op .. '.answer', 'point and normal must each be three words, 0x and eight hexadecimal digits, comma separated' end
		return nil, 'done ' .. (answer.hit and '1' or '0') .. ' ' .. answer.point .. ' ' .. answer.normal
	end
	return op .. '.answer', 'state must be need_terrain or done, got ' .. tostring (state)
end
"#;

/// The number of EECH terrain types the provider's `type` is checked against (`TERRAIN_TYPE_LAST`), and the longest hint, mirrored from the C side.
const TERRAIN_TYPE_COUNT: i64 = 62;
const TERRAIN_HINT_MAX: i64 = 127;

/// Builds the provider the engine keeps from the host's Lua function.
fn terrain_provider(lua: &Lua, function: LuaFunction) -> LuaResult<TerrainProvider> {
    let build: LuaFunction = lua.load(TERRAIN_BRIDGE).set_name("eech terrain bridge").into_function()?;
    let bridge: LuaFunction = build.call((function.clone(), TERRAIN_TYPE_COUNT, TERRAIN_HINT_MAX))?;
    let ground_build: LuaFunction = lua.load(GROUND_BRIDGE).set_name("eech ground bridge").into_function()?;
    let ground: LuaFunction = ground_build.call((function.clone(), TERRAIN_TYPE_COUNT, TERRAIN_HINT_MAX))?;
    let tree_build: LuaFunction = lua.load(TREE_BRIDGE).set_name("eech tree bridge").into_function()?;
    let tree: LuaFunction = tree_build.call(function)?;
    let lua = lua.clone();
    Ok(Box::new(move |query| match query {
        TerrainQuery::GroundAbove { .. } | TerrainQuery::GroundBelow { .. } | TerrainQuery::GroundImpact { .. } => call_ground(&ground, query),
        TerrainQuery::TreeBegin { .. } | TerrainQuery::TreeTerrain { .. } => call_tree(&tree, query),
        query => call_terrain(&lua, &bridge, query),
    }))
}

/// A float pattern as the provider receives it: `0x` and eight hexadecimal digits.
fn tree_word_text(word: u32) -> String {
    format!("0x{word:08x}")
}

fn tree_words_text(words: &[u32; 3]) -> String {
    format!("{},{},{}", tree_word_text(words[0]), tree_word_text(words[1]), tree_word_text(words[2]))
}

fn tree_word(token: &str, marker: &str) -> Result<u32, String> {
    token
        .strip_prefix("0x")
        .filter(|digits| digits.len() == 8)
        .and_then(|digits| u32::from_str_radix(digits, 16).ok())
        .ok_or_else(|| format!("[terrain-provider:{marker}] the tree bridge returned a word that is not 0x and eight hexadecimal digits"))
}

fn tree_words(text: &str, marker: &str) -> Result<[u32; 3], String> {
    let parts: Vec<&str> = text.split(',').collect();
    match parts[..] {
        [x, y, z] => Ok([tree_word(x, marker)?, tree_word(y, marker)?, tree_word(z, marker)?]),
        _ => Err(format!(
            "[terrain-provider:{marker}] the tree bridge returned a position that is not three words"
        )),
    }
}

/// The tree questions (EEDCS-49, D3) through the tree bridge: patterns in, patterns out.
fn call_tree(bridge: &LuaFunction, query: TerrainQuery) -> Result<TerrainAnswer, String> {
    let (op, winter, old, new, point, normal, ignore, skip, elevation) = match &query {
        TerrainQuery::TreeBegin {
            old,
            new,
            point,
            normal,
            ignore_forest_edge_trees,
            skip_tree_top_test,
            winter,
        } => (
            "tree_begin",
            *winter,
            Some(tree_words_text(old)),
            Some(tree_words_text(new)),
            Some(tree_words_text(point)),
            Some(tree_words_text(normal)),
            Some(*ignore_forest_edge_trees),
            Some(*skip_tree_top_test),
            None,
        ),
        TerrainQuery::TreeTerrain { elevation, winter } => ("tree_terrain", *winter, None, None, None, None, None, None, Some(tree_word_text(*elevation))),
        _ => return Err("[terrain-provider:tree.callback] a non-tree question reached the tree bridge".to_string()),
    };
    let (marker, text): (Option<String>, Option<String>) = bridge
        .call((op, winter, old, new, point, normal, ignore, skip, elevation))
        .map_err(|error| format!("[terrain-provider:{op}.callback] {error}"))?;
    if let Some(marker) = marker {
        return Err(format!("[terrain-provider:{marker}] {}", text.unwrap_or_default()));
    }
    let text = text.ok_or_else(|| format!("[terrain-provider:{op}.answer] the tree bridge returned nothing"))?;
    let marker = format!("{op}.answer");
    let parts: Vec<&str> = text.split(' ').collect();
    match parts[..] {
        ["need_terrain", end, x, z] => Ok(TerrainAnswer::Tree(TreeStep::NeedTerrain {
            end: match end {
                "old" => TreeEnd::Old,
                "new" => TreeEnd::New,
                _ => return Err(format!("[terrain-provider:{marker}] the tree bridge returned an unknown end")),
            },
            x: tree_word(x, &marker)?,
            z: tree_word(z, &marker)?,
        })),
        ["done", hit, point, normal] => Ok(TerrainAnswer::Tree(TreeStep::Done {
            hit: match hit {
                "0" => false,
                "1" => true,
                _ => return Err(format!("[terrain-provider:{marker}] the tree bridge returned a hit that is neither 0 nor 1")),
            },
            point: tree_words(point, &marker)?,
            normal: tree_words(normal, &marker)?,
        })),
        _ => Err(format!("[terrain-provider:{marker}] the tree bridge returned an answer of an unknown shape")),
    }
}

/// The `N` numbers of an answer's text as the nearest `f32`s; a malformed text is a refusal under `marker`.
fn terrain_numbers<const N: usize>(text: &str, marker: &str) -> Result<[f64; N], String> {
    let mut numbers = [0.0f64; N];
    let mut parts = text.split_whitespace();
    for slot in &mut numbers {
        *slot = parts
            .next()
            .and_then(|part| part.parse::<f64>().ok())
            .filter(|value| value.is_finite())
            .ok_or_else(|| format!("[terrain-provider:{marker}] the answer's numbers are malformed"))?;
    }
    if parts.next().is_some() {
        return Err(format!("[terrain-provider:{marker}] the answer has more numbers than expected"));
    }
    Ok(numbers)
}

fn terrain_f32(value: f64, marker: &str) -> Result<f32, String> {
    nearest_f32(value).ok_or_else(|| format!("[terrain-provider:{marker}] a number is beyond the single-precision range"))
}

fn terrain_i32(value: f64, marker: &str) -> Result<i32, String> {
    if value.fract() == 0.0 && value.abs() <= f64::from(i32::MAX) {
        Ok(value as i32)
    } else {
        Err(format!("[terrain-provider:{marker}] an integer fact is out of range"))
    }
}

/// What the Lua terrain bridge returns, in order: the refusal marker, the text of the answer's numbers, the surface type, the record's hint and whether a
/// surface is stored (each absent when the answer does not carry it).
type BridgeAnswer = (Option<String>, Option<String>, Option<i64>, Option<String>, Option<bool>);

/// The extra fields of an ambient question as a table for the bridge (`alt`, `delta`, `in_cockpit` and the `children` list of `{ class, previous }`), or nothing for any other question.
fn terrain_extra(lua: &Lua, query: &TerrainQuery) -> Result<Option<LuaTable>, String> {
    let e = |error: LuaError| error.to_string();
    let TerrainQuery::Ambient {
        in_cockpit,
        alt,
        delta,
        children,
        ..
    } = query
    else {
        return Ok(None);
    };
    let extra = lua.create_table().map_err(e)?;
    extra.set("alt", f64::from(*alt)).map_err(e)?;
    extra.set("delta", f64::from(*delta)).map_err(e)?;
    extra.set("in_cockpit", *in_cockpit).map_err(e)?;
    let list = lua.create_table().map_err(e)?;
    for (index, child) in children.iter().enumerate() {
        let entry = lua.create_table().map_err(e)?;
        entry.set("class", child.class).map_err(e)?;
        entry.set("previous", f64::from(child.previous)).map_err(e)?;
        list.set(index + 1, entry).map_err(e)?;
    }
    extra.set("children", list).map_err(e)?;
    Ok(Some(extra))
}

fn call_terrain(lua: &Lua, bridge: &LuaFunction, query: TerrainQuery) -> Result<TerrainAnswer, String> {
    let extra = terrain_extra(lua, &query).map_err(|error| format!("[terrain-provider:ambient.callback] {error}"))?;
    let (op, x, z, hint, winter) = match &query {
        TerrainQuery::Map { winter } => ("map", None, None, None, *winter),
        TerrainQuery::Point { x, z, hint, winter } => ("point", Some(f64::from(*x)), Some(f64::from(*z)), hint.clone(), *winter),
        TerrainQuery::Fast { x, z, winter } => ("fast", Some(f64::from(*x)), Some(f64::from(*z)), None, *winter),
        TerrainQuery::Sector { x, z, winter } => ("sector", Some(f64::from(*x)), Some(f64::from(*z)), None, *winter),
        TerrainQuery::Ambient { x, z, winter, .. } => ("ambient", Some(f64::from(*x)), Some(f64::from(*z)), None, *winter),
        // the ground questions go to the ground bridge (`terrain_provider`), never here
        TerrainQuery::GroundAbove { .. } | TerrainQuery::GroundBelow { .. } | TerrainQuery::GroundImpact { .. } => {
            return Err("[terrain-provider:point.callback] a ground question reached the point bridge".to_string())
        }
        // and the tree questions to the tree bridge
        TerrainQuery::TreeBegin { .. } | TerrainQuery::TreeTerrain { .. } => {
            return Err("[terrain-provider:point.callback] a tree question reached the point bridge".to_string())
        }
    };
    let (marker, text, kind, hint_out, stored): BridgeAnswer = bridge
        .call((op, x, z, hint, winter, extra))
        .map_err(|error| format!("[terrain-provider:{op}.callback] {error}"))?;
    if let Some(marker) = marker {
        return Err(format!("[terrain-provider:{marker}] {}", text.unwrap_or_default()));
    }
    let text = text.ok_or_else(|| format!("[terrain-provider:{op}.answer] the terrain bridge returned nothing"))?;
    match op {
        "map" => {
            let n = terrain_numbers::<11>(&text, "map.facts")?;
            Ok(TerrainAnswer::Map(TerrainMapFacts {
                sector_x_max: terrain_i32(n[0], "map.facts")?,
                sector_z_max: terrain_i32(n[1], "map.facts")?,
                fast_width: terrain_i32(n[2], "map.facts")?,
                fast_height: terrain_i32(n[3], "map.facts")?,
                min_x: terrain_f32(n[4], "map.facts")?,
                min_z: terrain_f32(n[5], "map.facts")?,
                max_x: terrain_f32(n[6], "map.facts")?,
                max_z: terrain_f32(n[7], "map.facts")?,
                min_height: terrain_f32(n[8], "map.facts")?,
                max_height: terrain_f32(n[9], "map.facts")?,
                scaled_height_difference: terrain_f32(n[10], "map.facts")?,
            }))
        }
        "point" => {
            let n = terrain_numbers::<5>(&text, "point.elevation")?;
            let kind = kind.ok_or_else(|| "[terrain-provider:point.type] the terrain bridge returned no type".to_string())?;
            let kind = i32::try_from(kind).map_err(|_| "[terrain-provider:point.type] the type is out of range".to_string())?;
            Ok(TerrainAnswer::Point {
                elevation: terrain_f32(n[0], "point.elevation")?,
                last_elevation: terrain_f32(n[1], "point.elevation")?,
                normal: [
                    terrain_f32(n[2], "point.normal")?,
                    terrain_f32(n[3], "point.normal")?,
                    terrain_f32(n[4], "point.normal")?,
                ],
                kind,
                hint: hint_out,
                stored: stored.ok_or_else(|| "[terrain-provider:point.stored] the terrain bridge returned no stored flag".to_string())?,
            })
        }
        "ambient" => {
            // the bridge returns the amplitudes as the text, exactly as many as the children asked about (the bridge checked the count; the engine boundary checks it again)
            let amplitudes = text
                .split_whitespace()
                .map(|part| {
                    part.parse::<f64>()
                        .ok()
                        .filter(|value| value.is_finite())
                        .ok_or_else(|| "[terrain-provider:ambient.amplitudes] an amplitude is malformed".to_string())
                        .and_then(|value| terrain_f32(value, "ambient.amplitudes"))
                })
                .collect::<Result<Vec<f32>, String>>()?;
            Ok(TerrainAnswer::Ambient { amplitudes })
        }
        "sector" => {
            // the bridge returns the flags as the text and the return value in the type slot
            let returned = kind.ok_or_else(|| "[terrain-provider:sector.returned] the terrain bridge returned no return value".to_string())?;
            let returned = i32::try_from(returned).map_err(|_| "[terrain-provider:sector.returned] the return value is out of range".to_string())?;
            Ok(TerrainAnswer::Sector {
                returned,
                flags: text.chars().map(|flag| flag == '1').collect(),
            })
        }
        _ => {
            let n = terrain_numbers::<1>(&text, "fast.elevation")?;
            Ok(TerrainAnswer::Fast {
                elevation: terrain_f32(n[0], "fast.elevation")?,
            })
        }
    }
}

/// The host's environment provider as the campaign calls it (EEDCS-51, D3): the physical weather and wind field and its update. The campaign hands the provider a request table
/// `{ op = "import" | "export" | "init_random" | "init_fixed" | "weather" | "wind" | "gust" | "ambient" | "step", winter, ... }` and reads a table back (the fields of each operation are in
/// aphavoc/.../ss_hostenv.h and docs/d3-environment-provider.md). Everything about the shape of the answer is checked here, in Lua, where the values are, and reaches Rust as plain
/// text: the numbers are formatted exactly (`%.17g`) and parsed with `nearest_f32`, as every number the engine takes from a host callback (the engine calls a callback under
/// round-toward-zero). The request's numbers are handed to the provider as Lua numbers, unchanged: a negative zero, a subnormal and a draw reach it bit for bit.
///
/// Markers (the first line of the error the engine stops with carries `[env-provider:<marker>]`):
///   <op>.callback   the provider raised an error (its text follows)
///   <op>.answer     the answer is not a table, or a number of it is missing, not finite, or out of its domain (a mode, a flag)
///   ambient.answer  one of rain_light, rain_heavy, wind_light, wind_heavy is not a finite number, or a wind level is outside 0 to 1
///   <op>.state      the field in the answer is missing a number, not finite, a mode outside 0 to 4, or a flag that is not a boolean
const ENV_BRIDGE: &str = r#"
local provider, floats, ints = ...
local huge = math.huge
local function finite (v) return type (v) == 'number' and v == v and v ~= huge and v ~= -huge end
local function integer (v) return finite (v) and v == math.floor (v) end
local function fmt (v) return string.format ('%.17g', v) end
local function state_text (state, op)
	if type (state) ~= 'table' then return nil, op .. '.state', 'the field must be a table, got ' .. type (state) end
	local out = {}
	for _, name in ipairs (floats) do
		if not finite (state [name]) then return nil, op .. '.state', name .. ' must be a finite number' end
		out [#out + 1] = fmt (state [name])
	end
	for i, name in ipairs (ints) do
		local v = state [name]
		if i <= 2 then
			if not integer (v) or v < 0 or v > 4 then return nil, op .. '.state', name .. ' must be an integer from 0 to 4' end
		else
			if type (v) ~= 'boolean' then return nil, op .. '.state', name .. ' must be a boolean' end
			v = v and 1 or 0
		end
		out [#out + 1] = string.format ('%d', v)
	end
	return table.concat (out, ' ')
end
local function mode (v) return integer (v) and v >= 0 and v <= 4 end
local function vector3 (v) return type (v) == 'table' and finite (v [1]) and finite (v [2]) and finite (v [3]) end
return function (request)
	-- THE CONTRACT IS CAPTURED HERE, BEFORE the provider is called: the provider is handed the request table and may change it, so every decision below that depends on what the campaign
	-- asked (the operation, whether a wind vector was wanted, whether this is a server, whether the lightning draw was already made) is made from these values, never from the table
	local op = request.op
	local want_vector = request.want_vector == true
	local server = request.server == true
	local has_draw = request.lightning_draw ~= nil
	local has_frames = request.frames ~= nil
	local request_valid = request.valid == true
	local request_has_handle = request.has_handle == true
	local request_have_playing = request.playing ~= nil
	local ok, answer = pcall (provider, request)
	if not ok then return op .. '.callback', tostring (answer) end
	if type (answer) ~= 'table' then return op .. '.answer', 'the ' .. op .. ' answer must be a table, got ' .. type (answer) end
	if op == 'import' then
		return nil, 'ok'
	elseif op == 'export' or op == 'init_random' or op == 'init_fixed' then
		local text, marker, message = state_text (answer.state, op)
		if not text then return marker, message end
		return nil, text
	elseif op == 'weather' then
		if not (mode (answer.current_mode) and mode (answer.target_mode) and mode (answer.simple_mode)) then return 'weather.answer', 'the modes must be integers from 0 to 4' end
		if not finite (answer.transitional_state) then return 'weather.answer', 'transitional_state must be a finite number' end
		return nil, string.format ('%d %d %s %d', answer.current_mode, answer.target_mode, fmt (answer.transitional_state), answer.simple_mode)
	elseif op == 'wind' then
		if not finite (answer.speed) then return 'wind.answer', 'speed must be a finite number' end
		local v = answer.vector
		if want_vector then
			if not vector3 (v) then return 'wind.answer', 'vector must be three finite numbers' end
		else
			v = { 0, 0, 0 }
		end
		return nil, string.format ('%s %s %s %s', fmt (answer.speed), fmt (v [1]), fmt (v [2]), fmt (v [3]))
	elseif op == 'gust' then
		if not finite (answer.modulation) then return 'gust.answer', 'modulation must be a finite number' end
		return nil, fmt (answer.modulation)
	elseif op == 'ambient' then
		-- the four ambient sound levels (EEDCS-51): rain light, rain heavy, wind light, wind heavy; the wind levels are the original's piecewise function of r in [0, 1]
		for _, name in ipairs { 'rain_light', 'rain_heavy', 'wind_light', 'wind_heavy' } do
			if not finite (answer [name]) then return 'ambient.answer', name .. ' must be a finite number' end
		end
		for _, name in ipairs { 'wind_light', 'wind_heavy' } do
			if answer [name] < 0 or answer [name] > 1 then return 'ambient.answer', name .. ' must be within 0 to 1' end
		end
		return nil, string.format ('%s %s %s %s', fmt (answer.rain_light), fmt (answer.rain_heavy), fmt (answer.wind_light), fmt (answer.wind_heavy))
	elseif op == 'sound_lifetime_init' then
		-- the physical lifetime of a new sound effect (EEDCS-51): one finite number from 0 up
		if not finite (answer.lifetime) or answer.lifetime < 0 then return op .. '.answer', 'lifetime must be a finite number from 0 up' end
		return nil, fmt (answer.lifetime)
	elseif op == 'sound_lifetime_update' then
		-- one call of a sound effect's update routine: a request to know whether the handle is playing (once, only for the server's valid effect that holds one), or the lifetimes after the call, whether a valid
		-- effect's lifetime went below zero and the decision
		if answer.need_playing == true then
			if not (server and request_valid and request_has_handle) then return op .. '.answer', 'the provider asked whether a handle is playing where none may be asked about' end
			if request_have_playing then return op .. '.answer', 'the provider asked whether the handle is playing after it was told' end
			return nil, '1'
		end
		if answer.need_playing ~= nil and answer.need_playing ~= false then return op .. '.answer', 'need_playing must be a boolean or nil' end
		if not finite (answer.lifetime) then return op .. '.answer', 'lifetime must be a finite number' end
		if not finite (answer.valid_lifetime) then return op .. '.answer', 'valid_lifetime must be a finite number' end
		if type (answer.expired) ~= 'boolean' then return op .. '.answer', 'expired must be a boolean, got ' .. type (answer.expired) end
		local action = ({ none = 0, kill = 1, unlink = 2 }) [answer.action]
		if action == nil then return op .. '.answer', "action must be 'none', 'kill' or 'unlink'" end
		return nil, string.format ('0 %s %s %d %d', fmt (answer.lifetime), fmt (answer.valid_lifetime), answer.expired and 1 or 0, action)
	elseif op == 'sound_pause_resume' then
		-- one effect under a pause or a resume (EEDCS-51): the host's answer is explicit in every field (a missing flag is not false) and the valid lifetime is the exact stored or copied number
		for _, name in ipairs { 'changed', 'valid', 'play', 'relink' } do
			if type (answer [name]) ~= 'boolean' then return op .. '.answer', name .. ' must be a boolean, got ' .. type (answer [name]) end
		end
		if not finite (answer.valid_lifetime) then return op .. '.answer', 'valid_lifetime must be a finite number' end
		return nil, string.format ('0 %d %d %s %d %d', answer.changed and 1 or 0, answer.valid and 1 or 0, fmt (answer.valid_lifetime), answer.play and 1 or 0, answer.relink and 1 or 0)
	elseif op == 'rotor_sound' then
		-- the acoustics of ONE helicopter (EEDCS-51, D3): every logical channel by name with its levels and the ordered requests; an unknown name, an extra field, a missing or non-finite level, a pitch on the
		-- winding channel and a delay on a resume are refused, and a missing number is never read as zero
		local names = { 'looping', 'slap', 'turbine1', 'turbine2', 'apu', 'winding' }
		local known = {}
		for i, name in ipairs (names) do known [name] = i end
		local channels = answer.channels
		if type (channels) ~= 'table' then return op .. '.answer', 'channels must be a table, got ' .. type (channels) end
		for name in pairs (channels) do
			if known [name] == nil then return op .. '.answer', 'channels has the unknown channel ' .. tostring (name) end
		end
		local out = { '0' }
		for _, name in ipairs (names) do
			local c = channels [name]
			if type (c) ~= 'table' then return op .. '.answer', 'channels.' .. name .. ' must be a table, got ' .. type (c) end
			for key in pairs (c) do
				if key ~= 'amplification' and key ~= 'pitch' then return op .. '.answer', 'channels.' .. name .. ' has the unknown field ' .. tostring (key) end
			end
			if not finite (c.amplification) then return op .. '.answer', 'channels.' .. name .. '.amplification must be a finite number' end
			local mask, pitch = 3, c.pitch
			if name == 'winding' then
				if pitch ~= nil then return op .. '.answer', 'the winding channel writes the amplification only and must carry no pitch' end
				mask, pitch = 1, 0
			elseif not finite (pitch) then
				return op .. '.answer', 'channels.' .. name .. '.pitch must be a finite number'
			end
			out [#out + 1] = string.format ('%d %s %s', mask, fmt (c.amplification), fmt (pitch))
		end
		local actions = answer.actions
		if type (actions) ~= 'table' or actions [9] ~= nil then return op .. '.answer', 'actions must be a list of at most 8 requests' end
		local listed = 0
		for _ in pairs (actions) do listed = listed + 1 end
		if listed ~= #actions then return op .. '.answer', 'actions must be a plain list' end
		out [#out + 1] = string.format ('%d', listed)
		local kinds = { pause = 0, resume = 1 }
		for i, request in ipairs (actions) do
			if type (request) ~= 'table' then return op .. '.answer', 'actions [' .. i .. '] must be a table, got ' .. type (request) end
			for key in pairs (request) do
				if key ~= 'action' and key ~= 'channel' and key ~= 'delay' then return op .. '.answer', 'actions [' .. i .. '] has the unknown field ' .. tostring (key) end
			end
			local kind = kinds [request.action]
			if kind == nil then return op .. '.answer', 'actions [' .. i .. "].action must be 'pause' or 'resume'" end
			local channel = known [request.channel]
			if channel == nil or channel > 5 then return op .. '.answer', 'actions [' .. i .. '].channel must be looping, slap, turbine1, turbine2 or apu' end
			local delay = request.delay
			if kind == 1 then
				if delay ~= nil then return op .. '.answer', 'actions [' .. i .. '] resumes with a delay, which a resume does not carry' end
				delay = 0
			elseif not finite (delay) then
				return op .. '.answer', 'actions [' .. i .. '].delay must be a finite number'
			end
			out [#out + 1] = string.format ('%d %d %s', kind, channel - 1, fmt (delay))
		end
		return nil, table.concat (out, ' ')
	elseif op == 'lightning_add' or op == 'lightning_update' then
		-- the physical lightning queue (EEDCS-51): an update the host cannot finish without the strike animation's frame count says so and changes nothing, once
		if answer.need_frames == true then
			if op ~= 'lightning_update' then return op .. '.answer', 'an add never needs the frame count' end
			if has_frames then return op .. '.answer', 'the provider asked for the frame count it was given' end
			return nil, '1'
		end
		if answer.need_frames ~= nil and answer.need_frames ~= false then return op .. '.answer', 'need_frames must be a boolean or nil' end
		local slot = -1
		if op == 'lightning_add' then
			slot = answer.slot
			if not integer (slot) or slot < -1 or slot > 31 then return op .. '.answer', 'slot must be an integer from -1 (none) to 31' end
		end
		local queue = answer.queue
		if type (queue) ~= 'table' or queue [33] ~= nil then return op .. '.answer', 'queue must be a list of exactly 32 strikes' end
		local out = { '0', string.format ('%d', slot) }
		for i = 1, 32 do
			local s = queue [i]
			if type (s) ~= 'table' then return op .. '.answer', 'queue [' .. i .. '] must be a table' end
			if type (s.in_use) ~= 'boolean' then return op .. '.answer', 'queue [' .. i .. '].in_use must be a boolean' end
			if not integer (s.type) or s.type < 0 or s.type > 5 then return op .. '.answer', 'queue [' .. i .. '].type must be an integer from 0 to 5' end
			if not integer (s.frame) then return op .. '.answer', 'queue [' .. i .. '].frame must be an integer' end
			out [#out + 1] = s.in_use and '1' or '0'
			out [#out + 1] = string.format ('%d', s.type)
			for _, name in ipairs { 'x', 'y', 'z', 'time_started', 'time_length' } do
				if not finite (s [name]) then return op .. '.answer', 'queue [' .. i .. '].' .. name .. ' must be a finite number' end
				out [#out + 1] = fmt (s [name])
			end
			out [#out + 1] = string.format ('%d', s.frame)
		end
		return nil, table.concat (out, ' ')
	end
	-- step
	if answer.need_draw == true then
		-- only a SERVER's FIRST question, before the draw, may be answered with a request for the draw: a client never draws, and a draw already made is not asked for again
		if not server then return 'step.answer', 'the provider asked for the lightning draw of a client, which never draws' end
		if has_draw then return 'step.answer', 'the provider asked for the lightning draw it was given' end
		if not finite (answer.timer) then return 'step.answer', 'timer must be a finite number' end
		return nil, '1 ' .. fmt (answer.timer)
	end
	if answer.need_draw ~= nil and answer.need_draw ~= false then return 'step.answer', 'need_draw must be a boolean or nil' end
	-- a completed step says whether the timer expired, explicitly (a missing field is not 'false')
	local expired = answer.expired
	if type (expired) ~= 'boolean' then return 'step.answer', 'expired must be a boolean, got ' .. type (expired) end
	-- the draw was made because the first answer reported an expiry: the final answer must complete and still report it
	if has_draw and not expired then return 'step.answer', 'the lightning draw was made for an expiry the final answer does not report' end
	local strike, new_timer = { 0, 0, 0 }, 0
	if expired then
		-- a server's expiry is negotiated: it is reported only with the draw its new timer needs (a client's new timer is set by the server, and is not read)
		if server and not has_draw then return 'step.answer', 'the lightning timer expired without the draw the new timer needs' end
		strike = answer.strike
		if not vector3 (strike) then return 'step.answer', 'strike must be three finite numbers' end
		if server then
			new_timer = answer.new_timer
			if not finite (new_timer) then return 'step.answer', 'new_timer must be a finite number' end
		end
	end
	local text, marker, message = state_text (answer.state, 'step')
	if not text then return marker, message end
	return nil, string.format ('0 %d %s %s %s %s %s', expired and 1 or 0, fmt (strike [1]), fmt (strike [2]), fmt (strike [3]), fmt (new_timer), text)
end
"#;

/// The names of the field's floats and integers in the tables of the provider's protocol, in the order of the arrays of the engine's state (the enums of ss_hostenv.h). The last
/// three integers are booleans in the tables.
const ENV_FLOAT_NAMES: [&str; EECH_ENV_FLOATS] = [
    "weather_radius",
    "weather_mode_transitional_period",
    "weather_mode_transitional_status",
    "weather_position_x",
    "weather_position_y",
    "weather_position_z",
    "weather_velocity_x",
    "weather_velocity_y",
    "weather_velocity_z",
    "wind_effect_radius",
    "wind_gusting_value",
    "wind_minimum_speed",
    "wind_maximum_speed",
    "wind_direction_x",
    "wind_direction_y",
    "wind_direction_z",
    "wind_effect_position_x",
    "wind_effect_position_y",
    "wind_effect_position_z",
    "wind_effect_velocity_x",
    "wind_effect_velocity_y",
    "wind_effect_velocity_z",
    "lightning_timer",
];

const ENV_INT_NAMES: [&str; EECH_ENV_INTS] = [
    "weather_mode",
    "target_weather_mode",
    "local_weather_model",
    "weather_increasing",
    "wind_increasing",
];

/// Builds the provider the engine keeps from the host's Lua function.
fn environment_provider(lua: &Lua, function: LuaFunction) -> LuaResult<EnvProvider> {
    let build: LuaFunction = lua.load(ENV_BRIDGE).set_name("eech environment bridge").into_function()?;
    let bridge: LuaFunction = build.call((function, ENV_FLOAT_NAMES.to_vec(), ENV_INT_NAMES.to_vec()))?;
    let lua = lua.clone();
    Ok(Box::new(move |request| call_environment(&lua, &bridge, request)))
}

/// The function that answers the sample metadata question: the boot's `sound_data` when it gave one, else the host's environment provider (the reference host's answers it). There is no campaign reader to
/// fall back to, so a `sound_data` that is not a function is refused, with the stable marker; a provider that cannot answer is refused with the same marker at the first question, which the engine
/// asks before it applies anything.
fn sound_data_function(config: &LuaTable, environment: &LuaFunction) -> LuaResult<LuaFunction> {
    match config.get::<LuaValue>("sound_data")? {
        LuaValue::Nil => Ok(environment.clone()),
        LuaValue::Function(function) => Ok(function),
        value => Err(LuaError::runtime(format!(
            "eech: [sound-data-provider:missing] the sample metadata provider must be a function, got {}",
            value.type_name()
        ))),
    }
}

/// A Lua number that is a whole number inside an `i32` (an integer, or a float with no fraction), or the stable refusal naming the field.
fn catalogue_integer(value: LuaValue, what: &str) -> Result<i32, String> {
    let refuse = || format!("[sound-data-provider:answer] {what} must be a whole number that fits a 32-bit integer");
    match value {
        LuaValue::Integer(number) => i32::try_from(number).map_err(|_| refuse()),
        LuaValue::Number(number) if number.is_finite() && number.fract() == 0.0 && number >= f64::from(i32::MIN) && number <= f64::from(i32::MAX) => {
            Ok(number as i32)
        }
        _ => Err(refuse()),
    }
}

fn catalogue_boolean(value: LuaValue, what: &str) -> Result<bool, String> {
    match value {
        LuaValue::Boolean(flag) => Ok(flag),
        other => Err(format!("[sound-data-provider:answer] {what} must be a boolean, got {}", other.type_name())),
    }
}

/// Builds the sample metadata provider the engine keeps from the host's Lua function (the host's environment provider unless the boot gave a separate `sound_data` function). The request is
/// `{ op = "sound_catalogue", slots = N, entries = { { index = slot, name = asset }, ... } }` and the answer `{ header_present = bool, header_count = n, entries = { { rate, original_rate, matched = bool, size }, ... } }`
/// in the request's order. A Lua error, a missing field and a value of the wrong kind are refusals that name the sample metadata provider; the engine's boundary then checks the whole answer.
fn sound_catalogue_provider(lua: &Lua, function: LuaFunction) -> SoundCatalogueProvider {
    let lua = lua.clone();
    Box::new(move |request: &SoundCatalogueRequest| -> Result<SoundCatalogueReply, String> {
        let callback = |error: LuaError| format!("[sound-data-provider:callback] {error}");
        let table = lua.create_table().map_err(callback)?;
        table.set("op", "sound_catalogue").map_err(callback)?;
        table.set("slots", request.slots).map_err(callback)?;
        let entries = lua.create_table().map_err(callback)?;
        for (position, (slot, name)) in request.descriptors.iter().enumerate() {
            let entry = lua.create_table().map_err(callback)?;
            entry.set("index", *slot).map_err(callback)?;
            entry.set("name", name.as_str()).map_err(callback)?;
            entries.set(position + 1, entry).map_err(callback)?;
        }
        table.set("entries", entries).map_err(callback)?;
        let answer: LuaTable = match function.call::<LuaValue>(table).map_err(callback)? {
            LuaValue::Table(answer) => answer,
            other => {
                return Err(format!(
                    "[sound-data-provider:answer] the provider answered with {} where a table was expected",
                    other.type_name()
                ))
            }
        };
        let field = |name: &str| answer.get::<LuaValue>(name).map_err(callback);
        let header_present = catalogue_boolean(field("header_present")?, "header_present")?;
        let header_count = catalogue_integer(field("header_count")?, "header_count")?;
        let listed = match field("entries")? {
            LuaValue::Table(listed) => listed,
            other => return Err(format!("[sound-data-provider:answer] entries must be a table, got {}", other.type_name())),
        };
        let mut entries = Vec::with_capacity(request.descriptors.len());
        for position in 1..=listed.raw_len() {
            let entry = match listed.get::<LuaValue>(position).map_err(callback)? {
                LuaValue::Table(entry) => entry,
                other => {
                    return Err(format!(
                        "[sound-data-provider:answer] entries[{position}] must be a table, got {}",
                        other.type_name()
                    ))
                }
            };
            let member = |name: &str| entry.get::<LuaValue>(name).map_err(callback);
            entries.push(SoundCatalogueEntry {
                rate: catalogue_integer(member("rate")?, &format!("entries[{position}].rate"))?,
                original_rate: catalogue_integer(member("original_rate")?, &format!("entries[{position}].original_rate"))?,
                matched: catalogue_boolean(member("matched")?, &format!("entries[{position}].matched"))?,
                size: catalogue_integer(member("size")?, &format!("entries[{position}].size"))?,
            });
        }
        Ok(SoundCatalogueReply {
            header_present,
            header_count,
            entries,
        })
    })
}

/// The function that answers the sound backend availability question: the boot's `sound_backend` when it gave one, else the host's environment provider (the reference host answers it). There is no campaign device
/// call to fall back to, so a `sound_backend` that is not a function is refused, with the stable marker; a provider that cannot answer is refused with the same marker family at the question, which the engine asks
/// before it stores anything.
fn sound_backend_function(config: &LuaTable, environment: &LuaFunction) -> LuaResult<LuaFunction> {
    match config.get::<LuaValue>("sound_backend")? {
        LuaValue::Nil => Ok(environment.clone()),
        LuaValue::Function(function) => Ok(function),
        value => Err(LuaError::runtime(format!(
            "eech: [sound-backend-provider:missing] the sound backend provider must be a function, got {}",
            value.type_name()
        ))),
    }
}

/// Builds the sound backend provider the engine keeps from the host's Lua function. The request is
/// `{ op = "sound_backend", phase = "initialise", no_sound = bool, dedicated = bool, load_samples = bool }` and the answer `{ available = bool }`. A Lua error, a missing field and a value of the wrong kind are
/// refusals that name the sound backend provider; whether `available = true` is acceptable is the campaign's decision at the C boundary (it is refused by name until the loader and playback routes are the host's).
fn sound_backend_provider(lua: &Lua, function: LuaFunction) -> SoundBackendProvider {
    let lua = lua.clone();
    Box::new(move |request: &SoundBackendRequest| -> Result<SoundBackendReply, String> {
        let callback = |error: LuaError| format!("[sound-backend-provider:callback] {error}");
        if request.phase != SOUND_BACKEND_PHASE_INITIALISE {
            return Err(format!(
                "[sound-backend-provider:request] the phase {} is not the initialisation",
                request.phase
            ));
        }
        let table = lua.create_table().map_err(callback)?;
        table.set("op", "sound_backend").map_err(callback)?;
        table.set("phase", "initialise").map_err(callback)?;
        table.set("no_sound", request.no_sound).map_err(callback)?;
        table.set("dedicated", request.dedicated).map_err(callback)?;
        table.set("load_samples", request.load_samples).map_err(callback)?;
        let answer: LuaTable = match function.call::<LuaValue>(table).map_err(callback)? {
            LuaValue::Table(answer) => answer,
            other => {
                return Err(format!(
                    "[sound-backend-provider:answer] the provider answered with {} where a table was expected",
                    other.type_name()
                ))
            }
        };
        match answer.get::<LuaValue>("available").map_err(callback)? {
            LuaValue::Boolean(available) => Ok(SoundBackendReply { available }),
            other => Err(format!(
                "[sound-backend-provider:answer] available must be a boolean, got {}",
                other.type_name()
            )),
        }
    })
}

fn env_state_table(lua: &Lua, state: &EnvState) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    for (name, value) in ENV_FLOAT_NAMES.iter().zip(state.f.iter()) {
        table.set(*name, f64::from(*value))?;
    }
    for (index, (name, value)) in ENV_INT_NAMES.iter().zip(state.n.iter()).enumerate() {
        if index < 2 {
            table.set(*name, *value)?;
        } else {
            table.set(*name, *value != 0)?;
        }
    }
    Ok(table)
}

fn env_named_floats(lua: &Lua, names: &[&str], values: &[f32]) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    for (name, value) in names.iter().zip(values.iter()) {
        table.set(*name, f64::from(*value))?;
    }
    Ok(table)
}

fn env_op_name(op: i32) -> &'static str {
    match op {
        EECH_ENV_IMPORT => "import",
        EECH_ENV_EXPORT => "export",
        EECH_ENV_INIT_RANDOM => "init_random",
        EECH_ENV_INIT_FIXED => "init_fixed",
        EECH_ENV_WEATHER => "weather",
        EECH_ENV_WIND => "wind",
        EECH_ENV_GUST => "gust",
        EECH_ENV_STEP => "step",
        EECH_ENV_AMBIENT => "ambient",
        EECH_ENV_LIGHTNING_ADD => "lightning_add",
        EECH_ENV_LIGHTNING_UPDATE => "lightning_update",
        EECH_ENV_SOUND_LIFETIME_INIT => "sound_lifetime_init",
        EECH_ENV_SOUND_LIFETIME_UPDATE => "sound_lifetime_update",
        EECH_ENV_SOUND_PAUSE_RESUME => "sound_pause_resume",
        EECH_ENV_ROTOR_SOUND => "rotor_sound",
        _ => "unknown",
    }
}

/// The request as the provider's table: the numbers are Lua numbers, unchanged (a negative zero and a draw arrive bit for bit).
fn env_request_table(lua: &Lua, request: &EnvRequest) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    table.set("op", env_op_name(request.op))?;
    table.set("winter", request.winter != 0)?;
    let map_names = ["min_x", "max_x", "min_z", "max_z", "mid_x", "mid_z"];
    match request.op {
        EECH_ENV_IMPORT => table.set("state", env_state_table(lua, &request.state)?)?,
        EECH_ENV_EXPORT => {}
        EECH_ENV_INIT_RANDOM => {
            let draws = lua.create_table()?;
            for (index, draw) in request.draw.iter().enumerate() {
                draws.set(index + 1, *draw)?;
            }
            table.set("draws", draws)?;
            let config_names = [
                "rain_radius_minimum",
                "rain_radius_maximum",
                "rain_speed",
                "wind_radius_minimum",
                "wind_radius_maximum",
                "wind_speed",
            ];
            table.set("config", env_named_floats(lua, &config_names, &request.init_config)?)?;
            table.set("map", env_named_floats(lua, &map_names, &request.map)?)?;
        }
        EECH_ENV_INIT_FIXED => table.set("mode", request.fixed_mode)?,
        EECH_ENV_WEATHER => {
            table.set("x", f64::from(request.x))?;
            table.set("z", f64::from(request.z))?;
        }
        EECH_ENV_WIND => {
            table.set("x", f64::from(request.x))?;
            table.set("z", f64::from(request.z))?;
            table.set("want_vector", request.want_vector != 0)?;
        }
        EECH_ENV_GUST => table.set("phase", f64::from(request.phase))?,
        EECH_ENV_AMBIENT => {
            table.set("x", f64::from(request.x))?;
            table.set("z", f64::from(request.z))?;
            table.set("y", f64::from(request.ambient_y))?;
            table.set("alt", f64::from(request.ambient_alt))?;
            table.set("cloud_base", f64::from(request.ambient_cloud_base))?;
            table.set("in_cockpit", request.in_cockpit != 0)?;
        }
        EECH_ENV_LIGHTNING_ADD => {
            table.set("type", request.lightning_type)?;
            table.set("length", f64::from(request.lightning_length))?;
            table.set("x", f64::from(request.x))?;
            table.set("z", f64::from(request.z))?;
            table.set("environment_active", request.lightning_environment_active != 0)?;
            // the time of day is what the original read from the active environment: with none there is nothing to give
            if request.lightning_environment_active != 0 {
                table.set("time_of_day", f64::from(request.lightning_time_of_day))?;
            }
        }
        EECH_ENV_LIGHTNING_UPDATE => {
            table.set("time_of_day", f64::from(request.lightning_time_of_day))?;
            if request.lightning_have_frames != 0 {
                table.set("frames", request.lightning_frames)?;
            }
        }
        EECH_ENV_SOUND_LIFETIME_INIT => {
            table.set("looping", request.sound_looping != 0)?;
            table.set("music", request.sound_music != 0)?;
            let samples = lua.create_table()?;
            let count = usize::try_from(request.sound_count).unwrap_or(0);
            for (index, (rate, size)) in request
                .sound_rate
                .iter()
                .zip(request.sound_size.iter())
                .take(count.min(EECH_SOUND_SAMPLES))
                .enumerate()
            {
                let sample = lua.create_table()?;
                sample.set("rate", *rate)?;
                sample.set("size", *size)?;
                samples.set(index + 1, sample)?;
            }
            table.set("samples", samples)?;
        }
        EECH_ENV_SOUND_LIFETIME_UPDATE => {
            table.set("server", request.sound_server != 0)?;
            table.set("delta", f64::from(request.sound_delta))?;
            table.set("valid", request.sound_valid != 0)?;
            table.set("looping", request.sound_looping != 0)?;
            table.set("lifetime", f64::from(request.sound_lifetime))?;
            table.set("valid_lifetime", f64::from(request.sound_valid_lifetime))?;
            table.set("has_handle", request.sound_has_handle != 0)?;
            if request.sound_have_playing != 0 {
                table.set("playing", request.sound_playing != 0)?;
            }
        }
        EECH_ENV_SOUND_PAUSE_RESUME => {
            table.set("resume", request.sound_resume != 0)?;
            table.set("valid", request.sound_valid != 0)?;
            table.set("member", request.sound_member != 0)?;
            table.set("valid_lifetime", f64::from(request.sound_valid_lifetime))?;
            if request.sound_resume == 0 {
                table.set("delay", f64::from(request.sound_delay))?;
            }
        }
        EECH_ENV_ROTOR_SOUND => {
            // the semantic role (never the EECH enumeration), the stored rpm, the global flight model and its acceleration, and the engine words only where the original reads them
            let role = match request.rotor_player {
                EECH_ROTOR_PLAYER_AI => "ai",
                EECH_ROTOR_PLAYER_LOCAL => "local",
                EECH_ROTOR_PLAYER_REMOTE => "remote",
                other => return Err(LuaError::runtime(format!("the rotor player role {other} is not one of the protocol's roles"))),
            };
            table.set("player", role)?;
            table.set("rpm", f64::from(request.rotor_rpm))?;
            table.set("flight_model", request.rotor_have_flight_model != 0)?;
            if request.rotor_have_flight_model != 0 {
                table.set("acceleration", env_named_floats(lua, &["x", "y", "z"], &request.rotor_acceleration)?)?;
                if request.rotor_player == EECH_ROTOR_PLAYER_LOCAL {
                    table.set("left_rpm", f64::from(request.rotor_left_rpm))?;
                    table.set("right_rpm", f64::from(request.rotor_right_rpm))?;
                    table.set("apu_rpm", f64::from(request.rotor_apu_rpm))?;
                }
            }
        }
        EECH_ENV_STEP => {
            table.set("delta", f64::from(request.delta))?;
            table.set("server", request.server != 0)?;
            table.set("in_cockpit", request.in_cockpit != 0)?;
            if request.have_camera != 0 {
                table.set("camera", env_named_floats(lua, &["x", "y", "z"], &request.camera)?)?;
            }
            if request.have_draw != 0 {
                table.set("lightning_draw", request.lightning_draw)?;
            }
            table.set("map", env_named_floats(lua, &map_names, &request.map)?)?;
        }
        other => return Err(LuaError::runtime(format!("unknown environment operation {other}"))),
    }
    Ok(table)
}

/// The numbers of an answer's text; a count that is not `count`, or a number that is not finite, is a refusal under `marker`.
fn env_numbers<const N: usize>(text: &str, marker: &str) -> Result<[f64; N], String> {
    let mut numbers = [0.0f64; N];
    let mut parts = text.split_whitespace();
    for slot in &mut numbers {
        *slot = parts
            .next()
            .and_then(|part| part.parse::<f64>().ok())
            .filter(|value| value.is_finite())
            .ok_or_else(|| format!("[env-provider:{marker}] the answer's numbers are malformed or too few"))?;
    }
    if parts.next().is_some() {
        return Err(format!("[env-provider:{marker}] the answer has more numbers than the {N} expected"));
    }
    Ok(numbers)
}

/// The numbers of a field (the floats, then the integers) and of a step answer (its six leading numbers, then the field).
const ENV_STATE_NUMBERS: usize = EECH_ENV_FLOATS + EECH_ENV_INTS;
const ENV_STEP_NUMBERS: usize = 6 + ENV_STATE_NUMBERS;
/// The numbers of a lightning queue answer: the leading `0`, the slot, and eight numbers for each slot of the queue.
const ENV_LIGHTNING_NUMBERS: usize = 2 + 8 * EECH_LIGHTNING_SLOTS;

fn env_f32(value: f64, marker: &str) -> Result<f32, String> {
    nearest_f32(value).ok_or_else(|| format!("[env-provider:{marker}] a number is beyond the single-precision range"))
}

fn env_i32(value: f64, marker: &str) -> Result<i32, String> {
    if value.fract() == 0.0 && value.abs() <= f64::from(i32::MAX) {
        Ok(value as i32)
    } else {
        Err(format!("[env-provider:{marker}] an integer is out of range"))
    }
}

/// The field from the front of an answer's numbers: the floats, then the integers (the modes, then the flags as 0 or 1).
fn env_state_from<'a>(numbers: &mut impl Iterator<Item = &'a f64>, marker: &str) -> Result<EnvState, String> {
    let mut state = EnvState::default();
    for slot in state.f.iter_mut() {
        let value = numbers.next().ok_or_else(|| format!("[env-provider:{marker}] the field is short"))?;
        *slot = env_f32(*value, marker)?;
    }
    for slot in state.n.iter_mut() {
        let value = numbers.next().ok_or_else(|| format!("[env-provider:{marker}] the field is short"))?;
        *slot = env_i32(*value, marker)?;
    }
    Ok(state)
}

fn env_mode(value: f64, marker: &str) -> Result<i32, String> {
    let mode = env_i32(value, marker)?;
    if (0..EECH_ENV_MODE_COUNT).contains(&mode) {
        Ok(mode)
    } else {
        Err(format!("[env-provider:{marker}] the mode {mode} is not one of the modes"))
    }
}

/// What the Lua environment bridge returns: the refusal marker and the text of the answer's numbers (or the refusal's message).
type EnvBridgeAnswer = (Option<String>, Option<String>);

fn call_environment(lua: &Lua, bridge: &LuaFunction, request: &EnvRequest) -> Result<EnvReply, String> {
    let op = env_op_name(request.op);
    let table = env_request_table(lua, request).map_err(|error| format!("[env-provider:{op}.callback] {error}"))?;
    let (marker, text): EnvBridgeAnswer = bridge.call(table).map_err(|error| format!("[env-provider:{op}.callback] {error}"))?;
    if let Some(marker) = marker {
        return Err(format!("[env-provider:{marker}] {}", text.unwrap_or_default()));
    }
    let text = text.ok_or_else(|| format!("[env-provider:{op}.answer] the environment bridge returned nothing"))?;
    let answer = format!("{op}.answer");
    let state = format!("{op}.state");
    let mut reply = EnvReply::default();
    match request.op {
        EECH_ENV_IMPORT => {}
        EECH_ENV_EXPORT | EECH_ENV_INIT_RANDOM | EECH_ENV_INIT_FIXED => {
            let numbers = env_numbers::<ENV_STATE_NUMBERS>(&text, &state)?;
            reply.state = env_state_from(&mut numbers.iter(), &state)?;
        }
        EECH_ENV_WEATHER => {
            let n = env_numbers::<4>(&text, &answer)?;
            reply.current_mode = env_mode(n[0], &answer)?;
            reply.target_mode = env_mode(n[1], &answer)?;
            reply.transitional_state = env_f32(n[2], &answer)?;
            reply.simple_mode = env_mode(n[3], &answer)?;
        }
        EECH_ENV_WIND => {
            let n = env_numbers::<4>(&text, &answer)?;
            reply.speed = env_f32(n[0], &answer)?;
            reply.vector = [env_f32(n[1], &answer)?, env_f32(n[2], &answer)?, env_f32(n[3], &answer)?];
        }
        EECH_ENV_GUST => {
            let n = env_numbers::<1>(&text, &answer)?;
            reply.modulation = env_f32(n[0], &answer)?;
        }
        EECH_ENV_AMBIENT => {
            let n = env_numbers::<4>(&text, &answer)?;
            reply.ambient_levels = [
                env_f32(n[0], &answer)?,
                env_f32(n[1], &answer)?,
                env_f32(n[2], &answer)?,
                env_f32(n[3], &answer)?,
            ];
        }
        EECH_ENV_SOUND_LIFETIME_INIT => {
            let n = env_numbers::<1>(&text, &answer)?;
            reply.sound_lifetime = env_f32(n[0], &answer)?;
        }
        EECH_ENV_SOUND_LIFETIME_UPDATE => {
            // "1" when the host needs to know whether the handle is playing; else "0 <lifetime> <valid lifetime> <expired> <action>"
            if text.trim() == "1" {
                reply.sound_need_playing = 1;
            } else {
                let n = env_numbers::<5>(&text, &answer)?;
                reply.sound_lifetime = env_f32(n[1], &answer)?;
                reply.sound_valid_lifetime = env_f32(n[2], &answer)?;
                reply.sound_expired = env_i32(n[3], &answer)?;
                reply.sound_action = env_i32(n[4], &answer)?;
            }
        }
        EECH_ENV_SOUND_PAUSE_RESUME => {
            // "0 <changed> <valid> <valid lifetime> <play> <relink>"
            let n = env_numbers::<6>(&text, &answer)?;
            reply.sound_changed = env_i32(n[1], &answer)?;
            reply.sound_valid = env_i32(n[2], &answer)?;
            reply.sound_valid_lifetime = env_f32(n[3], &answer)?;
            reply.sound_play = env_i32(n[4], &answer)?;
            reply.sound_relink = env_i32(n[5], &answer)?;
        }
        EECH_ENV_ROTOR_SOUND => {
            // "0", then for each of the channels "<write mask> <amplification> <pitch>", then "<count>" and for each request "<resume> <channel> <delay>"
            let n = text
                .split_whitespace()
                .map(|part| part.parse::<f64>().ok().filter(|value| value.is_finite()))
                .collect::<Option<Vec<f64>>>()
                .ok_or_else(|| format!("[env-provider:{answer}] the answer's numbers are malformed"))?;
            let fixed = 2 + 3 * EECH_ROTOR_CHANNELS;
            // the status, the channel triples, the count and the request triples, taken by structure: the whole length and the count are checked before anything is written
            let (Some(channel_numbers), Some(&count_number), Some(request_numbers)) = (n.get(1..fixed - 1), n.get(fixed - 1), n.get(fixed..)) else {
                return Err(format!("[env-provider:{answer}] the answer's numbers are too few"));
            };
            let count = env_i32(count_number, &answer)?;
            let count = usize::try_from(count)
                .ok()
                .filter(|count| *count <= EECH_ROTOR_ACTIONS)
                .ok_or_else(|| format!("[env-provider:{answer}] the number of requests is out of range"))?;
            if request_numbers.len() != 3 * count {
                return Err(format!(
                    "[env-provider:{answer}] the answer has {} numbers where {} requests need {}",
                    n.len(),
                    count,
                    fixed + 3 * count
                ));
            }
            let malformed = || format!("[env-provider:{answer}] the answer's numbers are malformed");
            // the destinations are the reply's own arrays, so no position can be out of them
            for (((write, amplification), pitch), triple) in reply
                .rotor_write
                .iter_mut()
                .zip(reply.rotor_amplification.iter_mut())
                .zip(reply.rotor_pitch.iter_mut())
                .zip(channel_numbers.chunks(3))
            {
                let &[mask, level, tone] = triple else {
                    return Err(malformed());
                };
                *write = env_i32(mask, &answer)?;
                *amplification = env_f32(level, &answer)?;
                *pitch = env_f32(tone, &answer)?;
            }
            reply.rotor_action_count = env_i32(count_number, &answer)?;
            for (((resume, channel), delay), triple) in reply
                .rotor_action_resume
                .iter_mut()
                .zip(reply.rotor_action_channel.iter_mut())
                .zip(reply.rotor_action_delay.iter_mut())
                .zip(request_numbers.chunks(3))
            {
                let &[flag, which, wait] = triple else {
                    return Err(malformed());
                };
                *resume = env_i32(flag, &answer)?;
                *channel = env_i32(which, &answer)?;
                *delay = env_f32(wait, &answer)?;
            }
        }
        EECH_ENV_LIGHTNING_ADD | EECH_ENV_LIGHTNING_UPDATE => {
            // "1" when an update needs the strike animation's frame count; else "0 <slot>" and eight numbers for each of the queue's slots (in_use, type, x, y, z, time started, length, frame)
            if text.trim() == "1" {
                reply.lightning_need_frames = 1;
                reply.lightning_slot = -1;
            } else {
                let n = env_numbers::<ENV_LIGHTNING_NUMBERS>(&text, &answer)?;
                // the leading 0 is skipped; the count was checked whole by env_numbers
                let mut numbers = n.iter().skip(1);
                let mut next = || -> Result<f64, String> {
                    numbers
                        .next()
                        .copied()
                        .ok_or_else(|| format!("[env-provider:{answer}] the queue answer is short"))
                };
                reply.lightning_slot = env_i32(next()?, &answer)?;
                for strike in reply.lightning_queue.iter_mut() {
                    strike.in_use = env_i32(next()?, &answer)?;
                    strike.kind = env_i32(next()?, &answer)?;
                    strike.x = env_f32(next()?, &answer)?;
                    strike.y = env_f32(next()?, &answer)?;
                    strike.z = env_f32(next()?, &answer)?;
                    strike.time_started = env_f32(next()?, &answer)?;
                    strike.time_length = env_f32(next()?, &answer)?;
                    strike.current_frame_number = env_i32(next()?, &answer)?;
                }
            }
        }
        _ => {
            // step: "1 <timer>" when the host needs the draw, else "0 <expired> <strike x y z> <new timer> <the field>"
            let first = text.split_whitespace().next().unwrap_or_default();
            if first == "1" {
                let n = env_numbers::<2>(&text, &answer)?;
                reply.need_draw = 1;
                reply.new_timer = env_f32(n[1], &answer)?;
            } else {
                let n = env_numbers::<ENV_STEP_NUMBERS>(&text, &answer)?;
                let expired = env_i32(n[1], &answer)?;
                if expired != 0 && expired != 1 {
                    return Err(format!("[env-provider:{answer}] expired is neither 0 nor 1"));
                }
                reply.expired = expired;
                reply.strike = [env_f32(n[2], &answer)?, env_f32(n[3], &answer)?, env_f32(n[4], &answer)?];
                reply.new_timer = env_f32(n[5], &answer)?;
                reply.state = env_state_from(&mut n.iter().skip(6), &state)?;
            }
        }
    }
    Ok(reply)
}

/// What the Lua ground bridge returns, in order: the refusal marker, the answer's numbers as text, then the record's surface type, hint and stored flag, the predicate and
/// whether the terrain was read (each absent when the answer does not carry it).
type GroundBridgeAnswer = (
    Option<String>,
    Option<String>,
    Option<i64>,
    Option<String>,
    Option<bool>,
    Option<bool>,
    Option<bool>,
);

fn ground_record(numbers: [f64; 4], op: &str, kind: Option<i64>, hint: Option<String>, stored: Option<bool>) -> Result<TerrainRecord, String> {
    let marker = format!("{op}.record");
    let [last_elevation, nx, ny, nz] = numbers;
    let kind = kind.ok_or_else(|| format!("[terrain-provider:{op}.type] the ground bridge returned no type"))?;
    Ok(TerrainRecord {
        last_elevation: terrain_f32(last_elevation, &marker)?,
        normal: [terrain_f32(nx, &marker)?, terrain_f32(ny, &marker)?, terrain_f32(nz, &marker)?],
        kind: i32::try_from(kind).map_err(|_| format!("[terrain-provider:{op}.type] the type is out of range"))?,
        hint,
        stored: stored.ok_or_else(|| format!("[terrain-provider:{marker}] the ground bridge returned no stored flag"))?,
    })
}

fn ground_triple(numbers: [f64; 3], marker: &str) -> Result<[f32; 3], String> {
    let [x, y, z] = numbers;
    Ok([terrain_f32(x, marker)?, terrain_f32(y, marker)?, terrain_f32(z, marker)?])
}

/// The ground-contact questions (EEDCS-48, D3 G2) through `GROUND_BRIDGE`: the request fields cross as exact numbers and the answer's shape is checked in Lua, where the values are.
fn call_ground(bridge: &LuaFunction, query: TerrainQuery) -> Result<TerrainAnswer, String> {
    fn widen(values: &[f32; 3]) -> [f64; 3] {
        let [x, y, z] = *values;
        [f64::from(x), f64::from(y), f64::from(z)]
    }
    let (op, winter, hint, world, first, second) = match &query {
        TerrainQuery::GroundAbove { position, world, hint, winter } => ("ground_above", *winter, hint.clone(), *world, widen(position), [0.0; 3]),
        TerrainQuery::GroundBelow { position, world, hint, winter } => ("ground_below", *winter, hint.clone(), *world, widen(position), [0.0; 3]),
        TerrainQuery::GroundImpact {
            above,
            below,
            world,
            hint,
            winter,
        } => ("ground_impact", *winter, hint.clone(), *world, widen(above), widen(below)),
        _ => return Err("[terrain-provider:point.callback] a non-ground question reached the ground bridge".to_string()),
    };
    let [world_min_x, world_min_z, world_max_x, world_max_z] = world;
    let [x, y, z] = first;
    let [below_x, below_y, below_z] = second;
    let (marker, text, kind, hint_out, stored, value, queried): GroundBridgeAnswer = bridge
        .call((
            op,
            winter,
            hint,
            f64::from(world_min_x),
            f64::from(world_min_z),
            f64::from(world_max_x),
            f64::from(world_max_z),
            x,
            y,
            z,
            below_x,
            below_y,
            below_z,
        ))
        .map_err(|error| format!("[terrain-provider:{op}.callback] {error}"))?;
    if let Some(marker) = marker {
        return Err(format!("[terrain-provider:{marker}] {}", text.unwrap_or_default()));
    }
    let text = text.ok_or_else(|| format!("[terrain-provider:{op}.answer] the ground bridge returned nothing"))?;
    if op == "ground_impact" {
        let [ax, ay, az, bx, by, bz, ix, iy, iz, last, nx, ny, nz] = terrain_numbers::<13>(&text, "ground_impact.answer")?;
        return Ok(TerrainAnswer::GroundImpact {
            above: ground_triple([ax, ay, az], "ground_impact.position")?,
            below: ground_triple([bx, by, bz], "ground_impact.position")?,
            impact: ground_triple([ix, iy, iz], "ground_impact.position")?,
            record: ground_record([last, nx, ny, nz], op, kind, hint_out, stored)?,
        });
    }
    let value = value.ok_or_else(|| format!("[terrain-provider:{op}.answer] the ground bridge returned no predicate"))?;
    match queried {
        Some(true) => {
            let n = terrain_numbers::<4>(&text, &format!("{op}.record"))?;
            Ok(TerrainAnswer::GroundPredicate {
                value,
                record: Some(ground_record(n, op, kind, hint_out, stored)?),
            })
        }
        Some(false) => Ok(TerrainAnswer::GroundPredicate { value, record: None }),
        None => Err(format!("[terrain-provider:{op}.answer] the ground bridge returned no queried flag")),
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
            aim_ids_into(&table, &None, &data.target_host_id)?;
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
            aim_ids_into(&table, &None, &data.target_host_id)?;
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
            aim_ids_into(&table, &None, &data.target_host_id)?;
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
                    if let Some(id) = &entry.host_id {
                        object.set("host_id", id.as_str()).map_err(e)?;
                    }
                    objects.set(index + 1, object).map_err(e)?;
                }
                window_table.set("objects", objects).map_err(e)?;
                table.set("window", window_table).map_err(e)?;
            }
        }
        AimRequest::Identify(data) => {
            // the facts of a structure the host is asked to issue an identity for, by value
            table.set("mode", "identify").map_err(e)?;
            table.set("entity_type", data.entity_type).map_err(e)?;
            table.set("sub_type", data.sub_type).map_err(e)?;
            table.set("side", data.side).map_err(e)?;
            table.set("alive", data.alive).map_err(e)?;
            table.set("damage_level", data.damage_level).map_err(e)?;
            table.set("initial_damage_level", data.initial_damage_level).map_err(e)?;
            table.set("position", data.position.to_vec()).map_err(e)?;
            table.set("serial", data.serial).map_err(e)?;
        }
        AimRequest::Bridge(data) => {
            // a question about the physical collapse of a bridge segment, by value; `terrain` is present once the host has asked for it, `window` once the vehicles walk has been given one
            table.set("mode", "bridge").map_err(e)?;
            table
                .set(
                    "op",
                    match data.op {
                        1 => "kill",
                        2 => "shape",
                        3 => "neighbour",
                        4 => "fall",
                        5 => "vehicles",
                        _ => "rest",
                    },
                )
                .map_err(e)?;
            table.set("segment_kind", data.segment_kind).map_err(e)?;
            table.set("alive", data.alive).map_err(e)?;
            table.set("server", data.server).map_err(e)?;
            table.set("pred_present", data.pred_present).map_err(e)?;
            table.set("pred_alive", data.pred_alive).map_err(e)?;
            table.set("succ_present", data.succ_present).map_err(e)?;
            table.set("succ_alive", data.succ_alive).map_err(e)?;
            table.set("which", data.which).map_err(e)?;
            table.set("neighbour_present", data.neighbour_present).map_err(e)?;
            table.set("neighbour_kind", data.neighbour_kind).map_err(e)?;
            table.set("neighbour_alive", data.neighbour_alive).map_err(e)?;
            table.set("position", data.position.to_vec()).map_err(e)?;
            table.set("delta_time", data.delta_time).map_err(e)?;
            if let Some((height, water)) = data.terrain {
                let terrain = lua.create_table().map_err(e)?;
                terrain.set("height", height).map_err(e)?;
                terrain.set("water", water).map_err(e)?;
                table.set("terrain", terrain).map_err(e)?;
            }
            table.set("segment_length", data.segment_length).map_err(e)?;
            table.set("sector_side_length", data.sector_side_length).map_err(e)?;
            table.set("min_x_sector", data.min_x_sector).map_err(e)?;
            table.set("max_x_sector", data.max_x_sector).map_err(e)?;
            table.set("min_z_sector", data.min_z_sector).map_err(e)?;
            table.set("max_z_sector", data.max_z_sector).map_err(e)?;
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
                    if let Some(id) = &entry.host_id {
                        object.set("host_id", id.as_str()).map_err(e)?;
                    }
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

/// A text field that may be absent: the identity of an object an action names, repeated by the host (the campaign compares it with the identity it offered the object under).
fn aim_optional_text(table: &LuaTable, field: &str) -> Result<Option<String>, String> {
    match table.get::<LuaValue>(field).map_err(|e| e.to_string())? {
        LuaValue::Nil => Ok(None),
        LuaValue::String(text) => Ok(Some(text.to_str().map_err(|e| e.to_string())?.to_owned())),
        other => Err(format!("aim result {field} must be a string, got {}", other.type_name())),
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
                AimRequest::Bridge(_) => {
                    // a bridge question can ask for the terrain under the segment (a fall) or, for the vehicles walk, the window { sx, sz, after }
                    reply.need_lookup = true;
                    match table.get::<LuaValue>("need").map_err(|e| e.to_string())? {
                        LuaValue::String(need) if &*need.to_str().map_err(|e| e.to_string())? == "terrain" => reply.bridge_need_terrain = true,
                        LuaValue::Table(need) => {
                            reply.bridge_need = Some((aim_integer(&need, "sx")?, aim_integer(&need, "sz")?, aim_integer(&need, "after")?));
                        }
                        _ => return Err("aim result need_lookup answers a bridge question with a need that is neither the terrain nor a window".into()),
                    }
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
                reply.blast_target_host_id = aim_optional_text(&table, "host_id")?;
                reply.blast_modifier = number("modifier")?;
            }
        }
        AimRequest::Identify(_) => {
            // the identity the host issues for the structure: a string, mandatory (the campaign checks it is new, printable and not longer than 128 bytes)
            reply.identify_host_id = match aim_optional_text(&table, "host_id")? {
                Some(id) => Some(id),
                None => return Err("aim result host_id must be a string for an identify request".into()),
            };
        }
        AimRequest::Bridge(data) => {
            // the answer of the op, each field mandatory and exact; a fall names its figures only for what it did
            match data.op {
                1 => reply.bridge_kill = aim_boolean(&table, "kill")?,
                2 => reply.bridge_variant = aim_integer(&table, "variant")?,
                3 => reply.bridge_notify = aim_boolean(&table, "notify")?,
                4 => {
                    reply.bridge_fell = aim_boolean(&table, "fell")?;
                    reply.bridge_landed = aim_boolean(&table, "landed")?;
                    reply.bridge_remove = aim_boolean(&table, "remove")?;
                    if reply.bridge_fell {
                        reply.bridge_dropped_y = number("dropped_y")?;
                        reply.bridge_floor_height = number("floor_height")?;
                    }
                    if reply.bridge_landed {
                        reply.bridge_landed_y = number("landed_y")?;
                    }
                }
                6 => {
                    // where a segment rests when a saved world is unpacked: the height it rests at, or nothing
                    reply.bridge_landed = aim_boolean(&table, "landed")?;
                    if reply.bridge_landed {
                        reply.bridge_landed_y = number("landed_y")?;
                    }
                }
                _ => {
                    let action = match table.get::<LuaValue>("action").map_err(|e| e.to_string())? {
                        LuaValue::String(text) => text.to_str().map_err(|e| e.to_string())?.to_owned(),
                        other => return Err(format!("aim result action must be a string, got {}", other.type_name())),
                    };
                    reply.bridge_action = match action.as_str() {
                        "none" => 0,
                        "act" => 1,
                        other => return Err(format!("aim result action {other:?} is not none or act")),
                    };
                    if reply.bridge_action != 0 {
                        reply.bridge_handle = aim_integer(&table, "handle")?;
                        reply.bridge_target_host_id = aim_optional_text(&table, "host_id")?;
                    }
                }
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
        // TEST-ONLY (feature `collision-fixture`, EEDCS-49): the probe of the original object and tree collision queries against the loaded world; not part of the shipping
        // module. It needs no host callbacks: neither query asks one. The seed is a whole number; the case counts are checked again by the engine wrapper. The engine refuses it
        // unless every floating-point exception is masked (x87 and SSE), which it only reads.
        //
        // The arguments are converted HERE, from the raw argument list, with the same conversions the typed tuple (u32, i32, i32) used (`FromLuaMulti`), so that a conversion refusal is
        // inside the method: a probe that was entered consumes any arming of the controlled geometry witness (`collision_geometry_arm`) whatever refuses it, and the next probe is
        // never unexpectedly controlled. Nothing is borrowed across the conversion (it can run Lua metamethods); the engine call clears the arming itself on its own refusals.
        #[cfg(feature = "collision-fixture")]
        methods.add_method("collision_probe", |lua, this, arguments: mlua::MultiValue| {
            let result: mlua::Result<String> = (|| {
                let (seed, entity_cases, tree_cases): (u32, i32, i32) = mlua::FromLuaMulti::from_lua_multi(arguments, lua)?;
                this.engine.collision_probe(seed, entity_cases, tree_cases).map_err(lua_error)
            })();
            if result.is_err() {
                let _ = this.engine.collision_geometry_arm(false);
            }
            result
        });
        // TEST-ONLY (feature `collision-fixture`, EEDCS-49): the optional private tree-query transcript of the collision probe (arm, then read the state and facts or a chunk of records);
        // not part of the shipping module.
        #[cfg(feature = "collision-fixture")]
        methods.add_method("collision_trace_arm", |_, this, on: bool| {
            this.engine.collision_trace_arm(on).map_err(lua_error)
        });
        // TEST-ONLY (feature `collision-fixture`, EEDCS-49): arms the controlled forest geometry witness for the NEXT collision_probe only (consumed by it); not part of the shipping module.
        #[cfg(feature = "collision-fixture")]
        methods.add_method("collision_geometry_arm", |_, this, on: bool| {
            this.engine.collision_geometry_arm(on).map_err(lua_error)
        });
        #[cfg(feature = "collision-fixture")]
        methods.add_method("collision_trace_read", |_, this, (kind, offset, count): (i32, i32, i32)| {
            this.engine.collision_trace_read(kind, offset, count).map_err(lua_error)
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
        // TEST-ONLY (feature `model-fixture`, EEDCS-50): read-only questions to the original collision-model accessors; not part of the shipping module. The arguments are
        // whole numbers within the page bounds, checked before the engine is entered.
        #[cfg(feature = "model-fixture")]
        methods.add_method("model_summary", |_, this, ()| this.engine.model_summary().map_err(lua_error));
        #[cfg(feature = "model-fixture")]
        methods.add_method("model_catalogue", |_, this, (first, count): (LuaValue, LuaValue)| {
            let first = model_int(&first, "first", 1, i32::MAX - 1)?;
            let count = model_int(&count, "count", 1, eech_engine::MODEL_PAGE_SCENES)?;
            this.engine.model_catalogue(first, count).map_err(lua_error)
        });
        #[cfg(feature = "model-fixture")]
        methods.add_method("model_geometry", |_, this, (scene, first_depth, count): (LuaValue, LuaValue, LuaValue)| {
            let scene = model_int(&scene, "scene", 1, i32::MAX - 1)?;
            let first_depth = model_int(&first_depth, "first_depth", 0, i32::MAX - eech_engine::MODEL_PAGE_TRIANGLES)?;
            let count = model_int(&count, "count", 1, eech_engine::MODEL_PAGE_TRIANGLES)?;
            this.engine.model_geometry(scene, first_depth, count).map_err(lua_error)
        });
        #[cfg(feature = "model-fixture")]
        methods.add_method("model_state", |_, this, ()| this.engine.model_state().map_err(lua_error));
        // TEST-ONLY (feature `model-fixture`, EEDCS-50 D3): one page of the scenes' final texture animation lists as the original loader left them; not part of the shipping module.
        #[cfg(feature = "model-fixture")]
        methods.add_method("model_animations", |_, this, (first, count): (LuaValue, LuaValue)| {
            let first = model_int(&first, "first", 1, i32::MAX - eech_engine::MODEL_PAGE_SCENES)?;
            let count = model_int(&count, "count", 1, eech_engine::MODEL_PAGE_SCENES)?;
            this.engine.model_animations(first, count).map_err(lua_error)
        });
        // TEST-ONLY (feature `model-host-fixture`, EEDCS-50 D3): the host's model answers counted, and the campaign's own model arrays overwritten so that any read of them
        // would show; not part of the shipping module.
        #[cfg(feature = "model-host-fixture")]
        methods.add_method("model_host_counters", |_, this, ()| this.engine.model_host_counters().map_err(lua_error));
        // TEST-ONLY (feature `model-host-fixture`, EEDCS-50 D3 L2): one page of the scenes' final texture animation lists after the host-fed load, in the original observer's row grammar
        #[cfg(feature = "model-host-fixture")]
        methods.add_method("model_host_animations", |_, this, (first, count): (LuaValue, LuaValue)| {
            let first = model_int(&first, "first", 1, i32::MAX - eech_engine::MODEL_PAGE_SCENES)?;
            let count = model_int(&count, "count", 1, eech_engine::MODEL_PAGE_SCENES)?;
            this.engine.model_host_animations(first, count).map_err(lua_error)
        });
        // TEST-ONLY (feature `model-host-fixture`, EEDCS-52 D3): the host-authored target-point offsets as the campaign holds them, and the direct guidance consumer called for real
        #[cfg(feature = "model-host-fixture")]
        methods.add_method("model_host_target_offsets", |_, this, (first, count): (LuaValue, LuaValue)| {
            let first = model_int(&first, "first", 1, i32::MAX - eech_engine::MODEL_PAGE_SCENES)?;
            let count = model_int(&count, "count", 1, eech_engine::MODEL_PAGE_SCENES)?;
            this.engine.model_host_target_offsets(first, count).map_err(lua_error)
        });
        // TEST-ONLY (feature `model-host-fixture`, EEDCS-52 D3): the target-point query called for real on live entities; `mutate` changes a leaf's live placement for the call
        #[cfg(feature = "model-host-fixture")]
        methods.add_method("model_host_target_queries", |_, this, mutate: bool| {
            this.engine.model_host_target_queries(mutate).map_err(lua_error)
        });
        #[cfg(feature = "model-host-fixture")]
        methods.add_method("model_host_attack_consumer", |_, this, ()| {
            this.engine.model_host_attack_consumer().map_err(lua_error)
        });
        #[cfg(feature = "model-host-fixture")]
        methods.add_method("model_host_poison", |_, this, ()| this.engine.model_host_poison().map_err(lua_error));
        // the complete outer floating-point state around a real public entry (see eech_engine_model_host_fpu); the arguments are whole numbers in range, checked before the engine is entered
        #[cfg(feature = "model-host-fixture")]
        methods.add_method(
            "model_host_fpu",
            |_, this, (kind, scene, depth, rounding): (LuaValue, LuaValue, LuaValue, LuaValue)| {
                let kind = model_int(&kind, "kind", 0, 1)?;
                let scene = model_int(&scene, "scene", 1, i32::MAX - 1)?;
                let depth = model_int(&depth, "depth", 0, i32::MAX - 1)?;
                let rounding = model_int(&rounding, "rounding", 0, 3)?;
                this.engine.model_host_fpu(kind, scene, depth, rounding).map_err(lua_error)
            },
        );
        // the scene and depth are whole numbers in range, checked before the engine is entered (the same helper as the original-accessor fixture's)
        // TEST-ONLY (feature `model-host-fixture`): the REAL add_3d_horizon_image called on the campaign's horizon environment with a basename no installation has; the engine stops by name
        // and is poisoned (see Engine::model_host_horizon_refusal). Not part of the shipping module.
        #[cfg(feature = "model-host-fixture")]
        methods.add_method("model_host_horizon_refusal", |_, this, ()| {
            this.engine.model_host_horizon_refusal().map_err(lua_error)
        });
        #[cfg(feature = "model-host-fixture")]
        methods.add_method("model_host_probe", |_, this, (scene, depth): (LuaValue, LuaValue)| {
            let scene = model_int(&scene, "scene", 1, i32::MAX - 1)?;
            let depth = model_int(&depth, "depth", 0, i32::MAX - 1)?;
            this.engine.model_host_probe(scene, depth).map_err(lua_error)
        });
        // TEST-ONLY (feature `env-observer`, EEDCS-51): the read-only observer's report and unread events; not part of the shipping module.
        #[cfg(feature = "env-observer")]
        methods.add_method("env_observer_report", |_, this, ()| this.engine.env_observer_report().map_err(lua_error));
        #[cfg(feature = "env-observer")]
        methods.add_method("env_observer_events", |_, this, ()| this.engine.env_observer_events().map_err(lua_error));
        // TEST-ONLY (feature `env-fixture`, EEDCS-51, D3): `env_case` with a report buffer of the caller's size (a short-buffer guard); not part of the shipping module.
        #[cfg(feature = "env-fixture")]
        methods.add_method("env_case_capacity", |_, this, (group, index, capacity): (i32, i32, usize)| {
            this.engine.env_case_capacity(group, index, capacity).map_err(lua_error)
        });
        // TEST-ONLY (feature `env-observer`, EEDCS-51, D3): the helicopter rotor sound probe's report and its drained calls; not part of the shipping module.
        #[cfg(feature = "env-observer")]
        methods.add_method("env_rotor_report", |_, this, ()| this.engine.env_rotor_report().map_err(lua_error));
        #[cfg(feature = "env-observer")]
        methods.add_method("env_rotor_events", |_, this, ()| this.engine.env_rotor_events().map_err(lua_error));
        // TEST-ONLY (feature `env-observer`, EEDCS-51, D3): the resumed-parent playback probe's report and its drained calls; not part of the shipping module.
        #[cfg(feature = "env-observer")]
        methods.add_method("env_play_report", |_, this, ()| this.engine.env_play_report().map_err(lua_error));
        #[cfg(feature = "env-observer")]
        methods.add_method("env_play_events", |_, this, ()| this.engine.env_play_events().map_err(lua_error));
        // TEST-ONLY (feature `env-fixture`, EEDCS-51): one case of the original session environment characterisation (group, index); not part of the shipping module.
        #[cfg(feature = "env-fixture")]
        methods.add_method("env_case", |_, this, (group, index): (i32, i32)| {
            this.engine.env_case(group, index).map_err(lua_error)
        });
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
        // TEST-ONLY (feature `weapon-fixture`, EEDCS-46, R5): the controlled bridge case of the ORIGINAL collapse (op 0 survey, op 1 state, op 2 the declared direct structural failure of a segment, op 3
        // disarm), given the same complete host as a frame; not part of the shipping module.
        #[cfg(feature = "weapon-fixture")]
        methods.add_method("cmb_case_bridge", |lua, this, (op, ordinal, index, criterion): (i32, i32, i32, i32)| {
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
                .cmb_case_bridge(
                    op,
                    ordinal,
                    index,
                    criterion,
                    &mut binder,
                    &mut observer,
                    &mut reach,
                    &mut mover,
                    &mut sighter,
                    &mut sensor,
                    &mut aimer,
                )
                .map_err(lua_error)
        }); // TEST-ONLY (feature `weapon-fixture`, EEDCS-46): the ordered combat event log, drained from the cursor; not part of the shipping module.
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
        // TEST-ONLY (feature `weapon-fixture`, EEDCS-49 D3): the controlled moving entity query case (latch three finite segments about one routed vehicle, then ask the production entity
        // query for them); not part of the shipping module.
        #[cfg(feature = "weapon-fixture")]
        methods.add_method("wpn_moving_query_prepare", |_, this, host_id: String| {
            this.engine.wpn_moving_query_prepare(&host_id).map_err(lua_error)
        });
        #[cfg(feature = "weapon-fixture")]
        methods.add_method("wpn_moving_query_run", |_, this, ()| this.engine.wpn_moving_query_run().map_err(lua_error));
        // TEST-ONLY (feature `weapon-fixture`, EEDCS-49 D3, REQUIRED_CITY_BOX_LIFECYCLE_RETENTION): drives one real bound city through the campaign's shared drain, empty relink and restore, relist services with the
        // production entity query before and after; terminal, never followed by a physical run; not part of the shipping module.
        #[cfg(feature = "weapon-fixture")]
        methods.add_method("wpn_city_lifecycle", |_, this, ()| this.engine.wpn_city_lifecycle().map_err(lua_error));
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
        // TEST-ONLY (feature `terrain-fixture`, EEDCS-47): the run-wide report of the terrain queries; not part of the shipping module.
        #[cfg(feature = "terrain-fixture")]
        methods.add_method("terrain_trace", |_, this, ()| this.engine.terrain_trace().map_err(lua_error));
        // TEST-ONLY (feature `terrain-fixture`, EEDCS-47): clears that trace; not part of the shipping module.
        #[cfg(feature = "terrain-fixture")]
        methods.add_method("terrain_trace_reset", |_, this, ()| this.engine.terrain_trace_reset().map_err(lua_error));
        // TEST-ONLY (feature `terrain-fixture`, EEDCS-47): the read-only deterministic terrain sweep; not part of the shipping module. In the unmoved module it asks
        // nothing of the host; in the host-terrain module its queries are questions to the terrain provider.
        #[cfg(feature = "terrain-fixture")]
        methods.add_method("terrain_probe", |_, this, (seed, count, emit): (u32, i32, i32)| {
            this.engine.terrain_probe(seed, count, emit).map_err(lua_error)
        });
        // TEST-ONLY (feature `terrain-fixture`, EEDCS-47): the campaign's record predicate through the real client queries; not part of the shipping module.
        #[cfg(feature = "terrain-fixture")]
        methods.add_method("terrain_face_probe", |_, this, (x, z): (f32, f32)| {
            this.engine.terrain_face_probe(x, z).map_err(lua_error)
        });
        // TEST-ONLY (feature `ground-fixture`, EEDCS-48): the read-only ground-contact probe of the original point_above_ground, point_below_ground and
        // get_ground_impact_point against the loaded terrain; not part of the shipping module. It takes no host callbacks of its own. In the unmoved (original)
        // module the three read the DLL's terrain; in the host-terrain module (EECH_CAMPAIGN_HOST_TERRAIN) the elevation, normal and type reads of the probe's
        // queries are questions to the terrain provider that boot installed for the rest of the process, so the engine must have been booted with one.
        #[cfg(feature = "ground-fixture")]
        methods.add_method("ground_probe", |_, this, (seed, points, impacts): (u32, i32, i32)| {
            this.engine.ground_probe(seed, points, impacts).map_err(lua_error)
        });
        // TEST-ONLY (feature `sector-height-observer`, EEDCS-49 D3): the read-only report of the sector tallest-structure observer; not part of the shipping module.
        #[cfg(feature = "sector-height-observer")]
        methods.add_method("sector_height_summary", |_, this, ()| this.engine.sector_height_summary().map_err(lua_error));
        #[cfg(feature = "sector-height-observer")]
        methods.add_method("sector_height_event", |_, this, index: i32| {
            this.engine.sector_height_event(index).map_err(lua_error)
        });
        #[cfg(feature = "sector-height-observer")]
        methods.add_method("sector_height_clearance", |_, this, index: i32| {
            this.engine.sector_height_clearance(index).map_err(lua_error)
        });
        #[cfg(feature = "sector-height-observer")]
        methods.add_method("sector_height_scan", |_, this, index: i32| {
            this.engine.sector_height_scan(index).map_err(lua_error)
        });
        // TEST-ONLY (feature `sector-height-witness`, EEDCS-49 D3, ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1): the campaign's own server-session save of the live session (the original store_session); the name is checked
        // before the engine is entered, and nothing is overwritten, rotated or deleted. Not part of the shipping module.
        #[cfg(feature = "sector-height-witness")]
        methods.add_method("sector_height_witness_save", |_, this, name: String| {
            this.engine.sector_height_witness_save(&name).map_err(lua_error)
        });
        // TEST-ONLY (feature `route-fixture`, EEDCS-50 D3): what the ORIGINAL airfield route parser read and produced during the campaign load; not part of the shipping module.
        #[cfg(feature = "route-fixture")]
        methods.add_method("route_summary", |_, this, ()| this.engine.route_summary().map_err(lua_error));
        #[cfg(feature = "route-fixture")]
        methods.add_method("route_call", |_, this, index: i32| this.engine.route_call(index).map_err(lua_error));
        // TEST-ONLY (feature `target-point-fixture`, EEDCS-52 D3): what the ORIGINAL target-point query, its callers and the boot census of the offset table did; not part of the shipping module.
        #[cfg(feature = "target-point-fixture")]
        methods.add_method("target_point_summary", |_, this, ()| this.engine.target_point_summary().map_err(lua_error));
        #[cfg(feature = "target-point-fixture")]
        methods.add_method("target_point_offsets", |_, this, generation: i32| {
            this.engine.target_point_offsets(generation).map_err(lua_error)
        });
        #[cfg(feature = "target-point-fixture")]
        methods.add_method("target_point_event", |_, this, index: i32| {
            this.engine.target_point_event(index).map_err(lua_error)
        });
        #[cfg(feature = "target-point-fixture")]
        methods.add_method("target_point_probe", |_, this, ()| this.engine.target_point_probe().map_err(lua_error));
        #[cfg(feature = "target-point-fixture")]
        methods.add_method("target_point_setup", |_, this, ()| this.engine.target_point_setup().map_err(lua_error));
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
        // The structures with the identity the host issued for each (EEDCS-46, R3): one that has none is identified now, which asks the host once for it.  Copied values; the identity is opaque.
        methods.add_method("structures", |lua, this, ()| {
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
            let structures = this
                .engine
                .structures(&mut binder, &mut observer, &mut reach, &mut mover, &mut sighter, &mut sensor, &mut aimer)
                .map_err(lua_error)?;
            lua.to_value_with(&structures, options())
        });
        // A physical fact the host reports about an object it holds the identity of (EEDCS-46, R3): { host_id = <identity>, damage_level = <integer, optional>, destroyed = <boolean, optional> }.
        // The campaign applies it once through the object's own handlers (which ask the host what they need) and returns what it applied.
        methods.add_method("report_object", |lua, this, report: LuaTable| {
            let host_id = match report.get::<LuaValue>("host_id")? {
                LuaValue::String(text) => text.to_str()?.to_owned(),
                other => {
                    return Err(LuaError::runtime(format!(
                        "eech: bad argument: report host_id must be a string, got {}",
                        other.type_name()
                    )))
                }
            };
            let damage_level = match report.get::<LuaValue>("damage_level")? {
                LuaValue::Nil => None,
                LuaValue::Integer(level) => {
                    Some(i32::try_from(level).map_err(|_| LuaError::runtime("eech: bad argument: report damage_level is out of range"))?)
                }
                LuaValue::Number(level) if level.fract() == 0.0 && level.abs() < 2147483648.0 => Some(level as i32),
                other => {
                    return Err(LuaError::runtime(format!(
                        "eech: bad argument: report damage_level must be an integer, got {}",
                        other.type_name()
                    )))
                }
            };
            let destroyed = match report.get::<LuaValue>("destroyed")? {
                LuaValue::Nil => false,
                LuaValue::Boolean(destroyed) => destroyed,
                other => {
                    return Err(LuaError::runtime(format!(
                        "eech: bad argument: report destroyed must be a boolean, got {}",
                        other.type_name()
                    )))
                }
            };
            if damage_level.is_none() && !destroyed {
                return Err(LuaError::runtime("eech: bad argument: a report states a damage_level or destroyed = true"));
            }
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
            let receipt = this
                .engine
                .report_object(
                    &ObjectReport {
                        host_id,
                        damage_level,
                        destroyed,
                    },
                    &mut binder,
                    &mut observer,
                    &mut reach,
                    &mut mover,
                    &mut sighter,
                    &mut sensor,
                    &mut aimer,
                )
                .map_err(lua_error)?;
            lua.to_value_with(&receipt, options())
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

/// TEST-ONLY (feature `model-fixture`, EEDCS-50): a Lua number that must be a whole number within `min..=max`. A missing, non-numeric, non-finite, fractional or
/// out-of-range value is refused here, before the engine is entered.
#[cfg(any(feature = "model-fixture", feature = "model-host-fixture"))]
fn model_int(value: &LuaValue, name: &str, min: i32, max: i32) -> LuaResult<i32> {
    let number = match value {
        LuaValue::Integer(i) => *i as f64,
        LuaValue::Number(n) => *n,
        other => {
            return Err(LuaError::runtime(format!(
                "eech: bad argument: {name} must be a number, got {}",
                other.type_name()
            )))
        }
    };
    if !number.is_finite() || number.fract() != 0.0 || number < f64::from(min) || number > f64::from(max) {
        return Err(LuaError::runtime(format!(
            "eech: bad argument: {name} must be a whole number within {min}..={max}"
        )));
    }
    Ok(number as i32)
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

// ---------------------------------------------------------------------------------------------------------------------
// the host's model provider (EEDCS-50, D3; aphavoc/.../mb_hostmodel.h)

/// The host's model provider as the campaign calls it: `model (op, scene, depth)`, with `op` one of
///
///   "catalogue"   no scene, no depth.  Returns `{ scenes = N }`: how many scene indices the host's models cover.
///   "scene"       a scene index.  Returns `{ name = text, loaded = bool, collider = bool, radius = f, bb = {six}, bb2 = {six} }`: the scene's compiled name (an echo that
///                 host and campaign mean the same scene by the same index; empty for the invalid index 0), whether the host has a model for it, whether that model has a
///                 collider (only a loaded scene has one), the radius, the bounds `xmin xmax ymin ymax zmin zmax` and the bounds without lines (the ones the collision
///                 tests use).  Nothing of the legacy loader's private object numbering is part of it.
///   "triangle"    a scene index and a depth (never negative).  Returns `{ result = "triangle", p = {nine} }` (point 0 x y z, point 1, point 2), `{ result = "end" }` (the
///                 scene has a collider and the list is over at this depth), `{ result = "absent" }` (the scene is loaded and has no collider) or
///                 `{ result = "refused", reason = text }` (the host cannot answer for this scene or depth).
///
///   "route_node"  a scene index, a cursor (`depth`, never negative) and a sub-object kind (`kind`, a whole number: the fourth argument of the provider). Returns the
///                 node cursor of the ROUTE the scene files under that kind: `{ result = "node", p = {three}, name = text }` (the node's decoded x y z), `{ result = "end", name }`
///                 (the route has no node at this cursor), `{ result = "absent", name }` (the host has no route of that kind for the scene) or `{ result = "refused", reason }`.
///   "route_face"  the same three numbers. Returns the ORDERED line-face cursor: `{ result = "face", point0 = n, point1 = n, red = n, green = n, blue = n, polygons = bool, name }`
///                 (the two node numbers the face joins, already resolved, and the colour 0..255 and polygon bit of its surface), `end`, `absent` or `refused` as above. `name` is the
///                 kind's compiled name (an echo that host and campaign mean the same kind by the same number); the campaign checks it.
///
///   "target_point_offset"  a scene index (never the invalid index 0; no depth). Returns `{ result = "offset", p = {three}, name = text }`: the scene's LOCAL target-point offset, the
///                 three coordinates the campaign's initialiser used to compute by constructing the scene (the TARGET_POINT sub-object's position, or the bounding-box centre with y
///                 clamped at zero; a scene no file defines is a valid slot and answers the zero offset), or `{ result = "refused", reason }`. `name` is the scene's compiled name (an echo).
///                 There is no `end` or `absent` answer: the campaign copies the three coordinates and asks again at every initialiser invocation.
///
///   "target_point_chain"  a scene index (the instance's scene; 0 is asked too). Returns `{ result = "chain", offsets = {one to eight whole numbers 0..65535}, name = text }`: the position of the
///                 scene's first TARGET_POINT sub-object among its siblings at each level, topmost level first (the host names the chain, the campaign does not search), or `{ result = "absent", name }`
///                 when the scene has none, or `{ result = "refused", reason }`.
///   "target_point_world"  a scene index and, in the sixth place (the text), the campaign's CURRENT placement as `15 + 9 * levels` comma-separated bit patterns: the instance's world position
///                 (3), attitude (9, row-major) and relative scale (3), then for each level of the chain just named the live relative position (3), heading, pitch, roll and relative scale (3).
///                 Returns `{ result = "world", p = {three}, name = text }`: the world position of that sub-object, or `{ result = "refused", reason }` (a level count that is not the chain's is a refusal).
///   "target_point_fixed"  a shape index (never 0) and, in the sixth place, three bit patterns (the entity position the original copied) and, only when the cached host offset has a horizontal
///                 part, a fourth (the heading). Returns `{ result = "fixed", p = {three}, name = text }` or `{ result = "refused", reason }` (a heading needed and not given is a refusal).
///
///   "entity_collision"  a scene index (never 0) and, in the sixth place (the text), 18 (a STATIONARY candidate) or 22 (a MOVING one) comma-separated bit patterns: the segment's old end (3)
///                 and new end (3), then the candidate's position (3) and attitude (9, row-major), or its old position (3), new position (3), attitude (9) and speed (1). The caller's
///                 intercept point and normal are OUTPUTS of the original and do not cross. Returns `{ result = "hit" or "miss", p = {six}, mask = n, name = text }`: whether the segment meets the
///                 scene's collision triangles (its bounding box without lines when it has none) at that pose; `mask` (0..63: bits 0 to 2 the intercept point, bits 3 to 5 the normal) says which
///                 components the original's arithmetic assigned (it assigns the whole normal first and the whole point only after a plane crossing, so a miss can have assigned the normal or
///                 both) and `p` holds their computed words as bit patterns (zero for the others), or `{ result = "refused", reason }`. `name` is the scene's compiled name (an echo).
///
///   "entity_candidate"  what the host decides about ONE candidate of the entity scan (EEDCS-49, D3, third slice): a scene index (0 until the campaign has read the candidate's shape: a
///                 candidate skipped before that, a city and a city building have none) and, in the sixth place (the text), 44 comma-separated bit patterns, the FACTS the campaign has read
///                 from the entity it is visiting, in this order: the segment's old end (3) and new end (3), the collision-test-mobile flag, the collision-test-fixed flag, the entity is a
///                 city, the entity is a city building, the candidate is a building listed under a city whose box was met, the city has a first building (each exactly 0 or 1), the sector's
///                 tallest structure height, the position (3), the motion vector (3), the frame time, the UPDATED answer (0 not asked, 1 false, 2 true), the heading, the align-with-terrain
///                 flag, the terrain face normal given (0 or 1), that normal (3), the attitude (9, row-major), the city's bounding box min (3) and max (3), the fixed facts were read (0 or 1)
///                 and the placement facts were read (0 or 1). FOR A MOBILE candidate of a sector list (never a building listed under a city) the text carries, after the 44 patterns, one space and the candidate's semantic HOST IDENTITY (1 to 128 bytes): the host decides the
///                 candidate's position, motion vector and attitude from ITS OWN canonical record of that identity and never from facts 13 to 18 and 27 to 35, and refuses by name a mobile candidate it holds no record of. The facts are STAGED: the host asks for what a branch needs only when the original branch reached it. Returns `{ result, name = text }`
///                 and, for `test`, `p`: `result` is `skip` (nothing more to do for the candidate), `need_updated` (the campaign performs its UPDATED getter, which has an effect of its own, and
///                 asks again with fact 20 filled in), `need_fixed` (the campaign reads the fixed flag and what the fixed branch reads, and asks again with facts 7 to 9, 11, the scene, the
///                 position, the city box and fact 42 filled in), `need_placement` (the campaign reads the attitude of a mobile candidate, or the heading and align flag of a fixed one, and
///                 asks again with fact 43 filled in), `need_normal` (the campaign asks the terrain for the face normal at the candidate's position and asks again with facts 23 to 26 filled in),
///                 `buildings` (a city's box is met; its buildings are offered one by one in the campaign's list order), `test` with `p` the 12 (stationary: position (3), attitude (9)) or 16
///                 (moving: old position (3), new position (3), attitude (9), speed) floats of the narrow phase's pose, which the campaign then sends with `entity_collision`, or `refused` with
///                 `reason` (a scene the branch needs and cannot use is refused here, never defaulted). `name` is the scene's compiled name (an echo; empty for scene 0).
///
///   "entity_scan"  the ENUMERATION of the entity scan (EEDCS-49, D3, fourth slice): scene 0 and, in the sixth place (the text), 16 comma-separated bit patterns, the FACTS: the segment's old end (3) and new
///                 end (3), the sector side length, the world map's min x, min z, max x and max z, the PHASE (0 begin, 1 an entry is served, 2 the cursor's list has no more entries, 3 the offered entry's
///                 decisions are finished), the cursor sector x and z the host last gave, the served entry is the source or the target (0 or 1) and the offered entry hit (0 or 1). The host decides the sector window,
///                 the traversal (x outer, z inner, each list from its TAIL), the exclusions and the first-hit end, and (EEDCS-49, sixth slice) WHICH entities stand in each sector and in what order: the host's
///                 own roster, which the campaign keeps informed with events of this same question. After the 16 patterns the text may carry a space and an identity: the key of the entry the campaign is standing at
///                 (the phases 1 and 3), a roster event's subject (below), or a hosted mobile's semantic host identity. The roster events are the phase fact 4 (an entity was inserted into a sector's list: facts 0 to 2
///                 the position, fact 3 the entity's type, fact 6 the sector side length; answered `placed` with the key the host issued as `name`), 5 (an entity leaves its list, the identity naming its key;
///                 answered `removed`) and 6 (a new boot; answered `cleared`). Returns `{ result }` and, for `first`, `p` (two bit patterns: the sector x and z, whole numbers): `first` (serve the TAIL of that
///                 sector's list: its key is `name`, empty for an empty list), `next` (serve what follows the entry the campaign is standing at: `name` is its key, empty at the end of the list), `offer` (offer the
///                 current entry for its decisions and report whether it hit), `done_none` (the scan ends, no hit), `done_hit` (it ends with the entry offered last), `placed`, `removed`, `cleared`, or `refused` with
///                 `reason` (an ended, unknown or misplaced key is refused by name, never skipped and never the end of a list). `name` is a decimal key of at most 20 digits and only `first`, `next` and `placed`
///                 carry one, and so does a `removed` that names the record a city's box keeps (phases 5 and 8 alone: a roster key, and for a child removal the list it was asked about). Phase 15 (a city's bounding-box expansion, Task49: the identity place is the city's sector key, one space and the building's key, every other fact zero) is answered `expanded` with `p` the six bit patterns of the box the host built (minimum then maximum) and `name` the city's key.
///
///   "entity_pose"  an OBSERVATION of a mobile candidate's pose (EEDCS-49, D3, fifth slice): scene 0, `depth` the stage (0 the offer: the position and motion vector the original read for every mobile
///                 candidate; 1 the placement: those and the attitude, read when the broad phase passed), `kind` the entity's type, `definition` whether it is alive (0 or 1) and, in the sixth place (the text),
///                 15 comma-separated bit patterns (the position (3), the motion vector (3), the attitude (9), zero for the words of a stage that was not read), then, when the roster has an identity for the
///                 candidate, a space and its semantic HOST IDENTITY (the text of the identity `bind_mobile` issued: the roster's whole domain of 1 to 128 bytes). Returns `{ result, kind, difference }`: `result` is `recorded` (the host keeps a movement
///                 record of the identity), `unrecorded` (it issued the identity to a member it keeps no record for), `retired`, `unknown` (it never issued it, or keeps no movers) or `no_id` (the candidate had
///                 none), `kind` the record's kind (0 none, 1 routed vehicle, 2 ship, 3 person, 4 fixed-wing aircraft, 5 helicopter) and `difference` the bits (0..31) of how the words compare with the record:
///                 1 position, 2 attitude (the placement stage only), 4 motion against a held vector (never together with 8), 8 no held motion vector, 16 motion not bit for bit the product, rounded toward zero and computed in integers, of the record's attitude row 2 and its speed (also set when an operand is not finite). The campaign refuses by marker an offer-stage reply with bit 2 and any reply with bits 4 and 8. NOTHING DECIDES FROM THE ANSWER: it is
///                 recorded and its shape is checked, and a candidate is offered, tested and skipped exactly as without it. `refused` stops the engine with the reason.
///
///   "object_table"  no scene, no depth. Returns `{ result = "table", objects = n }`: how many objects the host's common database holds (the campaign's room for custom objects is its own).
///   "object"     a scene index, a cursor (`depth`, never negative: the approximation or the sub-object) and, in the fourth place, the ROLE the object has in the scene (0 main, 1
///                 approximation, 2 shadow polygon object, 3 collision object, 4 sub-object). Returns `{ result = "object", name = text, counts = {six whole numbers within 0..65535:
///                 points, faces, surfaces, point normals, lighting normals, culling normals offset}, radius = f, bb = {six}, bb2 = {six} }`, `{ result = "absent", name }` (the
///                 files name no object in that role) or `{ result = "refused", reason }`. `name` is the scene's compiled name (the echo). No object number crosses either way.
///   "custom_object"  the same question about the scene a custom scene file defines (the campaign reads the database's record of a scene and later the custom file that replaces it);
///                 "object" is always the database's record. A scene no custom file defines is `refused`.
///   "animation"  a scene index and a cursor (`depth`). Returns the ORDERED animated-surface references of an .EES scene: `{ result = "name", animation = text, name }` (the LOGICAL
///                 animation name; no file index is part of the contract), `end`, `absent` (the scene is not an .EES scene) or `refused`.
///
/// A float is a finite Lua number (rounded to the nearest single-precision value) or the exact bit pattern as text (`"0x3f800000"`, ten bytes). A list of six or nine floats is
/// EITHER a table holding exactly the numbers 1..N (a number at every index, nothing at N+1 and no key outside 1..N) OR a text of exactly N bit patterns separated by single
/// commas (`11 * N - 1` bytes); anything else is refused before any of it is parsed or allocated. "end" and "absent" are the original accessor's FALSE and are not errors; a
/// "refused" answer, an error raised by the provider, a malformed answer, a non-finite number and an answer of the wrong kind stop the engine with the provider's marker
/// `[model-provider:<marker>]` in the message. The provider is called synchronously and its answers are never cached here.
///
/// EXACT NUMBERS. mlua turns any Lua number within `f64::EPSILON` of an integer into an integer (the patched Lua 5.1 `lua_isinteger`), so a tiny or subnormal value read as a
/// `LuaValue` would arrive as 0 and a negative zero would lose its sign. No physical number, and no whole number the campaign checks, is therefore read as a `LuaValue`: the
/// bridge below is a Lua function that reads every field of the answer ONCE (a stateful getter is not asked twice), checks its raw Lua type, and hands Rust text: the number
/// formatted at 17 significant digits (an exact image of the double, -0 and subnormals included) which `nearest_f32` narrows, or the bit-pattern text. This is the protected
/// pattern `exact_numbers` uses for the poses: nothing longjmps across Rust. WHAT IS AND IS NOT CLAIMED about floating point: the numbers the bridge hands back are decoded by a
/// correctly rounded `f64` parse and narrowed by `nearest_f32`, which takes the nearest single-precision value of the three candidates around the cast's guess and lets the cast's own
/// result win an exact tie (so a zero keeps its sign). The supported boundary is the engine's callback, which is ALWAYS entered under round-toward-zero, and the numeric tests run under
/// round-toward-zero; under that mode an exact midpoint between two singles resolves to the one nearer zero. NO claim is made that the result is the same under another rounding mode:
/// the cast's guess at an exact tie differs between them (for example the midpoint 1 + 3 * 2^-24 lies between two singles and the guess is the lower under round-toward-zero and the upper
/// under round-upward), and `nearest_f32` is deliberately unchanged. The unit-test helper checks the ROUNDING MODE only (`fegetround`). The provider code (Lua, libc formatting, the parse)
/// may raise exception flags and is not claimed to leave the x87 or MXCSR words untouched; the caller's COMPLETE state (control and status words, MXCSR) is what the engine entry
/// restores, and that is measured at the public entries by the diagnostic build's `eech_engine_model_host_fpu`, not asserted here.
const MODEL_BRIDGE: &str = r#"
local provider = ...
local floor, huge, format, concat, next, type, find = math.floor, math.huge, string.format, table.concat, next, type, string.find

local function finite(x) return type(x) == 'number' and x == x and x ~= huge and x ~= -huge end

-- one scalar: a finite number, or exactly one bit pattern
local function scalar(v)
	if type(v) == 'number' then
		if not finite(v) then return nil, 'is not finite' end
		return 'n:' .. format('%.17g', v)
	elseif type(v) == 'string' then
		if #v ~= 10 then return nil, 'text must be one bit pattern, 0x and eight hexadecimal digits' end
		return 's:' .. v
	end
	return nil, 'must be a number or a bit pattern, got ' .. type(v)
end

-- exactly n floats: a table of the numbers 1..n and nothing else, or n bit patterns separated by single commas
local function list(v, n)
	local kind = type(v)
	if kind == 'string' then
		if #v ~= 11 * n - 1 then return nil, 'text must be exactly ' .. n .. ' bit patterns separated by commas' end
		return 's:' .. v
	elseif kind == 'table' then
		local out = {}
		for i = 1, n do
			local x = v[i]
			if not finite(x) then return nil, 'element ' .. i .. ' is not a finite number' end
			out[i] = format('%.17g', x)
		end
		if v[n + 1] ~= nil then return nil, 'has more than ' .. n .. ' numbers' end
		local seen = 0
		for k in next, v do
			seen = seen + 1
			if seen > n or type(k) ~= 'number' or k < 1 or k > n or k ~= floor(k) then return nil, 'has a key outside 1..' .. n end
		end
		return 'n:' .. concat(out, ' ')
	end
	return nil, 'must be a table of ' .. n .. ' numbers or a text of ' .. n .. ' bit patterns, got ' .. kind
end

local function whole(v)
	if not finite(v) or v ~= floor(v) or v < 0 or v > 2147483647 then return nil end
	return format('%d', v)
end

-- a node number or a colour: a whole number within 0..max, as text
local function ranged(v, max)
	if not finite(v) or v ~= floor(v) or v < 0 or v > max then return nil end
	return format('%d', v)
end

return function(op, scene, depth, kind, definition, key)
	local a = provider(op, scene, depth, kind, definition, key)
	if type(a) ~= 'table' then return nil, 'the provider returned ' .. type(a) .. ' instead of a table' end
	if op == 'custom_order' then
		local result = a.result
		if result == 'found' or result == 'absent' or result == 'end' then return result end
		if result == 'refused' then
			local reason = a.reason
			return 'refused', type(reason) == 'string' and reason:sub(1, 200) or ''
		end
		return nil, 'result must be found, absent, end or refused'
	end
	if op == 'object_table' then
		if a.result ~= 'table' then return nil, 'result must be table' end
		local objects = ranged(a.objects, 2147483647)
		if not objects then return nil, 'objects must be a whole number within 0..2147483647' end
		return 'object_table', objects
	end
	if op == 'object' or op == 'custom_object' or op == 'animation' then
		local name, result = a.name, a.result
		if type(name) ~= 'string' or #name > 127 or find(name, '\0', 1, true) then return nil, 'name must be text of at most 127 bytes without a NUL' end
		if result == 'refused' then
			local reason = a.reason
			return 'refused', type(reason) == 'string' and reason:sub(1, 200) or ''
		end
		if op ~= 'animation' then
			if result == 'object' then
				local counts = a.counts
				if type(counts) ~= 'table' or counts[7] ~= nil then return nil, 'counts must be a table of six whole numbers' end
				local parts = {}
				for i = 1, 6 do
					parts[i] = ranged(counts[i], 65535)
					if not parts[i] then return nil, 'counts element ' .. i .. ' must be a whole number within 0..65535' end
				end
				local r, why = scalar(a.radius)
				if not r then return nil, 'radius ' .. why end
				local b, why1 = list(a.bb, 6)
				if not b then return nil, 'bb ' .. why1 end
				local b2, why2 = list(a.bb2, 6)
				if not b2 then return nil, 'bb2 ' .. why2 end
				return 'object', concat(parts, ' '), r, b, b2, nil, nil, name
			elseif result == 'absent' then
				return 'absent', nil, nil, nil, nil, nil, nil, name
			end
			return nil, 'result must be object, absent or refused'
		end
		if result == 'name' then
			local animation = a.animation
			if type(animation) ~= 'string' or #animation < 1 or #animation > 127 or find(animation, '\0', 1, true) then return nil, 'animation must be text of 1 to 127 bytes without a NUL' end
			return 'name', animation, nil, nil, nil, nil, nil, name
		elseif result == 'end' or result == 'absent' then
			return result, nil, nil, nil, nil, nil, nil, name
		end
		return nil, 'result must be name, end, absent or refused'
	end
	if op == 'target_point_offset' then
		local name, result = a.name, a.result
		if result == 'refused' then
			local reason = a.reason
			return 'refused', type(reason) == 'string' and reason:sub(1, 200) or ''
		end
		if type(name) ~= 'string' or #name > 127 or find(name, '\0', 1, true) then return nil, 'name must be text of at most 127 bytes without a NUL' end
		if result ~= 'offset' then return nil, 'result must be offset or refused' end
		local p, why = list(a.p, 3)
		if not p then return nil, 'p ' .. why end
		return 'offset', p, nil, nil, nil, nil, nil, name
	end
	if op == 'target_point_chain' then
		local name, result = a.name, a.result
		if result == 'refused' then
			local reason = a.reason
			return 'refused', type(reason) == 'string' and reason:sub(1, 200) or ''
		end
		if type(name) ~= 'string' or #name > 127 or find(name, '\0', 1, true) then return nil, 'name must be text of at most 127 bytes without a NUL' end
		if result == 'absent' then return 'absent', nil, nil, nil, nil, nil, nil, name end
		if result ~= 'chain' then return nil, 'result must be chain, absent or refused' end
		local offsets = a.offsets
		if type(offsets) ~= 'table' then return nil, 'offsets must be a table of one to eight whole numbers' end
		local parts, seen = {}, 0
		for i = 1, 8 do
			if offsets[i] == nil then break end
			parts[i] = ranged(offsets[i], 65535)
			if not parts[i] then return nil, 'offsets element ' .. i .. ' must be a whole number within 0..65535' end
			seen = seen + 1
		end
		if seen < 1 then return nil, 'offsets must have one to eight elements' end
		local keys = 0
		for k in next, offsets do
			keys = keys + 1
			if keys > seen or type(k) ~= 'number' or k < 1 or k > seen or k ~= floor(k) then return nil, 'offsets has a key outside 1..' .. seen end
		end
		return 'chain', concat(parts, ' '), nil, nil, nil, nil, nil, name
	end
	if op == 'entity_collision' then
		local name, result = a.name, a.result
		if result == 'refused' then
			local reason = a.reason
			return 'refused', type(reason) == 'string' and reason:sub(1, 200) or ''
		end
		if type(name) ~= 'string' or #name > 127 or find(name, ' ', 1, true) then return nil, 'name must be text of at most 127 bytes without a NUL' end
		if result ~= 'hit' and result ~= 'miss' then return nil, 'result must be hit, miss or refused' end
		local p, why = list(a.p, 6)
		if not p then return nil, 'p ' .. why end
		local mask = ranged(a.mask, 63)
		if not mask then return nil, 'mask must be a whole number within 0..63' end
		return result, p, mask, nil, nil, nil, nil, name
	end
	if op == 'entity_candidate' then
		local name, result = a.name, a.result
		if result == 'refused' then
			local reason = a.reason
			return 'refused', type(reason) == 'string' and reason:sub(1, 200) or ''
		end
		if type(name) ~= 'string' or #name > 127 or find(name, ' ', 1, true) then return nil, 'name must be text of at most 127 bytes without a NUL' end
		if result == 'skip' or result == 'need_updated' or result == 'need_normal' or result == 'buildings' or result == 'need_fixed' or result == 'need_placement' then
			if a.p ~= nil then return nil, 'p is only for test' end
			return result, nil, nil, nil, nil, nil, nil, name
		end
		if result ~= 'test' then return nil, 'result must be skip, need_updated, need_normal, need_fixed, need_placement, buildings, test or refused' end
		local given = a.p
		local count = type(given) == 'string' and (#given + 1) / 11 or type(given) == 'table' and #given or nil
		if count ~= 12 and count ~= 16 then return nil, 'p must be 12 or 16 floats' end
		local p, why = list(given, count)
		if not p then return nil, 'p ' .. why end
		return 'test', p, nil, nil, nil, nil, nil, name
	end
	if op == 'entity_pose' then
		local result = a.result
		if result == 'refused' then
			local reason = a.reason
			return 'refused', type(reason) == 'string' and reason:sub(1, 200) or ''
		end
		if result ~= 'recorded' and result ~= 'unrecorded' and result ~= 'retired' and result ~= 'unknown' and result ~= 'no_id' then
			return nil, 'result must be recorded, unrecorded, retired, unknown, no_id or refused'
		end
		local record_kind = ranged(a.kind, 5)
		if not record_kind then return nil, 'kind must be a whole number within 0..5' end
		local difference = ranged(a.difference, 31)
		if not difference then return nil, 'difference must be a whole number within 0..31' end
		return result, record_kind, difference
	end
	if op == 'entity_scan' then
		local result = a.result
		if result == 'refused' then
			local reason = a.reason
			return 'refused', type(reason) == 'string' and reason:sub(1, 200) or ''
		end
		local key = a.name
		if key == nil then key = '' end
		if type(key) ~= 'string' or #key > 127 or find(key, ' ', 1, true) or find(key, ' ', 1, true) then return nil, 'name must be text of at most 127 bytes without a NUL or a space' end
		if result == 'first' then
			local p, why = list(a.p, 2)
			if not p then return nil, 'p ' .. why end
			return 'first', p, nil, nil, nil, nil, nil, key
		end
		if result == 'expanded' then
			local p, why = list(a.p, 6)
			if not p then return nil, 'p ' .. why end
			return 'expanded', p, nil, nil, nil, nil, nil, key
		end
		-- a sector's high-water height (phases 17 and 18): the sector's x, z and the word, as three BIT PATTERNS only (a number would be a float, and the word is raw: never a number here), and no placement name
		if result == 'height' or result == 'restored' then
			if type(a.p) ~= 'string' then return nil, 'p must be three bit patterns, the sector x, z and the word' end
			local p, why = list(a.p, 3)
			if not p then return nil, 'p ' .. why end
			if key ~= '' then return nil, 'name is only for first, next and placed' end
			return result, p, nil, nil, nil, nil, nil, key
		end
		if result == 'next' or result == 'placed' then
			if a.p ~= nil then return nil, 'p is only for first' end
			return result, nil, nil, nil, nil, nil, nil, key
		end
		-- a removal of a city's placement or of its last building may name the record the host keeps for the box it built: the answer carries it as its name (the Rust side checks the phase and the relation to the question)
		if result == 'removed' then
			if a.p ~= nil then return nil, 'p is only for first' end
			return 'removed', nil, nil, nil, nil, nil, nil, key
		end
		if result == 'offer' or result == 'done_none' or result == 'done_hit' or result == 'cleared' then
			if a.p ~= nil then return nil, 'p is only for first' end
			if key ~= '' then return nil, 'name is only for first, next and placed' end
			return result
		end
		return nil, 'result must be first, expanded, height, restored, next, offer, done_none, done_hit, placed, removed, cleared or refused'
	end
	if op == 'target_point_world' or op == 'target_point_fixed' then
		local name, result = a.name, a.result
		if result == 'refused' then
			local reason = a.reason
			return 'refused', type(reason) == 'string' and reason:sub(1, 200) or ''
		end
		if type(name) ~= 'string' or #name > 127 or find(name, '\0', 1, true) then return nil, 'name must be text of at most 127 bytes without a NUL' end
		local expected = op == 'target_point_world' and 'world' or 'fixed'
		if result ~= expected then return nil, 'result must be ' .. expected .. ' or refused' end
		local p, why = list(a.p, 3)
		if not p then return nil, 'p ' .. why end
		return expected, p, nil, nil, nil, nil, nil, name
	end
	if op == 'route_node' or op == 'route_face' then
		local name, result = a.name, a.result
		if type(name) ~= 'string' or #name > 127 or find(name, '\0', 1, true) then return nil, 'name must be text of at most 127 bytes without a NUL' end
		if result == 'node' and op == 'route_node' then
			local p, why = list(a.p, 3)
			if not p then return nil, 'p ' .. why end
			return 'node', p, nil, nil, nil, nil, nil, name
		elseif result == 'face' and op == 'route_face' then
			local point0, point1 = ranged(a.point0, 65535), ranged(a.point1, 65535)
			local red, green, blue = ranged(a.red, 255), ranged(a.green, 255), ranged(a.blue, 255)
			if not point0 or not point1 then return nil, 'point0 and point1 must be whole numbers within 0..65535' end
			if not red or not green or not blue then return nil, 'red, green and blue must be whole numbers within 0..255' end
			if type(a.polygons) ~= 'boolean' then return nil, 'polygons must be a boolean' end
			-- the values after the tag: both node numbers, red, green, then blue and the polygon bit together as "<blue>:<0 or 1>" (the fifth text)
			return 'face', point0, point1, red, green, blue .. (a.polygons and ':1' or ':0'), nil, name
		elseif result == 'end' or result == 'absent' then
			return result, nil, nil, nil, nil, nil, nil, name
		elseif result == 'refused' then
			local reason = a.reason
			return 'refused', type(reason) == 'string' and reason:sub(1, 200) or ''
		end
		return nil, 'result must be node or face (as asked), end, absent or refused'
	end
	if op == 'catalogue' then
		local scenes = whole(a.scenes)
		if not scenes then return nil, 'scenes must be a whole number within 0..2147483647' end
		return 'catalogue', scenes
	elseif op == 'scene' then
		local name, loaded, collider, radius, bb, bb2 = a.name, a.loaded, a.collider, a.radius, a.bb, a.bb2
		if type(name) ~= 'string' or #name > 127 or find(name, '\0', 1, true) then return nil, 'name must be text of at most 127 bytes without a NUL' end
		if type(loaded) ~= 'boolean' then return nil, 'loaded must be a boolean' end
		if type(collider) ~= 'boolean' then return nil, 'collider must be a boolean' end
		local r, why = scalar(radius)
		if not r then return nil, 'radius ' .. why end
		local b, why1 = list(bb, 6)
		if not b then return nil, 'bb ' .. why1 end
		local b2, why2 = list(bb2, 6)
		if not b2 then return nil, 'bb2 ' .. why2 end
		return 'scene', name, loaded and '1' or '0', collider and '1' or '0', r, b, b2
	end
	local result = a.result
	if result == 'triangle' then
		local p, why = list(a.p, 9)
		if not p then return nil, 'p ' .. why end
		return 'triangle', p
	elseif result == 'end' or result == 'absent' then
		return result
	elseif result == 'refused' then
		local reason = a.reason
		return 'refused', type(reason) == 'string' and reason:sub(1, 200) or ''
	end
	return nil, 'result must be triangle, end, absent or refused'
end
"#;

/// One bit pattern (`0x` and eight hexadecimal digits) as the finite float it names.
fn model_hex(text: &str, what: &str, operation: &str) -> Result<f32, String> {
    let number = model_bits(text, what, operation)?;
    if number.is_finite() {
        Ok(number)
    } else {
        Err(format!("[model-provider:{operation}.answer] {what}: not finite"))
    }
}

/// One bit pattern (`0x` and eight hexadecimal digits) as the float it names, whatever the pattern: a word the campaign handed to the host and the host gives back untouched (the intercept
/// point and normal of an entity collision question) is the caller's own, and the original left such a word as it found it, a non-finite one included.
fn model_bits(text: &str, what: &str, operation: &str) -> Result<f32, String> {
    let malformed = |why: &str| format!("[model-provider:{operation}.answer] {what}: {why}");
    let digits = text
        .strip_prefix("0x")
        .filter(|digits| digits.len() == 8 && digits.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| malformed("a bit pattern is 0x and eight hexadecimal digits"))?;
    let bits = u32::from_str_radix(digits, 16).map_err(|_| malformed("not a bit pattern"))?;
    Ok(f32::from_bits(bits))
}

/// Exactly N floats from the bridge's text for a list: `n:` and N numbers at 17 digits separated by spaces, or `s:` and N bit patterns separated by commas. Nothing is
/// collected: the N slots are filled one token at a time and a further token is an error, so no input can make this allocate more than N floats.
fn bridged_floats<const N: usize>(pair: &str, what: &str, operation: &str) -> Result<[f32; N], String> {
    bridged_list(pair, what, operation, false)
}

/// As `bridged_floats`, but every bit pattern of the `s:` form is accepted, the non-finite ones too (see `model_bits`). The `n:` form is numbers and stays finite.
fn bridged_words<const N: usize>(pair: &str, what: &str, operation: &str) -> Result<[f32; N], String> {
    bridged_list(pair, what, operation, true)
}

fn bridged_list<const N: usize>(pair: &str, what: &str, operation: &str, any_bits: bool) -> Result<[f32; N], String> {
    let malformed = |why: &str| format!("[model-provider:{operation}.answer] {what}: {why}");
    let mut out = [0.0_f32; N];
    match pair.split_once(':') {
        Some(("n", numbers)) => {
            let mut tokens = numbers.split(' ');
            for slot in out.iter_mut() {
                let value = tokens
                    .next()
                    .and_then(|token| token.parse::<f64>().ok())
                    .ok_or_else(|| malformed("the bridge's numbers are malformed"))?;
                *slot = nearest_f32(value).ok_or_else(|| malformed("a number is beyond the single-precision range"))?;
            }
            if tokens.next().is_some() {
                return Err(malformed("more numbers than expected"));
            }
        }
        Some(("s", patterns)) => {
            let mut parts = patterns.split(',');
            for slot in out.iter_mut() {
                let pattern = parts.next().ok_or_else(|| malformed("too few bit patterns"))?;
                *slot = if any_bits {
                    model_bits(pattern, what, operation)?
                } else {
                    model_hex(pattern, what, operation)?
                };
            }
            if parts.next().is_some() {
                return Err(malformed("more bit patterns than expected"));
            }
        }
        _ => return Err(malformed("the bridge's list is malformed")),
    }
    Ok(out)
}

/// One float from the bridge's text for a scalar (a list of one).
fn bridged_float(pair: &str, what: &str, operation: &str) -> Result<f32, String> {
    bridged_floats::<1>(pair, what, operation).map(|[value]| value)
}

/// The campaign's floats as the text the provider receives: eight-digit bit patterns separated by commas (a value is never rounded on its way to the host).
fn pose_text(pose: &[u32]) -> String {
    pose.iter().map(|bits| format!("0x{bits:08x}")).collect::<Vec<_>>().join(",")
}

/// The fact of an entity-scan question that carries its phase (`CAMPAIGN_SCAN_PHASE`), and the two phases whose removal the host may answer with the name of a record it keeps (a city's box that outlives its
/// sector placement): a roster REMOVE of the city's placement (5) and the CHILD_REMOVE of its last building (8). The release of the city (16, a private acknowledgement) and every other event name none.
const SCAN_PHASE_FACT: usize = 11;
const SCAN_PHASE_REMOVE: f32 = 5.0;
const SCAN_PHASE_CHILD_REMOVE: f32 = 8.0;

/// The name a removal carries is a record the host retains, legitimate only in its declared domain: a roster key (one to twenty decimal digits without a leading zero, as the campaign reads one), asked at the
/// phase of a REMOVE or a CHILD_REMOVE, and for a CHILD_REMOVE the key of the list the removal was asked about (the first word of its identity place, `<list> <child>`). The phase is the whole number the
/// campaign sent, compared bit for bit: a neighbouring float, a negative zero, a non-finite word or no phase at all is none of them. The campaign checks the same again before it keeps the name.
fn check_retained_record(facts: &[u32], identity: &str, key: &str, operation: &str) -> Result<(), String> {
    let phase = facts.get(SCAN_PHASE_FACT).copied();
    if phase != Some(SCAN_PHASE_REMOVE.to_bits()) && phase != Some(SCAN_PHASE_CHILD_REMOVE.to_bits()) {
        return Err(format!(
            "[model-provider:{operation}.answer] name is only for first, next, placed and the removal of a city's record at the phase of a roster removal or of a city child removal"
        ));
    }
    let is_roster_key = (1..=20).contains(&key.len()) && !key.starts_with('0') && key.bytes().all(|byte| byte.is_ascii_digit());
    if !is_roster_key {
        return Err(format!(
            "[model-provider:{operation}.answer] the retained record's name is not a roster key: one to twenty decimal digits without a leading zero"
        ));
    }
    if phase == Some(SCAN_PHASE_CHILD_REMOVE.to_bits()) && identity.split_once(' ').map(|(list, _)| list) != Some(key) {
        return Err(format!(
            "[model-provider:{operation}.answer] the record named for a city child removal names another list than the one it was asked about"
        ));
    }
    Ok(())
}

fn call_model(bridge: &LuaFunction, query: ModelQuery) -> Result<ModelAnswer, String> {
    let (operation, scene, depth, kind, definition, key) = match query.clone() {
        ModelQuery::Catalogue => ("catalogue", None, None, None, None, None),
        ModelQuery::Scene { scene } => ("scene", Some(scene), None, None, None, None),
        ModelQuery::Triangle { scene, depth } => ("triangle", Some(scene), Some(depth), None, None, None),
        ModelQuery::RouteNode { scene, kind, index } => ("route_node", Some(scene), Some(index), Some(kind), None, None),
        ModelQuery::RouteFace { scene, kind, index } => ("route_face", Some(scene), Some(index), Some(kind), None, None),
        ModelQuery::ObjectTable => ("object_table", None, None, None, None, None),
        ModelQuery::TargetPointOffset { scene } => ("target_point_offset", Some(scene), None, None, None, None),
        ModelQuery::TargetPointChain { scene } => ("target_point_chain", Some(scene), None, None, None, None),
        // the campaign's current placement crosses as exact bit patterns in the text place
        ModelQuery::TargetPointWorld { scene, pose } => ("target_point_world", Some(scene), None, None, None, Some(pose_text(&pose))),
        ModelQuery::TargetPointFixed { scene, pose } => ("target_point_fixed", Some(scene), None, None, None, Some(pose_text(&pose))),
        // the segment and the candidate's pose cross as exact bit patterns in the text place (18 floats: stationary, 22: moving); the caller's intercept point and normal are outputs and do not cross
        ModelQuery::EntityCollision { scene, pose } => ("entity_collision", Some(scene), None, None, None, Some(pose_text(&pose))),
        // the facts of one candidate of the entity scan cross as exact bit patterns in the text place (44 floats); the scene is 0 until the campaign has read the shape
        // a mobile candidate's semantic host identity follows the facts after one space (the facts are 44 fixed-width patterns); the host decides that candidate's pose from its own record of it
        ModelQuery::EntityCandidate { scene, facts, host_id } => (
            "entity_candidate",
            Some(scene),
            None,
            None,
            None,
            Some(if host_id.is_empty() {
                pose_text(&facts)
            } else {
                format!("{} {host_id}", pose_text(&facts))
            }),
        ),
        // the facts of the enumeration cross as exact bit patterns in the text place (16 floats); the scene is 0
        // the identity, when there is one, follows the facts after a single space: a key of the host's roster (the entry the campaign is standing at, or a removal) or a hosted mobile's semantic identity (an insertion)
        ModelQuery::EntityScan { facts, identity } => (
            "entity_scan",
            Some(0),
            None,
            None,
            None,
            Some(if identity.is_empty() {
                pose_text(&facts)
            } else {
                format!("{} {identity}", pose_text(&facts))
            }),
        ),
        // the words of an observation cross as exact bit patterns in the text place (15 floats), followed, when the candidate has one, by a space and its host identity; the stage, the entity's type and its
        // alive flag are the depth, kind and definition places
        ModelQuery::EntityPose {
            stage,
            entity_type,
            alive,
            host_id,
            words,
        } => (
            "entity_pose",
            Some(0),
            Some(stage),
            Some(entity_type),
            Some(alive),
            Some(if host_id.is_empty() {
                pose_text(&words)
            } else {
                format!("{} {host_id}", pose_text(&words))
            }),
        ),
        // the fourth place carries the role the object has in the scene
        ModelQuery::Object {
            scene,
            role,
            cursor,
            definition,
            custom,
        } => (
            if custom { "custom_object" } else { "object" },
            Some(scene),
            Some(cursor),
            Some(role),
            Some(definition),
            None,
        ),
        ModelQuery::Animation { scene, index, definition } => ("animation", Some(scene), Some(index), None, Some(definition), None),
        // the announcement of the order of the custom scene files: the role place says whether this step ends it
        ModelQuery::CustomOrder { scene, position, end, key } => ("custom_order", Some(scene), Some(position), Some(i32::from(end)), None, Some(key)),
    };
    type Reply = (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    );
    let (tag, a, b, c, d, e, f, g): Reply = bridge
        .call((operation, scene, depth, kind, definition, key))
        .map_err(|error| format!("[model-provider:{operation}.callback] {error}"))?;
    let Some(tag) = tag else {
        return Err(format!(
            "[model-provider:{operation}.answer] {}",
            a.unwrap_or_else(|| "the provider's answer is malformed".to_string())
        ));
    };
    let missing = || format!("[model-provider:{operation}.answer] the bridge returned an incomplete answer");
    match (query, tag.as_str()) {
        (ModelQuery::Catalogue, "catalogue") => {
            let scenes = a
                .ok_or_else(missing)?
                .parse::<i32>()
                .map_err(|_| format!("[model-provider:{operation}.answer] scenes is out of range"))?;
            Ok(ModelAnswer::Catalogue { scenes })
        }
        (ModelQuery::Scene { .. }, "scene") => Ok(ModelAnswer::Scene(ModelScene {
            name: a.ok_or_else(missing)?,
            loaded: b.ok_or_else(missing)? == "1",
            collider: c.ok_or_else(missing)? == "1",
            radius: bridged_float(&d.ok_or_else(missing)?, "radius", operation)?,
            bounding_box: bridged_floats::<6>(&e.ok_or_else(missing)?, "bb", operation)?,
            bounding_box_without_lines: bridged_floats::<6>(&f.ok_or_else(missing)?, "bb2", operation)?,
        })),
        (ModelQuery::Triangle { .. }, "triangle") => Ok(ModelAnswer::Triangle(ModelTriangle::Found(bridged_floats::<9>(
            &a.ok_or_else(missing)?,
            "p",
            operation,
        )?))),
        (ModelQuery::Triangle { .. }, "end") => Ok(ModelAnswer::Triangle(ModelTriangle::End)),
        (ModelQuery::Triangle { .. }, "absent") => Ok(ModelAnswer::Triangle(ModelTriangle::Absent)),
        (ModelQuery::Triangle { scene, depth }, "refused") => Err(format!(
            "[model-provider:triangle.refused] the host cannot answer for scene {scene} depth {depth}: {}",
            a.unwrap_or_default()
        )),
        (ModelQuery::RouteNode { .. }, "node") => Ok(ModelAnswer::RouteNode {
            kind_name: g.ok_or_else(missing)?,
            step: ModelRouteNode::Found(bridged_floats::<3>(&a.ok_or_else(missing)?, "p", operation)?),
        }),
        (ModelQuery::RouteNode { .. }, "end") => Ok(ModelAnswer::RouteNode {
            kind_name: g.ok_or_else(missing)?,
            step: ModelRouteNode::End,
        }),
        (ModelQuery::RouteNode { .. }, "absent") => Ok(ModelAnswer::RouteNode {
            kind_name: g.ok_or_else(missing)?,
            step: ModelRouteNode::Absent,
        }),
        (ModelQuery::RouteFace { .. }, "face") => {
            let number = |text: Option<String>, what: &str| -> Result<i32, String> {
                text.ok_or_else(missing)?
                    .parse::<i32>()
                    .map_err(|_| format!("[model-provider:{operation}.answer] {what} is out of range"))
            };
            // the fifth text carries the blue byte and the polygon bit as "<blue>:<0 or 1>"
            let tail = e.ok_or_else(missing)?;
            let (blue_text, polygons_text) = tail.split_once(':').ok_or_else(missing)?;
            let polygons = match polygons_text {
                "1" => true,
                "0" => false,
                _ => return Err(format!("[model-provider:{operation}.answer] polygons is not a boolean")),
            };
            let blue = blue_text
                .parse::<i32>()
                .map_err(|_| format!("[model-provider:{operation}.answer] blue is out of range"))?;
            Ok(ModelAnswer::RouteFace {
                kind_name: g.ok_or_else(missing)?,
                step: ModelRouteFace::Found {
                    point: [number(a, "point0")?, number(b, "point1")?],
                    colour: [number(c, "red")?, number(d, "green")?, blue],
                    polygons,
                },
            })
        }
        (ModelQuery::RouteFace { .. }, "end") => Ok(ModelAnswer::RouteFace {
            kind_name: g.ok_or_else(missing)?,
            step: ModelRouteFace::End,
        }),
        (ModelQuery::RouteFace { .. }, "absent") => Ok(ModelAnswer::RouteFace {
            kind_name: g.ok_or_else(missing)?,
            step: ModelRouteFace::Absent,
        }),
        (ModelQuery::CustomOrder { .. }, "found") => Ok(ModelAnswer::CustomOrder(ModelOrder::Accepted)),
        (ModelQuery::CustomOrder { .. }, "absent") => Ok(ModelAnswer::CustomOrder(ModelOrder::Absent)),
        (ModelQuery::CustomOrder { .. }, "end") => Ok(ModelAnswer::CustomOrder(ModelOrder::End)),
        (ModelQuery::CustomOrder { scene, position, key, .. }, "refused") => Err(format!(
            "[model-provider:custom_order.refused] the host cannot take the walk announcement (scene {scene}, position {position}, directory {key:?}): {}",
            a.unwrap_or_default()
        )),
        (ModelQuery::ObjectTable, "object_table") => {
            let count = |text: Option<String>, what: &str| -> Result<i32, String> {
                text.ok_or_else(missing)?
                    .parse::<i32>()
                    .map_err(|_| format!("[model-provider:{operation}.answer] {what} is out of range"))
            };
            Ok(ModelAnswer::ObjectTable { objects: count(a, "objects")? })
        }
        (ModelQuery::Object { .. }, "object") => {
            let mut counts = [0_i32; 6];
            let text = a.ok_or_else(missing)?;
            let mut tokens = text.split(' ');
            for slot in counts.iter_mut() {
                *slot = tokens
                    .next()
                    .and_then(|token| token.parse::<i32>().ok())
                    .ok_or_else(|| format!("[model-provider:{operation}.answer] the counts are malformed"))?;
            }
            if tokens.next().is_some() {
                return Err(format!("[model-provider:{operation}.answer] more counts than expected"));
            }
            Ok(ModelAnswer::Object {
                scene_name: g.ok_or_else(missing)?,
                step: ModelObject::Found(ModelObjectFacts {
                    counts,
                    radius: bridged_float(&b.ok_or_else(missing)?, "radius", operation)?,
                    bounding_box: bridged_floats::<6>(&c.ok_or_else(missing)?, "bb", operation)?,
                    bounding_box2: bridged_floats::<6>(&d.ok_or_else(missing)?, "bb2", operation)?,
                }),
            })
        }
        (ModelQuery::Object { .. }, "absent") => Ok(ModelAnswer::Object {
            scene_name: g.ok_or_else(missing)?,
            step: ModelObject::Absent,
        }),
        (ModelQuery::Object { scene, role, cursor, .. }, "refused") => Err(format!(
            "[model-provider:object.refused] the host cannot answer for scene {scene} role {role} cursor {cursor}: {}",
            a.unwrap_or_default()
        )),
        (ModelQuery::Animation { .. }, "name") => Ok(ModelAnswer::Animation {
            scene_name: g.ok_or_else(missing)?,
            step: ModelAnimation::Name(a.ok_or_else(missing)?),
        }),
        (ModelQuery::Animation { .. }, "end") => Ok(ModelAnswer::Animation {
            scene_name: g.ok_or_else(missing)?,
            step: ModelAnimation::End,
        }),
        (ModelQuery::Animation { .. }, "absent") => Ok(ModelAnswer::Animation {
            scene_name: g.ok_or_else(missing)?,
            step: ModelAnimation::Absent,
        }),
        (ModelQuery::Animation { scene, index, .. }, "refused") => Err(format!(
            "[model-provider:animation.refused] the host cannot answer for scene {scene} reference {index}: {}",
            a.unwrap_or_default()
        )),
        (ModelQuery::TargetPointOffset { .. }, "offset") => Ok(ModelAnswer::TargetPointOffset {
            scene_name: g.ok_or_else(missing)?,
            offset: bridged_floats::<3>(&a.ok_or_else(missing)?, "p", operation)?,
        }),
        (ModelQuery::TargetPointOffset { scene }, "refused") => Err(format!(
            "[model-provider:target_point_offset.refused] the host cannot answer for scene {scene}: {}",
            a.unwrap_or_default()
        )),
        (ModelQuery::TargetPointChain { .. }, "chain") => {
            // the bridge has already checked one to eight whole numbers; the count is bounded again here so that no input can make this allocate more
            let offsets = a
                .ok_or_else(missing)?
                .split(' ')
                .take(9)
                .map(|token| {
                    token
                        .parse::<i32>()
                        .map_err(|_| format!("[model-provider:{operation}.answer] offsets is out of range"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            if offsets.is_empty() || offsets.len() > 8 {
                return Err(format!("[model-provider:{operation}.answer] a chain has one to eight levels"));
            }
            Ok(ModelAnswer::TargetPointChain {
                scene_name: g.ok_or_else(missing)?,
                offsets: Some(offsets),
            })
        }
        (ModelQuery::TargetPointChain { .. }, "absent") => Ok(ModelAnswer::TargetPointChain {
            scene_name: g.ok_or_else(missing)?,
            offsets: None,
        }),
        (ModelQuery::EntityCollision { .. }, "hit" | "miss") => Ok(ModelAnswer::EntityCollision {
            scene_name: g.ok_or_else(missing)?,
            hit: tag == "hit",
            write_mask: b
                .ok_or_else(missing)?
                .parse::<u32>()
                .map_err(|_| format!("[model-provider:{operation}.answer] the write mask is out of range"))?,
            words: bridged_words::<6>(&a.ok_or_else(missing)?, "p", operation)?,
        }),
        (ModelQuery::EntityCandidate { .. }, "skip" | "need_updated" | "need_normal" | "need_fixed" | "need_placement" | "buildings") => {
            Ok(ModelAnswer::EntityCandidate {
                scene_name: g.ok_or_else(missing)?,
                decision: match tag.as_str() {
                    "skip" => CandidateDecision::Skip,
                    "need_updated" => CandidateDecision::NeedUpdated,
                    "need_normal" => CandidateDecision::NeedNormal,
                    "need_fixed" => CandidateDecision::NeedFixed,
                    "need_placement" => CandidateDecision::NeedPlacement,
                    _ => CandidateDecision::Buildings,
                },
            })
        }
        (ModelQuery::EntityScan { .. }, "expanded") => {
            let words = bridged_floats::<6>(&a.ok_or_else(missing)?, "p", operation)?;
            Ok(ModelAnswer::EntityScan {
                decision: ScanDecision::Expanded(words),
                key: g.unwrap_or_default(),
            })
        }
        (ModelQuery::EntityScan { .. }, "height" | "restored") => {
            let words = bridged_words::<3>(&a.ok_or_else(missing)?, "p", operation)?;
            Ok(ModelAnswer::EntityScan {
                decision: if tag == "height" {
                    ScanDecision::Height(words)
                } else {
                    ScanDecision::Restored(words)
                },
                key: g.unwrap_or_default(),
            })
        }
        (ModelQuery::EntityScan { .. }, "first") => {
            let cursor = bridged_floats::<2>(&a.ok_or_else(missing)?, "p", operation)?;
            Ok(ModelAnswer::EntityScan {
                decision: ScanDecision::First(cursor),
                key: g.unwrap_or_default(),
            })
        }
        (ModelQuery::EntityScan { facts, identity }, "removed") => {
            let key = g.unwrap_or_default();
            if !key.is_empty() {
                check_retained_record(&facts, &identity, &key, operation)?;
            }
            Ok(ModelAnswer::EntityScan {
                decision: ScanDecision::Removed,
                key,
            })
        }
        (ModelQuery::EntityScan { .. }, "next" | "offer" | "done_none" | "done_hit" | "placed" | "cleared") => Ok(ModelAnswer::EntityScan {
            decision: match tag.as_str() {
                "next" => ScanDecision::Next,
                "offer" => ScanDecision::Offer,
                "done_none" => ScanDecision::DoneNone,
                "placed" => ScanDecision::Placed,
                "cleared" => ScanDecision::Cleared,
                _ => ScanDecision::DoneHit,
            },
            key: g.unwrap_or_default(),
        }),
        (ModelQuery::EntityCandidate { .. }, "test") => {
            // the bridge has already checked that the list holds exactly 12 or 16 floats; the count is read from the text and bounded again here
            let text = a.ok_or_else(missing)?;
            let count = text.split_once(':').map_or(0, |(_, rest)| rest.split([' ', ',']).count());
            let pose = match count {
                12 => bridged_floats::<12>(&text, "p", operation)?.to_vec(),
                16 => bridged_floats::<16>(&text, "p", operation)?.to_vec(),
                other => return Err(format!("[model-provider:{operation}.answer] a candidate pose has 12 or 16 floats, not {other}")),
            };
            Ok(ModelAnswer::EntityCandidate {
                scene_name: g.ok_or_else(missing)?,
                decision: CandidateDecision::Test(pose),
            })
        }
        (ModelQuery::TargetPointWorld { .. }, "world") | (ModelQuery::TargetPointFixed { .. }, "fixed") => Ok(ModelAnswer::TargetPointPosition {
            scene_name: g.ok_or_else(missing)?,
            position: bridged_floats::<3>(&a.ok_or_else(missing)?, "p", operation)?,
        }),
        (ModelQuery::EntityPose { .. }, "recorded" | "unrecorded" | "retired" | "unknown" | "no_id") => {
            let number = |text: Option<String>, what: &str| -> Result<u32, String> {
                text.ok_or_else(missing)?
                    .parse::<u32>()
                    .map_err(|_| format!("[model-provider:{operation}.answer] {what} is out of range"))
            };
            Ok(ModelAnswer::EntityPose {
                class: match tag.as_str() {
                    "recorded" => PoseClass::Recorded,
                    "unrecorded" => PoseClass::Unrecorded,
                    "retired" => PoseClass::Retired,
                    "unknown" => PoseClass::Unknown,
                    _ => PoseClass::NoId,
                },
                kind: number(a, "kind")?,
                difference: number(b, "difference")?,
            })
        }
        (ModelQuery::EntityPose { .. }, "refused") => Err(format!(
            "[model-provider:{operation}.refused] the host cannot observe the candidate: {}",
            a.unwrap_or_default()
        )),
        (ModelQuery::EntityScan { .. }, "refused") => Err(format!(
            "[model-provider:{operation}.refused] the host cannot answer the entity scan: {}",
            a.unwrap_or_default()
        )),
        (
            ModelQuery::TargetPointChain { scene }
            | ModelQuery::TargetPointWorld { scene, .. }
            | ModelQuery::TargetPointFixed { scene, .. }
            | ModelQuery::EntityCollision { scene, .. }
            | ModelQuery::EntityCandidate { scene, .. },
            "refused",
        ) => Err(format!(
            "[model-provider:{operation}.refused] the host cannot answer for scene {scene}: {}",
            a.unwrap_or_default()
        )),
        (ModelQuery::RouteNode { scene, kind, index } | ModelQuery::RouteFace { scene, kind, index }, "refused") => Err(format!(
            "[model-provider:{operation}.refused] the host cannot answer for scene {scene} kind {kind} cursor {index}: {}",
            a.unwrap_or_default()
        )),
        (_, other) => Err(format!(
            "[model-provider:{operation}.answer] the bridge answered a {operation} question with {other:?}"
        )),
    }
}

/// The provider as the engine keeps it: the Lua bridge around the host's function, built once at boot (so nothing needs a Lua handle afterwards).
fn model_provider(lua: &Lua, function: LuaFunction) -> LuaResult<ModelProvider> {
    let build: LuaFunction = lua.load(MODEL_BRIDGE).set_name("eech model bridge").into_function()?;
    let bridge: LuaFunction = build.call(function)?;
    Ok(Box::new(move |query| call_model(&bridge, query)))
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
    // EEDCS-50 (D3): the host owns the physical models and the campaign has no model arrays of its own to fall back to, so the provider is required, and a boot without
    // one is refused here by name. (The original-characterisation build, feature model-fixture, keeps the original accessors on purpose, asks no host and installs none.)
    let model = match config.get::<LuaValue>("model")? {
        LuaValue::Function(function) => Some(model_provider(lua, function)?),
        LuaValue::Nil if cfg!(feature = "model-fixture") => None,
        LuaValue::Nil => return Err(LuaError::runtime(
            "eech: [model-provider:missing] boot was given no model provider; the host supplies the physical models (catalogue, scenes, collision triangles)",
        )),
        value => {
            return Err(LuaError::runtime(format!(
                "eech: [model-provider:missing] model must be a function, got {}",
                value.type_name()
            )))
        }
    };
    // EEDCS-47: the host owns the physical terrain and the campaign has no solver of its own to fall back to, so the provider is required, and is
    // refused here, before anything boots, with the stable marker a driver can tell from any other failure
    let terrain = match config.get::<LuaValue>("terrain")? {
        LuaValue::Function(function) => function,
        LuaValue::Nil => {
            return Err(LuaError::runtime(
                "eech: [terrain-provider:missing] boot was given no terrain provider; the host supplies the physical terrain (map facts, point surface, fast elevation)",
            ))
        }
        value => {
            return Err(LuaError::runtime(format!(
                "eech: [terrain-provider:missing] the terrain provider must be a function, got {}",
                value.type_name()
            )))
        }
    };
    let model = if cfg!(feature = "model-fixture") { None } else { model };
    // EEDCS-51 (D3): the host owns the physical weather and wind field and the campaign has no solver of its own to fall back to, so this provider is required too, and is refused
    // here, before anything boots, with the stable marker a driver can tell from any other failure
    let environment = match config.get::<LuaValue>("environment")? {
        LuaValue::Function(function) => function,
        LuaValue::Nil => {
            return Err(LuaError::runtime(
                "eech: [env-provider:missing] boot was given no environment provider; the host supplies the physical weather and wind field (its queries, its initialisation, its evolution and the lightning timer)",
            ))
        }
        value => {
            return Err(LuaError::runtime(format!(
                "eech: [env-provider:missing] the environment provider must be a function, got {}",
                value.type_name()
            )))
        }
    };
    // EEDCS-51 (D3): the host owns the physical sample metadata at initialisation (rates, and the sizes in its own sound.hdr / sound.dat). A provider that is not a function is refused here, before anything boots.
    let sound_data = sound_data_function(&config, &environment)?;
    // EEDCS-51 (D3): and the physical sound backend availability at initialisation, asked in the same boot behind the campaign's own flag test.
    let sound_backend = sound_backend_function(&config, &environment)?;
    let arguments: Vec<String> = config.get::<Option<Vec<String>>>("arguments")?.unwrap_or_default();
    let config = EngineConfig {
        install_root: PathBuf::from(config.get::<String>("install_root")?),
        map_path: config.get("map")?,
        campaign_directory: config.get::<Option<String>>("campaign_directory")?.unwrap_or_else(|| "camp01".into()),
        campaign_filename: config.get("campaign")?,
        gunship: gunship(&config.get::<Option<String>>("gunship")?.unwrap_or_else(|| "apache".into()))?,
        random_seed: config.get::<Option<u32>>("seed")?.unwrap_or(1),
        winter: config.get::<Option<bool>>("winter")?.unwrap_or(false),
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
        model,
        terrain_provider(lua, terrain)?,
        environment_provider(lua, environment)?,
        sound_catalogue_provider(lua, sound_data),
        sound_backend_provider(lua, sound_backend),
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
    // TEST-ONLY (feature `env-observer`, EEDCS-51): arms (true) or disarms (false) the read-only observer; arming is valid only before boot
    #[cfg(feature = "env-observer")]
    exports.set(
        "env_observer_arm",
        lua.create_function(|_, on: bool| eech_engine::env_observer_arm(on).map_err(lua_error))?,
    )?;
    // TEST-ONLY (feature `env-observer`, EEDCS-51, D3): arms (true) or disarms (false) the helicopter rotor sound probe, which nothing else arms; arming is valid only before boot
    #[cfg(feature = "env-observer")]
    exports.set(
        "env_rotor_arm",
        lua.create_function(|_, on: bool| eech_engine::env_rotor_arm(on).map_err(lua_error))?,
    )?;
    // TEST-ONLY (feature `env-observer`, EEDCS-51, D3): arms (true) or disarms (false) the resumed-parent playback probe, which nothing else arms; arming is valid only before boot
    #[cfg(feature = "env-observer")]
    exports.set(
        "env_play_arm",
        lua.create_function(|_, on: bool| eech_engine::env_play_arm(on).map_err(lua_error))?,
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
                target_host_id: Some("veh:5".into()),
            })
        };
        let answering: LuaFunction = lua
            .load(
                "return function(r) assert(r.mode == 'health' and r.op == 'timed' and r.critical_model == 1 and r.damage_level == 14 and r.initial_damage_level == 100 and r.explosive == true \
                 and r.flammable == false and r.delta_time == 0.5 and r.draw == 6553 and r.self_hit == false and r.target_host_id == 'veh:5')
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
                target_host_id: Some("site:9".into()),
            })
        };
        let planning: LuaFunction = lua
            .load(
                "return function(r) assert(r.mode == 'impact' and r.kill_code == 3 and r.has_target == true and r.target_alive == false and r.position[1] == 250 and r.position[3] == 750 and r.target_host_id == 'site:9')
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
                        host_id: Some("obj-8".into()),
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
    // The bridge question reaches the host's Lua with its facts by value and, for a fall, the terrain only once the host has asked for it; each op's answer is read field by field.
    #[test]
    fn a_bridge_question_reaches_lua_by_value_and_each_answer_is_read_field_by_field() -> Result<(), Box<dyn std::error::Error>> {
        use eech_engine::AimBridge;
        let lua = Lua::new();
        let request = |op: i32, terrain: Option<(f32, bool)>| {
            AimRequest::Bridge(AimBridge {
                op,
                segment_kind: 3,
                alive: true,
                server: true,
                pred_present: true,
                pred_alive: false,
                succ_present: false,
                succ_alive: false,
                which: 1,
                neighbour_present: true,
                neighbour_kind: 2,
                neighbour_alive: true,
                position: [250.0, 120.0, 250.0],
                delta_time: 0.1,
                terrain,
                segment_length: 10.0,
                sector_side_length: 100,
                min_x_sector: 0,
                max_x_sector: 9,
                min_z_sector: 0,
                max_z_sector: 9,
                window: None,
            })
        };
        let host: LuaFunction = lua
            .load(
                "return function(r) assert(r.mode == 'bridge' and r.segment_kind == 3 and r.alive == true and r.server == true and r.pred_present == true and r.pred_alive == false and r.which == 1 \
                 and r.neighbour_present == true and r.neighbour_kind == 2 and r.position[2] == 120 and r.segment_length == 10 and r.sector_side_length == 100 and r.window == nil)
                 if r.op == 'kill' then return { state = 'done', kill = true } end
                 if r.op == 'shape' then return { state = 'done', variant = 4 } end
                 if r.op == 'neighbour' then return { state = 'done', notify = true } end
                 if r.op == 'fall' then
                   if r.terrain == nil then return { state = 'need_lookup', need = 'terrain' } end
                   assert(r.terrain.height == 100 and r.terrain.water == true)
                   return { state = 'done', fell = true, landed = true, remove = true, dropped_y = 99.5, floor_height = 99, landed_y = 99 }
                 end
                 if r.op == 'rest' then
                   if r.terrain == nil then return { state = 'need_lookup', need = 'terrain' } end
                   return { state = 'done', landed = true, landed_y = r.terrain.height - 1 }
                 end
                 return { state = 'need_lookup', need = { sx = 2, sz = 2, after = 0 } } end",
            )
            .eval()?;
        assert!(call_aim(&lua, &host, request(1, None))?.bridge_kill);
        assert_eq!(call_aim(&lua, &host, request(2, None))?.bridge_variant, 4);
        assert!(call_aim(&lua, &host, request(3, None))?.bridge_notify);
        let asking = call_aim(&lua, &host, request(4, None))?;
        assert!(asking.need_lookup && asking.bridge_need_terrain);
        let fall = call_aim(&lua, &host, request(4, Some((100.0, true))))?;
        assert_eq!(
            (
                fall.bridge_fell,
                fall.bridge_landed,
                fall.bridge_remove,
                fall.bridge_dropped_y,
                fall.bridge_floor_height,
                fall.bridge_landed_y
            ),
            (true, true, true, 99.5, 99.0, 99.0)
        );
        let walk = call_aim(&lua, &host, request(5, None))?;
        assert!(walk.need_lookup && walk.bridge_need == Some((2, 2, 0)));
        // REST: the terrain is asked for, then the height the host rests the segment at comes back
        let asking = call_aim(&lua, &host, request(6, None))?;
        assert!(asking.need_lookup && asking.bridge_need_terrain);
        let rest = call_aim(&lua, &host, request(6, Some((100.0, true))))?;
        assert_eq!(
            (rest.bridge_landed, rest.bridge_landed_y, rest.bridge_fell, rest.bridge_remove),
            (true, 99.0, false, false)
        );
        let stays: LuaFunction = lua.load("return function(r) return { state = 'done', landed = false } end").eval()?;
        let stayed = call_aim(&lua, &stays, request(6, None))?;
        assert_eq!((stayed.bridge_landed, stayed.bridge_landed_y), (false, 0.0));
        let unsaid: LuaFunction = lua.load("return function(r) return { state = 'done', landed = true } end").eval()?;
        assert!(call_aim(&lua, &unsaid, request(6, Some((1.0, false))))
            .err()
            .ok_or("accepted")?
            .contains("landed_y"));
        // a field missing, an action that is not one, and a lookup that is neither are errors; a fall that did not fall names no figure
        for (what, body) in [
            ("kill", "return { state = 'done' }"),
            ("variant", "return { state = 'done', notify = true }"),
            ("fell", "return { state = 'done', landed = false, remove = true }"),
            (
                "dropped_y",
                "return { state = 'done', fell = true, landed = false, remove = false, floor_height = 1 }",
            ),
            (
                "landed_y",
                "return { state = 'done', fell = true, landed = true, remove = true, dropped_y = 1, floor_height = 1 }",
            ),
            ("neither", "return { state = 'need_lookup', need = 'wind' }"),
        ] {
            let op = match what {
                "kill" => 1,
                "variant" => 2,
                _ => 4,
            };
            let f: LuaFunction = lua.load(format!("return function(r) {body} end")).eval()?;
            assert!(
                call_aim(&lua, &f, request(op, Some((1.0, false)))).err().ok_or("accepted")?.contains(what),
                "{what}"
            );
        }
        let quiet: LuaFunction = lua
            .load("return function(r) return { state = 'done', fell = false, landed = false, remove = true } end")
            .eval()?;
        let supported = call_aim(&lua, &quiet, request(4, None))?;
        assert_eq!((supported.bridge_fell, supported.bridge_remove, supported.bridge_dropped_y), (false, true, 0.0));
        let act: LuaFunction = lua
            .load("return function(r) return { state = 'done', action = 'act', handle = 7 } end")
            .eval()?;
        let acted = call_aim(&lua, &act, request(5, None))?;
        assert_eq!((acted.bridge_action, acted.bridge_handle), (1, 7));
        let bad: LuaFunction = lua.load("return function(r) return { state = 'done', action = 'explode' } end").eval()?;
        assert!(call_aim(&lua, &bad, request(5, None)).err().ok_or("accepted")?.contains("is not none or act"));
        Ok(())
    }
    // The identity of a structure is asked of the host by value and the one it issues is read as a string, never defaulted; the identity an offered object carries reaches Lua, and the identity the host repeats for
    // the object an action names is read as an optional string (the campaign compares it with the one it offered).
    #[test]
    fn a_structure_identity_question_reaches_lua_by_value_and_the_identities_of_offered_objects_are_read() -> Result<(), Box<dyn std::error::Error>> {
        use eech_engine::{AimBlast, AimBlastEntry, AimBlastWindow, AimIdentify};
        let lua = Lua::new();
        let identify = AimRequest::Identify(AimIdentify {
            entity_type: 5,
            sub_type: 41,
            side: 2,
            alive: true,
            damage_level: 80,
            initial_damage_level: 100,
            position: [100.0, 5.0, 250.0],
            serial: 3,
        });
        let host: LuaFunction = lua
            .load(
                "return function(r) assert(r.mode == 'identify' and r.entity_type == 5 and r.sub_type == 41 and r.side == 2 and r.alive == true and r.damage_level == 80 and r.initial_damage_level == 100 \
                 and r.position[1] == 100 and r.position[3] == 250 and r.serial == 3)
                 return { state = 'done', host_id = 'dcs:static:' .. r.serial } end",
            )
            .eval()?;
        assert_eq!(call_aim(&lua, &host, identify.clone())?.identify_host_id.as_deref(), Some("dcs:static:3"));
        // an identity that is missing or not a string is an error, not a default
        for (what, body) in [
            ("missing", "return { state = 'done' }"),
            ("not a string", "return { state = 'done', host_id = 7 }"),
        ] {
            let f: LuaFunction = lua.load(format!("return function(r) {body} end")).eval()?;
            assert!(call_aim(&lua, &f, identify.clone()).is_err(), "{what}");
        }
        // an offered object carries its identity into the window the host reads; the host repeats it for the object it reaches
        let entry = |handle: i32, id: Option<&str>| AimBlastEntry {
            handle,
            entity_type: 6,
            readable: true,
            alive: true,
            has_position: true,
            rocket: false,
            excluded: false,
            position: [160.0, 5.0, 150.0],
            host_id: id.map(str::to_owned),
        };
        let blast = AimRequest::Blast(AimBlast {
            position: [150.0, 5.0, 150.0],
            damage_capability: 400,
            warhead_type: 7,
            has_target: false,
            sector_side_length: 100,
            min_x_sector: 0,
            max_x_sector: 2,
            min_z_sector: 0,
            max_z_sector: 2,
            window: Some(AimBlastWindow {
                sx: 1,
                sz: 1,
                after: 0,
                end: true,
                entries: vec![entry(1, None), entry(2, Some("site:2"))],
            }),
            echo: false,
        });
        let host: LuaFunction = lua
            .load(
                "return function(r) assert(r.window.objects[1].host_id == nil and r.window.objects[2].host_id == 'site:2')
                 return { state = 'done', radius = 20, action = 'collision', handle = 2, modifier = 0.5, host_id = r.window.objects[2].host_id } end",
            )
            .eval()?;
        let reply = call_aim(&lua, &host, blast.clone())?;
        assert_eq!((reply.blast_handle, reply.blast_target_host_id.as_deref()), (2, Some("site:2")));
        let silent: LuaFunction = lua
            .load("return function(r) return { state = 'done', radius = 20, action = 'collision', handle = 1, modifier = 0.5 } end")
            .eval()?;
        assert_eq!(call_aim(&lua, &silent, blast.clone())?.blast_target_host_id, None);
        let wrong_type: LuaFunction = lua
            .load("return function(r) return { state = 'done', radius = 20, action = 'collision', handle = 1, modifier = 0.5, host_id = 5 } end")
            .eval()?;
        assert!(call_aim(&lua, &wrong_type, blast).is_err());
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

    // ---- the host's model provider (EEDCS-50, D3): the REAL Lua bridge, with numbers, text and stateful getters, under the engine's round-toward-zero ----

    type Test = Result<(), Box<dyn std::error::Error>>;

    /// the provider the engine keeps for a host function written in Lua: `source` evaluates to the function `model (op, scene, depth)`
    fn model_under_test(lua: &Lua, source: &str) -> Result<ModelProvider, Box<dyn std::error::Error>> {
        let function: LuaFunction = lua.load(source).eval()?;
        Ok(model_provider(lua, function)?)
    }

    /// one question, asked under round-toward-zero as the engine's callback runs it; the code under test must leave the ROUNDING MODE as it found it (`fegetround` only: not a proof of the flags,
    /// masks or MXCSR, which the native `eech_engine_model_host_fpu` measures at the public entries)
    fn ask(provider: &mut ModelProvider, query: ModelQuery) -> Result<ModelAnswer, String> {
        let (answer, kept) = eech_engine::under_round_toward_zero(|| provider(query));
        assert!(kept, "the provider changed the rounding mode");
        answer
    }

    fn bits<const N: usize>(values: [f32; N]) -> [u32; N] {
        values.map(f32::to_bits)
    }

    const SIX_ZEROS: &str = "0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000";
    const NINE_ZEROS: &str = "0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000";

    #[test]
    fn a_scene_answer_keeps_tiny_subnormal_and_negative_zero_numbers_exactly() -> Test {
        let lua = Lua::new();
        // 1e-20 is within f64::EPSILON of 0, so a LuaValue read would call it the integer 0; 1e-45 and 5e-324 are subnormal; -zero is negative zero
        let mut provider = model_under_test(
            &lua,
            "local zero = 0
             return function (op, scene, depth)
               return { name = 'AH64D_APACHE_LONGBOW', loaded = true, collider = false, radius = -zero,
                        bb = { 1e-20, -1e-20, 1e-45, -1e-45, 5e-324, -5e-324 }, bb2 = { -zero, zero, 0.1, -0.1, 3.0e38, -3.0e38 } }
             end",
        )?;
        let ModelAnswer::Scene(scene) = ask(&mut provider, ModelQuery::Scene { scene: 1 })? else {
            return Err("not a scene".into());
        };
        assert_eq!(scene.name, "AH64D_APACHE_LONGBOW");
        assert!(scene.loaded && !scene.collider);
        assert_eq!(scene.radius.to_bits(), (-0.0_f32).to_bits(), "negative zero kept its sign");
        assert_eq!(bits(scene.bounding_box), bits([1.0e-20_f32, -1.0e-20, 1.0e-45, -1.0e-45, 0.0, -0.0]));
        assert_eq!(bits(scene.bounding_box_without_lines), bits([-0.0_f32, 0.0, 0.1, -0.1, 3.0e38, -3.0e38]));
        Ok(())
    }

    #[test]
    fn bit_patterns_are_exact_and_a_non_finite_number_or_pattern_is_refused() -> Test {
        let lua = Lua::new();
        let mut provider = model_under_test(
            &lua,
            &format!(
                "return function (op, scene, depth)
                   if op == 'catalogue' then return {{ scenes = 3026 }} end
                   if op == 'scene' then
                     local radius = ({{ [1] = '0x80000001', [2] = '0x7fc00000', [3] = 0/0, [4] = 1/0, [5] = '0X3f800000' }})[scene]
                     return {{ name = '', loaded = false, collider = false, radius = radius, bb = '{SIX_ZEROS}', bb2 = '{SIX_ZEROS}' }}
                   end
                 end"
            ),
        )?;
        let ModelAnswer::Scene(scene) = ask(&mut provider, ModelQuery::Scene { scene: 1 })? else {
            return Err("not a scene".into());
        };
        assert_eq!(scene.radius.to_bits(), 0x8000_0001, "a subnormal bit pattern is exact");
        assert_eq!(bits(scene.bounding_box), [0; 6]);
        for scene in 2..=5 {
            let error = ask(&mut provider, ModelQuery::Scene { scene }).err().ok_or("accepted")?;
            assert!(error.contains("[model-provider:scene.answer] radius"), "scene {scene}: {error}");
        }
        assert_eq!(ask(&mut provider, ModelQuery::Catalogue)?, ModelAnswer::Catalogue { scenes: 3026 });
        Ok(())
    }

    /// the nine floats 1.0 to 9.0 as bit patterns
    const ONE_TO_NINE: &str = "0x3f800000,0x40000000,0x40400000,0x40800000,0x40a00000,0x40c00000,0x40e00000,0x41000000,0x41100000";

    #[test]
    fn a_route_node_and_a_route_face_keep_their_numbers_and_name_their_kind() -> Test {
        let lua = Lua::new();
        // 1e-45 is subnormal and a negative zero keeps its sign: the node crosses as exact bits, not as a number a LuaValue would round
        let mut provider = model_under_test(
            &lua,
            "local zero = 0
             return function (op, scene, depth, kind)
               if op == 'route_node' and depth == 0 then return { result = 'node', name = 'FIXED_WING_LANDING_ROUTE', p = { 1.5, -zero, 1e-45 } } end
               if op == 'route_node' then return { result = 'end', name = 'FIXED_WING_LANDING_ROUTE' } end
               if op == 'route_face' and depth == 0 then return { result = 'face', name = 'FIXED_WING_LANDING_ROUTE', point0 = 3, point1 = 7, red = 0, green = 255, blue = 16, polygons = false } end
               if op == 'route_face' and depth == 1 then return { result = 'absent', name = 'FIXED_WING_LANDING_ROUTE' } end
               return { result = 'end', name = 'FIXED_WING_LANDING_ROUTE' }
             end",
        )?;
        let node = ask(&mut provider, ModelQuery::RouteNode { scene: 5, kind: 76, index: 0 })?;
        let ModelAnswer::RouteNode {
            kind_name,
            step: ModelRouteNode::Found(position),
        } = node
        else {
            return Err("not a node".into());
        };
        assert_eq!(kind_name, "FIXED_WING_LANDING_ROUTE");
        assert_eq!(bits(position), bits([1.5_f32, -0.0, 1.0e-45]));
        assert_eq!(
            ask(&mut provider, ModelQuery::RouteNode { scene: 5, kind: 76, index: 3 })?,
            ModelAnswer::RouteNode {
                kind_name: "FIXED_WING_LANDING_ROUTE".into(),
                step: ModelRouteNode::End
            }
        );
        assert_eq!(
            ask(&mut provider, ModelQuery::RouteFace { scene: 5, kind: 76, index: 0 })?,
            ModelAnswer::RouteFace {
                kind_name: "FIXED_WING_LANDING_ROUTE".into(),
                step: ModelRouteFace::Found {
                    point: [3, 7],
                    colour: [0, 255, 16],
                    polygons: false
                }
            }
        );
        assert_eq!(
            ask(&mut provider, ModelQuery::RouteFace { scene: 5, kind: 76, index: 1 })?,
            ModelAnswer::RouteFace {
                kind_name: "FIXED_WING_LANDING_ROUTE".into(),
                step: ModelRouteFace::Absent
            }
        );
        Ok(())
    }

    #[test]
    fn a_route_answer_that_is_malformed_refused_or_of_the_wrong_kind_stops_with_its_marker() -> Test {
        let lua = Lua::new();
        // each answer is chosen by the cursor: 0 refused, 1 a face for a node question, 2 a colour out of range, 3 a polygon flag that is a number, 4 a node with four numbers,
        // 5 a name with a NUL, 6 no name
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene, depth, kind)
               local n = 'FIXED_WING_LANDING_ROUTE'
               if depth == 0 then return { result = 'refused', reason = 'no such model', name = n } end
               if depth == 1 then return { result = 'face', name = n, point0 = 0, point1 = 1, red = 0, green = 0, blue = 0, polygons = false } end
               if depth == 2 then return { result = 'face', name = n, point0 = 0, point1 = 1, red = 256, green = 0, blue = 0, polygons = false } end
               if depth == 3 then return { result = 'face', name = n, point0 = 0, point1 = 1, red = 0, green = 0, blue = 0, polygons = 1 } end
               if depth == 4 then return { result = 'node', name = n, p = { 1, 2, 3, 4 } } end
               if depth == 5 then return { result = 'end', name = 'A\\0B' } end
               return { result = 'end' }
             end",
        )?;
        let mut node = |index: i32| ask(&mut provider, ModelQuery::RouteNode { scene: 5, kind: 76, index });
        let refused = node(0).err().ok_or("accepted")?;
        assert!(
            refused.contains("[model-provider:route_node.refused]") && refused.contains("no such model"),
            "{refused}"
        );
        let wrong_kind = node(1).err().ok_or("accepted")?;
        assert!(wrong_kind.contains("[model-provider:route_node.answer]"), "{wrong_kind}");
        let four = node(4).err().ok_or("accepted")?;
        assert!(four.contains("[model-provider:route_node.answer]"), "{four}");
        let nul = node(5).err().ok_or("accepted")?;
        assert!(nul.contains("[model-provider:route_node.answer]"), "{nul}");
        let unnamed = node(6).err().ok_or("accepted")?;
        assert!(unnamed.contains("[model-provider:route_node.answer]"), "{unnamed}");
        let mut face = |index: i32| ask(&mut provider, ModelQuery::RouteFace { scene: 5, kind: 76, index });
        for cursor in [2, 3] {
            let error = face(cursor).err().ok_or("accepted")?;
            assert!(error.contains("[model-provider:route_face.answer]"), "cursor {cursor}: {error}");
        }
        Ok(())
    }

    #[test]
    fn a_target_point_offset_crosses_as_three_exact_bit_patterns_with_the_scenes_name() -> Test {
        let lua = Lua::new();
        // a negative zero, a subnormal and an ordinary number: the offset arrives as bits (a table of numbers would round them); the empty slot's zero is an answer, not a refusal
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene)
               if op == 'target_point_offset' and scene == 5 then return { result = 'offset', name = 'SCENE_FIVE', p = '0x80000000,0x00000001,0x40490fdb' } end
               if op == 'target_point_offset' and scene == 6 then return { result = 'offset', name = 'SCENE_SIX', p = { 1.5, 0, -2.25 } } end
               return { result = 'offset', name = 'EMPTY', p = '0x00000000,0x00000000,0x00000000' }
             end",
        )?;
        let ModelAnswer::TargetPointOffset { scene_name, offset } = ask(&mut provider, ModelQuery::TargetPointOffset { scene: 5 })? else {
            return Err("not an offset".into());
        };
        assert_eq!(scene_name, "SCENE_FIVE");
        assert_eq!(bits(offset), [0x8000_0000, 0x0000_0001, 0x4049_0fdb]);
        let ModelAnswer::TargetPointOffset { offset, .. } = ask(&mut provider, ModelQuery::TargetPointOffset { scene: 6 })? else {
            return Err("not an offset".into());
        };
        assert_eq!(bits(offset), bits([1.5_f32, 0.0, -2.25]));
        assert_eq!(
            ask(&mut provider, ModelQuery::TargetPointOffset { scene: 7 })?,
            ModelAnswer::TargetPointOffset {
                scene_name: "EMPTY".into(),
                offset: [0.0, 0.0, 0.0]
            }
        );
        Ok(())
    }

    #[test]
    fn a_target_point_offset_that_is_withheld_malformed_or_of_the_wrong_kind_stops_with_its_marker() -> Test {
        let lua = Lua::new();
        // each answer is chosen by the scene: 1 refused, 2 an end (the question has no cursor), 3 an absent, 4 two numbers, 5 a non-finite number, 6 four numbers, 7 no name,
        // 8 a name with a NUL, 9 a route node answered to this question, 10 text that is not three bit patterns, 11 a NaN bit pattern (refused by the bridge's decoder as `.answer`, not finite)
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene)
               local n = 'SCENE'
               if scene == 1 then return { result = 'refused', reason = 'no such model' } end
               if scene == 2 then return { result = 'end', name = n } end
               if scene == 3 then return { result = 'absent', name = n } end
               if scene == 4 then return { result = 'offset', name = n, p = { 1, 2 } } end
               if scene == 5 then return { result = 'offset', name = n, p = { 1, 0 / 0, 3 } } end
               if scene == 6 then return { result = 'offset', name = n, p = { 1, 2, 3, 4 } } end
               if scene == 7 then return { result = 'offset', p = { 1, 2, 3 } } end
               if scene == 8 then return { result = 'offset', name = 'A\\0B', p = { 1, 2, 3 } } end
               if scene == 9 then return { result = 'node', name = n, p = { 1, 2, 3 } } end
               if scene == 11 then return { result = 'offset', name = n, p = '0x7fc00000,0x00000000,0x00000000' } end
               return { result = 'offset', name = n, p = '1,2,3' }
             end",
        )?;
        let mut offset = |scene: i32| ask(&mut provider, ModelQuery::TargetPointOffset { scene });
        let refused = offset(1).err().ok_or("accepted")?;
        assert!(
            refused.contains("[model-provider:target_point_offset.refused]") && refused.contains("no such model"),
            "{refused}"
        );
        for scene in 2..=11 {
            let error = offset(scene).err().ok_or("accepted")?;
            assert!(error.contains("[model-provider:target_point_offset.answer]"), "scene {scene}: {error}");
        }
        // a NaN bit pattern is named as such, at the transport boundary
        let nan = offset(11).err().ok_or("accepted")?;
        assert!(nan.contains("p: not finite"), "{nan}");
        Ok(())
    }

    #[test]
    fn a_target_point_chain_crosses_as_sibling_positions_or_an_absence_with_the_scenes_name() -> Test {
        let lua = Lua::new();
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene)
               if scene == 5 then return { result = 'chain', name = 'SCENE_FIVE', offsets = { 0, 3, 65535 } } end
               if scene == 6 then return { result = 'chain', name = 'SCENE_SIX', offsets = { 1, 2, 3, 4, 5, 6, 7, 8 } } end
               if scene == 0 then return { result = 'absent', name = '' } end
               return { result = 'absent', name = 'NO_TARGET_POINT' }
             end",
        )?;
        assert_eq!(
            ask(&mut provider, ModelQuery::TargetPointChain { scene: 5 })?,
            ModelAnswer::TargetPointChain {
                scene_name: "SCENE_FIVE".into(),
                offsets: Some(vec![0, 3, 65535])
            }
        );
        let ModelAnswer::TargetPointChain { offsets: Some(offsets), .. } = ask(&mut provider, ModelQuery::TargetPointChain { scene: 6 })? else {
            return Err("not a chain".into());
        };
        assert_eq!(offsets, vec![1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(
            ask(&mut provider, ModelQuery::TargetPointChain { scene: 7 })?,
            ModelAnswer::TargetPointChain {
                scene_name: "NO_TARGET_POINT".into(),
                offsets: None
            }
        );
        // the invalid scene is asked too, and absent is its answer
        assert!(matches!(
            ask(&mut provider, ModelQuery::TargetPointChain { scene: 0 })?,
            ModelAnswer::TargetPointChain { offsets: None, .. }
        ));
        Ok(())
    }

    #[test]
    fn a_target_point_chain_that_is_withheld_too_deep_empty_fractional_or_of_the_wrong_kind_stops_with_its_marker() -> Test {
        let lua = Lua::new();
        // each answer is chosen by the scene: 1 refused, 2 nine levels, 3 none, 4 a fractional number, 5 a negative number, 6 a hole, 7 no name, 8 a world answer, 9 a bit pattern
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene)
               local n = 'SCENE'
               if scene == 1 then return { result = 'refused', reason = 'no such model' } end
               if scene == 2 then return { result = 'chain', name = n, offsets = { 0, 0, 0, 0, 0, 0, 0, 0, 0 } } end
               if scene == 3 then return { result = 'chain', name = n, offsets = {} } end
               if scene == 4 then return { result = 'chain', name = n, offsets = { 0.5 } } end
               if scene == 5 then return { result = 'chain', name = n, offsets = { -1 } } end
               if scene == 6 then return { result = 'chain', name = n, offsets = { [1] = 0, [3] = 2 } } end
               if scene == 7 then return { result = 'chain', offsets = { 0 } } end
               if scene == 8 then return { result = 'world', name = n, p = { 1, 2, 3 } } end
               return { result = 'chain', name = n, offsets = '0x00000000' }
             end",
        )?;
        let mut chain = |scene: i32| ask(&mut provider, ModelQuery::TargetPointChain { scene });
        let refused = chain(1).err().ok_or("accepted")?;
        assert!(
            refused.contains("[model-provider:target_point_chain.refused]") && refused.contains("no such model"),
            "{refused}"
        );
        for scene in 2..=9 {
            let error = chain(scene).err().ok_or("accepted")?;
            assert!(error.contains("[model-provider:target_point_chain.answer]"), "scene {scene}: {error}");
        }
        Ok(())
    }

    #[test]
    fn an_entity_collision_question_carries_only_the_segment_and_the_pose_and_the_computed_words_and_the_write_mask_come_back_exactly() -> Test {
        let lua = Lua::new();
        // the provider answers a hit that assigned everything for a stationary pose (18 words) and a miss that assigned only the normal for a moving one (22). It hands back words of
        // the pose it received (words 6 to 11, which are candidate words here: the outputs of the original are NOT part of the question) so that a negative zero, a subnormal, a
        // signalling NaN pattern and an infinity must survive both crossings; the unassigned point words are zero
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene, depth, kind, definition, key)
               assert (op == 'entity_collision' and depth == nil and kind == nil and definition == nil)
               local words = {}
               for word in key:gmatch ('[^,]+') do words[#words + 1] = word end
               local given = {}
               for i = 7, 12 do given[#given + 1] = words[i] end
               if #words == 18 then
                 return { result = 'hit', mask = 63, name = 'CANDIDATE', p = table.concat (given, ',') }
               end
               given[1], given[2], given[3] = '0x00000000', '0x00000000', '0x00000000'
               return { result = 'miss', mask = 56, name = 'CANDIDATE', p = table.concat (given, ',') }
             end",
        )?;
        let weird = [-0.0_f32, f32::from_bits(1), 1234.5, f32::from_bits(0x7fa0_0001), f32::INFINITY, -1.0];
        let pose = |count: usize| -> Vec<u32> {
            let mut words: Vec<u32> = (0..6).map(|i| (i as f32 + 0.25).to_bits()).collect();
            words.extend(weird.map(f32::to_bits));
            words.extend((0..count - 12).map(|i| (i as f32 + 0.5).to_bits()));
            words
        };
        let ModelAnswer::EntityCollision {
            scene_name,
            hit,
            write_mask,
            words,
        } = ask(&mut provider, ModelQuery::EntityCollision { scene: 7, pose: pose(18) })?
        else {
            return Err("not an entity collision answer".into());
        };
        assert_eq!((scene_name.as_str(), hit, write_mask), ("CANDIDATE", true, 63));
        assert_eq!(bits(words), bits(weird));
        let ModelAnswer::EntityCollision { hit, write_mask, words, .. } = ask(&mut provider, ModelQuery::EntityCollision { scene: 7, pose: pose(22) })? else {
            return Err("not an entity collision answer".into());
        };
        assert_eq!(
            (hit, write_mask),
            (false, 56),
            "a miss that assigned only the normal is not read as a hit or as a full write"
        );
        assert_eq!(bits(words), bits([0.0, 0.0, 0.0, weird[3], weird[4], weird[5]]));
        Ok(())
    }

    #[test]
    fn an_entity_candidate_question_carries_the_facts_by_bit_pattern_and_every_decision_comes_back_with_its_state_and_exact_pose() -> Test {
        let lua = Lua::new();
        // the scene picks the decision: 1 skip, 2 need_updated, 3 need_normal, 4 buildings (a city: no name), 5 a test with the first 12 facts as the pose, 6 a test with the first 16, 7 need_fixed, 8 need_placement
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene, depth, kind, definition, key)
               assert (op == 'entity_candidate' and depth == nil and kind == nil and definition == nil)
               local words = {}
               for word in key:gmatch ('[^,]+') do words[#words + 1] = word end
               assert (#words == 44, 'the facts are 44 patterns')
               local function first (n) return table.concat (words, ',', 1, n) end
               local names = { 'skip', 'need_updated', 'need_normal', 'buildings' }
               if scene >= 1 and scene <= 4 then return { result = names[scene], name = scene == 4 and '' or 'CANDIDATE' } end
               if scene == 7 then return { result = 'need_fixed', name = '' } end
               if scene == 8 then return { result = 'need_placement', name = 'CANDIDATE' } end
               return { result = 'test', name = 'CANDIDATE', p = first (scene == 5 and 12 or 16) }
             end",
        )?;
        // finite words with a negative zero and a subnormal, which must survive both crossings
        let facts: Vec<u32> = [-0.0_f32, f32::from_bits(1), 1234.5, -1.0, 0.0, 2.5]
            .iter()
            .cycle()
            .take(44)
            .map(|float| float.to_bits())
            .collect();
        let decide = |provider: &mut ModelProvider, scene: i32| {
            ask(
                provider,
                ModelQuery::EntityCandidate {
                    scene,
                    facts: facts.clone(),
                    host_id: String::new(),
                },
            )
        };
        for (scene, expected) in [
            (1, CandidateDecision::Skip),
            (2, CandidateDecision::NeedUpdated),
            (3, CandidateDecision::NeedNormal),
            (4, CandidateDecision::Buildings),
            (7, CandidateDecision::NeedFixed),
            (8, CandidateDecision::NeedPlacement),
        ] {
            let ModelAnswer::EntityCandidate { decision, scene_name } = decide(&mut provider, scene)? else {
                return Err("not an entity candidate decision".into());
            };
            assert_eq!(decision, expected);
            assert_eq!(
                scene_name,
                if scene == 4 || scene == 7 { "" } else { "CANDIDATE" },
                "the scene's name, and none for a city or before a shape was read"
            );
        }
        for (scene, count) in [(5, 12_usize), (6, 16)] {
            let ModelAnswer::EntityCandidate {
                decision: CandidateDecision::Test(pose),
                ..
            } = decide(&mut provider, scene)?
            else {
                return Err("not a test decision".into());
            };
            assert_eq!(pose.len(), count);
            assert_eq!(
                pose.iter().map(|float| float.to_bits()).collect::<Vec<u32>>(),
                facts.iter().take(count).copied().collect::<Vec<u32>>(),
                "the pose crosses exactly: a negative zero and a subnormal included"
            );
        }
        Ok(())
    }

    #[test]
    fn an_entity_scan_question_carries_the_facts_by_bit_pattern_and_every_decision_comes_back_with_its_state_and_cursor() -> Test {
        let lua = Lua::new();
        // the first fact's pattern picks the decision: 1 first (sector 2, 3), 2 next, 3 offer, 4 done_none, 5 done_hit
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene, depth, kind, definition, key)
               assert (op == 'entity_scan' and scene == 0 and depth == nil and kind == nil and definition == nil)
               local words = {}
               for word in key:gmatch ('[^,]+') do words[#words + 1] = word end
               assert (#words == 16, 'the facts are 16 patterns')
               local pick = tonumber (words[1]:sub (-1))
               if pick == 1 then return { result = 'first', p = '0x40000000,0x40400000' } end
               local names = { [2] = 'next', [3] = 'offer', [4] = 'done_none', [5] = 'done_hit' }
               return { result = names[pick] }
             end",
        )?;
        let ask_with = |provider: &mut ModelProvider, pick: u32| {
            let facts: Vec<u32> = std::iter::once(pick).chain((1..16).map(|at| 0x3f80_0000 + at)).collect();
            ask(
                provider,
                ModelQuery::EntityScan {
                    facts,
                    identity: String::new(),
                },
            )
        };
        assert_eq!(
            ask_with(&mut provider, 1)?,
            ModelAnswer::EntityScan {
                decision: ScanDecision::First([2.0, 3.0]),
                key: String::new()
            }
        );
        assert_eq!(
            ask_with(&mut provider, 2)?,
            ModelAnswer::EntityScan {
                decision: ScanDecision::Next,
                key: String::new()
            }
        );
        assert_eq!(
            ask_with(&mut provider, 3)?,
            ModelAnswer::EntityScan {
                decision: ScanDecision::Offer,
                key: String::new()
            }
        );
        assert_eq!(
            ask_with(&mut provider, 4)?,
            ModelAnswer::EntityScan {
                decision: ScanDecision::DoneNone,
                key: String::new()
            }
        );
        assert_eq!(
            ask_with(&mut provider, 5)?,
            ModelAnswer::EntityScan {
                decision: ScanDecision::DoneHit,
                key: String::new()
            }
        );
        Ok(())
    }

    #[test]
    fn an_entity_scan_question_carries_a_roster_key_or_identity_after_the_facts_and_the_key_comes_back_in_the_name() -> Test {
        let lua = Lua::new();
        // the identity place after the 16 patterns picks the answer: `first` serves the key 77, `next` the key in the place plus one, an insertion answers the key it issued, a removal and a reset answer none
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene, depth, kind, definition, key)
               local words, identity = key:match ('^([^ ]+) ?(.*)$')
               local pick = tonumber (words:match ('0x[0-9a-f]+'):sub (-1))
               if pick == 1 then return { result = 'first', p = '0x40000000,0x40400000', name = '77' } end
               if pick == 2 then return { result = 'next', name = tostring (tonumber (identity) + 1) } end
               if pick == 4 then return { result = 'placed', name = '9001' } end
               if pick == 5 then assert (identity == '9001'); return { result = 'removed' } end
               if pick == 6 then return { result = 'cleared' } end
               if pick == 7 then return { result = 'offer', name = '5' } end
               if pick == 8 then return { result = 'next', name = 'two words' } end
             end",
        )?;
        let ask_with = |provider: &mut ModelProvider, pick: u32, identity: &str| {
            let facts: Vec<u32> = std::iter::once(pick).chain((1..16).map(|at| 0x3f80_0000 + at)).collect();
            ask(
                provider,
                ModelQuery::EntityScan {
                    facts,
                    identity: identity.to_string(),
                },
            )
        };
        assert_eq!(
            ask_with(&mut provider, 1, "")?,
            ModelAnswer::EntityScan {
                decision: ScanDecision::First([2.0, 3.0]),
                key: "77".into()
            }
        );
        assert_eq!(
            ask_with(&mut provider, 2, "41")?,
            ModelAnswer::EntityScan {
                decision: ScanDecision::Next,
                key: "42".into()
            }
        );
        assert_eq!(
            ask_with(&mut provider, 4, "eech-world:7")?,
            ModelAnswer::EntityScan {
                decision: ScanDecision::Placed,
                key: "9001".into()
            }
        );
        assert_eq!(
            ask_with(&mut provider, 5, "9001")?,
            ModelAnswer::EntityScan {
                decision: ScanDecision::Removed,
                key: String::new()
            }
        );
        assert_eq!(
            ask_with(&mut provider, 6, "")?,
            ModelAnswer::EntityScan {
                decision: ScanDecision::Cleared,
                key: String::new()
            }
        );
        // a name that belongs to no answer of its kind, or that is not one token, is a malformed answer
        for pick in [7, 8] {
            let Err(error) = ask_with(&mut provider, pick, "") else {
                return Err(format!("pick {pick}: a malformed key was accepted").into());
            };
            assert!(error.contains("name"), "pick {pick}: {error}");
        }
        Ok(())
    }

    /// the question of a roster or city event as the engine sends it: sixteen words, all zero but the phase (fact 11), and the identity place after them
    fn scan_event(phase: f32, identity: &str) -> ModelQuery {
        let mut facts = [0_u32; 16];
        facts[11] = phase.to_bits();
        ModelQuery::EntityScan {
            facts: facts.to_vec(),
            identity: identity.to_string(),
        }
    }

    /// what the provider answers: the Lua table the REAL provider (`host.model`) would return, handed through the REAL bridge
    fn scan_answer(lua: &Lua, provider: &mut ModelProvider, query: ModelQuery, answer: &str) -> Result<ModelAnswer, String> {
        lua.load(format!("ANSWER = {answer}")).exec().map_err(|error| error.to_string())?;
        ask(provider, query)
    }

    fn removal_provider(lua: &Lua) -> Result<ModelProvider, Box<dyn std::error::Error>> {
        model_under_test(lua, "return function () return ANSWER end")
    }

    fn removed(key: &str) -> ModelAnswer {
        ModelAnswer::EntityScan {
            decision: ScanDecision::Removed,
            key: key.to_string(),
        }
    }

    #[test]
    fn a_removal_names_the_record_the_host_retains_for_a_citys_box_only_in_the_two_removal_phases_that_declare_it() -> Test {
        let lua = Lua::new();
        let mut provider = removal_provider(&lua)?;
        // the roster REMOVE (5) of a city placement names its identity place (the placement's key) and is answered with the record's key; the CHILD_REMOVE (8) of the last building names "<list> <child>" and is
        // answered with the list's own key. The key must come back whole: a bridge that let the name pass and returned none would hand the campaign a plain removal
        assert_eq!(
            scan_answer(&lua, &mut provider, scan_event(5.0, "7"), "{ result = 'removed', name = '11193' }")?,
            removed("11193")
        );
        assert_eq!(
            scan_answer(
                &lua,
                &mut provider,
                scan_event(5.0, "7"),
                "{ result = 'removed', name = '18446744073709551615' }"
            )?,
            removed("18446744073709551615")
        );
        assert_eq!(
            scan_answer(&lua, &mut provider, scan_event(8.0, "11193 7"), "{ result = 'removed', name = '11193' }")?,
            removed("11193")
        );
        Ok(())
    }

    #[test]
    fn a_plain_removal_and_the_release_of_a_city_come_back_without_a_name() -> Test {
        let lua = Lua::new();
        let mut provider = removal_provider(&lua)?;
        for (phase, identity) in [(5.0, "7"), (8.0, "11193 7"), (16.0, "11193")] {
            assert_eq!(
                scan_answer(&lua, &mut provider, scan_event(phase, identity), "{ result = 'removed' }")?,
                removed(""),
                "phase {phase}"
            );
            assert_eq!(
                scan_answer(&lua, &mut provider, scan_event(phase, identity), "{ result = 'removed', name = '' }")?,
                removed(""),
                "phase {phase}: an empty name is none"
            );
        }
        Ok(())
    }

    #[test]
    fn a_name_on_a_removal_outside_its_declared_domain_is_a_malformed_answer() -> Test {
        let lua = Lua::new();
        let mut provider = removal_provider(&lua)?;
        // the private RELEASE acknowledgement (16) and every other event name no record on a removal; a removal answered at the wrong phase never gets a name through
        for (phase, identity) in [
            (0.0, ""),
            (1.0, "7"),
            (2.0, "7"),
            (3.0, "7"),
            (4.0, "eech-world:7"),
            (6.0, ""),
            (7.0, "11193 eech-world:7"),
            (9.0, "11193"),
            (10.0, "11193 7"),
            (11.0, "11193 7"),
            (12.0, "11193 eech-world:7"),
            (13.0, "11193"),
            (14.0, "7"),
            (15.0, "7 8"),
            (16.0, "11193"),
            (17.0, "11193"),
        ] {
            let Err(error) = scan_answer(&lua, &mut provider, scan_event(phase, identity), "{ result = 'removed', name = '11193' }") else {
                return Err(format!("phase {phase}: a name on a removal was accepted").into());
            };
            assert!(error.contains("entity_scan.answer") && error.contains("name"), "phase {phase}: {error}");
        }
        // a phase that is not exactly the whole number 5 or 8 (a neighbouring float, a negative zero, a non-finite word, no phase at all) is no removal phase
        for bits in [
            0x40a0_0001,
            0x409f_ffff,
            0x8000_0000,
            0x7fc0_0000,
            0x7f80_0000,
            0x4100_0001,
            0x40a0_0000 | 0x8000_0000,
        ] {
            let mut facts = [0_u32; 16];
            facts[11] = bits;
            let query = ModelQuery::EntityScan {
                facts: facts.to_vec(),
                identity: "11193 7".to_string(),
            };
            let Err(error) = scan_answer(&lua, &mut provider, query, "{ result = 'removed', name = '11193' }") else {
                return Err(format!("phase bits {bits:#010x}: a name on a removal was accepted").into());
            };
            assert!(error.contains("name"), "phase bits {bits:#010x}: {error}");
        }
        let short = ModelQuery::EntityScan {
            facts: vec![0_u32; 11],
            identity: "11193 7".to_string(),
        };
        let Err(error) = scan_answer(&lua, &mut provider, short, "{ result = 'removed', name = '11193' }") else {
            return Err("a name on a removal that states no phase was accepted".into());
        };
        assert!(error.contains("name"), "{error}");
        // the other results stay without a name wherever they are asked: the retaining answer is the removal's alone
        for result in ["cleared", "offer", "done_none", "done_hit"] {
            for phase in [5.0, 6.0, 8.0, 16.0] {
                let Err(error) = scan_answer(
                    &lua,
                    &mut provider,
                    scan_event(phase, "11193 7"),
                    &format!("{{ result = '{result}', name = '11193' }}"),
                ) else {
                    return Err(format!("{result} at phase {phase}: a name was accepted").into());
                };
                assert!(error.contains("name is only for"), "{result} at phase {phase}: {error}");
            }
        }
        Ok(())
    }

    #[test]
    fn a_retained_record_name_is_a_roster_key_and_a_city_child_removal_names_the_list_it_was_asked_about() -> Test {
        let lua = Lua::new();
        let mut provider = removal_provider(&lua)?;
        // a roster key is one to twenty decimal digits without a leading zero, as the campaign's own check reads it; a name of any other shape, of the wrong type or with a tail is no key
        for name in [
            "'0'",
            "'0123'",
            "'1x'",
            "'x1'",
            "'11193 '",
            "' 11193'",
            "'11193\\n'",
            "'+5'",
            "'5.0'",
            "'-5'",
            "'123456789012345678901'",
            "'1,2'",
            "'0x1'",
            "'@1'",
            "11193",
            "true",
            "{}",
        ] {
            let Err(error) = scan_answer(&lua, &mut provider, scan_event(5.0, "7"), &format!("{{ result = 'removed', name = {name} }}")) else {
                return Err(format!("name {name}: a malformed retained name was accepted").into());
            };
            assert!(error.contains("entity_scan.answer") && error.contains("name"), "name {name}: {error}");
        }
        // valid-looking wrong relations of a CHILD_REMOVE: the key of another list, the child's own key, a key of the list's digits plus a digit, and an identity place that holds no child at all
        for (identity, name) in [
            ("11193 7", "11194"),
            ("11193 7", "7"),
            ("11193 7", "111937"),
            ("11193 7", "1119"),
            ("11193", "11193"),
            ("", "11193"),
        ] {
            let Err(error) = scan_answer(
                &lua,
                &mut provider,
                scan_event(8.0, identity),
                &format!("{{ result = 'removed', name = '{name}' }}"),
            ) else {
                return Err(format!("{identity:?} naming {name}: another list was accepted").into());
            };
            assert!(
                error.contains("entity_scan.answer") && error.contains("names another list"),
                "{identity:?} naming {name}: {error}"
            );
        }
        // a cursor is only for `first`: a retained removal carrying one is as malformed as a plain one
        for phase in [5.0, 8.0] {
            let Err(error) = scan_answer(
                &lua,
                &mut provider,
                scan_event(phase, "11193 7"),
                "{ result = 'removed', name = '11193', p = '0x40000000,0x40400000' }",
            ) else {
                return Err(format!("phase {phase}: a cursor on a removal was accepted").into());
            };
            assert!(error.contains("p is only for first"), "phase {phase}: {error}");
        }
        // the unchanged domains: a refusal still stops the engine with the host's reason whatever it names, and a wrong result is never read as a removal
        let Err(error) = scan_answer(
            &lua,
            &mut provider,
            scan_event(5.0, "7"),
            "{ result = 'refused', reason = 'no', name = '11193' }",
        ) else {
            return Err("a refusal was accepted".into());
        };
        assert!(error.contains("entity_scan.refused"), "{error}");
        let Err(error) = scan_answer(&lua, &mut provider, scan_event(5.0, "7"), "{ result = 'remove', name = '11193' }") else {
            return Err("an unknown result was accepted".into());
        };
        assert!(error.contains("result must be"), "{error}");
        Ok(())
    }

    #[test]
    fn an_expanded_scan_answer_carries_six_bit_patterns_and_the_citys_key_and_a_short_or_non_finite_list_is_refused() -> Test {
        let lua = Lua::new();
        // phase 15: the box the host built, minimum then maximum, as bit patterns (a signed zero and the inverted box's magnitudes keep their bits); a list of five, one with a non-finite word and a
        // number table with a missing element are malformed answers of their own marker, never a box
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene, depth, kind, definition, key)
               local pick = tonumber (key:match ('0x[0-9a-f]+'):sub (-1))
               if pick == 1 then return { result = 'expanded', name = '17', p = '0x80000000,0x49742400,0xc9742400,0x00000001,0x40600000,0xc0880000' } end
               if pick == 2 then return { result = 'expanded', name = '17', p = '0x00000000,0x49742400,0xc9742400,0x00000001,0x40600000' } end
               if pick == 3 then return { result = 'expanded', name = '17', p = '0x00000000,0x49742400,0xc9742400,0x00000001,0x40600000,0x7f800000' } end
               if pick == 4 then return { result = 'expanded', name = '17', p = { 1, 2, 3, 4, 5 } } end
               if pick == 5 then return { result = 'expanded', name = '17' } end
             end",
        )?;
        let mut ask_pick = |pick: u32| {
            let facts: Vec<u32> = std::iter::once(0x3f80_0000 + pick).chain((1..16).map(|_| 0)).collect();
            ask(
                &mut provider,
                ModelQuery::EntityScan {
                    facts,
                    identity: "17 18".to_string(),
                },
            )
        };
        assert_eq!(
            ask_pick(1)?,
            ModelAnswer::EntityScan {
                decision: ScanDecision::Expanded([-0.0, 1.0e6, -1.0e6, f32::from_bits(1), 3.5, -4.25]),
                key: "17".into()
            }
        );
        if let ModelAnswer::EntityScan {
            decision: ScanDecision::Expanded(words),
            ..
        } = ask_pick(1)?
        {
            assert_eq!(words[0].to_bits(), 0x8000_0000, "the signed zero keeps its bit");
        }
        for pick in [2, 3, 4, 5] {
            let Err(error) = ask_pick(pick) else {
                return Err(format!("pick {pick}: a malformed box was accepted").into());
            };
            assert!(error.contains("entity_scan"), "pick {pick}: {error}");
        }
        Ok(())
    }

    #[test]
    fn a_sector_height_answer_carries_three_bit_patterns_and_every_raw_word_comes_back_exactly_and_a_number_or_a_short_list_or_a_name_is_refused() -> Test {
        let lua = Lua::new();
        // the provider picks its answer by the last digit of the first fact: the sector and the word are bit patterns, never numbers (the word is raw)
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene, depth, kind, definition, key)
               assert (op == 'entity_scan' and scene == 0)
               local pick = tonumber (key:sub (10, 10))
               if pick == 1 then return { result = 'height', p = '0x40a00000,0x41100000,0x7fc0f00d' } end
               if pick == 2 then return { result = 'restored', p = '0x40a00000,0x41100000,0xff800000' } end
               if pick == 3 then return { result = 'height', p = { 5, 9, 1.5 } } end
               if pick == 4 then return { result = 'height', p = '0x40a00000,0x41100000' } end
               if pick == 5 then return { result = 'height', name = '12', p = '0x40a00000,0x41100000,0x3f800000' } end
               if pick == 6 then return { result = 'height' } end
               if pick == 7 then return { result = 'restored', p = '0x40a00000,0x41100000,0x80000000' } end
               if pick == 8 then return { result = 'height', p = '0x40a00000,0x41100000,0x3f800000,0x00000000' } end
             end",
        )?;
        let mut ask_pick = |pick: u32| {
            let facts: Vec<u32> = std::iter::once(0x3f80_0000 + pick).chain((1..16).map(|_| 0)).collect();
            ask(
                &mut provider,
                ModelQuery::EntityScan {
                    facts,
                    identity: String::new(),
                },
            )
        };
        let words_of = |answer: ModelAnswer| match answer {
            ModelAnswer::EntityScan {
                decision: ScanDecision::Height(words) | ScanDecision::Restored(words),
                ..
            } => Some(words.map(f32::to_bits)),
            _ => None,
        };
        // a quiet NaN with a payload, a negative infinity and a negative zero keep every bit
        assert_eq!(words_of(ask_pick(1)?), Some([0x40a0_0000, 0x4110_0000, 0x7fc0_f00d]));
        assert!(matches!(
            ask_pick(1)?,
            ModelAnswer::EntityScan {
                decision: ScanDecision::Height(_),
                ..
            }
        ));
        assert_eq!(words_of(ask_pick(2)?), Some([0x40a0_0000, 0x4110_0000, 0xff80_0000]));
        assert!(matches!(
            ask_pick(2)?,
            ModelAnswer::EntityScan {
                decision: ScanDecision::Restored(_),
                ..
            }
        ));
        assert_eq!(words_of(ask_pick(7)?), Some([0x40a0_0000, 0x4110_0000, 0x8000_0000]));
        for pick in [3, 4, 5, 6, 8] {
            let Err(error) = ask_pick(pick) else {
                return Err(format!("pick {pick}: a malformed sector height answer was accepted").into());
            };
            assert!(error.contains("entity_scan"), "pick {pick}: {error}");
        }
        Ok(())
    }

    #[test]
    fn an_entity_pose_question_carries_the_words_by_bit_pattern_and_the_identity_and_every_class_comes_back() -> Test {
        let lua = Lua::new();
        // the stage, the entity's type and the alive flag are the numbers; the text is the 15 patterns, then (when there is one) a space and the identity, which may hold any text. The provider picks the
        // class by the last digit of the first pattern, and answers the kind and difference it was given in the second and third
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene, depth, kind, definition, key)
               assert (op == 'entity_pose' and scene == 0 and depth == 1 and kind == 4 and definition == 1)
               local patterns, rest = key:sub (1, 164), key:sub (165)
               local words = {}
               for word in patterns:gmatch ('[^,]+') do words[#words + 1] = word end
               assert (#words == 15, 'the words are 15 patterns')
               assert (words[15] == '0x7fa0000e', 'the last word arrives as sent')
               local pick = tonumber (words[1]:sub (-1))
               local names = { [1] = 'recorded', [2] = 'unrecorded', [3] = 'retired', [4] = 'unknown', [5] = 'no_id' }
               if rest ~= '' then assert (rest:sub (1, 1) == ' ' and rest:sub (2) == 'eech world:id', 'the identity follows one space') end
               return { result = names[pick], kind = pick == 1 and 5 or 0, difference = pick == 1 and 21 or 0 }
             end",
        )?;
        let mut ask_pose = |pick: u32, host_id: &str| {
            let words: Vec<u32> = std::iter::once(0x7fa0_0000 + pick).chain((1..15).map(|at| 0x7fa0_0000 + at)).collect();
            ask(
                &mut provider,
                ModelQuery::EntityPose {
                    stage: 1,
                    entity_type: 4,
                    alive: 1,
                    host_id: host_id.to_string(),
                    words,
                },
            )
        };
        assert_eq!(
            ask_pose(1, "eech world:id")?,
            ModelAnswer::EntityPose {
                class: PoseClass::Recorded,
                kind: 5,
                difference: 21
            }
        );
        for (pick, class) in [
            (2, PoseClass::Unrecorded),
            (3, PoseClass::Retired),
            (4, PoseClass::Unknown),
            (5, PoseClass::NoId),
        ] {
            assert_eq!(
                ask_pose(pick, if pick == 5 { "" } else { "eech world:id" })?,
                ModelAnswer::EntityPose { class, kind: 0, difference: 0 }
            );
        }
        Ok(())
    }

    #[test]
    fn an_entity_pose_answer_that_is_refused_malformed_or_out_of_range_stops_with_its_marker() -> Test {
        let lua = Lua::new();
        // the last digit of the first pattern picks the answer: 1 a refusal, 2 an unknown result, 3 a kind of 6, 4 a difference of 32, 5 no kind, 6 a fractional difference, 7 a negative kind
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene, depth, kind, definition, key)
               local pick = tonumber (key:match ('^0x%x%x%x%x%x%x%x(%x)'))
               if pick == 1 then return { result = 'refused', reason = 'no movers' } end
               if pick == 2 then return { result = 'perhaps', kind = 0, difference = 0 } end
               if pick == 3 then return { result = 'recorded', kind = 6, difference = 0 } end
               if pick == 4 then return { result = 'recorded', kind = 1, difference = 32 } end
               if pick == 5 then return { result = 'recorded', difference = 0 } end
               if pick == 6 then return { result = 'recorded', kind = 1, difference = 0.5 } end
               return { result = 'recorded', kind = -1, difference = 0 }
             end",
        )?;
        let mut decide = |pick: u32| {
            let words: Vec<u32> = std::iter::once(pick).chain(std::iter::repeat_n(0, 14)).collect();
            ask(
                &mut provider,
                ModelQuery::EntityPose {
                    stage: 0,
                    entity_type: 4,
                    alive: 1,
                    host_id: "eech-world:1".to_string(),
                    words,
                },
            )
        };
        let refused = decide(1).err().ok_or("accepted")?;
        assert!(
            refused.contains("[model-provider:entity_pose.refused]") && refused.contains("no movers"),
            "{refused}"
        );
        for pick in 2..=7 {
            let error = decide(pick).err().ok_or("accepted")?;
            assert!(error.contains("[model-provider:entity_pose.answer]"), "answer {pick}: {error}");
        }
        Ok(())
    }

    #[test]
    fn an_entity_scan_answer_that_is_refused_malformed_or_of_the_wrong_kind_stops_with_its_marker() -> Test {
        let lua = Lua::new();
        // the scene... carries nothing here: the first fact's last digit picks the answer: 1 a refusal, 2 first without a cursor, 3 first with one pattern, 4 next that carries a cursor, 5 an unknown result,
        // 6 first with a non-finite cursor
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene, depth, kind, definition, key)
               local pick = tonumber (key:match ('^0x%x%x%x%x%x%x%x(%x)'))
               if pick == 1 then return { result = 'refused', reason = 'no host' } end
               if pick == 2 then return { result = 'first' } end
               if pick == 3 then return { result = 'first', p = '0x40000000' } end
               if pick == 4 then return { result = 'next', p = '0x40000000,0x40000000' } end
               if pick == 5 then return { result = 'perhaps' } end
               return { result = 'first', p = '0x7f800000,0x40000000' }
             end",
        )?;
        let mut decide = |pick: u32| {
            let facts: Vec<u32> = std::iter::once(pick).chain(std::iter::repeat_n(0, 15)).collect();
            ask(
                &mut provider,
                ModelQuery::EntityScan {
                    facts,
                    identity: String::new(),
                },
            )
        };
        let refused = decide(1).err().ok_or("accepted")?;
        assert!(
            refused.contains("[model-provider:entity_scan.refused]") && refused.contains("no host"),
            "{refused}"
        );
        for pick in 2..=6 {
            let error = decide(pick).err().ok_or("accepted")?;
            assert!(error.contains("[model-provider:entity_scan.answer]"), "answer {pick}: {error}");
        }
        Ok(())
    }

    #[test]
    fn an_entity_candidate_answer_that_is_refused_malformed_unnamed_or_of_the_wrong_kind_stops_with_its_marker() -> Test {
        let lua = Lua::new();
        // the scene picks the answer: 1 a refusal, 2 a skip that carries a pose, 3 a test of 13 floats, 4 an unknown result, 5 an unnamed skip, 6 a test with a non-finite float,
        // 7 a test with no pose, 8 a name with a NUL
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene)
               local zeros = function (n) return string.rep ('0x00000000,', n):sub (1, -2) end
               if scene == 1 then return { result = 'refused', reason = 'no such model' } end
               if scene == 2 then return { result = 'skip', name = 'CANDIDATE', p = zeros (12) } end
               if scene == 3 then return { result = 'test', name = 'CANDIDATE', p = zeros (13) } end
               if scene == 4 then return { result = 'perhaps', name = 'CANDIDATE' } end
               if scene == 5 then return { result = 'skip' } end
               if scene == 6 then return { result = 'test', name = 'CANDIDATE', p = '0x7f800000,' .. zeros (11) } end
               if scene == 7 then return { result = 'test', name = 'CANDIDATE' } end
               return { result = 'skip', name = 'A\\0B' }
             end",
        )?;
        let mut decide = |scene: i32| {
            ask(
                &mut provider,
                ModelQuery::EntityCandidate {
                    scene,
                    facts: vec![0; 44],
                    host_id: String::new(),
                },
            )
        };
        let refused = decide(1).err().ok_or("accepted")?;
        assert!(
            refused.contains("[model-provider:entity_candidate.refused]") && refused.contains("no such model"),
            "{refused}"
        );
        for scene in [2, 3, 4, 5, 6, 7, 8] {
            let error = decide(scene).err().ok_or("accepted")?;
            assert!(error.contains("[model-provider:entity_candidate.answer]"), "scene {scene}: {error}");
        }
        Ok(())
    }

    #[test]
    fn an_entity_collision_answer_that_is_refused_short_unnamed_masked_out_of_range_or_of_the_wrong_kind_stops_with_its_marker() -> Test {
        let lua = Lua::new();
        // each answer is chosen by the scene: 1 refused, 2 five words, 3 no name, 4 a name with a NUL, 5 the answer of a target point question, 6 no mask, 7 mask 64, 8 mask 1.5,
        // 9 mask -1, 10 a mask that is not a number
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene)
               local n = 'SCENE'
               if scene == 1 then return { result = 'refused', reason = 'no such model' } end
               if scene == 2 then return { result = 'hit', mask = 63, name = n, p = { 1, 2, 3, 4, 5 } } end
               if scene == 3 then return { result = 'miss', mask = 56, p = { 0, 0, 0, 4, 5, 6 } } end
               if scene == 4 then return { result = 'hit', mask = 63, name = 'A\0B', p = { 1, 2, 3, 4, 5, 6 } } end
               if scene == 5 then return { result = 'world', mask = 63, name = n, p = { 1, 2, 3, 4, 5, 6 } } end
               if scene == 6 then return { result = 'miss', name = n, p = { 0, 0, 0, 4, 5, 6 } } end
               if scene == 7 then return { result = 'hit', mask = 64, name = n, p = { 1, 2, 3, 4, 5, 6 } } end
               if scene == 8 then return { result = 'hit', mask = 1.5, name = n, p = { 1, 2, 3, 4, 5, 6 } } end
               if scene == 9 then return { result = 'hit', mask = -1, name = n, p = { 1, 2, 3, 4, 5, 6 } } end
               return { result = 'hit', mask = 0 / 0, name = n, p = { 1, 2, 3, 4, 5, 6 } }
             end",
        )?;
        let mut collide = |scene: i32| ask(&mut provider, ModelQuery::EntityCollision { scene, pose: vec![0; 18] });
        let refused = collide(1).err().ok_or("accepted")?;
        assert!(
            refused.contains("[model-provider:entity_collision.refused]") && refused.contains("no such model"),
            "{refused}"
        );
        for scene in 2..=10 {
            let error = collide(scene).err().ok_or("accepted")?;
            assert!(error.contains("[model-provider:entity_collision.answer]"), "scene {scene}: {error}");
        }
        Ok(())
    }

    #[test]
    fn a_target_point_pose_reaches_the_provider_as_exact_bits_and_the_position_comes_back_as_exact_bits() -> Test {
        let lua = Lua::new();
        // the provider hands back the first three words of the pose it received (the root position of a world question, the base of a fixed one): a negative zero, a subnormal and an
        // ordinary number must survive both crossings
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene, depth, kind, definition, key)
               assert (depth == nil and kind == nil and definition == nil)
               local first = key:match ('^([^,]+,[^,]+,[^,]+)')
               if op == 'target_point_world' then return { result = 'world', name = 'WORLD', p = first } end
               return { result = 'fixed', name = 'FIXED', p = first }
             end",
        )?;
        let pose = |first: [f32; 3], rest: usize| -> Vec<u32> {
            let mut words: Vec<u32> = first.map(f32::to_bits).to_vec();
            words.extend((0..rest).map(|i| (i as f32 + 0.5).to_bits()));
            words
        };
        let weird = [-0.0_f32, f32::from_bits(1), 1234.5];
        let ModelAnswer::TargetPointPosition { scene_name, position } = ask(
            &mut provider,
            ModelQuery::TargetPointWorld {
                scene: 3,
                pose: pose(weird, 12 + 18),
            },
        )?
        else {
            return Err("not a position".into());
        };
        assert_eq!(scene_name, "WORLD");
        assert_eq!(bits(position), bits(weird));
        for rest in [0, 1] {
            let ModelAnswer::TargetPointPosition { scene_name, position } = ask(
                &mut provider,
                ModelQuery::TargetPointFixed {
                    scene: 4,
                    pose: pose(weird, rest),
                },
            )?
            else {
                return Err("not a position".into());
            };
            assert_eq!(scene_name, "FIXED");
            assert_eq!(bits(position), bits(weird));
        }
        Ok(())
    }

    #[test]
    fn a_target_point_position_that_is_withheld_non_finite_or_of_the_wrong_kind_stops_with_its_marker() -> Test {
        let lua = Lua::new();
        // each answer is chosen by the scene: 1 refused, 2 a NaN, 3 two numbers, 4 an answer of the other kind, 5 no name, 6 a name with a NUL, 7 a chain answer
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene)
               local kind = op == 'target_point_world' and 'world' or 'fixed'
               local other = op == 'target_point_world' and 'fixed' or 'world'
               local n = 'SCENE'
               if scene == 1 then return { result = 'refused', reason = 'no such model' } end
               if scene == 2 then return { result = kind, name = n, p = { 1, 0 / 0, 3 } } end
               if scene == 3 then return { result = kind, name = n, p = { 1, 2 } } end
               if scene == 4 then return { result = other, name = n, p = { 1, 2, 3 } } end
               if scene == 5 then return { result = kind, p = { 1, 2, 3 } } end
               if scene == 6 then return { result = kind, name = 'A\\0B', p = { 1, 2, 3 } } end
               return { result = 'chain', name = n, offsets = { 0 } }
             end",
        )?;
        for world in [true, false] {
            let label = if world { "target_point_world" } else { "target_point_fixed" };
            let mut question = |scene: i32| {
                let query = if world {
                    ModelQuery::TargetPointWorld { scene, pose: vec![0; 24] }
                } else {
                    ModelQuery::TargetPointFixed { scene, pose: vec![0; 3] }
                };
                ask(&mut provider, query)
            };
            let refused = question(1).err().ok_or("accepted")?;
            assert!(
                refused.contains(&format!("[model-provider:{label}.refused]")) && refused.contains("no such model"),
                "{refused}"
            );
            for scene in 2..=7 {
                let error = question(scene).err().ok_or("accepted")?;
                assert!(error.contains(&format!("[model-provider:{label}.answer]")), "{label} scene {scene}: {error}");
            }
        }
        Ok(())
    }

    #[test]
    fn an_object_table_an_object_and_an_animation_name_cross_as_counts_exact_bits_and_logical_names() -> Test {
        let lua = Lua::new();
        // the table answer is a count only (a room for custom objects is the campaign's own policy and is not read even if a host offers one); an object carries its six counts, its
        // radius and both boxes as exact bits; the custom definition of a scene is the same question under another name; an animation is a LOGICAL name, never an index
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene, depth, kind)
               if op == 'object_table' then return { result = 'table', objects = 4165, room = 2000 } end
               if op == 'object' or op == 'custom_object' then
                 if depth == 0 then return { result = 'object', name = 'AH64D_APACHE_LONGBOW', counts = { 4, 3, 2, 1, 0, 65535 }, radius = '0x80000001', bb = '0x3f800000,0x40000000,0x40400000,0x40800000,0x40a00000,0x40c00000', bb2 = { 1, 2, 3, 4, 5, 6 } } end
                 if depth == 1 then return { result = 'absent', name = 'AH64D_APACHE_LONGBOW' } end
               end
               if op == 'animation' then
                 if depth == 0 then return { result = 'name', animation = 'USA_COPTER_DIGIT__001', name = 'AH64D_APACHE_LONGBOW' } end
                 if depth == 1 then return { result = 'end', name = 'AH64D_APACHE_LONGBOW' } end
                 if depth == 2 then return { result = 'absent', name = 'AH64D_APACHE_LONGBOW' } end
               end
               return { result = 'refused', reason = 'no' }
             end",
        )?;
        assert_eq!(ask(&mut provider, ModelQuery::ObjectTable)?, ModelAnswer::ObjectTable { objects: 4165 });
        for custom in [false, true] {
            let ModelAnswer::Object {
                scene_name,
                step: ModelObject::Found(facts),
            } = ask(
                &mut provider,
                ModelQuery::Object {
                    scene: 1,
                    role: 0,
                    cursor: 0,
                    definition: 0,
                    custom,
                },
            )?
            else {
                return Err("not an object".into());
            };
            assert_eq!(scene_name, "AH64D_APACHE_LONGBOW");
            assert_eq!(facts.counts, [4, 3, 2, 1, 0, 65535]);
            assert_eq!(facts.radius.to_bits(), 0x8000_0001, "a subnormal bit pattern is exact");
            assert_eq!(
                bits(facts.bounding_box),
                [0x3f80_0000, 0x4000_0000, 0x4040_0000, 0x4080_0000, 0x40a0_0000, 0x40c0_0000]
            );
            assert_eq!(facts.bounding_box2, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
            assert!(matches!(
                ask(
                    &mut provider,
                    ModelQuery::Object {
                        scene: 1,
                        role: 3,
                        cursor: 1,
                        definition: 0,
                        custom
                    }
                )?,
                ModelAnswer::Object { step: ModelObject::Absent, .. }
            ));
        }
        assert_eq!(
            ask(
                &mut provider,
                ModelQuery::Animation {
                    scene: 1,
                    index: 0,
                    definition: 0
                }
            )?,
            ModelAnswer::Animation {
                scene_name: "AH64D_APACHE_LONGBOW".to_string(),
                step: ModelAnimation::Name("USA_COPTER_DIGIT__001".to_string())
            }
        );
        assert!(matches!(
            ask(
                &mut provider,
                ModelQuery::Animation {
                    scene: 1,
                    index: 1,
                    definition: 0
                }
            )?,
            ModelAnswer::Animation { step: ModelAnimation::End, .. }
        ));
        assert!(matches!(
            ask(
                &mut provider,
                ModelQuery::Animation {
                    scene: 1,
                    index: 2,
                    definition: 0
                }
            )?,
            ModelAnswer::Animation {
                step: ModelAnimation::Absent,
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn the_campaigns_walk_announcement_reaches_the_provider_with_its_directory_and_its_answers_come_back() -> Test {
        let lua = Lua::new();
        // the sixth argument is the directory name; the kind place says whether the step is the closing one
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene, depth, kind, definition, key)
               if op ~= 'custom_order' then return { result = 'refused', reason = 'no' } end
               if kind == 1 then return { result = 'end' } end
               if key == 'MODDIR' and scene == 7 and depth == 2 then return { result = 'found' } end
               if key == 'GONE' then return { result = 'absent' } end
               return { result = 'refused', reason = 'unexpected ' .. tostring (key) }
             end",
        )?;
        let step = |scene: i32, position: i32, end: bool, key: &str| ModelQuery::CustomOrder {
            scene,
            position,
            end,
            key: key.to_string(),
        };
        assert_eq!(ask(&mut provider, step(7, 2, false, "MODDIR"))?, ModelAnswer::CustomOrder(ModelOrder::Accepted));
        assert_eq!(ask(&mut provider, step(7, 3, false, "GONE"))?, ModelAnswer::CustomOrder(ModelOrder::Absent));
        assert_eq!(ask(&mut provider, step(0, 4, true, ""))?, ModelAnswer::CustomOrder(ModelOrder::End));
        let refused = ask(&mut provider, step(7, 0, false, "OTHER")).err().ok_or("accepted")?;
        assert!(
            refused.contains("[model-provider:custom_order.refused]") && refused.contains("OTHER"),
            "{refused}"
        );
        Ok(())
    }

    #[test]
    fn an_object_or_animation_answer_that_is_malformed_numeric_or_refused_stops_with_its_marker() -> Test {
        let lua = Lua::new();
        // each answer is chosen by the cursor: an animation given as an index, a name with a NUL, an empty name, a count out of range, five counts, a missing radius, a refusal
        let mut provider = model_under_test(
            &lua,
            "return function (op, scene, depth, kind)
               local n = 'AH64D_APACHE_LONGBOW'
               if op == 'animation' then
                 if depth == 0 then return { result = 'id', id = 7, name = n } end
                 if depth == 1 then return { result = 'name', animation = 'A\\0B', name = n } end
                 if depth == 2 then return { result = 'name', animation = '', name = n } end
                 if depth == 3 then return { result = 'refused', reason = 'animation_name_unknown', name = n } end
                 return { result = 'name', animation = 'X' }
               end
               if depth == 0 then return { result = 'object', name = n, counts = { 1, 2, 3, 4, 5, 65536 }, radius = 1, bb = { 1, 2, 3, 4, 5, 6 }, bb2 = { 1, 2, 3, 4, 5, 6 } } end
               if depth == 1 then return { result = 'object', name = n, counts = { 1, 2, 3, 4, 5 }, radius = 1, bb = { 1, 2, 3, 4, 5, 6 }, bb2 = { 1, 2, 3, 4, 5, 6 } } end
               if depth == 2 then return { result = 'object', name = n, counts = { 1, 2, 3, 4, 5, 6 }, bb = { 1, 2, 3, 4, 5, 6 }, bb2 = { 1, 2, 3, 4, 5, 6 } } end
               if depth == 3 then return { result = 'refused', reason = 'object_outside_table', name = n } end
               return { result = 'object', name = n, counts = { 1, 2, 3, 4, 5, 6 }, radius = 1, bb = { 1, 2, 3, 4, 5, 6 }, bb2 = { 1, 2, 3, 4, 5, -1/0 } }
             end",
        )?;
        let mut animation = |index: i32| {
            ask(
                &mut provider,
                ModelQuery::Animation {
                    scene: 1,
                    index,
                    definition: 0,
                },
            )
        };
        for cursor in [0, 1, 2, 4] {
            let error = animation(cursor).err().ok_or("accepted")?;
            assert!(error.contains("[model-provider:animation.answer]"), "animation {cursor}: {error}");
        }
        let refused = animation(3).err().ok_or("accepted")?;
        assert!(
            refused.contains("[model-provider:animation.refused]") && refused.contains("animation_name_unknown"),
            "{refused}"
        );
        let mut object = |cursor: i32| {
            ask(
                &mut provider,
                ModelQuery::Object {
                    scene: 1,
                    role: 0,
                    cursor,
                    definition: 0,
                    custom: false,
                },
            )
        };
        for cursor in [0, 1, 2, 4] {
            let error = object(cursor).err().ok_or("accepted")?;
            assert!(error.contains("[model-provider:object.answer]"), "object {cursor}: {error}");
        }
        let refused = object(3).err().ok_or("accepted")?;
        assert!(
            refused.contains("[model-provider:object.refused]") && refused.contains("object_outside_table"),
            "{refused}"
        );
        Ok(())
    }

    #[test]
    fn a_triangle_is_a_table_or_a_text_of_exactly_nine_and_nothing_longer_or_sparser() -> Test {
        let lua = Lua::new();
        // (accepted, name), in the order of the Lua table below; the depth the bridge passes through picks the case
        let expect: [(bool, &str); 20] = [
            (true, "exact table"),
            (true, "exact text"),
            (false, "one extra at 10"),
            (false, "extra at 11 after a hole at 10"),
            (false, "extra at 1000"),
            (false, "a string key"),
            (false, "a key 0"),
            (false, "eight numbers"),
            (false, "a hole at 6"),
            (false, "ten patterns of text"),
            (false, "six patterns of text"),
            (false, "patterns separated by comma and space"),
            (false, "a hundred thousand patterns of text"),
            (false, "a NaN element"),
            (false, "a bit pattern inside a table"),
            (false, "a boolean"),
            (false, "no p"),
            (false, "the right length with semicolons"),
            (true, "upper-case digits"),
            (false, "an infinity pattern"),
        ];
        let mut provider = model_under_test(
            &lua,
            &format!(
                "local cases = {{
                   {{ result = 'triangle', p = {{ 1, 2, 3, 4, 5, 6, 7, 8, 9 }} }},
                   {{ result = 'triangle', p = '{ONE_TO_NINE}' }},
                   {{ result = 'triangle', p = {{ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 }} }},
                   {{ result = 'triangle', p = {{ 1, 2, 3, 4, 5, 6, 7, 8, 9, [11] = 11 }} }},
                   {{ result = 'triangle', p = {{ 1, 2, 3, 4, 5, 6, 7, 8, 9, [1000] = 1 }} }},
                   {{ result = 'triangle', p = {{ 1, 2, 3, 4, 5, 6, 7, 8, 9, extra = 1 }} }},
                   {{ result = 'triangle', p = {{ 1, 2, 3, 4, 5, 6, 7, 8, 9, [0] = 1 }} }},
                   {{ result = 'triangle', p = {{ 1, 2, 3, 4, 5, 6, 7, 8 }} }},
                   {{ result = 'triangle', p = {{ 1, 2, 3, 4, 5, nil, 7, 8, 9 }} }},
                   {{ result = 'triangle', p = '{ONE_TO_NINE},0x41200000' }},
                   {{ result = 'triangle', p = '{SIX_ZEROS}' }},
                   {{ result = 'triangle', p = string.gsub('{ONE_TO_NINE}', ',', ', ') }},
                   {{ result = 'triangle', p = string.rep('0x00000000,', 100000) }},
                   {{ result = 'triangle', p = {{ 1, 2, 3, 4, 5, 6, 7, 8, 0/0 }} }},
                   {{ result = 'triangle', p = {{ 1, 2, 3, 4, 5, 6, 7, 8, '0x3f800000' }} }},
                   {{ result = 'triangle', p = true }},
                   {{ result = 'triangle' }},
                   {{ result = 'triangle', p = string.gsub('{ONE_TO_NINE}', ',', ';') }},
                   {{ result = 'triangle', p = '0x3F800000,0x40000000,0x40400000,0x40800000,0x40A00000,0x40C00000,0x40E00000,0x41000000,0x41100000' }},
                   {{ result = 'triangle', p = '0x7f800000,0x40000000,0x40400000,0x40800000,0x40a00000,0x40c00000,0x40e00000,0x41000000,0x41100000' }},
                 }}
                 return function (op, scene, depth) return (cases[depth + 1]) end"
            ),
        )?;
        let mut first_two = Vec::new();
        for (depth, (accepted, name)) in expect.iter().enumerate() {
            let answer = ask(&mut provider, ModelQuery::Triangle { scene: 1, depth: depth as i32 });
            match (accepted, answer) {
                (true, Ok(ModelAnswer::Triangle(ModelTriangle::Found(points)))) => {
                    assert_eq!(bits(points), bits([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]), "{name}");
                    first_two.push(points);
                }
                (false, Err(error)) => assert!(error.contains("[model-provider:triangle.answer]"), "{name}: {error}"),
                (accepted, other) => return Err(format!("{name}: accepted={accepted} but the bridge gave {other:?}").into()),
            }
        }
        assert_eq!(first_two.len(), 3, "the table, the text and the upper-case text");
        Ok(())
    }

    #[test]
    fn every_field_of_an_answer_is_read_from_a_stateful_getter_exactly_once() -> Test {
        let lua = Lua::new();
        let mut provider = model_under_test(
            &lua,
            &format!(
                "READS = {{}}
                 local function counted (t, tag)
                   return setmetatable({{}}, {{ __index = function (_, k) local key = tag .. tostring (k); READS[key] = (READS[key] or 0) + 1; return t[k] end }})
                 end
                 return function (op, scene, depth)
                   READS = {{}}
                   if op == 'triangle' then
                     return counted ({{ result = 'triangle', p = counted ({{ 1, 2, 3, 4, 5, 6, 7, 8, 9 }}, 'p') }}, 'a.')
                   end
                   return counted ({{ name = 'X', loaded = true, collider = true, radius = 2.5, bb = counted ({{ 1, 2, 3, 4, 5, 6 }}, 'bb'), bb2 = '{SIX_ZEROS}' }}, 'a.')
                 end"
            ),
        )?;
        let reads = |lua: &Lua| -> Result<Vec<(String, i64)>, Box<dyn std::error::Error>> {
            let table: LuaTable = lua.globals().get("READS")?;
            let mut all = Vec::new();
            for pair in table.pairs::<String, i64>() {
                all.push(pair?);
            }
            all.sort();
            Ok(all)
        };
        ask(&mut provider, ModelQuery::Triangle { scene: 1, depth: 0 })?;
        let triangle = reads(&lua)?;
        // result, p, and p[1..9] plus p[10] (the bound on the list): each read once, nothing else read
        let names: Vec<&str> = triangle.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(triangle.len(), 12, "{triangle:?}");
        assert!(triangle.iter().all(|(_, n)| *n == 1), "{triangle:?}");
        assert!(names.contains(&"a.result") && names.contains(&"a.p") && names.contains(&"p10"), "{names:?}");
        ask(&mut provider, ModelQuery::Scene { scene: 1 })?;
        let scene = reads(&lua)?;
        assert!(scene.iter().all(|(_, n)| *n == 1), "{scene:?}");
        for field in ["a.name", "a.loaded", "a.collider", "a.radius", "a.bb", "a.bb2", "bb7"] {
            assert!(scene.iter().any(|(k, _)| k == field), "{field} was not read: {scene:?}");
        }
        assert_eq!(scene.len(), 6 + 6 + 1, "{scene:?}");
        Ok(())
    }

    #[test]
    fn the_whole_numbers_and_the_declarations_of_an_answer_reject_what_is_not_exactly_what_they_name() -> Test {
        let lua = Lua::new();
        // the catalogue count: a whole number only (1e-20 is within EPSILON of 0 and must not become the integer 0)
        let mut provider = model_under_test(
            &lua,
            "local cases = { { scenes = 3026 }, { scenes = 1e-20 }, { scenes = 3026.5 }, { scenes = -1 }, { scenes = 2^31 }, { scenes = '3026' }, { scenes = 0/0 }, {}, { scenes = 0 } }
             local i = 0
             return function () i = i + 1; return cases[i] end",
        )?;
        let accepted = [true, false, false, false, false, false, false, false, true];
        for (index, accepted) in accepted.iter().enumerate() {
            let answer = ask(&mut provider, ModelQuery::Catalogue);
            match (accepted, answer) {
                (true, Ok(ModelAnswer::Catalogue { .. })) => {}
                (false, Err(error)) => assert!(error.contains("[model-provider:catalogue.answer]"), "case {index}: {error}"),
                (accepted, other) => return Err(format!("case {index}: accepted={accepted} but the bridge gave {other:?}").into()),
            }
        }
        // the scene declarations
        let mut provider = model_under_test(
            &lua,
            &format!(
                "local function scene (over)
                   local t = {{ name = 'X', loaded = true, collider = false, radius = 1, bb = '{SIX_ZEROS}', bb2 = '{SIX_ZEROS}' }}
                   for k, v in pairs (over) do t[k] = v end
                   if over.drop then t[over.drop] = nil end
                   return t
                 end
                 local cases = {{ scene {{}}, scene {{ name = string.rep ('N', 127) }}, scene {{ name = string.rep ('N', 128) }}, scene {{ name = 'A\\0B' }}, scene {{ name = 1 }},
                   scene {{ loaded = 1 }}, scene {{ collider = 'yes' }}, scene {{ drop = 'loaded' }}, scene {{ radius = {{}} }}, scene {{ radius = '0x1' }}, scene {{ drop = 'bb' }},
                   scene {{ bb2 = '{NINE_ZEROS}' }}, scene {{ bb = {{ 1, 2, 3, 4, 5 }} }} }}
                 local i = 0
                 return function () i = i + 1; return cases[i] end"
            ),
        )?;
        let accepted = [true, true, false, false, false, false, false, false, false, false, false, false, false];
        for (index, accepted) in accepted.iter().enumerate() {
            let answer = ask(&mut provider, ModelQuery::Scene { scene: 1 });
            match (accepted, answer) {
                (true, Ok(ModelAnswer::Scene(_))) => {}
                (false, Err(error)) => assert!(error.contains("[model-provider:scene.answer]"), "case {index}: {error}"),
                (accepted, other) => return Err(format!("case {index}: accepted={accepted} but the bridge gave {other:?}").into()),
            }
        }
        Ok(())
    }

    #[test]
    fn the_kinds_of_triangle_answer_and_the_failures_of_the_provider_are_each_named() -> Test {
        let lua = Lua::new();
        let mut provider = model_under_test(
            &lua,
            "local cases = { { result = 'end' }, { result = 'absent' }, { result = 'refused', reason = 'no such model' }, { result = 'maybe' }, {}, { result = 7 }, 'nothing', 42 }
             local i = 0
             return function (op, scene, depth)
               i = i + 1
               if i == 9 then error ('the host fell over') end
               if i == 10 then return nil end
               return cases[i]
             end",
        )?;
        let query = ModelQuery::Triangle { scene: 7, depth: 3 };
        assert_eq!(ask(&mut provider, query.clone())?, ModelAnswer::Triangle(ModelTriangle::End));
        assert_eq!(ask(&mut provider, query.clone())?, ModelAnswer::Triangle(ModelTriangle::Absent));
        let refused = ask(&mut provider, query.clone()).err().ok_or("accepted")?;
        assert!(
            refused.contains("[model-provider:triangle.refused]") && refused.contains("scene 7 depth 3") && refused.contains("no such model"),
            "{refused}"
        );
        for _ in 0..3 {
            let error = ask(&mut provider, query.clone()).err().ok_or("accepted")?;
            assert!(error.contains("[model-provider:triangle.answer]"), "{error}");
        }
        for _ in 0..2 {
            let error = ask(&mut provider, query.clone()).err().ok_or("accepted")?;
            assert!(
                error.contains("[model-provider:triangle.answer]") && error.contains("instead of a table"),
                "{error}"
            );
        }
        let error = ask(&mut provider, query.clone()).err().ok_or("accepted")?;
        assert!(
            error.contains("[model-provider:triangle.callback]") && error.contains("the host fell over"),
            "{error}"
        );
        let error = ask(&mut provider, query.clone()).err().ok_or("accepted")?;
        assert!(
            error.contains("[model-provider:triangle.answer]") && error.contains("instead of a table"),
            "{error}"
        );
        Ok(())
    }

    // ---- the host terrain provider as the campaign calls it (EEDCS-47): the real bridge and the real answer parsing, with Lua providers of the test's choosing ----

    fn terrain_of(lua: &Lua, source: &str) -> Result<TerrainProvider, Box<dyn std::error::Error>> {
        let function: LuaFunction = lua.load(format!("return function (r) {source} end")).eval()?;
        Ok(terrain_provider(lua, function)?)
    }

    fn point(x: f32, z: f32, hint: Option<&str>) -> TerrainQuery {
        TerrainQuery::Point {
            x,
            z,
            hint: hint.map(str::to_owned),
            winter: false,
        }
    }

    /// The error of a provider call that must be refused (`expect_err` is denied in this crate): a call that is accepted fails the test with the answer.
    fn refusal_of(result: Result<TerrainAnswer, String>, what: &str) -> String {
        assert!(result.is_err(), "{what}: the call must be refused, got {result:?}");
        result.err().unwrap_or_default()
    }

    fn refused(provider: &mut TerrainProvider, query: TerrainQuery, marker: &str) {
        let error = refusal_of(provider(query), marker);
        assert!(error.starts_with(&format!("[terrain-provider:{marker}]")), "{marker}: {error}");
        assert!(
            !error.contains('\n') || error.lines().next().is_some_and(|line| line.contains(marker)),
            "{error}"
        );
    }

    fn tree_begin() -> TerrainQuery {
        TerrainQuery::TreeBegin {
            old: [1.0f32.to_bits(), 2.0f32.to_bits(), 3.0f32.to_bits()],
            new: [(-0.0f32).to_bits(), 5.0f32.to_bits(), 6.0f32.to_bits()],
            point: [0x7fa0_0001, 0x8000_0000, 0],
            normal: [0xffc0_0123, 1, 0x8000_0001],
            ignore_forest_edge_trees: true,
            skip_tree_top_test: false,
            winter: true,
        }
    }

    #[test]
    fn a_tree_question_reaches_the_provider_as_bit_patterns_and_its_answers_reach_the_campaign_as_bit_patterns() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        // the request carries the four position triples as patterns, the two flags as booleans and the season; nothing is a Lua number
        let mut provider = terrain_of(
            &lua,
            "assert (r.op == 'tree_begin' and r.winter == true and r.old == '0x3f800000,0x40000000,0x40400000' and r.new == '0x80000000,0x40a00000,0x40c00000'
                     and r.point == '0x7fa00001,0x80000000,0x00000000' and r.normal == '0xffc00123,0x00000001,0x80000001'
                     and r.ignore_forest_edge_trees == true and r.skip_tree_top_test == false and r.elevation == nil)
             return { state = 'need_terrain', ['end'] = 'new', x = '0x7f800001', z = '0x80000000' }",
        )?;
        assert_eq!(
            provider(tree_begin())?,
            TerrainAnswer::Tree(TreeStep::NeedTerrain {
                end: TreeEnd::New,
                x: 0x7f80_0001,
                z: 0x8000_0000
            })
        );
        let mut provider = terrain_of(
            &lua,
            "assert (r.op == 'tree_terrain' and r.winter == false and r.elevation == '0x41200000' and r.old == nil)
             return { state = 'done', hit = true, point = '0x7fa00001,0x80000000,0x00000007', normal = '0xffc00123,0x00000008,0x80000001' }",
        )?;
        assert_eq!(
            provider(TerrainQuery::TreeTerrain {
                elevation: 0x4120_0000,
                winter: false
            })?,
            TerrainAnswer::Tree(TreeStep::Done {
                hit: true,
                point: [0x7fa0_0001, 0x8000_0000, 7],
                normal: [0xffc0_0123, 8, 0x8000_0001]
            })
        );
        Ok(())
    }

    #[test]
    fn a_tree_answer_that_is_not_a_well_formed_step_is_refused_with_its_marker() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let ok_point = "'0x00000000,0x00000000,0x00000000'";
        let cases: Vec<(String, &str)> = vec![
            ("error ('the tree host is gone')".into(), "tree_begin.callback"),
            ("return 3".into(), "tree_begin.answer"),
            ("return { state = 'waiting' }".into(), "tree_begin.answer"),
            (
                "return { state = 'need_terrain', ['end'] = 'middle', x = '0x00000000', z = '0x00000000' }".into(),
                "tree_begin.answer",
            ),
            (
                "return { state = 'need_terrain', ['end'] = 'old', x = 1.5, z = '0x00000000' }".into(),
                "tree_begin.answer",
            ),
            (
                "return { state = 'need_terrain', ['end'] = 'old', x = '0x0000000', z = '0x00000000' }".into(),
                "tree_begin.answer",
            ),
            (
                format!("return {{ state = 'done', hit = 1, point = {ok_point}, normal = {ok_point} }}"),
                "tree_begin.answer",
            ),
            (
                format!("return {{ state = 'done', hit = false, point = '0x00000000,0x00000000', normal = {ok_point} }}"),
                "tree_begin.answer",
            ),
            (
                format!("return {{ state = 'done', hit = false, point = {ok_point}, normal = 0 }}"),
                "tree_begin.answer",
            ),
        ];
        for (source, marker) in cases {
            let mut provider = terrain_of(&lua, &source)?;
            refused(&mut provider, tree_begin(), marker);
        }
        // the same shape rules hold for the answer to a terrain elevation, under its own marker
        let mut provider = terrain_of(&lua, "return { state = 'maybe' }")?;
        refused(&mut provider, TerrainQuery::TreeTerrain { elevation: 0, winter: false }, "tree_terrain.answer");
        Ok(())
    }

    #[test]
    fn a_terrain_point_answer_reaches_the_campaign_exactly_and_the_request_reaches_the_provider_by_value() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        // the request carries op, x, z and the record's hint; the answer's tiny and negative-zero figures survive the round trip bit for bit
        let mut provider = terrain_of(
            &lua,
            "assert (r.op == 'point' and r.x == 1.5 and r.z == -2.5 and r.hint == 'previous' and r.winter == false)
             return { elevation = 123.5, last_elevation = 125.25, normal = { 2 ^ -130, 1, -1 / (1 / 0) }, type = 9, hint = 'next' }",
        )?;
        let answer = provider(point(1.5, -2.5, Some("previous")))?;
        let TerrainAnswer::Point {
            elevation,
            last_elevation,
            normal,
            kind,
            hint,
            stored,
        } = answer
        else {
            return Err("not a point answer".into());
        };
        assert_eq!((elevation, last_elevation, kind, hint.as_deref()), (123.5, 125.25, 9, Some("next")));
        assert!(stored, "an answer that does not say otherwise returned a surface, so it stored one");
        assert_eq!(
            normal.map(f32::to_bits),
            [0x0008_0000, 1.0f32.to_bits(), 0x8000_0000],
            "a subnormal and a negative zero are kept exactly"
        );
        // no hint in the request is nil to the provider, and no hint in the answer is None
        let mut provider = terrain_of(
            &lua,
            "assert (r.hint == nil and r.x ~= r.x) return { elevation = 0, last_elevation = 0, normal = { 0, 1, 0 }, type = 0 }",
        )?;
        let TerrainAnswer::Point { hint, .. } = provider(point(f32::NAN, 1.0, None))? else {
            return Err("not a point answer".into());
        };
        assert_eq!(hint, None, "a not-a-number position is passed on as it is, for the host to answer");
        Ok(())
    }

    #[test]
    fn whether_a_surface_is_stored_is_the_providers_word_by_value_and_not_its_hint() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let good = "elevation = 1, last_elevation = 1, normal = { 0, 1, 0 }, type = 9";
        let stored_of = |extra: &str| -> Result<(Option<String>, bool), Box<dyn std::error::Error>> {
            let mut provider = terrain_of(&lua, &format!("return {{ {good}{extra} }}"))?;
            match provider(point(1.0, 2.0, None))? {
                TerrainAnswer::Point { hint, stored, .. } => Ok((hint, stored)),
                other => Err(format!("not a point answer: {other:?}").into()),
            }
        };
        // a generic host returns a valid answer and no hint: a surface is stored
        assert_eq!(stored_of("")?, (None, true));
        // the reference host's no-face answer says it explicitly, with or without a hint
        assert_eq!(stored_of(", stored = false")?, (None, false));
        assert_eq!(stored_of(", stored = true, hint = 'h'")?, (Some("h".to_string()), true));
        assert_eq!(stored_of(", stored = false, hint = 'h'")?, (Some("h".to_string()), false));
        for bad in [", stored = 1", ", stored = 'true'", ", stored = {}"] {
            let mut provider = terrain_of(&lua, &format!("return {{ {good}{bad} }}"))?;
            refused(&mut provider, point(1.0, 2.0, None), "point.stored");
        }
        Ok(())
    }

    #[test]
    fn a_malformed_point_answer_is_refused_with_the_marker_of_what_is_wrong() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let good = "elevation = 1, last_elevation = 1, normal = { 0, 1, 0 }, type = 9";
        let cases: Vec<(String, String, &str)> = vec![
            ("return nil".to_string(), "nothing".into(), "point.answer"),
            ("return 5".to_string(), "a number".into(), "point.answer"),
            (
                "return { last_elevation = 1, normal = { 0, 1, 0 }, type = 9 }".to_string(),
                "no elevation".into(),
                "point.elevation",
            ),
            (
                "return { elevation = 0 / 0, last_elevation = 1, normal = { 0, 1, 0 }, type = 9 }".to_string(),
                "a NaN elevation".into(),
                "point.elevation",
            ),
            (
                "return { elevation = 1, normal = { 0, 1, 0 }, type = 9 }".to_string(),
                "no stored elevation".into(),
                "point.elevation",
            ),
            (
                "return { elevation = 1, last_elevation = 1 / 0, normal = { 0, 1, 0 }, type = 9 }".to_string(),
                "an infinite stored elevation".into(),
                "point.elevation",
            ),
            (
                "return { elevation = '1', last_elevation = 1, normal = { 0, 1, 0 }, type = 9 }".to_string(),
                "a text elevation".into(),
                "point.elevation",
            ),
            (
                "return { elevation = 1, last_elevation = 1, normal = { 0, 1 }, type = 9 }".to_string(),
                "two normal components".into(),
                "point.normal",
            ),
            (
                "return { elevation = 1, last_elevation = 1, normal = { 0, 1, 0 / 0 }, type = 9 }".to_string(),
                "a NaN normal".into(),
                "point.normal",
            ),
            (
                "return { elevation = 1, last_elevation = 1, type = 9 }".to_string(),
                "no normal".into(),
                "point.normal",
            ),
            (
                "return { elevation = 1, last_elevation = 1, normal = { 0, 1, 0 }, type = -1 }".to_string(),
                "a negative type".into(),
                "point.type",
            ),
            (
                "return { elevation = 1, last_elevation = 1, normal = { 0, 1, 0 }, type = 62 }".to_string(),
                "a type beyond the table".into(),
                "point.type",
            ),
            (
                "return { elevation = 1, last_elevation = 1, normal = { 0, 1, 0 }, type = 9.5 }".to_string(),
                "a fractional type".into(),
                "point.type",
            ),
            (
                "return { elevation = 1, last_elevation = 1, normal = { 0, 1, 0 }, type = '9' }".to_string(),
                "a text type".into(),
                "point.type",
            ),
            (
                "return { elevation = 1, last_elevation = 1, normal = { 0, 1, 0 } }".to_string(),
                "no type".into(),
                "point.type",
            ),
            (format!("return {{ {good}, hint = 12345 }}"), "a number hint".into(), "point.hint"),
            (format!("return {{ {good}, hint = '' }}"), "an empty hint".into(), "point.hint"),
            (
                format!("return {{ {good}, hint = string.rep ('a', 128) }}"),
                "a hint over the limit".into(),
                "point.hint",
            ),
            (format!("return {{ {good}, hint = 'bad\\0nul' }}"), "a hint with a NUL".into(), "point.hint"),
            (
                format!("return {{ {good}, hint = 'caf\\233' }}"),
                "a hint outside printable ASCII".into(),
                "point.hint",
            ),
            (
                format!("return {{ {good}, hint = 'with space\\9tab' }}"),
                "a hint with a control character".into(),
                "point.hint",
            ),
        ];
        for (source, name, marker) in cases {
            let mut provider = terrain_of(&lua, &source)?;
            let error = refusal_of(provider(point(1.0, 2.0, None)), &name);
            assert!(error.starts_with(&format!("[terrain-provider:{marker}]")), "{name}: {error}");
        }
        // the limit itself, and a provider's own error with its original text
        let mut at_the_limit = terrain_of(&lua, &format!("return {{ {good}, hint = string.rep ('a', 127) }}"))?;
        assert!(matches!(at_the_limit(point(1.0, 2.0, None))?, TerrainAnswer::Point { hint: Some(h), .. } if h.len() == 127));
        let mut failing = terrain_of(&lua, "error ('provider failed on purpose')")?;
        let error = refusal_of(failing(point(1.0, 2.0, None)), "a raising provider");
        assert!(
            error.starts_with("[terrain-provider:point.callback]") && error.contains("provider failed on purpose"),
            "{error}"
        );
        Ok(())
    }

    #[test]
    fn the_fast_and_map_operations_are_read_and_refused_the_same_way() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let mut fast = terrain_of(
            &lua,
            "assert (r.op == 'fast' and r.x == 10 and r.z == 20 and r.hint == nil and r.winter == true) return { elevation = 77.75 }",
        )?;
        assert_eq!(
            fast(TerrainQuery::Fast {
                x: 10.0,
                z: 20.0,
                winter: true
            })?,
            TerrainAnswer::Fast { elevation: 77.75 }
        );
        let mut bad = terrain_of(&lua, "return { elevation = 0 / 0 }")?;
        refused(&mut bad, TerrainQuery::Fast { x: 1.0, z: 1.0, winter: false }, "fast.elevation");
        let mut none = terrain_of(&lua, "return nil")?;
        refused(&mut none, TerrainQuery::Fast { x: 1.0, z: 1.0, winter: false }, "fast.answer");
        let mut raising = terrain_of(&lua, "error ('no grid')")?;
        refused(&mut raising, TerrainQuery::Fast { x: 1.0, z: 1.0, winter: false }, "fast.callback");

        let facts = "sector_x_max = 76, sector_z_max = 92, fast_width = 513, fast_height = 513, min_x = 0, min_z = 0, max_x = 155647, max_z = 188415,
                     min_height = -1, max_height = 3069, scaled_height_difference = 0.09366";
        let mut map = terrain_of(&lua, &format!("assert (r.op == 'map' and r.winter == true) return {{ {facts} }}"))?;
        let TerrainAnswer::Map(read) = map(TerrainQuery::Map { winter: true })? else {
            return Err("not a map answer".into());
        };
        assert_eq!((read.sector_x_max, read.sector_z_max, read.fast_width, read.fast_height), (76, 92, 513, 513));
        assert_eq!(
            (read.min_x, read.max_x, read.max_z, read.min_height, read.max_height),
            (0.0, 155647.0, 188415.0, -1.0, 3069.0)
        );
        assert_eq!(read.scaled_height_difference.to_bits(), 0.09366f32.to_bits());
        for (name, source) in [
            ("a fractional sector count", facts.replace("sector_x_max = 76", "sector_x_max = 76.5")),
            ("a zero sector count", facts.replace("sector_z_max = 92", "sector_z_max = 0")),
            ("a negative grid size", facts.replace("fast_width = 513", "fast_width = -1")),
            ("a NaN extent", facts.replace("max_x = 155647", "max_x = 0 / 0")),
            ("a missing height", facts.replace("max_height = 3069,", "")),
        ] {
            let mut provider = terrain_of(&lua, &format!("return {{ {source} }}"))?;
            let error = refusal_of(provider(TerrainQuery::Map { winter: false }), name);
            assert!(error.starts_with("[terrain-provider:map.facts]"), "{name}: {error}");
        }
        let mut no_map = terrain_of(&lua, "return 'nothing'")?;
        refused(&mut no_map, TerrainQuery::Map { winter: false }, "map.answer");
        let mut raising = terrain_of(&lua, "error ('no map')")?;
        refused(&mut raising, TerrainQuery::Map { winter: false }, "map.callback");
        Ok(())
    }

    // ---- the host environment provider as the campaign calls it (EEDCS-51, D3): the real bridge and the real answer parsing, with Lua providers of the test's choosing ----

    fn env_of(lua: &Lua, source: &str) -> Result<EnvProvider, Box<dyn std::error::Error>> {
        let state = format!(
            "local function state () local s = {{}} for _, n in ipairs {{ {} }} do s [n] = 0 end s.weather_mode = 1 s.target_weather_mode = 1 s.local_weather_model = false s.weather_increasing = false s.wind_increasing = false return s end",
            ENV_FLOAT_NAMES.iter().map(|name| format!("'{name}'")).collect::<Vec<_>>().join(", ")
        );
        let function: LuaFunction = lua.load(format!("{state} return function (r) {source} end")).eval()?;
        Ok(environment_provider(lua, function)?)
    }

    fn env_request(op: i32) -> EnvRequest {
        EnvRequest { op, ..EnvRequest::default() }
    }

    fn env_refusal(provider: &mut EnvProvider, request: &EnvRequest, marker: &str) {
        let result = provider(request);
        assert!(result.is_err(), "{marker}: the call must be refused, got {result:?}");
        let message = result.err().unwrap_or_default();
        assert!(message.contains(&format!("[env-provider:{marker}]")), "{marker}: {message}");
    }

    #[test]
    fn an_environment_weather_answer_reaches_the_campaign_exactly_and_the_request_reaches_the_provider_by_value() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let mut provider = env_of(
            &lua,
            "assert (r.op == 'weather' and r.x == 1.5 and r.z == -2.5 and r.winter == true)
             return { current_mode = 2, target_mode = 3, transitional_state = 2 ^ -130, simple_mode = 3 }",
        )?;
        let request = EnvRequest {
            x: 1.5,
            z: -2.5,
            winter: 1,
            ..env_request(EECH_ENV_WEATHER)
        };
        let reply = provider(&request)?;
        assert_eq!((reply.current_mode, reply.target_mode, reply.simple_mode), (2, 3, 3));
        assert_eq!(reply.transitional_state.to_bits(), 0x0008_0000, "a subnormal is kept exactly");
        Ok(())
    }

    #[test]
    fn an_environment_step_reaches_the_provider_with_its_signed_zero_camera_and_draw_and_its_answer_is_read_whole() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        // a negative-zero delta, the camera and the draw arrive as Lua numbers, unchanged; the map extents are named
        let mut provider = env_of(
            &lua,
            "assert (r.op == 'step' and 1 / r.delta == -math.huge and r.server == true and r.in_cockpit == false)
             assert (r.camera.x == 1 and r.camera.y == 2 and r.camera.z == 3 and r.lightning_draw == 0.25 and r.map.mid_z == 7)
             local s = state () s.lightning_timer = 31.5
             return { expired = true, strike = { 10, 0, -20 }, new_timer = 31.5, state = s }",
        )?;
        let request = EnvRequest {
            delta: -0.0,
            server: 1,
            have_camera: 1,
            camera: [1.0, 2.0, 3.0],
            have_draw: 1,
            lightning_draw: 0.25,
            map: [0.0, 1.0, 2.0, 3.0, 4.0, 7.0],
            ..env_request(EECH_ENV_STEP)
        };
        let reply = provider(&request)?;
        assert_eq!(
            (reply.need_draw, reply.expired, reply.strike, reply.new_timer),
            (0, 1, [10.0, 0.0, -20.0], 31.5)
        );
        assert_eq!(reply.state.f.get(EECH_ENV_FLOATS - 1).copied(), Some(31.5));
        assert_eq!(reply.state.n, [1, 1, 0, 0, 0]);
        Ok(())
    }

    #[test]
    fn an_environment_step_that_needs_the_draw_is_answered_once_and_a_second_ask_for_it_is_refused() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let mut asking = env_of(&lua, "return { need_draw = true, timer = -0.5 }")?;
        let reply = asking(&EnvRequest {
            server: 1,
            ..env_request(EECH_ENV_STEP)
        })?;
        assert_eq!((reply.need_draw, reply.new_timer), (1, -0.5));
        // asked again WITH the draw it already has, the same answer is a refusal
        env_refusal(
            &mut asking,
            &EnvRequest {
                server: 1,
                have_draw: 1,
                lightning_draw: 0.5,
                ..env_request(EECH_ENV_STEP)
            },
            "step.answer",
        );
        Ok(())
    }

    #[test]
    fn a_server_expiry_without_the_draw_is_refused_and_a_client_expiry_needs_none() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let mut expiring = env_of(&lua, "return { expired = true, strike = { 0, 0, 0 }, new_timer = 0, state = state () }")?;
        env_refusal(
            &mut expiring,
            &EnvRequest {
                server: 1,
                ..env_request(EECH_ENV_STEP)
            },
            "step.answer",
        );
        let reply = expiring(&EnvRequest {
            server: 0,
            ..env_request(EECH_ENV_STEP)
        })?;
        assert_eq!((reply.expired, reply.new_timer), (1, 0.0));
        Ok(())
    }

    #[test]
    fn an_environment_state_that_is_not_a_field_is_refused_with_the_state_marker() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        for (what, mutation) in [
            ("nan", "s.weather_radius = 0 / 0"),
            ("infinity", "s.wind_gusting_value = 1 / 0"),
            ("missing_float", "s.lightning_timer = nil"),
            ("string_float", "s.lightning_timer = '1'"),
            ("mode_out_of_range", "s.weather_mode = 5"),
            ("mode_fraction", "s.target_weather_mode = 1.5"),
            ("numeric_flag", "s.local_weather_model = 1"),
            ("missing_flag", "s.wind_increasing = nil"),
        ] {
            let mut provider = env_of(&lua, &format!("local s = state () {mutation} return {{ state = s }}"))?;
            for op in [EECH_ENV_EXPORT, EECH_ENV_INIT_FIXED, EECH_ENV_INIT_RANDOM] {
                let marker = format!("{}.state", env_op_name(op));
                let result = provider(&env_request(op));
                assert!(result.is_err(), "{what}/{marker}: accepted");
                assert!(
                    result.err().unwrap_or_default().contains(&format!("[env-provider:{marker}]")),
                    "{what}/{marker}"
                );
            }
        }
        // a valid field round-trips, with its flags as 0 or 1
        let mut provider = env_of(&lua, "local s = state () s.weather_increasing = true return { state = s }")?;
        let reply = provider(&env_request(EECH_ENV_EXPORT))?;
        assert_eq!(reply.state.n, [1, 1, 0, 1, 0]);
        Ok(())
    }

    #[test]
    fn a_malformed_environment_answer_is_refused_by_operation_and_nothing_falls_back() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let mut nothing = env_of(&lua, "return 'nothing'")?;
        let mut raising = env_of(&lua, "error ('no field')")?;
        for op in [
            EECH_ENV_IMPORT,
            EECH_ENV_EXPORT,
            EECH_ENV_INIT_RANDOM,
            EECH_ENV_INIT_FIXED,
            EECH_ENV_WEATHER,
            EECH_ENV_WIND,
            EECH_ENV_GUST,
            EECH_ENV_STEP,
        ] {
            let name = env_op_name(op);
            env_refusal(&mut nothing, &env_request(op), &format!("{name}.answer"));
            env_refusal(&mut raising, &env_request(op), &format!("{name}.callback"));
        }
        let mut modes = env_of(&lua, "return { current_mode = 9, target_mode = 1, transitional_state = 0, simple_mode = 1 }")?;
        env_refusal(&mut modes, &env_request(EECH_ENV_WEATHER), "weather.answer");
        let mut speed = env_of(&lua, "return { speed = 0 / 0, vector = { 0, 0, 0 } }")?;
        env_refusal(
            &mut speed,
            &EnvRequest {
                want_vector: 1,
                ..env_request(EECH_ENV_WIND)
            },
            "wind.answer",
        );
        let mut no_vector = env_of(&lua, "return { speed = 3 }")?;
        env_refusal(
            &mut no_vector,
            &EnvRequest {
                want_vector: 1,
                ..env_request(EECH_ENV_WIND)
            },
            "wind.answer",
        );
        assert_eq!(no_vector(&env_request(EECH_ENV_WIND))?.speed, 3.0, "a vector nobody asked for is not required");
        let mut gust = env_of(&lua, "return { modulation = 1 / 0 }")?;
        env_refusal(&mut gust, &env_request(EECH_ENV_GUST), "gust.answer");
        Ok(())
    }

    // ---- the request contract is captured BEFORE the provider is called (EEDCS-51, review ENV-PHYSICAL-2): a provider that edits the request it is handed gains nothing ----

    #[test]
    fn a_provider_that_edits_the_request_cannot_change_what_the_campaign_asked() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        // a wind vector was asked for; the provider clears the flag in the table and answers with a speed only: refused, not a silent zero vector
        let mut wind = env_of(&lua, "r.want_vector = false return { speed = 3 }")?;
        env_refusal(
            &mut wind,
            &EnvRequest {
                want_vector: 1,
                ..env_request(EECH_ENV_WIND)
            },
            "wind.answer",
        );
        // and with the flag as asked, the vector it sent is the one that arrives
        let mut vector = env_of(&lua, "return { speed = 3, vector = { 1, 0, -2 } }")?;
        let reply = vector(&EnvRequest {
            want_vector: 1,
            ..env_request(EECH_ENV_WIND)
        })?;
        assert_eq!((reply.speed, reply.vector), (3.0, [1.0, 0.0, -2.0]));
        // the draw was made; the provider erases it from the table and asks for it again: refused
        let mut again = env_of(&lua, "r.lightning_draw = nil return { need_draw = true, timer = 0 }")?;
        env_refusal(
            &mut again,
            &EnvRequest {
                server: 1,
                have_draw: 1,
                lightning_draw: 0.5,
                ..env_request(EECH_ENV_STEP)
            },
            "step.answer",
        );
        // a client; the provider says it is a server and asks for the draw (which would advance the shared random word): refused
        let mut client = env_of(&lua, "r.server = true return { need_draw = true, timer = 0 }")?;
        env_refusal(
            &mut client,
            &EnvRequest {
                server: 0,
                ..env_request(EECH_ENV_STEP)
            },
            "step.answer",
        );
        // a server's expiry the provider reports as complete by claiming the draw was made
        let mut claimed = env_of(
            &lua,
            "r.lightning_draw = 0.5 return { expired = true, strike = { 0, 0, 0 }, new_timer = 5, state = state () }",
        )?;
        env_refusal(
            &mut claimed,
            &EnvRequest {
                server: 1,
                ..env_request(EECH_ENV_STEP)
            },
            "step.answer",
        );
        // the operation itself is read once: a provider that renames it cannot turn one answer into another
        let mut renamed = env_of(&lua, "r.op = 'gust' return { modulation = 0.25 }")?;
        env_refusal(&mut renamed, &env_request(EECH_ENV_WIND), "wind.answer");
        Ok(())
    }

    #[test]
    fn a_completed_step_must_say_explicitly_whether_it_expired_and_a_draw_needs_an_expiry() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        for (what, source) in [
            ("missing", "return { state = state () }"),
            ("nil", "return { expired = nil, state = state () }"),
            ("number", "return { expired = 0, state = state () }"),
            ("string", "return { expired = 'false', state = state () }"),
        ] {
            let mut provider = env_of(&lua, source)?;
            let message = provider(&env_request(EECH_ENV_STEP)).err().unwrap_or_default();
            assert!(
                message.contains("[env-provider:step.answer]") && message.contains("expired"),
                "{what}: {message}"
            );
        }
        // an explicit false is a completed step with no expiry, for a server and for a client
        let mut calm = env_of(&lua, "return { expired = false, state = state () }")?;
        for server in [0, 1] {
            let reply = calm(&EnvRequest {
                server,
                ..env_request(EECH_ENV_STEP)
            })?;
            assert_eq!((reply.need_draw, reply.expired), (0, 0));
        }
        // after the draw the final answer must complete AND still report the expiry
        env_refusal(
            &mut calm,
            &EnvRequest {
                server: 1,
                have_draw: 1,
                lightning_draw: 0.5,
                ..env_request(EECH_ENV_STEP)
            },
            "step.answer",
        );
        // a client that is asked nothing and expires is accepted without any draw
        let mut expiring = env_of(&lua, "return { expired = true, strike = { 3, 0, 4 }, state = state () }")?;
        let reply = expiring(&EnvRequest {
            server: 0,
            ..env_request(EECH_ENV_STEP)
        })?;
        assert_eq!((reply.expired, reply.strike, reply.new_timer), (1, [3.0, 0.0, 4.0], 0.0));
        // the server's normal order: the question asks for the draw once, the second answer reports the expiry
        let mut server = env_of(&lua, "if r.lightning_draw == nil then return { need_draw = true, timer = -1 } end return { expired = true, strike = { 0, 0, 0 }, new_timer = 60, state = state () }")?;
        let first = server(&EnvRequest {
            server: 1,
            ..env_request(EECH_ENV_STEP)
        })?;
        assert_eq!((first.need_draw, first.new_timer), (1, -1.0));
        let second = server(&EnvRequest {
            server: 1,
            have_draw: 1,
            lightning_draw: 0.5,
            ..env_request(EECH_ENV_STEP)
        })?;
        assert_eq!((second.need_draw, second.expired, second.new_timer), (0, 1, 60.0));
        Ok(())
    }

    // ---- the sector surface enumeration through the Lua bridge (EEDCS-51): the request by value, the answer checked in full ----

    #[test]
    fn a_sector_answer_keeps_the_original_return_value_apart_from_the_flags_and_a_malformed_one_is_refused_by_marker() -> Result<(), Box<dyn std::error::Error>>
    {
        let lua = Lua::new();
        let flags = |set: &[usize]| -> String { (0..62).map(|id| if set.contains(&id) { '1' } else { '0' }).collect() };
        let query = || TerrainQuery::Sector { x: 1.5, z: -2.5, winter: true };
        // the request reaches the provider as op, position and season; the return value (7: 1 + six transitions) is NOT the number of flagged surfaces (2), and both arrive
        let mut provider = terrain_of(
            &lua,
            &format!(
                "assert (r.op == 'sector' and r.x == 1.5 and r.z == -2.5 and r.winter == true and r.hint == nil) return {{ returned = 7, flags = '{}' }}",
                flags(&[9, 46])
            ),
        )?;
        let TerrainAnswer::Sector { returned, flags: read } = provider(query())? else {
            return Err("not a sector answer".into());
        };
        assert_eq!(returned, 7);
        assert_eq!((read.len(), read.iter().filter(|flag| **flag).count()), (62, 2));
        assert_eq!(
            [9usize, 46, 0, 61].map(|id| read.get(id).copied()),
            [Some(true), Some(true), Some(false), Some(false)]
        );
        // a return value equal to the number of flags is the smallest valid one
        let mut single = terrain_of(&lua, &format!("return {{ returned = 1, flags = '{}' }}", flags(&[9])))?;
        assert!(matches!(single(query())?, TerrainAnswer::Sector { returned: 1, .. }));
        // each malformed answer is refused under its own marker
        for (what, source, marker) in [
            (
                "flags too short",
                format!("return {{ returned = 3, flags = '{}' }}", flags(&[9]).chars().take(61).collect::<String>()),
                "sector.flags",
            ),
            (
                "flags too long",
                format!("return {{ returned = 3, flags = '{}0' }}", flags(&[9])),
                "sector.flags",
            ),
            (
                "flags not binary",
                format!("return {{ returned = 3, flags = '{}' }}", flags(&[9]).replacen("0", "2", 1)),
                "sector.flags",
            ),
            ("flags a table", "return { returned = 3, flags = {} }".to_string(), "sector.flags"),
            ("no flags", "return { returned = 3 }".to_string(), "sector.flags"),
            (
                "no surface flagged",
                format!("return {{ returned = 3, flags = '{}' }}", flags(&[])),
                "sector.returned",
            ),
            (
                "return value below the flags",
                format!("return {{ returned = 1, flags = '{}' }}", flags(&[9, 46])),
                "sector.returned",
            ),
            (
                "return value zero",
                format!("return {{ returned = 0, flags = '{}' }}", flags(&[9])),
                "sector.returned",
            ),
            (
                "return value fractional",
                format!("return {{ returned = 2.5, flags = '{}' }}", flags(&[9])),
                "sector.returned",
            ),
            ("return value missing", format!("return {{ flags = '{}' }}", flags(&[9])), "sector.returned"),
            (
                "return value not finite",
                format!("return {{ returned = 1 / 0, flags = '{}' }}", flags(&[9])),
                "sector.returned",
            ),
            ("not a table", "return 'nothing'".to_string(), "sector.answer"),
            ("withheld", "error ('withheld')".to_string(), "sector.callback"),
        ] {
            let mut bad = terrain_of(&lua, &source)?;
            let error = refusal_of(bad(query()), what);
            assert!(error.starts_with(&format!("[terrain-provider:{marker}]")), "{what}: {error}");
        }
        Ok(())
    }

    use eech_engine::TerrainAmbientChild;

    // ---- the ambient sound questions through the Lua bridges (EEDCS-51): the request by value, the answer checked in full under its marker ----

    #[test]
    fn an_ambient_environment_answer_is_four_levels_read_exactly_and_a_malformed_one_is_refused_by_marker() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let request = EnvRequest {
            x: 1.5,
            z: -2.5,
            ambient_y: 100.0,
            ambient_alt: 12.5,
            ambient_cloud_base: 6000.0,
            in_cockpit: 1,
            winter: 1,
            ..env_request(EECH_ENV_AMBIENT)
        };
        // the observations reach the provider as numbers and a boolean; the four levels come back exactly (a subnormal rain level is kept)
        let mut provider = env_of(
            &lua,
            "assert (r.op == 'ambient' and r.x == 1.5 and r.z == -2.5 and r.y == 100 and r.alt == 12.5 and r.cloud_base == 6000 and r.in_cockpit == true and r.winter == true)
             return { rain_light = 2 ^ -130, rain_heavy = 1.5, wind_light = 0.5, wind_heavy = 1 }",
        )?;
        let reply = provider(&request)?;
        assert_eq!(
            reply.ambient_levels.map(f32::to_bits),
            [0x0008_0000, 1.5f32.to_bits(), 0.5f32.to_bits(), 1.0f32.to_bits()]
        );
        // each malformed answer is refused under the ambient marker: not a table, a level missing, not finite, or a wind level outside 0 to 1; a withheld answer under its callback marker
        for (what, source, marker) in [
            ("not a table", "return 'levels'", "ambient.answer"),
            (
                "rain level missing",
                "return { rain_heavy = 0, wind_light = 0, wind_heavy = 0 }",
                "ambient.answer",
            ),
            (
                "rain level not finite",
                "return { rain_light = 1 / 0, rain_heavy = 0, wind_light = 0, wind_heavy = 0 }",
                "ambient.answer",
            ),
            (
                "wind level not a number",
                "return { rain_light = 0, rain_heavy = 0, wind_light = '0', wind_heavy = 0 }",
                "ambient.answer",
            ),
            (
                "wind level above 1",
                "return { rain_light = 0, rain_heavy = 0, wind_light = 1.5, wind_heavy = 0 }",
                "ambient.answer",
            ),
            (
                "wind level below 0",
                "return { rain_light = 0, rain_heavy = 0, wind_light = 0, wind_heavy = -0.5 }",
                "ambient.answer",
            ),
            ("withheld", "error ('withheld')", "ambient.callback"),
        ] {
            let mut bad = env_of(&lua, source)?;
            let result = bad(&request);
            assert!(
                matches!(&result, Err(message) if message.contains(&format!("[env-provider:{marker}]"))),
                "{what}: {result:?}"
            );
        }
        Ok(())
    }

    // ---- the physical lightning queue through the Lua bridge (EEDCS-51): the request by value, the whole queue read exactly, a malformed answer refused by marker ----

    /// A Lua source for a list of 32 strikes where `edit` (Lua) may change the first.
    fn queue_source(first: &str) -> String {
        format!(
            "local q = {{}} for i = 1, 32 do q [i] = {{ in_use = false, type = 0, x = 0, y = 0, z = 0, time_started = 0, time_length = 0, frame = 0 }} end
             local s = q [1] {first}"
        )
    }

    #[test]
    fn a_lightning_add_reaches_the_provider_by_value_and_the_whole_queue_comes_back_exactly() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let request = EnvRequest {
            lightning_type: 2,
            lightning_length: 0.3,
            x: 1500.0,
            z: -2500.0,
            lightning_environment_active: 1,
            lightning_time_of_day: 100.25,
            ..env_request(EECH_ENV_LIGHTNING_ADD)
        };
        let mut provider = env_of(
            &lua,
            &format!(
                "assert (r.op == 'lightning_add' and r.type == 2 and r.length == 0.30000001192092896 and r.x == 1500 and r.z == -2500 and r.environment_active == true and r.time_of_day == 100.25)
                 {}
                 s.in_use = true s.type = 2 s.x = r.x s.y = 6000 s.z = r.z s.time_started = r.time_of_day s.time_length = r.length s.frame = 0
                 return {{ slot = 0, queue = q }}",
                queue_source("")
            ),
        )?;
        let reply = provider(&request)?;
        assert_eq!(reply.lightning_slot, 0);
        assert_eq!(reply.lightning_need_frames, 0);
        let first = reply.lightning_queue[0];
        assert_eq!((first.in_use, first.kind, first.current_frame_number), (1, 2, 0));
        assert_eq!(
            [first.x, first.y, first.z, first.time_started, first.time_length].map(f32::to_bits),
            [1500.0f32, 6000.0, -2500.0, 100.25, 0.3].map(f32::to_bits)
        );
        assert!(reply.lightning_queue.iter().skip(1).all(|strike| strike.in_use == 0));
        // with no active environment nothing is given for the time of day
        let none = EnvRequest {
            lightning_environment_active: 0,
            ..request
        };
        let mut absent = env_of(
            &lua,
            &format!(
                "assert (r.environment_active == false and r.time_of_day == nil) {} return {{ slot = -1, queue = q }}",
                queue_source("")
            ),
        )?;
        assert_eq!(absent(&none)?.lightning_slot, -1);
        Ok(())
    }

    #[test]
    fn a_lightning_update_may_ask_for_the_frame_count_once_and_a_malformed_queue_is_refused_by_marker() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let update = EnvRequest {
            lightning_time_of_day: 10.5,
            ..env_request(EECH_ENV_LIGHTNING_UPDATE)
        };
        // the host needs the frame count: nothing else is read from the answer
        let mut asks = env_of(
            &lua,
            "assert (r.op == 'lightning_update' and r.time_of_day == 10.5 and r.frames == nil) return { need_frames = true }",
        )?;
        let reply = asks(&update)?;
        assert_eq!((reply.lightning_need_frames, reply.lightning_slot), (1, -1));
        // given the count the request carries it, and a second request for it is refused
        let given = EnvRequest {
            lightning_have_frames: 1,
            lightning_frames: 8,
            ..update
        };
        let mut again = env_of(&lua, "return { need_frames = true }")?;
        assert!(matches!(again(&given), Err(message) if message.contains("[env-provider:lightning_update.answer]")));
        let mut done = env_of(
            &lua,
            &format!(
                "assert (r.frames == 8) {} s.in_use = true s.type = 2 s.frame = 4 return {{ queue = q }}",
                queue_source("")
            ),
        )?;
        let reply = done(&given)?;
        assert_eq!(
            (reply.lightning_need_frames, reply.lightning_slot, reply.lightning_queue[0].current_frame_number),
            (0, -1, 4)
        );
        // an add never asks for a frame count
        let add = env_request(EECH_ENV_LIGHTNING_ADD);
        let mut wrong = env_of(&lua, "return { need_frames = true }")?;
        assert!(matches!(wrong(&add), Err(message) if message.contains("[env-provider:lightning_add.answer]")));
        // each malformed add answer is refused under the add marker, a withheld one under its callback marker
        for (what, source, marker) in [
            ("not a table", "return 'queue'".to_string(), "lightning_add.answer"),
            (
                "slot beyond the queue",
                format!("{} return {{ slot = 32, queue = q }}", queue_source("")),
                "lightning_add.answer",
            ),
            ("slot missing", format!("{} return {{ queue = q }}", queue_source("")), "lightning_add.answer"),
            ("queue missing", "return { slot = 0 }".to_string(), "lightning_add.answer"),
            ("a short queue", "return { slot = 0, queue = { {} } }".to_string(), "lightning_add.answer"),
            (
                "a long queue",
                format!("{} q [33] = q [1] return {{ slot = 0, queue = q }}", queue_source("")),
                "lightning_add.answer",
            ),
            (
                "in_use not a boolean",
                format!("{} s.in_use = 1 return {{ slot = 0, queue = q }}", queue_source("")),
                "lightning_add.answer",
            ),
            (
                "a type outside the enum",
                format!("{} s.type = 6 return {{ slot = 0, queue = q }}", queue_source("")),
                "lightning_add.answer",
            ),
            (
                "a number not finite",
                format!("{} s.x = 1 / 0 return {{ slot = 0, queue = q }}", queue_source("")),
                "lightning_add.answer",
            ),
            (
                "a frame that is not an integer",
                format!("{} s.frame = 1.5 return {{ slot = 0, queue = q }}", queue_source("")),
                "lightning_add.answer",
            ),
            ("withheld", "error ('withheld')".to_string(), "lightning_add.callback"),
        ] {
            let mut bad = env_of(&lua, &source)?;
            let result = bad(&add);
            assert!(
                matches!(&result, Err(message) if message.contains(&format!("[env-provider:{marker}]"))),
                "{what}: {result:?}"
            );
        }
        Ok(())
    }

    #[test]
    fn a_sound_lifetime_init_reaches_the_provider_by_value_and_one_finite_lifetime_comes_back_exactly() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let mut request = EnvRequest {
            sound_count: 2,
            sound_looping: 0,
            sound_music: 1,
            ..env_request(EECH_ENV_SOUND_LIFETIME_INIT)
        };
        request.sound_rate[..2].copy_from_slice(&[4, 2]);
        request.sound_size[..2].copy_from_slice(&[6, 5]);
        let mut provider = env_of(
            &lua,
            "assert (r.op == 'sound_lifetime_init' and r.looping == false and r.music == true and #r.samples == 2)
             assert (r.samples [1].rate == 4 and r.samples [1].size == 6 and r.samples [2].rate == 2 and r.samples [2].size == 5)
             return { lifetime = 4 }",
        )?;
        assert_eq!(provider(&request)?.sound_lifetime, 4.0);
        for (_what, source, marker) in [
            ("not a table", "return 4", "sound_lifetime_init.answer"),
            ("a missing lifetime", "return {}", "sound_lifetime_init.answer"),
            ("a negative lifetime", "return { lifetime = -1 }", "sound_lifetime_init.answer"),
            ("a lifetime that is not finite", "return { lifetime = 1 / 0 }", "sound_lifetime_init.answer"),
            ("a lifetime that is a string", "return { lifetime = '4' }", "sound_lifetime_init.answer"),
            ("withheld", "error ('withheld')", "sound_lifetime_init.callback"),
        ] {
            let mut bad = env_of(&lua, source)?;
            env_refusal(&mut bad, &request, marker);
        }
        Ok(())
    }

    #[test]
    fn a_sound_lifetime_update_reaches_the_provider_by_value_and_its_decision_comes_back_exactly() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let request = EnvRequest {
            sound_server: 1,
            sound_valid: 1,
            sound_looping: 0,
            sound_has_handle: 1,
            sound_delta: 2.0,
            sound_lifetime: 1.0,
            sound_valid_lifetime: 0.0,
            ..env_request(EECH_ENV_SOUND_LIFETIME_UPDATE)
        };
        let mut provider = env_of(
            &lua,
            "assert (r.op == 'sound_lifetime_update' and r.server == true and r.valid == true and r.looping == false and r.has_handle == true and r.playing == nil)
             assert (r.delta == 2 and r.lifetime == 1 and r.valid_lifetime == 0)
             return { lifetime = -1, valid_lifetime = 0, expired = true, action = 'kill' }",
        )?;
        let reply = provider(&request)?;
        assert_eq!(
            (reply.sound_lifetime, reply.sound_valid_lifetime, reply.sound_expired, reply.sound_action),
            (-1.0, 0.0, 1, eech_engine::EECH_SOUND_ACTION_KILL)
        );
        // the question whether the handle is playing, then the answer with the campaign's reply in the request
        let mut asking = env_of(&lua, "if r.playing == nil then return { need_playing = true } end assert (r.playing == false) return { lifetime = 1, valid_lifetime = 0, expired = false, action = 'none' }")?;
        assert_eq!(asking(&request)?.sound_need_playing, 1);
        let told = EnvRequest {
            sound_have_playing: 1,
            sound_playing: 0,
            ..request
        };
        let reply = asking(&told)?;
        assert_eq!((reply.sound_need_playing, reply.sound_action, reply.sound_expired), (0, 0, 0));
        for (_what, source, marker) in [
            (
                "an action that is not one of the three",
                "return { lifetime = 1, valid_lifetime = 0, expired = false, action = 'explode' }",
                "sound_lifetime_update.answer",
            ),
            (
                "an action as a number",
                "return { lifetime = 1, valid_lifetime = 0, expired = false, action = 1 }",
                "sound_lifetime_update.answer",
            ),
            (
                "expired as a number",
                "return { lifetime = 1, valid_lifetime = 0, expired = 1, action = 'none' }",
                "sound_lifetime_update.answer",
            ),
            (
                "a lifetime that is not finite",
                "return { lifetime = 0 / 0, valid_lifetime = 0, expired = false, action = 'none' }",
                "sound_lifetime_update.answer",
            ),
            (
                "a valid lifetime missing",
                "return { lifetime = 1, expired = false, action = 'none' }",
                "sound_lifetime_update.answer",
            ),
            ("withheld", "error ('withheld')", "sound_lifetime_update.callback"),
        ] {
            let mut bad = env_of(&lua, source)?;
            env_refusal(&mut bad, &request, marker);
        }
        // the question about the handle where none may be asked: an effect with no handle, a client, an invalid effect, and a handle already said to be playing
        for (what, asked) in [
            (
                "an effect with no handle",
                EnvRequest {
                    sound_has_handle: 0,
                    ..request
                },
            ),
            ("a client routine", EnvRequest { sound_server: 0, ..request }),
            ("an invalid effect", EnvRequest { sound_valid: 0, ..request }),
            (
                "a handle already said to be playing",
                EnvRequest {
                    sound_have_playing: 1,
                    ..request
                },
            ),
        ] {
            let mut ask = env_of(&lua, "return { need_playing = true }")?;
            let result = ask(&asked);
            assert!(
                matches!(&result, Err(message) if message.contains("[env-provider:sound_lifetime_update.answer]")),
                "{what}: {result:?}"
            );
        }
        Ok(())
    }

    #[test]
    fn a_sound_pause_resume_reaches_the_provider_by_value_and_its_decision_comes_back_exactly() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let pause = EnvRequest {
            sound_resume: 0,
            sound_valid: 1,
            sound_member: 1,
            sound_delay: 1.5,
            sound_valid_lifetime: 0.25,
            ..env_request(EECH_ENV_SOUND_PAUSE_RESUME)
        };
        let mut provider = env_of(
            &lua,
            "assert (r.op == 'sound_pause_resume' and r.resume == false and r.valid == true and r.member == true and r.delay == 1.5 and r.valid_lifetime == 0.25)
             return { changed = true, valid = false, valid_lifetime = 1.5, play = false, relink = false }",
        )?;
        let reply = provider(&pause)?;
        assert_eq!(
            (
                reply.sound_changed,
                reply.sound_valid,
                reply.sound_valid_lifetime,
                reply.sound_play,
                reply.sound_relink
            ),
            (1, 0, 1.5, 0, 0)
        );
        // a resume carries no delay; the stored valid lifetime comes back as the exact word it went with, and the host may ask to play and to relink
        let resume = EnvRequest {
            sound_resume: 1,
            sound_valid: 0,
            sound_member: 0,
            sound_delay: 0.0,
            sound_valid_lifetime: -0.0,
            ..env_request(EECH_ENV_SOUND_PAUSE_RESUME)
        };
        let mut resuming = env_of(
            &lua,
            "assert (r.resume == true and r.valid == false and r.member == false and r.delay == nil and 1 / r.valid_lifetime < 0)
             return { changed = true, valid = true, valid_lifetime = r.valid_lifetime, play = true, relink = true }",
        )?;
        let reply = resuming(&resume)?;
        assert_eq!(
            (
                reply.sound_changed,
                reply.sound_valid,
                reply.sound_valid_lifetime.to_bits(),
                reply.sound_play,
                reply.sound_relink
            ),
            (1, 1, (-0.0f32).to_bits(), 1, 1)
        );
        for (_what, source, marker) in [
            (
                "a flag as a number",
                "return { changed = 1, valid = false, valid_lifetime = 1.5, play = false, relink = false }",
                "sound_pause_resume.answer",
            ),
            (
                "a flag missing",
                "return { changed = true, valid = false, valid_lifetime = 1.5, play = false }",
                "sound_pause_resume.answer",
            ),
            (
                "a valid lifetime that is not finite",
                "return { changed = true, valid = false, valid_lifetime = 1 / 0, play = false, relink = false }",
                "sound_pause_resume.answer",
            ),
            (
                "a valid lifetime missing",
                "return { changed = true, valid = false, play = false, relink = false }",
                "sound_pause_resume.answer",
            ),
            ("not a table", "return 1", "sound_pause_resume.answer"),
            ("withheld", "error ('withheld')", "sound_pause_resume.callback"),
        ] {
            let mut bad = env_of(&lua, source)?;
            env_refusal(&mut bad, &pause, marker);
        }
        Ok(())
    }

    const ROTOR_CHANNELS: &str = "channels = { looping = { amplification = 0.5, pitch = 1 }, slap = { amplification = 0.25, pitch = 1 }, turbine1 = { amplification = 0.75, pitch = 1.5 }, \
        turbine2 = { amplification = 1, pitch = 1.25 }, apu = { amplification = 0.125, pitch = 0.5 }, winding = { amplification = 0.75 } }";

    #[test]
    fn a_rotor_sound_question_reaches_the_provider_by_value_and_its_whole_answer_comes_back_exactly() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let local = EnvRequest {
            rotor_player: EECH_ROTOR_PLAYER_LOCAL,
            rotor_have_flight_model: 1,
            rotor_rpm: 50.0,
            rotor_acceleration: [3.0, 4.0, -0.0],
            rotor_left_rpm: 50.0,
            rotor_right_rpm: 60.0,
            rotor_apu_rpm: 5.0,
            ..env_request(EECH_ENV_ROTOR_SOUND)
        };
        let mut provider = env_of(
            &lua,
            &format!(
                "assert (r.op == 'rotor_sound' and r.player == 'local' and r.rpm == 50 and r.flight_model == true)
                 assert (r.acceleration.x == 3 and r.acceleration.y == 4 and 1 / r.acceleration.z < 0)
                 assert (r.left_rpm == 50 and r.right_rpm == 60 and r.apu_rpm == 5)
                 return {{ {ROTOR_CHANNELS}, actions = {{
                   {{ action = 'resume', channel = 'turbine1' }}, {{ action = 'pause', channel = 'turbine2', delay = 0 }}, {{ action = 'resume', channel = 'apu' }},
                   {{ action = 'pause', channel = 'looping', delay = 0 }}, {{ action = 'pause', channel = 'slap', delay = 0 }} }} }}"
            ),
        )?;
        let reply = provider(&local)?;
        assert_eq!(reply.rotor_write, [3, 3, 3, 3, 3, 1]);
        assert_eq!(reply.rotor_amplification, [0.5, 0.25, 0.75, 1.0, 0.125, 0.75]);
        assert_eq!(reply.rotor_pitch, [1.0, 1.0, 1.5, 1.25, 0.5, 0.0]);
        assert_eq!(reply.rotor_action_count, 5);
        assert_eq!(reply.rotor_action_resume[..5], [1, 0, 1, 0, 0]);
        assert_eq!(reply.rotor_action_channel[..5], [2, 3, 4, 0, 1]);
        // an AI helicopter under a global flight model: the acceleration is given, the engine words are not
        let ai = EnvRequest {
            rotor_player: EECH_ROTOR_PLAYER_AI,
            rotor_left_rpm: 0.0,
            rotor_right_rpm: 0.0,
            rotor_apu_rpm: 0.0,
            ..local
        };
        let mut ai_provider = env_of(
            &lua,
            &format!(
                "assert (r.player == 'ai' and r.flight_model == true and r.acceleration.x == 3 and r.left_rpm == nil and r.right_rpm == nil and r.apu_rpm == nil)
                 return {{ {ROTOR_CHANNELS}, actions = {{ {{ action = 'resume', channel = 'looping' }}, {{ action = 'resume', channel = 'slap' }} }} }}"
            ),
        )?;
        assert_eq!(ai_provider(&ai)?.rotor_action_count, 2);
        // no flight model: no acceleration, and a remote helicopter is a role of its own
        let remote = EnvRequest {
            rotor_player: EECH_ROTOR_PLAYER_REMOTE,
            rotor_have_flight_model: 0,
            rotor_acceleration: [0.0; 3],
            ..local
        };
        let mut remote_provider = env_of(
            &lua,
            &format!(
                "assert (r.player == 'remote' and r.flight_model == false and r.acceleration == nil and r.left_rpm == nil)
                 return {{ {ROTOR_CHANNELS}, actions = {{ {{ action = 'pause', channel = 'looping', delay = 0 }}, {{ action = 'pause', channel = 'slap', delay = 0 }} }} }}"
            ),
        )?;
        assert_eq!(remote_provider(&remote)?.rotor_action_resume[..2], [0, 0]);
        // a role that is not the protocol's is refused before the provider is asked
        let mut never = env_of(&lua, "error ('must not be asked')")?;
        assert!(never(&EnvRequest { rotor_player: 9, ..local }).is_err());
        // every damaged answer is refused under its marker, and a withheld one under the callback marker
        let two = "actions = { { action = 'resume', channel = 'looping' }, { action = 'resume', channel = 'slap' } }";
        let channels = ROTOR_CHANNELS;
        for (_what, source, marker) in [
            ("not a table", "return 1".to_string(), "rotor_sound.answer"),
            ("no channels", format!("return {{ {two} }}"), "rotor_sound.answer"),
            (
                "an unknown channel",
                format!(
                    "return {{ {}, {two} }}",
                    channels.replace("channels = {", "channels = { extra = { amplification = 1, pitch = 1 },")
                ),
                "rotor_sound.answer",
            ),
            (
                "a channel missing",
                format!("return {{ {}, {two} }}", channels.replace("slap = { amplification = 0.25, pitch = 1 }, ", "")),
                "rotor_sound.answer",
            ),
            (
                "a pitch on the winding channel",
                format!(
                    "return {{ {}, {two} }}",
                    channels.replace("winding = { amplification = 0.75 }", "winding = { amplification = 0.75, pitch = 1 }")
                ),
                "rotor_sound.answer",
            ),
            (
                "a pitch missing",
                format!(
                    "return {{ {}, {two} }}",
                    channels.replace("looping = { amplification = 0.5, pitch = 1 }", "looping = { amplification = 0.5 }")
                ),
                "rotor_sound.answer",
            ),
            (
                "an amplification missing",
                format!(
                    "return {{ {}, {two} }}",
                    channels.replace("looping = { amplification = 0.5, pitch = 1 }", "looping = { pitch = 1 }")
                ),
                "rotor_sound.answer",
            ),
            (
                "an amplification that is not finite",
                format!("return {{ {}, {two} }}", channels.replace("amplification = 0.5", "amplification = 1 / 0")),
                "rotor_sound.answer",
            ),
            (
                "a field that is not one",
                format!(
                    "return {{ {}, {two} }}",
                    channels.replace("amplification = 0.5, pitch = 1 }", "amplification = 0.5, pitch = 1, volume = 1 }")
                ),
                "rotor_sound.answer",
            ),
            ("no actions", format!("return {{ {channels} }}"), "rotor_sound.answer"),
            (
                "nine actions",
                format!(
                    "return {{ {channels}, actions = {{ {} }} }}",
                    ["{ action = 'resume', channel = 'looping' }"; 9].join(", ")
                ),
                "rotor_sound.answer",
            ),
            (
                "an unknown action",
                format!("return {{ {channels}, actions = {{ {{ action = 'play', channel = 'looping' }} }} }}"),
                "rotor_sound.answer",
            ),
            (
                "a winding request",
                format!("return {{ {channels}, actions = {{ {{ action = 'resume', channel = 'winding' }} }} }}"),
                "rotor_sound.answer",
            ),
            (
                "a resume with a delay",
                format!("return {{ {channels}, actions = {{ {{ action = 'resume', channel = 'looping', delay = 0 }} }} }}"),
                "rotor_sound.answer",
            ),
            (
                "a pause with no delay",
                format!("return {{ {channels}, actions = {{ {{ action = 'pause', channel = 'looping' }} }} }}"),
                "rotor_sound.answer",
            ),
            (
                "a delay that is not finite",
                format!("return {{ {channels}, actions = {{ {{ action = 'pause', channel = 'looping', delay = 0 / 0 }} }} }}"),
                "rotor_sound.answer",
            ),
            (
                "a request with a field that is not one",
                format!("return {{ {channels}, actions = {{ {{ action = 'resume', channel = 'looping', volume = 1 }} }} }}"),
                "rotor_sound.answer",
            ),
            ("withheld", "error ('withheld')".to_string(), "rotor_sound.callback"),
        ] {
            let mut bad = env_of(&lua, &source)?;
            env_refusal(&mut bad, &ai, marker);
        }
        Ok(())
    }

    #[test]
    fn an_ambient_terrain_question_reaches_the_provider_by_value_and_its_amplitudes_are_one_finite_number_per_child() -> Result<(), Box<dyn std::error::Error>>
    {
        let lua = Lua::new();
        let query = |children: Vec<TerrainAmbientChild>| TerrainQuery::Ambient {
            x: 1.5,
            z: -2.5,
            winter: true,
            in_cockpit: false,
            alt: 12.5,
            delta: 0.25,
            children,
        };
        let two = || {
            vec![
                TerrainAmbientChild { class: 0, previous: 0.125 },
                TerrainAmbientChild { class: 5, previous: 0.25 },
            ]
        };
        let mut provider = terrain_of(
            &lua,
            "assert (r.op == 'ambient' and r.x == 1.5 and r.z == -2.5 and r.winter == true and r.alt == 12.5 and r.delta == 0.25 and r.in_cockpit == false and #r.children == 2
                     and r.children [1].class == 0 and r.children [1].previous == 0.125 and r.children [2].class == 5 and r.children [2].previous == 0.25)
             return { amplitudes = { 0.5, 2 ^ -130 } }",
        )?;
        let TerrainAnswer::Ambient { amplitudes } = provider(query(two()))? else {
            return Err("not an ambient answer".into());
        };
        assert_eq!(
            amplitudes.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
            vec![0.5f32.to_bits(), 0x0008_0000]
        );
        // no child: an empty list is the answer
        let mut none = terrain_of(&lua, "return { amplitudes = {} }")?;
        assert!(matches!(none(query(Vec::new()))?, TerrainAnswer::Ambient { amplitudes } if amplitudes.is_empty()));
        // the count of the amplitudes is checked against the count of the children CAPTURED before the provider ran (it may change its request), and each must be finite
        for (what, source, marker) in [
            ("one amplitude for two children", "return { amplitudes = { 0.5 } }", "ambient.amplitudes"),
            (
                "three amplitudes for two children",
                "return { amplitudes = { 0.5, 0.5, 0.5 } }",
                "ambient.amplitudes",
            ),
            ("not finite", "return { amplitudes = { 0.5, 0 / 0 } }", "ambient.amplitudes"),
            ("not numbers", "return { amplitudes = { 0.5, '0.5' } }", "ambient.amplitudes"),
            ("no list", "return {}", "ambient.amplitudes"),
            ("not a table", "return 'silence'", "ambient.answer"),
            ("withheld", "error ('withheld')", "ambient.callback"),
            (
                "a provider that empties its request still owes two",
                "r.children = {} return { amplitudes = {} }",
                "ambient.amplitudes",
            ),
        ] {
            let mut bad = terrain_of(&lua, source)?;
            let error = refusal_of(bad(query(two())), what);
            assert!(error.starts_with(&format!("[terrain-provider:{marker}]")), "{what}: {error}");
        }
        Ok(())
    }

    // ---- the ground-contact operations (EEDCS-48, D3 G2): the real ground bridge and answer parsing, with Lua providers of the test's choosing ----

    const WORLD: [f32; 4] = [-10.5, -20.5, 3000.25, 4000.75];

    fn above_of(position: [f32; 3], hint: Option<&str>) -> TerrainQuery {
        TerrainQuery::GroundAbove {
            position,
            world: WORLD,
            hint: hint.map(str::to_owned),
            winter: false,
        }
    }

    fn below_of(position: [f32; 3], hint: Option<&str>) -> TerrainQuery {
        TerrainQuery::GroundBelow {
            position,
            world: WORLD,
            hint: hint.map(str::to_owned),
            winter: true,
        }
    }

    fn impact_of(above: [f32; 3], below: [f32; 3], hint: Option<&str>) -> TerrainQuery {
        TerrainQuery::GroundImpact {
            above,
            below,
            world: WORLD,
            hint: hint.map(str::to_owned),
            winter: false,
        }
    }

    const RECORD: &str = "{ last_elevation = 12.5, normal = { 0, 1, 0 }, type = 9, stored = true, hint = 'next' }";

    fn the_record() -> TerrainRecord {
        TerrainRecord {
            last_elevation: 12.5,
            normal: [0.0, 1.0, 0.0],
            kind: 9,
            hint: Some("next".to_string()),
            stored: true,
        }
    }

    #[test]
    fn a_ground_question_reaches_the_provider_by_value_and_its_answer_comes_back_exactly() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        // above: the request is the position, the campaign's world area and the record's hint; the answer's negative zero and subnormal survive bit for bit
        let mut above = terrain_of(
            &lua,
            "assert (r.op == 'ground_above' and r.x == 1.5 and r.y == -2.5 and r.z == 3.75 and r.hint == 'previous' and r.winter == false)
             assert (r.world.min_x == -10.5 and r.world.min_z == -20.5 and r.world.max_x == 3000.25 and r.world.max_z == 4000.75)
             return { above = true, queried = true, record = { last_elevation = -1 / (1 / 0), normal = { 2 ^ -130, 1, 0 }, type = 0, stored = false } }",
        )?;
        let TerrainAnswer::GroundPredicate { value, record: Some(record) } = above(above_of([1.5, -2.5, 3.75], Some("previous")))? else {
            return Err("not a ground predicate with a record".into());
        };
        assert!(value);
        assert_eq!(record.last_elevation.to_bits(), 0x8000_0000, "a negative zero is kept");
        assert_eq!(record.normal.map(f32::to_bits), [0x0008_0000, 1.0f32.to_bits(), 0]);
        assert_eq!(
            (record.kind, record.hint.as_deref(), record.stored),
            (0, None, false),
            "no hint is None, and stored is the provider's word"
        );
        // below: the same request shape under its own op, in the other season; a decided answer carries no record
        let mut below = terrain_of(
            &lua,
            "assert (r.op == 'ground_below' and r.x == 5 and r.y == 6 and r.z == 7 and r.hint == nil and r.winter == true and r.above == nil)
             return { below = false, queried = false }",
        )?;
        assert_eq!(
            below(below_of([5.0, 6.0, 7.0], None))?,
            TerrainAnswer::GroundPredicate { value: false, record: None }
        );
        // impact: both endpoints, the world area and the hint; every output crosses
        let mut impact = terrain_of(
            &lua,
            &format!(
                "assert (r.op == 'ground_impact' and r.above.x == 1 and r.above.y == 2 and r.above.z == 3 and r.below.x == 4 and r.below.y == 5 and r.below.z == 6)
                 assert (r.hint == 'h' and r.x == nil and r.world.max_z == 4000.75)
                 return {{ above = {{ x = 1, y = 2.5, z = 3 }}, below = {{ x = 4, y = 4.75, z = 6 }}, impact = {{ x = 4, y = 4.5, z = 6 }}, type = 9, record = {RECORD} }}"
            ),
        )?;
        assert_eq!(
            impact(impact_of([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], Some("h")))?,
            TerrainAnswer::GroundImpact {
                above: [1.0, 2.5, 3.0],
                below: [4.0, 4.75, 6.0],
                impact: [4.0, 4.5, 6.0],
                record: the_record(),
            }
        );
        Ok(())
    }

    #[test]
    fn a_malformed_ground_answer_is_refused_before_the_campaign_reads_any_of_it() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let impact_with = |body: &str| {
            format!("return {{ above = {{ x = 1, y = 2, z = 3 }}, below = {{ x = 1, y = 2, z = 3 }}, impact = {{ x = 1, y = 2, z = 3 }}, type = 9, record = {RECORD}{body} }}")
        };
        // (provider source, query, the marker)
        let cases: Vec<(String, TerrainQuery, &str)> = vec![
            ("return nil".into(), above_of([0.0; 3], None), "ground_above.answer"),
            ("return 'yes'".into(), below_of([0.0; 3], None), "ground_below.answer"),
            ("error ('no ground')".into(), impact_of([0.0; 3], [1.0; 3], None), "ground_impact.callback"),
            // the predicate is a boolean, not a number or a string; `queried` is a boolean
            ("return { above = 1, queried = false }".into(), above_of([0.0; 3], None), "ground_above.answer"),
            (
                "return { below = 'true', queried = false }".into(),
                below_of([0.0; 3], None),
                "ground_below.answer",
            ),
            ("return { above = true }".into(), above_of([0.0; 3], None), "ground_above.answer"),
            // the answer of the other op is not this op's answer
            (
                "return { below = true, queried = false }".into(),
                above_of([0.0; 3], None),
                "ground_above.answer",
            ),
            // a record is present exactly when the terrain was read
            (
                format!("return {{ above = true, queried = false, record = {RECORD} }}"),
                above_of([0.0; 3], None),
                "ground_above.record",
            ),
            (
                "return { above = true, queried = true }".into(),
                above_of([0.0; 3], None),
                "ground_above.record",
            ),
            // the record is complete and exact: strings are not coerced, NaN is refused, stored is a boolean, the type is an integer in range
            (
                format!("return {{ above = true, queried = true, record = {} }}", RECORD.replace("12.5", "'12.5'")),
                above_of([0.0; 3], None),
                "ground_above.record",
            ),
            (
                format!("return {{ above = true, queried = true, record = {} }}", RECORD.replace("12.5", "0 / 0")),
                above_of([0.0; 3], None),
                "ground_above.record",
            ),
            (
                format!(
                    "return {{ above = true, queried = true, record = {} }}",
                    RECORD.replace("{ 0, 1, 0 }", "{ 0, 1 }")
                ),
                above_of([0.0; 3], None),
                "ground_above.record",
            ),
            (
                format!(
                    "return {{ above = true, queried = true, record = {} }}",
                    RECORD.replace("{ 0, 1, 0 }", "{ 0, '1', 0 }")
                ),
                above_of([0.0; 3], None),
                "ground_above.record",
            ),
            (
                format!(
                    "return {{ below = true, queried = true, record = {} }}",
                    RECORD.replace("stored = true", "stored = 1")
                ),
                below_of([0.0; 3], None),
                "ground_below.record",
            ),
            (
                format!(
                    "return {{ below = true, queried = true, record = {} }}",
                    RECORD.replace("stored = true", "stored = 'true'")
                ),
                below_of([0.0; 3], None),
                "ground_below.record",
            ),
            (
                format!(
                    "return {{ below = true, queried = true, record = {} }}",
                    RECORD.replace("type = 9", "type = 62")
                ),
                below_of([0.0; 3], None),
                "ground_below.type",
            ),
            (
                format!(
                    "return {{ below = true, queried = true, record = {} }}",
                    RECORD.replace("type = 9", "type = 1.5")
                ),
                below_of([0.0; 3], None),
                "ground_below.type",
            ),
            (
                format!("return {{ below = true, queried = true, record = {} }}", RECORD.replace("'next'", "''")),
                below_of([0.0; 3], None),
                "ground_below.record",
            ),
            // the impact: every endpoint and the impact point are three finite numbers, the type is the record's, the record is whole
            (
                impact_with("").replace("impact = { x = 1, y = 2, z = 3 }, ", ""),
                impact_of([0.0; 3], [1.0; 3], None),
                "ground_impact.position",
            ),
            (
                impact_with("").replace("above = { x = 1, y = 2, z = 3 }", "above = { x = 1, y = '2', z = 3 }"),
                impact_of([0.0; 3], [1.0; 3], None),
                "ground_impact.position",
            ),
            (
                impact_with("").replace("below = { x = 1, y = 2, z = 3 }", "below = { x = 1, y = 2 }"),
                impact_of([0.0; 3], [1.0; 3], None),
                "ground_impact.position",
            ),
            (
                impact_with("").replace("impact = { x = 1, y = 2, z = 3 }", "impact = { x = 1, y = 2, z = 1 / 0 }"),
                impact_of([0.0; 3], [1.0; 3], None),
                "ground_impact.position",
            ),
            (
                impact_with("").replace("type = 9, record", "type = 8, record"),
                impact_of([0.0; 3], [1.0; 3], None),
                "ground_impact.type",
            ),
            (
                impact_with("").replace("type = 9, record", "record"),
                impact_of([0.0; 3], [1.0; 3], None),
                "ground_impact.type",
            ),
            (
                impact_with("").replace(RECORD, "5"),
                impact_of([0.0; 3], [1.0; 3], None),
                "ground_impact.record",
            ),
        ];
        for (index, (source, query, marker)) in cases.into_iter().enumerate() {
            let mut provider = terrain_of(&lua, &source)?;
            let error = refusal_of(provider(query), &format!("case {index}: {source}"));
            assert!(error.starts_with(&format!("[terrain-provider:{marker}]")), "case {index} ({source}): {error}");
        }
        // a record that does not say whether it stored a surface stored one, as a point answer does; with no hint it is a host that keeps no per-record state
        let mut bare = terrain_of(
            &lua,
            "return { below = false, queried = true, record = { last_elevation = 1, normal = { 0, 1, 0 }, type = 9 } }",
        )?;
        assert!(matches!(
            bare(below_of([0.0; 3], None))?,
            TerrainAnswer::GroundPredicate {
                value: false,
                record: Some(TerrainRecord { stored: true, hint: None, .. })
            }
        ));
        // a provider can answer a position with a plain Lua integer: that is a number, and exact
        let mut integers = terrain_of(
            &lua,
            "return { above = true, queried = true, record = { last_elevation = 3, normal = { 0, 1, 0 }, type = 2.0, stored = true } }",
        )?;
        assert!(matches!(
            integers(above_of([0.0; 3], None))?,
            TerrainAnswer::GroundPredicate { value: true, record: Some(TerrainRecord { last_elevation, kind: 2, .. }) } if last_elevation == 3.0
        ));
        Ok(())
    }

    // ---- the host's sample metadata provider as a Lua function (EEDCS-51, D3): the request it is given and the answer it may give ----

    fn catalogue_provider_of(lua: &Lua, body: &str) -> Result<SoundCatalogueProvider, Box<dyn std::error::Error>> {
        let function: LuaFunction = lua.load(format!("return function (request) {body} end")).eval()?;
        Ok(sound_catalogue_provider(lua, function))
    }

    fn catalogue_request() -> SoundCatalogueRequest {
        SoundCatalogueRequest {
            slots: 2122,
            descriptors: vec![(94, "lstrike.pcm".to_string()), (2087, "ch46e_windup_ext.pcm".to_string())],
        }
    }

    #[test]
    fn the_catalogue_question_reaches_the_hosts_lua_by_value_and_its_answer_is_read_entry_by_entry() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let mut provider = catalogue_provider_of(
            &lua,
            "assert (request.op == 'sound_catalogue' and request.slots == 2122 and #request.entries == 2) \
             assert (request.entries[1].index == 94 and request.entries[1].name == 'lstrike.pcm') \
             assert (request.entries[2].index == 2087 and request.entries[2].name == 'ch46e_windup_ext.pcm') \
             return { header_present = true, header_count = 7, entries = { \
               { rate = 22050, original_rate = 22050, matched = true, size = 96 }, \
               { rate = 11025.0, original_rate = 11025, matched = false, size = 0 } } }",
        )?;
        let reply = provider(&catalogue_request())?;
        assert!(reply.header_present);
        assert_eq!(reply.header_count, 7);
        assert_eq!(
            reply.entries,
            vec![
                SoundCatalogueEntry {
                    rate: 22050,
                    original_rate: 22050,
                    matched: true,
                    size: 96
                },
                SoundCatalogueEntry {
                    rate: 11025,
                    original_rate: 11025,
                    matched: false,
                    size: 0
                }
            ]
        );
        Ok(())
    }

    #[test]
    fn a_catalogue_answer_of_the_wrong_kind_or_shape_is_refused_by_name() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let entry = "{ rate = 22050, original_rate = 22050, matched = true, size = 1 }";
        let cases: Vec<(&str, String)> = vec![
            ("nothing", "return nil".to_string()),
            ("a string", "return 'yes'".to_string()),
            ("a count that is not whole", format!("return {{ header_present = true, header_count = 1.5, entries = {{ {entry}, {entry} }} }}")),
            ("a count that does not fit", format!("return {{ header_present = true, header_count = 4294967296, entries = {{ {entry}, {entry} }} }}")),
            ("a present flag that is a number", format!("return {{ header_present = 1, header_count = 1, entries = {{ {entry}, {entry} }} }}")),
            ("no entries table", "return { header_present = true, header_count = 1 }".to_string()),
            ("an entry that is not a table", format!("return {{ header_present = true, header_count = 1, entries = {{ {entry}, 5 }} }}")),
            (
                "a rate that is a string",
                format!("return {{ header_present = true, header_count = 1, entries = {{ {entry}, {{ rate = '22050', original_rate = 1, matched = true, size = 1 }} }} }}"),
            ),
            (
                "a matched flag that is a number",
                format!("return {{ header_present = true, header_count = 1, entries = {{ {entry}, {{ rate = 1, original_rate = 1, matched = 1, size = 1 }} }} }}"),
            ),
            (
                "a missing size",
                format!("return {{ header_present = true, header_count = 1, entries = {{ {entry}, {{ rate = 1, original_rate = 1, matched = false }} }} }}"),
            ),
            (
                "a length that is not a number",
                format!("return {{ header_present = true, header_count = 1, entries = {{ {entry}, {{ rate = 1, original_rate = 1, matched = true, size = 0/0 }} }} }}"),
            ),
        ];
        for (label, body) in cases {
            let mut provider = catalogue_provider_of(&lua, &body)?;
            let Err(error) = provider(&catalogue_request()) else {
                return Err(format!("{label}: the answer was accepted").into());
            };
            assert!(error.starts_with("[sound-data-provider:answer]"), "{label}: {error}");
        }
        Ok(())
    }

    #[test]
    fn a_catalogue_provider_that_raises_is_refused_by_name_and_a_provider_that_is_not_a_function_is_refused_before_boot(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let mut provider = catalogue_provider_of(&lua, "error ('the host has no sound files', 0)")?;
        let Err(error) = provider(&catalogue_request()) else {
            return Err("a raising provider answered".into());
        };
        assert!(
            error.starts_with("[sound-data-provider:callback]") && error.contains("the host has no sound files"),
            "{error}"
        );
        // an old host's environment provider answers every operation it knows and raises on one it does not: that is the same refusal, named
        let mut old_host = catalogue_provider_of(
            &lua,
            "if request.op ~= 'weather' then error ('host.env: unknown op ' .. request.op, 0) end return {}",
        )?;
        let Err(error) = old_host(&catalogue_request()) else {
            return Err("a host that does not know the operation answered".into());
        };
        assert!(
            error.starts_with("[sound-data-provider:callback]") && error.contains("unknown op sound_catalogue"),
            "{error}"
        );
        // the boot's choice of the function: its own `sound_data`, else the environment provider; one that is given and is not a function stops the boot with the stable marker before the engine is touched
        let environment: LuaFunction = lua.load("return function () return 'environment' end").eval()?;
        let empty: LuaTable = lua.load("return {}").eval()?;
        assert_eq!(sound_data_function(&empty, &environment)?.call::<String>(())?, "environment");
        let separate: LuaTable = lua.load("return { sound_data = function () return 'separate' end }").eval()?;
        assert_eq!(sound_data_function(&separate, &environment)?.call::<String>(())?, "separate");
        for given in ["5", "'text'", "{}", "true"] {
            let config: LuaTable = lua.load(format!("return {{ sound_data = {given} }}")).eval()?;
            match sound_data_function(&config, &environment) {
                Ok(_) => return Err(format!("a sample metadata provider {given} was accepted").into()),
                Err(error) => assert!(error.to_string().contains("[sound-data-provider:missing]"), "{error}"),
            }
        }
        Ok(())
    }

    // ---- the host's sound backend availability provider as a Lua function (EEDCS-51, D3): the request it is given and the answer it may give ----

    fn backend_provider_of(lua: &Lua, body: &str) -> Result<SoundBackendProvider, Box<dyn std::error::Error>> {
        let function: LuaFunction = lua.load(format!("return function (request) {body} end")).eval()?;
        Ok(sound_backend_provider(lua, function))
    }

    fn backend_request() -> SoundBackendRequest {
        SoundBackendRequest {
            phase: SOUND_BACKEND_PHASE_INITIALISE,
            no_sound: false,
            dedicated: true,
            load_samples: true,
        }
    }

    #[test]
    fn the_backend_question_reaches_the_hosts_lua_by_value_and_only_a_boolean_answer_is_read() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let mut provider = backend_provider_of(
            &lua,
            "assert (request.op == 'sound_backend' and request.phase == 'initialise') \
             assert (request.no_sound == false and request.dedicated == true and request.load_samples == true) \
             return { available = false }",
        )?;
        assert!(!provider(&backend_request())?.available);
        // the host's answer is carried as it is, whatever it is: the campaign decides what it accepts
        let mut available = backend_provider_of(&lua, "return { available = true }")?;
        assert!(available(&backend_request())?.available);
        for (label, body) in [
            ("a nil", "return nil"),
            ("a number", "return 5"),
            ("a string", "return 'yes'"),
            ("no field", "return {}"),
            ("a number as the flag", "return { available = 0 }"),
            ("a string as the flag", "return { available = 'false' }"),
        ] {
            let mut provider = backend_provider_of(&lua, body)?;
            let Err(error) = provider(&backend_request()) else {
                return Err(format!("{label}: an unreadable answer was accepted").into());
            };
            assert!(error.starts_with("[sound-backend-provider:answer]"), "{label}: {error}");
        }
        Ok(())
    }

    #[test]
    fn a_backend_provider_that_raises_or_is_not_a_function_is_refused_by_name_and_a_foreign_phase_is_never_sent() -> Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();
        let mut provider = backend_provider_of(&lua, "error ('the host has no audio', 0)")?;
        let Err(error) = provider(&backend_request()) else {
            return Err("a raising provider answered".into());
        };
        assert!(
            error.starts_with("[sound-backend-provider:callback]") && error.contains("the host has no audio"),
            "{error}"
        );
        // a host whose environment provider does not know the operation is the same named refusal
        let mut old_host = backend_provider_of(
            &lua,
            "if request.op ~= 'weather' then error ('host.env: unknown op ' .. request.op, 0) end return {}",
        )?;
        let Err(error) = old_host(&backend_request()) else {
            return Err("a host that does not know the operation answered".into());
        };
        assert!(
            error.starts_with("[sound-backend-provider:callback]") && error.contains("unknown op sound_backend"),
            "{error}"
        );
        let mut calls = backend_provider_of(&lua, "return { available = false }")?;
        let Err(error) = calls(&SoundBackendRequest { phase: 2, ..backend_request() }) else {
            return Err("a foreign phase was sent".into());
        };
        assert!(error.starts_with("[sound-backend-provider:request]"), "{error}");
        // the boot's choice of the function: its own `sound_backend`, else the environment provider; one that is given and is not a function stops the boot with the stable marker before the engine is touched
        let environment: LuaFunction = lua.load("return function () return 'environment' end").eval()?;
        let empty: LuaTable = lua.load("return {}").eval()?;
        assert_eq!(sound_backend_function(&empty, &environment)?.call::<String>(())?, "environment");
        let separate: LuaTable = lua.load("return { sound_backend = function () return 'separate' end }").eval()?;
        assert_eq!(sound_backend_function(&separate, &environment)?.call::<String>(())?, "separate");
        for given in ["5", "'text'", "{}", "true"] {
            let config: LuaTable = lua.load(format!("return {{ sound_backend = {given} }}")).eval()?;
            match sound_backend_function(&config, &environment) {
                Ok(_) => return Err(format!("a sound backend provider {given} was accepted").into()),
                Err(error) => assert!(error.to_string().contains("[sound-backend-provider:missing]"), "{error}"),
            }
        }
        Ok(())
    }
}
