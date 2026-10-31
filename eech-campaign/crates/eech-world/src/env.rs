//! The reference host's weather and wind field (EEDCS-51, D3): the physical environment the campaign asks about and the host evolves, owned by the host, independently of the
//! campaign DLL. The arithmetic is EECH's own, kept in C (`csrc/env_ref.c`: the original weather ring model, the wind zone and its gust modulation, the initialisation
//! and the physical part of the per-update evolution, with the original types and expression order); this module owns the field and offers the campaign-facing vocabulary as a
//! Lua provider, `host.env_provider ()`, to be given to `dc.boot` as `environment = ...`.
//!
//! The provider is one function taking a request table `{ op = ..., winter = <boolean>, ... }` and answering a table. The campaign keeps no physical weather of its own: its
//! session carries a read-only mirror of what these answers say, which it refreshes from them (docs/d3-environment-provider.md has the full vocabulary).
//!
//! - `import { state }`: the host takes the campaign's field as given (the session's defaults when it is created; a client's or a loaded campaign's weather).
//! - `export {}`: `{ state }`.
//! - `init_random { draws, config, map }`: the original's random initialisation from the ten draws the campaign made, in the original's order; `{ state }`.
//! - `init_fixed { mode }`: the original's fixed weather; `{ state }`.
//! - `weather { x, z }`: `{ current_mode, target_mode, transitional_state, simple_mode }`.
//! - `wind { x, z, want_vector }`: `{ speed, vector }`.
//! - `gust { phase }`: `{ modulation }`.
//! - `step { delta, server, in_cockpit, camera?, lightning_draw?, map }`: the physical update of one session update. `{ need_draw = true, timer }` (the field is NOT changed) when the
//!   server's lightning timer has expired and the next timer needs the one draw of the shared random word, which only the campaign may make; the campaign then asks again with
//!   `lightning_draw`. Otherwise `{ state, expired, strike?, new_timer? }`.
//!
//! The request's numbers are read as the exact bits Lua holds (a negative zero and a subnormal are not erased) and must be exact single-precision values; anything else is an
//! error with a message and changes nothing. There is no other solver to fall back to.

use crate::terrain::{coordinate_reader, CoordinateReader};
use mlua::prelude::*;
use std::cell::RefCell;
use std::ffi::c_char;
use std::rc::Rc;

pub const FLOATS: usize = 23;
pub const INTS: usize = 5;
pub const DRAWS: usize = 10;

