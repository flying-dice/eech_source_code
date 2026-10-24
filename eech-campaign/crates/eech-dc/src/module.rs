//! The Lua surface of the module.

use eech_engine::{
    ArrivalAnswer, Engine, EngineConfig, EngineError, Gunship, HelicopterGuidanceAnswer, HelicopterQuery, LossDecision, LossDescriptor, MemberDescriptor,
    MovementMode, MovementResult, MovementStep, Side, SupplyReachDescriptor, VehiclePose,
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
    let shape = || {
        format!(
            "{field} must be {}",
            if N == 1 {
                "a finite number".to_owned()
            } else {
                format!("{N} finite numbers")
            }
        )
    };
    let text = text.ok_or_else(shape)?;
    let mut parts = text.split_whitespace();
    let mut numbers = [0.0f32; N];
    for slot in &mut numbers {
        *slot = parts.next().and_then(|part| part.parse::<f64>().ok()).and_then(nearest_f32).ok_or_else(shape)?;
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
            let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
            let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
            let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
            let mut mover = |step| call_move_vehicle(lua, &move_vehicle, step);
            this.engine
                .frame(milliseconds, &mut binder, &mut observer, &mut reach, &mut mover)
                .map_err(lua_error)
        });
        // TEST-ONLY (feature `loss-fixture`, EEDCS-41): not part of the shipping module.
        #[cfg(feature = "loss-fixture")]
        methods.add_method("loss_fixture", |lua, this, entity_id: i32| {
            let bind_mobile = this.bind_mobile.clone();
            let observe_loss = this.observe_loss.clone();
            let supply_reached = this.supply_reached.clone();
            let move_vehicle = this.move_vehicle.clone();
            let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
            let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
            let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
            let mut mover = |step| call_move_vehicle(lua, &move_vehicle, step);
            this.engine
                .loss_fixture(entity_id, &mut binder, &mut observer, &mut reach, &mut mover)
                .map_err(lua_error)
        });
        // TEST-ONLY (feature `loss-fixture`, EEDCS-41): releases the tasks targeting a member's group (setup); not part of the shipping module.
        #[cfg(feature = "loss-fixture")]
        methods.add_method("loss_release", |lua, this, entity_id: i32| {
            let bind_mobile = this.bind_mobile.clone();
            let observe_loss = this.observe_loss.clone();
            let supply_reached = this.supply_reached.clone();
            let move_vehicle = this.move_vehicle.clone();
            let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
            let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
            let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
            let mut mover = |step| call_move_vehicle(lua, &move_vehicle, step);
            this.engine
                .loss_release(entity_id, &mut binder, &mut observer, &mut reach, &mut mover)
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
                let mut binder = |descriptor| call_bind_mobile(lua, &bind_mobile, descriptor);
                let mut observer = |descriptor| call_observe_loss(lua, &observe_loss, descriptor);
                let mut reach = |descriptor| call_supply_reached(lua, &supply_reached, descriptor);
                let mut mover = |step| call_move_vehicle(lua, &move_vehicle, step);
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
    let engine = Engine::boot(&config, &mut binder, &mut observer, &mut reach, &mut mover).map_err(lua_error)?;
    Ok(LuaEngine {
        engine,
        bind_mobile,
        observe_loss,
        supply_reached,
        move_vehicle,
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