/// The names of the field's floats and integers in the tables of the protocol, in the order of csrc/env_ref.h (the order of the engine's `campaign_env_state`). The last three
/// integers are booleans in the tables. These are the same names, in the same order, as the campaign module's bridge (eech-dc `ENV_FLOAT_NAMES`/`ENV_INT_NAMES`).
pub const FLOAT_NAMES: [&str; FLOATS] = [
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

pub const INT_NAMES: [&str; INTS] = [
    "weather_mode",
    "target_weather_mode",
    "local_weather_model",
    "weather_increasing",
    "wind_increasing",
];

const CONFIG_NAMES: [&str; 6] = [
    "rain_radius_minimum",
    "rain_radius_maximum",
    "rain_speed",
    "wind_radius_minimum",
    "wind_radius_maximum",
    "wind_speed",
];
const MAP_NAMES: [&str; 6] = ["min_x", "max_x", "min_z", "max_z", "mid_x", "mid_z"];

/// `env_ref` (csrc/env_ref.h): opaque.
#[repr(C)]
pub struct EnvRef {
    _private: [u8; 0],
}

extern "C" {
    fn env_ref_new() -> *mut EnvRef;
    fn env_ref_free(field: *mut EnvRef);
    fn env_ref_import(field: *mut EnvRef, f: *const f32, n: *const i32, error: *mut c_char, error_size: usize) -> i32;
    fn env_ref_export(field: *const EnvRef, f: *mut f32, n: *mut i32, error: *mut c_char, error_size: usize) -> i32;
    fn env_ref_init_random(field: *mut EnvRef, draw: *const f64, config: *const f32, map: *const f32, error: *mut c_char, error_size: usize) -> i32;
    fn env_ref_init_fixed(field: *mut EnvRef, mode: i32, error: *mut c_char, error_size: usize) -> i32;
    fn env_ref_weather(
        field: *const EnvRef,
        x: f32,
        z: f32,
        winter: i32,
        modes: *mut i32,
        transitional_state: *mut f32,
        error: *mut c_char,
        error_size: usize,
    ) -> i32;
    fn env_ref_wind(field: *const EnvRef, x: f32, z: f32, want_vector: i32, speed: *mut f32, vector: *mut f32, error: *mut c_char, error_size: usize) -> i32;
    fn env_ref_gust(phase: f32, modulation: *mut f32, error: *mut c_char, error_size: usize) -> i32;
    fn env_ref_step(
        field: *mut EnvRef,
        in_f: *const f32,
        in_i: *const i32,
        draw: *const f64,
        out_i: *mut i32,
        out_f: *mut f32,
        error: *mut c_char,
        error_size: usize,
    ) -> i32;
}

const ERROR_SIZE: usize = 256;

fn message(buffer: &[u8]) -> String {
    let end = buffer.iter().position(|byte| *byte == 0).unwrap_or(buffer.len());
    String::from_utf8_lossy(&buffer[..end]).into_owned()
}

/// The field's numbers: the floats, then the integers (the modes, then the three flags as 0 or 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    pub f: [f32; FLOATS],
    pub n: [i32; INTS],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weather {
    pub current_mode: i32,
    pub target_mode: i32,
    pub simple_mode: i32,
    pub transitional_state: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wind {
    pub speed: f32,
    pub vector: [f32; 3],
}

/// One physical update's inputs: `f = { delta, camera x, y, z, map min x, max x, min z, max z, mid x, mid z }`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepInput {
    pub f: [f32; 10],
    pub winter: bool,
    pub server: bool,
    pub in_cockpit: bool,
    pub have_camera: bool,
    pub draw: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StepOutcome {
    /// the server's lightning timer expired and the next timer needs the draw: nothing was changed; the timer is what the decrement left
    NeedDraw {
        timer: f32,
    },
    Done {
        expired: bool,
        strike: [f32; 3],
        new_timer: f32,
    },
}

/// The host's field: owned here, the C kernels operate on it.
pub struct Field {
    raw: *mut EnvRef,
}

impl Drop for Field {
    fn drop(&mut self) {
        // SAFETY: `raw` came from `env_ref_new` and is freed exactly once.
        unsafe { env_ref_free(self.raw) }
    }
}

impl Field {
    pub fn new() -> Result<Field, String> {
        // SAFETY: a plain allocation.
        let raw = unsafe { env_ref_new() };
        if raw.is_null() {
            Err("host.env: out of memory".to_string())
        } else {
            Ok(Field { raw })
        }
    }

    pub fn import(&mut self, state: &State) -> Result<(), String> {
        let mut error = [0u8; ERROR_SIZE];
        // SAFETY: the arrays have the lengths the kernel reads; `error` is writable for its size.
        let status = unsafe { env_ref_import(self.raw, state.f.as_ptr(), state.n.as_ptr(), error.as_mut_ptr().cast(), ERROR_SIZE) };
        if status == 0 {
            Ok(())
        } else {
            Err(message(&error))
        }
    }

    pub fn export(&self) -> Result<State, String> {
        let mut error = [0u8; ERROR_SIZE];
        let mut state = State {
            f: [0.0; FLOATS],
            n: [0; INTS],
        };
        // SAFETY: as above.
        let status = unsafe { env_ref_export(self.raw, state.f.as_mut_ptr(), state.n.as_mut_ptr(), error.as_mut_ptr().cast(), ERROR_SIZE) };
        if status == 0 {
            Ok(state)
        } else {
            Err(message(&error))
        }
    }

    pub fn init_random(&mut self, draw: &[f64; DRAWS], config: &[f32; 6], map: &[f32; 6]) -> Result<(), String> {
        let mut error = [0u8; ERROR_SIZE];
        // SAFETY: as above.
        let status = unsafe { env_ref_init_random(self.raw, draw.as_ptr(), config.as_ptr(), map.as_ptr(), error.as_mut_ptr().cast(), ERROR_SIZE) };
        if status == 0 {
            Ok(())
        } else {
            Err(message(&error))
        }
    }

    pub fn init_fixed(&mut self, mode: i32) -> Result<(), String> {
        let mut error = [0u8; ERROR_SIZE];
        // SAFETY: as above.
        let status = unsafe { env_ref_init_fixed(self.raw, mode, error.as_mut_ptr().cast(), ERROR_SIZE) };
        if status == 0 {
            Ok(())
        } else {
            Err(message(&error))
        }
    }

    pub fn weather(&self, x: f32, z: f32, winter: bool) -> Result<Weather, String> {
        let mut error = [0u8; ERROR_SIZE];
        let mut modes = [0i32; 3];
        let mut transitional_state = 0.0f32;
        // SAFETY: `modes` has the three entries the kernel writes.
        let status = unsafe {
            env_ref_weather(
                self.raw,
                x,
                z,
                i32::from(winter),
                modes.as_mut_ptr(),
                &mut transitional_state,
                error.as_mut_ptr().cast(),
                ERROR_SIZE,
            )
        };
        if status == 0 {
            Ok(Weather {
                current_mode: modes[0],
                target_mode: modes[1],
                simple_mode: modes[2],
                transitional_state,
            })
        } else {
            Err(message(&error))
        }
    }

    pub fn wind(&self, x: f32, z: f32, want_vector: bool) -> Result<Wind, String> {
        let mut error = [0u8; ERROR_SIZE];
        let mut speed = 0.0f32;
        let mut vector = [0.0f32; 3];
        // SAFETY: `vector` has the three entries the kernel writes.
        let status = unsafe {
            env_ref_wind(
                self.raw,
                x,
                z,
                i32::from(want_vector),
                &mut speed,
                vector.as_mut_ptr(),
                error.as_mut_ptr().cast(),
                ERROR_SIZE,
            )
        };
        if status == 0 {
            Ok(Wind { speed, vector })
        } else {
            Err(message(&error))
        }
    }

    pub fn gust(phase: f32) -> Result<f32, String> {
        let mut error = [0u8; ERROR_SIZE];
        let mut modulation = 0.0f32;
        // SAFETY: as above.
        let status = unsafe { env_ref_gust(phase, &mut modulation, error.as_mut_ptr().cast(), ERROR_SIZE) };
        if status == 0 {
            Ok(modulation)
        } else {
            Err(message(&error))
        }
    }

    pub fn step(&mut self, input: &StepInput) -> Result<StepOutcome, String> {
        let mut error = [0u8; ERROR_SIZE];
        let in_i = [
            i32::from(input.winter),
            i32::from(input.server),
            i32::from(input.in_cockpit),
            i32::from(input.have_camera),
            i32::from(input.draw.is_some()),
        ];
        let mut out_i = [0i32; 2];
        let mut out_f = [0.0f32; 4];
        // The draw is passed by ADDRESS, built before the call from its bits (an integer move, no floating-point instruction in the glue): a by-value f64 is homed through the x87 stack by a
        // 32-bit callee before its guard has saved the caller's state (the actual disassembly of d4e5c34d2). The C side reads it only inside its guard, and only when have_draw is set.
        let draw_bits: u64 = input.draw.map_or(0, f64::to_bits);
        // SAFETY: the arrays have the lengths the kernel reads and writes; `draw_bits` is a readable 8-byte value, the bits of an f64, alive for the call.
        let status = unsafe {
            env_ref_step(
                self.raw,
                input.f.as_ptr(),
                in_i.as_ptr(),
                (&draw_bits as *const u64).cast::<f64>(),
                out_i.as_mut_ptr(),
                out_f.as_mut_ptr(),
                error.as_mut_ptr().cast(),
                ERROR_SIZE,
            )
        };
        if status != 0 {
            return Err(message(&error));
        }
        if out_i[0] != 0 {
            return Ok(StepOutcome::NeedDraw { timer: out_f[3] });
        }
        Ok(StepOutcome::Done {
            expired: out_i[1] != 0,
            strike: [out_f[0], out_f[1], out_f[2]],
            new_timer: out_f[3],
        })
    }
}

////////////////////////////////////////
//
// the Lua provider
//
////////////////////////////////////////

fn refuse(text: impl std::fmt::Display) -> LuaError {
    LuaError::runtime(format!("host.env: {text}"))
}

/// A request number with the exact bits Lua holds (see terrain.rs), or a refusal.
fn number(reader: &CoordinateReader, table: &LuaTable, key: impl IntoLua, what: &str) -> LuaResult<f64> {
    reader.read.call::<()>((table, key))?;
    if !reader.slot.get::<bool>("is_number")? {
        let kind: String = reader.slot.get("kind")?;
        return Err(refuse(format_args!("{what} must be a number, got {kind}")));
    }
    reader.slot.get("value")
}

/// A request number that is exactly a single-precision value (the campaign's own); a finite value that a float would round is refused. A non-finite value is passed on:
/// the kernels refuse it where the original could not have been given it, and answer it as the original does where it could (a position).
fn single(reader: &CoordinateReader, table: &LuaTable, key: impl IntoLua, what: &str) -> LuaResult<f32> {
    let value = number(reader, table, key, what)?;
    let narrowed = value as f32;
    if value.is_finite() && f64::from(narrowed) != value {
        return Err(refuse(format_args!("{what} {value} is not an exact single-precision value")));
    }
    Ok(narrowed)
}

fn integer(reader: &CoordinateReader, table: &LuaTable, key: impl IntoLua, what: &str) -> LuaResult<i32> {
    let value = number(reader, table, key, what)?;
    if value.fract() != 0.0 || value.abs() > f64::from(i32::MAX) {
        return Err(refuse(format_args!("{what} {value} is not an integer")));
    }
    Ok(value as i32)
}

fn boolean(table: &LuaTable, key: &str) -> LuaResult<bool> {
    match table.get::<LuaValue>(key)? {
        LuaValue::Boolean(value) => Ok(value),
        other => Err(refuse(format_args!("{key} must be a boolean, got {}", other.type_name()))),
    }
}

fn subtable(table: &LuaTable, key: &str) -> LuaResult<LuaTable> {
    match table.get::<LuaValue>(key)? {
        LuaValue::Table(inner) => Ok(inner),
        other => Err(refuse(format_args!("{key} must be a table, got {}", other.type_name()))),
    }
}

fn singles<const N: usize>(reader: &CoordinateReader, table: &LuaTable, names: &[&str; N], what: &str) -> LuaResult<[f32; N]> {
    let mut out = [0.0f32; N];
    for (slot, name) in out.iter_mut().zip(names.iter()) {
        *slot = single(reader, table, *name, &format!("{what}.{name}"))?;
    }
    Ok(out)
}

fn read_state(reader: &CoordinateReader, table: &LuaTable) -> LuaResult<State> {
    let mut state = State {
        f: [0.0; FLOATS],
        n: [0; INTS],
    };
    for (slot, name) in state.f.iter_mut().zip(FLOAT_NAMES.iter()) {
        *slot = single(reader, table, *name, &format!("state.{name}"))?;
    }
    for (index, (slot, name)) in state.n.iter_mut().zip(INT_NAMES.iter()).enumerate() {
        *slot = if index < 2 {
            integer(reader, table, *name, &format!("state.{name}"))?
        } else {
            i32::from(boolean(table, name)?)
        };
    }
    Ok(state)
}

fn state_table(lua: &Lua, state: &State) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    for (name, value) in FLOAT_NAMES.iter().zip(state.f.iter()) {
        table.set(*name, f64::from(*value))?;
    }
    for (index, (name, value)) in INT_NAMES.iter().zip(state.n.iter()).enumerate() {
        if index < 2 {
            table.set(*name, *value)?;
        } else {
            table.set(*name, *value != 0)?;
        }
    }
    Ok(table)
}

fn vector_table(lua: &Lua, values: &[f32; 3]) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    for (index, value) in values.iter().enumerate() {
        table.set(index + 1, f64::from(*value))?;
    }
    Ok(table)
}

fn answer_state(lua: &Lua, field: &Field) -> LuaResult<LuaTable> {
    let answer = lua.create_table()?;
    answer.set("state", state_table(lua, &field.export().map_err(refuse)?)?)?;
    Ok(answer)
}

fn respond(lua: &Lua, field: &RefCell<Field>, request: &LuaTable) -> LuaResult<LuaTable> {
    let op = match request.get::<LuaValue>("op")? {
        LuaValue::String(op) => op.to_str()?.to_string(),
        other => return Err(refuse(format_args!("op must be a string, got {}", other.type_name()))),
    };
    let reader = coordinate_reader(lua)?;
    let mut field = field.try_borrow_mut().map_err(|_| refuse("the field is already in use"))?;
    let answer = lua.create_table()?;
    match op.as_str() {
        "import" => {
            let state = read_state(&reader, &subtable(request, "state")?)?;
            field.import(&state).map_err(refuse)?;
        }
        "export" => return answer_state(lua, &field),
        "init_random" => {
            let draws = subtable(request, "draws")?;
            let mut draw = [0.0f64; DRAWS];
            for (index, slot) in draw.iter_mut().enumerate() {
                *slot = number(&reader, &draws, index + 1, &format!("draws[{}]", index + 1))?;
            }
            let config = singles(&reader, &subtable(request, "config")?, &CONFIG_NAMES, "config")?;
            let map = singles(&reader, &subtable(request, "map")?, &MAP_NAMES, "map")?;
            field.init_random(&draw, &config, &map).map_err(refuse)?;
            return answer_state(lua, &field);
        }
        "init_fixed" => {
            let mode = integer(&reader, request, "mode", "mode")?;
            field.init_fixed(mode).map_err(refuse)?;
            return answer_state(lua, &field);
        }
        "weather" => {
            let winter = boolean(request, "winter")?;
            let (x, z) = (single(&reader, request, "x", "x")?, single(&reader, request, "z", "z")?);
            let weather = field.weather(x, z, winter).map_err(refuse)?;
            answer.set("current_mode", weather.current_mode)?;
            answer.set("target_mode", weather.target_mode)?;
            answer.set("transitional_state", f64::from(weather.transitional_state))?;
            answer.set("simple_mode", weather.simple_mode)?;
        }
        "wind" => {
            let want_vector = boolean(request, "want_vector")?;
            let (x, z) = (single(&reader, request, "x", "x")?, single(&reader, request, "z", "z")?);
            let wind = field.wind(x, z, want_vector).map_err(refuse)?;
            answer.set("speed", f64::from(wind.speed))?;
            if want_vector {
                answer.set("vector", vector_table(lua, &wind.vector)?)?;
            }
        }
        "gust" => {
            let phase = single(&reader, request, "phase", "phase")?;
            answer.set("modulation", f64::from(Field::gust(phase).map_err(refuse)?))?;
        }
        "step" => {
            let mut f = [0.0f32; 10];
            f[0] = single(&reader, request, "delta", "delta")?;
            let have_camera = !matches!(request.get::<LuaValue>("camera")?, LuaValue::Nil);
            if have_camera {
                let camera = singles(&reader, &subtable(request, "camera")?, &["x", "y", "z"], "camera")?;
                f[1..4].copy_from_slice(&camera);
            }
            f[4..10].copy_from_slice(&singles(&reader, &subtable(request, "map")?, &MAP_NAMES, "map")?);
            let draw = match request.get::<LuaValue>("lightning_draw")? {
                LuaValue::Nil => None,
                _ => Some(number(&reader, request, "lightning_draw", "lightning_draw")?),
            };
            let input = StepInput {
                f,
                winter: boolean(request, "winter")?,
                server: boolean(request, "server")?,
                in_cockpit: boolean(request, "in_cockpit")?,
                have_camera,
                draw,
            };
            match field.step(&input).map_err(refuse)? {
                StepOutcome::NeedDraw { timer } => {
                    answer.set("need_draw", true)?;
                    answer.set("timer", f64::from(timer))?;
                }
                StepOutcome::Done { expired, strike, new_timer } => {
                    answer.set("state", state_table(lua, &field.export().map_err(refuse)?)?)?;
                    answer.set("expired", expired)?;
                    if expired {
                        answer.set("strike", vector_table(lua, &strike)?)?;
                        answer.set("new_timer", f64::from(new_timer))?;
                    }
                }
            }
        }
        other => return Err(refuse(format_args!("unknown op {other:?}"))),
    }
    Ok(answer)
}

/// A new provider function with a field of its own (empty until the campaign imports its session's defaults).
pub fn provider(lua: &Lua) -> LuaResult<LuaFunction> {
    let field = Rc::new(RefCell::new(Field::new().map_err(LuaError::runtime)?));
    lua.create_function(move |lua, request: LuaTable| respond(lua, &field, &request))
}

/// Installs `host.env_provider ()`: the reference host's environment provider, to be given to `dc.boot` as `environment = ...`. Each call makes a new field; it is a shared helper
/// for the reference host's scripts, not a default: the module never takes a provider from the host's globals, and a boot without `environment` is refused.
pub fn install(lua: &Lua, host: &LuaTable) -> LuaResult<()> {
    host.set("env_provider", lua.create_function(|lua, ()| provider(lua))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> State {
        let mut f = [0.0f32; FLOATS];
        f[1] = 120.0; // the transitional period
        f[11] = 1.0; // the minimum wind speed
        f[12] = 4.0;
        State { f, n: [1, 1, 0, 0, 0] }
    }

    fn map() -> [f32; 6] {
        [-1000.0, 1000.0, -1000.0, 1000.0, 0.0, 0.0]
    }

    fn input(delta: f32) -> StepInput {
        let m = map();
        StepInput {
            f: [delta, 0.0, 0.0, 0.0, m[0], m[1], m[2], m[3], m[4], m[5]],
            winter: false,
            server: true,
            in_cockpit: false,
            have_camera: true,
            draw: None,
        }
    }

    fn lua_with_provider() -> (Lua, LuaFunction) {
        let lua = Lua::new();
        let host = lua.create_table().unwrap();
        install(&lua, &host).unwrap();
        let make: LuaFunction = host.get("env_provider").unwrap();
        let provider: LuaFunction = make.call(()).unwrap();
        (lua, provider)
    }

    fn state_chunk(lua: &Lua, state: &State) -> LuaTable {
        state_table(lua, state).unwrap()
    }

    #[test]
    fn a_field_with_no_state_refuses_every_question() {
        let field = Field::new().unwrap();
        assert!(field.export().unwrap_err().contains("no state"));
        assert!(field.weather(0.0, 0.0, false).is_err());
        assert!(field.wind(0.0, 0.0, true).is_err());
    }

    #[test]
    fn the_gust_modulation_is_the_originals_piecewise_table() {
        assert_eq!(Field::gust(0.0).unwrap(), 0.0);
        // Exactly representable (dyadic) phases only. 1/7 is not one: the float nearest 1/7 times 7 is 1.0000000447, which the original's x87 build keeps in an extended register
        // (floor 1, ceil 2, so it interpolates a hair towards the next sample: 0.24999999), while an SSE build rounds the product to 1.0 (0.25); both are the original arithmetic, so no
        // exact value can be asserted there. A dyadic phase makes phase * 7, the segment, the fraction and the result exact in any precision and either unit.
        // Phase 1/8: val = 0.875, between the implicit 0 before the first sample and the first sample (0.25): 0 + 0.875 * 0.25 = 0.21875.
        assert_eq!(Field::gust(0.125).unwrap().to_bits(), 0x3e60_0000);
        // Phase 1/16: val = 0.4375: 0.4375 * 0.25 = 0.109375.
        assert_eq!(Field::gust(0.0625).unwrap().to_bits(), 0x3de0_0000);
        for refused in [-0.5f32, 1.0, 2.0, f32::NAN] {
            assert!(Field::gust(refused).is_err(), "{refused}");
        }
    }

    #[test]
    fn a_fixed_weather_is_global_and_a_refused_mode_changes_nothing() {
        let mut field = Field::new().unwrap();
        field.import(&defaults()).unwrap();
        field.init_fixed(3).unwrap();
        let weather = field.weather(10.0, -20.0, false).unwrap();
        assert_eq!((weather.current_mode, weather.target_mode, weather.simple_mode), (3, 3, 3));
        let before = field.export().unwrap();
        assert!(field.init_fixed(9).is_err());
        assert_eq!(field.export().unwrap(), before);
    }

    #[test]
    fn an_import_that_is_not_a_field_is_refused_whole() {
        let mut field = Field::new().unwrap();
        field.import(&defaults()).unwrap();
        let before = field.export().unwrap();
        let mut nan = defaults();
        nan.f[3] = f32::NAN;
        assert!(field.import(&nan).is_err());
        let mut mode = defaults();
        mode.n[0] = 5;
        assert!(field.import(&mode).is_err());
        let mut flag = defaults();
        flag.n[3] = 2;
        assert!(field.import(&flag).is_err());
        assert_eq!(field.export().unwrap(), before);
    }

    /// The processor's floating-point state of this thread, read and written directly -- the x87 control, status AND tag words and the whole of MXCSR -- so that the guard around every
    /// kernel is checked against the REAL registers, in full, rather than against one quotient or a masked part of a word. Only where those registers exist.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    mod fp {
        use std::arch::asm;

        /// The complete state the guard promises to give back (the x87 words are the full words, no mask).
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct Env {
            pub mxcsr: u32,
            pub control: u16,
            pub status: u16,
            pub tag: u16,
        }

        pub fn mxcsr() -> u32 {
            let mut value = 0u32;
            // SAFETY: stores the 32-bit MXCSR to a local.
            unsafe { asm!("stmxcsr dword ptr [{}]", in(reg) &mut value as *mut u32, options(nostack, preserves_flags)) };
            value
        }

        pub fn set_mxcsr(value: u32) {
            // SAFETY: loads MXCSR from a local; the values written are built on a prior read with valid fields.
            unsafe { asm!("ldmxcsr dword ptr [{}]", in(reg) &value as *const u32, options(nostack, readonly)) };
        }

        /// The x87 environment image (fnstenv: control, status and tag words, the exception pointers). fnstenv masks every exception, so the image is loaded straight back.
        pub fn x87_image() -> [u8; 28] {
            let mut image = [0u8; 28];
            // SAFETY: stores the 28-byte environment to a local and loads the same image back.
            unsafe {
                asm!("fnstenv [{}]", in(reg) image.as_mut_ptr(), options(nostack));
                asm!("fldenv [{}]", in(reg) image.as_ptr(), options(nostack, readonly));
            }
            image
        }

        pub fn set_x87_image(image: &[u8; 28]) {
            // SAFETY: loads an environment image built from a prior read with valid words.
            unsafe { asm!("fldenv [{}]", in(reg) image.as_ptr(), options(nostack, readonly)) };
        }

        pub fn now() -> Env {
            let image = x87_image();
            Env {
                mxcsr: mxcsr(),
                control: u16::from_le_bytes([image[0], image[1]]),
                status: u16::from_le_bytes([image[4], image[5]]),
                tag: u16::from_le_bytes([image[8], image[9]]),
            }
        }

        /// Arranges a caller state deliberately: the x87 rounding control, a full status word and an all-empty tag word, and MXCSR with its own rounding control and sticky flags.
        pub fn seed(x87_rounding: u16, status: u16, sse_rounding: u32, sticky: u32) {
            let mut image = x87_image();
            let control = (u16::from_le_bytes([image[0], image[1]]) & !0x0c00) | (x87_rounding << 10);
            image[0..2].copy_from_slice(&control.to_le_bytes());
            image[4..6].copy_from_slice(&status.to_le_bytes());
            image[8..10].copy_from_slice(&0xffffu16.to_le_bytes());
            set_x87_image(&image);
            set_mxcsr(0x1f80 | (sse_rounding << 13) | sticky);
        }
    }

    /// Puts the thread's COMPLETE floating-point environment back whatever a test did to it, even when an assertion fails.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    struct RestoreFp {
        image: [u8; 28],
        mxcsr: u32,
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    impl Drop for RestoreFp {
        fn drop(&mut self) {
            fp::set_x87_image(&self.image);
            fp::set_mxcsr(self.mxcsr);
        }
    }

    /// Every operation, accepted and refused, under every pair of rounding controls (the four of the x87 unit against the four of the SSE unit, so the mixed pairs too), with MXCSR's sticky
    /// flags clear and set and with three deliberately arranged x87 status words (clear; the condition code C3 and every exception flag; the stack top 4 and the codes C0, C1 and C2):
    /// the caller's complete x87 control, status and tag words and complete MXCSR are exactly what the test seeded -- no mask -- after each call. The operations include
    /// `init_random`, the first `NeedDraw` answer and the final `Done` after the draw, the simple-mode weather answer and the wind with and without a vector, and a refusal of each kind.
    /// And whatever the caller's state, the kernels' answers are the same bits (they run in their own mode, round toward zero).
    ///
    /// What this does not cover: the Rust code around the call (building arguments, comparing results) is the test's own and runs in the seeded state too; on an x87-only target
    /// it can itself touch the status word, which this test would then report. Source only: Root runs it on the actual targets.
    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn the_callers_complete_floating_point_state_survives_every_operation_in_every_mode() {
        let _restore = RestoreFp {
            image: fp::x87_image(),
            mxcsr: fp::mxcsr(),
        };
        let mut local = defaults();
        local.n[2] = 1;
        local.f[0] = 20000.0;
        local.f[3] = 123.456;
        local.f[6] = 7.77;
        local.f[22] = 100.0;
        let mut expiring = local;
        expiring.f[22] = 1.0;
        let mut bad_gust = local;
        bad_gust.f[10] = 2.0;
        let mut nan = local;
        nan.f[3] = f32::NAN;
        let draws: [f64; DRAWS] = [0.5, 1.0, 0.25, 0.75, 0.5, 0.5, 0.0, 0.25, 0.75, -0.5];
        let mut out_of_range = draws;
        out_of_range[0] = 2.0;
        let config: [f32; 6] = [20000.0, 60000.0, 10.0, 8000.0, 40000.0, 5.0];
        let map_extents = map();
        let mut with_draw = input(2.0);
        with_draw.draw = Some(0.5);
        let mut bad_draw = with_draw;
        bad_draw.draw = Some(f64::NAN);
        let mut blind = input(1.0);
        blind.have_camera = false;

        type Answers = (State, Weather, Wind, f32, State, StepOutcome, StepOutcome);
        let mut answers: Vec<Answers> = Vec::new();
        for x87_rounding in 0u16..4 {
            for sse_rounding in 0u32..4 {
                for sticky in [0u32, 0x3f] {
                    for status in [0x0000u16, 0x403f, 0x2700] {
                        fp::seed(x87_rounding, status, sse_rounding, sticky);
                        let expected = fp::now();
                        let check = |what: &str| {
                            assert_eq!(
                                fp::now(),
                                expected,
                                "{what}: x87 rounding {x87_rounding}, SSE rounding {sse_rounding}, MXCSR sticky {sticky:#x}, x87 status {status:#06x}"
                            );
                        };
                        let mut field = Field::new().unwrap();
                        field.import(&local).unwrap();
                        check("import");
                        field.export().unwrap();
                        check("export");
                        let weather = field.weather(9000.0, -3000.0, false).unwrap();
                        check("weather (with the simple-mode choice)");
                        let wind = field.wind(1.0, 2.0, true).unwrap();
                        check("wind with a vector");
                        field.wind(1.0, 2.0, false).unwrap();
                        check("wind without a vector");
                        let gust = Field::gust(0.5).unwrap();
                        check("gust");
                        // the first answer of an expiring server timer, and the final one after the draw
                        field.import(&expiring).unwrap();
                        let need = field.step(&input(2.0)).unwrap();
                        check("step: the first question (need_draw)");
                        assert!(matches!(need, StepOutcome::NeedDraw { .. }), "{need:?}");
                        let done = field.step(&with_draw).unwrap();
                        check("step: the final answer after the draw");
                        assert!(matches!(done, StepOutcome::Done { expired: true, .. }), "{done:?}");
                        field.import(&local).unwrap();
                        field.step(&input(0.1)).unwrap();
                        check("step: no expiry");
                        let stepped = field.export().unwrap();
                        // init_random: accepted, and refused (a draw the original's macro cannot give)
                        field.init_random(&draws, &config, &map_extents).unwrap();
                        check("init_random");
                        let initialised = field.export().unwrap();
                        assert!(field.init_random(&out_of_range, &config, &map_extents).is_err());
                        check("init_random refused");
                        field.init_fixed(2).unwrap();
                        check("init_fixed");
                        // the refusals, each of which runs part of the validation under the guard
                        assert!(field.init_fixed(9).is_err());
                        check("init_fixed refused");
                        assert!(field.import(&nan).is_err());
                        check("import refused");
                        assert!(Field::gust(2.0).is_err());
                        check("gust refused");
                        field.import(&bad_gust).unwrap();
                        assert!(field.wind(1.0, 2.0, true).is_err());
                        check("wind refused (the gust phase is outside the original's domain)");
                        field.import(&local).unwrap();
                        assert!(field.step(&blind).is_err());
                        check("step refused (no camera)");
                        assert!(field.step(&input(f32::INFINITY)).is_err());
                        check("step refused (infinite delta)");
                        field.import(&expiring).unwrap();
                        assert!(field.step(&bad_draw).is_err());
                        check("step refused (a draw the original's macro cannot give)");
                        assert!(Field::new().unwrap().export().is_err());
                        check("export refused (no state)");
                        let again = field.step(&with_draw).unwrap();
                        check("step: the final answer again");
                        answers.push((stepped, weather, wind, gust, initialised, need, again));
                    }
                }
            }
        }
        for answer in answers.iter().skip(1) {
            assert_eq!(answer, &answers[0], "the caller's state reached the kernels");
        }
    }

    #[test]
    fn a_step_decrements_a_server_timer_and_asks_for_the_one_draw_without_changing_anything() {
        let mut state = defaults();
        state.n[2] = 1; // local weather model
        state.f[0] = 20000.0; // weather radius
        state.f[22] = 1.0; // lightning timer
        state.f[6] = 1.0;
        let mut field = Field::new().unwrap();
        field.import(&state).unwrap();
        let before = field.export().unwrap();
        match field.step(&input(2.0)).unwrap() {
            StepOutcome::NeedDraw { timer } => assert_eq!(timer, -1.0),
            other => panic!("{other:?}"),
        }
        assert_eq!(field.export().unwrap(), before);
        let mut with_draw = input(2.0);
        with_draw.draw = Some(0.5);
        match field.step(&with_draw).unwrap() {
            StepOutcome::Done { expired, strike, new_timer } => {
                assert!(expired);
                assert_eq!(strike, [0.0, 0.0, 0.0]);
                // 20 s + (240 s - 20 s) * 0.5, with the original's unparenthesised macros: 20*1 + ((4*60 - 20*1) * 0.5)
                assert_eq!(new_timer, 20.0 + (240.0 - 20.0) * 0.5);
            }
            other => panic!("{other:?}"),
        }
        let after = field.export().unwrap();
        assert_eq!(after.f[22], 20.0 + (240.0 - 20.0) * 0.5);
        assert_eq!(
            after.f[3], 2.0,
            "the weather moved by its velocity times the delta, after the strike took the old centre"
        );
    }

    #[test]
    fn a_client_timer_only_runs_while_non_zero_and_expires_without_a_draw() {
        let mut state = defaults();
        state.n[2] = 1;
        state.f[0] = 20000.0;
        state.f[22] = 1.0;
        let mut field = Field::new().unwrap();
        field.import(&state).unwrap();
        let mut client = input(2.0);
        client.server = false;
        match field.step(&client).unwrap() {
            StepOutcome::Done { expired, new_timer, .. } => {
                assert!(expired);
                assert_eq!(new_timer, 0.0);
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(field.export().unwrap().f[22], 0.0);
    }

    #[test]
    fn a_weather_zone_with_no_camera_refuses_the_step_and_changes_nothing() {
        let mut state = defaults();
        state.n[2] = 1;
        state.f[0] = 20000.0;
        let mut field = Field::new().unwrap();
        field.import(&state).unwrap();
        let before = field.export().unwrap();
        let mut blind = input(1.0);
        blind.have_camera = false;
        assert!(field.step(&blind).unwrap_err().contains("camera"));
        assert_eq!(field.export().unwrap(), before);
    }

    // ---- the original's branch-specific reads and its gust wrap (review ENV-PHYSICAL-3): nothing is refused that the original completes ----

    fn nan_camera(y_only: bool) -> StepInput {
        let mut step = input(1.0);
        step.f[1..4].copy_from_slice(&if y_only { [0.0, f32::NAN, 0.0] } else { [f32::NAN, f32::NAN, f32::NAN] });
        step
    }

    /// A local weather zone whose centre is far outside its radius of the camera at the origin (a finite camera sees the dry zone), the lightning far away.
    fn far_zone() -> State {
        let mut zone = defaults();
        zone.n[2] = 1;
        zone.f[0] = 20000.0;
        zone.f[3] = 50000.0;
        zone.f[5] = 50000.0;
        zone.f[22] = 1.0e6;
        zone
    }

    #[test]
    fn a_camera_the_original_does_not_read_is_not_refused_but_a_missing_required_one_is() {
        // a local model with a zone reads the camera's x and z: a not-finite y (never read) is no reason to refuse, and the finite x and z are what answers (the dry zone, mode 1)
        let mut field = Field::new().unwrap();
        field.import(&far_zone()).unwrap();
        field.step(&nan_camera(true)).unwrap();
        let after = field.export().unwrap();
        assert_eq!((after.n[0], after.n[1], after.f[2]), (1, 1, 0.0));
        // a not-finite x and z ARE read, and the original completes: the range is not a number, no ring comparison holds, the heavy-rain branch answers (summer), the field stays finite
        let mut field = Field::new().unwrap();
        field.import(&far_zone()).unwrap();
        field.step(&nan_camera(false)).unwrap();
        let after = field.export().unwrap();
        assert_eq!((after.n[0], after.n[1], after.f[2]), (3, 3, 0.0));
        // the required camera absent: refused, and nothing changed
        let mut field = Field::new().unwrap();
        field.import(&far_zone()).unwrap();
        let before = field.export().unwrap();
        let mut blind = input(1.0);
        blind.have_camera = false;
        assert!(field.step(&blind).unwrap_err().contains("camera"));
        assert_eq!(field.export().unwrap(), before);
        // the global model, and a local model with no zone, read no camera at all
        for local_without_zone in [false, true] {
            let mut calm = defaults();
            calm.n[2] = i32::from(local_without_zone);
            let mut field = Field::new().unwrap();
            field.import(&calm).unwrap();
            let mut step = nan_camera(false);
            field.step(&step).unwrap();
            step.have_camera = false;
            if local_without_zone {
                // the first step evolved the zero-radius zone by the original's own arithmetic (0 - 100 is below the minimum, so the radius is the minimum and grows from there); a zone with a radius
                // DOES need the camera, so the same step without one is now, legitimately, refused -- and changes nothing
                let evolved = field.export().unwrap();
                assert_eq!((evolved.f[0], evolved.n[3]), (5000.0, 1), "the original's radius clamp");
                assert!(field.step(&step).unwrap_err().contains("camera"));
                assert_eq!(field.export().unwrap(), evolved);
            }
            // the independent assertion: the calm state again, and a step with NO camera at all completes (no zone, or the global model: the original reads none)
            field.import(&calm).unwrap();
            field.step(&step).unwrap();
        }
    }

    #[test]
    fn a_gust_phase_the_original_wraps_is_wrapped_not_refused_before_its_arithmetic() {
        let phase_after = |gust: f32, delta: f32| {
            let mut state = defaults();
            state.f[10] = gust;
            let mut field = Field::new().unwrap();
            field.import(&state).unwrap();
            field.step(&input(delta))?;
            Ok::<f32, String>(field.export()?.f[10])
        };
        // a phase set below zero reaches a valid one in the update, as in the original (-0.25 + 5 * 0.1)
        assert_eq!(phase_after(-0.25, 5.0), Ok(0.25));
        // an ordinary wrap, and a phase far beyond the old cutoff of a million: the original's loop subtracts one until it is below one
        let wrapped = phase_after(0.95, 1.0).unwrap();
        assert!((0.0..0.06).contains(&wrapped) && (wrapped - 0.05).abs() < 1.0e-5, "{wrapped}");
        assert_eq!(phase_after(2.0e6, 0.0), Ok(0.0));
        // a phase that stays outside [0, 1) is a field the wind query then refuses, as the original's assertion does
        assert_eq!(phase_after(-5.0, 0.0), Ok(-5.0));
        // the one unsupported condition: the update would make the phase infinite, which the original's wrap could never leave
        assert!(phase_after(0.5, f32::INFINITY).unwrap_err().contains("infinite"));
        // a not-a-number delta gives a not-finite field, which is refused whole
        assert!(phase_after(0.5, f32::NAN).is_err());
    }

    #[test]
    fn a_negative_delta_is_arithmetic_not_a_refusal() {
        let mut state = defaults();
        state.f[16] = 10.0;
        state.f[19] = 2.0;
        let mut field = Field::new().unwrap();
        field.import(&state).unwrap();
        field.step(&input(-1.0)).unwrap();
        assert_eq!(field.export().unwrap().f[16], 8.0);
    }

    #[test]
    fn an_infinite_delta_that_would_ask_for_a_draw_is_refused_before_the_early_success_and_commits_nothing() {
        // a server's zone whose timer an infinite delta makes -infinity: the early need_draw success used to bypass the final check of the field and report the infinite timer as the
        // protocol's number
        let mut zone = defaults();
        zone.n[2] = 1;
        zone.f[0] = 20000.0;
        zone.f[22] = 1.0;
        let mut field = Field::new().unwrap();
        field.import(&zone).unwrap();
        let before = field.export().unwrap();
        let error = field.step(&input(f32::INFINITY)).unwrap_err();
        assert!(error.contains("not finite"), "{error}");
        assert_eq!(field.export().unwrap(), before);
        // a finite delta of the same zone still asks for the draw, with a finite timer
        match field.step(&input(2.0)).unwrap() {
            StepOutcome::NeedDraw { timer } => assert_eq!(timer, -1.0),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn the_provider_reads_a_negative_zero_and_refuses_what_a_float_would_round() {
        let (lua, provider) = lua_with_provider();
        let mut state = defaults();
        state.f[16] = -0.0; // the wind zone's position
        state.f[19] = 1.0;
        let request = lua.create_table().unwrap();
        request.set("op", "import").unwrap();
        request.set("state", state_chunk(&lua, &state)).unwrap();
        provider.call::<LuaTable>(request).unwrap();
        // a step of a negative-zero delta leaves a negative-zero position negative (-0 + 1 * -0)
        let step = lua.create_table().unwrap();
        step.set("op", "step").unwrap();
        step.set("winter", false).unwrap();
        step.set("server", true).unwrap();
        step.set("in_cockpit", false).unwrap();
        step.set("delta", -0.0f64).unwrap();
        let map_table = lua.create_table().unwrap();
        for (name, value) in MAP_NAMES.iter().zip(map().iter()) {
            map_table.set(*name, f64::from(*value)).unwrap();
        }
        step.set("map", map_table).unwrap();
        let answer: LuaTable = provider.call(step.clone()).unwrap();
        let after: LuaTable = answer.get("state").unwrap();
        let x: f64 = after.get("wind_effect_position_x").unwrap();
        assert!(x == 0.0 && x.is_sign_negative(), "{x}");
        // a delta no float holds is refused, with the state unchanged
        step.set("delta", 0.1f64).unwrap();
        let error = provider.call::<LuaTable>(step).unwrap_err().to_string();
        assert!(error.contains("exact single-precision"), "{error}");
    }

    #[test]
    fn the_provider_refuses_a_state_table_with_a_string_number_or_a_numeric_flag() {
        let (lua, provider) = lua_with_provider();
        let make = |mutate: &dyn Fn(&LuaTable)| {
            let request = lua.create_table().unwrap();
            request.set("op", "import").unwrap();
            let table = state_chunk(&lua, &defaults());
            mutate(&table);
            request.set("state", table).unwrap();
            provider.call::<LuaTable>(request)
        };
        assert!(make(&|t| t.set("weather_radius", "5").unwrap()).is_err());
        assert!(make(&|t| t.set("local_weather_model", 1).unwrap()).is_err());
        assert!(make(&|t| t.set("weather_mode", 1.5).unwrap()).is_err());
        assert!(make(&|_| {}).is_ok());
    }
}
